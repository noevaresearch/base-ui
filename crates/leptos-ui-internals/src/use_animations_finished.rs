//! Port of `packages/react/src/internals/useAnimationsFinished.ts` — the animation-completion
//! runner (`TODO.md`, item `infra: internals`; the transition/animation checkpoint recorded
//! in that entry's note).
//!
//! Upstream returns a stable callback that executes a function once all the element's
//! animations have finished, retrying while replacements churn (`useAnimationsFinished.ts:44-158`):
//! every run cancels the previous run's pending frame (`:64`); the element resolves from the
//! maybe-ref argument per invocation (`:66-69`); the immediate-call arm fires when
//! `getAnimations` is missing or the `BASE_UI_ANIMATIONS_DISABLED` kill switch is set
//! (`:91-97`); `exec()` awaits every animation's `finished` promise and, on the rejection
//! path (an animation canceled mid-flight), retries the whole body while any current
//! animation is still pending or unfinished, else completes (`:99-127`); with
//! `waitForStartingStyleRemoved` the first check is deferred until the
//! `[data-starting-style]` attribute is gone — one fallback frame when it is not present,
//! else a `MutationObserver` on the attribute with the abort listener disconnecting it
//! (`:129-154`); otherwise one frame before the first check (`:156`). `done()` runs the
//! callback in `flushSync` by default — each callback its own commit so later completions
//! observe earlier updates (`:73-81`) — or, with `batch: true`, hands it to the module-level
//! `flushBeforePaint` that coalesces every callback queued within the same microtask
//! checkpoint into one commit, re-checking each entry's abort signal at flush time
//! (`:9-32`, `:83-88`).
//!
//! Spec: `specs/library/internals/implementation.md` ("Element rendering
//! (`useRenderElement`) and transition state" — the mechanism summary, verified against the
//! source) and `specs/library/internals/behavior.md` ("State model" and "Keyboard
//! interactions" — the observable matrix, which is upstream's
//! `useAnimationsFinished.test.tsx`).
//!
//! ## Rust adaptations
//!
//! - `elementOrRef` (`:45`) is a source closure returning the element, read per `run()`
//!   invocation — the `resolveRef` per-call resolution (`:66`); the maybe-ref union has no
//!   Rust shape, so the "if ref, its current value" branch dissolves into the caller's
//!   closure (the `use_composite_list_item.rs` element-slot convention).
//! - `useStableCallback` (`:51`) is N/A: the returned [`RunOnceAnimationsFinish`] is one
//!   value per hook call, so its identity is stable by construction (the
//!   `use_value_changed.rs` convention).
//! - `waitForStartingStyleRemoved`/`batch` are reactive sources read (untracked) per
//!   `run()` invocation — upstream reads the render-scope values through the stable
//!   trampoline, which is the same latest-value semantics.
//! - `flushSync` (`:79`, `:24`) has no Leptos counterpart: the reactive graph applies
//!   writes synchronously, so a direct call already satisfies "runs before the browser can
//!   paint" and "later completions observe earlier updates" (each callback's signal writes
//!   are visible to the next callback's reads). The batch mode keeps upstream's
//!   coalescing *structure* — a module-level queue plus one flush per microtask checkpoint
//!   (`:18-32`) — ported to a thread-local with the `composite_list.rs` scheduler pattern:
//!   `window.queueMicrotask` on wasm, a test-installed override or a synchronous run on
//!   host. The upstream "single commit" is not observable in the port (no commit
//!   boundary); what the host suite pins is the coalescing contract itself.
//! - `globalThis.BASE_UI_ANIMATIONS_DISABLED` (`:93`) is read from the JS global on wasm;
//!   host builds have no JS realm and never reach the check (the element source is `None`
//!   there, the early return at `:67-69`).
//! - Promise handlers and the starting-style observer use `Closure::once_into_js` — the
//!   JS function is owned by the JS GC and reclaimed when the promise chain settles /
//!   the observer disconnects, upstream's exact lifetime (a leaked closure would
//!   otherwise outlive the page; a Rust-owned one would need explicit teardown for a
//!   browser-managed lifetime).
//! - `exec()`'s re-read of `getAnimations` (`:111`) fail-soft maps a vanished accessor to
//!   an empty list (an empty `Promise.all` resolves — the done path); upstream would throw
//!   a `TypeError` into the promise rejection handler there, whose next step is the same
//!   done() — the arm is unreachable through the public surface anyway because the
//!   immediate-call check at `:91` guarantees a callable accessor before any `exec()`.
//! - `getAnimations` is invoked through a dynamic property read (not the web-sys IDL
//!   binding) — the wasm-suite monkey-patch contract (`useAnimationsFinished.test.tsx:40-44`),
//!   the same dynamic-lookup rule the composite suite records for `scrollTo`.
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::traits::{Get, GetUntracked};
use wasm_bindgen::JsCast;
use web_sys::Element;

