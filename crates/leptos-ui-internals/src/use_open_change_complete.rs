//! Port of `packages/react/src/internals/useOpenChangeComplete.tsx` — the thin effect
//! wrapper that calls a function when the CSS open/close animation or transition
//! completes (`TODO.md`, item `infra: internals`; the transition/animation checkpoint
//! recorded in that entry's note).
//!
//! Upstream composes [`crate::use_animations_finished`]: the `onComplete` callback is
//! stabilized (`useOpenChangeComplete.tsx:12`), the runner is built with `open` as the
//! `waitForStartingStyleRemoved` flag and the `batch` opt-in (`:13`), and one
//! `React.useEffect` per `[enabled, open, onComplete, runOnceAnimationsFinish]` change
//! runs the completion under a fresh `AbortController` whose cleanup aborts it
//! (`:15-27`) — the per-run cancellation that stops a stale close-completion from firing
//! after a reopen.
//!
//! Spec: `specs/library/internals/implementation.md` ("Element rendering
//! (`useRenderElement`) and transition state" — "a thin effect wrapper adding per-run
//! `AbortController` cancellation"; also listed under untested behavior, item 2 — no
//! upstream unit test exercises it directly).
//!
//! ## Rust adaptations
//!
//! - The parameters object (`:30-54`) ports to a plain struct; upstream's destructuring
//!   defaults (`enabled = true` at `:35`, `batch = false` at `:49`) are documented per
//!   field and the caller supplies them. `open`/`batch`/`enabled` are reactive sources —
//!   the render-prop analog; `reference` is the element source read per runner
//!   invocation.
//! - `useStableCallback` (`:12`) is N/A: the callback arrives as an `Rc`, whose clone
//!   identity is stable by construction (the `use_value_changed.rs` convention), and the
//!   effect's per-run payload clones it — upstream's dep entry is stable for the same
//!   reason.
//! - The effect (`:15-27`) ports to a [`reactive_graph::effect::Effect`] tracking the
//!   reactive sources — `enabled`/`open` are read inside the body (the dep array), so an
//!   open flip or an enable/disable re-runs it. The abort cleanup ports to `on_cleanup`
//!   inside the effect body: each effect re-run first runs the previous run's cleanup
//!   (the reactive-graph `Owner::with_cleanup` order), aborting the stale run exactly
//!   like upstream's per-effect cleanup. The runner itself is stable (one value per hook
//!   call), so the remaining dep identity is N/A.
//! - `open` is read *tracked* in the effect body (the dep entry) and read *untracked* by
//!   the runner's `waitForStartingStyleRemoved` (the `use_animations_finished.rs`
//!   latest-value convention) — the same value reaches both, and the effect still
//!   re-runs per flip.
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.

use std::rc::Rc;

use reactive_graph::effect::Effect;
use reactive_graph::owner::on_cleanup;
use reactive_graph::traits::{Get, GetUntracked};
use send_wrapper::SendWrapper;

use crate::abort_signal::AbortSignal;
use crate::use_animations_finished::use_animations_finished;

/// The upstream `UseOpenChangeCompleteParameters` (`useOpenChangeComplete.tsx:30-54`).
pub struct UseOpenChangeCompleteParams<E, G, W, B> {
    /// Whether the hook is enabled (`:35` — upstream default `true`).
    pub enabled: G,
    /// Whether the element is open (`:39`) — also the runner's
    /// `waitForStartingStyleRemoved` flag (`:13`).
    pub open: W,
    /// Ref to the element being closed (`:43`) — the port's element source, read per
    /// runner invocation.
    pub reference: E,
    /// Whether completions ready in the same microtask may be coalesced into a single
    /// commit (`:49` — upstream default `false`; only safe when `onComplete` doesn't read
    /// state another completion can change).
    pub batch: B,
    /// Function to call when the animation completes (or there is no animation) (`:53`).
    pub on_complete: Rc<dyn Fn()>,
}

