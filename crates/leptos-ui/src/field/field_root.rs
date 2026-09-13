//! The Field root — port of `FieldRoot` (`packages/react/src/field/root/FieldRoot.tsx`).
//!
//! Upstream is two layers: `FieldRoot` wraps `FieldRootInner` in a `LabelableProvider`
//! (`:202-211`), and the inner body builds the field state, the validation machine,
//! and the registration pair before rendering its `div` under the context provider
//! (`:22-194`). The port keeps both: [`field_root_view`] runs the LabelableProvider
//! scope (the internals' owner bridge) and the inner body; [`FieldRoot`] is the
//! `#[component]` wrapper.
//!
//! The bridge-window mechanics (the direction-provider docs-page pattern): an
//! rg-0.2 `Owner::with` swaps only the *reactive-graph* thread-local — the leptos
//! owner stays active inside the window — so leptos `Effect`s created in the window
//! run on the leptos runtime while the rg contexts (`LabelableProvider`,
//! `FormContext`) resolve against the bridge owner. The window owner is forgotten to
//! outlive the page (its provided memo/context must outlive the subtree).
//!
//! The context bag rides the leptos runtime (the accordion/meter precedent: the
//! internals' rg-0.2 signals cannot drive the leptos view tree), re-homing upstream's
//! `FieldRootContext` value — derived state, the validity record, the four guarded
//! setters, `shouldValidateOnChange`, `registerFieldControl`, and the whole
//! `validation` machine.
//!
//! Root body, in upstream order (`FieldRootInner`):
//!
//! 1. `useFormContext()` (`:26`) — the Form's errors/mode/submit-count through the
//!    bridge window.
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
use send_wrapper::SendWrapper;
use serde_json::Value;

use leptos_ui_internals::field_constants::{FieldValidityData, DEFAULT_VALIDITY_STATE};
use leptos_ui_internals::form_context::FormValidationMode;

use crate::field::context::{FieldRootActions, FieldRootContext, FieldStateValue};
use crate::field::parts_view::{field_state_attributes, LiveFieldAttributes};
use crate::field::registration::root_registration;
use crate::field::validation::{use_field_validation, UseFieldValidationParams, ValidationOutcome};

/// The Field root props for the view layer — upstream's destructured set
/// (`FieldRoot.tsx:28-42`). (`FieldRootProps` itself is the `#[component]`-generated
/// struct for the public component.)
pub struct FieldRootViewProps {
    /// `validate` (`:31`) — `None` is the `() => null` default (`:46`).
    pub validate: Option<Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>>,
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
    /// `...elementProps` (`:41`) — the last bag (plain attributes override the state
    /// walk).
    pub element_attributes: Vec<(String, String)>,
    /// The parts subtree.
    pub children: Option<leptos::children::Children>,
}

impl Default for FieldRootViewProps {
    fn default() -> Self {
        FieldRootViewProps {
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
            element_attributes: Vec::new(),
            children: None,
        }
    }
}

/// The view function — upstream's `FieldRoot` → `LabelableProvider` →
/// `FieldRootInner` chain (`:202-211`). Must be called inside a reactive owner.
pub fn field_root_view(props: FieldRootViewProps) -> impl IntoView {
    let FieldRootViewProps {
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
        element_attributes,
        children,
    } = props;

    // The LabelableProvider scope (`:202-211`): the provider publishes inside its own
    // rg-0.2 owner and the inner body (which reads that context) builds inside the
    // same window. `Owner::with` swaps only the rg thread-local — the leptos owner
    // stays active, so the leptos Effects the body creates are live (the
    // direction-provider bridge precedent).
    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(|| {
        // The provider publishes itself under `SharedLabelableContext` (its own
        // `provide_context` call) — the leaf parts' `use_labelable_context` reads that
        // type; no second provide here.
        leptos_ui_internals::labelable_provider::provide_labelable_context();

        // `useFormContext()` (`:26`) — the inert default outside a `<Form>` keeps
        // every read safe. The destructuring defaults (`:31-46`) resolve here too:
        // `validate`'s `() => null` default and `validationMode`'s Form fallback.
        let form = leptos_ui_internals::form_context::use_form_context();
        let validate = validate.unwrap_or_else(|| Rc::new(|_, _| ValidationOutcome::Valid));
        let validation_mode = validation_mode.unwrap_or(form.validation_mode);
        field_root_inner(FieldRootInnerParams {
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
            element_attributes,
            children,
            form,
        })
    });

    // The bridge owner stays alive for the page's lifetime (the direction-provider
    // precedent: the provided memo/context must outlive the subtree).
    std::mem::forget(bridge_owner);

    view
}

/// The inner-body parameters — everything [`field_root_view`] destructured plus the
/// Form-bridge bag the window resolved.
struct FieldRootInnerParams {
    validate: Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>,
    validation_debounce_time: u32,
    validation_mode: FormValidationMode,
    name: Option<String>,
    disabled: bool,
    invalid: Option<bool>,
    dirty: Option<bool>,
    touched: Option<bool>,
    actions: Option<Rc<dyn Fn(FieldRootActions)>>,
    class: Option<String>,
    element_attributes: Vec<(String, String)>,
    /// The parts subtree (built in this body — the bridge window is still open).
    children: Option<leptos::children::Children>,
    form: leptos_ui_internals::form_context::FormContextValue,
}

