//! Port of the Base UI Checkbox component suite
//! (`specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).

pub mod indicator;
pub mod root;

pub use indicator::{CheckboxIndicatorProps, checkbox_indicator_element};
pub use root::{CheckboxRootProps, checkbox_root_element};

/// Re-export commonly used types and utilities
pub use leptos_ui_internals::types::BaseUIEvent;
