//! Port of `packages/utils/src/shadowDom.ts` (Base UI Phase A util).
//!
//! Upstream is a three-export shadow-DOM-safe DOM/event utility module:
//!
//! - `activeElement(doc)` descends through nested shadow roots from `doc.activeElement` while
//!   each level's `shadowRoot.activeElement` is non-null
//!   (`packages/utils/src/shadowDom.ts:3-11`).
//! - `contains(parent, child)` answers containment across shadow boundaries: null-ish arguments
//!   return `false` (`packages/utils/src/shadowDom.ts:14-16`), the native `contains` is attempted
//!   first (`packages/utils/src/shadowDom.ts:21-23`), and when the child's root node is a
//!   `ShadowRoot` the walk climbs out of shadow roots through `parentNode || host`
//!   (`packages/utils/src/shadowDom.ts:26-34`), with the shadow-root predicate delegated to
//!   `isShadowRoot` from `@floating-ui/utils/dom` (`packages/utils/src/shadowDom.ts:1,26`;
//!   `specs/utils/shadowDom.md` "Edge cases").
//! - `getTarget(event)` returns the composed-path target while the event is being dispatched and
//!   falls back to `event.target` once dispatch has completed
//!   (`packages/utils/src/shadowDom.ts:39-48`) — the only export the upstream suite exercises
//!   (`packages/utils/src/shadowDom.test.ts:1-32`, cited by `specs/utils/shadowDom.md`).
//!
//! Only `getTarget` is test-proven upstream; `activeElement` and `contains` are marked UNVERIFIED
//! in `specs/utils/shadowDom.md` ("Public API surface") and are pinned here by
//! implementation-derived tests rather than a reference suite.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream's optional/nullable parameters become [`Option`], with `None` standing for JS
//!   `null`/`undefined` (which upstream treats identically via `!parent || !child` and `?.`).
//! - `getTarget`'s `'composedPath' in event` guard (`packages/utils/src/shadowDom.ts:40`) maps to
//!   a `js_sys::Reflect::has` probe, and `event.composedPath()[0] ?? event.target` maps to an
//!   undefined/null check on the first path entry (JS `??` treats exactly `null`/`undefined` as
//!   missing). A failed probe falls through to the same `target` read as the older-browser branch
//!   (`packages/utils/src/shadowDom.ts:46-48`).
//! - The delegated `isShadowRoot` is mirrored directly rather than bound to `floating-ui-leptos`
//!   (the precedent set by `owner.rs` for the same situation): upstream is
//!   `value instanceof ShadowRoot || value instanceof getWindow(value).ShadowRoot`
//!   (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/
//!   floating-ui.utils.dom.mjs:39-44`). The first disjunct maps to
//!   `JsCast::dyn_ref::<ShadowRoot>` (an `instanceof` against this module's realm); the second —
//!   which keeps cross-realm shadow roots (e.g. inside an iframe's document) recognized — maps to
//!   `constructor.prototype.isPrototypeOf(value)` on the node's own realm's `ShadowRoot`
//!   constructor, which is `instanceof`'s exact prototype-chain semantics. The `typeof ShadowRoot
//!   === 'undefined'` guard (`floating-ui.utils.dom.mjs:42`) maps to the constructor lookup
//!   failing (an absent global is not a function), and the `!hasWindow()` guard is moot in a wasm
//!   browser context, which always has a window.
//! - The host-chain walk's `(next.parentNode as Element) || (next as unknown as ShadowRoot).host`
//!   (`packages/utils/src/shadowDom.ts:32`) is a no-op cast lie in upstream: `parentNode` of an
//!   element slotted directly into a shadow root *is* the `ShadowRoot` object, and the `host`
//!   read only fires when `parentNode` is null (i.e. `next` is itself a `ShadowRoot`, whose
//!   `host` is the next hop outward). The port models `next` honestly as an [`Option<Node>`] and
//!   branches `parentNode` first, then the `ShadowRoot` host read — including upstream's
//!   implicit loop exit when neither exists (a non-shadow-root node with no parent yields an
//!   `undefined` host, ending the walk).
//! - JS object identity (`parent === next`, `packages/utils/src/shadowDom.ts:29`) maps to
//!   `JsValue`'s `PartialEq` (JS `===`), the same comparison `owner.rs` relies on for realm
//!   identity.

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, Event, EventTarget, Node, ShadowRoot};

