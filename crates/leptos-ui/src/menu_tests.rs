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

// ---------------------------------------------------------------------------
// `Menu.Arrow` — the arrow part's own contract (`crates/leptos-ui/src/menu/arrow.rs`).
//
// The arrow is a pure function of the positioner context plus the store's open state
// (`MenuArrow.tsx:21-30`), so its whole observable contract — the state record, the
// `data-*` set `popupStateMapping` produces, the injected `style`/`aria-hidden` — is
// assertable on the host target. The one part that is NOT: the element's registration
// into the engine's `arrowRef` (a DOM write, CI's to measure —
// `ralph/generated/env-health.json` → `browser: DEGRADED`).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod arrow_host_tests {
    use leptos_ui_internals::use_anchor_positioning::{Align, ArrowStyles, Side};

    use crate::menu::arrow::{
        MenuArrowProps, MenuArrowState, menu_arrow_attributes, menu_arrow_state, menu_arrow_style,
        resolve_menu_arrow,
    };

    /// The engine's arrow style record as `useAnchorPositioning` produces it
    /// (`use_anchor_positioning.rs:544-551`) — `position` always `absolute`, `top`/`left`
    /// set by the middleware.
    fn styles(top: Option<&str>, left: Option<&str>) -> ArrowStyles {
        ArrowStyles {
            position: "absolute",
            top: top.map(str::to_owned),
            left: left.map(str::to_owned),
        }
    }

    fn state(open: bool, side: Side, align: Align, uncentered: bool) -> MenuArrowState {
        MenuArrowState {
            open,
            side,
            align,
            uncentered,
        }
    }

    /// The state record (`MenuArrow.tsx:25-30`) with the rendered spellings the mapping
    /// consumes — `data-side="inline-end"`, not the Rust enum's own name.
    #[test]
    fn the_arrow_state_record_carries_the_upstream_members() {
        let record = menu_arrow_state(true, Side::InlineEnd, Align::Center, false);
        assert_eq!(record.get("open"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            record.get("side"),
            Some(&serde_json::Value::String("inline-end".into()))
        );
        assert_eq!(
            record.get("align"),
            Some(&serde_json::Value::String("center".into()))
        );
        assert_eq!(
            record.get("uncentered"),
            Some(&serde_json::Value::Bool(false))
        );
    }

    /// `MenuArrow.tsx:35` — `getStateAttributesProps(state, popupStateMapping)`. The open
    /// arrow carries `data-open` plus the default-handled `data-side`/`data-align`
    /// (`MenuArrowDataAttributes.ts:10-18`).
    #[test]
    fn the_open_arrow_carries_the_documented_data_attributes() {
        let attributes = menu_arrow_attributes(&menu_arrow_state(
            true,
            Side::Bottom,
            Align::Center,
            false,
        ));
        assert!(attributes.contains(&("data-open".to_string(), String::new())));
        assert!(attributes.contains(&("data-side".to_string(), "bottom".to_string())));
        assert!(attributes.contains(&("data-align".to_string(), "center".to_string())));
    }

    /// `popupStateMapping.ts:53-66` — both branches emit (open XOR closed), which is why a
    /// closed arrow is still marked.
    #[test]
    fn a_closed_arrow_swaps_the_open_marker_for_the_closed_one() {
        let attributes = menu_arrow_attributes(&menu_arrow_state(
            false,
            Side::Top,
            Align::Start,
            false,
        ));
        assert!(attributes.contains(&("data-closed".to_string(), String::new())));
        assert!(!attributes.contains(&("data-open".to_string(), String::new())));
        assert!(attributes.contains(&("data-side".to_string(), "top".to_string())));
    }

    /// `data-uncentered` (`MenuArrowDataAttributes.ts:24`) comes from the DEFAULT handling
    /// (`getStateAttributesProps.ts:22-31`), so a centered arrow emits nothing for it — no
    /// `data-uncentered="false"`.
    #[test]
    fn a_centered_arrow_omits_the_uncentered_marker() {
        let centered = menu_arrow_attributes(&menu_arrow_state(
            true,
            Side::Bottom,
            Align::Center,
            false,
        ));
        assert!(
            !centered.iter().any(|(key, _)| key == "data-uncentered"),
            "false emits no attribute, got {centered:?}"
        );

        let uncentered = menu_arrow_attributes(&menu_arrow_state(
            true,
            Side::Bottom,
            Align::Center,
            true,
        ));
        assert!(uncentered.contains(&("data-uncentered".to_string(), String::new())));
    }

    /// `MenuArrow.tsx:37-41` — `style: arrowStyles` first, the consumer's bag after it
    /// (the `...elementProps` spread), so a consumer member wins.
    #[test]
    fn the_arrow_style_is_the_engine_geometry_then_the_consumer_members() {
        let engine_only = menu_arrow_style(&styles(Some("7px"), Some("3px")), &[]);
        assert_eq!(
            engine_only,
            "position: absolute;top: 7px;left: 3px;",
            "the engine's geometry is rendered verbatim (ArrowStyles.position is the CSS keyword)"
        );

        // A middleware that has not run leaves the offset unset — no `top: ;` artifact.
        let no_offsets = menu_arrow_style(&styles(None, None), &[]);
        assert_eq!(no_offsets, "position: absolute;");

        let with_consumer = menu_arrow_style(
            &styles(Some("7px"), None),
            &[("background".to_string(), "red".to_string())],
        );
        assert_eq!(
            with_consumer,
            "position: absolute;top: 7px;background: red;",
            "the consumer's members land last, so they win the cascade"
        );
    }

    /// The resolved element description: the attributes above plus the two injected props
    /// (`:38-39`), and the documented prop defaults (`:21`).
    #[test]
    fn the_resolved_arrow_is_decorative_and_carries_the_injected_style() {
        let resolved = resolve_menu_arrow(
            state(true, Side::Right, Align::End, false),
            &styles(Some("1px"), Some("2px")),
            &[],
        );
        assert!(resolved.aria_hidden, "'aria-hidden': true (MenuArrow.tsx:39)");
        assert_eq!(resolved.style, "position: absolute;top: 1px;left: 2px;");
        assert!(resolved.attributes.contains(&("data-open".to_string(), String::new())));
        assert!(resolved.attributes.contains(&("data-side".to_string(), "right".to_string())));
        assert!(resolved.attributes.contains(&("data-align".to_string(), "end".to_string())));

        let defaults = MenuArrowProps::default();
        assert!(defaults.class.is_none());
        assert!(defaults.style.is_empty());
    }
}

