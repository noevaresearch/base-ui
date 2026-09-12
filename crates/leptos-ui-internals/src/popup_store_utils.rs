//! Port of `packages/react/src/utils/popups/popupStoreUtils.ts` — the store-level
//! machinery (open-change sequencing, trigger registration) plus the render-coupled
//! hooks that compose it with the reactive effects (`usePopupRootStore`,
//! `PopupHandleAttachment`, `useTriggerDataForwarding`, `useImplicitActiveTrigger`,
//! `useOpenStateTransitions`, `usePopupInteractionProps`, `usePopupRootSync`).
//!
//! Rust adaptations:
//! - `applyPopupOpenChange`'s hover flush (`popupStoreUtils.ts:302-307`,
//!   `ReactDOM.flushSync(changeState)`) has no port-side counterpart: the crate's
//!   store writes are synchronous (the `use_animations_finished.rs` flushSync note),
//!   so both branches collapse into the direct call.
//! - `attachPreventUnmountOnClose` installs the request flag on the event details
//!   dynamically upstream (`:223-231`); the port carries it as an optional shared cell
//!   on [`BaseUIChangeEventDetails`] (see that type's field docs) — [`attach_prevent_unmount_on_close`]
//!   attaches it and returns the reader.
//! - `FOCUSABLE_POPUP_PROPS` (`:32-35`) is a static attribute bag upstream
//!   (`tabIndex: -1` plus [`FOCUSABLE_ATTRIBUTE`]); the port spells it as the DOM
//!   attribute pairs the view layer merges.
//! - Upstream's store type is narrowed structurally (`PopupStoreWithOpen`,
//!   `:47-53`) to read the Root's `setOpen` off the created store; the port's
//!   `ReactStore` handle carries no `setOpen` member, so the hooks that open/close
//!   take it as an explicit [`OnOpenChangeFn`] parameter — the Phase B family Roots
//!   forward their store's writer.
//! - The store-level writes this module performs are idempotent fixpoints (every
//!   write is guarded by a change check), so a subscription-feeding-effect loop (the
//!   `useImplicitActiveTrigger` reconciliation re-running on its own store writes)
//!   converges exactly like the upstream layout-effect-deps cycle does across
//!   renders.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, GetValue, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use web_sys::Element;

use leptos_ui_utils::use_id::use_id;
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_ref_with_init::use_ref_with_init;
use leptos_ui_utils::use_stable_callback::{StableCallback, use_stable_callback};

use crate::create_base_ui_event_details::BaseUIChangeEventDetails;
use crate::floating_ui::constants::FOCUSABLE_ATTRIBUTE;
use crate::floating_ui::popup_store::{InstantType, PopupStoreContext, PopupStoreState, selectors};
use crate::floating_ui::tree::use_floating_parent_node_id;
use crate::floating_ui::types::OnOpenChangeFn;
use crate::floating_ui::types::RootOpenChangeEventDetails;
use crate::floating_ui::use_synced_floating_root_context::{
    UseSyncedFloatingRootContextOptions, use_synced_floating_root_context,
};
use crate::use_open_change_complete::{UseOpenChangeCompleteParams, use_open_change_complete};
use crate::use_transition_status::{UseTransitionStatus, use_transition_status};

/// The store handle the utils operate on — upstream's `PopupTriggerDataStore<State>`
/// narrowing (`store.ts:223-226`) is structural; the port passes the concrete
/// `ReactStore` handle.
pub type PopupStore<P> = Rc<
    leptos_ui_utils::react_store::ReactStore<
        PopupStoreState<P>,
        PopupStoreContext<RootOpenChangeEventDetails>,
    >,
>;

/// `FOCUSABLE_POPUP_PROPS` (`popupStoreUtils.ts:32-35`): `tabIndex: -1` plus the
/// floating-ui focusable marker, as the DOM attribute pairs the view layer spreads.
pub fn focusable_popup_props() -> Vec<(String, String)> {
    vec![
        ("tabindex".to_owned(), "-1".to_owned()),
        (FOCUSABLE_ATTRIBUTE.to_owned(), String::new()),
    ]
}

/// Port of `createDefaultInitialFocus` (`popupStoreUtils.ts:42-45`): when opened by
/// touch the popup element is focused (keeping the virtual keyboard closed — required
/// for Android; iOS handles this automatically), otherwise the default behavior runs.
/// The port returns the focus-manager's `InitialFocus` function form; the popup ref is
/// the store context's `popupRef` slot.
pub fn create_default_initial_focus(
    popup_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
) -> crate::floating_ui::floating_focus_manager::InitialFocus {
    use crate::floating_ui::floating_focus_manager::{InitialFocus, ResolvedFocusTarget};
    use leptos_ui_utils::use_enhanced_click_handler::InteractionType;

    InitialFocus::Fn(Rc::new(move |interaction_type: InteractionType| {
        use web_sys::wasm_bindgen::JsCast;
        if interaction_type == InteractionType::Touch {
            // The `Cell` slot is read with a take/restore (the labelable ref-slot
            // convention — a `Cell` has no non-destructive read for non-`Copy` values).
            let taken = popup_ref.take();
            let resolved = taken
                .clone()
                .map(|element| ResolvedFocusTarget::Element(element.dyn_into().unwrap()));
            popup_ref.set(taken);
            resolved.unwrap_or(ResolvedFocusTarget::Default)
        } else {
            ResolvedFocusTarget::Default
        }
    }))
}

/// Port of `syncTriggerCount` (`popupStoreUtils.ts:122-127`): while the popup is open,
/// the store's `triggerCount` mirrors the trigger registry's size. Registrations while
/// closed do not notify the store (behavior.md, "Edge cases → Trigger registration").
pub fn sync_trigger_count<P: Clone + 'static>(store: &PopupStore<P>) {
    let snapshot = store.get_snapshot();
    // `store.select('open')` (`:124`).
    let trigger_count = store.context.trigger_elements.size() as u32;
    if selectors::open(&snapshot) && snapshot.trigger_count != trigger_count {
        store.set_field(|state| &mut state.trigger_count, trigger_count);
    }
}

