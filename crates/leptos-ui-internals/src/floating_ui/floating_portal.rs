//! Port of `packages/react/src/floating-ui-react/components/FloatingPortal.tsx` — portals
//! the floating element into a container element (by default `document.body`) so it can
//! escape clipping ancestors while staying logically in place
//! (`specs/library/floating-ui-react/behavior.md`, "DOM structure & portal behavior":
//! "`FloatingPortal` creates a `data-base-ui-portal` host under `document.body` by default,
//! accepts element/ref/lazy containers, re-parents on container switch, and feeds the
//! `aria-owns` owner").
//!
//! ## Upstream shape being ported
//!
//! - `FocusManagerState` (`FloatingPortal.tsx:29-38`) — the state a `FloatingFocusManager`
//!   inside the portal pushes up through the context; the portal uses it to decide whether
//!   to realize its outside guards (`:194-195`), route their Tab focus (`:258-292`), and
//!   close on focus out (`:285-290`).
//! - `PortalContext` + `usePortalContext` (`:40-49`) — `portalNode`, the
//!   `setFocusManagerState` setter, and the four focus-guard refs; consumed by
//!   `FloatingFocusManager` (inside guards + state push) and by nested
//!   `useFloatingPortalNode` calls (a nested portal renders into the parent portal node —
//!   implementation.md, "PortalContext").
//! - `useFloatingPortalNode` (`:71-154`) — the container-resolution state machine
//!   (`:95-125`): an explicitly `null` container waits (tearing any existing portal down);
//!   otherwise resolve the explicit node/ref → parent portal node → `document.body`; any
//!   change tears the host down and rebuilds it in the new container. The host `div` carries
//!   the generated `useId` id plus `data-base-ui-portal` (`:127-136`), and `nodeId` reads
//!   the rendered id so a consumer `id` override stays truthful for `aria-owns`
//!   (`:145-153`).
//! - `FloatingPortal` (`:165-298`) — the node machinery plus the non-modal tabbability
//!   guards: capture-phase `focusin`/`focusout` listeners on the portal node swap portal
//!   content's tabbability via `disableFocusInside`/`enableFocusInside` (`:198-226`), a
//!   reopen restores it before the focus manager's queued focus-on-open step (`:228-236`),
//!   and while a non-modal focus manager is active and open, the two outside `FocusGuard`
//!   spans and the hidden `aria-owns` owner element (`role={portalOwnerRole}`, `:270`) are
//!   realized around the portaled children (`:254-293`).
//!
//! ## Rust adaptations
//!
//! - The crate is view-free (the `csp_provider`/`direction_provider`/
//!   `floating_delay_group` convention), so upstream's React render output ports to
//!   imperative DOM work:
//!   - `useFloatingPortalNode`'s `createPortal(portalElement, containerElement)` subtree
//!     (`:138-143`) becomes the layout effect appending the created host `div` to the
//!     resolved container directly — DOM re-parenting is the portal; on container change the
//!     old host is removed and a new one appended, the same net DOM effect as upstream's
//!     unmount/remount (whose intermediate `setPortalNode(null)` render is unobservable
//!     synchronously).
//!   - The ref callback ignoring `null` (`:84-91`) disappears: the host is created and
//!     cleared imperatively, and the caller-supplied `node_ref` handle receives the element
//!     on creation and `None` on teardown (upstream's merged ref receives `null` from React
//!     on unmount while `setPortalNodeRef` ignores it).
//!   - `FloatingPortal`'s guard/owner spans are created when `shouldRenderGuards` turns true
//!     and removed (unmounted) when it turns false, and exposed on [`FloatingPortalHandle`]
//!     for the consumer to place — upstream renders them at the `FloatingPortal` call site
//!     in the React tree, a position the view-free port cannot know. The children portal
//!     (`:272`) becomes "the consumer appends into `node.node()`", and the children follow
//!     a rebuilt host on container change the way upstream's `createPortal` re-render does.
//! - React state ports to signals: `portalNode` (`:83`) and `focusManagerState` (`:188`)
//!   become `RwSignal`s — upstream's memoized context value keyed on `portalNode` (`:247`)
//!   re-rendered consumers, and the port's consumers read the signals instead. The four
//!   guard refs (`:183-186`) stay `Rc<RefCell<Option<_>>>` handles.
//! - The `container` prop's `HTMLElement | ShadowRoot | null | RefObject | undefined` union
//!   ports to [`FloatingPortalContainer`]: `null` and `undefined` are distinct variants (the
//!   wait branch `:97-104` vs the fall-through `:106-109`), and the node-or-ref case is a
//!   resolver closure re-read on every effect run (`isNode(containerProp) ? containerProp :
//!   containerProp.current`, `:107`; a `None` result falls through like a null ref current,
//!   the `??` chain).
//! - The `document.body` fallback uses the global document rather than `ownerDocument`
//!   (`:109`), faithfully — implementation.md "Container precedence" notes upstream assumes
//!   the main document there (shadow roots are supported only via an explicit `container`).
//! - `focusManagerState.onOpenChange` (`:32-35`) declares the narrow `{ reason?, event? }`
//!   data shape but is called with a full `createChangeEventDetails` result (`:288`); the
//!   port types it as [`RootOpenChangeEventDetails`], the structural superset.
//! - `FloatingPortalLite` (exercised by `FloatingPortal.test.tsx:160-171`) is *not* ported
//!   here — its source lives in `packages/react/src/utils/FloatingPortalLite.tsx`, outside
//!   this unit's file set (implementation.md, "Dependencies on other Base UI internals");
//!   it belongs to the `infra: utils` item.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, on_cleanup, provide_context, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, FocusEvent, HtmlElement, Node};

use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::merge_cleanups::{CleanupFn, merge_cleanups};
use leptos_ui_utils::use_id;
use leptos_ui_utils::use_iso_layout_effect;

use crate::floating_ui::create_attribute::create_attribute;
use crate::floating_ui::focus_guard::{FocusGuardProps, create_focus_guard};
use crate::floating_ui::reasons;
use crate::floating_ui::tabbable::{
    disable_focus_inside, enable_focus_inside, get_next_tabbable, get_previous_tabbable,
    is_outside_event,
};
use crate::floating_ui::types::{OnOpenChangeFn, RootOpenChangeEventDetails};

/// Port of `FocusManagerState` (`FloatingPortal.tsx:29-38`): the focus-manager state a
/// `FloatingFocusManager` inside the portal pushes up through [`PortalContextValue`].
#[derive(Clone)]
pub struct FocusManagerState {
    /// `modal` (`:30`).
    pub modal: bool,
    /// `open` (`:31`).
    pub open: bool,
    /// `onOpenChange` (`:32-35`) — `store.setOpen` in practice; the port types the details
    /// as the full change-details superset (see the module docs).
    pub on_open_change: OnOpenChangeFn,
    /// `domReference` (`:36`).
    pub dom_reference: Option<Element>,
    /// `closeOnFocusOut` (`:37`).
    pub close_on_focus_out: bool,
}

/// A focus-guard ref (`FloatingPortal.tsx:43-46`) — the `React.RefObject<HTMLSpanElement>`
/// handles the outside guards (and a FocusManager's inside guards) register into.
pub type SharedGuardRef = Rc<RefCell<Option<HtmlElement>>>;

