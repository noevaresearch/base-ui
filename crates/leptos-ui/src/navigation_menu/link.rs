//! Navigation Menu Link component
//! 
//! The link component represents a clickable link within the navigation menu.

use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::navigation_menu::types::*;

/// Navigation Menu Link component
/// 
/// The link component represents a clickable link within the navigation menu.
#[component]
pub fn NavigationMenuLink(
    /// The href for the link
    href: String,
    /// Whether this link is currently active
    #[prop(default = false)]
    active: bool,
    /// Callback when the link is clicked
    #[prop(default = || Callback::new(|_: ()| {}))]
    on_click: Callback<()>,
    /// Children components
    children: Children,
) -> impl IntoView {
    let link_classes = move || {
        format!(
            "navigation-menu-link {}",
            if active { "active" } else { "" }
        )
    };

    // Handle click events
    let on_click_handler = move |ev: MouseEvent| {
        ev.prevent_default();
        on_click.send(());
    };

    view! {
        <a
            class=link_classes
            href=href
            data-active=active
            aria-current=if active { "page" } else { "" }
            on:click=on_click_handler
        >
            {children()}
        </a>
    }
}