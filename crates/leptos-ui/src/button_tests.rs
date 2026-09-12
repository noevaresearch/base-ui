//! Tests for the Button port — mirrors of `packages/react/src/button/Button.test.tsx`
//! (the suite behavior.md mines), over the real materialized element in a
//! wasm-browser suite and the description-level contracts in an owner-scoped host
//! suite.
//!
//! Host suite: the description-level contracts that need no DOM (the props bag
//! order, the state map, the defaults).
//! Wasm suite: the materialized-element contracts (the native `<button>` root with
//! `type="button"`, `data-disabled`, the aria/tabindex matrix, disabled click
//! suppression, keyboard activation on the non-native path, attribute override,
//! render-prop tag preservation).
//!
//! The full disabled/focusability attribute policy and the keyboard-activation
//! engines are `use_button`'s own suite-tested contracts
//! (crates/leptos-ui-internals/src/use_button.rs, 23 wasm tests mirroring the
//! upstream useButton.test.tsx matrix); this facade's suite pins that the
//! component wires them through with the documented props and bag order.

// The wasm-only harness items (Recorder, mount_button) are dead code on the host
// target — the dual-target test-module convention (the toggle/dialog precedent,
// whose host suites import only the description-level surface).
#![allow(unused_imports, dead_code)]

use super::*;
#[allow(unused_imports)]
use crate::button::{button_element, ButtonHandlers, ButtonProps, ButtonState};

