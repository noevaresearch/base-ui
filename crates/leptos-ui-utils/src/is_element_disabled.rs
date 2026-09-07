//! Port of `packages/utils/src/isElementDisabled.ts` (Base UI Phase A util).
//!
//! Upstream is a single-export, pure DOM predicate: an element counts as disabled when it is
//! null-ish, carries a `disabled` attribute (any value), or carries `aria-disabled` equal to
//! the exact string `'true'` (`packages/utils/src/isElementDisabled.ts:1-7`). The unit has no
//! test file upstream — `ralph/generated/utils.json:132-140` lists `testFiles: []` — so every
//! behavioral claim is inferred from the unit's own source (`specs/utils/isElementDisabled.md`,
//! all claims UNVERIFIED there) and pinned by this module's own tests rather than a reference
//! suite.
//!
//! The checks are deliberately attribute-only and do not walk ancestors. This is load-bearing
//! for consumers: the Select root uses it as the typeahead `disabledIndices` predicate so
//! hidden force-mounted items used for closed-trigger typeahead aren't dropped by a visibility
//! filter (`packages/react/src/select/root/SelectRoot.tsx:379-384`), and the composite root
//! gates arrow-key interception on it (`packages/react/src/internals/composite/root/
//! useCompositeRoot.ts:229`). An element natively disabled through an ancestor (e.g. inside a
//! disabled `<fieldset>`) or with a `disabled` IDL property but no attribute is therefore
//! reported enabled (`packages/utils/src/isElementDisabled.ts:4-5`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The `HTMLElement | null` parameter becomes [`Option`]: `None` maps to upstream's `null`
//!   (and to a runtime `undefined`, which the loose `element == null` comparison
//!   (`packages/utils/src/isElementDisabled.ts:3`) treats identically). Both map to the
//!   "treated as disabled" result, so one type carries the whole null-ish branch.
//! - The `boolean` return becomes [`bool`].

use web_sys::HtmlElement;

/// The upstream `isElementDisabled` export
/// (`packages/utils/src/isElementDisabled.ts:1-7`): returns `true` when the element is absent
/// ([`None`]), has a `disabled` attribute with any value, or has `aria-disabled="true"` —
/// comparing `aria-disabled` strictly against the lowercase string, so `"false"`, `"TRUE"`,
/// or an empty value do not count. Only attributes are consulted: no properties, no ancestor
/// walk, no caching.
pub fn is_element_disabled(element: Option<&HtmlElement>) -> bool {
    match element {
        None => true,
        Some(element) => {
            element.has_attribute("disabled")
                || element.get_attribute("aria-disabled").as_deref() == Some("true")
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Document, HtmlElement};

    use super::*;

    // The behavior under test is DOM attribute reads — `hasAttribute`/`getAttribute` — so the
    // tests run in a real browser via the wasm32 test runner (`.cargo/config.toml` wires it to
    // chromedriver), like the crate's other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> Document {
        web_sys::window()
            .unwrap_throw()
            .document()
            .unwrap_throw()
    }

    // All fixtures are detached (`createElement` without `append_child`): the predicate reads
    // only the passed element's retained attributes, so a detached element is still evaluated
    // correctly (`specs/utils/isElementDisabled.md`, "Edge cases"), and nothing needs cleanup
    // from the shared document body.
    fn make_element(document: &Document, tag: &str) -> HtmlElement {
        document
            .create_element(tag)
            .unwrap_throw()
            .unchecked_into::<HtmlElement>()
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:3`): the leading
    // `element == null` short-circuit treats a null-ish element as disabled — and the loose
    // equality also accepts a runtime `undefined` (`specs/utils/isElementDisabled.md` marks
    // this UNVERIFIED by upstream tests); the port's `None` covers both.
    #[wasm_bindgen_test]
    fn treats_a_nullish_element_as_disabled() {
        assert!(is_element_disabled(None));
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:4-5`): an element with
    // neither relevant attribute is enabled.
    #[wasm_bindgen_test]
    fn treats_a_plain_element_as_enabled() {
        let document = document();
        let element = make_element(&document, "div");
        assert!(!is_element_disabled(Some(&element)));
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:4`): `hasAttribute` is
    // presence-based, so even the invalid value `disabled="false"` counts as disabled
    // (`specs/utils/isElementDisabled.md`, "Edge cases").
    #[wasm_bindgen_test]
    fn counts_a_disabled_attribute_with_any_value() {
        let document = document();
        let element = make_element(&document, "button");
        element.set_attribute("disabled", "").unwrap_throw();
        assert!(is_element_disabled(Some(&element)));

        let element = make_element(&document, "button");
        element.set_attribute("disabled", "false").unwrap_throw();
        assert!(is_element_disabled(Some(&element)));
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:5`): the string form of
    // the check — `aria-disabled="true"` without a `disabled` attribute.
    #[wasm_bindgen_test]
    fn counts_aria_disabled_true() {
        let document = document();
        let element = make_element(&document, "button");
        element.set_attribute("aria-disabled", "true").unwrap_throw();
        assert!(is_element_disabled(Some(&element)));
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:5`): the comparison is
    // strictly against the string `'true'`, so other values do not count
    // (`specs/utils/isElementDisabled.md`, "Accessibility").
    #[wasm_bindgen_test]
    fn requires_the_exact_string_true_for_aria_disabled() {
        let document = document();
        for value in ["false", "TRUE", ""] {
            let element = make_element(&document, "button");
            element.set_attribute("aria-disabled", value).unwrap_throw();
            assert!(
                !is_element_disabled(Some(&element)),
                "aria-disabled={value:?} must not count as disabled"
            );
        }
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:4-5`): the two checks
    // are OR-ed — attribute presence on the first check decides even when `aria-disabled`
    // holds a non-`'true'` value.
    #[wasm_bindgen_test]
    fn combines_both_checks_with_or() {
        let document = document();
        let element = make_element(&document, "button");
        element.set_attribute("disabled", "false").unwrap_throw();
        element
            .set_attribute("aria-disabled", "false")
            .unwrap_throw();
        assert!(is_element_disabled(Some(&element)));
    }

    // Implementation-derived (`packages/utils/src/isElementDisabled.ts:4-5`): attribute-only,
    // no ancestor walk — an element inside a container carrying `disabled` is still reported
    // enabled (`specs/utils/isElementDisabled.md`, "Edge cases"); the Select root's typeahead
    // relies on exactly this semantics
    // (`packages/react/src/select/root/SelectRoot.tsx:379-384`).
    #[wasm_bindgen_test]
    fn does_not_walk_ancestors() {
        let document = document();
        let fieldset = make_element(&document, "fieldset");
        fieldset.set_attribute("disabled", "").unwrap_throw();
        let input = make_element(&document, "input");
        fieldset.append_child(&input).unwrap_throw();

        assert!(is_element_disabled(Some(&fieldset)));
        assert!(!is_element_disabled(Some(&input)));
    }
}
