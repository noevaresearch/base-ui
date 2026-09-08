//! Port of `packages/react/src/floating-ui-react/hooks/useClientPoint.ts` — positions
//! the floating element relative to a client point (in the viewport), such as the
//! mouse position (`specs/library/floating-ui-react/behavior.md`, "Public API
//! surface": `useClientPoint` (`enabled`, `axis`); "DOM structure & portal behavior":
//! "`useClientPoint` repositions via virtual reference rects rather than DOM moves").
//!
//! ## Rust adaptations
//!
//! - The virtual element (`createVirtualElement`, `useClientPoint.ts:10-82`) ports to
//!   [`ClientPointVirtualElement`] implementing the external crate's
//!   `VirtualElement` trait (the `wraps-external` delegation), with its interior
//!   per-invocation state (`offsetX`/`offsetY`/`isAutoUpdateEvent`) in a shared
//!   `Rc<RefCell>` — the JS closure state. The rect math itself is extracted verbatim
//!   into [`compute_client_point_rect`], pure over its inputs, so the axis/offset/
//!   freeze table is host-testable. JS truthiness (`data.x &&` — a `0` coordinate
//!   does not lock the offset) is preserved explicitly.
//! - The `positionReference` writes go to the store (the single source of truth —
//!   see the `use_floating` module docs); the store's no-op-skip rule preserves
//!   upstream's identical-write behavior.
//! - `pointerType`/`reactive` React state (`useClientPoint.ts:126-127`) become local
//!   signals: `pointerType` feeds the `openCheck` derivation
//!   (`useClientPoint.ts:175`), and `reactive` is the effect re-run bump for the
//    "cursor returned from the floating element" path (`useClientPoint.ts:167`).
//! - The main effect's dependency array (`useClientPoint.ts:211-221`) becomes the
//!   reactive reads of the effect body: `open`/`floating`/`domReference`
//!   (through `openCheck`), `pointerType`, and the `reactive` bump. The cleanup
//!   return (`useClientPoint.ts:187-190,210`) becomes `on_cleanup` inside the effect
//!   — it fires before each re-run and at disposal, the same lifecycle.
//! - The blur handler's `dataRef.current.floatingContext?.refs.floating`-shaped
//!   snapshot concerns do not arise here; the effect reads the store directly.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use floating_ui_dom::{ClientRectObject, VirtualElement};
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, Event, MouseEvent, PointerEvent};

use leptos_ui_utils::add_event_listener::EventListenerUnsubscribe;
use leptos_ui_utils::owner::owner_window;
use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_iso_layout_effect;

use crate::floating_ui::element_props::{ElementHandlers, ElementProps, FloatingContextSource};
use crate::floating_ui::event::is_mouse_like_pointer_type;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::types::{ContextData, ReferenceType, VirtualReference};

/// Port of `UseClientPointProps['axis']` (`useClientPoint.ts:14,101`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientPointAxis {
    /// `'x'`.
    X,
    /// `'y'`.
    Y,
    /// `'both'` (default).
    Both,
}

/// Port of `UseClientPointProps` (`useClientPoint.ts:88-102`) with the documented
/// defaults (`useClientPoint.ts:113`).
#[derive(Clone)]
pub struct UseClientPointProps {
    /// `enabled` (`useClientPoint.ts:96` — default `true`).
    pub enabled: bool,
    /// `axis` (`useClientPoint.ts:101` — default `'both'`).
    pub axis: ClientPointAxis,
}

impl Default for UseClientPointProps {
    fn default() -> Self {
        Self {
            enabled: true,
            axis: ClientPointAxis::Both,
        }
    }
}

/// The interior per-invocation state of `createVirtualElement`
/// (`useClientPoint.ts:20-22` — closure variables `offsetX`/`offsetY`/
/// `isAutoUpdateEvent`).
#[derive(Default)]
pub struct ClientPointShimState {
    offset_x: Option<f64>,
    offset_y: Option<f64>,
    is_auto_update_event: bool,
}

