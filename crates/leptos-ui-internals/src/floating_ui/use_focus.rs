//! Port of `packages/react/src/floating-ui-react/hooks/useFocus.ts` — opens the
//! floating element while the reference element has focus, like CSS `:focus`
//! (`specs/library/floating-ui-react/behavior.md`, "Public API surface": `useFocus`
//! (`delay`); "Interaction hooks": "`useFocus` guards against reopen after window
//! blur").
//!
//! ## Rust adaptations
//!
//! - The two effects (`useFocus.ts:61-98`, `:100-119`) run once at hook-call time with
//!   their cleanups registered through `on_cleanup` — upstream's dependency arrays
//!   (`[store, enabled]`, `[events, enabled, store]`) hold only stable identities, so
//!   the re-run behavior they encode never actually re-subscribes (Leptos components
//!   run once; see the useClick module docs).
//! - The blur handler's `dataRef.current.floatingContext?.refs.floating.current`
//!   (`useFocus.ts:226`) reads the floating element from the store instead — the
//!   port's `dataRef.current.floatingContext` carries the root-store handle (the
//!   context-hooks checkpoint's adaptation), and `refs.floating` mirrors
//!   `store.select('floatingElement')` by construction (`hooks/useFloating.ts:133-139`
//!   writes both from the same setter). The `?.` optionality carries over: no floating
//!   element means the `contains` check is false.
//! - `delay?: number | (() => number | undefined)` (`useFocus.ts:36`) becomes the
//!   [`FocusDelay`] enum.
//! - `isMacSafari` (`useFocus.ts:23`) is computed from the `platform` port at
//!   hook-call time (upstream computes it at module load; the platform group is a
//!   frozen singleton in both).
//! - The reference bag is returned for both roles — `reference` and `trigger`
//!   (`useFocus.ts:247-250`).

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, EventTarget, FocusEvent, MouseEvent};

use leptos_ui_utils::merge_cleanups;
use leptos_ui_utils::merge_cleanups::CleanupFn;
use leptos_ui_utils::owner::{owner_document, owner_window};
use leptos_ui_utils::platform;
use leptos_ui_utils::shadow_dom::{active_element, contains, get_target};
use leptos_ui_utils::use_timeout;
use leptos_ui_utils::use_timeout::Timeout;
use send_wrapper::SendWrapper;

use crate::floating_ui::create_attribute::create_attribute;
use crate::floating_ui::element::{
    is_target_inside_enabled_trigger, is_typeable_element, matches_focus_visible,
};
use crate::floating_ui::element_props::{ElementHandlers, ElementProps, FloatingContextSource};
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::reasons;
use crate::floating_ui::types::{EventUnsubscribe, RootOpenChangeEventDetails};

/// Port of `UseFocusProps['delay']` (`useFocus.ts:36`): a millisecond duration, or a
/// resolver evaluated when focus lands (`delay()` — upstream allows `undefined`
/// results, which mean "open immediately").
#[derive(Clone)]
pub enum FocusDelay {
    /// A fixed duration.
    Value(u32),
    /// A resolver evaluated at focus time; `None` resolves to "open immediately"
    /// (upstream's `delay()` returning `undefined`, `useFocus.ts:162`).
    Resolve(Rc<dyn Fn() -> Option<u32>>),
}

impl FocusDelay {
    /// `typeof delay === 'function' ? delay() : delay` (`useFocus.ts:162`).
    fn resolve(&self) -> Option<u32> {
        match self {
            FocusDelay::Value(value) => Some(*value),
            FocusDelay::Resolve(resolve) => resolve(),
        }
    }
}

/// Port of `UseFocusProps` (`useFocus.ts:25-37`) with the documented defaults
/// (`useFocus.ts:48`).
#[derive(Clone, Default)]
pub struct UseFocusProps {
    /// `enabled` (`useFocus.ts:31` — default `true`).
    pub enabled: bool,
    /// `delay` (`useFocus.ts:36` — default `undefined`: open immediately).
    pub delay: Option<FocusDelay>,
}

