//! The root's shared utilities — `packages/react/src/combobox/root/utils/index.ts:1-68`
//! and `constants.ts:1-5` ported, plus the filter composition from
//! `useFilter.ts:33-49`.
//!
//! Upstream shape:
//! - [`get_combobox_popup_id`] (`index.ts:13-15`): the popup-id convention shared by
//!   the popup (writer) and the trigger (`aria-controls` reader), so it lives in one
//!   place — `` `${rootId}-popup` ``.
//! - [`create_collator_item_filter`] (`index.ts:23-34`): the nullish-rejecting wrapper
//!   over the collator filter's `contains`.
//! - [`create_single_selection_collator_filter`] (`index.ts:40-68`): shows all items
//!   while the query is empty, and while the query exactly matches the selected
//!   value's label (`selectedString.length === query.length` after the collator
//!   containment probe) — the single-selection browse affordance.
//! - [`use_combobox_filter`]'s composition (`useFilter.ts:33-49`): multiple mode wraps
//!   the collator filter; single mode wraps the selection-aware one seeded with the
//!   current value. The React memoization is a render-phase artifact — Leptos
//!   components run once, so the port returns the composed closure directly.
//! - [`NO_ACTIVE_VALUE`] / `INITIAL_LAST_HIGHLIGHT` (`constants.ts:1-5`): the
//!   highlight-dedup sentinels `setIndices`/`emitHighlight` use. Upstream
//!   `NO_ACTIVE_VALUE` is a `Symbol('none')` — a value no real item can carry; the
//!   port represents it as the string encoding no JSON value produces (documented at
//!   the constant).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The unit's items/values are the crate's dynamic [`Value`] representation (the
//!   `resolve_value_label` precedent), but the ported core filter
//!   (`leptos_ui_internals::filter`) is the wasm-bound `JsValue` engine (its module
//!   docs: the `Intl.Collator` machinery exists only in JS). The seam is
//!   [`json_value_to_js`]: a structural conversion — strings/numbers/booleans/null
//!   direct, arrays/objects via `js_sys` — so the `{ value }`/`{ label }` field
//!   detection inside `stringify_as_label_js` sees the same shapes upstream does.
//! - The label projection adapters (`FilterItemToString`) run on the dynamic side and
//!   are wrapped into the `JsValue`-taking closure the filter expects.
//! - The upstream `index.ts` filters receive raw `item` values; the port's wrappers
//!   take [`Value`] and convert at the seam.

use std::rc::Rc;

use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use leptos_ui_internals::filter::{Filter, GetFilterOptions, get_filter};

/// The default id assigned to `Combobox.Popup` when the input is rendered inside it
/// (`index.ts:13-15`), shared by the popup (which applies it) and the trigger (which
/// references it via `aria-controls`).
pub fn get_combobox_popup_id(root_id: Option<&str>) -> Option<String> {
    root_id.map(|id| format!("{id}-popup"))
}

/// The item-to-string adapter — upstream `FilterItemToString` (`index.ts:5-7`): the
/// label projection, optionally with a `selected` variant for the selected-value
/// stringification.
#[derive(Clone, Default)]
pub struct FilterItemToString {
    pub to_string: Option<Rc<dyn Fn(&Value) -> String>>,
    pub selected: Option<Rc<dyn Fn(&Value) -> String>>,
}

/// `NO_ACTIVE_VALUE` (`constants.ts:1`): the sentinel no real item can carry. The
/// upstream `Symbol('none')` has no JSON encoding; the port pins this string — it
/// begins with a NUL, which no JSON-encodable real value stringifies to at the label
/// positions the dedup compares.
pub const NO_ACTIVE_VALUE: &str = "\u{0}base-ui:none";

/// `INITIAL_LAST_HIGHLIGHT`'s index (`constants.ts:3-5`): the pre-first-highlight
/// state of the dedup pair. The sentinel `value` is [`NO_ACTIVE_VALUE`].
pub const INITIAL_LAST_HIGHLIGHT_INDEX: i64 = -1;

/// The structural `Value → JsValue` conversion at the filter seam (see the module
/// adaptation notes). JSON's six shapes map one-to-one onto JS values.
pub fn json_value_to_js(value: &Value) -> JsValue {
    match value {
        Value::Null => JsValue::NULL,
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsValue::from_f64(i as f64)
            } else {
                JsValue::from_f64(n.as_f64().unwrap_or(f64::NAN))
            }
        }
        Value::String(s) => JsValue::from_str(s),
        Value::Array(items) => {
            let array = js_sys::Array::new();
            for item in items {
                array.push(&json_value_to_js(item));
            }
            array.into()
        }
        Value::Object(map) => {
            let object = js_sys::Object::new();
            for (key, field) in map {
                js_sys::Reflect::set(&object, &JsValue::from_str(key), &json_value_to_js(field))
                    .expect("set a converted object field");
            }
            object.into()
        }
    }
}

/// Wraps a dynamic-side adapter into the `JsValue`-taking closure the filter expects.
fn adapter_to_js(adapter: Option<&FilterItemToString>) -> Option<impl Fn(&JsValue) -> String + '_> {
    let to_string = adapter.and_then(|a| a.to_string.clone());
    to_string.map(move |project| move |value: &JsValue| project(&js_value_to_json(value)))
}

