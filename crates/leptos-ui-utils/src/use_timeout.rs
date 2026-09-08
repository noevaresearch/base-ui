//! Port of `packages/utils/src/useTimeout.ts` (Base UI Phase A util).
//!
//! Upstream is a single 52-line module: a `Timeout` class plus the `useTimeout()` hook owning
//! one instance per component (`packages/utils/src/useTimeout.ts:9-41`,
//! `packages/utils/src/useTimeout.ts:46-52`). The class is a one-shot timer handle:
//!
//! - `start(delay, fn)` clears any previously scheduled call, then schedules a wrapper on the
//!   ambient `setTimeout` — the wrapper resets the id slot to `EMPTY` *before* invoking `fn`,
//!   so a fired one-shot leaves the instance idle again and code inside `fn` observes an idle
//!   instance (`packages/utils/src/useTimeout.ts:19-25`).
//! - `isStarted()` reports whether the id slot is live
//!   (`packages/utils/src/useTimeout.ts:27-29`).
//! - `clear` (a pre-bound arrow-function property) is guarded on `currentId !== EMPTY`, so
//!   clearing an idle or already-fired instance is a no-op
//!   (`packages/utils/src/useTimeout.ts:31-36`).
//! - `disposeEffect` returns `clear`, shaped as a React effect cleanup
//!   (`packages/utils/src/useTimeout.ts:38-40`).
//!
//! Source of truth: **none.** The upstream repo has no test file for this unit
//! (`specs/utils/useTimeout.md`, "Source of truth"; `ralph/generated/utils.json:392-397` lists
//! `testFiles: []`), so every upstream claim in this module is a source-derived description of
//! current implementation behavior, not test-proven behavior. The tests below pin the ported
//! contract itself.
//!
//! Ecosystem role: the sibling `Interval` class extends this one
//! (`packages/utils/src/useInterval.ts:8`), and the scroll-lock unit composes it as a library —
//! `useScrollLock` imports `Timeout` (`packages/utils/src/useScrollLock.ts:7`) and the
//! `ScrollLocker` singleton defers both applying and releasing the page lock onto 0ms
//! `Timeout` instances (`packages/utils/src/useScrollLock.ts:231-232`,
//! `packages/utils/src/useScrollLock.ts:237,245`). This module is therefore the shared base
//! the `use_interval` port anticipated ("the `Timeout` port (its own TODO item) can extract a
//! shared base later", `crates/leptos-ui-utils/src/use_interval.rs` adaptation note) — that
//! port is left untouched here; collapsing the duplicated handle logic into it is a separate
//! decision.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The `EMPTY = 0` sentinel (`packages/utils/src/useTimeout.ts:5-7`) becomes [`Option`]:
//!   upstream needed the numeric `0` because the id slot is a JS number; the port stores the
//!   pending id in a clone-shared `Rc<Cell<Option<TimeoutId>>>`, the same shape as the
//!   `use_interval`/`use_idle_callback`/`use_animation_frame` ports, so clones of the handle
//!   share the slot the way JS callers share the instance (upstream's `currentId` field,
//!   `packages/utils/src/useTimeout.ts:14`).
//! - The zero-argument, single-invocation `fn: Function`
//!   (`packages/utils/src/useTimeout.ts:19`) becomes [`Timeout::start`]'s
//!   `impl FnOnce() + 'static`: invoked at most once, with no payload
//!   (`specs/utils/useTimeout.md`, "Events"). `delay` is a `u32` passed straight through to
//!   the host — no clamping or validation here; host-environment clamping (e.g. the 4 ms
//!   minimum for nested timers) applies exactly as upstream (spec, "Edge cases").
//! - The load-bearing one-shot mechanic — the scheduled wrapper resetting `currentId` before
//!   invoking `fn` (`packages/utils/src/useTimeout.ts:21-24`) — lives inside
//!   [`Timeout::start`]'s job wrapper, so it is shared by both dispatch targets: a fired
//!   timeout leaves the instance idle, re-entrant code inside the callback observes an idle
//!   instance, and a `start` from inside the callback re-arms a fresh timer cleanly
//!   (`specs/utils/useTimeout.md`, "Edge cases"). This is the exact point where the sibling
//!   `Interval` deliberately diverges — its wrapper does not reset the id, so the id stays
//!   live across every tick (`packages/utils/src/useInterval.ts:20-22`).
//! - The pre-binding asymmetry (`clear`/`disposeEffect` are bound arrow properties while
//!   `start`/`isStarted` are prototype methods — `specs/utils/useTimeout.md`, "Edge cases") is
//!   N/A in Rust: every method is callable on the clone-shared handle, and
//!   [`Timeout::dispose_effect`] still returns the clear closure for callers building their
//!   own mount-effect plumbing, matching upstream's detachable-shape use in the hook.
//! - Dispatch goes through the ambient `globalThis.setTimeout`/`clearTimeout` resolved per
//!   call (`packages/utils/src/useTimeout.ts:21-24`, `:31-36`), matching the `use_interval`
//!   precedent of reading ambient browser globals at dispatch time — upstream resolves them
//!   as bare globals rather than through an `ownerWindow` lookup, so it performs no realm
//!   handling of its own (`specs/utils/useTimeout.md`, "DOM structure"). JS garbage-collects
//!   the bound wrapper; the port keeps each registration's wrapper closure in a live registry
//!   keyed by the native id. A one-shot registration is *complete* once it has fired — the
//!   native side will never call the wrapper again — so the fired wrapper retires itself at
//!   the end of its own invocation (the `use_interval` port retires wrappers only on clear,
//!   because an interval wrapper is invoked repeatedly). A wrapper may also be cleared from
//!   inside a different wrapper's invocation. Both retire paths defer the actual drop to a
//!   sweep that never drops a wrapper whose id is still executing on the JS stack (an
//!   active-id set marks invocations) — the same wasm-bindgen no-drop-while-invocable rule the
//!   `use_interval` port documents.
//! - Host builds (native `cargo test`) have no browser globals, so tests install a dispatch
//!   override ([`DISPATCH_OVERRIDE`]) — the analog of stubbing the ambient
//!   `setTimeout`/`clearTimeout` globals, which is exactly how upstream consumers test the
//!   deferral this handle provides (e.g. "Flush the scroll locker's deferred lock, scheduled
//!   on a 0ms timeout", `packages/react/src/dialog/root/DialogRoot.test.tsx:1985-1988`).
//! - The `useTimeout` hook (`packages/utils/src/useTimeout.ts:46-52`) dissolves React's
//!   `useRefWithInit`/`useOnMount` plumbing into Leptos owner semantics, exactly like the
//!   crate's other timer-adjacent hooks: a component's setup body runs once, so the local
//!   binding is the stable instance the way upstream's `useRefWithInit` memoizes it
//!   (`packages/utils/src/useRefWithInit.ts:16-22`), and [`on_cleanup`] plays the role of the
//!   mount effect's returned cleanup (`packages/utils/src/useTimeout.ts:49` via
//!   `packages/utils/src/useOnMount.ts:8-12` and `packages/utils/src/useTimeout.ts:38-40`) —
//!   a pending timeout is cleared when the owning reactive scope is disposed, so `fn` never
//!   runs post-unmount (`specs/utils/useTimeout.md`, "Edge cases"). Must be called inside a
//!   reactive owner (a component).
//! - `'use client'` (`packages/utils/src/useTimeout.ts:1`) is N/A — there is no React Server
//!   Components boundary in Rust.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use send_wrapper::SendWrapper;
#[cfg(target_arch = "wasm32")]
use std::collections::{HashMap, HashSet};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

