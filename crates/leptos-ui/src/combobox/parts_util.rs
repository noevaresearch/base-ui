//! The per-part shared utilities — `packages/react/src/combobox/utils/parts.ts:1-57`
//! and `handleInputPress.ts:1-41` ported.
//!
//! Upstream shape:
//! - `usePopupSide` (`parts.ts:10-16`): the store's retained last side is only
//!   meaningful while the positioner is mounted — the hook nulls it otherwise. The
//!   upstream hook reads three state slices reactively; the port takes the resolved
//!   values (the reactive wiring belongs to the view layer that consumes it, and the
//!   pure predicate is what the tests pin).
//! - `useListEmpty` (`parts.ts:21-23`): whether the filtered list has no items. The
//!   upstream hook reads the derived-items context; the port takes the filtered
//!   window's length.
//! - `getChipNavigationKeys` (`parts.ts:26-31`): the RTL-swapped arrow pair.
//! - [`get_index_after_chip_removal`] (`parts.ts:34-39`): where the highlight lands
//!   once the chip at `index` is removed, or none for no highlight.
//! - [`click_highlighted_item`] (`parts.ts:42-56`): commits the highlighted item by
//!   clicking it, tagging the originating event through `selectionEventRef` so the
//!   item's handler can attribute the selection.
//! - [`handle_input_press`] (`handleInputPress.ts:10-41`): the press funnel shared by
//!   Input, Chips, and InputGroup — veto on a prevented Base UI handler, ignore
//!   presses whose target is an interactive element or an ignored target, prevent
//!   default, focus the input, and optionally open with reason `inputPress`.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - `handleInputPress`'s `event` is the React synthetic-mouse-event shape; the port
//!   takes the pieces the body reads: the `baseUIHandlerPrevented` flag, the
//!   current/target elements, and a `prevent_default` flag the caller applies (the
//!   port cannot mutate the caller's event object through `&` — the preventDefault
//!   call's observable contract is "the default is suppressed", which the flag
//!   communicates). The tests pin the flag and focus behavior, the same surface the
//!   upstream test's mock pins.
//! - `isElement` (`@floating-ui/utils/dom`) ports to the `EventTarget.dyn_ref::<
//!   Element>` probe; `getTarget`/`contains`/`isInteractiveElement` come from the
//!   ported floating layer (`floating_ui::element`).

use wasm_bindgen::JsCast;
use web_sys::Element;

use leptos_ui_internals::floating_ui::element::{get_target, is_interactive_element};
use leptos_ui_internals::floating_ui::reasons;

use crate::combobox::store::{ChangeCommandDetails, ComboboxStore};

/// The `usePopupSide` predicate (`parts.ts:10-16`): the retained side is meaningful
/// only while the positioner is mounted.
pub fn use_popup_side(
    mounted: bool,
    positioner_element: Option<&Element>,
    popup_side: Option<String>,
) -> Option<String> {
    if mounted && positioner_element.is_some() {
        popup_side
    } else {
        None
    }
}

/// Whether the filtered list has no items to show (`parts.ts:21-23`).
pub fn use_list_empty(filtered_items_len: usize) -> bool {
    filtered_items_len == 0
}

/// The arrow keys that move the chip highlight backwards and forwards, in that order
/// (`parts.ts:26-31`).
pub fn get_chip_navigation_keys(direction: &str) -> (&'static str, &'static str) {
    if direction == "rtl" {
        ("ArrowRight", "ArrowLeft")
    } else {
        ("ArrowLeft", "ArrowRight")
    }
}

/// Where the highlight lands once the chip at `index` is removed, or `None` for no
/// highlight (`parts.ts:34-39`).
pub fn get_index_after_chip_removal(index: usize, chip_count: usize) -> Option<usize> {
    // The `index >= chipCount - 1 ? chipCount - 2 : index` walk runs in signed
    // arithmetic upstream; the underflow case is the `undefined` return.
    let next_index = if index >= chip_count.saturating_sub(1) {
        chip_count as isize - 2
    } else {
        index as isize
    };
    if next_index >= 0 {
        Some(next_index as usize)
    } else {
        None
    }
}

