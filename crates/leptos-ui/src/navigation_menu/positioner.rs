//! Navigation Menu Positioner component
//!
//! The positioner component handles positioning of the popup relative to the trigger.

use leptos::prelude::*;

/// Navigation Menu Positioner component
///
/// The positioner component handles positioning of the popup relative to the trigger.
#[component]
pub fn NavigationMenuPositioner(
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build positioner classes and attributes
    let positioner_classes = "navigation-menu-positioner".to_string();

    // Render the positioner component
    view! {
        <div
            class=positioner_classes
            // Accessibility attributes
            role="none"
            // Positioning styles
            style="position: relative;"
        >
            {children()}
        </div>
    }
}
