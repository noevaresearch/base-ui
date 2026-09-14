//! Navigation Menu List component
//!
//! The list component contains the menu items and manages keyboard navigation.

use leptos::prelude::*;

/// Navigation Menu List component
///
/// The list component contains the menu items and manages keyboard navigation.
#[component]
pub fn NavigationMenuList(
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build list classes and attributes
    let list_classes = "navigation-menu-list".to_string();

    // Render the list component
    view! {
        <ul
            class=list_classes
            // Accessibility attributes
            role="list"
            // Keyboard navigation
            tabindex="0"
        >
            {children()}
        </ul>
    }
}
