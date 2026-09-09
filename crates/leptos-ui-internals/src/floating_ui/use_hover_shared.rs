//! Port of `packages/react/src/floating-ui-react/hooks/useHoverShared.ts` — the shared
//! hover vocabulary: the `HandleClose` contract `safePolygon` implements and the hover
//! hooks consume, plus the delay/pointer-type resolution helpers both sides use
//! (`specs/library/floating-ui-react/behavior.md`, "Public API surface": `safePolygon()`
//! factory + `useHover`'s `handleClose` option; "Hover corridor chain" cross-cutting
//! behavior).
//!
//! ## Rust adaptations
//!
//! - `HandleClose` (`useHoverShared.ts:24-27`) is a function with `__options` attached.
//!   Rust has no function properties, so it ports to [`HandleClose`]: the factory plus
//!   the options the `__options` member carries (read by `useHover`'s
//!   `blockPointerEvents` layout effect `useHover.ts:355` and by
//!   `useHoverReferenceInteraction`'s `handleCloseOptions` sync `:158-161`).
//! - `HandleCloseContext.elements` (`useHoverShared.ts:15` —
//!   `Pick<ExtendedElements, 'domReference' | 'floating'>`) ports to the two flat
//!   [`HandleCloseContext`] fields: the values are read once per factory invocation
//!   upstream (the spread of the floating context snapshot, `useHover.ts:190-196`), so
//!   plain options replace the live-objects bag.
//! - `HoverDelay = number | Partial<{ open: close }>` (`useHoverShared.ts:29`) is the
//!   ported [`crate::floating_ui::types::Delay`]; the `Delay | (() => Delay)` prop form
//!   (`useHover.ts:45`) ports to [`DelayInput`], and `number | (() => number)`
//!   (`useHover.ts:39`) to [`RestMsInput`] — the same `Value`/`Resolve` split the
//!   `use_focus` port's `FocusDelay` uses.
//! - `resolveValue`'s non-mouse-like pointer-type short-circuit
//!   (`useHoverShared.ts:31-44`) — touch/pen-less pointers resolve every delay to `0`
//!   (immediate) — ports inside [`get_delay`].
//! - `isTargetInsideEnabledTrigger` re-export (`useHoverShared.ts:4`) is a `pub use` of
//!   the `element` port (the same function; upstream re-exports it under the
//!   `isInsideEnabledTrigger` alias).

use std::rc::Rc;

use floating_ui_dom::Placement;
use web_sys::MouseEvent;

use crate::floating_ui::event::is_mouse_like_pointer_type;
use crate::floating_ui::tree::SharedFloatingTreeStore;
use crate::floating_ui::types::Delay;

pub use crate::floating_ui::element::is_target_inside_enabled_trigger as is_inside_enabled_trigger;

/// Port of `HandleCloseOptions` (`useHoverShared.ts:6-9`): the static options a
/// `handleClose` factory carries (upstream `fn.__options`). `safePolygon`'s options
/// extend this interface with nothing extra (`safePolygon.ts:83` — the port's
/// [`crate::floating_ui::safe_polygon::SafePolygonOptions`] alias).
#[derive(Clone, Default)]
pub struct HandleCloseOptions {
    /// `blockPointerEvents` (`useHoverShared.ts:7` — default `false`): whether the
    /// hover machinery may set `pointer-events: none` on a scope element so the
    /// corridor stays traversable.
    pub block_pointer_events: bool,
    /// `getScope` (`useHoverShared.ts:8`): the element the pointer-events mutation
    /// scopes to; `None` lets the callers fall back through their scope chains
    /// (`useHoverFloatingInteraction.ts:127-132`).
    pub get_scope: Option<Rc<dyn Fn() -> Option<web_sys::Element>>>,
}

/// Port of `HandleClose` (`useHoverShared.ts:24-27`) — a `handleClose` factory: invoked
/// once per close session with the [`HandleCloseContext`], returning the document-level
/// `mousemove` handler that decides when the popup closes. The [`HandleClose::options`]
/// field is upstream's `fn.__options` (see the struct docs).
#[derive(Clone)]
pub struct HandleClose {
    /// The factory body — `safePolygon`'s returned `fn` (`safePolygon.ts:94`).
    pub factory: HandleCloseFactory,
    /// `fn.__options` (`safePolygon.ts:445-448`).
    pub options: HandleCloseOptions,
}

/// The factory signature of [`HandleClose`] (`useHoverShared.ts:25` — the
/// `(context) => (event: MouseEvent) => void` shape, curried into two calls).
pub type HandleCloseFactory = Rc<dyn Fn(&HandleCloseContext) -> MouseMoveHandler>;

/// The document-level `mousemove` handler a [`HandleClose`] factory returns per
/// invocation (`safePolygon.ts:129` — `onMouseMove`), shared where the hooks stash it
/// (`useHover.ts:81` `handlerRef`, `useHoverInteractionSharedState.ts:14` `handler`).
pub type MouseMoveHandler = Rc<dyn Fn(&MouseEvent)>;