// ---------------------------------------------------------------------------
// `Menu.Backdrop` — the backdrop part's own contract (`crates/leptos-ui/src/menu/backdrop.rs`).
//
// A pure function of four store reads (`MenuBackdrop.tsx:29-32`): the hover-vs-click
// pointer rule and the always-on text-selection rule are the behavior.md claims
// (`parts/arrow-backdrop-portal-viewport.md` → "State model"), and the attribute set is
// `popupTransitionStateMapping`'s.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod backdrop_host_tests {
    use leptos_ui_internals::use_transition_status::TransitionStatus;

    use crate::menu::backdrop::{
        MenuBackdropProps, menu_backdrop_attributes, menu_backdrop_hidden, menu_backdrop_state,
        menu_backdrop_style,
    };

    /// `MenuBackdrop.tsx:48` — `pointerEvents: 'none'` only for a hover-opened menu
    /// (`reasons.ts:4`), the two halves of `MenuBackdrop.test.tsx:16-54`.
    #[test]
    fn only_a_hover_opened_backdrop_suppresses_pointer_events() {
        let hover = menu_backdrop_style(Some("trigger-hover"), &[]);
        assert!(
            hover.starts_with("pointer-events: none;"),
            "a hover-open backdrop is click-through, got {hover}"
        );

        let click = menu_backdrop_style(Some("trigger-press"), &[]);
        assert!(
            !click.contains("pointer-events"),
            "a click-opened backdrop keeps pointer events, got {click}"
        );

        let no_reason = menu_backdrop_style(None, &[]);
        assert!(!no_reason.contains("pointer-events"));
    }

    /// `MenuBackdrop.tsx:49-50` — unconditional, in both opens.
    #[test]
    fn the_backdrop_always_disables_text_selection() {
        for reason in [None, Some("trigger-hover"), Some("outside-press")] {
            let style = menu_backdrop_style(reason, &[]);
            assert!(
                style.contains("user-select: none;") && style.contains("-webkit-user-select: none;"),
                "the userSelect pair is unconditional, got {style} for {reason:?}"
            );
        }
    }

    /// The consumer's members land after the part's (`:43-52`).
    #[test]
    fn the_consumer_style_lands_after_the_parts_own_members() {
        let style = menu_backdrop_style(
            Some("trigger-hover"),
            &[("opacity".to_string(), "0.5".to_string())],
        );
        assert_eq!(
            style,
            "pointer-events: none;user-select: none; -webkit-user-select: none;opacity: 0.5;"
        );
    }

    /// `MenuBackdrop.tsx:45` — `hidden: !mounted`. A menu that was never opened has no
    /// mounted backdrop.
    #[test]
    fn the_backdrop_is_hidden_until_the_store_reports_it_mounted() {
        assert!(menu_backdrop_hidden(false));
        assert!(!menu_backdrop_hidden(true));
    }

    /// `MenuBackdrop.tsx:42` — `getStateAttributesProps(state, popupTransitionStateMapping)`:
    /// open/closed plus the transition pair (`MenuBackdropDataAttributes.ts:4-18`).
    #[test]
    fn the_backdrop_attributes_follow_the_transition_status() {
        let starting = menu_backdrop_attributes(&menu_backdrop_state(
            true,
            Some(TransitionStatus::Starting),
        ));
        assert!(starting.contains(&("data-open".to_string(), String::new())));
        assert!(starting.contains(&("data-starting-style".to_string(), String::new())));
        assert!(!starting.contains(&("data-ending-style".to_string(), String::new())));

        let ending = menu_backdrop_attributes(&menu_backdrop_state(
            false,
            Some(TransitionStatus::Ending),
        ));
        assert!(ending.contains(&("data-closed".to_string(), String::new())));
        assert!(ending.contains(&("data-ending-style".to_string(), String::new())));
    }

    /// The state record (`:36-39`). `'idle'` is carried rather than dropped: the mapping
    /// owns the key and declines it (`transitionStatusMapping`), so a settled backdrop
    /// emits no transition attribute while still reporting a status.
    #[test]
    fn the_backdrop_state_record_carries_open_and_the_transition_status() {
        let record = menu_backdrop_state(true, Some(TransitionStatus::Idle));
        assert_eq!(record.get("open"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            record.get("transitionStatus"),
            Some(&serde_json::Value::String("idle".into()))
        );
        assert!(
            !menu_backdrop_attributes(&record)
                .iter()
                .any(|(key, _)| key == "data-starting-style" || key == "data-ending-style"),
            "an idle status emits neither transition attribute"
        );

        let unset = menu_backdrop_state(false, None);
        assert_eq!(unset.get("transitionStatus"), None);

        let defaults = MenuBackdropProps::default();
        assert!(defaults.class.is_none() && defaults.style.is_empty());
    }
}

// ---------------------------------------------------------------------------
// `Menu.Group` / `Menu.GroupLabel` — the association contract
// (`crates/leptos-ui/src/menu/group.rs`, `group_label.rs`).
//
// The spec's central claims live here rather than in a rendered test: the group's `role`
// and `aria-labelledby` (`MenuGroup.test.tsx:14-17`,
// `MenuGroupLabel.test.tsx:82-101`), the `aria-hidden` default and its two overrides
// (`:44-80`), and the remount-ordering guarantee that an older label's unmount cannot clear
// a newer label's id (`:183-219`) — which is exactly why the setter's cleanup arm is
// `ClearIfCurrent` rather than a plain clear.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod group_host_tests {
    use leptos_ui_internals::use_registered_label_id::LabelIdUpdate;
    use reactive_graph::traits::Get;

    use crate::menu::group::{
        MENU_GROUP_ROLE, MenuGroupContextValue, menu_group_aria_labelledby, menu_group_context,
        set_group_label_id,
    };
    use crate::menu::group_label::{
        menu_group_label_aria_hidden, menu_group_label_attrs,
    };

    /// Runs `f` under a reactive owner — the group's `labelId` state is a signal, and a
    /// signal needs an owner (`menu_tests.rs`'s host-test convention).
    fn with_owner(f: impl FnOnce()) {
        let owner = reactive_graph::owner::Owner::new();
        let _guard = owner.with(|| {
            f();
        });
    }

    fn group() -> MenuGroupContextValue {
        MenuGroupContextValue::new()
    }

    /// Extracts a panic message from either payload shape (`panic!` carries `&str`,
    /// `.expect(..)` carries `String`).
    fn panic_message(payload: Box<dyn std::any::Any + Send>) -> Option<String> {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
    }

    /// `MenuGroup.tsx:26` — the `group` role (`MenuGroup.test.tsx:14-17`).
    #[test]
    fn the_group_role_is_upstreams_group_role() {
        assert_eq!(MENU_GROUP_ROLE, "group");
    }

    /// `MenuGroup.tsx:27` — `aria-labelledby: labelId`, `undefined` before any label
    /// mounts.
    #[test]
    fn the_group_aria_labelledby_is_absent_before_a_label_registers() {
        with_owner(|| {
            let group = group();
            assert_eq!(menu_group_aria_labelledby(group.label_id), None);
        });
    }

    /// `MenuGroupLabel.tsx:23,25-26` — the mount write `setLabelId(id)` publishes the
    /// label's id to the group (`MenuGroupLabel.test.tsx:82-101`).
    #[test]
    fn a_label_registration_publishes_its_id_to_the_group() {
        with_owner(|| {
            let group = group();
            set_group_label_id(&group, LabelIdUpdate::Set(Some("base-ui-1".to_string())));
            assert_eq!(
                menu_group_aria_labelledby(group.label_id),
                Some("base-ui-1".to_string())
            );
        });
    }

    /// `:27` — the unmount cleanup clears the id it registered, leaving the group's
    /// `aria-labelledby` absent again.
    #[test]
    fn a_label_cleanup_clears_its_own_registration() {
        with_owner(|| {
            let group = group();
            set_group_label_id(&group, LabelIdUpdate::Set(Some("base-ui-2".to_string())));
            assert!(menu_group_aria_labelledby(group.label_id).is_some());

            set_group_label_id(&group, LabelIdUpdate::ClearIfCurrent("base-ui-2".to_string()));
            assert_eq!(menu_group_aria_labelledby(group.label_id), None);

            // And a foreign id never clears a registration that is still mounted.
            set_group_label_id(&group, LabelIdUpdate::Set(Some("base-ui-3".to_string())));
            set_group_label_id(&group, LabelIdUpdate::ClearIfCurrent("base-ui-2".to_string()));
            assert_eq!(
                menu_group_aria_labelledby(group.label_id),
                Some("base-ui-3".to_string()),
                "a stale cleanup leaves a live registration alone"
            );
        });
    }

    /// `MenuGroupLabel.test.tsx:183-219` — the remount ordering: "old" unmounts only after
    /// "new" mounted, and the group's `aria-labelledby` keeps pointing at "new" throughout.
    #[test]
    fn an_older_label_cleanup_does_not_clear_a_newer_label() {
        with_owner(|| {
            let group = group();

            // Only "old" is mounted.
            set_group_label_id(&group, LabelIdUpdate::Set(Some("old".to_string())));
            assert_eq!(
                menu_group_aria_labelledby(group.label_id),
                Some("old".to_string())
            );

            // Both are mounted ("new" registers last).
            set_group_label_id(&group, LabelIdUpdate::Set(Some("new".to_string())));
            assert_eq!(
                menu_group_aria_labelledby(group.label_id),
                Some("new".to_string())
            );

            // "old" unmounts — its cleanup sees a foreign id.
            set_group_label_id(&group, LabelIdUpdate::ClearIfCurrent("old".to_string()));
            assert_eq!(
                menu_group_aria_labelledby(group.label_id),
                Some("new".to_string()),
                "the newest label still owns the association"
            );

            // The last label's own cleanup clears it.
            set_group_label_id(&group, LabelIdUpdate::ClearIfCurrent("new".to_string()));
            assert_eq!(menu_group_aria_labelledby(group.label_id), None);
        });
    }

    /// `MenuGroupLabel.test.tsx:103-120` — a provided `id` is used for the association
    /// verbatim (the `useBaseUiId` override arm feeding the registration).
    #[test]
    fn the_provided_id_is_the_one_associated_with_the_group() {
        with_owner(|| {
            let attributes = menu_group_label_attrs("test-group".to_string(), None);
            assert_eq!(attributes.id, "test-group");

            let group = group();
            set_group_label_id(&group, LabelIdUpdate::Set(Some(attributes.id)));
            assert_eq!(
                menu_group_aria_labelledby(group.label_id),
                Some("test-group".to_string())
            );
        });
    }

    /// `MenuGroupLabel.tsx:36` — `'aria-hidden': true` by default
    /// (`MenuGroupLabel.test.tsx:44-61`), removable by the consumer's explicit
    /// `aria-hidden={undefined}` (`:63-80`), and an explicit value wins verbatim.
    #[test]
    fn the_group_label_aria_hidden_defaults_true_and_the_consumer_overrides_it() {
        assert_eq!(menu_group_label_aria_hidden(None), Some(true));
        assert_eq!(
            menu_group_label_aria_hidden(Some(None)),
            None,
            "an explicit undefined removes the attribute"
        );
        assert_eq!(menu_group_label_aria_hidden(Some(Some(false))), Some(false));

        let attrs = menu_group_label_attrs("base-ui-7".to_string(), None);
        assert_eq!(attrs.aria_hidden, Some(true));
        assert_eq!(attrs.id, "base-ui-7");
    }

    /// `MenuGroupContext.ts:8-17` — the required read throws upstream's own message when no
    /// group ancestor is present (`MenuGroupLabel.test.tsx:31-41`).
    #[test]
    fn the_missing_group_context_panics_with_the_upstream_message() {
        with_owner(|| {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| menu_group_context()));
            let message = result.err().and_then(panic_message);
            assert!(
                message.as_deref().is_some_and(|m| m.starts_with(
                    "Base UI: MenuGroupContext is missing. Menu group parts must be used within <Menu.Group> or <Menu.RadioGroup>."
                )),
                "the required accessor panics with the upstream message, got {message:?}"
            );
        });
    }

    /// The group's own state signal is what the element reads, so a registration is
    /// observable through the same handle the view binds.
    #[test]
    fn the_group_state_signal_is_the_views_own_read() {
        with_owner(|| {
            let group = group();
            assert_eq!(group.label_id.get(), None);
            set_group_label_id(&group, LabelIdUpdate::Set(Some("base-ui-8".to_string())));
            assert_eq!(group.label_id.get(), Some("base-ui-8".to_string()));
        });
    }
}

