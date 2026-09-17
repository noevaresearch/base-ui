//! Menu arrow — `Menu.Arrow`, port of `packages/react/src/menu/arrow/MenuArrow.tsx`.
//!
//! WHAT THIS REPLACES. The previous `arrow.rs` was a facade, not a port: it rendered a
//! hardcoded `<div class="menu-arrow">` wrapping an invented `<div class="menu-arrow-inner">`,
//! emitted `data-arrow`/`data-arrow-hidden` (attributes upstream has no such names for — its
//! set is [`crate::menu::utils`]'s `popupStateMapping` plus `data-uncentered`,
//! `MenuArrowDataAttributes.ts`), and exported four "hooks"
//! (`use_menu_arrow_props`, `use_menu_arrow_hidden`, `use_menu_arrow_style`) that return
//! constants and have no upstream counterpart at all. A hardcoded class string is the
//! "uniform DOM shell" `specs/library/menu/behavior.md` → "Uniform DOM shell" forbids.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from
//! (`MenuArrow.tsx`, 49 lines total):
//! - the two context reads — the store (`:21`, for `open`) and the required positioner
//!   context (`:22`, `{ arrowRef, side, align, arrowUncentered, arrowStyles }`), which
//!   throws outside a `<Menu.Positioner>` (`MenuPositionerContext.ts:12-21`).
//! - the state record `{ open, side, align, uncentered }` (`:25-30`).
//! - the state → `data-*` mapping: `stateAttributesMapping: popupStateMapping` (`:35`), so
//!   the element carries `data-open`/`data-closed` (`popupStateMapping.ts:53-66`) plus
//!   `data-side`/`data-align`/`data-uncentered` from the default handling
//!   (`getStateAttributesProps.ts:22-31`; the three names are
//!   `MenuArrowDataAttributes.ts:10-24`).
//! - the injected props (`:37-41`): `style: arrowStyles` and `aria-hidden: true`, with the
//!   consumer's own bag landing after them (upstream's `...elementProps` spread) so a
//!   consumer member wins — the merge order `crates/leptos-ui/src/input.rs` documents for
//!   the same spread.
//! - the ref merge `[arrowRef, forwardedRef]` (`:34`) — the element fills the engine's
//!   `arrowRef` slot so the arrow middleware can measure it (`useAnchorPositioning`'s
//!   `arrowRef`, the popover part's precedent at `popover/parts.rs:646-662`).
//!
//! DEFERRED, with the reason, none of it silently dropped:
//! - `render` (`:17`, the element-substitution prop) — the crate-wide
//!   `library: the view paths drop render's element form` item; the same deferral every
//!   ported part carries.
//! - the forwarded ref (`:19`, `forwardedRef`) — Leptos has no `forwardRef`: a consumer
//!   reaches the node with `NodeRef` on their own wrapper (the crate's `ref_callback`
//!   convention where a part exposes one, `otp_field.rs:1924`). The arrow's own
//!   `arrowRef` half IS ported (it is the middleware's contract, not the consumer's).

use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::popup_state_mapping::popup_state_mapping;
use leptos_ui_internals::state_attributes::get_state_attributes_props;
use leptos_ui_internals::use_anchor_positioning::{Align, ArrowStyles, Side};
use reactive_graph::traits::Get;
use serde_json::{Map, Value};

use crate::menu::positioner::{align_attr, menu_positioner_context, side_attr};
use crate::menu::store::use_menu_store;

/// `MenuArrow.Props` (`MenuArrow.tsx:17` over `BaseUIComponentProps<'div', MenuArrowState>`):
/// the consumer's element props. The part injects its own `style`/`aria-hidden` ahead of
/// this bag (module docs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuArrowProps {
    /// `className` (`MenuArrow.tsx:20`).
    pub class: Option<String>,
    /// `style` (`:20`) — the consumer's own style members, landing after the engine's
    /// computed arrow styles (module docs).
    pub style: Vec<(String, String)>,
}

/// `MenuArrowState` (`MenuArrow.tsx:44-62`) — the state record the mapping consumes
/// (`:25-30`), in the shape `getStateAttributesProps` reads (`getStateAttributesProps.ts:5-31`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuArrowState {
    /// `open` (`:48`) — the store's open state (`:24`).
    pub open: bool,
    /// `side` (`:52`) — the rendered side from the positioner context (`:22`).
    pub side: Side,
    /// `align` (`:56`) — the rendered alignment from the positioner context (`:22`).
    pub align: Align,
    /// `uncentered` (`:60`) — the positioner's `arrowUncentered` (`:22`, `:29`).
    pub uncentered: bool,
}

/// `MenuArrow.tsx:25-30` — the state record as the mapping's input map.
///
/// `side`/`align` are inserted with their rendered spellings ([`side_attr`]/[`align_attr`],
/// the popup part's convention) so the default handling emits `data-side="inline-end"`
/// rather than the Rust enum's own name; the key names are upstream's
/// (`getStateAttributesProps` only lowercases, `state_attributes.rs:153`).
pub fn menu_arrow_state(
    open: bool,
    side: Side,
    align: Align,
    uncentered: bool,
) -> Map<String, Value> {
    let mut state = Map::new();
    state.insert("open".to_owned(), Value::Bool(open));
    state.insert("side".to_owned(), Value::String(side_attr(side).to_owned()));
    state.insert(
        "align".to_owned(),
        Value::String(align_attr(align).to_owned()),
    );
    // `uncentered` stays a boolean: the default handling emits a bare `data-uncentered`
    // while true and nothing while false (`state_attributes.rs:152-156`),
    // which is what `MenuArrowDataAttributes.ts:24` documents.
    state.insert("uncentered".to_owned(), Value::Bool(uncentered));
    state
}

