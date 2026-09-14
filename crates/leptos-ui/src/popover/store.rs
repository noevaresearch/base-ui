//! The popover store — `packages/react/src/popover/store/PopoverStore.ts` ported
//! over the shared popup machinery (`floating_ui::popup_store`), the dialog/store.rs
//! house pattern.
//!
//! Upstream shape (implementation.md, "Store-centric state model"):
//! - `PopoverStore extends ReactStore<Readonly<State<Payload>>, Context, Selectors>`
//!   (`PopoverStore.ts:77`) where `State = PopupStoreState & { disabled, instantType,
//!   modal, focusManagerModal, openMethod, openChangeReason, stickIfOpen,
//!   titleElementId, descriptionElementId, openOnHover, closeDelay, adaptiveOrigin }`
//!   (`:18-31`). The port carries the shared spine in the [`PopupStore`] and the
//!   popover-only members on a reactive [`PopoverExtraState`] slab (the
//!   `MenuExtraState`/`DialogExtraState` precedent — the payload slot carries the
//!   trigger registration's payload, so the extra slab is where the family fields go).
//! - `setOpen(nextOpen, eventDetails)` (`PopoverStore.ts:95-168`) is the single
//!   mutation entry every interaction funnels into: the classification
//!   (`:99-104`), the `preventUnmountOnClose` attachment (`:106-108`), the
//!   triggerless `closePress` trigger backfill (`:112-122`), the `onOpenChange`
//!   callback + cancel gate (`:124-128`), the floating-root dispatch (`:130`), the
//!   `createPopupOpenState` commit (`:130-155`), the hover `stickIfOpen`
//!   patient-click arming with the 500 ms [`PATIENT_CLICK_THRESHOLD`] window
//!   (`:146-157`), and the `instantType` mapping (`:159-167`).
//! - `createInitialState` (`:189-217`) seeds the family defaults — including the
//!   `open: true` ⇒ `mounted: true` overlay so `defaultOpen` renders on first paint.
//! - `createNullPopoverStore` (`:171-187`): the inert detached fallback — the port
//!   reuses the shared no-writer store the dialog `create_handle` builds
//!   (dialog/mod.rs `create_handle`, the `createNullDialogStore` precedent).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update as _};
use wasm_bindgen::JsCast;

use leptos_ui_internals::constants::PATIENT_CLICK_THRESHOLD;
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::popup_store::{self, InstantType, PopupStoreState};
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::popup_store_utils::{PopupStore, attach_prevent_unmount_on_close};
use leptos_ui_utils::use_timeout::Timeout;

/// The event details type — `PopoverRoot.ChangeEventDetails`. The shared machinery is
/// typed over the floating layer's `BaseUIChangeEventDetails` (the
/// `MenuChangeEventDetails` precedent: the custom payload stays empty at the floating
/// layer).
pub type PopoverChangeEventDetails = RootOpenChangeEventDetails;

/// `onOpenChange` (`PopoverRoot.tsx:36-38`): `(open, eventDetails)`.
pub type OnOpenChange = Rc<dyn Fn(bool, &RootOpenChangeEventDetails)>;
/// `onOpenChangeComplete` (`PopoverRoot.tsx:39`): `(open)`.
pub type OnOpenChangeComplete = Rc<dyn Fn(bool)>;

/// The popover `instantType` union (`PopoverStore.ts:26`): the shared popup values
/// plus `'click'` and `'trigger-change'`, minus `'delay'`. The shared
/// [`InstantType`] enum merges the unions (`popup_store.rs` module docs), so the
/// port stores the shared enum and simply emits only the popover subset — the
/// `MenuInstantType` exhaustiveness wrapper has no popover-exclusive variant to
/// carry.

// ---------------------------------------------------------------------------
// Popover-specific store state (`PopoverStore.ts:18-31`)
// ---------------------------------------------------------------------------

