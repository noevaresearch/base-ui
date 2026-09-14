//! Menu item - a regular menu item
//! 
//! This is a port of Base UI's MenuItem from React to Leptos.

use leptos::prelude::*;
use leptos_ui_internals::*;
use leptos_ui_utils::*;
use crate::menu::store::{use_menu_store, MenuStoreContext};
use crate::menu::utils::{MenuEventReason, MenuInteractionType};
use leptos::ev::KeyboardEvent;
use wasm_bindgen::JsCast;

/// Menu item component
/// 
/// A regular menu item that can be selected and triggered.
#[component]
pub fn MenuItem(
    /// Whether the item is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether to close the menu when this item is clicked
    #[prop(default = true)]
    close_on_click: bool,
    /// Custom content for the item
    children: Children,
    /// Custom render function for the item
    #[prop(optional)]
    render: Option<fn() -> web_sys::HtmlElement>,
    /// Unique identifier for the item
    #[prop(into, optional)]
    id: Option<String>,
    /// Callback when the item is selected
    #[prop(default = None)]
    on_select: Option<impl Fn(MenuInteractionType) + Clone + 'static>,
) -> impl IntoView {
    let store = use_menu_store();
    let open = store.open();
    
    // For now, we'll render a simple menu item
    // In a real implementation, this would have more complex behavior
    
    view! {
        <div
            class="menu-item"
            data-disabled=disabled
            data-id=id
            role="menuitem"
            tabindex=if disabled { None } else { Some(0) }
            on:click=move |_| {
                if !disabled && close_on_click {
                    store.set_open(false);
                }
            }
            on:keydown=move |ev: KeyboardEvent| {
                if !disabled {
                    match ev.key().as_str() {
                        "Enter" | " " => {
                            ev.prevent_default();
                            // Trigger the menu item action
                            // In a real implementation, this would call an on_select callback
                            if let Some(on_select) = on_select.clone() {
                                on_select(MenuInteractionType::Keyboard);
                            }
                        },
                        "ArrowDown" | "ArrowUp" => {
                            ev.prevent_default();
                            // Navigate within menu items
                            // In a real implementation, this would navigate to next/prev item
                        },
                        _ => {}
                    }
                }
            }
        >
            {children()}
        </div>
    }
}

/// Hook to use menu item functionality
pub fn use_menu_item() -> (ReadSignal<bool>, impl Fn(bool)) {
    let store = use_menu_store();
    let open = store.open();
    let (read, write) = create_signal(open.get_untracked());
    Effect::new(move || {
        write.set(open.get());
    });
    (read, move |is_open: bool| store.set_open(is_open))
}

/// Hook to get the menu item ID
pub fn use_menu_item_id() -> Option<String> {
    // In a real implementation, this would read from context or props
    None
}

/// Hook to check if the menu item is disabled
pub fn use_menu_item_disabled() -> bool {
    // In a real implementation, this would read from context or props
    false
}

/// Hook to get the menu item close on click state
pub fn use_menu_item_close_on_click() -> bool {
    // In a real implementation, this would read from context or props
    true
}