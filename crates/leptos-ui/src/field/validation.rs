//! The validation machine — re-homed port of `useFieldValidation`
//! (`packages/react/src/field/root/useFieldValidation.ts`) on leptos-owned signals.
//!
//! Runtime note (the accordion/meter precedent): the internals crate's reactive
//! modules run on reactive-graph 0.2, which cannot drive leptos 0.7's view tree; the
//! reactive logic re-homes into the component crate on leptos signals. The internals
//! crate keeps the view-independent vocabulary this module consumes: the record
//! shapes (`DEFAULT_VALIDITY_STATE`, `FieldValidityData`, `FieldValidityState`), the
//! `get_combined_field_validity_data` combination, and the form-context default (the
//! inert `FormRef` registry the machine projects `formValues` from).
//!
//! Rust trait-import rule this module obeys (the dual-runtime law the compiler
//! taught): leptos signal traits come from `leptos::prelude::*` (named imports of
//! rg-0.2's traits would shadow the glob — the two runtimes' `Get`/`Set` are
//! different traits with the same names); the internals' rg-0.2 handles are read via
//! fully-qualified calls only (`reactive_graph::traits::GetUntracked::get_untracked`).
//!
//! Behavior contract (implementation.md "The validation engine"):
//! - Epoch guard: every `change`/`commit` bumps the counter; an awaited async result
//!   whose epoch is stale is discarded (`useFieldValidation.ts:99,124-125,323-325`).
//! - `change` clears the timer, honors cancellation, and either debounces `commit`
//!   (validating on change **and** non-empty value — the gap-§3 bypass included) or
//!   commits immediately with `revalidate = !validateOnChange` (`:349-365`).
//! - The `revalidate` branch publishes a resolved `valueMissing` immediately while
//!   deferring other native errors to the next blur/submit (`:244-276`).
//! - Native verdicts suppress `valueMissing` while the field is not dirty
//!   (`:194-223`); barred controls (`willValidate === false`) get the synthetic
//!   all-false state (`:238-242`).
//! - Custom-validity ownership: the field's message is written to the real input via
//!   `setCustomValidity`, remembering the displaced message and restoring it only
//!   when the control still shows the field's own (or is barred) (`:167-182`).
//! - Async pending rules: neutral while in flight except fresh native failures and a
//!   previous custom error outside onSubmit mode; a rejected validator publishes
//!   nothing (`:303-325`).
//! - `getValidationProps` is the only source of `aria-invalid` (`:367-376`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use leptos_ui_internals::field_constants::{
    DEFAULT_VALIDITY_STATE, FieldValidityData, FieldValidityState,
};
use leptos_ui_internals::form_context::{FormContextValue, FormValidationMode};
use leptos_ui_internals::get_combined_field_validity_data::get_combined_field_validity_data;
use leptos_ui_utils::use_timeout::Timeout;

use crate::field::context::FieldStateValue;

/// `RegisteredInput` (`useFieldValidation.ts:18-21`).
#[derive(Clone)]
pub struct RegisteredInput {
    /// `controlRef` (`:19`).
    pub control_ref: Rc<Cell<Option<web_sys::Element>>>,
    /// `value` (`:20`) — `None` is the JS `undefined`.
    pub value: Option<String>,
}

/// `RegisteredInputs` (`:23`) — insertion order is registration (mount) order, the
/// representative-input search order (`findRepresentativeInput`, `:51-66`).
pub type RegisteredInputs = Rc<RefCell<Vec<(HtmlInputElement, RegisteredInput)>>>;

/// The validator's result — upstream's
/// `string | string[] | null | void | Promise<...>` (`:394-397`).
pub enum ValidationOutcome {
    /// `null` / `undefined` / `''` / an empty array — valid.
    Valid,
    /// Message(s); empty strings drop out (`:332`'s `filter(Boolean)`).
    Invalid(Vec<String>),
    /// A thenable — driven on the wasm microtask queue under the epoch guard.
    Future(std::pin::Pin<Box<dyn std::future::Future<Output = ValidationOutcome>>>),
}

