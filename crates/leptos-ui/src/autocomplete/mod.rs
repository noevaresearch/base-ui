//! Autocomplete component - a text input suggesting options as the user types
//! 
//! Ported from Base UI's React autocomplete component to Leptos

pub mod root;
pub mod value;
pub mod item;

// Re-export the main components
pub use root::*;
pub use value::*;
pub use item::*;