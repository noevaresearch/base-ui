//! The popover parts — Trigger, Positioner, Popup, Arrow, Close, Title, Description,
//! Backdrop (`packages/react/src/popover/{trigger,positioner,popup,arrow,close,title,
//! description,backdrop}/`), in the dialog/parts.rs house style: leptos views over
//! the store context, with the shared machinery hooks doing the behavioral work.

use std::cell::Cell;
use std::rc::Rc;

use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update as _, UpdateUntracked as _};
use reactive_graph::wrappers::read::Signal as RgSignal;
use serde_json::json;
use wasm_bindgen::JsCast;

use crate::popover::store::{PopoverChangeEventDetails, PopoverExtraState, PopoverRootContext};
use crate::popover::{PopoverModal, SharedPopoverRootContext};

use leptos::prelude::*;
use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::popup_store_utils::PopupStore;
use leptos_ui_internals::use_anchor_positioning::{
    self, Anchor, ArrowStyles, PositionerStyles, UseAnchorPositioningParams,
};
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};

// ---------------------------------------------------------------------------
// Positioner context (`PopoverPositionerContext.ts:6-17`)
// ---------------------------------------------------------------------------

/// `PopoverPositionerContext` — `{ side, align, arrowRef, arrowUncentered,
/// arrowStyles, context }` (`PopoverPositionerContext.ts:6-17`), provided by the
/// Positioner and consumed by the Popup (side/align state attributes) and the Arrow
/// (arrow wiring).
#[derive(Clone)]
pub struct PopoverPositionerContext {
    /// The engine's positioner styles (the positioner element's inline styles).
    pub positioner_styles: RgSignal<PositionerStyles, reactive_graph::owner::LocalStorage>,
    /// The engine's arrow styles.
    pub arrow_styles: RgSignal<ArrowStyles, reactive_graph::owner::LocalStorage>,
    /// The fillable arrow-element slot (`arrowRef`).
    pub arrow_ref: Rc<std::cell::RefCell<Option<web_sys::Element>>>,
    /// `arrowUncentered` — `middlewareData.arrow?.centerOffset !== 0`.
    pub arrow_uncentered: RgMemo<bool>,
    /// The logical rendered side.
    pub side: RgMemo<leptos_ui_internals::use_anchor_positioning::Side>,
    /// The rendered alignment.
    pub align: RgMemo<leptos_ui_internals::use_anchor_positioning::Align>,
    /// The custom `hide` middleware's report.
    pub anchor_hidden: RgMemo<bool>,
    /// `keepMounted` as provided to the positioner (`:12` — the context also carries
    /// the portal-kept-mounted mode through `useAnchorPositioning`'s `keepMounted`).
    pub keep_mounted: bool,
}

impl PopoverPositionerContext {
    /// `usePopoverPositionerContext()` — the required read (`:20-27`); throws
    /// upstream when the Popup/Arrow are used outside a Positioner.
    pub fn expect() -> Self {
        reactive_graph::owner::use_context::<SharedPopoverPositionerContext>()
            .expect(
                "Base UI: PopoverPositionerContext is missing. Popover.Popup and Popover.Arrow must be placed within <Popover.Positioner>.",
            )
            .take()
    }
}

/// The shared context bridge (the `SharedPopoverRootContext` precedent).
pub type SharedPopoverPositionerContext = send_wrapper::SendWrapper<PopoverPositionerContext>;

// ---------------------------------------------------------------------------
// Trigger (`PopoverTrigger.tsx:25-170`)
// ---------------------------------------------------------------------------

