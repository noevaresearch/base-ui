//! Checkbox state model — the derivations, the state-to-attribute walk, and the
//! change-funnel gates of `packages/react/src/checkbox`
//! (`specs/library/checkbox/behavior.md`, `specs/library/checkbox/implementation.md`).
//!
//! Everything here is view-independent and host-testable: the snapshot record
//! upstream memoizes as `CheckboxRootState` (`CheckboxRoot.tsx:278-288`), the
//! `getCheckboxStateAttributesMapping` walk
//! (`checkbox/utils/getCheckboxStateAttributesMapping.ts:6-24`), and the pure
//! folds the component's body performs before any rendering (the disabled/name/
//! value composition at `CheckboxRoot.tsx:93-96`, the computed checked pair at
//! `:147-150`, the hidden-input gates at `:402-404`/`:201-207`, the two-veto
//! change funnel at `:211-246`).
//!
//! Runtime split (the crate's dual-runtime law — see `field/mod.rs` and
//! `accordion/mod.rs`): the internals crate's hooks are typed over
//! reactive-graph 0.2, while leptos 0.7's view tree tracks reactive-graph 0.1.
//! The view layers of this unit therefore re-home the state machine on leptos
//! signals (the accordion/field precedent) and consume only the *view-independent*
//! internals vocabulary: [`get_state_attributes_props`] +
//! [`field_validity_mapping`] for the walk, the event/`visuallyHidden` constants,
//! and the pure helpers re-homed here.

use std::collections::BTreeMap;

use serde_json::Value;

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_internals::state_attributes::{
    StateAttributeProps, StateAttributesMapping, field_validity_mapping, get_state_attributes_props,
};

/// The change-event details type — upstream's
/// `BaseUIChangeEventDetails<CheckboxRoot.ChangeEventReason>` with
/// `ChangeEventReason = REASONS.none` (`CheckboxRoot.tsx:513-515`). The payload is
/// the native input event (`CheckboxRoot.tsx:223`).
pub type CheckboxChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::Event>;

/// The group-facing change-event details the checkbox hands to the group's
/// `setValue`/child-props factories. The checkbox group's port types its own
/// details payload as `()` (`CheckboxGroupChangeEventDetails`), so the native event
/// cannot ride this one; no reader in either unit consumes it for the group path
/// (upstream shares one object across both consumers, whose `reason` is
/// `REASONS.none` and whose payload the group's own contexts never read).
pub type CheckboxGroupFacingDetails = BaseUIChangeEventDetails<(), ()>;

/// `PARENT_CHECKBOX` (`CheckboxRoot.tsx:38`) — the group-parent marker attribute.
pub const PARENT_CHECKBOX: &str = "data-parent";

/// `CheckboxRootDataAttributes.checked` (`CheckboxRootDataAttributes.ts:4`).
pub const DATA_CHECKED: &str = "data-checked";
/// `CheckboxRootDataAttributes.unchecked` (`CheckboxRootDataAttributes.ts:8`).
pub const DATA_UNCHECKED: &str = "data-unchecked";
/// `CheckboxRootDataAttributes.indeterminate` (`CheckboxRootDataAttributes.ts:12`).
pub const DATA_INDETERMINATE: &str = "data-indeterminate";
/// `CheckboxRootDataAttributes.disabled` (`CheckboxRootDataAttributes.ts:16`).
pub const DATA_DISABLED: &str = "data-disabled";
/// `CheckboxRootDataAttributes.readonly` (`CheckboxRootDataAttributes.ts:20`).
pub const DATA_READONLY: &str = "data-readonly";
/// `CheckboxRootDataAttributes.required` (`CheckboxRootDataAttributes.ts:24`).
pub const DATA_REQUIRED: &str = "data-required";
/// `CheckboxRootDataAttributes.valid` (`CheckboxRootDataAttributes.ts:28`).
pub const DATA_VALID: &str = "data-valid";
/// `CheckboxRootDataAttributes.invalid` (`CheckboxRootDataAttributes.ts:32`).
pub const DATA_INVALID: &str = "data-invalid";
/// `CheckboxRootDataAttributes.touched` (`CheckboxRootDataAttributes.ts:36`).
pub const DATA_TOUCHED: &str = "data-touched";
/// `CheckboxRootDataAttributes.dirty` (`CheckboxRootDataAttributes.ts:40`).
pub const DATA_DIRTY: &str = "data-dirty";
/// `CheckboxRootDataAttributes.filled` (`CheckboxRootDataAttributes.ts:44`).
pub const DATA_FILLED: &str = "data-filled";
/// `CheckboxRootDataAttributes.focused` (`CheckboxRootDataAttributes.ts:48`).
pub const DATA_FOCUSED: &str = "data-focused";