/// Port of `useTriggerRegistration` (`popupStoreUtils.ts:144-183`): the stable
/// registration callback for the trigger element. The live registration is tracked as
/// a `(store, id, element)` triple so unregistering targets the store the element was
/// actually registered in (`:148-175`); a repeat call registering the same triple is a
/// no-op (`:158-165`).
///
/// The caller must re-run the callback from a layout effect keyed on `[store, id]` to
/// migrate an already-registered element (`:135-139`).
pub fn use_trigger_registration<P: Clone + 'static>(
    id: Option<String>,
    store: &PopupStore<P>,
) -> StableCallback<Option<Element>, ()> {
    // The upstream `registrationRef` (`:148-152`), owned by the stable callback so the
    // tracked triple lives as long as the registration identity does.
    let registration: Rc<RefCell<Option<(PopupStore<P>, String, Element)>>> =
        Rc::new(RefCell::new(None));
    let registration_cell = Rc::clone(&registration);
    let store = Rc::clone(store);
    use_stable_callback(Some(move |element: Option<Element>| {
        let previous = registration_cell.borrow_mut().take();

        if let Some((registered_store, registered_id, registered_element)) = previous {
            let same_registration = Rc::ptr_eq(&registered_store, &store)
                && Some(&registered_element) == element.as_ref()
                && Some(&registered_id) == id.as_ref();

            if same_registration {
                // Already registered where it belongs, so the caller's migration effect
                // is free on mount (`:163`).
                *registration_cell.borrow_mut() =
                    Some((registered_store, registered_id, registered_element));
                return;
            }

            // Unregister from the store the element was actually registered in
            // (`:168-175`).
            if registered_store
                .context
                .trigger_elements
                .get_by_id(&registered_id)
                .as_ref()
                == Some(&registered_element)
            {
                registered_store
                    .context
                    .trigger_elements
                    .delete(&registered_id);
                sync_trigger_count(&registered_store);
            }
        }

        if let (Some(element), Some(id)) = (&element, &id) {
            *registration_cell.borrow_mut() =
                Some((Rc::clone(&store), id.clone(), element.clone()));
            store.context.trigger_elements.add(id, element.clone());
            sync_trigger_count(&store);
        }
    }))
}

/// Port of `createPopupOpenState` (`popupStoreUtils.ts:190-221`) — the pure next-state
/// builder tested directly (behavior.md, "State model → `createPopupOpenState`").
#[derive(Clone, Debug, PartialEq)]
pub struct PopupOpenState {
    pub open: bool,
    pub prevent_unmounting_on_close: bool,
    pub active_trigger_id: Option<String>,
    pub active_trigger_element: Option<Element>,
}

pub fn create_popup_open_state<P>(
    state: &PopupStoreState<P>,
    open: bool,
    trigger: Option<&Element>,
    prevent_unmount_on_close: bool,
) -> PopupOpenState {
    let mut prevent_unmounting_on_close = state.prevent_unmounting_on_close;
    if open {
        // Opening starts a new close cycle, so clear any previous request to keep the
        // popup mounted (`:197-199`).
        prevent_unmounting_on_close = false;
    } else if prevent_unmount_on_close {
        prevent_unmounting_on_close = true;
    }

    let trigger_id = trigger.and_then(|element| {
        let id = element.get_attribute("id");
        id
    });
    let mut active_trigger_id = state.active_trigger_id.clone();
    let mut active_trigger_element = state.active_trigger_element.clone();

    // If a popup is closing, the `trigger` may be undefined — keep the previous value
    // so exit animations play and focus returns correctly (`:208-213`).
    if trigger_id.is_some() || open {
        active_trigger_id = trigger_id;
        active_trigger_element = trigger.cloned();
    }

    PopupOpenState {
        open,
        prevent_unmounting_on_close,
        active_trigger_id,
        active_trigger_element,
    }
}

/// The reader half of [`attach_prevent_unmount_on_close`] — the shared flag cell,
/// queried with `.get()` after `onOpenChange` returns (`popupStoreUtils.ts:283`).
pub type PreventUnmountOnCloseFlag = Rc<Cell<bool>>;

/// Port of `attachPreventUnmountOnClose` (`popupStoreUtils.ts:223-231`): installs the
/// request flag on the details and returns the reader evaluated after
/// `onOpenChange` returns (`popupStoreUtils.ts:283`).
pub fn attach_prevent_unmount_on_close(
    event_details: &mut RootOpenChangeEventDetails,
) -> PreventUnmountOnCloseFlag {
    event_details.attach_prevent_unmount_on_close()
}

/// The options bag of [`apply_popup_open_change`] (`popupStoreUtils.ts:255-258`).
/// `extra_state` is applied *before* the popup open state — upstream merges
/// `{ ...options.extraState, ...popupOpenState }` (`:286-289`), so the open state's
/// fields win and the requested `open` overrides any extra `open`.
pub struct ApplyPopupOpenChangeOptions<'a, P> {
    /// `onBeforeDispatch` (`:256`) — e.g. updating inline-rect coordinates.
    pub on_before_dispatch: Option<Box<dyn Fn() + 'a>>,
    /// `extraState` (`:257`) — e.g. the last change reason.
    pub extra_state: Option<Box<dyn Fn(&mut PopupStoreState<P>) + 'a>>,
}

impl<'a, P> Default for ApplyPopupOpenChangeOptions<'a, P> {
    fn default() -> Self {
        Self {
            on_before_dispatch: None,
            extra_state: None,
        }
    }
}