/// Port of the `PortalContext` value (`FloatingPortal.tsx:40-47`).
pub struct PortalContextValue {
    /// `portalNode` (`:41`) — the mounted portal host. Upstream's `portalNode` React state
    /// keys the memoized context value (`:247`), so consumers re-render (and re-run their
    /// effects) when it changes; the port's signal stands in for both.
    pub portal_node: RwSignal<Option<HtmlElement>, LocalStorage>,
    /// `setFocusManagerState` (`:42`) — upstream's React state setter (`:188`); the port's
    /// FocusManager writes the twin signal.
    pub focus_manager_state: RwSignal<Option<FocusManagerState>, LocalStorage>,
    /// `beforeInsideRef` (`:43`).
    pub before_inside_ref: SharedGuardRef,
    /// `afterInsideRef` (`:44`).
    pub after_inside_ref: SharedGuardRef,
    /// `beforeOutsideRef` (`:45`).
    pub before_outside_ref: SharedGuardRef,
    /// `afterOutsideRef` (`:46`).
    pub after_outside_ref: SharedGuardRef,
}

/// The context handle — the `Rc` value rides behind a `SendWrapper` because
/// `provide_context` requires `Send + Sync` (the `SharedFloatingTreeStore`/
/// `SharedFloatingDelayGroupContext` bridge).
pub type SharedFloatingPortalContext = SendWrapper<Rc<PortalContextValue>>;

/// `usePortalContext` (`FloatingPortal.tsx:49`): the nearest provider's value, `None` when
/// the portal is not inside another one (upstream's `null` default).
pub fn use_portal_context() -> Option<SharedFloatingPortalContext> {
    use_context::<SharedFloatingPortalContext>()
}

/// The node-or-ref resolver — re-read on every layout-effect run, standing in for
/// upstream's re-rendered `containerProp` (see [`FloatingPortalContainer::Resolve`]).
pub type ContainerResolver = Rc<dyn Fn() -> ResolvedContainer>;

/// The per-run resolution of a [`FloatingPortalContainer::Resolve`] closure — the three
/// states `containerProp` itself can take (`FloatingPortal.tsx:55-56`).
pub enum ResolvedContainer {
    /// `container === null` (`:97-104`): wait — tear any existing portal down.
    Null,
    /// `container === undefined` or a null ref current (`:106-109`): fall through to the
    /// parent portal node, then `document.body` (the `??` chain skips a null ref current).
    Auto,
    /// An explicit node (`HTMLElement | ShadowRoot`).
    Node(Node),
}

/// Port of `UseFloatingPortalNodeProps['container']` (`FloatingPortal.tsx:55-56`).
pub enum FloatingPortalContainer {
    /// Always `null` (`:97-104`): wait for the container to be resolved — any existing
    /// portal tears down.
    Null,
    /// Always `undefined` (`:106-109`): resolve to the parent portal node (nested
    /// portals), then `document.body`.
    Auto,
    /// A node or ref: the closure resolves the current value on every effect run
    /// (`isNode(containerProp) ? containerProp : containerProp.current`, `:107`).
    Resolve(ContainerResolver),
}

/// A caller-owned handle receiving the portal host element — upstream's forwarded `ref`
/// (`FloatingPortal.tsx:54`): written on host creation, cleared on teardown (see the module
/// docs for the unmount null-delivery).
pub type SharedNodeRef = Rc<RefCell<Option<HtmlElement>>>;

/// Port of `UseFloatingPortalNodeProps` (`FloatingPortal.tsx:53-59`).
pub struct UseFloatingPortalNodeProps {
    /// `container` (`:55-56`) — default `undefined`, i.e. [`FloatingPortalContainer::Auto`].
    pub container: FloatingPortalContainer,
    /// `ref` (`:54`).
    pub node_ref: Option<SharedNodeRef>,
    /// The consumer `id` override (upstream reads it through the rendered element's props,
    /// `:147-152`; here it replaces the generated `useId` value on the host, and
    /// [`FloatingPortalNode::node_id`] prefers the mounted element's attribute so the
    /// `aria-owns` owner stays truthful either way).
    pub id: Option<String>,
    /// `elementProps` (`:58`) — the forwarded HTML attributes spread onto the host div
    /// (upstream's `data-testid`/`className`/... props, `FloatingPortal.test.tsx:126-139`).
    /// Applied after the generated id/`data-base-ui-portal` pair, so a caller `id` here
    /// overrides it the way the JSX spread order does.
    pub element_props: Vec<(String, String)>,
}

impl Default for UseFloatingPortalNodeProps {
    fn default() -> Self {
        Self {
            container: FloatingPortalContainer::Auto,
            node_ref: None,
            id: None,
            element_props: Vec::new(),
        }
    }
}

/// Port of `UseFloatingPortalNodeResult` (`FloatingPortal.tsx:61-69`).
#[derive(Clone)]
pub struct FloatingPortalNode {
    portal_node: RwSignal<Option<HtmlElement>, LocalStorage>,
    id_signal: Signal<String, LocalStorage>,
}

impl FloatingPortalNode {
    /// `node` (`:146`) — the mounted host element, `None` while the container is
    /// unresolved (upstream's initial/waiting state). The tracked read lets effects that
    /// depend on the host re-run on mount/unmount/container change.
    pub fn node(&self) -> Option<HtmlElement> {
        self.portal_node.get()
    }

    /// `nodeId` (`:147-152`) — "Use the exact rendered value so `aria-owns` never points at
    /// an ID absent from the DOM": the mounted element's `id` attribute when the host
    /// exists, else the rendered-value analog (override or generated id) for the
    /// not-yet-mounted host.
    pub fn node_id(&self) -> Option<String> {
        match self.portal_node.get_untracked() {
            Some(node) => node.get_attribute("id"),
            None => Some(self.id_signal.get_untracked()),
        }
    }

    /// The tracked `node` read the effects use.
    pub(crate) fn node_signal(&self) -> RwSignal<Option<HtmlElement>, LocalStorage> {
        self.portal_node
    }
}

/// Node-identity comparison — web-sys types compare as their underlying JS object
/// references, the strict-equals semantics (the `popupTriggerMap.ts:82-93` adaptation
/// note).
fn same_node(a: Option<&Node>, b: Option<&Node>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// The `document.body` fallback (`FloatingPortal.tsx:109`) — the global document, per the
/// module docs.
fn document_body() -> Option<Node> {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.body())
        .map(|body| body.into())
}