/// The upstream `activeElement` export (`packages/utils/src/shadowDom.ts:3-11`): the document's
/// active element, descended through nested shadow roots while each level's
/// `shadowRoot.activeElement` is non-null — i.e. the actually focused element even when focus
/// sits inside a (possibly nested) open or closed shadow tree, where `doc.activeElement` alone
/// would stop at the outermost host. UNVERIFIED upstream (no test asserts it,
/// `specs/utils/shadowDom.md` "Focus management"); pinned by this module's tests.
pub fn active_element(document: &Document) -> Option<Element> {
    let mut element = document.active_element();

    // while (element?.shadowRoot?.activeElement != null) { element = element.shadowRoot.activeElement; }
    loop {
        let deeper = element
            .as_ref()
            .and_then(Element::shadow_root)
            .and_then(|shadow_root| shadow_root.active_element());
        match deeper {
            Some(nested) => element = Some(nested),
            None => break,
        }
    }

    element
}

/// The upstream `contains` export (`packages/utils/src/shadowDom.ts:13-37`): shadow-DOM-safe
/// containment. Null-ish arguments return `false`
/// (`packages/utils/src/shadowDom.ts:14-16`); otherwise the native `contains` answers first
/// (`packages/utils/src/shadowDom.ts:21-23`), and when that misses but the child's root node is a
/// `ShadowRoot`, the walk climbs `parentNode || host` until it meets the parent or runs out of
/// ancestors (`packages/utils/src/shadowDom.ts:26-34`). Cross-shadow containment is UNVERIFIED
/// upstream (`specs/utils/shadowDom.md` "Edge cases"); pinned by this module's tests.
pub fn contains(parent: Option<&Element>, child: Option<&Element>) -> bool {
    let (Some(parent), Some(child)) = (parent, child) else {
        return false;
    };

    let parent_node: &Node = parent.as_ref();
    let child_node: &Node = child.as_ref();
    let root_node = child_node.get_root_node();

    // First, attempt with the faster native method.
    if parent_node.contains(Some(child_node)) {
        return true;
    }

    // Then fall back to traversing out of shadow roots when needed.
    if is_shadow_root(&root_node) {
        let parent_value: &JsValue = parent_node.as_ref();
        let mut next: Option<Node> = Some(child_node.clone());
        while let Some(current) = next {
            let current_value: &JsValue = current.as_ref();
            if current_value == parent_value {
                return true;
            }
            // next = (next.parentNode as Element) || (next as unknown as ShadowRoot).host;
            next = current.parent_node().or_else(|| {
                current
                    .dyn_ref::<ShadowRoot>()
                    .map(|shadow_root| shadow_root.host().unchecked_into::<Node>())
            });
        }
    }

    false
}

/// The upstream `getTarget` export (`packages/utils/src/shadowDom.ts:39-48`): the event's real
/// target. While the event is being dispatched this is the composed-path target — the element the
/// event was originally dispatched on, even when the handler is attached to a retargeting
/// ancestor (`packages/utils/src/shadowDom.test.ts:5-19`) — and once dispatch has completed the
/// composed path is empty (`packages/utils/src/shadowDom.test.ts:28`), so `event.target` answers
/// (`packages/utils/src/shadowDom.test.ts:21-31`). `None` models upstream's nullable `target`.
pub fn get_target(event: &Event) -> Option<EventTarget> {
    if js_sys::Reflect::has(event.as_ref(), &JsValue::from_str("composedPath")).unwrap_or(false) {
        // The composed path is empty once the event is no longer being dispatched,
        // so fall back to `target` for handlers running after dispatch completes.
        let first = event.composed_path().get(0);
        if !first.is_undefined() && !first.is_null() {
            return Some(first.unchecked_into::<EventTarget>());
        }
        return event.target();
    }

    // TS assumes `composedPath()` always exists, but older browsers without
    // shadow DOM support still fall back to `target`.
    event.target()
}

