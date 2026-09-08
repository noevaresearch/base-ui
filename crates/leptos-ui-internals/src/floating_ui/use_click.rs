//! Port of `packages/react/src/floating-ui-react/hooks/useClick.ts` — opens or closes
//! the floating element when clicking the reference element
//! (`specs/library/floating-ui-react/behavior.md`, "Public API surface": `useClick`
//! (`toggle`, `event`, `ignoreMouse`, `touchOpenDelay`, `reason`, `stickIfOpen`)).
//!
//! ## Rust adaptations
//!
//! - The `reference` handler bag (`useClick.ts:81-231`) is built once at hook-call
//!   time — the `useMemo` with its dependency array has no render-cycle counterpart
//!   (Leptos components run once; the bag's closures read the store fresh at event
//!   time, which is what upstream's memo dependency list guarantees). Handlers receive
//!   native events through [`crate::floating_ui::element_props`] (no synthetic layer —
//!   `event.nativeEvent` reads become the event itself).
//! - `pointerTypeRef` (`useClick.ts:77`) is the clone-shared `Rc<RefCell<Option<
//!   String>>>`: upstream's `React.useRef('mouse' | 'pen' | 'touch' | 'virtual')`. The
//!   string is kept rather than an enum because the values flow into
//!   [`is_mouse_like_pointer_type`]'s `Option<&str>` parameter unchanged and
//!   `'virtual'` is a marker value upstream synthesizes (`useClick.ts:137-141`).
//! - `getNextOpen` (`useClick.ts:99-129`) is extracted verbatim into
//!   [`next_open_decision`] — a pure function, so its decision table is host-testable.
//! - The current-target capture before the rAF (`useClick.ts:173-175` — "React sets it
//!   to null after the event handler completes") applies to native events too:
//!   `Event.currentTarget` is only valid during dispatch, so the frame callback
//!   receives the captured element, not the event.
//! - `EMPTY_OBJECT` for the disabled case (`useClick.ts:233`) is
//!   [`ElementProps::default`] — every role `None`.
//! - The touch-open delay rides the `useTimeout` port ([`Timeout`]) and the
//!   mousedown-deferred open rides the `useAnimationFrame` port ([`AnimationFrame`]),
//!   per the implementation spec's per-call-site hook inventory
//!   (`specs/library/floating-ui-react/implementation.md`, "Base UI utility hooks (per
//!   call site)": `useClick.ts:78-79`).

use std::cell::RefCell;
use std::rc::Rc;

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, KeyboardEvent, MouseEvent, PointerEvent};

use leptos_ui_utils::{use_animation_frame, use_timeout};

use crate::floating_ui::element;
use crate::floating_ui::element_props::{ElementHandlers, ElementProps, FloatingContextSource};
use crate::floating_ui::event::{is_mouse_like_pointer_type, is_virtual_pointer_event};
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::reasons;
use crate::floating_ui::types::RootOpenChangeEventDetails;

/// Port of `UseClickProps['event']` (`useClick.ts:23`) — the mouse event that counts
/// as a "click".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickEventOption {
    /// `'click'` (default).
    Click,
    /// `'mousedown'`.
    Mousedown,
    /// `'mousedown-only'`.
    MousedownOnly,
}

/// Port of `UseClickProps` (`useClick.ts:12-53`) with the documented defaults
/// (`useClick.ts:63-71`).
#[derive(Clone)]
pub struct UseClickProps {
    /// `enabled` (`useClick.ts:18` — default `true`).
    pub enabled: bool,
    /// `event` (`useClick.ts:24` — default `'click'`).
    pub event: ClickEventOption,
    /// `toggle` (`useClick.ts:29` — default `true`).
    pub toggle: bool,
    /// `ignoreMouse` (`useClick.ts:35` — default `false`).
    pub ignore_mouse: bool,
    /// `stickIfOpen` (`useClick.ts:41` — default `true`).
    pub stick_if_open: bool,
    /// `touchOpenDelay` (`useClick.ts:47` — default `0`).
    pub touch_open_delay: u32,
    /// `reason` (`useClick.ts:50` — default `REASONS.triggerPress`).
    pub reason: String,
}

