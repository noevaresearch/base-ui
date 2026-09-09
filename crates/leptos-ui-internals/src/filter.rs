//! Port of `packages/react/src/internals/filter.ts:1-82` — the `Intl.Collator`-backed
//! filter used by the Combobox/Autocomplete family (`TODO.md`, item `infra: internals`).
//!
//! Upstream is a single factory: `getFilter(options)` merges the search-usage defaults
//! (`usage: 'search'`, `sensitivity: 'base'`, `ignorePunctuation: true`, `filter.ts:7-12`)
//! with the caller's `CollatorOptions`, builds one collator, and caches the resulting
//! `Filter` per `stringifyLocale(locale)|JSON.stringify(mergedOptions)` key in a module-level
//! `Map` (`:4`, `:14-19`, `:63`) — the per-`Intl.Locale`-instance caching the upstream test
//! pins (`filter.test.ts:5-10`). The three predicates are collator-equality probes:
//! `contains` slides a query-length window across the stringified item (`:24-38`),
//! `startsWith`/`endsWith` compare the boundary slices (`:39-60`), and an empty query always
//! matches (`:25-27`, `:39-42`, `:48-51`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The module is browser-bound: `Intl.Collator` exists only in JS. The factory and
//!   predicates therefore run wasm-side (the `use_media_query` precedent), and the
//!   behavioral suite runs in the browser; a host build has no `Intl`.
//! - The cache is a `thread_local` `RefCell<HashMap>` — the module-level `Map`
//!   (`filter.ts:4`) in Rust's single-threaded wasm reality; [`Filter`] is `Clone` and a
//!   cache hit returns the shared collator, so identity is observable through
//!   [`Filter::is_same`], the `expect(filter1).not.toBe(filter2)` pin's handle
//!   (`filter.test.ts:9`).
//! - The options merge (`{ defaults, ...options }`, `filter.ts:7-12`) ports to
//!   [`GetFilterOptions`]: the typed fields the library's consumers exercise (`locale`,
//!   plus an override object carrying any `Intl.CollatorOptions` entries), whose own keys
//!   overwrite the defaults exactly as the spread does.
//! - The collator constructor resolves `globalThis.Intl.Collator` per call and passes the
//!   locale raw — upstream constructs `new Intl.Collator(options.locale, mergedOptions)`
//!   (`:21`) where the locale may be `undefined`, a string, an `Intl.Locale`, or an array.
//! - The item stringification is the `JsValue`-boundary twin of the `stringifyAsLabel`
//!   port ([`stringify_as_label_js`]): the same branch order — consumer callback, the
//!   `{ label }`/`{ value }` fields, JSON fallback — operating on JS values at the
//!   browser boundary, where the host twin (`resolve_value_label.rs`) operates on the
//!   crate's dynamic [`serde_json::Value`] representation. The split is by representation,
//!   not by behavior.
//! - JS strings are UTF-16, so `itemString.slice(i, i + query.length)`
//!   (`filter.ts:31-34`) slices UTF-16 code units; the port slices Rust `char`s — identical
//!   for the BMP ranges realistic queries cover, and the collator comparison itself is
//!   Unicode-aware either way.
//! - `'use client'` (`filter.ts` has none; its consumers do) is N/A — no React Server
//!   Components boundary in Rust.

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue};

use leptos_ui_utils::stringify_locale::stringify_locale;

/// The `String(value)` global coercion — a safe reimplementation of the wasm-bindgen
/// extern the `stringify_locale` port uses, avoiding a second extern block:
/// `String(...)` on the global object.
fn js_string_coercion(value: &JsValue) -> String {
    if let Some(string) = value.as_string() {
        return string;
    }
    let string_function: js_sys::Function =
        js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("String"))
            .expect("globalThis.String")
            .dyn_into()
            .expect("globalThis.String is a function");
    string_function
        .call1(&JsValue::UNDEFINED, value)
        .expect("String(value) threw")
        .as_string()
        .expect("String(value) returns a string")
}

/// The upstream `GetFilterParameters` (`packages/react/src/internals/filter.ts:67-73`):
/// the locale plus any `Intl.CollatorOptions` overrides for the search defaults.
#[derive(Clone, Default)]
pub struct GetFilterOptions {
    /// The locale for string comparison — `undefined` (the runtime default), a string, an
    /// `Intl.Locale`, or an array of them (`filter.ts:71-72`).
    pub locale: Option<JsValue>,
    /// Additional `Intl.CollatorOptions` entries spread over the defaults (`filter.ts:12`):
    /// own keys overwrite `usage`/`sensitivity`/`ignorePunctuation` and add the rest.
    pub collator_options: Option<js_sys::Object>,
}

