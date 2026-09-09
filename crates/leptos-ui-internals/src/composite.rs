//! Port of `packages/react/src/internals/composite/composite.ts` — the pure vocabulary the
//! composite cluster (root/list/item, still to come as later checkpoints of this unit)
//! builds on: the key constants and the `COMPOSITE_KEYS`/`MODIFIER_KEYS` sets, the
//! native-input detection (`isNativeInput`), the manual scroll math
//! ([`scroll_into_view_if_needed`]), and the re-exports of the floating-ui composite
//! navigation helpers upstream re-exports through this module
//! (`composite.ts:4-11`).
//!
//! Spec: `specs/library/internals/behavior.md` ("DOM structure & portal behavior" —
//! `scrollIntoViewIfNeeded`'s contract, `composite.test.ts:30-36`) and
//! `specs/library/internals/implementation.md` ("DOM/portal strategy and why" — the manual
//! scroll math rationale: avoids scrolling the full ancestor scroll chain, exact RTL
//! semantics, and explicit `scroll-margin-*`/`scroll-padding-*` handling; "Dependencies on
//! other Base UI internals" — the floating-ui re-export list). Every claim was verified
//! against the source before porting.
//!
//! Rust adaptations (behavior-preserving where the contract is defined):
//!
//! - Upstream's `'horizontal' | 'vertical' | 'both'` orientation string ports to the
//!   crate's existing [`crate::floating_ui::types::Orientation`] enum — the same closed
//!   three-arm union the floating-ui grid navigator already uses.
//! - `COMPOSITE_KEYS`/`MODIFIER_KEYS` (`composite.ts:22-26`) are `Set`/array constants
//!   upstream; the port defines them as `&str` arrays plus an [`is_composite_key`]
//!   helper — `.has(key)`/`.includes(key)` call sites become `contains`/`is_composite_key`.
//!   The key-name constants are re-defined here (not re-exported from
//!   `crate::floating_ui::constants`) to mirror upstream's module structure — both modules
//!   own identical `'ArrowUp'`-style strings independently.
//! - `isHTMLElement` (`@floating-ui/utils/dom`) is [`floating_ui_dom::dom::is_html_element`],
//!   the same crate-backed predicate the other ports use (the `tabbable` precedent).
//! - `element.selectionStart != null` (`isNativeInput`, `composite.ts:35`) reads a web-sys
//!   `Result`: an `Err` (a browser that *throws* for selection-unsupported input types, e.g.
//!   `type="number"` in Chrome, where the JS getter throws `InvalidStateError` and would
//!   crash this function the same way) maps to `None` — fail-soft where upstream throws.
//!   Realistic call sites (composite items' event targets) never hit the throwing path.
//! - `!element.scrollTo` guard (`scroll_into_view_if_needed`, `composite.ts:50`): a typed
//!   [`web_sys::HtmlElement`] always has `scrollTo`, so the truthiness guard collapses into
//!   the `None` parameter arms.
//! - `getComputedStyle(element)` (`getStyles`, `composite.ts:162-174`) can return `null` for
//!   a detached document; the port maps that to all-zero styles rather than panicking —
//!   the zero styles then leave the scroll target at the container's current offsets, the
//!   same outcome as every margin/padding parsing to `NaN → 0`.
//! - `parseFloat(styles.scrollMarginTop)` (`:165-172`) ports to [`js_parse_float`] — the JS
//!   prefix-parse semantics (leading numeric prefix, `NaN` on no-parse) with the `|| 0`
//!   fallback applied, so `"20px"` → `20` and `"auto"` → `0`.
//! - JS number arithmetic over `offsetLeft`/`offsetWidth`/`scrollLeft`/… ports to `f64`
//!   throughout; `scrollLeft`/`scrollTop` are doubles in the platform API already, and the
//!   integer geometry reads are widened at their use sites.

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, EventTarget, HtmlElement, HtmlInputElement};

use crate::direction_context::TextDirection;
use crate::floating_ui::types::Orientation;

// ---------------------------------------------------------------------------
// The floating-ui re-exports (`composite.ts:4-11`) — the composite navigation
// helpers the root/list consumers pull through this module one-stop, exactly
// as upstream re-exports them here from `floating-ui-react/utils`.
// ---------------------------------------------------------------------------

