use leptos::prelude::*;
use leptos_ui_utils::generate_id;
use leptos_ui_internals::use_base_ui_id;

/// Open state for collapsible component
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TransitionStatus {
    Idle,
    Starting,
    Ending,
}

impl std::fmt::Debug for TransitionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Starting => write!(f, "Starting"),
            Self::Ending => write!(f, "Ending"),
        }
    }
}

/// State object passed to render props
#[derive(Clone)]
pub struct State {
    pub open: bool,
    pub disabled: bool,
    pub transition_status: TransitionStatus,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("open", &self.open)
            .field("disabled", &self.disabled)
            .field("transition_status", &self.transition_status)
            .finish()
    }
}

/// Create event change details for collapsible
#[derive(Clone)]
pub struct ChangeEventDetails {
    pub reason: ChangeReason,
    pub is_canceled: RwSignal<bool>,
}

impl ChangeEventDetails {
    pub fn cancel(&self) {
        self.is_canceled.set(true);
    }

    pub fn is_canceled(&self) -> bool {
        *self.is_canceled.read()
    }
}

/// Change reason for collapsible
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChangeReason {
    TriggerPress,
    None,
}