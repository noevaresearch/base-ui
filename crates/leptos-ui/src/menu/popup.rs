//! Menu popup - the container for menu items
//! 
//! This is a port of Base UI's MenuPopup from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store};
use leptos::prelude::*;

/// Popup component for the menu
/// 
/// Renders the menu popup container that holds all menu items.
#[component]
pub fn MenuPopup(
    /// Whether the popup is modal
    #[prop(default = true)]
    modal: bool,
    /// Focus management options
    #[prop(optional)]
    final_focus: Option<web_sys::HtmlElement>,
    /// Popup content
    children: Children,
) -> impl IntoView {
    let menu_store = use_menu_store();
    let open = menu_store.open();
    
    view! {
        <div
            class="menu-popup"
            role="menu"
            aria-modal=modal
            data-open=open.get()
            style:display=open.get().then_some("block").unwrap_or_else(|| "none")
        >
            {children()}
        </div>
    }
}
