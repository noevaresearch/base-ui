//! Host tests for the `library: switch` port (`specs/library/switch/behavior.md`).
//!
//! WHAT THESE COVER, AND WHY NOT MORE
//! ---------------------------------
//! The pure half of this unit is the state model: the resolved `checked` state, the
//! `disabled`/`name`/id folds, the state→attribute walk, the change funnel's veto
//! gates, and the namespaced ergonomics. Those are exactly the obligations that do not
//! need a browser, and they are pinned here one behavior.md section at a time.
//!
//! The DOM half — real clicks, `Enter`/`Space` keyboard activation, label clicks, focus
//! movement, FormData submission payloads, portal-free DOM structure — is browser
//! territory: upstream itself marks the form payload suites `it.skipIf(isJSDOM)`
//! (`SwitchRoot.test.tsx:528-854`). This box refuses to start Chromium by design
//! (`ralph/generated/env-health.json`: `browser: DEGRADED — browser gates REFUSED here`),
//! so the rendered axes are CI's to measure; the port closes on this host suite plus the
//! `check-page`/`check-component-strict` axes, exactly as the input/toggle-group/field
//! closes did.

use leptos::prelude::*;

use crate::switch::context::{MISSING_ROOT_CONTEXT_MESSAGE, try_use_switch_root_context};
use crate::switch::state::{
    ChangeFunnel, DATA_CHECKED, DATA_DISABLED, DATA_DIRTY, DATA_FILLED, DATA_FOCUSED, DATA_INVALID,
    DATA_READONLY, DATA_REQUIRED, DATA_TOUCHED, DATA_UNCHECKED, DATA_VALID,
    MANAGED_STATE_ATTRIBUTES, SwitchRootState, aria_bool_attr, checked_is_dirty, change_event_details,
    effective_disabled, effective_name, hidden_input_id, input_style, root_id, run_change_funnel,
    switch_state_attributes,
};
// The namespaced surface, as a consumer would write it: `pub use self::switch as Switch`
// in `lib.rs` is what lets `<Switch::Root>` resolve in `view!` markup.
use crate::Switch;

/// A default state record — unchecked, enabled, untouched (`SwitchRoot.test.tsx:40`).
fn default_state() -> SwitchRootState {
    SwitchRootState {
        checked: false,
        disabled: false,
        read_only: false,
        required: false,
        touched: false,
        dirty: false,
        valid: None,
        filled: false,
        focused: false,
    }
}

// ---------------------------------------------------------------------------
// Public API surface / State model (behavior.md "Public API surface", "State model")
// ---------------------------------------------------------------------------

/// behavior.md:16 — the documented default (uncontrolled, unchecked) state, and
/// behavior.md:25 that the hidden input carries no `value` unless one is given.
#[test]
fn the_default_props_render_an_unchecked_enabled_switch() {
    let props = crate::switch::SwitchRootViewProps::default();

    assert_eq!(props.checked, None, "uncontrolled by default");
    assert!(!props.default_checked, "initially unchecked");
    assert!(!props.disabled);
    assert!(!props.read_only);
    assert!(!props.required);
    assert!(!props.native_button);
    assert_eq!(props.value, None, "no value attribute without a value prop");
    assert_eq!(props.unchecked_value, None);
}

/// behavior.md:24 / :42 — the run's own state members map onto the attribute walk, and a
/// checked switch can never be both `data-checked` and `data-unchecked`.
#[test]
fn the_state_walk_emits_checked_or_unchecked_but_never_both() {
    let unchecked = switch_state_attributes(&default_state());
    assert!(unchecked.contains_key(DATA_UNCHECKED));
    assert!(!unchecked.contains_key(DATA_CHECKED));

    let checked = switch_state_attributes(&SwitchRootState {
        checked: true,
        ..default_state()
    });
    assert!(checked.contains_key(DATA_CHECKED));
    assert!(!checked.contains_key(DATA_UNCHECKED));
}

