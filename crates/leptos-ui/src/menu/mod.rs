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
//! - **Portal**: Renders the popup under a different part of the DOM
//! - **Positioner**: Positions the popup against the anchor
//! - **Popup**: The menu popup that contains all menu items
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
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_ui::menu::*;
//!
//! #[component]
//! fn MenuExample() -> impl IntoView {
//!     let open = create_rw_signal(false);
//!
//!     view! {
//!         <MenuRoot open=open>
//!             <MenuTrigger>
//!                 "Open Menu"
//!             </MenuTrigger>
//!             <MenuPortal>
//!                 <MenuPositioner>
//!                     <MenuPopup>
//!                         <MenuItem>
//!                             "Item 1"
//!                         </MenuItem>
//!                         <MenuItem>
//!                             "Item 2"
//!                         </MenuItem>
//!                     </MenuPopup>
//!                 </MenuPositioner>
//!             </MenuPortal>
//!         </MenuRoot>
//!     }
//! }
//! ```
pub mod item;
pub mod portal;
pub mod positioner;
pub mod popup;
pub mod root;
pub mod store;
pub mod trigger;
pub mod utils;

// Re-export main components for easy access.
//
// The namespaced part surface: upstream's docs teach `<Menu.Item />`,
// `<Menu.Portal>`, `<Menu.Positioner>` and `<Menu.Popup />`, so the port's spelling is
// `<Menu::Item>`, `<Menu::Portal>`, `<Menu::Positioner>`, `<Menu::Popup>` — a module path
// whose final segment is the part name (`crates/leptos-ui/tests/ns_component_path.rs`).
//
// The exports are explicit rather than globbed so that each part's helpers stay reachable
// at their own module path (`menu::positioner::side_attr`) without hoisting a generic
// name into the crate-wide `menu::*` glob the other units share.
pub use item::Item;
/// The pre-rewrite flat names, kept as aliases so existing call sites keep compiling.
pub use item::Item as MenuItem;
/// The pre-rewrite flat name for the portal part (the `MenuItem` alias precedent).
pub use portal::Portal as MenuPortal;
/// The pre-rewrite flat name for the positioner part.
pub use positioner::Positioner as MenuPositioner;
/// The pre-rewrite flat name for the popup part.
pub use popup::Popup as MenuPopup;
pub use portal::{MenuPortalContextValue, Portal, menu_portal_owner_role};
pub use positioner::{
    BackdropCutout, MenuPositionerContext, MenuPositionerResolution, MenubarOrientation,
    PositionMethod, Positioner, SharedMenuPositionerContext, resolve_menu_positioner,
};
pub use popup::{
    Popup, menu_popup_hover_interaction_enabled, menu_popup_initial_focus,
    menu_popup_is_context_menu, menu_popup_return_focus,
    menu_popup_should_stop_composite_key, menu_popup_state,
};
pub use root::*;
pub use trigger::*;

// Re-export types
pub use store::*;
pub use utils::*;

// The context reads and the element-attribute producers are re-exported under their own
// names so a caller (or the Arrow/Backdrop checkpoints) reaches them without a path detour.
pub use portal::{menu_portal_context, menu_portal_should_render, use_menu_portal_context};
pub use positioner::{
    menu_positioner_backdrop_cutout, menu_positioner_context, menu_positioner_popup_modal,
    menu_positioner_should_render_backdrop, use_menu_positioner_context,
};
pub use popup::{menu_popup_attributes, menu_popup_root_owner_id};
