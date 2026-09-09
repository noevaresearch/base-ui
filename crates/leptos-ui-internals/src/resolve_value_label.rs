//! Port of `packages/react/src/internals/resolveValueLabel.tsx:1-158` — the label-resolution
//! vocabulary shared by the Select-family components (`TODO.md`, item `infra: internals`).
//!
//! Upstream exports the group classifier ([`is_grouped_items`], `:22-28`), the leaf
//! flattener ([`flatten_leaf_items`], `:30-36`), the null-label probe
//! ([`has_null_item_label`], `:41-66`), the two stringifiers ([`stringify_as_label`],
//! `:68-81`; [`stringify_as_value`], `:83-91`), and the selected-label resolver
//! ([`resolve_selected_label`], `:93-140`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Items and values are the crate's dynamic [`Value`] representation (the
//!   `state_attributes` precedent): upstream's `any`-shaped items — flat `{ value, label }`
//!   records, groups (any object carrying an actual `items` **array**, `:18-20`), and the
//!   plain-record alternative input — are all one runtime shape in JS. The items input's
//!   `undefined` state ports to `Option<&Value>`.
//! - `'value' in item`/`item.label != null` guards port to [`Value::get`] presence plus
//!   `!is_null()`: a missing key and a `null`/`undefined` payload are the same "absent"
//!   outcome in every branch below, which the dynamic representation's single `Null`
//!   mirrors.
//! - The plain-record lookup's `Object.hasOwn(items, value)`
//!   (`:113`) ports to [`serde_json::Map::contains_key`] keyed by the JS property-key
//!   coercion of the value — `js_to_string` (`state_attributes`' shared coercion), which
//!   yields `"null"`, `"42"`, `"true"`, and the string itself for the shapes upstream
//!   exercises, including the prototype-member safety: JSON maps have no prototype, so the
//!   `hasOwn` guard's outcome (unclaimed keys like `"constructor"` fall back to the
//!   stringified value, `resolveValueLabel.test.ts:123-148`) is structural.
//! - The flat-item match `item.value === value` (`:123`, `:132`) ports to `Value`
//!   equality. One inherent divergence: JS numbers are a single type (`1 === 1.0`), while
//!   JSON distinguishes integer and float encodings — component values are realistically
//!   consistent, and the label tests are string/null-based.
//! - [`resolve_selected_label`] returns [`Value`] where upstream returns
//!   `React.ReactNode` — the label extracted from an item is passed through verbatim (a
//!   node upstream, a dynamic value here); every stringified branch produces
//!   `Value::String`.
//! - [`resolve_multiple_labels`] (`:142-157`) is deferred: it folds resolved labels into an
//!   array of React nodes with `", "` separators, is untested upstream (the implementation
//!   spec's "Anything in source" item 9), and its node-array return shape belongs to the
//!   Select-family view layer that consumes it. It ports alongside that consumer.
//! - `'use client'` (`resolveValueLabel.tsx:1`) is N/A — no React Server Components
//!   boundary in Rust.

use serde_json::Value;

use crate::serialize_value::serialize_value;
use crate::state_attributes::js_to_string;

/// The group test (`packages/react/src/internals/resolveValueLabel.tsx:18-20`): an object
/// carrying an actual `items` array. Key presence alone would misclassify an item with an
/// unrelated or optional `items` field.
fn is_group(item: &Value) -> bool {
    item.is_object() && item.get("items").is_some_and(|items| items.is_array())
}

/// The upstream `isGroupedItems` (`packages/react/src/internals/resolveValueLabel.tsx:22-28`):
/// whether the items list starts with a group. A missing/non-list input is ungrouped
/// (the optional chain `items?.[0]` falls through `isGroup(undefined)`).
pub fn is_grouped_items(items: Option<&Value>) -> bool {
    items.and_then(|items| items.get(0)).is_some_and(is_group)
}