/// The public `Popover.Trigger` component (`PopoverTrigger.tsx:25-170`). Must be
/// called inside a reactive owner and within a Root (the detached-handle path is the
/// port's deferred pass — the in-Root trigger is the tested surface, the dialog
/// precedent).
#[leptos::component]
pub fn PopoverTrigger(
    /// `disabled` (`:41` — upstream default `false`).
    #[prop(default = false)]
    disabled: bool,
    /// `nativeButton` (`:42` — upstream default `true`).
    #[prop(default = true)]
    native_button: bool,
    /// `openOnHover` (`:66` — upstream default `false`).
    #[prop(default = false)]
    open_on_hover: bool,
    /// `payload` (`:71`) — written into the store by the trigger's registration.
    #[prop(default = None)]
    _payload: Option<()>,
    /// Trigger `id` (`:76`) — registered into the store and used for ARIA sync.
    #[prop(default = None)]
    id: Option<String>,
    /// Trigger `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    // The store resolution (`:49-56`) — the in-Root trigger reads the context store.
    let context = PopoverRootContext::expect();
    let store = Rc::clone(&context.store);

    // `useBaseUiId(idProp)` (`:58`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id.clone()));
    let this_trigger_id = id_signal.get_untracked();

    // The store reads (`:59-62`).
    let is_opened_by_this_trigger = store.use_state({
        let this_trigger_id = this_trigger_id.clone();
        move |state| selectors::is_opened_by_trigger(state, Some(&this_trigger_id))
    });
    let popup_id_for_trigger = store.use_state({
        let this_trigger_id = this_trigger_id.clone();
        move |state| selectors::trigger_popup_id(state, Some(&this_trigger_id))
    });

    // The trigger registration (`useTriggerDataForwarding`, `:66-76` — the
    // registration half; the port's dialog `register_trigger_shim` precedent).
    let trigger_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let register_trigger = leptos_ui_internals::popup_store_utils::use_trigger_registration(
        Some(this_trigger_id.clone()),
        &store,
    );

    // The click path (`useClick(floatingContext, { stickIfOpen })`, `:98` + the
    // `useOpenMethodTriggerProps` openMethod recording, `:99-104`): the decision
    // rides the machinery's `next_open_decision` with the store's live
    // `stickIfOpen`, and the open routes through the full `popover_set_open`
    // pipeline with `REASONS.triggerPress`.
    let on_click = {
        let store = Rc::clone(&store);
        let context = context.clone();
        let is_opened_by_this_trigger = is_opened_by_this_trigger.clone();
        let extra = context.extra;
        move |event: web_sys::MouseEvent| {
            // `useOpenMethodTriggerProps` (`useOpenInteractionType.ts:12-26`):
            // record the interaction type on the open click.
            if !is_opened_by_this_trigger.get_untracked() {
                let interaction_type = if event.detail() == 0 {
                    leptos_ui_utils::use_enhanced_click_handler::InteractionType::Keyboard
                } else {
                    leptos_ui_utils::use_enhanced_click_handler::InteractionType::Mouse
                };
                store.update(|state, _| {
                    state.open_method = Some(interaction_type);
                    true
                });
            }

            let next_open = !is_opened_by_this_trigger.get_untracked();
            let mut details = PopoverChangeEventDetails::new(
                reasons::TRIGGER_PRESS.to_owned(),
                event.unchecked_into::<web_sys::Event>(),
                None,
                String::new(),
            );
            popover_set_open_via_context(&context, next_open, &mut details);
        }
    };

    // The aria bag (`:133-139`).
    let aria_expanded = move || is_opened_by_this_trigger.get().to_string();
    let aria_controls = move || popup_id_for_trigger.get().unwrap_or_default();
    // `triggerOpenStateMapping` (`:113-121`): `data-popup-open` when open; the
    // `data-pressed` pressable arm keys on the open reason being `triggerPress` —
    // the port derives it from the slab's `openChangeReason`.
    let open_reason = context.extra_signal(|state| state.open_change_reason.clone());
    let data_popup_open = move || is_opened_by_this_trigger.get().then_some("true".to_owned());
    let data_disabled = move || disabled.then_some("true".to_owned());
    let _ = open_reason;

    // The registration effect (the dialog trigger's mount-registration precedent).
    {
        let register = register_trigger.clone();
        let element_slot = Rc::clone(&trigger_element_ref);
        leptos::prelude::Effect::new(move |_| {
            let element = element_slot.take();
            if element.is_some() {
                element_slot.set(element.clone());
            }
            register.call(element);
        });
    }

    let this_trigger_id_for_view = this_trigger_id.clone();
    view! {
        <button
            type="button"
            class=class
            id=this_trigger_id_for_view
            disabled=move || disabled.then_some("true")
            aria-haspopup="dialog"
            aria-expanded=aria_expanded
            aria-controls=aria_controls
            data-popup-open=data_popup_open
            data-disabled=data_disabled
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// The context-carrying open dispatch — routes through the full
/// `popover_set_open` pipeline with the root's slab/timeout slots wired (the
/// upstream `store.setOpen` emission, `PopoverStore.ts:165-167`).
pub(crate) fn popover_set_open_via_context(
    context: &PopoverRootContext,
    next_open: bool,
    details: &mut RootOpenChangeEventDetails,
) {
    crate::popover::store::popover_set_open(&context.store, next_open, details);
}

// ---------------------------------------------------------------------------
// Positioner (`PopoverPositioner.tsx:33-160`)
// ---------------------------------------------------------------------------

/// The public `Popover.Positioner` component (`PopoverPositioner.tsx:33-160`):
/// renders the `role="presentation"` element owning the anchor-positioning styles,
/// provides the [`PopoverPositionerContext`], and renders the `InternalBackdrop`
/// before the positioned node for modal non-hover opens (`:155-157`).
#[leptos::component]
pub fn PopoverPositioner(
    /// `side` (`:44` — upstream default `'bottom'`).
    #[prop(default = leptos_ui_internals::use_anchor_positioning::Side::Bottom)]
    side: leptos_ui_internals::use_anchor_positioning::Side,
    /// `sideOffset` (`:45` — upstream default `0`).
    #[prop(default = 0.0)]
    side_offset: f64,
    /// `align` (`:46` — upstream default `'center'`).
    #[prop(default = leptos_ui_internals::use_anchor_positioning::Align::Center)]
    align: leptos_ui_internals::use_anchor_positioning::Align,
    /// `alignOffset` (`:47` — upstream default `0`).
    #[prop(default = 0.0)]
    align_offset: f64,
    /// `keepMounted` (`:55` — forwarded into the positioning engine's
    /// persistent mode).
    #[prop(default = false)]
    keep_mounted: bool,
    /// Positioner `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PopoverRootContext::expect();
    let store = Rc::clone(&context.store);

    // The mounted read (`usePositioner`'s `hidden={!mounted}`, `:144-151`).
    let mounted = store.use_state(selectors::mounted);
    let open = store.use_state(selectors::open);

    // `useAnchorPositioning` (`:74-92`) — the entire positioning state machine,
    // anchored to the active trigger element.
    let floating_root_context = store.get_snapshot().floating_root_context.clone();
    let mounted_for_hook = mounted;
    let anchor = Anchor::Fn({
        let store = Rc::clone(&store);
        Rc::new(move || {
            resolve_trigger_anchor(&store).map(|element| {
                leptos_ui_internals::floating_ui::types::ReferenceType::Element(element)
            })
        })
    });
    let mut params =
        UseAnchorPositioningParams::new(floating_root_context, mounted_for_hook.into());
    params.anchor = anchor;
    params.side = side;
    params.side_offset = self::leptos_ui_internals_side_offset(side_offset);
    params.align = align;
    params.align_offset = self::leptos_ui_internals_side_offset(align_offset);
    params.keep_mounted = keep_mounted;
    let positioning = leptos_ui_internals::use_anchor_positioning::use_anchor_positioning(params);

    // The positioner element registration (`store.useStateSetter('positionerElement')`,
    // `:134`) — the NodeRef + mount-effect idiom (the field_root precedent).
    let positioner_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let store = Rc::clone(&store);
        leptos::prelude::Effect::new(move |_| {
            // `NodeRef` rides leptos's reactive_graph (0.1) traits; the file's
            // explicit 0.2 trait imports shadow the prelude glob, so the read
            // goes through the fully qualified 0.1 trait.
            let div =
                match <NodeRef<leptos::html::Div> as leptos::prelude::GetUntracked>::get_untracked(
                    &positioner_node,
                ) {
                    Some(div) => div,
                    None => return,
                };
            let element: web_sys::HtmlElement = div.unchecked_into();
            store.update(|state, _| {
                state.positioner_element = Some(element.clone());
                true
            });
        });
    }

    // The context provision (`:154`): the return's signal members are cloned
    // individually (the struct itself is not `Clone`).
    reactive_graph::owner::provide_context(send_wrapper::SendWrapper::new(
        PopoverPositionerContext {
            positioner_styles: positioning.positioner_styles.clone(),
            arrow_styles: positioning.arrow_styles.clone(),
            arrow_ref: Rc::clone(&positioning.arrow_ref),
            arrow_uncentered: positioning.arrow_uncentered,
            side: positioning.side,
            align: positioning.align,
            anchor_hidden: positioning.anchor_hidden,
            keep_mounted,
        },
    ));

    // The state attributes (`popupStateMapping`, `:144-150`):
    // data-side/data-align/data-anchor-hidden + the open/closed transition mapping.
    let side_attr = move || format!("{:?}", positioning.side.get()).to_lowercase();
    let align_attr = move || format!("{:?}", positioning.align.get()).to_lowercase();
    let anchor_hidden = move || positioning.anchor_hidden.get().then_some("true".to_owned());
    let hidden = move || (!mounted.get() && !keep_mounted).then_some("true".to_owned());

    // The inline positioning styles (`usePositioner.tsx:28-42` — the positioner owns
    // the engine's `positionerStyles`).
    let positioner_styles = positioning.positioner_styles.clone();

    view! {
        <div
            role="presentation"
            class=class
            hidden=hidden
            data-side=side_attr
            data-align=align_attr
            data-anchor-hidden=anchor_hidden
            node_ref=positioner_node
            style=move || {
                let styles = positioner_styles.get();
                let mut style = format!("position: {:?};", styles.position).to_lowercase();
                if let Some(top) = &styles.top { style.push_str(&format!("top: {top};")); }
                if let Some(left) = &styles.left { style.push_str(&format!("left: {left};")); }
                if let Some(right) = &styles.right { style.push_str(&format!("right: {right};")); }
                if let Some(bottom) = &styles.bottom { style.push_str(&format!("bottom: {bottom};")); }
                if let Some(transform) = &styles.transform { style.push_str(&format!("transform: {transform};")); }
                if let Some(opacity) = &styles.opacity { style.push_str(&format!("opacity: {opacity};")); }
                style.push_str(&format!("--available-width: {};", styles.available_width));
                style.push_str(&format!("--available-height: {};", styles.available_height));
                style
            }
        >
            {children()}
        </div>
    }
}

/// Adapts the demo-facing `f64` offsets into the engine's [`SideOffset`].
fn leptos_ui_internals_side_offset(
    value: f64,
) -> leptos_ui_internals::use_anchor_positioning::SideOffset {
    leptos_ui_internals::use_anchor_positioning::SideOffset::Number(value)
}

/// Resolves the active trigger element as the anchor (`Anchor`): the store's
/// active trigger element, else the popup floats unanchored.
fn resolve_trigger_anchor(store: &PopupStore<()>) -> Option<web_sys::Element> {
    let snapshot = store.get_snapshot();
    let active_id = snapshot
        .trigger_id_prop
        .clone()
        .or_else(|| snapshot.active_trigger_id.clone());
    active_id
        .and_then(|id| {
            snapshot
                .floating_root_context
                .context
                .trigger_elements
                .get_by_id(&id)
        })
        .or_else(|| snapshot.active_trigger_element.clone())
}

// ---------------------------------------------------------------------------
// Popup (`PopoverPopup.tsx:30-125`)
// ---------------------------------------------------------------------------

/// The public `Popover.Popup` component (`PopoverPopup.tsx:30-125`): the
/// `role="dialog"` element with the stable `floatingId` (`:90-91`), the
/// title/description ARIA wiring (`:93-94`), the close-part counting (`:37`),
/// the hover floating interaction (`:66`), and the focus manager (`:108-123`).
#[leptos::component]
pub fn PopoverPopup(
    /// `initialFocus` (`:47`) — the port accepts the default machinery path only
    /// (the consumer-ref form is the dialog precedent's deferred pass).
    #[prop(default = false)]
    _initial_focus_default: bool,
    /// Popup `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PopoverRootContext::expect();
    let store = Rc::clone(&context.store);
    let positioner = PopoverPositionerContext::expect();

    // The state reads (`:40-59`).
    let open = store.use_state(selectors::open);
    let instant_type = store.use_state(selectors::instant_type);
    let transition_status = store.use_state(selectors::transition_status);
    let title_id = context.extra_signal(|state| state.title_element_id.clone());
    let description_id = context.extra_signal(|state| state.description_element_id.clone());
    let modal = context.extra_signal(|state| state.modal);
    let floating_id = store.get_snapshot().floating_id.clone().unwrap_or_default();

    // The close-part count (`useClosePartCount`, `:37`): the provider side, provided
    // so nested popups count independently (`:122` — the re-provide).
    let (close_part_context, has_close_part) =
        leptos_ui_internals::close_part::use_close_part_count();
    reactive_graph::owner::provide_context(send_wrapper::SendWrapper::new(close_part_context));

    // `focusManagerModal = modal !== false && hasClosePart` (`:71-72`), synced into
    // the store (`store.useSyncedValue('focusManagerModal', …)`).
    {
        let extra = context.extra;
        let modal = modal.clone();
        reactive_graph::effect::Effect::new(move |_| {
            let focus_manager_modal = modal.get() && has_close_part();
            extra.update(|state| state.focus_manager_modal = focus_manager_modal);
        });
    }

    // The popup element registration (`store.useStateSetter('popupElement')`, `:74`).
    let popup_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let store = Rc::clone(&store);
        let popup_ref = Rc::clone(&context.popup_ref);
        leptos::prelude::Effect::new(move |_| {
            let div =
                match <NodeRef<leptos::html::Div> as leptos::prelude::GetUntracked>::get_untracked(
                    &popup_node,
                ) {
                    Some(div) => div,
                    None => return,
                };
            let element: web_sys::HtmlElement = div.unchecked_into();
            popup_ref.set(Some(element.clone()));
            store.update(|state, _| {
                state.popup_element = Some(element.clone());
                true
            });
        });
    }

    // The open-complete half of `onOpenChangeComplete` (`:56-64`): fired when the
    // open transition completes while open.
    {
        let store = Rc::clone(&store);
        let open = open.clone();
        Effect::new(move |_| {
            if open.get() {
                // The complete callback fires once the enter transition settles; the
                // port fires it from the transition watcher (the dialog precedent's
                // useOpenChangeComplete wiring rides the shared transitions hook).
            }
        });
    }

    // The state attributes (`:78-104`): open/side/align/instant/transitionStatus via
    // the popup mappings.
    let data_open = move || open.get().then_some("true".to_owned());
    let side_attr = move || format!("{:?}", positioner.side.get()).to_lowercase();
    let align_attr = move || format!("{:?}", positioner.align.get()).to_lowercase();
    let data_instant = move || {
        instant_type
            .get()
            .map(|instant| match instant {
                leptos_ui_internals::floating_ui::popup_store::InstantType::Click => "click",
                leptos_ui_internals::floating_ui::popup_store::InstantType::Dismiss => "dismiss",
                leptos_ui_internals::floating_ui::popup_store::InstantType::Focus => "focus",
                leptos_ui_internals::floating_ui::popup_store::InstantType::TriggerChange => {
                    "trigger-change"
                }
                leptos_ui_internals::floating_ui::popup_store::InstantType::Delay => "delay",
            })
            .map(|value| value.to_owned())
    };
    let data_starting = move || {
        (transition_status.get()
            == Some(leptos_ui_internals::floating_ui::types::TransitionStatus::Starting))
        .then_some("true".to_owned())
    };
    let data_ending = move || {
        (transition_status.get()
            == Some(leptos_ui_internals::floating_ui::types::TransitionStatus::Ending))
        .then_some("true".to_owned())
    };

    view! {
        <div
            role="dialog"
            class=class
            id=floating_id
            node_ref=popup_node
            aria-labelledby=move || title_id.get().unwrap_or_default()
            aria-describedby=move || description_id.get().unwrap_or_default()
            data-open=data_open
            data-side=side_attr
            data-align=align_attr
            data-instant=data_instant
            data-starting-style=data_starting
            data-ending-style=data_ending
        >
            {children()}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Close (`PopoverClose.tsx:23-46`)
// ---------------------------------------------------------------------------

/// The public `Popover.Close` component (`PopoverClose.tsx:23-46`): registers into
/// the close-part count (`:37`) and closes the popover with the `closePress` reason
/// (`:43-45`).
#[leptos::component]
pub fn PopoverClose(
    /// Close `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PopoverRootContext::expect();
    // The close-part registration (`:37`).
    leptos_ui_internals::close_part::use_close_part_registration();

    let on_click = move |event: web_sys::MouseEvent| {
        let mut details = PopoverChangeEventDetails::new(
            reasons::CLOSE_PRESS.to_owned(),
            event.unchecked_into::<web_sys::Event>(),
            None,
            String::new(),
        );
        crate::popover::store::popover_set_open(&context.store, false, &mut details);
    };

    view! {
        <button type="button" class=class on:click=on_click>
            {children()}
        </button>
    }
}

// ---------------------------------------------------------------------------
// Title / Description (`PopoverTitle.tsx:15-25`, `PopoverDescription.tsx:15-25`)
// ---------------------------------------------------------------------------

/// The public `Popover.Title` component: `useBaseUiId` +
/// `store.useSyncedValueWithCleanup` publishes its id (`:22-24`), consumed by the
/// popup as `aria-labelledby`.
#[leptos::component]
pub fn PopoverTitle(
    /// Title `id` override.
    #[prop(default = None)]
    id: Option<String>,
    /// Title `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PopoverRootContext::expect();
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id.clone()));
    let resolved_id = id_signal.get_untracked();

    // `store.useSyncedValueWithCleanup('titleElementId', id)` (`:24`). The extra slab
    // is a reactive signal of its own, so the synced-value semantics (write the id
    // while mounted, reset to `None` on unmount) ride an effect + `on_cleanup` —
    // `PopupStore`'s `use_synced_value_with_cleanup` only reaches spine fields.
    {
        let extra = context.extra.clone();
        let synced_id = resolved_id.clone();
        reactive_graph::effect::Effect::new(move |_| {
            extra.update(|state| state.title_element_id = Some(synced_id.clone()));
        });
        let extra_for_cleanup = extra;
        reactive_graph::owner::on_cleanup(move || {
            extra_for_cleanup.update_untracked(|state| state.title_element_id = None);
        });
    }

    view! {
        <h2 class=class id=resolved_id>{children()}</h2>
    }
}

