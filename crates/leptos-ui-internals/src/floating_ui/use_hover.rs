//! Port of `packages/react/src/floating-ui-react/hooks/useHover.ts` — opens the
//! floating element while hovering over the reference element, like CSS `:hover`
//! (`specs/library/floating-ui-react/behavior.md`, "Public API surface": `useHover`
//! (`delay`, `restMs`, `handleClose`, `move`); "Events": mouseenter/mouseleave/
//! mousemove listeners with `movementX`/`movementY` tremor filtering; "Edge cases":
//! the rapid enter/leave and corridor behaviors).
//!
//! ## Rust adaptations
//!
//! - The hook keeps its state local (upstream's `pointerTypeRef`/`interactedInsideRef`/
//!   `handlerRef`/`blockMouseMoveRef`/`performedPointerEventsMutationRef`/
//!   `unbindMouseMoveRef`/`restTimeoutPendingRef`, `useHover.ts:79-85`) — the port
//!   shares them as `Rc` `Cell`/`RefCell` handles between the effects and the returned
//!   handler bag, the `useDismiss` adaptation.
//! - The `useValueAsRef` mirrors of `handleClose`/`delay`/`restMs` (`:75-77`) are
//!   folded away: the port's props are per-call constants (components run once), so
//!   the handlers read them directly — the stable-identity concern those refs solve is
//!   React render-phase specific (the `useClick` adaptation).
//! - The effects port to `use_iso_layout_effect` with the reactive reads standing in
//!   for the dependency arrays. Two unmount-only effects (`:396-403`, `:405-407`) fold
//!   into `on_cleanup` registrations at hook-call time: upstream re-runs them when the
//!  DOM reference changes, which the port folds into the single final cleanup (the
//!   hook body runs once; reference swaps ride store updates, not re-renders).
//! - `addEventListener(trigger, 'mousemove', onReferenceMouseEnter, { once: true })`
//!   (`:320`) ports to [`add_event_listener_once`]: web-sys's option plumbing has no
//!   `once` variant, so the wrapper unsubscribes after the first delivery.
//! - `body.style.pointerEvents = ...` mutations go through the shared-state module's
//!   inline-style helpers (the legacy body-mutation variant, `useHover.ts:353-384`).
//! - The `handleClose` context is built from `dataRef.current.floatingContext`'s live
//!   members — placement/elements/nodeId (`useHover.ts:190-196` upstream spread) —
//!   see `use_floating`'s stamping effect for where those live.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, EventTarget, MouseEvent, PointerEvent};

use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_timeout;

use crate::floating_ui::element::is_interactive_element;
use crate::floating_ui::element_props::{ElementHandlers, ElementProps, FloatingContextSource};
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::reasons;
use crate::floating_ui::tree::{use_floating_parent_node_id, use_floating_tree};
use crate::floating_ui::types::RootOpenChangeEventDetails;
use crate::floating_ui::use_hover_interaction_shared_state::{
    remove_pointer_events, set_pointer_events,
};
use crate::floating_ui::use_hover_shared::{
    DelayInput, HandleClose, HandleCloseContext, HandleCloseContextBase, MouseMoveHandler,
    RestMsInput, get_delay, get_rest_ms, is_click_like_open_event, is_hover_open_event,
};

/// Port of `UseHoverProps` (`useHover.ts:27-52`) with the documented defaults
/// (`useHover.ts:63`).
#[derive(Clone)]
pub struct UseHoverProps {
    /// `handleClose` (`useHover.ts:33` — default `null`): the corridor factory (e.g.
    /// [`crate::floating_ui::safe_polygon::safe_polygon`]) deciding when the popup
    /// closes after the cursor leaves the reference.
    pub handle_close: Option<HandleClose>,
    /// `restMs` (`useHover.ts:39` — default `0`): wait for the cursor to rest before
    /// opening.
    pub rest_ms: RestMsInput,
    /// `delay` (`useHover.ts:45` — default `0`): wait before changing the open state.
    pub delay: DelayInput,
    /// `move` (`useHover.ts:51` — default `true`): whether moving the cursor over the
    /// floating element opens it without a regular hover event.
    pub move_: bool,
}

// Handwritten rather than derived: the derived `bool` default is `false`, but
// upstream's destructured default is `move = true` (`useHover.ts:63`) — the
// `UseFocusProps` precedent.
impl Default for UseHoverProps {
    fn default() -> Self {
        Self {
            handle_close: None,
            rest_ms: RestMsInput::default(),
            delay: DelayInput::default(),
            move_: true,
        }
    }
}

/// The `{ once: true }` listener registration (`useHover.ts:320`): unsubscribes after
/// the first delivery — and the unsubscribe itself stays valid afterwards (a second
/// call is a no-op), matching the merged-cleanup flow. Shared with the split hover
/// hooks, which register the same once-per-enter re-arm (`useHoverReferenceInteraction.
/// ts:380`).
pub(crate) fn add_event_listener_once(
    target: &EventTarget,
    event_type: &str,
    handler: impl FnMut(&Event) + 'static,
) -> Box<dyn Fn()> {
    let handle: Rc<RefCell<Option<EventListenerUnsubscribe>>> = Rc::new(RefCell::new(None));
    let handle_for_listener = Rc::clone(&handle);
    let unsubscribe = leptos_ui_utils::add_event_listener(target, event_type, {
        let mut handler = handler;
        move |event: &Event| {
            handler(event);
            if let Some(unsubscribe) = handle_for_listener.borrow_mut().take() {
                unsubscribe.unsubscribe();
            }
        }
    });
    *handle.borrow_mut() = Some(unsubscribe);
    Box::new(move || {
        if let Some(unsubscribe) = handle.borrow_mut().take() {
            unsubscribe.unsubscribe();
        }
    })
}

