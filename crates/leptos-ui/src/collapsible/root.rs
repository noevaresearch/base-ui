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

    let open_signal = match open {
        Some(o) => create_rw_signal(o),
        None => create_rw_signal(default_open),
    };

    let disabled_signal = create_rw_signal(disabled);

    let panel_id = create_rw_signal(use_base_ui_id());

    let transition_status = use_transition_status(
        open_signal,
        true,  // enable_idle_state
        true,  // defer_ending_state
        false, // animate_initial_open
    );

    let state = State {
        open: open_signal,
        disabled: disabled_signal,
        transition_status: transition_status.transition_status,
    };

    let set_panel_id_state = Box::new(move |panel_id: Option<String>| {
        panel_id.set(panel_id);
    });

    let collapse_trigger = move |new_open: bool| {
        let cancel_signal = create_rw_signal(false);
        let details = ChangeEventDetails {
            reason: ChangeReason::TriggerPress,
            is_canceled: cancel_signal,
        };
        on_open_change.clone()(new_open, details);
        if !cancel_signal.get() {
            open_signal.set(new_open);
        }
    };

    leptos::view! {
        <CollapsibleRootProvider
            default_panel_id={panel_id.get().unwrap_or_default()}
            set_panel_id_state={set_panel_id_state}
            state={state}
            disabled={disabled_signal}
            on_open_change={collapse_trigger}
        >
            {children()}
        </CollapsibleRootProvider>
    }
}

#[derive(Clone)]
pub struct CollapsibleRootProvider {
    pub default_panel_id: String,
    pub set_panel_id_state: Box<dyn Fn(Option<String>) + Send + Sync>,
    pub state: State,
    pub disabled: RwSignal<bool>,
    pub on_open_change: Box<dyn Fn(bool, ChangeEventDetails) + Send + Sync>,
}

impl IntoView for CollapsibleRootProvider {
    fn into_view(self) -> View {
        leptos::view! {
            <div>
                {self.children()}
            </div>
        }
    }
}