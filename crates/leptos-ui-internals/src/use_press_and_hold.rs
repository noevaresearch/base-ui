//! Port of `packages/react/src/internals/usePressAndHold.ts` — the pointer-type-aware
//! hold-repeat machine behind the NumberField stepper button (the unit's only upstream
//! consumer, `packages/react/src/number-field/root/useNumberFieldStepper.ts`).
//!
//! On pointer down the hook performs one `tick` immediately, then — after `startDelay`
//! (default 400 ms, `usePressAndHold.ts:10-11`) — repeats it at `tickDelay` (default 60 ms)
//! until the tick returns `false` or the pointer is released
//! (`specs/library/internals/implementation.md`, "Small hooks", the `usePressAndHold`
//! bullet). The spec flags the whole hook as untested upstream ("Anything in source not
//! explained by any test", item 1: the touch-intent threshold, pen-as-touch handling,
//! global contextmenu suppression, `shouldSkipClick` semantics, and start/tick delays are
//! entirely unverified there), so the tests below pin the ported mechanics themselves —
//! the `useHoverFloatingInteraction` precedent.
//!
//! ## Rust adaptations
//!
//! - The local refs (`isPressedRef`/`movesAfterTouchRef`/`downCoordsRef`/
//!   `isTouchingButtonRef`/`ignoreClickRef`/`pointerTypeRef`,
//!   `usePressAndHold.ts:95-100`) become `Rc` `Cell`/`RefCell` handles shared between the
//!   timers, the effects, and the returned handler bag — the `useDismiss` adaptation.
//! - The unsubscribe-function refs (`unsubscribeFromGlobalContextMenuRef`/
//!   `unsubscribeFromGlobalPointerUpRef`, `usePressAndHold.ts:101-102`, `NOOP`-seeded)
//!   become `Rc<RefCell<Option<EventListenerUnsubscribe>>>`: `None` is the `NOOP` default,
//!   and unsubscribing takes the handle out (the owned-handle semantics of
//!   [`leptos_ui_utils::add_event_listener`] — dropping or explicitly unsubscribing both
//!   remove the listener, so leaving `None` behind is equivalent to upstream leaving the
//!   spent unsubscribe function in the slot).
//! - `startAutoChange` (`usePressAndHold.ts:112-162`) and `stopAutoChange`
//!   (`:104-110`, a `useStableCallback`) become shared `Rc` closures — the hook body runs
//!   once, so the stable-identity machinery matters only where upstream re-runs render
//!   bodies; `shouldSkipClick` keeps the [`use_stable_callback`] wrapper (`:275`) with the
//!   owned-`MouseEvent` argument the `'static` arg bound requires (web-sys event handles
//!   are reference-counted clones).
//! - The `pointerHandlers` object (`:181-273`) becomes a struct of
//!   `ElementEventHandler` closures. Upstream's handlers receive React synthetic events
//!   and forward `event.nativeEvent` into `startAutoChange`/`tick`; the port's handlers
//!   receive the DOM events directly, so the native event is the event itself.
//! - `disabled` is a reactive source ([`Get`] + [`GetUntracked`] + `Clone`): upstream's
//!   handlers read the latest render's value and the `disabled` effect (`:172-179`) reacts
//!   to changes, so the port reads it untracked in the handlers and tracks it inside a
//!   [`reactive_graph::effect::Effect`] standing in for that effect. The unmount-only
//!   effect (`:164-170`) folds into an [`on_cleanup`] registration — its deps are stable,
//!   so the cleanup is the only observable part.
//! - Timer scheduling goes through the crate's [`use_timeout`]/[`use_interval`] handles
//!   (`:91-93`) — upstream's `useTimeout`/`useInterval` hook pair — and the global
//!   listeners register through [`add_event_listener`]/[`add_event_listener_with_options`]
//!   (the `{ once: true }` options object at `:147` maps to
//!   [`AddEventListenerOptions::set_once`]).
//! - The `elementRef` param (`:51`) is the port's element-source resolver read at
//!   `startAutoChange` time (`:115-118`), the `useAnimationsFinished` convention for
//!   upstream's `React.RefObject` read.
//! - `'use client'` (`usePressAndHold.ts:1`) is N/A — there is no React Server Components
//!   boundary in Rust.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::effect::Effect;
use reactive_graph::owner::on_cleanup;
use reactive_graph::traits::{Get, GetUntracked};
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, Event, HtmlElement, MouseEvent, PointerEvent, TouchEvent};

use crate::floating_ui::element_props::ElementEventHandler;
use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::add_event_listener::{add_event_listener, add_event_listener_with_options};
use leptos_ui_utils::owner::owner_window;
use leptos_ui_utils::use_interval::use_interval;
use leptos_ui_utils::use_stable_callback::{StableCallback, use_stable_callback};
use leptos_ui_utils::use_timeout::use_timeout;

/// `TOUCH_TIMEOUT` (`usePressAndHold.ts:13`) — the intentional-touch check window.
const TOUCH_TIMEOUT: u32 = 50;
/// `MAX_POINTER_MOVES_AFTER_TOUCH` (`usePressAndHold.ts:14`) — more moves than this within
/// the check window is a scroll/pinch gesture, not a press.
const MAX_POINTER_MOVES_AFTER_TOUCH: u32 = 3;

