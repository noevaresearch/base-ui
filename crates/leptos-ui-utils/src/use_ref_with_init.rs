//! Port of `packages/utils/src/useRefWithInit.ts` (Base UI Phase A util).
//!
//! Upstream is a 23-line hook: a module-private `UNINITIALIZED` sentinel object
//! (`packages/utils/src/useRefWithInit.ts:4`), and a hook that allocates a `React.useRef` box,
//! and — on the first call only, detected by the sentinel — replaces the sentinel with
//! `init(initArg)` (`packages/utils/src/useRefWithInit.ts:16-22`):
//!
//! ```ts
//! export function useRefWithInit(init, initArg) {
//!   const ref = React.useRef(UNINITIALIZED);
//!   if (ref.current === UNINITIALIZED) {
//!     ref.current = init(initArg);
//!   }
//!   return ref;
//! }
//! ```
//!
//! The returned value is the ref box itself, so callers read and write `.current` on it
//! (`packages/utils/src/useRefWithInit.ts:16-22`); the proven usage shape is
//! `useRefWithInit(() => new ReactStore<State>(initial)).current`
//! (`packages/utils/src/store/ReactStore.test.tsx:11-13`). The sentinel exists because a React
//! render body re-runs on every render: without the guard, `init` would be *invoked* on every
//! render even though only the first result is kept. Two behaviors of that first-render
//! contract are test-proven upstream, both through the `ReactStore` suite (the only detected
//! test exercising this hook — `packages/utils/src/store/ReactStore.test.tsx:5`; the hook has
//! no dedicated test file, per `specs/utils/useRefWithInit.md`, "Source of truth"):
//!
//! - Initialization happens synchronously during the first render, before the component body
//!   finishes — the fixture calls the hook and immediately uses the created store in the same
//!   render pass (`packages/utils/src/store/ReactStore.test.tsx:30-37`).
//! - The instance identity is stable across prop-driven re-renders, with `init` not re-run: a
//!   spy installed on the first-render instance still intercepts calls after two re-renders,
//!   and the update lands on the same instance (`packages/utils/src/store/ReactStore.test.tsx:161-184`).
//!
//! Every other upstream claim (one-way latch, sentinel collision impossibility, throwing-init
//! behavior, unmount/remount, StrictMode) is UNVERIFIED upstream — source-derived only, per
//! `specs/utils/useRefWithInit.md`. The tests below pin the ported contract itself.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The `React.useRef` box becomes reactive_graph's [`StoredValue`] — the crate's confirmed
//!   `useRef`-as-mutable-box mapping (`specs/architecture.md`, the state-management quick
//!   reference's `useRef` row). The handle is `Copy` and `'static`, exactly like a JS ref
//!   object that callers hand to closures: the returned box can be read and written from
//!   detached callbacks (the `ReactStore` subscription pattern, the future merged-refs callback
//!   pattern) without going through any reactive machinery — [`StoredValue`] is the
//!   explicitly non-reactive flavor ("signposted always-non-reactive" API: `read_value`,
//!   `with_value`, `set_value` do not subscribe and `set_value` notifies nothing), matching
//!   `React.RefObject`'s non-reactive `.current`.
//! - [`LocalStorage`] is used rather than [`SyncStorage`]: the crate is a single-threaded
//!   browser-CSR library (the `Rc`/`RefCell`/`SendWrapper` handles in the
//!   `use_interval`/`use_idle_callback`/`use_animation_frame` ports), and its consumers store
//!   non-`Send` values (DOM nodes, `Rc`-based handles) — upstream refs are single-realm
//!   objects too.
//! - The sentinel guard dissolves into Leptos owner semantics — the crate's established
//!   dissolution for this hook (`crates/leptos-ui-utils/src/use_interval.rs:56-64`,
//!   `use_animation_frame.rs`, `use_idle_callback.rs`, `fast_hooks.rs`, all citing
//!   `packages/utils/src/useRefWithInit.ts:16-22`): a Leptos component body runs once, so
//!   calling `init` eagerly during the hook call reproduces both proven behaviors — the
//!   initialization is synchronous (it happens inside the call, before the body continues,
//!   `packages/utils/src/store/ReactStore.test.tsx:30-37`) and it cannot re-run (there is no
//!   re-run of the body to re-invoke it), while the returned handle is the stable instance
//!   across every later render-equivalent. The sentinel itself has no port counterpart: it
//!   guards re-execution of a re-running render body, which Leptos does not have.
//! - Consequences of the dissolution, per `specs/utils/useRefWithInit.md`, "Edge cases":
//!   throwing `init` propagates out of the hook call — the same net effect as upstream's
//!   render-time throw (the box is never initialized), but there is no "next render re-invokes
//!   `init`" recovery because a panicking body never re-runs; StrictMode double-invocation is
//!   N/A (Leptos has no StrictMode; upstream's one detected test explicitly renders with
//!   `strict: false`, `packages/utils/src/store/ReactStore.test.tsx:169`); and the "no
//!   memoization dependencies" property (later `initArg` changes ignored) is vacuous — there
//!   is no later call to pass anything to.
//! - The two-argument overload (`useRefWithInit(init, initArg)`,
//!   `packages/utils/src/useRefWithInit.ts:14`, JSDoc example `:7-11`) has no port
//!   counterpart: its motivation is ergonomic (the init function "doesn't need to be an inline
//!   closure") because JS evaluates call arguments eagerly on every render. A Rust closure
//!   captures the argument natively — `use_ref_with_init(|| sort_columns(&columns))` — and the
//!   body runs once, so there is no per-render closure allocation to avoid.
//! - Per-call-site independence is preserved trivially: each call allocates its own box, so
//!   two hook calls in the same owner get independent instances (upstream's per-component
//!   instance state, `packages/utils/src/useRefWithInit.ts:16`; consistent with the detected
//!   tests where separately rendered fixtures hold separate stores,
//!   `packages/utils/src/store/ReactStore.test.tsx:53-66`).
//! - Lifetime: the box is arena-allocated to the calling owner, so owner disposal drops the
//!   value — upstream's ref is garbage-collected with the component instance. The recorded
//!   reactive_graph boundary: access after disposal panics rather than returning a
//!   still-referenced JS object (there is no GC to keep it alive). Must be called inside a
//!   reactive owner (a component): without one the box registers nowhere and is never
//!   disposed — the port of React's invalid-hook-call error in a system that cannot throw
//!   (the `use_iso_layout_effect` precedent).
//! - `'use client'` (`packages/utils/src/useRefWithInit.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.

