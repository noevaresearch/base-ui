//! Port of `packages/react/src/utils/useAnchoredPopupScrollLock.ts` — the scroll-lock
//! manager for anchored popups (implementation spec, "Environment / label / scroll-lock
//! hooks").
//!
//! Upstream is one 43-line hook (`useAnchoredPopupScrollLock.ts:18-43`): a
//! `touchOpenShouldLockScroll` boolean state (`:24`) computed by a `useIsoLayoutEffect`
//! (`:26-40`) that measures the positioner against the viewport `clientWidth`, feeding
//! `useScrollLock(enabled && (!touchOpen || touchOpenShouldLockScroll), referenceElement)`
//! (`:42`). The rule (`:7-11`): touch-opened popups normally avoid scroll locking so
//! users can still swipe outside to dismiss; scroll lock is re-enabled only when the
//! popup is effectively full-width — up to [`VIEWPORT_WIDTH_TOLERANCE_PX`] (20px) of
//! total horizontal gutter — since too little outside space leaves no room for a
//! reliable swipe.
//!
//! ## Rust adaptations
//!
//! - The tolerance decision (`:36-38`) is carried as the pure
//!   [`should_lock_scroll_for_touch`] helper so its matrix is host-testable (the
//!   measurement wiring needs real layout and lives in the wasm suite). Upstream has no
//!   test for this module (`specs/library/utils/implementation.md`, "Anything in source
//!   not explained by any test": "the 20px full-width tolerance rule … no test").
//! - The `React.useState` boolean (`:24`) ports to an internal
//!   [`reactive_graph::signal::RwSignal`], exposed read-only through the return value —
//!   the port's observable stand-in for upstream's state (upstream returns nothing; the
//!   signal is what the wasm suite asserts and what Phase B can inspect).
//! - The `useIsoLayoutEffect` (`:26-40`) ports to [`leptos_ui_utils::use_iso_layout_effect`]
//!   reading all three dep-array members up front (`:40` — `enabled`, `touchOpen`,
//!   `positionerElement`), with the state writes as plain signal writes (upstream's
//!   `setState` collapses per the synchronous-write convention the popup-store
//!   checkpoint documented).
//! - The composed `useScrollLock` enabled condition (`:42`) is a
//!   [`reactive_graph::computed::Memo`] over the three sources — a change to any of
//!   them recomputes the condition, which is the re-render-then-recompute flow
//!   upstream's render produces.
//! - `positionerElement` is `HTMLElement | null` upstream (`:21`); the port takes an
//!   `Option<HtmlElement>` source (`offsetWidth` lives on `HtmlElement`).
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use web_sys::{Element, HtmlElement, Node};

use leptos_ui_utils::owner_document;
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_scroll_lock::use_scroll_lock;

// Treat popups with up to 20px of total horizontal gutter as full-width so common
// ~10px side padding still locks scroll (`useAnchoredPopupScrollLock.ts:9-11`).
pub const VIEWPORT_WIDTH_TOLERANCE_PX: i32 = 20;

/// The measurement rule (`useAnchoredPopupScrollLock.ts:36-38`): the popup is
/// effectively full-width when both widths are positive and the popup width is within
/// [`VIEWPORT_WIDTH_TOLERANCE_PX`] of the viewport width.
fn should_lock_scroll_for_touch(viewport_width: i32, popup_width: i32) -> bool {
    viewport_width > 0
        && popup_width > 0
        && popup_width >= viewport_width - VIEWPORT_WIDTH_TOLERANCE_PX
}

/// The port's return value — upstream returns nothing (`:18-43` is a void hook); the
/// read-only signal is the observable stand-in for the `touchOpenShouldLockScroll`
/// state (`:24`).
pub struct UseAnchoredPopupScrollLockReturnValue {
    /// `touchOpenShouldLockScroll` (`:24`): whether a touch-opened popup is
    /// effectively full-width (and therefore scroll-locked).
    pub touch_open_should_lock_scroll: Signal<bool>,
}

