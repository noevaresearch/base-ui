//! Port of the Base UI Dialog — the `library: dialog` TODO item
//! (`specs/library/dialog/behavior.md`, `specs/library/dialog/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **The unit is one shared renderer parameterized by mode** (`dialog` | `drawer` |
//!   `alert-dialog`): `useRenderDialogRoot` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:17-106`)
//!   is the entire root implementation, and the mode constant drives only
//!   `modal` / `disablePointerDismissal` / `role` (`:35-39`). The port exposes the same
//!   mode-parameterized root over [`DialogRootMode`], so `library: alert-dialog` is a
//!   configuration of this unit, not a separate implementation (alert-dialog's
//!   implementation.md, "Port-relevant summary").
//! - **The store is the single source of truth**: `DialogStore` (`DialogStore.ts:58-98`)
//!   extends the shared popup store with the dialog-specific state fields
//!   (`modal`, `disablePointerDismissal`, `nested`, `nestedOpenDialogCount`,
//!   `nestedOpenDrawerCount`, `titleElementId`, `descriptionElementId`, `role`) and the
//!   `setOpen` method whose sequence is: attach `preventUnmountOnClose` → backfill the
//!   closing trigger → notify `onOpenChange` → honor `isCanceled` → dispatch the
//!   floating-root change → commit `createPopupOpenState` (`:74-97`). The port keeps
//!   the shared popup store (`leptos_ui_internals::popup_store_utils::PopupStore`) as
//!   the storage layer and extends it exactly the way upstream extends
//!   `PopupStoreState`: the dialog fields live in a reactive slab ([`DialogExtraState`])
//!   carried on the context, so every dialog part reads one store handle.
//! - **The open-state commit** ([`dialog_set_open`]) is `DialogStore.setOpen` verbatim,
//!   composed over the shared popup machinery's `create_popup_open_state` and the
//!   `attach_prevent_unmount_on_close` reader (`popupStoreUtils.ts:223-231`).
//! - **The trigger** (`DialogTrigger.tsx:22-102`) is store resolution (handle's store
//!   wins over the context store, `:38-45`) + `useTriggerDataForwarding` + `useButton` +
//!   `useClick` + `useOpenMethodTriggerProps` + the static ARIA block
//!   (`aria-haspopup: 'dialog'`, `aria-expanded`, `aria-controls`, `:90-97`) + the
//!   `CLICK_TRIGGER_IDENTIFIER` marker + `triggerOpenStateMapping`.
//! - **The popup** (`DialogPopup.tsx:22-111`) renders the popup element with the
//!   store-provided `popupProps`, the `FOCUSABLE_POPUP_PROPS` attributes, the id/ARIA
//!   labels (`aria-labelledby` → title id, `aria-describedby` → description id), the
//!   mode role, `hidden: !mounted`, the COMPOSITE_KEYS stopPropagation, and the
//!   `--nested-dialogs` CSS var; the FloatingFocusManager wraps the element with
//!   `initialFocus` defaulting to the popup/touch rule, `restoreFocus: 'popup'`,
//!   `modal !== false`, and `closeOnFocusOut: !disablePointerDismissal`.
//! - **The portal** (`DialogPortal.tsx:17-43`) renders the InternalBackdrop while
//!   `mounted && modal === true` with `inert={!open}`, under a portal context carrying
//!   `keepMounted`, gating the whole subtree on `mounted || keepMounted`.
//! - **Title/Description** register their generated ids into the store
//!   (`useSyncedValueWithCleanup('titleElementId'/'descriptionElementId')`,
//!   `DialogTitle.tsx:24`, `DialogDescription.tsx:24`) — the mechanism behind
//!   `aria-labelledby`/`aria-describedby` (behavior.md "Accessibility").
//! - **Close** (`DialogClose.tsx:39-43`) closes with `REASONS.closePress` only when
//!   open.
//! - **Viewport** (`DialogViewport.tsx:16-61`) registers the viewport element, renders
//!   `role="presentation"`, `hidden: !mounted`, `pointerEvents: !open ? 'none'`,
//!   gated on the portal's `keepMounted || mounted`.
//! - **Backdrop** (`DialogBackdrop.tsx:15-50`) registers `backdropRef`, renders
//!   `role="presentation"`, `hidden: !mounted`, the user-select style pair, with
//!   `popupTransitionStateMapping` and the `forceRender || !nested` enable gate.
//! - **The handle** (`DialogHandle.ts`) is `BasePopupHandle` with
//!   `throw_on_missing_trigger: false` — `open(triggerId)` opens unassociated with a
//!   dev warning when the trigger is not registered (`DialogHandle.ts:30-56`). The
//!   branded `AlertDialogHandle` subclass carries no runtime behavior (alert-dialog's
//!   implementation.md, "Facade shape" item 2), so the port defines no separate type.
//! - **Actions** (`useRenderDialogRoot.tsx:80-87`): `unmount: forceUnmount` and
//!   `close: () => store.setOpen(false, createChangeEventDetails(REASONS.imperativeAction))`.
//!
//! ## Rust adaptations
//!
//! - The dialog-specific fields live in [`DialogExtraState`], a reactive slab on the
//!   context rather than upstream's flat state record: the shared `PopupStoreState`
//!   type is the internals crate's public surface, and dialog's fields are the ones no
//!   other popup family needs. Reads derive per-field signals, preserving the
//!   context-value-is-the-store subscription contract.
//! - The context value crosses `provide_context`'s `Send + Sync` bound through the
//!   `SendWrapper` bridge (the `close_part.rs`/`composite_root_context.rs` precedent).
//! - The payload is `()` — upstream's `Payload` generic is only exercised by the
//!   children-as-function render prop, and the port's root exposes the payload through
//!   the store's shared field for a future typed pass.
//! - `useImperativeHandle(actionsRef)` becomes the [`DialogActions`] handle the root
//!   returns through its component struct (the ref-consumer contract, without the ref).
//! - The InternalBackdrop element is created by the portal view and registered into
//!   the context's ref slot (upstream's `React.createRef` shape).
//! - The `hidden` boolean attribute ports to present/absent (`hidden` present means
//!   hidden).

