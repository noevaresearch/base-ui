//! Menu link item — `Menu.LinkItem`, the port of
//! `packages/react/src/menu/link-item/MenuLinkItem.tsx` (106 lines) plus the one module it
//! owns, `MenuLinkItemDataAttributes.ts` (`:4`).
//!
//! WHAT THIS REPLACES. The previous file (spelled `link-item.rs`) was a facade, not a port, and
//! it had never been type-checked: a hyphen is not a legal Rust identifier, so `menu/mod.rs`
//! could never declare the module and nothing in the crate ever referenced it. Its body
//! (1) invented props upstream does not carry (`href` as a required `String`, `target_blank`,
//! `aria_label`, `aria_describedby`); (2) invented EIGHT helpers — `use_menu_link_item_props`
//! (returning `MenuLinkItemProps::default()`) plus seven "hooks" (`use_menu_link_item_href`,
//! `_disabled`, `_close_on_click`, `_id`, `_aria_label`, `_aria_describedby`, `_target_blank`)
//! that return constants; (3) defaulted `closeOnClick` to **`true`** where upstream defaults it
//! to `false` (`MenuLinkItem.tsx:29`); (4) navigated by calling `window.open_with_url` —
//! navigation the browser performs natively through the anchor, and which upstream never
//! performs; and (5) rendered the hardcoded `class="menu-link-item"` shell that
//! `specs/library/menu/behavior.md` → "Uniform DOM shell" forbids.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the props (`MenuLinkItem.tsx:24-32`): `id` (`:27`), `label` (`:28`), `closeOnClick`
//!   (default **`false`**, `:29`), `className`/`style` (`:26,30`), and the `...elementProps`
//!   rest (`:31`) — the bag that actually carries the link's `href`, since upstream has no
//!   `href` prop at all.
//! - the composite-list registration and the item's store reads (`:36,43`):
//!   `useCompositeListItem({ guess: true, label })`, `store.useState('isActive',
//!   listItem.index)` — the item's index in the parent menu's list is what `highlighted`
//!   compares against.
//! - `id = useBaseUiId(idProp)` (`:40`).
//! - the optional positioner read (`:37-38`, `useMenuPositionerContext(true)`): the `nodeId`
//!   it yields is the payload of the `itemhover` emission inside
//!   `useMenuItemCommonProps` (`:69-80`), which this port defers with the rest of the hover
//!   protocol (see DEFERRED below).
//! - the close request (`:52-61` through `useMenuItemCommonProps.ts:81-85`): with
//!   `closeOnClick` the click runs the store's ONE mutation gate with reason `itemPress`
//!   (`reason-parts.ts:8`); with `closeOnClick={false}` — the default — no request is made at
//!   all.
//! - the element (`:67-73`): upstream renders `'a'` with `state = { highlighted }`
//!   (`MenuLinkItemState`, `:86-91`), so the ONLY state attribute is `data-highlighted`
//!   (`MenuLinkItemDataAttributes.ts:4`) — deliberately unlike its sibling items, whose state
//!   also carries `checked` (`MenuCheckboxItem.tsx:71-78`, `MenuRadioItem.tsx:69-76`) and
//!   `disabled` (`MenuItem.tsx:38-40`). The item has no `disabled` prop and no checked state,
//!   which is why this module's resolved record carries neither `data-disabled` nor
//!   `data-checked`/`data-unchecked`.
//! - `role="menuitem"` and the roving `tabIndex = open && highlighted ? 0 : -1`
//!   (`useMenuItemCommonProps.ts:62-63`) — the mined suite queries this part by that role
//!   (`MenuLinkItem.test.tsx:53`), including while it renders as an anchor.
//!
//! The one behaviour upstream gets from the ELEMENT CHOICE rather than from code: because the
//! root is an `<a>`, `Enter`/`Space` activation navigates natively
//! (`MenuLinkItem.test.tsx:69-115`) and the typeahead session's `Space` suppression
//! (`useMenuItemCommonProps.ts:64-68`, `MenuLinkItem.test.tsx:118-165`) is what keeps a live
//! typeahead from navigating. The port renders the same `<a>` root, so both remain the
//! browser's own behaviour; nothing here re-implements them.
//!
//! DEFERRED, each with the reason it is not this checkpoint's, and none silently dropped:
//! - `useButton`'s layer and the `getItemProps` merge that spreads it (`:47-50,63-65`) — the
//!   same deferral [`crate::menu::item::Item`]'s module docs record. Upstream's merge applies
//!   that bag LAST (`:71`), so the deferral's ordering consequence is stated rather than hidden:
//!   the button layer is non-native (`native: false`, `:48`), i.e. keyboard activation is the
//!   non-button path, which for this anchor is the browser's native link activation anyway.
//! - `onMouseMove`'s `itemhover` emission and `onMouseUp`'s context-menu protocol
//!   (`useMenuItemCommonProps.ts:69-80,86-118`): unreachable in this tree for the reasons
//!   [`crate::menu::item::Item`]'s docs state (the positioner part supplies no node id yet, and
//!   the context-menu root context is not ported).
//! - `render` and the forwarded refs (`:72`) — the crate-wide items named in
//!   [`crate::menu::arrow`]'s docs (`library: the view paths drop render's element form`).
//!   `render` matters more here than elsewhere (the mined suite composes a router `<Link>`
//!   through it, `MenuLinkItem.test.tsx:44-45`), so it is called out as this part's largest
//!   remaining fidelity gap rather than folded into the general deferral.

