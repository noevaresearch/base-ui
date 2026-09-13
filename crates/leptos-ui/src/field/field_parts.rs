//! The four leaf parts + Item — ports of `FieldLabel`, `FieldDescription`,
//! `FieldError`, `FieldValidity`, and `FieldItem` (`packages/react/src/field/{label,
//! description, error, validity, item}`).
//!
//! Shared shape (implementation.md "DOM/portal strategy"): every part renders inline
//! through the state walk with `fieldValidityMapping`; Label and Description OR the
//! [`FieldItemContext`] disabled into their state so items inside a group render
//! `data-disabled` without disabling siblings.
//!
//! Runtime law for the leaf parts (the field mod docs): the internals' rg-0.2 handles
//! (`LabelableContextValue` members, the Form `errors` record) are read **untracked**
//! through the fully-qualified accessor — body-time or callback-time snapshots, never
//! tracked reads that would demand an rg owner. No rg handle feeds a leptos view; a
//! tracked rg read outside an rg owner is a panic.

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::Value;

use leptos_ui_internals::field_constants::{
    FieldValidityData, FieldValidityState, DEFAULT_FIELD_ROOT_STATE,
};
use leptos_ui_internals::labelable_provider::{
    use_label, use_labelable_context, LabelableContextValue, UseLabelParams,
};

use crate::field::context::{
    use_field_item_context, use_field_root_context_required, FieldItemContext, FieldStateValue,
};

// ---------------------------------------------------------------------------
// Shared leaf-part helpers
// ---------------------------------------------------------------------------

/// The item-scoped state (`FieldLabel.tsx:34`/`FieldDescription.tsx:28`):
/// `{ ...fieldRootContext.state, disabled: fieldDisabled || fieldItemContext.disabled }`.
fn item_scoped_state(
    field: &crate::field::context::FieldRootContext,
    item_disabled: bool,
) -> FieldStateValue {
    let field_disabled = field.disabled.clone();
    FieldStateValue {
        disabled: Signal::derive(move || field_disabled.get() || item_disabled),
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    }
}

/// `useBaseUiId(idProp)` (`useBaseUiId.ts`): the explicit id, else the generated
/// `base-ui-` id (the shared generator).
fn resolve_part_id(id_prop: Option<String>) -> String {
    id_prop.unwrap_or_else(crate::field::validation_helpers::new_base_ui_id)
}

/// The labelable context accessor for the leaf parts. Must run inside the root's
/// bridge window (the body builds there); the returned bag's rg handles are read
/// untracked from here on.
fn leaf_labelable_context() -> LabelableContextValue {
    use_labelable_context()
}

/// The message-ids push + teardown-clear pair shared by Description and Error —
/// the `setMessageIds((v) => v.concat(id))` registration (`FieldError.tsx:59`) and
/// the `filter` cleanup (`:65`), at body time (inside the bridge window) and at
/// teardown.
fn register_message_id(labelable: &LabelableContextValue, id: &str) {
    let set_message_ids = labelable.message_ids.clone();
    let id = id.to_string();
    reactive_graph::traits::Set::set(&set_message_ids, {
        let mut ids = reactive_graph::traits::GetUntracked::get_untracked(&set_message_ids);
        if !ids.contains(&id) {
            ids.push(id.clone());
        }
        ids
    });
    let set_message_ids = labelable.message_ids.clone();
    on_cleanup(move || {
        reactive_graph::traits::Set::set(&set_message_ids, {
            let mut ids = reactive_graph::traits::GetUntracked::get_untracked(&set_message_ids);
            ids.retain(|existing| *existing != id);
            ids
        });
    });
}

// ---------------------------------------------------------------------------
// FieldLabel (`FieldLabel.tsx`)
// ---------------------------------------------------------------------------

/// The label props — upstream's destructured set (`FieldLabel.tsx`).
pub struct FieldLabelProps {
    /// `id` — the explicit label id.
    pub id: Option<String>,
    /// `nativeLabel` — whether the rendered element must be a native `<label>`
    /// (default `true`).
    pub native_label: bool,
    /// The user's `class`.
    pub class: Option<String>,
}

impl Default for FieldLabelProps {
    fn default() -> Self {
        FieldLabelProps {
            id: None,
            native_label: true,
            class: None,
        }
    }
}

