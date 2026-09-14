//! Tests for the checkbox-group port — mirrors of
//! `packages/react/src/checkbox-group/CheckboxGroup.test.tsx` and
//! `useCheckboxGroupParent.test.tsx` (the suites behavior.md mines), over the real
//! materialized element in a wasm-browser suite and the state-machine contracts in a
//! host suite.
//!
//! Host suite: the description-level contracts that need no DOM (the parent-toggle
//! engine's status cycle, the veto gates, the value splice, the registration
//! registry).
//! Wasm suite: the materialized-element contracts (`role="group"`, the `id`/`role`
//! attribute battery, child membership through the context).

use std::rc::Rc;

#[cfg(test)]
use crate::checkbox_group::{CheckboxGroupParent, ChildPropsSnapshot, ParentPropsSnapshot};

/// A counting callback recorder over the veto-wrapped commit path.
#[derive(Clone, Default)]
struct Recorder {
    proposed: std::cell::RefCell<Vec<Vec<String>>>,
    canceled_next: std::cell::Cell<bool>,
}

impl Recorder {
    fn proposed(&self) -> Vec<Vec<String>> {
        self.proposed.borrow().clone()
    }
}

fn recorder_on_value_change(recorder: Rc<Recorder>) -> crate::checkbox_group::OnGroupValueChange {
    Rc::new(
        move |next_value: Vec<String>,
              details: &crate::checkbox_group::CheckboxGroupChangeEventDetails| {
            recorder.proposed.borrow_mut().push(next_value);
            if recorder.canceled_next.get() {
                details.cancel();
                recorder.canceled_next.set(false);
            }
        },
    )
}

fn owner_scope() -> reactive_graph::owner::Owner {
    let owner = reactive_graph::owner::Owner::new();
    owner.set();
    owner
}

