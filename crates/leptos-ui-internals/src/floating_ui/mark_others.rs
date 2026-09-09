//! Port of `packages/react/src/floating-ui-react/utils/markOthers.ts` — the
//! reference-counted `aria-hidden`/`inert`/`data-base-ui-inert` marking of every
//! element outside a keep-list, the AT-hiding half of the FocusManager⇄utils
//! pipeline (`specs/library/floating-ui-react/implementation.md`, "DOM/portal
//! strategy and why": "`markOthers`-driven `aria-hidden`/`data-base-ui-inert`
//! marking with reference-counted cleanup and `aria-live` preservation";
//! `specs/library/floating-ui-react/parts/safePolygon-utils.md:93-110` covers the
//! marking semantics the suite proves).
//!
//! The upstream file is a fork of `theKashey/aria-hidden` (`markOthers.ts:1-2`)
//! adding conditional `aria-hidden` support; the port reproduces the fork, not the
//! upstream library.
//!
//! ## Rust adaptations
//!
//! - The `WeakMap<Element, number>` counters and `WeakSet<Element>` uncontrolled
//!   sets (`markOthers.ts:14-26`) become [`js_sys::Map`] / [`js_sys::Set`] handles:
//!   both key by SameValueZero, the same element identity the JS code relies on.
//!   The GC difference a `Weak*` collection buys is bounded here by upstream's own
//!   design — every map/set is manually replaced with a fresh one the moment
//!   `lockCount` returns to 0 (`markOthers.ts:205-211`), so entries can only
//!   outlive their marking session, never accumulate across sessions.
//! - The module-level mutable state becomes `thread_local!` `RefCell`s: the
//!   functions only run in a browser/jsdom realm (every call site touches the
//!   DOM), where execution is single-threaded, and upstream's module scope is
//!   exactly one shared instance per document lifetime.
//! - The `Undo` return (`markOthers.ts:6`) is the crate's [`CleanupFn`] vocabulary
//!   (`Box<dyn FnOnce()>`, the shape [`leptos_ui_utils::merge_cleanups`]
//!   composes), so FocusManager's eventual `mergeCleanups` call sites compose it
//!   unchanged.
//! - `markOthers(avoidElements, options = {})` destructures defaults
//!   `ariaHidden = false, inert = false, mark = true` (`markOthers.ts:216`);
//!   [`MarkOthersOptions`] pins those defaults in its [`Default`] impl, so
//!   `MarkOthersOptions::default()` is the upstream no-options call.
//! - `ownerDocument(avoidElements[0]).body` (`markOthers.ts:217`) keeps the empty
//!   list tolerance: `avoid_elements.first()` coerces through
//!   [`leptos_ui_utils::owner_document`]'s `Option<&Node>` to the global document
//!   fallback, the same resolution `ownerDocument(undefined)` performs upstream.
//!   The `.body` read is a plain property access upstream — a realm without a body
//!   element has no meaningful marking scope, and the port unwraps (panics) there
//!   rather than inventing a fallback the fork does not define.

use std::cell::RefCell;

use floating_ui_dom::dom::{DomNodeOrWindow, get_node_name};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, Node, ShadowRoot};

use leptos_ui_utils::merge_cleanups::CleanupFn;
use leptos_ui_utils::owner_document;

/// The `markerName` constant (`markOthers.ts:19`).
const MARKER_NAME: &str = "data-base-ui-inert";

/// The `ControlAttribute` union (`markOthers.ts:20`) — the two attributes whose
/// application is reference-counted. Index order is the `counters` object's key
/// order (`markOthers.ts:14-17`).
const CONTROL_ATTRIBUTES: [&str; 2] = ["inert", "aria-hidden"];

/// The per-attribute `counters` maps and `uncontrolledElementsSets`
/// (`markOthers.ts:14-17`, `markOthers.ts:22-25`), keyed by control attribute.
/// Interior mutability only because the reset arm replaces them wholesale.
struct Counters {
    maps: Vec<js_sys::Map>,
    uncontrolled_sets: Vec<js_sys::Set>,
}

thread_local! {
    static COUNTERS: RefCell<Counters> = RefCell::new(Counters::fresh());
    static MARKER_COUNTER_MAP: RefCell<js_sys::Map> = RefCell::new(js_sys::Map::new());
    static LOCK_COUNT: RefCell<u32> = const { RefCell::new(0) };
}

