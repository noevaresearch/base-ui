//! Port of `packages/react/src/utils/popups/popupHandle.ts` — the shared
//! implementation for popup handles that coordinate detached triggers with a mounted
//! root (`specs/library/utils/implementation.md`, "Popup store core (`popups/`)" →
//! `popupHandle.ts`). The companion hook
//! (`packages/react/src/utils/popups/usePopupHandleStore.ts`) ports as
//! [`use_popup_handle_store`] at the bottom of this module: the 36-line hook is
//! inseparable from the handle contract it reads, and a second module would carry no
//! code of its own.
//!
//! A handle is created without a Root and handed to triggers through the framework's
//! prop plumbing. While no Root is mounted, the handle exposes an inert fallback
//! store so detached triggers can render and register into it; when a Root attaches,
//! the handle points at the Root's live store, notifies subscribers (detached
//! triggers re-render and migrate their registration), and the attachment stack
//! restores the previous still-mounted root when an overlapping root detaches first.
//!
//! ## Rust adaptations
//!
//! - The `HandleStore`/`Store` generic pair (`popupHandle.ts:68-71`,
//!   `Store extends HandleStore & PopupHandleStoreWithOpen`) collapses to one
//!   parameter. The split is a TS typing device — the detached-trigger view may omit
//!   `setOpen` entirely (Dialog's and PreviewCard's do, `:47-50`) — with no runtime
//!   counterpart: in Rust both are the same concrete store handle, the narrowing
//!   lives in the [`PopupHandleStoreProvider`] trait detached triggers read (no
//!   `setOpen` member), and `setOpen` is only ever invoked through the attached
//!   root's [`PopupHandleStoreWithOpen`] impl.
//! - Upstream's structural `setOpen` member (`:51-56`) becomes
//!   [`PopupHandleStoreWithOpen`]; its impl for the concrete popup store forwards to
//!   the context's `on_open_change` slot — the Root's writer, wired by
//!   `use_popup_root_store` (the `popup_store_utils.rs` module docs' explicit-writer
//!   convention). A store created without a writer is a silent no-op: it can only be
//!   the fallback store, which upstream never calls `setOpen` on either.
//! - `attachedStores`/`attachedStoreValue`/`storeListeners` (`:78-89`) are
//!   `Rc<RefCell<…>>` cells so the deferred overlap-warning frame (`:155-175`) can
//!   observe the stack after the handle itself has moved into its owning `Rc` (the
//!   frame's callback outlives the `attach_store` call). Store identity —
//!   `this.attachedStoreValue !== store` (`:194`) and `lastIndexOf(newStore)`
//!   (`:178`) — is [`Rc::ptr_eq`], the port's object-identity probe.
//! - The `process.env.NODE_ENV !== 'production'` gates (`:155`, `:219`, `:253`,
//!   `:276`) port to `#[cfg(debug_assertions)]` (the `popup_trigger_map.rs`
//!   convention); the four console warnings route through the `warn()` log-once
//!   channel, whose dedup collapses upstream's repeat emissions (dev-warning channel
//!   difference only — the behavioral arms are ungated).
//! - `createChangeEventDetails(REASONS.imperativeAction, undefined, triggerElement)`
//!   (`:263`, `:286`) rides the factory's `event ?? new Event('base-ui')` default
//!   (`createBaseUIEventDetails.ts:129-132`); the port constructs that event
//!   explicitly (the browser-realm-only [`synthetic_base_ui_event`]) and spells the
//!   omitted `customProperties` argument as `String::new()` —
//!   `BaseUIChangeEventDetails::new` is struct sugar without the factory's defaults
//!   (the `create_base_ui_event_details.rs` module docs).
//! - The missing-trigger `throw` (`:244-251`) is a Rust panic, verbatim message and
//!   ungated like upstream (it throws in production too); the detached no-op arms
//!   keep their early returns.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::graph::untrack;
use reactive_graph::owner::LocalStorage;
use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Set;
use send_wrapper::SendWrapper;
use web_sys::Element;

use leptos_ui_utils::react_store::ReactStore;
use leptos_ui_utils::store::StoreUnsubscribe;

#[cfg(debug_assertions)]
use leptos_ui_utils::use_animation_frame::AnimationFrame;
#[cfg(debug_assertions)]
use leptos_ui_utils::warn;

use crate::floating_ui::popup_store::PopupStoreContext;
use crate::floating_ui::popup_store::PopupStoreState;
use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
use crate::floating_ui::reasons;
use crate::floating_ui::types::RootOpenChangeEventDetails;
use crate::popup_store_utils::PopupRootStoreHandle;

