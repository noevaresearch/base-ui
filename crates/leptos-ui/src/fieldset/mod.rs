//! Fieldset components — port of Base UI Fieldset (`packages/react/src/fieldset/`).
//!
//! The `library: fieldset` TODO item. Two parts, one context
//! (`specs/library/fieldset/behavior.md`, `specs/library/fieldset/implementation.md`):
//!
//! - [`root::FieldsetRoot`] / [`root::fieldset_root_view`] — the native `<fieldset>`
//!   (`FieldsetRoot.tsx:34`) that carries the effective `disabled` and the derived
//!   `aria-labelledby`, wrapped around the consumer's subtree by the context provider
//!   (`FieldsetRoot.tsx:55-57`).
//! - [`legend::FieldsetLegend`] / [`legend::fieldset_legend_view`] — the styled
//!   `<div>` (`FieldsetLegend.tsx:28`; deliberately NOT a native `<legend>`,
//!   implementation.md "DOM/portal strategy") that registers its id with the root.
//!
//! Both parts are `useRenderElement` configurations (implementation.md:57 — every
//! conformance-tested behavior is the engine's), split the way the rest of the crate
//! splits a unit: a pure element builder ([`root::fieldset_element`],
//! [`legend::fieldset_legend_element`]) plus a view function that adds the children
//! and materializes the merged bag.
//!
//! Runtime law (the field/collapsible precedent): the context rides the LEPTOS
//! runtime. `Field.Root` reads it through
//! `use_context::<FieldsetRootContext>()` from the leptos owner chain
//! (`crates/leptos-ui/src/field/field_root.rs:202-215`, upstream
//! `FieldRoot.tsx:10,44,48`), and both parts' `data-*`/`aria-*` attributes derive
//! from leptos signals, so nothing in the view tree reads an rg-0.2 handle. The one
//! rg-0.2 seam is the LEGEND's id registration, which rides the ported
//! `use_registered_label_id` hook (its own rg-0.2 owner, `legend.rs`).

pub mod legend;
pub mod root;
pub mod tests;

pub use legend::{
    FieldsetLegend, FieldsetLegendElementProps, FieldsetLegendState, FieldsetLegendViewProps,
    fieldset_legend_element, fieldset_legend_view,
};
pub use root::{
    FieldsetRoot, FieldsetRootContext, FieldsetRootElementProps, FieldsetRootState,
    FieldsetRootViewProps, fieldset_element, fieldset_root_view,
};
