//! Port of `packages/react/src/internals/field-root-context/FieldRootContext.ts` — the
//! context bag `Field.Root` provides and the field parts consume
//! (`specs/library/internals/implementation.md`, "Context providers/consumers" — the
//! `FieldRootContext` row: the provider is `Field.Root`, not this unit; the in-unit
//! consumer is [`crate::field_register_control::use_register_field_control`] reading
//! `registerFieldControl`. Untested upstream per "Anything in source not explained by any
//! test" item 4, so the tests below pin the written mechanics per the PrehydrationScript
//! precedent).
//!
//! The context default (`FieldRootContext.ts:32-61`) is an all-`NOOP` shell so the field
//! hooks work outside a `<Field.Root>` (`:65-75`'s accessor only panics when
//! `optional === false`), and `useFieldRootContext` detects that shell by identity —
//! `context.setValidityData === NOOP`.
//!
//! ## Rust adaptations
//!
//! - React context ports to reactive-graph's owner-scoped [`use_context`] behind the
//!   [`SharedFieldRootContext`] `SendWrapper` bridge (the `composite_list.rs` precedent).
//!   This unit defines the bag, the default shell, and the accessors; the provide call
//!   site is the Phase B `Field.Root` component (the provider is not in this unit).
//! - The `SetStateAction` setters (`setValidityData`, `setTouched`, `setDirty`,
//!   `setFilled`, `setFocused`) port to the [`RwSignal`] handles themselves — a React
//!   state setter is a write (the `labelable_provider.rs` convention); functional updates
//!   compose `get_untracked` + conditional `set` at the call site (React's same-value
//!   bail-out reproduced as the inequality guard, the `use_transition_status.rs`
//!   precedent). `validityData`/`setValidityData` collapse onto one
//!   `validity_data: RwSignal<FieldValidityData>` handle: tracked reads are the value,
//!   writes are the setter.
//! - The shell-identity check (`:68`, `setValidityData === NOOP`) ports to an
//!   `Rc::ptr_eq` against the shared no-op `registerFieldControl` the default shell hands
//!   out — the `labelable_provider.rs` `NOOP_REGISTER_CONTROL_ID` precedent. The port's
//!   `setValidityData` is a signal handle with no `NOOP` identity, and both members are
//!   `NOOP` exactly in the default shell (every provider supplies real implementations),
//!   so the registration fn carries the identity check.
//! - The default shell is built fresh per provider-less access inside the caller's owner
//!   (the `labelable_provider.rs` precedent — owner-registered reactive storage must not
//!   live in statics, the `prehydration_script.rs` hazard), with the exception of the
//!   shared no-op registration fn the identity check needs.
//! - `validation: UseFieldValidationReturnValue`
//!   (`packages/react/src/field/root/useFieldValidation.ts:408-416`) ports to
//!   [`FieldValidationBag`]: the return-value vocabulary ported ahead of the Phase B
//!   `useFieldValidation` hook (the stringifyLocale-over-formatNumber precedent — the
//!   default shell constructs it, `FieldRootContext.ts:52-60`). `getValidationProps`
//!   merges over the crate's `(name, lazy-value)` attribute bag
//!   (`use_render_element.rs`/`labelable_provider.rs` convention) instead of returning an
//!   `HTMLProps` record, and `commit`'s async wrapper (`:414`, `Promise<void>`) collapses
//!   to a synchronous call: in-unit callers never await it and the port's state writes
//!   are synchronous (the `use_animations_finished.rs` flushSync note).
//! - `invalid`/`name`/`disabled` (`boolean | undefined` / `string | undefined`) port to
//!   `Signal<Option<…>>` reads; `state: FieldRootState` — upstream a fresh `useMemo`
//!   object per render (`FieldRoot.tsx:124-134`) — ports to a derived signal over the
//!   member handles, so the bag stays fresh the way React's re-render did.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::wrappers::read::Signal;
use reactive_graph::wrappers::read::Signal as ReadSignal;
use send_wrapper::SendWrapper;
use serde_json::Value;
use web_sys::HtmlInputElement;

use crate::field_constants::{
    DEFAULT_FIELD_ROOT_STATE, DEFAULT_VALIDITY_STATE, FieldRootState, FieldValidityData,
};
use crate::field_register_control::FieldControlRegistration;
use crate::floating_ui::element_props::ElementAttributeFn;
use crate::form_context::FormValidationMode;
use crate::labelable_provider::ControlIdSource;

// ---------------------------------------------------------------------------
// The validation return-value vocabulary (useFieldValidation.ts:18-23, :408-416)
// ---------------------------------------------------------------------------