/// `isTouchLikePointerType` (`usePressAndHold.ts:19-21`): pen is treated as touch-like to
/// avoid forcing the software keyboard on stylus taps (the Linux-Chrome note in the
/// upstream comment).
pub fn is_touch_like_pointer_type(pointer_type: &str) -> bool {
    pointer_type == "touch" || pointer_type == "pen"
}

/// The `tick` parameter (`usePressAndHold.ts:28`): called on each tick during a hold with
/// the triggering native event, returning `false` to stop the auto-change sequence.
pub type PressAndHoldTick = Rc<dyn Fn(Option<&Event>) -> bool>;

/// The `onStop` parameter (`usePressAndHold.ts:32`): called when the hold ends via the
/// global `pointerup` event.
pub type PressAndHoldOnStop = Rc<dyn Fn(&PointerEvent)>;

/// The shared `startAutoChange` closure (`usePressAndHold.ts:112-162`).
type StartAutoChange = Rc<dyn Fn(Option<&Event>)>;

/// `UsePressAndHoldParameters` (`usePressAndHold.ts:23-52`). Upstream's destructuring
/// defaults are documented per field; callers without a reactive `disabled` pass
/// `RwSignal::new(false)` (the `UseOpenChangeCompleteParams` convention).
pub struct UsePressAndHoldParams<D, E> {
    /// `disabled` (`:24` — upstream default `false`): a reactive source; `true` stops any
    /// running sequence and makes the handlers inert.
    pub disabled: D,
    /// `tick` (`:28`): called once immediately on press and repeatedly while held;
    /// returning `false` stops the sequence.
    pub tick: PressAndHoldTick,
    /// `onStop` (`:32`): called when the hold ends via the global `pointerup` event.
    pub on_stop: Option<PressAndHoldOnStop>,
    /// `tickDelay` (`:37` — upstream default `60`).
    pub tick_delay: u32,
    /// `startDelay` (`:42` — upstream default `400`).
    pub start_delay: u32,
    /// `scrollDistance` (`:47` — upstream default `8`): pointer movement distance (px)
    /// that cancels the hold, treated as scrolling.
    pub scroll_distance: f64,
    /// `elementRef` (`:51`): the anchor element used to resolve `ownerWindow` for the
    /// global listeners, read per [`use_press_and_hold`] invocation of `startAutoChange`.
    pub element_ref: E,
}

/// The `pointerHandlers` object (`usePressAndHold.ts:181-273`) — the handlers the consumer
/// spreads onto the element.
pub struct PressAndHoldPointerHandlers {
    /// `onTouchStart` (`:182-184`).
    pub on_touch_start: ElementEventHandler<TouchEvent>,
    /// `onTouchEnd` (`:185-187`).
    pub on_touch_end: ElementEventHandler<TouchEvent>,
    /// `onPointerDown` (`:188-223`).
    pub on_pointer_down: ElementEventHandler<PointerEvent>,
    /// `onPointerUp` (`:224-230`).
    pub on_pointer_up: ElementEventHandler<PointerEvent>,
    /// `onPointerMove` (`:231-245`).
    pub on_pointer_move: ElementEventHandler<PointerEvent>,
    /// `onMouseEnter` (`:246-258`).
    pub on_mouse_enter: ElementEventHandler<MouseEvent>,
    /// `onMouseLeave` (`:259-265`).
    pub on_mouse_leave: ElementEventHandler<MouseEvent>,
    /// `onMouseUp` (`:266-272`).
    pub on_mouse_up: ElementEventHandler<MouseEvent>,
}

/// `UsePressAndHoldReturnValue` (`usePressAndHold.ts:54-72`).
pub struct UsePressAndHoldReturnValue {
    /// `pointerHandlers` (`:55-64`).
    pub pointer_handlers: PressAndHoldPointerHandlers,
    /// `shouldSkipClick` (`:71`): returns `true` if the element's `onClick` should be
    /// skipped — preventing double-firing on mouse clicks (already handled by
    /// `onPointerDown`) and suppressing the synthesized click after a touch hold. Called
    /// with the owned event (the `'static` arg bound of [`StableCallback`]).
    pub should_skip_click: StableCallback<MouseEvent, bool>,
}

