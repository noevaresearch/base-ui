//! Port of `packages/react/src/floating-ui-react/utils/element.ts` — the DOM-element
//! classification helpers the interaction hooks and FocusManager share
//! (`specs/library/floating-ui-react/implementation.md`, "Dependencies on other Base UI
//! internals": `utils/element.ts`).
//!
//! ## Rust adaptations
//!
//! - The re-export line (`element.ts:3-8`) — `activeElement`/`contains`/`getTarget` from
//!   `@base-ui/utils/shadowDom` — ports to [`leptos_ui_utils::shadow_dom`]'s ported
//!   functions, re-exported here for call-site parity.
//! - `isTargetInsideEnabledTrigger` reads `TooltipTriggerDataAttributes.triggerDisabled`
//!   (`element.ts:6,10-31`) — the cross-component reach-in the implementation spec
//!   documents (`specs/library/floating-ui-react/implementation.md`, "Cross-component
//!   reach-in"). The attribute name is defined here as
//!   [`TOOLTIP_TRIGGER_DISABLED`] until the tooltip component unit (Phase B) ports its
//!   data-attributes module; the string is upstream's.
//! - The `PopupTriggerMap` (`element.ts:5,12`) is part of the `infra: utils` unit
//!   (not yet ported). The trigger-membership checks
//!   ([`is_target_inside_enabled_trigger`]) take the two operations the unit performs on
//!   it — `hasElement` and iteration over `(id, trigger)` pairs — through a minimal
//!   trait [`PopupTriggerLookup`], which the popups port will implement. This keeps the
//!   helper's behavior (including the "contains" walk, `:24-28`) independent of the
//!   registry's representation.
//! - `isVirtualPointerEvent`'s platform branches live in [`crate::floating_ui::event`].

use leptos_ui_utils::platform;
use web_sys::wasm_bindgen::JsValue;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, EventTarget, HtmlElement, Node};

use crate::floating_ui::constants::{FOCUSABLE_ATTRIBUTE, TYPEABLE_SELECTOR};

pub use leptos_ui_utils::shadow_dom::{active_element, contains, get_target};

/// The `TooltipTriggerDataAttributes.triggerDisabled` attribute name
/// (`element.ts:21,26` — read, never written, by this unit): a trigger carrying it is
/// disabled, so hover/focus on it must not keep the popup open. Defined here until the
/// tooltip unit ports its data-attributes module (see the module docs).
pub const TOOLTIP_TRIGGER_DISABLED: &str = "data-trigger-disabled";

/// The trigger-registry surface `is_target_inside_enabled_trigger` needs — upstream's
/// `PopupTriggerMap.hasElement`/`entries` calls (`element.ts:20,24`).
pub trait PopupTriggerLookup {
    /// Whether the map holds the element as a trigger itself (`element.ts:20`).
    fn has_element(&self, element: &Element) -> bool;
    /// Iterate `(id, trigger)` pairs for the contains-walk (`element.ts:24-28`).
    fn for_each_trigger(&self, visit: &mut dyn FnMut(Option<&str>, &Element));
}

/// `isTargetInsideEnabledTrigger(target, triggerElements)` (`element.ts:10-31`): whether
/// the target is a registered trigger (or inside one) that is not disabled. Used by the
/// hover hooks to keep a popup open while the pointer is over another enabled trigger.
pub fn is_target_inside_enabled_trigger(
    target: Option<&EventTarget>,
    trigger_elements: &dyn PopupTriggerLookup,
) -> bool {
    let Some(target_element) = target.and_then(|target| target.dyn_ref::<Element>()) else {
        return false;
    };

    if trigger_elements.has_element(target_element) {
        return !target_element.has_attribute(TOOLTIP_TRIGGER_DISABLED);
    }

    let mut found: Option<bool> = None;
    trigger_elements.for_each_trigger(&mut |_id, trigger| {
        if found.is_none() && contains(Some(trigger), Some(target_element)) {
            found = Some(!trigger.has_attribute(TOOLTIP_TRIGGER_DISABLED));
        }
    });

    found.unwrap_or(false)
}

