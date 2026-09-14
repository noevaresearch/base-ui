//! Combobox.Clear — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/clear/ComboboxClear.tsx`).
//!
//! The behavior layer (visibility per selection mode, the click plan, the
//! disabled gate) is the already-ported [`plan_clear_click`]/[`clear_visible`]
//! runtime; this module is the seam that reads the store selections the part
//! makes (`ComboboxClear.tsx:55-69`), derives the visible/disabled state, and
//! — for the click — executes the plan against the real store commands
//! (the `onClick` body at `:97-118`). The DOM reads (the raw input value from
//! the input-value context, the keyboard-active ref) arrive as arguments, the
//! commands issue through the store context — host-testable by construction.

use std::rc::Rc;

use serde_json::Value;

use crate::combobox::store::{ComboboxStore, SetIndicesInput, selectors};
use crate::combobox::value_chips::{
    ClearClickPlan, REASON_CLEAR_PRESS, clear_visible, plan_clear_click,
};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;

use crate::combobox::store::ChangeCommandDetails;

/// The part's props (`ComboboxClear.Props`, `ComboboxClear.tsx:38-44`) the
/// wiring consumes beyond the store state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComboboxClearProps {
    /// The `disabled` prop (`:41`) — OR-ed into the effective gate.
    pub disabled: bool,
    /// The `keepMounted` prop (`:43`) — forces the render gate.
    pub keep_mounted: bool,
}

/// The part's derived state (`ComboboxClear.tsx:47-77`): what the element's
/// state attributes and the render gate read.
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxClearState {
    /// The effective `disabled` (`:71`): fieldDisabled || comboboxDisabled ||
    /// disabledProp. The port carries the two store-origin gates as arguments'
    /// product — the store's own flags plus the prop.
    pub disabled: bool,
    /// The `visible` derivation (`:57-67`).
    pub visible: bool,
    /// The store's `open` (`:62`).
    pub open: bool,
}

/// Reads the part's state (`ComboboxClear.tsx:55-77`). `field_disabled` is the
/// `useFieldRootContext` gate — the port's Field integration passes it;
/// `input_value` the raw input-value context read (`:68`).
pub fn combobox_clear_state(
    store: &ComboboxStore,
    props: &ComboboxClearProps,
    field_disabled: bool,
    input_value: &str,
) -> ComboboxClearState {
    let state = store.select(|s| s.clone());
    let visible = clear_visible(
        &state.selection_mode,
        input_value,
        &state.selected_value,
        selectors::has_selection_chips(&state),
    );
    ComboboxClearState {
        disabled: field_disabled || state.disabled || props.disabled,
        visible,
        open: state.open,
    }
}

/// The render gate (`ComboboxClear.tsx:119-123` upstream analog — the
/// transition machinery's `mounted`): render when `keepMounted` or mounted.
/// The port's caller passes its transition-mounted flag.
pub fn clear_render_gate(props: &ComboboxClearProps, mounted: bool) -> bool {
    props.keep_mounted || mounted
}

/// The element's constant wiring (`ComboboxClear.tsx:87-95`): `tabIndex: -1`
/// (never steals focus from the input) and the mousedown `preventDefault`.
pub const CLEAR_TAB_INDEX: i32 = -1;
/// The mousedown default-prevention flag (`:90-92`).
pub const CLEAR_MOUSE_DOWN_PREVENTED: bool = true;

