//! Navigation Menu Trigger component
//! 
//! The trigger component handles user interactions (hover, click, keyboard)
//! that open/close the navigation menu.

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
};
use crate::{
    navigation_menu::types::{CloseReason, ActivationDirection},
    navigation_menu::constants::*,
};

/// Navigation Menu Trigger component
/// 
/// The trigger component handles user interactions (hover, click, keyboard)
/// that open/close the navigation menu.
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
    // Element refs
    let trigger_ref = NodeRef::<HtmlElement<button>>::new();

    // State
    let is_hovering = create_rw_signal(false);
    let is_clicked = create_rw_signal(false);

    // Handle hover interactions
    let handle_mouse_enter = {
        let on_activate = on_activate.clone();
        
        move |_| {
            if disabled {
                return;
            }
            
            is_hovering.set(true);
            
            // Open on hover if not already active
            if !active {
                on_activate("trigger".to_string());
            }
        }
    };

    let handle_mouse_leave = {
        let on_deactivate = on_deactivate.clone();
        
        move |_| {
            if disabled {
                return;
            }
            
            is_hovering.set(false);
            
            // Close on hover leave if not clicked
            if !is_clicked() {
                on_deactivate(());
            }
        }
    };

    // Handle click interactions
    let handle_click = {
        let on_activate = on_activate.clone();
        let on_deactivate = on_deactivate.clone();
        
        move |_| {
            if disabled {
                return;
            }
            
            is_clicked.set(true);
            
            // Toggle active state
            if active {
                on_deactivate(());
            } else {
                on_activate("trigger".to_string());
            }
        }
    };

    // Handle key events for keyboard navigation
    let handle_key_down = {
        let on_activate = on_activate.clone();
        
        move |event: leptos::ev::KeyboardEvent| {
            if disabled {
                return;
            }
            
            match event.key().as_str() {
                KEY_ARROW_DOWN | KEY_ARROW_UP | KEY_ARROW_LEFT | KEY_ARROW_RIGHT => {
                    // Open on arrow key
                    if !active {
                        on_activate("keyboard".to_string());
                    }
                    event.prevent_default();
                }
                KEY_ESCAPE => {
                    // Close on escape
                    on_deactivate(());
                    event.prevent_default();
                }
                _ => {}
            }
        }
    };

    // Handle blur events
    let handle_blur = {
        let on_deactivate = on_deactivate.clone();
        
        move |_| {
            if disabled {
                return;
            }
            
            // Close on blur if not hovering
            if !is_hovering() {
                on_deactivate(());
            }
            
            is_clicked.set(false);
        }
    };

    // Build trigger props
    let trigger_props = leptos::ev::MouseEvents::new()
        .on_mouse_enter(handle_mouse_enter)
        .on_mouse_leave(handle_mouse_leave);

    // Render the trigger
    view! {
        <button
            r#ref=trigger_ref
            class="navigation-menu-trigger"
            data-active=active
            data-disabled=disabled
            data-hovering=is_hovering()
            data-clicked=is_clicked()
            // Accessibility attributes
            aria-expanded=active
            aria-haspopup="true"
            aria-controls="navigation-menu-popup"
            // Event handlers
            on:click=handle_click
            on:blur=handle_blur
            {trigger_props}
        >
            {children()}
        </button>
    }
}