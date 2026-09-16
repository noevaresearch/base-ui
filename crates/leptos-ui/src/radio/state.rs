//! Radio state model — the derivations and the state→attribute walk of
//! `packages/react/src/radio/` (the `library: radio` TODO item;
//! `specs/library/radio/behavior.md`, `specs/library/radio/implementation.md`).
//!
//! Everything here is view-independent and host-testable: the record upstream memoizes
//! as `RadioRootState` (`packages/react/src/radio/root/RadioRoot.tsx:214-223`), the
//! `stateAttributesMapping` walk (`packages/react/src/radio/utils/stateAttributesMapping.ts:7-20`),
//! and the pure folds the component's body performs before rendering — the
//! `disabled`/`readOnly`/`required` composition over third-party providers
//! (`RadioRoot.tsx:78-81`), the `checked` duality (`:83`), the instance/labelable id
//! split (`:115-117`), the hidden input's `visuallyHidden` recipe (`:177`) and the
//! `value` identity rule (`:179`).
//!
//! Runtime split (the crate's dual-runtime law — see `field/mod.rs`, `switch/state.rs`):
//! the internals crate's hooks are typed over reactive-graph 0.2 while leptos 0.7's view
//! tree tracks 0.1. This module therefore keeps only the *view-independent* vocabulary:
//! the pure folds, the attribute walk over the ported engine
//! ([`get_state_attributes_props`] + [`field_validity_mapping`] +
//! [`transition_status_mapping`]), and the `visuallyHidden` constants.
//!
//! ## The `value` duality
//!
//! Upstream's `value` is generic (`RadioRoot.Props<Value>`, `:310-332`) and admits
//! `string | number | null`; `null` is a *selectable* value, not "absent"
//! (behavior.md:25 and :65, `RadioRoot.test.tsx:42-56`), while `undefined` is a separate
//! state the change funnel refuses to commit (`:190`). The port keeps both with a double
//! [`Option`] — outer = `undefined`, inner = `Value | null` — which is the same encoding
//! the sibling unit `library: radio-group` already published for a controlled value
//! (`crates/leptos-ui/src/radio_group.rs:769`, whose doc calls the outer the
//! controlled/uncontrolled distinction and the inner "upstream's `Value | null`"). See
//! [`is_checked`] for why collapsing the two would break a proven claim.

use std::collections::BTreeMap;

use serde_json::Value;

use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::state_attributes::{
    StateAttributeProps, StateAttributesMapping, field_validity_mapping,
    get_state_attributes_props, transition_status_mapping,
};
use leptos_ui_internals::use_transition_status::TransitionStatus;

/// The change-event details type — upstream's
/// `BaseUIChangeEventDetails<RadioRoot.ChangeEventReason>` with
/// `ChangeEventReason = REASONS.none` (`RadioRoot.tsx:194`,
/// `createChangeEventDetails(REASONS.none, event.nativeEvent)`) and the native input
/// event as the payload.
pub type RadioChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::Event>;

/// `RadioRootDataAttributes.checked` (`RadioRootDataAttributes.ts:4`).
pub const DATA_CHECKED: &str = "data-checked";
/// `RadioRootDataAttributes.unchecked` (`RadioRootDataAttributes.ts:8`).
pub const DATA_UNCHECKED: &str = "data-unchecked";
/// `RadioRootDataAttributes.disabled` (`RadioRootDataAttributes.ts:12`).
pub const DATA_DISABLED: &str = "data-disabled";
/// `RadioRootDataAttributes.readonly` (`RadioRootDataAttributes.ts:16`).
pub const DATA_READONLY: &str = "data-readonly";
/// `RadioRootDataAttributes.required` (`RadioRootDataAttributes.ts:20`).
pub const DATA_REQUIRED: &str = "data-required";
/// `RadioRootDataAttributes.valid` (`RadioRootDataAttributes.ts:24`).
pub const DATA_VALID: &str = "data-valid";
/// `RadioRootDataAttributes.invalid` (`RadioRootDataAttributes.ts:28`).
pub const DATA_INVALID: &str = "data-invalid";
/// `RadioRootDataAttributes.touched` (`RadioRootDataAttributes.ts:32`).
pub const DATA_TOUCHED: &str = "data-touched";
/// `RadioRootDataAttributes.dirty` (`RadioRootDataAttributes.ts:36`).
pub const DATA_DIRTY: &str = "data-dirty";
/// `RadioRootDataAttributes.filled` (`RadioRootDataAttributes.ts:40`).
pub const DATA_FILLED: &str = "data-filled";
/// `RadioRootDataAttributes.focused` (`RadioRootDataAttributes.ts:44`).
pub const DATA_FOCUSED: &str = "data-focused";
/// `TransitionStatusDataAttributes.startingStyle` (`RadioIndicatorDataAttributes.ts:26`) —
/// the indicator's enter hook.
pub const DATA_STARTING_STYLE: &str = "data-starting-style";
/// `TransitionStatusDataAttributes.endingStyle` (`RadioIndicatorDataAttributes.ts:30`) —
/// the indicator's exit hook.
pub const DATA_ENDING_STYLE: &str = "data-ending-style";

