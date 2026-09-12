//! Port of the Base UI Accordion — the `library: accordion` TODO item
//! (`specs/library/accordion/behavior.md`, `specs/library/accordion/implementation.md`).
//!
//! Upstream's structural fact (implementation.md, opening paragraph): **Accordion is a
//! thin composition layer over the internal collapsible machinery.** Accordion owns the
//! state — a `Value[]` array on the root — and each item instantiates the collapsible
//! layer in a permanently controlled mode. This port composes over the same reactive
//! state machine the crate's collapsible module established (signals + contexts), with
//! the root's array algebra ported verbatim:
//!
//! - `handleValueChange` (`packages/react/src/accordion/root/AccordionRoot.tsx:67-97`):
//!   non-`multiple` toggles by value identity — `[]` if `value[0] === newValue`, else
//!   `[newValue]` (`:73-79`); `multiple` + open appends to a copy, `multiple` + close
//!   filters out (`:80-95`); the user's `onValueChange` receives the *attempted* next
//!   value **before** any state write and the write is skipped when
//!   `details.isCanceled` (`:75-79`, `:83-87`, `:90-94`) — the cancel protocol ordering
//!   behavior.md's "Events" section documents.
//! - Item `onOpenChange` wrapper chains two cancel checks: the item-level callback
//!   first, return if cancelled, then the root's `handleValueChange`
//!   (`packages/react/src/accordion/item/AccordionItem.tsx:60-70`) — so an item-level
//!   cancel blocks even `onValueChange` (implementation.md, "Item: derived open state").
//! - `isOpen` is the pure membership derivation
//!   (`packages/react/src/accordion/item/AccordionItem.tsx:58`); a missing item `value`
//!   falls back to a generated `useBaseUiId` id (`:52-54`).
//! - The item drives the collapsible layer with `open: isOpen` **as a controlled prop**
//!   (`:72-76`): every mutation is routed event → item wrapper → root algebra, never
//!   through the collapsible layer's own setter.
//! - ARIA wiring is id-registry based (implementation.md, "DOM/portal strategy"):
//!   trigger `aria-controls` → resolved panel id **only while open** +
//!   `aria-expanded` always (`AccordionTrigger.tsx:54-59`); panel `role="region"` +
//!   `aria-labelledby` → resolved trigger id (`AccordionPanel.tsx:116-118`); manual ids
//!   are honored verbatim, missing ids fall back to generated `useBaseUiId` ids
//!   (`AccordionItem.tsx:52-54,107-111`).
//! - `useButton({ disabled, focusableWhenDisabled: true, native })`
//!   (`AccordionTrigger.tsx:37-41`) supplies the button semantics: the port composes
//!   `use_button` with `focusable_when_disabled: Some(true)` so a disabled trigger stays
//!   tabbable (`tabindex="0"`, behavior.md "Focus management"), and Space/Enter on a
//!   non-native trigger dispatch the synthetic click on **keyup**/keydown respectively
//!   (`useButton.ts:154-217`) — on a native `<button>` the browser supplies both.
//! - Disabled resolution `disabledProp || contextDisabled`
//!   (`AccordionTrigger.tsx:35`): a root- or item-disabled accordion beats a trigger's
//!   own `disabled={false}` (behavior.md "Edge cases").
//! - State reaches the DOM as data attributes, not rendered props (implementation.md
//!   "DOM/portal strategy"): `data-open`/`data-closed` via the shared
//!   `collapsibleOpenStateMapping` on the panel, `data-panel-open` via
//!   `triggerOpenStateMapping` on the trigger, `data-disabled` when disabled, and the
//!   item's `data-index`/`data-orientation` (`stateAttributesMapping.ts:7-12`).
//! - The deprecated `orientation`/`loopFocus` props are accepted no-ops beyond state
//!   (`AccordionRoot.tsx:196-223`); `hiddenUntilFound`/`keepMounted`/
//!   `transitionStatus`/measurement are the collapsible layer's machinery — the port's
//!   panel renders mounted-closed panels with the `hidden` attribute
//!   (`AccordionPanel.test.tsx:97-110`) and defers transition measurement to the
//!   collapsible layer's reactive status.
//!
//! ## Rust adaptations
//!
//! - The controlled `value: Option<Vec<String>>` + `default_value: Vec<String>` split is
//!   upstream's `useControlled` tri-state (packages/utils/src/useControlled.ts:82-91):
//!   controlled mode never writes internal state; uncontrolled mode owns the array.
//! - Item values are `String` (upstream `Value = any`; behavior.md's "State model"
//!   records values are matched by identity — strings carry that contract; custom
//!   non-string values are the `Value` typing gap implementation.md §6 already flags as
//!   runtime-untested upstream).
//! - The pure algebra [`accordion_next_value`] is extracted verbatim from
//!   `handleValueChange` so the array contracts are host-testable without a DOM.

