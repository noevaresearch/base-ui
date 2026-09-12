//! The Field contexts — `FieldRootContext` and `FieldItemContext`
//! (`packages/react/src/field/root/FieldRoot.tsx:152-184`,
//! `packages/react/src/field/item/FieldItemContext.ts`).
//!
//! React context ports to the component crate's leptos-owned bags (the
//! accordion/meter precedent: the internals crate's reactive-graph-0.2 runtime cannot
//! drive leptos 0.7's view tree — every signal that feeds a view rides the leptos
//! types; internals' rg-0.2 handles are read untracked, callback-time only). The root
//! bag mirrors upstream's `FieldRootContext` shape with the
//! `validityData`/`setValidityData` pair collapsed onto one handle (the
//! internals-module convention). This bag is the *inbound* boundary for the future
//! form-aware controls (Checkbox, RadioGroup, Select, … register through
//! `register_field_control` + `validation`).

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use web_sys::HtmlInputElement;

use leptos_ui_internals::field_constants::{FieldRootState, FieldValidityData};
use leptos_ui_internals::field_register_control::FieldControlRegistration;
use leptos_ui_internals::form_context::FormValidationMode;
use leptos_ui_internals::labelable_provider::ControlIdSource;

use crate::field::validation::{AttributeFn, FieldValidation};

/// The registration entry point — upstream's `registerFieldControl(source, registration)`
/// (`FieldRootContext.ts:25-28`); `None` unregisters.
pub type RegisterFieldControlFn = Rc<dyn Fn(ControlIdSource, Option<FieldControlRegistration>)>;

/// The resolved field state bag the parts consume — upstream's memoized state object
/// (`FieldRoot.tsx:111-121`). Every member rides its own leptos signal so the parts'
/// `data-*` attributes stay live.
#[derive(Clone)]
pub struct FieldStateValue {
    /// `disabled` (`:113`) — the root disabled || the fieldset inheritance (the
    /// fieldset unit is unported, so the inheritance is permanently `false`).
    pub disabled: Signal<bool>,
    /// `touched` (`:114`).
    pub touched: Signal<bool>,
    /// `dirty` (`:115`).
    pub dirty: Signal<bool>,
    /// `valid` (`:116`) — the tri-state: `None` is upstream's `null` (unvalidated or
    /// disabled-suppressed, `FieldRoot.tsx:109`).
    pub valid: Signal<Option<bool>>,
    /// `filled` (`:117`).
    pub filled: Signal<bool>,
    /// `focused` (`:118`).
    pub focused: Signal<bool>,
}

impl FieldStateValue {
    /// The `FieldRootState` snapshot at call time (upstream's render-scoped object).
    pub fn snapshot(&self) -> FieldRootState {
        FieldRootState {
            disabled: self.disabled.get_untracked(),
            touched: self.touched.get_untracked(),
            dirty: self.dirty.get_untracked(),
            valid: self.valid.get_untracked(),
            filled: self.filled.get_untracked(),
            focused: self.focused.get_untracked(),
        }
    }
}

/// The imperative `actionsRef` handle (`FieldRoot.tsx:233-235`).
pub type FieldRootActions = Rc<dyn Fn()>;

/// The root context bag — upstream's `FieldRootContext` value (`FieldRoot.tsx:152-184`;
/// shape `FieldRootContext.ts:12-30`).
#[derive(Clone)]
pub struct FieldRootContext {
    /// `invalid` (`:154`) — the app-controlled flag (`invalid` prop || form error),
    /// not the computed validity.
    pub invalid: Signal<bool>,
    /// `name` (`:155`) — the effective name (root prop, else the registered control's
    /// fallback — the source's resolution order).
    pub name: Signal<Option<String>>,
    /// `validityData` + `setValidityData` (`:156-157`) — one handle.
    pub validity_data: RwSignal<FieldValidityData>,
    /// `disabled` (`:158`).
    pub disabled: Signal<bool>,
    /// `setTouched` (`:159`) — the guarded setter.
    pub set_touched: Rc<dyn Fn(bool)>,
    /// `setDirty` (`:160`) — the guarded setter that also feeds `markedDirtyRef`.
    pub set_dirty: Rc<dyn Fn(bool)>,
    /// `setFilled` (`:161`).
    pub set_filled: Rc<dyn Fn(bool)>,
    /// `setFocused` (`:162`).
    pub set_focused: Rc<dyn Fn(bool)>,
    /// `validationMode` (`:163`).
    pub validation_mode: FormValidationMode,
    /// `shouldValidateOnChange` (`:164`).
    pub should_validate_on_change: Rc<dyn Fn() -> bool>,
    /// `state` (`:165`).
    pub state: FieldStateValue,
    /// `registerFieldControl` (`:166`).
    pub register_field_control: RegisterFieldControlFn,
    /// `validation` (`:167`).
    pub validation: FieldValidation,
}