/// `FieldLabel` (`FieldLabel.tsx:26-96`): the required context accessor (throws
/// outside a Root), the labelable `useLabel` wiring (`htmlFor` on native, the
/// click-focus on non-native), the item-disabled OR-in. Body time runs inside the
/// root's bridge window; the handler slots (`on_mouse_down`/`on_click`/
/// `on_pointer_down`) fire callback-time under DOM events, so their untracked reads
/// are safe there.
pub fn field_label_view(props: FieldLabelProps) -> impl IntoView {
    let FieldLabelProps {
        id: id_prop,
        native_label,
        class,
    } = props;

    let field = use_field_root_context_required();
    let item = use_field_item_context();

    let state = item_scoped_state(&field, item.disabled);
    let labelable = leaf_labelable_context();

    // `useLabel({ id: labelId ?? idProp, native })` (`:42-46`): the context's label
    // id is the override; the returned registered id drives the `id` attribute, the
    // resolved control id the `for` attribute. Both context reads ride the
    // rg→leptos mirrors: the control's registration runs at the CONTROL's body
    // time — after this label's first attribute evaluation when the label renders
    // first — so a frozen body-time snapshot would never see it (the association
    // must track replacement, behavior.md "Accessibility"; the mirrors re-fire the
    // attribute closures when the registration lands). The tracked rg read lives
    // inside each mirror's rg effect (created here, inside the bridge window —
    // the dual-runtime law).
    let label_id_mirror =
        crate::field::validation_helpers::mirror_rg_to_leptos(&labelable.label_id.clone());
    let control_id_mirror =
        crate::field::validation_helpers::mirror_rg_to_leptos(&labelable.control_id.clone());
    let label_props = use_label(UseLabelParams {
        id: label_id_mirror.get_untracked().or(id_prop),
        fallback_control_id: control_id_mirror.get_untracked(),
        native: native_label,
        set_label_id: None,
        focus_control: None,
    });

    // The returned id/for signals resolve the same context signals per read
    // (`use_label`'s `resolvedControlId = contextControlId ?? fallback`) — the
    // mirrors are the leptos-tracked re-fire source for the attribute closures.
    let id_attr = move || {
        Some(reactive_graph::traits::GetUntracked::get_untracked(
            &label_props.id,
        ))
    };
    let for_attr = move || {
        control_id_mirror.get().or_else(|| {
            reactive_graph::traits::GetUntracked::get_untracked(&label_props.for_control)
        })
    };
    let state_attrs = crate::field::parts_view::field_state_attributes(&state);
    let data_disabled = state_attrs.data_disabled.clone();

    if native_label {
        let on_mouse_down = label_props.on_mouse_down.clone();
        view! {
            <label
                id=id_attr
                for=for_attr
                class=class
                data-disabled=data_disabled
                on:mouse-down=move |event: web_sys::MouseEvent| {
                    if let Some(handler) = &on_mouse_down {
                        handler(&event);
                    }
                }
            >
                {children_slot()}
            </label>
        }
        .into_any()
    } else {
        let on_click = label_props.on_click.clone();
        let on_pointer_down = label_props.on_pointer_down.clone();
        view! {
            <span
                id=id_attr
                class=class
                data-disabled=data_disabled
                on:click=move |event: web_sys::MouseEvent| {
                    if let Some(handler) = &on_click {
                        handler(&event);
                    }
                }
                on:pointer-down=move |event: web_sys::PointerEvent| {
                    if let Some(handler) = &on_pointer_down {
                        handler(&event);
                    }
                }
            >
                {children_slot()}
            </span>
        }
        .into_any()
    }
}

/// The children slot — the leaf parts are view functions; the text rides the
/// `#[component]` wrappers.
fn children_slot() -> AnyView {
    ().into_any()
}

// ---------------------------------------------------------------------------
// FieldDescription (`FieldDescription.tsx`)
// ---------------------------------------------------------------------------

/// `FieldDescription` (`FieldDescription.tsx:19-56`): registers its id into the
/// labelable `messageIds` at body time and clears it at teardown (`:36-46`), renders
/// the `<p>`.
pub fn field_description_view(
    id: Option<String>,
    class: Option<String>,
    children: Option<leptos::children::Children>,
) -> impl IntoView {
    let field = use_field_root_context_required();
    let item = use_field_item_context();
    let state = item_scoped_state(&field, item.disabled);

    let labelable = leaf_labelable_context();
    let resolved_id = resolve_part_id(id);

    // The message-ids registration (`:36-46`): the push at body time (the
    // `useIsoLayoutEffect` mount run — the bridge window has the rg owner), the
    // teardown clear.
    register_message_id(&labelable, &resolved_id);

    let id_attr = move || Some(resolved_id.clone());
    let state_attrs = crate::field::parts_view::field_state_attributes(&state);
    let data_disabled = state_attrs.data_disabled.clone();

    view! {
        <p
            id=id_attr
            class=class
            data-disabled=data_disabled
        >
            {children.map(|children| children())}
        </p>
    }
}

