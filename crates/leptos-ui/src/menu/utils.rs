//! Menu utilities - helper functions and constants
//! 
//! This is a port of Base UI's menu utilities from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use std::rc::Rc;

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
    pub const DROPDOWN_COLLISION_AVOIDANCE: bool = true;
    
    /// Popup collision avoidance
    pub const POPUP_COLLISION_AVOIDANCE: bool = true;
    
    /// Default hover open delay in milliseconds
    pub const HOVER_OPEN_DELAY: u32 = 200;
    
    /// Default hover close delay in milliseconds
    pub const HOVER_CLOSE_DELAY: u32 = 200;
    
    /// Default outside press grace period in milliseconds
    pub const OUTSIDE_PRESS_GRACE_PERIOD: u32 = 500;
    
    /// Default touch close shield in milliseconds
    pub const TOUCH_CLOSE_SHIELD: u32 = 300;
}

/// Menu state attributes mapping
pub fn state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-open", "open"),
        ("data-closed", "closed"),
        ("data-transitioning", "transitioning"),
        ("data-starting-style", "starting"),
        ("data-ending-style", "ending"),
        ("data-instant", "instant"),
    ]
}

/// Menu item attributes mapping
pub fn item_state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-highlighted", "highlighted"),
        ("data-disabled", "disabled"),
        ("data-checked", "checked"),
        ("data-unchecked", "unchecked"),
    ]
}

/// Menu popup attributes mapping
pub fn popup_state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-open", "open"),
        ("data-closed", "closed"),
        ("data-transitioning", "transitioning"),
        ("data-starting-style", "starting"),
        ("data-ending-style", "ending"),
        ("data-instant", "instant"),
        ("data-rootownerid", "root-owner"),
    ]
}

/// Menu trigger attributes mapping
pub fn trigger_state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-popup-open", "popup-open"),
        ("data-pressed", "pressed"),
        ("data-disabled", "disabled"),
    ]
}

/// Menu arrow attributes mapping
pub fn arrow_state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-arrow", "arrow"),
        ("data-arrow-hidden", "arrow-hidden"),
    ]
}

/// Menu backdrop attributes mapping
pub fn backdrop_state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-backdrop", "backdrop"),
        ("data-backdrop-hidden", "backdrop-hidden"),
    ]
}

/// Menu viewport attributes mapping
pub fn viewport_state_attributes_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data-viewport", "viewport"),
        ("data-current", "current"),
        ("data-previous", "previous"),
        ("data-activation-direction", "activation-direction"),
    ]
}

/// Generate menu CSS custom properties
pub fn css_custom_properties() -> Vec<(&'static str, &'static str)> {
    vec![
        ("--available-width", "100%"),
        ("--available-height", "100%"),
        ("--anchor-width", "auto"),
        ("--anchor-height", "auto"),
        ("--transform-origin", "center"),
        ("--positioner-width", "auto"),
        ("--positioner-height", "auto"),
        ("--arrow-size", "8px"),
    ]
}

/// Menu keyboard navigation constants
pub mod keyboard {
    /// Arrow down key
    pub const ARROW_DOWN: &str = "ArrowDown";
    
    /// Arrow up key
    pub const ARROW_UP: &str = "ArrowUp";
    
    /// Arrow left key
    pub const ARROW_LEFT: &str = "ArrowLeft";
    
    /// Arrow right key
    pub const ARROW_RIGHT: &str = "ArrowRight";
    
    /// Home key
    pub const HOME: &str = "Home";
    
    /// End key
    pub const END: &str = "End";
    
    /// Enter key
    pub const ENTER: &str = "Enter";
    
    /// Space key
    pub const SPACE: &str = " ";
    
    /// Escape key
    pub const ESCAPE: &str = "Escape";
    
    /// Tab key
    pub const TAB: &str = "Tab";
    
    /// Shift key
    pub const SHIFT: &str = "Shift";
}

/// Menu event reasons
#[derive(Clone, Debug, PartialEq)]
pub enum MenuEventReason {
    /// Item was pressed
    ItemPress,
    /// Menu was cancelled
    CancelOpen,
    /// Click outside the menu
    OutsidePress,
    /// Sibling menu opened
    SiblingOpen,
    /// Trigger was hovered
    TriggerHover,
}

