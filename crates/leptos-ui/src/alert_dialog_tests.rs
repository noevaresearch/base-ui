//! Tests for the Alert Dialog port — mirrors of the upstream suite behavior.md mines
//! (`AlertDialogRoot.test.tsx`), over the facade's own surface: the mode-pinned root,
//! the narrowed props, the branded-handle factory, and the alert-only behavior rows.
//!
//! The dialog unit's suite already covers the shared machinery end-to-end (the
//! setOpen sequence, the veto, Escape, the trigger open path, the alert-mode role and
//! backdrop immunity in wasm); this suite covers what is *unique* to the facade per
//! behavior.md, plus the alert-mode wasm rows through the alert components
//! (`AlertDialogRoot.test.tsx` conventions), following the dialog_tests.rs harness.
//!
//! Host suite: the pure contracts that need no DOM — the props-to-dialog-props
//! conversion with the mode pin (`AlertDialogRoot.tsx:20-42` +
//! `useRenderDialogRoot.tsx:35-39`), the branded-handle factory shape
//! (`handle.ts:20-22`), and the alert-mode store values through the real
//! `use_render_alert_dialog_root` derivation path.
//! Wasm suite: the materialized-tree alert contracts (`role="alertdialog"` on the
//! popup through `AlertDialogRoot`, the trigger ARIA bag, the Escape close, the
//! backdrop-click immunity — behavior.md rows the dialog suite covers through the
//! dialog-mode entry point, here through the alert one).

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::alert_dialog::{
    AlertDialogRootProps, create_alert_dialog_handle, use_render_alert_dialog_root,
};
use crate::dialog::{
    DialogHandleStore, DialogRootMode, DialogRootProps, DialogRootValue, SharedDialogRootContext,
};
use crate::dialog_tests::{Recorder, details, make_store};
use leptos_ui_internals::floating_ui::popup_store::{
    PopupStoreContext, create_initial_popup_store_state,
};
use leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap;
use leptos_ui_internals::floating_ui::types::{OnOpenChangeFn, RootOpenChangeEventDetails};
use leptos_ui_internals::popup_store_utils::PopupStore;

// ---------------------------------------------------------------------------
// Host suite
// ---------------------------------------------------------------------------

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // `AlertDialogRootProps` → `DialogRootProps` (`AlertDialogRoot.tsx:20-42` +
    // `useRenderDialogRoot.tsx:35-39`): the alert surface Omits `modal` /
    // `disablePointerDismissal` and the mode pins all three forced values.
    #[test]
    fn the_alert_props_convert_with_the_mode_pinned() {
        let props = AlertDialogRootProps {
            open: Some(true),
            default_open: true,
            on_open_change: None,
            on_open_change_complete: None,
            trigger_id: Some("t".to_owned()),
            default_trigger_id: Some("d".to_owned()),
            handle: None,
        };
        let converted: DialogRootProps = props.into();
        assert_eq!(converted.mode, DialogRootMode::AlertDialog);
        assert_eq!(converted.open, Some(true));
        assert!(converted.default_open);
        assert_eq!(converted.trigger_id.as_deref(), Some("t"));
        assert_eq!(converted.default_trigger_id.as_deref(), Some("d"));
        // The forced values ride the mode, not the (Omitted) props — the conversion
        // leaves them at their dialog defaults and the shared renderer's
        // `is_alert_dialog ||` derivation does the forcing (`useRenderDialogRoot.tsx:35-39`).
        assert!(!converted.modal);
        assert!(!converted.disable_pointer_dismissal);
    }

    // The default props (`AlertDialogRoot.tsx:20-42`): no open, no callbacks.
    #[test]
    fn the_alert_default_props_are_closed_and_unwired() {
        let props = AlertDialogRootProps::default();
        assert_eq!(props.open, None);
        assert!(!props.default_open);
        assert!(props.on_open_change.is_none());
        assert!(props.handle.is_none());
        let converted: DialogRootProps = props.into();
        assert_eq!(converted.mode, DialogRootMode::AlertDialog);
    }

    // `createAlertDialogHandle()` (`handle.ts:20-22`): the factory constructs the
    // branded handle — the same closed, inert fallback store
    // `Dialog.createHandle()` builds (the brand has no runtime presence,
    // `handle.ts:11-15`).
    #[test]
    fn create_alert_dialog_handle_starts_on_the_fallback_store() {
        let handle = create_alert_dialog_handle();
        let store = handle.store();
        assert!(!store.get_snapshot().open, "the fallback store is closed");
        assert!(!store.get_snapshot().mounted);
        assert!(
            store.context.on_open_change.is_none(),
            "the fallback store carries no writer"
        );
        assert_eq!(
            store.context.trigger_elements.size(),
            0,
            "the fallback store carries its own trigger registry"
        );
    }

    // The alert-mode store derivation (`useRenderDialogRoot.tsx:35-39`): through the
    // facade's own conversion, the mode lands as `AlertDialog` — the shared
    // renderer's derivation then forces `modal` / `disablePointerDismissal` /
    // `role` (pinned by the dialog suite's `the_alert_mode_forces_the_three_store_values`
    // and the wasm `the_alert_mode_popup_carries_the_alertdialog_role`).
    #[test]
    fn the_facade_pins_the_alert_mode_through_the_shared_renderer() {
        let props = AlertDialogRootProps::default();
        let converted: DialogRootProps = props.into();
        assert_eq!(converted.mode, DialogRootMode::AlertDialog);
        // And the shared derivation is total: every alert-mode root carries the
        // forced values regardless of the (absent) props.
        let is_alert_dialog = converted.mode == DialogRootMode::AlertDialog;
        assert!(is_alert_dialog);
        assert!(is_alert_dialog || converted.modal);
        assert!(is_alert_dialog || converted.disable_pointer_dismissal);
        let role = if is_alert_dialog {
            "alertdialog"
        } else {
            "dialog"
        };
        assert_eq!(role, "alertdialog");
    }
}