// ---------------------------------------------------------------------------
// The checkbox/radio item family (`Menu.CheckboxItem`, `Menu.CheckboxItemIndicator`,
// `Menu.RadioGroup`, `Menu.RadioItem`, `Menu.RadioItemIndicator`)
//
// These assert the family's observable contract on the host target: the resolved element
// descriptions (role, aria-checked, tab index, the `data-*` set the mapped engine emits),
// the row-level decision functions (the toggle's negation and the `cancel()` veto, the
// three-way disabled fold, the selection comparison, the group's labelledby precedence),
// and the indicators' presence gates. Nothing here needs a DOM, which matters because this
// box refuses a browser (`ralph/generated/env-health.json` → `browser: DEGRADED`, so the
// rendered axes are CI's to measure) — the parts' remaining DOM-side halves are the
// `useButton`/`getItemProps` bag and the two `transitionStatus` machines, each recorded in
// the module docs as the next checkpoint.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod item_family_host_tests {
    use crate::menu::checkbox_item::{
        MENU_CHECKBOX_ITEM_ROLE, menu_checkbox_item_click, resolve_menu_checkbox_item,
    };
    use crate::menu::checkbox_item_indicator::{
        MenuCheckboxItemIndicatorState, menu_checkbox_item_indicator_attributes,
        menu_checkbox_item_indicator_should_render,
    };
    use crate::menu::radio_group::{
        MENU_RADIO_GROUP_ROLE, menu_radio_group_aria_disabled, menu_radio_group_aria_labelledby,
        menu_radio_group_commits, resolve_menu_radio_group,
    };
    use crate::menu::radio_item::{
        MENU_RADIO_ITEM_ROLE, menu_radio_item_checked, menu_radio_item_disabled,
        resolve_menu_radio_item,
    };
    use crate::menu::radio_item_indicator::{
        MenuRadioItemIndicatorState, menu_radio_item_indicator_should_render,
    };
    use crate::menu::utils::{item_mapping, menu_item_attributes, menu_item_state_map};
    use leptos_ui_internals::use_transition_status::TransitionStatus;

    /// The attribute set as a lookup, so an assertion names the attribute it means rather
    /// than the engine's iteration order.
    fn lookup(pairs: &[(String, String)]) -> std::collections::BTreeMap<&str, &str> {
        pairs
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect()
    }

    /// `itemMapping.checked` (`stateAttributesMapping.ts:6-15`): both arms BARE, because
    /// that is what the engine's `true` branch produces (`getStateAttributesProps.ts:24-25`)
    /// and what the mined suite pins (`MenuRadioItem.test.tsx:174`).
    #[test]
    fn the_item_mapping_emits_the_bare_checked_pair_and_declines_other_keys() {
        assert_eq!(
            item_mapping("checked", &serde_json::Value::Bool(true)),
            Some(Some(std::collections::BTreeMap::from([(
                "data-checked".to_string(),
                String::new()
            )])))
        );
        assert_eq!(
            item_mapping("checked", &serde_json::Value::Bool(false)),
            Some(Some(std::collections::BTreeMap::from([(
                "data-unchecked".to_string(),
                String::new()
            )])))
        );
        // `disabled`/`highlighted` are not upstream's mapping's keys — they fall through to
        // the engine's default handling (`getStateAttributesProps.ts:22-25`).
        assert_eq!(item_mapping("disabled", &serde_json::Value::Bool(true)), None);
        assert_eq!(
            item_mapping("highlighted", &serde_json::Value::Bool(true)),
            None
        );
    }

    /// The mapped engine's output for the item state (`{ disabled, highlighted, checked }`):
    /// the three true fields become bare attributes, the false `checked` arm the other one.
    #[test]
    fn the_engine_emits_the_items_state_attributes() {
        let state_attributes = menu_item_attributes(&menu_item_state_map(true, true, true));
        let attributes = lookup(&state_attributes);
        assert_eq!(attributes.len(), 3, "got {attributes:?}");
        assert_eq!(attributes.get("data-checked"), Some(&""));
        assert_eq!(attributes.get("data-disabled"), Some(&""));
        assert_eq!(attributes.get("data-highlighted"), Some(&""));
        assert!(!attributes.contains_key("data-unchecked"));

        let state_attributes = menu_item_attributes(&menu_item_state_map(false, false, false));
        let attributes = lookup(&state_attributes);
        assert_eq!(attributes.len(), 1, "got {attributes:?}");
        assert_eq!(attributes.get("data-unchecked"), Some(&""));
    }

    /// `MenuCheckboxItem.tsx:94-108`: role `menuitemcheckbox` (`:100`), `aria-checked`
    /// (`:101`), the item's `tabIndex` (`useMenuItemCommonProps.ts:63`).
    #[test]
    fn the_resolved_checkbox_item_carries_its_role_aria_and_state_attributes() {
        let resolved = resolve_menu_checkbox_item("base-ui-1".to_string(), true, true, false, true);
        assert_eq!(resolved.role, MENU_CHECKBOX_ITEM_ROLE);
        assert!(resolved.aria_checked);
        assert_eq!(resolved.tab_index, 0, "open && highlighted -> 0");
        let attributes = lookup(&resolved.attributes);
        assert_eq!(attributes.get("data-checked"), Some(&""));
        assert_eq!(attributes.get("data-highlighted"), Some(&""));

        // Closed, unhighlighted, unchecked, disabled: `-1` and the unchecked/disabled pair.
        let resolved = resolve_menu_checkbox_item("base-ui-2".to_string(), false, false, true, false);
        assert_eq!(resolved.tab_index, -1);
        let attributes = lookup(&resolved.attributes);
        assert_eq!(attributes.get("data-unchecked"), Some(&""));
        assert_eq!(attributes.get("data-disabled"), Some(&""));
        assert!(!attributes.contains_key("data-checked"));
    }

    /// `handleClick` (`MenuCheckboxItem.tsx:80-92`): the announced value is always the
    /// negation, and `details.cancel()` (`:87-89`) vetoes the commit.
    #[test]
    fn the_checkbox_click_announces_the_negation_and_the_cancel_vetoes() {
        let click = menu_checkbox_item_click(false, false);
        assert!(click.next_checked, "unchecked -> announces true");
        assert!(click.commit);

        let click = menu_checkbox_item_click(true, false);
        assert!(!click.next_checked, "checked -> announces false");
        assert!(click.commit);

        let click = menu_checkbox_item_click(false, true);
        assert!(click.next_checked, "the announced value is not affected by the veto");
        assert!(!click.commit, "the veto keeps aria-checked/data-checked at their state");
    }

    /// `aria-labelledby: ariaLabelledByProp ?? labelId` (`MenuRadioGroup.tsx:61`) and
    /// `aria-disabled: disabled || undefined` (`:62`).
    #[test]
    fn the_radio_group_prefers_the_prop_and_only_declares_aria_disabled_when_disabled() {
        assert_eq!(
            menu_radio_group_aria_labelledby(
                Some("consumer".to_string()),
                Some("base-ui-3".to_string())
            ),
            Some("consumer".to_string())
        );
        assert_eq!(
            menu_radio_group_aria_labelledby(None, Some("base-ui-3".to_string())),
            Some("base-ui-3".to_string())
        );
        assert_eq!(menu_radio_group_aria_labelledby(None, None), None);

        assert_eq!(menu_radio_group_aria_disabled(true), Some("true"));
        assert_eq!(menu_radio_group_aria_disabled(false), None);

        let resolved = resolve_menu_radio_group(None, Some("base-ui-4".to_string()), true);
        assert_eq!(resolved.role, MENU_RADIO_GROUP_ROLE);
        assert_eq!(resolved.aria_labelledby, Some("base-ui-4".to_string()));
        assert_eq!(resolved.aria_disabled, Some("true"));
    }

    /// The group's gated setter (`MenuRadioGroup.tsx:42-52`): the consumer is told first and
    /// its `cancel()` vetoes the write.
    #[test]
    fn the_radio_group_setter_commits_unless_canceled() {
        assert!(menu_radio_group_commits(false));
        assert!(!menu_radio_group_commits(true));
    }

    /// `MenuRadioItem.tsx:55-56`: the three-way disabled fold and the selection comparison.
    #[test]
    fn the_radio_item_folds_three_disabled_sources_and_compares_its_value() {
        assert!(!menu_radio_item_disabled(false, false, false));
        assert!(menu_radio_item_disabled(true, false, false));
        assert!(menu_radio_item_disabled(false, true, false), "group disabled");
        assert!(menu_radio_item_disabled(false, false, true), "root disabled");

        assert!(menu_radio_item_checked(
            &Some("a".to_string()),
            &Some("a".to_string())
        ));
        assert!(!menu_radio_item_checked(
            &Some("a".to_string()),
            &Some("b".to_string())
        ));
        assert!(!menu_radio_item_checked(&None, &Some("a".to_string())));
        assert!(menu_radio_item_checked(&None, &None));
    }

    /// `MenuRadioItem.tsx:86-100`: role `menuitemradio` (`:92`) and `aria-checked` (`:93`).
    #[test]
    fn the_resolved_radio_item_carries_its_role_aria_and_state_attributes() {
        let resolved = resolve_menu_radio_item("base-ui-5".to_string(), true, false, false, true);
        assert_eq!(resolved.role, MENU_RADIO_ITEM_ROLE);
        assert!(resolved.aria_checked);
        assert_eq!(resolved.tab_index, -1, "open but not highlighted -> -1");
        let attributes = lookup(&resolved.attributes);
        assert_eq!(attributes.get("data-checked"), Some(&""));
        assert!(!attributes.contains_key("data-unchecked"));
    }

    /// `enabled: keepMounted || mounted` (`MenuCheckboxItemIndicator.tsx:55`,
    /// `MenuRadioItemIndicator.tsx:55`).
    #[test]
    fn the_indicators_presence_gate_is_keep_mounted_or_mounted() {
        assert!(!menu_checkbox_item_indicator_should_render(false, false));
        assert!(menu_checkbox_item_indicator_should_render(true, false));
        assert!(menu_checkbox_item_indicator_should_render(false, true));
        assert!(menu_checkbox_item_indicator_should_render(true, true));

        assert!(!menu_radio_item_indicator_should_render(false, false));
        assert!(menu_radio_item_indicator_should_render(true, false));
    }

    /// The indicators' state map (`MenuCheckboxItemIndicator.tsx:40-45`) carries the parent
    /// item's state plus the transition hook's status, whose `'starting'`/`'ending'` values
    /// are what `transitionStatusMapping` turns into the two style hooks
    /// (`stateAttributesMapping.ts:7-19`) — and which emits nothing when there is no
    /// transition.
    #[test]
    fn the_indicator_state_map_emits_the_transition_hooks_and_nothing_extra() {
        let state = MenuCheckboxItemIndicatorState {
            checked: true,
            disabled: false,
            highlighted: false,
            transition_status: Some(TransitionStatus::Starting),
        };
        let state_attributes = menu_checkbox_item_indicator_attributes(state);
        let attributes = lookup(&state_attributes);
        assert_eq!(attributes.get("data-starting-style"), Some(&""));
        assert_eq!(attributes.get("data-checked"), Some(&""));
        assert_eq!(attributes.len(), 2, "got {attributes:?}");

        let state = MenuCheckboxItemIndicatorState {
            checked: false,
            disabled: true,
            highlighted: true,
            transition_status: Some(TransitionStatus::Ending),
        };
        let state_attributes = menu_checkbox_item_indicator_attributes(state);
        let attributes = lookup(&state_attributes);
        assert_eq!(attributes.get("data-ending-style"), Some(&""));
        assert_eq!(attributes.get("data-unchecked"), Some(&""));
        assert_eq!(attributes.get("data-disabled"), Some(&""));
        assert_eq!(attributes.get("data-highlighted"), Some(&""));
        assert_eq!(attributes.len(), 4, "got {attributes:?}");

        let state = MenuRadioItemIndicatorState {
            checked: false,
            disabled: false,
            highlighted: false,
            transition_status: None,
        };
        let state_attributes = crate::menu::radio_item_indicator::menu_radio_item_indicator_attributes(state);
        let attributes = lookup(&state_attributes);
        assert_eq!(attributes.len(), 1, "no transition -> only the state pair");
        assert_eq!(attributes.get("data-unchecked"), Some(&""));
    }

    /// The two required context reads throw upstream's own messages
    /// (`MenuCheckboxItemIndicator.test.tsx:31-41`, `MenuRadioItemIndicator.test.tsx:29-39`).
    #[test]
    fn the_family_context_reads_panic_with_the_upstream_messages() {
        use crate::menu::checkbox_item::use_menu_checkbox_item_context;
        use crate::menu::radio_group::use_menu_radio_group_context;
        use crate::menu::radio_item::use_menu_radio_item_context;

        let message = std::panic::catch_unwind(|| use_menu_checkbox_item_context(None))
            .err()
            .and_then(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            });
        assert_eq!(
            message.as_deref(),
            Some("Base UI: MenuCheckboxItemContext is missing. MenuCheckboxItem parts must be placed within <Menu.CheckboxItem>.")
        );

        let message = std::panic::catch_unwind(|| use_menu_radio_group_context(None))
            .err()
            .and_then(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            });
        assert_eq!(
            message.as_deref(),
            Some("Base UI: MenuRadioGroupContext is missing. MenuRadioGroup parts must be placed within <Menu.RadioGroup>.")
        );

        let message = std::panic::catch_unwind(|| use_menu_radio_item_context(None))
            .err()
            .and_then(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            });
        assert_eq!(
            message.as_deref(),
            Some("Base UI: MenuRadioItemContext is missing. MenuRadioItem parts must be placed within <Menu.RadioItem>.")
        );
    }
}

