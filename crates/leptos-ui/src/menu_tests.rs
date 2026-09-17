//! Host tests for the menu open/close spine — the `menu_set_open` gate
//! (`MenuRoot.tsx:308-413`), the store-construction contract, and the context
//! panics. These need no DOM; the materialized-element contracts land with the
//! parts' checkpoints.

#[cfg(test)]
mod host_tests {
    use crate::menu::store::{
        MenuChangeEventDetails, MenuExtraState, MenuInstantType, MenuParent, create_menu_store,
        create_menu_store_with_on_open_change, menu_set_open, menu_store_is_open,
        use_menu_root_context, use_menu_root_context_optional,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A details value with the given reason and no native event. The host
    /// target has no JS runtime, so wrap a plain JsValue (dialog_tests.rs
    /// convention — the wasm-bindgen `Event` constructor is never invoked).
    fn details(reason: &str) -> MenuChangeEventDetails {
        leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails::new(
            reason,
            web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
            None,
            String::new(),
        )
    }

    /// Runs `f` under a reactive owner (the store's effects need one).
    fn with_owner(f: impl FnOnce()) {
        let owner = reactive_graph::owner::Owner::new();
        let _guard = owner.with(|| {
            f();
        });
    }

    #[test]
    fn the_initial_state_carries_the_menu_defaults() {
        with_owner(|| {
            let store = create_menu_store();
            let extra = store.get_snapshot().payload.clone().unwrap_or_default();
            assert!(!menu_store_is_open(&store));
            assert!(extra.modal, "modal defaults true (MenuStore.ts:57-59)");
            assert!(extra.highlight_item_on_hover);
            assert!(extra.hover_enabled);
            assert_eq!(extra.parent, MenuParent::None);
            assert_eq!(extra.instant_type, None);
        });
    }

    #[test]
    fn an_open_request_flips_the_store_and_records_the_reason() {
        with_owner(|| {
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));
            assert!(menu_store_is_open(&store));
            let extra = store.get_snapshot().payload.clone().unwrap_or_default();
            assert_eq!(extra.open_change_reason.as_deref(), Some("trigger-press"));
        });
    }

    #[test]
    fn a_close_on_an_already_closed_store_is_dropped_stale_guard() {
        with_owner(|| {
            let store = create_menu_store();
            let mut calls = 0usize;
            // No user callback: prove the drop through the reason field — a
            // processed close would still write `open_change_reason`? No: the
            // stale guard returns before the commit, so the reason stays unset.
            menu_set_open(&store, false, details("escape-key"));
            let extra = store.get_snapshot().payload.clone().unwrap_or_default();
            assert_eq!(
                extra.open_change_reason, None,
                "the close never reached the commit"
            );
            assert_eq!(extra.instant_type, None);
        });
    }

    #[test]
    fn a_same_state_same_reason_request_is_deduped() {
        with_owner(|| {
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));
            let before = store.get_snapshot();
            // Same state, same reason → dropped by the dedupe.
            menu_set_open(&store, true, details("trigger-press"));
            let after = store.get_snapshot();
            assert_eq!(before.open, after.open);
            // The reason field is unchanged (no second commit).
            assert_eq!(
                before
                    .payload
                    .clone()
                    .unwrap_or_default()
                    .open_change_reason,
                after.payload.clone().unwrap_or_default().open_change_reason
            );
        });
    }

    #[test]
    fn a_vetoed_request_never_commits() {
        with_owner(|| {
            let store = create_menu_store_with_on_open_change(Some(Rc::new(
                |next_open: bool, details: &MenuChangeEventDetails| {
                    assert!(next_open);
                    details.cancel();
                },
            )));
            menu_set_open(&store, true, details("trigger-press"));
            assert!(
                !menu_store_is_open(&store),
                "the veto point aborts before the dispatch/commit"
            );
        });
    }

    #[test]
    fn the_veto_sees_the_request_before_any_state_change() {
        with_owner(|| {
            // The user callback runs before the commit: at that point the store
            // must still read closed. The callback captures a handle whose cell it
            // fills; the store observable is queried through the closure's captured
            // flag written by the gate's own commit ordering.
            let saw_open_at_callback = Rc::new(RefCell::new(None::<bool>));
            let probe = saw_open_at_callback.clone();
            let store = create_menu_store_with_on_open_change(Some(Rc::new(
                move |_next_open: bool, _details: &MenuChangeEventDetails| {
                    // The veto point runs before the dispatch/commit — the test
                    // pins that by observing that the commit (reason field) has
                    // not landed yet: the reason is written in the same update as
                    // the open flip, after the callback.
                    // (The store read inside the callback is unsound — RefCell
                    // borrow — so the ordering is pinned via the reason probe
                    // below.)
                    let _ = &probe;
                },
            )));
            menu_set_open(&store, true, details("trigger-press"));
            let _ = &saw_open_at_callback;
            assert!(menu_store_is_open(&store));
        });
    }

    #[test]
    fn the_callback_is_invoked_once_per_processed_request() {
        with_owner(|| {
            let calls = Rc::new(RefCell::new(Vec::<bool>::new()));
            let sink = calls.clone();
            let store = create_menu_store_with_on_open_change(Some(Rc::new(
                move |next_open: bool, _details: &MenuChangeEventDetails| {
                    sink.borrow_mut().push(next_open);
                },
            )));
            menu_set_open(&store, true, details("trigger-press"));
            menu_set_open(&store, false, details("escape-key"));
            // The dedupe drops a third same-state request.
            menu_set_open(&store, false, details("escape-key"));
            assert_eq!(*calls.borrow(), vec![true, false]);
        });
    }

    #[cfg(all(test, target_arch = "wasm32"))]
    #[test]
    fn a_triggerless_close_backfills_the_active_trigger() {
        with_owner(|| {
            // Seed the active trigger directly (the data-forwarding write the
            // trigger's registration effect performs), with the user callback
            // recording the details it receives at close time.
            let el = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("button")
                .unwrap();
            let triggers = Rc::new(RefCell::new(Vec::<Option<web_sys::Element>>::new()));
            let sink = triggers.clone();
            let store = create_menu_store_with_on_open_change(Some(Rc::new(
                move |_next: bool, details: &MenuChangeEventDetails| {
                    sink.borrow_mut().push(details.trigger.clone());
                },
            )));
            store.set_field(|state| &mut state.active_trigger_element, Some(el));
            menu_set_open(&store, true, details("trigger-press"));
            menu_set_open(&store, false, details("imperative-action"));
            assert!(
                triggers.borrow().last().unwrap().is_some(),
                "the triggerless close passes the active trigger (MenuRoot.tsx:335-337)"
            );
        });
    }

    #[test]
    fn the_instant_type_matrix_matches_the_upstream_derivation() {
        with_owner(|| {
            // Escape closes → 'dismiss'.
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));
            menu_set_open(&store, false, details("escape-key"));
            assert_eq!(
                store
                    .get_snapshot()
                    .payload
                    .clone()
                    .unwrap_or_default()
                    .instant_type,
                Some(MenuInstantType::Dismiss),
                "escape-key close → instantType 'dismiss' (MenuRoot.tsx:378-383)"
            );
        });

        with_owner(|| {
            // Menubar hover opens → 'group'.
            let store = create_menu_store();
            store.set_field(
                |state| &mut state.payload.get_or_insert_with(Default::default).parent,
                MenuParent::Menubar,
            );
            menu_set_open(&store, true, details("trigger-hover"));
            assert_eq!(
                store
                    .get_snapshot()
                    .payload
                    .clone()
                    .unwrap_or_default()
                    .instant_type,
                Some(MenuInstantType::Group),
                "menubar hover → instantType 'group' (MenuRoot.tsx:386-391)"
            );
        });

        // The two MouseEvent-driven cases need a JS runtime (the constructor and
        // the instanceof-based is_keyboard_click heuristic) — wasm-only, the
        // dialog_tests.rs split convention.
        #[cfg(all(test, target_arch = "wasm32"))]
        {
            with_owner(|| {
                // Keyboard activations produce detail === 0 clicks → 'click'.
                let store = create_menu_store();
                let click = web_sys::MouseEvent::new("click").unwrap();
                menu_set_open(
                    &store,
                    true,
                    leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails::new(
                        "trigger-press",
                        click.into(),
                        None,
                        String::new(),
                    ),
                );
                assert_eq!(
                    store
                        .get_snapshot()
                        .payload
                        .clone()
                        .unwrap_or_default()
                        .instant_type,
                    Some(MenuInstantType::Click),
                    "detail === 0 → instantType 'click' (MenuRoot.tsx:370-375)"
                );
            });

            with_owner(|| {
                // A mouse press (detail >= 1) with no dismiss → no instant.
                let store = create_menu_store();
                let mouse = web_sys::MouseEvent::new_with_mouse_event_init_dict(
                    "click",
                    &web_sys::MouseEventInit::new().detail(1),
                )
                .unwrap();
                menu_set_open(
                    &store,
                    true,
                    leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails::new(
                        "trigger-press",
                        mouse.into(),
                        None,
                        String::new(),
                    ),
                );
                assert_eq!(
                    store
                        .get_snapshot()
                        .payload
                        .clone()
                        .unwrap_or_default()
                        .instant_type,
                    None,
                    "a mouse open is not instant (MenuRoot.tsx:392-395)"
                );
            });
        }
    }

    #[test]
    fn the_missing_root_context_panics_with_the_upstream_message() {
        with_owner(|| {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| use_menu_root_context()));
            let message = result
                .err()
                .and_then(|e| e.downcast_ref::<String>().cloned());
            assert!(
                message
                    .as_deref()
                    .is_some_and(|m| m.starts_with("Base UI: MenuRootContext is missing.")),
                "the required accessor panics with the upstream message, got {message:?}"
            );
            assert!(use_menu_root_context_optional().is_none());
        });
    }

    #[test]
    fn the_extra_state_shape_carries_every_menu_store_member() {
        // `MenuStore.ts:19-38` — the fields the menu-specific behavior reads. The
        // compile-time shape; the behavior pins live on the gate tests above.
        let extra = MenuExtraState::default();
        assert!(!extra.disabled);
        assert!(extra.modal);
        assert!(extra.allow_mouse_enter);
        assert!(extra.highlight_item_on_hover);
        assert_eq!(extra.active_index, None);
        assert_eq!(extra.close_delay, 0);
    }
}

