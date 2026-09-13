//! Navigation Menu Arrow component
//! 
//! The arrow component renders an arrow indicator for the popup.

use leptos::*;
use leptos::html::div;

use crate::{
    navigation_menu::constants::*,
};

/// Navigation Menu Arrow component
/// 
/// The arrow component renders an arrow indicator for the popup.
#[component]
pub fn NavigationMenuArrow(
    /// Whether the arrow is visible
    visible: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build arrow classes and attributes
    let arrow_classes = format!(
        "navigation-menu-arrow {}",
        if visible { "visible" } else { "" }
    );

    // Render the arrow
    view! {
        <div
            class=arrow_classes
            data-visible=visible
            // Accessibility attributes
            aria-hidden=!visible
        >
            {children()}
        </div>
    }
}