use leptos_ui_utils::use_animation_frame::{AnimationFrame, use_animation_frame};

use crate::abort_signal::AbortSignal;
use crate::state_attributes::STARTING_STYLE;

/// The element source — upstream's `elementOrRef` parameter
/// (`useAnimationsFinished.ts:45`), resolved per [`RunOnceAnimationsFinish::run`]
/// invocation. `None` is upstream's `null` (the early return at `:67-69`).
pub type ElementSource = Rc<dyn Fn() -> Option<Element>>;

// The batched-callback registry (`useAnimationsFinished.ts:9`) — module-level upstream,
// thread-local in Rust. `None` is "no flush pending".
thread_local! {
    static PENDING_BATCH: RefCell<Option<Vec<PendingBatchCallback>>> = const { RefCell::new(None) };
}

/// One queued batch entry: the callback plus the signal whose aborted flag is re-checked
/// at flush time (`:84-88`).
struct PendingBatchCallback {
    run: Box<dyn FnOnce()>,
    signal: Option<AbortSignal>,
}

// The host flush scheduler override — the `composite_list.rs` scheduler pattern (which
// follows the `timeout_manager.rs` dispatch-override precedent): host builds have no
// microtask queue, so tests install a manual scheduler to pin the coalescing contract
// deterministically. Absent an override the host flush runs synchronously.
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static BATCH_FLUSH_SCHEDULER_OVERRIDE: RefCell<Option<Rc<dyn Fn(Box<dyn FnOnce()>)>>> =
        const { RefCell::new(None) };
}

/// Installs (or clears) the host batch-flush scheduler override. wasm builds schedule
/// through `window.queueMicrotask` and have no override.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_batch_flush_scheduler_override(scheduler: Option<Rc<dyn Fn(Box<dyn FnOnce()>)>>) {
    BATCH_FLUSH_SCHEDULER_OVERRIDE.with(|cell| *cell.borrow_mut() = scheduler);
}

/// `flushBeforePaint` (`useAnimationsFinished.ts:18-32`): the first callback of a
/// checkpoint creates the batch and schedules one flush; later callbacks in the same
/// checkpoint join it.
fn flush_before_paint(run: Box<dyn FnOnce()>, signal: Option<AbortSignal>) {
    let is_new_batch = PENDING_BATCH.with(|pending| {
        let mut pending = pending.borrow_mut();
        let is_new_batch = pending.is_none();
        pending
            .get_or_insert_with(Vec::new)
            .push(PendingBatchCallback { run, signal });
        is_new_batch
    });

    if is_new_batch {
        schedule_batch_flush();
    }
}

/// Schedules the coalesced flush (module docs): one microtask on wasm, the override or an
/// immediate run on host. The queue is emptied *before* the callbacks run
/// (`useAnimationsFinished.ts:23`), so a callback that completes another animation during
/// the flush starts a new batch.
fn schedule_batch_flush() {
    let flush = Box::new(|| {
        let batch = PENDING_BATCH.with(|pending| pending.borrow_mut().take());
        if let Some(batch) = batch {
            for callback in batch {
                // The flush-time abort re-check (`:84-88`).
                if !callback.signal.as_ref().is_some_and(AbortSignal::aborted) {
                    (callback.run)();
                }
            }
        }
    });

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let function =
                js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js(flush));
            window.queue_microtask(&function);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let scheduler = BATCH_FLUSH_SCHEDULER_OVERRIDE.with(|cell| cell.borrow().clone());
        match scheduler {
            Some(scheduler) => scheduler(flush),
            None => flush(),
        }
    }
}

/// `globalThis.BASE_UI_ANIMATIONS_DISABLED` (`useAnimationsFinished.ts:93`). wasm reads
/// the JS global; host builds never reach the check (module docs).
fn animations_disabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Reflect::get(
            &js_sys::global(),
            &wasm_bindgen::JsValue::from_str("BASE_UI_ANIMATIONS_DISABLED"),
        )
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// The `typeof resolvedElement.getAnimations !== 'function'` check
/// (`useAnimationsFinished.ts:91-92`) — a property read, not a call.
fn has_get_animations(element: &Element) -> bool {
    js_sys::Reflect::get(element, &wasm_bindgen::JsValue::from_str("getAnimations"))
        .map(|value| value.is_function())
        .unwrap_or(false)
}