/// The `data-*` members this unit's walk can emit — the names the writer binds.
/// Every other `data-*` a consumer passes rides the `...elementProps` rest untouched
/// (`RadioRoot.tsx:50`).
pub const MANAGED_STATE_ATTRIBUTES: [&str; 13] = [
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
    DATA_STARTING_STYLE,
    DATA_ENDING_STYLE,
];

/// `RadioRootState` (`RadioRoot.tsx:271-308`): `FieldRootState` spread
/// (`:216`) plus the four radio members. Upstream's interface also lists
/// `touched`/`dirty`/`valid`/`filled`/`focused`, which arrive from the spread — the
/// port carries them explicitly so the walk can see them.
///
/// The record doubles as the Root context value (`:225`, `RadioRootContext.ts:4`),
/// which is why `Radio.Root` and `Radio.Indicator` always agree on their style hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioRootState {
    /// `checked` (`:220`) — the per-render derivation of [`is_checked`].
    pub checked: bool,
    /// `disabled` (`:217`) — the three-provider fold of [`effective_disabled`].
    pub disabled: bool,
    /// `readOnly` (`:219`).
    pub read_only: bool,
    /// `required` (`:216`).
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

impl RadioRootState {
    /// The `serde_json` state map [`get_state_attributes_props`] walks
    /// (`packages/react/src/internals/getStateAttributesProps.ts:12`). The keys are
    /// upstream's member names: the walk lowercases them (`readOnly` → `data-readonly`),
    /// and the custom mapping intercepts `checked` and `valid`.
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

/// `RadioIndicatorState` (`RadioIndicator.tsx:72-77`): `RadioRootState` spread plus the
/// transition status (`:29-32`). The Indicator's walk is therefore the Root's walk plus
/// the two transition hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioIndicatorState {
    /// `...rootState` (`:30`).
    pub root: RadioRootState,
    /// `transitionStatus` (`:30`, `:76`) — `None` is upstream's `undefined`.
    pub transition_status: Option<TransitionStatus>,
}

impl RadioIndicatorState {
    /// `to_state_map` plus the `transitionStatus` member
    /// (`useTransitionStatus.ts:23-25`), which the mapping intercepts into
    /// `data-starting-style` / `data-ending-style`.
    pub fn to_state_map(self) -> serde_json::Map<String, Value> {
        let mut map = self.root.to_state_map();
        let status = match self.transition_status {
            Some(TransitionStatus::Starting) => Value::String("starting".to_string()),
            Some(TransitionStatus::Ending) => Value::String("ending".to_string()),
            Some(TransitionStatus::Idle) => Value::String("idle".to_string()),
            None => Value::Null,
        };
        map.insert("transitionStatus".to_string(), status);
        map
    }
}

/// The `stateAttributesMapping` (`utils/stateAttributesMapping.ts:7-20`):
/// `transitionStatusMapping` and `fieldValidityMapping` spread in, with `checked`
/// replaced by the mutually exclusive `data-checked`/`data-unchecked` pair
/// (`:8-13`) — truthy emits bare `data-checked`, falsy bare `data-unchecked`, so the
/// two hooks are never both present (behavior.md:24, `RadioRoot.test.tsx:26-29`).
///
/// `transitionStatusMapping` is applied before `fieldValidityMapping` because the
/// shared mapping returns `Some(None)` for a key it owns but declines (an `idle` status
/// emits nothing) — falling through to the validity mapping instead would let the
/// generic walk re-emit `data-transitionstatus` for that key.
pub fn get_radio_state_attributes_mapping()
-> impl Fn(&str, &Value) -> Option<Option<StateAttributeProps>> {
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

        if let Some(mapped) = transition_status_mapping(key, value) {
            return Some(mapped);
        }

        field_validity_mapping(key, value)
    }
}

