//! Navigation Menu Root component
//! 
//! The root component manages the overall navigation menu state, including
//! the currently active item, open/close state, and provides context to
//! all child components.

use leptos::*;
use leptos_ui_internals::{
    use_render_element::{UseRenderElementComponentProps, use_render_element},
    use_transition_status::TransitionStatus,
};
use crate::{
    navigation_menu::types::{NavigationMenuRootProps, NavigationMenuContext, CloseReason, ActivationDirection},
    navigation_menu::constants::*,
};

/// Navigation Menu Root component
/// 
/// The root component manages the overall navigation menu state, including
/// the currently active item, open/close state, and provides context to
/// all child components.
#[component]
pub fn NavigationMenuRoot(
    /// The controlled value of the currently open menu item
    value: Option<String>,
    /// The default value when uncontrolled
    default_value: Option<String>,
    /// Callback when the value changes
    on_value_change: Callback<Option<String>, ()>,
    /// Delay before opening on hover (in milliseconds)
    delay: u32,
    /// Delay before closing on hover (in milliseconds)  
    close_delay: u32,
    /// Orientation of the menu
    orientation: Orientation,
    /// Whether the menu is nested inside another menu
    nested: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    // State management using simple signals
    let (current_value, set_value) = create_signal(value.or(default_value));

    // Computed open state
    let open = current_value().is_some();

    // Transition status for mounting/unmounting
    let (mounted, set_mounted) = create_signal(false);
    let transition_status = create_rw_signal(TransitionStatus::None);

    // Previous trigger element ref for focus management
    let prev_trigger_element = NodeRef::<HtmlElement<button>>::new();

    // Viewport inert state
    let viewport_inert = create_rw_signal(false);

    // Activation direction
    let activation_direction = create_rw_signal(None::<ActivationDirection>);

    // Positioner element ref
    let positioner_element = NodeRef::<HtmlElement<div>>::new();

    // Popup element ref
    let popup_element = NodeRef::<HtmlElement<nav>>::new();

    // Viewport element ref
    let viewport_element = NodeRef::<HtmlElement<nav>>::new();

    // Viewport target element ref
    let viewport_target_element = NodeRef::<HtmlElement<div>>::new();

    // Floating root context
    let floating_root_context = create_rw_signal(None);

    // Shared auto-size reset ref
    let popup_auto_size_reset_ref = create_rw_signal(None);

    // Handle value change
    let handle_value_change = {
        let set_value = set_value.clone();
        let viewport_inert = viewport_inert.clone();
        let activation_direction = activation_direction.clone();
        let floating_root_context = floating_root_context.clone();
        
        move |new_value: Option<String>, reason: CloseReason| {
            set_value(new_value);
            viewport_inert.set(false);
            
            // Reset activation direction and floating context on close
            if new_value.is_none() {
                activation_direction.set(None);
                floating_root_context.set(None);
            }
        }
    };

    // Handle unmount (focus return logic)
    let handle_unmount = {
        let prev_trigger_element = prev_trigger_element.clone();
        
        move || {
            let blocked_reasons = [
                CloseReason::TriggerHover,
                CloseReason::OutsidePress,
                CloseReason::FocusOut,
            ];
            
            // Skip refocusing for certain close reasons
            if blocked_reasons.contains(&CloseReason::TriggerHover) 
                || blocked_reasons.contains(&CloseReason::OutsidePress) 
                || blocked_reasons.contains(&CloseReason::FocusOut) {
                return;
            }
            
            // Focus return logic
            if let Some(prev_trigger) = prev_trigger_element.get() {
                let document = document();
                if let Ok(body) = document.query_selector("body") {
                    if let Some(body) = body {
                        // Focus previous trigger only if focus is on body or inside popup
                        let is_focus_on_body = body.is_same_node(&prev_trigger);
                        
                        if is_focus_on_body {
                            let _ = prev_trigger.focus();
                        }
                    }
                }
            }
        }
    };

    // Context value to provide to children
    let context = NavigationMenuContext {
        value: current_value(),
        set_value: Box::new(move |new_value: Option<String>| {
            let current_val = current_value();
            if let Some(val) = &current_val {
                if let Some(new_val) = &new_value {
                    if val == new_val {
                        return; // No change
                    }
                }
            }
            
            handle_value_change(new_value, CloseReason::None);
        }),
        mounted: mounted(),
        transition_status: transition_status(),
        activation_direction: activation_direction.read().unwrap_or(None),
        position: None, // Will be set by positioner
    };

    // Create context provider
    provide_context::<NavigationMenuContext>(context);

    // Render the root element
    view! {
        <nav
            role="navigation"
            class="navigation-menu-root"
            data-nested=nested
            data-orientation=orientation
            // Additional attributes based on state
            data-mounted=mounted
            data-transition-status=transition_status
            data-open=open
        >
            {children()}
        </nav>
    }
}