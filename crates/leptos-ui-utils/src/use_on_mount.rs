//! Port of `packages/utils/src/useOnMount.ts` (Base UI Phase A util).
//!
//! Upstream is a single 13-line file: one hook whose entire body delegates the caller's
//! callback to `React.useEffect` with a stable empty dependency array
//! (`packages/utils/src/useOnMount.ts:8-12`):
//!
//! ```ts
//! export function useOnMount(fn: React.EffectCallback) {
//!   /* eslint-disable react-hooks/exhaustive-deps */
//!   React.useEffect(fn, EMPTY_ARRAY);
//!   /* eslint-enable react-hooks/exhaustive-deps */
//! }
//! ```
//!
//! The dependency list is `EMPTY_ARRAY`, a module-level frozen singleton that is
//! referentially stable for the lifetime of the process
//! (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`), so React sees
//! equal deps on every render and skips the effect: the body runs once after the commit and
//! a returned destructor runs on unmount (`specs/utils/useOnMount.md`, "State model",
//! "Edge cases").
//!
//! Source of truth: **none.** The upstream repo has no test file for this unit
//! (`ralph/generated/utils.json:350-352` lists `testFiles: []`; `specs/utils/useOnMount.md`,
//! "Source of truth"), so every upstream claim in this module is a source-derived description
//! of current implementation behavior, not test-proven behavior. The tests below pin the
//! ported contract itself. The destructor-returning half of the contract is load-bearing for
//! every detected upstream call site — all six are the `useOnMount(x.disposeEffect)` shape
//! (`packages/utils/src/useInterval.ts:39`, `packages/utils/src/useAnimationFrame.ts:153`,
//! `packages/utils/src/useTimeout.ts:49`, `packages/utils/src/useIdleCallback.ts:59`,
//! `packages/react/src/toast/provider/ToastProvider.tsx:32`,
//! `packages/react/src/floating-ui-react/hooks/useHoverInteractionSharedState.ts:128`) — so
//! the port preserves that shape rather than dissolving it into caller-side plumbing.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Mount timing: [`Effect::new`]'s first run executes on the next tick of the ambient
//!   executor (reactive_graph spawns the effect through `spawn_local`; its first iteration is
//!   gated on the spawned task being polled), NOT synchronously during the hook call — the
//!   analog of `React.useEffect`'s deferred post-commit run. This is the noted
//!   `useEffect` → Leptos `Effect` mapping (`specs/architecture.md`, "React ↔ Leptos
//!   state-management quick reference"), and it is the property that distinguishes this hook
//!   from the synchronous render-phase [`crate::use_on_first_render`], whose port runs the
//!   callback during the calling body (`crates/leptos-ui-utils/src/use_on_first_render.rs`,
//!   "Rust adaptations"). The ambient executor must be initialized before the run
//!   (`leptos::mount` does this in production; tests use `Executor::init_futures_executor`).
//! - The stable `EMPTY_ARRAY` dependency list becomes [`untrack`]: a plain effect would
//!   otherwise track the reactive values its body reads and re-run when they change, while
//!   upstream never re-runs regardless of what `fn` reads because its dependency list never
//!   changes (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`).
//!   Untracking the callback removes tracking entirely, so the effect has no sources, never
//!   re-runs, and the callback runs exactly once per instance — the analog of the
//!   empty-deps contract, not of listing the reads (`specs/utils/useOnMount.md`, "Edge
//!   cases" — identity changes to `fn` are deliberately ignored upstream; here the callback
//!   is consumed by the single deferred run and there is no re-render to re-register it).
//! - The `React.EffectCallback` parameter (`packages/utils/src/useOnMount.ts:8` — a function
//!   optionally returning a destructor) becomes [`use_on_mount`]'s
//!   `callback: impl FnOnce() -> R, R: EffectReturn` bound: a callback returning `()` is the
//!   `undefined`-returning shape and registers no cleanup, and a callback returning any
//!   zero-argument callable is the destructor-returning shape every upstream call site uses.
//!   The [`EffectReturn`] blanket impl is coherent because `()` does not implement
//!   `FnOnce()`, so the two return shapes cannot collide.
//! - The destructor contract becomes [`on_cleanup`], registered inside the effect body on the
//!   effect's own reactive owner: upstream forwards `fn`'s return value to React, which runs
//!   it on unmount, and here the registration fires when the effect's owner is cleaned up —
//!   once per instance, since the untracked effect never re-runs. [`on_cleanup`] requires a
//!   `Send + Sync` closure, so the port wraps the caller's (possibly non-`Send`) cleanup in
//!   [`SendWrapper`] at the registration boundary — the crate's `use_interval` precedent
//!   (`crates/leptos-ui-utils/src/use_interval.rs:164-169`) — preserving upstream's
//!   "any callable is a destructor" breadth.
//! - Unmount: the effect is kept alive in a [`StoredValue`] arena-allocated to the calling
//!   owner, the crate's `use_iso_layout_effect` precedent
//!   (`crates/leptos-ui-utils/src/use_iso_layout_effect.rs`): owner disposal drops the
//!   effect's notification channel, which ends the executor task and cancels the effect. If
//!   disposal happens before the first run is ever polled, the callback is dropped unrun and
//!   no cleanup is registered — the analog of React skipping an effect whose commit never
//!   flushed. Must be called inside a reactive owner (a component): without one the
//!   [`StoredValue`] registers nowhere and the effect is never disposed — the port of React's
//!   invalid-hook-call error in a system that cannot throw.
//! - Server rendering: upstream's effect body does not run during server rendering
//!   (`specs/utils/useOnMount.md`, "Edge cases"); the port inherits the equivalent behavior
//!   from Leptos itself — effects do not run server-side in the framework. The host test
//!   target is a test environment with an initialized executor (the jsdom analog, where
//!   React effects do run), not a server-render environment, so no environment probe is
//!   added — upstream `useOnMount` itself has no environment check
//!   (`packages/utils/src/useOnMount.ts:8-12`).
//! - StrictMode/double-invoke (`specs/utils/useOnMount.md`, "Edge cases" — upstream adds no
//!   guard, so `fn` may run twice under React 18+ development StrictMode): N/A — Leptos has
//!   no StrictMode double-invocation to guard against.
//! - Error propagation: upstream invokes `fn` with no try/catch, so an exception surfaces
//!   through React's normal error handling (`specs/utils/useOnMount.md`, "Edge cases"); the
//!   port adds no guard either — a panicking callback unwinds out of the effect run.
//! - Nesting: no shared module-level state; each call site registers its own independent
//!   effect (`packages/utils/src/useOnMount.ts:8-12`).
//! - `'use client'` (`packages/utils/src/useOnMount.ts:1`) is N/A — there is no React Server
//!   Components boundary in Rust.

