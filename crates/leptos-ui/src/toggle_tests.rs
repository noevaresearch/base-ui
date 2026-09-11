//! Tests for the Toggle port — mirrors of `packages/react/src/toggle/Toggle.test.tsx`
//! (the suite behavior.md mines), over the real materialized element in a
//! wasm-browser suite and the state-machine contracts in an owner-scoped host suite.
//!
//! Host suite: the description-level contracts that need no DOM (the `useControlled`
//! mode fix, the state map, the value normalization).
//! Wasm suite: the materialized-element contracts (`aria-pressed`, `data-pressed`/
//! `data-disabled`, `type="button"`, the click transition machine with the shared
//! details cancel protocol, disabled-click suppression).

use std::sync::Arc;
use std::sync::Mutex;

use super::*;
#[cfg(test)]
use crate::toggle::{ToggleGroupContext, ToggleProps, ToggleState, use_toggle_group_context};

/// The shared details type the callbacks and the group committer exchange
/// (`packages/react/src/toggle/Toggle.tsx:86`).
type Details = leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails<
    (),
    web_sys::MouseEvent,
>;

/// A counting callback recorder that satisfies the `Send + Sync` bounds of the
/// context/callback types (the wasm single thread makes the mutex uncontended).
#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<RecorderState>>);

#[derive(Default)]
struct RecorderState {
    calls: u32,
    pressed_values: Vec<bool>,
    commits: Vec<(String, bool, bool)>,
}

