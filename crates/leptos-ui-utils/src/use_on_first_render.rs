//! Port of `packages/utils/src/useOnFirstRender.ts` (Base UI Phase A util).
//!
//! Upstream is a 10-line hook: a `React.useRef` latch initialized to `true`
//! (`packages/utils/src/useOnFirstRender.ts:5`) guarding a single render-phase invocation of
//! the caller-supplied callback, with the latch flipped to `false` *before* the call
//! (`packages/utils/src/useOnFirstRender.ts:6-8`):
//!
//! ```ts
//! export function useOnFirstRender(fn: Function) {
//!   const ref = React.useRef(true);
//!   if (ref.current) {
//!     ref.current = false;
//!     fn();
//!   }
//! }
//! ```
//!
//! The latch exists because a React render body re-runs on every render: without the guard,
//! `fn` would be invoked again on each one. Flipping before the call makes the guard
//! one-way — once `fn` has run it can never be retried, even if `fn` throws or triggers a
//! synchronous re-render (`specs/utils/useOnFirstRender.md`, "State model", "Edge cases").
//!
//! This unit has no upstream test file (`ralph/generated/utils.json:342-347` lists
//! `testFiles: []`), so every behavioral claim is source-derived and UNVERIFIED by a test,
//! per `specs/utils/useOnFirstRender.md`, "Source of truth". The proven consumer contract
//! comes from the two detected call sites, which both use the hook to seed a shared store
//! synchronously during the root's first render, before parts read it: `SelectRoot` writes
//! the prop bags into the store because "`useSyncedValues` writes in a layout effect, after
//! all descendants have rendered" (`packages/react/src/select/root/SelectRoot.tsx:435-441`),
//! and `AriaCombobox` does the same because its parts "read them with `useStore` during
//! render, and a layout effect commits only after all children have rendered"
//! (`packages/react/src/combobox/root/AriaCombobox.tsx:1446-1455`). Both properties the
//! contract needs — the callback runs during the calling render, and runs exactly once per
//! instance — are pinned by the tests below rather than bound to a reference suite.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The once-latch dissolves into Leptos owner semantics — the crate's established
//!   dissolution for once-guards over a `useRef` box (`crates/leptos-ui-utils/src/use_ref_with_init.rs:55-64`,
//!   `use_interval.rs:56-64`, `use_animation_frame.rs`, `use_idle_callback.rs`,
//!   `fast_hooks.rs`): a Leptos component body runs once, so there is no re-running render
//!   body for the latch to guard against, and invoking the callback eagerly during the hook
//!   call reproduces every observable upstream property (`specs/utils/useOnFirstRender.md`):
//!   "State model" — at most one invocation per instance, no way to re-arm (there is no
//!   later execution of the body to re-invoke it); "Edge cases" — render-phase timing
//!   (synchronous, during the calling body, before the first run of any effect created after
//!   the hook call: a Leptos `Effect`'s first run is queued to the executor and a
//!   `RenderEffect`'s first run happens at its creation, both strictly after a hook call
//!   that precedes them in the body), throw/re-entrancy (a panicking callback propagates out
//!   of the hook call and nothing re-runs it — there is no latch-checking next pass to retry
//!   from), changing-callback identity (vacuous — no later call ever receives one),
//!   StrictMode double render (N/A — Leptos has no StrictMode double-invocation to dedupe;
//!   upstream's ref exists precisely to absorb it), unmount/remount (a remount is a fresh
//!   body execution, so the callback runs again on the new instance), nesting (each call
//!   site is its own invocation, no shared state), and rapid repeated invocations (the body
//!   executes once; there are no repeated invocations of the hook call itself). Keeping a
//!   `StoredValue` latch would be observationally dead code: created fresh per hook call,
//!   always `true` at the check, and never read again — in every calling context the
//!   guard-less form behaves identically, so the latch, like `useRefWithInit`'s sentinel,
//!   "has no port counterpart: it guards re-execution of a re-running render body, which
//!   Leptos does not have."
//! - The parameter type: upstream takes the bare `Function` interface — no parameter or
//!   return typing (`packages/utils/src/useOnFirstRender.ts:4`) — and discards the return
//!   value while returning nothing itself (`packages/utils/src/useOnFirstRender.ts:4-9`).
//!   The port takes `impl FnOnce()`: the widest callable matching a single-invocation,
//!   return-discarded contract (`FnOnce` also accepts consuming closures the narrower
//!   `Fn`/`FnMut` bounds would reject, and both detected call sites pass closures over
//!   render-scope values they use exactly once). Upstream's parameter is named `fn`, a Rust
//!   keyword, renamed `callback`.
//! - Calling context: the intended context is a component body — the first-render scope,
//!   where both upstream call sites live. Unlike the other hook ports there is no allocation
//!   to arena-allocate, so no reactive owner is *required* at runtime; upstream's
//!   rules-of-hooks constraint has no Rust counterpart to enforce (the
//!   `use_iso_layout_effect` precedent for unenforceable calling-context rules). Called
//!   anywhere else, the callback still runs exactly once per hook call — the same behavior
//!   upstream's fresh-ref-per-execution would produce in that context.
//! - `'use client'` (`packages/utils/src/useOnFirstRender.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.

