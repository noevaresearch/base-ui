//! Port of `packages/react/src/utils/useMixedToggleClickHandler.ts` — the
//! click-suppression pair for triggers of popups toggled by different events: "a button
//! that opens a popup on mousedown and closes it on click. This hook prevents the popup
//! from closing immediately after the mouse button is released"
//! (`useMixedToggleClickHandler.ts:7-11`).
//!
//! Upstream is one hook (`useMixedToggleClickHandler.ts:12-43`): an `ignoreClickRef`
//! (`:14`) plus a returned `{ onMouseDown, onClick }` bag (`:21-41`). `onMouseDown`
//! arms the flag when the mousedown would *change* the open state —
//! `(mouseDownAction === 'open' && !open) || (mouseDownAction === 'close' && open)`
//! (`:23`) — and simultaneously arms a document-level `{ once: true }` `click` listener
//! that clears it (`:26-32`); `onClick` consumes the flag and calls
//! `event.preventBaseUIHandler()` (`:36-39`), so the closing half of the mixed toggle
//! does not immediately reopen. The once-listener is what self-cleans the flag — there
//! is no effect bookkeeping (implementation spec, "Interaction-type & click-semantics
//! hooks": "the ignore-click suppression window … no test").
//!
//! ## Rust adaptations
//!
//! - `ignoreClickRef` (`:14`) is an [`Rc<Cell<bool>>`]; both handlers share it, the
//!   same one-instance-per-hook lifetime upstream's ref has.
//! - The `{ once: true }` document listener (`:26-32`) ports to
//!   [`leptos_ui_utils::add_event_listener_with_options`] with
//!   `AddEventListenerOptions::new().once(true)` — the browser's own once-handling is
//!   the self-cleanup, exactly upstream's mechanism. The returned unsubscribe handle is
//!   deliberately forgotten (`std::mem::forget`): dropping it would *remove* the
//!   listener (that crate's documented drop semantics), while upstream never
//!   proactively removes the once-listener — it self-removes on the first click through
//!   the browser's `once` handling, and stays armed until then if no click ever comes.
//!   Forgetting the handle reproduces that lifetime (a listener registration and its
//!   closure per armed mousedown, the same unbounded arming upstream has).
//! - `open` is a reactive source read untracked inside the handlers — upstream reads
//!   the render-time `open` prop through the `useMemo` closure (`:23`), the latest
//!   value at dispatch time (the `use_button` port's `disabled` convention). The
//!   `useMemo` re-creation on `[enabled, mouseDownAction, open]` (`:42`) has no Rust
//!   shape: `enabled`/`mouseDownAction` are component-shape props (static in the port's
//!   run-once model), and the `open` read is live instead of per-render.
//! - The `enabled` early return (`:17-19`) returns both slots `None` — upstream's
//!   `EMPTY_OBJECT` bag.
//! - The `onClick` slot is [`BaseUIEvent`]-typed (`:35` takes the
//!   `BaseUIEvent<React.MouseEvent>` wrapper) so the suppression is the shared
//!   prevention mark downstream composed chains observe.
//! - `UseMixedToggleClickHandlerState` (`:61`) is the empty record upstream's State
//!   type declares; the port's bag carries it implicitly (no state members).
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.

use std::cell::Cell;
use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked};
use web_sys::MouseEvent;
use web_sys::wasm_bindgen::JsCast;

use leptos_ui_utils::add_event_listener_with_options;
use leptos_ui_utils::owner_document;

use crate::floating_ui::element_props::ElementEventHandler;
use crate::types::BaseUIEvent;

/// Upstream's `mouseDownAction` union (`useMixedToggleClickHandler.ts:52-54`): which
/// action the mousedown half performs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixedToggleMouseDownAction {
    /// The mousedown opens the popup.
    Open,
    /// The mousedown closes the popup.
    Close,
}