/// The `resolvedElement.getAnimations()` call (`useAnimationsFinished.ts:100`, `:111`) —
/// a dynamic property read so an instance override is respected (module docs). A missing
/// accessor or non-array result fail-soft maps to an empty list.
fn get_animations(element: &Element) -> js_sys::Array {
    js_sys::Reflect::get(element, &wasm_bindgen::JsValue::from_str("getAnimations"))
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
        .and_then(|function| function.call0(element).ok())
        .and_then(|value| value.dyn_into::<js_sys::Array>().ok())
        .unwrap_or_else(js_sys::Array::new)
}

/// `exec()` (`useAnimationsFinished.ts:99-127`): await every animation's `finished`
/// promise; on rejection (an animation canceled mid-flight), retry the whole body while
/// any current animation is still pending or unfinished, else complete.
fn exec(element: &Element, signal: Option<&AbortSignal>, done: &Rc<dyn Fn()>) {
    let animations = get_animations(element);
    let finished = js_sys::Array::new();
    for animation in animations.iter() {
        match js_sys::Reflect::get(&animation, &wasm_bindgen::JsValue::from_str("finished")) {
            // `animation.finished` (`:100`) — a promise to await.
            Ok(value) if value.is_instance_of::<js_sys::Promise>() => {
                finished.push(&value);
            }
            // A missing/broken `finished` never blocks: `Promise.all` over it would carry
            // the raw value through as a resolved entry.
            _ => {
                finished.push(&js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED));
            }
        };
    }

    let all = js_sys::Promise::all(&finished);

    // The fulfillment arm (`:101-105`). The handlers are `Closure::once_into_js` —
    // JS-GC-owned, reclaimed when the chain settles (module docs).
    let on_fulfilled = js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js({
        let signal = signal.cloned();
        let done = Rc::clone(done);
        move |_: wasm_bindgen::JsValue| {
            if !signal.as_ref().is_some_and(AbortSignal::aborted) {
                done();
            }
        }
    }));

    // The rejection arm (`:106-125`): an animation was canceled — re-check the current
    // animations and retry while any of them is still pending or unfinished.
    let on_rejected = js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js({
        let element = element.clone();
        let signal = signal.cloned();
        let done = Rc::clone(done);
        move |_: wasm_bindgen::JsValue| {
            if signal.as_ref().is_some_and(AbortSignal::aborted) {
                return;
            }

            let current_animations = get_animations(&element);
            let still_running = current_animations.iter().any(|animation| {
                let pending =
                    js_sys::Reflect::get(&animation, &wasm_bindgen::JsValue::from_str("pending"))
                        .ok()
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);
                let play_state =
                    js_sys::Reflect::get(&animation, &wasm_bindgen::JsValue::from_str("playState"))
                        .ok()
                        .and_then(|value| value.as_string());
                pending || play_state.as_deref() != Some("finished")
            });

            if still_running {
                exec(&element, signal.as_ref(), &done);
                return;
            }

            done();
        }
    }));

    // `Promise.all(...).then(onFulfilled, onRejected)` (`:100-126`) — called dynamically
    // so both handlers are JS-GC-owned values; `then2` would require Rust-side ownership
    // of the handlers, which outlives the chain (module docs).
    if let Some(then) = js_sys::Reflect::get(&all, &wasm_bindgen::JsValue::from_str("then"))
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
    {
        let _ = then.call2(&all, &on_fulfilled, &on_rejected);
    }
}

/// The stable runner upstream returns (`useAnimationsFinished.ts:51-158`) — one value per
/// hook call, stable by construction. Clone-free: the hook hands it over once.
pub struct RunOnceAnimationsFinish {
    frame: AnimationFrame,
    element: ElementSource,
    wait_for_starting_style_removed: Rc<dyn Fn() -> bool>,
    batch: Rc<dyn Fn() -> bool>,
}

