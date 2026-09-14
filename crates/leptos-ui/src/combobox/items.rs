//! The derived-items pipeline — `packages/react/src/combobox/items/itemCollection.ts:1-67`
//! and `createItems.ts:98-163` ported over the crate's dynamic [`Value`] representation
//! and the ported `resolve_value_label` / `item_equality` internals.
//!
//! Upstream shape:
//! - [`find_collection_item`] ports `findCollectionItem` (`itemCollection.ts:8-25`): an
//!   exact map hit wins; with a custom comparer (not `defaultItemEquality`) it falls
//!   back to a linear scan comparing each derived value; `defaultItemEquality`
//!   short-circuits the scan since the map key already encodes the equality.
//! - The opaque `ComboboxItemCollection` brand and the `RejectGroupShapedItems` compile
//!   guards (`createItems.ts:21-48`) are TypeScript soundness devices with no runtime
//!   behavior — the implementation spec's "Anything in source" item 12 records the
//!   compile-time group-shape rejection as having no runtime counterpart beyond the
//!   collection-shape throw (which belongs to the root and is ported there). The port
//!   ships the runtime [`ItemCollection`] shape only.
//! - `createComboboxItems` (`createItems.ts:99-163`): the lazy `valueToItem` index built
//!   on first access ([`ensure_derived`], first-occurrence-wins with a dev warning for
//!   duplicate derived values), the `data` passthrough (unloaded data stays `None`, not
//!   an empty list), the pure `value` projection, `hasValue`, `itemLabel` (the accessor
//!   itself), and `label` (collection lookup, then `stringifyAsLabel` with the caller's
//!   fallback).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The collection is a struct, not a closure-built object literal: `ensureDerived`'s
//!   `valueToItem === null` lazy-init ports to a `RefCell<Option<Map>>` slot. `Clone`
//!   shares the same underlying `Rc` cells — upstream collections are passed by
//!   reference everywhere, and the createItems test creates module-scope collections
//!   reused by more than one root, so the port's clone must too (the
//!   `PopupTriggerMap` shared-identity precedent).
//! - Items/values are [`serde_json::Value`] (the `resolve_value_label` precedent);
//!   `getValue`/`getLabel`/`fallback` are `Rc<dyn Fn>` accessors the caller supplies.
//!   The generic `Item`/`Value` type parameters collapse onto one runtime type, as in
//!   every dynamic-value port.
//! - The duplicate-value dev warning goes through [`leptos_ui_utils::error::error`]
//!   (`createItems.ts:2`), which itself gates on debug builds — the upstream
//!   `process.env.NODE_ENV !== 'production'` guard's port-side analog.
//! - The nullish-item skip in the index build (`createItems.ts:110-112`, documented as
//!   defensive) ports to skipping `Value::Null` leaves.
//! - `findCollectionItem`'s `exactItem !== undefined || isEqual === defaultItemEquality`
//!   split ports to `Option` presence plus an `is_default` flag on the comparer input:
//!   the port's `find_collection_item` takes the equality function plus a bool marking
//!   it as the default (`Object.is`-semantic) comparer, because Rust closures have no
//!   identity to compare. Callers passing the default comparer get the short-circuit;
//!   custom comparers get the linear scan — the observable contract either way.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde_json::Value;

use leptos_ui_internals::item_equality::compare_item_equality;
use leptos_ui_internals::resolve_value_label::{flatten_leaf_items, stringify_as_label};
use leptos_ui_utils::error::error;

/// The equality comparer type — upstream `ItemEqualityComparer<Value>`
/// (`itemCollection.ts:5`, imported from `itemEquality.ts`): `(a, b) => boolean`.
pub type ItemEqualityComparer = Rc<dyn Fn(&Value, &Value) -> bool>;

/// The map key for the derived-value index: JS `Map` keys are SameValueZero-identity,
/// so the port keys on the value's JSON encoding (the dynamic representation's
/// equality — see `item_equality.rs`'s adaptation notes; the index path only ever
/// serves the default comparer's exact lookups, whose semantics the encoding
/// preserves for the primitive values `createItems` derives).
fn index_key(value: &Value) -> String {
    value.to_string()
}

/// `findCollectionItem` (`itemCollection.ts:8-25`): the exact map hit wins; with a
/// custom comparer the scan compares each derived value; the default comparer
/// short-circuits (the map key already encodes the equality).
pub fn find_collection_item(
    value_to_item: &HashMap<String, Value>,
    item_value: &Value,
    is_default_equality: bool,
    is_equal: &ItemEqualityComparer,
) -> Option<Value> {
    let exact_item = value_to_item.get(&index_key(item_value));
    if exact_item.is_some() || is_default_equality {
        return exact_item.cloned();
    }

    for (derived_value, item) in value_to_item {
        let derived: Value =
            serde_json::from_str(derived_value).expect("index keys are JSON encodings");
        if compare_item_equality(Some(&derived), Some(item_value), |a, b| is_equal(a, b)) {
            return Some(item.clone());
        }
    }

    None
}

