//! Port of `packages/react/src/internals/form-context/FormContext.ts` — the context bag
//! Form provides and the field machinery consumes (`specs/library/internals/
//! implementation.md`, "Context providers/consumers" — the `FormContext` row: the provider
//! is `Form`, not this unit; the in-unit consumer is
//! [`crate::field_register_control::use_field_control_registration`] reading `formRef`.
//! Untested upstream per "Anything in source not explained by any test" item 4, so the
//! tests below pin the written mechanics per the PrehydrationScript precedent).
//!
//! The load-bearing member is `formRef.current.fields`
//! (`FormContext.ts:13-28`) — the imperative registry Form submits against: `Map<id,
//! { getValue, name, controlRef, validityData, validate }>`. Registration order is submit
//! order, which is why re-registration must update the entry *in place* rather than
//! delete+re-add (the `useRegisterFieldControl.ts:18-20` comment; the
//! `useFieldControlRegistration.ts:86-103` note) — the port's [`FormFields`] is an
//! insertion-ordered registry reproducing JS `Map` `set`/`delete` semantics (the
//! `labelable_provider.rs` registration-vec precedent).
//!
//! ## Rust adaptations
//!
//! - React context ports to reactive-graph's owner-scoped [`use_context`] behind the
//!   [`SharedFormContext`] `SendWrapper` bridge (the `composite_list.rs` precedent). This
//!   unit defines the bag, the default shell, and the accessor; the provide call site is
//!   the Phase B `Form` component (the provider is not in this unit).
//! - The default context (`FormContext.ts:33-46`) is upstream's single module-level object:
//!   every provider-less consumer shares one instance. The port shares the load-bearing
//!   part — the default [`FormRef`] is one [`thread_local`] registry handed out by
//!   [`use_form_context`] — because cross-hook sharing is observable: without a `<Form>`,
//!   the field-registration hooks and the validation machinery (Phase B) must agree on one
//!   `fields` map or validity updates silently no-op. The inert members (`elementRef`,
//!   `submitCountRef`, the errors signal) are built fresh per access: nothing writes them
//!   in the default shell, so sharing them is unobservable — and fresh construction keeps
//!   owner-registered reactive storage out of statics (the `prehydration_script.rs`
//!   `ArcRwSignal` hazard).
//! - `errors: Record<string, string | string[]>` ports to [`FormErrors`], an
//!   insertion-ordered vec of pairs preserving JS string-key object order, with
//!   [`FormErrorValue`] for the `string | string[]` union.
//! - `validationMode: Form.ValidationMode` (`Form.tsx:183`, `:237`) ports to the
//!   [`FormValidationMode`] enum.
//! - `elementRef: RefObject<HTMLFormElement | null>` / `submitCountRef: RefObject<number>`
//!   port to the crate's `Rc<Cell<…>>` ref-slot shape (the `types.rs` convention).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, use_context};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use serde_json::Value;
use web_sys::HtmlFormElement;

use crate::field_constants::FieldValidityData;

// ---------------------------------------------------------------------------
// The fields registry (FormContext.ts:13-28)
// ---------------------------------------------------------------------------

/// The imperative value getter — upstream `getValue: () => unknown`
/// (`FormContext.ts:25`); `None` is the JS `undefined` return.
pub type GetFieldValueFn = Rc<dyn Fn() -> Option<Value>>;

/// One registered field — upstream's fields-map entry
/// (`FormContext.ts:16-27`). `validate`'s JSDoc (`:18-21`): after it returns, the registry
/// entry reflects the latest synchronous validity verdict; async validators do not block
/// submit.
#[derive(Clone)]
pub struct FormFieldEntry {
    /// `name` (`:17`).
    pub name: Option<String>,
    /// `validate` (`:18-21`).
    pub validate: Rc<dyn Fn()>,
    /// `validityData` (`:23`).
    pub validity_data: FieldValidityData,
    /// `controlRef` (`:24`) — `RefObject<HTMLElement | null>`, the crate's ref slot
    /// (`types.rs` convention).
    pub control_ref: Rc<Cell<Option<web_sys::Element>>>,
    /// `getValue` (`:25`).
    pub get_value: GetFieldValueFn,
}

