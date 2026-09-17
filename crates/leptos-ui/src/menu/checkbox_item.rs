//! Menu checkbox item — `Menu.CheckboxItem`, the port of
//! `packages/react/src/menu/checkbox-item/MenuCheckboxItem.tsx` (182 lines) plus the two
//! modules it owns, `MenuCheckboxItemContext.ts` and `MenuCheckboxItemDataAttributes.ts`.
//!
//! WHAT THIS REPLACES. The previous file (spelled `checkbox-item.rs`) was a facade, not a
//! port, and it had never been type-checked: a hyphen is not a legal Rust identifier, so
//! `menu/mod.rs` could never declare the module, and its body imported
//! `crate::menu::utils::item_state_attributes_mapping` — a function that exists nowhere in
//! the crate (`grep -rn item_state_attributes_mapping crates/` matches only that call site
//! and its two siblings). It also invented props upstream does not carry (`aria_label`,
//! `aria_describedby`, a `render: Option<fn() -> HtmlElement>`) and rendered a hardcoded
//! `class="menu-item"` shell — the "uniform DOM shell" `specs/library/menu/behavior.md`
//! forbids.
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the props and their documented defaults (`MenuCheckboxItem.tsx:28-41`): `checked`
//!   (controlled, `:137`), `defaultChecked` (`@default false`, `:144`), `onCheckedChange`
//!   (`:148-149`), `disabled` (default `false`, `:34`), `closeOnClick` (default **`false`**,
//!   `:35` — the checkbox item's own default, unlike `Menu.Item`'s `true`), `label`
//!   (`:32`), `nativeButton` (default `false`, `:33`), `id` (`:31`).
//! - the `checked` duality through the shared `useControlled` port (`:53-58`):
//!   `controlled: checkedProp`, `default: defaultChecked ?? false`, `name: 'MenuCheckboxItem'`,
//!   and `state: 'checked'` — the last is what the dev-mode diagnostics name, which is why
//!   the port passes `state: Some("checked")` instead of the hook's `'value'` default.
//! - the disabled fold `disabledProp || rootDisabled` (`:48-49`), the highlight read
//!   `store.useState('isActive', listItem.index)` (`:50`) and the item's composite-list
//!   registration (`:43`) — the same three [`crate::menu::item::Item`] resolves.
//! - the click transition (`:80-92`): event details created with reason
//!   `REASONS.itemPress` (`:81`), `onCheckedChange(!checked, details)` (`:85`), the
//!   cancellation veto (`:87-89`), then the flip `setChecked(c => !c)` (`:91`).
//! - the close request on click (`useMenuItemCommonProps.ts:81-85`, reached through
//!   `useMenuItem`'s merged bag at `useMenuItem.ts:49-62`): with `closeOnClick` the item
//!   asks the store to close with the `itemPress` reason.
//! - the element (`:94-108`): `role="menuitemcheckbox"` (`:100`), `aria-checked` (`:101`),
//!   the item's `tabIndex` (`useMenuItemCommonProps.ts:63`) and its `data-*` set through
//!   the state mapping `itemMapping` (`:96`) — `data-checked`/`data-unchecked`
//!   (`MenuCheckboxItemDataAttributes.ts:6,10`) plus the engine's bare handling of
//!   `disabled`/`highlighted` (`:12,16`).
//! - the context provider (`:110-112`) carrying the state record (`:71-78`), read
//!   (required) by [`crate::menu::checkbox_item_indicator::CheckboxItemIndicator`].
//!
//! DEFERRED, each with the reason it is not this checkpoint's, and none silently dropped:
//! - `useMenuItem`'s `useButton` layer and its merged `getItemProps` bag
//!   (`useMenuItem.ts:29-34,47-65`) — the same deferral
//!   [`crate::menu::item::Item`]'s module docs record. Upstream splits the click's two
//!   responsibilities between that bag's `onClick` (the close request) and this file's
//!   `handleClick` (the toggle); the port places both on the element in the merge's own
//!   order, so no documented behaviour is dropped by the deferral.
//! - `onMouseMove`'s `itemhover` emission and `onMouseUp`'s context-menu protocol
//!   (`useMenuItemCommonProps.ts:69-80,86-118`): unreachable in this tree for the reasons
//!   `Item`'s docs state (the positioner supplies the `nodeId` payload, and no
//!   context-menu root is ported, so upstream's own guard is false).
//! - `render` and the forwarded refs (`:107`) — the crate-wide items named in
//!   [`crate::menu::arrow`]'s docs.

