//! Port of `packages/utils/src/useInterval.ts` (Base UI Phase A util).
//!
//! Upstream is a single 42-line module: an `Interval` class extending the `Timeout` class from
//! the sibling `useTimeout` module, plus the `useInterval()` hook owning one instance per
//! component (`packages/utils/src/useInterval.ts:10-42`). `Interval` inherits `Timeout`'s
//! `currentId` slot, `isStarted()`, and `disposeEffect`
//! (`packages/utils/src/useTimeout.ts:14`, `:27-29`, `:38-40`) and overrides two members:
//!
//! - `start(delay, fn)` clears any previously scheduled call, then schedules a wrapper on
//!   `setInterval` — the wrapper invokes `fn()` with zero arguments on every tick
//!   (`packages/utils/src/useInterval.ts:18-23`).
//! - `clear` (a pre-bound arrow-function class property) is guarded on `currentId !== EMPTY`,
//!   so clearing an idle instance is a no-op (`packages/utils/src/useInterval.ts:25-30`).
//!
//! The load-bearing difference from the parent `Timeout`: `Timeout.start` resets `currentId`
//! to `EMPTY` inside its callback — one-shot semantics
//! (`packages/utils/src/useTimeout.ts:21-24`) — while `Interval.start` deliberately does not,
//! so the id stays live across every tick until `clear()` runs, which is what makes the
//! inherited `isStarted()` meaningful for a repeating timer
//! (`packages/utils/src/useInterval.ts:20-22`).
//!
//! Source of truth: **none.** The upstream repo has no test file for this unit
//! (`specs/utils/useInterval.md`, "Source of truth"), so every upstream claim in this module is
//! a source-derived description of current implementation behavior, not test-proven behavior.
//! The tests below pin the ported contract itself.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Rust has no class inheritance, so `Interval` carries the members it would inherit — the
//!   `EMPTY`-initialized id slot, `isStarted`, and `disposeEffect` — directly. The duplication
//!   is a few lines of handle logic; the crate's `use_idle_callback`/`use_animation_frame`
//!   ports already establish the self-contained-handle precedent, and the `Timeout` port (its
//!   own TODO item) can extract a shared base later if that proves worthwhile.
//! - The `EMPTY = 0` sentinel (`packages/utils/src/useInterval.ts:6-8`) becomes [`Option`]:
//!   upstream needed the numeric `0` because the id slot is a JS number; the port stores the
//!   pending id in a clone-shared `Rc<Cell<Option<IntervalId>>>`, the same shape as the
//!   `use_idle_callback`/`use_animation_frame` ports, so clones of the handle share the slot
//!   the way JS callers share the instance.
//! - The zero-argument, repeatedly-invoked `fn: Function`
//!   (`packages/utils/src/useInterval.ts:20-22`) becomes [`Interval::start`]'s
//!   `impl FnMut() + 'static`: the callback receives no arguments and may be invoked on every
//!   tick. `delay` is a `u32` passed straight through to `setInterval` — no clamping or
//!   validation here; host-environment clamping (e.g. the 4 ms minimum for nested timers)
//!   applies exactly as upstream (`specs/utils/useInterval.md`, "Edge cases").
//! - Dispatch goes through the ambient `globalThis.setInterval`/`clearInterval` resolved per
//!   call (`packages/utils/src/useInterval.ts:20-22`, `:25-30`), matching the crate's
//!   `use_idle_callback` precedent of reading ambient browser globals at dispatch time. JS
//!   garbage-collects the bound wrapper; the port keeps each registration's wrapper closure in
//!   a live registry keyed by the native id and drops it only after the native cancel. Because
//!   an interval wrapper is invoked repeatedly and can be cleared *during its own invocation*
//!   (a self-clearing or self-rescheduling callback), the drop is deferred: cleared wrappers
//!   move to a retired registry, and sweeps never drop a wrapper whose id is still executing
//!   on the JS stack (an active-id set marks invocations). This is the same wasm-bindgen
//!   no-drop-while-invocable rule the `use_idle_callback` port documents, adapted for a
//!   repeating callback.
//! - The `useInterval` hook (`packages/utils/src/useInterval.ts:36-42`) dissolves React's
//!   `useRefWithInit`/`useOnMount` plumbing into Leptos owner semantics, exactly like the
//!   crate's other timer-adjacent hooks: a component's setup body runs once, so the local
//!   binding is the stable instance the way upstream's `useRefWithInit` memoizes it
//!   (`packages/utils/src/useRefWithInit.ts:16-22`), and [`on_cleanup`] plays the role of the
//!   mount effect's returned cleanup (`packages/utils/src/useInterval.ts:39` via
//!   `packages/utils/src/useOnMount.ts:8-12` and `packages/utils/src/useTimeout.ts:38-40`) —
//!   a pending interval is cleared when the owning reactive scope is disposed. Must be called
//!   inside a reactive owner (a component).
//! - `'use client'` (`packages/utils/src/useInterval.ts:1`) is N/A — there is no React Server
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