/// The upstream `useOnFirstRender` hook (`packages/utils/src/useOnFirstRender.ts:4-9`):
/// invokes `callback` exactly once, synchronously, during the calling body's execution —
/// the first-render scope — and never again for that execution. Intended to be called in a
/// component body (see the module docs). UNVERIFIED upstream — the hook has no dedicated
/// test; the contract is pinned by this module's tests per `specs/utils/useOnFirstRender.md`,
/// "Source of truth".
pub fn use_on_first_render(callback: impl FnOnce()) {
    callback();
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
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

    /// A counting callback standing in for the `() => store.update({...})` closures at the
    /// detected call sites (`packages/react/src/select/root/SelectRoot.tsx:437-441`):
    /// increments the counter it shares with the test.
    fn counting_callback() -> (Rc<Cell<usize>>, impl FnOnce()) {
        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        (calls, move || {
            counter.set(counter.get() + 1);
        })
    }

    // Pins the proven synchronous-invocation contract
    // (`packages/react/src/select/root/SelectRoot.tsx:435-441` — the store must be seeded
    // during the root's own render, before parts render; `specs/utils/useOnFirstRender.md`,
    // "Edge cases" — render-phase timing, UNVERIFIED upstream): the callback runs during the
    // hook call itself, so the effect is observable in the same body, with no executor poll.
    #[test]
    fn the_callback_runs_synchronously_during_the_hook_call() {
        let owner = in_owner();

        let (calls, callback) = counting_callback();
        use_on_first_render(callback);

        assert_eq!(
            calls.get(),
            1,
            "the callback runs during the hook call, not lazily afterwards"
        );

        owner.cleanup();
    }

    // Pins the once-per-instance state model (`packages/utils/src/useOnFirstRender.ts:5-9`,
    // spec "State model" — a permanent no-op after the first run; "Edge cases" — rapid
    // repeated invocations, UNVERIFIED upstream): after the single hook call, no executor
    // activity ever re-invokes the callback — there is no deferred retry, matching upstream's
    // one-way latch.
    #[test]
    fn the_callback_is_never_invoked_again_after_the_first_run() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (calls, callback) = counting_callback();
        use_on_first_render(callback);
        assert_eq!(calls.get(), 1);

        Executor::poll_local();
        Executor::poll_local();
        assert_eq!(
            calls.get(),
            1,
            "nothing re-invokes the callback after the first run"
        );

        owner.cleanup();
    }

    // Pins the render-phase timing edge case (`packages/utils/src/useOnFirstRender.ts:6-8`,
    // spec "Edge cases" — `fn` runs before any effect of the same commit; UNVERIFIED
    // upstream): an effect registered in the same body after the hook call has not run yet
    // when the callback has already completed, and its first run comes only on the executor
    // poll.
    #[test]
    fn the_callback_runs_before_effects_created_in_the_same_body() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (calls, callback) = counting_callback();
        use_on_first_render(callback);

        let effect_runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&effect_runs);
        Effect::new(move || {
            counter.set(counter.get() + 1);
        });

        assert_eq!(
            calls.get(),
            1,
            "the callback has already run during the body"
        );
        assert_eq!(
            effect_runs.get(),
            0,
            "the same body's effect has not run yet"
        );

        Executor::poll_local();
        assert_eq!(effect_runs.get(), 1, "the effect's first run comes later");
        assert_eq!(calls.get(), 1);

        owner.cleanup();
    }

    // Pins per-call-site independence (`packages/utils/src/useOnFirstRender.ts:5`, spec
    // "Edge cases" — nesting: independent latches, no shared state; UNVERIFIED upstream):
    // two hook calls in the same body each run their own callback exactly once.
    #[test]
    fn independent_call_sites_each_run_once() {
        let owner = in_owner();

        let (first_calls, first_callback) = counting_callback();
        let (second_calls, second_callback) = counting_callback();
        use_on_first_render(first_callback);
        use_on_first_render(second_callback);

        assert_eq!(first_calls.get(), 1);
        assert_eq!(second_calls.get(), 1, "each call site runs its own callback");

        owner.cleanup();
    }

    // Pins the unmount/remount edge case (`packages/utils/src/useOnFirstRender.ts:5`, spec
    // "Edge cases" — a remount creates a fresh instance so `fn` runs again; UNVERIFIED
    // upstream): a fresh owner (the remount analog) gets its own invocation.
    #[test]
    fn a_fresh_owner_is_a_fresh_instance() {
        let calls = Rc::new(Cell::new(0usize));

        let first_owner = in_owner();
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });
        first_owner.cleanup();

        let second_owner = in_owner();
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });

        assert_eq!(
            calls.get(),
            2,
            "the fresh instance runs its own callback (1 + 1), not a carried-over latch"
        );

        second_owner.cleanup();
    }

    // Pins the throw/re-entrancy net effect (`packages/utils/src/useOnFirstRender.ts:7-8`,
    // spec "Edge cases" — a throwing `fn` propagates during render and is never retried;
    // UNVERIFIED upstream): the panic propagates out of the hook call itself — there is no
    // effect boundary to swallow it into an error state, and no next pass exists to retry
    // from.
    #[test]
    #[should_panic(expected = "first-render callback boom")]
    fn a_panicking_callback_propagates_out_of_the_hook_call() {
        let _owner = in_owner();
        use_on_first_render(|| panic!("first-render callback boom"));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the proven synchronous-invocation contract in a real realm
    // (`packages/react/src/select/root/SelectRoot.tsx:435-441`).
    #[wasm_bindgen_test]
    fn the_callback_runs_synchronously_during_the_hook_call() {
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });

        assert_eq!(calls.get(), 1, "the callback runs during the hook call");

        owner.cleanup();
    }

    // Pins the once-per-instance state model in a real realm
    // (`packages/utils/src/useOnFirstRender.ts:5-9`): no executor activity re-invokes the
    // callback after the first run.
    #[wasm_bindgen_test]
    fn the_callback_is_never_invoked_again_after_the_first_run() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });
        assert_eq!(calls.get(), 1);

        Executor::poll_local();
        assert_eq!(calls.get(), 1, "nothing re-invokes the callback");

        owner.cleanup();
    }

    // Pins the render-phase timing edge case in a real realm
    // (`packages/utils/src/useOnFirstRender.ts:6-8`): the callback completes before the
    // first run of an effect created after the hook call in the same body.
    #[wasm_bindgen_test]
    fn the_callback_runs_before_effects_created_in_the_same_body() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });

        let effect_runs = Rc::new(Cell::new(0usize));
        let effect_counter = Rc::clone(&effect_runs);
        Effect::new(move || {
            effect_counter.set(effect_counter.get() + 1);
        });

        assert_eq!(calls.get(), 1);
        assert_eq!(effect_runs.get(), 0);

        Executor::poll_local();
        assert_eq!(effect_runs.get(), 1);
        assert_eq!(calls.get(), 1);

        owner.cleanup();
    }

    // Pins per-call-site independence in a real realm
    // (`packages/utils/src/useOnFirstRender.ts:5`).
    #[wasm_bindgen_test]
    fn independent_call_sites_each_run_once() {
        let owner = Owner::new();
        owner.set();

        let first_calls = Rc::new(Cell::new(0usize));
        let first_counter = Rc::clone(&first_calls);
        use_on_first_render(move || {
            first_counter.set(first_counter.get() + 1);
        });
        let second_calls = Rc::new(Cell::new(0usize));
        let second_counter = Rc::clone(&second_calls);
        use_on_first_render(move || {
            second_counter.set(second_counter.get() + 1);
        });

        assert_eq!(first_calls.get(), 1);
        assert_eq!(second_calls.get(), 1);

        owner.cleanup();
    }

    // Pins the unmount/remount edge case in a real realm
    // (`packages/utils/src/useOnFirstRender.ts:5`): a fresh owner gets its own invocation.
    #[wasm_bindgen_test]
    fn a_fresh_owner_is_a_fresh_instance() {
        let calls = Rc::new(Cell::new(0usize));

        let first_owner = Owner::new();
        first_owner.set();
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });
        first_owner.cleanup();

        let second_owner = Owner::new();
        second_owner.set();
        let counter = Rc::clone(&calls);
        use_on_first_render(move || {
            counter.set(counter.get() + 1);
        });

        assert_eq!(calls.get(), 2);

        second_owner.cleanup();
    }
}
