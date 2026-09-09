//! Port of the type vocabulary of `packages/react/src/floating-ui-react/types.ts` (Base UI's
//! floating-ui-react unit), the shared shapes every hook and component of the unit hangs off
//! of (`specs/library/floating-ui-react/behavior.md`, "Public API surface (props, parts,
//! subcomponents)"; `specs/library/floating-ui-react/implementation.md`, "Store backbone").
//!
//! ## Rust adaptations
//!
//! - `ReferenceType = Element | VirtualElement` (`types.ts:154`) becomes
//!   [`ReferenceType`]: the element arm holds the real DOM node, the virtual arm holds the
//!   positioning shim as a shared trait object implementing `floating_ui_utils`'s
//!   `VirtualElement` (the external crate `floating-ui-dom` re-exports; the unit's TODO
//!   `wraps-external` mandates binding that crate instead of reimplementing the floating-ui
//!   packages — `specs/library/floating-ui-react/implementation.md`, "Dependencies on other
//!   Base UI internals").
//! - `ReferenceType`'s `VirtualElement` arm is `Rc`-shared rather than by-value: upstream
//!   virtual elements are JS objects with reference semantics, handed to the positioning
//!   engine and store repeatedly (the `positionReference` coalescing selector,
//!   `packages/react/src/floating-ui-react/components/FloatingRootStore.ts:37`).
//! - `Delay = number | Partial<{ open: number; close: number }>` (`types.ts:91`) becomes
//!   [`Delay`]. Delays are millisecond durations feeding `Timeout.start`
//!   (`packages/react/src/floating-ui-react/components/FloatingDelayGroup.tsx:73`,
//!   `hooks/useHover.ts:87-88`), whose port takes `u32`.
//! - `ContextData` (`types.ts:116-120`) — the free-form `dataRef.current` bag — becomes a
//!   struct with one field per key the unit itself writes (the greppable set:
//!   `openEvent`, `insideReactTree`, `orientation`, `__escapeKeyBubbles`,
//!   `__outsidePressBubbles`; `floatingContext` is added with the context hooks that
//!   introduce it). Upstream's `[key: string]: any` index signature is open-ended; unknown
//!   keys belong to consumers outside this unit and reappear here as those units port.
//! - `FloatingEvents` (`types.ts:110-114`) becomes [`FloatingEvents`], the shared event-bus
//!   handle. Upstream's payload is `any`; the port's root-store bus carries
//!   [`FloatingUIOpenChangeDetails`] (the only payload emitted on it —
//!   `components/FloatingRootStore.ts:117`), and the tree bus carries
//!   [`FloatingTreeEvent`] (the two events emitted on it:
//!   `'floating.closed'` with the closing mouse event —
//!   `hooks/useHoverFloatingInteraction.ts:171`, `hooks/useHoverReferenceInteraction.ts:201`
//!   — and `'virtualfocus'` with the element to focus —
//!   `hooks/useListNavigation.ts:317,562`).
//! - `FloatingUIOpenChangeDetails` (`packages/react/src/internals/types.ts`, the
//!   openchange payload built by
//!   `packages/react/src/floating-ui-react/components/FloatingRootStore.ts:109-115`) is
//!   defined here until the `infra: internals` unit ports `internals/types`; the field set
//!   is the constructor at that citation.
//! - `ExtendedRefs`/`ExtendedElements` (`types.ts:95-108`) hold React `RefObject`s; the
//!   port's ref shape is the fillable `Rc<Cell<Option<E>>>` slot
//!   (`crate::types::HTMLProps`'s ref member), so the ref fields use that and the setters
//!   return closures. The full structs land with the context hooks that construct them.
//! - `TransitionStatus` comes from `internals/useTransitionStatus`
//!   (`packages/react/src/internals/useTransitionStatus`), a unit not yet ported; the
//!   string union ports to [`TransitionStatus`] here and moves behind a re-export when that
//!   unit lands. `useFloatingRootContext` and `useSyncedFloatingRootContext` always seed it
//!   `undefined` (`hooks/useFloatingRootContext.ts:47`,
//!   `hooks/useSyncedFloatingRootContext.ts:67`), so only the type is needed now.
//! - `NarrowedElement<T>` (`types.ts:93`) is a TypeScript conditional-type erasure device
//!   (`T extends Element ? T : Element`); the port holds `Element` directly.
//! - `Prettify<T>` (`types.ts:87-89`) is a TypeScript display helper with no runtime; N/A.
//! - The floating-ui re-export block (`types.ts:28-85`) ports to `pub use` of the
//!   `floating-ui-dom` crate's equivalents — the delegation the TODO `wraps-external`
//!   mandates. `detectOverflow`/`getOverflowAncestors`/`platform` are re-exported from
//!   their defining crates (`floating_ui_core`/`floating_ui_utils::dom`/
//!   `floating_ui_dom::Platform`), matching upstream's re-export of them through
//!   `@floating-ui/react-dom`.

