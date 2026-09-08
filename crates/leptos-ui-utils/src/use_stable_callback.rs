//! Port of `packages/utils/src/useStableCallback.ts` (Base UI Phase A util).
//!
//! Upstream is a 62-line module: a `'use client'` directive, two sibling-util imports, a
//! module-level effect-primitive selection, a module-local `Callback` alias
//! (`packages/utils/src/useStableCallback.ts:14`), the `Stable` record type
//! (`packages/utils/src/useStableCallback.ts:16-23`), the hook itself
//! (`packages/utils/src/useStableCallback.ts:35-40`), and two module-local helpers:
//!
//! ```ts
//! export function useStableCallback<T extends Callback>(callback: T | undefined): T {
//!   const stable = useRefWithInit(createStableCallback).current;
//!   stable.next = callback;
//!   useSafeInsertionEffect(stable.effect);
//!   return stable.trampoline;
//! }
//! ```
//!
//! The hook stabilizes the function passed so the returned `trampoline` is "always the same
//! between renders": it becomes non-reactive to the values it captures, so it can sit in
//! `React.useMemo`/`React.useEffect` dependency lists without re-triggering them, and — unlike
//! React 19.2's `React.useEffectEvent` — can travel through contexts and event-handler props
//! (`packages/utils/src/useStableCallback.ts:25-34`). Mechanically: every render writes the
//! latest function into the record's `next` slot (`packages/utils/src/useStableCallback.ts:37`),
//! a scheduled effect promotes `callback = next`
//! (`packages/utils/src/useStableCallback.ts:38`, `packages/utils/src/useStableCallback.ts:47-49`),
//! and the `trampoline` created once inside `createStableCallback`
//! (`packages/utils/src/useStableCallback.ts:42-52`) dispatches every call — forwarding all
//! arguments, returning the callback's result — to whatever the `callback` slot currently holds,
//! silently doing nothing and returning `undefined` when that slot is `undefined`
//! (`packages/utils/src/useStableCallback.ts:46`).
//!
//! Source of truth: **none.** This unit has no dedicated test file upstream — `testFiles` is
//! empty in the generated manifest (`ralph/generated/utils.json:383-390`) and no
//! `useStableCallback.test.*` file exists anywhere in the repo (per
//! `specs/utils/useStableCallback.md`, "Source of truth"). Every upstream claim is therefore
//! source-derived and UNVERIFIED by definition; the tests below pin the ported contract itself.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - **Effect-primitive selection resolves to the identity fallback.** Upstream freezes
//!   `useSafeInsertionEffect` once at module evaluation: real `useInsertionEffect` when the
//!   environment has one, otherwise the identity `(fn) => fn()` — "React 17 doesn't have
//!   useInsertionEffect" / "Preact replaces useInsertionEffect with useLayoutEffect and fires
//!   too late" (`packages/utils/src/useStableCallback.ts:5-12`). Leptos has no
//!   insertion-effect primitive (its effect tiers are `Effect`/`RenderEffect`, mapped to
//!   `useEffect`/`useLayoutEffect` in `specs/architecture.md`, "React ↔ Leptos state-management
//!   quick reference" — nothing runs *during commit before layout effects*), so the port pins
//!   the fallback branch, exactly as React 17 and Preact environments resolve. The spec records
//!   that branch's consequences, and the port reproduces each one: the promotion
//!   `callback = next` executes synchronously during the hook call
//!   (`packages/utils/src/useStableCallback.ts:12`), the trampoline is returned with the current
//!   render's callback already active, and the initial-render render-phase throw never occurs in
//!   such environments (`specs/utils/useStableCallback.md`, "Edge cases" — "React 17 / Preact
//!   fallback environments"). The `assertNotCalled` sentinel
//!   (`packages/utils/src/useStableCallback.ts:43-45`, `packages/utils/src/useStableCallback.ts:54-62`)
//!   is therefore unreachable in the port — the promotion overwrites the initial slot before the
//!   trampoline can ever escape the hook call — and its dev-only
//!   `Base UI: Cannot call an event handler while rendering.` throw has no port counterpart; the
//!   same holds upstream in fallback environments. (The `safeReact` port deferred this decision
//!   to this unit: `crates/leptos-ui-utils/src/safe_react.rs:28-33`.) A deferred-promotion
//!   design (a reactive `Effect` promotion) was considered and rejected: it would make a
//!   trampoline call from a mount-time `use_iso_layout_effect` — a legitimate upstream pattern,
//!   since layout effects run after insertion effects — hit the sentinel and panic in debug
//!   builds, a failure mode the pinned fallback branch upstream never produces.
//! - **Record shape.** Upstream's `Stable` record has four slots (`next`, `callback`,
//!   `trampoline`, `effect`, `packages/utils/src/useStableCallback.ts:16-23`). The port keeps
//!   the two data slots verbatim — `next` ("The next value for callback") and `callback` ("The
//!   function to be called by trampoline") — because they are the load-bearing state. The
//!   `trampoline` and `effect` slots become, respectively, the returned [`StableCallback`] handle
//!   and the inline promotion step, because in Rust both would be closure fields capturing the
//!   record itself — self-referential `Rc` cycles that leak (upstream's JS cluster is collectable
//!   through the GC; an `Rc` cycle is not). The dispatch and promotion *logic* is byte-for-byte
//!   the upstream semantics, only not stored as self-closures.
//! - **The `useRefWithInit` composition is preserved**, not dissolved: the record is allocated
//!   through [`crate::use_ref_with_init::use_ref_with_init`]
//!   (`packages/utils/src/useStableCallback.ts:36`), the crate's ported sibling util — its
//!   initialization is synchronous and once-only per call site, and its `StoredValue` box is
//!   arena-allocated to the calling owner
//!   (`crates/leptos-ui-utils/src/use_ref_with_init.rs:41-54`). Must be called inside a reactive
//!   owner (a component) for the record to be disposed with the calling scope — the port of
//!   React's invalid-hook-call error in a system that cannot throw
//!   (`crates/leptos-ui-utils/src/use_ref_with_init.rs:84-90`).
//! - **Nullable parameter, optional result.** Upstream's `callback: T | undefined`
//!   (`packages/utils/src/useStableCallback.ts:35`) becomes `Option<F>`; upstream's dispatch
//!   `stable.callback?.(...args)` — which returns `undefined` when the promoted callback is
//!   `undefined` (the hook was last called with `undefined`), a silent no-op
//!   (`packages/utils/src/useStableCallback.ts:46`; `specs/utils/useStableCallback.md`, "Events")
//!   — becomes an [`Option`]`<R>` return from [`StableCallback::call`]: `Some(result)` when a
//!   callback is promoted, [`None`] when not. Upstream callers that pass a concrete function
//!   (the overwhelming pattern, e.g. `packages/react/src/tabs/tab/TabsTab.tsx:76`) get the
//!   dispatch either way; callers that coalesce `undefined` themselves
//!   (`packages/utils/src/store/ReactStore.ts:193`, `fn ?? NOOP`) may instead pass [`None`] and
//!   get the no-op.
//! - **Argument forwarding.** Upstream forwards all call arguments positionally via rest/spread
//!   (`packages/utils/src/useStableCallback.ts:46`). Rust has no varargs: [`StableCallback::call`]
//!   takes one generic argument `A`, with multi-argument upstream callbacks mapping to a tuple
//!   (`f(a, b)` → `call((a, b))`) and zero-argument ones to `()`. No argument wrapping, mutation,
//!   or prevention semantics are applied — upstream applies none
//!   (`specs/utils/useStableCallback.md`, "Events").
//! - **Identity stability.** Upstream's `trampoline` is created once and never reassigned, so
//!   the reference returned is identical for the hook instance's lifetime regardless of how the
//!   wrapped function's identity changes
//!   (`packages/utils/src/useStableCallback.ts:46-52`; `specs/utils/useStableCallback.md`,
//!   "State model"). Rust closures have no observable reference identity; the port's analog is
//!   the [`StableCallback`] handle: a cheap [`Clone`] over one shared record, where every clone
//!   dispatches through the same `callback` slot — consumers that pass the handle around by
//!   value (the reason upstream wants a stable reference: effects, refs, contexts) share one
//!   dispatch target, and nothing re-allocates it.
//! - **Per-render bookkeeping is once-only.** `stable.next = callback` runs unconditionally on
//!   every upstream render, overwriting the previous value including with `undefined`
//!   (`packages/utils/src/useStableCallback.ts:37`); a Leptos body runs once, so the write (and
//!   the promotion) happen exactly once per hook call. The rapid-re-render and stale-callback
//!   edge cases (`specs/utils/useStableCallback.md`, "Edge cases") have no port counterpart —
//!   there is no render N+1 to be stale relative to. Re-calling the hook is not possible
//!   (no re-render), matching upstream's per-call-site instance model.
//! - **Unmount.** No cleanup is registered — upstream's promotion effect returns nothing and
//!   registers no destructor, and after unmount the trampoline keeps dispatching to the last
//!   promoted callback with no mounted-guard
//!   (`packages/utils/src/useStableCallback.ts:38`, `packages/utils/src/useStableCallback.ts:47-49`;
//!   `specs/utils/useStableCallback.md`, "Edge cases" — "Unmount"). The port reproduces this by
//!   giving the [`StableCallback`] handle its own `Rc` to the record: owner disposal drops the
//!   `StoredValue`'s reference (the ref-with-the-instance analog) while the handle keeps the
//!   record alive and dispatching — upstream's GC survival of the escaped closure. A trampoline
//!   call after disposal therefore works, rather than tripping the recorded reactive_graph
//!   access-after-disposal boundary.
//! - **Reentrancy.** Upstream's dispatch reads `stable.callback` and invokes it, so a callback
//!   that calls its own trampoline again recurses naturally in JS
//!   (`packages/utils/src/useStableCallback.ts:46`). The port clones the callback out of the
//!   record before invoking, so the same reentrancy cannot double-borrow the `RefCell`.
//! - `'use client'` (`packages/utils/src/useStableCallback.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::owner::StoredValue;
use reactive_graph::traits::WithValue;

