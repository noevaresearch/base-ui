//! Port of `packages/react/src/utils/dispatchClickWithModifiers.ts` — dispatches a
//! constructed `click` on the target so it carries the source event's modifier state,
//! which `element.click()` always reports as unpressed
//! (`specs/library/internals/implementation.md`, "Event synthesis over `element.click()`":
//! `composed: true` keeps shadow-DOM hosts working, modifier preservation keeps
//! modifier-aware consumers working, and `detail: 0` marks it as keyboard-generated).
//!
//! Sole current consumer: [`crate::use_button`]'s keyboard click synthesis
//! (`packages/react/src/internals/use-button/useButton.ts:148,174,215`).
//!
//! ## Rust adaptations
//!
//! - Upstream's `sourceEvent: ModifierState` parameter is a structural type — any object
//!   exposing `shiftKey`/`ctrlKey`/`altKey`/`metaKey` (in practice the native mouse or
//!   keyboard event behind a React synthetic). The port reads those four members off the
//!   incoming [`web_sys::Event`] through dynamic property lookups, which is exactly the
//!   structural contract: `MouseEvent`, `PointerEvent`, and `KeyboardEvent` all carry
//!   them, and a value that does not simply reads as `false`.
//! - The `new (ownerWindow(target).PointerEvent)(...)` construction
//!   (`dispatchClickWithModifiers.ts:25`) looks the constructor up on the target's owner
//!   window so the event is built in the target's realm; the port performs the same
//!   window-scoped lookup through `Reflect`, falling back to the current-global
//!   [`web_sys::PointerEvent`] constructor when the lookup fails (the `ownerWindow`
//!   Phase A util supplies the window).
//! - The `{ detail = 0 }` options object (`dispatchClickWithModifiers.ts:22`) becomes an
//!   explicit `detail` parameter — there are no default arguments in Rust; the sole
//!   upstream call site passes no options, i.e. `0`.
//! - Upstream ignores `dispatchEvent`'s boolean return (`dispatchClickWithModifiers.ts:24`);
//!   so does the port.

use js_sys::Reflect;
use wasm_bindgen::JsCast;
use web_sys::{Element, PointerEvent};

use leptos_ui_utils::owner::owner_window;

/// The upstream `dispatchClickWithModifiers` (`dispatchClickWithModifiers.ts:19-36`).
pub fn dispatch_click_with_modifiers(target: &Element, source_event: &web_sys::Event, detail: i32) {
    let click = construct_click_event(target, source_event, detail);
    if let Some(click) = click {
        let _ = target.dispatch_event(click.as_ref());
    }
}