use std::sync::Arc;

#[cfg(test)]
use serde_json::json;

use leptos::prelude::*;
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::floating_ui::reasons;
use leptos_ui_utils::warn::warn;

// The internals crate's hooks (`use_button`) are typed over reactive_graph 0.2
// signals, while leptos 0.7's prelude signals are the same crate at 0.1.x — the
// two worlds coexist per the toggle/collapsible precedent: the aliased rg-0.2
// types feed the internals calls, the prelude types feed views and context.
use reactive_graph::signal::RwSignal as RgRwSignal;

use web_sys::KeyboardEvent;

/// The upstream change-details type
/// (`packages/react/src/accordion/root/AccordionRoot.tsx:226-229` —
/// `createChangeEventDetails(reason, nativeEvent)`): exposes `cancel()` /
/// `isCanceled()` to the consumer.
pub type AccordionChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::Event>;

/// `onValueChange` (`AccordionRoot.tsx:64`): `(nextValue, eventDetails)`.
pub type OnValueChange = Arc<dyn Fn(&[String], &AccordionChangeEventDetails) + Send + Sync>;
/// Item-level `onOpenChange` (`AccordionItem.tsx:64`): `(nextOpen, eventDetails)`.
pub type OnOpenChange = Arc<dyn Fn(bool, &AccordionChangeEventDetails) + Send + Sync>;

/// The root's array algebra, extracted verbatim from `handleValueChange`
/// (`packages/react/src/accordion/root/AccordionRoot.tsx:73-95`): non-`multiple`
/// toggles by identity against `value[0]` (ignoring `nextOpen` — implementation.md
/// gap §2), `multiple` appends on open and filters on close.
pub fn accordion_next_value(
    value: &[String],
    new_value: &str,
    next_open: bool,
    multiple: bool,
) -> Vec<String> {
    if !multiple {
        if value.first().map(|v| v.as_str()) == Some(new_value) {
            Vec::new()
        } else {
            vec![new_value.to_string()]
        }
    } else if next_open {
        let mut next: Vec<String> = value.to_vec();
        next.push(new_value.to_string());
        next
    } else {
        value
            .iter()
            .filter(|v| v.as_str() != new_value)
            .cloned()
            .collect()
    }
}

/// `AccordionRootContext` (`packages/react/src/accordion/root/AccordionRootContext.ts:5-16`):
/// the root→item/panel context. `handle_value_change` is the root's algebra commit
/// (`:67-97`); `value` mirrors the root's current open array.
#[derive(Clone)]
pub struct AccordionRootContext {
    /// `disabled` (`:7`).
    pub disabled: bool,
    /// `handleValueChange` (`:8`) — `(newValue, nextOpen, details)`.
    pub handle_value_change: Arc<dyn Fn(&str, bool, &AccordionChangeEventDetails) + Send + Sync>,
    /// `hiddenUntilFound` default (`:9`).
    pub hidden_until_found: bool,
    /// `keepMounted` default (`:10`).
    pub keep_mounted: bool,
    /// `value` (`:12`) — the live open-values read.
    pub value: Signal<Vec<String>>,
}

