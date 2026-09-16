//! Tests for the input port — the claims `specs/library/input/behavior.md` proves for
//! `library: input`, pinned over the one thing this unit actually is: the props mapping
//! (`crates/leptos-ui/src/input.rs`, module docs).
//!
//! Upstream's `Input` is a pure delegation wrapper — a `React.forwardRef` whose body is
//! `return <Field.Control ref={forwardedRef} {...props} />`
//! (`packages/react/src/input/Input.tsx:12-17`), calling no hooks, reading no context and owning
//! no state (`specs/library/input/implementation.md`, "State machine / hooks used"). So the host
//! suite below asserts the delegation contract field by field, and each behavior.md section gets
//! its own test — including the sections behavior.md marks N/A, where the honest claim is that
//! the wrapper contributes nothing to that area (a claim worth pinning: it is what makes "Input
//! adds no behaviour of its own" more than a slogan).
//!
//! ## What this suite deliberately does NOT claim
//!
//! The DOM consequences of the delegation — the rendered `<input>`, the field-state `data-*`
//! battery, the value/`aria-*` wiring — are `Field.Control`'s own contract and are pinned by the
//! field unit's suite (`crates/leptos-ui/src/field_tests.rs`) plus
//! `FieldControl.test.tsx:530-558`'s pass-through, not re-asserted here. This box cannot run a
//! browser (`ralph/generated/env-health.json`: `browser: DEGRADED — browser gates REFUSED here by
//! lib/browser-budget.mjs (4 GB cgroup)`) and no CI job runs the crate's wasm suite
//! (`tooling: the crate DOM/wasm suite runs nowhere but this box`), so no DOM claim is made in
//! this file's name: the close rests on the host assertions below.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::*;
use crate::field::field_control::{FieldControlViewProps, ValueChangeEventDetails};

mod host_tests {
    use super::*;

    /// The delegation contract (`packages/react/src/input/Input.tsx:16`): every member the
    /// caller passed arrives at the control unchanged, and no member is invented.
    fn assert_same_control_bag(actual: &FieldControlViewProps, expected: &FieldControlViewProps) {
        assert_eq!(actual.id, expected.id, "id");
        assert_eq!(actual.name, expected.name, "name");
        assert_eq!(actual.value, expected.value, "value");
        assert_eq!(actual.default_value, expected.default_value, "default_value");
        assert_eq!(actual.disabled, expected.disabled, "disabled");
        assert_eq!(actual.auto_focus, expected.auto_focus, "auto_focus");
        assert_eq!(actual.class, expected.class, "class");
        assert_eq!(
            actual.element_attributes, expected.element_attributes,
            "element_attributes"
        );
        assert!(
            same_callback(&actual.on_value_change, &expected.on_value_change),
            "on_value_change must be the caller's own callback, not a re-wrap"
        );
    }

    fn same_callback(
        actual: &Option<InputChangeHandler>,
        expected: &Option<InputChangeHandler>,
    ) -> bool {
        match (actual, expected) {
            (None, None) => true,
            (Some(actual), Some(expected)) => Rc::ptr_eq(actual, expected),
            _ => false,
        }
    }

    // -----------------------------------------------------------------------
    // Public API surface (props, parts, subcomponents)
    // -----------------------------------------------------------------------

    /// behavior.md, "Public API surface": "`className` — accepted as a string and applied to the
    /// root element" (`packages/react/test/conformanceTests/className.tsx:20-23`). The wrapper's
    /// half of that claim is the mapping onto the control's `class`; the control is what applies
    /// it to the rendered node.
    #[test]
    fn class_name_is_applied_to_the_root_through_the_controls_own_class_member() {
        let control = input_view_props(InputViewProps {
            class: Some("input-root".to_string()),
            ..InputViewProps::default()
        });

        assert_eq!(control.class.as_deref(), Some("input-root"));
        // A bare `<Input />` asks for no class at all — upstream's `className === undefined`.
        assert_eq!(
            input_view_props(InputViewProps::default()).class,
            None,
            "an omitted className stays omitted rather than becoming an empty string"
        );
    }

