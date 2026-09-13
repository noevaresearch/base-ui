//! Navigation Menu Popup component
//! 
//! The popup component renders the popup container for menu content.

use leptos::*;
use leptos_ui_internals::{
    types::BaseUIComponentProps,
};
use crate::{
    types::NavigationMenuPosition,
    constants::*,
};

/// Navigation Menu Popup component
/// 
/// The popup component renders the popup container for menu content.
#[component]
pub fn NavigationMenuPopup(
    /// Position information for the popup
    position: Option<NavigationMenuPosition>,
    /// Whether the popup is mounted
    mounted: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build popup classes and attributes
    let popup_classes = format!(
        "navigation-menu-popup {}",
        if mounted { "mounted" } else { "" }
    );

    let popup_attributes = vec![
        ("data-mounted", mounted.to_string()),
        ("data-open", mounted.to_string()),
        ("data-side", position.as_ref().map(|p| p.side.to_string()).unwrap_or_default()),
        ("data-align", position.as_ref().map(|p| p.align.to_string()).unwrap_or_default()),
    ];

    // Render the popup
    view! {
        <nav
            class=popup_classes
            data-mounted=mounted
            data-open=mounted
            data-side=position.as_ref().map(|p| format!("{:?}", p.side)).unwrap_or_default()
            data-align=position.as_ref().map(|p| format!("{:?}", p.align)).unwrap_or_default()
            // Accessibility attributes
            role="menu"
            aria-hidden=!mounted
            // Tab index for keyboard navigation
            tabindex=(-1)
            // Styling
            style=format!(
                "position: absolute; z-index: {}; transition: all {} {};",
                POPUP_Z_INDEX,
                TRANSITION_DURATION,
                TRANSITION_TIMING_FUNCTION
            )
        >
            {children()}
        </nav>
    }
}

impl BaseUIComponentProps for NavigationMenuPopup {
    type Element = HtmlElement<nav>;
}