/// The full walk over one Root snapshot — the ported engine
/// ([`get_state_attributes_props`]) driven by this unit's mapping.
pub fn radio_state_attributes(state: &RadioRootState) -> StateAttributeProps {
    let state_map = state.to_state_map();
    let mapping = get_radio_state_attributes_mapping();
    get_state_attributes_props(&state_map, Some(&mapping as &StateAttributesMapping))
}

/// The Indicator's walk — the same mapping over the extended map, so the transition
/// hooks ride along with the Root's own state (implementation.md: "`transitionStatus` is
/// folded into the state object (`RadioIndicator.tsx:29-32`) so it reaches the data
/// attributes via the mapping below").
pub fn radio_indicator_state_attributes(state: &RadioIndicatorState) -> StateAttributeProps {
    let state_map = state.to_state_map();
    let mapping = get_radio_state_attributes_mapping();
    get_state_attributes_props(&state_map, Some(&mapping as &StateAttributesMapping))
}

// ---------------------------------------------------------------------------
// The pure folds
// ---------------------------------------------------------------------------

/// `disabled = fieldDisabled || fieldItemContext.disabled || disabledGroup || disabledProp`
/// (`RadioRoot.tsx:78`). Every source is a live read at the call site; this fold is the
/// boolean half.
pub fn effective_disabled(
    field_disabled: bool,
    field_item_disabled: bool,
    group_disabled: bool,
    disabled_prop: bool,
) -> bool {
    field_disabled || field_item_disabled || group_disabled || disabled_prop
}

/// `readOnly = readOnlyGroup || readOnlyProp` (`RadioRoot.tsx:79`) — nullish-free `||`,
/// so a group `readOnly` wins.
pub fn effective_read_only(group_read_only: bool, read_only_prop: bool) -> bool {
    group_read_only || read_only_prop
}

/// `required = requiredGroup || requiredProp` (`RadioRoot.tsx:80`).
pub fn effective_required(group_required: bool, required_prop: bool) -> bool {
    group_required || required_prop
}

/// `checked = groupContext ? checkedValue === value : value === ''`
/// (`RadioRoot.tsx:83`).
///
/// `value` is `Option<Option<&str>>` on purpose — the same double encoding the sibling
/// group unit published for a controlled value (`radio_group.rs:769`): the outer `Option`
/// is upstream's `undefined` (the prop absent), the inner one is upstream's `Value | null`.
/// The distinction is load-bearing, not pedantic: `value={null}` is a *selectable* value
/// whose selection is proven (behavior.md:65, `RadioRoot.test.tsx:42-56`), and upstream's
/// `===` separates it from `undefined` — `undefined === undefined` is true, but the
/// group's `checkedValue` is initialized to `null` and never holds `undefined`, so an
/// absent `value` can never match. Collapsing the two would make a `null`-valued radio
/// unselectable.
///
/// Standalone (`value === ''`) compares the inner value to the empty string, which is
/// upstream's own no-group rule.
pub fn is_checked(
    in_group: bool,
    group_checked_value: Option<&str>,
    value: Option<Option<&str>>,
) -> bool {
    let Some(inner) = value else {
        // `undefined === anything` is false for every value the group can hold.
        return false;
    };

    if in_group {
        group_checked_value == inner
    } else {
        inner == Some("")
    }
}

/// `id: nativeButton ? inputId : id` (`RadioRoot.tsx:131`) — the visible control's id.
///
/// In `nativeButton` mode the labelable id lands on the control (and the hidden input
/// gets none, [`hidden_input_id`]); otherwise the generated instance id does
/// (behavior.md:46-47, `RadioRoot.test.tsx:73-77`).
pub fn root_id(native_button: bool, control_id: &str, generated_id: &str) -> String {
    if native_button {
        control_id.to_string()
    } else {
        generated_id.to_string()
    }
}

/// `hiddenInputId = nativeButton ? undefined : inputId` (`RadioRoot.tsx:117`).
pub fn hidden_input_id(native_button: bool, control_id: &str) -> Option<String> {
    if native_button {
        None
    } else {
        Some(control_id.to_string())
    }
}

/// `aria-disabled`-style boolean attributes: upstream renders the string `"true"` or
/// nothing (`RadioRoot.tsx:233-243` — `disabled` never becomes the HTML `disabled`
/// attribute on the visible control).
pub fn aria_bool_attr(flag: bool) -> Option<String> {
    flag.then(|| "true".to_string())
}

