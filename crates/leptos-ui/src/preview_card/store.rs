//! The preview-card store — `packages/react/src/preview-card/store/PreviewCardStore.ts`
//! ported over the shared popup machinery (`floating_ui::popup_store`), the
//! popover/store.rs house pattern.
//!
//! Upstream shape (implementation.md, "The store is the state machine"):
//! - `PreviewCardStore extends ReactStore` with state = the shared
//!   `PopupStoreState<Payload>` plus three preview-card additions: `instantType`
//!   (`'dismiss' | 'focus' | undefined`), `adaptiveOrigin`, and `closeDelay`
//!   (initialized to the `CLOSE_DELAY` constant) (`PreviewCardStore.ts:20-24`).
//!   Context adds `inlineRectCoordsRef`, the ref holding hovered-line
//!   coordinates (`:26-28`, `:132-139`). The port carries the shared spine in
//!   the [`PopupStore`], the `instantType` + `closeDelay` members on the
//!   reactive [`PreviewCardExtraState`] slab (the popover `PopoverExtraState`
//!   precedent — the payload slot carries the trigger registration's payload),
//!   and the coords ref on the context (`Rc<Cell<Option<InlineRectCoords>>>`,
//!   the `inline_rect::InlineRectCoordsRef` shape).
//! - Controlled/uncontrolled resolution is done by store selectors
//!   (`openProp ?? open`, `triggerIdProp ?? activeTriggerId`) — the shared
//!   spine's fields; `defaultOpen` only seeds internal `open`, which the
//!   selector shadows whenever `openProp` is defined (implementation.md
//!   "Store-centric state model").
//! - The single open/close mutation is `store.setOpen(nextOpen, eventDetails)`
//!   → the shared `applyPopupOpenChange` sequence: notify `onOpenChange` →
//!   honor `cancel()` → run `onBeforeDispatch` → dispatch to the floating root
//!   → commit state (`popupStoreUtils.ts:241-308`; the cancel gate at `:270-272`
//!   is what implements `eventDetails.cancel()`). The preview-card-specific
//!   `onBeforeDispatch` captures hovered inline-rect coordinates when the
//!   reason is `triggerHover` and the event carries pointer coordinates, but
//!   only when the trigger changed (`inlineRectCoordsRef.current?.element !==
//!   eventDetails.trigger`) — the "card stays anchored to the originally
//!   opened line" rule (`PreviewCardStore.ts:75-96`). For hover reasons the
//!   state commit is wrapped in `ReactDOM.flushSync` upstream (the wasm port's
//!   synchronous store update is the flushSync-equivalent — the update lands
//!   before `setOpen` returns). The same sequence maps the change reason onto
//!   `instantType`: focus-open → `'focus'`, press/Escape close → `'dismiss'`,
//!   hover → `undefined` (`popupStoreUtils.ts:291-297`).
//! - `createNullPreviewCardStore` (`PreviewCardStore.ts:105-113`): the inert
//!   detached fallback — the port reuses the shared no-writer store the dialog
//!   `create_handle` builds (the popover/store.rs `createNullPopoverStore`
//!   precedent).
//! - `closeDelay` seeds the `CLOSE_DELAY` constant (600/300 per
//!   `packages/react/src/preview-card/utils/constants.ts:1-2`) and is
//!   overwritten by each registering trigger's resolved close delay (the
//!   `useTriggerDataForwarding` write, `popupStoreUtils.ts:376-385`).

use std::cell::Cell;
use std::rc::Rc;

use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update as _};

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::popup_store::{self, InstantType, PopupStoreState};
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::inline_rect::{InlineRectCoords, InlineRectCoordsRef};
use leptos_ui_internals::popup_store_utils::{PopupStore, attach_prevent_unmount_on_close};

/// The upstream `OPEN_DELAY` constant (`utils/constants.ts:1`) — the trigger
/// `delay` default.
pub const OPEN_DELAY: u32 = 600;
/// The upstream `CLOSE_DELAY` constant (`utils/constants.ts:2`) — the trigger
/// `closeDelay` default and the store slab's seed.
pub const CLOSE_DELAY: u32 = 300;

