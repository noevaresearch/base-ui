//! The Field root — port of `FieldRoot` (`packages/react/src/field/root/FieldRoot.tsx`).
//!
//! Upstream is two layers: `FieldRoot` wraps `FieldRootInner` in a `LabelableProvider`
//! (`:202-211`), and the inner body builds the field state, the validation machine,
//! and the registration pair before rendering its `div` under the context provider
//! (`:22-194`). The port keeps both: [`field_root_view`] runs the LabelableProvider
//! scope (the internals' owner-bridge — the direction-provider docs-page pattern) and
//! the inner body; `FieldRoot` is the `#[component]` wrapper.
//!
//! The context bag rides the leptos runtime (the accordion/meter precedent: the
//! internals' rg-0.2 signals cannot drive the leptos view tree), re-homing upstream's
//! `FieldRootContext` value — derived state, the validity record, the four guarded
//! setters, `shouldValidateOnChange`, `registerFieldControl`, and the whole
//! `validation` machine. The internals' `FieldRootContextValue`/`FieldValidationBag`
//! stay the vocabulary this bag mirrors; the *consumers* of the mirror are this
//! crate's parts (the future form-aware controls register through the same bag).
//!
//! Root body, in upstream order (`FieldRootInner`):
//!
//! 1. `useFormContext()` (`:26`) — the Form's errors/mode/submit-count through the
//!    internals' owner bridge (inside [`field_root_view`]'s bridge window).
//! 2. The destructuring defaults (`:28-42`).
//! 3. The four state booleans + validity record (`:50-104`).
//! 4. The derived `valid` gate (`:109`) and the state bag (`:111-121`).
//! 5. `useFieldValidation` (`:123-134`).
//! 6. `useFieldControlRegistration` (`:136-146`) + `actionsRef` (`:148-150`).
//! 7. The context value + provide (`:152-184`, `:193`).
//! 8. The root element with `fieldValidityMapping` (`:186-191`).

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use serde_json::Value;

use leptos_ui_internals::field_constants::{DEFAULT_VALIDITY_STATE, FieldValidityData};
use leptos_ui_internals::form_context::FormValidationMode;

use crate::field::context::{FieldRootActions, FieldRootContext, FieldStateValue};
use crate::field::parts_view::{FieldRootAttributes, field_state_attributes};
use crate::field::registration::root_registration;
use crate::field::validation::{UseFieldValidationParams, ValidationOutcome, use_field_validation};

/// The Field root props — upstream's destructured set (`FieldRoot.tsx:28-42`).
pub struct FieldRootProps {
    /// `validate` (`:31`) — `None` is the `() => null` default (`:46`).
    pub validate:
        Option<Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>>,
    /// `validationDebounceTime` (`:32`) — default `0`.
    pub validation_debounce_time: u32,
    /// `validationMode` (`:33`) — `None` resolves to the Form's mode, else `onSubmit`.
    pub validation_mode: Option<FormValidationMode>,
    /// `name` (`:34`) — wins over the control's fallback (the source's resolution
    /// order; the discrepancy-log entry).
    pub name: Option<String>,
    /// `disabled` (`:35`) — default `false`.
    pub disabled: bool,
    /// `invalid` (`:36`).
    pub invalid: Option<bool>,
    /// `dirty` (`:37`) — the controlled override.
    pub dirty: Option<bool>,
    /// `touched` (`:38`) — the controlled override.
    pub touched: Option<bool>,
    /// `actionsRef` (`:39`) — receives the imperative `validate` handle at body time.
    pub actions: Option<Rc<dyn Fn(FieldRootActions)>>,
    /// The user's `class` (`:30`).
    pub class: Option<String>,
    /// The user's `style` (`:40`).
    pub style: Option<Vec<(String, String)>>,
    /// `...elementProps` (`:41`) — the last bag (plain attributes override the state
    /// walk).
    pub element_attributes: Vec<(String, String)>,
    /// The parts subtree.
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

/// The view function — upstream's `FieldRoot` → `LabelableProvider` →
/// `FieldRootInner` chain (`:202-211`). Must be called inside a reactive owner.
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

    // The LabelableProvider scope (`:202-211`): the internals' provider publish runs
    // inside its own rg-0.2 owner and the inner body (which reads that context)
    // builds inside the same window — the direction-provider docs-page bridge, where
    // `Owner::with` (not a bare `set()`) saves and restores the surrounding owner.
    let bridge_owner = reactive_graph::owner::Owner::new();
    bridge_owner.with(|| provide_labelable_context());

    // `useFormContext()` (`:26`) — through the same internals runtime (the bridge
    // window); the inert default outside a `<Form>` keeps every read safe.
    let form = bridge_owner.with(|| leptos_ui_internals::form_context::use_form_context());
    let form_errors = form.errors.clone();
    let form_validation_mode = form.validation_mode;
    let submit_count_ref = form.submit_count_ref.clone();
    let form_ref = form.form_ref.clone();
    let form_element_ref = form.element_ref.clone();
    let form_clear_errors = form.clear_errors.clone();

    // The destructuring defaults (`:31-42`).
    let validate = validate
        .unwrap_or_else(|| Rc::new(|_, _| ValidationOutcome::Valid));
    let validation_mode =
        validation_mode.unwrap_or(form_validation_mode);
    // `useFieldsetRootContext(true)?.disabled` (`:44`) — the fieldset unit is
    // unported, so the hook's `None` default is the permanent answer here.
    let disabled_fieldset = false;

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

