//! Port of `packages/utils/src/areArraysEqual.ts` (Base UI Phase A util).
//!
//! Upstream compares two arrays element-wise, with an optional per-item comparer defaulting to
//! `Object.is` (`packages/utils/src/areArraysEqual.ts:9-27`): a length mismatch is `false`
//! without comparing any items (`packages/utils/src/areArraysEqual.test.ts:13-15`), and
//! otherwise the first failing item decides
//! (`packages/utils/src/areArraysEqual.test.ts:9-11`). The comparer is authoritative — there is
//! deliberately no same-reference fast path, so a comparer that always returns `false` makes
//! `areArraysEqual(arr, arr)` return `false`
//! (`packages/utils/src/areArraysEqual.test.ts:49-55`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The upstream two-argument call shape becomes [`are_arrays_equal`]; the optional
//!   `itemComparer` parameter (`packages/utils/src/areArraysEqual.ts:12`) becomes the separate
//!   [`are_arrays_equal_by`], which takes the comparer as a required argument.
//! - The `Object.is` default is carried over as the [`ObjectIs`] trait: its `f32`/`f64` impls
//!   use the SameValue algorithm (`NaN` equals `NaN`; `0.0` and `-0.0` differ —
//!   `packages/utils/src/areArraysEqual.test.ts:26-32`), and its impls for the other covered
//!   types coincide with `==`.
//! - JS's implicit "objects compare by reference" default has no generic Rust equivalent, so
//!   user-defined types implement [`ObjectIs`] explicitly (`std::ptr::eq` replicates JS
//!   reference semantics) or pass a comparer to [`are_arrays_equal_by`].
//! - Slices replace `ReadonlyArray` (upstream never mutates its inputs).

/// The default item comparison for [`are_arrays_equal`], porting upstream's `Object.is` default
/// (`packages/utils/src/areArraysEqual.ts:12`).
///
/// `Object.is` differs from Rust's `==` only for floats: `NaN` compares equal to `NaN`, while
/// `0.0` does not compare equal to `-0.0`
/// (`packages/utils/src/areArraysEqual.ts:7`). The provided impls mirror that: `f32`/`f64` use
/// the SameValue algorithm; every other implemented type compares with `==`, which coincides
/// with `Object.is` for those types.
pub trait ObjectIs {
    /// Compares two items with `Object.is` semantics.
    fn object_is(&self, other: &Self) -> bool;
}

macro_rules! impl_object_is_by_partial_eq {
    ($($item_type:ty),* $(,)?) => {
        $(
            impl ObjectIs for $item_type {
                fn object_is(&self, other: &Self) -> bool {
                    self == other
                }
            }
        )*
    };
}

impl_object_is_by_partial_eq!(
    bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, String, &str,
);

impl ObjectIs for f32 {
    fn object_is(&self, other: &Self) -> bool {
        if self.is_nan() && other.is_nan() {
            return true;
        }
        if *self == 0.0 && *other == 0.0 {
            return self.is_sign_positive() == other.is_sign_positive();
        }
        self == other
    }
}

impl ObjectIs for f64 {
    fn object_is(&self, other: &Self) -> bool {
        if self.is_nan() && other.is_nan() {
            return true;
        }
        if *self == 0.0 && *other == 0.0 {
            return self.is_sign_positive() == other.is_sign_positive();
        }
        self == other
    }
}

/// Compares two arrays element-wise, with the upstream default `Object.is` item comparison
/// (`packages/utils/src/areArraysEqual.ts:9-27`).
///
/// A length mismatch returns `false` before any item comparison
/// (`packages/utils/src/areArraysEqual.ts:14-18`); otherwise every item pair is compared in
/// order until one fails (`packages/utils/src/areArraysEqual.ts:20-24`). Two empty arrays are
/// equal (`packages/utils/src/areArraysEqual.test.ts:22-24`). There is deliberately no
/// same-slice-reference fast path: the upstream contract requires the item comparison to run
/// even for the identical array reference
/// (`packages/utils/src/areArraysEqual.test.ts:49-55`).
pub fn are_arrays_equal<T: ObjectIs>(array1: &[T], array2: &[T]) -> bool {
    are_arrays_equal_by(array1, array2, T::object_is)
}