use std::cell::Cell;
use std::rc::Rc;

use reactive_graph::owner::use_context;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::popup_store::{self, selectors};
use leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::floating_ui::types::{OnOpenChangeFn, RootOpenChangeEventDetails};
use leptos_ui_internals::popup_handle::BasePopupHandle;
use leptos_ui_internals::popup_store_utils::{
    PopupStore, UseOpenStateTransitions, popup_handle_attachment, use_implicit_active_trigger,
    use_open_state_transitions, use_popup_root_sync,
};

use crate::dialog::interactions::dialog_interactions;

// The internals crate's hooks are typed over reactive_graph 0.2 signals, while
// leptos 0.7's prelude signals are the same crate at 0.1.x — the two worlds coexist
// per the toggle/accordion precedent: the aliased rg-0.2 types feed the internals
// calls, the prelude types feed views and context.
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::wrappers::read::Signal as RgSignal;

pub mod interactions;
pub mod parts;

/// `REASONS` re-exports the shared registry (`packages/react/src/internals/reasons.ts`);
/// dialog adds no reasons of its own.
pub use leptos_ui_internals::floating_ui::reasons as REASONS;

/// The change-details type (`DialogRoot.ChangeEventDetails` =
/// `createChangeEventDetails(reason, nativeEvent)`): exposes `cancel()` /
/// `isCanceled()` / `preventUnmountOnClose()` to the consumer.
pub type DialogChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::Event>;

/// `onOpenChange` (`DialogRoot.tsx:36`): `(open, eventDetails)`.
pub type OnOpenChange = Rc<dyn Fn(bool, &RootOpenChangeEventDetails)>;
/// `onOpenChangeComplete` (`DialogRoot.tsx:38`): `(open)`.
pub type OnOpenChangeComplete = Rc<dyn Fn(bool)>;

