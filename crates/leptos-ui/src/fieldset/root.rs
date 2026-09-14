//! Port of the Base UI Fieldset — the `library: fieldset` TODO item
//! (`specs/library/fieldset/behavior.md`, `specs/library/fieldset/implementation.md`).

use leptos::prelude::*;

/// Fieldset root context
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FieldsetRootContext {
    /// Current legend id (set by legend)
    pub legend_id: Signal<Option<String>>,
    /// Setter for legend id
    pub set_legend_id: WriteSignal<Option<String>>,
    /// Effective disabled state
    pub disabled: bool,
}

/// Fieldset root component - the main fieldset container
#[component]
pub fn FieldsetRoot(
    /// Whether the fieldset is disabled
    #[prop(default = false.into())]
    disabled: bool,
    children: Children,
) -> impl IntoView {
    // Read parent fieldset context if it exists
    let parent_context = use_context::<FieldsetRootContext>();
    let parent_disabled = parent_context.map(|ctx| ctx.disabled).unwrap_or(false);

    // Internal state for legend id
    let (legend_id, set_legend_id) = signal(None::<String>);

    // Effective disabled: parent OR prop
    let effective_disabled = parent_disabled || disabled;

    // Context provider
    provide_context(FieldsetRootContext {
        legend_id: legend_id.into(),
        set_legend_id,
        disabled: effective_disabled,
    });

    view! {
        <fieldset
            disabled=effective_disabled
            aria-labelledby=move || legend_id.get().unwrap_or_default()
        >
            {children()}
        </fieldset>
    }
}

/// Fieldset legend component
#[component]
pub fn FieldsetLegend(
    /// Optional custom id for the legend
    #[prop(optional)]
    id: Option<String>,
    children: Children,
) -> impl IntoView {
    // Consume fieldset root context
    let context = use_context::<FieldsetRootContext>()
        .expect("FieldsetLegend must be used within FieldsetRoot");

    // Register this legend with the root
    context.set_legend_id.set(id.clone());

    on_cleanup(move || {
        // Clear the legend id when this legend unmounts
        context.set_legend_id.set(None);
    });

    view! {
        <div id=id>
            {children()}
        </div>
    }
}
