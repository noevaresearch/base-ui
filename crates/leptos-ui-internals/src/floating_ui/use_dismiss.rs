//! Port of `packages/react/src/floating-ui-react/hooks/useDismiss.ts` — closes the
//! floating element when a dismissal is requested: the `escape` key or a press outside
//! of it (`specs/library/floating-ui-react/behavior.md`, "Public API surface":
//! `useDismiss` (`escapeKey`, `outsidePress`, `referencePress`, `outsidePressEvent`,
//! `bubbles`); "Keyboard interactions": IME-composition guard + `bubbles.escapeKey`
//! cascade; "Edge cases": the `outsidePressEvent: 'intentional'` press-observation
//! model).
//!
//! ## Rust adaptations
//!
//! - The two effects port to `use_iso_layout_effect` with the reactive reads standing
//!   in for the dependency arrays (the `use_client_point` adaptation): the openchange
//!   session-boundary effect (`useDismiss.ts:279-292`, deps `[events]` — stable) runs
//!   once at hook-call time, and the listener-registration effect
//!   (`useDismiss.ts:294-767`, deps include `open`/`floatingElement`) re-runs on open
//!   flips with its cleanup in `on_cleanup`.
//! - `pressStartedInsideRef`/`pressStartPreventedRef`/`suppressNextOutsideClickRef`/
//!   `sawPressWhileOpenRef`/`isComposingRef`/`currentPointerTypeRef`/`touchStateRef`
//!   (`useDismiss.ts:146-162`) port to `Rc`-shared `Cell`/`RefCell` handles so the
//!   handler closures and the effect share one mutable session.
//! - `dataRef.current.insideReactTree` is upstream's plain boolean — the port's
//!   `ContextData::inside_react_tree` is a `bool` (see the `types` module docs).
//! - `dataRef.current.floatingContext?.nodeId` (`useDismiss.ts:174,370`) reads the
//!   stashed [`crate::floating_ui::types::ContextData::floating_node_id`] — the node id
//!   `useFloating` stamps next to the store handle (see that field's docs).
//! - The prop resolvers keep upstream's shapes: `outsidePress`'s
//!   `boolean | ((event) => boolean)` becomes [`OutsidePress`],
//!   `outsidePressEvent`'s `PressType | { mouse, touch } | (() => …)` becomes
//!   [`OutsidePressEvent`], and `bubbles`'s `boolean | { escapeKey, outsidePress }`
//!   normalizes through [`normalize_prop`] into [`NormalizedBubbles`]
//!   (`useDismiss.ts:35-44`).
//! - `useStableCallback` handlers port to `Rc` closures reading the store fresh at
//!   event time (the `useClick` adaptation — upstream's stable-identity concern is
//!   React render-phase specific).
//! - `getTarget`/`contains` come from the `shadowDom` port; `getParentNode`/
//!   `isLastTraversableNode`/`isElement`/`isHTMLElement`/`getComputedStyle` bind the
//!   external `floating-ui-dom`'s `dom` module — the same functions upstream imports
//!   from `@floating-ui/utils/dom` (`useDismiss.ts:9-16`). The
//!   `@floating-ui/utils/dom`'s `isShadowRoot` (`instanceof ShadowRoot`) ports to a
//!   local `dyn_ref` check; the `shadowDom` util's realm-delegating variant is a
//!   different function (used elsewhere).
//! - `touchStateRef.startTime` (`useDismiss.ts:157`) is recorded upstream but never
//!   read anywhere in the hook — the port drops the dead field.
//! - The `'touches' in event` scrollbar-guard check (`useDismiss.ts:445`) ports to an
//!   `TouchEvent` interface probe — the same runtime discriminator (`in` reads the
//!   prototype chain; touch-family events are exactly the ones carrying `touches`).
//! - The listener target "once" re-targeting (`addTargetEventListenerOnce`,
//!   `useDismiss.ts:558-572`) keeps upstream's mechanism: a listener registered on the
//!   event's target mid-dispatch (capture at the document) still fires when the
//!   dispatch reaches that target's own phase, then removes itself.

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use floating_ui_dom::dom::{
    get_computed_style, get_parent_node, is_element, is_last_traversable_node,
};
use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;
use web_sys::{
    Document, Element, Event, EventTarget, HtmlElement, KeyboardEvent, MouseEvent, Node, NodeList,
    PointerEvent, ShadowRoot, TouchEvent,
};

use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::platform;
use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_timeout;
use leptos_ui_utils::use_timeout::Timeout;

use crate::floating_ui::create_attribute::create_attribute;
use crate::floating_ui::element::{is_event_target_within, is_root_element};
use crate::floating_ui::element_props::{ElementHandlers, ElementProps, FloatingContextSource};
use crate::floating_ui::event::is_virtual_click;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::nodes::get_node_children;
use crate::floating_ui::reasons;
use crate::floating_ui::tree::{SharedFloatingTreeStore, use_floating_tree};
use crate::floating_ui::types::{
    FloatingContext, FloatingUIOpenChangeDetails, RootOpenChangeEventDetails,
};

/// Port of `PressType` (`useDismiss.ts:29`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressType {
    /// `'intentional'` — dismisses on an outside `click` whose press began while the
    /// floating element was open (`useDismiss.ts:83`).
    Intentional,
    /// `'sloppy'` — fires on `pointerdown` for mouse, `touchend`/scroll-away for touch
    /// (`useDismiss.ts:84`).
    Sloppy,
}

/// Port of `UseDismissProps['outsidePress']`
/// (`useDismiss.ts:80` — `boolean | ((event) => boolean)`).
#[derive(Clone)]
pub enum OutsidePress {
    /// The boolean form.
    Bool(bool),
    /// The guard-function form — returning `false` ignores the event's target.
    Fn(Rc<dyn Fn(&Event) -> bool>),
}

impl Default for OutsidePress {
    fn default() -> Self {
        OutsidePress::Bool(true)
    }
}

/// Port of `UseDismissProps['outsidePressEvent']`
/// (`useDismiss.ts:86-98` — `'intentional' | 'sloppy' | { mouse, touch } | (() => …)`).
#[derive(Clone)]
pub enum OutsidePressEvent {
    /// `'sloppy'` (default).
    Sloppy,
    /// `'intentional'`.
    Intentional,
    /// The `{ mouse, touch }` form.
    PerPointer {
        /// The press type for mouse pointers (`pen` counts as mouse —
        /// `useDismiss.ts:346`).
        mouse: PressType,
        /// The press type for touch pointers.
        touch: PressType,
    },
    /// The resolver form, evaluated per press (`useDismiss.ts:350-352`). A resolver
    /// returning a nested resolver is unrepresentable upstream (the function's return
    /// is consumed once); it is treated as the `'sloppy'` default here.
    Resolve(Rc<dyn Fn() -> OutsidePressEvent>),
}

impl Default for OutsidePressEvent {
    fn default() -> Self {
        OutsidePressEvent::Sloppy
    }
}

/// Port of `UseDismissProps['bubbles']` (`useDismiss.ts:103-104`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubblesOption {
    /// The bare boolean form — both keys at once.
    Bool(bool),
    /// The object form, either key omitted.
    PerKey {
        /// `{ escapeKey }` — omitted means `false`.
        escape_key: Option<bool>,
        /// `{ outsidePress }` — omitted means `true`.
        outside_press: Option<bool>,
    },
}

/// The normalized `bubbles` shape — `normalizeProp`'s return
/// (`useDismiss.ts:38-43`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NormalizedBubbles {
    /// `escapeKey` (`useDismiss.ts:39-40` — default `false`).
    pub escape_key: bool,
    /// `outsidePress` (`useDismiss.ts:41-42` — default `true`).
    pub outside_press: bool,
}

/// Port of `normalizeProp` (`useDismiss.ts:35-44`).
pub fn normalize_prop(normalizable: Option<BubblesOption>) -> NormalizedBubbles {
    match normalizable {
        Some(BubblesOption::Bool(value)) => NormalizedBubbles {
            escape_key: value,
            outside_press: value,
        },
        Some(BubblesOption::PerKey {
            escape_key,
            outside_press,
        }) => NormalizedBubbles {
            escape_key: escape_key.unwrap_or(false),
            outside_press: outside_press.unwrap_or(true),
        },
        None => NormalizedBubbles {
            escape_key: false,
            outside_press: true,
        },
    }
}

/// The `dataRef.current.__escapeKeyBubbles` / `__outsidePressBubbles` pair
/// (`useDismiss.ts:173,304-305`) — the key the blocking-child check consults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleKey {
    /// `__escapeKeyBubbles`.
    EscapeKey,
    /// `__outsidePressBubbles`.
    OutsidePress,
}

impl BubbleKey {
    /// The child's stamped value for this key (`useDismiss.ts:178` —
    /// `child.context.dataRef.current[bubbleKey]`; an unstamped child reads
    /// `undefined`).
    fn child_flag(self, child: &FloatingContext) -> Option<bool> {
        let data = child.data_ref.borrow();
        match self {
            BubbleKey::EscapeKey => data.escape_key_bubbles,
            BubbleKey::OutsidePress => data.outside_press_bubbles,
        }
    }
}

/// Port of `getOutsidePressEvent`'s resolution core (`useDismiss.ts:344-359`) —
/// pure over the recorded pointer type and the prop value, so the
/// `pen`/absent-pointer coalescing and the per-pointer table are host-testable.
/// `current_pointer_type` is upstream's `currentPointerTypeRef.current` (`''` when no
/// press has been recorded).
pub fn resolve_outside_press_event_type(
    current_pointer_type: Option<&str>,
    prop: &OutsidePressEvent,
) -> PressType {
    // `const computedType = type === 'pen' || !type ? 'mouse' : type;`
    // (`useDismiss.ts:345-346`).
    let is_touch = current_pointer_type == Some("touch");

    // The resolved prop value (`useDismiss.ts:348-352` — the function form is consumed
    // once).
    let resolved = match prop {
        OutsidePressEvent::Resolve(resolve) => resolve(),
        resolved => resolved.clone(),
    };

    match resolved {
        OutsidePressEvent::Sloppy => PressType::Sloppy,
        OutsidePressEvent::Intentional => PressType::Intentional,
        OutsidePressEvent::PerPointer { mouse, touch } => {
            if is_touch {
                touch
            } else {
                mouse
            }
        }
        OutsidePressEvent::Resolve(_) => PressType::Sloppy,
    }
}

/// Port of `shouldIgnoreEvent` (`useDismiss.ts:361-367`): in `intentional` mode every
/// event type but `click` is ignored (press lifecycle only), and in `sloppy` mode
/// `click` itself is ignored (the compat press events handle it).
pub fn should_ignore_event(press_type: PressType, event_type: &str) -> bool {
    (press_type == PressType::Intentional && event_type != "click")
        || (press_type == PressType::Sloppy && event_type == "click")
}

/// The interior per-session state of `touchStateRef` (`useDismiss.ts:156-162`).
/// Upstream also records `startTime: Date.now()` — never read anywhere in the hook;
/// the port drops the dead field (see the module docs).
#[derive(Debug, Clone, Copy)]
struct TouchState {
    start_x: f64,
    start_y: f64,
    dismiss_on_touch_end: bool,
    dismiss_on_mouse_down: bool,
}

