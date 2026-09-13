//! Combobox Store - Central state management for all combobox parts
//!
//! Port of Base UI's Combobox store to Leptos.
//! This store drives all combobox parts and manages the state machine.

use leptos::*;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::__rt::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, Event as WebEvent};
use std::rc::Rc;
use std::cell::RefCell;

/// Combobox store state
#[derive(Clone, Debug)]
pub struct ComboboxStore<T = String> {
    /// Whether the combobox is open
    pub open: bool,
    
    /// Currently selected value(s)
    pub value: Option<T>,
    
    /// Current input text
    pub input_value: String,
    
    /// Whether multiple selection is enabled
    pub multiple: bool,
    
    /// Whether the combobox is disabled
    pub disabled: bool,
    
    /// Whether the combobox is read-only
    pub read_only: bool,
    
    /// Whether the combobox is required
    pub required: bool,
    
    /// Whether to use grid navigation
    pub grid: bool,
    
    /// Whether to render inline (no portal)
    pub inline: bool,
    
    /// Whether to use modal behavior
    pub modal: bool,
    
    /// Whether to use virtual scrolling
    pub virtualized: bool,
    
    /// Whether to auto-highlight the first item
    pub auto_highlight: bool,
    
    /// Whether to highlight items on hover
    pub highlight_item_on_hover: bool,
    
    /// Whether to loop navigation at ends
    pub loop_focus: bool,
    
    /// Whether to open on input click
    pub open_on_input_click: bool,
    
    /// Available items
    pub items: Vec<T>,
    
    /// Pre-filtered items (for controlled filtering)
    pub filtered_items: Vec<T>,
    
    /// Custom filter function
    pub filter: Option<Callback<(T, String), bool>>,
    
    /// Maximum number of items to show
    pub limit: Option<i32>,
    
    /// Currently highlighted item index (filtered coordinates)
    pub active_index: Option<usize>,
    
    /// Selected item index (filtered coordinates)  
    pub selected_index: Option<usize>,
    
    /// Input element reference
    pub input_element: Option<NodeRef<HtmlInputElement>>,
    
    /// Trigger element reference
    pub trigger_element: Option<NodeRef<HtmlElement>>,
    
    /// Popup element reference
    pub popup_element: Option<NodeRef<HtmlElement>>,
    
    /// List element reference
    pub list_element: Option<NodeRef<HtmlElement>>,
    
    /// Command handlers
    pub commands: ComboboxCommands,
}

/// Command handlers for programmatic control
#[derive(Clone)]
pub struct ComboboxCommands {
    /// Set the open state
    pub set_open: Rc<dyn Fn(bool, Option<ComboboxChangeReason>)>,
    
    /// Set the input value
    pub set_input_value: Rc<dyn Fn(String, Option<ComboboxChangeReason>)>,
    
    /// Set the selected value(s)
    pub set_selected_value: Rc<dyn Fn(Option<T>, Option<ComboboxChangeReason>)>,
    
    /// Set the active and selected indices
    pub set_indices: Rc<dyn Fn(Option<usize>, Option<usize>)>,
    
    /// Handle item selection
    pub handle_selection: Rc<dyn Fn(T, Option<ComboboxChangeReason>)>,
    
    /// Force mount the popup
    pub force_mount: Rc<dyn Fn(bool)>,
    
    /// Request form submission
    pub request_submit: Rc<dyn Fn()>,
    
    /// Callback when open change completes
    pub on_open_change_complete: Rc<dyn Fn(bool)>,
}

/// Change reasons for combobox state transitions
#[derive(Clone, Debug, PartialEq)]
pub enum ComboboxChangeReason {
    /// Item was pressed/clicked
    ItemPress,
    /// Input was cleared
    InputClear,
    /// Trigger was pressed
    TriggerPress,
    /// Keyboard interaction
    Keyboard,
    /// Opened programmatically
    None,
    /// Cancelled open
    CancelOpen,
}

impl Default for ComboboxChangeReason {
    fn default() -> Self {
        Self::None
    }
}

impl ComboboxStore {
    /// Create a new combobox store
    pub fn new<T>() -> Self {
        Self {
            open: false,
            value: None,
            input_value: String::new(),
            multiple: false,
            disabled: false,
            read_only: false,
            required: false,
            grid: false,
            inline: false,
            modal: false,
            virtualized: false,
            auto_highlight: true,
            highlight_item_on_hover: true,
            loop_focus: true,
            open_on_input_click: false,
            items: Vec::new(),
            filtered_items: Vec::new(),
            filter: None,
            limit: None,
            active_index: None,
            selected_index: None,
            input_element: None,
            trigger_element: None,
            popup_element: None,
            list_element: None,
            commands: ComboboxCommands::new(),
        }
    }
    
    /// Update the store state
    pub fn update<T>(&mut self, updater: impl FnOnce(&mut ComboboxStore<T>)) {
        updater(self);
    }
    
    /// Get the currently filtered items
    pub fn get_filtered_items<T>(&self) -> &[T] {
        &self.filtered_items
    }
    
    /// Get the currently active item
    pub fn get_active_item<T>(&self) -> Option<&T> {
        self.active_index
            .and_then(|idx| self.filtered_items.get(idx))
    }
    
    /// Get the currently selected item
    pub fn get_selected_item<T>(&self) -> Option<&T> {
        self.selected_index
            .and_then(|idx| self.filtered_items.get(idx))
    }
    
