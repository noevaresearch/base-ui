//! Navigation Menu Positioner component
//! 
//! The positioner component handles positioning and styling for the popup.

use leptos::*;
use leptos_ui_internals::{
    hooks::use_positioner,
    types::BaseUIComponentProps,
};
use crate::{
    types::NavigationMenuPosition,
    constants::*,
};

/// Navigation Menu Positioner component
/// 
/// The positioner component handles positioning and styling for the popup.
#[component]
pub fn NavigationMenuPositioner(
    /// Position information
    position: Option<NavigationMenuPosition>,
    /// Whether the positioner is mounted
    mounted: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Use positioner hook for positioning
    let (_, positioner_props) = use_positioner(
        move || {
            position.clone()
        },
        move || {
            mounted
        },
    );

    // Build positioner classes and attributes
    let positioner_classes = format!(
        "navigation-menu-positioner {}",
        if mounted { "mounted" } else { "" }
    );

    let positioner_attributes = vec![
        ("data-mounted", mounted.to_string()),
        ("data-open", mounted.to_string()),
        ("data-side", position.as_ref().map(|p| p.side.to_string()).unwrap_or_default()),
        ("data-align", position.as_ref().map(|p| p.align.to_string()).unwrap_or_default()),
        ("data-anchor-hidden", position.as_ref().map(|p| p.anchor_hidden.to_string()).unwrap_or_default()),
    ];

    // Render the positioner
    view! {
        <div
            class=positioner_classes
            data-mounted=mounted
            data-open=mounted
            data-side=position.as_ref().map(|p| format!("{:?}", p.side)).unwrap_or_default()
            data-align=position.as_ref().map(|p| format!("{:?}", p.align)).unwrap_or_default()
            data-anchor-hidden=position.as_ref().map(|p| p.anchor_hidden.to_string()).unwrap_or_default()
            // Accessibility attributes
            role="presentation"
            // Styling
            style=format!(
                "position: absolute; z-index: {}; pointer-events: none; transition: all {} {};",
                TRIGGER_Z_INDEX,
                TRANSITION_DURATION,
                TRANSITION_TIMING_FUNCTION
            )
            {positioner_props}
        >
            {children()}
        </div>
    }
}

impl BaseUIComponentProps for NavigationMenuPositioner {
    type Element = HtmlElement<div>;
}