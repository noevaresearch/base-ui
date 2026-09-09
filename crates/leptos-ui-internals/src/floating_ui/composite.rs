//! Port of the list-navigation helpers in
//! `packages/react/src/floating-ui-react/utils/composite.ts` that
//! [`crate::floating_ui::use_typeahead`] consumes — the [`DisabledIndices`]
//! vocabulary, [`is_list_index_disabled`], and the visibility pair
//! ([`is_element_visible`]/[`is_hidden_by_styles`])
//! (`specs/library/floating-ui-react/implementation.md`, "Notable per-hook state
//! machines": grid navigation "is a positional-arg shim … with the row-structure
//! inference and virtualized-gap logic living in `utils/composite`").
//!
//! The rest of `composite.ts` (the `getGridNavigatedIndex` matrix and the grid-cell
//! map) is ported with the `useListNavigation`/`gridNavigation` checkpoint — this
//! module only carries the subset the typeahead hook imports
//! (`packages/react/src/floating-ui-react/hooks/useTypeahead.ts:7`).
//!
//! ## Rust adaptations
//!
//! - `DisabledIndices` (`composite.ts:7` — `ReadonlyArray<number> |
//!   ((index: number) => boolean)`) becomes [`DisabledIndices`]: a list arm and a
//!   predicate arm. Indices are `i32` so the `-1` "no index" sentinel the navigation
//!   hooks use flows through unchanged.
//! - `isElementVisible(element, styles?)` (`composite.ts:511-527`) folds its optional
//!   computed-styles parameter into the function body — every current caller resolves
//!   the element's own styles, and a Rust caller holding styles for a *different*
//!   element would be a bug the JS signature allows silently. `checkVisibility` keeps
//!   its upstream feature-detect (`:520`) — resolved through `Reflect` so a browser
//!   without the API (or a polyfill-less runtime) takes the manual
//!   `display !== 'none' && display !== 'contents'` fallback exactly as upstream.
//! - `text[0]`/`text[1]` indexing in the typeahead's doubled-letter bail-out is
//!   UTF-16-code-unit indexing upstream; the port compares the first two `char`s —
//!   identical for every realistic list label, differing only for astral-plane
//!   leading characters (see [`crate::floating_ui::use_typeahead`]).
//! - `element.matches(':disabled')` (`composite.ts:496`) can only throw for an
//!   invalid selector, and `:disabled` is valid — the port maps a hypothetical
//!   rejection to `false` rather than propagating a JsValue error.

use std::rc::Rc;

use web_sys::wasm_bindgen::JsCast;
use web_sys::{CssStyleDeclaration, Element};

/// Port of the `DisabledIndices` type (`composite.ts:7`): the explicit disabled set,
/// either as an array of indices or a per-index predicate — the same shape as
/// `useListNavigation`'s `disabledIndices`
/// (`packages/react/src/floating-ui-react/hooks/useTypeahead.ts:36-43`).
#[derive(Clone)]
pub enum DisabledIndices {
    /// The array arm — `ReadonlyArray<number>`.
    List(Vec<i32>),
    /// The predicate arm — `(index: number) => boolean`.
    Predicate(Rc<dyn Fn(i32) -> bool>),
}

/// Port of `isListIndexDisabled` (`composite.ts:471-505`): whether list entry
/// `index` must be skipped — explicitly disabled through `disabledIndices`, hidden,
/// natively disabled, or (only when no explicit set is given) carrying a
/// `disabled`/`aria-disabled="true"` attribute.
pub fn is_list_index_disabled(
    list: &[Option<Element>],
    index: i32,
    disabled_indices: Option<&DisabledIndices>,
) -> bool {
    let is_explicitly_disabled = match disabled_indices {
        Some(DisabledIndices::Predicate(predicate)) => predicate(index),
        Some(DisabledIndices::List(indices)) => indices.contains(&index),
        None => false,
    };

    if is_explicitly_disabled {
        return true;
    }

    // JS `list[index]` on an out-of-bounds (or negative) index reads `undefined` —
    // the element-less arm (`composite.ts:480-482`).
    let element = if index < 0 {
        None
    } else {
        list.get(index as usize).cloned().flatten()
    };
    let Some(element) = element else {
        return false;
    };

    if !is_element_visible(Some(&element)) {
        return true;
    }

    // A natively disabled element can never receive focus, so it must always be
    // skipped, even when `disabledIndices` marks it as enabled. Only
    // `aria-disabled` items can be focusable-while-disabled (`composite.ts:488-493`).
    if element.matches(":disabled").unwrap_or(false) {
        return true;
    }

    disabled_indices.is_none()
        && (element.has_attribute("disabled")
            || element.get_attribute("aria-disabled").as_deref() == Some("true"))
}