use std::cell::RefCell;
use std::rc::Rc;

use floating_ui_dom::{MiddlewareVec, Placement, Strategy, VirtualElement};
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use web_sys::{Element, Event, HtmlElement, MouseEvent};

use crate::create_base_ui_event_details::BaseUIChangeEventDetails;

/// Port of the `Delay` type (`types.ts:91`): a single millisecond duration, or separate
/// open/close durations with either side omitted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delay {
    /// `number` — one duration for open and close.
    Value(u32),
    /// `Partial<{ open: number; close: number }>` — per-direction durations.
    Partial {
        open: Option<u32>,
        close: Option<u32>,
    },
}

impl Delay {
    /// Resolves the open direction (`delay.open` in upstream consumers, e.g.
    /// `hooks/useHover.ts:91`), falling back to the single value.
    pub fn open(self) -> u32 {
        match self {
            Delay::Value(value) => value,
            Delay::Partial { open, .. } => open.unwrap_or(0),
        }
    }

    /// Resolves the close direction (`delay.close` in upstream consumers, e.g.
    /// `hooks/useHover.ts:91`), falling back to the single value.
    pub fn close(self) -> u32 {
        match self {
            Delay::Value(value) => value,
            Delay::Partial { close, .. } => close.unwrap_or(0),
        }
    }
}

/// Port of `TransitionStatus` from `packages/react/src/internals/useTransitionStatus` —
/// the mount/transition phase of the floating element
/// (`components/FloatingRootStore.ts:13`; `hooks/useFloatingRootContext.ts:47` seeds it
/// `undefined`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionStatus {
    /// `'initial'` — mounted, transition about to start.
    Initial,
    /// `'starting'` — enter transition running.
    Starting,
    /// `'ending'` — exit transition running.
    Ending,
}

/// A virtual reference — the `VirtualElement` arm of [`ReferenceType`] with an identity
/// token. Upstream virtual elements are JS objects compared by `Object.is` identity; the
/// Rust `Box<dyn VirtualElement>` is `Clone` (cloning creates a NEW identity, like JS
/// object spread), so the token carries the stable identity across clones — fresh shims
/// get fresh tokens, mirroring upstream's fresh shim objects per `setPositionReference`
/// call (`hooks/useFloating.ts:95-101`).
pub struct VirtualReference {
    /// The positioning shim.
    pub element: Box<dyn VirtualElement<Element>>,
    /// The identity token — see the struct docs.
    pub id: u64,
}

// Handwritten rather than derived: the derived impls would bound `Element`-related
// generics; `Box<dyn VirtualElement<Element>>: Clone` holds (the dyn-trait machinery
// provides it) and the token is `Copy`.
impl Clone for VirtualReference {
    fn clone(&self) -> Self {
        Self {
            element: self.element.clone(),
            id: self.id,
        }
    }
}

thread_local! {
    static NEXT_VIRTUAL_REFERENCE_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

impl VirtualReference {
    /// Wraps a virtual-element implementation with a fresh identity token.
    pub fn new(element: impl VirtualElement<Element> + 'static) -> Self {
        Self {
            element: Box::new(element),
            id: NEXT_VIRTUAL_REFERENCE_ID.with(|next| {
                let id = next.get();
                next.set(id + 1);
                id
            }),
        }
    }
}

/// Port of `ReferenceType = Element | VirtualElement` (`types.ts:154`): the positioning
/// reference is either a real DOM element or a virtual element (a rect-shaped object,
/// `hooks/useFloating.ts:95-101`).
#[derive(Clone)]
pub enum ReferenceType {
    /// A real DOM element.
    Element(Element),
    /// A virtual element — `getBoundingClientRect`/`getClientRects` shim (upstream's
    /// `VirtualElement` from `@floating-ui/react-dom`, bound to the external crate's
    /// trait).
    Virtual(VirtualReference),
}

impl ReferenceType {
    /// Upstream `isElement(reference)` (`hooks/useFloating.ts:95,112,120`): whether the
    /// reference is a real DOM element.
    pub fn is_element(&self) -> bool {
        matches!(self, ReferenceType::Element(_))
    }

    /// The real element, if this is one — upstream `isElement(x) ? x : null`.
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            ReferenceType::Element(element) => Some(element),
            ReferenceType::Virtual(_) => None,
        }
    }

    /// The virtual element's `contextElement`, if this is one — upstream
    /// `VirtualElement.contextElement` (`@floating-ui/react-dom`'s shape; used by
    /// `isEventTargetWithin`'s overflow checks and `useHover`'s pointer-target logic).
    pub fn context_element(&self) -> Option<Element> {
        match self {
            ReferenceType::Element(element) => Some(element.clone()),
            ReferenceType::Virtual(virtual_reference) => {
                virtual_reference.element.context_element()
            }
        }
    }
}

