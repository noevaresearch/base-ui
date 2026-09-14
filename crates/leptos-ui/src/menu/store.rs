//! Menu store - central state management for the menu system
//! 
//! This is a port of Base UI's MenuStore from React to Leptos.

use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

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
    pub floating_tree_root: Option<Arc<dyn Any + Send + Sync>>,
    /// Keyboard event relay for menubar
    pub keyboard_event_relay: Option<Arc<dyn Any + Send + Sync>>,
    /// Allow mouse up trigger flag
    pub allow_mouse_up_trigger: Arc<RefCell<bool>>,
}

/// Menu orientation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl PartialEq for MenuParent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MenuParent::Root, MenuParent::Root) => true,
            (MenuParent::Submenu { id: id1 }, MenuParent::Submenu { id: id2 }) => id1 == id2,
            (MenuParent::Menubar { id: id1 }, MenuParent::Menubar { id: id2 }) => id1 == id2,
            (MenuParent::ContextMenu { id: id1 }, MenuParent::ContextMenu { id: id2 }) => id1 == id2,
            _ => false,
        }
    }
}

impl PartialEq for MenuStore {
    fn eq(&self, other: &Self) -> bool {
        self.open.get_untracked() == other.open.get_untracked()
            && self.active_trigger == other.active_trigger
            && self.highlighted_item == other.highlighted_item
            && self.orientation == other.orientation
            && self.modal == other.modal
            && self.highlight_item_on_hover == other.highlight_item_on_hover
            && self.root_id == other.root_id
            && self.parent == other.parent
            && self.floating_tree_root.is_none()
            && other.floating_tree_root.is_none()
            && self.keyboard_event_relay.is_none()
            && other.keyboard_event_relay.is_none()
            && *self.allow_mouse_up_trigger.borrow() == *other.allow_mouse_up_trigger.borrow()
    }
}

impl MenuStore {
    /// Create a new menu store
    pub fn new() -> Self {
        Self {
            open: RwSignal::new(false),
            active_trigger: None,
            highlighted_item: None,
            orientation: MenuOrientation::Vertical,
            modal: true,
            highlight_item_on_hover: true,
            root_id: None,
            parent: None,
            floating_tree_root: None,
            keyboard_event_relay: None,
            allow_mouse_up_trigger: Arc::new(RefCell::new(false)),
        }
    }

    /// Get the open state
    pub fn open(&self) -> RwSignal<bool> {
        self.open
    }

    /// Set the open state
    pub fn set_open(&self, open: bool) {
        self.open.set(open);
    }

    /// Get the active trigger
    pub fn active_trigger(&self) -> Option<web_sys::HtmlElement> {
        self.active_trigger.clone()
    }

    /// Set the active trigger
    pub fn set_active_trigger(&mut self, element: Option<web_sys::HtmlElement>) {
        self.active_trigger = element;
    }

    /// Get the highlighted item
    pub fn highlighted_item(&self) -> Option<String> {
        self.highlighted_item.clone()
    }

    /// Set the highlighted item
    pub fn set_highlighted_item(&mut self, item: Option<String>) {
        self.highlighted_item = item;
    }

    /// Get the orientation
    pub fn orientation(&self) -> MenuOrientation {
        self.orientation
    }

    /// Set the orientation
    pub fn set_orientation(&mut self, orientation: MenuOrientation) {
        self.orientation = orientation;
    }

    /// Get the modal state
    pub fn modal(&self) -> bool {
        self.modal
    }

    /// Set the modal state
    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    /// Get the highlight item on hover state
    pub fn highlight_item_on_hover(&self) -> bool {
        self.highlight_item_on_hover
    }

    /// Set the highlight item on hover state
    pub fn set_highlight_item_on_hover(&mut self, highlight: bool) {
        self.highlight_item_on_hover = highlight;
    }

    /// Get the root ID
    pub fn root_id(&self) -> Option<String> {
        self.root_id.clone()
    }

    /// Set the root ID
    pub fn set_root_id(&mut self, id: Option<String>) {
        self.root_id = id;
    }

    /// Get the parent
    pub fn parent(&self) -> Option<MenuParent> {
        self.parent.clone()
    }

    /// Set the parent
    pub fn set_parent(&mut self, parent: Option<MenuParent>) {
        self.parent = parent;
    }

    /// Get the floating tree root
    pub fn floating_tree_root(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.floating_tree_root.clone()
    }

    /// Set the floating tree root
    pub fn set_floating_tree_root(&mut self, root: Option<Arc<dyn Any + Send + Sync>>) {
        self.floating_tree_root = root;
    }

    /// Get the keyboard event relay
    pub fn keyboard_event_relay(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.keyboard_event_relay.clone()
    }

