//! Switch state model — the derivations, the state→attribute walk, and the
//! change funnel of `packages/react/src/switch/`
//! (the `library: switch` TODO item; `specs/library/switch/behavior.md`,
//! `specs/library/switch/implementation.md`).
//!
//! Everything here is view-independent and host-testable: the record upstream
//! memoizes as `SwitchRootState` (`packages/react/src/switch/root/SwitchRoot.tsx:207-216`),
//! the `stateAttributesMapping` walk
//! (`packages/react/src/switch/stateAttributesMapping.ts:6-16`), the pure folds the
//! component's body performs before rendering (the `disabled`/`name` composition at
//! `SwitchRoot.tsx:73-74`, the labelable/instance id split at `:81-84`, the hidden
//! input's recipe at `:167`, and the change funnel at `:172-193`).
//!
//! Runtime split (the crate's dual-runtime law — see `field/mod.rs`,
//! `checkbox/state.rs`): the internals crate's hooks are typed over reactive-graph
//! 0.2 while leptos 0.7's view tree tracks 0.1. This module therefore keeps only the
//! *view-independent* vocabulary: the pure folds, the attribute walk over the ported
//! engine ([`get_state_attributes_props`] + [`field_validity_mapping`]), and the
//! event/`visuallyHidden` constants.

use std::collections::BTreeMap;

use serde_json::Value;

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::state_attributes::{
    StateAttributeProps, StateAttributesMapping, field_validity_mapping, get_state_attributes_props,
};

/// The change-event details type — upstream's
/// `BaseUIChangeEventDetails<SwitchRoot.ChangeEventReason>` with
/// `ChangeEventReason = REASONS.none` (`SwitchRoot.tsx:326`) and the native input
/// event as the payload (`SwitchRoot.tsx:184`).
pub type SwitchChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::Event>;

/// `SwitchRootDataAttributes.checked` (`SwitchRootDataAttributes.ts:4`).
pub const DATA_CHECKED: &str = "data-checked";
/// `SwitchRootDataAttributes.unchecked` (`SwitchRootDataAttributes.ts:8`).
pub const DATA_UNCHECKED: &str = "data-unchecked";
/// `SwitchRootDataAttributes.disabled` (`SwitchRootDataAttributes.ts:12`).
pub const DATA_DISABLED: &str = "data-disabled";
/// `SwitchRootDataAttributes.readonly` (`SwitchRootDataAttributes.ts:16`).
pub const DATA_READONLY: &str = "data-readonly";
/// `SwitchRootDataAttributes.required` (`SwitchRootDataAttributes.ts:20`).
pub const DATA_REQUIRED: &str = "data-required";
/// `SwitchRootDataAttributes.valid` (`SwitchRootDataAttributes.ts:24`).
pub const DATA_VALID: &str = "data-valid";
/// `SwitchRootDataAttributes.invalid` (`SwitchRootDataAttributes.ts:28`).
pub const DATA_INVALID: &str = "data-invalid";
/// `SwitchRootDataAttributes.touched` (`SwitchRootDataAttributes.ts:32`).
pub const DATA_TOUCHED: &str = "data-touched";
/// `SwitchRootDataAttributes.dirty` (`SwitchRootDataAttributes.ts:36`).
pub const DATA_DIRTY: &str = "data-dirty";
/// `SwitchRootDataAttributes.filled` (`SwitchRootDataAttributes.ts:40`).
pub const DATA_FILLED: &str = "data-filled";
/// `SwitchRootDataAttributes.focused` (`SwitchRootDataAttributes.ts:44`).
pub const DATA_FOCUSED: &str = "data-focused";

/// The `data-*` members this unit's walk can emit — the names the writer binds.
/// Every other `data-*` a consumer passes rides the `...elementProps` rest
/// untouched (`SwitchRoot.tsx:55`).
pub const MANAGED_STATE_ATTRIBUTES: [&str; 11] = [
    DATA_CHECKED,
    DATA_UNCHECKED,
    DATA_DISABLED,
    DATA_READONLY,
    DATA_REQUIRED,
    DATA_VALID,
    DATA_INVALID,
    DATA_TOUCHED,
    DATA_DIRTY,
    DATA_FILLED,
    DATA_FOCUSED,
];