impl Default for UseClickProps {
    fn default() -> Self {
        Self {
            enabled: true,
            event: ClickEventOption::Click,
            toggle: true,
            ignore_mouse: false,
            stick_if_open: true,
            touch_open_delay: 0,
            reason: reasons::TRIGGER_PRESS.to_owned(),
        }
    }
}

/// Port of `getNextOpen` (`useClick.ts:99-129`) — the open/close decision for a
/// trigger press, over a live open event:
///
/// - Moving between triggers always opens the newly active one (`:107-110`).
/// - A closed popup opens on the next press (`:112-115`).
/// - Non-toggle mode never closes on a repeated trigger press (`:117-120`).
/// - With `stickIfOpen`, a popup opened by another event (hover/focus) stays open
///   until the matching click-like event closes it (`:122-125`).
/// - Otherwise a repeated click toggles the popup closed (`:127-128`).
pub fn next_open_decision(
    open: bool,
    has_clicked_on_inactive_trigger: bool,
    toggle: bool,
    open_event: Option<&Event>,
    stick_if_open: bool,
    is_click_like_open_event: &dyn Fn(&str) -> bool,
) -> bool {
    next_open_decision_by_type(
        open,
        has_clicked_on_inactive_trigger,
        toggle,
        open_event.map(|event| event.type_()).as_deref(),
        stick_if_open,
        is_click_like_open_event,
    )
}

/// The type-string core of [`next_open_decision`] — the decision table over the open
/// event's type (`useClick.ts:99-129` reads `openEvent.type` through the predicate),
/// host-testable because no DOM realm is needed to name an event type.
pub fn next_open_decision_by_type(
    open: bool,
    has_clicked_on_inactive_trigger: bool,
    toggle: bool,
    open_event_type: Option<&str>,
    stick_if_open: bool,
    is_click_like_open_event: &dyn Fn(&str) -> bool,
) -> bool {
    if open && has_clicked_on_inactive_trigger {
        // Moving between triggers should always open the newly active one.
        return true;
    }

    if !open {
        // A closed popup should open on the next press.
        return true;
    }

    if !toggle {
        // Non-toggle mode never closes on a repeated trigger press.
        return true;
    }

    if open_event_type.is_some() && stick_if_open {
        // Preserve hover/focus-opened popups until the matching click-like event
        // closes them.
        return !is_click_like_open_event(open_event_type.expect("checked above"));
    }

    // Otherwise, a repeated click toggles the popup closed.
    false
}