/// `AccordionItemContext` (`packages/react/src/accordion/item/AccordionItemContext.ts:5-11`):
/// the item→header/trigger/panel context — the item state plus the trigger-id registry
/// (`defaultTriggerId`, `triggerId`, `setTriggerId`, `:8-10`).
#[derive(Clone)]
pub struct AccordionItemContext {
    /// `defaultTriggerId` (`:6`).
    pub default_trigger_id: String,
    /// `triggerId` (`:9`) — resolved: registered manual id, else the generated
    /// fallback, else `None` once the trigger unmounts (`AccordionItem.tsx:107-111`).
    pub trigger_id: RwSignal<Option<String>>,
}

/// The root commit machine — `handleValueChange`'s tail (`:75-94`): call the user's
/// `onValueChange` with the attempted value **first**, skip the state write when
/// cancelled. The controlled mode's write-suppression is [`use_controlled`]'s setter
/// contract (packages/utils/src/useControlled.ts:82-91).
fn root_commit(
    set_value: &impl Fn(Vec<String>),
    on_value_change: Option<&OnValueChange>,
    next_value: Vec<String>,
    details: &AccordionChangeEventDetails,
) {
    if let Some(callback) = on_value_change {
        callback(&next_value, details);
    }
    if details.is_canceled() {
        return;
    }
    set_value(next_value);
}

/// The public `Accordion.Root` component — upstream's `AccordionRoot` body
/// (`packages/react/src/accordion/root/AccordionRoot.tsx:22-133`) with the root
/// context provided (`:108-118`). Must be called inside a reactive owner.
#[component]
pub fn AccordionRoot(
    /// Controlled open item values; `None` while uncontrolled
    /// (`:31`, upstream `value`).
    #[prop(default = None)]
    value: Option<Vec<String>>,
    /// Uncontrolled initial open items (`:30` — upstream default the shared frozen
    /// `EMPTY_ARRAY`, `:44`).
    #[prop(default = Vec::new())]
    default_value: Vec<String>,
    /// `onValueChange(nextValue, eventDetails)` (`:32`).
    #[prop(default = None)]
    on_value_change: Option<OnValueChange>,
    /// Allow multiple open items (`:33` — upstream default `false`).
    #[prop(default = false)]
    multiple: bool,
    /// Disable every item (`:34`).
    #[prop(default = false)]
    disabled: bool,
    /// Render closed panels with `hidden="until-found"` (`:36`).
    #[prop(default = false)]
    hidden_until_found: bool,
    /// Keep closed panels mounted (`:37`).
    #[prop(default = false)]
    keep_mounted: bool,
    /// Deprecated orientation passthrough (`:38`, `:196-202`).
    #[prop(default = String::new())]
    orientation: String,
    /// Root `id` passthrough.
    #[prop(default = None)]
    id: Option<String>,
    /// Root `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: Children,
) -> impl IntoView {
    // The dev-only keepMounted conflict warning (`:46-56`).
    if cfg!(debug_assertions) && hidden_until_found && !keep_mounted {
        warn().log(&[
            "The `keepMounted={false}` prop on `Accordion.Root` is ignored when `hiddenUntilFound` is enabled, since panels must remain mounted while closed.",
        ]);
    }

    // `useControlled({ controlled: valueProp, default: defaultValue ?? EMPTY_ARRAY })`
    // (`:59-65`): the tri-state. Rust adaptation: the mode is fixed from the
    // initial prop's defined-ness (packages/utils/src/useControlled.ts:41), so the
    // controlled branch snapshots and mirrors the prop while suppressing internal
    // writes; the uncontrolled branch owns the canonical array. Both live on
    // SyncStorage signals (the context's Send+Sync bound — Rc/LocalStorage are
    // out).
    let is_controlled = value.is_some();
    let controlled_value = value.clone();
    let value_signal = RwSignal::new(value.unwrap_or_else(|| default_value.clone()));

    // The exposed read (`useControlled.ts:45` — the controlled value wins while
    // controlled, else the internal state).
    let value = Signal::derive(move || {
        if is_controlled {
            match controlled_value.as_ref() {
                Some(v) => v.clone(),
                None => value_signal.get(),
            }
        } else {
            value_signal.get()
        }
    });

    // The guarded setter (`:82-91`): a no-op while controlled, a real write while
    // uncontrolled.
    let set_value = {
        let value_signal = value_signal.clone();
        move |next_value: Vec<String>| {
            if !is_controlled {
                value_signal.set(next_value);
            }
        }
    };

    // `handleValueChange` (`:67-97`) — the algebra plus the cancel-protocol ordering.
    let on_for_commit = on_value_change.clone();
    let handle_value_change: Arc<dyn Fn(&str, bool, &AccordionChangeEventDetails) + Send + Sync> =
        Arc::new(
            move |new_value: &str, next_open: bool, details: &AccordionChangeEventDetails| {
                let current = value.get_untracked();
                let next_value = accordion_next_value(&current, new_value, next_open, multiple);
                root_commit(&set_value, on_for_commit.as_ref(), next_value, details);
            },
        );

    provide_context(AccordionRootContext {
        disabled,
        handle_value_change,
        hidden_until_found,
        keep_mounted,
        value,
    });

    // `useRenderElement('div', …, { state })` (`:120-125`): the root `div` with the
    // state → data-attribute mapping (rootStateAttributesMapping suppresses `value`
    // and exposes `disabled`/`orientation`, `:14-16`).
    view! {
        <div
            id={id}
            class={class}
            data-disabled={move || disabled.then_some("true")}
            data-orientation={(!orientation.is_empty()).then(|| orientation.clone())}
        >
            {children()}
        </div>
    }
}