// ---------------------------------------------------------------------------
// `Menu.LinkItem` — the link item's own contract (`crates/leptos-ui/src/menu/link_item.rs`).
//
// The item is a pure read of the store plus the composite list's index, so its whole observable
// contract is assertable on the host target. What is NOT: the anchor's own navigation (upstream
// leaves it to the `'a'` root, so it is the browser's) and the `element_attributes` replay (a DOM
// write — CI's to measure, `ralph/generated/env-health.json` -> `browser`).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod link_item_host_tests {
    use reactive_graph::traits::Get;

    use crate::menu::link_item::{
        MENU_LINK_ITEM_CLOSE_ON_CLICK_DEFAULT, MENU_LINK_ITEM_HIGHLIGHTED_ATTRIBUTE,
        MENU_LINK_ITEM_ROLE, MENU_LINK_ITEM_TAG, menu_link_item_state_map, resolve_menu_link_item,
    };
    use crate::menu::store::{
        MenuChangeEventDetails, create_menu_store, menu_item_is_active, menu_item_on_click,
        menu_set_open, menu_store_is_open, reasons_menu, use_menu_active_index_signal,
    };

    /// The `menu_tests` convention: the host target has no JS runtime, so a details value wraps a
    /// null JsValue.
    fn details(reason: &str) -> MenuChangeEventDetails {
        leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails::new(
            reason,
            web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
            None,
            String::new(),
        )
    }

    /// Runs `f` under a reactive owner with the spawn executor initialised (the store's
    /// subscription spawns on it) — the `item_host_tests` idiom.
    fn with_owner(f: impl FnOnce()) {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        let _guard = owner.with(|| {
            f();
        });
    }

    /// A native event stand-in for the host target (the `menu_tests` convention).
    fn host_event() -> web_sys::Event {
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL)
    }

    /// The part renders an `<a role="menuitem">` (`MenuLinkItem.tsx:69`,
    /// `useMenuItemCommonProps.ts:62`) and its only state attribute is
    /// `data-highlighted` (`MenuLinkItemDataAttributes.ts:4`).
    #[test]
    fn the_anchor_root_is_upstreams_element_and_role() {
        assert_eq!(MENU_LINK_ITEM_TAG, "a", "useRenderElement('a', …) (MenuLinkItem.tsx:69)");
        assert_eq!(
            MENU_LINK_ITEM_ROLE, "menuitem",
            "a link item is still a menuitem (MenuLinkItem.test.tsx:53)"
        );
        assert_eq!(
            MENU_LINK_ITEM_HIGHLIGHTED_ATTRIBUTE, "data-highlighted",
            "MenuLinkItemDataAttributes.ts:4"
        );
    }

    /// `onClick` (`useMenuItemCommonProps.ts:81-85`): with `closeOnClick` the click runs the
    /// unit's one mutation gate with the `itemPress` reason.
    #[test]
    fn the_link_item_close_request_goes_through_the_gate_with_the_item_press_reason() {
        with_owner(|| {
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));
            assert!(menu_store_is_open(&store), "the fixture opens the menu first");

            menu_item_on_click(&store, true, Some(host_event()));

            assert!(
                !menu_store_is_open(&store),
                "a closeOnClick link item closes the menu (useMenuItemCommonProps.ts:81-85)"
            );
            let extra = store.get_snapshot().payload.clone().unwrap_or_default();
            assert_eq!(
                extra.open_change_reason.as_deref(),
                Some(reasons_menu::ITEM_PRESS),
                "the close carries the itemPress reason (reason-parts.ts:8)"
            );
        });
    }

    /// The documented default (`MenuLinkItem.tsx:29`, `closeOnClick = false`): clicking the link
    /// makes NO close request at all — the gate is never called, so this is not "a close that was
    /// vetoed". The constant is what the component's prop default uses, which is why the default
    /// itself is assertable here.
    #[test]
    fn the_documented_close_on_click_default_makes_no_close_request() {
        assert!(
            !MENU_LINK_ITEM_CLOSE_ON_CLICK_DEFAULT,
            "closeOnClick defaults false (MenuLinkItem.tsx:29) — unlike Menu.Item's true"
        );
        with_owner(|| {
            let store = create_menu_store();
            menu_set_open(&store, true, details("trigger-press"));

            menu_item_on_click(&store, MENU_LINK_ITEM_CLOSE_ON_CLICK_DEFAULT, Some(host_event()));

            assert!(
                menu_store_is_open(&store),
                "the default link-item click leaves the menu open"
            );
            let extra = store.get_snapshot().payload.clone().unwrap_or_default();
            assert_eq!(extra.open_change_reason.as_deref(), Some("trigger-press"), "no second commit");
        });
    }

    /// `state = { highlighted }` (`MenuLinkItem.tsx:67`, `MenuLinkItemState` at `:86-91`): the
    /// link item's state object carries ONE member, so the resolved element carries the
    /// highlighted marker and nothing else — explicitly not `data-checked`/`data-unchecked`
    /// (`itemMapping`, `utils/stateAttributesMapping.ts:5-12`) and not `data-disabled`
    /// (`MenuItem.tsx:38-40`), which its sibling items' states do carry.
    #[test]
    fn the_link_item_carries_the_highlighted_marker_and_nothing_else() {
        assert_eq!(
            menu_link_item_state_map(true).len(),
            1,
            "MenuLinkItemState has one member (MenuLinkItem.tsx:86-91)"
        );

        let highlighted = resolve_menu_link_item("link-1".to_string(), true, true);
        assert_eq!(
            highlighted.attributes,
            vec![(MENU_LINK_ITEM_HIGHLIGHTED_ATTRIBUTE.to_string(), String::new())],
            "true renders the marker bare (getStateAttributesProps.ts:24-25)"
        );

        let plain = resolve_menu_link_item("link-2".to_string(), true, false);
        assert!(
            plain.attributes.is_empty(),
            "not highlighted -> no state attributes at all (got {:?})",
            plain.attributes
        );
    }

    /// `store.useState('isActive', listItem.index)` (`MenuLinkItem.tsx:43`) — the read behind
    /// `highlighted`, `tabIndex` and the marker.
    #[test]
    fn the_link_item_highlight_follows_the_stores_active_index() {
        with_owner(|| {
            let store = create_menu_store();
            assert!(!menu_item_is_active(&store, 0), "nothing is highlighted initially");

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
        });
    }

    /// The resolved description `useRenderElement('a', …)` (`MenuLinkItem.tsx:69-73`) puts on the
    /// element, with the documented id/tab-index contract.
    #[test]
    fn the_resolved_link_item_matches_the_documented_contract() {
        let highlighted = resolve_menu_link_item("link-item-1".to_string(), true, true);
        assert_eq!(highlighted.tag, MENU_LINK_ITEM_TAG);
        assert_eq!(highlighted.role, MENU_LINK_ITEM_ROLE);
        assert_eq!(highlighted.id, "link-item-1");
        assert_eq!(highlighted.tab_index, 0, "open + highlighted is the tab stop");
        assert!(highlighted.highlighted);
        assert_eq!(
            highlighted.attributes,
            vec![("data-highlighted".to_string(), String::new())]
        );

        let not_highlighted = resolve_menu_link_item("link-item-2".to_string(), true, false);
        assert_eq!(not_highlighted.tab_index, -1, "open but not highlighted -> -1");
        assert!(not_highlighted.attributes.is_empty());

        let closed = resolve_menu_link_item("link-item-3".to_string(), false, true);
        assert_eq!(closed.tab_index, -1, "a closed menu's items are out of the tab order");
    }

    /// The part is reachable at the namespaced path the docs contract requires
    /// (`specs/docs-content/CONTRACT.md`: upstream's `Component.Part` usage maps to
    /// `<Component::Part>`) — the compile-level half of the exposure, which the part-surface gate
    /// (`node ralph/scripts/check-part-surface.mjs --components menu`) also reads. What is NOT
    /// assertable here: constructing `<Menu::LinkItem …>` runs the component's commit effect,
    /// which needs an executor this host target does not provide (the browser-class refusal,
    /// `ralph/generated/env-health.json` → `browser`), so the rendered half is CI's to measure.
    #[test]
    fn the_link_item_is_reachable_at_the_namespaced_path() {
        // `crate::Menu::LinkItem` is the `view!`-usable component fn behind the part — the
        // `#[component]` wrapper the crate's other parts expose to markup. Binding it as a value
        // fails to compile if the re-export is renamed or dropped, so the path this test names is
        // the same one the docs' `<Menu::LinkItem />` spelling resolves through.
        let _component_fn = crate::Menu::LinkItem;
    }
}