/// `formRef.current.fields: Map<string, FormFieldEntry>` (`FormContext.ts:13-28`) — an
/// insertion-ordered registry reproducing JS `Map` semantics: `set` on an existing key
/// updates **in place** (submit ordering is registration ordering — the
/// `useRegisterFieldControl.ts:18-20` comment), `delete` removes the entry.
#[derive(Clone, Default)]
pub struct FormFields {
    entries: Vec<(String, FormFieldEntry)>,
}

impl FormFields {
    /// `fields.set(id, entry)` (`useFieldControlRegistration.ts:71-77`, `:113-119`) —
    /// upserting in place so a re-registration never reorders the registry.
    pub fn set(&mut self, id: String, entry: FormFieldEntry) {
        match self.entries.iter_mut().find(|(key, _)| *key == id) {
            Some(existing) => *existing = (id, entry),
            None => self.entries.push((id, entry)),
        }
    }

    /// `fields.delete(id)` (`useFieldControlRegistration.ts:80-84`, `:122-131`).
    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|(key, _)| key != id);
        self.entries.len() != before
    }

    /// `fields.get(id)`.
    pub fn get(&self, id: &str) -> Option<&FormFieldEntry> {
        self.entries
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, e)| e)
    }

    /// `fields.size`.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `fields.size === 0`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `(id, entry)` pairs in registration (submit) order.
    pub fn iter(&self) -> impl Iterator<Item = &(String, FormFieldEntry)> {
        self.entries.iter()
    }
}

/// `formRef.current` (`FormContext.ts:13`): the mutable form state the ref object points
/// at — currently just the fields registry.
#[derive(Clone, Default)]
pub struct FormState {
    /// `fields` (`:14-28`).
    pub fields: FormFields,
}

/// `formRef` (`FormContext.ts:13`) — the ref object's port: a shared handle onto
/// [`FormState`] whose interior is mutated in place (the `useFieldControlRegistration`
/// register/cleanup closures all reach the same registry through it).
pub type FormRef = Rc<RefCell<FormState>>;

// ---------------------------------------------------------------------------
// The context bag (FormContext.ts:9-46)
// ---------------------------------------------------------------------------

/// One `errors` record value — upstream `string | string[]` (`FormContext.ts:7`).
#[derive(Clone, Debug, PartialEq)]
pub enum FormErrorValue {
    Single(String),
    Multiple(Vec<String>),
}

/// `Errors` (`FormContext.ts:7`) — `Record<string, string | string[]>`, carried as an
/// insertion-ordered vec of pairs (module docs).
pub type FormErrors = Vec<(String, FormErrorValue)>;

/// The field-validation mode — upstream `Form.ValidationMode` (`Form.tsx:183`, `:237`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FormValidationMode {
    /// `'onSubmit'` — the upstream default (`FormContext.ts:42`).
    #[default]
    OnSubmit,
    /// `'onBlur'`.
    OnBlur,
    /// `'onChange'`.
    OnChange,
}

/// The error-clearing call — upstream `clearErrors: (name: string | undefined) => void`
/// (`FormContext.ts:11`); `None` is the `undefined` name.
pub type ClearErrorsFn = Rc<dyn Fn(Option<&str>)>;

