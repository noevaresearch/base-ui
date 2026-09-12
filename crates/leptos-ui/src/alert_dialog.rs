//! Port of the Base UI Alert Dialog — the `library: alert-dialog` TODO item
//! (`specs/library/alert-dialog/behavior.md`, `specs/library/alert-dialog/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **The unit is a thin facade over the dialog unit** ("Facade shape",
//!   implementation.md:10-33): it contributes exactly three things — (1) the mode
//!   override (`AlertDialogRoot` is a one-line call to the shared dialog root renderer
//!   with mode `'alert-dialog'`, `AlertDialogRoot.tsx:14-16`), (2) a
//!   nominally-branded handle with no runtime presence (`handle.ts:11-15`), and (3)
//!   type narrowing + re-exports (`index.parts.ts:1-10`, `index.ts:5-33`). Everything
//!   else is the dialog unit's machinery. This port is therefore a configuration of
//!   the already-ported [`crate::dialog`] unit, per the "Port-relevant summary"
//!   (implementation.md:208-210): "a port needs the `dialog` unit plus the
//!   `utils/popups` stack; nothing else in `alert-dialog/` will require independent
//!   reimplementation."
//! - **The mode-forced values are the entire alert-dialog delta**
//!   (`useRenderDialogRoot.tsx:35-39`): `modal` forced `true`,
//!   `disablePointerDismissal` forced `true`, and `role` becomes `'alertdialog'`.
//!   The shared renderer already implements this via
//!   [`crate::dialog::DialogRootMode::AlertDialog`], so the root here pins the mode
//!   and drops the props the alert surface must not expose
//!   (`AlertDialogRoot.tsx:20-42` Omits `modal` / `disablePointerDismissal` /
//!   `handle` / `actionsRef` / `onOpenChange` — re-declared with alert-dialog types).
//! - **Seven of the nine parts are direct re-exports of dialog parts with no wrapper
//!   component** (`index.parts.ts:2-7`): Backdrop, Close, Description, Popup, Portal,
//!   Title, Viewport. The dialog iteration's port carries those as behavior inside
//!   the shared renderer/store (their machinery is ported and suite-tested) rather
//!   than as separate public part components, so there is nothing new to alias for
//!   them here — this module re-exports the part surface that exists
//!   ([`AlertDialogTrigger`]) and the types, without fabricating wrapper components
//!   upstream itself does not have.
//! - **The branded handle is compile-time only** (`handle.ts:11-15`): the
//!   `__alertDialogBrand` field has no runtime presence, and the only artifact
//!   exercising it is a type-level spec (`AlertDialogRoot.spec.tsx:32-42`) —
//!   implementation.md, untested item 1 (implementation.md:216-220). The port defines
//!   the alias so the crate surface names the alert type, without inventing runtime
//!   behavior upstream doesn't have.

use std::rc::Rc;

use crate::dialog::{
    use_render_dialog_root, DialogHandle, DialogRootMode, DialogRootProps,
    SharedDialogRootContext,
};

/// The trigger — `AlertDialogTrigger` is literally `DialogTrigger` re-exported under
/// a narrowed interface (`AlertDialogTrigger.tsx:16-31`); the Rust port's trigger has
/// no `handle` type parameter to narrow, so the re-export is the component itself.
pub use crate::dialog::parts::DialogTrigger as AlertDialogTrigger;

/// The change-details type — aliased from the dialog type, the port of the
/// `AlertDialogRoot.ChangeEventDetails` alias (`AlertDialogRoot.tsx:44-50`
/// re-exports the dialog reason/details types).
pub use crate::dialog::DialogChangeEventDetails as AlertDialogChangeEventDetails;

/// `AlertDialogHandle<Payload>` extends `DialogHandle<Payload>` adding only a
/// type-level brand field with no runtime presence (`handle.ts:11-15`); all state,
/// attachment, and imperative behavior is inherited from `DialogHandle`
/// (implementation.md, "Facade shape" item 2). The port defines the alias — the brand
/// is compile-time-only upstream (untested item 1, implementation.md:216-220), so
/// there is no runtime behavior to add.
pub type AlertDialogHandle = DialogHandle;

