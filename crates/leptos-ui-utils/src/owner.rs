//! Port of `packages/utils/src/owner.ts` (Base UI Phase A util).
//!
//! Upstream is a five-line module with exactly two exports and no logic of its own beyond one
//! fallback branch each (`packages/utils/src/owner.ts:1-5`):
//!
//! - `ownerWindow` is a pure re-export alias of `getWindow` from `@floating-ui/utils/dom`
//!   (`packages/utils/src/owner.ts:1`). The delegated implementation is
//!   `(node == null || node.ownerDocument?.defaultView) || window`
//!   (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/
//!   floating-ui.utils.dom.mjs:13-16`); the port mirrors those semantics directly rather than
//!   binding to `floating-ui-leptos` — `specs/utils/owner.md` records that crate only as an
//!   observation ("a natural bind target"), noting the unit's own TODO entry has no
//!   `wraps-external:` field, so there is no recorded delegation to honor.
//! - `ownerDocument(node) { return node?.ownerDocument || document; }`
//!   (`packages/utils/src/owner.ts:3-4`): resolve the node's own document, falling back to the
//!   browser-global document when the node is null-ish or its `ownerDocument` is falsy.
//!
//! The unit has no test file upstream — `ralph/generated/utils.json:168-173` lists
//! `testFiles: []` — so every behavioral claim is implementation-derived
//! (`specs/utils/owner.md` marks all of them UNVERIFIED) and pinned by this module's own tests
//! rather than a reference suite.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Both parameters become `Option<&Node>`: `None` maps to upstream's `null` (and to a runtime
//!   `undefined`, which both implementations treat identically — `ownerDocument`'s `?.` and
//!   `getWindow`'s `node == null`). `Node` is the minimal DOM type carrying `ownerDocument`;
//!   callers holding a more derived element type (`Element`, `HtmlElement`, …) coerce through
//!   the macro-generated `AsRef<Node>` impls, and a caller holding a non-Node `EventTarget`
//!   (e.g. the window, which upstream's `any` parameter accepted) reproduces the JS behavior —
//!   a property access that yields `undefined`, i.e. the global fallback — with
//!   `JsCast::dyn_ref::<Node>()` returning `None`.
//! - The browser-global `document` / `window` fallbacks become `web_sys::window()` unwraps: in
//!   a wasm browser context the global realm is exactly that window/document pair. Upstream has
//!   no SSR guard and would throw a bare `ReferenceError` when touching the `document` global
//!   in a realm without one (`packages/utils/src/owner.ts:4`); `unwrap_throw` preserves
//!   "throws when there is no global realm" as the failure mode.

use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Document, Node, Window};

/// The upstream `ownerWindow` export (`packages/utils/src/owner.ts:1`): a re-export alias of
/// `@floating-ui/utils/dom`'s `getWindow`, whose implementation
/// (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/
/// floating-ui.utils.dom.mjs:13-16`) resolves `node.ownerDocument.defaultView` and falls back
/// to the global window when the node is null-ish, its `ownerDocument` is null, or its
/// document has no `defaultView`. This is the realm resolution consumers rely on for
/// iframe/shadow-root safety — e.g. `ownerWindow(element).getComputedStyle(element)`
/// (`packages/react/src/tabs/indicator/TabsIndicator.tsx:258`).
pub fn owner_window(node: Option<&Node>) -> Window {
    node.and_then(Node::owner_document)
        .and_then(|document| document.default_view())
        .unwrap_or_else(global_window)
}

/// The upstream `ownerDocument` export (`packages/utils/src/owner.ts:3-4`):
/// `return node?.ownerDocument || document;` — the node's own document by native DOM
/// semantics, falling back to the global document for a null-ish node or a falsy
/// `ownerDocument`. That fallback is what makes the util safe for
/// `ownerDocument(ref.current)` before mount (`specs/utils/owner.md`, "Edge cases").
pub fn owner_document(node: Option<&Node>) -> Document {
    node.and_then(Node::owner_document)
        .unwrap_or_else(global_document)
}

fn global_window() -> Window {
    web_sys::window().unwrap_throw()
}

