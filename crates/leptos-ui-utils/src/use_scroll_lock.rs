//! Port of `packages/utils/src/useScrollLock.ts` (Base UI Phase A util).
//!
//! Upstream is a 319-line module with four layers
//! (`packages/utils/src/useScrollLock.ts:1-319`):
//!
//! 1. Viewport-scroller resolution: the page's scroll overflow comes from `<html>` when it
//!    establishes its own scroll container, and propagates from `<body>` otherwise — an
//!    `overflow` style on the other element does not lock the page
//!    (`packages/utils/src/useScrollLock.ts:14-18`), decided via the external
//!    `@floating-ui/utils/dom` `isOverflowElement` helper (`:2`).
//! 2. Two lock strategies chosen by scrollbar kind: overlay scrollbars (and iOS) hide the
//!    viewport scroller's `overflow` (`:64-87`); inset scrollbars reposition `<body>` to
//!    compensate for the scrollbar width, transfer the scroll position, and tag `<html>` with
//!    `data-base-ui-scroll-locked` (`:89-226`), re-locking on window resize (`:208-214`) and
//!    bailing out entirely under WebKit pinch-zoom (`:100-103`). When
//!    `scrollbar-gutter: stable` already absorbs the scrollbar, only the gutter is touched
//!    (`:33-62,158-163`).
//! 3. A module-singleton `ScrollLocker` reference-counting the lock across every consumer on
//!    the page (`:228-304`): acquiring is deferred onto a 0ms `Timeout` (`:234-240`), the lock
//!    is applied only while a consumer is enabled, releasing is deferred the same way
//!    (`:242-247`), and when the page is *already* locked by someone else the locker attaches
//!    an attributes-only `MutationObserver` on `<html>`/`<body>` and takes over the moment the
//!    external lock clears instead of snapshotting the locked state (`:269-288`).
//! 4. The `useScrollLock(enabled?, referenceElement?)` hook wiring acquire/release into the
//!    effect lifecycle of `enabled` (`:312-319`).
//!
//! Source of truth: **none.** The upstream repo has no test file for this unit
//! (`specs/utils/useScrollLock.md`, "Corpus note"; `ralph/generated/utils.json` lists
//! `testFiles: []`), so behavior was mined from consumer suites — modal Dialog locks the page
//! (`packages/react/src/dialog/root/DialogRoot.test.tsx`), anchored popups route through
//! `useAnchoredPopupScrollLock`
//! (`packages/react/src/utils/useAnchoredPopupScrollLock.ts:42`) — and claims about unit
//! internals that no consumer test isolates are marked UNVERIFIED in the spec. The tests below
//! pin the ported contract itself, mirroring the consumer suites' observable signals: inline
//! `overflow: hidden` on `<html>`/`<body>` and the `data-base-ui-scroll-locked` attribute
//! (`specs/utils/useScrollLock.md`, "Accessibility").
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - `isOverflowElement` is ported inline from its real source
//!   (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/
//!   floating-ui.utils.dom.mjs:45-53`): the `overflow + overflowY + overflowX` concatenation
//!   is probed for the five tokens (`auto|scroll|overlay|hidden|clip`) and `display` must be
//!   neither `inline` nor `contents`. The computed style resolves through the element's own
//!   realm via [`owner_window`] (the crate's shadow-DOM/realm-safety convention) where
//!   upstream uses the bare global `getComputedStyle` — identical for same-realm elements and
//!   strictly safer across realms.
//! - The module-level `originalHtmlStyles`/`originalBodyStyles`/`originalHtmlScrollBehavior`
//!   slots (`packages/utils/src/useScrollLock.ts:10-12`) become [`thread_local!`] snapshots.
//!   Upstream shares them across every consumer exactly like the singleton that writes them;
//!   the `ScrollLocker`'s guards (`:257`) ensure only one live lock's snapshot exists at a
//!   time, so the shared-slot semantics carry over. Restoring uses `setProperty(name, "")`,
//!   which CSSOM defines to remove the declaration — the same thing upstream's
//!   `Object.assign(style, original)` does when the snapshotted inline value was unset.
//! - `ScrollLocker` keeps the upstream field-for-field shape — `lockCount`, `restore`, and the
//!   two 0ms [`Timeout`]s from the `useTimeout` port (`packages/utils/src/useScrollLock.ts:7`
//!   imports it; the `useTimeout` TODO entry records this exact composition as why that port
//!   was picked first). `acquire` returns the release cleanup as a [`FnOnce`] box, the
//!   effect-cleanup contract of `:317` — upstream returned the same bound method, so a caller
//!   invoking it twice was possible in JS and corrupts the count; the port's [`FnOnce`] makes
//!   the one-call-per-acquire contract static, matching how the React effect lifecycle (the
//!   only upstream caller) uses it.
//! - The `restore` slot is [`Restore`]: either an applied lock's cleanup or the external-lock
//!   wait state. Upstream stores `() => observer.disconnect()` there and resets it to `null`
//!   inside the observer callback (`:274-275`); the port must keep the observer *and* its
//!   callback closure alive while waiting, and a `wasm_bindgen::Closure` cannot be dropped
//!   while it is executing — so ending a wait parks the callback in a graveyard
//!   ([`RETIRED_CALLBACKS`], the `use_timeout` port's `RETIRED_JOBS` precedent) and the next
//!   [`ScrollLocker::acquire`]/[`ScrollLocker::release`] entry point sweeps it. The graveyard
//!   is deliberately *not* swept from [`ScrollLocker::lock`]: the takeover path calls `lock`
//!   from inside the retiring callback, so a sweep there could drop a still-executing closure.
//! - `preventScrollInsetScrollbars`'s closure-scoped locals (`scrollTop`, `scrollLeft`,
//!   `updateGutterOnly`, the resize `AnimationFrame` — `:95-98`) become an internal struct
//!   shared by its `lockScroll`/`cleanup`/`handleResize` functions, with `html`/`body`/`win`
//!   resolved once at creation exactly like upstream's captured bindings (`:90-93`).
//! - `parseFloat` margins (`:147-148`) become a px-suffix parse defaulting to `0`: computed
//!   browser values are always `Npx`, and the only other parseFloat outcome (NaN) is falsy in
//!   the truthiness checks upstream feeds it into, just like `0` — the two differ solely in
//!   the never-observed unparseable-margin-plus-scrollbar combination.
//! - The resize-listener guard (`typeof win.removeEventListener === 'function'`,
//!   `:222-224`) exists upstream for JS test teardown races; the browser wasm target always
//!   has the function and the host target cannot run this path at all, so the port always
//!   unsubscribes (the [`crate::add_event_listener`] handle does it on unsubscribe *and*
//!   drop, idempotently).
//! - The hook's dependency array `[enabled, referenceElement]`
//!   (`packages/utils/src/useScrollLock.ts:318`) becomes two reactive sources ([`Get`]): both
//!   are read at the top of the effect body, so a change to either re-runs the effect —
//!   release, then (if still enabled) re-acquire — matching the deps-array contract. This is
//!   load-bearing for `enabled`: the dialog consumer flips it with the open state
//!   (`packages/react/src/dialog/root/useDialogRoot.ts:86`), and a Leptos component body runs
//!   once, so a plain value would never release the lock. The default-parameter hazard
//!   upstream guards against in its regression test — `undefined` falling through to the
//!   `enabled = true` default (`specs/utils/useScrollLock.md`, "Public API surface") — is
//!   unrepresentable here: the port requires an explicit value. `'use client'`
//!   (`packages/utils/src/useScrollLock.ts:1`) is N/A — there is no React Server Components
//!   boundary in Rust.
//!
//! Must be called inside a reactive owner (a component), like the other hook ports.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use reactive_graph::traits::Get;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{
    CssStyleDeclaration, Document, Element, HtmlElement, MutationObserver, MutationObserverInit,
    Node, Window,
};

use crate::add_event_listener::add_event_listener;
use crate::owner::{owner_document, owner_window};
use crate::platform::platform;
use crate::use_animation_frame::AnimationFrame;
use crate::use_iso_layout_effect::use_iso_layout_effect;
use crate::use_timeout::Timeout;

/// The lock's page-visible tag
/// (`packages/utils/src/useScrollLock.ts:192`): consumer tests detect it on `<html>`
/// (`specs/utils/useScrollLock.md`, "Accessibility").
const SCROLL_LOCKED_ATTRIBUTE: &str = "data-base-ui-scroll-locked";

