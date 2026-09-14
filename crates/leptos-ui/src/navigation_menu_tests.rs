//! Navigation Menu tests

use leptos::prelude::*;

#[test]
fn navigation_menu_compilation_test() {
    // Test that the navigation menu module can be imported
    let _ = crate::navigation_menu::NavigationMenuRoot::<String>;

    // Test that basic types can be imported
    let _ = crate::navigation_menu::Orientation::Horizontal;

    // This test passes if the above compiles
}
