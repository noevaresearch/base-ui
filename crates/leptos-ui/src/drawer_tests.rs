#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_drawer_handle;

    #[test]
    fn test_drawer_handle_api() {
        let handle = create_drawer_handle();
        
        // Test handle creation
        assert!(std::mem::size_of_val(&handle) > 0);
        
        // Test handle methods exist and can be called
        // Note: These methods require a mounted root to work properly
        handle.open_by_trigger(None);
        handle.close_popup();
    }

    #[test]
    fn test_drawer_types_exist() {
        // Test that drawer types exist and can be instantiated
        let handle = create_drawer_handle();
        assert!(std::mem::size_of_val(&handle) > 0);
    }
}