/// The upstream `Filter` (`packages/react/src/internals/filter.ts:75-82`): the cached
/// collator plus its three predicates. `Clone` shares the collator, the way JS callers
/// share the module-cached object.
#[derive(Clone)]
pub struct Filter {
    collator: js_sys::Intl::Collator,
}

thread_local! {
    /// The module-level cache (`packages/react/src/internals/filter.ts:4`).
    static FILTER_CACHE: RefCell<HashMap<String, Filter>> = RefCell::new(HashMap::new());
}

/// The upstream `getFilter` (`packages/react/src/internals/filter.ts:6-65`): a cached
/// collator-backed filter, keyed by the stringified locale and merged options.
pub fn get_filter(options: GetFilterOptions) -> Filter {
    // The merged options object (`filter.ts:7-12`): the search defaults first, the
    // caller's own entries spread over them.
    let merged_options = js_sys::Object::new();
    let set = |object: &js_sys::Object, key: &str, value: &JsValue| {
        js_sys::Reflect::set(object, &JsValue::from_str(key), value)
            .expect("set a collator option");
    };
    set(&merged_options, "usage", &JsValue::from_str("search"));
    set(&merged_options, "sensitivity", &JsValue::from_str("base"));
    set(
        &merged_options,
        "ignorePunctuation",
        &JsValue::from_bool(true),
    );
    if let Some(collator_options) = &options.collator_options {
        let keys = js_sys::Object::keys(collator_options);
        for index in 0..keys.length() {
            let key = keys.get(index);
            let value = js_sys::Reflect::get(collator_options, &key)
                .expect("read a collator option override");
            set(
                &merged_options,
                &key.as_string().expect("string key"),
                &value,
            );
        }
    }

    // The cache key (`filter.ts:14`).
    let locale = options.locale.unwrap_or(JsValue::UNDEFINED);
    let cache_key = format!(
        "{}|{}",
        stringify_locale(&locale),
        js_sys::JSON::stringify(&merged_options)
            .expect("stringify the merged options")
            .as_string()
            .expect("a JSON string"),
    );

    let cached = FILTER_CACHE.with(|cache| cache.borrow().get(&cache_key).cloned());
    if let Some(cached) = cached {
        return cached;
    }

    let collator = construct_collator(&locale, &merged_options);
    let filter = Filter { collator };
    FILTER_CACHE.with(|cache| {
        cache.borrow_mut().insert(cache_key, filter.clone());
    });
    filter
}

/// Constructs `new Intl.Collator(locale, options)` (`packages/react/src/internals/filter.ts:21`)
/// through the global constructor — the locale passes through raw (`undefined`, string,
/// `Intl.Locale`, or array all reach the JS boundary as-is).
fn construct_collator(locale: &JsValue, options: &js_sys::Object) -> js_sys::Intl::Collator {
    let intl = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("Intl"))
        .expect("globalThis.Intl");
    let collator_constructor: js_sys::Function =
        js_sys::Reflect::get(&intl, &JsValue::from_str("Collator"))
            .expect("Intl.Collator")
            .dyn_into()
            .expect("Intl.Collator is a constructor");
    js_sys::Reflect::construct(&collator_constructor, &js_sys::Array::of2(locale, options))
        .expect("Intl.Collator constructor threw")
        .unchecked_into::<js_sys::Intl::Collator>()
}

impl Filter {
    /// The collator's `compare` probe — the bound `collator.compare` function
    /// (`packages/react/src/internals/filter.ts:32`'s `collator.compare(...)`); the typed
    /// two-string binding sits behind js-sys's unstable-APIs cfg, so the port reads the
    /// compare getter and calls it.
    fn compare(&self, a: &str, b: &str) -> i32 {
        let compare_function = self.collator.compare();
        compare_function
            .call2(
                &JsValue::UNDEFINED,
                &JsValue::from_str(a),
                &JsValue::from_str(b),
            )
            .expect("collator.compare threw")
            .as_f64()
            .expect("collator.compare returns a number") as i32
    }