impl ValidationOutcome {
    /// `result ? [].concat(result).filter(Boolean) : []` (`:332`).
    pub fn into_errors(self) -> Vec<String> {
        match self {
            ValidationOutcome::Valid | ValidationOutcome::Future(_) => Vec::new(),
            ValidationOutcome::Invalid(errors) => errors
                .into_iter()
                .filter(|message| !message.is_empty())
                .collect(),
        }
    }
}

/// The lazy attribute-value closure (`ElementAttributeFn`'s shape).
pub type AttributeFn = Rc<dyn Fn() -> Option<String>>;

/// Reads a `Cell<Option<T>>` slot — the take/replace-back pattern the internals use
/// for the non-`Copy` element slots (`field_register_control.rs`'s
/// `control_ref.replace(None)` dance). Crate-visible: the control consumes it (the
/// machine's `inputRef`/`controlRef` slots are module-scoped types).
pub(crate) fn cell_peek<T: Clone>(cell: &Cell<Option<T>>) -> Option<T> {
    let carried = cell.replace(None);
    cell.set(carried.clone());
    carried
}

/// `isEligibleInput` (`:32-44`): `:disabled` excludes; otherwise the input
/// participates unless an explicit `form` attribute associates it elsewhere —
/// context crosses portals, DOM position does not decide membership.
pub fn is_eligible_input(
    input: &HtmlInputElement,
    form_element: Option<&web_sys::HtmlFormElement>,
) -> bool {
    if input.matches(":disabled").unwrap_or(false) {
        return false;
    }
    match form_element {
        None => true,
        Some(form) => {
            let own_form = input.form();
            own_form.as_ref() == Some(form) || (own_form.is_none() && !input.has_attribute("form"))
        }
    }
}

/// `findRepresentativeInput` (`:51-66`).
fn find_representative_input(
    inputs: &RegisteredInputs,
    form_element: Option<&web_sys::HtmlFormElement>,
) -> Option<HtmlInputElement> {
    let registry = inputs.borrow();
    let mut fallback: Option<HtmlInputElement> = None;
    for (input, _) in registry.iter() {
        if !is_eligible_input(input, form_element) {
            continue;
        }
        if !input.validity().valid() {
            return Some(input.clone());
        }
        if fallback.is_none() {
            fallback = Some(input.clone());
        }
    }
    fallback
}

/// `makeState` (`:68-70`).
fn make_state(custom_error: bool) -> FieldValidityState {
    FieldValidityState {
        valid: Some(!custom_error),
        custom_error,
        ..DEFAULT_VALIDITY_STATE
    }
}

/// `getNativeErrors` (`:72-74`).
fn get_native_errors(element: Option<&HtmlInputElement>) -> Vec<String> {
    match element {
        Some(element) => match element.validation_message() {
            Ok(message) if message.is_empty() => Vec::new(),
            Ok(message) => vec![message],
            Err(_) => Vec::new(),
        },
        None => Vec::new(),
    }
}