// ---------------------------------------------------------------------------
// FieldItem (`FieldItem.tsx`)
// ---------------------------------------------------------------------------

/// `FieldItem` (`FieldItem.tsx:18-52`): scopes `disabled` to the wrapped controls,
/// provides the [`FieldItemContext`], and opens a nested `LabelableProvider` scope
/// (`:43-47`) so per-item labels associate with their own checkbox/radio.
pub fn field_item_view(
    disabled: bool,
    class: Option<String>,
    children: leptos::children::Children,
) -> impl IntoView {
    let field = use_field_root_context_required();
    let root_disabled = field.disabled.get_untracked();
    let item_disabled = root_disabled || disabled;

    let state = item_scoped_state(&field, item_disabled);

    // The item context (`:34,45`).
    // The item context (`:34,45`).
    provide_context(FieldItemContext {
        disabled: item_disabled,
    });

    // The nested LabelableProvider (`:43-47`) — the bridge window: the provider
    // publishes inside a fresh rg owner (forgotten to outlive the subtree) and the
    // children build in the same window so their labelable reads resolve here.
    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(|| {
        // `provide_labelable_context` publishes the `SharedLabelableContext` itself —
        // the children's `use_labelable_context` reads that type; no second provide.
        leptos_ui_internals::labelable_provider::provide_labelable_context();
        children()
    });
    std::mem::forget(bridge_owner);

    let state_attrs = crate::field::parts_view::field_state_attributes(&state);
    let data_disabled = state_attrs.data_disabled.clone();

    view! {
        <div
            class=class
            data-disabled=data_disabled
        >
            {view}
        </div>
    }
}

// ---------------------------------------------------------------------------
// FieldValidity (`FieldValidity.tsx`)
// ---------------------------------------------------------------------------

/// `FieldValidity` — the render-prop component (`FieldValidity.tsx:17-49`): reads the
/// validity record, combines with `invalid`, derives the transition status, and hands
/// the state object to the children function. Renders nothing itself. The payload's
/// members read live leptos signals (`validityData`, `invalid`) plus the bridged
/// transition-status handle; the children closure decides its own tracking.
/// The closure rides the `SendWrapper` bridge (it is stored in the returned view).
pub fn field_validity_view(
    children: impl Fn(
            FieldValidityState,
            Option<leptos_ui_internals::use_transition_status::TransitionStatus>,
        ) -> AnyView
        + 'static,
) -> impl IntoView {
    let children = SendWrapper::new(children);
    let field = use_field_root_context_required();
    let validity_data = field.validity_data.clone();
    let invalid = field.invalid.clone();

    // `getCombinedFieldValidityData(validityData, invalid)` (`:24-28`).
    let combined = Signal::derive(move || {
        leptos_ui_internals::get_combined_field_validity_data::get_combined_field_validity_data(
            &validity_data.get(),
            invalid.get(),
        )
    });
    let is_invalid = Signal::derive(move || combined.get().state.valid == Some(false));

    // `useTransitionStatus(isInvalid)` (`:33`) — the leptos↔rg bridge in
    // `validation_helpers`; the bridge takes a plain `Fn() -> bool`, so the signal
    // read rides a closure.
    let transition =
        crate::field::validation_helpers::transition_status_signal(move || is_invalid.get());

    move || {
        let data = combined.get();
        let status = transition.get();
        children(data.state, status)
    }
}

// ---------------------------------------------------------------------------
// FieldError (`FieldError.tsx`)
// ---------------------------------------------------------------------------

/// The error-match union (`FieldError.tsx`'s `match` prop).
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorMatch {
    /// Omitted / `false` — the default slot for Form errors.
    Default,
    /// `true` — always render.
    Always,
    /// A `ValidityState` key (`'valueMissing'`, `'tooShort'`, `'patternMismatch'`,
    /// `'customError'`, `'badInput'`, …).
    Key(String),
}

