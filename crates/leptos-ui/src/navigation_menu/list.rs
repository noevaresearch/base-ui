//! Navigation Menu List component
//! 
//! The list component manages the overall menu structure and keyboard navigation.

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
};
use crate::{
    navigation_menu::types::Orientation,
    navigation_menu::constants::*,
};

/// Navigation Menu List component
/// 
/// The list component manages the overall menu structure and keyboard navigation.
#[component]
pub fn NavigationMenuList(
    /// Orientation of the menu
    orientation: Orientation,
    /// Whether this is a nested menu
    nested: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build list classes and attributes
    let list_classes = format!(
        "navigation-menu-list {}",
        if nested { "nested" } else { "" }
    );

    let list_attributes = vec![
        ("data-orientation", format!("{:?}", orientation)),
        ("data-nested", nested.to_string()),
    ];

    // Render the list
    view! {
        <nav
            class=list_classes
            data-orientation=format!("{:?}", orientation)
            data-nested=nested.to_string()
            role="menu"
            aria-orientation=match orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            }
        >
            {children()}
        </nav>
    }
}