/// `style: name ? visuallyHiddenInput : visuallyHidden` (`RadioRoot.tsx:177`): the
/// named-input recipe keeps the input in the form's submission set, the anonymous one
/// removes it from the layout without a `name`.
///
/// The recipes themselves are NOT this unit's to spell — they are the Phase A util
/// (`packages/utils/src/visuallyHidden.ts:3-24`, ported once as
/// [`leptos_ui_utils::visually_hidden`]) and this unit only SELECTS between the two
/// constants, the checkbox/switch precedent (`checkbox/state.rs:341-347`). A per-consumer
/// copy is how this unit previously drifted off upstream (an invented `clip: rect(0 0 0 0)`,
/// a camelCase `clipPath` React key that is invalid CSS in a `style` attribute, and an
/// anonymous recipe that was `position: absolute` instead of upstream's
/// `position: fixed; top: 0; left: 0`).
pub fn input_style(is_named: bool) -> &'static [(&'static str, &'static str)] {
    if is_named {
        leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN_INPUT
    } else {
        leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN
    }
}

/// The `visuallyHidden` recipes as a `style` string (the `switch` port's
/// `style_string`).
pub fn style_string(declarations: &[(&str, &str)]) -> String {
    declarations
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `...(value !== undefined ? { value: serializeValue(value) } : EMPTY_OBJECT)`
/// (`RadioRoot.tsx:179`): the hidden input carries the identity tag serialized
/// (`serializeValue.ts:5-11` — strings verbatim, everything else `JSON.stringify`d, so a
/// `null` value serializes to the four-character string `null`).
///
/// The outer `Option` is upstream's `undefined`; the inner one is `Value | null` (see
/// [`is_checked`]). Only the absent case omits the attribute.
pub fn input_value_attr(value: Option<Option<&str>>) -> Option<String> {
    value.map(|inner| serialize_value(inner))
}

/// `serializeValue(value)` (`internals/serializeValue.ts:5-11`): a string passes through,
/// anything else is `JSON.stringify`d. Because the port's value domain is
/// `string | null`, the only non-string member is `None` → `"null"`.
pub fn serialize_value(value: Option<&str>) -> String {
    value.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string())
}

/// `value === undefined` (`RadioRoot.tsx:190`) — the change funnel's own guard, which
/// refuses to commit a Root that carries no identity tag. It is a check on the *outer*
/// encoding only: a `null` value is defined, so it commits (and in fact is the case
/// behavior.md:65 proves selectable).
pub fn has_value(value: Option<&Option<String>>) -> bool {
    value.is_some()
}

/// `shouldRender = keepMounted || mounted` (`RadioIndicator.tsx:36`) — the Indicator's mount
/// gate (`:57-59`'s early `return null` is the complement). behavior.md:26 proves both halves
/// of the observable: absent while the Root is unchecked, present while checked.
pub fn indicator_should_render(keep_mounted: bool, mounted: bool) -> bool {
    keep_mounted || mounted
}

/// The visible control's tag (`RadioRoot.tsx:218`, `:240-246`): a `span` by default, a real
/// `<button>` under `nativeButton` — or when the `render` element substitutes one
/// (`useRenderElement.tsx:164-196`). behavior.md:13 records the `span` default through the
/// conformance suite's `refInstanceof: HTMLSpanElement`, and behavior.md:51 the
/// `nativeButton` DOM shape.
pub fn control_tag(native_button: bool, render_tag: Option<&str>) -> &'static str {
    if native_button || render_tag == Some("button") {
        "button"
    } else {
        "span"
    }
}

/// The `role` on the visible control (`RadioRoot.tsx:127`). behavior.md:43 proves the value
/// through `getByRole('radio')`.
pub const ROLE_RADIO: &str = "radio";

/// The hidden input's `type` (`RadioRoot.tsx:171`).
pub const INPUT_TYPE_RADIO: &str = "radio";

/// The `aria-hidden` value the hidden input carries (`RadioRoot.tsx:178`): the control is the
/// accessible element, the input is an implementation detail (behavior.md:45-47's id-linking
/// contract depends on it).
pub const INPUT_ARIA_HIDDEN: &str = "true";

/// The hidden input's `tabIndex` (`RadioRoot.tsx:176`) — never in the tab order; the group's
/// composite root owns roving focus (behavior.md:32's ArrowDown).
pub const INPUT_TAB_INDEX: &str = "-1";
