//! Port of `packages/react/src/floating-ui-react/components/FloatingRootStore.ts` — the
//! store backbone the whole floating-ui unit hangs off of
//! (`specs/library/floating-ui-react/implementation.md`, "Store backbone": "a single
//! `FloatingRootStore` ... handed around by reference, interaction hooks subscribe to it
//! with `useState` selectors, and cross-hook coordination happens through (a) store
//! selectors, (b) an `openchange` event bus, and (c) a mutable `dataRef`").
//!
//! ## Rust adaptations
//!
//! - Upstream extends `ReactStore<State, Context, selectors>` with a named selector
//!   registry (`FloatingRootStore.ts:33-40`) used by string-keyed `useState('open')`
//!   calls. The port follows the `ReactStore` port's convention: selector functions are
//!   passed at the call site and the registry becomes the free selector functions
//!   [`selectors`] (see `crates/leptos-ui-utils/src/react_store.rs` module docs).
//!   [`selectors::reference_element`] ports the coalescing selector
//!   `positionReference ?? referenceElement` (`FloatingRootStore.ts:37`) — the
//!   virtual-element positioning mechanism staying invisible to consumers.
//! - The context's non-reactive members port with interior mutability where upstream
//!   mutates them in place: `onOpenChange` and `nested` are re-assigned every render by
//!   the context hooks (`hooks/useFloatingRootContext.ts:76-77`,
//!   `hooks/useSyncedFloatingRootContext.ts:110-111`), so they live in a `RefCell`/`Cell`
//!   behind setter methods; `dataRef` is the shared `Rc<RefCell<ContextData>>` created
//!   once in the constructor (`FloatingRootStore.ts:77`); `triggerElements` is created by
//!   the caller and never swapped (`:80`).
//! - The arrow-function class properties `syncOpenEvent`/`dispatchOpenChange`/`setOpen`
//!   (`FloatingRootStore.ts:91-135`) port to methods on the shared store handle — the
//!   stable-identity concern they solve (passing bound methods into effects) is
//!   React-specific (see the `ReactStore` port's render-phase notes).
//! - `setOpen`'s `syncOnly` mode (`:126-130`) ports verbatim: the popup-store-owned
//!   path forwards to `onOpenChange` without dispatching; the standalone path dispatches
//!   first (`:132` — events before the callback), then calls back.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use web_sys::{Element, Event};

use leptos_ui_utils::react_store::ReactStore;

use crate::floating_ui::event::is_click_like_event;
use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
use crate::floating_ui::types::{
    ContextData, FloatingEvents, FloatingUIOpenChangeDetails, OnOpenChangeFn, ReferenceType,
    RootOpenChangeEventDetails, TransitionStatus, VirtualReference,
};

/// Port of `FloatingRootState` (`FloatingRootStore.ts:11-22`).
#[derive(Clone, Debug)]
pub struct FloatingRootState {
    /// (`:12`)
    pub open: bool,
    /// (`:13`)
    pub transition_status: Option<TransitionStatus>,
    /// The real DOM reference element, `None` when the reference is virtual (`:14`).
    pub dom_reference_element: Option<Element>,
    /// (`:15`)
    pub reference_element: Option<ReferenceType>,
    /// (`:16`)
    pub floating_element: Option<Element>,
    /// The virtual (or real) element positioning is anchored to (`:17`).
    pub position_reference: Option<ReferenceType>,
    /// The ID of the floating element (`:18-21`).
    pub floating_id: Option<String>,
}

/// Port of `FloatingRootStoreContext` (`FloatingRootStore.ts:24-31`).
pub struct FloatingRootStoreContext {
    /// (`:25-26`) — reassigned by the context hooks on every render
    /// (`hooks/useFloatingRootContext.ts:76`).
    on_open_change: RefCell<Option<OnOpenChangeFn>>,
    /// (`:27`) — created once (`:77`), shared by reference.
    pub data_ref: Rc<RefCell<ContextData>>,
    /// (`:28`) — the `openchange` coordination bus.
    pub events: FloatingEvents,
    /// (`:29`) — reassigned by the context hooks (`hooks/useFloatingRootContext.ts:77`).
    nested: Cell<bool>,
    /// (`:30`) — the multi-trigger registry.
    pub trigger_elements: PopupTriggerMap,
}

impl FloatingRootStoreContext {
    /// Reads `onOpenChange` (`FloatingRootStore.ts:25-26`).
    pub fn on_open_change(&self) -> Option<OnOpenChangeFn> {
        self.on_open_change.borrow().clone()
    }