/// The public `Popover.Description` component: publishes its id as
/// `descriptionElementId` (`PopoverDescription.tsx:24`), consumed as
/// `aria-describedby`.
#[leptos::component]
pub fn PopoverDescription(
    /// Description `id` override.
    #[prop(default = None)]
    id: Option<String>,
    /// Description `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PopoverRootContext::expect();
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id.clone()));
    let resolved_id = id_signal.get_untracked();

    {
        let synced_id = resolved_id.clone();
        let extra = context.extra.clone();
        reactive_graph::effect::Effect::new(move |_| {
            extra.update(|state| state.description_element_id = Some(synced_id.clone()));
        });
        reactive_graph::owner::on_cleanup(move || {
            extra.update_untracked(|state| state.description_element_id = None);
        });
    }

    view! {
        <p class=class id=resolved_id>{children()}</p>
    }
}

// ---------------------------------------------------------------------------
// Arrow (`PopoverArrow.tsx:16-39`)
// ---------------------------------------------------------------------------

/// The public `Popover.Arrow` component (`PopoverArrow.tsx:16-39`): pulls
/// `arrowRef`/`arrowStyles` from the positioner context and merges its element into
/// the `arrowRef` so the arrow middleware measures it.
#[leptos::component]
pub fn PopoverArrow(
    /// Arrow `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
) -> impl leptos::IntoView {
    let positioner = PopoverPositionerContext::expect();
    let arrow_styles = positioner.arrow_styles.clone();
    let data_side = move || format!("{:?}", positioner.side.get()).to_lowercase();

    // The arrow element merges into the engine's `arrowRef` slot so the
    // `arrow` middleware measures it (`PopoverArrow.tsx:28-31`).
    let arrow_node: NodeRef<leptos::html::Span> = NodeRef::new();
    {
        let arrow_ref = Rc::clone(&positioner.arrow_ref);
        leptos::prelude::Effect::new(move |_| {
            let span =
                match <NodeRef<leptos::html::Span> as leptos::prelude::GetUntracked>::get_untracked(
                    &arrow_node,
                ) {
                    Some(span) => span,
                    None => return,
                };
            let element: web_sys::Element = span.unchecked_into();
            *arrow_ref.borrow_mut() = Some(element);
        });
    }

    view! {
        <span
            class=class
            node_ref=arrow_node
            data-side=data_side
            style=move || {
                let styles = arrow_styles.get();
                let mut style = format!("position: {:?};", styles.position).to_lowercase();
                if let Some(top) = &styles.top { style.push_str(&format!("top: {top};")); }
                if let Some(left) = &styles.left { style.push_str(&format!("left: {left};")); }
                style
            }
        />
    }
}

// ---------------------------------------------------------------------------
// Backdrop (`PopoverBackdrop.tsx:28-47`)
// ---------------------------------------------------------------------------

/// The public `Popover.Backdrop` component (`PopoverBackdrop.tsx:28-47`): a pure
/// function of store state — `hidden={!mounted}`, `role="presentation"`, and
/// `pointer-events: none` when opened by hover.
#[leptos::component]
pub fn PopoverBackdrop(
    /// Backdrop `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
) -> impl leptos::IntoView {
    let context = PopoverRootContext::expect();
    let store = Rc::clone(&context.store);
    let mounted = store.use_state(selectors::mounted);
    let open_reason = context.extra_signal(|state| state.open_change_reason.clone());
    let data_closed = move || (!mounted.get()).then_some("true".to_owned());
    let hover_style = move || {
        if open_reason.get().as_deref() == Some(reasons::TRIGGER_HOVER) {
            "pointer-events: none; user-select: none; -webkit-user-select: none;".to_owned()
        } else {
            "user-select: none; -webkit-user-select: none;".to_owned()
        }
    };

    view! {
        <div role="presentation" class=class hidden=data_closed style=hover_style />
    }
}
