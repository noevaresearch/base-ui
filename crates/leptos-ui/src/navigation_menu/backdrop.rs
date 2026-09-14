//! Navigation Menu Backdrop component
//!
//! The backdrop component provides a background overlay for the popup.

use leptos::prelude::*;

use crate::navigation_menu::constants::*;

/// Navigation Menu Backdrop component
///
/// The backdrop component provides a background overlay for the popup.
#[component]
pub fn NavigationMenuBackdrop(
    /// Whether the backdrop is visible
    #[prop(default = false)]
    visible: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build backdrop classes and attributes
    let backdrop_classes = format!(
        "navigation-menu-backdrop {}",
        if visible { "visible" } else { "" }
    );

    // Render the backdrop
    view! {
        <div
            class=backdrop_classes
            data-visible=visible
            // Accessibility attributes
            aria-hidden=!visible
            // Styling
            style=format!(
                "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: {}; background: rgba(0, 0, 0, 0.5);",
                BACKDROP_Z_INDEX
            )
        >
            {children()}
        </div>
    }
}
