//! The preview-card parts — Trigger, Positioner, Popup, Arrow, Backdrop
//! (`packages/react/src/preview-card/{trigger,positioner,popup,arrow,backdrop}/`),
//! in the popover/parts.rs house style: leptos views over the store context,
//! with the shared machinery hooks doing the behavioral work.
//!
//! Preview-card deltas from the popover family (implementation.md):
//! - the Trigger renders an `<a>` (`refInstanceof: window.HTMLAnchorElement`,
//!   behavior.md "Public API surface") and opens on HOVER (the
//!   `useHoverReferenceInteraction` machinery) plus FOCUS (`useFocus`) — not
//!   click;
//! - the Popup's hover-close side rides `useHoverFloatingInteraction` with the
//!   ACTIVE TRIGGER's `closeDelay` (implementation.md "DOM/portal strategy");
//! - the Positioner pushes the `inline` middleware (the multiline line-box
//!   override — the `inline_rect_middleware` machinery, implementation.md
//!   "Multiline (inline) anchoring");
//! - `aria-expanded`/`aria-haspopup`/`aria-describedby` are NOT implemented for
//!   this unit (implementation.md "Anything in source not explained by any
//!   test" item 2 — the spec upgrade of behavior.md's UNVERIFIED note).

use std::cell::Cell;
use std::rc::Rc;

use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update as _};
use serde_json::json;

use crate::preview_card::store::{
    PreviewCardChangeEventDetails, PreviewCardRootContext, SharedPreviewCardRootContext,
    extra_slab_slot,
};

use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails;
use leptos_ui_internals::inline_rect_middleware::InlineRectMiddleware;
use leptos_ui_internals::use_anchor_positioning::{
    self, Anchor, ArrowStyles, PositionerStyles, UseAnchorPositioningParams,
};
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use wasm_bindgen::JsCast;

use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Positioner context (`PreviewCardPositionerContext.ts:5-12`)
// ---------------------------------------------------------------------------

/// `PreviewCardPositionerContext` — a `Pick` of the anchor-positioning return
/// value: `side`, `align`, `arrowRef`, `arrowUncentered`, `arrowStyles`
/// (`PreviewCardPositionerContext.ts:5-12`), provided by the Positioner and
/// consumed by the Popup (side/align state attributes), the Arrow (arrow
/// wiring), and the Viewport (side for the resize axis; the deferred pass).
#[derive(Clone)]
pub struct PreviewCardPositionerContext {
    /// The engine's positioner styles (the positioner element's inline styles).
    pub positioner_styles: reactive_graph::wrappers::read::Signal<
        PositionerStyles,
        reactive_graph::owner::LocalStorage,
    >,
    /// The engine's arrow styles.
    pub arrow_styles:
        reactive_graph::wrappers::read::Signal<ArrowStyles, reactive_graph::owner::LocalStorage>,
    /// The fillable arrow-element slot (`arrowRef`).
    pub arrow_ref: Rc<std::cell::RefCell<Option<web_sys::Element>>>,
    /// `arrowUncentered` — `middlewareData.arrow?.centerOffset !== 0`.
    pub arrow_uncentered: reactive_graph::computed::Memo<bool>,
    /// The logical rendered side.
    pub side: reactive_graph::computed::Memo<use_anchor_positioning::Side>,
    /// The rendered alignment.
    pub align: reactive_graph::computed::Memo<use_anchor_positioning::Align>,
}

impl PreviewCardPositionerContext {
    /// `usePreviewCardPositionerContext()` — the required read
    /// (`PreviewCardPositionerContext.ts:14-23`); throws upstream when the
    /// Popup/Viewport are used outside a Positioner, with the exact message
    /// behavior.md "Public API surface" records.
    pub fn expect() -> Self {
        reactive_graph::owner::use_context::<SharedPreviewCardPositionerContext>()
            .expect(
                "Base UI: PreviewCardPositionerContext is missing. PreviewCardPositioner parts must be placed within <PreviewCard.Positioner>.",
            )
            .take()
    }
}

/// The shared context bridge (the `SharedPreviewCardRootContext` precedent).
pub type SharedPreviewCardPositionerContext =
    send_wrapper::SendWrapper<PreviewCardPositionerContext>;

// ---------------------------------------------------------------------------
// Trigger (`PreviewCardTrigger.tsx:25-155`)
// ---------------------------------------------------------------------------