/// `SwitchRootState` (`SwitchRoot.tsx:241-258`): `FieldRootState` spread plus the
/// four switch members. This is the record handed to parts through
/// `SwitchRootContext` (`SwitchRootContext.ts:6` — the context value *is* the state
/// object), which is why Root and Thumb always agree on their style hooks
/// (`SwitchRoot.test.tsx:412-440`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchRootState {
    /// `checked` (`:245`) — the resolved (controlled-or-uncontrolled) state.
    pub checked: bool,
    /// `disabled` (`:249`) — the Field-folded flag (`:73`).
    pub disabled: bool,
    /// `readOnly` (`:253`).
    pub read_only: bool,
    /// `required` (`:257`).
    pub required: bool,
    /// `...fieldState.touched`.
    pub touched: bool,
    /// `...fieldState.dirty`.
    pub dirty: bool,
    /// `...fieldState.valid` — `None` is upstream's `null` (unvalidated).
    pub valid: Option<bool>,
    /// `...fieldState.filled`.
    pub filled: bool,
    /// `...fieldState.focused`.
    pub focused: bool,
}

impl SwitchRootState {
    /// The `serde_json` state map [`get_state_attributes_props`] walks
    /// (`packages/react/src/internals/getStateAttributesProps.ts:12`). The keys are
    /// upstream's member names: the walk lowercases them (`readOnly` → `data-readonly`,
    /// `valid` → handled by the mapping) and the custom mapping intercepts `checked`.
    pub fn to_state_map(self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("checked".to_string(), Value::Bool(self.checked));
        map.insert("disabled".to_string(), Value::Bool(self.disabled));
        map.insert("readOnly".to_string(), Value::Bool(self.read_only));
        map.insert("required".to_string(), Value::Bool(self.required));
        map.insert("touched".to_string(), Value::Bool(self.touched));
        map.insert("dirty".to_string(), Value::Bool(self.dirty));
        map.insert(
            "valid".to_string(),
            match self.valid {
                Some(valid) => Value::Bool(valid),
                None => Value::Null,
            },
        );
        map.insert("filled".to_string(), Value::Bool(self.filled));
        map.insert("focused".to_string(), Value::Bool(self.focused));
        map
    }
}

/// The `stateAttributesMapping` (`stateAttributesMapping.ts:6-16`): `fieldValidityMapping`
/// spread in, with the `checked` member replaced by the mutually exclusive
/// `data-checked`/`data-unchecked` pair (`:8-14`) — `checked` truthy emits
/// `data-checked=""`, falsy emits `data-unchecked=""`, and, unlike the checkbox's
/// `indeterminate` arm, this mapping has no third state to consume the field with.
pub fn get_switch_state_attributes_mapping() -> impl Fn(&str, &Value) -> Option<Option<StateAttributeProps>>
{
    |key: &str, value: &Value| {
        if key == "checked" {
            let attribute = if value == &Value::Bool(true) {
                DATA_CHECKED
            } else {
                DATA_UNCHECKED
            };
            return Some(Some(BTreeMap::from([(
                attribute.to_string(),
                String::new(),
            )])));
        }

        field_validity_mapping(key, value)
    }
}

/// The full walk over one snapshot — the ported engine
/// ([`get_state_attributes_props`]) driven by this unit's mapping.
pub fn switch_state_attributes(state: &SwitchRootState) -> StateAttributeProps {
    let state_map = state.to_state_map();
    let mapping = get_switch_state_attributes_mapping();
    get_state_attributes_props(&state_map, Some(&mapping as &StateAttributesMapping))
}

// ---------------------------------------------------------------------------
// The pure folds
// ---------------------------------------------------------------------------

/// `disabled = fieldDisabled || disabledProp` (`SwitchRoot.tsx:73`). The Field read
/// is a live signal at the call site; this fold is the boolean half.
pub fn effective_disabled(field_disabled: bool, disabled_prop: bool) -> bool {
    field_disabled || disabled_prop
}

/// `name = fieldName ?? nameProp` (`SwitchRoot.tsx:74`) — nullish, so a Field-provided
/// name wins over the prop, and an explicitly-null Field name falls through.
pub fn effective_name(field_name: Option<String>, name_prop: Option<String>) -> Option<String> {
    field_name.or(name_prop)
}

