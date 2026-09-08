//! Port of `packages/utils/src/useAnimationFrame.ts` (Base UI Phase A util).
//!
//! Upstream couples three pieces of state:
//!
//! - A process-global scheduler coalescing every frame request made before the next native
//!   animation frame into a single `requestAnimationFrame` registration
//!   (`packages/utils/src/useAnimationFrame.ts:15-84`). It uses an array as the backing
//!   structure so canceling is `O(1)` slot-nulling and never touches the native
//!   `cancelAnimationFrame` (`packages/utils/src/useAnimationFrame.ts:16-22`), with a live-callback
//!   count turning a leftover frame full of canceled entries into an `O(1)` no-op
//!   (`packages/utils/src/useAnimationFrame.ts:45-49`).
//! - An instance handle keeping at most one frame pending, clearing any previously scheduled
//!   call, and canceling the pending frame on unmount
//!   (`packages/utils/src/useAnimationFrame.ts:109-156`).
//! - A `resetAnimationFrameScheduler` escape hatch for test environments, replacing the shared
//!   scheduler while continuing its id sequence and emptying the previous queue in place so a
//!   still-pending native frame runs nothing
//!   (`packages/utils/src/useAnimationFrame.ts:96-107`).
//!
//! The proven contract comes from the suite's single test: after canceling the same frame id
//! twice, a later-registered callback still fires exactly once when the pending frame runs
//! (`packages/utils/src/useAnimationFrame.test.ts:14-31`). Everything else this module implements
//! (canceling actually drops the callback, batched dispatch with timestamp forwarding,
//! coalescing, reset semantics, instance lifecycle) is UNVERIFIED upstream per
//! `specs/utils/useAnimationFrame.md` and inferred from the source lines cited below.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The `EMPTY = null` sentinel (`packages/utils/src/useAnimationFrame.ts:7-11`) becomes
//!   [`Option`]: upstream needed `null` because the instance field doubles as an id slot, and
//!   the port never stores the native `requestAnimationFrame` return value at all (the scheduler
//!   deliberately never calls the native `cancelAnimationFrame`), so a plain `usize` id starting
//!   at 1 has no empty-sentinel problem.
//! - Rust forbids an associated function and a method sharing a name in one inherent impl, so
//!   upstream's static/instance overloading of `request`/`cancel`
//!   (`packages/utils/src/useAnimationFrame.ts:114-120` and `:127-140`) splits: the static form
//!   becomes the module-level [`request_animation_frame`]/[`cancel_animation_frame`] functions,
//!   while the instance keeps the upstream [`AnimationFrame::request`]/[`AnimationFrame::cancel`]
//!   names.
//! - Queued callbacks are [`Box<dyn FnOnce(f64)>`]: the batch is moved out of the queue before
//!   dispatch (`packages/utils/src/useAnimationFrame.ts:37-43`), so every entry fires at most
//!   once. Canceled entries are nulled slots, skipped by the dispatch like upstream's
//!   `currentCallbacks[i]?.(timestamp)` (`packages/utils/src/useAnimationFrame.ts:46-48`).
//! - JS garbage-collects the bound `tick` method; the port must keep the native registration's
//!   closure alive itself. Each registration stores its tick closure in the scheduler until the
//!   frame fires, then retires it and drops it on the next request — a fired closure can never be
//!   invoked again, so deferring the drop past its own invocation keeps wasm-bindgen's
//!   no-drop-during-invocation rule intact.
//! - The dev-only re-registration when the global `requestAnimationFrame` implementation changes
//!   while a frame is pending (`packages/utils/src/useAnimationFrame.ts:58-69`) keeps its
//!   `process.env.NODE_ENV !== 'production'` gating as `cfg!(debug_assertions)`, matching the
//!   dev-warning gating used in the crate's `react_store` port.
//! - The `useAnimationFrame` hook
//!   (`packages/utils/src/useAnimationFrame.ts:150-156`) dissolves React's
//!   `useRefWithInit`/`useOnMount` plumbing into Leptos owner semantics: a component's setup body
//!   runs once, so the local binding is the stable instance, and [`on_cleanup`] plays the role of
//!   the mount effect's returned cleanup. Like the upstream hook, it must be called inside a
//!   reactive owner (a component). The hook is UNVERIFIED upstream (no test asserts it) — same as
//!   [`AnimationFrame::request`]'s at-most-one-pending semantics.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use send_wrapper::SendWrapper;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

