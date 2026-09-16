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
pub fn CollapsibleTrigger(children: Children) -> impl IntoView {
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

// ---------------------------------------------------------------------------
// The namespaced part surface (`Collapsible::Root`, `Collapsible::Trigger`, …)
// ---------------------------------------------------------------------------
//
// Upstream teaches `<Collapsible.Root><Collapsible.Trigger /><Collapsible.Panel>`; this port's
// spelling is the same tree with Rust's path separator (`specs/docs-content/CONTRACT.md`, the
// React→Rust mapping table). Each item forwards through the macro-generated props struct of the
// `Collapsible*` wrapper right above — one implementation, one props surface, a second *name*
// (the one the docs examples must teach). `pub use self::collapsible as Collapsible;` in
// `lib.rs` is what makes `<Collapsible::Root>` resolvable from a consumer.

/// `Collapsible.Root` — upstream's `<Collapsible.Root>`; same component as [`CollapsibleRoot`].
#[allow(non_snake_case)]
pub fn Root(props: CollapsibleRootProps) -> impl IntoView {
    CollapsibleRoot(props)
}

/// `Collapsible.Trigger` — upstream's `<Collapsible.Trigger>`; same component as
/// [`CollapsibleTrigger`].
#[allow(non_snake_case)]
pub fn Trigger(props: CollapsibleTriggerProps) -> impl IntoView {
    CollapsibleTrigger(props)
}

/// `Collapsible.Panel` — upstream's `<Collapsible.Panel>`; same component as [`CollapsiblePanel`].
#[allow(non_snake_case)]
pub fn Panel(props: CollapsiblePanelProps) -> impl IntoView {
    CollapsiblePanel(props)
}
