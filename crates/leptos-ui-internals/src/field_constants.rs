//! Port of `packages/react/src/internals/field-constants/constants.ts` — the shared field
//! state/validity vocabulary (`specs/library/internals/implementation.md`, "Context
//! providers/consumers" — the `FieldRootContext` row; untested upstream per "Anything in
//! source not explained by any test" item 4, so the tests below pin the written shapes per
//! the PrehydrationScript precedent).
//!
//! Upstream exports four members. [`field_validity_mapping`] (constants.ts:34-43) is NOT
//! duplicated here: it was already ported with the state-attribute machinery
//! ([`crate::state_attributes::field_validity_mapping`], where the module docs record the
//! deferral of these `DEFAULT_*` constants "typed against the not-yet-ported field unit").
//! This module ports the remaining three constants together with the type vocabulary they
//! are declared against — `FieldValidityData` and `FieldRootState`
//! (`packages/react/src/field/root/FieldRoot.tsx:213-270`), pulled in ahead of the Phase B
//! `FieldRoot` component per the stringifyLocale-over-formatNumber precedent
//! (`TODO.md`, utils: stringifyLocale note): the constants are literally typed against
//! these structs upstream, and the form/field context ports of this unit
//! ([`crate::form_context`], [`crate::field_root_context`],
//! [`crate::field_register_control`]) cannot exist without them.
//!
//! ## Rust adaptations
//!
//! - The anonymous `FieldValidityData['state']` object (eleven `ValidityState` flags with
//!   `valid: boolean | null`) ports to [`FieldValidityState`] with `valid: Option<bool>` —
//!   `None` is the "not yet validated" `null` (`DEFAULT_VALIDITY_STATE`, constants.ts:4-16).
//! - `value: unknown` / `initialValue: unknown` (`FieldRoot.tsx:221-222`) port to
//!   `serde_json::Value`, the crate's dynamic-value representation (the
//!   `resolve_value_label` precedent). Upstream's `undefined` and `null` collapse onto
//!   [`Value::Null`] there: every in-unit consumer compares these members for equality or
//!   truthiness, never for the null/undefined distinction, so the collapse is
//!   outcome-equivalent (the `state_attributes.rs` `is_truthy` precedent).
//! - [`DEFAULT_FIELD_STATE_ATTRIBUTES`] (constants.ts:18-27) is upstream's `Pick` of
//!   [`FieldRootState`] — a partial struct has no Rust spelling and no in-unit consumer
//!   reads it separately, so its five values are inlined into [`DEFAULT_FIELD_ROOT_STATE`]
//!   (constants.ts:29-32) and the constant itself is not re-exported.

use serde_json::Value;

/// The `ValidityState`-shaped record behind `FieldValidityData.state`
/// (`FieldRoot.tsx:214-225`; `DEFAULT_VALIDITY_STATE`'s shape, constants.ts:4-16): the ten
/// native constraint flags plus `valid`, where `None` is the "not yet validated" `null`.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldValidityState {
    pub bad_input: bool,
    pub custom_error: bool,
    pub pattern_mismatch: bool,
    pub range_overflow: bool,
    pub range_underflow: bool,
    pub step_mismatch: bool,
    pub too_long: bool,
    pub too_short: bool,
    pub type_mismatch: bool,
    pub value_missing: bool,
    /// `valid: boolean | null` (`FieldRoot.tsx:225`) — `None` is the unvalidated `null`.
    pub valid: Option<bool>,
}

/// `DEFAULT_VALIDITY_STATE` (`constants.ts:4-16`): every flag `false`, `valid` unvalidated.
pub const DEFAULT_VALIDITY_STATE: FieldValidityState = FieldValidityState {
    bad_input: false,
    custom_error: false,
    pattern_mismatch: false,
    range_overflow: false,
    range_underflow: false,
    step_mismatch: false,
    too_long: false,
    too_short: false,
    type_mismatch: false,
    value_missing: false,
    valid: None,
};

