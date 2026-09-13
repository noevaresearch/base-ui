//! Autocomplete root component - the main container for autocomplete functionality
 
use leptos::*;
use leptos::prelude::{ElementChild, IntoView, ClassAttribute, Callback, signal, OnAttribute, Get, Set};

use super::value::AutocompleteValue;

/// Display modes for autocomplete
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AutocompleteMode {
    #[default]
    List,
    Both,
    Inline,
    None,
}

/// Auto-highlight behavior
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AutoHighlight {
    #[default]
    First,
    None,
}

/// Props for the Autocomplete component
#[derive(Debug, Clone)]
pub struct AutocompleteRootProps<T: Clone + 'static> {
    /// List of items to suggest
    pub items: Vec<T>,
    /// Current value of the input
    pub value: Option<String>,
    /// Default value when uncontrolled
    pub default_value: Option<String>,
    /// Callback when value changes
    pub on_value_change: Option<Rc<dyn Fn(String)>>,
    /// Display mode
    pub mode: AutocompleteMode,
    /// Auto highlight
    pub auto_highlight: AutoHighlight,
    /// Keep highlight
    pub keep_highlight: bool,
    /// Filter function
    pub filter: Option<Callback<String, Vec<T>>>,
    /// Locale
    pub locale: Option<String>,
    /// Item to string
    pub item_to_string_value: Option<Callback<T, String>>,
    /// Open on input click
    pub open_on_input_click: bool,
    /// Default open
    pub default_open: bool,
    /// Open
    pub open: bool,
    /// Inline
    pub inline: bool,
    /// Name
    pub name: Option<String>,
    /// Form
    pub form: Option<String>,
    /// Required
    pub required: bool,
    /// Disabled
    pub disabled: bool,
    /// Readonly
    pub readonly: bool,
    /// Submit on item click
    pub submit_on_item_click: bool,
}

impl<T: Clone + 'static> Default for AutocompleteRootProps<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            value: None,
            default_value: None,
            on_value_change: None,
            mode: AutocompleteMode::default(),
            auto_highlight: AutoHighlight::default(),
            keep_highlight: false,
            filter: None,
            locale: None,
            item_to_string_value: None,
            open_on_input_click: false,
            default_open: false,
            open: false,
            inline: false,
            name: None,
            form: None,
            required: false,
            disabled: false,
            readonly: false,
            submit_on_item_click: false,
        }
    }
}

/// The root component for autocomplete functionality
pub fn AutocompleteRoot<T: Clone>(
    props: AutocompleteRootProps<T>,
) -> impl IntoView {
    let AutocompleteRootProps {
        items,
        value,
        default_value,
        on_value_change,
        mode,
        auto_highlight,
        keep_highlight,
        filter,
        locale,
        item_to_string_value,
        open_on_input_click,
        default_open,
        open,
        inline,
        name,
        form,
        required,
        disabled,
        readonly,
        submit_on_item_click,
    } = props;

    let (internal_value, set_internal_value) = signal(default_value.unwrap_or_default());
    let (inline_input_value, set_inline_input_value) = signal(String::new());

    let resolved_input_value = move || {
        value.clone().unwrap_or_else(|| internal_value.get())
    };

    let handle_value_change = move |new_value: String| {
        set_inline_input_value.set(String::new());
        
        if value.is_none() {
            set_internal_value.set(new_value.clone());
        }
        
        if let Some(callback) = on_value_change {
            callback(new_value);
        }
    };

    view! {
        <div class="autocomplete-root">
            <AutocompleteValue value=resolved_input_value() />
        </div>
    }
}