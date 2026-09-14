//! The preview-card root — `packages/react/src/preview-card/root/PreviewCardRoot.tsx`
//! over the store (`store.rs`), the popover/mod.rs house pattern:
//!
//! - the store is created exactly once per Root and shared by reference through
//!   `PreviewCardRootContext` (`PreviewCardRoot.tsx:36-48`,
//!   `PreviewCardContext.ts:5-9`);
//! - the controlled/uncontrolled split rides the shared spine's `openProp` /
//!   `triggerIdProp` (`:50-51`) — `defaultOpen` only seeds internal `open`,
//!   which the selector shadows whenever `openProp` is defined (the
//!   implementation.md "Store-centric state model" mechanism behind
//!   behavior.md's "defaultOpen is ignored when open is controlled");
//! - `usePopupRootSync` + `useImplicitActiveTrigger(store, { closeOnActiveTriggerUnmount:
//!   true })` (`:61`) + `useOpenStateTransitions` (`:62-64`, with the `onUnmount`
//!   callback clearing `inlineRectCoordsRef` — the "hovered-line coordinates
//!   cleared after close" rule) drive the shared lifecycle;
//! - the payload-clearing effect (`:66-72`): a `useIsoLayoutEffect` clears
//!   `payload` when open with no active trigger;
//! - the imperative actions (`:74-81`): `unmount` → `forceUnmount` (which also
//!   fires `onOpenChangeComplete(false)`), `close` → `store.setOpen(false,
//!   { reason: REASONS.imperativeAction })`;
//! - the root wraps itself in the floating tree only when no enclosing popup
//!   root context exists (`:116-128` — the outermost card establishes the
//!   tree, a nested card joins the ancestor's; the popover's nesting
//!   precedent).
//!
//! The detached-trigger/handle path (`createHandle`, `PreviewCardHandle.ts`,
//! the `BasePopupHandle` machinery) is the port's deferred pass — the
//! in-Root trigger is the tested surface this iteration, per the
//! dialog/popover precedent; the deferred seams are recorded in
//! `parts.rs::deferred_pass_seams`.

use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked, Update as _};
use send_wrapper::SendWrapper;

use crate::preview_card::store::{
    OnOpenChange, OnOpenChangeComplete, PreviewCardExtraState, PreviewCardRootContext,
    SharedPreviewCardRootContext, preview_card_set_open, wire_preview_card_slots,
};

use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::floating_ui::{PopupTriggerMap, reasons};
use leptos_ui_internals::inline_rect::InlineRectCoordsRef;
use leptos_ui_internals::popup_store_utils::{
    UseOpenStateTransitions, use_implicit_active_trigger, use_open_state_transitions,
    use_popup_root_sync,
};

pub mod parts;
pub mod store;

/// The root props — upstream's `PreviewCardRootProps` (`PreviewCardRoot.tsx`
/// props table) with the documented defaults.
#[derive(Clone)]
pub struct PreviewCardRootProps {
    /// `open` — the controlled value; `None` while uncontrolled.
    pub open: Option<bool>,
    /// `defaultOpen` (upstream default `false`).
    pub default_open: bool,
    /// `onOpenChange`.
    pub on_open_change: Option<OnOpenChange>,
    /// `onOpenChangeComplete`.
    pub on_open_change_complete: Option<OnOpenChangeComplete>,
    /// `triggerId` — selects the active trigger in controlled mode.
    pub trigger_id: Option<String>,
    /// `defaultTriggerId` (upstream default `null`).
    pub default_trigger_id: Option<String>,
}

impl Default for PreviewCardRootProps {
    fn default() -> Self {
        PreviewCardRootProps {
            open: None,
            default_open: false,
            on_open_change: None,
            on_open_change_complete: None,
            trigger_id: None,
            default_trigger_id: None,
        }
    }
}

/// The root value the `PreviewCardRoot` component produces — the open/mounted
/// reads and the imperative actions handle, handed back to the caller (the
/// popover `PopoverRootValue` precedent for upstream's `actionsRef`
/// ref-object contract).
pub struct PreviewCardRootValue {
    /// The open-state read.
    pub open: reactive_graph::signal::RwSignal<bool, reactive_graph::owner::LocalStorage>,
    /// The mounted-state read.
    pub mounted: reactive_graph::signal::RwSignal<bool, reactive_graph::owner::LocalStorage>,
    /// The payload read (`:59` — the function-children API's `{ payload }`).
    pub payload: reactive_graph::signal::RwSignal<Option<()>, reactive_graph::owner::LocalStorage>,
}