/// Upstream's `FieldRootInner` body (`FieldRoot.tsx:22-194`).
fn field_root_inner(params: FieldRootInnerParams) -> impl IntoView {
    let FieldRootInnerParams {
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
        element_attributes,
        children,
        form,
    } = params;

    let form_errors = form.errors.clone();
    let submit_count_ref = form.submit_count_ref.clone();
    let form_ref = form.form_ref.clone();

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
        let name = name.clone();
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

    // The form-error lookup + `invalid` (`:93-96`). The errors record is the
    // internals' rg-0.2 signal (it feeds no view here), read through the
    // fully-qualified untracked accessor — the dual-runtime law.
    let name_for_errors = effective_name.clone();
    let invalid = Signal::derive(move || {
        let has_form_error = match name_for_errors.get() {
            Some(field_name) => {
                let errors = reactive_graph::traits::GetUntracked::get_untracked(&form_errors);
                errors.iter().any(|(key, error)| {
                    *key == field_name
                        && match error {
                            leptos_ui_internals::form_context::FormErrorValue::Single(message) => {
                                !message.is_empty()
                            }
                            leptos_ui_internals::form_context::FormErrorValue::Multiple(
                                messages,
                            ) => !messages.is_empty(),
                        }
                })
            }
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
        filled: filled.into(),
        focused: focused.into(),
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
    let (validate_field_control, registration) =
        root_registration(crate::field::registration::RootRegistrationParams {
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
            form_element_ref: form.element_ref.clone(),
        });
    let register_field_control = registration.register;

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
    // The context value (`:152-184`): the Rc-heavy bag rides the `SendWrapper` bridge
    // (the `SharedFormContext`/`composite_list` precedent — leptos `provide_context`
    // demands `Send + Sync`, this crate's handles are main-thread `Rc`s).
    provide_context(SendWrapper::new(context));

    // The element (`:186-191`): the state walk through `fieldValidityMapping`,
    // materialized into typed slots (the accordion root precedent). The attributes
    // derive live from the state bag.
    let root_attrs: LiveFieldAttributes = field_state_attributes(&state);

    // The children build inside this body — which the caller runs INSIDE the
    // bridge window (`bridge_owner.with(|| field_root_inner(..))` in
    // `field_root_view`) — so the parts' labelable reads (`use_label`,
    // `use_labelable_id`, `use_labelable_context`) resolve against this root's
    // provider; the views the children return are plain leptos views, safe to
    // mount outside the window (the direction-provider bridge-window precedent).
    let children_view = children.map(|children| children());

    // The `...elementProps` rest bag (`:41`, the last layer — the user's plain
    // attributes override the state walk per the later-bag-wins rule). Leptos
    // `view!` has no attribute spread, so the bag lands through a mount effect
    // writing the real node (the control's identical channel).
    let root_node: NodeRef<leptos::html::Div> = NodeRef::new();
    {
        let element_attributes = element_attributes.clone();
        Effect::new(move |_| {
            let Some(div) = root_node.get() else {
                return;
            };
            let element: &web_sys::Element = wasm_bindgen::JsCast::unchecked_ref(&div);
            for (name, value) in &element_attributes {
                if value.is_empty() {
                    let _ = element.remove_attribute(name);
                } else {
                    let _ = element.set_attribute(name, value);
                }
            }
        });
    }

    view! {
        <div
            node_ref=root_node
            class=class
            data-disabled=root_attrs.data_disabled
            data-touched=root_attrs.data_touched
            data-dirty=root_attrs.data_dirty
            data-valid=root_attrs.data_valid
            data-invalid=root_attrs.data_invalid
            data-filled=root_attrs.data_filled
            data-focused=root_attrs.data_focused
        >
            {children_view}
        </div>
    }
}

/// The public `Field.Root` component — the `#[component]` wrapper over
/// [`field_root_view`] (upstream's outer `FieldRoot` layer, `:202-211`; the
/// `actionsRef` slot rides [`FieldRootProps`] through [`field_root_view`]).
#[component]
pub fn FieldRoot(
    /// `validate(value, formValues)` — sync or async; `None` is the `() => null`
    /// default (`:31`, `:46`).
    #[prop(default = None, optional)]
    validate: Option<Rc<dyn Fn(&Value, &serde_json::Map<String, Value>) -> ValidationOutcome>>,
    /// `validationDebounceTime` (ms) (`:32`).
    #[prop(default = 0, optional)]
    validation_debounce_time: u32,
    /// `validationMode` (`:33`) — `None` resolves to the Form's mode.
    #[prop(default = None, optional)]
    validation_mode: Option<FormValidationMode>,
    /// `name` (`:34`) — wins over the control's fallback.
    #[prop(default = None, optional)]
    name: Option<String>,
    /// `disabled` (`:35`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// `invalid` (`:36`).
    #[prop(default = None, optional)]
    invalid: Option<bool>,
    /// `dirty` (`:37`) — the controlled override.
    #[prop(default = None, optional)]
    dirty: Option<bool>,
    /// `touched` (`:38`) — the controlled override.
    #[prop(default = None, optional)]
    touched: Option<bool>,
    /// The user's `class`.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The parts subtree.
    children: Children,
) -> impl IntoView {
    field_root_view(FieldRootViewProps {
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
        element_attributes: Vec::new(),
        children: Some(children),
    })
}