/// The id the ambient `setTimeout` global returns — upstream's `TimeoutId` number
/// (`packages/utils/src/useTimeout.ts:5`), with the `EMPTY = 0` sentinel
/// (`packages/utils/src/useTimeout.ts:7`) modeled as [`Option`] instead.
pub type TimeoutId = u32;

/// The scheduled unit — upstream's `fn: Function` parameter
/// (`packages/utils/src/useTimeout.ts:19`), invoked at most once with no arguments, boxed
/// because it crosses the dispatch boundary as a stored callback.
type TimeoutJob = Box<dyn FnOnce()>;

/// A one-shot timer handle keeping at most one callback pending — upstream's `Timeout` class
/// (`packages/utils/src/useTimeout.ts:9-41`). Cheap to clone; clones share the same pending-id
/// slot.
#[derive(Clone, Default)]
pub struct Timeout {
    /// Upstream `currentId` (`packages/utils/src/useTimeout.ts:14`, initialized to the `EMPTY`
    /// sentinel); `None` is upstream's `EMPTY`.
    current_id: Rc<Cell<Option<TimeoutId>>>,
}

impl Timeout {
    /// Upstream `static create()` (`packages/utils/src/useTimeout.ts:10-12`).
    pub fn create() -> Self {
        Self::default()
    }

    /// The currently pending timeout id, if any — upstream's public `currentId` field
    /// (`packages/utils/src/useTimeout.ts:14`).
    pub fn current_id(&self) -> Option<TimeoutId> {
        self.current_id.get()
    }

    /// Whether a timeout is pending — upstream's `isStarted()`
    /// (`packages/utils/src/useTimeout.ts:27-29`). Unlike the repeating `Interval`, a fired
    /// one-shot resets the slot inside its own wrapper
    /// (`packages/utils/src/useTimeout.ts:21-24`), so this returns `false` again after the
    /// callback has run.
    pub fn is_started(&self) -> bool {
        self.current_id.get().is_some()
    }

    /// Executes `callback` after `delay`, clearing any previously scheduled call
    /// (`packages/utils/src/useTimeout.ts:19-25`). At most one live timeout exists per
    /// instance: re-`start` cancels the pending timeout instead of stacking, so callbacks from
    /// superseded `start` calls never fire (`specs/utils/useTimeout.md`, "Edge cases"). The
    /// first invocation happens after one full `delay` — plain `setTimeout` semantics, no
    /// leading call.
    pub fn start(&self, delay: u32, callback: impl FnOnce() + 'static) {
        self.clear();
        // Upstream schedules a wrapper that resets `currentId` to EMPTY *before* invoking `fn`
        // (`packages/utils/src/useTimeout.ts:21-24`) — the one-shot latch. The wrapper closes
        // over the instance, so the port's job wrapper closes over the shared id slot.
        let id_slot = Rc::clone(&self.current_id);
        let job: TimeoutJob = Box::new(move || {
            id_slot.set(None);
            callback();
        });
        let id = set_timeout(job, delay);
        self.current_id.set(Some(id));
    }

