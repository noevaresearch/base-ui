//! Tests for the checkbox port — mirrors of
//! `packages/react/src/checkbox/root/CheckboxRoot.test.tsx`,
//! `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx` and
//! `packages/react/src/checkbox/indicator/CheckboxIndicator.test.tsx` (the suites
//! `specs/library/checkbox/behavior.md` mines).
//!
//! Host suite: the view-independent contracts — the snapshot record, the
//! `getCheckboxStateAttributesMapping` walk, the pure folds (`CheckboxRoot.tsx:93-150`,
//! `:201-207`, `:402-404`), the two-veto change funnel (`:211-246`) and the
//! missing-context contract.
//! Wasm suite: the materialized-element contracts — the three-sibling DOM, the
//! aria/data attribute matrix, the click-through-the-hidden-input toggle path, the
//! indeterminate re-assert, the readOnly/veto reversions, Enter-vs-Space, the
//! group-parent markers and the Indicator's mount machine.
//!
//! Harness law (the crate's wasm convention): a leptos `Effect` is *scheduled*, so
//! every assertion on effect-written output follows the async `settle()` — the
//! `form_tests`/`context_menu_tests` lesson (`context_menu_tests.rs:296-311`).

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use crate::checkbox::{
        MISSING_ROOT_CONTEXT_MESSAGE, try_use_checkbox_root_context, use_checkbox_root_context,
    };
    use crate::checkbox::state::{
        CheckboxRootState, DATA_CHECKED, DATA_DIRTY, DATA_DISABLED, DATA_FILLED, DATA_FOCUSED,
        DATA_INDETERMINATE, DATA_INVALID, DATA_READONLY, DATA_REQUIRED, DATA_TOUCHED,
        DATA_UNCHECKED, DATA_VALID, checked_is_dirty, checkbox_state_attributes, computed_checked,
        computed_indeterminate, controlled_checked, effective_disabled, effective_name,
        effective_value, group_override, indicator_rendered, indicator_should_render, input_id,
        input_name, input_style, input_value, root_id, should_render_unchecked_value_input,
        splice_group_value,
    };
    use serde_json::Value;

    fn owner_scope() -> reactive_graph::owner::Owner {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    /// The unchecked/enabled seed — tests name only the members they assert on.
    fn state() -> CheckboxRootState {
        CheckboxRootState {
            checked: false,
            disabled: false,
            read_only: false,
            required: false,
            indeterminate: false,
            touched: false,
            dirty: false,
            valid: None,
            filled: false,
            focused: false,
        }
    }

    // behavior.md "State model" (`CheckboxRoot.test.tsx:546-575`, `:495-504`): the
    // `checked` member becomes the mutually exclusive `data-checked`/`data-unchecked`
    // pair (the custom mapping, `getCheckboxStateAttributesMapping.ts:6-24`).
    #[test]
    fn the_state_walk_emits_the_exclusive_checked_pair() {
        let checked = checkbox_state_attributes(&CheckboxRootState {
            checked: true,
            ..state()
        });
        assert!(checked.contains_key(DATA_CHECKED));
        assert!(!checked.contains_key(DATA_UNCHECKED));

        let unchecked = checkbox_state_attributes(&state());
        assert!(unchecked.contains_key(DATA_UNCHECKED));
        assert!(!unchecked.contains_key(DATA_CHECKED));
    }

    // behavior.md "State model" (`:432-436`, `:495-504`): while `indeterminate` the
    // mapping consumes `checked` with NO attribute — neither `data-checked` nor
    // `data-unchecked` — and `data-indeterminate` comes from the generic arm.
    #[test]
    fn the_indeterminate_flag_suppresses_the_checked_pair() {
        let attrs = checkbox_state_attributes(&CheckboxRootState {
            checked: true,
            indeterminate: true,
            ..state()
        });
        assert!(!attrs.contains_key(DATA_CHECKED));
        assert!(!attrs.contains_key(DATA_UNCHECKED));
        assert!(attrs.contains_key(DATA_INDETERMINATE));
    }

    // behavior.md "Accessibility" (`:57`): `aria-checked` is `'mixed'` while
    // indeterminate, otherwise the boolean as a string.
    #[test]
    fn aria_checked_prefers_the_indeterminate_flag() {
        assert_eq!(
            CheckboxRootState {
                checked: true,
                indeterminate: true,
                ..state()
            }
            .aria_checked(),
            "mixed"
        );
        assert_eq!(
            CheckboxRootState {
                checked: true,
                ..state()
            }
            .aria_checked(),
            "true"
        );
        assert_eq!(state().aria_checked(), "false");
    }

    // behavior.md "State model" (`:546-575`): `data-disabled`/`data-readonly`/
    // `data-required` are emitted only while their member is true (the generic
    // truthiness arm), and the lifecycle hooks (`data-touched`/`data-dirty`/
    // `data-filled`/`data-focused`) ride the same walk (`:1315-1456`).
    #[test]
    fn the_state_walk_carries_the_boolean_hooks() {
        let attrs = checkbox_state_attributes(&CheckboxRootState {
            disabled: true,
            read_only: true,
            required: true,
            touched: true,
            dirty: true,
            filled: true,
            focused: true,
            ..state()
        });
        for name in [
            DATA_DISABLED,
            DATA_READONLY,
            DATA_REQUIRED,
            DATA_TOUCHED,
            DATA_DIRTY,
            DATA_FILLED,
            DATA_FOCUSED,
        ] {
            assert!(attrs.contains_key(name), "{name} is emitted");
        }

        let bare = checkbox_state_attributes(&state());
        for name in [
            DATA_DISABLED,
            DATA_READONLY,
            DATA_REQUIRED,
            DATA_TOUCHED,
            DATA_DIRTY,
            DATA_FILLED,
            DATA_FOCUSED,
        ] {
            assert!(!bare.contains_key(name), "{name} is omitted when false");
        }
    }

    // behavior.md "State model" (`:1472-1503`): the Field validity state surfaces as
    // `data-valid`/`data-invalid` through `fieldValidityMapping`; `null` (unvalidated)
    // emits neither (`state_attributes.rs:188-204`).
    #[test]
    fn the_state_walk_folds_the_field_validity_state() {
        let valid = checkbox_state_attributes(&CheckboxRootState {
            valid: Some(true),
            ..state()
        });
        assert!(valid.contains_key(DATA_VALID));
        assert!(!valid.contains_key(DATA_INVALID));

        let invalid = checkbox_state_attributes(&CheckboxRootState {
            valid: Some(false),
            ..state()
        });
        assert!(invalid.contains_key(DATA_INVALID));
        assert!(!invalid.contains_key(DATA_VALID));

        let unvalidated = checkbox_state_attributes(&state());
        assert!(!unvalidated.contains_key(DATA_VALID));
        assert!(!unvalidated.contains_key(DATA_INVALID));
    }

    // behavior.md "Accessibility"/"State model" (`:1315-1328` in a Field; the fold at
    // `CheckboxRoot.tsx:93-94`): `disabled` is the OR of the four sources.
    #[test]
    fn the_four_disabled_sources_are_ored() {
        assert!(!effective_disabled(false, false, false, false));
        assert!(effective_disabled(true, false, false, false));
        assert!(effective_disabled(false, true, false, false));
        assert!(effective_disabled(false, false, true, false));
        assert!(effective_disabled(false, false, false, true));
    }

    // implementation.md "State derivation order" (`CheckboxRoot.tsx:95-96`): the
    // Field's `name` wins over the prop; `value` falls back to the resolved `name`.
    #[test]
    fn the_field_name_wins_and_the_value_falls_back_to_it() {
        assert_eq!(
            effective_name(Some("field".into()), Some("prop".into())),
            Some("field".into())
        );
        assert_eq!(
            effective_name(None, Some("prop".into())),
            Some("prop".into())
        );
        assert_eq!(
            effective_value(None, Some("resolved".into())),
            Some("resolved".into())
        );
        assert_eq!(
            effective_value(Some("explicit".into()), Some("resolved".into())),
            Some("explicit".into())
        );
        assert_eq!(effective_value(None, None), None);
    }

    // implementation.md "State derivation order" (`CheckboxRoot.tsx:147-150`): the
    // group-parent pair folds the group's state in, and only then.
    #[test]
    fn computed_checked_and_indeterminate_fold_the_group_state() {
        assert!(computed_checked(true, true, false));
        assert!(!computed_checked(true, false, true));
        assert!(computed_checked(false, false, true));

        assert!(computed_indeterminate(true, true, false));
        assert!(computed_indeterminate(true, false, true));
        assert!(!computed_indeterminate(true, false, false));
        assert!(computed_indeterminate(false, true, false) == false);
        assert!(computed_indeterminate(false, false, true));

        // `groupProps.checked ?? checkedProp` (`:119-124`).
        assert!(group_override(Some(true), false));
        assert!(!group_override(Some(false), true));
        assert!(group_override(None, true));
    }

    // behavior.md "State model" (group membership, `:506-529`): a checkbox carrying a
    // `value` inside a group is controlled by membership; the group parent and the
    // group-derived `checked` are the two other arms (`CheckboxRoot.tsx:137-141`).
    #[test]
    fn controlled_checked_follows_group_membership_unless_parent() {
        let group = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            controlled_checked(Some("b"), Some(&group), false, None),
            Some(true)
        );
        assert_eq!(
            controlled_checked(Some("z"), Some(&group), false, None),
            Some(false)
        );
        // The parent arm ignores membership and takes the group-derived value.
        assert_eq!(
            controlled_checked(Some("a"), Some(&group), true, Some(false)),
            Some(false)
        );
        // Outside a group the group-derived value is the only controlled arm.
        assert_eq!(controlled_checked(None, None, false, Some(true)), Some(true));
        assert_eq!(controlled_checked(None, None, false, None), None);
    }

    // behavior.md "Events" (`:1176-1246`, `:1228-1246`): the `uncheckedValue` input's
    // condition — checked suppresses it, a group defers to the group's own
    // submission, a nameless or parent checkbox submits nothing, and `disabled`
    // withholds it (`CheckboxRoot.tsx:402-404`).
    #[test]
    fn the_unchecked_value_input_gate_matches_the_submission_rules() {
        assert!(should_render_unchecked_value_input(
            false,
            false,
            Some("field"),
            false,
            Some("off")
        ));
        assert!(!should_render_unchecked_value_input(
            true,
            false,
            Some("field"),
            false,
            Some("off")
        ));
        assert!(!should_render_unchecked_value_input(
            false,
            true,
            Some("field"),
            false,
            Some("off")
        ));
        assert!(!should_render_unchecked_value_input(
            false,
            false,
            None,
            false,
            Some("off")
        ));
        assert!(!should_render_unchecked_value_input(
            false,
            false,
            Some("field"),
            true,
            Some("off")
        ));
        assert!(!should_render_unchecked_value_input(
            false,
            false,
            Some("field"),
            false,
            None
        ));
    }

    // implementation.md "DOM/portal strategy" (`:64`): the two id lanes —
    // `rootId = nativeButton ? controlId : id`, and the hidden input owns the
    // labelable id only in the default mode (`CheckboxRoot.tsx:108`, `:204`).
    #[test]
    fn the_two_id_lanes_follow_native_button() {
        assert_eq!(root_id(false, "control", "generated"), "generated");
        assert_eq!(root_id(true, "control", "generated"), "control");
        assert_eq!(input_id(false, "control"), Some("control".to_string()));
        assert_eq!(input_id(true, "control"), None);
    }

    // behavior.md "DOM structure" (`:72`): the `name` is placed only on the hidden
    // input, and a group parent never carries one (`CheckboxRoot.tsx:201`).
    #[test]
    fn the_group_parent_is_excluded_from_submission() {
        assert_eq!(input_name(false, Some("field")), Some("field".to_string()));
        assert_eq!(input_name(true, Some("field")), None);
        assert_eq!(input_name(false, None), None);
    }

    // behavior.md "Events" (`:1077-1174`): the input `value` — the `value` prop, or
    // inside a group the prop only while checked, with `|| ''` dropping the empty
    // string (the React <19 workaround, `CheckboxRoot.tsx:256-260`).
    #[test]
    fn the_hidden_input_value_follows_the_group_gate() {
        assert_eq!(
            input_value(false, false, Some("apple")),
            Some("apple".to_string())
        );
        assert_eq!(
            input_value(true, true, Some("apple")),
            Some("apple".to_string())
        );
        assert_eq!(input_value(true, false, Some("apple")), Some(String::new()));
        assert_eq!(input_value(false, false, None), None);
    }

    // implementation.md "Anything in source not explained by any test" (`:96`): the
    // hidden input's style switches on form participation.
    #[test]
    fn the_two_hidden_input_style_modes() {
        assert_eq!(
            input_style(Some("field")),
            leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN_INPUT
        );
        assert_eq!(
            input_style(None),
            leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN
        );
    }

    // behavior.md "Events" (`:213-238` under a group): the additive splice — push on
    // check (idempotent), filter out on uncheck (`CheckboxRoot.tsx:239-245`).
    #[test]
    fn the_group_value_splice_is_additive() {
        let group = vec!["a".to_string()];
        assert_eq!(
            splice_group_value(&group, "b", true),
            vec!["a".to_string(), "b".to_string()]
        );
        // Already-present values are not duplicated.
        assert_eq!(splice_group_value(&group, "a", true), group);
        assert_eq!(
            splice_group_value(&group, "a", false),
            Vec::<String>::new()
        );
    }

    // implementation.md "State machine" (`CheckboxRoot.tsx:184-193`): the dirty fold —
    // checked differs from the Field's initial value.
    #[test]
    fn checked_is_dirty_tracks_the_initial_value() {
        assert!(checked_is_dirty(true, &Value::Bool(false)));
        assert!(!checked_is_dirty(true, &Value::Bool(true)));
        assert!(!checked_is_dirty(false, &Value::Null));
    }

    // behavior.md "DOM structure" (`:74`, `:49-108`): the Indicator's two predicates —
    // `rendered = checked || indeterminate` and `shouldRender = keepMounted || mounted`.
    #[test]
    fn the_indicator_mount_predicates() {
        assert!(indicator_rendered(true, false));
        assert!(indicator_rendered(false, true));
        assert!(!indicator_rendered(false, false));

        assert!(indicator_should_render(true, false));
        assert!(indicator_should_render(false, true));
        assert!(!indicator_should_render(false, false));
    }

    // behavior.md "Edge cases" (`:99`): outside a Root there is no context — the
    // non-panicking probe is the host-testable half (a wasm panic is an uncatchable
    // trap, the meter/field precedent).
    #[test]
    fn try_use_checkbox_root_context_is_none_outside_a_root() {
        let _owner = owner_scope();
        assert!(try_use_checkbox_root_context().is_none());
    }

    // behavior.md "Edge cases" (`CheckboxIndicator.test.tsx:37-47`): the throwing
    // accessor carries the exact upstream message.
    #[test]
    #[should_panic(expected = "CheckboxRootContext is missing")]
    fn the_missing_root_context_is_the_upstream_error() {
        let _owner = owner_scope();
        let _ = use_checkbox_root_context();
        let _ = MISSING_ROOT_CONTEXT_MESSAGE;
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use leptos::mount::mount_to;
    use leptos::prelude::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{HtmlElement, HtmlInputElement};

    use crate::checkbox::indicator::{CheckboxIndicatorViewProps, checkbox_indicator_view};
    use crate::checkbox::root::{CheckboxRootViewProps, checkbox_root_view};
    use crate::checkbox::state::{
        ChangeFunnel, CheckboxChangeEventDetails, CheckboxGroupFacingDetails, DATA_CHECKED,
        DATA_INDETERMINATE, DATA_UNCHECKED, change_event_details, run_change_funnel,
    };

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Settles the mounted tree: drains the local executor, yields to leptos's own
    /// scheduler, then gives the browser a real turn (the `form_tests` two-step flush,
    /// which is what the Effect-driven writers need before an assertion can read them).
    async fn settle() {
        for _ in 0..32 {
            any_spawner::Executor::poll_local();
        }
        leptos::task::tick().await;
        for _ in 0..32 {
            any_spawner::Executor::poll_local();
        }
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                .unwrap();
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
        for _ in 0..32 {
            any_spawner::Executor::poll_local();
        }
    }

    /// One real animation frame per step, settling between them. The transition/animation
    /// completion path resolves on a *requested frame* (`useAnimationsFinished.ts:156`,
    /// the internals' `await_frame` idiom) — `settle()`'s timer turn does not guarantee
    /// that frame has run, so the exit-completion that unmounts an indicator can still be
    /// pending when a timer-only settle returns.
    async fn settle_frames(frames: u32) {
        for _ in 0..frames {
            let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(&resolve)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            settle().await;
        }
    }

    fn container() -> HtmlElement {
        let container: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        container
    }

    /// Mounts the Root view — the real composition path (props in, ported view out).
    /// The handle is FORGOTTEN: dropping it unmounts the view and cancels the reactive
    /// owner before the assertions run (the meter/field convention).
    fn mount_root(props: CheckboxRootViewProps) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = container();
        std::mem::forget(mount_to(container.clone(), move || checkbox_root_view(props)));
        container
    }

    /// Mounts a Root whose children are a real `Checkbox.Indicator`.
    fn mount_root_with_indicator(keep_mounted: bool) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = container();
        std::mem::forget(mount_to(container.clone(), move || {
            checkbox_root_view(CheckboxRootViewProps {
                children: Some(Box::new(move || {
                    checkbox_indicator_view(CheckboxIndicatorViewProps {
                        keep_mounted,
                        ..CheckboxIndicatorViewProps::default()
                    })
                    .into_any()
                })),
                ..CheckboxRootViewProps::default()
            })
        }));
        container
    }

    /// The visible control — the only element carrying an explicit `role="checkbox"`
    /// (the hidden input's role is implicit, `behavior.md:70`).
    fn control(container: &HtmlElement) -> HtmlElement {
        container
            .query_selector("[role=\"checkbox\"]")
            .unwrap()
            .expect("the visible control rendered")
            .dyn_into::<HtmlElement>()
            .unwrap()
    }

    fn hidden_input(container: &HtmlElement) -> HtmlInputElement {
        container
            .query_selector("input[type=\"checkbox\"]")
            .unwrap()
            .expect("the hidden input rendered")
            .dyn_into::<HtmlInputElement>()
            .unwrap()
    }

    fn aria_checked(control: &HtmlElement) -> Option<String> {
        control.get_attribute("aria-checked")
    }

    /// A real activating click on the control (the browser runs the element's
    /// activation behavior, so the port's re-dispatch onto the hidden input happens for
    /// real — `CheckboxRoot.tsx:365-378`).
    fn click(element: &HtmlElement) {
        element.click();
    }

    fn key_event(kind: &str, key: &str) -> web_sys::KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key(key);
        web_sys::KeyboardEvent::new_with_keyboard_event_init_dict(kind, &init).unwrap()
    }

    fn press(element: &HtmlElement, key: &str) {
        let _ = element.dispatch_event(
            &key_event("keydown", key)
                .dyn_ref::<web_sys::Event>()
                .unwrap()
                .clone(),
        );
        let _ = element.dispatch_event(
            &key_event("keyup", key)
                .dyn_ref::<web_sys::Event>()
                .unwrap()
                .clone(),
        );
    }

    // behavior.md "DOM structure & portal behavior" (`CheckboxRoot.test.tsx:213-223`,
    // `:577-585`): the root renders the visible control plus the sibling hidden
    // `<input type="checkbox">` (untabbable, `aria-hidden`, `tabindex=-1`), and the
    // `name` rides only the hidden input.
    #[wasm_bindgen_test]
    async fn the_root_renders_the_three_siblings_with_the_hidden_input_last() {
        let container = mount_root(CheckboxRootViewProps {
            name: Some("accept".to_string()),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        assert_eq!(
            control.tag_name(),
            "SPAN",
            "the default visible control is a span"
        );
        assert_eq!(control.get_attribute("name"), None, "no name on the control");

        let input = hidden_input(&container);
        assert_eq!(input.get_attribute("name").as_deref(), Some("accept"));
        assert_eq!(input.get_attribute("tabindex").as_deref(), Some("-1"));
        assert_eq!(input.get_attribute("aria-hidden").as_deref(), Some("true"));
        assert_eq!(input.get_attribute("type").as_deref(), Some("checkbox"));

        // The hidden input follows the visible control in document order (upstream's
        // role-index [0]/[1] ordering, `:213-223`).
        let position = control.compare_document_position(&input);
        assert!(
            position & web_sys::Node::DOCUMENT_POSITION_FOLLOWING != 0,
            "the hidden input follows the control"
        );

        // The unchecked seed: `aria-checked="false"`, `data-unchecked`, no `data-checked`.
        assert_eq!(aria_checked(&control).as_deref(), Some("false"));
        assert!(control.has_attribute(DATA_UNCHECKED));
        assert!(!control.has_attribute(DATA_CHECKED));
        assert!(!input.checked());
    }

    // behavior.md "Events" (`CheckboxRoot.test.tsx:213-238`, `:269-280`): one click
    // toggles false → true, reports the next value once, and updates both surfaces
    // (`aria-checked`, the `data-*` hook and the hidden input's property).
    #[wasm_bindgen_test]
    async fn a_click_commits_the_checked_state_and_reports_it_once() {
        let calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_cb = Rc::clone(&calls);
        let container = mount_root(CheckboxRootViewProps {
            on_checked_change: Some(Rc::new(move |checked: bool, _details| {
                calls_for_cb.borrow_mut().push(checked);
            })),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        click(&control);
        settle().await;

        assert_eq!(aria_checked(&control).as_deref(), Some("true"));
        assert!(control.has_attribute(DATA_CHECKED));
        assert!(!control.has_attribute(DATA_UNCHECKED));
        assert!(hidden_input(&container).checked());
        assert_eq!(&*calls.borrow(), &[true], "one call per user click");

        // A second click returns to the unchecked state.
        click(&control);
        settle().await;
        assert_eq!(aria_checked(&control).as_deref(), Some("false"));
        assert!(!hidden_input(&container).checked());
        assert_eq!(&*calls.borrow(), &[true, false]);
    }

    // behavior.md "State model" (`:438-493`, `:432-436`): `indeterminate` renders
    // `aria-checked="mixed"`, mirrors the property onto the hidden input, wins over
    // `checked`, and SURVIVES a click (the native toggle is reverted) while `checked`
    // still advances.
    #[wasm_bindgen_test]
    async fn indeterminate_renders_mixed_and_survives_a_click() {
        let container = mount_root(CheckboxRootViewProps {
            indeterminate: true,
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        assert_eq!(aria_checked(&control).as_deref(), Some("mixed"));
        assert!(control.has_attribute(DATA_INDETERMINATE));
        assert!(!control.has_attribute(DATA_CHECKED));
        let input = hidden_input(&container);
        assert!(input.indeterminate(), "the flag mirrors onto the input");

        click(&control);
        settle().await;
        assert_eq!(
            aria_checked(&control).as_deref(),
            Some("mixed"),
            "a click never consumes the flag"
        );
        let input = hidden_input(&container);
        assert!(input.checked(), "the native toggle still commits `checked`");
        assert!(
            input.indeterminate(),
            "the flag is re-asserted after the native click cleared it"
        );
    }

    // behavior.md "State model" (`:374-385`, `:1458-1470`): a disabled checkbox cannot
    // be toggled and reports the disabled posture (`aria-disabled`, `data-disabled`).
    #[wasm_bindgen_test]
    async fn a_disabled_checkbox_never_toggles() {
        let calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_cb = Rc::clone(&calls);
        let container = mount_root(CheckboxRootViewProps {
            disabled: true,
            on_checked_change: Some(Rc::new(move |checked: bool, _details| {
                calls_for_cb.borrow_mut().push(checked);
            })),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        assert_eq!(control.get_attribute("aria-disabled").as_deref(), Some("true"));
        assert!(control.has_attribute("data-disabled"));
        assert_eq!(hidden_input(&container).get_attribute("disabled").is_some(), true);

        click(&control);
        settle().await;
        assert_eq!(aria_checked(&control).as_deref(), Some("false"));
        assert!(!hidden_input(&container).checked());
        assert!(calls.borrow().is_empty(), "zero callbacks on a disabled click");
    }

    // behavior.md "State model" (`:388-397`) / "Events" (`:313-338`): `readOnly` marks
    // `aria-readonly` and the change path re-preventDefaults at the single funnel — both
    // surfaces stay put.
    #[wasm_bindgen_test]
    async fn read_only_keeps_both_surfaces_unchanged() {
        let calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_cb = Rc::clone(&calls);
        let container = mount_root(CheckboxRootViewProps {
            read_only: true,
            on_checked_change: Some(Rc::new(move |checked: bool, _details| {
                calls_for_cb.borrow_mut().push(checked);
            })),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        assert_eq!(
            control.get_attribute("aria-readonly").as_deref(),
            Some("true")
        );
        assert!(control.has_attribute("data-readonly"));

        click(&control);
        settle().await;
        assert_eq!(aria_checked(&control).as_deref(), Some("false"));
        assert!(
            !hidden_input(&container).checked(),
            "the controlled-input commit reverts the native toggle"
        );
        assert!(calls.borrow().is_empty(), "readOnly never reports a change");
    }

    // behavior.md "State model" (`:282-298`): a consumer `cancel()` vetoes the state
    // update — the callback ran (once) and neither surface moved (the funnel
    // re-asserts the `checked` property after the browser flipped it).
    #[wasm_bindgen_test]
    async fn a_cancelled_change_leaves_both_surfaces_unchanged() {
        let calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_cb = Rc::clone(&calls);
        let container = mount_root(CheckboxRootViewProps {
            on_checked_change: Some(Rc::new(move |checked: bool, details| {
                calls_for_cb.borrow_mut().push(checked);
                details.cancel();
            })),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        click(&control);
        settle().await;

        assert_eq!(aria_checked(&control).as_deref(), Some("false"));
        assert!(!control.has_attribute(DATA_CHECKED));
        assert!(!hidden_input(&container).checked());
        assert_eq!(&*calls.borrow(), &[true], "the veto happens after the callback");
    }

    // behavior.md "Keyboard interactions" (`:353-364`, `:1784-1785`): Enter never
    // toggles the checkbox — upstream cancels the NATIVE event (`CheckboxRoot.tsx:354`)
    // so native button activation cannot run, swallows the shared button layer's Enter
    // handling, and one microtask later clicks the form's default submitter unless a
    // consumer/ancestor called `preventDefault()` during propagation.
    //
    // NOTE on the layer: behavior.md also records that an ancestor observes
    // `event.defaultPrevented === false` while the event propagates (`:849-852`). That is
    // a property of React's SYNTHETIC event, which caches the native value when it is
    // created, so Root's later native `preventDefault()` cannot retroactively update it.
    // A native-DOM port has exactly one event object, so the ancestor here observes the
    // real cancelled flag — the conservative direction. Recorded (not silently diverged)
    // in ralph/logs/spec-discrepancies.md; the opt-out half of this contract is exercised
    // by the Form-facing Enter tests, which need a form to submit.
    #[wasm_bindgen_test]
    async fn enter_never_toggles() {
        let calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_cb = Rc::clone(&calls);
        let container = mount_root(CheckboxRootViewProps {
            on_checked_change: Some(Rc::new(move |checked: bool, _details| {
                calls_for_cb.borrow_mut().push(checked);
            })),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        control.focus().unwrap();
        let event = key_event("keydown", "Enter");
        let _ = control.dispatch_event(&event.dyn_ref::<web_sys::Event>().unwrap().clone());
        settle().await;

        assert_eq!(aria_checked(&control).as_deref(), Some("false"));
        assert!(calls.borrow().is_empty(), "Enter never reports a change");
        assert!(
            event.default_prevented(),
            "the native event is cancelled so native activation cannot toggle (`:354`)"
        );
    }

    // behavior.md "Keyboard interactions" (`:340-351`): Space toggles the checkbox.
    #[wasm_bindgen_test]
    async fn space_toggles_the_checkbox() {
        let container = mount_root(CheckboxRootViewProps::default());
        settle().await;

        let control = control(&container);
        control.focus().unwrap();
        press(&control, " ");
        settle().await;

        assert_eq!(aria_checked(&control).as_deref(), Some("true"));
        assert!(hidden_input(&container).checked());
    }

    // behavior.md "Accessibility" (`:62-86`, implementation.md "DOM/portal strategy"):
    // a consumer `id` is honored on the labelable control (the hidden input in the
    // default mode) while the visible element carries the internal instance id.
    #[wasm_bindgen_test]
    async fn the_consumer_id_lands_on_the_labelable_control() {
        let container = mount_root(CheckboxRootViewProps {
            id: Some("agree".to_string()),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let input = hidden_input(&container);
        assert_eq!(input.get_attribute("id").as_deref(), Some("agree"));
        assert_ne!(
            control(&container).get_attribute("id"),
            Some("agree".to_string()),
            "the visible element carries the internal id lane"
        );
    }

    // behavior.md "Accessibility" (`:24-31`, `:388-397`): the ARIA posture members are
    // emitted only while set.
    #[wasm_bindgen_test]
    async fn the_aria_posture_members_follow_their_props() {
        let container = mount_root(CheckboxRootViewProps {
            required: true,
            ..CheckboxRootViewProps::default()
        });
        settle().await;
        let control = control(&container);
        assert_eq!(control.get_attribute("aria-required").as_deref(), Some("true"));
        assert_eq!(control.get_attribute("aria-readonly"), None);
        assert_eq!(control.get_attribute("aria-disabled"), None);
    }

    // implementation.md "Context providers/consumers" + behavior.md "DOM structure"
    // (`:73`): the Indicator mounts only when the state warrants it, mirrors the
    // state's `data-*` hooks, and `keepMounted` pins it.
    #[wasm_bindgen_test]
    async fn the_indicator_mounts_only_when_checked_or_indeterminate() {
        let container = mount_root_with_indicator(false);
        settle().await;
        assert!(
            container.query_selector("span span").unwrap().is_none(),
            "an unchecked indicator is not mounted"
        );

        let control = control(&container);
        click(&control);
        settle().await;
        let indicator = container
            .query_selector("span span")
            .unwrap()
            .expect("the checked indicator mounts");
        assert!(indicator.has_attribute(DATA_CHECKED));

        // Unchecking removes it again (no exit animation is defined here): the exit
        // completion is frame-driven, so this waits real frames rather than a timer turn.
        click(&control);
        settle_frames(2).await;
        assert!(
            container.query_selector("span span").unwrap().is_none(),
            "unchecking unmounts the indicator"
        );
    }

    #[wasm_bindgen_test]
    async fn keep_mounted_pins_the_indicator_in_every_state() {
        let container = mount_root_with_indicator(true);
        settle().await;

        let indicator = container
            .query_selector("span span")
            .unwrap()
            .expect("keepMounted mounts the indicator while unchecked");
        assert!(indicator.has_attribute(DATA_UNCHECKED));

        let control = control(&container);
        click(&control);
        settle().await;
        assert!(
            container.query_selector("span span").unwrap().is_some(),
            "keepMounted keeps it mounted while checked"
        );
    }

    // implementation.md "Anything in source not explained by any test" (`:93`): a
    // parent checkbox carries the `data-parent` styling hook and is excluded from
    // submission (`:201`).
    #[wasm_bindgen_test]
    async fn the_group_parent_gets_the_parent_marker() {
        let container = mount_root(CheckboxRootViewProps {
            parent: true,
            name: Some("all".to_string()),
            ..CheckboxRootViewProps::default()
        });
        settle().await;

        let control = control(&container);
        assert!(control.has_attribute("data-parent"));
        assert_eq!(
            hidden_input(&container).get_attribute("name"),
            None,
            "a group parent is excluded from submission"
        );
    }

    // implementation.md "Anything in source not explained by any test" (`:97`): the
    // hidden input's style switches on form participation (the two recipes).
    #[wasm_bindgen_test]
    async fn the_hidden_input_is_visually_hidden_in_both_modes() {
        let named = mount_root(CheckboxRootViewProps {
            name: Some("accept".to_string()),
            ..CheckboxRootViewProps::default()
        });
        settle().await;
        let style = hidden_input(&named).get_attribute("style").unwrap_or_default();
        assert!(!style.is_empty(), "the form-participating input is hidden");

        let nameless = mount_root(CheckboxRootViewProps::default());
        settle().await;
        let style = hidden_input(&nameless)
            .get_attribute("style")
            .unwrap_or_default();
        assert!(!style.is_empty(), "the nameless input is hidden");
    }

    // implementation.md "State machine" (`CheckboxRoot.tsx:211-246`): the two-veto
    // funnel — the gate order, and that a cancelled `onCheckedChange` never reaches the
    // group's callback (upstream's two-consumer, two-veto-point chain).
    //
    // The native event payload is why this half lives in the wasm lane: the details
    // type carries a real `web_sys::Event` (`CheckboxRoot.tsx:223`).
    #[wasm_bindgen_test]
    async fn the_change_funnel_vetoes_in_upstream_order() {
        let checked_calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let group_calls: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));

        // One fresh pair per step: upstream mints the details object per native change
        // (`CheckboxRoot.tsx:223`, `createChangeEventDetails`), so a `cancel()` from an
        // earlier step must not leak into a later one. Reusing a single canceled `details`
        // (the pre-fix oracle) made the group-veto and commit steps mis-report as
        // `CanceledByCheckedChange`.
        let fresh_details = || {
            (
                change_event_details(),
                CheckboxGroupFacingDetails::new(
                    leptos_ui_internals::floating_ui::reasons::NONE,
                    (),
                    None,
                    (),
                ),
            )
        };

        // A default-prevented native event is ignored entirely.
        let (details, group_details) = fresh_details();
        let outcome = run_change_funnel(
            true,
            false,
            true,
            Some(&|checked, _| checked_calls.borrow_mut().push(checked)),
            Some(&|checked, _| group_calls.borrow_mut().push(checked)),
            &details,
            &group_details,
        );
        assert_eq!(outcome, ChangeFunnel::DefaultPrevented);
        assert!(checked_calls.borrow().is_empty());

        // `readOnly` re-prevents before either consumer runs.
        let (details, group_details) = fresh_details();
        let outcome = run_change_funnel(
            true,
            true,
            false,
            Some(&|checked, _| checked_calls.borrow_mut().push(checked)),
            Some(&|checked, _| group_calls.borrow_mut().push(checked)),
            &details,
            &group_details,
        );
        assert_eq!(outcome, ChangeFunnel::ReadOnly);
        assert!(checked_calls.borrow().is_empty());

        // The consumer's veto stops the group callback from running at all.
        let veto: Rc<dyn Fn(bool, &CheckboxChangeEventDetails)> = Rc::new(|checked, details| {
            details.cancel();
            checked_calls.borrow_mut().push(checked);
        });
        let (details, group_details) = fresh_details();
        let outcome = run_change_funnel(
            true,
            false,
            false,
            Some(veto.as_ref()),
            Some(&|checked, _| group_calls.borrow_mut().push(checked)),
            &details,
            &group_details,
        );
        assert_eq!(outcome, ChangeFunnel::CanceledByCheckedChange);
        assert_eq!(&*checked_calls.borrow(), &[true]);
        assert!(group_calls.borrow().is_empty(), "the group veto point is skipped");

        // The group's own veto leaves the local state uncommitted.
        let group_veto: Rc<dyn Fn(bool, &CheckboxGroupFacingDetails)> =
            Rc::new(|checked, details| {
                details.cancel();
                group_calls.borrow_mut().push(checked);
            });
        let (details, group_details) = fresh_details();
        let outcome = run_change_funnel(
            true,
            false,
            false,
            None,
            Some(group_veto.as_ref()),
            &details,
            &group_details,
        );
        assert_eq!(outcome, ChangeFunnel::CanceledByGroup);

        // With both consumers accepting, the funnel commits.
        let (details, group_details) = fresh_details();
        let outcome = run_change_funnel(
            true,
            false,
            false,
            None,
            None,
            &details,
            &group_details,
        );
        assert_eq!(outcome, ChangeFunnel::Commit);
    }
}
