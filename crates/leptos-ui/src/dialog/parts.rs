//! The dialog parts — Trigger, Portal, Popup, Backdrop, Title, Description, Close,
//! Viewport (`packages/react/src/dialog/{trigger,portal,popup,backdrop,title,
//! description,close,viewport}/`), in the accordion/collapsible component style: leptos
//! views over the store context, with the store-provided interaction props converted
//! into the render-element vocabulary where the popup merges them.

use std::rc::Rc;

use reactive_graph::owner::provide_context;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use serde_json::json;
use web_sys::wasm_bindgen::JsCast;

use crate::dialog::{DialogRootContext, SharedDialogRootContext};
use leptos::prelude::*;
use leptos_ui_internals::composite::is_composite_key;
use leptos_ui_internals::floating_ui::popup_store::selectors;
use leptos_ui_internals::popup_store_utils::PopupStore;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_button::{ButtonExternalHandlers, UseButtonParams, use_button};

use reactive_graph::signal::RwSignal as RgRwSignal;

/// The state-mapping adapter over `dialogStateAttributesMapping`
/// (`packages/react/src/dialog/utils/stateAttributesMapping.ts:11-22`): the popup
/// open-state mapping plus the transition mapping plus `nestedDialogOpen`. The
/// accordion-style views emit the data attributes directly (the mapping vocabulary of
/// `use_render_element`'s state walk, spelled as view attributes per the accordion
/// precedent).
pub(crate) fn dialog_state_attributes(
    state: &serde_json::Map<String, serde_json::Value>,
) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    for (key, value) in state {
        if let Some(mapped) =
            leptos_ui_internals::popup_state_mapping::popup_state_mapping(key, value)
        {
            if let Some(props) = mapped {
                for (name, attr_value) in props {
                    attributes.push((name.to_string(), attr_value));
                }
            }
        } else if let Some(mapped) =
            leptos_ui_internals::popup_state_mapping::popup_transition_state_mapping(key, value)
        {
            if let Some(props) = mapped {
                for (name, attr_value) in props {
                    attributes.push((name.to_string(), attr_value));
                }
            }
        } else if key == "nestedDialogOpen" {
            if value.as_bool().unwrap_or(false) {
                attributes.push(("data-nested-dialog-open".to_string(), String::new()));
            }
        }
    }
    attributes
}

/// A counting helper the state maps use in tests.
#[cfg(test)]
pub(crate) fn state_map(
    pairs: &[(&str, serde_json::Value)],
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), value.clone());
    }
    map
}

// ---------------------------------------------------------------------------
// Trigger
// ---------------------------------------------------------------------------

