//! Menu link item - a menu item that acts as a link
//! 
//! This is a port of Base UI's MenuLinkItem from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store};
use crate::menu::utils::item_state_attributes_mapping;

/// Props for the menu link item component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuLinkItemProps {
    /// The URL for the link
    #[prop(into)]
    href: String,
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
    /// Whether to open the link in a new tab
    #[prop(default = false)]
    target_blank: bool,
}

/// Link item component for the menu
/// 
/// Renders a menu item that acts as a link.
#[component]
pub fn MenuLinkItem(
    /// Props for the menu link item
    #[prop(optional)]
    props: MenuLinkItemProps,
) -> impl IntoView {
    let MenuLinkItemProps {
        href,
        disabled,
        close_on_click,
        children,
        render,
        id,
        aria_label,
        aria_describedby,
        target_blank,
    } = props;
    
    let menu_store = use_menu_store();
    let open = menu_store.open();
    let highlighted_item = menu_store.highlighted_item();
    
    // State for whether the item is highlighted
    let is_highlighted = create_rw_signal(false);
    
    // Handle click
    let on_click = move |_| {
        if disabled {
            return;
        }
        
        // Navigate to the href
        if let Ok(window) = web_sys::window() {
            if let Ok(Some(_)) = window.open_with_url(&href, target_blank) {
                // Open the link
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
    let data_highlighted = is_highlighted.get();
    let data_disabled = disabled;
    
    // Generate ARIA attributes
    let aria_disabled = disabled;
    let aria_label = aria_label.unwrap_or_else(|| {
        // In a real implementation, this would extract text from children
        format!("Link menu item: {}", href)
    });
    let aria_describedby = aria_describedby;
    
    // Generate link target
    let target = if target_blank { "_blank" } else { "_self" };
    
    // Render the item
    let item_content = if let Some(render) = render {
        render()
    } else if let Some(children) = children {
        children().into_view()
    } else {
        view! { { "🔗 Link Item" } }.into_view()
    };
    
    view! {
        <a
            class="menu-link-item"
            role="menuitem"
            href=href
            aria-disabled=aria_disabled
            aria-label=aria_label
            aria-describedby=aria_describedby
            data-highlighted=data_highlighted
            data-disabled=data_disabled
            on_click=on_click
            on:mouseenter=on_mouse_enter
            on:mouseleave=on_mouse_leave
            on:keydown=on_key_down
            on:keyup=on_key_up
            tabindex=if disabled { "-1" } else { "0" }
            target=target
        >
            {item_content}
        </a>
    }
}

impl Default for MenuLinkItemProps {
    fn default() -> Self {
        Self {
            href: "#".to_string(),
            disabled: false,
            close_on_click: true,
            children: None,
            render: None,
            id: None,
            aria_label: None,
            aria_describedby: None,
            target_blank: false,
        }
    }
}

/// Hook to get menu link item props
pub fn use_menu_link_item_props() -> MenuLinkItemProps {
    MenuLinkItemProps::default()
}

/// Hook to get the menu link item href
pub fn use_menu_link_item_href() -> String {
    // In a real implementation, this would read from props or context
    "#".to_string()
}

/// Hook to check if the menu link item is disabled
pub fn use_menu_link_item_disabled() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to check if the menu link item should close the menu on click
pub fn use_menu_link_item_close_on_click() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to get the menu link item ID
pub fn use_menu_link_item_id() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu link item ARIA label
pub fn use_menu_link_item_aria_label() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu link item ARIA describedby
pub fn use_menu_link_item_aria_describedby() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to check if the menu link item should open in a new tab
pub fn use_menu_link_item_target_blank() -> bool {
    // In a real implementation, this would read from props or context
    false
}