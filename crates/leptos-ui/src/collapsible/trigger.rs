use leptos::*;
use crate::collapsible::*;

pub fn collapsible_trigger(
    id: impl Into<Option<CowStr>>,
    disabled: impl Into<Option<bool>>,
    children: Children,
) -> impl IntoView {
    let id = id.into();
    let disabled = disabled.into();

    let context = use_collapsible_root_context();

    let is_disabled = context.state.disabled || disabled.unwrap_or(false);

    let open = context.state.open;
    let panel_id = context.default_panel_id.clone();

    let trigger_id = if let Some(id) = id {
        Some(id.into_owned())
    } else {
        None
    };

    // Handle trigger click
    let handle_click = move |_| {
        if is_disabled {
            return;
        }

        let new_open = !open;
        let cancel_signal = create_rw_signal(false);
        let details = ChangeEventDetails {
            reason: ChangeReason::TriggerPress,
            is_canceled: cancel_signal,
        };

        (context.on_open_change)(new_open, details);
    };

    // Determine aria-expanded
    let aria_expanded = open;

    // Determine aria-controls
    let aria_controls = if open { Some(panel_id.clone()) } else { None };

    leptos::view! {
        <button
            type="button"
            disabled=is_disabled
            aria-expanded=aria_expanded
            aria-controls=aria_controls
            class:disabled=is_disabled
            on:click=handle_click
        >
            {children()}
        </button>
    }
}