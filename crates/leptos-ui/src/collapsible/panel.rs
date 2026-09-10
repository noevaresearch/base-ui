use leptos::prelude::*;
use crate::collapsible::*;

pub fn collapsible_panel(
    id: impl Into<Option<CowStr>>,
    keep_mounted: impl Into<bool> + Copy,
    hidden_until_found: impl Into<bool> + Copy,
    children: Children,
) -> impl IntoView {
    let id = id.into();
    let keep_mounted = keep_mounted.into();
    let hidden_until_found = hidden_until_found.into();

    let context = use_collapsible_root_context().unwrap();

    let open = context.state.open.get();
    let transition_status = context.state.transition_status;
    let panel_id_signal = context.set_panel_id_state;

    let panel_id = if let Some(id) = id {
        Some(id.into_owned())
    } else {
        None
    };

    let is_hidden = !open && !keep_mounted && !hidden_until_found;

    let panel_id_for_attr = if let Some(id) = panel_id {
        Some(id)
    } else {
        Some(context.default_panel_id.clone())
    };

    // Panel id management
    let reactive_panel_id = if let Some(id) = panel_id {
        create_rw_signal(id)
    } else {
        panel_id_signal
    };

    let is_hidden = !open && !keep_mounted && !hidden_until_found;

    leptos::view! {
        <div
            id={reactive_panel_id}
            hidden={is_hidden}
            class:keep-mounted=keep_mounted
            class:hidden-until-found=hidden_until_found
        >
            {children()}
        </div>
    }
}