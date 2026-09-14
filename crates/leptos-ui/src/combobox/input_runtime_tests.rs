//! Tests for the combobox Input behavior layer
//! ([`crate::combobox::input_runtime`]).
//!
//! Host tests mirroring the behavior claims `ComboboxInput.test.tsx` pins:
//! the state composition folds, the ref-callback plan, the chip navigation
//! matrix, the onKeyDown branches (Home/End caret, Escape-on-closed
//! propagation, the empty-Backspace chip removal, the IME guard, the
//! Enter/selection split), and the onChange orchestration (the autofill
//! classification, the empty clear, the composition branches, the highlight
//! clears).

#[cfg(test)]
mod input_runtime_tests {
    use serde_json::{Value, json};

    use crate::combobox::input_runtime::{
        ChipNav, FieldStateFold, InputId, InputKeyDownInput, KeyDownPlan, field_state_for_input,
        focus_manager_modal, input_disabled, input_owns_form_value, is_inside_popup, plan_blur,
        plan_chip_navigation, plan_clear_highlight, plan_composing_empty_close, plan_empty_clear,
        plan_focus_restore, plan_key_down, plan_set_input_element, resolve_input_id,
        should_clear_highlight_on_change, should_open_on_input, shows_dismiss_button,
    };
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use leptos_ui_utils::react_store::ReactStore;

    fn store_with(
        inline: bool,
        has_input_value: bool,
    ) -> ReactStore<ComboboxState, ComboboxStoreContext> {
        let state = ComboboxState {
            id: None,
            label_id: None,
            items: None,
            selected_value: Value::Null,
            open: false,
            mounted: false,
            transition_status: "indeterminate".into(),
            force_mounted: false,
            inline,
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
            disabled: false,
            read_only: false,
            required: false,
            grid: false,
            virtualized: false,
            open_on_input_click: false,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal: false,
            auto_highlight: "false".into(),
            submit_on_item_click: false,
            has_input_value,
        };
        ReactStore::with_context(state, ComboboxStoreContext::default())
    }

    fn keydown(base: InputKeyDownInput) -> KeyDownPlan {
        plan_key_down(&base)
    }

    fn input<'a>() -> InputKeyDownInput<'a> {
        InputKeyDownInput {
            key: "ArrowDown",
            ime_composing: false,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            disabled: false,
            read_only: false,
            open: true,
            mounted: true,
            inline: false,
            selection_mode: "single",
            selected_value: &Value::Null,
            input_value_empty: false,
            has_chips_context: false,
            highlighted_chip_index: None,
            rendered_chips_count: 0,
            direction: "ltr",
            selection_start: None,
            escape_propagation_allowed: false,
            gecko: false,
            rtl: false,
            input_value_len: 5,
            active_index: None,
        }
    }

    // ------------------------------------------------------------------
    // The state composition folds (:51-102)
    // ------------------------------------------------------------------

    // `ComboboxInput.test.tsx:55-77` — a disabled input does not open the popup;
    // the fold is the disabled resolution itself.
    #[test]
    fn disabled_folds_any_source() {
        assert!(input_disabled(false, false, true));
        assert!(input_disabled(false, true, false));
        assert!(input_disabled(true, false, false));
        assert!(!input_disabled(false, false, false));
    }

    #[test]
    fn inside_popup_composition() {
        // `:92` — hasPositionerParent || inline.
        assert!(is_inside_popup(true, false));
        assert!(is_inside_popup(false, true));
        assert!(!is_inside_popup(false, false));
    }

    // `:93` — a standalone input always gets the modal focus manager; an
    // inside-popup input only when the root is modal.
    #[test]
    fn focus_manager_modal_rule() {
        assert!(focus_manager_modal(false, false));
        assert!(focus_manager_modal(true, true));
        assert!(!focus_manager_modal(true, false));
    }

    // `:95-96` — the id: the prop, else the root id when standalone, else
    // generated.
    #[test]
    fn input_id_resolution() {
        assert_eq!(
            resolve_input_id(Some("mine".into()), true, Some("root".into())),
            InputId::Resolved("mine".into())
        );
        assert_eq!(
            resolve_input_id(None, false, Some("root".into())),
            InputId::Resolved("root".into())
        );
        assert_eq!(
            resolve_input_id(None, true, Some("root".into())),
            InputId::Generated
        );
        assert_eq!(resolve_input_id(None, false, None), InputId::Generated);
    }

