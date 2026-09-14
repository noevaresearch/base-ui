//! Menu root - the main menu container that manages state and provides context
//! 
//! This is a port of Base UI's MenuRoot from React to Leptos.

use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store, MenuStoreContext};
use leptos::ev::{KeyboardEvent, MouseEvent};
use wasm_bindgen::JsCast;

/// Menu root component
/// 
/// The main container that manages menu state and provides context to child components.
#[component]
pub fn MenuRoot(
    /// Whether the menu is open
    #[prop(optional)]
    open: Option<Signal<bool>>,
    /// Callback triggered when the menu open state changes
    #[prop(optional)]
    on_open_change: Option<Callback<bool>>,
    /// Whether the menu is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Menu orientation
    #[prop(default = MenuOrientation::Vertical)]
    orientation: MenuOrientation,
    /// Whether the menu is modal (blocks outside clicks)
    #[prop(default = true)]
    modal: bool,
    /// Whether to highlight items on hover
    #[prop(default = true)]
    highlight_item_on_hover: bool,
    /// Root ID for nested menus
    #[prop(optional)]
    root_id: Option<String>,
    children: Children,
) -> impl IntoView {
    let open_signal = open.unwrap_or_else(|| create_rw_signal(false).into());
    let store_context = use_menu_store();
    
    // Provide the store context
    provide_context(store_context);
    
    // Render the menu root
    view! {
        <div
            class="menu-root"
            data-open=open_signal.get()
            data-disabled=disabled
            data-orientation=match orientation {
                MenuOrientation::Horizontal => "horizontal",
                MenuOrientation::Vertical => "vertical",
            }
            data-modal=modal
            data-highlight-on-hover=highlight_item_on_hover
        >
            {children()}
        </div>
    }
}

/// Menu orientation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuOrientation {
    Horizontal,
    Vertical,
}

impl Default for MenuOrientation {
    fn default() -> Self {
        MenuOrientation::Vertical
    }
}

/// Hook to use the menu root context
pub fn use_menu_root_context() -> MenuStoreContext {
    use_context().expect("MenuRootContext not found")
}

/// Hook to get the menu open state
pub fn use_menu_open() -> Signal<bool> {
    let store = use_menu_store();
    store.open().into()
}

/// Hook to set the menu open state
pub fn use_menu_set_open() -> impl Fn(bool) + 'static {
    let store = use_menu_store();
    move |is_open| store.set_open(is_open)
}