use reactive_graph::owner::{LocalStorage, StoredValue};

/// The upstream `useRefWithInit` hook (`packages/utils/src/useRefWithInit.ts:15-23`): allocates
/// the stable box, initializing it synchronously with `init()` — invoked exactly once, during
/// this call, before the hook returns — and returns the box itself for the caller to read and
/// write. The one-argument overload; the two-argument `initArg` overload has no port
/// counterpart (see the module docs). Must be called inside a reactive owner (a component) for
/// the box to be disposed with the calling scope. UNVERIFIED beyond the two ReactStore-proven
/// behaviors — the hook has no dedicated upstream test; see the module docs.
pub fn use_ref_with_init<T>(init: impl FnOnce() -> T) -> StoredValue<T, LocalStorage>
where
    T: 'static,
{
    StoredValue::new_local(init())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{GetValue, IsDisposed, SetValue, WithValue};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    /// A counting init standing in for the `() => new ReactStore(...)` factory
    /// (`packages/utils/src/store/ReactStore.test.tsx:12`): returns a fresh boxed value and
    /// increments the invocation counter it shares with the test.
    fn counting_init() -> (Rc<Cell<usize>>, impl FnOnce() -> usize) {
        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        (calls, move || {
            counter.set(counter.get() + 1);
            41
        })
    }

    // Pins the proven synchronous-initialization contract
    // (`packages/utils/src/store/ReactStore.test.tsx:34-38` — the fixture calls the hook and
    // uses the created store in the same render pass): `init` runs during the hook call
    // itself, so the value is readable immediately afterwards, in the same body, without any
    // executor poll.
    #[test]
    fn init_runs_synchronously_and_the_value_is_available_in_the_same_body() {
        let owner = in_owner();

        let (calls, init) = counting_init();
        let reference = use_ref_with_init(init);

        assert_eq!(
            calls.get(),
            1,
            "init runs during the hook call, not lazily afterwards"
        );
        assert_eq!(
            reference.try_get_value(),
            Some(41),
            "the initialized value is readable in the same body"
        );

        owner.cleanup();
    }

    // Pins the proven identity-stability contract
    // (`packages/utils/src/store/ReactStore.test.tsx:161-184` — the first-render instance
    // stays the instance across re-renders, with init not re-run): the handle is the stable
    // box — writes through one copy of the handle are observed through another (the
    // detached-closure/subscription pattern), and no additional init call ever happens.
    #[test]
    fn the_handle_is_stable_and_writes_persist_without_re_initialization() {
        let owner = in_owner();

        let (calls, init) = counting_init();
        let reference = use_ref_with_init(init);

        // The detached-closure pattern (the ReactStore subscription shape): a Copy handle is
        // handed to a closure invoked after the body has returned.
        let captured = reference;
        let observed = move || captured.with_value(|value| *value);

        reference.set_value(7);
        assert_eq!(
            observed(),
            7,
            "writes through one handle copy are observed through another"
        );
        assert_eq!(
            calls.get(),
            1,
            "init is never re-invoked — the box is initialized exactly once"
        );

        owner.cleanup();
    }

    // Pins the non-reactivity of the box (`React.RefObject`'s `.current` is non-reactive;
    // `specs/architecture.md`, the `useRef` row — "a plain mutable cell with no reactivity"):
    // reads through the non-reactive API do not subscribe an effect, and writes do not notify
    // one.
    #[test]
    fn the_box_is_non_reactive_to_effects() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = in_owner();

        let reference = use_ref_with_init(|| 0usize);

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        let reader = reference;
        Effect::new(move || {
            let _read = reader.with_value(|value| *value);
            counter.set(counter.get() + 1);
        });
        any_spawner::Executor::poll_local();
        assert_eq!(runs.get(), 1, "the initial read runs the effect once");

        reference.set_value(1);
        any_spawner::Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "writing the box does not notify the subscribed effect"
        );

        owner.cleanup();
    }

    // Pins per-call-site independence (`packages/utils/src/useRefWithInit.ts:16`, spec "Edge
    // cases" — each component calling the hook gets its own ref/instance, no shared or
    // module-level state): two hook calls in the same owner hold independent values.
    #[test]
    fn two_call_sites_get_independent_boxes() {
        let owner = in_owner();

        let (first_calls, first_init) = counting_init();
        let (second_calls, second_init) = counting_init();
        let first = use_ref_with_init(first_init);
        let second = use_ref_with_init(second_init);

        assert_eq!(first_calls.get(), 1);
        assert_eq!(
            second_calls.get(),
            1,
            "each call site runs its own init once"
        );

        first.set_value(100);
        assert_eq!(
            second.try_get_value(),
            Some(41),
            "the other box is untouched"
        );

        owner.cleanup();
    }

    // Pins the remount edge case (`specs/utils/useRefWithInit.md`, "Edge cases" — UNVERIFIED
    // upstream: a remount creates a fresh ref with a fresh init call): a fresh owner gets a
    // fresh box and a fresh init invocation.
    #[test]
    fn a_fresh_owner_is_a_fresh_instance() {
        let first_owner = in_owner();
        let (first_calls, first_init) = counting_init();
        let _first = use_ref_with_init(first_init);
        first_owner.cleanup();

        let second_owner = in_owner();
        let (second_calls, second_init) = counting_init();
        let second = use_ref_with_init(second_init);

        assert_eq!(
            first_calls.get(),
            1,
            "the disposed instance's init ran exactly once"
        );
        assert_eq!(
            second_calls.get(),
            1,
            "the fresh instance runs init itself, not the previous one's count"
        );
        assert_eq!(second.try_get_value(), Some(41));

        second_owner.cleanup();
        assert!(second.is_disposed());
    }

    // Pins the lifetime boundary (the module docs — the recorded reactive_graph deviation):
    // the box is disposed with the calling owner, and access afterwards panics rather than
    // returning a still-referenced value (there is no GC to keep it alive).
    #[test]
    #[should_panic(expected = "already been disposed")]
    fn reading_the_box_after_owner_disposal_panics() {
        let owner = in_owner();
        let reference = use_ref_with_init(|| 0usize);

        owner.cleanup();
        assert!(reference.is_disposed());
        let _ = reference.get_value();
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{GetValue, IsDisposed, SetValue, WithValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the proven synchronous-initialization contract in a real realm
    // (`packages/utils/src/store/ReactStore.test.tsx:34-38`).
    #[wasm_bindgen_test]
    fn init_runs_synchronously_and_the_value_is_available_in_the_same_body() {
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        let reference = use_ref_with_init(move || {
            counter.set(counter.get() + 1);
            41
        });

        assert_eq!(calls.get(), 1, "init runs during the hook call");
        assert_eq!(reference.try_get_value(), Some(41));

        owner.cleanup();
    }

    // Pins the proven identity-stability contract in a real realm
    // (`packages/utils/src/store/ReactStore.test.tsx:161-184`): writes through one handle copy
    // are observed through another, with init never re-invoked.
    #[wasm_bindgen_test]
    fn the_handle_is_stable_and_writes_persist_without_re_initialization() {
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        let reference = use_ref_with_init(move || {
            counter.set(counter.get() + 1);
            41
        });

        let captured = reference;
        let observed = move || captured.with_value(|value| *value);

        reference.set_value(7);
        assert_eq!(observed(), 7);
        assert_eq!(calls.get(), 1, "init is never re-invoked");

        owner.cleanup();
    }

    // Pins the non-reactivity of the box in a real realm: writes do not notify a subscribed
    // effect.
    #[wasm_bindgen_test]
    fn the_box_is_non_reactive_to_effects() {
        use reactive_graph::effect::Effect;

        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let reference = use_ref_with_init(|| 0usize);

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        let reader = reference;
        Effect::new(move || {
            let _read = reader.with_value(|value| *value);
            counter.set(counter.get() + 1);
        });
        any_spawner::Executor::poll_local();
        assert_eq!(runs.get(), 1);

        reference.set_value(1);
        any_spawner::Executor::poll_local();
        assert_eq!(runs.get(), 1, "writing the box notifies nothing");

        owner.cleanup();
    }

    // Pins per-call-site independence in a real realm (`packages/utils/src/useRefWithInit.ts:16`).
    #[wasm_bindgen_test]
    fn two_call_sites_get_independent_boxes() {
        let owner = Owner::new();
        owner.set();

        let first = use_ref_with_init(|| 1usize);
        let second = use_ref_with_init(|| 2usize);

        first.set_value(100);
        assert_eq!(
            second.try_get_value(),
            Some(2),
            "the other box is untouched"
        );

        owner.cleanup();
    }

    // Pins the remount edge case in a real realm (spec "Edge cases" — UNVERIFIED upstream): a
    // fresh owner is a fresh instance with its own init call.
    #[wasm_bindgen_test]
    fn a_fresh_owner_is_a_fresh_instance() {
        let calls = Rc::new(Cell::new(0usize));

        let first_owner = Owner::new();
        first_owner.set();
        {
            let counter = Rc::clone(&calls);
            let _first = use_ref_with_init(move || {
                counter.set(counter.get() + 1);
                41
            });
        }
        first_owner.cleanup();

        let second_owner = Owner::new();
        second_owner.set();
        let counter = Rc::clone(&calls);
        let second = use_ref_with_init(move || {
            counter.set(counter.get() + 1);
            42
        });

        assert_eq!(
            calls.get(),
            2,
            "the fresh instance runs its own init (1 + 1), not a re-run of the first"
        );
        assert_eq!(second.try_get_value(), Some(42));

        second_owner.cleanup();
        assert!(second.is_disposed());
    }
}
