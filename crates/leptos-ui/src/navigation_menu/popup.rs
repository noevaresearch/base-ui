//! Navigation Menu Popup component
//! 
//! The popup component manages the popup container and positioning.

use leptos::prelude::*;

/// Navigation Menu Popup component
/// 
/// The popup component manages the popup container and positioning.
#[component]
pub fn NavigationMenuPopup(
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build popup classes and attributes
    let popup_classes = "navigation-menu-popup".to_string();

    // Render the popup component
    view! {
        <div
            class=popup_classes
            // Accessibility attributes
            role="dialog"
            aria-modal="true"
            aria-label="Menu popup"
        >
            {children()}
        </div>
    }
}