//! Autocomplete component - provides search and selection functionality
//!
//! Ported from Base UI's React autocomplete component to Leptos
//!
//! This is a basic implementation that provides the API surface for autocomplete functionality.

pub mod item;
pub mod root;
pub mod value;

// Re-export components without causing ambiguity
pub use item::AutocompleteItem;
pub use root::AutoHighlight;
pub use root::AutocompleteMode;
pub use root::AutocompleteRoot;
pub use root::AutocompleteRootProps;
pub use value::AutocompleteValue;

// Additional type definitions for test compatibility
pub type AutocompleteChangeEventDetails = String;
pub type AutocompleteChangeEventReason = String;
pub type AutocompleteHighlightEventDetails = String;
pub type AutocompleteHighlightEventReason = String;