/// The id the ambient `setInterval` global returns — upstream's `IntervalId` number
/// (`packages/utils/src/useInterval.ts:6`), with the `EMPTY = 0` sentinel
/// (`packages/utils/src/useInterval.ts:8`) modeled as [`Option`] instead.
pub type IntervalId = u32;

/// The scheduled unit — upstream's `fn: Function` parameter
/// (`packages/utils/src/useInterval.ts:18`), invoked once per tick, boxed because it crosses
/// the dispatch boundary as a stored callback.
type IntervalJob = Box<dyn FnMut()>;

/// An instance handle keeping at most one repeating callback pending — upstream's `Interval`
/// class (`packages/utils/src/useInterval.ts:10-31`) including the members inherited from
/// `Timeout` (`packages/utils/src/useTimeout.ts:14`, `:27-29`, `:38-40`). Cheap to clone;
/// clones share the same pending-id slot.
#[derive(Clone, Default)]
pub struct Interval {
    /// Upstream `currentId` (`packages/utils/src/useTimeout.ts:14`, initialized to the `EMPTY`
    /// sentinel); `None` is upstream's `EMPTY`.
    current_id: Rc<Cell<Option<IntervalId>>>,
}

impl Interval {
    /// Upstream `static create()` (`packages/utils/src/useInterval.ts:11-13`).
    pub fn create() -> Self {
        Self::default()
    }

    /// The currently pending interval id, if any — upstream's public `currentId` field
    /// (`packages/utils/src/useTimeout.ts:14`). Unlike the one-shot timers, it stays set
    /// across every tick — only [`Interval::clear`] resets it
    /// (`packages/utils/src/useInterval.ts:20-22`).
    pub fn current_id(&self) -> Option<IntervalId> {
        self.current_id.get()
    }

    /// Whether an interval is pending — upstream's inherited `isStarted()`
    /// (`packages/utils/src/useTimeout.ts:27-29`). Stays `true` across every tick of a running
    /// interval, because `Interval.start` — unlike `Timeout.start`
    /// (`packages/utils/src/useTimeout.ts:21-24`) — does not reset the id inside the callback
    /// (`packages/utils/src/useInterval.ts:20-22`).
    pub fn is_started(&self) -> bool {
        self.current_id.get().is_some()
    }

    /// Executes `callback` at `delay` intervals, clearing any previously scheduled call
    /// (`packages/utils/src/useInterval.ts:18-23`). At most one live interval exists per
    /// instance: re-`start` cancels the pending interval instead of stacking, so callbacks
    /// from superseded `start` calls never fire
    /// (`specs/utils/useInterval.md`, "Edge cases"). The first invocation happens after one
    /// full `delay` — plain `setInterval` semantics, no leading call.
    pub fn start(&self, delay: u32, mut callback: impl FnMut() + 'static) {
        self.clear();
        // Unlike `Timeout.start`, the wrapper does not reset the id slot — the id stays live
        // across every tick (`packages/utils/src/useInterval.ts:20-22`).
        let id = set_interval(Box::new(move || callback()), delay);
        self.current_id.set(Some(id));
    }

    /// Cancels the pending interval, if any
    /// (`packages/utils/src/useInterval.ts:25-30`); a no-op when nothing is pending —
    /// clearing an idle or already-cleared instance neither throws nor calls
    /// `clearInterval(0)` (`specs/utils/useInterval.md`, "Edge cases").
    pub fn clear(&self) {
        if let Some(id) = self.current_id.take() {
            clear_interval(id);
        }
    }

    /// Returns the cancel behavior as a cleanup — upstream's inherited `disposeEffect`
    /// (`packages/utils/src/useTimeout.ts:38-40`), consumed by `useOnMount` in the upstream
    /// hook (`packages/utils/src/useInterval.ts:39`). [`use_interval`] wires it to
    /// [`on_cleanup`] directly; exposed for callers building their own mount-effect plumbing.
    pub fn dispose_effect(&self) -> impl Fn() + 'static {
        let interval = self.clone();
        move || interval.clear()
    }
}