use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_composite_list_item::{
    UseCompositeListItemParams, use_composite_list_item,
};
use leptos_ui_utils::use_controlled::{SetValueAction, UseControlledProps, use_controlled};
use reactive_graph::computed::Memo as RgMemo;
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::Signal as RgSignal;

use crate::menu::store::{
    MenuChangeEventDetails, MenuItemMetadata, MenuStore, menu_change_event_details,
    menu_item_on_click, menu_item_tab_index, reasons_menu, use_menu_active_index_signal,
    use_menu_disabled_signal, use_menu_open_signal, use_menu_store,
};
use crate::menu::utils::{menu_item_attributes, menu_item_state_map};

/// `role: 'menuitemcheckbox'` (`MenuCheckboxItem.tsx:100`). The mined suite queries the
/// element by this role throughout (`MenuCheckboxItem.test.tsx:48`'s
/// `getByRole('menuitemcheckbox')`).
pub const MENU_CHECKBOX_ITEM_ROLE: &str = "menuitemcheckbox";

/// `MenuCheckboxItemDataAttributes.checked` (`:6`) — the attribute the `checked` state
/// maps to when true. The mapping lives in
/// [`crate::menu::utils::item_mapping`] (upstream's own home for `itemMapping`,
/// `packages/react/src/menu/utils/stateAttributesMapping.ts:3-9`).
pub const MENU_CHECKBOX_ITEM_CHECKED_ATTRIBUTE: &str = "data-checked";

/// `MenuCheckboxItemDataAttributes.unchecked` (`:10`) — the attribute the `checked` state
/// maps to when false.
pub const MENU_CHECKBOX_ITEM_UNCHECKED_ATTRIBUTE: &str = "data-unchecked";

/// `MenuCheckboxItemDataAttributes.disabled` (`:12`).
pub const MENU_CHECKBOX_ITEM_DISABLED_ATTRIBUTE: &str = "data-disabled";

/// `MenuCheckboxItemDataAttributes.highlighted` (`:16`).
pub const MENU_CHECKBOX_ITEM_HIGHLIGHTED_ATTRIBUTE: &str = "data-highlighted";

/// `MenuCheckboxItemContext` (`MenuCheckboxItemContext.ts:4-8`): `{ checked, highlighted,
/// disabled }` — the value `MenuCheckboxItem` publishes for its own parts.
///
/// Upstream's value is a fresh object per render and its consumers re-render with it; the
/// port carries reactive reads instead, which is the same contract (`Signal` is the port's
/// reactive read) in the crate's vocabulary.
#[derive(Clone)]
pub struct MenuCheckboxItemContextValue {
    /// `checked` (`MenuCheckboxItem.tsx:71-78`) — the exposed `useControlled` value, so a
    /// controlled item's consumer sees the controlled prop.
    pub checked: RgSignal<bool, LocalStorage>,
    /// `highlighted` (`:74`).
    pub highlighted: RgSignal<bool, LocalStorage>,
    /// `disabled` — the folded `disabledProp || rootDisabled` (`:72`).
    pub disabled: RgSignal<bool, LocalStorage>,
}

/// The context bridge: the value crosses `provide_context`'s `Send + Sync` bound through
/// the `SendWrapper` bridge — the [`crate::menu::group::SharedMenuGroupContext`]
/// precedent (the `positioner.rs` shape the group module documents).
pub type SharedMenuCheckboxItemContext = send_wrapper::SendWrapper<MenuCheckboxItemContextValue>;

/// `useMenuCheckboxItemContext` (`MenuCheckboxItemContext.ts:14-23`): the required read,
/// which throws upstream's own message outside `<Menu.CheckboxItem>` — the contract the
/// mined suite asserts verbatim
/// (`MenuCheckboxItemIndicator.test.tsx:31-41`).
pub fn use_menu_checkbox_item_context(
    value: Option<SharedMenuCheckboxItemContext>,
) -> MenuCheckboxItemContextValue {
    match value {
        Some(value) => value.take(),
        None => panic!(
            "Base UI: MenuCheckboxItemContext is missing. MenuCheckboxItem parts must be placed within <Menu.CheckboxItem>."
        ),
    }
}

