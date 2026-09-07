//! Port of `packages/utils/src/mergeCleanups.ts` (Base UI Phase A util).
//!
//! Upstream combines multiple cleanup functions into a single cleanup function
//! (`packages/utils/src/mergeCleanups.ts:6`): the variadic `mergeCleanups(...cleanups)` accepts
//! an arbitrary-length argument list mixing `() => void` cleanups with the falsy placeholders
//! `false | null | undefined` (the upstream `Cleanup` union,
//! `packages/utils/src/mergeCleanups.ts:1`), and returns a cleanup that invokes each real
//! cleanup in argument order, skipping the placeholders
//! (`packages/utils/src/mergeCleanups.ts:7-14`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The `Cleanup` union collapses into [`Option`]: the three falsy placeholders are all
//!   "absent" values, so [`None`] stands in for `undefined`/`false`/`null` and
//!   `Some(Box<dyn FnOnce()>)` ([`CleanupFn`]) stands in for a real cleanup. The statically
//!   typed parameter also makes the spec's "truthy non-function values" question
//!   (`specs/utils/mergeCleanups.md`) moot: a non-function cannot appear in the list.
//! - The variadic argument list becomes [`impl IntoIterator`]`<Item = `[`Option`]`<`[`CleanupFn`]`>>`,
//!   so callers pass a `vec![...]` of mixed cleanups and placeholders (or any iterator). The
//!   list is materialized eagerly, mirroring how upstream arguments are evaluated before the
//!   `mergeCleanups` call itself.
//! - The returned merged cleanup is [`impl FnOnce`]`()` — callable with zero arguments exactly
//!   like the upstream `() => void` return (`specs/utils/mergeCleanups.md` marks argument
//!   forwarding and re-invocation as UNVERIFIED upstream; the Rust type forbids relying on
//!   either: constituents take no arguments, and the merged cleanup is consumed by a call).
//! - A panicking constituent propagates the unwind and skips later cleanups (the spec marks
//!   throwing-cleanups behavior UNVERIFIED upstream); no guard is added.

/// A single erasable cleanup function — the callable half of the upstream `Cleanup` union
/// (`packages/utils/src/mergeCleanups.ts:1`). Constituents take no arguments and run at most
/// once, so [`FnOnce`] is the faithful shape (see the module docs).
pub type CleanupFn = Box<dyn FnOnce()>;

/// Combines multiple cleanup functions into a single cleanup function
/// (`packages/utils/src/mergeCleanups.ts:3-5`).
///
/// Calling the returned cleanup invokes each `Some` constituent in argument order
/// (`packages/utils/src/mergeCleanups.ts:8-13`), skipping the `None` placeholders; a `None`
/// never breaks execution of the surrounding cleanups.
pub fn merge_cleanups(cleanups: impl IntoIterator<Item = Option<CleanupFn>>) -> impl FnOnce() {
    let cleanups: Vec<CleanupFn> = cleanups.into_iter().flatten().collect();
    move || {
        for cleanup in cleanups {
            cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    // Mirrors `packages/utils/src/mergeCleanups.test.ts:5-14`: `mergeCleanups(first, undefined,
    // false, null, second)()` calls each real cleanup exactly once, in argument order, with the
    // interleaved falsy placeholders skipped. Upstream asserts the two halves separately
    // (`toHaveBeenCalledTimes(1)` on `mergeCleanups.test.ts:11-12`, then
    // `invocationCallOrder` comparison on `:13`); a single shared invocation log captures both,
    // because Rust's `FnOnce` boxes cannot be inspected after being consumed.
    #[test]
    fn calls_each_cleanup_in_order_and_skips_empty_values() {
        let invocations = Rc::new(RefCell::new(Vec::new()));

        let first_invocations = Rc::clone(&invocations);
        let first: CleanupFn = Box::new(move || first_invocations.borrow_mut().push("first"));

        let second_invocations = Rc::clone(&invocations);
        let second: CleanupFn = Box::new(move || second_invocations.borrow_mut().push("second"));

        let merged = merge_cleanups(vec![Some(first), None, None, None, Some(second)]);
        merged();

        assert_eq!(*invocations.borrow(), ["first", "second"]);
    }

    // All-placeholder input: the loop (`packages/utils/src/mergeCleanups.ts:8-13`) simply has
    // nothing to invoke. Upstream only ever exercises placeholders interleaved with real
    // cleanups, so this pins the zero-real-cleanups half of the same skip logic.
    #[test]
    fn placeholder_only_input_runs_nothing() {
        let invocations = Rc::new(RefCell::new(Vec::<&'static str>::new()));

        let merged = merge_cleanups(vec![None, None, None]);
        merged();

        assert!(invocations.borrow().is_empty());
    }

    // Downstream composes merged cleanups inside merged cleanups (e.g.
    // `packages/react/src/floating-ui-react/hooks/useDismiss.ts:711-719` nests
    // `mergeCleanups` results as constituents). The Rust port keeps that working through the
    // same boxing the other constituents need.
    #[test]
    fn nested_merged_cleanups_compose() {
        let invocations = Rc::new(RefCell::new(Vec::new()));

        let outer_invocations = Rc::clone(&invocations);
        let outer: CleanupFn = Box::new(move || outer_invocations.borrow_mut().push("outer"));

        let inner_first_invocations = Rc::clone(&invocations);
        let inner_first = move || inner_first_invocations.borrow_mut().push("inner_first");
        let inner_second_invocations = Rc::clone(&invocations);
        let inner_second = move || inner_second_invocations.borrow_mut().push("inner_second");

        let nested: CleanupFn = Box::new(merge_cleanups(vec![
            Some(Box::new(inner_first) as CleanupFn),
            Some(Box::new(inner_second) as CleanupFn),
        ]));

        let merged = merge_cleanups(vec![Some(nested), None, Some(outer)]);
        merged();

        assert_eq!(*invocations.borrow(), ["inner_first", "inner_second", "outer"]);
    }
}