/// Port of `HandleCloseContextBase` (`useHoverShared.ts:22` — the context minus
/// `onClose`/`tree`/`x`/`y`): the floating-context slice the factory reads, built by
/// the hover hooks from `dataRef.current.floatingContext` (`useHover.ts:190-196`) or
/// supplied by the consumer (`useHoverReferenceInteraction.ts:47`
/// `getHandleCloseContext`).
#[derive(Clone, Default)]
pub struct HandleCloseContextBase {
    /// `placement` (`useHoverShared.ts:14`) — the popup's placement; `None` makes every
    /// handler invocation bail (`safePolygon.ts:134` `side == null`).
    pub placement: Option<Placement>,
    /// `elements.domReference` (`useHoverShared.ts:15`).
    pub dom_reference: Option<web_sys::Element>,
    /// `elements.floating` (`useHoverShared.ts:15`).
    pub floating: Option<web_sys::Element>,
    /// `nodeId` (`useHoverShared.ts:17`).
    pub node_id: Option<String>,
    /// `leave` (`useHoverShared.ts:19`) — carried for the unit's public `HandleClose`
    /// surface; no in-tree writer exists (the field is read by consumers outside this
    /// unit).
    pub leave: Option<bool>,
}

/// Port of `HandleCloseContext` (`useHoverShared.ts:11-20`): the full factory argument.
#[derive(Clone)]
pub struct HandleCloseContext {
    /// `x` (`useHoverShared.ts:12`) — the cursor X at close-session start (the leave
    /// event's `clientX`, `useHover.ts:193`); `None` makes the handler bail
    /// (`safePolygon.ts:134`).
    pub x: Option<f64>,
    /// `y` (`useHoverShared.ts:13`).
    pub y: Option<f64>,
    /// The base context (placement/elements/nodeId/leave).
    pub base: HandleCloseContextBase,
    /// `onClose` (`useHoverShared.ts:16`) — the close request the handler invokes.
    pub on_close: Rc<dyn Fn()>,
    /// `tree` (`useHoverShared.ts:18`) — the floating tree, for the open-child
    /// suppression (`safePolygon.ts:167-180`).
    pub tree: Option<SharedFloatingTreeStore>,
}

/// Port of `HoverDelay`'s prop form (`useHover.ts:45` — `Delay | (() => Delay)`): a
/// concrete delay or a resolver evaluated at read time.
#[derive(Clone)]
pub enum DelayInput {
    /// A concrete delay.
    Value(Delay),
    /// The resolver form (`typeof delay === 'function'`).
    Resolve(Rc<dyn Fn() -> Delay>),
}

impl Default for DelayInput {
    /// The `delay = 0` destructured default (`useHover.ts:63`).
    fn default() -> Self {
        DelayInput::Value(Delay::Value(0))
    }
}

/// Port of `restMs`'s prop form (`useHover.ts:39` — `number | (() => number)`).
#[derive(Clone)]
pub enum RestMsInput {
    /// A concrete duration.
    Value(u32),
    /// The resolver form.
    Resolve(Rc<dyn Fn() -> u32>),
}

impl Default for RestMsInput {
    /// The `restMs = 0` destructured default (`useHover.ts:63`).
    fn default() -> Self {
        RestMsInput::Value(0)
    }
}

/// Port of `UseHoverFloatingInteractionProps['closeDelay']`
/// (`useHoverFloatingInteraction.ts:41` — `number | (() => number)`): the same
/// `Value`/`Resolve` split as [`RestMsInput`], aliased for the floating-side hook.
pub type CloseDelayInput = RestMsInput;

/// Adapts a [`CloseDelayInput`] into the [`DelayInput`] vocabulary [`get_delay`]
/// takes — upstream passes the plain-number prop straight into `getDelay`
/// (`useHoverFloatingInteraction.ts:168`), where a resolved `number` short-circuits
/// the per-direction lookup (`useHoverShared.ts:52-54`).
pub fn number_input_as_delay(value: &RestMsInput) -> DelayInput {
    match value {
        RestMsInput::Value(value) => DelayInput::Value(Delay::Value(*value)),
        RestMsInput::Resolve(resolve) => DelayInput::Value(Delay::Value(resolve())),
    }
}

/// Port of `getDelay` (`useHoverShared.ts:46-57`): resolves the delay for one
/// direction. A non-mouse-like pointer type (`pointerType != null &&
/// !isMouseLikePointerType(pointerType)`, `:35`) forces `0` — the immediate-open
/// behavior touch input relies on (`useHover.ts:268-271`). The function form resolves
/// first (`:39-41`); the single-value form serves both directions
/// ([`Delay::open`]/[`Delay::close`], which also carry the `Partial` fallback).
pub fn get_delay(value: &DelayInput, prop: &'static str, pointer_type: Option<&str>) -> u32 {
    if pointer_type
        .map(|pointer_type| !is_mouse_like_pointer_type(Some(pointer_type), false))
        .unwrap_or(false)
    {
        return 0;
    }

    let delay = match value {
        DelayInput::Value(delay) => *delay,
        DelayInput::Resolve(resolve) => resolve(),
    };

    match prop {
        "open" => delay.open(),
        _ => delay.close(),
    }
}

