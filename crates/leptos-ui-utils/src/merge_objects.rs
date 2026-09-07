//! Port of `packages/utils/src/mergeObjects.ts` (Base UI Phase A util).
//!
//! Upstream is a tiny pure combinator (`packages/utils/src/mergeObjects.ts:1-15`) with no
//! dedicated test file: `specs/utils/mergeObjects.md` records that its observable behavior is
//! proven entirely through its two consumer suites —
//! `packages/react/src/merge-props/mergeProps.test.ts` (the `style` key is the only key
//! `mergeProps` routes through it, `packages/react/src/merge-props/mergeProps.ts:165-172`) and
//! `packages/react/src/internals/useRenderElement.test.tsx` (whole prop bags at
//! `packages/react/src/internals/useRenderElement.tsx:86`, `style` again at
//! `packages/react/src/internals/useRenderElement.tsx:115`).
//!
//! The algorithm (`packages/utils/src/mergeObjects.ts:5-14`) has four input branches:
//!
//! 1. Left truthy, right falsy → return the left object by reference, with no copy
//!    (`packages/utils/src/mergeObjects.ts:5-7`).
//! 2. Left falsy, right truthy → return the right object by reference
//!    (`packages/utils/src/mergeObjects.ts:8-10`).
//! 3. Both truthy → a fresh object `{ ...a, ...b }`: a shallow merge where the right object's
//!    values overwrite conflicting left keys (`packages/utils/src/mergeObjects.ts:11-13`).
//! 4. Both falsy → `undefined` (`packages/utils/src/mergeObjects.ts:14`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - "Falsy" — `undefined`, plus `null` by the same truthiness checks, though the spec marks
//!   non-`undefined` falsy inputs UNVERIFIED — becomes [`Option::None`]; a present object is
//!   `Some(&BTreeMap)`. The record representation follows the crate convention established by
//!   `fast_object_shallow_compare`: std has no shared map trait, so the record is fixed to
//!   `BTreeMap` for deterministic key order. A record-vs-`null` operand is unrepresentable for
//!   the same reason the `instanceof Object` guard of `fastObjectShallowCompare` was.
//! - The two by-reference branches become [`Cow::Borrowed`] — exactly upstream's no-copy
//!   `return a` / `return b` (`packages/utils/src/mergeObjects.ts:5-10`; the spec marks
//!   reference identity UNVERIFIED by tests, but the source states it) — and the both-present
//!   branch becomes [`Cow::Owned`], the fresh `{ ...a, ...b }` object. Consumers mirror
//!   upstream's call sites directly: `?? {}` fallbacks become
//!   `merge_objects(...).unwrap_or_default()` (e.g.
//!   `packages/react/src/internals/useRenderElement.tsx:86`), and bare uses keep the
//!   [`Cow`] and call `into_owned()`/deref where upstream would read fields.
//! - The spread's right-wins overwrite (`packages/utils/src/mergeObjects.ts:12`) is the
//!   `extend` pass: keys of `b` overwrite conflicting keys of `a`, while keys present on only
//!   one side survive. The merge is shallow — nested records are shared/cloned per `V`'s
//!   [`Clone`], never recursively merged (the spec marks deep merging UNVERIFIED, inferred from
//!   `packages/utils/src/mergeObjects.ts:12`).
//! - Input non-mutation (spec marks it UNVERIFIED upstream) is structural: the function takes
//!   shared references and builds the both-present result in a fresh map, so no caller's record
//!   can be mutated — and the frozen-`EMPTY_OBJECT` consumer edge case
//!   (`packages/react/src/internals/useRenderElement.test.tsx:652-681`) cannot throw, because
//!   nothing is ever written through a borrowed reference.

use std::borrow::Cow;
use std::collections::BTreeMap;

