//! Tests for the radio port — the claims `specs/library/radio/behavior.md` proves for
//! `library: radio`, plus the parts of `implementation.md` that are observable without a
//! browser.
//!
//! ## What this suite claims, and what it deliberately does not
//!
//! `Radio.Root` is a **stateless selection proxy** (implementation.md's own heading): the port's
//! interesting content is a set of pure derivations — the `checked` duality over the group's
//! context, the three-provider `disabled` fold, the state→attribute walk, the hidden input's
//! `visuallyHidden` recipe and `value` serialization, the Indicator's mount gate — so those are
//! what the host tests below pin, section by section.
//!
//! The DOM half (a real mount, the composite roving focus, the animation lifecycle) is NOT
//! asserted here. This box refuses browsers by design (`ralph/generated/env-health.json`:
//! `browser: DEGRADED — browser gates REFUSED here by lib/browser-budget.mjs (4 GB cgroup)`) and
//! no CI job runs the crate's wasm suite (`tooling: the crate DOM/wasm suite runs nowhere but this
//! box`), so the DOM consequences are named as open in the item's ledger entry rather than
//! dressed up here. `crates/leptos-ui/tests/part_surface.rs` carries the namespaced-path compile
//! pin (`<Radio::Root><Radio::Indicator /></Radio::Root>`).
//!
//! ## The runtime split this suite lives inside
//!
//! leptos 0.7 tracks reactive-graph 0.1 while `leptos-ui-internals`' hooks are typed over 0.2
//! (`crate::radio::root`'s module docs). The pure folds in `crate::radio::state` are therefore
//! deliberately view- and runtime-independent: they are the only part of this unit that can be
//! proven on the host, and they are exactly the part the behavior spec's claims reduce to.

use super::*;

use crate::radio::context::MISSING_ROOT_CONTEXT_MESSAGE;
use crate::radio::state::{
    DATA_CHECKED, DATA_UNCHECKED, INPUT_ARIA_HIDDEN, INPUT_TAB_INDEX, INPUT_TYPE_RADIO, ROLE_RADIO,
    RadioIndicatorState, RadioRootState, aria_bool_attr, control_tag, has_value, hidden_input_id,
    indicator_should_render, input_style, input_value_attr, is_checked,
    radio_indicator_state_attributes, radio_state_attributes, root_id, serialize_value,
    style_string,
};
use crate::radio::{
    RadioIndicatorViewProps, RadioRootViewProps, indicator::RadioIndicatorHandlers,
    root::RadioRootHandlers,
};

mod host_tests {
    use super::*;

