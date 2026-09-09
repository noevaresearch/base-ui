//! Port of `packages/react/src/internals/TimeoutManager.ts:1-29` — the multi-timeout
//! manager used by popup-type components to run several keyed delays against one instance
//! (`TODO.md`, item `infra: internals`).
//!
//! Upstream is a 29-line class: `ids: Map<string, number>` plus three pre-bound methods —
//! `start(key, delay, fn)` clears the key's pending timeout first (replacement semantics,
//! `TimeoutManager.ts:7-15`), schedules a wrapper that deletes its own entry *before*
//! invoking `fn` (`:9-12`), and records the id; `clear(key)` clears and deletes only a live
//! entry (`:17-23`); `clearAll()` clears and drops everything (`:25-28`). The callbacks run
//! on the raw ambient `setTimeout` — this is a non-React utility, not the `useTimeout`
//! hook (the implementation spec's "Pure units" note).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The `ids` map lives behind an `Rc<RefCell<…>>` and the handle is [`Clone`] — clones
//!   share one registry the way JS callers share the instance's field
//!   (`TimeoutManager.ts:5`), the crate's shared-handle convention (`use_timeout.rs`'s
//!   slot-shape note).
//! - The fired wrapper's own-entry delete (`TimeoutManager.ts:10`) ports verbatim as an
//!   unconditional `remove(key)` — safe for the same reason it is upstream: `start` cleared
//!   any predecessor first, so an entry present when a wrapper fires is that wrapper's own.
//!   A `clear` after a timeout has fired finds no entry and is a no-op (`:17-23`), which
//!   the safe-clear test pins; code inside `fn` observes the key as cleared, and a
//!   re-`start` from inside `fn` re-arms cleanly.
//! - Dispatch resolves the ambient timer globals per call (`TimeoutManager.ts:9`, `:20`,
//!   `:26` — bare JS globals, no realm handling of their own). On the wasm production
//!   target that is the browser's `setTimeout`/`clearTimeout` through the window; on host
//!   builds there are no timer globals, so tests install a dispatch override
//!   ([`DISPATCH_OVERRIDE`]) — the analog of stubbing the ambient globals, which is exactly
//!   how upstream's suite observes the deferral (`vi.useFakeTimers()`,
//!   `TimeoutManager.test.ts:5-11`). Both targets run the full behavioral suite: the host
//!   fake-timer suite inside the default `cargo test` gate, the wasm suite in the browser
//!   against real timers (the `use_timeout` port's dual-target precedent).
//! - The wasm wrapper is registered with `Closure::once_into_js`, transferring its
//!   ownership to the JS side: a one-shot registration is complete once it has fired, so
//!   the native side never needs to drop the closure — no live-job registry, unlike the
//!   repeating-timer ports whose wrappers must stay droppable for `clear`.
//! - JS numbers → `i32` for the timer id and the millisecond delay — the `web-sys` binding
//!   types; upstream's `as unknown as number` cast comment (`:12`) exists for the Node
//!   typing, not behavior.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The id the ambient timer global returns — upstream's `TimeoutManager.ts:12` `number`.
#[cfg(not(target_arch = "wasm32"))]
pub type TimeoutManagerId = i32;

/// The scheduled unit — upstream's `fn: () => void` parameter
/// (`packages/react/src/internals/TimeoutManager.ts:7`), boxed because it crosses the
/// dispatch boundary as a stored callback.
#[cfg(not(target_arch = "wasm32"))]
type TimeoutJob = Box<dyn FnOnce()>;

/// The upstream `TimeoutManager` (`packages/react/src/internals/TimeoutManager.ts:4-29`).
/// Clones share one id registry, mirroring the JS instance's single `ids` map.
#[derive(Clone, Default)]
pub struct TimeoutManager {
    ids: Rc<RefCell<HashMap<String, i32>>>,
}

impl TimeoutManager {
    /// The upstream `start` (`packages/react/src/internals/TimeoutManager.ts:7-15`): a new
    /// `start` for an existing key replaces the pending timeout — the first callback never
    /// fires.
    pub fn start(&self, key: &str, delay: i32, f: impl FnOnce() + 'static) {
        self.clear(key);

        let id = schedule(key.to_string(), Rc::clone(&self.ids), f, delay);
        self.ids.borrow_mut().insert(key.to_string(), id);
    }

    /// The upstream `clear` (`packages/react/src/internals/TimeoutManager.ts:17-23`):
    /// cancels a live timeout and drops its entry; a missing or already-fired key is a
    /// safe no-op.
    pub fn clear(&self, key: &str) {
        let id = self.ids.borrow_mut().remove(key);
        if let Some(id) = id {
            cancel(id);
        }
    }

