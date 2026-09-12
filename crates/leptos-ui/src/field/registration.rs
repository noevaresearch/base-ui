//! The re-homed root-side registration — port of `useFieldControlRegistration`
//! (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:17-172`).
//!
//! Upstream the hook lives in internals and reads `formRef.current.fields` through
//! context. The port re-homes the reactive parts into the component crate (the
//! accordion/meter precedent) while keeping the internals vocabulary: the
//! [`FormFields`] registry, [`FieldValidityData`], [`get_combined_field_validity_data`].
//!
//! Ownership semantics the hook owns (implementation.md "Registration pair"):
//! - the initial dirty baseline is captured exactly once and belongs to the field,
//!   not to a control instance (`:86-103`);
//! - a replaced control cancels the previous one's pending validation
//!   (`change(undefined, true)`, `:150-153`);
//! - the registry entry carries `getValue`/`name`/`controlRef`/`validityData`/
//!   `validate` (`:65-78`, `:113-120`);
//! - the imperative `validate` marks the field dirty before committing (`:53-63`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set};
use serde_json::Value;
use wasm_bindgen::JsCast;

use leptos_ui_internals::field_constants::FieldValidityData;
use leptos_ui_internals::form_context::{
    FormFieldEntry, FormRef, GetFieldValueFn,
};
use leptos_ui_internals::labelable_provider::ControlIdSource;

/// The hook parameters — the leptos-world handles plus the two Form-bridge slots.
pub struct RootRegistrationParams {
    /// `change` (`:175`).
    pub change: Rc<dyn Fn(Option<Value>, bool)>,
    /// `commit` (`:176`) — the full-signature commit.
    pub commit: Rc<dyn Fn(Value, bool)>,
    /// `invalid` (`:177`).
    pub invalid: reactive_graph::wrappers::read::Signal<bool>,
    /// `markedDirtyRef` (`:178`).
    pub marked_dirty_ref: Rc<Cell<bool>>,
    /// `name` (`:179`) — the root's own name (static per root body).
    pub name: Option<String>,
    /// `setRegisteredFieldName` (`:180`).
    pub set_registered_field_name: Rc<dyn Fn(Option<String>)>,
    /// `registeredFieldIdRef` (`:181`).
    pub registered_field_id_ref: Rc<RefCell<Option<String>>>,
    /// `validityData` + `setValidityData` (`:182-183`) — one handle.
    pub validity_data: RwSignal<FieldValidityData>,
    /// `formRef` — the Form bridge (the internals' inert default outside a `<Form>`).
    pub form_ref: FormRef,
    /// `elementRef` — the `<form>` element slot.
    pub element_ref: Rc<Cell<Option<web_sys::HtmlFormElement>>>,
}

/// The return — upstream's `[validate, register] as const` (`:171`).
pub struct RootRegistrationReturn {
    /// `validate` — the field-level action behind `actionsRef` (`:53-63`).
    pub validate: Rc<dyn Fn()>,
    /// `register` — the `FieldRootContext.registerFieldControl` implementation.
    pub register:
        Rc<dyn Fn(ControlIdSource, Option<leptos_ui_internals::field_register_control::FieldControlRegistration>)>,
}

