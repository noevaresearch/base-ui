//! Tests for the combobox value/chips/clear parts runtime
//! ([`crate::combobox::value_chips`]).
//!
//! Host tests mirroring the behavior claims `specs/library/combobox/parts/value-chips.md`
//! pins from the upstream suites (`ComboboxValue.test.tsx`, `ComboboxChips.test.tsx`,
//! `ComboboxChip.test.tsx`, `ComboboxChipRemove.test.tsx`, `ComboboxClear.test.tsx`):
//! the Value display precedence, the Chips role/open-clear rules, the chip
//! keyboard matrix, the ChipRemove clear-and-remove plan, and the Clear
//! visibility/click/mount gates.

#[cfg(test)]
mod value_chips_tests {
    use std::rc::Rc;

    use serde_json::{Value, json};

    use crate::combobox::value_chips::{
        ChipsRuntime, ValueChildren, ValueDisplay, chip_key_down_blocked, chips_role,
        clear_should_render, clear_visible, default_comparer, display_label_string,
        find_item_index_values, items_have_null_value_label, plan_chip_key_down, plan_chip_remove,
        plan_chip_remove_propagation, plan_clear_click, resolve_value_display,
    };

    // ------------------------------------------------------------------
    // Combobox.Value — display precedence (value-chips.md "State model")
    // ------------------------------------------------------------------

    fn labeler(label: &'static str) -> Rc<dyn Fn(&Value) -> String> {
        Rc::new(move |_| label.to_string())
    }

    // `ComboboxValue.test.tsx:620-683` — uncontrolled defaults render the label
    // for strings/numbers/booleans/objects.
    #[test]
    fn uncontrolled_string_default_resolves_the_item_label() {
        let items = json!([
            { "value": "apple", "label": "Apple" },
            { "value": "banana", "label": "Banana" },
        ]);
        let display = resolve_value_display(
            &ValueChildren::None,
            true,
            true,
            false,
            "single",
            &json!("apple"),
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Single(Value::String("Apple".into())));
    }

    // `ComboboxValue.test.tsx:831-856` — placeholder shown only when the resolved
    // display is empty (null-value item with null label).
    #[test]
    fn placeholder_wins_only_when_the_items_label_is_null() {
        // A null-value item with a null label: the display falls through to the
        // placeholder.
        let items = json!([{ "value": null, "label": null }]);
        let display = resolve_value_display(
            &ValueChildren::None,
            true,
            false,
            false,
            "single",
            &Value::Null,
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Placeholder);

        // A null-value item WITH a label suppresses the placeholder (the
        // hasNullItemLabel gate) and the null value matches that item, so its
        // label is the display.
        let items = json!([{ "value": null, "label": "Nothing selected" }]);
        let display = resolve_value_display(
            &ValueChildren::None,
            true,
            false,
            true,
            "single",
            &Value::Null,
            Some(&items),
            None,
        );
        assert_eq!(
            display,
            ValueDisplay::Single(Value::String("Nothing selected".into())),
            "the null item's label resolves, not the placeholder"
        );
    }

    // `ComboboxValue.test.tsx:760-802` — children (node or function) wins over
    // placeholder.
    #[test]
    fn children_win_over_placeholder() {
        for children in [ValueChildren::Node, ValueChildren::RenderFunction] {
            let display = resolve_value_display(
                &children,
                true,
                false,
                false,
                "single",
                &Value::Null,
                None,
                None,
            );
            assert_ne!(display, ValueDisplay::Placeholder);
        }
    }

    // `ComboboxValue.test.tsx:525-592` — multiple mode folds labels with
    // `", "` separators.
    #[test]
    fn multiple_mode_joins_labels_with_comma_separators() {
        let items = json!([
            { "value": "sans", "label": "Sans-serif" },
            { "value": "serif", "label": "Serif" },
        ]);
        let display = resolve_value_display(
            &ValueChildren::None,
            false,
            true,
            false,
            "multiple",
            &json!(["sans", "serif"]),
            Some(&items),
            None,
        );
        match display {
            ValueDisplay::Multiple(labels) => {
                assert_eq!(
                    labels,
                    vec![
                        Value::String("Sans-serif".into()),
                        Value::String(", ".into()),
                        Value::String("Serif".into()),
                    ]
                );
            }
            other => panic!("expected Multiple, got {:?}", other),
        }
    }