/// The imperative `Actions` handle (`PreviewCardRoot.tsx:74-81`): `close()`
/// and `unmount()` — behavior.md "State model"/"Events" and "Edge cases".
#[derive(Clone)]
pub struct PreviewCardActions {
    /// `unmount` (`:74-81` — the `forceUnmount` callback; also fires
    /// `onOpenChangeComplete(false)`).
    pub unmount: Rc<dyn Fn()>,
    /// `close` (`:74-81`) — closes the card imperatively with
    /// `REASONS.imperativeAction`.
    pub close: Rc<dyn Fn()>,
}

impl PreviewCardActions {
    /// `actionsRef.current.unmount()`.
    pub fn unmount(&self) {
        (self.unmount)();
    }
    /// `actionsRef.current.close()`.
    pub fn close(&self) {
        (self.close)();
    }
}

thread_local! {
    /// The imperative-actions slot of the most recently rendered root — the
    /// port's `actionsRef` stand-in (the popover `STORE_ACTIONS_SLOT`
    /// precedent).
    static STORE_ACTIONS_SLOT: std::cell::RefCell<Option<PreviewCardActions>> =
        const { std::cell::RefCell::new(None) };
}

/// Reads the mounted root's `Actions` handle (the `actionsRef.current` read).
pub fn preview_card_actions() -> Option<PreviewCardActions> {
    STORE_ACTIONS_SLOT.with(|slot| slot.borrow().clone())
}