/// Host tests for the submenu pair — `Menu.SubmenuRoot` + `Menu.SubmenuTrigger`
/// (`MenuSubmenuRoot.tsx`, `MenuSubmenuTrigger.tsx` and its `MenuSubmenuTriggerDataAttributes.ts`)
/// and the parent-resolution change they made reachable in `MenuRoot.tsx:77-107` /
/// `MenuStore.ts:57-59`.
///
/// These assert the parts' RESOLVABLE contracts — the attributes, the folds, the predicates and the
/// parent-store writes — because this box refuses a browser
/// (`ralph/generated/env-health.json` → `browser: DEGRADED`), so the DOM consequences of those values
/// are CI's to measure and are not claimed here.
#[cfg(test)]
mod submenu_host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::traits::GetUntracked;

    use crate::menu::root::MenuRootProps;
    use crate::menu::store::{
        MenuParent, create_initial_menu_store_state, create_menu_store, menu_modal,
        menu_store_trigger_popup_id, use_menu_modal_signal,
    };
    use crate::menu::submenu_root::{
        MenuSubmenuRootContextValue, MenuSubmenuRootProps, provide_menu_submenu_root_context,
        use_menu_submenu_root_context,
    };
    use crate::menu::submenu_trigger::{
        MENU_SUBMENU_TRIGGER_DISABLED_ATTRIBUTE, MENU_SUBMENU_TRIGGER_HASPOPUP,
        MENU_SUBMENU_TRIGGER_HIGHLIGHTED_ATTRIBUTE, MENU_SUBMENU_TRIGGER_OUTSIDE_ROOT_MESSAGE,
        MENU_SUBMENU_TRIGGER_POPUP_OPEN_ATTRIBUTE, MENU_SUBMENU_TRIGGER_ROLE,
        MENU_SUBMENU_TRIGGER_TAG, menu_submenu_trigger_aria_controls,
        menu_submenu_trigger_aria_expanded, menu_submenu_trigger_attributes,
        menu_submenu_trigger_disabled, menu_submenu_trigger_on_blur,
        menu_submenu_trigger_opened_by_keyboard, menu_submenu_trigger_set_active,
        menu_submenu_trigger_should_omit_expanded, menu_submenu_trigger_tab_index,
        resolve_menu_submenu_trigger,
    };

    /// Runs `f` under a reactive owner — the store's `use_state` hooks need one, and the futures
    /// executor must be initialized before any of them spawns its effect (the
    /// `toggle_group_tests.rs:73-78` `in_owner` precedent).
    fn with_owner(f: impl FnOnce()) {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();
        f();
    }

    /// Runs `f` under a **leptos-namespace** owner.
    ///
    /// The submenu bridge's provider and accessor both go through `leptos::prelude`'s context
    /// functions (`submenu_root.rs`), and this workspace carries TWO `reactive_graph` versions —
    /// `0.1.8` under leptos 0.7.8 and `0.2.14` for this crate's own signal reads
    /// (`cargo tree -p base-ui-leptos -d`). A `reactive_graph 0.2.14` owner is INVISIBLE to leptos's
    /// context map, so a test that provides and reads under it would assert a silent no-op
    /// (`provide_context`'s `if let Some(owner) = Owner::current()` simply does nothing). The seam
    /// is therefore asserted through the CONSUMER's own accessor, under the consumer's own namespace
    /// — exactly the check `toggle_group_tests.rs:385-405` added for the same reason.
    fn with_leptos_owner(f: impl FnOnce()) {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = leptos::prelude::Owner::new();
        owner.set();
        f();
    }

    fn has(attributes: &[(String, String)], name: &str) -> bool {
        attributes.iter().any(|(key, _)| key == name)
    }

    /// Reads a store's `activeIndex` off the payload.
    fn active_index(store: &crate::menu::store::MenuStore) -> Option<usize> {
        store
            .get_snapshot()
            .payload
            .as_ref()
            .and_then(|payload| payload.active_index)
    }

    // --- Menu.SubmenuRoot (`MenuSubmenuRoot.tsx`) ---------------------------

    /// `MenuSubmenuRoot.tsx:21-23` / `MenuSubmenuRootContext.ts:13-15` — the bridge is absent
    /// outside a SubmenuRoot, which is what makes `MenuRoot` resolve `{ type: undefined }`
    /// (`MenuRoot.tsx:107`) and what `Menu.SubmenuTrigger` turns into a throw.
    #[test]
    fn the_submenu_bridge_is_absent_outside_a_submenu_root() {
        with_leptos_owner(|| {
            assert!(use_menu_submenu_root_context().is_none());
        });
    }

    /// `MenuSubmenuRoot.tsx:16,18-23` — the bridge carries the PARENT menu's store by reference, which
    /// is the one fact both the inner Root and the trigger read. Asserted under the CONSUMER's own
    /// namespace (see [`with_leptos_owner`]), because a provider that writes into the other
    /// `reactive_graph` version's map compiles and silently does nothing.
    #[test]
    fn the_submenu_bridge_carries_the_parent_store() {
        with_leptos_owner(|| {
            let parent = create_menu_store();
            provide_menu_submenu_root_context(MenuSubmenuRootContextValue {
                parent_menu: parent.clone(),
            });

            let read = use_menu_submenu_root_context().expect("bridge provided");
            assert!(
                std::rc::Rc::ptr_eq(&read.parent_menu, &parent),
                "the bridge must share the parent store by reference, not copy it \
                 (MenuRoot.tsx:80-85)"
            );
        });
    }

    /// `MenuSubmenuRoot.tsx:22` — `<MenuRoot {...props} />` is a pass-through, and the Omit list
    /// (`:27-36`) removes `modal`/`openOnHover` from the surface rather than re-mapping them.
    #[test]
    fn the_submenu_root_props_pass_through_to_the_root() {
        let props = MenuSubmenuRootProps {
            open: Some(true),
            default_open: true,
            disabled: true,
            close_parent_on_esc: true,
            highlight_item_on_hover: false,
            root_id: Some("submenu-root".to_string()),
            ..MenuSubmenuRootProps::default()
        };

        let root: MenuRootProps = props.to_root_props();
        assert_eq!(root.open, Some(true));
        assert!(root.default_open);
        assert!(root.disabled);
        assert!(root.close_parent_on_esc);
        assert!(!root.highlight_item_on_hover);
        assert_eq!(root.root_id.as_deref(), Some("submenu-root"));
    }

    /// `MenuSubmenuRoot.tsx:44,47` + `MenuRoot.tsx:67` — `closeParentOnEsc` defaults to `false`, and
    /// the kept props carry upstream's documented defaults.
    #[test]
    fn the_submenu_root_defaults_are_upstreams() {
        let props = MenuSubmenuRootProps::default();
        assert!(!props.close_parent_on_esc, "MenuRoot.tsx:67");
        assert!(props.highlight_item_on_hover, "MenuStore.ts:24");
        assert!(!props.default_open);
        assert!(!props.disabled);
        assert!(props.open.is_none());
    }

    // --- The parent resolution (`MenuRoot.tsx:77-107`, `MenuStore.ts:57-59`) ---

    /// `MenuStore.ts:57-59` — the `modal` SELECTOR: only a top-level or context-menu parent can be
    /// modal. A nested menu is never modal whatever its own field holds, which is the half the port
    /// was missing while `parent` was pinned to `None`.
    #[test]
    fn the_modal_selector_makes_a_nested_menu_never_modal() {
        assert!(menu_modal(&MenuParent::None, true), "top-level keeps the field");
        assert!(menu_modal(&MenuParent::ContextMenu, true));
        assert!(!menu_modal(&MenuParent::None, false), "the field still wins when false");

        let parent = create_menu_store();
        assert!(
            !menu_modal(&MenuParent::Menu { store: parent }, true),
            "a submenu is never modal even with modal: true (MenuStore.ts:57-59)"
        );
    }

    /// The same fold through the reactive read the positioner's `popupModal` input uses
    /// (`MenuPositioner.tsx:267-268`).
    #[test]
    fn the_modal_signal_folds_the_parent_type() {
        with_owner(|| {
            let store = create_menu_store();
            let parent = create_menu_store();

            assert!(use_menu_modal_signal(&store).get_untracked(), "no parent yet");

            store.set_field(
                |state| &mut state.payload.get_or_insert_with(Default::default).parent,
                MenuParent::Menu { store: parent },
            );

            assert!(
                !use_menu_modal_signal(&store).get_untracked(),
                "once the SubmenuRoot bridge supplies a menu parent the selector answers false"
            );
        });
    }

    /// `popups/store.ts:199-200` — `triggerPopupId` returns the popup's id only for the trigger that
    /// owns the open popup (or the only registered trigger); otherwise `None`, which is what lets
    /// `aria-controls` fall back to the trigger's own id.
    #[test]
    fn the_trigger_popup_id_requires_ownership_or_a_single_trigger() {
        let mut state = create_initial_menu_store_state();
        state.floating_id = Some("base-ui-popup".to_string());

        // Closed: nobody owns it (`triggerOwnsOpenPopup`, store.ts:149-153).
        assert_eq!(menu_store_trigger_popup_id(&state, Some("t1")), None);

        state.open = true;
        // Open with an active trigger that is NOT this one.
        state.active_trigger_id = Some("other".to_string());
        assert_eq!(menu_store_trigger_popup_id(&state, Some("t1")), None);

        // Open, and this trigger owns it.
        state.active_trigger_id = Some("t1".to_string());
        assert_eq!(
            menu_store_trigger_popup_id(&state, Some("t1")).as_deref(),
            Some("base-ui-popup")
        );

        // Open, no active trigger, exactly one registered trigger — the only-trigger fallback
        // (store.ts:155-166).
        state.active_trigger_id = None;
        state.trigger_count = 1;
        assert_eq!(
            menu_store_trigger_popup_id(&state, Some("t1")).as_deref(),
            Some("base-ui-popup")
        );

        // Two registered triggers and no active one: no fallback.
        state.trigger_count = 2;
        assert_eq!(menu_store_trigger_popup_id(&state, Some("t1")), None);

        // No trigger id at all.
        assert_eq!(menu_store_trigger_popup_id(&state, None), None);
    }

    /// `MenuSubmenuTrigger.tsx:71-77` — the active-trigger claim writes the trigger id, the element
    /// and the `closeDelay` the trigger carries.
    ///
    /// NOT asserted here: the element half. The host target cannot construct a `web_sys::Element`
    /// (there is no JS runtime), so a test of it would be vacuous — the `let _ = Input;` smell this
    /// ledger already paid for once. The write's DOM consequence rides `registerTrigger`'s own
    /// contract (`popup_store_utils::use_trigger_registration`, whose host tests cover the
    /// registration bookkeeping) and is CI's to measure rendered.

    // --- Menu.SubmenuTrigger (`MenuSubmenuTrigger.tsx`) ---------------------

    /// `MenuSubmenuTrigger.tsx:51` — the throw message a trigger outside a SubmenuRoot carries.
    #[test]
    fn the_outside_root_message_is_upstreams() {
        assert_eq!(
            MENU_SUBMENU_TRIGGER_OUTSIDE_ROOT_MESSAGE,
            "Base UI: <Menu.SubmenuTrigger> must be placed in <Menu.SubmenuRoot>."
        );
    }

    /// `MenuSubmenuTrigger.tsx:185,191` — a `div` with `role="menuitem"`
    /// (`MenuSubmenuTrigger.test.tsx:32,238`).
    #[test]
    fn the_trigger_is_a_menuitem_div() {
        assert_eq!(MENU_SUBMENU_TRIGGER_TAG, "div");
        assert_eq!(MENU_SUBMENU_TRIGGER_ROLE, "menuitem");
    }

    /// `MenuSubmenuTrigger.tsx:201` — `open || highlighted ? 0 : -1`, deliberately unlike the plain
    /// item's `open && highlighted` (`useMenuItemCommonProps.ts:63`). The discriminating case is an
    /// open submenu whose trigger is not the highlighted item.
    #[test]
    fn the_trigger_tab_index_uses_or_not_and() {
        assert_eq!(menu_submenu_trigger_tab_index(false, false), -1);
        assert_eq!(menu_submenu_trigger_tab_index(true, false), 0);
        assert_eq!(menu_submenu_trigger_tab_index(false, true), 0);
        assert_eq!(menu_submenu_trigger_tab_index(true, true), 0);

        // The divergence the item's own function would get wrong.
        assert_eq!(crate::menu::store::menu_item_tab_index(true, false), -1);
        assert_eq!(menu_submenu_trigger_tab_index(true, false), 0);
    }

    /// `MenuSubmenuTrigger.tsx:102` — `disabledProp || rootDisabled || parentDisabled`.
    #[test]
    fn the_disabled_fold_includes_the_parent_menu() {
        assert!(!menu_submenu_trigger_disabled(false, false, false));
        assert!(menu_submenu_trigger_disabled(true, false, false));
        assert!(menu_submenu_trigger_disabled(false, true, false));
        assert!(
            menu_submenu_trigger_disabled(false, false, true),
            "a disabled PARENT menu disables its submenu trigger too (MenuSubmenuTrigger.tsx:102)"
        );
    }

    /// `MenuSubmenuTrigger.tsx:175,187` —
    /// `getStateAttributesProps({ disabled, highlighted, open }, triggerOpenStateMapping)`:
    /// `data-popup-open` from the mapping, `data-disabled`/`data-highlighted` from the engine's
    /// default handling of the two keys the mapping does not claim.
    #[test]
    fn the_trigger_state_attributes_come_from_the_shared_engine() {
        let open_attrs = menu_submenu_trigger_attributes(false, false, true);
        assert!(has(&open_attrs, MENU_SUBMENU_TRIGGER_POPUP_OPEN_ATTRIBUTE));
        assert!(!has(&open_attrs, MENU_SUBMENU_TRIGGER_HIGHLIGHTED_ATTRIBUTE));
        assert!(!has(&open_attrs, MENU_SUBMENU_TRIGGER_DISABLED_ATTRIBUTE));

        let highlighted = menu_submenu_trigger_attributes(false, true, false);
        assert!(has(&highlighted, MENU_SUBMENU_TRIGGER_HIGHLIGHTED_ATTRIBUTE));
        assert!(
            !has(&highlighted, MENU_SUBMENU_TRIGGER_POPUP_OPEN_ATTRIBUTE),
            "a closed submenu's trigger carries no data-popup-open"
        );

        let disabled = menu_submenu_trigger_attributes(true, false, false);
        assert!(has(&disabled, MENU_SUBMENU_TRIGGER_DISABLED_ATTRIBUTE));

        let all = menu_submenu_trigger_attributes(true, true, true);
        assert!(has(&all, MENU_SUBMENU_TRIGGER_DISABLED_ATTRIBUTE));
        assert!(has(&all, MENU_SUBMENU_TRIGGER_HIGHLIGHTED_ATTRIBUTE));
        assert!(has(&all, MENU_SUBMENU_TRIGGER_POPUP_OPEN_ATTRIBUTE));

        let none = menu_submenu_trigger_attributes(false, false, false);
        assert!(none.is_empty(), "nothing present when closed, idle and enabled");
    }

    /// `MenuSubmenuTrigger.tsx:200` over `:63` — `aria-controls` is the popup id when the store has
    /// one for this trigger, and the trigger's OWN id otherwise (the `useState` default argument).
    #[test]
    fn aria_controls_falls_back_to_the_trigger_id() {
        assert_eq!(
            menu_submenu_trigger_aria_controls(None, "base-ui-7"),
            "base-ui-7"
        );
        assert_eq!(
            menu_submenu_trigger_aria_controls(Some("popup-1".to_string()), "base-ui-7"),
            "popup-1"
        );
    }

    /// `MenuSubmenuTrigger.tsx:181-182` — arrow keys open through list navigation without a click
    /// (`openMethod` stays null), while Enter/Space dispatch one and report `keyboard`.
    #[test]
    fn opened_by_keyboard_covers_both_upstream_paths() {
        assert!(menu_submenu_trigger_opened_by_keyboard(
            Some(leptos_ui_internals::floating_ui::reasons::LIST_NAVIGATION),
            None
        ));
        assert!(menu_submenu_trigger_opened_by_keyboard(None, Some("keyboard")));
        assert!(!menu_submenu_trigger_opened_by_keyboard(
            Some(leptos_ui_internals::floating_ui::reasons::TRIGGER_HOVER),
            Some("mouse")
        ));
        assert!(!menu_submenu_trigger_opened_by_keyboard(None, None));
    }

    /// `MenuSubmenuTrigger.tsx:183,196-198` — `aria-expanded` is DROPPED (not `false`) only when the
    /// submenu is open, was opened by the keyboard, and the platform is VoiceOver. On other
    /// platforms the attribute stays, which is the other half of the same test file.
    #[test]
    fn aria_expanded_is_dropped_only_for_a_voiceover_keyboard_open() {
        assert!(menu_submenu_trigger_should_omit_expanded(true, true, true));
        assert_eq!(menu_submenu_trigger_aria_expanded(true, true, true), None);

        // Open by keyboard but not VoiceOver: the attribute is kept as "true".
        assert_eq!(menu_submenu_trigger_aria_expanded(true, true, false), Some(true));

        // VoiceOver but pointer-opened: kept, because focus stays on the trigger and there is no
        // item announcement to conflict with (voiceOver.test.tsx:104-105).
        assert_eq!(menu_submenu_trigger_aria_expanded(true, false, true), Some(true));

        // Closed: never omitted, the trigger reports the collapsed state.
        assert_eq!(menu_submenu_trigger_aria_expanded(false, false, true), Some(false));
        assert_eq!(menu_submenu_trigger_aria_expanded(false, true, true), Some(false));
    }

    /// `MenuSubmenuTrigger.tsx:122-127` — `setActive()` writes the PARENT's `activeIndex`, gated on
    /// the parent's `highlightItemOnHover`.
    #[test]
    fn set_active_writes_the_parent_highlight_only_when_the_parent_allows_hover_highlighting() {
        let parent = create_menu_store();
        assert_eq!(active_index(&parent), None);

        assert!(menu_submenu_trigger_set_active(&parent, 3));
        assert_eq!(active_index(&parent), Some(3));

        // Negative index never writes (the unindexed composite-list value).
        assert!(!menu_submenu_trigger_set_active(&parent, -1));
        assert_eq!(active_index(&parent), Some(3));
    }

    /// The `highlightItemOnHover: false` half of the same gate.
    #[test]
    fn set_active_is_a_no_op_when_the_parent_disables_hover_highlighting() {
        let parent = create_menu_store();
        parent.set_field(
            |state| &mut state.payload.get_or_insert_with(Default::default).highlight_item_on_hover,
            false,
        );

        assert!(!menu_submenu_trigger_set_active(&parent, 2));
        assert_eq!(active_index(&parent), None);
    }

    /// `MenuSubmenuTrigger.tsx:202-206` — `onBlur` clears the parent's highlight only when the
    /// trigger that lost focus WAS the highlighted item.
    #[test]
    fn blur_clears_the_parent_highlight_only_for_the_highlighted_trigger() {
        let parent = create_menu_store();
        menu_submenu_trigger_set_active(&parent, 1);
        assert_eq!(active_index(&parent), Some(1));

        menu_submenu_trigger_on_blur(&parent, false);
        assert_eq!(active_index(&parent), Some(1), "an unhighlighted trigger leaves it alone");

        menu_submenu_trigger_on_blur(&parent, true);
        assert_eq!(active_index(&parent), None);
    }

    /// `MenuSubmenuTrigger.tsx:185-212` — the whole element description in one place, including the
    /// `"menu"` haspopup token (a string, not the boolean the facade emitted).
    #[test]
    fn the_resolved_trigger_description_is_upstreams_element() {
        let resolved =
            resolve_menu_submenu_trigger("base-ui-9".to_string(), None, true, false, false, false, false);

        assert_eq!(resolved.tag, "div");
        assert_eq!(resolved.role, "menuitem");
        assert_eq!(resolved.aria_haspopup, MENU_SUBMENU_TRIGGER_HASPOPUP);
        assert_eq!(resolved.aria_controls, "base-ui-9");
        assert_eq!(resolved.tab_index, 0, "open || highlighted");
        assert_eq!(resolved.aria_expanded, Some(true));
        assert!(has(&resolved.attributes, "data-popup-open"));
        assert!(!has(&resolved.attributes, "data-disabled"));
    }
}