/// Port of `applyPopupOpenChange` (`popupStoreUtils.ts:241-308`): the shared
/// open-change sequence — notify `onOpenChange`, honor cancellation, dispatch the
/// floating root change, map the reason to an `instantType`, and commit the state
/// update (behavior.md, "Events → `applyPopupOpenChange`").
pub fn apply_popup_open_change<P: Clone + 'static>(
    store: &PopupStore<P>,
    next_open: bool,
    event_details: &mut RootOpenChangeEventDetails,
    options: ApplyPopupOpenChangeOptions<'_, P>,
) {
    let reason = event_details.reason.clone();
    let is_hover = reason == crate::floating_ui::reasons::TRIGGER_HOVER;
    let is_focus_open = next_open && reason == crate::floating_ui::reasons::TRIGGER_FOCUS;
    let is_dismiss_close = !next_open
        && (reason == crate::floating_ui::reasons::TRIGGER_PRESS
            || reason == crate::floating_ui::reasons::ESCAPE_KEY);

    let should_prevent_unmount_on_close = attach_prevent_unmount_on_close(event_details);

    if let Some(on_open_change) = &store.context.on_open_change {
        on_open_change(next_open, event_details);
    }

    if event_details.is_canceled() {
        return;
    }

    if let Some(on_before_dispatch) = &options.on_before_dispatch {
        on_before_dispatch();
    }

    store
        .get_snapshot()
        .floating_root_context
        .dispatch_open_change(next_open, event_details);

    let change_state = |store: &PopupStore<P>| {
        let popup_open_state = create_popup_open_state(
            &store.get_snapshot(),
            next_open,
            event_details.trigger.as_ref(),
            should_prevent_unmount_on_close.get(),
        );

        store.update(|state, _| {
            if let Some(extra_state) = &options.extra_state {
                extra_state(state);
            }
            state.open = popup_open_state.open;
            state.prevent_unmounting_on_close = popup_open_state.prevent_unmounting_on_close;
            state.active_trigger_id = popup_open_state.active_trigger_id.clone();
            state.active_trigger_element = popup_open_state.active_trigger_element.clone();

            // The reason → instantType mapping (`:291-297`): a focus open and a
            // dismissal close set their keys, a hover open writes `undefined`
            // (clearing), and every other reason leaves the previous value.
            if is_focus_open {
                state.instant_type = Some(InstantType::Focus);
            } else if is_dismiss_close {
                state.instant_type = Some(InstantType::Dismiss);
            } else if is_hover {
                state.instant_type = None;
            }
            true
        });
    };

    // Flush synchronously for hover so `node.getAnimations()` sees the new state
    // (`:302-304`) — the port's store writes are already synchronous.
    change_state(store);
}

/// Port of `usePopupRootStore` (`popupStoreUtils.ts:73-96`): creates and owns a popup
/// store on behalf of a Root part. The store is created exactly once (`useRefWithInit`,
/// `:84`), with the floating id and nesting state resolved on the first call, then the
/// synced floating root context is set up (`:86-93`) so controlled props and open
/// changes flow both ways. Must be called inside a reactive owner (a component).
///
/// Rust adaptations:
/// - `useId()` (`:81`) — the port's [`use_id`] always yields a string (its module docs
///   record React 17's `undefined`-fallback path as unportable), so the floating id is
///   always `Some`.
/// - Upstream reads `store.setOpen` off the created store (`PopupStoreWithOpen`,
///   `:47-53`); the port passes the Root's own writer as [`set_open`] (see the module
///   docs).
pub fn use_popup_root_store<P: Clone + 'static>(
    create_store: impl FnOnce(Option<String>, bool) -> PopupStore<P>,
    set_open: OnOpenChangeFn,
    treat_popup_as_floating_element: bool,
) -> PopupStore<P> {
    // `const floatingId = useId()` (`:81`).
    let floating_id = use_id(RwSignal::<Option<String>>::new(None), None);
    // `const nested = useFloatingParentNodeId() != null` (`:82`).
    let nested = use_floating_parent_node_id().is_some();

    // `const store = useRefWithInit(() => createStore(floatingId, nested)).current`
    // (`:84`) — the factory runs exactly once.
    let store = use_ref_with_init({
        let floating_id = floating_id.clone();
        move || create_store(Some(floating_id.get_untracked()), nested)
    })
    .get_value();

    // `useSyncedFloatingRootContext({ popupStore: store, … })` (`:86-93`) — the
    // floating root context is read off the created store's state (`:89`).
    use_synced_floating_root_context(UseSyncedFloatingRootContextOptions {
        popup_store: Rc::clone(&store),
        treat_popup_as_floating_element,
        floating_root_context: Some(store.get_snapshot().floating_root_context.clone()),
        floating_id: Some(floating_id.get_untracked()),
        nested,
        on_open_change: set_open,
    });

    store
}

/// Port of `PopupRootStoreHandle<Store>` (`popupStoreUtils.ts:59-61`): the subset of a
/// popup handle that a Root needs to bind its store to. Both the real handle classes
/// (Phase B's `BasePopupHandle` port) and any test double satisfy it.
pub trait PopupRootStoreHandle<S>: 'static {
    /// `attachStore(store)` (`:60`) — returns the detach cleanup.
    fn attach_store(&self, store: Rc<S>) -> Box<dyn FnOnce()>;
}

/// Port of `PopupHandleAttachment` (`popupStoreUtils.ts:108-120`): attaches a Root's
/// store to a handle for this component's committed lifetime. Upstream is a
/// null-rendering component rendered before the Root's interactions and user children
/// so its layout effect runs before descendant layout effects — letting descendants
/// call the handle during the Root's initial commit without attaching during render,
/// which would leak suspended or abandoned stores (`:99-107`). Leptos has no
/// null-render component: the port is the layout-effect call the Root part makes in
/// the same position (before rendering descendants), preserving the effect-phase (not
/// render-phase) attach. Store subscribers are notified by `attachStore` in this
/// ordinary layout phase, where synchronous updates are permitted.
///
/// The `[handle, store]` deps (`:117`) are stable per Root instance — a Root's handle
/// and store do not change identity over its lifetime — so the effect attaches once
/// and the cleanup detaches on disposal.
pub fn popup_handle_attachment<S: 'static>(handle: Rc<dyn PopupRootStoreHandle<S>>, store: Rc<S>) {
    use_iso_layout_effect(move || {
        let detach = handle.attach_store(Rc::clone(&store));
        let detach = SendWrapper::new(detach);
        on_cleanup(move || detach.take()());
    });
}

/// The trigger-owned state a [`use_trigger_data_forwarding`] caller applies while the
/// trigger is active — upstream's `stateUpdates: Pick<State, Key>` (`:325`), spread
/// into each update; the port takes the mutation closure, applied after the
/// active-trigger fields are written.
pub type TriggerStateUpdates<P> = Rc<dyn Fn(&mut PopupStoreState<P>)>;

/// The return of [`use_trigger_data_forwarding`] — upstream's
/// `{ registerTrigger, isMountedByThisTrigger }` (`:387`).
pub struct UseTriggerDataForwarding {
    /// The stable registration callback to merge into the trigger element's ref
    /// (`:387`).
    pub register_trigger: StableCallback<Option<Element>, ()>,
    /// Whether the popup's mounted state is owned by this trigger
    /// (`store.useState('isMountedByTrigger', triggerId)`, `:327`).
    pub is_mounted_by_this_trigger: RwSignal<bool, reactive_graph::owner::LocalStorage>,
}

