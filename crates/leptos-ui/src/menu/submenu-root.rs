//! Menu submenu root - root component for nested menus
//! 
//! This is a port of Base UI's MenuSubmenuRoot from React to Leptos.

use leptos::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{MenuStore, MenuStoreContext, MenuParent, MenuOrientation};

/// Props for the menu submenu root component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSubmenuRootProps {
    /// Whether the submenu is open (controlled)
    #[prop(into, optional)]
    open: Option<RwSignal<bool>>,
    /// Default open state (uncontrolled)
    #[prop(default = false)]
    default_open: bool,
    /// Callback when the submenu opens or closes
    #[prop(into, optional)]
    on_open_change: Option<Callback<bool>>,
    /// Whether the submenu is modal (blocks outside clicks)
    #[prop(default = true)]
    modal: bool,
    /// Whether to highlight items on hover
    #[prop(default = true)]
    highlight_item_on_hover: bool,
    /// Menu orientation
    #[prop(default = MenuOrientation::Vertical)]
    orientation: MenuOrientation,
    /// Whether the submenu is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Custom ID for the submenu
    #[prop(into, optional)]
    id: Option<String>,
    /// Children components
    children: Children,
}

/// Submenu root component for the menu
/// 
/// Root component for nested menus.
#[component]
pub fn MenuSubmenuRoot(
    /// Props for the menu submenu root
    #[prop(optional)]
    props: MenuSubmenuRootProps,
) -> impl IntoView {
    let MenuSubmenuRootProps {
        open,
        default_open,
        on_open_change,
        modal,
        highlight_item_on_hover,
        orientation,
        disabled,
        id,
        children,
    } = props;
    
    let open_signal = open.unwrap_or_else(|| create_rw_signal(default_open));
    
    // Create the submenu store
    let mut store = MenuStore::new();
    store.set_open(open_signal.get_untracked());
    store.set_modal(modal);
    store.set_highlight_item_on_hover(highlight_item_on_hover);
    store.set_orientation(orientation);
    store.set_root_id(id.clone());
    
    // Set parent as submenu
    store.set_parent(Some(MenuParent::Submenu { 
        id: id.clone().unwrap_or_else(|| "submenu".to_string()) 
    }));
    
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
    
    // Render the submenu root
    view! {
        <div class="menu-submenu-root" role="none">
            {children()}
        </div>
    }
}

impl Default for MenuSubmenuRootProps {
    fn default() -> Self {
        Self {
            open: None,
            default_open: false,
            on_open_change: None,
            modal: true,
            highlight_item_on_hover: true,
            orientation: MenuOrientation::Vertical,
            disabled: false,
            id: None,
            children: Children::new(|_| view! {}),
        }
    }
}

/// Hook to access the submenu store context
pub fn use_submenu_store() -> MenuStoreContext {
    use_context::<MenuStoreContext>().expect("MenuStoreContext not found")
}

/// Hook to access the submenu open state
pub fn use_submenu_open() -> RwSignal<bool> {
    use_submenu_store().open()
}

/// Hook to access the submenu orientation
pub fn use_submenu_orientation() -> MenuOrientation {
    use_submenu_store().orientation()
}

/// Hook to access the submenu modal state
pub fn use_submenu_modal() -> bool {
    use_submenu_store().modal()
}

/// Hook to access the submenu highlight item on hover state
pub fn use_submenu_highlight_item_on_hover() -> bool {
    use_submenu_store().highlight_item_on_hover()
}

/// Hook to access the submenu root ID
pub fn use_submenu_root_id() -> Option<String> {
    use_submenu_store().root_id()
}

/// Hook to access the submenu parent information
pub fn use_submenu_parent() -> Option<MenuParent> {
    use_submenu_store().parent()
}