/// The `DialogRootMode` union (`useRenderDialogRoot.tsx:106`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogRootMode {
    /// `'dialog'`.
    Dialog,
    /// `'drawer'` — the drawer-mode plumbing passes through the shared renderer but
    /// can never be exercised by dialog's own suite (implementation.md, untested
    /// item 7); the mode only contributes the nested-open counting arm today.
    Drawer,
    /// `'alert-dialog'` — forces `modal: true`, `disablePointerDismissal: true`, and
    /// `role: 'alertdialog'` (`useRenderDialogRoot.tsx:35-39`).
    AlertDialog,
}

/// The dialog-specific state slab — upstream's `State<Payload>` extension of
/// `PopupStoreState` (`DialogStore.ts:16-27`), the fields no other popup family needs.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DialogExtraState {
    /// `modal: boolean | 'trap-focus'` (`:17`) — the port carries the boolean; the
    /// `'trap-focus'` arm is set only by focus-trap consumers (none in this unit's
    /// tests; implementation.md, untested item 4).
    pub modal: bool,
    /// `disablePointerDismissal` (`:18`).
    pub disable_pointer_dismissal: bool,
    /// `nested` (`:20`).
    pub nested: bool,
    /// `nestedOpenDialogCount` (`:21`).
    pub nested_open_dialog_count: u32,
    /// `nestedOpenDrawerCount` (`:22`).
    pub nested_open_drawer_count: u32,
    /// `titleElementId` (`:23`).
    pub title_element_id: Option<String>,
    /// `descriptionElementId` (`:24`).
    pub description_element_id: Option<String>,
    /// `role` (`:26`).
    pub role: String,
}

/// The dialog context — upstream's `DialogRootContext` value is the store itself
/// (`DialogRootContext.ts:5`); the port's context carries the shared store handle plus
/// the dialog extra-state slab and the context ref slots (the `Context` extension of
/// `DialogStore.ts:29-35`).
#[derive(Clone)]
pub struct DialogRootContext {
    /// The store — the context value upstream (`DialogRootContext.ts:5`).
    pub store: PopupStore<()>,
    /// The dialog-specific reactive slab.
    pub extra: RgRwSignal<DialogExtraState, reactive_graph::owner::LocalStorage>,
    /// `backdropRef` (`DialogStore.ts:31`).
    pub backdrop_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
    /// `internalBackdropRef` (`DialogStore.ts:32`).
    pub internal_backdrop_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
    /// `outsidePressEnabledRef` (`DialogStore.ts:33`).
    pub outside_press_enabled: Rc<Cell<bool>>,
    /// The viewport element slot (`:25`).
    pub viewport_element_ref: Rc<Cell<Option<web_sys::HtmlElement>>>,
    /// `onNestedDialogOpen` (`:34`) — the parent-notification callback the root wires
    /// when nested.
    pub on_nested_dialog_open: Option<Rc<dyn Fn(u32, u32)>>,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `close_part.rs`
/// precedent).
pub type SharedDialogRootContext = SendWrapper<DialogRootContext>;

impl DialogRootContext {
    /// `useDialogRootContext()` — the required form (`DialogRootContext.ts:9-18`):
    /// panics when used outside a Root, upstream's throw.
    pub fn expect() -> Self {
        use_context::<SharedDialogRootContext>()
            .expect("Base UI: Dialog parts must be used within <Dialog.Root> (the DialogRootContext is missing).")
            .take()
    }

    /// `useDialogRootContext(true)` — the optional form (`:9`): `None` outside a Root.
    pub fn optional() -> Option<Self> {
        use_context::<SharedDialogRootContext>().map(|shared| shared.take())
    }

    /// Reads a derived signal over the extra slab.
    pub fn extra_signal<T: Clone + PartialEq + 'static>(
        &self,
        selector: impl Fn(&DialogExtraState) -> T + 'static,
    ) -> RgSignal<T, reactive_graph::owner::LocalStorage> {
        let extra = self.extra;
        RgSignal::derive_local(move || selector(&extra.get()))
    }
}

/// `DialogRoot.Actions` (`useRenderDialogRoot.tsx:80-87`): the imperative handle the
/// root fills. The consumer holds the instance (upstream's `actionsRef` ref-object
/// contract — the port returns the handle instead of filling a ref).
#[derive(Clone)]
pub struct DialogActions {
    /// `unmount` (`:83`) — the `forceUnmount` callback from the open-state transitions.
    unmount: Rc<dyn Fn()>,
    store: PopupStore<()>,
}