/// Port of `PopupHandleStoreProvider<HandleStore>` (`popupHandle.ts:17-36`): the
/// minimal store contract popup handles expose to detached triggers. Detached
/// triggers read [`PopupHandleStoreProvider::store`] during render and subscribe
/// with [`PopupHandleStoreProvider::subscribe_store`] to follow the handle between
/// its fallback store and a root's live store. Deliberately no `setOpen` — the
/// upstream narrowing that keeps detached triggers from driving open state
/// (`popupHandle.ts:47-50`).
pub trait PopupHandleStoreProvider<HandleStore: 'static> {
    /// `store` (`popupHandle.ts:119-121`) — the store currently exposed by the
    /// handle: the attached root's store, or the inert fallback while no root is
    /// attached.
    fn store(&self) -> HandleStore;

    /// `serverStore` (`popupHandle.ts:128-130`) — the stable fallback store used for
    /// server rendering and hydration, because a handle can be shared by concurrent
    /// server-rendered requests and must never record a live root store during
    /// render.
    fn server_store(&self) -> HandleStore;

    /// `subscribeStore` (`popupHandle.ts:137-143`) — subscribes to changes of the
    /// exposed store pointer; returns the unsubscribe cleanup.
    fn subscribe_store(&self, listener: Rc<dyn Fn()>) -> StoreUnsubscribe;
}

/// Port of `PopupHandleStoreWithTriggers` (`popupHandle.ts:42-44`): the store shape
/// [`BasePopupHandle::open_by_trigger`] needs — the context's trigger registry, to
/// resolve a trigger element by id on both the attached root's store and the
/// fallback store.
pub trait PopupHandleStoreWithTriggers {
    /// The store context's `triggerElements` member (`popupHandle.ts:43`).
    fn popup_trigger_elements(&self) -> &PopupTriggerMap;
}

/// Port of `PopupHandleStoreWithOpen` (`popupHandle.ts:51-56`): the root-owned
/// store's `setOpen`. Only the attached root store needs it — the detached view may
/// omit it entirely, since it is never called while detached (`:47-50`).
pub trait PopupHandleStoreWithOpen: PopupHandleStoreWithTriggers {
    /// The store's `setOpen` member (`popupHandle.ts:52-55`), driven only through
    /// [`BasePopupHandle::open_by_trigger`] / [`BasePopupHandle::close_popup`] while
    /// a root is attached.
    fn popup_set_open(&self, open: bool, event_details: RootOpenChangeEventDetails);
}

impl<P> PopupHandleStoreWithTriggers
    for ReactStore<PopupStoreState<P>, PopupStoreContext<RootOpenChangeEventDetails>>
{
    fn popup_trigger_elements(&self) -> &PopupTriggerMap {
        &self.context.trigger_elements
    }
}

impl<P> PopupHandleStoreWithOpen
    for ReactStore<PopupStoreState<P>, PopupStoreContext<RootOpenChangeEventDetails>>
{
    fn popup_set_open(&self, open: bool, event_details: RootOpenChangeEventDetails) {
        // The port's store handle carries the Root's writer as the context's
        // `on_open_change` slot (the `use_popup_root_store` wiring). A store without
        // a writer is the fallback-store shape — unreachable while attached, so the
        // missing arm is a silent no-op (see the module docs).
        if let Some(set_open) = &self.context.on_open_change {
            set_open(open, &event_details);
        }
    }
}

/// The factory-default event (`createBaseUIEventDetails.ts:129-132`,
/// `event ?? new Event('base-ui')`) the handle's `setOpen` details carry — an
/// imperative open/close has no native event. Browser-realm only: the open/close
/// paths run inside components and tests on wasm.
fn synthetic_base_ui_event() -> web_sys::Event {
    web_sys::Event::new("base-ui")
        .expect("the Event constructor is available in the browser realm")
}

/// Port of `BasePopupHandle<HandleStore, Store>` (`popupHandle.ts:68-288`): the
/// shared implementation behind every concrete popup handle. Subclasses provide the
/// component-specific imperative methods; this base owns the fallback store, the
/// root-store attachment stack, subscriber notifications, and the development
/// warning for overlapping roots.
///
/// The handle is shared (`Rc`) between the Root that attaches its store and the
/// detached triggers that subscribe to it.
pub struct BasePopupHandle<T: PopupHandleStoreWithTriggers + PopupHandleStoreWithOpen + 'static>
{
    /// `fallbackStore` (`popupHandle.ts:105`) — the inert, closed store handed to
    /// detached triggers while no root is attached.
    fallback_store: Rc<T>,

    /// `componentName` (`popupHandle.ts:106`) — prefixes the dev warnings, e.g.
    /// `'Menu'` produces `MenuHandle.open()` in warning text.
    component_name: &'static str,