/// The popover-specific members layered over the shared [`PopupStoreState`] —
/// `PopoverStore.ts:18-31` (the `PopupStoreState<Payload> & {...}` intersection;
/// the shared fields live on the spine, the popover fields ride the extra slab).
#[derive(Clone, Debug, PartialEq)]
pub struct PopoverExtraState {
    /// `disabled` (`:19`) — the active trigger's disabled state, forwarded by the
    /// trigger's registration; the popup reads it to gate hover-close.
    pub disabled: bool,
    /// `modal: boolean | 'trap-focus'` (`:22`) — the port carries the boolean; the
    /// `'trap-focus'` arm's only observable delta (the `sloppy` mouse
    /// outside-press policy, `PopoverRoot.tsx:236-243`) is carried as the separate
    /// [`PopoverExtraState::trap_focus`] flag the interactions gate reads.
    pub modal: bool,
    /// The `'trap-focus'` modal arm (`:22`).
    pub trap_focus: bool,
    /// `focusManagerModal` (`:23`) — synced by the popup (`PopoverPopup.tsx:71-72`).
    pub focus_manager_modal: bool,
    /// `openChangeReason` (`:28`) — the current open-change reason.
    pub open_change_reason: Option<String>,
    /// `stickIfOpen` (`:29`) — the patient-click window state.
    pub stick_if_open: bool,
    /// `titleElementId` (`:30`) — synced by the title part (`PopoverTitle.tsx:24`).
    pub title_element_id: Option<String>,
    /// `descriptionElementId` (`:31`) — synced by the description part
    /// (`PopoverDescription.tsx:24`).
    pub description_element_id: Option<String>,
    /// `openOnHover` (`:32`) — the active trigger's openOnHover, forwarded by the
    /// trigger's registration.
    pub open_on_hover: bool,
    /// `closeDelay` (`:33`) — the active trigger's closeDelay, forwarded by the
    /// trigger's registration.
    pub close_delay: u32,
}

impl Default for PopoverExtraState {
    /// `createInitialState`'s popover defaults (`PopoverStore.ts:196-206`).
    fn default() -> Self {
        PopoverExtraState {
            disabled: false,
            modal: false,
            trap_focus: false,
            focus_manager_modal: false,
            open_change_reason: None,
            stick_if_open: true,
            title_element_id: None,
            description_element_id: None,
            open_on_hover: false,
            close_delay: 0,
        }
    }
}

/// The popover root context — upstream's `PopoverRootContext` value is the store
/// itself (`PopoverRootContext.ts:5`); the port's context carries the shared store
/// handle, the reactive extra slab, and the context ref slots (`PopoverStore.ts:39-44`
/// — `popupRef`, `triggerFocusTargetRef`, `beforeContentFocusGuardRef`,
/// `stickIfOpenTimeout`; the `DialogRootContext` precedent).
#[derive(Clone)]
pub struct PopoverRootContext {
    /// The store — the context value upstream (`PopoverRootContext.ts:5`).
    pub store: PopupStore<()>,
    /// The popover-specific reactive slab.
    pub extra: RgRwSignal<PopoverExtraState, reactive_graph::owner::LocalStorage>,
    /// `popupRef` (`PopoverStore.ts:41`).
    pub popup_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
    /// `triggerFocusTargetRef` (`:42`).
    pub trigger_focus_target_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
    /// `beforeContentFocusGuardRef` (`:43`).
    pub before_content_focus_guard_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
    /// `stickIfOpenTimeout` (`:44`) — the patient-click window timer.
    pub stick_if_open_timeout: Rc<Timeout>,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the dialog precedent).
pub type SharedPopoverRootContext = send_wrapper::SendWrapper<PopoverRootContext>;

impl PopoverRootContext {
    /// `usePopoverRootContext()` — the required form (`PopoverRootContext.ts:9-18`):
    /// panics when used outside a Root, upstream's throw.
    pub fn expect() -> Self {
        reactive_graph::owner::use_context::<SharedPopoverRootContext>()
            .expect(
                "Base UI: PopoverRootContext is missing. Popover parts must be placed within <Popover.Root>.",
            )
            .take()
    }

    /// `usePopoverRootContext(true)` — the optional form (`:9`): `None` outside a Root.
    pub fn optional() -> Option<Self> {
        reactive_graph::owner::use_context::<SharedPopoverRootContext>().map(|shared| shared.take())
    }

    /// Reads a derived signal over the extra slab (the dialog `extra_signal` helper).
    pub fn extra_signal<T: Clone + PartialEq + 'static>(
        &self,
        selector: impl Fn(&PopoverExtraState) -> T + 'static,
    ) -> reactive_graph::wrappers::read::Signal<T, reactive_graph::owner::LocalStorage> {
        let extra = self.extra;
        reactive_graph::wrappers::read::Signal::derive_local(move || selector(&extra.get()))
    }
}

// ---------------------------------------------------------------------------
// createInitialState (`PopoverStore.ts:189-217`)
// ---------------------------------------------------------------------------