/// `FieldValidityData` (`packages/react/src/field/root/FieldRoot.tsx:213-228`): the field's
/// full validity record — the native/stateful flags, the collapsed error message, the error
/// list, and the current/initial values.
///
/// `value`/`initialValue` are `unknown` upstream; the port carries them as
/// [`serde_json::Value`] with `undefined`/`null` collapsed onto [`Value::Null`] (module
/// docs), which is also `initialValue`'s declared default (`FieldRoot.tsx:200`).
#[derive(Clone, Debug, PartialEq)]
pub struct FieldValidityData {
    pub state: FieldValidityState,
    pub error: String,
    pub errors: Vec<String>,
    pub value: Value,
    pub initial_value: Value,
}

impl Default for FieldValidityData {
    /// `Field.Root`'s initial `useState` value (`FieldRoot.tsx:112-119`): the default
    /// validity state, no errors, `null` values.
    fn default() -> Self {
        Self {
            state: DEFAULT_VALIDITY_STATE,
            error: String::new(),
            errors: Vec::new(),
            value: Value::Null,
            initial_value: Value::Null,
        }
    }
}

/// `FieldRootState` (`packages/react/src/field/root/FieldRoot.tsx:237-270`): the state
/// object the field renders through (`data-*` attributes via
/// [`crate::state_attributes::field_validity_mapping`]).
///
/// `valid` is `boolean | null` (`:253`) — `None` is the unvalidated/`disabled`-suppressed
/// `null` (`FieldRoot.tsx:107` computes `disabled ? null : validityData.state.valid`).
#[derive(Clone, Debug, PartialEq)]
pub struct FieldRootState {
    pub disabled: bool,
    pub touched: bool,
    pub dirty: bool,
    pub valid: Option<bool>,
    pub filled: bool,
    pub focused: bool,
}

/// `DEFAULT_FIELD_ROOT_STATE` (`constants.ts:29-32`) — `disabled: false` plus the
/// `DEFAULT_FIELD_STATE_ATTRIBUTES` pick (constants.ts:18-27, inlined; module docs).
pub const DEFAULT_FIELD_ROOT_STATE: FieldRootState = FieldRootState {
    disabled: false,
    touched: false,
    dirty: false,
    valid: None,
    filled: false,
    focused: false,
};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins `DEFAULT_VALIDITY_STATE` (constants.ts:4-16): the ten native flags are `false`
    // and `valid` is the unvalidated `null` (the port's `None`).
    #[test]
    fn default_validity_state_is_all_false_and_unvalidated() {
        let state = DEFAULT_VALIDITY_STATE;
        assert!(!state.bad_input);
        assert!(!state.custom_error);
        assert!(!state.pattern_mismatch);
        assert!(!state.range_overflow);
        assert!(!state.range_underflow);
        assert!(!state.step_mismatch);
        assert!(!state.too_long);
        assert!(!state.too_short);
        assert!(!state.type_mismatch);
        assert!(!state.value_missing);
        assert_eq!(state.valid, None, "valid starts unvalidated (null)");
    }

    // Pins `DEFAULT_FIELD_ROOT_STATE` (constants.ts:29-32 over :18-27): the state-attribute
    // pick values plus `disabled: false`.
    #[test]
    fn default_root_state_matches_the_constants() {
        let state = DEFAULT_FIELD_ROOT_STATE;
        assert!(!state.disabled);
        assert_eq!(state.valid, None);
        assert!(!state.touched);
        assert!(!state.dirty);
        assert!(!state.filled);
        assert!(!state.focused);
    }

    // Pins `Field.Root`'s initial validity data (`FieldRoot.tsx:112-119`): the default
    // state, no errors, `null` values — the default the no-provider context shells build
    // on (FieldRootContext.ts:35-41, FormContext consumers).
    #[test]
    fn default_validity_data_matches_field_roots_initial_state() {
        let data = FieldValidityData::default();
        assert_eq!(data.state, DEFAULT_VALIDITY_STATE);
        assert_eq!(data.error, "");
        assert!(data.errors.is_empty());
        assert_eq!(data.value, Value::Null);
        assert_eq!(data.initial_value, Value::Null);
    }
}
