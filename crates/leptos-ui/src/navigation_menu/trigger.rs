//! Navigation Menu Trigger component
//! 
//! The trigger component handles user interactions and opens/closes the menu.

use leptos::*;
use leptos::html::button;
use leptos::ev::{MouseEvent, FocusEvent};

use crate::{
    navigation_menu::{
        constants::*,
        types::{NavigationMenuTriggerProps, CloseReason},
    },
};

/// Navigation Menu Trigger component
/// 
/// The trigger component handles user interactions and opens/closes the menu.
#[component]
pub fn NavigationMenuTrigger(
    /// Whether this trigger is currently active
    active: bool,
    /// Whether the trigger is disabled
    disabled: bool,
    /// Callback when the trigger is activated
    on_activate: Callback<String>,
    /// Callback when the trigger is deactivated
    on_deactivate: Callback<()>,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Create refs for DOM elements
    let trigger_ref = NodeRef::<HtmlElement<button>>::new();

    // Create handlers for user interactions
    let handle_click = move |ev: MouseEvent| {
        if !disabled {
            if active {
                on_deactivate.call(());
            } else {
                on_activate.call("trigger".to_string());
            }
        }
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        if !disabled {
            match ev.key() {
                "Enter" | " " => {
                    ev.prevent_default();
                    if active {
                        on_deactivate.call(());
                    } else {
                        on_activate.call("trigger".to_string());
                    }
                }
                "ArrowDown" | "ArrowRight" => {
                    ev.prevent_default();
                    on_activate.call("next".to_string());
                }
                "ArrowUp" | "ArrowLeft" => {
                    ev.prevent_default();
                    on_activate.call("prev".to_string());
                }
                _ => {}
            }
        }
    };

    let handle_mouseenter = move |_: MouseEvent| {
        if !disabled {
            on_activate.call("hover".to_string());
        }
    };

    let handle_blur = move |_: FocusEvent| {
        if !disabled {
            on_deactivate.call(());
        }
    };

    // Build trigger classes and attributes
    let trigger_classes = format!(
        "navigation-menu-trigger {} {}",
        if active { "active" } else { "" },
        if disabled { "disabled" } else { "" }
    );

    // Render the trigger component
    view! {
        <button
            class=trigger_classes
            data-active=active
            data-disabled=disabled
            // Accessibility attributes
            aria-expanded=active
            aria-disabled=disabled
            // Event handlers
            on:click=handle_click
            on:keydown=handle_keydown
            on:mouseenter=handle_mouseenter
            on:blur=handle_blur
            disabled=disabled
            ref=trigger_ref
        >
            {children()}
        </button>
    }
}