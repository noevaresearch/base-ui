//! Port of `packages/react/src/utils/getElementAtPoint.ts` — the hit-test helper that
//! takes a `getRootNode()` result (document or shadow root) rather than the owner
//! document, because `Document.elementFromPoint` retargets shadow content to the shadow
//! host, which then fails `contains()` checks against a popup inside that shadow root
//! (`getElementAtPoint.ts:3-5`; consumed by `useSwipeDismiss`, implementation.md,
//! "DOM/portal strategy and why" → "Shadow-DOM-safe lookups").
//!
//! Upstream duck-types the root as `Node & Partial<Pick<Document,
//! 'elementFromPoint'>>` and calls it only when the method exists
//! (`:11`). The two real `elementFromPoint` carriers are `Document` and `ShadowRoot`;
//! any other node (or a null root) yields `null` upstream, `None` here.

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Document, Element, Node, ShadowRoot};

/// Port of `getElementAtPoint` (`getElementAtPoint.ts:6-12`). The coordinates are CSS
/// pixels; the web-sys binding takes `f32` (JS numbers are doubles at runtime).
pub fn get_element_at_point(root: Option<&Node>, x: f64, y: f64) -> Option<Element> {
    let root = root?;
    let (x, y) = (x as f32, y as f32);

    // `typeof root?.elementFromPoint === 'function'` (`:11`): the duck-type check
    // narrows to the two node kinds carrying the method; everything else is `null`.
    if let Some(document) = root.dyn_ref::<Document>() {
        return document.element_from_point(x, y);
    }
    if let Some(shadow_root) = root.dyn_ref::<ShadowRoot>() {
        return shadow_root.element_from_point(x, y);
    }
    None
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use web_sys::{HtmlElement, ShadowRootInit, ShadowRootMode};

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    // The document-root fast path (`:11`): the element rendered at the point comes
    // back; a point outside the viewport yields `null`.
    #[wasm_bindgen_test]
    fn resolves_through_the_document_root() {
        let element = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        element
            .style()
            .set_css_text("position: absolute; left: 50px; top: 50px; width: 20px; height: 20px;");
        document().body().unwrap().append_child(&element).unwrap();

        assert_eq!(
            get_element_at_point(Some(document().as_ref() as &Node), 55.0, 55.0).as_ref(),
            Some(element.as_ref() as &Element)
        );
        assert_eq!(
            get_element_at_point(Some(document().as_ref() as &Node), 5000.0, 5000.0),
            None
        );

        document().body().unwrap().remove_child(&element).unwrap();
    }

    // The retargeting contract (`:3-5`): through the shadow root the *inner* element
    // is found, while the document root retargets the same point to the host.
    #[wasm_bindgen_test]
    fn resolves_shadow_content_through_its_own_root() {
        let host = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        host.style()
            .set_css_text("position: absolute; left: 50px; top: 50px; width: 40px; height: 40px;");
        let shadow = host
            .attach_shadow(&ShadowRootInit::new(ShadowRootMode::Open))
            .unwrap();
        let inner = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        inner.style().set_css_text("width: 20px; height: 20px;");
        shadow.append_child(&inner).unwrap();
        document().body().unwrap().append_child(&host).unwrap();

        let through_shadow = get_element_at_point(Some(shadow.as_ref() as &Node), 55.0, 55.0);
        let through_document = get_element_at_point(Some(document().as_ref() as &Node), 55.0, 55.0);

        assert_eq!(
            through_shadow.as_ref().map(|el| el.as_ref() as &Element),
            Some(inner.as_ref() as &Element)
        );
        assert_eq!(
            through_document.as_ref().map(|el| el.as_ref() as &Element),
            Some(host.as_ref() as &Element)
        );

        document().body().unwrap().remove_child(&host).unwrap();
    }

    // A root without an `elementFromPoint` method yields `null` upstream; only
    // `Document` and `ShadowRoot` carry it, so any other node is `None` here.
    #[wasm_bindgen_test]
    fn yields_none_for_a_root_without_the_method() {
        let div = document().create_element("div").unwrap();
        assert_eq!(
            get_element_at_point(Some(div.as_ref() as &Node), 0.0, 0.0),
            None
        );
        assert_eq!(get_element_at_point(None, 0.0, 0.0), None);
    }
}
