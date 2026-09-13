//! Navigation Menu Portal component
//! 
//! The portal component handles rendering content outside the normal flow.

use leptos::*;
use leptos_ui_internals::{
    types::BaseUIComponentProps,
};
use crate::{
    constants::*,
};

/// Navigation Menu Portal component
/// 
/// The portal component handles rendering content outside the normal flow.
#[component]
pub fn NavigationMenuPortal(
    /// Whether to keep the portal mounted
    keep_mounted: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build portal classes and attributes
    let portal_classes = format!(
        "navigation-menu-portal {}",
        if keep_mounted { "keep-mounted" } else { "" }
    );

    let portal_attributes = vec![
        ("data-keep-mounted", keep_mounted.to_string()),
    ];

    // Render the portal
    view! {
        <div
            class=portal_classes
            data-keep-mounted=keep_mounted
            // Accessibility attributes
            role="presentation"
            // Styling
            style=format!(
                "position: fixed; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: {};",
                POPUP_Z_INDEX
            )
        >
            {children()}
        </div>
    }
}

impl BaseUIComponentProps for NavigationMenuPortal {
    type Element = HtmlElement<div>;
}