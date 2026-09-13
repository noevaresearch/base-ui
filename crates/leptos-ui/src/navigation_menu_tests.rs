//! Navigation Menu tests

use leptos::*;
use leptos_ui::navigation_menu::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_menu_creation() {
        // Test basic navigation menu creation
        let _ = view! {
            <NavigationMenuRoot
                value=None
                default_value=None
                on_value_change=|_| {}
                delay=200
                close_delay=100
                orientation=Orientation::Horizontal
                nested=false
            >
                <NavigationMenuItem>
                    <NavigationMenuTrigger
                        active=false
                        disabled=false
                        on_activate=|_| {}
                        on_deactivate=|_| {}
                    >
                        "Item 1"
                    </NavigationMenuTrigger>
                    <NavigationMenuContent
                        value="item1".to_string()
                        active=false
                        keep_mounted=false
                    >
                        "Content 1"
                    </NavigationMenuContent>
                </NavigationMenuItem>
            </NavigationMenuRoot>
        };
    }

    #[test]
    fn test_navigation_menu_link() {
        // Test navigation menu link
        let _ = view! {
            <NavigationMenuLink
                href="#".to_string()
                active=false
                on_click=|_| {}
            >
                "Link"
            </NavigationMenuLink>
        };
    }

    #[test]
    fn test_navigation_menu_list() {
        // Test navigation menu list
        let _ = view! {
            <NavigationMenuList
                orientation=Orientation::Horizontal
                nested=false
            >
                <NavigationMenuItem>
                    <NavigationMenuTrigger
                        active=false
                        disabled=false
                        on_activate=|_| {}
                        on_deactivate=|_| {}
                    >
                        "Item"
                    </NavigationMenuTrigger>
                </NavigationMenuItem>
            </NavigationMenuList>
        };
    }
}