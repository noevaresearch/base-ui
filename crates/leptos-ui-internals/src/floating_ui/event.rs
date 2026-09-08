//! Port of `packages/react/src/floating-ui-react/utils/event.ts` — the platform-aware
//! event-classification helpers shared by the interaction hooks
//! (`specs/library/floating-ui-react/implementation.md`, "Dependencies on other Base UI
//! internals": `utils/event.ts`).
//!
//! ## Rust adaptations
//!
//! - `isReactEvent(event)` (`event.ts:8-10`) tests for React's synthetic-event wrapper
//!   (`'nativeEvent' in event`). The port has no synthetic-event layer — DOM events are
//!   handled directly — so the predicate has no meaningful domain here and is not ported;
//!   call sites that branch on it are folded to their DOM branch when they port.
//! - `stopEvent` (`event.ts:3-6`) takes the DOM event directly.

use leptos_ui_utils::platform;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent, PointerEvent};

/// `stopEvent(event)` (`event.ts:3-6`): `preventDefault` + `stopPropagation`.
pub fn stop_event(event: &Event) {
    event.prevent_default();
    event.stop_propagation();
}

/// `isVirtualClick(event)` (`event.ts:13-23`): detects the "virtual" ( synthesized,
/// keyboard/a11y-driven) clicks screen readers and some platforms produce, so hooks can
/// treat them as keyboard-like. Adapted from react-spectrum's `isVirtualEvent` (upstream
/// comment `:12`).
///
/// The three branches, in order (`:14-22`):
/// 1. A trusted pointer event with an empty `pointerType` — the VoiceOver click shape.
/// 2. Android: a click with the primary button held (`buttons === 1`) is virtual.
/// 3. Otherwise: `detail === 0` without a `pointerType`.
pub fn is_virtual_click(event: &MouseEvent) -> bool {
    if let Some(pointer_event) = event.dyn_ref::<PointerEvent>() {
        if pointer_event.pointer_type().is_empty() && event.is_trusted() {
            return true;
        }
    }

    if platform().os.android && pointer_type_of(event).is_some() {
        return event.type_() == "click" && event.buttons() == 1;
    }

    event.detail() == 0 && pointer_type_of(event).is_none()
}

/// `isVirtualPointerEvent(event)` (`event.ts:25-44`): detects the zero-sized/synthetic
/// pointer events iOS VoiceOver and Android produce. The jsdom branch (`:26-28`)
/// short-circuits to `false` in test environments — the load-bearing bypass the
/// implementation spec flags (`specs/library/floating-ui-react/implementation.md`,
/// "Anything in source not explained by any test" item 7).
pub fn is_virtual_pointer_event(event: &PointerEvent) -> bool {
    if platform().env.jsdom {
        return false;
    }

    let width = event.width();
    let height = event.height();
    let pressure = event.pressure();
    let detail = event.detail();
    let pointer_type = event.pointer_type();

    (!platform().os.android && width == 0 && height == 0)
        || (platform().os.android
            && width == 1
            && height == 1
            && pressure == 0.0
            && detail == 0
            && pointer_type == "mouse")
        // iOS VoiceOver returns 0.333• for width/height (upstream comment `:37`).
        || (width < 1
            && height < 1
            && pressure == 0.0
            && detail == 0
            && pointer_type == "touch")
}

/// `isMouseLikePointerType(pointerType, strict?)` (`event.ts:46-54`): mouse-like pointer
/// types. Non-strict mode treats the empty string and an absent pointer type as
/// mouse-like; strict mode requires a concrete mouse/pen type. The pen inclusion follows
/// upstream's Chromium-on-Linux note (`:47-48`).
pub fn is_mouse_like_pointer_type(pointer_type: Option<&str>, strict: bool) -> bool {
    if strict {
        matches!(pointer_type, Some("mouse") | Some("pen"))
    } else {
        matches!(pointer_type, None | Some("") | Some("mouse") | Some("pen"))
    }
}

/// `isClickLikeEvent(event)` (`event.ts:56-59`): the event types that count as explicit
/// clicks for `syncOpenEvent`'s open-event precedence
/// (`components/FloatingRootStore.ts:91-101`).
pub fn is_click_like_event(event: &Event) -> bool {
    let event_type = event.type_();
    event_type == "click" || event_type == "mousedown" || event_type == "keydown" || event_type == "keyup"
}

/// The `PointerEvent.pointerType` of a mouse-family event, when the underlying event
/// really is a pointer event — the runtime `pointerType` access upstream's union types
/// (`MouseEvent | PointerEvent`) perform (`event.ts:14,18,22`).
///
/// Keyboard events have no `pointerType`; this returns `None` for them so
/// [`is_virtual_click`]'s third branch treats keyboard clicks as pointer-type-less.
pub fn pointer_type_of(event: &MouseEvent) -> Option<String> {
    event
        .dyn_ref::<PointerEvent>()
        .map(|pointer_event| pointer_event.pointer_type())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins `isMouseLikePointerType` (`event.ts:46-54`): mouse and pen are mouse-like;
    // non-strict mode additionally treats the empty string and an absent pointer type as
    // mouse-like (the Chromium-on-Linux pen note at `:47-48` is why pen counts).
    #[test]
    fn mouse_like_pointer_types_follow_the_strict_mode_split() {
        assert!(is_mouse_like_pointer_type(Some("mouse"), false));
        assert!(is_mouse_like_pointer_type(Some("pen"), false));
        assert!(is_mouse_like_pointer_type(Some(""), false));
        assert!(is_mouse_like_pointer_type(None, false));

        assert!(is_mouse_like_pointer_type(Some("mouse"), true));
        assert!(is_mouse_like_pointer_type(Some("pen"), true));

        assert!(!is_mouse_like_pointer_type(Some(""), true), "strict mode rejects the empty type");
        assert!(!is_mouse_like_pointer_type(None, true), "strict mode rejects an absent type");
        assert!(!is_mouse_like_pointer_type(Some("touch"), false));
        assert!(!is_mouse_like_pointer_type(Some("touch"), true));
    }
}
