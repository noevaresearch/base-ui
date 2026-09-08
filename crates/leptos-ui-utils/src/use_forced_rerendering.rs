//! Port of `packages/utils/src/useForcedRerendering.ts` (Base UI Phase A util).
//!
//! Upstream is a 13-line hook: one piece of anonymous state initialized to an empty object
//! whose value is destructured away — only the setter survives
//! (`packages/utils/src/useForcedRerendering.ts:8`) — and a `useCallback`-memoized zero-arg
//! function invoking `setState({})` with a freshly allocated object
//! (`packages/utils/src/useForcedRerendering.ts:10-12`). Because each call changes the state
//! identity, React schedules a rerender of the calling component; the state itself is never
//! read, reset, or exposed — it exists purely as a rerender counter-by-identity
//! (`specs/utils/useForcedRerendering.md`, "State model").
//!
//! This unit has no upstream test file (`ralph/generated/utils.json:287-292` lists
//! `testFiles: []`), so every behavioral claim is source-derived and UNVERIFIED by a test, per
//! `specs/utils/useForcedRerendering.md`. The proven consumer contract comes from the three
//! call sites: `TabsIndicator` registers the returned function as the tabs list context's
//! indicator-update listener (`packages/react/src/tabs/indicator/TabsIndicator.tsx:52-56`),
//! `StoreInspector` invokes it from a one-millisecond timeout inside a store subscription
//! (`packages/utils/src/store/StoreInspector.tsx:250-258`), and `NumberFieldRoot` calls it
//! from non-reactive handlers that need measurements recomputed
//! (`packages/react/src/number-field/root/NumberFieldRoot.tsx:123`, `:269`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The anonymous `{}` state becomes reactive_graph's [`Trigger`] — "a data-less signal with
//!   the sole purpose of notifying other reactive code of a change". Upstream's state value is
//!   never read (`packages/utils/src/useForcedRerendering.ts:8` discards it), and Rust has no
//!   object identity for a discarded `{}` to change, so the value-bearing-state mechanics are
//!   replaced by the framework primitive for exactly this shape. The hook has no read surface,
//!   matching upstream's write-only state (`specs/utils/useForcedRerendering.md`, "State
//!   model": "no way to read or reset the internal state").
//! - `setState({})` with a fresh identity becomes [`Trigger::notify()`]
//!   (`packages/utils/src/useForcedRerendering.ts:11`): an unconditional mark-dirty with no
//!   equality dedup, so every call is a real notification — successive calls are never no-ops
//!   (`specs/utils/useForcedRerendering.md`, "Edge cases"), and how many subscriber runs
//!   actually occur is the executor's coalescing decision, the analog of React's batching.
//! - Upstream's returned function re-renders the calling component, which implicitly re-runs
//!   the render body reading the measured/derived values. Leptos component bodies run once, so
//!   there is no implicit re-render to receive the notification; the port exposes
//!   [`ForcedRerendering::track`] as the explicit subscription point. A consumer places it
//!   inside the effect or derived computation that plays the role of the re-rendering render
//!   body (for `TabsIndicator`, the measurement effect; for `NumberFieldRoot`, the
//!   positioning/metrics effect), and every [`ForcedRerendering::rerender`] call re-runs it.
//!   This is the one surface upstream does not need because React re-renders implicitly.
//! - The `useCallback` empty-dependency memoization (`packages/utils/src/useForcedRerendering.ts:10-12`)
//!   is N/A: Leptos components run once, so the handle is created exactly once per hook call
//!   and is [`Copy`], giving the same stable identity upstream's memoization guarantees — same
//!   as the `use_controlled` port's documented adaptation (`specs/architecture.md`, the
//!   `useCallback` row).
//! - Unmount: upstream registers no effect and no cleanup, and a post-unmount call merely
//!   schedules an update React discards (`specs/utils/useForcedRerendering.md`, "Edge
//!   cases"). [`Trigger`] is arena-allocated to the calling owner; after the owner is
//!   disposed, `notify()` finds no live inner and is a silent no-op — 1:1 with upstream.
//!   The hook itself registers no `on_cleanup`, matching upstream's cleanup-free hook.
//! - `'use client'` (`packages/utils/src/useForcedRerendering.ts:1`) is N/A — there is no
//!   React Server Components boundary in Rust.
//! - Calling the function during render is not guarded against
//!   (`specs/utils/useForcedRerendering.md`, "Edge cases"); the port adds no guard either —
//!   `notify()` during a tracked computation behaves like React's setState-during-render:
//!   unguarded, at the caller's own risk.

use reactive_graph::signal::Trigger;
use reactive_graph::traits::{Notify, Track};

/// The upstream return value: a handle that forces a rerender
/// (`packages/utils/src/useForcedRerendering.ts:10-12`). [`Copy`], so it can be registered as
/// a listener and invoked from detached closures (timeouts, subscriptions) exactly like
/// upstream's memoized function.
#[derive(Clone, Copy, Debug)]
pub struct ForcedRerendering {
    trigger: Trigger,
}

impl ForcedRerendering {
    /// The upstream returned function (`packages/utils/src/useForcedRerendering.ts:11`):
    /// schedules a rerender of the consumer's reactive computations — every call is a real
    /// notification (no value, so no equality dedup), and coalescing is the executor's
    /// decision, as React's batching is upstream's. Safe to call after the hook's owner is
    /// disposed (a silent no-op, like a post-unmount update React discards) and from outside
    /// any reactive context (the `StoreInspector` timeout pattern).
    pub fn rerender(&self) {
        self.trigger.notify();
    }

