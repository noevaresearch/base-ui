//! Menu store and state management
//! 
//! Ported from packages/react/src/menu/store/MenuStore.ts

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use leptos::*;
use leptos_ui_utils::*;
use leptos_ui_internals::*;
use send_wrapper::SendWrapper;

/// Menu store state
#[derive(Clone, Debug)]
pub struct MenuStore {
    /// Open state
    open: RwSignal<bool>,
    
    /// Mounted state  
    mounted: RwSignal<bool>,
    
    /// Transition status
    transition_status: RwSignal<TransitionStatus>,
    
    /// Active trigger element
    active_trigger_element: RwSignal<Option<NodeRef>>,
    
    /// Popup element
    popup_ref: RwSignal<NodeRef>,
    
    /// Menu parent type
    parent: RwSignal<MenuParent>,
    
    /// Internal state
    internal: RwSignal<MenuInternalState>,
}

/// Menu internal state
#[derive(Clone, Debug)]
struct MenuInternalState {
    /// Trigger registry
    trigger_registry: TriggerRegistry,
    
    /// Composite list items
    composite_items: CompositeItems,
    
    /// Open change handlers
    on_open_change: Option<SendWrapper<Callback<bool>>>,
}

/// Trigger registry for managing trigger elements
#[derive(Clone, Debug)]
struct TriggerRegistry {
    triggers: Rc<RefCell<Vec<TriggerEntry>>>,
}

/// Trigger entry  
#[derive(Clone, Debug)]
struct TriggerEntry {
    id: String,
    element: NodeRef,
    active: bool,
}

/// Composite list items for keyboard navigation
#[derive(Clone, Debug)]
struct CompositeItems {
    items: Rc<RefCell<Vec<CompositeItem>>>,
}

/// Composite item
#[derive(Clone, Debug)]
struct CompositeItem {
    id: String,
    element: NodeRef,
    label: String,
    disabled: bool,
}

/// Menu parent type
#[derive(Clone, Debug, PartialEq)]
pub enum MenuParent {
    Root,
    Submenu,
    Menubar,
    ContextMenu,
    Toolbar,
}

/// Transition status
#[derive(Clone, Debug, PartialEq)]
pub enum TransitionStatus {
    /// No transition
    None,
    /// Entering
    Entering,
    /// Entered
    Entered,
    /// Exiting
    Exiting,
    /// Exited
    Exited,
}

impl MenuStore {
    /// Create a new menu store
    pub fn new(parent: MenuParent) -> Self {
        let store = Self {
            open: RwSignal::new(false),
            mounted: RwSignal::new(false),
            transition_status: RwSignal::new(TransitionStatus::None),
            active_trigger_element: RwSignal::new(None),
            popup_ref: RwSignal::new(NodeRef::new()),
            parent: RwSignal::new(parent),
            internal: RwSignal::new(MenuInternalState::new()),
        };
        
        store
    }
    
    /// Get open state
    pub fn open(&self) -> RwSignal<bool> {
        self.open
    }
    
    /// Get mounted state
    pub fn mounted(&self) -> RwSignal<bool> {
        self.mounted
    }
    
    /// Get transition status
    pub fn transition_status(&self) -> RwSignal<TransitionStatus> {
        self.transition_status
    }
    
    /// Get active trigger element
    pub fn active_trigger_element(&self) -> RwSignal<Option<NodeRef>> {
        self.active_trigger_element
    }
    
    /// Get popup ref
    pub fn popup_ref(&self) -> RwSignal<NodeRef> {
        self.popup_ref
    }
    
    /// Get parent type
    pub fn parent(&self) -> RwSignal<MenuParent> {
        self.parent
    }
    
    /// Get internal state
    pub fn internal(&self) -> RwSignal<MenuInternalState> {
        self.internal
    }
    
    /// Set open state
    pub fn set_open(&self, open: bool) {
        let was_open = self.open.get();
        
        if was_open == open {
            return; // No change
        }
        
        self.open.set(open);
        
        // Handle mount/unmount
        if open {
            self.mounted.set(true);
        } else {
            // Start exit transition
            self.transition_status.set(TransitionStatus::Exiting);
            
            // Schedule unmount after transition
            let store = self.clone();
            leptos::utils::set_timeout_with_handle(
                move || {
                    store.mounted.set(false);
                    store.transition_status.set(TransitionStatus::Exited);
                },
                std::time::Duration::from_millis(200), // Match transition duration
            );
        }
        
        // Call on_open_change handler if present
        if let Some(callback) = &self.internal().get_untracked().on_open_change {
            callback.call(open);
        }
    }
    
    /// Register a trigger
    pub fn register_trigger(&self, id: String, element: NodeRef) {
        let mut internal = self.internal().get_untracked();
        internal.trigger_registry.register(id, element);
        self.internal.set(internal);
    }
    
    /// Unregister a trigger
    pub fn unregister_trigger(&self, id: &str) {
        let mut internal = self.internal().get_untracked();
        internal.trigger_registry.unregister(id);
        self.internal.set(internal);
    }
    
    /// Set on open change handler
    pub fn set_on_open_change(&self, callback: Callback<bool>) {
        let mut internal = self.internal().get_untracked();
        internal.on_open_change = Some(SendWrapper::new(callback));
        self.internal.set(internal);
    }
    
    /// Register composite item
    pub fn register_composite_item(&self, item: CompositeItem) {
        let mut internal = self.internal().get_untracked();
        internal.composite_items.register(item);
        self.internal.set(internal);
    }
    
    /// Unregister composite item
    pub fn unregister_composite_item(&self, id: &str) {
        let mut internal = self.internal().get_untracked();
        internal.composite_items.unregister(id);
        self.internal.set(internal);
    }
}

impl MenuInternalState {
    fn new() -> Self {
        Self {
            trigger_registry: TriggerRegistry::new(),
            composite_items: CompositeItems::new(),
            on_open_change: None,
        }
    }
}

impl TriggerRegistry {
    fn new() -> Self {
        Self {
            triggers: Rc::new(RefCell::new(Vec::new())),
        }
    }
    
    fn register(&mut self, id: String, element: NodeRef) {
        self.triggers.borrow_mut().push(TriggerEntry { id, element, active: false });
    }
    
    fn unregister(&mut self, id: &str) {
        self.triggers.borrow_mut().retain(|entry| entry.id != id);
    }
    
    fn get(&self) -> Vec<TriggerEntry> {
        self.triggers.borrow().clone()
    }
}

impl CompositeItems {
    fn new() -> Self {
        Self {
            items: Rc::new(RefCell::new(Vec::new())),
        }
    }
    
    fn register(&mut self, item: CompositeItem) {
        self.items.borrow_mut().push(item);
    }
    
    fn unregister(&mut self, id: &str) {
        self.items.borrow_mut().retain(|item| item.id != id);
    }
    
    fn get(&self) -> Vec<CompositeItem> {
        self.items.borrow().clone()
    }
}