/// Reads a `Cell<Option<Element>>` slot non-destructively (the labelable ref-slot
/// take/restore convention — a `Cell` has no non-destructive read for non-`Copy`
/// values).
fn element_slot_read(slot: &Cell<Option<Element>>) -> Option<Element> {
    let taken = slot.take();
    let resolved = taken.clone();
    slot.set(taken);
    resolved
}

/// Port of `useTriggerDataForwarding` (`popupStoreUtils.ts:318-388`): sets up trigger
/// data forwarding to the store — registration composed with active-trigger data
/// claiming. Must be called inside a reactive owner (a component).
///
/// Rust adaptations:
/// - `triggerElementRef` (`:323`) is the port's shared `Cell` slot; the caller keeps
///   it current as the element renders.
/// - The second effect's `...Object.values(stateUpdates)` deps (`:385`) have no
///   counterpart — the closure is stable and its writes are idempotent, so the
///   re-run is driven by the mounted flag alone.
/// - The `'isMountedByTrigger'` selector argument is the hook-time id (component
///   bodies run once); a family whose trigger id changes reactively bridges it
///   through the migration effect the way the upstream re-render does.
pub fn use_trigger_data_forwarding<P: Clone + 'static>(
    trigger_id: Option<String>,
    trigger_element_ref: Rc<Cell<Option<Element>>>,
    store: &PopupStore<P>,
    state_updates: TriggerStateUpdates<P>,
) -> UseTriggerDataForwarding {
    // `store.useState('isMountedByTrigger', triggerId)` (`:327`).
    let is_mounted_by_this_trigger = store.use_state({
        let trigger_id = trigger_id.clone();
        move |state| selectors::is_mounted_by_trigger(state, trigger_id.as_deref())
    });

    let base_register_trigger = use_trigger_registration(trigger_id.clone(), store);

    // `applyTriggerData` (`:334-358`) — applies trigger-owned state (active-trigger
    // ownership and payload) when the trigger registers. Stable so payload /
    // `stateUpdates` changes do not change the ref identity (which would needlessly
    // churn registration); it reads the latest closure values when invoked.
    let apply_store = Rc::clone(store);
    let apply_state_updates = Rc::clone(&state_updates);
    let apply_trigger_data = use_stable_callback(Some(move |element: Element| {
        let snapshot = apply_store.get_snapshot();
        let open = selectors::open(&snapshot);
        let active_trigger_id = snapshot.active_trigger_id.clone();

        if active_trigger_id.as_deref() == trigger_id.as_deref() {
            let state_updates = Rc::clone(&apply_state_updates);
            let trigger_element = element.clone();
            apply_store.update(move |state, _| {
                state.active_trigger_element = Some(trigger_element);
                if open {
                    state_updates(state);
                }
                true
            });
            return;
        }

        if active_trigger_id.is_none() && open {
            // If a popup is already open, a detached trigger can mount before any
            // active trigger has been established. Claim the first registered trigger
            // so trigger-owned focus management and ARIA relationships work
            // (`:347-357`).
            let state_updates = Rc::clone(&apply_state_updates);
            let trigger_id = trigger_id.clone();
            let trigger_element = element.clone();
            apply_store.update(move |state, _| {
                state.active_trigger_id = trigger_id;
                state.active_trigger_element = Some(trigger_element);
                state_updates(state);
                true
            });
        }
    }));

    // `registerTrigger` (`:362-367`) — stable, so the merged ref on the rendered
    // element keeps its identity for the trigger's whole lifetime.
    let base_register = base_register_trigger.clone();
    let apply = apply_trigger_data.clone();
    let register_trigger = use_stable_callback(Some(move |element: Option<Element>| {
        base_register.call(element.clone());
        if let Some(element) = element {
            apply.call(element);
        }
    }));

    // A stable ref does not re-fire on a store or id change, so migrate here instead:
    // unregister from the previous store, then register the element the trigger still
    // renders into the current one (`:369-374`).
    {
        let register = register_trigger.clone();
        let slot = Rc::clone(&trigger_element_ref);
        use_iso_layout_effect(move || {
            register.call(element_slot_read(&slot));
            let register = register.clone();
            let cleanup = SendWrapper::new(move || {
                register.call(None);
            });
            on_cleanup(move || (*cleanup)());
        });
    }

    // Refresh `activeTriggerElement` + trigger-owned state while mounted
    // (`:376-385`).
    {
        let store = Rc::clone(store);
        let slot = Rc::clone(&trigger_element_ref);
        let state_updates = Rc::clone(&state_updates);
        use_iso_layout_effect(move || {
            if is_mounted_by_this_trigger.get() {
                let trigger_element = element_slot_read(&slot);
                let state_updates = Rc::clone(&state_updates);
                store.update(move |state, _| {
                    state.active_trigger_element = trigger_element;
                    state_updates(state);
                    true
                });
            }
        });
    }

    UseTriggerDataForwarding {
        register_trigger,
        is_mounted_by_this_trigger,
    }
}

