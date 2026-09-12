//! The validation machine — port of `useFieldValidation`
//! (`packages/react/src/field/root/useFieldValidation.ts`).
//!
//! The React hook instantiates once per `Field.Root` and shares itself with every
//! part through `FieldRootContext`. The port keeps that shape: one [`FieldValidation`]
//! value built by [`use_field_validation`] from the root's reactive state, holding the
//! whole machine — the input registry, the epoch-guarded `commit`, the debounce-aware
//! `change`, and `getValidationProps` (the sole source of `aria-invalid`).
//!
//! Rust adaptations (the React-hook-to-Leptos seams):
//!
//! - **Freshness.** Upstream's closures capture the render-fresh `state`/`invalid`
//!   through `useCallback` dep arrays; the port's bag is built once per root body, so
//!   the two captures that must stay fresh ride live signals instead: `state` is the
//!   root's [`FieldStateValue`] (derived `valid`/`disabled` — the revalidate gate and
//!   `getValidationProps` read them at call time) and `invalid` is a derived
//!   `Signal<bool>` read by the Form-registry update. The remaining captures
//!   (`markedDirtyRef`, `registeredFieldIdRef`, `submitCountRef`) are already
//!   interior-mutable cells upstream — the port shares them as-is.
//! - `validityData`/`setValidityData` collapse onto the root's one signal handle; the
//!   `commit` body reads it untracked where upstream reads the render-scoped snapshot
//!   (`:314`'s `validityData.state.customError` — the same last-published record, the
//!   functional-update-composition convention from `field_root_context.rs`).
//! - The async `commit` (`:123`, `Promise<void>`) splits: the synchronous prefix runs
//!   inline (native verdict, pending rules, sync validators), and a validator future
//!   is driven on the wasm microtask queue into the same finisher the sync arm uses —
//!   epoch-guarded, and a rejection publishes nothing (`:319-325`). In-unit callers
//!   never awaited the promise (the `field_root_context.rs` synchronous-commit note).
//! - `useTimeout` rides the ported [`leptos_ui_utils::use_timeout::Timeout`] (clones
//!   share the pending slot, so `commit` and `change` clear the same timer exactly as
//!   the upstream class instance did); the debounce arm starts only while validating
//!   on change and the value is non-empty (`:356-364`, implementation.md gap §3's
//!   unobserved bypass included).
//! - The native `ValidityState` read goes through the real DOM (`input.validity()`),
//!   so the field interoperates with native constraint validation instead of
//!   reimplementing it (implementation.md "The DOM is used as constraint-validation
//!   state").

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::GetUntracked;
use reactive_graph::wrappers::read::Signal;
use serde_json::Value;
use web_sys::HtmlInputElement;

use leptos_ui_internals::field_constants::{
    DEFAULT_VALIDITY_STATE, FieldValidityData, FieldValidityState,
};
use leptos_ui_internals::form_context::{FormContextValue, FormValidationMode};
use leptos_ui_internals::get_combined_field_validity_data::get_combined_field_validity_data;
use leptos_ui_utils::use_timeout::Timeout;

use crate::field::context::FieldStateValue;

// ---------------------------------------------------------------------------
// The registry vocabulary (`useFieldValidation.ts:18-23`)
// ---------------------------------------------------------------------------

/// `RegisteredInput` (`:18-21`).
#[derive(Clone)]
pub struct RegisteredInput {
    /// `controlRef` (`:19`) — the registered control's element slot.
    pub control_ref: Rc<Cell<Option<web_sys::Element>>>,
    /// `value` (`:20`) — `None` is the JS `undefined`.
    pub value: Option<String>,
}

/// `RegisteredInputs` (`:23`) — the insertion-ordered `Map` (the
/// `field_root_context.rs` vec convention): representative-input search follows
/// registration (mount) order (`findRepresentativeInput`, `:51-66`).
pub type RegisteredInputs = Rc<RefCell<Vec<(HtmlInputElement, RegisteredInput)>>>;

// ---------------------------------------------------------------------------
// The validator result union
// ---------------------------------------------------------------------------