/// Commits the highlighted item by clicking it, tagging the originating event so the
/// item's handler can attribute the selection to it (`parts.ts:42-56`).
pub fn click_highlighted_item(
    store: &ComboboxStore,
    active_index: usize,
    native_event: &web_sys::Event,
) {
    let list_item = store
        .context
        .list_ref
        .borrow()
        .get(active_index)
        .cloned()
        .flatten();

    if let Some(list_item) = list_item {
        *store.context.selection_event_ref.borrow_mut() = Some(native_event.clone());
        let _ = js_sys::Reflect::get(&list_item, &wasm_bindgen::JsValue::from_str("click"))
            .and_then(|click| click.dyn_into::<js_sys::Function>())
            .ok()
            .map(|click| {
                let _ = click.call0(&list_item);
            });
        *store.context.selection_event_ref.borrow_mut() = None;
    }
}

/// The `handleInputPress` inputs the port extracts from the React synthetic event —
/// see the module adaptation notes.
pub struct InputPressEvent<'a> {
    /// The `baseUIHandlerPrevented` flag (`handleInputPress.ts:11-13`).
    pub base_ui_handler_prevented: bool,
    /// The event's `currentTarget`.
    pub current_target: Option<&'a Element>,
    /// The native event, whose `target` the shadow-safe `getTarget` resolves.
    pub native_event: Option<&'a web_sys::Event>,
    /// Whether the caller should apply `event.preventDefault()` — the port sets it
    /// instead of mutating the caller's event object. Shared (`Rc`), because
    /// `Cell::clone` copies: callers clone-and-inspect after the funnel runs.
    pub prevent_default: std::rc::Rc<std::cell::Cell<bool>>,
}

/// The press funnel shared by Input, Chips, and InputGroup
/// (`handleInputPress.ts:10-41`): focus the input and optionally open with reason
/// `inputPress`, ignoring interactive targets. Returns whether the default was
/// suppressed (the caller applies it to the real event).
pub fn handle_input_press(
    event: &InputPressEvent<'_>,
    store: &ComboboxStore,
    disabled: bool,
    should_ignore_target: Option<&dyn Fn(Option<&Element>) -> bool>,
) -> bool {
    if event.base_ui_handler_prevented {
        return false;
    }

    let target = event
        .native_event
        .and_then(get_target)
        .and_then(|target| target.dyn_into::<Element>().ok());
    let target_element: Option<&Element> = target.as_ref();
    let is_interactive = target_element.is_some_and(|e| is_interactive_element(Some(e)));
    let is_ignored = should_ignore_target.is_some_and(|predicate| predicate(target_element));
    if event.current_target.is_some()
        && target_element != event.current_target
        && (is_ignored || is_interactive)
    {
        return false;
    }

    event.prevent_default.set(true);

    if disabled {
        return true;
    }

    if let Some(input) = store.context.input_ref.borrow().as_ref() {
        let _ = js_sys::Reflect::get(input, &wasm_bindgen::JsValue::from_str("focus"))
            .ok()
            .and_then(|focus| focus.dyn_into::<js_sys::Function>().ok())
            .map(|focus| {
                let _ = focus.call0(input);
            });
    }

    if store.get_snapshot().open_on_input_click {
        let details = ChangeCommandDetails::new(reasons::INPUT_PRESS, make_stub_event(), None, ());
        (store.context.set_open.clone())(true, &details);
    }

    true
}

/// The stub event the command details carry when the caller supplied no native event —
/// upstream `createChangeEventDetails` defaults its `event` argument to
/// `new Event('base-ui')` (`createBaseUIEventDetails.ts:129-132`).
fn make_stub_event() -> web_sys::Event {
    web_sys::Event::new("base-ui").expect("construct the stub event")
}
