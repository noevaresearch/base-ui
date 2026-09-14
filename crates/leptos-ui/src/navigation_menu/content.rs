//! Navigation Menu Content component
//!
//! The content component displays the menu content when triggered.

use leptos::prelude::*;

/// Navigation Menu Content component
///
/// The content component displays the menu content when triggered.
#[component]
pub fn NavigationMenuContent(
    /// The value for this menu item
    value: String,
    /// Whether this content is currently active
    #[prop(default = false)]
    active: bool,
    /// Whether to keep the content mounted even when inactive
    #[prop(default = false)]
    keep_mounted: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build content classes and attributes
    let content_classes = format!(
        "navigation-menu-content {}",
        if active { "active" } else { "" }
    );

    // Render the content component
    view! {
        <div
            class=content_classes
            data-value=value
            data-active=active
            data-keep-mounted=keep_mounted
            // Accessibility attributes
            role="region"
            aria-hidden=!active
            style=format!(
                "display: {};",
                if keep_mounted || active { "block" } else { "none" }
            )
        >
            {children()}
        </div>
    }
}
