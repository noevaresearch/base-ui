//! Port of the `internals` unit's state-to-attribute machinery — three small upstream files:
//!
//! - `packages/react/src/internals/getStateAttributesProps.ts:1-32` — [`get_state_attributes_props`]
//!   and the [`StateAttributesMapping`] vocabulary;
//! - `packages/react/src/internals/stateAttributesMapping.ts:1-20` —
//!   [`transition_status_mapping`];
//! - `packages/react/src/internals/TransitionStatusDataAttributes.ts:1-9` — the
//!   [`STARTING_STYLE`]/[`ENDING_STYLE`] attribute constants;
//! - plus `fieldValidityMapping` from `packages/react/src/internals/field-constants/
//!   constants.ts:34-43` (its only consumer-facing piece exercised by this unit's tests —
//!   `stateAttributesMapping.test.ts:25-33`; the module's `DEFAULT_*` state constants are
//!   typed against the not-yet-ported field unit and are deferred with it).
//!
//! `getStateAttributesProps` turns a component's state object into `data-*` attribute props
//! (`packages/react/src/internals/getStateAttributesProps.ts:5-31`): truthy fields become
//! attributes, `true` becomes the bare (empty-valued) form, and a per-field custom mapping
//! overrides both — its callback receives the field value and may return a props object or
//! `null` to emit nothing. `transitionStatusMapping` is the canonical custom mapping: it
//! emits `data-starting-style`/`data-ending-style` hooks during a transition and `null`
//! (no attributes) outside one (`stateAttributesMapping.ts:7-19`).
//!
//! Rust adaptations (behavior-preserving where the contract is defined):
//!
//! - Upstream's dynamic state object ports to [`serde_json::Map`]`<String, `[`Value`]`>` —
//!   the crate's dynamic-value representation (the `resolve_value_label` precedent), since
//!   the function's whole contract is shape-agnostic iteration (`for (const key in state)`,
//!   `getStateAttributesProps.ts:12`). One divergence is inherent to the representation:
//!   `serde_json::Map` iterates keys in sorted order, while JS object iteration is insertion
//!   order — observable only through the output map's own iteration order, never through its
//!   attribute set, which is what consumers read.
//! - The custom-mapping object (`{ [key]: (value) => props | null }`) ports to a single
//!   [`StateAttributesMapping`] callable receiving `(key, value)`: returning
//!   `None` mirrors `hasOwnProperty(key)` being false (fall through to the default
//!   handling, `getStateAttributesProps.ts:15`), `Some(None)` mirrors the callback returning
//!   `null` (`:17-21`), and `Some(Some(props))` mirrors the returned props object being
//!   merged (`:18`). [`transition_status_mapping`] and [`field_validity_mapping`] are
//!   written in exactly that shape, keyed by their state field name.
//! - JS truthiness (`value ? ... : skip`, `getStateAttributesProps.ts:24-28`) ports to
//!   [`is_truthy`] over `Value`: `null`/`false`/`0`/`""`/`NaN` are falsy, everything else
//!   (including empty arrays/objects, as in JS) is truthy. `true` is detected strictly
//!   before the general truthy branch (`:24`), producing the empty-valued attribute.
//! - `value.toString()` (`:27`) ports to [`js_to_string`], the JS `String()` coercion over
//!   `Value` (arrays join with `","`, plain objects coerce to `"[object Object]"`,
//!   integral floats drop the trailing `.0`). Number formatting beyond the integral/simple
//!   cases approximates JS's shortest-round-trip output; component state values are
//!   realistically simple.
//! - `key.toLowerCase()` (`:25`) ports to `str::to_lowercase` — Unicode-aware where JS is
//!   locale-independent, coinciding for the ASCII field names the library emits.
//! - Upstream's `undefined` transition status (`stateAttributesMapping.test.ts:21`) and
//!   `null` validity collapse onto `Value::Null`, the dynamic representation's single empty
//!   value — same mapping outcome (`null` → no attributes) for both.

use serde_json::Value;
use std::collections::BTreeMap;

/// `startingStyle` (`packages/react/src/internals/TransitionStatusDataAttributes.ts:4`) —
/// present when the component begins animating in.
pub const STARTING_STYLE: &str = "data-starting-style";

/// `endingStyle` (`packages/react/src/internals/TransitionStatusDataAttributes.ts:8`) —
/// present when the component is animating out.
pub const ENDING_STYLE: &str = "data-ending-style";

/// The attribute props a state mapping produces: attribute name → value (the empty string
/// for bare `data-*` attributes).
pub type StateAttributeProps = BTreeMap<String, String>;

/// The ported custom-mapping shape: called per state field with `(key, value)`.
///
/// - `None` — no mapping for this key (the `hasOwnProperty` miss,
///   `packages/react/src/internals/getStateAttributesProps.ts:15`); default handling applies.
/// - `Some(None)` — the mapping declined the field (the JS callback returning `null`,
///   `:17-21`); no attributes are emitted for it.
/// - `Some(Some(props))` — the mapping's props, merged into the output (`:18`).
pub type StateAttributesMapping<'a> =
    dyn Fn(&str, &Value) -> Option<Option<StateAttributeProps>> + 'a;