/// The inline `<html>` style slots saved by the inset lock
/// (`packages/utils/src/useScrollLock.ts:117-121`).
#[derive(Default)]
struct HtmlStyleSnapshot {
    scrollbar_gutter: String,
    overflow_y: String,
    overflow_x: String,
}

/// The inline `<body>` style slots saved by the inset lock
/// (`packages/utils/src/useScrollLock.ts:124-132`).
#[derive(Default)]
struct BodyStyleSnapshot {
    position: String,
    height: String,
    width: String,
    box_sizing: String,
    overflow_y: String,
    overflow_x: String,
    scroll_behavior: String,
}

thread_local! {
    /// Upstream `originalHtmlStyles` (`packages/utils/src/useScrollLock.ts:10`).
    static ORIGINAL_HTML_STYLES: RefCell<HtmlStyleSnapshot> = RefCell::new(HtmlStyleSnapshot {
        scrollbar_gutter: String::new(),
        overflow_y: String::new(),
        overflow_x: String::new(),
    });

    /// Upstream `originalBodyStyles` (`packages/utils/src/useScrollLock.ts:11`).
    static ORIGINAL_BODY_STYLES: RefCell<BodyStyleSnapshot> = RefCell::new(BodyStyleSnapshot {
        position: String::new(),
        height: String::new(),
        width: String::new(),
        box_sizing: String::new(),
        overflow_y: String::new(),
        overflow_x: String::new(),
        scroll_behavior: String::new(),
    });

    /// Upstream `originalHtmlScrollBehavior` (`packages/utils/src/useScrollLock.ts:12`).
    static ORIGINAL_HTML_SCROLL_BEHAVIOR: RefCell<String> = RefCell::new(String::new());

    /// Observer callback closures whose wait state has ended. A `wasm_bindgen::Closure` must
    /// not be dropped while it is executing on the JS stack, and the takeover path ends the
    /// wait from inside its own callback — so the callback is parked here and dropped at the
    /// next [`ScrollLocker::acquire`]/[`ScrollLocker::release`] entry point, which provably
    /// runs outside any callback invocation (the `use_timeout` port's `RETIRED_JOBS`
    /// precedent).
    static RETIRED_CALLBACKS: RefCell<Vec<Closure<dyn FnMut()>>> = const { RefCell::new(Vec::new()) };

    /// Upstream `SCROLL_LOCKER` (`packages/utils/src/useScrollLock.ts:304`): one lock
    /// bookkeeping singleton shared by every consumer on the page
    /// (`specs/utils/useScrollLock.md`, "State model").
    static SCROLL_LOCKER: Rc<ScrollLocker> = Rc::new(ScrollLocker::new());
}

/// Drops parked observer callback closures that are provably no longer executing — safe at
/// the [`ScrollLocker::acquire`]/[`ScrollLocker::release`] entry points, which can never run
/// inside a callback invocation (the takeover path never sweeps; see [`RETIRED_CALLBACKS`]).
fn sweep_retired_callbacks() {
    RETIRED_CALLBACKS.with(|retired| retired.borrow_mut().clear());
}

/// Resolves the computed style through the element's own realm — upstream's bare global
/// `getComputedStyle` (`packages/utils/src/useScrollLock.ts:21,108-109`), made realm-safe per
/// the crate's `ownerWindow` convention.
fn computed_style<E: AsRef<Element> + ?Sized>(element: &E) -> CssStyleDeclaration {
    let element: &Element = element.as_ref();
    let node: &Node = element.as_ref();
    computed_style_with(&owner_window(Some(node)), element)
}

/// The computed style resolved by an explicit window — upstream's
/// `win.getComputedStyle(...)` form (`packages/utils/src/useScrollLock.ts:21`).
fn computed_style_with(win: &Window, element: &Element) -> CssStyleDeclaration {
    win.get_computed_style(element)
        .expect_throw("Base UI: failed to read the computed style while scroll-locking")
        .expect_throw("Base UI: the element has no computed style while scroll-locking")
}

/// `ownerDocument(referenceElement)` (`packages/utils/src/useScrollLock.ts:28,41,65,261`).
fn owner_document_for(reference_element: Option<&Element>) -> Document {
    owner_document(reference_element.map(|element| {
        let node: &Node = element.as_ref();
        node
    }))
}

/// `ownerWindow(...)` over an element (`packages/utils/src/useScrollLock.ts:93,264`): resolves
/// the element's own `ownerDocument.defaultView`, falling back to the global window.
fn owner_window_for<E: AsRef<Element> + ?Sized>(element: Option<&E>) -> Window {
    owner_window(element.map(|value| {
        let element: &Element = value.as_ref();
        let node: &Node = element.as_ref();
        node
    }))
}

fn document_element_of(document: &Document) -> HtmlElement {
    document
        .document_element()
        .expect_throw("Base UI: the document has no documentElement to scroll-lock")
        .dyn_into()
        .expect_throw("Base UI: the documentElement is not an HTML element")
}

fn body_of(document: &Document) -> HtmlElement {
    document
        .body()
        .expect_throw("Base UI: the document has no body to scroll-lock")
}

/// Reads one inline/computed style declaration.
fn get_style_property(style: &CssStyleDeclaration, property: &str) -> String {
    style.get_property_value(property).expect_throw(&format!(
        "Base UI: failed to read the {property} style while scroll-locking"
    ))
}

/// Sets one inline style declaration. An empty `value` removes the declaration — the
/// CSSOM-defined behavior of `setProperty(name, "")`, which is what upstream's
/// `Object.assign(style, original)` restore does when the snapshotted inline value was unset
/// (`packages/utils/src/useScrollLock.ts:85,197-198`).
fn set_style_property(style: &CssStyleDeclaration, property: &str, value: &str) {
    style.set_property(property, value).expect_throw(&format!(
        "Base UI: failed to set the {property} style while scroll-locking"
    ));
}

/// `parseFloat` of a computed length (`packages/utils/src/useScrollLock.ts:147-148`): the px
/// suffix stripped, `0` when unparseable — see the module docs for the NaN adaptation.
fn parse_px(computed_value: &str) -> f64 {
    computed_value
        .trim()
        .strip_suffix("px")
        .and_then(|number| number.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// `win.innerWidth`/`win.innerHeight` (`packages/utils/src/useScrollLock.ts:30,142-143`).
fn window_inner_size(
    win: &Window,
    inner: fn(&Window) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>,
) -> f64 {
    inner(win)
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0)
}

/// Port of `@floating-ui/utils/dom`'s `isOverflowElement`
/// (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/
/// floating-ui.utils.dom.mjs:45-53`), the external helper upstream imports
/// (`packages/utils/src/useScrollLock.ts:2`).
fn is_overflow_element(element: &Element) -> bool {
    let style = computed_style(element);
    let overflow = get_style_property(&style, "overflow");
    let overflow_y = get_style_property(&style, "overflow-y");
    let overflow_x = get_style_property(&style, "overflow-x");
    let display = get_style_property(&style, "display");

    let concatenated = format!("{overflow}{overflow_y}{overflow_x}");
    (concatenated.contains("auto")
        || concatenated.contains("scroll")
        || concatenated.contains("overlay")
        || concatenated.contains("hidden")
        || concatenated.contains("clip"))
        && display != "inline"
        && display != "contents"
}

/// The viewport-scroller rule (`packages/utils/src/useScrollLock.ts:14-18`): the viewport's
/// overflow comes from `<html>` when it establishes its own scroll container, and propagates
/// from `<body>` otherwise — an `overflow` style on the other element doesn't lock the page.
fn get_viewport_scroller(html: &HtmlElement, body: &HtmlElement) -> HtmlElement {
    let html_element: &Element = html.as_ref();
    if is_overflow_element(html_element) {
        html.clone()
    } else {
        body.clone()
    }
}

/// Whether the page is currently scroll-locked by anyone
/// (`packages/utils/src/useScrollLock.ts:20-22`): the viewport scroller's computed `overflowY`
/// is `hidden` or `clip`.
fn is_page_scroll_locked(win: &Window, html: &HtmlElement, body: &HtmlElement) -> bool {
    let scroller = get_viewport_scroller(html, body);
    let scroller_element: &Element = scroller.as_ref();
    let overflow_y = get_style_property(&computed_style_with(win, scroller_element), "overflow-y");
    overflow_y.contains("hidden") || overflow_y.contains("clip")
}

/// Whether the platform uses inset (layout-taking) scrollbars
/// (`packages/utils/src/useScrollLock.ts:24-31`).
fn has_inset_scrollbars(reference_element: Option<&Element>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = reference_element;
        // `typeof document === 'undefined'` (`packages/utils/src/useScrollLock.ts:25-27`): a
        // realm without a document cannot have scrollbars. A native target has no JS realm at
        // all.
        return false;
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;

        if js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("document"))
            .map(|document| document.is_undefined())
            .unwrap_or(true)
        {
            return false;
        }
        let document = owner_document_for(reference_element);
        // `ownerWindow(doc)` (`packages/utils/src/useScrollLock.ts:29`): a `Document` has no
        // `ownerDocument`, so upstream's resolution falls through to the global window.
        let win = owner_window(None);
        let html = document_element_of(&document);
        let inner_width = window_inner_size(&win, Window::inner_width);
        f64::from(html.client_width()) < inner_width
    }
}

