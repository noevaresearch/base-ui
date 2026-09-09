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