/// The delegated shadow-root predicate (`packages/utils/src/shadowDom.ts:1`), mirrored from
/// `@floating-ui/utils/dom`'s `isShadowRoot`
/// (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/
/// floating-ui.utils.dom.mjs:39-44`) — see the module docs for the adaptation notes.
fn is_shadow_root(value: &Node) -> bool {
    // `value instanceof ShadowRoot`
    if value.dyn_ref::<ShadowRoot>().is_some() {
        return true;
    }

    // `value instanceof getWindow(value).ShadowRoot` — the node's own realm, so a shadow root
    // from another realm (e.g. an iframe's document) is still recognized.
    let Some(window) = value
        .owner_document()
        .and_then(|document| document.default_view())
    else {
        return false;
    };
    let Ok(constructor) = js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("ShadowRoot"))
    else {
        return false;
    };
    let Ok(constructor) = constructor.dyn_into::<js_sys::Function>() else {
        return false;
    };
    // `value instanceof constructor` ≡ `constructor.prototype.isPrototypeOf(value)`
    let Ok(prototype) = js_sys::Reflect::get(constructor.as_ref(), &JsValue::from_str("prototype"))
    else {
        return false;
    };
    let Ok(prototype) = prototype.dyn_into::<js_sys::Object>() else {
        return false;
    };
    let value: &JsValue = value.as_ref();
    prototype.is_prototype_of(value)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Event, EventInit, EventTarget, HtmlElement, ShadowRootInit, ShadowRootMode};

    use super::*;

    // The behavior under test is real DOM event dispatch, focus, and shadow-root traversal —
    // shadow roots cannot be constructed outside a browser, so the tests run via the wasm32 test
    // runner (`.cargo/config.toml` wires it to chromedriver), like the crate's other wasm test
    // modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap_throw().document().unwrap_throw()
    }

    // `JsValue`'s `PartialEq` is JS `===` (wasm-bindgen docs), which for two DOM object
    // references is identity — exactly the equality the composed-path target assertion needs.
    fn same_object(a: &JsValue, b: &JsValue) -> bool {
        a == b
    }

    fn append_to_body(document: &web_sys::Document, element: &Element) {
        document
            .body()
            .unwrap_throw()
            .append_child(element)
            .unwrap_throw();
    }

    fn attach_open_shadow_root(host: &Element) -> web_sys::ShadowRoot {
        host.attach_shadow(&ShadowRootInit::new(ShadowRootMode::Open))
            .unwrap_throw()
    }

    // Mirrors `packages/utils/src/shadowDom.test.ts:5-19`: a bubbling `click` dispatched on a
    // child is handled by a listener on the parent, and `getTarget` inside that listener returns
    // the child — the composed-path target — not the retargeted ancestor.
    #[wasm_bindgen_test]
    fn get_target_returns_the_composed_path_target_during_dispatch() {
        let document = document();
        let parent = document.create_element("div").unwrap_throw();
        let child = document.create_element("span").unwrap_throw();
        parent.append_child(&child).unwrap_throw();
        append_to_body(&document, &parent);

        let observed: Rc<RefCell<Option<JsValue>>> = Rc::new(RefCell::new(None));
        let listener = {
            let observed = Rc::clone(&observed);
            Closure::<dyn FnMut(&Event)>::new(move |event: &Event| {
                *observed.borrow_mut() = get_target(event).map(JsValue::from);
            })
        };
        let parent_target: &EventTarget = parent.as_ref();
        parent_target
            .add_event_listener_with_callback("click", listener.as_ref().unchecked_ref())
            .unwrap_throw();

        let event_init = EventInit::new();
        event_init.set_bubbles(true);
        let event =
            Event::new_with_event_init_dict("click", &event_init).unwrap_throw();
        child.dispatch_event(&event).unwrap_throw();

        let target = observed
            .borrow()
            .clone()
            .expect_throw("the listener recorded getTarget's result");
        assert!(same_object(&target, child.as_ref()));

        parent_target
            .remove_event_listener_with_callback("click", listener.as_ref().unchecked_ref())
            .unwrap_throw();
        parent.remove();
    }

    // Mirrors `packages/utils/src/shadowDom.test.ts:21-31`: once dispatch has completed,
    // `composedPath()` is empty and `getTarget` still returns the element dispatched upon via
    // the `event.target` fallback.
    #[wasm_bindgen_test]
    fn get_target_falls_back_to_target_once_dispatch_has_completed() {
        let document = document();
        let element = document.create_element("div").unwrap_throw();
        append_to_body(&document, &element);

        let event = Event::new("click").unwrap_throw();
        element.dispatch_event(&event).unwrap_throw();

        assert_eq!(event.composed_path().length(), 0);
        let target = get_target(&event).expect_throw("the fallback target is present");
        assert!(same_object(target.as_ref(), element.as_ref()));

        element.remove();
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:46-48`, UNVERIFIED upstream —
    // `specs/utils/shadowDom.md` "Events"): an event object without `composedPath` (the
    // older-browser case the `'composedPath' in event` guard exists for) falls back to `target`.
    // A plain object carrying only a `target` property stands in for such an event, which no
    // real modern event can be.
    #[wasm_bindgen_test]
    fn get_target_falls_back_to_target_when_composed_path_is_absent() {
        let document = document();
        let element = document.create_element("div").unwrap_throw();

        let synthetic = js_sys::Object::new();
        js_sys::Reflect::set(synthetic.as_ref(), &JsValue::from_str("target"), element.as_ref())
            .unwrap_throw();
        assert!(
            !js_sys::Reflect::has(synthetic.as_ref(), &JsValue::from_str("composedPath"))
                .unwrap_throw()
        );
        let synthetic: Event = synthetic.unchecked_into();

        let target = get_target(&synthetic).expect_throw("the fallback target is present");
        assert!(same_object(target.as_ref(), element.as_ref()));
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:3-11`, UNVERIFIED upstream —
    // `specs/utils/shadowDom.md` "Focus management"): a focused light-DOM element is returned
    // directly, without any shadow-root descent.
    #[wasm_bindgen_test]
    fn active_element_returns_the_focused_light_dom_element() {
        let document = document();
        let input = document.create_element("input").unwrap_throw();
        append_to_body(&document, &input);
        input
            .unchecked_ref::<HtmlElement>()
            .focus()
            .unwrap_throw();

        let focused = active_element(&document).expect_throw("a light-DOM element is focused");
        assert!(same_object(focused.as_ref(), input.as_ref()));

        input.remove();
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:3-11`, UNVERIFIED upstream —
    // `specs/utils/shadowDom.md` "Focus management"): with focus inside nested shadow roots,
    // `document.activeElement` stops at the outermost host, and the walk descends each level's
    // `shadowRoot.activeElement` to reach the actually focused element.
    #[wasm_bindgen_test]
    fn active_element_descends_through_nested_shadow_roots() {
        let document = document();
        let outer_host = document.create_element("div").unwrap_throw();
        let outer_shadow = attach_open_shadow_root(&outer_host);
        let inner_host = document.create_element("div").unwrap_throw();
        outer_shadow
            .unchecked_ref::<Node>()
            .append_child(&inner_host)
            .unwrap_throw();
        let inner_shadow = attach_open_shadow_root(&inner_host);
        let button = document.create_element("button").unwrap_throw();
        inner_shadow
            .unchecked_ref::<Node>()
            .append_child(&button)
            .unwrap_throw();
        append_to_body(&document, &outer_host);
        button
            .unchecked_ref::<HtmlElement>()
            .focus()
            .unwrap_throw();

        let focused = active_element(&document).expect_throw("a shadow-DOM element is focused");
        assert!(same_object(focused.as_ref(), button.as_ref()));

        outer_host.remove();
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:21-23`, UNVERIFIED upstream —
    // `specs/utils/shadowDom.md` "Edge cases"): light-DOM containment is answered by the native
    // `contains`, which includes the parent itself and excludes reversed or unrelated pairs.
    #[wasm_bindgen_test]
    fn contains_answers_light_dom_containment_natively() {
        let document = document();
        let parent = document.create_element("div").unwrap_throw();
        let child = document.create_element("span").unwrap_throw();
        let other = document.create_element("p").unwrap_throw();
        parent.append_child(&child).unwrap_throw();

        assert!(contains(Some(&parent), Some(&child)));
        assert!(contains(Some(&parent), Some(&parent)));
        assert!(!contains(Some(&child), Some(&parent)));
        assert!(!contains(Some(&parent), Some(&other)));
    }

    // Mirrors `packages/utils/src/shadowDom.ts:14-16`: null-ish arguments return `false` before
    // any DOM access.
    #[wasm_bindgen_test]
    fn contains_answers_false_for_nullish_arguments() {
        let document = document();
        let parent = document.create_element("div").unwrap_throw();

        assert!(!contains(None, Some(&parent)));
        assert!(!contains(Some(&parent), None));
        assert!(!contains(None, None));
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:26-34`, UNVERIFIED upstream —
    // `specs/utils/shadowDom.md` "Edge cases"): the native method misses across the shadow
    // boundary, and the host-chain walk finds the shadow-tree descendant from its host — and
    // only from it.
    #[wasm_bindgen_test]
    fn contains_traverses_out_of_a_shadow_root_to_the_host() {
        let document = document();
        let host = document.create_element("div").unwrap_throw();
        let shadow_root = attach_open_shadow_root(&host);
        let inner = document.create_element("span").unwrap_throw();
        shadow_root
            .unchecked_ref::<Node>()
            .append_child(&inner)
            .unwrap_throw();
        append_to_body(&document, &host);
        let unrelated = document.create_element("p").unwrap_throw();
        append_to_body(&document, &unrelated);

        assert!(contains(Some(&host), Some(&inner)));
        assert!(!contains(Some(&unrelated), Some(&inner)));
        assert!(!contains(Some(&inner), Some(&host)));

        host.remove();
        unrelated.remove();
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:26-34`, UNVERIFIED upstream):
    // the walk alternates `parentNode` and `host` hops, so containment holds across *nested*
    // shadow roots too: inner → innerShadowRoot → innerHost → outerShadowRoot → outerHost.
    #[wasm_bindgen_test]
    fn contains_traverses_through_nested_shadow_roots() {
        let document = document();
        let outer_host = document.create_element("div").unwrap_throw();
        let outer_shadow = attach_open_shadow_root(&outer_host);
        let inner_host = document.create_element("div").unwrap_throw();
        outer_shadow
            .unchecked_ref::<Node>()
            .append_child(&inner_host)
            .unwrap_throw();
        let inner_shadow = attach_open_shadow_root(&inner_host);
        let inner = document.create_element("span").unwrap_throw();
        inner_shadow
            .unchecked_ref::<Node>()
            .append_child(&inner)
            .unwrap_throw();
        append_to_body(&document, &outer_host);

        assert!(contains(Some(&outer_host), Some(&inner)));

        outer_host.remove();
    }

    // Implementation-derived (`packages/utils/src/shadowDom.ts:18-34`, UNVERIFIED upstream):
    // neither the native attempt nor the host-chain walk requires document attachment — a
    // detached host's shadow root is still the child's root node.
    #[wasm_bindgen_test]
    fn contains_works_for_a_detached_shadow_root() {
        let document = document();
        let host = document.create_element("div").unwrap_throw();
        let shadow_root = attach_open_shadow_root(&host);
        let inner = document.create_element("span").unwrap_throw();
        shadow_root
            .unchecked_ref::<Node>()
            .append_child(&inner)
            .unwrap_throw();

        assert!(contains(Some(&host), Some(&inner)));
    }
}
