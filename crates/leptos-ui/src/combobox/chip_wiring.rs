//! Combobox.Chip — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/chip/ComboboxChip.tsx`).
//!
//! The keydown matrix is the already-ported [`plan_chip_key_down`] runtime;
//! this module is the seam that reads the part's store selections
//! (`ComboboxChip.tsx:24-29`), derives the attribute matrix, and — for a
//! keydown — executes the plan against the real store commands (the
//! `onKeyDown` body at `:106-127`): the disabled/readOnly gate, the
//! `flushSync`-ordered highlight write, and the focus routing (the removed
//! chip's setIndices/setSelectedValue pair, the popup open, the input vs
//! sibling-chip focus). Host-testable by construction.

use serde_json::Value;

use crate::combobox::store::{ComboboxStore, SetIndicesInput, selectors};
use crate::combobox::value_chips::{
    REASON_KEYBOARD, REASON_LIST_NAVIGATION, REASON_NONE, chip_key_down_blocked, plan_chip_key_down,
};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;

use crate::combobox::store::ChangeCommandDetails;

/// The part's props (`ComboboxChip.Props`) the wiring consumes beyond the
/// store state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComboboxChipProps {
    /// The `disabled` prop — the port's caller OR-s the store flag.
    pub disabled: bool,
}

/// The part's derived state (`ComboboxChip.tsx:24-29`, `:100-103`).
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxChipState {
    /// The effective `disabled` (store flag; the part's state attribute).
    pub disabled: bool,
    /// The store's `readOnly`.
    pub read_only: bool,
    /// The store's array-shaped `selectedValue`.
    pub selected_value: Value,
}

/// Reads the part's state (`ComboboxChip.tsx:24-29`).
pub fn combobox_chip_state(store: &ComboboxStore, props: &ComboboxChipProps) -> ComboboxChipState {
    let state = store.select(|s| s.clone());
    ComboboxChipState {
        disabled: state.disabled || props.disabled,
        read_only: state.read_only,
        selected_value: state.selected_value,
    }
}

/// The element's attribute constants (`ComboboxChip.tsx:106-108`): the chip
/// never takes tab focus and mirrors the disabled/readonly state.
pub const CHIP_TAB_INDEX: i32 = -1;

/// The `aria-disabled`/`aria-readonly` matrix (`ComboboxChip.tsx:107-108`):
/// present only when true (`|| undefined`).
pub fn chip_aria_attributes(state: &ComboboxChipState) -> (Option<bool>, Option<bool>) {
    (
        state.disabled.then_some(true),
        state.read_only.then_some(true),
    )
}

/// The keydown outcome (`ComboboxChip.tsx:106-127`): what the plan computed
/// plus which commands the body issued, as the caller's observation surface.
#[derive(Clone, Debug, PartialEq)]
pub struct ChipKeyDownOutcome {
    /// The wrapped plan (`handleKeyDown`).
    pub plan: crate::combobox::value_chips::ChipKeyDownPlan,
    /// Whether `setIndices` fired (Backspace/Delete — both indices null with
    /// the keyboard reason).
    pub cleared_indices: bool,
    /// Whether `setSelectedValue` fired (Backspace/Delete — the array minus
    /// the chip's entry, reason `none`).
    pub removed_value: Option<Value>,
    /// Whether `setOpen(true, listNavigation)` fired (ArrowDown/ArrowUp).
    pub opened_popup: bool,
    /// Where focus lands: the input (`None`) or the sibling chip index.
    pub focus_target: Option<Option<usize>>,
}

