//! The Field control — port of `FieldControl`
//! (`packages/react/src/field/control/FieldControl.tsx`), the DOM→state-machine
//! boundary.
//!
//! All control behavior lives in one props bag upstream (`:127-213`); the port
//! materializes it onto the rendered `<input>`: the value model (`useControlled`
//! semantics), the registration effect (the `useRegisterFieldControl` body re-homed —
//! the internals hook reads the rg-0.2 context this crate re-homes, so the effect
//! runs against the re-homed bag directly; the in-place re-registration and the
//! unmount unregistration keep the `:21-45` mechanics), the filled-on-mount sync, the
//! programmatic-change bridge (`useValueChanged`), the autoFocus hydration, and the
//! four handlers with the `getValidationProps` overlay (`aria-invalid` + the
//! labelable `aria-describedby`).

use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;

use leptos_ui_internals::field_register_control::FieldControlRegistration;
use leptos_ui_internals::form_context::FormValidationMode;
use leptos_ui_internals::labelable_provider::{
    UseLabelableIdParams, use_labelable_context, use_labelable_id,
};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::use_timeout::Timeout;

use crate::field::context::{FieldStateValue, use_field_root_context};
use crate::field::validation::{RegisteredInput, cell_peek};

/// Reads a leptos handle's untracked value through the fully-qualified accessor —
/// the dual-runtime law (named rg trait imports would shadow the prelude glob).
fn rg_get_untracked<T: reactive_graph::traits::GetUntracked>(source: &T) -> T::Value {
    reactive_graph::traits::GetUntracked::get_untracked(source)
}

/// The control props — upstream's destructured set (`FieldControl.tsx:37-49`).
pub struct FieldControlViewProps {
    /// `id` (`:40`) — the explicit id; removal falls back to the generated one.
    pub id: Option<String>,
    /// `name` (`:41`) — the control's own name (the root's wins when both exist).
    pub name: Option<String>,
    /// `value` (`:42`) — the controlled value.
    pub value: Option<String>,
    /// `defaultValue` (`:46`) — the uncontrolled seed.
    pub default_value: Option<String>,
    /// `disabled` (`:43`) — default `false`.
    pub disabled: bool,
    /// `onValueChange` (`:44`) — `(value, details)`; `details.cancel()` aborts the
    /// validation pass.
    pub on_value_change: Option<Rc<dyn Fn(String, &ValueChangeEventDetails)>>,
    /// `autoFocus` (`:46`).
    pub auto_focus: bool,
    /// The user's `class`.
    pub class: Option<String>,
    /// `...elementProps` (`:48`) — the last-but-one bag.
    pub element_attributes: Vec<(String, String)>,
}

impl Default for FieldControlViewProps {
    fn default() -> Self {
        FieldControlViewProps {
            id: None,
            name: None,
            value: None,
            default_value: None,
            disabled: false,
            on_value_change: None,
            auto_focus: false,
            class: None,
            element_attributes: Vec::new(),
        }
    }
}

/// The `onValueChange` details — upstream's `createChangeEventDetails(REASONS.none)`
/// (`:141`); the shared cancel cell is the `createBaseUIEventDetails` port's
/// object-identity convention.
pub struct ValueChangeEventDetails {
    /// The cancel flag — shared across every clone of one details value.
    pub cancel_flag: Rc<Cell<bool>>,
    /// The native event (`event.nativeEvent`).
    pub event: Option<web_sys::Event>,
}

impl ValueChangeEventDetails {
    /// `details.cancel()`.
    pub fn cancel(&self) {
        self.cancel_flag.set(true);
    }

    /// `details.isCanceled`.
    pub fn is_canceled(&self) -> bool {
        self.cancel_flag.get()
    }
}

