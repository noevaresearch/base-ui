//! Port of the `isMouseWithinBounds` pair (Base UI Phase A util, TODO item
//! `utils: isMouseWithinBounds`).
//!
//! Upstream has two same-named exports, and the spec
//! (`specs/utils/isMouseWithinBounds.md`) covers both under this unit:
//!
//! - The live, tested two-argument variant in
//!   `packages/react/src/utils/getPseudoElementBounds.ts:20-29`: `isMouseWithinBounds(event,
//!   element)` — the only form any consumer uses (the Select, Menu, and Combobox triggers pass
//!   the trigger element explicitly
//!   (`packages/react/src/select/trigger/SelectTrigger.tsx:193`,
//!   `packages/react/src/menu/trigger/MenuTrigger.tsx:149`,
//!   `packages/react/src/combobox/trigger/ComboboxTrigger.tsx:233`)) and the only form proven
//!   by upstream tests (`packages/react/src/utils/getPseudoElementBounds.test.ts:61-78`). It
//!   allows an inclusive ±[`BOUNDARY_OFFSET`] px drift on every edge so a fast click whose
//!   pointer drifts slightly during press-release is not mistaken for a
//!   drag-off-and-release cancellation
//!   (`packages/react/src/utils/getPseudoElementBounds.ts:11-14`).
//! - The deprecated registered single-argument variant in
//!   `packages/utils/src/isMouseWithinBounds.ts:4-19` — the file the unit registry lists
//!   (`ralph/generated/utils.json:141-148`, with `testFiles: []`): reads
//!   `event.currentTarget` and insets the rect by 1px per side, a Safari `mouseleave`
//!   workaround for trigger-aligned items (issue #869,
//!   `packages/utils/src/isMouseWithinBounds.ts:9-12`). Upstream marks it `@deprecated`
//!   ("no longer used internally and will be removed in the next version",
//!   `packages/utils/src/isMouseWithinBounds.ts:1-3`); the port carries the same status on
//!   [`is_mouse_within_bounds_registered`].
//!
//! The two-argument variant delegates geometry to the companion
//! [`get_pseudo_element_bounds`] export
//! (`packages/react/src/utils/getPseudoElementBounds.ts:31-69`), which returns the element's
//! bounding rect expanded symmetrically to include its `::before`/`::after` boxes when either
//! renders content (computed `content` other than `'none'`,
//! `packages/react/src/utils/getPseudoElementBounds.ts:43-47`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The jsdom bypass (`platform.env.jsdom` early return,
//!   `packages/react/src/utils/getPseudoElementBounds.ts:36-38`) is omitted: it exists only to
//!   dodge jsdom's unimplemented `getComputedStyle(elt, pseudoElt)` stub, and the port never
//!   runs in jsdom — its wasm tests execute in a real browser, where the pseudo-element path
//!   is live (the spec marks the jsdom behavior as the environment-gated test
//!   `packages/react/src/utils/getPseudoElementBounds.test.ts:10-21`).
//! - `ownerWindow(element)` (`packages/react/src/utils/getPseudoElementBounds.ts:33`) resolves
//!   the element's own realm's window; the port inlines the same resolution through
//!   `ownerDocument().defaultView()` rather than depending on the not-yet-ported `utils: owner`
//!   module — switch to it once that item lands.
//! - Computed CSS `width`/`height` are strings like `"40px"`, so the upstream
//!   `parseFloat(...) || 0` fallback (`packages/react/src/utils/getPseudoElementBounds.ts:50-53`)
//!   maps a non-numeric prefix (`"auto"`) to 0. The port parses the leading numeric prefix the
//!   way JS `parseFloat` does ([`parse_float_prefix`]) — a strict full-string parse would
//!   wrongly reject every real computed value — and substitutes 0 for `NaN` like `|| 0`.
//! - The registered variant's `event.currentTarget` is `null` outside event dispatch (DOM
//!   semantics), and upstream would throw a `TypeError` reading a rect off it; valid usage
//!   (during dispatch) never reaches that. The port returns `false` for the null-ish
//!   currentTarget instead of panicking.

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{CssStyleDeclaration, Element, HtmlElement, MouseEvent, Window};

/// Inclusive drift tolerance on every edge, in CSS pixels — the upstream `BOUNDARY_OFFSET`
/// (`packages/react/src/utils/getPseudoElementBounds.ts:11-14`): a fast click whose pointer
/// drifts slightly during press-release isn't mistaken for a drag-off-and-release
/// cancellation.
const BOUNDARY_OFFSET: f64 = 5.0;

