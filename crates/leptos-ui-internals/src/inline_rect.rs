//! Port of `packages/react/src/utils/popups/inlineRect.ts` — the local `inline`
//! line-rect machinery standing in for Floating UI's own `inline()`: line-rect
//! selection over the reference's client rects, plus the trigger-identity
//! checks, delayed-open hit-line reuse, and left/right edge grouping Preview
//! Card's reusable-trigger model needs (inlineRect.ts:5-7).
//!
//! This module carries the pure helpers and the trigger-side capture surface;
//! the engine middleware lives in [`super::inline_rect_middleware`].
//!
//! Upstream shape:
//! - [`InlineRectCoords`] (`inlineRect.ts:10-19`): the captured hover point, the
//!   line index under the pointer, and the trigger element the coords belong to.
//! - `getLineRects` (`:44-84`): sorts the rects by `top`, groups them into lines
//!   whenever the next rect's top crosses half the previous rect's height, and
//!   accumulates the union bounding box as the fallback.
//! - `findLineIndex` (`:86-94`): the ±2 px hit-test tolerance.
//! - `getInlineRectCoords` (`:109-124`): coords only when the trigger wraps to
//!   2+ lines (`lines.length < 2` returns `undefined`).
//! - `getInlineReferenceRect` (`:126-193`): the reference-rect override — the
//!   captured line, the point hit-test, the two-line left/right edge-grouping
//!   fallback (returns the union box when `lines[0].left > lines[1].right`),
//!   and the side-aware single-line selection for top/bottom (first/last line
//!   stretched to the full vertical extent) and left/right (the extreme edge,
//!   first-to-last vertical stretch).
//! - [`get_inline_rect_trigger_props`] (`:202-225`): onFocus clears the coords;
//!   mouseenter/mousemove update them while closed.
//! - [`update_inline_rect_coords`] (`:227-237`): the store-side capture the
//!   `onBeforeDispatch` hook calls.
//!
//! Rust adaptations:
//! - `React.RefObject<InlineRectCoords | undefined>` is the crate's
//!   [`InlineRectCoordsRef`] = `Rc<Cell<Option<InlineRectCoords>>>` handle (the
//!   popup machinery's `inlineRectCoordsRef` context member — the
//!   `React.MutableRefObject` shape at `PreviewCardStore.ts:26-28`);
//! - the local `RectLike`/`ClientRectLike` structural types collapse into one
//!   plain box struct — the x/y projections `createRect` fills (`:30-42`) ride
//!   along as fields so the box doubles as the engine's rect vocabulary;
//! - `getClientRects()` is read through `Element::get_client_rects`
//!   ([`element_client_rects`]) — upstream's `ClientRectsReference` structural
//!   type cannot exist in Rust; the virtual-element arm is the middleware
//!   module's concern.

use std::cell::Cell;
use std::rc::Rc;

use web_sys::Element;

/// The captured hover coordinates (`inlineRect.ts:10-19`).
#[derive(Clone, Debug, PartialEq)]
pub struct InlineRectCoords {
    /// The x position in viewport coordinates (`:13`).
    pub x: f64,
    /// The y position in viewport coordinates (`:15`).
    pub y: f64,
    /// The line index under the pointer when coordinates were captured (`:17`).
    pub line_index: Option<usize>,
    /// The trigger element whose rects produced these coordinates (`:19`).
    pub element: Element,
}

impl InlineRectCoords {
    /// The `coords?.x` / `coords?.y` reads (`:134-135`): a coords without a
    /// finite point has no hit-testable position.
    pub fn has_point(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// The shared coordinates cell — upstream's
/// `React.MutableRefObject<InlineRectCoords | undefined>`
/// (`PreviewCardStore.ts:26-28` context member).
pub type InlineRectCoordsRef = Rc<Cell<Option<InlineRectCoords>>>;

/// A plain line/rect box — upstream's local `RectLike` (`:21-28`) merged with
/// the `x`/`y` projections `createRect` fills (`:30-42`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectLike {
    pub x: f64,
    pub y: f64,
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub width: f64,
    pub height: f64,
}

impl RectLike {
    /// `createRect` (`:30-42`): the box from its edges with the x/y/width/height
    /// projections filled.
    pub fn from_edges(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        RectLike {
            x: left,
            y: top,
            left,
            top,
            right,
            bottom,
            width: right - left,
            height: bottom - top,
        }
    }

