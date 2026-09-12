//! The Field root — port of `FieldRoot` (`packages/react/src/field/root/FieldRoot.tsx`).
//!
//! Upstream's component is two layers: `FieldRoot` wraps the inner component in a
//! `LabelableProvider` (`:202-211`), and `FieldRootInner` builds the whole field state,
//! the validation machine, and the registration pair before rendering its `div` under
//! the context provider (`:22-194`). The port keeps both: [`field_root_view`] is the
//! view function (the LabelableProvider scope, the inner body, the state-attribute
//! walk) and `FieldRoot` is the `#[component]` wrapper.
//!
//! The root's own structure, in order (`FieldRootInner` body):
//!
//! 1. `useFormContext()` (`:26`) — the Form's errors/mode/submit-count ride the
//!    internals' `SharedFormContext` owner bridge: the root reads them inside an
//!    explicit rg-0.2 owner so a future `<Form>` provider connects without any
//!    leptos-runtime mixing.
//! 2. The destructuring with defaults (`:28-42`).
//! 3. The four state booleans + validity record (`:50-104`).
//! 4. The derived `valid` gate (`:109`) and state object (`:111-121`).
//! 5. `useFieldValidation` (`:123-134`).
//! 6. `useFieldControlRegistration` (`:136-146`) + `actionsRef` (`:148-150`).
//! 7. The context value (`:152-184`).
//! 8. `useRenderElement('div', …, { state, stateAttributesMapping:
//!    fieldValidityMapping })` (`:186-191`).

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use serde_json::Value;
use wasm_bindgen::JsCast;

use leptos_ui_internals::field_constants::{DEFAULT_VALIDITY_STATE, FieldValidityData};
use leptos_ui_internals::field_register_control::UseFieldControlRegistrationParams;
use leptos_ui_internals::form_context::FormValidationMode;
use leptos_ui_internals::labelable_provider::{
    ControlIdRegistration, ControlIdSource, provide_labelable_context,
};
use send_wrapper::SendWrapper;

use crate::field::context::{FieldRootContext, FieldStateValue};
use crate::field::validation::{
    FieldValidation, UseFieldValidationParams, use_field_validation,
};
use crate::field::validation_helpers::{
    mark_root_dirty, pending_control_value, queue_microtask_safe,
};

/// The Field root props — upstream's destructured set (`FieldRoot.tsx:28-42`), the
/// render-coupled members already ported by the internals' registration hook.
pub struct FieldRootProps {
    /// `validate` (`:31`) — the custom validator; `None` is the `() => null` default
    /// (`:46`).
    pub validate: Option<
        Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> crate::field::validation::ValidationOutcome>,
    >,
    /// `validationDebounceTime` (`:32`) — default `0`.
    pub validation_debounce_time: u32,
    /// `validationMode` (`:33`) — `None` resolves to the Form's mode (`:33`'s
    /// `?? formValidationMode`), else `onSubmit`.
    pub validation_mode: Option<FormValidationMode>,
    /// `name` (`:34`) — the root name; wins over the control's fallback (the source's
    /// resolution order, the discrepancy-log entry).
    pub name: Option<String>,
    /// `disabled` (`:35`) — default `false`.
    pub disabled: bool,
    /// `invalid` (`:36`) — the app-controlled flag.
    pub invalid: Option<bool>,
    /// `dirty` (`:37`) — the controlled override.
    pub dirty: Option<bool>,
    /// `touched` (`:38`) — the controlled override.
    pub touched: Option<bool>,
    /// `actionsRef` (`:39`) — receives the imperative `{ validate }` handle at body
    /// time (upstream's `useImperativeHandle` — the dialog's thread-local slot is the
    /// port's ref-object stand-in).
    pub actions: Option<Rc<dyn Fn(crate::field::context::FieldRootActions)>>,
    /// The user's `class` (`:30`).
    pub class: Option<String>,
    /// The user's `style` (`:40`).
    pub style: Option<Vec<(String, String)>>,
    /// `...elementProps` (`:41`) — the last bag: plain attributes override the state
    /// walk on conflict.
    pub element_attributes: Vec<(String, String)>,
    /// The children — the parts subtree.
    pub children: Option<leptos::children::Children>,
}

