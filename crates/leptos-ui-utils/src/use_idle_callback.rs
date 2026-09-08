//! Port of `packages/utils/src/useIdleCallback.ts` (Base UI Phase A util).
//!
//! Upstream is a two-layer scheduling utility:
//!
//! - The `IdleCallback` class: an instance handle keeping at most one callback pending. `start`
//!   clears any previously scheduled call and schedules a wrapper that resets `currentId` to
//!   `null` before invoking the user callback (`packages/utils/src/useIdleCallback.ts:28-34`),
//!   `clear` cancels the pending callback and nulls the id
//!   (`packages/utils/src/useIdleCallback.ts:36-41`), and `disposeEffect` hands `clear` back by
//!   identity for effect-cleanup use (`packages/utils/src/useIdleCallback.ts:43-45`).
//! - The `useIdleCallback` hook: creates the instance once via `useRefWithInit` and registers
//!   `disposeEffect` as a mount-effect cleanup via `useOnMount`
//!   (`packages/utils/src/useIdleCallback.ts:56-62`).
//!
//! Dispatch goes through the ambient `requestIdleCallback` when the environment has one and
//! degrades to a `setTimeout(fn, 0)` macrotask otherwise
//! (`packages/utils/src/useIdleCallback.ts:10-13`) — a macrotask runs after the current task
//! (and thus the current commit) but, unlike `requestIdleCallback`, is not guaranteed to run
//! after the next paint (`packages/utils/src/useIdleCallback.ts:7-9`). Both dispatch branches
//! are proven upstream: the main suite stubs the globals with a Map-backed manual queue
//! (`packages/utils/src/useIdleCallback.test.tsx:25-32`) and the fallback suite removes both
//! globals and drives fake timers (`packages/utils/src/useIdleCallback.fallback.test.ts:7-13`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Upstream captures `supportsIdleCallback` once at module load
//!   (`packages/utils/src/useIdleCallback.ts:5`). Whether the capability is captured at import
//!   time or resolved per call is UNVERIFIED upstream — no test distinguishes the two
//!   (`specs/utils/useIdleCallback.md`, "DOM structure & portal behavior"). The port resolves
//!   the ambient globals per call, matching the crate's `use_animation_frame` precedent of
//!   reading ambient browser globals at dispatch time. This is also what lets one test binary
//!   exercise both dispatch branches: upstream needed two separate test files precisely because
//!   the module captures the capability at import time.
//! - `currentId: number | null` (`packages/utils/src/useIdleCallback.ts:20`) becomes a
//!   clone-shared `Rc<Cell<Option<IdleCallbackId>>>`, the same shape as the `AnimationFrame`
//!   port's `currentId`: upstream relies on the id slot being observable (`currentId` is read by
//!   the suite at `packages/utils/src/useIdleCallback.test.tsx:98`) and clones of the handle
//!   share the slot the way JS callers share the instance.
//! - JS garbage-collects the bound dispatch wrapper; the port must keep each registration's
//!   closure alive itself. Pending wrappers live in a registry keyed by the id the ambient
//!   global returned; a fired wrapper moves itself to a retired list *after* the job completes,
//!   so the drop — deferred to the next entry into the module — never happens during the
//!   closure's own invocation (the same wasm-bindgen rule the `use_animation_frame` port
//!   documents for its tick closures). A canceled wrapper is dropped immediately: after the
//!   native cancel it can never be invoked again.
//! - The zero-arg `fn: () => void` parameter (`packages/utils/src/useIdleCallback.ts:28`)
//!   becomes [`IdleCallback::start`]'s `impl FnOnce() + 'static`; the callback receives no
//!   arguments and its return value is discarded, exactly like upstream (the payload shape is
//!   UNVERIFIED upstream — the tests pass argument-less `vi.fn()`s,
//!   `specs/utils/useIdleCallback.md`, "Events").
//! - Rust has no JS object identity, so the suite's `expect(dispose).toBe(idleCallback.clear)`
//!   (`packages/utils/src/useIdleCallback.test.tsx:110-111`) has no analog; `dispose_effect`
//!   returns a closure over a handle clone that behaves identically — canceling through it
//!   prevents the callback from firing.
//! - The `useIdleCallback` hook (`packages/utils/src/useIdleCallback.ts:56-62`) dissolves
//!   React's `useRefWithInit`/`useOnMount` plumbing into Leptos owner semantics, exactly like
//!   the `use_animation_frame` port's hook: a component's setup body runs once, so the local
//!   binding is the stable instance the way upstream's `useRefWithInit` memoizes it
//!   (`packages/utils/src/useIdleCallback.test.tsx:123-137`), and [`on_cleanup`] plays the role
//!   of the mount effect's returned cleanup — a pending callback is canceled when the owning
//!   reactive scope is disposed (`packages/utils/src/useIdleCallback.test.tsx:148-154`). Must
//!   be called inside a reactive owner (a component).
//! - Host builds (native `cargo test`) have no browser globals, so the dispatch layer has a
//!   process-global override seam — the Rust analog of upstream's `vi.stubGlobal` of
//!   `requestIdleCallback`/`cancelIdleCallback`
//!   (`packages/utils/src/useIdleCallback.test.tsx:26-29`). Host tests install a manual queue;
//!   the wasm production target always resolves the real ambient globals per call and falls
//!   back to `globalThis.setTimeout(fn, 0)` when `requestIdleCallback` is not a function.
//! - `'use client'` (`packages/utils/src/useIdleCallback.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use send_wrapper::SendWrapper;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

