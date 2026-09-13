//! Autocomplete item component - a single item in the suggestions list

use leptos::*;
use leptos::prelude::{ElementChild, ClassAttribute, OnAttribute, GlobalAttributes};

/// Props for the AutocompleteItem component
#[derive(Debug, Clone)]
pub struct AutocompleteItemProps<T: Clone + std::fmt::Display + 'static> {
    /// The value of this item
    pub value: T,
    /// Callback when this item is clicked
    pub on_click: Option<Rc<dyn Fn(T)>>,
}

impl<T: Clone + std::fmt::Display> Default for AutocompleteItemProps<T> {
    fn default() -> Self {
        Self {
            value: panic!("AutocompleteItem requires a value"),
            on_click: None,
        }
    }
}

/// A single item in the autocomplete suggestions list
pub fn AutocompleteItem<T: Clone + std::fmt::Display + 'static>(props: AutocompleteItemProps<T>) -> impl IntoView {
    let AutocompleteItemProps { value, on_click } = props;

    let handle_click = move |_| {
        if let Some(callback) = on_click {
            callback(value.clone());
        }
    };

    view! {
        <div
            class="autocomplete-item"
            on:click=handle_click
            role="option"
            tabindex="0"
        >
            {value.to_string()}
        </div>
    }
}

/// Convenience function for creating an AutocompleteItem
pub fn autocomplete_item<T: Clone + std::fmt::Display + 'static>(
    value: T,
    on_click: Option<Rc<dyn Fn(T)>>,
) -> AutocompleteItemProps<T> {
    AutocompleteItemProps { value, on_click }
}