/// Port of `useImplicitActiveTrigger` (`popupStoreUtils.ts:416-537`): keeps trigger
/// registration state synchronized while the popup is open — the Root-side
/// reconciliation loop. Must be called inside a reactive owner (a component).
///
/// When a popup opens without an explicit trigger id and exactly one trigger is
/// registered, that trigger is claimed as the active trigger. When the active trigger
/// id is still registered but its element changed, the active element is refreshed.
/// When the active trigger id is missing from the registry but the same element is
/// still registered under a different id, the active id is reassociated to the
/// registered id instead of being treated as lost. When the active trigger
/// unregisters, the default path preserves existing ownership so non-closing popup
/// families do not silently claim a different trigger while staying open.
///
/// If `close_on_active_trigger_unmount` is enabled, unregistering a previously
/// resolved active trigger requests a close after a microtask so a same-tick
/// replacement trigger with the same id can register first. An active trigger id that
/// has not matched a registered trigger yet is treated as pending and does not request
/// a close.
///
/// Rust adaptations:
/// - The `store.setOpen` member (`:516`) is the explicit [`set_open`] parameter (see
///   the module docs).
/// - The deferred close lands on `window.queueMicrotask` on wasm (the
///   `composite_list.rs` scheduling precedent); the host target runs it inline, since
///   this module's tests for the deferred path are browser-only.
/// - The same-element-under-another-id search (`:462-469`) iterates the registry
///   without an ordering guarantee (the port's map is `HashMap`-backed; upstream's is
///   an insertion-ordered `Map`). Ordering only matters when one element is
///   registered under several ids — a production-only shape the dev guard panics on.
pub fn use_implicit_active_trigger<P: Clone + 'static>(
    store: &PopupStore<P>,
    set_open: OnOpenChangeFn,
    close_on_active_trigger_unmount: bool,
) {
    // `resolvedActiveTriggerIdRef` (`:424`) — distinguishes a trigger that unmounted
    // from a new active trigger that has not hydrated yet.
    let resolved_active_trigger_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // The four subscriptions (`:425-435`); reading all of them inside the effect is
    // the deps array (`:529-536`).
    let open = store.use_state(selectors::open);
    let reactive_trigger_count = store.use_state(selectors::trigger_count);
    let active_trigger_id = store.use_state(selectors::active_trigger_id);
    let reactive_active_trigger_element = store.use_state(selectors::active_trigger_element);

    let store = Rc::clone(store);
    use_iso_layout_effect(move || {
        let open_now = open.get();
        let _reactive_trigger_count = reactive_trigger_count.get();
        let _active_trigger_id = active_trigger_id.get();
        let _reactive_active_trigger_element = reactive_active_trigger_element.get();

        if !open_now {
            *resolved_active_trigger_id.borrow_mut() = None;
            if store.get_snapshot().trigger_count != 0 {
                store.set_field(|state: &mut PopupStoreState<P>| &mut state.trigger_count, 0);
            }
            return;
        }

        let trigger_count = store.context.trigger_elements.size() as u32;
        // `stateUpdates` (`:447-450`): the outer `Option` distinguishes "no update"
        // (upstream's `undefined`) from an update to `None` (upstream's `null`).
        let mut updates_trigger_count: Option<u32> = None;
        let mut updates_active_trigger_id: Option<Option<String>> = None;
        let mut updates_active_trigger_element: Option<Option<Element>> = None;

        if store.get_snapshot().trigger_count != trigger_count {
            updates_trigger_count = Some(trigger_count);
        }

        let snapshot = store.get_snapshot();
        let current_active_trigger_id = snapshot.active_trigger_id.clone();
        let mut lost_active_trigger_id: Option<String> = None;

        if let Some(current_id) = &current_active_trigger_id {
            let registered_element = store.context.trigger_elements.get_by_id(current_id);
            match registered_element {
                None => {
                    // The same element registered under another id (e.g. the rendered
                    // trigger carries its own DOM `id` differing from the internal
                    // one) reassociates the active id (`:462-469`).
                    let active_element = snapshot.active_trigger_element.clone();
                    let mut reassociated: Option<(String, Element)> = None;
                    store
                        .context
                        .trigger_elements
                        .for_each_entry(|trigger_id, trigger_element| {
                            if reassociated.is_none()
                                && Some(trigger_element) == active_element.as_ref()
                            {
                                reassociated =
                                    Some((trigger_id.to_owned(), trigger_element.clone()));
                            }
                        });

                    if let Some((found_id, found_element)) = reassociated {
                        updates_active_trigger_id = Some(Some(found_id.clone()));
                        updates_active_trigger_element = Some(Some(found_element));
                        *resolved_active_trigger_id.borrow_mut() = Some(found_id);
                    } else if resolved_active_trigger_id.borrow().as_deref() == Some(current_id) {
                        lost_active_trigger_id = Some(current_id.clone());
                    } else {
                        *resolved_active_trigger_id.borrow_mut() = None;
                    }
                }
                Some(registered_element) => {
                    *resolved_active_trigger_id.borrow_mut() = Some(current_id.clone());
                    if Some(&registered_element) != snapshot.active_trigger_element.as_ref() {
                        updates_active_trigger_element = Some(Some(registered_element));
                    }
                }
            }
        } else {
            *resolved_active_trigger_id.borrow_mut() = None;
        }

        // The implicit claim (`:488-496`): open with no active trigger and exactly one
        // registration claims that trigger.
        if lost_active_trigger_id.is_none()
            && current_active_trigger_id.is_none()
            && trigger_count == 1
        {
            let mut implicit_claim: Option<(String, Element)> = None;
            store
                .context
                .trigger_elements
                .for_each_entry(|trigger_id, trigger_element| {
                    if implicit_claim.is_none() {
                        implicit_claim = Some((trigger_id.to_owned(), trigger_element.clone()));
                    }
                });
            if let Some((implicit_id, implicit_element)) = implicit_claim {
                updates_active_trigger_id = Some(Some(implicit_id.clone()));
                updates_active_trigger_element = Some(Some(implicit_element));
                *resolved_active_trigger_id.borrow_mut() = Some(implicit_id);
            }
        }

        // Commit the updates (`:498-504`).
        if updates_trigger_count.is_some()
            || updates_active_trigger_id.is_some()
            || updates_active_trigger_element.is_some()
        {
            store.update(move |state, _| {
                if let Some(trigger_count) = updates_trigger_count {
                    state.trigger_count = trigger_count;
                }
                if let Some(active_trigger_id) = updates_active_trigger_id {
                    state.active_trigger_id = active_trigger_id;
                }
                if let Some(active_trigger_element) = updates_active_trigger_element {
                    state.active_trigger_element = active_trigger_element;
                }
                true
            });
        }

        // The deferred close (`:506-528`).
        if close_on_active_trigger_unmount {
            if let Some(lost_id) = lost_active_trigger_id {
                // Defer so a same-tick replacement trigger with the same id can
                // register first (`:508`).
                let store = Rc::clone(&store);
                let set_open = Rc::clone(&set_open);
                queue_microtask(move || {
                    let snapshot = store.get_snapshot();
                    if selectors::open(&snapshot)
                        && snapshot.active_trigger_id.as_deref() == Some(lost_id.as_str())
                        && store.context.trigger_elements.get_by_id(&lost_id).is_none()
                    {
                        let event_details = RootOpenChangeEventDetails::new(
                            crate::floating_ui::reasons::NONE,
                            web_sys::Event::new("base-ui").unwrap(),
                            None,
                            String::new(),
                        );
                        set_open(false, &event_details);
                        // If closing is canceled, keep the previous active trigger
                        // ownership for the still-open popup instead of claiming
                        // another trigger implicitly (`:517-524`).
                        if !event_details.is_canceled() {
                            store.update(|state, _| {
                                state.active_trigger_id = None;
                                state.active_trigger_element = None;
                                true
                            });
                        }
                    }
                });
            }
        }
    });
}

