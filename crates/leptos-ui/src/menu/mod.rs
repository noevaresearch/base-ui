//! Menu components - a comprehensive menu system with support for submenus, checkboxes, radio groups, and more.
//!
//! This is a port of Base UI's Menu component from React to Leptos.
//!
//! ## Structure
//!
//! The menu system consists of several main components:
//!
//! - **Root**: The main menu container that manages state and provides context
//! - **Trigger**: The button that opens the menu
//! - **Popup**: The menu popup that contains all menu items
//! - **Positioner**: Handles positioning and collision detection
//! - **Portal**: Renders the popup in a different part of the DOM
//! - **Items**: Various menu item types (regular, checkbox, radio, link, etc.)
//! - **Groups**: For organizing related items
//! - **Indicators**: Visual indicators for checkbox/radio items
//! - **Arrow/Backdrop**: Visual elements for the popup
//! - **Viewport**: Handles scrolling and viewport constraints
//!
//! ## Dependencies
//!
//! This module depends on several other leptos-ui components:
//!
//! - `leptos_ui_internals`: For floating-ui integration, composite lists, and other utilities
//! - `leptos_ui_utils`: For various hooks and utilities
//!
//! ## Usage Example
//!
pub mod item;
pub mod popup;
pub mod portal;
pub mod positioner;
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_ui::menu::*;
///
/// #[component]
/// fn MenuExample() -> impl IntoView {
///     let open = create_rw_signal(false);
///     
///     view! {
///         <MenuRoot open=open>
///             <MenuTrigger>
///                 "Open Menu"
///             </MenuTrigger>
///             <MenuPopup>
///                 <MenuItem>
///                     "Item 1"
///                 </MenuItem>
///                 <MenuItem>
///                     "Item 2"
///                 </MenuItem>
///             </MenuPopup>
///         </MenuRoot>
///     }
/// }
/// ```
pub mod root;
pub mod store;
pub mod trigger;
pub mod utils;

// Re-export main components for easy access.
//
// `pub use item::Item;` is what makes the namespaced part surface resolvable: upstream's
// docs teach `<Menu.Item />`, so the port's spelling is `<Menu::Item>`, with the part
// name as the final path segment (`crates/leptos-ui/tests/ns_component_path.rs`). Only
// the parts whose bodies are real ports are aliased here — `Menu.Popup`, `Menu.Portal`
// and `Menu.Positioner` still carry placeholder bodies, and exposing them as ergonomic
// aliases over a placeholder is the same defect in the other direction (the `library:
// namespaced part surface (menus batch)` note says so in as many words).
pub use item::Item;
/// The pre-rewrite flat name, kept as an alias so existing call sites keep compiling.
pub use item::Item as MenuItem;
pub use popup::*;
pub use root::*;
pub use trigger::*;

// Re-export types
pub use store::*;
pub use utils::*;
