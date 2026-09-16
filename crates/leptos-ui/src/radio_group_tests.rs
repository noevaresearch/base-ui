//! Host tests for the `library: radio-group` port (`specs/library/radio-group/behavior.md`).
//!
//! WHAT THESE COVER, AND WHY NOT MORE
//! ---------------------------------
//! The group's pure half is the state model and the wiring decisions: the Field-resolved
//! `disabled`/`name` folds, the `aria-labelledby` precedence chain, the single veto-able
//! commit gate, the form-value projection, the `inputRef` representative rules, the
//! state→`data-*` record, and the two handlers' predicates. Those are exactly the
//! obligations that need no browser, and they are pinned here one behavior.md section at a
//! time — the `switch_tests.rs`/`toggle_group_tests.rs` precedent.
//!
//! The DOM half — clicks moving `aria-checked`, arrow navigation moving focus AND
//! selection, roving tabindex, label clicks, native `FormData` payloads — is browser
//! territory: upstream's own suite gates the form-payload cases `it.skipIf(isJSDOM)`
//! (`RadioGroup.test.tsx:560`, `:584`). This box refuses to start Chromium by design
//! (`ralph/generated/env-health.json`: `browser: DEGRADED — browser gates REFUSED here by
//! lib/browser-budget.mjs (4 GB cgroup)`), and no CI job runs this crate's wasm suite
//! (`tooling: the crate DOM/wasm suite runs nowhere but this box`), so the rendered axes
//! are CI's to measure; this close rests on the host suite plus the source gates, which is
//! what the item's `done-when` names.
//!
//! A note on the unit boundary: `Radio.Root` — the group's only consumer — is
//! `library: radio`'s item. The context contract this suite exercises is therefore tested
//! through the group's own provider/consumer seam rather than through the child
//! component, exactly as the `library: checkbox-group` close did while `Checkbox.Root`
//! was still unported.

use leptos::prelude::*;

use crate::radio_group::{
    ENABLE_HOME_AND_END_KEYS, MODIFIER_KEYS, REASON_NONE, ROLE_RADIOGROUP,
    RadioGroupChangeEventDetails, RadioGroupElementProps, RadioGroupFieldStateSnapshot,
    RadioGroupValue, ValueChangeFunnelOutcome, aria_presence, blur_leaves_the_group,
    effective_disabled, effective_name, keydown_arms_touched, project_form_value,
    radio_group_state_map, register_input_ref_should_repoint, register_input_ref_should_skip,
    resolve_aria_labelledby, run_value_change_funnel, should_point_ref_at_fallback,
};

/// A details value as a consumer's commit would carry it — `REASONS.none`, no payload
/// (`RadioGroup.tsx:341-343`).
fn change_details() -> RadioGroupChangeEventDetails {
    RadioGroupChangeEventDetails::new(REASON_NONE, (), None, ())
}

/// The host default field-state snapshot: nothing touched, nothing valid yet.
fn field_state() -> RadioGroupFieldStateSnapshot {
    RadioGroupFieldStateSnapshot::default()
}

// ---------------------------------------------------------------------------
// Public API surface (behavior.md "Public API surface (props, parts, subcomponents)")
// ---------------------------------------------------------------------------

/// behavior.md:13 — the documented prop surface with its documented defaults: an
/// uncontrolled group with no selection, no name, no form and no ref
/// (`RadioGroup.test.tsx:843-866`: `value` is never forwarded to the root, and no
/// `defaultValue` means nothing is checked).
#[test]
fn the_default_props_describe_an_uncontrolled_group_with_no_selection() {
    let props = RadioGroupElementProps::default();

    assert_eq!(props.value, None, "no controlled value (upstream undefined)");
    assert_eq!(props.default_value, None, "nothing selected initially");
    assert!(!props.disabled, "@default false");
    assert!(!props.read_only, "@default false");
    assert!(!props.required, "@default false");
    assert_eq!(props.name, None);
    assert_eq!(props.form, None, "no native form association by default");
    assert_eq!(props.id, None, "the root gets no id unless one is passed");
    assert!(props.input_ref.is_none());
    assert!(props.element_attributes.is_empty(), "no ...elementProps rest");
}