/// The initial state seed — the shared spine via
/// [`popup_store::create_initial_popup_store_state`] plus the popover family
/// defaults, with the `open: true` ⇒ `mounted: true` overlay (`:211-215`) so
/// `defaultOpen` renders on first paint (behavior.md, `defaultOpen`).
///
/// `floating_id`/`nested` resolve in `usePopupRootStore` (`PopoverRoot.tsx:40`,
/// factory `:112-127`) — the caller passes them through.
pub fn create_initial_popover_store_state(
    floating_id: Option<String>,
    nested: bool,
) -> PopupStoreState<()> {
    let trigger_elements = leptos_ui_internals::floating_ui::PopupTriggerMap::new();
    let state =
        popup_store::create_initial_popup_store_state(&trigger_elements, floating_id, nested);
    // `if (state.open && initialState?.mounted === undefined) { state.mounted = true; }`
    // (`:211-215`): the `defaultOpen` seed. The port's root wires `open`/`mounted`
    // into the spine after creation (see `use_popover_root_store`).
    state
}

// ---------------------------------------------------------------------------
// setOpen — the open-change pipeline (`PopoverStore.ts:95-168`)
// ---------------------------------------------------------------------------

/// The single mutation entry every popover interaction funnels into —
/// `PopoverStore.setOpen` (`PopoverStore.ts:95-168`).
///
/// Sequence (each step cited to the upstream line):
/// 1. Classification (`:99-104`): hover (`triggerHover`), keyboard click
///    (`triggerPress` with `event.detail === 0`), dismiss-close (`escapeKey` or
///    reason `null`).
/// 2. `attachPreventUnmountOnClose` (`:106-108`).
/// 3. Triggerless `closePress` trigger backfill (`:112-122`): registered active
///    trigger → active trigger element → undefined.
/// 4. `context.onOpenChange` + the cancel gate (`:124-128`).
/// 5. The floating-root dispatch (`:130`).
/// 6. `createPopupOpenState` commit (`:134-152`) with `openChangeReason` written.
/// 7. Hover opens/closes arm the patient-click window (`:146-157`):
///    `stickIfOpen: true` + the 500 ms [`PATIENT_CLICK_THRESHOLD`] reset.
/// 8. `instantType` mapping (`:159-167`): keyboard click → `'click'`, dismiss-close
///    → `'dismiss'`, `focusOut` → `'focus'`.
pub fn popover_set_open(
    store: &PopupStore<()>,
    next_open: bool,
    event_details: &mut RootOpenChangeEventDetails,
) {
    let is_hover = event_details.reason == reasons::TRIGGER_HOVER;
    // `(eventDetails.event as MouseEvent).detail === 0` (`:100-103`) — keyboard
    // clicks dispatch a MouseEvent with `detail` 0. The instanceof check needs a
    // JS runtime: `dyn_ref`-ing on the host target panics (no JS runtime, so no
    // event can be a MouseEvent — the menu store's `is_mouse_event` split,
    // menu/store.rs `:313-321`).
    #[cfg(target_arch = "wasm32")]
    let is_keyboard_click = event_details.reason == reasons::TRIGGER_PRESS
        && event_details
            .event
            .dyn_ref::<web_sys::MouseEvent>()
            .map(|mouse_event| mouse_event.detail() == 0)
            .unwrap_or(false);
    #[cfg(not(target_arch = "wasm32"))]
    let is_keyboard_click = false;
    let is_dismiss_close = !next_open
        && (event_details.reason == reasons::ESCAPE_KEY || event_details.reason.is_empty());

    // `attachPreventUnmountOnClose(eventDetails)` (`:106-108`).
    let prevent_unmount_requested = attach_prevent_unmount_on_close(event_details);

    // The triggerless `closePress` backfill (`:112-122`).
    if !next_open && event_details.reason == reasons::CLOSE_PRESS && event_details.trigger.is_none()
    {
        let snapshot = store.get_snapshot();
        let active_trigger_id = snapshot
            .trigger_id_prop
            .clone()
            .or_else(|| snapshot.active_trigger_id.clone());
        if let Some(active_trigger_id) = active_trigger_id {
            event_details.trigger = snapshot
                .floating_root_context
                .context
                .trigger_elements
                .get_by_id(&active_trigger_id)
                .or_else(|| snapshot.active_trigger_element.clone());
        }
    }

    // `this.context.onOpenChange?.(...)` (`:124`).
    if let Some(on_open_change) = &store.context.on_open_change {
        on_open_change(next_open, event_details);
    }

    // The cancel gate (`:126-128`).
    if event_details.is_canceled() {
        return;
    }

    // The floating-root dispatch (`:130`).
    store
        .get_snapshot()
        .floating_root_context
        .dispatch_open_change(next_open, event_details);

    // `createPopupOpenState` + the `openChangeReason` write (`:134-152`).
    let (open, prevent_unmounting_on_close, active_trigger_id, active_trigger_element) = {
        let snapshot = store.get_snapshot();
        let popup_open_state = leptos_ui_internals::popup_store_utils::create_popup_open_state(
            &snapshot,
            next_open,
            event_details.trigger.as_ref(),
            prevent_unmount_requested.get(),
        );
        (
            popup_open_state.open,
            popup_open_state.prevent_unmounting_on_close,
            popup_open_state.active_trigger_id,
            popup_open_state.active_trigger_element,
        )
    };
    store.update(|state, _| {
        state.open = open;
        state.prevent_unmounting_on_close = prevent_unmounting_on_close;
        state.active_trigger_id = active_trigger_id.clone();
        state.active_trigger_element = active_trigger_element.clone();
        true
    });

    // The patient-click window (`:146-157`): only hover changes arm it; the
    // `stickIfOpen` write and the timeout are store context — the port carries the
    // timeout on the root context and the flag on the extra slab.
    if is_hover {
        if let Some(stick_if_open) = extra_slab_slot() {
            stick_if_open.update(|state| state.stick_if_open = true);
        }
        if let Some(timeout_slot) = stick_timeout_slot() {
            let slab = extra_slab_slot();
            let store = Rc::clone(store);
            timeout_slot.start(PATIENT_CLICK_THRESHOLD, move || {
                if let Some(stick_if_open) = slab {
                    stick_if_open.update(|state| state.stick_if_open = false);
                }
                let _ = &store;
            });
        }
    }

    // `instantType` (`:159-167`).
    let instant_type = if is_keyboard_click {
        Some(InstantType::Click)
    } else if is_dismiss_close {
        Some(InstantType::Dismiss)
    } else if event_details.reason == reasons::FOCUS_OUT {
        Some(InstantType::Focus)
    } else {
        None
    };
    store.update(|state, _| {
        state.instant_type = instant_type;
        true
    });
    // `openChangeReason` rides the extra slab (`:139`).
    if let Some(slab) = extra_slab_slot() {
        slab.update(|state| state.open_change_reason = Some(event_details.reason.clone()));
    }
}