/// Port of `isHiddenByStyles` (`composite.ts:507-509`): the styles-level hidden
/// check — `visibility: hidden|collapse`.
pub fn is_hidden_by_styles(styles: &CssStyleDeclaration) -> bool {
    let visibility = styles.get_property_value("visibility").unwrap_or_default();
    visibility == "hidden" || visibility == "collapse"
}

/// Port of `isElementVisible` (`composite.ts:511-527`): the effective-visibility
/// check — connected, not visibility-hidden, and (through the `checkVisibility`
/// feature-detect or the display fallback) not display-hidden.
pub fn is_element_visible(element: Option<&Element>) -> bool {
    let Some(element) = element else {
        return false;
    };
    if !element.is_connected() {
        return false;
    }
    let styles = web_sys::window()
        .and_then(|window| window.get_computed_style(element).ok())
        .flatten();
    let Some(styles) = styles else {
        return false;
    };
    if is_hidden_by_styles(&styles) {
        return false;
    }

    // `typeof element.checkVisibility === 'function'` (`composite.ts:520`) — the
    // feature-detect resolves through Reflect so the binding needs no web-sys feature.
    let check_visibility = js_sys::Reflect::get(element.as_ref(), &"checkVisibility".into())
        .ok()
        .filter(|value| value.is_function());
    if let Some(function) =
        check_visibility.and_then(|value| value.dyn_into::<js_sys::Function>().ok())
    {
        return js_sys::Reflect::apply(&function, element.as_ref(), &js_sys::Array::new())
            .map(|result| result.is_truthy())
            .unwrap_or(false);
    }

    let display = styles.get_property_value("display").unwrap_or_default();
    display != "none" && display != "contents"
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins the explicit-disabled arms (`composite.ts:474-478`): the predicate arm is
    // consulted per index and the array arm is a membership check; an explicit hit
    // short-circuits before any element is read (so a null entry still reports
    // disabled).
    #[test]
    fn explicit_disabled_sets_decide_before_the_element_is_consulted() {
        let predicate = DisabledIndices::Predicate(Rc::new(|index: i32| index % 2 == 0));
        assert!(is_list_index_disabled(&[], 0, Some(&predicate)));
        assert!(!is_list_index_disabled(&[], 1, Some(&predicate)));

        let list = DisabledIndices::List(vec![2, 5]);
        assert!(is_list_index_disabled(&[], 2, Some(&list)));
        assert!(is_list_index_disabled(&[], 5, Some(&list)));
        assert!(!is_list_index_disabled(&[], 3, Some(&list)));
    }

    // Pins the element-less arm (`composite.ts:480-482`): a missing entry (negative,
    // past-the-end, or a `null` slot) is not disabled by itself — only the explicit
    // set can disable it.
    #[test]
    fn a_missing_element_entry_is_not_disabled() {
        let list: Vec<Option<Element>> = vec![None];
        assert!(!is_list_index_disabled(&list, 0, None));
        assert!(!is_list_index_disabled(&list, -1, None));
        assert!(!is_list_index_disabled(&list, 1, None));
        assert!(is_list_index_disabled(
            &list,
            0,
            Some(&DisabledIndices::List(vec![0]))
        ));
    }

    // Pins the no-explicit-set attribute fallback (`composite.ts:495-497`): without
    // `disabledIndices` the function reports disabled for entries the element list
    // marks — untestable without DOM (the attribute checks need an element), so the
    // element-carrying arms are pinned in the wasm suite.
    #[test]
    fn no_explicit_set_leaves_the_decision_to_the_element_arms() {
        assert!(!is_list_index_disabled(&[], 0, None));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// A connected element — visibility checks require `isConnected`
    /// (`composite.ts:515`), so every fixture is appended to the document body.
    fn attached_element(tag: &str) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let element: web_sys::HtmlElement =
            document.create_element(tag).unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&element).unwrap();
        element
    }

    // Pins the visibility arms through real computed styles
    // (`composite.ts:484-486` + `isElementVisible`): a connected element hidden with
    // `display: none` reports disabled; a visible one does not.
    #[wasm_bindgen_test]
    fn a_display_none_element_is_disabled_and_a_visible_one_is_not() {
        let visible = attached_element("button");
        let hidden = attached_element("button");
        hidden.style().set_property("display", "none").unwrap();
        let list = vec![Some(visible.clone().into()), Some(hidden.clone().into())];

        assert!(is_element_visible(Some(visible.as_ref())));
        assert!(!is_element_visible(Some(hidden.as_ref())));
        assert!(!is_list_index_disabled(&list, 0, None));
        assert!(is_list_index_disabled(&list, 1, None));
    }

    // Pins the visibility:hidden arm (`composite.ts:507-509`): `visibility: hidden`
    // hides through `isHiddenByStyles`, the same check the navigation hooks ride.
    #[wasm_bindgen_test]
    fn a_visibility_hidden_element_is_disabled() {
        let hidden = attached_element("button");
        hidden.style().set_property("visibility", "hidden").unwrap();
        assert!(!is_element_visible(Some(hidden.as_ref())));
        assert!(is_hidden_by_styles(
            &web_sys::window()
                .unwrap()
                .get_computed_style(hidden.as_ref())
                .unwrap()
                .unwrap()
        ));
    }

    // Pins the native-disabled arm (`composite.ts:488-493`): a `:disabled` match
    // always reports disabled, even when an explicit set marks the index enabled.
    #[wasm_bindgen_test]
    fn a_natively_disabled_element_is_disabled_even_when_explicitly_enabled() {
        let disabled = attached_element("button");
        disabled.set_attribute("disabled", "").unwrap();
        let list = vec![Some(disabled.clone().into())];

        assert!(
            is_list_index_disabled(&list, 0, None),
            "the no-explicit-set path reports the :disabled match"
        );
        assert!(
            is_list_index_disabled(&list, 0, Some(&DisabledIndices::List(vec![]))),
            "an explicit empty set still skips the natively disabled element"
        );
    }

    // Pins the `aria-disabled` fallback (`composite.ts:495-497`): without an explicit
    // set, `aria-disabled="true"` reports disabled — but only then, since
    // aria-disabled items are focusable-while-disabled.
    #[wasm_bindgen_test]
    fn an_aria_disabled_element_is_disabled_only_without_an_explicit_set() {
        let aria_disabled = attached_element("button");
        aria_disabled
            .set_attribute("aria-disabled", "true")
            .unwrap();
        let list = vec![Some(aria_disabled.clone().into())];

        assert!(is_list_index_disabled(&list, 0, None));
        assert!(
            !is_list_index_disabled(&list, 0, Some(&DisabledIndices::List(vec![]))),
            "the explicit set owns the decision when provided"
        );
    }

    // Pins the feature-detect fallback ordering (`composite.ts:520-526`): with
    // `checkVisibility` available (real Chrome), visibility flows through it — and a
    // detached element is never visible regardless of styling
    // (`!element.isConnected`, `composite.ts:515`).
    #[wasm_bindgen_test]
    fn a_detached_element_is_not_visible_even_when_styled() {
        let document = web_sys::window().unwrap().document().unwrap();
        let detached: web_sys::HtmlElement = document
            .create_element("button")
            .unwrap()
            .dyn_into()
            .unwrap();
        assert!(!detached.is_connected());
        assert!(!is_element_visible(Some(detached.as_ref())));
    }
}
