//! Menu radio group — `Menu.RadioGroup`, the port of
//! `packages/react/src/menu/radio-group/MenuRadioGroup.tsx` (129 lines) and its context
//! module `MenuRadioGroupContext.ts` (24 lines).
//!
//! WHAT THIS REPLACES. The previous file (spelled `radio-group.rs`) was a facade, not a
//! port, and it had never been type-checked: a hyphen is not a legal Rust identifier, so
//! `menu/mod.rs` could never declare the module, and its body imported
//! `crate::menu::utils::item_state_attributes_mapping` — a function that exists nowhere in
//! the crate. Its context was an invented tuple carrying a `String` id, an
//! `Option<Callback<String>>` and a hardcoded value type, it minted its own state instead
//! of reading the Root's, and it carried props upstream does not have (`id`, `aria_label`,
//! `aria_describedby` — upstream's bag reaches the element through the consumer's own
//! `...elementProps` spread).
//!
//! WHAT IS PORTED HERE, with the upstream line each member comes from:
//! - the props (`MenuRadioGroup.tsx:22-32`): `value` (controlled, `:94`), `defaultValue`
//!   (`:100`), `onValueChange` (`:104-105`), `disabled` (default `false`, `:28`), and
//!   `aria-labelledby` (`:30`) — the last read as a prop rather than a bag member because
//!   the element composer consults it (`:61`).
//! - `labelId` and its setter (`:34`, `React.useState<string | undefined>(undefined)`),
//!   held as a signal so the element's `aria-labelledby` re-renders when a label registers.
//! - the value duality through the shared `useControlled` port (`:36-40`):
//!   `controlled: valueProp`, `default: defaultValue`, `name: 'MenuRadioGroup'` and — unlike
//!   the checkbox item — NO `state` label, which is why the port leaves
//!   [`UseControlledProps::state`] at `None` (the hook's `'value'` default).
//! - the gated setter (`:42-52`): `onValueChange(newValue, eventDetails)` first (`:44`),
//!   then the `eventDetails.isCanceled` veto (`:46-48`), then the write (`:50`).
//! - the element (`:56-65`): `role="group"` (`:60`), `aria-labelledby:
//!   ariaLabelledByProp ?? labelId` (`:61`), `aria-disabled: disabled || undefined` (`:62`),
//!   and the consumer's bag last.
//! - the context value and its provider (`:67-79`): `{ value, setValue, disabled }`
//!   (`MenuRadioGroupContext.ts:5-9`) published to the items, and — the wiring
//!   `specs/library/menu/implementation.md` → "Group/label wiring" names — the
//!   `MenuGroupContext` provider carrying the SAME `setLabelId` setter
//!   ([`crate::menu::group::MenuGroupContextValue`]) so a `Menu.GroupLabel` inside either
//!   group kind associates with it.
//!
//! The value type is `Option<String>` — the crate's [`crate::radio_group::RadioGroupValue`]
//! convention, whose module docs record that the mined behavior spec only ever proves
//! string values. Upstream's `value?: any` (`:94`) is wider; the port's narrower type is
//! stated here rather than implied, and it is the same narrowing the standalone
//! `RadioGroup` unit already ships.
//!
//! DEFERRED, with the reason: `render` and the forwarded ref (`:57-58`) — the crate-wide
//! items named in [`crate::menu::arrow`]'s docs. `useStableCallback` (`:42`) is upstream's
//! render-phase identity concern, which does not exist in Leptos (the `use_controlled`
//! port's own module docs record the same reasoning for its setter).

use std::rc::Rc;

use leptos::prelude::*;
use leptos_ui_utils::use_controlled::{SetValueAction, UseControlledProps, use_controlled};
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::Signal as RgSignal;

use crate::menu::group::{MenuGroupContextValue, SharedMenuGroupContext};
use crate::menu::store::MenuChangeEventDetails;

/// `role: 'group'` (`MenuRadioGroup.tsx:60`). `MenuRadioGroup.test.tsx:14-17` asserts it.
pub const MENU_RADIO_GROUP_ROLE: &str = "group";

/// The group's value type — see the module docs on the narrowing.
pub type MenuRadioValue = Option<String>;

/// `MenuRadioGroup.ChangeEventDetails` (`:122`) — the root's own details type, which is
/// what the item hands the setter (`MenuRadioItem.tsx:79-83`).
pub type MenuRadioGroupChangeEventDetails = MenuChangeEventDetails;