/// Whether `scrollbar-gutter: stable` alone absorbs the scrollbar, probed by measuring the
/// scroll container with a forced scrollbar versus without
/// (`packages/utils/src/useScrollLock.ts:33-62`).
fn supports_stable_scrollbar_gutter(reference_element: Option<&Element>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = reference_element;
        // No JS realm on the host target — the `CSS` global is absent, upstream's guard
        // (`packages/utils/src/useScrollLock.ts:34-39`) resolves false.
        return false;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // `typeof CSS !== 'undefined' && CSS.supports && CSS.supports('scrollbar-gutter',
        // 'stable')` (`packages/utils/src/useScrollLock.ts:34-35`): the two-argument
        // `CSS.supports` form. A realm without the `CSS` global throws inside the binding
        // (the caught `Err`), mirroring the `typeof CSS !== 'undefined'` guard.
        let supported = web_sys::css::supports_with_value("scrollbar-gutter", "stable");
        if !supported.unwrap_or(false) {
            return false;
        }

        let document = owner_document_for(reference_element);
        let html = document_element_of(&document);
        let body = body_of(&document);
        let scroll_container = get_viewport_scroller(&html, &body);

        let html_style = html.style();
        let container_style = scroll_container.style();
        let original_container_overflow_y = get_style_property(&container_style, "overflow-y");
        let original_html_style_gutter = get_style_property(&html_style, "scrollbar-gutter");

        set_style_property(&html_style, "scrollbar-gutter", "stable");

        set_style_property(&container_style, "overflow-y", "scroll");
        let before = scroll_container.offset_width();

        set_style_property(&container_style, "overflow-y", "hidden");
        let after = scroll_container.offset_width();

        set_style_property(
            &container_style,
            "overflow-y",
            &original_container_overflow_y,
        );
        set_style_property(&html_style, "scrollbar-gutter", &original_html_style_gutter);

        before == after
    }
}

/// The overlay-scrollbar lock (`packages/utils/src/useScrollLock.ts:64-87`): hide the
/// viewport scroller's overflow, restoring the exact original inline values on cleanup.
fn prevent_scroll_overlay_scrollbars(reference_element: Option<&Element>) -> Box<dyn FnOnce()> {
    let document = owner_document_for(reference_element);
    let html = document_element_of(&document);
    let body = body_of(&document);

    // If an `overflow` style is present on <html>, we need to lock it, because a lock on
    // <body> won't have any effect. But if <body> has an `overflow` style (like
    // `overflow-x: hidden`), we need to lock it instead, as sticky elements shift otherwise.
    let element_to_lock = get_viewport_scroller(&html, &body);
    let lock_style = element_to_lock.style();
    let original_overflow_y = get_style_property(&lock_style, "overflow-y");
    let original_overflow_x = get_style_property(&lock_style, "overflow-x");

    set_style_property(&lock_style, "overflow-y", "hidden");
    set_style_property(&lock_style, "overflow-x", "hidden");

    Box::new(move || {
        let style = element_to_lock.style();
        set_style_property(&style, "overflow-y", &original_overflow_y);
        set_style_property(&style, "overflow-x", &original_overflow_x);
    })
}

/// The closure-scoped state of the inset-scrollbar lock (`packages/utils/src/useScrollLock.ts:
/// 89-226`): the captured document bindings plus the mutable locals its `lockScroll`,
/// `cleanup`, and `handleResize` closures share.
struct InsetScrollLock {
    html: HtmlElement,
    body: HtmlElement,
    win: Window,
    reference_element: Option<Element>,
    resize_frame: AnimationFrame,
    scroll_top: Cell<i32>,
    scroll_left: Cell<i32>,
    update_gutter_only: Cell<bool>,
}

impl InsetScrollLock {
    /// Upstream `lockScroll` (`packages/utils/src/useScrollLock.ts:105-194`): measure, then
    /// write — never read the DOM past the divider.
    fn lock_scroll(&self) {
        // DOM reads:
        let html_style = self.html.style();
        let body_style = self.body.style();
        let html_computed = computed_style(&self.html);
        let body_computed = computed_style(&self.body);
        let html_scrollbar_gutter_value = get_style_property(&html_computed, "scrollbar-gutter");
        let has_both_edges = html_scrollbar_gutter_value.contains("both-edges");
        let scrollbar_gutter_value = if has_both_edges {
            "stable both-edges"
        } else {
            "stable"
        };

        self.scroll_top.set(self.html.scroll_top());
        self.scroll_left.set(self.html.scroll_left());

        ORIGINAL_HTML_STYLES.with(|snapshot| {
            let mut original = snapshot.borrow_mut();
            original.scrollbar_gutter = get_style_property(&html_style, "scrollbar-gutter");
            original.overflow_y = get_style_property(&html_style, "overflow-y");
            original.overflow_x = get_style_property(&html_style, "overflow-x");
        });
        ORIGINAL_HTML_SCROLL_BEHAVIOR.with(|behavior| {
            *behavior.borrow_mut() = get_style_property(&html_style, "scroll-behavior");
        });

        ORIGINAL_BODY_STYLES.with(|snapshot| {
            let mut original = snapshot.borrow_mut();
            original.position = get_style_property(&body_style, "position");
            original.height = get_style_property(&body_style, "height");
            original.width = get_style_property(&body_style, "width");
            original.box_sizing = get_style_property(&body_style, "box-sizing");
            original.overflow_y = get_style_property(&body_style, "overflow-y");
            original.overflow_x = get_style_property(&body_style, "overflow-x");
            original.scroll_behavior = get_style_property(&body_style, "scroll-behavior");
        });

        let is_scrollable_y = self.html.scroll_height() > self.html.client_height();
        let is_scrollable_x = self.html.scroll_width() > self.html.client_width();
        let has_constant_overflow_y = get_style_property(&html_computed, "overflow-y") == "scroll"
            || get_style_property(&body_computed, "overflow-y") == "scroll";
        let has_constant_overflow_x = get_style_property(&html_computed, "overflow-x") == "scroll"
            || get_style_property(&body_computed, "overflow-x") == "scroll";

        // Values can be negative in Firefox
        let inner_width = window_inner_size(&self.win, Window::inner_width);
        let inner_height = window_inner_size(&self.win, Window::inner_height);
        let scrollbar_width = (inner_width - f64::from(self.body.client_width())).max(0.0);
        let scrollbar_height = (inner_height - f64::from(self.body.client_height())).max(0.0);

        // Avoid shift due to the default <body> margin.
        let margin_y = parse_px(&get_style_property(&body_computed, "margin-top"))
            + parse_px(&get_style_property(&body_computed, "margin-bottom"));
        let margin_x = parse_px(&get_style_property(&body_computed, "margin-left"))
            + parse_px(&get_style_property(&body_computed, "margin-right"));
        let element_to_lock = get_viewport_scroller(&self.html, &self.body);

        let update_gutter_only = supports_stable_scrollbar_gutter(self.reference_element.as_ref());
        self.update_gutter_only.set(update_gutter_only);

        // DOM writes — do not read the DOM past this point!
        if update_gutter_only {
            set_style_property(&html_style, "scrollbar-gutter", scrollbar_gutter_value);
            let lock_style = element_to_lock.style();
            set_style_property(&lock_style, "overflow-y", "hidden");
            set_style_property(&lock_style, "overflow-x", "hidden");
            return;
        }

        set_style_property(&html_style, "scrollbar-gutter", scrollbar_gutter_value);
        set_style_property(&html_style, "overflow-y", "hidden");
        set_style_property(&html_style, "overflow-x", "hidden");

        if is_scrollable_y || has_constant_overflow_y {
            set_style_property(&html_style, "overflow-y", "scroll");
        }
        if is_scrollable_x || has_constant_overflow_x {
            set_style_property(&html_style, "overflow-x", "scroll");
        }

        let body_height = if margin_y != 0.0 || scrollbar_height != 0.0 {
            format!("calc(100dvh - {}px)", margin_y + scrollbar_height)
        } else {
            "100dvh".to_string()
        };
        let body_width = if margin_x != 0.0 || scrollbar_width != 0.0 {
            format!("calc(100vw - {}px)", margin_x + scrollbar_width)
        } else {
            "100vw".to_string()
        };
        set_style_property(&body_style, "position", "relative");
        set_style_property(&body_style, "height", &body_height);
        set_style_property(&body_style, "width", &body_width);
        set_style_property(&body_style, "box-sizing", "border-box");
        // Assign the longhands that `cleanup` restores, so nothing is left behind.
        set_style_property(&body_style, "overflow-y", "hidden");
        set_style_property(&body_style, "overflow-x", "hidden");
        set_style_property(&body_style, "scroll-behavior", "unset");

        self.body.set_scroll_top(self.scroll_top.get());
        self.body.set_scroll_left(self.scroll_left.get());
        self.html
            .set_attribute(SCROLL_LOCKED_ATTRIBUTE, "")
            .expect_throw("Base UI: failed to tag the document as scroll-locked");
        set_style_property(&html_style, "scroll-behavior", "unset");
    }

