//! Port of `packages/react/src/floating-ui-react/utils/tabbable.ts` — the
//! focusability classifier the FocusManager/Portal pipeline rides
//! (`specs/library/floating-ui-react/implementation.md`, "DOM/portal strategy and
//! why": non-modal tabbability guards swap portal content via
//! `disableFocusInside`/`enableFocusInside`, a `data-tabindex` mirror
//! (`tabbable.ts:262-282`), and `FocusGuard` spans route Tab to the
//! next/previous document tabbable). The inclusion/exclusion rules the suite
//! proves are `specs/library/floating-ui-react/parts/safePolygon-utils.md:57-91`,
//! and the `aria-disabled` posture (focusable-but-semantically-disabled) is
//! `specs/library/floating-ui-react/parts/safePolygon-utils.md:107-110`.
//!
//! [`disable_focus_inside`]/[`enable_focus_inside`] are the
//! `data-tabindex`-mirror tabbability swap the portal applies to its own content
//! (`implementation.md`, "DOM/portal strategy and why": non-modal tabbability
//! guards), and [`get_next_tabbable`]/[`get_previous_tabbable`] are the
//! FocusGuard routing primitives.
//!
//! ## Rust adaptations
//!
//! - The `FocusableElement = HTMLElement | SVGElement` union (`tabbable.ts:6`)
//!   becomes [`web_sys::Element`] — web-sys types don't model TS unions, and every
//!   operation below is defined on the shared `Element` surface.
//! - `element.tabIndex` (`tabbable.ts:98`) is the `HTMLOrSVGElement` mixin
//!   property, so a plain `HtmlElement` binding would miss `svg[tabindex]`
//!   candidates. The port reads it through `js_sys::Reflect`, which covers both
//!   mixin implementors and preserves the JS `undefined` semantics for anything
//!   else: a non-numeric read maps to `-1`, failing the `>= 0` checks exactly
//!   like `undefined >= 0` does.
//! - `(container as HTMLSlotElement).assignedElements({ flatten: true })`
//!   (`tabbable.ts:147-151`) — web-sys 0.3.105 binds only `assignedNodes` +
//!   `AssignedNodesOptions`, so the port filters the flattened node list to
//!   elements, the spec definition of `assignedElements`.
//! - `container.shadowRoot.children` (`tabbable.ts:154-156`) — web-sys binds no
//!   `children` on `ShadowRoot` (a `ParentNode` member), so the port reads
//!   `Node::child_nodes()` filtered to elements: the same element-only,
//!   tree-ordered set.
//! - `isShadowRoot(rootNode)` (`tabbable.ts:26`) is not re-exported by the
//!   `floating-ui-dom` `dom` module the port binds (`get_node_name`,
//!   `is_html_element`, `get_computed_style` are); the root node is detected with
//!   `dyn_ref::<ShadowRoot>` instead — the same truth table (the `markOthers`
//!   `unwrapHost` precedent in [`crate::floating_ui::mark_others`]).
//! - `isElementVisible(element, styles)` (`tabbable.ts:91`) — the port's
//!   [`crate::floating_ui::composite::is_element_visible`] folded its optional
//!   computed-styles parameter into the function body, so the element itself
//!   re-resolves the styles it already fetched here; the ancestor arm still uses
//!   the single fetch (`styles.display !== 'none'`, `tabbable.ts:94`).
//! - `element.matches(':disabled')` (`tabbable.ts:60`) can only reject for an
//!   invalid selector and `:disabled` is valid — a hypothetical rejection maps to
//!   `false` rather than propagating a `JsValue` error (the `composite.ts:496`
//!   precedent).
//! - `isOutsideEvent(event: FocusEvent | React.FocusEvent)` (`tabbable.ts:256`)
//!   keeps only the native arm — there is no React synthetic event realm here —
//!   and `event.currentTarget` resolves through `dyn_into::<Element>`.

use floating_ui_dom::dom::{get_computed_style, get_node_name, is_html_element};
use wasm_bindgen::JsValue;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlDetailsElement, HtmlInputElement, ShadowRoot};

use leptos_ui_utils::owner_document;

use crate::floating_ui::composite::is_element_visible;
use crate::floating_ui::element::{active_element, contains};

/// The `CANDIDATE_SELECTOR` constant (`tabbable.ts:8-9`) — the tag/attribute
/// filter every candidate must match before the structural and visibility checks.
const CANDIDATE_SELECTOR: &str = "a[href],button,input,select,textarea,summary,details,iframe,object,embed,[tabindex],[contenteditable]:not([contenteditable=\"false\"]),audio[controls],video[controls]";

/// The JS `===` identity `indexOf`/`find` compare element references by
/// (`tabbable.ts:131-143,206,237`) — web-sys wrapper equality is not structural,
/// and the underlying `JsValue` strict equality is object identity.
fn same_value<T: AsRef<JsValue>, U: AsRef<JsValue>>(a: &T, b: &U) -> bool {
    a.as_ref() == b.as_ref()
}

