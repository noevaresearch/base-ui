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
            data-disabled={disabled.then_some("true")}
            data-open={open.get().then_some("true")}
            data-closed={(!open.get()).then_some("true")}
        >
            <slot/>
        </div>
    }
}

/// Collapsible Trigger component - the button that toggles the collapsible
#[component]
pub fn CollapsibleTrigger() -> impl IntoView {
    let context = expect_context::<CollapsibleContext>();
    
    view! {
        <button
            aria-expanded={context.open.get().then_some("true")}
            on:click=move |_| {
                if !context.disabled {
                    let next_open = !context.open.get_untracked();
                    context.set_open.set(next_open);
                }
            }
            disabled=context.disabled
        >
            <slot/>
        </button>
    }
}

/// Collapsible Panel component - the content that shows/hides
#[component]
pub fn CollapsiblePanel(
    /// Whether to keep the panel mounted in the DOM when closed
    #[prop(default = false.into())]
    keep_mounted: bool,
) -> impl IntoView {
    let context = expect_context::<CollapsibleContext>();
    
    let visibility_class = if keep_mounted || context.open.get() {
        "visible"
    } else {
        "hidden"
    };
    
    view! {
        <div class={visibility_class}>
            <slot/>
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