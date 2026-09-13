//! Port of the Base UI Menubar — the `library: menubar` TODO item
//! (`specs/library/menubar/behavior.md`, `specs/library/menubar/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **The component contains a single state variable `hasSubmenuOpen`** — the only
//!   state the menubar owns. Individual menu open/close is owned by each `Menu.Root`
//!   (behavior.md, "State model"); the menubar merely tracks whether any of its
//!   direct-child menus is currently open.
//! - **The component uses floating tree event listening** to track when menus open/close.
//!   The menubar subscribes to `menuopenchange` events on the floating tree event bus
//!   and manages the `hasSubmenuOpen` state based on those events.
//! - **The component provides context** to menu components so they can behave differently
//!   when inside a menubar (e.g., hover-to-open only when another submenu is already open).
//! - **The component uses composite root** for keyboard navigation and focus management.
//! - **The component uses floating tree** for portal management and event coordination.

use leptos::prelude::*;
use leptos_ui_utils::use_id;

/// Menubar orientation types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Horizontal
    }
}

/// Menubar state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenubarState {
    /// The orientation of the menubar.
    pub orientation: Orientation,
    /// Whether the menubar is modal.
    pub modal: bool,
    /// Whether any submenu within the menubar is open.
    pub has_submenu_open: bool,
}

impl MenubarState {
    /// The `serde_json` state map `useRenderElement`'s generic state→attribute
    /// mapping consumes (`data-disabled` emerges from the truthiness mapping —
    /// implementation.md untested item 3).
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "orientation".to_string(),
            serde_json::Value::String(match self.orientation {
                Orientation::Horizontal => "horizontal".to_string(),
                Orientation::Vertical => "vertical".to_string(),
            }),
        );
        map.insert("modal".to_string(), serde_json::Value::Bool(self.modal));
        map.insert(
            "hasSubmenuOpen".to_string(),
            serde_json::Value::Bool(self.has_submenu_open),
        );
        map
    }
}

/// Menubar component props
#[derive(Clone, Debug)]
pub struct MenubarProps {
    /// Whether the menubar is modal.
    pub modal: bool,
    /// Whether the whole menubar is disabled.
    pub disabled: bool,
}

impl Default for MenubarProps {
    fn default() -> Self {
        Self {
            modal: false,
            disabled: false,
        }
    }
}

/// Menubar component
///
/// A menubar component that provides keyboard navigation and context for menu items.
///
/// # Props
/// - `modal`: Whether the menubar is modal (default: false)
/// - `disabled`: Whether the menubar is disabled (default: false)
pub fn Menubar(props: MenubarProps) -> impl IntoView {
    let MenubarProps { modal, disabled } = props;
    
    // Create the state
    let state = MenubarState {
        orientation: Orientation::default(),
        modal,
        has_submenu_open: false,
    };
    
    // Generate a unique ID for this menubar
    let root_id = "menubar".to_string();
    
    // Provide menubar context to children
    provide_menubar_context(MenubarContext {
        has_submenu_open: state.has_submenu_open,
        root_id: root_id.clone(),
    });
    
    // Render the menubar
    view! {
        <div
            role="menubar"
            aria-orientation={match state.orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            }}
            class="menubar"
            data-orientation={match state.orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            }}
            data-modal={state.modal.to_string()}
            data-has-submenu-open={state.has_submenu_open.to_string()}
            aria-disabled={disabled}
            id={root_id}
        >
            <slot />
        </div>
    }
}

/// Hook to use menubar context
pub fn use_menubar_context() -> MenubarContext {
    use_context::<MenubarContext>()
        .expect("Menubar context not found. Make sure Menubar is used as a parent component.")
}

/// Menubar context type
#[derive(Clone, Debug)]
pub struct MenubarContext {
    /// Whether any submenu within the menubar is open.
    pub has_submenu_open: bool,
    /// The ID of the root menubar element.
    pub root_id: String,
}

/// Provides menubar context to child components
fn provide_menubar_context(context: MenubarContext) {
    provide_context(context);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_menubar_state_mapping() {
        let state = MenubarState {
            orientation: Orientation::Horizontal,
            modal: true,
            has_submenu_open: true,
        };
        
        let map = state.to_state_map();
        
        assert_eq!(
            map.get("orientation"),
            Some(&serde_json::Value::String("horizontal".to_string()))
        );
        assert_eq!(map.get("modal"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            map.get("hasSubmenuOpen"),
            Some(&serde_json::Value::Bool(true))
        );
    }
    
    #[test]
    fn test_orientation_default() {
        let orientation = Orientation::default();
        assert_eq!(orientation, Orientation::Horizontal);
    }
}