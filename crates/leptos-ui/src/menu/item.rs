//! Menu item — `Menu.Item`, port of `packages/react/src/menu/item/MenuItem.tsx` and the
//! two hooks it composes (`useMenuItem.ts`, `useMenuItemCommonProps.ts`).
//!
//! WHAT THIS REPLACES. The previous `item.rs` was a facade, not a port: it rendered a
//! hardcoded `<div class="menu-item">` with a hand-rolled click handler and exported
//! four "hooks" (`use_menu_item`, `use_menu_item_id`, `use_menu_item_disabled`,
//! `use_menu_item_close_on_click`) that have no upstream counterpart at all — they
//! returned constants (`None`, `false`, `true`). The hardcoded class is what
//! `specs/library/menu/behavior.md` → "Uniform DOM shell" forbids, and the fabricated
//! hooks were public API claiming behaviour the unit never had.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the `id` — `useBaseUiId(idProp)` (`MenuItem.tsx:35`).
//! - `role="menuitem"` and `tabIndex = open && highlighted ? 0 : -1`
//!   (`useMenuItemCommonProps.ts:62-63`), both read from the store
//!   (`MenuItem.tsx:38-40`).
//! - the `onClick` close request with the `itemPress` reason
//!   (`useMenuItemCommonProps.ts:81-85`) through the unit's one mutation gate.
//! - the composite-list-item registration (`MenuItem.tsx:33`,
//!   `useCompositeListItem({ guess: true, label })`) — the item's index, which feeds
//!   `highlighted` (`:40`) and therefore `tabIndex` and `data-highlighted`.
//! - `data-highlighted` / `data-disabled` (`MenuItemDataAttributes.ts:3,6`).
//! - the disabled fold `disabledProp || rootDisabled` (`MenuItem.tsx:38-39`).
//! - the `label` prop, which overrides the item's keyboard-navigation text
//!   (`MenuItem.tsx:89-91`).
//!
//! DEFERRED, each with the reason it is not this checkpoint's, and none of them
//! silently dropped:
//! - The `useButton` layer (`useMenuItem.ts:29-34`, `focusableWhenDisabled: true`,
//!   `composite: true`) and the `getItemProps` merge that spreads it (`:47-65`). Its
//!   output is a merged props bag applied through the crate's element-rendering path
//!   (`use_render_element`); wiring that path for this unit is the checkpoint that also
//!   carries `Menu.Positioner`, whose context supplies the item's `nodeId`. The item's
//!   own DOM contract above does not depend on it.
//! - `onMouseMove`'s `'itemhover'` emission (`useMenuItemCommonProps.ts:69-80`): its
//!   payload is the POSITIONER's `nodeId`, and upstream itself returns early when there
//!   is none (`:70-72`). The positioner part is still a placeholder, so the port's
//!   absent node id is upstream's own no-op path, not a dropped behaviour.
//! - `onMouseUp`'s context-menu protocol (`:86-118`): upstream guards the whole block on
//!   `useContextMenuRootContext(true)` being defined (`:88,104`) — with no context-menu
//!   root in scope the branch is unreachable. The context-menu unit is not ported (no
//!   ledger item exists for it yet), so the guard is false here too.
//! - `onKeyDown`'s `typingRef` read (`:64-68`): upstream reads it optionally
//!   (`typingRef?.current`) and the ref is written by the typeahead session owner
//!   (`MenuStore.ts:193`, the composite/list part). A false read is upstream's own path
//!   while that owner is unported.

use leptos::prelude::*;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_composite_list_item::{
    use_composite_list_item, UseCompositeListItemParams,
};
// The reactive plumbing spelling the crate's parts use (`popover/parts.rs:9-11`): the
// `reactive_graph` aliases plus the `Get`/`GetUntracked` traits the signal reads need.
use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked};

use crate::menu::store::{
    MenuItemMetadata, MenuStore, menu_item_on_click, menu_item_state_attributes,
    menu_item_tab_index, use_menu_active_index_signal, use_menu_disabled_signal,
    use_menu_open_signal, use_menu_store,
};

/// `MenuItem.Props` (`MenuItem.tsx:77-102`) in the crate's spelling; upstream's
/// documented defaults are on [`MenuItemProps::default`].
#[derive(Clone)]
pub struct MenuItemProps {
    /// `className` (`MenuItem.tsx:23` via `BaseUIComponentProps`).
    pub class: Option<String>,
    /// `style` (`MenuItem.tsx:29`).
    pub style: Vec<(String, String)>,
    /// `id` (`:24,35`).
    pub id: Option<String>,
    /// `label` (`:25`) — overrides the text used for keyboard text navigation.
    pub label: Option<String>,
    /// `nativeButton` (`:26`, default `false`): the item renders a `<div>`, so the
    /// button semantics are the non-native ones.
    pub native_button: bool,
    /// `disabled` (`:27`, default `false`).
    pub disabled: bool,
    /// `closeOnClick` (`:28`, default `true`).
    pub close_on_click: bool,
}