/// A `setInterval` with automatic cleanup and guard — upstream's `useInterval()` hook
/// (`packages/utils/src/useInterval.ts:36-42`): creates the instance (a Leptos component body
/// runs once, so the binding is the stable instance the way upstream's `useRefWithInit`
/// memoizes it) and clears any pending interval when the owning reactive scope is disposed
/// (upstream registers the instance's `disposeEffect` through `useOnMount`). UNVERIFIED
/// upstream — no test asserts the hook; must be called inside a reactive owner (a component).
pub fn use_interval() -> Interval {
    let interval = Interval::create();
    let cleanup = SendWrapper::new(interval.dispose_effect());
    on_cleanup(move || (*cleanup)());
    interval
}

thread_local! {
    /// Native registrations whose wrapper closure must stay alive until the interval is
    /// cleared, keyed by the id the ambient global returned. The wrapper fires repeatedly, so
    /// unlike a one-shot timer it is never retired by firing — only by clearing.
    #[cfg(target_arch = "wasm32")]
    static LIVE_JOBS: RefCell<HashMap<IntervalId, Closure<dyn FnMut()>>> =
        RefCell::new(HashMap::new());

    /// Cleared wrapper closures awaiting drop. A wrapper may be cleared during its own
    /// invocation (a self-clearing or self-rescheduling callback), so the drop is deferred to
    /// a sweep that provably runs outside that invocation — see [`sweep_retired_jobs`].
    #[cfg(target_arch = "wasm32")]
    static RETIRED_JOBS: RefCell<HashMap<IntervalId, Closure<dyn FnMut()>>> =
        RefCell::new(HashMap::new());

    /// Ids whose wrapper is currently executing on the JS stack. Sweeps must not drop a
    /// retired wrapper whose id is here — wasm-bindgen forbids dropping a `Closure` while it
    /// is being invoked.
    #[cfg(target_arch = "wasm32")]
    static ACTIVE_IDS: RefCell<HashSet<IntervalId>> = RefCell::new(HashSet::new());

    /// Host-only dispatch override — the Rust analog of stubbing the ambient
    /// `setInterval`/`clearInterval` globals, which host builds do not have. The wasm
    /// production target always resolves the real ambient globals per call.
    #[cfg(not(target_arch = "wasm32"))]
    static DISPATCH_OVERRIDE: RefCell<Option<(RequestOverride, CancelOverride)>> =
        const { RefCell::new(None) };
}

/// Drops retired wrapper closures that are provably no longer on the JS stack. Safe at any
/// module entry point — including a nested one from inside a running job — because a wrapper
/// cleared during its own invocation stays parked in [`RETIRED_JOBS`] until its invocation has
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

/// Dispatches one registration through `globalThis.setInterval(fn, delay)`
/// (`packages/utils/src/useInterval.ts:20-22`) and keeps the wrapper closure alive until the
/// interval is cleared. The wrapper marks its own id active around every tick so a
/// self-clearing or self-rescheduling callback can retire it mid-invocation without anyone
/// dropping it while it is still on the stack.
#[cfg(target_arch = "wasm32")]
fn set_interval(mut job: IntervalJob, delay: u32) -> IntervalId {
    sweep_retired_jobs();

    let native_id_slot = Rc::new(Cell::new(0u32));
    let id_for_wrapper = Rc::clone(&native_id_slot);
    let wrapper = Closure::wrap(Box::new(move || {
        let native_id = id_for_wrapper.get();
        ACTIVE_IDS.with(|active| active.borrow_mut().insert(native_id));
        job();
        ACTIVE_IDS.with(|active| active.borrow_mut().remove(&native_id));
    }) as Box<dyn FnMut()>);

    // The ambient global never fires synchronously (`setInterval` is async by specification),
    // so the registration below is complete before the wrapper can possibly run and read
    // `native_id_slot`.
    let set_interval_function = get_global_function("setInterval")
        .expect("globalThis.setInterval must be a function to schedule an interval");
    let native_id = set_interval_function
        .call2(
            &js_sys::global(),
            wrapper.as_ref().unchecked_ref(),
            &JsValue::from_f64(f64::from(delay)),
        )
        .expect("setInterval threw while scheduling an interval");
    let native_id = native_id
        .as_f64()
        .expect("interval dispatch must return a numeric id")
        as IntervalId;
    native_id_slot.set(native_id);
    LIVE_JOBS.with(|jobs| {
        jobs.borrow_mut().insert(native_id, wrapper);
    });
    native_id
}