impl std::fmt::Debug for ReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceType::Element(_) => f.write_str("ReferenceType::Element(..)"),
            ReferenceType::Virtual(_) => f.write_str("ReferenceType::Virtual(..)"),
        }
    }
}

/// Upstream compares references with `Object.is` (store updates,
/// `crates/leptos-ui-utils/src/store.rs` module docs): DOM elements by JS identity
/// (`strict_equals`), virtual elements by object identity — the port's token (see
/// [`VirtualReference`]).
impl PartialEq for ReferenceType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ReferenceType::Element(a), ReferenceType::Element(b)) => a == b,
            (ReferenceType::Virtual(a), ReferenceType::Virtual(b)) => a.id == b.id,
            _ => false,
        }
    }
}

impl From<Element> for ReferenceType {
    fn from(element: Element) -> Self {
        ReferenceType::Element(element)
    }
}

impl From<HtmlElement> for ReferenceType {
    fn from(element: HtmlElement) -> Self {
        ReferenceType::Element(element.into())
    }
}

/// Port of `ContextData` (`types.ts:116-120`), the mutable `dataRef.current` bag shared
/// through the store context — per-key struct fields instead of an index signature (see
/// the module docs). `Default` matches upstream's `{}` seed
/// (`components/FloatingRootStore.ts:77`).
pub struct ContextData {
    /// The event that opened the popup, mirrored by `syncOpenEvent`
    /// (`components/FloatingRootStore.ts:91-101`) — upstream `openEvent`.
    pub open_event: Option<Event>,
    /// The floating context backbone, attached by `useFloating`
    /// (`hooks/useFloating.ts:183`) — upstream `floatingContext`. The port stores the
    /// root-store handle, which is what every `floatingContext` read derives from (the
    /// context hooks construct `FloatingContext` views around this handle).
    pub floating_context: Option<Rc<crate::floating_ui::floating_root_store::FloatingRootStore>>,
    /// The node id of the stashed floating context — upstream's
    /// `dataRef.current.floatingContext?.nodeId` read (`hooks/useDismiss.ts:174,370`).
    /// Upstream reaches it through the whole context object; the port stashes the id
    /// alongside the store handle (stamped by the same `useFloating` effect) so the
    /// tree lookups can run without holding the context type (which would cycle back
    /// into this bag).
    pub floating_node_id: Option<String>,
    /// Whether the pointer's press lifecycle is inside the floating tree, tracked by
    /// `useDismiss`'s capture-phase markers (`hooks/useDismiss.ts:169,236,392`) —
    /// upstream `insideReactTree`, a plain boolean (three write/read sites, all
    /// boolean; see `hooks/useDismiss.ts` and `components/FloatingFocusManager.tsx`).
    pub inside_react_tree: bool,
    /// The list-navigation orientation shared with nested popups — upstream
    /// `orientation` (`hooks/useListNavigation.ts:523-524` reads the parent's).
    pub orientation: Option<Orientation>,
    /// The `bubbles.escapeKey` value `useDismiss` stamps per open session for the
    /// floating-tree cascade (`hooks/useDismiss.ts:178`) — upstream `__escapeKeyBubbles`.
    pub escape_key_bubbles: Option<bool>,
    /// The `bubbles.outsidePress` equivalent — upstream `__outsidePressBubbles`.
    pub outside_press_bubbles: Option<bool>,
}