/// The `&JsValue` view of a node — the SameValueZero key identity the JS `WeakMap`
/// /`WeakSet`/`Set` operations use (`markOthers.ts:144-151,161-167,182-199`).
fn js_ref(node: &Node) -> &JsValue {
    node.as_ref()
}

impl Counters {
    fn fresh() -> Self {
        Counters {
            maps: CONTROL_ATTRIBUTES
                .iter()
                .map(|_| js_sys::Map::new())
                .collect(),
            uncontrolled_sets: CONTROL_ATTRIBUTES
                .iter()
                .map(|_| js_sys::Set::new_empty())
                .collect(),
        }
    }

    fn reset(&mut self) {
        *self = Counters::fresh();
    }

    fn index(control_attribute: &str) -> usize {
        CONTROL_ATTRIBUTES
            .iter()
            .position(|attribute| *attribute == control_attribute)
            .unwrap_or_else(|| panic!("unknown control attribute: {control_attribute}"))
    }

    fn map(&self, control_attribute: &str) -> &js_sys::Map {
        &self.maps[Self::index(control_attribute)]
    }

    fn uncontrolled_set(&self, control_attribute: &str) -> &js_sys::Set {
        &self.uncontrolled_sets[Self::index(control_attribute)]
    }
}

/// The `MarkOthersOptions` shape (`markOthers.ts:8-12`) with the destructured
/// defaults pinned (`markOthers.ts:216`) — [`MarkOthersOptions::default()`] is the
/// upstream no-options call.
#[derive(Clone, Copy, Debug)]
pub struct MarkOthersOptions {
    /// `ariaHidden` — mark others with `aria-hidden="true"`.
    pub aria_hidden: bool,
    /// `inert` — mark others with the `inert` attribute; takes precedence over
    /// `aria_hidden` when both are set (`markOthers.ts:110-114`).
    pub inert: bool,
    /// `mark` — additionally apply the [`MARKER_NAME`] marker to others.
    pub mark: bool,
}

impl Default for MarkOthersOptions {
    fn default() -> Self {
        MarkOthersOptions {
            aria_hidden: false,
            inert: false,
            mark: true,
        }
    }
}

/// The `unwrapHost` helper (`markOthers.ts:33-39`): the nearest shadow-host
/// element, walking up through `parentNode` — how a keep target living inside a
/// shadow root is corrected to the element the light-DOM parent chain contains.
fn unwrap_host(node: Option<&Node>) -> Option<Element> {
    let node = node?;
    if let Some(shadow_root) = node.dyn_ref::<ShadowRoot>() {
        return Some(shadow_root.host());
    }
    unwrap_host(node.parent_node().as_ref())
}

/// The `correctElements` helper (`markOthers.ts:41-56`): targets kept as-is when
/// the parent contains them, else replaced by their unwrapped shadow host, else
/// dropped.
fn correct_elements(parent: &Element, targets: &[Element]) -> Vec<Element> {
    targets
        .iter()
        .filter_map(|target| {
            if parent.contains(Some(target.as_ref())) {
                return Some(target.clone());
            }
            let corrected_target = unwrap_host(Some(target.as_ref()))?;
            parent
                .contains(Some(corrected_target.as_ref()))
                .then_some(corrected_target)
        })
        .collect()
}

/// The `buildKeepSet` helper (`markOthers.ts:58-70`): each target plus its full
/// ancestor chain, as a SameValueZero set of nodes.
fn build_keep_set(targets: &[Element]) -> js_sys::Set {
    let keep = js_sys::Set::new_empty();
    for target in targets {
        // Owned iteration: `parent_node()` returns a fresh wrapper each hop, so the
        // walk carries `Option<Node>` values rather than references into temporaries.
        let mut node: Option<Node> = Some(target.clone().into());
        while let Some(current) = &node {
            let value = js_ref(current);
            if keep.has(value) {
                break;
            }
            keep.add(value);
            node = current.parent_node();
        }
    }
    keep
}

/// The `new Set<Node>(avoidElements)` / `new Set<Node>(controlElements)` stop sets
/// (`markOthers.ts:120,138`) — plain JS `Set`s keyed by element identity.
fn elements_to_set(elements: &[Element]) -> js_sys::Set {
    let set = js_sys::Set::new_empty();
    for element in elements {
        set.add(js_ref(element.as_ref()));
    }
    set
}