/// `RegisteredInput` (`useFieldValidation.ts:18-21`).
#[derive(Clone)]
pub struct RegisteredInput {
    /// `controlRef` (`:19`).
    pub control_ref: Rc<Cell<Option<web_sys::Element>>>,
    /// `value` (`:20` — `string | undefined`; `None` is `undefined`).
    pub value: Option<String>,
}

/// `RegisteredInputs` (`useFieldValidation.ts:23`) — `Map<HTMLInputElement,
/// RegisteredInput>` keyed by element identity in registration (mount) order: the
/// representative-input search follows insertion order (`findRepresentativeInput`,
/// `useFieldValidation.ts:60-72`). The insertion-ordered vec reproduces the `Map`
/// semantics (the `labelable_provider.rs` precedent).
pub type RegisteredInputs = Rc<RefCell<Vec<(HtmlInputElement, RegisteredInput)>>>;

/// `getValidationProps` (`useFieldValidation.ts:367-378` over `:409`) — merges the
/// labelable description props and the `aria-invalid` flag into the external attribute
/// bag; `disabled` is the control-side disabled flag the caller passes.
pub type ValidationPropsFn = Rc<dyn Fn(bool, &mut Vec<(String, ElementAttributeFn)>)>;

/// `registerInput` (`useFieldValidation.ts:108-114` over `:412`): sets the registration
/// and returns the unregister closure (the upstream `void | (() => void)` union — the
/// implementation always returns the deleter, so the port does too).
pub type RegisterInputFn = Rc<dyn Fn(&HtmlInputElement, RegisteredInput) -> Rc<dyn Fn()>>;

/// `getInputControl` (`useFieldValidation.ts:118-121` over `:413`).
pub type GetInputControlFn = Rc<dyn Fn() -> Option<web_sys::HtmlElement>>;

/// `commit` (`useFieldValidation.ts:414`) — synchronous in the port (module docs).
pub type FieldCommitFn = Rc<dyn Fn(Value)>;

/// `change` (`useFieldValidation.ts:415`) — `(value, cancelPending)`, where `None` is the
/// JS `undefined` value and `cancelPending` defaults `false`.
pub type FieldChangeFn = Rc<dyn Fn(Option<Value>, bool)>;

/// The `validation` member's type — upstream `UseFieldValidationReturnValue`
/// (`useFieldValidation.ts:408-416`), ported ahead of the Phase B hook (module docs).
#[derive(Clone)]
pub struct FieldValidationBag {
    /// `getValidationProps` (`:409`).
    pub get_validation_props: ValidationPropsFn,
    /// `inputRef` (`:410`) — the fallback representative input when none are registered
    /// (`useFieldValidation.ts:226-231`).
    pub input_ref: Rc<Cell<Option<HtmlInputElement>>>,
    /// `registeredInputs` (`:411`).
    pub registered_inputs: RegisteredInputs,
    /// `registerInput` (`:412`).
    pub register_input: RegisterInputFn,
    /// `getInputControl` (`:413`).
    pub get_input_control: GetInputControlFn,
    /// `commit` (`:414`).
    pub commit: FieldCommitFn,
    /// `change` (`:415`).
    pub change: FieldChangeFn,
}

// ---------------------------------------------------------------------------
// The context bag (FieldRootContext.ts:12-61)
// ---------------------------------------------------------------------------

/// The registration call — upstream `registerFieldControl(source, registration)`
/// (`FieldRootContext.ts:25-28`); `None` is the `undefined` registration (unregister).
pub type RegisterFieldControlFn = Rc<dyn Fn(ControlIdSource, Option<FieldControlRegistration>)>;

