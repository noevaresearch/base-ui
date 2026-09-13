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
    use_labelable_context, use_labelable_id, ControlIdSource, UseLabelableIdParams,
};
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::use_timeout::Timeout;

use crate::field::context::{use_field_root_context, FieldStateValue};
use crate::field::validation::cell_peek;

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
    // The label's id publication and the message-ids registrations are OTHER
    // parts' body-time writes — after this control's first attribute evaluation
    // when the control renders first — so the attribute closures track leptos
    // mirrors (the dual-runtime law keeps the tracked rg reads inside the
    // mirrors' rg effects; see `mirror_rg_to_leptos`).
    let label_id_mirror =
        crate::field::validation_helpers::mirror_rg_to_leptos(&labelable.label_id.clone());
    let message_ids_mirror =
        crate::field::validation_helpers::mirror_rg_to_leptos(&labelable.message_ids.clone());
    let aria_labelledby_attr = move || label_id_mirror.get();

    // `const id = useLabelableId({ id: idProp })` (`:75`).
    let id_signal = use_labelable_id(UseLabelableIdParams {
        id: id_prop,
        enabled: true,
    });

    // `useControlled` (`:77-82`): the mode fixes from the initial prop's defined-ness
    // (packages/utils/src/useControlled.ts:41); the uncontrolled branch owns the
    // value.
    let is_controlled = value_prop.is_some();
    let value_signal: RwSignal<Option<String>> = RwSignal::new(match &value_prop {
        Some(value) => Some(value.clone()),
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

    // The control instance's identity token (the upstream `useRefWithInit(() =>
    // Symbol())` — one `ControlIdSource` per control, not per registration run).
    let source = ControlIdSource::new();
    let control_ref: Rc<Cell<Option<web_sys::Element>>> = Rc::new(Cell::new(None));

    // The element slot: upstream passes `validation.inputRef` straight into
    // `useRegisterFieldControl` (`FieldControl.tsx:135`), and `useRenderElement`'s
    // ref fork mounts the real `<input>` into that cell BEFORE any layout effect
    // reads it. Leptos `view!` has no ref fork, so the element arrives through a
    // `NodeRef` and a leptos Effect (which fires post-mount, then re-runs on an
    // element swap) resyncs the machine's inert `Rc<Cell>` slots — the closest
    // honest mirror of `ref` → `useIsoLayoutEffect` ordering.
    let input_node: NodeRef<leptos::html::Input> = NodeRef::new();
    {
        let input_ref = Rc::clone(&validation.input_ref);
        let control_ref = Rc::clone(&control_ref);
        Effect::new(move |_| {
            let Some(input) = input_node.get() else {
                return;
            };
            input_ref.set(Some(input.clone()));
            control_ref.set(Some(input.unchecked_into::<web_sys::Element>()));
        });
    }
    // The registration effect (`useRegisterFieldControl.ts:21-45`, re-homed): ONE
    // source token per control instance (the upstream `useRefWithInit(() =>
    // Symbol())` identity — minting a fresh token per run would mark every
    // re-registration a control *replacement* and cancel pending validation), and
    // the registration pushes in place on every value/name/disabled flip (the
    // tracked reads are upstream's hook dependency array; the layout effect → a
    // leptos effect). A `None` registration while disabled is the upstream
    // `enabled: false` registration skip. The `getValue` arm is the live DOM read
    // (`getValueForForm`, `:36-47`): the registry validates against the input's
    // current value, not the registration snapshot.
    {
        let register = field.register_field_control.clone();
        let control_ref = Rc::clone(&control_ref);
        let input_ref = Rc::clone(&validation.input_ref);
        let id_signal = id_signal.clone();
        let disabled_signal = disabled_signal.clone();
        let resolved_name = resolved_name.clone();
        let value_unwrapped = value_unwrapped.clone();
        Effect::new(move |_| {
            // Rebuilt per run: the effect body is FnMut, and a GetControlValueFn
            // closing over the shared slot can be constructed fresh each pass.
            let get_value = {
                let input_ref = Rc::clone(&input_ref);
                Rc::new(move || cell_peek(&input_ref).map(|input| Value::String(input.value())))
                    as leptos_ui_internals::field_register_control::GetControlValueFn
            };
            let registration = if disabled_signal.get() {
                None
            } else {
                Some(FieldControlRegistration {
                    control_ref: Rc::clone(&control_ref),
                    // The internals handle is an rg-0.2 signal — read through the
                    // fully-qualified accessor (the dual-runtime law).
                    id: Some(reactive_graph::traits::GetUntracked::get_untracked(
                        &id_signal,
                    )),
                    name: resolved_name.get(),
                    get_value: Some(get_value),
                    value: value_unwrapped.get().map(|text| Value::String(text)),
                })
            };
            register(source.clone(), registration);
        });
    }
    // The unmount unregistration (`:40-45`): the SAME source token, so the
    // unregistration clears this control's handover state. `on_cleanup` demands
    // `Send` — the Rc closures ride the `SendWrapper` bridge (the close_part.rs
    // convention).
    {
        let register = field.register_field_control.clone();
        let source = source.clone();
        let cleanup = send_wrapper::SendWrapper::new(move || {
            register(source, None);
        });
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }

    // Filled-on-mount sync (`:100-105`).
    {
        let value_unwrapped = value_unwrapped.clone();
        let input_ref = Rc::clone(&validation.input_ref);
        let set_filled = set_filled.clone();
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
        let set_dirty = set_dirty.clone();
        let validation_change = validation.change.clone();
        Effect::new(move |_| {
            let Some(serialized) = value_unwrapped.get() else {
                return;
            };
            if !is_controlled {
                return; // upstream's effect body early-returns on `undefined`
            }
            clear_errors(resolved_name.get().as_deref());
            set_dirty(
                serialized
                    != validity_data
                        .get_untracked()
                        .initial_value
                        .as_str()
                        .unwrap_or(""),
            );
            (validation_change)(Some(Value::String(serialized)), false);
        });
    }

    // autoFocus hydration (`:121-125`).
    {
        let control_ref = Rc::clone(&control_ref);
        let set_focused = set_focused.clone();
        Effect::new(move |_| {
            if !auto_focus {
                return;
            }
            let Some(element) = cell_peek(&control_ref) else {
                return;
            };
            let Some(node) = element.dyn_ref::<web_sys::Node>() else {
                return;
            };
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
                    != validity_data
                        .get_untracked()
                        .initial_value
                        .as_str()
                        .unwrap_or(""),
            );
            set_filled(!input_value.is_empty());
            value_signal.set(Some(input_value.clone()));

            // The native-prevention + cancel gate (`:155-158`).
            if !event.default_prevented() && !details.is_canceled() {
                clear_errors(resolved_name.get_untracked().as_deref());
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
        let set_touched = set_touched.clone();
        let set_focused = set_focused.clone();
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
                                != validity_data
                                    .get_untracked()
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
                        let value = cell_peek(&input_ref)
                            .map(|input| input.value())
                            .unwrap_or(value);
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
        move || {
            Some(reactive_graph::traits::GetUntracked::get_untracked(
                &id_signal,
            ))
        }
    };
    let name_attr = move || resolved_name.get();
    let value_attr = {
        let value_unwrapped = value_unwrapped.clone();
        move || value_unwrapped.get()
    };
    let disabled_attr = move || disabled_signal.get().then(|| String::new());
    let aria_invalid_attr = {
        let state = state.clone();
        move || {
            (state.valid.get() == Some(false) && !state.disabled.get() && !disabled_prop)
                .then(|| "true".to_string())
        }
    };
    // `getValidationProps(disabled, props)` (`:210`) — the `aria-describedby`
    // member: the labelable merge installs the message ids. The Rc handler's
    // tracked rg reads cannot run inside a leptos attribute closure (the
    // dual-runtime panic), so the merged value derives from the message-ids
    // mirror instead: the user's static value (the `elementProps` member —
    // behavior.md "user-provided aria-describedby values are preserved and
    // appended to") followed by this scope's message ids, first-occurrence
    // deduped, `join(' ') || undefined` (the provider's merge, `:68-81`).
    // Deviation (recorded): upstream's merge also folds the PARENT labelable
    // scope's message ids for controls nested in a `Field.Item`; the port's
    // context bag does not expose the parent chain, so a nested item-scoped
    // control merges only its own scope here.
    let user_describedby = element_attributes
        .iter()
        .find(|(name, _)| name == "aria-describedby")
        .map(|(_, value)| value.clone());
    let aria_describedby_attr = move || {
        let mut ids: Vec<String> = user_describedby
            .as_deref()
            .map(|value| value.split(' ').map(str::to_string).collect())
            .unwrap_or_default();
        ids.extend(message_ids_mirror.get());
        let mut seen = Vec::new();
        for id in ids {
            if !seen.contains(&id) {
                seen.push(id);
            }
        }
        let joined = seen.join(" ");
        (!joined.is_empty()).then_some(joined)
    };

    // The class merge: the state walk's `data-*` attributes bind separately (the
    // accordion precedent); `class` rides as-is.
    let (
        data_disabled_attr,
        data_touched_attr,
        data_dirty_attr,
        data_valid_attr,
        data_invalid_attr,
        data_filled_attr,
        data_focused_attr,
    ) = {
        let live = crate::field::parts_view::field_state_attributes(&state);
        (
            live.data_disabled,
            live.data_touched,
            live.data_dirty,
            live.data_valid,
            live.data_invalid,
            live.data_filled,
            live.data_focused,
        )
    };

    // The `...elementProps` rest bag (`:48`, the last-but-one layer): native props
    // (`required`, `type`, `minLength`, `pattern`, `readOnly` — behavior.md
    // "pass through to the rendered `<input>`") and plain attributes spread onto
    // the element AFTER the internal `id`/`name`/`aria-*` members but with the
    // user winning (the `useButton.ts:228` later-bag-wins rule). Leptos `view!`
    // has no attribute spread, so the bag lands through a mount effect writing
    // the real node — post-mount, re-applied on element swap (the node-ref effect
    // precedent above), and the `aria-describedby` member is intentionally
    // skipped (the merged binding above owns that attribute).
    {
        let input_node = input_node.clone();
        let element_attributes = element_attributes.clone();
        Effect::new(move |_| {
            let Some(input) = input_node.get() else {
                return;
            };
            let element: &web_sys::Element = input.unchecked_ref();
            for (name, value) in &element_attributes {
                if name == "aria-describedby" {
                    continue;
                }
                if value.is_empty() {
                    let _ = element.remove_attribute(name);
                } else {
                    let _ = element.set_attribute(name, value);
                }
            }
        });
    }

    view! {
        <input
            node_ref=input_node
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
    #[prop(default = None, optional)]
    id: Option<String>,
    /// The control's own name (`:41`).
    #[prop(default = None, optional)]
    name: Option<String>,
    /// The controlled value (`:42`).
    #[prop(default = None, optional)]
    value: Option<String>,
    /// The uncontrolled seed (`:46`).
    #[prop(default = None, optional)]
    default_value: Option<String>,
    /// `disabled` (`:43`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// The user's class.
    #[prop(default = None, optional)]
    class: Option<String>,
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