impl Default for MenuItemProps {
    fn default() -> Self {
        Self {
            class: None,
            style: Vec::new(),
            id: None,
            label: None,
            native_button: false,
            disabled: false,
            close_on_click: true,
        }
    }
}

/// The resolved description of the item's root element — the members
/// `useRenderElement('div', componentProps, …)` (`MenuItem.tsx:59-63`) puts on the
/// element, in the crate's attribute vocabulary. Host-testable: the resolution carries
/// no DOM access, so the item's contract can be asserted without a browser (this box
/// refuses one — `ralph/generated/env-health.json` → `browser`).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemResolved {
    /// The item's id (`MenuItem.tsx:24,35`).
    pub id: String,
    /// `role` (`useMenuItemCommonProps.ts:62`).
    pub role: &'static str,
    /// `tabIndex` (`:63`).
    pub tab_index: i32,
    /// The folded disabled state (`MenuItem.tsx:38-39`).
    pub disabled: bool,
    /// `highlighted` (`:40`).
    pub highlighted: bool,
    /// The `data-*` state attributes (`MenuItemDataAttributes.ts:3,6`).
    pub attributes: Vec<(&'static str, &'static str)>,
}

/// Resolves the item's element description (`MenuItem.tsx:54-63` +
/// `useMenuItemCommonProps.ts:59-88`). Pure apart from the id the caller resolved, so
/// the item's observable contract is assertable on the host target.
pub fn resolve_menu_item(
    id: String,
    open: bool,
    highlighted: bool,
    disabled: bool,
) -> MenuItemResolved {
    MenuItemResolved {
        id,
        role: "menuitem",
        tab_index: menu_item_tab_index(open, highlighted),
        disabled,
        highlighted,
        attributes: menu_item_state_attributes(disabled, highlighted),
    }
}

/// `Menu.Item` — upstream's `MenuItem` (`MenuItem.tsx:17-64`).
///
/// Renders a `<div>` carrying the item's role, tab index, state attributes and the close
/// request; see the module docs for what is ported and what the next checkpoint owns.
#[component]
pub fn Item(
    /// `id` (`MenuItem.tsx:24`) — defaults to the generated `base-ui-<n>` id the hook
    /// produces (`useBaseUiId`).
    #[prop(optional)]
    id: Option<String>,
    /// `label` (`:25`).
    #[prop(optional)]
    label: Option<String>,
    /// `nativeButton` (`:26`, default `false`).
    #[prop(default = false)]
    native_button: bool,
    /// `disabled` (`:27`, default `false`) — ORed with the root's own `disabled`
    /// (`:38-39`).
    #[prop(default = false)]
    disabled: bool,
    /// `closeOnClick` (`:28`, default `true`).
    #[prop(default = true)]
    close_on_click: bool,
    /// `className` (`:23`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:29`).
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The item's content.
    children: Children,
) -> impl IntoView {
    // `useMenuRootContext()` (`MenuItem.tsx:37`) — throws without a Root, upstream's
    // own contract ("Public API surface", `specs/library/menu/behavior.md`).
    let context = use_menu_store();
    let store: MenuStore = context.store;

    // `useBaseUiId(idProp)` (`:35`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id));

    // `useCompositeListItem({ guess: true, label })` (`:33`) — the item's index in the
    // menu's composite list, which `isActive` compares against (`MenuStore.ts:73`). The
    // list itself is provided by the popup part; without one the registry's no-op
    // default resolves the index to the unindexed value and nothing is highlighted
    // (`useCompositeListItem.ts:46-50`).
    let list_item = use_composite_list_item::<
        MenuItemMetadata,
        RgRwSignal<Option<i32>, reactive_graph::owner::LocalStorage>,
    >(UseCompositeListItemParams {
        guess: true,
        index: RgRwSignal::new_local(None::<i32>),
        label: Some(label),
        metadata: Some(MenuItemMetadata::RegularItem),
        text_ref: None,
    });

    // The store reads (`MenuItem.tsx:38-40`, `useMenuItemCommonProps.ts:55`).
    let open = use_menu_open_signal(&store);
    let root_disabled = use_menu_disabled_signal(&store);
    let active_index = use_menu_active_index_signal(&store);
    let disabled_signal = RgMemo::new(move |_| disabled || root_disabled.get());
    let highlighted = RgMemo::new(move |_| {
        let index = list_item.index.get();
        index >= 0 && active_index.get() == Some(index as usize)
    });

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    view! {
        <div
            role="menuitem"
            id=move || id_signal.get()
            tabindex=move || menu_item_tab_index(open.get(), highlighted.get())
            data-disabled=move || disabled_signal.get().then_some("true")
            data-highlighted=move || highlighted.get().then_some("true")
            class=class
            style=style_attribute
            on:click=move |event: leptos::ev::MouseEvent| {
                // `onClick` (`useMenuItemCommonProps.ts:81-85`). The native event rides
                // the details so the gate's `is_mouse_event` instant-type heuristic reads
                // it (`store.rs:323`) — what upstream's `domEvent` carries.
                let store = store.clone();
                menu_item_on_click(&store, close_on_click, Some(event.into()));
            }
        >
            {children()}
        </div>
    }
}