/// The scheduler-generated frame id — upstream `AnimationFrameId`
/// (`packages/utils/src/useAnimationFrame.ts:5`), produced by the scheduler's `nextId` counter
/// (`packages/utils/src/useAnimationFrame.ts:28`). Not the native `requestAnimationFrame` return
/// value, which this port discards exactly like upstream.
pub type AnimationFrameId = usize;

type FrameCallback = Box<dyn FnOnce(f64)>;

thread_local! {
    static SCHEDULER: RefCell<Rc<RefCell<Scheduler>>> =
        RefCell::new(Rc::new(RefCell::new(Scheduler::new())));
    /// The last-seen global `requestAnimationFrame`, for the dev-only fake-timer teardown guard
    /// (`packages/utils/src/useAnimationFrame.ts:13`).
    #[cfg(target_arch = "wasm32")]
    static LAST_RAF: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}

struct Scheduler {
    /// Backing array of frame callbacks; canceled entries are nulled in place for `O(1)` cancel
    /// (`packages/utils/src/useAnimationFrame.ts:24`).
    callbacks: Vec<Option<FrameCallback>>,

    /// How many queued entries are still live
    /// (`packages/utils/src/useAnimationFrame.ts:26`).
    callbacks_count: usize,

    next_id: AnimationFrameId,

    start_id: AnimationFrameId,

    is_scheduled: bool,

    /// Native frame registrations whose tick closure must stay alive until the frame fires;
    /// matched by generation, removed once fired (see the module docs on closure lifetime).
    #[cfg(target_arch = "wasm32")]
    live_ticks: Vec<(u64, Closure<dyn Fn(f64)>)>,

    /// Fired tick closures, dropped on the next request — never during their own invocation.
    #[cfg(target_arch = "wasm32")]
    retired_ticks: Vec<Closure<dyn Fn(f64)>>,