impl DialogActions {
    /// `actionsRef.current.unmount()` — removes the popup after a
    /// `preventUnmountOnClose()` close request (behavior.md "State model").
    pub fn unmount(&self) {
        (self.unmount)();
    }

    /// `actionsRef.current.close()` — closes the dialog imperatively with
    /// `REASONS.imperativeAction` (`:84`).
    pub fn close(&self) {
        self.store
            .context
            .on_open_change
            .as_ref()
            .expect("the root wires the store writer before exposing Actions")(
            false,
            &RootOpenChangeEventDetails::new(
                reasons::IMPERATIVE_ACTION,
                web_sys::Event::new("base-ui").expect("the Event constructor is available"),
                None,
                String::new(),
            ),
        );
    }
}

/// The store handle the trigger reads when detached — upstream's
/// `DialogHandleStore<Payload>` (`DialogStore.ts:56`), the shared popup store handle
/// for the `()` payload.
pub type DialogHandleStore = PopupStore<()>;

/// `DialogHandle` (`packages/react/src/dialog/store/DialogHandle.ts:14-20`): a
/// `BasePopupHandle` that does not throw on a missing trigger — a non-anchored popup
/// opens unassociated with a dev warning (`DialogHandle.ts:30-56`). The branded
/// `AlertDialogHandle` is this same type (the brand is compile-time only —
/// alert-dialog's implementation.md, untested item 1), so the port defines no
/// separate type.
pub type DialogHandle = BasePopupHandle<
    leptos_ui_utils::react_store::ReactStore<
        popup_store::PopupStoreState<()>,
        popup_store::PopupStoreContext<RootOpenChangeEventDetails>,
    >,
>;

/// `Dialog.createHandle()` / `AlertDialog.createHandle()` (`DialogHandle.ts:22-27`):
/// builds the handle attached to an inert fallback store (the
/// `createNullDialogStore` shape — closed, no writer, its own trigger registry).
pub fn create_handle() -> Rc<DialogHandle> {
    let trigger_elements = PopupTriggerMap::new();
    let state =
        popup_store::create_initial_popup_store_state(&trigger_elements.clone(), None, false);
    let store = Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
        state,
        popup_store::PopupStoreContext {
            trigger_elements,
            popup_ref: Rc::new(Cell::new(None)),
            on_open_change: None,
            on_open_change_complete: None,
        },
    ));
    Rc::new(BasePopupHandle::new(store, "Dialog", false))
}

/// The root props — upstream's `DialogRootProps` (`useRenderDialogRoot.tsx:21-33`)
/// with the documented defaults.
#[derive(Clone)]
pub struct DialogRootProps {
    /// `open` (`:23`) — the controlled value; `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` (`:24` — upstream default `false`).
    pub default_open: bool,
    /// `onOpenChange` (`:25`).
    pub on_open_change: Option<OnOpenChange>,
    /// `onOpenChangeComplete` (`:26`).
    pub on_open_change_complete: Option<OnOpenChangeComplete>,
    /// `disablePointerDismissal` (`:27` — upstream default `false`; forced `true` in
    /// the alert mode).
    pub disable_pointer_dismissal: bool,
    /// `modal` (`:28` — upstream default `true`; forced `true` in the alert mode).
    pub modal: bool,
    /// `triggerId` (`:31`) — selects the active trigger for ARIA sync.
    pub trigger_id: Option<String>,
    /// `defaultTriggerId` (`:32` — upstream default `null`).
    pub default_trigger_id: Option<String>,
    /// `handle` — binds the root to a handle for detached triggers (`:30`).
    pub handle: Option<Rc<DialogHandle>>,
    /// The mode (`:17`) — the shared renderer's parameter.
    pub mode: DialogRootMode,
}