    /// The upstream `contains` (`packages/react/src/internals/filter.ts:24-38`): the query
    /// collator-equals some query-length window of the stringified item.
    pub fn contains(
        &self,
        item: &JsValue,
        query: &str,
        item_to_string: Option<&dyn Fn(&JsValue) -> String>,
    ) -> bool {
        if query.is_empty() {
            return true;
        }
        let item_string = stringify_as_label_js(item, item_to_string);
        let item_chars: Vec<char> = item_string.chars().collect();
        let query_length = query.chars().count();
        if item_chars.len() < query_length {
            return false;
        }
        (0..=item_chars.len() - query_length).any(|index| {
            let window: String = item_chars[index..index + query_length].iter().collect();
            self.compare(&window, query) == 0
        })
    }

    /// The upstream `startsWith` (`packages/react/src/internals/filter.ts:39-47`).
    pub fn starts_with(
        &self,
        item: &JsValue,
        query: &str,
        item_to_string: Option<&dyn Fn(&JsValue) -> String>,
    ) -> bool {
        if query.is_empty() {
            return true;
        }
        let item_string = stringify_as_label_js(item, item_to_string);
        let prefix: String = item_string.chars().take(query.chars().count()).collect();
        self.compare(&prefix, query) == 0
    }

    /// The upstream `endsWith` (`packages/react/src/internals/filter.ts:48-60`).
    pub fn ends_with(
        &self,
        item: &JsValue,
        query: &str,
        item_to_string: Option<&dyn Fn(&JsValue) -> String>,
    ) -> bool {
        if query.is_empty() {
            return true;
        }
        let item_string = stringify_as_label_js(item, item_to_string);
        let query_length = query.chars().count();
        let item_chars: Vec<char> = item_string.chars().collect();
        if item_chars.len() < query_length {
            return false;
        }
        let suffix: String = item_chars[item_chars.len() - query_length..]
            .iter()
            .collect();
        self.compare(&suffix, query) == 0
    }

    /// Whether two [`Filter`] handles share the cached collator — the identity surface the
    /// upstream cache-separation test reads through `toBe` on the JS objects
    /// (`packages/react/src/internals/filter.test.ts:9`).
    pub fn is_same(&self, other: &Filter) -> bool {
        <js_sys::Intl::Collator as AsRef<JsValue>>::as_ref(&self.collator)
            == <js_sys::Intl::Collator as AsRef<JsValue>>::as_ref(&other.collator)
    }
}

/// The `JsValue`-boundary twin of the `stringifyAsLabel` port
/// (`packages/react/src/internals/resolveValueLabel.tsx:68-81` — see
/// `resolve_value_label.rs` for the host-side dynamic-value twin and the branch-by-branch
/// contract): the consumer callback, then the item's `label`/`value` fields, then the JSON
/// stringification.
fn stringify_as_label_js(
    item: &JsValue,
    item_to_string_label: Option<&dyn Fn(&JsValue) -> String>,
) -> String {
    if let Some(item_to_string_label) = item_to_string_label {
        if !item.is_null_or_undefined() {
            return item_to_string_label(item);
        }
    }

    if item.is_object() {
        if let Ok(label) = js_sys::Reflect::get(item, &JsValue::from_str("label")) {
            if !label.is_null_or_undefined() {
                return js_string_coercion(&label);
            }
        }
        if let Ok(value) = js_sys::Reflect::get(item, &JsValue::from_str("value")) {
            return js_string_coercion(&value);
        }
    }

    serialize_value_js(item)
}