/// The `getParentElement` helper (`tabbable.ts:11-27`): the assigned slot wins,
/// then the light-DOM parent, then a shadow root's host — the composed tree walk
/// `isFocusableElement`'s ancestor loop rides.
fn get_parent_element(element: &Element) -> Option<Element> {
    if let Some(assigned_slot) = element.assigned_slot() {
        return Some(assigned_slot.into());
    }

    if let Some(parent_element) = element.parent_element() {
        return Some(parent_element);
    }

    element
        .get_root_node()
        .dyn_ref::<ShadowRoot>()
        .map(|root_node| root_node.host())
}

/// The `getDetailsSummary` helper (`tabbable.ts:29-37`): the first `<summary>`
/// element child, or `None`.
fn get_details_summary(details: &Element) -> Option<Element> {
    let children = details.children();
    for index in 0..children.length() {
        let child = children.item(index)?;
        if get_node_name((&child).into()) == "summary" {
            return Some(child);
        }
    }

    None
}

/// The `isWithinOpenDetailsSummary` helper (`tabbable.ts:39-42`): whether
/// `element` is the summary itself or inside it — the content a closed
/// `<details>` still shows.
fn is_within_open_details_summary(element: &Element, details: &Element) -> bool {
    get_details_summary(details).is_some_and(|summary| {
        same_value(element, &summary) || contains(Some(&summary), Some(element))
    })
}

/// The `<details>.open` IDL read (`tabbable.ts:75` — the unchecked
/// `HTMLDetailsElement` cast is only reached for nodeName `'details'`, where it
/// always succeeds).
fn is_open_details(element: &Element) -> bool {
    element
        .dyn_ref::<HtmlDetailsElement>()
        .map(|details| details.open())
        .unwrap_or(false)
}

/// The `input.type` IDL read (`tabbable.ts:55` — the unchecked
/// `HTMLInputElement` cast is only reached for nodeName `'input'`).
fn is_hidden_input(element: &Element) -> bool {
    element
        .dyn_ref::<HtmlInputElement>()
        .map(|input| input.type_() == "hidden")
        .unwrap_or(false)
}

/// The `isFocusableCandidate` helper (`tabbable.ts:44-57`): the selector filter
/// plus the three structural exemptions — a `<summary>` only counts as the first
/// summary of its `<details>` parent, a `<details>` with a summary is not itself a
/// candidate, and `input[type=hidden]` is out.
fn is_focusable_candidate(element: &Element) -> bool {
    if !element.matches(CANDIDATE_SELECTOR).unwrap_or(false) {
        return false;
    }

    let node_name = get_node_name(element.into());

    if node_name == "summary" {
        let is_first_summary_of_details = element
            .parent_element()
            .map(|parent| {
                get_node_name((&parent).into()) == "details"
                    && get_details_summary(&parent)
                        .is_some_and(|summary| same_value(element, &summary))
            })
            .unwrap_or(false);
        if !is_first_summary_of_details {
            return false;
        }
    }

    if node_name == "details" && get_details_summary(element).is_some() {
        return false;
    }

    if node_name == "input" && is_hidden_input(element) {
        return false;
    }

    true
}

/// The `isVisibleInTabbableTree` helper (`tabbable.ts:87-95`): the element itself
/// gets the full effective-visibility check, while ancestors only fail the walk on
/// `display: none` — `visibility` inheritance is the descendant's own computed
/// value, which is how a `visibility: visible` override survives a hidden
/// ancestor.
fn is_visible_in_tabbable_tree(element: &Element, is_ancestor: bool) -> bool {
    let styles = get_computed_style(element);

    if !is_ancestor {
        return is_element_visible(Some(element));
    }

    styles.get_property_value("display").unwrap_or_default() != "none"
}

/// The `isFocusableElement` helper (`tabbable.ts:59-85`): the candidate filter
/// plus the composed-ancestor walk — `inert` at any level, a closed
/// `<details>` (unless the element is its visible summary), the `hidden`
/// attribute, and non-`display: none` visibility all exclude.
fn is_focusable_element(element: &Element) -> bool {
    if !is_focusable_candidate(element) || !element.is_connected() {
        return false;
    }

    if element.matches(":disabled").unwrap_or(false) {
        return false;
    }

    // `for (let current = element; current; current = getParentElement(current))`
    let mut current: Option<Element> = Some(element.clone());
    while let Some(node) = current {
        let is_ancestor = !same_value(&node, element);
        let is_slot = get_node_name((&node).into()) == "slot";

        if node.has_attribute("inert") {
            return false;
        }

        let closed_details_excludes = is_ancestor
            && get_node_name((&node).into()) == "details"
            && !is_open_details(&node)
            && !is_within_open_details_summary(element, &node);

        if closed_details_excludes
            || node.has_attribute("hidden")
            || (!is_slot && !is_visible_in_tabbable_tree(&node, is_ancestor))
        {
            return false;
        }

        current = get_parent_element(&node);
    }

    true
}

/// The `getTabIndex` helper (`tabbable.ts:97-112`): the reflected `tabIndex`
/// property, with the naturally-tabbable `details`/`audio`/`video`/
/// `contenteditable` elements promoted from `-1` to `0`.
fn get_tab_index(element: &Element) -> i32 {
    // `element.tabIndex` — read through Reflect (see the module docs): a
    // non-numeric read is JS `undefined`, which fails `undefined < 0` upstream and
    // every `>= 0` check below, exactly like the `-1` fallback.
    let tab_index = js_sys::Reflect::get(element.as_ref(), &"tabIndex".into())
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(-1.0) as i32;

    if tab_index < 0 {
        let node_name = get_node_name(element.into());
        let is_content_editable = element
            .dyn_ref::<web_sys::HtmlElement>()
            .map(|html_element| html_element.is_content_editable())
            .unwrap_or(false);
        if node_name == "details"
            || node_name == "audio"
            || node_name == "video"
            || is_content_editable
        {
            return 0;
        }
    }

    tab_index
}