/// The public `Accordion.Item` component (`AccordionItem.tsx` body): derives `isOpen`
/// from the root context (`:58`), wraps `onOpenChange` with the two-layer cancel
/// protocol (`:60-70`), drives the collapsible layer as a permanently controlled
/// consumer (`:72-76`), and provides the item context (`:113-122`).
#[component]
pub fn AccordionItem(
    /// The item's identity value; falls back to a generated id (`:52-54`).
    #[prop(default = None)]
    value: Option<String>,
    /// Disable this item (`:31`).
    #[prop(default = false)]
    disabled: bool,
    /// Item-level open callback with cancel semantics (`:33`).
    #[prop(default = None)]
    on_open_change: Option<OnOpenChange>,
    /// Item `id` passthrough.
    #[prop(default = None)]
    id: Option<String>,
    /// Item `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: Children,
) -> impl IntoView {
    let root = use_context::<AccordionRootContext>().expect(
        "AccordionRootContext is missing. Accordion parts must be placed within <Accordion.Root>.",
    );

    // `const fallbackValue = useBaseUiId()` + `value = valueProp ?? fallbackValue`
    // (`:52-54`).
    let item_value = value.unwrap_or_else(new_base_ui_id);

    // `disabled = disabledProp || contextDisabled` (`:66`).
    let item_disabled = disabled || root.disabled;

    // `isOpen = openValues.indexOf(value) !== -1` (`:58`) — the pure derivation,
    // reactive over the root array.
    let root_value_for_derive = root.value;
    let item_value_for_derive = item_value.clone();
    let is_open = Signal::derive(move || {
        root_value_for_derive
            .get()
            .iter()
            .any(|v| v == &item_value_for_derive)
    });

    // The wrapped `onOpenChange` (`:60-70`): item callback first, return if it
    // cancelled, then the root's algebra.
    let item_callback = on_open_change.clone();
    let handle_value_change = root.handle_value_change.clone();
    let wrapped_on_open_change: Arc<dyn Fn(bool, &AccordionChangeEventDetails) + Send + Sync> = {
        let item_value = item_value.clone();
        Arc::new(
            move |next_open: bool, details: &AccordionChangeEventDetails| {
                if let Some(callback) = &item_callback {
                    callback(next_open, details);
                }
                if details.is_canceled() {
                    return;
                }
                handle_value_change(&item_value, next_open, details);
            },
        )
    };

    // `useCollapsibleRoot({ open: isOpen, onOpenChange, disabled })` (`:72-76`): the
    // collapsible layer is permanently controlled — every mutation routes through the
    // wrapper above; the root array is the single source of truth, so `mounted`
    // (`useCollapsibleRoot.ts:23`) derives from the same `isOpen`.
    let mounted = is_open;
    let transition_status = RwSignal::new(None);
    let panel_id = RwSignal::new(None::<String>);
    // `defaultPanelId = useBaseUiId()` (`useCollapsibleRoot.ts:25`): one
    // generated id per item, seeded once at item setup — not per render.
    let default_panel_id = new_base_ui_id();

    // The item state (`:96-105`): root state + `hidden: !isOpen && !mounted` + index
    // + disabled + open. The composite-list index is the vestigial roving-focus
    // machinery (implementation.md gap §1) — surfaced as `data-index` only.
    let item_state = AccordionItemState {
        open: is_open,
        disabled: RwSignal::new(item_disabled),
        mounted,
        transition_status,
        panel_id,
        default_panel_id,
        on_open_change: wrapped_on_open_change,
        root_handle_value_change: root.handle_value_change.clone(),
    };

    // The trigger-id registry (`:107-111`): `None` keeps the generated fallback,
    // `Some(None)` (upstream `null`) means the trigger unmounted.
    let default_trigger_id = new_base_ui_id();
    let trigger_id = RwSignal::new(Some(default_trigger_id.clone()));

    provide_context(AccordionItemContext {
        default_trigger_id,
        trigger_id,
    });
    provide_context(item_state);

    // `useRenderElement('div', …, { state, stateAttributesMapping:
    // accordionStateAttributesMapping })` (`:124-129`): `data-open`/`data-closed`,
    // `data-disabled`, `data-index`, `data-orientation` (stateAttributesMapping.ts:7-12).
    let index = next_item_index();
    view! {
        <div
            id={id}
            class={class}
            data-open={move || is_open.get().then_some("true")}
            data-closed={move || (!is_open.get()).then_some("true")}
            data-disabled={move || item_disabled.then_some("true")}
            data-index={index.to_string()}
            data-orientation="horizontal"
        >
            {children()}
        </div>
    }
}