/// Port of `useFloatingPortalNode` (`FloatingPortal.tsx:71-154`). Must be called inside a
/// reactive owner (a component).
pub fn use_floating_portal_node(props: UseFloatingPortalNodeProps) -> FloatingPortalNode {
    let UseFloatingPortalNodeProps {
        container,
        node_ref,
        id,
        element_props,
    } = props;

    // `const uniqueId = useId()` (`:76`).
    let id_signal = use_id(Signal::derive_local(move || id.clone()), None);

    // `const portalContext = usePortalContext(); const parentPortalNode =
    // portalContext?.portalNode;` (`:77-78`) — a nested portal renders into the parent
    // portal node (`:108`). The signal handle is captured here (the parent context at call
    // time) and read inside the effect, standing in for the `[containerProp,
    // parentPortalNode]` dependency array (`:125`).
    let parent_portal_node = use_portal_context().map(|context| context.portal_node);

    // `const [portalNode, setPortalNode] = React.useState(null)` (`:83`).
    let portal_node_signal: RwSignal<Option<HtmlElement>, LocalStorage> = RwSignal::new_local(None);
    // `const containerRef = React.useRef(null)` (`:93`) — the change-detection ref.
    let container_ref: Rc<RefCell<Option<Node>>> = Rc::new(RefCell::new(None));

    let node_ref = node_ref.clone();
    let attr = create_attribute("portal");

    {
        let portal_node_signal = portal_node_signal.clone();
        let container_ref = Rc::clone(&container_ref);
        let id_signal = id_signal.clone();
        use_iso_layout_effect(move || {
            // The per-run `containerProp` read (upstream's dependency-array entry, `:125`).
            let value = match &container {
                FloatingPortalContainer::Null => ResolvedContainer::Null,
                FloatingPortalContainer::Auto => ResolvedContainer::Auto,
                FloatingPortalContainer::Resolve(resolve) => resolve(),
            };

            // `if (containerProp === null)` (`:97`): wait — tear any existing portal down.
            if matches!(value, ResolvedContainer::Null) {
                if container_ref.borrow().is_some() {
                    *container_ref.borrow_mut() = None;
                    if let Some(old) = portal_node_signal.get_untracked() {
                        let _ = old.remove();
                    }
                    portal_node_signal.set(None);
                    if let Some(node_ref) = &node_ref {
                        *node_ref.borrow_mut() = None;
                    }
                }
                return;
            }

            // The unconditional parent read mirrors the dependency-array entry (`:125`):
            // an explicit container ignores the value but still re-runs when it changes.
            let parent_node: Option<Node> = parent_portal_node
                .as_ref()
                .and_then(|signal| signal.get())
                .map(|element| element.into());

            // `(containerProp && (isNode(...) ? containerProp : containerProp.current)) ??
            // parentPortalNode ?? document.body` (`:106-109`).
            let resolved_from_prop = match value {
                ResolvedContainer::Auto => None,
                ResolvedContainer::Node(node) => Some(node),
                ResolvedContainer::Null => unreachable!("handled above"),
            };
            let resolved_container = resolved_from_prop.or(parent_node).or_else(document_body);

            let current = container_ref.borrow().clone();

            // `if (resolvedContainer == null)` (`:111-118`): tear down.
            if resolved_container.is_none() {
                if current.is_some() {
                    *container_ref.borrow_mut() = None;
                    if let Some(old) = portal_node_signal.get_untracked() {
                        let _ = old.remove();
                    }
                    portal_node_signal.set(None);
                    if let Some(node_ref) = &node_ref {
                        *node_ref.borrow_mut() = None;
                    }
                }
                return;
            }

            // `if (containerRef.current !== resolvedContainer)` (`:120-124`): rebuild in
            // the new container. Upstream's intermediate `setPortalNode(null)` render is
            // unobservable synchronously; the port removes the old host and appends the new
            // one in one pass (see the module docs).
            if !same_node(current.as_ref(), resolved_container.as_ref()) {
                *container_ref.borrow_mut() = resolved_container.clone();

                let old_host = portal_node_signal.get_untracked();
                if let Some(node_ref) = &node_ref {
                    *node_ref.borrow_mut() = None;
                }

                // `useRenderElement('div', ..., { props: [{ id: uniqueId, [attr]: '' },
                // elementProps] })` (`:127-136`): the host div with the generated id and
                // the `data-base-ui-portal` marker, then the forwarded attributes.
                let document = web_sys::window()
                    .expect("realm")
                    .document()
                    .expect("document");
                let host: HtmlElement = document
                    .create_element("div")
                    .expect("div creation")
                    .dyn_into()
                    .expect("div is an HtmlElement");
                host.set_attribute("id", &id_signal.get_untracked())
                    .expect("attribute name is valid");
                host.set_attribute(&attr, "")
                    .expect("attribute name is valid");
                for (name, value) in &element_props {
                    host.set_attribute(name, value)
                        .expect("attribute name is valid");
                }

                resolved_container
                    .as_ref()
                    .expect("resolved container")
                    .append_child(&host)
                    .expect("portal host appended");

                // `createPortal(children, portalNode)` (`:272`): the portaled children
                // follow the rebuilt host — upstream re-renders them into the new portal
                // node; the port moves them across.
                if let Some(old_host) = old_host {
                    while let Some(child) = old_host.first_child() {
                        host.append_child(&child).expect("child moved");
                    }
                    let _ = old_host.remove();
                }

                if let Some(node_ref) = &node_ref {
                    *node_ref.borrow_mut() = Some(host.clone());
                }
                portal_node_signal.set(Some(host));
            }
        });
    }

    FloatingPortalNode {
        portal_node: portal_node_signal,
        id_signal,
    }
}

/// `shouldRenderGuards` (`FloatingPortal.tsx:194-195`): `!!focusManagerState && !modal &&
/// open && !!portalNode` — extracted for the host-testable pin.
fn should_render_guards(
    focus_manager_state: Option<&FocusManagerState>,
    has_portal_node: bool,
) -> bool {
    match focus_manager_state {
        Some(state) => !state.modal && state.open && has_portal_node,
        None => false,
    }
}

/// A realized guard/owner element — see the module docs: the portal owns the lifecycle
/// (created when the guards turn active, removed when they turn inactive) while the
/// consumer places the element (upstream renders it at the `FloatingPortal` call site).
/// The guard's focus listener handle lives here too: dropping it would drop the listener
/// closure and detach the handler.
#[derive(Clone, Default)]
pub struct RealizedElement {
    element: Rc<RefCell<Option<HtmlElement>>>,
    listener: Rc<RefCell<Option<EventListenerUnsubscribe>>>,
}

impl RealizedElement {
    pub fn get(&self) -> Option<HtmlElement> {
        self.element.borrow().clone()
    }

    pub(crate) fn set(&self, element: HtmlElement, listener: Option<EventListenerUnsubscribe>) {
        let mut listener_slot = self.listener.borrow_mut();
        if let Some(old) = self.element.borrow_mut().replace(element) {
            let _ = old.remove();
        }
        *listener_slot = listener;
    }

    pub(crate) fn clear(&self) {
        let mut listener_slot = self.listener.borrow_mut();
        if let Some(old) = self.element.borrow_mut().take() {
            let _ = old.remove();
        }
        if let Some(unsubscribe) = listener_slot.take() {
            unsubscribe.unsubscribe();
        }
    }
}

/// Port of the `FloatingPortal` component props (`FloatingPortal.tsx:302-315`).
pub struct FloatingPortalOptions {
    /// `container` (`:308`) — forwarded to [`use_floating_portal_node`].
    pub container: FloatingPortalContainer,
    /// The forwarded ref (`:178`) — receives the portal host element.
    pub node_ref: Option<SharedNodeRef>,
    /// The `id` override (`BaseUIComponentProps.id`).
    pub id: Option<String>,
    /// The forwarded HTML attributes (`:169`'s `...elementProps`).
    pub element_props: Vec<(String, String)>,
    /// `portalOwnerRole` (`:313`) — the role for the hidden `aria-owns` owner element.
    pub portal_owner_role: Option<String>,
}

impl Default for FloatingPortalOptions {
    fn default() -> Self {
        Self {
            container: FloatingPortalContainer::Auto,
            node_ref: None,
            id: None,
            element_props: Vec::new(),
            portal_owner_role: None,
        }
    }
}

/// The handle returned by [`provide_floating_portal`].
pub struct FloatingPortalHandle {
    /// The node machinery this portal drives (`:172-181`).
    pub node: FloatingPortalNode,
    /// The provided context (`:253-295`) — what a nested portal or a FocusManager inside
    /// this portal consumes.
    pub context: SharedFloatingPortalContext,
    /// `shouldRenderGuards` (`:194-195`) — reactive: whether the outside guards and the
    /// `aria-owns` owner are currently realized.
    pub should_render_guards: RwSignal<bool, LocalStorage>,
    /// The realized outside guard spans (`:255-267`, `:273-293`) — `None` while inactive.
    pub before_outside_guard: RealizedElement,
    pub after_outside_guard: RealizedElement,
    /// The hidden `aria-owns` owner span (`:270`).
    pub aria_owns_owner: RealizedElement,
}