/// Collects the elements a `querySelectorAll` matched
/// (`markOthers.ts:130-133` — `Array.from(body.querySelectorAll('[aria-live]'))`).
fn query_all_elements(scope: &Element, selector: &str) -> Vec<Element> {
    let Ok(node_list) = scope.query_selector_all(selector) else {
        return Vec::new();
    };
    js_sys::Array::from(&node_list.into())
        .iter()
        .filter_map(|value| value.dyn_into::<Element>().ok())
        .collect()
}

/// The `collectOutsideElements` helper (`markOthers.ts:72-100`): the depth-first
/// list of elements under `root` that are neither in `keep_elements` (recursed
/// into) nor in `stop_elements` (pruned), skipping `<script>` nodes.
fn collect_outside_elements(
    root: &Element,
    keep_elements: &js_sys::Set,
    stop_elements: &js_sys::Set,
) -> Vec<Element> {
    let mut outside = Vec::new();

    fn walk(
        parent: Option<&Element>,
        keep_elements: &js_sys::Set,
        stop_elements: &js_sys::Set,
        outside: &mut Vec<Element>,
    ) {
        let Some(parent) = parent else {
            return;
        };
        if stop_elements.has(js_ref(parent.as_ref())) {
            return;
        }

        let children = parent.children();
        for index in 0..children.length() {
            let Some(node) = children.item(index) else {
                continue;
            };
            // `getNodeName(node) === 'script'` (`markOthers.ts:85`) — the delegated
            // `@floating-ui/utils/dom` helper, lowercased node name.
            if get_node_name(DomNodeOrWindow::Node(node.as_ref())) == "script" {
                continue;
            }

            if keep_elements.has(js_ref(node.as_ref())) {
                walk(Some(&node), keep_elements, stop_elements, outside);
            } else {
                outside.push(node);
            }
        }
    }

    walk(Some(root), keep_elements, stop_elements, &mut outside);

    outside
}