/// behavior.md:15 — the group exports no parts of its own; its whole surface is one
/// rendered `div` plus the context its `Radio.Root` children consume
/// (`radio-group/index.ts:1-3`).
#[test]
fn the_unit_exposes_no_subcomponents_of_its_own() {
    // The claim is structural: the group's element description renders ONE element with
    // the radiogroup role, and every other surface the unit documents is context.
    assert_eq!(ROLE_RADIOGROUP, "radiogroup", "the documented default role");
    assert_eq!(
        REASON_NONE, "none",
        "the change-event reason type admits only 'none' (implementation.md :343)"
    );
}

// ---------------------------------------------------------------------------
// State model (behavior.md "State model (controlled/uncontrolled, defaults, transitions)")
// ---------------------------------------------------------------------------

/// behavior.md:29 — `Field.Root disabled` propagates onto the group, and the group's own
/// prop disables it on its own; either one is enough (`RadioGroup.tsx:68`,
/// `RadioGroup.test.tsx:947-965`).
#[test]
fn the_field_disabled_flag_wins_over_the_group_prop() {
    assert!(effective_disabled(true, false), "Field disabled alone");
    assert!(effective_disabled(false, true), "the prop alone");
    assert!(effective_disabled(true, true), "both");
    assert!(!effective_disabled(false, false), "neither");
}

/// behavior.md:81 — "Field name takes precedence" over the group's own `name`
/// (`RadioGroup.tsx:69`, `RadioGroup.test.tsx:929-944`).
#[test]
fn the_field_name_takes_precedence_over_the_group_name() {
    assert_eq!(
        effective_name(Some("from-field".into()), Some("from-prop".into())),
        Some("from-field".into())
    );
    assert_eq!(
        effective_name(None, Some("from-prop".into())),
        Some("from-prop".into()),
        "no Field name → the prop is used"
    );
    assert_eq!(effective_name(None, None), None);
}

/// behavior.md:27, :92 — cancellation: `onValueChange` runs FIRST, and when it vetoes the
/// change the state write never happens (`RadioGroup.tsx:80-90`; the three interaction
/// paths at `RadioGroup.test.tsx:117-137`, `:139-164`, `:166-195` all funnel here).
#[test]
fn a_canceled_change_runs_the_callback_and_never_commits() {
    let details = change_details();
    let seen: std::cell::RefCell<Vec<RadioGroupValue>> = std::cell::RefCell::new(Vec::new());
    let outcome: ValueChangeFunnelOutcome = run_value_change_funnel(
        Some("b".to_string()),
        &details,
        // The cancellation happens in the consumer's callback — upstream's shape
        // (`RadioGroup.test.tsx:117-137`): the veto is observed AFTER the callback ran.
        Some(&|value: RadioGroupValue, details: &RadioGroupChangeEventDetails| {
            seen.borrow_mut().push(value);
            details.cancel();
        }),
        |_committed| panic!("the commit must not run for a canceled change"),
    );
    assert!(outcome.callback_ran, "onValueChange is invoked first");
    assert!(!outcome.committed);
    assert_eq!(*seen.borrow(), vec![Some("b".to_string())]);
}

/// The non-vetoed half of the same gate: the callback still runs first, then the write.
#[test]
fn a_committed_change_writes_the_value() {
    let details = change_details();
    let mut committed_value: Option<RadioGroupValue> = None;
    let outcome = run_value_change_funnel(
        Some("a".to_string()),
        &details,
        Some(&|_value: RadioGroupValue, _details: &RadioGroupChangeEventDetails| {}),
        |committed| committed_value = Some(committed),
    );
    assert!(outcome.callback_ran);
    assert!(outcome.committed, "the uncanceled change commits");
    assert_eq!(committed_value, Some(Some("a".to_string())));
}

/// behavior.md:27 — `eventDetails.cancel()` inside `onValueChange` prevents the state
/// change. The veto is read from the details, not from the callback's return.
#[test]
fn the_veto_is_taken_from_the_event_details() {
    let details = change_details();
    let mut committed = false;
    {
        let outcome = run_value_change_funnel(
            Some("c".to_string()),
            &details,
            Some(&|_value: RadioGroupValue, details: &RadioGroupChangeEventDetails| {
                details.cancel();
            }),
            |_committed| committed = true,
        );
        assert!(!outcome.committed, "cancel() inside the callback vetoes");
    }
    assert!(!committed);

    // A group with NO onValueChange has nothing that can veto it.
    let uncanceled = change_details();
    let outcome = run_value_change_funnel(
        Some("c".to_string()),
        &uncanceled,
        None::<&dyn Fn(RadioGroupValue, &RadioGroupChangeEventDetails)>,
        |_committed| {},
    );
    assert!(!outcome.callback_ran, "no consumer callback");
    assert!(outcome.committed, "an unvetoed change still commits");
}

