//! Port of `packages/utils/src/useEnhancedClickHandler.ts` (Base UI Phase A util).
//!
//! Upstream is a 50-line hook returning two event-handler props, `{ onClick, onPointerDown }`
//! (`packages/utils/src/useEnhancedClickHandler.ts:49`), giving a caller a cross-browser answer
//! to "which pointer type produced this click?": Safari and Firefox deliver a plain `MouseEvent`
//! to click handlers (it carries no `pointerType`), so the hook records `event.pointerType` on
//! every non-default-prevented `pointerdown` and replays that recording at click time
//! (`packages/utils/src/useEnhancedClickHandler.ts:20-25`, `:42`); Chrome and Edge deliver a
//! real `PointerEvent` whose live `pointerType` is used directly
//! (`packages/utils/src/useEnhancedClickHandler.ts:38-40`); and a click whose `event.detail`
//! is `0` was triggered by the keyboard and is reported as `'keyboard'`
//! (`packages/utils/src/useEnhancedClickHandler.ts:32-36`).
//!
//! The unit has no test files upstream — `ralph/generated/utils.json` records `testFiles: []`
//! for it — so every behavioral claim is UNVERIFIED per `specs/utils/useEnhancedClickHandler.md`
//! and pinned by this module's own tests rather than a reference suite.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The `InteractionType` union `'mouse' | 'touch' | 'pen' | 'keyboard' | ''`
//!   (`packages/utils/src/useEnhancedClickHandler.ts:4`) becomes the [`InteractionType`] enum,
//!   with the `''` member — a meaningful "unknown / no pointerdown recorded yet" sentinel, not
//!   a filler — mapped to [`InteractionType::Unknown`]. Upstream blind-casts
//!   `event.pointerType as InteractionType` (`packages/utils/src/useEnhancedClickHandler.ts:24`,
//!   `:25`, `:40`), which would smuggle any out-of-union string past TypeScript; the port routes
//!   every `pointerType` read through the same normalization (`mouse`/`touch`/`pen` map to
//!   their variants, anything else — including an empty string — to
//!   [`InteractionType::Unknown`]), the member upstream's own type already promises for "not a
//!   known pointer type".
//! - The handler's event parameter is the union `React.MouseEvent | React.PointerEvent`
//!   (`packages/utils/src/useEnhancedClickHandler.ts:14`). Rust has no union, so both returned
//!   handler props hand the event to the shared handler as its common supertype
//!   `&web_sys::MouseEvent` (the crate's event-callback convention, cf.
//!   [`crate::add_event_listener`]'s `FnMut(&Event)`). The borrow targets the same underlying
//!   JS object, whose prototype chain still identifies it — a pointerdown's event downcasts
//!   back to `web_sys::PointerEvent` — so "the original event object passed through
//!   unmodified" (`specs/utils/useEnhancedClickHandler.md`, "Events") holds.
//! - Upstream's click-time duck-typing `'pointerType' in event`
//!   (`packages/utils/src/useEnhancedClickHandler.ts:38`) becomes
//!   `JsCast::dyn_ref::<web_sys::PointerEvent>()` — an `instanceof` check that succeeds exactly
//!   when the browser delivered a `PointerEvent` (Chrome/Edge) and fails for a `MouseEvent`
//!   (Safari/Firefox), which is precisely what the `in` probe distinguishes at runtime.
//! - `React.useRef<InteractionType>('')` (`packages/utils/src/useEnhancedClickHandler.ts:16`)
//!   becomes an `Rc<Cell<InteractionType>>` shared by both returned closures. No signal, no
//!   effects, no cleanup: upstream registers nothing and the ref never triggers renders, so
//!   unlike the crate's `use_controlled`/`use_animation_frame` ports this hook needs no
//!   reactive owner.
//! - `React.useCallback([handler])` (`packages/utils/src/useEnhancedClickHandler.ts:27`, `:46`)
//!   is a React render-phase identity artifact — N/A in Leptos, like the `use_controlled`
//!   port's setter. The closures are created exactly once per hook call and capture the
//!   handler once; a caller needing fresh data reads it from signals inside the handler.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;

