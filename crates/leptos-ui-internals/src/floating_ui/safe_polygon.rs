//! Port of `packages/react/src/floating-ui-react/safePolygon.ts` — the
//! `handleClose` factory that keeps a hover-opened popup reachable while the cursor
//! travels from the reference element to the floating element
//! (`specs/library/floating-ui-react/behavior.md`, "Public API surface": `safePolygon()`
//! factory; "Hover corridor chain" cross-cutting behavior).
//!
//! ## Rust adaptations
//!
//! - `SafePolygonOptions extends HandleCloseOptions {}` (`safePolygon.ts:83`) is an
//!   empty extension — the port aliases [`SafePolygonOptions`] onto
//!   [`HandleCloseOptions`] (the shared-state hook's `handleCloseOptions` field carries
//!   the same shape).
//! - The factory's per-invocation traversal state (`hasLanded`, `lastX`, `lastY`,
//!   `lastCursorTime` — `safePolygon.ts:96-99`) ports to `Rc`-shared `Cell`s so the
//!   returned handler stays an `Fn` shared by `Rc` (upstream closes over locals).
//!   Creating a handler from the factory resets the state — the fresh-handler-means-
//!   fresh-corridor property the traversal-reset test pins
//!   (`packages/react/src/floating-ui-react/safePolygon.test.ts:311-339`).
//! - `performance.now()` (`safePolygon.ts:99,102`) ports to the same API through
//!   `window.performance` (the `typeof performance !== 'undefined'` fallback yields
//!   `0.0` when no window is present). Only elapsed deltas are consumed
//!   (`isCursorMovingSlowly`), so the origin does not matter.
//! - `getBoundingClientRect()` reads (`:182-183`) port to
//!   [`floating_ui_dom::ClientRectObject`] — the DOMRect conversion the crate's other
//!   rect readers use (`hooks/useFloating.ts` port) — so every edge read
//!   (`rect.left`/`right`/`top`/`bottom`/`x`/`y`/`width`/`height`) is field-exact.
//! - `getTarget`/`contains` come from the `shadowDom` port and `isElement` from the
//!   external `floating-ui-dom`'s `dom` module — the same imports upstream makes
//!   (`safePolygon.ts:1,5`).
//! - The pure geometry helpers (`hasIntersectingEdge`, `isPointInQuadrilateral`,
//!   `isInsideRect`, `isInsideAxisAlignedRect` — `safePolygon.ts:14-81`) port verbatim
//!   as free functions, host-testable over [`ClientRectObject`].

use std::cell::Cell;
use std::rc::Rc;

use floating_ui_dom::ClientRectObject;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};

use leptos_ui_utils::shadow_dom::{contains, get_target};
use leptos_ui_utils::use_timeout::Timeout;

use crate::floating_ui::nodes::get_node_children;
use crate::floating_ui::use_hover_shared::{
    HandleClose, HandleCloseContext, HandleCloseFactory, HandleCloseOptions,
};

/// Port of `SafePolygonOptions` (`safePolygon.ts:83`) — an empty extension of
/// [`HandleCloseOptions`].
pub type SafePolygonOptions = HandleCloseOptions;

/// `CURSOR_SPEED_THRESHOLD` (`safePolygon.ts:10`).
const CURSOR_SPEED_THRESHOLD: f64 = 0.1;
/// `CURSOR_SPEED_THRESHOLD_SQUARED` (`safePolygon.ts:11`).
const CURSOR_SPEED_THRESHOLD_SQUARED: f64 = CURSOR_SPEED_THRESHOLD * CURSOR_SPEED_THRESHOLD;
/// `POLYGON_BUFFER` (`safePolygon.ts:12`).
const POLYGON_BUFFER: f64 = 0.5;

/// The 40 ms intent window (`safePolygon.ts:437`): a cursor inside the polygon that has
/// not yet landed closes the popup after it.
const INTENT_TIMEOUT_MS: u32 = 40;

/// Port of `hasIntersectingEdge` (`safePolygon.ts:14-23`) — the ray-casting edge test.
fn has_intersecting_edge(point_x: f64, point_y: f64, xi: f64, yi: f64, xj: f64, yj: f64) -> bool {
    (yi >= point_y) != (yj >= point_y) && point_x <= ((xj - xi) * (point_y - yi)) / (yj - yi) + xi
}

/// Port of `isPointInQuadrilateral` (`safePolygon.ts:25-56`) — even-odd ray casting
/// over the four edges.
fn is_point_in_quadrilateral(
    point_x: f64,
    point_y: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
) -> bool {
    let mut is_inside_value = false;

    if has_intersecting_edge(point_x, point_y, x1, y1, x2, y2) {
        is_inside_value = !is_inside_value;
    }
    if has_intersecting_edge(point_x, point_y, x2, y2, x3, y3) {
        is_inside_value = !is_inside_value;
    }
    if has_intersecting_edge(point_x, point_y, x3, y3, x4, y4) {
        is_inside_value = !is_inside_value;
    }
    if has_intersecting_edge(point_x, point_y, x4, y4, x1, y1) {
        is_inside_value = !is_inside_value;
    }

    is_inside_value
}