use crate::use_ref_with_init::use_ref_with_init;

/// The upstream `Callback` alias (`packages/utils/src/useStableCallback.ts:14`): the erased
/// function type the record stores. Upstream erases through `(...args: any[]) => any`; the port
/// erases through a trait object over the one generic argument.
type BoxedCallback<A, R> = Rc<dyn Fn(A) -> R>;

/// The upstream `Stable` record (`packages/utils/src/useStableCallback.ts:16-23`), minus the
/// `trampoline`/`effect` slots that become the returned handle and the inline promotion step
/// (see the module docs — self-referential closure fields would form uncollectable `Rc` cycles).
struct StableRecord<A: 'static, R: 'static> {
    /// "The next value for callback" (`packages/utils/src/useStableCallback.ts:17-18`).
    next: Option<BoxedCallback<A, R>>,
    /// "The function to be called by trampoline. This must fail during the initial render
    /// phase." (`packages/utils/src/useStableCallback.ts:19-20`) — the initial sentinel is the
    /// fallback branch's unreachable state in the port (see the module docs), so the slot starts
    /// empty and is filled by the synchronous promotion below.
    callback: Option<BoxedCallback<A, R>>,
}

impl<A: 'static, R: 'static> StableRecord<A, R> {
    /// The upstream `createStableCallback` factory
    /// (`packages/utils/src/useStableCallback.ts:42-52`): the record starts with `next`
    /// `undefined` (`packages/utils/src/useStableCallback.ts:44`) and the callback slot unset.
    fn new() -> Self {
        Self {
            next: None,
            callback: None,
        }
    }

