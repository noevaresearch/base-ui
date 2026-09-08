//! Port of `packages/utils/src/usePreviousValue.ts` (Base UI Phase A util).
//!
//! Upstream is a 20-line hook: one piece of `useState` state holding `{ current, previous }`,
//! seeded on the first render with the incoming value and the `null` sentinel
//! (`packages/utils/src/usePreviousValue.ts:10-13`); a render-phase conditional that, whenever
//! the tracked value fails `Object.is` against the stored current, calls `setState` with the
//! new value as current and the old one as `previous`
//! (`packages/utils/src/usePreviousValue.ts:15-17`); and a `return state.previous` read during
//! that same render (`packages/utils/src/usePreviousValue.ts:19`). Because React applies a
//! render-phase `setState` with an immediate re-render before committing, the committed output
//! always exposes the previous value of the *current* tracked value — the change is absorbed
//! within the same commit (`specs/utils/usePreviousValue.md`, "State model").
//!
//! The proven contract comes from `packages/utils/src/usePreviousValue.test.tsx` (235 lines,
//! 11 tests; see `specs/utils/usePreviousValue.md`): the sentinel on first render, the
//! previous value across a change chain, `Object.is` equality semantics (NaN unchanged, ±0
//! distinguished), inert equal-value renders, reference-identity object comparison,
//! optional-value tracking, and batched changes observing only the committed final value.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Upstream reads the tracked value once per render; Leptos components run once and props
//!   are reactive sources, so `value` becomes `V: Get<Value = T>` (the `use_controlled` port's
//!   convention, `specs/architecture.md`, the state-management quick reference). The seed
//!   snapshot is an untracked read at hook-call time
//!   (`packages/utils/src/usePreviousValue.ts:11`).
//! - Upstream's render-phase `setState` + immediate re-render
//!   (`packages/utils/src/usePreviousValue.ts:15-17`) becomes a transition performed at read
//!   time inside the returned derived signal — the read plays the role of the re-running
//!   render body. Consequences, all matching the upstream observable contract:
//!   - reading after a source change is synchronous (no executor involvement needed),
//!     matching the committed-output timing the suite asserts
//!     (`packages/utils/src/usePreviousValue.test.tsx:47-51`);
//!   - equal-value reads are inert — the transition only fires when `Object.is` fails, so
//!     repeated reads and same-value writes never advance the previous value
//!     (`packages/utils/src/usePreviousValue.test.tsx:88-91`, `:140-144`);
//!   - values changed without an intervening read are never observed: the state only
//!     remembers the last-read current, so a read after k changes transitions directly from
//!     the last-read value — the analog of React's update batching, where only the final
//!     committed value is seen (`packages/utils/src/usePreviousValue.test.tsx:207-215`);
//!   - if no consumer ever reads the returned signal, no transition runs. Upstream's
//!     transition runs on every render, but its return value is the hook's only observable
//!     surface (`specs/utils/usePreviousValue.md`, "DOM structure", "Events"), so the
//!     difference is unobservable.
//! - `Object.is` (`packages/utils/src/usePreviousValue.ts:15`) is the crate's existing
//!   [`ObjectIs`] port from `are_arrays_equal`: NaN equals NaN, `0.0` and `-0.0` differ, and
//!   the other covered types compare with `==`, coinciding with `Object.is`
//!   (`packages/utils/src/areArraysEqual.ts:7`). User-defined types implement [`ObjectIs`]
//!   explicitly — `Rc::ptr_eq`/`std::ptr::eq` replicates the JS reference-identity comparison
//!   the object test pins (`packages/utils/src/usePreviousValue.test.tsx:147-169`).
//! - Upstream's `T | null` return and `previous: null` seed (`packages/utils/src/usePreviousValue.ts:12`,
//!   `:19`) collapse into Rust's `Option<T>`: the outer layer is the "no previous" sentinel
//!   and a genuine previous value is always `Some`. Upstream itself cannot distinguish its
//!   sentinel from a tracked `null` (both are `null`; `packages/utils/src/usePreviousValue.test.tsx:182`
//!   and `:188` assert identical values), so the port is strictly more precise. JS's
//!   `undefined`/`null` duality — `Object.is(undefined, null)` is `false`, so `undefined →
//!   null` is a detected change returning `undefined`
//!   (`packages/utils/src/usePreviousValue.test.tsx:184-185`) — has no Rust counterpart: both
//!   are `Option::None`, and a `None → None` pair is `Object.is`-equal, hence no transition.
//! - Upstream's `useState` box (`packages/utils/src/usePreviousValue.ts:10-13`) becomes an
//!   `Rc<RefCell<PreviousState<T>>>` captured by the returned derived signal — per-hook-call
//!   internal state that dies with the signal (disposed with the calling owner), the crate's
//!   established internal-state pattern (`use_interval.rs`, `use_enhanced_click_handler.rs`).
//!   The state is deliberately non-reactive: reactivity flows only from the `value` source
//!   through the derived signal, matching upstream, where the state only ever advances in
//!   lockstep with a value change during a render
//!   (`packages/utils/src/usePreviousValue.ts:15-17`).
//! - Upstream's generic-inference test (`packages/utils/src/usePreviousValue.test.tsx:218-234`)
//!   is compile-time enforced by the Rust signature — every test below instantiates a concrete
//!   `T` — so it has no dedicated runtime test.
//! - `'use client'` (`packages/utils/src/usePreviousValue.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.
//!
//! Must be called inside a reactive owner (a component): the returned signal is
//! arena-allocated to the calling owner and disposed with it, like the other hook ports.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::Signal;