/// `usePressAndHold` (`usePressAndHold.ts:80-286`). Must be called inside a reactive owner
/// (the timers and the effects register cleanups).
pub fn use_press_and_hold<D, E>(params: UsePressAndHoldParams<D, E>) -> UsePressAndHoldReturnValue
where
    D: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    E: Fn() -> Option<HtmlElement> + 'static,
{
    let UsePressAndHoldParams {
        disabled,
        tick,
        on_stop,
        tick_delay,
        start_delay,
        scroll_distance,
        element_ref,
    } = params;

    let element_ref: Rc<dyn Fn() -> Option<HtmlElement>> = Rc::new(element_ref);

    // `const startTickTimeout = useTimeout(); const tickInterval = useInterval();
    // const intentionalTouchCheckTimeout = useTimeout();` (`:91-93`).
    let start_tick_timeout = use_timeout();
    let tick_interval = use_interval();
    let intentional_touch_check_timeout = use_timeout();

    // The refs (`:95-102`).
    let is_pressed_ref = Rc::new(Cell::new(false));
    let moves_after_touch_ref = Rc::new(Cell::new(0u32));
    let down_coords_ref = Rc::new(Cell::new((0.0f64, 0.0f64)));
    let is_touching_button_ref = Rc::new(Cell::new(false));
    let ignore_click_ref = Rc::new(Cell::new(false));
    let pointer_type_ref = Rc::new(RefCell::new(String::new()));
    let unsubscribe_from_global_context_menu_ref: Rc<RefCell<Option<EventListenerUnsubscribe>>> =
        Rc::new(RefCell::new(None));
    let unsubscribe_from_global_pointer_up_ref: Rc<RefCell<Option<EventListenerUnsubscribe>>> =
        Rc::new(RefCell::new(None));

    // `const stopAutoChange = useStableCallback(...)` (`:104-110`): clears the three
    // timers, unsubscribes the global context-menu listener, and resets the move counter.
    // The global pointerup listener deliberately stays registered (the comment at
    // `:134-137`) so a hold that auto-stops at a boundary still fires `onStop` on release.
    let stop_auto_change: StableCallback<(), ()> = {
        let intentional_touch_check_timeout = intentional_touch_check_timeout.clone();
        let start_tick_timeout = start_tick_timeout.clone();
        let tick_interval = tick_interval.clone();
        let context_menu_ref = Rc::clone(&unsubscribe_from_global_context_menu_ref);
        let moves_after_touch_ref = Rc::clone(&moves_after_touch_ref);
        use_stable_callback::<(), (), _>(Some(move |()| {
            intentional_touch_check_timeout.clear();
            start_tick_timeout.clear();
            tick_interval.clear();
            if let Some(handle) = context_menu_ref.borrow_mut().take() {
                handle.unsubscribe();
            }
            moves_after_touch_ref.set(0);
        }))
    };

    // `startAutoChange` (`:112-162`).
    let start_auto_change: StartAutoChange = {
        let stop_auto_change = stop_auto_change.clone();
        let element_ref = Rc::clone(&element_ref);
        let context_menu_ref = Rc::clone(&unsubscribe_from_global_context_menu_ref);
        let pointer_up_ref = Rc::clone(&unsubscribe_from_global_pointer_up_ref);
        let is_pressed_ref = Rc::clone(&is_pressed_ref);
        let on_stop = on_stop.clone();
        let tick = Rc::clone(&tick);
        let start_tick_timeout = start_tick_timeout.clone();
        let tick_interval = tick_interval.clone();
        Rc::new(move |trigger_native_event: Option<&Event>| {
            stop_auto_change.call(());

            let Some(element) = element_ref() else {
                return;
            };
            let win = owner_window(Some(element.as_ref() as &web_sys::Node));

            // A global context menu listener is necessary to prevent the context menu
            // from appearing when the touch is slightly outside of the element's hit
            // area (`:126-132`).
            let handle =
                add_event_listener(&win, "contextmenu", |event: &Event| event.prevent_default());
            *context_menu_ref.borrow_mut() = Some(handle);

            // The release listener: replace any existing one first so a
            // mouseleave/mouseenter cycle during a hold doesn't stack listeners
            // (`:134-148`).
            if let Some(existing) = pointer_up_ref.borrow_mut().take() {
                existing.unsubscribe();
            }
            let listener_is_pressed_ref = Rc::clone(&is_pressed_ref);
            let listener_stop_auto_change = stop_auto_change.clone();
            let listener_on_stop = on_stop.clone();
            let once = AddEventListenerOptions::new();
            once.set_once(true);
            let pointer_up_handle = add_event_listener_with_options(
                &win,
                "pointerup",
                move |event: &Event| {
                    listener_is_pressed_ref.set(false);
                    listener_stop_auto_change.call(());
                    if let Some(on_stop) = &listener_on_stop {
                        on_stop(event.unchecked_ref::<PointerEvent>());
                    }
                },
                once,
            );
            *pointer_up_ref.borrow_mut() = Some(pointer_up_handle);

            // The immediate tick; a `false` return stops the sequence before any timer
            // is scheduled (`:150-153`).
            if !tick(trigger_native_event) {
                stop_auto_change.call(());
                return;
            }

            let tick_interval = tick_interval.clone();
            let tick_for_interval = Rc::clone(&tick);
            let stop_for_interval = stop_auto_change.clone();
            let trigger = trigger_native_event.cloned();
            start_tick_timeout.start(start_delay, move || {
                tick_interval.start(tick_delay, move || {
                    if !tick_for_interval(trigger.as_ref()) {
                        stop_for_interval.call(());
                    }
                });
            });
        })
    };

    // The unmount-only effect (`:164-170`): its deps are the stable `stopAutoChange`, so
    // the cleanup is the only observable part.
    {
        let stop_auto_change = stop_auto_change.clone();
        let pointer_up_ref = Rc::clone(&unsubscribe_from_global_pointer_up_ref);
        let cleanup = SendWrapper::new(move || {
            stop_auto_change.call(());
            if let Some(handle) = pointer_up_ref.borrow_mut().take() {
                handle.unsubscribe();
            }
        });
        on_cleanup(move || (*cleanup)());
    }

    // The `disabled` effect (`:172-179`): disabling resets the press state and stops any
    // running sequence.
    {
        let disabled = disabled.clone();
        let is_pressed_ref = Rc::clone(&is_pressed_ref);
        let is_touching_button_ref = Rc::clone(&is_touching_button_ref);
        let pointer_type_ref = Rc::clone(&pointer_type_ref);
        let stop_auto_change = stop_auto_change.clone();
        Effect::new(move |_| {
            if disabled.get() {
                is_pressed_ref.set(false);
                is_touching_button_ref.set(false);
                pointer_type_ref.borrow_mut().clear();
                stop_auto_change.call(());
            }
        });
    }

    let pointer_handlers = PressAndHoldPointerHandlers {
        on_touch_start: {
            let is_touching_button_ref = Rc::clone(&is_touching_button_ref);
            Rc::new(move |_event: &TouchEvent| {
                is_touching_button_ref.set(true);
            })
        },
        on_touch_end: {
            let is_touching_button_ref = Rc::clone(&is_touching_button_ref);
            Rc::new(move |_event: &TouchEvent| {
                is_touching_button_ref.set(false);
            })
        },
        on_pointer_down: {
            let disabled = disabled.clone();
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            let ignore_click_ref = Rc::clone(&ignore_click_ref);
            let is_pressed_ref = Rc::clone(&is_pressed_ref);
            let down_coords_ref = Rc::clone(&down_coords_ref);
            let moves_after_touch_ref = Rc::clone(&moves_after_touch_ref);
            let start_auto_change = Rc::clone(&start_auto_change);
            let stop_auto_change = stop_auto_change.clone();
            let intentional_touch_check_timeout = intentional_touch_check_timeout.clone();
            Rc::new(move |event: &PointerEvent| {
                if event.default_prevented() || event.button() != 0 || disabled.get_untracked() {
                    return;
                }

                *pointer_type_ref.borrow_mut() = event.pointer_type();
                ignore_click_ref.set(false);
                is_pressed_ref.set(true);
                down_coords_ref.set((f64::from(event.client_x()), f64::from(event.client_y())));

                let is_touch_pointer = is_touch_like_pointer_type(&event.pointer_type());

                if !is_touch_pointer {
                    event.prevent_default();
                    start_auto_change(Some(event.as_ref() as &Event));
                } else {
                    // Check if the pointerdown was intentional and not the result of a
                    // scroll or pinch-zoom (`:204-221`). The `stillPressed` guard
                    // prevents races with pointerup occurring before the timeout fires
                    // on quick taps.
                    let trigger = event.clone().unchecked_into::<Event>();
                    let moves_after_touch_ref = Rc::clone(&moves_after_touch_ref);
                    let is_pressed_ref = Rc::clone(&is_pressed_ref);
                    let ignore_click_ref = Rc::clone(&ignore_click_ref);
                    let start_auto_change = Rc::clone(&start_auto_change);
                    let stop_auto_change = stop_auto_change.clone();
                    intentional_touch_check_timeout.start(TOUCH_TIMEOUT, move || {
                        let moves = moves_after_touch_ref.get();
                        moves_after_touch_ref.set(0);
                        let still_pressed = is_pressed_ref.get();
                        if still_pressed && moves < MAX_POINTER_MOVES_AFTER_TOUCH {
                            start_auto_change(Some(trigger.as_ref()));
                            // Synthesized click after hold should be ignored.
                            ignore_click_ref.set(true);
                        } else {
                            // No auto-change (simple tap or scroll gesture), allow the
                            // click handler to perform a single action.
                            ignore_click_ref.set(false);
                            stop_auto_change.call(());
                        }
                    });
                }
            })
        },
        on_pointer_up: {
            let is_pressed_ref = Rc::clone(&is_pressed_ref);
            Rc::new(move |event: &PointerEvent| {
                // Ensure we mark the press as released for touch flows even if
                // auto-change never started, so the delayed auto-change check won't
                // start after a quick tap (`:224-230`).
                if is_touch_like_pointer_type(&event.pointer_type()) {
                    is_pressed_ref.set(false);
                }
            })
        },
        on_pointer_move: {
            let disabled = disabled.clone();
            let is_pressed_ref = Rc::clone(&is_pressed_ref);
            let moves_after_touch_ref = Rc::clone(&moves_after_touch_ref);
            let down_coords_ref = Rc::clone(&down_coords_ref);
            let stop_auto_change = stop_auto_change.clone();
            Rc::new(move |event: &PointerEvent| {
                if disabled.get_untracked()
                    || !is_touch_like_pointer_type(&event.pointer_type())
                    || !is_pressed_ref.get()
                {
                    return;
                }

                moves_after_touch_ref.set(moves_after_touch_ref.get() + 1);

                let (x, y) = down_coords_ref.get();
                let dx = x - f64::from(event.client_x());
                let dy = y - f64::from(event.client_y());

                // `dx ** 2 + dy ** 2 > scrollDistance ** 2` (`:242`).
                if dx * dx + dy * dy > scroll_distance * scroll_distance {
                    stop_auto_change.call(());
                }
            })
        },
        on_mouse_enter: {
            let disabled = disabled.clone();
            let is_pressed_ref = Rc::clone(&is_pressed_ref);
            let is_touching_button_ref = Rc::clone(&is_touching_button_ref);
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            let start_auto_change = Rc::clone(&start_auto_change);
            Rc::new(move |event: &MouseEvent| {
                if event.default_prevented()
                    || disabled.get_untracked()
                    || !is_pressed_ref.get()
                    || is_touching_button_ref.get()
                    || is_touch_like_pointer_type(&pointer_type_ref.borrow())
                {
                    return;
                }

                start_auto_change(Some(event.as_ref() as &Event));
            })
        },
        on_mouse_leave: {
            let is_touching_button_ref = Rc::clone(&is_touching_button_ref);
            let stop_auto_change = stop_auto_change.clone();
            Rc::new(move |_event: &MouseEvent| {
                if is_touching_button_ref.get() {
                    return;
                }

                stop_auto_change.call(());
            })
        },
        on_mouse_up: {
            let is_touching_button_ref = Rc::clone(&is_touching_button_ref);
            let stop_auto_change = stop_auto_change.clone();
            Rc::new(move |_event: &MouseEvent| {
                if is_touching_button_ref.get() {
                    return;
                }

                stop_auto_change.call(());
            })
        },
    };

    // `shouldSkipClick` (`:275-283`).
    let should_skip_click: StableCallback<MouseEvent, bool> = {
        let pointer_type_ref = Rc::clone(&pointer_type_ref);
        let ignore_click_ref = Rc::clone(&ignore_click_ref);
        use_stable_callback::<MouseEvent, bool, _>(Some(move |event: MouseEvent| {
            if event.default_prevented() {
                return true;
            }
            if is_touch_like_pointer_type(&pointer_type_ref.borrow()) {
                return ignore_click_ref.get();
            }
            event.detail() != 0
        }))
    };

    UsePressAndHoldReturnValue {
        pointer_handlers,
        should_skip_click,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins `isTouchLikePointerType` (`usePressAndHold.ts:19-21`): pen is treated as
    // touch-like (the stylus/software-keyboard note in the upstream comment) while mouse
    // and the empty default of `pointerTypeRef` are not.
    #[test]
    fn is_touch_like_pointer_type_matrix() {
        assert!(is_touch_like_pointer_type("touch"));
        assert!(is_touch_like_pointer_type("pen"));
        assert!(!is_touch_like_pointer_type("mouse"));
        assert!(!is_touch_like_pointer_type(""));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use js_sys::Promise;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Event, EventTarget, HtmlElement, MouseEvent, PointerEvent, TouchEvent, Window};

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// The hook has no upstream test file (`specs/library/internals/implementation.md`,
    /// "Anything in source not explained by any test", item 1), so this suite pins the
    /// written mechanics with short real-timer delays (`startDelay` 40 ms, `tickDelay`
    /// 30 ms) in the browser — the `useHoverFloatingInteraction` precedent.
    struct Harness {
        /// Keeps the reactive owner (the hook's cleanups and effects) alive for the whole
        /// test — never read directly.
        #[allow(dead_code)]
        owner: Owner,
        handlers: PressAndHoldPointerHandlers,
        should_skip_click: StableCallback<MouseEvent, bool>,
        /// One entry per `tick` call: the trigger event's type, `None` when the hook was
        /// reached without one.
        ticks: Rc<RefCell<Vec<Option<String>>>>,
        /// One entry per `onStop` call: the release event's type.
        stops: Rc<RefCell<Vec<String>>>,
        disabled: RwSignal<bool>,
        tick_delay: u32,
        start_delay: u32,
        /// The `tick` returns `false` from this call number on — `u32::MAX` keeps every
        /// tick returning `true`.
        stop_after_tick: Rc<Cell<u32>>,
        element: HtmlElement,
    }

    impl Harness {
        fn new() -> Self {
            // The effects run through the ambient executor (the `use_media_query.rs`
            // wasm-suite note); re-initializing returns `Err`.
            let _ = any_spawner::Executor::init_futures_executor();
            let owner = Owner::new();
            owner.set();

            let document = web_sys::window()
                .expect("no window")
                .document()
                .expect("no document");
            let element: HtmlElement = document
                .create_element("div")
                .expect("create_element failed")
                .dyn_into()
                .expect("a div is an HtmlElement");
            document
                .body()
                .expect("a body")
                .append_child(&element)
                .unwrap();

            let ticks: Rc<RefCell<Vec<Option<String>>>> = Rc::new(RefCell::new(Vec::new()));
            let stops: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
            let tick_calls = Rc::new(Cell::new(0u32));
            let stop_after_tick = Rc::new(Cell::new(u32::MAX));

            let ticks_for_tick = Rc::clone(&ticks);
            let calls_for_tick = Rc::clone(&tick_calls);
            let stop_after_for_tick = Rc::clone(&stop_after_tick);
            let tick: PressAndHoldTick = Rc::new(move |trigger: Option<&Event>| {
                ticks_for_tick
                    .borrow_mut()
                    .push(trigger.map(|event| event.type_()));
                calls_for_tick.set(calls_for_tick.get() + 1);
                calls_for_tick.get() <= stop_after_for_tick.get()
            });

            let stops_for_stop = Rc::clone(&stops);
            let on_stop: PressAndHoldOnStop = Rc::new(move |event: &PointerEvent| {
                stops_for_stop.borrow_mut().push(event.type_());
            });

            let element_for_source = element.clone();
            let disabled = RwSignal::new(false);
            let return_value = use_press_and_hold(UsePressAndHoldParams {
                disabled: disabled.clone(),
                tick,
                on_stop: Some(on_stop),
                tick_delay: 30,
                start_delay: 40,
                scroll_distance: 8.0,
                element_ref: move || Some(element_for_source.clone()),
            });

            Self {
                owner,
                handlers: return_value.pointer_handlers,
                should_skip_click: return_value.should_skip_click,
                ticks,
                stops,
                disabled,
                tick_delay: 30,
                start_delay: 40,
                stop_after_tick,
                element,
            }
        }

        fn tick_count(&self) -> usize {
            self.ticks.borrow().len()
        }

        fn stop_count(&self) -> usize {
            self.stops.borrow().len()
        }

        fn window(&self) -> Window {
            web_sys::window().expect("no window")
        }

        /// Dispatches on the window — the global `pointerup`/`contextmenu` listeners the
        /// hook registers there are the consumers.
        fn dispatch_on_window(&self, event: &Event) -> bool {
            self.window()
                .dispatch_event(event)
                .expect("dispatch failed")
        }

        /// A pointerdown whose default is prevented by a temporary capture listener on
        /// the element — the `event.defaultPrevented` guard's input (`usePressAndHold.ts:189`).
        fn prevented_pointer_down(&self) -> PointerEvent {
            let event = pointer_event("pointerdown", "mouse", 0, 0, 0);
            let preventer: Closure<dyn Fn(Event)> = Closure::new(|event: Event| {
                event.prevent_default();
            });
            let target: &EventTarget = self.element.as_ref();
            target
                .add_event_listener_with_callback("pointerdown", preventer.as_ref().unchecked_ref())
                .expect("addEventListener failed");
            target.dispatch_event(&event).expect("dispatch failed");
            target
                .remove_event_listener_with_callback(
                    "pointerdown",
                    preventer.as_ref().unchecked_ref(),
                )
                .expect("removeEventListener failed");
            assert!(event.default_prevented(), "the preventer must have run");
            event
        }
    }

    impl Drop for Harness {
        fn drop(&mut self) {
            // The wasm-bindgen-test harness renders into the body, so fixtures remove
            // their top-level elements on drop (the `mark_others` harness wall).
            let _ = self.element.remove();
        }
    }

    fn sleep(ms: i32) -> Promise {
        Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .expect("no window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .expect("setTimeout failed");
        })
    }

    fn pointer_event(
        type_: &str,
        pointer_type: &str,
        button: i16,
        client_x: i32,
        client_y: i32,
    ) -> PointerEvent {
        let init = web_sys::PointerEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_pointer_type(pointer_type);
        init.set_button(button);
        init.set_client_x(client_x);
        init.set_client_y(client_y);
        PointerEvent::new_with_event_init_dict(type_, &init).expect("PointerEvent failed")
    }

    fn mouse_event(type_: &str, detail: i32) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_detail(detail);
        MouseEvent::new_with_mouse_event_init_dict(type_, &init).expect("MouseEvent failed")
    }

    fn touch_event(type_: &str) -> TouchEvent {
        TouchEvent::new(type_).expect("TouchEvent failed")
    }

    fn click(detail: i32) -> MouseEvent {
        mouse_event("click", detail)
    }

    // The happy path (`usePressAndHold.ts:188-223` mouse arm + `:112-162`): a mouse
    // pointerdown ticks immediately (with the triggering native event), the interval
    // starts after the start delay, and a release fires `onStop` exactly once and stops
    // the ticks.
    #[wasm_bindgen_test(async)]
    async fn a_mouse_press_ticks_immediately_then_repeats_and_release_fires_on_stop_once() {
        let harness = Harness::new();

        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0, 0, 0));
        assert_eq!(harness.tick_count(), 1, "the immediate tick on pointerdown");
        assert_eq!(
            harness.ticks.borrow()[0].as_deref(),
            Some("pointerdown"),
            "the tick receives the triggering native event"
        );

        JsFuture::from(sleep((harness.start_delay + 4 * harness.tick_delay) as i32))
            .await
            .expect("sleep failed");
        let before = harness.tick_count();
        assert!(
            before >= 2,
            "the repeating ticks started after the start delay"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "mouse", 0, 0, 0));
        assert_eq!(
            harness.stop_count(),
            1,
            "onStop fired exactly once on release"
        );

        JsFuture::from(sleep((2 * harness.tick_delay + 50) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(harness.tick_count(), before, "the ticks stopped on release");
        assert_eq!(harness.stop_count(), 1, "and onStop never re-fires");
    }

    // The auto-stop-at-a-boundary rule (`usePressAndHold.ts:150-153` and the
    // `:134-137` comment): a repeat tick returning `false` stops the sequence, but the
    // release listener deliberately stays registered through `stopAutoChange`, so `onStop`
    // still fires on release.
    #[wasm_bindgen_test(async)]
    async fn a_false_tick_stops_the_sequence_but_the_release_still_fires_on_stop() {
        let harness = Harness::new();
        harness.stop_after_tick.set(1);

        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0, 0, 0));
        assert_eq!(harness.tick_count(), 1, "the immediate tick returned true");

        JsFuture::from(sleep((harness.start_delay + 3 * harness.tick_delay) as i32))
            .await
            .expect("sleep failed");
        let before = harness.tick_count();
        assert!(
            before >= 2,
            "the second tick (returning false) ran from the interval"
        );

        JsFuture::from(sleep((2 * harness.tick_delay + 50) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            before,
            "the false return stopped the sequence"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "mouse", 0, 0, 0));
        assert_eq!(
            harness.stop_count(),
            1,
            "the release listener stayed registered through stopAutoChange"
        );
    }

    // The quick-tap guard (`usePressAndHold.ts:206-221` plus `:224-230`): a touch
    // pointerup before the intentional-touch check fires clears `isPressedRef`, so the
    // delayed check takes the no-auto-change branch and the click handler is allowed to
    // perform the single action.
    #[wasm_bindgen_test(async)]
    async fn a_quick_touch_tap_does_not_start_the_auto_change_and_allows_the_click() {
        let harness = Harness::new();

        (harness.handlers.on_touch_start)(&touch_event("touchstart"));
        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "touch", 0, 0, 0));
        (harness.handlers.on_pointer_up)(&pointer_event("pointerup", "touch", 0, 0, 0));
        (harness.handlers.on_touch_end)(&touch_event("touchend"));

        JsFuture::from(sleep((TOUCH_TIMEOUT + 60) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            0,
            "the released touch never starts the auto-change sequence"
        );
        assert!(
            !harness
                .should_skip_click
                .call(click(0))
                .expect("the stable callback"),
            "a simple tap allows the click handler"
        );
    }

    // The touch hold (`usePressAndHold.ts:206-221`): an intentional touch still pressed
    // when the check fires starts the sequence and marks the synthesized click for
    // skipping (`shouldSkipClick`, `:275-283` touch arm).
    #[wasm_bindgen_test(async)]
    async fn a_touch_hold_starts_the_sequence_and_skips_the_synthesized_click() {
        let harness = Harness::new();

        (harness.handlers.on_touch_start)(&touch_event("touchstart"));
        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "touch", 0, 0, 0));

        JsFuture::from(sleep((TOUCH_TIMEOUT + 60) as i32))
            .await
            .expect("sleep failed");
        assert!(
            harness.tick_count() >= 1,
            "the intentional touch started the auto-change sequence"
        );
        assert_eq!(
            harness.ticks.borrow()[0].as_deref(),
            Some("pointerdown"),
            "the delayed start still receives the triggering native event"
        );

        let event = click(1);
        assert!(
            harness
                .should_skip_click
                .call(event)
                .expect("the stable callback"),
            "the synthesized click after a touch hold is skipped"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "touch", 0, 0, 0));
        assert_eq!(harness.stop_count(), 1, "the release fires onStop");
    }

    // The scroll-distance cancel (`usePressAndHold.ts:231-245`): a touch move beyond the
    // configured distance cancels the running hold (treated as scrolling).
    #[wasm_bindgen_test(async)]
    async fn a_pointer_move_beyond_the_scroll_distance_cancels_the_hold() {
        let harness = Harness::new();

        (harness.handlers.on_touch_start)(&touch_event("touchstart"));
        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "touch", 0, 0, 0));
        JsFuture::from(sleep((TOUCH_TIMEOUT + 60) as i32))
            .await
            .expect("sleep failed");
        let before = harness.tick_count();
        assert!(before >= 1, "the hold started");

        // 20px down: `dx ** 2 + dy ** 2 = 400 > 8 ** 2` (`usePressAndHold.ts:242`).
        (harness.handlers.on_pointer_move)(&pointer_event("pointermove", "touch", 0, 0, 20));

        JsFuture::from(sleep((harness.tick_delay * 4 + 50) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            before,
            "the move beyond the scroll distance canceled the repeating ticks"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "touch", 0, 0, 0));
        assert_eq!(harness.stop_count(), 1);
    }

    // The leave/re-enter cycle during a hold (`usePressAndHold.ts:259-265` and the
    // `:134-137` comment): mouseleave stops the sequence, a mouseenter while still
    // pressed restarts it, and the release fires `onStop` exactly once — the replaced
    // `pointerup` listener never stacks. Also pins the mouse arm of `shouldSkipClick`
    // (`:282`): a real mouse click (`detail != 0`) is skipped, a `detail: 0` click is not.
    #[wasm_bindgen_test(async)]
    async fn a_leave_re_enter_cycle_restarts_the_hold_without_stacking_the_release_listener() {
        let harness = Harness::new();

        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0, 0, 0));
        (harness.handlers.on_mouse_leave)(&mouse_event("mouseleave", 0));
        JsFuture::from(sleep((harness.start_delay + 3 * harness.tick_delay) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            1,
            "the mouseleave stopped the sequence after the immediate tick"
        );

        (harness.handlers.on_mouse_enter)(&mouse_event("mouseenter", 0));
        assert_eq!(harness.tick_count(), 2, "the re-enter ticks immediately");
        assert_eq!(
            harness.ticks.borrow()[1].as_deref(),
            Some("mouseenter"),
            "the restart receives the mouseenter native event"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "mouse", 0, 0, 0));
        assert_eq!(
            harness.stop_count(),
            1,
            "the replaced release listener never stacks"
        );

        assert!(
            harness
                .should_skip_click
                .call(click(1))
                .expect("the stable callback"),
            "a real mouse click is skipped (the pointer already handled it)"
        );
        assert!(
            !harness
                .should_skip_click
                .call(click(0))
                .expect("the stable callback"),
            "a detail-0 click is not skipped on the mouse path"
        );
    }

    // The disabled policy (`usePressAndHold.ts:172-179` for the reactive effect and
    // `:189` for the guards): a disabled press is inert — the `defaultPrevented` and
    // `button` guards behave the same — and disabling mid-hold resets the state and stops
    // the running sequence.
    #[wasm_bindgen_test(async)]
    async fn disabling_is_inert_and_disabling_mid_hold_stops_the_sequence() {
        let harness = Harness::new();
        harness.disabled.set(true);
        any_spawner::Executor::poll_local();

        (harness.handlers.on_pointer_down)(&harness.prevented_pointer_down());
        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 2, 0, 0));
        JsFuture::from(sleep((harness.start_delay + 2 * harness.tick_delay) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            0,
            "a prevented or non-primary press is inert while disabled"
        );

        harness.disabled.set(false);
        any_spawner::Executor::poll_local();
        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0, 0, 0));
        assert_eq!(harness.tick_count(), 1);
        JsFuture::from(sleep(harness.start_delay as i32))
            .await
            .expect("sleep failed");

        harness.disabled.set(true);
        any_spawner::Executor::poll_local();
        let before = harness.tick_count();
        JsFuture::from(sleep((2 * harness.tick_delay + 50) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            before,
            "disabling mid-hold stops the repeating ticks"
        );

        (harness.handlers.on_mouse_enter)(&mouse_event("mouseenter", 0));
        JsFuture::from(sleep((harness.start_delay + 2 * harness.tick_delay) as i32))
            .await
            .expect("sleep failed");
        assert_eq!(
            harness.tick_count(),
            before,
            "the disabled reset cleared the pressed state, so a re-enter cannot restart"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "mouse", 0, 0, 0));
    }

    // The global context-menu suppressor (`usePressAndHold.ts:126-132` and the
    // `handleContextMenu` body at `:122-124`): while a hold is active, `contextmenu` on
    // the window is prevented (the slightly-outside-hit-area case); after release the
    // listener is gone.
    #[wasm_bindgen_test(async)]
    async fn the_global_context_menu_is_suppressed_while_holding_only() {
        let harness = Harness::new();

        (harness.handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0, 0, 0));
        assert!(
            !harness.dispatch_on_window(&mouse_event("contextmenu", 0)),
            "the context menu is prevented while holding"
        );

        harness.dispatch_on_window(&pointer_event("pointerup", "mouse", 0, 0, 0));
        assert!(
            harness.dispatch_on_window(&mouse_event("contextmenu", 0)),
            "the suppressor is unsubscribed once the hold ends"
        );
    }

    // The element-source bail (`usePressAndHold.ts:115-118`): without an element the
    // `startAutoChange` path stops before registering any global listener or timer — the
    // immediate tick never runs and the release is inert.
    #[wasm_bindgen_test(async)]
    async fn a_missing_element_bails_before_registering_anything() {
        let owner = Owner::new();
        owner.set();

        let ticks: Rc<RefCell<Vec<Option<String>>>> = Rc::new(RefCell::new(Vec::new()));
        let stops: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ticks_for_tick = Rc::clone(&ticks);
        let stops_for_stop = Rc::clone(&stops);
        let return_value = use_press_and_hold(UsePressAndHoldParams {
            disabled: RwSignal::new(false),
            tick: Rc::new(move |trigger: Option<&Event>| {
                ticks_for_tick.borrow_mut().push(trigger.map(|e| e.type_()));
                true
            }),
            on_stop: Some(Rc::new(move |event: &PointerEvent| {
                stops_for_stop.borrow_mut().push(event.type_());
            })),
            tick_delay: 30,
            start_delay: 40,
            scroll_distance: 8.0,
            element_ref: || None,
        });

        (return_value.pointer_handlers.on_pointer_down)(&pointer_event(
            "pointerdown",
            "mouse",
            0,
            0,
            0,
        ));
        assert!(
            ticks.borrow().is_empty(),
            "the missing element stops the sequence before the tick"
        );

        let window = web_sys::window().expect("no window");
        window
            .dispatch_event(&pointer_event("pointerup", "mouse", 0, 0, 0))
            .expect("dispatch failed");
        assert!(
            stops.borrow().is_empty(),
            "no global release listener was registered"
        );

        JsFuture::from(sleep((40 + 2 * 30) as i32))
            .await
            .expect("sleep failed");
        assert!(ticks.borrow().is_empty(), "and no timer ever ticked");
    }
}