    /// behavior.md, "Public API surface": "`<Input />` is a single-part component with no
    /// subcomponents; the entire suite mounts the bare element with no wrapper parts"
    /// (`packages/react/src/input/Input.test.tsx:9-12`). There is therefore no `Input::Root` to
    /// add (contrast `crates/leptos-ui/src/field/mod.rs:36-56`, whose unit documents seven parts),
    /// and the wrapper's own contribution to the props bag must be empty.
    #[test]
    fn the_surface_is_the_single_input_part_with_no_subcomponents() {
        let bare = input_view_props(InputViewProps::default());

        assert_same_control_bag(&bare, &FieldControlViewProps::default());
        assert!(
            bare.element_attributes.is_empty(),
            "a bare <Input /> must not fabricate a single attribute of its own"
        );
    }

    /// behavior.md, "Public API surface": arbitrary DOM props (`lang`, `data-*`, `data-testid`,
    /// `style`) "spread onto the default root element"
    /// (`packages/react/test/conformanceTests/propForwarding.tsx:23-36`). The port's path for
    /// that spread is the control's `element_attributes`
    /// (`crates/leptos-ui/src/field/field_control.rs:583-601`) — the `...elementProps` rest — so
    /// the bag must arrive in the caller's order and unaltered. This is also the surface the
    /// unit's own docs hero needs: it renders `<Input placeholder="…" />`
    /// (`specs/docs-content/input/demos.json`, `propsExercised.Input: ["placeholder","className"]`).
    #[test]
    fn arbitrary_dom_props_spread_onto_the_control_in_the_callers_order() {
        let control = input_view_props(InputViewProps {
            element_attributes: vec![
                ("placeholder".to_string(), "Name".to_string()),
                ("lang".to_string(), "en".to_string()),
                ("data-testid".to_string(), "root".to_string()),
                ("required".to_string(), String::new()),
            ],
            ..InputViewProps::default()
        });

        assert_eq!(
            control.element_attributes,
            vec![
                ("placeholder".to_string(), "Name".to_string()),
                ("lang".to_string(), "en".to_string()),
                ("data-testid".to_string(), "root".to_string()),
                ("required".to_string(), String::new()),
            ]
        );
    }

    /// `style` (BaseUIComponentProps, `packages/react/src/internals/types.ts:36-61`) is folded
    /// into the same element bag, because that is upstream's own path — its `{...props}` spread
    /// (`packages/react/src/input/Input.tsx:16`) carries `style` into `...elementProps`. The fold
    /// must not reorder the caller's members ahead of their own `style`, so a caller-supplied
    /// `style` member still occupies the LAST position and keeps upstream's later-bag-wins
    /// outcome (`packages/react/src/merge-props/mergeProps.ts:166-184`).
    #[test]
    fn the_style_declarations_ride_the_element_bag_and_the_callers_own_style_still_wins() {
        let control = input_view_props(InputViewProps {
            style: vec![
                ("color".to_string(), "red".to_string()),
                ("display".to_string(), "block".to_string()),
            ],
            element_attributes: vec![
                ("placeholder".to_string(), "Name".to_string()),
                ("style".to_string(), "color: blue;".to_string()),
            ],
            ..InputViewProps::default()
        });

        assert_eq!(
            control.element_attributes,
            vec![
                ("style".to_string(), "color: red; display: block;".to_string()),
                ("placeholder".to_string(), "Name".to_string()),
                ("style".to_string(), "color: blue;".to_string()),
            ],
            "the component style joins the bag first; the caller's later member still wins"
        );
        // No style declared ⇒ no style member at all, which is what keeps the bare bag equal to
        // the control's own default (the previous test).
        assert_eq!(
            style_declarations(&[]),
            "",
            "an empty declaration list formats to the empty string"
        );
    }