    #[cfg(target_arch = "wasm32")]
    next_tick_generation: u64,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            callbacks_count: 0,
            next_id: 1,
            start_id: 1,
            is_scheduled: false,
            #[cfg(target_arch = "wasm32")]
            live_ticks: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            retired_ticks: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            next_tick_generation: 0,
        }
    }

    /// Upstream `Scheduler.request`
    /// (`packages/utils/src/useAnimationFrame.ts:52-71`). Takes the scheduler handle because
    /// registering a native frame stores a tick closure bound to this exact scheduler instance —
    /// a reset may replace the global in between, and the pending frame must keep running the
    /// scheduler it was requested on.
    fn request(rc: &Rc<RefCell<Self>>, callback: FrameCallback) -> AnimationFrameId {
        let mut this = rc.borrow_mut();

        // Drop tick closures whose frames already fired: they can no longer be invoked by the
        // browser (their tick ran to completion before retiring).
        #[cfg(target_arch = "wasm32")]
        this.retired_ticks.clear();

        let id = this.next_id;
        this.next_id += 1;
        this.callbacks.push(Some(callback));
        this.callbacks_count += 1;

        let did_raf_change = check_last_raf_changed();
        if !this.is_scheduled || did_raf_change {
            #[cfg(target_arch = "wasm32")]
            {
                let generation = this.next_tick_generation;
                this.next_tick_generation += 1;
                let tick_rc = Rc::clone(rc);
                let tick = Closure::wrap(Box::new(move |timestamp: f64| {
                    // Update the scheduling state before iterating — callbacks may request new
                    // frames (`packages/utils/src/useAnimationFrame.ts:40-43`).
                    let batch = { tick_rc.borrow_mut().tick_begin() };
                    for callback in batch {
                        callback(timestamp);
                    }
                    tick_rc.borrow_mut().tick_end(generation);
                }) as Box<dyn Fn(f64)>);
                call_global_request_animation_frame(tick.as_ref().unchecked_ref());
                this.live_ticks.push((generation, tick));
                this.is_scheduled = true;
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Host builds (native `cargo test`) have no frame loop; tests play the role of
                // the browser by driving `tick_begin` directly.
                this.is_scheduled = true;
            }
        }
        id
    }

    /// Upstream `Scheduler.cancel` (`packages/utils/src/useAnimationFrame.ts:73-83`): an id
    /// outside the current queue window (already delivered by a previous frame) or already
    /// canceled is a no-op, so canceling twice decrements the live count only once.
    fn cancel(&mut self, id: AnimationFrameId) {
        if id < self.start_id {
            return;
        }
        let index = id - self.start_id;
        if index >= self.callbacks.len() {
            return;
        }
        if self.callbacks[index].is_none() {
            return;
        }
        self.callbacks[index] = None;
        self.callbacks_count -= 1;
    }

    /// The first half of upstream's `tick` (`packages/utils/src/useAnimationFrame.ts:34-43`):
    /// clears the scheduled flag and swaps the queue out before dispatching, so re-entrant
    /// requests during the batch land in a fresh window with `startId` advanced past the ids
    /// being delivered.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn tick_begin(&mut self) -> Vec<FrameCallback> {
        self.is_scheduled = false;

        let current_callbacks = std::mem::take(&mut self.callbacks);
        let current_callbacks_count = self.callbacks_count;
        self.callbacks_count = 0;
        self.start_id = self.next_id;

        if current_callbacks_count > 0 {
            current_callbacks.into_iter().flatten().collect()
        } else {
            Vec::new()
        }
    }

    /// Retires the tick closure whose frame just fired. Runs after the batch — never from within
    /// the closure's own invocation — so the drop is deferred to the next request.
    #[cfg(target_arch = "wasm32")]
    fn tick_end(&mut self, generation: u64) {
        if let Some(position) = self
            .live_ticks
            .iter()
            .position(|(tick_generation, _)| *tick_generation == generation)
        {
            let (_, retired) = self.live_ticks.swap_remove(position);
            self.retired_ticks.push(retired);
        }
    }
}

