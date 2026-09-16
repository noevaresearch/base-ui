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

// ---------------------------------------------------------------------------
// The namespaced part surface (`Field::Root`, `Field::Label`, …)
// ---------------------------------------------------------------------------
//
// Upstream teaches `<Field.Root><Field.Label /><Field.Control /><Field.Error />`; this port's
// spelling is the same tree with Rust's path separator (`specs/docs-content/CONTRACT.md`, the
// React→Rust mapping table). `Field` documents SEVEN parts (behavior.md "Public API surface",
// `specs/library/field/behavior.md`): Root, Label, Control, Description, Item, Error, Validity —
// and each item below forwards through the macro-generated props struct of the `Field*` wrapper
// that already exists for it, so there is one implementation and one props surface, only a second
// *name* (the one the docs examples must teach). `pub use self::field as Field;` in `lib.rs` is
// what makes `<Field::Root>` resolvable from a consumer.

/// `Field.Root` — upstream's `<Field.Root>`; same component as [`field_root::FieldRoot`].
#[allow(non_snake_case)]
pub fn Root(props: field_root::FieldRootProps) -> impl leptos::IntoView {
    field_root::FieldRoot(props)
}

/// `Field.Control` — upstream's `<Field.Control>`; same component as
/// [`field_control::FieldControl`].
#[allow(non_snake_case)]
pub fn Control(props: field_control::FieldControlProps) -> impl leptos::IntoView {
    field_control::FieldControl(props)
}

/// `Field.Label` — upstream's `<Field.Label>`; same component as [`field_parts::FieldLabel`].
#[allow(non_snake_case)]
pub fn Label(props: field_parts::FieldLabelProps) -> impl leptos::IntoView {
    field_parts::FieldLabel(props)
}

/// `Field.Description` — upstream's `<Field.Description>`; same component as
/// [`field_parts::FieldDescription`].
#[allow(non_snake_case)]
pub fn Description(props: field_parts::FieldDescriptionProps) -> impl leptos::IntoView {
    field_parts::FieldDescription(props)
}

/// `Field.Item` — upstream's `<Field.Item>`; same component as [`field_parts::FieldItem`].
#[allow(non_snake_case)]
pub fn Item(props: field_parts::FieldItemProps) -> impl leptos::IntoView {
    field_parts::FieldItem(props)
}

/// `Field.Error` — upstream's `<Field.Error>`; same component as [`field_parts::FieldError`].
#[allow(non_snake_case)]
pub fn Error(props: field_parts::FieldErrorProps) -> impl leptos::IntoView {
    field_parts::FieldError(props)
}

/// `Field.Validity` — upstream's `<Field.Validity>`; same component as
/// [`field_parts::FieldValidity`].
#[allow(non_snake_case)]
pub fn Validity(props: field_parts::FieldValidityProps) -> impl leptos::IntoView {
    field_parts::FieldValidity(props)
}