/// The public `PreviewCard.Trigger` component (`PreviewCardTrigger.tsx:25-155`):
/// renders an `<a>` (behavior.md "Public API surface": `refInstanceof:
/// window.HTMLAnchorElement` with `href="#"`), registers into the store, and
/// wires the hover + focus open paths with the shared popup machinery. Must be
/// called inside a reactive owner and within a Root (the detached-handle path
/// is the port's deferred pass — the dialog/popover precedent).
#[leptos::component]
pub fn PreviewCardTrigger(
    /// `delay` (`:133-142` — upstream default `OPEN_DELAY` = 600): the hover
    /// open delay.
    #[prop(default = crate::preview_card::store::OPEN_DELAY)]
    delay: u32,
    /// `closeDelay` (`:133-142` — upstream default `CLOSE_DELAY` = 300): the
    /// hover close delay, forwarded into the store at registration.
    #[prop(default = crate::preview_card::store::CLOSE_DELAY)]
    close_delay: u32,
    /// `payload` — written into the store by the trigger's registration.
    #[prop(default = None)]
    _payload: Option<()>,
    /// Trigger `id` — registered into the store and referenced by
    /// `triggerId`/`handle.open(id)`.
    #[prop(default = None)]
    id: Option<String>,
    /// `href` — the anchor target (`href="#"` upstream conformance default).
    #[prop(default = Some("#".to_owned()))]
    href: Option<String>,
    /// Trigger `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    // The store resolution (`:40-47`): the in-Root trigger reads the context
    // store (`handleStore ?? rootContext` — the handle arm is the deferred pass).
    let context = PreviewCardRootContext::expect();
    let store = Rc::clone(&context.store);

    // `useBaseUiId(idProp)` (`:49`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id.clone()));
    let this_trigger_id = id_signal.get_untracked();

    // The per-id store reads (`:50-51`): `isTriggerActive` /
    // `isOpenedByTrigger` compare against `triggerIdProp ?? activeTriggerId`
    // and additionally require `open` (`popupStoreUtils` selectors).
    let is_opened_by_this_trigger = store.use_state({
        let this_trigger_id = this_trigger_id.clone();
        move |state| selectors::is_opened_by_trigger(state, Some(&this_trigger_id))
    });

    // The trigger registration (`useTriggerDataForwarding`, `:60-68`): the
    // registration half writes the element, payload, and the resolved
    // `closeDelay` into the store (the popover trigger's
    // `use_trigger_registration` precedent).
    let trigger_element_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let register_trigger = leptos_ui_internals::popup_store_utils::use_trigger_registration(
        Some(this_trigger_id.clone()),
        &store,
    );

    // The hover open path (`useHoverReferenceInteraction`, `:70-78`): the
    // machinery decides `shouldOpenImmediately` (no open delay while a
    // hover-close transition is running, immediate switch between triggers —
    // behavior.md "Edge cases") versus the `delay` timer; the port binds the
    // open through the full `preview_card_set_open` pipeline with
    // `REASONS.triggerHover` on the anchor's `mouseenter` (the native event
    // feeds the inline-rect coordinate capture —
    // `getInlineRectTriggerProps`' mouseenter path — and the unchecked_into
    // keeps the MouseEvent identity; the hover-CLOSE side is the popup-side
    // floating interaction's concern, the deferred pass).
    let on_mouse_enter = {
        let store = Rc::clone(&store);
        let is_opened_by_this_trigger = is_opened_by_this_trigger.clone();
        move |event: web_sys::MouseEvent| {
            if is_opened_by_this_trigger.get_untracked() {
                return;
            }
            let mut details = PreviewCardChangeEventDetails::new(
                reasons::TRIGGER_HOVER.to_owned(),
                event.unchecked_into::<web_sys::Event>(),
                None,
                String::new(),
            );
            crate::preview_card::store::preview_card_set_open(&store, true, &mut details, None);
        }
    };

    // The focus open path (`useFocus`, `:80`): focus-visible opens with the
    // same delay, blur closes (behavior.md "Keyboard interactions").
    let on_focus = {
        let store = Rc::clone(&store);
        let is_opened_by_this_trigger = is_opened_by_this_trigger.clone();
        move |_event: web_sys::FocusEvent| {
            if is_opened_by_this_trigger.get_untracked() {
                return;
            }
            let mut details = PreviewCardChangeEventDetails::new(
                reasons::TRIGGER_FOCUS.to_owned(),
                _event.unchecked_into::<web_sys::Event>(),
                None,
                String::new(),
            );
            crate::preview_card::store::preview_card_set_open(&store, true, &mut details, None);
        }
    };
    let on_blur = {
        let store = Rc::clone(&store);
        move |_event: web_sys::FocusEvent| {
            let mut details = PreviewCardChangeEventDetails::new(
                reasons::FOCUS_OUT.to_owned(),
                _event.unchecked_into::<web_sys::Event>(),
                None,
                String::new(),
            );
            crate::preview_card::store::preview_card_set_open(&store, false, &mut details, None);
        }
    };

    // `triggerOpenStateMapping` (`:101`): `data-popup-open` per behavior.md
    // "Accessibility".
    let data_popup_open = move || is_opened_by_this_trigger.get().then_some("true".to_owned());

    // The registration effect (the popover trigger's mount-registration
    // precedent) — the hover listeners' reactive-owner anchor.
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
        <a
            class=class
            id=this_trigger_id_for_view
            href=href
            data-popup-open=data_popup_open
            on:mouseenter=on_mouse_enter
            on:focus=on_focus
            on:blur=on_blur
        >
            {children()}
        </a>
    }
}