/// The item state the item provides to trigger/panel — the `CollapsibleRootContext`
/// seam (`AccordionItem.tsx:87-94,132`): the accordion's Trigger/Panel consume the
/// same shape the standalone Collapsible provides.
#[derive(Clone)]
pub struct AccordionItemState {
    /// The derived `open` (`:72-76` controlled prop) — reactive over the root array.
    pub open: Signal<bool>,
    /// The resolved item `disabled`.
    pub disabled: RwSignal<bool>,
    /// The collapsible layer's `mounted` (`useCollapsibleRoot.ts:23`) — derives from
    /// the same `isOpen` under the permanently-controlled contract.
    pub mounted: Signal<bool>,
    /// The collapsible layer's `transitionStatus`.
    pub transition_status:
        RwSignal<Option<leptos_ui_internals::use_transition_status::TransitionStatus>>,
    /// The collapsible layer's panel-id registry (`useCollapsibleRoot.ts:25-28`).
    pub panel_id: RwSignal<Option<String>>,
    /// `defaultPanelId` (`useCollapsibleRoot.ts:25`) — one generated `base-ui-`
    /// id per item, the panel's resolved id when no manual id is registered.
    pub default_panel_id: String,
    /// The wrapped `onOpenChange` (`:60-70`) — trigger activations land here.
    pub on_open_change: Arc<dyn Fn(bool, &AccordionChangeEventDetails) + Send + Sync>,
    /// The root's `handleValueChange` (`:67-97`) — the commit the wrapper delegates to.
    pub root_handle_value_change:
        Arc<dyn Fn(&str, bool, &AccordionChangeEventDetails) + Send + Sync>,
}