/// Port of the `getBoundingClientRect` body (`useClientPoint.ts:26-81`), extracted
/// verbatim into a pure function over its inputs. `dom_rect` is `None` when the shim
/// has no DOM element — upstream's `{ width: 0, height: 0, x: 0, y: 0 }` fallback
/// (`useClientPoint.ts:27-32`). The `open_event_type`/`pointer_type` pair feeds
/// `canTrackCursorOnAutoUpdate` (`useClientPoint.ts:36-38`): only hover/move-opened
/// popups track the cursor across auto-update passes.
pub fn compute_client_point_rect(
    state: &mut ClientPointShimState,
    dom_rect: Option<ClientRectObject>,
    axis: ClientPointAxis,
    x: Option<f64>,
    y: Option<f64>,
    open_event_type: Option<&str>,
    pointer_type: Option<&str>,
) -> ClientRectObject {
    let dom_rect = dom_rect.unwrap_or_else(|| ClientRectObject {
        width: 0.0,
        height: 0.0,
        x: 0.0,
        y: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    });

    let is_x_axis = matches!(axis, ClientPointAxis::X | ClientPointAxis::Both);
    let is_y_axis = matches!(axis, ClientPointAxis::Y | ClientPointAxis::Both);
    // `['mouseenter', 'mousemove'].includes(data.dataRef.current.openEvent?.type || '')
    // && data.pointerType !== 'touch'` (`useClientPoint.ts:36-38`).
    let can_track_cursor_on_auto_update =
        matches!(open_event_type, Some("mouseenter" | "mousemove"))
            && pointer_type != Some("touch");

    // (Upstream seeds these from `domRect` and immediately zeroes them at
    // `useClientPoint.ts:55-56`; the seed is dead there too, so the port starts at 0.)
    let mut width;
    let mut height;
    let mut x_pos = dom_rect.x;
    let mut y_pos = dom_rect.y;

    // `if (offsetX == null && data.x && isXAxis)` (`useClientPoint.ts:45-47`) —
    // `data.x &&` is JS truthiness: a `0` coordinate does not lock the offset.
    if state.offset_x.is_none() && x.map_or(false, |x| x != 0.0) && is_x_axis {
        state.offset_x = Some(dom_rect.x - x.expect("checked non-null above"));
    }
    if state.offset_y.is_none() && y.map_or(false, |y| y != 0.0) && is_y_axis {
        state.offset_y = Some(dom_rect.y - y.expect("checked non-null above"));
    }

    // `x -= offsetX || 0` (`useClientPoint.ts:53-54`).
    x_pos -= state.offset_x.unwrap_or(0.0);
    y_pos -= state.offset_y.unwrap_or(0.0);
    width = 0.0;
    height = 0.0;

    if !state.is_auto_update_event || can_track_cursor_on_auto_update {
        // Track the cursor: zero-size on the non-axis dimensions, cursor coordinates
        // on the axis ones (`useClientPoint.ts:58-63`).
        width = if axis == ClientPointAxis::Y {
            dom_rect.width
        } else {
            0.0
        };
        height = if axis == ClientPointAxis::X {
            dom_rect.height
        } else {
            0.0
        };
        if is_x_axis && x.is_some() {
            x_pos = x.expect("checked non-null above");
        }
        if is_y_axis && y.is_some() {
            y_pos = y.expect("checked non-null above");
        }
    } else if state.is_auto_update_event && !can_track_cursor_on_auto_update {
        // Freeze at the locked position: keep the (already offset-adjusted)
        // coordinates, sizing only the axis dimensions from the DOM element
        // (`useClientPoint.ts:63-66`).
        height = if axis == ClientPointAxis::X {
            dom_rect.height
        } else {
            height
        };
        width = if axis == ClientPointAxis::Y {
            dom_rect.width
        } else {
            width
        };
    }

    state.is_auto_update_event = true;

    ClientRectObject {
        width,
        height,
        x: x_pos,
        y: y_pos,
        top: y_pos,
        right: x_pos + width,
        bottom: y_pos + height,
        left: x_pos,
    }
}

/// Port of `createVirtualElement`'s return (`useClientPoint.ts:24-81`): the
/// positioning shim folding the cursor into the reference rect. The interior state is
/// shared through an `Rc<RefCell>` so clones behave like the JS closure object.
struct ClientPointVirtualElement {
    /// `contextElement: domElement || undefined` (`useClientPoint.ts:25`).
    context_element: Option<Element>,
    /// The per-invocation closure state (`useClientPoint.ts:20-22`).
    state: Rc<RefCell<ClientPointShimState>>,
    axis: ClientPointAxis,
    x: Option<f64>,
    y: Option<f64>,
    /// The pointer type snapshot at shim creation (`useClientPoint.ts:15` — the React
    /// state value at the time `setReference` ran).
    pointer_type: Option<String>,
    /// The shared `dataRef` — read per rect call for the open-event type
    /// (`useClientPoint.ts:37`).
    data_ref: Rc<RefCell<ContextData>>,
}

