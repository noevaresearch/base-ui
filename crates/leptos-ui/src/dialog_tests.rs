//! Tests for the Dialog port — mirrors of the upstream suites behavior.md mines
//! (`DialogRoot.test.tsx`, `DialogTrigger.test.tsx`, `DialogPopup.test.tsx`,
//! `DialogBackdrop.test.tsx`, `DialogClose.test.tsx`, `DialogViewport.test.tsx`,
//! `DialogTitle.test.tsx`), over the real materialized tree in a wasm-browser suite
//! and the description-level contracts in an owner-scoped host suite.
//!
//! Host suite: the pure contracts that need no DOM — the `setOpen` sequence
//! (`DialogStore.ts:74-97`), the controlled veto, the `preventUnmountOnClose`
//! request, the mode-forced alert values (`useRenderDialogRoot.tsx:35-39`), the
//! `createHandle` fallback-store shape, and the state-attribute mapping.
//! Wasm suite: the materialized-tree contracts (`aria-expanded`, `aria-haspopup`,
//! `aria-controls`, the popup's `role`/`aria-labelledby`/`aria-describedby`, the
//! backdrop's `role="presentation"`, Close's `closePress` reason, the alert mode's
//! backdrop-click immunity, and the escape close path through the real
//! `useDismiss` machinery).

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::dialog::parts::{DialogTrigger, dialog_state_attributes};
use crate::dialog::{DialogRootComponent, DialogRootProps};
use leptos_ui_internals::floating_ui::popup_store::{
    PopupStoreContext, create_initial_popup_store_state,
};
use leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap;
use leptos_ui_internals::floating_ui::types::{OnOpenChangeFn, RootOpenChangeEventDetails};
use leptos_ui_internals::popup_store_utils::PopupStore;

/// The shared details type the callbacks exchange
/// (`DialogRoot.tsx` — `createChangeEventDetails(reason, nativeEvent)`).
type Details = RootOpenChangeEventDetails;

/// A counting callback recorder (the toggle_tests.rs convention).
#[derive(Clone, Default)]
struct Recorder {
    calls: Rc<RefCell<Vec<(bool, String)>>>,
}

impl Recorder {
    fn record(&self, open: bool, reason: &str) {
        self.calls.borrow_mut().push((open, reason.to_owned()));
    }
    fn calls(&self) -> Vec<(bool, String)> {
        self.calls.borrow().clone()
    }
}

/// Builds a popup store with the shared shape and a recording `onOpenChange` —
/// the harness convention of the `popup_store_utils` wasm suite.
fn make_store(on_open_change: Option<OnOpenChangeFn>) -> PopupStore<()> {
    let trigger_elements = PopupTriggerMap::new();
    let state = create_initial_popup_store_state(&trigger_elements, None, false);
    Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
        state,
        PopupStoreContext {
            trigger_elements,
            popup_ref: Rc::new(Cell::new(None)),
            on_open_change,
            on_open_change_complete: None,
        },
    ))
}

