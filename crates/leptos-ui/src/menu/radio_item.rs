//! Menu radio item — `Menu.RadioItem`, the port of
//! `packages/react/src/menu/radio-item/MenuRadioItem.tsx` (154 lines) plus the two modules
//! it owns, `MenuRadioItemContext.ts` and `MenuRadioItemDataAttributes.ts`.
//!
//! WHAT THIS REPLACES. The previous file (spelled `radio-item.rs`) was a facade, not a
//! port, and it had never been type-checked: a hyphen is not a legal Rust identifier, so
//! `menu/mod.rs` could never declare the module, and its body imported
//! `crate::menu::utils::item_state_attributes_mapping` — a function that exists nowhere in
//! the crate. It rendered a hardcoded shell, invented props upstream does not carry, and
//! had no path to the group's selection at all, which is the item's whole contract.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the props (`MenuRadioItem.tsx:27-38`): the required `value` (`:126`), `disabled`
//!   (default `false`, `:34`), `closeOnClick` (default `false`, `:34` — the item's own
//!   default, unlike `Menu.Item`'s `true`), `label` (`:31`), `nativeButton` (default
//!   `false`, `:32`), `id` (`:30`).
//! - the composite-list registration and the item's store reads
//!   (`:40-46`): `useCompositeListItem({ guess: true, label })`, `store.useState('isActive',
//!   listItem.index)` and the root's `disabled` (`:54`).
//! - the group read (`MenuRadioGroupContext.ts:15-24`): `value`, `setValue` and the group's
//!   `disabled` from [`crate::menu::radio_group`], required — rendering the item outside a
//!   `Menu.RadioGroup` throws upstream's own message
//!   (`MenuRadioItem.test.tsx:36-52`).
//! - the three-way disabled fold `disabledProp || groupDisabled || rootDisabled` (`:55`) and
//!   the selection comparison `checked = selectedValue === value` (`:56`).
//! - the click transition (`:78-84`): event details created with reason `REASONS.itemPress`
//!   (`:79`) and `setSelectedValue(value, details)` (`:83`) — the group's own gated setter,
//!   which is where the `cancel()` veto and the `onValueChange` call live
//!   (`MenuRadioGroup.tsx:42-52`).
//! - the close request on click (`useMenuItemCommonProps.ts:81-85`, through `useMenuItem`'s
//!   merged bag at `useMenuItem.ts:49-62`): with `closeOnClick` the item asks the store to
//!   close with the `itemPress` reason.
//! - the element (`:86-100`): `role="menuitemradio"` (`:92`), `aria-checked` (`:93`), the
//!   item's `tabIndex` (`useMenuItemCommonProps.ts:63`) and its `data-*` set through the
//!   state mapping `itemMapping` (`:88`) — `data-checked`/`data-unchecked`
//!   (`MenuRadioItemDataAttributes.ts:4,7`) plus the engine's bare handling of
//!   `disabled`/`highlighted` (`:12,16`).
//! - the context provider (`:102`) carrying the state record (`:69-76`), read (required) by
//!   [`crate::menu::radio_item_indicator::RadioItemIndicator`].
//!
//! DEFERRED, each with the reason it is not this checkpoint's, and none silently dropped:
//! - `useMenuItem`'s `useButton` layer and its merged `getItemProps` bag
//!   (`useMenuItem.ts:29-34,47-65`) — the same deferral
//!   [`crate::menu::item::Item`]'s module docs record. Upstream splits the click's two
//!   responsibilities between that bag's `onClick` (the close request) and this file's
//!   `handleClick` (the selection); the port places both on the element in the merge's own
//!   order, so no documented behaviour is dropped by the deferral.
//! - `onMouseMove`'s `itemhover` emission and `onMouseUp`'s context-menu protocol
//!   (`useMenuItemCommonProps.ts:69-80,86-118`): unreachable in this tree for the reasons
//!   `Item`'s docs state.
//! - `render` and the forwarded refs (`:99`) — the crate-wide items named in
//!   [`crate::menu::arrow`]'s docs.

