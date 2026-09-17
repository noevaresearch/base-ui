//! Menu backdrop — `Menu.Backdrop`, port of `packages/react/src/menu/backdrop/MenuBackdrop.tsx`.
//!
//! WHAT THIS REPLACES. The previous `backdrop.rs` was a facade, not a port: it rendered a
//! hardcoded `<div class="menu-backdrop">` carrying invented `data-backdrop`/
//! `data-backdrop-hidden` attributes (upstream's set is `data-open`/`data-closed` plus the
//! transition pair, `MenuBackdropDataAttributes.ts:4-18`) and exported three "hooks"
//! (`use_menu_backdrop_props`, `use_menu_backdrop_hidden`, `use_menu_backdrop_style`) with no
//! upstream counterpart that returned constants.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from (`MenuBackdrop.tsx`,
//! 65 lines total):
//! - the four store reads (`:29-32`): `open`, `mounted`, `transitionStatus`,
//!   `lastOpenChangeReason`.
//! - the state record `{ open, transitionStatus }` (`:36-39`).
//! - `role="presentation"` and `hidden={!mounted}` (`:44-45`).
//! - the inline style (`:46-50`): `pointerEvents: 'none'` **only** when the menu was last
//!   opened by `triggerHover` (`REASONS.triggerHover`, `reasons.ts:4`), and always
//!   `userSelect`/`WebkitUserSelect: 'none'`. Both halves are behavior.md's
//!   `parts/arrow-backdrop-portal-viewport.md` → "State model" (hover-open sets
//!   `pointerEvents: 'none'`; click-open does not) — the `userSelect` half is one of that
//!   file's recorded untested surfaces (`implementation.md:100`), ported because the
//!   source carries it, and marked as such rather than dressed up as a proven contract.
//! - the state → `data-*` mapping: `stateAttributesMapping: popupTransitionStateMapping`
//!   (`:42`), so the element carries `data-open`/`data-closed`/`data-starting-style`/
//!   `data-ending-style` (`popupStateMapping.ts:53-73`; the names are
//!   `MenuBackdropDataAttributes.ts:4-18`).
//! - the props order (`:43-52`): the part's own bag first, the consumer's `elementProps`
//!   second, so a consumer member wins (the merge order `input.rs` documents).
//!
//! DEFERRED, with the reason, none of it silently dropped:
//! - the context-menu ref merge (`:33-35`, `contextMenuContext?.backdropRef`): the
//!   context-menu unit is not ported (no ledger item for it yet) and upstream's read is
//!   optional — with no context-menu root in scope upstream passes `forwardedRef` alone,
//!   which is this port's path. The read is recorded in
//!   `ralph/logs/spec-discrepancies.md` with the rest of this checkpoint's deferrals.
//! - the forwarded ref (`:20`) and `render` (`:19`) — the crate-wide items noted in
//!   [`crate::menu::arrow`]'s docs.

use leptos::prelude::*;
use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::popup_state_mapping::popup_transition_state_mapping;
use leptos_ui_internals::state_attributes::get_state_attributes_props;
use leptos_ui_internals::use_transition_status::TransitionStatus;
use reactive_graph::traits::Get;
use serde_json::{Map, Value};

use crate::menu::popup::transition_status_attr;
use crate::menu::store::use_menu_store;

/// `MenuBackdrop.Props` (`MenuBackdrop.tsx:21` over `BaseUIComponentProps<'div',
/// MenuBackdropState>`): the consumer's element props, landing after the part's own bag
/// (module docs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuBackdropProps {
    /// `className` (`MenuBackdrop.tsx:24`).
    pub class: Option<String>,
    /// `style` (`:24`) — the consumer's own style members, applied after the part's
    /// (module docs).
    pub style: Vec<(String, String)>,
}

/// `MenuBackdropState` (`MenuBackdrop.tsx:52-61`) — the state record the mapping consumes
/// (`:36-39`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuBackdropState {
    /// `open` (`:54`) — the store's open state (`:29`).
    pub open: bool,
    /// `transitionStatus` (`:58`) — the store's transition status (`:31`).
    pub transition_status: Option<TransitionStatus>,
}

