//! Port of `packages/react/src/floating-ui-react/hooks/useHoverFloatingInteraction.ts`
//! — the floating-side half of the split hover hooks: keeping the popup open while the
//! pointer is over its content, the close-delay close on leaving the floating element,
//! the corridor pointer-events scope chain, and the `floating.closed` tree chain that
//! lets a parent close after its child does (`specs/library/floating-ui-react/
//! implementation.md`, "Notable per-hook state machines": the split-pair half of
//! "Hover (3 implementations, one shared instance)"; "DOM/portal strategy": the
//! pointer-events scope fallback chain). This hook has no dedicated upstream test file
//! (`specs/library/floating-ui-react/implementation.md`, "Anything in source not
//! explained by any test" item 1) — it is exercised only indirectly by the tooltip /
//! preview-card / navigation-menu component suites — so the port's tests pin the
//! written mechanics directly.
//!
//! The reference-side half is
//! [`crate::floating_ui::use_hover_reference_interaction`]; both share one mutable
//! [`HoverInteraction`] through the store's `dataRef`.
//!
//! ## Rust adaptations
//!
//! - The `useStableCallback` closures fold into `Rc` handles (the `useDismiss`
//!   adaptation); the `useValueAsRef`/ref-mirror concerns are React render-phase
//!   specific (components run once).
//! - `tree.events.off('floating.closed', onNodeClosed)` (`:195,243,255`) ports to a
//!   shared unsubscribe handle: the port's bus removes listeners by handle rather
//!   than by function identity (see the `EventEmitter` port's docs).
//! - The `closeDelay` prop form (`number | (() => number)`, `:41`) ports to
//!   [`CloseDelayInput`], adapted into `get_delay`'s vocabulary per invocation
//!   ([`number_input_as_delay`]).
//! - `parentFloating.style.pointerEvents = ''` (`:116`) ports to the shared-state
//!   module's `remove_pointer_events` — assigning `''` removes the declaration in
//!   CSSOM, which `removeProperty` does explicitly.
//! - The effect cleanups port to `on_cleanup` registrations: the unmount-only
//!   `clearPointerEvents` (`:92-94`) registers at hook time, the per-run cleanups
//!   (the scope mutation's `:140-142` and the listeners' `:254-256`) register inside
//!   their layout effects.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, MouseEvent};

use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_timeout;

use crate::floating_ui::element::is_interactive_element;
use crate::floating_ui::element_props::FloatingContextSource;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::nodes::get_node_children;
use crate::floating_ui::reasons;
use crate::floating_ui::tree::{use_floating_parent_node_id, use_floating_tree};
use crate::floating_ui::types::{EventUnsubscribe, FloatingTreeEvent, RootOpenChangeEventDetails};
use crate::floating_ui::use_hover_interaction_shared_state::{
    apply_safe_polygon_pointer_events_mutation, clear_safe_polygon_pointer_events_mutation,
    remove_pointer_events, use_hover_interaction_shared_state,
};
use crate::floating_ui::use_hover_shared::{
    CloseDelayInput, get_delay, is_click_like_open_event, is_hover_open_event,
    number_input_as_delay,
};
use crate::floating_ui::use_hover_shared::is_inside_enabled_trigger;

/// Port of `UseHoverFloatingInteractionProps` (`useHoverFloatingInteraction.ts:29-47`)
/// with the destructured defaults (`:56`).
#[derive(Clone)]
pub struct UseHoverFloatingInteractionProps {
    /// `enabled` (`:35` — default `true`): gates every internal effect and listener.
    pub enabled: bool,
    /// `closeDelay` (`:41` — default `0`): waits before closing after the pointer
    /// leaves the floating element.
    pub close_delay: CloseDelayInput,
    /// `nodeId` (`:46`): the tree node id override for floats that participate in the
    /// tree without a `FloatingContext` (`:211` — the
    /// `dataRef.current.floatingContext?.nodeId ?? nodeIdProp` fallback tail).
    pub node_id: Option<String>,
}