/// The handle the ambient dispatch global returns — upstream's `currentId` number
/// (`packages/utils/src/useIdleCallback.ts:20`).
pub type IdleCallbackId = u32;

/// The scheduled unit — upstream's `fn: () => void`
/// (`packages/utils/src/useIdleCallback.ts:28`), boxed because it crosses the dispatch boundary
/// as a one-shot callback.
type IdleCallbackJob = Box<dyn FnOnce()>;

/// An instance handle keeping at most one idle callback pending — upstream's `IdleCallback`
/// class (`packages/utils/src/useIdleCallback.ts:15-46`). Cheap to clone; clones share the same
/// pending-id slot.
#[derive(Clone, Default)]
pub struct IdleCallback {
    /// Upstream `currentId` (`packages/utils/src/useIdleCallback.ts:20`); `None` is upstream's
    /// `null`.
    current_id: Rc<Cell<Option<IdleCallbackId>>>,
}

impl IdleCallback {
    /// Upstream `static create()` (`packages/utils/src/useIdleCallback.ts:16-18`) — two calls
    /// return distinct instances
    /// (`packages/utils/src/useIdleCallback.test.tsx:44-50`).
    pub fn create() -> Self {
        Self::default()
    }

    /// The currently pending dispatch id, if any — upstream's public `currentId` field
    /// (`packages/utils/src/useIdleCallback.ts:20`). It is `None` once the scheduled callback
    /// has run, so a later [`IdleCallback::clear`] cannot cancel an unrelated callback
    /// (`packages/utils/src/useIdleCallback.test.tsx:97-98`).
    pub fn current_id(&self) -> Option<IdleCallbackId> {
        self.current_id.get()
    }

    /// Schedules `callback` to run asynchronously after the current task, clearing any
    /// previously scheduled call (`packages/utils/src/useIdleCallback.ts:22-34`). With native
    /// `requestIdleCallback` the callback runs during idle time after the next paint; the
    /// `setTimeout(0)` fallback runs after the current task but is not guaranteed to run after
    /// the next paint. The wrapper clears the pending id before invoking, so the callback may
    /// `start` the instance again (`packages/utils/src/useIdleCallback.ts:30-33`).
    pub fn start(&self, callback: impl FnOnce() + 'static) {
        self.clear();
        let current_id = Rc::clone(&self.current_id);
        let id = request_idle_callback(Box::new(move || {
            current_id.set(None);
            callback();
        }));
        self.current_id.set(Some(id));
    }

    /// Cancels the pending callback, if any
    /// (`packages/utils/src/useIdleCallback.ts:36-41`); a no-op when nothing is pending —
    /// including after the scheduled callback already ran, because the id slot was reset
    /// (`packages/utils/src/useIdleCallback.test.tsx:97-98`).
    pub fn clear(&self) {
        if let Some(id) = self.current_id.take() {
            cancel_idle_callback(id);
        }
    }

    /// Returns the cancel behavior as a cleanup — upstream `disposeEffect`
    /// (`packages/utils/src/useIdleCallback.ts:43-45`), which returns `clear` by identity and is
    /// consumed by `useOnMount` in the upstream hook
    /// (`packages/utils/src/useIdleCallback.test.tsx:110-116`). [`use_idle_callback`] wires it
    /// to [`on_cleanup`] directly; exposed for callers building their own mount-effect
    /// plumbing.
    pub fn dispose_effect(&self) -> impl Fn() + 'static {
        let idle_callback = self.clone();
        move || idle_callback.clear()
    }
}