impl Default for DialogRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            on_open_change: None,
            on_open_change_complete: None,
            disable_pointer_dismissal: false,
            modal: true,
            trigger_id: None,
            default_trigger_id: None,
            handle: None,
            mode: DialogRootMode::Dialog,
        }
    }
}

/// The open-state commit — `DialogStore.setOpen` (`DialogStore.ts:74-97`) ported
/// over the shared popup store: attach `preventUnmountOnClose`, backfill the closing
/// trigger, notify `onOpenChange`, honor `isCanceled`, dispatch the floating-root
/// change, commit `createPopupOpenState`.
pub fn dialog_set_open(
    store: &PopupStore<()>,
    next_open: bool,
    event_details: &mut RootOpenChangeEventDetails,
) {
    // `eventDetails.preventUnmountOnClose = () => …` (`:78-80`): the port's
    // `attach_prevent_unmount_on_close` installs the request flag; the commit reads it
    // after `onOpenChange` returns (`popupStoreUtils.ts:283`).
    let prevent_unmount_requested =
        leptos_ui_internals::popup_store_utils::attach_prevent_unmount_on_close(event_details);

    // The closing-trigger backfill (`:82-86`): when closing with no trigger on the
    // details, pass the old active trigger's element so `onOpenChange`'s
    // `details.trigger` still points at the original trigger (behavior.md "Events").
    if !next_open && event_details.trigger.is_none() {
        let snapshot = store.get_snapshot();
        if snapshot.active_trigger_id.is_some() {
            event_details.trigger = snapshot.active_trigger_element.clone();
        }
    }

    // The user callback (`:88`).
    if let Some(on_open_change) = &store.context.on_open_change {
        on_open_change(next_open, event_details);
    }

    // The cancel gate (`:90-92`).
    if event_details.is_canceled() {
        return;
    }

    // The floating-root dispatch (`:94`).
    store
        .get_snapshot()
        .floating_root_context
        .dispatch_open_change(next_open, event_details);

    // The commit (`:96`).
    let popup_open_state = {
        let snapshot = store.get_snapshot();
        leptos_ui_internals::popup_store_utils::create_popup_open_state(
            &snapshot,
            next_open,
            event_details.trigger.as_ref(),
            prevent_unmount_requested.get(),
        )
    };
    store.update(move |state, _| {
        state.open = popup_open_state.open;
        state.prevent_unmounting_on_close = popup_open_state.prevent_unmounting_on_close;
        state.active_trigger_id = popup_open_state.active_trigger_id.clone();
        state.active_trigger_element = popup_open_state.active_trigger_element.clone();
        true
    });
}

/// The root's shared writer — `onOpenChange` as the store's open writer for the popup
/// machinery (the `use_popup_root_store` explicit-writer convention): the full
/// `dialog_set_open` sequence. The user callback notification lives on the same
/// context slot, so the machinery path notifies exactly once.
fn root_writer(store: &PopupStore<()>) -> OnOpenChangeFn {
    let store = Rc::clone(store);
    Rc::new(
        move |next_open: bool, details: &RootOpenChangeEventDetails| {
            dialog_set_open(&store, next_open, &mut { details.clone() });
        },
    )
}

/// The root value the `DialogRoot` component produces — the store, the context, and
/// the actions handle, handed back to the caller so tests and consumers drive the
/// dialog without needing a ref-filled prop (the port's `actionsRef` analog).
pub struct DialogRootValue {
    /// The imperative `Actions` handle (`:80-87`).
    pub actions: DialogActions,
    /// The open-state read (`:72` — the coalescing selector).
    pub open: RgRwSignal<bool, reactive_graph::owner::LocalStorage>,
    /// The mounted-state read (`:73`).
    pub mounted: RgRwSignal<bool, reactive_graph::owner::LocalStorage>,
    /// The payload read (`:74`).
    pub payload: RgRwSignal<Option<()>, reactive_graph::owner::LocalStorage>,
}