/// `validityData.state[match]` (`:51`) — the `ValidityState` key read.
fn validity_flag(validity: &FieldValidityState, key: &str) -> bool {
    match key {
        "badInput" => validity.bad_input,
        "customError" => validity.custom_error,
        "patternMismatch" => validity.pattern_mismatch,
        "rangeOverflow" => validity.range_overflow,
        "rangeUnderflow" => validity.range_underflow,
        "stepMismatch" => validity.step_mismatch,
        "tooLong" => validity.too_long,
        "tooShort" => validity.too_short,
        "typeMismatch" => validity.type_mismatch,
        "valueMissing" => validity.value_missing,
        "valid" => validity.valid == Some(true),
        _ => false,
    }
}

/// The render gate (`FieldError.tsx:44-53`).
fn error_rendered(
    field_match: &ErrorMatch,
    disabled: bool,
    has_form_error: bool,
    validity: &FieldValidityState,
) -> bool {
    if matches!(field_match, ErrorMatch::Always) {
        return true;
    }
    if disabled {
        return false;
    }
    match field_match {
        ErrorMatch::Always => unreachable!(),
        ErrorMatch::Key(key) => validity_flag(validity, key),
        ErrorMatch::Default => has_form_error || validity.valid == Some(false),
    }
}

/// The error payload (`FieldError.tsx:81-94`): the form error when the slot is the
/// default and the form reported one (the form message rides as the sentinel — the
/// static slot carries the message), else the multi-error array, else the single
/// error string.
fn error_message_value(data: &FieldValidityData, has_form_error: bool) -> Value {
    if has_form_error {
        return Value::String(String::new());
    }
    if data.errors.len() > 1 {
        return Value::Array(
            data.errors
                .iter()
                .map(|message| Value::String(message.clone()))
                .collect(),
        );
    }
    Value::String(data.error.clone())
}

