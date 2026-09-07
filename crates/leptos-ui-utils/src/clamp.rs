//! Port of `packages/utils/src/clamp.ts` (Base UI Phase A util).
//!
//! Upstream is a stateless pure function: `Math.max(min, Math.min(val, max))`, where `min`
//! defaults to `Number.MIN_SAFE_INTEGER` and `max` to `Number.MAX_SAFE_INTEGER`
//! (`packages/utils/src/clamp.ts:1-7`). The first argument is the value being clamped and the
//! second/third are the lower/upper bounds (`packages/utils/src/clamp.test.ts:6-7`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The function is generic over `PartialOrd` instead of JS-number-only: JS values are `f64`,
//!   but the Rust port clamps integer quantities (indices, counters) just as often. For every
//!   `f64`-expressible non-`NaN` input the comparisons reproduce the upstream
//!   `Math.max`/`Math.min` composition exactly.
//! - The upstream default-argument call shapes (`packages/utils/src/clamp.ts:3-4`) have no
//!   direct Rust equivalent; the defaults are exposed as the [`MIN_SAFE_INTEGER`] and
//!   [`MAX_SAFE_INTEGER`] constants so any default-bounds call can be written as
//!   `clamp(value, MIN_SAFE_INTEGER, MAX_SAFE_INTEGER)`.
//! - `NaN` bounds: the spec marks `NaN` inputs as unverified
//!   (`specs/utils/clamp.md`, "Unproven behaviors"), so the port keeps the natural comparison
//!   result — a `NaN` *value* returns `NaN` (matching upstream's `Math.min`/`Math.max`
//!   propagation), while a `NaN` *bound* is simply never selected, returning `value` (upstream
//!   would propagate the `NaN`).

/// The upstream `min` default, porting `Number.MIN_SAFE_INTEGER`
/// (`packages/utils/src/clamp.ts:3`).
pub const MIN_SAFE_INTEGER: f64 = -9_007_199_254_740_991.0;

/// The upstream `max` default, porting `Number.MAX_SAFE_INTEGER`
/// (`packages/utils/src/clamp.ts:4`).
pub const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// Clamps `value` into the inclusive `[min, max]` range
/// (`packages/utils/src/clamp.ts:6`).
///
/// A value below `min` returns `min` (`packages/utils/src/clamp.test.ts:6`,
/// `packages/utils/src/clamp.test.ts:8`), a value above `max` returns `max`
/// (`packages/utils/src/clamp.test.ts:7`), and a value inside the range — including the exact
/// boundaries — is returned unchanged. When `min > max` the upstream
/// `Math.max(min, Math.min(val, max))` composition always resolves to `min`, and the
/// comparison-based implementation here does the same. Note this differs from
/// [`Ord::clamp`], which panics on an inverted range.
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors `packages/utils/src/clamp.test.ts:5-9`.
    #[test]
    fn clamps_a_value_based_on_min_and_max() {
        assert_eq!(clamp(1.0, 2.0, 4.0), 2.0);
        assert_eq!(clamp(5.0, 2.0, 4.0), 4.0);
        assert_eq!(clamp(-5.0, -1.0, 5.0), -1.0);
    }

    // Unproven but inferred from the upstream composition (`packages/utils/src/clamp.ts:6`):
    // an in-range value, including the exact boundaries, is returned unchanged.
    #[test]
    fn passes_an_in_range_value_through_unchanged() {
        assert_eq!(clamp(3.0, 2.0, 4.0), 3.0);
        assert_eq!(clamp(2.0, 2.0, 4.0), 2.0);
        assert_eq!(clamp(4.0, 2.0, 4.0), 4.0);
    }

    // Upstream's `Math.max(min, Math.min(val, max))` resolves to `min` whenever `min > max`
    // (`packages/utils/src/clamp.ts:6`); the port reproduces that instead of panicking like
    // `Ord::clamp`.
    #[test]
    fn returns_min_when_the_bounds_are_inverted() {
        assert_eq!(clamp(3.0, 4.0, 2.0), 4.0);
    }

    // JS numbers are `f64`, but the port is generic: integer clamps must behave identically.
    #[test]
    fn works_for_integer_types() {
        assert_eq!(clamp(5i32, 1, 3), 3);
        assert_eq!(clamp(0i64, 1, 3), 1);
    }

    // `NaN` is unverified upstream (`specs/utils/clamp.md`, "Unproven behaviors"); the port
    // keeps the natural comparison result: a `NaN` value propagates (like upstream's
    // `Math.min`/`Math.max`), a `NaN` bound is never selected.
    #[test]
    fn handles_nan_inputs_with_the_documented_comparison_semantics() {
        assert!(clamp(f64::NAN, 2.0, 4.0).is_nan());
        assert_eq!(clamp(3.0, f64::NAN, 4.0), 3.0);
        assert_eq!(clamp(3.0, 2.0, f64::NAN), 3.0);
    }

    // The upstream default bounds (`packages/utils/src/clamp.ts:3-4`) are the safe-integer
    // range; clamping into it must reproduce the default-argument call shape.
    #[test]
    fn clamps_to_the_safe_integer_range_using_the_default_bounds() {
        assert_eq!(MIN_SAFE_INTEGER, -9_007_199_254_740_991.0);
        assert_eq!(MAX_SAFE_INTEGER, 9_007_199_254_740_991.0);
        assert_eq!(
            clamp(1e20, MIN_SAFE_INTEGER, MAX_SAFE_INTEGER),
            MAX_SAFE_INTEGER
        );
        assert_eq!(
            clamp(-1e20, MIN_SAFE_INTEGER, MAX_SAFE_INTEGER),
            MIN_SAFE_INTEGER
        );
    }
}
