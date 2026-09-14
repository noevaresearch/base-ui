//! Menu store - central state management for the menu system
//! 
//! This is a port of Base UI's MenuStore from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// Menu store state
#[derive(Clone, Debug)]
pub struct MenuStore {
    /// Whether the menu is open
    pub open: RwSignal<bool>,
    /// The active trigger element
    pub active_trigger: Option<web_sys::HtmlElement>,
    /// The current highlighted item
    pub highlighted_item: Option<String>,
    /// Menu orientation
    pub orientation: MenuOrientation,
    /// Whether the menu is modal (blocks outside clicks)
    pub modal: bool,
    /// Whether to highlight items on hover
    pub highlight_item_on_hover: bool,
    /// Root ID for nested menus
    pub root_id: Option<String>,
    /// Parent menu information for nested menus
    pub parent: Option<MenuParent>,
    /// Floating tree root context
    pub floating_tree_root: Option<Rc<dyn Any>>,
    /// Keyboard event relay for menubar
    pub keyboard_event_relay: Option<Rc<dyn Any>>,
    /// Allow mouse up trigger flag
    pub allow_mouse_up_trigger: Rc<RefCell<bool>>,
}

/// Menu orientation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuOrientation {
    Horizontal,
    Vertical,
}

/// Menu parent information for nested menus
#[derive(Clone, Debug)]
pub enum MenuParent {
    Root,
    Submenu { id: String },
    Menubar { id: String },
    ContextMenu { id: String },
}

impl MenuStore {
    /// Create a new menu store
    pub fn new() -> Self {
        Self {
            open: create_rw_signal(false),
            active_trigger: None,
            highlighted_item: None,
            orientation: MenuOrientation::Vertical,
            modal: true,
            highlight_item_on_hover: true,
            root_id: None,
            parent: None,
            floating_tree_root: None,
            keyboard_event_relay: None,
            allow_mouse_up_trigger: Rc::new(RefCell::new(false)),
        }
    }

    /// Set the menu open state
    pub fn set_open(&self, open: bool) {
        self.open.set(open);
    }

    /// Set the active trigger element
    pub fn set_active_trigger(&mut self, element: Option<web_sys::HtmlElement>) {
        self.active_trigger = element;
    }

    /// Set the highlighted item
    pub fn set_highlighted_item(&mut self, item_id: Option<String>) {
        self.highlighted_item = item_id;
    }

    /// Set the menu orientation
    pub fn set_orientation(&mut self, orientation: MenuOrientation) {
        self.orientation = orientation;
    }

    /// Set whether the menu is modal
    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    /// Set whether to highlight items on hover
    pub fn set_highlight_item_on_hover(&mut self, highlight: bool) {
        self.highlight_item_on_hover = highlight;
    }

    /// Set the root ID for nested menus
    pub fn set_root_id(&mut self, id: Option<String>) {
        self.root_id = id;
    }

    /// Set the parent menu information
    pub fn set_parent(&mut self, parent: Option<MenuParent>) {
        self.parent = parent;
    }

    /// Set the floating tree root context
    pub fn set_floating_tree_root(&mut self, root: Option<Rc<dyn Any>>) {
        self.floating_tree_root = root;
    }

    /// Set the keyboard event relay
    pub fn set_keyboard_event_relay(&mut self, relay: Option<Rc<dyn Any>>) {
        self.keyboard_event_relay = relay;
    }

    /// Allow or disallow mouse up trigger
    pub fn set_allow_mouse_up_trigger(&self, allow: bool) {
        *self.allow_mouse_up_trigger.borrow_mut() = allow;
    }

    /// Check if mouse up trigger is allowed
    pub fn allow_mouse_up_trigger(&self) -> bool {
        *self.allow_mouse_up_trigger.borrow()
    }
}

impl Default for MenuStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provider for menu store
#[derive(Clone, PartialEq)]
pub struct MenuStoreContext(Rc<MenuStore>);

impl MenuStoreContext {
    /// Create a new menu store context
    pub fn new(store: MenuStore) -> Self {
        Self(Rc::new(store))
    }

    /// Get the menu store
    pub fn store(&self) -> Rc<MenuStore> {
        self.0.clone()
    }

    /// Get the open signal
    pub fn open(&self) -> RwSignal<bool> {
        self.0.open.clone()
    }

    /// Get the active trigger
    pub fn active_trigger(&self) -> Option<web_sys::HtmlElement> {
        self.0.active_trigger.clone()
    }

    /// Get the highlighted item
    pub fn highlighted_item(&self) -> Option<String> {
        self.0.highlighted_item.clone()
    }

    /// Get the orientation
    pub fn orientation(&self) -> MenuOrientation {
        self.0.orientation
    }

    /// Get whether the menu is modal
    pub fn modal(&self) -> bool {
        self.0.modal
    }

    /// Get whether to highlight items on hover
    pub fn highlight_item_on_hover(&self) -> bool {
        self.0.highlight_item_on_hover
    }

    /// Get the root ID
    pub fn root_id(&self) -> Option<String> {
        self.0.root_id.clone()
    }

    /// Get the parent menu information
    pub fn parent(&self) -> Option<MenuParent> {
        self.0.parent.clone()
    }

    /// Get the floating tree root context
    pub fn floating_tree_root(&self) -> Option<Rc<dyn Any>> {
        self.0.floating_tree_root.clone()
    }

    /// Get the keyboard event relay
    pub fn keyboard_event_relay(&self) -> Option<Rc<dyn Any>> {
        self.0.keyboard_event_relay.clone()
    }

    /// Check if mouse up trigger is allowed
    pub fn allow_mouse_up_trigger(&self) -> bool {
        self.0.allow_mouse_up_trigger()
    }
}