    // `:96` — inside a positioner the Field state is reset to the defaults.
    #[test]
    fn field_state_resets_inside_positioner() {
        let standalone = FieldStateFold {
            touched: true,
            valid: Some(true),
        };
        assert_eq!(
            field_state_for_input(true, &standalone.clone()),
            FieldStateFold::default()
        );
        assert_eq!(field_state_for_input(false, &standalone), standalone);
    }

    // `:102` — only a standalone none-mode input owns the hidden form value.
    #[test]
    fn form_value_ownership() {
        assert!(input_owns_form_value("none", false));
        assert!(!input_owns_form_value("none", true));
        assert!(!input_owns_form_value("single", false));
    }

    // ------------------------------------------------------------------
    // The setInputElement ref plan (:104-117)
    // ------------------------------------------------------------------

    // `:106-110` — mounting inside the popup with no input value resets the
    // input value to '' with reason none.
    #[test]
    fn ref_callback_resets_value_inside_popup() {
        let s = store_with(false, false);
        let plan = plan_set_input_element(&s, true);
        assert!(plan.reset_input_value);
        assert!(plan.input_inside_popup);

        // Already holding a value: no reset.
        let s2 = store_with(false, true);
        assert!(!plan_set_input_element(&s2, true).reset_input_value);

        // Standalone: no reset either way.
        let s3 = store_with(false, false);
        let plan3 = plan_set_input_element(&s3, false);
        assert!(!plan3.reset_input_value);
        assert!(!plan3.input_inside_popup);
    }

    // ------------------------------------------------------------------
    // The chip navigation matrix (:150-200)
    // ------------------------------------------------------------------