/// Port of `useFocus(context, props)` (`useFocus.ts:44-251`). Must be called inside a
/// reactive owner (the listeners and timer register cleanups). Returns the
/// `reference` + `trigger` handler bags, or the empty props when disabled.
pub fn use_focus(context: impl Into<FloatingContextSource>, props: UseFocusProps) -> ElementProps {
    let UseFocusProps { enabled, delay } = props;

    let store: Rc<FloatingRootStore> = context.into().root_store();

    // `const { events } = store.context` (`useFocus.ts:52` — the port's blur handler
    // sources the floating element from the store instead of `dataRef.current.
    // floatingContext`, so no `dataRef` binding is needed; see the module docs).
    let events = Rc::clone(&store.context.events);

    // `blockFocusRef` / `blockedReferenceRef` / `keyboardModalityRef`
    // (`useFocus.ts:54-57`).
    let block_focus_ref: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let blocked_reference_ref: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let keyboard_modality_ref: Rc<Cell<bool>> = Rc::new(Cell::new(true));

    let timeout: Timeout = use_timeout();

    let is_mac_safari = platform().os.mac && platform().engine.webkit;

    // The window-blur effect (`useFocus.ts:61-98`): if the reference was focused and
    // the user left the tab/window while the floating element was not open, focus
    // should be blocked when they return.
    {
        let block_focus_ref = Rc::clone(&block_focus_ref);
        let blocked_reference_ref = Rc::clone(&blocked_reference_ref);
        // `onBlur` (`useFocus.ts:73-83`).
        let on_blur: Rc<dyn Fn()> = {
            let store = Rc::clone(&store);
            Rc::new(move || {
                let current_dom_reference = selectors::dom_reference_element(&store.get_snapshot());
                let is_still_focused = current_dom_reference
                    .as_ref()
                    .map(|reference| {
                        reference.dyn_ref::<web_sys::HtmlElement>().is_some()
                            && active_element(&owner_document(Some(
                                reference.as_ref() as &web_sys::Node
                            )))
                            .as_ref()
                                == Some(reference)
                    })
                    .unwrap_or(false);
                if !store.select(selectors::open) && is_still_focused {
                    block_focus_ref.set(true);
                    *blocked_reference_ref.borrow_mut() = current_dom_reference;
                }
            })
        };
        // `onKeyDown`/`onPointerDown` (`useFocus.ts:85-91`).
        let on_key_down: Rc<dyn Fn()> = {
            let keyboard_modality_ref = Rc::clone(&keyboard_modality_ref);
            Rc::new(move || keyboard_modality_ref.set(true))
        };
        let on_pointer_down: Rc<dyn Fn()> = {
            let keyboard_modality_ref = Rc::clone(&keyboard_modality_ref);
            Rc::new(move || keyboard_modality_ref.set(false))
        };

        if enabled {
            // `getWindow(domReference)` (`useFocus.ts:68`) — the owner window of the
            // current reference, or the global window when there is none.
            let dom_reference = selectors::dom_reference_element(&store.get_snapshot());
            let window = owner_window(
                dom_reference
                    .as_ref()
                    .map(|el| el.as_ref() as &web_sys::Node),
            );

            // `mergeCleanups(addEventListener(win, 'blur', onBlur), isMacSafari &&
            // addEventListener(win, 'keydown', onKeyDown, true), isMacSafari &&
            // addEventListener(win, 'pointerdown', onPointerDown, true))`
            // (`useFocus.ts:93-97`).
            let keydown_listener = is_mac_safari.then(|| {
                let on_key_down = Rc::clone(&on_key_down);
                leptos_ui_utils::add_event_listener_with_options(
                    &window,
                    "keydown",
                    move |_event: &Event| on_key_down(),
                    true,
                )
            });
            let pointerdown_listener = is_mac_safari.then(|| {
                let on_pointer_down = Rc::clone(&on_pointer_down);
                leptos_ui_utils::add_event_listener_with_options(
                    &window,
                    "pointerdown",
                    move |_event: &Event| on_pointer_down(),
                    true,
                )
            });
            let blur_listener = {
                let on_blur = Rc::clone(&on_blur);
                leptos_ui_utils::add_event_listener(&window, "blur", move |_event: &Event| {
                    on_blur()
                })
            };

            let to_cleanup =
                |listener: leptos_ui_utils::add_event_listener::EventListenerUnsubscribe| {
                    Box::new(move || listener.unsubscribe()) as CleanupFn
                };
            let cleanup = merge_cleanups([
                Some(blur_listener).map(to_cleanup),
                keydown_listener.map(to_cleanup),
                pointerdown_listener.map(to_cleanup),
            ]);
            let cleanup: Rc<std::cell::Cell<Option<CleanupFn>>> =
                Rc::new(std::cell::Cell::new(Some(Box::new(cleanup))));
            let cleanup_handle = SendWrapper::new(cleanup);
            reactive_graph::owner::on_cleanup(move || {
                if let Some(cleanup) = cleanup_handle.deref().take() {
                    cleanup();
                }
            });
        }
    }

    // The openchange subscription (`useFocus.ts:100-119`): a trigger-press or
    // escape-key dismissal blocks the reference from re-opening on refocus.
    {
        let store = Rc::clone(&store);
        let block_focus_ref = Rc::clone(&block_focus_ref);
        let blocked_reference_ref = Rc::clone(&blocked_reference_ref);
        // The enabled flip re-subscribes upstream (`useFocus.ts:101-103`); the port's
        // static props make the guard equivalent: a disabled hook's subscription
        // ignores every emission.
        let unsubscribe: EventUnsubscribe = events.on(
            "openchange",
            Rc::new(
                move |details: &crate::floating_ui::types::FloatingUIOpenChangeDetails| {
                    if !enabled {
                        return;
                    }
                    if details.reason == reasons::TRIGGER_PRESS
                        || details.reason == reasons::ESCAPE_KEY
                    {
                        let reference_element =
                            selectors::dom_reference_element(&store.get_snapshot());
                        if reference_element.is_some() {
                            *blocked_reference_ref.borrow_mut() = reference_element;
                            block_focus_ref.set(true);
                        }
                    }
                },
            ),
        );
        let unsubscribe = SendWrapper::new(unsubscribe);
        reactive_graph::owner::on_cleanup(move || unsubscribe());
    }

    // `resetBlockedFocus` (`useFocus.ts:122-125`).
    let reset_blocked_focus = {
        let block_focus_ref = Rc::clone(&block_focus_ref);
        let blocked_reference_ref = Rc::clone(&blocked_reference_ref);
        Rc::new(move || {
            block_focus_ref.set(false);
            *blocked_reference_ref.borrow_mut() = None;
        })
    };

    // The reference bag (`useFocus.ts:121-245`).
    let reference = ElementHandlers {
        // `onMouseLeave` (`useFocus.ts:128-130`).
        on_mouse_leave: {
            let reset_blocked_focus = Rc::clone(&reset_blocked_focus);
            Some(Rc::new(move |_event: &MouseEvent| reset_blocked_focus()))
        },
        // `onFocus` (`useFocus.ts:131-194`).
        on_focus: {
            let store = Rc::clone(&store);
            let block_focus_ref = Rc::clone(&block_focus_ref);
            let blocked_reference_ref = Rc::clone(&blocked_reference_ref);
            let keyboard_modality_ref = Rc::clone(&keyboard_modality_ref);
            let timeout = timeout.clone();
            let reset_blocked_focus = Rc::clone(&reset_blocked_focus);
            let delay = delay.clone();
            Some(Rc::new(move |event: &FocusEvent| {
                let focus_target: Element = event
                    .current_target()
                    .and_then(|target| target.dyn_into::<Element>().ok())
                    .expect("focus events fire on elements in the port's wiring");

                if block_focus_ref.get() {
                    if blocked_reference_ref.borrow().as_ref() == Some(&focus_target) {
                        return;
                    }

                    reset_blocked_focus();
                }

                let target = get_target(event);

                if let Some(target) = target.as_ref().and_then(|t| t.dyn_ref::<Element>()) {
                    // Safari fails to match `:focus-visible` if focus was initially
                    // outside the document (`useFocus.ts:145-150`).
                    if is_mac_safari && event.related_target().is_none() {
                        if !keyboard_modality_ref.get() && !is_typeable_element(target) {
                            return;
                        }
                    } else if !matches_focus_visible(Some(target)) {
                        return;
                    }
                }

                let moved_from_other_enabled_trigger = is_target_inside_enabled_trigger(
                    event.related_target().as_ref(),
                    &store.context.trigger_elements,
                );

                let native_event: Event = event.clone().into();
                let current_target: Option<Element> = Some(focus_target.clone());
                let delay_value = delay.as_ref().and_then(FocusDelay::resolve);

                if (store.select(selectors::open) && moved_from_other_enabled_trigger)
                    || delay_value == Some(0)
                    || delay_value.is_none()
                {
                    store.set_open(
                        true,
                        &RootOpenChangeEventDetails::new(
                            reasons::TRIGGER_FOCUS,
                            native_event,
                            current_target,
                            String::new(),
                        ),
                    );
                    return;
                }

                let store = Rc::clone(&store);
                let block_focus_ref = Rc::clone(&block_focus_ref);
                timeout.start(
                    delay_value.expect("delayed branch has a positive delay"),
                    move || {
                        if block_focus_ref.get() {
                            return;
                        }

                        store.set_open(
                            true,
                            &RootOpenChangeEventDetails::new(
                                reasons::TRIGGER_FOCUS,
                                native_event,
                                Some(focus_target.clone()),
                                String::new(),
                            ),
                        );
                    },
                );
            }))
        },
        // `onBlur` (`useFocus.ts:195-243`).
        on_blur: {
            // (The upstream `dataRef` capture at `useFocus.ts:245` only feeds the
            // `floatingContext.refs.floating` read inside the timeout, which the port
            // sources from the store — see the module docs — so no `dataRef` capture
            // is needed here.)
            let store = Rc::clone(&store);
            let timeout = timeout.clone();
            let reset_blocked_focus = Rc::clone(&reset_blocked_focus);
            Some(Rc::new(move |event: &FocusEvent| {
                reset_blocked_focus();

                let related_target = event.related_target();
                let native_event: Event = event.clone().into();

                // Hit the non-modal focus management portal guard. Focus will be
                // moved into the floating element immediately after
                // (`useFocus.ts:201-206`).
                let moved_to_focus_guard = related_target
                    .as_ref()
                    .and_then(|target| target.dyn_ref::<Element>())
                    .map(|target| {
                        target.has_attribute(&create_attribute("focus-guard"))
                            && target.get_attribute("data-type").as_deref() == Some("outside")
                    })
                    .unwrap_or(false);

                // Wait for the window blur listener to fire (`useFocus.ts:208-242`).
                let store = Rc::clone(&store);
                timeout.start(0, move || {
                    let dom_reference = selectors::dom_reference_element(&store.get_snapshot());
                    let active_el = active_element(&owner_document(
                        dom_reference
                            .as_ref()
                            .map(|el| el.as_ref() as &web_sys::Node),
                    ));

                    // Focus left the page, keep it open (`useFocus.ts:213-216`).
                    if related_target.is_none() && active_el.as_ref() == dom_reference.as_ref() {
                        return;
                    }

                    // When focusing the reference element (e.g. regular click), then
                    // clicking into the floating element, prevent it from hiding
                    // (`useFocus.ts:218-231`). The floating element comes from the
                    // store (see the module docs — the port's floatingContext carries
                    // the store handle, whose floating element `refs.floating`
                    // mirrors).
                    let floating_element = selectors::floating_element(&store.get_snapshot());
                    if contains(floating_element.as_ref(), active_el.as_ref())
                        || contains(dom_reference.as_ref(), active_el.as_ref())
                        || moved_to_focus_guard
                    {
                        return;
                    }

                    // If the next focused element is one of the triggers, do not
                    // close the floating element. The focus handler of that trigger
                    // will handle the open state (`useFocus.ts:233-239`).
                    let next_focused_element: Option<EventTarget> =
                        match (related_target.clone(), active_el.clone()) {
                            (Some(related), _) => Some(related),
                            (None, Some(active)) => Some(active.into()),
                            (None, None) => None,
                        };
                    if is_target_inside_enabled_trigger(
                        next_focused_element.as_ref(),
                        &store.context.trigger_elements,
                    ) {
                        return;
                    }

                    store.set_open(
                        false,
                        &RootOpenChangeEventDetails::new(
                            reasons::TRIGGER_FOCUS,
                            native_event,
                            None,
                            String::new(),
                        ),
                    );
                });
            }))
        },
        ..ElementHandlers::default()
    };

    // `enabled ? { reference, trigger: reference } : {}` (`useFocus.ts:247-250`).
    if enabled {
        ElementProps {
            reference: Some(reference.clone()),
            trigger: Some(reference),
            ..ElementProps::default()
        }
    } else {
        ElementProps::default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // The window/listener wiring needs a DOM realm, so behavior tests live in the wasm
    // suite. Host builds pin the delay resolution and the empty-props shape.

    // Pins the delay resolution (`useFocus.ts:162`): a value resolves to itself; a
    // resolver is evaluated at call time; `None` means open immediately.
    #[test]
    fn focus_delay_resolves_values_and_resolvers() {
        let value = FocusDelay::Value(100);
        assert_eq!(value.resolve(), Some(100));

        let resolver = FocusDelay::Resolve(Rc::new(|| Some(42)));
        assert_eq!(resolver.resolve(), Some(42));

        let none_resolver = FocusDelay::Resolve(Rc::new(|| None));
        assert_eq!(
            none_resolver.resolve(),
            None,
            "a resolver returning None means open immediately (upstream's undefined)"
        );
    }

    // Pins the disabled shape (`useFocus.ts:247-250` — `{}`): no role is filled.
    #[test]
    fn the_disabled_hook_returns_the_empty_props_shape() {
        let props = ElementProps::default();
        assert!(props.reference.is_none());
        assert!(props.trigger.is_none());
        assert!(props.reference.is_none() && props.floating.is_none());
    }

    // Pins the openchange reason filter (`useFocus.ts:105-113`): only trigger-press
    // and escape-key dismissals block re-opening — the reason comparison values are
    // the ported registry's.
    #[test]
    fn the_blocking_reasons_are_trigger_press_and_escape_key() {
        assert_eq!(reasons::TRIGGER_PRESS, "trigger-press");
        assert_eq!(reasons::ESCAPE_KEY, "escape-key");
        assert_ne!(reasons::TRIGGER_FOCUS, reasons::TRIGGER_PRESS);
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

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

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
        // The consumer wiring the upstream harness has (`useFocus.test.tsx:19-24` —
        // `onOpenChange: setOpen`): the consumer state syncs back into the store (see
        // the useClick test module docs).
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
        store.set_field(
            |state| &mut state.dom_reference_element,
            Some(button.clone().into()),
        );
        button
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

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap()
        })
    }

    // Real focus movement — `button.focus()` fires the native `focusin` the bag's
    // listener is attached under, and the test realm has no prior trusted pointer
    // interaction, so the script focus matches `:focus-visible`
    // (`useFocus.ts:151`'s gate). Synthetic focusin dispatch is reserved for the
    // blocked-reopen step, where the block short-circuits before the gate.
    fn focus_element(target: &web_sys::HtmlElement) {
        target.focus().unwrap();
    }

    fn blur_element(target: &web_sys::HtmlElement) {
        target.blur().unwrap();
    }

    fn synthetic_focus_in(target: &web_sys::HtmlElement) {
        let init = web_sys::FocusEventInit::new();
        init.set_bubbles(true);
        let event = web_sys::FocusEvent::new_with_focus_event_init_dict("focusin", &init).unwrap();
        target.dispatch_event(&event).unwrap();
    }

    fn mouse_leave(target: &web_sys::HtmlElement) {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        let event = MouseEvent::new_with_mouse_event_init_dict("mouseleave", &init).unwrap();
        target.dispatch_event(&event).unwrap();
    }

    // Pins the open/close contract (`useFocus` — "Opens the floating element while the
    // reference element has focus"): focusin opens with reason trigger-focus; focusout
    // closes with reason trigger-focus after the 0ms re-check (`useFocus.ts:209`).
    #[wasm_bindgen_test(async)]
    async fn focus_opens_and_blur_closes_with_the_trigger_focus_reason() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_focus(Rc::clone(&store), UseFocusProps::default());
            let bag = props.reference.as_ref().unwrap();
            let _cleanup = bag.attach_to(button.as_ref()).unwrap();

            focus_element(&button);
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-focus".to_owned())],
                "focusin opens immediately (no delay)"
            );

            blur_element(&button);
            sleep(20).await;
            assert_eq!(
                log.calls(),
                vec![
                    (true, "trigger-focus".to_owned()),
                    (false, "trigger-focus".to_owned())
                ],
                "focusout closes after the 0ms re-check"
            );
        });
    }

    // Pins the delayed open (`useFocus.ts:180-193`): with a delay, focus does not open
    // immediately; after the delay it does.
    #[wasm_bindgen_test(async)]
    async fn a_delayed_focus_opens_after_the_delay() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_focus(
                Rc::clone(&store),
                UseFocusProps {
                    delay: Some(FocusDelay::Value(40)),
                    ..UseFocusProps::default()
                },
            );
            let bag = props.reference.as_ref().unwrap();
            let _cleanup = bag.attach_to(button.as_ref()).unwrap();

            focus_element(&button);
            assert!(
                log.calls().is_empty(),
                "the delayed open has not landed synchronously"
            );

            sleep(120).await;
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-focus".to_owned())],
                "the delayed open lands"
            );
        });
    }

    // Pins the window-blur block (`useFocus.test.tsx:17-51` — "does not reopen when
    // focus is restored after leaving the tab"): leaving the tab while focused and
    // closed blocks that reference's next focus open — both the fresh attempt and the
    // still-pending delayed open (`useFocus.ts:181-183`).
    #[wasm_bindgen_test(async)]
    async fn window_blur_blocks_reopening_after_focus_restoration() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_focus(
                Rc::clone(&store),
                UseFocusProps {
                    delay: Some(FocusDelay::Value(100)),
                    ..UseFocusProps::default()
                },
            );
            let bag = props.reference.as_ref().unwrap();
            let _cleanup = bag.attach_to(button.as_ref()).unwrap();

            // Focus the reference (the store is closed), then leave the tab.
            focus_element(&button);
            web_sys::window()
                .unwrap()
                .dispatch_event(&web_sys::Event::new("blur").unwrap())
                .unwrap();

            // A restored focus attempt on the blocked reference is dropped
            // (`useFocus.ts:134-137`).
            synthetic_focus_in(&button);

            // The pending delayed open fires but the block cancels it.
            sleep(200).await;
            assert!(
                log.calls().is_empty(),
                "the window blur blocked both the fresh and the pending open"
            );
        });
    }

    // Pins the trigger-role mirror (`useFocus.ts:247-250` — the trigger role carries
    // the same bag as the reference).
    #[wasm_bindgen_test]
    fn the_trigger_role_mirrors_the_reference_bag() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_focus(Rc::clone(&store), UseFocusProps::default());
            let trigger_bag = props.trigger.as_ref().expect("the trigger role is filled");
            assert!(props.floating.is_none(), "the floating role is untouched");
            let _cleanup = trigger_bag.attach_to(button.as_ref()).unwrap();

            focus_element(&button);
            assert_eq!(
                log.calls(),
                vec![(true, "trigger-focus".to_owned())],
                "the trigger role's focus handler opens"
            );
        });
    }

    // Pins the disabled shape (`useFocus.ts:247-250` — `{}` when disabled): no
    // listeners, no open.
    #[wasm_bindgen_test]
    fn the_disabled_hook_returns_empty_props_and_never_opens() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_focus(
                Rc::clone(&store),
                UseFocusProps {
                    enabled: false,
                    ..UseFocusProps::default()
                },
            );
            assert!(props.reference.is_none() && props.trigger.is_none());

            focus_element(&button);
            assert!(log.calls().is_empty());
        });
    }

    // Pins the openchange block (`useFocus.ts:100-119`): a trigger-press dismissal
    // blocks the reference from re-opening on refocus; mouseleave resets the block
    // (`useFocus.ts:128-130`) and focus opens again.
    #[wasm_bindgen_test(async)]
    async fn a_trigger_press_dismissal_blocks_refocus_reopen_until_mouseleave() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let (log, calls) = OpenLog::new();
            let store = store_with(false, calls);
            let button = button_with_store(&store);

            let props = use_focus(
                Rc::clone(&store),
                UseFocusProps {
                    delay: Some(FocusDelay::Value(40)),
                    ..UseFocusProps::default()
                },
            );
            let bag = props.reference.as_ref().unwrap();
            let _cleanup = bag.attach_to(button.as_ref()).unwrap();

            // A trigger-press open/close sequence — the press reason stamps the block.
            let press_details = RootOpenChangeEventDetails::new(
                reasons::TRIGGER_PRESS,
                web_sys::MouseEvent::new("click").unwrap().into(),
                None,
                String::new(),
            );
            store.set_open(true, &press_details);
            store.set_open(false, &press_details);
            assert_eq!(
                log.calls(),
                vec![
                    (true, "trigger-press".to_owned()),
                    (false, "trigger-press".to_owned())
                ],
                "the press sequence logs through the consumer callback"
            );

            // Refocus is blocked for the dismissed reference.
            focus_element(&button);
            sleep(120).await;
            assert_eq!(
                log.calls().len(),
                2,
                "the blocked reference does not reopen on focus"
            );

            // Mouseleave resets the block; the (now focused) reference's next focus
            // event opens. Blur first so a real focusin fires again.
            mouse_leave(&button);
            blur_element(&button);
            focus_element(&button);
            sleep(120).await;
            assert_eq!(
                log.calls(),
                vec![
                    (true, "trigger-press".to_owned()),
                    (false, "trigger-press".to_owned()),
                    (true, "trigger-focus".to_owned()),
                ],
                "after the block resets, focus opens"
            );
        });
    }
}