impl Recorder {
    fn record_call(&self, pressed: bool) {
        let mut state = self.0.lock().unwrap();
        state.calls += 1;
        state.pressed_values.push(pressed);
    }
    fn record_commit(&self, value: String, next_pressed: bool, canceled: bool) {
        self.0.lock().unwrap().commits.push((value, next_pressed, canceled));
    }
    fn calls(&self) -> u32 {
        self.0.lock().unwrap().calls
    }
    fn pressed_values(&self) -> Vec<bool> {
        self.0.lock().unwrap().pressed_values.clone()
    }
    fn commits(&self) -> Vec<(String, bool, bool)> {
        self.0.lock().unwrap().commits.clone()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use serde_json::json;

    use super::*;

    fn in_owner() -> reactive_graph::owner::Owner {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // Pins the value normalization (`packages/react/src/toggle/Toggle.tsx:43-44` —
    // `valueProp || undefined`): an empty string normalizes to the generated id, a
    // real value passes through.
    #[test]
    fn falsy_value_normalizes_to_none() {
        assert_eq!(crate::toggle::resolve_value(None), None);
        assert_eq!(
            crate::toggle::resolve_value(Some(String::new())),
            None
        );
        assert_eq!(
            crate::toggle::resolve_value(Some("one".to_string())),
            Some("one".to_string())
        );
    }

    // Pins the state map's shape (`:75-78`): the record the state mapping walks, with
    // the truthiness mapping emitting `data-pressed`/`data-disabled`
    // (ToggleDataAttributes.ts:4,8).
    #[test]
    fn the_state_map_carries_disabled_and_pressed() {
        let map = ToggleState {
            disabled: false,
            pressed: true,
        }
        .to_state_map();
        assert_eq!(map.get("pressed"), Some(&json!(true)));
        assert_eq!(map.get("disabled"), Some(&json!(false)));
    }

    // Pins the documented defaults (`:30,31,38`): uncontrolled, unpressed,
    // undisabled, native-button.
    #[test]
    fn default_props_match_upstream_defaults() {
        let props = ToggleProps::default();
        assert_eq!(props.pressed, None, "pressed is uncontrolled by default");
        assert!(!props.default_pressed, "defaultPressed defaults to false");
        assert!(!props.disabled, "disabled defaults to false");
        assert!(props.native_button, "nativeButton defaults to true");
        assert_eq!(props.value, None);
    }

    // Pins the group-context absence contract (`:45-46`): no provider in scope means
    // the standalone branch — `useToggleGroupContext()` returns undefined upstream.
    #[test]
    fn no_group_context_outside_a_provider() {
        let _owner = in_owner();
        assert!(use_toggle_group_context().is_none());
    }

    // Pins the grouped-branch presence contract: providing the context makes the
    // branch point fire (`packages/react/src/toggle-group/ToggleGroupContext.ts:21-23`
    // — the context's presence is the unit's single branch point).
    #[test]
    fn a_provided_group_context_is_visible_to_the_component() {
        use reactive_graph::owner::provide_context;

        let _owner = in_owner();
        provide_context(ToggleGroupContext {
            value: std::sync::Arc::new(vec!["one".to_string()]),
            set_group_value: None,
            disabled: false,
            is_value_initialized: true,
        });
        let context = use_toggle_group_context().expect("the context is provided");
        assert!(context.value.iter().any(|v| v == "one"));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::computed::Memo;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Event, HtmlButtonElement};

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn key_event(key: &str) -> web_sys::KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key(key);
        web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    /// Builds and mounts one Toggle; returns the mounted `<button>` element with the
    /// (forgotten) listener cleanup — the harness convention of the
    /// composite_view/use_button wasm suites.
    fn mount_toggle(props: ToggleProps) -> HtmlButtonElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = toggle_element(props).expect("the standalone toggle renders");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        element.dyn_into::<HtmlButtonElement>().unwrap()
    }

    fn click(element: &HtmlButtonElement) {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
            .unwrap();
        element
            .dispatch_event(&event.dyn_ref::<Event>().unwrap().clone())
            .unwrap();
    }

    fn aria_pressed(element: &HtmlButtonElement) -> String {
        element.get_attribute("aria-pressed").expect("aria-pressed present")
    }

    // behavior.md "Accessibility": `aria-pressed` mirrors the pressed state as the
    // string `'true'`/`'false'` (Toggle.test.tsx:49-53 — uncontrolled seed).
    #[wasm_bindgen_test]
    fn an_uncontrolled_toggle_starts_with_aria_pressed_false() {
        let button = mount_toggle(ToggleProps {
            default_pressed: false,
            ..ToggleProps::default()
        });
        assert_eq!(aria_pressed(&button), "false", "the seed is false");
    }

    // behavior.md "State model" transition (`Toggle.test.tsx:54-64`): each click
    // toggles — 'false' → 'true' → 'false' over two clicks.
    #[wasm_bindgen_test]
    fn each_click_toggles_the_pressed_state() {
        let button = mount_toggle(ToggleProps::default());
        assert_eq!(aria_pressed(&button), "false");
        click(&button);
        assert_eq!(aria_pressed(&button), "true", "the first click presses");
        click(&button);
        assert_eq!(aria_pressed(&button), "false", "the second click unpresses");
    }

    // behavior.md "State model" controlled (`Toggle.test.tsx:34-45`): the controlled
    // prop fully determines the rendered state — the machine's controlled setter is a
    // no-op (packages/utils/src/useControlled.ts:84-86), so a click does not change
    // the rendered attribute (the external prop flip is the only state source).
    #[wasm_bindgen_test]
    fn a_controlled_toggle_does_not_commit_on_click() {
        let button = mount_toggle(ToggleProps {
            pressed: Some(false),
            ..ToggleProps::default()
        });
        assert_eq!(aria_pressed(&button), "false");
        click(&button);
        assert_eq!(
            aria_pressed(&button),
            "false",
            "the controlled setter is a no-op (useControlled.ts:84-86)"
        );
    }

    // behavior.md "Events" (`Toggle.test.tsx:80-81`): `onPressedChange` fires once per
    // activating click with the *next* pressed value.
    #[wasm_bindgen_test]
    fn on_pressed_change_fires_with_the_next_pressed_value() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_toggle(ToggleProps {
            on_pressed_change: Some(Arc::new(move |pressed: bool, _details: &Details| {
                recorder_for_cb.record_call(pressed);
            })),
            ..ToggleProps::default()
        });
        click(&button);
        click(&button);
        assert_eq!(recorder.calls(), 2, "one call per activating click");
        assert_eq!(
            recorder.pressed_values(),
            vec![true, false],
            "the first argument is the next pressed value"
        );
    }

