//! Port of the Base UI Checkbox unit (`packages/react/src/checkbox`) — the
//! `library: checkbox` TODO item (`specs/library/checkbox/behavior.md`,
//! `specs/library/checkbox/implementation.md`,
//! `specs/library/checkbox/implementation.citations.json`).
//!
//! Upstream's shape (implementation.md, "State machine / hooks used"): no reducer —
//! one controlled boolean (`useControlled`, `CheckboxRoot.tsx:137-145`) plus a
//! render-only `indeterminate` flag, wrapped in hooks that each own one concern
//! (`useButton`, the two layout effects, `useValueChanged`,
//! `useRegisterFieldControl`, the labelable pair, the CheckboxGroup/Field/Form
//! contexts). All user interaction funnels through the hidden input's single native
//! `change` event (`:211-246`); the visible control re-dispatches its click onto that
//! input (`:365-378`).
//!
//! Module map:
//! - [`state`] — the pure folds, the state record, the state→attribute walk and the
//!   change-funnel gates (host-testable, view-independent).
//! - [`root`] — `Checkbox.Root`: the three-sibling DOM, the context provider, the
//!   Field/Form/Labelable/CheckboxGroup integration, the listeners.
//! - [`indicator`] — `Checkbox.Indicator`: the transition-status mount machine and
//!   the mirrored attribute walk.
//!
//! Runtime law (the crate's dual-runtime split — `field/mod.rs`): leptos 0.7 tracks
//! reactive-graph 0.1 while the internals crate's hooks are 0.2-typed. The unit
//! re-homes its state machine on leptos signals (the accordion/`Field.Control`
//! precedent) and bridges the rg-0.2 handles it still consumes
//! (`CheckboxGroupContext.value`, `useLabelableId`, `useAriaLabelledBy`,
//! `useTransitionStatus`, `useOpenChangeComplete`) one crossing at a time.

pub mod indicator;
pub mod root;
pub mod state;

pub use indicator::{
    CheckboxIndicatorHandlers, CheckboxIndicatorViewProps, checkbox_indicator_view,
};
pub use root::{
    CheckboxRootContextValue, CheckboxRootHandlers, CheckboxRootViewProps,
    MISSING_ROOT_CONTEXT_MESSAGE, checkbox_root_view, provide_checkbox_root_context,
    try_use_checkbox_root_context, use_checkbox_root_context,
};
pub use state::{
    ChangeFunnel, CheckboxChangeEventDetails, CheckboxRootState, DATA_CHECKED, DATA_DIRTY,
    DATA_DISABLED, DATA_FILLED, DATA_FOCUSED, DATA_INDETERMINATE, DATA_INVALID, DATA_READONLY,
    DATA_REQUIRED, DATA_TOUCHED, DATA_UNCHECKED, DATA_VALID, PARENT_CHECKBOX, change_event_details,
    checkbox_state_attributes, computed_checked, computed_indeterminate, controlled_checked,
    effective_disabled, effective_name, effective_value, get_checkbox_state_attributes_mapping,
    indicator_rendered, indicator_should_render, input_id, input_name, input_value,
    run_change_funnel, should_render_unchecked_value_input, splice_group_value,
};