/// The `applyAttributeToOthers` body (`markOthers.ts:102-213`) — shared by the
/// public entry, which only resolves the document body first.
fn apply_attribute_to_others(
    uncorrected_avoid_elements: &[Element],
    body: &Element,
    aria_hidden: bool,
    inert: bool,
    mark: bool,
) -> CleanupFn {
    // `controlAttribute` (`markOthers.ts:109-114`): `inert` wins over `aria-hidden`.
    let control_attribute: Option<&'static str> = if inert {
        Some("inert")
    } else if aria_hidden {
        Some("aria-hidden")
    } else {
        None
    };

    let avoid_elements = correct_elements(body, uncorrected_avoid_elements);
    let marker_targets: Vec<Element> = if mark {
        let stop_elements = elements_to_set(&avoid_elements);
        collect_outside_elements(body, &build_keep_set(&avoid_elements), &stop_elements)
    } else {
        Vec::new()
    };
    let mut hidden_elements: Vec<Element> = Vec::new();
    let mut marked_elements: Vec<Element> = Vec::new();

    // The counter map + uncontrolled set this call owns (`markOthers.ts:125-129`):
    // fresh Rust handles to the same JS objects upstream's closure captures by
    // reference — a wholesale reset replaces the thread-local, not these handles,
    // matching the JS closure semantics exactly.
    let counter_map: Option<js_sys::Map> =
        control_attribute.map(|attribute| COUNTERS.with_borrow(|c| c.map(attribute).clone()));
    let uncontrolled_set: Option<js_sys::Set> = control_attribute
        .map(|attribute| COUNTERS.with_borrow(|c| c.uncontrolled_set(attribute).clone()));

    if let Some(attribute) = control_attribute {
        let aria_live_elements = correct_elements(body, &query_all_elements(body, "[aria-live]"));
        let mut control_elements = avoid_elements.clone();
        control_elements.extend(aria_live_elements);
        let stop_elements = elements_to_set(&control_elements);
        let control_targets =
            collect_outside_elements(body, &build_keep_set(&control_elements), &stop_elements);

        let map = counter_map.clone().unwrap_or_else(js_sys::Map::new);
        let current_uncontrolled_set = uncontrolled_set
            .clone()
            .unwrap_or_else(js_sys::Set::new_empty);

        for node in &control_targets {
            let attr = node.get_attribute(attribute);
            // `attr !== null && attr !== 'false'` (`markOthers.ts:143`) — an empty
            // string attribute value counts as already hidden.
            let already_hidden = attr.as_deref().is_some_and(|value| value != "false");
            let counter_value = map.get(js_ref(node.as_ref())).as_f64().unwrap_or(0.0) + 1.0;

            map.set(js_ref(node.as_ref()), &JsValue::from(counter_value));
            hidden_elements.push(node.clone());

            if counter_value == 1.0 && already_hidden {
                current_uncontrolled_set.add(js_ref(node.as_ref()));
            }

            if !already_hidden {
                // `controlAttribute === 'inert' ? '' : 'true'` (`markOthers.ts:154`).
                node.set_attribute(attribute, if attribute == "inert" { "" } else { "true" })
                    .ok();
            }
        }
    }

    if mark {
        MARKER_COUNTER_MAP.with_borrow(|marker_counter_map| {
            for node in &marker_targets {
                let marker_value = marker_counter_map
                    .get(js_ref(node.as_ref()))
                    .as_f64()
                    .unwrap_or(0.0)
                    + 1.0;

                marker_counter_map.set(js_ref(node.as_ref()), &JsValue::from(marker_value));
                marked_elements.push(node.clone());

                if marker_value == 1.0 {
                    node.set_attribute(MARKER_NAME, "").ok();
                }
            }
        });
    }

    LOCK_COUNT.with_borrow_mut(|lock_count| *lock_count += 1);

    Box::new(move || {
        if let Some(counter_map) = &counter_map {
            for element in &hidden_elements {
                let current_counter_value = counter_map
                    .get(js_ref(element.as_ref()))
                    .as_f64()
                    .unwrap_or(0.0);
                let counter_value = current_counter_value - 1.0;
                counter_map.set(js_ref(element.as_ref()), &JsValue::from(counter_value));

                if counter_value == 0.0 {
                    if let Some(attribute) = control_attribute {
                        let is_uncontrolled = uncontrolled_set
                            .as_ref()
                            .map(|set| set.has(js_ref(element.as_ref())))
                            .unwrap_or(false);
                        if !is_uncontrolled {
                            // Externally owned attributes are preserved
                            // (`markOthers.ts:182-184` — the uncontrolled set's
                            // whole job).
                            element.remove_attribute(attribute).ok();
                        }
                    }

                    if let Some(set) = &uncontrolled_set {
                        set.delete(js_ref(element.as_ref()));
                    }
                }
            }
        }

        if mark {
            MARKER_COUNTER_MAP.with_borrow(|marker_counter_map| {
                for element in &marked_elements {
                    let marker_value = marker_counter_map
                        .get(js_ref(element.as_ref()))
                        .as_f64()
                        .unwrap_or(0.0)
                        - 1.0;

                    marker_counter_map.set(js_ref(element.as_ref()), &JsValue::from(marker_value));

                    if marker_value == 0.0 {
                        element.remove_attribute(MARKER_NAME).ok();
                    }
                }
            });
        }

        let unlocked = LOCK_COUNT.with_borrow_mut(|lock_count| {
            *lock_count -= 1;
            *lock_count == 0
        });

        // The `!lockCount` reset (`markOthers.ts:205-211`) — every map and set is
        // replaced so nothing outlives the marking session.
        if unlocked {
            COUNTERS.with_borrow_mut(Counters::reset);
            MARKER_COUNTER_MAP.with_borrow_mut(|map| *map = js_sys::Map::new());
        }
    })
}

/// The `markOthers` export (`markOthers.ts:215-219`): marks every element outside
/// `avoid_elements` (after shadow-host correction) per the options, and returns
/// the reference-counted undo — call it exactly once to release this call's marks.
pub fn mark_others(avoid_elements: &[Element], options: MarkOthersOptions) -> CleanupFn {
    let MarkOthersOptions {
        aria_hidden,
        inert,
        mark,
    } = options;
    let body: HtmlElement = owner_document(avoid_elements.first().map(|element| element.as_ref()))
        .body()
        .unwrap_or_else(|| panic!("markOthers: the owner document has no body element"));
    apply_attribute_to_others(avoid_elements, &body, aria_hidden, inert, mark)
}

