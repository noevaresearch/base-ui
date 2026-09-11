use leptos::prelude::*;

/// Collapsible Root component - the main container that manages state
#[component]
pub fn CollapsibleRoot(
    /// Whether the collapsible is open by default
    #[prop(default = false.into())]
    default_open: bool,
    /// Whether the collapsible is disabled
    #[prop(default = false.into())]
    disabled: bool,
    children: Children,
) -> impl IntoView {
    let (open, set_open) = signal(default_open);
    
    // Context provider
    provide_context::<CollapsibleContext>(CollapsibleContext {
        open,
        set_open,
        disabled,
    });
    
    view! {
        <div
            data-disabled={move || disabled.then_some("true")}
            data-open={move || open.get().then_some("true")}
            data-closed={move || (!open.get()).then_some("true")}
        >
            {children()}
        </div>
    }
}

/// Collapsible Trigger component - the button that toggles the collapsible
#[component]
pub fn CollapsibleTrigger(
    children: Children,
) -> impl IntoView {
    let context = expect_context::<CollapsibleContext>();
    
    view! {
        <button
            aria-expanded={move || context.open.get().then_some("true")}
            on:click=move |_| {
                if !context.disabled {
                    let next_open = !context.open.get_untracked();
                    context.set_open.set(next_open);
                }
            }
            disabled=context.disabled
        >
            {children()}
        </button>
    }
}

/// Collapsible Panel component - the content that shows/hides
#[component]
pub fn CollapsiblePanel(
    /// Whether to keep the panel mounted in the DOM when closed
    #[prop(default = false.into())]
    keep_mounted: bool,
    children: Children,
) -> impl IntoView {
    let context = expect_context::<CollapsibleContext>();
    
    view! {
        <div class=move || {
            if keep_mounted || context.open.get() {
                "visible"
            } else {
                "hidden"
            }
        }>
            {children()}
        </div>
    }
}

/// Context for collapsible components
#[derive(Clone)]
pub struct CollapsibleContext {
    pub open: ReadSignal<bool>,
    pub set_open: WriteSignal<bool>,
    pub disabled: bool,
}

/// Re-export all components for easier imports
pub mod prelude {
    pub use super::*;
}