/// `getState` (`:194-223`): copies the native flags, suppressing `valueMissing`
/// while the field is not dirty. The flag walk follows `Object.keys` order minus
/// `valid` (`:205-214`): the first non-`valueMissing` flag short-circuits.
fn get_state(element: &HtmlInputElement, marked_dirty: bool) -> FieldValidityState {
    let native = element.validity();
    let computed = FieldValidityState {
        bad_input: native.bad_input(),
        custom_error: native.custom_error(),
        pattern_mismatch: native.pattern_mismatch(),
        range_overflow: native.range_overflow(),
        range_underflow: native.range_underflow(),
        step_mismatch: native.step_mismatch(),
        too_long: native.too_long(),
        too_short: native.too_short(),
        type_mismatch: native.type_mismatch(),
        value_missing: native.value_missing(),
        valid: Some(native.valid()),
    };

    let mut has_only_value_missing_error = false;
    let flags = [
        ("badInput", computed.bad_input),
        ("customError", computed.custom_error),
        ("patternMismatch", computed.pattern_mismatch),
        ("rangeOverflow", computed.range_overflow),
        ("rangeUnderflow", computed.range_underflow),
        ("stepMismatch", computed.step_mismatch),
        ("tooLong", computed.too_long),
        ("tooShort", computed.too_short),
        ("typeMismatch", computed.type_mismatch),
        ("valueMissing", computed.value_missing),
    ];
    for (key, flag) in flags.iter() {
        if *key == "valueMissing" && *flag {
            has_only_value_missing_error = true;
        } else if *flag {
            return computed;
        }
    }

    // Only make `valueMissing` mark the field invalid if it's been changed
    // (`:216-221`).
    let mut computed = computed;
    if has_only_value_missing_error && !marked_dirty {
        computed.valid = Some(true);
        computed.value_missing = false;
    }
    computed
}

/// `UseFieldValidationParameters` (`:392-406`). The freshness adaptation: upstream's
/// closures re-read render-fresh `state`/`invalid` through dep arrays; the port's bag
/// is built once per root body, so those two ride live signals.
pub struct UseFieldValidationParams {
    /// `validate` (`:394-397`) — the root's stable validator.
    pub validate: Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>,
    /// `validityData` (`:398`) + `setValidityData` (`:393`) — one handle.
    pub validity_data: RwSignal<FieldValidityData>,
    /// `validationDebounceTime` (`:399`).
    pub validation_debounce_time: u32,
    /// `invalid` (`:400`) — live.
    pub invalid: Signal<bool>,
    /// `markedDirtyRef` (`:401`).
    pub marked_dirty_ref: Rc<Cell<bool>>,
    /// `state` (`:402`) — the derived bag; `valid`/`disabled` read live.
    pub state: FieldStateValue,
    /// `shouldValidateOnChange` (`:403`).
    pub should_validate_on_change: Rc<dyn Fn() -> bool>,
    /// `validationMode` (`:404`).
    pub validation_mode: FormValidationMode,
    /// `registeredFieldIdRef` (`:405`).
    pub registered_field_id_ref: Rc<RefCell<Option<String>>>,
}

/// `UseFieldValidationReturnValue` (`:408-416`) — the machine shared through context.
#[derive(Clone)]
pub struct FieldValidation {
    /// `getValidationProps` (`:409`) — merges the labelable description props and
    /// `aria-invalid` onto the caller's attribute vec.
    pub get_validation_props: Rc<dyn Fn(bool, &mut Vec<(String, AttributeFn)>)>,
    /// `inputRef` (`:410`) — the fallback representative input.
    pub input_ref: Rc<Cell<Option<HtmlInputElement>>>,
    /// `registeredInputs` (`:411`).
    pub registered_inputs: RegisteredInputs,
    /// `registerInput` (`:412`).
    pub register_input: Rc<dyn Fn(&HtmlInputElement, RegisteredInput) -> Rc<dyn Fn()>>,
    /// `getInputControl` (`:413`).
    pub get_input_control: Rc<dyn Fn() -> Option<web_sys::HtmlElement>>,
    /// `commit` (`:414`, the `revalidate = false` default arm).
    pub commit: Rc<dyn Fn(Value)>,
    /// `commit`'s full signature (`:123`).
    pub commit_with_revalidate: Rc<dyn Fn(Value, bool)>,
    /// `change` (`:415`).
    pub change: Rc<dyn Fn(Option<Value>, bool)>,
    /// The shared validity record handle (the `validityData`/`setValidityData`
    /// collapse).
    pub validity_data: RwSignal<FieldValidityData>,
    /// The inert-shell marker (the required accessor's identity check).
    pub(crate) inert_marker: bool,
}

