//! Tests for the combobox root's mutator layer ([`crate::combobox::root_runtime`]).
//!
//! Host tests over a fake event type, mirroring the behavior claims the upstream
//! suite pins (`root/ComboboxRoot.test.tsx`): the highlight dedup, the
//! indices-write + emit path, the typed-input classification and its side
//! effects, the query freeze/release cycle on close/reopen, the close-path input
//! clears with their reasons and custom `isItemPress` flag, the selection funnel's
//! toggle/link/cancel behavior, and the unmount reconciliation.

#[cfg(test)]
mod root_runtime_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use serde_json::{Value, json};

    use crate::combobox::root_runtime::{
        IndicesOptions, PendingQueryHighlight, REASON_INPUT_CHANGE, REASON_INPUT_CLEAR,
        REASON_ITEM_PRESS, RootMode, RootRuntimeCallbacks, RootRuntimeHandles,
        stringify_value_label,
    };
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};

    /// The fake event the host tests instantiate the details type over — the same
    /// genericity `BaseUIChangeEventDetails`'s module docs describe.
    #[derive(Clone, Debug, PartialEq)]
    struct FakeEvent {
        event_type: String,
        input_type: Option<String>,
        target: Option<usize>,
    }

    impl FakeEvent {
        fn typed(input_type: &str) -> Self {
            Self {
                event_type: "input".into(),
                input_type: Some(input_type.into()),
                target: None,
            }
        }
    }

    /// The log every callback writes to — the observation surface the assertions
    /// read.
    #[derive(Default)]
    struct Log {
        entries: RefCell<Vec<String>>,
        input: RefCell<String>,
        open: RefCell<bool>,
        value: RefCell<Value>,
        canceled: RefCell<bool>,
        /// Whether the composition puts the input inside the popup.
        inside_popup: RefCell<bool>,
    }

    impl Log {
        fn push(&self, entry: String) {
            self.entries.borrow_mut().push(entry);
        }
        fn calls(&self) -> Vec<String> {
            self.entries.borrow().clone()
        }
    }

    fn callbacks(
        log: &Rc<Log>,
        store: &Rc<leptos_ui_utils::react_store::ReactStore<ComboboxState, ComboboxStoreContext>>,
    ) -> RootRuntimeCallbacks<FakeEvent> {
        let log_cb = Rc::clone(log);
        let store_cb = Rc::clone(store);
        RootRuntimeCallbacks {
            set_open_unwrapped: Rc::new(move |open| {
                *log_cb.open.borrow_mut() = open;
                // Mirror into the store's `open` — upstream's `setOpenUnwrapped` is
                // the same commit that feeds the store (the store field is the
                // state the mutators re-read).
                store_cb.set_field(|state| &mut state.open, open);
            }),
            set_input_value_unwrapped: {
                let log_cb = Rc::clone(log);
                Rc::new(move |next| *log_cb.input.borrow_mut() = next)
            },
            set_selected_value_unwrapped: {
                let log_cb = Rc::clone(log);
                let store_cb = Rc::clone(store);
                Rc::new(move |value| {
                    *log_cb.value.borrow_mut() = value.clone();
                    // The same store-mirror the open commit carries: the selection
                    // toggle re-reads `selectedValue` from the store state.
                    store_cb.set_field(|state| &mut state.selected_value, value);
                })
            },
            on_open_change: {
                let log_cb = Rc::clone(log);
                Rc::new(move |open, details| {
                    log_cb.push(format!("onOpenChange({open},{})", details.reason));
                    if *log_cb.canceled.borrow() {
                        details.cancel();
                    }
                })
            },
            on_input_value_change: {
                let log_cb = Rc::clone(log);
                Rc::new(move |next, details| {
                    log_cb.push(format!(
                        "onInputValueChange({next:?},{},{:?})",
                        details.reason, details.custom
                    ))
                })
            },
            on_selected_value_change: {
                let log_cb = Rc::clone(log);
                Rc::new(move |value, details| {
                    log_cb.push(format!("onSelectedValueChange({value},{})", details.reason));
                    if *log_cb.canceled.borrow() {
                        details.cancel();
                    }
                })
            },
            on_item_highlighted: {
                let log_cb = Rc::clone(log);
                Rc::new(move |value, index, reason| {
                    log_cb.push(format!(
                        "onItemHighlighted({value:?},{},{reason})",
                        index as isize
                    ))
                })
            },
            set_close_query: {
                let log_cb = Rc::clone(log);
                Rc::new(move |query| {
                    log_cb.push(format!("setCloseQuery({query:?})"));
                })
            },
            set_query_changed_after_open: {
                let log_cb = Rc::clone(log);
                Rc::new(move |changed| {
                    log_cb.push(format!("setQueryChangedAfterOpen({changed})"));
                })
            },
            set_touched: {
                let log_cb = Rc::clone(log);
                Rc::new(move |touched| log_cb.push(format!("setTouched({touched})")))
            },
            set_focused: {
                let log_cb = Rc::clone(log);
                Rc::new(move |focused| log_cb.push(format!("setFocused({focused})")))
            },
            validation_commit: {
                let log_cb = Rc::clone(log);
                Rc::new(move |value| log_cb.push(format!("validation.commit({value})")))
            },
            on_open_change_complete: {
                let log_cb = Rc::clone(log);
                Rc::new(move |open| log_cb.push(format!("onOpenChangeComplete({open})")))
            },
            input_value: {
                let log_cb = Rc::clone(log);
                Rc::new(move || log_cb.input.borrow().clone())
            },
            input_inside_popup: {
                let log_cb = Rc::clone(log);
                Rc::new(move || *log_cb.inside_popup.borrow())
            },
            event_input_info: Rc::new(|event: &FakeEvent| {
                (event.event_type.clone(), event.input_type.clone())
            }),
            event_target: Rc::new(|_event: &FakeEvent| None),
            synthetic_event: Rc::new(|| FakeEvent {
                event_type: "base-ui".into(),
                input_type: None,
                target: None,
            }),
        }
    }

    fn state(selection_mode: &str) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: Some(vec![json!("Apple"), json!("Banana"), json!("Cherry")]),
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

    fn runtime(selection_mode: &str) -> (Rc<RootRuntimeHandles<FakeEvent>>, Rc<Log>) {
        let log = Rc::new(Log::default());
        let store = Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
            state(selection_mode),
            ComboboxStoreContext::default(),
        ));
        let mode = RootMode::from_state(&store.select(|state| state.clone()));
        let runtime = RootRuntimeHandles::new(Rc::clone(&store), mode, callbacks(&log, &store));
        (runtime, log)
    }

    fn open_the(runtime: &RootRuntimeHandles<FakeEvent>, log: &Log) {
        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            "none",
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_open(true, &details);
        log.entries.borrow_mut().clear();
    }

    // --- emit_highlight -------------------------------------------------------

    #[test]
    fn first_clear_highlight_is_suppressed_by_the_sentinel() {
        let (runtime, log) = runtime("single");
        // Index -1 from the initial sentinel: no emission (upstream
        // `AriaCombobox.tsx:602-607` — the dedup keeps the mount-time clear from
        // firing `onItemHighlighted`).
        runtime.emit_highlight(None, -1, "none");
        assert!(log.calls().is_empty(), "no calls: {:?}", log.calls());
    }

    #[test]
    fn second_clear_after_a_real_highlight_emits_once() {
        let (runtime, log) = runtime("single");
        runtime.emit_highlight(Some(json!("Apple")), 0, "keyboard");
        assert_eq!(
            log.calls(),
            vec!["onItemHighlighted(Some(String(\"Apple\")),0,keyboard)"]
        );
        log.entries.borrow_mut().clear();

        runtime.emit_highlight(None, -1, "none");
        assert_eq!(log.calls(), vec!["onItemHighlighted(None,-1,none)"]);
    }

    // --- set_indices ----------------------------------------------------------

    #[test]
    fn set_indices_writes_both_indices_atomically_and_emits_the_value() {
        let (runtime, log) = runtime("single");
        runtime
            .store
            .context
            .values_ref
            .borrow_mut()
            .extend([json!("Apple"), json!("Banana")]);

        runtime.set_indices(IndicesOptions {
            active_index: Some(Some(1)),
            selected_index: Some(Some(0)),
            reason: Some("keyboard".into()),
        });

        assert_eq!(runtime.store.select(|state| state.active_index), Some(1));
        assert_eq!(runtime.store.select(|state| state.selected_index), Some(0));
        assert_eq!(
            log.calls(),
            vec!["onItemHighlighted(Some(String(\"Banana\")),1,keyboard)"]
        );
    }

    #[test]
    fn set_indices_clearing_emits_the_minus_one_path_and_defaults_the_reason() {
        let (runtime, log) = runtime("single");
        runtime.set_indices(IndicesOptions {
            active_index: Some(None),
            ..Default::default()
        });
        // From the initial sentinel the first clear is suppressed (the dedup).
        assert!(log.calls().is_empty(), "no calls: {:?}", log.calls());

        runtime.emit_highlight(Some(json!("Apple")), 0, "none");
        log.entries.borrow_mut().clear();
        runtime.set_indices(IndicesOptions {
            active_index: Some(None),
            ..Default::default()
        });
        assert_eq!(log.calls(), vec!["onItemHighlighted(None,-1,none)"]);
    }

    #[test]
    fn set_indices_without_an_active_index_skips_the_emission() {
        let (runtime, log) = runtime("single");
        runtime.set_indices(IndicesOptions {
            selected_index: Some(Some(2)),
            ..Default::default()
        });
        assert_eq!(runtime.store.select(|state| state.selected_index), Some(2));
        assert!(log.calls().is_empty(), "no calls: {:?}", log.calls());
    }

    // --- set_input_value ------------------------------------------------------

    #[test]
    fn typed_input_schedules_the_pending_highlight_and_flags_the_query() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_INPUT_CHANGE,
            FakeEvent::typed("insertText"),
            None,
            (false,),
        );
        runtime.set_input_value("Ap".into(), &details);

        assert_eq!(log.input.borrow().as_str(), "Ap");
        assert_eq!(
            *runtime.pending_query_highlight.borrow(),
            Some(PendingQueryHighlight {
                has_query: true,
                ..Default::default()
            })
        );
        // Auto-highlight arms index 0 on a query with no active index and an open popup.
        assert_eq!(runtime.store.select(|state| state.active_index), Some(0));
        assert!(
            log.calls()
                .iter()
                .any(|entry| entry == "setQueryChangedAfterOpen(true)")
        );
    }

    #[test]
    fn non_typed_input_classification_excludes_autofill_and_includes_composition() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);

        // Autofill (`insertReplacementText`) is not typed input: no pending
        // highlight, no auto-highlight.
        let autofill = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_INPUT_CHANGE,
            FakeEvent::typed("insertReplacementText"),
            None,
            (false,),
        );
        runtime.set_input_value("Apple".into(), &autofill);
        assert_eq!(*runtime.pending_query_highlight.borrow(), None);
        assert_eq!(runtime.store.select(|state| state.active_index), None);

        // A composition commit is typed input even with a missing inputType.
        let composition = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_INPUT_CHANGE,
            FakeEvent {
                event_type: "compositionend".into(),
                input_type: None,
                target: None,
            },
            None,
            (false,),
        );
        runtime.set_input_value("Ap".into(), &composition);
        assert!(runtime.pending_query_highlight.borrow().is_some());
    }

    #[test]
    fn empty_input_clear_with_input_inside_popup_schedules_the_selection_restore() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);
        *log.inside_popup.borrow_mut() = true;

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_INPUT_CLEAR,
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_input_value(String::new(), &details);

        assert_eq!(
            *runtime.pending_query_highlight.borrow(),
            Some(PendingQueryHighlight {
                has_query: false,
                selection: true,
                toggled_value: None
            })
        );
    }

    #[test]
    fn canceled_input_change_does_not_commit_or_schedule() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_INPUT_CHANGE,
            FakeEvent::typed("insertText"),
            None,
            (false,),
        );
        details.cancel();
        runtime.set_input_value("Ap".into(), &details);

        assert_eq!(log.input.borrow().as_str(), "", "no commit");
        assert_eq!(*runtime.pending_query_highlight.borrow(), None);
    }

    // --- the query freeze/release cycle ----------------------------------------

    #[test]
    fn single_mode_close_freezes_the_query_and_reopen_releases_it() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);
        *log.input.borrow_mut() = "Ap".into();
        runtime.query_changed_after_open.set(true);

        // Close: the query freezes for the exit animation (upstream
        // `AriaCombobox.tsx:780-787`).
        let close_details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            "none",
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_open(false, &close_details);
        assert_eq!(
            runtime.close_query.borrow().as_deref(),
            Some("Ap"),
            "the query froze"
        );
        assert!(!runtime.store.select(|state| state.open));

        // Reopen: the frozen query releases (upstream `AriaCombobox.tsx:762-764`
        // routing into `handleInterruptedReopen`).
        runtime.set_open(true, &close_details);
        assert_eq!(runtime.close_query.borrow().as_ref(), None, "released");
        assert!(
            log.calls()
                .iter()
                .any(|entry| entry == "setCloseQuery(None)")
        );
    }

    #[test]
    fn multiple_mode_close_clears_the_input_outside_the_popup_with_is_item_press() {
        let (runtime, log) = runtime("multiple");
        open_the(&runtime, &log);
        *log.input.borrow_mut() = "Ba".into();
        runtime.query_changed_after_open.set(true);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_ITEM_PRESS,
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_open(false, &details);

        // The input cleared with reason input-clear and the isItemPress custom flag
        // (upstream `AriaCombobox.tsx:793-809`).
        assert_eq!(log.input.borrow().as_str(), "");
        let calls = log.calls();
        let clear_call = calls
            .iter()
            .find(|entry| entry.starts_with("onInputValueChange(\"\""))
            .expect("a clear call");
        assert!(clear_call.contains("input-clear"), "reason: {clear_call}");
        assert!(clear_call.contains("(true,)"), "isItemPress: {clear_call}");
        // The query also froze for the exit animation.
        assert_eq!(runtime.close_query.borrow().as_deref(), Some("Ba"));
    }

    #[test]
    fn multiple_mode_close_inside_the_popup_defers_the_clear() {
        let (runtime, log) = runtime("multiple");
        open_the(&runtime, &log);
        *log.inside_popup.borrow_mut() = true;
        *log.input.borrow_mut() = "Ba".into();
        runtime.query_changed_after_open.set(true);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            "none",
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_open(false, &details);

        // The input keeps its value (the clear defers to unmount) but the active
        // index cleared (`AriaCombobox.tsx:790-792`).
        assert_eq!(log.input.borrow().as_str(), "Ba");
        assert_eq!(runtime.store.select(|state| state.active_index), None);
    }

    #[test]
    fn canceled_open_change_prevents_the_commit_and_the_freeze() {
        let (runtime, log) = runtime("single");
        *log.canceled.borrow_mut() = true;

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            "none",
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_open(true, &details);
        assert!(runtime.store.select(|state| state.open) == false);
        assert_eq!(log.input.borrow().as_str(), "");
    }

    // --- set_selected_value ----------------------------------------------------

    #[test]
    fn single_mode_selection_outside_the_popup_fills_the_input() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_ITEM_PRESS,
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_selected_value(json!("Apple"), &details);

        assert_eq!(*log.value.borrow(), json!("Apple"));
        // The input filled from the selection's label (`AriaCombobox.tsx:845-852`).
        assert_eq!(log.input.borrow().as_str(), "Apple");
        // The fill's details carry the originating reason.
        assert!(
            log.calls()
                .iter()
                .any(|entry| entry.contains("input-change") || entry.contains("item-press"))
        );
    }

    #[test]
    fn multiple_mode_selection_does_not_fill_the_input() {
        let (runtime, log) = runtime("multiple");
        open_the(&runtime, &log);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_ITEM_PRESS,
            FakeEvent::typed(""),
            None,
            (false,),
        );
        runtime.set_selected_value(json!(["Apple"]), &details);
        assert_eq!(log.input.borrow().as_str(), "");
    }

    #[test]
    fn canceled_value_change_prevents_commit_and_fill() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);

        let details = crate::combobox::root_runtime::RuntimeDetails::<FakeEvent>::new(
            REASON_ITEM_PRESS,
            FakeEvent::typed(""),
            None,
            (false,),
        );
        details.cancel();
        runtime.set_selected_value(json!("Apple"), &details);

        assert_eq!(*log.value.borrow(), Value::Null);
        assert_eq!(log.input.borrow().as_str(), "");
    }

    // --- handle_selection ------------------------------------------------------

    #[test]
    fn single_mode_handle_selection_selects_then_closes() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);

        let event = FakeEvent::typed("");
        runtime.handle_selection(None, &event, json!("Banana"));

        assert_eq!(*log.value.borrow(), json!("Banana"));
        assert!(!runtime.store.select(|state| state.open), "closed");
        assert!(
            log.calls()
                .iter()
                .any(|entry| entry == "onOpenChange(false,item-press)")
        );
    }

    #[test]
    fn multiple_mode_handle_selection_toggles_and_keeps_the_popup_open() {
        let (runtime, log) = runtime("multiple");
        open_the(&runtime, &log);

        let event = FakeEvent::typed("");
        runtime.handle_selection(None, &event, json!("Apple"));
        assert_eq!(*log.value.borrow(), json!(["Apple"]));
        assert!(runtime.store.select(|state| state.open), "stays open");
        // Not filtering (empty input): no clear, no close.
        assert!(
            !log.calls()
                .iter()
                .any(|entry| entry.starts_with("onInputValueChange(\"\""))
        );

        // Toggle off.
        runtime.handle_selection(None, &event, json!("Apple"));
        assert_eq!(*log.value.borrow(), json!([]));
    }

    #[test]
    fn multiple_mode_handle_selection_while_filtering_clears_the_query() {
        let (runtime, log) = runtime("multiple");
        open_the(&runtime, &log);
        *log.input.borrow_mut() = "Ba".into();
        // Typing through `setInputValue` is what sets the flag upstream; the test
        // sets it directly (the typed-input path has its own test above).
        runtime.query_changed_after_open.set(true);

        let event = FakeEvent::typed("");
        runtime.handle_selection(None, &event, json!("Banana"));
        assert_eq!(*log.value.borrow(), json!(["Banana"]));
        assert_eq!(log.input.borrow().as_str(), "");
        let calls = log.calls();
        let clear_call = calls
            .iter()
            .find(|entry| entry.starts_with("onInputValueChange(\"\""))
            .expect("a clear call");
        assert!(clear_call.contains("input-clear"));
    }

    #[test]
    fn canceled_value_change_halts_the_selection_funnel() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);
        *log.canceled.borrow_mut() = true;

        let event = FakeEvent::typed("");
        runtime.handle_selection(None, &event, json!("Banana"));

        assert_eq!(*log.value.borrow(), Value::Null, "no commit");
        assert!(runtime.store.select(|state| state.open), "no close");
    }

    // --- handle_unmount --------------------------------------------------------

    #[test]
    fn unmount_notifies_completion_resets_flags_and_syncs_the_single_input() {
        let (runtime, log) = runtime("single");
        open_the(&runtime, &log);
        *log.input.borrow_mut() = "typed filter".into();
        runtime.query_changed_after_open.set(true);
        *runtime.close_query.borrow_mut() = Some("typed".into());
        runtime.store.set_field(|state| &mut state.mounted, true);

        runtime.handle_unmount();

        assert_eq!(log.input.borrow().as_str(), "");
        assert!(!runtime.store.select(|state| state.mounted));
        assert_eq!(runtime.close_query.borrow().as_ref(), None);
        assert_eq!(runtime.store.select(|state| state.active_index), None);
        assert!(
            log.calls()
                .iter()
                .any(|entry| entry == "onOpenChangeComplete(false)")
        );
        // The clear of a typed filter (no selection) carries the input-clear reason.
        let calls = log.calls();
        let clear_call = calls
            .iter()
            .find(|entry| entry.starts_with("onInputValueChange(\"\""))
            .expect("a clear call");
        assert!(clear_call.contains("input-clear"), "reason: {clear_call}");
    }

    #[test]
    fn unmount_in_none_mode_clears_both_indices() {
        let (runtime, log) = runtime("none");
        open_the(&runtime, &log);
        runtime
            .store
            .set_field(|state| &mut state.active_index, Some(1));
        runtime
            .store
            .set_field(|state| &mut state.selected_index, Some(1));

        runtime.handle_unmount();

        assert_eq!(runtime.store.select(|state| state.active_index), None);
        assert_eq!(runtime.store.select(|state| state.selected_index), None);
    }

    // --- stringify_value_label --------------------------------------------------

    #[test]
    fn label_stringify_uses_the_callback_then_the_record_fields() {
        assert_eq!(stringify_value_label(&json!("plain"), None), "plain");
        assert_eq!(
            stringify_value_label(&json!({"label": "Apple", "value": 1}), None),
            "Apple"
        );
        assert_eq!(stringify_value_label(&json!({"value": 7}), None), "7");
    }
}