/// The `{ left, right, top, bottom }` bounds object the upstream `getPseudoElementBounds`
/// export returns (`packages/react/src/utils/getPseudoElementBounds.ts:4-9`, `:63-68`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementBounds {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

/// The upstream live `isMouseWithinBounds(event, element)` export
/// (`packages/react/src/utils/getPseudoElementBounds.ts:20-29`): `true` when the event's
/// `clientX`/`clientY` fall within the element's pseudo-element-expanded bounds, with an
/// inclusive ±[`BOUNDARY_OFFSET`] px tolerance on every edge. The element does not need to be
/// attached to the document for the tolerance semantics to hold — only its rect is read
/// (`packages/react/src/utils/getPseudoElementBounds.test.ts:61-78`, `:82-86`).
pub fn is_mouse_within_bounds(event: &MouseEvent, element: &HtmlElement) -> bool {
    let bounds = get_pseudo_element_bounds(element);
    let x = f64::from(event.client_x());
    let y = f64::from(event.client_y());

    x >= bounds.left - BOUNDARY_OFFSET
        && x <= bounds.right + BOUNDARY_OFFSET
        && y >= bounds.top - BOUNDARY_OFFSET
        && y <= bounds.bottom + BOUNDARY_OFFSET
}

/// The upstream `getPseudoElementBounds(element)` companion export
/// (`packages/react/src/utils/getPseudoElementBounds.ts:31-69`): the element's bounding rect,
/// expanded symmetrically to include its `::before`/`::after` boxes when either renders
/// content. An element with neither pseudo-element rendering content returns its plain rect
/// (`packages/react/src/utils/getPseudoElementBounds.ts:43-47`).
pub fn get_pseudo_element_bounds(element: &HtmlElement) -> ElementBounds {
    let rect = element.get_bounding_client_rect();
    let bounds = ElementBounds {
        left: rect.left(),
        right: rect.right(),
        top: rect.top(),
        bottom: rect.bottom(),
    };

    let Some(window) = element
        .owner_document()
        .and_then(|document| document.default_view())
    else {
        return bounds;
    };

    let before_styles = computed_style(&window, element, "::before");
    let after_styles = computed_style(&window, element, "::after");

    let content = |styles: Option<&CssStyleDeclaration>| {
        styles
            .map(|styles| styles.get_property_value("content").unwrap_throw())
            .unwrap_or_else(|| "none".to_string())
    };
    let has_pseudo_elements =
        content(before_styles.as_ref()) != "none" || content(after_styles.as_ref()) != "none";

    if !has_pseudo_elements {
        return bounds;
    }

    let rect_width = rect.width();
    let rect_height = rect.height();
    let total_width = rect_width
        .max(style_length(before_styles.as_ref(), "width"))
        .max(style_length(after_styles.as_ref(), "width"));
    let total_height = rect_height
        .max(style_length(before_styles.as_ref(), "height"))
        .max(style_length(after_styles.as_ref(), "height"));

    let width_diff = total_width - rect_width;
    let height_diff = total_height - rect_height;

    ElementBounds {
        left: bounds.left - width_diff / 2.0,
        right: bounds.right + width_diff / 2.0,
        top: bounds.top - height_diff / 2.0,
        bottom: bounds.bottom + height_diff / 2.0,
    }
}

/// The upstream deprecated `isMouseWithinBounds(event)` export
/// (`packages/utils/src/isMouseWithinBounds.ts:4-19`): the single-argument variant that reads
/// `event.currentTarget` — the element the dispatched event is being delivered to — and
/// insets its rect by 1px per side, a workaround for Safari misfiring `mouseleave` on
/// trigger-aligned items (issue #869). Meaningful only during dispatch, when `currentTarget`
/// is set; a null-ish currentTarget (dispatch already finished) yields `false` — see the
/// module docs for the throw-vs-`false` adaptation.
#[deprecated(
    note = "no longer used internally upstream and will be removed in the next version (packages/utils/src/isMouseWithinBounds.ts:1-3)"
)]
pub fn is_mouse_within_bounds_registered(event: &MouseEvent) -> bool {
    let Some(target) = event.current_target() else {
        return false;
    };
    let Some(element) = target.dyn_ref::<Element>() else {
        return false;
    };
    let rect = element.get_bounding_client_rect();
    let x = f64::from(event.client_x());
    let y = f64::from(event.client_y());

    rect.top() + 1.0 <= y
        && y <= rect.bottom() - 1.0
        && rect.left() + 1.0 <= x
        && x <= rect.right() - 1.0
}

