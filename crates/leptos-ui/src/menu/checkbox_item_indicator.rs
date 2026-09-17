//! Menu checkbox item indicator — `Menu.CheckboxItemIndicator`, the port of
//! `packages/react/src/menu/checkbox-item-indicator/MenuCheckboxItemIndicator.tsx` (94
//! lines) and `MenuCheckboxItemIndicatorDataAttributes.ts` (22 lines).
//!
//! WHAT THIS REPLACES. The previous file (spelled `checkbox-item-indicator.rs`) was a
//! facade: never declared (a hyphen is not a legal Rust identifier), rendering a shell with
//! `create_rw_signal(false)` in place of the context read — its own comments said "In a real
//! implementation, this would read the checked state from context" and "this would listen
//! for open change events" — plus an `Effect::new` whose body did nothing. It carried one
//! prop (`style: Option<String>`) where upstream's bag is the consumer's `...elementProps`.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the props (`MenuCheckboxItemIndicator.tsx:20`): `keepMounted` (default `false`, `:69`)
//!   and the consumer's `render`/`className`/`style`/`...elementProps` members.
//! - the required context read (`:22`): `useMenuCheckboxItemContext()` throws upstream's
//!   own message outside a `Menu.CheckboxItem`
//!   (`MenuCheckboxItemIndicator.test.tsx:31-41`).
//! - the mount machine (`:26-38`): `useTransitionStatus(item.checked)` — with
//!   `enableIdleState`/`deferEndingState` at their `false` defaults
//!   (`useTransitionStatus.ts:92-96`) — plus `useOpenChangeComplete({ batch: true,
//!   enabled: !item.checked, open: item.checked, ref: indicatorRef, onComplete: if
//!   (!item.checked) setMounted(false) })`.
//! - the state record (`:40-45`): `{ checked, disabled, highlighted, transitionStatus }`,
//!   which is what `itemMapping` (`:50`) walks — `data-checked`/`data-unchecked`
//!   (`MenuCheckboxItemIndicatorDataAttributes.ts:6,10`), the bare `data-disabled`
//!   (`:14`), and the two transition hooks (`:18,22`).
//! - the element (`:47-56`): a `<span>` carrying `aria-hidden: true` before the consumer's
//!   own bag, gated by `enabled: keepMounted || mounted` (`:55`).
//!
//! RUNTIME LAW, stated because it is the one piece with a machine behind it: the two hooks
//! are the internals crate's rg-0.2 ports, so — exactly as
//! [`crate::checkbox::indicator`]'s module docs record for the same pair — the port runs
//! them inside a dedicated rg-0.2 owner and mirrors every crossing value with one effect per
//! runtime. The bridge is shared with [`crate::menu::radio_item_indicator`], which is the
//! same machine over the other context.
//!
//! DEFERRED, with the reason: `render` and the forwarded ref (`:49`) — the crate-wide items
//! named in [`crate::menu::arrow`]'s docs. The DOM consequences of the resolved attributes
//! are CI's axis: this box refuses a browser (`ralph/generated/env-health.json` →
//! `browser: DEGRADED`), so what is asserted here is the resolved description and the
//! presence gate, not a mounted element.

use std::cell::Cell;
use std::rc::Rc;

use leptos::children::ChildrenFn;
use leptos::prelude::*;
use reactive_graph::traits::{Get as RgGet, GetUntracked as RgGetUntracked, Set as RgSet};
use reactive_graph::wrappers::read::Signal as RgSignal;
use wasm_bindgen::JsCast;

use leptos_ui_internals::use_open_change_complete::{
    UseOpenChangeCompleteParams, use_open_change_complete,
};
use leptos_ui_internals::use_transition_status::{TransitionStatus, use_transition_status};

use crate::menu::checkbox_item::{
    SharedMenuCheckboxItemContext, use_menu_checkbox_item_context,
};
use crate::menu::popup::transition_status_attr;
use crate::menu::utils::{menu_item_attributes, menu_item_state_map};

/// `MenuCheckboxItemIndicatorDataAttributes.checked` (`:6`).
pub const MENU_CHECKBOX_ITEM_INDICATOR_CHECKED_ATTRIBUTE: &str = "data-checked";

/// `MenuCheckboxItemIndicatorDataAttributes.unchecked` (`:10`).
pub const MENU_CHECKBOX_ITEM_INDICATOR_UNCHECKED_ATTRIBUTE: &str = "data-unchecked";

/// `aria-hidden: true` (`MenuCheckboxItemIndicator.tsx:52`) — the indicator is decorative;
/// the item's own `aria-checked` carries the semantics
/// (`MenuCheckboxItemDataAttributes.ts:2-4`).
pub const MENU_CHECKBOX_ITEM_INDICATOR_ARIA_HIDDEN: &str = "true";