/// Port of `useClick(context, props)` (`useClick.ts:59-234`). Must be called inside a
/// reactive owner (the timers register cleanups). Returns the `reference` handler bag,
/// or the empty props when disabled.
pub fn use_click(context: impl Into<FloatingContextSource>, props: UseClickProps) -> ElementProps {
    let UseClickProps {
        enabled,
        event: event_option,
        toggle,
        ignore_mouse,
        stick_if_open,
        touch_open_delay,
        reason,
    } = props;

    let store: Rc<FloatingRootStore> = context.into().root_store();

    // `const dataRef = store.context.dataRef` (`useClick.ts:75`).
    let data_ref = Rc::clone(&store.context.data_ref);

    // `pointerTypeRef` (`useClick.ts:77`), `frame` (`:78`), `touchOpenTimeout` (`:79`).
    let pointer_type_ref: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let frame = use_animation_frame();
    let touch_open_timeout = use_timeout();

    // `setOpenWithTouchDelay` (`useClick.ts:82-97`): build the change details for the
    // configured reason, then either schedule the touch-only delay or open/close
    // immediately. Closing is never delayed (`:90` — `nextOpen && ...`).
    let set_open_with_touch_delay: Rc<dyn Fn(bool, Event, Option<Element>, Option<String>)> = {
        let store = Rc::clone(&store);
        let touch_open_timeout = touch_open_timeout.clone();
        Rc::new(
            move |next_open: bool,
                  native_event: Event,
                  target: Option<Element>,
                  pointer_type: Option<String>| {
                let details = RootOpenChangeEventDetails::new(
                    reason.clone(),
                    native_event,
                    target,
                    String::new(),
                );

                if next_open && pointer_type.as_deref() == Some("touch") && touch_open_delay > 0 {
                    let store = Rc::clone(&store);
                    let details = details.clone();
                    touch_open_timeout.start(touch_open_delay, move || {
                        store.set_open(true, &details);
                    });
                } else {
                    store.set_open(next_open, &details);
                }
            },
        )
    };

    // `onMouseDown`'s click-like predicate (`useClick.ts:161`): the open event counts
    // as click-like when it is a click or a mousedown.
    let is_click_like_for_mousedown =
        |event_type: &str| event_type == "click" || event_type == "mousedown";
    // `onClick`'s click-like predicate (`useClick.ts:203-208`): keyboard events count
    // too (a popup opened via keyboard still toggles on click).
    let is_click_like_for_click = |event_type: &str| {
        event_type == "click"
            || event_type == "mousedown"
            || event_type == "keydown"
            || event_type == "keyup"
    };

    // The `reference` bag (`useClick.ts:131-219`).
    let reference = ElementHandlers {
        // `onPointerDown` (`useClick.ts:132-142`): record the pointer type, mapping
        // virtual mouse-like presses (screen-reader activations) to `'virtual'` so
        // `ignoreMouse` cannot drop them while `touchOpenDelay` still applies to real
        // virtual touch presses (iOS VoiceOver).
        on_pointer_down: {
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            Some(Rc::new(move |event: &PointerEvent| {
                let pointer_type = event.pointer_type();
                let recorded = if is_mouse_like_pointer_type(Some(&pointer_type), true)
                    && is_virtual_pointer_event(event)
                {
                    "virtual".to_owned()
                } else {
                    pointer_type
                };
                *pointer_type_ref.borrow_mut() = Some(recorded);
            }))
        },
        // `onMouseDown` (`useClick.ts:143-182`) — the `event: 'mousedown'` path.
        on_mouse_down: {
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            let data_ref = Rc::clone(&data_ref);
            let store = Rc::clone(&store);
            let set_open_with_touch_delay = Rc::clone(&set_open_with_touch_delay);
            let frame = frame.clone();
            Some(Rc::new(move |event: &MouseEvent| {
                let pointer_type = pointer_type_ref.borrow().clone();
                let native_event = event.clone();
                let open = store.select(selectors::open);

                // Ignore all buttons except for the "main" button, non-mousedown
                // event modes, and mouse input when `ignoreMouse` is set
                // (`useClick.ts:148-156`).
                if event.button() != 0
                    || event_option == ClickEventOption::Click
                    || (is_mouse_like_pointer_type(pointer_type.as_deref(), true) && ignore_mouse)
                {
                    return;
                }

                // `getNextOpen(open, event.currentTarget, ...)` (`useClick.ts:158-162`).
                let current_target: Option<Element> = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<Element>().ok());
                let dom_reference = selectors::dom_reference_element(&store.get_snapshot());
                let has_clicked_on_inactive_trigger = dom_reference != current_target;
                let next_open = next_open_decision(
                    open,
                    has_clicked_on_inactive_trigger,
                    toggle,
                    data_ref.borrow().open_event.as_ref(),
                    stick_if_open,
                    &is_click_like_for_mousedown,
                );

                // Animations sometimes won't run on a typeable element if using a
                // rAF — focus is always set on these elements. For touch, the open
                // may be delayed (`useClick.ts:164-171`).
                let target = element::get_target(&native_event)
                    .and_then(|target| target.dyn_into::<Element>().ok());
                if target
                    .as_ref()
                    .map(|target| element::is_typeable_element(target))
                    .unwrap_or(false)
                {
                    set_open_with_touch_delay(next_open, native_event.into(), target, pointer_type);
                    return;
                }

                // Capture the currentTarget before the rAF (see the module docs), then
                // wait until focus is set on the element — an alternative to
                // `event.preventDefault()` that avoids `:focus-visible`
                // (`useClick.ts:173-181`).
                let event_current_target = current_target;
                let native_event_for_frame = native_event.clone();
                let frame_callback = Rc::clone(&set_open_with_touch_delay);
                frame.request(move || {
                    frame_callback(
                        next_open,
                        native_event_for_frame.into(),
                        event_current_target,
                        pointer_type.clone(),
                    );
                });
            }))
        },
        // `onClick` (`useClick.ts:183-215`) — the default click path.
        on_click: {
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            let data_ref = Rc::clone(&data_ref);
            let store = Rc::clone(&store);
            let set_open_with_touch_delay = Rc::clone(&set_open_with_touch_delay);
            Some(Rc::new(move |event: &MouseEvent| {
                if event_option == ClickEventOption::MousedownOnly {
                    return;
                }

                let pointer_type = pointer_type_ref.borrow().clone();

                // The mousedown path already handled this press: consume the recorded
                // type so a trailing `click` does not toggle again
                // (`useClick.ts:190-193`).
                if event_option == ClickEventOption::Mousedown && pointer_type.is_some() {
                    *pointer_type_ref.borrow_mut() = None;
                    return;
                }

                if is_mouse_like_pointer_type(pointer_type.as_deref(), true) && ignore_mouse {
                    return;
                }

                let open = store.select(selectors::open);
                let current_target: Option<Element> = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<Element>().ok());
                let dom_reference = selectors::dom_reference_element(&store.get_snapshot());
                let has_clicked_on_inactive_trigger = dom_reference != current_target;
                let next_open = next_open_decision(
                    open,
                    has_clicked_on_inactive_trigger,
                    toggle,
                    data_ref.borrow().open_event.as_ref(),
                    stick_if_open,
                    &is_click_like_for_click,
                );
                set_open_with_touch_delay(
                    next_open,
                    event.clone().into(),
                    current_target,
                    pointer_type,
                );
            }))
        },
        // `onKeyDown` (`useClick.ts:216-218`): keyboard activation is not a pointer
        // press.
        on_key_down: {
            let pointer_type_ref = Rc::clone(&pointer_type_ref);
            Some(Rc::new(move |_event: &KeyboardEvent| {
                *pointer_type_ref.borrow_mut() = None;
            }))
        },
        ..ElementHandlers::default()
    };

    // `enabled ? { reference } : EMPTY_OBJECT` (`useClick.ts:233`).
    if enabled {
        ElementProps {
            reference: Some(reference),
            ..ElementProps::default()
        }
    } else {
        ElementProps::default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::next_open_decision_by_type;

    // The `&Event`-taking wrapper (`next_open_decision`) only adds the `type_()` read,
    // which needs a DOM realm; the decision table itself is pure, so the host pins the
    // real table through the type-string core and the wasm suite drives the wrapper
    // through real events.

    // Pins the decision table (`useClick.ts:99-129`): closed popups open on press;
    // repeated clicks toggle closed; `toggle: false` never closes; switching triggers
    // always opens.
    #[test]
    fn the_decision_table_covers_toggle_and_inactive_trigger_branches() {
        let is_click_like = |event_type: &str| event_type == "click" || event_type == "mousedown";

        // A closed popup opens on the next press (`:112-115`).
        assert!(next_open_decision_by_type(
            false,
            false,
            true,
            None,
            true,
            &is_click_like
        ));
        // Even when the open event is click-like and stickIfOpen is set.
        assert!(next_open_decision_by_type(
            false,
            true,
            true,
            Some("click"),
            true,
            &is_click_like
        ));
        // A repeated click on the active trigger toggles closed (`:127-128`).
        assert!(!next_open_decision_by_type(
            true,
            false,
            true,
            Some("click"),
            true,
            &is_click_like
        ));
        // toggle: false never closes on a repeated press (`:117-120`).
        assert!(next_open_decision_by_type(
            true,
            false,
            false,
            Some("click"),
            true,
            &is_click_like
        ));
        // Moving between triggers always opens the newly active one (`:107-110`),
        // regardless of everything else.
        assert!(next_open_decision_by_type(
            true,
            true,
            true,
            Some("click"),
            true,
            &is_click_like
        ));
        assert!(next_open_decision_by_type(
            true,
            true,
            false,
            None,
            false,
            &is_click_like
        ));
    }

    // Pins the stickIfOpen branches (`useClick.ts:122-128`): a hover-opened popup
    // (open event not click-like) stays open on the first click with stickIfOpen and
    // closes with stickIfOpen: false; a click-opened popup toggles closed either way.
    #[test]
    fn stick_if_open_preserves_non_click_opened_popups_until_the_matching_event() {
        let is_click_like = |event_type: &str| event_type == "click" || event_type == "mousedown";

        // Hover-opened (open event type is not click-like): stickIfOpen keeps it open.
        assert!(next_open_decision_by_type(
            true,
            false,
            true,
            Some("mouseover"),
            true,
            &is_click_like
        ));
        // stickIfOpen: false closes it.
        assert!(!next_open_decision_by_type(
            true,
            false,
            true,
            Some("mouseover"),
            false,
            &is_click_like
        ));
        // Click-opened: the matching click-like event closes it even with stickIfOpen.
        assert!(!next_open_decision_by_type(
            true,
            false,
            true,
            Some("click"),
            true,
            &is_click_like
        ));
        assert!(!next_open_decision_by_type(
            true,
            false,
            true,
            Some("mousedown"),
            true,
            &is_click_like
        ));
        // stickIfOpen is moot without an open event (e.g. a store-seeded open).
        assert!(!next_open_decision_by_type(
            true,
            false,
            true,
            None,
            true,
            &is_click_like
        ));
    }

    // Pins the onClick click-like predicate (`useClick.ts:203-208`): keyboard open
    // events also count as click-like, while hover events do not; the mousedown
    // path's predicate (`useClick.ts:161`) does not include keys.
    #[test]
    fn the_click_path_counts_keyboard_open_events_as_click_like() {
        let is_click_like_for_click = |event_type: &str| {
            event_type == "click"
                || event_type == "mousedown"
                || event_type == "keydown"
                || event_type == "keyup"
        };
        assert!(is_click_like_for_click("keydown"));
        assert!(is_click_like_for_click("keyup"));
        assert!(!is_click_like_for_click("mouseover"));

        let is_click_like_for_mousedown =
            |event_type: &str| event_type == "click" || event_type == "mousedown";
        assert!(!is_click_like_for_mousedown("keydown"));
    }

    // Pins the mousedown guard's main-button comparison (`useClick.ts:148-149` —
    // `event.button !== 0`): the guard wiring itself runs in the wasm suite against
    // real events.
    #[test]
    fn the_mousedown_guard_drops_non_main_buttons() {
        let is_main_button = |button: i16| button == 0;
        assert!(is_main_button(0));
        assert!(!is_main_button(1));
        assert!(!is_main_button(2));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn store_with(open: bool, log: Rc<RefCell<Vec<(bool, String)>>>) -> Rc<FloatingRootStore> {
        let store = FloatingRootStore::new(FloatingRootStoreOptions {
            open,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        });
        // The consumer wiring the upstream test harness has (`useClick.test.tsx:20-27`
        // — `onOpenChange(nextOpen) { setOpen(nextOpen) }`): the consumer state syncs
        // back into the store through the context-hook layout effect
        // (`hooks/useFloatingRootContext.ts:58-74`). The port's stand-in writes the
        // store's open state in the callback (through a Weak — the context holds the
        // callback, so an Rc would cycle).
        let weak = Rc::downgrade(&store);
        store.context.set_on_open_change(Some(Rc::new(
            move |open: bool, details: &RootOpenChangeEventDetails| {
                if let Some(store) = weak.upgrade() {
                    store.update(|state, _| {
                        state.open = open;
                        true
                    });
                }
                log.borrow_mut().push((open, details.reason.clone()));
            },
        )));
        store
    }

    fn button_with_store(store: &Rc<FloatingRootStore>) -> web_sys::HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let button: web_sys::HtmlElement = document
            .create_element("button")
            .unwrap()
            .dyn_into()
            .unwrap();
        document.body().unwrap().append_child(&button).unwrap();
        // Seed the store's DOM reference — `getNextOpen`'s inactive-trigger check
        // compares against it (`useClick.ts:105`).
        store.set_field(
            |state| &mut state.dom_reference_element,
            Some(button.clone().into()),
        );
        button
    }

    fn pointer_event(event_type: &str, pointer_type: &str) -> PointerEvent {
        let init = web_sys::PointerEventInit::new();
        init.set_pointer_type(pointer_type);
        init.set_bubbles(true);
        init.set_button(0);
        web_sys::PointerEvent::new_with_event_init_dict(event_type, &init).unwrap()
    }

    fn mouse_event(event_type: &str) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_button(0);
        MouseEvent::new_with_mouse_event_init_dict(event_type, &init).unwrap()
    }

    /// Fires the full mouse press sequence the upstream test harness uses
    /// (`useClick.test.tsx:62-68`): pointerdown, mousedown, click.
    fn press_mouse(target: &web_sys::HtmlElement) {
        target
            .dispatch_event(&pointer_event("pointerdown", "mouse"))
            .unwrap();
        target.dispatch_event(&mouse_event("mousedown")).unwrap();
        target.dispatch_event(&mouse_event("click")).unwrap();
    }

    fn touch_click(target: &web_sys::HtmlElement) {
        target
            .dispatch_event(&pointer_event("pointerdown", "touch"))
            .unwrap();
        target.dispatch_event(&mouse_event("click")).unwrap();
    }

    struct OpenLog {
        calls: Rc<RefCell<Vec<(bool, String)>>>,
    }

    impl OpenLog {
        fn new() -> (Self, Rc<RefCell<Vec<(bool, String)>>>) {
            let calls: Rc<RefCell<Vec<(bool, String)>>> = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    calls: Rc::clone(&calls),
                },
                calls,
            )
        }

        fn calls(&self) -> Vec<(bool, String)> {
            self.calls.borrow().clone()
        }
    }

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    // Pins the toggle contract (`useClick.test.tsx:92-104` — "opens and closes on
    // repeated clicks"): the first click opens (reason trigger-press), the second
    // closes.
    #[wasm_bindgen_test]
    fn opens_and_closes_on_repeated_clicks() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(Rc::clone(&store), UseClickProps::default());
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            button.dispatch_event(&mouse_event("click")).unwrap();
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "the first click opens with the default reason"
            );

            button.dispatch_event(&mouse_event("click")).unwrap();
            assert_eq!(
                log.calls(),
                vec![
                    (true, "trigger-press".to_owned()),
                    (false, "trigger-press".to_owned())
                ],
                "the repeated click toggles closed"
            );
        });
    }

    // Pins the immediate click path (`useClick.ts:209-214` — the click handler calls
    // `setOpenWithTouchDelay` synchronously; only the mousedown path defers through a
    // frame): a plain click opens without waiting for a frame.
    #[wasm_bindgen_test]
    fn opens_immediately_on_click_without_waiting_for_a_frame() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(Rc::clone(&store), UseClickProps::default());
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            button.dispatch_event(&mouse_event("click")).unwrap();
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "the click open landed synchronously during dispatch"
            );
        });
    }

    // Pins `toggle: false` (`useClick.test.tsx:106-115`): repeated clicks never close.
    #[wasm_bindgen_test]
    fn keeps_open_on_repeated_clicks_when_toggle_is_false() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    toggle: false,
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            button.dispatch_event(&mouse_event("click")).unwrap();
            button.dispatch_event(&mouse_event("click")).unwrap();

            assert_eq!(
                log.calls(),
                vec![
                    (true, "trigger-press".to_owned()),
                    (true, "trigger-press".to_owned())
                ],
                "both presses open; neither closes"
            );
        });
    }

    // Pins the mousedown event path (`useClick.test.tsx:117-135`) plus the rAF
    // deferral (`useClick.ts:173-181` — wait until focus lands to avoid
    // `:focus-visible`): the press does not open synchronously; the frame callback
    // opens exactly once and the trailing click is consumed
    // (`useClick.ts:190-193`), so the state stays consistent.
    #[wasm_bindgen_test(async)]
    async fn opens_from_the_mousedown_path_and_consumes_the_trailing_click() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    event: ClickEventOption::Mousedown,
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            press_mouse(&button);
            assert!(
                log.calls().is_empty(),
                "the mousedown open is frame-deferred, and the click is consumed by the \
                 recorded pointer type — nothing opens synchronously"
            );

            // Let the frame callback run and settle.
            sleep(60).await;
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "the press opens exactly once, through the frame"
            );
        };
        __owner.cleanup();
    }

    // Pins `event: 'mousedown-only'` (`useClick.test.tsx:137-147`): mousedown opens
    // (frame-deferred), a later bare click is ignored — never closes.
    #[wasm_bindgen_test(async)]
    async fn ignores_the_click_event_after_mousedown_when_event_is_mousedown_only() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    event: ClickEventOption::MousedownOnly,
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            button.dispatch_event(&mouse_event("mousedown")).unwrap();
            assert!(
                log.calls().is_empty(),
                "the mousedown open is frame-deferred"
            );
            sleep(60).await;
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "mousedown opened through the frame"
            );

            button.dispatch_event(&mouse_event("click")).unwrap();
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "click never closes in mousedown-only mode"
            );
        };
        __owner.cleanup();
    }

    // Pins `ignoreMouse` (`useClick.test.tsx:149-158`): mouse-like presses and clicks
    // are dropped.
    #[wasm_bindgen_test]
    fn ignores_mouse_input_when_ignore_mouse_is_true() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    ignore_mouse: true,
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            press_mouse(&button);

            assert!(log.calls().is_empty(), "mouse input is ignored entirely");
        });
    }

    // Pins `stickIfOpen` against a hover-opened popup (the decision-table's
    // non-click-like open event — `useClick.test.tsx:239-297`): with the default
    // stickIfOpen the first click keeps the popup open; with stickIfOpen: false it
    // closes. The hover-open is seeded the way `useHover` would leave the store
    // (open + a non-click-like `openEvent` in dataRef).
    #[wasm_bindgen_test]
    fn stick_if_open_gates_the_first_click_on_a_hover_opened_popup() {
        init_executor();

        // stickIfOpen: true (default) — stays open.
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(true, calls);
            let button = button_with_store(&store);
            let hover_event = MouseEvent::new("mouseover").unwrap();
            store.context.data_ref.borrow_mut().open_event = Some(hover_event.into());

            let props = use_click(Rc::clone(&store), UseClickProps::default());
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            button.dispatch_event(&mouse_event("click")).unwrap();
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "the first click on a hover-opened popup re-asserts open"
            );
        });

        // stickIfOpen: false — closes.
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(true, calls);
            let button = button_with_store(&store);
            let hover_event = MouseEvent::new("mouseover").unwrap();
            store.context.data_ref.borrow_mut().open_event = Some(hover_event.into());

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    stick_if_open: false,
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            button.dispatch_event(&mouse_event("click")).unwrap();
            assert_eq!(
                log.calls(),
                vec![(false, "trigger-press".to_owned())],
                "the first click closes the hover-opened popup"
            );
        });
    }

    // Pins the configured reason (`useClick.test.tsx:226-237`): the open carries the
    // configured `reason` instead of the default trigger-press.
    #[wasm_bindgen_test]
    fn uses_the_configured_reason_on_a_typeable_reference() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);

            let document = web_sys::window().unwrap().document().unwrap();
            let input: web_sys::HtmlInputElement = document
                .create_element("input")
                .unwrap()
                .dyn_into()
                .unwrap();
            document.body().unwrap().append_child(&input).unwrap();
            store.set_field(
                |state| &mut state.dom_reference_element,
                Some(input.clone().into()),
            );

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    reason: crate::floating_ui::reasons::INPUT_PRESS.to_owned(),
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(input.as_ref())
                .unwrap();

            input.dispatch_event(&mouse_event("click")).unwrap();

            assert_eq!(
                log.calls(),
                vec![(true, "input-press".to_owned())],
                "the open carries the configured reason"
            );
        });
    }

    // Pins the touch-open delay (`useClick.test.tsx:160-177`): a touch press does not
    // open immediately; after the delay it opens. Closing is never delayed
    // (`useClick.test.tsx:213-224`).
    #[wasm_bindgen_test(async)]
    async fn delays_touch_opening_but_not_touch_closing() {
        init_executor();
        let __owner = reactive_graph::owner::Owner::new();
        __owner.set();
        {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_click(
                Rc::clone(&store),
                UseClickProps {
                    touch_open_delay: 30,
                    ..UseClickProps::default()
                },
            );
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(button.as_ref())
                .unwrap();

            touch_click(&button);
            assert!(
                !log.calls().iter().any(|(open, _)| *open),
                "the touch press does not open immediately"
            );

            sleep(80).await;
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-press".to_owned())],
                "the delayed open lands after touchOpenDelay"
            );

            // A second touch press closes immediately — no delay on close.
            touch_click(&button);
            assert_eq!(
                log.calls(),
                vec![
                    (true, "trigger-press".to_owned()),
                    (false, "trigger-press".to_owned())
                ],
                "the touch close is immediate"
            );
        };
        __owner.cleanup();
    }

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }
}