    // `ComboboxValue.test.tsx:879-898` — an empty array in multiple mode is "no
    // value": the placeholder displays.
    #[test]
    fn empty_array_in_multiple_mode_shows_placeholder() {
        let display = resolve_value_display(
            &ValueChildren::None,
            true,
            false,
            false,
            "multiple",
            &json!([]),
            None,
            None,
        );
        assert_eq!(display, ValueDisplay::Placeholder);
    }

    // `ComboboxValue.test.tsx:387-440` — the display re-derives from the current
    // items (no staleness): the function is pure over its inputs.
    #[test]
    fn label_resolution_follows_the_current_items_array() {
        let items = json!([{ "value": "a", "label": "Old" }]);
        let display = resolve_value_display(
            &ValueChildren::None,
            false,
            true,
            false,
            "single",
            &json!("a"),
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Single(Value::String("Old".into())));

        let items = json!([{ "value": "a", "label": "New" }]);
        let display = resolve_value_display(
            &ValueChildren::None,
            false,
            true,
            false,
            "single",
            &json!("a"),
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Single(Value::String("New".into())));
    }

    // `ComboboxValue.test.tsx:358-385` — duplicate values resolve to the first
    // match's label.
    #[test]
    fn duplicate_item_values_resolve_to_the_first_match() {
        let items = json!([
            { "value": "a", "label": "First" },
            { "value": "a", "label": "Second" },
        ]);
        let display = resolve_value_display(
            &ValueChildren::None,
            false,
            true,
            false,
            "single",
            &json!("a"),
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Single(Value::String("First".into())));
    }

    // `ComboboxValue.test.tsx:489-521` — grouped items participate in resolution.
    #[test]
    fn grouped_items_participate_in_label_resolution() {
        let items = json!([
            {
                "label": "Fruits",
                "items": [{ "value": "apple", "label": "Apple" }],
            },
        ]);
        let display = resolve_value_display(
            &ValueChildren::None,
            false,
            true,
            false,
            "single",
            &json!("apple"),
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Single(Value::String("Apple".into())));
    }

    // The `shouldCheckNullItemLabel` gate — the selector is off when a value
    // exists or children render, so a null-labeled item cannot suppress the
    // placeholder in a state where it would never matter anyway.
    #[test]
    fn null_item_label_gate_is_off_when_not_needed() {
        // With a selected value the scan never runs — the display resolves.
        let items = json!([{ "value": null, "label": "Nothing" }]);
        let display = resolve_value_display(
            &ValueChildren::None,
            true,
            true,
            true,
            "single",
            &json!("x"),
            Some(&items),
            None,
        );
        assert_eq!(display, ValueDisplay::Single(Value::String("x".into())));
    }

    // The `items_have_null_value_label` re-export — the selector's scan.
    #[test]
    fn null_item_label_scan_classifies_items() {
        assert!(items_have_null_value_label(Some(&json!([
            { "value": null, "label": "None" },
        ]))));
        assert!(!items_have_null_value_label(Some(&json!([
            { "value": null, "label": null },
        ]))));
        // Grouped form.
        assert!(items_have_null_value_label(Some(&json!([
            { "items": [{ "value": null, "label": "None" }] },
        ]))));
    }

    // The label stringifier seam — `stringifyValueLabel` under a custom
    // stringifier.
    #[test]
    fn display_label_string_uses_the_carried_stringifier() {
        assert_eq!(
            display_label_string(&json!("apple"), Some(&labeler("APPLE"))),
            "APPLE"
        );
    }

    // ------------------------------------------------------------------
    // Combobox.Chips — role + highlighted-chip clear (value-chips.md "A11y")
    // ------------------------------------------------------------------

    // `ComboboxChips.test.tsx:17-39` — no role when empty, toolbar with chips.
    #[test]
    fn chips_role_is_toolbar_only_when_chips_exist() {
        assert_eq!(chips_role(false), None);
        assert_eq!(chips_role(true), Some("toolbar"));
    }

