//! Navigation Menu types and type definitions

use leptos::prelude::*;

/// Navigation Menu root component props
pub struct NavigationMenuRootProps<Value: 'static + Send + Sync> {
    /// The controlled value of the currently open menu item
    pub value: Option<Value>,
    /// The default value when uncontrolled
    pub default_value: Option<Value>,
    /// Delay before opening on hover (in milliseconds)
    pub delay: u32,
    /// Delay before closing on hover (in milliseconds)  
    pub close_delay: u32,
    /// Orientation of the menu
    pub orientation: Orientation,
    /// Whether the menu is nested inside another menu
    pub nested: bool,
    /// Children components
    pub children: Children,
}

/// Navigation Menu trigger props
pub struct NavigationMenuTriggerProps {
    /// Whether this trigger is currently active
    pub active: bool,
    /// Whether the trigger is disabled
    pub disabled: bool,
    /// Children components
    pub children: Children,
}

/// Navigation Menu content props
pub struct NavigationMenuContentProps {
    /// The value for this menu item
    pub value: String,
    /// Whether this content is currently active
    pub active: bool,
    /// Whether to keep the content mounted even when inactive
    pub keep_mounted: bool,
    /// Children components
    pub children: Children,
}

/// Navigation Menu link props
pub struct NavigationMenuLinkProps {
    /// The href for the link
    pub href: String,
    /// Whether this link is currently active
    pub active: bool,
    /// Children components
    pub children: Children,
}

/// Navigation Menu orientation
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Horizontal
    }
}

/// Navigation Menu position
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuPosition {
    pub side: Side,
    pub align: Align,
    pub anchor_hidden: bool,
}

/// Navigation Menu side
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

/// Navigation Menu alignment
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Align {
    Start,
    Center,
    End,
}

/// Navigation Menu activation direction
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum ActivationDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Navigation Menu context values - simplified to String for now
#[derive(Clone, Debug)]
pub struct NavigationMenuContext {
    pub value: Option<String>,
    pub set_value: Callback<Option<String>>,
    pub mounted: bool,
    pub activation_direction: Option<ActivationDirection>,
    pub position: Option<NavigationMenuPosition>,
}

/// Navigation Menu item context
#[derive(Clone, Debug)]
pub struct NavigationMenuItemContext {
    pub value: String,
}