    /// Cancels the pending timeout, if any
    /// (`packages/utils/src/useTimeout.ts:31-36`); a no-op when nothing is pending — clearing
    /// an idle or already-fired instance neither throws nor calls `clearTimeout(0)`
    /// (`specs/utils/useTimeout.md`, "Edge cases").
    pub fn clear(&self) {
        if let Some(id) = self.current_id.take() {
            clear_timeout(id);
        }
    }

    /// Returns the clear behavior as a cleanup — upstream's `disposeEffect`
    /// (`packages/utils/src/useTimeout.ts:38-40`), consumed by `useOnMount` in the upstream
    /// hook (`packages/utils/src/useTimeout.ts:49`). [`use_timeout`] wires it to
    /// [`on_cleanup`] directly; exposed for callers building their own mount-effect plumbing.
    pub fn dispose_effect(&self) -> impl Fn() + 'static {
        let timeout = self.clone();
        move || timeout.clear()
    }
}

/// A `setTimeout` with automatic cleanup and guard — upstream's `useTimeout()` hook
/// (`packages/utils/src/useTimeout.ts:46-52`): creates the instance (a Leptos component body
/// runs once, so the binding is the stable instance the way upstream's `useRefWithInit`
/// memoizes it) and clears any pending timeout when the owning reactive scope is disposed
/// (upstream registers the instance's `disposeEffect` through `useOnMount`). UNVERIFIED
/// upstream — no test asserts the hook; must be called inside a reactive owner (a component).
pub fn use_timeout() -> Timeout {
    let timeout = Timeout::create();
    let cleanup = SendWrapper::new(timeout.dispose_effect());
    on_cleanup(move || (*cleanup)());
    timeout
}

thread_local! {
    /// Native registrations whose wrapper closure must stay alive until the timeout fires or
    /// is cleared, keyed by the id the ambient global returned. A wrapper fires at most once,
    /// so a registration leaves this registry either by firing (its own retirement) or by
    /// clearing.
    #[cfg(target_arch = "wasm32")]
    static LIVE_JOBS: RefCell<HashMap<TimeoutId, Closure<dyn FnMut()>>> =
        RefCell::new(HashMap::new());

    /// Retired wrapper closures awaiting drop. A wrapper can be retired during another
    /// wrapper's invocation (a callback that clears or reschedules siblings), so the drop is
    /// deferred to a sweep that provably runs outside that invocation — see
    /// [`sweep_retired_jobs`].
    #[cfg(target_arch = "wasm32")]
    static RETIRED_JOBS: RefCell<HashMap<TimeoutId, Closure<dyn FnMut()>>> =
        RefCell::new(HashMap::new());

    /// Ids whose wrapper is currently executing on the JS stack. Sweeps must not drop a
    /// retired wrapper whose id is here — wasm-bindgen forbids dropping a `Closure` while it
    /// is being invoked.
    #[cfg(target_arch = "wasm32")]
    static ACTIVE_IDS: RefCell<HashSet<TimeoutId>> = RefCell::new(HashSet::new());

    /// Host-only dispatch override — the Rust analog of stubbing the ambient
    /// `setTimeout`/`clearTimeout` globals, which host builds do not have. The wasm
    /// production target always resolves the real ambient globals per call.
    #[cfg(not(target_arch = "wasm32"))]
    static DISPATCH_OVERRIDE: RefCell<Option<(RequestOverride, CancelOverride)>> =
        const { RefCell::new(None) };
}

/// Drops retired wrapper closures that are provably no longer on the JS stack. Safe at any
/// module entry point — including a nested one from inside a running job — because a wrapper
/// retired during an invocation stays parked in [`RETIRED_JOBS`] until that invocation has
/// unwound (its id leaves [`ACTIVE_IDS`] only after the wrapper's body returns).
#[cfg(target_arch = "wasm32")]
fn sweep_retired_jobs() {
    ACTIVE_IDS.with(|active| {
        let active = active.borrow();
        RETIRED_JOBS.with(|retired| {
            retired.borrow_mut().retain(|id, _| active.contains(id));
        });
    });
}

/// Reads a global function by name, resolved per call (see the module docs).
#[cfg(target_arch = "wasm32")]
fn get_global_function(name: &str) -> Option<js_sys::Function> {
    js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str(name))
        .ok()
        .and_then(|value| value.dyn_into().ok())
}

/// Moves a registration's wrapper from the live registry to the retired registry — either
/// because it fired (a one-shot registration is complete; the native side will never call it
/// again) or because it was cleared. The drop itself is deferred to the sweep.
#[cfg(target_arch = "wasm32")]
fn retire_job(id: TimeoutId) {
    LIVE_JOBS.with(|jobs| {
        if let Some(retired) = jobs.borrow_mut().remove(&id) {
            RETIRED_JOBS.with(|retired_jobs| {
                retired_jobs.borrow_mut().insert(id, retired);
            });
        }
    });
}

