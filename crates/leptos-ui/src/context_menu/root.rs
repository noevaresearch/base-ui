//! Context Menu Root Component

use leptos::prelude::*;

/// Context menu root component
#[component]
pub fn ContextMenuRoot(
    /// Whether the context menu is open by default
    #[prop(default = false)]
    default_open: bool,
    /// Children components
    children: Children,
) -> impl IntoView {
    let (open, set_open) = signal(default_open);

    view! {
        <div
            data-context-menu
            data-open={move || open.get().then_some("true")}
            data-closed={move || (!open.get()).then_some("true")}
        >
            {children()}
        </div>
    }
}