// The marking-semantics tests need a real DOM realm (attribute surgery, shadow
// roots): they mirror `packages/react/src/floating-ui-react/utils/markOthers.test.ts`.
//
// Harness note: upstream sweeps the body in `afterEach`
// (`markOthers.test.ts:4-6`), but the wasm-bindgen-test harness renders its own
// output elements into the body — clearing it kills the harness (the
// getDefaultFormSubmitter port documents the same wall). Fixtures are tracked in
// a [`Fixture`] and detached on drop instead.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
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
    /// `afterEach` body sweep (`markOthers.test.ts:4-6`) can run as a drop-time
    /// `Element.remove()` per element — removal, not `innerHTML` clearing, keeps
    /// the wasm-bindgen-test harness alive.
    struct Fixture(Vec<Element>);

    impl Fixture {
        fn append(&mut self, element: &Element) {
            body().append_child(element).expect("append should succeed");
            self.0.push(element.clone());
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for element in self.0.drain(..) {
                element.remove();
            }
        }
    }

    fn create_div() -> Element {
        document()
            .create_element("div")
            .expect("div should be creatable")
    }

    fn create_element(tag: &str) -> Element {
        document()
            .create_element(tag)
            .unwrap_or_else(|_| panic!("{tag} should be creatable"))
    }

    /// `markOthers(targets, { ariaHidden: true })`
    /// (`markOthers.test.ts:14`, `:29`, `:62`, `:94`, `:132`, `:183`, `:282`, `:305`).
    fn mark_aria_hidden(targets: &[Element]) -> CleanupFn {
        mark_others(
            targets,
            MarkOthersOptions {
                aria_hidden: true,
                ..MarkOthersOptions::default()
            },
        )
    }

    fn assert_aria_hidden(element: &Element) {
        assert_eq!(
            element.get_attribute("aria-hidden"),
            Some("true".to_owned()),
            "aria-hidden should be 'true'"
        );
    }

    #[wasm_bindgen_test]
    fn single_call() {
        let mut fixture = Fixture(Vec::new());
        let other = create_div();
        fixture.append(&other);
        let target = create_div();
        fixture.append(&target);

        let cleanup = mark_aria_hidden(&[target.clone()]);

        assert_aria_hidden(&other);

        cleanup();

        assert_eq!(
            other.get_attribute("aria-hidden"),
            None,
            "aria-hidden should be removed after cleanup"
        );
    }

    #[wasm_bindgen_test]
    fn multiple_calls() {
        let mut fixture = Fixture(Vec::new());
        let other = create_div();
        fixture.append(&other);
        let target = create_div();
        fixture.append(&target);

        let cleanup = mark_aria_hidden(&[target.clone()]);

        assert_aria_hidden(&other);

        let next_target = create_div();
        fixture.append(&next_target);

        let next_cleanup = mark_aria_hidden(&[next_target.clone()]);

        assert_aria_hidden(&target);
        assert_eq!(
            next_target.get_attribute("aria-hidden"),
            None,
            "a fresh keep target is not marked"
        );

        next_target.remove();

        next_cleanup();

        assert_eq!(target.get_attribute("aria-hidden"), None);
        assert_aria_hidden(&other);

        cleanup();

        assert_eq!(other.get_attribute("aria-hidden"), None);

        fixture.append(&next_target);
    }

    #[wasm_bindgen_test]
    fn out_of_order_cleanup() {
        let mut fixture = Fixture(Vec::new());
        let other = create_div();
        fixture.append(&other);
        let target = create_div();
        target
            .set_attribute("data-testid", "")
            .expect("set should succeed");
        fixture.append(&target);

        let cleanup = mark_aria_hidden(&[target.clone()]);

        assert_aria_hidden(&other);

        let next_target = create_div();
        fixture.append(&next_target);

        let next_cleanup = mark_aria_hidden(&[next_target.clone()]);

        assert_aria_hidden(&target);
        assert_eq!(next_target.get_attribute("aria-hidden"), None);

        cleanup();

        assert_eq!(next_target.get_attribute("aria-hidden"), None);
        assert_aria_hidden(&target);
        assert_aria_hidden(&other);

        next_cleanup();

        assert_eq!(next_target.get_attribute("aria-hidden"), None);
        assert_eq!(other.get_attribute("aria-hidden"), None);
        assert_eq!(target.get_attribute("aria-hidden"), None);
    }

    #[wasm_bindgen_test]
    fn multiple_cleanups_with_differing_control_attribute() {
        let mut fixture = Fixture(Vec::new());
        let other = create_div();
        fixture.append(&other);
        let target = create_div();
        target
            .set_attribute("data-testid", "1")
            .expect("set should succeed");
        fixture.append(&target);

        let cleanup = mark_aria_hidden(&[target.clone()]);

        assert_aria_hidden(&other);

        let target2 = create_div();
        target2
            .set_attribute("data-testid", "2")
            .expect("set should succeed");
        fixture.append(&target2);

        let cleanup2 = mark_others(&[target2.clone()], MarkOthersOptions::default());

        assert_ne!(target.get_attribute("aria-hidden"), Some("true".to_owned()));
        assert_eq!(
            target.get_attribute("data-base-ui-inert"),
            Some(String::new()),
            "the mark-only call marks the first call's keep target"
        );

        cleanup();

        assert_eq!(other.get_attribute("aria-hidden"), None);

        cleanup2();

        assert_eq!(
            target.get_attribute("data-base-ui-inert"),
            None,
            "the marker is removed by its own cleanup"
        );
    }

    #[wasm_bindgen_test]
    fn mixed_control_attribute_usage() {
        let mut fixture = Fixture(Vec::new());
        let other = create_div();
        fixture.append(&other);

        let a = create_div();
        a.set_attribute("data-testid", "A")
            .expect("set should succeed");
        fixture.append(&a);

        let b = create_div();
        b.set_attribute("data-testid", "B")
            .expect("set should succeed");
        fixture.append(&b);

        let c = create_div();
        c.set_attribute("data-testid", "C")
            .expect("set should succeed");
        fixture.append(&c);

        let cleanup_a = mark_aria_hidden(&[a.clone()]);

        assert_aria_hidden(&other);
        assert!(!other.has_attribute("inert"));
        assert_eq!(
            other.get_attribute("data-base-ui-inert"),
            Some(String::new())
        );

        let cleanup_b = mark_others(
            &[b.clone()],
            MarkOthersOptions {
                inert: true,
                ..MarkOthersOptions::default()
            },
        );

        assert_aria_hidden(&other);
        assert_eq!(other.get_attribute("inert"), Some(String::new()));
        assert_eq!(
            other.get_attribute("data-base-ui-inert"),
            Some(String::new())
        );

        let cleanup_c = mark_others(&[c.clone()], MarkOthersOptions::default());

        assert_aria_hidden(&other);
        assert_eq!(other.get_attribute("inert"), Some(String::new()));
        assert_eq!(
            other.get_attribute("data-base-ui-inert"),
            Some(String::new())
        );

        cleanup_c();

        assert_aria_hidden(&other);
        assert_eq!(other.get_attribute("inert"), Some(String::new()));
        assert_eq!(
            other.get_attribute("data-base-ui-inert"),
            Some(String::new())
        );

        cleanup_b();

        assert_aria_hidden(&other);
        assert!(!other.has_attribute("inert"));
        assert_eq!(
            other.get_attribute("data-base-ui-inert"),
            Some(String::new())
        );

        cleanup_a();

        assert!(!other.has_attribute("aria-hidden"));
        assert!(!other.has_attribute("inert"));
        assert!(!other.has_attribute("data-base-ui-inert"));
    }

    #[wasm_bindgen_test]
    fn tracks_externally_controlled_attributes_per_control_attribute() {
        let mut fixture = Fixture(Vec::new());
        let container = create_div();
        let keep = create_div();
        let outside = create_div();

        outside
            .set_attribute("inert", "")
            .expect("set should succeed");
        container
            .append_child(&keep)
            .expect("append should succeed");
        container
            .append_child(&outside)
            .expect("append should succeed");
        fixture.append(&container);

        let cleanup_inert = mark_others(
            &[keep.clone()],
            MarkOthersOptions {
                inert: true,
                ..MarkOthersOptions::default()
            },
        );
        let cleanup_aria_hidden = mark_aria_hidden(&[keep.clone()]);

        assert!(outside.has_attribute("inert"));
        assert_aria_hidden(&outside);

        cleanup_aria_hidden();

        assert!(outside.has_attribute("inert"));
        assert!(
            !outside.has_attribute("aria-hidden"),
            "the externally owned inert attribute outlives the aria-hidden cleanup"
        );

        cleanup_inert();

        assert!(
            outside.has_attribute("inert"),
            "the pre-existing inert attribute is never removed"
        );
        assert!(!outside.has_attribute("aria-hidden"));
    }

    #[wasm_bindgen_test]
    fn preserves_externally_owned_aria_hidden_during_concurrent_overlaps() {
        let mut fixture = Fixture(Vec::new());
        let keep = create_div();
        let outside = create_div();
        outside
            .set_attribute("aria-hidden", "true")
            .expect("set should succeed");
        fixture.append(&keep);
        fixture.append(&outside);

        let cleanup_aria_hidden = mark_others(
            &[keep.clone()],
            MarkOthersOptions {
                aria_hidden: true,
                mark: false,
                ..MarkOthersOptions::default()
            },
        );
        let cleanup_inert = mark_others(
            &[keep.clone()],
            MarkOthersOptions {
                inert: true,
                mark: false,
                ..MarkOthersOptions::default()
            },
        );

        assert_aria_hidden(&outside);
        assert!(outside.has_attribute("inert"));

        cleanup_inert();

        assert_aria_hidden(&outside);
        assert!(
            !outside.has_attribute("inert"),
            "the inert cleanup does not strip the externally owned aria-hidden"
        );

        cleanup_aria_hidden();

        assert_aria_hidden(&outside);
        assert!(!outside.has_attribute("inert"));
    }

    #[wasm_bindgen_test]
    fn does_not_let_mark_only_overlap_disturb_control_cleanup_bookkeeping() {
        let mut fixture = Fixture(Vec::new());
        let keep = create_div();
        let outside = create_div();
        fixture.append(&keep);
        fixture.append(&outside);

        let cleanup_mark_only = mark_others(&[keep.clone()], MarkOthersOptions::default());
        let cleanup_control_only = mark_others(
            &[keep.clone()],
            MarkOthersOptions {
                aria_hidden: true,
                mark: false,
                ..MarkOthersOptions::default()
            },
        );

        assert!(outside.has_attribute("data-base-ui-inert"));
        assert_aria_hidden(&outside);

        cleanup_mark_only();

        assert!(
            !outside.has_attribute("data-base-ui-inert"),
            "the mark-only cleanup removes only its own marker"
        );
        assert_aria_hidden(&outside);

        cleanup_control_only();

        assert!(!outside.has_attribute("data-base-ui-inert"));
        assert!(!outside.has_attribute("aria-hidden"));
    }

    #[wasm_bindgen_test]
    fn does_not_recurse_infinitely_with_target_inside_anchor_in_shadow_root() {
        let mut fixture = Fixture(Vec::new());
        let host = create_div();
        fixture.append(&host);

        let shadow_root = host
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .expect("shadow root should attach");
        let anchor = create_element("a");
        anchor
            .set_attribute("href", "https://floating-ui.com")
            .expect("set should succeed");

        let target = create_element("button");
        anchor.append_child(&target).expect("append should succeed");
        shadow_root
            .append_child(&anchor)
            .expect("append should succeed");

        let cleanup = mark_aria_hidden(&[target.clone()]);

        assert!(
            !host.has_attribute("aria-hidden"),
            "the host contains the target, so it must not have been hidden"
        );

        cleanup();
    }

    #[wasm_bindgen_test]
    fn uses_shadow_root_host_as_avoid_element_when_parent_chain_includes_anchor() {
        let mut fixture = Fixture(Vec::new());
        let outside = create_div();
        fixture.append(&outside);

        let host = create_div();
        fixture.append(&host);

        let shadow_root = host
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .expect("shadow root should attach");
        let anchor = create_element("a");
        anchor
            .set_attribute("href", "https://floating-ui.com")
            .expect("set should succeed");

        let target = create_element("button");
        anchor.append_child(&target).expect("append should succeed");
        shadow_root
            .append_child(&anchor)
            .expect("append should succeed");

        let cleanup = mark_aria_hidden(&[target.clone()]);

        assert_aria_hidden(&outside);
        assert_eq!(
            host.get_attribute("aria-hidden"),
            None,
            "the shadow host is corrected to the avoid element, not marked"
        );

        cleanup();

        assert_eq!(outside.get_attribute("aria-hidden"), None);
    }
}
