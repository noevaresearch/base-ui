//! Combobox tests — the store/pipeline/parts-util slice of `library: combobox`.
//!
//! Host tests cover the pure-logic layers (the store's selector table, the
//! items/collection pipeline, the chip-removal index walk, the popup-id convention).
//! The wasm (in-browser) tests cover the JS-bound surfaces: the collator filter
//! factories (mirroring `root/utils/index.test.ts`), `clickHighlightedItem`
//! (mirroring `utils/parts.test.ts`), and `handleInputPress` (mirroring
//! `utils/handleInputPress.test.ts`).

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use serde_json::json;
    use std::rc::Rc;

    use crate::combobox::items::ItemEqualityComparer;
    use crate::combobox::items::{
        CreateComboboxItemsOptions, create_combobox_items, find_collection_item,
    };
    use crate::combobox::parts_util::{get_chip_navigation_keys, get_index_after_chip_removal};
    use crate::combobox::root_utils::{NO_ACTIVE_VALUE, get_combobox_popup_id};
    use crate::combobox::store::{ComboboxState, SetIndicesInput, selectors};

    /// A never-equal comparer as the typed `Rc<dyn Fn>` the collection API expects.
    fn no_eq() -> ItemEqualityComparer {
        Rc::new(|_a: &serde_json::Value, _b: &serde_json::Value| false)
    }

    /// A numeric-tolerant comparer as the typed `Rc<dyn Fn>` the collection API
    /// expects: a number matches its string form (the custom-equality shape
    /// upstream's scan exists for).
    fn lenient_eq() -> ItemEqualityComparer {
        Rc::new(|a: &serde_json::Value, b: &serde_json::Value| {
            a == b
                || a.as_i64()
                    .and_then(|number| {
                        b.as_str()
                            .map(|string| string.parse::<i64>().ok() == Some(number))
                    })
                    .unwrap_or(false)
                || b.as_i64()
                    .and_then(|number| {
                        a.as_str()
                            .map(|string| string.parse::<i64>().ok() == Some(number))
                    })
                    .unwrap_or(false)
        })
    }

    fn state_with(selected_value: serde_json::Value, selection_mode: &str) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: None,
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
            open_on_input_click: false,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal: false,
            auto_highlight: "always".into(),
            submit_on_item_click: false,
            has_input_value: false,
        }
    }

    // behavior.md "State model" (`store.ts:134-143`): `hasSelectedValue` — `[]`
    // reads as "no value" in multiple mode; `null`/absence reads as no value in any
    // mode; any present primitive counts.
    #[test]
    fn has_selected_value_treats_the_empty_array_as_no_value_in_multiple_mode() {
        let mut state = state_with(json!([]), "multiple");
        assert!(!selectors::has_selected_value(&state));

        state.selected_value = json!(["a"]);
        assert!(selectors::has_selected_value(&state));

        // The empty array is a *value* in single mode — only the multiple-mode
        // array branch demotes it.
        state.selection_mode = "single".into();
        state.selected_value = json!([]);
        assert!(selectors::has_selected_value(&state));

        state.selected_value = serde_json::Value::Null;
        assert!(!selectors::has_selected_value(&state));

        state.selection_mode = "none".into();
        state.selected_value = json!("text");
        assert!(selectors::has_selected_value(&state));
    }

    // behavior.md "Whole-unit cross-cutting behavior" (`store.ts:129-132`):
    // `hasSelectionChips` is "non-empty array", mode-independent.
    #[test]
    fn has_selection_chips_requires_a_non_empty_array() {
        let state = state_with(json!(["a", "b"]), "multiple");
        assert!(selectors::has_selection_chips(&state));

        let state = state_with(json!([]), "multiple");
        assert!(!selectors::has_selection_chips(&state));

        let state = state_with(json!("a"), "single");
        assert!(!selectors::has_selection_chips(&state));
    }

    // `isSelected` (`store.ts:158-167`): the array selection fans through the
    // comparer; a single selection compares directly; the default comparer is the
    // Object.is-equivalent value equality.
    #[test]
    fn is_selected_fans_the_array_through_the_comparer() {
        let state = state_with(json!(["a", "b"]), "multiple");
        assert!(selectors::is_selected(&state, &json!("a")));
        assert!(selectors::is_selected(&state, &json!("b")));
        assert!(!selectors::is_selected(&state, &json!("c")));

        let state = state_with(json!("a"), "single");
        assert!(selectors::is_selected(&state, &json!("a")));
        assert!(!selectors::is_selected(&state, &json!("b")));
    }

    // A custom `isItemEqualToValue` routes through `isSelected` — the store carries
    // the function, not a baked-in equality.
    #[test]
    fn is_selected_honors_the_state_s_custom_comparer() {
        let mut state = state_with(json!([1]), "multiple");
        state.is_item_equal_to_value =
            Rc::new(|a, b| a.as_str() == b.as_str() || (a.is_number() && b.is_number()));
        assert!(selectors::is_selected(&state, &json!(1)));
        assert!(!selectors::is_selected(&state, &json!("1")));
    }

    // `isActive` (`store.ts:157`) and the `setIndices` "leave unchanged" semantics.
    #[test]
    fn is_active_matches_only_the_exact_index() {
        let mut state = state_with(json!(null), "none");
        state.active_index = Some(2);
        assert!(selectors::is_active(&state, 2));
        assert!(!selectors::is_active(&state, 1));
    }

    // `hasNullItemLabel` (`store.ts:145-147`): the `enabled` gate short-circuits
    // before the items read.
    #[test]
    fn has_null_item_label_is_gated_by_the_enabled_flag() {
        let mut state = state_with(json!(null), "none");
        state.items = Some(vec![json!({"value": null, "label": "none"})]);
        assert!(selectors::has_null_item_label(&state, true));
        assert!(!selectors::has_null_item_label(&state, false));

        // No items prop at all: false even when enabled.
        state.items = None;
        assert!(!selectors::has_null_item_label(&state, true));
    }

    // `getComboboxPopupId` (`root/utils/index.test.ts:14-18`): derives an id only
    // when the root has one.
    #[test]
    fn popup_id_derives_only_with_a_root_id() {
        assert_eq!(
            get_combobox_popup_id(Some("fruit")),
            Some("fruit-popup".to_string())
        );
        assert_eq!(get_combobox_popup_id(None), None);
    }

    // The `NO_ACTIVE_VALUE` sentinel is not producible by any real derived value's
    // label — the load-bearing property of the upstream `Symbol('none')`.
    #[test]
    fn no_active_value_cannot_collide_with_a_json_label() {
        assert_ne!(NO_ACTIVE_VALUE, "none");
        assert!(NO_ACTIVE_VALUE.starts_with('\u{0}'));
    }

    // `getIndexAfterChipRemoval` (`utils/parts.test.ts:6-8` + `parts.ts:34-39`):
    // removing the only chip leaves no highlight; otherwise the highlight stays
    // put unless the removed chip was the last.
    #[test]
    fn chip_removal_index_walk_matches_the_upstream_matrix() {
        assert_eq!(get_index_after_chip_removal(0, 1), None);
        assert_eq!(get_index_after_chip_removal(0, 2), Some(0));
        assert_eq!(get_index_after_chip_removal(1, 2), Some(0));
        assert_eq!(get_index_after_chip_removal(1, 3), Some(1));
        assert_eq!(get_index_after_chip_removal(2, 3), Some(1));
        assert_eq!(get_index_after_chip_removal(0, 0), None);
    }

    // `getChipNavigationKeys` (`parts.ts:26-31`): RTL swaps the pair.
    #[test]
    fn chip_navigation_keys_swap_in_rtl() {
        assert_eq!(get_chip_navigation_keys("ltr"), ("ArrowLeft", "ArrowRight"));
        assert_eq!(get_chip_navigation_keys("rtl"), ("ArrowRight", "ArrowLeft"));
    }

    // `createComboboxItems` (`createItems.ts`): the lazy index, first-occurrence
    // dedup, the value projection's null passthrough, and the label fallback chain.
    #[test]
    fn collection_derives_values_and_labels_lazily_with_first_occurrence_wins() {
        let data = vec![
            json!({"id": 1, "name": "Alice"}),
            json!({"id": 2, "name": "Bob"}),
        ];
        let collection = create_combobox_items(
            Some(data),
            CreateComboboxItemsOptions {
                get_value: Rc::new(|item: &serde_json::Value| item["id"].clone()),
                get_label: Rc::new(|item: &serde_json::Value| {
                    item["name"].as_str().unwrap().to_string()
                }),
            },
        );

        // The value projection reads the accessor, not the item.
        let value = (collection.value)(&json!({"id": 2, "name": "Bob"}));
        assert_eq!(value, json!(2));

        // Labels resolve through the collection's own data.
        let label = (collection.label)(&json!(1), true, &no_eq(), None);
        assert_eq!(label, "Alice");

        // Values outside the collection fall back through the caller's fallback,
        // then the raw stringification.
        let fallback_label = (collection.label)(
            &json!(99),
            true,
            &no_eq(),
            Some(&|value| format!("fallback-{}", value)),
        );
        assert_eq!(fallback_label, "fallback-99");
        let stringified = (collection.label)(&json!("raw"), true, &no_eq(), None);
        assert_eq!(stringified, "raw");
    }

    // `data` passes through rather than defaulting: unloaded data stays the absence
    // of items, not an empty list (`createItems.ts:131-135`).
    #[test]
    fn collection_preserves_unloaded_data_as_absence() {
        let collection = create_combobox_items(
            None,
            CreateComboboxItemsOptions {
                get_value: Rc::new(|item: &serde_json::Value| item.clone()),
                get_label: Rc::new(|item: &serde_json::Value| item.to_string()),
            },
        );
        assert!(collection.data.is_none());
        // And a value cannot belong to an absent dataset.
        assert!(!(collection.has_value)(&json!("x"), true, &no_eq()));
    }

    // Grouped data flattens: the accessors receive leaves, not groups. A leaf
    // directly inside a grouped list contributes nothing (the port's documented
    // stand-in for upstream's TypeError on the mixed shape) — so the fixture uses
    // fully grouped data.
    #[test]
    fn collection_flattens_grouped_data() {
        let data = vec![
            json!({"group": "A", "items": [{"id": 1, "name": "Alice"}]}),
            json!({"group": "B", "items": [{"id": 2, "name": "Bob"}]}),
        ];
        let collection = create_combobox_items(
            Some(data),
            CreateComboboxItemsOptions {
                get_value: Rc::new(|item: &serde_json::Value| item["id"].clone()),
                get_label: Rc::new(|item: &serde_json::Value| {
                    item["name"].as_str().unwrap().to_string()
                }),
            },
        );
        assert_eq!((collection.label)(&json!(1), true, &no_eq(), None), "Alice");
        assert_eq!((collection.label)(&json!(2), true, &no_eq(), None), "Bob");
    }

    // `findCollectionItem` (`itemCollection.ts:8-25`): the exact map hit wins even
    // under a custom comparer; a custom comparer falls back to the scan; the
    // default comparer short-circuits.
    #[test]
    fn find_collection_item_splits_exact_hit_from_custom_scan() {
        let map: std::collections::HashMap<String, serde_json::Value> = [
            (json!(1).to_string(), json!("one")),
            (json!(2).to_string(), json!("two")),
        ]
        .into_iter()
        .collect();

        // Exact hit under the default equality: no scan needed.
        assert_eq!(
            find_collection_item(&map, &json!(2), true, &no_eq()),
            Some(json!("two"))
        );

        // Miss under the default equality: short-circuit, no scan — a custom
        // comparer would have matched (1 ≈ "1"), proving the branch split.
        assert_eq!(
            find_collection_item(&map, &json!("1"), true, &no_eq()),
            None
        );

        // Custom comparer: the scan compares derived values through it. The
        // string-form comparer sees derived `1` (encoded `"1"`) and queried `1`
        // (encoded `"1"`) as equal — the map miss the exact-key path could not
        // resolve under a non-key-identity equality.
        assert_eq!(
            find_collection_item(&map, &json!("1"), false, &lenient_eq()),
            Some(json!("one"))
        );
    }

    // The `setIndices` input shape: both index fields optional, `None` meaning
    // leave-unchanged — the atomically-written pair's API surface (`store.ts:102-108`).
    #[test]
    fn set_indices_input_defaults_to_leave_unchanged() {
        let input = SetIndicesInput::default();
        assert!(input.active_index.is_none());
        assert!(input.selected_index.is_none());
        assert!(input.reason.is_none());
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use serde_json::json;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::Element;

    use crate::combobox::parts_util::{
        InputPressEvent, click_highlighted_item, handle_input_press,
    };
    use crate::combobox::root_utils::{
        UseComboboxFilterOptions, create_collator_item_filter,
        create_single_selection_collator_filter, use_combobox_filter,
    };
    use crate::combobox::store::{ComboboxState, ComboboxStore, ComboboxStoreContext};
    use leptos_ui_internals::filter::{GetFilterOptions, get_filter};

    wasm_bindgen_test_configure!(run_in_browser);

    fn filter_for_locale() -> leptos_ui_internals::filter::Filter {
        get_filter(GetFilterOptions {
            locale: Some(wasm_bindgen::JsValue::from_str("en")),
            collator_options: None,
        })
    }

    // behavior.md "Derived-value pipeline" + `root/utils/index.test.ts:23-32`:
    // `createCollatorItemFilter` rejects nullish items and filters primitives and
    // projected objects.
    #[wasm_bindgen_test]
    fn collator_item_filter_rejects_nullish_and_filters_projected_objects() {
        let filter = create_collator_item_filter(filter_for_locale(), None);
        assert!(!filter(&serde_json::Value::Null, "app"));
        assert!(filter(&json!("Apple"), "app"));
        assert!(!filter(&json!("Banana"), "app"));

        let object_filter = create_collator_item_filter(
            filter_for_locale(),
            Some(crate::combobox::root_utils::FilterItemToString {
                to_string: Some(Rc::new(|item: &serde_json::Value| {
                    item["name"].as_str().unwrap().to_string()
                })),
                selected: None,
            }),
        );
        assert!(object_filter(&json!({"name": "Banana"}), "nan"));
    }

    // `createSingleSelectionCollatorFilter` (`root/utils/index.test.ts:34-51`):
    // empty query shows all; nullish items reject; the query matching the selected
    // label exactly shows everything; otherwise normal filtering.
    #[wasm_bindgen_test]
    fn single_selection_filter_shows_all_for_empty_query_and_exact_selected_match() {
        let plain = create_single_selection_collator_filter(filter_for_locale(), None, None);
        assert!(!plain(&serde_json::Value::Null, "app"));
        assert!(plain(&json!("Apple"), ""));

        let with_selection = create_single_selection_collator_filter(
            filter_for_locale(),
            None,
            Some(json!("Apple")),
        );
        // The query exactly matches the selected label (case-insensitively) → all
        // items visible, even non-matching ones.
        assert!(with_selection(&json!("Banana"), "apple"));
        // A partial match filters normally.
        assert!(!with_selection(&json!("Banana"), "app"));
        assert!(with_selection(&json!("Banana"), "nan"));
    }

    // `useComboboxFilter` (`useFilter.ts:33-49`): the mode split — multiple mode
    // filters strictly, single mode keeps every item visible while the query
    // matches the selection.
    #[wasm_bindgen_test]
    fn combobox_filter_composition_splits_by_mode() {
        let multiple = use_combobox_filter(UseComboboxFilterOptions {
            multiple: true,
            ..UseComboboxFilterOptions::default()
        });
        assert!(!(multiple.contains)(&json!("Banana"), "apple"));

        let single = use_combobox_filter(UseComboboxFilterOptions {
            multiple: false,
            value: Some(json!("Apple")),
            ..UseComboboxFilterOptions::default()
        });
        assert!((single.contains)(&json!("Banana"), "apple"));
    }

    fn fresh_state() -> ComboboxState {
        ComboboxState {
            id: None,
            label_id: None,
            items: None,
            selected_value: serde_json::Value::Null,
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
            selection_mode: "none".into(),
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
            auto_highlight: "always".into(),
            submit_on_item_click: false,
            has_input_value: false,
        }
    }

    // `clickHighlightedItem` (`utils/parts.test.ts:10-42`): a no-op when the
    // highlighted item is not rendered; otherwise clicks the rendered item with the
    // originating event staged in `selectionEventRef` and cleared after.
    #[wasm_bindgen_test]
    fn click_highlighted_item_stages_the_originating_event_and_clicks() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let document = web_sys::window().unwrap().document().unwrap();
        let store = ComboboxStore::with_context(fresh_state(), ComboboxStoreContext::default());
        let item = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&item).unwrap();

        // Highlighted item not rendered → no-op, no staged event.
        let native = web_sys::KeyboardEvent::new("keydown").unwrap();
        click_highlighted_item(&store, 1, &native);
        assert!(store.context.selection_event_ref.borrow().is_none());

        // Rendered at index 0 → clicked, the event visible during the click, then
        // cleared.
        store.context.list_ref.borrow_mut().push(Some(item.clone()));
        let clicked_during = Rc::new(std::cell::Cell::new(false));
        let selection_flag = Rc::clone(&store.context.selection_event_ref);
        let listener = {
            let clicked_during = Rc::clone(&clicked_during);
            let native_for_probe = native.clone();
            move |_event: &web_sys::Event| {
                clicked_during.set(
                    selection_flag.borrow().as_ref()
                        == Some(
                            &native_for_probe
                                .clone()
                                .dyn_into::<web_sys::Event>()
                                .unwrap(),
                        ),
                );
            }
        };
        let handler: Box<dyn FnMut(&web_sys::Event)> = Box::new(listener);
        let closure = wasm_bindgen::closure::Closure::wrap(handler);
        item.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        std::mem::forget(closure);

        click_highlighted_item(&store, 0, &native);
        assert!(
            clicked_during.get(),
            "the originating event is staged during the click"
        );
        assert!(
            store.context.selection_event_ref.borrow().is_none(),
            "cleared after the click"
        );
    }

    // `handleInputPress` (`utils/handleInputPress.test.ts:7-31`): an event whose
    // target is not an Element still preventDefaults and focuses the input.
    #[wasm_bindgen_test]
    fn input_press_with_a_non_element_target_prevents_default_and_focuses() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let document = web_sys::window().unwrap().document().unwrap();
        let current_target: Element = document.create_element("div").unwrap().into();
        let input: Element = document.create_element("input").unwrap().into();
        document.body().unwrap().append_child(&input).unwrap();
        let focused = Rc::new(std::cell::Cell::new(false));
        let closure = {
            let focused = Rc::clone(&focused);
            wasm_bindgen::closure::Closure::wrap(
                Box::new(move || focused.set(true)) as Box<dyn FnMut()>
            )
        };
        input
            .add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref())
            .unwrap();
        std::mem::forget(closure);

        let store = ComboboxStore::with_context(fresh_state(), ComboboxStoreContext::default());
        *store.context.input_ref.borrow_mut() = Some(input.clone());

        let prevent_default = Rc::new(std::cell::Cell::new(false));
        let event = InputPressEvent {
            base_ui_handler_prevented: false,
            current_target: Some(&current_target),
            native_event: None, // no native event → no target at all
            prevent_default: Rc::clone(&prevent_default),
        };

        handle_input_press(&event, &store, false, None);
        assert!(prevent_default.get(), "the default is suppressed");
        assert!(focused.get(), "the input is focused");
    }

    // `handleInputPress` (`handleInputPress.ts:11-13`): a prevented Base UI handler
    // vetoes the whole funnel.
    #[wasm_bindgen_test]
    fn input_press_vetoes_when_the_base_ui_handler_was_prevented() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let document = web_sys::window().unwrap().document().unwrap();
        let current_target: Element = document.create_element("div").unwrap().into();
        let input: Element = document.create_element("input").unwrap().into();
        document.body().unwrap().append_child(&input).unwrap();
        let focused = Rc::new(std::cell::Cell::new(false));
        let closure = {
            let focused = Rc::clone(&focused);
            wasm_bindgen::closure::Closure::wrap(
                Box::new(move || focused.set(true)) as Box<dyn FnMut()>
            )
        };
        input
            .add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref())
            .unwrap();
        std::mem::forget(closure);

        let store = ComboboxStore::with_context(fresh_state(), ComboboxStoreContext::default());
        *store.context.input_ref.borrow_mut() = Some(input);

        let prevent_default = Rc::new(std::cell::Cell::new(false));
        let event = InputPressEvent {
            base_ui_handler_prevented: true,
            current_target: Some(&current_target),
            native_event: None,
            prevent_default: Rc::clone(&prevent_default),
        };
        assert!(!handle_input_press(&event, &store, false, None));
        assert!(!prevent_default.get());
        assert!(!focused.get());
    }

    // `handleInputPress` (`handleInputPress.ts:16-24`): a press on a different,
    // interactive target is ignored; a disabled press preventDefaults but does not
    // focus; `openOnInputClick` opens with the `input-press` reason.
    #[wasm_bindgen_test]
    fn input_press_matrix_ignores_interactive_targets_and_opens_on_click() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let document = web_sys::window().unwrap().document().unwrap();
        let current_target: Element = document.create_element("div").unwrap().into();
        let button: Element = document.create_element("button").unwrap().into();
        document.body().unwrap().append_child(&button).unwrap();
        // The veto case needs a REAL event target: `Reflect::set`-ing an own
        // "target" property does not stage the event's internal slot, so
        // `getTarget` (→ `event.target()`) would still see nothing. Dispatch a
        // genuine mousedown on the button and run the funnel inside its
        // listener, where `event.target()` resolves to the button.
        let native = web_sys::MouseEvent::new("mousedown").unwrap();
        let vetoed = Rc::new(std::cell::Cell::new(false));
        let veto_probe = Rc::new(std::cell::Cell::new(false));
        let veto_flag = Rc::clone(&vetoed);
        let veto_default = Rc::clone(&veto_probe);
        let listener_document = document.clone();
        {
            let veto_flag = Rc::clone(&veto_flag);
            let veto_default = Rc::clone(&veto_default);
            let listener = move |event: web_sys::Event| {
                let document = &listener_document;
                let store =
                    ComboboxStore::with_context(fresh_state(), ComboboxStoreContext::default());
                let input: Element = document.create_element("input").unwrap().into();
                document.body().unwrap().append_child(&input).unwrap();
                *store.context.input_ref.borrow_mut() = Some(input);
                let current_target = document.create_element("div").unwrap();
                let ignored = handle_input_press(
                    &InputPressEvent {
                        base_ui_handler_prevented: false,
                        current_target: Some(&current_target.into()),
                        native_event: Some(&event),
                        prevent_default: Rc::clone(&veto_default),
                    },
                    &store,
                    false,
                    None,
                );
                veto_flag.set(!ignored);
            };
            let closure = wasm_bindgen::closure::Closure::wrap(
                Box::new(listener) as Box<dyn FnMut(web_sys::Event)>
            );
            button
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            std::mem::forget(closure);
        }
        // Interactive target (a button) different from the current target → veto.
        let _ = button.dispatch_event(&native).unwrap();
        assert!(vetoed.get(), "the interactive target vetoes the funnel");
        assert!(
            !veto_probe.get(),
            "the vetoed funnel never touches preventDefault"
        );

        let store = ComboboxStore::with_context(fresh_state(), ComboboxStoreContext::default());
        let input: Element = document.create_element("input").unwrap().into();
        document.body().unwrap().append_child(&input).unwrap();
        *store.context.input_ref.borrow_mut() = Some(input);

        // Disabled: preventDefaults but never focuses.
        let focused_probe = Rc::new(std::cell::Cell::new(false));
        let closure = {
            let focused_probe = Rc::clone(&focused_probe);
            wasm_bindgen::closure::Closure::wrap(
                Box::new(move || focused_probe.set(true)) as Box<dyn FnMut()>
            )
        };
        let input2 = store.context.input_ref.borrow().as_ref().unwrap().clone();
        input2
            .add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref())
            .unwrap();
        std::mem::forget(closure);
        let event2 = InputPressEvent {
            base_ui_handler_prevented: false,
            current_target: Some(&current_target),
            native_event: None,
            prevent_default: Rc::new(std::cell::Cell::new(false)),
        };
        assert!(handle_input_press(&event2, &store, true, None));
        assert!(!focused_probe.get());

        // `openOnInputClick` → `setOpen(true, reason: 'input-press')`.
        let opened = Rc::new(std::cell::Cell::new(false));
        let reason_seen = Rc::new(std::cell::Cell::new(false));
        let opened_for_set = Rc::clone(&opened);
        let reason_for_set = Rc::clone(&reason_seen);
        let mut store = ComboboxStore::with_context(fresh_state(), ComboboxStoreContext::default());
        store.context.set_open = Rc::new(move |open, details| {
            opened_for_set.set(open);
            reason_for_set.set(details.reason == "input-press");
        });
        let event3 = InputPressEvent {
            base_ui_handler_prevented: false,
            current_target: Some(&current_target),
            native_event: None,
            prevent_default: Rc::new(std::cell::Cell::new(false)),
        };
        // Flip the flag through a state update (the store's own setter).
        store.update(|next, _| {
            next.open_on_input_click = true;
            true
        });
        handle_input_press(&event3, &store, false, None);
        assert!(opened.get(), "the press opens the popup");
        assert!(reason_seen.get(), "with the input-press reason");
    }
}
