//! Port of `packages/react/src/field/utils/getCombinedFieldValidityData.ts` — the leaf
//! utility combining the field's stateful validity data with the external invalid flag
//! (`specs/library/internals/implementation.md`, "Dependencies on other Base UI internals":
//! `packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:5` —
//! pulled in as a required dependency of the field-register-control port per the
//! stringifyLocale-over-formatNumber precedent, `TODO.md`, utils: stringifyLocale note).
//!
//! Upstream is a single pure function (`getCombinedFieldValidityData.ts:7-15`): spread the
//! validity data and override `state.valid` to `!invalid && validityData.state.valid`. The
//! app-controlled invalidity (the `invalid` prop or `<Form>` errors) keeps the field marked
//! invalid even while disabled; only computed validity is suppressed when disabled
//! (`FieldRoot.tsx:106-108`'s comment — the same rule this function's consumers encode).

use crate::field_constants::FieldValidityData;

/// Port of `getCombinedFieldValidityData` (`getCombinedFieldValidityData.ts:7-15`): the
/// stateful validity data with `state.valid` combined against the external invalid flag —
/// `valid` stays `true` only when the field computed validity AND nothing external marks it
/// invalid. Returns a fresh record (the spread), leaving the input untouched.
pub fn get_combined_field_validity_data(
    validity_data: &FieldValidityData,
    invalid: bool,
) -> FieldValidityData {
    let mut combined = validity_data.clone();
    combined.state.valid = field_validity(validity_data.state.valid, invalid);
    combined
}

/// The `!invalid && validityData.state.valid` expression (`:12`) over the tri-state
/// `valid`: a computed `null` (not yet validated, or disabled-suppressed — module docs on
/// [`FieldValidityState`]) stays `null` regardless of `invalid`.
fn field_validity(valid: Option<bool>, invalid: bool) -> Option<bool> {
    valid.map(|valid| !invalid && valid)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use serde_json::Value;

    use super::*;
    use crate::field_constants::{DEFAULT_VALIDITY_STATE, FieldValidityState};

    fn validity_data(valid: Option<bool>) -> FieldValidityData {
        FieldValidityData {
            state: FieldValidityState {
                valid,
                ..DEFAULT_VALIDITY_STATE
            },
            error: "e".to_string(),
            errors: vec!["e".to_string()],
            value: Value::from("v"),
            initial_value: Value::from("i"),
        }
    }

    // Pins the combination rule (`getCombinedFieldValidityData.ts:12`): external invalidity
    // forces `valid: false`; computed validity survives when nothing external is invalid;
    // the unvalidated `null` passes through unchanged.
    #[test]
    fn validity_combines_against_the_external_invalid_flag() {
        let combined = get_combined_field_validity_data(&validity_data(Some(true)), false);
        assert_eq!(combined.state.valid, Some(true), "valid && !invalid");

        let combined = get_combined_field_validity_data(&validity_data(Some(true)), true);
        assert_eq!(
            combined.state.valid,
            Some(false),
            "external invalidity marks the field invalid"
        );

        let combined = get_combined_field_validity_data(&validity_data(Some(false)), false);
        assert_eq!(
            combined.state.valid,
            Some(false),
            "computed invalidity stands"
        );

        let combined = get_combined_field_validity_data(&validity_data(None), true);
        assert_eq!(
            combined.state.valid, None,
            "the unvalidated null is not coerced"
        );
    }

    // Pins the spread semantics (`:10-13`): everything except `state.valid` is carried
    // through untouched, and the input record is not mutated.
    #[test]
    fn the_combination_only_touches_state_valid() {
        let source = validity_data(Some(true));
        let combined = get_combined_field_validity_data(&source, true);

        assert_eq!(combined.error, "e");
        assert_eq!(combined.errors, vec!["e".to_string()]);
        assert_eq!(combined.value, Value::from("v"));
        assert_eq!(combined.initial_value, Value::from("i"));
        // The untouched flags survive the combination too.
        assert_eq!(
            combined.state.value_missing,
            DEFAULT_VALIDITY_STATE.value_missing
        );
        // The input is left as it was.
        assert_eq!(source.state.valid, Some(true));
    }
}
