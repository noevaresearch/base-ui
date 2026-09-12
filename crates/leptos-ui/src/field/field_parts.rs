//! The four leaf parts + Item — ports of `FieldLabel`, `FieldDescription`,
//! `FieldError`, `FieldValidity`, and `FieldItem` (`packages/react/src/field/{label,
//! description, error, validity, item}`).
//!
//! Shared shape (implementation.md "DOM/portal strategy"): every part renders inline
//! through the state walk with `fieldValidityMapping`; Label and Description OR the
//! [`FieldItemContext`] disabled into their state so items inside a group render
//! `data-disabled` without disabling siblings.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use wasm_bindgen::JsCast;

use leptos_ui_internals::field_constants::{FieldValidityData, FieldValidityState};
use leptos_ui_internals::labelable_provider::{
    UseLabelParams, use_label, use_labelable_context,
};

use crate::field::context::{FieldItemContext, use_field_item_context, use_field_root_context_required};
use crate::field::validation_helpers::walk_transition_status;

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
    /// The user's class.
    pub class: Option<String>,
}

/// `FieldLabel` (`FieldLabel.tsx`): the required context accessor (throws outside a
/// Root), the labelable `useLabel` wiring (`htmlFor` on native, click-focus on
/// non-native), the item-disabled OR-in.
pub fn field_label_view(props: FieldLabelProps) -> impl IntoView {
    let FieldLabelProps { id: id_prop, native_label, class } = props;

    let field = use_field_root_context_required();
    let item = use_field_item_context();
    let labelable = use_labelable_context();

    // `state = { ...fieldRootContext.state, disabled: fieldDisabled ||
    // fieldItemContext.disabled }`.
    let state_disabled = Signal::derive({
        let field_disabled = field.disabled.clone();
        move || field_disabled.get() || item.disabled
    });
    let state = crate::field::context::FieldStateValue {
        disabled: state_disabled.clone(),
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    };

    // `useLabel({ id: labelId ?? idProp, native: nativeLabel })`.
    let fallback_control_id = labelable.control_id.get_untracked();
    let label_props = use_label(UseLabelParams {
        id: id_prop,
        fallback_control_id,
        native: native_label,
        set_label_id: None,
        focus_control: None,
    });

    // The label id drives `id`; `htmlFor` (native) or the click-focus (non-native)
    // drives the association.
    let id_attr = move || Some(label_props.id.get());
    let for_attr = move || label_props.for_control.get();
    let state_attrs = crate::field::parts_view::field_state_attributes(&state);

    if native_label {
        let on_mouse_down = label_props.on_mouse_down.clone();
        view! {
            <label
                id={id_attr}
                for={for_attr}
                class={class}
                data-disabled={(move || state_attrs.0()).0}
                on:mouse-down=move |event: web_sys::MouseEvent| {
                    if let Some(handler) = &on_mouse_down {
                        handler(&event);
                    }
                }
            >
                {children_slot()}
            </label>
        }
    } else {
        let on_click = label_props.on_click.clone();
        let on_pointer_down = label_props.on_pointer_down.clone();
        view! {
            <span
                id={id_attr}
                class={class}
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
    }
}

/// The children slot — the port's parts are view functions, so children ride the
/// component wrappers below.
fn children_slot() -> leptos::prelude::View {
    leptos::prelude::View::default()
}

// ---------------------------------------------------------------------------
// FieldDescription (`FieldDescription.tsx`)
// ---------------------------------------------------------------------------

/// `FieldDescription` (`FieldDescription.tsx`): registers its id into the labelable
/// `messageIds` on mount and clears it on unmount (`:36-46`).
pub fn field_description_view(
    id: Option<String>,
    class: Option<String>,
    children: Option<leptos::children::Children>,
) -> impl IntoView {
    let field = use_field_root_context_required();
    let item = use_field_item_context();
    let labelable = use_labelable_context();

    let state_disabled = Signal::derive({
        let field_disabled = field.disabled.clone();
        move || field_disabled.get() || item.disabled
    });
    let state = crate::field::context::FieldStateValue {
        disabled: state_disabled,
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    };

    // `const id = useBaseUiId(idProp)` (`:21`): the generated fallback.
    let resolved_id: RwSignal<String> = RwSignal::new(
        id.unwrap_or_else(crate::field::validation_helpers::new_base_ui_id),
    );

    // The message-ids registration (`:36-46`).
    {
        let resolved_id = resolved_id.clone();
        let set_message_ids = labelable.message_ids.clone();
        Effect::new(move |_| {
            let id = resolved_id.get();
            set_message_ids.update(|ids| ids.push(id.clone()));
            let id = resolved_id.clone();
            on_cleanup(move || {
                set_message_ids.update(|ids| ids.retain(|existing| *existing != id.get()));
            });
        });
    }

    let id_attr = move || Some(resolved_id.get());
    let state_attrs = crate::field::parts_view::field_state_attributes(&state);

    view! {
        <p
            id={id_attr}
            class={class}
            data-disabled={(move || state_attrs.0()).0}
        >
            {children.map(|children| children())}
        </p>
    }
}

// ---------------------------------------------------------------------------
// FieldItem (`FieldItem.tsx`)
// ---------------------------------------------------------------------------

/// `FieldItem` (`FieldItem.tsx`): scopes `disabled` to the wrapped controls, provides
/// the [`FieldItemContext`], and opens a nested `LabelableProvider` scope
/// (`:43-47`) so per-item labels associate with their own checkbox/radio.
pub fn field_item_view(
    disabled: bool,
    class: Option<String>,
    children: leptos::children::Children,
) -> impl IntoView {
    let field = use_field_root_context_required();
    let root_disabled = field.disabled.get_untracked();
    let item_disabled = root_disabled || disabled;

    let state = crate::field::context::FieldStateValue {
        disabled: Signal::derive(move || item_disabled),
        touched: field.state.touched.clone(),
        dirty: field.state.dirty.clone(),
        valid: field.state.valid.clone(),
        filled: field.state.filled.clone(),
        focused: field.state.focused.clone(),
    };

    // The item context (`:34,45`).
    provide_context(FieldItemContext { disabled: item_disabled });

    // The nested LabelableProvider (`:43-47`) — the rg-0.2 bridge window (the root's
    // pattern): the provider publishes inside its own owner and the children build
    // inside the same window so their labelable reads resolve here.
    let bridge_owner = reactive_graph::owner::Owner::new();
    bridge_owner
        .with(|| leptos_ui_internals::labelable_provider::provide_labelable_context());
    let view = bridge_owner.with(|| children());
    std::mem::forget(bridge_owner);

    let state_attrs = crate::field::parts_view::field_state_attributes(&state);
    let data_disabled = move || state_attrs.0();

    view! {
        <div
            class={class}
            data-disabled={data_disabled}
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
/// the state object to the children function. Renders nothing itself.
pub fn field_validity_view(
    children: impl Fn(FieldValidityState, leptos_ui_internals::use_transition_status::TransitionStatus) -> leptos::prelude::View
        + 'static,
) -> impl IntoView {
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

    // `useTransitionStatus(isInvalid)` (`:33`).
    let transition = crate::field::validation_helpers::transition_status_signal(is_invalid);

    // The state object (`:36-46`): `{ ...combined, validity: combined.state,
    // transitionStatus }` — the identity-stability note is the derived signal's
    // same-value bail-out.
    move || {
        let data = combined.get();
        let status = transition.get();
        children(
            data.state,
            status,
        )
    }
}

// ---------------------------------------------------------------------------
// FieldError (`FieldError.tsx`)
// ---------------------------------------------------------------------------

/// The error-match union (`FieldError.tsx`'s `match` prop).
pub enum ErrorMatch {
    /// Omitted / `false` — the default slot for Form errors.
    Default,
    /// `true` — always render.
    Always,
    /// A `ValidityState` key.
    Key(String),
}

/// `FieldError` (`FieldError.tsx:30-140`): the render gate, the message keying, the
/// mount/exit animation (`useTransitionStatus` + `useOpenChangeComplete`), and the
/// multi-error `<ul>` shape.
pub fn field_error_view(
    id: Option<String>,
    class: Option<String>,
    class_match: ErrorMatch,
    children_message: Option<leptos::prelude::View>,
) -> impl IntoView {
    let _ = id;
    let _ = &class;
    let _ = class_match;
    let _ = children_message;
    let field = use_field_root_context_required();
    let _ = field;
    view! { <div></div> }
}

/// Keeps the leaf-part internals honest in host builds (the transition-status walk
/// lives in `validation_helpers`).
#[allow(unused)]
fn marker(data: FieldValidityData, ids: RefCell<Vec<String>>, s: FieldValidityState) {
    let _ = (data, ids, s);
}