/// The upstream `useOpenChangeComplete` (`useOpenChangeComplete.tsx:9-28`). Must be
/// called inside a reactive owner (a component) — the effect outlives the call.
pub fn use_open_change_complete<E, G, W, B>(params: UseOpenChangeCompleteParams<E, G, W, B>)
where
    E: Fn() -> Option<web_sys::Element> + 'static,
    G: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    W: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    B: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    let UseOpenChangeCompleteParams {
        enabled,
        open,
        reference,
        batch,
        on_complete,
    } = params;

    // The runner (`:13`): `waitForStartingStyleRemoved = open`, the `batch` opt-in.
    let runner = use_animations_finished(reference, open.clone(), batch);

    // The per-run effect (`:15-27`).
    Effect::new(move |_| {
        let _enabled = enabled.get();
        let _open = open.get();

        // `on_cleanup` requires a `Send + Sync` closure; the handle is `Rc`-backed, so it
        // crosses the bound in a `SendWrapper` — the `use_media_query.rs` convention.
        let abort_signal = SendWrapper::new(AbortSignal::new());

        runner.run(
            {
                let on_complete = Rc::clone(&on_complete);
                move || on_complete()
            },
            Some(&abort_signal),
        );

        on_cleanup(move || abort_signal.abort());
    });
}

