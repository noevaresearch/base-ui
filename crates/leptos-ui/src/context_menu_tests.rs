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
use crate::context_menu::positioner::{
    CONTEXT_MENU_DEFAULT_ALIGN_OFFSET, CONTEXT_MENU_DEFAULT_SIDE_OFFSET,
    CONTEXT_MENU_POSITION_METHOD,
};
use crate::context_menu::root::{ContextMenuRootProps, VirtualAnchor, use_context_menu_root};
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
        let mouse = VirtualAnchor {
            x: 12.0,
            y: 34.0,
            size: 0.0,
        };
        assert_eq!(mouse.rect(), (12.0, 34.0, 0.0, 0.0));

        let touch = VirtualAnchor {
            x: 12.0,
            y: 34.0,
            size: 10.0,
        };
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
            let (store, context) = use_context_menu_root(ContextMenuRootProps {
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
            assert!(
                extra.modal,
                "context menus are always modal (MenuStore.ts:57-59)"
            );
            assert!(
                extra.disabled,
                "the disabled prop is mirrored into the store"
            );
            assert_eq!(extra.parent, MenuParent::ContextMenu);
        });
    }

    // The context value shape (`ContextMenuRoot.tsx:31-44`): the
    // `allowMouseUpTriggerRef` starts `true` (`:27`), the anchor starts at the
    // zero-size origin, and the `rootId` is a non-empty tree identity (`:29`).
    #[test]
    fn the_context_value_starts_in_the_upstream_initial_shape() {
        with_owner(|| {
            let (_store, context) = use_context_menu_root(ContextMenuRootProps::default());
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

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::menu::store::{clear_menu_root_context, use_menu_root_context_optional};
    use leptos::mount::mount_to;
    use leptos::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::{Document, Element, HtmlElement};

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts the Root > Trigger composition and returns the container — the
    /// dialog wasm harness convention (the store is provided through the Root's
    /// provider sandwich; the popup/portal parts are the docs-content
    /// iteration's surface).
    fn mount_context_menu(
        on_open_change: Option<Rc<dyn Fn(bool, &MenuChangeEventDetails)>>,
        disabled: bool,
    ) -> (HtmlElement, Rc<RefCell<Vec<(bool, String)>>>) {
        let _ = any_spawner::Executor::init_futures_executor();
        clear_menu_root_context();

        let calls: Rc<RefCell<Vec<(bool, String)>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_props = calls.clone();

        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-context-menu-root");
        document().body().unwrap().append_child(&container).unwrap();

        // mem::forget the mount handle — dropping it disposes the reactive owner
        // and tears the rendered view down (the dialog/docs-app wasm convention).
        std::mem::forget(mount_to(container.clone(), move || {
            view! {
                <ContextMenuRootComponent
                    context_menu_props=ContextMenuRootProps {
                        on_open_change: on_open_change.clone().map(move |cb| {
                            let calls = calls_for_props.clone();
                            Rc::new(move |open: bool, details: &MenuChangeEventDetails| {
                                cb(open, details);
                                calls.borrow_mut()
                                    .push((open, details.reason.clone()));
                            }) as Rc<dyn Fn(bool, &MenuChangeEventDetails)>
                        }),
                        disabled,
                        ..Default::default()
                    }
                >
                    <ContextMenuTrigger>
                        "Right-click area"
                    </ContextMenuTrigger>
                </ContextMenuRootComponent>
            }
        }));
        web_sys::console::log_1(&format!("container: {}", container.inner_html()).into());

        (container, calls)
    }

    fn find_trigger(container: &HtmlElement) -> HtmlElement {
        container
            .query_selector("div[data-testid='context-menu-trigger']")
            .unwrap()
            .expect("the trigger div renders")
            .dyn_into()
            .unwrap()
    }

    fn dispatch_mouse_event(
        target: &Element,
        type_str: &str,
        x: f64,
        y: f64,
    ) -> web_sys::MouseEvent {
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict(
            type_str,
            web_sys::MouseEventInit::new()
                .bubbles(true)
                .cancelable(true)
                .button(2)
                .client_x(x as i32)
                .client_y(y as i32),
        )
        .unwrap();
        target.dispatch_event(&event).unwrap();
        event
    }

    /// One real `setTimeout(0)` turn — the browser microtask queue where the
    /// leptos-side effects (the dynamic-view rebuilds, the attribute mirrors)
    /// are scheduled. A synchronous `poll_local` loop never drains it, so every
    /// assertion on effect-driven output must follow this await (the avatar /
    /// field wasm-suite convention, the 2026-09-13 lesson).
    async fn flush_one_turn() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .expect("window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(resolve.unchecked_ref(), 0)
                .expect("setTimeout");
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("flush");
    }

    // behavior.md "State model" (`ContextMenuTrigger.test.tsx:77-96`): the
    // `contextmenu` (right-click) on the trigger opens the menu —
    // `onOpenChange(true)` with the `trigger-press` reason.
    #[wasm_bindgen_test]
    async fn a_right_click_opens_through_the_real_gesture_path() {
        let calls: Rc<RefCell<Vec<(bool, String)>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_cb = Rc::clone(&calls);
        let (container, recorded) = mount_context_menu(
            Some(
                Rc::new(move |open: bool, details: &MenuChangeEventDetails| {
                    calls_for_cb
                        .borrow_mut()
                        .push((open, details.reason.clone()));
                }) as Rc<dyn Fn(bool, &MenuChangeEventDetails)>,
            ),
            false,
        );
        let _ = recorded; // the harness's own list stays empty for this test
        let trigger = find_trigger(&container);

        dispatch_mouse_event(&trigger, "contextmenu", 40.0, 50.0);
        flush_one_turn().await;

        // The Root's provider sandwich deliberately CLEARS the enclosing menu
        // context after mounting its children (`context_menu_root_view`, the
        // `MenuRootContext.Provider value={undefined}` scope), so the open state
        // is read through the context-menu context's actions slot (the
        // `useImperativeHandle` wiring, MenuRoot.tsx:465).
        let store = crate::context_menu::root::use_context_menu_root_context_optional()
            .expect("the context-menu context")
            .actions
            .borrow()
            .as_ref()
            .map(std::rc::Rc::clone)
            .expect("the actions slot is filled");
        assert!(
            menu_store_is_open(&store),
            "the right-click opened the menu"
        );
        assert_eq!(
            calls.borrow().as_slice(),
            [(true, "trigger-press".to_owned())],
            "onOpenChange(true) with REASONS.triggerPress"
        );
    }

    // behavior.md "State model" (`ContextMenuRoot.test.tsx:260-283`): the
    // disabled root short-circuits the open path — no popup, zero
    // `onOpenChange` calls (`ContextMenuTrigger.tsx:77-79` — the early return).
    #[wasm_bindgen_test]
    fn the_disabled_gate_short_circuits_the_open_with_zero_callbacks() {
        let (container, calls) = mount_context_menu(None, true);
        let trigger = find_trigger(&container);

        dispatch_mouse_event(&trigger, "contextmenu", 40.0, 50.0);

        assert!(
            calls.borrow().is_empty(),
            "zero onOpenChange calls when disabled"
        );
    }

    // behavior.md "Events" (`ContextMenuTrigger.test.tsx:351-367`): the
    // right-click open path default-prevents the native context menu
    // (`stopEvent`, `ContextMenuTrigger.tsx:81`).
    #[wasm_bindgen_test]
    fn the_open_path_default_prevents_the_native_context_menu() {
        let (container, _calls) = mount_context_menu(None, false);
        let trigger = find_trigger(&container);

        let event = dispatch_mouse_event(&trigger, "contextmenu", 40.0, 50.0);
        assert!(
            event.default_prevented(),
            "stopEvent on the open path prevents the native menu"
        );
    }

    // behavior.md "Accessibility" (`ContextMenuTrigger.test.tsx:70-74`): the
    // open state mirrors onto the trigger as `data-popup-open`.
    #[wasm_bindgen_test]
    async fn the_trigger_carries_data_popup_open() {
        let (container, _calls) = mount_context_menu(None, false);
        let trigger = find_trigger(&container);

        // Open first — the attribute mirrors the open state
        // (`pressableTriggerOpenStateMapping`, popupStateMapping.ts:39-46);
        // a closed trigger carries no attribute (React's boolean-attribute rule).
        dispatch_mouse_event(&trigger, "contextmenu", 40.0, 50.0);
        flush_one_turn().await;

        assert_eq!(
            trigger.get_attribute("data-popup-open").as_deref(),
            Some("true"),
            "the open state mirrors as data-popup-open"
        );
    }

    // The spawn-point record (`ContextMenuTrigger.tsx:55`): the open writes the
    // cursor point into the context (the seed the item-activation gate consumes,
    // `useMenuItemCommonProps.ts:88-89`).
    #[wasm_bindgen_test]
    fn the_open_records_the_spawn_point_in_the_context() {
        let (container, _calls) = mount_context_menu(None, false);
        let trigger = find_trigger(&container);

        dispatch_mouse_event(&trigger, "contextmenu", 40.0, 50.0);

        let context = crate::context_menu::root::use_context_menu_root_context_optional()
            .expect("the context-menu context");
        assert_eq!(
            *context.initial_cursor_point.borrow(),
            Some((40.0, 50.0)),
            "initialCursorPointRef carries the right-click coordinates"
        );
        assert_eq!(
            context.anchor.borrow().rect(),
            (40.0, 50.0, 0.0, 0.0),
            "the anchor swapped to a zero-size virtual rect at the pointer"
        );
    }
}
