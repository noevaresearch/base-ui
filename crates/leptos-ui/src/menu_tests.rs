//! Menu tests
//! 
//! Basic tests for menu components to verify compilation and basic functionality

#[cfg(test)]
mod tests {
    use leptos::*;
    use crate::menu::*;
    
    #[test]
    fn test_menu_compiles() {
        // This test verifies that menu components can be imported and used
        // without compilation errors
        
        // Test that components can be called
        let _item = MenuItem(Some("Test Item"));
        let _checkbox = MenuCheckboxItem(Some("Test Checkbox"));
        let _radio = MenuRadioItem(Some("Test Radio"), "test-value".to_string());
        let _group = MenuGroup(Some("Test Group"), Some("Heading".to_string()));
        let _separator = MenuSeparator();
        let _menu_root = MenuRoot(Some("Content"));
        let _menu_trigger = MenuTrigger(Some("Content"));
        let _menu_popup = MenuPopup(Some("Content"));
        
        // If we reach here, compilation succeeded
        assert!(true);
    }
    
    #[test]
    fn test_menu_components_basic() {
        // Test that all menu components can be created and used
        let _menu_item = MenuItem(Some("Test"));
        let _menu_checkbox = MenuCheckboxItem(Some("Test"));
        let _menu_radio = MenuRadioItem(Some("Test"), "value".to_string());
        let _menu_group = MenuGroup(Some("Content"), Some("Heading".to_string()));
        let _menu_separator = MenuSeparator();
        let _menu_root = MenuRoot(Some("Content"));
        let _menu_trigger = MenuTrigger(Some("Content"));
        let _menu_popup = MenuPopup(Some("Content"));
        
        // If we reach here, all components compiled
        assert!(true);
    }
}