fn computed_style(
    window: &Window,
    element: &Element,
    pseudo_element: &str,
) -> Option<CssStyleDeclaration> {
    window
        .get_computed_style_with_pseudo_elt(element, pseudo_element)
        .unwrap_throw()
}

/// The CSS-length read behind `parseFloat(styles.width) || 0`
/// (`packages/react/src/utils/getPseudoElementBounds.ts:50-53`): parse the leading numeric
/// prefix of the computed value and fall back to 0 when there is none (`"auto"`, missing
/// style object).
fn style_length(styles: Option<&CssStyleDeclaration>, property: &str) -> f64 {
    let parsed = styles
        .map(|styles| {
            parse_float_prefix(&styles.get_property_value(property).unwrap_throw())
        })
        .unwrap_or(f64::NAN);
    if parsed.is_nan() { 0.0 } else { parsed }
}

/// JS `parseFloat` semantics over a leading numeric prefix: optional sign, `Infinity`,
/// decimal digits with optional fraction, optional exponent — `NaN` when the string starts
/// with no number at all (computed CSS values like `"40px"` parse as `40`).
fn parse_float_prefix(value: &str) -> f64 {
    let trimmed = value.trim_start();
    let bytes = trimmed.as_bytes();
    let mut index = 0;

    let negative = index < bytes.len() && (bytes[index] == b'+' || bytes[index] == b'-');
    if negative {
        index += 1;
    }
    if trimmed[index..].starts_with("Infinity") {
        return if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }

    let mut saw_digit = false;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
        saw_digit = true;
    }
    if index < bytes.len() && bytes[index] == b'.' {
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
            saw_digit = true;
        }
    }
    if !saw_digit {
        return f64::NAN;
    }
    if index < bytes.len() && (bytes[index] == b'e' || bytes[index] == b'E') {
        let mut exponent_end = index + 1;
        if exponent_end < bytes.len() && (bytes[exponent_end] == b'+' || bytes[exponent_end] == b'-')
        {
            exponent_end += 1;
        }
        let exponent_start = exponent_end;
        while exponent_end < bytes.len() && bytes[exponent_end].is_ascii_digit() {
            exponent_end += 1;
        }
        if exponent_end > exponent_start {
            index = exponent_end;
        }
    }

    trimmed[..index].parse::<f64>().unwrap_or(f64::NAN)
}

