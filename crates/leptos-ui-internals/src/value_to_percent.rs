//! Port of `packages/react/src/utils/valueToPercent.ts` — the single-expression helper
//! mapping a value within `[min, max]` onto the 0–100 percentage scale. Consumed by the
//! Slider/Progress/Meter positioning math (implementation.md, "Downstream consumers").
//!
//! Upstream has no test (implementation.md, "Submodules with no test anywhere": the
//! `max === min` division is unguarded and its behavior undefined), so only the
//! defined-domain behavior is pinned here.

/// `valueToPercent` (`valueToPercent.ts:1-3`).
pub fn value_to_percent(value: f64, min: f64, max: f64) -> f64 {
    ((value - min) * 100.0) / (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The defined domain: min maps to 0, max maps to 100, midpoints interpolate.
    #[test]
    fn maps_the_range_onto_the_percentage_scale() {
        assert_eq!(value_to_percent(0.0, 0.0, 100.0), 0.0);
        assert_eq!(value_to_percent(100.0, 0.0, 100.0), 100.0);
        assert_eq!(value_to_percent(50.0, 0.0, 100.0), 50.0);
        assert_eq!(value_to_percent(25.0, 10.0, 60.0), 30.0);
    }
}