/// A `requestIdleCallback` with automatic cleanup and guard, mirroring `useTimeout` — upstream's
/// `useIdleCallback()` hook (`packages/utils/src/useIdleCallback.ts:48-62`): creates the
/// instance (a Leptos component body runs once, so the binding is the stable instance the way
/// upstream's `useRefWithInit` memoizes it) and cancels any pending callback when the owning
/// reactive scope is disposed (upstream registers the instance's `disposeEffect` through
/// `useOnMount`; `packages/utils/src/useIdleCallback.test.tsx:148-154` proves the observable
/// contract). Must be called inside a reactive owner (a component).
pub fn use_idle_callback() -> IdleCallback {
    let idle_callback = IdleCallback::create();
    let cleanup = SendWrapper::new(idle_callback.dispose_effect());
    on_cleanup(move || (*cleanup)());
    idle_callback
}

thread_local! {
    /// Native registrations whose wrapper closure must stay alive until the job fires, keyed by
    /// the id the ambient global returned. A wrapper may only be dropped here after it can no
    /// longer be invoked: canceled registrations (dropped on cancel) and fired registrations
    /// (moved to [`RETIRED_JOBS`] after the job completes).
    #[cfg(target_arch = "wasm32")]
    static LIVE_JOBS: RefCell<HashMap<IdleCallbackId, Closure<dyn FnMut()>>> =
        RefCell::new(HashMap::new());

    /// Fired wrapper closures, dropped on the next entry into the dispatch layer — never during
    /// their own invocation (see the module docs on closure lifetime).
    #[cfg(target_arch = "wasm32")]
    static RETIRED_JOBS: RefCell<Vec<Closure<dyn FnMut()>>> = RefCell::new(Vec::new());

    /// Host-only dispatch override — the Rust analog of upstream's `vi.stubGlobal` of the
    /// `requestIdleCallback`/`cancelIdleCallback` globals
    /// (`packages/utils/src/useIdleCallback.test.tsx:26-29`). The wasm production target always
    /// dispatches through the real ambient globals instead.
    #[cfg(not(target_arch = "wasm32"))]
    static DISPATCH_OVERRIDE: RefCell<Option<(RequestOverride, CancelOverride)>> =
        const { RefCell::new(None) };
}

/// Drops wrapper closures whose jobs have already fired. Safe at any module entry point —
/// including a nested one from inside a running job — because a retired wrapper is by
/// definition no longer on the JS stack (it retired after its own invocation completed).
#[cfg(target_arch = "wasm32")]
fn sweep_retired_jobs() {
    RETIRED_JOBS.with(|retired| retired.borrow_mut().clear());
}

/// Reads a global function by name — the Rust form of upstream's
/// `typeof requestIdleCallback === 'function'` probe
/// (`packages/utils/src/useIdleCallback.ts:5`), resolved per call (see the module docs).
#[cfg(target_arch = "wasm32")]
fn get_global_function(name: &str) -> Option<js_sys::Function> {
    js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str(name))
        .ok()
        .and_then(|value| value.dyn_into().ok())
}

/// Dispatches one job through `globalThis.requestIdleCallback`, falling back to
/// `globalThis.setTimeout(fn, 0)` (`packages/utils/src/useIdleCallback.ts:10-12`), and keeps
/// the wrapper closure alive until the job fires.
#[cfg(target_arch = "wasm32")]
fn request_idle_callback(job: IdleCallbackJob) -> IdleCallbackId {
    sweep_retired_jobs();

    let mut job = Some(job);
    let native_id_slot = Rc::new(Cell::new(0u32));
    let id_for_wrapper = Rc::clone(&native_id_slot);
    let wrapper = Closure::wrap(Box::new(move || {
        let job = job
            .take()
            .expect("idle callback wrapper must be invoked at most once");
        job();
        // Retire this wrapper: after the job completes, move it out of the live registry so
        // the deferred drop never happens during its own invocation.
        let id = id_for_wrapper.get();
        LIVE_JOBS.with(|jobs| {
            if let Some(fired) = jobs.borrow_mut().remove(&id) {
                RETIRED_JOBS.with(|retired| retired.borrow_mut().push(fired));
            }
        });
    }) as Box<dyn FnMut()>);

    // The ambient globals never fire synchronously (`requestIdleCallback`/`setTimeout` are
    // async by specification), so the registration below is complete before the wrapper can
    // possibly run and read `native_id_slot`.
    let native_id = if let Some(request) = get_global_function("requestIdleCallback") {
        request
            .call1(&js_sys::global(), wrapper.as_ref().unchecked_ref())
            .expect("requestIdleCallback threw while scheduling an idle callback")
    } else {
        let request = get_global_function("setTimeout")
            .expect("globalThis.setTimeout must be a function for the idle callback fallback");
        request
            .call2(
                &js_sys::global(),
                wrapper.as_ref().unchecked_ref(),
                &JsValue::from_f64(0.0),
            )
            .expect("setTimeout threw while scheduling the idle callback fallback")
    };
    let native_id = native_id
        .as_f64()
        .expect("idle callback dispatch must return a numeric id")
        as IdleCallbackId;
    native_id_slot.set(native_id);
    LIVE_JOBS.with(|jobs| {
        jobs.borrow_mut().insert(native_id, wrapper);
    });
    native_id
}

