//! Navigation Menu Trigger component
//! 
//! The trigger component handles user interactions and manages trigger state.

use leptos::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::KeyboardEvent;

use crate::navigation_menu::types::*;

/// Navigation Menu Trigger component
/// 
/// The trigger component handles user interactions and manages trigger state.
#[component]
pub fn NavigationMenuTrigger(
    /// Whether this trigger is currently active
    #[prop(default = false)]
    active: bool,
    /// Whether the trigger is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Callback when the trigger is activated
    #[prop(default = || Callback::new(|_: String| {}))]
    on_activate: Callback<String>,
    /// Callback when the trigger is deactivated
    #[prop(default = || Callback::new(|_: ()| {}))]
    on_deactivate: Callback<()>,
    /// Children components
    children: Children,
) -> impl IntoView {
    let trigger_ref = NodeRef::new();
    let trigger_classes = move || {
        format!(
            "navigation-menu-trigger {} {}",
            if active { "active" } else { "" },
            if disabled { "disabled" } else { "" }
        )
    };

    // Handle keyboard events
    let on_key_down = move |ev: KeyboardEvent| {
        if disabled {
            return;
        }

        match ev.key().as_str() {
            "Enter" | " " => {
                on_activate.send("trigger".to_string());
                ev.prevent_default();
            }
            "ArrowDown" | "ArrowRight" => {
                on_activate.send("next".to_string());
                ev.prevent_default();
            }
            "ArrowUp" | "ArrowLeft" => {
                on_activate.send("prev".to_string());
                ev.prevent_default();
            }
            _ => {}
        }
    };

    // Handle click events
    let on_click = move |ev: web_sys::MouseEvent| {
        if disabled {
            return;
        }
        on_activate.send("trigger".to_string());
        ev.prevent_default();
    };

    // Handle mouse enter events
    let on_mouse_enter = move || {
        if disabled {
            return;
        }
        on_activate.send("hover".to_string());
    };

    // Handle mouse leave events
    let on_mouse_leave = move || {
        if disabled {
            return;
        }
        on_deactivate.send(());
    };

    view! {
        <button
            class=trigger_classes
            data-active=active
            data-disabled=disabled
            on:keydown=on_key_down
            on:click=on_click
            on:mouseenter=on_mouse_enter
            on:mouseleave=on_mouse_leave
            disabled=disabled
            node_ref=trigger_ref
        >
            {children()}
        </button>
    }
}