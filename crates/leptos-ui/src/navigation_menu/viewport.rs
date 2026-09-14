//! Navigation Menu Viewport component
//!
//! The viewport component handles viewport-related positioning and clipping.

use leptos::prelude::*;

/// Navigation Menu Viewport component
///
/// The viewport component handles viewport-related positioning and clipping.
#[component]
pub fn NavigationMenuViewport(
    /// Whether the viewport is inert (non-interactive)
    #[prop(default = false)]
    inert: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build viewport classes and attributes
    let viewport_classes = format!(
        "navigation-menu-viewport {}",
        if inert { "inert" } else { "" }
    );

    // Render the viewport component
    view! {
        <div
            class=viewport_classes
            data-inert=inert
            // Accessibility attributes
            role="none"
            // Viewport styles
            style="position: relative; overflow: hidden;"
        >
            {children()}
        </div>
    }
}