    // `ComboboxChips.test.tsx:277-309` — reopening the popup clears the
    // highlighted chip.
    #[test]
    fn opening_the_popup_clears_the_highlighted_chip() {
        let chips = ChipsRuntime::default();
        chips.highlighted_chip_index.set(Some(1));
        assert!(chips.sync_open(true), "reset happened");
        assert_eq!(chips.highlighted_chip_index.get(), None);

        // While closed, sync is a no-op.
        chips.highlighted_chip_index.set(Some(0));
        assert!(!chips.sync_open(false));
        assert_eq!(chips.highlighted_chip_index.get(), Some(0));

        // Open again with the still-highlighted chip: reset.
        assert!(chips.sync_open(true));
        // Already-clear + open: no reset to report.
        assert!(!chips.sync_open(true));
    }

    // ------------------------------------------------------------------
    // Combobox.Chip — the keyboard matrix (value-chips.md "Keyboard")
    // ------------------------------------------------------------------

    // `ComboboxChip.test.tsx:299-308` — ArrowRight steps to the next chip; on
    // the last chip it moves to the input (nextIndex = undefined).
    #[test]
    fn arrow_right_steps_to_next_chip_then_the_input() {
        let plan = plan_chip_key_down("ArrowRight", false, false, false, 0, 3, 3, "ltr");
        assert_eq!(plan.next_index, Some(1));
        assert!(plan.stop_event);

        let plan = plan_chip_key_down("ArrowRight", false, false, false, 2, 3, 3, "ltr");
        assert_eq!(plan.next_index, None);
    }

    // `ComboboxChip.test.tsx:310-315` — ArrowLeft on the first chip moves to the
    // input.
    #[test]
    fn arrow_left_on_the_first_chip_moves_to_the_input() {
        let plan = plan_chip_key_down("ArrowLeft", false, false, false, 0, 3, 3, "ltr");
        assert_eq!(plan.next_index, None);
    }

    // `ComboboxChip.test.tsx:452-492` — navigation walks only rendered chips.
    #[test]
    fn navigation_is_bounded_by_rendered_chips() {
        // 3 selected values but only 1 rendered chip: ArrowRight from the last
        // rendered chip hands focus to the input.
        let plan = plan_chip_key_down("ArrowRight", false, false, false, 0, 1, 3, "ltr");
        assert_eq!(plan.next_index, None);
    }

    // `ComboboxChip.test.tsx:410-450` — RTL mirrors arrow semantics.
    #[test]
    fn rtl_mirrors_arrow_semantics() {
        let plan = plan_chip_key_down("ArrowRight", false, false, false, 0, 3, 3, "rtl");
        assert_eq!(
            plan.next_index, None,
            "ArrowRight in RTL is the 'previous' key"
        );
        let plan = plan_chip_key_down("ArrowLeft", false, false, false, 0, 3, 3, "rtl");
        assert_eq!(plan.next_index, Some(1));
    }

    // `ComboboxChip.test.tsx:494-520`, `:388-408` — Backspace/Delete remove the
    // chip.
    #[test]
    fn backspace_and_delete_remove_the_chip() {
        for key in ["Backspace", "Delete"] {
            let plan = plan_chip_key_down(key, false, false, false, 1, 3, 3, "ltr");
            assert!(plan.remove_chip);
            assert!(plan.stop_event);
            // `getIndexAfterChipRemoval(1, 3)` — the neighbor highlight.
            assert_eq!(plan.next_index, Some(1));
        }
        // Removing the last of TWO chips lands on the surviving neighbor
        // (`getIndexAfterChipRemoval(1, 2)` → 0).
        let plan = plan_chip_key_down("Backspace", false, false, false, 1, 2, 2, "ltr");
        assert!(plan.remove_chip);
        assert_eq!(plan.next_index, Some(0));

        // Removing the only chip lands on no highlight (the underflow →
        // `undefined` arm of `getIndexAfterChipRemoval`).
        let plan = plan_chip_key_down("Backspace", false, false, false, 0, 1, 1, "ltr");
        assert!(plan.remove_chip);
        assert_eq!(plan.next_index, None);
    }