impl RunOnceAnimationsFinish {
    /// The runner invocation (`useAnimationsFinished.ts:52-157`): `fnToExecute` plus the
    /// optional abort signal. A `None` element no-ops (`:67-69`).
    pub fn run(&self, fn_to_execute: impl FnOnce() + 'static, signal: Option<&AbortSignal>) {
        self.frame.cancel();

        let Some(element) = (self.element)() else {
            return;
        };

        // `done` (`:73-89`). The callback is consumed exactly once — the retries of the
        // promise chain share this slot, and only one arm of the settled chain reaches it.
        let done: Rc<dyn Fn()> = {
            let fn_to_execute = RefCell::new(Some(Box::new(fn_to_execute) as Box<dyn FnOnce()>));
            let batch = (self.batch)();
            let signal = signal.cloned();
            Rc::new(move || {
                let Some(fn_to_execute) = fn_to_execute.borrow_mut().take() else {
                    return;
                };
                if !batch {
                    // The default path (`:74-81`): run now — see the flushSync adaptation
                    // in the module docs. Each callback gets its own commit upstream; in
                    // the port every write is already synchronously visible to the next
                    // callback's reads.
                    fn_to_execute();
                    return;
                }

                flush_before_paint(
                    Box::new(fn_to_execute),
                    // Re-check at flush time (`:84-88`).
                    signal.clone(),
                );
            })
        };

        // The immediate-call arm (`:91-97`).
        if !has_get_animations(&element) || animations_disabled() {
            done();
            return;
        }

        if (self.wait_for_starting_style_removed)() {
            let starting_style_attribute = STARTING_STYLE;

            // Without the attribute, one more frame gives the "open" animations a chance
            // to register (`:132-137`).
            if !element.has_attribute(starting_style_attribute) {
                let element = element.clone();
                let signal = signal.cloned();
                self.frame
                    .request(move || exec(&element, signal.as_ref(), &done));
                return;
            }

            // Wait for the attribute to have been removed (`:139-150`). The observer
            // callback is `Closure::once_into_js` — JS-owned, reclaimed with the
            // disconnected observer (module docs).
            let observer_element = element.clone();
            let observer_signal = signal.cloned();
            let on_mutations =
                wasm_bindgen::JsValue::from(wasm_bindgen::closure::Closure::once_into_js(
                    move |_: js_sys::Array, observer: web_sys::MutationObserver| {
                        if !observer_element.has_attribute(starting_style_attribute) {
                            observer.disconnect();
                            exec(&observer_element, observer_signal.as_ref(), &done);
                        }
                    },
                ));
            if let Ok(observer) = web_sys::MutationObserver::new(on_mutations.unchecked_ref()) {
                let mut init = web_sys::MutationObserverInit::new();
                init.set_attributes(true);
                init.set_attribute_filter(&js_sys::Array::of1(&starting_style_attribute.into()));
                let _ = observer.observe_with_options(&element, &init);

                // The abort signal disconnects the observer (`:152`).
                if let Some(signal) = signal {
                    signal.add_abort_listener(move || observer.disconnect());
                }
            }
            return;
        }

        // One frame before the first check (`:156`).
        self.frame.request({
            let element = element.clone();
            let signal = signal.cloned();
            move || exec(&element, signal.as_ref(), &done)
        });
    }
}

