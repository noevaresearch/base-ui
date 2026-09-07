//! Port of `packages/utils/src/addEventListener.ts` (Base UI Phase A util).
//!
//! Upstream adds an event listener and returns a cleanup function that removes it, forwarding
//! the identical `(type, listener, options)` triple to both calls so that capture/passive
//! matching works for removal (`packages/utils/src/addEventListener.ts:52-62`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The listener is a `FnMut(&Event)` closure instead of a JS function or
//!   `{ handleEvent }` object.
//! - The returned unsubscribe is an owned handle rather than a `() => void` function, because
//!   the handle must own the `wasm_bindgen::Closure` to keep the listener alive. Call
//!   [`EventListenerUnsubscribe::unsubscribe`] explicitly, or drop the handle: dropping
//!   removes the listener (upstream would keep it registered if the unsubscribe function were
//!   never called, but a dropped Rust handle would leave a dangling callback behind, so
//!   removing is the only safe interpretation).

use wasm_bindgen::{JsCast, UnwrapThrowExt, prelude::Closure};
use web_sys::{AddEventListenerOptions, Event, EventTarget};

/// Options accepted by [`add_event_listener_with_options`], mirroring the upstream parameter
/// type `options?: boolean | AddEventListenerOptions`
/// (`packages/utils/src/addEventListener.ts:44`).
#[derive(Clone, Debug)]
pub enum EventListenerOptions {
    /// The bare `boolean` form: `true` means `{ capture: true }`, `false` means
    /// `{ capture: false }`.
    Capture(bool),
    /// The object form, supporting `capture`, `once`, and `passive`.
    Options(AddEventListenerOptions),
}

impl From<bool> for EventListenerOptions {
    fn from(capture: bool) -> Self {
        EventListenerOptions::Capture(capture)
    }
}

impl From<AddEventListenerOptions> for EventListenerOptions {
    fn from(options: AddEventListenerOptions) -> Self {
        EventListenerOptions::Options(options)
    }
}

/// The cleanup handle returned by [`add_event_listener`] and
/// [`add_event_listener_with_options`]. It owns the registered callback; see the module docs.
pub struct EventListenerUnsubscribe {
    target: EventTarget,
    event_type: String,
    callback: Option<Closure<dyn FnMut(Event)>>,
    options: Option<EventListenerOptions>,
}

impl EventListenerUnsubscribe {
    /// Removes the listener, forwarding the identical `(type, listener, options)` triple that
    /// was used to register it (`packages/utils/src/addEventListener.test.ts:17-19`).
    pub fn unsubscribe(mut self) {
        self.remove();
    }

    fn remove(&mut self) {
        if let Some(callback) = self.callback.take() {
            let function = callback.as_ref().unchecked_ref::<js_sys::Function>();
            let result = match self.options.as_ref() {
                Some(EventListenerOptions::Capture(capture)) => {
                    self.target.remove_event_listener_with_callback_and_bool(
                        &self.event_type,
                        function,
                        *capture,
                    )
                }
                Some(EventListenerOptions::Options(options)) => {
                    // web-sys types removal options as `EventListenerOptions` (mirroring the
                    // DOM's `removeEventListener` signature); removal matching only reads
                    // `capture`, which is exactly what the JS engine reads from the identical
                    // options object upstream forwards
                    // (`packages/utils/src/addEventListener.test.ts:17-19`).
                    let removal_options = web_sys::EventListenerOptions::new();
                    removal_options.set_capture(options.get_capture().unwrap_or(false));
                    self.target
                        .remove_event_listener_with_callback_and_event_listener_options(
                            &self.event_type,
                            function,
                            &removal_options,
                        )
                }
                None => self
                    .target
                    .remove_event_listener_with_callback(&self.event_type, function),
            };
            result.expect_throw("Base UI: failed to remove an event listener");
        }
    }
}

impl Drop for EventListenerUnsubscribe {
    fn drop(&mut self) {
        self.remove();
    }
}

/// Adds an event listener and returns a cleanup handle to remove it.
///
/// Forwards the exact `(type, listener)` pair to `target.addEventListener`; the returned
/// handle's [`EventListenerUnsubscribe::unsubscribe`] forwards the identical pair to
/// `target.removeEventListener`. The listener is delivered every matching event until
/// unsubscribed.
pub fn add_event_listener<T, F>(
    target: &T,
    event_type: &str,
    listener: F,
) -> EventListenerUnsubscribe
where
    T: AsRef<EventTarget> + ?Sized,
    F: FnMut(&Event) + 'static,
{
    add_event_listener_impl(target, event_type, listener, None)
}