/// Same as [`are_arrays_equal`], with the upstream optional third parameter filled in
/// (`packages/utils/src/areArraysEqual.ts:12`).
///
/// The comparer fully replaces the default item comparison — including `Object.is` semantics
/// (`packages/utils/src/areArraysEqual.test.ts:45-47`) — and is still consulted when both
/// arguments are the same slice (`packages/utils/src/areArraysEqual.test.ts:49-55`).
pub fn are_arrays_equal_by<T, F>(array1: &[T], array2: &[T], item_comparer: F) -> bool
where
    F: Fn(&T, &T) -> bool,
{
    if array1.len() != array2.len() {
        return false;
    }

    array1
        .iter()
        .zip(array2)
        .all(|(item1, item2)| item_comparer(item1, item2))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:5-7`.
    #[test]
    fn returns_true_for_arrays_with_the_same_elements_in_the_same_order() {
        assert!(are_arrays_equal(&[1, 2, 3], &[1, 2, 3]));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:9-11`.
    #[test]
    fn returns_false_when_elements_differ() {
        assert!(!are_arrays_equal(&[1, 2, 3], &[1, 2, 4]));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:13-15`.
    #[test]
    fn returns_false_when_lengths_differ() {
        assert!(!are_arrays_equal(&[1, 2, 3], &[1, 2]));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:17-20`: passing the identical slice
    // twice compares equal under the default semantics.
    #[test]
    fn returns_true_for_the_same_array_reference() {
        let array = [1, 2, 3];
        assert!(are_arrays_equal(&array, &array));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:22-24`.
    #[test]
    fn returns_true_for_two_empty_arrays() {
        assert!(are_arrays_equal::<u8>(&[], &[]));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:26-28` (Object.is semantics).
    #[test]
    fn treats_nan_as_equal_to_nan() {
        assert!(are_arrays_equal(&[f64::NAN], &[f64::NAN]));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:30-32` (Object.is semantics).
    #[test]
    fn treats_zero_as_different_from_negative_zero() {
        assert!(!are_arrays_equal(&[0.0], &[-0.0]));
    }

    struct Item {
        id: u32,
    }

    // JS compares object items by reference by default
    // (`packages/utils/src/areArraysEqual.test.ts:34-36`); `std::ptr::eq` replicates those
    // reference semantics in Rust.
    impl ObjectIs for Item {
        fn object_is(&self, other: &Self) -> bool {
            std::ptr::eq(self, other)
        }
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:34-36`: separately constructed items
    // with equal fields are not equal under reference semantics, while the identical slice
    // still compares equal item-by-item (`packages/utils/src/areArraysEqual.test.ts:17-20`).
    #[test]
    fn compares_items_by_reference_under_reference_semantics_types() {
        let array1 = [Item { id: 1 }];
        let array2 = [Item { id: 1 }];

        assert!(!are_arrays_equal(&array1, &array2));
        assert!(are_arrays_equal(&array1, &array1));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:38-43`.
    #[test]
    fn uses_the_provided_comparer() {
        let array1 = [Item { id: 1 }, Item { id: 2 }];
        let array2 = [Item { id: 1 }, Item { id: 2 }];

        assert!(are_arrays_equal_by(
            &array1,
            &array2,
            |x: &Item, y: &Item| x.id == y.id
        ));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:45-47`: the comparer replaces the
    // default `Object.is` semantics entirely, so `0` and `-0` compare equal.
    #[test]
    fn lets_the_provided_comparer_override_object_is_semantics() {
        assert!(are_arrays_equal_by(&[0.0], &[-0.0], |x: &f64, y: &f64| x == y));
    }

    // Mirrors `packages/utils/src/areArraysEqual.test.ts:49-55`: the comparer is authoritative
    // and is consulted even when both arguments are the identical slice — a comparer returning
    // `false` forces a `false` result, proving there is no same-reference fast path.
    #[test]
    fn consults_the_provided_comparer_even_for_the_same_array_reference() {
        let array = [1, 2, 3];
        let calls = Cell::new(0);

        let result = are_arrays_equal_by(&array, &array, |_x: &i32, _y: &i32| {
            calls.set(calls.get() + 1);
            false
        });

        assert!(!result);
        assert_eq!(calls.get(), 1, "comparer ran for the first item pair");
    }
}
