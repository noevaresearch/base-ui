//! Port of `packages/react/src/utils/useOpenInteractionType.ts` — the open-method
//! tracking pair behind every popup family's trigger and its `openMethod`-driven
//! styling (implementation spec, "Interaction-type & click-semantics hooks").
//!
//! Upstream is two hooks (`useOpenInteractionType.ts:8-62`):
//!
//! - `useOpenMethodTriggerProps(open, setOpenMethod)` (`:8-37`) — a `useStableCallback`
//!   click handler gated on the live open state, composed through
//!   `useEnhancedClickHandler` (`:28`) into the `{ onClick, onPointerDown }` trigger bag.
//!   The recorded method is `interactionType || (platform.os.ios ? 'touch' : '')`
//!   (`:17-24`): on iOS Safari the hitslop around touch targets means a tap outside the
//!   element's bounds fires `mousedown` without `pointerdown`, leaving `interactionType`
//!   `''` — the implementation spec's "iOS fallback" row
//!   (`specs/library/utils/implementation.md`, "Interaction-type & click-semantics hooks").
//! - `useOpenInteractionType(open)` (`:44-62`) — the `InteractionType | null` state
//!   (`:45`), the trigger bag from the first hook (`:47`), and a `useValueChanged` reset
//!   to `null` on every open → close transition (`:49-53`).
//!
//! ## Rust adaptations
//!
//! - The `InteractionType` union is the `useEnhancedClickHandler` port's enum
//!   ([`leptos_ui_utils::use_enhanced_click_handler::InteractionType`]); upstream's
//!   falsy-`''` check (`:18`) is the [`InteractionType::Unknown`] sentinel comparison —
//!   the port's normalization of every non-DOM `pointerType` string, documented there.
//!   The `||`-fallback and the reset guard are carried as the pure
//!   [`resolve_open_method`] / [`should_reset_open_method`] helpers so their matrices are
//!   host-testable (the full click flow needs real DOM events and lives in the wasm
//!   suite).
//! - The open state is a reactive source read untracked inside the handler — upstream's
//!   `typeof open === 'function' ? open() : open` (`:14`) reads the latest render's value
//!   at dispatch time (the `use_button` port's `disabled` convention).
//! - `useStableCallback` (`:12`) is N/A: the handler is bound once per hook call, so its
//!   identity is stable by construction (the `useValueChanged` port's documented reasoning).
//! - The `React.useState`/`setOpenMethod` pair ports to an internal
//!   [`reactive_graph::signal::RwSignal`]; the setter travels into
//!   [`use_open_method_trigger_props`] as a plain `Fn(Option<InteractionType>)` slot, so
//!   Phase B components that track the method in their own store can pass a writer
//!   closure instead (upstream passes the raw `Dispatch<SetStateAction<…>>`).
//! - `useValueChanged(open, …)` (`:49-53`) is the [`crate::use_value_changed`] port; the
//!   callback's current-open read runs inside its untracked section, matching the
//!   dep-array read upstream.
//! - The returned `openMethod` is a read-only [`Signal`] — upstream exposes the state
//!   value through the `useMemo` return (`:55-61`), never the setter.
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.
//! - No upstream test covers this module (`specs/library/utils/implementation.md`,
//!   "Anything in source not explained by any test": the iOS fallback and the
//!   reset-on-close are "only observable via components' `openMethod`-driven styling") —
//!   the suites below pin the written mechanics proportionately.

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use web_sys::MouseEvent;

use leptos_ui_utils::platform::platform;
use leptos_ui_utils::use_enhanced_click_handler::{
    EnhancedClickHandlers, InteractionType, use_enhanced_click_handler,
};

use crate::use_value_changed::use_value_changed;