/// JS truthiness over a dynamic value (`getStateAttributesProps.ts:24-28` gate).
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        // JS `NaN` is falsy; serde_json numbers cannot be NaN, so the arm is unreachable in
        // practice and kept for documentation symmetry.
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        // JS arrays and objects are truthy even when empty.
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// The JS `String()` coercion over a dynamic value — the truthy branch's
/// `value.toString()` (`packages/react/src/internals/getStateAttributesProps.ts:27`) and
/// the array/object coercions JS applies implicitly. Shared with the
/// `resolve_value_label`/`serialize_value` ports, whose stringification fallbacks use the
/// same coercion.
pub fn js_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => js_number_to_string(n),
        Value::String(s) => s.clone(),
        // JS `Array.prototype.toString` joins with "," and renders null/undefined slots as
        // "" (they flatten to empty through the recursion's Null arm producing "null" — so
        // mirror the join with the empty-string slot rule directly).
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                other => js_to_string(other),
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// JS number-to-string: integral values within the safe-integer range drop the fractional
/// part (JS `String(42.0)` is `"42"`, where serde_json would print `"42.0"`).
fn js_number_to_string(n: &serde_json::Number) -> String {
    match n.as_f64() {
        Some(f) if f.is_finite() && f == f.trunc() && f.abs() < 9.007_199_254_740_992e15 => {
            format!("{}", f as i64)
        }
        Some(f) => format!("{f}"),
        None => n.to_string(),
    }
}

/// The upstream `getStateAttributesProps`
/// (`packages/react/src/internals/getStateAttributesProps.ts:5-31`).
pub fn get_state_attributes_props(
    state: &serde_json::Map<String, Value>,
    custom_mapping: Option<&StateAttributesMapping>,
) -> StateAttributeProps {
    let mut props = StateAttributeProps::new();

    for (key, value) in state {
        if let Some(mapping) = custom_mapping {
            if let Some(custom_props) = mapping(key, value) {
                if let Some(custom_props) = custom_props {
                    props.extend(custom_props);
                }
                // `Some(None)` — the mapping returned null: skip the field entirely
                // (`getStateAttributesProps.ts:20-21`).
                continue;
            }
        }

        if value == &Value::Bool(true) {
            props.insert(format!("data-{}", key.to_lowercase()), String::new());
        } else if is_truthy(value) {
            props.insert(format!("data-{}", key.to_lowercase()), js_to_string(value));
        }
    }

    props
}

/// The upstream `transitionStatusMapping.transitionStatus`
/// (`packages/react/src/internals/stateAttributesMapping.ts:10-19`) in the
/// [`StateAttributesMapping`] shape: emits the transition hook attributes during
/// `'starting'`/`'ending'` and declines every other value (including idle and the empty
/// value), letting the default handling skip them.
pub fn transition_status_mapping(key: &str, value: &Value) -> Option<Option<StateAttributeProps>> {
    if key != "transitionStatus" {
        return None;
    }

    let hook = match value.as_str() {
        Some("starting") => STARTING_STYLE,
        Some("ending") => ENDING_STYLE,
        _ => return Some(None),
    };

    Some(Some(BTreeMap::from([(hook.to_string(), String::new())])))
}