/// Executes the Clear click (`ComboboxClear.tsx:97-118`) against the store's
/// command slots: the disabled/readOnly block, the input-value clear, the
/// selected-value clear (array vs null), the indices command with the
/// selected-index split, and the trailing input focus.
///
/// Returns the executed plan (the caller's test seam — the commands are the
/// store context's NOOP seeds until the root assigns them, so tests swap in
/// recording closures on the context, exactly the root-runtime tests' pattern).
pub fn execute_clear_click(
    store: &ComboboxStore,
    input_value: &str,
    keyboard_active: bool,
    event: web_sys::Event,
) -> ClearClickPlan {
    let state = store.select(|s| s.clone());
    let plan = plan_clear_click(
        state.disabled,
        state.read_only,
        &state.selection_mode,
        &state.selected_value,
        keyboard_active,
    );
    if plan.blocked {
        return plan;
    }

    let details =
        BaseUIChangeEventDetails::<(), web_sys::Event>::new(REASON_CLEAR_PRESS, event, None, ());

    if let Some(value) = &plan.input_value {
        (store.context.set_input_value)(value.clone(), &details);
    }
    if let Some(next) = &plan.next_selected_value {
        (store.context.set_selected_value)(next.clone(), &details);
    }
    // The indices command (`:111-114`): both indices when a selection mode is
    // active, the active index only in `none` mode.
    (store.context.set_indices)(SetIndicesInput {
        active_index: Some(None),
        selected_index: plan.clear_selected_index.then_some(None),
        reason: Some(plan.indices_reason.to_string()),
    });
    // The trailing input focus (`:117`) — the caller's DOM side; the wasm
    // wiring focuses the element the store context carries.
    if let Some(input) = store.context.input_ref.borrow().as_ref() {
        focus_element(input);
    }

    plan
}

/// The `inputRef.current?.focus()` call (`:117`), factored for the host tests
/// (an `Element` focus is wasm-only; the host seam no-ops on the shared element
/// handle — the recording slots are the tests' observation surface).
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

/// Convenience re-export guard: the keyboard/pointer reason split lives in the
/// runtime; this alias keeps the wiring's public surface self-describing.
pub fn clear_indices_reason(keyboard_active: bool) -> &'static str {
    if keyboard_active {
        crate::combobox::value_chips::REASON_KEYBOARD
    } else {
        crate::combobox::value_chips::REASON_POINTER
    }
}

#[cfg(test)]
mod clear_wiring_tests {
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use crate::combobox::value_chips::REASON_POINTER;

    fn state(selection_mode: &str, selected_value: Value) -> ComboboxState {
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
            selection_mode: selection_mode.into(),
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
        selection_mode: &str,
        selected_value: Value,
    ) -> (ComboboxStore, Rc<RefCell<Vec<String>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log_cb = Rc::clone(&log);
        let mut ctx = ComboboxStoreContext::default();
        ctx.set_input_value = Rc::new(move |value: String, details: &ChangeCommandDetails| {
            log_cb
                .borrow_mut()
                .push(format!("inputValue({value:?},reason={})", details.reason));
        });
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
        let store = ComboboxStore::with_context(state(selection_mode, selected_value), ctx);
        (store, log)
    }

