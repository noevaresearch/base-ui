//! Menu arrow - visual arrow for the menu popup
//! 
//! This is a port of Base UI's MenuArrow from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::utils::arrow_state_attributes_mapping;

/// Props for the menu arrow component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuArrowProps {
    /// Whether the arrow is hidden
    #[prop(default = false)]
    hidden: bool,
    /// Custom styles for the arrow
    #[prop(optional)]
    style: Option<String>,
}

/// Arrow component for the menu
/// 
/// Renders a visual arrow for the menu popup.
#[component]
pub fn MenuArrow(
    /// Props for the menu arrow
    #[prop(optional)]
    props: MenuArrowProps,
) -> impl IntoView {
    let MenuArrowProps { hidden, style } = props;
    
    // Generate state attributes
    let state_attrs = arrow_state_attributes_mapping();
    
    view! {
        <div
            class="menu-arrow"
            data-arrow=!hidden
            data-arrow-hidden=hidden
            style=style.unwrap_or_default()
        >
            {/* Arrow content - could be an SVG or CSS triangle */}
            <div class="menu-arrow-inner"></div>
        </div>
    }
}

impl Default for MenuArrowProps {
    fn default() -> Self {
        Self {
            hidden: false,
            style: None,
        }
    }
}

/// Hook to get menu arrow props
pub fn use_menu_arrow_props() -> MenuArrowProps {
    MenuArrowProps::default()
}

/// Hook to check if the menu arrow is hidden
pub fn use_menu_arrow_hidden() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to get the menu arrow style
pub fn use_menu_arrow_style() -> Option<String> {
    // In a real implementation, this would read from props or context
    None
}