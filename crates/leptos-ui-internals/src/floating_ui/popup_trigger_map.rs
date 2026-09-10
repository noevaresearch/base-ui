//! Provisional port of `packages/react/src/utils/popups/popupTriggerMap.ts` — the
//! id→trigger-element registry the floating store context carries
//! (`components/FloatingRootStore.ts:30`, consumed by the hover/dismiss/focus trigger
//! checks, e.g. `hooks/useHoverReferenceInteraction.ts:118-142`).
//!
//! The full `popups` unit belongs to the `infra: utils` TODO item (in progress); the
//! floating-ui unit could not compile without the type, so the registry ported here
//! first with its upstream behavior (add/evict, delete, membership, lookup, iteration)
//! and moves behind a re-export of the popups port when that item lands. The unit's
//! own test suite (`popupTriggerMap.test.ts`) now ports here too — see the wasm test
//! module — on top of the floating-ui consumers' original behavior tests.
//!
//! ## Rust adaptations
//!
//! - Upstream's dev-only duplicate-element check (`popupTriggerMap.ts:41-61`,
//!   `process.env.NODE_ENV !== 'production'`) ports to a `debug_assertions` gate: the
//!   panic fires in dev builds and tests, stripped from release — the Rust analog of
//!   NODE_ENV gating. Upstream throws; a Rust panic is the analog for a
//!   programming-error contract.
//! - `hasElement`/`hasMatchingElement` compare element identity
//!   (`popupTriggerMap.ts:82-93` — `registered === element`); `web_sys`'s `PartialEq`
//!   is strict-equals, the same JS identity semantics.
//! - Upstream instances are shared by JS object reference across the store contexts
//!   that carry them — the popup-store context and each floating store's context hold
//!   one live registry, so a trigger registration is visible through every handle
//!   (`useSyncedFloatingRootContext` hands the popup store's map to the floating store
//!   it syncs, `hooks/useSyncedFloatingRootContext.ts:56,70`). The port backs the map
//!   with `Rc<RefCell<…>>`: methods take `&self`, and `Clone` produces a handle onto
//!   the *same* registry (the analog of passing the object by reference), which is what
//!   lets one map live in both contexts.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use web_sys::Element;

/// Port of upstream `PopupTriggerMap` (`popupTriggerMap.ts:28-118`): stores trigger
/// elements keyed by id for multi-trigger popups. Clones share one registry (see the
/// module docs on shared identity); mutation is interior so any handle can register.
#[derive(Clone, Default)]
pub struct PopupTriggerMap {
    id_map: Rc<RefCell<HashMap<String, Element>>>,
}

impl PopupTriggerMap {
    /// Upstream constructor (`popupTriggerMap.ts:31-33`).
    pub fn new() -> Self {
        Self::default()
    }

    /// `add(id, element)` (`popupTriggerMap.ts:40-64`): registers the trigger, evicting
    /// any previous element registered under the same id. The dev-mode guard panics when
    /// the same element is registered under two different ids
    /// (`popupTriggerMap.ts:48-50`).
    pub fn add(&self, id: &str, element: Element) {
        #[cfg(debug_assertions)]
        {
            let existing_id = self
                .id_map
                .borrow()
                .iter()
                .find(|(_, registered)| *registered == &element)
                .map(|(existing_id, _)| existing_id.clone());
            if let Some(existing_id) = existing_id {
                assert!(
                    existing_id == id,
                    "Base UI: A trigger element cannot be registered under multiple IDs in \
                     PopupTriggerMap."
                );
            }
        }

        self.id_map.borrow_mut().insert(id.to_owned(), element);
    }

    /// `delete(id)` (`popupTriggerMap.ts:70-81`).
    pub fn delete(&self, id: &str) {
        self.id_map.borrow_mut().remove(id);
    }

    /// `hasElement(element)` (`popupTriggerMap.ts:83-93`): whether the element is
    /// registered as a trigger (by identity).
    pub fn has_element(&self, element: &Element) -> bool {
        self.id_map
            .borrow()
            .values()
            .any(|registered| registered == element)
    }