use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_composite_list_item::{
    UseCompositeListItemParams, use_composite_list_item,
};
use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::Signal as RgSignal;

use crate::menu::checkbox_item::{
    MENU_CHECKBOX_ITEM_DISABLED_ATTRIBUTE, MENU_CHECKBOX_ITEM_HIGHLIGHTED_ATTRIBUTE,
};
use crate::menu::radio_group::{
    MenuRadioValue, SharedMenuRadioGroupContext, use_menu_radio_group_context,
};
use crate::menu::store::{
    MenuChangeEventDetails, MenuItemMetadata, MenuStore, menu_change_event_details,
    menu_item_on_click, menu_item_tab_index, reasons_menu, use_menu_active_index_signal,
    use_menu_disabled_signal, use_menu_open_signal, use_menu_store,
};
use crate::menu::utils::{menu_item_attributes, menu_item_state_map};

/// `role: 'menuitemradio'` (`MenuRadioItem.tsx:92`). The mined suite queries the element by
/// this role throughout (`MenuRadioItem.test.tsx:114`'s `getByRole('menuitemradio')`).
pub const MENU_RADIO_ITEM_ROLE: &str = "menuitemradio";

/// `MenuRadioItemDataAttributes.checked` (`:4`). The mapping itself lives in
/// [`crate::menu::utils::item_mapping`] — upstream's `itemMapping` maps BOTH item kinds'
/// `checked` field through the checkbox item's attribute names
/// (`packages/react/src/menu/utils/stateAttributesMapping.ts:3-9`), which is why this
/// module imports them from [`crate::menu::checkbox_item`] rather than restating them.
pub const MENU_RADIO_ITEM_CHECKED_ATTRIBUTE: &str = "data-checked";

/// `MenuRadioItemDataAttributes.unchecked` (`:7`).
pub const MENU_RADIO_ITEM_UNCHECKED_ATTRIBUTE: &str = "data-unchecked";

/// `MenuRadioItemContext` (`MenuRadioItemContext.ts:4-8`): `{ checked, highlighted,
/// disabled }`.
#[derive(Clone)]
pub struct MenuRadioItemContextValue {
    /// `checked` (`MenuRadioItem.tsx:69-76`) — the `selectedValue === value` comparison.
    pub checked: RgSignal<bool, LocalStorage>,
    /// `highlighted` (`:72`).
    pub highlighted: RgSignal<bool, LocalStorage>,
    /// `disabled` — the three-way fold (`:73`).
    pub disabled: RgSignal<bool, LocalStorage>,
}

/// The context bridge — the [`SharedMenuRadioGroupContext`] precedent.
pub type SharedMenuRadioItemContext = send_wrapper::SendWrapper<MenuRadioItemContextValue>;

/// `useMenuRadioItemContext` (`MenuRadioItemContext.ts:14-23`): the required read, which
/// throws upstream's own message outside `<Menu.RadioItem>` — the contract the mined suite
/// asserts verbatim (`MenuRadioItemIndicator.test.tsx:29-39`).
pub fn use_menu_radio_item_context(
    value: Option<SharedMenuRadioItemContext>,
) -> MenuRadioItemContextValue {
    match value {
        Some(value) => value.take(),
        None => panic!(
            "Base UI: MenuRadioItemContext is missing. MenuRadioItem parts must be placed within <Menu.RadioItem>."
        ),
    }
}

/// The `MenuRadioItemContext` read against the current reactive owner
/// (`MenuRadioItemContext.ts:14`).
pub fn menu_radio_item_context() -> MenuRadioItemContextValue {
    use_menu_radio_item_context(use_context::<SharedMenuRadioItemContext>())
}

/// `checked = selectedValue === value` (`MenuRadioItem.tsx:56`) — the item is selected when
/// the group's value is its own. Upstream's `===` over `any` ports to equality over
/// [`MenuRadioValue`].
pub fn menu_radio_item_checked(selected_value: &MenuRadioValue, value: &MenuRadioValue) -> bool {
    selected_value == value
}