/// `isEventTargetWithin(event, node)` (`element.ts:33-45`): whether the event's
/// composed path includes the node, falling back to a target-contains check where
/// `composedPath` is unavailable (`:42-44` — the unreachable-by-design branch the
/// implementation spec flags; every engine the port targets exposes `composedPath`).
pub fn is_event_target_within(event: &Event, node: Option<&Node>) -> bool {
    let Some(node) = node else {
        return false;
    };

    let node_value: &JsValue = node.as_ref();
    let path = event.composed_path();
    if path.length() > 0 {
        // `JsValue: PartialEq` is JS `===` — the same identity comparison
        // `composedPath().includes(node)` performs (`element.ts:39`).
        return path.iter().any(|item| item == *node_value);
    }

    // Browsers without shadow DOM (`element.ts:42-44`).
    event
        .target()
        .and_then(|target| target.dyn_into::<Node>().ok())
        .map(|target| node.contains(Some(&target)))
        .unwrap_or(false)
}

/// `isRootElement(element)` (`element.ts:47-49`): the scroll-container roots the dismiss
/// scrollbar hit-test targets (`hooks/useDismiss.ts:443-475`).
pub fn is_root_element(element: &Element) -> bool {
    element.matches("html,body").unwrap_or(false)
}

/// `isTypeableElement(element)` (`element.ts:51-53`): matches [`TYPEABLE_SELECTOR`].
pub fn is_typeable_element(element: &Element) -> bool {
    element
        .dyn_ref::<HtmlElement>()
        .map(|html_element| html_element.matches(TYPEABLE_SELECTOR).unwrap_or(false))
        .unwrap_or(false)
}

/// `isInteractiveElement(element)` (`element.ts:55-61`): whether the element or an
/// ancestor matches the interactive-element selector. Upstream tolerates `null` input
/// (`element?.closest(...) != null`); the port takes `Option`.
pub fn is_interactive_element(element: Option<&Element>) -> bool {
    const INTERACTIVE_SELECTOR: &str = "button,a[href],[role=\"button\"],select,[tabindex]:not([tabindex=\"-1\"])";
    element
        .map(|element| {
            element
                .closest(&format!("{INTERACTIVE_SELECTOR},{TYPEABLE_SELECTOR}"))
                .map(|found| found.is_some())
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// `isTypeableCombobox(element)` (`element.ts:63-68`): a typeable element with
/// `role="combobox"`.
pub fn is_typeable_combobox(element: Option<&Element>) -> bool {
    let Some(element) = element else {
        return false;
    };
    element.get_attribute("role").as_deref() == Some("combobox") && is_typeable_element(element)
}

/// `matchesFocusVisible(element)` (`element.ts:70-81`): whether the element matches
/// `:focus-visible`. jsdom always reports `true` (`:72-75` — "We don't want to block
/// focus from working with `visibleOnly`"), as does any selector error (`:78-80`).
pub fn matches_focus_visible(element: Option<&Element>) -> bool {
    let Some(element) = element else {
        return true;
    };
    if platform().env.jsdom {
        return true;
    }
    element
        .matches(":focus-visible")
        .unwrap_or(true)
}

/// `getFloatingFocusElement(floatingElement)` (`element.ts:83-96`): resolves the
/// element focus should be managed on — the floating element itself when it carries
/// [`FOCUSABLE_ATTRIBUTE`] (the positioning-wrapper marker), else its first marked
/// descendant, else the floating element unchanged.
pub fn get_floating_focus_element(floating_element: Option<&Element>) -> Option<Element> {
    let floating_element = floating_element?;
    if floating_element.has_attribute(FOCUSABLE_ATTRIBUTE) {
        return Some(floating_element.clone());
    }
    floating_element
        .query_selector(&format!("[{FOCUSABLE_ATTRIBUTE}]"))
        .ok()
        .flatten()
        .or_else(|| Some(floating_element.clone()))
}