// Handwritten rather than derived: `enabled`'s upstream default is `true`.
impl Default for UseHoverFloatingInteractionProps {
    fn default() -> Self {
        Self {
            enabled: true,
            close_delay: CloseDelayInput::default(),
            node_id: None,
        }
    }
}

/// Port of `useHoverFloatingInteraction(context, parameters)`
/// (`useHoverFloatingInteraction.ts:52-273`). Must be called inside a reactive owner.
/// Returns nothing (`:55` — the hook wires effects only).
pub fn use_hover_floating_interaction(
    context: impl Into<FloatingContextSource>,
    props: UseHoverFloatingInteractionProps,
) {
    let UseHoverFloatingInteractionProps {
        enabled,
        close_delay,
        node_id,
    } = props;

    // `'rootStore' in context ? context.rootStore : context` (`:58`).
    let store: Rc<FloatingRootStore> = context.into().root_store();
    let inner = store.rc();

    // `store.useState(...)` (`:60-62`).
    let open = inner.use_state(selectors::open);
    let floating_element_signal = inner.use_state(selectors::floating_element);
    let dom_reference_element_signal = inner.use_state(selectors::dom_reference_element);
    let data_ref = Rc::clone(&store.context.data_ref);

    // `useFloatingTree()` / `useFloatingParentNodeId()` (`:65-66`).
    let tree = use_floating_tree(None);
    let parent_id = use_floating_parent_node_id();
    let instance = use_hover_interaction_shared_state(Rc::clone(&store));

    // `const childClosedTimeout = useTimeout();` (`:69`).
    let child_closed_timeout = use_timeout();

    // `isClickLikeOpenEvent` (`:71-73`).
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

    // `isHoverOpen` (`:75-77`).
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

    // `clearPointerEvents` (`:79-81`).
    let clear_pointer_events: Rc<dyn Fn()> = {
        let instance = Rc::clone(&instance);
        Rc::new(move || clear_safe_polygon_pointer_events_mutation(&instance))
    };

    // The reset-on-close layout effect (`:83-90`).
    {
        let instance = Rc::clone(&instance);
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        use_iso_layout_effect(move || {
            let open_value = open.get();
            if !open_value {
                *instance.pointer_type.borrow_mut() = None;
                instance.rest_timeout_pending.set(false);
                instance.interacted_inside.set(false);
                clear_pointer_events();
            }
        });
    }

    // The unmount-only cleanup (`:92-94`).
    {
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        let cleanup = send_wrapper::SendWrapper::new(clear_pointer_events);
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }

    // The corridor pointer-events scope effect (`:96-156`): while open through a hover
    // with `blockPointerEvents`, claim the mutation on the scope element — the
    // `getScope` override, the cached scope, the parent floating, the closest
    // `[data-rootownerid]` ancestor, or the document body, in that order.
    {
        let enabled = enabled;
        let instance = Rc::clone(&instance);
        let is_hover_open = Rc::clone(&is_hover_open);
        let tree = tree.clone();
        let parent_id = parent_id.clone();
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        use_iso_layout_effect(move || {
            if !enabled {
                return;
            }
            let open_value = open.get();
            let dom_reference: Option<Element> = dom_reference_element_signal.get();
            let floating: Option<Element> = floating_element_signal.get();

            let block_pointer_events = instance
                .handle_close_options
                .borrow()
                .as_ref()
                .map(|options| options.block_pointer_events)
                .unwrap_or(false);
            if !(open_value
                && block_pointer_events
                && is_hover_open()
                && dom_reference.is_some()
                && floating.is_some())
            {
                return;
            }
            let dom_reference = dom_reference.unwrap();
            let floating_el = floating.unwrap();
            let doc = owner_document(Some(floating_el.as_ref() as &web_sys::Node));

            // `parentFloating` (`:112-113`).
            let parent_floating: Option<Element> = tree.as_ref().and_then(|tree| {
                let nodes = tree.nodes.borrow();
                let node = nodes
                    .iter()
                    .find(|node| node.id.as_deref() == parent_id.as_deref())?;
                let context = node.context.borrow().as_ref().cloned()?;
                context.elements.floating.get_untracked()
            });

            // Let the parent floating element stay interactive (`:115-117`).
            if let Some(parent_floating) = parent_floating.as_ref() {
                remove_pointer_events(parent_floating);
            }

            // A keep-mounted submenu can appear in the tree before it opens, so a
            // cached scope or parent lookup may resolve to the submenu itself
            // (`:119-132`).
            let cached_scope_element = instance
                .pointer_events_scope_element
                .borrow()
                .clone()
                .filter(|scope| scope != &floating_el);
            let parent_scope_element = parent_floating
                .clone()
                .filter(|parent| parent != &floating_el);
            let scope_element = instance
                .handle_close_options
                .borrow()
                .as_ref()
                .and_then(|options| {
                    options.get_scope.as_ref().and_then(|get_scope| get_scope())
                })
                .or(cached_scope_element)
                .or(parent_scope_element)
                .or_else(|| {
                    dom_reference
                        .closest("[data-rootownerid]")
                        .ok()
                        .flatten()
                })
                .or_else(|| doc.body().map(|body| body.into()));

            if let Some(scope_element) = scope_element {
                apply_safe_polygon_pointer_events_mutation(
                    &instance,
                    &scope_element,
                    &dom_reference,
                    &floating_el,
                );
            }

            // The returned cleanup (`:140-142`).
            let clear = send_wrapper::SendWrapper::new(Rc::clone(&clear_pointer_events));
            reactive_graph::owner::on_cleanup(move || (*clear)());
        });
    }

    // The floating-element listener effect (`:158-272`). The reactive
    // `floatingElement` read stands in for the dependency array.
    {
        let instance = Rc::clone(&instance);
        let store = Rc::clone(&store);
        let data_ref = Rc::clone(&data_ref);
        let close_delay = close_delay.clone();
        let node_id = node_id.clone();
        let is_hover_open = Rc::clone(&is_hover_open);
        let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
        let clear_pointer_events = Rc::clone(&clear_pointer_events);
        let tree = tree.clone();
        let parent_id = parent_id.clone();
        let child_closed_timeout = child_closed_timeout.clone();
        use_iso_layout_effect(move || {
            if !enabled {
                return;
            }
            let floating: Option<Element> = floating_element_signal.get();

            // `hasParentChildren` (`:163-165`).
            let has_parent_children: Rc<dyn Fn() -> bool> = {
                let tree = tree.clone();
                let parent_id = parent_id.clone();
                Rc::new(move || {
                    tree.as_ref()
                        .zip(parent_id.as_ref())
                        .map(|(tree, parent_id)| {
                            !get_node_children(
                                &tree.nodes.borrow(),
                                Some(parent_id.as_str()),
                                false,
                            )
                            .is_empty()
                        })
                        .unwrap_or(false)
                })
            };

            // `closeWithDelay` (`:167-180`).
            let close_with_delay: Rc<dyn Fn(&MouseEvent)> = {
                let close_delay = close_delay.clone();
                let instance = Rc::clone(&instance);
                let store = Rc::clone(&store);
                let tree = tree.clone();
                Rc::new(move |event: &MouseEvent| {
                    let close_delay = get_delay(
                        &number_input_as_delay(&close_delay),
                        "close",
                        instance.pointer_type.borrow().as_deref(),
                    );
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
                    } else {
                        instance.open_change_timeout.clear();
                        close();
                    }
                })
            };

            // `handleInteractInside` (`:182-190`).
            let handle_interact_inside = {
                let instance = Rc::clone(&instance);
                move |event: &Event| {
                    let target: Option<Element> = get_target(event)
                        .and_then(|target| target.dyn_into::<Element>().ok());
                    if !is_interactive_element(target.as_ref()) {
                        instance.interacted_inside.set(false);
                        return;
                    }

                    instance.interacted_inside.set(
                        target
                            .map(|target| {
                                target
                                    .closest("[aria-haspopup]")
                                    .map(|found| found.is_some())
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false),
                    );
                }
            };

            // The `floating.closed` subscription handle — the port's stand-in for
            // `tree.events.off('floating.closed', onNodeClosed)`
            // (`:195,201,243,255`).
            let on_node_closed_subscription: Rc<RefCell<Option<EventUnsubscribe>>> =
                Rc::new(RefCell::new(None));

            // `onNodeClosed` (`:237-247`).
            let on_node_closed: crate::floating_ui::types::EventListener<
                FloatingTreeEvent,
            > = {
                let tree = tree.clone();
                let parent_id = parent_id.clone();
                let has_parent_children = Rc::clone(&has_parent_children);
                let child_closed_timeout = child_closed_timeout.clone();
                let store = Rc::clone(&store);
                let subscription = Rc::clone(&on_node_closed_subscription);
                Rc::new(move |payload: &FloatingTreeEvent| {
                    let FloatingTreeEvent::FloatingClosed(event) = payload else {
                        return;
                    };
                    if tree.is_none() || parent_id.is_none() || has_parent_children() {
                        return;
                    }
                    // Allow the mouseenter event to fire in case child was closed
                    // because mouse moved into parent (`:241-246`).
                    let tree_for_timer = tree.as_ref().unwrap().clone();
                    let store = Rc::clone(&store);
                    let subscription = Rc::clone(&subscription);
                    let event = event.clone();
                    child_closed_timeout.start(0, move || {
                        if let Some(unsubscribe) = subscription.borrow_mut().take() {
                            unsubscribe();
                        }
                        store.set_open(
                            false,
                            &RootOpenChangeEventDetails::new(
                                reasons::TRIGGER_HOVER,
                                event.clone().into(),
                                None,
                                String::new(),
                            ),
                        );
                        tree_for_timer.events.emit(
                            "floating.closed",
                            &FloatingTreeEvent::FloatingClosed(event),
                        );
                    });
                })
            };

            // `onFloatingMouseEnter` (`:192-197`).
            let on_floating_mouse_enter = {
                let instance = Rc::clone(&instance);
                let child_closed_timeout = child_closed_timeout.clone();
                let clear_pointer_events = Rc::clone(&clear_pointer_events);
                let subscription = Rc::clone(&on_node_closed_subscription);
                move |_event: &MouseEvent| {
                    instance.open_change_timeout.clear();
                    child_closed_timeout.clear();
                    if let Some(unsubscribe) = subscription.borrow_mut().take() {
                        unsubscribe();
                    }
                    clear_pointer_events();
                }
            };

            // `onFloatingMouseLeave` (`:199-235`).
            let on_floating_mouse_leave = {
                let instance = Rc::clone(&instance);
                let store = Rc::clone(&store);
                let data_ref = Rc::clone(&data_ref);
                let tree = tree.clone();
                let node_id = node_id.clone();
                let has_parent_children = Rc::clone(&has_parent_children);
                let is_hover_open = Rc::clone(&is_hover_open);
                let is_click_like_open_event = Rc::clone(&is_click_like_open_event);
                let clear_pointer_events = Rc::clone(&clear_pointer_events);
                let close_with_delay = Rc::clone(&close_with_delay);
                let on_node_closed_listener = Rc::clone(&on_node_closed);
                let subscription = Rc::clone(&on_node_closed_subscription);
                move |event: &MouseEvent| {
                    if has_parent_children() && tree.is_some() {
                        // `tree.events.on('floating.closed', onNodeClosed)`
                        // (`:200-203`).
                        let unsubscribe = tree
                            .as_ref()
                            .unwrap()
                            .events
                            .on("floating.closed", Rc::clone(&on_node_closed_listener));
                        *subscription.borrow_mut() = Some(unsubscribe);
                        return;
                    }

                    if is_inside_enabled_trigger(
                        event.related_target().as_ref(),
                        &store.context.trigger_elements,
                    ) {
                        // If the mouse is leaving the reference element to another
                        // trigger, don't explicitly close the popup as it will be
                        // moved. (`:205-209`)
                        return;
                    }

                    let current_node_id = data_ref
                        .borrow()
                        .floating_node_id
                        .clone()
                        .or_else(|| node_id.clone());
                    let related: Option<Element> = event
                        .related_target()
                        .and_then(|related| related.dyn_into::<Element>().ok());
                    let is_moving_into_descendant_floating = tree
                        .as_ref()
                        .zip(current_node_id.as_ref())
                        .map(|(tree, current_node_id)| {
                            get_node_children(
                                &tree.nodes.borrow(),
                                Some(current_node_id.as_str()),
                                false,
                            )
                            .iter()
                            .any(|node| {
                                let node_floating = node
                                    .context
                                    .borrow()
                                    .as_ref()
                                    .and_then(|context| {
                                        context.elements.floating.get_untracked()
                                    });
                                contains(node_floating.as_ref(), related.as_ref())
                            })
                        })
                        .unwrap_or(false);

                    if is_moving_into_descendant_floating {
                        return;
                    }

                    // If the safePolygon handler is active, let it handle the close
                    // logic. (`:225-229`)
                    let handler = instance.handler.borrow().clone();
                    if let Some(handler) = handler {
                        handler(event);
                        return;
                    }

                    clear_pointer_events();
                    if is_hover_open() && !is_click_like_open_event() {
                        close_with_delay(event);
                    }
                }
            };

            // The listener fan-out (`:249-257`).
            let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

            if let Some(floating) = floating.as_ref() {
                let on_floating_mouse_enter = on_floating_mouse_enter.clone();
                let unsubscribe = leptos_ui_utils::add_event_listener(
                    floating,
                    "mouseenter",
                    move |event: &Event| {
                        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                            on_floating_mouse_enter(mouse_event);
                        }
                    },
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));

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

                let handle_interact_inside = handle_interact_inside;
                let unsubscribe = leptos_ui_utils::add_event_listener_with_options(
                    floating,
                    "pointerdown",
                    handle_interact_inside,
                    true,
                );
                cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
            }

            cleanups.push(Some(Box::new({
                let subscription = Rc::clone(&on_node_closed_subscription);
                move || {
                    if let Some(unsubscribe) = subscription.borrow_mut().take() {
                        unsubscribe();
                    }
                }
            })));

            let merged = merge_cleanups(cleanups);
            let merged = send_wrapper::SendWrapper::new(merged);
            reactive_graph::owner::on_cleanup(move || merged.take()());
        });
    }
}