/// `MenuBackdrop.tsx:36-39` — the state record as the mapping's input map. The
/// `transitionStatus` value is inserted with its rendered spelling
/// ([`transition_status_attr`]) because the ported `transitionStatusMapping` consumes that
/// spelling, and `'idle'` is carried rather than dropped: the mapping owns the key and
/// declines it explicitly (`popup_state_mapping.rs`'s composition test, `state_attributes.rs`
/// transition table) — the popup part's convention (`popup.rs:151-159`).
pub fn menu_backdrop_state(
    open: bool,
    transition_status: Option<TransitionStatus>,
) -> Map<String, Value> {
    let mut state = Map::new();
    state.insert("open".to_owned(), Value::Bool(open));
    if let Some(status) = transition_status {
        state.insert(
            "transitionStatus".to_owned(),
            Value::String(transition_status_attr(status).to_owned()),
        );
    }
    state
}

/// The backdrop element's `data-*` attributes:
/// `getStateAttributesProps(state, popupTransitionStateMapping)` (`MenuBackdrop.tsx:42`).
pub fn menu_backdrop_attributes(state: &Map<String, Value>) -> Vec<(String, String)> {
    get_state_attributes_props(state, Some(&popup_transition_state_mapping))
        .into_iter()
        .collect()
}

/// `MenuBackdrop.tsx:46-50` — the inline style. `pointerEvents: 'none'` iff the last open
/// change's reason is `triggerHover` (`:48`, `reasons.ts:4`); the two `userSelect` members
/// are unconditional (`:49-50`). The consumer's members follow (module docs).
pub fn menu_backdrop_style(
    last_open_change_reason: Option<&str>,
    consumer: &[(String, String)],
) -> String {
    let mut style = String::new();
    if last_open_change_reason == Some(reasons::TRIGGER_HOVER) {
        style.push_str("pointer-events: none;");
    }
    style.push_str("user-select: none; -webkit-user-select: none;");
    for (property, value) in consumer {
        style.push_str(&format!("{property}: {value};"));
    }
    style
}

/// `MenuBackdrop.tsx:45` — `hidden: !mounted`.
pub fn menu_backdrop_hidden(mounted: bool) -> bool {
    !mounted
}

/// `Menu.Backdrop` — upstream's `MenuBackdrop` (`MenuBackdrop.tsx:20-54`).
///
/// A pure function of store state: `role="presentation"`, hidden while unmounted, and
/// hover-open pointer suppression. See the module docs for what is ported and deferred.
#[component]
pub fn Backdrop(
    /// `className` (`:24`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:24`).
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The element's content — upstream keeps children inside the rendered `<div>`
    /// (the `elementProps` spread, `:51`).
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    // `const { store } = useMenuRootContext()` (`:27`) — throws without a Root
    // (`MenuRootContext.ts`, the ancestry contract in `behavior.md`).
    let context = use_menu_store();
    let store = context.store;

    // The store reads (`:29-32`).
    let open = store.use_state(selectors::open);
    let mounted = store.use_state(selectors::mounted);
    let transition_status = store.use_state(selectors::transition_status);
    let last_open_change_reason = store.use_state(|state| {
        selectors::payload(state).and_then(|extra| extra.open_change_reason)
    });

    let class = class.unwrap_or_default();

    view! {
        <div
            role="presentation"
            class=class
            hidden=move || menu_backdrop_hidden(mounted.get()).then_some("true")
            style=move || menu_backdrop_style(last_open_change_reason.get().as_deref(), &style)
            data-open=move || open.get().then_some("true")
            data-closed=move || (!open.get()).then_some("true")
            data-starting-style=move || {
                (transition_status.get() == Some(TransitionStatus::Starting)).then_some("true")
            }
            data-ending-style=move || {
                (transition_status.get() == Some(TransitionStatus::Ending)).then_some("true")
            }
        >
            {children.map(|children| children())}
        </div>
    }
}