/// The `MenuCheckboxItemContext` read against the current reactive owner
/// (`MenuCheckboxItemContext.ts:14`).
pub fn menu_checkbox_item_context() -> MenuCheckboxItemContextValue {
    use_menu_checkbox_item_context(use_context::<SharedMenuCheckboxItemContext>())
}

/// The checkbox item's resolved element description — the members
/// `useRenderElement('div', componentProps, …)` (`MenuCheckboxItem.tsx:94-108`) puts on the
/// element, in the crate's vocabulary. Resolving carries no DOM access, so the item's
/// contract is assertable on the host target (this box refuses a browser —
/// `ralph/generated/env-health.json` → `browser`).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuCheckboxItemResolved {
    /// The item's id (`useBaseUiId(idProp)`, `:45`).
    pub id: String,
    /// `role` (`:100`).
    pub role: &'static str,
    /// `aria-checked` (`:101`) — the checked state itself, not its string form.
    pub aria_checked: bool,
    /// `tabIndex` (`useMenuItemCommonProps.ts:63`): `open && highlighted ? 0 : -1`.
    pub tab_index: i32,
    /// The resolved `data-*` set (`:96` over `itemMapping`).
    pub attributes: Vec<(String, String)>,
}

/// Resolves the checkbox item's element description (`MenuCheckboxItem.tsx:94-108`).
pub fn resolve_menu_checkbox_item(
    id: String,
    open: bool,
    highlighted: bool,
    disabled: bool,
    checked: bool,
) -> MenuCheckboxItemResolved {
    MenuCheckboxItemResolved {
        id,
        role: MENU_CHECKBOX_ITEM_ROLE,
        aria_checked: checked,
        tab_index: menu_item_tab_index(open, highlighted),
        attributes: menu_item_attributes(&menu_item_state_map(disabled, highlighted, checked)),
    }
}

/// The click transition's outcome (`MenuCheckboxItem.tsx:80-92`): the value the consumer's
/// `onCheckedChange` is told, and whether the flip is committed. Pure, so the cancellation
/// veto (`:87-89`) is assertable without a browser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuCheckboxItemClick {
    /// The first argument `onCheckedChange` receives: `!checked` at click time (`:85`).
    pub next_checked: bool,
    /// `!eventDetails.isCanceled` (`:87-89`) — false means the consumer vetoed the toggle.
    pub commit: bool,
}

/// `handleClick`'s decision (`MenuCheckboxItem.tsx:80-92`): the announced value is always
/// the negation of the current state; the commit is vetoed by `details.cancel()`.
pub fn menu_checkbox_item_click(checked: bool, canceled: bool) -> MenuCheckboxItemClick {
    MenuCheckboxItemClick {
        next_checked: !checked,
        commit: !canceled,
    }
}