    /// The subscription point playing the role of upstream's implicit re-render: tracks the
    /// notification inside the current reactive computation (effect, memo, derived signal),
    /// which then re-runs on every [`ForcedRerendering::rerender`] call. Upstream has no
    /// counterpart — React's re-render is implicit — but Leptos components run once, so the
    /// consumer must opt in explicitly (see the module docs).
    pub fn track(&self) {
        self.trigger.track();
    }
}

/// The upstream `useForcedRerendering` hook
/// (`packages/utils/src/useForcedRerendering.ts:7-13`): returns a handle whose `rerender()`
/// forces the consumer's tracked reactive computations to re-run. Must be called inside a
/// reactive owner (a component): the underlying trigger is arena-allocated to the calling
/// owner and disposed with it. UNVERIFIED upstream — no test asserts the hook; see the module
/// docs.
pub fn use_forced_rerendering() -> ForcedRerendering {
    ForcedRerendering {
        trigger: Trigger::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    /// Creates the hook plus a counting effect that tracks the handle — the consumer shape
    /// playing the role of the re-rendering render body. Returns the effect's run count; the
    /// initial run happens on the next executor poll.
    fn counting_consumer(rerender: ForcedRerendering) -> Rc<Cell<usize>> {
        let runs = Rc::new(Cell::new(0));
        let counter = Rc::clone(&runs);
        Effect::new(move || {
            rerender.track();
            counter.set(counter.get() + 1);
        });
        runs
    }

    // Pins the core transition (`packages/utils/src/useForcedRerendering.ts:10-12`, spec
    // "State model" — each call schedules a rerender; UNVERIFIED upstream): every `rerender()`
    // call re-runs the consumer's tracked computation.
    #[test]
    fn every_rerender_call_re_runs_the_tracked_computation() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let rerender = use_forced_rerendering();
        let runs = counting_consumer(rerender);

        Executor::poll_local();
        assert_eq!(runs.get(), 1, "the initial run is the render-body analog");

        rerender.rerender();
        Executor::poll_local();
        assert_eq!(runs.get(), 2);

        rerender.rerender();
        Executor::poll_local();
        assert_eq!(runs.get(), 3);

        owner.cleanup();
    }

    // Pins the fresh-identity semantics (`packages/utils/src/useForcedRerendering.ts:11`,
    // spec "Edge cases" — successive calls are not no-ops by value comparison; UNVERIFIED
    // upstream): two calls in the same tick both land, coalescing into a single re-run — the
    // executor's decision, the analog of React's update batching.
    #[test]
    fn rapid_calls_coalesce_into_a_single_re_run() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let rerender = use_forced_rerendering();
        let runs = counting_consumer(rerender);

        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        rerender.rerender();
        rerender.rerender();
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            2,
            "both calls notify, but the flush runs the subscriber once"
        );

        owner.cleanup();
    }

    // Pins the detached-invocation consumer shape (`packages/react/src/tabs/indicator/TabsIndicator.tsx:54-56`
    // registers the returned function as a listener and `packages/utils/src/store/StoreInspector.tsx:250-258`
    // invokes it from a timeout closure, both after the component body has returned;
    // UNVERIFIED upstream): the handle is a `Copy` value that notifies from a detached,
    // untracked closure — no observer is running when it fires.
    #[test]
    fn the_handle_invokes_correctly_from_a_detached_closure() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let rerender = use_forced_rerendering();
        let runs = counting_consumer(rerender);
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        // The listener-registration pattern: the handle is copied out and stored, then
        // invoked from outside any tracked computation.
        let listener = move || rerender.rerender();
        listener();
        Executor::poll_local();
        assert_eq!(runs.get(), 2);

        owner.cleanup();
    }

    // Pins the unmount edge case (`packages/utils/src/useForcedRerendering.ts:7-13`, spec
    // "Edge cases" — a post-unmount call merely schedules an update React discards;
    // UNVERIFIED upstream): after the owner is disposed, `rerender()` is a silent no-op that
    // neither panics nor re-runs the (also disposed) subscriber.
    #[test]
    fn rerender_after_owner_disposal_is_a_silent_no_op() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let rerender = use_forced_rerendering();
        let runs = counting_consumer(rerender);
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        owner.cleanup();
        rerender.rerender();
        Executor::poll_local();
        assert_eq!(runs.get(), 1, "the disposed trigger notifies nothing");
    }

    // Pins the per-instance state model (`packages/utils/src/useForcedRerendering.ts:8`, spec
    // "State model" — fully internal, per-hook-call state; UNVERIFIED upstream): two hook
    // instances in the same owner notify only their own consumers.
    #[test]
    fn independent_instances_do_not_interfere() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let first = use_forced_rerendering();
        let second = use_forced_rerendering();
        let first_runs = counting_consumer(first);
        let second_runs = counting_consumer(second);

        Executor::poll_local();
        assert_eq!(first_runs.get(), 1);
        assert_eq!(second_runs.get(), 1);

        first.rerender();
        Executor::poll_local();
        assert_eq!(first_runs.get(), 2);
        assert_eq!(second_runs.get(), 1, "the other instance stays untouched");

        owner.cleanup();
    }
}