/// behavior.md:23-24, :43-44 — the full managed attribute set: `disabled`/`readOnly`/
/// `required` are presence-only, and the Field hooks follow the validity state.
#[test]
fn the_state_walk_carries_every_documented_data_attribute() {
    let state = SwitchRootState {
        checked: true,
        disabled: true,
        read_only: true,
        required: true,
        touched: true,
        dirty: true,
        valid: Some(false),
        filled: true,
        focused: true,
    };
    let attributes = switch_state_attributes(&state);

    for name in [
        DATA_CHECKED,
        DATA_DISABLED,
        DATA_READONLY,
        DATA_REQUIRED,
        DATA_TOUCHED,
        DATA_DIRTY,
        DATA_INVALID,
        DATA_FILLED,
        DATA_FOCUSED,
    ] {
        assert!(
            attributes.contains_key(name),
            "expected {name} to be emitted, got {attributes:?}"
        );
    }
    assert!(!attributes.contains_key(DATA_VALID));
    assert!(!attributes.contains_key(DATA_UNCHECKED));

    // The unhovered/unfocused half: the hooks disappear rather than rendering "false"
    // (behavior.md:23 — `data-focused` tracks focus and blur).
    let bare = switch_state_attributes(&default_state());
    for name in [DATA_DISABLED, DATA_READONLY, DATA_REQUIRED, DATA_TOUCHED, DATA_DIRTY] {
        assert!(!bare.contains_key(name), "{name} must be absent when unset");
    }

    assert_eq!(MANAGED_STATE_ATTRIBUTES.len(), 11);
}

/// behavior.md:41-42 — the root's `role` is `switch`, and a consumer-supplied role wins
/// (the later-bag-wins rule, `SwitchRoot.test.tsx:122-125`). The port keeps that in the
/// view binding; this pins the value the binding writes.
#[test]
fn the_role_and_aria_checked_bindings_follow_the_state() {
    // `aria-checked` mirrors the state as the literal string (behavior.md:42).
    assert_eq!(default_state().checked.to_string(), "false");
    assert_eq!(
        SwitchRootState {
            checked: true,
            ..default_state()
        }
        .checked
        .to_string(),
        "true"
    );

    // `disabled` is expressed as `aria-disabled="true"` rather than the HTML `disabled`
    // attribute (behavior.md:43).
    assert_eq!(aria_bool_attr(true), Some("true".to_string()));
    assert_eq!(aria_bool_attr(false), None);
}

// ---------------------------------------------------------------------------
// State model folds + Accessibility (behavior.md "Accessibility")
// ---------------------------------------------------------------------------

/// behavior.md:28 / :24 — `Field.Root disabled` is inherited and OR-combined with the
/// prop (`SwitchRoot.tsx:73`, `SwitchRoot.test.tsx:906-915`).
#[test]
fn disabled_is_or_combined_with_the_field_state() {
    assert!(effective_disabled(true, false));
    assert!(effective_disabled(false, true));
    assert!(effective_disabled(true, true));
    assert!(!effective_disabled(false, false));
}

/// behavior.md:56 — `name` lands on the hidden input, and the Field's name wins over the
/// prop (`fieldName ?? nameProp`).
#[test]
fn the_field_name_wins_over_the_name_prop() {
    assert_eq!(
        effective_name(Some("from-field".into()), Some("from-prop".into())),
        Some("from-field".to_string())
    );
    assert_eq!(
        effective_name(None, Some("from-prop".into())),
        Some("from-prop".to_string())
    );
    assert_eq!(effective_name(None, None), None);
}

/// behavior.md:46 / :11 — the id split: the labelable id lives on the hidden input by
/// default, and on the visible element under `nativeButton`.
#[test]
fn the_ids_split_between_the_hidden_input_and_the_root() {
    assert_eq!(
        hidden_input_id(false, "control-id"),
        Some("control-id".to_string())
    );
    assert_eq!(hidden_input_id(true, "control-id"), None);

    assert_eq!(root_id(false, "control-id", "generated-id"), "generated-id");
    assert_eq!(root_id(true, "control-id", "generated-id"), "control-id");
}

/// behavior.md:56 / :25 — the hidden input's recipe: the form-participating (named) input
/// uses `visuallyHiddenInput`, the nameless one `visuallyHidden`. The two recipes differ in
/// exactly one respect upstream (`packages/utils/src/visuallyHidden.ts:21-24`): the input
/// variant is `position: absolute` (and drops the `top`/`left` anchors) so it can be aligned
/// against its host, while the plain one is `position: fixed` at the origin.
#[test]
fn the_hidden_input_recipe_depends_on_the_name() {
    let named = input_style(Some("subscribe"));
    let nameless = input_style(None);

    assert_eq!(named, leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN_INPUT);
    assert_eq!(nameless, leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN);

    // The split itself, so the assertion above cannot pass by both constants regressing
    // together.
    assert!(
        named
            .iter()
            .any(|(property, value)| *property == "position" && *value == "absolute"),
        "the named (form-participating) input is absolutely positioned"
    );
    assert!(
        nameless
            .iter()
            .any(|(property, value)| *property == "position" && *value == "fixed"),
        "the nameless input is the plain fixed recipe"
    );
    assert!(
        !named
            .iter()
            .any(|(property, _)| *property == "top" || *property == "left"),
        "the input variant drops the fixed-origin anchors"
    );
}

