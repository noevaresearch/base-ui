//! Menu radio item - a menu item with radio functionality
//! 
//! This is a port of Base UI's MenuRadioItem from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store};
use crate::menu::utils::item_state_attributes_mapping;

/// Props for the menu radio item component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRadioItemProps {
    /// Value for this radio item
    #[prop(into)]
    value: String,
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

/// Radio item component for the menu
/// 
/// Renders a menu item with radio functionality.
#[component]
pub fn MenuRadioItem(
    /// Props for the menu radio item
    #[prop(optional)]
    props: MenuRadioItemProps,
) -> impl IntoView {
    let MenuRadioItemProps {
        value,
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
    
    // Get the radio group context to determine if this item is selected
    let group_context = use_context::<crate::menu::radio_group::MenuRadioGroupContext>();
    let is_checked = if let Some(ctx) = &group_context {
        let group_value = &ctx.1;
        group_value.get() == value
    } else {
        false
    };
    
    // State for whether the item is highlighted
    let is_highlighted = create_rw_signal(false);
    
    // Handle click
    let on_click = move |_| {
        if disabled {
            return;
        }
        
        // Update the selected value in the group
        if let Some(ctx) = &group_context {
            let group_value = &ctx.1;
            group_value.set(value.clone());
            
            // Call the change callback
            if let Some(callback) = &ctx.2 {
                callback.call(value.clone());
            }
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
    let data_checked = is_checked;
    let data_unchecked = !is_checked;
    
    // Generate ARIA attributes
    let aria_disabled = disabled;
    let aria_checked = is_checked;
    let aria_label = aria_label.unwrap_or_else(|| {
        // In a real implementation, this would extract text from children
        format!("Radio menu item: {}", value)
    });
    let aria_describedby = aria_describedby;
    
    // Render the item
    let item_content = if let Some(render) = render {
        render()
    } else if let Some(children) = children {
        children().into_view()
    } else {
        view! { { "○ Radio Item" } }.into_view()
    };
    
    view! {
        <div
            class="menu-radio-item"
            role="menuitemradio"
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
            <div class="menu-radio-indicator">
                {if is_checked { "●" } else { "" }}
            </div>
            {item_content}
        </div>
    }
}

impl Default for MenuRadioItemProps {
    fn default() -> Self {
        Self {
            value: "".to_string(),
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

/// Hook to get menu radio item props
pub fn use_menu_radio_item_props() -> MenuRadioItemProps {
    MenuRadioItemProps::default()
}

/// Hook to get the menu radio item value
pub fn use_menu_radio_item_value() -> String {
    // In a real implementation, this would read from props or context
    "".to_string()
}

/// Hook to check if the menu radio item is disabled
pub fn use_menu_radio_item_disabled() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to check if the menu radio item should close the menu on click
pub fn use_menu_radio_item_close_on_click() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to get the menu radio item ID
pub fn use_menu_radio_item_id() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu radio item ARIA label
pub fn use_menu_radio_item_aria_label() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu radio item ARIA describedby
pub fn use_menu_radio_item_aria_describedby() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to check if the menu radio item is checked
pub fn use_menu_radio_item_checked() -> bool {
    // In a real implementation, this would read from context
    false
}