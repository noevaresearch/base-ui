//! Menu radio item indicator — `Menu.RadioItemIndicator`, the port of
//! `packages/react/src/menu/radio-item-indicator/MenuRadioItemIndicator.tsx` (94 lines) and
//! `MenuRadioItemIndicatorDataAttributes.ts` (22 lines).
//!
//! WHAT THIS REPLACES. The previous file (spelled `radio-item-indicator.rs`) was a facade:
//! never declared (a hyphen is not a legal Rust identifier), rendering a shell with
//! `create_rw_signal(false)` where the context read belongs — its own comment said "In a
//! real implementation, this would read the checked state from context" — and an
//! `Effect::new` whose body did nothing. The element it rendered was a `<div>` where
//! upstream renders a `<span>`.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the props (`MenuRadioItemIndicator.tsx:20`): `keepMounted` (default `false`, `:69`) and
//!   the consumer's `render`/`className`/`style`/`...elementProps` members.
//! - the required context read (`:22`): `useMenuRadioItemContext()` throws upstream's own
//!   message outside a `Menu.RadioItem` (`MenuRadioItemIndicator.test.tsx:29-39`).
//! - the mount machine (`:26-38`): `useTransitionStatus(item.checked)` with
//!   `enableIdleState`/`deferEndingState` at their `false` defaults
//!   (`useTransitionStatus.ts:92-96`) plus `useOpenChangeComplete({ batch: true, enabled:
//!   !item.checked, open: item.checked, ref: indicatorRef, onComplete: if (!item.checked)
//!   setMounted(false) })`.
//! - the state record (`:40-45`): `{ checked, disabled, highlighted, transitionStatus }`,
//!   walked by `itemMapping` (`:50`) — `data-checked`/`data-unchecked`
//!   (`MenuRadioItemIndicatorDataAttributes.ts:6,10`), the bare `data-disabled` (`:14`) and
//!   the two transition hooks (`:18,22`).
//! - the element (`:47-56`): a `<span>` carrying `aria-hidden: true` before the consumer's
//!   own bag, gated by `enabled: keepMounted || mounted` (`:55`).
//!
//! The two indicators are the same machine over two contexts: this file mirrors
//! [`crate::menu::checkbox_item_indicator`], whose module docs carry the runtime law (the
//! rg-0.2 owner + the per-runtime mirrors, the [`crate::checkbox::indicator`] shape) and the
//! deferral of `render`.
//!
//! DEFERRED, with the reason: `render` and the forwarded ref (`:49`) — the crate-wide items
//! named in [`crate::menu::arrow`]'s docs. The DOM consequences of the resolved attributes
//! are CI's axis: this box refuses a browser (`ralph/generated/env-health.json` →
//! `browser: DEGRADED`).

use leptos::children::ChildrenFn;
use leptos::prelude::*;
use reactive_graph::traits::Get as RgGet;
use wasm_bindgen::JsCast;

use leptos_ui_internals::use_transition_status::TransitionStatus;

use crate::menu::checkbox_item_indicator::use_indicator_transition;
use crate::menu::popup::transition_status_attr;
use crate::menu::radio_item::{SharedMenuRadioItemContext, use_menu_radio_item_context};
use crate::menu::utils::{menu_item_attributes, menu_item_state_map};

/// `MenuRadioItemIndicatorDataAttributes.checked` (`:6`).
pub const MENU_RADIO_ITEM_INDICATOR_CHECKED_ATTRIBUTE: &str = "data-checked";

/// `MenuRadioItemIndicatorDataAttributes.unchecked` (`:10`).
pub const MENU_RADIO_ITEM_INDICATOR_UNCHECKED_ATTRIBUTE: &str = "data-unchecked";

/// `aria-hidden: true` (`MenuRadioItemIndicator.tsx:52`) — the indicator is decorative.
pub const MENU_RADIO_ITEM_INDICATOR_ARIA_HIDDEN: &str = "true";

/// `MenuRadioItemIndicatorState` (`MenuRadioItemIndicator.tsx:72-89`): `{ checked,
/// disabled, highlighted, transitionStatus }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuRadioItemIndicatorState {
    /// `checked` (`:76`) — the parent item's selection state, through the context.
    pub checked: bool,
    /// `disabled` (`:80`).
    pub disabled: bool,
    /// `highlighted` (`:84`).
    pub highlighted: bool,
    /// `transitionStatus` (`:88`) — the hook's output (`:26`).
    pub transition_status: Option<TransitionStatus>,
}

/// The indicator's state as the attribute engine's input map
/// (`MenuRadioItemIndicator.tsx:40-45` over `itemMapping`, `:50`).
pub fn menu_radio_item_indicator_state_map(
    state: MenuRadioItemIndicatorState,
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

/// `getStateAttributesProps(state, itemMapping)` (`MenuRadioItemIndicator.tsx:50`) — the
/// indicator element's resolved `data-*` set.
pub fn menu_radio_item_indicator_attributes(
    state: MenuRadioItemIndicatorState,
) -> Vec<(String, String)> {
    menu_item_attributes(&menu_radio_item_indicator_state_map(state))
}

/// `enabled: keepMounted || mounted` (`MenuRadioItemIndicator.tsx:55`).
pub fn menu_radio_item_indicator_should_render(keep_mounted: bool, mounted: bool) -> bool {
    keep_mounted || mounted
}

/// `Menu.RadioItemIndicator` — upstream's `MenuRadioItemIndicator`
/// (`MenuRadioItemIndicator.tsx:16-59`).
///
/// Renders a decorative `<span aria-hidden="true">` whose presence is
/// `keepMounted || mounted` and whose `data-*` set is the parent item's state.
#[component]
pub fn RadioItemIndicator(
    /// `keepMounted` (`:20`, `@default false`).
    #[prop(default = false)] keep_mounted: bool,
    /// `className` (`:20`).
    #[prop(optional, into)] class: Option<String>,
    /// `style` (`:20`).
    #[prop(default = Vec::new())] style: Vec<(String, String)>,
    /// The indicator's content.
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    // `useMenuRadioItemContext()` (`:22`) — the REQUIRED read.
    let item = use_menu_radio_item_context(use_context::<SharedMenuRadioItemContext>());
    let checked = item.checked;
    let disabled = item.disabled;
    let highlighted = item.highlighted;

    // `useTransitionStatus(item.checked)` (`:26`) + `useOpenChangeComplete({…})` (`:28-38`).
    let transition = use_indicator_transition(checked);
    let mounted = transition.mounted;
    let status = transition.status;

    let should_render = Signal::derive(move || {
        menu_radio_item_indicator_should_render(keep_mounted, mounted.get())
    });

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    let indicator_node: NodeRef<leptos::html::Span> = NodeRef::new();

    // The consumer's `class`/`style` land on the node like the rest of the element's
    // attributes (the `crate::menu::checkbox_item_indicator` precedent).
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

    // `if (!shouldRender) return null` (`:47-56`).
    view! {
        <Show when=move || should_render.get() fallback=|| ()>
            <span
                node_ref=indicator_node
                aria-hidden=MENU_RADIO_ITEM_INDICATOR_ARIA_HIDDEN
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
