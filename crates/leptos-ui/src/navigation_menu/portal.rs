//! Navigation Menu Portal component
//! 
//! The portal component renders content outside the normal DOM flow.

use leptos::*;
use leptos::html::div;

use crate::{
    navigation_menu::constants::*,
};

/// Navigation Menu Portal component
/// 
/// The portal component renders content outside the normal DOM flow.
#[component]
pub fn NavigationMenuPortal(
    /// Whether the portal is mounted
    mounted: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Build portal classes and attributes
    let portal_classes = format!(
        "navigation-menu-portal {}",
        if mounted { "mounted" } else { "" }
    );

    // Render the portal
    view! {
        <div
            class=portal_classes
            data-mounted=mounted
        >
            {children()}
        </div>
    }
}