/// `UseMixedToggleClickHandlerParameters` (`useMixedToggleClickHandler.ts:45-59`), with
/// upstream's documented defaults noted per field.
pub struct UseMixedToggleClickHandlerParams<O> {
    /// `enabled` (`:47` — upstream default `true`).
    pub enabled: bool,
    /// `mouseDownAction` (`:52`).
    pub mouse_down_action: MixedToggleMouseDownAction,
    /// `open` (`:56`) — the current open state of the popup, a reactive source.
    pub open: O,
}

/// The returned `{ onMouseDown, onClick }` bag (`useMixedToggleClickHandler.ts:21-41`).
/// Both slots are `None` when `enabled` is false — upstream's `EMPTY_OBJECT` (`:17-19`).
#[derive(Clone, Default)]
pub struct MixedToggleClickHandlers {
    /// `onMouseDown` (`:22`) — arms the suppression window.
    pub on_mouse_down: Option<ElementEventHandler<MouseEvent>>,
    /// `onClick` (`:35`) — consumes the window through the shared prevention mark.
    pub on_click: Option<ElementEventHandler<BaseUIEvent<MouseEvent>>>,
}

/// The arming decision (`useMixedToggleClickHandler.ts:23`) as a pure function:
/// `(mouseDownAction === 'open' && !open) || (mouseDownAction === 'close' && open)`.
fn should_ignore_click(mouse_down_action: MixedToggleMouseDownAction, open: bool) -> bool {
    match mouse_down_action {
        MixedToggleMouseDownAction::Open => !open,
        MixedToggleMouseDownAction::Close => open,
    }
}