/// The upstream `fieldValidityMapping.valid`
/// (`packages/react/src/internals/field-constants/constants.ts:34-43`) in the
/// [`StateAttributesMapping`] shape: `true` emits `data-valid`, `false` emits
/// `data-invalid`, and the unknown state (`null`) emits nothing. The attribute names are
/// `FieldControlDataAttributes.valid`/`invalid`
/// (`packages/react/src/field/control/FieldControlDataAttributes.ts:8,12`), defined here
/// until the field unit ports (the module doc notes the deferral).
pub fn field_validity_mapping(key: &str, value: &Value) -> Option<Option<StateAttributeProps>> {
    if key != "valid" {
        return None;
    }

    let attribute = match value {
        Value::Null => return Some(None),
        Value::Bool(true) => "data-valid",
        Value::Bool(false) => "data-invalid",
        _ => return Some(None),
    };

    Some(Some(BTreeMap::from([(
        attribute.to_string(),
        String::new(),
    )])))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn state(pairs: serde_json::Value) -> serde_json::Map<String, Value> {
        pairs.as_object().expect("state object").clone()
    }

    // Mirrors `packages/react/src/internals/getStateAttributesProps.test.ts:5-18`.
    #[test]
    fn converts_the_state_fields_to_data_attributes() {
        let result = get_state_attributes_props(
            &state(json!({
                "checked": true,
                "orientation": "vertical",
                "count": 42,
            })),
            None,
        );

        assert_eq!(
            result,
            BTreeMap::from([
                ("data-checked".to_string(), String::new()),
                ("data-orientation".to_string(), "vertical".to_string()),
                ("data-count".to_string(), "42".to_string()),
            ])
        );
    }

    // Mirrors `packages/react/src/internals/getStateAttributesProps.test.ts:20-29`.
    #[test]
    fn changes_the_field_names_to_lowercase() {
        let result = get_state_attributes_props(&state(json!({ "readOnly": true })), None);

        assert_eq!(
            result,
            BTreeMap::from([("data-readonly".to_string(), String::new())])
        );
    }

    // Mirrors `packages/react/src/internals/getStateAttributesProps.test.ts:31-39`.
    #[test]
    fn changes_true_values_to_a_data_attribute_without_a_value() {
        let result = get_state_attributes_props(
            &state(json!({ "required": true, "disabled": false })),
            None,
        );

        assert_eq!(
            result,
            BTreeMap::from([("data-required".to_string(), String::new())])
        );
    }

    // Mirrors `packages/react/src/internals/getStateAttributesProps.test.ts:41-49`.
    #[test]
    fn does_not_include_false_values() {
        let result = get_state_attributes_props(
            &state(json!({ "required": true, "disabled": false })),
            None,
        );

        assert!(!result.contains_key("data-disabled"));
    }

    // Mirrors `packages/react/src/internals/getStateAttributesProps.test.ts:51-67`.
    #[test]
    fn supports_custom_mapping() {
        let result = get_state_attributes_props(
            &state(json!({
                "checked": true,
                "orientation": "vertical",
                "count": 42,
            })),
            Some(&|key: &str, value: &Value| {
                if key != "checked" {
                    return None;
                }
                Some(Some(BTreeMap::from([(
                    "data-state".to_string(),
                    if value == &Value::Bool(true) {
                        "checked".to_string()
                    } else {
                        "unchecked".to_string()
                    },
                )])))
            }),
        );

        assert_eq!(
            result,
            BTreeMap::from([
                ("data-state".to_string(), "checked".to_string()),
                ("data-orientation".to_string(), "vertical".to_string()),
                ("data-count".to_string(), "42".to_string()),
            ])
        );
    }

    // Mirrors `packages/react/src/internals/getStateAttributesProps.test.ts:69-82`.
    #[test]
    fn supports_nulls_returned_from_custom_mapping() {
        let result = get_state_attributes_props(
            &state(json!({ "checked": false, "orientation": "vertical" })),
            Some(&|key: &str, value: &Value| {
                if key != "checked" {
                    return None;
                }
                if value == &Value::Bool(true) {
                    Some(Some(BTreeMap::from([(
                        "data-state".to_string(),
                        "checked".to_string(),
                    )])))
                } else {
                    Some(None)
                }
            }),
        );

        assert_eq!(
            result,
            BTreeMap::from([("data-orientation".to_string(), "vertical".to_string())])
        );
    }

    // Mirrors `packages/react/src/internals/stateAttributesMapping.test.ts:10-17`.
    #[test]
    fn the_transition_status_mapping_emits_the_transition_data_attributes() {
        let starting =
            transition_status_mapping("transitionStatus", &json!("starting")).expect("mapped key");
        assert_eq!(
            starting.expect("props"),
            BTreeMap::from([(STARTING_STYLE.to_string(), String::new())])
        );

        let ending =
            transition_status_mapping("transitionStatus", &json!("ending")).expect("mapped key");
        assert_eq!(
            ending.expect("props"),
            BTreeMap::from([(ENDING_STYLE.to_string(), String::new())])
        );
    }

    // Mirrors `packages/react/src/internals/stateAttributesMapping.test.ts:19-22` (the
    // `undefined` status collapses onto `Value::Null`, the dynamic representation's empty
    // value — same mapping outcome).
    #[test]
    fn the_transition_status_mapping_emits_nothing_outside_a_transition() {
        assert_eq!(
            transition_status_mapping("transitionStatus", &json!("idle")),
            Some(None)
        );
        assert_eq!(
            transition_status_mapping("transitionStatus", &Value::Null),
            Some(None)
        );
    }

    // Mirrors `packages/react/src/internals/stateAttributesMapping.test.ts:26-29` — the
    // attribute names are pinned against literals so a rename cannot slip through
    // (`stateAttributesMapping.test.ts:5-7`).
    #[test]
    fn the_field_validity_mapping_emits_the_validity_data_attributes() {
        let valid = field_validity_mapping("valid", &json!(true)).expect("mapped key");
        assert_eq!(
            valid.expect("props"),
            BTreeMap::from([("data-valid".to_string(), String::new())])
        );

        let invalid = field_validity_mapping("valid", &json!(false)).expect("mapped key");
        assert_eq!(
            invalid.expect("props"),
            BTreeMap::from([("data-invalid".to_string(), String::new())])
        );
    }

    // Mirrors `packages/react/src/internals/stateAttributesMapping.test.ts:31-33`.
    #[test]
    fn the_field_validity_mapping_emits_nothing_while_validity_is_unknown() {
        assert_eq!(field_validity_mapping("valid", &Value::Null), Some(None));
    }
}