    /// Upstream `cleanup` (`packages/utils/src/useScrollLock.ts:196-206`).
    fn cleanup(&self) {
        let html_style = self.html.style();
        let body_style = self.body.style();

        ORIGINAL_HTML_STYLES.with(|snapshot| {
            let original = snapshot.borrow();
            set_style_property(&html_style, "scrollbar-gutter", &original.scrollbar_gutter);
            set_style_property(&html_style, "overflow-y", &original.overflow_y);
            set_style_property(&html_style, "overflow-x", &original.overflow_x);
        });
        ORIGINAL_BODY_STYLES.with(|snapshot| {
            let original = snapshot.borrow();
            set_style_property(&body_style, "position", &original.position);
            set_style_property(&body_style, "height", &original.height);
            set_style_property(&body_style, "width", &original.width);
            set_style_property(&body_style, "box-sizing", &original.box_sizing);
            set_style_property(&body_style, "overflow-y", &original.overflow_y);
            set_style_property(&body_style, "overflow-x", &original.overflow_x);
            set_style_property(&body_style, "scroll-behavior", &original.scroll_behavior);
        });

        if !self.update_gutter_only.get() {
            self.html.set_scroll_top(self.scroll_top.get());
            self.html.set_scroll_left(self.scroll_left.get());
            self.html
                .remove_attribute(SCROLL_LOCKED_ATTRIBUTE)
                .expect_throw("Base UI: failed to remove the scroll-locked tag");
            let behavior = ORIGINAL_HTML_SCROLL_BEHAVIOR.with(|behavior| behavior.borrow().clone());
            set_style_property(&html_style, "scroll-behavior", &behavior);
        }
    }

    /// Upstream `handleResize` (`packages/utils/src/useScrollLock.ts:208-211`): undo the lock
    /// and re-apply it from fresh measurements on the next animation frame.
    fn handle_resize(self: &Rc<Self>) {
        self.cleanup();
        let this = Rc::clone(self);
        self.resize_frame.request(move || this.lock_scroll());
    }
}

/// The inset-scrollbar lock (`packages/utils/src/useScrollLock.ts:89-226`).
fn prevent_scroll_inset_scrollbars(reference_element: Option<&Element>) -> Box<dyn FnOnce()> {
    let document = owner_document_for(reference_element);
    let html = document_element_of(&document);
    let body = body_of(&document);
    let win = owner_window_for(Some(&html));

    // Pinch-zoom in Safari causes a shift. Just don't lock scroll if there's any pinch-zoom.
    if platform().engine.webkit {
        let zoomed = win
            .visual_viewport()
            .map(|viewport| viewport.scale() != 1.0)
            .unwrap_or(false);
        if zoomed {
            return Box::new(|| {});
        }
    }

    let lock = Rc::new(InsetScrollLock {
        html,
        body,
        win,
        reference_element: reference_element.cloned(),
        resize_frame: AnimationFrame::create(),
        scroll_top: Cell::new(0),
        scroll_left: Cell::new(0),
        update_gutter_only: Cell::new(false),
    });

    lock.lock_scroll();
    let unsubscribe_resize = add_event_listener(&lock.win, "resize", {
        let listener = Rc::clone(&lock);
        move |_| listener.handle_resize()
    });

    Box::new(move || {
        lock.resize_frame.cancel();
        lock.cleanup();
        // Upstream guards `typeof win.removeEventListener === 'function'` because this
        // cleanup can run after JS test teardown
        // (`packages/utils/src/useScrollLock.ts:219-224`); the browser wasm target always has
        // the function and the host target cannot run this path at all. The handle removes
        // the listener on unsubscribe *and* drop, idempotently.
        unsubscribe_resize.unsubscribe();
    })
}

/// What [`ScrollLocker::restore`] holds: either an applied lock whose cleanup undoes the
/// style mutations, or the external-lock wait state whose cleanup detaches the observer.
enum Restore {
    Wait(ExternalLockWait),
    Lock(Box<dyn FnOnce()>),
}

/// The external-lock wait state (`packages/utils/src/useScrollLock.ts:270-287`): an
/// attributes-only observer on `<html>`/`<body>` plus the callback closure that must outlive
/// the wait.
struct ExternalLockWait {
    observer: MutationObserver,
    callback: Closure<dyn FnMut()>,
}

impl ExternalLockWait {
    /// Upstream `observer.disconnect()` (`packages/utils/src/useScrollLock.ts:274`), with the
    /// callback parked for a later sweep instead of dropped — see [`RETIRED_CALLBACKS`].
    fn disconnect(self) {
        self.observer.disconnect();
        RETIRED_CALLBACKS.with(|retired| retired.borrow_mut().push(self.callback));
    }
}

/// Upstream `ScrollLocker` (`packages/utils/src/useScrollLock.ts:228-302`): the page-wide
/// reference-counted lock state.
struct ScrollLocker {
    lock_count: Cell<i32>,
    restore: RefCell<Option<Restore>>,
    timeout_lock: Timeout,
    timeout_unlock: Timeout,
}

impl ScrollLocker {
    fn new() -> Self {
        Self {
            lock_count: Cell::new(0),
            restore: RefCell::new(None),
            timeout_lock: Timeout::create(),
            timeout_unlock: Timeout::create(),
        }
    }

    /// Upstream `acquire` (`packages/utils/src/useScrollLock.ts:234-240`): count the consumer
    /// and defer the lock onto a 0ms timeout when this is the only one. Returns the release
    /// cleanup the hook registers with its effect.
    fn acquire(self: &Rc<Self>, reference_element: Option<Element>) -> Box<dyn FnOnce()> {
        sweep_retired_callbacks();
        self.lock_count.set(self.lock_count.get() + 1);
        if self.lock_count.get() == 1 && self.restore.borrow().is_none() {
            let this = Rc::clone(self);
            self.timeout_lock
                .start(0, move || this.lock(reference_element.as_ref()));
        }
        let this = Rc::clone(self);
        Box::new(move || this.release())
    }

    /// Upstream `release` (`packages/utils/src/useScrollLock.ts:242-247`): drop the count and
    /// defer the unlock onto a 0ms timeout when the last consumer left and a lock is active.
    fn release(self: &Rc<Self>) {
        sweep_retired_callbacks();
        self.lock_count.set(self.lock_count.get() - 1);
        if self.lock_count.get() == 0 && self.restore.borrow().is_some() {
            let this = Rc::clone(self);
            self.timeout_unlock.start(0, move || this.unlock());
        }
    }

    /// Upstream `unlock` (`packages/utils/src/useScrollLock.ts:249-254`). The restore is taken
    /// before it runs so a re-entrant `lock`/`release` from inside the cleanup cannot observe
    /// a half-cleared state; upstream's order (call, then null) is observationally identical
    /// because its cleanups never re-enter the locker.
    fn unlock(&self) {
        if self.lock_count.get() == 0 {
            match self.restore.borrow_mut().take() {
                Some(Restore::Wait(wait)) => wait.disconnect(),
                Some(Restore::Lock(cleanup)) => cleanup(),
                None => {}
            }
        }
    }

