use leptos::*;
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

    let context = use_collapsible_root_context();

    let open = context.state.open;
    let transition_status = context.state.transition_status;

    // Panel id management
    let panel_id = if let Some(id) = id {
        Some(id.into_owned())
    } else {
        None
    };

    let is_hidden = !open && !keep_mounted && !hidden_until_found;

    let should_prevent_open_animation = create_rw_signal(false);
    let force_panel_idle = create_rw_signal(false);

    leptos::view! {
        <div
            id={if let Some(id) = panel_id { Some(id) } else { None }}
            hidden={is_hidden}
            class:keep-mounted=keep_mounted
            class:hidden-until-found=hidden_until_found
        >
            {children()}
        </div>
    }
}