/// Port of `UseDismissProps` (`useDismiss.ts:46-109`) with the documented defaults
/// (`useDismiss.ts:120-128`).
#[derive(Clone)]
pub struct UseDismissProps {
    /// `enabled` (`useDismiss.ts:52` — default `true`).
    pub enabled: bool,
    /// `escapeKey` (`useDismiss.ts:57` — default `true`).
    pub escape_key: bool,
    /// `outsidePress` (`useDismiss.ts:80` — default `true`).
    pub outside_press: OutsidePress,
    /// `outsidePressEvent` (`useDismiss.ts:86-98` — default `'sloppy'`).
    pub outside_press_event: OutsidePressEvent,
    /// `referencePress` (`useDismiss.ts:66` — a lazy getter; default `alwaysFalse`,
    /// `useDismiss.ts:31-33,125`).
    pub reference_press: Option<Rc<dyn Fn() -> bool>>,
    /// `bubbles` (`useDismiss.ts:103-104`).
    pub bubbles: Option<BubblesOption>,
    /// `externalTree` (`useDismiss.ts:108`).
    pub external_tree: Option<SharedFloatingTreeStore>,
}

impl Default for UseDismissProps {
    fn default() -> Self {
        Self {
            enabled: true,
            escape_key: true,
            outside_press: OutsidePress::Bool(true),
            outside_press_event: OutsidePressEvent::Sloppy,
            reference_press: None,
            bubbles: None,
            external_tree: None,
        }
    }
}

/// Whether an event target is a `ShadowRoot` — the `isShadowRoot` import from
/// `@floating-ui/utils/dom` (`useDismiss.ts:15`), an `instanceof ShadowRoot` check.
fn is_shadow_root_node(node: &Node) -> bool {
    node.dyn_ref::<ShadowRoot>().is_some()
}

