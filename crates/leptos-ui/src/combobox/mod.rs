//! Combobox - A searchable select component with keyboard navigation
//!
//! Port of Base UI's Combobox component to Leptos.
//! 
//! # Architecture
//! 
//! The combobox is built around a central store pattern where one store drives
//! all parts. The implementation follows the Base UI behavior spec closely.
//!
//! ## Key Components
//! 
//! - **Root**: The main combobox container with state management
//! - **Input**: The text input field for filtering and displaying values
//! - **Trigger**: The button that opens/closes the popup
//! - **Popup**: The overlay containing the list of options
//! - **List**: Container for the selectable items
//! - **Item**: Individual selectable option
//! - **Value**: Display component for selected values
//! - **Label**: Accessibility label component
//! - **Portal**: For rendering the popup outside normal flow
//! - **Positioner**: Handles positioning the popup relative to the input
//! - **Clear**: Button to clear the current selection
//! - **Chips/Chip**: For displaying selected items in multiple mode
//! - **Group/GroupLabel**: For grouping related items
//! 
//! ## State Model
//! 
//! The combobox manages several independent state dimensions:
//! 
//! - **Open/Closed**: Controls popup visibility
//! - **Value**: The currently selected value(s)  
//! - **Input Value**: The text currently in the input field
//! - **Active Index**: Which item is highlighted (keyboard navigation)
//! - **Selected Index**: Which item is selected
//! 
//! ## Dependencies
//! 
//! This module relies on utilities from:
//! - `leptos_ui_utils`: For hooks like `use_controlled`, `use_stable_callback`, etc.
//! - `leptos_ui_internals`: For floating-ui integration and internal utilities

// Core modules - temporarily disabled for compilation
// pub mod root;
// pub mod store;

// Component modules - temporarily disabled for compilation
// pub mod input;
// pub mod trigger;
// pub mod popup;
// pub mod list;
// pub mod item;
// pub mod value;
// pub mod label;
// pub mod portal;
// pub mod positioner;
// pub mod clear;
// pub mod chips;
// pub mod group;
pub mod simple;
// pub mod store;

// Re-export main types and functions for public API - temporarily disabled
// pub use root::*;
// pub use input::*;
// pub use trigger::*;
// pub use popup::*;
// pub use list::*;
// pub use item::*;
// pub use value::*;
// pub use label::*;
// pub use portal::*;
// pub use positioner::*;
// pub use clear::*;
// pub use chips::*;
// pub use group::*;

// Convenience re-exports for common usage patterns - temporarily disabled
// pub use store::*;

// Test module
#[cfg(test)]
mod tests;