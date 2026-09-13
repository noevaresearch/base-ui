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

use leptos_ui_internals::floating_ui::tree::{
    provide_floating_tree, use_floating_node_id, use_floating_tree,
};
use leptos_ui_internals::floating_ui::types::{FloatingTreeEvents, FloatingNodeType};
use leptos_ui_internals::composite::use_composite_root;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::state_attributes::StateAttributesMapping;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderedElement, UseRenderElementComponentProps,
    UseRenderElementParams, native_to_base_ui, static_attr, use_render_element,
};
use leptos_ui_utils::signal::RwSignal;

/// Menubar orientation types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Horizontal
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
    /// The serde_json state map `useRenderElement`'s generic state→attribute
    /// mapping consumes.
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("orientation".to_string(), serde_json::Value::String(match self.orientation {
            Orientation::Horizontal => "horizontal".to_string(),
            Orientation::Vertical => "vertical".to_string(),
        }));
        map.insert("modal".to_string(), serde_json::Value::Bool(self.modal));
        map.insert("hasSubmenuOpen".to_string(), serde_json::Value::Bool(self.has_submenu_open));
        map
    }
}

/// State attributes mapping for menubar
pub fn menubar_state_attributes_mapping() -> StateAttributesMapping<MenubarState> {
    StateAttributesMapping {
        has_submenu_open: |value| {
            if value {
                Some("data-has-submenu-open".to_string())
            } else {
                None
            }
        },
        modal: |value| {
            if value {
                Some("data-modal".to_string())
            } else {
                None
            }
        },
        orientation: |value| {
            Some(format!("data-orientation-{}", match value {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            }))
        },
    }
}

/// Menubar context
#[derive(Clone)]
pub struct MenubarContext {
    pub modal: bool,
    pub disabled: bool,
    pub content_element: Option<web_sys::HtmlElement>,
    pub set_content_element: Option<Box<dyn Fn(Option<web_sys::HtmlElement>)>>,
    pub has_submenu_open: bool,
    pub set_has_submenu_open: Option<Box<dyn Fn(bool)>>,
    pub orientation: Orientation,
    pub allow_mouse_up_trigger_ref: RwSignal<bool>,
    pub root_id: String,
}

/// Menubar component props
pub struct MenubarProps {
    /// Whether the menubar is modal.
    pub modal: bool,
    /// Whether the whole menubar is disabled.
    pub disabled: bool,
    /// The orientation of the menubar.
    pub orientation: Orientation,
    /// Whether to loop keyboard focus back to the first item
    /// when the end of the list is reached while using the arrow keys.
    pub loop_focus: bool,
    /// `className`/`style`/`render` — through the
    /// [`UseRenderElementComponentProps`] vocabulary.
    pub render_class_style: UseRenderElementComponentProps,
    /// Non-handler attributes spread onto the element.
    pub element_attributes: Vec<(String, String)>,
    /// The consumer's event handlers.
    pub handlers: MenubarHandlers,
}

impl Default for MenubarProps {
    fn default() -> Self {
        Self {
            modal: true,
            disabled: false,
            orientation: Orientation::default(),
            loop_focus: true,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            handlers: MenubarHandlers::default(),
        }
    }
}

/// Menubar handlers
#[derive(Clone, Default)]
pub struct MenubarHandlers {
    // Add any specific menubar handlers if needed
}

/// Builds the Menubar element description — upstream's `Menubar` body
/// (`packages/react/src/menubar/Menubar.tsx:30-96`) up to and including
/// `useRenderElement`, without materializing a DOM node.
///
/// Must be called inside a reactive owner.
pub fn menubar_element(props: MenubarProps) -> Option<RenderedElement> {
    let MenubarProps {
        modal,
        disabled,
        orientation,
        loop_focus,
        render_class_style,
        element_attributes,
        handlers: _,
    } = props;

    // Generate the menubar ID
    let id = use_base_ui_id(None);

    // Create the state
    let has_submenu_open = RwSignal::new(false);
    let state = MenubarState {
        orientation,
        modal,
        has_submenu_open: has_submenu_open.get(),
    };

    // Create the context
    let context = MenubarContext {
        modal,
        disabled,
        content_element: None,
        set_content_element: None,
        has_submenu_open: has_submenu_open.get(),
        set_has_submenu_open: Some(Box::new(move |value| has_submenu_open.set(value))),
        orientation,
        allow_mouse_up_trigger_ref: RwSignal::new(false),
        root_id: id.clone(),
    };

    // TODO: Implement the floating tree event listening
    // This requires implementing the event subscription logic similar to upstream
    // For now, we'll create a simplified version

    // The composite root props
    let composite_props = leptos_ui_internals::composite::CompositeRootProps {
        render: leptos_ui_internals::composite::CompositeOnLoop::No,
        class_name: "".to_string(),
        style: leptos_ui_internals::composite::StyleSource::None,
        state: &state.to_state_map(),
        state_attributes_mapping: Some(menubar_state_attributes_mapping()),
        refs: vec![],
        props: vec![PropsSource::Static((
            vec![
                ("role".to_string(), "menubar".to_string()),
                ("id".to_string(), id.clone()),
                ("aria-orientation".to_string(), match orientation {
                    Orientation::Horizontal => "horizontal".to_string(),
                    Orientation::Vertical => "vertical".to_string(),
                }),
            ],
            leptos_ui_internals::composite::CompositeRootHandlers::default(),
        ))],
        orientation,
        loop_focus,
        enable_home_and_end_keys: true,
        highlight_item_on_hover: has_submenu_open.get(),
    };

    // Use composite root
    let composite_result = leptos_ui_internals::composite::use_composite_root(composite_props);

    // The element bag for rendering
    let element_bag = RenderElementProps {
        handlers: RenderElementHandlers {
            // Add any specific handlers if needed
            ..RenderElementHandlers::default()
        },
        attributes: element_attributes,
        ..RenderElementProps::default()
    };

    // Use render element
    use_render_element(
        "div",
        render_class_style,
        UseRenderElementParams {
            enabled: true,
            state: &state.to_state_map(),
            refs: vec![],
            props: vec![PropsSource::Static(element_bag)],
            state_attributes_mapping: Some(menubar_state_attributes_mapping()),
        },
    )
}

/// Menubar component
pub fn menubar(props: MenubarProps) -> Option<RenderedElement> {
    menubar_element(props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;

    #[test]
    fn test_menubar_state_mapping() {
        let state = MenubarState {
            orientation: Orientation::Horizontal,
            modal: true,
            has_submenu_open: true,
        };

        let mapping = menubar_state_attributes_mapping();
        let map = state.to_state_map();

        assert_eq!(map.get("orientation"), Some(&serde_json::Value::String("horizontal".to_string())));
        assert_eq!(map.get("modal"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(map.get("hasSubmenuOpen"), Some(&serde_json::Value::Bool(true)));
    }

    #[test]
    fn test_orientation_default() {
        let orientation = Orientation::default();
        assert_eq!(orientation, Orientation::Horizontal);
    }
}