//! Autocomplete item component - a single item in the suggestions list
//!
//! Ported from Base UI's React AutocompleteItem component to Leptos

use leptos::prelude::*;
use leptos::*;
use std::rc::Rc;

/// Component for individual items in the suggestions list
#[component]
pub fn AutocompleteItem<T: Clone + Send + Sync + 'static + std::fmt::Display>(
    value: T,
    on_click: Option<Rc<dyn Fn(T)>>,
    disabled: bool,
    class: Option<String>,
) -> impl IntoView {
    let value_str = value.to_string();

    view! {
        <div
            class=class.clone().unwrap_or_else(|| "autocomplete-item".to_string())
            aria-disabled=disabled
            on:click=move |_| {
                if !disabled {
                    if let Some(on_click) = &on_click {
                        on_click(value.clone());
                    }
                }
            }
        >
            {value_str}
        </div>
    }
}