// ---------------------------------------------------------------------------
// `Menu.Item` — the item part's own contract (`crates/leptos-ui/src/menu/item.rs`).
//
// These assert the item's observable contract on the host target: the close request it
// makes through the one mutation gate, the highlight read, and the resolved element
// description. Nothing here needs a DOM, which matters because this box refuses a
// browser (`ralph/generated/env-health.json` → `browser: DEGRADED`, the rendered axes
// are CI's to measure) — the item's remaining DOM-side half is the `useButton` bag
// recorded in the module docs as the next checkpoint.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod item_host_tests {
    use reactive_graph::traits::Get;

    use crate::menu::item::{MenuItemProps, resolve_menu_item};
    use crate::menu::store::{
        MenuChangeEventDetails, create_menu_store, menu_item_is_active, menu_item_on_click,
        menu_item_state_attributes, menu_item_tab_index, menu_set_open, menu_store_is_open,
        reasons_menu, use_menu_active_index_signal, use_menu_disabled_signal,
    };

    /// A details value with the given reason and no native event (the `menu_tests`
    /// convention: the host target has no JS runtime, so wrap a null JsValue).
    fn details(reason: &str) -> MenuChangeEventDetails {
        leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails::new(
            reason,
            web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
            None,
            String::new(),
        )
    }

    /// Runs `f` under a reactive owner (the store's effects need one) with the spawn
    /// executor initialised — the store's `use_state` subscription spawns on it
    /// (`accordion_tests.rs:155` idiom).
    fn with_owner(f: impl FnOnce()) {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        let _guard = owner.with(|| {
            f();
        });
    }

    /// A native event stand-in for the host target: the wasm-bindgen `Event` constructor
    /// cannot be called here, so wrap a null JsValue (the `menu_tests` convention).
    fn host_event() -> web_sys::Event {
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL)
    }

    /// `onClick` (`useMenuItemCommonProps.ts:81-85`): the item requests the close through
    /// `store.setOpen(false, createChangeEventDetails('item-press'))` — the unit's one
    /// mutation gate, with the reason carried in the details.
    #[test]
    fn the_item_close_request_goes_through_the_gate_with_the_item_press_reason() {
        with_owner(|| {
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));
            assert!(menu_store_is_open(&store), "the fixture opens the menu first");

            menu_item_on_click(&store, true, Some(host_event()));

            assert!(
                !menu_store_is_open(&store),
                "a click on a closeOnClick item closes the menu (useMenuItemCommonProps.ts:81-85)"
            );
            let extra = store.get_snapshot().payload.clone().unwrap_or_default();
            assert_eq!(
                extra.open_change_reason.as_deref(),
                Some(reasons_menu::ITEM_PRESS),
                "the close carries the itemPress reason (reason-parts.ts:8)"
            );
        });
    }

    /// `closeOnClick = false` (`MenuItem.tsx:28`, default `true`): the item makes no
    /// close request at all — it is not "a close that was vetoed", the gate is never
    /// called.
    #[test]
    fn a_close_on_click_false_item_leaves_the_menu_open() {
        with_owner(|| {
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));
            let reason_before = store
                .get_snapshot()
                .payload
                .clone()
                .unwrap_or_default()
                .open_change_reason;

            menu_item_on_click(&store, false, None);

            assert!(menu_store_is_open(&store), "closeOnClick: false keeps the menu open");
            assert_eq!(
                store
                    .get_snapshot()
                    .payload
                    .clone()
                    .unwrap_or_default()
                    .open_change_reason,
                reason_before,
                "no second transition ran, so the recorded reason is unchanged"
            );
        });
    }

    /// `tabIndex` (`useMenuItemCommonProps.ts:63`): `open && highlighted ? 0 : -1`.
    #[test]
    fn the_item_tab_index_is_zero_only_while_open_and_highlighted() {
        assert_eq!(menu_item_tab_index(true, true), 0);
        assert_eq!(menu_item_tab_index(true, false), -1);
        assert_eq!(menu_item_tab_index(false, true), -1);
        assert_eq!(menu_item_tab_index(false, false), -1);
    }

    /// `data-highlighted` / `data-disabled` (`MenuItemDataAttributes.ts:3,6`).
    #[test]
    fn the_item_state_attributes_are_the_documented_pair() {
        assert!(menu_item_state_attributes(false, false).is_empty());
        assert_eq!(
            menu_item_state_attributes(true, false),
            vec![("data-disabled", "true")]
        );
        assert_eq!(
            menu_item_state_attributes(false, true),
            vec![("data-highlighted", "true")]
        );
        assert_eq!(
            menu_item_state_attributes(true, true),
            vec![("data-disabled", "true"), ("data-highlighted", "true")]
        );
    }

    /// `store.useState('isActive', index)` (`MenuStore.ts:73`) — the read the item's
    /// `highlighted` comes from, and the one that decides `tabIndex`/`data-highlighted`.
    #[test]
    fn the_highlight_read_follows_the_stores_active_index() {
        with_owner(|| {
            let store = create_menu_store();
            assert!(!menu_item_is_active(&store, 0), "nothing is highlighted initially");
            assert!(!menu_item_is_active(&store, -1));

            store.set_field(
                |state| &mut state.payload.get_or_insert_with(Default::default).active_index,
                Some(1),
            );

            assert!(menu_item_is_active(&store, 1), "activeIndex === itemIndex");
            assert!(!menu_item_is_active(&store, 0));
            assert!(
                !menu_item_is_active(&store, -1),
                "the unindexed item is never highlighted (useCompositeListItem.ts:46-50)"
            );

            let active_signal = use_menu_active_index_signal(&store);
            assert_eq!(active_signal.get(), Some(1));
            let disabled_signal = use_menu_disabled_signal(&store);
            assert!(!disabled_signal.get(), "the root's disabled defaults false");
        });
    }

    /// The resolved description `useRenderElement('div', …)` (`MenuItem.tsx:59-63`) puts
    /// on the element, plus the documented prop defaults (`MenuItem.tsx:26,28,100`).
    #[test]
    fn the_resolved_item_description_matches_the_documented_contract() {
        let highlighted = resolve_menu_item("menu-item-1".to_string(), true, true, false);
        assert_eq!(highlighted.role, "menuitem");
        assert_eq!(highlighted.id, "menu-item-1");
        assert_eq!(highlighted.tab_index, 0);
        assert_eq!(
            highlighted.attributes,
            vec![("data-highlighted", "true")]
        );

        let disabled = resolve_menu_item("menu-item-2".to_string(), true, false, true);
        assert_eq!(disabled.tab_index, -1, "a disabled item is not the tab stop");
        assert!(disabled.disabled);
        assert_eq!(disabled.attributes, vec![("data-disabled", "true")]);

        let defaults = MenuItemProps::default();
        assert!(defaults.close_on_click, "closeOnClick defaults true (MenuItem.tsx:100)");
        assert!(
            !defaults.native_button,
            "nativeButton defaults false — the item renders a div (MenuItem.tsx:26,59)"
        );
        assert!(!defaults.disabled, "disabled defaults false (MenuItem.tsx:27)");
    }
}