/// The upstream `useAnimationsFinished` (`useAnimationsFinished.ts:44-158`). The element
/// source is read per `run()` invocation; the two flags are reactive sources read
/// (untracked) per invocation — the latest-value semantics of upstream's per-render
/// closures. Must be called inside a reactive owner (the animation-frame hook).
pub fn use_animations_finished<E, W, B>(
    element: E,
    wait_for_starting_style_removed: W,
    batch: B,
) -> RunOnceAnimationsFinish
where
    E: Fn() -> Option<Element> + 'static,
    W: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    B: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    let frame = use_animation_frame();

    RunOnceAnimationsFinish {
        frame,
        element: Rc::new(element),
        wait_for_starting_style_removed: Rc::new({
            let wait_for_starting_style_removed = wait_for_starting_style_removed.clone();
            move || wait_for_starting_style_removed.get_untracked()
        }),
        batch: Rc::new(move || batch.get_untracked()),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // The host scheduler override — tests play the role of the microtask checkpoint (the
    // `composite_list.rs` test pattern). The override collects the flush work items
    // instead of running them, so coalescing is countable and the flush is manual.
    fn install_collecting_scheduler() -> Rc<RefCell<Vec<Box<dyn FnOnce()>>>> {
        let work_queue: Rc<RefCell<Vec<Box<dyn FnOnce()>>>> = Rc::new(RefCell::new(Vec::new()));
        set_batch_flush_scheduler_override(Some(Rc::new({
            let work_queue = Rc::clone(&work_queue);
            move |work| work_queue.borrow_mut().push(work)
        })));
        work_queue
    }

    fn drain(work_queue: &Rc<RefCell<Vec<Box<dyn FnOnce()>>>>) {
        let work_items: Vec<_> = work_queue.borrow_mut().drain(..).collect();
        for work in work_items {
            work();
        }
    }

    // Port-owned pin for the coalescing contract (`useAnimationsFinished.ts:18-32`):
    // callbacks queued within one checkpoint share ONE flush, run in queue order, and a
    // queued entry whose signal aborted before the flush is skipped (`:84-88`).
    #[test]
    fn coalesces_a_checkpoint_into_one_flush_in_order_and_skips_aborted_entries() {
        let work_queue = install_collecting_scheduler();

        let calls = Rc::new(RefCell::new(Vec::new()));
        let aborted_signal = AbortSignal::new();
        let live_signal = AbortSignal::new();

        flush_before_paint(
            Box::new({
                let calls = Rc::clone(&calls);
                move || calls.borrow_mut().push("first")
            }),
            None,
        );
        flush_before_paint(
            Box::new({
                let calls = Rc::clone(&calls);
                move || calls.borrow_mut().push("second")
            }),
            Some(live_signal),
        );
        flush_before_paint(
            Box::new({
                let calls = Rc::clone(&calls);
                move || calls.borrow_mut().push("aborted")
            }),
            Some(aborted_signal.clone()),
        );

        // One checkpoint, one scheduled flush.
        assert_eq!(work_queue.borrow().len(), 1);

        aborted_signal.abort();
        drain(&work_queue);

        assert_eq!(
            &*calls.borrow(),
            &["first", "second"],
            "queue order, abort re-checked at flush time"
        );

        // After the flush the registry is empty (`:23`), so the next callback starts a
        // new batch.
        flush_before_paint(
            Box::new({
                let calls = Rc::clone(&calls);
                move || calls.borrow_mut().push("third")
            }),
            None,
        );
        assert_eq!(
            work_queue.borrow().len(),
            1,
            "a fresh batch is scheduled after the flush"
        );
        drain(&work_queue);
        assert_eq!(&*calls.borrow(), &["first", "second", "third"]);
    }

    // Port-owned pin for the re-entrancy rule (`:22-24`): the queue is emptied before the
    // callbacks run, so a callback that completes another animation during the flush
    // starts a NEW batch rather than appending to the one being drained.
    #[test]
    fn a_callback_queued_during_the_flush_starts_a_new_batch() {
        let work_queue = install_collecting_scheduler();

        let ran_inner = Rc::new(Cell::new(false));
        flush_before_paint(
            {
                let ran_inner = Rc::clone(&ran_inner);
                Box::new(move || {
                    flush_before_paint(
                        Box::new({
                            let ran_inner = Rc::clone(&ran_inner);
                            move || ran_inner.set(true)
                        }),
                        None,
                    );
                })
            },
            None,
        );

        assert_eq!(work_queue.borrow().len(), 1);
        drain(&work_queue);

        assert_eq!(
            work_queue.borrow().len(),
            1,
            "the inner flush_before_paint scheduled a fresh batch"
        );
        assert!(!ran_inner.get());
        drain(&work_queue);
        assert!(
            ran_inner.get(),
            "the new batch flushed on its own checkpoint"
        );
    }

    // Port-owned pin for the host default (module docs): absent an override the flush
    // runs synchronously — no scheduler to wait on.
    #[test]
    fn the_host_default_flushes_synchronously_without_an_override() {
        set_batch_flush_scheduler_override(None);

        let ran = Rc::new(Cell::new(false));
        flush_before_paint(
            Box::new({
                let ran = Rc::clone(&ran);
                move || ran.set(true)
            }),
            None,
        );

        assert!(ran.get());
    }

    // Mirrors the element resolution early return
    // (`useAnimationsFinished.ts:66-69`): a `None` element no-ops without touching the
    // frame scheduler or the callback.
    #[test]
    fn a_null_element_no_ops() {
        let owner = in_owner();

        let called = Rc::new(Cell::new(false));
        let runner = use_animations_finished(|| None, RwSignal::new(false), RwSignal::new(false));

        let called_for_callback = Rc::clone(&called);
        runner.run(move || called_for_callback.set(true), None);

        assert!(!called.get(), "the callback never fires without an element");

        owner.cleanup();
    }
}