/// Cancels a pending registration through `globalThis.clearInterval`
/// (`packages/utils/src/useInterval.ts:25-30`). The wrapper is retired rather than dropped
/// immediately: a clear can happen from inside the wrapper's own invocation, and dropping a
/// `Closure` while wasm-bindgen is invoking it panics — the sweep defers the drop until the
/// invocation has unwound.
#[cfg(target_arch = "wasm32")]
fn clear_interval(id: IntervalId) {
    sweep_retired_jobs();

    let clear_interval_function = get_global_function("clearInterval")
        .expect("globalThis.clearInterval must be a function to cancel an interval");
    clear_interval_function
        .call1(&js_sys::global(), &JsValue::from_f64(f64::from(id)))
        .expect("clearInterval threw while canceling an interval");
    LIVE_JOBS.with(|jobs| {
        if let Some(retired) = jobs.borrow_mut().remove(&id) {
            RETIRED_JOBS.with(|retired_jobs| {
                retired_jobs.borrow_mut().insert(id, retired);
            });
        }
    });
}

/// The host override's request half: queue a registration, return its id.
#[cfg(not(target_arch = "wasm32"))]
type RequestOverride = Box<dyn Fn(IntervalJob, u32) -> IntervalId>;

/// The host override's cancel half: drop a queued registration by id.
#[cfg(not(target_arch = "wasm32"))]
type CancelOverride = Box<dyn Fn(IntervalId)>;

/// Host-target dispatch: there are no browser globals to resolve, so tests install an override
/// ([`DISPATCH_OVERRIDE`]) — the analog of stubbing the ambient `setInterval`/`clearInterval`
/// globals, which host builds do not have.
#[cfg(not(target_arch = "wasm32"))]
fn set_interval(job: IntervalJob, delay: u32) -> IntervalId {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (request, _) = dispatch.as_ref().expect(
            "no interval dispatcher installed on the host target: host builds have no \
             setInterval/clearInterval globals, so tests must install a dispatch override",
        );
        request(job, delay)
    })
}

