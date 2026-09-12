//! The field contexts — `FieldRootContext` and `FieldItemContext`
//! (`packages/react/src/field/root/FieldRoot.tsx:152-193`,
//! `packages/react/src/field/item/FieldItemContext.ts`).
//!
//! React context ports to the component crate's leptos-owned bags (the
//! accordion/meter precedent: the internals crate's reactive-graph-0.2 runtime
//! cannot drive leptos 0.7's view tree). The root bag mirrors upstream's
//! `FieldRootContext` shape — derived state, the validity record, the four state
//! setters, the registration entry point, and the whole validation machine — with
//! the `setValidityData`/`validityData` pair collapsed onto one signal handle (the
//! `field_root_context.rs` internals-module convention).

use std::rc::Rc;
use std::cell::RefCell;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::wrappers::read::Signal;
use serde_json::Value;
use web_sys::HtmlInputElement;

use leptos_ui_internals::field_constants::{FieldRootState, FieldValidityData};
use leptos_ui_internals::state_attributes::{
    get_state_attributes_props, StateAttributeProps, StateAttributesMapping,
};

use crate::field::validation::FieldValidation;

/// The resolved field state bag the parts consume — upstream's `state: FieldRootState`
/// context member (`FieldRoot.tsx:165`), the memoized record at `:111-121`. Every
/// member rides its own leptos signal so parts' `data-*` attributes stay live.
#[derive(Clone)]
pub struct FieldStateValue {
    /// `disabled` (`:113`) — root disabled || the fieldset inheritance (the fieldset
    /// unit is unported; the hook returns `None`, matching `useFieldsetRootContext(true)?.disabled`
    /// being `undefined` outside a `<Fieldset.Root>`).
    pub disabled: Signal<bool>,
    /// `touched` (`:114`) — the controlled prop resolved over the internal state.
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
    /// The JSON state map [`field_state_attributes`] walks — `disabled`, `touched`,
    /// `dirty`, `valid`, `filled`, `focused` exactly as upstream's state object is
    /// keyed (`FieldRoot.tsx:112-120`); `valid: null` stays `Value::Null` so the
    /// custom mapping emits nothing (the neutral phase).
    pub fn to_state_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("disabled".to_string(), Value::Bool(self.disabled.get_untracked()));
        map.insert("touched".to_string(), Value::Bool(self.touched.get_untracked()));
        map.insert("dirty".to_string(), Value::Bool(self.dirty.get_untracked()));
        map.insert(
            "valid".to_string(),
            match self.valid.get_untracked() {
                Some(true) => Value::Bool(true),
                Some(false) => Value::Bool(false),
                None => Value::Null,
            },
        );
        map.insert("filled".to_string(), Value::Bool(self.filled.get_untracked()));
        map.insert("focused".to_string(), Value::Bool(self.focused.get_untracked()));
        map
    }
}

/// The root context bag — upstream's `FieldRootContext` value (`FieldRoot.tsx:152-184`;
/// shape `FieldRootContext.ts:12-30`). The control-interop surface (what the future
/// form-aware controls consume): `register_field_control` + `validation`.
#[derive(Clone)]
pub struct FieldRootContext {
    /// `invalid` (`FieldRoot.tsx:154`) — the app-controlled flag (`invalid` prop ||
    /// form error), not the computed validity.
    pub invalid: Signal<bool>,
    /// `name` (`:155`) — the *effective* name: the root prop, else the registered
    /// control's fallback (the source's resolution order — the discrepancy-log entry).
    pub name: Signal<Option<String>>,
    /// `validityData` + `setValidityData` (`:156-157`) — one signal handle (the
    /// internals-module convention).
    pub validity_data: RwSignal<FieldValidityData>,
    /// `disabled` (`:158`).
    pub disabled: Signal<bool>,
    /// `setTouched` (`:159`) — the guarded setter: a no-op while `touched` is controlled.
    pub set_touched: Rc<dyn Fn(bool)>,
    /// `setDirty` (`:160`) — the guarded setter that also feeds `markedDirtyRef`.
    pub set_dirty: Rc<dyn Fn(bool)>,
    /// `setFilled` (`:161`).
    pub set_filled: Rc<dyn Fn(bool)>,
    /// `setFocused` (`:162`).
    pub set_focused: Rc<dyn Fn(bool)>,
    /// `validationMode` (`:163`) — the resolved mode (root prop ?? the Form's).
    pub validation_mode: leptos_ui_internals::form_context::FormValidationMode,
    /// `shouldValidateOnChange` (`:164`) — `onChange` mode, or `onSubmit` after a submit.
    pub should_validate_on_change: Rc<dyn Fn() -> bool>,
    /// `state` (`:165`) — the six-member derived bag.
    pub state: FieldStateValue,
    /// `registerFieldControl` (`:166`) — the root-side registration
    /// (`useFieldControlRegistration`'s `register` half).
    pub register_field_control:
        Rc<dyn Fn(leptos_ui_internals::labelable_provider::ControlIdSource, Option<leptos_ui_internals::field_register_control::FieldControlRegistration>)>,
    /// `validation` (`:167`) — the machine (`getValidationProps`, the input registry,
    /// `commit`/`change`).
    pub validation: FieldValidation,
}