/// `Menu.CheckboxItem` — upstream's `MenuCheckboxItem` (`MenuCheckboxItem.tsx:24-113`).
///
/// Renders a `<div role="menuitemcheckbox">` carrying the item's id, aria-checked,
/// tab index and `data-*` state, toggling its checked value (and announcing it) on click.
/// See the module docs for what is ported and what is deferred.
#[component]
pub fn CheckboxItem(
    /// `id` (`MenuCheckboxItem.tsx:31`) — defaults to the generated `base-ui-<n>` id
    /// (`useBaseUiId`, `:45`).
    #[prop(optional)] id: Option<String>,
    /// `label` (`:32`) — overrides the text used for keyboard text navigation.
    #[prop(optional)] label: Option<String>,
    /// `nativeButton` (`:33`, default `false`): the item renders a `<div>`, so the button
    /// semantics are the non-native ones. Consumed by the deferred `useButton` layer.
    #[prop(default = false)] native_button: bool,
    /// `disabled` (`:34`, default `false`) — ORed with the root's own `disabled` (`:49`).
    #[prop(default = false)] disabled: bool,
    /// `closeOnClick` (`:35`, default **`false`**) — the checkbox item's own default.
    #[prop(default = false)] close_on_click: bool,
    /// `checked` (`:36`, `:137`) — the controlled value; `None` is upstream's `undefined`
    /// (uncontrolled), so `defaultChecked` seeds the state.
    #[prop(optional)] checked: Option<bool>,
    /// `defaultChecked` (`:37`, `@default false`).
    #[prop(default = false)] default_checked: bool,
    /// `onCheckedChange` (`:38`, `:148-149`) — called with the new value and the event
    /// details, whose `cancel()` vetoes the change (`:87-89`).
    #[prop(optional)] on_checked_change: Option<Rc<dyn Fn(bool, &MenuChangeEventDetails)>>,
    /// `className` (`:29` via `BaseUIComponentProps`).
    #[prop(optional, into)] class: Option<String>,
    /// `style` (`:39`).
    #[prop(default = Vec::new())] style: Vec<(String, String)>,
    /// The item's content.
    children: Children,
) -> impl IntoView {
    // `useMenuRootContext()` (`:47`) — throws without a Root (behavior.md, "Public API
    // surface").
    let context = use_menu_store();
    let store: MenuStore = context.store;

    // `useBaseUiId(idProp)` (`:45`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id));

    // `useCompositeListItem({ guess: true, label })` (`:43`) — the item's index, which
    // feeds `highlighted` (`:50`) exactly as it does for `Menu.Item`.
    let list_item = use_composite_list_item::<MenuItemMetadata, RgRwSignal<Option<i32>, LocalStorage>>(
        UseCompositeListItemParams {
            guess: true,
            index: RgRwSignal::new_local(None::<i32>),
            label: Some(label),
            metadata: Some(MenuItemMetadata::RegularItem),
            text_ref: None,
        },
    );

    // The store reads (`:48-51`).
    let open = use_menu_open_signal(&store);
    let root_disabled = use_menu_disabled_signal(&store);
    let active_index = use_menu_active_index_signal(&store);
    let disabled_signal = RgMemo::new(move |_| disabled || root_disabled.get());
    let highlighted = RgMemo::new(move |_| {
        let index = list_item.index.get();
        index >= 0 && active_index.get() == Some(index as usize)
    });

    // `const [checked, setChecked] = useControlled({...})` (`:53-58`). The controlled
    // source is the prop itself (a static per mount in the port's component vocabulary,
    // hence a derived local signal), the default is `defaultChecked ?? false`, and the
    // `state: 'checked'` label is carried so the dev diagnostics name what upstream names.
    let controlled_source: RgSignal<Option<bool>, LocalStorage> = RgSignal::derive_local(move || checked);
    let (checked_signal, set_checked) = use_controlled(UseControlledProps {
        controlled: controlled_source,
        default: RgRwSignal::new_local(default_checked),
        name: "MenuCheckboxItem",
        state: Some("checked"),
    });

    // `<MenuCheckboxItemContext.Provider value={state}>` (`:110-112`).
    let item_context = MenuCheckboxItemContextValue {
        checked: checked_signal,
        highlighted: RgSignal::derive_local(move || highlighted.get()),
        disabled: RgSignal::derive_local(move || disabled_signal.get()),
    };
    provide_context(send_wrapper::SendWrapper::new(item_context.clone()));

    let checked_for_view = item_context.checked;
    let highlighted_for_view = item_context.highlighted;
    let disabled_for_view = item_context.disabled;
    let set_checked = Rc::new(move |action| set_checked(action));
    let on_checked_change_for_view = on_checked_change.clone();

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    view! {
        <div
            role=MENU_CHECKBOX_ITEM_ROLE
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

                // `handleClick` (`MenuCheckboxItem.tsx:80-92`).
                let details =
                    menu_change_event_details(reasons_menu::ITEM_PRESS, Some(native));
                let click = menu_checkbox_item_click(checked_for_view.get_untracked(), false);
                if let Some(callback) = on_checked_change_for_view.as_deref() {
                    callback(click.next_checked, &details);
                }
                if details.is_canceled() {
                    return;
                }
                set_checked(SetValueAction::Update(Box::new(|current: &bool| !current)));
            }
        >
            {children()}
        </div>
    }
}