/// Collects the elements a `querySelectorAll` matched.
fn query_all_elements(scope: &EventTarget, selector: &str) -> Vec<Element> {
    let result: Result<NodeList, JsValue> = if let Some(document) = scope.dyn_ref::<Document>() {
        document.query_selector_all(selector)
    } else if let Some(shadow_root) = scope.dyn_ref::<ShadowRoot>() {
        shadow_root.query_selector_all(selector)
    } else {
        return Vec::new();
    };
    match result {
        Ok(node_list) => js_sys::Array::from(&node_list.into())
            .iter()
            .filter_map(|value| value.dyn_into::<Element>().ok())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Port of `addTargetEventListenerOnce` (`useDismiss.ts:558-572`): registers
/// `listener` on the event's own target for the event's own type, fires at most once,
/// and removes itself. Registered mid-dispatch (the capture listeners run at the
/// document), it still fires when the dispatch reaches the target's own phase — the
/// DOM consults the target's (already updated) listener list when it gets there.
fn add_target_event_listener_once(event: &Event, listener: Rc<dyn Fn(&Event)>) {
    let Some(target) = get_target(event) else {
        return;
    };

    let handle: Rc<RefCell<Option<EventListenerUnsubscribe>>> = Rc::new(RefCell::new(None));
    let handle_for_listener = Rc::clone(&handle);
    let event_type = event.type_();
    let unsubscribe =
        leptos_ui_utils::add_event_listener(&target, &event_type, move |received: &Event| {
            listener(received);
            if let Some(unsubscribe) = handle_for_listener.borrow_mut().take() {
                unsubscribe.unsubscribe();
            }
        });
    *handle.borrow_mut() = Some(unsubscribe);
}

/// Port of `useDismiss(context, props)` (`useDismiss.ts:116-811`). Must be called
/// inside a reactive owner (the listeners and timers register cleanups). Returns the
/// `reference` + `floating` + `trigger` handler bags, or the empty props when
/// disabled.
pub fn use_dismiss(
    context: impl Into<FloatingContextSource>,
    props: UseDismissProps,
) -> ElementProps {
    let UseDismissProps {
        enabled,
        escape_key,
        outside_press,
        outside_press_event,
        reference_press,
        bubbles,
        external_tree,
    } = props;

    let store: Rc<FloatingRootStore> = context.into().root_store();
    let inner = store.rc();

    // `const open = store.useState('open')` /
    // `const floatingElement = store.useState('floatingElement')` (`useDismiss.ts:132-133`).
    let open = inner.use_state(selectors::open);
    let floating = inner.use_state(selectors::floating_element);

    // `const { dataRef, events } = store.context` (`useDismiss.ts:134`).
    let data_ref = Rc::clone(&store.context.data_ref);
    let events = Rc::clone(&store.context.events);

    // `const tree = useFloatingTree(externalTree)` (`useDismiss.ts:136`).
    let tree = use_floating_tree(external_tree);

    // `const outsidePress = typeof outsidePressProp === 'function' ? outsidePressFn :
    // outsidePressProp; const outsidePressEnabled = outsidePress !== false`
    // (`useDismiss.ts:137-141`).
    let outside_press_enabled = !matches!(outside_press, OutsidePress::Bool(false));

    // `const { escapeKey: escapeKeyBubbles, outsidePress: outsidePressBubbles } =
    // normalizeProp(bubbles)` (`useDismiss.ts:144`).
    let NormalizedBubbles {
        escape_key: escape_key_bubbles,
        outside_press: outside_press_bubbles,
    } = normalize_prop(bubbles);

    // The refs (`useDismiss.ts:146-162`).
    let press_started_inside: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let press_start_prevented: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    // Ignore only the very next outside click after dragging from inside to outside.
    let suppress_next_outside_click: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    // A click whose press began before the floating element opened is the tail of that
    // gesture, not a new outside press.
    let saw_press_while_open: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let is_composing: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let current_pointer_type: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let touch_state: Rc<RefCell<Option<TouchState>>> = Rc::new(RefCell::new(None));

    // `const cancelDismissOnEndTimeout = useTimeout();` /
    // `const clearInsideReactTreeTimeout = useTimeout();` (`useDismiss.ts:164-165`).
    let cancel_dismiss_on_end_timeout = use_timeout();
    let clear_inside_react_tree_timeout = use_timeout();

    // `clearInsideReactTree` (`useDismiss.ts:167-170`).
    let clear_inside_react_tree: Rc<dyn Fn()> = {
        let data_ref = Rc::clone(&data_ref);
        let clear_inside_react_tree_timeout = clear_inside_react_tree_timeout.clone();
        Rc::new(move || {
            clear_inside_react_tree_timeout.clear();
            data_ref.borrow_mut().inside_react_tree = false;
        })
    };

    // `hasBlockingChild` (`useDismiss.ts:172-181`): an open child that does not bubble
    // this key blocks the parent's dismissal. `!child.context.dataRef.current[key]` is
    // JS truthiness — an unstamped (`undefined`) child blocks too.
    let has_blocking_child: Rc<dyn Fn(BubbleKey) -> bool> = {
        let data_ref = Rc::clone(&data_ref);
        let tree = tree.clone();
        Rc::new(move |key: BubbleKey| {
            let Some(tree) = tree.as_ref() else {
                return false;
            };
            let node_id = data_ref.borrow().floating_node_id.clone();
            let nodes = tree.nodes.borrow();
            get_node_children(&nodes, node_id.as_deref(), false)
                .iter()
                .any(|child| {
                    child
                        .context
                        .borrow()
                        .as_ref()
                        .is_some_and(|child_context| {
                            child_context.open.get_untracked()
                                && !matches!(key.child_flag(child_context), Some(true))
                        })
                })
        })
    };

    // `isEventWithinOwnElements` (`useDismiss.ts:183-188`).
    let is_event_within_own_elements: Rc<dyn Fn(&Event) -> bool> = {
        let store = Rc::clone(&store);
        Rc::new(move |event: &Event| {
            let snapshot = store.get_snapshot();
            is_event_target_within(
                event,
                snapshot
                    .floating_element
                    .as_ref()
                    .map(|element| element.as_ref() as &Node),
            ) || is_event_target_within(
                event,
                snapshot
                    .dom_reference_element
                    .as_ref()
                    .map(|element| element.as_ref() as &Node),
            )
        })
    };

    // `closeOnReferencePress` (`useDismiss.ts:190-202`) — the reference bag's
    // `onPointerDown`/`onClick` close path.
    let close_on_reference_press: Rc<dyn Fn(&Event)> = {
        let store = Rc::clone(&store);
        let reference_press = reference_press.clone();
        Rc::new(move |event: &Event| {
            let reference_press = reference_press
                .as_ref()
                .map(|check| check())
                .unwrap_or(false);
            if !reference_press {
                return;
            }

            store.set_open(
                false,
                &RootOpenChangeEventDetails::new(
                    reasons::TRIGGER_PRESS,
                    event.clone(),
                    None,
                    String::new(),
                ),
            );
        })
    };

    // `closeOnEscapeKeyDown` (`useDismiss.ts:204-233`) — shared by the document
    // `keydown` listener and the reference/floating `onKeyDown` slots.
    let close_on_escape_key_down: Rc<dyn Fn(&KeyboardEvent)> = {
        let store = Rc::clone(&store);
        let is_composing = Rc::clone(&is_composing);
        let has_blocking_child = Rc::clone(&has_blocking_child);
        Rc::new(move |event: &KeyboardEvent| {
            if !store.select(selectors::open) || !enabled || !escape_key || event.key() != "Escape"
            {
                return;
            }

            // Wait until IME is settled. Pressing `Escape` while composing should
            // close the compose menu, but not the floating element
            // (`useDismiss.ts:210-214`).
            if is_composing.get() {
                return;
            }

            if !escape_key_bubbles && has_blocking_child(BubbleKey::EscapeKey) {
                return;
            }

            let event_details = RootOpenChangeEventDetails::new(
                reasons::ESCAPE_KEY,
                event.clone().into(),
                None,
                String::new(),
            );

            store.set_open(false, &event_details);

            if !event_details.is_canceled() {
                event.prevent_default();
            }

            if !escape_key_bubbles && !event_details.is_propagation_allowed() {
                event.stop_propagation();
            }
        })
    };

    // `markInsideReactTree` (`useDismiss.ts:235-238`).
    let mark_inside_react_tree: Rc<dyn Fn()> = {
        let data_ref = Rc::clone(&data_ref);
        let clear_inside_react_tree_timeout = clear_inside_react_tree_timeout.clone();
        let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
        Rc::new(move || {
            data_ref.borrow_mut().inside_react_tree = true;
            let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
            clear_inside_react_tree_timeout.start(0, move || clear_inside_react_tree());
        })
    };

    // `markPressStartedInsideReactTree` (`useDismiss.ts:240-259`).
    let mark_press_started_inside: Rc<dyn Fn(&Event)> = {
        let store = Rc::clone(&store);
        let press_started_inside = Rc::clone(&press_started_inside);
        let press_start_prevented = Rc::clone(&press_start_prevented);
        Rc::new(move |event: &Event| {
            if !store.select(selectors::open) || !enabled {
                return;
            }

            let button = event
                .dyn_ref::<MouseEvent>()
                .map(|mouse_event| mouse_event.button())
                .unwrap_or(0);
            if button != 0 {
                return;
            }

            // Only treat presses that start within the floating DOM subtree as inside.
            // This avoids suppressing parent dismissal when interacting with nested
            // portals (`useDismiss.ts:246-252`).
            let target: Option<Element> =
                get_target(event).and_then(|target| target.dyn_into::<Element>().ok());
            let floating_element = selectors::floating_element(&store.get_snapshot());
            if !contains(floating_element.as_ref(), target.as_ref()) {
                return;
            }

            if !press_started_inside.get() {
                press_started_inside.set(true);
                press_start_prevented.set(false);
            }
        })
    };

    // `markInsidePressStartPrevented` (`useDismiss.ts:261-275`).
    let mark_inside_press_start_prevented: Rc<dyn Fn(&Event)> = {
        let store = Rc::clone(&store);
        let press_started_inside = Rc::clone(&press_started_inside);
        let press_start_prevented = Rc::clone(&press_start_prevented);
        Rc::new(move |event: &Event| {
            if !store.select(selectors::open) || !enabled {
                return;
            }

            if !event.default_prevented() {
                return;
            }

            if press_started_inside.get() {
                press_start_prevented.set(true);
            }
        })
    };

    // The openchange session-boundary effect (`useDismiss.ts:279-292`): only the
    // closing half ends the session — `setOpen(true)` on an already-open element
    // (hovering an inactive trigger) must not drop a press mid-gesture. Deps are the
    // stable `events` handle, so the subscription is registered once.
    {
        let saw_press_while_open = Rc::clone(&saw_press_while_open);
        let unsubscribe = events.on(
            "openchange",
            Rc::new(move |details: &FloatingUIOpenChangeDetails| {
                if !details.open {
                    saw_press_while_open.set(false);
                }
            }),
        );
        let unsubscribe = SendWrapper::new(unsubscribe);
        reactive_graph::owner::on_cleanup(move || unsubscribe());
    }

    // The listener-registration effect (`useDismiss.ts:294-767`): the reactive reads
    // stand in for the dependency array (`open`, `floatingElement`; the rest are
    // stable identities), and the effect's cleanup return ports to `on_cleanup`.
    {
        let data_ref = Rc::clone(&data_ref);
        let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
        let has_blocking_child = Rc::clone(&has_blocking_child);
        let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
        let close_on_escape_key_down = Rc::clone(&close_on_escape_key_down);
        let open = open.clone();
        let floating = floating.clone();
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let floating_element = floating.get();

            // Closed/disabled branch (`useDismiss.ts:295-302`): the reset runs in the
            // effect body (not the cleanup, which also fires mid-gesture on dependency
            // changes), and upstream's `return clearInsideReactTree` cleanup ports to
            // an `on_cleanup` registration.
            if !open_value || !enabled {
                if !open_value {
                    saw_press_while_open.set(false);
                }
                let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
                let cleanup = SendWrapper::new(move || clear_inside_react_tree());
                reactive_graph::owner::on_cleanup(move || (*cleanup)());
                return;
            }

            // `dataRef.current.__escapeKeyBubbles = escapeKeyBubbles;`
            // `dataRef.current.__outsidePressBubbles = outsidePressBubbles;`
            // (`useDismiss.ts:304-305`).
            {
                let mut data = data_ref.borrow_mut();
                data.escape_key_bubbles = Some(escape_key_bubbles);
                data.outside_press_bubbles = Some(outside_press_bubbles);
            }

            // The effect-local timers (`useDismiss.ts:307-308`).
            let composition_timeout = Timeout::create();
            let prevented_press_suppression_timeout = Timeout::create();

            // `const doc = ownerDocument(floatingElement)` (`useDismiss.ts:309`).
            let doc: Document = owner_document(
                floating_element
                    .as_ref()
                    .map(|element| element.as_ref() as &Node),
            );

            // `handleCompositionStart` (`useDismiss.ts:311-314`).
            let handle_composition_start = {
                let composition_timeout = composition_timeout.clone();
                let is_composing = Rc::clone(&is_composing);
                move |_event: &Event| {
                    composition_timeout.clear();
                    is_composing.set(true);
                }
            };

            // `handleCompositionEnd` (`useDismiss.ts:316-328`).
            let handle_composition_end = {
                let composition_timeout = composition_timeout.clone();
                let is_composing = Rc::clone(&is_composing);
                move |_event: &Event| {
                    // Safari fires `compositionend` before `keydown`, so the reset
                    // waits until the next tick (0ms elsewhere; the 5ms WebKit delay
                    // keeps the browser workaround while the test stays 0ms).
                    let is_composing = Rc::clone(&is_composing);
                    composition_timeout.start(
                        if platform().engine.webkit { 5 } else { 0 },
                        move || {
                            is_composing.set(false);
                        },
                    );
                }
            };

            // `suppressImmediateOutsideClickAfterPreventedStart`
            // (`useDismiss.ts:330-337`).
            let suppress_immediate_outside_click_after_prevented_start: Rc<dyn Fn()> = {
                let suppress_next_outside_click = Rc::clone(&suppress_next_outside_click);
                let prevented_press_suppression_timeout =
                    prevented_press_suppression_timeout.clone();
                Rc::new(move || {
                    suppress_next_outside_click.set(true);
                    // Firefox can emit the synthetic outside click in a later task
                    // after pointer lock exit, so microtask clearing is too early
                    // (`useDismiss.ts:332-336`).
                    let suppress_next_outside_click = Rc::clone(&suppress_next_outside_click);
                    prevented_press_suppression_timeout.start(0, move || {
                        suppress_next_outside_click.set(false);
                    });
                })
            };

            // `resetPressStartState` (`useDismiss.ts:339-342`).
            let reset_press_start_state: Rc<dyn Fn()> = {
                let press_started_inside = Rc::clone(&press_started_inside);
                let press_start_prevented = Rc::clone(&press_start_prevented);
                Rc::new(move || {
                    press_started_inside.set(false);
                    press_start_prevented.set(false);
                })
            };

            // `getOutsidePressEvent` (`useDismiss.ts:344-359`).
            let get_outside_press_event: Rc<dyn Fn() -> PressType> = {
                let current_pointer_type = Rc::clone(&current_pointer_type);
                let outside_press_event = outside_press_event.clone();
                Rc::new(move || {
                    resolve_outside_press_event_type(
                        current_pointer_type.borrow().as_deref(),
                        &outside_press_event,
                    )
                })
            };

            // `isEventWithinFloatingTree` (`useDismiss.ts:369-378`).
            let is_event_within_floating_tree: Rc<dyn Fn(&Event) -> bool> = {
                let data_ref = Rc::clone(&data_ref);
                let tree = tree.clone();
                let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
                Rc::new(move |event: &Event| {
                    let target_is_inside_children = tree.as_ref().is_some_and(|tree| {
                        let node_id = data_ref.borrow().floating_node_id.clone();
                        let nodes = tree.nodes.borrow();
                        get_node_children(&nodes, node_id.as_deref(), false)
                            .iter()
                            .any(|node| {
                                let floating_element = node
                                    .context
                                    .borrow()
                                    .as_ref()
                                    .map(|node_context| {
                                        node_context
                                            .root_store
                                            .get_snapshot()
                                            .floating_element
                                            .clone()
                                    })
                                    .flatten();
                                is_event_target_within(
                                    event,
                                    floating_element
                                        .as_ref()
                                        .map(|element| element.as_ref() as &Node),
                                )
                            })
                    });

                    target_is_inside_children || is_event_within_own_elements(event)
                })
            };

            // `closeOnPressOutside` (`useDismiss.ts:380-513`).
            let close_on_press_outside: Rc<dyn Fn(&Event)> = {
                let store = Rc::clone(&store);
                let data_ref = Rc::clone(&data_ref);
                let get_outside_press_event = Rc::clone(&get_outside_press_event);
                let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
                let is_event_within_floating_tree = Rc::clone(&is_event_within_floating_tree);
                let has_blocking_child = Rc::clone(&has_blocking_child);
                let outside_press = outside_press.clone();
                let saw_press_while_open = Rc::clone(&saw_press_while_open);
                let suppress_next_outside_click = Rc::clone(&suppress_next_outside_click);
                let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
                let prevented_press_suppression_timeout =
                    prevented_press_suppression_timeout.clone();
                Rc::new(move |event: &Event| {
                    let press_type = get_outside_press_event();
                    if should_ignore_event(press_type, &event.type_()) {
                        // A new press began outside the floating element and its
                        // trigger. Clear any leftover drag-out suppression so this
                        // press's eventual click can dismiss (`useDismiss.ts:382-387`).
                        if event.type_() != "click" && !is_event_within_own_elements(event) {
                            prevented_press_suppression_timeout.clear();
                            suppress_next_outside_click.set(false);
                        }
                        clear_inside_react_tree();
                        return;
                    }

                    if data_ref.borrow().inside_react_tree {
                        clear_inside_react_tree();
                        return;
                    }

                    let target: Option<EventTarget> = get_target(event);
                    let target_element: Option<Element> = target
                        .as_ref()
                        .and_then(|target| target.dyn_ref::<Element>())
                        .cloned();
                    let inert_selector = format!("[{}]", create_attribute("inert"));

                    // `const targetRoot = isElement(target) ? target.getRootNode() :
                    // null` + the shadow-root-aware marker scan
                    // (`useDismiss.ts:397-405`).
                    let markers: Vec<Element> = {
                        let target_root: Option<Node> = target_element
                            .as_ref()
                            .map(|element| element.get_root_node());
                        let search_scope: Option<EventTarget> = match target_root.as_ref() {
                            Some(root) if is_shadow_root_node(root) => {
                                Some(root.clone().unchecked_into::<EventTarget>())
                            }
                            _ => {
                                let document = owner_document(
                                    selectors::floating_element(&store.get_snapshot())
                                        .as_ref()
                                        .map(|element| element.as_ref() as &Node),
                                );
                                Some(document.into())
                            }
                        };
                        match search_scope {
                            Some(scope) => query_all_elements(&scope, &inert_selector),
                            None => Vec::new(),
                        }
                    };

                    // `const triggers = store.context.triggerElements` — if another
                    // trigger is clicked, don't close the floating element
                    // (`useDismiss.ts:407-416`).
                    if let Some(target_element) = target_element.as_ref() {
                        let triggers = &store.context.trigger_elements;
                        if triggers.has_element(target_element)
                            || triggers.has_matching_element(|trigger| {
                                contains(Some(trigger), Some(target_element))
                            })
                        {
                            return;
                        }
                    }

                    // The `targetRootAncestor` walk (`useDismiss.ts:418-426`).
                    let mut target_root_ancestor: Option<Element> = target_element.clone();
                    loop {
                        let Some(current) = target_root_ancestor.clone() else {
                            break;
                        };
                        if is_last_traversable_node(current.as_ref() as &Node) {
                            break;
                        }
                        let next_parent = get_parent_node(current.as_ref() as &Node);
                        if is_last_traversable_node(&next_parent) || !is_element(&next_parent) {
                            break;
                        }
                        target_root_ancestor = next_parent.dyn_into::<Element>().ok();
                    }

                    // Check if the click occurred on a third-party element injected
                    // after the floating element rendered (`useDismiss.ts:428-441`).
                    if !markers.is_empty() {
                        if let Some(target_element) = target_element.as_ref() {
                            if !is_root_element(target_element)
                                // Clicked on a direct ancestor (e.g. FloatingOverlay).
                                && !contains(
                                    Some(target_element),
                                    selectors::floating_element(&store.get_snapshot()).as_ref(),
                                )
                                // If the target root element contains none of the
                                // markers, then the element was injected after the
                                // floating element rendered.
                                && markers.iter().all(|marker| {
                                    target_root_ancestor
                                        .as_ref()
                                        .map(|ancestor| {
                                            !contains(Some(ancestor), Some(marker))
                                        })
                                        .unwrap_or(true)
                                })
                            {
                                return;
                            }
                        }
                    }

                    // Check if the click occurred on the scrollbar. Skip for touch
                    // events: scrollbars don't receive touch events on most platforms
                    // (`useDismiss.ts:443-475`).
                    if let Some(target_element) = target_element
                        .as_ref()
                        .and_then(|element| element.dyn_ref::<HtmlElement>())
                    {
                        let is_touch_event = event.dyn_ref::<TouchEvent>().is_some();
                        if !is_touch_event {
                            let last_traversable =
                                is_last_traversable_node(target_element.as_ref() as &Node);
                            let style = get_computed_style(target_element);
                            let scroll_re = |value: &str| value == "auto" || value == "scroll";
                            let is_scrollable_x = last_traversable
                                || scroll_re(
                                    &style.get_property_value("overflow-x").unwrap_or_default(),
                                );
                            let is_scrollable_y = last_traversable
                                || scroll_re(
                                    &style.get_property_value("overflow-y").unwrap_or_default(),
                                );

                            let can_scroll_x = is_scrollable_x
                                && target_element.client_width() > 0
                                && target_element.scroll_width() > target_element.client_width();
                            let can_scroll_y = is_scrollable_y
                                && target_element.client_height() > 0
                                && target_element.scroll_height() > target_element.client_height();

                            let is_rtl =
                                style.get_property_value("direction").unwrap_or_default() == "rtl";

                            // Check click position relative to scrollbar
                            // (`useDismiss.ts:459-468`).
                            let offset_x = event
                                .dyn_ref::<MouseEvent>()
                                .map(|mouse_event| mouse_event.offset_x())
                                .unwrap_or(0);
                            let offset_y = event
                                .dyn_ref::<MouseEvent>()
                                .map(|mouse_event| mouse_event.offset_y())
                                .unwrap_or(0);

                            let pressed_vertical_scrollbar = can_scroll_y
                                && (if is_rtl {
                                    offset_x
                                        <= target_element.offset_width()
                                            - target_element.client_width()
                                } else {
                                    offset_x > target_element.client_width()
                                });

                            let pressed_horizontal_scrollbar =
                                can_scroll_x && offset_y > target_element.client_height();

                            if pressed_vertical_scrollbar || pressed_horizontal_scrollbar {
                                return;
                            }
                        }
                    }

                    if is_event_within_floating_tree(event) {
                        return;
                    }

                    // Only `click` events reach this point in intentional mode
                    // (`useDismiss.ts:481-501`).
                    if press_type == PressType::Intentional {
                        // Press-less clicks (keyboard, assistive technology,
                        // `element.click()`) report no click count; `isVirtualClick`
                        // also catches the ones that do.
                        let detail = event
                            .dyn_ref::<MouseEvent>()
                            .map(|mouse_event| mouse_event.detail())
                            .unwrap_or(0);
                        let is_virtual = event
                            .dyn_ref::<MouseEvent>()
                            .map(is_virtual_click)
                            .unwrap_or(false);
                        if detail != 0 && !is_virtual && !saw_press_while_open.get() {
                            return;
                        }

                        // A press that starts inside and ends outside gets one
                        // suppressed outside click. Run this after inside-target
                        // checks so inside clicks don't consume the one-shot
                        // suppression.
                        if suppress_next_outside_click.get() {
                            prevented_press_suppression_timeout.clear();
                            suppress_next_outside_click.set(false);
                            return;
                        }
                    }

                    if let OutsidePress::Fn(check) = &outside_press {
                        if !check(event) {
                            return;
                        }
                    }

                    if has_blocking_child(BubbleKey::OutsidePress) {
                        return;
                    }

                    store.set_open(
                        false,
                        &RootOpenChangeEventDetails::new(
                            reasons::OUTSIDE_PRESS,
                            event.clone(),
                            None,
                            String::new(),
                        ),
                    );
                    clear_inside_react_tree();
                })
            };

            // `handlePointerDown` (`useDismiss.ts:515-527`).
            let handle_pointer_down: Rc<dyn Fn(&PointerEvent)> = {
                let store = Rc::clone(&store);
                let get_outside_press_event = Rc::clone(&get_outside_press_event);
                let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
                let close_on_press_outside = Rc::clone(&close_on_press_outside);
                Rc::new(move |event: &PointerEvent| {
                    if get_outside_press_event() != PressType::Sloppy
                        || event.pointer_type() == "touch"
                        || !store.select(selectors::open)
                        || !enabled
                        || is_event_within_own_elements(event.as_ref())
                    {
                        return;
                    }

                    close_on_press_outside(event.as_ref());
                })
            };

            // `handleTouchStart` (`useDismiss.ts:529-556`).
            let handle_touch_start: Rc<dyn Fn(&TouchEvent)> = {
                let store = Rc::clone(&store);
                let get_outside_press_event = Rc::clone(&get_outside_press_event);
                let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
                let touch_state = Rc::clone(&touch_state);
                let cancel_dismiss_on_end_timeout = cancel_dismiss_on_end_timeout.clone();
                Rc::new(move |event: &TouchEvent| {
                    if get_outside_press_event() != PressType::Sloppy
                        || !store.select(selectors::open)
                        || !enabled
                        || is_event_within_own_elements(event.as_ref())
                    {
                        return;
                    }

                    if let Some(touch) = event.touches().get(0) {
                        *touch_state.borrow_mut() = Some(TouchState {
                            start_x: touch.client_x() as f64,
                            start_y: touch.client_y() as f64,
                            dismiss_on_touch_end: false,
                            dismiss_on_mouse_down: true,
                        });

                        let touch_state = Rc::clone(&touch_state);
                        cancel_dismiss_on_end_timeout.start(1000, move || {
                            if let Some(state) = touch_state.borrow_mut().as_mut() {
                                state.dismiss_on_touch_end = false;
                                state.dismiss_on_mouse_down = false;
                            }
                        });
                    }
                })
            };

            // `closeOnPressOutsideCapture` (`useDismiss.ts:579-607`).
            let close_on_press_outside_capture: Rc<dyn Fn(&Event)> = {
                let cancel_dismiss_on_end_timeout = cancel_dismiss_on_end_timeout.clone();
                let saw_press_while_open = Rc::clone(&saw_press_while_open);
                let current_pointer_type = Rc::clone(&current_pointer_type);
                let touch_state = Rc::clone(&touch_state);
                let handle_pointer_down = Rc::clone(&handle_pointer_down);
                let close_on_press_outside = Rc::clone(&close_on_press_outside);
                Rc::new(move |event: &Event| {
                    cancel_dismiss_on_end_timeout.clear();

                    // Only `pointerdown` marks a press; `mousedown` is its
                    // compatibility event, and counting it would misattribute a
                    // gesture that started before open (`useDismiss.ts:582-590`).
                    if event.type_() == "pointerdown" {
                        if let Some(pointer_event) = event.dyn_ref::<PointerEvent>() {
                            // Only a primary press can produce a `click`.
                            if pointer_event.button() == 0 {
                                saw_press_while_open.set(true);
                            }
                            *current_pointer_type.borrow_mut() = Some(pointer_event.pointer_type());
                        }
                    }

                    if event.type_() == "mousedown"
                        && touch_state
                            .borrow()
                            .as_ref()
                            .is_some_and(|state| !state.dismiss_on_mouse_down)
                    {
                        return;
                    }

                    let handler: Rc<dyn Fn(&Event)> = if event.type_() == "pointerdown" {
                        let handle_pointer_down = Rc::clone(&handle_pointer_down);
                        Rc::new(move |target_event: &Event| {
                            if let Some(pointer_event) = target_event.dyn_ref::<PointerEvent>() {
                                handle_pointer_down(pointer_event);
                            }
                        })
                    } else {
                        let close_on_press_outside = Rc::clone(&close_on_press_outside);
                        Rc::new(move |target_event: &Event| close_on_press_outside(target_event))
                    };
                    add_target_event_listener_once(event, handler);
                })
            };

            // `handlePressEndCapture` (`useDismiss.ts:609-655`).
            let handle_press_end_capture: Rc<dyn Fn(&Event)> = {
                let saw_press_while_open = Rc::clone(&saw_press_while_open);
                let press_started_inside = Rc::clone(&press_started_inside);
                let press_start_prevented = Rc::clone(&press_start_prevented);
                let reset_press_start_state = Rc::clone(&reset_press_start_state);
                let get_outside_press_event = Rc::clone(&get_outside_press_event);
                let is_event_within_floating_tree = Rc::clone(&is_event_within_floating_tree);
                let outside_press = outside_press.clone();
                let suppress_immediate_outside_click_after_prevented_start =
                    Rc::clone(&suppress_immediate_outside_click_after_prevented_start);
                let prevented_press_suppression_timeout =
                    prevented_press_suppression_timeout.clone();
                let suppress_next_outside_click = Rc::clone(&suppress_next_outside_click);
                let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
                Rc::new(move |event: &Event| {
                    // A cancelled gesture produces no click. Not cleared on
                    // `pointerup`: the click fires after it and must still find the
                    // press (`useDismiss.ts:610-614`).
                    if event.type_() == "pointercancel" {
                        saw_press_while_open.set(false);
                    }

                    if !press_started_inside.get() {
                        return;
                    }

                    let press_started_inside_default_prevented = press_start_prevented.get();
                    reset_press_start_state();

                    if get_outside_press_event() != PressType::Intentional {
                        return;
                    }

                    if event.type_() == "pointercancel" {
                        if press_started_inside_default_prevented {
                            suppress_immediate_outside_click_after_prevented_start();
                        }
                        return;
                    }

                    if is_event_within_floating_tree(event) {
                        return;
                    }

                    // If pointerdown was prevented, no click may be generated for that
                    // interaction. However, Firefox may still emit an immediate click
                    // after pointerup (e.g. NumberField scrub with pointer lock), so
                    // suppress for one tick to absorb that synthetic click only
                    // (`useDismiss.ts:638-645`).
                    if press_started_inside_default_prevented {
                        suppress_immediate_outside_click_after_prevented_start();
                        return;
                    }

                    // Avoid suppressing when outsidePress explicitly ignores this
                    // target (`useDismiss.ts:647-650`).
                    if let OutsidePress::Fn(check) = &outside_press {
                        if !check(event) {
                            return;
                        }
                    }

                    prevented_press_suppression_timeout.clear();
                    suppress_next_outside_click.set(true);
                    clear_inside_react_tree();
                })
            };

            // `handleTouchMove` (`useDismiss.ts:657-684`).
            let handle_touch_move: Rc<dyn Fn(&TouchEvent)> = {
                let get_outside_press_event = Rc::clone(&get_outside_press_event);
                let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
                let touch_state = Rc::clone(&touch_state);
                let close_on_press_outside = Rc::clone(&close_on_press_outside);
                let cancel_dismiss_on_end_timeout = cancel_dismiss_on_end_timeout.clone();
                Rc::new(move |event: &TouchEvent| {
                    if get_outside_press_event() != PressType::Sloppy
                        || touch_state.borrow().is_none()
                        || is_event_within_own_elements(event.as_ref())
                    {
                        return;
                    }

                    let Some(touch) = event.touches().get(0) else {
                        return;
                    };

                    let distance = {
                        let state = touch_state.borrow();
                        let state = state.as_ref().expect("checked non-null above");
                        let delta_x = (touch.client_x() as f64 - state.start_x).abs();
                        let delta_y = (touch.client_y() as f64 - state.start_y).abs();
                        (delta_x * delta_x + delta_y * delta_y).sqrt()
                    };

                    if distance > 5.0 {
                        if let Some(state) = touch_state.borrow_mut().as_mut() {
                            state.dismiss_on_touch_end = true;
                        }
                    }

                    if distance > 10.0 {
                        close_on_press_outside(event.as_ref());
                        cancel_dismiss_on_end_timeout.clear();
                        *touch_state.borrow_mut() = None;
                    }
                })
            };

            // `handleTouchEnd` (`useDismiss.ts:690-705`).
            let handle_touch_end: Rc<dyn Fn(&TouchEvent)> = {
                let get_outside_press_event = Rc::clone(&get_outside_press_event);
                let is_event_within_own_elements = Rc::clone(&is_event_within_own_elements);
                let touch_state = Rc::clone(&touch_state);
                let close_on_press_outside = Rc::clone(&close_on_press_outside);
                let cancel_dismiss_on_end_timeout = cancel_dismiss_on_end_timeout.clone();
                Rc::new(move |event: &TouchEvent| {
                    if get_outside_press_event() != PressType::Sloppy
                        || touch_state.borrow().is_none()
                        || is_event_within_own_elements(event.as_ref())
                    {
                        return;
                    }

                    if touch_state
                        .borrow()
                        .as_ref()
                        .is_some_and(|state| state.dismiss_on_touch_end)
                    {
                        close_on_press_outside(event.as_ref());
                    }

                    cancel_dismiss_on_end_timeout.clear();
                    *touch_state.borrow_mut() = None;
                })
            };

            // The capture wrappers (`useDismiss.ts:574-577,686-688,707-709`). The
            // once-target re-registration widens the handlers to `&Event`; the
            // registered event name is the handler's own, so the downcast always
            // matches in practice.
            let handle_touch_start_once: Rc<dyn Fn(&Event)> = {
                let handle_touch_start = Rc::clone(&handle_touch_start);
                Rc::new(move |event: &Event| {
                    if let Some(touch_event) = event.dyn_ref::<TouchEvent>() {
                        handle_touch_start(touch_event);
                    }
                })
            };
            let handle_touch_move_once: Rc<dyn Fn(&Event)> = {
                let handle_touch_move = Rc::clone(&handle_touch_move);
                Rc::new(move |event: &Event| {
                    if let Some(touch_event) = event.dyn_ref::<TouchEvent>() {
                        handle_touch_move(touch_event);
                    }
                })
            };
            let handle_touch_end_once: Rc<dyn Fn(&Event)> = {
                let handle_touch_end = Rc::clone(&handle_touch_end);
                Rc::new(move |event: &Event| {
                    if let Some(touch_event) = event.dyn_ref::<TouchEvent>() {
                        handle_touch_end(touch_event);
                    }
                })
            };
            let handle_touch_start_capture: Rc<dyn Fn(&Event)> = {
                let current_pointer_type = Rc::clone(&current_pointer_type);
                let handle_touch_start_once = Rc::clone(&handle_touch_start_once);
                Rc::new(move |event: &Event| {
                    *current_pointer_type.borrow_mut() = Some("touch".to_owned());
                    add_target_event_listener_once(event, Rc::clone(&handle_touch_start_once));
                })
            };
            let handle_touch_move_capture: Rc<dyn Fn(&Event)> = {
                let handle_touch_move_once = Rc::clone(&handle_touch_move_once);
                Rc::new(move |event: &Event| {
                    add_target_event_listener_once(event, Rc::clone(&handle_touch_move_once));
                })
            };
            let handle_touch_end_capture: Rc<dyn Fn(&Event)> = {
                let handle_touch_end_once = Rc::clone(&handle_touch_end_once);
                Rc::new(move |event: &Event| {
                    add_target_event_listener_once(event, Rc::clone(&handle_touch_end_once));
                })
            };

            // The document listeners (`useDismiss.ts:711-739`).
            let to_cleanup = |listener: EventListenerUnsubscribe| -> Option<CleanupFn> {
                Some(Box::new(move || listener.unsubscribe()))
            };

            let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

            if escape_key {
                let keydown_listener = {
                    let close_on_escape_key_down = Rc::clone(&close_on_escape_key_down);
                    leptos_ui_utils::add_event_listener(&doc, "keydown", move |event: &Event| {
                        if let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() {
                            close_on_escape_key_down(keyboard_event);
                        }
                    })
                };
                let composition_start_listener = {
                    leptos_ui_utils::add_event_listener(
                        &doc,
                        "compositionstart",
                        handle_composition_start,
                    )
                };
                let composition_end_listener = {
                    leptos_ui_utils::add_event_listener(
                        &doc,
                        "compositionend",
                        handle_composition_end,
                    )
                };
                cleanups.push(Some(Box::new(merge_cleanups(vec![
                    to_cleanup(keydown_listener),
                    to_cleanup(composition_start_listener),
                    to_cleanup(composition_end_listener),
                ]))));
            }

            if outside_press_enabled {
                let close_on_press_outside_capture_for_click =
                    Rc::clone(&close_on_press_outside_capture);
                let click_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "click",
                    move |event: &Event| close_on_press_outside_capture_for_click(event),
                    true,
                );
                let close_on_press_outside_capture_for_pointerdown =
                    Rc::clone(&close_on_press_outside_capture);
                let pointerdown_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "pointerdown",
                    move |event: &Event| close_on_press_outside_capture_for_pointerdown(event),
                    true,
                );
                let handle_press_end_capture_for_pointerup = Rc::clone(&handle_press_end_capture);
                let pointerup_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "pointerup",
                    move |event: &Event| handle_press_end_capture_for_pointerup(event),
                    true,
                );
                let handle_press_end_capture_for_pointercancel =
                    Rc::clone(&handle_press_end_capture);
                let pointercancel_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "pointercancel",
                    move |event: &Event| handle_press_end_capture_for_pointercancel(event),
                    true,
                );
                let close_on_press_outside_capture_for_mousedown =
                    Rc::clone(&close_on_press_outside_capture);
                let mousedown_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "mousedown",
                    move |event: &Event| close_on_press_outside_capture_for_mousedown(event),
                    true,
                );
                let handle_press_end_capture_for_mouseup = Rc::clone(&handle_press_end_capture);
                let mouseup_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "mouseup",
                    move |event: &Event| handle_press_end_capture_for_mouseup(event),
                    true,
                );

                let capture_and_passive = web_sys::AddEventListenerOptions::new();
                capture_and_passive.set_capture(true);
                capture_and_passive.set_passive(true);

                let handle_touch_start_capture_for_listener =
                    Rc::clone(&handle_touch_start_capture);
                let touchstart_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "touchstart",
                    move |event: &Event| handle_touch_start_capture_for_listener(event),
                    capture_and_passive.clone(),
                );
                let handle_touch_move_capture_for_listener = Rc::clone(&handle_touch_move_capture);
                let touchmove_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "touchmove",
                    move |event: &Event| handle_touch_move_capture_for_listener(event),
                    capture_and_passive.clone(),
                );
                let handle_touch_end_capture_for_listener = Rc::clone(&handle_touch_end_capture);
                let touchend_listener = leptos_ui_utils::add_event_listener_with_options(
                    &doc,
                    "touchend",
                    move |event: &Event| handle_touch_end_capture_for_listener(event),
                    capture_and_passive,
                );

                cleanups.extend(
                    [
                        click_listener,
                        pointerdown_listener,
                        pointerup_listener,
                        pointercancel_listener,
                        mousedown_listener,
                        mouseup_listener,
                        touchstart_listener,
                        touchmove_listener,
                        touchend_listener,
                    ]
                    .map(to_cleanup),
                );
            }

            let unsubscribe = merge_cleanups(cleanups);

            // The effect cleanup (`useDismiss.ts:741-748`). The merged cleanup is
            // `FnOnce`, so it rides in a take-once cell (the use_focus cleanup
            // pattern).
            let press_started_inside = Rc::clone(&press_started_inside);
            let press_start_prevented = Rc::clone(&press_start_prevented);
            let suppress_next_outside_click = Rc::clone(&suppress_next_outside_click);
            let clear_inside_react_tree = Rc::clone(&clear_inside_react_tree);
            let cleanup: Rc<Cell<Option<CleanupFn>>> =
                Rc::new(Cell::new(Some(Box::new(move || {
                    unsubscribe();
                    composition_timeout.clear();
                    prevented_press_suppression_timeout.clear();
                    press_started_inside.set(false);
                    press_start_prevented.set(false);
                    suppress_next_outside_click.set(false);
                    clear_inside_react_tree();
                }))));
            let cleanup_handle = SendWrapper::new(cleanup);
            reactive_graph::owner::on_cleanup(move || {
                if let Some(cleanup) = cleanup_handle.deref().take() {
                    cleanup();
                }
            });
        });
    }

    // The `reference` bag (`useDismiss.ts:769-776`).
    let reference = ElementHandlers {
        on_key_down: {
            let close_on_escape_key_down = Rc::clone(&close_on_escape_key_down);
            Some(Rc::new(move |event: &KeyboardEvent| {
                close_on_escape_key_down(event)
            }))
        },
        on_pointer_down: {
            let close_on_reference_press = Rc::clone(&close_on_reference_press);
            Some(Rc::new(move |event: &PointerEvent| {
                let event: Event = event.clone().into();
                close_on_reference_press(&event);
            }))
        },
        on_click: {
            let close_on_reference_press = Rc::clone(&close_on_reference_press);
            Some(Rc::new(move |event: &MouseEvent| {
                close_on_reference_press(event)
            }))
        },
        ..ElementHandlers::default()
    };

    // The `floating` bag (`useDismiss.ts:778-805`).
    let floating_handlers = ElementHandlers {
        on_key_down: {
            let close_on_escape_key_down = Rc::clone(&close_on_escape_key_down);
            Some(Rc::new(move |event: &KeyboardEvent| {
                close_on_escape_key_down(event)
            }))
        },
        // `onPointerDown`/`onMouseDown` mark a prevented press start — `onMouseDown`
        // may be blocked if `event.preventDefault()` is called in `onPointerDown`
        // (`useDismiss.ts:781-785`).
        on_pointer_down: {
            let mark_inside_press_start_prevented = Rc::clone(&mark_inside_press_start_prevented);
            Some(Rc::new(move |event: &PointerEvent| {
                let event: Event = event.clone().into();
                mark_inside_press_start_prevented(&event);
            }))
        },
        on_mouse_down: {
            let mark_inside_press_start_prevented = Rc::clone(&mark_inside_press_start_prevented);
            Some(Rc::new(move |event: &MouseEvent| {
                let event: Event = event.clone().into();
                mark_inside_press_start_prevented(&event);
            }))
        },
        on_click_capture: {
            let mark_inside_react_tree = Rc::clone(&mark_inside_react_tree);
            Some(Rc::new(move |_event: &MouseEvent| mark_inside_react_tree()))
        },
        on_mouse_down_capture: {
            let mark_inside_react_tree = Rc::clone(&mark_inside_react_tree);
            let mark_press_started_inside = Rc::clone(&mark_press_started_inside);
            Some(Rc::new(move |event: &MouseEvent| {
                let event: Event = event.clone().into();
                mark_inside_react_tree();
                mark_press_started_inside(&event);
            }))
        },
        on_pointer_down_capture: {
            let mark_inside_react_tree = Rc::clone(&mark_inside_react_tree);
            let mark_press_started_inside = Rc::clone(&mark_press_started_inside);
            Some(Rc::new(move |event: &PointerEvent| {
                let event: Event = event.clone().into();
                mark_inside_react_tree();
                mark_press_started_inside(&event);
            }))
        },
        on_mouse_up_capture: {
            let mark_inside_react_tree = Rc::clone(&mark_inside_react_tree);
            Some(Rc::new(move |_event: &MouseEvent| mark_inside_react_tree()))
        },
        on_touch_end_capture: {
            let mark_inside_react_tree = Rc::clone(&mark_inside_react_tree);
            Some(Rc::new(move |_event: &TouchEvent| mark_inside_react_tree()))
        },
        on_touch_move_capture: {
            let mark_inside_react_tree = Rc::clone(&mark_inside_react_tree);
            Some(Rc::new(move |_event: &TouchEvent| mark_inside_react_tree()))
        },
        ..ElementHandlers::default()
    };

    // `enabled ? { reference, floating, trigger: reference } : {}`
    // (`useDismiss.ts:807-810`).
    if enabled {
        ElementProps {
            reference: Some(reference.clone()),
            floating: Some(floating_handlers),
            trigger: Some(reference),
            ..ElementProps::default()
        }
    } else {
        ElementProps::default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::rc::Rc;

    use super::{
        BubblesOption, NormalizedBubbles, OutsidePressEvent, PressType, normalize_prop,
        resolve_outside_press_event_type, should_ignore_event,
    };

    // Pins `normalizeProp` (`useDismiss.ts:35-44`; upstream's normalizeProp tests at
    // `useDismiss.test.tsx:605-647`): the boolean form maps to both keys, the object
    // form defaults `escapeKey` to `false` and `outsidePress` to `true`, and the
    // undefined form defaults both.
    #[test]
    fn normalize_prop_covers_the_boolean_object_and_undefined_forms() {
        assert_eq!(
            normalize_prop(None),
            NormalizedBubbles {
                escape_key: false,
                outside_press: true
            },
            "undefined defaults to escapeKey false, outsidePress true"
        );
        assert_eq!(
            normalize_prop(Some(BubblesOption::Bool(true))),
            NormalizedBubbles {
                escape_key: true,
                outside_press: true
            },
            "`true` maps to both keys"
        );
        assert_eq!(
            normalize_prop(Some(BubblesOption::Bool(false))),
            NormalizedBubbles {
                escape_key: false,
                outside_press: false
            },
            "`false` maps to both keys"
        );
        assert_eq!(
            normalize_prop(Some(BubblesOption::PerKey {
                escape_key: None,
                outside_press: None
            })),
            normalize_prop(None),
            "the empty object form carries the same defaults"
        );
        assert_eq!(
            normalize_prop(Some(BubblesOption::PerKey {
                escape_key: Some(true),
                outside_press: None
            })),
            NormalizedBubbles {
                escape_key: true,
                outside_press: true
            },
            "an omitted outsidePress defaults to true"
        );
        assert_eq!(
            normalize_prop(Some(BubblesOption::PerKey {
                escape_key: None,
                outside_press: Some(false)
            })),
            NormalizedBubbles {
                escape_key: false,
                outside_press: false
            },
            "an omitted escapeKey defaults to false"
        );
    }

    // Pins `getOutsidePressEvent`'s resolution core (`useDismiss.ts:344-359`): the
    // recorded pointer type coalesces `pen` and the no-press `''` to the mouse arm of
    // the per-pointer table, `touch` selects the touch arm, and the plain string and
    // resolver forms ignore the pointer entirely.
    #[test]
    fn the_outside_press_event_type_resolves_per_pointer_and_resolver_forms() {
        let per_pointer = OutsidePressEvent::PerPointer {
            mouse: PressType::Sloppy,
            touch: PressType::Intentional,
        };

        assert_eq!(
            resolve_outside_press_event_type(Some("touch"), &per_pointer),
            PressType::Intentional,
            "a recorded touch pointer selects the touch arm"
        );
        for pointer in [None, Some(""), Some("pen"), Some("mouse")] {
            assert!(
                resolve_outside_press_event_type(pointer, &per_pointer) == PressType::Sloppy,
                "pointer {:?} coalesces to the mouse arm",
                pointer
            );
        }

        assert_eq!(
            resolve_outside_press_event_type(Some("touch"), &OutsidePressEvent::Sloppy),
            PressType::Sloppy,
            "the plain string form ignores the pointer"
        );
        assert_eq!(
            resolve_outside_press_event_type(None, &OutsidePressEvent::Intentional),
            PressType::Intentional,
            "the plain intentional form ignores the pointer"
        );

        let resolver = OutsidePressEvent::Resolve({
            let per_pointer = per_pointer;
            Rc::new(move || per_pointer.clone())
        });
        assert_eq!(
            resolve_outside_press_event_type(Some("touch"), &resolver),
            PressType::Intentional,
            "the resolver form is consumed per press"
        );
    }

    // Pins `shouldIgnoreEvent` (`useDismiss.ts:361-367`): intentional mode ignores
    // every event type but `click`; sloppy mode ignores `click` itself.
    #[test]
    fn should_ignore_event_splits_the_two_press_models() {
        for event_type in [
            "pointerdown",
            "mousedown",
            "touchstart",
            "touchend",
            "touchmove",
        ] {
            assert!(
                should_ignore_event(PressType::Intentional, event_type),
                "intentional mode ignores {event_type}"
            );
            assert!(
                !should_ignore_event(PressType::Sloppy, event_type),
                "sloppy mode handles {event_type}"
            );
        }
        assert!(!should_ignore_event(PressType::Intentional, "click"));
        assert!(should_ignore_event(PressType::Sloppy, "click"));
    }

    // Pins the blocking-child key semantics (`useDismiss.ts:178` —
    // `!child.context.dataRef.current[bubbleKey]`): the JS truthiness makes an
    // unstamped child block, so only an explicit `true` clears the block.
    #[test]
    fn the_bubble_key_covers_both_stamped_flags() {
        use super::BubbleKey::{EscapeKey, OutsidePress};
        assert_eq!(EscapeKey, EscapeKey);
        assert_eq!(OutsidePress, OutsidePress);
        assert_ne!(EscapeKey, OutsidePress);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use web_sys::{Element, Event, EventTarget, KeyboardEvent, MouseEvent, PointerEvent};

    use leptos_ui_utils::merge_cleanups::CleanupFn;

    use crate::floating_ui::element_props::FloatingContextSource;
    use crate::floating_ui::floating_root_store::{FloatingRootStore, FloatingRootStoreOptions};
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::reasons;
    use crate::floating_ui::tree::SharedFloatingTreeStore;
    use crate::floating_ui::types::{FloatingNodeType, ReferenceType, RootOpenChangeEventDetails};
    use crate::floating_ui::use_dismiss::{
        BubblesOption, OutsidePress, OutsidePressEvent, UseDismissProps, use_dismiss,
    };

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type CallLog = Rc<RefCell<Vec<(bool, String)>>>;

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// Builds a real `FloatingContext` around the store — the tree nodes carry the
    /// context handle now (`use_floating`'s stamping), so the cascade tests stamp what
    /// the blocking check reads.
    fn context_for(
        store: &Rc<FloatingRootStore>,
    ) -> Rc<crate::floating_ui::types::FloatingContext> {
        use crate::floating_ui::use_floating::use_base_ui_floating;
        use crate::floating_ui::use_position::UsePositionOptions;

        use_base_ui_floating(
            UsePositionOptions {
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
            Rc::clone(store),
        )
        .context
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn body() -> web_sys::HtmlElement {
        document().body().unwrap()
    }

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }

    /// A store whose `onOpenChange` mirrors the controlled loop, with caller-supplied
    /// trigger elements (the registry is constructed before the store, like the
    /// popup components do).
    fn store_with_triggers(
        open: bool,
        log: Option<CallLog>,
        sync_state: bool,
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
        if let Some(log) = log {
            let store_for_callback = Rc::clone(&store);
            store.context.set_on_open_change(Some(Rc::new(
                move |next_open: bool, details: &RootOpenChangeEventDetails| {
                    log.borrow_mut().push((next_open, details.reason.clone()));
                    if sync_state {
                        store_for_callback.set_field(|state| &mut state.open, next_open);
                    }
                },
            )));
        }
        store
    }

    fn store_with(open: bool, log: Option<CallLog>, sync_state: bool) -> Rc<FloatingRootStore> {
        store_with_triggers(open, log, sync_state, PopupTriggerMap::new())
    }

    /// Seeds the store with a real reference + floating element appended to the body
    /// (`useFloatingRootContext`'s element sync — the elements the dismissal checks
    /// classify events against). Returns the elements so a test can detach them.
    fn attach_elements(store: &Rc<FloatingRootStore>) -> (Element, Element) {
        let reference = document().create_element("button").unwrap();
        let floating = document().create_element("div").unwrap();
        body().append_child(&reference).unwrap();
        body().append_child(&floating).unwrap();
        store.set_field(
            |state| &mut state.reference_element,
            Some(ReferenceType::Element(reference.clone())),
        );
        store.set_field(
            |state| &mut state.dom_reference_element,
            Some(reference.clone()),
        );
        store.set_field(|state| &mut state.floating_element, Some(floating.clone()));
        (reference, floating)
    }

    /// Attaches the hook's bags to the store's reference/floating elements. The
    /// reference bag rides both the `reference` and `trigger` roles
    /// (`useDismiss.ts:807-810`). Returns the elements (for detaching) plus the
    /// unsubscribe handles — an `EventListenerUnsubscribe` removes its listener on
    /// drop, so the handles must live as long as the test's assertions.
    fn attach_bags(
        store: &Rc<FloatingRootStore>,
        props: UseDismissProps,
    ) -> (Element, Element, Vec<CleanupFn>) {
        let (reference, floating) = attach_elements(store);
        let element_props = use_dismiss(FloatingContextSource::from(Rc::clone(store)), props);
        let mut cleanups: Vec<CleanupFn> = Vec::new();
        if let Some(bag) = element_props.reference.as_ref() {
            cleanups.extend(bag.attach_to(reference.as_ref()));
        }
        if let Some(bag) = element_props.floating.as_ref() {
            cleanups.extend(bag.attach_to(floating.as_ref()));
        }
        (reference, floating, cleanups)
    }

    fn detach(element: &Element) {
        let _ = element.remove();
    }

    fn pointer_event(type_: &str, pointer_type: &str, button: i16) -> PointerEvent {
        let init = web_sys::PointerEventInit::new();
        init.set_bubbles(true);
        init.set_pointer_type(pointer_type);
        init.set_button(button);
        PointerEvent::new_with_event_init_dict(type_, &init).unwrap()
    }

    fn mouse_event(type_: &str, detail: i32, client_x: i32, client_y: i32) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_detail(detail);
        init.set_client_x(client_x);
        init.set_client_y(client_y);
        MouseEvent::new_with_mouse_event_init_dict(type_, &init).unwrap()
    }

    fn escape_keydown() -> KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        // `fireEvent`-equivalent: the cancelable flag is what makes the
        // `preventDefault` observable on the dispatched object.
        init.set_cancelable(true);
        init.set_key("Escape");
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    fn bubbling_event(type_: &str) -> Event {
        let init = web_sys::EventInit::new();
        init.set_bubbles(true);
        Event::new_with_event_init_dict(type_, &init).unwrap()
    }

    fn dispatch(target: &EventTarget, event: &Event) {
        target.dispatch_event(event).unwrap();
    }

    fn outside_div() -> Element {
        let element = document().create_element("div").unwrap();
        body().append_child(&element).unwrap();
        element
    }

    /// Builds a real `TouchEvent` with one touch at the given coordinates — the
    /// `Touch`/`TouchList` constructors are not reachable through the web-sys
    /// dictionary bindings, so the fixture dispatches through the JS realm.
    fn touch_event(type_: &str, client_x: f64, client_y: f64) -> web_sys::TouchEvent {
        let code = format!(
            "(() => {{ const t = new Touch({{ identifier: 1, target: document.body, clientX: {x}, clientY: {y} }}); return new TouchEvent('{t}', {{ bubbles: true, touches: [t], changedTouches: [t] }}); }})()",
            x = client_x,
            y = client_y,
            t = type_
        );
        js_sys::eval(&code)
            .unwrap()
            .dyn_into::<web_sys::TouchEvent>()
            .unwrap()
    }

    fn calls(log: &CallLog) -> Vec<(bool, String)> {
        log.borrow().clone()
    }

    /// The logged *closes* only — the session test's redundant `setOpen(true)` calls
    /// also land in the log (upstream's `onOpenChange` receives every call), and the
    /// dismissal assertions only count the closing half.
    fn closes(log: &CallLog) -> Vec<String> {
        log.borrow()
            .iter()
            .filter(|(open, _)| !open)
            .map(|(_, reason)| reason.clone())
            .collect()
    }

    // Pins the default Escape dismissal (`useDismiss.test.tsx:94-111`): an Escape
    // keydown anywhere dismisses the open popup with reason `escapeKey`, and the
    // event is default-prevented (the dismissal was not canceled).
    #[wasm_bindgen_test]
    fn escape_dismisses_and_prevents_default() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());

            let keydown = escape_keydown();
            dispatch(body().as_ref(), &keydown);

            assert_eq!(
                calls(&log),
                vec![(false, reasons::ESCAPE_KEY.to_owned())],
                "Escape dismisses the open popup"
            );
            assert!(
                keydown.default_prevented(),
                "the un-canceled dismissal default-prevents the keydown"
            );

            detach(&reference);
            detach(&floating);
        };
        __owner.cleanup();
    }

    // Pins the veto path (`useDismiss.test.tsx:115-128`): a canceled close skips the
    // `preventDefault` (the Escape key keeps its native scroll-dismiss role).
    #[wasm_bindgen_test]
    fn escape_does_not_prevent_default_when_the_close_is_canceled() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, None, true);
            // A consumer that vetoes every close — `onOpenChange` receives the
            // details and cancels them (`useDismiss.ts:225-227` reads `isCanceled`).
            let log_for_callback = Rc::clone(&log);
            store.context.set_on_open_change(Some(Rc::new(
                move |_open: bool, details: &RootOpenChangeEventDetails| {
                    log_for_callback
                        .borrow_mut()
                        .push((_open, details.reason.clone()));
                    details.cancel();
                },
            )));
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());

            let keydown = escape_keydown();
            dispatch(body().as_ref(), &keydown);

            assert_eq!(
                calls(&log),
                vec![(false, reasons::ESCAPE_KEY.to_owned())],
                "the dismissal still reaches the consumer"
            );
            assert!(
                !keydown.default_prevented(),
                "a canceled dismissal does not prevent the default"
            );

            detach(&reference);
            detach(&floating);
        };
        __owner.cleanup();
    }

    // Pins the IME guard (`useDismiss.test.tsx:149-179`): Escape during an active
    // composition does not dismiss; after `compositionend` settles, it does.
    #[wasm_bindgen_test(async)]
    async fn ime_composition_guards_the_escape_dismissal() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());

            dispatch(body().as_ref(), &bubbling_event("compositionstart"));
            dispatch(body().as_ref(), &escape_keydown());
            assert!(
                calls(&log).is_empty(),
                "Escape during IME composition does not dismiss"
            );

            dispatch(body().as_ref(), &bubbling_event("compositionend"));
            // The compositionend reset rides a timeout (`useDismiss.ts:320-327` —
            // 5ms on WebKit, 0 elsewhere).
            sleep(30).await;
            dispatch(body().as_ref(), &escape_keydown());
            assert_eq!(
                calls(&log),
                vec![(false, reasons::ESCAPE_KEY.to_owned())],
                "Escape after the composition settles dismisses"
            );

            detach(&reference);
            detach(&floating);
        };
        __owner.cleanup();
    }

    // Pins the default outside-press dismissal (`useDismiss.test.tsx:181-250`): a
    // pointerdown outside the floating element (sloppy mode) dismisses with reason
    // `outsidePress`, and a second press on the closed popup does nothing.
    #[wasm_bindgen_test(async)]
    async fn outside_pointer_press_dismisses_and_a_closed_popup_ignores_further_presses() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());
            let outside = outside_div();

            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert_eq!(
                calls(&log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "the outside pointerdown dismisses"
            );
            // The consumer loop synced the store's open state synchronously, so the
            // closed popup's press gate (`useDismiss.ts:519`) stops the next press.
            sleep(10).await;
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert_eq!(
                calls(&log).len(),
                1,
                "a closed popup does not dismiss again"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the inside-target classification (`useDismiss.test.tsx:517-550`): presses
    // and clicks inside the floating element never dismiss.
    #[wasm_bindgen_test]
    fn presses_inside_the_floating_element_do_not_dismiss() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());

            dispatch(floating.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            dispatch(floating.as_ref(), &mouse_event("click", 1, 0, 0));
            dispatch(floating.as_ref(), &pointer_event("pointerdown", "touch", 0));
            assert!(
                calls(&log).is_empty(),
                "inside presses (mouse and touch) and clicks do not dismiss"
            );

            detach(&reference);
            detach(&floating);
        };
        __owner.cleanup();
    }

    // Pins the prop gates (`useDismiss.test.tsx:497-516`): `escapeKey: false` and
    // `outsidePress: false` disable their respective dismissal paths.
    #[wasm_bindgen_test]
    fn escape_key_false_and_outside_press_false_disable_the_paths() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    escape_key: false,
                    outside_press: OutsidePress::Bool(false),
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();

            dispatch(body().as_ref(), &escape_keydown());
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert!(calls(&log).is_empty(), "both dismissal paths are disabled");

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the outsidePress function guard (`useDismiss.test.tsx:264-269,551-557`):
    // a guard returning false for the target ignores it; returning true lets the
    // dismissal through.
    #[wasm_bindgen_test]
    fn the_outside_press_function_guard_filters_targets() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    outside_press: OutsidePress::Fn(Rc::new(|_event| false)),
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();

            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert!(calls(&log).is_empty(), "the guard vetoed the dismissal");

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the reference-press close path (`useDismiss.test.tsx:252-257,511-516`):
    // `referencePress` is a lazy getter — the default never closes, an explicit
    // `true` closes with reason `triggerPress` on the reference bag's press events.
    #[wasm_bindgen_test]
    fn reference_press_closes_only_when_the_lazy_getter_enables_it() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            // Default: the getter defaults to `alwaysFalse` (`useDismiss.ts:125`).
            let default_log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let default_store = store_with(true, Some(default_log.clone()), true);
            let (reference, floating, _cleanups) =
                attach_bags(&default_store, UseDismissProps::default());
            dispatch(
                reference.as_ref(),
                &pointer_event("pointerdown", "mouse", 0),
            );
            assert!(
                calls(&default_log).is_empty(),
                "the default referencePress never closes"
            );
            detach(&reference);
            detach(&floating);

            // Enabled: the press closes with the triggerPress reason.
            let enabled_log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let enabled_store = store_with(true, Some(enabled_log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &enabled_store,
                UseDismissProps {
                    reference_press: Some(Rc::new(|| true)),
                    ..UseDismissProps::default()
                },
            );
            dispatch(
                reference.as_ref(),
                &pointer_event("pointerdown", "mouse", 0),
            );
            assert_eq!(
                calls(&enabled_log),
                vec![(false, reasons::TRIGGER_PRESS.to_owned())],
                "an enabled referencePress closes on the reference pointerdown"
            );
            detach(&reference);
            detach(&floating);
        };
        __owner.cleanup();
    }

    // Pins the intentional-mode press gate (`useDismiss.test.tsx:1078-1091,1288-1296`):
    // an outside `click` with `detail: 1` dismisses only when a primary press was
    // observed while open.
    #[wasm_bindgen_test]
    fn intentional_mode_click_dismisses_only_after_an_observed_press() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    outside_press_event: OutsidePressEvent::Intentional,
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();

            // No press observed: the detail-1 click is the tail of an unseen gesture.
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert!(
                calls(&log).is_empty(),
                "a click whose press was never observed does not dismiss"
            );

            // A primary press observed while open (the sloppy-gated pointerdown
            // handler does not dismiss in intentional mode, but the capture-phase
            // press tracker records it).
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert!(
                calls(&log).is_empty(),
                "the pointerdown itself does not dismiss in intentional mode"
            );
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert_eq!(
                calls(&log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "the click following an observed press dismisses"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the press-less click acceptance (`useDismiss.test.tsx:1129-1146`): clicks
    // with `detail: 0` (keyboard, assistive tech, `element.click()`) dismiss without
    // a prior press; non-primary presses do not count as presses.
    #[wasm_bindgen_test]
    fn intentional_mode_press_less_clicks_dismiss_and_non_primary_presses_do_not_count() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    outside_press_event: OutsidePressEvent::Intentional,
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();

            // A non-primary press is not recorded as a press.
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 2));
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert!(
                calls(&log).is_empty(),
                "a non-primary press does not make the trailing click a dismissal"
            );

            // A detail-0 click (press-less) dismisses directly.
            dispatch(outside.as_ref(), &mouse_event("click", 0, 0, 0));
            assert_eq!(
                calls(&log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "a press-less click dismisses"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the cancelled-press rule (`useDismiss.test.tsx:1313-1327`): a
    // `pointercancel` drops the recorded press, so the trailing click does not
    // dismiss.
    #[wasm_bindgen_test]
    fn intentional_mode_a_cancelled_press_does_not_count() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    outside_press_event: OutsidePressEvent::Intentional,
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();

            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            dispatch(outside.as_ref(), &bubbling_event("pointercancel"));
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert!(
                calls(&log).is_empty(),
                "the cancelled press no longer backs the trailing click"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the drag-out suppression (`useDismiss.test.tsx:1026-1076`): a press that
    // starts inside and releases outside suppresses exactly one trailing outside
    // click; the next outside click dismisses.
    #[wasm_bindgen_test]
    fn intentional_mode_a_drag_from_inside_to_outside_suppresses_one_click() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    outside_press_event: OutsidePressEvent::Intentional,
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();

            // The drag: press inside the floating element, release outside.
            dispatch(floating.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            dispatch(outside.as_ref(), &pointer_event("pointerup", "mouse", 0));
            assert!(calls(&log).is_empty(), "the drag itself does not dismiss");

            // The trailing click is the one-shot suppressed click.
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert!(
                calls(&log).is_empty(),
                "the drag's trailing click is suppressed"
            );

            // A further outside press + click dismisses.
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert_eq!(
                calls(&log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "a fresh press + click dismisses after the suppression is consumed"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the session boundary (`useDismiss.test.tsx:1189-1234` batch-reopen and
    // `:1236-1262` redundant-open — merged into one session log): only the closing
    // half of an openchange resets the press record — a same-batch close+reopen (no
    // closed effect run between them) drops the press, while a redundant
    // `setOpen(true)` on an already-open popup keeps it. Every `setOpen` — the
    // test-driven ones included — lands in the log (upstream's `onOpenChange`
    // receives every call and `setOpen` does not dedupe), so the count assertions
    // run against stated baselines.
    #[wasm_bindgen_test]
    fn intentional_mode_the_press_record_follows_the_openchange_session() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    outside_press_event: OutsidePressEvent::Intentional,
                    ..UseDismissProps::default()
                },
            );
            let outside = outside_div();
            let none_details = || {
                RootOpenChangeEventDetails::new(
                    reasons::NONE,
                    Event::new("mousedown").unwrap(),
                    None,
                    String::new(),
                )
            };

            // A redundant `setOpen(true)` mid-gesture keeps the press on record
            // (`useDismiss.test.tsx:1246-1260`) — only the closing half ends the
            // session, so the trailing click still dismisses.
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            store.set_open(true, &none_details());
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert_eq!(
                closes(&log),
                vec![reasons::OUTSIDE_PRESS.to_owned()],
                "the press survived the redundant open dispatch"
            );

            // Reopen, then observe a fresh press in the live session and close+reopen
            // in the same batch (`useDismiss.test.tsx:1201-1209`): React never renders
            // `open === false` there, so only the openchange emission observes the
            // session boundary and drops the press.
            store.set_open(true, &none_details());
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            store.set_open(false, &none_details());
            store.set_open(true, &none_details());
            assert_eq!(
                calls(&log).len(),
                5,
                "the log baseline: redundant open + dismissal, reopen, close/reopen pair"
            );

            // The gesture's trailing click belongs to the previous session and must
            // not dismiss the reopened element (`useDismiss.test.tsx:1224-1227`).
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert_eq!(
                calls(&log).len(),
                5,
                "the new session's first unbacked click does not dismiss"
            );

            // A press observed in the new session still closes
            // (`useDismiss.test.tsx:1229-1233`).
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            dispatch(outside.as_ref(), &mouse_event("click", 1, 0, 0));
            assert_eq!(
                closes(&log),
                vec![
                    reasons::OUTSIDE_PRESS.to_owned(),
                    reasons::NONE.to_owned(),
                    reasons::OUTSIDE_PRESS.to_owned(),
                ],
                "a press observed in the new session still closes"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the trigger-registry guard (`useDismiss.ts:407-416`): a press on a
    // registered trigger (or inside one) does not close the popup — switching
    // triggers must not close through dismissal.
    #[wasm_bindgen_test]
    fn a_press_on_a_registered_trigger_does_not_dismiss() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let trigger = document().create_element("button").unwrap();
            body().append_child(&trigger).unwrap();
            let mut triggers = PopupTriggerMap::new();
            triggers.add("trigger-1", trigger.clone());
            let store = store_with_triggers(true, Some(log.clone()), true, triggers);
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());

            dispatch(trigger.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert!(
                calls(&log).is_empty(),
                "the registered trigger press does not dismiss"
            );

            // A press inside the trigger's subtree is equally protected.
            let child = document().create_element("span").unwrap();
            trigger.append_child(&child).unwrap();
            dispatch(child.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert!(
                calls(&log).is_empty(),
                "a press inside the trigger is covered"
            );

            detach(&reference);
            detach(&floating);
            detach(&trigger);
        };
        __owner.cleanup();
    }

    // Pins the scrollbar hit-test (`useDismiss.ts:443-475` — Chromium-only upstream,
    // jsdom reports no scrollbars): a click on a scrollable element's scrollbar zone
    // does not dismiss; a click in its content area does.
    #[wasm_bindgen_test]
    fn a_click_on_a_scrollable_elements_scrollbar_does_not_dismiss() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(
                &store,
                UseDismissProps {
                    // The scrollbar check runs in `closeOnPressOutside` — drive it
                    // through the intentional click path (synthetic clicks report
                    // `detail: 0`, the press-less acceptance).
                    outside_press_event: OutsidePressEvent::Intentional,
                    ..UseDismissProps::default()
                },
            );

            let scroller = document().create_element("div").unwrap();
            scroller
                .set_attribute("style", "position: fixed; left: 0px; top: 0px; width: 50px; height: 50px; overflow-y: auto;")
                .unwrap();
            let content = document().create_element("div").unwrap();
            content.set_attribute("style", "height: 200px;").unwrap();
            scroller.append_child(&content).unwrap();
            body().append_child(&scroller).unwrap();
            // Force a layout so clientWidth/clientHeight/scrollWidth are measured.
            let _ = scroller.get_bounding_client_rect();

            // A click far to the right of the 50px-wide scroller lands on its
            // vertical scrollbar zone (`offsetX > clientWidth`).
            dispatch(scroller.as_ref(), &mouse_event("click", 0, 300, 10));
            assert!(
                calls(&log).is_empty(),
                "the scrollbar click does not dismiss"
            );

            // A click inside the content area (offsetX < clientWidth) dismisses.
            dispatch(scroller.as_ref(), &mouse_event("click", 0, 10, 10));
            assert_eq!(
                calls(&log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "the content-area click dismisses"
            );

            detach(&reference);
            detach(&floating);
            detach(&scroller);
        };
        __owner.cleanup();
    }

    // Pins the touch scroll-away dismissal (`useDismiss.ts:657-684`): a touchmove
    // more than 10px from its touchstart dismisses (sloppy mode).
    #[wasm_bindgen_test]
    fn a_touch_scroll_away_dismisses() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = store_with(true, Some(log.clone()), true);
            let (reference, floating, _cleanups) = attach_bags(&store, UseDismissProps::default());
            let outside = outside_div();

            dispatch(
                outside.as_ref(),
                &touch_event("touchstart", 100.0, 100.0).as_ref(),
            );
            assert!(
                calls(&log).is_empty(),
                "the touchstart alone does not dismiss"
            );

            // A 4px move stays under the 5px threshold: no dismissal yet.
            dispatch(
                outside.as_ref(),
                &touch_event("touchmove", 104.0, 100.0).as_ref(),
            );
            assert!(calls(&log).is_empty(), "a small move does not dismiss");

            // A 50px move crosses the 10px scroll-away threshold.
            dispatch(
                outside.as_ref(),
                &touch_event("touchmove", 150.0, 100.0).as_ref(),
            );
            assert_eq!(
                calls(&log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "the scroll-away dismisses with the outsidePress reason"
            );

            detach(&reference);
            detach(&floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the outside-press cascade (`useDismiss.test.tsx:649-712`): with the
    // default `bubbles`, one outside press closes a nested pair; with
    // `bubbles={{ outsidePress: false }}` on the inner popup, the press stops at the
    // innermost (the child's stamped flag blocks the parent).
    #[wasm_bindgen_test(async)]
    async fn the_outside_press_cascade_follows_the_child_bubbles_flag() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let tree: SharedFloatingTreeStore = SharedFloatingTreeStore::new(Rc::new(
                crate::floating_ui::tree::FloatingTreeStore::new(),
            ));

            // Both popups defer their state sync (the React commit lag), so the
            // blocking check reads the child's last-known open state during the
            // dispatch.
            let parent_log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let parent = store_with(true, Some(parent_log.clone()), false);
            parent.context.data_ref.borrow_mut().floating_node_id = Some("parent".to_owned());
            let (parent_reference, parent_floating, _parent_cleanups) = attach_bags(
                &parent,
                UseDismissProps {
                    external_tree: Some(tree.clone()),
                    ..UseDismissProps::default()
                },
            );

            let child_log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let child = store_with(true, Some(child_log.clone()), false);
            child.context.data_ref.borrow_mut().floating_node_id = Some("child".to_owned());
            tree.add_node(Rc::new(FloatingNodeType {
                id: Some("child".to_owned()),
                parent_id: Some("parent".to_owned()),
                context: RefCell::new(Some(context_for(&child))),
            }));
            let (child_reference, child_floating, _child_cleanups) = attach_bags(
                &child,
                UseDismissProps {
                    external_tree: Some(tree.clone()),
                    bubbles: Some(BubblesOption::PerKey {
                        escape_key: None,
                        outside_press: Some(false),
                    }),
                    ..UseDismissProps::default()
                },
            );

            let outside = outside_div();
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));

            assert_eq!(
                calls(&child_log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "the innermost popup dismisses"
            );
            assert!(
                calls(&parent_log).is_empty(),
                "the non-bubbling child blocks the parent's dismissal"
            );

            // The child's state syncs (the commit): the parent is no longer blocked.
            // The context's open mirror catches up on the executor tick.
            child.set_field(|state| &mut state.open, false);
            sleep(10).await;
            any_spawner::Executor::poll_local();
            dispatch(outside.as_ref(), &pointer_event("pointerdown", "mouse", 0));
            assert_eq!(
                calls(&parent_log),
                vec![(false, reasons::OUTSIDE_PRESS.to_owned())],
                "with the child closed, the parent dismisses on the next press"
            );

            detach(&parent_reference);
            detach(&parent_floating);
            detach(&child_reference);
            detach(&child_floating);
            detach(&outside);
        };
        __owner.cleanup();
    }

    // Pins the Escape cascade (`useDismiss.test.tsx:785-847`): with the default
    // non-bubbling `bubbles.escapeKey`, an open child blocks the parent's Escape
    // (first Escape closes only the inner); a bubbling child lets one Escape close
    // both.
    #[wasm_bindgen_test(async)]
    async fn the_escape_cascade_follows_the_child_bubbles_flag() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let tree: SharedFloatingTreeStore = SharedFloatingTreeStore::new(Rc::new(
                crate::floating_ui::tree::FloatingTreeStore::new(),
            ));

            let parent_log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let parent = store_with(true, Some(parent_log.clone()), false);
            parent.context.data_ref.borrow_mut().floating_node_id = Some("parent".to_owned());
            let (parent_reference, parent_floating, _parent_cleanups) = attach_bags(
                &parent,
                UseDismissProps {
                    external_tree: Some(tree.clone()),
                    ..UseDismissProps::default()
                },
            );

            // Bubbling child: `bubbles: true` stamps `__escapeKeyBubbles` true, so
            // the parent is not blocked.
            let child_log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let child = store_with(true, Some(child_log.clone()), false);
            child.context.data_ref.borrow_mut().floating_node_id = Some("child".to_owned());
            tree.add_node(Rc::new(FloatingNodeType {
                id: Some("child".to_owned()),
                parent_id: Some("parent".to_owned()),
                context: RefCell::new(Some(context_for(&child))),
            }));
            let (child_reference, child_floating, _child_cleanups) = attach_bags(
                &child,
                UseDismissProps {
                    external_tree: Some(tree.clone()),
                    bubbles: Some(BubblesOption::Bool(true)),
                    ..UseDismissProps::default()
                },
            );

            dispatch(body().as_ref(), &escape_keydown());
            assert_eq!(
                calls(&child_log),
                vec![(false, reasons::ESCAPE_KEY.to_owned())],
                "the inner popup dismisses"
            );
            assert_eq!(
                calls(&parent_log),
                vec![(false, reasons::ESCAPE_KEY.to_owned())],
                "the bubbling child lets the Escape close the parent in the same press"
            );

            detach(&parent_reference);
            detach(&parent_floating);
            detach(&child_reference);
            detach(&child_floating);
        };
        __owner.cleanup();
    }
}