    /// `throwOnMissingTrigger` (`popupHandle.ts:107`) — anchored popups (Menu,
    /// Popover, Tooltip, PreviewCard) throw on `open(triggerId)` with no matching
    /// registered trigger; Dialog is not anchored and instead opens unassociated
    /// with a dev warning.
    throw_on_missing_trigger: bool,

    /// `attachedStores` (`popupHandle.ts:78`) — the stores of every root currently
    /// using this handle, in attach order, so `attach_store`'s cleanup can restore
    /// the previous still-mounted root instead of leaving it uncontrollable.
    attached_stores: Rc<RefCell<Vec<Rc<T>>>>,

    /// `attachedStoreValue` (`popupHandle.ts:84`) — the root that currently controls
    /// the handle; imperative methods are no-ops while this is `None`.
    attached_store: Rc<RefCell<Option<Rc<T>>>>,

    /// `storeListeners` (`popupHandle.ts:89`) — the subscribers notified when the
    /// exposed store pointer changes.
    store_listeners: Rc<RefCell<Vec<Rc<dyn Fn()>>>>,

    /// The dev-only overlap-warning frame (`popupHandle.ts:155-175`), created lazily
    /// so it never appears on instances in production (upstream's `:162-164` note).
    #[cfg(debug_assertions)]
    overlap_warning_frame: RefCell<Option<AnimationFrame>>,
}

/// The `setActiveStore` private method (`popupHandle.ts:193-200`) as a free function
/// over the shared cells — the detach cleanup runs without borrowing the handle.
fn set_active_store<T: PopupHandleStoreWithTriggers + PopupHandleStoreWithOpen + 'static>(
    attached_store: &Rc<RefCell<Option<Rc<T>>>>,
    store_listeners: &Rc<RefCell<Vec<Rc<dyn Fn()>>>>,
    store: Option<Rc<T>>,
) {
    // `if (this.attachedStoreValue !== store)` (`:194`) — pointer identity.
    let changed = match (&*attached_store.borrow(), &store) {
        (Some(current), Some(next)) => !Rc::ptr_eq(current, next),
        (None, None) => false,
        _ => true,
    };
    if changed {
        *attached_store.borrow_mut() = store;
        // Cloned first so a listener subscribing during notification cannot
        // re-enter the borrowed list.
        let listeners = store_listeners.borrow().clone();
        for listener in listeners {
            listener();
        }
    }
}