/// The three-way disabled fold (`MenuRadioItem.tsx:55`):
/// `disabledProp || groupDisabled || rootDisabled`.
pub fn menu_radio_item_disabled(
    disabled_prop: bool,
    group_disabled: bool,
    root_disabled: bool,
) -> bool {
    disabled_prop || group_disabled || root_disabled
}

/// The item's resolved element description (`MenuRadioItem.tsx:86-100`).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuRadioItemResolved {
    /// The item's id (`useBaseUiId(idProp)`, `:42`).
    pub id: String,
    /// `role` (`:92`).
    pub role: &'static str,
    /// `aria-checked` (`:93`).
    pub aria_checked: bool,
    /// `tabIndex` (`useMenuItemCommonProps.ts:63`).
    pub tab_index: i32,
    /// The resolved `data-*` set (`:88` over `itemMapping`).
    pub attributes: Vec<(String, String)>,
}

/// Resolves the radio item's element description (`MenuRadioItem.tsx:86-100`).
pub fn resolve_menu_radio_item(
    id: String,
    open: bool,
    highlighted: bool,
    disabled: bool,
    checked: bool,
) -> MenuRadioItemResolved {
    MenuRadioItemResolved {
        id,
        role: MENU_RADIO_ITEM_ROLE,
        aria_checked: checked,
        tab_index: menu_item_tab_index(open, highlighted),
        attributes: menu_item_attributes(&menu_item_state_map(disabled, highlighted, checked)),
    }
}