// The hook needs a DOM realm (elements, native listeners, timers): the wasm suite pins
// the written mechanics — the hook has no dedicated upstream test file (the
// implementation spec's untested-source item 1).
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use std::cell::Cell;

    use reactive_graph::owner::provide_context;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::EventTarget;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::tree::{
        FloatingNodeContext, FloatingTreeStore, SharedFloatingTreeStore,
    };
    use crate::floating_ui::types::FloatingNodeType;
    use crate::floating_ui::use_hover_interaction_shared_state::{
        HoverInteraction, apply_safe_polygon_pointer_events_mutation, inline_style,
    };
    use crate::floating_ui::use_hover_shared::HandleCloseOptions;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type CallLog = Rc<RefCell<Vec<(bool, String)>>>;

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// The controlled-loop harness (the `useHover` test adaptation): the consumer
    /// syncs the callback back into the store through a Weak handle.
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

    fn element(tag: &str) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element(tag).unwrap().dyn_into().unwrap()
    }

    /// A store whose floating element is mounted, marked hover-opened through the
    /// mirrored `openEvent`, and open.
    fn open_hover_store(log: CallLog) -> Rc<FloatingRootStore> {
        let store = store_with(true, log);
        let floating = element("div");
        let document = web_sys::window().unwrap().document().unwrap();
        document.body().unwrap().append_child(&floating).unwrap();
        store.set_field(|state| &mut state.floating_element, Some(floating.into()));
        store.context.data_ref.borrow_mut().open_event =
            Some(MouseEvent::new("mouseenter").unwrap().into());
        store
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

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }

    fn pointer_events(element: &Element) -> Option<String> {
        inline_style(element)
            .map(|style| style.get_property_value("pointer-events").unwrap())
            .filter(|value| !value.is_empty())
    }

    // The floating `mouseleave` closes a hover-opened popup with reason trigger-hover
    // (`:225-234` — the no-tree top-level path through `closeWithDelay`).
    #[wasm_bindgen_test(async)]
    async fn floating_mouse_leave_closes_a_hover_opened_popup() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps::default(),
            );

            floating
                .dispatch_event(&mouse_event("mouseleave", None))
                .unwrap();

            sleep(10).await;
            assert_eq!(
                calls(&log),
                vec![(false, reasons::TRIGGER_HOVER.to_owned())],
                "the leave closed the hover-opened popup"
            );
        };
        __owner.cleanup();
    }

    // Moving into a descendant floating element does not close (`:211-223` — the
    // `getNodeChildren` related-target walk).
    #[wasm_bindgen_test(async)]
    async fn moving_into_a_descendant_floating_does_not_close() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();

            // The tree: this popup's node ("parent-node", supplied through the
            // `nodeId` override) with a registered child whose context's floating
            // element is mounted.
            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            provide_context(crate::floating_ui::tree::FloatingTreeContext(tree.clone()));
            let child_floating = element("div");
            let document = web_sys::window().unwrap().document().unwrap();
            document.body().unwrap().append_child(&child_floating).unwrap();

            let child_context: Rc<crate::floating_ui::types::FloatingContext> = {
                use crate::floating_ui::use_floating::UseFloatingOptions;
                use crate::floating_ui::use_position::UsePositionOptions;
                let result = crate::floating_ui::use_floating::use_floating(UseFloatingOptions {
                    position: UsePositionOptions {
                        placement: reactive_graph::wrappers::read::Signal::derive(|| {
                            floating_ui_dom::Placement::Bottom
                        }),
                        strategy: reactive_graph::wrappers::read::Signal::derive(|| {
                            floating_ui_dom::Strategy::Absolute
                        }),
                        middleware: reactive_graph::wrappers::read::Signal::derive(|| {
                            send_wrapper::SendWrapper::new(Vec::new())
                        }),
                        transform: reactive_graph::wrappers::read::Signal::derive(|| true),
                        while_elements_mounted: None,
                    },
                    open: None,
                    on_open_change: None,
                    elements_reference: None,
                    elements_floating: None,
                    root_context: Some(Rc::clone(&store)),
                    node_id: None,
                    external_tree: None,
                });
                store.set_field(
                    |state| &mut state.floating_element,
                    Some(child_floating.clone().into()),
                );
                result.context
            };
            tree.add_node(Rc::new(FloatingNodeType {
                id: Some("child-node".to_owned()),
                parent_id: Some("parent-node".to_owned()),
                context: RefCell::new(Some(child_context)),
            }));

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps {
                    node_id: Some("parent-node".to_owned()),
                    ..UseHoverFloatingInteractionProps::default()
                },
            );

            floating
                .dispatch_event(&mouse_event(
                    "mouseleave",
                    Some(child_floating.as_ref() as &EventTarget),
                ))
                .unwrap();

            sleep(10).await;
            assert!(
                calls(&log).is_empty(),
                "moving into the descendant floating element does not close: {:?}",
                calls(&log)
            );
        };
        __owner.cleanup();
    }

    // The `floating.closed` chain (`:200-203` + `:237-247`): a nested popup whose
    // parent has children subscribes on floating-leave instead of closing itself; a
    // later `floating.closed` arriving when the parent no longer has registered
    // children schedules the 0ms-grace close, which unsubscribes, closes, and
    // re-emits.
    #[wasm_bindgen_test(async)]
    async fn the_floating_closed_chain_closes_after_the_grace() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();

            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            provide_context(crate::floating_ui::tree::FloatingTreeContext(tree.clone()));
            // This popup's parent node id via the ambient node context.
            provide_context(FloatingNodeContext {
                id: Some("parent-node".to_owned()),
                parent_id: None,
            });
            // The parent has a registered child — `hasParentChildren` true.
            let sibling = Rc::new(FloatingNodeType {
                id: Some("sibling-node".to_owned()),
                parent_id: Some("parent-node".to_owned()),
                context: RefCell::new(None),
            });
            tree.add_node(Rc::clone(&sibling));

            // A probe for the re-emitted chain event.
            let re_emissions: Rc<Cell<u8>> = Rc::new(Cell::new(0));
            let re_emissions_handle = Rc::clone(&re_emissions);
            tree.events.on(
                "floating.closed",
                Rc::new(move |_payload| {
                    re_emissions_handle.set(re_emissions_handle.get() + 1);
                }),
            );

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps::default(),
            );

            floating
                .dispatch_event(&mouse_event("mouseleave", None))
                .unwrap();
            assert!(
                calls(&log).is_empty(),
                "the leave only subscribed — no close yet"
            );

            // The sibling unregisters (its node removed), then a `floating.closed`
            // arrives — the guard now passes.
            tree.remove_node(&sibling);
            tree.events.emit(
                "floating.closed",
                &FloatingTreeEvent::FloatingClosed(mouse_event("mouseleave", None)),
            );
            assert!(
                calls(&log).is_empty(),
                "the 0ms grace defers the close"
            );

            sleep(10).await;
            assert_eq!(
                calls(&log),
                vec![(false, reasons::TRIGGER_HOVER.to_owned())],
                "the grace timer closed the popup"
            );
            assert_eq!(
                re_emissions.get(),
                2,
                "the probe saw the triggering emission and the chain re-emit"
            );
        };
        __owner.cleanup();
    }

    // `handleInteractInside` (`:182-190`): a capture-phase pointerdown on an
    // interactive element with an `aria-haspopup` ancestor marks `interactedInside`;
    // one on a plain element clears it.
    #[wasm_bindgen_test(async)]
    async fn interacted_inside_tracks_haspopup_interactions() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();

            let menu_button = element("button");
            menu_button.set_attribute("aria-haspopup", "menu").unwrap();
            floating.append_child(&menu_button).unwrap();
            let plain = element("div");
            floating.append_child(&plain).unwrap();

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps::default(),
            );
            let instance = store
                .context
                .data_ref
                .borrow()
                .hover_interaction_state
                .clone()
                .expect("the hook stashed the shared instance");

            let pointer_down = web_sys::PointerEvent::new("pointerdown").unwrap();
            menu_button.dispatch_event(&pointer_down).unwrap();
            assert!(
                instance.interacted_inside.get(),
                "the haspopup interaction marked interactedInside"
            );

            plain.dispatch_event(&pointer_down).unwrap();
            assert!(
                !instance.interacted_inside.get(),
                "the plain-element interaction cleared interactedInside"
            );
        };
        __owner.cleanup();
    }

    // The reset-on-close effect (`:83-90`): closing clears the pointer type, the rest
    // state, the inside-interaction marker, and the instance's pointer-events
    // mutation.
    #[wasm_bindgen_test(async)]
    async fn closing_resets_the_shared_instance() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));

            // Pre-stash a dirty instance so the hook reuses it (`:118-131`).
            let instance = HoverInteraction::create();
            *instance.pointer_type.borrow_mut() = Some("mouse".to_owned());
            instance.rest_timeout_pending.set(true);
            instance.interacted_inside.set(true);
            let scope = element("div");
            let reference = element("div");
            let floating = element("div");
            apply_safe_polygon_pointer_events_mutation(
                &instance,
                &scope,
                &reference,
                &floating,
            );
            store.context.data_ref.borrow_mut().hover_interaction_state =
                Some(Rc::clone(&instance));

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps::default(),
            );

            store.update(|state, _| {
                state.open = false;
                true
            });
            // The layout-effect re-run resolves as a queued microtask
            // (`specs/architecture.md`, "Layout effect").
            any_spawner::Executor::poll_local();

            assert!(
                instance.pointer_type.borrow().is_none(),
                "the pointer type reset"
            );
            assert!(!instance.rest_timeout_pending.get(), "the rest state reset");
            assert!(
                !instance.interacted_inside.get(),
                "the inside-interaction marker reset"
            );
            assert_eq!(
                pointer_events(&scope),
                None,
                "the pointer-events mutation cleared"
            );
        };
        __owner.cleanup();
    }

    // The scope fallback chain (`:119-138`): with no `getScope`, no cached scope, and
    // no parent floating, the scope is the closest `[data-rootownerid]` ancestor of
    // the reference.
    #[wasm_bindgen_test(async)]
    async fn the_pointer_events_scope_falls_back_to_the_root_owner() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));

            let root_owner = element("div");
            root_owner.set_attribute("data-rootownerid", "").unwrap();
            let reference = element("div");
            root_owner.append_child(&reference).unwrap();
            let document = web_sys::window().unwrap().document().unwrap();
            document.body().unwrap().append_child(&root_owner).unwrap();
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();
            store.set_field(
                |state| &mut state.dom_reference_element,
                Some(reference.clone().into()),
            );

            // The corridor options the active trigger's reference-side call would have
            // synced (`useHoverReferenceInteraction.ts:158-161`).
            let instance = HoverInteraction::create();
            *instance.handle_close_options.borrow_mut() = Some(HandleCloseOptions {
                block_pointer_events: true,
                ..HandleCloseOptions::default()
            });
            store.context.data_ref.borrow_mut().hover_interaction_state =
                Some(Rc::clone(&instance));

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps::default(),
            );

            assert_eq!(
                pointer_events(&root_owner).as_deref(),
                Some("none"),
                "the root-owner ancestor carries the mutation"
            );
            assert_eq!(
                pointer_events(&reference).as_deref(),
                Some("auto"),
                "the reference stays interactive"
            );
            assert_eq!(
                pointer_events(&floating).as_deref(),
                Some("auto"),
                "the floating element stays interactive"
            );
        };
        __owner.cleanup();
    }

    // The close delay (`:167-180`): a `closeDelay` defers the floating-leave close on
    // the shared `openChangeTimeout`.
    #[wasm_bindgen_test(async)]
    async fn the_close_delay_defers_the_floating_leave_close() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = open_hover_store(Rc::clone(&log));
            let floating = selectors::floating_element(&store.get_snapshot()).unwrap();

            use_hover_floating_interaction(
                Rc::clone(&store),
                UseHoverFloatingInteractionProps {
                    close_delay: CloseDelayInput::Value(100),
                    ..UseHoverFloatingInteractionProps::default()
                },
            );
            let instance = store
                .context
                .data_ref
                .borrow()
                .hover_interaction_state
                .clone()
                .expect("the hook stashed the shared instance");

            floating
                .dispatch_event(&mouse_event("mouseleave", None))
                .unwrap();
            assert!(
                calls(&log).is_empty(),
                "the close has not fired before the delay elapses"
            );
            assert!(
                instance.open_change_timeout.is_started(),
                "the shared open-change timer is running"
            );

            sleep(150).await;
            assert_eq!(
                calls(&log),
                vec![(false, reasons::TRIGGER_HOVER.to_owned())],
                "the close fires at the delay expiry"
            );
        };
        __owner.cleanup();
    }
}