/// Builds the portal — the body of the `FloatingPortal` component, view-free (see the
/// module docs). The `useFloatingPortalNode` call happens before the context provision, so
/// the node machinery sees the *parent* portal context, not this portal's own (`:176` vs
/// `:253`). Must be called inside a reactive owner (a component).
pub fn provide_floating_portal(options: FloatingPortalOptions) -> FloatingPortalHandle {
    let FloatingPortalOptions {
        container,
        node_ref,
        id,
        element_props,
        portal_owner_role,
    } = options;

    let node = use_floating_portal_node(UseFloatingPortalNodeProps {
        container,
        node_ref,
        id,
        element_props,
    });
    let portal_node_signal = node.node_signal();

    // `const beforeOutsideRef = React.useRef(...)` etc. (`:183-186`).
    let before_outside_ref: SharedGuardRef = Rc::new(RefCell::new(None));
    let after_outside_ref: SharedGuardRef = Rc::new(RefCell::new(None));
    let before_inside_ref: SharedGuardRef = Rc::new(RefCell::new(None));
    let after_inside_ref: SharedGuardRef = Rc::new(RefCell::new(None));

    // `const [focusManagerState, setFocusManagerState] = React.useState(null)` (`:188`).
    let focus_manager_state: RwSignal<Option<FocusManagerState>, LocalStorage> =
        RwSignal::new_local(None);

    let context = Rc::new(PortalContextValue {
        portal_node: portal_node_signal,
        focus_manager_state: focus_manager_state.clone(),
        before_inside_ref: Rc::clone(&before_inside_ref),
        after_inside_ref: Rc::clone(&after_inside_ref),
        before_outside_ref: Rc::clone(&before_outside_ref),
        after_outside_ref: Rc::clone(&after_outside_ref),
    });

    // `const focusInsideDisabledRef = React.useRef(false)` (`:189`).
    let focus_inside_disabled = Rc::new(Cell::new(false));

    // The non-modal tabbability swap (`:198-226`): capture-phase focusin/focusout on the
    // portal node, re-registered per `[portalNode, modal]`.
    {
        let portal_node_signal = portal_node_signal.clone();
        let focus_manager_state = focus_manager_state.clone();
        let focus_inside_disabled = Rc::clone(&focus_inside_disabled);
        use_iso_layout_effect(move || {
            let state = focus_manager_state.get();
            // `const modal = focusManagerState?.modal` (`:191`).
            let modal = state.as_ref().map(|state| state.modal);
            let portal_node = portal_node_signal.get();

            // `if (!portalNode || modal) return undefined;` (`:199`).
            let Some(portal_node) = portal_node else {
                return;
            };
            if modal == Some(true) {
                return;
            }

            fn on_focus(
                event: &Event,
                portal_node: &HtmlElement,
                focus_inside_disabled: &Cell<bool>,
            ) {
                // `if (portalNode && event.relatedTarget && isOutsideEvent(event))` (`:207`).
                let focus_event: &FocusEvent = match event.dyn_ref() {
                    Some(focus_event) => focus_event,
                    None => return,
                };
                if focus_event.related_target().is_none() {
                    return;
                }
                if !is_outside_event(focus_event, Some(&portal_node)) {
                    return;
                }

                if event.type_() == "focusin" {
                    if focus_inside_disabled.get() {
                        enable_focus_inside(&portal_node);
                        focus_inside_disabled.set(false);
                    }
                } else {
                    disable_focus_inside(&portal_node);
                    focus_inside_disabled.set(true);
                }
            }

            let portal_node_for_focusin = portal_node.clone();
            let focus_inside_disabled_for_focusin = Rc::clone(&focus_inside_disabled);
            let focusin = leptos_ui_utils::add_event_listener_with_options(
                &portal_node,
                "focusin",
                move |event: &Event| {
                    on_focus(
                        event,
                        &portal_node_for_focusin,
                        &focus_inside_disabled_for_focusin,
                    );
                },
                true,
            );
            let portal_node_for_focusout = portal_node.clone();
            let focus_inside_disabled_for_focusout = Rc::clone(&focus_inside_disabled);
            let focusout = leptos_ui_utils::add_event_listener_with_options(
                &portal_node,
                "focusout",
                move |event: &Event| {
                    on_focus(
                        event,
                        &portal_node_for_focusout,
                        &focus_inside_disabled_for_focusout,
                    );
                },
                true,
            );

            let merged: Vec<Option<CleanupFn>> = vec![
                Some(Box::new(move || focusin.unsubscribe())),
                Some(Box::new(move || focusout.unsubscribe())),
            ];
            let merged = SendWrapper::new(merge_cleanups(merged));
            on_cleanup(move || merged.take()());
        });
    }

    // The reopen-restore (`:228-236`): restore tabbability before the focus manager's
    // queued focus-on-open step runs.
    {
        let portal_node_signal = portal_node_signal.clone();
        let focus_manager_state = focus_manager_state.clone();
        let focus_inside_disabled = Rc::clone(&focus_inside_disabled);
        use_iso_layout_effect(move || {
            let state = focus_manager_state.get();
            let open = state.as_ref().map(|state| state.open);
            let portal_node = portal_node_signal.get();

            // `if (!portalNode || open !== true || !focusInsideDisabledRef.current) return;`
            // (`:229`).
            let (Some(portal_node), Some(true)) = (portal_node, open) else {
                return;
            };
            if !focus_inside_disabled.get() {
                return;
            }

            enable_focus_inside(&portal_node);
            focus_inside_disabled.set(false);
        });
    }

    // The guards + `aria-owns` owner (`:254-293`, view-free — see the module docs):
    // created while `shouldRenderGuards` holds, removed when it stops.
    let before_outside_guard = RealizedElement::default();
    let after_outside_guard = RealizedElement::default();
    let aria_owns_owner = RealizedElement::default();
    let should_render_guards_signal: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
    {
        let node = node.clone();
        let portal_node_signal = portal_node_signal.clone();
        let focus_manager_state = focus_manager_state.clone();
        let should_render_guards_signal = should_render_guards_signal.clone();
        let before_outside_guard = before_outside_guard.clone();
        let after_outside_guard = after_outside_guard.clone();
        let aria_owns_owner = aria_owns_owner.clone();
        let before_outside_ref = Rc::clone(&before_outside_ref);
        let after_outside_ref = Rc::clone(&after_outside_ref);
        let before_inside_ref = Rc::clone(&before_inside_ref);
        let after_inside_ref = Rc::clone(&after_inside_ref);
        let portal_owner_role = portal_owner_role.clone();
        use_iso_layout_effect(move || {
            let state = focus_manager_state.get();
            let has_portal_node = portal_node_signal.get().is_some();
            let should = should_render_guards(state.as_ref(), has_portal_node);
            should_render_guards_signal.set(should);

            if !should {
                before_outside_guard.clear();
                after_outside_guard.clear();
                aria_owns_owner.clear();
                // The unmount null-delivery (`:257`, `:276`).
                *before_outside_ref.borrow_mut() = None;
                *after_outside_ref.borrow_mut() = None;
                return;
            }

            if before_outside_guard.get().is_none() {
                // `<FocusGuard data-type="outside" ref={beforeOutsideRef} onFocus={...}>`
                // (`:255-267`).
                let guard = create_focus_guard(FocusGuardProps {
                    attributes: vec![("data-type".to_owned(), "outside".to_owned())],
                });
                let focus_handler = before_outside_focus_handler(
                    portal_node_signal.clone(),
                    focus_manager_state.clone(),
                    Rc::clone(&before_inside_ref),
                );
                let listener = leptos_ui_utils::add_event_listener(&guard, "focus", focus_handler);
                *before_outside_ref.borrow_mut() = Some(guard.clone());
                before_outside_guard.set(guard, Some(listener));
            }

            if aria_owns_owner.get().is_none() {
                // `<span role={portalOwnerRole} aria-owns={portalNodeId}
                // style={ownerVisuallyHidden} />` (`:270`).
                let owner: HtmlElement = web_sys::window()
                    .expect("realm")
                    .document()
                    .expect("document")
                    .create_element("span")
                    .expect("span creation")
                    .dyn_into()
                    .expect("span is an HtmlElement");
                if let Some(role) = &portal_owner_role {
                    owner
                        .set_attribute("role", role)
                        .expect("attribute name is valid");
                }
                if let Some(node_id) = node.node_id() {
                    owner
                        .set_attribute("aria-owns", &node_id)
                        .expect("attribute name is valid");
                }
                // The `ownerVisuallyHidden` style
                // (`packages/react/src/internals/constants.ts:36-41`), provisionally hosted
                // here — it belongs to the `infra: internals` constants the same way
                // `focus_guard` belongs to `infra: utils`.
                for (name, value) in OWNER_VISUALLY_HIDDEN {
                    owner.style().set_property(name, value).expect("style set");
                }
                aria_owns_owner.set(owner, None);
            }

            if after_outside_guard.get().is_none() {
                // `<FocusGuard data-type="outside" ref={afterOutsideRef} onFocus={...}>`
                // (`:274-293`).
                let guard = create_focus_guard(FocusGuardProps {
                    attributes: vec![("data-type".to_owned(), "outside".to_owned())],
                });
                let focus_handler = after_outside_focus_handler(
                    portal_node_signal.clone(),
                    focus_manager_state.clone(),
                    Rc::clone(&after_inside_ref),
                );
                let listener = leptos_ui_utils::add_event_listener(&guard, "focus", focus_handler);
                *after_outside_ref.borrow_mut() = Some(guard.clone());
                after_outside_guard.set(guard, Some(listener));
            }
        });
    }

    // `const portalContextValue = React.useMemo(...)` + the provider (`:238-248`, `:253`).
    let shared_context = SharedFloatingPortalContext::new(Rc::clone(&context));
    provide_context(shared_context.clone());

    FloatingPortalHandle {
        node,
        context: shared_context,
        should_render_guards: should_render_guards_signal,
        before_outside_guard,
        after_outside_guard,
        aria_owns_owner,
    }
}