/// The `interactionType || (platform.os.ios ? 'touch' : '')` fallback
/// (`useOpenInteractionType.ts:17-24`): a resolved interaction type passes through, and
/// the falsy [`InteractionType::Unknown`] sentinel (upstream `''`) falls back to
/// [`InteractionType::Touch`] on iOS — the hitslop `mousedown`-without-`pointerdown`
/// case — or stays [`InteractionType::Unknown`] elsewhere.
fn resolve_open_method(interaction_type: InteractionType, ios: bool) -> InteractionType {
    if interaction_type != InteractionType::Unknown {
        interaction_type
    } else if ios {
        InteractionType::Touch
    } else {
        InteractionType::Unknown
    }
}

/// The `useValueChanged` guard (`useOpenInteractionType.ts:50`): the recorded method
/// resets exactly on an open → close transition.
fn should_reset_open_method(previous_open: bool, open: bool) -> bool {
    previous_open && !open
}

/// Port of `useOpenMethodTriggerProps` (`useOpenInteractionType.ts:8-37`): the
/// `{ onClick, onPointerDown }` trigger bag whose click records the interaction type
/// that opened the component, gated on the live open state.
///
/// `set_open_method` is upstream's `setOpenMethod` state setter (`:10`); it receives
/// `Some(method)` on every open-click and is never called with `None` by this hook.
pub fn use_open_method_trigger_props<O, S>(open: O, set_open_method: S) -> EnhancedClickHandlers
where
    O: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    S: Fn(Option<InteractionType>) + 'static,
{
    // `handleTriggerClick` (`:12-26`), minus the `useStableCallback` wrapper (see the
    // module docs).
    let handle_trigger_click = move |_: &MouseEvent, interaction_type: InteractionType| {
        // `const isOpen = typeof open === 'function' ? open() : open` (`:14`) — the
        // reactive read stands in for the live getter.
        if !open.get_untracked() {
            set_open_method(Some(resolve_open_method(
                interaction_type,
                // `platform.os.ios` (`:22`).
                platform().os.ios,
            )));
        }
    };

    // `const { onClick, onPointerDown } = useEnhancedClickHandler(handleTriggerClick)`
    // (`:28`); the `useMemo` bag (`:30-36`) is the returned value itself — the port's
    // handlers are bound once, so the memo's identity concern has no Rust shape.
    use_enhanced_click_handler(handle_trigger_click)
}

/// `useOpenInteractionType`'s return object (`useOpenInteractionType.ts:55-61`).
pub struct UseOpenInteractionTypeReturnValue {
    /// `openMethod` (`:57`): the interaction type that opened the component; `None` is
    /// upstream's `null` (the initial value, `:45`, and the post-close reset, `:51`).
    pub open_method: Signal<Option<InteractionType>>,
    /// `triggerProps` (`:58`): the bag from [`use_open_method_trigger_props`].
    pub trigger_props: EnhancedClickHandlers,
}