    /// Re-assigns `onOpenChange` — upstream's render-phase patch
    /// (`hooks/useFloatingRootContext.ts:76`,
    /// `hooks/useSyncedFloatingRootContext.ts:110`).
    pub fn set_on_open_change(&self, on_open_change: Option<OnOpenChangeFn>) {
        *self.on_open_change.borrow_mut() = on_open_change;
    }

    /// Reads `nested` (`FloatingRootStore.ts:29`).
    pub fn nested(&self) -> bool {
        self.nested.get()
    }

    /// Re-assigns `nested` — upstream's render-phase patch
    /// (`hooks/useFloatingRootContext.ts:77`,
    /// `hooks/useSyncedFloatingRootContext.ts:111`).
    pub fn set_nested(&self, nested: bool) {
        self.nested.set(nested);
    }
}

/// The named selectors (`FloatingRootStore.ts:33-40`) as free functions — the port's
/// call-site equivalent of the string-keyed registry (see the module docs).
pub mod selectors {
    use super::*;

    /// `open` (`FloatingRootStore.ts:34`).
    pub fn open(state: &FloatingRootState) -> bool {
        state.open
    }

    /// `transitionStatus` (`FloatingRootStore.ts:35`).
    pub fn transition_status(state: &FloatingRootState) -> Option<TransitionStatus> {
        state.transition_status
    }

    /// `domReferenceElement` (`FloatingRootStore.ts:36`).
    pub fn dom_reference_element(state: &FloatingRootState) -> Option<Element> {
        state.dom_reference_element.clone()
    }

    /// `referenceElement` = `positionReference ?? referenceElement`
    /// (`FloatingRootStore.ts:37`) — the coalescing selector virtual-element positioning
    /// rides on (`specs/library/floating-ui-react/implementation.md`, "Store backbone").
    pub fn reference_element(state: &FloatingRootState) -> Option<ReferenceType> {
        state
            .position_reference
            .clone()
            .or_else(|| state.reference_element.clone())
    }

    /// `floatingElement` (`FloatingRootStore.ts:38`).
    pub fn floating_element(state: &FloatingRootState) -> Option<Element> {
        state.floating_element.clone()
    }

    /// `floatingId` (`FloatingRootStore.ts:39`).
    pub fn floating_id(state: &FloatingRootState) -> Option<String> {
        state.floating_id.clone()
    }
}

/// Port of `FloatingRootStoreOptions` (`FloatingRootStore.ts:42-57`).
pub struct FloatingRootStoreOptions {
    pub open: bool,
    pub transition_status: Option<TransitionStatus>,
    pub reference_element: Option<ReferenceType>,
    pub floating_element: Option<Element>,
    pub trigger_elements: PopupTriggerMap,
    pub floating_id: Option<String>,
    /// When true, `setOpen` only forwards to `onOpenChange`; the popup store owns
    /// `dispatchOpenChange` in this mode (`:49-53`).
    pub sync_only: bool,
    pub nested: bool,
    pub on_open_change: Option<OnOpenChangeFn>,
}

/// Port of `FloatingRootStore` (`FloatingRootStore.ts:59-136`): the `ReactStore` over
/// [`FloatingRootState`] with the context above, plus the open/close side-effect
/// methods. Created inside a reactive owner and held as `Rc` (see the `ReactStore`
/// port's docs). The inner store rides in an `Rc` so the render-phase hooks
/// (`use_state`/`use_synced_value`) can capture the shared handle from any borrowed
/// reference via [`FloatingRootStore::rc`].
pub struct FloatingRootStore {
    inner: Rc<ReactStore<FloatingRootState, FloatingRootStoreContext>>,
    sync_only: bool,
}