/// Dispatches one registration through `globalThis.setTimeout(fn, delay)`
/// (`packages/utils/src/useTimeout.ts:21-24`) and keeps the wrapper closure alive until the
/// timeout fires or is cleared. The wrapper marks its own id active around the job so a
/// callback that retires other wrappers mid-invocation cannot have them dropped while their
/// ids are on the stack, and retires itself afterwards — a one-shot registration is complete
/// once it has fired.
#[cfg(target_arch = "wasm32")]
fn set_timeout(job: TimeoutJob, delay: u32) -> TimeoutId {
    sweep_retired_jobs();

    let native_id_slot = Rc::new(Cell::new(0u32));
    let id_for_wrapper = Rc::clone(&native_id_slot);
    let mut job = Some(job);
    let wrapper = Closure::wrap(Box::new(move || {
        let native_id = id_for_wrapper.get();
        ACTIVE_IDS.with(|active| active.borrow_mut().insert(native_id));
        if let Some(job) = job.take() {
            job();
        }
        ACTIVE_IDS.with(|active| active.borrow_mut().remove(&native_id));
        retire_job(native_id);
    }) as Box<dyn FnMut()>);

    // The ambient global never fires synchronously (`setTimeout` is async by specification),
    // so the registration below is complete before the wrapper can possibly run and read
    // `native_id_slot`.
    let set_timeout_function = get_global_function("setTimeout")
        .expect("globalThis.setTimeout must be a function to schedule a timeout");
    let native_id = set_timeout_function
        .call2(
            &js_sys::global(),
            wrapper.as_ref().unchecked_ref(),
            &JsValue::from_f64(f64::from(delay)),
        )
        .expect("setTimeout threw while scheduling a timeout");
    let native_id = native_id
        .as_f64()
        .expect("timeout dispatch must return a numeric id")
        as TimeoutId;
    native_id_slot.set(native_id);
    LIVE_JOBS.with(|jobs| {
        jobs.borrow_mut().insert(native_id, wrapper);
    });
    native_id
}

/// Cancels a pending registration through `globalThis.clearTimeout`
/// (`packages/utils/src/useTimeout.ts:31-36`). The wrapper is retired rather than dropped
/// immediately: a clear can happen from inside another wrapper's invocation, and dropping a
/// `Closure` while wasm-bindgen is invoking it panics — the sweep defers the drop until the
/// invocation has unwound.
#[cfg(target_arch = "wasm32")]
fn clear_timeout(id: TimeoutId) {
    sweep_retired_jobs();

    let clear_timeout_function = get_global_function("clearTimeout")
        .expect("globalThis.clearTimeout must be a function to cancel a timeout");
    clear_timeout_function
        .call1(&js_sys::global(), &JsValue::from_f64(f64::from(id)))
        .expect("clearTimeout threw while canceling a timeout");
    retire_job(id);
}

/// The host override's request half: queue a registration, return its id.
#[cfg(not(target_arch = "wasm32"))]
type RequestOverride = Box<dyn Fn(TimeoutJob, u32) -> TimeoutId>;

/// The host override's cancel half: drop a queued registration by id.
#[cfg(not(target_arch = "wasm32"))]
type CancelOverride = Box<dyn Fn(TimeoutId)>;

/// Host-target dispatch: there are no browser globals to resolve, so tests install an override
/// ([`DISPATCH_OVERRIDE`]) — the analog of stubbing the ambient `setTimeout`/`clearTimeout`
/// globals, which host builds do not have.
#[cfg(not(target_arch = "wasm32"))]
fn set_timeout(job: TimeoutJob, delay: u32) -> TimeoutId {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (request, _) = dispatch.as_ref().expect(
            "no timeout dispatcher installed on the host target: host builds have no \
             setTimeout/clearTimeout globals, so tests must install a dispatch override",
        );
        request(job, delay)
    })
}

/// Host-target cancel — see [`set_timeout`] for why the override exists.
#[cfg(not(target_arch = "wasm32"))]
fn clear_timeout(id: TimeoutId) {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (_, cancel) = dispatch.as_ref().expect(
            "no timeout dispatcher installed on the host target: host builds have no \
             setTimeout/clearTimeout globals, so tests must install a dispatch override",
        );
        cancel(id)
    })
}

/// Installs the host dispatch override on behalf of a sibling module's tests (the
/// `use_scroll_lock` port drives this crate's [`Timeout`] through the same override), so the
/// sibling does not have to re-derive the override slot.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn install_dispatch_override_for_tests(
    request: Box<dyn Fn(Box<dyn FnOnce()>, u32) -> u32>,
    cancel: Box<dyn Fn(u32)>,
) {
    DISPATCH_OVERRIDE.with(|dispatch| {
        *dispatch.borrow_mut() = Some((request, cancel));
    });
}

