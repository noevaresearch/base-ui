//! Port of the Base UI Field unit — the `library: field` TODO item
//! (`specs/library/field/behavior.md`, `specs/library/field/implementation.md`).
//!
//! The unit renders inline (no portals — implementation.md "DOM/portal strategy"),
//! so every part is a real Leptos view component: the accordion/meter precedent
//! (component-crate context bags on leptos signals; the internals crate's
//! reactive-graph-0.2 runtime cannot drive leptos 0.7's view tree, the cross-crate
//! owner split the meter wasm run caught). The internals vocabulary stays
//! view-independent: the record shapes (`DEFAULT_VALIDITY_STATE`,
//! [`leptos_ui_internals::field_constants::FieldValidityData`]), the
//! `get_combined_field_validity_data` combination, `get_state_attributes_props` +
//! `field_validity_mapping` (the state walk), and the `use_transition_status` hook
//! (bridged through a dedicated owner in `validation_helpers`).
//!
//! Module map:
//! - [`context`] — the re-homed `FieldRootContext`/`FieldItemContext` bags.
//! - [`validation`] — the `useFieldValidation` machine re-homed on leptos signals.
//! - [`registration`] — the re-homed root-side `useFieldControlRegistration`.
//! - [`field_root`] — `Field.Root`.
//! - [`field_control`] — `Field.Control`.
//! - [`field_parts`] — Label / Description / Error / Validity / Item.
//! - [`parts_view`] — the shared state-walk materialization.
//! - [`validation_helpers`] — the id generator + the transition-status bridge.

pub mod context;
pub mod field_control;
pub mod field_parts;
pub mod field_root;
pub mod parts_view;
pub mod registration;
pub mod validation;
pub mod validation_helpers;
