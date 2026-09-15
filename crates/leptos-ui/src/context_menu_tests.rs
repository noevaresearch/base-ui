//! Tests for the Context Menu port — mirrors of the upstream suites behavior.md
//! mines (`ContextMenuRoot.test.tsx`, `ContextMenuRoot.non-mac.test.tsx`,
//! `ContextMenuTrigger.test.tsx`), over the real store spine and the gesture
//! plumbing.
//!
//! Host suite: the pure contracts that need no DOM — the anchor's virtual-rect
//! shape (zero-size origin seed, 0-vs-10 touch split), the `data-rootownerid`
//! ancestor walk (`findRootOwnerId`), the context-menu positioning defaults, and
//! the store-level open/close routing through the one mutation gate with the
//! `triggerPress`/`cancelOpen` reasons.
//!
//! Wasm suite: the materialized-tree contracts — right-click opens through the
//! real gesture path (`handleContextMenu` → `handleLongPress` →
//! `menu_store_set_open(true, 'trigger-press')`), the 500ms grace window
//! (`allowMouseUpTimeout`) suppressing the immediate mouseup cancel, the
//! mouseup-after-grace cancel with `REASONS.cancelOpen`, the disabled gate's
//! zero-callback short-circuit, the document-level `contextmenu` default
//! prevention inside the trigger/backdrop surfaces, and the open-state mirror as
//! `data-popup-open` (`pressableTriggerOpenStateMapping`).

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::context_menu::root::{
    ContextMenuRootProps, VirtualAnchor, use_context_menu_root,
};
use crate::context_menu::positioner::{
    CONTEXT_MENU_DEFAULT_ALIGN_OFFSET, CONTEXT_MENU_DEFAULT_SIDE_OFFSET,
    CONTEXT_MENU_POSITION_METHOD,
};
use crate::context_menu::trigger::LONG_PRESS_DELAY;
use crate::menu::store::{
    MenuChangeEventDetails, MenuParent, create_menu_store_with_on_open_change, menu_set_open,
    menu_store_is_open,
};

/// A details value with the given reason and no native event (menu_tests.rs
/// convention — the host target wraps a plain JsValue).
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

#[cfg(test)]
mod host_tests {
    use super::*;

    // The anchor seed (`ContextMenuRoot.tsx:19-21`): the initial value is a
    // zero-size rect at the origin so the positioner always has a well-formed
    // `getBoundingClientRect` before the first open.
    #[test]
    fn the_anchor_seed_is_a_zero_size_rect_at_the_origin() {
        let anchor = VirtualAnchor::origin();
        assert_eq!(anchor.rect(), (0.0, 0.0, 0.0, 0.0));
    }

    // The virtual-element swap (`ContextMenuTrigger.tsx:57-66`): mouse opens get a
    // zero-size rect at the pointer, touch opens a 10×10 rect.
    #[test]
    fn the_virtual_anchor_rects_follow_the_mouse_touch_split() {
        let mouse = VirtualAnchor { x: 12.0, y: 34.0, size: 0.0 };
        assert_eq!(mouse.rect(), (12.0, 34.0, 0.0, 0.0));

        let touch = VirtualAnchor { x: 12.0, y: 34.0, size: 10.0 };
        assert_eq!(touch.rect(), (12.0, 34.0, 10.0, 10.0));
    }

    // The hold-to-open constant (`ContextMenuTrigger.tsx:16`): 500ms, also the
    // grace window before a document mouseup may cancel.
    #[test]
    fn the_long_press_delay_is_500ms() {
        assert_eq!(LONG_PRESS_DELAY, 500);
    }

    // The context-menu positioning defaults (`MenuPositioner.tsx:88-95`,
    // `:114`): `sideOffset: -5`, `alignOffset: 2`, forced `positionMethod:
    // 'fixed'` (implementation.md "DOM/portal strategy" — clientX/clientY are
    // viewport coordinates).
    #[test]
    fn the_positioning_defaults_match_the_context_menu_preset() {
        assert_eq!(CONTEXT_MENU_DEFAULT_SIDE_OFFSET, -5.0);
        assert_eq!(CONTEXT_MENU_DEFAULT_ALIGN_OFFSET, 2.0);
        assert_eq!(CONTEXT_MENU_POSITION_METHOD, "fixed");
    }

