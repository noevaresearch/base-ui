//! Port of `packages/react/src/floating-ui-react/utils/createAttribute.ts` — the
//! `data-base-ui-*` attribute-name factory the unit's components use for their
//! marker attributes (e.g. the portal host's `data-base-ui-portal`,
//! `components/FloatingPortal.tsx:127-143`).

/// `createAttribute(name)` (`createAttribute.ts:1-3`): prefixes the name with
/// `data-base-ui-`.
pub fn create_attribute(name: &str) -> String {
    format!("data-base-ui-{name}")
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins the prefix (`createAttribute.ts:2`).
    #[test]
    fn the_attribute_name_is_prefixed() {
        assert_eq!(create_attribute("portal"), "data-base-ui-portal");
    }
}
