//! Port of `packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.ts`
//! — the reference/trigger half of the split hover hooks: the hover enter/leave wiring
//! for wrapper/delegated trigger layouts, the delayed-open and close-delay timers, the
//! corridor (`handleClose`) session, and the rest-ms engine the trigger `mousemove`
//! drives (`specs/library/floating-ui-react/behavior.md`, "Public API surface":
//! `useHoverReferenceInteraction` (`handleClose`, `mouseOnly`, `restMs`, `delay`,
//! `move`, `triggerElementRef`, `isClosing`); "Events": native-path target resolution
//! and the wrapper-fallback trigger resolution; "Edge cases": the immediate reopen
//! during a hover-driven close transition).
//!
//! Upstream splits `useHover` into two halves that share one mutable
//! [`HoverInteraction`] through the store's `dataRef` (`useHoverInteractionSharedState`
//! — the implementation spec's "Hover (3 implementations, one shared instance)"). This
//! hook owns the reference/trigger side; the floating side is
//! [`crate::floating_ui::use_hover_floating_interaction`].
//!
//! ## Rust adaptations
//!
//! - The `useValueAsRef` mirrors of `handleClose`/`delay`/`restMs`/`enabled`/
//!   `shouldOpen`/`isClosing` (`:103-108`) fold away: the port's props are per-call
//!   constants (components run once), so the handlers read them directly — the
//!   `useHover`/`useClick` adaptation.
//! - `useStableCallback` folds into `Rc` closures shared between the effects and the
//!   returned bag (the `useDismiss` adaptation).
//! - The upstream return is `HTMLProps | undefined` — a flat handler bag the consumer
//!   spreads on a wrapper/trigger element (the upstream tests spread it on a wrapper,
//!   `useHoverReferenceInteraction.test.tsx:58,191,259`). The port's flat bag is
//!   [`ElementHandlers`]; `None` is the disabled case (`:417-419`).
//! - The document-level `mousemove` removal
//!   (`doc.removeEventListener('mousemove', instance.handler)`, `:150`) ports to a
//!   stored unsubscribe handle: `removeEventListener` needs the exact function object,
//!   which the port's listener registry identifies by handle instead.
//! - `ReactDOM.flushSync` around the touch-path open (`:490-492`) folds away — React's
//!   synchronous-render wrapper has no port counterpart; the open runs directly (the
//!   `useHover` adaptation).
//! - The two registration surfaces the hook exists for are visible in the port as
//!   upstream builds them: the native `mouseenter`/`mouseleave`/`mouseout` listeners
//!   attach to the resolved trigger (`triggerElementRef.current ??
//!   domReferenceElement`, `:210-215`), while the returned bag — pointer-type
//!   recording (`onPointerDown`/`onPointerEnter`) and the rest engine (`onMouseMove`)
//!   — is what consumers attach to a wrapper. The upstream tests exercise both at
//!   once (native `fireEvent.mouseEnter(trigger)` reaching the trigger listeners while
//!   the bag lives on the wrapper).

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::traits::GetUntracked;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, EventTarget, MouseEvent, PointerEvent};

use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_iso_layout_effect;

use crate::floating_ui::element_props::{ElementEventHandler, ElementHandlers, FloatingContextSource};
use crate::floating_ui::event::is_mouse_like_pointer_type;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::reasons;
use crate::floating_ui::tree::{use_floating_tree, SharedFloatingTreeStore};
use crate::floating_ui::types::{FloatingTreeEvent, RootOpenChangeEventDetails, TransitionStatus};
use crate::floating_ui::use_hover::add_event_listener_once;
use crate::floating_ui::use_hover_interaction_shared_state::{
    apply_safe_polygon_pointer_events_mutation, clear_safe_polygon_pointer_events_mutation,
    use_hover_interaction_shared_state,
};
use crate::floating_ui::use_hover_shared::{
    DelayInput, HandleClose, HandleCloseContext, HandleCloseContextBase, RestMsInput, get_delay,
    get_rest_ms, is_click_like_open_event, is_inside_enabled_trigger,
};

/// Port of `UseHoverReferenceInteractionProps` (`useHoverReferenceInteraction.ts:31-66`)
/// with the destructured defaults (`:78-92`).
#[derive(Clone)]
pub struct UseHoverReferenceInteractionProps {
    /// `enabled` (`:32` — default `true`).
    pub enabled: bool,
    /// `handleClose` (`:33` — default `null`): the corridor factory deciding when the
    /// popup closes after the cursor leaves the trigger.
    pub handle_close: Option<HandleClose>,
    /// `restMs` (`:34` — default `0`): wait for the cursor to rest before opening.
    pub rest_ms: RestMsInput,
    /// `delay` (`:35` — default `0`): wait before changing the open state.
    pub delay: DelayInput,
    /// `move` (`:36` — default `true`): whether a trigger `mousemove` re-arms the
    /// enter path (the `{ once: true }` listener, `:380`).
    pub move_: bool,
    /// `mouseOnly` (`:37` — default `false`).
    pub mouse_only: bool,
    /// `externalTree` (`:38`): the explicit floating tree overriding the ambient
    /// context.
    pub external_tree: Option<SharedFloatingTreeStore>,
    /// `isActiveTrigger` (`:45` — default `true`): whether this call controls the
    /// active trigger; `false` syncs nothing into the shared instance and falls back
    /// to no trigger for the native listeners (`:158-161`, `:210-212`).
    pub is_active_trigger: bool,
    /// `triggerElementRef` (`:46` — default an empty ref): the active trigger when the
    /// native listeners live on a wrapper.
    pub trigger_element_ref: Rc<RefCell<Option<Element>>>,
    /// `getHandleCloseContext` (`:47`): the corridor-context supplier used when the
    /// store's `dataRef.current.floatingContext` is not stamped yet
    /// (`:317`).
    pub get_handle_close_context: Option<Rc<dyn Fn() -> Option<HandleCloseContextBase>>>,
    /// `isClosing` (`:48`): an owner's in-progress-close report overriding the store's
    /// `transitionStatus === 'ending'` check (`:262`).
    pub is_closing: Option<Rc<dyn Fn() -> bool>>,
    /// `shouldOpen` (`:53`): the veto read before every hover-driven open attempt
    /// (`:114-116`).
    pub should_open: Option<Rc<dyn Fn() -> bool>>,
    /// `guardStaleOpen` (`:65` — default `false`): the Chrome dropped-`mouseleave`
    /// backup cancelling a pending open from the trigger's `mouseout` (`:364-376`).
    pub guard_stale_open: bool,
}