/// `useFieldRootContext()` (`FieldRootContext.ts:65-75`, the default-`optional` form):
/// falls back to an inert shell outside a Root — upstream's NOOP default bag. The
/// shell's setters write to owner-local signals nobody observes and its registration
/// is a no-op, so a stray control stays inert exactly as upstream's default keeps it.
pub fn use_field_root_context() -> FieldRootContext {
    use_context::<FieldRootContext>().unwrap_or_else(|| {
        let inert_touched: RwSignal<bool> = RwSignal::new(false);
        let inert_dirty: RwSignal<bool> = RwSignal::new(false);
        let inert_filled: RwSignal<bool> = RwSignal::new(false);
        let inert_focused: RwSignal<bool> = RwSignal::new(false);
        let inert_validity: RwSignal<FieldValidityData> =
            RwSignal::new(FieldValidityData::default());
        FieldRootContext {
            invalid: Signal::derive(|| false),
            name: Signal::derive(|| None),
            validity_data: inert_validity,
            disabled: Signal::derive(|| false),
            set_touched: Rc::new(move |v| inert_touched.set(v)),
            set_dirty: Rc::new(move |v| inert_dirty.set(v)),
            set_filled: Rc::new(move |v| inert_filled.set(v)),
            set_focused: Rc::new(move |v| inert_focused.set(v)),
            validation_mode:
                leptos_ui_internals::form_context::FormValidationMode::OnSubmit,
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
    })
}

/// `useFieldRootContext(false)` (`FieldRootContext.ts:68-72`): the required form —
/// Label/Description/Error/Validity/Item throw outside a Root with the upstream message.
pub fn use_field_root_context_required() -> FieldRootContext {
    // The shell-detection port: upstream checks `setValidityData === NOOP`; the port's
    // equivalent inert marker is the validation bag's inert registry identity — an
    // accessor-level check keeps the parts' code identical to upstream's shape.
    let context = use_field_root_context();
    if context.validation.is_inert() {
        panic!(
            "Base UI: FieldRootContext is missing. Field parts must be placed within <Field.Root>."
        );
    }
    context
}

/// `FieldItemContext` (`FieldItemContext.ts:4-14`): `{ disabled }` per `Field.Item` —
/// Label and Description OR it into their state so items inside a group render
/// `data-disabled` without disabling siblings.
#[derive(Clone)]
pub struct FieldItemContext {
    /// The item's resolved disabled (root disabled || the item prop).
    pub disabled: bool,
}

/// `useFieldItemContext()` (`FieldItemContext.ts:17-22`): defaults to
/// `{ disabled: false }` outside an Item.
pub fn use_field_item_context() -> FieldItemContext {
    use_context::<FieldItemContext>().unwrap_or(FieldItemContext { disabled: false })
}

/// The `fieldValidityMapping` state walk (`constants.ts:34-43` through the ported
/// [`get_state_attributes_props`]): `valid: true` → `data-valid`, `valid: false` →
/// `data-invalid`, `valid: null` → nothing, every other truthy member → the generic
/// `data-<key>`. The port's per-render attribute derivation for every part — the
/// state record is re-walked at each reactive read (the progress
/// `get_state_attributes_props` precedent, materialized into the typed slots).
pub fn field_state_attributes(
    state: &FieldStateValue,
) -> StateAttributeProps {
    let mut state_map = serde_json::Map::new();
    state_map.insert("disabled".to_string(), Value::Bool(state.disabled.get_untracked()));
    state_map.insert("touched".to_string(), Value::Bool(state.touched.get_untracked()));
    state_map.insert("dirty".to_string(), Value::Bool(state.dirty.get_untracked()));
    state_map.insert(
        "valid".to_string(),
        match state.valid.get_untracked() {
            Some(v) => Value::Bool(v),
            None => Value::Null,
        },
    );
    state_map.insert("filled".to_string(), Value::Bool(state.filled.get_untracked()));
    state_map.insert("focused".to_string(), Value::Bool(state.focused.get_untracked()));
    get_state_attributes_props(
        &state_map,
        Some(
            &(leptos_ui_internals::state_attributes::field_validity_mapping
                as StateAttributesMapping),
        ),
    )
}

/// The shared input registry type — upstream's `RegisteredInputs`
/// (`useFieldValidation.ts:23`) as the insertion-ordered vec (the internals-module
/// convention); re-exported here for the control's wiring.
pub type RegisteredInputsMap =
    Rc<RefCell<Vec<(HtmlInputElement, leptos_ui_internals::field_root_context::RegisteredInput)>>>;