    /// The upstream `clearAll` (`packages/react/src/internals/TimeoutManager.ts:25-28`):
    /// cancels every live timeout and empties the registry.
    pub fn clear_all(&self) {
        let ids: Vec<i32> = self.ids.borrow_mut().drain().map(|(_, id)| id).collect();
        for id in ids {
            cancel(id);
        }
    }
}

/// Schedules the fired wrapper — the unconditional own-entry delete before `fn`
/// (`packages/react/src/internals/TimeoutManager.ts:9-12`) — against the target's timer
/// global.
#[cfg(target_arch = "wasm32")]
fn schedule(
    key: String,
    ids: Rc<RefCell<HashMap<String, i32>>>,
    f: impl FnOnce() + 'static,
    delay: i32,
) -> i32 {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;

    let wrapper = Closure::once_into_js(move || {
        ids.borrow_mut().remove(&key);
        f();
    });

    let window = web_sys::window().expect("window for setTimeout");
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            wrapper.unchecked_ref::<js_sys::Function>(),
            delay,
        )
        .expect("setTimeout threw while scheduling a timeout")
}

/// Cancels a live registration against the target's timer global
/// (`packages/react/src/internals/TimeoutManager.ts:20`, `:26`).
#[cfg(target_arch = "wasm32")]
fn cancel(id: i32) {
    if let Some(window) = web_sys::window() {
        window.clear_timeout_with_handle(id);
    }
}

thread_local! {
    /// Host-only dispatch override — the Rust analog of stubbing the ambient
    /// `setTimeout`/`clearTimeout` globals, which host builds do not have. The wasm
    /// production target always resolves the real ambient globals per call.
    #[cfg(not(target_arch = "wasm32"))]
    static DISPATCH_OVERRIDE: RefCell<Option<(RequestOverride, CancelOverride)>> =
        const { RefCell::new(None) };
}

/// The host override's request half: queue a registration, return its id.
#[cfg(not(target_arch = "wasm32"))]
type RequestOverride = Box<dyn Fn(TimeoutJob, i32) -> i32>;

/// The host override's cancel half: drop a queued registration by id.
#[cfg(not(target_arch = "wasm32"))]
type CancelOverride = Box<dyn Fn(i32)>;

/// Host-target dispatch: there are no browser globals to resolve, so tests install an
/// override ([`DISPATCH_OVERRIDE`]) — the analog of stubbing the ambient
/// `setTimeout`/`clearTimeout` globals.
#[cfg(not(target_arch = "wasm32"))]
fn schedule(
    key: String,
    ids: Rc<RefCell<HashMap<String, i32>>>,
    f: impl FnOnce() + 'static,
    delay: i32,
) -> i32 {
    let job: TimeoutJob = Box::new(move || {
        ids.borrow_mut().remove(&key);
        f();
    });
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (request, _) = dispatch.as_ref().expect(
            "no timeout dispatcher installed on the host target: host builds have no \
             setTimeout/clearTimeout globals, so tests must install a dispatch override",
        );
        request(job, delay)
    })
}