/// Port of `isInsideRect` (`safePolygon.ts:58-65`).
fn is_inside_rect(point_x: f64, point_y: f64, rect: &ClientRectObject) -> bool {
    point_x >= rect.x
        && point_x <= rect.x + rect.width
        && point_y >= rect.y
        && point_y <= rect.y + rect.height
}

/// Port of `isInsideAxisAlignedRect` (`safePolygon.ts:67-81`) — the corner order is
/// normalized (`Math.min`/`Math.max`).
fn is_inside_axis_aligned_rect(
    point_x: f64,
    point_y: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> bool {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);

    point_x >= min_x && point_x <= max_x && point_y >= min_y && point_y <= max_y
}

/// `performance.now()` (`safePolygon.ts:99,102`) — `0.0` when no window is present
/// (upstream's `typeof performance !== 'undefined' ? performance.now() : 0`).
fn performance_now() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now())
        .unwrap_or(0.0)
}

/// Port of `safePolygon(options)` (`safePolygon.ts:90-451`): generates a safe polygon
/// area that the user can traverse without closing the floating element once leaving
/// the reference element. The returned [`HandleClose`] is passed as `useHover`'s
/// `handleClose` (see the behavior spec's "Hover corridor chain").
///
/// The factory closes over one shared [`Timeout`] across invocations (`:92`); each
/// factory invocation creates fresh traversal state (`:96-99`), so a fresh handler
/// always means a fresh corridor.
pub fn safe_polygon(options: SafePolygonOptions) -> HandleClose {
    let HandleCloseOptions {
        block_pointer_events,
        get_scope,
    } = options;
    let timeout = Timeout::create();
    let factory: HandleCloseFactory = Rc::new(move |context: &HandleCloseContext| {
        let side = context
            .base
            .placement
            .as_ref()
            .map(|placement| placement.side());
        let has_landed = Rc::new(Cell::new(false));
        let last_x = Rc::new(Cell::new(None::<f64>));
        let last_y = Rc::new(Cell::new(None::<f64>));
        let last_cursor_time = Rc::new(Cell::new(performance_now()));

        let dom_reference = context.base.dom_reference.clone();
        let floating = context.base.floating.clone();
        let context_x = context.x;
        let context_y = context.y;
        let on_close = Rc::clone(&context.on_close);
        let tree = context.tree.clone();
        let node_id = context.base.node_id.clone();
        let timeout = timeout.clone();

        // `close` (`safePolygon.ts:124-127`): clear the shared intent timer,
        // then request the close.
        let close = {
            let timeout = timeout.clone();
            let on_close = Rc::clone(&on_close);
            move || {
                timeout.clear();
                on_close();
            }
        };

        // `hasOpenChildNode` (`safePolygon.ts:167-169`): any open child in the
        // floating tree suppresses the close (`onlyOpenChildren` defaults to
        // true upstream). Shared by the early abort and the close paths.
        let has_open_child_node: Rc<dyn Fn() -> bool> = {
            let tree = tree.clone();
            let node_id = node_id.clone();
            Rc::new(move || {
                tree.as_ref()
                    .map(|tree| {
                        let nodes = tree.nodes.borrow();
                        !get_node_children(&nodes, node_id.as_deref(), true).is_empty()
                    })
                    .unwrap_or(false)
            })
        };

        // `closeIfNoOpenChild` (`safePolygon.ts:171-175`) — shared by the
        // synchronous close paths and the intent timer.
        let close_if_no_open_child: Rc<dyn Fn()> = {
            let close = close;
            let has_open_child_node = Rc::clone(&has_open_child_node);
            Rc::new(move || {
                if !has_open_child_node() {
                    close();
                }
            })
        };

        // `isCursorMovingSlowly` (`safePolygon.ts:101-122`): the tremor guard
        // — a cursor moving slower than the threshold (per elapsed time)
        // keeps the popup open.
        let is_cursor_moving_slowly = move |next_x: f64, next_y: f64| -> bool {
            let current_time = performance_now();
            let elapsed_time = current_time - last_cursor_time.get();

            if last_x.get().is_none() || last_y.get().is_none() || elapsed_time == 0.0 {
                last_x.set(Some(next_x));
                last_y.set(Some(next_y));
                last_cursor_time.set(current_time);
                return false;
            }

            let delta_x = next_x - last_x.get().expect("checked above");
            let delta_y = next_y - last_y.get().expect("checked above");
            let distance_squared = delta_x * delta_x + delta_y * delta_y;
            let threshold_squared = elapsed_time * elapsed_time * CURSOR_SPEED_THRESHOLD_SQUARED;

            last_x.set(Some(next_x));
            last_y.set(Some(next_y));
            last_cursor_time.set(current_time);

            distance_squared < threshold_squared
        };

        let is_cursor_moving_slowly = Rc::new(is_cursor_moving_slowly);

        Rc::new(move |event: &MouseEvent| {
            timeout.clear();

            let (Some(dom_reference), Some(floating)) = (dom_reference.as_ref(), floating.as_ref())
            else {
                return;
            };
            let Some(side) = side else {
                return;
            };
            let (Some(x), Some(y)) = (context_x, context_y) else {
                return;
            };

            let client_x = event.client_x() as f64;
            let client_y = event.client_y() as f64;
            let target: Option<Element> =
                get_target(event).and_then(|target| target.dyn_into::<Element>().ok());
            let is_leave = event.type_() == "mouseleave";
            let is_over_floating_el = contains(Some(floating), target.as_ref());
            let is_over_reference_el = contains(Some(dom_reference), target.as_ref());

            if is_over_floating_el {
                has_landed.set(true);

                if !is_leave {
                    return;
                }
            }

            if is_over_reference_el {
                has_landed.set(false);

                if !is_leave {
                    has_landed.set(true);
                    return;
                }
            }

            // Prevent overlapping floating element from being stuck in an
            // open-close loop: https://github.com/floating-ui/floating-ui/issues/1910
            // (upstream `isElement(event.relatedTarget) && contains(...)` — the
            // `dyn_ref` is the `isElement` interface check).
            if is_leave
                && event
                    .related_target()
                    .and_then(|related| related.dyn_ref::<Element>().cloned())
                    .map(|related| contains(Some(floating), Some(&related)))
                    .unwrap_or(false)
            {
                return;
            }

            // If any nested child is open, abort.
            if has_open_child_node() {
                return;
            }

            let ref_rect: ClientRectObject = dom_reference.get_bounding_client_rect().into();
            let rect: ClientRectObject = floating.get_bounding_client_rect().into();
            let cursor_leave_from_right = x > rect.right - rect.width / 2.0;
            let cursor_leave_from_bottom = y > rect.bottom - rect.height / 2.0;
            let is_floating_wider = rect.width > ref_rect.width;
            let is_floating_taller = rect.height > ref_rect.height;
            let left = if is_floating_wider {
                ref_rect.left
            } else {
                rect.left
            };
            let right = if is_floating_wider {
                ref_rect.right
            } else {
                rect.right
            };
            let top = if is_floating_taller {
                ref_rect.top
            } else {
                rect.top
            };
            let bottom = if is_floating_taller {
                ref_rect.bottom
            } else {
                rect.bottom
            };

            // If the pointer is leaving from the opposite side, the "buffer"
            // logic creates a point where the floating element remains open,
            // but should be ignored. A constant of 1 handles floating point
            // rounding errors. (`safePolygon.ts:197-205`)
            if (side == floating_ui_dom::Side::Top && y >= ref_rect.bottom - 1.0)
                || (side == floating_ui_dom::Side::Bottom && y <= ref_rect.top + 1.0)
                || (side == floating_ui_dom::Side::Left && x >= ref_rect.right - 1.0)
                || (side == floating_ui_dom::Side::Right && x <= ref_rect.left + 1.0)
            {
                close_if_no_open_child();
                return;
            }

            // Ignore when the cursor is within the rectangular trough between
            // the two elements. Since the triangle is created from the cursor
            // point, which can start beyond the ref element's edge, traversing
            // back and forth from the ref to the floating element can cause it
            // to close. This ensures it always remains open in that case.
            // (`safePolygon.ts:212-260`)
            let is_inside_trough_rect = match side {
                floating_ui_dom::Side::Top => is_inside_axis_aligned_rect(
                    client_x,
                    client_y,
                    left,
                    ref_rect.top + 1.0,
                    right,
                    rect.bottom - 1.0,
                ),
                floating_ui_dom::Side::Bottom => is_inside_axis_aligned_rect(
                    client_x,
                    client_y,
                    left,
                    rect.top + 1.0,
                    right,
                    ref_rect.bottom - 1.0,
                ),
                floating_ui_dom::Side::Left => is_inside_axis_aligned_rect(
                    client_x,
                    client_y,
                    rect.right - 1.0,
                    bottom,
                    ref_rect.left + 1.0,
                    top,
                ),
                floating_ui_dom::Side::Right => is_inside_axis_aligned_rect(
                    client_x,
                    client_y,
                    ref_rect.right - 1.0,
                    bottom,
                    rect.left + 1.0,
                    top,
                ),
            };

            if is_inside_trough_rect {
                return;
            }

            // `hasLanded` miss (`safePolygon.ts:262-265`).
            if has_landed.get() && !is_inside_rect(client_x, client_y, &ref_rect) {
                close_if_no_open_child();
                return;
            }

            // Slow-cursor miss (`safePolygon.ts:267-270`).
            if !is_leave && is_cursor_moving_slowly(client_x, client_y) {
                close_if_no_open_child();
                return;
            }

            // The quadrilateral test per placement (`safePolygon.ts:274-432`).
            let is_inside_polygon = match side {
                floating_ui_dom::Side::Top => {
                    let cursor_x_offset = if is_floating_wider {
                        POLYGON_BUFFER / 2.0
                    } else {
                        POLYGON_BUFFER * 4.0
                    };
                    let cursor_point_one_x = if is_floating_wider {
                        x + cursor_x_offset
                    } else if cursor_leave_from_right {
                        x + cursor_x_offset
                    } else {
                        x - cursor_x_offset
                    };
                    let cursor_point_two_x = if is_floating_wider {
                        x - cursor_x_offset
                    } else if cursor_leave_from_right {
                        x + cursor_x_offset
                    } else {
                        x - cursor_x_offset
                    };
                    let cursor_point_y = y + POLYGON_BUFFER + 1.0;

                    let common_y_left = if cursor_leave_from_right {
                        rect.bottom - POLYGON_BUFFER
                    } else if is_floating_wider {
                        rect.bottom - POLYGON_BUFFER
                    } else {
                        rect.top
                    };
                    let common_y_right = if cursor_leave_from_right {
                        if is_floating_wider {
                            rect.bottom - POLYGON_BUFFER
                        } else {
                            rect.top
                        }
                    } else {
                        rect.bottom - POLYGON_BUFFER
                    };

                    is_point_in_quadrilateral(
                        client_x,
                        client_y,
                        cursor_point_one_x,
                        cursor_point_y,
                        cursor_point_two_x,
                        cursor_point_y,
                        rect.left,
                        common_y_left,
                        rect.right,
                        common_y_right,
                    )
                }
                floating_ui_dom::Side::Bottom => {
                    let cursor_x_offset = if is_floating_wider {
                        POLYGON_BUFFER / 2.0
                    } else {
                        POLYGON_BUFFER * 4.0
                    };
                    let cursor_point_one_x = if is_floating_wider {
                        x + cursor_x_offset
                    } else if cursor_leave_from_right {
                        x + cursor_x_offset
                    } else {
                        x - cursor_x_offset
                    };
                    let cursor_point_two_x = if is_floating_wider {
                        x - cursor_x_offset
                    } else if cursor_leave_from_right {
                        x + cursor_x_offset
                    } else {
                        x - cursor_x_offset
                    };
                    let cursor_point_y = y - POLYGON_BUFFER;

                    let common_y_left = if cursor_leave_from_right {
                        rect.top + POLYGON_BUFFER
                    } else if is_floating_wider {
                        rect.top + POLYGON_BUFFER
                    } else {
                        rect.bottom
                    };
                    let common_y_right = if cursor_leave_from_right {
                        if is_floating_wider {
                            rect.top + POLYGON_BUFFER
                        } else {
                            rect.bottom
                        }
                    } else {
                        rect.top + POLYGON_BUFFER
                    };

                    is_point_in_quadrilateral(
                        client_x,
                        client_y,
                        cursor_point_one_x,
                        cursor_point_y,
                        cursor_point_two_x,
                        cursor_point_y,
                        rect.left,
                        common_y_left,
                        rect.right,
                        common_y_right,
                    )
                }
                floating_ui_dom::Side::Left => {
                    let cursor_y_offset = if is_floating_taller {
                        POLYGON_BUFFER / 2.0
                    } else {
                        POLYGON_BUFFER * 4.0
                    };
                    let cursor_point_one_y = if is_floating_taller {
                        y + cursor_y_offset
                    } else if cursor_leave_from_bottom {
                        y + cursor_y_offset
                    } else {
                        y - cursor_y_offset
                    };
                    let cursor_point_two_y = if is_floating_taller {
                        y - cursor_y_offset
                    } else if cursor_leave_from_bottom {
                        y + cursor_y_offset
                    } else {
                        y - cursor_y_offset
                    };
                    let cursor_point_x = x + POLYGON_BUFFER + 1.0;

                    let common_x_top = if cursor_leave_from_bottom {
                        rect.right - POLYGON_BUFFER
                    } else if is_floating_taller {
                        rect.right - POLYGON_BUFFER
                    } else {
                        rect.left
                    };
                    let common_x_bottom = if cursor_leave_from_bottom {
                        if is_floating_taller {
                            rect.right - POLYGON_BUFFER
                        } else {
                            rect.left
                        }
                    } else {
                        rect.right - POLYGON_BUFFER
                    };

                    is_point_in_quadrilateral(
                        client_x,
                        client_y,
                        common_x_top,
                        rect.top,
                        common_x_bottom,
                        rect.bottom,
                        cursor_point_x,
                        cursor_point_one_y,
                        cursor_point_x,
                        cursor_point_two_y,
                    )
                }
                floating_ui_dom::Side::Right => {
                    let cursor_y_offset = if is_floating_taller {
                        POLYGON_BUFFER / 2.0
                    } else {
                        POLYGON_BUFFER * 4.0
                    };
                    let cursor_point_one_y = if is_floating_taller {
                        y + cursor_y_offset
                    } else if cursor_leave_from_bottom {
                        y + cursor_y_offset
                    } else {
                        y - cursor_y_offset
                    };
                    let cursor_point_two_y = if is_floating_taller {
                        y - cursor_y_offset
                    } else if cursor_leave_from_bottom {
                        y + cursor_y_offset
                    } else {
                        y - cursor_y_offset
                    };
                    let cursor_point_x = x - POLYGON_BUFFER;

                    let common_x_top = if cursor_leave_from_bottom {
                        rect.left + POLYGON_BUFFER
                    } else if is_floating_taller {
                        rect.left + POLYGON_BUFFER
                    } else {
                        rect.right
                    };
                    let common_x_bottom = if cursor_leave_from_bottom {
                        if is_floating_taller {
                            rect.left + POLYGON_BUFFER
                        } else {
                            rect.right
                        }
                    } else {
                        rect.left + POLYGON_BUFFER
                    };

                    is_point_in_quadrilateral(
                        client_x,
                        client_y,
                        cursor_point_x,
                        cursor_point_one_y,
                        cursor_point_x,
                        cursor_point_two_y,
                        common_x_top,
                        rect.top,
                        common_x_bottom,
                        rect.bottom,
                    )
                }
            };

            if !is_inside_polygon {
                close_if_no_open_child();
            } else if !has_landed.get() {
                // The intent window (`safePolygon.ts:437`): a cursor inside
                // the polygon that has not landed gets one intent window to
                // arrive; further movement restarts it (the handler clears
                // the timer on entry).
                let close_if_no_open_child = Rc::clone(&close_if_no_open_child);
                timeout.start(INTENT_TIMEOUT_MS, move || close_if_no_open_child());
            }
        })
    });

    HandleClose {
        options: HandleCloseOptions {
            block_pointer_events,
            get_scope,
        },
        factory,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> ClientRectObject {
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

    // Pins `hasIntersectingEdge` (`safePolygon.ts:14-23`): the ray-cast crossing
    // requires one edge endpoint above and the other at-or-below the point's height,
    // and the crossing to sit at-or-right of the point (`pointX <= x_at_pointY`).
    #[test]
    fn has_intersecting_edge_follows_the_ray_cast_rule() {
        // Vertical edge crossing the point's height to its right: crossing counts.
        assert!(has_intersecting_edge(0.0, 0.0, 0.5, -2.0, 0.5, 2.0));
        // The same edge to the left of the point: no crossing.
        assert!(!has_intersecting_edge(1.0, 0.0, 0.5, -2.0, 0.5, 2.0));
        // Horizontal edge (both endpoints on one side of the point's height): never
        // crosses.
        assert!(!has_intersecting_edge(1.0, 0.0, 0.0, 5.0, 2.0, 5.0));
    }

    // Pins `isPointInQuadrilateral` (`safePolygon.ts:25-56`): a point inside the unit
    // square counts, a point outside does not, and the test is orientation-independent
    // (the ray-cast toggle works for either winding).
    #[test]
    fn is_point_in_quadrilateral_classifies_inside_and_outside() {
        let inside = is_point_in_quadrilateral(0.5, 0.5, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0);
        assert!(inside);

        let outside = is_point_in_quadrilateral(2.0, 0.5, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0);
        assert!(!outside);

        // Clockwise winding of the same square: same result.
        let clockwise = is_point_in_quadrilateral(0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0);
        assert!(clockwise);
    }

    // Pins `isInsideRect` (`safePolygon.ts:58-65`).
    #[test]
    fn is_inside_rect_bounds_the_point() {
        let rect = rect(0.0, 0.0, 100.0, 100.0);
        assert!(is_inside_rect(50.0, 50.0, &rect));
        assert!(is_inside_rect(0.0, 0.0, &rect), "the top-left corner is in");
        assert!(
            is_inside_rect(100.0, 100.0, &rect),
            "the bottom-right corner is in (inclusive bounds)"
        );
        assert!(!is_inside_rect(-1.0, 50.0, &rect));
        assert!(!is_inside_rect(50.0, 101.0, &rect));
    }

    // Pins `isInsideAxisAlignedRect` (`safePolygon.ts:67-81`): the corners are
    // normalized, so the traversal-order of the two corners does not matter.
    #[test]
    fn is_inside_axis_aligned_rect_normalizes_corner_order() {
        assert!(is_inside_axis_aligned_rect(
            50.0, 50.0, 0.0, 0.0, 100.0, 100.0
        ));
        assert!(
            is_inside_axis_aligned_rect(50.0, 50.0, 100.0, 100.0, 0.0, 0.0),
            "swapped corners still classify"
        );
        assert!(!is_inside_axis_aligned_rect(
            150.0, 50.0, 0.0, 0.0, 100.0, 100.0
        ));
    }
}

// The pipeline tests need a real DOM realm (elements, events, timers): they mirror
// `packages/react/src/floating-ui-react/safePolygon.test.ts`.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::cell::RefCell;

    use wasm_bindgen::JsValue;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::floating_ui::floating_root_store::{FloatingRootStore, FloatingRootStoreOptions};
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::tree::{FloatingTreeStore, SharedFloatingTreeStore};
    use crate::floating_ui::types::FloatingNodeType;
    use crate::floating_ui::use_hover_shared::HandleCloseContextBase;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// The per-placement scenario upstream's `createPlacementScenario` builds
    /// (`safePolygon.test.ts:74-123`): a 100×100 reference with a 100×100 floating
    /// element offset 120px along the placement axis, the leave point on the shared
    /// edge, the trough point 10px further, and a point far outside the corridor.
    fn scenario(
        placement: &str,
    ) -> (
        ClientRectObject,
        ClientRectObject,
        (f64, f64),
        (f64, f64),
        (f64, f64),
    ) {
        let reference_rect = rect_object(0.0, 0.0, 100.0, 100.0);
        match placement {
            "top" => (
                reference_rect,
                rect_object(0.0, -120.0, 100.0, 100.0),
                (50.0, 0.0),
                (50.0, -10.0),
                (50.0, 150.0),
            ),
            "bottom" => (
                reference_rect,
                rect_object(0.0, 120.0, 100.0, 100.0),
                (50.0, 100.0),
                (50.0, 110.0),
                (50.0, -50.0),
            ),
            "left" => (
                reference_rect,
                rect_object(-120.0, 0.0, 100.0, 100.0),
                (0.0, 50.0),
                (-10.0, 50.0),
                (150.0, 50.0),
            ),
            // "right" and the default arm.
            _ => (
                reference_rect,
                rect_object(120.0, 0.0, 100.0, 100.0),
                (100.0, 50.0),
                (110.0, 50.0),
                (-50.0, 50.0),
            ),
        }
    }

    fn rect_object(x: f64, y: f64, width: f64, height: f64) -> ClientRectObject {
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

    /// Overrides `element.getBoundingClientRect` with a closure returning a rect-shaped
    /// object (the upstream tests stub the same method,
    /// `safePolygon.test.ts:135-136`).
    fn stub_rect(element: &Element, rect: &ClientRectObject) {
        let object = js_sys::Object::new();
        let set = |name: &str, value: f64| {
            js_sys::Reflect::set(&object, &JsValue::from_str(name), &JsValue::from(value)).unwrap()
        };
        set("x", rect.x);
        set("y", rect.y);
        set("width", rect.width);
        set("height", rect.height);
        set("top", rect.top);
        set("right", rect.right);
        set("bottom", rect.bottom);
        set("left", rect.left);

        let closure =
            Closure::wrap(Box::new(move || object.clone().into()) as Box<dyn Fn() -> JsValue>);
        js_sys::Reflect::set(
            element.as_ref(),
            &JsValue::from_str("getBoundingClientRect"),
            closure.as_ref().unchecked_ref(),
        )
        .unwrap();
        closure.forget();
    }

    /// `createMouseMoveEvent` (`safePolygon.test.ts:26-37`) — an un-dispatched
    /// `mousemove` carrying the coordinates (its target reads as `None`, the stub's
    /// `target = null` default).
    fn mouse_move_event(client_x: f64, client_y: f64) -> MouseEvent {
        let init = web_sys::MouseEventInit::new();
        init.set_client_x(client_x as i32);
        init.set_client_y(client_y as i32);
        MouseEvent::new_with_mouse_event_init_dict("mousemove", &init).unwrap()
    }

    struct CloseCounter(Rc<Cell<u32>>);

    impl CloseCounter {
        fn new() -> (Self, Rc<Cell<u32>>) {
            let count = Rc::new(Cell::new(0));
            (Self(Rc::clone(&count)), count)
        }

        fn calls(&self) -> u32 {
            self.0.get()
        }
    }

    fn tree_with_child(
        parent_id: &str,
        child_id: &str,
        child_open: bool,
        with_context: bool,
    ) -> SharedFloatingTreeStore {
        let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
        let context = if with_context {
            Some(context_with_open(child_open))
        } else {
            None
        };
        tree.add_node(Rc::new(FloatingNodeType {
            id: Some(child_id.to_owned()),
            parent_id: Some(parent_id.to_owned()),
            context: RefCell::new(context),
        }));
        tree
    }

    /// A real `FloatingContext` around a store with the given open state — the node
    /// contexts carry the context handle whose `open` signal the open-child check
    /// reads. Needs an executor and an owner (the context hooks).
    fn context_with_open(open: bool) -> Rc<crate::floating_ui::types::FloatingContext> {
        crate::floating_ui::use_floating::use_base_ui_floating(
            crate::floating_ui::use_position::UsePositionOptions {
                placement: reactive_graph::wrappers::read::Signal::derive(|| {
                    floating_ui_dom::Placement::Bottom
                }),
                strategy: reactive_graph::wrappers::read::Signal::derive(|| {
                    floating_ui_dom::Strategy::Absolute
                }),
                middleware: reactive_graph::wrappers::read::Signal::derive(|| {
                    send_wrapper::SendWrapper::new(Vec::new())
                }),
                transform: reactive_graph::wrappers::read::Signal::derive(|| true),
                while_elements_mounted: None,
            },
            FloatingRootStore::new(FloatingRootStoreOptions {
                open,
                transition_status: None,
                reference_element: None,
                floating_element: None,
                trigger_elements: PopupTriggerMap::new(),
                floating_id: None,
                sync_only: false,
                nested: false,
                on_open_change: None,
            }),
        )
        .context
    }

    // `keeps open while moving through the trough on <placement>`
    // (`safePolygon.test.ts:187-208`): the trough point does not close on any
    // placement.
    #[wasm_bindgen_test]
    fn keeps_open_while_moving_through_the_trough_on_any_placement() {
        for placement in ["top", "bottom", "left", "right"] {
            let (reference_rect, floating_rect, leave_point, trough_point, _outside_point) =
                scenario(placement);
            let dom_reference = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("button")
                .unwrap();
            let floating = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();
            stub_rect(&dom_reference, &reference_rect);
            stub_rect(&floating, &floating_rect);

            let (counter, count) = CloseCounter::new();
            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            let context = HandleCloseContext {
                x: Some(leave_point.0),
                y: Some(leave_point.1),
                base: HandleCloseContextBase {
                    placement: Some(placement_from_str(placement)),
                    dom_reference: Some(dom_reference),
                    floating: Some(floating),
                    node_id: Some("root".to_owned()),
                    leave: None,
                },
                on_close: Rc::new(move || count.set(count.get() + 1)),
                tree: Some(tree),
            };

            let handler = (safe_polygon(SafePolygonOptions::default()).factory)(&context);
            handler(&mouse_move_event(trough_point.0, trough_point.1));

            assert_eq!(
                counter.calls(),
                0,
                "the trough point keeps the popup open on {placement}"
            );
        }
    }

    fn placement_from_str(placement: &str) -> floating_ui_dom::Placement {
        match placement {
            "top" => floating_ui_dom::Placement::Top,
            "bottom" => floating_ui_dom::Placement::Bottom,
            "left" => floating_ui_dom::Placement::Left,
            _ => floating_ui_dom::Placement::Right,
        }
    }

    // `closes when moving away from corridor on <placement>`
    // (`safePolygon.test.ts:210-231`): the outside point closes on every placement.
    #[wasm_bindgen_test]
    fn closes_when_moving_away_from_the_corridor_on_any_placement() {
        for placement in ["top", "bottom", "left", "right"] {
            let (reference_rect, floating_rect, leave_point, _trough_point, outside_point) =
                scenario(placement);
            let dom_reference = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("button")
                .unwrap();
            let floating = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();
            stub_rect(&dom_reference, &reference_rect);
            stub_rect(&floating, &floating_rect);

            let (counter, count) = CloseCounter::new();
            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            let context = HandleCloseContext {
                x: Some(leave_point.0),
                y: Some(leave_point.1),
                base: HandleCloseContextBase {
                    placement: Some(placement_from_str(placement)),
                    dom_reference: Some(dom_reference),
                    floating: Some(floating),
                    node_id: Some("root".to_owned()),
                    leave: None,
                },
                on_close: Rc::new(move || count.set(count.get() + 1)),
                tree: Some(tree),
            };

            let handler = (safe_polygon(SafePolygonOptions::default()).factory)(&context);
            handler(&mouse_move_event(outside_point.0, outside_point.1));

            assert_eq!(
                counter.calls(),
                1,
                "the outside point closes the popup on {placement}"
            );
        }
    }

    // `does not close when a nested child is open`
    // (`safePolygon.test.ts:129-158`): the open child suppresses both the synchronous
    // close and the intent timer.
    #[wasm_bindgen_test(async)]
    async fn does_not_close_when_a_nested_child_is_open() {
        init_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        {
            let (reference_rect, floating_rect, _leave_point, _trough_point, _outside_point) =
                scenario("right");
            let dom_reference = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("button")
                .unwrap();
            let floating = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();
            stub_rect(&dom_reference, &reference_rect);
            stub_rect(&floating, &floating_rect);

            let (counter, count) = CloseCounter::new();
            let tree = tree_with_child("root", "child", true, true);
            let context = HandleCloseContext {
                x: Some(2.0),
                y: Some(0.0),
                base: HandleCloseContextBase {
                    placement: Some(floating_ui_dom::Placement::Right),
                    dom_reference: Some(dom_reference),
                    floating: Some(floating),
                    node_id: Some("root".to_owned()),
                    leave: None,
                },
                on_close: Rc::new(move || count.set(count.get() + 1)),
                tree: Some(tree),
            };

            let handler = (safe_polygon(SafePolygonOptions::default()).factory)(&context);
            // Upstream's (3, -1) move — the opposite-side x check reads the LEFT edge
            // for right placement, so this point reaches the intent-timer path.
            handler(&mouse_move_event(3.0, -1.0));

            sleep(60).await;
            assert_eq!(
                counter.calls(),
                0,
                "the open child suppresses the close even after the intent window"
            );
        }
        owner.cleanup();
    }

    // `does not close when an open nested child is behind a contextless intermediary
    // node` (`safePolygon.test.ts:160-190`): the children walk recurses through the
    // contextless node.
    #[wasm_bindgen_test(async)]
    async fn does_not_close_when_an_open_child_is_behind_a_contextless_intermediary() {
        init_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        {
            let (reference_rect, floating_rect, _leave_point, _trough_point, _outside_point) =
                scenario("right");
            let dom_reference = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("button")
                .unwrap();
            let floating = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();
            stub_rect(&dom_reference, &reference_rect);
            stub_rect(&floating, &floating_rect);

            let (counter, count) = CloseCounter::new();
            let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
            let open_child_store = context_with_open(true);
            // `inline-root` is added with no context (`safePolygon.test.ts:212`).
            tree.add_node(Rc::new(FloatingNodeType {
                id: Some("inline-root".to_owned()),
                parent_id: Some("root".to_owned()),
                context: RefCell::new(None),
            }));
            tree.add_node(Rc::new(FloatingNodeType {
                id: Some("child".to_owned()),
                parent_id: Some("inline-root".to_owned()),
                context: RefCell::new(Some(open_child_store)),
            }));

            let context = HandleCloseContext {
                x: Some(2.0),
                y: Some(0.0),
                base: HandleCloseContextBase {
                    placement: Some(floating_ui_dom::Placement::Right),
                    dom_reference: Some(dom_reference),
                    floating: Some(floating),
                    node_id: Some("root".to_owned()),
                    leave: None,
                },
                on_close: Rc::new(move || count.set(count.get() + 1)),
                tree: Some(tree),
            };

            let handler = (safe_polygon(SafePolygonOptions::default()).factory)(&context);
            handler(&mouse_move_event(3.0, -1.0));

            sleep(60).await;
            assert_eq!(
                counter.calls(),
                0,
                "the open grandchild through the contextless intermediary suppresses the close"
            );
        }
        owner.cleanup();
    }

    // `closes after intent timeout when no nested child is open`
    // (`safePolygon.test.ts:192-221`): the same geometry with a closed child closes
    // through the 40 ms intent window.
    #[wasm_bindgen_test(async)]
    async fn closes_after_the_intent_timeout_when_no_nested_child_is_open() {
        init_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        {
            let (reference_rect, floating_rect, _leave_point, _trough_point, _outside_point) =
                scenario("right");
            let dom_reference = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("button")
                .unwrap();
            let floating = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap();
            stub_rect(&dom_reference, &reference_rect);
            stub_rect(&floating, &floating_rect);

            let (counter, count) = CloseCounter::new();
            let tree = tree_with_child("root", "child", false, true);
            let context = HandleCloseContext {
                x: Some(2.0),
                y: Some(0.0),
                base: HandleCloseContextBase {
                    placement: Some(floating_ui_dom::Placement::Right),
                    dom_reference: Some(dom_reference),
                    floating: Some(floating),
                    node_id: Some("root".to_owned()),
                    leave: None,
                },
                on_close: Rc::new(move || count.set(count.get() + 1)),
                tree: Some(tree),
            };

            let handler = (safe_polygon(SafePolygonOptions::default()).factory)(&context);
            handler(&mouse_move_event(3.0, -1.0));

            sleep(60).await;
            assert_eq!(
                counter.calls(),
                1,
                "the intent window closes the popup exactly once"
            );
        }
        owner.cleanup();
    }

    // `resets traversal state for a new handler invocation`
    // (`safePolygon.test.ts:311-339`): the second handler from one factory does not
    // inherit the first handler's `hasLanded` history.
    #[wasm_bindgen_test]
    fn resets_traversal_state_for_a_new_handler_invocation() {
        let (reference_rect, floating_rect, _leave_point, trough_point, _outside_point) =
            scenario("right");
        let dom_reference = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("button")
            .unwrap();
        let floating: Element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        stub_rect(&dom_reference, &reference_rect);
        stub_rect(&floating, &floating_rect);

        let (counter, count) = CloseCounter::new();
        let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));
        let context = HandleCloseContext {
            x: Some(100.0),
            y: Some(50.0),
            base: HandleCloseContextBase {
                placement: Some(floating_ui_dom::Placement::Right),
                dom_reference: Some(dom_reference),
                floating: Some(floating.clone()),
                node_id: Some("root".to_owned()),
                leave: None,
            },
            on_close: Rc::new(move || count.set(count.get() + 1)),
            tree: Some(tree),
        };

        let handle_close = safe_polygon(SafePolygonOptions::default());

        // The first handler lands on the floating element (the event dispatches on it,
        // so the composed path carries it — upstream passes it as the event target,
        // `safePolygon.test.ts:331-333`).
        let first_handler = (handle_close.factory)(&context);
        let landed_event = mouse_move_event(130.0, 50.0);
        floating.dispatch_event(&landed_event).unwrap();
        first_handler(&landed_event);

        // The second handler moves straight to the trough point: with shared state the
        // stale `hasLanded` would close here.
        let second_handler = (handle_close.factory)(&context);
        second_handler(&mouse_move_event(trough_point.0, trough_point.1));

        assert_eq!(
            counter.calls(),
            0,
            "the fresh handler does not inherit the first handler's movement history"
        );
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
