//! Menu submenu trigger - triggers for nested menus
//! 
//! This is a port of Base UI's MenuSubmenuTrigger from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store, MenuStoreContext};
use crate::menu::utils::{MenuSide, MenuAlign};

/// Props for the menu submenu trigger component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSubmenuTriggerProps {
    /// Whether the submenu trigger is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether to open the submenu on hover
    #[prop(default = true)]
    open_on_hover: bool,
    /// Delay before opening on hover (in milliseconds)
    #[prop(default = 200)]
    hover_open_delay: u32,
    /// Delay before closing on hover (in milliseconds)
    #[prop(default = 200)]
    hover_close_delay: u32,
    /// Label for the submenu trigger
    #[prop(into, optional)]
    label: Option<String>,
    /// Custom content for the submenu trigger
    #[prop(optional)]
    children: Option<Children>,
    /// Custom render function for the submenu trigger
    #[prop(optional)]
    render: Option<fn() -> HtmlElement>,
}

/// Submenu trigger component for the menu
/// 
/// Triggers nested menus and handles keyboard navigation.
#[component]
pub fn MenuSubmenuTrigger(
    /// Props for the menu submenu trigger
    #[prop(optional)]
    props: MenuSubmenuTriggerProps,
) -> impl IntoView {
    let MenuSubmenuTriggerProps {
        disabled,
        open_on_hover,
        hover_open_delay,
        hover_close_delay,
        label,
        children,
        render,
    } = props;
    
    let menu_store = use_menu_store();
    let open = menu_store.open();
    let highlighted_item = menu_store.highlighted_item();
    
    // State for hover handling
    let is_hovering = create_rw_signal(false);
    let hover_timeout = create_rw_signal(None::<Timeout>);
    
    // Handle click
    let on_click = move |_| {
        if disabled {
            return;
        }
        
        // Toggle the submenu open state
        let new_open = !open.get();
        open.set(new_open);
        
        // Update highlighted item
        if new_open {
            menu_store.set_highlighted_item(Some("submenu".to_string()));
        }
    };
    
    // Handle mouse enter
    let on_mouse_enter = move |_| {
        if disabled {
            return;
        }
        
        is_hovering.set(true);
        
        if open_on_hover {
            // Set timeout to open submenu
            let timeout = Timeout::new(hover_open_delay, move || {
                open.set(true);
            });
            hover_timeout.set(Some(timeout));
        }
    };
    
    // Handle mouse leave
    let on_mouse_leave = move |_| {
        if disabled {
            return;
        }
        
        is_hovering.set(false);
        
        // Clear any pending hover open timeout
        if let Some(timeout) = hover_timeout.get_untracked() {
            timeout.clear();
            hover_timeout.set(None);
        }
        
        if open_on_hover {
            // Set timeout to close submenu
            let timeout = Timeout::new(hover_close_delay, move || {
                open.set(false);
            });
            hover_timeout.set(Some(timeout));
        }
    };
    
    // Handle key down (for keyboard accessibility)
    let on_key_down = move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        
        match event.key().as_str() {
            "Enter" | " " | "ArrowDown" | "ArrowRight" => {
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
    
    // Generate ARIA attributes
    let aria_disabled = disabled;
    let aria_label = label.unwrap_or_else(|| "Submenu".to_string());
    let aria_expanded = open.get();
    let aria_haspopup = true;
    
    // Render the submenu trigger
    let trigger_content = if let Some(render) = render {
        render()
    } else if let Some(children) = children {
        children().into_view()
    } else {
        view! { { "▼" } }.into_view()
    };
    
    view! {
        <div
            class="menu-submenu-trigger"
            role="menuitem"
            aria-disabled=aria_disabled
            aria-label=aria_label
            aria-expanded=aria_expanded
            aria-haspopup=aria_haspopup
            data-highlighted=is_hovering.get()
            data-disabled=disabled
            on_click=on_click
            on:mouseenter=on_mouse_enter
            on:mouseleave=on_mouse_leave
            on:keydown=on_key_down
            on:keyup=on_key_up
            tabindex=if disabled { "-1" } else { "0" }
        >
            {trigger_content}
        </div>
    }
}

impl Default for MenuSubmenuTriggerProps {
    fn default() -> Self {
        Self {
            disabled: false,
            open_on_hover: true,
            hover_open_delay: 200,
            hover_close_delay: 200,
            label: None,
            children: None,
            render: None,
        }
    }
}

/// Hook to get menu submenu trigger props
pub fn use_menu_submenu_trigger_props() -> MenuSubmenuTriggerProps {
    MenuSubmenuTriggerProps::default()
}

/// Hook to check if the menu submenu trigger is disabled
pub fn use_menu_submenu_trigger_disabled() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to check if the menu submenu trigger should open on hover
pub fn use_menu_submenu_trigger_open_on_hover() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to get hover open delay
pub fn use_menu_submenu_trigger_hover_open_delay() -> u32 {
    // In a real implementation, this would read from props or context
    200
}

/// Hook to get hover close delay
pub fn use_menu_submenu_trigger_hover_close_delay() -> u32 {
    // In a real implementation, this would read from props or context
    200
}

/// Hook to get the submenu trigger label
pub fn use_menu_submenu_trigger_label() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}