/// Generated fallback ids — `useBaseUiId` (`useBaseUiId.ts:9-11`, `base-ui-` prefix),
/// threaded through the same generator the internals crate uses so SSR/hydration
/// stability matches the rest of the port.
fn new_base_ui_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("base-ui-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// The per-item document-position index (`useCompositeListItem`'s registration order,
/// `useCompositeListItem.ts:62-82`) — observable only through `data-index`
/// (implementation.md gap §1).
fn next_item_index() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static INDEX: AtomicUsize = AtomicUsize::new(0);
    INDEX.fetch_add(1, Ordering::Relaxed)
}

/// The public `Accordion.Header` component — the stateless `h3` passthrough
/// (`packages/react/src/accordion/header/AccordionHeader.tsx:21-28`): reads the item
/// state and renders the heading with the shared mapping; contributes no behavior.
#[component]
pub fn AccordionHeader(
    /// Header `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: Children,
) -> impl IntoView {
    let _ = use_context::<AccordionItemState>();
    view! {
        <h3 class=class>{children()}</h3>
    }
}

/// The public `Accordion.Trigger` component (`AccordionTrigger.tsx` body): consumes the
/// collapsible seam, resolves `disabled = disabledProp || contextDisabled` (`:35`),
/// composes `useButton({ disabled, focusableWhenDisabled: true, native })` (`:37-41`),
/// and renders `aria-controls` (only while open) + `aria-expanded` + `id` (`:54-59`).
#[component]
pub fn AccordionTrigger(
    /// Manual trigger `id` (`:28`) — registered into the item's id registry (`:47-52`).
    #[prop(default = None)]
    id: Option<String>,
    /// Trigger-level `disabled` (`:29`) — loses to a root/item disable (`:35`).
    #[prop(default = false)]
    disabled: bool,
    /// `nativeButton` (`:32`) — `false` renders the `role="button"` + synthetic
    /// Space/Enter activation path.
    #[prop(default = true)]
    native_button: bool,
    /// Trigger `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    children: Children,
) -> impl IntoView {
    let item = use_context::<AccordionItemState>().expect(
        "AccordionItemState is missing. Accordion.Trigger must be placed within an Accordion.Item.",
    );
    let item_ctx = use_context::<AccordionItemContext>()
        .expect("AccordionItemContext is missing. Accordion.Trigger must be placed within an Accordion.Item.");

    // `disabled = disabledProp || contextDisabled` (`:35`): the root/item disable
    // wins (behavior.md "Edge cases").
    let resolved_disabled = disabled || item.disabled.get_untracked();

    // The manual-id registration effect (`:47-52`): register `idProp` (empty string
    // treated as absent), keep the generated fallback when absent, mark `null`
    // (registry `None`) on unmount.
    let registered_id = id.filter(|v| !v.is_empty());
    {
        let trigger_id = item_ctx.trigger_id;
        if let Some(registered) = registered_id.clone() {
            trigger_id.set(Some(registered));
        }
        on_cleanup({
            let trigger_id = trigger_id.clone();
            move || trigger_id.set(None)
        });
    }

    // `useButton({ disabled, focusableWhenDisabled: true, native: nativeButton })`
    // (`:37-41`): focusable-when-disabled keeps the trigger tabbable
    // (`useButton.ts:28-34`).
    let button = leptos_ui_internals::use_button::use_button(
        leptos_ui_internals::use_button::UseButtonParams {
            disabled: RgRwSignal::new(resolved_disabled),
            focusable_when_disabled: Some(true),
            tab_index: 0,
            native: native_button,
            composite: None,
        },
    );
    let button_props = (button.get_button_props)(
        leptos_ui_internals::use_button::ButtonExternalHandlers::default(),
    );

    // The activation machine (`:54-59`): `onClick: handleTrigger` — the collapsible
    // seam's wrapped `onOpenChange` with the triggerPress reason stamped
    // (`useCollapsibleRoot.ts:30-41`).
    let on_click = {
        let item = item.clone();
        move |_event: &web_sys::MouseEvent| {
            if resolved_disabled {
                return;
            }
            let next_open = !item.open.get_untracked();
            let details = AccordionChangeEventDetails::new(
                reasons::TRIGGER_PRESS,
                web_sys::Event::new("click").unwrap(),
                None,
                (),
            );
            (item.on_open_change)(next_open, &details);
            // The commit itself rides the wrapper → root algebra (`:60-70` →
            // `:67-97`); the permanently-controlled collapsible layer follows the
            // root array, so there is no second write here (`:72-76`).
        }
    };

    // Non-native synthetic activation (`useButton.ts:154-217`): Enter dispatches the
    // click on keydown, Space on keyup. A native `<button>` gets both from the
    // browser, so the synthetic path is armed only when `nativeButton={false}`.
    let on_key_down = {
        let on_click = on_click.clone();
        move |event: KeyboardEvent| {
            if native_button || resolved_disabled {
                return;
            }
            if event.key() == "Enter" {
                on_click(&web_sys::MouseEvent::new("click").unwrap());
            }
        }
    };
    let on_key_up = {
        let on_click = on_click.clone();
        move |event: KeyboardEvent| {
            if native_button || resolved_disabled {
                return;
            }
            if event.key() == " " || event.key() == "Spacebar" {
                on_click(&web_sys::MouseEvent::new("click").unwrap());
            }
        }
    };

    // `triggerOpenStateMapping` (`:61-66`): `data-panel-open` when open
    // (collapsibleOpenStateMapping.ts:13-24). `aria-controls = open ? panelId :
    // undefined` (`AccordionTrigger.tsx:55`, panelId from the collapsible root
    // context `:33` — `registeredPanelId === null ? undefined :
    // (registeredPanelId ?? defaultPanelId)`, `useCollapsibleRoot.ts:28`).
    // `aria-expanded` always (`:56`).
    let panel_id_signal = item.panel_id.clone();
    let default_panel_id = item.default_panel_id.clone();
    let aria_controls = move || {
        if !item.open.get() {
            return None;
        }
        // The port wires no unmount-`null` registry write, so `None` here is the
        // unset registry — the `?? defaultPanelId` arm supplies the generated id.
        Some(
            panel_id_signal
                .get()
                .unwrap_or_else(|| default_panel_id.clone()),
        )
    };

    // The focusable-when-disabled tabindex (`useButton.ts:28-34` via
    // `use_focusable_when_disabled`) — resolved once, rendered statically.
    let tabindex = button_props
        .attributes
        .iter()
        .find(|(k, _)| k == "tabindex")
        .map(|(_, v)| v.clone())
        .and_then(|resolve| resolve());

    view! {
        <button
            type="button"
            class={class}
            id={registered_id.clone().or(Some(item_ctx.default_trigger_id.clone()))}
            disabled={resolved_disabled.then_some("true")}
            tabindex={tabindex}
            aria-expanded={move || item.open.get().to_string()}
            aria-controls={aria_controls}
            data-panel-open={move || item.open.get().then_some("true")}
            on:click=move |ev: web_sys::MouseEvent| { on_click(&ev); }
            on:keydown=on_key_down
            on:keyup=on_key_up
        >
            {children()}
        </button>
    }
}

