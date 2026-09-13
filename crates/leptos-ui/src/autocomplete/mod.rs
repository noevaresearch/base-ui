//! Autocomplete component - provides search and selection functionality
//! 
//! Ported from Base UI's React autocomplete component to Leptos
//! 
//! This is a basic implementation that provides the API surface for autocomplete functionality.

pub mod root;
pub mod value;
pub mod item;

pub use root::*;
pub use value::*;
pub use item::*;

// Re-export types from the main component
pub use root::{
    AutocompleteMode,
    AutoHighlight,
    AutocompleteRootProps,
};