/// The upstream `InteractionType` union
/// (`packages/utils/src/useEnhancedClickHandler.ts:4`): the kind of input that produced a
/// click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionType {
    /// A mouse pointer.
    Mouse,

    /// A touch contact.
    Touch,

    /// A pen/stylus contact.
    Pen,

    /// The click was triggered by the keyboard (`event.detail === 0`,
    /// `packages/utils/src/useEnhancedClickHandler.ts:33`).
    Keyboard,

    /// Upstream's `''` member — no pointerdown was recorded yet, the value a click without a
    /// preceding pointerdown reports (`packages/utils/src/useEnhancedClickHandler.ts:16`,
    /// `:42`).
    Unknown,
}

/// Maps a DOM `pointerType` string onto [`InteractionType`] — the Rust form of upstream's
/// `event.pointerType as InteractionType` casts
/// (`packages/utils/src/useEnhancedClickHandler.ts:24`, `:25`, `:40`). `mouse`/`touch`/`pen`
/// are the only values the DOM defines; anything else — including `''` — maps to
/// [`InteractionType::Unknown`], upstream's own "not a known pointer type" member.
fn interaction_type_from_pointer_type(pointer_type: &str) -> InteractionType {
    match pointer_type {
        "mouse" => InteractionType::Mouse,
        "touch" => InteractionType::Touch,
        "pen" => InteractionType::Pen,
        _ => InteractionType::Unknown,
    }
}

/// Upstream's click-time interaction-type resolution
/// (`packages/utils/src/useEnhancedClickHandler.ts:32-44`), split into a pure function over
/// the event's observable properties so the host suite can exercise the branch machine
/// without a DOM; the wasm suite re-runs it through real constructed events.
///
/// `detail` is the click event's `detail` (click count; `0` means keyboard-triggered), and
/// `pointer_type` is `Some` exactly when the event is a `PointerEvent` carrying `pointerType`
/// — the Rust encoding of upstream's `'pointerType' in event` probe
/// (`packages/utils/src/useEnhancedClickHandler.ts:38`). Returns the resolved
/// [`InteractionType`] plus whether the click consumes the recorded pointerdown value (the
/// `lastClickInteractionTypeRef.current = ''` reset at
/// `packages/utils/src/useEnhancedClickHandler.ts:44`): a keyboard click early-returns before
/// the reset (`packages/utils/src/useEnhancedClickHandler.ts:33-36`), so a stale recording
/// survives it, while both pointer branches fall through to it
/// (`packages/utils/src/useEnhancedClickHandler.ts:38-44`).
fn resolve_click_interaction_type(
    detail: i32,
    pointer_type: Option<&str>,
    recorded: InteractionType,
) -> (InteractionType, bool) {
    // `event.detail` has the number of clicks performed on the element. 0 means it was
    // triggered by the keyboard (`packages/utils/src/useEnhancedClickHandler.ts:32-36`).
    if detail == 0 {
        return (InteractionType::Keyboard, false);
    }
    let resolved = match pointer_type {
        // Chrome and Edge correctly use PointerEvent
        // (`packages/utils/src/useEnhancedClickHandler.ts:38-40`).
        Some(pointer_type) => interaction_type_from_pointer_type(pointer_type),
        // Otherwise the event is a MouseEvent (Safari/Firefox): replay the pointerdown
        // recording (`packages/utils/src/useEnhancedClickHandler.ts:42`).
        None => recorded,
    };
    (resolved, true)
}

/// The two handler props upstream returns
/// (`packages/utils/src/useEnhancedClickHandler.ts:49`), shaped for direct attachment to a
/// trigger element. The closures share one interaction-type recording per hook instance, so
/// separate triggers keep independent state
/// (`packages/utils/src/useEnhancedClickHandler.ts:16`).
pub struct EnhancedClickHandlers {
    /// Upstream `onClick`, bound to `handleClick`
    /// (`packages/utils/src/useEnhancedClickHandler.ts:30-47`, `:49`).
    pub on_click: Box<dyn Fn(&web_sys::MouseEvent)>,

