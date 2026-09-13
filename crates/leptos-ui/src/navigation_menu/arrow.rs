//! Navigation Menu Arrow component
//! 
//! The arrow component renders an arrow indicator for positioned popups.

use leptos::*;
use leptos_ui_internals::{
    types::BaseUIComponentProps,
};
use crate::{
    constants::*,
};

/// Navigation Menu Arrow component
/// 
/// The arrow component renders an arrow indicator for positioned popups.
#[component]
pub fn NavigationMenuArrow(
    /// Whether the arrow is centered
    centered: bool,
    /// Children components (optional arrow content)
    children: Option<Children>,
) -> impl IntoView {
    // Build arrow classes and attributes
    let arrow_classes = format!(
        "navigation-menu-arrow {}",
        if centered { "centered" } else { "" }
    );

    let arrow_attributes = vec![
        ("data-centered", centered.to_string()),
    ];

    // Render the arrow
    view! {
        <div
            class=arrow_classes
            data-centered=centered
            // Accessibility attributes
            role="presentation"
            // Styling
            style=format!(
                "position: absolute; width: 0; height: 0; border-style: solid; z-index: {};",
                POPUP_Z_INDEX + 1
            )
        >
            {children.map(|children| view! { {children()} })}
        </div>
    }
}

impl BaseUIComponentProps for NavigationMenuArrow {
    type Element = HtmlElement<div>;
}