    /// Set the keyboard event relay
    pub fn set_keyboard_event_relay(&mut self, relay: Option<Arc<dyn Any + Send + Sync>>) {
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

// Make MenuStore Send and Sync where possible
unsafe impl Send for MenuStore {}
unsafe impl Sync for MenuStore {}

/// Context provider for menu store
///
/// Wraps the store in `Rc<RefCell<..>>` so all setters work through shared
/// references (interior mutability) — the store is cloned into every event
/// closure, so `&mut self` setters are unusable at the call sites.
#[derive(Clone)]
pub struct MenuStoreContext(Rc<RefCell<MenuStore>>);

// The menu crate only runs on the wasm/browser single-threaded runtime.
unsafe impl Send for MenuStoreContext {}
unsafe impl Sync for MenuStoreContext {}

impl MenuStoreContext {
    /// Create a new menu store context
    pub fn new(store: MenuStore) -> Self {
        Self(Rc::new(RefCell::new(store)))
    }

    /// Get a clone of the underlying menu store value
    pub fn store(&self) -> MenuStore {
        self.0.borrow().clone()
    }

    /// Get the open signal
    pub fn open(&self) -> RwSignal<bool> {
        self.0.borrow().open
    }

    /// Set the open state
    pub fn set_open(&self, open: bool) {
        self.0.borrow().open.set(open);
    }

    /// Get the active trigger
    pub fn active_trigger(&self) -> Option<web_sys::HtmlElement> {
        self.0.borrow().active_trigger()
    }

    /// Set the active trigger
    pub fn set_active_trigger(&self, element: Option<web_sys::HtmlElement>) {
        self.0.borrow_mut().active_trigger = element;
    }

    /// Get the highlighted item
    pub fn highlighted_item(&self) -> Option<String> {
        self.0.borrow().highlighted_item()
    }

    /// Set the highlighted item
    pub fn set_highlighted_item(&self, item_id: Option<String>) {
        self.0.borrow_mut().highlighted_item = item_id;
    }

    /// Get the orientation
    pub fn orientation(&self) -> MenuOrientation {
        self.0.borrow().orientation
    }

    /// Set the orientation
    pub fn set_orientation(&self, orientation: MenuOrientation) {
        self.0.borrow_mut().orientation = orientation;
    }

    /// Get whether the menu is modal
    pub fn modal(&self) -> bool {
        self.0.borrow().modal
    }

    /// Set whether the menu is modal
    pub fn set_modal(&self, modal: bool) {
        self.0.borrow_mut().modal = modal;
    }

    /// Get whether to highlight items on hover
    pub fn highlight_item_on_hover(&self) -> bool {
        self.0.borrow().highlight_item_on_hover
    }

    /// Set whether to highlight items on hover
    pub fn set_highlight_item_on_hover(&self, highlight: bool) {
        self.0.borrow_mut().highlight_item_on_hover = highlight;
    }

    /// Get the root ID
    pub fn root_id(&self) -> Option<String> {
        self.0.borrow().root_id()
    }

    /// Set the root ID for nested menus
    pub fn set_root_id(&self, id: Option<String>) {
        self.0.borrow_mut().root_id = id;
    }

    /// Get the parent menu information
    pub fn parent(&self) -> Option<MenuParent> {
        self.0.borrow().parent()
    }

    /// Set the parent menu information
    pub fn set_parent(&self, parent: Option<MenuParent>) {
        self.0.borrow_mut().parent = parent;
    }

    /// Get the floating tree root context
    pub fn floating_tree_root(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.0.borrow().floating_tree_root()
    }

    /// Set the floating tree root context
    pub fn set_floating_tree_root(&self, root: Option<Arc<dyn Any + Send + Sync>>) {
        self.0.borrow_mut().floating_tree_root = root;
    }

    /// Get the keyboard event relay
    pub fn keyboard_event_relay(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.0.borrow().keyboard_event_relay()
    }

    /// Set the keyboard event relay
    pub fn set_keyboard_event_relay(&self, relay: Option<Arc<dyn Any + Send + Sync>>) {
        self.0.borrow_mut().keyboard_event_relay = relay;
    }

    /// Check if keyboard event relay is equal
    pub fn keyboard_event_relay_eq(&self, other: &Option<Arc<dyn Any + Send + Sync>>) -> bool {
        // For now, just compare as None vs Some
        self.0.borrow().keyboard_event_relay.is_none() && other.is_none()
    }

    /// Allow or disallow mouse up trigger
    pub fn set_allow_mouse_up_trigger(&self, allow: bool) {
        self.0.borrow().set_allow_mouse_up_trigger(allow);
    }

    /// Check if mouse up trigger is allowed
    pub fn allow_mouse_up_trigger(&self) -> bool {
        self.0.borrow().allow_mouse_up_trigger()
    }
}

/// Hook to use the menu store in components
pub fn use_menu_store() -> MenuStoreContext {
    // For now, create a default store
    create_menu_store()
}

/// Create a new menu store context for root components
pub fn create_menu_store() -> MenuStoreContext {
    MenuStoreContext::new(MenuStore::new())
}