impl FieldValidation {
    /// The inert bag of the provider-less shell (`FieldRootContext.ts:52-60`).
    pub fn inert() -> Self {
        FieldValidation {
            get_validation_props: Rc::new(|_, _| {}),
            input_ref: Rc::new(Cell::new(None)),
            registered_inputs: Rc::new(RefCell::new(Vec::new())),
            register_input: Rc::new(|_, _| Rc::new(|| {})),
            get_input_control: Rc::new(|| None),
            commit: Rc::new(|_| {}),
            commit_with_revalidate: Rc::new(|_, _| {}),
            change: Rc::new(|_, _| {}),
            validity_data: RwSignal::new(FieldValidityData::default()),
            inert_marker: true,
        }
    }

    /// The shell-identity check the required accessor rides.
    pub fn is_inert(&self) -> bool {
        self.inert_marker
    }
}

/// Port of `useFieldValidation` (`:76-390`). Must be called inside a reactive owner.
pub fn use_field_validation(params: UseFieldValidationParams) -> FieldValidation {
    let UseFieldValidationParams {
        validate,
        validity_data,
        validation_debounce_time,
        invalid,
        marked_dirty_ref,
        state,
        should_validate_on_change,
        validation_mode,
        registered_field_id_ref,
    } = params;

    // `const { elementRef, formRef } = useFormContext()` (`:79`) — the inert default
    // outside a `<Form>`.
    let FormContextValue {
        element_ref,
        form_ref,
        ..
    } = leptos_ui_internals::form_context::use_form_context();

    // `useLabelableContext()` (`:94`) — called inside the root's labelable scope (the
    // bridge window), so the ids resolve against the real provider; the reads are
    // callback-time (untracked, fully qualified on the rg-0.2 handle), the only
    // cross-runtime shape this port allows.
    let labelable = leptos_ui_internals::labelable_provider::use_labelable_context();
    let control_id = labelable.control_id.clone();
    let get_description_props = labelable.get_description_props.clone();

    let timeout = Timeout::create();
    let input_ref: Rc<Cell<Option<HtmlInputElement>>> = Rc::new(Cell::new(None));
    let registered_inputs: RegisteredInputs = Rc::new(RefCell::new(Vec::new()));
    let validation_commit_id: Rc<Cell<u32>> = Rc::new(Cell::new(0));
    let custom_validity: Rc<RefCell<Option<(HtmlInputElement, String, String)>>> =
        Rc::new(RefCell::new(None));

    // `registerInput` (`:108-116`).
    let register_input: Rc<dyn Fn(&HtmlInputElement, RegisteredInput) -> Rc<dyn Fn()>> = {
        let registered_inputs = Rc::clone(&registered_inputs);
        Rc::new(move |element, registration| {
            registered_inputs
                .borrow_mut()
                .push((element.clone(), registration));
            let registered_inputs = Rc::clone(&registered_inputs);
            let element = element.clone();
            Rc::new(move || {
                registered_inputs
                    .borrow_mut()
                    .retain(|(registered, _)| *registered != element);
            })
        })
    };

    // `getInputControl` (`:118-121`).
    let get_input_control: Rc<dyn Fn() -> Option<web_sys::HtmlElement>> = {
        let registered_inputs = Rc::clone(&registered_inputs);
        let element_ref = element_ref.clone();
        Rc::new(move || {
            let form_element = cell_peek(&element_ref);
            let representative =
                find_representative_input(&registered_inputs, form_element.as_ref());
            let registry = registered_inputs.borrow();
            representative.and_then(|element| {
                registry
                    .iter()
                    .find(|(registered, _)| *registered == element)
                    .and_then(|(_, registration)| cell_peek(&registration.control_ref))
                    .and_then(|control| control.dyn_into::<web_sys::HtmlElement>().ok())
            })
        })
    };

    // `resolveRepresentativeInput` (`:228-232`).
    let resolve_representative = {
        let registered_inputs = Rc::clone(&registered_inputs);
        let input_ref = Rc::clone(&input_ref);
        let element_ref = element_ref.clone();
        move || {
            let form_element = cell_peek(&element_ref);
            if !registered_inputs.borrow().is_empty() {
                find_representative_input(&registered_inputs, form_element.as_ref())
            } else {
                cell_peek(&input_ref)
            }
        }
    };

    // `commit` (`:123-347`).
    let commit_impl: Rc<dyn Fn(Value, bool)> = {
        let validity_data = validity_data.clone();
        let validation_commit_id = Rc::clone(&validation_commit_id);
        let custom_validity = Rc::clone(&custom_validity);
        let control_id = control_id.clone();
        let form_ref = Rc::clone(&form_ref);
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        let should_validate_on_change = Rc::clone(&should_validate_on_change);
        let validate = Rc::clone(&validate);
        let resolve_representative = resolve_representative.clone();
        let timeout = timeout.clone();

        Rc::new(move |value: Value, revalidate: bool| {
            validation_commit_id.set(validation_commit_id.get() + 1);
            let commit_id = validation_commit_id.get();

            // `updateRegisteredFieldValidity` (`:127-150`); the `= invalid` default
            // reads live.
            let update_registered = {
                let form_ref = Rc::clone(&form_ref);
                let registered_field_id_ref = Rc::clone(&registered_field_id_ref);
                let control_id = control_id.clone();
                let invalid = invalid.clone();
                move |next: &FieldValidityData, external_invalid: Option<bool>| {
                    let field_id = registered_field_id_ref.borrow().clone().or_else(|| {
                        reactive_graph::traits::GetUntracked::get_untracked(&control_id)
                    });
                    let Some(field_id) = field_id else { return };
                    let external_invalid =
                        external_invalid.unwrap_or_else(|| GetUntracked::get_untracked(&invalid));
                    let combined = get_combined_field_validity_data(next, external_invalid);
                    let mut form_state = form_ref.borrow_mut();
                    if let Some(entry) = form_state.fields.get(&field_id) {
                        let mut entry = entry.clone();
                        entry.validity_data = combined;
                        form_state.fields.set(field_id, entry);
                    }
                }
            };

            // `makeValidityData` (`:152-165`).
            let make_validity_data = {
                let value = value.clone();
                let initial_value = GetUntracked::get_untracked(&validity_data).initial_value;
                move |validity_state: FieldValidityState,
                      error_messages: Vec<String>|
                      -> FieldValidityData {
                    let errors = if validity_state.valid == Some(false) {
                        error_messages
                    } else {
                        Vec::new()
                    };
                    FieldValidityData {
                        value: value.clone(),
                        state: validity_state,
                        error: errors.first().cloned().unwrap_or_default(),
                        errors,
                        initial_value: initial_value.clone(),
                    }
                }
            };

            // `setCustomValidity` (`:167-173`).
            let set_custom_validity = {
                let custom_validity = Rc::clone(&custom_validity);
                move |element: &HtmlInputElement, message: &str| {
                    let displaced = if element.validity().custom_error() {
                        element.validation_message().unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let owned_message = message.replace("\r\n", "\n").replace('\r', "\n");
                    let _ = element.set_custom_validity(&owned_message);
                    *custom_validity.borrow_mut() =
                        Some((element.clone(), owned_message, displaced));
                }
            };

            // `clearCustomValidity` (`:175-182`).
            let clear_custom_validity = {
                let custom_validity = Rc::clone(&custom_validity);
                move || {
                    let record = custom_validity.borrow_mut().take();
                    if let Some((element, owned_message, displaced)) = record {
                        let restore = !element.will_validate()
                            || element.validation_message().unwrap_or_default() == owned_message;
                        if restore {
                            let _ = element.set_custom_validity(&displaced);
                        }
                    }
                }
            };

            // `publish` (`:184-192`).
            let publish = {
                let validity_data = validity_data.clone();
                let make_validity_data = make_validity_data.clone();
                let update_registered = update_registered.clone();
                move |validity_state: FieldValidityState,
                      error_messages: Vec<String>,
                      external_invalid: Option<bool>| {
                    let next = make_validity_data(validity_state, error_messages);
                    update_registered(&next, external_invalid);
                    validity_data.set(next);
                }
            };

            // `refreshState` (`:238-242`).
            let refresh_state = {
                let resolve_representative = resolve_representative.clone();
                let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
                move |element: &mut Option<HtmlInputElement>| -> FieldValidityState {
                    *element = resolve_representative();
                    match element {
                        Some(element) if element.will_validate() => {
                            get_state(element, marked_dirty_ref.get())
                        }
                        _ => make_state(false),
                    }
                }
            };

            // The post-validator finisher (`:334-346`).
            let finish = {
                let make_validity_data = make_validity_data.clone();
                let update_registered = update_registered.clone();
                let set_custom_validity = set_custom_validity.clone();
                let validity_data = validity_data.clone();
                move |mut next_state: FieldValidityState,
                      element: Option<HtmlInputElement>,
                      mut errors: Vec<String>| {
                    if !errors.is_empty() {
                        next_state.valid = Some(false);
                        next_state.custom_error = true;
                        // Keep custom errors for barred controls in field state only
                        // (`:337-340`).
                        if let Some(element) = &element {
                            if element.will_validate() {
                                set_custom_validity(element, &errors.join("\n"));
                            }
                        }
                    } else {
                        errors = get_native_errors(element.as_ref());
                    }
                    let next = make_validity_data(next_state, errors);
                    update_registered(&next, None);
                    validity_data.set(next);
                }
            };

            let mut element = resolve_representative();

            // The `revalidate` branch (`:244-276`).
            if revalidate {
                // `state.valid !== false || !element` (`:245`) — combined validity,
                // read live.
                if GetUntracked::get_untracked(&state.valid) != Some(false) || element.is_none() {
                    return;
                }

                // `!element.validity.valueMissing` (`:249`).
                if element
                    .as_ref()
                    .is_some_and(|e| !e.validity().value_missing())
                {
                    clear_custom_validity();
                    let current_element = resolve_representative();
                    let foreign = current_element
                        .as_ref()
                        .filter(|e| e.validity().custom_error())
                        .map(|e| get_native_errors(Some(e)))
                        .unwrap_or_default();
                    publish(make_state(!foreign.is_empty()), foreign, Some(false));
                    return;
                }

                // A stale custom error can coexist with valueMissing; defer any other
                // native errors (`:262-272`).
                if let Some(current) = &element {
                    let native = current.validity();
                    let deferred = native.bad_input()
                        || native.pattern_mismatch()
                        || native.range_overflow()
                        || native.range_underflow()
                        || native.step_mismatch()
                        || native.too_long()
                        || native.too_short()
                        || native.type_mismatch();
                    if deferred {
                        return;
                    }
                }
                // Value is still missing: publish the current native state
                // (`:274-276`).
            }

            timeout.clear();
            clear_custom_validity();

            let mut next_state = refresh_state(&mut element);
            let validation_errors = get_native_errors(element.as_ref());

            let is_validating_on_change = should_validate_on_change();

            // Native or externally set errors take precedence outside onChange
            // validation (`:288-289`).
            if validation_errors.is_empty() || is_validating_on_change {
                // `formValues` (`:293-298`).
                let form_values: serde_json::Map<String, Value> = {
                    let form_state = form_ref.borrow();
                    let mut values = serde_json::Map::new();
                    for (_, entry) in form_state.fields.iter() {
                        if let Some(name) = &entry.name {
                            values.insert(name.clone(), (entry.get_value)().unwrap_or(Value::Null));
                        }
                    }
                    values
                };

                match validate(&value, &form_values) {
                    ValidationOutcome::Future(future) => {
                        // The pending rules (`:308-317`).
                        if next_state.valid == Some(false) {
                            publish(next_state.clone(), validation_errors.clone(), None);
                        } else if validation_mode == FormValidationMode::OnSubmit
                            || !GetUntracked::get_untracked(&validity_data)
                                .state
                                .custom_error
                        {
                            next_state.valid = None;
                            publish(next_state.clone(), validation_errors.clone(), None);
                        }

                        // `await` + the epoch guard (`:321-326`); a rejection
                        // publishes nothing (`:319-325`).
                        let validity_data = validity_data.clone();
                        let finish = finish.clone();
                        let validation_commit_id = Rc::clone(&validation_commit_id);
                        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
                        let resolve_representative = resolve_representative.clone();
                        let committed_id = commit_id;
                        wasm_bindgen_futures::spawn_local(async move {
                            let outcome = future.await;
                            if validation_commit_id.get() != committed_id {
                                return;
                            }
                            let errors = outcome.into_errors();
                            let mut element = None;
                            let next_state = {
                                let resolved = resolve_representative();
                                element = resolved;
                                match &element {
                                    Some(element) if element.will_validate() => {
                                        get_state(element, marked_dirty_ref.get())
                                    }
                                    _ => make_state(false),
                                }
                            };
                            finish(next_state, element, errors);
                        });
                        return;
                    }
                    outcome => {
                        finish(next_state.clone(), element.clone(), outcome.into_errors());
                        return;
                    }
                }
            }

            // `:346`.
            let next = make_validity_data(next_state, validation_errors);
            update_registered(&next, None);
            validity_data.set(next);
        })
    };

    let commit: Rc<dyn Fn(Value)> = {
        let commit_impl = Rc::clone(&commit_impl);
        Rc::new(move |value| commit_impl(value, false))
    };

    // `change` (`:349-365`).
    let change: Rc<dyn Fn(Option<Value>, bool)> = {
        let timeout = timeout.clone();
        let validation_commit_id = Rc::clone(&validation_commit_id);
        let should_validate_on_change = Rc::clone(&should_validate_on_change);
        let commit_impl = Rc::clone(&commit_impl);
        Rc::new(move |value, cancel_pending| {
            timeout.clear();
            validation_commit_id.set(validation_commit_id.get() + 1);
            if cancel_pending {
                return;
            }
            let Some(value) = value else { return };
            let validate_on_change = should_validate_on_change();
            let is_empty_string = value == Value::String(String::new());
            if validate_on_change && !is_empty_string && validation_debounce_time > 0 {
                let commit_impl = Rc::clone(&commit_impl);
                timeout.start(validation_debounce_time, move || {
                    commit_impl(value, false);
                });
            } else {
                commit_impl(value, !validate_on_change);
            }
        })
    };

    // `getValidationProps` (`:367-376`).
    let get_validation_props: Rc<dyn Fn(bool, &mut Vec<(String, AttributeFn)>)> = {
        let state = state.clone();
        let get_description_props = Rc::clone(&get_description_props);
        Rc::new(
            move |disabled: bool, external: &mut Vec<(String, AttributeFn)>| {
                get_description_props(external);
                if GetUntracked::get_untracked(&state.valid) == Some(false)
                    && !GetUntracked::get_untracked(&state.disabled)
                    && !disabled
                {
                    external.push((
                        "aria-invalid".to_string(),
                        Rc::new(|| Some("true".to_string())) as AttributeFn,
                    ));
                }
            },
        )
    };

    // The machine's reads of the labelable scope are callback-time (untracked); no
    // rg-0.2 signal ever feeds a view here.
    let _ = &control_id;

    FieldValidation {
        get_validation_props,
        input_ref,
        registered_inputs,
        register_input,
        get_input_control,
        commit,
        commit_with_revalidate: commit_impl,
        change,
        validity_data,
        inert_marker: false,
    }
}
