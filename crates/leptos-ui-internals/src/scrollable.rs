//! Port of `packages/react/src/utils/scrollable.ts` — the scrollability predicates the
//! swipe-to-dismiss gesture gating (and ScrollArea/Select scrolling) walk the ancestor
//! chain with (implementation.md, "Downstream consumers": "Drawer (`useSwipeDismiss`,
//! `scrollable`, ...)", "ScrollArea and Select scrolling").
//!
//! No test in the unit targets this module directly (implementation.md, "Submodules
//! with no test anywhere": "`scrollable.ts` — `allowOverflowIntent` mode,
//! `hasScrollableAncestor`, and shadow-boundary traversal via `getParentNode`: no
//! direct test; exercised only indirectly through `useSwipeDismiss` scroll-gating
//! tests and Drawer/Select component tests"), so the ported behavior is pinned by
//! wasm browser tests here.
//!
//! The shadow-boundary note carried in the source comments (`scrollable.ts:46-47`,
//! `:66-67`) overstates the walk: `getParentNode` does cross shadow boundaries and
//! slots, but a shadow child's `parentNode` is the `ShadowRoot` itself, and
//! `isHTMLElement(ShadowRoot)` ends the walk *before* the host is reached — so a
//! light-DOM scrollable ancestor above the shadow host is not found. The walk does
//! reach same-shadow-tree ancestors. See `ralph/logs/spec-discrepancies.md`.

use floating_ui_dom::dom::{get_computed_style, get_parent_node, is_last_traversable_node};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlElement, Node};

/// The `ScrollAxis` union (`scrollable.ts:8`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Horizontal,
    Vertical,
}

fn computed_overflow(element: &HtmlElement, property: &str) -> Option<String> {
    Some(
        get_computed_style(element.as_ref() as &web_sys::Element)
            .get_property_value(property)
            .ok()?,
    )
}

/// Port of `isScrollableY` (`scrollable.ts:10-21`): the computed `overflow-y` must be
/// `auto`/`scroll`, and the element must actually overflow that axis — or, with
/// `allowOverflowIntent`, merely have layout size on it (a container that overflows
/// only once extra space is added, e.g. drawer keyboard scroll slack, still counts).
pub fn is_scrollable_y(element: &HtmlElement, allow_overflow_intent: bool) -> bool {
    let Some(overflow_y) = computed_overflow(element, "overflow-y") else {
        return false;
    };
    if overflow_y != "auto" && overflow_y != "scroll" {
        return false;
    }
    if allow_overflow_intent {
        element.client_height() > 0
    } else {
        element.scroll_height() > element.client_height()
    }
}

/// Port of `isScrollableX` (`scrollable.ts:23-29`) — the horizontal twin of
/// [`is_scrollable_y`].
pub fn is_scrollable_x(element: &HtmlElement, allow_overflow_intent: bool) -> bool {
    let Some(overflow_x) = computed_overflow(element, "overflow-x") else {
        return false;
    };
    if overflow_x != "auto" && overflow_x != "scroll" {
        return false;
    }
    if allow_overflow_intent {
        element.client_width() > 0
    } else {
        element.scroll_width() > element.client_width()
    }
}

/// Port of `isScrollable` (`scrollable.ts:31-39`).
pub fn is_scrollable(element: &HtmlElement, axis: ScrollAxis, allow_overflow_intent: bool) -> bool {
    match axis {
        ScrollAxis::Vertical => is_scrollable_y(element, allow_overflow_intent),
        ScrollAxis::Horizontal => is_scrollable_x(element, allow_overflow_intent),
    }
}

fn is_html_element(node: &Node) -> bool {
    node.dyn_ref::<HtmlElement>().is_some()
}

/// Port of `hasScrollableAncestor` (`scrollable.ts:41-58`): `true` when any node
/// between `target` (inclusive) and `root` (exclusive) is scrollable on any of
/// `axes`. `root` itself is never tested.
pub fn has_scrollable_ancestor(
    target: &HtmlElement,
    root: &HtmlElement,
    axes: &[ScrollAxis],
) -> bool {
    let root_node: &Node = root;
    let mut node: Option<Node> = Some(target.clone().dyn_into().unwrap());
    while let Some(current) = node {
        if !is_html_element(current.as_ref())
            || current == *root_node
            || is_last_traversable_node(current.as_ref())
        {
            break;
        }
        let element: HtmlElement = current.clone().dyn_into().unwrap();
        for axis in axes {
            if is_scrollable(&element, *axis, false) {
                return true;
            }
        }
        node = Some(get_parent_node(element.as_ref() as &Node));
    }
    false
}

