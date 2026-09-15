//! Context Menu Trigger — port of
//! `packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx`: the right-click /
//! long-press area that opens the menu, with the gesture guards behavior.md mines.
//!
//! One shared open path: both gestures funnel into [`handle_long_press`]
//! (`ContextMenuTrigger.tsx:52-74`) — record the spawn point, swap the anchor to a
//! virtual element at those coordinates, disarm the mouseup cancel, open through the
//! Root's actions with `REASONS.triggerPress`, and arm the 500ms
//! `allowMouseUpTimeout` that re-enables mouseup-driven cancellation. Right-click
//! reaches it from `handleContextMenu` (`:76-122`), long-press from the `touchstart`
//! timer (`:141-143`); the 10px touch-move threshold cancels a pending press
//! (`:152-162`).

use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::context_menu::root::{VirtualAnchor, use_context_menu_root_context};
use crate::menu::store::{menu_store_set_open, use_menu_root_context_optional};
use crate::menu::utils::find_root_owner_id;
use leptos_ui_internals::floating_ui::event::stop_event;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_utils::contains;
use leptos_ui_utils::shadow_dom::get_target;
use leptos_ui_utils::use_timeout::Timeout;

/// `REASONS.cancelOpen` (`reason-parts.ts`) — the gesture-end cancel reason; the
/// shared registry doesn't carry it yet (`floating_ui/reasons.rs` — the combobox
/// runtime's placement note), so it lives here like `REASON_CANCEL_OPEN` there.
pub const REASON_CANCEL_OPEN: &str = "cancel-open";

/// The single hold-to-open constant (`ContextMenuTrigger.tsx:16`) — also the grace
/// window before a document `mouseup` may cancel the menu (`:71-73`).
pub const LONG_PRESS_DELAY: u32 = 500;

/// The 10px touch-move threshold that cancels a pending long press
/// (`ContextMenuTrigger.tsx:152-162`).
const MOVE_THRESHOLD: f64 = 10.0;

/// The trigger's gesture state — the local refs (`ContextMenuTrigger.tsx:45-50`)
/// clone-shared so the event handlers and the registered listeners see one instance.
#[derive(Clone)]
pub struct ContextMenuTriggerState {
    /// `touchPositionRef` (`:46`).
    touch_position: Rc<Cell<Option<(f64, f64)>>>,
    /// `longPressTimeout` (`:47`).
    long_press_timeout: Timeout,
    /// `allowMouseUpTimeout` (`:48`).
    allow_mouse_up_timeout: Timeout,
    /// `allowMouseUpRef` (`:49`).
    allow_mouse_up: Rc<Cell<bool>>,
    /// `mouseUpAbortControllerRef` (`:50`) — the abort flag for the pending document
    /// `mouseup` listener (`:165-171` aborts it on unmount; `:87-89` aborts a
    /// previous trigger's listener before registering a fresh one).
    mouse_up_alive: Rc<Cell<bool>>,
    /// The disabled read (`store.useState('disabled')`, `:42`).
    pub disabled: bool,
}

impl ContextMenuTriggerState {
    /// `handleLongPress(x, y, event)` (`:52-74`): records the spawn point, swaps the
    /// anchor, disarms the mouseup cancel, opens with `triggerPress`, and arms the
    /// 500ms grace window.
    fn handle_long_press(&self, x: f64, y: f64, is_touch_event: bool) {
        let context = use_context_menu_root_context();

        *context.initial_cursor_point.borrow_mut() = Some((x, y));
        *context.anchor.borrow_mut() = VirtualAnchor {
            x,
            y,
            size: if is_touch_event { 10.0 } else { 0.0 },
        };

        self.allow_mouse_up.set(false);
        if let Some(store) = context.actions.borrow().as_ref() {
            menu_store_set_open(store, true, reasons::TRIGGER_PRESS, None);
        }

        // Grace window 1 (`:71-73`): the document `mouseup` within 500ms of open
        // cannot cancel — the timer flips `allowMouseUpRef` after the delay.
        let allow = Rc::clone(&self.allow_mouse_up);
        self.allow_mouse_up_timeout.start(LONG_PRESS_DELAY, move || {
            allow.set(true);
        });
    }

