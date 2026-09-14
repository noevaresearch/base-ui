//! Menu portal - renders the menu popup in a different part of the DOM
//!
//! This is a port of Base UI's MenuPortal from React to Leptos.

use crate::menu::store::{MenuStoreContext, use_menu_store};
use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use wasm_bindgen::JsCast;

/// Portal component for the menu
///
/// Renders the menu popup in a different part of the DOM.
#[component]
pub fn MenuPortal(
    /// Whether the portal should be kept mounted
    #[prop(default = false)]
    keep_mounted: bool,
    /// The container element for the portal
    #[prop(optional)]
    container: Option<web_sys::HtmlElement>,
    children: Children,
) -> impl IntoView {
    let menu_store = use_menu_store();
    let open = menu_store.open();

    // For now, we'll just render the children without actual portal functionality
    // In a real implementation, this would move the DOM nodes to a different location

    view! {
        <div
            class="menu-portal"
            data-keep-mounted=keep_mounted
            hidden=!open.get() && !keep_mounted
        >
            {children()}
        </div>
    }
}

/// Hook to check if the portal is kept mounted
pub fn use_menu_portal_keep_mounted() -> bool {
    // In a real implementation, this would read from props or context
    false
}

/// Hook to get the portal container
pub fn use_menu_portal_container() -> Option<web_sys::HtmlElement> {
    // In a real implementation, this would read from props or context
    None
}

/// Helper function to create a portal container
pub fn create_portal_container() -> web_sys::HtmlElement {
    let div = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|doc| doc.create_element("div").ok())
        .unwrap();
    div.set_class_name("menu-portal-container");
    div.dyn_into().unwrap()
}

/// Helper function to append a portal container to the body
pub fn append_portal_container_to_body(container: &web_sys::HtmlElement) {
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        body.append_child(container).unwrap();
    }
}

/// Helper function to remove a portal container from the DOM
pub fn remove_portal_container(container: &web_sys::HtmlElement) {
    if container.parent_node().is_some() {
        container.remove();
    }
}