impl<T: PopupHandleStoreWithTriggers + PopupHandleStoreWithOpen + 'static>
    BasePopupHandle<T>
{
    /// The constructor (`popupHandle.ts:104-108`): the fallback store, the component
    /// name for dev warnings, and whether `open(triggerId)` throws on a missing
    /// registered trigger.
    pub fn new(fallback_store: Rc<T>, component_name: &'static str, throw_on_missing_trigger: bool) -> Self {
        Self {
            fallback_store,
            component_name,
            throw_on_missing_trigger,
            attached_stores: Rc::new(RefCell::new(Vec::new())),
            attached_store: Rc::new(RefCell::new(None)),
            store_listeners: Rc::new(RefCell::new(Vec::new())),
            #[cfg(debug_assertions)]
            overlap_warning_frame: RefCell::new(None),
        }
    }

    /// The `store` getter (`popupHandle.ts:119-121`): the attached root's store, or
    /// the inert fallback store while no root is attached.
    pub fn store(&self) -> Rc<T> {
        self.attached_store
            .borrow()
            .as_ref()
            .map(Rc::clone)
            .unwrap_or_else(|| Rc::clone(&self.fallback_store))
    }

    /// The `serverStore` getter (`popupHandle.ts:128-130`): always the fallback
    /// store, because a handle can be shared by concurrent SSR requests and must
    /// never record a live root store during render.
    pub fn server_store(&self) -> Rc<T> {
        Rc::clone(&self.fallback_store)
    }

    /// `subscribeStore` (`popupHandle.ts:137-143`): notifies the listener whenever
    /// the attached store pointer changes so detached triggers re-render and
    /// re-bind. Returns the unsubscribe cleanup.
    pub fn subscribe_store(&self, listener: Rc<dyn Fn()>) -> StoreUnsubscribe {
        self.store_listeners.borrow_mut().push(Rc::clone(&listener));
        let store_listeners = Rc::clone(&self.store_listeners);
        Rc::new(move || {
            store_listeners
                .borrow_mut()
                .retain(|registered| !Rc::ptr_eq(registered, &listener));
        })
    }

    /// `attachStore` (`popupHandle.ts:151-187`): points the handle at a root's store
    /// and notifies subscribers so detached triggers re-render and re-register into
    /// it. Returns the cleanup that detaches the store again and restores control to
    /// the most recently attached root still mounted (or none).
    pub fn attach_store(&self, new_store: Rc<T>) -> Box<dyn FnOnce()> {
        self.attached_stores.borrow_mut().push(Rc::clone(&new_store));
        set_active_store(
            &self.attached_store,
            &self.store_listeners,
            Some(Rc::clone(&new_store)),
        );

        #[cfg(debug_assertions)]
        if self.attached_stores.borrow().len() > 1 {
            // More than one root is attached at once — usually a transient overlap
            // during an animated route transition (`:157-161`). Defer the check by a
            // frame and only warn if the overlap is still present once the
            // transition has settled, so a clean handoff doesn't warn regardless of
            // the exact unmount timing. The frame is created lazily so it never
            // appears on instances in production (`:162-164`).
            let mut frame_slot = self.overlap_warning_frame.borrow_mut();
            let frame = frame_slot.get_or_insert_with(AnimationFrame::create);
            let attached_stores = Rc::clone(&self.attached_stores);
            frame.request(move || {
                if attached_stores.borrow().len() > 1 {
                    warn().log(&[
                        "A handle is attached to more than one mounted root at the same time. The most recently mounted root takes over and the previous one stops being controlled by the handle. A handle should be used by a single root that stays mounted for the lifetime of the handle.",
                    ]);
                }
            });
        }

        let attached_stores = Rc::clone(&self.attached_stores);
        let attached_store = Rc::clone(&self.attached_store);
        let store_listeners = Rc::clone(&self.store_listeners);
        Box::new(move || {
            // `lastIndexOf(newStore)` (`:178`) — pointer identity.
            let index = attached_stores
                .borrow()
                .iter()
                .rposition(|store| Rc::ptr_eq(store, &new_store));
            if let Some(index) = index {
                attached_stores.borrow_mut().remove(index);
            }
            // Restore control to the most recently attached root that is still
            // mounted, or detach fully (`:182-185`) — clearing unconditionally would
            // leave a still-mounted older root uncontrollable when a newer
            // overlapping root detaches first (e.g. a canceled route transition).
            let next = attached_stores.borrow().last().cloned();
            set_active_store(&attached_store, &store_listeners, next);
        })
    }

    /// `openByTrigger` (`popupHandle.ts:215-265`): opens the attached root's store
    /// and associates it with the trigger with the given id — or a no-op (with a dev
    /// warning) while no root is attached. Shared by every concrete handle's public
    /// `open()`, which only narrows the parameter type. Call from an event handler
    /// or an effect, not during rendering.
    ///
    /// With a trigger id but no matching registered trigger, anchored popups panic
    /// ([`Self::new`]'s `throw_on_missing_trigger`); non-anchored handles open
    /// unassociated with a dev warning.
    pub fn open_by_trigger(&self, trigger_id: Option<&str>) {
        let Some(attached_store) = self.attached_store.borrow().as_ref().map(Rc::clone) else {
            #[cfg(debug_assertions)]
            warn().log(&[&format!(
                "{}Handle.open() was called while no root using this handle is mounted. The call was ignored; mount a root with this handle before opening it imperatively.",
                self.component_name
            )]);
            return;
        };

        // Resolution searches the whole attachment stack newest-first, then the
        // fallback map (`:228-241`): during the commit in which a root attaches, a
        // still-mounted detached trigger has not re-registered into that store yet —
        // it is still registered wherever it lived before, so an imperative
        // open-by-id called in that commit still resolves the trigger instead of
        // treating it as missing.
        // `if (triggerId)` (`:236`) — the empty string is falsy upstream.
        let truthy_id = trigger_id.filter(|id| !id.is_empty());
        let mut trigger_element: Option<Element> = None;
        if let Some(trigger_id) = truthy_id.as_deref() {
            let from_stack = {
                let attached_stores = self.attached_stores.borrow();
                attached_stores
                    .iter()
                    .rev()
                    .find_map(|store| store.popup_trigger_elements().get_by_id(trigger_id))
            };
            trigger_element = from_stack
                .or_else(|| self.fallback_store.popup_trigger_elements().get_by_id(trigger_id));
        }

        if truthy_id.is_some() && trigger_element.is_none() {
            let trigger_id = truthy_id.as_deref().unwrap_or_default();
            if self.throw_on_missing_trigger {
                panic!(
                    "Base UI: {}Handle.open() was called with the trigger id \"{}\", but no matching trigger is registered with this handle. An anchored popup cannot open without a trigger to anchor to. Pass the id of a mounted {}.Trigger that has this handle set on its \"handle\" prop.",
                    self.component_name, trigger_id, self.component_name
                );
            }

            #[cfg(debug_assertions)]
            warn().log(&[&format!(
                "{}Handle.open: No trigger found with id \"{}\". The popup will open, but the trigger will not be associated with it.",
                self.component_name, trigger_id
            )]);
        }

        attached_store.popup_set_open(
            true,
            RootOpenChangeEventDetails::new(
                reasons::IMPERATIVE_ACTION,
                synthetic_base_ui_event(),
                trigger_element,
                String::new(),
            ),
        );
    }

    /// `closePopup` (`popupHandle.ts:273-287`): closes the popup by setting the
    /// attached root's store to closed — or a no-op (with a dev warning) while no
    /// root is attached. Shared by every concrete handle's public `close()`. Call
    /// from an event handler or an effect, not during rendering.
    pub fn close_popup(&self) {
        let Some(attached_store) = self.attached_store.borrow().as_ref().map(Rc::clone) else {
            #[cfg(debug_assertions)]
            warn().log(&[&format!(
                "{}Handle.close() was called while no root using this handle is mounted. The call was ignored.",
                self.component_name
            )]);
            return;
        };

        attached_store.popup_set_open(
            false,
            RootOpenChangeEventDetails::new(
                reasons::IMPERATIVE_ACTION,
                synthetic_base_ui_event(),
                None,
                String::new(),
            ),
        );
    }
}