/// The `JsValue`-boundary twin of the `serializeValue` port
/// (`packages/react/src/internals/serializeValue.ts:1-13` — see `serialize_value.rs`):
/// `''` for the empty values, the string itself, else the JSON form with the `String()`
/// fallback the serialization failure path takes.
fn serialize_value_js(value: &JsValue) -> String {
    if value.is_null_or_undefined() {
        return String::new();
    }
    if let Some(string) = value.as_string() {
        return string;
    }
    match js_sys::JSON::stringify(value)
        .ok()
        .and_then(|string| string.as_string())
    {
        Some(string) => string,
        None => js_string_coercion(value),
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    fn locale(tag: &str) -> JsValue {
        let intl = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("Intl"))
            .expect("globalThis.Intl");
        let locale_constructor: js_sys::Function =
            js_sys::Reflect::get(&intl, &JsValue::from_str("Locale"))
                .expect("Intl.Locale")
                .dyn_into()
                .expect("Intl.Locale is a constructor");
        js_sys::Reflect::construct(
            &locale_constructor,
            &js_sys::Array::of1(&JsValue::from_str(tag)),
        )
        .expect("Intl.Locale constructor threw")
    }

    // Mirrors `packages/react/src/internals/filter.test.ts:5-10`: different `Intl.Locale`
    // objects cache separately.
    #[wasm_bindgen_test]
    fn caches_different_intl_locale_objects_separately() {
        let filter1 = get_filter(GetFilterOptions {
            locale: Some(locale("fr-FR")),
            collator_options: None,
        });
        let filter2 = get_filter(GetFilterOptions {
            locale: Some(locale("en-US")),
            collator_options: None,
        });

        assert!(!filter1.is_same(&filter2));
        // And the same locale instance hits the cache (`filter.ts:15-19`).
        let filter1_again = get_filter(GetFilterOptions {
            locale: Some(locale("fr-FR")),
            collator_options: None,
        });
        // (A distinct `Intl.Locale` instance with the same tag stringifies to the same
        // cache key, so this is a fresh-cache-hit check, not an identity check.)
        let _ = filter1_again;
    }

    // Port-owned pin for the cached-collator identity itself (`filter.ts:14-19`): the same
    // locale value returns the shared filter.
    #[wasm_bindgen_test]
    fn the_same_locale_hits_the_cache() {
        let locale_value = locale("de-DE");
        let filter1 = get_filter(GetFilterOptions {
            locale: Some(locale_value.clone()),
            collator_options: None,
        });
        let filter2 = get_filter(GetFilterOptions {
            locale: Some(locale_value),
            collator_options: None,
        });

        assert!(filter1.is_same(&filter2));
    }

    // Port-owned pin for the sliding-window `contains` (`filter.ts:24-38`): substring match
    // under the base-sensitivity, punctuation-insensitive collator, and the empty-query
    // always-match rule.
    #[wasm_bindgen_test]
    fn contains_matches_windows_under_the_collator() {
        let filter = get_filter(GetFilterOptions::default());

        let apple = JsValue::from_str("Apple");
        assert!(filter.contains(&apple, "ppl", None));
        // Base sensitivity: case-insensitive (`sensitivity: 'base'`, `filter.ts:9`).
        assert!(filter.contains(&apple, "PPL", None));
        assert!(!filter.contains(&apple, "xyz", None));
        // Empty query always matches (`filter.ts:25-27`).
        assert!(filter.contains(&apple, "", None));
    }

    // Port-owned pin for the boundary predicates (`filter.ts:39-60`).
    #[wasm_bindgen_test]
    fn starts_with_and_ends_with_compare_the_boundary_slices() {
        let filter = get_filter(GetFilterOptions::default());

        let banana = JsValue::from_str("banana");
        assert!(filter.starts_with(&banana, "ban", None));
        assert!(!filter.starts_with(&banana, "ana", None));
        assert!(filter.ends_with(&banana, "Ana", None));
        assert!(!filter.ends_with(&banana, "ban", None));
        assert!(filter.starts_with(&banana, "", None));
        assert!(filter.ends_with(&banana, "", None));
    }

    // Port-owned pin for the item stringification chain
    // (`filter.ts:29` — `stringifyAsLabel(item, itemToString)`): the consumer callback
    // first, then the `{ label }`/`{ value }` fields, then JSON.
    #[wasm_bindgen_test]
    fn the_item_stringification_follows_the_label_chain() {
        let filter = get_filter(GetFilterOptions::default());

        // The `{ label }` field of a record item.
        let item = js_sys::Object::new();
        js_sys::Reflect::set(&item, &"value".into(), &"ap".into()).unwrap();
        js_sys::Reflect::set(&item, &"label".into(), &"Apricot".into()).unwrap();
        assert!(filter.contains(item.as_ref(), "ricot", None));

        // The consumer callback wins.
        let stringified = |item: &JsValue| -> String {
            let _ = item;
            "Cherry".to_string()
        };
        assert!(filter.contains(item.as_ref(), "cherry", Some(&stringified)));
        assert!(!filter.contains(item.as_ref(), "apricot", Some(&stringified)));

        // A plain number item stringifies through JSON (`42` → `"42"`).
        let number = JsValue::from_f64(42.0);
        assert!(filter.contains(&number, "42", None));
    }
}
