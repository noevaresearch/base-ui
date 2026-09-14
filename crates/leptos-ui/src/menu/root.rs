//! Menu root - the main menu container that manages state and provides context
//! 
//! This is a port of Base UI's MenuRoot from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use leptos::ev::{MouseEvent, KeyboardEvent};
use leptos::prelude::*;

/// Root component for the menu system
/// 
/// Manages the menu state and provides context to all child components.
#[component]
pub fn MenuRoot(
    /// Whether the menu is open (controlled)
    #[prop(into, optional)]
    open: Option<RwSignal<bool>>,
    /// Default open state (uncontrolled)
    #[prop(default = false)]
    default_open: bool,
    /// Callback when the menu opens or closes
    #[prop(into, optional)]
    on_open_change: Option<Callback<bool>>,
    /// Whether the menu is modal (blocks outside clicks)
    #[prop(default = true)]
    modal: bool,
    /// Whether to highlight items on hover
    #[prop(default = true)]
    highlight_item_on_hover: bool,
    /// Menu orientation
    #[prop(default = MenuOrientation::Vertical)]
    orientation: MenuOrientation,
    /// Root ID for nested menus
    #[prop(into, optional)]
    root_id: Option<String>,
    /// Whether the menu is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    let open_signal = open.unwrap_or_else(|| create_rw_signal(default_open));
    
    // Create the menu store
    let mut store = MenuStore::new();
    store.set_open(open_signal.get_untracked());
    store.set_modal(modal);
    store.set_highlight_item_on_hover(highlight_item_on_hover);
    store.set_orientation(orientation);
    store.set_root_id(root_id.clone());
    
    // Set parent as root
    store.set_parent(Some(MenuParent::Root));
    
    // Create the store context
    let store_context = MenuStoreContext::new(store);
    
    // Handle open changes
    let on_open_change = on_open_change.unwrap_or_else(|| Callback::new(|_| {}));
    
    // Effect to sync open state
    Effect::new(move || {
        let is_open = open_signal.get();
        store_context.set_open(is_open);
        on_open_change.call(is_open);
    });
    
    // Effect to handle disabled state
    Effect::new(move || {
        if disabled {
            open_signal.set(false);
        }
    });
    
    // Provide the store context
    provide_context(store_context);
    
    // Render the menu root
    view! {
        <div class="menu-root" role="none">
            {children()}
        </div>
    }
}

/// Hook to access the menu store context
pub fn use_menu_store() -> MenuStoreContext {
    use_context::<MenuStoreContext>().expect("MenuStoreContext not found")
}

/// Hook to access the menu open state
pub fn use_menu_open() -> RwSignal<bool> {
    use_menu_store().open()
}

/// Hook to access the menu orientation
pub fn use_menu_orientation() -> MenuOrientation {
    use_menu_store().orientation()
}

/// Hook to access the menu modal state
pub fn use_menu_modal() -> bool {
    use_menu_store().modal()
}

/// Hook to access the menu highlight item on hover state
pub fn use_menu_highlight_item_on_hover() -> bool {
    use_menu_store().highlight_item_on_hover()
}

/// Hook to access the menu root ID
pub fn use_menu_root_id() -> Option<String> {
    use_menu_store().root_id()
}

/// Hook to access the menu parent information
pub fn use_menu_parent() -> Option<MenuParent> {
    use_menu_store().parent()
}