/// The event details type — `PreviewCardRoot.ChangeEventDetails`. The shared
/// machinery is typed over the floating layer's `BaseUIChangeEventDetails` (the
/// popover `PopoverChangeEventDetails` precedent).
pub type PreviewCardChangeEventDetails = RootOpenChangeEventDetails;

/// `onOpenChange` (`PreviewCardRoot.tsx:36-38`): `(open, eventDetails)`.
pub type OnOpenChange = Rc<dyn Fn(bool, &RootOpenChangeEventDetails)>;
/// `onOpenChangeComplete` (`PreviewCardRoot.tsx:39`): `(open)`.
pub type OnOpenChangeComplete = Rc<dyn Fn(bool)>;

// ---------------------------------------------------------------------------
// Preview-card-specific store state (`PreviewCardStore.ts:20-24`)
// ---------------------------------------------------------------------------

/// The preview-card-specific members layered over the shared
/// [`PopupStoreState`] — `PreviewCardStore.ts:20-24` (the shared fields live on
/// the spine; `adaptiveOrigin` is the Viewport-only positioning middleware the
/// port's deferred pass notes on the viewport part).
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewCardExtraState {
    /// `instantType` (`:21`): `'dismiss' | 'focus' | undefined` — mapped from
    /// the change reason by the open-change sequence (focus-open → focus,
    /// press/Escape close → dismiss, hover → undefined). The shared
    /// [`InstantType`] enum merges the popup unions; the port stores the
    /// shared enum and emits only the preview-card subset.
    pub instant_type: Option<InstantType>,
    /// `closeDelay` (`:23`) — seeded to [`CLOSE_DELAY`], overwritten by the
    /// active trigger's resolved close delay at registration.
    pub close_delay: u32,
}

impl Default for PreviewCardExtraState {
    /// `createInitialState`'s preview-card defaults (`PreviewCardStore.ts`
    /// `createInitialPreviewCardStoreState` — `closeDelay` seeds the constant).
    fn default() -> Self {
        PreviewCardExtraState {
            instant_type: None,
            close_delay: CLOSE_DELAY,
        }
    }
}

/// The preview-card root context — upstream's `PreviewCardRootContext` value is
/// the store itself (`PreviewCardContext.ts:5-9`); the port's context carries
/// the shared store handle, the reactive extra slab, and the context ref slot
/// `inlineRectCoordsRef` (`PreviewCardStore.ts:26-28` — the ref holding
/// hovered-line coordinates; the `PopoverRootContext` ref-slot precedent).
#[derive(Clone)]
pub struct PreviewCardRootContext {
    /// The store — the context value upstream (`PreviewCardContext.ts:5-9`).
    pub store: PopupStore<()>,
    /// The preview-card-specific reactive slab.
    pub extra: RgRwSignal<PreviewCardExtraState, reactive_graph::owner::LocalStorage>,
    /// `inlineRectCoordsRef` (`PreviewCardStore.ts:26-28`) — the hovered-line
    /// coordinate capture the open-change hook and the trigger props write
    /// through and the inline middleware reads.
    pub inline_rect_coords: InlineRectCoordsRef,
    /// `popupRef` — the popup element slot the transition listeners read
    /// (the popover context's `popupRef` precedent; `PopupStore.ts:41` shape).
    pub popup_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
}

impl PreviewCardRootContext {
    /// `usePreviewCardRootContext()` — the required form (`PreviewCardContext.ts:13-22`):
    /// panics when used outside a Root, upstream's throw with the exact message
    /// behavior.md "Public API surface" records.
    pub fn expect() -> Self {
        reactive_graph::owner::use_context::<SharedPreviewCardRootContext>()
            .expect(
                "Base UI: PreviewCardRootContext is missing. PreviewCard parts must be placed within <PreviewCard.Root>.",
            )
            .take()
    }

    /// `usePreviewCardRootContext(true)` — the optional form (`:13`): `None`
    /// outside a Root (the Trigger and the Root's own nesting detection read
    /// this form).
    pub fn optional() -> Option<Self> {
        reactive_graph::owner::use_context::<SharedPreviewCardRootContext>()
            .map(|shared| shared.take())
    }