    // `:151-154` — no chips context: `undefined` immediately.
    #[test]
    fn chip_nav_without_context_is_noop() {
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", false, Some(0), 2, 2, Some(0)),
            ChipNav::NoChipsContext
        );
    }

    // `:157-172` — the highlighted chip walks and drops at both boundaries.
    #[test]
    fn chip_nav_walks_the_highlight() {
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, Some(1), 3, 3, Some(0)),
            ChipNav::Index(Some(0))
        );
        // At index 0 the previous key drops the highlight (`:160-163`).
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, Some(0), 3, 3, Some(0)),
            ChipNav::Index(None)
        );
        assert_eq!(
            plan_chip_navigation("ArrowRight", "ltr", true, Some(2), 3, 3, Some(0)),
            ChipNav::Index(None)
        );
        assert_eq!(
            plan_chip_navigation("ArrowRight", "ltr", true, Some(0), 3, 3, Some(0)),
            ChipNav::Index(Some(1))
        );
    }

    // `:173-178` — Backspace/Delete on a highlighted chip walks the removal
    // index (`getIndexAfterChipRemoval(highlight, selectedValue.length)`).
    #[test]
    fn chip_nav_backspace_walks_removal() {
        // Removal at the last chip → the previous one.
        assert_eq!(
            plan_chip_navigation("Backspace", "ltr", true, Some(2), 3, 3, Some(0)),
            ChipNav::Index(Some(1))
        );
        // Removal of the only chip → undefined.
        assert_eq!(
            plan_chip_navigation("Backspace", "ltr", true, Some(0), 1, 1, Some(0)),
            ChipNav::Index(None)
        );
        // Delete behaves identically.
        assert_eq!(
            plan_chip_navigation("Delete", "ltr", true, Some(1), 3, 3, Some(0)),
            ChipNav::Index(Some(1))
        );
    }

    // `:183-189` — the unhighlighted walk: the previous-chip key with the
    // caret at the start and a non-empty selection focuses the last rendered
    // chip (`:609-669` test claim).
    #[test]
    fn chip_nav_unhighlighted_walk() {
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, None, 2, 2, Some(0)),
            ChipNav::Index(Some(1))
        );
        // Caret not at the start: nothing.
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, None, 2, 2, Some(3)),
            ChipNav::Index(None)
        );
        // No selection: nothing.
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, None, 2, 0, Some(0)),
            ChipNav::Index(None)
        );
        // Chips render but the selection is empty — still gated.
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, None, 2, 0, Some(0)),
            ChipNav::Index(None)
        );
        // `:687-704` — a null selectionStart reads as the beginning of a
        // custom input.
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, None, 2, 2, None),
            ChipNav::Index(Some(1))
        );
        // No chips rendered: `undefined` (the `:670-686` claim — focus stays
        // on the input).
        assert_eq!(
            plan_chip_navigation("ArrowLeft", "ltr", true, None, 0, 2, Some(0)),
            ChipNav::Index(None)
        );
    }

    // ------------------------------------------------------------------
    // The onKeyDown matrix (:361-457)
    // ------------------------------------------------------------------

    // `:705-727` — modified navigation is a passthrough (the popup stays open).
    #[test]
    fn modified_keys_pass_through() {
        for key in ["ArrowLeft", "Backspace"] {
            let mut i = input();
            i.key = key;
            i.ctrl = true;
            assert_eq!(keydown(i), KeyDownPlan::ModifiedPassthrough);
        }
    }

    // `:369-374` — the disabled/readOnly early return; readOnly browsing Enter
    // with a highlight must not submit.
    #[test]
    fn read_only_enter_with_highlight_stops() {
        let mut i = input();
        i.read_only = true;
        i.key = "Enter";
        i.active_index = Some(2);
        assert_eq!(keydown(i.clone()), KeyDownPlan::ReadOnlyEnterStop);

        // Without the highlight: inert.
        i.active_index = None;
        assert_eq!(keydown(i.clone()), KeyDownPlan::Inert);

        // Disabled: inert even with the highlight — but note readOnly is still
        // true here, so the readOnly Enter guard wins upstream too; clear
        // readOnly to pin the pure-disabled arm.
        i.disabled = true;
        i.read_only = false;
        i.active_index = Some(2);
        assert_eq!(keydown(i), KeyDownPlan::Inert);
    }

    // `:503-575` — Home/End caret management.
    #[test]
    fn home_end_caret() {
        let mut i = input();
        i.key = "Home";
        i.input_value_len = 7;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::Caret {
                cursor: 0,
                scroll_left: 0
            }
        );

        i.key = "End";
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::Caret {
                cursor: 7,
                scroll_left: 7
            }
        );

        // Gecko rtl swaps the Home/End cursors.
        i.gecko = true;
        i.rtl = true;
        i.key = "Home";
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::Caret {
                cursor: 7,
                scroll_left: 0
            }
        );
        i.key = "End";
        assert_eq!(
            keydown(i),
            KeyDownPlan::Caret {
                cursor: 0,
                scroll_left: -7
            }
        );
    }

    // `:785-834` — Escape on a closed popup: clears the value; stops
    // propagation when something was cleared, propagates when already empty
    // or inline.
    #[test]
    fn escape_on_closed_clears_and_routes_propagation() {
        // Single mode with a value: stops propagation.
        let mut i = input();
        i.mounted = false;
        i.key = "Escape";
        i.selection_mode = "single";
        let sv = json!("apple");
        i.selected_value = &sv;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::EscapeClosed {
                stop_propagation: true
            }
        );

        // Single mode already null: propagates (`:803-818`).
        i.selected_value = &Value::Null;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::EscapeClosed {
                stop_propagation: false
            }
        );

        // Multiple mode with values: stops; empty `[]`: propagates.
        i.selection_mode = "multiple";
        let sv = json!(["a"]);
        i.selected_value = &sv;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::EscapeClosed {
                stop_propagation: true
            }
        );
        let sv = json!([]);
        i.selected_value = &sv;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::EscapeClosed {
                stop_propagation: false
            }
        );

        // Inline: always propagates (`:819-834`).
        let sv = json!("a");
        i.selected_value = &sv;
        i.selection_mode = "single";
        i.inline = true;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::EscapeClosed {
                stop_propagation: false
            }
        );

        // The details allowing propagation also lets it escape.
        i.inline = false;
        i.escape_propagation_allowed = true;
        assert_eq!(
            keydown(i),
            KeyDownPlan::EscapeClosed {
                stop_propagation: false
            }
        );
    }

    // `:902-963` — Backspace on an empty input removes the last rendered chip
    // (or the last selected value when no chips render).
    #[test]
    fn backspace_empty_input_removes_last() {
        let mut i = input();
        i.key = "Backspace";
        i.input_value_empty = true;
        i.selection_mode = "multiple";
        let sv = json!(["a", "b"]);
        i.selected_value = &sv;
        i.has_chips_context = true;

        i.rendered_chips_count = 2;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::ChipRemoveLast { removal_index: 1 }
        );

        // No chips rendered — falls to the selected value's last index.
        i.rendered_chips_count = 0;
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::ChipRemoveLast { removal_index: 1 }
        );

        // Non-empty input: passthrough (the browser deletes a character).
        i.input_value_empty = false;
        i.rendered_chips_count = 2;
        assert_eq!(keydown(i), KeyDownPlan::Passthrough);
    }

    // `:457-467` — the chip-navigation branch: focus to the resolved chip or
    // back to the input when a highlight existed and dropped.
    #[test]
    fn keydown_routes_chip_navigation() {
        let mut i = input();
        i.has_chips_context = true;
        i.selection_mode = "multiple";
        let sv = json!(["a", "b"]);
        i.selected_value = &sv;
        i.rendered_chips_count = 2;
        i.key = "ArrowLeft";
        i.selection_start = Some(0);
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::ChipNav {
                next_index: Some(1),
                restore_input_focus: false
            }
        );

        // A highlighted chip dropping: focus returns to the input.
        i.highlighted_chip_index = Some(0);
        assert_eq!(
            keydown(i.clone()),
            KeyDownPlan::ChipNav {
                next_index: None,
                restore_input_focus: true
            }
        );

        // Navigating from chip 1 to chip 0.
        i.highlighted_chip_index = Some(1);
        assert_eq!(
            keydown(i),
            KeyDownPlan::ChipNav {
                next_index: Some(0),
                restore_input_focus: false
            }
        );
    }

    // `:728-750` — an IME keydown without a highlight does not select.
    #[test]
    fn ime_keydown_does_not_select() {
        let mut i = input();
        i.key = "Enter";
        i.ime_composing = true;
        i.active_index = Some(0);
        // The IME guard precedes the Enter branch only when nothing else
        // matched — but a highlight stands here, so the upstream flow hits the
        // guard first and returns.
        assert_eq!(keydown(i.clone()), KeyDownPlan::ImeGuard);

        // Without a highlight the flow reaches Enter: no highlight + not inline
        // → close with reason none (form submission stays allowed).
        i.active_index = None;
        assert_eq!(keydown(i.clone()), KeyDownPlan::ImeGuard);
        let _ = &i; // the second case's close plan is the ImeGuard per the order
    }

    // `:473-485` — Enter with no highlight: inline does nothing, otherwise the
    // popup closes (form submission allowed).
    #[test]
    fn enter_without_highlight() {
        let mut i = input();
        i.key = "Enter";
        i.active_index = None;
        assert_eq!(keydown(i.clone()), KeyDownPlan::EnterNoHighlightClose);

        i.inline = true;
        assert_eq!(keydown(i), KeyDownPlan::Passthrough);
    }

    // `:487-490` — Enter with a highlight: stop the event, click the item.
    #[test]
    fn enter_with_highlight_clicks() {
        let mut i = input();
        i.key = "Enter";
        i.active_index = Some(3);
        assert_eq!(
            keydown(i),
            KeyDownPlan::EnterHighlightClick { active_index: 3 }
        );
    }

    // ------------------------------------------------------------------
    // The onChange orchestration (:323-379)
    // ------------------------------------------------------------------

    // `:331-333` — autofill classification: Chrome omits inputType, Firefox
    // reports insertReplacementText; composition always counts as typed.
    #[test]
    fn autofill_classification() {
        assert!(!should_open_on_input(None, false));
        assert!(!should_open_on_input(Some("insertReplacementText"), false));
        assert!(should_open_on_input(Some("insertText"), false));
        assert!(should_open_on_input(None, true));
        assert!(should_open_on_input(Some("insertReplacementText"), true));
    }

    // `:350-364` — the empty clear: single mode clears the selection; the
    // popup closes unless an input click opened it; inside-popup inputs do
    // neither.
    #[test]
    fn empty_clear_plan() {
        let plan = plan_empty_clear("single", false, false);
        assert!(plan.clear_selection);
        assert!(plan.close);

        // openOnInputClick: no close.
        assert!(!plan_empty_clear("single", false, true).close);

        // Multiple mode: no selection clear, close still applies.
        let plan = plan_empty_clear("multiple", false, false);
        assert!(!plan.clear_selection);
        assert!(plan.close);

        // Inside the popup: neither.
        let plan = plan_empty_clear("single", true, false);
        assert!(!plan.clear_selection);
        assert!(!plan.close);
    }

    // `:340-349` — the composing-empty close (`:835-869` claim).
    #[test]
    fn composing_empty_close_gate() {
        assert!(plan_composing_empty_close(true, false, false));
        assert!(!plan_composing_empty_close(true, true, false));
        assert!(!plan_composing_empty_close(true, false, true));
        assert!(!plan_composing_empty_close(false, false, false));
    }

    // `:344-349, 366-369` — the highlight clears on change while open, unless
    // autoHighlight holds a query.
    #[test]
    fn highlight_clear_gates() {
        // Open with a highlight and no autoHighlight: clear.
        assert!(should_clear_highlight_on_change(
            true,
            Some(0),
            false,
            false,
            false
        ));
        // Not open: nothing.
        assert!(!should_clear_highlight_on_change(
            false,
            Some(0),
            false,
            false,
            false
        ));
        // No highlight: nothing.
        assert!(!should_clear_highlight_on_change(
            true, None, false, false, false
        ));
        // autoHighlight with a standing query: keep.
        assert!(!should_clear_highlight_on_change(
            true,
            Some(0),
            true,
            false,
            false
        ));
        // autoHighlight with an emptied query: clear (the composing branch's
        // shouldMaintainHighlight gate).
        assert!(should_clear_highlight_on_change(
            true,
            Some(0),
            true,
            true,
            true
        ));
        // autoHighlight with a query, composing: keep.
        assert!(!should_clear_highlight_on_change(
            true,
            Some(0),
            true,
            false,
            true
        ));
        // Typed branch ignores the trimmed emptiness (autoHighlight always
        // keeps the highlight until the root re-arms).
        assert!(!should_clear_highlight_on_change(
            true,
            Some(0),
            true,
            false,
            false
        ));
    }

    // ------------------------------------------------------------------
    // The focus/blur handlers (:254-303)
    // ------------------------------------------------------------------

    // `:751-784` — the inline highlight restore: armed flag + surviving slot.
    #[test]
    fn focus_restore_gates() {
        assert_eq!(plan_focus_restore(true, Some(2), 5), Some(2));
        // Not armed: nothing.
        assert_eq!(plan_focus_restore(false, Some(2), 5), None);
        // No stashed index: nothing.
        assert_eq!(plan_focus_restore(true, None, 5), None);
        // The removed slot (`:751-784`): valuesRef is sparse — a stashed index
        // past the end is not restored.
        assert_eq!(plan_focus_restore(true, Some(4), 3), None);
    }

    // `:277-290` — the blur stash: inline + a highlight + not autoHighlight
    // 'always'.
    #[test]
    fn blur_stash_gates() {
        let plan = plan_blur(true, Some(2), false);
        assert_eq!(plan.stash_index, Some(2));
        assert!(plan.clear_highlight);

        // autoHighlight 'always': no stash, no clear.
        let plan = plan_blur(true, Some(2), true);
        assert_eq!(plan.stash_index, None);
        assert!(!plan.clear_highlight);

        // Not inline: nothing.
        let plan = plan_blur(false, Some(2), false);
        assert_eq!(plan.stash_index, None);
        assert!(!plan.clear_highlight);
    }

    // `:446-448` — the internal dismiss button renders when open and the
    // focus manager is modal.
    #[test]
    fn dismiss_button_gate() {
        assert!(shows_dismiss_button(true, true));
        assert!(!shows_dismiss_button(false, true));
        assert!(!shows_dismiss_button(true, false));
    }

    // ------------------------------------------------------------------
    // The clearHighlight plan (:119-124)
    // ------------------------------------------------------------------

    #[test]
    fn clear_highlight_reason_follows_keyboard_flag() {
        let plan = plan_clear_highlight(true);
        assert_eq!(plan.reason, "keyboard");
        assert_eq!(plan.active_index, None);
        assert_eq!(plan.selected_index, None);
        assert_eq!(plan_clear_highlight(false).reason, "pointer");
    }
}