/// The `ownerVisuallyHidden` style (`packages/react/src/internals/constants.ts:36-41`) —
/// provisionally hosted here pending the `infra: internals` constants port (the
/// `focus_guard` precedent).
pub const OWNER_VISUALLY_HIDDEN: &[(&str, &str)] = &[
    ("clip-path", "inset(50%)"),
    ("position", "fixed"),
    ("top", "0px"),
    ("left", "0px"),
];

/// `.focus()` on a tabbable element — upstream's `?.focus()` calls on the
/// `FocusableElement` union (`:264`, `:282`): `HTMLElement` or `SVGElement` share the focus
/// method through the `HTMLOrSVGElement` mixin (the `tabbable` module's `FocusableElement`
/// adaptation).
fn focus_element(element: &Element) {
    if let Some(html) = element.dyn_ref::<HtmlElement>() {
        let _ = html.focus();
        return;
    }
    if let Some(svg) = element.dyn_ref::<web_sys::SvgElement>() {
        let _ = svg.focus();
    }
}

/// The before-outside guard's focus handler (`FloatingPortal.tsx:258-266`): route Tab back
/// inside, or to the previous document tabbable relative to the reference. The focus
/// manager state is read fresh at event time (upstream's handler closure is re-created per
/// render; the port reads the signal).
fn before_outside_focus_handler(
    portal_node_signal: RwSignal<Option<HtmlElement>, LocalStorage>,
    focus_manager_state: RwSignal<Option<FocusManagerState>, LocalStorage>,
    before_inside_ref: SharedGuardRef,
) -> impl FnMut(&Event) + 'static {
    move |event: &Event| {
        let focus_event: &FocusEvent = match event.dyn_ref() {
            Some(focus_event) => focus_event,
            None => return,
        };
        let portal_node = portal_node_signal.get_untracked();
        // `if (isOutsideEvent(event, portalContext.portalNode)) {
        //   beforeInsideRef.current?.focus(); }` (`:259-261`).
        if is_outside_event(focus_event, portal_node.as_deref()) {
            if let Some(inside) = before_inside_ref.borrow().as_ref() {
                let _ = inside.focus();
            }
            return;
        }

        // `const domReference = focusManagerState ? focusManagerState.domReference : null;
        // getPreviousTabbable(domReference)?.focus();` (`:262-265`).
        let dom_reference = focus_manager_state
            .get_untracked()
            .and_then(|state| state.dom_reference);
        if let Some(previous) = get_previous_tabbable(dom_reference.as_ref()) {
            focus_element(&previous);
        }
    }
}