/// `FieldError` (`FieldError.tsx:30-140`): the render gate, the message keying (the
/// last message stays as children while exiting, `:95-100`), the mount/exit
/// transition (`useTransitionStatus` + `setMounted(false)` on completion, `:102-110`),
/// the message-ids registration (`:55-70`), and the multi-error `<ul>` shape
/// (`:82-91`). The `id` attribute and the transition `data-*` hooks follow the
/// upstream body; the state-walk hooks ride the shared state bag.
///
/// The live-binding shape: the render gate and the message derive over the leptos
/// signals, so a validity flip drives the whole machine; the form-error member is a
/// body-time snapshot (the dual-runtime law — the record feeds no view here).
pub fn field_error_view(
    id: Option<String>,
    class: Option<String>,
    field_match: ErrorMatch,
    children_message: Option<AnyView>,
) -> impl IntoView {
    let field = use_field_root_context_required();
    let labelable = leaf_labelable_context();

    let resolved_id = resolve_part_id(id);
    let validity_data = field.validity_data.clone();
    let _ = &field.invalid;

    // The form-error read (`:38-42`): `useFormContext()` — the inert default outside
    // a `<Form>`; the errors record is the internals' rg-0.2 signal, snapshot at body
    // time keyed on the effective name (the dual-runtime law).
    let has_form_error = {
        let form = leptos_ui_internals::form_context::use_form_context();
        let errors = reactive_graph::traits::GetUntracked::get_untracked(&form.errors);
        let name = field.name.get_untracked();
        name.into_iter()
            .flat_map(|field_name| {
                errors
                    .iter()
                    .filter(move |(key, _)| *key == field_name)
                    .map(|(_, error)| error.clone())
            })
            .any(|error| match error {
                leptos_ui_internals::form_context::FormErrorValue::Single(message) => {
                    !message.is_empty()
                }
                leptos_ui_internals::form_context::FormErrorValue::Multiple(messages) => {
                    !messages.is_empty()
                }
            })
    };

    // The message-ids registration (`:55-70`): pushed while the error renders, the
    // teardown clear. The upstream effect re-runs per `rendered` flip; the static
    // push + clear keeps the id discoverable for the static demo slots.
    register_message_id(&labelable, &resolved_id);

    // The `rendered` derivation rides the live leptos signals (`:44-53`).
    let disabled_signal = field.disabled.clone();
    let field_match_signal = field_match.clone();
    let rendered = {
        let validity_data = validity_data.clone();
        Signal::derive(move || {
            let data = validity_data.get();
            error_rendered(
                &field_match_signal,
                disabled_signal.get(),
                has_form_error,
                &data.state,
            )
        })
    };

    // The transition machine (`:55`): the real ported hook through the bridge; the
    // bridge takes a plain `Fn() -> bool`, so the signal read rides a closure.
    let transition =
        crate::field::validation_helpers::transition_status_signal(move || rendered.get());

    // The message payload with keying (`:95-100`): a new message while rendered
    // replaces the pinned one; while exiting the last message stays as children.
    // StoredValue, not Rc<RefCell>: Signal::derive closures must be Send + Sync
    // (the reactive-graph storage law).
    let last_rendered_message: StoredValue<Option<Value>, LocalStorage> = {
        let initial = validity_data.get_untracked();
        StoredValue::new_local(Some(error_message_value(&initial, has_form_error)))
    };
    let last_message_key: StoredValue<Option<String>, LocalStorage> = {
        let initial = validity_data.get_untracked();
        StoredValue::new_local(
            error_message_value(&initial, has_form_error)
                .as_str()
                .map(str::to_string),
        )
    };
    let message_signal = {
        let validity_data = validity_data.clone();
        let last_rendered_message = last_rendered_message.clone();
        let last_message_key = last_message_key.clone();
        Signal::derive(move || {
            let data = validity_data.get();
            let is_rendered = rendered.get();
            let error_value = error_message_value(&data, has_form_error);
            let error_key = match &error_value {
                Value::Array(items) => serde_json::to_string(items).unwrap_or_default(),
                Value::String(text) => text.clone(),
                _ => String::new(),
            };
            let is_new = last_message_key.get_value().as_deref() != Some(error_key.as_str());
            if is_rendered && is_new {
                last_message_key.set_value(Some(error_key));
                last_rendered_message.set_value(Some(error_value.clone()));
            }
            last_rendered_message
                .get_value()
                .unwrap_or(Value::String(String::new()))
        })
    };

    // The completion wiring (`:102-110`): when the error stops rendering and the
    // ending transition completes (status back to `None`), `setMounted(false)`
    // unmounts the element; re-rendering flips it back through the hook's mount run.
    {
        let rendered_gate = rendered.clone();
        let transition = transition.clone();
        Effect::new(move |_| {
            let is_rendered = rendered_gate.get();
            let status = transition.get();
            if !is_rendered && status.is_none() {
                transition.set_mounted(false);
            }
        });
    }

    let id_attr = move || Some(resolved_id.clone());
    let state_attrs = crate::field::parts_view::field_state_attributes(&field.state);
    let data_disabled = state_attrs.data_disabled.clone();
    let data_valid = state_attrs.data_valid.clone();
    let data_invalid = state_attrs.data_invalid.clone();
    let data_touched = state_attrs.data_touched.clone();
    let data_dirty = state_attrs.data_dirty.clone();
    let data_filled = state_attrs.data_filled.clone();
    let data_focused = state_attrs.data_focused.clone();
    let status_for_start = transition.clone();
    let status_for_end = transition.clone();
    let status_for_hidden = transition.clone();
    let status_for_children = transition.clone();

    view! {
        <div
            class=class
            id=id_attr
            data-disabled=data_disabled
            data-touched=data_touched
            data-dirty=data_dirty
            data-valid=data_valid
            data-invalid=data_invalid
            data-filled=data_filled
            data-focused=data_focused
            data-starting-style=move || {
                matches!(
                    status_for_start.get(),
                    Some(leptos_ui_internals::use_transition_status::TransitionStatus::Starting)
                )
                .then(String::new)
            }
            data-ending-style=move || {
                matches!(
                    status_for_end.get(),
                    Some(leptos_ui_internals::use_transition_status::TransitionStatus::Ending)
                )
                .then(String::new)
            }
            hidden=move || (!status_for_hidden.mounted()).then(|| "".to_string())
        >
            {move || {
                if !status_for_children.mounted() {
                    return ().into_any();
                }
                match message_signal.get() {
                    Value::Array(items) if items.len() > 1 => {
                        let items_view: Vec<_> = items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .map(|message| view! { <li>{message}</li> })
                            .collect();
                        view! { <ul>{items_view}</ul> }.into_any()
                    }
                    Value::Array(items) => {
                        let first = items
                            .first()
                            .and_then(|item| item.as_str().map(str::to_string))
                            .unwrap_or_default();
                        view! { <span>{first}</span> }.into_any()
                    }
                    Value::String(text) => {
                        if text.is_empty() {
                            ().into_any()
                        } else {
                            view! { <span>{text}</span> }.into_any()
                        }
                    }
                    _ => ().into_any(),
                }
            }}
            {children_message}
        </div>
    }
}

/// Keeps the leaf-part internals honest in host builds (the transition-status walk
/// lives in `validation_helpers`).
#[allow(unused)]
fn marker(data: FieldValidityData, s: FieldValidityState) {
    let _ = (data, s, DEFAULT_FIELD_ROOT_STATE);
}