/// The Root-side half of the boundary: a `BasePopupHandle` satisfies
/// `PopupRootStoreHandle` (`popupStoreUtils.ts:59-61`) so the Root attaches its
/// store through `popup_handle_attachment` (the `popup_store_utils.rs` port).
impl<T: PopupHandleStoreWithTriggers + PopupHandleStoreWithOpen + 'static> PopupRootStoreHandle<T>
    for BasePopupHandle<T>
{
    fn attach_store(&self, store: Rc<T>) -> Box<dyn FnOnce()> {
        BasePopupHandle::attach_store(self, store)
    }
}

/// The detached-trigger half: a `BasePopupHandle` satisfies
/// `PopupHandleStoreProvider<Rc<T>>` (`popupHandle.ts:17-36`) so
/// [`use_popup_handle_store`] can read and subscribe to it.
impl<T: PopupHandleStoreWithTriggers + PopupHandleStoreWithOpen + 'static>
    PopupHandleStoreProvider<Rc<T>>
    for BasePopupHandle<T>
{
    fn store(&self) -> Rc<T> {
        BasePopupHandle::store(self)
    }

    fn server_store(&self) -> Rc<T> {
        BasePopupHandle::server_store(self)
    }

    fn subscribe_store(&self, listener: Rc<dyn Fn()>) -> StoreUnsubscribe {
        BasePopupHandle::subscribe_store(self, listener)
    }
}