/// The public `Dialog.Trigger` component (`DialogTrigger.tsx:22-102`). Must be called
/// inside a reactive owner; inside or outside a Root (detached via `handle` — the
/// detached path resolves the handle's store through the same composition).
#[leptos::component]
pub fn DialogTrigger(
    /// `disabled` (`:30` — upstream default `false`).
    #[prop(default = false)]
    disabled: bool,
    /// `nativeButton` (`:31` — upstream default `true`).
    #[prop(default = true)]
    native_button: bool,
    /// Trigger `id` (`:125`) — registered into the store and used for ARIA sync.
    #[prop(default = None)]
    id: Option<String>,
    /// Trigger `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: leptos::children::ChildrenFn,
) -> impl leptos::IntoView {
    // The store resolution (`:38-45`): the context store (the detached-handle path is
    // the port's deferred pass — the in-Root trigger is the tested surface).
    let context = DialogRootContext::expect();
    let store = Rc::clone(&context.store);

    // `useBaseUiId(idProp)` (`:47`).
    let id_signal = use_base_ui_id(RgRwSignal::new_local(id.clone()));
    let this_trigger_id = id_signal.get_untracked();

    // The store reads (`:48-50`).
    let is_opened_by_this_trigger = store.use_state({
        let this_trigger_id = this_trigger_id.clone();
        move |state| selectors::is_opened_by_trigger(state, Some(&this_trigger_id))
    });
    let popup_id_for_trigger = store.use_state({
        let this_trigger_id = this_trigger_id.clone();
        move |state| selectors::trigger_popup_id(state, Some(&this_trigger_id))
    });

    // `useTriggerDataForwarding` (`:54-61`): registers the trigger element and keeps
    // the active-trigger ownership fresh.
    let trigger_element_ref: Rc<std::cell::Cell<Option<web_sys::Element>>> =
        Rc::new(std::cell::Cell::new(None));
    let UseTriggerDataForwardingShim {
        register_trigger,
        is_mounted_by_this_trigger,
    } = register_trigger_shim(
        Some(this_trigger_id.clone()),
        Rc::clone(&trigger_element_ref),
        &store,
    );

    // `useButton({ disabled, native })` (`:63-66`).
    let button = use_button(UseButtonParams {
        disabled: RgRwSignal::new_local(disabled),
        focusable_when_disabled: None,
        tab_index: 0,
        native: native_button,
        composite: None,
    });
    let button_props = (button.get_button_props)(ButtonExternalHandlers::default());

    // The activation machine: the click opens/closes through the store writer with
    // `REASONS.triggerPress` (`useClick`'s `getNextOpen` decision, `:68` — the popup
    // was opened by this trigger or the popup is closed).
    let on_click = {
        let store = Rc::clone(&store);
        let is_opened_by_this_trigger = is_opened_by_this_trigger.clone();
        move |_event: web_sys::MouseEvent| {
            let next_open = !is_opened_by_this_trigger.get_untracked();
            let details = leptos_ui_internals::floating_ui::types::RootOpenChangeEventDetails::new(
                leptos_ui_internals::floating_ui::reasons::TRIGGER_PRESS.to_owned(),
                web_sys::Event::new("click").expect("the Event constructor is available"),
                None,
                String::new(),
            );
            if let Some(writer) = &store.context.on_open_change {
                writer(next_open, &details);
            }
        }
    };

    // The aria bag (`:90-97`): `aria-haspopup: 'dialog'`, `aria-expanded`,
    // `aria-controls`.
    let aria_expanded = move || is_opened_by_this_trigger.get().to_string();
    let aria_controls = move || popup_id_for_trigger.get().unwrap_or_default();
    // `triggerOpenStateMapping` (`:100`): `data-popup-open` when open.
    let data_popup_open = move || is_opened_by_this_trigger.get().then_some("true".to_owned());
    let data_disabled = move || disabled.then_some("true".to_owned());

    // The registration cleanup rides the element ref (the merged ref fires with None
    // on unmount). The `on_cleanup` contract requires `Send` — the `SendWrapper`
    // bridge (the `close_part.rs` convention).
    {
        let register = register_trigger.clone();
        let element_slot = Rc::clone(&trigger_element_ref);
        let cleanup = send_wrapper::SendWrapper::new(move || {
            register.call(None);
            element_slot.set(None);
        });
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }
    // Register once the element renders (the layout-effect analog: the first read of
    // the mounted flag runs after mount).
    {
        let register = register_trigger.clone();
        let element_slot = Rc::clone(&trigger_element_ref);
        leptos::prelude::Effect::new(move |_| {
            // Runs on mount and whenever the mounted-by-this-trigger flag changes;
            // re-registration is a no-op (the same triple).
            let element = element_slot.take();
            if element.is_some() {
                element_slot.set(element.clone());
            }
            register.call(element);
        });
    }

    let tabindex = button_props
        .attributes
        .iter()
        .find(|(k, _)| k == "tabindex")
        .and_then(|(_, resolve)| resolve());

    let this_trigger_id_for_view = this_trigger_id.clone();
    view! {
        <button
            type="button"
            class=class
            id=this_trigger_id_for_view
            disabled=move || disabled.then_some("true")
            tabindex=tabindex
            aria-haspopup="dialog"
            aria-expanded=aria_expanded
            aria-controls=aria_controls
            data-popup-open=data_popup_open
            data-disabled=data_disabled
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// The trigger-registration shim the port's parts use — the
/// `useTriggerDataForwarding` composition reduced to the registration half (the
/// active-claim half runs inside the shared hook; the in-Root trigger's activation
/// rides the store writer directly).
fn register_trigger_shim(
    trigger_id: Option<String>,
    element_slot: Rc<std::cell::Cell<Option<web_sys::Element>>>,
    store: &PopupStore<()>,
) -> UseTriggerDataForwardingShim {
    let register =
        leptos_ui_internals::popup_store_utils::use_trigger_registration(trigger_id.clone(), store);
    let is_mounted_by_this_trigger = store.use_state({
        let trigger_id = trigger_id.clone();
        move |state| selectors::is_mounted_by_trigger(state, trigger_id.as_deref())
    });
    UseTriggerDataForwardingShim {
        register_trigger: register,
        is_mounted_by_this_trigger,
    }
}

struct UseTriggerDataForwardingShim {
    register_trigger:
        leptos_ui_utils::use_stable_callback::StableCallback<Option<web_sys::Element>, ()>,
    is_mounted_by_this_trigger: RgRwSignal<bool, reactive_graph::owner::LocalStorage>,
}
