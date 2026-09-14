//! Menu utilities - helper functions and constants
//!
//! This is a port of Base UI's menu utilities from React to Leptos.

use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;

/// Menu-related enums and types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSide {
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuInteractionType {
    Mouse,
    Keyboard,
    Touch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuEventReason {
    MouseClick,
    KeyboardSelect,
    KeyboardClose,
    PointerEnter,
    PointerLeave,
    Focus,
    Blur,
    ValueChange,
}

impl std::fmt::Display for MenuEventReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuEventReason::MouseClick => write!(f, "mouseClick"),
            MenuEventReason::KeyboardSelect => write!(f, "keyboardSelect"),
            MenuEventReason::KeyboardClose => write!(f, "keyboardClose"),
            MenuEventReason::PointerEnter => write!(f, "pointerEnter"),
            MenuEventReason::PointerLeave => write!(f, "pointerLeave"),
            MenuEventReason::Focus => write!(f, "focus"),
            MenuEventReason::Blur => write!(f, "blur"),
            MenuEventReason::ValueChange => write!(f, "valueChange"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuContext {
    pub id: String,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuGroupContext(Rc<(String, Option<web_sys::HtmlElement>)>);

/// Menu-related constants
pub mod constants {
    /// Patient click threshold in milliseconds
    pub const PATIENT_CLICK_THRESHOLD: u32 = 1000;

    /// Typeahead reset delay in milliseconds
    pub const TYPEAHEAD_RESET_MS: u32 = 500;

    /// Dropdown collision avoidance
    pub const COLLISION_PADDING: f32 = 8.0;

    /// Menu animation duration
    pub const ANIMATION_DURATION: f32 = 0.2;

    /// Menu z-index
    pub const MENU_Z_INDEX: i32 = 1000;
}

/// Helper function to create menu event details
pub fn create_menu_event_details(
    reason: MenuEventReason,
    interaction_type: MenuInteractionType,
) -> String {
    format!(
        "{{ reason: '{}', interactionType: '{:?}' }}",
        reason, interaction_type
    )
}

/// Helper function to check if an element is disabled
pub fn is_element_disabled(element: &web_sys::HtmlElement) -> bool {
    element
        .get_attribute("disabled")
        .map_or(false, |d| d == "disabled" || d == "")
}

/// Helper function to check if an element is hidden
pub fn is_element_hidden(element: &web_sys::HtmlElement) -> bool {
    element
        .get_attribute("hidden")
        .map_or(false, |h| h == "hidden")
}

/// Helper function to find the root owner ID
pub fn find_root_owner_id(element: &web_sys::HtmlElement) -> Option<String> {
    // Walk up the DOM tree to find the root owner ID
    let mut current = Some(element.clone());

    while let Some(el) = current {
        if let Some(owner_id) = el.get_attribute("data-rootownerid") {
            return Some(owner_id);
        }

        if let Some(parent) = el.parent_element() {
            current = Some(parent.dyn_into().ok()?);
        } else {
            break;
        }
    }

    None
}

/// Helper function to check if a key is alphabetic
pub fn is_key_alphabetic(key: &str) -> bool {
    key.len() == 1 && key.chars().all(|c| c.is_alphabetic())
}

/// Helper function to normalize text for typeahead
pub fn normalize_text_for_typeahead(text: &str) -> String {
    text.chars().map(|c| c.to_lowercase().to_string()).collect()
}

/// Helper function to check if an event is a click-like event
pub fn is_click_like_event(event: &web_sys::Event) -> bool {
    matches!(event.type_().as_str(), "click" | "mousedown" | "mouseup")
}

/// Helper function to get the active element
pub fn get_active_element() -> Option<web_sys::HtmlElement> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element())
        .and_then(|el| el.dyn_into().ok())
}

/// Helper function to check if an element is in the viewport
pub fn is_element_in_viewport(element: &web_sys::HtmlElement) -> bool {
    let rect = element.get_bounding_client_rect();
    rect.width() > 0.0 && rect.height() > 0.0
}

/// Helper function to scroll an element into view
pub fn scroll_element_into_view(element: &web_sys::HtmlElement) {
    element.scroll_into_view_with_bool(true);
}

/// Helper function to get the scroll position
pub fn get_scroll_position() -> (f64, f64) {
    if let Some(window) = web_sys::window() {
        let x = window.scroll_x().unwrap_or(0.0);
        let y = window.scroll_y().unwrap_or(0.0);
        (x, y)
    } else {
        (0.0, 0.0)
    }
}

/// Helper function to set the scroll position
pub fn set_scroll_position(x: f64, y: f64) {
    if let Some(window) = web_sys::window() {
        window.scroll_with_x_and_y(x, y);
    }
}

/// Helper function to get the window dimensions
pub fn get_window_dimensions() -> (f64, f64) {
    if let Some(window) = web_sys::window() {
        let width = window.inner_width().unwrap().as_f64().unwrap_or(0.0);
        let height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);
        (width, height)
    } else {
        (0.0, 0.0)
    }
}

/// Helper function to check if an element is focused
pub fn is_element_focused(element: &web_sys::HtmlElement) -> bool {
    get_active_element()
        .map(|active| active == *element)
        .unwrap_or(false)
}

/// Helper function to focus an element
pub fn focus_element(element: &web_sys::HtmlElement) {
    element.focus();
}

/// Helper function to blur an element
pub fn blur_element(element: &web_sys::HtmlElement) {
    element.blur();
}