fn global_document() -> Document {
    global_window().document().unwrap_throw()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Document, HtmlIFrameElement, Node};

    use super::*;

    // The behavior under test is real DOM realm resolution — `ownerDocument`/`defaultView`
    // chains across documents — so the tests run in a real browser via the wasm32 test runner
    // (`.cargo/config.toml` wires it to chromedriver), like the crate's other wasm test
    // modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> Document {
        web_sys::window().unwrap_throw().document().unwrap_throw()
    }

    // `JsValue`'s `PartialEq` is JS `===` (wasm-bindgen docs), which for two DOM object
    // references is identity — exactly the equality "falls back to the global realm" needs.
    fn same_object(a: &JsValue, b: &JsValue) -> bool {
        a == b
    }

    // A standalone secondary document (never attached to the page): its nodes belong to that
    // document, and the document itself has a null `ownerDocument` and a null `defaultView`
    // (DOM spec), which is what the fallback-branch tests below observe.
    fn other_document(main: &Document) -> Document {
        main.implementation()
            .unwrap_throw()
            .create_html_document_with_title("owner test other document")
            .unwrap_throw()
    }

    // Implementation-derived (`packages/utils/src/owner.ts:4`): the `|| document` branch treats
    // a null-ish node as "use the global document" (`specs/utils/owner.md`, "Edge cases").
    #[wasm_bindgen_test]
    fn owner_document_with_no_node_falls_back_to_the_global_document() {
        let resolved = owner_document(None);
        assert!(same_object(resolved.as_ref(), global_document().as_ref()));
    }

    // Implementation-derived (`packages/utils/src/owner.ts:4`): an element resolves to the
    // document that created it — even detached, since `ownerDocument` is retained from
    // creation (`specs/utils/owner.md`, "Edge cases": the fallback is what makes
    // `ownerDocument(ref.current)` safe before mount).
    #[wasm_bindgen_test]
    fn owner_document_resolves_an_element_to_the_document_that_created_it() {
        let document = document();
        let element = document.create_element("div").unwrap_throw();
        let node: &Node = element.as_ref();

        let resolved = owner_document(Some(node));
        assert!(same_object(resolved.as_ref(), document.as_ref()));
    }

    // Implementation-derived (`packages/utils/src/owner.ts:4`): resolution goes through the
    // node's OWN `ownerDocument`, so an element from another document resolves to that
    // document — not the global one (`specs/utils/owner.md`, "Edge cases": cross-realm
    // resolution adds no logic beyond the fallback). A standalone `createHTMLDocument`
    // document stands in for the iframe/shadow-root case no upstream test exercises.
    #[wasm_bindgen_test]
    fn owner_document_resolves_an_element_from_another_document_to_that_document() {
        let document = document();
        let other = other_document(&document);
        let element = other.create_element("div").unwrap_throw();
        let node: &Node = element.as_ref();

        let resolved = owner_document(Some(node));
        assert!(same_object(resolved.as_ref(), other.as_ref()));
        assert!(!same_object(resolved.as_ref(), document.as_ref()));
    }

    // Implementation-derived (`packages/utils/src/owner.ts:4`): a node whose `ownerDocument`
    // is falsy falls back to the global document (`specs/utils/owner.md`, "Edge cases"). A
    // `Document` node's own `ownerDocument` is null by DOM spec, so passing one exercises
    // exactly that branch — and the fallback is observable because the global document is not
    // the standalone document.
    #[wasm_bindgen_test]
    fn owner_document_falls_back_when_the_node_has_no_owner_document() {
        let document = document();
        let other = other_document(&document);
        let node: &Node = other.as_ref();

        let resolved = owner_document(Some(node));
        assert!(same_object(resolved.as_ref(), document.as_ref()));
        assert!(!same_object(resolved.as_ref(), other.as_ref()));
    }

    // Implementation-derived (floating-ui.utils.dom.mjs:13-16): a null-ish node means the
    // global window (`specs/utils/owner.md`, "Edge cases": `ownerWindow` edge cases are owned
    // by the delegated `getWindow` implementation).
    #[wasm_bindgen_test]
    fn owner_window_with_no_node_falls_back_to_the_global_window() {
        let resolved = owner_window(None);
        assert!(same_object(
            resolved.as_ref(),
            web_sys::window().unwrap_throw().as_ref()
        ));
    }

    // Implementation-derived (floating-ui.utils.dom.mjs:13-16): an element resolves to its
    // document's `defaultView`.
    #[wasm_bindgen_test]
    fn owner_window_resolves_an_element_to_its_document_s_window() {
        let element = document().create_element("div").unwrap_throw();
        let node: &Node = element.as_ref();

        let resolved = owner_window(Some(node));
        assert!(same_object(
            resolved.as_ref(),
            web_sys::window().unwrap_throw().as_ref()
        ));
    }

    // Implementation-derived (floating-ui.utils.dom.mjs:13-16): a document without a
    // `defaultView` (a standalone `createHTMLDocument` document has a null one) sends the
    // resolution through the `|| window` fallback to the global window.
    #[wasm_bindgen_test]
    fn owner_window_falls_back_when_the_document_has_no_window() {
        let other = other_document(&document());
        let element = other.create_element("div").unwrap_throw();
        let node: &Node = element.as_ref();

        let resolved = owner_window(Some(node));
        assert!(same_object(
            resolved.as_ref(),
            web_sys::window().unwrap_throw().as_ref()
        ));
    }

    // The util's core purpose (`specs/utils/owner.md`, "Accessibility"/"DOM structure"):
    // resolve the correct realm through a DOM node. An iframe is the only in-page way to get a
    // genuinely separate realm, so this is the one test that attaches to the shared body —
    // and removes its node again so nothing is left behind.
    #[wasm_bindgen_test]
    fn resolves_across_realms_to_the_iframe_s_own_window_and_document() {
        let document = document();
        let iframe: HtmlIFrameElement = document
            .create_element("iframe")
            .unwrap_throw()
            .unchecked_into();
        document
            .body()
            .unwrap_throw()
            .append_child(&iframe)
            .unwrap_throw();

        let frame_window = iframe.content_window().unwrap_throw();
        let frame_document = frame_window.document().unwrap_throw();
        let element = frame_document.create_element("div").unwrap_throw();
        let node: &Node = element.as_ref();

        let resolved_window = owner_window(Some(node));
        assert!(same_object(resolved_window.as_ref(), frame_window.as_ref()));
        assert!(!same_object(
            resolved_window.as_ref(),
            web_sys::window().unwrap_throw().as_ref()
        ));

        let resolved_document = owner_document(Some(node));
        assert!(same_object(
            resolved_document.as_ref(),
            frame_document.as_ref()
        ));

        iframe.remove();
    }
}