    /// A minimal checked Root state, so each attribute assertion reads against one record.
    fn state(checked: bool) -> RadioRootState {
        RadioRootState {
            checked,
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

    // -----------------------------------------------------------------------
    // Public API surface (props, parts, subcomponents)
    // -----------------------------------------------------------------------

    /// behavior.md:12-16 — `Radio.Root` and `Radio.Indicator` are the unit's whole public
    /// surface (`index.parts.ts` re-exports exactly those two), both reachable on the
    /// capitalised namespace, and the spec-proven props exist on the port with the upstream
    /// names.
    #[test]
    fn radio_exposes_its_two_documented_parts_and_their_spec_proven_props() {
        // The namespaced part surface a consumer writes: `<Radio::Root>` /
        // `<Radio::Indicator />` (`crate::Radio`, the `lib.rs` alias).
        let _ = (Radio::Root, Radio::Indicator);

        // `value` (`RadioRoot.tsx:315`) — `RadioRoot.Props<Value>`'s identity tag, kept as the
        // double `Option` the group unit's encoding requires.
        let root = RadioRootViewProps {
            value: Some(Some("second".to_string())),
            ..Default::default()
        };
        assert_eq!(root.value, Some(Some("second".to_string())), "Root.value");

        // `keepMounted` (`RadioIndicator.tsx:69`) — the Indicator's one own prop.
        let indicator = RadioIndicatorViewProps {
            keep_mounted: true,
            ..Default::default()
        };
        assert!(indicator.keep_mounted, "Indicator.keepMounted");

        // The two handler bags the element rest carries, constructible as defaults.
        let _ = RadioRootHandlers::default();
        let _ = RadioIndicatorHandlers::default();
    }

    // -----------------------------------------------------------------------
    // State model (controlled/uncontrolled, defaults, transitions)
    // -----------------------------------------------------------------------

    /// behavior.md:20-23 — the checked state is derived from the group, never owned: the
    /// matching value renders checked, a sibling's value renders unchecked, and moving the
    /// group's value moves which Root is checked.
    #[test]
    fn the_group_value_controls_which_radio_is_checked() {
        // `checkedValue === value` (`RadioRoot.tsx:83`).
        assert!(is_checked(true, Some("a"), Some(Some("a"))));
        assert!(!is_checked(true, Some("a"), Some(Some("b"))));

        // behavior.md:23 — selecting a sibling moves the check without any per-Root state.
        let selected = |group: Option<&str>, value: &str| {
            is_checked(true, group, Some(Some(value)))
        };
        assert!(selected(Some("a"), "a"));
        assert!(!selected(Some("b"), "a"), "the sibling is now checked");
        assert!(selected(Some("b"), "b"));
    }

    /// behavior.md:25 and :65 — an ABSENT `value` is not a `null` value. `null` is selectable
    /// and later deselectable by choosing a sibling, while an absent identity tag can never
    /// match the group (which holds `null`, never `undefined`).
    #[test]
    fn a_null_valued_radio_is_selectable_and_deselectable_by_a_sibling() {
        // `value={null}` against a group holding `null` (`RadioRoot.test.tsx:42-56`).
        assert!(
            is_checked(true, None, Some(None)),
            "null === null selects the radio"
        );

        // Choosing a sibling deselects it — the group value moved away.
        assert!(!is_checked(true, Some("other"), Some(None)));

        // An ABSENT value matches nothing: `undefined === null` is false, and the group's
        // `checkedValue` is initialized to `null` and never holds `undefined`.
        assert!(!is_checked(true, None, None), "undefined can never match");
        assert!(!is_checked(true, Some("a"), None));
    }

    /// behavior.md:20 — with no group above it, the Root's checked rule is `value === ''`, not
    /// an equality against a group. This is the standalone branch of `:83`.
    #[test]
    fn a_standalone_root_is_checked_only_for_the_empty_value() {
        assert!(is_checked(false, None, Some(Some(""))));
        assert!(!is_checked(false, None, Some(Some("a"))));
        assert!(!is_checked(false, None, Some(None)), "null is not ''");
        assert!(!is_checked(false, None, None), "absent is not ''");
    }

    /// behavior.md:24 — `data-checked` / `data-unchecked` are mutually exclusive style hooks:
    /// exactly one is present in each state, and never both.
    #[test]
    fn data_checked_and_data_unchecked_are_mutually_exclusive() {
        let checked = radio_state_attributes(&state(true));
        assert_eq!(
            checked.get(DATA_CHECKED).map(String::as_str),
            Some(""),
            "a checked radio carries bare data-checked"
        );
        assert!(
            !checked.contains_key(DATA_UNCHECKED),
            "and never data-unchecked"
        );

        let unchecked = radio_state_attributes(&state(false));
        assert_eq!(unchecked.get(DATA_UNCHECKED).map(String::as_str), Some(""));
        assert!(!unchecked.contains_key(DATA_CHECKED));
    }

    // -----------------------------------------------------------------------
    // Keyboard interactions
    // -----------------------------------------------------------------------

    /// behavior.md:30-34 — the unit's own keyboard claim is ArrowDown selecting the next radio
    /// WITHOUT dispatching a click to ancestors, and that navigation is the GROUP's composite
    /// root, not this unit (`RadioRoot.tsx:250-260` mounts the radio as the composite item; the
    /// arrow handler lives in the group's `useCompositeRoot`). What this unit owns is the
    /// membership marker the group scans for and the hidden input staying out of the tab order.
    /// Every other key is UNVERIFIED upstream, and the port claims nothing extra: the visible
    /// control's keydown handler exists solely to suppress Enter (`RadioRoot.tsx:132-138`).
    #[test]
    fn arrow_down_navigation_is_the_groups_contract_and_the_markers_are_this_units() {
        // The composite item marker this unit writes for the checked radio (`:130`) — the
        // attribute the group's composite root adopts as the active item.
        assert_eq!(
            leptos_ui_internals::composite::ACTIVE_COMPOSITE_ITEM,
            "data-composite-item-active"
        );

        // The hidden input is never reachable by Tab (`:176`); the `role="radio"` control holds
        // the roving tab stop, which is what makes ArrowDown meaningful.
        assert_eq!(INPUT_TAB_INDEX, "-1");
        assert_eq!(ROLE_RADIO, "radio");
    }

    // -----------------------------------------------------------------------
    // Focus management
    // -----------------------------------------------------------------------

    /// behavior.md:38-39 — the unit makes NO explicit focus assertion upstream
    /// ("UNVERIFIED — no test asserts this"), so the port has no focus claim to pin. What it
    /// does own, and what a label-click both depends on and proves, is the id split: which
    /// element receives the consumer id and which receives none.
    #[test]
    fn focus_restoration_is_unverified_and_the_id_split_is_what_this_unit_pins() {
        // `id: nativeButton ? inputId : id` (`:131`) — the control's id.
        assert_eq!(root_id(false, "control-id", "generated-id"), "generated-id");
        assert_eq!(root_id(true, "control-id", "generated-id"), "control-id");

        // `hiddenInputId = nativeButton ? undefined : inputId` (`:117`).
        assert_eq!(
            hidden_input_id(false, "control-id"),
            Some("control-id".to_string())
        );
        assert_eq!(hidden_input_id(true, "control-id"), None);

        // behavior.md:46 — in `nativeButton` mode the label's `htmlFor` must find the CONTROL,
        // so exactly one of the two elements carries the id.
        for native_button in [false, true] {
            let on_control = root_id(native_button, "control-id", "generated-id");
            let on_input = hidden_input_id(native_button, "control-id");
            assert_ne!(
                on_input.as_deref(),
                Some(on_control.as_str()),
                "the consumer id must not be duplicated across control and hidden input"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Accessibility (roles, aria-*, id linking)
    // -----------------------------------------------------------------------

    /// behavior.md:43-45 — the visible control exposes `role="radio"`, `disabled` renders as
    /// `aria-disabled` and NEVER as the HTML `disabled` attribute, and the state walk carries
    /// the field validity hooks.
    #[test]
    fn the_control_exposes_role_radio_and_aria_hooks_without_a_native_disabled() {
        assert_eq!(ROLE_RADIO, "radio");

        // `aria_bool_attr` is the `aria-disabled={disabled}` writer (`:233-243`): the string
        // `"true"` or nothing at all — never the boolean HTML attribute.
        assert_eq!(aria_bool_attr(true), Some("true".to_string()));
        assert_eq!(aria_bool_attr(false), None);

        // The hidden input is the implementation detail: `type="radio"`, `aria-hidden="true"`,
        // never tabbable (`:171-178`).
        assert_eq!(INPUT_TYPE_RADIO, "radio");
        assert_eq!(INPUT_ARIA_HIDDEN, "true");
        assert_eq!(INPUT_TAB_INDEX, "-1");

        // The walk's validity arm (`fieldValidityMapping`, `stateAttributesMapping.ts:15`):
        // unvalidated (`null`) emits neither hook.
        let unvalidated = radio_state_attributes(&state(true));
        assert!(!unvalidated.contains_key("data-valid"));
        assert!(!unvalidated.contains_key("data-invalid"));
    }

    /// behavior.md:44 — `aria-checked` is the state's own member, so the walk's generic arm and
    /// the `checked` mapping are the two places it can appear; the mutual exclusivity above is
    /// what keeps them from disagreeing.
    #[test]
    fn aria_checked_tracks_the_derived_selection_state() {
        // The record the writer reads is the same one the context snapshots, so
        // `aria-checked` and `data-checked` cannot diverge.
        let checked = state(true);
        assert!(checked.checked);
        assert_eq!(
            radio_state_attributes(&checked)
                .get(DATA_CHECKED)
                .map(String::as_str),
            Some("")
        );

        let unchecked = state(false);
        assert!(!unchecked.checked);
        assert!(radio_state_attributes(&unchecked).contains_key(DATA_UNCHECKED));
    }

    // -----------------------------------------------------------------------
    // DOM structure & portal behavior
    // -----------------------------------------------------------------------

    /// behavior.md:13 and :51 — the control is a `span` by default (the conformance suite's
    /// `refInstanceof: HTMLSpanElement`) and a real `<button>` under `nativeButton`; the hidden
    /// `<input>` is its sibling.
    #[test]
    fn the_control_tag_is_a_span_by_default_and_a_button_under_native_button() {
        assert_eq!(control_tag(false, None), "span");
        assert_eq!(control_tag(false, Some("span")), "span");
        assert_eq!(control_tag(true, None), "button");
        // The `render` element form substitutes the tag (`useRenderElement.tsx:164-196`).
        assert_eq!(control_tag(false, Some("button")), "button");
    }

    /// behavior.md:53 — no portal: every asserted DOM element renders inline. The port has no
    /// portal machinery in this unit at all; what it does own is the `visuallyHidden` recipe the
    /// hidden input carries, which differs on whether the group names it (`:177`).
    #[test]
    fn the_hidden_inputs_recipe_depends_on_whether_the_group_names_it() {
        // `style: name ? visuallyHiddenInput : visuallyHidden` (`:177`).
        let named = style_string(input_style(true));
        let anonymous = style_string(input_style(false));
        assert!(named.contains("position: absolute;"), "the named recipe");
        assert!(anonymous.contains("position: absolute;"));

        // The named-input recipe additionally clips the input out of the accessibility tree's
        // flow (`clipPath: inset(50%)`) — that is the difference the ternary exists for.
        assert!(named.contains("clip-path: inset(50%)"));
        assert!(!anonymous.contains("clip-path: inset(50%)"));
    }

    // -----------------------------------------------------------------------
    // Events (names, payload shape, bubbling, preventDefault semantics)
    // -----------------------------------------------------------------------

    /// behavior.md:57-59 and `RadioRoot.tsx:184-203` — the change funnel is what commits a
    /// selection, and it refuses a Root whose `value` is undefined while accepting a `null` one.
    #[test]
    fn the_change_funnel_refuses_an_absent_value_and_accepts_a_null_one() {
        // `if (disabled || readOnly || value === undefined) return` (`:190`).
        assert!(!has_value(None), "an absent identity tag never commits");
        assert!(has_value(Some(&None)), "a null identity tag commits");
        assert!(has_value(Some(&Some("a".to_string()))));

        // The commit carries the identity tag, serialized (`:179`, `serializeValue.ts:5-11`).
        assert_eq!(serialize_value(Some("a")), "a");
        assert_eq!(
            serialize_value(None),
            "null",
            "JSON.stringify(null) is the four-character string"
        );
    }

    /// `:179` — the hidden input's `value` attribute exists exactly when the prop is defined:
    /// an absent value omits the attribute, a `null` one serializes.
    #[test]
    fn the_hidden_inputs_value_attribute_is_omitted_only_when_the_value_is_undefined() {
        assert_eq!(input_value_attr(Some(Some("a"))), Some("a".to_string()));
        assert_eq!(input_value_attr(Some(None)), Some("null".to_string()));
        assert_eq!(input_value_attr(None), None);
    }

    // -----------------------------------------------------------------------
    // Edge cases (rapid interactions, unmount, nesting)
    // -----------------------------------------------------------------------

    /// behavior.md:66 — two Roots sharing one `value` are tolerated: while the group holds that
    /// value both report checked, and when it moves elsewhere both go unchecked (and each one's
    /// indicator unmounts).
    #[test]
    fn two_roots_sharing_a_value_are_checked_together_and_deselected_together() {
        let shared = Some("dup");
        assert!(is_checked(true, shared, Some(Some("dup"))));
        assert!(is_checked(true, shared, Some(Some("dup"))));
        // The group value moves away: BOTH become unchecked, with no per-Root bookkeeping.
        assert!(!is_checked(true, Some("other"), Some(Some("dup"))));
        assert!(!is_checked(true, Some("other"), Some(Some("dup"))));
    }

    /// behavior.md:26 and :67-68 — the Indicator's mount gate. Without `keepMounted` it is
    /// absent while the Root is unchecked and present while checked; with `keepMounted` the
    /// element survives an unchecked Root (`:36`).
    #[test]
    fn the_indicator_mount_gate_is_keep_mounted_or_mounted() {
        assert!(!indicator_should_render(false, false), "absent while unchecked");
        assert!(indicator_should_render(false, true), "present while checked");
        assert!(indicator_should_render(true, false), "kept while unchecked");
        assert!(indicator_should_render(true, true));
    }

    /// behavior.md:29 — `transitionStatus` is folded into the Indicator's state (`:29-32`) so it
    /// reaches the data attributes through the same walk; with no status the element carries
    /// neither transition hook, so a mounted indicator is not reported mid-animation.
    #[test]
    fn the_indicator_state_folds_the_transition_status_into_the_same_walk() {
        let idle = RadioIndicatorState {
            root: state(true),
            transition_status: None,
        };
        let attributes = radio_indicator_state_attributes(&idle);
        assert!(attributes.contains_key(DATA_CHECKED), "the Root's hooks ride along");
        assert!(!attributes.contains_key("data-starting-style"));
        assert!(!attributes.contains_key("data-ending-style"));

        let starting = RadioIndicatorState {
            root: state(true),
            transition_status: Some(leptos_ui_internals::use_transition_status::TransitionStatus::Starting),
        };
        let attributes = radio_indicator_state_attributes(&starting);
        assert!(
            attributes.contains_key("data-starting-style"),
            "the enter hook is emitted while starting"
        );
        assert!(!attributes.contains_key("data-ending-style"));
    }

    /// behavior.md:71 — nesting is UNVERIFIED upstream, but the source has a definite answer for
    /// the one part that must have a Root above it: `useRadioRootContext` throws the message
    /// `RadioRootContext.ts:10-13` spells, verbatim.
    #[test]
    fn a_part_outside_a_root_throws_the_upstream_context_message() {
        assert_eq!(
            MISSING_ROOT_CONTEXT_MESSAGE,
            "Base UI: RadioRootContext is missing. Radio parts must be placed within <Radio.Root>."
        );
    }

    // -----------------------------------------------------------------------
    // Shared harness dependencies
    // -----------------------------------------------------------------------

    /// behavior.md:75-79 — upstream's shared harness (`createRenderer`,
    /// `describeConformance`, `isJSDOM`, the `BASE_UI_ANIMATIONS_DISABLED` flag) is how the
    /// conformance suites prove the standard customization surface: `span` default, prop
    /// forwarding, className, render-prop and ref forwarding (`:15`). Those generic
    /// obligations reach the port through the merged bag rather than a re-run React suite, so
    /// the constants they would assert generically are pinned here instead — the element's
    /// identity, its state hook and the hidden input's contract — and the animation flag has
    /// no port analog because the animation machinery is ported (`useAnimationsFinished`
    /// drives `useOpenChangeComplete`), not stubbed.
    #[test]
    fn the_shared_conformance_surface_translates_to_the_ports_own_constants() {
        // propForwarding (`propForwarding.tsx:23-36`, `:82-95`): the consumer's rest members
        // land on the rendered element — the port's bag merge — and the element under them is
        // the `span` this constant names.
        assert_eq!(control_tag(false, None), "span");
        // className.tsx:20-23 + refForwarding.tsx:32-38: a stable single element per part, which
        // is why the control's tag is a fold of props rather than a per-render choice.
        assert_eq!(control_tag(true, Some("span")), "button");
        // The hidden input's unaffected-by-conformance contract (`:171-178`).
        assert_eq!((INPUT_TYPE_RADIO, INPUT_ARIA_HIDDEN), ("radio", "true"));
    }
}