    /// The declaration spelling itself — the crate's shared writer format
    /// (`crates/leptos-ui/src/fieldset/root.rs:336-344`,
    /// `crates/leptos-ui/src/checkbox/root.rs:1281-1289`).
    #[test]
    fn the_style_declaration_spelling_matches_the_crates_own_writer() {
        assert_eq!(
            style_declarations(&[
                ("color".to_string(), "red".to_string()),
                ("display".to_string(), "block".to_string()),
            ]),
            "color: red; display: block;"
        );
    }

    // -----------------------------------------------------------------------
    // State model (controlled/uncontrolled, defaults, transitions)
    // -----------------------------------------------------------------------

    /// behavior.md marks this section N/A — the unit's own suite never tests `value`,
    /// `defaultValue` or any transition. What IS documented is that `InputProps` re-declares the
    /// two value members (`packages/react/src/input/Input.tsx:27,31`) and that the *control*
    /// owns the `isControlled = valueProp !== undefined` branch
    /// (`packages/react/src/field/control/FieldControl.tsx:84-87`), so the wrapper must carry
    /// both members separately and collapse neither: downstream, the DOM value comes from
    /// `value` when controlled and `defaultValue` when not
    /// (`packages/react/src/field/control/FieldControl.tsx:138`).
    #[test]
    fn controlled_value_and_uncontrolled_default_reach_the_control_separately() {
        let controlled = input_view_props(InputViewProps {
            value: Some("typed".to_string()),
            default_value: None,
            ..InputViewProps::default()
        });
        assert_eq!(controlled.value.as_deref(), Some("typed"));
        assert_eq!(controlled.default_value, None);

        let uncontrolled = input_view_props(InputViewProps {
            value: None,
            default_value: Some("seed".to_string()),
            ..InputViewProps::default()
        });
        assert_eq!(uncontrolled.value, None);
        assert_eq!(uncontrolled.default_value.as_deref(), Some("seed"));

        // No member is synthesized: an omitted value is `undefined` upstream, not `""`.
        let neither = input_view_props(InputViewProps::default());
        assert_eq!((neither.value, neither.default_value), (None, None));
    }

    // -----------------------------------------------------------------------
    // Keyboard interactions
    // -----------------------------------------------------------------------

    /// behavior.md marks this section N/A, and the wrapper cannot change that: the control's
    /// props bag carries exactly one handler slot, `on_value_change`
    /// (`crates/leptos-ui/src/field/field_control.rs:70-92`), which is the change path, not a
    /// keydown path — the Enter-commits semantics belong to the control's own view
    /// (`packages/react/src/field/control/FieldControl.tsx:188-207`). So the mapping below must
    /// be the identity over the caller's members: there is no slot through which keyboard
    /// behaviour could be added, and none is added.
    #[test]
    fn keyboard_interactions_are_the_controls_own_and_the_wrapper_adds_no_keyboard_handling() {
        let caller = InputViewProps {
            id: Some("field-1".to_string()),
            name: Some("quantity".to_string()),
            ..InputViewProps::default()
        };
        let expected = FieldControlViewProps {
            id: Some("field-1".to_string()),
            name: Some("quantity".to_string()),
            ..FieldControlViewProps::default()
        };

        assert_same_control_bag(&input_view_props(caller), &expected);
    }

    // -----------------------------------------------------------------------
    // Focus management
    // -----------------------------------------------------------------------