    /// `hasMatchingElement(predicate)` (`popupTriggerMap.ts:95-103`).
    pub fn has_matching_element(&self, predicate: impl Fn(&Element) -> bool) -> bool {
        self.id_map
            .borrow()
            .values()
            .any(|element| predicate(element))
    }

    /// `getById(id)` (`popupTriggerMap.ts:105-109`). Returns a clone — the registry
    /// lives behind the shared `Rc`, so a borrowed element cannot outlive the borrow
    /// guard.
    pub fn get_by_id(&self, id: &str) -> Option<Element> {
        self.id_map.borrow().get(id).cloned()
    }

    /// `entries()` (`popupTriggerMap.ts:111-115`): iteration over `(id, element)`
    /// pairs. Upstream returns a live iterator; the port visits through a closure so
    /// the borrow cannot outlive the call. The visit must not re-enter the map's
    /// mutators (it holds the registry borrow).
    pub fn for_each_entry(&self, mut visit: impl FnMut(&str, &Element)) {
        for (id, element) in &*self.id_map.borrow() {
            visit(id, element);
        }
    }

    /// `elements()` (`popupTriggerMap.ts:117-121`): iteration over the registered
    /// elements (see `for_each_entry` for the live-iterator adaptation).
    pub fn for_each_element(&self, mut visit: impl FnMut(&Element)) {
        for element in self.id_map.borrow().values() {
            visit(element);
        }
    }

    /// `size` (`popupTriggerMap.ts:123-127`).
    pub fn size(&self) -> usize {
        self.id_map.borrow().len()
    }
}

/// The [`PopupTriggerLookup`] bridge (`packages/react/src/floating-ui-react/utils/
/// element.ts:5,12` — the trigger-membership checks take the map): the registered ids
/// are never null, so the `Option<&str>` visit always carries `Some`.
impl crate::floating_ui::element::PopupTriggerLookup for PopupTriggerMap {
    fn has_element(&self, element: &Element) -> bool {
        PopupTriggerMap::has_element(self, element)
    }