/// Port of `useHover(context, props)` (`useHover.ts:59-467`). Must be called inside a
/// reactive owner (the listeners and timers register cleanups). Returns the
/// `reference` handler bag (`:466` — upstream returns `{ reference }` only).
pub fn use_hover(context: impl Into<FloatingContextSource>, props: UseHoverProps) -> ElementProps {
    let UseHoverProps {
        handle_close,
        rest_ms,
        delay,
        move_,
    } = props;

    let store: Rc<FloatingRootStore> = context.into().root_store();
    let inner = store.rc();

    // `const open = store.useState('open')` etc. (`useHover.ts:67-69`).
    let open = inner.use_state(selectors::open);
    let floating_element = inner.use_state(selectors::floating_element);
    let dom_reference_element = inner.use_state(selectors::dom_reference_element);
    let data_ref = Rc::clone(&store.context.data_ref);
    let events = Rc::clone(&store.context.events);

    // `const tree = useFloatingTree(); const parentId = useFloatingParentNodeId();`
    // (`useHover.ts:72-73`).
    let tree = use_floating_tree(None);
    let parent_id = use_floating_parent_node_id();

    // The local refs (`useHover.ts:79-85`).
    let pointer_type_ref: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let interacted_inside_ref = Rc::new(Cell::new(false));
    let handler_ref: Rc<RefCell<Option<MouseMoveHandler>>> = Rc::new(RefCell::new(None));
    let block_mouse_move_ref = Rc::new(Cell::new(true));
    let performed_pointer_events_mutation_ref = Rc::new(Cell::new(false));
    let unbind_mouse_move_ref: Rc<RefCell<Option<Box<dyn FnOnce()>>>> = Rc::new(RefCell::new(None));
    let rest_timeout_pending_ref = Rc::new(Cell::new(false));

    // `const timeout = useTimeout(); const restTimeout = useTimeout();` (`:87-88`).
    let timeout = use_timeout();
    let rest_timeout = use_timeout();

    // `isHoverOpen` (`:90-92`).
    let is_hover_open: Rc<dyn Fn() -> bool> = {
        let data_ref = Rc::clone(&data_ref);
        Rc::new(move || {
            is_hover_open_event(
                data_ref
                    .borrow()
                    .open_event
                    .as_ref()
                    .map(|event| event.type_())
                    .as_deref(),
            )
        })
    };

    // `isClickLikeOpenEvent` (`:94-96`).
    let is_click_like_open_event: Rc<dyn Fn() -> bool> = {
        let data_ref = Rc::clone(&data_ref);
        let interacted_inside_ref = Rc::clone(&interacted_inside_ref);
        Rc::new(move || {
            is_click_like_open_event(
                data_ref
                    .borrow()
                    .open_event
                    .as_ref()
                    .map(|event| event.type_())
                    .as_deref(),
                interacted_inside_ref.get(),
            )
        })
    };

    // `cleanupMouseMoveHandler` (`:98-101`).
    let cleanup_mouse_move_handler: Rc<dyn Fn()> = {
        let unbind_mouse_move_ref = Rc::clone(&unbind_mouse_move_ref);
        let handler_ref = Rc::clone(&handler_ref);
        Rc::new(move || {
            if let Some(unbind) = unbind_mouse_move_ref.borrow_mut().take() {
                unbind();
            }
            *handler_ref.borrow_mut() = None;
        })
    };

    // `clearPointerEvents` (`:103-109`): the legacy body mutation — the hook's own
    // single-writer record.
    let clear_pointer_events: Rc<dyn Fn()> = {
        let performed_pointer_events_mutation_ref =
            Rc::clone(&performed_pointer_events_mutation_ref);
        let store = Rc::clone(&store);
        Rc::new(move || {
            if performed_pointer_events_mutation_ref.get() {
                let floating = selectors::floating_element(&store.get_snapshot());
                let doc = owner_document(
                    floating
                        .as_ref()
                        .map(|element| element.as_ref() as &web_sys::Node),
                );
                if let Some(body) = doc.body() {
                    body.style().remove_property("pointer-events").unwrap();
                }
                performed_pointer_events_mutation_ref.set(false);
            }
        })
    };

    // The openchange listener (`useHover.ts:113-127`): a close cancels the pending
    // delay/rest timers and re-arms the move block. Deps are stable handles, so the
    // subscription registers once.
    {
        let timeout = timeout.clone();
        let rest_timeout = rest_timeout.clone();
        let block_mouse_move_ref = Rc::clone(&block_mouse_move_ref);
        let rest_timeout_pending_ref = Rc::clone(&rest_timeout_pending_ref);
        let unsubscribe = events.on(
            "openchange",
            Rc::new(
                move |details: &crate::floating_ui::types::FloatingUIOpenChangeDetails| {
                    if !details.open {
                        timeout.clear();
                        rest_timeout.clear();
                        block_mouse_move_ref.set(true);
                        rest_timeout_pending_ref.set(false);
                    }
                },
            ),
        );
        let unsubscribe = send_wrapper::SendWrapper::new(unsubscribe);
        reactive_graph::owner::on_cleanup(move || unsubscribe());
    }

    // The document-element `mouseleave` effect (`useHover.ts:129-157`): an open
    // hover-opened popup closes when the cursor leaves the document.
    {
        let handle_close = handle_close.clone();
        let open = open.clone();
        let floating_element = floating_element.clone();
        let store = Rc::clone(&store);
        let is_hover_open = Rc::clone(&is_hover_open);
        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
        use_iso_layout_effect(move || {
            let open_value = open.get();
            if handle_close.is_none() || !open_value {
                return;
            }

            let is_hover_open = Rc::clone(&is_hover_open);
            let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
            let store = Rc::clone(&store);
            let on_leave = move |event: &MouseEvent| {
                if is_click_like_open_event() {
                    return;
                }

                if is_hover_open() {
                    let trigger: Option<Element> = event
                        .current_target()
                        .and_then(|target| target.dyn_into::<Element>().ok());
                    store.set_open(
                        false,
                        &RootOpenChangeEventDetails::new(
                            reasons::TRIGGER_HOVER,
                            event.clone().into(),
                            trigger,
                            String::new(),
                        ),
                    );
                }
            };

            let floating = floating_element.get();
            let doc = owner_document(
                floating
                    .as_ref()
                    .map(|element| element.as_ref() as &web_sys::Node),
            );
            let Some(html) = doc.document_element() else {
                return;
            };
            let unsubscribe =
                leptos_ui_utils::add_event_listener(&html, "mouseleave", move |event: &Event| {
                    if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                        on_leave(mouse_event);
                    }
                });
            let unsubscribe = send_wrapper::SendWrapper::new(unsubscribe);
            reactive_graph::owner::on_cleanup(move || unsubscribe.take().unsubscribe());
        });
    }

    // The trigger/floating listener effect (`useHover.ts:162-347`) — the core hover
    // wiring. The reactive reads (`open`, `domReferenceElement`, `floatingElement`)
    // stand in for the dependency array.
    {
        let handle_close = handle_close.clone();
        let open = open.clone();
        let floating_element_signal = floating_element.clone();
        let dom_reference_element_signal = dom_reference_element.clone();
        let store = Rc::clone(&store);
        let data_ref = Rc::clone(&data_ref);
        let tree = tree.clone();
        let pointer_type_ref = Rc::clone(&pointer_type_ref);
        let rest_timeout_pending_ref = Rc::clone(&rest_timeout_pending_ref);
        let interacted_inside_ref = Rc::clone(&interacted_inside_ref);
        let handler_ref = Rc::clone(&handler_ref);
        let unbind_mouse_move_ref = Rc::clone(&unbind_mouse_move_ref);
        let block_mouse_move_ref = Rc::clone(&block_mouse_move_ref);
        let timeout = timeout.clone();
        let rest_timeout = rest_timeout.clone();
        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
        let rest_ms = rest_ms.clone();
        let delay = delay.clone();
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let trigger: Option<Element> = dom_reference_element_signal.get();
            let floating: Option<Element> = floating_element_signal.get();

            // `const trigger = domReferenceElement as HTMLElement | null;`
            // `if (isElement(trigger)) { ... }` (`:313-315`).
            let Some(trigger) = trigger else {
                return;
            };

            // `closeWithDelay` (`:163-173`): the pending document mousemove handler
            // (`handlerRef`) suppresses the delayed close — a corridor in flight owns
            // the close decision.
            let close_with_delay: Rc<dyn Fn(&MouseEvent, bool)> = {
                let delay = delay.clone();
                let pointer_type_ref = Rc::clone(&pointer_type_ref);
                let handler_ref = Rc::clone(&handler_ref);
                let timeout = timeout.clone();
                let store = Rc::clone(&store);
                Rc::new(move |event: &MouseEvent, run_else_branch: bool| {
                    let close_delay =
                        get_delay(&delay, "close", pointer_type_ref.borrow().as_deref());
                    if close_delay > 0 && handler_ref.borrow().is_none() {
                        let timeout = timeout.clone();
                        let store = Rc::clone(&store);
                        let event = event.clone();
                        timeout.start(close_delay, move || {
                            store.set_open(
                                false,
                                &RootOpenChangeEventDetails::new(
                                    reasons::TRIGGER_HOVER,
                                    event.into(),
                                    None,
                                    String::new(),
                                ),
                            );
                        });
                    } else if run_else_branch {
                        timeout.clear();
                        store.set_open(
                            false,
                            &RootOpenChangeEventDetails::new(
                                reasons::TRIGGER_HOVER,
                                event.clone().into(),
                                None,
                                String::new(),
                            ),
                        );
                    }
                })
            };

            // `handleInteractInside` (`:175-183`).
            let handle_interact_inside = {
                let interacted_inside_ref = Rc::clone(&interacted_inside_ref);
                move |event: &Event| {
                    let target: Option<Element> =
                        get_target(event).and_then(|target| target.dyn_into::<Element>().ok());
                    if !is_interactive_element(target.as_ref()) {
                        interacted_inside_ref.set(false);
                        return;
                    }

                    interacted_inside_ref.set(true);
                }
            };

            // `getHandleCloseHandler` (`:185-197`): builds the corridor context from
            // the stashed floating context, the event's cursor position, and the
            // injected tree.
            let get_handle_close_handler: Rc<
                dyn Fn(&MouseEvent, Rc<dyn Fn()>) -> Option<MouseMoveHandler>,
            > = {
                let handle_close = handle_close.clone();
                let data_ref = Rc::clone(&data_ref);
                let tree = tree.clone();
                Rc::new(move |event: &MouseEvent, on_close: Rc<dyn Fn()>| {
                    let handle_close = handle_close.as_ref()?;
                    let floating_context = data_ref.borrow().floating_context.clone()?;
                    let node_id = data_ref.borrow().floating_node_id.clone();
                    let base = HandleCloseContextBase {
                        placement: Some(floating_context.placement.get_untracked()),
                        dom_reference: floating_context.elements.dom_reference.get_untracked(),
                        floating: floating_context.elements.floating.get_untracked(),
                        node_id,
                        leave: None,
                    };
                    let context = HandleCloseContext {
                        x: Some(event.client_x() as f64),
                        y: Some(event.client_y() as f64),
                        base,
                        on_close,
                        tree: tree.clone(),
                    };
                    Some((handle_close.factory)(&context))
                })
            };

            // `onReferenceMouseEnter` (`:199-223`).
            let on_reference_mouse_enter = {
                let timeout = timeout.clone();
                let block_mouse_move_ref = Rc::clone(&block_mouse_move_ref);
                let rest_ms = rest_ms.clone();
                let delay = delay.clone();
                let pointer_type_ref = Rc::clone(&pointer_type_ref);
                let store = Rc::clone(&store);
                let open_value = open_value;
                move |event: &MouseEvent| {
                    timeout.clear();
                    block_mouse_move_ref.set(false);

                    // Only rest delay is set; there's no fallback delay. This will be
                    // handled by `onMouseMove` (`:203-205`).
                    if get_rest_ms(&rest_ms) > 0 && get_delay(&delay, "open", None) == 0 {
                        return;
                    }

                    let open_delay =
                        get_delay(&delay, "open", pointer_type_ref.borrow().as_deref());
                    let trigger: Option<Element> = event
                        .current_target()
                        .and_then(|target| target.dyn_into::<Element>().ok());

                    let dom_reference = selectors::dom_reference_element(&store.get_snapshot());

                    let is_over_inactive_trigger = dom_reference
                        .as_ref()
                        .zip(trigger.as_ref())
                        .map(|(dom_reference, trigger)| {
                            !contains(Some(dom_reference), Some(trigger))
                        })
                        .unwrap_or(false);

                    if open_delay > 0 {
                        let timeout = timeout.clone();
                        let store = Rc::clone(&store);
                        let event = event.clone();
                        timeout.start(open_delay, move || {
                            if !store.select(selectors::open) {
                                let trigger = trigger.clone();
                                store.set_open(
                                    true,
                                    &RootOpenChangeEventDetails::new(
                                        reasons::TRIGGER_HOVER,
                                        event.into(),
                                        trigger,
                                        String::new(),
                                    ),
                                );
                            }
                        });
                    } else if !open_value || is_over_inactive_trigger {
                        store.set_open(
                            true,
                            &RootOpenChangeEventDetails::new(
                                reasons::TRIGGER_HOVER,
                                event.clone().into(),
                                trigger,
                                String::new(),
                            ),
                        );
                    }
                }
            };

            // `onReferenceMouseLeave` (`:225-275`).
            let on_reference_mouse_leave = {
                let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                let clear_pointer_events = Rc::clone(&clear_pointer_events);
                let unbind_mouse_move_ref = Rc::clone(&unbind_mouse_move_ref);
                let rest_timeout = rest_timeout.clone();
                let rest_timeout_pending_ref = Rc::clone(&rest_timeout_pending_ref);
                let store = Rc::clone(&store);
                let floating = floating.clone();
                let timeout = timeout.clone();
                let open_value = open_value;
                let pointer_type_ref = Rc::clone(&pointer_type_ref);
                let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
                let handler_ref = Rc::clone(&handler_ref);
                let get_handle_close_handler = Rc::clone(&get_handle_close_handler);
                let close_with_delay = Rc::clone(&close_with_delay);
                move |event: &MouseEvent| {
                    if is_click_like_open_event() {
                        clear_pointer_events();
                        return;
                    }

                    if let Some(unbind) = unbind_mouse_move_ref.borrow_mut().take() {
                        unbind();
                    }

                    let doc = owner_document(
                        floating
                            .as_ref()
                            .map(|element| element.as_ref() as &web_sys::Node),
                    );
                    rest_timeout.clear();
                    rest_timeout_pending_ref.set(false);

                    let triggers = &store.context.trigger_elements;
                    if event
                        .related_target()
                        .and_then(|related| related.dyn_ref::<Element>().cloned())
                        .map(|related| triggers.has_element(&related))
                        .unwrap_or(false)
                    {
                        // If the mouse is leaving the reference element to another
                        // trigger, don't explicitly close the popup as it will be
                        // moved. (`:240-243`)
                        return;
                    }

                    let on_close: Rc<dyn Fn()> = {
                        let clear_pointer_events = Rc::clone(&clear_pointer_events);
                        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
                        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                        let close_with_delay = Rc::clone(&close_with_delay);
                        let event = event.clone();
                        Rc::new(move || {
                            clear_pointer_events();
                            cleanup_mouse_move_handler();
                            if !is_click_like_open_event() {
                                close_with_delay(&event, true);
                            }
                        })
                    };

                    if let Some(handler) = get_handle_close_handler(event, Rc::clone(&on_close)) {
                        // Prevent clearing `onScrollMouseLeave` timeout (`:254-257`).
                        if !open_value {
                            timeout.clear();
                        }

                        *handler_ref.borrow_mut() = Some(Rc::clone(&handler));
                        let unsubscribe = leptos_ui_utils::add_event_listener(
                            &doc,
                            "mousemove",
                            move |event: &Event| {
                                if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                                    handler(mouse_event);
                                }
                            },
                        );
                        *unbind_mouse_move_ref.borrow_mut() =
                            Some(Box::new(move || unsubscribe.unsubscribe()));

                        return;
                    }

                    // Allow interactivity without `safePolygon` on touch devices. With
                    // a pointer, a short close delay is an alternative, so it should
                    // work consistently. (`:265-274`)
                    let should_close = {
                        let pointer_type = pointer_type_ref.borrow().clone();
                        if pointer_type.as_deref() == Some("touch") {
                            let related: Option<Element> = event
                                .related_target()
                                .and_then(|related| related.dyn_into::<Element>().ok());
                            !contains(floating.as_ref(), related.as_ref())
                        } else {
                            true
                        }
                    };
                    if should_close {
                        close_with_delay(event, true);
                    }
                }
            };

            // `onScrollMouseLeave` (`:280-300`): ensure the floating element closes
            // after scrolling even if the pointer did not move.
            let on_scroll_mouse_leave = {
                let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                let data_ref = Rc::clone(&data_ref);
                let store = Rc::clone(&store);
                let clear_pointer_events = Rc::clone(&clear_pointer_events);
                let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
                let get_handle_close_handler = Rc::clone(&get_handle_close_handler);
                let close_with_delay = Rc::clone(&close_with_delay);
                move |event: &MouseEvent| {
                    if is_click_like_open_event()
                        || data_ref.borrow().floating_context.is_none()
                        || !store.select(selectors::open)
                    {
                        return;
                    }

                    let triggers = &store.context.trigger_elements;
                    if event
                        .related_target()
                        .and_then(|related| related.dyn_ref::<Element>().cloned())
                        .map(|related| triggers.has_element(&related))
                        .unwrap_or(false)
                    {
                        return;
                    }

                    let on_close: Rc<dyn Fn()> = {
                        let clear_pointer_events = Rc::clone(&clear_pointer_events);
                        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
                        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                        let close_with_delay = Rc::clone(&close_with_delay);
                        let event = event.clone();
                        Rc::new(move || {
                            clear_pointer_events();
                            cleanup_mouse_move_handler();
                            if !is_click_like_open_event() {
                                close_with_delay(&event, true);
                            }
                        })
                    };

                    if let Some(handler) = get_handle_close_handler(event, on_close) {
                        handler(event);
                    }
                }
            };

            // `onFloatingMouseEnter` / `onFloatingMouseLeave` (`:302-311`).
            let on_floating_mouse_enter = {
                let timeout = timeout.clone();
                let clear_pointer_events = Rc::clone(&clear_pointer_events);
                move || {
                    timeout.clear();
                    clear_pointer_events();
                }
            };
            let on_floating_mouse_leave = {
                let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                let close_with_delay = Rc::clone(&close_with_delay);
                move |event: &MouseEvent| {
                    if !is_click_like_open_event() {
                        close_with_delay(event, false);
                    }
                }
            };

            // The listener fan-out (`:313-328`).
            let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

            if open_value {
                let on_scroll_mouse_leave = on_scroll_mouse_leave.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    &trigger,
                    "mouseleave",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_scroll_mouse_leave(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            if move_ {
                let on_reference_mouse_enter = on_reference_mouse_enter.clone();
                let once_cleanup =
                    add_event_listener_once(trigger.as_ref(), "mousemove", move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_reference_mouse_enter(mouse_event);
                        }
                    });
                cleanups.push(Some(Box::new(once_cleanup)));
            }

            {
                let on_reference_mouse_enter = on_reference_mouse_enter.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    &trigger,
                    "mouseenter",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_reference_mouse_enter(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            {
                let on_reference_mouse_leave = on_reference_mouse_leave.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    &trigger,
                    "mouseleave",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_reference_mouse_leave(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            if let Some(floating) = floating.as_ref() {
                {
                    let on_scroll_mouse_leave = on_scroll_mouse_leave.clone();
                    let unsubscribe = leptos_ui_utils::add_event_listener(
                        floating,
                        "mouseleave",
                        move |event: &Event| {
                            if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                                on_scroll_mouse_leave(mouse_event);
                            }
                        },
                    );
                    cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
                }

                {
                    let on_floating_mouse_leave = on_floating_mouse_leave.clone();
                    let unsubscribe = leptos_ui_utils::add_event_listener(
                        floating,
                        "mouseleave",
                        move |event: &Event| {
                            if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                                on_floating_mouse_leave(mouse_event);
                            }
                        },
                    );
                    cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
                }

                let on_floating_mouse_enter = on_floating_mouse_enter.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    floating,
                    "mouseenter",
                    move |_event: &Event| on_floating_mouse_enter(),
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));

                let handle_interact_inside = handle_interact_inside;
                let unsubscribe = leptos_ui_utils::add_event_listener_with_options(
                    floating,
                    "pointerdown",
                    handle_interact_inside,
                    true,
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            let merged = merge_cleanups(cleanups);
            let merged = send_wrapper::SendWrapper::new(merged);
            reactive_graph::owner::on_cleanup(move || merged.take()());
        });
    }

    // The blockPointerEvents body mutation (`useHover.ts:353-384`) — the legacy
    // variant this hook owns; the split hover hooks use the shared-state module's
    // scoped mutation instead.
    {
        let handle_close = handle_close.clone();
        let open = open.clone();
        let floating_element = floating_element.clone();
        let dom_reference_element = dom_reference_element.clone();
        let is_hover_open = Rc::clone(&is_hover_open);
        let performed_pointer_events_mutation_ref =
            Rc::clone(&performed_pointer_events_mutation_ref);
        let tree = tree.clone();
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let floating: Option<Element> = floating_element.get();
            let dom_reference: Option<Element> = dom_reference_element.get();
            let block_pointer_events = handle_close
                .as_ref()
                .map(|handle_close| handle_close.options.block_pointer_events)
                .unwrap_or(false);

            if open_value && block_pointer_events && is_hover_open() {
                performed_pointer_events_mutation_ref.set(true);

                if let (Some(dom_reference), Some(floating)) = (dom_reference, floating) {
                    let doc = owner_document(Some(floating.as_ref() as &web_sys::Node));
                    let body = doc.body();

                    // Let the parent floating element stay interactive (`:364-369`).
                    let parent_floating = tree.as_ref().and_then(|tree| {
                        let nodes = tree.nodes.borrow();
                        let parent_context = nodes
                            .iter()
                            .find(|node| node.id.as_deref() == parent_id.as_deref())
                            .and_then(|node| node.context.borrow().as_ref().cloned());
                        parent_context.and_then(|context| context.elements.floating.get_untracked())
                    });
                    if let Some(parent_floating) = parent_floating {
                        remove_pointer_events(&parent_floating);
                    }

                    set_pointer_events(body.as_ref().map(|body| body as &Element), "none");
                    set_pointer_events(Some(&dom_reference), "auto");
                    set_pointer_events(Some(&floating), "auto");

                    let dom_reference = dom_reference.clone();
                    let cleanup = send_wrapper::SendWrapper::new(move || {
                        if let Some(body) = body {
                            body.style().remove_property("pointer-events").unwrap();
                        }
                        remove_pointer_events(&dom_reference);
                        remove_pointer_events(&floating);
                    });
                    reactive_graph::owner::on_cleanup(move || cleanup.take()());
                }
            }
        });
    }

    // The reset-on-close effect (`useHover.ts:386-394`).
    {
        let open = open.clone();
        let pointer_type_ref = Rc::clone(&pointer_type_ref);
        let rest_timeout_pending_ref = Rc::clone(&rest_timeout_pending_ref);
        let interacted_inside_ref = Rc::clone(&interacted_inside_ref);
        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        use_iso_layout_effect(move || {
            let open_value = open.get();
            if !open_value {
                *pointer_type_ref.borrow_mut() = None;
                rest_timeout_pending_ref.set(false);
                interacted_inside_ref.set(false);
                cleanup_mouse_move_handler();
                clear_pointer_events();
            }
        });
    }

    // The unmount cleanups (`useHover.ts:396-407`).
    {
        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
        let timeout = timeout.clone();
        let rest_timeout = rest_timeout.clone();
        let interacted_inside_ref = Rc::clone(&interacted_inside_ref);
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        let cleanups = send_wrapper::SendWrapper::new(move || {
            cleanup_mouse_move_handler();
            timeout.clear();
            rest_timeout.clear();
            interacted_inside_ref.set(false);
            clear_pointer_events();
        });
        reactive_graph::owner::on_cleanup(move || (*cleanups)());
    }

    // The `reference` bag (`useHover.ts:409-464`).
    let reference = ElementHandlers {
        // `setPointerRef` (`:410-412`) — shared by onPointerDown and onPointerEnter.
        on_pointer_down: {
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            Some(Rc::new(move |event: &PointerEvent| {
                *pointer_type_ref.borrow_mut() = Some(event.pointer_type());
            }))
        },
        on_pointer_enter: {
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            Some(Rc::new(move |event: &PointerEvent| {
                *pointer_type_ref.borrow_mut() = Some(event.pointer_type());
            }))
        },
        // `onMouseMove` (`:417-462`) — the rest-ms engine.
        on_mouse_move: {
            let rest_ms = rest_ms.clone();
            let store = Rc::clone(&store);
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            let rest_timeout = rest_timeout.clone();
            let rest_timeout_pending_ref = Rc::clone(&rest_timeout_pending_ref);
            let block_mouse_move_ref = Rc::clone(&block_mouse_move_ref);
            Some(Rc::new(move |event: &MouseEvent| {
                let native_event = event.clone();
                let trigger: Option<Element> = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<Element>().ok());

                // `true` when there are multiple triggers per floating element and
                // user hovers over the one that wasn't used to open the floating
                // element. (`:421-425`)
                let dom_reference = selectors::dom_reference_element(&store.get_snapshot());
                let target: Option<Element> = event
                    .target()
                    .and_then(|target| target.dyn_into::<Element>().ok());
                let is_over_inactive_trigger = dom_reference
                    .as_ref()
                    .map(|dom_reference| !contains(Some(dom_reference), target.as_ref()))
                    .unwrap_or(false);

                let handle_mouse_move: Rc<dyn Fn()> = {
                    let block_mouse_move_ref = Rc::clone(&block_mouse_move_ref);
                    let store = Rc::clone(&store);
                    let is_over_inactive_trigger = is_over_inactive_trigger;
                    let native_event = native_event.clone();
                    let trigger = trigger.clone();
                    Rc::new(move || {
                        if !block_mouse_move_ref.get()
                            && (!store.select(selectors::open) || is_over_inactive_trigger)
                        {
                            store.set_open(
                                true,
                                &RootOpenChangeEventDetails::new(
                                    reasons::TRIGGER_HOVER,
                                    native_event.clone().into(),
                                    trigger.clone(),
                                    String::new(),
                                ),
                            );
                        }
                    })
                };

                if (store.select(selectors::open) && !is_over_inactive_trigger)
                    || get_rest_ms(&rest_ms) == 0
                {
                    return;
                }

                // Ignore insignificant movements to account for tremors (`:443-450`).
                if !is_over_inactive_trigger
                    && rest_timeout_pending_ref.get()
                    && (event.movement_x() as f64).powi(2) + (event.movement_y() as f64).powi(2)
                        < 2.0
                {
                    return;
                }

                rest_timeout.clear();

                if pointer_type_ref.borrow().as_deref() == Some("touch") {
                    handle_mouse_move();
                } else if is_over_inactive_trigger {
                    handle_mouse_move();
                } else {
                    rest_timeout_pending_ref.set(true);
                    let handle_mouse_move = Rc::clone(&handle_mouse_move);
                    rest_timeout.start(get_rest_ms(&rest_ms), move || handle_mouse_move());
                }
            }))
        },
        ..ElementHandlers::default()
    };

    // `return React.useMemo(() => ({ reference }), [reference])` (`:466`).
    ElementProps {
        reference: Some(reference),
        ..ElementProps::default()
    }
}