use crate::are_arrays_equal::ObjectIs;

/// The upstream `useState` state shape (`packages/utils/src/usePreviousValue.ts:10-13`): the
/// last-seen tracked value and the value seen before it — `None` until the first change.
struct PreviousState<T> {
    current: T,
    previous: Option<T>,
}

/// The upstream `usePreviousValue` hook (`packages/utils/src/usePreviousValue.ts:9-20`):
/// tracks `value` and exposes the value seen before the current one — the "no previous"
/// sentinel until the first `Object.is` change, then the last-seen value from before each
/// change. Reads are synchronous and transitions are idempotent (equal-value reads never
/// advance the previous value); values changed without an intervening read are never
/// observed, the analog of React's batching. See the module docs for the read-time-transition
/// adaptation.
pub fn use_previous_value<T, V>(value: V) -> Signal<Option<T>, LocalStorage>
where
    T: ObjectIs + Clone + 'static,
    V: Get<Value = T> + GetUntracked<Value = T> + 'static,
{
    // Upstream's seed (`packages/utils/src/usePreviousValue.ts:10-13`): `current` is the
    // first render's value, `previous` is the null sentinel.
    let state = Rc::new(RefCell::new(PreviousState {
        current: value.get_untracked(),
        previous: None,
    }));

    // Upstream's per-render body (`packages/utils/src/usePreviousValue.ts:15-19`), performed
    // at read time — the read is the re-running render body (see the module docs).
    Signal::derive_local(move || {
        let current = value.get();
        let mut state = state.borrow_mut();
        if !current.object_is(&state.current) {
            let last_seen = state.current.clone();
            state.previous = Some(last_seen);
            state.current = current;
        }
        state.previous.clone()
    })
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use any_spawner::Executor;
    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;

    use super::*;
    use crate::are_arrays_equal::ObjectIs;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:20-32`: the first read exposes the
    // sentinel, not the initial value.
    #[test]
    fn returns_none_on_the_first_read() {
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("first");
        let previous = use_previous_value(source);

        assert_eq!(previous.get_untracked(), None);

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:34-52`: each change exposes the
    // value from before it, across a chain — and the read lands synchronously after the
    // change (no executor involvement), the committed-output timing upstream asserts.
    #[test]
    fn returns_the_previous_value_as_the_tracked_value_changes() {
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("first");
        let previous = use_previous_value(source);

        assert_eq!(previous.get_untracked(), None);

        source.set("second");
        assert_eq!(previous.get_untracked(), Some("first"));

        source.set("third");
        assert_eq!(previous.get_untracked(), Some("second"));

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:54-75` (adapted — upstream's
    // `any`-typed prop lets one instance chain numbers into booleans; a Rust instantiation
    // fixes `T`, so the chain stays numeric): non-string primitives work the same.
    #[test]
    fn works_with_non_string_primitives() {
        let owner = in_owner();

        let source: RwSignal<i64> = RwSignal::new(42);
        let previous = use_previous_value(source);

        assert_eq!(previous.get_untracked(), None);

        source.set(100);
        assert_eq!(previous.get_untracked(), Some(42));

        source.set(7);
        assert_eq!(previous.get_untracked(), Some(100));

        source.set(3);
        assert_eq!(previous.get_untracked(), Some(7));

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:77-92`: NaN compared against NaN
    // is Object.is-equal, so no transition happens and the sentinel survives — the case
    // Rust's `==` (NaN != NaN) would get wrong without the `ObjectIs` port.
    #[test]
    fn treats_nan_as_unchanged() {
        let owner = in_owner();

        let source: RwSignal<f64> = RwSignal::new(f64::NAN);
        let previous = use_previous_value(source);

        assert_eq!(previous.get_untracked(), None);

        source.set(f64::NAN);
        assert_eq!(
            previous.get_untracked(),
            None,
            "Object.is(NaN, NaN) — no transition"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:94-107`: a change TO NaN from a
    // different value is detected.
    #[test]
    fn returns_the_previous_value_when_changing_to_nan() {
        let owner = in_owner();

        let source: RwSignal<f64> = RwSignal::new(1.0);
        let previous = use_previous_value(source);

        source.set(f64::NAN);
        assert_eq!(previous.get_untracked(), Some(1.0));

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:109-125`: +0 and -0 are
    // distinguished in both directions — the other half of the SameValue semantics Rust's
    // `==` (0.0 == -0.0) would suppress. Rust's `==` cannot tell the zeros apart, so the
    // assertions check the sign bit.
    #[test]
    fn distinguishes_positive_and_negative_zero() {
        let owner = in_owner();

        let source: RwSignal<f64> = RwSignal::new(0.0);
        let previous = use_previous_value(source);

        source.set(-0.0);
        let returned = previous.get_untracked();
        assert!(
            matches!(returned, Some(value) if value == 0.0 && value.is_sign_positive()),
            "0.0 → -0.0 must be a change returning +0.0, got {returned:?}"
        );

        source.set(0.0);
        let returned = previous.get_untracked();
        assert!(
            matches!(returned, Some(value) if value == 0.0 && value.is_sign_negative()),
            "-0.0 → 0.0 must be a change returning -0.0, got {returned:?}"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:127-145`: rerenders without a
    // value change are inert — re-reads and same-value writes never advance the previous
    // value.
    #[test]
    fn ignores_reads_where_the_value_does_not_change() {
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("stable");
        let previous = use_previous_value(source);

        assert_eq!(previous.get_untracked(), None);

        // The unrelated-prop rerenders of the upstream test are, from the hook's point of
        // view, reads with an unchanged value:
        assert_eq!(previous.get_untracked(), None);
        assert_eq!(previous.get_untracked(), None);

        // An explicit same-value write is equally inert:
        source.set("stable");
        assert_eq!(previous.get_untracked(), None);

        owner.cleanup();
    }

    /// A stand-in for the upstream test's JS object literals
    /// (`packages/utils/src/usePreviousValue.test.tsx:147-169`): contents can be equal across
    /// instances while the `Rc` handle preserves allocation identity, which [`ObjectIs`]
    /// compares — the Rust replication of JS reference identity (`Rc::ptr_eq` as
    /// `std::ptr::eq`, `specs/utils/usePreviousValue.md`, "State model").
    #[derive(Clone, Debug)]
    struct Tracked {
        _value: u8,
        handle: Rc<()>,
    }

    impl ObjectIs for Tracked {
        fn object_is(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.handle, &other.handle)
        }
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:147-169`: three distinct object
    // instances with equal contents produce three change transitions, and each returned
    // previous is the *same instance* as the one from the prior render — the `toBe`
    // assertions become `Rc::ptr_eq` checks.
    #[test]
    fn compares_custom_types_by_reference_identity() {
        let owner = in_owner();

        let obj1 = Tracked {
            _value: 1,
            handle: Rc::new(()),
        };
        let obj2 = Tracked {
            _value: 1,
            handle: Rc::new(()),
        };
        let obj3 = Tracked {
            _value: 1,
            handle: Rc::new(()),
        };

        let source: RwSignal<Tracked, LocalStorage> = RwSignal::new_local(obj1.clone());
        let previous = use_previous_value(source);

        assert!(
            previous.get_untracked().is_none(),
            "the first read exposes the sentinel"
        );

        source.set(obj2.clone());
        let returned = previous.get_untracked();
        assert!(
            matches!(&returned, Some(value) if Rc::ptr_eq(&value.handle, &obj1.handle)),
            "the previous value must be the same instance as obj1, got {returned:?}"
        );

        source.set(obj3.clone());
        let returned = previous.get_untracked();
        assert!(
            matches!(&returned, Some(value) if Rc::ptr_eq(&value.handle, &obj2.handle)),
            "the previous value must be the same instance as obj2, got {returned:?}"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:171-191` minus the
    // `undefined → null` step (adapted — JS's two empty values are one `Option::None` in
    // Rust, and a `None → None` pair is Object.is-equal, so that transition has no Rust
    // counterpart; see the module docs): optional values are tracked as first-class values,
    // and the sentinel (`None`) stays distinct from a tracked `Some(None)` previous.
    #[test]
    fn tracks_optional_values_with_the_sentinel_distinct_from_a_tracked_none() {
        let owner = in_owner();

        let source: RwSignal<Option<&'static str>> = RwSignal::new(None);
        let previous = use_previous_value(source);

        assert_eq!(
            previous.get_untracked(),
            None,
            "an initial None exposes the sentinel (:174-182)"
        );

        source.set(Some("defined"));
        assert_eq!(
            previous.get_untracked(),
            Some(None),
            "None → Some exposes the initial None as the previous value (:187-188)"
        );

        source.set(None);
        assert_eq!(
            previous.get_untracked(),
            Some(Some("defined")),
            "Some → None exposes the Some as the previous value (:190-191)"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/usePreviousValue.test.tsx:194-216`: three value changes
    // issued before the consumer observes them produce exactly one observation, reporting the
    // value from before the batch — the hook never surfaces values no read ever saw, the
    // analog of React committing only the final batched value.
    #[test]
    fn never_observes_intermediate_values_between_reads() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let previous = use_previous_value(source);

        let observed: Rc<RefCell<Vec<Option<&'static str>>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&observed);
        Effect::new(move || {
            sink.borrow_mut().push(previous.get());
        });
        Executor::poll_local();
        assert_eq!(&*observed.borrow(), &[None]);

        source.set("first");
        source.set("second");
        source.set("third");
        Executor::poll_local();
        assert_eq!(
            &*observed.borrow(),
            &[None, Some("initial")],
            "the batch of three changes is observed once, as a change from 'initial'"
        );

        owner.cleanup();
    }

    // Pins the discrete-commit reactivity (spec "Events" — "the only observable 'signal' is
    // the updated return value on the next committed render",
    // `packages/utils/src/usePreviousValue.test.tsx:47-51`): the returned value is a real
    // reactive source, so an observing consumer re-runs and sees the advanced previous on
    // each separate change.
    #[test]
    fn notifies_an_observing_consumer_on_each_value_change() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("a");
        let previous = use_previous_value(source);

        let observed: Rc<RefCell<Vec<Option<&'static str>>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&observed);
        Effect::new(move || {
            sink.borrow_mut().push(previous.get());
        });
        Executor::poll_local();
        assert_eq!(&*observed.borrow(), &[None]);

        source.set("b");
        Executor::poll_local();
        assert_eq!(&*observed.borrow(), &[None, Some("a")]);

        source.set("c");
        Executor::poll_local();
        assert_eq!(&*observed.borrow(), &[None, Some("a"), Some("b")]);

        owner.cleanup();
    }
}
