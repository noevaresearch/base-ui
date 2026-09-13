//! Menu types and constants
//!
//! Ported from packages/react/src/menu/index.ts and related files

use leptos::*;
use leptos::prelude::*;
use leptos_ui_utils::*;
use leptos_ui_internals::*;

/// Menu component public API
pub mod menu {
    use super::*;

    /// Menu root component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuRootProps {
        pub open: MaybeProp<bool>,
        pub on_open_change: Callback<bool>,
        pub modal: MaybeProp<bool>,
        pub root_id: MaybeProp<String>,
        pub children: Children,
    }

    /// Menu trigger component props  
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuTriggerProps {
        pub as_child: MaybeProp<bool>,
        pub children: Children,
    }

    /// Menu popup component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuPopupProps {
        pub as_child: MaybeProp<bool>,
        pub children: Children,
    }

    /// Menu item component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuItemProps {
        pub as_child: MaybeProp<bool>,
        pub disabled: MaybeProp<bool>,
        pub children: Children,
    }

    /// Menu checkbox item component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuCheckboxItemProps {
        pub as_child: MaybeProp<bool>,
        pub disabled: MaybeProp<bool>,
        pub checked: MaybeProp<bool>,
        pub on_checked_change: Callback<bool>,
        pub children: Children,
    }

    /// Menu radio item component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuRadioItemProps {
        pub as_child: MaybeProp<bool>,
        pub disabled: MaybeProp<bool>,
        pub value: String,
        pub children: Children,
    }

    /// Menu submenu trigger component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuSubmenuTriggerProps {
        pub as_child: MaybeProp<bool>,
        pub disabled: MaybeProp<bool>,
        pub children: Children,
    }

    /// Menu group component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuGroupProps {
        pub children: Children,
    }

    /// Menu radio group component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuRadioGroupProps {
        pub value: MaybeProp<String>,
        pub on_value_change: Callback<String>,
        pub children: Children,
    }

    /// Menu separator component props (re-exported from separator library)
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuSeparatorProps {
        pub as_child: MaybeProp<bool>,
    }

    /// Menu portal component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuPortalProps {
        pub children: Children,
    }

    /// Menu arrow component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuArrowProps {
        pub as_child: MaybeProp<bool>,
    }

    /// Menu backdrop component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuBackdropProps {
        pub as_child: MaybeProp<bool>,
    }

    /// Menu viewport component props
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuViewportProps {
        pub as_child: MaybeProp<bool>,
    }
}

/// Menu constants
pub mod constants {
    /// Typeahead reset timeout in milliseconds
    pub const TYPEAHEAD_RESET_MS: u32 = 500;

    /// Patient click threshold in milliseconds  
    pub const PATIENT_CLICK_THRESHOLD: u32 = 200;

    /// Dropdown collision avoidance preset
    pub const DROPDOWN_COLLISION_AVOIDANCE: &str = "drop";

    /// Popup collision avoidance preset
    pub const POPUP_COLLISION_AVOIDANCE: &str = "flip";
}

/// Menu state and context types
pub mod state {
    use super::*;

    /// Menu parent type classification
    #[derive(Clone, Debug, PartialEq)]
    pub enum MenuParent {
        Root,
        Submenu,
        Menubar,
        ContextMenu,
        Toolbar,
    }

    /// Menu instant type classification
    #[derive(Clone, Debug, PartialEq)]
    pub enum MenuInstantType {
        Click,
        Dismiss,
        Group,
    }

    /// Menu open reason
    #[derive(Clone, Debug, PartialEq)]
    pub enum MenuOpenReason {
        Pointer,
        Keyboard,
        Focus,
        Hover,
        ImminentUnmount,
        Close,
    }

    /// Menu close reason  
    #[derive(Clone, Debug, PartialEq)]
    pub enum MenuCloseReason {
        Pointer,
        Keyboard,
        Focus,
        Hover,
        ImminentUnmount,
        Close,
    }
}