/// `Menu.RadioItem` — upstream's `MenuRadioItem` (`MenuRadioItem.tsx:23-103`).
///
/// Renders a `<div role="menuitemradio">` whose checked state is the group's selection
/// compared with its own `value`; clicking it routes the selection through the group's
/// gated setter. See the module docs for what is ported and what is deferred.
#[component]
pub fn RadioItem(
    /// `value` (`MenuRadioItem.tsx:35`, `:126`) — required upstream; `None` here is a
    /// caller's explicit "no value" and compares equal to an unset group (upstream's
    /// `undefined === undefined`).
    #[prop(optional, into)] value: MenuRadioValue,
    /// `label` (`:31`) — overrides the text used for keyboard text navigation.
    #[prop(optional)] label: Option<String>,
    /// `nativeButton` (`:32`, default `false`). Consumed by the deferred `useButton` layer.
    #[prop(default = false)] native_button: bool,
    /// `disabled` (`:33`, default `false`) — folded with the group's and the root's (`:55`).
    #[prop(default = false)] disabled: bool,
    /// `closeOnClick` (`:34`, default **`false`**).
    #[prop(default = false)] close_on_click: bool,
    /// `id` (`:30`) — defaults to the generated `base-ui-<n>` id (`useBaseUiId`, `:42`).
    #[prop(optional)] id: Option<String>,
    /// `className` (`:29` via `BaseUIComponentProps`).
    #[prop(optional, into)] class: Option<String>,
    /// `style` (`:36`).
    #[prop(default = Vec::new())] style: Vec<(String, String)>,
    /// The item's content.
    children: Children,
) -> impl IntoView {
    // `useMenuRootContext()` (`:44`) — throws without a Root.
    let context = use_menu_store();
    let store: MenuStore = context.store;

    // `useMenuRadioGroupContext()` (`:48-52`) — the REQUIRED group read
    // (`MenuRadioItem.test.tsx:36-52`).
    let group = use_menu_radio_group_context(use_context::<SharedMenuRadioGroupContext>());

    // `useBaseUiId(idProp)` (`:42`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id));

    // `useCompositeListItem({ guess: true, label })` (`:40`).
    let list_item = use_composite_list_item::<MenuItemMetadata, RgRwSignal<Option<i32>, LocalStorage>>(
        UseCompositeListItemParams {
            guess: true,
            index: RgRwSignal::new_local(None::<i32>),
            label: Some(label),
            metadata: Some(MenuItemMetadata::RegularItem),
            text_ref: None,
        },
    );

    // The store reads (`:45,54`).
    let open = use_menu_open_signal(&store);
    let root_disabled = use_menu_disabled_signal(&store);
    let active_index = use_menu_active_index_signal(&store);
    let highlighted = RgMemo::new(move |_| {
        let index = list_item.index.get();
        index >= 0 && active_index.get() == Some(index as usize)
    });

    // `disabled = disabledProp || groupDisabled || rootDisabled` (`:55`) and
    // `checked = selectedValue === value` (`:56`).
    let group_value = group.value;
    let group_disabled = group.disabled;
    let value_for_view = value.clone();
    let disabled_signal = RgMemo::new(move |_| {
        menu_radio_item_disabled(disabled, group_disabled.get(), root_disabled.get())
    });
    let checked_signal = RgMemo::new(move |_| {
        menu_radio_item_checked(&group_value.get_untracked(), &value_for_view)
    });

    // `<MenuRadioItemContext.Provider value={state}>` (`:102`).
    let item_context = MenuRadioItemContextValue {
        checked: RgSignal::derive_local(move || checked_signal.get()),
        highlighted: RgSignal::derive_local(move || highlighted.get()),
        disabled: RgSignal::derive_local(move || disabled_signal.get()),
    };
    provide_context(send_wrapper::SendWrapper::new(item_context.clone()));

    let checked_for_view = item_context.checked;
    let highlighted_for_view = item_context.highlighted;
    let disabled_for_view = item_context.disabled;
    let value_for_click = value.clone();
    let set_selected_value = group.set_value;

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    view! {
        <div
            role=MENU_RADIO_ITEM_ROLE
            id=move || id_signal.get()
            aria-checked=move || checked_for_view.get().to_string()
            tabindex=move || menu_item_tab_index(open.get(), highlighted_for_view.get())
            data-disabled=move || disabled_for_view.get().then_some("")
            data-highlighted=move || highlighted_for_view.get().then_some("")
            data-checked=move || checked_for_view.get().then_some("")
            data-unchecked=move || (!checked_for_view.get()).then_some("")
            class=class
            style=style_attribute
            on:click=move |event: leptos::ev::MouseEvent| {
                // The close request the merged bag carries into `onClick`
                // (`useMenuItemCommonProps.ts:81-85`) — emitted first, the merge order.
                let native: web_sys::Event = event.into();
                menu_item_on_click(&store, close_on_click, Some(native.clone()));

                // `handleClick` (`MenuRadioItem.tsx:78-84`): the GROUP's gated setter runs
                // the consumer's `onValueChange` and the `cancel()` veto
                // (`MenuRadioGroup.tsx:42-52`).
                let details = menu_change_event_details(reasons_menu::ITEM_PRESS, Some(native));
                (set_selected_value)(value_for_click.clone(), &details);
            }
        >
            {children()}
        </div>
    }
}

/// The two `data-*` attribute names this item shares with the checkbox item, re-exported so
/// a consumer reading the radio item's contract has them at this module's path too.
/// (`MenuRadioItemDataAttributes.ts:4,7,12,16` names all four; the mapping that emits them —
/// upstream's single `itemMapping` for both item kinds — lives in [`crate::menu::utils`].)
pub use crate::menu::checkbox_item::{
    MENU_CHECKBOX_ITEM_DISABLED_ATTRIBUTE as MENU_RADIO_ITEM_DISABLED_ATTRIBUTE,
    MENU_CHECKBOX_ITEM_HIGHLIGHTED_ATTRIBUTE as MENU_RADIO_ITEM_HIGHLIGHTED_ATTRIBUTE,
};

/// The event details type the item hands the group's setter (`:79`).
pub type MenuRadioItemChangeEventDetails = MenuChangeEventDetails;