impl std::fmt::Display for MenuEventReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuEventReason::ItemPress => write!(f, "itemPress"),
            MenuEventReason::CancelOpen => write!(f, "cancelOpen"),
            MenuEventReason::OutsidePress => write!(f, "outsidePress"),
            MenuEventReason::SiblingOpen => write!(f, "siblingOpen"),
            MenuEventReason::TriggerHover => write!(f, "triggerHover"),
        }
    }
}

/// Menu interaction types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuInteractionType {
    /// Mouse interaction
    Mouse,
    /// Keyboard interaction
    Keyboard,
    /// Touch interaction
    Touch,
}

/// Menu position sides
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuSide {
    Top,
    Bottom,
    Left,
    Right,
}

impl MenuSide {
    /// Get the opposite side
    pub fn opposite(self) -> Self {
        match self {
            MenuSide::Top => MenuSide::Bottom,
            MenuSide::Bottom => MenuSide::Top,
            MenuSide::Left => MenuSide::Right,
            MenuSide::Right => MenuSide::Left,
        }
    }
}

/// Menu alignments
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuAlign {
    Start,
    Center,
    End,
}

/// Menu focus management options
#[derive(Clone, Debug, PartialEq)]
pub enum MenuFocusManagement {
    /// Return focus to the trigger
    ReturnFocus,
    /// Focus a specific element
    FinalFocus(web_sys::HtmlElement),
    /// Don't manage focus
    None,
}

impl Default for MenuFocusManagement {
    fn default() -> Self {
        MenuFocusManagement::ReturnFocus
    }
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
        
        // Get parent element
        current = el.parent_element();
    }
    
    None
}

/// Helper function to create a keyboard event
pub fn create_keyboard_event(key: &str, code: &str) -> web_sys::KeyboardEvent {
    web_sys::KeyboardEvent::new_with_event_init_dict(
        key,
        &web_sys::KeyboardEventInit::new().with_code(code),
    )
    .unwrap()
}

/// Helper function to create a mouse event
pub fn create_mouse_event(event_type: &str) -> web_sys::MouseEvent {
    web_sys::MouseEvent::new(event_type).unwrap()
}

/// Helper function to create a focus event
pub fn create_focus_event() -> web_sys::FocusEvent {
    web_sys::FocusEvent::new("focus").unwrap()
}

/// Helper function to create a blur event
pub fn create_blur_event() -> web_sys::FocusEvent {
    web_sys::FocusEvent::new("blur").unwrap()
}

/// Menu utility functions
pub mod utils {
    use super::*;
    
    /// Check if a key is a navigation key
    pub fn is_navigation_key(key: &str) -> bool {
        matches!(
            key,
            keyboard::ARROW_DOWN
                | keyboard::ARROW_UP
                | keyboard::ARROW_LEFT
                | keyboard::ARROW_RIGHT
                | keyboard::HOME
                | keyboard::END
                | keyboard::TAB
        )
    }
    
    /// Check if a key is an activation key
    pub fn is_activation_key(key: &str) -> bool {
        matches!(key, keyboard::ENTER | keyboard::SPACE)
    }
    
    /// Check if a key is a closing key
    pub fn is_closing_key(key: &str) -> bool {
        key == keyboard::ESCAPE
    }
    
    /// Check if a key is a text input key
    pub fn is_text_input_key(key: &str) -> bool {
        // This is a simplified check - in reality, we'd need to check more keys
        key.len() == 1 && key.is_alphabetic()
    }
    
    /// Normalize a string for typeahead matching
    pub fn normalize_for_typeahead(text: &str) -> String {
        text
            .chars()
            .map(|c| c.to_lowercase())
            .collect::<String>()
    }
    
    /// Check if a string matches a typeahead query
    pub fn matches_typeahead(text: &str, query: &str) -> bool {
        let normalized_text = normalize_for_typeahead(text);
        let normalized_query = normalize_for_typeahead(query);
        
        normalized_text.contains(&normalized_query)
    }
    
    /// Get the next item in a list for navigation
    pub fn get_next_item_index(
        current_index: Option<usize>,
        direction: i32,
        items_len: usize,
        loop_focus: bool,
    ) -> Option<usize> {
        if items_len == 0 {
            return None;
        }

        let current = current_index.unwrap_or(0);
        let next = (current as i32 + direction) as usize;

        if loop_focus {
            Some(next % items_len)
        } else if next < items_len {
            Some(next)
        } else {
            None
        }
    }
}