/// A counting callback recorder (the wasm single thread makes the mutex
/// uncontended — the toggle_tests.rs convention).
#[derive(Clone, Default)]
struct Recorder(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl Recorder {
    fn record(&self, name: &str) {
        self.0.lock().unwrap().push(name.to_string());
    }
    fn calls(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // implementation.md:104-110 (`ButtonState` exists solely to drive
    // `data-disabled` via the generic truthiness mapping).
    #[test]
    fn the_state_map_carries_disabled_for_data_disabled() {
        let enabled = ButtonState { disabled: false }.to_state_map();
        let disabled = ButtonState { disabled: true }.to_state_map();
        assert_eq!(enabled.get("disabled"), Some(&serde_json::json!(false)));
        assert_eq!(disabled.get("disabled"), Some(&serde_json::json!(true)));
    }

    // Upstream defaults (`Button.tsx:18-21`): `disabled = false`,
    // `focusableWhenDisabled = false`, `nativeButton = true`.
    #[test]
    fn props_default_to_native_enabled_button() {
        let props = ButtonProps::default();
        assert!(!props.disabled);
        assert!(!props.focusable_when_disabled);
        assert!(props.native_button);
        assert!(props.element_attributes.is_empty());
        let handlers = ButtonHandlers::default();
        assert!(handlers.on_click.is_none());
        assert!(handlers.on_key_down.is_none());
    }

    // The facade always renders — upstream's `useRenderElement` call has no
    // `enabled` gate (a leaf with no conditional rendering).
    #[test]
    fn the_description_is_always_produced() {
        let _owner = in_owner();
        let rendered = button_element(ButtonProps::default());
        assert!(rendered.is_some());
        let rendered = rendered.unwrap();
        assert_eq!(rendered.tag, "button");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Event, HtmlButtonElement, HtmlElement};

    use super::*;
    use std::rc::Rc;
    use leptos_ui_internals::use_render_element::UseRenderElementComponentProps;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn key_event(event_type: &str, key: &str) -> web_sys::KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        // Real keyboard events are cancelable — without this, `preventDefault` in
        // the handlers is a no-op and the defaultPrevented gates can never trigger
        // (the use_button harness convention).
        init.set_cancelable(true);
        init.set_key(key);
        web_sys::KeyboardEvent::new_with_keyboard_event_init_dict(event_type, &init).unwrap()
    }

    fn mouse_event(event_type: &str) -> web_sys::MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        web_sys::MouseEvent::new_with_mouse_event_init_dict(event_type, &init).unwrap()
    }

    /// Builds and mounts one Button; returns the mounted root element with the
    /// (forgotten) listener cleanup — the harness convention of the
    /// toggle/dialog wasm suites.
    fn mount_button(props: ButtonProps) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = button_element(props).expect("the button renders");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        element.dyn_into::<HtmlElement>().unwrap()
    }

    fn click(element: &HtmlElement) {
        element
            .dispatch_event(&mouse_event("click").dyn_ref::<Event>().unwrap().clone())
            .unwrap();
    }

    fn fire_key(element: &HtmlElement, event_type: &str, key: &str) -> bool {
        element
            .dispatch_event(
                &key_event(event_type, key)
                    .dyn_ref::<Event>()
                    .unwrap()
                    .clone(),
            )
            .unwrap()
    }

    // behavior.md "DOM structure & portal behavior" +
    // "Accessibility" (`Button.test.tsx:12,60-64`): the default root is a native
    // `<button>` (conformance `refInstanceof: window.HTMLButtonElement`), and the
    // render engine forces `type="button"` (`useRenderElement.tsx:232-240`).
    #[wasm_bindgen_test]
    fn the_default_root_is_a_native_button_with_type_button() {
        let button = mount_button(ButtonProps::default());
        assert_eq!(button.tag_name(), "BUTTON");
        let native = button.dyn_ref::<HtmlButtonElement>().unwrap();
        assert_eq!(
            native.type_(),
            "button",
            "the form-submit default is overridden"
        );
    }

    // behavior.md "Accessibility" (`Button.test.tsx:118-120`): a native disabled
    // button gets the `disabled` attribute + `data-disabled`, and explicitly does
    // NOT get `aria-disabled` (useFocusableWhenDisabled.ts:38-47).
    #[wasm_bindgen_test]
    fn a_native_disabled_button_gets_disabled_and_data_disabled_but_no_aria_disabled() {
        let button = mount_button(ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        });
        let native = button.dyn_ref::<HtmlButtonElement>().unwrap();
        assert!(native.disabled(), "the native disabled attribute is set");
        assert_eq!(
            button.get_attribute("data-disabled").as_deref(),
            Some(""),
            "data-disabled present (the generic truthiness mapping)"
        );
        assert_eq!(
            button.get_attribute("aria-disabled"),
            None,
            "no aria-disabled on the native path"
        );
    }

    // behavior.md "Accessibility" (`Button.test.tsx:60-64,155-157`): a non-native
    // disabled button gets `aria-disabled="true"` + `data-disabled`, and NO
    // `disabled` attribute.
    #[wasm_bindgen_test]
    fn a_non_native_disabled_button_gets_aria_disabled_and_no_disabled_attribute() {
        let button = mount_button(ButtonProps {
            native_button: false,
            disabled: true,
            ..ButtonProps::default()
        });
        assert_eq!(button.tag_name(), "BUTTON");
        assert_eq!(
            button.get_attribute("aria-disabled").as_deref(),
            Some("true"),
            "aria-disabled carries the disabled signal"
        );
        assert_eq!(button.get_attribute("data-disabled").as_deref(), Some(""));
        assert!(
            button
                .dyn_ref::<HtmlButtonElement>()
                .map(|b| !b.disabled())
                .unwrap_or(true),
            "no native disabled attribute (useFocusableWhenDisabled.ts:45-47)"
        );
    }

    // behavior.md "Focus management" (`Button.test.tsx:158-161,197-200,306-311`):
    // non-native disabled → `tabindex="-1"` (skipped by Tab); disabled +
    // `focusableWhenDisabled` → `tabindex="0"` (reachable, `aria-disabled`
    // still carries the signal).
    #[wasm_bindgen_test]
    fn the_tabindex_matrix_matches_the_disabled_focusability_policy() {
        let skipped = mount_button(ButtonProps {
            native_button: false,
            disabled: true,
            ..ButtonProps::default()
        });
        assert_eq!(
            skipped.get_attribute("tabindex").as_deref(),
            Some("-1"),
            "non-native disabled: tabindex -1"
        );

        let focusable = mount_button(ButtonProps {
            native_button: false,
            disabled: true,
            focusable_when_disabled: true,
            ..ButtonProps::default()
        });
        assert_eq!(
            focusable.get_attribute("tabindex").as_deref(),
            Some("0"),
            "focusableWhenDisabled: tabindex 0"
        );
        assert_eq!(
            focusable.get_attribute("aria-disabled").as_deref(),
            Some("true"),
            "aria-disabled still signals disabled while focusable"
        );
        assert_eq!(
            focusable.get_attribute("disabled"),
            None,
            "the native disabled attribute is omitted (it would block focus)"
        );
    }

    // behavior.md "Events" (`Button.test.tsx:129-132,206-209`): disabled
    // suppresses `click` (0 handler invocations) — the internal disabled guard in
    // `getButtonProps` runs before the consumer's handler (the later-bag-runs-first
    // rule, mergeProps.ts:229-244).
    #[wasm_bindgen_test]
    fn a_disabled_button_suppresses_click_handlers() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_button(ButtonProps {
            disabled: true,
            handlers: ButtonHandlers {
                on_click: Some(Rc::new(move |_event: &web_sys::MouseEvent| {
                    recorder_for_cb.record("click");
                })),
                ..ButtonHandlers::default()
            },
            ..ButtonProps::default()
        });
        click(&button);
        assert!(
            recorder.calls().is_empty(),
            "the disabled guard swallows the external handler (useButton.ts:104-110)"
        );
    }

    // behavior.md "Events" (`Button.test.tsx:107-113`): an enabled button fires
    // the consumer's `onClick`.
    #[wasm_bindgen_test]
    fn an_enabled_button_fires_the_consumer_on_click() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_button(ButtonProps {
            handlers: ButtonHandlers {
                on_click: Some(Rc::new(move |_event: &web_sys::MouseEvent| {
                    recorder_for_cb.record("click");
                })),
                ..ButtonHandlers::default()
            },
            ..ButtonProps::default()
        });
        click(&button);
        assert_eq!(recorder.calls(), vec!["click"]);
    }

    // behavior.md "Keyboard interactions" (`Button.test.tsx:126-132`): disabled
    // suppresses keyboard activation — no handler calls from Enter/Space.
    #[wasm_bindgen_test]
    fn a_disabled_button_suppresses_keyboard_activation() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_button(ButtonProps {
            native_button: false,
            disabled: true,
            focusable_when_disabled: true,
            handlers: ButtonHandlers {
                on_click: Some(Rc::new(move |_event: &web_sys::MouseEvent| {
                    recorder_for_cb.record("click");
                })),
                ..ButtonHandlers::default()
            },
            ..ButtonProps::default()
        });
        fire_key(&button, "keydown", "Enter");
        fire_key(&button, "keyup", "Enter");
        fire_key(&button, "keydown", " ");
        fire_key(&button, "keyup", " ");
        assert!(
            recorder.calls().is_empty(),
            "keyboard activation is fully suppressed on the focusable-when-disabled button \
             (the policy's preventDefault-on-non-Tab keydown, useFocusableWhenDisabled.ts:23-28)"
        );
    }

    // behavior.md "Keyboard interactions" (`Button.test.tsx:69-70`): non-native,
    // enabled: Enter and Space each dispatch a real click — one per activation.
    // (The Space-on-keyup / Enter-on-keydown split and the synthetic dispatch's
    // modifier preservation are `use_button`'s suite-tested contracts; this pins
    // the facade wires the pipeline through.)
    #[wasm_bindgen_test]
    fn keyboard_activation_dispatches_real_clicks_on_a_non_native_button() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let button = mount_button(ButtonProps {
            native_button: false,
            handlers: ButtonHandlers {
                on_click: Some(Rc::new(move |_event: &web_sys::MouseEvent| {
                    recorder_for_cb.record("click");
                })),
                ..ButtonHandlers::default()
            },
            ..ButtonProps::default()
        });
        fire_key(&button, "keydown", "Enter");
        fire_key(&button, "keyup", " ");
        assert_eq!(
            recorder.calls(),
            vec!["click", "click"],
            "Enter (keydown) and Space (keyup) each activate once"
        );
    }

    // behavior.md "DOM structure" + upstream props precedence
    // (`useButton.ts:228` — `otherExternalProps` last, the `<Button type="submit">`
    // override, `Button.spec.tsx:4`): the consumer's plain attribute wins over the
    // internal bag's same-named member.
    #[wasm_bindgen_test]
    fn the_consumers_attribute_overrides_the_internal_type_button() {
        let button = mount_button(ButtonProps {
            element_attributes: vec![("type".to_string(), "submit".to_string())],
            ..ButtonProps::default()
        });
        let native = button.dyn_ref::<HtmlButtonElement>().unwrap();
        assert_eq!(
            native.type_(),
            "submit",
            "the user value wins (elementProps sits before getButtonProps in the merge, \
             so its plain attributes win — useButton.ts:228)"
        );
    }

    // behavior.md "DOM structure" (`Button.test.tsx:20-27,62`): the `render` prop
    // replaces the root — an `<a>` remains tag 'A', no wrapper element added.
    #[wasm_bindgen_test]
    fn the_render_prop_replaces_the_root_element_identity() {
        use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = button_element(ButtonProps {
            native_button: false,
            render_class_style: UseRenderElementComponentProps {
                render: Some(RenderProp::Element {
                    tag: "a".to_string(),
                    props: RenderElementProps::default(),
                }),
                ..UseRenderElementComponentProps::default()
            },
            ..ButtonProps::default()
        })
        .expect("the button renders");
        assert_eq!(
            rendered.tag, "a",
            "the render element is the root — the tag is preserved"
        );
        // The non-native semantics ride along regardless of tag
        // (`Button.test.tsx:26-27`).
        let attrs: Vec<_> = rendered
            .props
            .handlers
            .attributes
            .iter()
            .map(|(n, _)| n.clone())
            .collect();
        assert!(
            attrs.iter().any(|n| n == "role"),
            "role=button applies on the custom element"
        );
    }
}
