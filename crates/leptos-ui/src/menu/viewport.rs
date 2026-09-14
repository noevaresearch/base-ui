//! Menu viewport - handles scrolling and viewport constraints
//! 
//! This is a port of Base UI's MenuViewport from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::utils::viewport_state_attributes_mapping;

/// Props for the menu viewport component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuViewportProps {
    /// Whether the viewport should scroll
    #[prop(default = true)]
    scrollable: bool,
    /// Maximum height for the viewport
    #[prop(optional)]
    max_height: Option<String>,
    /// Custom styles for the viewport
    #[prop(optional)]
    style: Option<String>,
}

/// Viewport component for the menu
/// 
/// Handles scrolling and viewport constraints for the menu popup.
#[component]
pub fn MenuViewport(
    /// Props for the menu viewport
    #[prop(optional)]
    props: MenuViewportProps,
) -> impl IntoView {
    let MenuViewportProps { scrollable, max_height, style } = props;
    
    // Generate state attributes
    let state_attrs = viewport_state_attributes_mapping();
    
    // Combine custom style with max height
    let combined_style = if let Some(max_height) = max_height {
        format!("{}max-height: {};", style.unwrap_or_default(), max_height)
    } else {
        style.unwrap_or_default()
    };
    
    view! {
        <div
            class="menu-viewport"
            data-scrollable=scrollable
            style=combined_style
        >
            {children()}
        </div>
    }
}

impl Default for MenuViewportProps {
    fn default() -> Self {
        Self {
            scrollable: true,
            max_height: None,
            style: None,
        }
    }
}

/// Hook to get menu viewport props
pub fn use_menu_viewport_props() -> MenuViewportProps {
    MenuViewportProps::default()
}

/// Hook to check if the menu viewport is scrollable
pub fn use_menu_viewport_scrollable() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to get the menu viewport max height
pub fn use_menu_viewport_max_height() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}

/// Hook to get the menu viewport style
pub fn use_menu_viewport_style() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}