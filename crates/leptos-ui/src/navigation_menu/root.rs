//! Navigation Menu Root component
//! 
//! The root component provides the main navigation menu context and manages state.

use leptos::prelude::*;

use crate::navigation_menu::types::*;

/// Navigation Menu Root component
/// 
/// The root component provides the main navigation menu context and manages state.
#[component]
pub fn NavigationMenuRoot<Value: 'static + Send + Sync + Clone + ToString + std::str::FromStr + Default>(
    /// The controlled value of the currently open menu item
    #[prop(default = None)]
    value: Option<Value>,
    /// The default value when uncontrolled
    #[prop(default = None)]
    default_value: Option<Value>,
    /// Delay before opening on hover (in milliseconds)
    #[prop(default = 50)]
    delay: u32,
    /// Delay before closing on hover (in milliseconds)  
    #[prop(default = 50)]
    close_delay: u32,
    /// Orientation of the menu
    #[prop(default = Orientation::Horizontal)]
    orientation: Orientation,
    /// Whether the menu is nested inside another menu
    #[prop(default = false)]
    nested: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Create signals for menu state
    let (current_value, set_current_value) = signal(value);
    let (mounted, set_mounted) = signal(false);
    
    // Build root classes and attributes
    let root_classes = format!(
        "navigation-menu root {} {}",
        if nested { "nested" } else { "" },
        match orientation {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        }
    );

    // Provide context and render children
    provide_context(NavigationMenuContext {
        value: current_value.get_untracked().map(|v| v.to_string()),
        set_value: Callback::new(move |new_value: Option<String>| {
            // Convert String back to Value - this is a simplification
            if let Some(val_str) = new_value {
                if let Ok(parsed_value) = val_str.parse::<Value>() {
                    set_current_value.set(Some(parsed_value));
                } else {
                    set_current_value.set(Some(Value::default()));
                }
            } else {
                set_current_value.set(None);
            }
        }),
        mounted: mounted.get_untracked(),
        activation_direction: None, // Placeholder - should be derived from actual interactions
        position: None, // Placeholder - should be derived from positioning logic
    });
    let children_view = children();

    // Render the root component
    view! {
        <div
            class=root_classes
            data-nested=nested
            data-orientation=match orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            }
            data-mounted=mounted
            // Accessibility attributes
            role="navigation"
            aria-label="Main navigation"
        >
            {children_view}
        </div>
    }
}