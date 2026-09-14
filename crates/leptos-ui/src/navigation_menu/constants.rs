//! Navigation Menu constants

/// Patient click threshold in milliseconds
pub const PATIENT_CLICK_THRESHOLD: u32 = 700;

/// Visually hidden class for accessibility
pub const OWNER_VISUALLY_HIDDEN: &str = "visually-hidden";

/// Popup collision avoidance configuration
pub const POPUP_COLLISION_AVOIDANCE: f32 = 5.0;

/// Dropdown collision avoidance configuration  
pub const DROPDOWN_COLLISION_AVOIDANCE: f32 = 5.0;

/// Navigation menu trigger identifier for event handling
pub const NAVIGATION_MENU_TRIGGER_IDENTIFIER: &str = "data-navigation-menu-trigger";

/// Navigation menu CSS custom property names
pub const POSITIONER_WIDTH_VAR: &str = "--positioner-width";
pub const POSITIONER_HEIGHT_VAR: &str = "--positioner-height";
pub const POPUP_WIDTH_VAR: &str = "--popup-width";
pub const POPUP_HEIGHT_VAR: &str = "--popup-height";

/// Navigation menu data attribute names
pub const DATA_OPEN: &str = "data-open";
pub const DATA_STARTING_STYLE: &str = "data-starting-style";
pub const DATA_ENDING_STYLE: &str = "data-ending-style";
pub const DATA_SIDE: &str = "data-side";
pub const DATA_ACTIVATION_DIRECTION: &str = "data-activation-direction";

/// Default delay values
pub const DEFAULT_DELAY: u32 = 200;
pub const DEFAULT_CLOSE_DELAY: u32 = 100;

/// Focus guard class names
pub const FOCUS_GUARD_BEFORE_INSIDE: &str = "focus-guard-before-inside";
pub const FOCUS_GUARD_AFTER_INSIDE: &str = "focus-guard-after-inside";
pub const FOCUS_GUARD_BEFORE_OUTSIDE: &str = "focus-guard-before-outside";
pub const FOCUS_GUARD_AFTER_OUTSIDE: &str = "focus-guard-after-outside";

/// Menu state attribute mappings
pub const POPUP_OPEN_STATE_MAPPING: &[&str] = &[DATA_OPEN, DATA_STARTING_STYLE, DATA_ENDING_STYLE];

pub const TRIGGER_OPEN_STATE_MAPPING: &[&str] = &[DATA_OPEN, DATA_ACTIVATION_DIRECTION];

pub const ICON_OPEN_STATE_MAPPING: &[&str] = &[DATA_OPEN];

pub const TRANSITION_STATE_MAPPING: &[&str] = &[DATA_STARTING_STYLE, DATA_ENDING_STYLE];

/// Navigation menu keyboard event keys
pub const KEY_ARROW_DOWN: &str = "ArrowDown";
pub const KEY_ARROW_UP: &str = "ArrowUp";
pub const KEY_ARROW_LEFT: &str = "ArrowLeft";
pub const KEY_ARROW_RIGHT: &str = "ArrowRight";
pub const KEY_ESCAPE: &str = "Escape";

/// Navigation menu CSS transition properties
pub const TRANSITION_DURATION: &str = "150ms";
pub const TRANSITION_TIMING_FUNCTION: &str = "cubic-bezier(0.4, 0, 0.2, 1)";

/// Navigation menu z-index values
pub const TRIGGER_Z_INDEX: i32 = 1;
pub const POPUP_Z_INDEX: i32 = 50;
pub const BACKDROP_Z_INDEX: i32 = 40;
