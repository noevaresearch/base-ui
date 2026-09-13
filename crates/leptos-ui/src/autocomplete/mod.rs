//! Autocomplete component - provides search and selection functionality
//! 
//! Ported from Base UI's React autocomplete component to Leptos
//! 
//! The Autocomplete component is a thin adapter over the Combobox runtime,
//! providing the Base UI API surface on top of the underlying combobox behavior.

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
    AutocompleteChangeEventDetails,
    AutocompleteChangeEventReason,
    AutocompleteHighlightEventDetails,
    AutocompleteHighlightEventReason,
    AutocompleteRootProps,
};