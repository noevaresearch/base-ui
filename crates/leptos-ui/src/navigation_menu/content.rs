//! Navigation Menu Content component
//! 
//! The content component displays the menu content for a specific item.

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
    use_transition_status::TransitionStatus,
};
use crate::{
    navigation_menu::types::NavigationMenuContentProps,
    navigation_menu::constants::*,
};

/// Navigation Menu Content component
/// 
/// The content component displays the menu content for a specific item.
#[component]
pub fn NavigationMenuContent(
    /// The value for this menu item
    value: String,
    /// Whether this content is currently active
    active: bool,
    /// Whether to keep the content mounted even when inactive
    keep_mounted: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // Transition status for enter/exit animations
    let (mounted, set_mounted) = create_signal(active || keep_mounted);
    let transition_status = create_rw_signal(
        if active {
            TransitionStatus::Mounted
        } else {
            TransitionStatus::None
        }
    );

    // Handle open change completion
    let handle_open_change_complete = {
        let value = value.clone();
        
        move || {
            // Handle completion of open/close transition
            // This is used for cleanup and focus management
            if !active {
                // Content is closing, perform cleanup
                set_mounted(false);
            }
        }
    };

    // Content visibility based on active state and keep_mounted
    let is_visible = active || keep_mounted;
    let is_hidden = !is_visible;

    // Build content classes and attributes
    let content_classes = format!(
        "navigation-menu-content {}",
        if active { "active" } else { "" }
    );

    // Render the content
    view! {
        <nav
            class=content_classes
            data-value=value
            data-active=active
            data-keep-mounted=keep_mounted
            data-transition-status=transition_status
            data-open=active
            data-starting-style=if transition_status == TransitionStatus::Starting { "true" } else { "" }
            data-ending-style=if transition_status == TransitionStatus::Ending { "true" } else { "" }
            // Conditional attributes
            hidden=is_hidden
            // Accessibility attributes
            role="menu"
            aria-labelledby=format!("trigger-{}", value)
            aria-expanded=active
            // Tab index for keyboard navigation
            tabindex=-1
        >
            {children()}
        </nav>
    }
}