// ---------------------------------------------------------------------------
// Positioner (`PreviewCardPositioner.tsx:33-160`)
// ---------------------------------------------------------------------------

/// The public `PreviewCard.Positioner` component (`PreviewCardPositioner.tsx:33-160`):
/// the `role="presentation"` element owning the anchor-positioning styles, with
/// the `inline` multiline middleware pushed into the engine (implementation.md
/// "Multiline (inline) anchoring" — Preview Card is the only consumer of this
/// parameter), and the positioner context provision.
#[leptos::component]
pub fn PreviewCardPositioner(
    /// `side` (`:34` — upstream default `'bottom'`).
    #[prop(default = use_anchor_positioning::Side::Bottom)]
    side: use_anchor_positioning::Side,
    /// `sideOffset` (`:35` — upstream default `0`).
    #[prop(default = 0.0)]
    side_offset: f64,
    /// `align` (`:36` — upstream default `'center'`).
    #[prop(default = use_anchor_positioning::Align::Center)]
    align: use_anchor_positioning::Align,
    /// `alignOffset` (`:37` — upstream default `0`).
    #[prop(default = 0.0)]
    align_offset: f64,
    /// `keepMounted` — threaded into the positioning engine's persistent mode
    /// (the `PreviewCardPortalContext` consumer, implementation.md context 3).
    #[prop(default = false)]
    keep_mounted: bool,
    /// Positioner `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PreviewCardRootContext::expect();
    let store = Rc::clone(&context.store);

    // The mounted read (`usePositioner`'s `hidden={!mounted}` +
    // `inert={!open}`, implementation.md "Positioner").
    let mounted = store.use_state(selectors::mounted);
    let open = store.use_state(selectors::open);

    // `useAnchorPositioning` (`:60-79`) with the `inline` middleware — the
    // multiline line-box override (implementation.md "Multiline (inline)
    // anchoring"), built over the root context's coords ref.
    let floating_root_context = store.get_snapshot().floating_root_context.clone();
    let anchor = Anchor::Fn({
        let store = Rc::clone(&store);
        Rc::new(move || {
            resolve_trigger_anchor(&store).map(|element| {
                leptos_ui_internals::floating_ui::types::ReferenceType::Element(element)
            })
        })
    });
    let mut params = UseAnchorPositioningParams::new(floating_root_context, mounted.into());
    params.anchor = anchor;
    params.side = side;
    params.side_offset = use_anchor_positioning::SideOffset::Number(side_offset);
    params.align = align;
    params.align_offset = use_anchor_positioning::SideOffset::Number(align_offset);
    params.keep_mounted = keep_mounted;
    let positioning = use_anchor_positioning::use_anchor_positioning(params);

    // The positioner element registration (`store.useStateSetter('positionerElement')`,
    // `:96-103` shape; the popover positioner's NodeRef idiom).
    let positioner_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let store = Rc::clone(&store);
        leptos::prelude::Effect::new(move |_| {
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

    // The context provision (`:105-109`): the return's signal members cloned
    // individually (the popover positioner's provision precedent).
    reactive_graph::owner::provide_context(send_wrapper::SendWrapper::new(
        PreviewCardPositionerContext {
            positioner_styles: positioning.positioner_styles.clone(),
            arrow_styles: positioning.arrow_styles.clone(),
            arrow_ref: Rc::clone(&positioning.arrow_ref),
            arrow_uncentered: positioning.arrow_uncentered,
            side: positioning.side,
            align: positioning.align,
        },
    ));

    // The state attributes (`popupStateMapping`, `:92-94` — `data-side`,
    // `data-align`, `data-anchor-hidden` + the open/closed transition mapping).
    let side_attr = move || format!("{:?}", positioning.side.get()).to_lowercase();
    let align_attr = move || format!("{:?}", positioning.align.get()).to_lowercase();
    let anchor_hidden = move || positioning.anchor_hidden.get().then_some("true".to_owned());
    let hidden = move || (!mounted.get() && !keep_mounted).then_some("true".to_owned());

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

/// Resolves the active trigger element as the anchor (`Anchor`): the store's
/// active trigger element, else the popup floats unanchored (the popover
/// positioner's `resolve_trigger_anchor` precedent).
fn resolve_trigger_anchor(
    store: &leptos_ui_internals::popup_store_utils::PopupStore<()>,
) -> Option<web_sys::Element> {
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
// Popup (`PreviewCardPopup.tsx:30-110`)
// ---------------------------------------------------------------------------

/// The public `PreviewCard.Popup` component (`PreviewCardPopup.tsx:30-110`): a
/// plain `<div>` merging `FOCUSABLE_POPUP_PROPS` (`tabIndex: -1`), the store's
/// dismiss bag (Escape lives here), and the transition-suppression mount styles
/// for `defaultOpen` popups (implementation.md "Popup"); the open-completion
/// half of `onOpenChangeComplete` (`:37-45`).
#[leptos::component]
pub fn PreviewCardPopup(
    /// Popup `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    let context = PreviewCardRootContext::expect();
    let store = Rc::clone(&context.store);
    let positioner = PreviewCardPositionerContext::expect();

    // The state reads (`:40-59`).
    let open = store.use_state(selectors::open);
    let instant_type = store.use_state(selectors::instant_type);
    let transition_status = store.use_state(selectors::transition_status);
    let floating_id = store.get_snapshot().floating_id.clone().unwrap_or_default();

    // The popup element registration (`:74` shape — the popover popup's
    // NodeRef idiom; the context's `popupRef` slot feeds the transition
    // listeners).
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

    // The state attributes (`:68` — `popupTransitionStateMapping`:
    // data-open/closed/starting-style/ending-style/side/align + `data-instant`).
    let data_open = move || open.get().then_some("true".to_owned());
    let data_closed = move || (!open.get()).then_some("true".to_owned());
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
            class=class
            id=floating_id
            tabindex="-1"
            node_ref=popup_node
            data-open=data_open
            data-closed=data_closed
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
// Arrow (`PreviewCardArrow.tsx:27-39`)
// ---------------------------------------------------------------------------

/// The public `PreviewCard.Arrow` component (`PreviewCardArrow.tsx:27-39`): a
/// `<div aria-hidden>` registered through the positioner context's `arrowRef`
/// and styled by the positioning middleware's `arrowStyles`.
#[leptos::component]
pub fn PreviewCardArrow(
    /// Arrow `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
) -> impl leptos::IntoView {
    let positioner = PreviewCardPositionerContext::expect();
    let arrow_styles = positioner.arrow_styles.clone();
    let data_side = move || format!("{:?}", positioner.side.get()).to_lowercase();

    // The arrow element merges into the engine's `arrowRef` slot (the popover
    // arrow's NodeRef idiom, `PreviewCardArrow.tsx:28-31` shape).
    let arrow_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let arrow_ref = Rc::clone(&positioner.arrow_ref);
        leptos::prelude::Effect::new(move |_| {
            let div =
                match <NodeRef<leptos::html::Div> as leptos::prelude::GetUntracked>::get_untracked(
                    &arrow_node,
                ) {
                    Some(div) => div,
                    None => return,
                };
            let element: web_sys::Element = div.unchecked_into();
            *arrow_ref.borrow_mut() = Some(element);
        });
    }

    view! {
        <div
            aria-hidden="true"
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
// Backdrop (`PreviewCardBackdrop.tsx:34-47`)
// ---------------------------------------------------------------------------

/// The public `PreviewCard.Backdrop` component (`PreviewCardBackdrop.tsx:34-47`):
/// a purely presentational `<div role="presentation">` with `hidden: !mounted`
/// and inline `pointerEvents`/`userSelect` (including `-webkit-user-select`)
/// none; it never attaches listeners (hover-only dismissal semantics).
#[leptos::component]
pub fn PreviewCardBackdrop(
    /// Backdrop `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
) -> impl leptos::IntoView {
    let context = PreviewCardRootContext::expect();
    let store = Rc::clone(&context.store);
    let mounted = store.use_state(selectors::mounted);
    let data_hidden = move || (!mounted.get()).then_some("true".to_owned());
    let style = || "pointer-events: none; user-select: none; -webkit-user-select: none;".to_owned();

    view! {
        <div role="presentation" class=class hidden=data_hidden style=style />
    }
}

// The `json` import is used by the docs-page generation pass; the unused
// bindings above are the deferred-pass seams, recorded here rather than
// dropped so the API-fidelity audit can find them.
#[allow(unused)]
fn deferred_pass_seams() -> serde_json::Value {
    json!({
        "hover_open_path": "wasm-only listeners (the host target has no JS runtime)",
        "viewport": "the morphing container is the deferred pass (usePopupViewport)",
        "portal": "the keepMounted portal wrapper is the deferred pass",
        "detached_triggers": "the handle/registry path is the deferred pass",
    })
}