/// Port of `useOpenStateTransitions` (`popupStoreUtils.ts:554-601`): manages the
/// mounted state of the popup — sets up the transition status listeners and handles
/// unmounting when needed, updating the `mounted`, `transitionStatus`, and
/// `preventUnmountingOnClose` states in the store. Must be called inside a reactive
/// owner (a component).
///
/// `open` is the reactive source the transitions follow; `onUnmount` is the optional
/// callback invoked when the popup is force-unmounted; `animate_initial_open` opts a
/// popup that mounts already open into playing its enter transition (the default
/// `false` keeps `defaultOpen`/SSR'd content from animating).
pub struct UseOpenStateTransitions {
    /// A function to forcibly unmount the popup (`:552`, `:600`).
    pub force_unmount: StableCallback<(), ()>,
    /// The current enter/exit transition status (`:600`).
    pub transition_status: RwSignal<Option<crate::floating_ui::types::TransitionStatus>>,
}

pub fn use_open_state_transitions<P: Clone + 'static>(
    open: impl Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    store: &PopupStore<P>,
    on_unmount: Option<Rc<dyn Fn()>>,
    animate_initial_open: bool,
) -> UseOpenStateTransitions {
    // `useTransitionStatus(open, false, false, animateInitialOpen)` (`:560-565`).
    let UseTransitionStatus {
        mounted,
        transition_status,
    } = use_transition_status(
        open.clone(),
        RwSignal::new(false),
        RwSignal::new(false),
        animate_initial_open,
    );

    // `store.useState('preventUnmountingOnClose')` (`:566`).
    let prevent_unmounting_on_close = store.use_state(selectors::prevent_unmounting_on_close);

    // Opening starts a new close cycle. Clear during render so the close-completion
    // hook below reads the synchronized value on the same pass (`:567-569`).
    let synced_prevent_unmounting_on_close = Signal::derive_local({
        let open = open.clone();
        move || {
            if open.get() {
                false
            } else {
                prevent_unmounting_on_close.get()
            }
        }
    });

    // `store.useSyncedValues({ mounted, transitionStatus, preventUnmountingOnClose })`
    // (`:571-575`).
    store.use_synced_value(|state: &mut PopupStoreState<P>| &mut state.mounted, mounted);
    store.use_synced_value(
        |state: &mut PopupStoreState<P>| &mut state.transition_status,
        transition_status,
    );
    store.use_synced_value(
        |state: &mut PopupStoreState<P>| &mut state.prevent_unmounting_on_close,
        synced_prevent_unmounting_on_close,
    );

    // `forceUnmount` (`:577-587`).
    let force_store = Rc::clone(store);
    let force_unmount = use_stable_callback(Some(move |()| {
        mounted.set(false);
        force_store.update(|state, _| {
            state.active_trigger_id = None;
            state.active_trigger_element = None;
            state.mounted = false;
            state.prevent_unmounting_on_close = false;
            true
        });
        if let Some(on_unmount) = &on_unmount {
            on_unmount();
        }
        if let Some(on_open_change_complete) = &force_store.context.on_open_change_complete {
            on_open_change_complete(false);
        }
    }));

    // `useOpenChangeComplete({ enabled: mounted && !open && !syncedPreventUnmountingOnClose, … })`
    // (`:589-598`).
    let popup_ref = Rc::clone(&store.context.popup_ref);
    let force = force_unmount.clone();
    use_open_change_complete(UseOpenChangeCompleteParams {
        enabled: Signal::derive_local({
            let open = open.clone();
            move || mounted.get() && !open.get() && !synced_prevent_unmounting_on_close.get()
        }),
        open: open.clone(),
        reference: move || {
            // The `Cell` slot is read with a take/restore (the labelable ref-slot
            // convention); the context slot carries the `HtmlElement` the popup ref
            // callback receives.
            let taken = popup_ref.take();
            let resolved = taken
                .as_ref()
                .map(|element| element.clone().unchecked_into::<Element>());
            popup_ref.set(taken);
            resolved
        },
        batch: RwSignal::new(false),
        on_complete: Rc::new(move || {
            force.call(());
        }),
    });

    UseOpenStateTransitions {
        force_unmount,
        transition_status,
    }
}

/// Port of `usePopupInteractionProps` (`popupStoreUtils.ts:605-624`): a
/// `useSyncedValues` write-through of the interaction props bags plus an unmount-only
/// effect that resets all three to fresh empty bags (`EMPTY_OBJECT` upstream). Must be
/// called inside a reactive owner (a component).
///
/// Rust adaptations:
/// - The bags are the hook-time values (component bodies run once — upstream's
///   per-render re-sync has no Leptos analog; a family needing reactive re-sync drives
///   it from its own effect).
/// - `EMPTY_OBJECT` is [`HTMLProps::default`] — a fresh empty bag per reset, matching
///   upstream's fresh `{}` object identity.
pub fn use_popup_interaction_props<P: Clone + 'static>(
    store: &PopupStore<P>,
    active_trigger_props: crate::types::HTMLProps,
    inactive_trigger_props: crate::types::HTMLProps,
    popup_props: crate::types::HTMLProps,
) {
    // `store.useSyncedValues(statePart)` (`:612`) — one-shot at hook time.
    store.update(|state, _| {
        state.active_trigger_props = active_trigger_props;
        state.inactive_trigger_props = inactive_trigger_props;
        state.popup_props = popup_props;
        true
    });

    // The unmount-only reset (`:614-623`).
    let reset_store = Rc::clone(store);
    use_iso_layout_effect(move || {
        let store = Rc::clone(&reset_store);
        let cleanup = SendWrapper::new(move || {
            store.update(|state, _| {
                state.active_trigger_props = crate::types::HTMLProps::default();
                state.inactive_trigger_props = crate::types::HTMLProps::default();
                state.popup_props = crate::types::HTMLProps::default();
                true
            });
        });
        on_cleanup(move || (*cleanup)());
    });
}