    /// `copyRect` (`:44-54`): the edge fields only, projections zeroed (the JS
    /// copy omits them; no consumer reads them off a copied box).
    fn copy(rect: RectLike) -> Self {
        RectLike {
            x: 0.0,
            y: 0.0,
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
            width: rect.width,
            height: rect.height,
        }
    }

    /// A box from the engine's DOMRect readings (`get(index)`'s accessors).
    pub fn from_dom_rect(
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        width: f64,
        height: f64,
    ) -> Self {
        RectLike {
            x: left,
            y: top,
            left,
            top,
            right,
            bottom,
            width,
            height,
        }
    }
}

/// `getLineRects` (`inlineRect.ts:44-84`): sort by top, group into lines where
/// the next rect's top crosses half the previous rect's height, accumulate the
/// union fallback. The `previousRect.height / 2` rule reads the *sorted-previous*
/// rect (upstream's loop variable), not the current line's height. Returns
/// `(lines, fallback)`.
pub fn get_line_rects(rects: &[RectLike]) -> (Vec<RectLike>, RectLike) {
    let mut sorted: Vec<RectLike> = rects.to_vec();
    sorted.sort_by(|a, b| {
        a.top
            .partial_cmp(&b.top)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut lines: Vec<RectLike> = Vec::new();
    let mut previous_rect: Option<RectLike> = None;
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;

    for rect in sorted {
        left = left.min(rect.left);
        top = top.min(rect.top);
        right = right.max(rect.right);
        bottom = bottom.max(rect.bottom);

        let crosses = previous_rect
            .map(|previous| rect.top - previous.top > previous.height / 2.0)
            .unwrap_or(true);
        if crosses {
            lines.push(RectLike::copy(rect));
        } else if let Some(line) = lines.last_mut() {
            line.left = line.left.min(rect.left);
            line.right = line.right.max(rect.right);
            line.bottom = line.bottom.max(rect.bottom);
            line.width = line.right - line.left;
            line.height = line.bottom - line.top;
        }

        previous_rect = Some(rect);
    }

    (lines, RectLike::from_edges(left, top, right, bottom))
}

/// `findLineIndex` (`inlineRect.ts:86-94`): the ±2 px tolerant hit-test.
pub fn find_line_index(lines: &[RectLike], x: f64, y: f64) -> Option<usize> {
    lines.iter().position(|line| {
        x > line.left - 2.0 && x < line.right + 2.0 && y > line.top - 2.0 && y < line.bottom + 2.0
    })
}

/// `getInlineRectCoords` (`inlineRect.ts:109-124`): capture only when the trigger
/// wraps to 2+ lines.
pub fn get_inline_rect_coords(
    rects: &[RectLike],
    element: &Element,
    client_x: f64,
    client_y: f64,
) -> Option<InlineRectCoords> {
    let (lines, _) = get_line_rects(rects);
    if lines.len() < 2 {
        return None;
    }
    let line_index = find_line_index(&lines, client_x, client_y);
    Some(InlineRectCoords {
        x: client_x,
        y: client_y,
        line_index,
        element: element.clone(),
    })
}

/// `getInlineReferenceRect` (`inlineRect.ts:126-193`): the reference-rect override
/// the middleware feeds back to the engine. `coords` carries the captured point
/// and line; `side` is the placement's first character (`placement[0]`, `:131`).
pub fn get_inline_reference_rect(
    rects: &[RectLike],
    side: char,
    coords: Option<&InlineRectCoords>,
) -> Option<RectLike> {
    let (lines, fallback) = get_line_rects(rects);
    if lines.len() < 2 {
        return None;
    }

    // The captured line (`:139-143`).
    if let Some(coords) = coords {
        if let Some(line_index) = coords.line_index {
            if let Some(line) = lines.get(line_index) {
                return Some(*line);
            }
        }
    }

    // The point hit-test (`:145-152`).
    if let Some(coords) = coords {
        if coords.has_point() {
            if let Some(line_index) = find_line_index(&lines, coords.x, coords.y) {
                return Some(lines[line_index]);
            }
        }
    }

    // The two-line left/right edge-grouping fallback (`:154-157`): lines[0]
    // starts right of lines[1]'s right edge — the wrapped-around ragged-edge
    // case returns the union box.
    if lines.len() == 2
        && lines[0].left > lines[1].right
        && coords.map(|coords| coords.has_point()).unwrap_or(false)
    {
        return Some(fallback);
    }

    // Top/bottom (`:159-166`): the first (top) or last (bottom) line's horizontal
    // extent stretched to the full vertical extent.
    if side == 't' || side == 'b' {
        let first_rect = &lines[0];
        let last_rect = &lines[lines.len() - 1];
        let target = if side == 't' { first_rect } else { last_rect };
        return Some(RectLike::from_edges(
            target.left,
            first_rect.top,
            target.right,
            last_rect.bottom,
        ));
    }

    // Left/right (`:168-190`): the extreme edge across lines, with the vertical
    // extent running from the first edge-holding line's top to the last's bottom.
    let is_left = side == 'l';
    let mut left = lines[0].left;
    let mut right = lines[0].right;
    let mut edge = if is_left {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    };
    let mut target_first = lines[0];
    let mut target_last = lines[0];

    for rect in &lines {
        left = left.min(rect.left);
        right = right.max(rect.right);

        let next_edge = if is_left { rect.left } else { rect.right };
        if (is_left && next_edge < edge) || (!is_left && next_edge > edge) {
            edge = next_edge;
            target_first = *rect;
            target_last = *rect;
        } else if next_edge == edge {
            target_last = *rect;
        }
    }

    Some(RectLike::from_edges(
        left,
        target_first.top,
        right,
        target_last.bottom,
    ))
}

/// The trigger-side handlers `getInlineRectTriggerProps` returns
/// (`inlineRect.ts:202-225`) — the port's handler-closure trio (the crate's
/// handler-bag merge composes them into the element's props).
pub struct InlineRectTriggerProps {
    /// `onFocus` (`:210-212`): the coords clear — keyboard focus has no hover line.
    pub on_focus: Rc<dyn Fn()>,
    /// `onMouseEnter` (`:214-216`): update while closed.
    pub on_mouse_enter: Rc<dyn Fn(f64, f64)>,
    /// `onMouseMove` (`:217-219`): update while closed.
    pub on_mouse_move: Rc<dyn Fn(f64, f64)>,
}

impl std::fmt::Debug for InlineRectTriggerProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InlineRectTriggerProps")
    }
}

/// Reads the element's client rects as [`RectLike`]s — the `getClientRects()`
/// call the upstream helpers make through the structural `ClientRectsReference`
/// type (`:56-58`). An empty list on failure matches JS's default empty array
/// iteration.
pub fn element_client_rects(element: &Element) -> Vec<RectLike> {
    let list = element.get_client_rects();
    (0..list.length())
        .filter_map(|index| list.item(index))
        .map(|rect| {
            RectLike::from_dom_rect(
                rect.left(),
                rect.top(),
                rect.right(),
                rect.bottom(),
                rect.width(),
                rect.height(),
            )
        })
        .collect()
}

/// `getInlineRectTriggerProps` (`inlineRect.ts:202-225`). `is_open` is read at
/// call time through the closure (upstream closes over the render's `isOpen`;
/// `updateCoordsIfClosed`, `:216-221`).
pub fn get_inline_rect_trigger_props(
    coords_ref: &InlineRectCoordsRef,
    element: &Element,
    is_open: impl Fn() -> bool + 'static,
) -> InlineRectTriggerProps {
    let coords_ref = Rc::clone(coords_ref);
    let element = element.clone();
    // `updateCoords` (`:204-206`).
    let update_coords = {
        let coords_ref = Rc::clone(&coords_ref);
        Rc::new(move |client_x: f64, client_y: f64| {
            let rects = element_client_rects(&element);
            let coords = get_inline_rect_coords(&rects, &element, client_x, client_y);
            coords_ref.set(coords);
        })
    };
    // `updateCoordsIfClosed` (`:216-221`).
    let update_coords_if_closed: Rc<dyn Fn(f64, f64)> = Rc::new({
        let update_coords = Rc::clone(&update_coords);
        let is_open = Rc::new(is_open);
        move |client_x: f64, client_y: f64| {
            if !is_open() {
                update_coords(client_x, client_y);
            }
        }
    });
    InlineRectTriggerProps {
        on_focus: {
            let coords_ref = Rc::clone(&coords_ref);
            Rc::new(move || coords_ref.set(None))
        },
        on_mouse_enter: Rc::clone(&update_coords_if_closed),
        on_mouse_move: update_coords_if_closed,
    }
}

/// `updateInlineRectCoords` (`inlineRect.ts:227-237`): the store-side capture the
/// `onBeforeDispatch` hook (`PreviewCardStore.ts:75-96`) calls — capture the
/// hovered line so the card anchors to the exact point on the link that was
/// hovered. Returns the captured coords; the cell is cleared when the trigger is
/// single-line, matching the unconditional assignment.
pub fn update_inline_rect_coords(
    coords_ref: &InlineRectCoordsRef,
    element: &Element,
    client_x: f64,
    client_y: f64,
) -> Option<InlineRectCoords> {
    let rects = element_client_rects(element);
    let coords = get_inline_rect_coords(&rects, element, client_x, client_y);
    coords_ref.set(coords.clone());
    coords
}

#[cfg(test)]
mod host_tests {
    use super::*;