impl std::ops::Deref for FloatingRootStore {
    type Target = ReactStore<FloatingRootState, FloatingRootStoreContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FloatingRootStore {
    /// The shared `ReactStore` handle the render-phase hooks take (`&Rc<Self>`
    /// receivers) — the upstream class's `this` identity.
    pub fn rc(&self) -> Rc<ReactStore<FloatingRootState, FloatingRootStoreContext>> {
        Rc::clone(&self.inner)
    }

    /// Port of the constructor (`FloatingRootStore.ts:66-86`): seeds the state with
    /// `positionReference = referenceElement` and
    /// `domReferenceElement = referenceElement as Element | null` (`:72-73`), and
    /// creates the non-reactive context members once (`:75-81`).
    pub fn new(options: FloatingRootStoreOptions) -> Rc<Self> {
        let FloatingRootStoreOptions {
            open,
            transition_status,
            reference_element,
            floating_element,
            trigger_elements,
            floating_id,
            sync_only,
            nested,
            on_open_change,
        } = options;

        let dom_reference_element = reference_element
            .as_ref()
            .and_then(ReferenceType::as_element)
            .cloned();

        let state = FloatingRootState {
            open,
            transition_status,
            dom_reference_element,
            reference_element: reference_element.clone(),
            floating_element,
            position_reference: reference_element,
            floating_id,
        };

        let context = FloatingRootStoreContext {
            on_open_change: RefCell::new(on_open_change),
            data_ref: Rc::new(RefCell::new(ContextData::default())),
            events: Rc::new(crate::floating_ui::types::EventEmitter::new()),
            nested: Cell::new(nested),
            trigger_elements,
        };

        Rc::new(Self {
            inner: Rc::new(ReactStore::with_context(state, context)),
            sync_only,
        })
    }

    /// `syncOpenEvent` (`FloatingRootStore.ts:91-101`): syncs the event
    /// `dataRef.current.openEvent` mirrors for hover/click disambiguation. A pending
    /// hover-open never overwrites a click-like open event, while a click event may
    /// upgrade a hover-open (`:95-97`).
    pub fn sync_open_event(&self, new_open: bool, event: Option<&Event>) {
        let currently_open = self.inner.get_snapshot().open;
        let is_click_like = event.map(is_click_like_event).unwrap_or(false);
        if !new_open || !currently_open || is_click_like {
            self.inner.context.data_ref.borrow_mut().open_event =
                if new_open { event.cloned() } else { None };
        }
    }

    /// `dispatchOpenChange` (`FloatingRootStore.ts:106-118`): the root-owned side
    /// effects for an open-state change — sync the open event, then emit `'openchange'`
    /// with the details payload (`:109-115`).
    pub fn dispatch_open_change(&self, new_open: bool, event_details: &RootOpenChangeEventDetails) {
        self.sync_open_event(new_open, Some(&event_details.event));

        let details = FloatingUIOpenChangeDetails {
            open: new_open,
            reason: event_details.reason.clone(),
            native_event: event_details.event.clone(),
            nested: self.inner.context.nested(),
            trigger_element: event_details.trigger.clone(),
        };

        self.inner.context.events.emit("openchange", &details);
    }

    /// `setOpen` (`FloatingRootStore.ts:126-135`): emits the `openchange` event through
    /// the internal event emitter and calls the `onOpenChange` handler — unless
    /// `syncOnly`, which forwards only (`:127-130`).
    pub fn set_open(&self, new_open: bool, event_details: &RootOpenChangeEventDetails) {
        if self.sync_only {
            if let Some(on_open_change) = self.inner.context.on_open_change() {
                on_open_change(new_open, event_details);
            }
            return;
        }

        self.dispatch_open_change(new_open, event_details);

        if let Some(on_open_change) = self.inner.context.on_open_change() {
            on_open_change(new_open, event_details);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use std::cell::Cell;

    /// A host-constructible virtual element — the positioning shim shape upstream's
    /// `useFloating.setPositionReference` builds (`hooks/useFloating.ts:95-101`).
    #[derive(Clone, PartialEq)]
    struct FakeVirtualElement {
        x: f64,
    }

    impl floating_ui_dom::VirtualElement<Element> for FakeVirtualElement {
        fn get_bounding_client_rect(&self) -> floating_ui_dom::ClientRectObject {
            floating_ui_dom::ClientRectObject {
                x: self.x,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                top: 0.0,
                right: self.x + 10.0,
                bottom: 10.0,
                left: self.x,
            }
        }

        fn get_client_rects(&self) -> Option<Vec<floating_ui_dom::ClientRectObject>> {
            None
        }

        fn context_element(&self) -> Option<Element> {
            None
        }
    }

    fn store_with(
        sync_only: bool,
        on_open_change: Option<OnOpenChangeFn>,
    ) -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only,
            nested: false,
            on_open_change,
        })
    }

    // Pins the constructor's state seeding (`FloatingRootStore.ts:69-74`): a null
    // reference seeds null `positionReference`/`domReferenceElement`. The element-carrying
    // seeding is exercised in the wasm suite (it needs a real DOM element).
    #[test]
    fn constructor_seeds_position_reference_from_the_reference_element() {
        let store = store_with(false, None);
        assert!(
            store.get_snapshot().position_reference.is_none(),
            "a null reference seeds a null position reference"
        );
        assert!(
            store.get_snapshot().dom_reference_element.is_none(),
            "a null reference seeds a null DOM reference"
        );
    }