/// Port of `usePopupRootSync` (`popupStoreUtils.ts:626-645`): two layout effects
/// clearing the component-specific `openMethod` state on close (`:631-635`) and on
/// unmount (`:637-644`). Not covered by any upstream unit test (implementation.md,
/// "Submodules with no test anywhere") — the paths are exercised through component
/// tests. Must be called inside a reactive owner (a component).
pub fn use_popup_root_sync<P: Clone + 'static>(
    store: &PopupStore<P>,
    open: impl Get<Value = bool> + Clone + 'static,
) {
    // The close-clearing effect (`:631-635`).
    {
        let store = Rc::clone(store);
        let open = open.clone();
        use_iso_layout_effect(move || {
            if !open.get() && store.get_snapshot().open_method.is_some() {
                store.set_field(
                    |state: &mut PopupStoreState<P>| &mut state.open_method,
                    None,
                );
            }
        });
    }

    // The unmount-clearing effect (`:637-644`).
    {
        let store = Rc::clone(store);
        use_iso_layout_effect(move || {
            let store = Rc::clone(&store);
            let cleanup = SendWrapper::new(move || {
                if store.get_snapshot().open_method.is_some() {
                    store.set_field(
                        |state: &mut PopupStoreState<P>| &mut state.open_method,
                        None,
                    );
                }
            });
            on_cleanup(move || (*cleanup)());
        });
    }
}

