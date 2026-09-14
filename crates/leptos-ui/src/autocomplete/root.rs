//! Autocomplete component - provides search and selection functionality
//!
//! Ported from Base UI's React autocomplete component to Leptos
//!
//! This implementation provides the basic API surface and behavior specified
//! in the behavior spec.

use leptos::ev::{FocusEvent, KeyboardEvent};
use leptos::prelude::*;
use leptos::*;

/// The display mode for the autocomplete
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutocompleteMode {
    #[default]
    List,
    Both,
    Inline,
    None,
}

/// Auto highlight behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoHighlight {
    First,
    Always,
    None,
}

impl Default for AutoHighlight {
    fn default() -> Self {
        Self::None
    }
}

/// Props for the Autocomplete component
#[derive(Clone, Default)]
pub struct AutocompleteRootProps<T: Clone + Send + Sync + 'static> {
    pub items: Option<Vec<T>>,
    pub value: Option<T>,
    pub on_value_change: Option<fn(String)>,
    pub mode: AutocompleteMode,
    pub auto_highlight: AutoHighlight,
    pub keep_highlight: bool,
    pub locale: Option<String>,
    pub open_on_input_click: bool,
    pub default_open: bool,
    pub name: Option<String>,
    pub required: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub id: Option<String>,
    pub class: Option<String>,
}

/// The main Autocomplete component
pub fn AutocompleteRoot<
    T: Clone + Send + Sync + 'static + std::fmt::Display + std::default::Default,
>(
    props: AutocompleteRootProps<T>,
) -> impl IntoView {
    // State management
    let items = RwSignal::new(props.items.unwrap_or_default());
    let value = RwSignal::new(props.value.map(|v| v.to_string()));
    let internal_value = RwSignal::new(String::new());
    let open = RwSignal::new(props.default_open);
    let active_index = RwSignal::new(None::<usize>);

    // Handle value changes
    let on_value_change_callback = props.on_value_change.unwrap_or(|_value: String| {});
    let handle_value_change = move |new_value: String| {
        value.set(Some(new_value.clone()));
        internal_value.set(new_value.clone());
        on_value_change_callback(new_value);
    };

    // Handle keyboard events
    let on_key_down = move |ev: KeyboardEvent| {
        if props.disabled || props.read_only {
            return;
        }

        let key = ev.key();

        match key.as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                if !open.get() {
                    open.set(true);
                }
                let current = active_index.get().unwrap_or(0);
                let next = (current + 1) % items.get().len();
                active_index.set(Some(next));
            }
            "ArrowUp" => {
                ev.prevent_default();
                if !open.get() {
                    open.set(true);
                }
                let current = active_index.get().unwrap_or(0);
                let prev = if current == 0 {
                    items.get().len() - 1
                } else {
                    current - 1
                };
                active_index.set(Some(prev));
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(idx) = active_index.get() {
                    if let Some(item) = items.get().get(idx) {
                        handle_value_change(item.to_string());
                    }
                }
                open.set(false);
            }
            "Escape" => {
                ev.prevent_default();
                open.set(false);
            }
            "Tab" => {
                ev.prevent_default();
                open.set(false);
            }
            _ => {}
        }
    };

    // Handle focus events
    let on_focus = move |_ev: FocusEvent| {
        if props.disabled || props.read_only {
            return;
        }

        if props.open_on_input_click {
            open.set(true);
        }
    };

    // Handle blur events
    let on_blur = move |_ev: FocusEvent| {
        if !props.keep_highlight {
            open.set(false);
        }
        active_index.set(None);
    };

    // Accessibility attributes
    let aria_autocomplete = match props.mode {
        AutocompleteMode::None => "none",
        _ => "list",
    };

    view! {
        <div
            class=props.class.clone().unwrap_or_else(|| "autocomplete-root".to_string())
            id=props.id.clone()
        >
            <input
                type="text"
                prop:value=move || internal_value.get()
                disabled=props.disabled
                readonly=props.read_only
                required=props.required
                name=props.name.clone()
                aria-autocomplete=aria_autocomplete
                aria-expanded=open.get()
                class="autocomplete-input"
            />

            // Show suggestions dropdown when open and not in 'none' mode
            {move || {
                if open.get() && props.mode != AutocompleteMode::None && !items.get().is_empty() {
                    view! {
                        <div class="autocomplete-dropdown" role="listbox">
                            {items.get().iter().enumerate().map(|(idx, item)| {
                                let item_clone = item.clone();
                                let item_str = item_clone.to_string();
                                let is_active = active_index.get() == Some(idx);

                                view! {
                                    <div
                                        class=if is_active {
                                            "autocomplete-item active"
                                        } else {
                                            "autocomplete-item"
                                        }
                                        role="option"
                                        aria-selected=is_active
                                        on:click=move |_| {
                                            if !props.disabled && !props.read_only {
                                                let value = item_str.clone();
                                                handle_value_change(value);
                                                open.set(false);
                                            }
                                        }
                                    >
                                        {item_str.clone()}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}
