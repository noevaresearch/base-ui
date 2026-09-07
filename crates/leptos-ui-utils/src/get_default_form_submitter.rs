//! Port of `packages/utils/src/getDefaultFormSubmitter.ts` (Base UI Phase A util).
//!
//! Upstream is a single-export pure function that returns the default button a browser uses
//! for implicit form submission — the element a custom form control can click to mirror
//! native Enter-key behavior, preserving browser semantics such as the submitter's click
//! event, `SubmitEvent.submitter`, and submitter-specific attributes
//! (`packages/utils/src/getDefaultFormSubmitter.ts:3-8`). The unit's sole test file pins the
//! behaviors this port is tested against (`packages/utils/src/getDefaultFormSubmitter.test.ts`,
//! mirrored in `specs/utils/getDefaultFormSubmitter.md`).
//!
//! The function follows the controls exposed by `form.elements`, which includes controls
//! associated through the `form` attribute, and deliberately does not filter out disabled
//! submitters — the default button is determined before disabled state is considered, and
//! clicking a disabled submitter is a no-op
//! (`packages/utils/src/getDefaultFormSubmitter.ts:10-12`). Delegating the traversal to the
//! DOM's own `form.elements` collection is what preserves those platform semantics, including
//! `input[type="image"]`'s absence from the collection
//! (`packages/utils/src/getDefaultFormSubmitter.ts:19`, `:25-26`); hand-rolling a DOM query
//! here would re-derive browser behavior the collection already encodes.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The `DefaultFormSubmitter` union (`HTMLButtonElement | HTMLInputElement`,
//!   `packages/utils/src/getDefaultFormSubmitter.ts:1`) becomes the [`DefaultFormSubmitter`]
//!   enum, preserving which concrete kind was matched. Callers that only need to click the
//!   submitter — the upstream use case
//!   (`packages/react/src/checkbox/root/CheckboxRoot.tsx:361`) — can go through
//!   [`DefaultFormSubmitter::as_html_element`], the common base of both union members.
//! - The nullable `form` parameter (`packages/utils/src/getDefaultFormSubmitter.ts:14-17`)
//!   becomes [`Option`]: `None` maps to upstream's `null` argument and yields a `None` return.
//! - The returned element handle references the same JS node the collection exposed (a clone
//!   of the web-sys handle, not a copy of the node), so identity comparisons — the upstream
//!   tests' `toBe` assertions (`packages/utils/src/getDefaultFormSubmitter.test.ts:23`, `:68`)
//!   — hold against a `querySelector` result for the same node.

use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlElement, HtmlFormElement, HtmlInputElement};

/// The upstream `DefaultFormSubmitter` export
/// (`packages/utils/src/getDefaultFormSubmitter.ts:1`): the concrete element kinds a form's
/// default submitter can be.
#[derive(Debug)]
pub enum DefaultFormSubmitter {
    /// A `<button>` submitter — explicit `type="submit"` or the implicit default submit type.
    Button(HtmlButtonElement),
    /// An `<input type="submit">` submitter.
    Input(HtmlInputElement),
}

impl DefaultFormSubmitter {
    /// The submitter as an [`HtmlElement`] — the common base both union members share, and
    /// the level the click-to-submit use case needs
    /// (`packages/react/src/checkbox/root/CheckboxRoot.tsx:361`).
    pub fn as_html_element(&self) -> &HtmlElement {
        match self {
            DefaultFormSubmitter::Button(button) => button.as_ref(),
            DefaultFormSubmitter::Input(input) => input.as_ref(),
        }
    }
}

impl AsRef<HtmlElement> for DefaultFormSubmitter {
    fn as_ref(&self) -> &HtmlElement {
        self.as_html_element()
    }
}