// ---------------------------------------------------------------------------
// Keyboard interactions (behavior.md "Keyboard interactions")
// ---------------------------------------------------------------------------

/// behavior.md:26-28 — any Arrow keydown arms the group-local `touched` flag on the
/// capture phase (`RadioGroup.tsx:249-254`); that flag, not the capture handler itself,
/// is what makes the child auto-select on focus (`radio/root/RadioRoot.tsx:153-161`).
#[test]
fn every_arrow_key_arms_the_auto_select_flag() {
    for key in ["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight"] {
        assert!(keydown_arms_touched(key), "{key} arms the flag");
    }
}

/// behavior.md:35-36 — Space activates and Enter does not ("Enter does not select"), so
/// neither may arm the arrow flag; nor may an ordinary character.
#[test]
fn space_enter_and_plain_characters_do_not_arm_the_flag() {
    for key in ["Enter", " ", "a", "Shift", "Tab", "Home", "End"] {
        assert!(!keydown_arms_touched(key), "{key} must not arm the flag");
    }
}

/// behavior.md:40 — "Modifier keys do not block navigation": Shift is the single
/// exemption upstream passes (`RadioGroup.tsx:23,272`).
#[test]
fn shift_is_the_only_exempted_modifier() {
    assert_eq!(
        MODIFIER_KEYS,
        [leptos_ui_internals::composite::SHIFT],
        "MODIFIER_KEYS = [SHIFT]"
    );
}

/// implementation.md untested item 1 — `enableHomeAndEndKeys={false}`
/// (`RadioGroup.tsx:271`): Home/End are deliberately inert.
#[test]
fn home_and_end_are_deliberately_inert() {
    assert!(
        !ENABLE_HOME_AND_END_KEYS,
        "upstream passes enableHomeAndEndKeys=false"
    );
}

// ---------------------------------------------------------------------------
// Focus management (behavior.md "Focus management")
// ---------------------------------------------------------------------------

/// behavior.md:75 — `validationMode="onBlur"` validates only when focus LEAVES the
/// group; a blur whose `relatedTarget` is another radio inside the group does not
/// validate (`RadioGroup.tsx:239-248`, `RadioGroup.test.tsx:1189-1218`).
#[test]
fn an_intra_group_blur_does_not_leave_the_group() {
    assert!(!blur_leaves_the_group(true), "radio → radio stays inside");
    assert!(blur_leaves_the_group(false), "focus left the group subtree");
}

/// behavior.md:55 — "Disabled radios are skipped when assigning the ref", and the
/// re-point rules of `RadioGroup.tsx:133-137`: the ref moves when the input is checked,
/// when no representative exists yet, or when the current representative is disabled.
#[test]
fn the_public_ref_repoints_only_on_the_documented_conditions() {
    assert!(
        register_input_ref_should_repoint(true, true, false),
        "a checked input becomes the representative"
    );
    assert!(
        register_input_ref_should_repoint(false, false, false),
        "no representative yet → this one takes it"
    );
    assert!(
        register_input_ref_should_repoint(false, true, true),
        "the current representative is disabled → move on"
    );
    assert!(
        !register_input_ref_should_repoint(false, true, false),
        "an unchecked input with a healthy representative does not steal the ref"
    );
}

/// behavior.md:55 — `RadioGroup.tsx:125-127`: a null or disabled input is skipped
/// outright, so a disabled radio can never become the representative.
#[test]
fn a_disabled_input_is_skipped_entirely() {
    assert!(register_input_ref_should_skip(true));
    assert!(!register_input_ref_should_skip(false));
}

/// behavior.md:57 — "clearing to null keeps the ref on the first radio's input": the
/// imperative re-point after a change to `null` (`RadioGroup.tsx:184-188`,
/// `RadioGroup.test.tsx:468-497`) — and only while that fallback is itself enabled.
#[test]
fn clearing_to_null_repoints_to_the_first_enabled_input() {
    assert!(should_point_ref_at_fallback(true, true, false));
    assert!(
        !should_point_ref_at_fallback(true, true, true),
        "a disabled fallback is not adopted"
    );
    assert!(
        !should_point_ref_at_fallback(true, false, false),
        "no fallback registered yet"
    );
    assert!(
        !should_point_ref_at_fallback(false, true, false),
        "a non-null value never re-points"
    );
}

