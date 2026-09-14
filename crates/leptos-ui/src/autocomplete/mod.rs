//! Autocomplete component - provides search and selection functionality
//! 
//! Ported from Base UI's React autocomplete component to Leptos
//! 
//! This is a basic implementation that provides the API surface for autocomplete functionality.

pub mod root;
pub mod value;
pub mod item;

// Re-export components without causing ambiguity
pub use root::AutocompleteRoot;
pub use value::AutocompleteValue;
pub use root::AutocompleteRootProps;
pub use root::AutocompleteMode;
pub use root::AutoHighlight;
pub use item::AutocompleteItem;

// Additional type definitions for test compatibility
pub type AutocompleteChangeEventDetails = String;
pub type AutocompleteChangeEventReason = String;
pub type AutocompleteHighlightEventDetails = String;
pub type AutocompleteHighlightEventReason = String;