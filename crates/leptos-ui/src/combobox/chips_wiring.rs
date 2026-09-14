//! Combobox.Chips — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/chips/ComboboxChips.tsx`).
//!
//! The behavior layer (the `toolbar` role rule, the open-clears-highlight
//! render reset) is the already-ported [`chips_role`]/[`ChipsRuntime`]
//! runtime; this module is the seam that reads the store selections the part
//! makes (`ComboboxChips.tsx:22-24`), derives the role, wires the
//! highlighted-chip state, and executes the container's `onMouseDown` through
//! the shared [`handle_input_press`] funnel (`:46-48`). The DOM reads arrive
//! as arguments, the commands issue through the store context —
//! host-testable by construction.

use std::cell::Cell;
use std::rc::Rc;

use crate::combobox::parts_util::{InputPressEvent, handle_input_press};
use crate::combobox::store::{ComboboxStore, selectors};
use crate::combobox::value_chips::{ChipsRuntime, chips_role};

/// The part's derived state (`ComboboxChips.tsx:22-44`): what the element's
/// role attribute and the press handler read.
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxChipsState {
    /// The store's `open` (`:22`).
    pub open: bool,
    /// The store's `hasSelectionChips` (`:23`) — drives the `toolbar` role.
    pub has_selection_chips: bool,
    /// The store's `disabled` (`:47`).
    pub disabled: bool,
}

/// Reads the part's state (`ComboboxChips.tsx:22-47`).
pub fn combobox_chips_state(store: &ComboboxStore) -> ComboboxChipsState {
    let state = store.select(|s| s.clone());
    ComboboxChipsState {
        open: state.open,
        has_selection_chips: selectors::has_selection_chips(&state),
        disabled: state.disabled,
    }
}

/// The element's `role` attribute (`ComboboxChips.tsx:42-44`): `Some("toolbar")`
/// only while selection chips render — none while empty.
pub fn chips_element_role(state: &ComboboxChipsState) -> Option<&'static str> {
    chips_role(state.has_selection_chips)
}

/// The part's per-instance state pair: the `highlightedChipIndex` React state
/// (`:26-35`) and the `chipsRef` array of rendered chip elements (`:38`). The
/// port carries both on one handle; the context value the upstream
/// `useMemo` builds is the handle itself shared by reference.
#[derive(Default)]
pub struct ComboboxChipsRuntime {
    /// The `highlightedChipIndex` state — cleared whenever the popup opens.
    pub chips: ChipsRuntime,
    /// The `chipsRef` array (`:38`): rendered chip elements in order; the
    /// chip's own registration writes its slot, navigation reads it.
    pub chips_ref: Rc<std::cell::RefCell<Vec<Option<web_sys::Element>>>>,
}

impl ComboboxChipsRuntime {
    /// The render-time reset (`ComboboxChips.tsx:31-33`): when the popup is
    /// open and a chip is highlighted, clear it. Returns `true` when reset.
    pub fn sync_open(&self, open: bool) -> bool {
        self.chips.sync_open(open)
    }
}

/// Executes the container's `onMouseDown` (`ComboboxChips.tsx:46-48`):
/// `handleInputPress(event, store, store.state.disabled)` — no ignore
/// predicate (unlike InputGroup, the container exempts nothing; the
/// outside-press predicate at the root is what keeps chips "inside").
///
/// Returns the funnel's outcome (the `handleInputPress` boolean — whether the
/// press was handled, i.e. default-prevented and the input focused/opened).
pub fn execute_chips_mouse_down(store: &ComboboxStore, event: &InputPressEvent<'_>) -> bool {
    let disabled = store.select(|s| s.disabled);
    handle_input_press(event, store, disabled, None)
}

#[cfg(test)]
mod chips_wiring_tests {
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};

    fn state(open: bool, selected_value: Value) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: Some(vec![json!("Apple"), json!("Banana")]),
            selected_value,
            open,
            mounted: false,
            transition_status: "indeterminate".into(),
            force_mounted: false,
            inline: false,
            active_index: None,
            selected_index: None,
            popup_props: Default::default(),
            list_props: Default::default(),
            input_props: Default::default(),
            trigger_props: Default::default(),
            item_props: Default::default(),
            positioner_element: None,
            list_element: None,
            popup_id: None,
            trigger_element: None,
            input_element: None,
            input_group_element: None,
            popup_side: None,
            open_method: None,
            input_inside_popup: false,
            input_owns_form_value: true,
            selection_mode: "multiple".into(),
            name: None,
            form: None,
            disabled: false,
            read_only: false,
            required: false,
            grid: false,
            virtualized: false,
            open_on_input_click: true,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal: false,
            auto_highlight: "true".into(),
            submit_on_item_click: false,
            has_input_value: false,
        }
    }

    // `ComboboxChips.test.tsx:17-39` — the role is `toolbar` only when
    // selection chips render, and absent when empty.
    #[test]
    fn role_is_toolbar_only_with_selection_chips() {
        let store =
            ComboboxStore::with_context(state(false, json!([])), ComboboxStoreContext::default());
        let derived = combobox_chips_state(&store);
        assert_eq!(chips_element_role(&derived), None);

        let store = ComboboxStore::with_context(
            state(false, json!(["a", "b"])),
            ComboboxStoreContext::default(),
        );
        let derived = combobox_chips_state(&store);
        assert_eq!(chips_element_role(&derived), Some("toolbar"));
    }

    // `ComboboxChips.test.tsx:277-309` — reopening the popup clears the
    // highlighted chip and leaves it unfocused.
    #[test]
    fn reopening_clears_the_highlighted_chip() {
        let runtime = ComboboxChipsRuntime::default();
        runtime.chips.highlighted_chip_index.set(Some(1));
        assert!(runtime.sync_open(true));
        assert_eq!(runtime.chips.highlighted_chip_index.get(), None);
        // Already clear: no reset, no notification.
        assert!(!runtime.sync_open(true));
    }

    // `ComboboxChips.test.tsx` (press claims) — the container press funnels
    // into handleInputPress with the store's disabled flag.
    #[test]
    fn mouse_down_funnels_through_handle_input_press() {
        let store = ComboboxStore::with_context(
            state(false, json!(["a"])),
            ComboboxStoreContext::default(),
        );
        let prevented = Rc::new(Cell::new(false));
        let event = InputPressEvent {
            base_ui_handler_prevented: false,
            current_target: None,
            native_event: None,
            prevent_default: Rc::clone(&prevented),
        };
        assert!(execute_chips_mouse_down(&store, &event));
        assert!(prevented.get(), "the press default-prevents");

        // A prevented Base UI handler vetoes the funnel.
        let vetoed = InputPressEvent {
            base_ui_handler_prevented: true,
            current_target: None,
            native_event: None,
            prevent_default: Rc::new(Cell::new(false)),
        };
        assert!(!execute_chips_mouse_down(&store, &vetoed));
    }

    // The chipsRef is shared by reference through the runtime handle (the
    // context-value `useMemo` shape) — navigation writes/reads one vec.
    #[test]
    fn chips_ref_is_shared_through_the_handle() {
        let runtime = ComboboxChipsRuntime::default();
        let chips_ref = Rc::clone(&runtime.chips_ref);
        chips_ref.borrow_mut().push(None);
        assert_eq!(runtime.chips_ref.borrow().len(), 1);
        let _ = RefCell::borrow(&runtime.chips_ref);
    }
}