    // Pins the context re-assignment surface the context hooks use
    // (`hooks/useFloatingRootContext.ts:76-77`): `onOpenChange` and `nested` are
    // writable after construction through the shared handle — upstream re-assigns them
    // every render.
    #[test]
    fn context_on_open_change_and_nested_are_reassignable() {
        let store = store_with(false, None);
        assert!(!store.context.nested());
        store.context.set_nested(true);
        assert!(store.context.nested(), "nested is patched in place");

        assert!(store.context.on_open_change().is_none());
        let seen: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let seen_handle = Rc::clone(&seen);
        store
            .context
            .set_on_open_change(Some(Rc::new(move |open: bool, _details| {
                seen_handle.set(open)
            })));
        assert!(store.context.on_open_change().is_some());
    }

    // Pins the coalescing selector (`FloatingRootStore.ts:37` —
    // `positionReference ?? referenceElement`): the position reference wins when set;
    // the base reference is the fallback; a null store yields null. Virtual arms are
    // host-constructible, so the selector's virtual-element mechanics — the mechanism
    // `useClientPoint`/`setPositionReference` ride on — pin here.
    #[test]
    fn the_reference_element_selector_coalesces_position_over_base() {
        let store = store_with(false, None);
        assert!(selectors::reference_element(&store.get_snapshot()).is_none());

        let base = ReferenceType::Virtual(VirtualReference::new(FakeVirtualElement { x: 1.0 }));
        let position = ReferenceType::Virtual(VirtualReference::new(FakeVirtualElement { x: 2.0 }));

        store.set_field(|state| &mut state.reference_element, Some(base.clone()));
        assert!(
            matches!(
                selectors::reference_element(&store.get_snapshot()),
                Some(ReferenceType::Virtual(_))
            ),
            "the base reference is the fallback when no position reference is set"
        );
        assert_eq!(
            selectors::reference_element(&store.get_snapshot()),
            Some(base.clone()),
            "the fallback is the base reference itself"
        );

        store.set_field(
            |state| &mut state.position_reference,
            Some(position.clone()),
        );
        assert_eq!(
            selectors::reference_element(&store.get_snapshot()),
            Some(position.clone()),
            "the position reference wins when set"
        );

        store.set_field(|state| &mut state.position_reference, None);
        assert_eq!(
            selectors::reference_element(&store.get_snapshot()),
            Some(base),
            "falling back after the position reference clears"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::cell::Cell;

    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn store_with(
        sync_only: bool,
        on_open_change: Option<OnOpenChangeFn>,
    ) -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only,
            nested: false,
            on_open_change,
        })
    }

    fn keydown_event() -> Event {
        web_sys::Event::new("keydown").unwrap()
    }

    fn mousemove_event() -> Event {
        web_sys::MouseEvent::new("mousemove").unwrap().into()
    }

    // Pins the standalone `setOpen` flow (`FloatingRootStore.ts:132-134`): the
    // `'openchange'` emission happens BEFORE the `onOpenChange` callback, and the
    // details payload carries open/reason/nativeEvent/nested/triggerElement
    // (`:109-115`).
    #[wasm_bindgen_test]
    fn set_open_emits_openchange_before_calling_on_open_change() {
        let order: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
        let order_for_callback = Rc::clone(&order);
        let store = store_with(
            false,
            Some(Rc::new(move |_open: bool, _details| {
                order_for_callback.borrow_mut().push("callback");
            })),
        );
        let order_for_listener = Rc::clone(&order);
        store.context.events.on(
            "openchange",
            Rc::new(move |_| order_for_listener.borrow_mut().push("emit")),
        );

        let details =
            RootOpenChangeEventDetails::new("trigger-press", keydown_event(), None, String::new());
        store.set_open(true, &details);

        assert_eq!(
            *order.borrow(),
            vec!["emit", "callback"],
            "the event bus fans out before the consumer callback"
        );
    }

    // Pins the details payload shape (`FloatingRootStore.ts:109-115`).
    #[wasm_bindgen_test]
    fn dispatch_open_change_builds_the_details_payload() {
        let store = store_with(false, None);
        store.context.set_nested(true);

        let payload: Rc<RefCell<Option<FloatingUIOpenChangeDetails>>> = Rc::new(RefCell::new(None));
        let payload_handle = Rc::clone(&payload);
        store.context.events.on(
            "openchange",
            Rc::new(move |details| *payload_handle.borrow_mut() = Some(details.clone())),
        );

        let trigger = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap();
        let event = keydown_event();
        let details = RootOpenChangeEventDetails::new(
            "trigger-press",
            event.clone(),
            Some(trigger.clone()),
            String::new(),
        );
        store.dispatch_open_change(true, &details);

        let payload = payload.borrow().clone().expect("the emission landed");
        assert!(payload.open);
        assert_eq!(payload.reason, "trigger-press");
        assert!(payload.native_event == event, "the native event is carried");
        assert!(payload.nested, "nested comes from the context");
        assert!(
            payload
                .trigger_element
                .map(|el| el == trigger)
                .unwrap_or(false),
            "the trigger element is carried"
        );
    }

    // Pins `syncOnly` (`FloatingRootStore.ts:126-130`): forwarding only — no emission.
    #[wasm_bindgen_test]
    fn sync_only_set_open_forwards_without_dispatching() {
        let calls: Rc<Cell<Vec<bool>>> = Rc::new(Cell::new(Vec::new()));
        let calls_handle = Rc::clone(&calls);
        let store = store_with(
            true,
            Some(Rc::new(move |open: bool, _details| {
                let mut log = calls_handle.take();
                log.push(open);
                calls_handle.set(log);
            })),
        );
        let emissions: Rc<Cell<u8>> = Rc::new(Cell::new(0));
        let emissions_handle = Rc::clone(&emissions);
        store.context.events.on(
            "openchange",
            Rc::new(move |_| emissions_handle.set(emissions_handle.get() + 1)),
        );

        let details =
            RootOpenChangeEventDetails::new("trigger-press", keydown_event(), None, String::new());
        store.set_open(true, &details);

        assert_eq!(calls.take(), vec![true], "the callback received the change");
        assert_eq!(
            emissions.get(),
            0,
            "no openchange emission in syncOnly mode"
        );
    }

    // Pins `syncOpenEvent`'s precedence rules (`FloatingRootStore.ts:91-101`). The write
    // guard is `!newOpen || !state.open || isClickLike(event)`: during a real open
    // transition `state.open` is still the old value (the open-state sync lands after),
    // so opens record their event through the `!state.open` branch, and the two
    // precedence rules live in the "already open" regime: a pending hover-open never
    // overwrites a click-like open event, while a click-like event upgrades a
    // hover-open. Closing always clears.
    #[wasm_bindgen_test]
    fn sync_open_event_click_and_hover_precedence() {
        let store = store_with(false, None);

        // Seed: popup open with a click-like open event.
        store.update(|state, _| {
            state.open = true;
            true
        });
        store.sync_open_event(true, Some(&keydown_event()));
        assert_eq!(
            store
                .context
                .data_ref
                .borrow()
                .open_event
                .as_ref()
                .map(|event| event.type_()),
            Some("keydown".to_owned()),
            "the click-like open event is recorded"
        );

        // A pending hover-open never overwrites the click-like open event
        // (`newOpen && state.open && !clickLike` → no write).
        store.sync_open_event(true, Some(&mousemove_event()));
        assert_eq!(
            store
                .context
                .data_ref
                .borrow()
                .open_event
                .as_ref()
                .map(|event| event.type_()),
            Some("keydown".to_owned()),
            "the hover open did not overwrite the click-like event"
        );

        // Close clears the open event unconditionally (`newOpen ? event : undefined`).
        store.sync_open_event(false, None);
        assert!(
            store.context.data_ref.borrow().open_event.is_none(),
            "closing clears the open event"
        );

        // Seed a hover-open: recorded while the state is still closed (the transition
        // moment), then upgraded by a click-like event while open.
        store.update(|state, _| {
            state.open = false;
            true
        });
        store.sync_open_event(true, Some(&mousemove_event()));
        assert_eq!(
            store
                .context
                .data_ref
                .borrow()
                .open_event
                .as_ref()
                .map(|event| event.type_()),
            Some("mousemove".to_owned()),
            "the hover open is recorded while the state is still closed"
        );
        store.update(|state, _| {
            state.open = true;
            true
        });
        store.sync_open_event(true, Some(&keydown_event()));
        assert_eq!(
            store
                .context
                .data_ref
                .borrow()
                .open_event
                .as_ref()
                .map(|event| event.type_()),
            Some("keydown".to_owned()),
            "a click-like event upgrades the hover-open"
        );
    }

    // Pins the constructor's DOM-reference seeding (`FloatingRootStore.ts:72-73`): a
    // real element seeds `domReferenceElement`; a virtual element seeds it `null`
    // (`referenceElement as Element | null`).
    #[wasm_bindgen_test]
    fn constructor_seeds_dom_reference_from_real_elements_only() {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        let store = FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: Some(ReferenceType::Element(element.clone())),
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        });
        assert_eq!(
            store.get_snapshot().dom_reference_element,
            Some(element.clone()),
            "a real element seeds the DOM reference"
        );
        assert_eq!(
            store.get_snapshot().position_reference,
            Some(ReferenceType::Element(element)),
            "the position reference starts mirrored to the reference element"
        );
    }
}