/// The `getNamedRadioInput` helper (`tabbable.ts:114-121`): the element as a
/// named radio input, or `None` for every other shape.
fn get_named_radio_input(element: &Element) -> Option<HtmlInputElement> {
    if get_node_name(element.into()) != "input" {
        return None;
    }

    let input = element.dyn_ref::<HtmlInputElement>()?;
    (input.type_() == "radio" && !input.name().is_empty()).then(|| input.clone())
}

/// The `radio.form === input.form` identity check (`tabbable.ts:131,141`) —
/// `None === None` is the no-form case, same as the JS `undefined` comparison.
fn same_form(a: Option<web_sys::HtmlFormElement>, b: Option<web_sys::HtmlFormElement>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => same_value(&a, &b),
        _ => false,
    }
}

/// The `isTabbableRadio` helper (`tabbable.ts:123-144`): a named radio is only
/// tabbable when it is the checked one of its group — or the first candidate of
/// the group when none is checked.
fn is_tabbable_radio(element: &Element, candidates: &[Element]) -> bool {
    let Some(input) = get_named_radio_input(element) else {
        return true;
    };

    let in_group = |candidate: &Element| -> Option<HtmlInputElement> {
        get_named_radio_input(candidate)
            .filter(|radio| radio.name() == input.name() && same_form(radio.form(), input.form()))
    };

    let checked_radio = candidates
        .iter()
        .find_map(|candidate| in_group(candidate).filter(|radio| radio.checked()));
    if let Some(checked_radio) = checked_radio {
        return same_value(&checked_radio, &input);
    }

    let first_in_group = candidates.iter().find_map(in_group);
    first_in_group
        .map(|first| same_value(&first, &input))
        .unwrap_or(false)
}

/// The `getComposedChildren` helper (`tabbable.ts:146-159`): assigned elements
/// for a slot, the shadow root's element children for a shadow host, else the
/// plain element children.
fn get_composed_children(container: &Element) -> Vec<Element> {
    if is_html_element(container.as_ref()) && get_node_name(container.into()) == "slot" {
        if let Some(slot) = container.dyn_ref::<web_sys::HtmlSlotElement>() {
            let options = web_sys::AssignedNodesOptions::new();
            options.set_flatten(true);
            let assigned_elements: Vec<Element> = slot
                .assigned_nodes_with_options(&options)
                .iter()
                .filter_map(|node| node.dyn_into::<Element>().ok())
                .collect();
            if !assigned_elements.is_empty() {
                return assigned_elements;
            }
        }
    }

    if let Some(shadow_root) = container.shadow_root() {
        let child_nodes = shadow_root.child_nodes();
        return (0..child_nodes.length())
            .filter_map(|index| child_nodes.get(index))
            .filter_map(|node| node.dyn_into::<Element>().ok())
            .collect();
    }

    let children = container.children();
    (0..children.length())
        .filter_map(|index| children.item(index))
        .collect()
}

/// The `appendCandidates` helper (`tabbable.ts:161-169`): the depth-first
/// composed-tree walk collecting every candidate, parents before children.
fn append_candidates(container: &Element, list: &mut Vec<Element>) {
    for child in get_composed_children(container) {
        if is_focusable_candidate(&child) {
            list.push(child.clone());
        }

        append_candidates(&child, list);
    }
}

/// The `appendMatchingElements` helper (`tabbable.ts:171-179`): the same walk
/// collecting elements matching `selector` (HTML elements only, upstream's
/// `isHTMLElement` guard).
fn append_matching_elements(
    container: &Element,
    selector: &str,
    list: &mut Vec<web_sys::HtmlElement>,
) {
    for child in get_composed_children(container) {
        if let Some(html_element) = child.dyn_ref::<web_sys::HtmlElement>() {
            if html_element.matches(selector).unwrap_or(false) {
                list.push(html_element.clone());
            }
        }

        append_matching_elements(&child, selector, list);
    }
}

/// The `isTabbable` export (`tabbable.ts:181-183`): whether `element` is
/// focusable and carries a non-negative tab index.
pub fn is_tabbable(element: Option<&Element>) -> bool {
    let Some(element) = element else {
        return false;
    };
    is_focusable_element(element) && get_tab_index(element) >= 0
}

/// The `focusable` export (`tabbable.ts:185-189`): every focusable element in
/// `container`'s composed tree, in tree order.
pub fn focusable(container: &Element) -> Vec<Element> {
    let mut candidates: Vec<Element> = Vec::new();
    append_candidates(container, &mut candidates);
    candidates
        .into_iter()
        .filter(is_focusable_element)
        .collect()
}

/// The `tabbable` export (`tabbable.ts:191-196`): every focusable element with a
/// non-negative tab index, with the named-radio-group rule applied.
pub fn tabbable(container: &Element) -> Vec<Element> {
    let candidates = focusable(container);
    candidates
        .iter()
        .filter(|element| get_tab_index(element) >= 0 && is_tabbable_radio(element, &candidates))
        .cloned()
        .collect()
}

