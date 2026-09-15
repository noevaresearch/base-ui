//! Combobox.List — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/list/ComboboxList.tsx`).
//!
//! The part renders the listbox/grid element carrying the store's
//! pre-computed `listProps` bag plus its own attribute matrix (role,
//! `aria-multiselectable`, `aria-readonly`, `tabIndex`, the floating id), the
//! Enter-to-select keydown, the keyboard-active capture pair, and the
//! CompositeList registration with the labels gate. The DOM reads arrive as
//! arguments, the commands issue through the store context — host-testable by
//! construction (the chips-wiring convention).

use web_sys::Element;

use crate::combobox::parts_util::click_highlighted_item;
use crate::combobox::store::{ComboboxStore, selectors};

/// The list element's role (`ComboboxList.tsx:78`): `grid` mode renders a
/// grid, everything else the listbox.
pub fn list_role(grid: bool) -> &'static str {
    if grid { "grid" } else { "listbox" }
}

/// `aria-multiselectable` (`ComboboxList.tsx:79`): the literal `"true"` in
/// multiple mode, absent otherwise.
pub fn aria_multiselectable(selection_mode: &str) -> Option<&'static str> {
    (selection_mode == "multiple").then_some("true")
}

/// `aria-readonly` (`ComboboxList.tsx:80-82`): set on the listbox when the
/// root is readOnly, but NOT on a grid — there the attribute describes cell
/// editability, not selection, so it is left to the combobox element.
pub fn list_aria_readonly(grid: bool, read_only: bool) -> Option<bool> {
    if !grid && read_only { Some(true) } else { None }
}

/// Whether the list falls back to registering itself as the positioner
/// element (`ComboboxList.tsx:71` — the ref array's
/// `hasPositionerContext ? null : setPositionerElement` arm): a `List`
/// rendered without a `Positioner` still publishes the element so the
/// floating engine has something to anchor measurement against.
pub fn list_owns_positioner_element(has_positioner_context: bool) -> bool {
    !has_positioner_context
}

/// Whether rendered labels are registered for typeahead/autofill
/// (`ComboboxList.tsx:115-118`): with the `items` prop the typeahead labels
/// derive from the items so they survive the list unmounting, so the
/// rendered-label registration runs only when the list is force-mounted.
pub fn register_rendered_labels(has_items: bool, force_mounted: bool) -> bool {
    !(has_items && !force_mounted)
}

/// The CompositeList wrap (`ComboboxList.tsx:111-124`): skipped only in
/// virtualized mode, where the virtualizer owns the composite registry.
pub fn wraps_composite_list(virtualized: bool) -> bool {
    !virtualized
}

/// The list's `onKeyDown` outcome (`ComboboxList.tsx:83-99`): the disabled /
/// readOnly bail, the no-highlight form-submission passthrough, and the
/// highlighted-item selection through [`click_highlighted_item`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKeydownAction {
    /// `disabled || readOnly` — the handler returns before any branch
    /// (`:84-86`).
    Ignored,
    /// Enter with `activeIndex == null` — form submission is allowed through
    /// (`:88-94`).
    AllowFormSubmission,
    /// Enter with a highlight — the event is stopped and the highlighted
    /// item clicked (`:96-97`).
    SelectHighlighted,
}

/// Executes the list's `onKeyDown` body against the real store
/// (`ComboboxList.tsx:83-99`). Only Enter reaches a branch; every other key
/// falls through the upstream `if` untouched (the [`ListKeydownAction::
/// Ignored`] outcome without the disabled meaning — the caller distinguishes
/// only through the returned action's select arm, which other keys never
/// produce).
pub fn list_keydown(
    store: &ComboboxStore,
    key: &str,
    active_index: Option<usize>,
    native_event: &web_sys::Event,
) -> ListKeydownAction {
    let state = store.get_snapshot();
    if state.disabled || state.read_only {
        return ListKeydownAction::Ignored;
    }
    if key != "Enter" {
        return ListKeydownAction::Ignored;
    }
    match active_index {
        None => ListKeydownAction::AllowFormSubmission,
        Some(active_index) => {
            click_highlighted_item(store, active_index, native_event);
            ListKeydownAction::SelectHighlighted
        }
    }
}

/// The `onKeyDownCapture` / `onPointerMoveCapture` pair
/// (`ComboboxList.tsx:100-105`): any keydown inside the list marks the
/// interaction keyboard-driven; any pointer move marks it pointer-driven.
/// The port executes the writes against the shared `keyboardActiveRef` cell.
pub fn mark_keyboard_active(store: &ComboboxStore, keyboard: bool) {
    store.context.keyboard_active_ref.set(keyboard);
}

/// The state shape the part exposes (`ComboboxList.tsx:63-65`) — the single
/// `empty` member, folded over the derived items count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboboxListState {
    /// Whether the filtered list has no items (`:42`).
    pub empty: bool,
}

/// Folds the part's state (`ComboboxList.tsx:34-42`): the store reads plus
/// the derived-items length the root's pipeline owns.
pub fn list_state(store: &ComboboxStore, filtered_items_len: usize) -> ComboboxListState {
    let _ = selectors::list_props(&store.get_snapshot());
    ComboboxListState {
        empty: filtered_items_len == 0,
    }
}

/// A minimal native event the tests synthesize: [`click_highlighted_item`]
/// only tags the native event onto the selection ref and dispatches the
/// item's click — no keydown-specific API is consumed — so a plain Event is
/// the faithful stub (the stub-event host-target convention).
pub(crate) fn stub_keydown_event() -> web_sys::Event {
    web_sys::window()
        .and_then(|w| w.document())
        .map(|d| web_sys::Event::new("keydown").expect("keydown event"))
        .expect("no DOM in the host test path")
}

