//! Navigation Menu Backdrop component
//! 
//! The backdrop component renders a backdrop behind the popup.

use leptos::*;
use leptos_ui_internals::{
    types::BaseUIComponentProps,
};
use crate::{
    constants::*,
};

/// Navigation Menu Backdrop component
/// 
/// The backdrop component renders a backdrop behind the popup.
#[component]
pub fn NavigationMenuBackdrop(
    /// Whether the backdrop is visible
    visible: bool,
    /// Children components (optional custom backdrop content)
    children: Option<Children>,
) -> impl IntoView {
    // Build backdrop classes and attributes
    let backdrop_classes = format!(
        "navigation-menu-backdrop {}",
        if visible { "visible" } else { "" }
    );

    let backdrop_attributes = vec![
        ("data-visible", visible.to_string()),
    ];

    // Render the backdrop
    view! {
        <div
            class=backdrop_classes
            data-visible=visible
            // Accessibility attributes
            role="presentation"
            // Styling
            style=format!(
                "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0, 0, 0, 0.1); z-index: {};",
                BACKDROP_Z_INDEX
            )
        >
            {children.map(|children| view! { {children()} })}
        </div>
    }
}

impl BaseUIComponentProps for NavigationMenuBackdrop {
    type Element = HtmlElement<div>;
}