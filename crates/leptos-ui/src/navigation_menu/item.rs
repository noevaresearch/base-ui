//! Navigation Menu Item component
//! 
//! The item component represents a single menu item and manages its state.

use leptos::prelude::*;

use crate::navigation_menu::types::*;

/// Navigation Menu Item component
/// 
/// The item component represents a single menu item and manages its state.
#[component]
pub fn NavigationMenuItem(
    /// The value for this menu item
    value: String,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Create context for this item
    let item_context = NavigationMenuItemContext {
        value: value.clone(),
    };

    // Build item classes and attributes
    let item_classes = "navigation-menu-item".to_string();

    // Render the item component
    view! {
        <li
            class=item_classes
            data-value=value
            // Accessibility attributes
            role="none"
        >
            { move || {
                provide_context(item_context);
                children()
            }}
        </li>
    }
}