//! Navigation Menu Icon component
//! 
//! The icon component renders the dropdown/chevron icon for triggers.

use leptos::*;
use leptos_ui_internals::{
    types::BaseUIComponentProps,
};
use crate::{
    constants::*,
};

/// Navigation Menu Icon component
/// 
/// The icon component renders the dropdown/chevron icon for triggers.
#[component]
pub fn NavigationMenuIcon(
    /// Whether the icon is open (pointing up/down vs left/right)
    open: bool,
    /// Children components (optional custom icon content)
    children: Option<Children>,
) -> impl IntoView {
    // Build icon classes and attributes
    let icon_classes = format!(
        "navigation-menu-icon {}",
        if open { "open" } else { "" }
    );

    let icon_attributes = vec![
        ("data-open", open.to_string()),
    ];

    // Default icon content (chevron)
    let default_icon = view! {
        <span class="icon-chevron">v</span>
    };

    // Render the icon
    view! {
        <span
            class=icon_classes
            data-open=open
            // Accessibility attributes
            aria-hidden="true"
            // Styling
            style=format!(
                "display: inline-block; transition: transform {} {};",
                TRANSITION_DURATION,
                TRANSITION_TIMING_FUNCTION
            )
        >
            {children.unwrap_or(default_icon)}
        </span>
    }
}

impl BaseUIComponentProps for NavigationMenuIcon {
    type Element = HtmlElement<span>;
}