    fn fake_event() -> web_sys::Event {
        // The host-target convention (menu_tests.rs / dialog_tests.rs): no JS
        // runtime, so wrap a plain JsValue instead of invoking the wasm-bindgen
        // Event constructor. The recording slots never touch the event.
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL)
    }

    // `ComboboxClear.test.tsx` — visibility per mode.
    #[test]
    fn visible_follows_the_selection_mode() {
        let (store, _log) = recording_store("single", Value::Null);
        let props = ComboboxClearProps::default();
        assert!(!combobox_clear_state(&store, &props, false, "").visible);

        let (store, _log) = recording_store("single", json!("Apple"));
        assert!(combobox_clear_state(&store, &props, false, "").visible);

        // `none` mode: typed input drives visibility.
        let (store, _log) = recording_store("none", Value::Null);
        assert!(!combobox_clear_state(&store, &props, false, "").visible);
        assert!(combobox_clear_state(&store, &props, false, "typed").visible);

        // `multiple`: any chip.
        let (store, _log) = recording_store("multiple", json!([]));
        assert!(!combobox_clear_state(&store, &props, false, "").visible);
        let (store, _log) = recording_store("multiple", json!(["Apple"]));
        assert!(combobox_clear_state(&store, &props, false, "").visible);
    }

    // `ComboboxClear.test.tsx:180-207` — the disabled/readOnly gate blocks
    // everything.
    #[test]
    fn disabled_or_read_only_blocks_the_click() {
        let (store, log) = recording_store("single", json!("Apple"));
        store.set_field(|s| &mut s.disabled, true);
        let plan = execute_clear_click(&store, "", false, fake_event());
        assert!(plan.blocked);
        assert!(
            log.borrow().is_empty(),
            "no commands fired: {:?}",
            log.borrow()
        );

        let (store, log) = recording_store("single", json!("Apple"));
        store.set_field(|s| &mut s.read_only, true);
        let plan = execute_clear_click(&store, "", false, fake_event());
        assert!(plan.blocked);
        assert!(log.borrow().is_empty());

        // The prop gate OR-s in via the state derivation.
        let (store, _log) = recording_store("single", json!("Apple"));
        let st = combobox_clear_state(
            &store,
            &ComboboxClearProps {
                disabled: true,
                keep_mounted: false,
            },
            false,
            "",
        );
        assert!(st.disabled);
    }

    // `ComboboxClear.tsx:103-114` — the command sequence in `single` mode.
    #[test]
    fn single_mode_click_clears_input_value_and_null_selection() {
        let (store, log) = recording_store("single", json!("Apple"));
        let plan = execute_clear_click(&store, "App", false, fake_event());
        assert!(!plan.blocked);
        assert_eq!(
            log.borrow().clone(),
            vec![
                "inputValue(\"\",reason=clear-press)".to_string(),
                "selectedValue(null,reason=clear-press)".to_string(),
                "indices(active=Some(None),selected=Some(None),reason=Some(\"pointer\"))"
                    .to_string(),
            ]
        );
    }

    // `ComboboxClear.tsx:107-110` — array selections clear to `[]`.
    #[test]
    fn multiple_mode_click_clears_to_the_empty_array() {
        let (store, log) = recording_store("multiple", json!(["Apple", "Banana"]));
        let plan = execute_clear_click(&store, "", false, fake_event());
        assert!(!plan.blocked);
        assert_eq!(
            log.borrow().clone(),
            vec![
                "inputValue(\"\",reason=clear-press)".to_string(),
                "selectedValue([],reason=clear-press)".to_string(),
                "indices(active=Some(None),selected=Some(None),reason=Some(\"pointer\"))"
                    .to_string(),
            ]
        );
    }

    // `ComboboxClear.tsx:111-114` — `none` mode clears only the active index.
    #[test]
    fn none_mode_click_leaves_the_selection_alone() {
        let (store, log) = recording_store("none", Value::Null);
        let plan = execute_clear_click(&store, "typed", false, fake_event());
        assert!(!plan.blocked);
        assert!(plan.next_selected_value.is_none());
        assert_eq!(
            log.borrow().clone(),
            vec![
                "inputValue(\"\",reason=clear-press)".to_string(),
                "indices(active=Some(None),selected=None,reason=Some(\"pointer\"))".to_string(),
            ]
        );
    }

    // `ComboboxClear.tsx:101-102` — the keyboard/pointer reason split.
    #[test]
    fn keyboard_active_uses_the_keyboard_reason() {
        let (store, log) = recording_store("single", json!("Apple"));
        let plan = execute_clear_click(&store, "", true, fake_event());
        assert_eq!(plan.indices_reason, "keyboard");
        assert!(log.borrow().iter().any(|entry| entry.contains("keyboard")));
        assert_eq!(clear_indices_reason(false), REASON_POINTER);
    }

    // `ComboboxClear.tsx:87-95` — the constant element wiring.
    #[test]
    fn element_wiring_pins_the_focus_guard() {
        assert_eq!(CLEAR_TAB_INDEX, -1);
        assert!(CLEAR_MOUSE_DOWN_PREVENTED);
        // keepMounted forces the render gate.
        assert!(clear_render_gate(
            &ComboboxClearProps {
                disabled: false,
                keep_mounted: true
            },
            false
        ));
        assert!(!clear_render_gate(&ComboboxClearProps::default(), false));
    }
}