/// Port of `getRestMs` (`useHoverShared.ts:59-64`): the rest duration, resolving the
/// function form (`useHoverShared.ts:60-62`).
pub fn get_rest_ms(value: &RestMsInput) -> u32 {
    match value {
        RestMsInput::Value(value) => *value,
        RestMsInput::Resolve(resolve) => resolve(),
    }
}

/// Port of `isClickLikeOpenEvent` (`useHoverShared.ts:66-68`): the open event counts as
/// click-like when the pointer interacted with an interactive element inside the popup,
/// or the open event itself was a click/mousedown. `open_event_type` is upstream's
/// `dataRef.current.openEvent?.type`.
pub fn is_click_like_open_event(open_event_type: Option<&str>, interacted_inside: bool) -> bool {
    interacted_inside || open_event_type == Some("click") || open_event_type == Some("mousedown")
}

/// Port of `isHoverOpenEvent` (`useHoverShared.ts:70-72`): the open event was a
/// mouse-family event other than mousedown.
pub fn is_hover_open_event(open_event_type: Option<&str>) -> bool {
    open_event_type
        .is_some_and(|event_type| event_type.contains("mouse") && event_type != "mousedown")
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins `getDelay` (`useHoverShared.ts:46-57`): the single value serves both
    // directions; the partial form falls back per direction; a non-mouse-like pointer
    // type forces 0 regardless of the value; the function form resolves first.
    #[test]
    fn get_delay_resolves_directions_pointer_types_and_functions() {
        let value = DelayInput::Value(Delay::Value(300));
        assert_eq!(get_delay(&value, "open", Some("mouse")), 300);
        assert_eq!(get_delay(&value, "close", Some("mouse")), 300);

        let partial = DelayInput::Value(Delay::Partial {
            open: Some(100),
            close: Some(50),
        });
        assert_eq!(get_delay(&partial, "open", Some("mouse")), 100);
        assert_eq!(get_delay(&partial, "close", Some("mouse")), 50);
        assert_eq!(
            get_delay(
                &DelayInput::Value(Delay::Partial {
                    open: Some(100),
                    close: None
                }),
                "close",
                Some("mouse")
            ),
            0,
            "an omitted direction falls back to 0 (upstream `result?.[prop]`)"
        );

        // `pointerType != null && !isMouseLikePointerType(pointerType)` → resolveValue
        // returns the number 0 (`useHoverShared.ts:35-37`).
        assert_eq!(get_delay(&value, "open", Some("touch")), 0);
        assert_eq!(get_delay(&partial, "close", Some("touch")), 0);
        // An absent pointer type does not short-circuit (`pointerType != null` guard).
        assert_eq!(get_delay(&value, "open", None), 300);

        let resolved = DelayInput::Resolve(Rc::new(|| Delay::Value(120)));
        assert_eq!(get_delay(&resolved, "open", Some("mouse")), 120);
        assert_eq!(
            get_delay(&resolved, "open", Some("touch")),
            0,
            "the pointer-type short-circuit wins over the resolver"
        );
    }

    // Pins `getRestMs` (`useHoverShared.ts:59-64`): the function form resolves; the
    // value form passes through. (No pointer-type logic — that lives in `getDelay`.)
    #[test]
    fn get_rest_ms_resolves_value_and_function_forms() {
        assert_eq!(get_rest_ms(&RestMsInput::Value(80)), 80);
        assert_eq!(get_rest_ms(&RestMsInput::Resolve(Rc::new(|| 40))), 40);
    }

    // Pins `isClickLikeOpenEvent` (`useHoverShared.ts:66-68`): clicks and mousedowns
    // count, other event types do not, and `interactedInside` short-circuits the check.
    #[test]
    fn is_click_like_open_event_covers_clicks_and_inside_interactions() {
        assert!(is_click_like_open_event(Some("click"), false));
        assert!(is_click_like_open_event(Some("mousedown"), false));
        assert!(!is_click_like_open_event(Some("mousemove"), false));
        assert!(!is_click_like_open_event(None, false));
        assert!(
            is_click_like_open_event(Some("mousemove"), true),
            "interactedInside makes any open event click-like"
        );
    }

    // Pins `isHoverOpenEvent` (`useHoverShared.ts:70-72`): mouse-family event types
    // other than mousedown count.
    #[test]
    fn is_hover_open_event_matches_mouse_family_except_mousedown() {
        assert!(is_hover_open_event(Some("mousemove")));
        assert!(is_hover_open_event(Some("mouseover")));
        assert!(is_hover_open_event(Some("mouseleave")));
        assert!(!is_hover_open_event(Some("mousedown")));
        assert!(!is_hover_open_event(Some("pointermove")));
        assert!(!is_hover_open_event(None));
    }
}