/// The public `Accordion.Panel` component (`AccordionPanel.tsx` body): registers the
/// manual panel id into the collapsible layer's registry (`:69-74`), renders
/// `role="region"` + `aria-labelledby` → the resolved trigger id (`:116-118`), and
/// applies the mount/unmount gate — a closed non-`keepMounted` panel is absent from
/// the DOM (`useCollapsiblePanel.ts:73`; behavior.md "DOM structure").
#[component]
pub fn AccordionPanel(
    /// Manual panel `id` (`:34`) — registered into the collapsible layer's registry
    /// (`:69-74`), referenced by the trigger's `aria-controls`.
    #[prop(default = None)]
    id: Option<String>,
    /// Panel-level `keepMounted` override (`:35`); `None` inherits the root.
    #[prop(default = None)]
    keep_mounted: Option<bool>,
    /// Panel-level `hiddenUntilFound` override (`:36`); `None` inherits the root.
    #[prop(default = None)]
    hidden_until_found: Option<bool>,
    /// Panel `className` passthrough.
    #[prop(default = None)]
    class: Option<String>,
    /// Panel `children` — `ChildrenFn` so the body re-enters the `Show` branch on
    /// each open/mount transition (the boxed `FnOnce` form cannot live inside a
    /// reactive branch closure).
    children: ChildrenFn,
) -> impl IntoView {
    let root = use_context::<AccordionRootContext>().expect(
        "AccordionRootContext is missing. Accordion parts must be placed within <Accordion.Root>.",
    );
    let item = use_context::<AccordionItemState>().expect(
        "AccordionItemState is missing. Accordion.Panel must be placed within an Accordion.Item.",
    );
    let item_ctx = use_context::<AccordionItemContext>().expect(
        "AccordionItemContext is missing. Accordion.Panel must be placed within an Accordion.Item.",
    );

    // `hiddenUntilFound`/`keepMounted` default to the root, panel overrides
    // (`:38-39`); the dev conflict warning mirrors the root's (`:57-67`).
    let hidden_until_found = hidden_until_found.unwrap_or(root.hidden_until_found);
    let keep_mounted = keep_mounted.unwrap_or(root.keep_mounted);
    if cfg!(debug_assertions) && !keep_mounted && hidden_until_found {
        warn().log(&[
            "The `keepMounted={false}` prop on an `Accordion.Panel` is ignored when `hiddenUntilFound` is enabled on the panel or root, since the panel must remain mounted while closed.",
        ]);
    }

    // The panel-id registry effect (`:69-74`): `setPanelIdState((currentId) =>
    // registeredId ?? (currentId === null ? undefined : currentId))` — a manual id
    // registers verbatim; with no manual id the CURRENT registry value is
    // preserved. The port wires no unmount-`null` writer (`:72`), so an unset
    // registry stays unset and the `?? defaultPanelId` resolution arm
    // (`useCollapsibleRoot.ts:28`) supplies the item's generated id — the same
    // effective id upstream resolves without a manual id.
    let registered_panel_id = id.clone().filter(|v| !v.is_empty());
    if let Some(registered) = registered_panel_id.clone() {
        item.panel_id.set(Some(registered));
    }
    // `const id = idProp ?? defaultPanelId` (`:55`): the rendered element id is
    // ALWAYS present — the manual id or the item's generated default.
    let resolved_panel_id = registered_panel_id
        .clone()
        .unwrap_or_else(|| item.default_panel_id.clone());

    // `hidden = !open && !mounted` and the `shouldRender` gate
    // (`useCollapsiblePanel.ts:73`; `AccordionPanel.tsx:136-140`): a closed
    // non-kept-mounted panel unmounts; `hiddenUntilFound` forces it to stay mounted
    // with `hidden="until-found"`.
    let should_render =
        move || keep_mounted || hidden_until_found || item.open.get() || item.mounted.get();
    let hidden_attr = move || {
        if item.open.get() || item.mounted.get() {
            None
        } else if hidden_until_found {
            Some("until-found".to_string())
        } else {
            Some("hidden".to_string())
        }
    };

    let trigger_id = item_ctx.trigger_id;

    view! {
        <Show
            when=should_render
            fallback=|| ()
        >
            <div
                class={class.clone()}
                id={Some(resolved_panel_id.clone())}
                role="region"
                aria-labelledby={move || trigger_id.get()}
                hidden={hidden_attr}
                data-open={move || item.open.get().then_some("true")}
                data-closed={move || (!item.open.get()).then_some("true")}
                data-disabled={move || item.disabled.get().then_some("true")}
            >
                {children()}
            </div>
        </Show>
    }
}

// The JSON state maps the state→data-attribute mapping consumes — kept testable for
// the host suite (the mapping vocabulary of `use_render_element`'s state walk).
#[cfg(test)]
pub(crate) fn accordion_item_state_map(
    open: bool,
    disabled: bool,
    index: usize,
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("open".to_string(), json!(open));
    map.insert("disabled".to_string(), json!(disabled));
    map.insert("index".to_string(), json!(index));
    map
}