// Handwritten rather than derived: `Rc<FloatingRootStore>` has no `Default` (the store
// needs its option bag), and an empty bag is store-independent.
impl Default for ContextData {
    fn default() -> Self {
        Self {
            open_event: None,
            floating_context: None,
            floating_node_id: None,
            inside_react_tree: false,
            orientation: None,
            escape_key_bubbles: None,
            outside_press_bubbles: None,
        }
    }
}

/// Port of `UseListNavigationProps['orientation']`
/// (`hooks/useListNavigation.ts:59-62`) — the axis list navigation moves along, shared
/// through `dataRef.current.orientation` for nested popups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
    Both,
}

/// Port of `FloatingUIOpenChangeDetails`
/// (`packages/react/src/internals/types.ts`, constructed at
/// `components/FloatingRootStore.ts:109-115`): the payload emitted on the root store's
/// `'openchange'` bus — the cross-hook coordination event
/// (`specs/library/floating-ui-react/behavior.md`, "Events": every interaction hook
/// funnels state changes through it).
#[derive(Clone)]
pub struct FloatingUIOpenChangeDetails {
    /// The new open state (`:110`).
    pub open: bool,
    /// The `REASONS` key for the change (`:111`).
    pub reason: String,
    /// The native event that triggered the change (`:112`).
    pub native_event: Event,
    /// Whether the popup is nested inside another floating tree node (`:113`).
    pub nested: bool,
    /// The element that triggered the change, when there is one (`:114`).
    pub trigger_element: Option<Element>,
}

/// Port of `FloatingEvents` (`types.ts:110-114`) specialized to the root store's bus
/// payload (see the module docs): `emit`/`on`/`off` over named events. Behind an `Rc`
/// because the bus is shared by identity — the store context and every
/// `FloatingContext` handed out must see the SAME bus (`hooks/useFloating.ts:166`).
pub type FloatingEvents = Rc<EventEmitter<FloatingUIOpenChangeDetails>>;

/// The payload enum for the tree bus (`components/FloatingTreeStore.ts:11` —
/// `createEventEmitter()` shared by all tree members): the two events the unit emits on
/// it (see the module docs for citations).
#[derive(Clone)]
pub enum FloatingTreeEvent {
    /// `'floating.closed'` — a member closed; carries the closing mouse event
    /// (`hooks/useHoverFloatingInteraction.ts:171`).
    FloatingClosed(MouseEvent),
    /// `'virtualfocus'` — virtual focus moved to an item; carries the element
    /// (`hooks/useListNavigation.ts:317,562`).
    VirtualFocus(Element),
}

/// The tree bus type (`components/FloatingTreeStore.ts:11`), shared by `Rc` like the
/// root bus.
pub type FloatingTreeEvents = Rc<EventEmitter<FloatingTreeEvent>>;

/// A listener registered on an [`EventEmitter`].
pub type EventListener<P> = Rc<dyn Fn(&P)>;

/// The unsubscribe closure `EventEmitter::on` returns — the port's stand-in for upstream's
/// `off(event, listener)` call, which removes the identical function from the `Set`
/// (`createEventEmitter.ts:15-17`). The port's hooks stash this handle where upstream
/// stashes the listener function and pass it to `off`.
pub type EventUnsubscribe = Rc<dyn Fn()>;

struct ListenerEntry<P: 'static> {
    listener: EventListener<P>,
    removed: Rc<std::cell::Cell<bool>>,
}

// Handwritten rather than derived: the derive would bound `P: Clone`, but the payload
// sits behind an `Rc<dyn Fn(&P)>` — cloning shares the handle and clones nothing of `P`.
impl<P: 'static> Clone for ListenerEntry<P> {
    fn clone(&self) -> Self {
        Self {
            listener: Rc::clone(&self.listener),
            removed: Rc::clone(&self.removed),
        }
    }
}

/// Port of `createEventEmitter`'s return type
/// (`packages/react/src/floating-ui-react/utils/createEventEmitter.ts:3-18`): a
/// name-keyed bus. Upstream keys listeners in a `Set`, deduplicating the identical
/// function; `Rc<dyn Fn>` has no usable equality in Rust, so listeners are appended and
/// each unsubscribe handle removes only its own registration — the same adaptation the
/// `Store` port documents for its listener set (`crates/leptos-ui-utils/src/store.rs`).
/// Unsubscribing remains idempotent: a second call is a no-op, matching upstream's
/// `Set.delete` semantics (`createEventEmitter.ts:15-17`).
pub struct EventEmitter<P: 'static> {
    listeners: std::cell::RefCell<std::collections::HashMap<String, Vec<ListenerEntry<P>>>>,
}