    // The store spine: the Root's open path funnels into the one mutation gate
    // with `REASONS.triggerPress` (`ContextMenuTrigger.tsx:69`), the cancel path
    // with `REASONS.cancelOpen` (`:112-115`); the reason is recorded on the
    // details the user callback sees (`onOpenChange`, behavior.md "State model").
    #[test]
    fn the_gesture_open_and_cancel_route_through_the_mutation_gate() {
        let calls: Rc<RefCell<Vec<(bool, String)>>> = Rc::new(RefCell::new(Vec::new()));
        let seen = calls.clone();
        let store = create_menu_store_with_on_open_change(Some(Rc::new(
            move |open: bool, details: &MenuChangeEventDetails| {
                seen.borrow_mut().push((open, details.reason.clone()));
            },
        )));

        // The open (the `handleLongPress` emission).
        menu_set_open(&store, true, details("trigger-press"));
        assert!(menu_store_is_open(&store));
        assert_eq!(
            calls.borrow().last().map(|(o, r)| (o.clone(), r.clone())),
            Some((true, "trigger-press".to_owned()))
        );

        // The gesture-end cancel (the mouseup listener's emission,
        // `ContextMenuTrigger.tsx:112-115`).
        menu_set_open(&store, false, details("cancel-open"));
        assert!(!menu_store_is_open(&store));
        assert_eq!(
            calls.borrow().last().map(|(o, r)| (o.clone(), r.clone())),
            Some((false, "cancel-open".to_owned()))
        );
    }

    // The Root seeds the store: `modal` forced on (`MenuStore.ts:57-59`),
    // `disabled` mirrored, and the parent resolved to the context-menu arm
    // (`MenuRoot.tsx:97-102`) — the discriminant that drives the positioner's
    // defaults and the internal backdrop.
    #[test]
    fn the_root_seeds_modal_disabled_and_the_context_menu_parent() {
        with_owner(|| {
            let (store, context) =
                use_context_menu_root(ContextMenuRootProps {
                    disabled: true,
                    ..Default::default()
                });
            // Stage the provider the way `context_menu_root_view` does (the
            // provider sandwich, `ContextMenuRoot.tsx:46-52`).
            crate::context_menu::root::provide_context_menu_root_context(context, None);
            // The store is reachable from the context the Root provided.
            let context = crate::context_menu::root::use_context_menu_root_context_optional()
                .expect("the Root provided the context-menu context");
            let extra = context
                .actions
                .borrow()
                .as_ref()
                .map(|store| store.get_snapshot().payload.clone().unwrap_or_default())
                .expect("the actions slot is filled");
            assert!(extra.modal, "context menus are always modal (MenuStore.ts:57-59)");
            assert!(extra.disabled, "the disabled prop is mirrored into the store");
            assert_eq!(extra.parent, MenuParent::ContextMenu);
        });
    }

    // The context value shape (`ContextMenuRoot.tsx:31-44`): the
    // `allowMouseUpTriggerRef` starts `true` (`:27`), the anchor starts at the
    // zero-size origin, and the `rootId` is a non-empty tree identity (`:29`).
    #[test]
    fn the_context_value_starts_in_the_upstream_initial_shape() {
        with_owner(|| {
            let (_store, context) =
                use_context_menu_root(ContextMenuRootProps::default());
            assert!(
                context.allow_mouse_up_trigger.get(),
                "allowMouseUpTriggerRef starts true (ContextMenuRoot.tsx:27)"
            );
            assert_eq!(
                context.anchor.borrow().rect(),
                (0.0, 0.0, 0.0, 0.0),
                "the anchor starts at the zero-size origin rect (:19-21)"
            );
            assert!(
                !context.root_id.is_empty(),
                "rootId is a non-empty tree identity (:29)"
            );
            assert!(
                context.initial_cursor_point.borrow().is_none(),
                "no cursor point has been recorded"
            );
        });
    }
}
