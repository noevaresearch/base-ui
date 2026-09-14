//! Context Menu Trigger Component

use leptos::prelude::*;
use web_sys::MouseEvent;

/// Context menu trigger component
#[component]
pub fn ContextMenuTrigger(
    /// Children components
    children: Children,
) -> impl IntoView {
    let handle_context_menu = move |event: MouseEvent| {
        // Prevent default context menu behavior
        event.prevent_default();
    };

    view! {
        <div
            on:contextmenu=handle_context_menu
            on:click=move |event: MouseEvent| {
                // Prevent default click behavior when right-clicking
                if event.button() == 2 {
                    event.prevent_default();
                }
            }
        >
            {children()}
        </div>
    }
}