/// The internal collection shape — upstream `ItemCollection<Item, Value>`
/// (`itemCollection.ts:44-67`): the accessors the root projects items to values and
/// resolves selected values back to labels with.
#[derive(Clone)]
pub struct ItemCollection {
    /// Source items, preserving their flat or grouped structure for collection
    /// rendering. `None` when the data has not loaded, which the root reads as no
    /// items prop at all (`createItems.ts:131-135`).
    pub data: Option<Vec<Value>>,
    /// Projects a source item to the value used by selection APIs (`createItems.ts:137-144`).
    pub value: Rc<dyn Fn(&Value) -> Value>,
    /// Whether a projected value belongs to the collection's own data (`:145-147`).
    /// The `is_default_equality` flag rides along — see the module adaptation notes.
    pub has_value: Rc<dyn Fn(&Value, bool, &ItemEqualityComparer) -> bool>,
    /// Resolves a source item's label while filtering in the source-item domain (`:148-150`).
    pub item_label: Rc<dyn Fn(&Value) -> String>,
    /// Resolves a selected value's label, including values outside the mounted items;
    /// `fallback` labels the values the collection cannot resolve at all (`:151-158`).
    pub label: Rc<
        dyn Fn(&Value, bool, &ItemEqualityComparer, Option<&dyn Fn(&Value) -> String>) -> String,
    >,
}

/// The options — upstream `CreateComboboxItemsOptions` (`createItems.ts:51-72`).
pub struct CreateComboboxItemsOptions {
    /// Projects an item to the primitive value that identifies it, used as the item's
    /// selection value. `null` and `undefined` are reserved for no selection, and each
    /// item must derive a unique value.
    pub get_value: Rc<dyn Fn(&Value) -> Value>,
    /// Projects an item to the label string that represents it in the input and when
    /// matching the typed query.
    pub get_label: Rc<dyn Fn(&Value) -> String>,
}

/// `createComboboxItems` (`createItems.ts:99-163`): creates a collection for the
/// root's `items` prop. Values and labels are derived on first use — the accessors
/// never run at creation.
pub fn create_combobox_items(
    data: Option<Vec<Value>>,
    options: CreateComboboxItemsOptions,
) -> ItemCollection {
    let CreateComboboxItemsOptions {
        get_value,
        get_label,
    } = options;
    let get_value_in_index = get_value.clone();

    // Lazily indexes the collection's own `data`, so the accessors never run at
    // creation (`createItems.ts:103-127`).
    let value_to_item: Rc<RefCell<Option<HashMap<String, Value>>>> = Rc::new(RefCell::new(None));
    let data_for_index = data.clone();

    let ensure_derived = Rc::new(move || -> HashMap<String, Value> {
        let get_value = get_value_in_index.clone();
        let mut slot = value_to_item.borrow_mut();
        if slot.is_none() {
            let mut derived: HashMap<String, Value> = HashMap::new();

            let leaf_items: Vec<Value> = data_for_index
                .as_ref()
                .map(|items| flatten_leaf_items(&Value::Array(items.clone())))
                .unwrap_or_default();

            for item in leaf_items {
                // Skipped defensively: the data is documented as free of nullish entries.
                if item.is_null() {
                    continue;
                }

                let derived_value = get_value(&item);
                // First occurrence wins, so a duplicated derived value resolves to one
                // stable label.
                if !derived.contains_key(&index_key(&derived_value)) {
                    derived.insert(index_key(&derived_value), item);
                } else {
                    error()
                        .log(&["Two items passed to createItems() derived the value ", &derived_value.to_string(), ", so selection and label resolution cannot tell them apart: the first item wins the label and every item carrying the value renders as selected. Return a unique value from `getValue`."]);
                }
            }

            *slot = Some(derived);
        }
        slot.clone().unwrap()
    });

    // A pure projection with stable identity: the root feeds it to memos and effects,
    // and the collection never stores items it does not own (`createItems.ts:129-136`).
    let value_projection = {
        let get_value = get_value.clone();
        Rc::new(move |item: &Value| -> Value {
            if item.is_null() {
                return item.clone();
            }
            get_value(item)
        })
    };

    let has_value = {
        let ensure_derived = Rc::clone(&ensure_derived);
        Rc::new(
            move |item_value: &Value,
                  is_default_equality: bool,
                  is_equal: &ItemEqualityComparer| {
                find_collection_item(&ensure_derived(), item_value, is_default_equality, is_equal)
                    .is_some()
            },
        )
    };

    let label = {
        let ensure_derived = Rc::clone(&ensure_derived);
        let get_label = Rc::clone(&get_label);
        Rc::new(
            move |item_value: &Value,
                  is_default_equality: bool,
                  is_equal: &ItemEqualityComparer,
                  fallback: Option<&dyn Fn(&Value) -> String>|
                  -> String {
                let item = find_collection_item(
                    &ensure_derived(),
                    item_value,
                    is_default_equality,
                    is_equal,
                );
                if let Some(item) = item {
                    return get_label(&item);
                }

                stringify_as_label(item_value, fallback)
            },
        )
    };

    ItemCollection {
        data,
        value: value_projection,
        has_value,
        item_label: Rc::clone(&get_label),
        label,
    }
}