/// The context bag — upstream's `FormContext` (`FormContext.ts:9-31`).
#[derive(Clone)]
pub struct FormContextValue {
    /// `errors` (`:10`) — reactive over the Form's error record (module docs).
    pub errors: Signal<FormErrors, LocalStorage>,
    /// `clearErrors` (`:11`).
    pub clear_errors: ClearErrorsFn,
    /// `elementRef` (`:12`) — the `<form>` element the provider attaches.
    pub element_ref: Rc<Cell<Option<HtmlFormElement>>>,
    /// `formRef` (`:13-28`) — the imperative fields registry.
    pub form_ref: FormRef,
    /// `validationMode` (`:29`).
    pub validation_mode: FormValidationMode,
    /// `submitCountRef` (`:30`).
    pub submit_count_ref: Rc<Cell<u32>>,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `composite_list.rs`
/// precedent). The provide call site is the Phase B `Form` component.
pub type SharedFormContext = SendWrapper<FormContextValue>;

thread_local! {
    /// The shared default fields registry — the load-bearing piece of upstream's
    /// module-level default context object (`FormContext.ts:33-46`): every provider-less
    /// consumer reaches the same `fields` map (module docs).
    static DEFAULT_FORM_REF: FormRef = Rc::new(RefCell::new(FormState::default()));
}

thread_local! {
    /// The default `clearErrors` — upstream's `NOOP` (`FormContext.ts:41`).
    static NOOP_CLEAR_ERRORS: ClearErrorsFn = Rc::new(|_| {});
}

/// The default context shell (`FormContext.ts:33-46`): `elementRef: { current: null }`,
/// the **shared** default `formRef`, empty errors, `clearErrors: NOOP`,
/// `validationMode: 'onSubmit'`, `submitCountRef: { current: 0 }`. The inert members are
/// built fresh per access; only `form_ref` is shared (module docs).
fn default_form_context() -> FormContextValue {
    FormContextValue {
        errors: Signal::derive_local(|| Vec::new()),
        clear_errors: NOOP_CLEAR_ERRORS.with(Rc::clone),
        element_ref: Rc::new(Cell::new(None)),
        form_ref: DEFAULT_FORM_REF.with(Rc::clone),
        validation_mode: FormValidationMode::OnSubmit,
        submit_count_ref: Rc::new(Cell::new(0)),
    }
}

/// The accessor — upstream's `useFormContext` (`FormContext.ts:48-50`), falling back to
/// the default shell outside a provider. Must be called inside a reactive owner (a
/// component), like the other hook ports.
pub fn use_form_context() -> FormContextValue {
    use_context::<SharedFormContext>()
        .map(|shared| (*shared).clone())
        .unwrap_or_else(default_form_context)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::field_constants::{DEFAULT_VALIDITY_STATE, FieldValidityData};
    use reactive_graph::traits::GetUntracked;

    fn entry(id: &str) -> FormFieldEntry {
        FormFieldEntry {
            name: Some(id.to_string()),
            validate: Rc::new(|| {}),
            validity_data: FieldValidityData {
                state: DEFAULT_VALIDITY_STATE,
                ..FieldValidityData::default()
            },
            control_ref: Rc::new(Cell::new(None)),
            get_value: Rc::new(|| None),
        }
    }

    // Pins the in-place upsert (`FormContext.ts:14-28` Map semantics over the
    // `useRegisterFieldControl.ts:18-20` comment): re-registering the same id updates the
    // entry without moving it, so submit ordering is registration ordering.
    #[test]
    fn re_registering_updates_the_entry_in_place() {
        let mut fields = FormFields::default();
        fields.set("a".into(), entry("a"));
        fields.set("b".into(), entry("b"));
        fields.set("a".into(), entry("a2"));

        let order: Vec<&str> = fields.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(
            order,
            ["a", "b"],
            "the in-place update preserves the position"
        );
        assert_eq!(fields.get("a").unwrap().name.as_deref(), Some("a2"));
    }

    // Pins `delete` (`useFieldControlRegistration.ts:80-84`) and that a deleted id can be
    // re-added at the end (JS Map semantics).
    #[test]
    fn delete_removes_and_readd_appends() {
        let mut fields = FormFields::default();
        fields.set("a".into(), entry("a"));
        fields.set("b".into(), entry("b"));

        assert!(fields.delete("a"));
        assert!(!fields.delete("a"), "deleting a missing id is a no-op");

        fields.set("a".into(), entry("a"));
        let order: Vec<&str> = fields.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(order, ["b", "a"], "the re-added entry goes to the end");
    }

    // Pins the shared default registry (module docs): two accessor calls outside a
    // provider reach the same `fields` map — the cross-hook agreement the validation
    // machinery's `formRef.current.fields.get(fieldId)` depends on.
    #[test]
    fn providerless_consumers_share_the_default_fields_registry() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let first = use_form_context();
        let second = use_form_context();

        first
            .form_ref
            .borrow_mut()
            .fields
            .set("shared".into(), entry("s"));
        assert!(
            second.form_ref.borrow().fields.get("shared").is_some(),
            "the default formRef is the shared module-level registry"
        );

        owner.cleanup();
    }

    // Pins the default shell's inert members (`FormContext.ts:33-46`).
    #[test]
    fn the_default_shell_is_inert() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let context = use_form_context();
        assert!(context.errors.get_untracked().is_empty());
        assert_eq!(context.validation_mode, FormValidationMode::OnSubmit);
        assert_eq!(context.submit_count_ref.get(), 0);
        assert!(context.element_ref.take().is_none());

        owner.cleanup();
    }
}
