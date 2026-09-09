//! Port of the per-element prop-bag vocabulary the interaction hooks return — upstream
//! `ElementProps` (`packages/react/src/floating-ui-react/types.ts:147-152`) whose four
//! roles hold `React.HTMLProps` event-handler bags — plus the hook-parameter
//! normalization `FloatingRootContext | FloatingContext`
//! (`types.ts:122,124-137`; the composition recipe at
//! `specs/library/floating-ui-react/implementation.md`, "Cross-hook communication
//! pattern" step 1).
//!
//! ## Rust adaptations
//!
//! - React's synthetic-event props (`onPointerDown`, `onMouseDown`, …) become typed
//!   handler slots carrying the **native** DOM event (`web_sys`), since the port has no
//!   synthetic-event layer (the `isReactEvent` note in
//!   [`crate::floating_ui::event`]'s module docs). The slot set is the events the
//!   unit's interaction hooks actually attach (useClick: pointerdown/mousedown/click/
//!   keydown; useFocus: focus/blur/mouseleave; useClientPoint: pointer/pointer-enter/
//!   mouse-move/mouse-enter) — upstream's `React.HTMLAttributes` is an open bag whose
//!   non-handler members the hooks never fill.
//! - The DOM event name each slot attaches under is React's own delegation mapping:
//!   `onFocus`/`onBlur` ride the **bubbling** `focusin`/`focusout` (React 17+
//!   delegation — the hook logic reading `event.currentTarget` on a focused *child*
//!   relies on it), while `mouseenter`/`mouseleave`/`pointerenter` keep their native
//!   non-bubbling names (React's synthetic equivalents have the same non-bubbling
//!   semantics). Everything else maps 1:1.
//! - The `*Capture` slots (useDismiss's floating bag —
//!   `hooks/useDismiss.ts:786-797`) attach under the same event names in the
//!   **capture** phase, React's `on*Capture` → `{ capture: true }` mapping. The touch
//!   slots (`onTouchEndCapture`/`onTouchMoveCapture`) carry the native
//!   `web_sys::TouchEvent` (no synthetic layer).
//! - Non-handler members of the bag (`useListNavigation`'s
//!   `aria-activedescendant`, `hooks/useListNavigation.ts:756-764`) are React prop
//!   bags too — the port carries them in [`ElementHandlers::attributes`], one
//!   `(name, value)` entry per attribute whose value resolves lazily (the port's hook
//!   runs once, so a static `String` would freeze the render-time value; the closure
//!   re-reads the reactive sources at read time, the way a React re-render would
//!   re-derive the prop). [`ElementHandlers::attach_to`] does not attach them:
//!   attribute *delivery* is the view layer's job (the composition work upstream's
//!   consumer components do when spreading `getFloatingProps()` onto the element —
//!   `specs/architecture.md`, "Prop / class / style merging (mergeProps)").
//! - [`ElementHandlers::attach_to`] registers every filled slot on a target as native
//!   listeners and returns a merged cleanup — the composition work upstream's consumer
//!   components do when spreading `getReferenceProps()`/`getFloatingProps()` onto
//!   elements (there is no JSX spread to lean on). A slot whose dispatched event does
//!   not carry the slot's event interface (e.g. a plain `MouseEvent` for a
//!   `pointerdown` slot) is skipped, mirroring how a React consumer simply would not
//!   fire that synthetic prop; a subtype event (a real `PointerEvent` for a
//!   `mousedown` slot) is delivered.
//! - `FloatingRootContext | FloatingContext` (`types.ts:122,124-137`) becomes
//!   [`FloatingContextSource`]: upstream normalizes with
//!   `'rootStore' in context ? context.rootStore : context`
//!   (`hooks/useClick.ts:73` and identically in every interaction hook); the port's
//!   [`FloatingContextSource::root_store`] is that line.

use std::rc::Rc;

use web_sys::wasm_bindgen::JsCast;
use web_sys::{
    Event, EventTarget, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, TouchEvent,
};

use leptos_ui_utils::merge_cleanups;
use leptos_ui_utils::merge_cleanups::CleanupFn;

