//! Port of `packages/react/src/floating-ui-react/utils/getEmptyRootContext.ts` — the
//! throwaway root store consumers outside a real popup get (`FloatingRootStore.ts` used
//! by navigation-menu and the test fixtures,
//! `specs/library/floating-ui-react/implementation.md`, "Context providers/consumers":
//! "Not React context").

use std::rc::Rc;

use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

/// `getEmptyRootContext()` (`getEmptyRootContext.ts:5-17`): a `FloatingRootStore`
/// seeded with every closed/empty default — `open: false`, no elements, no id,
/// `syncOnly: false`, not nested, no `onOpenChange`.
pub fn get_empty_root_context() -> Rc<FloatingRootStore> {
    FloatingRootStore::new(FloatingRootStoreOptions {
        open: false,
        transition_status: None,
        reference_element: None,
        floating_element: None,
        trigger_elements: PopupTriggerMap::new(),
        floating_id: None,
        sync_only: false,
        nested: false,
        on_open_change: None,
    })
}