    // behavior.md "State model" cancellation (`Toggle.test.tsx:88-100`): calling
    // `eventDetails.cancel()` aborts the transition — `aria-pressed` stays `'false'`.
    #[wasm_bindgen_test]
    fn canceling_in_on_pressed_change_suppresses_the_state_change() {
        let button = mount_toggle(ToggleProps {
            on_pressed_change: Some(Arc::new(|_pressed: bool, details: &Details| {
                details.cancel();
            })),
            ..ToggleProps::default()
        });
        click(&button);
        assert_eq!(
            aria_pressed(&button),
            "false",
            "the canceled transition does not commit"
        );
    }

    // behavior.md "State model" grouped behavior (`Toggle.test.tsx:103-126`): under a
    // provided ToggleGroup context whose value contains the Toggle's `value`, the
    // group's `setGroupValue` commit fires with `(value, nextPressed, details)` on
    // click, and a Toggle-side `cancel()` vetoes it — the shared details object is
    // the contract (Toggle.tsx:88-97). The grouped render path goes through
    // `CompositeItem`, which requires a `CompositeRoot` in scope (ToggleGroup
    // provides one, `ToggleGroup.tsx:8,111`).
    #[wasm_bindgen_test]
    fn the_group_commit_and_the_shared_details_veto() {
        use reactive_graph::owner::provide_context;

        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        // The CompositeRoot context the grouped `CompositeItem` path requires
        // (ToggleGroup provides one via `CompositeRoot`); built with the harness
        // defaults (composite_view.rs).
        let any_index: Memo<i32> = Memo::new(|_| -1);
        leptos_ui_internals::composite_root_context::provide_composite_root_context(
            leptos_ui_internals::composite_root_context::CompositeRootContextValue {
                highlighted_index: any_index,
                on_highlighted_index_change: std::rc::Rc::new(|_index: i32, _scroll: bool| {}),
                highlight_item_on_hover: false,
                relay_keyboard_event: std::rc::Rc::new(
                    |_event: &web_sys::KeyboardEvent| {},
                ),
            },
        );

        let recorder = Recorder::default();
        let recorder_for_commit = recorder.clone();

        // The group's gated committer: records `(value, nextPressed,
        // details.isCanceled)` — the ToggleGroup side of the shared-details contract.
        provide_context(ToggleGroupContext {
            value: Arc::new(vec!["one".to_string()]),
            set_group_value: Some(Arc::new(
                move |value: &str, next_pressed: bool, details: &Details| {
                    recorder_for_commit.record_commit(
                        value.to_string(),
                        next_pressed,
                        details.is_canceled(),
                    );
                },
            )),
            disabled: false,
            is_value_initialized: true,
        });

        // The user callback cancels on the second click — vetoing both the group
        // commit and the local state change (Toggle.test.tsx:103-124). The counter
        // is `Arc<AtomicU32>`: the callback type's `Send + Sync` bound rejects the
        // `Rc<Cell<u32>>` this test used before it was first compiled for wasm32.
        let clicks = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let clicks_for_cb = std::sync::Arc::clone(&clicks);
        let rendered = toggle_element(ToggleProps {
            value: Some("one".to_string()),
            on_pressed_change: Some(Arc::new(move |_pressed: bool, details: &Details| {
                if clicks_for_cb.load(std::sync::atomic::Ordering::SeqCst) >= 1 {
                    details.cancel();
                }
            })),
            ..ToggleProps::default()
        })
        .expect("the grouped toggle renders via CompositeItem");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        let button = element.dyn_into::<HtmlButtonElement>().unwrap();

        click(&button);
        assert_eq!(
            recorder.commits(),
            vec![("one".to_string(), true, false)],
            "the first click commits (value, nextPressed, not-canceled)"
        );

        click(&button);
        assert_eq!(
            recorder.commits().len(),
            1,
            "the canceled second click never reaches the group commit"
        );
        assert_eq!(
            aria_pressed(&button),
            "false",
            "the canceled click does not commit the local state either"
        );
        // The owner keeps the contexts alive for the test's lifetime.
        std::mem::forget(owner);
    }