// Handwritten rather than derived: the bus starts empty for every payload type
// (`createEventEmitter.ts:4`), so `P` needs no `Default`.
impl<P: 'static> Default for EventEmitter<P> {
    fn default() -> Self {
        Self {
            listeners: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
}

impl<P: 'static> EventEmitter<P> {
    /// `createEventEmitter()` (`createEventEmitter.ts:3`).
    pub fn new() -> Self {
        Self::default()
    }

    /// `on(event, listener)` (`createEventEmitter.ts:9-14`), returning the unsubscribe
    /// closure that upstream expresses as `off(event, listener)`.
    pub fn on(&self, event: &str, listener: EventListener<P>) -> EventUnsubscribe {
        let removed = Rc::new(std::cell::Cell::new(false));
        self.listeners
            .borrow_mut()
            .entry(event.to_owned())
            .or_default()
            .push(ListenerEntry {
                listener: Rc::clone(&listener),
                removed: Rc::clone(&removed),
            });
        let listeners = self.listeners.clone();
        let event = event.to_owned();
        Rc::new(move || {
            if removed.get() {
                return;
            }
            removed.set(true);
            let mut listeners = listeners.borrow_mut();
            if let Some(slot) = listeners.get_mut(&event) {
                slot.retain(|entry| !entry.removed.get());
            }
        })
    }

    /// `emit(event, data)` (`createEventEmitter.ts:6-8`): every live listener for the
    /// event receives the payload. Upstream iterates the `Set` live; the port snapshots
    /// the listener list before dispatching (a `RefCell` cannot be re-borrowed while a
    /// listener it is iterating subscribes or unsubscribes — the same re-entrancy
    /// adaptation the `Store` port documents). An unsubscribed listener receives no
    /// later emission, even one dispatched from inside a listener it ran alongside.
    pub fn emit(&self, event: &str, data: &P) {
        let snapshot: Vec<_> = self
            .listeners
            .borrow()
            .get(event)
            .cloned()
            .unwrap_or_default();
        for entry in snapshot {
            if !entry.removed.get() {
                (entry.listener)(data);
            }
        }
    }
}

/// Port of `FloatingNodeType` (`types.ts:139-143`): one registered floating element in a
/// [`FloatingTreeStore`]. `context` is added by the context hooks
/// (`hooks/useFloating.ts:182-189` patches `node.context = context`); until those land
/// the field stays `None`-able with the store handle it will carry.
#[derive(Clone)]
pub struct FloatingNodeType {
    /// The node id — the `floatingId` of the registering popup
    /// (`components/FloatingTree.tsx:42`).
    pub id: Option<String>,
    /// The parent node id, `None` for top-level floats (`:42`).
    pub parent_id: Option<String>,
    /// The node's context, patched in once the popup's own context exists
    /// (`hooks/useFloating.ts:185-188`). Upstream stores the whole `FloatingContext`;
    /// the port stores the root-store handle, which is what every consumer of
    /// `node.context` reads through (`components`/`hooks` reach
    /// `node.context.elements`, `node.context.open`, `node.context.dataRef` — all
    /// derived from the root store's current state; the context hooks construct
    /// `FloatingContext` views around this handle). Behind a `RefCell` so the patch is
    /// possible through the shared `Rc` node handles the tree stores.
    pub context:
        std::cell::RefCell<Option<Rc<crate::floating_ui::floating_root_store::FloatingRootStore>>>,
}

/// Port of `FloatingTreeType = FloatingTreeStore` (`types.ts:145`).
pub type FloatingTreeType = Rc<crate::floating_ui::tree::FloatingTreeStore>;

/// Port of the positioning vocabulary upstream re-exports through
/// `types.ts:28-85` from `@floating-ui/react-dom` — bound to the external
/// `floating-ui-dom` crate per the unit's `wraps-external` TODO field
/// (`specs/library/floating-ui-react/implementation.md`, "Dependencies on other Base UI
/// internals").
pub use floating_ui_dom::{
    AlignedPlacement, Alignment, AutoUpdateOptions, Axis, Boundary, Coords, DetectOverflowOptions,
    Dimensions, ElementContext, ElementOrVirtual, ElementRects, Middleware, MiddlewareData,
    MiddlewareState, Padding, RootBoundary, Side, SideObject, auto_update, compute_position, dom,
};