/// The `getTabbableIn` helper (`tabbable.ts:198-211`): the tabbable element
/// `dir` steps from the active element — `None` when the container has no
/// tabbables or the step lands out of bounds (the `list[nextIndex]` `undefined`
/// read the public helpers fall back from). `dir` is the `1 | -1` literal.
fn get_tabbable_in(container: &Element, dir: i32) -> Option<Element> {
    let list = tabbable(container);
    let len = list.len();
    if len == 0 {
        return None;
    }

    let active = active_element(&owner_document(Some(container.as_ref())));
    let index = list.iter().position(|element| {
        active
            .as_ref()
            .is_some_and(|active| same_value(active, element))
    });
    // `index === -1 ? (dir === 1 ? 0 : len - 1) : index + dir` — no wraparound;
    // an out-of-bounds step is the caller's fallback arm.
    let next_index = match index {
        None => {
            if dir == 1 {
                0i64
            } else {
                len as i64 - 1
            }
        }
        Some(index) => index as i64 + dir as i64,
    };

    usize::try_from(next_index)
        .ok()
        .and_then(|next_index| list.get(next_index))
        .cloned()
}

/// The `getNextTabbable` export (`tabbable.ts:213-217`): the next tabbable after
/// the active element of `reference_element`'s document, falling back to the
/// reference itself.
pub fn get_next_tabbable(reference_element: Option<&Element>) -> Option<Element> {
    let body = owner_document(reference_element.map(|element| element.as_ref()))
        .body()
        .unwrap_or_else(|| panic!("getNextTabbable: the owner document has no body element"));
    get_tabbable_in(body.as_ref(), 1).or_else(|| reference_element.cloned())
}

/// The `getPreviousTabbable` export (`tabbable.ts:219-224`): the previous
/// tabbable before the active element of `reference_element`'s document, falling
/// back to the reference itself.
pub fn get_previous_tabbable(reference_element: Option<&Element>) -> Option<Element> {
    let body = owner_document(reference_element.map(|element| element.as_ref()))
        .body()
        .unwrap_or_else(|| panic!("getPreviousTabbable: the owner document has no body element"));
    get_tabbable_in(body.as_ref(), -1).or_else(|| reference_element.cloned())
}

/// The `getTabbableNearElement` helper (`tabbable.ts:226-244`): the tabbable
/// element `dir` steps from `reference_element` itself (not the active element),
/// wrapping modulo the list length — `None` when there are no tabbables or the
/// reference is not among them.
fn get_tabbable_near_element(reference_element: Option<&Element>, dir: i32) -> Option<Element> {
    let reference_element = reference_element?;

    let body = owner_document(Some(reference_element.as_ref()))
        .body()
        .unwrap_or_else(|| {
            panic!("getTabbableNearElement: the owner document has no body element")
        });
    let list = tabbable(body.as_ref());
    let element_count = list.len() as i64;
    if element_count == 0 {
        return None;
    }

    let index = list
        .iter()
        .position(|element| same_value(element, &reference_element))?;
    let next_index = (index as i64 + dir as i64 + element_count) % element_count;

    list.get(next_index as usize).cloned()
}

/// The `getTabbableAfterElement` export (`tabbable.ts:246-248`).
pub fn get_tabbable_after_element(reference_element: Option<&Element>) -> Option<Element> {
    get_tabbable_near_element(reference_element, 1)
}

/// The `getTabbableBeforeElement` export (`tabbable.ts:250-254`).
pub fn get_tabbable_before_element(reference_element: Option<&Element>) -> Option<Element> {
    get_tabbable_near_element(reference_element, -1)
}

/// The `isOutsideEvent` export (`tabbable.ts:256-260`): whether a focus event's
/// `relatedTarget` is missing or outside `container` (defaulting to the event's
/// current target).
pub fn is_outside_event(event: &web_sys::FocusEvent, container: Option<&Element>) -> bool {
    let container_element: Option<Element> = container.cloned().or_else(|| {
        event
            .current_target()
            .and_then(|target| target.dyn_into::<Element>().ok())
    });
    let related_target = event
        .related_target()
        .and_then(|target| target.dyn_into::<Element>().ok());

    !related_target.is_some_and(|related| contains(container_element.as_ref(), Some(&related)))
}

/// The `disableFocusInside` export (`tabbable.ts:262-268`): makes every tabbable
/// inside `container` untabbable, mirroring each original `tabindex` (including
/// its absence, as an empty string) into `data-tabindex`.
pub fn disable_focus_inside(container: &Element) {
    for element in tabbable(container) {
        // `element.dataset.tabindex = element.getAttribute('tabindex') || ''`
        let current_tabindex = element.get_attribute("tabindex").unwrap_or_default();
        element
            .set_attribute("data-tabindex", &current_tabindex)
            .ok();
        element.set_attribute("tabindex", "-1").ok();
    }
}