/// `MenuCheckboxItemIndicatorState` (`MenuCheckboxItemIndicator.tsx:72-89`): `{ checked,
/// disabled, highlighted, transitionStatus }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuCheckboxItemIndicatorState {
    /// `checked` (`:76`) — the parent item's state, read through the context.
    pub checked: bool,
    /// `disabled` (`:80`).
    pub disabled: bool,
    /// `highlighted` (`:84`).
    pub highlighted: bool,
    /// `transitionStatus` (`:88`) — the hook's output (`:26`).
    pub transition_status: Option<TransitionStatus>,
}

/// The indicator's state as the attribute engine's input map
/// (`MenuCheckboxItemIndicator.tsx:40-45` over `itemMapping`, `:50`).
pub fn menu_checkbox_item_indicator_state_map(
    state: MenuCheckboxItemIndicatorState,
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = menu_item_state_map(state.disabled, state.highlighted, state.checked);
    if let Some(status) = state.transition_status {
        map.insert(
            "transitionStatus".to_owned(),
            serde_json::Value::String(transition_status_attr(status).to_owned()),
        );
    }
    map
}

/// `getStateAttributesProps(state, itemMapping)` (`MenuCheckboxItemIndicator.tsx:50`) — the
/// indicator element's resolved `data-*` set.
pub fn menu_checkbox_item_indicator_attributes(
    state: MenuCheckboxItemIndicatorState,
) -> Vec<(String, String)> {
    menu_item_attributes(&menu_checkbox_item_indicator_state_map(state))
}

/// `enabled: keepMounted || mounted` (`MenuCheckboxItemIndicator.tsx:55`) — upstream's
/// `useRenderElement` renders nothing while this is false (`:47-56`), which the view
/// expresses as the crate's `<Show … fallback=|| ()>` gate.
pub fn menu_checkbox_item_indicator_should_render(keep_mounted: bool, mounted: bool) -> bool {
    keep_mounted || mounted
}

/// The bridged transition machine: the leptos-tracked mirrors of the rg-0.2 hooks' outputs
/// (the [`crate::checkbox::indicator`] shape).
pub struct IndicatorTransition {
    /// The tracked `transitionStatus` (`MenuCheckboxItemIndicator.tsx:26`).
    pub status: RwSignal<Option<TransitionStatus>>,
    /// The tracked `mounted` (`:26`) — the `enabled` gate's input (`:55`).
    pub mounted: RwSignal<bool>,
}

/// Runs `useTransitionStatus(item.checked)` + `useOpenChangeComplete({ batch: true, … })`
/// inside a dedicated rg-0.2 owner and mirrors their outputs into leptos signals.
///
/// Shared with [`crate::menu::radio_item_indicator`]: the two indicators differ only in the
/// context they read their `checked` from.
pub fn use_indicator_transition(checked: RgSignal<bool, reactive_graph::owner::LocalStorage>) -> IndicatorTransition {
    let open_rg = reactive_graph::signal::RwSignal::new(RgGetUntracked::get_untracked(&checked));
    let enabled_rg = reactive_graph::signal::RwSignal::new(!RgGetUntracked::get_untracked(&checked));
    let batch_rg = reactive_graph::signal::RwSignal::new(true);

    let indicator_element: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    let status = RwSignal::new(None::<TransitionStatus>);
    let mounted = RwSignal::new(RgGetUntracked::get_untracked(&checked));

    let hook_owner = reactive_graph::owner::Owner::new();
    let (mounted_rg, transition_rg) = hook_owner.with(|| {
        let hook = use_transition_status(
            open_rg.clone(),
            // `enableIdleState` / `deferEndingState` default to `false` upstream
            // (`useTransitionStatus.ts:92-96`'s parameter defaults).
            reactive_graph::signal::RwSignal::new(false),
            reactive_graph::signal::RwSignal::new(false),
            false,
        );

        // `useOpenChangeComplete` (`MenuCheckboxItemIndicator.tsx:28-38`): `enabled:
        // !item.checked`, `open: item.checked`, `batch: true`, and the completion that
        // unmounts an unchecked indicator.
        let open_for_complete = open_rg.clone();
        let mounted_for_complete = hook.mounted.clone();
        use_open_change_complete(UseOpenChangeCompleteParams {
            enabled: enabled_rg.clone(),
            open: open_rg.clone(),
            reference: {
                let indicator_element = Rc::clone(&indicator_element);
                move || {
                    let element = indicator_element.replace(None);
                    indicator_element.set(element.clone());
                    element
                }
            },
            batch: batch_rg.clone(),
            on_complete: Rc::new(move || {
                if !RgGetUntracked::get_untracked(&open_for_complete) {
                    RgSet::set(&mounted_for_complete, false);
                }
            }),
        });

        (hook.mounted, hook.transition_status)
    });
    std::mem::forget(hook_owner);

    // Seed the leptos mirrors from the hooks' initializers (their `useState` reads).
    status.set(RgGetUntracked::get_untracked(&transition_rg));
    mounted.set(RgGetUntracked::get_untracked(&mounted_rg));

    // leptos → rg-0.2: the hook's `open`/`enabled` inputs follow the tracked state.
    Effect::new({
        let open_rg = open_rg.clone();
        let enabled_rg = enabled_rg.clone();
        move |_| {
            let checked_now = RgGet::get(&checked);
            RgSet::set(&open_rg, checked_now);
            RgSet::set(&enabled_rg, !checked_now);
        }
    });

    // rg-0.2 → leptos: the two outputs the view reads.
    reactive_graph::effect::Effect::new({
        let transition_rg = transition_rg.clone();
        move |_| {
            status.set(RgGet::get(&transition_rg));
        }
    });
    reactive_graph::effect::Effect::new({
        let mounted_rg = mounted_rg.clone();
        move |_| {
            mounted.set(RgGet::get(&mounted_rg));
        }
    });

    IndicatorTransition { status, mounted }
}