    /// Reads a derived signal over the extra slab (the popover `extra_signal`
    /// helper).
    pub fn extra_signal<T: Clone + PartialEq + 'static>(
        &self,
        selector: impl Fn(&PreviewCardExtraState) -> T + 'static,
    ) -> reactive_graph::wrappers::read::Signal<T, reactive_graph::owner::LocalStorage> {
        let extra = self.extra;
        reactive_graph::wrappers::read::Signal::derive_local(move || selector(&extra.get()))
    }
}

/// The context type provided through the reactive owner — the `SendWrapper`
/// bridge `provide_context`'s `Send + Sync` contract requires (the popover
/// precedent).
pub type SharedPreviewCardRootContext = send_wrapper::SendWrapper<PreviewCardRootContext>;

// ---------------------------------------------------------------------------
// createInitialState (`PreviewCardStore.ts` createInitialPreviewCardStoreState)
// ---------------------------------------------------------------------------

/// The initial state seed — the shared spine via
/// [`popup_store::create_initial_popup_store_state`] plus the preview-card
/// family defaults, with the `open: true` ⇒ `mounted: true` overlay so
/// `defaultOpen` renders on first paint (behavior.md "State model":
/// `defaultOpen: true` renders the card open on mount).
///
/// `floating_id`/`nested` resolve in `usePopupRootStore` (`PreviewCardRoot.tsx:36-48`)
/// — the caller passes them through.
pub fn create_initial_preview_card_store_state(
    floating_id: Option<String>,
    nested: bool,
) -> PopupStoreState<()> {
    let trigger_elements = leptos_ui_internals::floating_ui::PopupTriggerMap::new();
    let state =
        popup_store::create_initial_popup_store_state(&trigger_elements, floating_id, nested);
    // The `defaultOpen` seed (the popover store.rs precedent, `:211-215` shape):
    // the root wires `open`/`mounted` into the spine after creation.
    state
}

// ---------------------------------------------------------------------------
// setOpen — the open-change pipeline (applyPopupOpenChange, popupStoreUtils.ts:241-308)
// ---------------------------------------------------------------------------