    /// The promotion step — upstream's `effect` record field
    /// (`packages/utils/src/useStableCallback.ts:22`, `packages/utils/src/useStableCallback.ts:47-49`):
    /// `stable.callback = stable.next`. Upstream schedules this through `useSafeInsertionEffect`
    /// (`packages/utils/src/useStableCallback.ts:38`); in the pinned fallback branch the
    /// scheduler is the identity `(fn) => fn()`
    /// (`packages/utils/src/useStableCallback.ts:12`), so the step runs synchronously during the
    /// hook call. Assignment, not a take: upstream keeps `next` populated
    /// (`packages/utils/src/useStableCallback.ts:48`).
    fn promote(&mut self) {
        self.callback = self.next.clone();
    }
}

/// The upstream `trampoline` (`packages/utils/src/useStableCallback.ts:21`,
/// `packages/utils/src/useStableCallback.ts:46`) as a nameable handle: a cheap-[`Clone`] view
/// over the one shared record, so consumers can store it in effects, refs, and contexts and
/// every copy dispatches through the same promoted callback — the Rust analog of upstream's
/// "always the same between renders" reference identity
/// (`packages/utils/src/useStableCallback.ts:25-26`). The handle holds its own `Rc` to the
/// record, so it keeps dispatching after the calling owner is disposed (upstream's trampoline
/// outlives unmount via GC; see the module docs).
#[derive(Clone)]
pub struct StableCallback<A: 'static, R: 'static> {
    record: Rc<RefCell<StableRecord<A, R>>>,
}