/// `Menu.CheckboxItemIndicator` — upstream's `MenuCheckboxItemIndicator`
/// (`MenuCheckboxItemIndicator.tsx:16-59`).
///
/// Renders a decorative `<span aria-hidden="true">` whose presence is
/// `keepMounted || mounted` and whose `data-*` set is the parent item's state. See the
/// module docs for what is ported and what is deferred.
#[component]
pub fn CheckboxItemIndicator(
    /// `keepMounted` (`:20`, `@default false`) — keeps the element in the DOM while the
    /// parent item is unchecked (`:69`).
    #[prop(default = false)] keep_mounted: bool,
    /// `className` (`:20`).
    #[prop(optional, into)] class: Option<String>,
    /// `style` (`:20`).
    #[prop(default = Vec::new())] style: Vec<(String, String)>,
    /// The indicator's content.
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    // `useMenuCheckboxItemContext()` (`:22`) — the REQUIRED read.
    let item = use_menu_checkbox_item_context(use_context::<SharedMenuCheckboxItemContext>());
    let checked = item.checked;
    let disabled = item.disabled;
    let highlighted = item.highlighted;

    // `useTransitionStatus(item.checked)` (`:26`) + `useOpenChangeComplete({…})` (`:28-38`).
    let transition = use_indicator_transition(checked);
    let mounted = transition.mounted;
    let status = transition.status;

    // `enabled: keepMounted || mounted` (`:55`).
    let should_render = Signal::derive(move || {
        menu_checkbox_item_indicator_should_render(keep_mounted, mounted.get())
    });

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    let indicator_node: NodeRef<leptos::html::Span> = NodeRef::new();

    // The consumer's `class`/`style` land on the node like the rest of the element's
    // attributes: they are static per mount, and applying them in an effect keeps the
    // gated subtree a re-callable view (the `radio::indicator` precedent for this element).
    {
        let class = class.clone();
        let style_attribute = style_attribute.clone();
        Effect::new(move |_| {
            let Some(span) = indicator_node.get() else {
                return;
            };
            let span: &web_sys::Element = span.unchecked_ref();
            if !class.is_empty() {
                let _ = span.set_attribute("class", &class);
            }
            if !style_attribute.is_empty() {
                let _ = span.set_attribute("style", &style_attribute);
            }
        });
    }

    // `if (!shouldRender) return null` (`:47-56` — `useRenderElement`'s `enabled: false`).
    view! {
        <Show when=move || should_render.get() fallback=|| ()>
            <span
                node_ref=indicator_node
                aria-hidden=MENU_CHECKBOX_ITEM_INDICATOR_ARIA_HIDDEN
                data-checked=move || RgGet::get(&checked).then_some("")
                data-unchecked=move || (!RgGet::get(&checked)).then_some("")
                data-disabled=move || RgGet::get(&disabled).then_some("")
                data-highlighted=move || RgGet::get(&highlighted).then_some("")
                data-starting-style=move || {
                    (status.get() == Some(TransitionStatus::Starting)).then_some("")
                }
                data-ending-style=move || {
                    (status.get() == Some(TransitionStatus::Ending)).then_some("")
                }
            >
                {children.as_ref().map(|children| children())}
            </span>
        </Show>
    }
}