/// Reads one boolean modifier member off the source event — the structural
/// `ModifierState` access (`dispatchClickWithModifiers.ts:3-8`).
fn source_modifier(source_event: &web_sys::Event, key: &str) -> bool {
    Reflect::get(source_event, &key.into())
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

/// The `new (ownerWindow(target).PointerEvent)('click', {...})` construction
/// (`dispatchClickWithModifiers.ts:25-35`): the window-scoped constructor with the
/// fixed init members — `bubbles`/`cancelable`/`composed: true`, the requested
/// `detail`, and the four source modifiers.
fn construct_click_event(target: &Element, source_event: &web_sys::Event, detail: i32) -> Option<PointerEvent> {
    let shift_key = source_modifier(source_event, "shiftKey");
    let ctrl_key = source_modifier(source_event, "ctrlKey");
    let alt_key = source_modifier(source_event, "altKey");
    let meta_key = source_modifier(source_event, "metaKey");

    let init = js_sys::Object::new();
    let set = |key: &str, value: bool| {
        let _ = Reflect::set(&init, &key.into(), &value.into());
    };
    set("bubbles", true);
    set("cancelable", true);
    set("composed", true);
    let _ = Reflect::set(&init, &"detail".into(), &detail.into());
    set("shiftKey", shift_key);
    set("ctrlKey", ctrl_key);
    set("altKey", alt_key);
    set("metaKey", meta_key);

    let win = owner_window(Some(target.as_ref() as &web_sys::Node));
    let constructor = Reflect::get(win.unchecked_ref::<js_sys::Object>(), &"PointerEvent".into())
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok());

    if let Some(constructor) = constructor {
        // `new PointerEvent('click', init)` — the type string is the first argument.
        let args = js_sys::Array::new();
        args.push(&"click".into());
        args.push(&init.into());
        let event = Reflect::construct(&constructor, &args)
            .ok()
            .and_then(|value| value.dyn_into::<PointerEvent>().ok());
        if event.is_some() {
            return event;
        }
    }

    // The lookup failed (no window-scoped constructor) — the current-global
    // constructor produces the same event within one realm.
    let mut init_dict = web_sys::PointerEventInit::new();
    init_dict.set_bubbles(true);
    init_dict.set_cancelable(true);
    init_dict.set_composed(true);
    init_dict.set_detail(detail);
    init_dict.set_shift_key(shift_key);
    init_dict.set_ctrl_key(ctrl_key);
    init_dict.set_alt_key(alt_key);
    init_dict.set_meta_key(meta_key);
    PointerEvent::new_with_event_init_dict("click", &init_dict).ok()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Event, EventTarget, KeyboardEvent, KeyboardEventInit, MouseEvent};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn button_element() -> Element {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let element = document
            .create_element("button")
            .expect("create_element failed");
        // Attached to the document — an element outside the tree has no propagation
        // path, so a bubbled click would never reach a document-level observer.
        document
            .body()
            .expect("a body")
            .append_child(&element)
            .unwrap();
        element
    }

    /// Counts `click` events reaching the document — the dispatched click bubbles
    /// (`bubbles: true`), so a document-level listener observes it.
    fn click_counter() -> (Rc<RefCell<Vec<MouseEvent>>>, EventListenerUnsubscribe) {
        let clicks: Rc<RefCell<Vec<MouseEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let listener_clicks = Rc::clone(&clicks);
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let unsubscribe = leptos_ui_utils::add_event_listener(
            document.as_ref() as &EventTarget,
            "click",
            move |event: &Event| {
                if let Some(click) = event.dyn_ref::<MouseEvent>() {
                    // The synthetic dispatched click and real user clicks both land
                    // here; tests only dispatch, so every entry is a dispatched one.
                    listener_clicks.borrow_mut().push(click.clone());
                }
            },
        );
        (clicks, unsubscribe)
    }

    fn key_event(key: &str) -> KeyboardEvent {
        let init = KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_key(key);
        init.set_shift_key(true);
        init.set_ctrl_key(true);
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .expect("KeyboardEvent failed")
    }

    #[wasm_bindgen_test]
    fn dispatches_a_click_carrying_the_source_modifiers_with_detail_0() {
        let (clicks, _keep) = click_counter();
        let target = button_element();

        let source = key_event("Enter");
        dispatch_click_with_modifiers(&target, source.as_ref(), 0);

        let clicks = clicks.borrow();
        assert_eq!(clicks.len(), 1, "the constructed click dispatches");
        let click = &clicks[0];
        assert_eq!(click.detail(), 0, "detail defaults to 0 — the native convention for keyboard-generated clicks (dispatchClickWithModifiers.ts:15-17)");
        assert!(click.shift_key(), "the source's shift state is preserved");
        assert!(click.ctrl_key(), "the source's ctrl state is preserved");
        assert!(!click.alt_key(), "an unpressed modifier reads as unpressed");
        assert!(!click.meta_key(), "an unpressed modifier reads as unpressed");
        assert_eq!(
            click
                .target()
                .map(|t| t.unchecked_into::<Element>() == target),
            Some(true),
            "the click targets the element it was dispatched on"
        );
    }

    #[wasm_bindgen_test]
    fn honors_an_explicit_detail_for_mouse_gestures() {
        let (clicks, _keep) = click_counter();
        let target = button_element();

        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        let source =
            MouseEvent::new_with_mouse_event_init_dict("mousedown", &init).expect("failed");

        dispatch_click_with_modifiers(&target, source.as_ref(), 1);

        let clicks = clicks.borrow();
        assert_eq!(clicks.len(), 1);
        assert_eq!(
            clicks[0].detail(),
            1,
            "the detail option passes through (dispatchClickWithModifiers.ts:15-17)"
        );
    }

    #[wasm_bindgen_test]
    fn the_untrusted_click_still_runs_native_activation_behavior() {
        // Like `click()`, the dispatched click activates the button's implicit
        // behavior — observed here as a form submission through a type="submit"
        // button (dispatchClickWithModifiers.ts:10-14).
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let form = document.create_element("form").expect("create_element failed");
        let submitted: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let submitted_listener = Rc::clone(&submitted);
        let unsubscribe = leptos_ui_utils::add_event_listener(
            form.as_ref() as &EventTarget,
            "submit",
            move |event: &Event| {
                event.prevent_default();
                *submitted_listener.borrow_mut() = true;
            },
        );
        let button = button_element();
        let _ = button.set_attribute("type", "submit");
        let _ = form.append_child(&button);
        let _ = document.body().expect("a body").append_child(&form);

        let source = key_event("Enter");
        dispatch_click_with_modifiers(&button, source.as_ref(), 0);

        assert!(
            *submitted.borrow(),
            "the untrusted click triggers the form submit (dispatchClickWithModifiers.ts:12-13)"
        );
        unsubscribe.unsubscribe();
    }
}