impl<A: 'static, R: 'static> StableCallback<A, R> {
    /// The upstream trampoline dispatch (`packages/utils/src/useStableCallback.ts:46`):
    /// `(...args) => stable.callback?.(...args)` — forwards `args` to the currently promoted
    /// callback and returns its result wrapped in [`Some`], or returns [`None`] without calling
    /// anything when the promoted callback is absent (upstream's silent no-op returning
    /// `undefined`; `specs/utils/useStableCallback.md`, "Events"). The callback is cloned out of
    /// the record before invocation so a callback that re-enters its own trampoline cannot
    /// double-borrow the record.
    pub fn call(&self, args: A) -> Option<R> {
        let callback = self.record.borrow().callback.clone();
        callback.map(|callback| callback(args))
    }
}

/// The upstream `useStableCallback` hook (`packages/utils/src/useStableCallback.ts:35-40`):
/// stabilizes `callback` so the returned handle always dispatches to it, with a reference the
/// caller may copy freely — non-reactive to anything the closure captures, usable from effects
/// and event handlers, and a silent no-op when constructed from [`None`]. The per-render
/// bookkeeping (`packages/utils/src/useStableCallback.ts:37-38`) runs once, during this call —
/// the Leptos body-runs-once dissolution of upstream's render/commit split, with the promotion
/// synchronous per the pinned fallback branch (see the module docs). UNVERIFIED upstream — the
/// hook has no dedicated test (`ralph/generated/utils.json:383-390`); the contract is pinned by
/// this module's tests per `specs/utils/useStableCallback.md`, "Source of truth". Must be called
/// inside a reactive owner (a component); see the module docs.
pub fn use_stable_callback<A, R, F>(callback: Option<F>) -> StableCallback<A, R>
where
    A: 'static,
    R: 'static,
    F: Fn(A) -> R + 'static,
{
    // `useRefWithInit(createStableCallback).current`
    // (`packages/utils/src/useStableCallback.ts:36`) — the record is created synchronously,
    // exactly once per call site, and anchored to the calling owner.
    let stable: StoredValue<Rc<RefCell<StableRecord<A, R>>>, reactive_graph::owner::LocalStorage> =
        use_ref_with_init(|| Rc::new(RefCell::new(StableRecord::new())));

    // `stable.next = callback` (`packages/utils/src/useStableCallback.ts:37`) — unconditional,
    // overwriting whatever was there, including with `None` (upstream's `undefined`).
    let boxed: Option<BoxedCallback<A, R>> =
        callback.map(|callback| Rc::new(callback) as BoxedCallback<A, R>);
    stable.with_value(|record| record.borrow_mut().next = boxed);

    // `useSafeInsertionEffect(stable.effect)` (`packages/utils/src/useStableCallback.ts:38`) —
    // the identity-fallback invocation `stable.effect()` in the pinned branch
    // (`packages/utils/src/useStableCallback.ts:12`): the promotion runs synchronously, during
    // this call, before the trampoline escapes.
    stable.with_value(|record| record.borrow_mut().promote());

    // `return stable.trampoline` (`packages/utils/src/useStableCallback.ts:39`) — the handle
    // shares the record; the `StoredValue` keeps its own reference for the owner's lifetime.
    StableCallback {
        record: stable.with_value(Rc::clone),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the dispatch shape (`packages/utils/src/useStableCallback.ts:46` — all arguments
    // forwarded, the callback's return value returned) and the fallback-branch immediacy
    // (`packages/utils/src/useStableCallback.ts:12` — the promotion runs synchronously during
    // the hook call, so the trampoline is active on return; `specs/utils/useStableCallback.md`,
    // "Edge cases" — "React 17 / Preact fallback environments"): the call works in the same
    // body, with no executor poll, and forwards both the argument and the result.
    #[test]
    fn the_trampoline_dispatches_to_the_hook_time_callback_immediately() {
        let owner = in_owner();

        let handler = use_stable_callback(Some(|value: i32| value * 2));

        assert_eq!(
            handler.call(21),
            Some(42),
            "the trampoline dispatches to the callback passed at hook time, \
             forwarding the argument and returning the result"
        );

        owner.cleanup();
    }

    // Pins the nullable contract (`packages/utils/src/useStableCallback.ts:35` —
    // `T | undefined`; `packages/utils/src/useStableCallback.ts:46` — the `?.` dispatch is a
    // silent no-op returning `undefined` when the promoted callback is absent;
    // `specs/utils/useStableCallback.md`, "Events"): a `None` hook call yields a trampoline
    // whose calls are no-ops returning `None`, not panics.
    #[test]
    fn a_none_callback_makes_the_trampoline_a_silent_no_op() {
        let owner = in_owner();

        let nothing: Option<fn(i32) -> i32> = None;
        let handler = use_stable_callback(nothing);

        assert_eq!(
            handler.call(1),
            None,
            "the absent callback dispatch is a silent no-op, not an error"
        );

        owner.cleanup();
    }

    // Pins identity stability (`packages/utils/src/useStableCallback.ts:46-52` — the
    // trampoline is created once and never reassigned, so every reference the caller holds
    // dispatches through the same slot; `specs/utils/useStableCallback.md`, "State model"):
    // clones of the handle are views over one shared record — calls through each copy all land
    // on the same underlying callback.
    #[test]
    fn every_clone_of_the_handle_dispatches_through_the_same_record() {
        let owner = in_owner();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        let handler = use_stable_callback(Some(move |value: i32| {
            counter.set(counter.get() + 1);
            value + 1
        }));

        let first = handler.clone();
        let second = handler.clone();
        assert_eq!(first.call(1), Some(2));
        assert_eq!(handler.call(10), Some(11));
        assert_eq!(second.call(100), Some(101));
        assert_eq!(
            calls.get(),
            3,
            "all three handle copies dispatched through the one shared callback slot"
        );

        owner.cleanup();
    }

    // Pins per-call-site independence (`packages/utils/src/useStableCallback.ts:36` — each call
    // site gets its own record from `useRefWithInit`; `specs/utils/useStableCallback.md`,
    // "Edge cases" — "Multiple instances / nesting": any number of usages neither interfere nor
    // share state): two hook calls in the same body dispatch independently.
    #[test]
    fn two_call_sites_get_independent_records() {
        let owner = in_owner();

        let first_handler = use_stable_callback(Some(|value: i32| value + 1));
        let second_handler = use_stable_callback(Some(|value: i32| value * 10));

        assert_eq!(first_handler.call(1), Some(2));
        assert_eq!(second_handler.call(1), Some(10), "the other record is untouched");

        owner.cleanup();
    }

    // Pins the unmount contract (`packages/utils/src/useStableCallback.ts:38`,
    // `packages/utils/src/useStableCallback.ts:47-49` — no cleanup is registered, and after
    // unmount the trampoline keeps dispatching to the last promoted callback with no
    // mounted-guard; `specs/utils/useStableCallback.md`, "Edge cases" — "Unmount"): the handle
    // keeps working after the owning scope is disposed, because it holds its own reference to
    // the record.
    #[test]
    fn the_trampoline_keeps_dispatching_after_owner_disposal() {
        let owner = in_owner();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        let handler = use_stable_callback(Some(move |value: i32| {
            counter.set(counter.get() + 1);
            value
        }));

        owner.cleanup();

        assert_eq!(
            handler.call(7),
            Some(7),
            "no mounted-guard: the trampoline dispatches to the last promoted callback \
             after the owner is gone"
        );
        assert_eq!(calls.get(), 1);
    }

    // Pins the reentrancy discipline behind the dispatch
    // (`packages/utils/src/useStableCallback.ts:46` — the JS trampoline reads `stable.callback`
    // and invokes it, so a callback re-entering its own trampoline recurses naturally): the
    // callback is cloned out of the record before invocation, so the recursion cannot
    // double-borrow it. The reentrant hop goes through the trampoline itself, like an upstream
    // callback that holds its own stable reference (wired through a slot filled after creation,
    // since the closure cannot capture the handle it is passed to).
    #[test]
    fn a_callback_reentering_its_own_trampoline_recurses() {
        let owner = in_owner();

        let slot: Rc<RefCell<Option<StableCallback<i32, i32>>>> = Rc::new(RefCell::new(None));
        let wired = Rc::clone(&slot);
        let handler = use_stable_callback(Some(move |value: i32| {
            if value <= 0 {
                return 0;
            }
            wired
                .borrow()
                .as_ref()
                .expect("the handle is wired after creation")
                .call(value - 1)
                .expect("the callback is promoted")
        }));
        *slot.borrow_mut() = Some(handler.clone());

        assert_eq!(
            handler.call(3),
            Some(0),
            "reentrant dispatches unwind through the same shared record"
        );

        owner.cleanup();
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the dispatch shape and fallback-branch immediacy in a real realm
    // (`packages/utils/src/useStableCallback.ts:12`, `:46`).
    #[wasm_bindgen_test]
    fn the_trampoline_dispatches_to_the_hook_time_callback_immediately() {
        let owner = Owner::new();
        owner.set();

        let handler = use_stable_callback(Some(|value: i32| value * 2));

        assert_eq!(handler.call(21), Some(42));

        owner.cleanup();
    }

    // Pins the nullable contract in a real realm
    // (`packages/utils/src/useStableCallback.ts:35`, `:46`).
    #[wasm_bindgen_test]
    fn a_none_callback_makes_the_trampoline_a_silent_no_op() {
        let owner = Owner::new();
        owner.set();

        let nothing: Option<fn(i32) -> i32> = None;
        let handler = use_stable_callback(nothing);

        assert_eq!(handler.call(1), None);

        owner.cleanup();
    }

    // Pins identity stability in a real realm
    // (`packages/utils/src/useStableCallback.ts:46-52`).
    #[wasm_bindgen_test]
    fn every_clone_of_the_handle_dispatches_through_the_same_record() {
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        let handler = use_stable_callback(Some(move |value: i32| {
            counter.set(counter.get() + 1);
            value + 1
        }));

        let clone = handler.clone();
        assert_eq!(handler.call(1), Some(2));
        assert_eq!(clone.call(10), Some(11));
        assert_eq!(calls.get(), 2, "both handle copies hit the one shared callback");

        owner.cleanup();
    }

    // Pins the unmount contract in a real realm
    // (`packages/utils/src/useStableCallback.ts:38`, `:47-49`): the trampoline dispatches after
    // owner disposal — its own reference keeps the record alive, the GC-survival analog.
    #[wasm_bindgen_test]
    fn the_trampoline_keeps_dispatching_after_owner_disposal() {
        let owner = Owner::new();
        owner.set();

        let calls = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&calls);
        let handler = use_stable_callback(Some(move |value: i32| {
            counter.set(counter.get() + 1);
            value
        }));

        owner.cleanup();

        assert_eq!(
            handler.call(7),
            Some(7),
            "no mounted-guard: dispatch works after disposal"
        );
        assert_eq!(calls.get(), 1);
    }
}