/// Merges two optional records, porting upstream's `mergeObjects`
/// (`packages/utils/src/mergeObjects.ts:1-15`).
///
/// When exactly one side is present, that side is returned borrowed with no copy
/// (`packages/utils/src/mergeObjects.ts:5-10`). When both are present, a fresh owned record is
/// produced with the right side's values overwriting conflicting left keys
/// (`packages/utils/src/mergeObjects.ts:11-13`). When both are absent, [`None`] is returned —
/// the `undefined` result upstream consumers fall back from with `?? {}`
/// (`packages/react/src/internals/useRenderElement.tsx:86`,
/// `packages/react/src/merge-props/mergeProps.test.ts:182-188`).
pub fn merge_objects<'a, K: Ord + Clone, V: Clone>(
    a: Option<&'a BTreeMap<K, V>>,
    b: Option<&'a BTreeMap<K, V>>,
) -> Option<Cow<'a, BTreeMap<K, V>>> {
    match (a, b) {
        (Some(a), None) => Some(Cow::Borrowed(a)),
        (None, Some(b)) => Some(Cow::Borrowed(b)),
        (Some(a), Some(b)) => {
            let mut merged = (*a).clone();
            merged.extend(b.iter().map(|(key, value)| (key.clone(), value.clone())));
            Some(Cow::Owned(merged))
        }
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use super::*;

    fn record<const N: usize>(entries: [(&str, &str); N]) -> BTreeMap<String, String> {
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    // Mirrors `packages/react/src/merge-props/mergeProps.test.ts:154-167`: two defined objects
    // shallow-merge with right-wins overwrite — the right object's `color: 'red'` beats the
    // left's `color: 'blue'`, while the left-only `backgroundColor: 'blue'` survives
    // (`packages/utils/src/mergeObjects.ts:11-13`).
    #[test]
    fn merges_two_defined_records_with_right_wins_overwrites() {
        let a = record([("color", "blue"), ("backgroundColor", "blue")]);
        let b = record([("color", "red")]);

        let merged = merge_objects(Some(&a), Some(&b)).unwrap();

        assert_eq!(
            merged.as_ref(),
            &record([("color", "red"), ("backgroundColor", "blue")])
        );
        assert!(
            matches!(merged, Cow::Owned(_)),
            "both-present branch builds a fresh record"
        );
    }

    // Mirrors `packages/react/src/merge-props/mergeProps.test.ts:169-180`: a missing left side
    // yields exactly the right object's content
    // (`packages/utils/src/mergeObjects.ts:8-10`).
    #[test]
    fn returns_the_right_record_content_when_the_left_is_missing() {
        let b = record([("color", "red")]);

        let merged = merge_objects(None, Some(&b)).unwrap();

        assert_eq!(merged.as_ref(), &record([("color", "red")]));
    }

    // Mirrors `packages/react/src/internals/useRenderElement.test.tsx:126-134`: a missing right
    // side leaves the left object's content unchanged, with no crash — the internal
    // `padding: 10px` survives when a style function returns `undefined`
    // (`packages/utils/src/mergeObjects.ts:5-7`).
    #[test]
    fn returns_the_left_record_content_unchanged_when_the_right_is_missing() {
        let a = record([("padding", "10px")]);

        let merged = merge_objects(Some(&a), None).unwrap();

        assert_eq!(merged.as_ref(), &record([("padding", "10px")]));
    }

    // Mirrors `packages/react/src/merge-props/mergeProps.test.ts:182-188`: both sides missing →
    // exactly `undefined`, which is what lets upstream callers write the `?? {}` fallback
    // (`packages/utils/src/mergeObjects.ts:14`).
    #[test]
    fn returns_none_when_both_sides_are_missing() {
        let merged: Option<Cow<BTreeMap<String, String>>> = merge_objects(None, None);

        assert!(merged.is_none());
        // The `?? {}` fallback shape from `packages/react/src/internals/useRenderElement.tsx:86`.
        assert!(merged.unwrap_or_default().is_empty());
    }

    // Pins the no-copy adaptation of `packages/utils/src/mergeObjects.ts:5-10`: a single
    // defined side is returned borrowed (zero value clones), while the both-present branch
    // clones into a fresh record. The spec marks reference identity UNVERIFIED upstream — the
    // source states it, so the port preserves it structurally via [`Cow::Borrowed`].
    #[test]
    fn borrows_a_single_defined_side_without_cloning_and_clones_for_both_present() {
        let clones = Rc::new(Cell::new(0));
        let counting = |value: &str| Counting {
            value: value.to_string(),
            clones: Rc::clone(&clones),
        };

        let mut a: BTreeMap<String, Counting> = BTreeMap::new();
        a.insert("color".to_string(), counting("blue"));

        let merged = merge_objects(Some(&a), None).unwrap();
        assert!(
            matches!(merged, Cow::Borrowed(_)),
            "single defined side is returned by reference"
        );
        assert_eq!(clones.get(), 0, "no-copy branch never clones values");

        let mut b: BTreeMap<String, Counting> = BTreeMap::new();
        b.insert("padding".to_string(), counting("10px"));

        let merged = merge_objects(Some(&a), Some(&b)).unwrap();
        assert!(
            matches!(merged, Cow::Owned(_)),
            "both-present branch builds a fresh record"
        );
        assert_eq!(
            clones.get(),
            2,
            "both-present branch clones each side's values once"
        );
    }

    // Mirrors the chained consumer behavior proven end-to-end by
    // `packages/react/src/internals/useRenderElement.test.tsx:462-475` and `:477-490`: styles
    // from three sources (internal, component, render element) all coexist after chaining
    // through the unit — `padding`, `color`, and `fontSize` all survive.
    #[test]
    fn chains_three_sources_with_all_keys_surviving() {
        let internal = record([("padding", "10px")]);
        let component = record([("color", "rgb(255, 0, 0)")]);
        let render = record([("fontSize", "16px")]);

        let merged = merge_objects(Some(&internal), Some(&component)).unwrap();
        let merged = merge_objects(Some(&merged), Some(&render)).unwrap();

        assert_eq!(
            merged.as_ref(),
            &record([
                ("padding", "10px"),
                ("color", "rgb(255, 0, 0)"),
                ("fontSize", "16px")
            ])
        );
    }

    // Mirrors the EMPTY_OBJECT safety case of
    // `packages/react/src/internals/useRenderElement.test.tsx:652-681` (style case at `:676-681`):
    // merging against a shared empty record — the `EMPTY_OBJECT` state input — does not throw,
    // and the provided style lands in the result. In Rust the frozen-ness is structural (shared
    // references cannot be mutated), so the observable halves are: no panic, and the style
    // content wins.
    #[test]
    fn merges_a_shared_empty_record_with_a_style_record() {
        let empty: BTreeMap<String, String> = BTreeMap::new();
        let style = record([("color", "red")]);

        let merged = merge_objects(Some(&empty), Some(&style)).unwrap();

        assert_eq!(merged.as_ref(), &record([("color", "red")]));
    }

    // A value type counting its clones, for pinning the borrow-vs-clone branches.
    struct Counting {
        value: String,
        clones: Rc<Cell<usize>>,
    }

    impl Clone for Counting {
        fn clone(&self) -> Self {
            self.clones.set(self.clones.get() + 1);
            Counting {
                value: self.value.clone(),
                clones: Rc::clone(&self.clones),
            }
        }
    }
}