/// The setter's signature (`MenuRadioGroupContext.ts:7`):
/// `(newValue, eventDetails) => void`.
pub type OnMenuRadioValueChange = Rc<dyn Fn(MenuRadioValue, &MenuRadioGroupChangeEventDetails)>;

/// `MenuRadioGroupState` (`MenuRadioGroup.tsx:114-119`): `{ disabled }` — the state object
/// the element composer receives (`:54`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuRadioGroupState {
    /// `disabled` (`:117`).
    pub disabled: bool,
}

/// The group's resolved element description — the members
/// `useRenderElement('div', …)` (`MenuRadioGroup.tsx:56-65`) puts on the element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRadioGroupResolved {
    /// `role` (`:60`).
    pub role: &'static str,
    /// `aria-labelledby: ariaLabelledByProp ?? labelId` (`:61`).
    pub aria_labelledby: Option<String>,
    /// `aria-disabled: disabled || undefined` (`:62`).
    pub aria_disabled: Option<&'static str>,
}

/// `ariaLabelledByProp ?? labelId` (`MenuRadioGroup.tsx:61`) — the consumer's prop wins
/// over the registered label's id.
pub fn menu_radio_group_aria_labelledby(
    aria_labelledby_prop: Option<String>,
    label_id: Option<String>,
) -> Option<String> {
    aria_labelledby_prop.or(label_id)
}

/// `aria-disabled: disabled || undefined` (`:62`) — present only while disabled.
pub fn menu_radio_group_aria_disabled(disabled: bool) -> Option<&'static str> {
    disabled.then_some("true")
}

/// Resolves the group's element description (`MenuRadioGroup.tsx:56-65`).
pub fn resolve_menu_radio_group(
    aria_labelledby_prop: Option<String>,
    label_id: Option<String>,
    disabled: bool,
) -> MenuRadioGroupResolved {
    MenuRadioGroupResolved {
        role: MENU_RADIO_GROUP_ROLE,
        aria_labelledby: menu_radio_group_aria_labelledby(aria_labelledby_prop, label_id),
        aria_disabled: menu_radio_group_aria_disabled(disabled),
    }
}

/// The gated setter's commit decision (`MenuRadioGroup.tsx:42-52`): the consumer is told
/// first (`:44`), and its `cancel()` vetoes the write (`:46-48`). Pure, so the veto is
/// assertable without a browser.
pub fn menu_radio_group_commits(canceled: bool) -> bool {
    !canceled
}

/// The `MenuRadioGroupContext` value (`MenuRadioGroupContext.ts:5-9`): `{ value, setValue,
/// disabled }` — what `Menu.RadioItem` reads to derive its own checked state.
///
/// Upstream's value is a fresh object per render; the port carries reactive reads plus the
/// gated setter, which is the same contract in the crate's vocabulary (the
/// [`crate::menu::checkbox_item::MenuCheckboxItemContextValue`] shape).
#[derive(Clone)]
pub struct MenuRadioGroupContextValue {
    /// `value` (`:6`) — the exposed `useControlled` value.
    pub value: RgSignal<MenuRadioValue, LocalStorage>,
    /// `setValue` (`:7`) — the gated setter (`MenuRadioGroup.tsx:42-52`).
    pub set_value: OnMenuRadioValueChange,
    /// `disabled` (`:8`).
    pub disabled: RgSignal<bool, LocalStorage>,
}

/// The context bridge: the value crosses `provide_context`'s `Send + Sync` bound through
/// the `SendWrapper` bridge — the [`SharedMenuGroupContext`] precedent.
pub type SharedMenuRadioGroupContext = send_wrapper::SendWrapper<MenuRadioGroupContextValue>;

/// `useMenuRadioGroupContext` (`MenuRadioGroupContext.ts:15-24`): the required read, which
/// throws upstream's own message outside `<Menu.RadioGroup>` — the contract the mined suite
/// asserts verbatim (`MenuRadioItem.test.tsx:36-52`).
pub fn use_menu_radio_group_context(
    value: Option<SharedMenuRadioGroupContext>,
) -> MenuRadioGroupContextValue {
    match value {
        Some(value) => value.take(),
        None => panic!(
            "Base UI: MenuRadioGroupContext is missing. MenuRadioGroup parts must be placed within <Menu.RadioGroup>."
        ),
    }
}

/// The `MenuRadioGroupContext` read against the current reactive owner
/// (`MenuRadioGroupContext.ts:15`).
pub fn menu_radio_group_context() -> MenuRadioGroupContextValue {
    use_menu_radio_group_context(use_context::<SharedMenuRadioGroupContext>())
}

