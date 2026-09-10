//! Port of `packages/react/src/utils/closePart.tsx` — the count-up/count-down
//! registration context telling a popup whether any Close part is rendered inside it
//! (implementation.md, "Context providers/consumers": the provider side is
//! `useClosePartCount`, consumed by `PopoverPopup`; the consumer side is
//! `useClosePartRegistration`, consumed by `PopoverClose`).
//!
//! What crosses the boundary: a stable register function returning its own cleanup;
//! what flows back: a boolean telling the popup whether any Close part is rendered
//! inside it. No test in the unit targets this module directly
//! (implementation.md, "Submodules with no test anywhere": "`closePart.tsx` — count
//! registration/cleanup, `hasClosePart` derivation: no test"), so the ported behavior
//! is pinned by tests here.
//!
//! Rust adaptations:
//! - `ClosePartContext` is the unit's only React context; the port provides it through
//!   the reactive owner behind the `SendWrapper` bridge (the `labelable_provider`
//!   precedent), with [`provide_close_part_context`] as the wiring the popup body
//!   performs before rendering its children.
//! - Upstream wraps `register` in `useStableCallback` so the context value's identity
//!   survives re-renders (`React.useMemo` over the stable callback); the port's
//!   component bodies run once, so the `Rc` closure is stable by construction and the
//!   wrapper collapses (the `use_hook_order`-style adaptation).
//! - `useClosePartRegistration`'s `useIsoLayoutEffect` cleanup return maps to
//!   `on_cleanup` (the crate's effect-teardown convention).

use std::rc::Rc;

use leptos_ui_utils::merge_cleanups::CleanupFn;
use reactive_graph::owner::{on_cleanup, provide_context, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;

/// Port of `ClosePartContextValue` (`closePart.tsx:6-8`): registering a close part
/// returns a cleanup that unregisters it.
#[derive(Clone)]
pub struct ClosePartContextValue {
    /// `register` (`closePart.tsx:7`) — the stable registration callback.
    pub register: Rc<dyn Fn() -> CleanupFn>,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `labelable_provider`
/// precedent).
pub type SharedClosePartContext = SendWrapper<ClosePartContextValue>;

/// Port of `useClosePartCount` (`closePart.tsx:12-29`): the provider side. The count
/// signal starts at 0, each registration bumps it, and each cleanup clamps it back
/// down with the `Math.max(0, ...)` floor (`:19`). The second return is the
/// `hasClosePart` derivation (`:27` — `closePartCount > 0`).
///
/// Must be called inside a reactive owner; the caller provides
/// [`SharedClosePartContext`] via [`provide_close_part_context`] for the
/// [`use_close_part_registration`] consumers.
pub fn use_close_part_count() -> (ClosePartContextValue, impl Fn() -> bool) {
    let close_part_count = RwSignal::new(0u32);

    // `useStableCallback(() => { setClosePartCount(count => count + 1); return () => {
    // setClosePartCount(count => Math.max(0, count - 1)); }; })` (`:15-21`). The
    // `saturating_sub` is the `Math.max(0, ...)` floor.
    let counter = close_part_count;
    let register: Rc<dyn Fn() -> CleanupFn> = Rc::new(move || {
        counter.set(counter.get_untracked() + 1);

        let cleanup_counter = counter;
        Box::new(move || {
            cleanup_counter.set(cleanup_counter.get_untracked().saturating_sub(1));
        })
    });

    let context = ClosePartContextValue { register };

    let has_close_part = move || close_part_count.get() > 0;
    (context, has_close_part)
}

/// Port of `useClosePartRegistration` (`closePart.tsx:31-36`): the consumer side.
/// Registers through the provided context on mount and unregisters on unmount via the
/// returned cleanup; absent a context provider, registration is a no-op
/// (`context?.register()` — `:35`).
pub fn use_close_part_registration() {
    let context = use_context::<SharedClosePartContext>();

    if let Some(shared) = context {
        let cleanup = (shared.register)();
        // The cleanup box is not `Send`; the `on_cleanup` contract requires it — the
        // `SendWrapper` bridge (the `field_register_control.rs` convention).
        let cleanup = SendWrapper::new(std::cell::Cell::new(Some(cleanup)));
        on_cleanup(move || {
            // `Cell::take` fully qualified — `SendWrapper` has its own `take` method.
            if let Some(cleanup) = std::cell::Cell::take(&cleanup) {
                (cleanup)();
            }
        });
    }
}

/// Provides the close-part context to the calling owner's descendants — the wiring the
/// `PopoverPopup` port performs before rendering its children.
pub fn provide_close_part_context(context: ClosePartContextValue) {
    provide_context(SendWrapper::new(context));
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use reactive_graph::owner::Owner;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    // The count-up/count-down matrix: `hasClosePart` flips on the first registration
    // and back on the last cleanup; cleanups clamp at zero (`:19`); each registration
    // is counted individually.
    #[wasm_bindgen_test]
    fn registration_counts_up_and_cleanup_counts_down_with_a_zero_floor() {
        let owner = Owner::new();
        owner.set();

        let (context, has_close_part) = use_close_part_count();
        assert!(!has_close_part(), "no close part registered yet");

        let first = (context.register)();
        assert!(
            has_close_part(),
            "the first registration flips hasClosePart on"
        );

        let second = (context.register)();
        assert!(has_close_part(), "a second registration keeps it on");

        (first)();
        assert!(has_close_part(), "one registration is still present");

        (second)();
        assert!(!has_close_part(), "the last cleanup flips hasClosePart off");

        // The `saturating_sub` floor: the count can never wrap below zero. (The port's
        // cleanups are `FnOnce`, so a double-cleanup of one handle cannot occur — the
        // floor guards the count arithmetic itself.)
        owner.unset();
    }

    // The consumer contract (`closePart.tsx:31-36`): registering through the provided
    // context counts, and disposal unregisters; without a provider the hook is inert.
    #[wasm_bindgen_test]
    fn the_consumer_registers_through_the_context_and_is_inert_without_one() {
        let owner = Owner::new();
        owner.set();

        let (context, has_close_part) = use_close_part_count();

        // Without a provider: the consumer is a no-op (`context?.register()`).
        use_close_part_registration();
        assert!(!has_close_part(), "no provider, no registration");

        provide_close_part_context(context);

        // Inside the provider's owner chain the consumer registers and disposal
        // unregisters through the returned cleanup.
        let nested = Owner::new();
        nested.set();
        use_close_part_registration();
        assert!(
            has_close_part(),
            "the consumer registered through the context"
        );
        nested.cleanup();
        nested.unset();
        assert!(!has_close_part(), "disposal unregistered");

        owner.unset();
    }
}
