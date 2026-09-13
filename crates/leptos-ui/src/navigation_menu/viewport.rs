//! Navigation Menu Viewport component
//! 
//! The viewport component provides clipping and scrolling for menu content.

use leptos::*;
use leptos_ui_internals::{
    types::BaseUIComponentProps,
};
use crate::{
    constants::*,
};

/// Navigation Menu Viewport component
/// 
/// The viewport component provides clipping and scrolling for menu content.
#[component]
pub fn NavigationMenuViewport(
    /// Whether the viewport is inert (prevents focus)
    inert: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build viewport classes and attributes
    let viewport_classes = format!(
        "navigation-menu-viewport {}",
        if inert { "inert" } else { "" }
    );

    let viewport_attributes = vec![
        ("data-inert", inert.to_string()),
    ];

    // Render the viewport
    view! {
        <div
            class=viewport_classes
            data-inert=inert
            // Accessibility attributes
            role="group"
            aria-label="Menu content"
            // Styling
            style=format!(
                "overflow: auto; max-height: 300px; z-index: {};",
                POPUP_Z_INDEX - 1
            )
        >
            {children()}
        </div>
    }
}

impl BaseUIComponentProps for NavigationMenuViewport {
    type Element = HtmlElement<div>;
}