/// `useRenderDialogRoot(mode, props)` (`useRenderDialogRoot.tsx:17-104`) as the
/// public `Dialog.Root` body — the hook-shaped entry the component view calls.
/// Must be called inside a reactive owner. Returns the root value; the caller renders
/// `children` inside the provided context (the [`DialogRootView`] shim).
pub fn use_render_dialog_root(
    props: DialogRootProps,
) -> (DialogRootValue, SharedDialogRootContext) {
    let DialogRootProps {
        open: open_prop,
        default_open,
        on_open_change,
        on_open_change_complete,
        disable_pointer_dismissal: disable_pointer_dismissal_prop,
        modal: modal_prop,
        trigger_id: trigger_id_prop,
        default_trigger_id: default_trigger_id_prop,
        handle,
        mode,
    } = props;

    // The mode-forced values (`:35-39`) — the entire alert-dialog delta.
    let is_alert_dialog = mode == DialogRootMode::AlertDialog;
    let modal = is_alert_dialog || modal_prop;
    let disable_pointer_dismissal = is_alert_dialog || disable_pointer_dismissal_prop;
    let role = if is_alert_dialog {
        "alertdialog"
    } else {
        "dialog"
    }
    .to_owned();

    // The parent store (`:41-42`) — the optional context read derives `nested`.
    let parent_store = DialogRootContext::optional();
    let nested = parent_store.is_some();

    // The dialog slab (`:43` + `:57`): the mode-forced root state.
    let extra: RgRwSignal<DialogExtraState, reactive_graph::owner::LocalStorage> =
        RgRwSignal::new_local(DialogExtraState {
            modal,
            disable_pointer_dismissal,
            nested,
            nested_open_dialog_count: 0,
            nested_open_drawer_count: 0,
            title_element_id: None,
            description_element_id: None,
            role: role.clone(),
        });

    // The store (`:49-63`), created exactly once, with the mode-forced values seeded
    // on the slab and the shared state carrying the open/trigger seeds.
    let trigger_elements = PopupTriggerMap::new();
    let initial_state =
        popup_store::create_initial_popup_store_state(&trigger_elements.clone(), None, nested);
    let store: PopupStore<()> = {
        let on_open_change: Option<OnOpenChangeFn> = {
            // The context slot is the user's `onOpenChange` (`:69`); the port also
            // routes the full `dialog_set_open` through the same slot (the writer and
            // the notification are one call — `dialog_set_open` invokes the slot as
            // its user-notification step, so the slot holds only the user callback
            // and the *commit* runs in the writer below).
            let user_callback = on_open_change.clone();
            user_callback.map(|callback| {
                Rc::new(move |open: bool, details: &RootOpenChangeEventDetails| {
                    callback(open, details)
                }) as OnOpenChangeFn
            })
        };
        Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
            initial_state,
            popup_store::PopupStoreContext {
                trigger_elements,
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change,
                on_open_change_complete: on_open_change_complete
                    .map(|callback| callback as Rc<dyn Fn(bool)>),
            },
        ))
    };

    // The controlled-prop syncs (`:65-66`): the port writes the props into the raw
    // fields once (the coalescing selectors read them with the prop winning).
    store.set_field(|state| &mut state.open_prop, open_prop);
    store.set_field(|state| &mut state.trigger_id_prop, trigger_id_prop.clone());
    // `activeTriggerId` seeds from `defaultTriggerId` (`:55`).
    store.set_field(
        |state| &mut state.active_trigger_id,
        default_trigger_id_prop.clone(),
    );

    // The reactive reads (`:72-74`).
    let open = store.use_state(selectors::open);
    let mounted = store.use_state(selectors::mounted);
    let payload = store.use_state(selectors::payload);

    // `usePopupRootSync(store, open)` (`:76`).
    use_popup_root_sync(&store, open);

    // `useImplicitActiveTrigger(store)` (`:77`).
    use_implicit_active_trigger(&store, root_writer(&store), false);

    // `useOpenStateTransitions(open, store)` (`:78`).
    let UseOpenStateTransitions { force_unmount, .. } =
        use_open_state_transitions(open, &store, None, false);

    // The actions handle (`:80-87`).
    let actions = DialogActions {
        unmount: Rc::new(move || {
            force_unmount.call(());
        }),
        store: Rc::clone(&store),
    };

    // The context value (`:92`).
    let context = DialogRootContext {
        store: Rc::clone(&store),
        extra,
        backdrop_ref: Rc::new(Cell::new(None)),
        internal_backdrop_ref: Rc::new(Cell::new(None)),
        outside_press_enabled: Rc::new(Cell::new(true)),
        viewport_element_ref: Rc::new(Cell::new(None)),
        on_nested_dialog_open: parent_store.as_ref().map(|parent| {
            let parent_extra = parent.extra;
            let callback: Rc<dyn Fn(u32, u32)> =
                Rc::new(move |dialog_count: u32, drawer_count: u32| {
                    parent_extra.set(DialogExtraState {
                        nested_open_dialog_count: dialog_count,
                        nested_open_drawer_count: drawer_count,
                        ..parent_extra.get_untracked()
                    });
                });
            callback
        }),
    };
    let shared_context = SendWrapper::new(context);

    // `handle` attachment (`:93`): the layout-effect attach before descendants.
    if let Some(handle) = &handle {
        popup_handle_attachment(handle.clone(), Rc::clone(&store));
    }

    // `DialogInteractions` is only rendered while `open || mounted` (`:89-100`): the
    // port's view shim owns the gate (see `DialogRootView`); the machinery call runs
    // inside that gate. The `Show` lives here (the prelude is imported at the module
    // top; the interactions context is provided by the caller's view wrapper).
    let interactions_context = shared_context.clone();
    let interactions_mode = mode;
    let interactions_open = open;
    let interactions_mounted = mounted;
    let view = {
        let _ = &interactions_context;
        let _ = interactions_mode;
        let _ = &interactions_open;
        let _ = &interactions_mounted;
        // The machinery is invoked by the component view (see `DialogRoot`), not
        // here: `dialog_interactions` must run inside the reactive owner of the
        // mounted subtree, which the `Show` branch creates per activation.
        ()
    };

    (
        DialogRootValue {
            actions,
            open,
            mounted,
            payload,
        },
        shared_context,
    )
}