#[cfg(all(test, target_arch = "wasm32"))]
#[allow(deprecated)]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Document, Element, EventTarget, MouseEventInit};

    use super::*;

    // The behavior under test is real DOM geometry — `getBoundingClientRect` and
    // `getComputedStyle(el, '::before'/'::after')` on laid-out elements — so the tests run in
    // a real browser via the wasm32 test runner (`.cargo/config.toml` wires it to
    // chromedriver), like the crate's other wasm test modules. jsdom cannot execute this
    // module's pseudo-element path at all (it is the reason the upstream jsdom test only
    // asserts the environment bypass).
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> Document {
        web_sys::window()
            .unwrap_throw()
            .document()
            .unwrap_throw()
    }

    // The upstream tests stub `getBoundingClientRect` per element
    // (`packages/react/src/utils/getPseudoElementBounds.test.ts:82-86`); Rust cannot
    // monkey-patch a DOM method, so the fixtures instead use absolutely positioned real
    // elements — `position: absolute` is relative to the initial containing block (no
    // positioned ancestor in the test page), giving the same rect the upstream stubs produce.
    // Each test uses a unique class name so a fixture left behind by a panicking test can
    // never restyle a later test's element, and removes its style tag and element on success.
    fn append_style(document: &Document, css: &str) -> Element {
        let style = document.create_element("style").unwrap_throw();
        style.set_text_content(Some(css));
        document
            .head()
            .unwrap_throw()
            .append_child(&style)
            .unwrap_throw();
        style
    }

    fn append_element(document: &Document, class_name: &str) -> HtmlElement {
        let element = document
            .create_element("div")
            .unwrap_throw()
            .unchecked_into::<HtmlElement>();
        element.set_class_name(class_name);
        document
            .body()
            .unwrap_throw()
            .append_child(&element)
            .unwrap_throw();
        element
    }

    fn mouse_up(x: i32, y: i32) -> MouseEvent {
        MouseEvent::new_with_mouse_event_init_dict(
            "mouseup",
            &MouseEventInit::new().client_x(x).client_y(y),
        )
        .unwrap_throw()
    }

    fn assert_bounds_eq(actual: ElementBounds, expected: (f64, f64, f64, f64)) {
        let (left, right, top, bottom) = expected;
        assert!(
            (actual.left - left).abs() < 1e-6
                && (actual.right - right).abs() < 1e-6
                && (actual.top - top).abs() < 1e-6
                && (actual.bottom - bottom).abs() < 1e-6,
            "expected bounds ({left}, {right}, {top}, {bottom}), got {actual:?}"
        );
    }

    // Mirrors `packages/react/src/utils/getPseudoElementBounds.test.ts:61-78` — the drift
    // test is not environment-gated upstream, and pins all four edges of the inclusive
    // ±5px `BOUNDARY_OFFSET` tolerance (element rect 100,50 → 120,60, matching the
    // upstream fixture).
    #[wasm_bindgen_test]
    fn allows_up_to_5px_of_mouse_drift_around_element_bounds() {
        let document = document();
        let style = append_style(
            &document,
            ".rwib-drift { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }",
        );
        let element = append_element(&document, "rwib-drift");

        let within = |x: i32, y: i32| is_mouse_within_bounds(&mouse_up(x, y), &element);

        // Left edge
        assert!(within(95, 55));
        assert!(!within(94, 55));
        // Right edge
        assert!(within(125, 55));
        assert!(!within(126, 55));
        // Top edge
        assert!(within(110, 45));
        assert!(!within(110, 44));
        // Bottom edge
        assert!(within(110, 65));
        assert!(!within(110, 66));

        element.remove();
        style.remove();
    }

    // Mirrors the rect passthrough expectation of the jsdom-only upstream test
    // (`packages/react/src/utils/getPseudoElementBounds.test.ts:10-21`, {left:100, right:120,
    // top:50, bottom:60} for a 20×10 element at (100,50)) — but in a real browser, where the
    // short-circuit is taken through the computed `content !== 'none'` check
    // (`packages/react/src/utils/getPseudoElementBounds.ts:43-47`), which the upstream jsdom
    // test cannot reach (it proves passthrough via the environment check instead).
    #[wasm_bindgen_test]
    fn returns_the_plain_rect_for_an_element_without_pseudo_elements() {
        let document = document();
        let style = append_style(
            &document,
            ".rwib-plain { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }",
        );
        let element = append_element(&document, "rwib-plain");

        assert_bounds_eq(
            get_pseudo_element_bounds(&element),
            (100.0, 120.0, 50.0, 60.0),
        );

        element.remove();
        style.remove();
    }

    // Mirrors the browser-only upstream test
    // (`packages/react/src/utils/getPseudoElementBounds.test.ts:23-59`): an element 20×10 at
    // (100,50) with a 40×30 `::before` yields {left:90, right:130, top:40, bottom:70}
    // (±10px on both axes). This is the path the spec marks proven only in browsers
    // (`specs/utils/isMouseWithinBounds.md`, "DOM structure & portal behavior").
    #[wasm_bindgen_test]
    fn expands_bounds_to_include_pseudo_elements() {
        let document = document();
        let style = append_style(
            &document,
            "
              .rwib-pseudo { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }
              .rwib-pseudo::before { content: \"\"; display: block; width: 40px; height: 30px; }
            ",
        );
        let element = append_element(&document, "rwib-pseudo");

        assert_bounds_eq(
            get_pseudo_element_bounds(&element),
            (90.0, 130.0, 40.0, 70.0),
        );

        element.remove();
        style.remove();
    }

    // Implementation-derived (`packages/react/src/utils/getPseudoElementBounds.ts:21-28`):
    // the two-argument predicate reads its bounds through `getPseudoElementBounds`, so the
    // ±5px tolerance applies to the expanded bounds — points inside a pseudo-element's
    // expansion (but outside the plain-rect tolerance window) count as within.
    #[wasm_bindgen_test]
    fn applies_the_drift_tolerance_to_the_expanded_pseudo_element_bounds() {
        let document = document();
        let style = append_style(
            &document,
            "
              .rwib-pseudo-tol { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }
              .rwib-pseudo-tol::before { content: \"\"; display: block; width: 40px; height: 30px; }
            ",
        );
        let element = append_element(&document, "rwib-pseudo-tol");

        let within = |x: i32, y: i32| is_mouse_within_bounds(&mouse_up(x, y), &element);

        // Expanded bounds (90,130,40,70) + the 5px tolerance → x in [85,135], y in [35,75]:
        // x=92/y=36 are inside only via the pseudo-element expansion, x=84/y=34 fall outside
        // even the expanded tolerance window.
        assert!(within(92, 55));
        assert!(!within(84, 55));
        assert!(within(110, 36));
        assert!(!within(110, 34));

        element.remove();
        style.remove();
    }

    // Implementation-derived (`packages/react/src/utils/getPseudoElementBounds.ts:50-53`):
    // the `parseFloat(...) || 0` fallback — an inline pseudo-element's computed `width` is
    // `"auto"`, which parses as NaN and contributes 0 to the max, leaving the plain rect.
    // No upstream test asserts the non-numeric fallback (the spec marks it UNVERIFIED);
    // this pins the port's `parse_float_prefix` + `|| 0` semantics.
    #[wasm_bindgen_test]
    fn treats_a_non_numeric_pseudo_element_size_as_zero() {
        let document = document();
        let style = append_style(
            &document,
            "
              .rwib-pseudo-auto { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }
              .rwib-pseudo-auto::before { content: \"\"; display: inline; }
            ",
        );
        let element = append_element(&document, "rwib-pseudo-auto");

        assert_bounds_eq(
            get_pseudo_element_bounds(&element),
            (100.0, 120.0, 50.0, 60.0),
        );

        element.remove();
        style.remove();
    }

    // Mirrors the deprecated registered variant's inset semantics
    // (`packages/utils/src/isMouseWithinBounds.ts:13-17`), all UNVERIFIED upstream (the unit
    // registry lists no test files) and pinned here from the cited implementation. The
    // variant reads `event.currentTarget`, so the event must be dispatched on the element —
    // during dispatch the DOM sets currentTarget, which is the only state the function sees.
    // Element rect 100,50 → 120,60 with a 1px inset per side.
    #[wasm_bindgen_test]
    fn insets_the_current_target_bounds_by_1px() {
        let document = document();
        let style = append_style(
            &document,
            ".rwib-registered { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }",
        );
        let element = append_element(&document, "rwib-registered");

        let observed = Rc::new(Cell::new(false));
        let handler = {
            let observed = Rc::clone(&observed);
            Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
                observed.set(is_mouse_within_bounds_registered(&event));
            })
        };
        let target: &EventTarget = element.as_ref();
        target
            .add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())
            .unwrap_throw();

        let dispatch = |x: i32, y: i32| {
            element.dispatch_event(&mouse_up(x, y)).unwrap_throw();
            observed.get()
        };

        assert!(dispatch(110, 55)); // center
        assert!(dispatch(101, 55)); // left + 1
        assert!(!dispatch(100, 55)); // left edge exactly
        assert!(dispatch(119, 55)); // right - 1
        assert!(!dispatch(120, 55)); // right edge exactly
        assert!(dispatch(110, 51)); // top + 1
        assert!(!dispatch(110, 50)); // top edge exactly
        assert!(dispatch(110, 59)); // bottom - 1
        assert!(!dispatch(110, 60)); // bottom edge exactly

        target
            .remove_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())
            .unwrap_throw();
        drop(handler);
        element.remove();
        style.remove();
    }

    // Implementation-derived: the Rust adaptation for the null-ish `currentTarget` — after
    // `dispatchEvent` returns, the DOM clears `currentTarget`, and the port answers `false`
    // instead of the TypeError upstream would throw
    // (`packages/utils/src/isMouseWithinBounds.ts:7`). Valid dispatch-time usage never sees
    // this state.
    #[wasm_bindgen_test]
    fn answers_false_when_current_target_is_absent() {
        let document = document();
        let style = append_style(
            &document,
            ".rwib-registered-null { position: absolute; left: 100px; top: 50px; width: 20px; height: 10px; }",
        );
        let element = append_element(&document, "rwib-registered-null");

        let event = mouse_up(110, 55);
        element.dispatch_event(&event).unwrap_throw();
        assert!(event.current_target().is_none());
        assert!(!is_mouse_within_bounds_registered(&event));

        element.remove();
        style.remove();
    }
}