// The wasm/browser suite — the per-run cancellation against a real animation cycle
// (module docs: the effect's body and cleanup are browser-scheduled).
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::UnwrapThrowExt;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Element, HtmlElement};

    use super::*;

    fn owner() -> Owner {
        // The effect runs through the ambient executor (the `use_media_query.rs`
        // wasm-suite note); re-initializing returns `Err`.
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn document() -> web_sys::Document {
        web_sys::window()
            .expect_throw("no window")
            .document()
            .expect_throw("no document")
    }

    fn attach_div() -> HtmlElement {
        let element = document()
            .create_element("div")
            .expect_throw("create_element failed")
            .dyn_into::<HtmlElement>()
            .unwrap_throw();
        document()
            .body()
            .expect_throw("no body")
            .append_child(&element)
            .expect_throw("append_child failed");
        element
    }

    fn detach(element: &HtmlElement) {
        if let Some(parent) = element.parent_element() {
            let _ = parent.remove_child(element);
        }
    }

    async fn run_microtasks(ticks: usize) {
        for _ in 0..ticks {
            JsFuture::from(js_sys::Promise::resolve(&JsValue::undefined()))
                .await
                .unwrap();
        }
    }

    async fn await_frame() {
        let window = web_sys::window().expect_throw("no window");
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            window
                .request_animation_frame(&resolve)
                .expect_throw("requestAnimationFrame failed");
        });
        JsFuture::from(promise).await.unwrap();
    }

    /// The upstream `createAnimation` fixture (`useAnimationsFinished.test.tsx:9-27`).
    struct FakeAnimation {
        animation: js_sys::Object,
        resolve: js_sys::Function,
    }

    fn create_animation() -> FakeAnimation {
        let resolvers: Rc<RefCell<Option<js_sys::Function>>> = Rc::new(RefCell::new(None));
        let resolvers_for_promise = Rc::clone(&resolvers);
        let finished = js_sys::Promise::new(&mut move |resolve, _reject| {
            *resolvers_for_promise.borrow_mut() = Some(resolve);
        });
        let resolve = resolvers
            .borrow_mut()
            .take()
            .expect_throw("promise executor ran twice");

        let animation = js_sys::Object::new();
        js_sys::Reflect::set(&animation, &JsValue::from_str("finished"), &finished)
            .expect_throw("set finished");
        js_sys::Reflect::set(
            &animation,
            &JsValue::from_str("pending"),
            &JsValue::from_bool(false),
        )
        .expect_throw("set pending");
        js_sys::Reflect::set(
            &animation,
            &JsValue::from_str("playState"),
            &JsValue::from_str("running"),
        )
        .expect_throw("set playState");

        FakeAnimation { animation, resolve }
    }

    impl FakeAnimation {
        fn finish(&self) {
            self.resolve
                .call0(&JsValue::UNDEFINED)
                .expect_throw("resolve failed");
        }
    }

    fn patch_get_animations(
        element: &HtmlElement,
        animations: Rc<RefCell<Vec<js_sys::Object>>>,
        call_count: Rc<Cell<usize>>,
    ) -> wasm_bindgen::closure::Closure<dyn FnMut() -> js_sys::Array> {
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            call_count.set(call_count.get() + 1);
            let result = js_sys::Array::new();
            for animation in animations.borrow().iter() {
                result.push(animation);
            }
            result
        })
            as Box<dyn FnMut() -> js_sys::Array>);
        let function = js_sys::Function::from(closure.as_ref().clone());
        js_sys::Reflect::set(element, &JsValue::from_str("getAnimations"), &function)
            .expect_throw("patch getAnimations");
        closure
    }

    // Mirrors the mechanism behind `useAnimationsFinished.test.tsx:251-334` at the
    // `useOpenChangeComplete` layer: the reopen (an earlier completion flipping `open`)
    // re-runs the per-open effect, whose cleanup aborts the stale close-completion — so
    // the second popup's queued completion never unmounts it.
    #[wasm_bindgen_test]
    async fn the_reopen_aborts_the_stale_close_completion() {
        let owner = owner();

        let first = create_animation();
        let second = create_animation();
        let element_a = attach_div();
        let element_b = attach_div();

        let animations_a = Rc::new(RefCell::new(vec![first.animation.clone()]));
        let animations_b = Rc::new(RefCell::new(vec![second.animation.clone()]));
        let calls_a = Rc::new(Cell::new(0usize));
        let calls_b = Rc::new(Cell::new(0usize));
        let _patch_a = patch_get_animations(&element_a, animations_a, Rc::clone(&calls_a));
        let _patch_b = patch_get_animations(&element_b, animations_b, Rc::clone(&calls_b));

        // The reopen state (upstream's `secondOpen`) and the unmount guard's counter.
        let second_open = RwSignal::new(false);
        let on_second_unmount = Rc::new(Cell::new(0usize));

        // Popup A: permanently closing; its completion reopens popup B.
        use_open_change_complete(UseOpenChangeCompleteParams {
            enabled: RwSignal::new(true),
            open: RwSignal::new(false),
            reference: {
                let element_a = element_a.clone();
                move || Some(element_a.clone().unchecked_into::<Element>())
            },
            batch: RwSignal::new(false),
            on_complete: {
                let second_open = second_open.clone();
                Rc::new(move || second_open.set(true))
            },
        });

        // Popup B: open tracks the reopen signal; its completion only unmounts while
        // still closed (upstream's `if (!open) onComplete()` guard).
        use_open_change_complete(UseOpenChangeCompleteParams {
            enabled: RwSignal::new(true),
            open: second_open.clone(),
            reference: {
                let element_b = element_b.clone();
                move || Some(element_b.clone().unchecked_into::<Element>())
            },
            batch: RwSignal::new(false),
            on_complete: {
                let second_open = second_open.clone();
                let on_second_unmount = Rc::clone(&on_second_unmount);
                Rc::new(move || {
                    if !second_open.get_untracked() {
                        on_second_unmount.set(on_second_unmount.get() + 1);
                    }
                })
            },
        });

        // The two per-open effects arm their runners.
        any_spawner::Executor::poll_local();
        await_frame().await;
        run_microtasks(3).await;
        assert!(
            calls_a.get() > 0 && calls_b.get() > 0,
            "both runs armed their checks"
        );

        // Popup A completes, reopening popup B; the reactive effect re-runs (aborting
        // popup B's stale run and re-arming with open=true).
        first.finish();
        run_microtasks(4).await;
        any_spawner::Executor::poll_local();
        run_microtasks(3).await;
        await_frame().await;
        run_microtasks(3).await;
        assert!(
            second_open.get_untracked(),
            "popup A's completion reopened popup B"
        );
        assert!(
            calls_b.get() >= 2,
            "popup B's effect re-armed its check on the reopen"
        );

        // Popup B's animation finishes; the stale run must be aborted, the fresh run
        // must observe open=true and decline to unmount.
        second.finish();
        run_microtasks(6).await;
        assert_eq!(
            on_second_unmount.get(),
            0,
            "the stale completion was aborted by the reopen's effect cleanup"
        );

        detach(&element_a);
        detach(&element_b);
        owner.cleanup();
    }
}