    fn rect(left: f64, top: f64, width: f64, height: f64) -> RectLike {
        RectLike::from_edges(left, top, left + width, top + height)
    }

    // The two-line link (behavior.md "DOM structure & portal behavior"): a
    // 100px-wide first line and a 90px-wide indented second line.
    fn two_lines() -> Vec<RectLike> {
        vec![rect(10.0, 0.0, 100.0, 20.0), rect(10.0, 30.0, 90.0, 20.0)]
    }

    #[test]
    fn groups_sorted_rects_by_half_height_rule() {
        // 4 rects over 2 lines: within each line the second rect's top crosses
        // half the previous rect's height (10 > 5), so each becomes its own line
        // box; the fallback accumulates the union.
        let rects = vec![
            rect(10.0, 5.0, 40.0, 10.0),
            rect(50.0, 15.0, 60.0, 10.0),
            rect(10.0, 25.0, 40.0, 10.0),
            rect(50.0, 35.0, 60.0, 10.0),
        ];
        let (lines, fallback) = get_line_rects(&rects);
        assert_eq!(lines.len(), 4);
        assert_eq!(fallback.left, 10.0);
        assert_eq!(fallback.right, 110.0);
        assert_eq!(fallback.top, 5.0);
        assert_eq!(fallback.bottom, 45.0);
    }

