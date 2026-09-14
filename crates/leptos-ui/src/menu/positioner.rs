//! Menu positioner - handles positioning and collision detection
//! 
//! This is a port of Base UI's MenuPositioner from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store, MenuStoreContext};
use crate::menu::utils::{MenuSide, MenuAlign};

/// Props for the menu positioner component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuPositionerProps {
    /// Anchor element or reference
    #[prop(optional)]
    anchor: Option<web_sys::HtmlElement>,
    /// Side of the anchor where the menu should appear
    #[prop(default = MenuSide::Bottom)]
    side: MenuSide,
    /// Alignment of the menu relative to the anchor
    #[prop(default = MenuAlign::Start)]
    align: MenuAlign,
    /// Offset from the anchor side
    #[prop(default = 0)]
    side_offset: f32,
    /// Offset from the anchor alignment
    #[prop(default = 0)]
    align_offset: f32,
    /// Whether to avoid collisions with the viewport
    #[prop(default = true)]
    collision_avoidance: bool,
    /// Whether the positioner should be kept in the DOM
    #[prop(default = false)]
    keep_mounted: bool,
}

/// Positioner component for the menu
/// 
/// Handles positioning and collision detection for the menu popup.
#[component]
pub fn MenuPositioner(
    /// Props for the menu positioner
    #[prop(optional)]
    props: MenuPositionerProps,
) -> impl IntoView {
    let MenuPositionerProps {
        anchor,
        side,
        align,
        side_offset,
        align_offset,
        collision_avoidance,
        keep_mounted,
    } = props;
    
    let menu_store = use_menu_store();
    let open = menu_store.open();
    
    // State for positioning
    let position = create_rw_signal::<Option<(f32, f32)>>(None);
    let size = create_rw_signal::<Option<(f32, f32)>>(None);
    
    // Calculate position
    Effect::new(move || {
        if !open.get() && !keep_mounted {
            return;
        }
        
        // In a real implementation, we would calculate the position here
        // For now, we'll just set a default position
        if let Some(anchor_el) = &anchor {
            let anchor_rect = anchor_el.get_bounding_client_rect();
            let anchor_left = anchor_rect.left();
            let anchor_top = anchor_rect.top();
            let anchor_width = anchor_rect.width();
            let anchor_height = anchor_rect.height();
            
            let (x, y) = match side {
                MenuSide::Bottom => (
                    anchor_left + align_offset,
                    anchor_top + anchor_height + side_offset,
                ),
                MenuSide::Top => (
                    anchor_left + align_offset,
                    anchor_top - side_offset,
                ),
                MenuSide::Left => (
                    anchor_left - side_offset,
                    anchor_top + align_offset,
                ),
                MenuSide::Right => (
                    anchor_left + anchor_width + side_offset,
                    anchor_top + align_offset,
                ),
            };
            
            position.set(Some((x, y)));
            size.set(Some((300.0, 400.0))); // Default size
        } else {
            // Fallback position
            position.set(Some((0.0, 0.0)));
            size.set(Some((300.0, 400.0)));
        }
    });
    
    // Handle collision avoidance
    Effect::new(move || {
        if !collision_avoidance {
            return;
        }
        
        // In a real implementation, we would handle collision avoidance here
        // For now, we'll just log that we would handle it
        if let Some((x, y)) = position.get_untracked() {
            let viewport_width = window().inner_width().unwrap().as_f64().unwrap() as f32;
            let viewport_height = window().inner_height().unwrap().as_f64().unwrap() as f32;
            
            if x + 300.0 > viewport_width {
                // Adjust position to fit in viewport
                position.set(Some((viewport_width - 300.0, y)));
            }
            
            if y + 400.0 > viewport_height {
                // Adjust position to fit in viewport
                position.set(Some((x, viewport_height - 400.0)));
            }
        }
    });
    
    view! {
        <div
            class="menu-positioner"
            data-side=match side {
                MenuSide::Top => "top",
                MenuSide::Bottom => "bottom",
                MenuSide::Left => "left",
                MenuSide::Right => "right",
            }
            data-align=match align {
                MenuAlign::Start => "start",
                MenuAlign::Center => "center",
                MenuAlign::End => "end",
            }
            data-keep-mounted=keep_mounted
            style=position.get()
                .map(|(x, y)| format!("position: absolute; left: {}px; top: {}px;", x, y))
                .unwrap_or_default()
            hidden=!open.get() && !keep_mounted
        >
            {children()}
        </div>
    }
}

impl Default for MenuPositionerProps {
    fn default() -> Self {
        Self {
            anchor: None,
            side: MenuSide::Bottom,
            align: MenuAlign::Start,
            side_offset: 0.0,
            align_offset: 0.0,
            collision_avoidance: true,
            keep_mounted: false,
        }
    }
}

/// Hook to get menu positioner props
pub fn use_menu_positioner_props() -> MenuPositionerProps {
    MenuPositionerProps::default()
}

/// Hook to get the menu position
pub fn use_menu_position() -> Option<(f32, f32)> {
    // In a real implementation, this would read from the position signal
    None
}

/// Hook to get the menu size
pub fn use_menu_size() -> Option<(f32, f32)> {
    // In a real implementation, this would read from the size signal
    None
}

/// Hook to check if collision avoidance is enabled
pub fn use_menu_collision_avoidance() -> bool {
    // In a real implementation, this would read from props or context
    true
}

/// Hook to check if the positioner is kept mounted
pub fn use_menu_keep_mounted() -> bool {
    // In a real implementation, this would read from props or context
    false
}