    // `ComboboxChip.test.tsx:318-363` — Enter/Space/ArrowDown/ArrowUp/printable
    // hand focus to the input (ArrowDown/ArrowUp additionally open the popup).
    #[test]
    fn activation_and_opening_keys_return_focus_to_the_input() {
        for key in ["Enter", " "] {
            let plan = plan_chip_key_down(key, false, false, false, 0, 3, 3, "ltr");
            assert_eq!(plan.next_index, None);
            assert!(plan.stop_event);
            assert!(!plan.open_popup);
        }
        for key in ["ArrowDown", "ArrowUp"] {
            let plan = plan_chip_key_down(key, false, false, false, 0, 3, 3, "ltr");
            assert_eq!(plan.next_index, None);
            assert!(plan.open_popup);
        }
        let plan = plan_chip_key_down("a", false, false, false, 0, 3, 3, "ltr");
        assert_eq!(plan.next_index, None);
    }

    // `ComboboxChip.test.tsx:365-386` — modified printables leave focus on the
    // chip.
    #[test]
    fn modified_printable_keys_leave_focus_on_the_chip() {
        for (ctrl, meta, alt) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let plan = plan_chip_key_down("a", ctrl, meta, alt, 1, 3, 3, "ltr");
            assert_eq!(plan.next_index, Some(1), "focus stays on the chip");
        }
    }

    // `ComboboxChip.test.tsx:36-90`, `:171-196` — disabled/readOnly blocks the
    // handler entirely.
    #[test]
    fn disabled_and_read_only_block_chip_key_down() {
        assert!(chip_key_down_blocked(true, false));
        assert!(chip_key_down_blocked(false, true));
        assert!(!chip_key_down_blocked(false, false));
    }

    // ------------------------------------------------------------------
    // Combobox.ChipRemove — the removal plan (value-chips.md "Events")
    // ------------------------------------------------------------------

    // `ComboboxChipRemove.test.tsx:151-179` — click removes the chip's value.
    #[test]
    fn chip_remove_builds_the_filtered_array() {
        let selected = vec![json!("a"), json!("b"), json!("c")];
        let comparer = default_comparer();
        let plan = plan_chip_remove(1, &selected, None, &selected, comparer.as_ref(), false);
        assert_eq!(plan.next_selected_value, json!(["a", "c"]));
        assert_eq!(plan.active_index_reason, "pointer");
        assert!(plan.propagation_blocked_by_default);
    }

    // `ComboboxChipRemove.tsx:64-85` — the active index clears only when the
    // removed item IS the active one in the visible list.
    #[test]
    fn active_index_clears_only_when_the_removed_item_is_active() {
        let selected = vec![json!("a"), json!("b")];
        let comparer = default_comparer();

        // Active on the removed item: clear.
        let plan = plan_chip_remove(0, &selected, Some(0), &selected, comparer.as_ref(), false);
        assert!(plan.clear_active_index);

        // Active on a different item: no clear.
        let plan = plan_chip_remove(0, &selected, Some(1), &selected, comparer.as_ref(), false);
        assert!(!plan.clear_active_index);

        // No active index at all: no clear.
        let plan = plan_chip_remove(0, &selected, None, &selected, comparer.as_ref(), false);
        assert!(!plan.clear_active_index);

        // The removed item is filtered out of the visible list: no clear (the
        // highlight can't equal the removed index).
        let visible = vec![json!("b")];
        let plan = plan_chip_remove(0, &selected, Some(0), &visible, comparer.as_ref(), false);
        assert!(!plan.clear_active_index);
    }

    // `ComboboxChipRemove.tsx:80-83` — the indices reason follows the
    // keyboard-active flag.
    #[test]
    fn active_index_reason_follows_keyboard_active() {
        let selected = vec![json!("a")];
        let comparer = default_comparer();
        let plan = plan_chip_remove(0, &selected, None, &selected, comparer.as_ref(), true);
        assert_eq!(plan.active_index_reason, "keyboard");
    }

    // `ComboboxChipRemove.test.tsx:283-335` — propagation blocked by default,
    // allowed when the user's onValueChange called allowPropagation().
    #[test]
    fn propagation_follows_the_allow_propagation_decision() {
        assert!(plan_chip_remove_propagation(false), "blocked by default");
        assert!(
            !plan_chip_remove_propagation(true),
            "allowed by the user handler"
        );
    }

    // ------------------------------------------------------------------
    // Combobox.Clear — visibility + click plan (value-chips.md "Clear")
    // ------------------------------------------------------------------

    // `ComboboxClear.test.tsx:83-105` (single), `:61-81` (multiple),
    // `:44-59` (autocomplete none-mode).
    #[test]
    fn clear_visibility_follows_the_selection_mode() {
        // none: visible with a typed value.
        assert!(clear_visible("none", "abc", &Value::Null, false));
        assert!(!clear_visible("none", "", &Value::Null, false));
        // single: visible with any non-null selection.
        assert!(clear_visible("single", "", &json!("a"), false));
        assert!(!clear_visible("single", "", &Value::Null, false));
        // multiple: visible with any chips.
        assert!(clear_visible("multiple", "", &json!(["a"]), true));
        assert!(!clear_visible("multiple", "", &json!([]), false));
    }

    // `ComboboxClear.tsx:119-123` — keepMounted keeps the button in the DOM.
    #[test]
    fn keep_mounted_stays_rendered_when_not_visible() {
        assert!(clear_should_render(true, false));
        assert!(clear_should_render(false, true));
        assert!(!clear_should_render(false, false));
    }

    // `ComboboxClear.test.tsx:180-207` — disabled/readOnly block the click.
    #[test]
    fn disabled_and_read_only_block_the_clear_click() {
        let plan = plan_clear_click(true, false, "single", &json!("a"), false);
        assert!(plan.blocked);
        assert_eq!(plan.input_value, None);
        assert!(!plan.focus_input);

        let plan = plan_clear_click(false, true, "single", &json!("a"), false);
        assert!(plan.blocked);

        let plan = plan_clear_click(false, false, "single", &json!("a"), false);
        assert!(!plan.blocked);
    }

    // `ComboboxClear.tsx:107-110` — single clears to null, multiple to [].
    #[test]
    fn clear_click_clears_null_single_and_empty_array_multiple() {
        let plan = plan_clear_click(false, false, "single", &json!("a"), false);
        assert_eq!(plan.next_selected_value, Some(Value::Null));
        assert_eq!(plan.input_value, Some(String::new()));
        assert!(plan.clear_selected_index);
        assert!(plan.focus_input);

        let plan = plan_clear_click(false, false, "multiple", &json!(["a", "b"]), false);
        assert_eq!(plan.next_selected_value, Some(json!([])));
    }

    // `ComboboxClear.tsx:111-114` — none-mode skips the selected-value command
    // entirely (typed-filter clear, the Autocomplete case).
    #[test]
    fn none_mode_clears_only_the_input_value() {
        let plan = plan_clear_click(false, false, "none", &Value::Null, false);
        assert_eq!(plan.next_selected_value, None);
        assert!(!plan.clear_selected_index);
        assert_eq!(plan.input_value, Some(String::new()));
        assert!(plan.focus_input);
    }

    // `ComboboxClear.tsx:101-102` — the indices reason follows the
    // keyboard-active flag.
    #[test]
    fn clear_indices_reason_follows_keyboard_active() {
        let plan = plan_clear_click(false, false, "single", &json!("a"), true);
        assert_eq!(plan.indices_reason, "keyboard");
        let plan = plan_clear_click(false, false, "single", &json!("a"), false);
        assert_eq!(plan.indices_reason, "pointer");
    }

    // ------------------------------------------------------------------
    // The helpers
    // ------------------------------------------------------------------

    #[test]
    fn find_item_index_values_locates_the_first_match() {
        let values = vec![json!("a"), json!("b"), json!("a")];
        assert_eq!(find_item_index_values(&values, &json!("a")), Some(0));
        assert_eq!(find_item_index_values(&values, &json!("b")), Some(1));
        assert_eq!(find_item_index_values(&values, &json!("z")), None);
    }
}
