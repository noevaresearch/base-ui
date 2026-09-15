//! Combobox.ChipRemove — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/chip-remove/ComboboxChipRemove.tsx`).
//!
//! The removal plan (the active-index clear walk, the array splice, the
//! propagation decision) is the already-ported [`plan_chip_remove`] runtime;
//! this module is the seam that reads the part's store selections
//! (`ComboboxChipRemove.tsx:24-28`), derives the effective disabled gate, and
//! executes the `removeChip` body (`:78-94`) against the real store commands:
//! the conditional setIndices clear, the setSelectedValue splice, and the
//! trailing input focus. Host-testable by construction.

use serde_json::Value;

use crate::combobox::store::{ComboboxStore, SetIndicesInput};
use crate::combobox::value_chips::{
    ChipRemovePlan, plan_chip_remove, plan_chip_remove_propagation,
};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;

use crate::combobox::store::ChangeCommandDetails;

/// The part's props (`ComboboxChipRemove.Props`) the wiring consumes beyond
/// the store state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComboboxChipRemoveProps {
    /// The `disabled` prop (`:16`).
    pub disabled: bool,
}

/// The part's derived state (`ComboboxChipRemove.tsx:24-33`).
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxChipRemoveState {
    /// The effective `disabled`: `comboboxDisabled || disabledProp`.
    pub disabled: bool,
    /// The store's `readOnly` (OR-ed into the useButton gate, `:36-40`).
    pub read_only: bool,
}

/// Reads the part's state (`ComboboxChipRemove.tsx:24-33`).
pub fn combobox_chip_remove_state(
    store: &ComboboxStore,
    props: &ComboboxChipRemoveProps,
) -> ComboboxChipRemoveState {
    let state = store.select(|s| s.clone());
    ComboboxChipRemoveState {
        disabled: state.disabled || props.disabled,
        read_only: state.read_only,
    }
}

/// The element's constant wiring (`ComboboxChipRemove.tsx:98-99`): the remove
/// button never takes tab focus, and its mousedown default-prevents so the
/// press never steals focus from the input.
pub const CHIP_REMOVE_TAB_INDEX: i32 = -1;
/// The mousedown default-prevention flag (`:100-102`).
pub const CHIP_REMOVE_MOUSE_DOWN_PREVENTED: bool = true;

/// The removal outcome (`ComboboxChipRemove.tsx:78-94` plus the propagation
/// handling at `:103-121`): what the plan computed plus which commands the
/// body issued.
#[derive(Clone, Debug, PartialEq)]
pub struct ChipRemoveOutcome {
    /// The wrapped plan.
    pub plan: ChipRemovePlan,
    /// Whether the conditional setIndices clear fired (`:64-85`).
    pub cleared_active_index: bool,
    /// The selected value the removal committed (`:88-91`).
    pub removed_value: Option<Value>,
    /// Whether the originating event's propagation was stopped
    /// (`:103-105`, `:113-115`).
    pub propagation_stopped: bool,
}

/// Executes the `removeChip` body (`ComboboxChipRemove.tsx:78-94`) against
/// the store's command slots. `index` is the chip's index into the selected
/// array (from the Chip context), `values` the `valuesRef.current` list,
/// `keyboard_active` the `keyboardActiveRef` flag, `event` the originating
/// native event the details carry.
pub fn execute_chip_remove(
    store: &ComboboxStore,
    props: &ComboboxChipRemoveProps,
    index: usize,
    values: &[Value],
    keyboard_active: bool,
    event: web_sys::Event,
) -> ChipRemoveOutcome {
    let state = store.select(|s| s.clone());
    let derived = combobox_chip_remove_state(store, props);
    let selected = state.selected_value.as_array().cloned().unwrap_or_default();

    let plan = plan_chip_remove(
        index,
        &selected,
        state.active_index,
        values,
        state.is_item_equal_to_value.as_ref(),
        keyboard_active,
    );

    let details = ChangeCommandDetails::new(
        crate::combobox::value_chips::REASON_CHIP_REMOVE_PRESS,
        event,
        None,
        (),
    );

    let mut outcome = ChipRemoveOutcome {
        cleared_active_index: false,
        removed_value: None,
        propagation_stopped: false,
        plan: plan.clone(),
    };

    if plan.clear_active_index {
        (store.context.set_indices)(SetIndicesInput {
            active_index: Some(None),
            selected_index: None,
            reason: Some(plan.active_index_reason.to_string()),
        });
        outcome.cleared_active_index = true;
    }

    // The disabled/readOnly gate on the useButton (`:36-40`) — the removal
    // itself is the button's activation, which the gate suppresses.
    if derived.disabled || derived.read_only {
        return outcome;
    }

    (store.context.set_selected_value)(plan.next_selected_value.clone(), &details);
    outcome.removed_value = Some(plan.next_selected_value.clone());

    // The trailing input focus (`:92`).
    if let Some(input) = store.context.input_ref.borrow().as_ref() {
        focus_element(input);
    }

    // The propagation handling (`:103-105`, `:113-115`): blocked unless the
    // user's onValueChange called `details.allowPropagation()` — the port
    // reads the details' flag after the command ran (the user callback is the
    // command slot's inner callee).
    outcome.propagation_stopped = !plan_chip_remove_propagation(details.is_propagation_allowed());

    outcome
}

