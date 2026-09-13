//! Navigation Menu Item component
//! 
//! The item component represents individual menu items and provides context
//! to its children (trigger and content).

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
};
use crate::{
    navigation_menu::types::{NavigationMenuItemContext, NavigationMenuItemValue},
    navigation_menu::constants::*,
};

/// Navigation Menu Item component
/// 
/// The item component represents individual menu items and provides context
/// to its children (trigger and content).
#[component]
pub fn NavigationMenuItem(
    /// The value for this menu item
    value: Option<String>,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Generate a unique ID if not provided
    let item_value = value.unwrap_or_else(|| {
        // Generate a unique ID using the current timestamp
        format!("item-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    });

    // Provide item context to children
    provide_context::<NavigationMenuItemContext>(NavigationMenuItemContext {
        value: item_value.clone(),
    });

    // Render the item
    view! {
        <div class="navigation-menu-item" data-value=item_value>
            {children()}
        </div>
    }
}

/// Navigation Menu Item value hook
/// 
/// Returns the current item value from context
pub fn use_navigation_menu_item_value() -> String {
    use_context::<NavigationMenuItemContext>()
        .map(|ctx| ctx.value)
        .unwrap_or_default()
}

/// Navigation Menu Item active hook
/// 
/// Returns whether this item is currently active based on the root context
pub fn use_navigation_menu_item_active() -> bool {
    // This would check against the root's current value
    // For now, return false as a placeholder
    false
}