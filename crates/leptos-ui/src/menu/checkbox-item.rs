//! Menu checkbox item - a menu item with checkbox functionality
//! 
//! This is a port of Base UI's MenuCheckboxItem from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store};
use crate::menu::utils::item_state_attributes_mapping;

/// Props for the menu checkbox item component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuCheckboxItemProps {
    /// Whether the checkbox is checked (controlled)
    #[prop(into, optional)]
    checked: Option<bool>,
    /// Default checked state (uncontrolled)
    #[prop(default = false)]
    default_checked: bool,
    /// Callback when the checkbox changes
    #[prop(into, optional)]
    on_checked_change: Option<Callback<bool>>,
    /// Whether the item is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the item should close the menu when clicked
    #[prop(default = true)]
    close_on_click: bool,
    /// Custom content for the item
    #[prop(optional)]
    children: Option<Children>,
    /// Custom render function for the item
    #[prop(optional)]
    render: Option<fn() -> HtmlElement>,
    /// Unique identifier for the item
    #[prop(into, optional)]
    id: Option<String>,
    /// ARIA label for the item
    #[prop(into, optional)]
    aria_label: Option<String>,
    /// ARIA describedby for the item
    #[prop(into, optional)]
    aria_describedby: Option<String>,
}

/// Checkbox item component for the menu
/// 
/// Renders a menu item with checkbox functionality.
#[component]
pub fn MenuCheckboxItem(
    /// Props for the menu checkbox item
    #[prop(optional)]
    props: MenuCheckboxItemProps,
) -> impl IntoView {
    let MenuCheckboxItemProps {
        checked,
        default_checked,
        on_checked_change,
        disabled,
        close_on_click,
        children,
        render,
        id,
        aria_label,
        aria_describedby,
    } = props;
    
    let menu_store = use_menu_store();
    let open = menu_store.open();
    let highlighted_item = menu_store.highlighted_item();
    
    // State for checked value
    let checked_signal = checked.unwrap_or_else(|| create_rw_signal(default_checked));
    let is_checked = checked_signal;
    
    // State for whether the item is highlighted
    let is_highlighted = create_rw_signal(false);
    
    // Handle click
    let on_click = move |_| {
        if disabled {
            return;
        }
        
        // Toggle checked state
        let new_checked = !is_checked.get();
        is_checked.set(new_checked);
        
        // Call the change callback
        if let Some(callback) = &on_checked_change {
            callback.call(new_checked);
        }
        
        // Close the menu if requested
        if close_on_click {
            open.set(false);
        }
        
        // Update the highlighted item
        if let Some(item_id) = &id {
            menu_store.set_highlighted_item(Some(item_id.clone()));
        }
    };
    
    // Handle mouse enter
    let on_mouse_enter = move |_| {
        if disabled {
            return;
        }
        
        is_highlighted.set(true);
        
        // Update the highlighted item in the store
        if let Some(item_id) = &id {
            menu_store.set_highlighted_item(Some(item_id.clone()));
        }
    };
    
    // Handle mouse leave
    let on_mouse_leave = move |_| {
        is_highlighted.set(false);
    };
    
    // Handle key down (for keyboard accessibility)
    let on_key_down = move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        
        match event.key().as_str() {
            "Enter" | " " => {
                event.prevent_default();
                on_click(());
            }
            "ArrowDown" | "ArrowUp" | "Home" | "End" => {
                // Navigation keys - handled by the menu root
                event.prevent_default();
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
                on_click(());
            }
            _ => {}
        }
    };
    
    // Generate state attributes
    let state_attrs = item_state_attributes_mapping();
    let data_checked = is_checked.get();
    let data_unchecked = !is_checked.get();
    
    // Generate ARIA attributes
    let aria_disabled = disabled;
    let aria_checked = is_checked.get();
    let aria_label = aria_label.unwrap_or_else(|| {
        // In a real implementation, this would extract text from children
        "Checkbox menu item".to_string()
    });
    let aria_describedby = aria_describedby;
    
    // Render the item
    let item_content = if let Some(render) = render {
        render()
    } else if let Some(children) = children {
        children().into_view()
    } else {
        view! { { "☐ Checkbox Item" } }.into_view()
    };
    
    view! {
        <div
            class="menu-checkbox-item"
            role="menuitemcheckbox"
            aria-disabled=aria_disabled
            aria-checked=aria_checked
            aria-label=aria_label
            aria-describedby=aria_describedby
            data-highlighted=is_highlighted.get()
            data-disabled=disabled
            data-checked=data_checked
            data-unchecked=data_unchecked
            on_click=on_click
            on:mouseenter=on_mouse_enter
            on:mouseleave=on_mouse_leave
            on:keydown=on_key_down
            on:keyup=on_key_up
            tabindex=if disabled { "-1" } else { "0" }
        >
            <div class="menu-checkbox-indicator">
                {if is_checked.get() { "✓" } else { "" }}
            </div>
            {item_content}
        </div>
    }
}

impl Default for MenuCheckboxItemProps {
    fn default() -> Self {
        Self {
            checked: None,
            default_checked: false,
            on_checked_change: None,
            disabled: false,
            close_on_click: true,
            children: None,
            render: None,
            id: None,
            aria_label: None,
            aria_describedby: None,
        }
    }
}

/// Hook to get menu checkbox item props
pub fn use_menu_checkbox_item_props() -> MenuCheckboxItemProps {
    MenuCheckboxItemProps::default()
}

/// Hook to get the menu checkbox item checked state
pub fn use_menu_checkbox_item_checked() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to get the menu checkbox item default checked state
pub fn use_menu_checkbox_item_default_checked() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to get the menu checkbox item change callback
pub fn use_menu_checkbox_item_on_checked_change() -> Option<Callback<bool>> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to check if the menu checkbox item is disabled
pub fn use_menu_checkbox_item_disabled() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to check if the menu checkbox item should close the menu on click
pub fn use_menu_checkbox_item_close_on_click() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to get the menu checkbox item ID
pub fn use_menu_checkbox_item_id() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu checkbox item ARIA label
pub fn use_menu_checkbox_item_aria_label() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu checkbox item ARIA describedby
pub fn use_menu_checkbox_item_aria_describedby() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}