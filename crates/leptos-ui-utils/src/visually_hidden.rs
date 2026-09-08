//! Port of `packages/utils/src/visuallyHidden.ts` (Base UI Phase A util).
//!
//! Upstream exports exactly two style-constant objects typed `React.CSSProperties`:
//! `visuallyHidden` (`packages/utils/src/visuallyHidden.ts:14-19`) and `visuallyHiddenInput`
//! (`packages/utils/src/visuallyHidden.ts:21-24`). Both spread one private base object
//! (`packages/utils/src/visuallyHidden.ts:3-12`) and differ only in `position`:
//! `visuallyHidden` is `position: fixed` anchored at the viewport's top-left corner via
//! `top: 0; left: 0`, while `visuallyHiddenInput` is `position: absolute` with no offsets,
//! leaving the element at its static position within its parent. There are no functions, no
//! hooks, no components, and no default export — the unit is pure style data
//! (`packages/utils/src/visuallyHidden.ts:1-24`).
//!
//! The declarations implement the standard CSS "visually hidden" technique: `clip-path:
//! inset(50%)` renders the box invisible without removing it from the accessibility tree, on
//! a 1×1 px box with zero border/padding, `overflow: hidden`, `white-space: nowrap`, and a
//! negative margin to neutralize layout side effects — deliberately avoiding `display: none`
//! and `visibility: hidden`, the hiding mechanisms the cross-unit tabbable suite proves
//! remove elements from the tab order
//! (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:123-131`,
//! `packages/react/src/floating-ui-react/utils/tabbable.test.ts:224-234`). That suite also
//! contains the only test-proven behavior available for this unit — a button carrying
//! `visuallyHidden` and a checkbox input carrying `visuallyHiddenInput` remain in the tab
//! order (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-315`,
//! `packages/react/src/floating-ui-react/utils/tabbable.test.ts:317-326`). The unit has no
//! test file of its own (`ralph/generated/utils.json:408-413` lists `testFiles: []`), so
//! every other claim is inferred from the unit's own 24-line source and pinned by this
//! module's own tests rather than a reference suite (`specs/utils/visuallyHidden.md` marks
//! those claims UNVERIFIED).
//!
//! Consumers apply the constants to elements that must stay functional — native input,
//! focusable, form-submittable — while invisible: component roots hide a real input this way
//! (Switch `packages/react/src/switch/root/SwitchRoot.tsx:6`, FocusGuard
//! `packages/react/src/utils/FocusGuard.tsx:5`; the spec's "Public API surface" lists the
//! full consumer set).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - A `React.CSSProperties` object literal maps to a static slice of `(CSS property, value)`
//!   pairs in the upstream key order, with the camelCase JS style-property names converted to
//!   the kebab-case CSS names that `CssStyleDeclaration::set_property` consumes (upstream
//!   reaches the DOM through React's style serialization or `Object.assign(element.style,
//!   ...)`, `packages/react/src/floating-ui-react/utils/tabbable.test.ts:310`).
//! - Upstream's numeric declarations — `border: 0`, `padding: 0`, `width: 1`, `height: 1`,
//!   `margin: -1` (`packages/utils/src/visuallyHidden.ts:7-11`) and `top: 0`/`left: 0`
//!   (`packages/utils/src/visuallyHidden.ts:17-18`) — are React's `CSSProperties` numeric
//!   form, which React's style serializer renders by appending `px` (none of these properties
//!   are unitless). The port pins those effective values, since the bare numerals are invalid
//!   CSS on their own.
//! - The shared-base spread (`...visuallyHiddenBase`,
//!   `packages/utils/src/visuallyHidden.ts:14-24`) maps to a private macro that spells the
//!   base declarations once and appends each constant's own tail, keeping the two constants
//!   in sync at zero runtime cost.
//! - The upstream exports are plain mutable object literals, so a caller mutating one would
//!   affect every consumer process-wide (UNVERIFIED as a contract —
//!   `specs/utils/visuallyHidden.md`, "State model"); the port's `&'static` slice makes that
//!   hazard structurally impossible.