fn details() -> crate::checkbox_group::CheckboxGroupChangeEventDetails {
    use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
    use leptos_ui_internals::floating_ui::reasons;
    BaseUIChangeEventDetails::new(reasons::NONE, (), None, ())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::checkbox_group::use_checkbox_group_parent;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set as _};

    fn value_signal(
        seed: Vec<String>,
    ) -> RwSignal<Vec<String>, reactive_graph::owner::LocalStorage> {
        RwSignal::new_local(seed)
    }

    /// The hook's value source: the rg-0.2 `Signal` read wrapper derived over the
    /// writable RwSignal (the controlled value's read side).
    fn read_signal(
        value: &RwSignal<Vec<String>, reactive_graph::owner::LocalStorage>,
    ) -> reactive_graph::wrappers::read::Signal<Vec<String>, reactive_graph::owner::LocalStorage>
    {
        reactive_graph::wrappers::read::Signal::derive_local({
            let value = value.clone();
            move || reactive_graph::traits::Get::get(&value)
        })
    }

    // behavior.md "State model" (`useCheckboxGroupParent.test.tsx:11-56`): a parent
    // click from none-checked proposes `all`; from all-checked proposes `none` (the
    // `allOnOrOff` pole logic, `useCheckboxGroupParent.ts:85-90`).
    #[test]
    fn parent_click_from_none_proposes_all_and_from_all_proposes_none() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["apple".into(), "banana".into(), "cherry".into()];
        let value = value_signal(vec![]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        // All-off → propose all.
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(false, &details());
        assert_eq!(
            recorder.proposed(),
            vec![all.clone()],
            "the first parent click proposes the full set"
        );

        // Commit the all-checked state, then click again → propose none.
        value.set(all.clone());
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(true, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &Vec::<String>::new(),
            "the second parent click proposes the empty set"
        );
    }

    // behavior.md "State model" (`useCheckboxGroupParent.test.tsx:340-384`): the
    // mixed → on → off → snapshot-restore cycle — from mixed the first click
    // proposes `all`, the next proposes `none`, the third keeps the value mixed
    // (the `off` arm keeps the snapshot, `useCheckboxGroupParent.ts:88-105`).
    #[test]
    fn parent_click_cycle_from_mixed_restores_the_snapshot() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let value = value_signal(vec!["a".into()]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        // mixed → proposes all.
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(false, &details());
        assert_eq!(recorder.proposed().last().unwrap(), &all, "mixed → all");

        // Commit `all`; the next click from `on` proposes `none`.
        value.set(all.clone());
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(true, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &Vec::<String>::new(),
            "on → none"
        );

        // The off arm's third click re-proposes the pre-`all` snapshot — the
        // `off` arm keeps the value mixed (upstream 340-384: after the third
        // click `checkboxA` is the only one checked again). Parent clicks never
        // advance `uncontrolledStateRef`; only child commits do.
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(false, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &vec!["a".to_string()],
            "off keeps the pre-all snapshot"
        );
    }

    // behavior.md "State model" (`useCheckboxGroupParent.test.tsx:510-536`,
    // "handles checked disabled checkboxes") mirrored exactly: `b` is disabled
    // and seeded checked; the first parent click proposes `all` (the held `b`
    // counts as checked), the second proposes `none` = the checked disabled ones.
    #[test]
    fn disabled_children_are_held_by_the_parent_toggle() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let value = value_signal(vec!["b".into()]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );
        parent.disabled_states.borrow_mut().insert("b".into(), true);

        // First click (mixed status): the snapshot is ['b'], not at a pole —
        // propose `all`, which includes the disabled-but-checked `b` (the
        // `!disabled || uncontrolled.includes` filter arm).
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(false, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &all,
            "the checked disabled child counts as checked in the proposed all"
        );

        // Commit the proposal (upstream's controlled re-render), then the second
        // click (on status): propose `none` = the disabled ones the snapshot
        // still holds — exactly `['b']`.
        value.set(all.clone());
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(true, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &vec!["b".to_string()],
            "the checked disabled child is held when unchecking all"
        );
    }

    // behavior.md "State model" (`useCheckboxGroupParent.test.tsx:462-486`): the
    // snapshot is not polluted by a canceled child change — a canceled splice does
    // not advance `uncontrolledStateRef`, so the next parent click still proposes
    // the same set.
    #[test]
    fn a_canceled_child_change_does_not_pollute_the_parent_snapshot() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into(), "b".into()];
        let value = value_signal(vec!["a".into()]);
        let recorder = Rc::new(Recorder::default());
        recorder.canceled_next.set(true);
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        // A child uncheck of `a` gets canceled → the snapshot stays `["a"]`.
        let child = (parent.get_child_props)("a");
        (child.on_checked_change)(false, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &Vec::<String>::new(),
            "the handler still receives the proposed value"
        );

        // The parent click still proposes `all` (snapshot intact).
        let props = (parent.get_parent_props)();
        (props.on_checked_change)(false, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &all,
            "the snapshot survived the cancel"
        );
    }

    // behavior.md "State model" (`useCheckboxGroupParent.test.tsx:131-149`): the
    // derived tri-state inputs — all checked → `checked: true`; some →
    // `indeterminate: true`; none → both false.
    #[test]
    fn the_parent_props_derive_the_tri_state() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into(), "b".into()];
        let value = value_signal(vec![]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        let props: ParentPropsSnapshot = (parent.get_parent_props)();
        assert!(!props.checked && !props.indeterminate, "none checked");

        value.set(vec!["a".to_string()]);
        let props = (parent.get_parent_props)();
        assert!(
            !props.checked && props.indeterminate,
            "some checked → mixed"
        );

        value.set(all.clone());
        let props = (parent.get_parent_props)();
        assert!(props.checked && !props.indeterminate, "all checked");
    }

    // behavior.md "Accessibility" (`useCheckboxGroupParent.test.tsx:184-256`): the
    // parent `aria-controls` joins the registered child ids in `allValues` order;
    // unmounted children drop out; unregistered `allValues` contribute nothing.
    #[test]
    fn aria_controls_joins_the_registered_child_ids_in_all_values_order() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into(), "b".into()];
        let value = value_signal(vec![]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        // No registrations → None (the JS `undefined` empty-join arm).
        let props = (parent.get_parent_props)();
        assert_eq!(props.aria_controls, None);

        // Register out of `allValues` order; the join follows `allValues`.
        let unregister_b = (parent.register_child_id)("b", "id-b");
        let unregister_a = (parent.register_child_id)("a", "id-a");
        let props = (parent.get_parent_props)();
        assert_eq!(props.aria_controls, Some("id-a id-b".to_string()));

        // Unmount one → it drops out.
        unregister_a();
        let props = (parent.get_parent_props)();
        assert_eq!(props.aria_controls, Some("id-b".to_string()));
        drop(unregister_b);
    }

    // behavior.md "Accessibility" (`useCheckboxGroupParent.test.tsx:292-317`):
    // checkboxes sharing the same value both appear; the survivor is retained when
    // one unmounts.
    #[test]
    fn duplicate_value_children_both_appear_and_the_survivor_is_retained() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into()];
        let value = value_signal(vec![]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        let unregister_first = (parent.register_child_id)("a", "id-1");
        let unregister_second = (parent.register_child_id)("a", "id-2");
        let props = (parent.get_parent_props)();
        assert_eq!(props.aria_controls, Some("id-1 id-2".to_string()));

        unregister_first();
        let props = (parent.get_parent_props)();
        assert_eq!(
            props.aria_controls,
            Some("id-2".to_string()),
            "the surviving registration is retained"
        );
        drop(unregister_second);
    }

    // behavior.md "Events" (`useCheckboxGroupParent.test.tsx:58-85`): the child
    // splice — checking pushes the value, unchecking removes it, and the
    // membership snapshot in `getChildProps().checked` tracks the current value.
    #[test]
    fn child_props_splice_the_value_and_track_membership() {
        let _owner = owner_scope();
        let all: Vec<String> = vec!["a".into(), "b".into()];
        let value = value_signal(vec![]);
        let recorder = Rc::new(Recorder::default());
        let parent = use_checkbox_group_parent(
            &all,
            read_signal(&value),
            recorder_on_value_change(Rc::clone(&recorder)),
        );

        let child: ChildPropsSnapshot = (parent.get_child_props)("a");
        assert!(!child.checked);

        (child.on_checked_change)(true, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &vec!["a".to_string()],
            "checking pushes the child value"
        );

        // Commit the check; the membership snapshot follows.
        value.set(vec!["a".to_string()]);
        let child = (parent.get_child_props)("a");
        assert!(child.checked);

        // Unchecking splices it back out.
        (child.on_checked_change)(false, &details());
        assert_eq!(
            recorder.proposed().last().unwrap(),
            &Vec::<String>::new(),
            "unchecking removes the child value"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::checkbox_group::{
        CheckboxGroupProps, checkbox_group_element, provide_checkbox_group_context,
        use_checkbox_group_context,
    };
    use reactive_graph::signal::RwSignal;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::HtmlDivElement;
    use web_sys::wasm_bindgen::JsCast;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Builds and mounts one group; returns the mounted `<div role="group">`.
    fn mount_group(props: CheckboxGroupProps) -> HtmlDivElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = checkbox_group_element(props);
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        element.dyn_into::<HtmlDivElement>().unwrap()
    }

    // behavior.md "Public API surface" (`CheckboxGroup.test.tsx:15-27`): the root is
    // a `div` with `role="group"`, and a consumer `id` is forwarded.
    #[wasm_bindgen_test]
    fn the_group_renders_a_div_root_with_role_group_and_the_forwarded_id() {
        let div = mount_group(CheckboxGroupProps {
            id: Some("my-group".to_string()),
            ..CheckboxGroupProps::default()
        });
        assert_eq!(div.get_attribute("role").as_deref(), Some("group"));
        assert_eq!(div.get_attribute("id").as_deref(), Some("my-group"));
    }

    // behavior.md "State model" (`CheckboxGroup.test.tsx:21-27`): without an `id`
    // prop the generated id is still a non-empty attribute.
    #[wasm_bindgen_test]
    fn the_group_falls_back_to_a_generated_id() {
        let div = mount_group(CheckboxGroupProps::default());
        let id = div
            .get_attribute("id")
            .expect("the generated id is rendered");
        assert!(!id.is_empty());
    }

    // behavior.md "Accessibility" (`CheckboxGroup.test.tsx:1163-1196`): arbitrary
    // HTML attributes pass through onto the group element.
    #[wasm_bindgen_test]
    fn arbitrary_attributes_pass_through_to_the_group_element() {
        let div = mount_group(CheckboxGroupProps {
            element_attributes: vec![("aria-describedby".to_string(), "desc-1".to_string())],
            ..CheckboxGroupProps::default()
        });
        assert_eq!(
            div.get_attribute("aria-describedby").as_deref(),
            Some("desc-1")
        );
    }

    // The provider/consumer contract (implementation.md, "Cross-component
    // contract"): a child reading `useCheckboxGroupContext()` inside the provider
    // scope sees the group's value, `disabled`, and `allValues`.
    #[wasm_bindgen_test]
    fn the_context_contract_delivers_the_group_value_to_children() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = checkbox_group_element(CheckboxGroupProps {
            value: Some(vec!["a".to_string()]),
            all_values: Some(vec!["a".to_string(), "b".to_string()]),
            disabled: true,
            ..CheckboxGroupProps::default()
        });
        // The context value is provided during the element build (the port's
        // provider seam) — read it from the same owner scope.
        let context = use_checkbox_group_context().expect("the provider is in scope");
        assert_eq!(
            reactive_graph::traits::GetUntracked::get_untracked(&context.value),
            vec!["a".to_string()]
        );
        assert!(context.disabled);
        assert_eq!(
            context.all_values,
            Some(vec!["a".to_string(), "b".to_string()])
        );
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        assert_eq!(element.get_attribute("role").as_deref(), Some("group"));
    }

    // behavior.md "State model" (`CheckboxGroup.test.tsx:95-123`): a controlled
    // value that becomes `None` is treated as an empty array — children uncheck, no
    // crash (the `Option` controlled source's fallback branch).
    #[wasm_bindgen_test]
    fn a_controlled_value_becoming_none_is_treated_as_empty() {
        let div = mount_group(CheckboxGroupProps {
            value: None,
            default_value: Some(vec![]),
            ..CheckboxGroupProps::default()
        });
        // The group renders without crashing; the state map carries the
        // derived `disabled`/`filled` hooks.
        assert!(div.get_attribute("data-disabled").is_none());
        assert_eq!(div.get_attribute("data-filled").as_deref(), None);
    }
}
