//! Port of `packages/react/src/utils/resolveAriaLabelledBy.ts` — the label-id resolution
//! helpers the Field/Slider/Progress/Meter form parts use to link their controls to an
//! accessible name (consumed through `internals/labelable-provider` per the unit's
//! implementation spec).
//!
//! No test in the unit targets these helpers directly (implementation.md,
//! "Anything in source not explained by any test" — covered implicitly through the
//! labelable-provider component tests).

/// Port of `getDefaultLabelId` (`resolveAriaLabelledBy.ts:3-5`): the `id == null`
/// check (`null` or `undefined` → `undefined`) maps to [`Option`], so a present id —
/// including the empty string — produces the `{id}-label` default.
pub fn get_default_label_id(id: Option<&str>) -> Option<String> {
    id.map(|id| format!("{id}-label"))
}

/// Port of `resolveAriaLabelledBy` (`resolveAriaLabelledBy.ts:7-11`): the field-level
/// label id wins over the local one — the JS `??` chain, where only `null`/`undefined`
/// fall through (an empty string is a present value and stays).
pub fn resolve_aria_labelled_by(
    field_label_id: Option<&str>,
    local_label_id: Option<&str>,
) -> Option<String> {
    field_label_id.or(local_label_id).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The `id == null ? undefined : \`${id}-label\`` matrix (`:3-5`).
    #[test]
    fn get_default_label_id_appends_the_label_suffix() {
        assert_eq!(
            get_default_label_id(Some("ctrl")),
            Some("ctrl-label".to_string())
        );
        assert_eq!(get_default_label_id(None), None);
        // The empty string is a present id, not nullish.
        assert_eq!(get_default_label_id(Some("")), Some("-label".to_string()));
    }

    // The `fieldLabelId ?? localLabelId` chain (`:7-11`).
    #[test]
    fn resolve_aria_labelled_by_prefers_the_field_label() {
        assert_eq!(
            resolve_aria_labelled_by(Some("field-label"), Some("local-label")),
            Some("field-label".to_string())
        );
        assert_eq!(
            resolve_aria_labelled_by(None, Some("local-label")),
            Some("local-label".to_string())
        );
        assert_eq!(resolve_aria_labelled_by(None, None), None);
        // A present-but-empty field id wins over the local one (`??` semantics).
        assert_eq!(
            resolve_aria_labelled_by(Some(""), Some("local-label")),
            Some(String::new())
        );
    }
}
