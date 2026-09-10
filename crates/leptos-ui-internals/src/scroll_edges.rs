//! Port of `packages/react/src/utils/scrollEdges.ts` — scroll-offset edge snapping for
//! the Select/ScrollArea edge-scroll interactions (behavior.md, "Edge cases →
//! `scrollEdges`").
//!
//! Upstream is two pure functions and one constant; the port keeps the same names as
//! free functions and a `&'static str`-style const. `clamp` comes from the Phase A
//! utils crate (`scrollEdges.ts:1`).

use leptos_ui_utils::clamp;

/// `SCROLL_EDGE_TOLERANCE_PX` (`scrollEdges.ts:3`) — offsets within this distance of a
/// scroll edge snap to it.
pub const SCROLL_EDGE_TOLERANCE_PX: f64 = 1.0;

/// `getMaxScrollOffset` (`scrollEdges.ts:5-7`) — the largest positive scroll offset for
/// a scrollable box (`scrollSize` is the full scrollable extent, `clientSize` the
/// visible viewport into it).
pub fn get_max_scroll_offset(scroll_size: f64, client_size: f64) -> f64 {
    (scroll_size - client_size).max(0.0)
}

/// `normalizeScrollOffset` (`scrollEdges.ts:9-27`) — snaps a scroll offset that sits
/// within [`SCROLL_EDGE_TOLERANCE_PX`] of an edge to that edge, so edge-detection
/// styling does not flicker on sub-pixel scroll positions. A non-positive `max`
/// (nothing to scroll) normalizes to 0, and when both tolerances overlap (max itself is
/// within tolerance) the closest edge wins.
pub fn normalize_scroll_offset(value: f64, max: f64) -> f64 {
    if max <= 0.0 {
        return 0.0;
    }

    let clamped = clamp(value, 0.0, max);
    let start_distance = clamped;
    let end_distance = max - clamped;
    let within_start_tolerance = start_distance <= SCROLL_EDGE_TOLERANCE_PX;
    let within_end_tolerance = end_distance <= SCROLL_EDGE_TOLERANCE_PX;

    if within_start_tolerance && within_end_tolerance {
        return if start_distance <= end_distance {
            0.0
        } else {
            max
        };
    }

    if within_start_tolerance {
        return 0.0;
    }

    if within_end_tolerance {
        return max;
    }

    clamped
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors `packages/react/src/utils/scrollEdges.test.ts:5-8`.
    #[test]
    fn returns_0_when_max_is_non_positive() {
        assert_eq!(normalize_scroll_offset(10.0, 0.0), 0.0);
        assert_eq!(normalize_scroll_offset(10.0, -5.0), 0.0);
    }

    // Mirrors `packages/react/src/utils/scrollEdges.test.ts:10-13`.
    #[test]
    fn snaps_to_the_start_edge_within_the_tolerance() {
        assert_eq!(normalize_scroll_offset(0.5, 10.0), 0.0);
    }

    // Mirrors `packages/react/src/utils/scrollEdges.test.ts:15-18`.
    #[test]
    fn snaps_to_the_end_edge_within_the_tolerance() {
        assert_eq!(normalize_scroll_offset(9.5, 10.0), 10.0);
    }

    // Mirrors `packages/react/src/utils/scrollEdges.test.ts:20-23`.
    #[test]
    fn keeps_values_away_from_edges_unchanged() {
        assert_eq!(normalize_scroll_offset(5.0, 10.0), 5.0);
    }

    // Mirrors `packages/react/src/utils/scrollEdges.test.ts:25-30` — when max equals
    // the tolerance both edge tolerances cover the whole range and the closest edge
    // wins on both sides of the midpoint.
    #[test]
    fn chooses_the_closest_edge_when_tolerances_overlap() {
        let max = SCROLL_EDGE_TOLERANCE_PX;

        assert_eq!(normalize_scroll_offset(0.0, max), 0.0);
        assert_eq!(normalize_scroll_offset(max, max), max);
        assert_eq!(normalize_scroll_offset(max * 0.4, max), 0.0);
        assert_eq!(normalize_scroll_offset(max * 0.6, max), max);
    }

    // Mirrors the export contract (`scrollEdges.ts:5-7`, implementation.md "Tested
    // modules with untested surface": `getMaxScrollOffset` is exported and consumed by
    // Select but absent from behavior.md) — clamps a negative range to 0.
    #[test]
    fn get_max_scroll_offset_clamps_a_negative_range_to_0() {
        assert_eq!(get_max_scroll_offset(300.0, 100.0), 200.0);
        assert_eq!(get_max_scroll_offset(100.0, 300.0), 0.0);
    }
}
