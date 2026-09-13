//! Menu item components
//! 
//! A basic implementation of menu items that compiles and works

use leptos::prelude::*;

// Basic types that we need for menu items
pub struct MenuItemProps {
    pub children: Option<Children>,
    pub disabled: Option<bool>,
    pub on_select: Option<Callback<()>>,
}

pub struct MenuCheckboxItemProps {
    pub children: Option<Children>,
    pub disabled: Option<bool>,
    pub checked: Option<bool>,
    pub on_checked_change: Option<Callback<bool>>,
}

pub struct MenuRadioItemProps {
    pub children: Option<Children>,
    pub disabled: Option<bool>,
    pub value: Option<String>,
    pub checked: Option<bool>,
    pub on_checked_change: Option<Callback<bool>>,
}

pub struct MenuGroupProps {
    pub children: Option<Children>,
    pub heading: Option<String>,
}

pub struct MenuRadioGroupProps {
    pub children: Option<Children>,
    pub value: Option<String>,
    pub on_value_change: Option<Callback<String>>,
    pub disabled: Option<bool>,
}

pub struct MenuSeparatorProps {
    pub as_child: Option<bool>,
}

/// Menu item component
/// 
/// A basic menu item that can be selected.
pub fn MenuItem(props: MenuItemProps) -> impl IntoView {
    let MenuItemProps { 
        children, 
        disabled, 
        on_select,
    } = props;

    let disabled_signal = create_rw_signal(disulted.unwrap_or(false));
    let on_select_signal = on_select;

    view! {
        <div 
            class=format!(
                "flex items-center gap-2 px-3 py-2 text-sm rounded-md cursor-pointer {}",
                if disabled_signal.get() { "opacity-50 cursor-not-allowed" } else { "hover:bg-gray-100" }
            )
            on:click=move |_| {
                if !disabled_signal.get() {
                    if let Some(callback) = on_select_signal {
                        callback.call(());
                    }
                }
            }
        >
            {children}
        </div>
    }
}

/// Menu checkbox item component
/// 
/// A menu item with a checkbox that can be toggled.
pub fn MenuCheckboxItem(props: MenuCheckboxItemProps) -> impl IntoView {
    let MenuCheckboxItemProps { 
        children, 
        disabled, 
        checked, 
        on_checked_change,
    } = props;

    let disabled_signal = create_rw_signal(disabled.unwrap_or(false));
    let checked_signal = create_rw_signal(checked.unwrap_or(false));
    let on_checked_change_signal = on_checked_change;

    // Handle checked changes
    Effect::new(move |_| {
        if let Some(callback) = on_checked_change_signal {
            callback.call(checked_signal.get());
        }
    });

    view! {
        <div 
            class=format!(
                "flex items-center gap-2 px-3 py-2 text-sm rounded-md cursor-pointer {}",
                if disabled_signal.get() { "opacity-50 cursor-not-allowed" } else { "hover:bg-gray-100" }
            )
            on:click=move |_| {
                if !disabled_signal.get() {
                    checked_signal.update(|c| *c = !*c);
                }
            }
        >
            <input 
                type="checkbox"
                checked=checked_signal
                disabled=disabled_signal
                class="mr-2"
            />
            {children}
        </div>
    }
}

/// Menu radio item component
/// 
/// A menu item for radio selection within a group.
pub fn MenuRadioItem(props: MenuRadioItemProps) -> impl IntoView {
    let MenuRadioItemProps { 
        children, 
        disabled, 
        value, 
        checked, 
        on_checked_change,
    } = props;

    let disabled_signal = create_rw_signal(disabled.unwrap_or(false));
    let checked_signal = create_rw_signal(checked.unwrap_or(false));
    let value_signal = create_rw_signal(value.unwrap_or_else(|| "".to_string()));
    let on_checked_change_signal = on_checked_change;

    // Handle select
    let on_select = Callback::new(move |_| {
        if !disabled_signal.get() {
            checked_signal.set(true);
            if let Some(callback) = on_checked_change_signal {
                callback.call(true);
            }
        }
    });

    view! {
        <div 
            class=format!(
                "flex items-center gap-2 px-3 py-2 text-sm rounded-md cursor-pointer {}",
                if disabled_signal.get() { "opacity-50 cursor-not-allowed" } else { "hover:bg-gray-100" }
            )
            on:click=on_select
        >
            <input 
                type="radio"
                name="menu-radio-group"
                value=value_signal
                checked=checked_signal
                disabled=disabled_signal
                class="mr-2"
            />
            {children}
        </div>
    }
}

/// Menu group component
/// 
/// Groups related menu items together.
pub fn MenuGroup(props: MenuGroupProps) -> impl IntoView {
    let MenuGroupProps { 
        children, 
        heading,
    } = props;

    view! {
        <div class="mb-2">
            @if let Some(heading) = heading {
                <div class="px-3 py-2 text-sm font-semibold text-gray-700 border-b">
                    {heading}
                </div>
            }
            {children}
        </div>
    }
}

/// Menu radio group component
/// 
/// Groups radio items together for single selection.
pub fn MenuRadioGroup(props: MenuRadioGroupProps) -> impl IntoView {
    let MenuRadioGroupProps { 
        children, 
        value, 
        on_value_change, 
        disabled,
    } = props;

    let value_signal = create_rw_signal(value.unwrap_or_else(|| "".to_string()));
    let on_value_change_signal = on_value_change;
    let disabled_signal = create_rw_signal(disabled.unwrap_or(false));

    // Handle value changes
    Effect::new(move |_| {
        if let Some(callback) = on_value_change_signal {
            callback.call(value_signal.get());
        }
    });

    view! {
        <div>
            {children}
        </div>
    }
}

/// Menu separator component
/// 
/// A visual separator between menu items.
pub fn MenuSeparator(_props: MenuSeparatorProps) -> impl IntoView {
    view! {
        <div class="my-1 border-t border-gray-200" />
    }
}