    /// behavior.md marks this section N/A beyond "the default root is an `HTMLInputElement`
    /// instance (a natively focusable element)" (`packages/react/src/input/Input.test.tsx:10`).
    /// The one focus-shaped member the port does have is the control's own `autoFocus`
    /// (`packages/react/src/field/control/FieldControl.tsx:46`, whose port reconciles it in the
    /// hydration effect, `crates/leptos-ui/src/field/field_control.rs:121-125`), so it is
    /// forwarded — and nothing else focus-related is invented.
    #[test]
    fn focus_management_forwards_the_controls_own_auto_focus_and_invents_nothing_else() {
        let focused = input_view_props(InputViewProps {
            auto_focus: true,
            ..InputViewProps::default()
        });
        assert!(focused.auto_focus, "autoFocus reaches the control");

        let unfocused = input_view_props(InputViewProps::default());
        assert!(!unfocused.auto_focus, "and stays false when not asked for");

        assert_same_control_bag(
            &unfocused,
            &FieldControlViewProps::default(),
        );
    }

    // -----------------------------------------------------------------------
    // Accessibility (roles, aria-*, id linking)
    // -----------------------------------------------------------------------

    /// behavior.md marks this section N/A: "the suite makes no explicit role, `aria-*`, or
    /// id-linking assertions", and the port's ARIA wiring is the control's own — `aria-labelledby`
    /// from the labelable id and the merged `aria-describedby`
    /// (`crates/leptos-ui/src/field/field_control.rs:529-546`). The wrapper's obligation is
    /// therefore to pass the caller's own ARIA members through the element bag untouched and to
    /// add none of its own.
    #[test]
    fn accessibility_the_wrapper_adds_no_aria_and_the_callers_members_pass_through() {
        let control = input_view_props(InputViewProps {
            element_attributes: vec![
                ("aria-labelledby".to_string(), "lbl".to_string()),
                ("aria-describedby".to_string(), "desc".to_string()),
            ],
            ..InputViewProps::default()
        });

        assert_eq!(
            control.element_attributes,
            vec![
                ("aria-labelledby".to_string(), "lbl".to_string()),
                ("aria-describedby".to_string(), "desc".to_string()),
            ]
        );
        // No `role` member is fabricated: the control's view sets the element's own semantics by
        // rendering a native `<input>` (behavior.md, "the default rendered element is a native
        // `<input>`"), not by writing a role attribute.
        assert!(
            !control
                .element_attributes
                .iter()
                .any(|(name, _)| name == "role"),
            "the wrapper must not add a role attribute of its own"
        );
    }

    // -----------------------------------------------------------------------
    // DOM structure & portal behavior
    // -----------------------------------------------------------------------

    /// behavior.md, "DOM structure & portal behavior": "Default DOM: one root element (the native
    /// `<input>`)"; "No portal behavior is exercised by the tests: the component renders inline
    /// where placed". The port keeps that by delegating: `Input` builds no node of its own, and
    /// the bag it hands over is exactly the control's own default for a bare call — so the only
    /// element in the tree is the control's `<input>`
    /// (`crates/leptos-ui/src/field/field_control.rs:604-628`), with no wrapper and no portal.
    #[test]
    fn the_dom_structure_is_the_controls_own_input_with_no_wrapper_node_or_portal() {
        let delegated = input_view_props(InputViewProps::default());

        assert_same_control_bag(&delegated, &FieldControlViewProps::default());
        assert_eq!(
            delegated.element_attributes.len(),
            0,
            "a bare delegation adds no attribute, so it cannot add a wrapper element either"
        );
    }

    // -----------------------------------------------------------------------
    // Events (names, payload shape, bubbling, preventDefault semantics)
    // -----------------------------------------------------------------------

    /// behavior.md marks this section N/A at the unit level — no event dispatch or payload shape
    /// is asserted by `Input.test.tsx` — but `onValueChange` is one of the three props the unit
    /// re-declares (`packages/react/src/input/Input.tsx:23`), so it is part of the surface and
    /// must be the caller's own callback: the control calls it with the serialized value and a
    /// details object whose `cancel()` veto is read back from the shared cell
    /// (`crates/leptos-ui/src/field/field_control.rs:82-92`). A re-wrap here would silently break
    /// that identity, so it is pinned with pointer equality and then invoked through the control's
    /// slot.
    #[test]
    fn events_carry_the_callers_own_on_value_change_callback_unchanged() {
        let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&seen);
        let callback: InputChangeHandler = Rc::new(
            move |value: String, details: &ValueChangeEventDetails| {
                assert!(!details.is_canceled(), "the details arrive uncancelled");
                sink.borrow_mut().push(value);
            },
        );