/// Cancels a pending registration — upstream's `cancelCallback` choice
/// (`packages/utils/src/useIdleCallback.ts:13`): `cancelIdleCallback` when the environment has
/// one, `clearTimeout` on the fallback path. The pending wrapper is dropped after the native
/// cancel: it can never be invoked again, so dropping it keeps wasm-bindgen's
/// no-drop-while-invocable rule intact.
#[cfg(target_arch = "wasm32")]
fn cancel_idle_callback(id: IdleCallbackId) {
    sweep_retired_jobs();

    if let Some(cancel) = get_global_function("cancelIdleCallback") {
        cancel
            .call1(&js_sys::global(), &JsValue::from_f64(id as f64))
            .expect("cancelIdleCallback threw while canceling an idle callback");
    } else {
        let cancel = get_global_function("clearTimeout")
            .expect("globalThis.clearTimeout must be a function for the idle callback fallback");
        cancel
            .call1(&js_sys::global(), &JsValue::from_f64(id as f64))
            .expect("clearTimeout threw while canceling the idle callback fallback");
    }
    LIVE_JOBS.with(|jobs| {
        jobs.borrow_mut().remove(&id);
    });
}

/// The host override's request half: queue a job, return its id.
#[cfg(not(target_arch = "wasm32"))]
type RequestOverride = Box<dyn Fn(IdleCallbackJob) -> IdleCallbackId>;

/// The host override's cancel half: drop a queued job by id.
#[cfg(not(target_arch = "wasm32"))]
type CancelOverride = Box<dyn Fn(IdleCallbackId)>;

/// Host-target dispatch: there are no browser globals to resolve, so tests install an override
/// (`DISPATCH_OVERRIDE`) — the analog of upstream's `vi.stubGlobal` of the ambient globals
/// (`packages/utils/src/useIdleCallback.test.tsx:26-29`).
#[cfg(not(target_arch = "wasm32"))]
fn request_idle_callback(job: IdleCallbackJob) -> IdleCallbackId {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (request, _) = dispatch.as_ref().expect(
            "no idle callback dispatcher installed on the host target: host builds have no \
             requestIdleCallback/setTimeout globals, so tests must install a dispatch override",
        );
        request(job)
    })
}