/// The context bag — upstream's `FieldRootContext` (`FieldRootContext.ts:12-30`).
#[derive(Clone)]
pub struct FieldRootContextValue {
    /// `invalid` (`:13`).
    pub invalid: Signal<Option<bool>, LocalStorage>,
    /// `name` (`:14`).
    pub name: Signal<Option<String>, LocalStorage>,
    /// `validityData` + `setValidityData` (`:15-16`) — one reactive handle (module docs).
    pub validity_data: RwSignal<FieldValidityData, LocalStorage>,
    /// `disabled` (`:17`).
    pub disabled: Signal<Option<bool>, LocalStorage>,
    /// `setTouched` (`:18`).
    pub set_touched: RwSignal<bool, LocalStorage>,
    /// `setDirty` (`:19`).
    pub set_dirty: RwSignal<bool, LocalStorage>,
    /// `setFilled` (`:20`).
    pub set_filled: RwSignal<bool, LocalStorage>,
    /// `setFocused` (`:21`).
    pub set_focused: RwSignal<bool, LocalStorage>,
    /// `validationMode` (`:22`).
    pub validation_mode: FormValidationMode,
    /// `shouldValidateOnChange` (`:23`).
    pub should_validate_on_change: Rc<dyn Fn() -> bool>,
    /// `state` (`:24`) — the derived field state (module docs).
    pub state: ReadSignal<FieldRootState, LocalStorage>,
    /// `registerFieldControl` (`:25-28`).
    pub register_field_control: RegisterFieldControlFn,
    /// `validation` (`:29`).
    pub validation: FieldValidationBag,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `composite_list.rs`
/// precedent). The provide call site is the Phase B `Field.Root` component.
pub type SharedFieldRootContext = SendWrapper<FieldRootContextValue>;

thread_local! {
    /// The no-op registration the default shell hands out — shared so the shell-identity
    /// check (`FieldRootContext.ts:68`'s `setValidityData === NOOP`) ports to an
    /// `Rc::ptr_eq` against this instance (module docs).
    static NOOP_REGISTER_FIELD_CONTROL: RegisterFieldControlFn = Rc::new(|_, _| {});
}

/// Whether `register_field_control` is the default shell's no-op — upstream's
/// `setValidityData === NOOP` shell check (`FieldRootContext.ts:68`; module docs).
fn is_noop_register_field_control(register: &RegisterFieldControlFn) -> bool {
    NOOP_REGISTER_FIELD_CONTROL.with(|noop| Rc::ptr_eq(register, noop))
}

/// The default `validation` bag (`FieldRootContext.ts:52-60`): identity
/// `getValidationProps`, a null `inputRef`, an empty `registeredInputs` map, `NOOP`
/// `registerInput`, `() => null` `getInputControl`, and inert `commit`/`change`. Built
/// fresh per access (module docs).
fn default_validation_bag() -> FieldValidationBag {
    FieldValidationBag {
        get_validation_props: Rc::new(|_, _| {}),
        input_ref: Rc::new(Cell::new(None)),
        registered_inputs: Rc::new(RefCell::new(Vec::new())),
        register_input: Rc::new(|_, _| Rc::new(|| {})),
        get_input_control: Rc::new(|| None),
        commit: Rc::new(|_| {}),
        change: Rc::new(|_, _| {}),
    }
}

/// Upstream's default context value (`FieldRootContext.ts:32-61`): `undefined`
/// `invalid`/`name`/`disabled`, the default validity data (`:35-41`), `NOOP` setters and
/// registration, `validationMode: 'onSubmit'`, `() => false`
/// `shouldValidateOnChange`, the default state, and the inert validation bag. Built fresh
/// per provider-less access inside the caller's owner (module docs).
fn default_field_root_context() -> FieldRootContextValue {
    let set_touched: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
    let set_dirty: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
    let set_filled: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
    let set_focused: RwSignal<bool, LocalStorage> = RwSignal::new_local(false);
    let validity_data: RwSignal<FieldValidityData, LocalStorage> =
        RwSignal::new_local(FieldValidityData {
            state: DEFAULT_VALIDITY_STATE,
            errors: Vec::new(),
            error: String::new(),
            value: Value::Null,
            initial_value: Value::Null,
        });
    let disabled: Signal<Option<bool>, LocalStorage> = Signal::derive_local(|| None);

    // `state: DEFAULT_FIELD_ROOT_STATE` (`:50`) — the literal constant: the shell's
    // members are inert, so the derived read never observes a change. The bag itself
    // holds the member handles alive (module docs).
    let state: ReadSignal<FieldRootState, LocalStorage> =
        ReadSignal::derive_local(|| DEFAULT_FIELD_ROOT_STATE);

    FieldRootContextValue {
        invalid: Signal::derive_local(|| None),
        name: Signal::derive_local(|| None),
        validity_data,
        disabled,
        set_touched,
        set_dirty,
        set_filled,
        set_focused,
        validation_mode: FormValidationMode::OnSubmit,
        should_validate_on_change: Rc::new(|| false),
        state,
        register_field_control: NOOP_REGISTER_FIELD_CONTROL.with(Rc::clone),
        validation: default_validation_bag(),
    }
}

/// The optional accessor — upstream's `useFieldRootContext()` (`FieldRootContext.ts:65-75`
/// with the default `optional = true`), falling back to the default shell outside a
/// provider. Must be called inside a reactive owner (a component), like the other hook
/// ports.
pub fn use_field_root_context() -> FieldRootContextValue {
    use_context::<SharedFieldRootContext>()
        .map(|shared| (*shared).clone())
        .unwrap_or_else(default_field_root_context)
}

/// The required accessor — upstream's `useFieldRootContext(false)`
/// (`FieldRootContext.ts:68-72`), panicking with the upstream message when only the
/// default shell is in scope.
pub fn use_field_root_context_required() -> FieldRootContextValue {
    let context = use_field_root_context();
    if is_noop_register_field_control(&context.register_field_control) {
        panic!(
            "Base UI: FieldRootContext is missing. Field parts must be placed within <Field.Root>."
        );
    }
    context
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::field_register_control::FieldControlRegistration;
    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{GetUntracked, Set};

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn registration(id: &str) -> FieldControlRegistration {
        FieldControlRegistration {
            control_ref: Rc::new(Cell::new(None)),
            id: Some(id.to_string()),
            name: None,
            get_value: None,
            value: None,
        }
    }

    // Pins the default shell (`FieldRootContext.ts:32-61`): the no-op registration keeps
    // a stray control inert, and the accessor falls back to the shell outside a provider.
    #[test]
    fn the_default_shell_keeps_a_stray_control_inert() {
        let owner = in_owner();

        let context = use_field_root_context();
        assert!(is_noop_register_field_control(
            &context.register_field_control
        ));
        assert_eq!(context.invalid.get_untracked(), None);
        assert_eq!(context.name.get_untracked(), None);
        assert_eq!(context.disabled.get_untracked(), None);
        assert_eq!(
            context.validity_data.get_untracked().state,
            DEFAULT_VALIDITY_STATE
        );
        assert_eq!(context.state.get_untracked(), DEFAULT_FIELD_ROOT_STATE);
        assert_eq!(context.validation_mode, FormValidationMode::OnSubmit);
        assert!(!(context.should_validate_on_change)(), "() => false");

        (context.register_field_control)(ControlIdSource::new(), Some(registration("a")));
        assert!(
            is_noop_register_field_control(&context.register_field_control),
            "the no-op registration leaves the shell untouched"
        );

        owner.cleanup();
    }

    // Pins the required accessor's panic (`FieldRootContext.ts:68-72`): the default shell
    // trips the missing-provider error with the upstream message (the
    // `use_composite_root.rs` panic-assertion precedent).
    #[test]
    fn the_required_accessor_panics_on_the_default_shell() {
        let result = std::panic::catch_unwind(|| {
            let owner = in_owner();
            use_field_root_context_required();
            owner.cleanup();
        });
        let message = result
            .err()
            .and_then(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            })
            .unwrap_or_default();
        assert!(
            message.starts_with("Base UI: FieldRootContext is missing."),
            "the missing-provider error reads {message:?}"
        );
    }