/// The upstream `flattenLeafItems` (`packages/react/src/internals/resolveValueLabel.tsx:30-36`):
/// the leaves under a possibly-grouped items list — grouped lists flatten their groups'
/// `items` arrays, flat lists pass through. The leaves are cloned out of the dynamic
/// representation. A non-group element inside a grouped list contributes nothing: upstream
/// would raise a `TypeError` reading `.items` off it (`:34`), and the resolver only calls
/// here behind [`is_grouped_items`] with list-shaped inputs, so the port flattens what is
/// present instead of panicking.
pub fn flatten_leaf_items(items: &Value) -> Vec<Value> {
    if is_grouped_items(Some(items)) {
        items
            .as_array()
            .expect("grouped items")
            .iter()
            .flat_map(|group| {
                group
                    .get("items")
                    .and_then(|items| items.as_array())
                    .cloned()
                    .unwrap_or_default()
            })
            .collect()
    } else {
        items.as_array().expect("flat items").clone()
    }
}

/// Whether any leaf item carries a `null` value with a present label
/// (`packages/react/src/internals/resolveValueLabel.tsx:41-66`). The plain-record input is
/// probed for the literal `"null"` key — the record form of the null-value placeholder.
pub fn has_null_item_label(items: Option<&Value>) -> bool {
    let Some(items) = items else {
        return false;
    };

    if !items.is_array() {
        return items
            .as_object()
            .is_some_and(|record| record.contains_key("null"));
    }

    let contains = |item: &Value| {
        item.is_object()
            && item.get("value").is_none_or(Value::is_null)
            && item.get("label").is_some_and(|label| !label.is_null())
    };

    if is_grouped_items(Some(items)) {
        items
            .as_array()
            .expect("grouped items")
            .iter()
            .any(|group| {
                group
                    .get("items")
                    .and_then(|items| items.as_array())
                    .map(|items| items.iter().any(&contains))
                    .unwrap_or(false)
            })
    } else {
        items.as_array().expect("flat items").iter().any(contains)
    }
}

/// The upstream `stringifyAsLabel` (`packages/react/src/internals/resolveValueLabel.tsx:68-81`):
/// the consumer's labeler, then the item's `label`/`value` fields, then the JSON
/// stringification. The `?? ''` guard on the callback's return
/// (`:70`) is absorbed by the `String` return type — a non-nullish return by construction.
pub fn stringify_as_label(
    item: &Value,
    item_to_string_label: Option<&dyn Fn(&Value) -> String>,
) -> String {
    if let Some(item_to_string_label) = item_to_string_label {
        if !item.is_null() {
            return item_to_string_label(item);
        }
    }

    if item.is_object() {
        if let Some(label) = item.get("label").filter(|label| !label.is_null()) {
            return js_to_string(label);
        }
        if let Some(value) = item.get("value") {
            return js_to_string(value);
        }
    }

    serialize_value(item)
}

/// The upstream `stringifyAsValue` (`packages/react/src/internals/resolveValueLabel.tsx:83-91`):
/// the consumer's stringifier, then the `{ value, label }` record's `value` field, then the
/// JSON stringification.
pub fn stringify_as_value(
    item: &Value,
    item_to_string_value: Option<&dyn Fn(&Value) -> String>,
) -> String {
    if let Some(item_to_string_value) = item_to_string_value {
        if !item.is_null() {
            return item_to_string_value(item);
        }
    }

    if item.is_object() && item.get("value").is_some() && item.get("label").is_some() {
        return serialize_value(item.get("value").expect("checked"));
    }

    serialize_value(item)
}