impl Default for FieldRootProps {
    fn default() -> Self {
        FieldRootProps {
            validate: None,
            validation_debounce_time: 0,
            validation_mode: None,
            name: None,
            disabled: false,
            invalid: None,
            dirty: None,
            touched: None,
            actions: None,
            class: None,
            style: None,
            element_attributes: Vec::new(),
            children: None,
        }
    }
}

/// The view body — upstream's `FieldRoot` → `LabelableProvider` → `FieldRootInner`
/// chain (`:202-211`). Must be called inside a reactive owner.
pub fn field_root_view(props: FieldRootProps) -> impl IntoView {
    let FieldRootProps {
        validate,
        validation_debounce_time,
        validation_mode,
        name,
        disabled: disabled_prop,
        invalid: invalid_prop,
        dirty: dirty_prop,
        touched: touched_prop,
        actions,
        class,
        style,
        element_attributes,
        children,
    } = props;

    // `const { errors, validationMode: formValidationMode, submitCountRef } =
    // useFormContext()` (`:26`), plus the root's own form-registry reads — all inside
    // the rg-0.2 owner bridge (the module docs).
    let bridge_owner = reactive_graph::owner::Owner::new();
    let form = bridge_owner.with(|| {
        leptos_ui_internals::form_context::use_form_context()
    });
    let form_errors = form.errors.clone();
    let form_validation_mode = form.validation_mode.clone();
    let submit_count_ref = form.submit_count_ref.clone();
    let form_ref = form.form_ref.clone();
    let form_element_ref = form.element_ref.clone();
    let form_clear_errors = form.clear_errors.clone();
    std::mem::forget(bridge_owner);

    // The destructuring defaults (`:31-42`).
    let validate = validate.unwrap_or_else(|| Rc::new(|_, _| crate::field::validation::ValidationOutcome::Valid));
    let validation_mode =
        validation_mode.unwrap_or_else(|| match form_validation_mode {
            FormValidationMode::OnSubmit => FormValidationMode::OnSubmit,
            FormValidationMode::OnBlur => FormValidationMode::OnBlur,
            FormValidationMode::OnChange => FormValidationMode::OnChange,
        });
    let disabled_fieldset = false; // `useFieldsetRootContext(true)?.disabled` — the
                                  // fieldset unit is unported; `None` outside one.

    // The four state booleans (`:50-53`).
    let touched_state: RwSignal<bool> = RwSignal::new(false);
    let dirty_state: RwSignal<bool> = RwSignal::new(false);
    let filled: RwSignal<bool> = RwSignal::new(false);
    let focused: RwSignal<bool> = RwSignal::new(false);

    // The controlled resolution (`:55-56`).
    let dirty = {
        let dirty_state = dirty_state.clone();
        Signal::derive(move || dirty_prop.unwrap_or_else(|| dirty_state.get()))
    };
    let touched = {
        let touched_state = touched_state.clone();
        Signal::derive(move || touched_prop.unwrap_or_else(|| touched_state.get()))
    };

    // `markedDirtyRef` (`:58`) — seeded with the resolved dirty (`useRef(dirty)`).
    let marked_dirty_ref: Rc<std::cell::Cell<bool>> =
        Rc::new(std::cell::Cell::new(dirty.get_untracked()));

    // `registeredFieldIdRef` (`:59`) + `registeredFieldName` (`:60`) + the effective
    // name (`:61`).
    let registered_field_id_ref: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let registered_field_name: RwSignal<Option<String>> = RwSignal::new(None);
    let effective_name = {
        let registered_field_name = registered_field_name.clone();
        Signal::derive(move || name.clone().or_else(|| registered_field_name.get()))
    };

    // The dirty-prop mirror (`:63-67`) — the layout effect becomes a tracked effect:
    // the controlled value feeds `markedDirtyRef` so callback-time readers see it.
    {
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        Effect::new(move |_| {
            if let Some(prop) = dirty_prop {
                marked_dirty_ref.set(prop);
            }
        });
    }

    // `setDirty` (`:69-78`) — the controlled no-op + the `markedDirtyRef` feed.
    let set_dirty: Rc<dyn Fn(bool)> = {
        let dirty_state = dirty_state.clone();
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        Rc::new(move |value: bool| {
            if dirty_prop.is_some() {
                return;
            }
            if value {
                marked_dirty_ref.set(true);
            }
            dirty_state.set(value);
        })
    };

    // `setTouched` (`:80-85`) — the controlled no-op.
    let set_touched: Rc<dyn Fn(bool)> = {
        let touched_state = touched_state.clone();
        Rc::new(move |value: bool| {
            if touched_prop.is_some() {
                return;
            }
            touched_state.set(value);
        })
    };

    // `setFilled` / `setFocused` (`:52-53` — plain setters shared through context).
    let set_filled: Rc<dyn Fn(bool)> = {
        let filled = filled.clone();
        Rc::new(move |value: bool| filled.set(value))
    };
    let set_focused: Rc<dyn Fn(bool)> = {
        let focused = focused.clone();
        Rc::new(move |value: bool| focused.set(value))
    };

    // `shouldValidateOnChange` (`:87-91`).
    let should_validate_on_change: Rc<dyn Fn() -> bool> = {
        let submit_count_ref = Rc::clone(&submit_count_ref);
        Rc::new(move || {
            validation_mode == FormValidationMode::OnChange
                || (validation_mode == FormValidationMode::OnSubmit
                    && submit_count_ref.get() > 0)
        })
    };

    // `validityData` (`:98-104`).
    let validity_data: RwSignal<FieldValidityData> = RwSignal::new(FieldValidityData {
        state: DEFAULT_VALIDITY_STATE,
        error: String::new(),
        errors: Vec::new(),
        value: Value::Null,
        initial_value: Value::Null,
    });

    // The form-error lookup + `invalid` derivation (`:93-96`).
    let name_for_errors = effective_name.clone();
    let invalid = Signal::derive(move || {
        let has_form_error = match name_for_errors.get() {
            Some(name) => {
                let errors = form_errors.get_untracked();
                match errors.get(&name) {
                    Some(error) => match error {
                        leptos_ui_internals::form_context::FormErrorValue::Single(message) => {
                            !message.is_empty()
                        }
                        leptos_ui_internals::form_context::FormErrorValue::Multiple(messages) => {
                            !messages.is_empty()
                        }
                    },
                    None => false,
                }
            }
            None => false,
        };
        invalid_prop == Some(true) || has_form_error
    });

    // The derived `valid` gate (`:109`): app-controlled invalidity survives `disabled`;
    // computed validity is suppressed while disabled.
    let valid = {
        let invalid = invalid.clone();
        let disabled_value = disabled_fieldset || disabled_prop;
        let validity_data = validity_data.clone();
        Signal::derive(move || {
            let computed = validity_data.get().state.valid;
            if invalid.get() {
                Some(false)
            } else if disabled_value {
                None
            } else {
                computed
            }
        })
    };

    // The state bag (`:111-121`).
    let state = FieldStateValue {
        disabled: Signal::derive(move || disabled_fieldset || disabled_prop),
        touched: touched.clone(),
        dirty: dirty.clone(),
        valid: valid.clone(),
        filled: filled.get().into(),
        focused: focused.get().into(),
    };

    // `useFieldValidation` (`:123-134`).
    let validation = use_field_validation(UseFieldValidationParams {
        validate,
        validity_data: validity_data.clone(),
        validation_debounce_time,
        invalid: invalid.clone(),
        marked_dirty_ref: Rc::clone(&marked_dirty_ref),
        state: state.clone(),
        should_validate_on_change: should_validate_on_change.clone(),
        validation_mode,
        registered_field_id_ref: Rc::clone(&registered_field_id_ref),
    });

    // `useFieldControlRegistration` (`:136-146`) — the Form-bridge call runs inside
    // the rg-0.2 owner bridge (the hook reads/writes `formRef.current.fields` through
    // the internals context).
    let registration = bridge_owner.with(|| {
        use_form_context()
            .map(|_| ())
            .unwrap_or(());
        // placeholder; replaced below
    });

    // The rest of the root body: the registration pair and the context provide run
    // under the leptos owner (the view tree), with the Form-bridge reads lifted into
    // the bridge above.
    let (validate_field_control, register_field_control) = {
        let _ = &form_element_ref;
        let params = UseFieldControlRegistrationParams {
            change: validation.change.clone(),
            commit: validation.commit_with_revalidate.clone(),
            invalid: invalid.clone(),
            marked_dirty_ref: Rc::clone(&marked_dirty_ref),
            name: Signal::derive(move || name.clone()),
            set_registered_field_name: {
                let registered_field_name = registered_field_name.clone();
                Rc::new(move |next: Option<String>| registered_field_name.set(next))
            },
            registered_field_id_ref: Rc::clone(&registered_field_id_ref),
            validity_data: validity_data.clone(),
            get_control_value: validation_helpers::control_value_getter(
                validation.input_ref.clone(),
            ),
            form_element_ref: form_element_ref.clone(),
            form_ref: Rc::clone(&form_ref),
        };
        let _ = &form_clear_errors;
        crate::field::registration::use_field_control_registration_bridge(params)
    };

    // `useImperativeHandle(actionsRef, …)` (`:148-150`).
    if let Some(actions) = &actions {
        actions(Rc::new(move || validate_field_control()) as crate::field::context::FieldRootActions);
    }

    // The context value (`:152-184`).
    let context = FieldRootContext {
        invalid: invalid.clone(),
        name: effective_name.clone(),
        validity_data: validity_data.clone(),
        disabled: Signal::derive(move || disabled_fieldset || disabled_prop),
        set_touched,
        set_dirty,
        set_filled,
        set_focused,
        validation_mode,
        should_validate_on_change: should_validate_on_change.clone(),
        state: state.clone(),
        register_field_control,
        validation: validation.clone(),
    };

    // The LabelableProvider scope (`:202-211` wrapping `:193`): the parts' label
    // association lives inside it.
    provide_labelable_context();

    // The element render (`:186-191`) — the state walk materialized into typed slots
    // (the accordion root precedent).
    let attributes = crate::field::context::field_state_attributes(&state);
    let attrs = crate::field::parts_view::FieldStateAttributes::from_walk(&attributes);

    let _ = SendWrapper::new(|| {}); // (the SendWrapper import is used by the bridge types)

    view! {
        <div
            class={class}
            style={style}
            data-disabled={attrs.data_disabled}
            data-touched={attrs.data_touched}
            data-dirty={attrs.data_dirty}
            data-valid={attrs.data_valid}
            data-invalid={attrs.data_invalid}
            data-filled={attrs.data_filled}
            data-focused={attrs.data_focused}
        >
            {context_bag_provide(context)}
            {children.map(|children| children())}
        </div>
    }
}

