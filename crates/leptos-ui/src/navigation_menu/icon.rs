//! Navigation Menu Icon component
//! 
//! The icon component renders indicators for menu items.

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
};
use crate::{
    navigation_menu::constants::*,
};

/// Navigation Menu Icon component
/// 
/// The icon component renders indicators for menu items.
#[component]
pub fn NavigationMenuIcon(
    /// Whether the icon is open
    open: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build icon classes and attributes
    let icon_classes = format!(
        "navigation-menu-icon {}",
        if open { "open" } else { "" }
    );

    // Render the icon
    view! {
        <span
            class=icon_classes
            data-open=open
        >
            {children()}
        </span>
    }
}