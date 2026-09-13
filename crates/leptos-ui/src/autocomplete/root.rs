//! Autocomplete component - provides search and selection functionality
//! 
//! Ported from Base UI's React autocomplete component to Leptos
//! 
//! This is a basic implementation that provides the API surface for autocomplete functionality.

use leptos::*;
use leptos::prelude::*;

/// The display mode for the autocomplete
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutocompleteMode {
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

/// Main props for the Autocomplete component
#[derive(Clone)]
pub struct AutocompleteRootProps<T: Clone + Send + Sync + 'static> {
    pub items: Option<Vec<T>>,
    pub filtered_items: Option<Vec<T>>,
    pub value: Option<T>,
    pub default_value: Option<T>,
    pub on_value_change: Option<Box<dyn Fn(T)>>,
    pub on_item_highlighted: Option<Box<dyn Fn(T)>>,
    pub mode: AutocompleteMode,
    pub auto_highlight: AutoHighlight,
    pub keep_highlight: bool,
    pub filter: Option<Box<dyn Fn(String, Vec<T>)>>,
    pub locale: Option<String>,
    pub item_to_string_value: Option<Box<dyn Fn(T) -> String>>,
    pub open_on_input_click: bool,
    pub default_open: bool,
    pub open: Option<bool>,
    pub inline: bool,
    pub name: Option<String>,
    pub form: Option<String>,
    pub required: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub submit_on_item_click: bool,
    pub id: Option<String>,
    pub class: Option<String>,
}

impl<T: Clone + Send + Sync + 'static> Default for AutocompleteRootProps<T> {
    fn default() -> Self {
        Self {
            items: None,
            filtered_items: None,
            value: None,
            default_value: None,
            on_value_change: None,
            on_item_highlighted: None,
            mode: AutocompleteMode::List,
            auto_highlight: AutoHighlight::None,
            keep_highlight: false,
            filter: None,
            locale: None,
            item_to_string_value: None,
            open_on_input_click: false,
            default_open: false,
            open: None,
            inline: false,
            name: None,
            form: None,
            required: false,
            disabled: false,
            readonly: false,
            submit_on_item_click: false,
            id: None,
            class: None,
        }
    }
}

/// The main Autocomplete component - basic implementation
#[component]
pub fn AutocompleteRoot<T: Clone + Send + Sync + 'static + std::fmt::Display>(
    #[prop(default = AutocompleteRootProps::default())] props: AutocompleteRootProps<T>,
) -> impl IntoView {
    let items = RwSignal::new(props.items.unwrap_or_default());
    let value = RwSignal::new(props.value);
    let open = RwSignal::new(props.open.unwrap_or(props.default_open));
    
    // Basic input change handler
    let on_input_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            let input_value = input.value();
            
            // Call value change callback if provided
            if let Some(callback) = &props.on_value_change {
                if let Some(default_val) = props.default_value.clone() {
                    callback(default_val);
                }
            }
        }
    };
    
    // Basic item click handler
    let on_item_click = move |item: T| {
        if let Some(callback) = &props.on_value_change {
            callback(item);
        }
    };
    
    view! {
        <div
            class=props.class.clone().unwrap_or_else(|| "autocomplete-root".to_string())
            id=props.id.clone()
        >
            <input
                type="text"
                prop:value=move || value.get().map(|v| v.to_string()).unwrap_or_default()
                on:input=on_input_change
                prop:disabled=props.disabled
                prop:readonly=props.readonly
                prop:name=props.name.clone()
                prop:form=props.form.clone()
                prop:required=props.required
                aria-autocomplete="list"
                aria-expanded=open.get()
                aria-haspopup="listbox"
            />
            
            // Show suggestions dropdown when open
            {move || {
                if open.get() {
                    view! {
                        <div class="autocomplete-dropdown">
                            {items.get().iter().map(|item| {
                                let item_clone = item.clone();
                                let on_item_click_clone = on_item_click.clone();
                                
                                view! {
                                    <div
                                        class="autocomplete-item"
                                        on:click=move |_| on_item_click_clone(item_clone.clone())
                                    >
                                        {item.to_string()}
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

/// Component to display the current value
#[component]
pub fn AutocompleteValue(value: String) -> impl IntoView {
    view! {
        <span class="autocomplete-value">{value}</span>
    }
}

/// Component for individual items in the suggestions list
#[component]
pub fn AutocompleteItem<T: Clone + Send + Sync + 'static + std::fmt::Display>(
    value: T,
    on_click: Option<Box<dyn Fn(T)>>,
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