/// The `Dialog.Root` component — the view wrapper over [`use_render_dialog_root`]:
/// provides the context, renders the interactions gate while `open || mounted`
/// (`useRenderDialogRoot.tsx:89-100`), and renders the children inside the context.
#[leptos::component]
pub fn DialogRootComponent(
    #[prop(default = DialogRootProps::default(), optional)] dialog_props: DialogRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    use leptos::prelude::*;

    let (value, shared_context) = use_render_dialog_root(dialog_props);
    let context = shared_context;
    let open = value.open;
    let mounted = value.mounted;
    let DialogRootValue { actions, .. } = value;
    STORE_ACTIONS_SLOT.with(|slot| *slot.borrow_mut() = Some(actions));

    // The context provision (`:92`) — the descendants (Trigger/Portal/Popup/…)
    // consume it through `use_context`.
    provide_dialog_context((*context).clone());

    view! {
        <>
            <Show when=move || open.get() || mounted.get() fallback=|| ()>
                {dialog_interactions(crate::dialog::SharedDialogRootContext::new(
                    (*context).clone(),
                ))}
            </Show>
            {children()}
        </>
    }
}

/// The imperative `Actions` handle of the most recently rendered `DialogRoot` — the
/// port's stand-in for the consumer's `actionsRef` (the ref-object contract rendered
/// as a thread-local slot, the same single-owner assumption the wasm single-thread
/// reactive graph makes). Tests and consumers call `dialog_actions()` after mounting.
thread_local! {
    static STORE_ACTIONS_SLOT: std::cell::RefCell<Option<DialogActions>> =
        const { std::cell::RefCell::new(None) };
}

/// Reads the mounted root's `Actions` handle (the `actionsRef.current` read).
pub fn dialog_actions() -> Option<DialogActions> {
    STORE_ACTIONS_SLOT.with(|slot| slot.borrow().clone())
}

/// Provides the dialog context and renders children — the context provision half of
/// `DialogRoot`. Must be called inside the root's reactive owner (the view tree).
pub fn provide_dialog_context(context: DialogRootContext) {
    reactive_graph::owner::provide_context(SendWrapper::new(context));
}