    // behavior.md "Edge cases" disabled interaction (`Toggle.test.tsx:140-145`): a
    // click on a disabled Toggle invokes `onPressedChange` zero times and leaves
    // `aria-pressed` at `'false'` — the native `disabled` attribute blocks the DOM
    // click, and useButton's guard preventDefaults (useButton.ts:104-110).
    #[wasm_bindgen_test]
    fn a_disabled_toggle_never_invokes_on_pressed_change() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_toggle(ToggleProps {
            disabled: true,
            on_pressed_change: Some(Arc::new(move |_pressed: bool, _details: &Details| {
                recorder_for_cb.record_call(_pressed);
            })),
            ..ToggleProps::default()
        });
        click(&button);
        assert_eq!(recorder.calls(), 0, "zero callbacks on a disabled click");
        assert_eq!(aria_pressed(&button), "false", "the state did not change");
    }

    // behavior.md "Accessibility" (`Toggle.test.tsx:136-138`): the disabled button
    // carries the native `disabled` attribute and `data-disabled`, and reports
    // `aria-pressed="false"`.
    #[wasm_bindgen_test]
    fn a_disabled_toggle_carries_the_disabled_attributes() {
        let button = mount_toggle(ToggleProps {
            disabled: true,
            ..ToggleProps::default()
        });
        assert!(button.disabled(), "the native disabled attribute is set");
        assert!(
            button.has_attribute("data-disabled"),
            "data-disabled is emitted by the state mapping"
        );
        assert_eq!(aria_pressed(&button), "false");
    }

    // behavior.md "DOM structure" (`Toggle.test.tsx:12` + conformance): the rendered
    // output is a single native `<button>` with `type="button"` (`useButton.ts:226`
    // + `useRenderElement.tsx:232-235`).
    #[wasm_bindgen_test]
    fn the_toggle_renders_a_native_button_with_type_button() {
        let button = mount_toggle(ToggleProps::default());
        assert_eq!(button.tag_name(), "BUTTON");
        assert_eq!(
            button.get_attribute("type").as_deref(),
            Some("button"),
            "the native-button attribute split emits type=button"
        );
    }

    // The state mapping emits `data-pressed` when pressed (ToggleDataAttributes.ts:4
    // — declared and produced, asserted by no upstream test; implementation.md's
    // untested-behavior item 1 — pinned here).
    #[wasm_bindgen_test]
    fn the_state_mapping_emits_data_pressed_when_pressed() {
        let button = mount_toggle(ToggleProps::default());
        assert!(
            !button.has_attribute("data-pressed"),
            "unpressed: the attribute is omitted"
        );
        click(&button);
        assert!(
            button.has_attribute("data-pressed"),
            "pressed: the truthy state value emits the bare data attribute"
        );
    }

    // The controlled flip (`Toggle.test.tsx:34-45`) at the description level: a
    // fresh evaluation with the controlled prop flipped carries the new state
    // (React re-renders; the port re-derives the state map per body run).
    #[wasm_bindgen_test]
    fn a_fresh_evaluation_reflects_a_controlled_flip() {
        let rendered = toggle_element(ToggleProps {
            pressed: Some(true),
            ..ToggleProps::default()
        })
        .expect("renders");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        let button = element.dyn_into::<HtmlButtonElement>().unwrap();
        assert_eq!(
            button.get_attribute("aria-pressed").as_deref(),
            Some("true"),
            "the controlled prop determines the rendered state"
        );
    }

    // The keyboard path: behavior.md "Keyboard interactions" is N/A upstream — the
    // native default is the contract. Pin that the internal pipeline does not
    // synthesize activation for a native standalone button (useButton.ts:154-163's
    // `should_click || is_native_button` early return).
    #[wasm_bindgen_test]
    fn enter_key_does_not_synthetically_activate_a_native_standalone_button() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_toggle(ToggleProps {
            handlers: ToggleHandlers {
                on_click: Some(std::rc::Rc::new(
                    move |_event: &leptos_ui_internals::types::BaseUIEvent<web_sys::MouseEvent>| {
                        recorder_for_cb.record_call(true);
                    },
                )),
            },
            ..ToggleProps::default()
        });
        button
            .dispatch_event(&key_event("Enter").dyn_ref::<Event>().unwrap().clone())
            .unwrap();
        assert_eq!(
            recorder.calls(),
            0,
            "no synthetic activation for a native standalone button"
        );
    }
}