/// The placement/strategy vocabulary (`types.ts:58,67`).
pub use floating_ui_dom::{ComputePositionConfig, ComputePositionReturn};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use std::cell::Cell;
    use std::cell::RefCell;

    // Pins the emitter contract (`createEventEmitter.ts:3-18`; behavior.md, "Shared open/close
    // contract" — the bus is the coordination backbone every hook subscribes through).
    #[test]
    fn emit_reaches_every_listener_for_the_event_only() {
        let bus: EventEmitter<u8> = EventEmitter::new();
        let a: EventListener<u8> = Rc::new(|payload| assert_eq!(*payload, 7));
        let b: EventListener<u8> = Rc::new(|payload| assert_eq!(*payload, 7));
        bus.on("openchange", Rc::clone(&a));
        bus.on("other", Rc::clone(&b));
        bus.emit("openchange", &7);
    }

    #[test]
    fn unsubscribe_stops_later_emissions_and_is_idempotent() {
        let bus: EventEmitter<u8> = EventEmitter::new();
        let calls: Rc<Cell<Vec<u8>>> = Rc::new(Cell::new(Vec::new()));
        let calls_for_listener = Rc::clone(&calls);
        let listener: EventListener<u8> = Rc::new(move |payload| {
            let mut log = calls_for_listener.take();
            log.push(*payload);
            calls_for_listener.set(log);
        });
        let unsubscribe = bus.on("openchange", listener);

        bus.emit("openchange", &1);
        unsubscribe();
        unsubscribe(); // second call is a no-op (upstream `Set.delete`)
        bus.emit("openchange", &2);

        assert_eq!(
            calls.take(),
            vec![1],
            "only the pre-unsubscribe emission lands"
        );
    }

    #[test]
    fn emit_snapshots_so_unsubscribing_mid_pass_takes_effect_immediately() {
        // Upstream iterates the live `Set` (`createEventEmitter.ts:7`); the port snapshots
        // the listener list (RefCell re-entrancy) but shares the per-listener removal flag
        // with the snapshot, so the observable outcome matches upstream's live iteration: a
        // listener unsubscribed during the pass is skipped for the remainder of that pass
        // (JS `Set` iteration skips entries deleted before they are reached) and for every
        // later pass.
        let bus: EventEmitter<u8> = EventEmitter::new();
        let first_calls: Rc<Cell<u8>> = Rc::new(Cell::new(0));
        let second_calls: Rc<Cell<u8>> = Rc::new(Cell::new(0));
        let unsubscribe_second: Rc<RefCell<Option<EventUnsubscribe>>> = Rc::new(RefCell::new(None));

        let first_calls_handle = Rc::clone(&first_calls);
        let unsubscribe_handle = Rc::clone(&unsubscribe_second);
        let first: EventListener<u8> = Rc::new(move |payload| {
            first_calls_handle.set(*payload);
            if let Some(unsubscribe) = unsubscribe_handle.borrow().as_ref() {
                unsubscribe();
            }
        });
        bus.on("openchange", first);

        let second_calls_handle = Rc::clone(&second_calls);
        let second: EventListener<u8> = Rc::new(move |payload| {
            second_calls_handle.set(second_calls_handle.get() + *payload);
        });
        *unsubscribe_second.borrow_mut() = Some(bus.on("openchange", second));

        bus.emit("openchange", &5);
        assert_eq!(first_calls.get(), 5, "the first listener received the pass");
        assert_eq!(
            second_calls.get(),
            0,
            "unsubscribing during the pass removes the listener from the same pass — \
             upstream's live `Set` iteration skips entries deleted before they are \
             reached (`createEventEmitter.ts:7` + `:15-17`)"
        );

        bus.emit("openchange", &5);
        assert_eq!(
            second_calls.get(),
            0,
            "the listener unsubscribed mid-pass receives nothing further"
        );
    }

    // Pins the Delay resolution (`types.ts:91`): the single value serves both directions;
    // the partial form falls back to 0 per direction (`delay.open`/`delay.close` consumers,
    // e.g. `hooks/useHover.ts:87-91`).
    #[test]
    fn delay_resolves_single_and_partial_forms() {
        assert_eq!(Delay::Value(300).open(), 300);
        assert_eq!(Delay::Value(300).close(), 300);
        assert_eq!(
            Delay::Partial {
                open: Some(100),
                close: None
            }
            .open(),
            100
        );
        assert_eq!(
            Delay::Partial {
                open: Some(100),
                close: None
            }
            .close(),
            0,
            "an omitted close duration falls back to 0"
        );
        assert_eq!(
            Delay::Partial {
                open: None,
                close: Some(50)
            }
            .open(),
            0
        );
    }
}

