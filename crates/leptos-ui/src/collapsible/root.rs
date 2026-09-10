use leptos::prelude::*;
use crate::collapsible::*;

pub fn collapsible_root(
    default_open: impl Into<bool> + Copy,
    open: impl Into<Option<bool>>,
    on_open_change: impl Fn(bool, ChangeEventDetails) + Send + Sync + 'static,
    disabled: impl Into<bool> + Copy,
    children: Children,
) -> impl IntoView {
    let default_open = default_open.into();
    let open = open.into();
    let disabled = disabled.into();

    // Use base-ui-id for panel id
    let default_panel_id = use_base_ui_id().to_string();

    let on_open_change = Memo::new(move |_| {
        let details = ChangeEventDetails {
            reason: ChangeReason::TriggerPress,
            is_canceled: create_rw_signal(false),
        };
        on_open_change.clone()(default_open, details);
    });

    let state = {
        let open = match open {
            Some(o) => o,
            None => default_open,
        };

        // Use transition status
        let (transition_status, _) = use_transition_status(open, true, true);

        State {
            open,
            disabled,
            transition_status,
        }
    };

    let set_panel_id_state = Box::new(|panel_id: Option<String>| {
        // Panel ID state management will be handled in the panel component
    });

    let collapse_trigger = on_open_change;

    leptos::view! {
        <CollapsibleRootProvider default_open=default_panel_id set_panel_id_state={move |_| {}} state={state} on_open_change={collapse_trigger} disabled={disabled}>
            {children()}
        </CollapsibleRootProvider>
    }
}

#[derive(Clone)]
pub struct CollapsibleRootProvider {
    pub children: Children,
}

impl IntoView for CollapsibleRootProvider {
    fn into_view(self) -> View {
        leptos::view! {
            <div>{self.children()}</div>
        }
    }
}