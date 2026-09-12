//! Port of the Base UI Field unit — the `library: field` TODO item
//! (`specs/library/field/behavior.md`, `specs/library/field/implementation.md`).
//!
//! The unit renders inline (no portals — implementation.md "DOM/portal strategy"),
//! so every part is a real Leptos view component: the accordion/meter precedent
//! (component-crate context values on leptos signals; the internals crate's
//! reactive-graph-0.2 runtime cannot drive leptos 0.7's view tree, the cross-crate
//! owner split the meter wasm run caught). The internals vocabulary is used where it
//! is view-independent: [`FIELD_VALIDITY_MAPPING`] (the state walk),
//! [`use_transition_status`] (FieldError's mount/exit animation) and the
//! `DEFAULT_VALIDITY_STATE`/[`FieldValidityData`] constants ride the internals
//! exports; the reactive bags (`FieldRootContextValue` and friends) are re-homed
//! here because their signals must be leptos-owned.
//!
//! Structural facts this port follows (implementation.md unless noted):
//!
//! - Root state is four independent booleans + the validity record; controlled
//!   `dirty`/`touched` resolve at read and gate the internal setters
//!   (`FieldRoot.tsx:50-85`).
//! - `markedDirtyRef` is the dirty-gate input of the validator's `valueMissing`
//!   suppression (`FieldRoot.tsx:58,63-67`; `useFieldValidation.ts:218-221`).
//! - App-controlled invalidity (`invalid` prop + `<Form>` errors) survives
//!   `disabled`; computed validity is suppressed to `null` while disabled
//!   (`FieldRoot.tsx:93-109`).
//! - The validation engine is one `commit` closure with an epoch guard: stale async
//!   results are discarded, `change` debounces only while validating on change and
//!   the value is non-empty, and native errors short-circuit the custom validator
//!   outside onChange mode (`useFieldValidation.ts:123-365`).
//! - Custom validity ownership writes the field's message to the real input via
//!   `setCustomValidity`, remembering the message it displaced and restoring it
//!   only when the control still shows the field's own message
//!   (`useFieldValidation.ts:167-182`).
//! - Registration is the two-hook pair: root-side
//!   [`use_field_control_registration`] owns the Form registry entry and the
//!   control handover; control-side [`use_register_field_control`] pushes the
//!   registration in place on every change (`field_register_control.rs`).

pub mod field_root;
pub mod field_control;
pub mod field_parts;
pub mod context;

pub use context::{
    FieldItemContext, FieldRootContext, FieldStateValue, use_field_item_context,
    use_field_root_context,
};
pub use field_control::{FieldControl, FieldControlProps, field_control};
pub use field_parts::{
    FieldDescription, FieldError, FieldItem, FieldLabel, FieldValidity,
};
pub use field_root::{FieldRoot, FieldRootProps, field_root_view};

// The attributes module re-derivation lives in `field_parts` (the
// `data-valid`/`data-invalid` walk) and is exported from `context` for the parts'
// shared use.
pub use context::field_state_attributes;
