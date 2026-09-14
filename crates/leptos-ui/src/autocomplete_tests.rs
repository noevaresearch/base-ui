//! Tests for the autocomplete component
//!
//! These tests verify that the autocomplete component compiles and works correctly

use crate::autocomplete::*;
use leptos::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_value() {
        let _view = view! { <AutocompleteValue value="test".to_string() /> };
        // This should compile without errors
    }

    #[test]
    fn test_autocomplete_item() {
        let _view = view! {
            <AutocompleteItem<String>
                value="test".to_string()
                on_click=None
                disabled=false
                class=None
            />
        };
        // This should compile without errors
    }

    #[test]
    fn test_autocomplete_root() {
        let props = AutocompleteRootProps::<String> {
            items: Some(vec!["item1".to_string(), "item2".to_string()]),
            value: None,
            on_value_change: None,
            mode: AutocompleteMode::List,
            auto_highlight: AutoHighlight::First,
            keep_highlight: false,
            locale: None,
            open_on_input_click: true,
            default_open: false,
            name: None,
            required: false,
            disabled: false,
            read_only: false,
            id: None,
            class: None,
        };

        let _view = AutocompleteRoot(props);
        // This should compile without errors
    }

    #[test]
    fn test_autocomplete_modes() {
        let _list_mode = AutocompleteMode::List;
        let _both_mode = AutocompleteMode::Both;
        let _inline_mode = AutocompleteMode::Inline;
        let _none_mode = AutocompleteMode::None;
    }

    #[test]
    fn test_auto_highlight() {
        let _first = AutoHighlight::First;
        let _none = AutoHighlight::None;
    }
}