/// Spells the shared base declarations (upstream `visuallyHiddenBase`,
/// `packages/utils/src/visuallyHidden.ts:3-12`) once and appends each export's own tail,
/// mirroring the upstream `...visuallyHiddenBase` spread at compile time.
macro_rules! visually_hidden_declarations {
    ($($tail:expr),* $(,)?) => {
        &[
            ("clip-path", "inset(50%)"),
            ("overflow", "hidden"),
            ("white-space", "nowrap"),
            ("border", "0px"),
            ("padding", "0px"),
            ("width", "1px"),
            ("height", "1px"),
            ("margin", "-1px"),
            $($tail,)*
        ]
    };
}

/// The upstream `visuallyHidden` constant (`packages/utils/src/visuallyHidden.ts:14-19`):
/// visually hides an element while keeping it rendered, focusable, and in the accessibility
/// tree — `position: fixed`, anchored at the viewport's top-left corner. Apply it by
/// iterating the slice and `CssStyleDeclaration::set_property`-ing each pair, the way
/// upstream's `Object.assign(element.style, ...)` consumers do
/// (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:310`).
pub const VISUALLY_HIDDEN: &[(&str, &str)] =
    visually_hidden_declarations![("position", "fixed"), ("top", "0px"), ("left", "0px"),];

/// The upstream `visuallyHiddenInput` constant (`packages/utils/src/visuallyHidden.ts:21-24`):
/// the visually-hidden declarations with `position: absolute` and no `top`/`left` offsets,
/// leaving the element at its static position within its parent — the form applied to hidden
/// native inputs (Switch, Checkbox, Radio; see the module docs).
pub const VISUALLY_HIDDEN_INPUT: &[(&str, &str)] =
    visually_hidden_declarations![("position", "absolute")];

#[cfg(test)]
mod tests {
    use super::*;

    // Pins the full declaration list of `visuallyHidden` in upstream key order — the base
    // (`packages/utils/src/visuallyHidden.ts:3-12`) plus the fixed positioning tail
    // (`packages/utils/src/visuallyHidden.ts:14-19`). Upstream has no test file
    // (`ralph/generated/utils.json:408-413`), so this is the port's own pinning of the
    // inferred contract, not a mirror of a reference suite. The px-suffixed numerals are the
    // effective `CSSProperties` values (see the module docs).
    #[test]
    fn visually_hidden_is_the_base_plus_fixed_positioning() {
        let expected: &[(&str, &str)] = &[
            ("clip-path", "inset(50%)"),
            ("overflow", "hidden"),
            ("white-space", "nowrap"),
            ("border", "0px"),
            ("padding", "0px"),
            ("width", "1px"),
            ("height", "1px"),
            ("margin", "-1px"),
            ("position", "fixed"),
            ("top", "0px"),
            ("left", "0px"),
        ];
        assert_eq!(VISUALLY_HIDDEN, expected);
    }

    // Pins `visuallyHiddenInput` (`packages/utils/src/visuallyHidden.ts:21-24`): the same
    // base with `position: absolute` and no `top`/`left` offsets.
    #[test]
    fn visually_hidden_input_is_the_base_plus_absolute_positioning() {
        let expected: &[(&str, &str)] = &[
            ("clip-path", "inset(50%)"),
            ("overflow", "hidden"),
            ("white-space", "nowrap"),
            ("border", "0px"),
            ("padding", "0px"),
            ("width", "1px"),
            ("height", "1px"),
            ("margin", "-1px"),
            ("position", "absolute"),
        ];
        assert_eq!(VISUALLY_HIDDEN_INPUT, expected);
    }