/// Port of `useMixedToggleClickHandler` (`useMixedToggleClickHandler.ts:12-43`).
pub fn use_mixed_toggle_click_handler<O>(
    params: UseMixedToggleClickHandlerParams<O>,
) -> MixedToggleClickHandlers
where
    O: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    let UseMixedToggleClickHandlerParams {
        enabled,
        mouse_down_action,
        open,
    } = params;

    // `if (!enabled) { return EMPTY_OBJECT; }` (`:17-19`).
    if !enabled {
        return MixedToggleClickHandlers::default();
    }

    // `const ignoreClickRef = React.useRef(false)` (`:14`).
    let ignore_click: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // `onMouseDown` (`:22-34`).
    let on_mouse_down: ElementEventHandler<MouseEvent> = {
        let ignore_click = Rc::clone(&ignore_click);
        let open = open.clone();
        Rc::new(move |event: &MouseEvent| {
            if should_ignore_click(mouse_down_action, open.get_untracked()) {
                ignore_click.set(true);

                // `ownerDocument(event.currentTarget).addEventListener('click', () => {
                // ignoreClickRef.current = false; }, { once: true })` (`:26-32`) — the
                // browser's once-handling is the self-cleanup; the handle is forgotten
                // because dropping it would remove the listener (see the module docs).
                let document = owner_document(
                    event
                        .current_target()
                        .as_ref()
                        .and_then(|target| target.dyn_ref::<web_sys::Node>()),
                );
                let clear = Rc::clone(&ignore_click);
                let options = web_sys::AddEventListenerOptions::new();
                options.set_once(true);
                let unsubscribe = add_event_listener_with_options(
                    &document,
                    "click",
                    move |_| clear.set(false),
                    options,
                );
                std::mem::forget(unsubscribe);
            }
        })
    };

    // `onClick` (`:35-40`).
    let on_click: ElementEventHandler<BaseUIEvent<MouseEvent>> = {
        let ignore_click = Rc::clone(&ignore_click);
        Rc::new(move |event: &BaseUIEvent<MouseEvent>| {
            if ignore_click.get() {
                ignore_click.set(false);
                event.prevent_base_ui_handler();
            }
        })
    };

    MixedToggleClickHandlers {
        on_mouse_down: Some(on_mouse_down),
        on_click: Some(on_click),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // The arming decision (`:23`) across every `mouseDownAction` × `open` cell — no
    // upstream test exists (implementation spec, "Anything in source not explained by
    // any test"); this pins the documented matrix, with the dispatch flow covered in
    // the wasm suite.
    #[test]
    fn the_suppression_window_arms_only_on_state_changing_mousedowns() {
        // `mouseDownAction: 'open'` arms when closed (the mousedown would open).
        assert!(should_ignore_click(MixedToggleMouseDownAction::Open, false));
        assert!(!should_ignore_click(MixedToggleMouseDownAction::Open, true));
        // `mouseDownAction: 'close'` arms when open (the mousedown would close).
        assert!(should_ignore_click(MixedToggleMouseDownAction::Close, true));
        assert!(!should_ignore_click(
            MixedToggleMouseDownAction::Close,
            false
        ));
    }

    // The `enabled` early return (`:17-19`): `EMPTY_OBJECT` upstream — no slots here.
    #[test]
    fn disabled_yields_the_empty_bag() {
        let handlers = use_mixed_toggle_click_handler(UseMixedToggleClickHandlerParams {
            enabled: false,
            mouse_down_action: MixedToggleMouseDownAction::Open,
            open: reactive_graph::signal::RwSignal::new(false),
        });
        assert!(handlers.on_mouse_down.is_none());
        assert!(handlers.on_click.is_none());
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use leptos_ui_utils::add_event_listener;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect("no window")
            .document()
            .expect("no document")
    }

    /// The mounted trigger: the bag attached through the same native-listener seam the
    /// view layer's JSX spread performs, with the armed-window invocations logged.
    struct Trigger {
        element: web_sys::Element,
        _cleanups: Vec<leptos_ui_utils::EventListenerUnsubscribe>,
    }

    fn mount(
        enabled: bool,
        mouse_down_action: MixedToggleMouseDownAction,
        open: &RwSignal<bool>,
    ) -> (Trigger, Rc<RefCell<Vec<bool>>>) {
        let handlers = use_mixed_toggle_click_handler(UseMixedToggleClickHandlerParams {
            enabled,
            mouse_down_action,
            open: open.clone(),
        });

        let element = document().create_element("div").expect("create_element");
        document()
            .body()
            .expect("a body")
            .append_child(&element)
            .unwrap();

        // A composed consumer observing the shared prevention mark — the downstream
        // click handler the suppression exists to gate (`:36-39`).
        let consumer_marks: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let mut cleanups = Vec::new();

        if let Some(on_mouse_down) = handlers.on_mouse_down {
            let unsubscribe = add_event_listener(&element, "mousedown", move |event| {
                if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                    on_mouse_down(mouse_event);
                }
            });
            cleanups.push(unsubscribe);
        }

        if let Some(on_click) = handlers.on_click {
            let consumer_marks_for_click = Rc::clone(&consumer_marks);
            let unsubscribe = add_event_listener(&element, "click", move |event| {
                if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                    let wrapped = BaseUIEvent::new(mouse_event.clone());
                    on_click(&wrapped);
                    consumer_marks_for_click
                        .borrow_mut()
                        .push(wrapped.base_ui_handler_prevented());
                }
            });
            cleanups.push(unsubscribe);
        }

        (
            Trigger {
                element,
                _cleanups: cleanups,
            },
            consumer_marks,
        )
    }

    fn fire_mouse(element: &web_sys::Element, event_type: &str) {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        let event =
            MouseEvent::new_with_mouse_event_init_dict(event_type, &init).expect("event failed");
        element
            .dispatch_event(event.as_ref())
            .expect("dispatch failed");
    }

    // The documented flow (`:7-11`): a button that opens on mousedown — the closing
    // click is suppressed so the popup does not immediately reopen.
    #[wasm_bindgen_test]
    fn the_closing_click_of_a_mousedown_toggled_popup_is_suppressed() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let open: RwSignal<bool> = RwSignal::new(false);
        let (trigger, marks) = mount(true, MixedToggleMouseDownAction::Open, &open);

        // The opening mousedown arms the window (`:23`, open-action while closed).
        fire_mouse(&trigger.element, "mousedown");
        // The release click is suppressed — the consumer observes the prevention mark.
        fire_mouse(&trigger.element, "click");
        assert_eq!(
            &*marks.borrow(),
            &[true],
            "the post-mousedown click is suppressed via preventBaseUIHandler"
        );

        // The once-listener consumed itself: a later click is not suppressed.
        fire_mouse(&trigger.element, "click");
        assert_eq!(
            &*marks.borrow(),
            &[true, false],
            "the suppression window is one click wide"
        );

        owner.cleanup();
    }

    // The mirrored flow: a popup that closes on mousedown — the reopening click after
    // the close-mousedown is suppressed (`:23`, close-action while open).
    #[wasm_bindgen_test]
    fn the_reopening_click_of_a_close_on_mousedown_popup_is_suppressed() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let open: RwSignal<bool> = RwSignal::new(true);
        let (trigger, marks) = mount(true, MixedToggleMouseDownAction::Close, &open);

        fire_mouse(&trigger.element, "mousedown");
        fire_mouse(&trigger.element, "click");
        assert_eq!(&*marks.borrow(), &[true]);

        owner.cleanup();
    }

    // The no-op cells: a mousedown that would not change the open state does not arm
    // the window (`:23` — both disjuncts false).
    #[wasm_bindgen_test]
    fn state_preserving_mousedowns_do_not_arm_the_window() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        // open-action while already open.
        let open: RwSignal<bool> = RwSignal::new(true);
        let (trigger, marks) = mount(true, MixedToggleMouseDownAction::Open, &open);
        fire_mouse(&trigger.element, "mousedown");
        fire_mouse(&trigger.element, "click");
        assert_eq!(&*marks.borrow(), &[false]);

        // close-action while already closed.
        let open: RwSignal<bool> = RwSignal::new(false);
        let (trigger, marks) = mount(true, MixedToggleMouseDownAction::Close, &open);
        fire_mouse(&trigger.element, "mousedown");
        fire_mouse(&trigger.element, "click");
        assert_eq!(&*marks.borrow(), &[false]);

        owner.cleanup();
    }

    // The window is armed at the document level (`:26`): any click clears it, even one
    // that misses the trigger — and the armed window survives across unrelated
    // dispatches until a click arrives.
    #[wasm_bindgen_test]
    fn the_once_window_lives_on_the_document_and_clears_on_any_click() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let open: RwSignal<bool> = RwSignal::new(false);
        let (trigger, marks) = mount(true, MixedToggleMouseDownAction::Open, &open);
        let elsewhere = document().create_element("span").expect("create_element");
        document()
            .body()
            .expect("a body")
            .append_child(&elsewhere)
            .unwrap();

        fire_mouse(&trigger.element, "mousedown");
        // A click elsewhere reaches the document listener first and consumes the
        // once-window; the trigger click is then not suppressed.
        fire_mouse(&elsewhere, "click");
        fire_mouse(&trigger.element, "click");
        assert_eq!(
            &*marks.borrow(),
            &[false],
            "a click outside the trigger consumed the once-window"
        );

        owner.cleanup();
    }

    // The live `open` read: the same trigger's arming follows the signal's current
    // value (upstream's per-render `open` closure at `:23`).
    #[wasm_bindgen_test]
    fn the_arming_follows_the_live_open_state() {
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let open: RwSignal<bool> = RwSignal::new(false);
        let (trigger, marks) = mount(true, MixedToggleMouseDownAction::Open, &open);

        // Closed → open-action arms.
        fire_mouse(&trigger.element, "mousedown");
        fire_mouse(&trigger.element, "click");
        assert_eq!(&*marks.borrow(), &[true]);

        // Open → open-action does not arm.
        open.set(true);
        fire_mouse(&trigger.element, "mousedown");
        fire_mouse(&trigger.element, "click");
        assert_eq!(&*marks.borrow(), &[true, false]);
        assert_eq!(open.get_untracked(), true);

        owner.cleanup();
    }
}