pub use crate::floating_ui::composite::{
    find_non_disabled_list_index, get_max_list_index, get_min_list_index,
    is_index_out_of_list_bounds, is_list_index_disabled,
};
pub use crate::floating_ui::event::stop_event;

/// `ARROW_UP` (`packages/react/src/internals/composite/composite.ts:13`).
pub const ARROW_UP: &str = "ArrowUp";

/// `ARROW_DOWN` (`packages/react/src/internals/composite/composite.ts:14`).
pub const ARROW_DOWN: &str = "ArrowDown";

/// `ARROW_LEFT` (`packages/react/src/internals/composite/composite.ts:15`).
pub const ARROW_LEFT: &str = "ArrowLeft";

/// `ARROW_RIGHT` (`packages/react/src/internals/composite/composite.ts:16`).
pub const ARROW_RIGHT: &str = "ArrowRight";

/// `HOME` (`packages/react/src/internals/composite/composite.ts:17`).
pub const HOME: &str = "Home";

/// `END` (`packages/react/src/internals/composite/composite.ts:18`).
pub const END: &str = "End";

/// `PAGE_UP` (`packages/react/src/internals/composite/composite.ts:19`) — exported but
/// deliberately *not* in [`COMPOSITE_KEYS`] and not handled by `useCompositeRoot`
/// (`specs/library/internals/implementation.md`, "Anything in source not explained by any
/// test" item 6).
pub const PAGE_UP: &str = "PageUp";

/// `PAGE_DOWN` (`packages/react/src/internals/composite/composite.ts:20`) — same status as
/// [`PAGE_UP`].
pub const PAGE_DOWN: &str = "PageDown";

/// `COMPOSITE_KEYS` (`packages/react/src/internals/composite/composite.ts:22`) — the key
/// whitelist `useCompositeRoot`'s keydown filter checks with `Set.has`
/// (`specs/library/internals/implementation.md`, "Highlight state machine": "key whitelist
/// (`COMPOSITE_KEYS`, `packages/react/src/internals/composite/composite.ts:22`)"). Home/End
/// are opt-in via `enableHomeAndEndKeys` at the root level, but they are whitelisted here —
/// the set is the filter, the opt-in happens after it.
pub const COMPOSITE_KEYS: [&str; 6] = [ARROW_UP, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, HOME, END];

/// The `COMPOSITE_KEYS.has(key)` check (`packages/react/src/internals/composite/
/// root/useCompositeRoot.ts:206-217` consumes it this way).
pub fn is_composite_key(key: &str) -> bool {
    COMPOSITE_KEYS.contains(&key)
}

/// `SHIFT` (`packages/react/src/internals/composite/composite.ts:24`).
pub const SHIFT: &str = "Shift";

/// `MODIFIER_KEYS` (`packages/react/src/internals/composite/composite.ts:25`) — the four
/// modifiers the root's default modifier gate blocks. `modifierKeys` opts *back in* listed
/// ones (`specs/library/internals/behavior.md`, "Keyboard interactions": the
/// `useCompositeRoot` modifier matrix).
pub const MODIFIER_KEYS: [&str; 4] = [SHIFT, "Control", "Alt", "Meta"];

/// Port of `isNativeInput` (`packages/react/src/internals/composite/composite.ts:32-42`):
/// whether the event target is a native input with caret-selection support (an `<input>`
/// whose `selectionStart` is non-null, or any `<textarea>`) — the root's keydown filter
/// yields arrow keys to such targets so text navigation keeps working
/// (`specs/library/internals/implementation.md`, "Highlight state machine": the
/// native-input yield arm).
pub fn is_native_input(element: &EventTarget) -> bool {
    let Some(element) = element.dyn_ref::<Element>() else {
        // `isHTMLElement(element)` (`composite.ts:28-30`) — non-elements fail both arms.
        return false;
    };

    if element.tag_name() == "INPUT" {
        // `element.selectionStart != null` (`composite.ts:35`). The web-sys read returns a
        // `Result`; an `Err` (the throwing-browser case) maps to `None` — see the module
        // docs.
        let selection_start = element
            .dyn_ref::<HtmlInputElement>()
            .and_then(|input| input.selection_start().ok().flatten());
        if selection_start.is_some() {
            return true;
        }
    }

    if element.tag_name() == "TEXTAREA" {
        // `isHTMLElement(element) && element.tagName === 'TEXTAREA'` (`composite.ts:38-40`).
        return true;
    }

    false
}