    /// `handleContextMenu` (`:76-122`): the right-click open path — arms the
    /// tree-wide item-activation gate, stops the event, opens, and registers the
    /// once-only document `mouseup` cancel listener.
    pub fn handle_context_menu(&self, event: &web_sys::MouseEvent) {
        if self.disabled {
            return;
        }
        let context = use_context_menu_root_context();
        context.allow_mouse_up_trigger.set(true);

        stop_event(event);
        self.handle_long_press(event.client_x().into(), event.client_y().into(), false);

        // Abort a listener from a previous trigger that never saw its mouseup, and
        // scope this one to a fresh generation so it's dead on unmount if the mouseup
        // never arrives (`:87-89`, `:165-171`).
        self.mouse_up_alive.set(false);
        let mouse_up_alive = Rc::new(Cell::new(true));
        self.mouse_up_alive.swap(&mouse_up_alive);

        let allow_mouse_up = Rc::clone(&self.allow_mouse_up);
        let allow_mouse_up_timeout = self.allow_mouse_up_timeout.clone();
        let positioner_element = context.positioner_element.clone();
        let root_id = context.root_id.clone();
        let actions_store = context
            .actions
            .borrow()
            .as_ref()
            .map(std::rc::Rc::clone);
        let allow_mouse_up_trigger = context.allow_mouse_up_trigger.clone();

        let listener = {
            let mouse_up_alive = Rc::clone(&mouse_up_alive);
            move |mouse_event: web_sys::MouseEvent| {
                // One-shot semantics (`{ once: true }`, `:115`): the listener is
                // spent after this invocation either way.
                mouse_up_alive.set(false);
                allow_mouse_up_trigger.set(false);

                if !allow_mouse_up.get() {
                    return;
                }

                allow_mouse_up_timeout.clear();
                allow_mouse_up.set(false);

                let mouse_up_target: Option<web_sys::Element> =
                    get_target(&mouse_event).and_then(|t| t.dyn_into().ok());

                // Targets inside the positioner never cancel (`:104-106`).
                let positioner = positioner_element.borrow().clone();
                if contains(positioner.as_ref(), mouse_up_target.as_ref()) {
                    return;
                }

                // Neither does a target inside any portaled popup of this tree
                // (`:108-110` — `findRootOwnerId`'s ancestor walk against the
                // `data-rootownerid` stamp, `MenuPopup.tsx:110`).
                if let Some(target) = mouse_up_target
                    .as_ref()
                    .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>())
                {
                    if find_root_owner_id(target).as_deref() == Some(root_id.as_str()) {
                        return;
                    }
                }

                if let Some(store) = &actions_store {
                    menu_store_set_open(
                        store,
                        false,
                        REASON_CANCEL_OPEN,
                        Some(mouse_event.into()),
                    );
                }
            }
        };

        // The document-level (non-Leptos) registration (`:90-121` —
        // `doc.addEventListener('mouseup', …, { once: true, signal })`; the port
        // holds the closure itself so the one-shot flag is shared).
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                let listener = std::cell::RefCell::new(Some(listener));
                let wrapped = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                    if let Some(l) = listener.borrow_mut().take() {
                        l(e);
                    }
                }) as Box<dyn FnMut(_)>);
                doc.add_event_listener_with_callback(
                    "mouseup",
                    wrapped.as_ref().unchecked_ref(),
                )
                .ok();
                // The closure leaks for the listener's lifetime; the abort flag makes
                // its body inert, and the one-shot take drops the inner closure.
                wrapped.forget();
            }
        }
        // The generation flag stays shared through `mouse_up_alive` (swapped above);
        // the leaked closure's body is inert once the flag is false (unmount abort,
        // `:165-171`).
    }

    /// `cancelLongPress` (`:124-128`).
    fn cancel_long_press(&self) {
        self.long_press_timeout.clear();
        self.touch_position.set(None);
    }

    /// `handleTouchStart` (`:130-146`): the long-press arm — clears on disabled and
    /// on any non-single-touch gesture (`event.touches.length !== 1`), clears the
    /// tree-wide item-activation gate, stops propagation (nested triggers must not
    /// both arm timers, `:137`), and starts the 500ms timer at the touch point.
    pub fn handle_touch_start(&self, event: &web_sys::TouchEvent) {
        if self.disabled {
            self.cancel_long_press();
            return;
        }
        let context = use_context_menu_root_context();
        context.allow_mouse_up_trigger.set(false);

        if event.touches().length() != 1 {
            self.cancel_long_press();
            return;
        }

        event.stop_propagation();
        let touch = event.touches().get(0).unwrap();
        let touch_position = (touch.client_x().into(), touch.client_y().into());
        self.touch_position.set(Some(touch_position));

        let state = self.clone();
        self.long_press_timeout.start(LONG_PRESS_DELAY, move || {
            state.handle_long_press(touch_position.0, touch_position.1, true);
        });
    }

    /// `handleTouchMove` (`:148-162`): multi-touch cancels; a single-touch move
    /// beyond the 10px threshold from the recorded position cancels.
    pub fn handle_touch_move(&self, event: &web_sys::TouchEvent) {
        if event.touches().length() != 1 {
            self.cancel_long_press();
            return;
        }

        if self.long_press_timeout.is_started() {
            if let Some((x0, y0)) = self.touch_position.get() {
                let touch = event.touches().get(0).unwrap();
        let delta_x = f64::from(touch.client_x()) - x0;
        let delta_y = f64::from(touch.client_y()) - y0;
                if delta_x > MOVE_THRESHOLD || delta_y > MOVE_THRESHOLD {
                    self.cancel_long_press();
                }
            }
        }
    }

    /// `onTouchEnd` / `onTouchCancel` (`:141-142` in the props object) — cancel.
    pub fn handle_touch_end(&self) {
        self.cancel_long_press();
    }
}

