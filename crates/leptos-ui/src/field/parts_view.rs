//! The Field parts' shared view vocabulary — the state-walk materialization.
//!
//! Upstream every part calls `useRenderElement(tag, componentProps, { state,
//! stateAttributesMapping: fieldValidityMapping })` — the state record walks through
//! `fieldValidityMapping` (`valid: true` → `data-valid`, `valid: false` →
//! `data-invalid`, `null` → nothing) plus the generic truthy arm (`disabled`,
//! `touched`, `dirty`, `filled`, `focused` → `data-<key>`). The port runs the real
//! ported engine ([`get_state_attributes_props`] + [`field_validity_mapping`]) per
//! reactive read and materializes the output into a typed struct the `view!` binds
//! (the progress `StatusAttributes` precedent; a struct avoids the illegal
//! impl-Trait-in-tuple return).

use leptos::prelude::SignalGet;
use reactive_graph::traits::GetUntracked;

use leptos_ui_internals::field_constants::{DEFAULT_FIELD_ROOT_STATE, FieldRootState};
use leptos_ui_internals::state_attributes::{
    StateAttributeProps, StateAttributesMapping, get_state_attributes_props,
};

use crate::field::context::FieldStateValue;

/// The typed materialization of one state walk — exactly one of `data_valid`/
/// `data_invalid` is `Some` (the mapping's output), the rest follow the generic arm.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FieldStateAttributes {
    /// `data-disabled`.
    pub data_disabled: Option<String>,
    /// `data-touched`.
    pub data_touched: Option<String>,
    /// `data-dirty`.
    pub data_dirty: Option<String>,
    /// `data-valid`.
    pub data_valid: Option<String>,
    /// `data-invalid`.
    pub data_invalid: Option<String>,
    /// `data-filled`.
    pub data_filled: Option<String>,
    /// `data-focused`.
    pub data_focused: Option<String>,
}

/// Runs the state walk over a static [`FieldRootState`] snapshot (the real ported
/// engine).
pub fn walk_state(state: &FieldRootState) -> FieldStateAttributes {
    let mut state_map = serde_json::Map::new();
    state_map.insert("disabled".to_string(), serde_json::Value::Bool(state.disabled));
    state_map.insert("touched".to_string(), serde_json::Value::Bool(state.touched));
    state_map.insert("dirty".to_string(), serde_json::Value::Bool(state.dirty));
    state_map.insert(
        "valid".to_string(),
        match state.valid {
            Some(v) => serde_json::Value::Bool(v),
            None => serde_json::Value::Null,
        },
    );
    state_map.insert("filled".to_string(), serde_json::Value::Bool(state.filled));
    state_map.insert("focused".to_string(), serde_json::Value::Bool(state.focused));

    let attributes: StateAttributeProps = get_state_attributes_props(
        &state_map,
        Some(
            &(leptos_ui_internals::state_attributes::field_validity_mapping
                as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>),
        ),
    );
    FieldStateAttributes::from_walk(&attributes)
}

impl FieldStateAttributes {
    /// Materializes the walk's output through the typed slots.
    pub fn from_walk(attributes: &StateAttributeProps) -> Self {
        FieldStateAttributes {
            data_disabled: attributes.get("data-disabled").cloned(),
            data_touched: attributes.get("data-touched").cloned(),
            data_dirty: attributes.get("data-dirty").cloned(),
            data_valid: attributes.get("data-valid").cloned(),
            data_invalid: attributes.get("data-invalid").cloned(),
            data_filled: attributes.get("data-filled").cloned(),
            data_focused: attributes.get("data-focused").cloned(),
        }
    }
}

/// The live attribute struct: each member is a reactive closure over the state bag, so
/// a state flip re-renders the attribute in place (the `data-open={move || …}`
/// accordion precedent). The walk runs per reactive read — the attributes derive, not
/// snapshot.
#[derive(Clone)]
pub struct LiveFieldAttributes {
    /// `data-disabled`.
    pub data_disabled: leptos::prelude::Signal<Option<String>>,
    /// `data-touched`.
    pub data_touched: leptos::prelude::Signal<Option<String>>,
    /// `data-dirty`.
    pub data_dirty: leptos::prelude::Signal<Option<String>>,
    /// `data-valid`.
    pub data_valid: leptos::prelude::Signal<Option<String>>,
    /// `data-invalid`.
    pub data_invalid: leptos::prelude::Signal<Option<String>>,
    /// `data-filled`.
    pub data_filled: leptos::prelude::Signal<Option<String>>,
    /// `data-focused`.
    pub data_focused: leptos::prelude::Signal<Option<String>>,
}

/// Derives the live attribute struct over the state bag. The slot closures read the
/// leptos signals (tracked), so the leptos view re-renders on state flips.
pub fn field_state_attributes(state: &FieldStateValue) -> LiveFieldAttributes {
    let disabled = state.disabled.clone();
    let touched = state.touched.clone();
    let dirty = state.dirty.clone();
    let valid = state.valid.clone();
    let filled = state.filled.clone();
    let focused = state.focused.clone();

    LiveFieldAttributes {
        data_disabled: leptos::prelude::Signal::derive(move || bool_slot(disabled.get())),
        data_touched: leptos::prelude::Signal::derive(move || bool_slot(touched.get())),
        data_dirty: leptos::prelude::Signal::derive(move || bool_slot(dirty.get())),
        data_valid: leptos::prelude::Signal::derive(move || {
            valid.get().and_then(|valid| valid.then(String::new))
        }),
        data_invalid: leptos::prelude::Signal::derive(move || {
            valid.get().and_then(|valid| (!valid).then(String::new))
        }),
        data_filled: leptos::prelude::Signal::derive(move || bool_slot(filled.get())),
        data_focused: leptos::prelude::Signal::derive(move || bool_slot(focused.get())),
    }
}

fn bool_slot(value: bool) -> Option<String> {
    value.then(String::new)
}

/// The walk over a snapshot — the body-time form for static parts and the tests.
pub fn field_state_attributes_snapshot(state: &FieldStateValue) -> FieldStateAttributes {
    walk_state(&state.snapshot())
}

/// The default state constant re-derivation (the inert shell's shape, kept for the
/// tests).
pub fn default_state() -> FieldRootState {
    DEFAULT_FIELD_ROOT_STATE
}

// The untracked trait stays imported for the callback-time readers.
#[allow(unused)]
fn untracked_marker(s: impl GetUntracked<Value = bool>) -> bool {
    s.get_untracked()
}

// `SignalGet` is the leptos-side read trait the closures above use through the
// prelude; the import keeps the single-trait read explicit.
#[allow(unused)]
fn signal_get_marker(s: leptos::prelude::Signal<bool>) -> bool {
    use leptos::prelude::SignalGet as _;
    s.get()
}