/// Port of `scrollIntoViewIfNeeded`
/// (`packages/react/src/internals/composite/composite.ts:44-144`): manual scroll math —
/// compute the scroll offset that brings `element` fully into `scroll_container`'s viewport
/// (respecting the element's `scroll-margin-*` and the container's `scroll-padding-*`), then
/// issue one `scrollTo({behavior: 'auto'})`.
///
/// Branch structure (upstream, `composite.ts:60-137`):
///
/// - Horizontal block: only when the container overflows horizontally and the orientation
///   is not `'vertical'`. In LTR, right-edge overflow is checked first (align right edges),
///   then left-edge overflow (align left edges). In RTL the checks swap — left-edge first,
///   because RTL scroll coordinates are negative-going — with the *same* two alignments.
/// - Vertical block: only when the container overflows vertically and the orientation is not
///   `'horizontal'`; top-edge overflow is checked first, then bottom-edge.
/// - No overflow on an axis (or the orientation excludes it): that axis keeps the
///   container's current scroll offset.
pub fn scroll_into_view_if_needed(
    scroll_container: Option<&HtmlElement>,
    element: Option<&HtmlElement>,
    direction: TextDirection,
    orientation: Orientation,
) {
    let (Some(scroll_container), Some(element)) = (scroll_container, element) else {
        // `!scrollContainer || !element` (`composite.ts:50`) — the `!element.scrollTo`
        // truthiness guard is subsumed by the typed parameter (module docs).
        return;
    };

    let mut target_x = scroll_container.scroll_left() as f64;
    let mut target_y = scroll_container.scroll_top() as f64;

    let is_overflowing_x =
        (scroll_container.client_width() as f64) < scroll_container.scroll_width() as f64;
    let is_overflowing_y =
        (scroll_container.client_height() as f64) < scroll_container.scroll_height() as f64;

    if is_overflowing_x && orientation != Orientation::Vertical {
        let element_offset_left = get_offset(scroll_container, element, OffsetSide::Left);
        let container_styles = get_styles(scroll_container);
        let element_styles = get_styles(element);

        if direction == TextDirection::Ltr {
            if element_offset_left
                + element.offset_width() as f64
                + element_styles.scroll_margin_right
                > scroll_container.scroll_left() as f64 + scroll_container.client_width() as f64
                    - container_styles.scroll_padding_right
            {
                // overflow to the right, scroll to align right edges
                target_x = element_offset_left
                    + element.offset_width() as f64
                    + element_styles.scroll_margin_right
                    - scroll_container.client_width() as f64
                    + container_styles.scroll_padding_right;
            } else if element_offset_left - element_styles.scroll_margin_left
                < scroll_container.scroll_left() as f64 + container_styles.scroll_padding_left
            {
                // overflow to the left, scroll to align left edges
                target_x = element_offset_left
                    - element_styles.scroll_margin_left
                    - container_styles.scroll_padding_left;
            }
        }

        if direction == TextDirection::Rtl {
            if element_offset_left - element_styles.scroll_margin_left
                < scroll_container.scroll_left() as f64 + container_styles.scroll_padding_left
            {
                // overflow to the left, scroll to align left edges
                target_x = element_offset_left
                    - element_styles.scroll_margin_left
                    - container_styles.scroll_padding_left;
            } else if element_offset_left
                + element.offset_width() as f64
                + element_styles.scroll_margin_right
                > scroll_container.scroll_left() as f64 + scroll_container.client_width() as f64
                    - container_styles.scroll_padding_right
            {
                // overflow to the right, scroll to align right edges
                target_x = element_offset_left
                    + element.offset_width() as f64
                    + element_styles.scroll_margin_right
                    - scroll_container.client_width() as f64
                    + container_styles.scroll_padding_right;
            }
        }
    }

    if is_overflowing_y && orientation != Orientation::Horizontal {
        let element_offset_top = get_offset(scroll_container, element, OffsetSide::Top);
        let container_styles = get_styles(scroll_container);
        let element_styles = get_styles(element);

        if element_offset_top - element_styles.scroll_margin_top
            < scroll_container.scroll_top() as f64 + container_styles.scroll_padding_top
        {
            // overflow upwards, align top edges
            target_y = element_offset_top
                - element_styles.scroll_margin_top
                - container_styles.scroll_padding_top;
        } else if element_offset_top
            + element.offset_height() as f64
            + element_styles.scroll_margin_bottom
            > scroll_container.scroll_top() as f64 + scroll_container.client_height() as f64
                - container_styles.scroll_padding_bottom
        {
            // overflow downwards, align bottom edges
            target_y = element_offset_top
                + element.offset_height() as f64
                + element_styles.scroll_margin_bottom
                - scroll_container.client_height() as f64
                + container_styles.scroll_padding_bottom;
        }
    }

    let scroll_options = web_sys::ScrollToOptions::new();
    scroll_options.set_left(target_x);
    scroll_options.set_top(target_y);
    scroll_options.set_behavior(web_sys::ScrollBehavior::Auto);
    scroll_container.scroll_to_with_scroll_to_options(&scroll_options);
}