/// The validator's result — upstream's
/// `string | string[] | null | void | Promise<...>` return (`:394-397`). The sync arms
/// carry the message(s); [`ValidationOutcome::Future`] defers to the spawned task.
pub enum ValidationOutcome {
    /// `null` / `undefined` / `''` / an empty array — the value is valid.
    Valid,
    /// A string or a string array — each non-empty entry is an error message.
    Invalid(Vec<String>),
    /// A thenable — resolved on the microtask queue, published under the epoch guard.
    Future(
        std::pin::Pin<Box<dyn std::future::Future<Output = ValidationOutcome> + 'static>>,
    ),
}

impl ValidationOutcome {
    /// The `validationErrors = result ? [].concat(result).filter(Boolean) : []`
    /// normalization (`:332`): empty strings drop out.
    pub fn into_errors(self) -> Vec<String> {
        match self {
            ValidationOutcome::Valid | ValidationOutcome::Future(_) => Vec::new(),
            ValidationOutcome::Invalid(errors) => {
                errors.into_iter().filter(|message| !message.is_empty()).collect()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The native-verdict helpers (`:25-74`, `:194-242`)
// ---------------------------------------------------------------------------

/// `isEligibleInput` (`:32-44`): `:disabled` excludes; otherwise the input belongs to
/// the surrounding `<Form>` unless an explicit `form` attribute opts it out — context
/// crosses portals, DOM position does not decide membership (implementation.md
/// "DOM/portal strategy").
pub fn is_eligible_input(
    input: &HtmlInputElement,
    form_element: Option<&web_sys::HtmlFormElement>,
) -> bool {
    if let Ok(matches) = input.matches(":disabled") {
        if matches {
            return false;
        }
    }
    match form_element {
        None => true,
        Some(form) => {
            let own_form = input.form();
            own_form.as_ref() == Some(form)
                || (own_form.is_none() && !input.has_attribute("form"))
        }
    }
}

/// `findRepresentativeInput` (`:51-66`): the first eligible currently-invalid input in
/// registration order, else the first eligible one.
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

/// `makeState` (`:68-70`): the barred-control synthetic state — all flags false,
/// `valid: !customError`, `customError`.
fn make_state(custom_error: bool) -> FieldValidityState {
    FieldValidityState { valid: Some(!custom_error), custom_error, ..DEFAULT_VALIDITY_STATE }
}

/// `getNativeErrors` (`:72-74`): the native validation message as a one-element list.
fn get_native_errors(element: Option<&HtmlInputElement>) -> Vec<String> {
    match element {
        Some(element) => {
            let message = element.validation_message();
            if message.is_empty() { Vec::new() } else { vec![message] }
        }
        None => Vec::new(),
    }
}

/// The `ValidityState` keys in `Object.keys(DEFAULT_VALIDITY_STATE)` order (`:16`) —
/// the `getState` reduction and the `revalidate` deferral walk this list. `valid` is
/// handled separately (the `:206-208` skip).
const VALIDITY_FLAG_READS: [(&str, fn(&FieldValidityState) -> bool); 10] = [
    ("badInput", |s| s.bad_input),
    ("customError", |s| s.custom_error),
    ("patternMismatch", |s| s.pattern_mismatch),
    ("rangeOverflow", |s| s.range_overflow),
    ("rangeUnderflow", |s| s.range_underflow),
    ("stepMismatch", |s| s.step_mismatch),
    ("tooLong", |s| s.too_long),
    ("tooShort", |s| s.too_short),
    ("typeMismatch", |s| s.type_mismatch),
    ("valueMissing", |s| s.value_missing),
];

/// `getState` (`:194-223`): copies the element's native flags, suppressing
/// `valueMissing` while the field is not dirty (the `hasOnlyValueMissingError`
/// reduction at `:203-221`).
fn get_state(element: &HtmlInputElement, marked_dirty: bool) -> FieldValidityState {
    let native = element.validity();
    let mut computed = FieldValidityState {
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
    for (key, read) in VALIDITY_FLAG_READS.iter() {
        let flag = read(&computed);
        if *key == "valueMissing" && flag {
            has_only_value_missing_error = true;
        } else if flag {
            return computed;
        }
    }

    // Only make `valueMissing` mark the field invalid if it's been changed to reduce
    // error noise (`:216-221`).
    if has_only_value_missing_error && !marked_dirty {
        computed.valid = Some(true);
        computed.value_missing = false;
    }
    computed
}

// ---------------------------------------------------------------------------
// The hook parameters and return
// ---------------------------------------------------------------------------

/// `UseFieldValidationParameters` (`:392-406`) — with the freshness adaptation from
/// the module docs: `state` and `invalid` are live signals, not snapshots.
pub struct UseFieldValidationParams {
    /// `validate` (`:394-397`) — the root's stable validator (default `() => null`).
    pub validate: Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>,
    /// `validityData` (`:398`) + `setValidityData` (`:393`) — one handle.
    pub validity_data: RwSignal<FieldValidityData>,
    /// `validationDebounceTime` (`:399`).
    pub validation_debounce_time: u32,
    /// `invalid` (`:400`) — live: read at registry-update time (the render-fresh
    /// capture, ported).
    pub invalid: Signal<bool>,
    /// `markedDirtyRef` (`:401`).
    pub marked_dirty_ref: Rc<Cell<bool>>,
    /// `state` (`:402`) — the root's derived state bag; the revalidate gate and
    /// `getValidationProps` read `valid`/`disabled` live (the render-fresh capture,
    /// ported).
    pub state: FieldStateValue,
    /// `shouldValidateOnChange` (`:403`).
    pub should_validate_on_change: Rc<dyn Fn() -> bool>,
    /// `validationMode` (`:404`) — the resolved mode (root prop ?? the Form's).
    pub validation_mode: FormValidationMode,
    /// `registeredFieldIdRef` (`:405`).
    pub registered_field_id_ref: Rc<RefCell<Option<String>>>,
}

/// `UseFieldValidationReturnValue` (`:408-416`) — the machine shared through context.
#[derive(Clone)]
pub struct FieldValidation {
    /// `getValidationProps` (`:409`) — appends the labelable description merge and
    /// `aria-invalid` onto the caller's `(name, lazy-value)` attribute bag (the
    /// `ValidationPropsFn` convention in `field_root_context.rs`).
    pub get_validation_props: Rc<
        dyn Fn(
            bool,
            &mut Vec<(String, leptos_ui_internals::floating_ui::element_props::ElementAttributeFn)>,
        ),
    >,
    /// `inputRef` (`:410`) — the fallback representative input when none are registered
    /// (`:226-232`).
    pub input_ref: Rc<Cell<Option<HtmlInputElement>>>,
    /// `registeredInputs` (`:411`).
    pub registered_inputs: RegisteredInputs,
    /// `registerInput` (`:412`) — sets the registration, returns the unregister closure.
    pub register_input: Rc<dyn Fn(&HtmlInputElement, RegisteredInput) -> Rc<dyn Fn()>>,
    /// `getInputControl` (`:413`).
    pub get_input_control: Rc<dyn Fn() -> Option<web_sys::HtmlElement>>,
    /// `commit` (`:414`) — the field's real state machine (the `revalidate = false`
    /// default arm; the root's registration hook drives the `true` arm through
    /// [`Self::commit_with_revalidate`]).
    pub commit: Rc<dyn Fn(Value)>,
    /// `commit`'s full signature (`:123`'s `async (value, revalidate = false)`) — the
    /// second entry point the root-side registration uses
    /// (`useFieldControlRegistration` calls `change(value, true)` and the imperative
    /// `validate` commits the fresh control value).
    pub commit_with_revalidate: Rc<dyn Fn(Value, bool)>,
    /// `change` (`:415`) — `(value, cancelPending)`; `cancelPending` defaults `false`.
    pub change: Rc<dyn Fn(Option<Value>, bool)>,
    /// The validity record handle — the machine and the context bag share one source
    /// of truth (the `validityData`/`setValidityData` collapse).
    pub validity_data: RwSignal<FieldValidityData>,
    /// The inert-shell marker: `true` only for the provider-less default bag — the
    /// required accessor's identity check (`FieldRootContext.ts:68`'s
    /// `setValidityData === NOOP` port).
    pub(crate) inert_marker: bool,
}

impl FieldValidation {
    /// The inert bag of the provider-less shell (`FieldRootContext.ts:52-60`): identity
    /// `getValidationProps`, a null `inputRef`, an empty registry, `NOOP` registration,
    /// `() => null` `getInputControl`, and inert `commit`/`change`.
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

// ---------------------------------------------------------------------------
// The hook
// ---------------------------------------------------------------------------

/// Port of `useFieldValidation` (`:76-390`). Must be called inside a reactive owner
/// (the debounce timeout's cleanup and the async publication register there).
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

    // `const { elementRef, formRef } = useFormContext()` (`:79`).
    let FormContextValue { element_ref, form_ref, .. } =
        leptos_ui_internals::form_context::use_form_context();

    // `const { controlId, getDescriptionProps } = useLabelableContext()` (`:94`).
    let labelable = leptos_ui_internals::labelable_provider::use_labelable_context();
    let control_id = labelable.control_id.clone();
    let get_description_props = labelable.get_description_props.clone();

    // `const timeout = useTimeout()` (`:96`) — one shared instance; clones share the
    // pending slot, so `commit`'s `timeout.clear()` and `change`'s clear the same
    // timer.
    let timeout = Timeout::create();

    // `const inputRef = React.useRef<HTMLInputElement | null>(null)` (`:97`).
    let input_ref: Rc<Cell<Option<HtmlInputElement>>> = Rc::new(Cell::new(None));

    // `const registeredInputs = useRefWithInit(() => new Map()).current` (`:98`).
    let registered_inputs: RegisteredInputs = Rc::new(RefCell::new(Vec::new()));

    // `const validationCommitIdRef = React.useRef(0)` (`:99`).
    let validation_commit_id: Rc<Cell<u32>> = Rc::new(Cell::new(0));

    // The custom-validity ownership record (`:101-103`):
    // `[element, message, displaced]`.
    let custom_validity: Rc<RefCell<Option<(HtmlInputElement, String, String)>>> =
        Rc::new(RefCell::new(None));

    // `registerInput` (`:108-116`).
    let register_input: Rc<dyn Fn(&HtmlInputElement, RegisteredInput) -> Rc<dyn Fn()>> = {
        let registered_inputs = Rc::clone(&registered_inputs);
        Rc::new(move |element, registration| {
            registered_inputs.borrow_mut().push((element.clone(), registration));
            let registered_inputs = Rc::clone(&registered_inputs);
            let element = element.clone();
            Rc::new(move || {
                registered_inputs
                    .borrow_mut()
                    .retain(|(registered, _)| *registered != element);
            })
        })
    };

    // `getInputControl` (`:118-121`): the representative input's registered control
    // element.
    let get_input_control: Rc<dyn Fn() -> Option<web_sys::HtmlElement>> = {
        let registered_inputs = Rc::clone(&registered_inputs);
        let element_ref = element_ref.clone();
        Rc::new(move || {
            let form_element = element_ref.borrow().clone();
            let representative =
                find_representative_input(&registered_inputs, form_element.as_ref());
            let registry = registered_inputs.borrow();
            representative.and_then(|element| {
                registry
                    .iter()
                    .find(|(registered, _)| *registered == element)
                    .and_then(|(_, registration)| registration.control_ref.borrow().clone())
                    .and_then(|control| control.dyn_into::<web_sys::HtmlElement>().ok())
            })
        })
    };

    // `resolveRepresentativeInput` (`:228-232`) — shared by the commit body and the
    // async arm.
    let resolve_representative = {
        let registered_inputs = Rc::clone(&registered_inputs);
        let input_ref = Rc::clone(&input_ref);
        let element_ref = element_ref.clone();
        move || {
            let form_element = element_ref.borrow().clone();
            if !registered_inputs.borrow().is_empty() {
                find_representative_input(&registered_inputs, form_element.as_ref())
            } else {
                input_ref.get()
            }
        }
    };

    // `const commit = useStableCallback(async (value, revalidate = false) => { … })`
    // (`:123-347`) — the full signature; the context's `commit` is the default-arm
    // wrapper.
    let commit_impl: Rc<dyn Fn(Value, bool)> = {
        let validity_data = validity_data.clone();
        let registered_inputs = Rc::clone(&registered_inputs);
        let validation_commit_id = Rc::clone(&validation_commit_id);
        let custom_validity = Rc::clone(&custom_validity);
        let control_id = control_id.clone();
        let element_ref = element_ref.clone();
        let form_ref = Rc::clone(&form_ref);
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        let should_validate_on_change = Rc::clone(&should_validate_on_change);
        let validate = Rc::clone(&validate);
        let resolve_representative = resolve_representative.clone();
        let timeout = timeout.clone();

        Rc::new(move |value: Value, revalidate: bool| {
            // `validationCommitIdRef.current += 1` (`:124-125`).
            validation_commit_id.set(validation_commit_id.get() + 1);
            let commit_id = validation_commit_id.get();

            // `updateRegisteredFieldValidity` (`:127-150`): the combined record into
            // the Form registry; `external_invalid: None` is the `= invalid` default,
            // read live (the freshness adaptation).
            let update_registered_field_validity = {
                let form_ref = Rc::clone(&form_ref);
                let registered_field_id_ref = Rc::clone(&registered_field_id_ref);
                let control_id = control_id.clone();
                let invalid = invalid.clone();
                move |next_validity_data: &FieldValidityData,
                      external_invalid: Option<bool>|
                      -> bool {
                    let field_id = registered_field_id_ref
                        .borrow()
                        .clone()
                        .or_else(|| control_id.get_untracked());
                    let Some(field_id) = field_id else { return false };
                    let external_invalid = external_invalid.unwrap_or_else(|| invalid.get_untracked());
                    let combined = get_combined_field_validity_data(
                        next_validity_data,
                        external_invalid,
                    );
                    let mut form_state = form_ref.borrow_mut();
                    match form_state.fields.get(&field_id) {
                        Some(entry) => {
                            let mut entry = entry.clone();
                            entry.validity_data = combined;
                            form_state.fields.set(field_id, entry);
                            true
                        }
                        None => false,
                    }
                }
            };

            // `makeValidityData` (`:152-165`): `errors` stay empty unless the state is
            // `valid === false` — "`valueMissing` may be suppressed while the native
            // message remains non-empty" (`:157`).
            let make_validity_data = {
                let value = value.clone();
                let initial_value = validity_data.get_untracked().initial_value;
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

            // `setCustomValidity` (`:167-173`): never reinstall a native message as
            // custom validity; `\r\n`/`\r` normalize to `\n` (`:170`).
            let set_custom_validity = {
                let custom_validity = Rc::clone(&custom_validity);
                move |element: &HtmlInputElement, message: &str| {
                    let displaced = if element.validity().custom_error() {
                        element.validation_message()
                    } else {
                        String::new()
                    };
                    let owned_message = message.replace("\r\n", "\n").replace('\r', "\n");
                    let _ = element.set_custom_validity(&owned_message);
                    *custom_validity.borrow_mut() =
                        Some((element.clone(), owned_message, displaced));
                }
            };

            // `clearCustomValidity` (`:175-182`): replacement transfers ownership;
            // barred controls hide `validationMessage`.
            let clear_custom_validity = {
                let custom_validity = Rc::clone(&custom_validity);
                move || {
                    let record = custom_validity.borrow_mut().take();
                    if let Some((element, owned_message, displaced)) = record {
                        let restore = !element.will_validate()
                            || element.validation_message() == owned_message;
                        if restore {
                            let _ = element.set_custom_validity(&displaced);
                        }
                    }
                }
            };

            // The post-validator finisher (`:334-346`): custom errors mark the state
            // and ride the real input (barred controls keep them in field state only);
            // otherwise the (possibly custom-cleared) native errors publish. Shared by
            // the sync arm and the async arm.
            let finish_with_validator_result = {
                let make_validity_data = make_validity_data.clone();
                let update_registered_field_validity =
                    update_registered_field_validity.clone();
                let set_custom_validity = set_custom_validity.clone();
                let validity_data = validity_data.clone();
                move |mut next_state: FieldValidityState,
                      element: Option<HtmlInputElement>,
                      errors: Vec<String>| {
                    let mut errors = errors;
                    if !errors.is_empty() {
                        next_state.valid = Some(false);
                        next_state.custom_error = true;
                        if let Some(element) = &element {
                            if element.will_validate() {
                                set_custom_validity(element, &errors.join("\n"));
                            }
                        }
                    } else {
                        errors = get_native_errors(element.as_ref());
                    }
                    let next = make_validity_data(next_state, errors);
                    update_registered_field_validity(&next, None);
                    validity_data.set(next);
                }
            };

            // `publish` (`:184-192`).
            let publish = {
                let validity_data = validity_data.clone();
                let make_validity_data = make_validity_data.clone();
                let update_registered_field_validity =
                    update_registered_field_validity.clone();
                move |validity_state: FieldValidityState,
                      error_messages: Vec<String>,
                      external_invalid: Option<bool>| {
                    let next = make_validity_data(validity_state, error_messages);
                    update_registered_field_validity(&next, external_invalid);
                    validity_data.set(next);
                }
            };

            // `refreshState` (`:238-242`) — re-resolves the representative element (the
            // inner `element =` assignment mutates the enclosing binding) and reads the
            // native verdict, barred controls included.
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

            // `:236` — the initial representative.
            let mut element = resolve_representative();

            // The `revalidate` branch (`:244-276`).
            if revalidate {
                // `state.valid !== false || !element` — the combined validity, read
                // live (the freshness adaptation).
                if state.valid.get_untracked() != Some(false) || element.is_none() {
                    return;
                }

                let value_missing_resolved = element
                    .as_ref()
                    .is_some_and(|element| !element.validity().value_missing());
                if value_missing_resolved {
                    // The required condition has been resolved by the user typing.
                    // Temporarily mark the field as valid for this change event; other
                    // native errors are caught by full validation on blur or submit
                    // (`:250-259`). Clearing can make another registered input with a
                    // custom error representative.
                    clear_custom_validity();
                    let current_element = resolve_representative();
                    let foreign = current_element
                        .as_ref()
                        .filter(|element| element.validity().custom_error())
                        .map(|element| get_native_errors(Some(element)))
                        .unwrap_or_default();
                    // `publish(makeState(foreign.length > 0), foreign, false)` — the
                    // explicit `false` ignores stale external invalid state.
                    publish(
                        make_state(!foreign.is_empty()),
                        foreign,
                        Some(false),
                    );
                    return;
                }

                // A stale custom error can coexist with valueMissing, but defer any
                // other native errors (`:262-272`).
                if let Some(current) = &element {
                    for (key, _) in VALIDITY_FLAG_READS.iter() {
                        if *key == "customError" {
                            continue;
                        }
                        let flag = match *key {
                            "badInput" => current.validity().bad_input(),
                            "patternMismatch" => current.validity().pattern_mismatch(),
                            "rangeOverflow" => current.validity().range_overflow(),
                            "rangeUnderflow" => current.validity().range_underflow(),
                            "stepMismatch" => current.validity().step_mismatch(),
                            "tooLong" => current.validity().too_long(),
                            "tooShort" => current.validity().too_short(),
                            "typeMismatch" => current.validity().type_mismatch(),
                            "valueMissing" => current.validity().value_missing(),
                            _ => false,
                        };
                        if *key != "valueMissing" && flag {
                            return;
                        }
                    }
                }
                // Value is still missing: publish the current native state so
                // valueMissing and the changed value are observable immediately. Full
                // custom validation still waits for its boundary (`:274-276`).
            }

            // `timeout.clear()` (`:278`).
            timeout.clear();

            // Do not read Base UI's previous message back as a native constraint
            // (`:281`).
            clear_custom_validity();

            let mut next_state = refresh_state(&mut element);
            let mut validation_errors = get_native_errors(element.as_ref());

            let is_validating_on_change = should_validate_on_change();

            // Native or externally set errors take precedence outside onChange
            // validation (`:288-289`).
            if validation_errors.is_empty() || is_validating_on_change {
                // `formValues` projection (`:293-298`): every named registered field's
                // live value.
                let form_values: serde_json::Map<String, Value> = {
                    let form_state = form_ref.borrow();
                    let mut values = serde_json::Map::new();
                    for (_, entry) in form_state.fields.iter() {
                        if let Some(name) = &entry.name {
                            values.insert(name.clone(), (entry.get_value)());
                        }
                    }
                    values
                };

                let result = validate(&value, &form_values);
                match result {
                    ValidationOutcome::Future(future) => {
                        // The pending rules (`:308-317`): validity is unknown while the
                        // validator runs, so go neutral — but keep what must block
                        // submission synchronously: fresh native failures always, and a
                        // previous custom error outside onSubmit mode. A previous
                        // native error is never kept (`nextState` already carries the
                        // fresh native verdict).
                        if next_state.valid == Some(false) {
                            publish(next_state.clone(), validation_errors.clone(), None);
                        } else if validation_mode == FormValidationMode::OnSubmit
                            || !validity_data.get_untracked().state.custom_error
                        {
                            next_state.valid = None;
                            publish(next_state.clone(), validation_errors.clone(), None);
                        }

                        // `result = await resultOrPromise` (`:321`) driven on the
                        // microtask queue; the rejected-validator rule (`:319-325`) — a
                        // rejection publishes nothing, so the previously published
                        // state stays and keeps blocking submission. The arm re-enters
                        // the shared finisher with the refreshed state (`:326`).
                        let validity_data = validity_data.clone();
                        let finish_with_validator_result = finish_with_validator_result.clone();
                        let validation_commit_id = Rc::clone(&validation_commit_id);
                        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
                        let registered_inputs = Rc::clone(&registered_inputs);
                        let resolve_representative = resolve_representative.clone();
                        let element_ref = element_ref.clone();
                        let committed_id = commit_id;
                        let _ = &element_ref;
                        wasm_bindgen_futures::spawn_local(async move {
                            let outcome = future.await;
                            // `:323-325` — the epoch guard.
                            if validation_commit_id.get() != committed_id {
                                return;
                            }
                            let errors = outcome.into_errors();
                            // `nextState = refreshState()` (`:326`) — re-resolve the
                            // representative and re-read the native verdict.
                            let element = resolve_representative();
                            let next_state = match &element {
                                Some(element) if element.will_validate() => {
                                    get_state(element, marked_dirty_ref.get())
                                }
                                _ => make_state(false),
                            };
                            let _ = &registered_inputs;
                            finish_with_validator_result(next_state, element, errors);
                        });
                        return;
                    }
                    outcome => {
                        let errors = outcome.into_errors();
                        finish_with_validator_result(
                            next_state.clone(),
                            element.clone(),
                            errors,
                        );
                        return;
                    }
                }
            }

            // `:346` — the final publication of the native short-circuit path.
            let next = make_validity_data(
                next_state,
                validation_errors,
            );
            update_registered_field_validity(&next, None);
            validity_data.set(next);
        })
    };

    // The context-facing default arm (`:414`).
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
                // `:358-361` — debounce only while validating on change and the value
                // is non-empty (gap §3's bypass: clearing validates immediately).
                let commit_impl = Rc::clone(&commit_impl);
                timeout.start(validation_debounce_time, move || {
                    commit_impl(value, false);
                });
            } else {
                // `commit(value, !validateOnChange)` (`:363`).
                commit_impl(value, !validate_on_change);
            }
        })
    };

    // `getValidationProps` (`:367-376`) — `state.valid`/`state.disabled` read live
    // (the freshness adaptation).
    let get_validation_props = {
        let state = state.clone();
        let get_description_props = Rc::clone(&get_description_props);
        Rc::new(
            move |disabled: bool,
                  external: &mut Vec<
                (String, leptos_ui_internals::floating_ui::element_props::ElementAttributeFn),
            >| {
                get_description_props(external);
                if state.valid.get_untracked() == Some(false)
                    && !state.disabled.get_untracked()
                    && !disabled
                {
                    external.push((
                        "aria-invalid".to_string(),
                        Rc::new(|| Some("true".to_string()))
                            as leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
                    ));
                }
            },
        )
    };

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