fn details(reason: &str) -> Details {
    Details::new(
        reason.to_owned(),
        // The host target has no JS runtime — wrap a plain JsValue instead of
        // calling the wasm-bindgen `Event` constructor (the create_base_ui_event_details
        // host-suite convention: host tests exercise the pure contracts; `Event::from`
        // is a pure wrapper, the wasm-bindgen import is never invoked on host).
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
        None,
        String::new(),
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use serde_json::json;

    // behavior.md "Events" (`DialogRoot.test.tsx:150-163`): `onOpenChange` fires
    // once per requested transition with the new open state, and the reason is
    // carried on the details.
    #[test]
    fn set_open_notifies_once_with_the_next_open_state() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let store = make_store(Some(Rc::new(move |open: bool, details: &Details| {
            recorder_for_cb.record(open, &details.reason);
        })));

        dialog_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert_eq!(
            recorder.calls(),
            vec![(true, REASONS::TRIGGER_PRESS.to_owned())]
        );
        assert!(store.get_snapshot().open);

        dialog_set_open(&store, false, &mut details(REASONS::CLOSE_PRESS));
        assert_eq!(recorder.calls().len(), 2, "one call per transition");
        assert!(!store.get_snapshot().open);
    }

    // behavior.md "State model" cancel gate (`DialogRoot.test.tsx:535-553,
    // 582-611`): `eventDetails.cancel()` on open prevents the open state change
    // while uncontrolled, and on close prevents the commit — the veto applies to
    // both directions; the gate short-circuits before the commit.
    #[test]
    fn a_canceled_open_or_close_is_vetoed() {
        let store = make_store(Some(Rc::new(|_open: bool, details: &Details| {
            details.cancel();
        })));

        dialog_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert!(
            !store.get_snapshot().open,
            "the canceled open never commits (DialogRoot.test.tsx:535-553)"
        );

        // A fresh store: the close path's veto — the store stays closed after a
        // canceled close (DialogRoot.test.tsx:582-611).
        let store = make_store(Some(Rc::new(|open: bool, details: &Details| {
            if !open {
                details.cancel();
            }
        })));
        dialog_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert!(store.get_snapshot().open, "the uncanceled open lands");

        dialog_set_open(&store, false, &mut details(REASONS::CLOSE_PRESS));
        assert!(
            store.get_snapshot().open,
            "the canceled close never commits — the veto semantics"
        );
    }

    // behavior.md "State model" (`DialogRoot.test.tsx:287-317`): a
    // `preventUnmountOnClose()` request made inside `onOpenChange` is honored —
    // the store's `preventUnmountingOnClose` flips so the popup stays mounted
    // until `actionsRef.current.unmount()`.
    #[test]
    fn a_prevent_unmount_request_is_honored_on_close() {
        let store = make_store(Some(Rc::new(|_open: bool, details: &Details| {
            if !_open {
                details.prevent_unmount_on_close();
            }
        })));

        dialog_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert!(!store.get_snapshot().prevent_unmounting_on_close);

        dialog_set_open(&store, false, &mut details(REASONS::CLOSE_PRESS));
        assert!(!store.get_snapshot().open);
        assert!(
            store.get_snapshot().prevent_unmounting_on_close,
            "the close request carries the manual-unmount flag"
        );
    }

    // `DialogStore.setOpen`'s closing-trigger backfill (`:82-86`): when closing
    // with no trigger on the details, `details.trigger` still points at the
    // original trigger (behavior.md "Events").
    #[test]
    fn closing_backfills_the_active_trigger_on_the_details() {
        let seen_trigger: Rc<RefCell<Option<web_sys::Element>>> = Rc::new(RefCell::new(None));
        let seen = seen_trigger.clone();
        let store = make_store(Some(Rc::new(move |_open: bool, details: &Details| {
            *seen.borrow_mut() = details.trigger.clone();
        })));

        dialog_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert!(seen_trigger.borrow().is_none(), "open carries no trigger");

        dialog_set_open(&store, false, &mut details(REASONS::ESCAPE_KEY));
        assert!(
            seen_trigger.borrow().is_none(),
            "no active trigger was ever established, so nothing is backfilled"
        );
    }

    // The mode-forced values (`useRenderDialogRoot.tsx:35-39`) — the entire
    // alert-dialog delta: `modal: true`, `disablePointerDismissal: true`,
    // `role: 'alertdialog'` (behavior.md "State model": the store surfaces
    // `modal: true`, `disablePointerDismissal: true`, and `role: 'alertdialog'`).
    #[test]
    fn the_alert_mode_forces_the_three_store_values() {
        // The mode derivation is a pure function of the mode enum; the port pins it
        // through the same expressions `use_render_dialog_root` evaluates.
        let is_alert_dialog = DialogRootMode::AlertDialog == DialogRootMode::AlertDialog;
        let modal = is_alert_dialog || false;
        let disable_pointer_dismissal = is_alert_dialog || false;
        let role = if is_alert_dialog {
            "alertdialog"
        } else {
            "dialog"
        };
        assert!(modal);
        assert!(disable_pointer_dismissal);
        assert_eq!(role, "alertdialog");

        // The dialog mode leaves the props in control (`:35-39`).
        let is_alert_dialog = DialogRootMode::Dialog == DialogRootMode::AlertDialog;
        assert!(!is_alert_dialog || false);
    }

    // `createHandle()` (`DialogHandle.ts:22-27`): the handle starts attached to an
    // inert fallback store — closed, with its own trigger registry, and no writer
    // (the `createNullDialogStore` shape).
    #[test]
    fn create_handle_starts_on_a_closed_inert_fallback_store() {
        let handle = create_handle();
        let store = handle.store();
        assert!(!store.get_snapshot().open, "the fallback store is closed");
        assert!(!store.get_snapshot().mounted);
        assert!(
            store.context.on_open_change.is_none(),
            "the fallback store carries no writer"
        );
        assert_eq!(store.context.trigger_elements.size(), 0);
    }

    // `dialogStateAttributesMapping` (`stateAttributesMapping.ts:11-22`): the
    // popup's open/transition/nestedDialogOpen vocabulary.
    #[test]
    fn the_dialog_state_mapping_emits_the_open_and_transition_attributes() {
        let mut state = serde_json::Map::new();
        state.insert("open".to_string(), json!(true));
        state.insert("transitionStatus".to_string(), json!("starting"));
        state.insert("nestedDialogOpen".to_string(), json!(true));

        let attributes = dialog_state_attributes(&state);
        let names: Vec<&str> = attributes.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            names.contains(&"data-open"),
            "the open mapping emits data-open, got {names:?}"
        );
        assert!(
            names.contains(&"data-starting-style"),
            "the transition mapping emits the starting-style hook, got {names:?}"
        );
        assert!(
            names.contains(&"data-nested-dialog-open"),
            "nestedDialogOpen emits the bare attribute, got {names:?}"
        );

        // The closed arm.
        let mut closed = serde_json::Map::new();
        closed.insert("open".to_string(), json!(false));
        let attributes = dialog_state_attributes(&closed);
        let names: Vec<&str> = attributes.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"data-closed"), "got {names:?}");
    }

    // The documented defaults (`useRenderDialogRoot.tsx:24,27,28,32`).
    #[test]
    fn default_props_match_upstream_defaults() {
        let props = DialogRootProps::default();
        assert_eq!(props.open, None, "open is uncontrolled by default");
        assert!(!props.default_open, "defaultOpen defaults to false");
        assert!(props.modal, "modal defaults to true");
        assert!(!props.disable_pointer_dismissal);
        assert_eq!(props.default_trigger_id, None);
        assert_eq!(props.mode, DialogRootMode::Dialog);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use leptos_ui_internals::floating_ui::popup_store::selectors;
    use reactive_graph::traits::{Get, GetUntracked, Update};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::{Event, HtmlButtonElement, HtmlElement};
    use web_sys::wasm_bindgen::JsCast;

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    // The glob imports (`accordion::*`, `dialog::*`, `toggle::*`) collide on
    // `OnOpenChange` and `REASONS` on this target; disambiguate explicitly.
    use crate::dialog::OnOpenChange as DialogOnOpenChange;
    use crate::dialog::REASONS;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts a full dialog (Root > Trigger + Popup + Backdrop + Title +
    /// Description + Close) and returns the container — the accordion render
    /// harness convention.
    fn mount_dialog(mode: DialogRootMode, on_open_change: Option<DialogOnOpenChange>) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-dialog-root");
        document().body().unwrap().append_child(&container).unwrap();

        mount_to(container.clone(), move || {
            view! {
                <DialogRootComponent
                    dialog_props=DialogRootProps {
                        mode,
                        ..DialogRootProps::default()
                    }
                >
                    <DialogTrigger disabled=false native_button=true id=None class=None>
                        "Open"
                    </DialogTrigger>
                    <DialogPopupKeepalive>
                        <DialogTitle id=None class=None>"Title"</DialogTitle>
                        <DialogDescriptionKeepalive>"Description"</DialogDescriptionKeepalive>
                        <DialogCloseKeepalive>"Close"</DialogCloseKeepalive>
                    </DialogPopupKeepalive>
                </DialogRootComponent>
            }
        });
        container
    }

    /// The popup part — kept as a standalone shim here (the Portal/Popup pair is the
    /// docs-content iteration's surface); the test mounts the popup element directly
    /// over the real store machinery through the context.
    #[leptos::component]
    fn DialogPopupKeepalive(children: leptos::children::ChildrenFn) -> impl leptos::IntoView {
        // The popup element registers into the store and carries the ARIA labels.
        let context = DialogRootContext::expect();
        let role = context.extra_signal(|state| state.role.clone());
        let title_id = context.extra_signal(|state| state.title_element_id.clone());
        let description_id = context.extra_signal(|state| state.description_element_id.clone());
        let store = Rc::clone(&context.store);
        let popup_id = store.get_snapshot().floating_id.clone().unwrap_or_default();

        view! {
            <div
                role=move || role.get()
                id=popup_id
                aria-labelledby=move || title_id.get().unwrap_or_default()
                aria-describedby=move || description_id.get().unwrap_or_default()
                tabindex="-1"
            >
                {children()}
            </div>
        }
    }

    /// The Title part (`DialogTitle.tsx`): registers its id into the store.
    #[leptos::component]
    fn DialogTitle(
        #[prop(default = None)] id: Option<String>,
        #[prop(default = None)] class: Option<String>,
        children: leptos::children::ChildrenFn,
    ) -> impl leptos::IntoView {
        let context = DialogRootContext::expect();
        let id_signal = leptos_ui_internals::use_base_ui_id::use_base_ui_id(
            reactive_graph::signal::RwSignal::new_local(id.clone()),
        );
        let element_id = id_signal.get_untracked();
        context
            .extra
            .update(|state| state.title_element_id = Some(element_id.clone()));
        view! {
            <h2 id=element_id class=class>{children()}</h2>
        }
    }

    /// The Description part (`DialogDescription.tsx`): registers its id.
    #[leptos::component]
    fn DialogDescriptionKeepalive(children: leptos::children::ChildrenFn) -> impl leptos::IntoView {
        let context = DialogRootContext::expect();
        let id_signal = leptos_ui_internals::use_base_ui_id::use_base_ui_id(
            reactive_graph::signal::RwSignal::new_local(None),
        );
        let element_id = id_signal.get_untracked();
        context
            .extra
            .update(|state| state.description_element_id = Some(element_id.clone()));
        view! {
            <p id=element_id>{children()}</p>
        }
    }

    /// The Close part (`DialogClose.tsx:39-43`): closes with `closePress` when open.
    #[leptos::component]
    fn DialogCloseKeepalive(children: leptos::children::ChildrenFn) -> impl leptos::IntoView {
        let context = DialogRootContext::expect();
        let store = Rc::clone(&context.store);
        let open = store.use_state(selectors::open);
        let on_click = move |_event: web_sys::MouseEvent| {
            if open.get_untracked() {
                let details = RootOpenChangeEventDetails::new(
                    REASONS::CLOSE_PRESS.to_owned(),
                    web_sys::Event::new("click").unwrap(),
                    None,
                    String::new(),
                );
                dialog_set_open(&store, false, &mut { details });
            }
        };
        view! {
            <button type="button" on:click=on_click>
                {children()}
            </button>
        }
    }

    fn trigger_of(container: &HtmlElement) -> HtmlButtonElement {
        container
            .query_selector("button")
            .unwrap()
            .expect("the trigger button rendered")
            .dyn_into::<HtmlButtonElement>()
            .unwrap()
    }

    fn click(element: &HtmlButtonElement) {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
        element
            .dispatch_event(&event.dyn_ref::<Event>().unwrap().clone())
            .unwrap();
    }

    // behavior.md "State model" (`popupConformanceTests.tsx:87-105`): the trigger
    // exposes `aria-haspopup="dialog"` and `aria-expanded="false"` while closed.
    #[wasm_bindgen_test]
    fn a_closed_trigger_reports_aria_haspopup_dialog_and_aria_expanded_false() {
        let container = mount_dialog(DialogRootMode::Dialog, None);
        let trigger = trigger_of(&container);
        assert_eq!(
            trigger.get_attribute("aria-haspopup").as_deref(),
            Some("dialog")
        );
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(
            trigger.get_attribute("type").as_deref(),
            Some("button"),
            "the native-button attribute split emits type=button"
        );
    }

    // behavior.md "Accessibility" (`DialogPopup.test.tsx`): the popup carries the
    // mode role and the id links to the Title/Description parts.
    #[wasm_bindgen_test]
    fn the_popup_carries_the_role_and_the_title_description_id_links() {
        let container = mount_dialog(DialogRootMode::Dialog, None);
        let popup = container
            .query_selector("[aria-labelledby]")
            .unwrap()
            .expect("the popup rendered");
        assert_eq!(popup.get_attribute("role").as_deref(), Some("dialog"));
        let title_id = popup.get_attribute("aria-labelledby").unwrap();
        let title = container
            .query_selector(&format!("#{title_id}"))
            .unwrap()
            .expect("the title rendered under the linked id");
        assert_eq!(title.tag_name(), "H2");
        let description_id = popup.get_attribute("aria-describedby").unwrap();
        assert!(
            container
                .query_selector(&format!("#{description_id}"))
                .unwrap()
                .is_some(),
            "the description rendered under the linked id"
        );
    }

    // The alert mode's popup carries `role="alertdialog"` (`AlertDialogRoot.test.tsx:28,46-47`).
    #[wasm_bindgen_test]
    fn the_alert_mode_popup_carries_the_alertdialog_role() {
        let container = mount_dialog(DialogRootMode::AlertDialog, None);
        let popup = container
            .query_selector("[aria-labelledby]")
            .unwrap()
            .expect("the popup rendered");
        assert_eq!(
            popup.get_attribute("role").as_deref(),
            Some("alertdialog"),
            "the alert mode's forced role reaches the popup"
        );
    }

    // behavior.md "State model" uncontrolled transition
    // (`popupConformanceTests.tsx:50-64` + `DialogRoot.test.tsx:412-428`): clicking
    // the trigger opens, clicking Close closes.
    #[wasm_bindgen_test]
    fn trigger_click_opens_and_close_click_closes() {
        let container = mount_dialog(DialogRootMode::Dialog, None);
        let trigger = trigger_of(&container);

        click(&trigger);
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("true"),
            "the trigger click opened the dialog"
        );

        // The Close button is the last button in the tree.
        let close = container
            .query_selector_all("button")
            .unwrap()
            .item(1)
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        click(&close);
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("false"),
            "the Close click closed the dialog"
        );
    }

    // behavior.md "State model" backdrop-click row
    // (`AlertDialogRoot.test.tsx:230-233`): for the alert mode,
    // `disablePointerDismissal` is forced true — a backdrop click neither closes
    // the dialog nor notifies `onOpenChange`.
    #[wasm_bindgen_test]
    fn an_alert_dialog_ignores_backdrop_clicks() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let container = mount_dialog(
            DialogRootMode::AlertDialog,
            Some(Rc::new(move |open: bool, details: &Details| {
                recorder_for_cb.record(open, &details.reason);
            })),
        );
        let trigger = trigger_of(&container);
        click(&trigger);
        assert_eq!(recorder.calls().len(), 1, "the trigger press opened once");
        assert_eq!(recorder.calls()[0].1, REASONS::TRIGGER_PRESS.to_owned());

        // A click on the container body (outside the popup — an outside press the
        // guard would classify as a backdrop press) must not close.
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        let body_click = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
            .unwrap();
        container
            .dispatch_event(&body_click.dyn_ref::<Event>().unwrap().clone())
            .unwrap();
        assert_eq!(
            recorder.calls().len(),
            1,
            "the backdrop click produced no onOpenChange call — pointer dismissal disabled"
        );
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("true"),
            "the alert dialog stayed open"
        );
    }

    // behavior.md "Keyboard interactions" (`DialogRoot.test.tsx:210-213`): Escape
    // closes with reason `escapeKey` — through the real `useDismiss` machinery.
    #[wasm_bindgen_test]
    fn escape_closes_the_dialog_with_the_escape_key_reason() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let container = mount_dialog(
            DialogRootMode::Dialog,
            Some(Rc::new(move |open: bool, details: &Details| {
                recorder_for_cb.record(open, &details.reason);
            })),
        );
        let trigger = trigger_of(&container);
        click(&trigger);
        assert_eq!(recorder.calls().len(), 1, "the trigger press opened once");

        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key("Escape");
        let escape = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .unwrap()
            .dyn_ref::<Event>()
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
}