/// The `side` argument of upstream's `getOffset` (`composite.ts:146-160`).
enum OffsetSide {
    Left,
    Top,
}

/// Port of `getOffset` (`packages/react/src/internals/composite/composite.ts:146-160`):
/// sum `element`'s `offsetLeft`/`offsetTop` up the `offsetParent` chain, stopping once the
/// chain reaches `ancestor` (which contributes nothing itself). An element with no
/// `offsetParent` (detached, or the root) contributes its own offset and the walk ends.
fn get_offset(ancestor: &HtmlElement, element: &HtmlElement, side: OffsetSide) -> f64 {
    let mut result = 0.0;

    let mut current = element.clone();
    while let Some(parent) = current.offset_parent() {
        result += match side {
            OffsetSide::Left => current.offset_left() as f64,
            OffsetSide::Top => current.offset_top() as f64,
        };
        // `element.offsetParent === ancestor` (`composite.ts:153`) — JsValue identity.
        if parent == *ancestor.as_ref() {
            break;
        }
        // `element = element.offsetParent as HTMLElement` (`composite.ts:156`) — the same
        // unchecked cast; offset parents are HTMLElements in every layout that reaches here.
        current = parent.unchecked_into::<HtmlElement>();
    }

    result
}

/// The parsed `scroll-margin-*`/`scroll-padding-*` styles upstream's `getStyles`
/// (`packages/react/src/internals/composite/composite.ts:162-174`) reads.
#[derive(Clone, Copy, Default)]
struct ScrollStyles {
    scroll_margin_top: f64,
    scroll_margin_right: f64,
    scroll_margin_bottom: f64,
    scroll_margin_left: f64,
    scroll_padding_top: f64,
    scroll_padding_right: f64,
    scroll_padding_bottom: f64,
    scroll_padding_left: f64,
}

/// Port of `getStyles` (`packages/react/src/internals/composite/composite.ts:162-174`):
/// computed styles parsed to numbers, `NaN → 0` via the `|| 0` fallbacks. A missing
/// computed-style object (detached document) reads as all zeros — module docs.
///
/// Upstream reads the camelCase IDL accessors (`styles.scrollMarginTop`, `:165-172`); the
/// port reads the same declarations through `getPropertyValue`, which the CSSOM defines
/// (and Chrome enforces) as kebab-case-only — a camelCase lookup returns `""`, which would
/// silently parse to 0.
fn get_styles(element: &HtmlElement) -> ScrollStyles {
    let styles = web_sys::window()
        .and_then(|window| window.get_computed_style(element).ok())
        .flatten();

    let read = |property: &str| -> f64 {
        styles
            .as_ref()
            .and_then(|styles| styles.get_property_value(property).ok())
            .map(|value| {
                let parsed = js_parse_float(&value);
                // `parseFloat(...) || 0` (`composite.ts:165-172`).
                if parsed.is_nan() { 0.0 } else { parsed }
            })
            .unwrap_or(0.0)
    };

    ScrollStyles {
        scroll_margin_top: read("scroll-margin-top"),
        scroll_margin_right: read("scroll-margin-right"),
        scroll_margin_bottom: read("scroll-margin-bottom"),
        scroll_margin_left: read("scroll-margin-left"),
        scroll_padding_top: read("scroll-padding-top"),
        scroll_padding_right: read("scroll-padding-right"),
        scroll_padding_bottom: read("scroll-padding-bottom"),
        scroll_padding_left: read("scroll-padding-left"),
    }
}