/// Host-target cancel — see [`set_interval`] for why the override exists.
#[cfg(not(target_arch = "wasm32"))]
fn clear_interval(id: IntervalId) {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (_, cancel) = dispatch.as_ref().expect(
            "no interval dispatcher installed on the host target: host builds have no \
             setInterval/clearInterval globals, so tests must install a dispatch override",
        );
        cancel(id)
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::collections::HashMap;

    use reactive_graph::owner::Owner;

    use super::*;

    /// The manual queue standing in for the browser's timer queue on the host target: every
    /// registration is queued with its delay, cancel removes it, and [`ManualQueue::flush`]
    /// fires each live registration's callback exactly once — the repeating semantics of
    /// `setInterval`, with the caller deciding how many ticks elapse.
    struct ManualQueue {
        intervals: Rc<RefCell<Vec<(IntervalId, Rc<RefCell<Option<IntervalJob>>>)>>>,
        delays: Rc<RefCell<HashMap<IntervalId, u32>>>,
    }

    impl ManualQueue {
        fn install() -> Self {
            let intervals: Rc<RefCell<Vec<(IntervalId, Rc<RefCell<Option<IntervalJob>>>)>>> =
                Rc::new(RefCell::new(Vec::new()));
            let delays: Rc<RefCell<HashMap<IntervalId, u32>>> =
                Rc::new(RefCell::new(HashMap::new()));
            let next_id = Cell::new(0u32);

            let request_intervals = Rc::clone(&intervals);
            let request_delays = Rc::clone(&delays);
            let request: RequestOverride = Box::new(move |job: IntervalJob, delay: u32| {
                next_id.set(next_id.get() + 1);
                let id = next_id.get();
                request_intervals
                    .borrow_mut()
                    .push((id, Rc::new(RefCell::new(Some(job)))));
                request_delays.borrow_mut().insert(id, delay);
                id
            });

            let cancel_intervals = Rc::clone(&intervals);
            let cancel_delays = Rc::clone(&delays);
            let cancel: CancelOverride = Box::new(move |id: IntervalId| {
                cancel_intervals
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                cancel_delays.borrow_mut().remove(&id);
            });

            DISPATCH_OVERRIDE.with(|dispatch| {
                *dispatch.borrow_mut() = Some((request, cancel));
            });
            Self { intervals, delays }
        }

        /// Fires one tick of every live registration, in scheduling order. A callback is taken
        /// out of its slot while it runs so the queue borrow is released — the callback may
        /// cancel or reschedule registrations — and put back afterwards unless its
        /// registration was canceled during its own invocation.
        fn flush(&self) {
            let ids: Vec<IntervalId> = self
                .intervals
                .borrow()
                .iter()
                .map(|(id, _)| *id)
                .collect();
            for id in ids {
                let callback = {
                    let intervals = self.intervals.borrow();
                    intervals
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .and_then(|(_, callback)| callback.borrow_mut().take())
                };
                let Some(mut callback) = callback else {
                    continue;
                };
                callback();
                let intervals = self.intervals.borrow();
                if let Some((_, callback_slot)) =
                    intervals.iter().find(|(queued, _)| *queued == id)
                {
                    let mut slot = callback_slot.borrow_mut();
                    assert!(
                        slot.is_none(),
                        "an interval callback slot was filled while empty"
                    );
                    *slot = Some(callback);
                }
            }
        }

        /// The delay a registration was queued with — pins that `start` forwards `delay`
        /// straight through to the host.
        fn delay_for(&self, id: IntervalId) -> Option<u32> {
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

    fn counting_callback() -> (Rc<Cell<u32>>, impl FnMut() + 'static) {
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        (fired, move || counter.set(counter.get() + 1))
    }

    // Pins the handle-construction contract (`packages/utils/src/useInterval.ts:11-13`): two
    // `create()` calls return distinct instances, each usable as its own scheduler.
    #[test]
    fn create_returns_a_distinct_instance_each_time() {
        let a: Interval = Interval::create();
        let b: Interval = Interval::create();
        assert!(
            !Rc::ptr_eq(&a.current_id, &b.current_id),
            "create() must return a new instance each time"
        );
    }

    // Pins the dispatch contract (`packages/utils/src/useInterval.ts:20-22`): the callback
    // does not run on `start` — the first invocation happens after one full delay — and the
    // interval repeats on every tick afterwards.
    #[test]
    fn start_does_not_fire_synchronously_and_fires_repeatedly() {
        let queue = ManualQueue::install();
        let interval = Interval::create();
        let (fired, callback) = counting_callback();

        interval.start(100, callback);
        assert_eq!(
            fired.get(),
            0,
            "the callback runs asynchronously after the first full delay, not synchronously"
        );

        queue.flush();
        assert_eq!(fired.get(), 1);

        queue.flush();
        assert_eq!(fired.get(), 2, "the interval repeats on every tick");
        assert!(
            interval.is_started(),
            "the id stays live across every tick — only clear() resets it"
        );
    }

    // Pins the inherited `isStarted` contract (`packages/utils/src/useTimeout.ts:27-29`) as
    // the spec defines it for a repeating timer (`specs/utils/useInterval.md`, "State model").
    #[test]
    fn is_started_tracks_the_pending_state() {
        let _queue = ManualQueue::install();
        let interval = Interval::create();
        assert!(!interval.is_started(), "a fresh instance is not started");

        let (_fired, callback) = counting_callback();
        interval.start(50, callback);
        assert!(interval.is_started(), "start() latches the pending id");

        interval.clear();
        assert!(!interval.is_started(), "clear() resets the id to EMPTY");
    }

    // Pins the replace-not-stack contract (`packages/utils/src/useInterval.ts:18-23`): a
    // second `start` clears the previously scheduled interval, so only the newest callback
    // fires — at most one live interval exists per instance.
    #[test]
    fn start_clears_any_previously_scheduled_interval() {
        let queue = ManualQueue::install();
        let interval = Interval::create();
        let (first_fired, first) = counting_callback();
        let (second_fired, second) = counting_callback();

        interval.start(100, first);
        interval.start(100, second);

        queue.flush();
        assert_eq!(second_fired.get(), 1);
        assert_eq!(first_fired.get(), 0, "the superseded callback never fires");

        queue.flush();
        assert_eq!(second_fired.get(), 2);
        assert_eq!(first_fired.get(), 0);
    }

    // Pins the cancel contract (`packages/utils/src/useInterval.ts:25-30`): a cleared
    // registration never fires.
    #[test]
    fn clear_cancels_a_pending_interval() {
        let queue = ManualQueue::install();
        let interval = Interval::create();
        let (fired, callback) = counting_callback();

        interval.start(100, callback);
        interval.clear();

        queue.flush();
        assert_eq!(fired.get(), 0);
    }

    // Pins the guard (`packages/utils/src/useInterval.ts:26-29`): clearing an idle or
    // already-cleared instance is a no-op — it neither throws nor calls `clearInterval(0)`.
    #[test]
    fn clear_on_an_idle_instance_is_a_no_op() {
        let queue = ManualQueue::install();
        let interval = Interval::create();

        interval.clear();
        interval.clear();
        assert!(!interval.is_started());
        assert!(interval.current_id().is_none());

        queue.flush();
    }

    // Pins the delay passthrough (`packages/utils/src/useInterval.ts:22`): `delay` is passed
    // straight to `setInterval` with no clamping or validation
    // (`specs/utils/useInterval.md`, "Edge cases").
    #[test]
    fn start_forwards_the_delay_unchanged() {
        let queue = ManualQueue::install();
        let interval = Interval::create();
        let (_fired, callback) = counting_callback();

        interval.start(250, callback);
        let id = interval
            .current_id()
            .expect("a started interval has a pending id");
        assert_eq!(queue.delay_for(id), Some(250));
    }

    // Pins the inherited `disposeEffect` contract (`packages/utils/src/useTimeout.ts:38-40`):
    // the returned cleanup cancels a pending interval.
    #[test]
    fn dispose_effect_cancels_a_pending_interval() {
        let queue = ManualQueue::install();
        let interval = Interval::create();
        let (fired, callback) = counting_callback();

        interval.start(100, callback);
        let dispose = interval.dispose_effect();
        dispose();

        queue.flush();
        assert_eq!(fired.get(), 0);
    }

    // Pins the self-clearing pattern (`specs/utils/useInterval.md`, "Edge cases"): a callback
    // that clears its own instance during a tick stops the interval cleanly, and the
    // instance's id slot is reset so it can be started again.
    #[test]
    fn a_callback_that_clears_itself_stops_firing() {
        let queue = ManualQueue::install();
        let interval = Interval::create();
        let fired = Rc::new(Cell::new(0u32));
        let handle = interval.clone();

        let counter = Rc::clone(&fired);
        interval.start(100, move || {
            counter.set(counter.get() + 1);
            if counter.get() == 2 {
                handle.clear();
            }
        });

        queue.flush();
        assert_eq!(fired.get(), 1);

        queue.flush();
        assert_eq!(fired.get(), 2);
        assert!(
            !interval.is_started(),
            "clearing from inside the callback cancels the interval"
        );

        queue.flush();
        assert_eq!(fired.get(), 2, "the interval does not fire after self-clearing");
    }

    // Pins per-instance scheduling (`specs/utils/useInterval.md`, "Edge cases"): each call
    // site gets its own independent `Interval` instance, so nested or sibling consumers do
    // not share scheduling state.
    #[test]
    fn two_instances_fire_independently() {
        let queue = ManualQueue::install();
        let first_interval = Interval::create();
        let second_interval = Interval::create();
        let (first_fired, first) = counting_callback();
        let (second_fired, second) = counting_callback();

        first_interval.start(100, first);
        second_interval.start(200, second);

        queue.flush();
        assert_eq!(first_fired.get(), 1);
        assert_eq!(second_fired.get(), 1);
    }

    // Pins the hook's unmount behavior (`packages/utils/src/useInterval.ts:39`): a pending
    // interval started via the hook's scheduler is cleared when the component unmounts —
    // reactive owner disposal plays the role of React unmount running the `useOnMount`
    // cleanup (`packages/utils/src/useOnMount.ts:8-12`).
    #[test]
    fn use_interval_cancels_the_pending_interval_when_the_reactive_owner_is_disposed() {
        let queue = ManualQueue::install();

        let owner = Owner::new();
        owner.set();
        let scheduler = use_interval();
        let (fired, callback) = counting_callback();
        scheduler.start(100, callback);

        owner.cleanup();
        queue.flush();
        assert_eq!(
            fired.get(),
            0,
            "the pending interval is cleared on unmount"
        );
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

    /// The global stub standing in for the browser's timer queue: `setInterval` queues every
    /// registration and returns a fresh numeric id, `clearInterval` removes it. Restores the
    /// originals on drop so a panicking test cannot poison the later ones.
    struct IntervalStub {
        original_set: JsValue,
        original_clear: JsValue,
        /// Kept alive for the stub's lifetime; the browser only sees the JS side.
        #[allow(dead_code)]
        set_closure: Closure<dyn FnMut(Function, JsValue) -> JsValue>,
        #[allow(dead_code)]
        clear_closure: Closure<dyn FnMut(JsValue)>,
        intervals: Rc<RefCell<Vec<(u32, Function)>>>,
        delays: Rc<RefCell<HashMap<u32, u32>>>,
    }

    impl IntervalStub {
        fn install() -> Self {
            let intervals: Rc<RefCell<Vec<(u32, Function)>>> = Rc::new(RefCell::new(Vec::new()));
            let delays: Rc<RefCell<HashMap<u32, u32>>> = Rc::new(RefCell::new(HashMap::new()));
            let next_id = Cell::new(0u32);

            let set_intervals = Rc::clone(&intervals);
            let set_delays = Rc::clone(&delays);
            let set_closure =
                Closure::wrap(Box::new(move |callback: Function, delay: JsValue| -> JsValue {
                    next_id.set(next_id.get() + 1);
                    let id = next_id.get();
                    set_intervals.borrow_mut().push((id, callback));
                    set_delays
                        .borrow_mut()
                        .insert(id, delay.as_f64().unwrap_or(0.0) as u32);
                    JsValue::from_f64(f64::from(id))
                })
                    as Box<dyn FnMut(Function, JsValue) -> JsValue>);

            let clear_intervals = Rc::clone(&intervals);
            let clear_delays = Rc::clone(&delays);
            let clear_closure = Closure::wrap(Box::new(move |id: JsValue| {
                let id = id.as_f64().unwrap_or(0.0) as u32;
                clear_intervals
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
                clear_delays.borrow_mut().remove(&id);
            }) as Box<dyn FnMut(JsValue)>);

            let global = js_sys::global();
            let set_key = JsValue::from_str("setInterval");
            let clear_key = JsValue::from_str("clearInterval");
            let original_set = Reflect::get(&global, &set_key)
                .expect("globalThis.setInterval must be readable");
            let original_clear = Reflect::get(&global, &clear_key)
                .expect("globalThis.clearInterval must be readable");
            set_global("setInterval", set_closure.as_ref().unchecked_ref());
            set_global("clearInterval", clear_closure.as_ref().unchecked_ref());

            Self {
                original_set,
                original_clear,
                set_closure,
                clear_closure,
                intervals,
                delays,
            }
        }

        /// Fires one tick of every live registration, in scheduling order. Registrations
        /// canceled during the flush (a self-clearing or self-rescheduling callback) are
        /// skipped by the snapshot loop.
        fn flush(&self) {
            let ids: Vec<u32> = self.intervals.borrow().iter().map(|(id, _)| *id).collect();
            for id in ids {
                let callback = {
                    let intervals = self.intervals.borrow();
                    intervals
                        .iter()
                        .find(|(queued, _)| *queued == id)
                        .map(|(_, callback)| callback.clone())
                };
                let Some(callback) = callback else {
                    continue;
                };
                callback
                    .call0(&JsValue::UNDEFINED)
                    .expect("the interval callback threw");
            }
        }

        /// The delay a registration was queued with — pins that `start` forwards `delay`
        /// straight through to the host.
        fn delay_for(&self, id: u32) -> Option<u32> {
            self.delays.borrow().get(&id).copied()
        }
    }

    impl Drop for IntervalStub {
        fn drop(&mut self) {
            set_global("setInterval", &self.original_set);
            set_global("clearInterval", &self.original_clear);
        }
    }

    // Pins the dispatch contract (`packages/utils/src/useInterval.ts:20-22`) through the
    // ambient globals: asynchronous first tick, repeating delivery, and an id that stays live
    // across every tick.
    #[wasm_bindgen_test]
    fn fires_repeatedly_through_the_ambient_globals() {
        let stub = IntervalStub::install();
        let interval = Interval::create();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);

        interval.start(100, move || counter.set(counter.get() + 1));
        assert_eq!(
            fired.get(),
            0,
            "the callback runs asynchronously after the first full delay, not synchronously"
        );

        stub.flush();
        assert_eq!(fired.get(), 1);
        assert!(interval.is_started());

        stub.flush();
        assert_eq!(fired.get(), 2, "the interval repeats on every tick");
        assert!(interval.is_started());

        drop(stub);
    }

    // Pins the delay passthrough and the replace-not-stack contract
    // (`packages/utils/src/useInterval.ts:18-23`): `delay` reaches the host unchanged and a
    // second `start` supersedes the first.
    #[wasm_bindgen_test]
    fn clear_cancels_and_a_second_start_supersedes_through_the_ambient_globals() {
        let stub = IntervalStub::install();
        let interval = Interval::create();

        let canceled = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&canceled);
        interval.start(100, move || counter.set(counter.get() + 1));
        let id = interval.current_id().expect("a started interval has an id");
        assert_eq!(stub.delay_for(id), Some(100));

        interval.clear();
        assert!(!interval.is_started());

        let superseding = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&superseding);
        interval.start(200, move || counter.set(counter.get() + 1));
        let id = interval.current_id().expect("a started interval has an id");
        assert_eq!(stub.delay_for(id), Some(200));

        stub.flush();
        assert_eq!(canceled.get(), 0);
        assert_eq!(superseding.get(), 1);

        drop(stub);
    }

    // Pins the closure-lifetime machinery for the self-clearing pattern: a callback that
    // clears its own instance during its own invocation retires its wrapper while that
    // wrapper is still on the JS stack — the sweep must defer the drop instead of panicking.
    #[wasm_bindgen_test]
    fn a_callback_that_clears_itself_during_its_own_invocation_stops_cleanly() {
        let stub = IntervalStub::install();
        let interval = Interval::create();
        let fired = Rc::new(Cell::new(0u32));
        let handle = interval.clone();

        let counter = Rc::clone(&fired);
        interval.start(100, move || {
            counter.set(counter.get() + 1);
            if counter.get() == 2 {
                handle.clear();
            }
        });

        stub.flush();
        assert_eq!(fired.get(), 1);

        stub.flush();
        assert_eq!(fired.get(), 2);
        assert!(!interval.is_started());

        stub.flush();
        assert_eq!(fired.get(), 2, "the interval does not fire after self-clearing");

        // The instance's id slot is reset, so it can be started again.
        let counter = Rc::clone(&fired);
        interval.start(100, move || counter.set(counter.get() + 1));
        stub.flush();
        assert_eq!(fired.get(), 3);

        drop(stub);
    }

    // Pins the closure-lifetime machinery for the self-rescheduling pattern: a callback that
    // re-`start`s its own instance during its own invocation both clears and schedules from
    // inside the wrapper — the sweep between the two must not drop the still-executing
    // wrapper.
    #[wasm_bindgen_test]
    fn a_callback_that_restarts_the_instance_during_its_own_invocation_supersedes_itself() {
        let stub = IntervalStub::install();
        let interval = Interval::create();
        let first_fired = Rc::new(Cell::new(0u32));
        let second_fired = Rc::new(Cell::new(0u32));
        let handle = interval.clone();

        let first_counter = Rc::clone(&first_fired);
        let second_counter = Rc::clone(&second_fired);
        interval.start(100, move || {
            first_counter.set(first_counter.get() + 1);
            let second_counter = Rc::clone(&second_counter);
            handle.start(100, move || second_counter.set(second_counter.get() + 1));
        });

        stub.flush();
        assert_eq!(first_fired.get(), 1);
        assert_eq!(second_fired.get(), 0, "the rescheduled callback waits a full delay");

        stub.flush();
        assert_eq!(first_fired.get(), 1, "the superseded callback never fires again");
        assert_eq!(second_fired.get(), 1);

        drop(stub);
    }

    // Pins the hook's unmount behavior (`packages/utils/src/useInterval.ts:39`): a pending
    // interval started via the hook's scheduler is cleared when the component unmounts —
    // reactive owner disposal plays the role of React unmount.
    #[wasm_bindgen_test]
    fn use_interval_cancels_the_pending_interval_when_the_reactive_owner_is_disposed() {
        let stub = IntervalStub::install();

        let owner = Owner::new();
        owner.set();
        let scheduler = use_interval();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        scheduler.start(100, move || counter.set(counter.get() + 1));

        owner.cleanup();
        stub.flush();
        assert_eq!(
            fired.get(),
            0,
            "the pending interval is cleared on unmount"
        );

        drop(stub);
    }
}