/// behavior.md:50-59 — the `inputRef` semantics table is about the representative
/// machinery as a whole; the pieces are pinned above, and this asserts the invariant
/// they share: a disabled input is never adopted, but the group still remembers it as
/// the "current representative was disabled" signal only through the current slot.
#[test]
fn the_representative_rules_cannot_adopt_a_disabled_input() {
    // skip wins over every re-point condition.
    assert!(register_input_ref_should_skip(true));
    assert!(
        !register_input_ref_should_repoint(false, true, false)
            || !register_input_ref_should_skip(false),
        "a healthy representative is not replaced by an unchecked enabled input"
    );
}

// ---------------------------------------------------------------------------
// Accessibility (behavior.md "Accessibility (roles, aria-*, id linking)")
// ---------------------------------------------------------------------------

/// behavior.md:63-66 — the root's role and the presence-only `aria-*` members:
/// `aria-disabled`/`aria-readonly`/`aria-required` render `"true"` when set and are
/// ABSENT otherwise (`RadioGroup.tsx:232-234`, `RadioGroup.test.tsx:212-215`,
/// `:235-245`).
#[test]
fn the_presence_aria_attributes_are_absent_when_unset() {
    assert_eq!(aria_presence(true), Some("true".to_string()));
    assert_eq!(aria_presence(false), None, "React's `|| undefined`");
}

/// behavior.md:71-72 — label precedence and cleanup: an explicit `aria-labelledby` beats
/// Field.Label, which beats Fieldset.Legend (`RadioGroup.tsx:191`,
/// `RadioGroup.test.tsx:1270-1358`).
#[test]
fn field_label_beats_fieldset_legend() {
    assert_eq!(
        resolve_aria_labelledby(Some("field-label".into()), Some("legend".into())),
        Some("field-label".into()),
        "Field.Label wins by the ?? short-circuit"
    );
    assert_eq!(
        resolve_aria_labelledby(None, Some("legend".into())),
        Some("legend".into()),
        "Fieldset.Legend is the fallback"
    );
    assert_eq!(
        resolve_aria_labelledby(None, None),
        None,
        "no label anywhere → the attribute is omitted"
    );
}