/// JS `parseFloat` (`composite.ts:165-172` — the parse behind every `getStyles` field):
/// leading whitespace is skipped, the longest numeric prefix parses, and a non-parse is
/// `NaN` (which the caller's `|| 0` folds to zero). No `Infinity` support: computed style
/// lengths never produce it, and a bare `Infinity` string has no realistic path here.
fn js_parse_float(value: &str) -> f64 {
    let rest = value.trim_start();

    let mut chars = rest.chars().peekable();
    let mut end = 0usize;
    let mut seen_digit = false;

    if matches!(chars.peek(), Some('+') | Some('-')) {
        chars.next();
        end += 1;
    }

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            seen_digit = true;
            chars.next();
            end += 1;
        } else {
            break;
        }
    }

    if chars.peek() == Some(&'.') {
        chars.next();
        end += 1;
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                seen_digit = true;
                chars.next();
                end += 1;
            } else {
                break;
            }
        }
    }

    if !seen_digit {
        return f64::NAN;
    }

    if matches!(chars.peek(), Some('e') | Some('E')) {
        let mut lookahead = chars.clone();
        lookahead.next();
        let mut exponent_end = end + 1;
        if matches!(lookahead.peek(), Some('+') | Some('-')) {
            lookahead.next();
            exponent_end += 1;
        }
        let mut exponent_digits = 0;
        while matches!(lookahead.peek(), Some(&c) if c.is_ascii_digit()) {
            lookahead.next();
            exponent_end += 1;
            exponent_digits += 1;
        }
        if exponent_digits > 0 {
            // Only `end` is consumed below — the suffix slice is taken directly from `rest`.
            end = exponent_end;
        }
    }

    rest[..end].parse::<f64>().unwrap_or(f64::NAN)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins the `COMPOSITE_KEYS` membership (`composite.ts:22`) — the six whitelisted keys,
    // with the PageUp/PageDown exports deliberately outside the set
    // (`specs/library/internals/implementation.md`, "Anything in source not explained by any
    // test" item 6) and the four `MODIFIER_KEYS` entries (`composite.ts:25`).
    #[test]
    fn the_key_sets_hold_the_upstream_memberships() {
        for key in [ARROW_UP, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, HOME, END] {
            assert!(is_composite_key(key), "{key} is whitelisted");
        }
        for key in [PAGE_UP, PAGE_DOWN, "Enter", " ", "Shift"] {
            assert!(!is_composite_key(key), "{key} is not whitelisted");
        }
        assert_eq!(MODIFIER_KEYS, [SHIFT, "Control", "Alt", "Meta"]);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn div() -> HtmlElement {
        document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    }

    /// The upstream test's `scrollTo: { value: scrollTo }` spy
    /// (`composite.test.ts:21`): the latest `ScrollToOptions` the container was handed,
    /// captured through a closure-backed function installed as the instance's own
    /// `scrollTo` — a shadow that method calls (dynamic property lookups) respect, unlike
    /// the IDL attribute getters.
    struct ScrollToSpy {
        closure: Closure<dyn Fn(JsValue)>,
        captured: Rc<RefCell<Option<js_sys::Object>>>,
    }

    impl ScrollToSpy {
        fn install(target: &HtmlElement) -> Self {
            let captured: Rc<RefCell<Option<js_sys::Object>>> = Rc::new(RefCell::new(None));
            let sink = captured.clone();
            let closure = Closure::wrap(Box::new(move |options: JsValue| {
                *sink.borrow_mut() = Some(options.unchecked_into::<js_sys::Object>());
            }) as Box<dyn Fn(JsValue)>);
            js_sys::Reflect::set(
                target.as_ref(),
                &"scrollTo".into(),
                closure.as_ref().unchecked_ref::<js_sys::Function>(),
            )
            .unwrap();
            Self { closure, captured }
        }

        fn options(&self) -> js_sys::Object {
            self.captured
                .borrow()
                .clone()
                .expect("scrollTo was not called")
        }

        fn get_f64(&self, property: &str) -> f64 {
            js_sys::Reflect::get(&self.options(), &property.into())
                .unwrap()
                .as_f64()
                .unwrap_or(f64::NAN)
        }
    }

    /// The real-layout fixture: a 100×100 scroll container (`overflow: auto`,
    /// `scrollbar-width: none` so the classic-scrollbar setting cannot shift
    /// `clientWidth`), an in-flow content box of the given scroll extents, and the item
    /// element absolutely positioned inside it — so, exactly like the upstream test's
    /// `offsetParent: { value: scrollContainer }` fake (`composite.test.ts:26`), the
    /// element's `offsetParent` is the scroll container itself and `offsetLeft`/`offsetTop`
    /// read back as the placed geometry. (Upstream fakes the geometry with
    /// `Object.defineProperties` (`composite.test.ts:16-28`) because jsdom does no layout; a
    /// real browser does not need the fakes. The content box must stay non-positioned: as
    /// the element's offsetParent it would join the `getOffset` walk, and in RTL a
    /// right-aligned in-flow child genuinely reports a negative `offsetLeft` — correct
    /// geometry, but not the relationship upstream's scenario describes.)
    struct Fixture {
        container: HtmlElement,
        element: HtmlElement,
        /// Keeps the container in the document for the test's lifetime.
        _in_document: web_sys::Node,
    }

    impl Fixture {
        fn new(rtl: bool, content_width: u32, content_height: u32) -> Self {
            let container = div();
            let style = container.style();
            style.set_property("width", "100px").unwrap();
            style.set_property("height", "100px").unwrap();
            style.set_property("overflow", "auto").unwrap();
            style.set_property("scrollbar-width", "none").unwrap();
            style.set_property("position", "relative").unwrap();
            if rtl {
                style.set_property("direction", "rtl").unwrap();
            }

            let content = div();
            let content_style = content.style();
            content_style
                .set_property("width", &format!("{content_width}px"))
                .unwrap();
            content_style
                .set_property("height", &format!("{content_height}px"))
                .unwrap();

            let element = div();
            element
                .style()
                .set_property("position", "absolute")
                .unwrap();
            content.append_child(&element).unwrap();
            container.append_child(&content).unwrap();
            let in_document = document().body().unwrap().append_child(&container).unwrap();

            Self {
                container,
                element,
                _in_document: in_document,
            }
        }

        /// Places the element inside the content box (`left`/`top` + `width`/`height`), so
        /// its `offsetLeft`/`offsetTop`/`offsetWidth`/`offsetHeight` read back as the given
        /// geometry.
        fn place(&self, left: f64, top: f64, width: f64, height: f64) {
            let style = self.element.style();
            style.set_property("left", &format!("{left}px")).unwrap();
            style.set_property("top", &format!("{top}px")).unwrap();
            style.set_property("width", &format!("{width}px")).unwrap();
            style
                .set_property("height", &format!("{height}px"))
                .unwrap();
        }

        /// The starting scroll position — the analog of the upstream test's faked
        /// `scrollLeft`/`scrollTop` (`composite.test.ts:19-20`).
        fn scroll_to_start(&self, left: f64, top: f64) {
            self.container.set_scroll_left(left as i32);
            self.container.set_scroll_top(top as i32);
        }
    }

    fn read_style(element: &HtmlElement, property: &str, value: &str) {
        element.style().set_property(property, value).unwrap();
    }

    // Mirrors `packages/react/src/internals/composite/composite.test.ts:6-39` —
    // "uses the left scroll margin when checking left overflow in RTL": the element sits
    // left of the viewport start (offset 10 minus the 20px scroll-margin-left is negative
    // against scrollLeft 0), so the RTL left-edge branch fires and the margin pushes the
    // target to -10.
    #[wasm_bindgen_test]
    fn uses_the_left_scroll_margin_when_checking_left_overflow_in_rtl() {
        let fixture = Fixture::new(true, 200, 100);
        fixture.place(10.0, 0.0, 10.0, 10.0);
        read_style(&fixture.element, "scroll-margin-left", "20px");
        fixture.scroll_to_start(0.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Rtl,
            Orientation::Horizontal,
        );

        assert_eq!(spy.get_f64("left"), -10.0);
        assert_eq!(spy.get_f64("top"), 0.0);
        assert_eq!(
            js_sys::Reflect::get(&spy.options(), &"behavior".into()).unwrap(),
            "auto",
            "the one-shot scroll is unanimated"
        );
    }

    // Port-owned pin (the LTR right-edge arm, `composite.ts:66-87`): an element whose
    // right edge (plus margin) passes the container's right viewport edge minus the scroll
    // padding scrolls to align the right edges — margins/paddings on both sides of the
    // inequality participate.
    #[wasm_bindgen_test]
    fn ltr_right_edge_overflow_aligns_the_right_edges() {
        let fixture = Fixture::new(false, 200, 100);
        read_style(&fixture.container, "scroll-padding-right", "8px");
        fixture.place(90.0, 0.0, 10.0, 10.0);
        read_style(&fixture.element, "scroll-margin-right", "5px");
        fixture.scroll_to_start(0.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Horizontal,
        );

        // 90 + 10 + 5 - 100 + 8 = 13.
        assert_eq!(spy.get_f64("left"), 13.0);
        assert_eq!(spy.get_f64("top"), 0.0);
    }

    // Port-owned pin (the LTR left-edge arm, `composite.ts:79-86`): an element left of the
    // current scroll position scrolls back to align the left edges, the scroll-margin-left
    // widening the gap and the container's scroll-padding-left pulling the viewport edge
    // in.
    #[wasm_bindgen_test]
    fn ltr_left_edge_overflow_aligns_the_left_edges() {
        let fixture = Fixture::new(false, 200, 100);
        read_style(&fixture.container, "scroll-padding-left", "4px");
        fixture.place(10.0, 0.0, 10.0, 10.0);
        read_style(&fixture.element, "scroll-margin-left", "6px");
        fixture.scroll_to_start(50.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Horizontal,
        );

        // Right-edge check: 10 + 10 + 0 = 20 > 50 + 100 - 0 = 150 is false. Left-edge
        // check: 10 - 6 = 4 < 50 + 4 = 54 → 4 - 4 = 0.
        assert_eq!(spy.get_f64("left"), 0.0);
    }

    // Port-owned pin (the RTL precedence, `composite.ts:89-111`): when an element
    // overflows BOTH edges in RTL, the left-edge branch wins — the else-if ordering is the
    // only difference from LTR, and it decides which alignment a doubly-out-of-view item
    // gets. The element spans offset -60..90 against a viewport at scrollLeft -50 (width
    // 100): the left check (-60 < -50) and the right check (90 > 50) both hold, and RTL
    // answers with the left-edge alignment where LTR's ordering would answer -10.
    #[wasm_bindgen_test]
    fn rtl_checks_the_left_edge_before_the_right_edge() {
        let fixture = Fixture::new(true, 200, 100);
        fixture.place(-60.0, 0.0, 150.0, 10.0);
        fixture.scroll_to_start(-50.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Rtl,
            Orientation::Horizontal,
        );

        assert_eq!(spy.get_f64("left"), -60.0);
    }

    // Port-owned pin (the vertical block, `composite.ts:114-137`): top-edge overflow
    // aligns the top edges with the element's scroll-margin-top and the container's
    // scroll-padding-top participating.
    #[wasm_bindgen_test]
    fn vertical_overflow_aligns_the_top_edges_on_upward_overflow() {
        let fixture = Fixture::new(false, 100, 200);
        read_style(&fixture.container, "scroll-padding-top", "3px");
        fixture.place(0.0, 10.0, 10.0, 10.0);
        read_style(&fixture.element, "scroll-margin-top", "2px");
        fixture.scroll_to_start(0.0, 40.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Vertical,
        );

        // Horizontal block excluded by the orientation. Top check: 10 - 2 = 8 < 40 + 3 =
        // 43 → 8 - 3 = 5.
        assert_eq!(
            spy.get_f64("left"),
            0.0,
            "the excluded axis keeps its offset"
        );
        assert_eq!(spy.get_f64("top"), 5.0);
    }

    // Port-owned pin (the bottom-edge arm, `composite.ts:125-136`): an element below the
    // viewport scrolls down to align the bottom edges.
    #[wasm_bindgen_test]
    fn vertical_overflow_aligns_the_bottom_edges_on_downward_overflow() {
        let fixture = Fixture::new(false, 100, 200);
        fixture.place(0.0, 120.0, 10.0, 10.0);
        read_style(&fixture.element, "scroll-margin-bottom", "7px");
        fixture.scroll_to_start(0.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Vertical,
        );

        // Top check: 120 - 0 = 120 < 0 + 0 = 0 is false. Bottom check: 120 + 10 + 7 =
        // 137 > 0 + 100 - 0 = 100 → 137 - 100 + 0 = 37.
        assert_eq!(spy.get_f64("top"), 37.0);
    }

    // Port-owned pin (the no-overflow arms, `composite.ts:57-58` + the guarded blocks):
    // without overflow on an axis the target keeps the container's current scroll offset —
    // the scroll call still happens (the container is scrolled to where it already is).
    #[wasm_bindgen_test]
    fn no_overflow_keeps_the_current_scroll_offsets() {
        let fixture = Fixture::new(false, 100, 100);
        fixture.place(0.0, 0.0, 10.0, 10.0);
        fixture.scroll_to_start(0.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Both,
        );

        assert_eq!(spy.get_f64("left"), 0.0);
        assert_eq!(spy.get_f64("top"), 0.0);
    }

    // Port-owned pin (the orientation gates, `composite.ts:60` and `:114`): a `Vertical`
    // orientation skips the horizontal block even when the container overflows
    // horizontally (and vice versa), so the excluded axis keeps its current offset while
    // the included axis scrolls.
    #[wasm_bindgen_test]
    fn the_orientation_excludes_its_axis_from_the_math() {
        let fixture = Fixture::new(false, 200, 200);
        fixture.place(90.0, 120.0, 10.0, 10.0);
        fixture.scroll_to_start(0.0, 0.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            Some(&fixture.container),
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Vertical,
        );

        // Horizontal excluded → left stays at the current scrollLeft (0); vertical fires →
        // 120 + 10 + 0 - 100 + 0 = 30.
        assert_eq!(spy.get_f64("left"), 0.0);
        assert_eq!(spy.get_f64("top"), 30.0);
    }

    // Port-owned pin (the null guards, `composite.ts:50-52`): a null container or a null
    // element is a silent no-op — no panic, no scroll.
    #[wasm_bindgen_test]
    fn a_null_container_or_element_is_a_no_op() {
        let fixture = Fixture::new(false, 200, 100);
        fixture.place(10.0, 0.0, 10.0, 10.0);
        let spy = ScrollToSpy::install(&fixture.container);

        scroll_into_view_if_needed(
            None,
            Some(&fixture.element),
            TextDirection::Ltr,
            Orientation::Both,
        );
        scroll_into_view_if_needed(
            Some(&fixture.container),
            None,
            TextDirection::Ltr,
            Orientation::Both,
        );
        scroll_into_view_if_needed(None, None, TextDirection::Ltr, Orientation::Both);

        assert!(
            spy.captured.borrow().is_none(),
            "no scroll call escaped the guards"
        );
    }

    // Pins `isNativeInput` (`composite.ts:32-42`) through real DOM: a text input (its
    // `selectionStart` is a number) and a textarea are native inputs; a div is not.
    #[wasm_bindgen_test]
    fn is_native_input_accepts_inputs_and_textareas_only() {
        let as_target = |element: web_sys::Element| -> EventTarget { element.into() };

        let input: web_sys::HtmlInputElement = document()
            .create_element("input")
            .unwrap()
            .dyn_into()
            .unwrap();
        input.set_type("text");
        assert!(is_native_input(&as_target(input.into())));

        let textarea: web_sys::HtmlTextAreaElement = document()
            .create_element("textarea")
            .unwrap()
            .dyn_into()
            .unwrap();
        assert!(is_native_input(&as_target(textarea.into())));

        let div_element = div();
        assert!(!is_native_input(&as_target(div_element.into())));
    }
}