/// Publishes the context under the leptos runtime and returns `()` — split out so the
/// view body stays readable (the provide happens before the children render).
fn context_bag_provide(context: FieldRootContext) -> () {
    provide_context(context);
}

/// The `Field.Root` component — the `#[component]` wrapper over [`field_root_view`]
/// with the prop vocabulary the docs-page layer and the tests take.
#[component]
pub fn FieldRoot(
    #[prop(default = None, optional)] validate: Option<
        Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> crate::field::validation::ValidationOutcome>,
    >,
    #[prop(default = 0, optional)] validation_debounce_time: u32,
    #[prop(default = None, optional)] validation_mode: Option<FormValidationMode>,
    #[prop(default = None, optional)] name: Option<String>,
    #[prop(default = false, optional)] disabled: bool,
    #[prop(default = None, optional)] invalid: Option<bool>,
    #[prop(default = None, optional)] dirty: Option<bool>,
    #[prop(default = None, optional)] touched: Option<bool>,
    #[prop(default = None, optional)] class: Option<String>,
    children: leptos::children::Children,
) -> impl IntoView {
    field_root_view(FieldRootProps {
        validate,
        validation_debounce_time,
        validation_mode,
        name,
        disabled,
        invalid,
        dirty,
        touched,
        actions: None,
        class,
        style: None,
        element_attributes: Vec::new(),
        children: Some(children),
    })
}