/// The after-outside guard's focus handler (`FloatingPortal.tsx:277-292`): route Tab back
/// inside, or to the next document tabbable, closing the popup on focus out when the focus
/// manager asked for it.
fn after_outside_focus_handler(
    portal_node_signal: RwSignal<Option<HtmlElement>, LocalStorage>,
    focus_manager_state: RwSignal<Option<FocusManagerState>, LocalStorage>,
    after_inside_ref: SharedGuardRef,
) -> impl FnMut(&Event) + 'static {
    move |event: &Event| {
        let focus_event: &FocusEvent = match event.dyn_ref() {
            Some(focus_event) => focus_event,
            None => return,
        };
        let portal_node = portal_node_signal.get_untracked();
        // `if (isOutsideEvent(event, portalContext.portalNode)) {
        //   afterInsideRef.current?.focus(); }` (`:278-280`).
        if is_outside_event(focus_event, portal_node.as_deref()) {
            if let Some(inside) = after_inside_ref.borrow().as_ref() {
                let _ = inside.focus();
            }
            return;
        }

        // `getNextTabbable(domReference)?.focus()` (`:281-283`).
        let dom_reference = focus_manager_state
            .get_untracked()
            .and_then(|state| state.dom_reference.clone());
        if let Some(next) = get_next_tabbable(dom_reference.as_ref()) {
            focus_element(&next);
        }

        // `if (focusManagerState?.closeOnFocusOut) { focusManagerState?.onOpenChange(false,
        // createChangeEventDetails(REASONS.focusOut, event.nativeEvent)); }` (`:285-290`).
        if let Some(state) = focus_manager_state.get_untracked() {
            if state.close_on_focus_out {
                (state.on_open_change)(
                    false,
                    &RootOpenChangeEventDetails::new(
                        reasons::FOCUS_OUT,
                        event.clone().into(),
                        None,
                        String::new(),
                    ),
                );
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    fn focus_manager_state(modal: bool, open: bool, close_on_focus_out: bool) -> FocusManagerState {
        FocusManagerState {
            modal,
            open,
            on_open_change: Rc::new(|_, _| {}),
            dom_reference: None,
            close_on_focus_out,
        }
    }

    // `shouldRenderGuards` (`FloatingPortal.tsx:194-195`): requires a focus manager state,
    // non-modal, open, and a mounted portal node.
    #[test]
    fn guards_render_only_for_an_active_non_modal_focus_manager_and_a_mounted_node() {
        assert!(
            !should_render_guards(None, true),
            "no focus manager state pushed yet"
        );
        let modal = focus_manager_state(true, true, true);
        assert!(
            !should_render_guards(Some(&modal), true),
            "modal managers render no outside guards"
        );
        let closed = focus_manager_state(false, false, true);
        assert!(
            !should_render_guards(Some(&closed), true),
            "the focus manager is not open"
        );
        let active = focus_manager_state(false, true, true);
        assert!(
            !should_render_guards(Some(&active), false),
            "the portal node is not mounted"
        );
        assert!(should_render_guards(Some(&active), true));
    }

    // The `ownerVisuallyHidden` shape (`packages/react/src/internals/constants.ts:36-41`):
    // the four declarations the `aria-owns` owner span carries.
    #[test]
    fn the_owner_visually_hidden_style_matches_upstream() {
        assert_eq!(
            OWNER_VISUALLY_HIDDEN,
            &[
                ("clip-path", "inset(50%)"),
                ("position", "fixed"),
                ("top", "0px"),
                ("left", "0px"),
            ]
        );
    }
}

// The portal machinery runs through the ported `use_iso_layout_effect`, whose browser
// binding only exists under the wasm/browser target — every behavioral pin therefore runs
// in the browser, like the crate's other effect-driven suites.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    use web_sys::EventTarget;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect("realm")
            .document()
            .expect("document")
    }

    fn body() -> Element {
        document()
            .body()
            .expect("body should exist")
            .unchecked_into()
    }

    /// Tracks every top-level fixture element a test appends to the body so the
    /// `afterEach` body sweep can run as a drop-time `Element.remove()` per element —
    /// removal, not `innerHTML` clearing, keeps the wasm-bindgen-test harness alive
    /// (the `mark_others`/`tabbable` precedent).
    struct Fixture(Vec<Element>);

    impl Fixture {
        fn append(&mut self, element: &Element) {
            body().append_child(element).expect("append should succeed");
            self.0.push(element.clone());
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for element in self.0.drain(..) {
                element.remove();
            }
        }
    }

    fn create_element(tag: &str) -> Element {
        document()
            .create_element(tag)
            .unwrap_or_else(|_| panic!("{tag} should be creatable"))
    }

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// Drains the executor so the tracked effects re-run after their inputs changed — the
    /// first `RenderEffect` run is synchronous, re-runs execute on an executor poll (the
    /// `floating_delay_group` harness note).
    fn flush() {
        for _ in 0..8 {
            any_spawner::Executor::poll_local();
        }
    }

    /// A `FocusEvent` with a `relatedTarget` — the portal's focus handling reads it
    /// (`FloatingPortal.tsx:207`).
    fn focus_event(event_type: &str, related_target: &Element) -> FocusEvent {
        let init = web_sys::FocusEventInit::new();
        init.set_related_target(Some(related_target.unchecked_ref::<EventTarget>()));
        FocusEvent::new_with_focus_event_init_dict(event_type, &init)
            .expect("focus event should construct")
    }

    fn inert_on_open_change() -> OnOpenChangeFn {
        Rc::new(|_, _| {})
    }

    fn focus_manager_state(
        modal: bool,
        open: bool,
        close_on_focus_out: bool,
        dom_reference: Option<Element>,
        on_open_change: OnOpenChangeFn,
    ) -> FocusManagerState {
        FocusManagerState {
            modal,
            open,
            on_open_change,
            dom_reference,
            close_on_focus_out,
        }
    }

    // `allows custom containers` (`FloatingPortal.test.tsx:31-44`): the host mounts inside
    // the given container and carries the `data-base-ui-portal` attribute.
    #[wasm_bindgen_test]
    fn allows_custom_containers() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let custom_root = create_element("div");
        custom_root.set_id("custom-root");
        fixture.append(&custom_root);

        let handle = {
            let custom_root = custom_root.clone();
            provide_floating_portal(FloatingPortalOptions {
                container: FloatingPortalContainer::Resolve(Rc::new(move || {
                    ResolvedContainer::Node(custom_root.clone().into())
                })),
                ..FloatingPortalOptions::default()
            })
        };
        flush();

        // The floating element portals into the host (`:36-42`).
        let floating = create_element("div");
        floating.set_id("floating");
        let host = handle.node.node().expect("portal host mounted");
        host.append_child(&floating).expect("floating appended");

        let parent = floating.parent_element().expect("host is the parent");
        assert!(parent.has_attribute("data-base-ui-portal"));
        assert_eq!(parent.parent_element().as_ref(), Some(&custom_root));

        // The forwarded ref receives the host (`:54`).
        let node_ref: SharedNodeRef = Rc::new(RefCell::new(None));
        let tracked = create_element("div");
        fixture.append(&tracked);
        let handle = {
            let tracked = tracked.clone();
            provide_floating_portal(FloatingPortalOptions {
                container: FloatingPortalContainer::Resolve(Rc::new(move || {
                    ResolvedContainer::Node(tracked.clone().into())
                })),
                node_ref: Some(node_ref.clone()),
                ..FloatingPortalOptions::default()
            })
        };
        flush();
        let ref_current: Option<Element> = node_ref.borrow().clone().map(|node| node.into());
        let host: Option<Element> = handle.node.node().map(|node| node.into());
        assert_eq!(ref_current.as_ref(), host.as_ref());
    }

    // `allows refs as containers` (`FloatingPortal.test.tsx:46-57`): the resolver reads the
    // ref's current value on every effect run.
    #[wasm_bindgen_test]
    fn allows_refs_as_containers() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let el = create_element("div");
        fixture.append(&el);
        let container_cell: Rc<RefCell<Option<Node>>> =
            Rc::new(RefCell::new(Some(el.clone().into())));

        let handle = {
            let container_cell = Rc::clone(&container_cell);
            provide_floating_portal(FloatingPortalOptions {
                container: FloatingPortalContainer::Resolve(Rc::new(move || match container_cell
                    .borrow()
                    .as_ref()
                {
                    Some(node) => ResolvedContainer::Node(node.clone()),
                    None => ResolvedContainer::Auto,
                })),
                ..FloatingPortalOptions::default()
            })
        };
        flush();

        let floating = create_element("div");
        let host = handle.node.node().expect("portal host mounted");
        host.append_child(&floating).expect("floating appended");

        let parent = floating.parent_element().expect("host is the parent");
        assert!(parent.has_attribute("data-base-ui-portal"));
        assert_eq!(parent.parent_element().as_ref(), Some(&el));
    }

    // `allows containers to be initially null` (`FloatingPortal.test.tsx:59-84`): the
    // explicit-null container waits; the portal mounts once it resolves.
    #[wasm_bindgen_test]
    fn allows_containers_to_be_initially_null() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let root = create_element("div");
        root.set_id("root");
        fixture.append(&root);

        let container_cell: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
        let version: RwSignal<u32, LocalStorage> = RwSignal::new_local(0);
        let handle = {
            let container_cell = Rc::clone(&container_cell);
            let version = version.clone();
            provide_floating_portal(FloatingPortalOptions {
                container: FloatingPortalContainer::Resolve(Rc::new(move || {
                    let _ = version.get();
                    match container_cell.borrow().as_ref() {
                        Some(element) => ResolvedContainer::Node(element.clone().into()),
                        None => ResolvedContainer::Null,
                    }
                })),
                ..FloatingPortalOptions::default()
            })
        };
        flush();
        assert!(
            handle.node.node().is_none(),
            "the portal waits while the container is explicitly null"
        );

        // The container resolves (`:64-66` — the ref callback sets it on render).
        *container_cell.borrow_mut() = Some(root.clone());
        version.set(version.get_untracked() + 1);
        flush();

        let floating = create_element("div");
        let host = handle.node.node().expect("portal host mounted");
        host.append_child(&floating).expect("floating appended");
        assert_eq!(
            host.parent_element().as_ref(),
            Some(&root),
            "the resolved container wins over the body fallback"
        );
    }

    // `reattaches the portal when the container changes` (`FloatingPortal.test.tsx:86-124`):
    // body → custom root → body, with the portaled children following and no leftovers in
    // the previous container.
    #[wasm_bindgen_test]
    fn reattaches_the_portal_when_the_container_changes() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let custom_root = create_element("div");
        fixture.append(&custom_root);

        let mode: RwSignal<u32, LocalStorage> = RwSignal::new_local(0);
        let handle = {
            let custom_root = custom_root.clone();
            let mode = mode.clone();
            provide_floating_portal(FloatingPortalOptions {
                container: FloatingPortalContainer::Resolve(Rc::new(move || match mode.get() {
                    0 => ResolvedContainer::Auto,
                    _ => ResolvedContainer::Node(custom_root.clone().into()),
                })),
                ..FloatingPortalOptions::default()
            })
        };
        flush();

        let floating = create_element("div");
        let body_host: Element = handle.node.node().expect("portal host mounted").into();
        body_host
            .append_child(&floating)
            .expect("floating appended");
        assert_eq!(
            body_host.parent_element(),
            Some(body()),
            "the undefined container falls through to document.body"
        );

        // Switch to the custom root (`:112-114`).
        mode.set(1);
        flush();
        let custom_host: Element = handle.node.node().expect("rebuilt host").into();
        assert_eq!(custom_host.parent_element().as_ref(), Some(&custom_root));
        assert_eq!(
            floating.parent_element().as_ref(),
            Some(&custom_host),
            "the portaled children follow the rebuilt host (`:272`)"
        );

        // Switch back to the body (`:116-120`).
        mode.set(0);
        flush();
        let final_host: Element = handle.node.node().expect("rebuilt host").into();
        assert_eq!(final_host.parent_element(), Some(body()));
        assert_eq!(floating.parent_element().as_ref(), Some(&final_host));
        assert!(
            !custom_root.contains(Some(final_host.unchecked_ref())),
            "after switching away, the floating content is no longer contained by the \
             previous container"
        );

        fixture.append(&final_host);
    }

    // `forwards HTML props to the portal element` (`FloatingPortal.test.tsx:126-139`).
    #[wasm_bindgen_test]
    fn forwards_html_props_to_the_portal_element() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let handle = provide_floating_portal(FloatingPortalOptions {
            element_props: vec![
                ("data-testid".to_owned(), "portal-element".to_owned()),
                ("class".to_owned(), "closed".to_owned()),
            ],
            ..FloatingPortalOptions::default()
        });
        flush();

        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);
        assert!(host.has_attribute("data-base-ui-portal"));
        assert_eq!(
            host.get_attribute("data-testid").as_deref(),
            Some("portal-element")
        );
        assert_eq!(host.get_attribute("class").as_deref(), Some("closed"));
    }

    // `uses the rendered portal ID for the aria-owns relationship`
    // (`FloatingPortal.test.tsx:141-158`): the consumer id override lands on the host and
    // the owner span's `aria-owns` points at it, once a non-modal focus manager is active.
    #[wasm_bindgen_test]
    fn uses_the_rendered_portal_id_for_the_aria_owns_relationship() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let handle = provide_floating_portal(FloatingPortalOptions {
            id: Some("custom-portal".to_owned()),
            ..FloatingPortalOptions::default()
        });
        flush();
        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);
        assert_eq!(host.get_attribute("id").as_deref(), Some("custom-portal"));

        // The FocusManager pushes its state up through the portal context
        // (`FloatingFocusManager.tsx:924-940` — the mechanism contract the FocusManager
        // port consumes).
        handle
            .context
            .focus_manager_state
            .set(Some(focus_manager_state(
                false,
                true,
                true,
                None,
                inert_on_open_change(),
            )));
        flush();

        assert!(handle.should_render_guards.get_untracked());
        let owner = handle.aria_owns_owner.get().expect("owner realized");
        assert_eq!(
            owner.get_attribute("aria-owns").as_deref(),
            Some("custom-portal")
        );
        assert!(
            !owner.has_attribute("role"),
            "the owner has no role by default (`FloatingFocusManager.test.tsx:1793-1835`)"
        );
        assert_eq!(
            owner
                .style()
                .get_property_value("position")
                .expect("position readable"),
            "fixed",
            "the owner is visually hidden (`ownerVisuallyHidden`)"
        );

        // The outside guards are realized with the guard contract
        // (`:255-267`/`:274-293` + `FocusGuard.tsx:26-40`).
        let before = handle.before_outside_guard.get().expect("before guard");
        assert_eq!(
            before.get_attribute("data-type").as_deref(),
            Some("outside")
        );
        assert!(before.has_attribute("data-base-ui-focus-guard"));
        assert_eq!(before.get_attribute("tabindex").as_deref(), Some("0"));
        assert_eq!(before.get_attribute("aria-hidden").as_deref(), Some("true"));
        let after = handle.after_outside_guard.get().expect("after guard");
        assert_eq!(after.get_attribute("data-type").as_deref(), Some("outside"));

        // Deactivating the focus manager unmounts them (`:937-939`).
        handle.context.focus_manager_state.set(None);
        flush();
        assert!(!handle.should_render_guards.get_untracked());
        assert!(handle.aria_owns_owner.get().is_none());
        assert!(handle.before_outside_guard.get().is_none());
        assert!(handle.after_outside_guard.get().is_none());
    }

    // `portalOwnerRole` sets the owner's role
    // (`FloatingFocusManager.test.tsx:1837-1875`).
    #[wasm_bindgen_test]
    fn portal_owner_role_sets_the_owner_role() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let handle = provide_floating_portal(FloatingPortalOptions {
            portal_owner_role: Some("group".to_owned()),
            ..FloatingPortalOptions::default()
        });
        flush();
        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);

        handle
            .context
            .focus_manager_state
            .set(Some(focus_manager_state(
                false,
                true,
                true,
                None,
                inert_on_open_change(),
            )));
        flush();

        let owner = handle.aria_owns_owner.get().expect("owner realized");
        assert_eq!(owner.get_attribute("role").as_deref(), Some("group"));
    }

    // The two-level portal (`FloatingPortal.tsx:108` — implementation.md "PortalContext"):
    // a portal created inside another one's context renders into the parent portal node.
    #[wasm_bindgen_test]
    fn a_nested_portal_renders_into_the_parent_portal_node() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let parent_owner = reactive_graph::owner::Owner::new();
        parent_owner.set();
        let parent = provide_floating_portal(FloatingPortalOptions::default());
        flush();

        // Nests under the parent's owner, so `usePortalContext` finds the parent context.
        let child_owner = reactive_graph::owner::Owner::new();
        child_owner.set();
        let child = provide_floating_portal(FloatingPortalOptions::default());
        flush();

        let parent_host: Element = parent.node.node().expect("parent host").into();
        fixture.append(&parent_host);
        let child_host: Element = child.node.node().expect("child host").into();
        assert_eq!(
            child_host.parent_element().as_ref(),
            Some(&parent_host),
            "the nested portal's host mounts inside the parent portal node"
        );
    }

    // The non-modal tabbability swap (`FloatingPortal.tsx:198-226`): focusout to an
    // outside target disables focus inside the portal node; focusin from outside restores
    // it.
    #[wasm_bindgen_test]
    fn the_non_modal_tabbability_swap_tracks_focus_crossings() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let handle = provide_floating_portal(FloatingPortalOptions::default());
        flush();
        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);

        let inside_button = create_element("button");
        host.append_child(&inside_button).expect("inside button");
        let outside_button = create_element("button");
        fixture.append(&outside_button);

        // focusout to an outside target (`:213-216`).
        let event = focus_event("focusout", &outside_button);
        inside_button
            .dispatch_event(&event)
            .expect("focusout dispatched");
        assert_eq!(
            inside_button.get_attribute("tabindex").as_deref(),
            Some("-1"),
            "disableFocusInside swaps the tabbability"
        );
        assert!(
            inside_button.has_attribute("data-tabindex"),
            "the data-tabindex mirror is written"
        );

        // focusin from an outside target (`:208-212`).
        let event = focus_event("focusin", &outside_button);
        inside_button
            .dispatch_event(&event)
            .expect("focusin dispatched");
        assert!(
            !inside_button.has_attribute("tabindex"),
            "enableFocusInside restores the recorded absence"
        );
        assert!(!inside_button.has_attribute("data-tabindex"));
    }

    // The reopen-restore (`FloatingPortal.tsx:228-236`): a disabled portal restores
    // tabbability when the focus manager reports open again.
    #[wasm_bindgen_test]
    fn a_reopen_restores_tabbability_before_the_queued_focus_on_open() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let handle = provide_floating_portal(FloatingPortalOptions::default());
        flush();
        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);

        let inside_button = create_element("button");
        host.append_child(&inside_button).expect("inside button");
        let outside_button = create_element("button");
        fixture.append(&outside_button);

        let event = focus_event("focusout", &outside_button);
        inside_button
            .dispatch_event(&event)
            .expect("focusout dispatched");
        assert_eq!(
            inside_button.get_attribute("tabindex").as_deref(),
            Some("-1")
        );

        // The focus manager re-opens (`:229-235`).
        handle
            .context
            .focus_manager_state
            .set(Some(focus_manager_state(
                false,
                true,
                true,
                None,
                inert_on_open_change(),
            )));
        flush();

        assert!(
            !inside_button.has_attribute("tabindex"),
            "tabbability is restored before the focus manager's queued focus-on-open step"
        );
        assert!(!inside_button.has_attribute("data-tabindex"));
    }

    // A modal focus manager state realizes no guards and keeps the tabbability listeners
    // off (`:199` — `if (!portalNode || modal)`).
    #[wasm_bindgen_test]
    fn a_modal_focus_manager_gets_no_guards_and_no_tabbability_swap() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let handle = provide_floating_portal(FloatingPortalOptions::default());
        flush();
        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);

        let inside_button = create_element("button");
        host.append_child(&inside_button).expect("inside button");
        let outside_button = create_element("button");
        fixture.append(&outside_button);

        handle
            .context
            .focus_manager_state
            .set(Some(focus_manager_state(
                true,
                true,
                true,
                None,
                inert_on_open_change(),
            )));
        flush();

        assert!(
            !handle.should_render_guards.get_untracked(),
            "modal managers render no outside guards"
        );

        let event = focus_event("focusout", &outside_button);
        inside_button
            .dispatch_event(&event)
            .expect("focusout dispatched");
        assert!(
            !inside_button.has_attribute("tabindex"),
            "the tabbability swap does not engage for modal managers"
        );
    }

    // The outside guards' focus routing (`FloatingPortal.tsx:258-292`): a focus arrival
    // from inside the portal routes to the next/previous document tabbable relative to the
    // reference, and the after guard closes on focus out when asked.
    #[wasm_bindgen_test]
    fn the_outside_guards_route_tab_and_close_on_focus_out() {
        init_executor();
        let mut fixture = Fixture(Vec::new());

        let previous = create_element("button");
        previous.set_id("guard-prev");
        fixture.append(&previous);
        let reference = create_element("button");
        reference.set_id("guard-ref");
        fixture.append(&reference);
        let next = create_element("button");
        next.set_id("guard-next");
        fixture.append(&next);

        let calls: Rc<RefCell<Vec<(bool, String)>>> = Rc::new(RefCell::new(Vec::new()));
        let handle = provide_floating_portal(FloatingPortalOptions::default());
        flush();
        let host: Element = handle.node.node().expect("portal host mounted").into();
        fixture.append(&host);

        let calls_for_state = Rc::clone(&calls);
        handle
            .context
            .focus_manager_state
            .set(Some(focus_manager_state(
                false,
                true,
                true,
                Some(reference.clone()),
                Rc::new(move |open: bool, details: &RootOpenChangeEventDetails| {
                    calls_for_state
                        .borrow_mut()
                        .push((open, details.reason.clone()));
                }),
            )));
        flush();

        // Place the guards at the portal call site (upstream renders them there; the
        // view-free port's consumer places them — see the module docs).
        let before_guard = handle.before_outside_guard.get().expect("before guard");
        fixture.append(&before_guard);
        let after_guard = handle.after_outside_guard.get().expect("after guard");
        fixture.append(&after_guard);

        // Tab backward out of the popup: the before guard routes to the previous document
        // tabbable relative to the active element (`:262-265` — getPreviousTabbable steps
        // from the document's active element, `tabbable.ts:219-224`) and does not close
        // (`:258-266` has no onOpenChange call).
        let _ = reference.unchecked_ref::<web_sys::HtmlElement>().focus();
        assert_eq!(
            document().active_element().as_ref(),
            Some(&reference),
            "the reference is focused before the shift-tab"
        );
        let event = focus_event("focus", &host);
        before_guard
            .dispatch_event(&event)
            .expect("before guard focus dispatched");
        let active = document().active_element();
        assert_eq!(
            active.as_ref(),
            Some(&previous),
            "the previous document tabbable is focused (active id = {:?})",
            active.as_ref().and_then(|el| el.get_attribute("id"))
        );
        assert!(
            calls.borrow().is_empty(),
            "the before guard never closes the popup"
        );

        // Tab forward out of the popup: the after guard routes to the next document
        // tabbable (`:281-283`) and closes the popup with the focusOut reason
        // (`:285-290`).
        let _ = reference.unchecked_ref::<web_sys::HtmlElement>().focus();
        let event = focus_event("focus", &host);
        after_guard
            .dispatch_event(&event)
            .expect("after guard focus dispatched");
        let active = document().active_element();
        assert_eq!(
            active.as_ref(),
            Some(&next),
            "the next document tabbable is focused (active id = {:?})",
            active.as_ref().and_then(|el| el.get_attribute("id"))
        );
        assert_eq!(
            calls.borrow().as_slice(),
            [(false, "focus-out".to_owned())],
            "closeOnFocusOut closes through the pushed onOpenChange"
        );
    }
}