/// Every state-walk-owned `data-*` name this unit manages (the writer's stale-removal
/// universe — the field parts' managed-name convention).
pub const MANAGED_STATE_ATTRIBUTES: [&str; 12] = [
    DATA_CHECKED,
    DATA_UNCHECKED,
    DATA_INDETERMINATE,
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

/// The memoized state record — upstream's `CheckboxRootState`
/// (`CheckboxRoot.tsx:278-288`): the Field lifecycle state spread plus the
/// checkbox's own five members (`:410-431`). This single object is both the
/// context value upstream and the input to the attribute mapping (`:290`) — which
/// is why Root and Indicator always agree on style hooks
/// (implementation.md, "Context providers/consumers").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxRootState {
    /// `checked` (`:281`) — the computed (group-aware) checkedness.
    pub checked: bool,
    /// `disabled` (`:282`).
    pub disabled: bool,
    /// `readOnly` (`:283`).
    pub read_only: bool,
    /// `required` (`:284`).
    pub required: bool,
    /// `indeterminate` (`:285`) — the computed (group-aware) flag.
    pub indeterminate: bool,
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

impl CheckboxRootState {
    /// The `serde_json` state map `getStateAttributesProps` walks
    /// (`packages/react/src/internals/getStateAttributesProps.ts:12`). The keys are
    /// upstream's state member names: the walk lowercases them (`readOnly` →
    /// `data-readonly`) and the custom mapping intercepts `checked`/`valid`.
    pub fn to_state_map(self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("checked".to_string(), Value::Bool(self.checked));
        map.insert("disabled".to_string(), Value::Bool(self.disabled));
        map.insert("readOnly".to_string(), Value::Bool(self.read_only));
        map.insert("required".to_string(), Value::Bool(self.required));
        map.insert("indeterminate".to_string(), Value::Bool(self.indeterminate));
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

    /// `aria-checked` (`CheckboxRoot.tsx:299`): `'mixed'` while indeterminate,
    /// otherwise the boolean as the string `'true'`/`'false'`. The `indeterminate`
    /// flag wins structurally — the flag is a prop mirror, never consumed by
    /// clicking (implementation.md, "State derivation order").
    pub fn aria_checked(self) -> String {
        if self.indeterminate {
            "mixed".to_string()
        } else {
            self.checked.to_string()
        }
    }
}

/// The `getCheckboxStateAttributesMapping` walk
/// (`checkbox/utils/getCheckboxStateAttributesMapping.ts:6-24`): the `checked`
/// member becomes the mutually exclusive `data-checked`/`data-unchecked` pair —
/// and emits neither while `indeterminate` (whose `data-indeterminate` comes from
/// the generic truthiness arm, `:12`'s comment) — while every other field falls
/// through to `fieldValidityMapping` (`:22`).
///
/// The mapping is a function of the *indeterminate* flag alone, which is exactly
/// what the upstream callback closes over (`state.indeterminate`).
pub fn get_checkbox_state_attributes_mapping(
    indeterminate: bool,
) -> impl Fn(&str, &Value) -> Option<Option<StateAttributeProps>> {
    move |key: &str, value: &Value| {
        if key == "checked" {
            if indeterminate {
                // `return {}` (`:10-13`): the field is consumed with no attributes.
                return Some(Some(StateAttributeProps::new()));
            }

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

/// The full walk over one snapshot — the real ported engine
/// ([`get_state_attributes_props`]) with this unit's mapping and the mapping's
/// `fieldValidityMapping` half.
pub fn checkbox_state_attributes(state: &CheckboxRootState) -> StateAttributeProps {
    let state_map = state.to_state_map();
    let mapping = get_checkbox_state_attributes_mapping(state.indeterminate);
    get_state_attributes_props(&state_map, Some(&mapping as &StateAttributesMapping))
}

// ---------------------------------------------------------------------------
// The pure folds (`CheckboxRoot.tsx:93-150`, `:201-207`, `:402-404`)
// ---------------------------------------------------------------------------

/// `disabled = rootDisabled || fieldItemContext.disabled || groupContext?.disabled
/// || disabledProp` (`CheckboxRoot.tsx:93-94`).
pub fn effective_disabled(
    field_disabled: bool,
    item_disabled: bool,
    group_disabled: bool,
    disabled_prop: bool,
) -> bool {
    field_disabled || item_disabled || group_disabled || disabled_prop
}

/// `name = fieldName ?? nameProp` (`:95`) — the Field's name wins.
pub fn effective_name(field_name: Option<String>, name_prop: Option<String>) -> Option<String> {
    field_name.or(name_prop)
}

/// `value = valueProp ?? name` (`:96`).
pub fn effective_value(value_prop: Option<String>, name: Option<String>) -> Option<String> {
    value_prop.or(name)
}

/// `computedChecked = isGroupedWithParent ? Boolean(groupChecked) : checked`
/// (`:147`).
pub fn computed_checked(is_grouped_with_parent: bool, group_checked: bool, checked: bool) -> bool {
    if is_grouped_with_parent {
        group_checked
    } else {
        checked
    }
}

/// `computedIndeterminate = isGroupedWithParent ? groupIndeterminate ||
/// indeterminate : indeterminate` (`:148-150`).
pub fn computed_indeterminate(
    is_grouped_with_parent: bool,
    group_indeterminate: bool,
    indeterminate: bool,
) -> bool {
    if is_grouped_with_parent {
        group_indeterminate || indeterminate
    } else {
        indeterminate
    }
}

/// The `useControlled` controlled arm (`:137-141`): group membership when the
/// checkbox carries a `value` inside a group and is not the group's `parent`, else
/// the group-derived / local `checked` prop.
pub fn controlled_checked(
    value: Option<&str>,
    group_value: Option<&[String]>,
    parent: bool,
    group_checked: Option<bool>,
) -> Option<bool> {
    match (value, group_value) {
        (Some(value), Some(group_value)) if !parent => {
            Some(group_value.iter().any(|member| member == value))
        }
        _ => group_checked,
    }
}

/// `groupChecked = groupProps.checked ?? checkedProp`, `groupIndeterminate =
/// groupProps.indeterminate ?? indeterminate` (`:119-124`) — the group override with
/// the local props as the defaults.
pub fn group_override(group_value: Option<bool>, local_value: bool) -> bool {
    group_value.unwrap_or(local_value)
}

/// The hidden `uncheckedValue` input's condition (`:402-404`):
/// `!checked && !groupContext && name && !parent && uncheckedValue !== undefined`.
pub fn should_render_unchecked_value_input(
    checked: bool,
    in_group: bool,
    name: Option<&str>,
    parent: bool,
    unchecked_value: Option<&str>,
) -> bool {
    !checked
        && !in_group
        && name.is_some_and(|name| !name.is_empty())
        && !parent
        && unchecked_value.is_some()
}

/// The hidden input's `name` (`:201`): `parent ? undefined : name` — group parents
/// are excluded from form submission.
pub fn input_name(parent: bool, name: Option<&str>) -> Option<String> {
    if parent {
        None
    } else {
        name.map(str::to_string)
    }
}

/// The hidden input's `id` (`:204`): `nativeButton ? undefined : controlId` — in the
/// native-button mode the visible element *is* the labelable control.
pub fn input_id(native_button: bool, control_id: &str) -> Option<String> {
    if native_button {
        None
    } else {
        Some(control_id.to_string())
    }
}

/// The visible element's `id` (`:108`): `rootId = nativeButton ? controlId : id`.
pub fn root_id(native_button: bool, control_id: &str, generated_id: &str) -> String {
    if native_button {
        control_id.to_string()
    } else {
        generated_id.to_string()
    }
}

/// The hidden input's `value` (`:256-260`): only when `valueProp` is defined; inside a
/// group it is gated on the checked state, and the `|| ''` fallback drops a falsy value
/// (the React <19 empty-value workaround).
pub fn input_value(in_group: bool, checked: bool, value_prop: Option<&str>) -> Option<String> {
    let value_prop = value_prop?;
    let resolved = if in_group {
        if checked { value_prop } else { "" }
    } else {
        value_prop
    };
    Some(if resolved.is_empty() {
        String::new()
    } else {
        resolved.to_string()
    })
}

/// The input `style` mode (`:207`): the form-participating input is laid out with the
/// `visuallyHiddenInput` recipe, the nameless one with plain `visuallyHidden`.
pub fn input_style(name: Option<&str>) -> &'static [(&'static str, &'static str)] {
    if name.is_some() {
        leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN_INPUT
    } else {
        leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN
    }
}

/// The `Indicator`'s mount predicate (`CheckboxIndicator.tsx:27`): `rendered =
/// rootState.checked || rootState.indeterminate`.
pub fn indicator_rendered(checked: bool, indeterminate: bool) -> bool {
    checked || indeterminate
}

/// `shouldRender = keepMounted || mounted` (`CheckboxIndicator.tsx:57`).
pub fn indicator_should_render(keep_mounted: bool, mounted: bool) -> bool {
    keep_mounted || mounted
}

/// The `useValueChanged` dirty fold (`CheckboxRoot.tsx:190`): `checked !==
/// validityData.initialValue`.
pub fn checked_is_dirty(checked: bool, initial_value: &Value) -> bool {
    match initial_value {
        Value::Bool(initial) => checked != *initial,
        // A non-boolean initial value (an unset `null`) is not equal to any boolean.
        Value::Null => false,
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// The change funnel (`CheckboxRoot.tsx:211-246`)
// ---------------------------------------------------------------------------

/// The two-veto change funnel's outcome for one input change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeFunnel {
    /// The native event arrived default-prevented — ignored entirely
    /// (`CheckboxRoot.tsx:212-215`, the React #9023 workaround).
    DefaultPrevented,
    /// `readOnly`: the change is re-prevented and nothing is committed (`:217-220`).
    ReadOnly,
    /// `onCheckedChange` cancelled — neither the state nor the group updates
    /// (`:225-229`).
    CanceledByCheckedChange,
    /// The group's `onCheckedChange` cancelled — the local state still did NOT commit
    /// (`:231-235`).
    CanceledByGroup,
    /// Accepted: the local state commits and the group value splices
    /// (`:237-245`).
    Commit,
}

/// Runs the two veto gates in upstream's order and returns the funnel outcome.
///
/// `on_checked_change` is the consumer's `onCheckedChange`, `group_on_change` the
/// group-supplied one (the parent checkbox's status-cycle factory or the
/// group-parent's child factory). The two gates use the same `isCanceled` protocol
/// upstream chains (`details.isCanceled` after each call).
pub fn run_change_funnel(
    next_checked: bool,
    read_only: bool,
    default_prevented: bool,
    on_checked_change: Option<&dyn Fn(bool, &CheckboxChangeEventDetails)>,
    group_on_change: Option<&dyn Fn(bool, &CheckboxGroupFacingDetails)>,
    details: &CheckboxChangeEventDetails,
    group_details: &CheckboxGroupFacingDetails,
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
        return ChangeFunnel::CanceledByCheckedChange;
    }

    if let Some(callback) = group_on_change {
        callback(next_checked, group_details);
    }

    if group_details.is_canceled() {
        return ChangeFunnel::CanceledByGroup;
    }

    ChangeFunnel::Commit
}

/// The group-value splice (`CheckboxRoot.tsx:239-245`): push on check, filter out on
/// uncheck — the additive update the group's `setValue` then commits.
pub fn splice_group_value(group_value: &[String], value: &str, next_checked: bool) -> Vec<String> {
    if next_checked {
        if group_value.iter().any(|member| member == value) {
            group_value.to_vec()
        } else {
            let mut next = group_value.to_vec();
            next.push(value.to_string());
            next
        }
    } else {
        group_value
            .iter()
            .filter(|member| member.as_str() != value)
            .cloned()
            .collect()
    }
}

/// A `REASONS.none` change details with no payload — the crate's port of
/// `createChangeEventDetails(REASONS.none, …)` for the call sites that need a
/// constructed rather than forwarded event.
pub fn change_event_details() -> CheckboxChangeEventDetails {
    BaseUIChangeEventDetails::new(
        reasons::NONE,
        web_sys::Event::new("change").unwrap(),
        None,
        (),
    )
}