/// `Platform` (`types.ts:59`) — the DOM platform implementation the positioning engine
/// binds to (`floating_ui_dom::Platform`).
pub use floating_ui_dom::Platform;

/// Port of `FloatingContext` (`types.ts:124-137`), deferred to the context-hooks
/// checkpoint: the type is `UsePositionFloatingReturn` extended with the store-owned
/// coordination members (`onOpenChange` is literally `store.setOpen`,
/// `hooks/useFloating.ts:165`), so it is constructed with the positioning engine it
/// wraps. The store, events, and dataRef it references are already ported here; the
/// struct appears with `use_floating`/`use_floating_root_context`.
///
/// Port of `UseFloatingOptions`/`UseFloatingReturn` (`types.ts:172-201,158-170`):
/// likewise constructed with the positioning engine (they embed its return shape);
/// deferred to the same checkpoint.
///
/// Port of `ElementProps` (`types.ts:147-152`) — the per-element prop bags interaction
/// hooks return: the port's prop surface is Leptos's native attribute system
/// (`specs/architecture.md`, "Prop / class / style merging (mergeProps)"), so the bags
/// become plain vectors of `(name, value)` attribute pairs plus event-handler maps,
/// defined with the hooks that produce them.
///
/// Port of the positioning placement enum upstream re-exports as `Placement`
/// (`types.ts:58`): [`floating_ui_dom::Placement`].

/// The `BaseUIChangeEventDetails<string>` shape the unit's `setOpen` contract uses
/// (`types.ts:129`, `components/FloatingRootStore.ts:25-26`): reason strings keyed by the
/// `REASONS` registry, the native event, and no custom payload at the floating layer.
pub type RootOpenChangeEventDetails = BaseUIChangeEventDetails<String>;

/// Port of `UseFloatingOptions['onOpenChange']` (`types.ts:193`) /
/// `FloatingRootStoreContext.onOpenChange`
/// (`components/FloatingRootStore.ts:25-26`): the consumer's open-state callback.
pub type OnOpenChangeFn = Rc<dyn Fn(bool, &RootOpenChangeEventDetails)>;

/// A placement the positioning engine resolves to — re-exported for hook option types
/// (`types.ts:58`).
pub type PositioningPlacement = Placement;

/// The positioning strategy (`types.ts:67`).
pub type PositioningStrategy = Strategy;

// ---------------------------------------------------------------------------
// Positioning-engine and context surface (`types.ts:87-201`), constructed with
// the context hooks — see `use_floating`/`use_floating_root_context`.
// ---------------------------------------------------------------------------

/// The engine's middleware vector — upstream `@floating-ui/react-dom`'s
/// `middleware` option (`ComputePositionConfig.middleware`). Behind a
/// `SendWrapper` because reactive-graph signal values must be `Send + Sync` while
/// boxed middleware is not; the same bridge `floating-ui-leptos` uses for its
/// `WrappedMiddleware`.
pub type WrappedMiddleware = SendWrapper<MiddlewareVec>;

/// `whileElementsMounted` (`node_modules/@floating-ui/react-dom`, `UseFloatingOptions`):
/// invoked when both elements are mounted and cleaned up when either unmounts — the
/// `autoUpdate` wiring point (`specs/library/floating-ui-react/implementation.md`,
/// "Dependencies on other Base UI internals").
pub type WhileElementsMountedCleanupFn = Rc<dyn Fn()>;
pub type WhileElementsMountedFn =
    dyn Fn(&ReferenceType, &Element, Rc<dyn Fn()>) -> WhileElementsMountedCleanupFn;

/// Port of `floatingStyles` (`@floating-ui/react-dom`'s `UseFloatingReturn`): the
/// pre-configured positioning styles for the floating element.
#[derive(Clone, Debug, PartialEq)]
pub struct PositioningStyles {
    pub position: Strategy,
    pub top: String,
    pub left: String,
    pub transform: Option<String>,
    pub will_change: Option<String>,
}

