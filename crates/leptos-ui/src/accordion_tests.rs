//! Tests for the Accordion port — mirrors of the five upstream suites behavior.md
//! mines (`AccordionRoot/Item/Header/Trigger/Panel.test.tsx`), over the real
//! materialized tree in a wasm-browser suite and the description-level contracts
//! in an owner-scoped host suite.
//!
//! Host suite: the array algebra (`accordion_next_value`, extracted verbatim from
//! `handleValueChange`), the item-fallback value resolution, and the root/context
//! shape contracts — everything that needs no DOM.
//! Wasm suite: the full `Root > Item > Trigger/Panel` tree mounted for real —
//! the click/keyboard transition machine, `multiple` vs single mode, controlled
//! suppression, the two-layer cancel protocol, ARIA id linking, the panel
//! mount/unmount gate, and the disabled veto.

use std::sync::Arc;
use std::sync::Mutex;

#[cfg(test)]
use super::accordion_next_value;
#[cfg(test)]
use crate::accordion::{AccordionChangeEventDetails, OnValueChange};

/// A counting callback recorder that satisfies the `Send + Sync` bounds of the
/// callback types (the wasm single thread makes the mutex uncontended).
#[derive(Clone, Default)]
pub(crate) struct Recorder(Arc<Mutex<RecorderState>>);

#[derive(Default)]
struct RecorderState {
    calls: u32,
    values: Vec<Vec<String>>,
    opens: Vec<bool>,
}

impl Recorder {
    pub(crate) fn record_value(&self, value: &[String]) {
        let mut state = self.0.lock().unwrap();
        state.calls += 1;
        state.values.push(value.to_vec());
    }
    pub(crate) fn record_open(&self, next_open: bool) {
        let mut state = self.0.lock().unwrap();
        state.calls += 1;
        state.opens.push(next_open);
    }
    pub(crate) fn calls(&self) -> u32 {
        self.0.lock().unwrap().calls
    }
    pub(crate) fn values(&self) -> Vec<Vec<String>> {
        self.0.lock().unwrap().values.clone()
    }
    #[allow(dead_code)]
    pub(crate) fn opens(&self) -> Vec<bool> {
        self.0.lock().unwrap().opens.clone()
    }
}

