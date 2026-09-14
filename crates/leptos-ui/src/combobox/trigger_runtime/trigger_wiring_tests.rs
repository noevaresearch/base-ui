//! Tests for the combobox Trigger behavior layer
//! ([`crate::combobox::trigger_runtime`]).
//!
//! Host tests mirroring the behavior claims `ComboboxTrigger.test.tsx`
//! pins: the aria matrix (tabIndex/role/aria-required/aria-readonly across
//! the inside-popup modes, `ComboboxTrigger.test.tsx:23,812-911`), the
//! disabled fold (`:44-171`), the pending-mouseup bail (`:172-201`), the
//! typeahead gate and sparse-label skip (`:297-324, 914-1098`), the
//! arrow-key opening (`:404-455`), the cancel-open mouseup bounds
//! (`:702-810`), and the data-state attributes (`:1160-1304`).

#[cfg(test)]
mod trigger_wiring_tests {
    use serde_json::{Value, json};

    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use crate::combobox::trigger_runtime::{
        BlurPlan, FocusPlan, KeyDownPlan, MouseDownPlan, MouseUpPlan, TriggerId,
        TypeaheadMatchPlan, is_mouse_within_bounds, labelable_id_input, plan_blur, plan_focus,
        plan_key_down, plan_mouse_down, plan_mouse_up, plan_trigger_aria, plan_typeahead_match,
        resolve_aria_controls, resolve_trigger_id, trigger_disabled, trigger_placeholder,
        trigger_state_attributes, trigger_state_map, typeahead_enabled,
    };
    use leptos_ui_utils::react_store::ReactStore;