/// Reads the current global `requestAnimationFrame` and reports whether it changed since the
/// previous observation — upstream's `didRAFChange` dev guard
/// (`packages/utils/src/useAnimationFrame.ts:58-69`), re-registering a frame when a fake
/// `requestAnimationFrame` is swapped in while one is already pending.
#[cfg(target_arch = "wasm32")]
fn check_last_raf_changed() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    let current = get_global_request_animation_frame();
    LAST_RAF.with(|last| {
        let changed = last
            .borrow()
            .as_ref()
            .is_some_and(|previous| *previous != current);
        *last.borrow_mut() = Some(current);
        changed
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn check_last_raf_changed() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn get_global_request_animation_frame() -> JsValue {
    js_sys::Reflect::get(
        &js_sys::global(),
        &JsValue::from_str("requestAnimationFrame"),
    )
    .expect("globalThis.requestAnimationFrame must be readable")
}

/// Dispatches through `globalThis.requestAnimationFrame`, discarding the native id — the
/// scheduler never cancels natively (`packages/utils/src/useAnimationFrame.ts:16-22`).
#[cfg(target_arch = "wasm32")]
fn call_global_request_animation_frame(callback: &js_sys::Function) {
    let global = js_sys::global();
    let request_animation_frame: js_sys::Function = get_global_request_animation_frame()
        .dyn_into()
        .unwrap_or_else(|error| {
            panic!("globalThis.requestAnimationFrame is not a function: {error:?}")
        });
    request_animation_frame
        .call1(&global, callback)
        .expect("requestAnimationFrame threw while scheduling a frame");
}

fn schedule_frame(callback: FrameCallback) -> AnimationFrameId {
    SCHEDULER.with(|scheduler| {
        let scheduler = Rc::clone(&scheduler.borrow());
        Scheduler::request(&scheduler, callback)
    })
}

/// Requests a frame callback on the process-global scheduler — upstream's static
/// `AnimationFrame.request(fn)`
/// (`packages/utils/src/useAnimationFrame.ts:114-116`). Returns an id usable with
/// [`cancel_animation_frame`].
pub fn request_animation_frame(callback: impl FnMut(f64) + 'static) -> AnimationFrameId {
    let mut callback = callback;
    schedule_frame(Box::new(move |timestamp| callback(timestamp)))
}

/// Cancels a previously requested frame by id — upstream's static `AnimationFrame.cancel(id)`
/// (`packages/utils/src/useAnimationFrame.ts:118-120`). Canceling twice is a no-op
/// (`packages/utils/src/useAnimationFrame.test.ts:22-23`).
pub fn cancel_animation_frame(id: AnimationFrameId) {
    SCHEDULER.with(|scheduler| scheduler.borrow().borrow_mut().cancel(id));
}

/// Replaces the shared scheduler and drops all pending animation frame callbacks — upstream
/// `resetAnimationFrameScheduler` (`packages/utils/src/useAnimationFrame.ts:96-107`). The id
/// sequence continues, so cancel calls made before the reset cannot cancel callbacks scheduled
/// after it, and a native frame still pending from before the reset empties its queue in place so
/// it runs nothing when it eventually fires.
///
/// For test environments only, mirroring upstream's suite, which calls it between every test
/// (`packages/utils/src/useAnimationFrame.test.ts:5-12`).
pub fn reset_animation_frame_scheduler() {
    SCHEDULER.with(|scheduler| {
        let previous = Rc::clone(&scheduler.borrow());
        let next_id = previous.borrow().next_id;
        *scheduler.borrow_mut() = Rc::new(RefCell::new(Scheduler {
            next_id,
            start_id: next_id,
            ..Scheduler::new()
        }));
        let mut old = previous.borrow_mut();
        old.callbacks = Vec::new();
        old.callbacks_count = 0;
    });
}

/// A handle keeping at most one frame pending — upstream's `AnimationFrame` instance API
/// (`packages/utils/src/useAnimationFrame.ts:109-145`). Cheap to clone; clones share the same
/// pending-frame slot.
#[derive(Clone, Default)]
pub struct AnimationFrame {
    /// Upstream `currentId` (`packages/utils/src/useAnimationFrame.ts:122`); `None` is upstream's
    /// `EMPTY` sentinel.
    current_id: Rc<Cell<Option<AnimationFrameId>>>,
}

impl AnimationFrame {
    /// Upstream `static create()` (`packages/utils/src/useAnimationFrame.ts:110-112`).
    pub fn create() -> Self {
        Self::default()
    }

    /// The currently pending frame id, if any — upstream's public `currentId` field
    /// (`packages/utils/src/useAnimationFrame.ts:122`).
    pub fn current_id(&self) -> Option<AnimationFrameId> {
        self.current_id.get()
    }

    /// Executes `callback` on the next animation frame, clearing any previously scheduled call
    /// (`packages/utils/src/useAnimationFrame.ts:124-133`). The wrapper clears the current id
    /// before invoking, so `callback` may request a new frame on the same handle.
    pub fn request(&self, callback: impl FnOnce() + 'static) {
        self.cancel();
        let current_id = Rc::clone(&self.current_id);
        let id = schedule_frame(Box::new(move |_timestamp| {
            current_id.set(None);
            callback();
        }));
        self.current_id.set(Some(id));
    }

    /// Cancels the pending frame, if any
    /// (`packages/utils/src/useAnimationFrame.ts:135-140`).
    pub fn cancel(&self) {
        if let Some(id) = self.current_id.take() {
            cancel_animation_frame(id);
        }
    }

    /// Returns the cancel function as a cleanup — upstream `disposeEffect`
    /// (`packages/utils/src/useAnimationFrame.ts:142-144`), consumed by `useOnMount` in the
    /// upstream hook. [`use_animation_frame`] wires it to [`on_cleanup`] directly; exposed for
    /// callers building their own mount-effect plumbing.
    pub fn dispose_effect(&self) -> impl Fn() + 'static {
        let frame = self.clone();
        move || frame.cancel()
    }
}

/// A `requestAnimationFrame` with automatic cleanup and guard — upstream's
/// `useAnimationFrame()` hook (`packages/utils/src/useAnimationFrame.ts:150-156`): creates the
/// instance (a Leptos component body runs once, so the binding is the stable instance the way
/// upstream's `useRefWithInit` memoizes it) and cancels any pending frame when the owning
/// reactive scope is disposed (upstream registers the instance's `disposeEffect` through
/// `useOnMount`). UNVERIFIED upstream — no test asserts the hook; must be called inside a
/// reactive owner.
pub fn use_animation_frame() -> AnimationFrame {
    let frame = AnimationFrame::create();
    let cleanup = SendWrapper::new(frame.dispose_effect());
    on_cleanup(move || (*cleanup)());
    frame
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    /// Plays the role of the browser on the host target: swaps the pending queue out and
    /// delivers it, the way a fired native frame would
    /// (`packages/utils/src/useAnimationFrame.test.ts:28`).
    fn fire_pending_frame(timestamp: f64) -> usize {
        let batch = SCHEDULER.with(|scheduler| scheduler.borrow().borrow_mut().tick_begin());
        let count = batch.len();
        for callback in batch {
            callback(timestamp);
        }
        count
    }

    // Mirrors `packages/utils/src/useAnimationFrame.test.ts:14-31` (the suite's only test) at the
    // accounting level: after canceling the same id twice, a later-registered callback still
    // fires exactly once when the pending frame runs — the double cancel decremented the live
    // count only once. The wasm suite re-runs the same contract through a mocked
    // `globalThis.requestAnimationFrame`.
    #[test]
    fn keeps_callback_accounting_correct_when_a_frame_is_canceled_more_than_once() {
        reset_animation_frame_scheduler();

        let canceled_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&canceled_fired);
        let first = request_animation_frame(move |_| counter.set(counter.get() + 1));
        cancel_animation_frame(first);
        cancel_animation_frame(first);

        let second_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&second_fired);
        request_animation_frame(move |_| counter.set(counter.get() + 1));

        assert_eq!(
            fire_pending_frame(0.0),
            1,
            "only the live callback remains in the batch"
        );
        assert_eq!(canceled_fired.get(), 0);
        assert_eq!(second_fired.get(), 1);

        reset_animation_frame_scheduler();
    }

    // Pins the reset contract (`packages/utils/src/useAnimationFrame.ts:96-107`): pending
    // callbacks are dropped, the id sequence continues, and a stale pre-reset cancel cannot
    // cancel a post-reset callback.
    #[test]
    fn reset_drops_pending_callbacks_and_continues_the_id_sequence() {
        reset_animation_frame_scheduler();

        let first_id = request_animation_frame(|_| {
            panic!("pending callback survived the reset");
        });
        reset_animation_frame_scheduler();

        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        let second_id = request_animation_frame(move |_| counter.set(counter.get() + 1));
        assert!(second_id > first_id, "the id sequence continues across resets");

        cancel_animation_frame(first_id);
        assert_eq!(fire_pending_frame(0.0), 1);
        assert_eq!(fired.get(), 1);

        reset_animation_frame_scheduler();
    }

    // Pins the queue-window scoping of cancel (`packages/utils/src/useAnimationFrame.ts:73-77`):
    // an id below the advanced startId is a no-op that neither panics nor corrupts the live
    // accounting of the next window.
    #[test]
    fn cancel_is_scoped_to_the_current_queue_window() {
        reset_animation_frame_scheduler();

        let first_id = request_animation_frame(|_| ());
        assert_eq!(fire_pending_frame(0.0), 1);
        cancel_animation_frame(first_id);

        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        let second_id = request_animation_frame(move |_| counter.set(counter.get() + 1));
        cancel_animation_frame(second_id);
        cancel_animation_frame(second_id);

        assert_eq!(fire_pending_frame(0.0), 0);
        assert_eq!(fired.get(), 0);

        reset_animation_frame_scheduler();
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use js_sys::{Function, Reflect};
    use reactive_graph::owner::Owner;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Installs a spy over `globalThis.requestAnimationFrame` that records every frame function
    /// instead of handing it to the browser — upstream's
    /// `vi.spyOn(globalThis, 'requestAnimationFrame')` mock
    /// (`packages/utils/src/useAnimationFrame.test.ts:16-19`). Restores the original on drop so a
    /// panicking test cannot poison the later ones (upstream relies on `afterEach`,
    /// `packages/utils/src/useAnimationFrame.test.ts:9-11`).
    struct RafSpy {
        original: JsValue,
        /// Kept alive for the spy's lifetime; the browser only sees the JS side of it.
        #[allow(dead_code)]
        spy: Closure<dyn FnMut(Function) -> JsValue>,
        captured: Rc<RefCell<Vec<Function>>>,
    }

    impl RafSpy {
        fn install() -> Self {
            let captured: Rc<RefCell<Vec<Function>>> = Rc::new(RefCell::new(Vec::new()));
            let sink = Rc::clone(&captured);
            let spy = Closure::wrap(Box::new(move |callback: Function| -> JsValue {
                sink.borrow_mut().push(callback);
                JsValue::from(sink.borrow().len() as u32)
            }) as Box<dyn FnMut(Function) -> JsValue>);

            let global = js_sys::global();
            let key = JsValue::from_str("requestAnimationFrame");
            let original =
                Reflect::get(&global, &key).expect("globalThis.requestAnimationFrame missing");
            Reflect::set(&global, &key, spy.as_ref().unchecked_ref())
                .expect("failed to install the requestAnimationFrame spy");
            Self {
                original,
                spy,
                captured,
            }
        }

        fn captured_count(&self) -> usize {
            self.captured.borrow().len()
        }

        /// Fires one captured frame function with the given timestamp, the way the browser would
        /// (`packages/utils/src/useAnimationFrame.test.ts:28`).
        fn fire(&self, index: usize, timestamp: f64) {
            let frame = self.captured.borrow()[index].clone();
            frame
                .call1(&JsValue::UNDEFINED, &JsValue::from_f64(timestamp))
                .expect("frame function threw");
        }
    }

    impl Drop for RafSpy {
        fn drop(&mut self) {
            Reflect::set(
                &js_sys::global(),
                &JsValue::from_str("requestAnimationFrame"),
                &self.original,
            )
            .expect("failed to restore requestAnimationFrame");
        }
    }

    // Mirrors `packages/utils/src/useAnimationFrame.test.ts:14-31` one to one: the spy captures
    // the scheduler's internal frame function, the first request is canceled twice, and the
    // frame fired for the first request still delivers the later-registered callback exactly
    // once.
    #[wasm_bindgen_test]
    fn keeps_callback_accounting_correct_when_a_frame_is_canceled_more_than_once() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let first_id = request_animation_frame(|_| ());
        cancel_animation_frame(first_id);
        cancel_animation_frame(first_id);

        let second_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&second_fired);
        request_animation_frame(move |_| counter.set(counter.get() + 1));

        spy.fire(0, 0.0);
        assert_eq!(second_fired.get(), 1);

        drop(spy);
        reset_animation_frame_scheduler();
    }

    // Pins the half the upstream test leaves to a no-op: the canceled callback itself never
    // runs (`packages/utils/src/useAnimationFrame.ts:73-83`, UNVERIFIED upstream).
    #[wasm_bindgen_test]
    fn a_canceled_callback_never_fires() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let canceled_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&canceled_fired);
        let canceled_id = request_animation_frame(move |_| counter.set(counter.get() + 1));
        cancel_animation_frame(canceled_id);

        let second_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&second_fired);
        request_animation_frame(move |_| counter.set(counter.get() + 1));

        spy.fire(0, 0.0);
        assert_eq!(canceled_fired.get(), 0);
        assert_eq!(second_fired.get(), 1);

        drop(spy);
        reset_animation_frame_scheduler();
    }

    // Pins coalescing and batched dispatch with timestamp forwarding
    // (`packages/utils/src/useAnimationFrame.ts:34-49` and `:66-69`, UNVERIFIED upstream):
    // requests made while a frame is pending share one native registration, and the native
    // timestamp reaches every live callback.
    #[wasm_bindgen_test]
    fn requests_made_while_a_frame_is_pending_share_a_single_native_registration() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let stamps: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
        let first_sink = Rc::clone(&stamps);
        request_animation_frame(move |timestamp| first_sink.borrow_mut().push(timestamp));
        let second_sink = Rc::clone(&stamps);
        request_animation_frame(move |timestamp| second_sink.borrow_mut().push(timestamp));

        assert_eq!(spy.captured_count(), 1);

        spy.fire(0, 1234.0);
        assert_eq!(*stamps.borrow(), [1234.0, 1234.0]);

        drop(spy);
        reset_animation_frame_scheduler();
    }

    // Pins the reset contract through the native-dispatch layer
    // (`packages/utils/src/useAnimationFrame.ts:96-107`): the pre-reset native frame runs
    // nothing, the reset scheduler registers a fresh frame, the id sequence continues, and a
    // stale pre-reset cancel cannot cancel the post-reset callback.
    #[wasm_bindgen_test]
    fn reset_drops_pending_callbacks_and_the_old_native_frame_runs_nothing() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let pre_reset_id = request_animation_frame(|_| {
            panic!("pre-reset callback survived the reset");
        });
        reset_animation_frame_scheduler();

        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        let post_reset_id = request_animation_frame(move |_| counter.set(counter.get() + 1));
        assert!(post_reset_id > pre_reset_id);
        assert_eq!(spy.captured_count(), 2);

        cancel_animation_frame(pre_reset_id);
        spy.fire(0, 0.0);
        assert_eq!(fired.get(), 0);

        spy.fire(1, 0.0);
        assert_eq!(fired.get(), 1);

        drop(spy);
        reset_animation_frame_scheduler();
    }

    // Pins the instance contract (`packages/utils/src/useAnimationFrame.ts:124-133`, UNVERIFIED
    // upstream): re-requesting on the same handle clears the previously scheduled call, keeps
    // the already-pending native frame, and clears `currentId` when the frame runs.
    #[wasm_bindgen_test]
    fn instance_request_clears_any_previously_scheduled_call() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let frame = AnimationFrame::create();
        let first_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&first_fired);
        frame.request(move || counter.set(counter.get() + 1));
        let second_fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&second_fired);
        frame.request(move || counter.set(counter.get() + 1));

        assert_eq!(spy.captured_count(), 1);
        assert!(frame.current_id().is_some());

        spy.fire(0, 0.0);
        assert_eq!(first_fired.get(), 0);
        assert_eq!(second_fired.get(), 1);
        assert!(frame.current_id().is_none());

        drop(spy);
        reset_animation_frame_scheduler();
    }

    // Pins instance cancel (`packages/utils/src/useAnimationFrame.ts:135-140`, UNVERIFIED
    // upstream): the pending frame is dropped and `currentId` clears immediately.
    #[wasm_bindgen_test]
    fn instance_cancel_drops_the_pending_callback() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let frame = AnimationFrame::create();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        frame.request(move || counter.set(counter.get() + 1));
        frame.cancel();
        assert!(frame.current_id().is_none());

        spy.fire(0, 0.0);
        assert_eq!(fired.get(), 0);

        drop(spy);
        reset_animation_frame_scheduler();
    }

    // Pins the hook's unmount behavior (`packages/utils/src/useAnimationFrame.ts:150-156`,
    // UNVERIFIED upstream): disposing the reactive owner cancels the pending frame, playing the
    // role of React unmount running the `useOnMount` cleanup.
    #[wasm_bindgen_test]
    fn use_animation_frame_cancels_the_pending_frame_when_the_reactive_owner_is_disposed() {
        reset_animation_frame_scheduler();
        let spy = RafSpy::install();

        let owner = Owner::new();
        owner.set();
        let frame = use_animation_frame();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        frame.request(move || counter.set(counter.get() + 1));

        owner.cleanup();
        spy.fire(0, 0.0);
        assert_eq!(fired.get(), 0);

        drop(spy);
        reset_animation_frame_scheduler();
    }
}