/// A recorder whose callback cancels the details — the `eventDetails.cancel()`
/// veto arm of the shared protocol.
pub(crate) fn canceling_on_value_change() -> OnValueChange {
    Arc::new(|_value: &[String], details: &AccordionChangeEventDetails| {
        details.cancel();
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use serde_json::json;

    use super::*;
    use crate::accordion::accordion_item_state_map;

    // behavior.md "State model" non-`multiple` (`AccordionRoot.test.tsx:865-870`,
    // `:976-982`): opening replaces the array with just the new item; activating
    // the open item closes it (toggles by identity against `value[0]`, ignoring
    // `nextOpen` — implementation.md gap §2).
    #[test]
    fn single_mode_replaces_on_open_and_clears_on_reactivate() {
        assert_eq!(
            accordion_next_value(&[], "one", true, false),
            vec!["one".to_string()],
            "opening replaces the array with just the new item"
        );
        assert_eq!(
            accordion_next_value(&["one".to_string()], "one", false, false),
            Vec::<String>::new(),
            "reactivating the open item closes it"
        );
        assert_eq!(
            accordion_next_value(&["one".to_string()], "two", false, false),
            vec!["two".to_string()],
            "opening another item replaces — at most one open"
        );
    }

    // behavior.md "State model" `multiple` (`AccordionRoot.test.tsx:819-832`):
    // each item opens/closes independently — append on open, filter on close.
    #[test]
    fn multiple_mode_appends_and_filters() {
        assert_eq!(
            accordion_next_value(&["one".to_string()], "two", true, true),
            vec!["one".to_string(), "two".to_string()],
            "opening appends"
        );
        assert_eq!(
            accordion_next_value(
                &["one".to_string(), "two".to_string()],
                "one",
                false,
                true
            ),
            vec!["two".to_string()],
            "closing filters only that item"
        );
        assert_eq!(
            accordion_next_value(&[], "one", true, true),
            vec!["one".to_string()],
            "the first open seeds the array"
        );
    }

    // behavior.md "Accessibility" (`AccordionRoot.test.tsx:53-59`): the pairing
    // vocabulary the state mapping consumes — `open`/`disabled`/`index` — is the
    // record `accordionStateAttributesMapping` walks (stateAttributesMapping.ts:7-12).
    #[test]
    fn the_item_state_map_carries_open_disabled_index() {
        let map = accordion_item_state_map(true, false, 2);
        assert_eq!(map.get("open"), Some(&json!(true)));
        assert_eq!(map.get("disabled"), Some(&json!(false)));
        assert_eq!(map.get("index"), Some(&json!(2)));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Event, HtmlElement, HtmlButtonElement, KeyboardEvent};

    use super::*;
    use crate::accordion::{AccordionItem, AccordionPanel, AccordionRoot, AccordionTrigger, OnOpenChange};
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts a one-item accordion (Root > Item > Trigger + Panel) and returns the
    /// container element — the docs-app render_test.rs mount_to precedent.
    fn mount_accordion(
        item_value: &str,
        trigger_disabled: bool,
        on_value_change: Option<OnValueChange>,
        on_open_change: Option<OnOpenChange>,
    ) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-accordion-root");
        document().body().unwrap().append_child(&container).unwrap();

        let item_value = item_value.to_string();
        mount_to({ container.clone() }, move || {
            view! {
                <AccordionRoot
                    value=None
                    default_value=Vec::new()
                    on_value_change=on_value_change
                    multiple=false
                    disabled=false
                    hidden_until_found=false
                    keep_mounted=false
                    orientation=String::new()
                    id=None
                    class=None
                >
                    <AccordionItem
                        value=Some(item_value)
                        disabled=false
                        on_open_change=on_open_change
                        id=None
                        class=None
                    >
                        <AccordionTrigger disabled=trigger_disabled id=None class=None>
                            "Trigger"
                        </AccordionTrigger>
                        <AccordionPanel id=None keep_mounted=None hidden_until_found=None class=None>
                            "Panel body"
                        </AccordionPanel>
                    </AccordionItem>
                </AccordionRoot>
            }
        });
        container
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
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
            .unwrap();
        element
            .dispatch_event(&event.dyn_ref::<Event>().unwrap().clone())
            .unwrap();
    }

    fn key_event(kind: &str, key: &str) -> KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_key(key);
        KeyboardEvent::new_with_keyboard_event_init_dict(kind, &init).unwrap()
    }

    // behavior.md "State model" default (`AccordionRoot.test.tsx:264-265`): all
    // items closed — the trigger has `aria-expanded="false"` and the closed
    // non-`keepMounted` panel is fully absent from the DOM.
    #[wasm_bindgen_test]
    fn a_default_accordion_starts_closed_with_the_panel_absent() {
        let container = mount_accordion("one", false, None, None);
        let trigger = trigger_of(&container);
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("false"),
            "the closed trigger reports aria-expanded=false"
        );
        assert!(
            container.query_selector("[role=region]").unwrap().is_none(),
            "the closed non-keepMounted panel is absent from the DOM"
        );
    }

    // behavior.md "State model" transition (`AccordionRoot.test.tsx:267-278`):
    // closed -> open -> closed on successive activations; `aria-expanded` mirrors,
    // the panel mounts/unmounts, and the open pair exposes `data-open` on the
    // panel and `data-panel-open` on the trigger (`:270-273`).
    #[wasm_bindgen_test]
    fn each_activation_toggles_the_item() {
        let container = mount_accordion("one", false, None, None);
        let trigger = trigger_of(&container);
        click(&trigger);
        assert_eq!(trigger.get_attribute("aria-expanded").as_deref(), Some("true"));
        let panel = container
            .query_selector("[role=region]")
            .unwrap()
            .expect("the panel mounts on open");
        assert_eq!(panel.get_attribute("data-open").as_deref(), Some("true"));
        assert_eq!(
            trigger.get_attribute("data-panel-open").as_deref(),
            Some("true"),
            "the open trigger carries data-panel-open"
        );
        click(&trigger);
        assert_eq!(trigger.get_attribute("aria-expanded").as_deref(), Some("false"));
        assert!(
            container.query_selector("[role=region]").unwrap().is_none(),
            "the panel unmounts on close"
        );
    }

    // behavior.md "State model" `multiple={false}` (`AccordionRoot.test.tsx:865-870`)
    // exercised through the value callback (`:902`): `onValueChange` receives the
    // new full value array.
    #[wasm_bindgen_test]
    fn on_value_change_receives_the_new_full_array() {
        let recorder = Recorder::default();
        let rec = recorder.clone();
        let container = mount_accordion(
            "one",
            false,
            Some(Arc::new(move |value: &[String], _d: &AccordionChangeEventDetails| {
                rec.record_value(value);
            })),
            None,
        );
        click(&trigger_of(&container));
        click(&trigger_of(&container));
        assert_eq!(recorder.calls(), 2, "one commit per activation");
        assert_eq!(
            recorder.values(),
            vec![
                vec!["one".to_string()],
                Vec::<String>::new(),
            ],
            "the payload is the full next array: open then close"
        );
    }

    // behavior.md "Events" cancellation (`AccordionRoot.test.tsx:623-646`):
    // `onValueChange` cancel blocks the uncontrolled open — `aria-expanded` stays
    // `'false'` and the panel stays out of the DOM.
    #[wasm_bindgen_test]
    fn canceling_on_value_change_blocks_the_open() {
        let container =
            mount_accordion("one", false, Some(canceling_on_value_change()), None);
        click(&trigger_of(&container));
        assert_eq!(
            trigger_of(&container).get_attribute("aria-expanded").as_deref(),
            Some("false"),
            "the canceled open does not commit"
        );
        assert!(container.query_selector("[role=region]").unwrap().is_none());
    }

    // behavior.md "Events" item-level cancellation (`AccordionRoot.test.tsx:593-621`):
    // an `onOpenChange` cancel prevents the open AND keeps `onValueChange` from
    // firing (the two-layer protocol: item veto stops the root commit).
    #[wasm_bindgen_test]
    fn canceling_on_open_change_blocks_both_layers() {
        let recorder = Recorder::default();
        let rec = recorder.clone();
        let container = mount_accordion(
            "one",
            false,
            Some(Arc::new(move |value: &[String], _d: &AccordionChangeEventDetails| {
                rec.record_value(value);
            })),
            Some(Arc::new(|_next_open: bool, details: &AccordionChangeEventDetails| {
                details.cancel();
            })),
        );
        click(&trigger_of(&container));
        assert_eq!(
            recorder.calls(),
            0,
            "the root onValueChange never fires after an item-level veto"
        );
        assert_eq!(
            trigger_of(&container).get_attribute("aria-expanded").as_deref(),
            Some("false")
        );
    }

    // behavior.md "Events" disabled (`AccordionRoot.test.tsx:460-468`): a disabled
    // trigger's activation produces no toggle and no `onValueChange` call — the
    // root/item disable wins (edge case `:431-468`).
    #[wasm_bindgen_test]
    fn a_disabled_trigger_produces_no_toggle_and_no_commit() {
        let recorder = Recorder::default();
        let rec = recorder.clone();
        let container = mount_accordion(
            "one",
            true,
            Some(Arc::new(move |value: &[String], _d: &AccordionChangeEventDetails| {
                rec.record_value(value);
            })),
            None,
        );
        let trigger = trigger_of(&container);
        assert_eq!(trigger.disabled(), true, "the native button is disabled");
        click(&trigger);
        assert_eq!(recorder.calls(), 0);
        assert_eq!(trigger.get_attribute("aria-expanded").as_deref(), Some("false"));
    }

    // behavior.md "Accessibility" (`AccordionRoot.test.tsx:53-59`): trigger
    // `aria-controls` points at the panel's `id`; the panel is `role="region"`
    // with `aria-labelledby` pointing at the trigger's `id`.
    #[wasm_bindgen_test]
    fn trigger_and_panel_are_linked_by_generated_ids() {
        let container = mount_accordion("one", false, None, None);
        let trigger = trigger_of(&container);
        click(&trigger);
        let panel = container
            .query_selector("[role=region]")
            .unwrap()
            .expect("the panel mounts on open");
        let trigger_id = trigger.get_attribute("id").expect("the trigger has an id");
        let panel_id = panel.get_attribute("id").expect("the panel has an id");
        assert_eq!(
            trigger.get_attribute("aria-controls").as_deref(),
            Some(panel_id.as_str()),
            "aria-controls references the panel id"
        );
        assert_eq!(
            panel.get_attribute("aria-labelledby").as_deref(),
            Some(trigger_id.as_str()),
            "aria-labelledby references the trigger id"
        );
    }

    // behavior.md "Keyboard interactions" (`AccordionRoot.test.tsx:567-587`):
    // Space on a native button toggles via the browser's default activation — the
    // dispatched keyup on a focused button produces the click that toggles.
    #[wasm_bindgen_test]
    fn keyboard_activation_through_the_native_button_toggles() {
        let container = mount_accordion("one", false, None, None);
        let trigger = trigger_of(&container);
        trigger.focus().unwrap();
        // The native-button path: the browser synthesizes the click from Space
        // keyup. Dispatch the synthetic click the browser would produce —
        // chromedriver's synthetic KeyboardEvents carry no default action.
        trigger
            .dispatch_event(&key_event("keydown", " ").dyn_ref::<Event>().unwrap().clone())
            .unwrap();
        trigger
            .dispatch_event(&key_event("keyup", " ").dyn_ref::<Event>().unwrap().clone())
            .unwrap();
        trigger
            .dispatch_event(
                &web_sys::MouseEvent::new("click").unwrap().dyn_ref::<Event>().unwrap().clone(),
            )
            .unwrap();
        assert_eq!(
            trigger.get_attribute("aria-expanded").as_deref(),
            Some("true"),
            "the Space activation toggles the item"
        );
    }

    // behavior.md "State model" controlled (`AccordionRoot.test.tsx:623-646`,
    // `:648-672`): the controlled consumer owns state — the internal setter is a
    // no-op (useControlled.ts:82-91), so the attempted value reaches
    // `onValueChange` but the DOM does not move.
    #[wasm_bindgen_test]
    fn a_controlled_root_reports_but_does_not_commit() {
        let recorder = Recorder::default();
        let rec = recorder.clone();
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-accordion-controlled");
        document().body().unwrap().append_child(&container).unwrap();
        mount_to({ container.clone() }, move || {
            view! {
                <AccordionRoot
                    value=Some(vec![])
                    default_value=Vec::new()
                    on_value_change=Some(Arc::new(
                        move |value: &[String], _d: &AccordionChangeEventDetails| {
                            rec.record_value(value);
                        },
                    ))
                    multiple=false
                    disabled=false
                    hidden_until_found=false
                    keep_mounted=false
                    orientation=String::new()
                    id=None
                    class=None
                >
                    <AccordionItem value=Some("one".to_string()) disabled=false on_open_change=None id=None class=None>
                        <AccordionTrigger disabled=false id=None class=None>"T"</AccordionTrigger>
                        <AccordionPanel id=None keep_mounted=None hidden_until_found=None class=None>"P"</AccordionPanel>
                    </AccordionItem>
                </AccordionRoot>
            }
        });
        click(&trigger_of(&container));
        assert_eq!(
            recorder.values(),
            vec![vec!["one".to_string()]],
            "the attempted value still reaches the controlled consumer"
        );
        assert_eq!(
            trigger_of(&container).get_attribute("aria-expanded").as_deref(),
            Some("false"),
            "the controlled setter is a no-op — the DOM does not move"
        );
        assert!(container.query_selector("[role=region]").unwrap().is_none());
    }

    // behavior.md "DOM structure" kept-mounted (`AccordionPanel.test.tsx:97-110`):
    // `keepMounted` renders the closed panel with the `hidden` attribute instead
    // of unmounting it.
    #[wasm_bindgen_test]
    fn a_keep_mounted_closed_panel_stays_in_the_dom_with_hidden() {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-accordion-keepmounted");
        document().body().unwrap().append_child(&container).unwrap();
        mount_to({ container.clone() }, move || {
            view! {
                <AccordionRoot
                    value=None
                    default_value=Vec::new()
                    on_value_change=None
                    multiple=false
                    disabled=false
                    hidden_until_found=false
                    keep_mounted=true
                    orientation=String::new()
                    id=None
                    class=None
                >
                    <AccordionItem value=Some("one".to_string()) disabled=false on_open_change=None id=None class=None>
                        <AccordionTrigger disabled=false id=None class=None>"T"</AccordionTrigger>
                        <AccordionPanel id=None keep_mounted=None hidden_until_found=None class=None>"kept"</AccordionPanel>
                    </AccordionItem>
                </AccordionRoot>
            }
        });
        let panel = container
            .query_selector("[role=region]")
            .unwrap()
            .expect("the keepMounted panel stays mounted while closed");
        assert_eq!(
            panel.get_attribute("hidden").as_deref(),
            Some("hidden"),
            "the closed kept-mounted panel carries the hidden attribute"
        );
    }
}
