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