// The hook needs a DOM realm (elements, native listeners, timers): the wasm suite
// mirrors `packages/react/src/floating-ui-react/hooks/useHover.test.tsx`.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::types::{Delay, ReferenceType};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type CallLog = Rc<RefCell<Vec<(bool, String)>>>;

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// The controlled-loop harness the upstream test fixture wires
    /// (`useHover.test.tsx:12-18` — `useFloating({ open: isOpen, onOpenChange:
    /// setIsOpen })`): the consumer syncs the callback back into the store through the
    /// context-hook layout effect (the port's stand-in writes the store's open state
    /// via a Weak — the `useClick` test adaptation).
    fn store_with(open: bool, log: CallLog) -> Rc<FloatingRootStore> {
        let store = FloatingRootStore::new(FloatingRootStoreOptions {
            open,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        });
        let weak = Rc::downgrade(&store);
        store.context.set_on_open_change(Some(Rc::new(
            move |open: bool, details: &RootOpenChangeEventDetails| {
                if let Some(store) = weak.upgrade() {
                    store.update(|state, _| {
                        state.open = open;
                        true
                    });
                }
                log.borrow_mut().push((open, details.reason.clone()));
            },
        )));
        store
    }

    fn button_with_store(store: &Rc<FloatingRootStore>) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let button: web_sys::HtmlElement = document
            .create_element("button")
            .unwrap()
            .dyn_into()
            .unwrap();
        document.body().unwrap().append_child(&button).unwrap();
        let floating = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&floating).unwrap();
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(button.clone().into())),
        );
        store.set_field(
            |state| &mut state.dom_reference_element,
            Some(button.clone().into()),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating));
        button
    }

    fn mouse_event(event_type: &str, related_target: Option<&EventTarget>) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        if let Some(related_target) = related_target {
            init.set_related_target(Some(related_target));
        }
        MouseEvent::new_with_mouse_event_init_dict(event_type, &init).unwrap()
    }

    /// A `mousemove` whose `movementX`/`movementY` are overridden on the instance —
    /// the `Object.defineProperty` spy the upstream restMs tests install
    /// (`useHover.test.tsx:148-152`).
    fn mouse_move_with_movement(movement_x: i32, movement_y: i32) -> MouseEvent {
        let event = mouse_event("mousemove", None);
        js_sys::Reflect::set(
            event.as_ref(),
            &JsValue::from_str("movementX"),
            &JsValue::from(movement_x),
        )
        .unwrap();
        js_sys::Reflect::set(
            event.as_ref(),
            &JsValue::from_str("movementY"),
            &JsValue::from(movement_y),
        )
        .unwrap();
        event
    }

    fn attach(
        props: &ElementProps,
        button: &web_sys::HtmlElement,
    ) -> leptos_ui_utils::merge_cleanups::CleanupFn {
        props
            .reference
            .as_ref()
            .expect("the reference bag")
            .attach_to(button.as_ref())
            .expect("slots attached")
    }

    fn calls(log: &CallLog) -> Vec<(bool, String)> {
        log.borrow().clone()
    }

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }

    // `opens on mouseenter` (`useHover.test.tsx:31-39`): the enter fires the open
    // synchronously with reason trigger-hover.
    #[wasm_bindgen_test]
    fn opens_on_mouseenter() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(Rc::clone(&store), UseHoverProps::default());
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();

            assert_eq!(
                calls(&log),
                vec![(true, reasons::TRIGGER_HOVER.to_owned())],
                "the mouseenter opens with reason trigger-hover"
            );
        });
    }

    // `closes on mouseleave` (`useHover.test.tsx:41-47`): the leave closes.
    #[wasm_bindgen_test]
    fn closes_on_mouseleave() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(Rc::clone(&store), UseHoverProps::default());
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            button
                .dispatch_event(&mouse_event("mouseleave", None))
                .unwrap();

            assert_eq!(
                calls(&log),
                vec![
                    (true, reasons::TRIGGER_HOVER.to_owned()),
                    (false, reasons::TRIGGER_HOVER.to_owned())
                ],
                "enter opens, leave closes, both with reason trigger-hover"
            );
        });
    }

    // `mouseleave on the floating element closes it (mouse)`
    // (`useHover.test.tsx:218-229`): leaving the reference toward the floating element
    // still closes without a corridor factory.
    #[wasm_bindgen_test]
    fn mouseleave_toward_the_floating_element_closes_it() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(Rc::clone(&store), UseHoverProps::default());
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();
            button
                .dispatch_event(&mouse_event(
                    "mouseleave",
                    Some(floating.as_ref() as &EventTarget),
                ))
                .unwrap();

            assert_eq!(
                calls(&log).last().map(|(open, _)| *open),
                Some(false),
                "the leave toward the floating element closes the popup"
            );
        });
    }

    // `prop: delay — symmetric number` (`useHover.test.tsx:50-70`): the open lands at
    // the delay expiry, not before.
    #[wasm_bindgen_test(async)]
    async fn delayed_open_lands_at_the_expiry_not_before() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(
                Rc::clone(&store),
                UseHoverProps {
                    delay: DelayInput::Value(Delay::Value(100)),
                    ..UseHoverProps::default()
                },
            );
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();

            sleep(50).await;
            assert!(
                calls(&log).is_empty(),
                "the open has not fired before the delay elapses"
            );

            sleep(100).await;
            assert_eq!(
                calls(&log),
                vec![(true, reasons::TRIGGER_HOVER.to_owned())],
                "the open fires at the delay expiry"
            );
        };
        __owner.cleanup();
    }

    // `prop: delay — close` (`useHover.test.tsx:100-114`): the leave defers the close
    // by the close delay.
    #[wasm_bindgen_test(async)]
    async fn close_delay_defers_the_close() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(
                Rc::clone(&store),
                UseHoverProps {
                    delay: DelayInput::Value(Delay::Partial {
                        open: None,
                        close: Some(100),
                    }),
                    ..UseHoverProps::default()
                },
            );
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            button
                .dispatch_event(&mouse_event("mouseleave", None))
                .unwrap();

            sleep(40).await;
            assert_eq!(
                calls(&log),
                vec![(true, reasons::TRIGGER_HOVER.to_owned())],
                "the close has not fired before the close delay elapses"
            );

            sleep(120).await;
            assert_eq!(
                calls(&log).last().map(|(open, _)| *open),
                Some(false),
                "the close fires after the close delay"
            );
        };
        __owner.cleanup();
    }

    // `open with close 0` (`useHover.test.tsx:116-134`): leaving before the open delay
    // elapses cancels the pending open.
    #[wasm_bindgen_test(async)]
    async fn leaving_before_the_open_delay_elapses_cancels_the_open() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(
                Rc::clone(&store),
                UseHoverProps {
                    delay: DelayInput::Value(Delay::Partial {
                        open: Some(100),
                        close: None,
                    }),
                    ..UseHoverProps::default()
                },
            );
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            sleep(30).await;
            button
                .dispatch_event(&mouse_event("mouseleave", None))
                .unwrap();
            sleep(120).await;

            assert!(
                !calls(&log).iter().any(|(open, _)| *open),
                "the pending delayed open never fires after the leave"
            );
        };
        __owner.cleanup();
    }

    // `restMs` (`useHover.test.tsx:137-193`): significant movement restarts the rest
    // timer, so the open only lands after the pointer has rested.
    #[wasm_bindgen_test(async)]
    async fn rest_ms_opens_only_after_the_pointer_rests() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(
                Rc::clone(&store),
                UseHoverProps {
                    rest_ms: RestMsInput::Value(100),
                    ..UseHoverProps::default()
                },
            );
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_move_with_movement(10, 10))
                .unwrap();
            sleep(50).await;
            button
                .dispatch_event(&mouse_move_with_movement(10, 10))
                .unwrap();
            sleep(30).await;
            assert!(
                calls(&log).is_empty(),
                "the significant movement restarted the rest timer"
            );

            sleep(120).await;
            assert_eq!(
                calls(&log),
                vec![(true, reasons::TRIGGER_HOVER.to_owned())],
                "the open lands after the pointer has rested"
            );
        };
        __owner.cleanup();
    }

    // `restMs does not reset timer for minor mouse movement`
    // (`useHover.test.tsx:195-213`): a tremor (`movementX: 1`) does not restart the
    // rest timer, so the pending open fires.
    #[wasm_bindgen_test(async)]
    async fn minor_movement_does_not_reset_the_rest_timer() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(false, Rc::clone(&log));
            let button = button_with_store(&store);

            let props = use_hover(
                Rc::clone(&store),
                UseHoverProps {
                    rest_ms: RestMsInput::Value(100),
                    ..UseHoverProps::default()
                },
            );
            let _cleanup = attach(&props, &button);

            button
                .dispatch_event(&mouse_move_with_movement(1, 0))
                .unwrap();
            sleep(30).await;
            button
                .dispatch_event(&mouse_move_with_movement(1, 0))
                .unwrap();
            sleep(150).await;

            assert_eq!(
                calls(&log),
                vec![(true, reasons::TRIGGER_HOVER.to_owned())],
                "the tremor kept the rest timer running"
            );
        };
        __owner.cleanup();
    }
}
