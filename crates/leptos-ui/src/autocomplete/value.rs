//! Autocomplete value component - displays the current value

use leptos::*;
use leptos::prelude::{ElementChild, ClassAttribute};

/// Component to display the current value of an autocomplete input
#[component]
pub fn AutocompleteValue(#[prop(default = String::new())] value: String) -> impl IntoView {
    view! {
        <div class="autocomplete-value">{value}</div>
    }
}