/// `Menu.RadioGroup` — upstream's `MenuRadioGroup` (`MenuRadioGroup.tsx:17-82`).
///
/// Renders the `<div role="group">` whose `labelId` its `Menu.GroupLabel` children register
/// into, publishes the selection context its items read, and keeps the group's own
/// `disabled` in that value. See the module docs for what is ported and deferred.
#[component]
pub fn RadioGroup(
    /// `value` (`MenuRadioGroup.tsx:25`, `:94`) — the controlled selection; `None` is
    /// upstream's `undefined` (uncontrolled), so `defaultValue` seeds the state.
    #[prop(optional, into)] value: MenuRadioValue,
    /// `defaultValue` (`:26`, `:100`).
    #[prop(optional, into)] default_value: MenuRadioValue,
    /// `onValueChange` (`:27`, `:104-105`) — called with the new value and the event
    /// details, whose `cancel()` vetoes the change (`:46-48`).
    #[prop(optional)] on_value_change: Option<OnMenuRadioValueChange>,
    /// `disabled` (`:28`, default `false`) — disables every descendant `Menu.RadioItem`
    /// (`MenuRadioItem.tsx:55`).
    #[prop(default = false)] disabled: bool,
    /// `'aria-labelledby'` (`:30`, `:61`) — wins over the registered label id.
    #[prop(optional, into)] aria_labelledby: Option<String>,
    /// `className` (`:24` via `BaseUIComponentProps`).
    #[prop(optional, into)] class: Option<String>,
    /// `style` (`:29`).
    #[prop(default = Vec::new())] style: Vec<(String, String)>,
    /// The group's contents (`MenuRadioGroupProps.children`, `:88`).
    children: Children,
) -> impl IntoView {
    // `const [labelId, setLabelId] = React.useState<string | undefined>(undefined)` (`:34`),
    // published through the SAME `MenuGroupContext` a `Menu.Group` publishes
    // (`:77` — "Group/label wiring", `specs/library/menu/implementation.md`).
    let group_context = MenuGroupContextValue::new();
    provide_context(send_wrapper::SendWrapper::new(group_context.clone()));

    // `const [value, setValueUnwrapped] = useControlled({...})` (`:36-40`). No `state`
    // label upstream, so none here either.
    let controlled_source: RgSignal<Option<MenuRadioValue>, LocalStorage> = match value {
        Some(value) => RgSignal::derive_local(move || Some(Some(value.clone()))),
        None => RgSignal::derive_local(|| None::<MenuRadioValue>),
    };
    let (value_signal, set_value_unwrapped) = use_controlled(UseControlledProps {
        controlled: controlled_source,
        default: RgRwSignal::new_local(default_value),
        name: "MenuRadioGroup",
        state: None,
    });

    // `setValue` (`:42-52`) — the gated committer the context publishes.
    let on_value_change_for_setter = on_value_change.clone();
    let set_value: OnMenuRadioValueChange = Rc::new(move |next: MenuRadioValue, details| {
        if let Some(callback) = on_value_change_for_setter.as_deref() {
            callback(next.clone(), details);
        }
        if !menu_radio_group_commits(details.is_canceled()) {
            return;
        }
        (set_value_unwrapped)(SetValueAction::Value(next));
    });

    // `<MenuRadioGroupContext.Provider value={context}>` (`:78`).
    let radio_context = MenuRadioGroupContextValue {
        value: value_signal,
        set_value,
        disabled: RgSignal::derive_local(move || disabled),
    };
    provide_context(send_wrapper::SendWrapper::new(radio_context.clone()));

    // The view captures the `Copy` state signals, not the context value: the
    // `menu::group` precedent (a view closure cannot capture the non-`Send` handles).
    let label_id = group_context.label_id;
    let aria_labelledby_for_view = aria_labelledby.clone();
    let disabled_for_view = radio_context.disabled;

    let class = class.unwrap_or_default();
    let style_attribute = style
        .into_iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<String>();

    view! {
        <div
            role=MENU_RADIO_GROUP_ROLE
            aria-labelledby=move || {
                menu_radio_group_aria_labelledby(
                    aria_labelledby_for_view.clone(),
                    label_id.get(),
                )
            }
            aria-disabled=move || menu_radio_group_aria_disabled(disabled_for_view.get())
            class=class
            style=style_attribute
        >
            {children()}
        </div>
    }
}