use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::types::FloatingContext;

/// One typed event-handler slot — a native-event callback shared by `Rc` so handler
/// bags clone cheaply and identity-stably (upstream handlers are stable function
/// references inside the hook's `useMemo` bags, e.g. `hooks/useClick.ts:81-231`).
pub type ElementEventHandler<E> = Rc<dyn Fn(&E)>;

/// One non-handler member of the bag — upstream's plain attribute props (the
/// `'aria-activedescendant': ...` entry of `useListNavigation`'s
/// `ariaActiveDescendantProp`, `hooks/useListNavigation.ts:756-764`). The value
/// resolves lazily so a reactive source re-reads at read time (see the module docs);
/// `None` is upstream's omitted prop (the `&&`-gated spread arm).
pub type ElementAttributeFn = Rc<dyn Fn() -> Option<String>>;

/// The event-handler bag for one element role — upstream's `React.HTMLProps`
/// member of `ElementProps` (`types.ts:147-152`), narrowed to the events the unit's
/// interaction hooks attach (see the module docs).
#[derive(Clone, Default)]
pub struct ElementHandlers {
    /// `onPointerDown` — [`crate::floating_ui::use_click`] and
    /// [`crate::floating_ui::use_client_point`] reference handlers.
    pub on_pointer_down: Option<ElementEventHandler<PointerEvent>>,
    /// `onMouseDown` — [`crate::floating_ui::use_click`]'s mousedown event path.
    pub on_mouse_down: Option<ElementEventHandler<MouseEvent>>,
    /// `onClick` — [`crate::floating_ui::use_click`]'s click event path.
    pub on_click: Option<ElementEventHandler<MouseEvent>>,
    /// `onKeyDown` — [`crate::floating_ui::use_click`] resets its pointer-type memory.
    pub on_key_down: Option<ElementEventHandler<KeyboardEvent>>,
    /// `onFocus` — [`crate::floating_ui::use_focus`]'s open path. Attached as
    /// `focusin` (see the module docs).
    pub on_focus: Option<ElementEventHandler<FocusEvent>>,
    /// `onBlur` — [`crate::floating_ui::use_focus`]'s close path. Attached as
    /// `focusout` (see the module docs).
    pub on_blur: Option<ElementEventHandler<FocusEvent>>,
    /// `onMouseEnter` — [`crate::floating_ui::use_client_point`]'s enter/move path.
    pub on_mouse_enter: Option<ElementEventHandler<MouseEvent>>,
    /// `onMouseLeave` — [`crate::floating_ui::use_focus`] resets blocked focus.
    pub on_mouse_leave: Option<ElementEventHandler<MouseEvent>>,
    /// `onMouseMove` — [`crate::floating_ui::use_client_point`]'s enter/move path.
    pub on_mouse_move: Option<ElementEventHandler<MouseEvent>>,
    /// `onPointerEnter` — [`crate::floating_ui::use_client_point`] records the
    /// pointer type.
    pub on_pointer_enter: Option<ElementEventHandler<PointerEvent>>,
    /// `onPointerMove` — [`crate::floating_ui::use_list_navigation`]'s floating-role
    /// pointer-modality tracker (`hooks/useListNavigation.ts:792-797`).
    pub on_pointer_move: Option<ElementEventHandler<PointerEvent>>,
    /// `onPointerLeave` — [`crate::floating_ui::use_list_navigation`]'s item-role
    /// reset path (`hooks/useListNavigation.ts:707-741`). Attached under the native
    /// non-bubbling `pointerleave` (see the module docs).
    pub on_pointer_leave: Option<ElementEventHandler<PointerEvent>>,
    /// `onClickCapture` — [`crate::floating_ui::use_dismiss`]'s inside-tree marker.
    /// Attached in the capture phase (see the module docs).
    pub on_click_capture: Option<ElementEventHandler<MouseEvent>>,
    /// `onMouseDownCapture` — [`crate::floating_ui::use_dismiss`]'s inside-tree and
    /// press-start markers. Attached in the capture phase.
    pub on_mouse_down_capture: Option<ElementEventHandler<MouseEvent>>,
    /// `onPointerDownCapture` — [`crate::floating_ui::use_dismiss`]'s inside-tree and
    /// press-start markers. Attached in the capture phase.
    pub on_pointer_down_capture: Option<ElementEventHandler<PointerEvent>>,
    /// `onMouseUpCapture` — [`crate::floating_ui::use_dismiss`]'s inside-tree marker.
    /// Attached in the capture phase.
    pub on_mouse_up_capture: Option<ElementEventHandler<MouseEvent>>,
    /// `onTouchEndCapture` — [`crate::floating_ui::use_dismiss`]'s inside-tree marker.
    /// Attached in the capture phase.
    pub on_touch_end_capture: Option<ElementEventHandler<TouchEvent>>,
    /// `onTouchMoveCapture` — [`crate::floating_ui::use_dismiss`]'s inside-tree marker.
    /// Attached in the capture phase.
    pub on_touch_move_capture: Option<ElementEventHandler<TouchEvent>>,
    /// The bag's non-handler attribute members (see the module docs) — upstream the
    /// plain attribute props spread alongside the handlers
    /// (`hooks/useListNavigation.ts:766-768,930-935`).
    pub attributes: Vec<(String, ElementAttributeFn)>,
}