#[cfg(test)]
mod list_wiring_tests {
    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use leptos_ui_utils::react_store::ReactStore;

    // The DOM-reading tests dispatch through real elements, so they carry
    // the `#[wasm_bindgen_test]` gate (the combobox_tests wasm_tests
    // convention — a plain `#[test]` compiled to wasm32 is invisible to the
    // browser runner).
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn store_with(disabled: bool, read_only: bool) -> ComboboxStore {
        let state = ComboboxState {
            id: None,
            label_id: None,
            items: None,
            selected_value: serde_json::Value::Null,
            open: true,
            mounted: true,
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
            selection_mode: "single".into(),
            name: None,
            form: None,
            disabled,
            read_only,
            required: false,
            grid: false,
            virtualized: false,
            open_on_input_click: false,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal: false,
            auto_highlight: "false".into(),
            submit_on_item_click: false,
            has_input_value: false,
        };
        ReactStore::with_context(state, ComboboxStoreContext::default())
    }

    // ------------------------------------------------------------------
    // The attribute matrix (ComboboxList.tsx:78-82)
    // ------------------------------------------------------------------

    // `ComboboxList.test.tsx:21-36` — role=listbox + aria-multiselectable in
    // multiple mode.
    #[test]
    fn the_role_and_multiselectable_matrix() {
        assert_eq!(list_role(false), "listbox");
        assert_eq!(list_role(true), "grid");
        assert_eq!(aria_multiselectable("multiple"), Some("true"));
        assert_eq!(aria_multiselectable("single"), None);
    }

    // `ComboboxList.test.tsx:38-56` + `:58-76` — the readOnly listbox
    // carries aria-readonly; the grid does not (cell editability belongs to
    // the combobox element).
    #[test]
    fn aria_readonly_skips_the_grid() {
        assert_eq!(list_aria_readonly(false, true), Some(true));
        assert_eq!(list_aria_readonly(false, false), None);
        assert_eq!(list_aria_readonly(true, true), None);
    }

    // ------------------------------------------------------------------
    // The ref fallback + the registration gates (ComboboxList.tsx:71, 111-124)
    // ------------------------------------------------------------------

    #[test]
    fn the_positioner_fallback_runs_without_a_positioner_context() {
        assert!(list_owns_positioner_element(false));
        assert!(!list_owns_positioner_element(true));
    }

    // `ComboboxList.tsx:115-118` — items-derived labels skip the rendered
    // registration unless the list is force-mounted.
    #[test]
    fn the_labels_gate_derives_from_items_and_force_mount() {
        assert!(!register_rendered_labels(true, false));
        assert!(register_rendered_labels(true, true));
        assert!(register_rendered_labels(false, false));
    }

    #[test]
    fn the_composite_wrap_skips_virtualized_mode() {
        assert!(wraps_composite_list(false));
        assert!(!wraps_composite_list(true));
    }

    // ------------------------------------------------------------------
    // The keydown body against the real store (ComboboxList.tsx:83-99)
    // ------------------------------------------------------------------

    // `ComboboxList.test.tsx:130-166` — Enter with no highlight does not
    // prevent default (the passthrough arm). The stub event needs a DOM, so
    // the keydown-family tests run in the browser.
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn enter_without_a_highlight_allows_form_submission() {
        let store = store_with(false, false);
        assert_eq!(
            list_keydown(&store, "Enter", None, &stub_keydown_event()),
            ListKeydownAction::AllowFormSubmission
        );
    }

    // `ComboboxList.test.tsx:168-196` — Enter selects the highlighted item.
    // The select path dispatches through a real element click, so it runs
    // only in the browser (the label-wiring wasm-gate convention).
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn enter_with_a_highlight_selects() {
        let store = store_with(false, false);
        assert_eq!(
            list_keydown(&store, "Enter", Some(0), &stub_keydown_event()),
            ListKeydownAction::SelectHighlighted
        );
    }

    // `ComboboxList.test.tsx:198-224` — the disabled/readOnly bail never
    // reaches a selection.
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn disabled_or_read_only_ignores_enter() {
        assert_eq!(
            list_keydown(
                &store_with(true, false),
                "Enter",
                Some(0),
                &stub_keydown_event()
            ),
            ListKeydownAction::Ignored
        );
        assert_eq!(
            list_keydown(
                &store_with(false, true),
                "Enter",
                Some(0),
                &stub_keydown_event()
            ),
            ListKeydownAction::Ignored
        );
    }

    // Non-Enter keys fall through the upstream `if` untouched.
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn other_keys_never_reach_a_branch() {
        let store = store_with(false, false);
        assert_eq!(
            list_keydown(&store, "ArrowDown", Some(0), &stub_keydown_event()),
            ListKeydownAction::Ignored
        );
    }

    // ------------------------------------------------------------------
    // The keyboard-active capture pair (ComboboxList.tsx:100-105)
    // ------------------------------------------------------------------

    #[test]
    fn the_capture_pair_drives_the_keyboard_flag() {
        let store = store_with(false, false);
        assert!(!store.context.keyboard_active_ref.get());
        mark_keyboard_active(&store, true);
        assert!(store.context.keyboard_active_ref.get());
        mark_keyboard_active(&store, false);
        assert!(!store.context.keyboard_active_ref.get());
    }

    // ------------------------------------------------------------------
    // The state fold (ComboboxList.tsx:63-65)
    // ------------------------------------------------------------------

    #[test]
    fn the_empty_state_folds_the_filtered_count() {
        let store = store_with(false, false);
        assert!(list_state(&store, 0).empty);
        assert!(!list_state(&store, 2).empty);
    }
}