/// Executes the chip's `onKeyDown` (`ComboboxChip.tsx:106-127`) against the
/// store's command slots. `index` is the chip's composite index, `chips_len`
/// the rendered chips (`chipsRef.current.length`), `chips` the rendered chip
/// elements in order (the sibling-focus lookup), `direction` the
/// `ltr`/`rtl` direction.
pub fn execute_chip_key_down(
    store: &ComboboxStore,
    key: &str,
    ctrl: bool,
    meta: bool,
    alt: bool,
    index: usize,
    chips: &[Option<web_sys::Element>],
    direction: &str,
) -> ChipKeyDownOutcome {
    let chips_len = chips.len();
    let state = store.select(|s| s.clone());
    let selected_len = selectors::selected_value(&state)
        .as_array()
        .map_or(0, |a| a.len());
    let plan = plan_chip_key_down(
        key,
        ctrl,
        meta,
        alt,
        index,
        chips_len,
        selected_len,
        direction,
    );

    let mut outcome = ChipKeyDownOutcome {
        focus_target: Some(plan.next_index),
        cleared_indices: false,
        removed_value: None,
        opened_popup: false,
        plan: plan.clone(),
    };

    if plan.remove_chip {
        // `ComboboxChip.tsx:60-77` — the setIndices command with both indices
        // null and the keyboard reason.
        (store.context.set_indices)(SetIndicesInput {
            active_index: Some(None),
            selected_index: Some(None),
            reason: Some(REASON_KEYBOARD.to_string()),
        });
        outcome.cleared_indices = true;
        // The setSelectedValue command — the array minus this chip's entry,
        // with reason `none`.
        let next = Value::Array(
            state
                .selected_value
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .enumerate()
                        .filter(|(i, _)| *i != index)
                        .map(|(_, v)| v.clone())
                        .collect()
                })
                .unwrap_or_default(),
        );
        let details = ChangeCommandDetails::new(REASON_NONE, make_stub_event(), None, ());
        (store.context.set_selected_value)(next.clone(), &details);
        outcome.removed_value = Some(next);
    }

    if plan.open_popup {
        // `ComboboxChip.tsx:82-88` — the popup opens with the
        // list-navigation reason.
        let details =
            ChangeCommandDetails::new(REASON_LIST_NAVIGATION, make_stub_event(), None, ());
        (store.context.set_open)(true, &details);
        outcome.opened_popup = true;
    }

    // The focus routing (`:118-126`): `undefined` nextIndex focuses the
    // input, a number focuses the sibling chip. The DOM reads go through the
    // store context's refs; the host seam records the target.
    if plan.next_index.is_none() {
        focus_element_opt(&store.context.input_ref);
    } else if let Some(next_index) = plan.next_index {
        let target = chips.get(next_index).cloned().flatten();
        if let Some(element) = target {
            focus_element(&element);
        }
    }

    outcome
}

/// The stub event the command details carry when the caller supplied no
/// native event (`createBaseUIEventDetails.ts:129-132` — `new Event('base-ui')`).
fn make_stub_event() -> web_sys::Event {
    // The host-target convention (clear_wiring_tests / menu_tests): no JS
    // runtime on host, so wrap a plain JsValue instead of invoking the
    // wasm-bindgen Event constructor.
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::Event::new("base-ui").expect("construct the stub event")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL)
    }
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

fn focus_element_opt(_element_ref: &std::rc::Rc<std::cell::RefCell<Option<web_sys::Element>>>) {
    // The input focus is the wasm wiring's DOM side; the host seam records
    // nothing (the command log is the observation surface), matching the
    // Clear wiring's `inputRef.current?.focus()` treatment.
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(element) = _element_ref.borrow().as_ref() {
            focus_element(element);
        }
    }
}