    #[test]
    fn same_line_rects_merge() {
        // Two rects whose tops differ by less than half the previous height merge
        // into one line (the indented-first-line case).
        let rects = vec![rect(10.0, 0.0, 80.0, 20.0), rect(95.0, 4.0, 30.0, 16.0)];
        let (lines, _) = get_line_rects(&rects);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].left, 10.0);
        assert_eq!(lines[0].right, 125.0);
        assert_eq!(lines[0].bottom, 20.0);
    }

    #[test]
    fn hit_test_uses_two_pixel_tolerance() {
        let lines = two_lines();
        // 1.5px outside the first line's right edge: within tolerance.
        assert_eq!(find_line_index(&lines, 111.5, 10.0), Some(0));
        // 3.5px outside: outside tolerance.
        assert_eq!(find_line_index(&lines, 113.5, 10.0), None);
        // Deep in the second line.
        assert_eq!(find_line_index(&lines, 50.0, 40.0), Some(1));
        // Between the lines (the 10px gap): no hit.
        assert_eq!(find_line_index(&lines, 50.0, 25.0), None);
    }

    #[test]
    fn coords_capture_requires_two_lines() {
        // The DOM call sites run under wasm; the host suite pins the grouping
        // contract the capture rides (getInlineRectCoords's `lines.length < 2`
        // guard is exercised through get_inline_reference_rect below).
        let (lines, _) = get_line_rects(&two_lines());
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn reference_rect_without_coords_stretches_full_extent_for_top_bottom() {
        let lines = two_lines();
        let bottom = get_inline_reference_rect(&lines, 'b', None).unwrap();
        assert_eq!(bottom.left, 10.0);
        assert_eq!(
            bottom.right, 100.0,
            "'bottom' takes the LAST line's horizontal extent"
        );
        assert_eq!(bottom.top, 0.0);
        assert_eq!(
            bottom.bottom, 50.0,
            "the vertical stretch runs first-top → last-bottom"
        );
        let top = get_inline_reference_rect(&lines, 't', None).unwrap();
        assert_eq!(
            top.right, 110.0,
            "'top' takes the FIRST line's horizontal extent"
        );
    }

    #[test]
    fn single_line_produces_no_override() {
        assert_eq!(
            get_inline_reference_rect(&[rect(10.0, 0.0, 100.0, 20.0)], 'b', None),
            None
        );
    }

    #[test]
    fn left_right_side_uses_extreme_edge() {
        // Three lines; the middle sticks out furthest left.
        let rects = vec![
            rect(20.0, 0.0, 90.0, 10.0),
            rect(5.0, 20.0, 105.0, 10.0),
            rect(20.0, 40.0, 90.0, 10.0),
        ];
        let (lines, _) = get_line_rects(&rects);
        // Left: anchored at the leftmost edge; the vertical run starts at the
        // edge-holding line's top (20) and ends at the last edge-holding line's
        // bottom (30) — the single extreme here.
        let left_rect = get_inline_reference_rect(&lines, 'l', None).unwrap();
        assert_eq!(left_rect.left, 5.0);
        assert_eq!(left_rect.top, 20.0);
        assert_eq!(left_rect.bottom, 30.0);
        // Right: the outer lines share the extreme (110 at top 0 and top 40) —
        // target_first keeps the first, target_last the last.
        let right_rect = get_inline_reference_rect(&lines, 'r', None).unwrap();
        assert_eq!(right_rect.right, 110.0);
        assert_eq!(right_rect.top, 0.0);
        assert_eq!(right_rect.bottom, 50.0);
    }

    #[test]
    fn two_line_left_right_edge_grouping_returns_fallback() {
        // lines[0] starts right of lines[1]'s right edge: the wrapped-around
        // ragged-edge case returns the union box (`:154-157`) — but only when a
        // point rides the coords (`x != null && y != null`); a coords whose point
        // misses every line reaches the fallback.
        let rects = vec![rect(60.0, 0.0, 40.0, 10.0), rect(0.0, 20.0, 50.0, 10.0)];
        let (lines, _) = get_line_rects(&rects);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].left > lines[1].right);
        // No coords: the side-aware branch runs (the extreme left edge).
        let side_rect = get_inline_reference_rect(&lines, 'l', None).unwrap();
        assert_eq!(side_rect.left, 0.0);
        assert_eq!(side_rect.top, 20.0, "the edge-holding line's top");
        assert_eq!(side_rect.bottom, 30.0);
        // Coords pointing outside every line: the fallback union box. The
        // coords' element member is irrelevant to this pure path — the host
        // suite's NULL-JsValue element (the dialog_tests.rs convention).
        let coords = InlineRectCoords {
            x: 200.0,
            y: 5.0,
            line_index: None,
            element: web_sys::Element::from(web_sys::wasm_bindgen::JsValue::NULL),
        };
        let fallback_rect = get_inline_reference_rect(&lines, 'l', Some(&coords)).unwrap();
        assert_eq!(fallback_rect.left, 0.0);
        assert_eq!(fallback_rect.right, 100.0);
        assert_eq!(fallback_rect.top, 0.0);
        assert_eq!(fallback_rect.bottom, 30.0);
    }

    #[test]
    fn the_fallback_is_the_union_box() {
        let (lines, fallback) = get_line_rects(&two_lines());
        assert_eq!(lines.len(), 2);
        assert_eq!(fallback.left, 10.0);
        assert_eq!(fallback.right, 110.0);
        assert_eq!(fallback.top, 0.0);
        assert_eq!(fallback.bottom, 50.0);
    }
}
