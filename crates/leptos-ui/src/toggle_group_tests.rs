//! Tests for the toggle-group port — mirrors of
//! `packages/react/src/toggle-group/ToggleGroup.test.tsx` (the suite
//! `specs/library/toggle-group/behavior.md` mines) plus the `.spec.tsx`'s type-level
//! value claims, over the real materialized element in a wasm-browser suite and the
//! state-machine contracts in an owner-scoped host suite.
//!
//! Host suite: the description-level contracts that need no DOM (the transition table
//! and its JS edge semantics, the callback-before-commit order and the cancel veto, the
//! `isValueInitialized` gate, the state→`data-*` record, and the context the children
//! read).
//! Wasm suite: the materialized-element contracts (`role="group"`, the `data-*`
//! battery, a composed `Toggle` child's membership through the context, the group's
//! disabled propagation, the click→commit path, and the view path's own mount).
//!
//! ## What this suite deliberately does NOT claim
//!
//! The port's children re-derive from a value *snapshot* (`toggle_group.rs`, module
//! docs), so a test that clicks a child must re-evaluate `toggle_element` to observe the
//! new display — exactly the "a fresh evaluation reflects a controlled flip" precedent
//! `toggle_tests.rs` records. Nothing here asserts DOM-node identity across a value
//! change: the retained-node update belongs to the child composition (the `avatar`
//! retain/`update_element` pattern), not to this unit, and the toolbar-nested
//! obligations (`ToggleGroup.test.tsx:313-329,339-365`) are blocked on `library:
//! toolbar` (module docs, "Rust adaptations").

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use reactive_graph::traits::GetUntracked;

use super::*;

/// A callback recorder that satisfies the `Send + Sync` bound the landed
/// `ToggleGroupContext` puts on the committer and on `onValueChange` (the wasm single
/// thread makes the mutex uncontended — the toggle unit's `Recorder` convention).
#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<Vec<Vec<String>>>>);

impl Recorder {
    fn record(&self, next: Vec<String>) {
        self.0.lock().unwrap().push(next);
    }

    fn proposed(&self) -> Vec<Vec<String>> {
        self.0.lock().unwrap().clone()
    }
}

/// The shared change-event details (`ToggleGroup.tsx:59`), synthesized the way the
/// host suites do: no JS runtime is involved, so the native-event slot wraps a null
/// `JsValue` (the `menu_tests.rs` convention).
fn details() -> ToggleGroupChangeEventDetails {
    use leptos_ui_internals::floating_ui::reasons;
    ToggleGroupChangeEventDetails::new(
        reasons::NONE,
        web_sys::MouseEvent::from(web_sys::wasm_bindgen::JsValue::NULL),
        None,
        (),
    )
}