use leptos::prelude::*;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_composite_list_item::{
    UseCompositeListItemParams, use_composite_list_item,
};
// The reactive plumbing spelling the crate's parts use (`popover/parts.rs:9-11`): the
// `reactive_graph` aliases plus the traits the signal reads need.
use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::Get;

use crate::menu::store::{
    MenuItemMetadata, MenuStore, menu_item_on_click, menu_item_tab_index, use_menu_active_index_signal,
    use_menu_open_signal, use_menu_store,
};
use crate::menu::utils::{menu_item_attributes, menu_item_state_map};

/// Upstream renders `'a'` (`MenuLinkItem.tsx:69`, `useRenderElement('a', …)`), which the mined
/// suite pins as `HTMLAnchorElement` (`MenuLinkItem.test.tsx:12`).
pub const MENU_LINK_ITEM_TAG: &str = "a";

/// `role: 'menuitem'` (`useMenuItemCommonProps.ts:62`) — a link item is still a `menuitem`
/// (`MenuLinkItem.test.tsx:53`'s `getAllByRole('menuitem')`).
pub const MENU_LINK_ITEM_ROLE: &str = "menuitem";

/// `MenuLinkItemDataAttributes.highlighted` (`:4`) — the item's only state attribute.
pub const MENU_LINK_ITEM_HIGHLIGHTED_ATTRIBUTE: &str = "data-highlighted";

/// The `closeOnClick` default (`MenuLinkItem.tsx:29`): **`false`**. Declared as a constant so the
/// documented default is itself assertable (the item's own suite pins it: clicking a link item
/// leaves the menu open unless `closeOnClick` is passed, `MenuLinkItemProps`'s JSDoc `:63-66`).
pub const MENU_LINK_ITEM_CLOSE_ON_CLICK_DEFAULT: bool = false;

/// The item's state object (`MenuLinkItem.tsx:67`, `MenuLinkItemState` at `:86-91`) — exactly
/// the `{ highlighted }` map upstream hands `useRenderElement`, so the engine's own mapping
/// produces the attribute set.
pub fn menu_link_item_state_map(
    highlighted: bool,
) -> serde_json::Map<String, serde_json::Value> {
    let mut state = menu_item_state_map(false, highlighted, false);
    // Upstream's `MenuLinkItemState` carries `highlighted` ONLY (`:86-91`), while the shared
    // `menu_item_state_map` models the sibling items' `{ disabled, highlighted, checked }`.
    // Dropping the two members the item does not have keeps the engine honest: leaving
    // `checked: false` in the map would emit `data-unchecked` (the `itemMapping` pair,
    // `utils/stateAttributesMapping.ts:5-12`) on an element that has no checked state at all.
    state.remove("disabled");
    state.remove("checked");
    state
}

/// The resolved description of the item's root element — the members
/// `useRenderElement('a', componentProps, …)` (`MenuLinkItem.tsx:67-73`) puts on the element,
/// in the crate's attribute vocabulary. Host-testable: the resolution carries no DOM access, so
/// the item's contract can be asserted without a browser (this box refuses one —
/// `ralph/generated/env-health.json` → `browser`).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuLinkItemResolved {
    /// The resolved tag (`:69`).
    pub tag: &'static str,
    /// The item's id (`:27,40`).
    pub id: String,
    /// `role` (`useMenuItemCommonProps.ts:62`).
    pub role: &'static str,
    /// `tabIndex` (`:63`).
    pub tab_index: i32,
    /// `highlighted` (`:43`).
    pub highlighted: bool,
    /// The resolved `data-*` set (`:67-70` over the state map) — `data-highlighted` when
    /// highlighted, and nothing when not.
    pub attributes: Vec<(String, String)>,
}

/// Resolves the link item's element description (`MenuLinkItem.tsx:67-73` +
/// `useMenuItemCommonProps.ts:59-88`). Pure apart from the id the caller resolved, so the item's
/// observable contract is assertable on the host target.
pub fn resolve_menu_link_item(id: String, open: bool, highlighted: bool) -> MenuLinkItemResolved {
    MenuLinkItemResolved {
        tag: MENU_LINK_ITEM_TAG,
        id,
        role: MENU_LINK_ITEM_ROLE,
        tab_index: menu_item_tab_index(open, highlighted),
        highlighted,
        attributes: menu_item_attributes(&menu_link_item_state_map(highlighted)),
    }
}

