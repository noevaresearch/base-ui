//! Provisional port of `packages/react/src/utils/popups/popupTriggerMap.ts` — the
//! id→trigger-element registry the floating store context carries
//! (`components/FloatingRootStore.ts:30`, consumed by the hover/dismiss/focus trigger
//! checks, e.g. `hooks/useHoverReferenceInteraction.ts:118-142`).
//!
//! The full `popups` unit belongs to the `infra: utils` TODO item (not yet ported); the
//! floating-ui unit cannot compile without the type, so the registry ports here first
//! with its upstream behavior (add/evict, delete, membership, lookup, iteration) and
//! moves behind a re-export of the popups port when that item lands. Only the methods
//! the floating-ui unit itself calls are behavior-tested here (`hasElement` ×3,
//! `elements` ×1, `entries` ×1 across the unit's sources); the rest port for shape
//! parity.
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

use std::collections::HashMap;

use web_sys::Element;

/// Port of upstream `PopupTriggerMap` (`popupTriggerMap.ts:28-118`): stores trigger
/// elements keyed by id for multi-trigger popups.
#[derive(Default)]
pub struct PopupTriggerMap {
    id_map: HashMap<String, Element>,
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
    pub fn add(&mut self, id: &str, element: Element) {
        #[cfg(debug_assertions)]
        {
            let existing_id = self
                .id_map
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

        self.id_map.insert(id.to_owned(), element);
    }

    /// `delete(id)` (`popupTriggerMap.ts:70-81`).
    pub fn delete(&mut self, id: &str) {
        self.id_map.remove(id);
    }

    /// `hasElement(element)` (`popupTriggerMap.ts:83-93`): whether the element is
    /// registered as a trigger (by identity).
    pub fn has_element(&self, element: &Element) -> bool {
        self.id_map.values().any(|registered| registered == element)
    }

    /// `hasMatchingElement(predicate)` (`popupTriggerMap.ts:95-103`).
    pub fn has_matching_element(&self, predicate: impl Fn(&Element) -> bool) -> bool {
        self.id_map.values().any(|element| predicate(element))
    }

    /// `getById(id)` (`popupTriggerMap.ts:105-109`).
    pub fn get_by_id(&self, id: &str) -> Option<&Element> {
        self.id_map.get(id)
    }

    /// `entries()` (`popupTriggerMap.ts:111-115`): iteration over `(id, element)`
    /// pairs. Upstream returns a live iterator; the port visits through a closure so
    /// the borrow cannot outlive the call.
    pub fn for_each_entry(&self, mut visit: impl FnMut(&str, &Element)) {
        for (id, element) in &self.id_map {
            visit(id, element);
        }
    }

    /// `elements()` (`popupTriggerMap.ts:117-121`): iteration over the registered
    /// elements (see `for_each_entry` for the live-iterator adaptation).
    pub fn for_each_element(&self, mut visit: impl FnMut(&Element)) {
        for element in self.id_map.values() {
            visit(element);
        }
    }

    /// `size` (`popupTriggerMap.ts:123-127`).
    pub fn size(&self) -> usize {
        self.id_map.len()
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