fn focus_element(element: &web_sys::Element) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::wasm_bindgen::JsCast;
        if let Some(focusable) = element.dyn_ref::<web_sys::HtmlElement>() {
            focusable.focus().ok();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = element;
    }
}

#[cfg(test)]
mod chip_remove_wiring_tests {
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};

    fn state(selected_value: Value, active_index: Option<usize>) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: Some(vec![json!("Apple"), json!("Banana")]),
            selected_value,
            open: false,
            mounted: false,
            transition_status: "indeterminate".into(),
            force_mounted: false,
            inline: false,
            active_index,
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

    /// A recording store: the command slots log their payloads.
    fn recording_store(
        selected_value: Value,
        active_index: Option<usize>,
    ) -> (ComboboxStore, Rc<RefCell<Vec<String>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut ctx = ComboboxStoreContext::default();
        let log_cb = Rc::clone(&log);
        ctx.set_selected_value = Rc::new(move |value: Value, details: &ChangeCommandDetails| {
            log_cb
                .borrow_mut()
                .push(format!("selectedValue({value},reason={})", details.reason));
        });
        let log_cb = Rc::clone(&log);
        ctx.set_indices = Rc::new(move |input: SetIndicesInput| {
            log_cb.borrow_mut().push(format!(
                "indices(active={:?},selected={:?},reason={:?})",
                input.active_index, input.selected_index, input.reason
            ));
        });
        (
            ComboboxStore::with_context(state(selected_value, active_index), ctx),
            log,
        )
    }

    fn event() -> web_sys::Event {
        // The host-target convention (clear_wiring_tests / menu_tests): no JS
        // runtime on host, so wrap a plain JsValue instead of invoking the
        // wasm-bindgen Event constructor. The recording slots never touch it.
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL)
    }

    // `ComboboxChipRemove.test.tsx:60-118` — the click removes the chip's
    // value: the selected array minus the entry, with the chip-remove-press
    // reason, plus the trailing input focus.
    #[test]
    fn click_removes_the_chip_value() {
        let (store, log) = recording_store(json!(["a", "b", "c"]), None);
        let outcome = execute_chip_remove(
            &store,
            &ComboboxChipRemoveProps::default(),
            1,
            &[],
            false,
            event(),
        );
        assert_eq!(outcome.removed_value, Some(json!(["a", "c"])));
        assert!(log.borrow().iter().any(|entry| {
            entry.starts_with("selectedValue([\"a\",\"c\"],reason=chip-remove-press)")
        }));
    }

    // `ComboboxChipRemove.test.tsx:120-190` — when the removed item is the
    // currently active one in the visible list, the active index clears with
    // the pointer reason (keyboard-active flips it to `keyboard`).
    #[test]
    fn removing_the_active_item_clears_the_active_index() {
        let (store, log) = recording_store(json!(["a", "b"]), Some(0));
        let outcome = execute_chip_remove(
            &store,
            &ComboboxChipRemoveProps::default(),
            0,
            &[json!("a"), json!("b")],
            false,
            event(),
        );
        assert!(outcome.cleared_active_index);
        assert!(log.borrow().contains(
            &"indices(active=Some(None),selected=None,reason=Some(\"pointer\"))".to_string()
        ));

        let (store, log) = recording_store(json!(["a", "b"]), Some(0));
        let _ = execute_chip_remove(
            &store,
            &ComboboxChipRemoveProps::default(),
            0,
            &[json!("a"), json!("b")],
            true,
            event(),
        );
        assert!(log.borrow().contains(
            &"indices(active=Some(None),selected=None,reason=Some(\"keyboard\"))".to_string()
        ));
    }

    // The filtered-out removed item can't equal the active index — no clear.
    #[test]
    fn removing_a_filtered_out_item_does_not_clear_the_active_index() {
        let (store, log) = recording_store(json!(["a", "b"]), Some(0));
        let outcome = execute_chip_remove(
            &store,
            &ComboboxChipRemoveProps::default(),
            1,
            &[json!("a")],
            false,
            event(),
        );
        assert!(!outcome.cleared_active_index);
        // Only the removal commits — no indices command.
        assert!(
            log.borrow()
                .iter()
                .all(|entry| entry.starts_with("selectedValue("))
        );
    }

    // The disabled/readOnly gate: the selected-value command never fires.
    #[test]
    fn disabled_or_read_only_suppresses_the_removal() {
        for props in [
            ComboboxChipRemoveProps { disabled: true },
            ComboboxChipRemoveProps::default(),
        ] {
            let (store, log) = recording_store(json!(["a"]), None);
            if props.disabled {
                let _ = execute_chip_remove(&store, &props, 0, &[], false, event());
                assert!(log.borrow().is_empty());
            } else {
                // readOnly comes from the store; flip it through a patched state.
                let mut s = state(json!(["a"]), None);
                s.read_only = true;
                let mut ctx = ComboboxStoreContext::default();
                let log_cb = Rc::clone(&log);
                ctx.set_selected_value = Rc::new(move |_: Value, _: &ChangeCommandDetails| {
                    log_cb.borrow_mut().push("removed".into());
                });
                let store = ComboboxStore::with_context(s, ctx);
                let _ = execute_chip_remove(&store, &props, 0, &[], false, event());
                assert!(log.borrow().is_empty());
            }
        }
    }
}