    /// Upstream `lock` (`packages/utils/src/useScrollLock.ts:256-301`).
    fn lock(self: &Rc<Self>, reference_element: Option<&Element>) {
        if self.lock_count.get() == 0 || self.restore.borrow().is_some() {
            return;
        }

        let document = owner_document_for(reference_element);
        let html = document_element_of(&document);
        let body = body_of(&document);
        let win = owner_window_for(Some(&html));

        // The page is already locked, either by the site author or by a non-Base UI overlay
        // that hasn't cleaned up yet. Leave it alone and wait for the lock to clear before
        // taking over, otherwise we'd snapshot the locked state and restore it after our own
        // lock is released.
        if is_page_scroll_locked(&win, &html, &body) {
            let this = Rc::clone(self);
            let reference = reference_element.cloned();
            let locked_win = win.clone();
            let locked_html = html.clone();
            let locked_body = body.clone();
            let callback = Closure::wrap(Box::new(move || {
                if is_page_scroll_locked(&locked_win, &locked_html, &locked_body) {
                    return;
                }
                // Upstream: `observer.disconnect(); this.restore = null;
                // this.lock(referenceElement);` (`packages/utils/src/useScrollLock.ts:274-276`).
                if let Some(Restore::Wait(wait)) = this.restore.borrow_mut().take() {
                    wait.disconnect();
                }
                this.lock(reference.as_ref());
            }) as Box<dyn FnMut()>);

            let observer = MutationObserver::new(callback.as_ref().unchecked_ref())
                .expect_throw("Base UI: failed to create the external scroll-lock observer");

            // Watch every attribute: locks are applied through inline styles, classes, or
            // attributes paired with a stylesheet (`data-scroll-locked` in
            // react-remove-scroll, for example)
            // (`packages/utils/src/useScrollLock.ts:279-281`).
            let options = MutationObserverInit::new();
            options.set_attributes(true);

            let html_node: &Node = html.as_ref();
            let body_node: &Node = body.as_ref();
            observer
                .observe_with_options(html_node, &options)
                .expect_throw("Base UI: failed to observe <html> for an external scroll lock");
            observer
                .observe_with_options(body_node, &options)
                .expect_throw("Base UI: failed to observe <body> for an external scroll lock");

            *self.restore.borrow_mut() =
                Some(Restore::Wait(ExternalLockWait { observer, callback }));
            return;
        }

        let has_overlay_scrollbars = platform().os.ios || !has_inset_scrollbars(reference_element);

        // On iOS, scroll locking does not work if the navbar is collapsed. Due to numerous
        // side effects and bugs that arise on iOS, it must be researched extensively before
        // being enabled to ensure it doesn't cause the following issues:
        // - Textboxes must scroll into view when focused, nor cause a glitchy scroll animation.
        // - The navbar must not force itself into view and cause layout shift.
        // - Scroll containers must not flicker upon closing a popup when it has an exit
        //   animation.
        *self.restore.borrow_mut() = Some(if has_overlay_scrollbars {
            Restore::Lock(prevent_scroll_overlay_scrollbars(reference_element))
        } else {
            Restore::Lock(prevent_scroll_inset_scrollbars(reference_element))
        });
    }
}

/// The upstream `SCROLL_LOCKER` singleton accessor
/// (`packages/utils/src/useScrollLock.ts:304`).
fn scroll_locker() -> Rc<ScrollLocker> {
    SCROLL_LOCKER.with(Rc::clone)
}