    // The fixed-vs-absolute split is the unit's only behavioral fork
    // (`specs/utils/visuallyHidden.md`, "Edge cases"): the base occupies the first eight
    // slots of both exports, only the fixed variant carries the `top`/`left` anchors, and the
    // `position` values are the sole remaining difference.
    #[test]
    fn the_two_exports_differ_only_in_position_and_its_offsets() {
        assert_eq!(&VISUALLY_HIDDEN[..8], &VISUALLY_HIDDEN_INPUT[..8]);
        assert!(
            VISUALLY_HIDDEN_INPUT
                .iter()
                .all(|(property, _)| *property != "top" && *property != "left")
        );
        assert_eq!(
            VISUALLY_HIDDEN
                .iter()
                .find(|(property, _)| *property == "position")
                .copied(),
            Some(("position", "fixed"))
        );
        assert_eq!(
            VISUALLY_HIDDEN_INPUT
                .iter()
                .find(|(property, _)| *property == "position")
                .copied(),
            Some(("position", "absolute"))
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{CssStyleDeclaration, HtmlButtonElement, HtmlInputElement};

    use super::*;

    // The behavior under test is CSSOM style application — whether the pinned declarations
    // are real, applicable CSS — so the tests run in a real browser via the wasm32 test
    // runner (`.cargo/config.toml` wires it to chromedriver), like the crate's other wasm
    // test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap_throw().document().unwrap_throw()
    }

    // The upstream consumption pattern proven by the cross-unit tabbable suite
    // (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:310,321`): assign the
    // constant's declarations onto an element's inline style.
    fn apply(style: &CssStyleDeclaration, declarations: &[(&str, &str)]) {
        for (property, value) in declarations {
            style.set_property(property, value).expect_throw(&format!(
                "Base UI: failed to set the {property} visually-hidden style"
            ));
        }
    }

    // Mirrors `packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-315`: the
    // constant applied to a button lands on every declaration, and — the load-bearing
    // contrast of `specs/utils/visuallyHidden.md`, "Keyboard interactions" — hides the
    // element with clip-path alone, never `display: none` / `visibility: hidden`, which is
    // what keeps it keyboard-reachable. Fixtures stay detached: the assertions read only the
    // element's own styles, so nothing needs cleanup from the shared document body.
    #[wasm_bindgen_test]
    fn visually_hidden_applies_to_a_button_and_avoids_the_tab_order_breakers() {
        let document = document();
        let button = document
            .create_element("button")
            .unwrap_throw()
            .unchecked_into::<HtmlButtonElement>();

        apply(&button.style(), VISUALLY_HIDDEN);

        for (property, value) in VISUALLY_HIDDEN {
            assert_eq!(
                button.style().get_property_value(property).unwrap_throw(),
                *value,
                "unexpected inline value for {property}"
            );
        }

        let computed = web_sys::window()
            .unwrap_throw()
            .get_computed_style(&button)
            .expect_throw("Base UI: failed to read the computed style")
            .expect_throw("Base UI: the element has no computed style");
        assert_ne!(
            computed.get_property_value("display").unwrap_throw(),
            "none",
            "visuallyHidden must not hide via display: none"
        );
        assert_ne!(
            computed.get_property_value("visibility").unwrap_throw(),
            "hidden",
            "visuallyHidden must not hide via visibility: hidden"
        );
    }

    // Mirrors `packages/react/src/floating-ui-react/utils/tabbable.test.ts:317-326`: the
    // input variant applied to a checkbox input likewise sets every declaration while
    // avoiding the tab-order-breaking hiding mechanisms.
    #[wasm_bindgen_test]
    fn visually_hidden_input_applies_to_a_checkbox_input_and_avoids_the_tab_order_breakers() {
        let document = document();
        let input = document
            .create_element("input")
            .unwrap_throw()
            .unchecked_into::<HtmlInputElement>();
        input.set_type("checkbox");

        apply(&input.style(), VISUALLY_HIDDEN_INPUT);

        for (property, value) in VISUALLY_HIDDEN_INPUT {
            assert_eq!(
                input.style().get_property_value(property).unwrap_throw(),
                *value,
                "unexpected inline value for {property}"
            );
        }

        let computed = web_sys::window()
            .unwrap_throw()
            .get_computed_style(&input)
            .expect_throw("Base UI: failed to read the computed style")
            .expect_throw("Base UI: the element has no computed style");
        assert_ne!(
            computed.get_property_value("display").unwrap_throw(),
            "none",
            "visuallyHiddenInput must not hide via display: none"
        );
        assert_ne!(
            computed.get_property_value("visibility").unwrap_throw(),
            "hidden",
            "visuallyHiddenInput must not hide via visibility: hidden"
        );
    }
}