/// Port of `findScrollableTouchTarget` (`scrollable.ts:60-77`): the first scrollable
/// element between `target` (inclusive) and `root` (exclusive), or `root` itself when
/// it is scrollable — `null` otherwise. A target that is not an `HTMLElement` starts
/// the walk from nothing (`scrollable.ts:68`).
pub fn find_scrollable_touch_target(
    target: Option<&EventTarget>,
    root: &HtmlElement,
    axis: ScrollAxis,
    allow_overflow_intent: bool,
) -> Option<HtmlElement> {
    let root_node: &Node = root;
    let mut node: Option<Node> = target
        .and_then(|t| t.dyn_ref::<HtmlElement>().cloned())
        .map(|element| element.dyn_into().unwrap());
    while let Some(current) = node {
        if !is_html_element(current.as_ref())
            || current == *root_node
            || is_last_traversable_node(current.as_ref())
        {
            break;
        }
        let element: HtmlElement = current.clone().dyn_into().unwrap();
        if is_scrollable(&element, axis, allow_overflow_intent) {
            return Some(element);
        }
        node = Some(get_parent_node(element.as_ref() as &Node));
    }

    is_scrollable(root, axis, allow_overflow_intent).then(|| root.clone())
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use web_sys::{HtmlElement, ShadowRootInit, ShadowRootMode};

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn div(style: &str) -> HtmlElement {
        let element: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        element.style().set_css_text(style);
        element
    }

    fn append(element: &HtmlElement) {
        document().body().unwrap().append_child(element).unwrap();
    }

    fn remove(element: &HtmlElement) {
        document().body().unwrap().remove_child(element).unwrap();
    }

    // The `auto`/`scroll` gate plus the overflow requirement
    // (`scrollable.ts:10-21`): an `auto` box with overflowing content is scrollable,
    // one without is not, and `visible` never is.
    #[wasm_bindgen_test]
    fn detects_vertical_scrollability() {
        let scrollable = div("position: absolute; overflow-y: auto; height: 50px; width: 50px;");
        let inner = div("height: 200px;");
        scrollable.append_child(&inner).unwrap();
        append(&scrollable);

        assert!(is_scrollable_y(&scrollable, false));
        assert!(is_scrollable(&scrollable, ScrollAxis::Vertical, false));

        let flat = div("position: absolute; overflow-y: auto; height: 50px; width: 50px;");
        append(&flat);
        assert!(!is_scrollable_y(&flat, false));
        remove(&flat);

        let visible = div("position: absolute; overflow-y: visible; height: 50px; width: 50px;");
        let tall = div("height: 200px;");
        visible.append_child(&tall).unwrap();
        append(&visible);
        assert!(!is_scrollable_y(&visible, false));
        remove(&visible);

        remove(&scrollable);
    }

    // The `allowOverflowIntent` mode (`scrollable.ts:15-20`): an `auto` box without
    // overflow still counts while it has layout size, and a zero-sized one does not.
    #[wasm_bindgen_test]
    fn allow_overflow_intent_counts_layout_size_instead_of_overflow() {
        let flat = div("position: absolute; overflow-y: auto; height: 50px; width: 50px;");
        append(&flat);

        assert!(is_scrollable_y(&flat, true));

        flat.style().set_property("display", "none").unwrap();
        assert!(!is_scrollable_y(&flat, true));

        remove(&flat);
    }

    // The ancestor walk (`scrollable.ts:41-58`): a scrollable ancestor between the
    // target and the root counts, the root itself never does.
    #[wasm_bindgen_test]
    fn finds_scrollable_ancestors_between_target_and_root() {
        let root = div("position: absolute; width: 100px; height: 100px;");
        let scrollable = div("overflow-y: auto; height: 50px; width: 50px;");
        let tall = div("height: 200px;");
        let target = div("width: 10px; height: 10px;");
        scrollable.append_child(&tall).unwrap();
        scrollable.append_child(&target).unwrap();
        root.append_child(&scrollable).unwrap();
        append(&root);

        assert!(has_scrollable_ancestor(
            &target,
            &root,
            &[ScrollAxis::Vertical]
        ));

        // The root itself is excluded from the walk.
        assert!(!has_scrollable_ancestor(
            &root,
            &root,
            &[ScrollAxis::Vertical]
        ));

        remove(&root);
    }

    // `findScrollableTouchTarget` (`scrollable.ts:60-77`): the scrollable ancestor,
    // then the root fallback, then null.
    #[wasm_bindgen_test]
    fn finds_the_first_scrollable_touch_target() {
        let root = div("position: absolute; width: 100px; height: 100px;");
        let scrollable = div("overflow-y: auto; height: 50px; width: 50px;");
        let tall = div("height: 200px;");
        let target = div("width: 10px; height: 10px;");
        scrollable.append_child(&tall).unwrap();
        scrollable.append_child(&target).unwrap();
        root.append_child(&scrollable).unwrap();
        append(&root);

        let target_event: &EventTarget = target.as_ref();
        assert_eq!(
            find_scrollable_touch_target(Some(target_event), &root, ScrollAxis::Vertical, false)
                .as_ref(),
            Some(&scrollable)
        );

        // The root fallback: no scrollable inside, but the root itself is one.
        let scrollable_root =
            div("position: absolute; overflow-y: auto; width: 100px; height: 50px;");
        let tall_root = div("height: 300px;");
        let plain_target = div("width: 10px; height: 10px;");
        scrollable_root.append_child(&tall_root).unwrap();
        scrollable_root.append_child(&plain_target).unwrap();
        append(&scrollable_root);

        let plain_event: &EventTarget = plain_target.as_ref();
        assert_eq!(
            find_scrollable_touch_target(
                Some(plain_event),
                &scrollable_root,
                ScrollAxis::Vertical,
                false
            )
            .as_ref(),
            Some(&scrollable_root)
        );

        remove(&root);
        remove(&scrollable_root);
    }

    // The shadow-boundary behavior, pinned against the walk's REAL semantics (see the
    // spec-discrepancies log): a scrollable ancestor inside the *same* shadow tree is
    // found (`scrollable.ts:49-55`), while the walk stops at the shadow root itself —
    // a light-DOM scrollable ancestor above the host is never reached, because
    // `getParentNode` lands on the `ShadowRoot` (a shadow child's `parentNode`) and
    // `isHTMLElement(ShadowRoot)` ends the loop (`:49`).
    #[wasm_bindgen_test]
    fn shadow_tree_walk_reaches_same_tree_ancestors_but_stops_at_the_shadow_root() {
        let light_scrollable =
            div("position: absolute; overflow-y: auto; height: 50px; width: 50px;");
        let light_tall = div("height: 200px;");
        light_scrollable.append_child(&light_tall).unwrap();

        let host = div("width: 20px; height: 20px;");
        light_scrollable.append_child(&host).unwrap();
        append(&light_scrollable);

        let shadow = host
            .attach_shadow(&ShadowRootInit::new(ShadowRootMode::Open))
            .unwrap();

        // (a) Same-tree ancestor: shadow > scrollable > target with the walk's root
        // above the scrollable — the in-tree scrollable is found.
        let tree_root: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        let tree_scrollable = div("overflow-y: auto; height: 50px; width: 50px;");
        let tree_tall = div("height: 200px;");
        let tree_target = div("width: 10px; height: 10px;");
        tree_scrollable.append_child(&tree_tall).unwrap();
        tree_scrollable.append_child(&tree_target).unwrap();
        tree_root.append_child(&tree_scrollable).unwrap();
        shadow.append_child(&tree_root).unwrap();

        assert!(has_scrollable_ancestor(
            &tree_target,
            &tree_root,
            &[ScrollAxis::Vertical]
        ));

        // (b) A light-DOM scrollable ancestor above the host is NOT found from shadow
        // content — the walk stops at the shadow root.
        let shadow_only_target: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        shadow.append_child(&shadow_only_target).unwrap();
        assert!(!has_scrollable_ancestor(
            &shadow_only_target,
            &light_scrollable,
            &[ScrollAxis::Vertical]
        ));

        remove(&light_scrollable);
    }
}