/// Clears the host dispatch override — the Drop half of
/// [`install_dispatch_override_for_tests`].
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn reset_dispatch_override_for_tests() {
    DISPATCH_OVERRIDE.with(|dispatch| {
        *dispatch.borrow_mut() = None;
    });
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::collections::HashMap;

    use reactive_graph::owner::Owner;

    use super::*;

    /// The manual queue standing in for the browser's timer queue on the host target: every
    /// registration is queued with its delay, cancel removes it, and [`ManualQueue::flush`]
    /// fires each live registration exactly once and consumes it — the one-shot semantics of
    /// `setTimeout`, with the caller deciding when time advances.
    struct ManualQueue {
        timeouts: Rc<RefCell<Vec<(TimeoutId, Rc<RefCell<Option<TimeoutJob>>>)>>>,
        delays: Rc<RefCell<HashMap<TimeoutId, u32>>>,
    }

    impl ManualQueue {
        fn install() -> Self {
            let timeouts: Rc<RefCell<Vec<(TimeoutId, Rc<RefCell<Option<TimeoutJob>>>)>>> =
                Rc::new(RefCell::new(Vec::new()));
            let delays: Rc<RefCell<HashMap<TimeoutId, u32>>> =
                Rc::new(RefCell::new(HashMap::new()));
            let next_id = Cell::new(0u32);

            let queue_timeouts = Rc::clone(&timeouts);
            let queue_delays = Rc::clone(&delays);
            let request: RequestOverride = Box::new(move |job: TimeoutJob, delay: u32| {
                next_id.set(next_id.get() + 1);
                let id = next_id.get();
                queue_timeouts
                    .borrow_mut()
                    .push((id, Rc::new(RefCell::new(Some(job)))));
                queue_delays.borrow_mut().insert(id, delay);
                id
            });

            let cancel_timeouts = Rc::clone(&timeouts);
            let cancel_delays = Rc::clone(&delays);
            let cancel: CancelOverride = Box::new(move |id: TimeoutId| {
                cancel_timeouts
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                cancel_delays.borrow_mut().remove(&id);
            });

            DISPATCH_OVERRIDE.with(|dispatch| {
                *dispatch.borrow_mut() = Some((request, cancel));
            });
            Self { timeouts, delays }
        }

        /// Fires every live registration exactly once, in scheduling order, and consumes it.
        /// A registration scheduled during the flush (a self-rescheduling callback) waits for
        /// the next flush, like a fresh native timer; a registration cleared during the flush
        /// is skipped by the snapshot loop.
        fn flush(&self) {
            let ids: Vec<TimeoutId> = self.timeouts.borrow().iter().map(|(id, _)| *id).collect();
            for id in ids {
                let job = {
                    let timeouts = self.timeouts.borrow();
                    timeouts
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .and_then(|(_, slot)| slot.borrow_mut().take())
                };
                // The registration is consumed when it fires (one-shot native semantics).
                self.timeouts.borrow_mut().retain(|(queued, _)| *queued != id);
                self.delays.borrow_mut().remove(&id);
                if let Some(job) = job {
                    job();
                }
            }
        }

        /// The delay a registration was queued with — pins that `start` forwards `delay`
        /// straight through to the host.
        fn delay_for(&self, id: TimeoutId) -> Option<u32> {
            self.delays.borrow().get(&id).copied()
        }
    }

    impl Drop for ManualQueue {
        fn drop(&mut self) {
            DISPATCH_OVERRIDE.with(|dispatch| {
                *dispatch.borrow_mut() = None;
            });
        }
    }

    fn counting_callback() -> (Rc<Cell<u32>>, impl FnOnce() + 'static) {
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        (fired, move || counter.set(counter.get() + 1))
    }

    // Pins the handle-construction contract (`packages/utils/src/useTimeout.ts:10-12`): two
    // `create()` calls return distinct instances, each usable as its own scheduler.
    #[test]
    fn create_returns_a_distinct_instance_each_time() {
        let a: Timeout = Timeout::create();
        let b: Timeout = Timeout::create();
        assert!(
            !Rc::ptr_eq(&a.current_id, &b.current_id),
            "create() must return a new instance each time"
        );
    }

    // Pins the dispatch contract (`packages/utils/src/useTimeout.ts:19-25`): the callback
    // does not run on `start` — the first invocation happens after one full delay — and it
    // runs exactly once, unlike the repeating `Interval`.
    #[test]
    fn start_does_not_fire_synchronously_and_fires_exactly_once() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (fired, callback) = counting_callback();

        timeout.start(100, callback);
        assert_eq!(
            fired.get(),
            0,
            "the callback runs asynchronously after the full delay, not synchronously"
        );

        queue.flush();
        assert_eq!(fired.get(), 1);

        queue.flush();
        assert_eq!(fired.get(), 1, "a one-shot timeout never fires a second time");
    }

    // Pins the one-shot latch (`packages/utils/src/useTimeout.ts:21-24`): the wrapper resets
    // the id slot to EMPTY before invoking the callback, so a fired timeout leaves the
    // instance idle again — the exact point where `Interval` deliberately diverges
    // (`packages/utils/src/useInterval.ts:20-22`).
    #[test]
    fn a_fired_timeout_leaves_the_instance_idle_again() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (_fired, callback) = counting_callback();

        timeout.start(100, callback);
        assert!(
            timeout.is_started(),
            "start() latches the pending id until the timer fires"
        );

        queue.flush();
        assert!(
            !timeout.is_started(),
            "the fired wrapper resets the id slot before invoking the callback"
        );
        assert!(timeout.current_id().is_none());
    }

    // Pins `isStarted` (`packages/utils/src/useTimeout.ts:27-29`) across the instance's
    // lifecycle: fresh (idle), started (pending), cleared (idle again).
    #[test]
    fn is_started_tracks_the_pending_state() {
        let _queue = ManualQueue::install();
        let timeout = Timeout::create();
        assert!(!timeout.is_started(), "a fresh instance is not started");

        let (_fired, callback) = counting_callback();
        timeout.start(50, callback);
        assert!(timeout.is_started(), "start() latches the pending id");

        timeout.clear();
        assert!(!timeout.is_started(), "clear() resets the id to EMPTY");
    }

    // Pins the replace-not-stack contract (`packages/utils/src/useTimeout.ts:19-25`): a
    // second `start` clears the previously scheduled timeout, so only the newest callback
    // fires — at most one live timeout exists per instance.
    #[test]
    fn start_clears_any_previously_scheduled_timeout() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (first_fired, first) = counting_callback();
        let (second_fired, second) = counting_callback();

        timeout.start(100, first);
        timeout.start(100, second);

        queue.flush();
        assert_eq!(second_fired.get(), 1);
        assert_eq!(first_fired.get(), 0, "the superseded callback never fires");

        queue.flush();
        assert_eq!(second_fired.get(), 1, "and only once");
        assert_eq!(first_fired.get(), 0);
    }

    // Pins the cancel contract (`packages/utils/src/useTimeout.ts:31-36`): a cleared
    // registration never fires.
    #[test]
    fn clear_cancels_a_pending_timeout() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (fired, callback) = counting_callback();

        timeout.start(100, callback);
        timeout.clear();

        queue.flush();
        assert_eq!(fired.get(), 0);
    }

    // Pins the guard (`packages/utils/src/useTimeout.ts:32-35`): clearing an idle or
    // already-fired instance is a no-op — it neither throws nor calls `clearTimeout(0)`.
    #[test]
    fn clear_on_an_idle_instance_is_a_no_op() {
        let _queue = ManualQueue::install();
        let timeout = Timeout::create();

        timeout.clear();
        timeout.clear();
        assert!(!timeout.is_started());
        assert!(timeout.current_id().is_none());
    }

    // Pins the delay passthrough (`packages/utils/src/useTimeout.ts:24-25`): `delay` is
    // passed straight to `setTimeout` with no clamping or validation
    // (`specs/utils/useTimeout.md`, "Edge cases").
    #[test]
    fn start_forwards_the_delay_unchanged() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (_fired, callback) = counting_callback();

        timeout.start(250, callback);
        let id = timeout
            .current_id()
            .expect("a started timeout has a pending id");
        assert_eq!(queue.delay_for(id), Some(250));
    }

    // Pins the inherited `disposeEffect` contract (`packages/utils/src/useTimeout.ts:38-40`):
    // the returned cleanup cancels a pending timeout.
    #[test]
    fn dispose_effect_cancels_a_pending_timeout() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (fired, callback) = counting_callback();

        timeout.start(100, callback);
        let dispose = timeout.dispose_effect();
        dispose();

        queue.flush();
        assert_eq!(fired.get(), 0);
    }

    // Pins the re-entrancy contract (`packages/utils/src/useTimeout.ts:21-24` plus
    // `specs/utils/useTimeout.md`, "Edge cases"): code inside the callback observes an idle
    // instance — `isStarted()` returns `false`, `clear()` is a no-op — and calling `start()`
    // from inside the callback re-arms a fresh timer cleanly.
    #[test]
    fn a_callback_observes_an_idle_instance_and_can_re_arm_cleanly() {
        let queue = ManualQueue::install();
        let timeout = Timeout::create();
        let (first_fired, _first) = counting_callback();
        let (second_fired, second) = counting_callback();
        let observed_started = Rc::new(Cell::new(true));
        let handle = timeout.clone();

        let first_counter = Rc::clone(&first_fired);
        let observed = Rc::clone(&observed_started);
        timeout.start(100, move || {
            first_counter.set(first_counter.get() + 1);
            observed.set(handle.is_started());
            handle.clear();
            handle.start(50, second);
        });

        queue.flush();
        assert_eq!(first_fired.get(), 1);
        assert!(
            !observed_started.get(),
            "the callback observes an idle instance: its own wrapper already reset the id slot"
        );

        queue.flush();
        assert_eq!(
            second_fired.get(),
            1,
            "the re-armed timer fires on a later tick, not within the same flush"
        );
        assert!(!timeout.is_started());
        assert_eq!(first_fired.get(), 1, "the one-shot first callback never re-fires");
    }

    // Pins per-instance scheduling (`specs/utils/useTimeout.md`, "Edge cases"): each call
    // site gets its own independent `Timeout` instance, so nested or sibling consumers do not
    // share scheduling state.
    #[test]
    fn two_instances_fire_independently() {
        let queue = ManualQueue::install();
        let first_timeout = Timeout::create();
        let second_timeout = Timeout::create();
        let (first_fired, first) = counting_callback();
        let (second_fired, second) = counting_callback();

        first_timeout.start(100, first);
        second_timeout.start(200, second);

        queue.flush();
        assert_eq!(first_fired.get(), 1);
        assert_eq!(second_fired.get(), 1);
    }

    // Pins the hook's unmount behavior (`packages/utils/src/useTimeout.ts:49`): a pending
    // timeout started via the hook's scheduler is cleared when the component unmounts —
    // reactive owner disposal plays the role of React unmount running the `useOnMount`
    // cleanup (`packages/utils/src/useOnMount.ts:8-12`).
    #[test]
    fn use_timeout_cancels_the_pending_timeout_when_the_reactive_owner_is_disposed() {
        let queue = ManualQueue::install();

        let owner = Owner::new();
        owner.set();
        let timeout = use_timeout();
        let (fired, callback) = counting_callback();
        timeout.start(100, callback);

        owner.cleanup();
        queue.flush();
        assert_eq!(fired.get(), 0, "the pending timeout is cleared on unmount");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use js_sys::{Function, Reflect};
    use reactive_graph::owner::Owner;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn set_global(key: &str, value: &JsValue) {
        Reflect::set(&js_sys::global(), &JsValue::from_str(key), value)
            .unwrap_or_else(|error| panic!("failed to set globalThis.{key}: {error:?}"));
    }

    /// The global stub standing in for the browser's timer queue: `setTimeout` queues every
    /// registration and returns a fresh numeric id, `clearTimeout` removes it. Restores the
    /// originals on drop so a panicking test cannot poison the later ones.
    struct TimeoutStub {
        original_set: JsValue,
        original_clear: JsValue,
        /// Kept alive for the stub's lifetime; the browser only sees the JS side.
        #[allow(dead_code)]
        set_closure: Closure<dyn FnMut(Function, JsValue) -> JsValue>,
        #[allow(dead_code)]
        clear_closure: Closure<dyn FnMut(JsValue)>,
        timeouts: Rc<RefCell<Vec<(u32, Function)>>>,
        delays: Rc<RefCell<HashMap<u32, u32>>>,
    }

    impl TimeoutStub {
        fn install() -> Self {
            let timeouts: Rc<RefCell<Vec<(u32, Function)>>> = Rc::new(RefCell::new(Vec::new()));
            let delays: Rc<RefCell<HashMap<u32, u32>>> = Rc::new(RefCell::new(HashMap::new()));
            let next_id = Cell::new(0u32);

            let set_timeouts = Rc::clone(&timeouts);
            let set_delays = Rc::clone(&delays);
            let set_closure =
                Closure::wrap(Box::new(move |callback: Function, delay: JsValue| -> JsValue {
                    next_id.set(next_id.get() + 1);
                    let id = next_id.get();
                    set_timeouts.borrow_mut().push((id, callback));
                    set_delays
                        .borrow_mut()
                        .insert(id, delay.as_f64().unwrap_or(0.0) as u32);
                    JsValue::from_f64(f64::from(id))
                })
                    as Box<dyn FnMut(Function, JsValue) -> JsValue>);

            let clear_timeouts = Rc::clone(&timeouts);
            let clear_delays = Rc::clone(&delays);
            let clear_closure = Closure::wrap(Box::new(move |id: JsValue| {
                let id = id.as_f64().unwrap_or(0.0) as u32;
                clear_timeouts
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                clear_delays.borrow_mut().remove(&id);
            }) as Box<dyn FnMut(JsValue)>);

            let global = js_sys::global();
            let set_key = JsValue::from_str("setTimeout");
            let clear_key = JsValue::from_str("clearTimeout");
            let original_set = Reflect::get(&global, &set_key)
                .expect("globalThis.setTimeout must be readable");
            let original_clear = Reflect::get(&global, &clear_key)
                .expect("globalThis.clearTimeout must be readable");
            set_global("setTimeout", set_closure.as_ref().unchecked_ref());
            set_global("clearTimeout", clear_closure.as_ref().unchecked_ref());

            Self {
                original_set,
                original_clear,
                set_closure,
                clear_closure,
                timeouts,
                delays,
            }
        }

        /// Fires every live registration exactly once, in scheduling order, and consumes it.
        /// A registration scheduled during the flush (a self-rescheduling callback) waits for
        /// the next flush, like a fresh native timer; a registration cleared during the flush
        /// is skipped by the snapshot loop.
        fn flush(&self) {
            let ids: Vec<u32> = self.timeouts.borrow().iter().map(|(id, _)| *id).collect();
            for id in ids {
                let callback = {
                    let timeouts = self.timeouts.borrow();
                    timeouts
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .map(|(_, callback)| callback.clone())
                };
                // The registration is consumed when it fires (one-shot native semantics).
                self.timeouts.borrow_mut().retain(|(queued, _)| *queued != id);
                self.delays.borrow_mut().remove(&id);
                if let Some(callback) = callback {
                    callback
                        .call0(&JsValue::UNDEFINED)
                        .expect("the timeout callback threw");
                }
            }
        }

        /// The delay a registration was queued with — pins that `start` forwards `delay`
        /// straight through to the host.
        fn delay_for(&self, id: u32) -> Option<u32> {
            self.delays.borrow().get(&id).copied()
        }
    }

    impl Drop for TimeoutStub {
        fn drop(&mut self) {
            set_global("setTimeout", &self.original_set);
            set_global("clearTimeout", &self.original_clear);
        }
    }

    // Pins the dispatch contract (`packages/utils/src/useTimeout.ts:19-25`) through the
    // ambient globals: asynchronous first invocation, exactly-once delivery, and an id slot
    // reset by the fired wrapper (the one-shot latch).
    #[wasm_bindgen_test]
    fn fires_exactly_once_through_the_ambient_globals() {
        let stub = TimeoutStub::install();
        let timeout = Timeout::create();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);

        timeout.start(100, move || counter.set(counter.get() + 1));
        assert_eq!(
            fired.get(),
            0,
            "the callback runs asynchronously after the full delay, not synchronously"
        );
        assert!(timeout.is_started());

        stub.flush();
        assert_eq!(fired.get(), 1);
        assert!(
            !timeout.is_started(),
            "the fired wrapper resets the id slot before invoking the callback"
        );

        stub.flush();
        assert_eq!(fired.get(), 1, "a one-shot timeout never fires a second time");

        drop(stub);
    }

    // Pins the delay passthrough and the replace-not-stack contract
    // (`packages/utils/src/useTimeout.ts:19-25`): `delay` reaches the host unchanged and a
    // second `start` supersedes the first.
    #[wasm_bindgen_test]
    fn clear_cancels_and_a_second_start_supersedes_through_the_ambient_globals() {
        let stub = TimeoutStub::install();
        let timeout = Timeout::create();

        let canceled = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&canceled);
        timeout.start(100, move || counter.set(counter.get() + 1));
        let id = timeout.current_id().expect("a started timeout has an id");
        assert_eq!(stub.delay_for(id), Some(100));

        timeout.clear();
        assert!(!timeout.is_started());

        let superseding = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&superseding);
        timeout.start(200, move || counter.set(counter.get() + 1));
        let id = timeout.current_id().expect("a restarted timeout has an id");
        assert_eq!(stub.delay_for(id), Some(200));

        stub.flush();
        assert_eq!(canceled.get(), 0);
        assert_eq!(superseding.get(), 1);

        drop(stub);
    }

    // Pins the closure-lifetime machinery for the self-rescheduling pattern: a callback that
    // re-`start`s its own instance during its own invocation both retires its own wrapper
    // (one-shot completion) and schedules a new registration from inside the wrapper — the
    // sweep between the two must not drop the still-executing wrapper.
    #[wasm_bindgen_test]
    fn a_callback_that_reschedules_the_instance_during_its_own_invocation_re_arms_cleanly() {
        let stub = TimeoutStub::install();
        let timeout = Timeout::create();
        let first_fired = Rc::new(Cell::new(0u32));
        let second_fired = Rc::new(Cell::new(0u32));
        let handle = timeout.clone();

        let first_counter = Rc::clone(&first_fired);
        let second_counter = Rc::clone(&second_fired);
        timeout.start(100, move || {
            first_counter.set(first_counter.get() + 1);
            let second_counter = Rc::clone(&second_counter);
            handle.start(100, move || second_counter.set(second_counter.get() + 1));
        });

        stub.flush();
        assert_eq!(first_fired.get(), 1);
        assert_eq!(
            second_fired.get(),
            0,
            "the rescheduled callback waits a full delay"
        );

        stub.flush();
        assert_eq!(first_fired.get(), 1, "the one-shot callback never re-fires");
        assert_eq!(second_fired.get(), 1);

        drop(stub);
    }

    // Pins the closure-lifetime machinery for the sibling-clear pattern: a callback that
    // clears a *different* pending instance during its own invocation retires that sibling's
    // wrapper while another wrapper is executing — the sweep must defer the drop.
    #[wasm_bindgen_test]
    fn a_callback_that_clears_a_sibling_instance_during_its_own_invocation_cancels_it() {
        let stub = TimeoutStub::install();
        let clearing = Timeout::create();
        let sibling = Timeout::create();
        let cleared_fired = Rc::new(Cell::new(0u32));
        let sibling_counter = Rc::clone(&cleared_fired);
        sibling.start(100, move || sibling_counter.set(sibling_counter.get() + 1));

        let clearing_handle = clearing.clone();
        let sibling_handle = sibling.clone();
        clearing.start(50, move || {
            sibling_handle.clear();
            let _ = clearing_handle.is_started();
        });

        stub.flush();
        assert_eq!(cleared_fired.get(), 0, "the cleared sibling never fires");
        assert!(!sibling.is_started());

        stub.flush();
        assert_eq!(cleared_fired.get(), 0);

        drop(stub);
    }

    // Pins the hook's unmount behavior (`packages/utils/src/useTimeout.ts:49`): a pending
    // timeout started via the hook's scheduler is cleared when the component unmounts —
    // reactive owner disposal plays the role of React unmount.
    #[wasm_bindgen_test]
    fn use_timeout_cancels_the_pending_timeout_when_the_reactive_owner_is_disposed() {
        let stub = TimeoutStub::install();

        let owner = Owner::new();
        owner.set();
        let timeout = use_timeout();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        timeout.start(100, move || counter.set(counter.get() + 1));

        owner.cleanup();
        stub.flush();
        assert_eq!(fired.get(), 0, "the pending timeout is cleared on unmount");

        drop(stub);
    }
}
