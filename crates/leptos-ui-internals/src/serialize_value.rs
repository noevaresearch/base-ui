//! Port of `packages/react/src/internals/serializeValue.ts:1-13` — the JSON-with-fallback
//! stringification the `resolveValueLabel` machinery bottoms out on
//! (`packages/react/src/internals/resolveValueLabel.tsx:80`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The dynamic value representation is [`Value`] (the `state_attributes` precedent), so
//!   the `value == null` guard (`serializeValue.ts:2-4`) covers `Value::Null` — JS's
//!   `undefined`/`null` pair collapses onto it, both stringifying to `''` upstream.
//! - `JSON.stringify` (`:9`) ports to [`serde_json::to_string`]. Two upstream failure paths
//!   cannot occur on this representation and are kept only for contract symmetry: a
//!   circular structure (the `catch` → `String(value)` fallback, `:10-12`) — [`Value`] is
//!   acyclic by construction — and `JSON.stringify(NaN)`'s `"null"` output, since
//!   [`serde_json::Number`] cannot represent `NaN` (the dynamic representation's numbers
//!   are always finite, matching JSON).

use serde_json::Value;

use crate::state_attributes::js_to_string;

/// The upstream `serializeValue`
/// (`packages/react/src/internals/serializeValue.ts:1-13`): `''` for the empty value, the
/// string itself, else its JSON form (falling back to the `String()` coercion if JSON
/// serialization fails, which the [`Value`] representation cannot provoke — see the module
/// docs).
pub fn serialize_value(value: &Value) -> String {
    if value.is_null() {
        return String::new();
    }
    if let Value::String(string) = value {
        return string.clone();
    }
    serde_json::to_string(value).unwrap_or_else(|_| js_to_string(value))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // Port-owned pins (`packages/react/src/internals/serializeValue.ts` has no test file —
    // the behavior spec exercises it only through `resolveValueLabel`): each branch of the
    // upstream function pinned against the output JS produces.
    #[test]
    fn serializes_each_value_kind_like_json_stringify() {
        assert_eq!(serialize_value(&Value::Null), "");
        assert_eq!(serialize_value(&json!("plain")), "plain");
        assert_eq!(serialize_value(&json!(42)), "42");
        assert_eq!(serialize_value(&json!(true)), "true");
        assert_eq!(serialize_value(&json!([1, 2])), "[1,2]");
        assert_eq!(serialize_value(&json!({"a": 1})), r#"{"a":1}"#);
    }
}
