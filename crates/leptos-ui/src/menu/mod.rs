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
pub mod arrow;
pub mod backdrop;
pub mod checkbox_item;
pub mod checkbox_item_indicator;
pub mod group;
pub mod group_label;
pub mod item;
pub mod link_item;
pub mod portal;
pub mod positioner;
pub mod popup;
pub mod radio_group;
pub mod radio_item;
pub mod radio_item_indicator;
pub mod root;
pub mod separator;
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
pub use arrow::Arrow;
pub use arrow::{
    MenuArrowProps, MenuArrowResolved, MenuArrowState, menu_arrow_attributes, menu_arrow_state,
    menu_arrow_style, resolve_menu_arrow,
};
/// The pre-rewrite flat name for the arrow part (the `MenuItem` alias precedent).
pub use arrow::Arrow as MenuArrow;
pub use backdrop::{Backdrop, MenuBackdropProps, menu_backdrop_attributes, menu_backdrop_hidden};
pub use backdrop::{Backdrop as MenuBackdrop};
/// The checkbox item and its context (`Menu.CheckboxItem`, upstream's `MenuCheckboxItem`).
pub use checkbox_item::{
    CheckboxItem, MENU_CHECKBOX_ITEM_DISABLED_ATTRIBUTE, MENU_CHECKBOX_ITEM_HIGHLIGHTED_ATTRIBUTE,
    MENU_CHECKBOX_ITEM_ROLE, MenuCheckboxItemClick, MenuCheckboxItemContextValue,
    MenuCheckboxItemResolved, SharedMenuCheckboxItemContext, menu_checkbox_item_click,
    menu_checkbox_item_context, resolve_menu_checkbox_item, use_menu_checkbox_item_context,
};
/// The pre-rewrite flat name for the checkbox item part (the `MenuItem` alias precedent).
pub use checkbox_item::CheckboxItem as MenuCheckboxItem;
pub use group::{
    Group, MENU_GROUP_ROLE, MenuGroupContextValue, SharedMenuGroupContext, menu_group_aria_labelledby,
    menu_group_context, menu_group_context_optional, set_group_label_id, use_menu_group_context,
};
pub use group::{Group as MenuGroup};
/// The checkbox item indicator (`Menu.CheckboxItemIndicator`).
pub use checkbox_item_indicator::{
    CheckboxItemIndicator, MENU_CHECKBOX_ITEM_INDICATOR_ARIA_HIDDEN,
    MenuCheckboxItemIndicatorState, menu_checkbox_item_indicator_attributes,
    menu_checkbox_item_indicator_should_render, menu_checkbox_item_indicator_state_map,
};
/// The pre-rewrite flat name for the checkbox item indicator part.
pub use checkbox_item_indicator::CheckboxItemIndicator as MenuCheckboxItemIndicator;
/// The radio group (`Menu.RadioGroup`).
pub use radio_group::{
    MENU_RADIO_GROUP_ROLE, MenuRadioGroupContextValue, MenuRadioGroupResolved, MenuRadioGroupState,
    MenuRadioValue, OnMenuRadioValueChange, SharedMenuRadioGroupContext, menu_radio_group_aria_disabled,
    menu_radio_group_aria_labelledby, menu_radio_group_commits, menu_radio_group_context,
    resolve_menu_radio_group, use_menu_radio_group_context,
};
/// The pre-rewrite flat name for the radio group part.
pub use radio_group::RadioGroup as MenuRadioGroup;
/// The radio item (`Menu.RadioItem`).
pub use radio_item::{
    MENU_RADIO_ITEM_ROLE, MenuRadioItemChangeEventDetails, MenuRadioItemContextValue,
    MenuRadioItemResolved, SharedMenuRadioItemContext, menu_radio_item_checked,
    menu_radio_item_context, menu_radio_item_disabled, resolve_menu_radio_item,
    use_menu_radio_item_context,
};
/// The pre-rewrite flat name for the radio item part.
pub use radio_item::RadioItem as MenuRadioItem;
/// The radio item indicator (`Menu.RadioItemIndicator`).
pub use radio_item_indicator::{
    MENU_RADIO_ITEM_INDICATOR_ARIA_HIDDEN, MenuRadioItemIndicatorState, RadioItemIndicator,
    menu_radio_item_indicator_attributes, menu_radio_item_indicator_should_render,
    menu_radio_item_indicator_state_map,
};
/// The pre-rewrite flat name for the radio item indicator part.
pub use radio_item_indicator::RadioItemIndicator as MenuRadioItemIndicator;
pub use group_label::{
    GroupLabel, MenuGroupLabelAttrs, menu_group_label_aria_hidden, menu_group_label_attrs,
};
pub use group_label::{GroupLabel as MenuGroupLabel};
pub use item::Item;
/// The pre-rewrite flat names, kept as aliases so existing call sites keep compiling.
pub use item::Item as MenuItem;
/// The link item part (`Menu.LinkItem`, upstream's `MenuLinkItem`) — the real port that replaced
/// the fabricated `link-item.rs` facade (see `link_item.rs`'s module docs).
pub use link_item::{
    LinkItem, MENU_LINK_ITEM_CLOSE_ON_CLICK_DEFAULT, MENU_LINK_ITEM_HIGHLIGHTED_ATTRIBUTE,
    MENU_LINK_ITEM_ROLE, MENU_LINK_ITEM_TAG, MenuLinkItemResolved, menu_link_item_state_map,
    resolve_menu_link_item,
};
/// The pre-rewrite flat name for the link item part (the `MenuItem` alias precedent).
pub use link_item::LinkItem as MenuLinkItem;
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

// The namespaced part names. `specs/docs-content/CONTRACT.md` maps upstream's `Component.Part`
// usage onto `<Component::Part>` markup, and upstream's `index.parts.ts:16-19` spells these three
// parts `Menu.Root` / `Menu.Trigger` / `Menu.Separator`
// (`packages/react/src/menu/index.parts.ts`). Root and Trigger are the real ports in this module
// (`root.rs`'s provider-only Root, `MenuRoot.tsx:636-648`; `trigger.rs`'s button); like every
// other part in this crate their canonical name is the part name itself, so the definitions carry
// it and the pre-rewrite `Menu*` spellings stay as flat aliases (the `MenuItem` precedent). The
// third is the separator unit's own part (`separator.rs`).
pub use root::Root;
pub use trigger::Trigger;
pub use separator::Separator;
/// The pre-rewrite flat names for the root and trigger parts.
pub use root::Root as MenuRootComponent;
pub use trigger::Trigger as MenuTrigger;

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