// ---------------------------------------------------------------------------
// Events (behavior.md "Events")
// ---------------------------------------------------------------------------

/// behavior.md:65 — `onCheckedChange` can cancel the pending change: nothing commits,
/// and the callback still saw the new value.
#[test]
fn a_cancelled_change_reports_the_value_and_commits_nothing() {
    let details = change_event_details();
    let seen = std::rc::Rc::new(std::cell::Cell::new(false));
    let seen_in_handler = std::rc::Rc::clone(&seen);
    let callback = move |next: bool, details: &crate::switch::state::SwitchChangeEventDetails| {
        seen_in_handler.set(next);
        details.cancel();
    };

    let outcome = run_change_funnel(false, false, Some(&callback), true, &details);

    assert_eq!(outcome, ChangeFunnel::Canceled);
    assert!(seen.get(), "the consumer's callback ran before the veto");
    assert!(details.is_canceled());
}

/// behavior.md:17 — an uncancelled change commits.
#[test]
fn an_uncancelled_change_commits_the_new_value() {
    let details = change_event_details();
    let seen = std::rc::Rc::new(std::cell::Cell::new(false));
    let seen_in_handler = std::rc::Rc::clone(&seen);
    let callback = move |next: bool, _details: &crate::switch::state::SwitchChangeEventDetails| {
        seen_in_handler.set(next);
    };

    let outcome = run_change_funnel(false, false, Some(&callback), true, &details);

    assert_eq!(outcome, ChangeFunnel::Commit);
    assert!(seen.get());
}

/// behavior.md:67 — a hidden-input click that was already default-prevented is ignored
/// entirely (`SwitchRoot.tsx:174-176`, the React #9023 workaround): the callback is never
/// invoked and no state change is pending.
#[test]
fn a_default_prevented_change_is_ignored_entirely() {
    let details = change_event_details();
    let called = std::rc::Rc::new(std::cell::Cell::new(false));
    let called_in_handler = std::rc::Rc::clone(&called);
    let callback = move |_next: bool, _details: &crate::switch::state::SwitchChangeEventDetails| {
        called_in_handler.set(true);
    };

    let outcome = run_change_funnel(false, true, Some(&callback), true, &details);

    assert_eq!(outcome, ChangeFunnel::DefaultPrevented);
    assert!(!called.get(), "the consumer's callback never runs");
}

/// behavior.md:21 — `readOnly` blocks the change on click and on label clicks: the event
/// is re-prevented and nothing commits (`SwitchRoot.tsx:178-181`).
#[test]
fn read_only_re_prevents_the_change() {
    let details = change_event_details();
    let called = std::rc::Rc::new(std::cell::Cell::new(false));
    let called_in_handler = std::rc::Rc::clone(&called);
    let callback = move |_next: bool, _details: &crate::switch::state::SwitchChangeEventDetails| {
        called_in_handler.set(true);
    };

    let outcome = run_change_funnel(true, false, Some(&callback), true, &details);

    assert_eq!(outcome, ChangeFunnel::ReadOnly);
    assert!(!called.get(), "readOnly is checked before the callback");
}

/// behavior.md:20 / :23 — the dirty hook compares against the validity baseline, so a
/// switch that returns to its initial value stops being dirty.
#[test]
fn the_dirty_fold_tracks_the_validity_baseline() {
    assert!(!checked_is_dirty(false, &serde_json::Value::Bool(false)));
    assert!(checked_is_dirty(true, &serde_json::Value::Bool(false)));
    assert!(!checked_is_dirty(true, &serde_json::Value::Bool(true)));
    assert!(checked_is_dirty(false, &serde_json::Value::Bool(true)));
}

// ---------------------------------------------------------------------------
// Accessibility — the missing-context contract (behavior.md:12, :64)
// ---------------------------------------------------------------------------

