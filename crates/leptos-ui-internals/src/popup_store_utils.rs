//! Port of the store-level half of `packages/react/src/utils/popups/popupStoreUtils.ts`
//! — the pieces of the popup open-change and trigger-registration machinery that run
//! against the store rather than against React's render cycle. The render-coupled hooks
//! (`usePopupRootStore`, `PopupHandleAttachment`, `useTriggerDataForwarding`,
//! `useImplicitActiveTrigger`, `useOpenStateTransitions`, `usePopupInteractionProps`,
//! `usePopupRootSync`) remain to port; they compose these pieces with the reactive
//! effects.
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

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use web_sys::Element;

use leptos_ui_utils::use_stable_callback::{StableCallback, use_stable_callback};

use crate::create_base_ui_event_details::BaseUIChangeEventDetails;
use crate::floating_ui::constants::FOCUSABLE_ATTRIBUTE;
use crate::floating_ui::popup_store::{InstantType, PopupStoreContext, PopupStoreState, selectors};
use crate::floating_ui::types::RootOpenChangeEventDetails;

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
