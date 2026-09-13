//! Autocomplete value component - displays the current value
//! 
//! Ported from Base UI's React AutocompleteValue component to Leptos

use leptos::*;
use leptos::prelude::*;

/// Component to display the current value
#[component]
pub fn AutocompleteValue(value: String) -> impl IntoView {
    view! {
        <span class="autocomplete-value">{value}</span>
    }
}