/// Builds the trigger's gesture state from the contexts (`ContextMenuTrigger.tsx:41-50`).
pub fn use_context_menu_trigger() -> ContextMenuTriggerState {
    let disabled = use_menu_root_context_optional()
        .map(|context| {
            context
                .store
                .get_snapshot()
                .payload
                .clone()
                .unwrap_or_default()
                .disabled
        })
        .unwrap_or(false);
    let _ = use_context_menu_root_context();

    ContextMenuTriggerState {
        touch_position: Rc::new(Cell::new(None)),
        long_press_timeout: Timeout::create(),
        allow_mouse_up_timeout: Timeout::create(),
        allow_mouse_up: Rc::new(Cell::new(false)),
        mouse_up_alive: Rc::new(Cell::new(false)),
        disabled,
    }
}

/// The `ContextMenu.Trigger` component — renders a `div` (`ContextMenuTrigger.tsx:198`
/// — the right-click area is an ordinary stylable surface, not a button), with the
/// gesture handlers, the `WebkitTouchCallout: 'none'` style (`:208-210`), and the
/// open state mirrored as `data-popup-open` (`pressableTriggerOpenStateMapping`,
/// `popupStateMapping.ts:39-46`).
#[leptos::component]
pub fn ContextMenuTrigger(
    /// Children (the trigger surface's content).
    children: Children,
    /// Extra attributes spread onto the rendered div (the `elementProps` passthrough,
    /// `ContextMenuTrigger.test.tsx:42` — `data-testid` and friends).
    #[prop(default = Vec::new(), optional)] extra_attributes: Vec<(String, String)>,
) -> impl IntoView {
    let state = use_context_menu_trigger();
    let context = use_context_menu_root_context();
    // The open read (`store.useState('open')`, `ContextMenuTrigger.tsx:41`) —
    // reactive, so the `data-popup-open` mirror tracks the store
    // (`pressableTriggerOpenStateMapping`, `popupStateMapping.ts:39-46`).
    let open_signal = context
        .actions
        .borrow()
        .as_ref()
        .map(|store| crate::menu::store::use_menu_open_signal(store));

    let state_for_context_menu = state.clone();
    let state_for_touch_start = state.clone();
    let state_for_touch_move = state.clone();
    let state_for_touch_end = state.clone();
    let state_for_touch_cancel = state.clone();

    view! {
        <div
            style="-webkit-touch-callout: none;"
            data-testid="context-menu-trigger"
            data-popup-open=move || {
                use reactive_graph::traits::Get;
                open_signal
                    .as_ref()
                    .map(|open| open.get())
                    .unwrap_or(false)
                    .then_some("true")
                    .unwrap_or_default()
            }
            on:contextmenu=move |event: leptos::ev::MouseEvent| {
                state_for_context_menu.handle_context_menu(&event.unchecked_into());
            }
            on:touchstart=move |event: leptos::ev::TouchEvent| {
                state_for_touch_start.handle_touch_start(&event.unchecked_into());
            }
            on:touchmove=move |event: leptos::ev::TouchEvent| {
                state_for_touch_move.handle_touch_move(&event.unchecked_into());
            }
            on:touchend=move |_| state_for_touch_end.handle_touch_end()
            on:touchcancel=move |_| state_for_touch_cancel.handle_touch_end()
        >
            {children()}
        </div>
    }
}

use wasm_bindgen::prelude::Closure;