    /// Check if an item is highlighted
    pub fn is_item_highlighted<T>(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.get_active_item() == Some(item)
    }
    
    /// Check if an item is selected
    pub fn is_item_selected<T>(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.get_selected_item() == Some(item)
    }
    
    /// Apply filtering to items
    pub fn apply_filtering<T>(&mut self, query: &str)
    where
        T: Clone,
    {
        if self.filter.is_none() && query.is_empty() {
            // No filter and empty query = show all items
            self.filtered_items = self.items.clone();
            return;
        }
        
        self.filtered_items = self.items.iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if let Some(filter_fn) = &self.filter {
                    // Use custom filter
                    if filter_fn(item.clone(), query.to_string()) {
                        Some(item.clone())
                    } else {
                        None
                    }
                } else {
                    // Default contains filter
                    let label = format!("{:?}", item); // TODO: Use item_to_string_label
                    if label.to_lowercase().contains(&query.to_lowercase()) {
                        Some(item.clone())
                    } else {
                        None
                    }
                }
            })
            .collect();
            
        // Apply limit if specified
        if let Some(limit) = self.limit {
            if limit > 0 {
                self.filtered_items.truncate(limit as usize);
            }
        }
        
        // Reset indices if they're out of bounds
        if self.filtered_items.len() == 0 {
            self.active_index = None;
            self.selected_index = None;
        } else {
            if let Some(active_idx) = self.active_index {
                if active_idx >= self.filtered_items.len() {
                    self.active_index = None;
                }
            }
            
            if let Some(selected_idx) = self.selected_index {
                if selected_idx >= self.filtered_items.len() {
                    self.selected_index = None;
                }
            }
        }
    }
    
    /// Move to the next item
    pub fn move_next<T>(&mut self)
    where
        T: Clone,
    {
        if self.filtered_items.is_empty() {
            return;
        }
        
        match self.active_index {
            Some(idx) => {
                if self.loop_focus {
                    // Wrap to beginning
                    self.active_index = Some(0);
                } else {
                    // Move to next, stop at end
                    if idx < self.filtered_items.len() - 1 {
                        self.active_index = Some(idx + 1);
                    } else {
                        self.active_index = None; // Clear highlight at end
                    }
                }
            }
            None => {
                // Start from beginning
                self.active_index = Some(0);
            }
        }
    }
    
    /// Move to the previous item
    pub fn move_previous<T>(&mut self)
    where
        T: Clone,
    {
        if self.filtered_items.is_empty() {
            return;
        }
        
        match self.active_index {
            Some(idx) => {
                if self.loop_focus {
                    // Wrap to end
                    self.active_index = Some(self.filtered_items.len() - 1);
                } else {
                    // Move to previous, stop at beginning
                    if idx > 0 {
                        self.active_index = Some(idx - 1);
                    } else {
                        self.active_index = None; // Clear highlight at beginning
                    }
                }
            }
            None => {
                // Start from end
                self.active_index = Some(self.filtered_items.len() - 1);
            }
        }
    }
    
    /// Select the currently active item
    pub fn select_active_item<T>(&mut self)
    where
        T: Clone,
    {
        if let Some(idx) = self.active_index {
            if let Some(item) = self.filtered_items.get(idx) {
                if self.multiple {
                    // Toggle selection in multiple mode
                    if let Some(ref mut current_value) = self.value {
                        // TODO: Handle multiple value toggling
                        // This is simplified - need to store Vec<T> for multiple mode
                    }
                } else {
                    // Set single value
                    self.value = Some(item.clone());
                }
                self.selected_index = Some(idx);
            }
        }
    }
    
    /// Clear the current selection
    pub fn clear_selection<T>(&mut self) {
        self.value = None;
        self.selected_index = None;
        self.input_value = String::new();
    }
    
    /// Clear the current query/highlight
    pub fn clear_query<T>(&mut self) {
        self.active_index = None;
        self.input_value = String::new();
    }
}

impl ComboboxCommands {
    /// Create new command handlers
    pub fn new<T>() -> Self {
        Self {
            set_open: Rc::new(|_open, _reason| {
                // TODO: Implement
            }),
            set_input_value: Rc::new(|_value, _reason| {
                // TODO: Implement
            }),
            set_selected_value: Rc::new(|_value, _reason| {
                // TODO: Implement
            }),
            set_indices: Rc::new(|_active_idx, _selected_idx| {
                // TODO: Implement
            }),
            handle_selection: Rc::new(|_item, _reason| {
                // TODO: Implement
            }),
            force_mount: Rc::new(|_mounted| {
                // TODO: Implement
            }),
            request_submit: Rc::new(|| {
                // TODO: Implement
            }),
            on_open_change_complete: Rc::new(|_open| {
                // TODO: Implement
            }),
        }
    }
}

/// Hook to create and access the combobox store
pub fn use_combobox_store<T>() -> ReadSignal<ComboboxStore<T>> {
    // TODO: Implement actual store creation and access
    // For now, return a default store
    let store = ComboboxStore::new::<T>();
    create_rw_signal(store)
}

/// Hook to get mutable access to the combobox store
pub fn use_combobox_store_mut<T>() -> WriteSignal<ComboboxStore<T>> {
    // TODO: Implement actual mutable store access
    let store = ComboboxStore::new::<T>();
    create_rw_signal(store)
}

/// Hook to get commands from the combobox store
pub fn use_combobox_commands<T>() -> ComboboxCommands {
    ComboboxCommands::new::<T>()
}