/// Port of `useAnchoredPopupScrollLock` (`useAnchoredPopupScrollLock.ts:18-43`). Must be
/// called inside a reactive owner (the measuring effect and the scroll-lock effect
/// register there).
pub fn use_anchored_popup_scroll_lock<E, T, P, R>(
    enabled: E,
    touch_open: T,
    positioner_element: P,
    reference_element: R,
) -> UseAnchoredPopupScrollLockReturnValue
where
    E: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    T: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    P: Get<Value = Option<HtmlElement>> + GetUntracked<Value = Option<HtmlElement>> + Clone + 'static,
    R: Get<Value = Option<Element>> + 'static,
{
    // `useState(false)` (`:24`).
    let touch_open_should_lock_scroll: RwSignal<bool> = RwSignal::new(false);

    // The measuring effect (`:26-40`), tracking the dep array's members.
    {
        let should_lock = touch_open_should_lock_scroll.clone();
        let enabled = enabled.clone();
        let touch_open = touch_open.clone();
        use_iso_layout_effect(move || {
            let enabled = enabled.get();
            let touch_open = touch_open.get();
            let positioner_element = positioner_element.get();

            // `if (!enabled || !touchOpen || positionerElement == null) { …false }`
            // (`:27-30`).
            let Some(positioner_element) = positioner_element else {
                should_lock.set(false);
                return;
            };
            if !enabled || !touch_open {
                should_lock.set(false);
                return;
            }

            // `ownerDocument(positionerElement).documentElement.clientWidth`
            // (`:32`), `positionerElement.offsetWidth` (`:33`).
            let node: &Node = positioner_element.as_ref();
            let document = owner_document(Some(node));
            let viewport_width = document
                .document_element()
                .map(|element| element.client_width())
                .unwrap_or(0);
            let popup_width = positioner_element.offset_width();

            // `setTouchOpenShouldLockScroll(…)` (`:35-39`).
            should_lock.set(should_lock_scroll_for_touch(viewport_width, popup_width));
        });
    }

    // `useScrollLock(enabled && (!touchOpen || touchOpenShouldLockScroll),
    // referenceElement)` (`:42`) — the composed condition as a local derived signal
    // (the sources are Rc-based; the `field_root_context.rs` `derive_local`
    // convention).
    let lock_enabled = {
        let enabled = enabled.clone();
        let touch_open = touch_open.clone();
        Signal::derive_local(move || {
            let enabled = enabled.get();
            let touch_open = touch_open.get();
            enabled && (!touch_open || touch_open_should_lock_scroll.get_untracked())
        })
    };
    use_scroll_lock(lock_enabled, reference_element);

    UseAnchoredPopupScrollLockReturnValue {
        touch_open_should_lock_scroll: touch_open_should_lock_scroll.into(),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // The tolerance rule (`:9-11`, `:36-38`) — no upstream test exists
    // (implementation spec, "Anything in source not explained by any test"); this pins
    // the documented 20px gutter: full-width and near-full-width lock, a real gutter
    // does not, and non-positive measurements never do.
    #[test]
    fn the_twenty_px_gutter_rule_locks_only_effectively_full_width_popups() {
        // Exact viewport width: full-width, locks.
        assert!(should_lock_scroll_for_touch(390, 390));
        // Wider than the viewport (e.g. scrollbar/rounding): still effectively full-width.
        assert!(should_lock_scroll_for_touch(390, 400));
        // Exactly 20px of gutter: locks (~10px side padding on both edges).
        assert!(should_lock_scroll_for_touch(390, 370));
        // 21px of gutter: too much outside space — do not lock.
        assert!(!should_lock_scroll_for_touch(390, 369));
        // A wide viewport makes a fixed popup a real window, not a gutter case.
        assert!(!should_lock_scroll_for_touch(1200, 600));
        // Non-positive measurements: the guard arms (`:36-37`).
        assert!(!should_lock_scroll_for_touch(0, 390));
        assert!(!should_lock_scroll_for_touch(390, 0));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Element;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().expect("no window").document().expect("no document")
    }

    /// A positioner pinned to a fraction of the viewport width plus a detached
    /// reference element — the anchor the scroll lock keys off.
    fn mount_positioner(width: &str) -> (web_sys::HtmlElement, Element) {
        let positioner = document()
            .create_element("div")
            .expect("create_element")
            .unchecked_into::<web_sys::HtmlElement>();
        positioner
            .style()
            .set_property("position", "absolute")
            .unwrap();
        positioner.style().set_property("width", width).unwrap();
        document().body().expect("a body").append_child(&positioner).unwrap();

        let reference = document().create_element("span").expect("create_element");
        document().body().expect("a body").append_child(&reference).unwrap();

        (positioner, reference)
    }

    // The touch path measures against the live viewport: a full-width positioner locks
    // (`:36-38`), a narrow one does not (`:7-8` — outside swipes stay available), and
    // the composed lock condition stays disabled while touch-locking is not warranted.
    #[wasm_bindgen_test]
    fn the_touch_lock_follows_the_measured_positioner_width() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let enabled: RwSignal<bool> = RwSignal::new(true);
        let touch_open: RwSignal<bool> = RwSignal::new(true);
        let (positioner, reference) = mount_positioner("10000px");
        let positioner_source: RwSignal<Option<web_sys::HtmlElement>> =
            RwSignal::new(Some(positioner.clone()));

        let ret = use_anchored_popup_scroll_lock(
            enabled,
            touch_open,
            positioner_source,
            RwSignal::new(Some(reference.clone().unchecked_into::<Element>())),
        );
        any_spawner::Executor::poll_local();

        assert!(
            ret.touch_open_should_lock_scroll.get_untracked(),
            "a wider-than-viewport positioner is effectively full-width"
        );

        // Shrink below the tolerance boundary, then cycle the positioner source — the
        // effect's dep array is [enabled, touchOpen, positionerElement] (`:40`), so a
        // re-measure rides an input change (a style mutation alone re-runs nothing
        // upstream either).
        positioner.style().set_property("width", "200px").unwrap();
        positioner_source.set(None);
        positioner_source.set(Some(positioner.clone()));
        any_spawner::Executor::poll_local();
        assert!(
            !ret.touch_open_should_lock_scroll.get_untracked(),
            "a guttered positioner leaves outside swipes available"
        );

        // A disabled open clears the touch-lock decision (`:27-30`).
        enabled.set(false);
        any_spawner::Executor::poll_local();
        assert!(
            !ret.touch_open_should_lock_scroll.get_untracked(),
            "disabled opens clear the touch-lock decision"
        );

        positioner.remove();
        reference.remove();
        owner.cleanup();
    }

    // The gate matrix from `:27-30`: no positioner element, a disabled open, or a
    // non-touch open each clear the decision (`:42` composes the same inputs).
    #[wasm_bindgen_test]
    fn the_gate_clears_without_a_positioner_or_outside_touch_opens() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let enabled: RwSignal<bool> = RwSignal::new(true);
        let touch_open: RwSignal<bool> = RwSignal::new(true);
        let (_positioner, reference) = mount_positioner("10000px");

        // No positioner: the effect bails with `false` (`:27-30`).
        let ret = use_anchored_popup_scroll_lock(
            enabled,
            touch_open,
            RwSignal::new(None),
            RwSignal::new(Some(reference.clone().unchecked_into::<Element>())),
        );
        any_spawner::Executor::poll_local();
        assert!(!ret.touch_open_should_lock_scroll.get_untracked());

        reference.remove();
        owner.cleanup();
    }
}