/// The provider-less default shell (`FieldRootContext.ts:32-61`): inert setters, an
/// inert validity record, and the no-op registration — a stray control stays inert
/// exactly as upstream's NOOP bag keeps it. Built fresh per access (the
/// internals-module convention: owner-registered reactive storage must not live in
/// statics).
fn inert_field_root_context() -> FieldRootContext {
    let inert_touched: RwSignal<bool> = RwSignal::new(false);
    let inert_dirty: RwSignal<bool> = RwSignal::new(false);
    let inert_filled: RwSignal<bool> = RwSignal::new(false);
    let inert_focused: RwSignal<bool> = RwSignal::new(false);
    FieldRootContext {
        invalid: Signal::derive(|| false),
        name: Signal::derive(|| None),
        validity_data: RwSignal::new(FieldValidityData::default()),
        disabled: Signal::derive(|| false),
        set_touched: Rc::new(move |v| inert_touched.set(v)),
        set_dirty: Rc::new(move |v| inert_dirty.set(v)),
        set_filled: Rc::new(move |v| inert_filled.set(v)),
        set_focused: Rc::new(move |v| inert_focused.set(v)),
        validation_mode: FormValidationMode::OnSubmit,
        should_validate_on_change: Rc::new(|| false),
        state: FieldStateValue {
            disabled: Signal::derive(|| false),
            touched: Signal::derive(|| false),
            dirty: Signal::derive(|| false),
            valid: Signal::derive(|| None),
            filled: Signal::derive(|| false),
            focused: Signal::derive(|| false),
        },
        register_field_control: Rc::new(|_, _| {}),
        validation: FieldValidation::inert(),
    }
}

/// `useFieldRootContext()` (`FieldRootContext.ts:65-75`, the default-`optional`
/// form): the Control's accessor — falls back to the inert shell outside a Root.
/// The bag is provided through the `SendWrapper` bridge (the `SharedFormContext`
/// precedent); the accessor unwraps it.
pub fn use_field_root_context() -> FieldRootContext {
    use_context::<SendWrapper<FieldRootContext>>()
        .map(|shared| (*shared).clone())
        .unwrap_or_else(inert_field_root_context)
}

/// `useFieldRootContext(false)` (`FieldRootContext.ts:68-72`): the required form —
/// Label/Description/Error/Validity/Item throw outside a Root with the upstream
/// message. The shell check is the validation bag's inert marker (`FieldValidation::
/// is_inert`), the `setValidityData === NOOP` identity check's port.
pub fn use_field_root_context_required() -> FieldRootContext {
    let context = use_field_root_context();
    if context.validation.is_inert() {
        panic!(
            "Base UI: FieldRootContext is missing. Field parts must be placed within <Field.Root>."
        );
    }
    context
}

/// `FieldItemContext` (`FieldItemContext.ts:4-14`): `{ disabled }` per `Field.Item`.
#[derive(Clone)]
pub struct FieldItemContext {
    /// The item's resolved disabled (root disabled || the item prop).
    pub disabled: bool,
}

/// `useFieldItemContext()` (`FieldItemContext.ts:17-22`): `{ disabled: false }`
/// outside an Item.
pub fn use_field_item_context() -> FieldItemContext {
    use_context::<FieldItemContext>().unwrap_or(FieldItemContext { disabled: false })
}

/// The shared registered-inputs type — upstream's `RegisteredInputs`
/// (`useFieldValidation.ts:23`) as the insertion-ordered vec; re-homed here for the
/// control's wiring.
pub type RegisteredInputsMap =
    Rc<RefCell<Vec<(HtmlInputElement, crate::field::validation::RegisteredInput)>>>;

/// The lazy attribute-value closure — the crate-wide convention
/// (`use_render_element.rs`'s `ElementAttributeFn` shape).
pub type FieldAttributeFn = AttributeFn;