/// The upstream `getDefaultFormSubmitter` export
/// (`packages/utils/src/getDefaultFormSubmitter.ts:14-34`): returns the default button a
/// browser uses for implicit form submission, or [`None`] when the form is absent or exposes
/// no submitter.
///
/// The traversal follows `form.elements` — including controls associated through the `form`
/// attribute, in collection order — and does not filter out disabled submitters; the first
/// submitter wins. See the module docs for the preserved platform semantics.
pub fn get_default_form_submitter(form: Option<&HtmlFormElement>) -> Option<DefaultFormSubmitter> {
    let Some(form) = form else {
        return None;
    };

    let elements = form.elements();
    (0..elements.length()).find_map(|index| {
        let candidate = elements.get_with_index(index)?;
        let tag_name = candidate.tag_name();

        if tag_name == "BUTTON" {
            let button = candidate.unchecked_ref::<HtmlButtonElement>();
            // Intentionally excludes input[type="image"]: Chromium omits it from
            // form.elements, so supporting it would require separate traversal for an exotic
            // submitter type (`packages/utils/src/getDefaultFormSubmitter.ts:25-26`).
            (button.type_() == "submit").then(|| DefaultFormSubmitter::Button(button.clone()))
        } else if tag_name == "INPUT" {
            let input = candidate.unchecked_ref::<HtmlInputElement>();
            (input.type_() == "submit").then(|| DefaultFormSubmitter::Input(input.clone()))
        } else {
            None
        }
    })
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Document, Element};

    use super::*;

    // The behavior under test is `form.elements` traversal — form-attribute association and
    // tree order are DOM semantics Node's environment does not implement — so the tests run
    // in a real browser via the wasm32 test runner (`.cargo/config.toml` wires it to
    // chromedriver), like the crate's other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    thread_local! {
        /// Unique suffix per fixture container, so a container left behind by a panicking
        /// test can never collide with a later test's `#test-form` lookup.
        static FIXTURE_SEQ: Cell<u32> = const { Cell::new(0) };
    }

    fn document() -> Document {
        web_sys::window()
            .unwrap_throw()
            .document()
            .unwrap_throw()
    }

    // Mirrors the upstream test harness `setup`
    // (`packages/utils/src/getDefaultFormSubmitter.test.ts:5-8`): builds the fixture markup
    // and returns the `#test-form` element — but inside a dedicated container appended to
    // the body instead of replacing the body's content. The wasm-bindgen-test harness
    // renders its own output elements into the body, so the upstream body replacement
    // would delete them in this harness (upstream runs under jsdom, where nothing else
    // lives in the body). The per-call container also isolates fixtures without an
    // `afterEach` sweep (`packages/utils/src/getDefaultFormSubmitter.test.ts:10-12`);
    // callers remove it when the test body completes.
    fn setup(document: &Document, html: &str) -> (HtmlFormElement, Element) {
        let seq = FIXTURE_SEQ.with(|seq| {
            seq.set(seq.get() + 1);
            seq.get()
        });
        let container = document.create_element("div").unwrap_throw();
        container
            .set_attribute(&format!("data-fixture-{seq}"), "")
            .unwrap_throw();
        container.set_inner_html(html);
        document
            .body()
            .unwrap_throw()
            .append_child(&container)
            .unwrap_throw();
        let form = container
            .query_selector("#test-form")
            .unwrap_throw()
            .expect("#test-form to exist in the fixture markup")
            .unchecked_into::<HtmlFormElement>();
        (form, container)
    }

    fn query_element(scope: &Element, selector: &str) -> Element {
        scope
            .query_selector(selector)
            .unwrap_throw()
            .unwrap_or_else(|| panic!("{selector} to exist in the fixture markup"))
    }

    // Upstream asserts with `toBe` — strict node identity
    // (`packages/utils/src/getDefaultFormSubmitter.test.ts:23`, `:68`); the returned handle
    // must reference the same JS node, not a copy of it.
    fn assert_same_node(submitter: &DefaultFormSubmitter, expected: &Element) {
        let submitter_value: &JsValue =
            <HtmlElement as AsRef<JsValue>>::as_ref(submitter.as_html_element());
        let expected_value: &JsValue = <Element as AsRef<JsValue>>::as_ref(expected);
        // `JsValue`'s `PartialEq` is the JS `===` operator, the same identity semantics as
        // the upstream `toBe` assertions.
        assert!(
            submitter_value == expected_value,
            "submitter is not the expected node"
        );
    }

    // Mirrors `packages/utils/src/getDefaultFormSubmitter.test.ts:14-24`: a submitter
    // associated through the `form` attribute from outside the form element is found, and
    // one placed before the form in document order wins over an internal submit button.
    #[wasm_bindgen_test]
    fn returns_the_first_submit_button_associated_with_the_form() {
        let document = document();
        let (form, container) = setup(
            &document,
            r#"
              <button id="external-before" form="test-form" type="submit">External before</button>
              <form id="test-form">
                <input type="checkbox" />
                <button id="internal" type="submit">Internal</button>
              </form>
            "#,
        );

        let submitter = get_default_form_submitter(Some(&form))
            .expect("the external form-attribute submitter to be found");

        assert_same_node(&submitter, &query_element(&container, "#external-before"));
        assert!(
            matches!(submitter, DefaultFormSubmitter::Button(_)),
            "external submitter to narrow to the Button variant"
        );
        container.remove();
    }

    // Mirrors `packages/utils/src/getDefaultFormSubmitter.test.ts:26-35`: disabled
    // submitters are not filtered out — the default button is determined before disabled
    // state is considered (`packages/utils/src/getDefaultFormSubmitter.ts:10-12`).
    #[wasm_bindgen_test]
    fn returns_a_disabled_submit_button_when_it_is_first_in_form_elements() {
        let document = document();
        let (form, container) = setup(
            &document,
            r#"
              <form id="test-form">
                <button id="disabled" type="submit" disabled>Disabled</button>
                <button id="enabled" type="submit">Enabled</button>
              </form>
            "#,
        );

        let submitter = get_default_form_submitter(Some(&form))
            .expect("the first (disabled) submitter to be found");

        assert_same_node(&submitter, &query_element(&container, "#disabled"));
        container.remove();
    }

    // Mirrors `packages/utils/src/getDefaultFormSubmitter.test.ts:37-46`: a `<button>` with
    // no `type` attribute (implicit submit type) counts as a submitter and wins over a
    // later explicit `type="submit"` button.
    #[wasm_bindgen_test]
    fn supports_buttons_with_the_default_submit_type() {
        let document = document();
        let (form, container) = setup(
            &document,
            r#"
              <form id="test-form">
                <button id="default">Default</button>
                <button id="explicit" type="submit">Explicit</button>
              </form>
            "#,
        );

        let submitter =
            get_default_form_submitter(Some(&form)).expect("the implicit submitter to be found");

        assert_same_node(&submitter, &query_element(&container, "#default"));
        container.remove();
    }

    // Mirrors `packages/utils/src/getDefaultFormSubmitter.test.ts:48-58`: `<input
    // type="submit">` counts as a submitter, while non-submit controls (`type="button"`
    // button, `type="reset"` input) preceding it do not match.
    #[wasm_bindgen_test]
    fn supports_input_submitters_from_form_elements_and_ignores_non_submit_controls() {
        let document = document();
        let (form, container) = setup(
            &document,
            r#"
              <form id="test-form">
                <button id="button" type="button">Button</button>
                <input id="reset" type="reset" />
                <input id="submit" type="submit" />
              </form>
            "#,
        );

        let submitter =
            get_default_form_submitter(Some(&form)).expect("the input submitter to be found");

        assert_same_node(&submitter, &query_element(&container, "#submit"));
        assert!(
            matches!(submitter, DefaultFormSubmitter::Input(_)),
            "input submitter to narrow to the Input variant"
        );
        container.remove();
    }

    // Mirrors `packages/utils/src/getDefaultFormSubmitter.test.ts:60-69`: no submitter
    // present (checkbox + `type="button"` button only) → `None`.
    #[wasm_bindgen_test]
    fn returns_none_when_there_is_no_default_submitter() {
        let document = document();
        let (form, container) = setup(
            &document,
            r#"
              <form id="test-form">
                <input type="checkbox" />
                <button type="button">Button</button>
              </form>
            "#,
        );

        assert!(get_default_form_submitter(Some(&form)).is_none());
        container.remove();
    }

    // Implementation-derived (`packages/utils/src/getDefaultFormSubmitter.ts:14-17`): the
    // upstream signature accepts `null` and short-circuits to `null`; the spec marks the
    // contract UNVERIFIED by tests (`specs/utils/getDefaultFormSubmitter.md`), asserted here
    // from the cited implementation.
    #[wasm_bindgen_test]
    fn accepts_a_null_form() {
        assert!(get_default_form_submitter(None).is_none());
    }

    // Implementation-derived (`packages/utils/src/getDefaultFormSubmitter.ts:22-29`): the
    // no-disabled-filtering rule is not button-specific — the spec notes no test asserts a
    // disabled `<input type="submit">` (`specs/utils/getDefaultFormSubmitter.md`), so this
    // pins the INPUT branch of the cited implementation.
    #[wasm_bindgen_test]
    fn returns_a_disabled_input_submitter() {
        let document = document();
        let (form, container) = setup(
            &document,
            r#"
              <form id="test-form">
                <input id="disabled-submit" type="submit" disabled />
                <button id="enabled" type="submit">Enabled</button>
              </form>
            "#,
        );

        let submitter = get_default_form_submitter(Some(&form))
            .expect("the disabled input submitter to be found");

        assert_same_node(&submitter, &query_element(&container, "#disabled-submit"));
        assert!(
            matches!(submitter, DefaultFormSubmitter::Input(_)),
            "input submitter to narrow to the Input variant"
        );
        container.remove();
    }
}