/// Host-target cancel — see [`schedule`] for why the override exists.
#[cfg(not(target_arch = "wasm32"))]
fn cancel(id: i32) {
    DISPATCH_OVERRIDE.with(|dispatch| {
        let dispatch = dispatch.borrow();
        let (_, cancel) = dispatch.as_ref().expect(
            "no timeout dispatcher installed on the host target: host builds have no \
             setTimeout/clearTimeout globals, so tests must install a dispatch override",
        );
        cancel(id)
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;

    use super::*;

    /// One queued registration — its id (for [`TimeoutManager::clear`]'s cancel path), the
    /// job, and the deadline the fake clock will fire it at.
    struct FakeTimer {
        id: i32,
        deadline: i32,
        job: TimeoutJob,
    }

    thread_local! {
        static FAKE_TIMERS: RefCell<Vec<FakeTimer>> = const { RefCell::new(Vec::new()) };
        static FAKE_CLOCK: Cell<i32> = const { Cell::new(0) };
        static FAKE_NEXT_ID: Cell<i32> = const { Cell::new(1) };
    }

    /// Installs the host dispatch override with a fake clock — the analog of upstream's
    /// `vi.useFakeTimers()` (`packages/react/src/internals/TimeoutManager.test.ts:5-7`).
    /// Thread-locals make each test thread independent, like a fresh vitest worker.
    fn use_fake_timers() {
        FAKE_TIMERS.with(|timers| timers.borrow_mut().clear());
        FAKE_CLOCK.with(|clock| clock.set(0));
        FAKE_NEXT_ID.with(|next| next.set(1));
        DISPATCH_OVERRIDE.with(|dispatch| {
            *dispatch.borrow_mut() = Some((
                Box::new(|job: TimeoutJob, delay: i32| {
                    let id = FAKE_NEXT_ID.with(|next| {
                        let id = next.get();
                        next.set(id + 1);
                        id
                    });
                    let deadline = FAKE_CLOCK.with(|clock| clock.get()) + delay;
                    FAKE_TIMERS.with(|timers| {
                        timers.borrow_mut().push(FakeTimer { id, deadline, job });
                    });
                    id
                }) as RequestOverride,
                Box::new(|id: i32| {
                    FAKE_TIMERS.with(|timers| {
                        timers.borrow_mut().retain(|timer| timer.id != id);
                    });
                }) as CancelOverride,
            ));
        });
    }

    /// Advances the fake clock and fires every registration whose deadline has passed, in
    /// deadline order — the analog of `vi.advanceTimersByTime`
    /// (`packages/react/src/internals/TimeoutManager.test.ts:20` and friends). Firing can
    /// schedule further timers; the loop drains everything due.
    fn advance_timers_by_time(ms: i32) {
        FAKE_CLOCK.with(|clock| clock.set(clock.get() + ms));
        loop {
            let now = FAKE_CLOCK.with(|clock| clock.get());
            let mut due = FAKE_TIMERS.with(|timers| {
                let mut timers = timers.borrow_mut();
                let mut due_indexes: Vec<usize> = timers
                    .iter()
                    .enumerate()
                    .filter(|(_, timer)| timer.deadline <= now)
                    .map(|(index, _)| index)
                    .collect();
                due_indexes.reverse();
                due_indexes
                    .into_iter()
                    .map(|index| timers.remove(index).job)
                    .collect::<Vec<_>>()
            });
            // Deadline order within this tick.
            due.reverse();
            if due.is_empty() {
                break;
            }
            for job in due {
                job();
            }
        }
    }

    fn counter() -> Rc<Cell<usize>> {
        Rc::new(Cell::new(0))
    }

    fn counted(counter: &Rc<Cell<usize>>) -> impl FnOnce() + 'static {
        let counter = Rc::clone(counter);
        move || counter.set(counter.get() + 1)
    }

    // Mirrors `packages/react/src/internals/TimeoutManager.test.ts:13-22`.
    #[test]
    fn calls_the_callback_after_the_specified_delay() {
        use_fake_timers();
        let manager = TimeoutManager::default();
        let calls = counter();

        manager.start("key", 100, counted(&calls));
        assert_eq!(calls.get(), 0);

        advance_timers_by_time(100);
        assert_eq!(calls.get(), 1);
    }

    // Mirrors `packages/react/src/internals/TimeoutManager.test.ts:24-35`.
    #[test]
    fn replaces_a_timeout_when_start_is_called_with_the_same_key() {
        use_fake_timers();
        let manager = TimeoutManager::default();
        let first_calls = counter();
        let second_calls = counter();

        manager.start("key", 100, counted(&first_calls));
        manager.start("key", 100, counted(&second_calls));

        advance_timers_by_time(100);
        assert_eq!(first_calls.get(), 0, "the first callback never fires");
        assert_eq!(second_calls.get(), 1);
    }

    // Mirrors `packages/react/src/internals/TimeoutManager.test.ts:37-46`.
    #[test]
    fn clears_a_pending_timeout() {
        use_fake_timers();
        let manager = TimeoutManager::default();
        let calls = counter();

        manager.start("key", 100, counted(&calls));
        manager.clear("key");

        advance_timers_by_time(100);
        assert_eq!(calls.get(), 0);
    }

    // Mirrors `packages/react/src/internals/TimeoutManager.test.ts:48-51`.
    #[test]
    fn does_not_error_when_clearing_a_non_existent_key() {
        use_fake_timers();
        let manager = TimeoutManager::default();
        manager.clear("nonexistent");
    }

    // Mirrors `packages/react/src/internals/TimeoutManager.test.ts:53-65`.
    #[test]
    fn clears_all_active_timeouts() {
        use_fake_timers();
        let manager = TimeoutManager::default();
        let first_calls = counter();
        let second_calls = counter();

        manager.start("a", 100, counted(&first_calls));
        manager.start("b", 200, counted(&second_calls));
        manager.clear_all();

        advance_timers_by_time(200);
        assert_eq!(first_calls.get(), 0);
        assert_eq!(second_calls.get(), 0);
    }

    // Mirrors `packages/react/src/internals/TimeoutManager.test.ts:67-75`: the fired
    // wrapper deleted its own entry, so the late `clear` finds nothing.
    #[test]
    fn allows_safe_clear_after_a_timeout_has_fired() {
        use_fake_timers();
        let manager = TimeoutManager::default();
        let calls = counter();

        manager.start("key", 100, counted(&calls));
        advance_timers_by_time(100);

        manager.clear("key");
        assert_eq!(calls.get(), 1);
    }
}
