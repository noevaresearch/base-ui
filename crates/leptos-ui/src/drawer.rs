//! Port of the Base UI Drawer — the `library: drawer` TODO item
//! (`specs/library/drawer/behavior.md`, `specs/library/drawer/implementation.md`).

use crate::dialog::*;
use leptos::prelude::*;
use std::rc::Rc;

// Re-export dialog types
pub use crate::dialog::{DialogRootComponent, DialogRootProps};

// The drawer handle type alias
pub type DrawerHandle = DialogHandle;

// Simple drawer implementation - wraps dialog with additional drawer-specific functionality
pub fn drawer_root(
    props: DialogRootProps,
    children: leptos::children::ChildrenFn,
) -> impl IntoView {
    let DialogRootProps {
        open,
        default_open,
        on_open_change,
        on_open_change_complete,
        disable_pointer_dismissal,
        modal,
        trigger_id,
        default_trigger_id,
        handle,
        mode,
    } = props;

    // For now, just use the dialog root with some drawer-specific defaults
    let drawer_props = DialogRootProps {
        open,
        default_open,
        on_open_change,
        on_open_change_complete,
        disable_pointer_dismissal: false, // Drawers typically allow pointer dismissal
        modal: false,                     // Drawers are typically not modal
        trigger_id,
        default_trigger_id,
        handle,
        mode,
    };

    view! {
        <DialogRootComponent
            dialog_props=drawer_props
            children=children
        />
    }
}

// Drawer handle creation function using the same pattern as alert dialog
pub fn create_drawer_handle() -> Rc<DialogHandle> {
    crate::dialog::create_handle()
}