/// The `enableFocusInside` export (`tabbable.ts:270-282`): restores what
/// [`disable_focus_inside`] swapped — each `data-tabindex` mirror is consumed,
/// restoring the recorded `tabindex` or removing the attribute when the mirror
/// was empty (the JS truthiness of `''`).
pub fn enable_focus_inside(container: &Element) {
    let mut elements: Vec<web_sys::HtmlElement> = Vec::new();
    append_matching_elements(container, "[data-tabindex]", &mut elements);
    for element in elements {
        let tabindex = element.get_attribute("data-tabindex");
        element.remove_attribute("data-tabindex").ok();
        match tabindex {
            Some(tabindex) if !tabindex.is_empty() => {
                element.set_attribute("tabindex", &tabindex).ok();
            }
            _ => {
                element.remove_attribute("tabindex").ok();
            }
        }
    }
}

// The focusability rules need a real DOM realm (selector matching, computed
// styles, shadow roots, `checkVisibility`): they mirror
// `packages/react/src/floating-ui-react/utils/tabbable.test.ts`.
//
// Harness note: upstream sweeps the body in `afterEach`
// (`tabbable.test.ts:6-8`), but the wasm-bindgen-test harness renders its own
// output elements into the body — clearing it kills the harness (the
// mark_others port documents the same wall). Fixtures are tracked in a
// [`Fixture`] and detached on drop instead, and the three exhaustive
// `toEqual` assertions (`tabbable.test.ts:18,36,104-109`) scope to a dedicated
// container element rather than `document.body` — the walk semantics are
// identical (no inert/hidden on the container's ancestor chain) and exhaustive
// body-list equality would otherwise be at the mercy of the harness's own DOM.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use leptos_ui_utils::visually_hidden::{VISUALLY_HIDDEN, VISUALLY_HIDDEN_INPUT};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect("window should exist")
            .document()
            .expect("document should exist")
    }

    fn body() -> Element {
        document()
            .body()
            .expect("body should exist")
            .unchecked_into()
    }

    /// Tracks every top-level fixture element a test appends to the body so the
    /// `afterEach` body sweep (`tabbable.test.ts:6-8`) can run as a drop-time
    /// `Element.remove()` per element — removal, not `innerHTML` clearing, keeps
    /// the wasm-bindgen-test harness alive.
    struct Fixture(Vec<Element>);

    impl Fixture {
        fn append(&mut self, element: &Element) {
            body().append_child(element).expect("append should succeed");
            self.0.push(element.clone());
        }

        /// Appends into a fixture container — the child dies with its parent at
        /// drop time, so only top-level elements are tracked.
        fn append_into(&self, parent: &Element, element: &Element) {
            parent.append_child(element).expect("append should succeed");
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for element in self.0.drain(..) {
                element.remove();
            }
        }
    }

    fn create_element(tag: &str) -> Element {
        document()
            .create_element(tag)
            .unwrap_or_else(|_| panic!("{tag} should be creatable"))
    }

    fn create_div() -> Element {
        create_element("div")
    }

    fn create_input() -> HtmlInputElement {
        create_element("input").unchecked_into()
    }

    fn as_html(element: &Element) -> web_sys::HtmlElement {
        element.clone().unchecked_into()
    }

    /// `Object.assign(element.style, visuallyHidden)` /
    /// `Object.assign(element.style, visuallyHiddenInput)`
    /// (`tabbable.test.ts:310`, `:321`) — the style-property loop the
    /// `visuallyHidden` util documents as its application contract.
    fn assign_styles(element: &Element, styles: &[(&str, &str)]) {
        let html_element = as_html(element);
        for (name, value) in styles {
            html_element
                .style()
                .set_property(name, value)
                .unwrap_or_else(|_| panic!("set_property({name}) should succeed"));
        }
    }

    /// `Object.defineProperty(element, 'checkVisibility', { value: () => result })`
    /// (`tabbable.test.ts:139-142`, `:200-203`) — an own property shadowing the
    /// native method. The closure is returned so it outlives the assertions that
    /// read the stub.
    fn stub_check_visibility(element: &Element, result: bool) -> Closure<dyn Fn() -> bool> {
        let closure: Closure<dyn Fn() -> bool> = Closure::wrap(Box::new(move || result));
        js_sys::Reflect::set(
            element.as_ref(),
            &"checkVisibility".into(),
            closure.as_ref(),
        )
        .expect("checkVisibility stub should install");
        closure
    }

    /// `Object.defineProperty(element, 'checkVisibility', { value: undefined })`
    /// (`tabbable.test.ts:214-217`) — the feature-detect must take its manual
    /// fallback arm.
    fn unset_check_visibility(element: &Element) {
        js_sys::Reflect::set(
            element.as_ref(),
            &"checkVisibility".into(),
            &JsValue::UNDEFINED,
        )
        .expect("checkVisibility removal should succeed");
    }

    /// `toEqual([...])` compared by element identity — the JS array equality the
    /// upstream assertions rely on.
    fn assert_same_elements(actual: &[Element], expected: &[Element]) {
        let actual_refs: Vec<&JsValue> = actual.iter().map(|element| element.as_ref()).collect();
        let expected_refs: Vec<&JsValue> =
            expected.iter().map(|element| element.as_ref()).collect();
        assert_eq!(actual_refs, expected_refs);
    }

    #[wasm_bindgen_test]
    fn includes_basic_tabbable_controls_and_excludes_hidden_inputs() {
        let mut fixture = Fixture(Vec::new());
        let container = create_div();
        fixture.append(&container);

        let button = create_element("button");
        let input = create_element("input");
        let hidden_input = create_input();

        hidden_input.set_type("hidden");
        fixture.append_into(&container, &button);
        fixture.append_into(&container, &input);
        fixture.append_into(&container, &hidden_input);

        assert_same_elements(&tabbable(&container), &[button, input]);
    }

    #[wasm_bindgen_test]
    fn includes_embedded_focusable_elements_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let iframe = create_element("iframe");

        fixture.append(&iframe);

        assert!(tabbable(&body()).iter().any(|e| same_value(e, &iframe)));
    }

    #[wasm_bindgen_test]
    fn excludes_disabled_controls_from_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let container = create_div();
        fixture.append(&container);

        let enabled_button = create_element("button");
        let disabled_button = create_element("button");

        // `disabledButton.disabled = true` (`tabbable.test.ts:33`) — the IDL
        // property reflects the attribute for a fresh element.
        disabled_button
            .set_attribute("disabled", "")
            .expect("set should succeed");
        fixture.append_into(&container, &enabled_button);
        fixture.append_into(&container, &disabled_button);

        assert_same_elements(&tabbable(&container), &[enabled_button]);
    }

    #[wasm_bindgen_test]
    fn includes_slotted_light_dom_elements_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let host = create_div();
        fixture.append(&host);

        let shadow_root = host
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .expect("shadow root should attach");
        let slot = create_element("slot");
        let button = create_element("button");

        shadow_root
            .append_child(&slot)
            .expect("append should succeed");
        host.append_child(&button).expect("append should succeed");

        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn excludes_unslotted_light_dom_elements_from_a_shadow_host_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let host = create_div();
        fixture.append(&host);

        let shadow_root = host
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .expect("shadow root should attach");
        let shadow_button = create_element("button");
        let hidden_light_button = create_element("button");

        shadow_root
            .append_child(&shadow_button)
            .expect("append should succeed");
        host.append_child(&hidden_light_button)
            .expect("append should succeed");

        let tabbable_elements = tabbable(&body());

        assert!(
            tabbable_elements
                .iter()
                .any(|e| same_value(e, &shadow_button)),
            "the shadow button should be tabbable"
        );
        assert!(
            !tabbable_elements
                .iter()
                .any(|e| same_value(e, &hidden_light_button)),
            "the unslotted light button should not be tabbable"
        );
    }

    #[wasm_bindgen_test]
    fn keeps_the_summary_tabbable_but_excludes_closed_details_content() {
        let mut fixture = Fixture(Vec::new());
        let details = create_element("details");
        let summary = create_element("summary");
        let button = create_element("button");

        summary.set_text_content(Some("Summary"));
        button.set_text_content(Some("Hidden"));
        details
            .append_child(&summary)
            .expect("append should succeed");
        details
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&details);

        let tabbable_elements = tabbable(&body());

        assert!(tabbable_elements.iter().any(|e| same_value(e, &summary)));
        assert!(!tabbable_elements.iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_only_the_first_summary_tabbable_and_includes_details_without_a_summary() {
        let mut fixture = Fixture(Vec::new());
        let container = create_div();
        fixture.append(&container);

        let closed_details = create_element("details");
        let closed_summary = create_element("summary");
        let hidden_button = create_element("button");
        let open_details = create_element("details");
        let open_summary = create_element("summary");
        let ignored_summary = create_element("summary");
        let visible_button = create_element("button");
        let summaryless_details = create_element("details");

        // `openDetails.open = true` (`tabbable.test.ts:92`).
        open_details
            .clone()
            .unchecked_into::<HtmlDetailsElement>()
            .set_open(true);

        closed_summary.set_text_content(Some("closed"));
        open_summary.set_text_content(Some("open"));
        ignored_summary.set_text_content(Some("ignored"));

        closed_details
            .append_child(&closed_summary)
            .expect("append should succeed");
        closed_details
            .append_child(&hidden_button)
            .expect("append should succeed");
        open_details
            .append_child(&open_summary)
            .expect("append should succeed");
        open_details
            .append_child(&ignored_summary)
            .expect("append should succeed");
        open_details
            .append_child(&visible_button)
            .expect("append should succeed");
        summaryless_details.set_text_content(Some("summaryless"));

        fixture.append_into(&container, &closed_details);
        fixture.append_into(&container, &open_details);
        fixture.append_into(&container, &summaryless_details);

        assert_same_elements(
            &tabbable(&container),
            &[
                closed_summary,
                open_summary,
                visible_button,
                summaryless_details,
            ],
        );
    }

    #[wasm_bindgen_test]
    fn keeps_aria_disabled_elements_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let element = create_div();

        element
            .set_attribute("tabindex", "0")
            .expect("set should succeed");
        element
            .set_attribute("aria-disabled", "true")
            .expect("set should succeed");
        fixture.append(&element);

        assert!(is_tabbable(Some(&element)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &element)));
    }

    #[wasm_bindgen_test]
    fn excludes_elements_hidden_with_css_visibility_from_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let button = create_element("button");

        as_html(&button)
            .style()
            .set_property("visibility", "hidden")
            .expect("set should succeed");
        fixture.append(&button);

        assert!(!is_tabbable(Some(&button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_descendants_of_display_contents_ancestors_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let dialog = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("display", "contents")
            .expect("set should succeed");
        // The jsdom-determinism stub (`tabbable.test.ts:139-142`) — inert in
        // Chromium, where the ancestor walk only consults `display`.
        let _stub = stub_check_visibility(&wrapper, false);
        dialog
            .set_attribute("role", "dialog")
            .expect("set should succeed");
        dialog.append_child(&button).expect("append should succeed");
        wrapper
            .append_child(&dialog)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_visible_descendants_of_display_contents_ancestors_in_the_tab_order_in_chromium() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("display", "contents")
            .expect("set should succeed");
        wrapper
            .set_attribute("tabindex", "0")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(!is_tabbable(Some(&wrapper)));
        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &wrapper)));
    }

    #[wasm_bindgen_test]
    fn excludes_descendants_of_hidden_display_contents_ancestors_from_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("display", "contents")
            .expect("set should succeed");
        as_html(&wrapper)
            .style()
            .set_property("visibility", "hidden")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(!is_tabbable(Some(&button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_descendants_that_override_ancestor_visibility_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("visibility", "hidden")
            .expect("set should succeed");
        as_html(&button)
            .style()
            .set_property("visibility", "visible")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn excludes_display_contents_candidates_when_checkvisibility_reports_them_hidden() {
        let mut fixture = Fixture(Vec::new());
        let button = create_element("button");

        as_html(&button)
            .style()
            .set_property("display", "contents")
            .expect("set should succeed");
        let _stub = stub_check_visibility(&button, false);
        fixture.append(&button);

        assert!(!is_tabbable(Some(&button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn excludes_display_contents_candidates_when_checkvisibility_is_unavailable() {
        let mut fixture = Fixture(Vec::new());
        let button = create_element("button");

        as_html(&button)
            .style()
            .set_property("display", "contents")
            .expect("set should succeed");
        unset_check_visibility(&button);
        fixture.append(&button);

        assert!(!is_tabbable(Some(&button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn excludes_descendants_of_display_none_ancestors_from_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("display", "none")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(!is_tabbable(Some(&button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn excludes_descendants_of_block_content_visibility_hidden_ancestors_from_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("content-visibility", "hidden")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(!is_tabbable(Some(&button)));
        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_descendants_of_display_contents_content_visibility_hidden_ancestors_in_the_tab_order()
    {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("display", "contents")
            .expect("set should succeed");
        as_html(&wrapper)
            .style()
            .set_property("content-visibility", "hidden")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_descendants_of_inline_content_visibility_hidden_ancestors_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let wrapper = create_div();
        let button = create_element("button");

        as_html(&wrapper)
            .style()
            .set_property("display", "inline")
            .expect("set should succeed");
        as_html(&wrapper)
            .style()
            .set_property("content-visibility", "hidden")
            .expect("set should succeed");
        wrapper
            .append_child(&button)
            .expect("append should succeed");
        fixture.append(&wrapper);

        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_content_visibility_hidden_candidates_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let button = create_element("button");

        as_html(&button)
            .style()
            .set_property("content-visibility", "hidden")
            .expect("set should succeed");
        fixture.append(&button);

        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_zero_size_elements_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let element = create_div();

        element
            .set_attribute("tabindex", "0")
            .expect("set should succeed");
        let style = as_html(&element).style();
        for (name, value) in [
            ("width", "0"),
            ("height", "0"),
            ("padding", "0"),
            ("border", "0"),
        ] {
            style
                .set_property(name, value)
                .unwrap_or_else(|_| panic!("set_property({name}) should succeed"));
        }
        fixture.append(&element);

        assert!(is_tabbable(Some(&element)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &element)));
    }

    #[wasm_bindgen_test]
    fn keeps_visually_hidden_elements_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let button = create_element("button");

        assign_styles(&button, VISUALLY_HIDDEN);
        fixture.append(&button);

        assert!(is_tabbable(Some(&button)));
        assert!(tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_visually_hidden_input_elements_in_the_tab_order() {
        let mut fixture = Fixture(Vec::new());
        let input = create_input();

        input.set_type("checkbox");
        let input_element: Element = input.unchecked_into();
        assign_styles(&input_element, VISUALLY_HIDDEN_INPUT);
        fixture.append(&input_element);

        assert!(is_tabbable(Some(&input_element)));
        assert!(
            tabbable(&body())
                .iter()
                .any(|e| same_value(e, &input_element))
        );
    }

    #[wasm_bindgen_test]
    fn keeps_only_the_checked_radio_in_a_named_group() {
        let mut fixture = Fixture(Vec::new());
        let first_radio = create_input();
        let checked_radio = create_input();
        let button = create_element("button");

        first_radio.set_type("radio");
        first_radio.set_name("group");
        checked_radio.set_type("radio");
        checked_radio.set_name("group");
        checked_radio.set_checked(true);

        fixture.append(first_radio.as_ref());
        fixture.append(checked_radio.as_ref());
        fixture.append(&button);

        let tabbable_elements = tabbable(&body());

        assert!(
            !tabbable_elements
                .iter()
                .any(|e| same_value(e, first_radio.unchecked_ref::<Element>()))
        );
        assert!(
            tabbable_elements
                .iter()
                .any(|e| same_value(e, checked_radio.unchecked_ref::<Element>()))
        );
        assert!(tabbable_elements.iter().any(|e| same_value(e, &button)));
    }

    #[wasm_bindgen_test]
    fn keeps_only_the_first_radio_when_a_named_group_has_no_checked_item() {
        let mut fixture = Fixture(Vec::new());
        let first_radio = create_input();
        let second_radio = create_input();

        first_radio.set_type("radio");
        first_radio.set_name("group");
        second_radio.set_type("radio");
        second_radio.set_name("group");

        fixture.append(first_radio.as_ref());
        fixture.append(second_radio.as_ref());

        let tabbable_elements = tabbable(&body());

        assert!(
            tabbable_elements
                .iter()
                .any(|e| same_value(e, first_radio.unchecked_ref::<Element>()))
        );
        assert!(
            !tabbable_elements
                .iter()
                .any(|e| same_value(e, second_radio.unchecked_ref::<Element>()))
        );
    }

    #[wasm_bindgen_test]
    fn treats_slotted_elements_inside_inert_shadow_content_as_untabbable() {
        let mut fixture = Fixture(Vec::new());
        let host = create_div();
        fixture.append(&host);

        let shadow_root = host
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .expect("shadow root should attach");
        let wrapper = create_div();
        let slot = create_element("slot");
        let button = create_element("button");

        wrapper
            .set_attribute("inert", "")
            .expect("set should succeed");
        wrapper.append_child(&slot).expect("append should succeed");
        shadow_root
            .append_child(&wrapper)
            .expect("append should succeed");
        host.append_child(&button).expect("append should succeed");

        assert!(!tabbable(&body()).iter().any(|e| same_value(e, &button)));
    }

    // --- Port-owned pins: the exports upstream's suite leaves to the
    // FocusManager/Portal tests (implementation.md, "Anything in source not
    // explained by any test" — the tabbable exports beyond `isTabbable`/`tabbable`
    // have no coverage in this unit's 18 files). Minimal mechanics only; the
    // upstream-mirrored suite above carries the behavioral weight.

    #[wasm_bindgen_test]
    fn disable_focus_inside_mirrors_tabindex_and_enable_focus_inside_restores_it() {
        let mut fixture = Fixture(Vec::new());
        let container = create_div();
        fixture.append(&container);

        let with_tabindex = create_element("button");
        with_tabindex
            .set_attribute("tabindex", "0")
            .expect("set should succeed");
        let without_tabindex = create_element("button");
        fixture.append_into(&container, &with_tabindex);
        fixture.append_into(&container, &without_tabindex);

        disable_focus_inside(&container);

        assert_eq!(
            with_tabindex.get_attribute("data-tabindex"),
            Some("0".to_owned())
        );
        assert_eq!(
            with_tabindex.get_attribute("tabindex"),
            Some("-1".to_owned())
        );
        assert_eq!(
            without_tabindex.get_attribute("data-tabindex"),
            Some(String::new()),
            "the absent tabindex mirrors as the empty string"
        );
        assert_eq!(
            without_tabindex.get_attribute("tabindex"),
            Some("-1".to_owned())
        );

        enable_focus_inside(&container);

        assert_eq!(with_tabindex.get_attribute("data-tabindex"), None);
        assert_eq!(
            with_tabindex.get_attribute("tabindex"),
            Some("0".to_owned())
        );
        assert_eq!(without_tabindex.get_attribute("data-tabindex"), None);
        assert_eq!(
            without_tabindex.get_attribute("tabindex"),
            None,
            "an empty mirror removes the tabindex attribute"
        );
    }

    #[wasm_bindgen_test]
    fn get_next_and_previous_tabbable_step_from_the_active_element() {
        let mut fixture = Fixture(Vec::new());
        let first = create_element("button");
        let second = create_element("button");
        let third = create_element("button");
        fixture.append(&first);
        fixture.append(&second);
        fixture.append(&third);

        as_html(&second).focus().expect("focus should succeed");

        let next = get_next_tabbable(Some(second.as_ref())).expect("next should resolve");
        let previous =
            get_previous_tabbable(Some(second.as_ref())).expect("previous should resolve");

        assert!(same_value(&next, &third));
        assert!(same_value(&previous, &first));
    }

    #[wasm_bindgen_test]
    fn get_tabbable_after_element_wraps_around_the_list() {
        let mut fixture = Fixture(Vec::new());
        let first = create_element("button");
        let second = create_element("button");
        fixture.append(&first);
        fixture.append(&second);

        // The near-element helpers hardcode the whole-document body walk
        // (`tabbable.ts:231`), so the harness's own tabbables share the list and
        // the fixtures cannot bound it. The pin asserts the wraparound mechanics
        // (the `(index + dir + count) % count` step, `tabbable.ts:242`) against
        // the module's own body list: stepping off the last element wraps to the
        // first, and before the first wraps to the last.
        let list = tabbable(&body());
        assert!(list.len() >= 2, "the fixtures should add two tabbables");

        let last = list.last().expect("list should be non-empty");
        let after_last = get_tabbable_after_element(Some(last));
        let first_element = list.first().expect("list should be non-empty");
        assert!(after_last.is_some_and(|element| same_value(&element, first_element)));

        let before_first = get_tabbable_before_element(Some(first_element));
        assert!(before_first.is_some_and(|element| same_value(&element, last)));
    }
}