/// The upstream `resolveSelectedLabel` (`packages/react/src/internals/resolveValueLabel.tsx:93-140`):
/// the label for a selected value — the consumer's labeler, an explicit object label, the
/// plain-record lookup (own keys only), the flat/grouped array match by `value`, and the
/// stringified fallback.
pub fn resolve_selected_label(
    value: &Value,
    items: Option<&Value>,
    item_to_string_label: Option<&dyn Fn(&Value) -> String>,
) -> Value {
    let fallback = || Value::String(stringify_as_label(value, item_to_string_label));

    if let Some(item_to_string_label) = item_to_string_label {
        if !value.is_null() {
            return Value::String(item_to_string_label(value));
        }
    }

    // Custom object with explicit label takes precedence (`:106-109`).
    if value.is_object() {
        if let Some(label) = value.get("label").filter(|label| !label.is_null()) {
            return label.clone();
        }
    }

    // Items provided as plain record map (`:111-115`).
    if let Some(items) = items.filter(|items| !items.is_array()) {
        let Some(record) = items.as_object() else {
            return fallback();
        };
        let key = js_to_string(value);
        return match record.get(&key) {
            Some(label) if !label.is_null() => label.clone(),
            _ => fallback(),
        };
    }

    // Items provided as array (flat or grouped) (`:117-137`).
    if let Some(items) = items.filter(|items| items.is_array()) {
        let flat_items = flatten_leaf_items(items);

        if value.is_null() || !value.is_object() {
            let match_label = flat_items
                .iter()
                .find(|item| item.is_object() && item.get("value") == Some(value))
                .and_then(|item| item.get("label"))
                .filter(|label| !label.is_null());
            if let Some(label) = match_label {
                return label.clone();
            }
            return fallback();
        }

        // Object without explicit label: try matching by its `value` property (`:130-136`).
        if let Some(value_key) = value.get("value") {
            let match_label = flat_items
                .iter()
                .find(|item| item.is_object() && item.get("value") == Some(value_key))
                .and_then(|item| item.get("label"))
                .filter(|label| !label.is_null());
            if let Some(label) = match_label {
                return label.clone();
            }
        }
    }

    fallback()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn list(values: serde_json::Value) -> Value {
        values
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:5-18` (the `it.each`
    // matrix; the `undefined` items field is omitted JSON — same ungrouped outcome).
    #[test]
    fn classifies_grouped_items_by_an_actual_items_array() {
        let items_field_missing = list(json!([{ "value": "a" }]));
        let items_field_scalar = list(json!([{ "value": "a", "items": 3 }]));
        let items_field_array = list(json!([{ "value": "group", "items": [] }]));
        let starts_with_flat_item =
            list(json!([{ "value": "a" }, { "value": "group", "items": [] }]));

        assert!(!is_grouped_items(Some(&items_field_missing)));
        assert!(!is_grouped_items(Some(&items_field_scalar)));
        assert!(is_grouped_items(Some(&items_field_array)));
        assert!(!is_grouped_items(Some(&starts_with_flat_item)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:20-24`.
    #[test]
    fn resolves_a_flat_item_label_when_the_item_has_an_unrelated_items_field() {
        let items = list(json!([{ "value": "a", "label": "A", "items": "metadata" }]));

        assert_eq!(
            resolve_selected_label(&json!("a"), Some(&items), None),
            json!("A")
        );
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:26-39`.
    #[test]
    fn the_null_label_probe_finds_a_null_valued_item_with_a_label_in_a_group() {
        let items = list(json!([{
            "value": "group-1",
            "items": [
                { "value": "a", "label": "A" },
                { "value": null, "label": "Select" },
            ],
        }]));

        assert!(has_null_item_label(Some(&items)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:41-53`.
    #[test]
    fn the_null_label_probe_rejects_a_null_valued_item_without_a_label() {
        let items = list(json!([{
            "value": "group-1",
            "items": [
                { "value": null, "label": null },
                { "value": "a", "label": "A" },
            ],
        }]));

        assert!(!has_null_item_label(Some(&items)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:55-64`.
    #[test]
    fn the_null_label_probe_rejects_groups_without_a_null_valued_item() {
        let items = list(json!([{
            "value": "group-1",
            "items": [{ "value": "a", "label": "A" }],
        }]));

        assert!(!has_null_item_label(Some(&items)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:66-78` — group
    // detection is by the `items` array, so custom heading keys change nothing.
    #[test]
    fn the_null_label_probe_supports_groups_with_custom_heading_keys() {
        let items = list(json!([{
            "heading": "group-1",
            "items": [
                { "value": "a", "label": "A" },
                { "value": null, "label": "Select" },
            ],
        }]));

        assert!(has_null_item_label(Some(&items)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:80-87`.
    #[test]
    fn the_null_label_probe_finds_a_null_valued_item_with_a_label_in_a_flat_list() {
        let items = list(json!([
            { "value": "a", "label": "A" },
            { "value": null, "label": "None" },
        ]));

        assert!(has_null_item_label(Some(&items)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:89-96`.
    #[test]
    fn the_null_label_probe_rejects_flat_lists_without_a_null_valued_item() {
        let items = list(json!([
            { "value": "a", "label": "A" },
            { "value": "b", "label": "B" },
        ]));

        assert!(!has_null_item_label(Some(&items)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:98-116`.
    #[test]
    fn the_null_label_probe_keys_the_record_form_on_the_literal_null_key() {
        let without_null = list(json!({
            "sans": "Sans-serif",
            "serif": "Serif",
            "mono": "Monospace",
        }));
        let with_null = list(json!({
            "null": "None",
            "sans": "Sans-serif",
            "serif": "Serif",
        }));

        assert!(!has_null_item_label(Some(&without_null)));
        assert!(has_null_item_label(Some(&with_null)));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:118-120`.
    #[test]
    fn the_null_label_probe_rejects_a_missing_input() {
        assert!(!has_null_item_label(None));
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:123-131`: JSON maps
    // have no prototype, so unclaimed member-named values fall back to the stringified
    // value — the `Object.hasOwn` guard's outcome, structurally.
    #[test]
    fn falls_back_to_the_stringified_label_when_the_value_matches_an_object_prototype_member() {
        let items = list(json!({
            "sans": "Sans-serif",
            "serif": "Serif",
            "mono": "Monospace",
        }));

        for member in ["constructor", "toString", "hasOwnProperty", "__proto__"] {
            assert_eq!(
                resolve_selected_label(&json!(member), Some(&items), None),
                json!(member),
                "member: {member}"
            );
        }
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:133-137`.
    #[test]
    fn resolves_an_own_key_that_matches_an_object_prototype_member() {
        let items = list(json!({
            "constructor": "Custom constructor",
            "sans": "Sans-serif",
        }));

        assert_eq!(
            resolve_selected_label(&json!("constructor"), Some(&items), None),
            json!("Custom constructor")
        );
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:139-143`.
    #[test]
    fn keeps_resolving_the_null_placeholder_key_in_a_record() {
        let items = list(json!({
            "null": "None",
            "sans": "Sans-serif",
        }));

        assert_eq!(
            resolve_selected_label(&Value::Null, Some(&items), None),
            json!("None")
        );
    }

    // Mirrors `packages/react/src/internals/resolveValueLabel.test.ts:145-148`.
    #[test]
    fn falls_back_to_the_stringified_label_when_an_own_key_has_a_nullish_label() {
        let with_undefined = list(json!({ "sans": null }));
        let with_null = list(json!({ "sans": null }));

        assert_eq!(
            resolve_selected_label(&json!("sans"), Some(&with_undefined), None),
            json!("sans")
        );
        assert_eq!(
            resolve_selected_label(&json!("sans"), Some(&with_null), None),
            json!("sans")
        );
    }

    // Port-owned pins for the stringifiers and the remaining resolver branches
    // (`packages/react/src/internals/resolveValueLabel.tsx:68-91`, `:117-137` — cited by
    // the implementation spec but only partially exercised upstream; the stringifier tests
    // live in the unported consumer units):
    // - the label stringifier's field chain and the value stringifier's `{ value, label }`
    //   branch;
    // - the grouped flatten + value match in the array resolver.
    #[test]
    fn the_stringifiers_follow_the_field_chain() {
        let item = json!({ "value": "a", "label": "A" });

        assert_eq!(stringify_as_label(&item, None), "A");
        // `serializeValue` passes strings through raw (`serializeValue.ts:5-7`), so the
        // `{ value, label }` branch emits the value unquoted.
        assert_eq!(stringify_as_value(&item, None), "a");

        // The consumer's labeler wins over the fields.
        assert_eq!(
            stringify_as_label(&item, Some(&|_item: &Value| "custom".to_string())),
            "custom"
        );

        // A plain value stringifies through JSON.
        assert_eq!(stringify_as_label(&json!(42), None), "42");
        assert_eq!(stringify_as_value(&Value::Null, None), "");
    }

    #[test]
    fn the_array_resolver_matches_through_groups() {
        let items = list(json!([
            {
                "heading": "Numbers",
                "items": [
                    { "value": 1, "label": "One" },
                    { "value": 2, "label": "Two" },
                ],
            },
            {
                "heading": "Letters",
                "items": [{ "value": "a", "label": "A" }],
            },
        ]));

        assert_eq!(
            resolve_selected_label(&json!(2), Some(&items), None),
            json!("Two")
        );
        assert_eq!(
            resolve_selected_label(&json!("a"), Some(&items), None),
            json!("A")
        );
        // No match: the value itself, stringified.
        assert_eq!(
            resolve_selected_label(&json!("zzz"), Some(&items), None),
            json!("zzz")
        );
    }
}
