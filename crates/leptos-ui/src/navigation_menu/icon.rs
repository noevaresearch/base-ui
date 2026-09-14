//! Navigation Menu Icon component
//! 
//! The icon component renders indicators for menu items.

use leptos::prelude::*;

/// Navigation Menu Icon component
/// 
/// The icon component renders indicators for menu items.
#[component]
pub fn NavigationMenuIcon(
    /// Whether the icon is open
    #[prop(default = false)]
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