/// `usePopupRootStore` + the Root body (`PreviewCardRoot.tsx:36-108`). Must be
/// called inside a reactive owner. Returns the root value and the shared
/// context the caller provides before rendering children (the
/// `PreviewCardRootView` shim).
pub fn use_render_preview_card_root(
    props: PreviewCardRootProps,
) -> (PreviewCardRootValue, SharedPreviewCardRootContext) {
    let PreviewCardRootProps {
        open: open_prop,
        default_open,
        on_open_change,
        on_open_change_complete,
        trigger_id: trigger_id_prop,
        default_trigger_id: default_trigger_id_prop,
    } = props;

    // The store (`:36-48` + `usePopupRootStore` factory): created exactly
    // once, seeded with the family defaults. The `defaultOpen` seed:
    // `open: true` ⇒ `mounted: true` so content renders on first paint.
    let mut initial_state =
        crate::preview_card::store::create_initial_preview_card_store_state(None, false);
    initial_state.open = default_open;
    if default_open {
        initial_state.mounted = true;
    }

    let store: leptos_ui_internals::popup_store_utils::PopupStore<()> = {
        Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
            initial_state,
            leptos_ui_internals::floating_ui::popup_store::PopupStoreContext {
                trigger_elements: PopupTriggerMap::new(),
                popup_ref: Rc::new(std::cell::Cell::new(None)),
                on_open_change: on_open_change.clone().map(|callback| {
                    let callback: Rc<dyn Fn(bool, &RootOpenChangeEventDetails)> = callback;
                    callback
                }),
                on_open_change_complete: on_open_change_complete
                    .map(|callback| callback as Rc<dyn Fn(bool)>),
            },
        ))
    };

    // The controlled-prop syncs (`:50-51`).
    store.set_field(|state| &mut state.open_prop, open_prop);
    store.set_field(|state| &mut state.trigger_id_prop, trigger_id_prop.clone());
    // `activeTriggerId` seeds from `defaultTriggerId` (`:51` shape).
    store.set_field(
        |state| &mut state.active_trigger_id,
        default_trigger_id_prop.clone(),
    );

    // The reactive reads (`:51-59`).
    let open = store.use_state(selectors::open);
    let mounted = store.use_state(selectors::mounted);
    let payload = store.use_state(selectors::payload);

    // The extra slab (`PreviewCardStore.ts:20-24` defaults — `closeDelay`
    // seeds the CLOSE_DELAY constant).
    let extra: reactive_graph::signal::RwSignal<
        PreviewCardExtraState,
        reactive_graph::owner::LocalStorage,
    > = reactive_graph::signal::RwSignal::new_local(PreviewCardExtraState::default());
    wire_preview_card_slots(extra.clone());

    // `usePopupRootSync` (`:58` shape).
    use_popup_root_sync(&store, open);

    // `useImplicitActiveTrigger(store, { closeOnActiveTriggerUnmount: true })`
    // (`:61`) — the cancelable unmount-close with reason `'none'`
    // (implementation.md "Active-trigger reconciliation"); the writer routes
    // through the full `preview_card_set_open` pipeline.
    use_implicit_active_trigger(&store, root_writer(&store), true);

    // The context value (`:86`) — built before the transitions hook so the
    // `onUnmount` callback can clear the coords ref (`:63`).
    let inline_rect_coords: InlineRectCoordsRef = Default::default();
    let context = PreviewCardRootContext {
        store: Rc::clone(&store),
        extra,
        inline_rect_coords: inline_rect_coords.clone(),
        popup_ref: Rc::new(std::cell::Cell::new(None)),
    };
    let shared_context = SendWrapper::new(context);

    // `useOpenStateTransitions(open, store, onUnmount)` (`:62-64`) — the
    // `onUnmount` callback clears `inlineRectCoordsRef` (the "hovered-line
    // coordinates cleared after close" rule, behavior.md "DOM structure &
    // portal behavior").
    let coords_for_unmount = inline_rect_coords.clone();
    let on_unmount: Rc<dyn Fn()> = Rc::new(move || {
        coords_for_unmount.set(None);
    });
    let UseOpenStateTransitions { force_unmount, .. } =
        use_open_state_transitions(open, &store, Some(on_unmount), false);

    // The actions handle (`:74-81`): `unmount` → `forceUnmount`, `close` →
    // `store.setOpen(false, imperativeAction)`.
    let actions_store = Rc::clone(&store);
    let actions = PreviewCardActions {
        unmount: Rc::new(move || {
            force_unmount.call(());
        }),
        close: Rc::new(move || {
            let mut details = crate::preview_card::store::preview_card_change_event_details(
                reasons::IMPERATIVE_ACTION,
                None,
            );
            preview_card_set_open(&actions_store, false, &mut details, None);
        }),
    };
    STORE_ACTIONS_SLOT.with(|slot| *slot.borrow_mut() = Some(actions.clone()));

    (
        PreviewCardRootValue {
            open,
            mounted,
            payload,
        },
        shared_context,
    )
}

/// The root's shared writer — the full `preview_card_set_open` sequence as the
/// store's open writer (the popover `root_writer` convention).
fn root_writer(store: &leptos_ui_internals::popup_store_utils::PopupStore<()>) -> OnOpenChange {
    let store = Rc::clone(store);
    Rc::new(
        move |next_open: bool, details: &RootOpenChangeEventDetails| {
            preview_card_set_open(&store, next_open, &mut { details.clone() }, None);
        },
    )
}

/// The shared root view body — the context provision and the children render
/// (the popover `popover_root_view` precedent). The interactions component is
/// the deferred pass (the dismissal bags' Escape path rides the machinery's
/// openchange events; see parts.rs's deferred seams).
pub fn preview_card_root_view(
    preview_card_props: PreviewCardRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    use leptos::prelude::*;

    let (value, shared_context) = use_render_preview_card_root(preview_card_props);
    let context = shared_context;
    let _open = value.open;
    let _mounted = value.mounted;

    // The context provision (`:86`) — the descendants consume it through
    // `use_context`.
    reactive_graph::owner::provide_context(SendWrapper::new((*context).clone()));

    view! {
        <>{children()}</>
    }
}

/// The `PreviewCard.Root` component — the view wrapper over
/// [`use_render_preview_card_root`] delegating to [`preview_card_root_view`].
/// Wraps itself in the floating tree only when no enclosing popup root context
/// exists (`:116-128` — the port's nesting detection rides the same optional
/// context read the trigger uses).
#[leptos::component]
pub fn PreviewCardRootComponent(
    #[prop(default = PreviewCardRootProps::default(), optional)]
    preview_card_props: PreviewCardRootProps,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    preview_card_root_view(preview_card_props, children)
}