use std::cell::RefCell;

use reactive_graph::effect::Effect;
use reactive_graph::graph::untrack;
use reactive_graph::owner::StoredValue;
use reactive_graph::owner::on_cleanup;
use send_wrapper::SendWrapper;

use crate::merge_cleanups::CleanupFn;

/// A value a mount-effect callback may return — the upstream `EffectCallback`'s
/// `void | Destructor` return union (`packages/utils/src/useOnMount.ts:8`).
///
/// `()` is the `undefined`-returning shape: no cleanup is registered. Any zero-argument
/// callable is a destructor: it runs once when the owning reactive scope is disposed (the
/// unmount analog). The blanket impl is coherent because `()` does not implement `FnOnce()`.
pub trait EffectReturn {
    /// Converts the callback's return value into the optional unmount cleanup.
    fn into_cleanup(self) -> Option<CleanupFn>;
}

impl EffectReturn for () {
    fn into_cleanup(self) -> Option<CleanupFn> {
        None
    }
}

impl<F: FnOnce() + 'static> EffectReturn for F {
    fn into_cleanup(self) -> Option<CleanupFn> {
        Some(Box::new(self))
    }
}

/// The upstream `useOnMount` hook (`packages/utils/src/useOnMount.ts:8-12`): runs `callback`
/// once, after mount, on the next executor tick — not synchronously during the calling body —
/// and runs the cleanup it returns when the owning reactive scope is disposed. The callback
/// runs exactly once per instance: reactive values it reads are untracked, so nothing ever
/// re-runs it. UNVERIFIED upstream — the hook has no dedicated test; the contract is pinned by
/// this module's tests per `specs/utils/useOnMount.md`, "Source of truth". Must be called
/// inside a reactive owner (a component); see the module docs.
pub fn use_on_mount<C, R>(callback: C)
where
    C: FnOnce() -> R + 'static,
    R: EffectReturn,
{
    // `Effect::new` re-invokes its body as `FnMut`; the callback is `FnOnce` (consumed by the
    // single deferred run), so it is parked in an `Option` and taken by whichever run executes
    // first. The effect is untracked, so there is exactly one run and the `None` branch is
    // unreachable in practice — it exists so the body cannot double-invoke even if it did.
    let callback = RefCell::new(Some(callback));
    let effect = Effect::new(move || {
        let Some(callback) = callback.borrow_mut().take() else {
            return;
        };
        let Some(cleanup) = untrack(callback).into_cleanup() else {
            return;
        };
        // Registered on the effect's own owner (the body runs inside the effect's cleanup
        // scope): fires at owner disposal — the unmount analog. `SendWrapper` adapts the
        // possibly non-`Send` cleanup to `on_cleanup`'s `Send + Sync` bound; the effect runs
        // on the local thread, so the `take` is same-thread (see the module docs).
        let cleanup = SendWrapper::new(cleanup);
        on_cleanup(move || {
            let cleanup = cleanup.take();
            cleanup();
        });
    });
    StoredValue::new(effect);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn counting_callback() -> (Rc<Cell<usize>>, impl FnOnce() + 'static) {
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        (runs, move || counter.set(counter.get() + 1))
    }

    // Pins the deferred mount timing (`packages/utils/src/useOnMount.ts:11` —
    // `React.useEffect`'s post-commit run; `specs/utils/useOnMount.md`, "State model" —
    // "runs once after the commit", UNVERIFIED upstream): the callback has NOT run when the
    // hook call returns, and runs only once the executor is polled. This is the property
    // distinguishing the hook from the synchronous `use_on_first_render`.
    #[test]
    fn the_callback_runs_on_the_next_executor_tick_not_during_the_hook_call() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (runs, callback) = counting_callback();
        use_on_mount(callback);
        assert_eq!(
            runs.get(),
            0,
            "the callback is deferred, not run during the hook call"
        );

        Executor::poll_local();
        assert_eq!(runs.get(), 1, "the deferred run happens on the next tick");

        owner.cleanup();
    }

    // Pins the stable-EMPTY_ARRAY-deps contract
    // (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`; spec "Edge
    // cases" — re-renders never re-run the effect, identity changes ignored, UNVERIFIED
    // upstream): the callback runs exactly once, and reactive values it reads are untracked —
    // changing them afterwards never re-invokes it.
    #[test]
    fn the_callback_runs_exactly_once_and_signal_reads_are_untracked() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let count = RwSignal::new(0);
        let (runs, callback) = counting_callback();
        use_on_mount(move || {
            // A read that a tracked effect would re-run on — upstream's effect never
            // re-runs regardless of what `fn` reads, so the read must not be tracked.
            let _read = count.get();
            callback();
        });
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        count.set(1);
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "nothing re-runs the callback — the analog of EMPTY_ARRAY deps"
        );

        owner.cleanup();
    }

    // Pins the destructor contract
    // (`packages/utils/src/useOnMount.ts:8` — `React.EffectCallback` may return a destructor;
    // `:11` forwards it to React, which runs it on unmount; spec "Edge cases" — unmount
    // cleanup, UNVERIFIED upstream): a callback returning a callable registers it, and the
    // cleanup fires when the owning scope is disposed.
    #[test]
    fn the_returned_cleanup_runs_when_the_owner_is_disposed() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let cleanups = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&cleanups);
        use_on_mount(move || move || counter.set(counter.get() + 1));
        Executor::poll_local();

        owner.cleanup();
        assert_eq!(
            cleanups.get(),
            1,
            "the returned destructor runs at owner disposal — the unmount analog"
        );
    }

    // Pins the `void` half of the `EffectCallback` return union
    // (`packages/utils/src/useOnMount.ts:8` — "a `fn` that returns `undefined` registers no
    // cleanup"; spec "Edge cases", UNVERIFIED upstream): a `()`-returning callback leaves
    // nothing to run at disposal.
    #[test]
    fn a_callback_returning_unit_registers_no_cleanup() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (runs, callback) = counting_callback();
        use_on_mount(callback);
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        owner.cleanup();
        Executor::poll_local();
        assert_eq!(runs.get(), 1, "no cleanup ran — nothing re-invoked anything");
    }

    // Pins per-call-site independence (`packages/utils/src/useOnMount.ts:8-12` — no shared
    // module-level state; spec "Edge cases" — nesting, UNVERIFIED upstream): two hook calls in
    // the same body each run their own callback exactly once.
    #[test]
    fn independent_call_sites_each_run_once() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (first_runs, first_callback) = counting_callback();
        let (second_runs, second_callback) = counting_callback();
        use_on_mount(first_callback);
        use_on_mount(second_callback);
        Executor::poll_local();

        assert_eq!(first_runs.get(), 1);
        assert_eq!(second_runs.get(), 1, "each call site runs its own callback");

        owner.cleanup();
    }

    // Pins the disposal-before-flush ordering (`packages/utils/src/useOnMount.ts:11` —
    // React skips an effect whose commit never flushed, so neither `fn` nor its destructor
    // ever runs; UNVERIFIED upstream): disposing the owner before the executor is polled
    // drops the unrun effect, and the callback is never invoked.
    #[test]
    fn a_callback_disposed_before_its_first_run_never_runs_nor_cleans_up() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let cleanups = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&cleanups);
        use_on_mount(move || move || counter.set(counter.get() + 1));

        owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            cleanups.get(),
            0,
            "an effect that never flushed runs no callback and registers no cleanup"
        );
    }

    // Pins error propagation (`packages/utils/src/useOnMount.ts:11` — `fn` is invoked with no
    // try/catch in the hook, so an exception surfaces through React's normal error handling;
    // spec "Edge cases", UNVERIFIED upstream): the panic unwinds out of the effect run, not
    // swallowed by the hook.
    #[test]
    #[should_panic(expected = "mount-effect callback boom")]
    fn a_panicking_callback_propagates_out_of_the_effect_run() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        use_on_mount(|| -> () { panic!("mount-effect callback boom") });
        Executor::poll_local();

        owner.cleanup();
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    // The behavior under test is real effect timing against an ambient executor, so the tests
    // run in a real browser via the wasm32 test runner (`.cargo/config.toml` wires it to
    // chromedriver), like the crate's other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the deferred mount timing in a real realm
    // (`packages/utils/src/useOnMount.ts:11`).
    #[wasm_bindgen_test]
    fn the_callback_runs_on_the_next_executor_tick_not_during_the_hook_call() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_on_mount(move || counter.set(counter.get() + 1));
        assert_eq!(runs.get(), 0, "the callback is deferred");
        Executor::poll_local();
        assert_eq!(runs.get(), 1, "the deferred run happens on the next tick");

        owner.cleanup();
    }

    // Pins the stable-deps contract in a real realm
    // (`packages/utils/src/useOnMount.ts:11`, `packages/utils/src/empty.ts:6`): signal reads
    // are untracked — the callback runs exactly once and never re-runs.
    #[wasm_bindgen_test]
    fn the_callback_runs_exactly_once_and_signal_reads_are_untracked() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let count = RwSignal::new(0);
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_on_mount(move || {
            let _read = count.get();
            counter.set(counter.get() + 1);
        });
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        count.set(1);
        Executor::poll_local();
        assert_eq!(runs.get(), 1, "nothing re-runs the callback");

        owner.cleanup();
    }

    // Pins the destructor contract in a real realm
    // (`packages/utils/src/useOnMount.ts:8`, `:11`).
    #[wasm_bindgen_test]
    fn the_returned_cleanup_runs_when_the_owner_is_disposed() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let cleanups = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&cleanups);
        use_on_mount(move || move || counter.set(counter.get() + 1));
        Executor::poll_local();

        owner.cleanup();
        assert_eq!(cleanups.get(), 1, "the destructor runs at owner disposal");
    }

    // Pins the `void` half of the return union in a real realm
    // (`packages/utils/src/useOnMount.ts:8`).
    #[wasm_bindgen_test]
    fn a_callback_returning_unit_registers_no_cleanup() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_on_mount(move || counter.set(counter.get() + 1));
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        owner.cleanup();
        Executor::poll_local();
        assert_eq!(runs.get(), 1, "no cleanup ran");
    }
}