/// The deferred-close scheduler: `window.queueMicrotask` on wasm (the
/// `composite_list.rs` scheduling precedent); the host target runs the work inline —
/// this module's deferred-path tests are browser-only, and host builds only need to
/// compile.
fn queue_microtask(work: impl FnOnce() + 'static) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let function =
                js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js(work));
            window.queue_microtask(&function);
            return;
        }
        work();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        work();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::floating_ui::popup_store::create_initial_popup_store_state;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    // Mirrors `popupStoreUtils.test.tsx:831-839`: opening clears a previous
    // `preventUnmountingOnClose` request and the input state is not mutated.
    #[test]
    fn opening_clears_a_previous_prevent_unmounting_request() {
        let mut state: PopupStoreState<()> =
            create_initial_popup_store_state(&PopupTriggerMap::new(), None, false);
        state.prevent_unmounting_on_close = true;

        let next = create_popup_open_state(&state, true, None, false);

        assert!(next.open);
        assert!(!next.prevent_unmounting_on_close);
        // The input state is not mutated (`:833`).
        assert!(state.prevent_unmounting_on_close);
    }

    // Mirrors `popupStoreUtils.test.tsx:841-847`: closing with the request set keeps
    // the popup mounted.
    #[test]
    fn closing_sets_prevent_unmounting_on_close_when_requested() {
        let state: PopupStoreState<()> =
            create_initial_popup_store_state(&PopupTriggerMap::new(), None, false);

        let next = create_popup_open_state(&state, false, None, true);

        assert!(!next.open);
        assert!(next.prevent_unmounting_on_close);
    }

    // Mirrors `popupStoreUtils.test.tsx:849-859`: closing without a trigger preserves
    // the previous active-trigger ownership (exit animations + focus return).
    #[test]
    fn closing_without_a_trigger_preserves_the_previous_active_trigger() {
        let mut state: PopupStoreState<()> =
            create_initial_popup_store_state(&PopupTriggerMap::new(), None, false);
        state.active_trigger_id = Some("trigger-1".to_owned());

        let next = create_popup_open_state(&state, false, None, false);

        assert_eq!(next.active_trigger_id, Some("trigger-1".to_owned()));
        assert!(
            next.active_trigger_element.is_none(),
            "no element was ever set"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::floating_ui::popup_store::create_initial_popup_store_state;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::reasons;
    use crate::floating_ui::types::OnOpenChangeFn;
    use std::cell::RefCell;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    fn make_store(on_open_change: Option<OnOpenChangeFn>) -> PopupStore<()> {
        let trigger_elements = PopupTriggerMap::new();
        let state = create_initial_popup_store_state(&trigger_elements, None, false);
        Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
            state,
            PopupStoreContext {
                trigger_elements,
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change,
                on_open_change_complete: None,
            },
        ))
    }

    fn details(reason: &str) -> RootOpenChangeEventDetails {
        RootOpenChangeEventDetails::new(
            reason,
            web_sys::Event::new("base-ui").unwrap(),
            None,
            String::new(),
        )
    }

    // Mirrors `popupStoreUtils.test.tsx:949-971`: the change `reason` maps to
    // `instantType` — `triggerFocus` → `'focus'`; `triggerPress`/`escapeKey` closes →
    // `'dismiss'`; `triggerHover` → the key present but `undefined`; `none` → the key
    // absent entirely (the previous value survives).
    #[wasm_bindgen_test]
    fn the_change_reason_maps_to_the_instant_type() {
        let run = |reason: &str, next_open: bool, seeded: Option<InstantType>| {
            let store = make_store(None);
            store.set_field(|state| &mut state.instant_type, seeded);

            let mut event_details = details(reason);
            apply_popup_open_change(
                &store,
                next_open,
                &mut event_details,
                ApplyPopupOpenChangeOptions::default(),
            );

            store.get_snapshot().instant_type
        };

        assert_eq!(
            run(reasons::TRIGGER_FOCUS, true, None),
            Some(InstantType::Focus)
        );
        assert_eq!(
            run(reasons::TRIGGER_PRESS, false, None),
            Some(InstantType::Dismiss)
        );
        assert_eq!(
            run(reasons::ESCAPE_KEY, false, None),
            Some(InstantType::Dismiss)
        );
        // `triggerHover` writes `undefined` — the key is present but clear.
        assert_eq!(
            run(reasons::TRIGGER_HOVER, true, Some(InstantType::Delay)),
            None
        );
        // `none` leaves the key absent entirely — the previous value survives.
        assert_eq!(
            run(reasons::NONE, true, Some(InstantType::Delay)),
            Some(InstantType::Delay)
        );
    }

    // Mirrors `popupStoreUtils.test.tsx:904-919`: the order of operations for a
    // non-canceled change — context `onOpenChange` → `onBeforeDispatch` →
    // `dispatchOpenChange` → store `update` (exactly once).
    #[wasm_bindgen_test]
    fn the_open_change_sequence_runs_in_order() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

        let log_open_change = Rc::clone(&log);
        let store = make_store(Some(Rc::new(
            move |_open: bool, _details: &RootOpenChangeEventDetails| {
                log_open_change.borrow_mut().push("onOpenChange");
            },
        )));

        let log_dispatch = Rc::clone(&log);
        // `dispatchOpenChange` emits through the root context's `'openchange'` bus
        // (FloatingRootStore.ts:106-118), so the third step is observed there.
        store
            .get_snapshot()
            .floating_root_context
            .context
            .events
            .on(
                "openchange",
                Rc::new(
                    move |_: &crate::floating_ui::types::FloatingUIOpenChangeDetails| {
                        log_dispatch.borrow_mut().push("dispatchOpenChange");
                    },
                ),
            );

        let log_update = Rc::clone(&log);
        let updates = Rc::clone(&log);
        store.subscribe(move |_| updates.borrow_mut().push("update"));
        // Subscribe fires once on registration in some store impls — drop the
        // baseline entry so the assertion sees only the open-change update.
        // (The port's `subscribe` never fires on registration; this is a no-op.)

        let log_before = Rc::clone(&log);
        let mut event_details = details(reasons::NONE);
        apply_popup_open_change(
            &store,
            true,
            &mut event_details,
            ApplyPopupOpenChangeOptions {
                on_before_dispatch: Some(Box::new(move || {
                    log_before.borrow_mut().push("onBeforeDispatch");
                })),
                extra_state: None,
            },
        );

        assert_eq!(
            *log.borrow(),
            vec![
                "onOpenChange",
                "onBeforeDispatch",
                "dispatchOpenChange",
                "update"
            ]
        );
        assert_eq!(log_update.borrow().len(), 4, "exactly one store update ran");
        assert!(store.get_snapshot().open);
    }

    // Mirrors `popupStoreUtils.test.tsx:921-934`: a canceled change short-circuits —
    // `onBeforeDispatch`, `dispatchOpenChange`, and the store update never run.
    #[wasm_bindgen_test]
    fn a_canceled_change_short_circuits_the_sequence() {
        let store = make_store(Some(Rc::new(
            |_open: bool, details: &RootOpenChangeEventDetails| {
                details.cancel();
            },
        )));

        let dispatched = Rc::new(Cell::new(false));
        let dispatched_handle = Rc::clone(&dispatched);
        store
            .get_snapshot()
            .floating_root_context
            .context
            .set_on_open_change(Some(Rc::new(move |_, _| dispatched_handle.set(true))));

        let mut event_details = details(reasons::NONE);
        apply_popup_open_change(
            &store,
            true,
            &mut event_details,
            ApplyPopupOpenChangeOptions::default(),
        );

        assert!(!dispatched.get(), "dispatchOpenChange never ran");
        assert!(!store.get_snapshot().open, "the store update never ran");
    }

    // Mirrors `popupStoreUtils.test.tsx:936-947`: `extraState` is merged into the
    // store update, but the requested `open` overrides any `open` in it.
    #[wasm_bindgen_test]
    fn extra_state_merges_with_the_requested_open_winning() {
        let store = make_store(None);

        let mut event_details = details(reasons::NONE);
        apply_popup_open_change(
            &store,
            false,
            &mut event_details,
            ApplyPopupOpenChangeOptions {
                on_before_dispatch: None,
                extra_state: Some(Box::new(|state: &mut PopupStoreState<()>| {
                    // The extra state tries to force the popup open.
                    state.open = true;
                    state.payload = Some(());
                })),
            },
        );

        assert!(!store.get_snapshot().open, "the requested open wins");
        assert_eq!(
            store.get_snapshot().payload,
            Some(()),
            "the rest of the extra state merged"
        );
    }

    // The `preventUnmountOnClose()` path (`popupStoreUtils.ts:266`, `:283`): a request
    // made inside `onOpenChange` is read after it returns and seeds the next state.
    #[wasm_bindgen_test]
    fn a_prevent_unmount_request_made_in_on_open_change_is_honored() {
        let store = make_store(Some(Rc::new(
            |_open: bool, details: &RootOpenChangeEventDetails| {
                details.prevent_unmount_on_close();
            },
        )));

        let mut event_details = details(reasons::NONE);
        apply_popup_open_change(
            &store,
            false,
            &mut event_details,
            ApplyPopupOpenChangeOptions::default(),
        );

        assert!(store.get_snapshot().prevent_unmounting_on_close);
    }

    // The registration flow (`popupStoreUtils.test.tsx:230-276`): a closed-popup
    // registration goes only into the context map — the store is never notified and
    // `triggerCount` stays 0 — and unregistering removes it.
    #[wasm_bindgen_test]
    fn closed_popup_registrations_do_not_notify_the_store() {
        use reactive_graph::owner::Owner;

        let owner = Owner::new();
        owner.set();

        let store = make_store(None);
        let register = use_trigger_registration(Some("trigger-1".to_owned()), &store);

        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap();
        register.call(Some(element.clone()));

        assert_eq!(store.context.trigger_elements.size(), 1);
        assert_eq!(
            store.context.trigger_elements.get_by_id("trigger-1"),
            Some(element.clone())
        );
        assert_eq!(
            store.get_snapshot().trigger_count,
            0,
            "registrations while closed do not sync the count"
        );

        register.call(None);
        assert_eq!(store.context.trigger_elements.size(), 0);

        owner.unset();
    }

    // `syncTriggerCount` (`popupStoreUtils.ts:122-127`): while open, the store's
    // triggerCount tracks the registry; while closed it does not.
    #[wasm_bindgen_test]
    fn sync_trigger_count_only_moves_while_open() {
        let store = make_store(None);
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap();

        store
            .context
            .trigger_elements
            .add("trigger-1", element.clone());
        sync_trigger_count(&store);
        assert_eq!(
            store.get_snapshot().trigger_count,
            0,
            "closed: the count stays 0"
        );

        store.set_field(|state| &mut state.open, true);
        sync_trigger_count(&store);
        assert_eq!(
            store.get_snapshot().trigger_count,
            1,
            "open: the count follows the registry"
        );

        store.context.trigger_elements.delete("trigger-1");
        sync_trigger_count(&store);
        assert_eq!(
            store.get_snapshot().trigger_count,
            0,
            "open: the count follows the registry down"
        );
    }
}