/// Port of `useOpenInteractionType` (`useOpenInteractionType.ts:44-62`). Must be called
/// inside a reactive owner (the reset effect registers there).
pub fn use_open_interaction_type<O>(open: O) -> UseOpenInteractionTypeReturnValue
where
    O: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    // `const [openMethod, setOpenMethod] = React.useState<InteractionType | null>(null)`
    // (`:45`).
    let open_method: RwSignal<Option<InteractionType>> = RwSignal::new(None);

    // `const triggerProps = useOpenMethodTriggerProps(open, setOpenMethod)` (`:47`).
    let trigger_props =
        use_open_method_trigger_props(open.clone(), move |method| open_method.set(method));

    // `useValueChanged(open, (previousOpen) => { if (previousOpen && !open)
    // setOpenMethod(null); })` (`:49-53`) — every open → close transition clears the
    // recorded method. The current-open read is the same source the effect tracks (the
    // callback runs untracked, matching the dep-array read upstream).
    {
        let open_now = open.clone();
        use_value_changed(open, move |previous_open: bool| {
            if should_reset_open_method(previous_open, open_now.get_untracked()) {
                open_method.set(None);
            }
        });
    }

    UseOpenInteractionTypeReturnValue {
        open_method: open_method.into(),
        trigger_props,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // The `||`-fallback matrix (`useOpenInteractionType.ts:17-24`): a resolved type
    // passes through untouched; the `''` sentinel falls back per platform. No upstream
    // test exists (implementation spec, "Anything in source not explained by any test")
    // — this pins the documented fallback.
    #[test]
    fn the_ios_fallback_resolves_the_empty_sentinel_only() {
        // Resolved types pass through regardless of platform.
        assert_eq!(
            resolve_open_method(InteractionType::Mouse, false),
            InteractionType::Mouse
        );
        assert_eq!(
            resolve_open_method(InteractionType::Touch, true),
            InteractionType::Touch
        );
        assert_eq!(
            resolve_open_method(InteractionType::Pen, false),
            InteractionType::Pen
        );
        assert_eq!(
            resolve_open_method(InteractionType::Keyboard, true),
            InteractionType::Keyboard
        );
        // The `''` sentinel: `'touch'` on iOS, `''` elsewhere.
        assert_eq!(
            resolve_open_method(InteractionType::Unknown, true),
            InteractionType::Touch
        );
        assert_eq!(
            resolve_open_method(InteractionType::Unknown, false),
            InteractionType::Unknown
        );
    }

    // The reset guard (`:50`) — exactly the open → close transitions, both directions
    // pinned. (The wiring that applies it is the `use_value_changed` port's own covered
    // contract; the recorded-method flow through real clicks lives in the wasm suite.)
    #[test]
    fn only_an_open_to_close_transition_resets() {
        assert!(should_reset_open_method(true, false), "open → close resets");
        assert!(
            !should_reset_open_method(false, true),
            "close → open does not reset"
        );
        assert!(
            !should_reset_open_method(false, false),
            "staying closed does not reset"
        );
        assert!(
            !should_reset_open_method(true, true),
            "staying open does not reset"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{PointerEvent, PointerEventInit};

    use super::*;
    use leptos_ui_utils::add_event_listener;
    use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect("no window")
            .document()
            .expect("no document")
    }

    /// Attaches the trigger bag to a real element — the JSX spread of `triggerProps`
    /// upstream (`useOpenInteractionType.ts:58` consumed at the trigger call sites).
    fn attach(handlers: EnhancedClickHandlers, element: &web_sys::Element) -> CleanupFn {
        let EnhancedClickHandlers {
            on_click,
            on_pointer_down,
        } = handlers;
        let click_unsubscribe = add_event_listener(element, "click", move |event| {
            if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                on_click(mouse_event);
            }
        });
        let pointer_down_unsubscribe = add_event_listener(element, "pointerdown", move |event| {
            if let Some(pointer_event) = event.dyn_ref::<PointerEvent>() {
                on_pointer_down(pointer_event);
            }
        });
        Box::new(merge_cleanups(vec![
            Some(Box::new(move || click_unsubscribe.unsubscribe()) as CleanupFn),
            Some(Box::new(move || pointer_down_unsubscribe.unsubscribe()) as CleanupFn),
        ])) as CleanupFn
    }

    fn mouse_click(detail: i32) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_detail(detail);
        MouseEvent::new_with_mouse_event_init_dict("click", &init).expect("MouseEvent failed")
    }

    fn pointer_click(pointer_type: &str) -> MouseEvent {
        let init = PointerEventInit::new();
        init.set_bubbles(true);
        init.set_detail(1);
        init.set_pointer_type(pointer_type);
        PointerEvent::new_with_event_init_dict("click", &init)
            .expect("PointerEvent failed")
            .unchecked_into::<MouseEvent>()
    }

    fn pointer_down(pointer_type: &str) -> PointerEvent {
        let init = PointerEventInit::new();
        init.set_bubbles(true);
        init.set_pointer_type(pointer_type);
        PointerEvent::new_with_event_init_dict("pointerdown", &init).expect("PointerEvent failed")
    }

    /// A Chrome-on-macOS test browser is not iOS: `platform().os.ios` is false, so the
    /// `''` fallback lands on `Unknown` — pinned so a platform-detection regression
    /// cannot silently flip the suite's expectations.
    #[wasm_bindgen_test]
    fn the_test_browser_is_not_ios() {
        assert!(!platform().os.ios);
    }

    // The full record flow: a pointer-type click records the live pointer type
    // (`useEnhancedClickHandler.ts:38-40`), a Safari-style MouseEvent click replays the
    // `''` sentinel (`:42`) which falls back to `Unknown` off-iOS (`:17-24`), and a
    // `detail: 0` click is the keyboard (`:33-36`).
    #[wasm_bindgen_test]
    fn clicks_record_the_open_method_until_a_close_transition_resets_it() {
        // The reset effect re-runs through the ambient executor (the `use_button`
        // wasm-suite note); re-initializing returns `Err`.
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let open: RwSignal<bool> = RwSignal::new(false);
        let UseOpenInteractionTypeReturnValue {
            open_method,
            trigger_props,
        } = use_open_interaction_type(open);
        let element = document().create_element("div").expect("create_element");
        document()
            .body()
            .expect("a body")
            .append_child(&element)
            .unwrap();
        let _cleanup = attach(trigger_props, &element);

        let method = || open_method.get_untracked();
        let fire = |event: &MouseEvent| {
            element
                .dispatch_event(event.as_ref())
                .expect("dispatch failed");
        };

        // `'' → Unknown` off-iOS (the Safari replay path).
        assert_eq!(method(), None);
        fire(&mouse_click(1));
        assert_eq!(method(), Some(InteractionType::Unknown));

        // A pointer-type click records the live type (`Chrome and Edge correctly use
        // PointerEvent`).
        fire(&pointer_click("mouse").unchecked_ref::<MouseEvent>());
        assert_eq!(method(), Some(InteractionType::Mouse));
        fire(&pointer_click("touch").unchecked_ref::<MouseEvent>());
        assert_eq!(method(), Some(InteractionType::Touch));

        // `event.detail === 0` is the keyboard (`useEnhancedClickHandler.ts:33-36`).
        fire(&mouse_click(0));
        assert_eq!(method(), Some(InteractionType::Keyboard));

        // The open gate: while open, clicks do not re-record (`:16`).
        open.set(true);
        any_spawner::Executor::poll_local();
        fire(&pointer_click("pen").unchecked_ref::<MouseEvent>());
        assert_eq!(
            method(),
            Some(InteractionType::Keyboard),
            "open clicks are gated off"
        );

        // The open → close transition resets to `null` (`:49-53`) — the reset is the
        // `use_value_changed` effect's re-run, scheduled on the ambient executor
        // (`use_press_and_hold`'s wasm-suite flush). Each transition flushes so the
        // previous-value ref commits the intermediate `true`.
        open.set(false);
        any_spawner::Executor::poll_local();
        assert_eq!(
            method(),
            None,
            "the open → close transition resets openMethod to null"
        );

        owner.cleanup();
    }

    // The pointerdown arm records too (`useEnhancedClickHandler.ts:18-28` — the handler
    // is invoked for both members of the pair), with the same open gate.
    #[wasm_bindgen_test]
    fn pointer_down_records_the_interaction_type_under_the_same_gate() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let open: RwSignal<bool> = RwSignal::new(false);
        let UseOpenInteractionTypeReturnValue {
            open_method,
            trigger_props,
        } = use_open_interaction_type(open);
        let element = document().create_element("div").expect("create_element");
        document()
            .body()
            .expect("a body")
            .append_child(&element)
            .unwrap();
        let _cleanup = attach(trigger_props, &element);

        assert_eq!(open_method.get_untracked(), None);
        element
            .dispatch_event(&pointer_down("touch"))
            .expect("dispatch failed");
        assert_eq!(
            open_method.get_untracked(),
            Some(InteractionType::Touch),
            "the pointerdown half records through the same handler"
        );

        // Open gate applies to the pointerdown half as well.
        open.set(true);
        element
            .dispatch_event(&pointer_down("mouse"))
            .expect("dispatch failed");
        assert_eq!(
            open_method.get_untracked(),
            Some(InteractionType::Touch),
            "open pointerdowns are gated off"
        );

        owner.cleanup();
    }
}