impl ElementHandlers {
    /// Attaches every filled slot to `target` as a bubble-phase native listener under
    /// the DOM event name React's synthetic prop maps to (see the module docs), and
    /// returns one merged cleanup — `None` when the bag is empty. The cleanup mirrors
    /// React's unmount-time listener removal; `removeEventListener` on a target that
    /// already dropped the listener is a browser-side no-op, so stale cleanups are safe.
    pub fn attach_to(&self, target: &EventTarget) -> Option<CleanupFn> {
        let mut cleanups: Vec<Option<CleanupFn>> = Vec::new();

        macro_rules! attach_bubble {
            ($slot:expr, $event_name:literal, $event_type:ty) => {
                if let Some(handler) = &$slot {
                    let handler = Rc::clone(handler);
                    let unsubscribe = leptos_ui_utils::add_event_listener(
                        target,
                        $event_name,
                        move |event: &Event| {
                            // The slot's own event name dispatches its own interface;
                            // a non-matching dispatch is skipped (see the module docs).
                            if let Some(typed) = event.dyn_ref::<$event_type>() {
                                handler(typed);
                            }
                        },
                    );
                    cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
                }
            };
        }

        macro_rules! attach_capture {
            ($slot:expr, $event_name:literal, $event_type:ty) => {
                if let Some(handler) = &$slot {
                    let handler = Rc::clone(handler);
                    let unsubscribe = leptos_ui_utils::add_event_listener_with_options(
                        target,
                        $event_name,
                        move |event: &Event| {
                            if let Some(typed) = event.dyn_ref::<$event_type>() {
                                handler(typed);
                            }
                        },
                        true,
                    );
                    cleanups.push(Some(Box::new(move || unsubscribe.unsubscribe())));
                }
            };
        }

        macro_rules! attach {
            ($slot:expr, $event_name:literal, $event_type:ty) => {
                attach_bubble!($slot, $event_name, $event_type);
            };
            ($slot:expr, $event_name:literal, $event_type:ty, capture) => {
                attach_capture!($slot, $event_name, $event_type);
            };
        }

        attach!(self.on_pointer_down, "pointerdown", PointerEvent);
        attach!(self.on_mouse_down, "mousedown", MouseEvent);
        attach!(self.on_click, "click", MouseEvent);
        attach!(self.on_key_down, "keydown", KeyboardEvent);
        attach!(self.on_focus, "focusin", FocusEvent);
        attach!(self.on_blur, "focusout", FocusEvent);
        attach!(self.on_mouse_enter, "mouseenter", MouseEvent);
        attach!(self.on_mouse_leave, "mouseleave", MouseEvent);
        attach!(self.on_mouse_move, "mousemove", MouseEvent);
        attach!(self.on_pointer_enter, "pointerenter", PointerEvent);
        attach!(self.on_pointer_move, "pointermove", PointerEvent);
        attach!(self.on_pointer_leave, "pointerleave", PointerEvent);
        attach!(self.on_click_capture, "click", MouseEvent, capture);
        attach!(self.on_mouse_down_capture, "mousedown", MouseEvent, capture);
        attach!(
            self.on_pointer_down_capture,
            "pointerdown",
            PointerEvent,
            capture
        );
        attach!(self.on_mouse_up_capture, "mouseup", MouseEvent, capture);
        attach!(self.on_touch_end_capture, "touchend", TouchEvent, capture);
        attach!(self.on_touch_move_capture, "touchmove", TouchEvent, capture);

        let merged = merge_cleanups(cleanups);
        (!self.is_empty()).then(|| Box::new(merged) as CleanupFn)
    }