        let control = input_view_props(InputViewProps {
            on_value_change: Some(Rc::clone(&callback)),
            ..InputViewProps::default()
        });

        let carried = control
            .on_value_change
            .as_ref()
            .expect("the caller's callback is carried to the control");
        assert!(
            Rc::ptr_eq(carried, &callback),
            "the wrapper hands the control the caller's own Rc"
        );

        let details = ValueChangeEventDetails {
            cancel_flag: Rc::new(Cell::new(false)),
            event: None,
        };
        carried("typed".to_string(), &details);
        assert_eq!(*seen.borrow(), vec!["typed".to_string()]);
    }

    // -----------------------------------------------------------------------
    // Edge cases (rapid interactions, unmount, nesting)
    // -----------------------------------------------------------------------

    /// behavior.md, "Edge cases": the only case the unit's suite covers is nesting — "the
    /// component tolerates being wrapped by an extra element when `render` supplies one"; rapid
    /// interactions and unmount cleanup are N/A. Since `render` is not part of this port's surface
    /// (module docs, "Recorded adaptations"), the honest wrapper-side claim is the one a props
    /// mapping can make and keep: it is STATELESS. Two identical calls produce identical bags, so
    /// nothing survives a re-run — a remount or a rapid re-invocation inherits no wrapper state,
    /// and every interaction invariant is the control's.
    #[test]
    fn edge_cases_the_mapping_is_stateless_so_remounts_and_rapid_interactions_are_the_controls() {
        let first = input_view_props(InputViewProps {
            class: Some("input-root".to_string()),
            element_attributes: vec![("placeholder".to_string(), "Name".to_string())],
            disabled: true,
            ..InputViewProps::default()
        });
        let second = input_view_props(InputViewProps {
            class: Some("input-root".to_string()),
            element_attributes: vec![("placeholder".to_string(), "Name".to_string())],
            disabled: true,
            ..InputViewProps::default()
        });

        assert_same_control_bag(&first, &second);
    }

    // -----------------------------------------------------------------------
    // Shared harness dependencies
    // -----------------------------------------------------------------------

    /// behavior.md, "Shared harness dependencies": the unit's sole test file "contains no
    /// hand-written behavioral tests. It delegates entirely to the shared conformance harness
    /// `describeConformance`, running the full default suite" — propsSpread, refForwarding,
    /// renderProp, className (`packages/react/test/describeConformance.tsx:44-67`). Two of those
    /// four suites are about the props mapping this module IS — `className`
    /// (`packages/react/test/conformanceTests/className.tsx:20-23`) and propsSpread
    /// (`packages/react/test/conformanceTests/propForwarding.tsx:23-36`) — and are pinned above;
    /// the other two (refForwarding, renderProp) are surface this port's delegation target does
    /// not carry, recorded as adaptations rather than claimed.
    #[test]
    fn the_conformance_suites_the_wrapper_owns_are_class_name_and_prop_spread() {
        let control = input_view_props(InputViewProps {
            class: Some("root-class".to_string()),
            element_attributes: vec![
                ("lang".to_string(), "en".to_string()),
                ("data-testid".to_string(), "root".to_string()),
                ("style".to_string(), "color: red;".to_string()),
            ],
            ..InputViewProps::default()
        });

        assert_eq!(control.class.as_deref(), Some("root-class"));
        assert_eq!(
            control.element_attributes,
            vec![
                ("lang".to_string(), "en".to_string()),
                ("data-testid".to_string(), "root".to_string()),
                ("style".to_string(), "color: red;".to_string()),
            ]
        );
    }
}
