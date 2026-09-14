//! Menu trigger - the button that opens the menu
//! 
//! This is a port of Base UI's MenuTrigger from React to Leptos.

use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store, MenuStoreContext};
use leptos::ev::{KeyboardEvent, MouseEvent};
use wasm_bindgen::JsCast;

/// Trigger component for the menu
/// 
/// Renders a button that opens the menu when clicked or hovered.
#[component]
pub fn MenuTrigger(
    /// Whether the trigger is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether to open on hover
    #[prop(default = false)]
    open_on_hover: bool,
    /// Delay before opening on hover (in milliseconds)
    #[prop(default = 200)]
    hover_open_delay: u32,
    /// Delay before closing on hover (in milliseconds)
    #[prop(default = 200)]
    hover_close_delay: u32,
    /// Custom trigger content
    children: Children,
) -> impl IntoView {
    let menu_store = use_menu_store();
    let open = menu_store.open();
    let active_trigger = menu_store.active_trigger();
    
    // State for hover handling
    let is_hovering = RwSignal::new(false);
    // TODO: Implement proper hover timeouts when Timeout Send/Sync issues are resolved
    // let hover_timeout = RwSignal::new(None::<Timeout>);
    
    // Handle click
    let on_click = move || {
        if disabled {
            return;
        }
        
        // Toggle the menu open state
        let new_open = !open.get();
        open.set(new_open);
        
        // Update active trigger
        if new_open {
            // In a real implementation, we'd get the trigger element here
            // For now, we'll set it to None
            menu_store.set_active_trigger(None);
        }
    };
    
    // Handle mouse enter
    let on_mouse_enter = move || {
        if disabled {
            return;
        }
        
        is_hovering.set(true);
        
        // TODO: Implement proper hover timeouts when Timeout Send/Sync issues are resolved
        if open_on_hover {
            // For now, open immediately on hover
            open.set(true);
        }
    };
    
    // Handle mouse leave
    let on_mouse_leave = move || {
        if disabled {
            return;
        }
        
        is_hovering.set(false);
        
        // TODO: Implement proper hover timeouts when Timeout Send/Sync issues are resolved
        if open_on_hover {
            // For now, close immediately on mouse leave
            open.set(false);
        }
    };
    
    // Handle key down (for keyboard accessibility)
    let on_key_down = move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        
        match event.key().as_str() {
            "Enter" | " " | "ArrowDown" | "ArrowUp" => {
                event.prevent_default();
                open.set(true);
            }
            _ => {}
        }
    };
    
    // Handle key up
    let on_key_up = move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        
        match event.key().as_str() {
            " " => {
                event.prevent_default();
                open.set(false);
            }
            _ => {}
        }
    };
    
    view! {
        <button
            class="menu-trigger"
            disabled=disabled
            on:click=move |_| {
                if disabled {
                    return;
                }
                
                // Toggle the menu open state
                let new_open = !open.get();
                open.set(new_open);
                
                // Update active trigger
                if new_open {
                    // In a real implementation, we'd get the trigger element here
                    menu_store.set_active_trigger(None);
                } else {
                    menu_store.set_active_trigger(None);
                }
            }
            on:mouseenter=move |_| {
                if disabled {
                    return;
                }
                
                is_hovering.set(true);
                
                // TODO: Implement proper hover timeouts when Timeout Send/Sync issues are resolved
                if open_on_hover {
                    // For now, open immediately on hover
                    open.set(true);
                }
            }
            on:mouseleave=move |_| {
                if disabled {
                    return;
                }
                
                is_hovering.set(false);
                
                // TODO: Implement proper hover timeouts when Timeout Send/Sync issues are resolved
                if open_on_hover {
                    // For now, close immediately on mouse leave
                    open.set(false);
                }
            }
            on:keydown=move |event: KeyboardEvent| {
                if disabled {
                    return;
                }
                
                match event.key().as_str() {
                    "Enter" | " " | "ArrowDown" | "ArrowUp" => {
                        event.prevent_default();
                        // Toggle the menu open state
                        let new_open = !open.get();
                        open.set(new_open);
                        
                        // Update active trigger
                        if new_open {
                            menu_store.set_active_trigger(None);
                        } else {
                            menu_store.set_active_trigger(None);
                        }
                    },
                    _ => {}
                }
            }
            on:keyup=move |event: KeyboardEvent| {
                if disabled {
                    return;
                }
                
                match event.key().as_str() {
                    " " => {
                        event.prevent_default();
                        // Toggle the menu open state
                        let new_open = !open.get();
                        open.set(new_open);
                        
                        // Update active trigger
                        if new_open {
                            menu_store.set_active_trigger(None);
                        } else {
                            menu_store.set_active_trigger(None);
                        }
                    },
                    _ => {}
                }
            }
            aria-haspopup="menu"
            aria-expanded=open.get()
            data-popup-open=open.get()
            data-pressed=is_hovering.get()
        >
            {children()}
        </button>
    }
}

/// Hook to check if the trigger is disabled
pub fn use_menu_trigger_disabled() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to check if the trigger should open on hover
pub fn use_menu_trigger_open_on_hover() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to get hover open delay
pub fn use_menu_trigger_hover_open_delay() -> u32 {
    // In a real implementation, this would read from props or context
    200
}

/// Hook to get hover close delay
pub fn use_menu_trigger_hover_close_delay() -> u32 {
    // In a real implementation, this would read from props or context
    200
}