impl Clone for ClientPointVirtualElement {
    fn clone(&self) -> Self {
        Self {
            context_element: self.context_element.clone(),
            state: Rc::clone(&self.state),
            axis: self.axis,
            x: self.x,
            y: self.y,
            pointer_type: self.pointer_type.clone(),
            data_ref: Rc::clone(&self.data_ref),
        }
    }
}

impl PartialEq for ClientPointVirtualElement {
    fn eq(&self, other: &Self) -> bool {
        self.context_element == other.context_element
            && self.axis == other.axis
            && self.x == other.x
            && self.y == other.y
            && self.pointer_type == other.pointer_type
            && Rc::ptr_eq(&self.data_ref, &other.data_ref)
            && Rc::ptr_eq(&self.state, &other.state)
    }
}

impl VirtualElement<Element> for ClientPointVirtualElement {
    fn get_bounding_client_rect(&self) -> ClientRectObject {
        let dom_rect = self
            .context_element
            .as_ref()
            .map(|element| element.get_bounding_client_rect().into());
        let open_event_type = self
            .data_ref
            .borrow()
            .open_event
            .as_ref()
            .map(|event| event.type_());
        compute_client_point_rect(
            &mut self.state.borrow_mut(),
            dom_rect,
            self.axis,
            self.x,
            self.y,
            open_event_type.as_deref(),
            self.pointer_type.as_deref(),
        )
    }

    fn get_client_rects(&self) -> Option<Vec<ClientRectObject>> {
        // The upstream shim defines no `getClientRects` (`useClientPoint.ts:24-81`).
        None
    }

    fn context_element(&self) -> Option<Element> {
        self.context_element.clone()
    }
}

/// `isMouseBasedEvent(event)` (`useClientPoint.ts:84-86`): the event carries mouse
/// coordinates — `clientX` exists only on mouse-family events (`event != null &&
/// (event as MouseEvent).clientX != null`).
fn is_mouse_based_event(event: Option<&Event>) -> bool {
    event
        .map(|event| event.dyn_ref::<MouseEvent>().is_some())
        .unwrap_or(false)
}