/// The `id` split (`SwitchRoot.tsx:81-84`): `id` is the internal instance id used by the
/// visible element in the default (non-native) mode, `controlId` is the labelable id;
/// the hidden input carries `hiddenInputId = nativeButton ? undefined : controlId`.
pub fn hidden_input_id(native_button: bool, control_id: &str) -> Option<String> {
    if native_button {
        None
    } else {
        Some(control_id.to_string())
    }
}

/// The visible element's `id` (`SwitchRoot.tsx:119`): `id: nativeButton ? controlId : id`
/// — with `nativeButton` the visible element *is* the labelable control
/// (`SwitchRoot.test.tsx:504-524`).
pub fn root_id(native_button: bool, control_id: &str, generated_id: &str) -> String {
    if native_button {
        control_id.to_string()
    } else {
        generated_id.to_string()
    }
}

/// The hidden input's `style` recipe (`SwitchRoot.tsx:167`): a named input (the one
/// that participates in form submission) is laid out with the `visuallyHiddenInput`
/// recipe, the nameless one with plain `visuallyHidden`.
pub fn input_style(name: Option<&str>) -> &'static [(&'static str, &'static str)] {
    if name.is_some() {
        leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN_INPUT
    } else {
        leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN
    }
}

/// The `useValueChanged` dirty fold (`SwitchRoot.tsx:101`): `checked !==
/// validityData.initialValue`.
pub fn checked_is_dirty(checked: bool, initial_value: &Value) -> bool {
    match initial_value {
        Value::Bool(initial) => checked != *initial,
        // An unset initial value (`null`) is not equal to any boolean — upstream's
        // `!==` against `null` is true for both booleans, but the seed value for an
        // unchecked field is `false`, so the arm above is what real flows take.
        Value::Null => false,
        _ => true,
    }
}

/// The `aria-*` presence rule this unit shares for `readOnly`/`required`
/// (`SwitchRoot.tsx:122-123`): the boolean renders as the literal `"true"` only when
/// set, and is absent otherwise (React's boolean-attribute rule).
pub fn aria_bool_attr(flag: bool) -> Option<String> {
    flag.then(|| "true".to_string())
}

// ---------------------------------------------------------------------------
// The change funnel (`SwitchRoot.tsx:172-193`)
// ---------------------------------------------------------------------------

/// The change funnel's outcome for one hidden-input change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeFunnel {
    /// The native event arrived default-prevented — ignored entirely
    /// (`SwitchRoot.tsx:174-176`, the React #9023 workaround).
    DefaultPrevented,
    /// `readOnly`: the change is re-prevented and nothing is committed (`:178-181`).
    ReadOnly,
    /// `onCheckedChange` cancelled — the state does not commit (`:188-190`).
    Canceled,
    /// Accepted: the local state commits (`:192`).
    Commit,
}

/// Runs the funnel gates in upstream's order and returns the outcome.
pub fn run_change_funnel(
    read_only: bool,
    default_prevented: bool,
    on_checked_change: Option<&dyn Fn(bool, &SwitchChangeEventDetails)>,
    next_checked: bool,
    details: &SwitchChangeEventDetails,
) -> ChangeFunnel {
    if default_prevented {
        return ChangeFunnel::DefaultPrevented;
    }

    if read_only {
        return ChangeFunnel::ReadOnly;
    }

    if let Some(callback) = on_checked_change {
        callback(next_checked, details);
    }

    if details.is_canceled() {
        return ChangeFunnel::Canceled;
    }

    ChangeFunnel::Commit
}

/// A `REASONS.none` change details — the port of
/// `createChangeEventDetails(REASONS.none, event.nativeEvent)` (`SwitchRoot.tsx:184`) for the
/// call sites that construct rather than forward an event.
///
/// The native-event slot wraps a plain `JsValue` rather than invoking the wasm-bindgen
/// `Event` constructor: the host target has no JS runtime and constructing a real `Event`
/// panics outside wasm (the `menu_tests.rs`/`dialog_tests.rs` convention). Production call
/// sites always have a real event and build their details from it (`root.rs`).
pub fn change_event_details() -> SwitchChangeEventDetails {
    BaseUIChangeEventDetails::new(
        reasons::NONE,
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
        None,
        (),
    )
}