    /// Whether no slot is filled — the `EMPTY_OBJECT` bags upstream hooks return when
    /// disabled (`hooks/useClick.ts:233`, `hooks/useFocus.ts:247-248`).
    pub fn is_empty(&self) -> bool {
        self.on_pointer_down.is_none()
            && self.on_mouse_down.is_none()
            && self.on_click.is_none()
            && self.on_key_down.is_none()
            && self.on_focus.is_none()
            && self.on_blur.is_none()
            && self.on_mouse_enter.is_none()
            && self.on_mouse_leave.is_none()
            && self.on_mouse_move.is_none()
            && self.on_pointer_enter.is_none()
            && self.on_pointer_move.is_none()
            && self.on_pointer_leave.is_none()
            && self.on_click_capture.is_none()
            && self.on_mouse_down_capture.is_none()
            && self.on_pointer_down_capture.is_none()
            && self.on_mouse_up_capture.is_none()
            && self.on_touch_end_capture.is_none()
            && self.on_touch_move_capture.is_none()
            && self.attributes.is_empty()
    }
}

/// Port of `ElementProps` (`types.ts:147-152`): the per-role handler bags an
/// interaction hook returns. A role left `None` is upstream's `undefined` role — the
/// consumer attaches nothing for it.
#[derive(Clone, Default)]
pub struct ElementProps {
    /// `reference` (`types.ts:148`).
    pub reference: Option<ElementHandlers>,
    /// `floating` (`types.ts:149`).
    pub floating: Option<ElementHandlers>,
    /// `item` (`types.ts:150`).
    pub item: Option<ElementHandlers>,
    /// `trigger` (`types.ts:151`) — the Base UI extension the trigger-facing hooks
    /// fill alongside `reference` (useFocus/useClientPoint return
    /// `{ reference, trigger: reference }`, `hooks/useFocus.ts:247-250`,
    /// `hooks/useClientPoint.ts:256-259`).
    pub trigger: Option<ElementHandlers>,
}

/// Port of the hooks' `context: FloatingRootContext | FloatingContext` parameter
/// (`types.ts:122,124-137`) with the normalization every interaction hook performs —
/// upstream `'rootStore' in context ? context.rootStore : context`
/// (`hooks/useClick.ts:73` and identically across the hooks).
pub enum FloatingContextSource {
    /// The `FloatingRootContext` arm — the store itself (`types.ts:122`).
    Store(Rc<FloatingRootStore>),
    /// The `FloatingContext` arm — the context constructed around a store
    /// (`types.ts:124-137`).
    Context(Rc<FloatingContext>),
}

impl FloatingContextSource {
    /// The normalization result — the store handle the hook operates on
    /// (`hooks/useClick.ts:73`).
    pub fn root_store(&self) -> Rc<FloatingRootStore> {
        match self {
            FloatingContextSource::Store(store) => Rc::clone(store),
            FloatingContextSource::Context(context) => Rc::clone(&context.root_store),
        }
    }
}

impl From<Rc<FloatingRootStore>> for FloatingContextSource {
    fn from(store: Rc<FloatingRootStore>) -> Self {
        FloatingContextSource::Store(store)
    }
}

