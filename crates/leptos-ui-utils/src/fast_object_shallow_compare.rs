//! Port of `packages/utils/src/fastObjectShallowCompare.ts` (Base UI Phase A util).
//!
//! Upstream has no test file at all (`ralph/generated/utils.json:70-76` records `testFiles: []`
//! for the unit), so every behavioral claim here traces to the implementation itself, per
//! `specs/utils/fastObjectShallowCompare.md`. The file's only comment is provenance, linking the
//! implementation to mui-x's vendored `x-internals/fastObjectShallowCompare`
//! (`packages/utils/src/fastObjectShallowCompare.ts:1`) — there is no runtime delegation to an
//! external npm package and no external crate to bind against, so the algorithm is implemented
//! here.
//!
//! The algorithm (`packages/utils/src/fastObjectShallowCompare.ts:4-32`):
//!
//! 1. Identity fast path — `a === b` returns `true` before any comparison
//!    (`packages/utils/src/fastObjectShallowCompare.ts:5-7`).
//! 2. A per-key pass over `a`'s enumerable keys: each value pair must satisfy `Object.is`
//!    (`packages/utils/src/fastObjectShallowCompare.ts:19`) and each key must be present on `b`
//!    (`packages/utils/src/fastObjectShallowCompare.ts:22-24`); the first failure returns
//!    `false`, so a `false` result can short-circuit before every key was visited.
//! 3. A final key-count equality between the two records
//!    (`packages/utils/src/fastObjectShallowCompare.ts:28-31`), which is what rejects keys
//!    present only on `b` — a key of `a` missing from `b` is already rejected by step 2.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The `T extends Record<string, any> | null` operand type
//!   (`packages/utils/src/fastObjectShallowCompare.ts:4`) becomes `&BTreeMap<K, V>`: std has no
//!   shared map trait, so the record representation is fixed to `BTreeMap` for deterministic
//!   iteration order — the boolean result never depends on upstream's `for...in` order, since
//!   every key of `a` is looked up on `b` and the final check compares counts. The
//!   `instanceof Object` guard (`packages/utils/src/fastObjectShallowCompare.ts:8-10`) is
//!   unrepresentable — operands are records by construction — and so is the null branch of the
//!   identity fast path (`null === null` returning `true`); Rust's types make a record-vs-null
//!   operand impossible to express, let alone conflate.
//! - The `Object.is` alias (`packages/utils/src/fastObjectShallowCompare.ts:2`) is the crate's
//!   existing [`ObjectIs`] trait, already ported for `areArraysEqual`: the `f32`/`f64` impls use
//!   the SameValue algorithm (`NaN` equals `NaN`, while `0.0` and `-0.0` differ) and the other
//!   impls coincide with `==`.
//! - Upstream enumerates with `for...in`, which includes inherited enumerable string-keyed
//!   properties and skips symbol-keyed and non-enumerable own properties
//!   (`packages/utils/src/fastObjectShallowCompare.ts:16-30`); Rust records have no prototype
//!   chain, so [`fast_object_shallow_compare`] compares own keys only.
//! - Upstream's two checks per key — value comparison then presence
//!   (`packages/utils/src/fastObjectShallowCompare.ts:19`,
//!   `packages/utils/src/fastObjectShallowCompare.ts:22-24`) — collapse into one map lookup: a
//!   key missing from `b` yields `None` and fails immediately, the same result upstream reaches
//!   through `is(a[key], undefined)` and/or the failed `key in b` check.
//! - Values are compared by identity, not structurally
//!   (`packages/utils/src/fastObjectShallowCompare.ts:19`): fresh structurally-equal nested
//!   records compare unequal. There is deliberately no [`ObjectIs`] impl for `BTreeMap` — it
//!   would silently turn this shallow comparison into a deep one; a nested-record value type
//!   implements [`ObjectIs`] over `std::ptr::eq` to replicate JS reference semantics (the same
//!   adaptation `are_arrays_equal` documents for object item types).

use std::collections::BTreeMap;

use crate::are_arrays_equal::ObjectIs;