/// `AlertDialog.createHandle()` (`handle.ts:20-22`): the factory just constructs the
/// (branded) handle — the same fallback-store shape `Dialog.createHandle()` builds.
pub fn create_alert_dialog_handle() -> Rc<AlertDialogHandle> {
    crate::dialog::create_handle()
}

/// The root props — upstream's `AlertDialogRootProps` (`AlertDialogRoot.tsx:20-42`)
/// Omits the dialog props that must not be configurable here: `modal` and
/// `disablePointerDismissal` (forced `true` by the mode), `actionsRef` and `handle`
/// (re-declared with alert-dialog types), and `onOpenChange` (re-declared with the
/// alert details type). The remaining fields carry the dialog semantics they alias
/// (`index.ts:5-33`).
#[derive(Clone)]
pub struct AlertDialogRootProps {
    /// `open` — the controlled value; `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` — upstream default `false`.
    pub default_open: bool,
    /// `onOpenChange` — re-declared with the alert-dialog details type (`:20-42`).
    pub on_open_change: Option<crate::dialog::OnOpenChange>,
    /// `onOpenChangeComplete`.
    pub on_open_change_complete: Option<crate::dialog::OnOpenChangeComplete>,
    /// `triggerId` — selects the active trigger for ARIA sync (behavior.md
    /// "Accessibility").
    pub trigger_id: Option<String>,
    /// `defaultTriggerId` — upstream default `null`.
    pub default_trigger_id: Option<String>,
    /// `handle` — binds the root to an `AlertDialogHandle` for detached triggers
    /// (behavior.md "State model", the detached-trigger rows).
    pub handle: Option<Rc<AlertDialogHandle>>,
}

impl Default for AlertDialogRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            on_open_change: None,
            on_open_change_complete: None,
            trigger_id: None,
            default_trigger_id: None,
            handle: None,
        }
    }
}

impl From<AlertDialogRootProps> for DialogRootProps {
    fn from(props: AlertDialogRootProps) -> Self {
        let AlertDialogRootProps {
            open,
            default_open,
            on_open_change,
            on_open_change_complete,
            trigger_id,
            default_trigger_id,
            handle,
        } = props;
        DialogRootProps {
            open,
            default_open,
            on_open_change,
            on_open_change_complete,
            // The Omitted props (`AlertDialogRoot.tsx:20-42`): the alert surface does
            // not accept them — the mode forces both `true`
            // (`useRenderDialogRoot.tsx:35-39`), so the dialog defaults here are
            // irrelevant; only the mode bit survives.
            disable_pointer_dismissal: false,
            modal: false,
            trigger_id,
            default_trigger_id,
            handle,
            mode: DialogRootMode::AlertDialog,
        }
    }
}

/// `AlertDialogRoot` (`AlertDialogRoot.tsx:14-16`) — a one-line call to the shared
/// dialog root renderer with mode `'alert-dialog'`. The port exposes the same
/// hook-shaped entry [`crate::dialog::use_render_dialog_root`] with the mode pinned.
pub fn use_render_alert_dialog_root(
    props: AlertDialogRootProps,
) -> (crate::dialog::DialogRootValue, SharedDialogRootContext) {
    use_render_dialog_root(props.into())
}

/// The `AlertDialog.Root` component — the view wrapper over
/// [`use_render_alert_dialog_root`]. Upstream's body is
/// `useRenderDialogRoot('alert-dialog', props)` (`AlertDialogRoot.tsx:14-16`); the
/// port delegates to the shared root view body
/// [`crate::dialog::dialog_root_view`] (the context provision, the interactions gate
/// while `open || mounted` — `useRenderDialogRoot.tsx:89-100` — and the children
/// render) rather than duplicating it. Named `AlertDialogRootComponent` per the
/// dialog port's `DialogRootComponent` convention (the `#[leptos::component]` macro
/// synthesizes a `<FnName>Props` struct, which would collide with the hand-written
/// [`AlertDialogRootProps`]).
#[leptos::component]
pub fn AlertDialogRootComponent(
    #[prop(default = AlertDialogRootProps::default(), optional)] alert_props: AlertDialogRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let dialog_props: DialogRootProps = alert_props.into();
    crate::dialog::dialog_root_view(dialog_props, children)
}