/// Port of `usePopupHandleStore` (`usePopupHandleStore.ts:17-36`): reads the store
/// currently exposed by a popup handle and subscribes to store-pointer changes.
/// Detached triggers use this to follow a handle as a root attaches or detaches —
/// while no root is attached the handle exposes its fallback store; once a root
/// attaches, subscribers re-render and read from the live root store.
///
/// Returns a signal of `None` when no handle is provided so callers can fall back to
/// their root context (`usePopupHandleStore.ts:13`), and requires a reactive owner
/// for the unsubscribe-on-disposal wiring.
///
/// Rust adaptations (see the module docs): the `useSyncExternalStore` bridge becomes
/// the signal the subscription feeds — initialized to the exposed store
/// (`getSnapshot`, `:31-33`), re-read on every handle notification (notifications
/// fire only on real pointer changes — `setActiveStore` guards with identity,
/// `:194`), and unsubscribed on disposal. The undefined-handle `subscribe` arm is
/// the `NOOP` unsubscribe (`:22-24`). The server snapshot (`:35`,
/// `handle.serverStore`) has no render-pass counterpart in the reactive world yet
/// (the `use_is_hydrating.rs` SSR seam); the fallback store it reads is by
/// construction the pre-attach snapshot, so a future hydration pass seeds the same
/// value. The `handle` prop changing identity re-subscribes upstream (`:28`); the
/// port captures the handle once — component bodies run once, and a trigger's
/// handle is stable per instance the way `popup_handle_attachment`'s
/// `[handle, store]` deps are.
pub fn use_popup_handle_store<Store: Clone + 'static>(
    handle: Option<Rc<dyn PopupHandleStoreProvider<Store>>>,
) -> RwSignal<Option<Store>, LocalStorage> {
    // The initial render's value (`usePopupHandleStore.ts:31-33`).
    let value = RwSignal::new_local(untrack(|| handle.as_ref().map(|handle| handle.store())));

    if let Some(handle) = &handle {
        let unsubscribe = handle.subscribe_store(Rc::new({
            let handle = Rc::clone(handle);
            move || value.set(Some(handle.store()))
        }));
        let unsubscribe = SendWrapper::new(unsubscribe);
        on_cleanup(move || (*unsubscribe)());
    }

    value
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    //! The attachment-stack, subscription, and store-pointer semantics — pure
    //! bookkeeping with no DOM: the host suite pins them over a stub store (the
    //! popup_store_utils.rs host-suite precedent). The open/close paths construct a
    //! browser-realm event and run in the wasm suite only.

    use super::*;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    #[derive(Default)]
    struct TestStore {
        trigger_elements: PopupTriggerMap,
        opens: RefCell<Vec<(bool, Option<Element>)>>,
    }

    impl PopupHandleStoreWithTriggers for TestStore {
        fn popup_trigger_elements(&self) -> &PopupTriggerMap {
            &self.trigger_elements
        }
    }

    impl PopupHandleStoreWithOpen for TestStore {
        fn popup_set_open(&self, open: bool, event_details: RootOpenChangeEventDetails) {
            self.opens
                .borrow_mut()
                .push((open, event_details.trigger.clone()));
        }
    }

    fn handle() -> BasePopupHandle<TestStore> {
        BasePopupHandle::new(
            Rc::new(TestStore::default()),
            "Menu",
            true,
        )
    }

    fn subscriber(log: &Rc<RefCell<Vec<&'static str>>>, label: &'static str) -> Rc<dyn Fn()> {
        let log = Rc::clone(log);
        Rc::new(move || log.borrow_mut().push(label))
    }

    // `store` (`popupHandle.ts:119-121`) resolves to the fallback store while no
    // root is attached.
    #[test]
    fn store_exposes_the_fallback_store_while_detached() {
        let handle = handle();
        assert!(Rc::ptr_eq(&handle.store(), &handle.fallback_store));
    }

    // `store` resolves to the attached root's store once one attaches; `serverStore`
    // (`:128-130`) stays the fallback either way.
    #[test]
    fn store_exposes_the_attached_root_and_server_store_stays_the_fallback() {
        let handle = handle();
        let root_store = Rc::new(TestStore::default());

        handle.attach_store(Rc::clone(&root_store));
        assert!(Rc::ptr_eq(&handle.store(), &root_store));
        assert!(Rc::ptr_eq(&handle.server_store(), &handle.fallback_store));
    }

    // Subscribers are notified when the pointer changes (`:193-200`) — and the
    // identity guard means re-pointing at the same store notifies no one.
    #[test]
    fn subscribers_are_notified_only_on_pointer_changes() {
        let handle = handle();
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

        let _unsubscribe = handle.subscribe_store(subscriber(&log, "notified"));
        assert!(log.borrow().is_empty(), "subscribing alone does not notify");

        let root_store = Rc::new(TestStore::default());
        let detach_first = handle.attach_store(Rc::clone(&root_store));
        assert_eq!(*log.borrow(), vec!["notified"], "attach notifies");

        log.borrow_mut().clear();
        let detach_second = handle.attach_store(Rc::clone(&root_store));
        assert!(
            log.borrow().is_empty(),
            "re-attaching the same store is a no-op notification"
        );

        // `lastIndexOf` removal (`:178`): the second cleanup removes one stack
        // instance, and the remaining one still controls — no notification.
        detach_second();
        assert!(log.borrow().is_empty());

        detach_first();
        assert_eq!(
            *log.borrow(),
            vec!["notified"],
            "the final detach flips the pointer to no root"
        );
    }

    // The unsubscribe cleanup (`:140-142`) removes the listener.
    #[test]
    fn unsubscribing_stops_notifications() {
        let handle = handle();
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

        let unsubscribe = handle.subscribe_store(subscriber(&log, "notified"));
        unsubscribe();

        handle.attach_store(Rc::new(TestStore::default()));
        assert!(log.borrow().is_empty(), "the removed listener is silent");
    }

    // The attachment-stack restore (`:177-186`): detaching an overlapping root
    // restores the previous still-mounted root instead of leaving it
    // uncontrollable, and detaching the last root returns to the fallback store.
    #[test]
    fn detaching_restores_the_previous_still_mounted_root() {
        let handle = handle();
        let first = Rc::new(TestStore::default());
        let second = Rc::new(TestStore::default());

        let detach_first = handle.attach_store(Rc::clone(&first));
        let detach_second = handle.attach_store(Rc::clone(&second));
        assert!(Rc::ptr_eq(&handle.store(), &second));

        detach_second();
        assert!(
            Rc::ptr_eq(&handle.store(), &first),
            "the previous root regains control"
        );

        detach_first();
        assert!(
            Rc::ptr_eq(&handle.store(), &handle.fallback_store),
            "no root left: back to the fallback store"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    //! The open/close paths (`openByTrigger`/`closePopup`) and the
    //! `use_popup_handle_store` subscription, over the concrete popup store — the
    //! realm where the `setOpen` details' synthetic event can be built.

    use std::cell::Cell;

    use reactive_graph::owner::Owner;
    use reactive_graph::traits::GetUntracked;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    use super::*;
    use crate::floating_ui::popup_store::create_initial_popup_store_state;

    wasm_bindgen_test_configure!(run_in_browser);

    type ConcreteStore = ReactStore<PopupStoreState<()>, PopupStoreContext<RootOpenChangeEventDetails>>;
    type Store = Rc<ConcreteStore>;

    struct ObservedStore {
        store: Store,
        opens: Rc<RefCell<Vec<(bool, String, Option<Element>)>>>,
    }

    fn make_observed_store() -> ObservedStore {
        let opens: Rc<RefCell<Vec<(bool, String, Option<Element>)>>> =
            Rc::new(RefCell::new(Vec::new()));
        let observed = Rc::clone(&opens);
        let trigger_elements = PopupTriggerMap::new();
        let state = create_initial_popup_store_state(&trigger_elements, None, false);
        let store = Rc::new(ReactStore::with_context(
            state,
            PopupStoreContext {
                trigger_elements,
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change: Some(Rc::new(move |open: bool, details: &RootOpenChangeEventDetails| {
                    observed
                        .borrow_mut()
                        .push((open, details.reason.clone(), details.trigger.clone()));
                })),
                on_open_change_complete: None,
            },
        ));
        ObservedStore { store, opens }
    }

    fn silent_store() -> Store {
        let trigger_elements = PopupTriggerMap::new();
        let state = create_initial_popup_store_state(&trigger_elements, None, false);
        Rc::new(ReactStore::with_context(
            state,
            PopupStoreContext {
                trigger_elements,
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change: None,
                on_open_change_complete: None,
            },
        ))
    }

    fn button() -> Element {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap()
    }

    fn handle(fallback: Store, throw_on_missing_trigger: bool) -> BasePopupHandle<ConcreteStore> {
        BasePopupHandle::new(fallback, "Menu", throw_on_missing_trigger)
    }

    // Mirrors `popupHandle.ts:261-264`: the imperative open drives the attached
    // root's `setOpen(true)` with the `imperativeAction` reason and the resolved
    // trigger element as the details' trigger.
    #[wasm_bindgen_test]
    fn open_by_trigger_opens_the_attached_store_with_the_resolved_trigger() {
        let observed = make_observed_store();
        let fallback = silent_store();
        let popup_handle = handle(fallback, true);

        let detach = popup_handle.attach_store(Rc::clone(&observed.store));
        let trigger = button();
        observed
            .store
            .context
            .trigger_elements
            .add("trigger-1", trigger.clone());

        popup_handle.open_by_trigger(Some("trigger-1"));

        assert_eq!(
            *observed.opens.borrow(),
            vec![(true, "imperative-action".to_owned(), Some(trigger))],
        );
        detach();
    }

    // Mirrors `popupHandle.ts:228-241`: during the attach commit a detached trigger
    // has not re-registered into the root's store yet — the resolution searches the
    // whole attachment stack newest-first, then the fallback map.
    #[wasm_bindgen_test]
    fn open_by_trigger_resolves_through_the_stack_and_the_fallback_store() {
        let observed = make_observed_store();
        let fallback = silent_store();
        let popup_handle = handle(Rc::clone(&fallback), true);

        // The trigger still lives in the fallback map (first root to attach).
        let trigger = button();
        fallback
            .context
            .trigger_elements
            .add("trigger-1", trigger.clone());

        let detach_first = popup_handle.attach_store(Rc::clone(&observed.store));
        let overlap = make_observed_store();
        let detach_second = popup_handle.attach_store(Rc::clone(&overlap.store));

        // A second trigger registered only in the previously attached root's store
        // resolves through the stack during the transient overlap.
        let other = button();
        observed
            .store
            .context
            .trigger_elements
            .add("trigger-2", other.clone());

        popup_handle.open_by_trigger(Some("trigger-2"));
        assert_eq!(
            overlap.opens.borrow().len(),
            1,
            "the newest attached root was opened"
        );
        assert_eq!(
            overlap.opens.borrow()[0].2,
            Some(other),
            "the element resolved from the previously attached store"
        );
        detach_second();

        popup_handle.open_by_trigger(Some("trigger-1"));
        assert_eq!(
            observed.opens.borrow().len(),
            1,
            "back to the first attached root"
        );
        assert_eq!(
            observed.opens.borrow()[0].2,
            Some(trigger),
            "the element resolved from the fallback store during the migration window"
        );
        detach_first();
    }

    // Mirrors `popupHandle.ts:243-251`: an anchored popup's `open(triggerId)` with
    // no matching registered trigger throws (in every mode).
    #[wasm_bindgen_test]
    #[should_panic(expected = "Base UI: MenuHandle.open() was called with the trigger id \"ghost\", but no matching trigger is registered with this handle")]
    fn open_by_trigger_panics_when_the_anchored_trigger_is_missing() {
        let observed = make_observed_store();
        let popup_handle = handle(silent_store(), true);
        popup_handle.attach_store(Rc::clone(&observed.store));

        popup_handle.open_by_trigger(Some("ghost"));
    }

    // Mirrors `popupHandle.ts:252-258`: a non-anchored handle (`throwOnMissingTrigger:
    // false`, Dialog) opens unassociated with a dev warning instead of throwing.
    #[wasm_bindgen_test]
    fn open_by_trigger_without_throw_opens_unassociated() {
        let observed = make_observed_store();
        let popup_handle = handle(silent_store(), false);
        popup_handle.attach_store(Rc::clone(&observed.store));

        popup_handle.open_by_trigger(Some("ghost"));

        assert_eq!(
            *observed.opens.borrow(),
            vec![(true, "imperative-action".to_owned(), None)],
            "the popup opens with no trigger associated"
        );
    }

    // Mirrors `popupHandle.ts:218-226`: the imperative open is a no-op while no
    // root using the handle is mounted.
    #[wasm_bindgen_test]
    fn open_by_trigger_while_detached_is_ignored() {
        let observed = make_observed_store();
        let popup_handle = handle(Rc::clone(&observed.store), true);

        popup_handle.open_by_trigger(Some("trigger-1"));

        assert!(
            observed.opens.borrow().is_empty(),
            "the detached store was never opened"
        );
    }

    // Mirrors `popupHandle.ts:286`: the imperative close drives the attached root's
    // `setOpen(false)` with the `imperativeAction` reason and no trigger.
    #[wasm_bindgen_test]
    fn close_popup_sets_the_attached_store_closed() {
        let observed = make_observed_store();
        let popup_handle = handle(silent_store(), true);
        popup_handle.attach_store(Rc::clone(&observed.store));

        popup_handle.close_popup();

        assert_eq!(
            *observed.opens.borrow(),
            vec![(false, "imperative-action".to_owned(), None)],
        );
    }

    // Mirrors `popupHandle.ts:276-284`: the imperative close is a no-op while
    // detached.
    #[wasm_bindgen_test]
    fn close_popup_while_detached_is_ignored() {
        let observed = make_observed_store();
        let popup_handle = handle(Rc::clone(&observed.store), true);

        popup_handle.close_popup();

        assert!(observed.opens.borrow().is_empty());
    }

    // Mirrors `usePopupHandleStore.ts:17-36`: the hook reads the exposed store,
    // follows the pointer flips between fallback and root store, and reads `None`
    // with no handle so callers can fall back to their root context.
    #[wasm_bindgen_test]
    fn use_popup_handle_store_follows_the_store_pointer() {
        let owner = Owner::new();
        owner.set();

        let observed = make_observed_store();
        let fallback = silent_store();
        let concrete = Rc::new(handle(Rc::clone(&fallback), true));
        let erased: Rc<dyn PopupHandleStoreProvider<Store>> = concrete.clone();

        let store = use_popup_handle_store(Some(Rc::clone(&erased)));
        assert!(
            Rc::ptr_eq(store.get_untracked().as_ref().unwrap(), &fallback),
            "detached: the fallback store"
        );

        let detach = concrete.attach_store(Rc::clone(&observed.store));
        assert!(
            Rc::ptr_eq(store.get_untracked().as_ref().unwrap(), &observed.store),
            "attached: the live root store"
        );

        detach();
        assert!(
            Rc::ptr_eq(store.get_untracked().as_ref().unwrap(), &fallback),
            "detached again: the fallback store"
        );

        let none = use_popup_handle_store::<Store>(None);
        assert!(none.get_untracked().is_none(), "no handle: None");

        owner.unset();
    }
}
