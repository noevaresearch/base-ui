use leptos::*;
use crate::collapsible::State;

/// Context for collapsible root
#[derive(Clone)]
pub struct CollapsibleRootContext {
    pub on_open_change: Box<dyn Fn(bool, ChangeEventDetails) + Send + Sync>,
    pub default_panel_id: String,
    pub set_panel_id_state: Box<dyn Fn(Option<String>) + Send + Sync>,
    pub state: State,
}

#[derive(Clone)]
pub struct CollapsibleRootProvider {
    pub children: Children,
}

pub fn use_collapsible_root_context() -> CollapsibleRootContext {
    leptos::use_context::<CollapsibleRootContext>()
        .expect("CollapsibleRootContext is missing. Collapsible parts must be placed within <Collapsible.Root>.")
}

pub fn provide_collapsible_root_context(
    on_open_change: impl Fn(bool, ChangeEventDetails) + Send + Sync + 'static,
    default_panel_id: impl Into<CowStr>,
    set_panel_id_state: impl Fn(Option<String>) + Send + Sync + 'static,
    default_open: bool,
    disabled: impl Into<bool> + Copy,
    transition_status: TransitionStatus,
) -> CollapsibleRootProvider {
    let default_panel_id = default_panel_id.into();
    let on_open_change = Box::new(on_open_change);
    let set_panel_id_state = Box::new(set_panel_id_state);

    let state = State {
        open: default_open,
        disabled: disabled.into(),
        transition_status,
    };

    leptos::provide_context(CollapsibleRootContext {
        on_open_change,
        default_panel_id,
        set_panel_id_state,
        state,
    });

    CollapsibleRootProvider {
        children: leptos::Children::default(),
    }
}