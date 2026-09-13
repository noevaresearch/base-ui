//! Navigation Menu Link component
//! 
//! The link component represents navigation links within the menu.

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
};
use crate::{
    navigation_menu::types::NavigationMenuLinkProps,
    navigation_menu::constants::*,
};

/// Navigation Menu Link component
/// 
/// The link component represents navigation links within the menu.
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
    // Handle link click
    let handle_click = {
        let on_click = on_click.clone();
        
        move |event: leptos::ev::MouseEvent| {
            event.prevent_default();
            on_click(());
        }
    };

    // Handle link blur for focus management
    let handle_blur = {
        let href = href.clone();
        
        move |_| {
            // Handle blur event for focus management
            // This would integrate with the focus guard system
        }
    };

    // Render the link
    view! {
        <a
            href=href
            class="navigation-menu-link"
            data-active=active
            // Accessibility attributes
            aria-current=if active { "page" } else { "" }
            aria-expanded=active
            // Event handlers
            on:blur=handle_blur
            on:click=handle_click
        >
            {children()}
        </a>
    }
}