    // Pins the identity-detection mechanism (module docs): a real provider's registration
    // fn is not the shared no-op, so the required accessor passes through.
    #[test]
    fn a_provided_registration_fn_is_not_the_noop() {
        let owner = in_owner();

        let register: RegisterFieldControlFn = Rc::new(|_, _| {});
        assert!(!is_noop_register_field_control(&register));

        owner.cleanup();
    }

    // Pins the inert validation bag (`FieldRootContext.ts:52-60`): the shell's
    // registeredInputs map stays empty (registerInput is NOOP), the inputRef stays null,
    // and get_input_control resolves to nothing. The registerInput arm runs in the wasm
    // suite against a real element.
    #[test]
    fn the_default_validation_bag_is_inert() {
        let owner = in_owner();

        let context = use_field_root_context();
        assert!(context.validation.registered_inputs.borrow().is_empty());
        assert!(context.validation.input_ref.take().is_none());
        assert!((context.validation.get_input_control)().is_none());
        // The commit/change defaults are inert calls.
        (context.validation.commit)(Value::Null);
        (context.validation.change)(None, false);

        owner.cleanup();
    }

    // Pins the setters-as-signal-handles convention (module docs): a write through the
    // handle is observable through the bag — the shape the field parts' consumers build
    // on.
    #[test]
    fn the_setter_handles_are_writable() {
        let owner = in_owner();

        let context = use_field_root_context();
        context.set_touched.set(true);
        context.set_focused.set(true);
        assert!(context.set_touched.get_untracked());
        assert!(context.set_focused.get_untracked());

        context.validity_data.set(FieldValidityData::default());
        assert_eq!(
            context.validity_data.get_untracked(),
            FieldValidityData::default()
        );

        owner.cleanup();
    }
}