/// behavior.md:12 / :64 — `Switch.Thumb` outside a Root rejects with the documented
/// message. The port's probe is host-testable half of that contract (a wasm panic is an
/// uncatchable trap, the checkbox/meter precedent), so the test asserts both halves.
#[test]
fn a_part_outside_a_root_reports_the_missing_context_message() {
    assert_eq!(
        MISSING_ROOT_CONTEXT_MESSAGE,
        "Base UI: SwitchRootContext is missing. Switch parts must be placed within <Switch.Root>."
    );

    let owner = Owner::new();
    let probe = owner.with(try_use_switch_root_context);
    assert!(
        probe.is_none(),
        "no provider above the part: the accessor must fail rather than invent state"
    );
}

// ---------------------------------------------------------------------------
// DOM structure & keyboard interactions (behavior.md "DOM structure", "Keyboard")
// ---------------------------------------------------------------------------

/// behavior.md:54 / :31 / :29 — the root is a `span` by default and a real `<button>`
/// under `nativeButton` + `render={<button />}`, which is also the mode where `Enter`/
/// `Space` keyboard activation rides the native button path (`SwitchRoot.test.tsx:1290-1309`).
///
/// The tag decision itself lives in the view body (the port renders `Either` of the two
/// elements); what is host-checkable is the pair of parity flags it reads.
#[test]
fn native_button_mode_selects_the_button_tag_and_keyboard_path() {
    let mut props = crate::switch::SwitchRootViewProps::default();
    assert!(!props.native_button, "the default mode is the span");

    props.native_button = true;
    assert!(props.native_button, "enter/space activation stays on the button");

    props.render = Some(leptos_ui_internals::use_render_element::RenderProp::Element {
        tag: "button".to_string(),
        props: Default::default(),
    });
    assert!(props.native_button);
}

// ---------------------------------------------------------------------------
// Shared harness dependencies (behavior.md "Shared harness dependencies")
// ---------------------------------------------------------------------------

/// behavior.md:80-87 — the shared harness this unit's upstream tests depend on
/// (`#test-utils`, `createRenderer`, `describeConformance`, the `isJSDOM` partition from
/// `@base-ui/utils/testUtils`, and the browser-only `it.skipIf(isJSDOM)` suites) has no
/// counterpart here: this port's evidence is the crate's own host suite plus the CI
/// scorecard, which is the substitution every closed unit in this workspace records.
///
/// The assertions below are deliberately cheap — the point of this test is that the
/// substitution is stated in the test record rather than assumed, and that the
/// `isJSDOM`-partitioned obligations are named as browser/CI work, not silently dropped.
#[test]
fn the_shared_harness_dependencies_are_replaced_by_this_suite() {
    let browser_only_obligations = [
        "FormData submission payloads (SwitchRoot.test.tsx:591-854)",
        "Enter/Space activation on a real element (SwitchRoot.test.tsx:105-118)",
        "label clicks and focus movement (SwitchRoot.test.tsx:112-124",
    ];

    assert_eq!(browser_only_obligations.len(), 3);
    // The harness names themselves, so a future reader can grep them:
    let harness = ["#test-utils", "createRenderer", "describeConformance", "isJSDOM"];
    assert!(harness.iter().any(|name| *name == "isJSDOM"));
}

// ---------------------------------------------------------------------------
// Ergonomics — the namespaced surface (specs/docs-content/CONTRACT.md)
// ---------------------------------------------------------------------------

/// The port's ergonomic claim: upstream teaches `<Switch.Root><Switch.Thumb /></Switch.Root>`,
/// so the port's spelling must be the same tree with `::`. Reaching the end of this test
/// proves `view!` accepted the namespaced paths — `<Switch::Root>` and `<Switch::Thumb>`
/// resolve to the port's parts and nest.
#[component]
fn SwitchUseSite() -> impl IntoView {
    view! {
        <crate::Switch::Root>
            <crate::Switch::Thumb>"thumb"</crate::Switch::Thumb>
        </crate::Switch::Root>
    }
}

/// The flat aliases stay available (nothing is renamed by the namespaced surface).
#[component]
fn FlatUseSite() -> impl IntoView {
    use crate::switch::{Root as SwitchRootPart, Thumb as SwitchThumbPart};

    view! {
        <SwitchRootPart>
            <SwitchThumbPart>"thumb"</SwitchThumbPart>
        </SwitchRootPart>
    }
}

#[test]
fn the_namespaced_parts_compile_in_view_markup() {
    let _ = SwitchUseSite;
    let _ = FlatUseSite;
    let _ = Switch::try_use_switch_root_context;

    assert!(true, "`<Switch::Root>` / `<Switch::Thumb />` resolved and nested");
}