// Handwritten rather than derived: `Rc<RefCell<Option<Element>>>` and the option
// closures have no meaningful derived default, and several defaults differ from the
// derived `bool` (`enabled`/`is_active_trigger`/`move_` are `true`).
impl Default for UseHoverReferenceInteractionProps {
    fn default() -> Self {
        Self {
            enabled: true,
            handle_close: None,
            rest_ms: RestMsInput::default(),
            delay: DelayInput::default(),
            move_: true,
            mouse_only: false,
            external_tree: None,
            is_active_trigger: true,
            trigger_element_ref: Rc::new(RefCell::new(None)),
            get_handle_close_context: None,
            is_closing: None,
            should_open: None,
            guard_stale_open: false,
        }
    }
}

/// Port of `useHoverReferenceInteraction(context, props)`
/// (`useHoverReferenceInteraction.ts:74-511`). Must be called inside a reactive owner
/// (the listeners and timers register cleanups). Returns the flat handler bag to
/// attach to the wrapper/trigger element — `None` when disabled (`:416-419`).
pub fn use_hover_reference_interaction(
    context: impl Into<FloatingContextSource>,
    props: UseHoverReferenceInteractionProps,
) -> Option<ElementHandlers> {
    let UseHoverReferenceInteractionProps {
        enabled,
        handle_close,
        rest_ms,
        delay,
        move_,
        mouse_only,
        external_tree,
        is_active_trigger,
        trigger_element_ref,
        get_handle_close_context,
        is_closing,
        should_open,
        guard_stale_open,
    } = props;

    // `'rootStore' in context ? context.rootStore : context` (`:94`).
    let store: Rc<FloatingRootStore> = context.into().root_store();
    let data_ref = Rc::clone(&store.context.data_ref);
    let events = Rc::clone(&store.context.events);

    // `const tree = useFloatingTree(externalTree);` (`:98`).
    let tree = use_floating_tree(external_tree);
    // `const instance = useHoverInteractionSharedState(store);` (`:100`).
    let instance = use_hover_interaction_shared_state(Rc::clone(&store));
    // `const isHoverCloseActiveRef = React.useRef(false);` (`:101`).
    let is_hover_close_active = Rc::new(Cell::new(false));

    // The render-phase `handleCloseOptions` sync (`:158-161`): only the active
    // trigger's call writes the shared instance.
    if is_active_trigger {
        *instance.handle_close_options.borrow_mut() =
            handle_close.as_ref().map(|handle_close| handle_close.options.clone());
    }

    // `isClickLikeOpenEvent` (`:110-112`).
    let is_click_like_open_event: Rc<dyn Fn() -> bool> = {
        let data_ref = Rc::clone(&data_ref);
        let instance = Rc::clone(&instance);
        Rc::new(move || {
            is_click_like_open_event(
                data_ref
                    .borrow()
                    .open_event
                    .as_ref()
                    .map(|event| event.type_())
                    .as_deref(),
                instance.interacted_inside.get(),
            )
        })
    };

    // `checkShouldOpen` (`:114-116`): `shouldOpenRef.current?.() !== false`.
    let check_should_open: Rc<dyn Fn() -> bool> = {
        let should_open = should_open.clone();
        Rc::new(move || should_open.as_ref().map(|should_open| should_open()).unwrap_or(true))
    };

    // `isOverInactiveTrigger` (`:118-142`): the trigger-map fast path, then the
    // delegated/wrapper fallback walking the map for a trigger containing the target.
    let is_over_inactive_trigger: Rc<dyn Fn(Option<&Element>, &Element, Option<&EventTarget>) -> bool> =
        {
            let store = Rc::clone(&store);
            Rc::new(
                move |current_dom_reference: Option<&Element>,
                      current_target: &Element,
                      target: Option<&EventTarget>| {
                    let all_triggers = &store.context.trigger_elements;

                    // Fast path for normal usage where handlers are attached directly
                    // to triggers (`:126-129`).
                    if all_triggers.has_element(current_target) {
                        return current_dom_reference
                            .map(|dom_reference| {
                                !contains(Some(dom_reference), Some(current_target))
                            })
                            .unwrap_or(true);
                    }

                    // Fallback for delegated/wrapper usage where currentTarget may be
                    // outside the trigger map (`:131-141`).
                    let Some(target_element) = target.and_then(|target| target.dyn_ref::<Element>())
                    else {
                        return false;
                    };

                    all_triggers
                        .has_matching_element(|trigger| {
                            contains(Some(trigger), Some(target_element))
                        })
                        && current_dom_reference
                            .map(|dom_reference| {
                                !contains(Some(dom_reference), Some(target_element))
                            })
                            .unwrap_or(true)
                },
            )
        };

    // The document-level `mousemove` subscription for the corridor handler — the
    // unsubscribe-handle stand-in for `doc.removeEventListener('mousemove',
    // instance.handler)` (`:144-152`).
    let doc_mouse_move_unsubscribe: Rc<RefCell<Option<EventListenerUnsubscribe>>> =
        Rc::new(RefCell::new(None));

    // `cleanupMouseMoveHandler` (`:144-152`).
    let cleanup_mouse_move_handler: Rc<dyn Fn()> = {
        let instance = Rc::clone(&instance);
        let unsubscribe_handle = Rc::clone(&doc_mouse_move_unsubscribe);
        Rc::new(move || {
            if instance.handler.borrow().is_some() {
                if let Some(unsubscribe) = unsubscribe_handle.borrow_mut().take() {
                    unsubscribe.unsubscribe();
                }
                *instance.handler.borrow_mut() = None;
            }
        })
    };

    // `clearPointerEvents` (`:154-156`).
    let clear_pointer_events: Rc<dyn Fn()> = {
        let instance = Rc::clone(&instance);
        Rc::new(move || clear_safe_polygon_pointer_events_mutation(&instance))
    };

    // The unmount cleanup (`:163`).
    {
        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
        let cleanup = send_wrapper::SendWrapper::new(cleanup_mouse_move_handler);
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }

    // The openchange listener (`:167-189`): a close records the hover-close state,
    // cancels the pending timers and re-arms the move block; an open clears the
    // hover-close marker. The deps hold only stable handles, so the subscription
    // registers once.
    if enabled {
        let instance = Rc::clone(&instance);
        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
        let is_hover_close_active = Rc::clone(&is_hover_close_active);
        let unsubscribe = events.on(
            "openchange",
            Rc::new(
                move |details: &crate::floating_ui::types::FloatingUIOpenChangeDetails| {
                    if !details.open {
                        is_hover_close_active.set(details.reason == reasons::TRIGGER_HOVER);
                        cleanup_mouse_move_handler();
                        instance.open_change_timeout.clear();
                        instance.rest_timeout.clear();
                        instance.block_mouse_move.set(true);
                        instance.rest_timeout_pending.set(false);
                    } else {
                        is_hover_close_active.set(false);
                    }
                },
            ),
        );
        let unsubscribe = send_wrapper::SendWrapper::new(unsubscribe);
        reactive_graph::owner::on_cleanup(move || unsubscribe());
    }

    // The trigger/floating listener effect (`:191-414`) — the core hover wiring. The
    // deps hold only stable handles and refs read once per run, so the port's single
    // run matches upstream's lifecycle (mount + `enabled` flips; the props are
    // per-call constants).
    {
        let handle_close = handle_close.clone();
        let rest_ms = rest_ms.clone();
        let delay = delay.clone();
        let store = Rc::clone(&store);
        let data_ref = Rc::clone(&data_ref);
        let tree = tree.clone();
        let instance = Rc::clone(&instance);
        let is_hover_close_active = Rc::clone(&is_hover_close_active);
        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
        let check_should_open = Rc::clone(&check_should_open);
        let is_over_inactive_trigger = Rc::clone(&is_over_inactive_trigger);
        let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        let doc_mouse_move_unsubscribe = Rc::clone(&doc_mouse_move_unsubscribe);
        let trigger_element_ref = Rc::clone(&trigger_element_ref);
        use_iso_layout_effect(move || {
            if !enabled {
                return;
            }

            // `closeWithDelay` (`:196-208`).
            let close_with_delay: Rc<dyn Fn(&MouseEvent, bool)> = {
                let delay = delay.clone();
                let instance = Rc::clone(&instance);
                let store = Rc::clone(&store);
                let tree = tree.clone();
                Rc::new(move |event: &MouseEvent, run_else_branch: bool| {
                    let close_delay =
                        get_delay(&delay, "close", instance.pointer_type.borrow().as_deref());
                    let close = {
                        let store = Rc::clone(&store);
                        let tree = tree.clone();
                        let event = event.clone();
                        move || {
                            store.set_open(
                                false,
                                &RootOpenChangeEventDetails::new(
                                    reasons::TRIGGER_HOVER,
                                    event.clone().into(),
                                    None,
                                    String::new(),
                                ),
                            );
                            if let Some(tree) = tree.as_ref() {
                                tree.events.emit(
                                    "floating.closed",
                                    &FloatingTreeEvent::FloatingClosed(event.clone()),
                                );
                            }
                        }
                    };
                    if close_delay > 0 {
                        instance.open_change_timeout.start(close_delay, close);
                    } else if run_else_branch {
                        instance.open_change_timeout.clear();
                        close();
                    }
                })
            };

            // The trigger resolution (`:210-215`): the wrapper's `triggerElementRef`
            // wins; otherwise the active trigger falls back to the store's DOM
            // reference.
            let trigger: Option<Element> = trigger_element_ref
                .borrow()
                .clone()
                .or_else(|| {
                    if is_active_trigger {
                        selectors::dom_reference_element(&store.get_snapshot())
                    } else {
                        None
                    }
                });
            let Some(trigger) = trigger else {
                return;
            };

            // `onMouseEnter` (`:218-302`).
            let on_mouse_enter = {
                let instance = Rc::clone(&instance);
                let store = Rc::clone(&store);
                let rest_ms = rest_ms.clone();
                let delay = delay.clone();
                let is_over_inactive_trigger = Rc::clone(&is_over_inactive_trigger);
                let check_should_open = Rc::clone(&check_should_open);
                let is_closing = is_closing.clone();
                let is_hover_close_active = Rc::clone(&is_hover_close_active);
                move |event: &MouseEvent| {
                    instance.open_change_timeout.clear();
                    instance.block_mouse_move.set(false);

                    if mouse_only
                        && !is_mouse_like_pointer_type(instance.pointer_type.borrow().as_deref(), false)
                    {
                        return;
                    }

                    // Only rest delay is set; there's no fallback delay. This will be
                    // handled by `onMouseMove` (`:226-227`).
                    let rest_ms_value = get_rest_ms(&rest_ms);
                    let open_delay =
                        get_delay(&delay, "open", instance.pointer_type.borrow().as_deref());
                    let event_target = get_target(event);
                    let current_target: Option<Element> = event
                        .current_target()
                        .and_then(|target| target.dyn_into::<Element>().ok());
                    let current_dom_reference =
                        selectors::dom_reference_element(&store.get_snapshot());
                    let mut trigger_node: Option<Element> = current_target.clone();

                    // Wrapper/delegated mode: resolve the actual trigger from the
                    // event target (`:235-243`).
                    if let Some(event_target_element) =
                        event_target.as_ref().and_then(|target| target.dyn_ref::<Element>())
                    {
                        if !store.context.trigger_elements.has_element(event_target_element) {
                            let mut found = false;
                            store.context.trigger_elements.for_each_element(
                                |trigger_element| {
                                    if !found
                                        && contains(
                                            Some(trigger_element),
                                            Some(event_target_element),
                                        )
                                    {
                                        trigger_node = Some(trigger_element.clone());
                                        found = true;
                                    }
                                },
                            );
                        }
                    }

                    // Wrapper/delegated mode fallback: if the wrapper contains the
                    // active trigger, treat this as re-entering that active trigger
                    // (`:245-254`).
                    if let (Some(current_target), Some(current_dom_reference)) =
                        (current_target.as_ref(), current_dom_reference.as_ref())
                    {
                        if !store.context.trigger_elements.has_element(current_target)
                            && contains(Some(current_target), Some(current_dom_reference))
                        {
                            trigger_node = Some(current_dom_reference.clone());
                        }
                    }

                    let is_over_inactive = match trigger_node.as_ref() {
                        Some(trigger_node) => is_over_inactive_trigger(
                            current_dom_reference.as_ref(),
                            trigger_node,
                            event_target.as_ref(),
                        ),
                        None => false,
                    };
                    let is_open = store.select(selectors::open);
                    let is_in_closing_transition = is_closing
                        .as_ref()
                        .map(|is_closing| is_closing())
                        .unwrap_or_else(|| {
                            store.select(selectors::transition_status)
                                == Some(TransitionStatus::Ending)
                        });
                    let is_hover_close_transition =
                        !is_open && is_in_closing_transition && is_hover_close_active.get();
                    let is_reentering_same_trigger_during_close_transition =
                        !is_over_inactive
                            && trigger_node
                                .as_ref()
                                .zip(current_dom_reference.as_ref())
                                .map(|(trigger_node, dom_reference)| {
                                    contains(Some(dom_reference), Some(trigger_node))
                                })
                                .unwrap_or(false)
                            && is_hover_close_transition;
                    let is_rest_only_delay = rest_ms_value > 0 && open_delay == 0;
                    let should_open_immediately = (is_over_inactive
                        && (is_open || is_hover_close_transition))
                        || is_reentering_same_trigger_during_close_transition;

                    let should_open = !is_open || is_over_inactive;

                    // Open immediately when moving between triggers while open, or
                    // during a hover-driven close transition (including same-trigger
                    // re-entry) (`:278-285`).
                    if should_open_immediately {
                        if check_should_open() {
                            store.set_open(
                                true,
                                &RootOpenChangeEventDetails::new(
                                    reasons::TRIGGER_HOVER,
                                    event.clone().into(),
                                    trigger_node,
                                    String::new(),
                                ),
                            );
                        }
                        return;
                    }

                    if is_rest_only_delay {
                        return;
                    }

                    if open_delay > 0 {
                        let instance = Rc::clone(&instance);
                        let store = Rc::clone(&store);
                        let check_should_open = Rc::clone(&check_should_open);
                        let event = event.clone();
                        instance.open_change_timeout.start(open_delay, move || {
                            if should_open && check_should_open() {
                                store.set_open(
                                    true,
                                    &RootOpenChangeEventDetails::new(
                                        reasons::TRIGGER_HOVER,
                                        event.clone().into(),
                                        trigger_node.clone(),
                                        String::new(),
                                    ),
                                );
                            }
                        });
                    } else if should_open {
                        if check_should_open() {
                            store.set_open(
                                true,
                                &RootOpenChangeEventDetails::new(
                                    reasons::TRIGGER_HOVER,
                                    event.clone().into(),
                                    trigger_node,
                                    String::new(),
                                ),
                            );
                        }
                    }
                }
            };

            // `onMouseLeave` (`:304-362`).
            let on_mouse_leave = {
                let instance = Rc::clone(&instance);
                let store = Rc::clone(&store);
                let data_ref = Rc::clone(&data_ref);
                let tree = tree.clone();
                let handle_close = handle_close.clone();
                let get_handle_close_context = get_handle_close_context.clone();
                let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                let cleanup_mouse_move_handler = Rc::clone(&cleanup_mouse_move_handler);
                let clear_pointer_events = Rc::clone(&clear_pointer_events);
                let doc_mouse_move_unsubscribe = Rc::clone(&doc_mouse_move_unsubscribe);
                let trigger_element_ref = Rc::clone(&trigger_element_ref);
                let close_with_delay = Rc::clone(&close_with_delay);
                move |event: &MouseEvent| {
                    if is_click_like_open_event() {
                        clear_pointer_events();
                        return;
                    }

                    cleanup_mouse_move_handler();

                    let dom_reference_element =
                        selectors::dom_reference_element(&store.get_snapshot());
                    let doc = owner_document(
                        dom_reference_element
                            .as_ref()
                            .map(|element| element.as_ref() as &web_sys::Node),
                    );
                    instance.rest_timeout.clear();
                    instance.rest_timeout_pending.set(false);

                    // `dataRef.current.floatingContext ?? getHandleCloseContext?.()`
                    // (`:317`): the stashed context's live members build the base, the
                    // consumer supplier is the fallback.
                    let handle_close_context_base: Option<HandleCloseContextBase> = {
                        let data = data_ref.borrow();
                        data.floating_context.as_ref().map(|floating_context| {
                            HandleCloseContextBase {
                                placement: Some(floating_context.placement.get_untracked()),
                                dom_reference: floating_context
                                    .elements
                                    .dom_reference
                                    .get_untracked(),
                                floating: floating_context.elements.floating.get_untracked(),
                                node_id: data.floating_node_id.clone(),
                                leave: None,
                            }
                        })
                    }
                    .or_else(|| {
                        get_handle_close_context
                            .as_ref()
                            .and_then(|get_handle_close_context| get_handle_close_context())
                    });

                    if is_inside_enabled_trigger(
                        event.related_target().as_ref(),
                        &store.context.trigger_elements,
                    ) {
                        return;
                    }

                    if let (Some(handle_close), Some(base)) =
                        (handle_close.as_ref(), handle_close_context_base)
                    {
                        // Prevent clearing the pending open timer (`:324-326`).
                        if !store.select(selectors::open) {
                            instance.open_change_timeout.clear();
                        }

                        let current_trigger = trigger_element_ref.borrow().clone();

                        let on_close: Rc<dyn Fn()> = {
                            let clear_pointer_events = Rc::clone(&clear_pointer_events);
                            let cleanup_mouse_move_handler =
                                Rc::clone(&cleanup_mouse_move_handler);
                            let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                            let store = Rc::clone(&store);
                            let close_with_delay = Rc::clone(&close_with_delay);
                            let event = event.clone();
                            Rc::new(move || {
                                clear_pointer_events();
                                cleanup_mouse_move_handler();
                                if enabled
                                    && !is_click_like_open_event()
                                    && current_trigger.as_ref()
                                        == selectors::dom_reference_element(&store.get_snapshot())
                                            .as_ref()
                                {
                                    close_with_delay(&event, true);
                                }
                            })
                        };

                        let handler = (handle_close.factory)(&HandleCloseContext {
                            x: Some(event.client_x() as f64),
                            y: Some(event.client_y() as f64),
                            base,
                            on_close,
                            tree: tree.clone(),
                        });

                        *instance.handler.borrow_mut() = Some(Rc::clone(&handler));
                        let handler_for_listener = Rc::clone(&handler);
                        let unsubscribe = leptos_ui_utils::add_event_listener(
                            &doc,
                            "mousemove",
                            move |event: &Event| {
                                if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                                    handler_for_listener(mouse_event);
                                }
                            },
                        );
                        *doc_mouse_move_unsubscribe.borrow_mut() = Some(unsubscribe);
                        // The handler runs immediately for the leave event itself
                        // (`:349`).
                        handler(event);

                        return;
                    }

                    // Allow interactivity without `safePolygon` on touch devices
                    // (`:354-357`).
                    let should_close =
                        if instance.pointer_type.borrow().as_deref() == Some("touch") {
                            let related: Option<Element> = event
                                .related_target()
                                .and_then(|related| related.dyn_into::<Element>().ok());
                            let floating = selectors::floating_element(&store.get_snapshot());
                            !contains(floating.as_ref(), related.as_ref())
                        } else {
                            true
                        };

                    if should_close {
                        close_with_delay(event, true);
                    }
                }
            };

            // The backup cancellation for Chrome's dropped `mouseleave` (`:364-372`).
            let on_mouse_out = {
                let instance = Rc::clone(&instance);
                let trigger = trigger.clone();
                move |event: &MouseEvent| {
                    let related: Option<Element> = event
                        .related_target()
                        .and_then(|related| related.dyn_ref::<Element>().cloned());
                    if contains(Some(&trigger), related.as_ref()) {
                        return; // moved within the trigger's own subtree
                    }
                    instance.open_change_timeout.clear();
                    instance.rest_timeout.clear();
                    instance.rest_timeout_pending.set(false);
                }
            };

            // The listener fan-out (`:374-391`).
            let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

            if guard_stale_open {
                let on_mouse_out = on_mouse_out.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    &trigger,
                    "mouseout",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_mouse_out(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            if move_ {
                let on_mouse_enter = on_mouse_enter.clone();
                let once_cleanup = add_event_listener_once(
                    trigger.as_ref(),
                    "mousemove",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_mouse_enter(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(once_cleanup)));
            }

            {
                let on_mouse_enter = on_mouse_enter.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    &trigger,
                    "mouseenter",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_mouse_enter(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            {
                let on_mouse_leave = on_mouse_leave.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    &trigger,
                    "mouseleave",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_mouse_leave(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            let merged = merge_cleanups(cleanups);
            let merged = send_wrapper::SendWrapper::new(merged);
            reactive_graph::owner::on_cleanup(move || merged.take()());
        });
    }

    // The returned bag (`:416-510`).
    if !enabled {
        return None;
    }

    // `setPointerRef` (`:421-423`) — shared by onPointerDown and onPointerEnter.
    let set_pointer_type: ElementEventHandler<PointerEvent> = {
        let instance = Rc::clone(&instance);
        Rc::new(move |event: &PointerEvent| {
            *instance.pointer_type.borrow_mut() = Some(event.pointer_type());
        })
    };

    // `onMouseMove` (`:428-499`) — the rest-ms engine plus the corridor
    // pointer-events mutation.
    let on_mouse_move: ElementEventHandler<MouseEvent> = {
        let instance = Rc::clone(&instance);
        let store = Rc::clone(&store);
        let rest_ms = rest_ms.clone();
        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
        let check_should_open = Rc::clone(&check_should_open);
        let is_over_inactive_trigger = Rc::clone(&is_over_inactive_trigger);
        Rc::new(move |event: &MouseEvent| {
            let native_event = event.clone();
            let trigger: Option<Element> = event
                .current_target()
                .and_then(|target| target.dyn_into::<Element>().ok());

            let current_dom_reference = selectors::dom_reference_element(&store.get_snapshot());
            let current_open = store.select(selectors::open);
            let target = event.target();
            let is_over_inactive = trigger
                .as_ref()
                .map(|trigger| {
                    is_over_inactive_trigger(
                        current_dom_reference.as_ref(),
                        trigger,
                        target.as_ref(),
                    )
                })
                .unwrap_or(false);

            if mouse_only
                && !is_mouse_like_pointer_type(instance.pointer_type.borrow().as_deref(), false)
            {
                return;
            }

            // The corridor pointer-events mutation while hovering an inactive trigger
            // of an open popup (`:440-453`).
            if current_open
                && is_over_inactive
                && instance
                    .handle_close_options
                    .borrow()
                    .as_ref()
                    .map(|options| options.block_pointer_events)
                    .unwrap_or(false)
            {
                let floating_element = selectors::floating_element(&store.get_snapshot());
                if let (Some(floating_element), Some(trigger)) =
                    (floating_element, trigger.as_ref())
                {
                    let scope_element = instance
                        .handle_close_options
                        .borrow()
                        .as_ref()
                        .and_then(|options| {
                            options.get_scope.as_ref().and_then(|get_scope| get_scope())
                        })
                        .or_else(|| {
                            trigger
                                .owner_document()
                                .and_then(|document| document.body())
                                .map(|body| body.into())
                        });

                    if let Some(scope_element) = scope_element {
                        apply_safe_polygon_pointer_events_mutation(
                            &instance,
                            &scope_element,
                            trigger,
                            &floating_element,
                        );
                    }
                }
            }

            let rest_ms_value = get_rest_ms(&rest_ms);
            if (current_open && !is_over_inactive) || rest_ms_value == 0 {
                return;
            }

            // Ignore insignificant movements to account for tremors (`:460-466`).
            if !is_over_inactive
                && instance.rest_timeout_pending.get()
                && (event.movement_x() as f64).powi(2) + (event.movement_y() as f64).powi(2) < 2.0
            {
                return;
            }

            instance.rest_timeout.clear();

            let handle_mouse_move: Rc<dyn Fn()> = {
                let instance = Rc::clone(&instance);
                let store = Rc::clone(&store);
                let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                let check_should_open = Rc::clone(&check_should_open);
                let is_over_inactive = is_over_inactive;
                let native_event = native_event.clone();
                let trigger = trigger.clone();
                Rc::new(move || {
                    instance.rest_timeout_pending.set(false);

                    // A delayed hover open should not override a click-like open that
                    // happened while the hover delay was pending (`:473-477`).
                    if is_click_like_open_event() {
                        return;
                    }

                    let latest_open = store.select(selectors::open);

                    if !instance.block_mouse_move.get()
                        && (!latest_open || is_over_inactive)
                        && check_should_open()
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

            if instance.pointer_type.borrow().as_deref() == Some("touch") {
                // The flushSync path (`:489-492`) — the open runs directly.
                handle_mouse_move();
            } else if is_over_inactive && current_open {
                handle_mouse_move();
            } else {
                instance.rest_timeout_pending.set(true);
                let handle_mouse_move = Rc::clone(&handle_mouse_move);
                instance
                    .rest_timeout
                    .start(rest_ms_value, move || handle_mouse_move());
            }
        })
    };

    Some(ElementHandlers {
        on_pointer_down: Some(Rc::clone(&set_pointer_type)),
        on_pointer_enter: Some(set_pointer_type),
        on_mouse_move: Some(on_mouse_move),
        ..ElementHandlers::default()
    })
}

// The hook needs a DOM realm (elements, native listeners, timers): the wasm suite
// mirrors `packages/react/src/floating-ui-react/hooks/useHoverReferenceInteraction.
// test.tsx`.
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
    /// (`useHoverReferenceInteraction.test.tsx:38-46` — `useFloating({ open,
    /// onOpenChange })` with the callback syncing back into the store): the port
    /// writes the store's open state via a Weak (the `useHover` test adaptation).
    fn store_with(
        open: bool,
        log: CallLog,
        trigger_elements: PopupTriggerMap,
    ) -> Rc<FloatingRootStore> {
        let store = FloatingRootStore::new(FloatingRootStoreOptions {
            open,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements,
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

    fn element(tag: &str) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element(tag).unwrap().dyn_into().unwrap()
    }

    /// The wrapper/trigger/floating fixture the upstream tests build
    /// (`useHoverReferenceInteraction.test.tsx:57-69`): the trigger is the store's
    /// reference and `triggerElementRef` target, the wrapper is the element the
    /// returned bag attaches to, the floating element is registered in the store.
    /// `nested_trigger` places the trigger inside the wrapper (tests 1-3, 5-6) or
    /// beside it (test 4).
    struct Fixture {
        store: Rc<FloatingRootStore>,
        trigger: web_sys::HtmlElement,
        wrapper: web_sys::HtmlElement,
    }

    fn fixture(open: bool, log: CallLog, nested_trigger: bool) -> Fixture {
        let trigger = element("button");
        let wrapper = element("div");
        let floating = element("div");
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        body.append_child(&wrapper).unwrap();
        if nested_trigger {
            wrapper.append_child(&trigger).unwrap();
        } else {
            body.append_child(&trigger).unwrap();
        }
        body.append_child(&floating).unwrap();

        let store = store_with(open, log, PopupTriggerMap::new());
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(trigger.clone().into())),
        );
        store.set_field(
            |state| &mut state.dom_reference_element,
            Some(trigger.clone().into()),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.into()));
        Fixture {
            store,
            trigger,
            wrapper,
        }
    }

    fn attach(
        props: &Option<ElementHandlers>,
        wrapper: &web_sys::HtmlElement,
    ) -> leptos_ui_utils::merge_cleanups::CleanupFn {
        props
            .as_ref()
            .expect("the hook returned a handler bag")
            .attach_to(wrapper.as_ref())
            .expect("slots attached")
    }

    fn calls(log: &CallLog) -> Vec<(bool, String)> {
        log.borrow().clone()
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

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }

    // `does not treat child target as inactive when handlers are on a wrapper`
    // (`useHoverReferenceInteraction.test.tsx:34-86`): with the trigger nested inside
    // the wrapper, moving over the active trigger emits no redundant openchange.
    #[wasm_bindgen_test(async)]
    async fn does_not_treat_child_target_as_inactive_when_handlers_are_on_a_wrapper() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let fixture = fixture(true, Rc::clone(&log), true);
            let trigger_element_ref = Rc::new(RefCell::new(Some(fixture.trigger.clone().into())));

            let props = use_hover_reference_interaction(
                Rc::clone(&fixture.store),
                UseHoverReferenceInteractionProps {
                    mouse_only: true,
                    rest_ms: RestMsInput::Value(100),
                    delay: DelayInput::Value(Delay::Partial {
                        open: None,
                        close: Some(0),
                    }),
                    move_: false,
                    trigger_element_ref,
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let _cleanup = attach(&props, &fixture.wrapper);

            fixture
                .wrapper
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            fixture
                .trigger
                .dispatch_event(&mouse_move_with_movement(10, 0))
                .unwrap();

            sleep(10).await;
            assert!(
                calls(&log).is_empty(),
                "moving over the active trigger emits no redundant openchange: {:?}",
                calls(&log)
            );
            assert!(fixture.store.select(selectors::open), "the popup stays open");
        };
        __owner.cleanup();
    }

    // `does not treat a synthetic child target as inactive when the native path
    // differs` (`useHoverReferenceInteraction.test.tsx:88-158`): a bubbled `mousemove`
    // from a child of the trigger resolves through the trigger-map fast path and
    // produces no openchange.
    #[wasm_bindgen_test(async)]
    async fn does_not_treat_a_synthetic_child_target_as_inactive() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let fixture = fixture(true, Rc::clone(&log), true);
            let child = element("span");
            fixture.trigger.append_child(&child).unwrap();
            let trigger_element_ref = Rc::new(RefCell::new(Some(fixture.trigger.clone().into())));

            let props = use_hover_reference_interaction(
                Rc::clone(&fixture.store),
                UseHoverReferenceInteractionProps {
                    mouse_only: true,
                    rest_ms: RestMsInput::Value(100),
                    delay: DelayInput::Value(Delay::Partial {
                        open: None,
                        close: Some(0),
                    }),
                    move_: false,
                    trigger_element_ref,
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let _cleanup = attach(&props, &fixture.wrapper);

            fixture
                .wrapper
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            child.dispatch_event(&mouse_move_with_movement(10, 0)).unwrap();

            sleep(10).await;
            assert!(
                calls(&log).is_empty(),
                "the skewed child target produces no openchange: {:?}",
                calls(&log)
            );
            assert!(fixture.store.select(selectors::open), "the popup stays open");
        };
        __owner.cleanup();
    }

    // `treats disabled child trigger as inactive in wrapper fallback mode`
    // (`useHoverReferenceInteraction.test.tsx:160-222`): hovering a trigger registered
    // as disabled while the popup is open counts as over-an-inactive-trigger — exactly
    // one openchange, the popup staying open. The native enter listeners live on the
    // active trigger (the `triggerElementRef`), the bag on the inactive wrapper.
    #[wasm_bindgen_test(async)]
    async fn treats_disabled_child_trigger_as_inactive_in_wrapper_fallback_mode() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));

            let active_trigger = element("button");
            let wrapper = element("div");
            let disabled_trigger = element("button");
            let floating = element("div");
            disabled_trigger
                .set_attribute("data-trigger-disabled", "")
                .unwrap();

            let document = web_sys::window().unwrap().document().unwrap();
            let body = document.body().unwrap();
            body.append_child(&active_trigger).unwrap();
            body.append_child(&wrapper).unwrap();
            wrapper.append_child(&disabled_trigger).unwrap();
            body.append_child(&floating).unwrap();

            // `context.rootStore.context.triggerElements.add('disabled-trigger', node)`
            // (`useHoverReferenceInteraction.test.tsx:195-199`) — the port registers
            // before the store construction (the map is handed to the store at
            // construction).
            let mut trigger_elements = PopupTriggerMap::new();
            trigger_elements.add("disabled-trigger", disabled_trigger.clone().into());
            let store = store_with(true, Rc::clone(&log), trigger_elements);
            store.set_field(
                |state| &mut state.reference_element,
                Some(ReferenceType::Element(active_trigger.clone().into())),
            );
            store.set_field(
                |state| &mut state.dom_reference_element,
                Some(active_trigger.clone().into()),
            );
            store.set_field(|state| &mut state.floating_element, Some(floating.into()));

            let props = use_hover_reference_interaction(
                Rc::clone(&store),
                UseHoverReferenceInteractionProps {
                    mouse_only: true,
                    rest_ms: RestMsInput::Value(100),
                    delay: DelayInput::Value(Delay::Partial {
                        open: None,
                        close: Some(0),
                    }),
                    move_: false,
                    trigger_element_ref: Rc::new(RefCell::new(Some(
                        active_trigger.clone().into(),
                    ))),
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let _cleanup = attach(&props, &wrapper);

            active_trigger
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            wrapper
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();
            disabled_trigger
                .dispatch_event(&mouse_move_with_movement(10, 0))
                .unwrap();

            sleep(10).await;
            assert_eq!(
                calls(&log),
                vec![(true, reasons::TRIGGER_HOVER.to_owned())],
                "exactly one openchange while the popup remains open"
            );
            assert!(store.select(selectors::open), "the tooltip remains open");
        };
        __owner.cleanup();
    }

    // `reopens immediately for same trigger in delegated wrapper mode during close
    // transition` (`useHoverReferenceInteraction.test.tsx:224-299`): re-entering the
    // same trigger during an active hover close transition reopens immediately,
    // bypassing the open delay.
    #[wasm_bindgen_test(async)]
    async fn reopens_immediately_for_same_trigger_during_close_transition() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let fixture = fixture(true, Rc::clone(&log), true);
            let trigger_element_ref = Rc::new(RefCell::new(Some(fixture.wrapper.clone().into())));

            let props = use_hover_reference_interaction(
                Rc::clone(&fixture.store),
                UseHoverReferenceInteractionProps {
                    mouse_only: true,
                    move_: false,
                    delay: DelayInput::Value(Delay::Partial {
                        open: Some(500),
                        close: Some(0),
                    }),
                    trigger_element_ref,
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let _cleanup = attach(&props, &fixture.wrapper);

            // `closeFromHover` (`useHoverReferenceInteraction.test.tsx:239-244`), then
            // the render's `transitionStatus = open ? undefined : 'ending'`
            // (`:246-248`).
            fixture.store.set_open(
                false,
                &RootOpenChangeEventDetails::new(
                    reasons::TRIGGER_HOVER,
                    mouse_event("mouseleave", None).into(),
                    None,
                    String::new(),
                ),
            );
            fixture.store.update(|state, _| {
                state.transition_status = Some(TransitionStatus::Ending);
                true
            });
            assert_eq!(
                calls(&log),
                vec![(false, reasons::TRIGGER_HOVER.to_owned())],
                "the popup closed from hover"
            );

            fixture
                .wrapper
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();

            sleep(10).await;
            assert_eq!(
                calls(&log),
                vec![
                    (false, reasons::TRIGGER_HOVER.to_owned()),
                    (true, reasons::TRIGGER_HOVER.to_owned()),
                ],
                "close from hover + immediate reopen without waiting the open delay"
            );
            assert!(
                fixture.store.select(selectors::open),
                "the tooltip re-rendered"
            );
        };
        __owner.cleanup();
    }

    // `reopens immediately when close transition state is provided externally`
    // (`useHoverReferenceInteraction.test.tsx:301-373`): the same immediate reopen via
    // the `isClosing` owner report instead of the store's transition status.
    #[wasm_bindgen_test(async)]
    async fn reopens_immediately_when_close_transition_state_is_provided_externally() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let fixture = fixture(true, Rc::clone(&log), true);
            let trigger_element_ref = Rc::new(RefCell::new(Some(fixture.wrapper.clone().into())));

            let is_closing_store = Rc::clone(&fixture.store);
            let props = use_hover_reference_interaction(
                Rc::clone(&fixture.store),
                UseHoverReferenceInteractionProps {
                    mouse_only: true,
                    move_: false,
                    delay: DelayInput::Value(Delay::Partial {
                        open: Some(500),
                        close: Some(0),
                    }),
                    trigger_element_ref,
                    is_closing: Some(Rc::new(move || {
                        !is_closing_store.select(selectors::open)
                    })),
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let _cleanup = attach(&props, &fixture.wrapper);

            fixture.store.set_open(
                false,
                &RootOpenChangeEventDetails::new(
                    reasons::TRIGGER_HOVER,
                    mouse_event("mouseleave", None).into(),
                    None,
                    String::new(),
                ),
            );
            assert_eq!(
                calls(&log),
                vec![(false, reasons::TRIGGER_HOVER.to_owned())],
                "the popup closed from hover"
            );

            fixture
                .wrapper
                .dispatch_event(&mouse_event("mouseenter", None))
                .unwrap();

            sleep(10).await;
            assert_eq!(
                calls(&log),
                vec![
                    (false, reasons::TRIGGER_HOVER.to_owned()),
                    (true, reasons::TRIGGER_HOVER.to_owned()),
                ],
                "the externally-reported closing state also reopens immediately"
            );
        };
        __owner.cleanup();
    }
}

// The render-phase `handleCloseOptions` sync needs no DOM realm: the host suite pins
// it (upstream `updates the handleClose options during render`,
// `useHoverReferenceInteraction.test.tsx:13-32`) plus the disabled-hook return
// (`:416-419`).
#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    use reactive_graph::owner::Owner;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::safe_polygon::{SafePolygonOptions, safe_polygon};

    fn store() -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        })
    }

    fn block_options(block: bool) -> SafePolygonOptions {
        SafePolygonOptions {
            block_pointer_events: block,
            ..SafePolygonOptions::default()
        }
    }

    // The active-trigger call syncs `handle_close_options` onto the shared instance
    // (`:158-161`): the first call's `false` is replaced by the rerender's `true`, and
    // a non-active-trigger call never overwrites it.
    #[test]
    fn the_active_trigger_call_syncs_handle_close_options() {
        let store = store();

        {
            let owner = Owner::new();
            owner.set();
            let _props = use_hover_reference_interaction(
                Rc::clone(&store),
                UseHoverReferenceInteractionProps {
                    handle_close: Some(safe_polygon(block_options(false))),
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let instance = store
                .context
                .data_ref
                .borrow()
                .hover_interaction_state
                .clone()
                .expect("the first call stashed the shared instance");
            assert!(
                !instance
                    .handle_close_options
                    .borrow()
                    .as_ref()
                    .expect("the options were synced")
                    .block_pointer_events,
                "the initial sync carries the factory's block=false"
            );
            owner.cleanup();
        }

        {
            let owner = Owner::new();
            owner.set();
            let _props = use_hover_reference_interaction(
                Rc::clone(&store),
                UseHoverReferenceInteractionProps {
                    handle_close: Some(safe_polygon(block_options(true))),
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let instance = store
                .context
                .data_ref
                .borrow()
                .hover_interaction_state
                .clone()
                .expect("the rerender reuses the stashed instance");
            assert!(
                instance
                    .handle_close_options
                    .borrow()
                    .as_ref()
                    .expect("the options were synced")
                    .block_pointer_events,
                "the rerender syncs the updated options synchronously"
            );
            owner.cleanup();
        }

        {
            let owner = Owner::new();
            owner.set();
            let _props = use_hover_reference_interaction(
                Rc::clone(&store),
                UseHoverReferenceInteractionProps {
                    handle_close: Some(safe_polygon(block_options(false))),
                    is_active_trigger: false,
                    ..UseHoverReferenceInteractionProps::default()
                },
            );
            let instance = store
                .context
                .data_ref
                .borrow()
                .hover_interaction_state
                .clone()
                .expect("the instance persists");
            assert!(
                instance
                    .handle_close_options
                    .borrow()
                    .as_ref()
                    .expect("the options persist")
                    .block_pointer_events,
                "a non-active-trigger call does not overwrite the synced options"
            );
            owner.cleanup();
        }
    }

    // A disabled hook returns no bag (`:416-419`).
    #[test]
    fn a_disabled_hook_returns_no_bag() {
        let owner = Owner::new();
        owner.set();
        let store = store();
        let props = use_hover_reference_interaction(
            Rc::clone(&store),
            UseHoverReferenceInteractionProps {
                enabled: false,
                ..UseHoverReferenceInteractionProps::default()
            },
        );
        assert!(props.is_none(), "disabled returns None");
        owner.cleanup();
    }
}
