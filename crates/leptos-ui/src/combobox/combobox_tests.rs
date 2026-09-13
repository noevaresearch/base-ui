//! Combobox tests
//!
//! Basic tests for the combobox implementation

use leptos::*;
use leptos::ev::*;
use leptos::html::*;

#[cfg(test)]
mod combobox_tests {
    use super::*;
    
    #[test]
    fn test_combobox_root_creation() {
        // Test that we can create a basic combobox root
        let combobox = view! {
            <ComboboxRoot>
                <ComboboxInput />
                <ComboboxTrigger />
                <ComboboxPopup>
                    <ComboboxList>
                        <ComboboxItem item="test" />
                    </ComboboxList>
                </ComboboxPopup>
            </ComboboxRoot>
        };
        
        // This test just ensures the code compiles
        assert!(true);
    }
    
    #[test]
    fn test_combobox_list_creation() {
        // Test creating a combobox list with items
        let items = vec!["Option 1", "Option 2", "Option 3"];
        let list = view! {
            <ComboboxList items=items>
            </ComboboxList>
        };
        
        // This test just ensures the code compiles
        assert!(true);
    }
    
    #[test]
    fn test_combobox_item_creation() {
        // Test creating a combobox item
        let item = view! {
            <ComboboxItem item="Test Item" />
        };
        
        // This test just ensures the code compiles
        assert!(true);
    }
    
    #[test]
    fn test_combobox_input_creation() {
        // Test creating a combobox input
        let input = view! {
            <ComboboxInput value="test" />
        };
        
        // This test just ensures the code compiles
        assert!(true);
    }
    
    #[test]
    fn test_combobox_trigger_creation() {
        // Test creating a combobox trigger
        let trigger = view! {
            <ComboboxTrigger />
        };
        
        // This test just ensures the code compiles
        assert!(true);
    }
    
    #[test]
    fn test_combobox_popup_creation() {
        // Test creating a combobox popup
        let popup = view! {
            <ComboboxPopup open=true>
                <ComboboxList>
                    <ComboboxItem item="test" />
                </ComboboxList>
            </ComboboxPopup>
        };
        
        // This test just ensures the code compiles
        assert!(true);
    }
    
    #[test]
    fn test_combobox_store_basic_functionality() {
        // Test basic store functionality
        let mut store = crate::combobox::store::ComboboxStore::new::<String>();
        
        // Test setting values
        store.update(|s| {
            s.open = true;
            s.value = Some("test".to_string());
            s.input_value = "test".to_string();
        });
        
        // Test getting values
        assert_eq!(store.open, true);
        assert_eq!(store.value, Some("test".to_string()));
        assert_eq!(store.input_value, "test".to_string());
        
        // Test filtering
        let test_items = vec!["Apple", "Banana", "Cherry", "Date"];
        store.items = test_items.clone();
        
        // Apply filtering
        store.apply_filtering("a");
        
        // Should contain "Apple" and "Banana"
        assert_eq!(store.filtered_items.len(), 2);
        assert!(store.filtered_items.contains(&"Apple".to_string()));
        assert!(store.filtered_items.contains(&"Banana".to_string()));
    }
    
    #[test]
    fn test_combobox_navigation() {
        // Test navigation functionality
        let mut store = crate::combobox::store::ComboboxStore::new::<String>();
        
        // Set up test items
        let test_items = vec!["Item 1", "Item 2", "Item 3"];
        store.items = test_items.clone();
        store.apply_filtering("");
        
        // Test moving to next item
        store.move_next();
        assert_eq!(store.active_index, Some(0));
        
        store.move_next();
        assert_eq!(store.active_index, Some(1));
        
        // Test moving to previous item
        store.move_previous();
        assert_eq!(store.active_index, Some(0));
        
        // Test wrapping at ends
        store.loop_focus = true;
        store.move_next(); // Move to end
        assert_eq!(store.active_index, Some(2));
        
        store.move_next(); // Wrap to beginning
        assert_eq!(store.active_index, Some(0));
    }
    
    #[test]
    fn test_combobox_selection() {
        // Test selection functionality
        let mut store = crate::combobox::store::ComboboxStore::new::<String>();
        
        // Set up test items
        let test_items = vec!["Item 1", "Item 2", "Item 3"];
        store.items = test_items.clone();
        store.apply_filtering("");
        
        // Select the second item
        store.active_index = Some(1);
        store.select_active_item();
        
        assert_eq!(store.selected_index, Some(1));
        assert_eq!(store.value, Some("Item 2".to_string()));
    }
    
    #[test]
    fn test_combobox_clear_functions() {
        // Test clear functionality
        let mut store = crate::combobox::store::ComboboxStore::new::<String>();
        
        // Set up some values
        store.update(|s| {
            s.value = Some("test".to_string());
            s.input_value = "test".to_string();
            s.active_index = Some(0);
            s.selected_index = Some(0);
        });
        
        // Test clearing selection
        store.clear_selection();
        assert_eq!(store.value, None);
        assert_eq!(store.selected_index, None);
        assert_eq!(store.input_value, String::new());
        
        // Test clearing query
        store.update(|s| {
            s.input_value = "test".to_string();
            s.active_index = Some(0);
        });
        
        store.clear_query();
        assert_eq!(store.active_index, None);
        assert_eq!(store.input_value, String::new());
    }
}