/// Same as [`add_event_listener`], with the upstream optional fourth parameter filled in.
///
/// `options` accepts `bool` (the bare `boolean` form) or `web_sys::AddEventListenerOptions`
/// via [`Into`]. On unsubscribe, the identical options are forwarded so that capture/passive
/// matching works for removal (`packages/utils/src/addEventListener.test.ts:11-19`).
pub fn add_event_listener_with_options<T, F>(
    target: &T,
    event_type: &str,
    listener: F,
    options: impl Into<EventListenerOptions>,
) -> EventListenerUnsubscribe
where
    T: AsRef<EventTarget> + ?Sized,
    F: FnMut(&Event) + 'static,
{
    add_event_listener_impl(target, event_type, listener, Some(options.into()))
}

fn add_event_listener_impl<T, F>(
    target: &T,
    event_type: &str,
    mut listener: F,
    options: Option<EventListenerOptions>,
) -> EventListenerUnsubscribe
where
    T: AsRef<EventTarget> + ?Sized,
    F: FnMut(&Event) + 'static,
{
    let target = target.as_ref();
    let callback: Closure<dyn FnMut(Event)> = Closure::new(move |event: Event| {
        listener(&event);
    });
    let function = callback.as_ref().unchecked_ref::<js_sys::Function>();
    let result = match options.as_ref() {
        Some(EventListenerOptions::Capture(capture)) => {
            target.add_event_listener_with_callback_and_bool(event_type, function, *capture)
        }
        Some(EventListenerOptions::Options(options)) => target
            .add_event_listener_with_callback_and_add_event_listener_options(
                event_type, function, options,
            ),
        None => target.add_event_listener_with_callback(event_type, function),
    };
    result.expect_throw("Base UI: failed to add an event listener");
    EventListenerUnsubscribe {
        target: target.clone(),
        event_type: event_type.to_string(),
        callback: Some(callback),
        options,
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Event, EventInit, MouseEvent};

    // These tests exercise real DOM semantics (capture/passive options, document trees) that
    // Node's minimal EventTarget does not implement, so they run in a browser.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn click_event() -> Event {
        let init = EventInit::new();
        init.set_cancelable(true);
        Event::new_with_event_init_dict("click", &init).unwrap_throw()
    }

    // Mirrors `packages/utils/src/addEventListener.test.ts:5-20`, using real dispatch instead
    // of a mock target (the mock contract — identical arguments on add and remove — is
    // observable here through delivery and non-delivery of the event).
    #[wasm_bindgen_test]
    fn adds_the_listener_and_returns_an_unsubscribe_handle() {
        let target = EventTarget::new().unwrap_throw();
        let calls = Rc::new(Cell::new(0u32));
        let calls_in_listener = calls.clone();

        let unsubscribe = add_event_listener(&target, "click", move |_event: &Event| {
            calls_in_listener.set(calls_in_listener.get() + 1);
        });

        target.dispatch_event(&click_event()).unwrap_throw();
        assert_eq!(calls.get(), 1, "listener fires while subscribed");

        unsubscribe.unsubscribe();

        target.dispatch_event(&click_event()).unwrap_throw();
        assert_eq!(calls.get(), 1, "listener no longer fires after unsubscribe");
    }

    // The upstream test never dispatches an event (its listener is a `vi.fn()` mock), so event
    // delivery is only inferred there; this asserts it directly.
    #[wasm_bindgen_test]
    fn delivers_the_event_payload_to_the_listener() {
        let target = EventTarget::new().unwrap_throw();
        let received = Rc::new(Cell::new(false));

        let unsubscribe = add_event_listener(&target, "click", {
            let received = received.clone();
            move |event: &Event| received.set(event.type_() == "click")
        });

        target.dispatch_event(&click_event()).unwrap_throw();
        unsubscribe.unsubscribe();

        assert!(received.get(), "listener received the dispatched event");
    }

    // `capture: true` is what makes the parent's listener fire for a (non-bubbling) event
    // dispatched on the child; if unsubscribe forwarded different options, the listener would
    // stay registered and fire again (`packages/utils/src/addEventListener.test.ts:11-19`).
    #[wasm_bindgen_test]
    fn unsubscribes_with_the_identical_capture_option() {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let parent = document.create_element("div").unwrap_throw();
        let child = document.create_element("button").unwrap_throw();
        parent.append_child(&child).unwrap_throw();
        let calls = Rc::new(Cell::new(0u32));

        let unsubscribe = add_event_listener_with_options(
            &parent,
            "click",
            {
                let calls = calls.clone();
                move |_event: &Event| calls.set(calls.get() + 1)
            },
            true,
        );

        child
            .dispatch_event(&MouseEvent::new("click").unwrap_throw())
            .unwrap_throw();
        assert_eq!(
            calls.get(),
            1,
            "capture listener fires for the child target"
        );

        unsubscribe.unsubscribe();

        child
            .dispatch_event(&MouseEvent::new("click").unwrap_throw())
            .unwrap_throw();
        assert_eq!(
            calls.get(),
            1,
            "capture listener removed with identical options"
        );
    }

    // The object form of the options parameter, asserting the `passive` flag end-to-end: a
    // passive listener's `preventDefault` is ignored, so the dispatch is reported as not
    // canceled; resubscribing without `passive` makes the same listener's `preventDefault` take
    // effect.
    #[wasm_bindgen_test]
    fn forwards_full_options_objects() {
        let target = EventTarget::new().unwrap_throw();

        let passive_options = AddEventListenerOptions::new();
        passive_options.set_passive(true);
        let unsubscribe = add_event_listener_with_options(
            &target,
            "click",
            |event: &Event| event.prevent_default(),
            passive_options,
        );
        let canceled = target.dispatch_event(&click_event()).unwrap_throw();
        unsubscribe.unsubscribe();
        assert!(canceled, "passive listener cannot cancel the event");

        let unsubscribe = add_event_listener_with_options(
            &target,
            "click",
            |event: &Event| event.prevent_default(),
            AddEventListenerOptions::new(),
        );
        let canceled = target.dispatch_event(&click_event()).unwrap_throw();
        unsubscribe.unsubscribe();
        assert!(!canceled, "non-passive listener cancels the event");
    }

    #[wasm_bindgen_test]
    fn supports_overlapping_subscriptions_independently() {
        let target = EventTarget::new().unwrap_throw();
        let first_calls = Rc::new(Cell::new(0u32));
        let second_calls = Rc::new(Cell::new(0u32));

        let unsubscribe_first = add_event_listener(&target, "click", {
            let calls = first_calls.clone();
            move |_event: &Event| calls.set(calls.get() + 1)
        });
        let unsubscribe_second = add_event_listener(&target, "click", {
            let calls = second_calls.clone();
            move |_event: &Event| calls.set(calls.get() + 1)
        });

        target.dispatch_event(&click_event()).unwrap_throw();
        unsubscribe_first.unsubscribe();
        target.dispatch_event(&click_event()).unwrap_throw();

        assert_eq!(
            first_calls.get(),
            1,
            "first listener stops firing after its own unsubscribe"
        );
        assert_eq!(
            second_calls.get(),
            2,
            "second listener is unaffected by the first unsubscribe"
        );
        unsubscribe_second.unsubscribe();
    }

    // Documented Rust adaptation: the handle owns the callback, so dropping it without an
    // explicit unsubscribe removes the listener instead of leaving a dangling one behind.
    #[wasm_bindgen_test]
    fn dropping_the_handle_removes_the_listener() {
        let target = EventTarget::new().unwrap_throw();
        let calls = Rc::new(Cell::new(0u32));

        {
            let _handle = add_event_listener(&target, "click", {
                let calls = calls.clone();
                move |_event: &Event| calls.set(calls.get() + 1)
            });
        }

        target.dispatch_event(&click_event()).unwrap_throw();
        assert_eq!(
            calls.get(),
            0,
            "listener removed when the handle is dropped"
        );
    }

    // Exercises the target kinds the upstream type spec proves supported
    // (`packages/utils/src/addEventListener.spec.ts:4-29`): window, document, an element, a
    // generic `Element`, a MediaQueryList, and a bare `EventTarget`.
    #[wasm_bindgen_test]
    fn supports_the_upstream_target_kinds() {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let element = document.create_element("div").unwrap_throw();
        let generic_element: web_sys::Element = document.create_element("div").unwrap_throw();
        let media_query_list = window
            .match_media("(min-width: 1px)")
            .unwrap_throw()
            .unwrap_throw();
        let generic_target = EventTarget::new().unwrap_throw();

        let on_window = add_event_listener(&window, "click", |_event: &Event| {});
        let on_document = add_event_listener(&document, "click", |_event: &Event| {});
        let on_element = add_event_listener(&element, "keydown", |_event: &Event| {});
        let on_generic_element =
            add_event_listener(&generic_element, "keydown", |_event: &Event| {});
        let on_media_query_list =
            add_event_listener(&media_query_list, "change", |_event: &Event| {});
        let on_generic_target = add_event_listener(&generic_target, "custom", |_event: &Event| {});

        on_window.unsubscribe();
        on_document.unsubscribe();
        on_element.unsubscribe();
        on_generic_element.unsubscribe();
        on_media_query_list.unsubscribe();
        on_generic_target.unsubscribe();
    }
}