/// Shallow-compares two records, porting upstream's `fastObjectShallowCompare`
/// (`packages/utils/src/fastObjectShallowCompare.ts:4-32`).
///
/// Two records compare equal when they are the same reference (the identity fast path,
/// `packages/utils/src/fastObjectShallowCompare.ts:5-7`), or when every key of `a` is present on
/// `b` with an [`ObjectIs`]-equal value (`packages/utils/src/fastObjectShallowCompare.ts:16-25`)
/// and both records have the same key count
/// (`packages/utils/src/fastObjectShallowCompare.ts:28-31`) — so extra keys on `b` are rejected
/// too. The identity fast path returns `true` without consulting [`ObjectIs`], and a `false`
/// result short-circuits at the first failing key: per-key comparisons run before the final
/// key-count check, exactly as upstream's control flow does.
pub fn fast_object_shallow_compare<K: Ord, V: ObjectIs>(
    a: &BTreeMap<K, V>,
    b: &BTreeMap<K, V>,
) -> bool {
    if std::ptr::eq(a, b) {
        return true;
    }

    for (key, a_value) in a {
        match b.get(key) {
            Some(b_value) if a_value.object_is(b_value) => {}
            _ => return false,
        }
    }

    a.len() == b.len()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use super::*;
    use crate::are_arrays_equal::ObjectIs;

    fn record<const N: usize>(entries: [(&str, f64); N]) -> BTreeMap<String, f64> {
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    // Mirrors the identity fast path (`packages/utils/src/fastObjectShallowCompare.ts:5-7`):
    // the same record reference compares equal without visiting any key, so a counting value
    // type's comparisons never run.
    #[test]
    fn returns_true_for_the_same_record_reference_without_comparing_any_values() {
        let comparisons = Rc::new(Cell::new(0));
        let mut a: BTreeMap<String, Counting> = BTreeMap::new();
        a.insert(
            "key".to_string(),
            Counting {
                value: 1.0,
                comparisons: Rc::clone(&comparisons),
            },
        );

        assert!(fast_object_shallow_compare(&a, &a));
        assert_eq!(
            comparisons.get(),
            0,
            "identity fast path skipped comparisons"
        );
    }

    // Records with the same key sets and `Object.is`-equal values compare equal
    // (`packages/utils/src/fastObjectShallowCompare.ts:16-31`).
    #[test]
    fn returns_true_for_records_with_the_same_keys_and_values() {
        let a = record([("a", 1.0), ("b", 2.0)]);
        let b = record([("a", 1.0), ("b", 2.0)]);

        assert!(fast_object_shallow_compare(&a, &b));
    }

    // Two records with zero keys on both sides compare equal via the final key-count check
    // (`packages/utils/src/fastObjectShallowCompare.ts:28-31`).
    #[test]
    fn returns_true_for_two_empty_records() {
        let a: BTreeMap<String, f64> = BTreeMap::new();
        let b: BTreeMap<String, f64> = BTreeMap::new();

        assert!(fast_object_shallow_compare(&a, &b));
    }

    // A differing value fails the `Object.is` check
    // (`packages/utils/src/fastObjectShallowCompare.ts:19`).
    #[test]
    fn returns_false_when_a_value_differs() {
        let a = record([("a", 1.0), ("b", 2.0)]);
        let b = record([("a", 1.0), ("b", 3.0)]);

        assert!(!fast_object_shallow_compare(&a, &b));
    }

    // A key of `a` missing from `b` fails the presence check
    // (`packages/utils/src/fastObjectShallowCompare.ts:22-24`).
    #[test]
    fn returns_false_when_a_key_of_a_is_missing_from_b() {
        let a = record([("a", 1.0), ("b", 2.0)]);
        let b = record([("a", 1.0)]);

        assert!(!fast_object_shallow_compare(&a, &b));
    }

    // An extra key present only on `b` fails the final key-count equality
    // (`packages/utils/src/fastObjectShallowCompare.ts:28-31`).
    #[test]
    fn returns_false_when_b_has_an_extra_key() {
        let a = record([("a", 1.0)]);
        let b = record([("a", 1.0), ("b", 2.0)]);

        assert!(!fast_object_shallow_compare(&a, &b));
    }

    // `Object.is` semantics: `NaN` values compare equal
    // (`packages/utils/src/fastObjectShallowCompare.ts:19`).
    #[test]
    fn treats_nan_values_as_equal() {
        let a = record([("x", f64::NAN)]);
        let b = record([("x", f64::NAN)]);

        assert!(fast_object_shallow_compare(&a, &b));
    }

    // `Object.is` semantics: `0` and `-0` compare unequal
    // (`packages/utils/src/fastObjectShallowCompare.ts:19`).
    #[test]
    fn treats_zero_and_negative_zero_as_unequal() {
        let a = record([("x", 0.0)]);
        let b = record([("x", -0.0)]);

        assert!(!fast_object_shallow_compare(&a, &b));
    }

    // Nested records are compared by reference, not structurally
    // (`packages/utils/src/fastObjectShallowCompare.ts:19`): fresh structurally-equal nested
    // records compare unequal, while both sides referencing the same nested record compare
    // equal.
    #[test]
    fn compares_nested_records_by_reference_not_structurally() {
        let inner1 = record([("deep", 1.0)]);
        let inner2 = record([("deep", 1.0)]);
        let shared = record([("deep", 1.0)]);

        let mut a: BTreeMap<String, Nested> = BTreeMap::new();
        a.insert("child".to_string(), Nested(&inner1));
        let mut b: BTreeMap<String, Nested> = BTreeMap::new();
        b.insert("child".to_string(), Nested(&inner2));
        assert!(!fast_object_shallow_compare(&a, &b));

        let mut c: BTreeMap<String, Nested> = BTreeMap::new();
        c.insert("child".to_string(), Nested(&shared));
        let mut d: BTreeMap<String, Nested> = BTreeMap::new();
        d.insert("child".to_string(), Nested(&shared));
        assert!(fast_object_shallow_compare(&c, &d));
    }

    // A `false` result short-circuits at the first failing key: values after it are never
    // compared (`packages/utils/src/fastObjectShallowCompare.ts:16-25`).
    #[test]
    fn stops_at_the_first_failing_key() {
        let comparisons = Rc::new(Cell::new(0));
        let counting = |value: f64| Counting {
            value,
            comparisons: Rc::clone(&comparisons),
        };

        let mut a: BTreeMap<String, Counting> = BTreeMap::new();
        a.insert("a".to_string(), counting(1.0));
        a.insert("b".to_string(), counting(2.0));
        a.insert("c".to_string(), counting(3.0));

        let mut b: BTreeMap<String, Counting> = BTreeMap::new();
        b.insert("a".to_string(), counting(1.0));
        b.insert("b".to_string(), counting(9.0));
        b.insert("c".to_string(), counting(3.0));

        assert!(!fast_object_shallow_compare(&a, &b));
        assert_eq!(
            comparisons.get(),
            2,
            "key `a` compared once, key `b` compared once and failed, key `c` never compared"
        );
    }

    // Per-key comparisons run before the final key-count check
    // (`packages/utils/src/fastObjectShallowCompare.ts:16-31`), so a record with an extra key is
    // rejected only after the shared keys' comparisons ran.
    #[test]
    fn runs_per_key_comparisons_before_the_key_count_check() {
        let comparisons = Rc::new(Cell::new(0));
        let counting = |value: f64| Counting {
            value,
            comparisons: Rc::clone(&comparisons),
        };

        let mut a: BTreeMap<String, Counting> = BTreeMap::new();
        a.insert("a".to_string(), counting(1.0));

        let mut b: BTreeMap<String, Counting> = BTreeMap::new();
        b.insert("a".to_string(), counting(1.0));
        b.insert("extra".to_string(), counting(2.0));

        assert!(!fast_object_shallow_compare(&a, &b));
        assert_eq!(
            comparisons.get(),
            1,
            "key `a` compared before the count check"
        );
    }

    // A value type whose `ObjectIs` counts invocations, for pinning the fast path and the
    // short-circuit order.
    struct Counting {
        value: f64,
        comparisons: Rc<Cell<usize>>,
    }

    impl ObjectIs for Counting {
        fn object_is(&self, other: &Self) -> bool {
            self.comparisons.set(self.comparisons.get() + 1);
            self.value.object_is(&other.value)
        }
    }

    // A nested-record value type replicating JS reference semantics with `std::ptr::eq`, the
    // same adaptation `are_arrays_equal` documents for object item types.
    struct Nested<'a>(&'a BTreeMap<String, f64>);

    impl ObjectIs for Nested<'_> {
        fn object_is(&self, other: &Self) -> bool {
            std::ptr::eq(self.0, other.0)
        }
    }
}
