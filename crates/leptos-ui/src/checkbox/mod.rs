//! Port of the Base UI Checkbox component suite
//! (`specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).

pub mod root;
pub mod indicator;

pub use root::{CheckboxRootProps, checkbox_root_element};
pub use indicator::{CheckboxIndicatorProps, checkbox_indicator_element};

/// Re-export commonly used types and utilities
pub use leptos_ui_internals::types::BaseUIEvent;