    // `markedDirtyRef = useRef(dirty)` (`:58`) — seeded with the resolved dirty.
    let marked_dirty_ref: Rc<std::cell::Cell<bool>> =
        Rc::new(std::cell::Cell::new(dirty.get_untracked()));

    // `registeredFieldIdRef` (`:59`), `registeredFieldName` (`:60`), the effective
    // name (`:61`).
    let registered_field_id_ref: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let registered_field_name: RwSignal<Option<String>> = RwSignal::new(None);
    let effective_name = {
        let registered_field_name = registered_field_name.clone();
        Signal::derive(move || name.clone().or_else(|| registered_field_name.get()))
    };

    // The dirty-prop mirror (`:63-67`): the layout effect becomes a tracked effect
    // (the body runs once; the effect keeps the mirror live across prop changes
    // delivered through rebuilds).
    {
        let marked_dirty_ref = Rc::clone(&marked_dirty_ref);
        Effect::new(move |_| {
            if let Some(prop) = dirty_prop {
                marked_dirty_ref.set(prop);
            }
        });
    }

    // `setDirty` (`:69-78`): the controlled no-op + the `markedDirtyRef` feed.
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

    // `setTouched` (`:80-85`): the controlled no-op.
    let set_touched: Rc<dyn Fn(bool)> = {
        let touched_state = touched_state.clone();
        Rc::new(move |value: bool| {
            if touched_prop.is_some() {
                return;
            }
            touched_state.set(value);
        })
    };

    // `setFilled` / `setFocused` (`:52-53`).
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
        Rc::new(move || {
            validation_mode == FormValidationMode::OnChange
                || (validation_mode == FormValidationMode::OnSubmit && submit_count_ref.get() > 0)
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

    // The form-error lookup + `invalid` (`:93-96`).
    let name_for_errors = effective_name.clone();
    let invalid = Signal::derive(move || {
        let has_form_error = match name_for_errors.get() {
            Some(field_name) => form_errors
                .get_untracked()
                .get(&field_name)
                .is_some_and(|error| match error {
                    leptos_ui_internals::form_context::FormErrorValue::Single(message) => {
                        !message.is_empty()
                    }
                    leptos_ui_internals::form_context::FormErrorValue::Multiple(messages) => {
                        !messages.is_empty()
                    }
                }),
            None => false,
        };
        invalid_prop == Some(true) || has_form_error
    });

    // The derived `valid` gate (`:109`): app-controlled invalidity survives
    // `disabled`; computed validity is suppressed while disabled.
    let valid = {
        let invalid = invalid.clone();
        let validity_data = validity_data.clone();
        Signal::derive(move || {
            if invalid.get() {
                Some(false)
            } else if disabled_fieldset || disabled_prop {
                None
            } else {
                validity_data.get().state.valid
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

    // `useFieldControlRegistration` (`:136-146`) — the re-homed root-side hook
    // (`registration.rs`): the Form-registry entry ownership, the initial-value
    // baseline, the source-keyed control handover.
    let (validate_field_control, register_field_control) = root_registration(
        crate::field::registration::RootRegistrationParams {
            change: validation.change.clone(),
            commit: validation.commit_with_revalidate.clone(),
            invalid: invalid.clone(),
            marked_dirty_ref: Rc::clone(&marked_dirty_ref),
            name: name.clone(),
            set_registered_field_name: {
                let registered_field_name = registered_field_name.clone();
                Rc::new(move |next: Option<String>| registered_field_name.set(next))
            },
            registered_field_id_ref: Rc::clone(&registered_field_id_ref),
            validity_data: validity_data.clone(),
            form_ref,
            form_element_ref,
        },
    );

    // `useImperativeHandle(actionsRef, …)` (`:148-150`).
    if let Some(actions) = &actions {
        actions(Rc::new(move || validate_field_control()));
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
    provide_context(context);

    // The element (`:186-191`): the state walk through `fieldValidityMapping`,
    // materialized into typed slots (the accordion root precedent). The attributes
    // derive live from the state bag.
    let root_attrs = FieldRootAttributes::derive(&state, &class, &style, &element_attributes);

    // The children build inside the bridge window so the parts' labelable reads
    // (`use_label`, `use_labelable_id`, `use_labelable_context`) resolve against this
    // root's provider; the views the children return are plain leptos views, safe to
    // mount outside the window.
    let view = bridge_owner.with(|| children.map(|children| children()));

    // The bridge owner stays alive for the page's lifetime (the direction-provider
    // precedent: the provided memo/context must outlive the subtree).
    std::mem::forget(bridge_owner);

    view! {
        <div
            class={class}
            style={style}
            data-disabled={root_attrs.data_disabled}
            data-touched={root_attrs.data_touched}
            data-dirty={root_attrs.data_dirty}
            data-valid={root_attrs.data_valid}
            data-invalid={root_attrs.data_invalid}
            data-filled={root_attrs.data_filled}
            data-focused={root_attrs.data_focused}
        >
            {view}
        </div>
    }
}

/// The `Field.Root` component — the `#[component]` wrapper over [`field_root_view`].
#[component]
pub fn FieldRoot(
    #[prop(default = None, optional)] validate: Option<
        Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>,
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

/// Silences the unused-capture lint while the Form-bridge reads land with the `<Form>`
/// component (the `library: form` item).
#[allow(unused)]
fn form_bridge_reserved(
    _clear_errors: leptos_ui_internals::form_context::ClearErrorsFn,
    _form_validation_mode: FormValidationMode,
    _get_untracked: GetUntracked,
    _set: Set<()>,
) {
}