    fn store_with() -> ReactStore<ComboboxState, ComboboxStoreContext> {
        let state = ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: Some(vec![json!("Apple"), json!("Banana")]),
            selected_value: Value::Null,
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
            input_inside_popup: true,
            input_owns_form_value: false,
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
            has_input_value: false,
        };
        ReactStore::with_context(state, ComboboxStoreContext::default())
    }

    // -- the disabled fold (ComboboxTrigger.test.tsx:44-171) ----------------

    #[test]
    fn disabled_folds_field_root_and_prop() {
        assert!(!trigger_disabled(false, false, false));
        assert!(trigger_disabled(true, false, false));
        assert!(trigger_disabled(false, true, false));
        assert!(trigger_disabled(false, false, true));
    }

    // -- the aria matrix (ComboboxTrigger.test.tsx:23, 812-911) -------------

    #[test]
    fn tab_index_is_negative_outside_the_popup() {
        // `:23-42` — not used as the main anchor: tabIndex -1, no combobox
        // role, listbox haspopup.
        let plan = plan_trigger_aria(TriggerId::Generated, false, false, false, false, None, None);
        assert_eq!(plan.tab_index, -1);
        assert_eq!(plan.role, None);
        assert_eq!(plan.aria_haspopup, "listbox");
        // The aria-required / aria-readonly pair rides only the combobox role
        // (`:823-834`, `:847-862`).
        assert_eq!(plan.aria_required, None);
        assert_eq!(plan.aria_readonly, None);
    }

    #[test]
    fn inside_popup_mode_carries_the_combobox_role_and_required() {
        // `:812-822` — aria-required set when required (input inside popup).
        let plan = plan_trigger_aria(TriggerId::Generated, true, false, true, false, None, None);
        assert_eq!(plan.tab_index, 0);
        assert_eq!(plan.role.as_deref(), Some("combobox"));
        assert_eq!(plan.aria_haspopup, "dialog");
        assert_eq!(plan.aria_required, Some(true));
        // `:835-846` — aria-readonly set when readOnly (input inside popup).
        let ro = plan_trigger_aria(TriggerId::Generated, true, false, false, true, None, None);
        assert_eq!(ro.aria_readonly, Some(true));
        assert_eq!(ro.aria_required, Some(false));
    }

    #[test]
    fn aria_expanded_tracks_open() {
        // `:885-911` — the open/closed aria matrix.
        let closed = plan_trigger_aria(TriggerId::Generated, true, false, false, false, None, None);
        assert!(!closed.aria_expanded);
        let open = plan_trigger_aria(TriggerId::Generated, true, true, false, false, None, None);
        assert!(open.aria_expanded);
    }

    #[test]
    fn aria_labelledby_resolves_through_the_field_chain() {
        // `resolveAriaLabelledBy` (`resolveAriaLabelledBy.ts:7-11`) — the
        // field label wins; the combobox label id is the local fallback.
        assert_eq!(
            leptos_ui_internals::resolve_aria_labelled_by(Some("field-lbl"), Some("lbl"))
                .as_deref(),
            Some("field-lbl")
        );
        assert_eq!(
            leptos_ui_internals::resolve_aria_labelled_by(None, Some("lbl")).as_deref(),
            Some("lbl")
        );
    }

    #[test]
    fn id_resolves_from_prop_then_root_inside_popup() {
        assert_eq!(
            resolve_trigger_id(Some("t".into()), true, Some("root".into())),
            TriggerId::Resolved("t".into())
        );
        // The root id is the fallback only in the inside-popup mode (`:87`).
        assert_eq!(
            resolve_trigger_id(None, true, Some("root".into())),
            TriggerId::Resolved("root".into())
        );
        assert_eq!(
            resolve_trigger_id(None, false, Some("root".into())),
            TriggerId::Generated
        );
    }

    #[test]
    fn labelable_id_registers_only_inside_popup() {
        assert_eq!(labelable_id_input(Some("t"), true).as_deref(), Some("t"));
        assert_eq!(labelable_id_input(Some("t"), false), None);
    }

    // -- aria-controls (ComboboxTrigger.tsx:90-98) --------------------------

    #[test]
    fn aria_controls_is_none_while_closed() {
        assert_eq!(
            resolve_aria_controls(false, true, None, Some("root"), None),
            None
        );
        assert_eq!(
            resolve_aria_controls(false, false, Some("p"), Some("root"), None),
            None
        );
    }

    #[test]
    fn aria_controls_falls_back_to_the_default_popup_id() {
        // Open with the input inside the popup: the stored custom id wins;
        // before the popup registers, the default `{rootId}-popup` applies
        // so the attribute lands on the same commit `open` becomes true.
        assert_eq!(
            resolve_aria_controls(true, true, Some("custom"), Some("root"), None).as_deref(),
            Some("custom")
        );
        assert_eq!(
            resolve_aria_controls(true, true, None, Some("root"), None).as_deref(),
            Some("root-popup")
        );
        assert_eq!(resolve_aria_controls(true, true, None, None, None), None);
    }

    #[test]
    fn aria_controls_with_a_standalone_input_needs_the_list_element() {
        // Open with a standalone input points at the list element's id
        // (`:96-98`); without the element registered there is nothing to
        // point at. (The element-id read itself is the wiring's
        // `get_attribute("id")` — host tests pin the plan's gates.)
        assert_eq!(
            resolve_aria_controls(true, false, None, Some("root"), None),
            None
        );
    }

    // -- focus / blur (ComboboxTrigger.tsx:166-188) --------------------------

    #[test]
    fn focus_schedules_the_mount_unless_disabled() {
        assert_eq!(
            plan_focus(false),
            FocusPlan {
                set_focused: true,
                force_mount: true
            }
        );
        assert_eq!(
            plan_focus(true),
            FocusPlan {
                set_focused: true,
                force_mount: false
            }
        );
    }

    #[test]
    fn blur_into_the_popup_is_not_a_blur() {
        let plan = plan_blur(true, true, "single", "", &Value::Null);
        assert!(plan.is_popup_focus_move);
        assert!(!plan.set_touched);
        assert!(plan.set_focused);
        assert_eq!(plan.commit_value, None);
    }

    #[test]
    fn blur_commits_the_mode_value_on_blur_validation() {
        // `none` mode validates the input value (`:185`).
        assert_eq!(
            plan_blur(false, true, "none", "query", &json!("x")).commit_value,
            Some(json!("query"))
        );
        // Selection modes validate the selected value (`:185`).
        assert_eq!(
            plan_blur(false, true, "single", "query", &json!("Apple")).commit_value,
            Some(json!("Apple"))
        );
        // A non-onBlur validation mode commits nothing (`:184-187`).
        assert_eq!(
            plan_blur(false, false, "single", "query", &json!("Apple")).commit_value,
            None
        );
    }

    #[test]
    fn blur_outside_the_popup_touches_and_unfocuses_the_field() {
        let plan = plan_blur(false, false, "single", "", &Value::Null);
        assert!(!plan.is_popup_focus_move);
        assert!(plan.set_touched);
        assert!(!plan.set_focused);
    }

    // -- mouse down / drag-selection pairing (ComboboxTrigger.tsx:189-243) --

    #[test]
    fn disabled_mouse_down_is_inert() {
        assert_eq!(
            plan_mouse_down(true, true, "mouse", false),
            MouseDownPlan {
                set_dom_reference: false,
                force_mount: false,
                focus_input: false,
                prevent_default: false,
                track_mouseup: false,
            }
        );
    }

    #[test]
    fn mouse_down_mounts_items_and_focuses_the_input() {
        // `:198-207` — items register for the initial highlight; non-touch
        // pointers move the focus; the default is suppressed only outside
        // the popup.
        assert_eq!(
            plan_mouse_down(false, true, "mouse", false),
            MouseDownPlan {
                set_dom_reference: false,
                force_mount: true,
                focus_input: true,
                prevent_default: false,
                track_mouseup: true,
            }
        );
        assert_eq!(
            plan_mouse_down(false, false, "mouse", false),
            MouseDownPlan {
                set_dom_reference: true,
                force_mount: true,
                focus_input: true,
                prevent_default: true,
                track_mouseup: false,
            }
        );
    }

    #[test]
    fn touch_pointer_never_moves_the_input_focus() {
        // `:201-207`.
        let plan = plan_mouse_down(false, false, "touch", false);
        assert!(!plan.focus_input);
        assert!(!plan.prevent_default);
        assert!(plan.set_dom_reference);
    }

    #[test]
    fn open_popup_does_not_install_the_mouseup_pairing() {
        // `:209-211` + `:240-242` — the pairing is installed only when
        // closed, and only with the input inside the popup.
        let open_inside = plan_mouse_down(false, true, "mouse", true);
        assert!(open_inside.force_mount);
        assert!(!open_inside.track_mouseup);
        let closed_outside = plan_mouse_down(false, false, "mouse", false);
        assert!(!closed_outside.track_mouseup);
    }

    #[test]
    fn mouseup_after_trigger_unmount_exits_silently() {
        // `:172-201` — the pending mouseup after the trigger unmounts must
        // not close anything.
        assert_eq!(
            plan_mouse_up(false, true, false, false, false, false),
            MouseUpPlan::TriggerGone
        );
    }

    #[test]
    fn mouseup_on_trigger_positioner_or_list_keeps_the_popup() {
        // `:225-231` — the containment walk.
        assert_eq!(
            plan_mouse_up(true, true, true, false, false, false),
            MouseUpPlan::Keep
        );
        assert_eq!(
            plan_mouse_up(true, true, false, true, false, false),
            MouseUpPlan::Keep
        );
        assert_eq!(
            plan_mouse_up(true, true, false, false, true, false),
            MouseUpPlan::Keep
        );
    }

    #[test]
    fn mouseup_within_5px_of_the_trigger_bounds_keeps_the_popup() {
        // `:736-777` — the release near the trigger bounds stays open.
        assert_eq!(
            plan_mouse_up(true, true, false, false, false, true),
            MouseUpPlan::Keep
        );
    }

    #[test]
    fn mouseup_beyond_the_bounds_closes_with_cancel_open() {
        // `:702-735, 778-810` — the release more than 5px out closes the
        // popup with the cancel-open reason.
        assert_eq!(
            plan_mouse_up(true, true, false, false, false, false),
            MouseUpPlan::Close
        );
    }

    #[test]
    fn mouseup_without_a_target_never_closes() {
        assert_eq!(
            plan_mouse_up(true, false, false, false, false, false),
            MouseUpPlan::Keep
        );
    }

    #[test]
    fn bounds_inflation_matches_get_pseudo_element_bounds() {
        // `getPseudoElementBounds.ts:20-29` — 5px on every side.
        assert!(is_mouse_within_bounds(
            105.0, 100.0, 100.0, 100.0, 200.0, 200.0
        ));
        assert!(is_mouse_within_bounds(
            96.0, 100.0, 100.0, 100.0, 200.0, 200.0
        ));
        assert!(!is_mouse_within_bounds(
            94.9, 100.0, 100.0, 100.0, 200.0, 200.0
        ));
        assert!(!is_mouse_within_bounds(
            205.1, 100.0, 100.0, 100.0, 200.0, 200.0
        ));
        assert!(!is_mouse_within_bounds(
            100.0, 205.1, 100.0, 100.0, 200.0, 200.0
        ));
    }

    // -- key down (ComboboxTrigger.tsx:244-253; test :404-455) --------------

    #[test]
    fn arrow_keys_open_with_list_navigation() {
        // `:404-439`.
        assert_eq!(
            plan_key_down("ArrowDown"),
            KeyDownPlan::OpenWithListNavigation
        );
        assert_eq!(
            plan_key_down("ArrowUp"),
            KeyDownPlan::OpenWithListNavigation
        );
    }

    #[test]
    fn other_keys_do_not_intercept() {
        assert_eq!(plan_key_down("Enter"), KeyDownPlan::Inert);
        assert_eq!(plan_key_down("Escape"), KeyDownPlan::Inert);
        assert_eq!(plan_key_down("a"), KeyDownPlan::Inert);
    }

    // -- typeahead (ComboboxTrigger.tsx:106-119; test :297-324, 914-1098) ---

    #[test]
    fn typeahead_is_gated_on_closed_readonly_and_single_mode() {
        // `:297-324` — readOnly never commits a typeahead match on a closed
        // trigger; `:109` — multiple/none modes are excluded.
        assert!(typeahead_enabled(false, false, false, "single"));
        assert!(!typeahead_enabled(true, false, false, "single"));
        assert!(!typeahead_enabled(false, true, false, "single"));
        assert!(!typeahead_enabled(false, false, true, "single"));
        assert!(!typeahead_enabled(false, false, false, "multiple"));
        assert!(!typeahead_enabled(false, false, false, "none"));
    }

    #[test]
    fn typeahead_match_commits_only_registered_values() {
        // `:202-227` — a sparse label (no value at the index) skips the
        // commit; a real value commits with the `none` reason.
        assert_eq!(
            plan_typeahead_match(Some(json!("Apple"))),
            TypeaheadMatchPlan::Commit(json!("Apple"))
        );
        assert_eq!(plan_typeahead_match(None), TypeaheadMatchPlan::Skip);
    }

    // -- placeholder / state attributes (test :1160-1304) --------------------

    #[test]
    fn placeholder_tracks_the_selection_mode() {
        // `:1217-1304` — data-placeholder when no value; not in none mode;
        // the empty array in multiple mode counts as no value.
        assert!(trigger_placeholder("single", false));
        assert!(!trigger_placeholder("single", true));
        assert!(!trigger_placeholder("none", false));
        assert!(trigger_placeholder("multiple", false));
    }

    #[test]
    fn state_attributes_carry_the_pressable_open_hooks() {
        // `pressableTriggerOpenStateMapping` — data-popup-open +
        // data-pressed while open, nothing when closed.
        let mut state = serde_json::Map::new();
        state.insert("open".into(), json!(true));
        let attrs = trigger_state_attributes(&state);
        assert!(attrs.contains(&("data-popup-open".to_string(), String::new())));
        assert!(attrs.contains(&("data-pressed".to_string(), String::new())));

        state.insert("open".into(), json!(false));
        assert!(trigger_state_attributes(&state).is_empty());
    }

    #[test]
    fn state_attributes_carry_validity_side_hooks() {
        // `fieldValidityMapping` — nothing before the first validation.
        let mut state = serde_json::Map::new();
        state.insert("valid".into(), Value::Null);
        assert!(trigger_state_attributes(&state).is_empty());
        state.insert("valid".into(), json!(true));
        assert!(
            trigger_state_attributes(&state).contains(&("data-valid".to_string(), String::new()))
        );
        state.insert("valid".into(), json!(false));
        assert!(
            trigger_state_attributes(&state).contains(&("data-invalid".to_string(), String::new()))
        );
    }

    #[test]
    fn state_attributes_carry_popup_side_and_list_empty_hooks() {
        // `:1161-1195` — data-popup-side only while the positioner mounts;
        // data-list-empty only while the filtered list is empty.
        let mut state = serde_json::Map::new();
        state.insert("popupSide".into(), json!("right"));
        let attrs = trigger_state_attributes(&state);
        assert!(attrs.contains(&("data-popup-side".to_string(), "right".to_string())));
        state.insert("popupSide".into(), Value::Null);
        assert!(trigger_state_attributes(&state).is_empty());

        let mut state = serde_json::Map::new();
        state.insert("listEmpty".into(), json!(true));
        assert!(
            trigger_state_attributes(&state)
                .contains(&("data-list-empty".to_string(), String::new()))
        );
        state.insert("listEmpty".into(), json!(false));
        assert!(trigger_state_attributes(&state).is_empty());
    }

    #[test]
    fn state_attributes_default_walk_emits_truthy_flags() {
        // `getStateAttributesProps.ts:24-28` — the default truthy walk.
        let mut state = serde_json::Map::new();
        state.insert("readOnly".into(), json!(true));
        state.insert("disabled".into(), json!(false));
        state.insert("placeholder".into(), json!(true));
        let attrs = trigger_state_attributes(&state);
        assert!(attrs.contains(&("data-read-only".to_string(), String::new())));
        assert!(attrs.contains(&("data-placeholder".to_string(), String::new())));
        assert!(!attrs.iter().any(|(k, _)| k == "data-disabled"));
    }

    // -- the store-facing composition ----------------------------------------

    #[test]
    fn state_map_folds_the_store_state() {
        let store = store_with();
        store.set_field(|state| &mut state.open, true);
        store.set_field(|state| &mut state.disabled, true);
        store.set_field(|state| &mut state.read_only, true);
        store.set_field(|state| &mut state.selected_value, json!("Apple"));

        let map = trigger_state_map(&store, false, false, true, None, false);
        assert_eq!(map.get("open"), Some(&json!(true)));
        // The fold: the store's disabled state wins.
        assert_eq!(map.get("disabled"), Some(&json!(true)));
        assert_eq!(map.get("readOnly"), Some(&json!(true)));
        // Selected value present — no placeholder hook.
        assert_eq!(map.get("placeholder"), Some(&json!(false)));
        assert_eq!(map.get("valid"), Some(&Value::Null));
        assert_eq!(map.get("touched"), Some(&json!(true)));
    }

    #[test]
    fn state_map_placeholder_in_multiple_mode_reads_the_array() {
        let store = store_with();
        store.set_field(|state| &mut state.selection_mode, "multiple".into());
        store.set_field(|state| &mut state.selected_value, json!([]));
        let map = trigger_state_map(&store, false, false, false, None, false);
        // The empty array counts as no selected value (`:1261-1282`).
        assert_eq!(map.get("placeholder"), Some(&json!(true)));
    }

    #[test]
    fn popup_side_is_null_while_unmounted() {
        // `usePopupSide` — the retained side is not surfaced while the
        // positioner is gone.
        let store = store_with();
        store.set_field(|state| &mut state.popup_side, Some("top".into()));
        store.set_field(|state| &mut state.mounted, true);
        let map = trigger_state_map(&store, false, false, false, None, false);
        assert_eq!(map.get("popupSide"), Some(&Value::Null));
    }
}