impl From<Rc<FloatingContext>> for FloatingContextSource {
    fn from(context: Rc<FloatingContext>) -> Self {
        FloatingContextSource::Context(context)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    use reactive_graph::wrappers::read::Signal;
    use send_wrapper::SendWrapper;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::use_floating::use_base_ui_floating;
    use crate::floating_ui::use_position::UsePositionOptions;

    // The engine's effects need the global executor + an owner (the use_floating
    // host-test setup).
    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn empty_store() -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        })
    }

    fn position_options() -> UsePositionOptions {
        UsePositionOptions {
            placement: Signal::derive(|| floating_ui_dom::Placement::Bottom),
            strategy: Signal::derive(|| floating_ui_dom::Strategy::Absolute),
            middleware: Signal::derive(|| SendWrapper::new(Vec::new())),
            transform: Signal::derive(|| true),
            while_elements_mounted: None,
        }
    }

    // Pins the handler-bag defaulting (`types.ts:147-152` — roles are optional; a
    // disabled hook returns the EMPTY_OBJECT bag, `hooks/useClick.ts:233`): a default
    // bag has no slots filled and `is_empty` agrees.
    #[test]
    fn a_default_handler_bag_has_no_slots_filled() {
        let bag = ElementHandlers::default();
        assert!(bag.is_empty(), "the default bag is the EMPTY_OBJECT shape");

        let bag = ElementHandlers {
            on_key_down: Some(Rc::new(|_: &KeyboardEvent| {})),
            ..ElementHandlers::default()
        };
        assert!(!bag.is_empty(), "one filled slot makes the bag non-empty");
    }

    // Pins the hook-parameter normalization (`hooks/useClick.ts:73` —
    // `'rootStore' in context ? context.rootStore : context`): both arms resolve to
    // the same store handle the context was built around.
    #[test]
    fn the_context_source_normalizes_to_the_same_store_from_both_arms() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = empty_store();

            let from_store = FloatingContextSource::from(Rc::clone(&store));
            assert!(Rc::ptr_eq(&from_store.root_store(), &store));

            // The FloatingContext arm: the real context constructed around the store
            // (`hooks/useFloating.ts:268` — `root_store: Rc::clone(&store)`), so the
            // normalization lands on the identical handle.
            let result = use_base_ui_floating(position_options(), Rc::clone(&store));
            let from_context = FloatingContextSource::from(Rc::clone(&result.context));

            assert!(Rc::ptr_eq(&from_context.root_store(), &store));
        });
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::cell::Cell;

    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn button() -> web_sys::HtmlElement {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap()
            .dyn_into()
            .unwrap()
    }

    // Pins the slot → DOM event name mapping (the module docs — React's delegation
    // mapping): each filled slot receives its own event type when the mapped event
    // dispatches on the attached target, and the merged cleanup removes them all.
    #[wasm_bindgen_test]
    fn attach_to_delivers_each_slot_its_mapped_event_and_cleanup_removes_them() {
        let target = button();

        let pointer_downs = Rc::new(Cell::new(0u8));
        let clicks = Rc::new(Cell::new(0u8));
        let focus_ins = Rc::new(Cell::new(0u8));
        let mouse_enters = Rc::new(Cell::new(0u8));

        let bag = ElementHandlers {
            on_pointer_down: {
                let counter = Rc::clone(&pointer_downs);
                Some(Rc::new(move |event: &PointerEvent| {
                    assert_eq!(event.type_(), "pointerdown");
                    counter.set(counter.get() + 1);
                }))
            },
            on_click: {
                let counter = Rc::clone(&clicks);
                Some(Rc::new(move |event: &MouseEvent| {
                    assert_eq!(event.type_(), "click");
                    counter.set(counter.get() + 1);
                }))
            },
            on_focus: {
                let counter = Rc::clone(&focus_ins);
                Some(Rc::new(move |event: &FocusEvent| {
                    assert_eq!(
                        event.type_(),
                        "focusin",
                        "onFocus rides the bubbling focusin"
                    );
                    counter.set(counter.get() + 1);
                }))
            },
            on_mouse_enter: {
                let counter = Rc::clone(&mouse_enters);
                Some(Rc::new(move |event: &MouseEvent| {
                    assert_eq!(event.type_(), "mouseenter");
                    counter.set(counter.get() + 1);
                }))
            },
            ..ElementHandlers::default()
        };

        let cleanup = bag.attach_to(target.as_ref()).expect("slots attached");

        target
            .dispatch_event(&PointerEvent::new("pointerdown").unwrap())
            .unwrap();
        target
            .dispatch_event(&MouseEvent::new("click").unwrap())
            .unwrap();
        target
            .dispatch_event(&FocusEvent::new("focusin").unwrap())
            .unwrap();
        target
            .dispatch_event(&MouseEvent::new("mouseenter").unwrap())
            .unwrap();

        assert_eq!(pointer_downs.get(), 1);
        assert_eq!(clicks.get(), 1);
        assert_eq!(focus_ins.get(), 1);
        assert_eq!(mouse_enters.get(), 1);

        cleanup();

        target
            .dispatch_event(&PointerEvent::new("pointerdown").unwrap())
            .unwrap();
        target
            .dispatch_event(&MouseEvent::new("click").unwrap())
            .unwrap();
        target
            .dispatch_event(&FocusEvent::new("focusin").unwrap())
            .unwrap();
        target
            .dispatch_event(&MouseEvent::new("mouseenter").unwrap())
            .unwrap();

        assert_eq!(
            pointer_downs.get(),
            1,
            "the cleanup removed the pointerdown listener"
        );
        assert_eq!(clicks.get(), 1, "the cleanup removed the click listener");
        assert_eq!(
            focus_ins.get(),
            1,
            "the cleanup removed the focusin listener"
        );
        assert_eq!(
            mouse_enters.get(),
            1,
            "the cleanup removed the mouseenter listener"
        );
    }

    // Pins the empty-bag shortcut: no slots → no listeners, no cleanup needed.
    #[wasm_bindgen_test]
    fn attach_to_on_an_empty_bag_returns_no_cleanup() {
        let target = button();
        assert!(
            ElementHandlers::default()
                .attach_to(target.as_ref())
                .is_none()
        );
    }

    // Pins the non-matching-dispatch skip (the module docs): a plain MouseEvent
    // dispatched for a pointerdown slot does not reach the slot's handler, while a
    // real PointerEvent on a mousedown slot is delivered as its supertype.
    #[wasm_bindgen_test]
    fn a_mismatched_dispatch_is_skipped_and_a_subtype_is_delivered() {
        let target = button();

        let pointer_downs = Rc::new(Cell::new(0u8));
        let mouse_downs = Rc::new(Cell::new(0u8));
        let bag = ElementHandlers {
            on_pointer_down: {
                let counter = Rc::clone(&pointer_downs);
                Some(Rc::new(move |_: &PointerEvent| {
                    counter.set(counter.get() + 1)
                }))
            },
            on_mouse_down: {
                let counter = Rc::clone(&mouse_downs);
                Some(Rc::new(move |_: &MouseEvent| {
                    counter.set(counter.get() + 1)
                }))
            },
            ..ElementHandlers::default()
        };
        let _cleanup = bag.attach_to(target.as_ref()).unwrap();

        // A plain mouse event does not satisfy the pointerdown slot's interface.
        target
            .dispatch_event(&MouseEvent::new("pointerdown").unwrap())
            .unwrap();
        assert_eq!(
            pointer_downs.get(),
            0,
            "the non-PointerEvent dispatch is skipped"
        );

        // A real pointer event satisfies the mousedown slot (PointerEvent extends
        // MouseEvent) — same delivery React's synthetic layer gives its handlers.
        target
            .dispatch_event(&PointerEvent::new("mousedown").unwrap())
            .unwrap();
        assert_eq!(
            mouse_downs.get(),
            1,
            "the PointerEvent reaches the mousedown slot"
        );
    }
}