/// Locks the scroll of the document when enabled — upstream `useScrollLock`
/// (`packages/utils/src/useScrollLock.ts:312-319`).
///
/// - `enabled` — whether to enable the scroll lock. A reactive source: flipping it re-runs
///   the effect, releasing (and re-acquiring when enabled again), the analog of upstream's
///   dependency array (`:318`). This is what the modal Dialog consumer drives with its open
///   state (`packages/react/src/dialog/root/useDialogRoot.ts:86`).
/// - `reference_element` — element to use as a reference for lock calculations (owner
///   document/window resolution and scrollbar measurements); the second dependency-array
///   entry. Pass `Signal::derive(|| None)` where upstream passes nothing.
///
/// Renders nothing; acquiring is deferred onto a 0ms timeout by the page-wide
/// [`ScrollLocker`], and the last release undoes every style and attribute mutation. Must be
/// called inside a reactive owner (a component).
pub fn use_scroll_lock<E, R>(enabled: E, reference_element: R)
where
    E: Get<Value = bool> + 'static,
    R: Get<Value = Option<Element>> + 'static,
{
    use_iso_layout_effect(move || {
        // Both dependency-array entries are read up front so a change to either re-runs the
        // effect (`packages/utils/src/useScrollLock.ts:318`).
        let enabled = enabled.get();
        let reference_element = reference_element.get();
        if !enabled {
            return;
        }
        let release = scroll_locker().acquire(reference_element);
        let release = SendWrapper::new(release);
        on_cleanup(move || (release.take())());
    });
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::wrappers::read::Signal;

    use super::*;
    use crate::use_timeout::{
        install_dispatch_override_for_tests, reset_dispatch_override_for_tests,
    };

    type Job = Box<dyn FnOnce()>;
    type Id = u32;

    /// The manual queue standing in for the browser's timer queue on the host target: every
    /// registration is queued with its delay, cancel removes it, and [`ManualQueue::flush`]
    /// fires each live registration exactly once. Mirrors the `use_timeout` port's host
    /// tests, which drive the same [`Timeout`] dispatch the locker defers through.
    struct ManualQueue {
        timeouts: Rc<RefCell<Vec<(Id, Rc<RefCell<Option<Job>>>)>>>,
        delays: Rc<RefCell<HashMap<Id, u32>>>,
    }

    impl ManualQueue {
        fn install() -> Self {
            let timeouts: Rc<RefCell<Vec<(Id, Rc<RefCell<Option<Job>>>)>>> =
                Rc::new(RefCell::new(Vec::new()));
            let delays: Rc<RefCell<HashMap<Id, u32>>> = Rc::new(RefCell::new(HashMap::new()));
            let next_id = Cell::new(0u32);

            let queue_timeouts = Rc::clone(&timeouts);
            let queue_delays = Rc::clone(&delays);
            let request = Box::new(move |job: Job, delay: u32| -> Id {
                next_id.set(next_id.get() + 1);
                let id = next_id.get();
                queue_timeouts
                    .borrow_mut()
                    .push((id, Rc::new(RefCell::new(Some(job)))));
                queue_delays.borrow_mut().insert(id, delay);
                id
            });

            let cancel_timeouts = Rc::clone(&timeouts);
            let cancel_delays = Rc::clone(&delays);
            let cancel = Box::new(move |id: Id| {
                cancel_timeouts
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                cancel_delays.borrow_mut().remove(&id);
            });

            install_dispatch_override_for_tests(request, cancel);
            Self { timeouts, delays }
        }

        /// Fires every live registration exactly once, in scheduling order.
        fn flush(&self) {
            let ids: Vec<Id> = self.timeouts.borrow().iter().map(|(id, _)| *id).collect();
            for id in ids {
                let job = {
                    let timeouts = self.timeouts.borrow();
                    timeouts
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .and_then(|(_, slot)| slot.borrow_mut().take())
                };
                self.timeouts
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                self.delays.borrow_mut().remove(&id);
                if let Some(job) = job {
                    job();
                }
            }
        }

        /// The delay a registration was queued with — pins that the locker's deferral uses
        /// the upstream 0ms timeouts.
        fn delay_for(&self, id: Id) -> Option<u32> {
            self.delays.borrow().get(&id).copied()
        }
    }

    impl Drop for ManualQueue {
        fn drop(&mut self) {
            reset_dispatch_override_for_tests();
        }
    }

    /// Installs a counting no-op lock on the locker without touching the DOM, so the state
    /// machine around `restore` is testable on a target with no realm.
    fn install_counting_lock(locker: &ScrollLocker) -> Rc<Cell<u32>> {
        let cleanups = Rc::new(Cell::new(0u32));
        *locker.restore.borrow_mut() = Some(Restore::Lock({
            let cleanups = Rc::clone(&cleanups);
            Box::new(move || cleanups.set(cleanups.get() + 1))
        }));
        cleanups
    }

    // --- ScrollLocker state machine (host-reachable paths only: the counting, the two 0ms
    // deferrals, and lock()'s early-return guards all precede any DOM access).

    // Pins the deferral (`packages/utils/src/useScrollLock.ts:234-240`): the first acquire
    // schedules exactly one lock on a 0ms timeout; further acquires while it is pending
    // neither re-schedule nor stack.
    #[test]
    fn the_first_acquire_schedules_one_deferred_lock_on_a_zero_ms_timeout() {
        let queue = ManualQueue::install();
        let locker = Rc::new(ScrollLocker::new());

        let _release_a = locker.acquire(None);
        let lock_id = locker.timeout_lock.current_id().expect("lock scheduled");
        assert_eq!(
            queue.delay_for(lock_id),
            Some(0),
            "the lock is deferred by 0ms"
        );

        let _release_b = locker.acquire(None);
        assert_eq!(
            locker.timeout_lock.current_id(),
            Some(lock_id),
            "a second acquire reuses the pending lock — no re-schedule, no stack"
        );
        assert_eq!(locker.lock_count.get(), 2);
    }

    // Pins the guard (`packages/utils/src/useScrollLock.ts:257`): a deferred lock whose
    // consumer released before the timeout fired applies nothing — the early return precedes
    // all DOM access, so this runs on the host without a realm.
    #[test]
    fn releasing_before_the_deferred_lock_fires_never_takes_the_lock() {
        let queue = ManualQueue::install();
        let locker = Rc::new(ScrollLocker::new());

        let release = locker.acquire(None);
        release();
        assert_eq!(locker.lock_count.get(), 0);
        assert!(
            locker.timeout_unlock.current_id().is_none(),
            "no lock was applied, so no unlock may be scheduled"
        );

        queue.flush();
        assert!(locker.restore.borrow().is_none());
        assert!(
            locker.timeout_lock.current_id().is_none(),
            "the fired job is spent"
        );
    }

    // Pins the unlock deferral (`packages/utils/src/useScrollLock.ts:242-247`): releasing to
    // zero schedules the unlock only when a lock is actually installed, and the fired unlock
    // runs the installed cleanup exactly once.
    #[test]
    fn releasing_to_zero_schedules_the_unlock_only_while_a_lock_is_installed() {
        let queue = ManualQueue::install();
        let locker = Rc::new(ScrollLocker::new());
        let cleanups = install_counting_lock(&locker);

        let release = locker.acquire(None);
        assert!(locker.timeout_unlock.current_id().is_none());
        release();
        let unlock_id = locker
            .timeout_unlock
            .current_id()
            .expect("unlock scheduled");
        assert_eq!(
            queue.delay_for(unlock_id),
            Some(0),
            "the unlock is deferred by 0ms"
        );

        queue.flush();
        assert_eq!(cleanups.get(), 1, "the installed cleanup ran once");
        assert!(locker.restore.borrow().is_none());
    }

    // Pins the release guard (`packages/utils/src/useScrollLock.ts:244`): with no lock
    // installed, releasing to zero schedules nothing.
    #[test]
    fn releasing_to_zero_without_an_installed_lock_schedules_nothing() {
        let _queue = ManualQueue::install();
        let locker = Rc::new(ScrollLocker::new());

        let release = locker.acquire(None);
        release();
        assert!(locker.timeout_unlock.current_id().is_none());
    }

    // Pins the lock guard (`packages/utils/src/useScrollLock.ts:257-259`): `lock` returns
    // early while a restore is installed — including the wait state — without touching the
    // DOM or disturbing the installed restore.
    #[test]
    fn lock_returns_early_while_a_restore_is_installed() {
        let queue = ManualQueue::install();
        let locker = Rc::new(ScrollLocker::new());
        let cleanups = install_counting_lock(&locker);

        let release = locker.acquire(None);
        queue.flush();
        assert_eq!(
            cleanups.get(),
            0,
            "the deferred lock must not disturb the installed restore"
        );
        release();
        queue.flush();
        assert_eq!(
            cleanups.get(),
            1,
            "the unlock deferred by the release runs the installed cleanup"
        );
        assert!(locker.restore.borrow().is_none());
    }

    // Pins the re-acquire race (`packages/utils/src/useScrollLock.ts:249-254`): an unlock
    // that fires after a consumer re-acquired must keep the page locked — the count guard
    // inside `unlock`.
    #[test]
    fn an_unlock_that_fires_after_a_reacquire_keeps_the_lock() {
        let queue = ManualQueue::install();
        let locker = Rc::new(ScrollLocker::new());
        let cleanups = install_counting_lock(&locker);

        let release = locker.acquire(None);
        release();
        assert!(
            locker.timeout_unlock.current_id().is_some(),
            "unlock deferred"
        );

        let reacquired = locker.acquire(None);
        assert_eq!(locker.lock_count.get(), 1);

        queue.flush();
        assert_eq!(
            cleanups.get(),
            0,
            "the unlock must not run while a consumer is active"
        );
        assert!(
            locker.restore.borrow().is_some(),
            "the lock survives the skipped unlock"
        );

        reacquired();
        queue.flush();
        assert_eq!(
            cleanups.get(),
            1,
            "the eventual unlock runs the cleanup exactly once"
        );
        assert!(locker.restore.borrow().is_none());
    }

    // --- Hook (host target = the non-DOM binding).

    // Pins the SSR analog (`packages/utils/src/useIsoLayoutEffect.ts:6` via the ported
    // `use_iso_layout_effect`): with no `document`, the hook's effect is the noop binding —
    // nothing is acquired and nothing is scheduled, exactly the upstream SSR behavior of the
    // `typeof document !== 'undefined'` selection.
    #[test]
    fn the_hook_is_the_noop_binding_on_a_realm_without_a_document() {
        let _queue = ManualQueue::install();
        let owner = Owner::new();
        owner.set();

        let enabled = RwSignal::new(true);
        let reference_element: Signal<Option<Element>> = Signal::derive(|| None);
        use_scroll_lock(enabled, reference_element);

        assert_eq!(
            SCROLL_LOCKER.with(|locker| locker.lock_count.get()),
            0,
            "the non-DOM binding acquires nothing"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use js_sys::{Function, Reflect};
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;
    use reactive_graph::wrappers::read::Signal;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    // The behavior under test is real DOM mutation (inline styles, attributes, observers), so
    // like the crate's other wasm test modules this runs in a real browser
    // (`.cargo/config.toml` wires the wasm32 runner to chromedriver).
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// The global stub standing in for the browser's timer queue — the `use_timeout` port's
    /// wasm-test precedent, used here to flush the locker's 0ms deferrals deterministically.
    /// Restores the originals on drop so a panicking test cannot poison the later ones.
    struct TimeoutStub {
        original_set: JsValue,
        original_clear: JsValue,
        #[allow(dead_code)]
        set_closure: Closure<dyn FnMut(Function, JsValue) -> JsValue>,
        #[allow(dead_code)]
        clear_closure: Closure<dyn FnMut(JsValue)>,
        timeouts: Rc<RefCell<Vec<(u32, Function)>>>,
    }

    impl TimeoutStub {
        fn install() -> Self {
            let timeouts: Rc<RefCell<Vec<(u32, Function)>>> = Rc::new(RefCell::new(Vec::new()));
            let next_id = Rc::new(Cell::new(0u32));

            let queue = Rc::clone(&timeouts);
            let counter = Rc::clone(&next_id);
            let set_closure = Closure::wrap(Box::new(
                move |callback: Function, _delay: JsValue| -> JsValue {
                    counter.set(counter.get() + 1);
                    let id = counter.get();
                    queue.borrow_mut().push((id, callback));
                    JsValue::from_f64(f64::from(id))
                },
            )
                as Box<dyn FnMut(Function, JsValue) -> JsValue>);

            let queue = Rc::clone(&timeouts);
            let clear_closure = Closure::wrap(Box::new(move |id: JsValue| {
                let id = id.as_f64().unwrap_or(0.0) as u32;
                queue.borrow_mut().retain(|(queued, _)| *queued != id);
            }) as Box<dyn FnMut(JsValue)>);

            let global = js_sys::global();
            let set_key = JsValue::from_str("setTimeout");
            let clear_key = JsValue::from_str("clearTimeout");
            let original_set =
                Reflect::get(&global, &set_key).expect("globalThis.setTimeout must be readable");
            let original_clear = Reflect::get(&global, &clear_key)
                .expect("globalThis.clearTimeout must be readable");
            Reflect::set(&global, &set_key, set_closure.as_ref().unchecked_ref())
                .expect("failed to stub setTimeout");
            Reflect::set(&global, &clear_key, clear_closure.as_ref().unchecked_ref())
                .expect("failed to stub clearTimeout");

            Self {
                original_set,
                original_clear,
                set_closure,
                clear_closure,
                timeouts,
            }
        }

        /// Fires every queued registration exactly once, in scheduling order.
        fn flush(&self) {
            let ids: Vec<u32> = self.timeouts.borrow().iter().map(|(id, _)| *id).collect();
            for id in ids {
                let callback = {
                    let timeouts = self.timeouts.borrow();
                    timeouts
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .map(|(_, callback)| callback.clone())
                };
                self.timeouts
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                if let Some(callback) = callback {
                    callback
                        .call0(&JsValue::UNDEFINED)
                        .expect("the timeout callback threw");
                }
            }
        }
    }

    impl Drop for TimeoutStub {
        fn drop(&mut self) {
            let global = js_sys::global();
            Reflect::set(
                &global,
                &JsValue::from_str("setTimeout"),
                &self.original_set,
            )
            .expect("failed to restore setTimeout");
            Reflect::set(
                &global,
                &JsValue::from_str("clearTimeout"),
                &self.original_clear,
            )
            .expect("failed to restore clearTimeout");
        }
    }

    /// Wipes exactly the state the unit touches — the upstream consumer suites' teardown
    /// (`packages/react/src/dialog/root/DialogRoot.test.tsx:1870-1874`,
    /// `packages/react/src/combobox/positioner/ComboboxPositioner.test.tsx:54-58`).
    struct DocumentCleanup;

    impl DocumentCleanup {
        fn install() -> Self {
            Self
        }
    }

    impl Drop for DocumentCleanup {
        fn drop(&mut self) {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let _ = document
                .document_element()
                .unwrap()
                .remove_attribute("style");
            let _ = document.body().unwrap().remove_attribute("style");
            let _ = document
                .body()
                .unwrap()
                .remove_attribute("data-scroll-locked");
            let _ = document
                .document_element()
                .unwrap()
                .remove_attribute(SCROLL_LOCKED_ATTRIBUTE);
        }
    }

    /// Instance-level global stubs for forcing environment branches — see the inset-path
    /// test. Restores on drop.
    struct RealmStub;

    impl RealmStub {
        fn install_inset_path() -> Self {
            // `CSS.supports('scrollbar-gutter', 'stable')` → false, so the gutter-only branch
            // is skipped (`packages/utils/src/useScrollLock.ts:34-39,151`).
            eval(
                "Object.defineProperty(window, 'CSS', { configurable: true, value: { supports: () => false } });",
            );
            // A viewport wider than the document establishes inset scrollbars
            // (`packages/utils/src/useScrollLock.ts:24-31`); the port reads both sizes
            // through the window/document instances these getters shadow.
            eval(
                "Object.defineProperty(window, 'innerWidth', { configurable: true, get: () => 1000 });",
            );
            eval(
                "Object.defineProperty(window, 'innerHeight', { configurable: true, get: () => 1000 });",
            );
            eval(
                "Object.defineProperty(document.documentElement, 'clientWidth', { configurable: true, get: () => 970 });",
            );
            RealmStub
        }
    }

    impl Drop for RealmStub {
        fn drop(&mut self) {
            eval(
                "delete window.CSS; delete window.innerWidth; delete window.innerHeight; delete document.documentElement.clientWidth;",
            );
        }
    }

    fn eval(script: &str) {
        js_sys::eval(script).expect("the test setup script threw");
    }

    fn document_element() -> HtmlElement {
        document_element_of(&web_sys::window().unwrap().document().unwrap())
    }

    fn body() -> HtmlElement {
        body_of(&web_sys::window().unwrap().document().unwrap())
    }

    fn inline_style(element: &HtmlElement, property: &str) -> String {
        get_style_property(&element.style(), property)
    }

    fn set_inline_style(element: &HtmlElement, property: &str, value: &str) {
        set_style_property(&element.style(), property, value);
    }

    /// Makes `<html>` the viewport scroller for the duration of a test: with a clean page the
    /// chromedriver realm has overlay scrollbars and `<html>` establishes no scroll
    /// container, so the lock would target `<body>` — pinning the scroller makes the
    /// lock's target deterministic (the upstream suites use the same setup,
    /// `packages/react/src/dialog/root/DialogRoot.test.tsx:1972`).
    fn make_html_the_viewport_scroller() {
        set_inline_style(&document_element(), "overflow", "auto");
    }

    /// A realm must not carry a lock, a wait state, or a parked callback across tests.
    fn assert_singleton_released() {
        assert_eq!(
            SCROLL_LOCKER.with(|locker| locker.lock_count.get()),
            0,
            "every acquired consumer must be released before the test ends"
        );
        assert!(
            SCROLL_LOCKER.with(|locker| locker.restore.borrow().is_none()),
            "no lock or wait state may outlive the test"
        );
        assert_eq!(
            RETIRED_CALLBACKS.with(|retired| retired.borrow().len()),
            0,
            "no parked observer callback may outlive the test"
        );
    }

    /// Yields to the microtask queue so a queued `MutationObserver` callback runs.
    async fn flush_microtasks() {
        let promise = js_sys::Promise::resolve(&JsValue::UNDEFINED);
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("microtask flush failed");
    }

    // Pins the deferral (`packages/utils/src/useScrollLock.ts:234-240`; spec "State model"):
    // acquiring does not lock synchronously — the lock lands on the next tick, and the last
    // release undoes every mutation.
    #[wasm_bindgen_test]
    fn acquiring_defers_the_lock_to_the_next_tick() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        make_html_the_viewport_scroller();

        let release = scroll_locker().acquire(None);
        assert_ne!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the lock is deferred, not synchronous"
        );

        stub.flush();
        assert_eq!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the deferred lock hides the viewport scroller's overflow"
        );

        release();
        stub.flush();
        assert_ne!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the release removes every inline mutation"
        );
        assert_singleton_released();
    }

    // Pins the restore of exact pre-lock inline values (spec "Edge cases" — UNVERIFIED
    // upstream, inferred from `packages/utils/src/useScrollLock.ts:114-122,196-206`).
    #[wasm_bindgen_test]
    fn the_release_restores_the_exact_pre_lock_inline_values() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        make_html_the_viewport_scroller();
        set_inline_style(&body(), "position", "static");

        let release = scroll_locker().acquire(None);
        stub.flush();
        assert_eq!(inline_style(&document_element(), "overflow-y"), "hidden");

        release();
        stub.flush();
        assert_eq!(
            inline_style(&document_element(), "overflow-y"),
            "auto",
            "the exact pre-lock inline value is restored"
        );
        assert_ne!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the unset overflow-x is removed again, not left hidden"
        );
        assert_eq!(
            inline_style(&body(), "position"),
            "static",
            "<body>'s pre-lock position is restored"
        );
        assert_singleton_released();
    }

    // Pins the reference counting (spec "Edge cases" — the menubar handoff shape): the page
    // stays locked until the LAST consumer releases
    // (`packages/utils/src/useScrollLock.ts:234-247`).
    #[wasm_bindgen_test]
    fn the_page_stays_locked_until_the_last_consumer_releases() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        make_html_the_viewport_scroller();

        let release_a = scroll_locker().acquire(None);
        stub.flush();
        assert_eq!(inline_style(&document_element(), "overflow-x"), "hidden");

        let release_b = scroll_locker().acquire(None);
        release_a();
        stub.flush();
        assert_eq!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "one remaining consumer keeps the page locked"
        );

        release_b();
        stub.flush();
        assert_ne!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the last release unlocks the page"
        );
        assert_singleton_released();
    }

    // Pins the viewport-scroller rule (`packages/utils/src/useScrollLock.ts:16-18`; the
    // upstream consumer helper `hasOwnScrollLock`,
    // `packages/react/src/dialog/root/DialogRoot.test.tsx:2258-2260`): an external
    // `<body>`-only lock is NOT a page lock, and the unit still applies its own lock on
    // `<html>` instead of waiting.
    #[wasm_bindgen_test]
    fn an_external_body_only_lock_does_not_defer_the_units_own_lock() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();

        // <html> owns the viewport scroll (`overflow-y: scroll` keeps it the scroller while
        // not matching the hidden/clip page-lock test), so `overflow-y: hidden` on <body>
        // leaves the page scrollable and must not be mistaken for an effective lock.
        set_inline_style(&document_element(), "overflow-y", "scroll");
        set_inline_style(&body(), "overflow-y", "hidden");

        let release = scroll_locker().acquire(None);
        stub.flush();
        assert_eq!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the unit applied its own lock instead of waiting on the <body> lock"
        );

        release();
        stub.flush();
        assert_singleton_released();
    }

    // Pins the external-lock takeover (`packages/utils/src/useScrollLock.ts:269-288`; spec
    // "State model"): with the page already locked, the unit waits instead of snapshotting
    // the locked state; when the external lock clears it takes over; and the final unlock
    // restores the state from *its own* lock — not the externally locked state, which would
    // leave the page locked forever.
    #[wasm_bindgen_test(async)]
    async fn an_external_page_lock_is_waited_out_then_taken_over() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();

        // A simulated third-party locker: `overflow-y: clip` on <html> is a distinct,
        // page-locking value the unit's own lock would never write.
        set_inline_style(&document_element(), "overflow-y", "clip");

        let release = scroll_locker().acquire(None);
        stub.flush();
        assert_eq!(
            inline_style(&document_element(), "overflow-y"),
            "clip",
            "the external lock is left untouched while waiting"
        );
        assert_ne!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "the unit must not stack a second lock while the page is externally locked"
        );
        assert_ne!(
            inline_style(&body(), "overflow-y"),
            "hidden",
            "nothing anywhere on the page is locked by the unit while it waits"
        );

        // The external locker releases; the attributes-only observer fires as a microtask
        // and the unit takes over. The chromedriver realm has overlay scrollbars and, with
        // the external lock cleared, `<html>` no longer establishes a scroll container — so
        // the takeover lock targets `<body>`.
        set_inline_style(&document_element(), "overflow-y", "");
        flush_microtasks().await;
        assert_eq!(
            inline_style(&body(), "overflow-x"),
            "hidden",
            "the unit takes over the moment the external lock clears"
        );

        // Closing the last consumer restores the state from the unit's own lock — the page
        // must not still carry the external locker's value.
        release();
        stub.flush();
        assert_eq!(
            inline_style(&document_element(), "overflow-y"),
            "",
            "the externally locked value must not be restored as if it were the unit's own"
        );
        assert_ne!(inline_style(&body(), "overflow-x"), "hidden");
        assert_singleton_released();
    }

    // Pins the full inset path's distinctive outputs
    // (`packages/utils/src/useScrollLock.ts:178-193`) by forcing the inset branch: a real
    // Chrome realm supports `scrollbar-gutter: stable` and has overlay scrollbars, so
    // `CSS.supports` is stubbed to false and the inset measurement is faked with
    // instance-level getters — the same globals the port reads.
    #[wasm_bindgen_test]
    fn the_inset_path_repositions_the_body_and_tags_the_document() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        let _realm = RealmStub::install_inset_path();

        let release = scroll_locker().acquire(None);
        stub.flush();
        let html = document_element();
        let body_element = body();
        assert_eq!(
            html.get_attribute(SCROLL_LOCKED_ATTRIBUTE),
            Some("".to_string()),
            "the inset lock tags <html> with its attribute"
        );
        assert_eq!(inline_style(&body_element, "position"), "relative");
        assert_eq!(inline_style(&body_element, "box-sizing"), "border-box");
        assert_ne!(
            inline_style(&body_element, "height"),
            "",
            "the body is sized to the viewport"
        );

        release();
        stub.flush();
        assert_eq!(
            html.get_attribute(SCROLL_LOCKED_ATTRIBUTE),
            None,
            "the attribute is removed on unlock"
        );
        assert_ne!(inline_style(&body_element, "position"), "relative");
        assert_singleton_released();
    }

    // Pins the viewport-scroller decision (`packages/utils/src/useScrollLock.ts:16-18`) at
    // the unit level.
    #[wasm_bindgen_test]
    fn the_viewport_scroller_follows_the_overflow_style_of_html() {
        let html = document_element();
        let body_element = body();

        set_inline_style(&html, "overflow", "");
        assert_same_element(&get_viewport_scroller(&html, &body_element), &body_element);

        set_inline_style(&html, "overflow-x", "hidden");
        assert_same_element(&get_viewport_scroller(&html, &body_element), &html);

        set_inline_style(&html, "overflow-x", "");
        set_inline_style(&html, "overflow", "clip");
        assert_same_element(&get_viewport_scroller(&html, &body_element), &html);

        set_inline_style(&html, "overflow", "");
    }

    // Pins the ported `isOverflowElement`
    // (`node_modules/.pnpm/@floating-ui+utils@0.2.12/.../floating-ui.utils.dom.mjs:45-53`):
    // the token set and the display exclusion.
    #[wasm_bindgen_test]
    fn is_overflow_element_matches_the_floating_ui_rule() {
        let document = web_sys::window().unwrap().document().unwrap();
        let element: HtmlElement = document
            .create_element("div")
            .expect("failed to create the test div")
            .dyn_into()
            .expect("the test div is an HTML element");
        // Computed styles are only meaningful for a rendered element.
        document
            .body()
            .unwrap()
            .append_child(&element)
            .expect("failed to attach the test div");

        for overflow in ["auto", "scroll", "overlay", "hidden", "clip"] {
            set_inline_style(&element, "overflow", overflow);
            assert!(
                is_overflow_element(element.as_ref()),
                "overflow: {overflow} establishes a scroll container"
            );
        }
        set_inline_style(&element, "overflow", "visible");
        assert!(!is_overflow_element(element.as_ref()));

        set_inline_style(&element, "overflow", "hidden");
        set_inline_style(&element, "display", "inline");
        assert!(
            !is_overflow_element(element.as_ref()),
            "display: inline never establishes a scroll container"
        );
        set_inline_style(&element, "display", "contents");
        assert!(
            !is_overflow_element(element.as_ref()),
            "display: contents never establishes a scroll container"
        );
        element.remove();
    }

    // --- Hook wiring in a real realm.

    // Pins the effect lifecycle (`packages/utils/src/useScrollLock.ts:312-319`): enabled
    // acquires through the singleton, owner disposal releases.
    #[wasm_bindgen_test]
    fn the_hook_locks_while_enabled_and_unlocks_on_owner_disposal() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        let _ = any_spawner::Executor::init_futures_executor();
        make_html_the_viewport_scroller();

        let owner = Owner::new();
        owner.set();

        let enabled = RwSignal::new(true);
        let reference_element: Signal<Option<Element>> = Signal::derive(|| None);
        use_scroll_lock(enabled, reference_element);
        assert_eq!(
            SCROLL_LOCKER.with(|locker| locker.lock_count.get()),
            1,
            "the first (synchronous) effect run acquires immediately"
        );

        stub.flush();
        assert_eq!(inline_style(&document_element(), "overflow-x"), "hidden");

        owner.cleanup();
        stub.flush();
        assert_ne!(
            inline_style(&document_element(), "overflow-x"),
            "hidden",
            "owner disposal releases the lock"
        );
        assert_singleton_released();
    }

    // Pins the disabled start (`packages/utils/src/useScrollLock.ts:314-316`): a disabled
    // consumer acquires nothing.
    #[wasm_bindgen_test]
    fn the_hook_acquires_nothing_while_disabled() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        let _ = any_spawner::Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let enabled = RwSignal::new(false);
        let reference_element: Signal<Option<Element>> = Signal::derive(|| None);
        use_scroll_lock(enabled, reference_element);
        stub.flush();
        assert_ne!(inline_style(&document_element(), "overflow-x"), "hidden");
        assert_ne!(inline_style(&body(), "overflow-x"), "hidden");
        assert_singleton_released();
    }

    // Pins the dependency-array replacement (the `Get` adaptation): flipping `enabled`
    // re-runs the effect, which releases and re-acquires — the load-bearing reactive flip
    // the dialog consumer depends on
    // (`packages/react/src/dialog/root/useDialogRoot.ts:86`).
    #[wasm_bindgen_test]
    fn flipping_enabled_releases_and_reacquires() {
        let _cleanup = DocumentCleanup::install();
        let stub = TimeoutStub::install();
        let _ = any_spawner::Executor::init_futures_executor();
        make_html_the_viewport_scroller();

        let owner = Owner::new();
        owner.set();

        let enabled = RwSignal::new(false);
        let reference_element: Signal<Option<Element>> = Signal::derive(|| None);
        use_scroll_lock(enabled, reference_element);
        stub.flush();
        assert_ne!(inline_style(&document_element(), "overflow-x"), "hidden");

        enabled.set(true);
        any_spawner::Executor::poll_local();
        assert_eq!(
            SCROLL_LOCKER.with(|locker| locker.lock_count.get()),
            1,
            "the re-run acquires"
        );
        stub.flush();
        assert_eq!(inline_style(&document_element(), "overflow-x"), "hidden");

        enabled.set(false);
        any_spawner::Executor::poll_local();
        stub.flush();
        assert_eq!(
            SCROLL_LOCKER.with(|locker| locker.lock_count.get()),
            0,
            "the re-run releases"
        );
        assert_ne!(inline_style(&document_element(), "overflow-x"), "hidden");
        assert_singleton_released();
    }

    fn assert_same_element(actual: &HtmlElement, expected: &HtmlElement) {
        assert!(
            js_sys::Object::from(actual.clone()) == js_sys::Object::from(expected.clone()),
            "the viewport scroller must be the same element"
        );
    }
}