// The element-touching trailing-focus test — a wasm-bindgen browser test
// module (the clear_wasm_tests convention): the removal's trailing
// `inputRef.current?.focus()` (`ComboboxChipRemove.tsx:92`) is a no-op on the
// host seam, so the focus-observing arm of the upstream suite
// (`ComboboxChipRemove.test.tsx:181`, "should focus input after removing
// chip") can only run in a real DOM.
#[cfg(all(test, target_arch = "wasm32"))]
mod chip_remove_wasm_tests {
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    fn state(selected_value: Value, active_index: Option<usize>) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: Some(vec![json!("Apple"), json!("Banana")]),
            selected_value,
            open: false,
            mounted: false,
            transition_status: "indeterminate".into(),
            force_mounted: false,
            inline: false,
            active_index,
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

    fn real_event() -> web_sys::Event {
        web_sys::MouseEvent::new("click")
            .unwrap()
            .dyn_into::<web_sys::Event>()
            .unwrap()
    }

    // `ComboboxChipRemove.test.tsx:181` — the removal focuses the input: the
    // value splice plus the trailing `inputRef.current?.focus()`, both
    // observable in a real DOM.
    #[wasm_bindgen_test]
    fn removal_focuses_the_input_after_the_value_commit() {
        let document = web_sys::window().unwrap().document().unwrap();
        let (log_tx, log) = {
            let log = Rc::new(RefCell::new(Vec::new()));
            (Rc::clone(&log), log)
        };
        let mut ctx = ComboboxStoreContext::default();
        let log_cb = Rc::clone(&log_tx);
        ctx.set_selected_value = Rc::new(move |value: Value, details: &ChangeCommandDetails| {
            log_cb
                .borrow_mut()
                .push(format!("selectedValue({value},reason={})", details.reason));
        });
        let input: web_sys::Element = document.create_element("input").unwrap().into();
        document.body().unwrap().append_child(&input).unwrap();
        *ctx.input_ref.borrow_mut() = Some(input.clone());
        let store = ComboboxStore::with_context(state(json!(["a", "b"]), None), ctx);

        let outcome = execute_chip_remove(
            &store,
            &ComboboxChipRemoveProps::default(),
            1,
            &[],
            false,
            real_event(),
        );
        assert_eq!(outcome.removed_value, Some(json!(["a"])));
        assert!(
            log.borrow()
                .iter()
                .any(|entry| entry.starts_with("selectedValue([\"a\"],reason=chip-remove-press)"))
        );
        let active = document.active_element().unwrap();
        assert!(
            active.dyn_ref::<web_sys::HtmlInputElement>().is_some(),
            "the input holds focus after the removal"
        );
    }

    // The disabled gate suppresses BOTH the removal and the trailing focus:
    // the early return (`:36-40`) precedes the `:92` focus.
    #[wasm_bindgen_test]
    fn disabled_removal_never_touches_the_dom() {
        let document = web_sys::window().unwrap().document().unwrap();
        let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let mut ctx = ComboboxStoreContext::default();
        let log_cb = Rc::clone(&log);
        ctx.set_selected_value = Rc::new(move |_: Value, _: &ChangeCommandDetails| {
            log_cb.borrow_mut().push("removed".into());
        });
        let input: web_sys::Element = document.create_element("input").unwrap().into();
        document.body().unwrap().append_child(&input).unwrap();
        *ctx.input_ref.borrow_mut() = Some(input);
        let store = ComboboxStore::with_context(state(json!(["a"]), None), ctx);

        // The suite shares one page: park focus on a scratch element so the
        // assertion below observes THIS test's DOM effect, not the previous
        // test's trailing focus.
        let scratch: web_sys::Element = document.create_element("div").unwrap().into();
        scratch.set_attribute("tabindex", "-1").unwrap();
        document.body().unwrap().append_child(&scratch).unwrap();
        scratch
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .focus()
            .unwrap();

        let outcome = execute_chip_remove(
            &store,
            &ComboboxChipRemoveProps { disabled: true },
            0,
            &[],
            false,
            real_event(),
        );
        assert!(outcome.removed_value.is_none());
        assert!(log.borrow().is_empty());
        assert!(
            !document
                .active_element()
                .unwrap()
                .dyn_ref::<web_sys::HtmlInputElement>()
                .is_some(),
            "the blocked removal never focuses the input"
        );
    }
}