/// Port of `useClientPoint(context, props)` (`useClientPoint.ts:109-260`). Must be
/// called inside a reactive owner. Returns the `reference` + `trigger` handler bags,
/// or the empty props when disabled.
pub fn use_client_point(
    context: impl Into<FloatingContextSource>,
    props: UseClientPointProps,
) -> ElementProps {
    let UseClientPointProps { enabled, axis } = props;

    let store: Rc<FloatingRootStore> = context.into().root_store();
    let inner = store.rc();

    // `store.useState('open')` / `('floatingElement')` / `('domReferenceElement')`
    // (`useClientPoint.ts:117-119`).
    let open = inner.use_state(selectors::open);
    let floating = inner.use_state(selectors::floating_element);
    let dom_reference = inner.use_state(selectors::dom_reference_element);

    // `const dataRef = store.context.dataRef` (`useClientPoint.ts:121`).
    let data_ref = Rc::clone(&store.context.data_ref);

    let initial_ref: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let cleanup_listener_ref: Rc<RefCell<Option<EventListenerUnsubscribe>>> =
        Rc::new(RefCell::new(None));

    // `[pointerType, setPointerType]` (`useClientPoint.ts:126`).
    let pointer_type: RwSignal<Option<String>, LocalStorage> = RwSignal::new_local(None);
    // `[reactive, setReactive]` (`useClientPoint.ts:127`) — the effect re-run bump for
    // the "cursor returned from the floating element" path (`useClientPoint.ts:167`).
    let reactive: RwSignal<u32, LocalStorage> = RwSignal::new_local(0);

    // `resetReference` (`useClientPoint.ts:129-131`).
    let reset_reference = {
        let store = Rc::clone(&store);
        Rc::new(move |reference: Option<Element>| {
            store.set_field(
                |state| &mut state.position_reference,
                reference.map(ReferenceType::Element),
            );
        })
    };

    // `setReference` (`useClientPoint.ts:133-157`).
    let set_reference = {
        let store = Rc::clone(&store);
        let data_ref = Rc::clone(&data_ref);
        let initial_ref = Rc::clone(&initial_ref);
        let dom_reference = dom_reference.clone();
        let pointer_type = pointer_type.clone();
        Rc::new(
            move |new_x: Option<f64>, new_y: Option<f64>, reference_element: Option<Element>| {
                if initial_ref.get() {
                    return;
                }

                // Prevent setting if the open event was not a mouse-like one (e.g.
                // focus to open, then hover over the reference element). Only apply
                // if the event exists (`useClientPoint.ts:139-144`).
                let has_open_event = data_ref.borrow().open_event.is_some();
                if has_open_event && !is_mouse_based_event(data_ref.borrow().open_event.as_ref()) {
                    return;
                }

                let dom_element = reference_element.or_else(|| dom_reference.get_untracked());
                let virtual_element = ClientPointVirtualElement {
                    context_element: dom_element,
                    state: Rc::new(RefCell::new(ClientPointShimState::default())),
                    axis,
                    x: new_x,
                    y: new_y,
                    pointer_type: pointer_type.get_untracked(),
                    data_ref: Rc::clone(&data_ref),
                };
                store.set_field(
                    |state| &mut state.position_reference,
                    Some(ReferenceType::Virtual(VirtualReference::new(
                        virtual_element,
                    ))),
                );
            },
        )
    };

    // `handleReferenceEnterOrMove` (`useClientPoint.ts:159-169`).
    let handle_reference_enter_or_move: crate::floating_ui::element_props::ElementEventHandler<
        MouseEvent,
    > = {
        let set_reference = Rc::clone(&set_reference);
        let open = open.clone();
        let cleanup_listener_ref = Rc::clone(&cleanup_listener_ref);
        let reactive = reactive.clone();
        Rc::new(move |event: &MouseEvent| {
            let client_x = event.client_x() as f64;
            let client_y = event.client_y() as f64;
            let current_target: Option<Element> = event
                .current_target()
                .and_then(|target| target.dyn_into::<Element>().ok());
            if !open.get_untracked() {
                (set_reference)(Some(client_x), Some(client_y), current_target);
            } else if cleanup_listener_ref.borrow().is_none() {
                // If there's no cleanup, there's no listener, but we want to ensure we
                // add the listener if the cursor landed on the floating element and
                // then back on the reference (i.e. it's interactive).
                (set_reference)(Some(client_x), Some(client_y), current_target);
                reactive.set(reactive.get_untracked() + 1);
            }
        })
    };

    // The main effect (`useClientPoint.ts:177-221`) — the reactive reads stand in for
    // the dependency array: `openCheck` (open/floating/pointerType), `enabled`,
    // `floating`, `domReference`, and the `reactive` bump.
    {
        let data_ref = Rc::clone(&data_ref);
        let cleanup_listener_ref = Rc::clone(&cleanup_listener_ref);
        let reset_reference = Rc::clone(&reset_reference);
        let set_reference = Rc::clone(&set_reference);
        let open = open.clone();
        let floating = floating.clone();
        let dom_reference = dom_reference.clone();
        let pointer_type = pointer_type.clone();
        let reactive = reactive.clone();
        use_iso_layout_effect(move || {
            let is_mouse_like = is_mouse_like_pointer_type(pointer_type.get().as_deref(), false);
            let open_value = open.get();
            let floating_value = floating.get();
            let dom_reference_value = dom_reference.get();
            let _reactive_bump = reactive.get();

            // The effect's cleanup (`useClientPoint.ts:187-190,210`): drop any window
            // listener from the previous run.
            {
                let cleanup_listener_ref = Rc::clone(&cleanup_listener_ref);
                let cleanup = SendWrapper::new(move || {
                    if let Some(listener) = cleanup_listener_ref.borrow_mut().take() {
                        listener.unsubscribe();
                    }
                });
                reactive_graph::owner::on_cleanup(move || cleanup());
            }

            if !enabled {
                (reset_reference)(dom_reference_value);
                return;
            }

            // `const openCheck = isMouseLikePointerType(pointerType) ? floating : open`
            // (`useClientPoint.ts:175`) — "continue following the mouse even if the
            // floating element is transitioning out" for mouse-like pointers; on
            // touch, follow only while open ("the floating element will move to the
            // dismissal touch point" otherwise).
            let open_check_truthy = if is_mouse_like {
                floating_value.is_some()
            } else {
                open_value
            };
            if !open_check_truthy {
                return;
            }

            let window = owner_window(
                floating_value
                    .as_ref()
                    .map(|element| element.as_ref() as &web_sys::Node),
            );

            let has_no_open_event = data_ref.borrow().open_event.is_none();
            if has_no_open_event || is_mouse_based_event(data_ref.borrow().open_event.as_ref()) {
                // `addEventListener(win, 'mousemove', handleMouseMove)`
                // (`useClientPoint.ts:204-205`).
                let floating_for_handler = floating_value.clone();
                let set_reference_for_handler = Rc::clone(&set_reference);
                let cleanup_listener_ref_for_handler = Rc::clone(&cleanup_listener_ref);
                let listener = leptos_ui_utils::add_event_listener(
                    &window,
                    "mousemove",
                    move |event: &Event| {
                        let mouse_event = event.dyn_ref::<MouseEvent>();
                        let target = get_target(event);
                        let target_element = target.as_ref().and_then(|t| t.dyn_ref::<Element>());
                        if !contains(floating_for_handler.as_ref(), target_element) {
                            (set_reference_for_handler)(
                                mouse_event.map(|m| m.client_x() as f64),
                                mouse_event.map(|m| m.client_y() as f64),
                                None,
                            );
                        } else {
                            // The cursor landed on the floating element: stop
                            // following (`useClientPoint.ts:199-201`).
                            if let Some(listener) =
                                cleanup_listener_ref_for_handler.borrow_mut().take()
                            {
                                listener.unsubscribe();
                            }
                        }
                    },
                );
                *cleanup_listener_ref.borrow_mut() = Some(listener);
            } else {
                // A non-mouse open event (e.g. focus): restore the DOM reference
                // (`useClientPoint.ts:206-208`).
                (reset_reference)(dom_reference_value);
            }
        });
    }

    // Clear virtual cursor references when the hook unmounts. Enabled flips are
    // handled above (`useClientPoint.ts:223-229`).
    {
        let store = SendWrapper::new(Rc::clone(&store));
        reactive_graph::owner::on_cleanup(move || {
            store.set_field(|state| &mut state.position_reference, None);
        });
    }

    // `initialRef` resets (`useClientPoint.ts:231-241`): re-arm after the floating
    // element disappears; latch when disabled while open.
    {
        let initial_ref = Rc::clone(&initial_ref);
        let floating = floating.clone();
        use_iso_layout_effect(move || {
            let floating_value = floating.get();
            if enabled && floating_value.is_none() {
                initial_ref.set(false);
            }
        });
    }
    {
        let initial_ref = Rc::clone(&initial_ref);
        let open = open.clone();
        use_iso_layout_effect(move || {
            let open_value = open.get();
            if !enabled && open_value {
                initial_ref.set(true);
            }
        });
    }

    // The reference bag (`useClientPoint.ts:243-254`).
    let reference = ElementHandlers {
        // `setPointerTypeRef` (`useClientPoint.ts:244-246`).
        on_pointer_down: {
            let pointer_type = pointer_type.clone();
            Some(Rc::new(move |event: &PointerEvent| {
                pointer_type.set(Some(event.pointer_type()));
            }))
        },
        on_pointer_enter: {
            let pointer_type = pointer_type.clone();
            Some(Rc::new(move |event: &PointerEvent| {
                pointer_type.set(Some(event.pointer_type()));
            }))
        },
        on_mouse_move: Some(Rc::clone(&handle_reference_enter_or_move)),
        on_mouse_enter: Some(Rc::clone(&handle_reference_enter_or_move)),
        ..ElementHandlers::default()
    };

    // `enabled ? { reference, trigger: reference } : {}` (`useClientPoint.ts:256-259`).
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
    use super::compute_client_point_rect;
    use super::{ClientPointAxis, ClientPointShimState};

    use floating_ui_dom::ClientRectObject;

    fn dom_rect(x: f64, y: f64, width: f64, height: f64) -> ClientRectObject {
        ClientRectObject {
            x,
            y,
            width,
            height,
            top: y,
            right: x + width,
            bottom: y + height,
            left: x,
        }
    }

    // Pins the first-call lock (`useClientPoint.ts:45-54,58-63`): the offset is locked
    // on the first call; the rect is zero-sized with cursor coordinates on the axis
    // dimensions.
    #[test]
    fn the_first_call_locks_the_offset_and_folds_the_cursor_in() {
        let mut state = ClientPointShimState::default();
        let rect = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(110.0),
            Some(70.0),
            Some("mousemove"),
            Some("mouse"),
        );

        // offset = domRect.x - data.x = -10 → x = domRect.x - offset = 110.
        assert_eq!(rect.x, 110.0);
        assert_eq!(rect.y, 70.0);
        // Zero-size on both axes for 'both'.
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
        assert_eq!(rect.top, 70.0);
        assert_eq!(rect.left, 110.0);
        assert_eq!(rect.right, 110.0);
        assert_eq!(rect.bottom, 70.0);
    }

    // Pins the auto-update freeze (`useClientPoint.ts:63-66` + `:36-38`): on later
    // calls, a focus-opened popup's rect freezes at the locked position; a
    // hover/mousemove-opened one keeps tracking the cursor.
    #[test]
    fn auto_update_freezes_unless_the_open_event_tracks_the_cursor() {
        // Focus-opened: openEvent type is not mouse-like → the cursor freezes.
        let mut state = ClientPointShimState::default();
        compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(110.0),
            Some(70.0),
            Some("focus"),
            Some("mouse"),
        );
        let frozen = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(160.0),
            Some(80.0),
            Some("focus"),
            Some("mouse"),
        );
        assert_eq!(
            (frozen.x, frozen.y),
            (110.0, 70.0),
            "the rect stays at the locked cursor position across auto-update passes"
        );

        // Hover-opened: the cursor keeps tracking.
        let mut state = ClientPointShimState::default();
        compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(110.0),
            Some(70.0),
            Some("mousemove"),
            Some("mouse"),
        );
        let tracking = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(160.0),
            Some(80.0),
            Some("mousemove"),
            Some("mouse"),
        );
        assert_eq!((tracking.x, tracking.y), (160.0, 80.0));
    }

    // Pins the touch exception (`useClientPoint.ts:38` — `data.pointerType !==
    // 'touch'`): a touch pointer never tracks the cursor across auto-update passes.
    #[test]
    fn touch_pointers_do_not_track_the_cursor_across_auto_updates() {
        let mut state = ClientPointShimState::default();
        compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(110.0),
            Some(70.0),
            Some("mousemove"),
            Some("touch"),
        );
        let frozen = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(160.0),
            Some(80.0),
            Some("mousemove"),
            Some("touch"),
        );
        assert_eq!(
            (frozen.x, frozen.y),
            (110.0, 70.0),
            "touch openers freeze during autoUpdate to avoid moving to the dismissal touch point"
        );
    }

    // Pins the axis behavior (`useClientPoint.test.tsx:324-342` — 'axis x'/'axis y'):
    // axis 'x' keeps the element's height and the element's y, tracking only the
    // cursor's x; axis 'y' mirrors it.
    #[test]
    fn axis_x_and_y_restrict_the_tracking_dimension() {
        let mut state = ClientPointShimState::default();
        let rect = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::X,
            Some(110.0),
            None,
            Some("mousemove"),
            Some("mouse"),
        );
        assert_eq!(rect.x, 110.0, "the cursor x is tracked");
        assert_eq!(rect.y, 50.0, "the element y is kept");
        assert_eq!(rect.height, 10.0, "the element height is kept on axis x");
        assert_eq!(rect.width, 0.0, "the width is zeroed on axis x");

        let mut state = ClientPointShimState::default();
        let rect = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Y,
            None,
            Some(70.0),
            Some("mousemove"),
            Some("mouse"),
        );
        assert_eq!(rect.x, 100.0, "the element x is kept");
        assert_eq!(rect.y, 70.0, "the cursor y is tracked");
        assert_eq!(rect.width, 20.0, "the element width is kept on axis y");
        assert_eq!(rect.height, 0.0, "the height is zeroed on axis y");
    }

    // Pins the JS-truthiness guard (`useClientPoint.ts:45-51` — `data.x &&`): a `0`
    // coordinate does not lock the offset, but the final rect still uses the
    // coordinate (`data.x != null ? data.x : x`).
    #[test]
    fn a_zero_coordinate_does_not_lock_the_offset() {
        let mut state = ClientPointShimState::default();
        let rect = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(0.0),
            Some(70.0),
            Some("mousemove"),
            Some("mouse"),
        );
        assert_eq!(rect.x, 0.0, "the coordinate is still used (non-null check)");
        // The offset never locked (the state has no offset): a second call with a
        // different x does not shift by a stale offset.
        let second = compute_client_point_rect(
            &mut state,
            Some(dom_rect(100.0, 50.0, 20.0, 10.0)),
            ClientPointAxis::Both,
            Some(0.0),
            Some(70.0),
            Some("mousemove"),
            Some("mouse"),
        );
        assert_eq!(second.x, 0.0);
    }

    // Pins the no-DOM-element fallback (`useClientPoint.ts:27-32`): a missing element
    // rects to the zero rect.
    #[test]
    fn a_missing_dom_element_uses_the_zero_rect() {
        let mut state = ClientPointShimState::default();
        let rect = compute_client_point_rect(
            &mut state,
            None,
            ClientPointAxis::Both,
            Some(30.0),
            Some(40.0),
            None,
            None,
        );
        assert_eq!(rect.x, 30.0);
        assert_eq!(rect.y, 40.0);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::cell::RefCell;

    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn store_with() -> Rc<FloatingRootStore> {
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

    fn mount_element(tag: &str) -> Element {
        let document = web_sys::window().unwrap().document().unwrap();
        let element = document.create_element(tag).unwrap();
        document.body().unwrap().append_child(&element).unwrap();
        element
    }

    fn mouse_event(event_type: &str, x: f64, y: f64) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_client_x(x as i32);
        init.set_client_y(y as i32);
        MouseEvent::new_with_mouse_event_init_dict(event_type, &init).unwrap()
    }

    fn flush() {
        // The store's notify → use_state mirror → RenderEffect re-run chain is
        // executor-timed; drain it (the ReactStore port's test wiring).
        for _ in 0..4 {
            any_spawner::Executor::poll_local();
        }
    }

    /// A fixed zero-rect virtual element for seeding a stale position reference.
    #[derive(Clone, PartialEq)]
    struct FixedVirtualElement;

    impl VirtualElement<Element> for FixedVirtualElement {
        fn get_bounding_client_rect(&self) -> ClientRectObject {
            ClientRectObject {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }
        }

        fn get_client_rects(&self) -> Option<Vec<ClientRectObject>> {
            None
        }

        fn context_element(&self) -> Option<Element> {
            None
        }
    }

    // Pins the closed-popup path (`useClientPoint.test.tsx:124-169`): a mousemove over
    // the reference while closed installs a virtual position reference whose
    // contextElement is the reference.
    #[wasm_bindgen_test]
    fn a_closed_popup_mousemove_installs_a_virtual_reference_on_the_trigger() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = store_with();
            let reference = mount_element("button");
            store.update(|state, _| {
                state.reference_element = Some(ReferenceType::Element(reference.clone()));
                state.dom_reference_element = Some(reference.clone());
                true
            });

            let props = use_client_point(Rc::clone(&store), UseClientPointProps::default());
            let bag = props.reference.as_ref().unwrap();
            let _cleanup = bag.attach_to(reference.as_ref()).unwrap();

            reference
                .dispatch_event(&mouse_event("mousemove", 400.0, 200.0))
                .unwrap();

            let position = store.get_snapshot().position_reference.clone();
            assert!(
                matches!(position, Some(ReferenceType::Virtual(_))),
                "the closed-popup mousemove installs a virtual reference"
            );
            let context = position.unwrap().context_element();
            assert!(
                context.map(|el| el == reference).unwrap_or(false),
                "the virtual element's contextElement is the reference"
            );
        });
    }

    // Pins the window-listener lifecycle (`useClientPoint.test.tsx:244-300` — "cleans
    // up window listener when closing or disabling"): an open, mouse-opened popup
    // with a mounted floating element installs a window mousemove that repositions to
    // the cursor; closing removes it, so further window mousemoves leave the position
    // reference alone.
    #[wasm_bindgen_test]
    fn the_window_listener_follows_the_open_state() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = store_with();
            let reference = mount_element("button");
            let floating = mount_element("div");
            store.update(|state, _| {
                state.reference_element = Some(ReferenceType::Element(reference.clone()));
                state.dom_reference_element = Some(reference.clone());
                state.floating_element = Some(floating.clone());
                true
            });

            let props = use_client_point(Rc::clone(&store), UseClientPointProps::default());
            let _cleanup = props
                .reference
                .as_ref()
                .unwrap()
                .attach_to(reference.as_ref())
                .unwrap();

            // Open with a mouse-like open event — the window listener installs.
            let mouse_open = web_sys::MouseEvent::new("mousemove").unwrap();
            store.context.data_ref.borrow_mut().open_event = Some(mouse_open.into());
            store.update(|state, _| {
                state.open = true;
                true
            });
            flush();

            // A window mousemove away from the floating element repositions.
            web_sys::window()
                .unwrap()
                .dispatch_event(&mouse_event("mousemove", 200.0, 210.0))
                .unwrap();
            let position = store.get_snapshot().position_reference.clone();
            assert!(
                matches!(position, Some(ReferenceType::Virtual(_))),
                "the window mousemove keeps a virtual reference"
            );
            let virtual_rect = match &position {
                Some(ReferenceType::Virtual(virtual_ref)) => {
                    virtual_ref.element.get_bounding_client_rect().x
                }
                _ => unreachable!(),
            };
            assert_eq!(
                virtual_rect, 200.0,
                "the rect folds the window cursor's x in"
            );

            // Close: the effect re-runs and removes the listener; further window
            // mousemoves do not reposition (the identity-debug comparison makes a
            // stale write observable).
            store.update(|state, _| {
                state.open = false;
                true
            });
            flush();
            let position_before = format!("{:?}", store.get_snapshot().position_reference);
            web_sys::window()
                .unwrap()
                .dispatch_event(&mouse_event("mousemove", 250.0, 250.0))
                .unwrap();
            let position_after = format!("{:?}", store.get_snapshot().position_reference);
            assert_eq!(
                position_before, position_after,
                "no window-listener repositioning after close"
            );
        });
    }

    // Pins the non-mouse open-event reset (`useClientPoint.ts:206-208` +
    // `:139-144` — "restores the DOM reference when opened by a non-mouse event",
    // `useClientPoint.test.tsx:469-522`): a focus-opened popup resets its position
    // reference to the real DOM reference, and later mousemoves cannot reposition
    // (the setReference guard).
    #[wasm_bindgen_test]
    fn a_non_mouse_open_event_restores_the_dom_reference_and_blocks_repositioning() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = store_with();
            let reference = mount_element("button");
            let floating = mount_element("div");
            store.update(|state, _| {
                state.reference_element = Some(ReferenceType::Element(reference.clone()));
                state.dom_reference_element = Some(reference.clone());
                state.floating_element = Some(floating.clone());
                true
            });

            let props = use_client_point(Rc::clone(&store), UseClientPointProps::default());
            let bag = props.reference.as_ref().unwrap();
            let _cleanup = bag.attach_to(reference.as_ref()).unwrap();

            // While closed, a mousemove installs a virtual reference.
            reference
                .dispatch_event(&mouse_event("mousemove", 500.0, 500.0))
                .unwrap();
            assert!(
                matches!(
                    store.get_snapshot().position_reference,
                    Some(ReferenceType::Virtual(_))
                ),
                "the closed-popup mousemove installed a virtual reference"
            );

            // Open via a keyboard-ish (non-mouse) event: the effect resets the
            // position reference to the DOM reference.
            let key_open = web_sys::KeyboardEvent::new("keydown").unwrap();
            store.context.data_ref.borrow_mut().open_event = Some(key_open.into());
            store.update(|state, _| {
                state.open = true;
                true
            });
            flush();

            let position = store.get_snapshot().position_reference.clone();
            assert!(
                matches!(&position, Some(ReferenceType::Element(el)) if *el == reference),
                "the non-mouse open restored the DOM reference"
            );

            // Later mousemoves cannot reposition: the openEvent is not mouse-based.
            reference
                .dispatch_event(&mouse_event("mousemove", 300.0, 400.0))
                .unwrap();
            let position_after = store.get_snapshot().position_reference.clone();
            assert!(
                matches!(&position_after, Some(ReferenceType::Element(el)) if *el == reference),
                "the non-mouse openEvent blocks repositioning"
            );
        });
    }

    // Pins the unmount clear (`useClientPoint.ts:223-229` — "clears virtual references
    // on unmount without retaining a stale DOM reference",
    // `useClientPoint.test.tsx:302-322`): disposal resets the position reference.
    #[wasm_bindgen_test]
    fn unmount_clears_the_position_reference() {
        init_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();

        let store = store_with();
        let reference = mount_element("button");
        store.update(|state, _| {
            state.reference_element = Some(ReferenceType::Element(reference.clone()));
            state.dom_reference_element = Some(reference.clone());
            true
        });

        let props = use_client_point(Rc::clone(&store), UseClientPointProps::default());
        let _cleanup = props
            .reference
            .as_ref()
            .unwrap()
            .attach_to(reference.as_ref())
            .unwrap();
        reference
            .dispatch_event(&mouse_event("mousemove", 40.0, 44.0))
            .unwrap();
        assert!(store.get_snapshot().position_reference.is_some());

        owner.cleanup();
        assert!(
            store.get_snapshot().position_reference.is_none(),
            "the unmount cleared the virtual cursor reference"
        );
    }

    // Pins the disabled shape (`useClientPoint.ts:256-259` — `{}`): no roles filled.
    #[wasm_bindgen_test]
    fn the_disabled_hook_returns_empty_props() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = store_with();
            let props = use_client_point(
                Rc::clone(&store),
                UseClientPointProps {
                    enabled: false,
                    ..UseClientPointProps::default()
                },
            );
            assert!(props.reference.is_none() && props.trigger.is_none());
        });
    }
}