/// `Menu.LinkItem` — upstream's `MenuLinkItem` (`MenuLinkItem.tsx:20-74`).
///
/// Renders an `<a role="menuitem">` whose `href` (and every other DOM prop) rides the
/// `element_attributes` rest bag, exactly as upstream's `...elementProps` spread carries them;
/// the caller's members are applied after the part's own, which is upstream's merge order
/// (`:71`'s `props: [itemProps, elementProps, getItemProps]`). See the module docs for what is
/// ported and what is deferred.
#[component]
pub fn LinkItem(
    /// `id` (`MenuLinkItem.tsx:27`) — defaults to the generated `base-ui-<n>` id the hook
    /// produces (`useBaseUiId`, `:40`).
    #[prop(optional)]
    id: Option<String>,
    /// `label` (`:28`) — overrides the text used for keyboard text navigation.
    #[prop(optional)]
    label: Option<String>,
    /// `closeOnClick` (`:29`, default **`false`**): whether a click on the link also asks the
    /// menu to close. A link item's default is `false` because the activation is a navigation.
    #[prop(default = MENU_LINK_ITEM_CLOSE_ON_CLICK_DEFAULT)]
    close_on_click: bool,
    /// `className` (`:26` via `BaseUIComponentProps`).
    #[prop(optional, into)]
    class: Option<String>,
    /// `style` (`:30`) — ordered declarations.
    #[prop(default = Vec::new())]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest (`:31`) — the link's own DOM props. `href` lives here, which is
    /// why this component does NOT take an `href` prop: upstream has none (`MenuLinkItem.tsx`'s
    /// `Props` at `:93-105` carries `label`, `id`, `closeOnClick` and the standard surface only).
    #[prop(default = Vec::new())]
    element_attributes: Vec<(String, String)>,
    /// The link's content.
    children: Children,
) -> impl IntoView {
    // `useMenuRootContext()` (`:42`) — throws without a Root, upstream's own contract
    // ("Public API surface", `specs/library/menu/behavior.md`).
    let context = use_menu_store();
    let store: MenuStore = context.store;

    // `useBaseUiId(idProp)` (`:40`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id));

    // `useCompositeListItem({ guess: true, label })` (`:36`) — the item's index in the parent
    // menu's composite list, which `isActive` compares against. Without an enclosing list
    // (the popup part) the registry's no-op default resolves the index to the unindexed value
    // and nothing is highlighted (`use_composite_list_item.rs`, the `Item` precedent).
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

    // The store reads (`:43`, `useMenuItemCommonProps.ts:55,63`).
    let open = use_menu_open_signal(&store);
    let active_index = use_menu_active_index_signal(&store);
    let store_for_click = store.clone();
    let highlighted = RgMemo::new(move |_| {
        let index = list_item.index.get();
        index >= 0 && active_index.get() == Some(index as usize)
    });

    // The `...elementProps` rest (`:31`) replayed onto the real node. `view!` has no attribute
    // spread, so the caller's bag is written by a commit effect — the checkbox-group
    // mount-writer / `fieldset::root::write_element_bag` shape. The members are set AFTER the
    // part's own, i.e. the caller wins on key conflict, which is upstream's bag order (`:71`).
    let node_ref = NodeRef::<leptos::html::A>::new();
    let extra_attributes = std::rc::Rc::new(element_attributes);
    Effect::new(move |_| {
        let attributes = std::rc::Rc::clone(&extra_attributes);
        if attributes.is_empty() {
            return;
        }
        // `NodeRef`'s untracked read is leptos's own `Get` (the 0.7 surface), not
        // `reactive_graph 0.2`'s — the version split `arrow.rs:205-211` documents.
        let Some(anchor) = <NodeRef<leptos::html::A> as leptos::prelude::GetUntracked>::get_untracked(
            &node_ref,
        ) else {
            return;
        };
        let element: web_sys::Element = web_sys::wasm_bindgen::JsCast::unchecked_into(anchor);
        for (name, value) in attributes.iter() {
            let _ = element.set_attribute(name, value);
        }
    });

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();
    let highlighted_for_view = highlighted;

    view! {
        <a
            role=MENU_LINK_ITEM_ROLE
            id=move || id_signal.get()
            tabindex=move || menu_item_tab_index(open.get(), highlighted_for_view.get())
            data-highlighted=move || highlighted_for_view.get().then_some("true")
            class=class
            style=style_attribute
            node_ref=node_ref
            on:click=move |event: leptos::ev::MouseEvent| {
                // `onClick` (`useMenuItemCommonProps.ts:81-85`): the close request, honoring the
                // item's own `closeOnClick` default of `false` (`MenuLinkItem.tsx:29`). The
                // navigation itself is the anchor's, exactly as upstream leaves it to the `'a'`
                // root (`:69`) — the facade this file replaces called `window.open` here, which
                // upstream never does.
                menu_item_on_click(&store_for_click, close_on_click, Some(event.into()));
            }
        >
            {children()}
        </a>
    }
}