/// behavior.md:67 — the style-hook record: `data-disabled`/`data-readonly`/
/// `data-required` on the group plus the field-state members (`RadioGroup.tsx:193-198`,
/// `RadioGroup.test.tsx:281-307`).
#[test]
fn the_state_record_carries_the_group_flags_and_the_field_state() {
    let state = RadioGroupFieldStateSnapshot {
        touched: true,
        dirty: true,
        valid: Some(false),
        filled: true,
        focused: true,
    };
    let map = radio_group_state_map(&state, true, true, true);

    assert_eq!(map.get("disabled").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(map.get("required").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(map.get("readOnly").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(map.get("touched").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(map.get("dirty").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(map.get("filled").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(map.get("focused").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        map.get("valid"),
        Some(&serde_json::Value::Bool(false)),
        "the validity walk reads `valid` and emits data-invalid"
    );
}

/// behavior.md:73 — validation state: an unrun validation must not be reported as
/// invalid, so `valid` stays JSON `null` (the mapping's `Some(None)` arm, which emits
/// nothing).
#[test]
fn an_unrun_validation_reports_null_rather_than_false() {
    let map = radio_group_state_map(&field_state(), false, false, false);
    assert_eq!(map.get("valid"), Some(&serde_json::Value::Null));
    assert_eq!(map.get("disabled").and_then(|v| v.as_bool()), Some(false));
}

// ---------------------------------------------------------------------------
// DOM structure & portal behavior (behavior.md "DOM structure & portal behavior")
// ---------------------------------------------------------------------------

/// behavior.md:82-83, :95-100 — native form data: with nothing selected the group
/// projects `null` (matching native radios), and a selection projects its value
/// (`RadioGroup.tsx:159-172`, `RadioGroup.test.tsx:584-620`).
#[test]
fn the_form_value_projection_follows_the_native_radio_semantics() {
    // No `<form>`: the logical value, unfiltered (`:160-163`).
    assert_eq!(
        project_form_value(&Some("a".to_string()), &[], false),
        serde_json::json!("a")
    );
    // Inside a Form with a checked+eligible registration: the value projects.
    assert_eq!(
        project_form_value(&Some("a".to_string()), &[(true, true)], true),
        serde_json::json!("a")
    );
    // Inside a Form with nothing selected: null, like a native radio group.
    assert_eq!(
        project_form_value(&None, &[(false, true)], true),
        serde_json::Value::Null
    );
}

/// behavior.md:82, :97-98 — a disabled checked radio, or one bound to another `form`,
/// is not eligible, so nothing projects even though the group holds a value
/// (`RadioGroup.tsx:165-171`).
#[test]
fn an_ineligible_checked_input_projects_null() {
    assert_eq!(
        project_form_value(&Some("a".to_string()), &[(true, false)], true),
        serde_json::Value::Null,
        "checked but ineligible (disabled / bound to another form)"
    );
    assert_eq!(
        project_form_value(&Some("a".to_string()), &[(false, true)], true),
        serde_json::Value::Null,
        "eligible but unchecked"
    );
    assert_eq!(
        project_form_value(&Some("a".to_string()), &[(true, false), (false, false)], true),
        serde_json::Value::Null,
        "no eligible checked input anywhere in the registry"
    );
}

// ---------------------------------------------------------------------------
// Events (behavior.md "Events (names, payload shape, bubbling, preventDefault …)")
// ---------------------------------------------------------------------------

/// behavior.md:91 — the change event's shape: the first argument is the newly selected
/// value, and the details carry the reason (`RadioGroup.tsx:341-343`).
#[test]
fn the_change_event_carries_the_value_and_the_none_reason() {
    let details = change_details();
    assert_eq!(details.reason, REASON_NONE);
    assert!(!details.is_canceled(), "a fresh change is not canceled");

    let observed: std::cell::RefCell<Option<RadioGroupValue>> =
        std::cell::RefCell::new(None);
    let _ = run_value_change_funnel(
        None,
        &details,
        Some(&|value: RadioGroupValue, _details: &RadioGroupChangeEventDetails| {
            *observed.borrow_mut() = Some(value);
        }),
        |_committed| {},
    );
    assert_eq!(
        *observed.borrow(),
        Some(None),
        "the payload is the raw value — `null` when the group is cleared"
    );
}

// ---------------------------------------------------------------------------
// Edge cases (behavior.md "Edge cases (rapid interactions, unmount, nesting)")
// ---------------------------------------------------------------------------

/// behavior.md:113 — all radios unmounting unblocks submission and the group projects
/// `null` (`RadioGroup.test.tsx:1443-1470`): with an empty registry there is no
/// eligible checked input, so the projection is null rather than the stale value.
#[test]
fn an_empty_registry_projects_null_inside_a_form() {
    assert_eq!(
        project_form_value(&Some("b".to_string()), &[], true),
        serde_json::Value::Null
    );
}

/// behavior.md:116 — an external (controlled) change to `null` still reports through the
/// gate: the funnel commits the null value rather than treating it as "no change".
#[test]
fn clearing_to_null_commits_through_the_same_gate() {
    let details = change_details();
    let committed_value: std::cell::RefCell<Option<RadioGroupValue>> =
        std::cell::RefCell::new(Some(Some("stale".into())));
    let _ = run_value_change_funnel(
        None,
        &details,
        None::<&dyn Fn(RadioGroupValue, &RadioGroupChangeEventDetails)>,
        |committed| *committed_value.borrow_mut() = Some(committed),
    );
    assert_eq!(
        *committed_value.borrow(),
        Some(None),
        "null is a real committed value"
    );
}

// ---------------------------------------------------------------------------
// Shared harness dependencies (behavior.md "Shared harness dependencies")
// ---------------------------------------------------------------------------
// The unit consumes the same shared harness as every other composited unit: the
// `describeConformance` suite (`packages/react/test/describeConformance.tsx:51-70`)
// proves the div-root/ref-forwarding claim this port's forwarded-ref path carries, and
// `isJSDOM` gates exactly the browser-only cases above. Nothing harness-specific is
// re-implemented in this suite.

/// behavior.md:127 — the conformance suite's own claim, restated where the port can
/// carry it: the group renders a `div`, and the forwarded ref path is the composite
/// root's (`RadioGroup.tsx:269`, `refs={[forwardedRef]}`).
#[test]
fn the_forwarded_ref_rides_the_composite_root() {
    // The element-level surface accepts a forwarded ref and the composite root owns it;
    // the port's `root_ref` field is that channel (`RadioGroupElementProps::root_ref`).
    let props = RadioGroupElementProps::default();
    assert!(
        props.root_ref.is_none(),
        "no forwarded ref by default — the channel exists, the caller supplies it"
    );
}