/// The inverse seam: the filter's callbacks hand back `JsValue`s (the raw item), which
/// the dynamic-side adapters read. Structural `JsValue → Value` over the JSON shapes
/// (non-JSON JS values — functions, symbols — do not occur: items are data).
pub fn js_value_to_json(value: &JsValue) -> Value {
    if value.is_null() || value.is_undefined() {
        return Value::Null;
    }
    if let Some(string) = value.as_string() {
        return Value::String(string);
    }
    if let Some(boolean) = value.as_bool() {
        return Value::Bool(boolean);
    }
    if let Some(number) = value.as_f64() {
        return serde_json::Number::from_f64(number)
            .map(Value::Number)
            .unwrap_or(Value::Null);
    }
    if value.is_instance_of::<js_sys::Array>() {
        let array: &js_sys::Array = value.unchecked_ref();
        return Value::Array(array.iter().map(|item| js_value_to_json(&item)).collect());
    }
    if value.is_instance_of::<js_sys::Object>() {
        let object: &js_sys::Object = value.unchecked_ref();
        let mut map = serde_json::Map::new();
        let entries = js_sys::Object::entries(object);
        for entry in entries.iter() {
            let pair: js_sys::Array = entry.unchecked_into();
            let key = pair.get(0).as_string().unwrap_or_default();
            map.insert(key, js_value_to_json(&pair.get(1)));
        }
        return Value::Object(map);
    }
    Value::Null
}

/// `createCollatorItemFilter` (`index.ts:23-34`): rejects nullish items, then probes
/// the collator containment.
pub fn create_collator_item_filter(
    collator_filter: Filter,
    item_to_string_label: Option<FilterItemToString>,
) -> impl Fn(&Value, &str) -> bool {
    move |item: &Value, query: &str| {
        if item.is_null() {
            return false;
        }

        let item_js = json_value_to_js(item);
        let adapter = adapter_to_js(item_to_string_label.as_ref());
        collator_filter.contains(
            &item_js,
            query,
            adapter.as_ref().map(|f| f as &dyn Fn(&JsValue) -> String),
        )
    }
}

/// `createSingleSelectionCollatorFilter` (`index.ts:40-68`): empty query shows
/// everything; a query that the selected value's label contains exactly (same
/// length) shows everything too; otherwise the item filters normally.
pub fn create_single_selection_collator_filter(
    collator_filter: Filter,
    item_to_string_label: Option<FilterItemToString>,
    selected_value: Option<Value>,
) -> impl Fn(&Value, &str) -> bool {
    move |item: &Value, query: &str| {
        if item.is_null() {
            return false;
        }
        if query.is_empty() {
            return true;
        }

        // `itemToStringLabel?.selected ?? itemToStringLabel` (`index.ts:47`).
        let selected_adapter = item_to_string_label
            .as_ref()
            .map(|adapter| FilterItemToString {
                to_string: adapter
                    .selected
                    .clone()
                    .or_else(|| adapter.to_string.clone()),
                selected: None,
            });
        let selected_string = match selected_value.as_ref() {
            Some(value) if !value.is_null() => {
                let value_js = json_value_to_js(&value);
                let adapter = adapter_to_js(selected_adapter.as_ref());
                leptos_ui_internals::filter::stringify_as_label_js(
                    &value_js,
                    adapter.as_ref().map(|f| f as &dyn Fn(&JsValue) -> String),
                )
            }
            _ => String::new(),
        };

        // Handle case-insensitive matching consistently (`index.ts:56-62`).
        if !selected_string.is_empty()
            && collator_filter.contains(&JsValue::from_str(&selected_string), query, None)
            && selected_string.chars().count() == query.chars().count()
        {
            return true;
        }

        let item_js = json_value_to_js(item);
        let adapter = adapter_to_js(item_to_string_label.as_ref());
        collator_filter.contains(
            &item_js,
            query,
            adapter.as_ref().map(|f| f as &dyn Fn(&JsValue) -> String),
        )
    }
}

/// The options — upstream `UseComboboxFilterOptions` (`useFilter.ts:12-26`): the
/// collator options plus the combobox additions. The port carries the fields the
/// composition reads (`multiple`, `value`) plus the locale passthrough the core
/// filter takes.
#[derive(Default)]
pub struct UseComboboxFilterOptions {
    pub locale: Option<String>,
    /// Whether the combobox is in multiple selection mode (`useFilter.ts:18-20`).
    pub multiple: bool,
    /// The current value of the combobox, used to keep every item visible while the
    /// query still matches the selection (`useFilter.ts:22-25`).
    pub value: Option<Value>,
}

/// The `Filter` shape the combobox parts consume (`useFilter.ts:28-32`): the combobox
/// mode-aware `contains` — the only predicate the combobox code path exercises.
pub struct ComboboxFilter {
    pub contains: Rc<dyn Fn(&Value, &str) -> bool>,
}

/// `useComboboxFilter` (`useFilter.ts:33-49`): wraps the core collator filter with
/// the mode-aware `contains` — multiple mode the plain collator wrapper, single mode
/// the selection-aware one seeded with the current value.
pub fn use_combobox_filter(options: UseComboboxFilterOptions) -> ComboboxFilter {
    let core_filter = get_filter(GetFilterOptions {
        locale: options.locale.as_ref().map(|l| JsValue::from_str(l)),
        collator_options: None,
    });
    let value = options.value;

    let contains: Rc<dyn Fn(&Value, &str) -> bool> = if options.multiple {
        let filter = create_collator_item_filter(core_filter.clone(), None);
        Rc::new(move |item, query| filter(item, query))
    } else {
        let filter =
            create_single_selection_collator_filter(core_filter.clone(), None, value.clone());
        Rc::new(move |item, query| filter(item, query))
    };

    ComboboxFilter { contains }
}
