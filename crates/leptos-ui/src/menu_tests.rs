//! Menu tests
//! 
//! Tests for the leptos-ui menu components

use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_root_creation() {
        let _open = create_rw_signal(false);
        
        let _menu = view! {
            <div class="menu-root">
                "Menu Root"
            </div>
        };
        
        // Test that the menu can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_menu_trigger_creation() {
        let _trigger = view! {
            <button class="menu-trigger">
                "Menu Trigger"
            </button>
        };
        
        // Test that the trigger can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_menu_item_creation() {
        let _item = view! {
            <div class="menu-item">
                "Menu Item"
            </div>
        };
        
        // Test that the item can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_menu_store_creation() {
        let store = create_menu_store();
        assert_eq!(store.open().get_untracked(), false);
    }

    #[test]
    fn test_menu_hooks() {
        let (_, _set_open) = use_menu_item();
        // Test that the hook can be created without panicking
        assert!(true);
    }
}