/// The arrow element's `data-*` attributes: `getStateAttributesProps(state, popupStateMapping)`
/// (`MenuArrow.tsx:35`). The ported producer already returns FINAL attribute names, so the
/// pairs are returned as they come; the element below binds this same set and a host test
/// pins the mapping's output so a silent divergence is a test failure rather than a
/// rendering mystery (the popup part's convention, `popup.rs:193-203`).
pub fn menu_arrow_attributes(state: &Map<String, Value>) -> Vec<(String, String)> {
    get_state_attributes_props(state, Some(&popup_state_mapping))
        .into_iter()
        .collect()
}

/// `MenuArrow.tsx:37-41` — the element's inline style: the engine's computed arrow styles
/// (`arrowStyles`, `useAnchorPositioning`'s `{ position, top, left }`) followed by the
/// consumer's own members, so a consumer member wins (the spread order, module docs).
///
/// The `position` value is rendered verbatim: `ArrowStyles.position` is already the CSS
/// keyword (`use_anchor_positioning.rs:546`).
pub fn menu_arrow_style(styles: &ArrowStyles, consumer: &[(String, String)]) -> String {
    let mut style = format!("position: {};", styles.position);
    if let Some(top) = &styles.top {
        style.push_str(&format!("top: {top};"));
    }
    if let Some(left) = &styles.left {
        style.push_str(&format!("left: {left};"));
    }
    for (property, value) in consumer {
        style.push_str(&format!("{property}: {value};"));
    }
    style
}

/// The resolved arrow element description — the members `useRenderElement('div', …)`
/// (`MenuArrow.tsx:32-42`) puts on the element, in the crate's attribute vocabulary.
/// Host-testable: the resolution carries no DOM access, so the arrow's contract can be
/// asserted without a browser, which this box refuses (`ralph/generated/env-health.json` →
/// `browser: DEGRADED`; the rendered axes are CI's).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuArrowResolved {
    /// The `data-*` set ([`menu_arrow_attributes`]).
    pub attributes: Vec<(String, String)>,
    /// `'aria-hidden': true` (`MenuArrow.tsx:39`) — the arrow is decorative.
    pub aria_hidden: bool,
    /// The inline style ([`menu_arrow_style`]).
    pub style: String,
}

/// Resolves the arrow element description (`MenuArrow.tsx:32-42`). Pure apart from the
/// engine's own style record the caller passes in.
pub fn resolve_menu_arrow(
    state: MenuArrowState,
    styles: &ArrowStyles,
    consumer_style: &[(String, String)],
) -> MenuArrowResolved {
    let state_map = menu_arrow_state(state.open, state.side, state.align, state.uncentered);
    MenuArrowResolved {
        attributes: menu_arrow_attributes(&state_map),
        // `'aria-hidden': true` (`:39`) — injected ahead of the consumer's bag.
        aria_hidden: true,
        style: menu_arrow_style(styles, consumer_style),
    }
}

/// `Menu.Arrow` — upstream's `MenuArrow` (`MenuArrow.tsx:17-42`).
///
/// Renders the `<div>` the arrow middleware measures: `aria-hidden`, the engine's computed
/// arrow styles, and the popup state attributes. See the module docs for what is ported and
/// what the crate-wide items own instead.
#[component]
pub fn Arrow(
    /// `className` (`:20`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:20`) — the consumer's members, applied after the engine's (module docs).
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The element's content — upstream keeps children inside the rendered `<div>`
    /// (`componentProps`'s `elementProps` spread, `:41`), which is how a consumer draws the
    /// arrow with an `<svg>`.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    // `const { store } = useMenuRootContext()` (`:21`).
    let context = use_menu_store();
    let store = context.store;

    // `const { arrowRef, side, align, arrowUncentered, arrowStyles } = useMenuPositionerContext()`
    // (`:22`) — the required read, which throws outside a Positioner
    // (`MenuPositionerContext.ts:12-21`).
    let positioner = menu_positioner_context(false)
        .expect("menu_positioner_context(false) panics when absent");

    // `const open = store.useState('open')` (`:24`).
    let open = store.use_state(selectors::open);

    // `ref: [arrowRef, forwardedRef]` (`:34`) — the element fills the engine's `arrowRef`
    // slot so the `arrow` middleware can measure it. The popover part's registration
    // (`popover/parts.rs:646-662`).
    let arrow_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let arrow_ref = Rc::clone(&positioner.arrow_ref);
        Effect::new(move |_| {
            // `NodeRef`'s untracked read is leptos's own trait (the 0.7 surface), not
            // `reactive_graph 0.2`'s — the version split the popup part's identical
            // UFCS spelling documents (`popup.rs:289-291`).
            let div = match <NodeRef<leptos::html::Div> as leptos::prelude::GetUntracked>::get_untracked(&arrow_node)
            {
                Some(div) => div,
                None => return,
            };
            let element: web_sys::Element = web_sys::wasm_bindgen::JsCast::unchecked_into(div);
            *arrow_ref.borrow_mut() = Some(element);
        });
    }

    let arrow_styles = positioner.arrow_styles.clone();
    let side = positioner.side;
    let align = positioner.align;
    let uncentered = positioner.arrow_uncentered;

    let class = class.unwrap_or_default();

    view! {
        <div
            class=class
            node_ref=arrow_node
            aria-hidden="true"
            style=move || menu_arrow_style(&arrow_styles.get(), &style)
            data-open=move || open.get().then_some("true")
            data-closed=move || (!open.get()).then_some("true")
            data-side=move || side_attr(side.get())
            data-align=move || align_attr(align.get())
            data-uncentered=move || uncentered.get().then_some("true")
        >
            {children.map(|children| children())}
        </div>
    }
}