    /// Upstream `onPointerDown`, bound to `handlePointerDown`
    /// (`packages/utils/src/useEnhancedClickHandler.ts:18-28`, `:49`).
    pub on_pointer_down: Box<dyn Fn(&web_sys::PointerEvent)>,
}

/// The upstream `useEnhancedClickHandler` hook
/// (`packages/utils/src/useEnhancedClickHandler.ts:13-50`): calls `handler` once on every
/// non-default-prevented `pointerdown` with the live pointer type
/// (`packages/utils/src/useEnhancedClickHandler.ts:20-25`) and again on every `click` — with
/// [`InteractionType::Keyboard`] when `event.detail === 0`, the live pointer type when the
/// click is a `PointerEvent`, and the recorded pointerdown value (initially and after every
/// consuming click [`InteractionType::Unknown`]) otherwise
/// (`packages/utils/src/useEnhancedClickHandler.ts:32-44`). The preventDefault semantics are
/// asymmetric: the `pointerdown` path no-ops entirely when the event is default-prevented
/// (`packages/utils/src/useEnhancedClickHandler.ts:20-22`), while the `click` path has no
/// guard and always invokes the handler (`packages/utils/src/useEnhancedClickHandler.ts:30-47`).
/// Neither path stops propagation, so bubbled events from descendants still reach the
/// handlers (`packages/utils/src/useEnhancedClickHandler.ts:18-47`).
pub fn use_enhanced_click_handler<H>(handler: H) -> EnhancedClickHandlers
where
    H: Fn(&web_sys::MouseEvent, InteractionType) + 'static,
{
    let handler = Rc::new(handler);

    // `lastClickInteractionTypeRef`, initialized to `''`
    // (`packages/utils/src/useEnhancedClickHandler.ts:16`).
    let recorded: Rc<Cell<InteractionType>> = Rc::new(Cell::new(InteractionType::Unknown));

    // `handlePointerDown` (`packages/utils/src/useEnhancedClickHandler.ts:18-28`).
    let pointer_down_handler = Rc::clone(&handler);
    let pointer_down_recorded = Rc::clone(&recorded);
    let on_pointer_down: Box<dyn Fn(&web_sys::PointerEvent)> = Box::new(move |event| {
        // No recording and no handler call when the event is default-prevented
        // (`packages/utils/src/useEnhancedClickHandler.ts:20-22`).
        if event.default_prevented() {
            return;
        }
        // The recording happens before the handler is invoked
        // (`packages/utils/src/useEnhancedClickHandler.ts:24-25`).
        let interaction_type = interaction_type_from_pointer_type(&event.pointer_type());
        pointer_down_recorded.set(interaction_type);
        pointer_down_handler(event, interaction_type);
    });

    // `handleClick` (`packages/utils/src/useEnhancedClickHandler.ts:30-47`).
    let click_handler = Rc::clone(&handler);
    let click_recorded = Rc::clone(&recorded);
    let on_click: Box<dyn Fn(&web_sys::MouseEvent)> = Box::new(move |event| {
        // `'pointerType' in event` (`packages/utils/src/useEnhancedClickHandler.ts:38`).
        let pointer_type = event
            .dyn_ref::<web_sys::PointerEvent>()
            .map(|pointer_event| pointer_event.pointer_type());
        let (interaction_type, consumes_recorded) = resolve_click_interaction_type(
            event.detail(),
            pointer_type.as_deref(),
            click_recorded.get(),
        );
        click_handler(event, interaction_type);
        if consumes_recorded {
            // Single-shot: the recording is consumed and cleared by the click that read it
            // (`packages/utils/src/useEnhancedClickHandler.ts:44`).
            click_recorded.set(InteractionType::Unknown);
        }
    });

    EnhancedClickHandlers {
        on_click,
        on_pointer_down,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:24`): the DOM
    // only defines `mouse`/`touch`/`pen` for `pointerType`; everything else — including `''` —
    // normalizes to the `''`-sentinel variant. Upstream's blind `as` cast would pass any
    // string through; the port normalizes to the member upstream's own type promises.
    #[test]
    fn pointer_type_strings_map_onto_the_interaction_type_variants() {
        assert_eq!(
            interaction_type_from_pointer_type("mouse"),
            InteractionType::Mouse
        );
        assert_eq!(
            interaction_type_from_pointer_type("touch"),
            InteractionType::Touch
        );
        assert_eq!(
            interaction_type_from_pointer_type("pen"),
            InteractionType::Pen
        );
        assert_eq!(
            interaction_type_from_pointer_type(""),
            InteractionType::Unknown
        );
        assert_eq!(
            interaction_type_from_pointer_type("anything-else"),
            InteractionType::Unknown
        );
    }

    // Pins the keyboard branch (`packages/utils/src/useEnhancedClickHandler.ts:32-36`):
    // `event.detail === 0` reports `'keyboard'` and early-returns before the reset, so the
    // recorded value survives (`packages/utils/src/useEnhancedClickHandler.ts:44` is skipped).
    #[test]
    fn a_zero_detail_click_resolves_to_keyboard_without_consuming_the_recording() {
        let (resolved, consumes_recorded) =
            resolve_click_interaction_type(0, Some("mouse"), InteractionType::Touch);
        assert_eq!(resolved, InteractionType::Keyboard);
        assert!(!consumes_recorded);
    }

    // Pins the PointerEvent branch (`packages/utils/src/useEnhancedClickHandler.ts:38-40`):
    // the live pointerType wins over the recording, and the click consumes it
    // (`packages/utils/src/useEnhancedClickHandler.ts:44`).
    #[test]
    fn a_pointer_event_click_resolves_to_the_live_pointer_type_and_consumes_the_recording() {
        let (resolved, consumes_recorded) =
            resolve_click_interaction_type(1, Some("pen"), InteractionType::Mouse);
        assert_eq!(resolved, InteractionType::Pen);
        assert!(consumes_recorded);
    }

    // Pins the MouseEvent branch (`packages/utils/src/useEnhancedClickHandler.ts:42`): the
    // recorded pointerdown value is replayed — including the `''` initial/reset value.
    #[test]
    fn a_mouse_event_click_replays_the_recording_and_consumes_it() {
        let (resolved, consumes_recorded) =
            resolve_click_interaction_type(1, None, InteractionType::Touch);
        assert_eq!(resolved, InteractionType::Touch);
        assert!(consumes_recorded);

        let (resolved, _) = resolve_click_interaction_type(1, None, InteractionType::Unknown);
        assert_eq!(resolved, InteractionType::Unknown);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    // The hook registers no listeners — it returns handler props the caller attaches — so the
    // tests construct real DOM events and invoke the returned closures directly, the way an
    // attached listener would deliver them. They run in a real browser via the wasm32 test
    // runner (`.cargo/config.toml` wires it to chromedriver), like the crate's other wasm test
    // modules: the `'pointerType' in event` branch needs real `PointerEvent`/`MouseEvent`
    // prototype chains to distinguish.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Records `(event type, interaction type)` per handler invocation. The event type proves
    /// the original event is passed through unmodified
    /// (`specs/utils/useEnhancedClickHandler.md`, "Events").
    type Sink = Rc<RefCell<Vec<(String, InteractionType)>>>;

    fn sink_handler(sink: &Sink) -> impl Fn(&web_sys::MouseEvent, InteractionType) + 'static {
        let sink = Rc::clone(sink);
        move |event, interaction_type| {
            sink.borrow_mut().push((event.type_(), interaction_type));
        }
    }

    /// A `MouseEvent` click with the given `detail` — the Safari/Firefox shape (no
    /// `pointerType`; `dyn_ref::<PointerEvent>` fails on it). `detail` defaults to `0` in the
    /// DOM dictionary, so the keyboard-triggered shape is `mouse_click(0)`.
    ///
    /// The init fields go through the `set_*` setters — the chained `detail()`-style
    /// builders are deprecated in web-sys.
    fn mouse_click(detail: i32) -> web_sys::MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_detail(detail);
        web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap_throw()
    }

    /// A real `PointerEvent` — the Chrome/Edge click shape (`detail` must be non-zero to pass
    /// the keyboard branch) or the `pointerdown` the hook records from.
    fn pointer_event(event_type: &str, pointer_type: &str, detail: i32) -> web_sys::PointerEvent {
        let init = web_sys::PointerEventInit::new();
        init.set_pointer_type(pointer_type);
        init.set_detail(detail);
        web_sys::PointerEvent::new_with_event_init_dict(event_type, &init).unwrap_throw()
    }

    /// A default-prevented click: created cancelable, then `preventDefault()`d, so
    /// `event.defaultPrevented` is `true` when the handler sees it.
    fn prevented_mouse_click(detail: i32) -> web_sys::MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_detail(detail);
        init.set_cancelable(true);
        let event =
            web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap_throw();
        event.prevent_default();
        event
    }

    /// A default-prevented pointerdown: created cancelable, then `preventDefault()`d.
    fn prevented_pointer_event(event_type: &str, pointer_type: &str) -> web_sys::PointerEvent {
        let init = web_sys::PointerEventInit::new();
        init.set_pointer_type(pointer_type);
        init.set_cancelable(true);
        let event =
            web_sys::PointerEvent::new_with_event_init_dict(event_type, &init).unwrap_throw();
        event.prevent_default();
        event
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:24-25`): a
    // pointerdown records the pointer type AND invokes the handler once with the live value.
    #[wasm_bindgen_test]
    fn a_pointerdown_invokes_the_handler_once_with_the_live_pointer_type() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_pointer_down)(&pointer_event("pointerdown", "touch", 0));

        assert_eq!(
            *sink.borrow(),
            vec![("pointerdown".to_string(), InteractionType::Touch)]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:20-22`): a
    // default-prevented pointerdown is a full no-op — no handler call AND no recording,
    // proven by the next MouseEvent click reporting `Unknown` instead of the prevented
    // event's type.
    #[wasm_bindgen_test]
    fn a_default_prevented_pointerdown_is_a_full_no_op() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_pointer_down)(&prevented_pointer_event("pointerdown", "mouse"));
        assert!(sink.borrow().is_empty());

        (handlers.on_click)(&mouse_click(1));
        assert_eq!(
            *sink.borrow(),
            vec![("click".to_string(), InteractionType::Unknown)]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:32-36`): a click
    // with `detail === 0` reports `'keyboard'` — `0` is the DOM dictionary default, i.e. the
    // shape a keyboard activation produces.
    #[wasm_bindgen_test]
    fn a_keyboard_click_reports_keyboard() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_click)(&mouse_click(0));

        assert_eq!(
            *sink.borrow(),
            vec![("click".to_string(), InteractionType::Keyboard)]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:33-36` vs `:44`):
    // a keyboard click early-returns before the reset, so the recorded pointer type survives
    // it and the next pointer click still replays it.
    #[wasm_bindgen_test]
    fn a_keyboard_click_does_not_clear_a_previously_recorded_pointer_type() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_pointer_down)(&pointer_event("pointerdown", "touch", 0));
        (handlers.on_click)(&mouse_click(0));
        (handlers.on_click)(&mouse_click(1));

        assert_eq!(
            *sink.borrow(),
            vec![
                ("pointerdown".to_string(), InteractionType::Touch),
                ("click".to_string(), InteractionType::Keyboard),
                ("click".to_string(), InteractionType::Touch),
            ]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:38-40`): a click
    // that is a real PointerEvent (Chrome/Edge) uses the live `pointerType` even with nothing
    // recorded.
    #[wasm_bindgen_test]
    fn a_pointer_event_click_uses_the_live_pointer_type() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_click)(&pointer_event("click", "pen", 1));

        assert_eq!(
            *sink.borrow(),
            vec![("click".to_string(), InteractionType::Pen)]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:42`): a click
    // that is a plain MouseEvent (Safari/Firefox) replays the pointerdown recording.
    #[wasm_bindgen_test]
    fn a_mouse_event_click_replays_the_recorded_pointerdown_type() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0));
        (handlers.on_click)(&mouse_click(1));

        assert_eq!(
            *sink.borrow(),
            vec![
                ("pointerdown".to_string(), InteractionType::Mouse),
                ("click".to_string(), InteractionType::Mouse),
            ]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:44`): the
    // recording is single-shot — consumed and cleared by the click that read it, so a second
    // MouseEvent click with no new pointerdown reports `Unknown`.
    #[wasm_bindgen_test]
    fn a_click_consumes_the_recording_so_a_second_mouse_event_click_reports_unknown() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0));
        (handlers.on_click)(&mouse_click(1));
        (handlers.on_click)(&mouse_click(1));

        assert_eq!(
            *sink.borrow(),
            vec![
                ("pointerdown".to_string(), InteractionType::Mouse),
                ("click".to_string(), InteractionType::Mouse),
                ("click".to_string(), InteractionType::Unknown),
            ]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:16`, `:42`): a
    // click without a preceding pointerdown — e.g. a programmatic click delivering a
    // MouseEvent — reports the initial `''` sentinel.
    #[wasm_bindgen_test]
    fn a_click_without_a_preceding_pointerdown_reports_unknown() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_click)(&mouse_click(1));

        assert_eq!(
            *sink.borrow(),
            vec![("click".to_string(), InteractionType::Unknown)]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:30-47` vs
    // `:20-22`): the preventDefault semantics are asymmetric — the click path has no
    // `defaultPrevented` guard and always invokes the handler.
    #[wasm_bindgen_test]
    fn a_default_prevented_click_still_invokes_the_handler() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_click)(&prevented_mouse_click(1));

        assert_eq!(
            *sink.borrow(),
            vec![("click".to_string(), InteractionType::Unknown)]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:25` + `:33-43`):
    // a full pointer click invokes the handler twice — once on pointerdown with the live
    // pointer type, once on click — with the original events passed through unmodified.
    #[wasm_bindgen_test]
    fn a_pointer_click_invokes_the_handler_twice_with_the_original_events() {
        let sink: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers = use_enhanced_click_handler(sink_handler(&sink));

        (handlers.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0));
        (handlers.on_click)(&pointer_event("click", "mouse", 1));

        assert_eq!(
            *sink.borrow(),
            vec![
                ("pointerdown".to_string(), InteractionType::Mouse),
                ("click".to_string(), InteractionType::Mouse),
            ]
        );
    }

    // Implementation-derived (`packages/utils/src/useEnhancedClickHandler.ts:16`): the
    // recording is per-hook-instance (upstream's `React.useRef`), so separate triggers keep
    // independent interaction state.
    #[wasm_bindgen_test]
    fn instances_keep_independent_interaction_state() {
        let sink_a: Sink = Rc::new(RefCell::new(Vec::new()));
        let sink_b: Sink = Rc::new(RefCell::new(Vec::new()));
        let handlers_a = use_enhanced_click_handler(sink_handler(&sink_a));
        let handlers_b = use_enhanced_click_handler(sink_handler(&sink_b));

        (handlers_a.on_pointer_down)(&pointer_event("pointerdown", "mouse", 0));
        (handlers_b.on_click)(&mouse_click(1));

        assert_eq!(
            *sink_a.borrow(),
            vec![("pointerdown".to_string(), InteractionType::Mouse)]
        );
        assert_eq!(
            *sink_b.borrow(),
            vec![("click".to_string(), InteractionType::Unknown)]
        );
    }
}