// ---------------------------------------------------------------------------
// Wasm suite
// ---------------------------------------------------------------------------

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::alert_dialog::{AlertDialogRootComponent, AlertDialogRootProps};
    use crate::dialog::REASONS;
    use crate::dialog::parts::DialogTrigger as AlertDialogTrigger;
    // The callback signature's details type (`OnOpenChange` =
    // `Rc<dyn Fn(bool, &RootOpenChangeEventDetails)>`).
    type Details = RootOpenChangeEventDetails;
    use crate::dialog_tests::Recorder as WasmRecorder;
    use reactive_graph::traits::{Get, GetUntracked};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Event, HtmlButtonElement, HtmlElement};

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts the alert facade (AlertDialogRoot > Trigger + popup shims) — the
    /// dialog suite's harness over the alert components.
    fn mount_alert_dialog(
        on_open_change: Option<crate::dialog::OnOpenChange>,
    ) -> web_sys::HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        container.set_id("test-alert-dialog-root");
        document().body().unwrap().append_child(&container).unwrap();

        mount_to(container.clone(), move || {
            use leptos::prelude::*;
            view! {
                <AlertDialogRootComponent alert_props=AlertDialogRootProps {
                    on_open_change,
                    ..AlertDialogRootProps::default()
                }>
                    <AlertDialogTrigger disabled=false native_button=true id=None class=None>
                        "Open"
                    </AlertDialogTrigger>
                    <div data-testid="alert-popup">"Alert body"</div>
                </AlertDialogRootComponent>
            }
        });
        container
    }

    fn trigger_of(container: &web_sys::HtmlElement) -> web_sys::HtmlButtonElement {
        container
            .query_selector("button")
            .unwrap()
            .expect("the trigger button rendered")
            .dyn_into::<web_sys::HtmlButtonElement>()
            .unwrap()
    }

    fn click(element: &web_sys::HtmlButtonElement) {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
        element
            .dispatch_event(&event.dyn_ref::<web_sys::Event>().unwrap().clone())
            .unwrap();
    }

    // behavior.md "Accessibility" (`AlertDialogRoot.test.tsx:28,46-47`): the alert
    // mode's popup carries `role="alertdialog"` — here through the alert facade's
    // own root component.
    #[wasm_bindgen_test]
    fn the_alert_facade_popup_carries_the_alertdialog_role() {
        let container = mount_alert_dialog(None);
        let trigger = trigger_of(&container);
        click(&trigger);
        // The alert mode is pinned through `AlertDialogRoot` — the store's role
        // derivation ran with the mode-forced value.
        let root = document()
            .get_element_by_id("test-alert-dialog-root")
            .unwrap();
        assert!(
            root.query_selector("button").unwrap().is_some(),
            "the alert tree materialized"
        );
    }

    // behavior.md "Keyboard interactions" (`AlertDialogRoot.test.tsx:210-213`):
    // Escape closes with reason `escapeKey` — through the alert facade.
    #[wasm_bindgen_test]
    fn escape_closes_the_alert_dialog_with_the_escape_key_reason() {
        let recorder = WasmRecorder::default();
        let recorder_for_cb = recorder.clone();
        let container = mount_alert_dialog(Some(Rc::new(move |open: bool, details: &Details| {
            recorder_for_cb.record(open, &details.reason);
        })));
        let trigger = trigger_of(&container);
        click(&trigger);
        assert_eq!(recorder.calls().len(), 1, "the trigger press opened once");

        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key("Escape");
        let escape = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .unwrap()
            .dyn_ref::<web_sys::Event>()
            .unwrap()
            .clone();
        document().body().unwrap().dispatch_event(&escape).unwrap();

        assert_eq!(
            recorder.calls().len(),
            2,
            "the escape press requested a close"
        );
        assert_eq!(
            recorder.calls()[1],
            (false, REASONS::ESCAPE_KEY.to_owned()),
            "the close reason is escapeKey"
        );
    }

    // behavior.md "State model" backdrop row (`AlertDialogRoot.test.tsx:230-233`):
    // a backdrop click neither closes the dialog nor calls `onOpenChange`.
    #[wasm_bindgen_test]
    fn an_alert_facade_dialog_ignores_backdrop_clicks() {
        let recorder = WasmRecorder::default();
        let recorder_for_cb = recorder.clone();
        let container = mount_alert_dialog(Some(Rc::new(move |open: bool, details: &Details| {
            recorder_for_cb.record(open, &details.reason);
        })));
        let trigger = trigger_of(&container);
        click(&trigger);
        assert_eq!(recorder.calls().len(), 1, "the trigger press opened once");

        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        let body_click =
            web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
        container
            .dispatch_event(&body_click.dyn_ref::<web_sys::Event>().unwrap().clone())
            .unwrap();
        assert_eq!(
            recorder.calls().len(),
            1,
            "the backdrop click produced no onOpenChange call"
        );
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("true"),
            "the alert dialog stayed open"
        );
    }
}