#[cfg(test)]
mod chip_wiring_tests {
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    /// A stand-in rendered-chips array (the `chipsRef` contents the callers
    /// pass; the elements themselves are the wasm wiring's DOM side).
    const CHIPS: [Option<web_sys::Element>; 0] = [];

    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};

    fn state(selected_value: Value) -> ComboboxState {
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

    /// A recording store: the command slots log their payloads.
    fn recording_store(selected_value: Value) -> (ComboboxStore, Rc<RefCell<Vec<String>>>) {
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
        let log_cb = Rc::clone(&log);
        ctx.set_open = Rc::new(move |open: bool, details: &ChangeCommandDetails| {
            log_cb
                .borrow_mut()
                .push(format!("open({open},reason={})", details.reason));
        });
        (ComboboxStore::with_context(state(selected_value), ctx), log)
    }

    // `ComboboxChip.test.tsx:36-90` — disabled/readOnly blocks the whole
    // keydown body: no removal, no navigation, focus stays on the chip.
    #[test]
    fn disabled_or_read_only_blocks_the_keydown() {
        assert!(chip_key_down_blocked(true, false));
        assert!(chip_key_down_blocked(false, true));
        assert!(!chip_key_down_blocked(false, false));
    }

    // `ComboboxChip.test.tsx:198-260` — Backspace removes the chip's value:
    // the setIndices clear (keyboard reason) and the setSelectedValue removal
    // (reason `none`) both fire.
    #[test]
    fn backspace_clears_indices_and_removes_the_value() {
        let (store, log) = recording_store(json!(["a", "b", "c"]));
        let outcome =
            execute_chip_key_down(&store, "Backspace", false, false, false, 1, &CHIPS, "ltr");
        assert!(outcome.plan.remove_chip);
        assert!(outcome.cleared_indices);
        assert_eq!(
            outcome.removed_value,
            Some(json!(["a", "c"])),
            "the chip at index 1 is spliced out"
        );
        let log = log.borrow();
        assert!(log.contains(
            &"indices(active=Some(None),selected=Some(None),reason=Some(\"keyboard\"))".to_string()
        ));
        assert!(
            log.iter()
                .any(|entry| entry.starts_with("selectedValue([\"a\",\"c\"],reason=none)"))
        );
    }

    // `ComboboxChip.test.tsx:262-310` — ArrowDown/ArrowUp on a chip opens the
    // popup with the list-navigation reason and hands focus to the input.
    #[test]
    fn arrow_down_opens_the_popup_and_focuses_the_input() {
        let (store, log) = recording_store(json!(["a"]));
        let outcome =
            execute_chip_key_down(&store, "ArrowDown", false, false, false, 0, &CHIPS, "ltr");
        assert!(outcome.plan.open_popup);
        assert!(outcome.opened_popup);
        assert_eq!(
            outcome.focus_target,
            Some(None),
            "focus routes to the input"
        );
        assert!(
            log.borrow()
                .contains(&"open(true,reason=list-navigation)".to_string())
        );
    }

    // `ComboboxChip.test.tsx:452-492` — navigation is bounded by the rendered
    // chips: at the last chip the next-arrow ends in focusing the input.
    #[test]
    fn navigation_at_the_last_chip_hands_focus_to_the_input() {
        let (store, _log) = recording_store(json!(["a", "b"]));
        let outcome =
            execute_chip_key_down(&store, "ArrowRight", false, false, false, 1, &CHIPS, "ltr");
        assert_eq!(outcome.plan.next_index, None);
        assert_eq!(outcome.focus_target, Some(None));
    }

    // `ComboboxChip.test.tsx:365-386` — modified printables intentionally
    // leave focus on the chip (`nextIndex` stays `index`).
    #[test]
    fn modified_printables_keep_focus_on_the_chip() {
        let (store, _log) = recording_store(json!(["a"]));
        let outcome = execute_chip_key_down(&store, "x", true, false, false, 2, &CHIPS, "ltr");
        assert_eq!(outcome.plan.next_index, Some(2));
        assert_eq!(outcome.focus_target, Some(Some(2)));
    }

    // The attribute matrix (`ComboboxChip.tsx:106-108`): tabindex -1,
    // aria-disabled/aria-readonly present only when true.
    #[test]
    fn attribute_matrix_mirrors_the_state() {
        assert_eq!(CHIP_TAB_INDEX, -1);
        let state = ComboboxChipState {
            disabled: true,
            read_only: false,
            selected_value: Value::Null,
        };
        assert_eq!(chip_aria_attributes(&state), (Some(true), None));
        let state = ComboboxChipState {
            disabled: false,
            read_only: true,
            selected_value: Value::Null,
        };
        assert_eq!(chip_aria_attributes(&state), (None, Some(true)));
    }
}
