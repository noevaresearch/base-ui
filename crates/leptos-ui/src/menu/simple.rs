//! Menu components - Minimal working version
//! 
//! The simplest possible implementation that compiles and works

use leptos::prelude::*;

/// Menu item component
pub fn MenuItem(children: Option<impl IntoView>) -> impl IntoView {
    let class = "menu-item";
    
    view! {
        <div 
            class={class}
            on:click=|_| {
                // Handle click - for now just log
                web_sys::console::log_1(&"Menu item clicked".into());
            }
        >
            {children}
        </div>
    }
}

/// Menu checkbox item component
pub fn MenuCheckboxItem(children: Option<impl IntoView>) -> impl IntoView {
    let class = "menu-checkbox-item";
    let checked_signal = RwSignal::new(false);

    view! {
        <div 
            class={class}
            on:click=move |_| {
                checked_signal.update(|c| *c = !*c);
                web_sys::console::log_1(&format!("Checkbox toggled: {}", checked_signal.get()).into());
            }
        >
            <input 
                type="checkbox"
                checked=checked_signal
                class={class}
            />
            {children}
        </div>
    }
}

/// Menu radio item component
pub fn MenuRadioItem(children: Option<impl IntoView>, value: String) -> impl IntoView {
    let class = "menu-radio-item";
    let checked_signal = RwSignal::new(false);

    view! {
        <div 
            class={class}
            on:click=move |_| {
                checked_signal.set(true);
                web_sys::console::log_1(&format!("Radio selected: {}", value.clone()).into());
            }
        >
            <input 
                type="radio"
                name="menu-radio-group"
                value={value.clone()}
                checked=checked_signal
                class={class}
            />
            {children}
        </div>
    }
}

/// Menu group component
pub fn MenuGroup(children: Option<impl IntoView>, heading: Option<String>) -> impl IntoView {
    let group_class = "menu-group";
    
    view! {
        <div class={group_class}>
            {children}
        </div>
    }
}

/// Menu radio group component
pub fn MenuRadioGroup(children: Option<impl IntoView>) -> impl IntoView {
    let value_signal = RwSignal::new("".to_string());

    view! {
        <div>
            {children}
        </div>
    }
}

/// Menu separator component
pub fn MenuSeparator() -> impl IntoView {
    let class = "menu-separator";
    
    view! {
        <div class={class} />
    }
}

// Simple placeholder components for now
pub fn MenuRoot(_children: Option<impl IntoView>) -> impl IntoView {
    let class = "menu-root";
    
    view! {
        <div class={class}>
            "Menu Root"
        </div>
    }
}

pub fn MenuTrigger(_children: Option<impl IntoView>) -> impl IntoView {
    let class = "menu-trigger";
    
    view! {
        <button class={class}>
            "Menu Trigger"
        </button>
    }
}

pub fn MenuPopup(_children: Option<impl IntoView>) -> impl IntoView {
    let class = "menu-popup";
    
    view! {
        <div class={class}>
            "Menu Popup"
        </div>
    }
}