    fn for_each_trigger(&self, visit: &mut dyn FnMut(Option<&str>, &Element)) {
        self.for_each_entry(|id, element| visit(Some(id), element));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    //! Port of `packages/react/src/utils/popups/popupTriggerMap.test.ts` — the unit's
    //! own test suite for the registry (behavior.md, "Edge cases → `PopupTriggerMap`").
    //!
    //! The upstream production-mode test (`popupTriggerMap.test.ts:126-146` — no
    //! duplicate check when `NODE_ENV=production`) has no port-side test: the dev gate
    //! is `#[cfg(debug_assertions)]`, compile-time in Rust, and the test harness builds
    //! debug. The remaining nine tests port 1:1.

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn button() -> Element {
        document().create_element("button").unwrap()
    }

    const DUPLICATE_ID_PANIC: &str =
        "Base UI: A trigger element cannot be registered under multiple IDs in PopupTriggerMap.";

    // Mirrors `popupTriggerMap.test.ts:17-24`.
    #[wasm_bindgen_test]
    fn adds_and_retrieves_elements_by_id() {
        let map = PopupTriggerMap::new();
        let element = button();

        map.add("trigger", element.clone());

        assert_eq!(map.get_by_id("trigger"), Some(element.clone()));
        assert!(map.has_element(&element));
        assert!(map.has_matching_element(|el| el == &element));
        assert_eq!(map.size(), 1);
    }

    // Mirrors `popupTriggerMap.test.ts:26-35` — reusing an id evicts the previous
    // element, which then no longer counts as registered.
    #[wasm_bindgen_test]
    fn replaces_an_existing_element_when_the_id_is_reused() {
        let map = PopupTriggerMap::new();
        let first = button();
        let second = button();

        map.add("trigger", first.clone());
        map.add("trigger", second.clone());

        assert_eq!(map.get_by_id("trigger"), Some(second.clone()));
        assert!(!map.has_element(&first));
        assert!(map.has_element(&second));
        assert_eq!(map.size(), 1);
    }

    // Mirrors `popupTriggerMap.test.ts:37-45`.
    #[wasm_bindgen_test]
    fn deletes_elements_by_id() {
        let map = PopupTriggerMap::new();
        let element = button();

        map.add("trigger", element.clone());
        map.delete("trigger");

        assert_eq!(map.get_by_id("trigger"), None);
        assert!(!map.has_element(&element));
        assert!(!map.has_matching_element(|el| el == &element));
        assert_eq!(map.size(), 0);
    }

    // Mirrors `popupTriggerMap.test.ts:47-54` — map-set semantics, not multiset.
    #[wasm_bindgen_test]
    fn does_not_duplicate_when_the_same_element_is_added_twice_with_the_same_id() {
        let map = PopupTriggerMap::new();
        let element = button();

        map.add("trigger", element.clone());
        map.add("trigger", element.clone());

        assert_eq!(map.get_by_id("trigger"), Some(element));
        assert_eq!(map.size(), 1);
    }

    // Mirrors `popupTriggerMap.test.ts:56-71` (the `NODE_ENV=development` arm — the
    // port's dev gate is compile-time, and the debug harness runs with it on).
    #[wasm_bindgen_test]
    #[should_panic(expected = "Base UI: A trigger element cannot be registered under multiple IDs")]
    fn panics_when_the_same_element_is_registered_under_multiple_ids() {
        let map = PopupTriggerMap::new();
        let element = button();

        map.add("first", element.clone());
        map.add("second", element);
    }

    // Mirrors `popupTriggerMap.test.ts:73-80` — deleting releases the claim.
    #[wasm_bindgen_test]
    fn allows_re_registering_an_element_under_a_new_id_after_it_was_deleted() {
        let map = PopupTriggerMap::new();
        let element = button();

        map.add("first", element.clone());
        map.delete("first");

        map.add("second", element.clone());
        assert_eq!(map.get_by_id("second"), Some(element));
        assert_eq!(map.size(), 1);
    }

    // Mirrors `popupTriggerMap.test.ts:82-95` — a delete only releases the deleted
    // id's own claim; other elements' claims are unaffected.
    #[wasm_bindgen_test]
    #[should_panic(expected = "Base UI: A trigger element cannot be registered under multiple IDs")]
    fn keeps_an_unrelated_element_claim_when_another_id_is_deleted() {
        let map = PopupTriggerMap::new();
        let first = button();
        let second = button();

        map.add("first", first.clone());
        map.add("second", second);
        map.delete("second");

        map.add("other", first);
    }

    // Mirrors `popupTriggerMap.test.ts:97-112` — an element evicted by id reuse is
    // free to claim a new id.
    #[wasm_bindgen_test]
    fn allows_an_element_evicted_by_id_reuse_to_register_under_a_new_id() {
        let map = PopupTriggerMap::new();
        let first = button();
        let second = button();

        map.add("trigger", first.clone());
        // `first` is no longer registered under `trigger`, so it may claim another id.
        map.add("trigger", second.clone());

        map.add("other", first.clone());
        assert_eq!(map.get_by_id("other"), Some(first));
        assert_eq!(map.get_by_id("trigger"), Some(second));
        assert_eq!(map.size(), 2);
    }

    // Mirrors `popupTriggerMap.test.ts:114-124` — re-adding under the *same* id does
    // not launder the claim; a second id still panics.
    #[wasm_bindgen_test]
    #[should_panic(expected = "Base UI: A trigger element cannot be registered under multiple IDs")]
    fn still_panics_when_a_re_added_element_is_registered_under_a_second_id() {
        let map = PopupTriggerMap::new();
        let element = button();

        map.add("first", element.clone());
        map.add("first", element.clone());

        map.add("second", element);
    }}
