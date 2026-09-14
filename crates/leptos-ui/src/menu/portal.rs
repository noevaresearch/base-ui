//! Menu portal - renders the menu popup in a different part of the DOM
//! 
//! This is a port of Base UI's MenuPortal from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store};

/// Props for the menu portal component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuPortalProps {
    /// Whether the portal should be kept mounted
    #[prop(default = false)]
    keep_mounted: bool,
    /// The container element for the portal
    #[prop(optional)]
    container: Option<web_sys::HtmlElement>,
}

/// Portal component for the menu
/// 
/// Renders the menu popup in a different part of the DOM.
#[component]
pub fn MenuPortal(
    /// Props for the menu portal
    #[prop(optional)]
    props: MenuPortalProps,
) -> impl IntoView {
    let MenuPortalProps { keep_mounted, container } = props;
    
    let menu_store = use_menu_store();
    let open = menu_store.open();
    
    // Create a container for the portal if none is provided
    let portal_container = if let Some(container) = container {
        container
    } else {
        // Create a default container
        let div = document().create_element("div").unwrap();
        div.set_class_name("menu-portal-container");
        document().body().unwrap().append_child(&div).unwrap();
        div.dyn_into().unwrap()
    };
    
    // Effect to clean up the portal container when unmounted
    Effect::new(move || {
        move || {
            if !keep_mounted {
                // Remove the portal container from the DOM
                if portal_container.parent_node().is_some() {
                    portal_container.remove();
                }
            }
        }
    });
    
    // Render the portal children
    view! {
        <Portal mount=portal_container>
            {children()}
        </Portal>
    }
}

impl Default for MenuPortalProps {
    fn default() -> Self {
        Self {
            keep_mounted: false,
            container: None,
        }
    }
}

/// Hook to get menu portal props
pub fn use_menu_portal_props() -> MenuPortalProps {
    MenuPortalProps::default()
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
    let div = document().create_element("div").unwrap();
    div.set_class_name("menu-portal-container");
    div.dyn_into().unwrap()
}

/// Helper function to append a portal container to the body
pub fn append_portal_container_to_body(container: &web_sys::HtmlElement) {
    document().body().unwrap().append_child(container).unwrap();
}

/// Helper function to remove a portal container from the DOM
pub fn remove_portal_container(container: &web_sys::HtmlElement) {
    if container.parent_node().is_some() {
        container.remove();
    }
}