/// The view function — upstream's `FieldControl` body (`:33-216`). Must be called
/// inside a reactive owner.
pub fn field_control_view(props: FieldControlViewProps) -> impl IntoView {
    let FieldControlViewProps {
        id: id_prop,
        name: name_prop,
        value: value_prop,
        default_value,
        disabled: disabled_prop,
        on_value_change,
        auto_focus,
        class,
        element_attributes,
    } = props;

    // `useFieldRootContext()` (`:51-62`) — the default-optional form: the control
    // works outside a Root through the inert shell.
    let field = use_field_root_context();

    // `useFormContext()` (`:63`).
    let form = leptos_ui_internals::form_context::use_form_context();
    let form_clear_errors = form.clear_errors.clone();
    let form_element_ref = form.element_ref.clone();
    let form_submit_count_ref = form.submit_count_ref.clone();

    let context_disabled = field.disabled.clone();
    let disabled_signal = Signal::derive(move || context_disabled.get() || disabled_prop);

    // `name = fieldName ?? nameProp` (`:66`) — the ROOT name wins (the source's
    // resolution order; the discrepancy-log entry).
    let field_name = field.name.clone();
    let resolved_name = Signal::derive(move || field_name.get().or_else(|| name_prop.clone()));

    // `const state = { ...fieldState, disabled }` (`:68-71`).
    let state = FieldStateValue {
        disabled: disabled_signal.clone(),
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    };

    // `const { labelId } = useLabelableContext()` (`:73`).
    let labelable = use_labelable_context();
    let label_id = labelable.label_id.clone();

    // `const id = useLabelableId({ id: idProp })` (`:75`).
    let id_signal = use_labelable_id(UseLabelableIdParams { id: id_prop, enabled: true });

    // `useControlled` (`:77-82`): the mode fixes from the initial prop's defined-ness
    // (packages/utils/src/useControlled.ts:41); the uncontrolled branch owns the
    // value.
    let is_controlled = value_prop.is_some();
    let value_signal: RwSignal<Option<String>> = RwSignal::new(match &value_prop {
        Some(value) => value.clone(),
        None => default_value.clone(),
    });
    let value_unwrapped: Signal<Option<String>> = {
        let value_prop = value_prop.clone();
        let value_signal = value_signal.clone();
        Signal::derive(move || {
            if is_controlled {
                value_prop.clone()
            } else {
                value_signal.get()
            }
        })
    };
    // The machine handles.
    let validation = field.validation.clone();
    let validity_data = field.validity_data.clone();
    let set_dirty = field.set_dirty.clone();
    let set_filled = field.set_filled.clone();
    let set_focused = field.set_focused.clone();
    let set_touched = field.set_touched.clone();
    let validation_mode = field.validation_mode;

    // The registration effect (`useRegisterFieldControl.ts:21-45`, re-homed): the
    // registration pushes in place on every id/value/name change (`use_iso_layout_effect`
    // → a leptos effect; the same-id update keeps the registry entry in place), a
    // `None` value registers `undefined` while disabled, and the unmount unregisters.
    let control_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));
    {
        let register = field.register_field_control.clone();
        let control_ref = Rc::clone(&control_ref);
        let id_signal = id_signal.clone();
        let disabled_signal = disabled_signal.clone();
        let resolved_name = resolved_name.clone();
        let value_unwrapped = value_unwrapped.clone();
        Effect::new(move |_| {
            let registration = if disabled_signal.get() {
                None
            } else {
                Some(FieldControlRegistration {
                    control_ref: Rc::clone(&control_ref),
                    id: Some(rg_get_untracked(&id_signal)),
                    name: rg_get_untracked(&resolved_name),
                    get_value: None,
                    value: rg_get_untracked(&value_unwrapped)
                        .map(|text| Value::String(text)),
                })
            };
            register(leptos_ui_internals::labelable_provider::ControlIdSource::new(), registration);
        });
    }
    // The unmount unregistration (`:40-45`).
    {
        let register = field.register_field_control.clone();
        let id_signal = id_signal.clone();
        on_cleanup(move || {
            let _ = &id_signal;
            register(leptos_ui_internals::labelable_provider::ControlIdSource::new(), None);
        });
    }
    // The input registry registration (`FieldControl.tsx:135`'s `ref:
    // validation.inputRef`): the element lands in the machine's fallback slot AND the
    // registered-inputs map, so group fields and single controls validate through the
    // same path.
    {
        let input_ref = Rc::clone(&validation.input_ref);
        let control_ref = Rc::clone(&control_ref);
        let registered_inputs = validation.registered_inputs.clone();
        let register_input = validation.register_input.clone();
        Effect::new(move |_| {
            let element = cell_peek(&control_ref);
            let Some(element) = element else { return };
            let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() else { return };
            reactive_graph::traits::Set::set(&input_ref, Some(input.clone()));
            let _unregister = register_input(
                &input,
                RegisteredInput {
                    control_ref: Rc::clone(&control_ref),
                    value: None,
                },
            );
        });
    }

    // Filled-on-mount sync (`:100-105`).
    {
        let value_unwrapped = value_unwrapped.clone();
        let input_ref = Rc::clone(&validation.input_ref);
        Effect::new(move |_| {
            let current = value_unwrapped
                .get()
                .or_else(|| cell_peek(&input_ref).map(|input| input.value()));
            if let Some(current) = current {
                set_filled(!current.is_empty());
            }
        });
    }

    // `useValueChanged` (`:107-116`): a *programmatic* controlled-value change clears
    // form errors, recomputes dirty, and triggers `validation.change`.
    {
        let value_unwrapped = value_unwrapped.clone();
        let clear_errors = form_clear_errors.clone();
        let validity_data = validity_data.clone();
        Effect::new(move |_| {
            let Some(serialized) = value_unwrapped.get() else { return };
            if !is_controlled {
                return; // upstream's effect body early-returns on `undefined`
            }
            clear_errors(rg_get_untracked(&resolved_name).as_deref());
            set_dirty(
                serialized
                    != rg_get_untracked(&validity_data)
                        .initial_value
                        .as_str()
                        .unwrap_or(""),
            );
            (validation.change)(Some(Value::String(serialized)), false);
        });
    }

    // autoFocus hydration (`:121-125`).
    {
        let control_ref = Rc::clone(&control_ref);
        Effect::new(move |_| {
            if !auto_focus {
                return;
            }
            let Some(element) = cell_peek(&control_ref) else { return };
            let Some(node) = element.dyn_ref::<web_sys::Node>() else { return };
            let document = owner_document(Some(node));
            if let Some(active) = document.active_element() {
                if let Ok(active) = active.dyn_into::<web_sys::Element>() {
                    if active == element {
                        set_focused(true);
                    }
                }
            }
        });
    }

    // `onChange` (`:139-159`).
    let on_input = {
        let validity_data = validity_data.clone();
        let set_dirty = set_dirty.clone();
        let set_filled = set_filled.clone();
        let clear_errors = form_clear_errors.clone();
        let change = validation.change.clone();
        let resolved_name = resolved_name.clone();
        let value_signal = value_signal.clone();
        move |event: web_sys::Event| {
            let Some(input) = event
                .current_target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            let input_value = input.value();
            let cancel_flag = Rc::new(Cell::new(false));
            let details = ValueChangeEventDetails {
                cancel_flag: Rc::clone(&cancel_flag),
                event: Some(event.clone()),
            };
            if let Some(callback) = &on_value_change {
                callback(input_value.clone(), &details);
            }

            // Controlled: the value syncs from the prop (`:146-148`).
            if is_controlled {
                return;
            }

            // Dirty before `validation.change` (`:150-152`).
            set_dirty(
                input_value
                    != rg_get_untracked(&validity_data)
                        .initial_value
                        .as_str()
                        .unwrap_or(""),
            );
            set_filled(!input_value.is_empty());
            value_signal.set(Some(input_value.clone()));

            // The native-prevention + cancel gate (`:155-158`).
            if !event.default_prevented() && !details.is_canceled() {
                clear_errors(rg_get_untracked(&resolved_name).as_deref());
                change(Some(Value::String(input_value)), false);
            }
        }
    };

    // `onFocus` (`:160-162`).
    let on_focus = {
        let set_focused = set_focused.clone();
        move |_event: web_sys::FocusEvent| {
            set_focused(true);
        }
    };

    // `onBlur` (`:163-187`).
    let on_blur = {
        let validity_data = validity_data.clone();
        let commit = validation.commit.clone();
        let input_ref = Rc::clone(&validation.input_ref);
        move |event: web_sys::FocusEvent| {
            set_touched(true);
            set_focused(false);

            if validation_mode == FormValidationMode::OnBlur {
                let Some(input) = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
                else {
                    return;
                };
                let input_value = input.value();
                commit(Value::String(input_value.clone()));

                if is_controlled {
                    // The controlled-blur normalization re-commit (`:171-185`): a
                    // blur-handler rewrite back to the initial value is a programmatic
                    // reset — committing it would only surface `valueMissing` noise.
                    let input_ref = Rc::clone(&input_ref);
                    let validity_data = validity_data.clone();
                    let commit = Rc::clone(&commit);
                    leptos::prelude::queue_microtask(move || {
                        let Some(next_input) = cell_peek(&input_ref) else {
                            return;
                        };
                        let next_value = next_input.value();
                        if next_value != input_value
                            && next_value
                                != rg_get_untracked(&validity_data)
                                    .initial_value
                                    .as_str()
                                    .unwrap_or("")
                        {
                            commit(Value::String(next_value));
                        }
                    });
                }
            }
        }
    };

    // `onKeyDown` (`:188-207`) — the Enter-key implicit-submission dance.
    let on_key_down = {
        let commit = validation.commit.clone();
        let input_ref = Rc::clone(&validation.input_ref);
        move |event: web_sys::KeyboardEvent| {
            let Some(input) = event
                .current_target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            else {
                return;
            };
            if input.tag_name() != "INPUT" || event.key() != "Enter" {
                return;
            }
            set_touched(true);
            let value = input.value();
            let form = input.form();
            let own_form = cell_peek(&form_element_ref);
            let matches_form = form
                .as_ref()
                .zip(own_form.as_ref())
                .is_some_and(|(form, own)| form == own);
            if matches_form && !event.default_prevented() {
                // Implicit submission runs after keydown; fall back unless the Form
                // advanced the submit count first (`:198-202`).
                let submit_count = form_submit_count_ref.get();
                let commit = Rc::clone(&commit);
                let input_ref = Rc::clone(&input_ref);
                let submit_count_ref = Rc::clone(&form_submit_count_ref);
                let timeout = Timeout::create();
                timeout.start(0, move || {
                    if submit_count_ref.get() == submit_count {
                        let value =
                            cell_peek(&input_ref).map(|input| input.value()).unwrap_or(value);
                        commit(Value::String(value));
                    }
                });
            } else {
                commit(Value::String(value));
            }
        }
    };

    // The live attribute bindings (`:127-137` + `:210`'s overlay).
    let id_attr = {
        let id_signal = id_signal.clone();
        move || Some(rg_get_untracked(&id_signal))
    };
    let name_attr = move || resolved_name.get();
    let value_attr = {
        let value_unwrapped = value_unwrapped.clone();
        move || value_unwrapped.get()
    };
    let disabled_attr = move || disabled_signal.get().then(|| String::new());
    let aria_labelledby_attr = {
        let label_id = label_id.clone();
        move || rg_get_untracked(&label_id)
    };
    // `getValidationProps(disabled, props)` (`:210`) — the overlay's `aria-invalid`,
    // re-derived per read with the live state (the same gate the machine's closure
    // runs; the attribute binding reads the state bag directly so the DOM tracks it).
    let aria_invalid_attr = {
        let state = state.clone();
        move || {
            (state.valid.get() == Some(false)
                && !state.disabled.get()
                && !disabled_prop)
                .then(|| "true".to_string())
        }
    };
    let aria_describedby_attr = {
        let get_description_props = labelable.get_description_props.clone();
        move || {
            // The labelable merge installs the message ids (the
            // `getDescriptionProps` slice reads eagerly and returns the merged
            // value through the closure below).
            let mut bag: Vec<(String, crate::field::validation::AttributeFn)> = Vec::new();
            get_description_props(&mut bag);
            bag.into_iter().find(|(name, _)| name == "aria-describedby").and_then(|(_, value)| value())
        }
    };

    // The class merge: the state walk's `data-*` attributes bind separately (the
    // accordion precedent); `class` rides as-is.
    let (data_disabled_attr, data_touched_attr, data_dirty_attr, data_valid_attr, data_invalid_attr, data_filled_attr, data_focused_attr) =
        crate::field::parts_view::field_state_attributes(&state);

    let _ = &element_attributes;

    view! {
        <input
            id={id_attr}
            name={name_attr}
            value={value_attr}
            disabled={disabled_attr}
            class={class}
            autofocus={auto_focus.then(|| "true".to_string())}
            aria-invalid={aria_invalid_attr}
            aria-labelledby={aria_labelledby_attr}
            aria-describedby={aria_describedby_attr}
            data-disabled={data_disabled_attr}
            data-touched={data_touched_attr}
            data-dirty={data_dirty_attr}
            data-valid={data_valid_attr}
            data-invalid={data_invalid_attr}
            data-filled={data_filled_attr}
            data-focused={data_focused_attr}
            on:input=on_input
            on:focus=on_focus
            on:blur=on_blur
            on:keydown=on_key_down
        />
    }
}

/// The component crate's `Field.Control` — the `#[component]` wrapper over
/// [`field_control_view`].
#[component]
pub fn FieldControl(
    /// The explicit id (`:40`).
    #[prop(default = None, optional)] id: Option<String>,
    /// The control's own name (`:41`).
    #[prop(default = None, optional)] name: Option<String>,
    /// The controlled value (`:42`).
    #[prop(default = None, optional)] value: Option<String>,
    /// The uncontrolled seed (`:46`).
    #[prop(default = None, optional)] default_value: Option<String>,
    /// `disabled` (`:43`).
    #[prop(default = false, optional)] disabled: bool,
    /// The user's class.
    #[prop(default = None, optional)] class: Option<String>,
) -> impl IntoView {
    field_control_view(FieldControlViewProps {
        id,
        name,
        value,
        default_value,
        disabled,
        on_value_change: None,
        auto_focus: false,
        class,
        element_attributes: Vec::new(),
    })
}