/// Port of `useFieldControlRegistration`. Must be called inside a reactive owner.
pub fn root_registration(params: RootRegistrationParams) -> (Rc<dyn Fn()>, RootRegistrationReturn) {
    let RootRegistrationParams {
        change,
        commit,
        invalid,
        marked_dirty_ref,
        name,
        set_registered_field_name,
        registered_field_id_ref,
        validity_data,
        form_ref,
        element_ref,
    } = params;

    // The initial-value baseline — captured exactly once, owned by the field
    // (`:86-103`'s `initialValueRef`); the leptos-side validity record holds it
    // (the `initialValue` member), so the once-guard is a local flag.
    let baseline_captured: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // The control handover state: the current control's source and its id — the
    // "registered field id" the Form registry keys on (`:111-119`).
    let current_source: Rc<RefCell<Option<ControlIdSource>>> = Rc::new(RefCell::new(None));
    let current_control_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));

    // `getControlValue` (`:65`): the registry's live value read — the elected
    // representative input's DOM value. The machine's `input_ref` slot is the
    // fallback; the registry projection uses `getValue` per entry, so this closure
    // reads the current control's element value.
    let get_value: GetFieldValueFn = {
        let current_control_ref = Rc::clone(&current_control_ref);
        Rc::new(move || {
            current_control_ref
                .get()
                .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| Value::String(input.value()))
                .unwrap_or(Value::Null)
        })
    };

    // The imperative `validate` (`:53-63`): mark dirty, then commit the fresh control
    // value.
    let validate: Rc<dyn Fn()> = {
        let commit = Rc::clone(&commit);
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        let current_control_ref = Rc::clone(&current_control_ref);
        Rc::new(move || {
            marked_dirty_ref.set(true);
            let value = current_control_ref
                .get()
                .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| Value::String(input.value()))
                .unwrap_or(Value::Null);
            commit(value, false);
        })
    };

    // `register` (`:65-172`): the source-keyed upsert into the Form registry, the
    // name fallback, and the pending-validation cancellation on control replacement.
    let register: Rc<
        dyn Fn(ControlIdSource, Option<leptos_ui_internals::field_register_control::FieldControlRegistration>),
    > = {
        let form_ref = Rc::clone(&form_ref);
        let validity_data = validity_data.clone();
        let registered_field_id_ref = Rc::clone(&registered_field_id_ref);
        let set_registered_field_name = Rc::clone(&set_registered_field_name);
        let current_source = Rc::clone(&current_source);
        let current_control_ref = Rc::clone(&current_control_ref);
        let baseline_captured = Rc::clone(&baseline_captured);
        let get_value = Rc::clone(&get_value);
        let change = Rc::clone(&change);
        let invalid = invalid.clone();
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);

        Rc::new(move |source, registration| {
            match registration {
                Some(registration) => {
                    // A replaced control cancels the previous one's pending
                    // validation (`:150-153`) — only when the source differs (the
                    // in-place re-registration of the same control is not a
                    // replacement).
                    let is_replacement = current_source
                        .borrow()
                        .as_ref()
                        .is_some_and(|current| {
                            // `ControlIdSource` is an opaque u64 wrapper; the source
                            // identity is its display value.
                            current.to_string() != source.to_string()
                        });
                    if is_replacement {
                        change(None, true);
                    }
                    *current_source.borrow_mut() = Some(source.clone());

                    // The control's element slot rides the registration
                    // (`:113-120`'s `controlRef`).
                    let control_ref = registration.control_ref.clone();
                    current_control_ref.set(control_ref.borrow().clone());

                    // The name resolution (`:73`, `:111-119`): the root's name wins;
                    // the control's name is the recorded fallback.
                    let resolved_name = if name.is_some() {
                        name.clone()
                    } else {
                        registration.name.clone()
                    };
                    set_registered_field_name(if name.is_none() {
                        registration.name.clone()
                    } else {
                        None
                    });

                    // The field id (`:111`): the registered control id, recorded once
                    // per field.
                    let field_id = registration
                        .id
                        .clone()
                        .or_else(|| registered_field_id_ref.borrow().clone());
                    if let Some(field_id) = field_id.clone() {
                        *registered_field_id_ref.borrow_mut() = Some(field_id.clone());
                    }

                    // The initial-value baseline (`:86-103`): captured exactly once.
                    let initial_value = if baseline_captured.get() {
                        validity_data.get_untracked().initial_value
                    } else {
                        baseline_captured.set(true);
                        registration.value.clone().unwrap_or(Value::Null)
                    };

                    // The registry entry (`:65-78`).
                    let Some(field_id) = field_id else { return };
                    let entry = FormFieldEntry {
                        name: resolved_name,
                        validate: {
                            let commit = Rc::clone(&commit);
                            let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
                            let current_control_ref = Rc::clone(&current_control_ref);
                            Rc::new(move || {
                                marked_dirty_ref.set(true);
                                let value = current_control_ref
                                    .get()
                                    .and_then(|element| {
                                        element.dyn_into::<web_sys::HtmlInputElement>().ok()
                                    })
                                    .map(|input| Value::String(input.value()))
                                    .unwrap_or(Value::Null);
                                commit(value, false);
                            })
                        },
                        validity_data: validity_data.get_untracked(),
                        control_ref: control_ref.clone(),
                        get_value: Rc::clone(&get_value),
                    };
                    form_ref.borrow_mut().fields.set(field_id, entry);

                    // Publish the baseline into the validity record so the control's
                    // dirty comparisons see it (`:86-103`'s initialValue contract).
                    if validity_data.get_untracked().initial_value != initial_value {
                        validity_data.update(|data| data.initial_value = initial_value);
                    }
                    let _ = &invalid;
                }
                None => {
                    // Unregistration (`:122-131`): drop the registry entry keyed on
                    // the current field id and clear the control handover.
                    *current_source.borrow_mut() = None;
                    current_control_ref.set(None);
                    if let Some(field_id) = registered_field_id_ref.borrow().clone() {
                        form_ref.borrow_mut().fields.delete(&field_id);
                    }
                }
            }
        })
    };

    (validate, RootRegistrationReturn { validate, register })
}

// `Set` is used through `validity_data.update`; keep the import honest.
#[allow(unused)]
fn set_trait_marker<T>(_: impl Set<Value = T>) {}
