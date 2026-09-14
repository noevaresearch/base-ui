//! Simple Combobox Implementation
//!
//! A minimal working combobox to get started with

use leptos::prelude::*;

/// Simple combobox component
pub fn Combobox<T>(_props: ComboboxProps<T>) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    view! {
        <div class="leptos-combobox">
            <input
                type="text"
                placeholder="Select an option..."
                class="leptos-combobox-input"
            />
            <button class="leptos-combobox-trigger">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </button>
            <div class="leptos-combobox-popup">
                <div class="leptos-combobox-list">
                    <div class="leptos-combobox-item">
                        "Option 1"
                    </div>
                    <div class="leptos-combobox-item">
                        "Option 2"
                    </div>
                    <div class="leptos-combobox-item">
                        "Option 3"
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Simple combobox props
pub struct ComboboxProps<T: 'static = String> {
    /// Placeholder text
    pub placeholder: String,
    /// Available options
    pub options: Vec<T>,
    /// Selected value
    pub value: Option<T>,
    /// Callback when value changes
    pub on_change: Option<Callback<T, ()>>,
}

impl<T> Default for ComboboxProps<T> {
    fn default() -> Self {
        Self {
            placeholder: "Select an option...".to_string(),
            options: Vec::new(),
            value: None,
            on_change: None,
        }
    }
}
