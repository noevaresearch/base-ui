//! Navigation Menu Link component
//! 
//! The link component handles navigation links within the menu.

use leptos::*;
use leptos::html::a;
use leptos::ev::MouseEvent;

use crate::{
    navigation_menu::{
        constants::*,
        types::{NavigationMenuLinkProps},
    },
};

/// Navigation Menu Link component
/// 
/// The link component handles navigation links within the menu.
#[component]
pub fn NavigationMenuLink(
    /// The href for the link
    href: String,
    /// Whether this link is currently active
    active: bool,
    /// Callback when the link is clicked
    on_click: Callback<()>,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Create handler for link clicks
    let handle_click = move |ev: MouseEvent| {
        ev.prevent_default();
        on_click.call(());
    };

    // Build link classes and attributes
    let link_classes = format!(
        "navigation-menu-link {}",
        if active { "active" } else { "" }
    );

    // Render the link component
    view! {
        <a
            class=link_classes
            data-active=active
            href=href
            // Accessibility attributes
            aria-current=if active { "page" } else { "" }
            // Event handlers
            on:click=handle_click
        >
            {children()}
        </a>
    }
}