/// Host-target cancel — see [`request_idle_callback`] for why the override exists.
#[cfg(not(target_arch = "wasm32"))]
fn cancel_idle_callback(id: IdleCallbackId) {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (_, cancel) = dispatch.as_ref().expect(
            "no idle callback dispatcher installed on the host target: host builds have no \
             requestIdleCallback/setTimeout globals, so tests must install a dispatch override",
        );
        cancel(id)
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;

    use super::*;

    /// The manual queue upstream's main suite stubs the globals with
    /// (`packages/utils/src/useIdleCallback.test.tsx:10-23`): `scheduleCallback` appends to the
    /// queue and returns a fresh numeric id, `cancelIdleCallback` removes by id, and
    /// `flushIdleCallbacks` drains and runs. Installed as the dispatch override; restored on
    /// drop so a panicking test cannot poison the later ones.
    struct ManualQueue {
        scheduled: Rc<RefCell<Vec<(IdleCallbackId, IdleCallbackJob)>>>,
    }

    impl ManualQueue {
        fn install() -> Self {
            let scheduled: Rc<RefCell<Vec<(IdleCallbackId, IdleCallbackJob)>>> =
                Rc::new(RefCell::new(Vec::new()));

            let request_scheduled = Rc::clone(&scheduled);
            let request_next_id = Cell::new(0u32);
            let request: RequestOverride = Box::new(move |job: IdleCallbackJob| {
                request_next_id.set(request_next_id.get() + 1);
                let id = request_next_id.get();
                request_scheduled.borrow_mut().push((id, job));
                id
            });

            let cancel_scheduled = Rc::clone(&scheduled);
            let cancel: CancelOverride = Box::new(move |id: IdleCallbackId| {
                cancel_scheduled
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
            });

            DISPATCH_OVERRIDE.with(|dispatch| {
                *dispatch.borrow_mut() = Some((request, cancel));
            });
            Self { scheduled }
        }

        /// Upstream `flushIdleCallbacks`
        /// (`packages/utils/src/useIdleCallback.test.tsx:19-23`): snapshot the queue, clear it,
        /// then run — a callback that schedules new work during the flush stays pending for the
        /// next flush, exactly like upstream.
        fn flush_idle_callbacks(&self) {
            let batch: Vec<IdleCallbackJob> = self
                .scheduled
                .borrow_mut()
                .drain(..)
                .map(|(_, job)| job)
                .collect();
            for job in batch {
                job();
            }
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

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:44-50` (spec "Public API surface"):
    // two `create()` calls return distinct instances, each usable as its own scheduler.
    #[test]
    fn create_returns_a_distinct_instance_each_time() {
        let a: IdleCallback = IdleCallback::create();
        let b: IdleCallback = IdleCallback::create();
        assert!(
            !Rc::ptr_eq(&a.current_id, &b.current_id),
            "create() must return a new instance each time"
        );
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:52-63` (spec "State model": dispatch
    // is asynchronous relative to the scheduling task).
    #[test]
    fn runs_the_scheduled_callback_after_the_current_task() {
        let queue = ManualQueue::install();
        let idle_callback = IdleCallback::create();
        let (fired, callback) = counting_callback();

        idle_callback.start(callback);

        assert_eq!(
            fired.get(),
            0,
            "the callback runs asynchronously after the current task, not synchronously"
        );

        queue.flush_idle_callbacks();
        assert_eq!(fired.get(), 1);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:65-74` (spec "Edge cases":
    // cancel-then-flush safety).
    #[test]
    fn clear_cancels_a_pending_callback() {
        let queue = ManualQueue::install();
        let idle_callback = IdleCallback::create();
        let (fired, callback) = counting_callback();

        idle_callback.start(callback);
        idle_callback.clear();

        queue.flush_idle_callbacks();
        assert_eq!(fired.get(), 0);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:76-87` (spec "State model": a second
    // start supersedes the previous callback — only the newest runs, the superseded one never
    // fires).
    #[test]
    fn start_clears_any_previously_scheduled_callback() {
        let queue = ManualQueue::install();
        let idle_callback = IdleCallback::create();
        let (first_fired, first) = counting_callback();
        let (second_fired, second) = counting_callback();

        idle_callback.start(first);
        idle_callback.start(second);

        queue.flush_idle_callbacks();
        assert_eq!(second_fired.get(), 1);
        assert_eq!(first_fired.get(), 0);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:89-103` (spec "Edge cases": reuse
    // after completion — the id slot resets to null at
    // `packages/utils/src/useIdleCallback.test.tsx:97-98`, so the instance can be started
    // again).
    #[test]
    fn can_be_reused_after_the_callback_has_run() {
        let queue = ManualQueue::install();
        let idle_callback = IdleCallback::create();
        let (fired, callback) = counting_callback();

        idle_callback.start(callback);
        queue.flush_idle_callbacks();
        assert_eq!(fired.get(), 1);
        assert!(
            idle_callback.current_id().is_none(),
            "the completed handle must be reset so a later clear() cannot cancel an unrelated \
             callback"
        );

        let (fired_again, callback_again) = counting_callback();
        idle_callback.start(callback_again);
        queue.flush_idle_callbacks();
        assert_eq!(fired_again.get(), 1);
        assert_eq!(fired.get(), 1, "the first callback is not re-run");
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:105-117` (spec "Public API surface":
    // `disposeEffect()` is the cleanup). The JS identity assertion
    // `expect(dispose).toBe(idleCallback.clear)`
    // (`packages/utils/src/useIdleCallback.test.tsx:110-111`) has no Rust analog, so the port
    // pins the behavior instead: canceling through the returned dispose function prevents the
    // callback from firing.
    #[test]
    fn dispose_effect_cancels_a_pending_callback() {
        let queue = ManualQueue::install();
        let idle_callback = IdleCallback::create();
        let (fired, callback) = counting_callback();

        idle_callback.start(callback);
        let dispose = idle_callback.dispose_effect();
        dispose();

        queue.flush_idle_callbacks();
        assert_eq!(fired.get(), 0);
    }

    // Pins per-instance scheduling (spec "State model": two callbacks started on two different
    // instances are both delivered by one flush, while one instance collapses to the newest
    // only).
    #[test]
    fn two_instances_deliver_both_callbacks_on_one_flush() {
        let queue = ManualQueue::install();
        let first_idle = IdleCallback::create();
        let second_idle = IdleCallback::create();
        let (first_fired, first) = counting_callback();
        let (second_fired, second) = counting_callback();

        first_idle.start(first);
        second_idle.start(second);

        queue.flush_idle_callbacks();
        assert_eq!(first_fired.get(), 1);
        assert_eq!(second_fired.get(), 1);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:139-155` (spec "Edge cases": a
    // pending callback started via the hook's scheduler is canceled when the component
    // unmounts) — reactive owner disposal plays the role of React unmount running the
    // `useOnMount` cleanup.
    #[test]
    fn use_idle_callback_cancels_the_pending_callback_when_the_reactive_owner_is_disposed() {
        let queue = ManualQueue::install();

        let owner = Owner::new();
        owner.set();
        let scheduler = use_idle_callback();
        let (fired, callback) = counting_callback();
        scheduler.start(callback);

        owner.cleanup();
        queue.flush_idle_callbacks();
        assert_eq!(
            fired.get(),
            0,
            "the pending callback is canceled on unmount"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;

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

    /// The global stub upstream's main suite installs in `beforeAll`
    /// (`packages/utils/src/useIdleCallback.test.tsx:25-32`): `requestIdleCallback` queues every
    /// callback and returns a fresh numeric id, `cancelIdleCallback` removes by id — the
    /// Map-backed manual queue. Restores the originals on drop so a panicking test cannot
    /// poison the later ones.
    struct IdleCallbackStub {
        original_request: JsValue,
        original_cancel: JsValue,
        /// Kept alive for the stub's lifetime; the browser only sees the JS side.
        #[allow(dead_code)]
        request_closure: Closure<dyn FnMut(Function) -> JsValue>,
        #[allow(dead_code)]
        cancel_closure: Closure<dyn FnMut(JsValue)>,
        scheduled: Rc<RefCell<Vec<(u32, Function)>>>,
    }

    impl IdleCallbackStub {
        fn install() -> Self {
            let scheduled: Rc<RefCell<Vec<(u32, Function)>>> = Rc::new(RefCell::new(Vec::new()));

            let request_scheduled = Rc::clone(&scheduled);
            let request_next_id = Cell::new(0u32);
            let request_closure = Closure::wrap(Box::new(move |callback: Function| -> JsValue {
                request_next_id.set(request_next_id.get() + 1);
                let id = request_next_id.get();
                request_scheduled.borrow_mut().push((id, callback));
                JsValue::from_f64(f64::from(id))
            })
                as Box<dyn FnMut(Function) -> JsValue>);

            let cancel_scheduled = Rc::clone(&scheduled);
            let cancel_closure = Closure::wrap(Box::new(move |id: JsValue| {
                let id = id.as_f64().unwrap_or(0.0) as u32;
                cancel_scheduled
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
            }) as Box<dyn FnMut(JsValue)>);

            let request_key = JsValue::from_str("requestIdleCallback");
            let cancel_key = JsValue::from_str("cancelIdleCallback");
            let global = js_sys::global();
            let original_request = Reflect::get(&global, &request_key)
                .expect("globalThis.requestIdleCallback must be readable");
            let original_cancel = Reflect::get(&global, &cancel_key)
                .expect("globalThis.cancelIdleCallback must be readable");
            set_global(
                "requestIdleCallback",
                request_closure.as_ref().unchecked_ref(),
            );
            set_global(
                "cancelIdleCallback",
                cancel_closure.as_ref().unchecked_ref(),
            );

            Self {
                original_request,
                original_cancel,
                request_closure,
                cancel_closure,
                scheduled,
            }
        }

        /// Upstream `flushIdleCallbacks`
        /// (`packages/utils/src/useIdleCallback.test.tsx:19-23`): snapshot, clear, then run.
        fn flush_idle_callbacks(&self) {
            let batch: Vec<Function> = self
                .scheduled
                .borrow_mut()
                .drain(..)
                .map(|(_, callback)| callback)
                .collect();
            for callback in batch {
                callback
                    .call0(&JsValue::UNDEFINED)
                    .expect("the scheduled idle callback threw");
            }
        }
    }

    impl Drop for IdleCallbackStub {
        fn drop(&mut self) {
            set_global("requestIdleCallback", &self.original_request);
            set_global("cancelIdleCallback", &self.original_cancel);
        }
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:52-63` (spec "State model": with
    // `requestIdleCallback` present, dispatch goes through the ambient global
    // (`packages/utils/src/useIdleCallback.test.tsx:26-31`) and the callback runs
    // asynchronously, only when idle callbacks are flushed).
    #[wasm_bindgen_test]
    fn runs_the_scheduled_callback_through_the_ambient_globals() {
        let stub = IdleCallbackStub::install();
        let idle_callback = IdleCallback::create();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);

        idle_callback.start(move || counter.set(counter.get() + 1));
        assert_eq!(
            fired.get(),
            0,
            "the callback runs asynchronously after the current task, not synchronously"
        );

        stub.flush_idle_callbacks();
        assert_eq!(fired.get(), 1);

        drop(stub);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:65-87` through the same ambient
    // dispatch path: `clear()` cancels a pending callback and a second `start()` supersedes the
    // first.
    #[wasm_bindgen_test]
    fn clear_cancels_and_a_second_start_supersedes_through_the_ambient_globals() {
        let stub = IdleCallbackStub::install();
        let idle_callback = IdleCallback::create();

        let canceled = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&canceled);
        idle_callback.start(move || counter.set(counter.get() + 1));
        idle_callback.clear();

        let superseding = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&superseding);
        idle_callback.start(move || counter.set(counter.get() + 1));

        stub.flush_idle_callbacks();
        assert_eq!(canceled.get(), 0);
        assert_eq!(superseding.get(), 1);

        drop(stub);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:89-103`: the id slot resets to null
    // once the callback ran (`packages/utils/src/useIdleCallback.test.tsx:97-98`) and the
    // instance can be started again.
    #[wasm_bindgen_test]
    fn the_instance_can_be_reused_after_the_callback_has_run() {
        let stub = IdleCallbackStub::install();
        let idle_callback = IdleCallback::create();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);

        idle_callback.start(move || counter.set(counter.get() + 1));
        stub.flush_idle_callbacks();
        assert_eq!(fired.get(), 1);
        assert!(idle_callback.current_id().is_none());

        let counter = Rc::clone(&fired);
        idle_callback.start(move || counter.set(counter.get() + 1));
        stub.flush_idle_callbacks();
        assert_eq!(fired.get(), 2);

        drop(stub);
    }

    // Mirrors `packages/utils/src/useIdleCallback.test.tsx:139-155` (spec "Edge cases"): a
    // pending callback started via the hook's scheduler is canceled when the component
    // unmounts — reactive owner disposal plays the role of React unmount.
    #[wasm_bindgen_test]
    fn use_idle_callback_cancels_the_pending_callback_when_the_reactive_owner_is_disposed() {
        let stub = IdleCallbackStub::install();

        let owner = Owner::new();
        owner.set();
        let scheduler = use_idle_callback();
        let fired = Rc::new(Cell::new(0u32));
        let counter = Rc::clone(&fired);
        scheduler.start(move || counter.set(counter.get() + 1));

        owner.cleanup();
        stub.flush_idle_callbacks();
        assert_eq!(
            fired.get(),
            0,
            "the pending callback is canceled on unmount"
        );

        drop(stub);
    }

    /// The fallback environment upstream's fallback suite creates in `beforeAll`
    /// (`packages/utils/src/useIdleCallback.fallback.test.ts:7-13`): both idle-callback globals
    /// removed, and `setTimeout`/`clearTimeout` replaced by a manual timer queue standing in
    /// for the fake timers (`vi.runOnlyPendingTimers()` at
    /// `packages/utils/src/useIdleCallback.fallback.test.ts:29` becomes
    /// [`FallbackStub::flush_pending_timers`]).
    struct FallbackStub {
        original_request: JsValue,
        original_cancel: JsValue,
        original_set_timeout: JsValue,
        original_clear_timeout: JsValue,
        /// Kept alive for the stub's lifetime; the browser only sees the JS side.
        #[allow(dead_code)]
        set_timeout_closure: Closure<dyn FnMut(Function) -> JsValue>,
        #[allow(dead_code)]
        clear_timeout_closure: Closure<dyn FnMut(JsValue)>,
        timers: Rc<RefCell<Vec<(u32, Function)>>>,
    }

    impl FallbackStub {
        fn install() -> Self {
            let timers: Rc<RefCell<Vec<(u32, Function)>>> = Rc::new(RefCell::new(Vec::new()));

            let set_timers = Rc::clone(&timers);
            let set_next_id = Cell::new(0u32);
            let set_timeout_closure =
                Closure::wrap(Box::new(move |callback: Function| -> JsValue {
                    set_next_id.set(set_next_id.get() + 1);
                    let id = set_next_id.get();
                    set_timers.borrow_mut().push((id, callback));
                    JsValue::from_f64(f64::from(id))
                }) as Box<dyn FnMut(Function) -> JsValue>);

            let clear_timers = Rc::clone(&timers);
            let clear_timeout_closure = Closure::wrap(Box::new(move |id: JsValue| {
                let id = id.as_f64().unwrap_or(0.0) as u32;
                clear_timers
                    .borrow_mut()
                    .retain(|(queued, _)| *queued != id);
            }) as Box<dyn FnMut(JsValue)>);

            let request_key = JsValue::from_str("requestIdleCallback");
            let cancel_key = JsValue::from_str("cancelIdleCallback");
            let set_key = JsValue::from_str("setTimeout");
            let clear_key = JsValue::from_str("clearTimeout");
            let global = js_sys::global();
            let original_request = Reflect::get(&global, &request_key)
                .expect("globalThis.requestIdleCallback must be readable");
            let original_cancel = Reflect::get(&global, &cancel_key)
                .expect("globalThis.cancelIdleCallback must be readable");
            let original_set_timeout =
                Reflect::get(&global, &set_key).expect("globalThis.setTimeout must be readable");
            let original_clear_timeout = Reflect::get(&global, &clear_key)
                .expect("globalThis.clearTimeout must be readable");

            set_global("requestIdleCallback", &JsValue::UNDEFINED);
            set_global("cancelIdleCallback", &JsValue::UNDEFINED);
            set_global("setTimeout", set_timeout_closure.as_ref().unchecked_ref());
            set_global(
                "clearTimeout",
                clear_timeout_closure.as_ref().unchecked_ref(),
            );

            Self {
                original_request,
                original_cancel,
                original_set_timeout,
                original_clear_timeout,
                set_timeout_closure,
                clear_timeout_closure,
                timers,
            }
        }

        /// Upstream `vi.runOnlyPendingTimers()`
        /// (`packages/utils/src/useIdleCallback.fallback.test.ts:29`).
        fn flush_pending_timers(&self) {
            let batch: Vec<Function> = self
                .timers
                .borrow_mut()
                .drain(..)
                .map(|(_, callback)| callback)
                .collect();
            for callback in batch {
                callback
                    .call0(&JsValue::UNDEFINED)
                    .expect("the pending timer callback threw");
            }
        }
    }

    impl Drop for FallbackStub {
        fn drop(&mut self) {
            set_global("requestIdleCallback", &self.original_request);
            set_global("cancelIdleCallback", &self.original_cancel);
            set_global("setTimeout", &self.original_set_timeout);
            set_global("clearTimeout", &self.original_clear_timeout);
        }
    }

    // Mirrors `packages/utils/src/useIdleCallback.fallback.test.ts:21-38` (spec "Edge cases":
    // with both idle-callback globals absent, the full schedule → supersede → cancel cycle
    // still works via the timer-based path).
    #[wasm_bindgen_test]
    fn schedules_and_cancels_callbacks_when_request_idle_callback_is_unavailable() {
        let stub = FallbackStub::install();
        let idle_callback = IdleCallback::create();

        let first = Rc::new(Cell::new(0u32));
        let second = Rc::new(Cell::new(0u32));

        let first_counter = Rc::clone(&first);
        idle_callback.start(move || first_counter.set(first_counter.get() + 1));
        let second_counter = Rc::clone(&second);
        idle_callback.start(move || second_counter.set(second_counter.get() + 1));

        stub.flush_pending_timers();
        assert_eq!(first.get(), 0, "the superseded callback never fires");
        assert_eq!(second.get(), 1);

        let first_counter = Rc::clone(&first);
        idle_callback.start(move || first_counter.set(first_counter.get() + 1));
        idle_callback.clear();

        stub.flush_pending_timers();
        assert_eq!(first.get(), 0, "the canceled callback never fires");

        drop(stub);
    }
}