/// The single mutation entry every preview-card interaction funnels into —
/// `store.setOpen` over the shared `applyPopupOpenChange` sequence
/// (`popupStoreUtils.ts:241-308`).
///
/// Sequence (each step cited):
/// 1. `attachPreventUnmountOnClose` (`:190-231` — the `preventUnmountOnClose()`
///    details attachment behavior.md "Events" records).
/// 2. The preview-card `onBeforeDispatch` (`PreviewCardStore.ts:75-96`): on a
///    hover open with pointer coordinates on the event and a changed trigger,
///    capture the hovered-line coordinates into the context ref — the "card
///    stays anchored to the originally opened line" rule.
/// 3. `context.onOpenChange` (`:262-268`).
/// 4. The cancel gate (`:270-272`).
/// 5. The floating-root dispatch + the `createPopupOpenState` commit
///    (`:274-290`) — for hover reasons upstream wraps the commit in
///    `ReactDOM.flushSync` (`:302-307`); the port's synchronous update is the
///    equivalent (the store update lands before `set_open` returns).
/// 6. The `instantType` mapping (`:291-297`): focus-open → `'focus'`,
///    press/Escape close → `'dismiss'`, hover → `undefined`.
pub fn preview_card_set_open(
    store: &PopupStore<()>,
    next_open: bool,
    event_details: &mut RootOpenChangeEventDetails,
    inline_rect_coords: Option<&InlineRectCoordsRef>,
) {
    let is_hover = event_details.reason == reasons::TRIGGER_HOVER;

    // `attachPreventUnmountOnClose(eventDetails)` (`:190-231`).
    let prevent_unmount_requested = attach_prevent_unmount_on_close(event_details);

    // The preview-card `onBeforeDispatch` (`PreviewCardStore.ts:75-96`): hover
    // opens capture the hovered-line coordinates when the trigger changed. The
    // pointer coordinates ride the native MouseEvent; the host target has no
    // JS runtime (the popover store.rs `is_mouse_event` split), so the capture
    // is wasm-only.
    #[cfg(target_arch = "wasm32")]
    if is_hover && next_open {
        use wasm_bindgen::JsCast as _;
        if let (Some(coords_ref), Some(mouse_event)) = (
            inline_rect_coords,
            event_details.event.dyn_ref::<web_sys::MouseEvent>(),
        ) {
            let trigger_changed = {
                // `Cell::get` needs `Copy`; the take-and-restore is the
                // non-Copy read (the coords are not `Clone`-cheap, one clone).
                let current = coords_ref.take();
                coords_ref.set(current.clone());
                // `inlineRectCoordsRef.current?.element !== eventDetails.trigger`
                // (`PreviewCardStore.ts:75-96`) — the Option is the identity:
                // a None trigger never equals a captured coords' element.
                !current
                    .map(|coords| Some(&coords.element) == event_details.trigger.as_ref())
                    .unwrap_or(true)
            };
            if trigger_changed {
                let trigger = event_details
                    .trigger
                    .clone()
                    .or_else(|| store.get_snapshot().active_trigger_element.clone());
                if let Some(trigger) = trigger {
                    coords_ref.set(Some(InlineRectCoords {
                        x: mouse_event.client_x() as f64,
                        y: mouse_event.client_y() as f64,
                        line_index: None,
                        element: trigger,
                    }));
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = inline_rect_coords;

    // `this.context.onOpenChange?.(...)` (`:262-268`).
    if let Some(on_open_change) = &store.context.on_open_change {
        on_open_change(next_open, event_details);
    }

    // The cancel gate (`:270-272`).
    if event_details.is_canceled() {
        return;
    }

    // The floating-root dispatch (`:274`).
    store
        .get_snapshot()
        .floating_root_context
        .dispatch_open_change(next_open, event_details);

    // The `createPopupOpenState` commit (`:274-290`) — the flushSync-wrapped
    // arm for hover reasons is the port's synchronous update.
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

    // The `instantType` mapping (`:291-297`): focus-open → `'focus'`, press or
    // Escape close → `'dismiss'`, hover → `undefined`.
    let is_press_close = !next_open
        && (event_details.reason == reasons::TRIGGER_PRESS
            || event_details.reason == reasons::ESCAPE_KEY);
    let instant_type = if event_details.reason == reasons::TRIGGER_FOCUS {
        Some(InstantType::Focus)
    } else if is_press_close {
        Some(InstantType::Dismiss)
    } else {
        None
    };
    store.update(|state, _| {
        state.instant_type = instant_type;
        true
    });
}

/// The shared details constructor — `createChangeEventDetails(reason, nativeEvent)`
/// (`PreviewCardRoot.tsx:20-22` shape; the popover store.rs precedent).
pub fn preview_card_change_event_details(
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

thread_local! {
    /// The per-root extra-slab slot the store pipeline writes through — the
    /// upstream store method's instance writes route through the root-wired
    /// slot (the popover `EXTRA_SLAB_SLOT` precedent).
    static EXTRA_SLAB_SLOT: std::cell::RefCell<Option<RgRwSignal<PreviewCardExtraState, reactive_graph::owner::LocalStorage>>> =
        const { std::cell::RefCell::new(None) };
}

/// Installs the per-root slab slot the open pipeline writes through — called by
/// the root (the popover `wire_popover_slots` precedent).
pub fn wire_preview_card_slots(
    extra: RgRwSignal<PreviewCardExtraState, reactive_graph::owner::LocalStorage>,
) {
    EXTRA_SLAB_SLOT.with(|slot| *slot.borrow_mut() = Some(extra));
}

/// The extra-slab slot read (the internal write path).
pub(crate) fn extra_slab_slot()
-> Option<RgRwSignal<PreviewCardExtraState, reactive_graph::owner::LocalStorage>> {
    EXTRA_SLAB_SLOT.with(|slot| slot.borrow().clone())
}