/// Port of `ExtendedRefs` (`types.ts:95-102`): the reference/floating ref objects and
/// the reactive setters Base UI layers over the positioning engine's refs
/// (`hooks/useFloating.ts:141-150`). The ref objects are the fillable
/// `Rc<RefCell<Option<_>>>` slots (the `{ current: T | null }` shape); the setters are
/// methods on the constructed context (they close over hook state upstream — see
/// `use_floating`).
#[derive(Clone, Default)]
pub struct ExtendedRefs {
    /// `refs.reference` (`:96`) — the positioning reference, element or virtual.
    pub reference: Rc<RefCell<Option<ReferenceType>>>,
    /// `refs.floating` (`:97`).
    pub floating: Rc<RefCell<Option<Element>>>,
    /// `refs.domReference` (`:98`) — the Base UI extension: the real DOM reference
    /// (`NarrowedElement`), `None` when the reference is virtual.
    pub dom_reference: Rc<RefCell<Option<Element>>>,
}

/// Port of `ExtendedElements` (`types.ts:104-108`): the elements the context exposes.
/// Upstream re-creates the object per render from the store state (`hooks/useFloating.
/// ts:152-158`); the port hands out store-backed signals so readers are always fresh.
#[derive(Clone)]
pub struct ExtendedElements {
    /// `elements.reference` (`:105`) — the positioning reference: upstream's
    /// `position.elements.reference`, which the store's coalescing selector mirrors
    /// (`positionReference ?? referenceElement`,
    /// `components/FloatingRootStore.ts:37`).
    pub reference: Signal<Option<ReferenceType>, LocalStorage>,
    /// `elements.floating` (`:106`).
    pub floating: Signal<Option<Element>, LocalStorage>,
    /// `elements.domReference` (`:107`).
    pub dom_reference: Signal<Option<Element>, LocalStorage>,
}

/// Port of `FloatingContext` (`types.ts:124-137`): the positioning data plus the
/// store-owned coordination members. Constructed by `use_floating`
/// (`hooks/useFloating.ts:160-174`) — `onOpenChange` is literally `store.setOpen`.
pub struct FloatingContext {
    // The `usePosition` data spread (`hooks/useFloating.ts:162` — `...position`).
    pub x: RwSignal<f64>,
    pub y: RwSignal<f64>,
    pub placement: RwSignal<Placement>,
    pub strategy: RwSignal<Strategy>,
    pub middleware_data: RwSignal<MiddlewareData>,
    pub is_positioned: RwSignal<bool>,
    pub update: Rc<dyn Fn()>,
    pub floating_styles: Signal<PositioningStyles, LocalStorage>,
    // The Base UI extensions (`:163-171`).
    pub open: Signal<bool, LocalStorage>,
    pub on_open_change: Rc<dyn Fn(bool, &RootOpenChangeEventDetails)>,
    pub events: FloatingEvents,
    pub data_ref: Rc<RefCell<ContextData>>,
    pub node_id: Option<String>,
    pub floating_id: Option<String>,
    pub refs: ExtendedRefs,
    pub elements: ExtendedElements,
    /// `refs.setReference` (`types.ts:99`, overridden `hooks/useFloating.ts:110-131`).
    pub set_reference: Rc<dyn Fn(Option<ReferenceType>)>,
    /// `refs.setFloating` (`types.ts:100`, overridden `:133-139`).
    pub set_floating: Rc<dyn Fn(Option<Element>)>,
    /// `refs.setPositionReference` (`types.ts:101`, defined `:93-108`).
    pub set_position_reference: Rc<dyn Fn(Option<ReferenceType>)>,
    pub root_store: Rc<crate::floating_ui::floating_root_store::FloatingRootStore>,
}

/// Port of `UseFloatingReturn` (`types.ts:158-170`): the positioning data plus the
/// context, refs, elements, and root store (`hooks/useFloating.ts:191-200`).
pub struct UseFloatingReturn {
    pub x: RwSignal<f64>,
    pub y: RwSignal<f64>,
    pub placement: RwSignal<Placement>,
    pub strategy: RwSignal<Strategy>,
    pub middleware_data: RwSignal<MiddlewareData>,
    pub is_positioned: RwSignal<bool>,
    pub update: Rc<dyn Fn()>,
    pub floating_styles: Signal<PositioningStyles, LocalStorage>,
    pub context: Rc<FloatingContext>,
    pub refs: ExtendedRefs,
    pub elements: ExtendedElements,
    pub root_store: Rc<crate::floating_ui::floating_root_store::FloatingRootStore>,
}