thread_local! {
    /// The per-root extra-slab slot `popover_set_open` writes through — the upstream
    /// store method's `this.set('stickIfOpen', …)` is an instance write; the port's
    /// free function routes it through the root-wired slot (the dialog
    /// `STORE_ACTIONS_SLOT` thread-local precedent for imperative handles).
    static EXTRA_SLAB_SLOT: RefCell<Option<RgRwSignal<PopoverExtraState, reactive_graph::owner::LocalStorage>>> =
        const { RefCell::new(None) };
    /// The per-root patient-click timeout slot (`createInitialContext`, `:44`).
    static STICK_TIMEOUT_SLOT: RefCell<Option<Rc<Timeout>>> = const { RefCell::new(None) };
}

/// Installs the per-root slots the [`popover_set_open`] pipeline writes through —
/// called by the root with its slab and timeout (the imperative-handle wiring).
pub fn wire_popover_slots(
    extra: RgRwSignal<PopoverExtraState, reactive_graph::owner::LocalStorage>,
    timeout: Rc<Timeout>,
) {
    EXTRA_SLAB_SLOT.with(|slot| *slot.borrow_mut() = Some(extra));
    STICK_TIMEOUT_SLOT.with(|slot| *slot.borrow_mut() = Some(timeout));
}

/// The extra-slab slot read (the internal write path).
fn extra_slab_slot() -> Option<RgRwSignal<PopoverExtraState, reactive_graph::owner::LocalStorage>> {
    EXTRA_SLAB_SLOT.with(|slot| slot.borrow().clone())
}

/// The stick-timeout slot read (the internal write path).
fn stick_timeout_slot() -> Option<Rc<Timeout>> {
    STICK_TIMEOUT_SLOT.with(|slot| slot.borrow().clone())
}

/// The shared details constructor — `createChangeEventDetails(reason, nativeEvent)`
/// (`PopoverRoot.tsx:20-22`).
pub fn popover_change_event_details(
    reason: &str,
    event: Option<web_sys::Event>,
) -> RootOpenChangeEventDetails {
    RootOpenChangeEventDetails::new(
        reason.to_owned(),
        event.unwrap_or_else(|| {
            web_sys::Event::new("base-ui").expect("the Event constructor is available")
        }),
        None,
        String::new(),
    )
}