/// An owner for the signal-bearing constructors. The executor init is the crate's host
/// convention (`meter_tests.rs`/`progress_tests.rs`): `use_controlled`'s dev-only
/// diagnostics attach effects, and an effect that spawns without an executor panics.
fn in_owner() -> reactive_graph::owner::Owner {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = reactive_graph::owner::Owner::new();
    owner.set();
    owner
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // behavior.md "State model" single selection (`ToggleGroup.test.tsx:48-52,69-73`):
    // the single-mode arms replace the set with `[newValue]` or clear it — `:70`.
    #[test]
    fn the_single_selection_arms_replace_and_clear() {
        let current = vec!["one".to_string()];
        assert_eq!(
            next_group_value(&current, false, "two", true),
            vec!["two".to_string()],
            "single mode replaces the whole set"
        );
        assert_eq!(
            next_group_value(&current, false, "two", false),
            Vec::<String>::new(),
            "single mode unpressing clears the set"
        );
    }

    // behavior.md "State model" multiple (`:244-261,303-305`): the multiple arms push
    // onto a copy and splice out the first occurrence — `:63-67`.
    #[test]
    fn the_multiple_arms_accumulate_and_remove() {
        let current = vec!["one".to_string()];
        assert_eq!(
            next_group_value(&current, true, "two", true),
            vec!["one".to_string(), "two".to_string()],
            "multiple mode pushes the new value onto a copy"
        );
        assert_eq!(
            next_group_value(&current, true, "one", false),
            Vec::<String>::new(),
            "multiple mode removes the pressed value"
        );
        assert_eq!(
            current,
            vec!["one".to_string()],
            "the reducer never mutates its input (the EMPTY_ARRAY copy-before-mutate discipline)"
        );
    }

    // implementation.md "Anything in source not explained by any test" item 7, measured
    // rather than assumed: `groupValue.splice(groupValue.indexOf(newValue), 1)` with an
    // absent `newValue` is `splice(-1, 1)` — JS drops the LAST element. The port
    // reproduces it deliberately, so the quirk is pinned instead of silently "fixed".
    #[test]
    fn the_removal_arm_reproduces_js_splice_of_index_minus_one() {
        let current = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(
            next_group_value(&current, true, "absent", false),
            vec!["a".to_string(), "b".to_string()],
            "indexOf -1 → splice(-1, 1) drops the last element, exactly as upstream"
        );
        let empty: Vec<String> = Vec::new();
        assert_eq!(
            next_group_value(&empty, true, "absent", false),
            Vec::<String>::new(),
            "splice on an empty array is a no-op"
        );
    }

    // `isValueInitialized` (`:43`): the raw props decide, so an explicit empty
    // `defaultValue` counts as initialized while an omitted pair does not — the gate the
    // Toggle unit's missing-`value` warning reads (`:38,52`).
    #[test]
    fn is_value_initialized_reads_the_raw_props() {
        let cases: Vec<(ToggleGroupElementProps, bool)> = vec![
            (ToggleGroupElementProps::default(), false),
            (
                ToggleGroupElementProps {
                    value: Some(Vec::new()),
                    ..Default::default()
                },
                true,
            ),
            (
                ToggleGroupElementProps {
                    default_value: Some(Vec::new()),
                    ..Default::default()
                },
                true,
                ),
        ];
        for (index, (props, expected)) in cases.into_iter().enumerate() {
            let _owner = in_owner();
            let runtime = ToggleGroupRuntime::new(&props);
            assert_eq!(
                runtime.is_value_initialized, expected,
                "isValueInitialized for case {index}",
            );
        }
    }

    // `ToggleGroupDataAttributes.ts:1-13` through `getStateAttributesProps`: booleans are
    // presence/absence, the orientation is a string, and `aria-orientation` is never
    // emitted (behavior.md, "Accessibility" — the `group` role has no orientation
    // property).
    #[test]
    fn the_state_map_matches_the_data_attribute_contract() {
        let off = ToggleGroupState {
            disabled: false,
            multiple: false,
            orientation: ToggleGroupOrientation::Horizontal,
        };
        let attributes = off.data_attributes();
        assert!(
            !attributes.iter().any(|(name, _)| name == "data-disabled"),
            "data-disabled is absent when enabled"
        );
        assert!(
            !attributes.iter().any(|(name, _)| name == "data-multiple"),
            "data-multiple is absent when single-selection"
        );
        assert_eq!(
            attributes
                .iter()
                .find(|(name, _)| name == "data-orientation")
                .map(|(_, value)| value.as_str()),
            Some("horizontal"),
            "data-orientation carries the orientation string"
        );
        assert!(
            !attributes
                .iter()
                .any(|(name, _)| name.starts_with("aria-")),
            "no aria attribute comes out of the state mapping"
        );

        let on = ToggleGroupState {
            disabled: true,
            multiple: true,
            orientation: ToggleGroupOrientation::Vertical,
        };
        let attributes = on.data_attributes();
        assert_eq!(
            attributes
                .iter()
                .find(|(name, _)| name == "data-disabled")
                .map(|(_, value)| value.as_str()),
            Some(""),
            "a true boolean emits a bare attribute"
        );
        assert_eq!(
            attributes
                .iter()
                .find(|(name, _)| name == "data-multiple")
                .map(|(_, value)| value.as_str()),
            Some(""),
        );
        assert_eq!(
            attributes
                .iter()
                .find(|(name, _)| name == "data-orientation")
                .map(|(_, value)| value.as_str()),
            Some("vertical"),
        );
    }

    // `:55-81`: `onValueChange` gets the *computed* next array and runs **before** the
    // commit (`:73-79`), and the commit lands on the value slot.
    #[test]
    fn the_committer_notifies_before_it_commits() {
        let _owner = in_owner();
        let recorder = Recorder::default();
        let recorder_for_callback = recorder.clone();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            default_value: Some(vec!["one".to_string()]),
            on_value_change: Some(Arc::new(move |next: Vec<String>, _details| {
                recorder_for_callback.record(next);
            })),
            ..Default::default()
        });

        assert_eq!(
            runtime.value.get_untracked(),
            vec!["one".to_string()],
            "the uncontrolled seed is defaultValue"
        );

        let commit = runtime.set_group_value();
        commit("two", true, &details());

        assert_eq!(
            recorder.proposed(),
            vec![vec!["two".to_string()]],
            "the callback receives the computed next array (single mode)"
        );
        assert_eq!(
            runtime.value.get_untracked(),
            vec!["two".to_string()],
            "the change commits after the callback returns"
        );
    }

    // behavior.md "Events" cancellation (`:546-564`): the handler still fires exactly
    // once, but the value does not change — the early return at `:75-77`.
    #[test]
    fn a_canceled_change_notifies_but_does_not_commit() {
        let _owner = in_owner();
        let recorder = Recorder::default();
        let recorder_for_callback = recorder.clone();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            default_value: Some(vec!["one".to_string()]),
            on_value_change: Some(Arc::new(move |next: Vec<String>, details| {
                recorder_for_callback.record(next);
                details.cancel();
            })),
            ..Default::default()
        });

        let commit = runtime.set_group_value();
        commit("two", true, &details());

        assert_eq!(
            recorder.proposed(),
            vec![vec!["two".to_string()]],
            "the caller observes the proposed value once"
        );
        assert_eq!(
            runtime.value.get_untracked(),
            vec!["one".to_string()],
            "a canceled change leaves the pressed set untouched"
        );
    }

    // behavior.md "State model" controlled (`:116-163`): while controlled the prop wins
    // and the setter is a no-op (`packages/utils/src/useControlled.ts:82-89`).
    #[test]
    fn a_controlled_value_wins_and_the_setter_is_a_no_op() {
        let _owner = in_owner();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            value: Some(vec!["one".to_string()]),
            default_value: Some(vec!["two".to_string()]),
            ..Default::default()
        });

        assert_eq!(
            runtime.value.get_untracked(),
            vec!["one".to_string()],
            "the controlled prop outranks the default"
        );

        let commit = runtime.set_group_value();
        commit("three", true, &details());
        assert_eq!(
            runtime.value.get_untracked(),
            vec!["one".to_string()],
            "clicks cannot move a controlled value"
        );
    }

    // `:85-93` as the child sees it: the context carries the value snapshot, the
    // effective disabled, the initialized gate, and a committer that routes through the
    // same veto-wrapped reducer.
    #[test]
    fn the_context_value_carries_the_state_the_toggles_read() {
        let _owner = in_owner();
        let recorder = Recorder::default();
        let recorder_for_callback = recorder.clone();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            default_value: Some(vec!["one".to_string()]),
            disabled: true,
            on_value_change: Some(Arc::new(move |next: Vec<String>, _details| {
                recorder_for_callback.record(next);
            })),
            ..Default::default()
        });

        let context = runtime.context_value(runtime.value.get_untracked());
        assert_eq!(
            context.value.as_ref(),
            &vec!["one".to_string()],
            "the snapshot is the array the membership probes read"
        );
        assert!(context.disabled, "the effective disabled crosses the context");
        assert!(
            context.is_value_initialized,
            "the dev-warning gate crosses the context"
        );

        let commit = context
            .set_group_value
            .as_ref()
            .expect("the group provides a committer");
        commit("two", true, &details());
        assert_eq!(
            recorder.proposed(),
            vec![vec!["two".to_string()]],
            "the context committer is the same veto-wrapped reducer"
        );
    }

    // The runtime's `disabled` is the effective value of `:45-46`; the toolbar arms do
    // not exist in this crate yet, so the property under test is that the prop passes
    // through unchanged rather than being dropped (`library: toolbar` owns the rest).
    #[test]
    fn the_effective_disabled_is_the_prop_outside_a_toolbar() {
        let _owner = in_owner();
        for disabled in [false, true] {
            let props = ToggleGroupElementProps {
                disabled,
                ..Default::default()
            };
            assert_eq!(props.state().disabled, disabled);
            assert_eq!(ToggleGroupRuntime::new(&props).disabled, disabled);
        }
    }

    // THE SEAM, tested where it can be tested without a browser: the provider must write
    // into the SAME context map the consuming unit reads. This workspace carries two
    // `reactive_graph` versions, and `leptos::prelude::provide_context` targets the one
    // the toggle unit's `use_toggle_group_context()` (a
    // `reactive_graph::owner::use_context`) never sees — measured this iteration as six
    // red wasm tests whose whole symptom set (standalone rendering, no composite
    // tabindex, no group commit) followed from exactly this mismatch. A provider that
    // writes to the wrong map still compiles, so the assertion has to read the context
    // back through the CONSUMER's own accessor.
    #[test]
    fn the_group_context_reaches_the_consuming_unit() {
        let _owner = in_owner();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            default_value: Some(vec!["one".to_string()]),
            disabled: true,
            ..Default::default()
        });
        let snapshot = runtime.value.get_untracked();
        provide_toggle_group_context(&runtime, snapshot.clone());

        let context = crate::toggle::use_toggle_group_context()
            .expect("the toggle unit must see the group context it consumes");
        assert_eq!(
            context.value.as_ref(),
            &snapshot,
            "the snapshot the provider wrote is what the consumer reads"
        );
        assert!(context.disabled, "the effective disabled crosses the seam");
        assert!(context.is_value_initialized);
    }

    // The membership derivation of behavior.md "Accessibility" (`:39-52,125-141`), end to
    // end on the host: a grouped `Toggle`'s `aria-pressed` is a function of the group's
    // snapshot, and the grouped path is the composite one. The attribute is a lazy
    // closure, so this reads it the way the element layer does — no DOM involved.
    #[test]
    fn a_child_toggle_derives_its_pressed_attribute_from_the_group_snapshot() {
        use crate::toggle::{ToggleProps, toggle_element};

        for (seed, expected_one, expected_two) in [
            (vec!["one".to_string()], "true", "false"),
            (vec!["two".to_string()], "false", "true"),
            (Vec::<String>::new(), "false", "false"),
        ] {
            let _owner = in_owner();
            // The composite root context the grouped `CompositeItem` path requires
            // (`toggle_tests.rs` builds the same harness shape).
            let any_index: reactive_graph::computed::Memo<i32> =
                reactive_graph::computed::Memo::new(|_| -1);
            leptos_ui_internals::composite_root_context::provide_composite_root_context(
                leptos_ui_internals::composite_root_context::CompositeRootContextValue {
                    highlighted_index: any_index,
                    on_highlighted_index_change: Rc::new(|_index: i32, _scroll: bool| {}),
                    highlight_item_on_hover: false,
                    relay_keyboard_event: Rc::new(|_event: &web_sys::KeyboardEvent| {}),
                },
            );

            let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
                default_value: Some(seed.clone()),
                ..Default::default()
            });
            let snapshot = runtime.value.get_untracked();
            provide_toggle_group_context(&runtime, snapshot);

            for (value, expected) in [("one", expected_one), ("two", expected_two)] {
                let rendered = toggle_element(ToggleProps {
                    value: Some(value.to_string()),
                    ..Default::default()
                })
                .expect("a grouped toggle renders through CompositeItem");
                let aria_pressed = rendered
                    .props
                    .handlers
                    .attributes
                    .iter()
                    .find(|(name, _)| name == "aria-pressed")
                    .map(|(_, value)| value())
                    .flatten();
                assert_eq!(
                    aria_pressed.as_deref(),
                    Some(expected),
                    "membership of '{value}' under seed {seed:?}"
                );
            }
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Element, Event, HtmlButtonElement};

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    use crate::toggle::{ToggleProps, toggle_element};

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().expect("window").document().expect("document")
    }

    /// One settled macrotask turn, so a mount's commit effects have run
    /// (`otp_field_view_tests.rs` convention).
    async fn flush_one_turn() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .expect("window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                .expect("set_timeout");
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.expect("turn");
    }

    fn click(element: &web_sys::Element) {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
        element
            .dispatch_event(event.dyn_ref::<Event>().unwrap())
            .unwrap();
    }

    /// The element path's harness: an owner, the group description (which provisions the
    /// group + composite contexts), then zero or more composed `Toggle` children appended
    /// into the group element — the order a consumer composes them.
    struct MountedGroup {
        root: Element,
        toggles: Vec<HtmlButtonElement>,
    }

    fn mount_group_with_toggles(
        props: ToggleGroupElementProps,
        toggle_values: &[&str],
    ) -> MountedGroup {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = toggle_group_element(props);
        let (root, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&root).unwrap();

        let toggles = toggle_values
            .iter()
            .map(|value| {
                let rendered = toggle_element(ToggleProps {
                    value: Some((*value).to_string()),
                    ..Default::default()
                })
                .expect("a grouped toggle renders through CompositeItem");
                let (element, cleanup) = rendered.create_element();
                std::mem::forget(cleanup);
                root.append_child(&element).unwrap();
                element.dyn_into::<HtmlButtonElement>().unwrap()
            })
            .collect();

        MountedGroup { root, toggles }
    }

    fn aria_pressed(button: &HtmlButtonElement) -> String {
        button
            .get_attribute("aria-pressed")
            .expect("aria-pressed present")
    }

    /// behavior.md "Accessibility" + "DOM structure": the root is a single `div` with
    /// `role="group"`, `data-orientation` from the prop, `data-multiple` only when true,
    /// and `data-disabled` only when disabled (`:13-16,209-241`).
    #[wasm_bindgen_test]
    fn the_group_root_carries_the_documented_attribute_surface() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = toggle_group_element(ToggleGroupElementProps {
            orientation: ToggleGroupOrientation::Vertical,
            multiple: true,
            element_attributes: vec![("data-foobar".to_string(), "yes".to_string())],
            ..Default::default()
        });
        let (root, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&root).unwrap();

        assert_eq!(root.tag_name(), "DIV", "the root is a single div");
        assert_eq!(
            root.get_attribute("role").as_deref(),
            Some("group"),
            "the default props bag carries role=group (`:95-97`)"
        );
        assert_eq!(
            root.get_attribute("data-orientation").as_deref(),
            Some("vertical")
        );
        assert!(
            root.has_attribute("data-multiple"),
            "multiple=true emits the bare attribute"
        );
        assert!(
            !root.has_attribute("data-disabled"),
            "disabled=false omits the attribute"
        );
        assert!(
            !root.has_attribute("aria-orientation"),
            "the state mapping emits data-* only"
        );
        assert_eq!(
            root.get_attribute("data-foobar").as_deref(),
            Some("yes"),
            "the consumer's ...elementProps rest lands on the root"
        );
    }

    /// behavior.md "State model"/"Accessibility": each `Toggle`'s `aria-pressed` reflects
    /// its membership in the group's value (`:39-52,125-141`). A fresh evaluation is the
    /// port's re-render analog, so the same context is read at a new snapshot.
    #[wasm_bindgen_test]
    fn a_toggle_child_reads_its_membership_from_the_group_value() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        for (seed, expected_one, expected_two) in [
            (vec!["one".to_string()], "true", "false"),
            (vec!["two".to_string()], "false", "true"),
            (Vec::<String>::new(), "false", "false"),
        ] {
            let rendered = toggle_group_element(ToggleGroupElementProps {
                default_value: Some(seed.clone()),
                ..Default::default()
            });
            let (root, cleanup) = rendered.create_element();
            std::mem::forget(cleanup);
            document().body().unwrap().append_child(&root).unwrap();

            let one = toggle_element(ToggleProps {
                value: Some("one".to_string()),
                ..Default::default()
            })
            .expect("renders");
            let (one, cleanup) = one.create_element();
            std::mem::forget(cleanup);
            let two = toggle_element(ToggleProps {
                value: Some("two".to_string()),
                ..Default::default()
            })
            .expect("renders");
            let (two, cleanup) = two.create_element();
            std::mem::forget(cleanup);

            assert_eq!(
                aria_pressed(&one.dyn_into::<HtmlButtonElement>().unwrap()),
                expected_one,
                "membership for 'one' with seed {seed:?}"
            );
            assert_eq!(
                aria_pressed(&two.dyn_into::<HtmlButtonElement>().unwrap()),
                expected_two,
                "membership for 'two' with seed {seed:?}"
            );
        }
    }

    /// behavior.md "State model" disabled propagation (`:175-180`): a disabled group
    /// reaches every child, which the toggle unit renders as the native `disabled`
    /// attribute plus `data-disabled`.
    #[wasm_bindgen_test]
    fn the_group_disabled_reaches_the_child_toggle() {
        let disabled = mount_group_with_toggles(
            ToggleGroupElementProps {
                disabled: true,
                ..Default::default()
            },
            &["one"],
        );
        assert!(
            disabled.toggles[0].disabled(),
            "the group's disabled reaches the child's native disabled"
        );
        assert!(
            disabled.toggles[0].has_attribute("data-disabled"),
            "and its data-disabled hook"
        );

        let enabled = mount_group_with_toggles(ToggleGroupElementProps::default(), &["one"]);
        assert!(!enabled.toggles[0].disabled());
        assert!(!enabled.toggles[0].has_attribute("data-disabled"));
    }

    /// behavior.md "Events" (`:533-543`): a click on a child commits through the group —
    /// `onValueChange` receives the full next array, and a fresh evaluation at the new
    /// snapshot shows the item pressed.
    #[wasm_bindgen_test]
    fn a_click_through_the_child_commits_the_group_value() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let recorder = Recorder::default();
        let recorder_for_callback = recorder.clone();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            default_value: Some(Vec::new()),
            on_value_change: Some(Arc::new(move |next: Vec<String>, _details| {
                recorder_for_callback.record(next);
            })),
            ..Default::default()
        });
        let snapshot = runtime.value.get_untracked();
        provide_toggle_group_context(&runtime, snapshot);

        let rendered = toggle_element(ToggleProps {
            value: Some("one".to_string()),
            ..Default::default()
        })
        .expect("renders");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        let button = element.dyn_into::<HtmlButtonElement>().unwrap();

        click(&button);

        assert_eq!(
            recorder.proposed(),
            vec![vec!["one".to_string()]],
            "the group's onValueChange receives the computed next array"
        );
        assert_eq!(
            runtime.value.get_untracked(),
            vec!["one".to_string()],
            "the click committed through the group's value slot"
        );

        // The display's re-derivation at the new snapshot (`toggle_group.rs` module docs:
        // the group re-runs the subtree with a fresh context).
        provide_toggle_group_context(&runtime, runtime.value.get_untracked());
        let re_rendered = toggle_element(ToggleProps {
            value: Some("one".to_string()),
            ..Default::default()
        })
        .expect("renders");
        let (updated, cleanup) = re_rendered.create_element();
        std::mem::forget(cleanup);
        assert_eq!(
            aria_pressed(&updated.dyn_into::<HtmlButtonElement>().unwrap()),
            "true",
            "a fresh evaluation at the new snapshot shows the item pressed"
        );
    }

    /// behavior.md "Events" cancellation (`:546-564`) through the group: a canceled
    /// change notifies but leaves both the value and the item's display untouched.
    #[wasm_bindgen_test]
    fn a_canceled_group_change_leaves_the_membership_untouched() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let recorder = Recorder::default();
        let recorder_for_callback = recorder.clone();
        let runtime = ToggleGroupRuntime::new(&ToggleGroupElementProps {
            default_value: Some(Vec::new()),
            on_value_change: Some(Arc::new(move |next: Vec<String>, details| {
                recorder_for_callback.record(next);
                details.cancel();
            })),
            ..Default::default()
        });
        let snapshot = runtime.value.get_untracked();
        provide_toggle_group_context(&runtime, snapshot);

        let rendered = toggle_element(ToggleProps {
            value: Some("one".to_string()),
            ..Default::default()
        })
        .expect("renders");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        let button = element.dyn_into::<HtmlButtonElement>().unwrap();

        click(&button);

        assert_eq!(
            recorder.proposed(),
            vec![vec!["one".to_string()]],
            "the vetoed change is still observed once"
        );
        assert!(
            runtime.value.get_untracked().is_empty(),
            "and it does not commit"
        );
        assert_eq!(
            aria_pressed(&button),
            "false",
            "the item's own commit is vetoed by the same shared details object"
        );
    }

    /// behavior.md "Focus management" (`:392-395`): the composite root gives the first
    /// item `tabindex="0"` and the rest `tabindex="-1"` — the roving-tabindex engine this
    /// unit's `CompositeRoot` wiring owns (`ToggleGroup.tsx:111-121`).
    #[wasm_bindgen_test]
    async fn the_composite_root_gives_the_first_item_the_tab_stop() {
        let mounted = mount_group_with_toggles(ToggleGroupElementProps::default(), &["one", "two"]);
        // The roving index is resolved by the composite list's registration effects, so
        // let one turn settle before reading the tab stops.
        flush_one_turn().await;
        assert_eq!(
            mounted.toggles[0].get_attribute("tabindex").as_deref(),
            Some("0"),
            "the first item is the tab stop"
        );
        assert_eq!(
            mounted.toggles[1].get_attribute("tabindex").as_deref(),
            Some("-1"),
            "the rest are roving"
        );
        assert_eq!(
            mounted.root.get_attribute("role").as_deref(),
            Some("group")
        );
    }

    /// The view path — `toggle_group_view`, the composition root the paired docs page
    /// will use: the root element, its attribute battery and the consumer's subtree all
    /// mount, and a composed child reads the provided context.
    #[wasm_bindgen_test]
    async fn the_view_path_mounts_the_root_and_its_subtree() {
        let _ = any_spawner::Executor::init_futures_executor();
        let container_el: web_sys::HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container_el).unwrap();
        let container: web_sys::Element = container_el.clone().unchecked_into();

        // `ChildrenFn` is `Arc<dyn Fn() -> AnyView + Send + Sync>`, so the subtree closure
        // is `Arc`-built (the checkbox-group suite's `ChildView` convention).
        let children: ChildrenFn = std::sync::Arc::new(move || {
            let rendered = toggle_element(ToggleProps {
                value: Some("one".to_string()),
                ..Default::default()
            })
            .expect("a grouped toggle renders through CompositeItem");
            let (element, cleanup) = rendered.create_element();
            std::mem::forget(cleanup);
            // The crate's only materialized-element-as-view bridge (the use-render page's
            // `RawElementView`, re-homed crate-side under the avatar unit).
            crate::avatar::AvatarDocView { element }.into_any()
        });

        std::mem::forget(mount_to(container_el, move || {
            toggle_group_view(ToggleGroupViewProps {
                element: ToggleGroupElementProps {
                    default_value: Some(vec!["one".to_string()]),
                    multiple: true,
                    ..Default::default()
                },
                children: Some(children),
            })
        }));

        // The attribute battery is written by a commit effect, so let one turn settle
        // (`otp_field_view_tests.rs` convention).
        flush_one_turn().await;

        let root = container
            .first_element_child()
            .expect("the group root mounted");
        assert_eq!(root.tag_name(), "DIV");
        assert_eq!(root.get_attribute("role").as_deref(), Some("group"));
        assert_eq!(
            root.get_attribute("data-orientation").as_deref(),
            Some("horizontal")
        );
        assert!(
            root.has_attribute("data-multiple"),
            "multiple=true is visible on the mounted root"
        );

        let button = root
            .query_selector("button")
            .unwrap()
            .expect("the subtree mounted inside the root");
        assert_eq!(
            button.get_attribute("aria-pressed").as_deref(),
            Some("true"),
            "the composed child reads the group's value snapshot"
        );
    }
}
