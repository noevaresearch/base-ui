//! Port of `packages/react/src/internals/useTransitionStatus.ts` — the render-phase-derived
//! transition status machine (`TODO.md`, item `infra: internals`; the transition/animation
//! checkpoint recorded in that entry's note).
//!
//! Upstream owns: the `'starting' | 'ending' | 'idle' | undefined` status state seeded from
//! `open`/`enableIdleState` (`useTransitionStatus.ts:23-25`); the `mounted` state seeded
//! `open && !animateInitialOpen` (`:29`); the three render-phase conditionals that derive
//! writes from the current state during the render pass (`:31-42`: `open && !mounted` →
//! mount + `'starting'` in the same pass — React re-renders before committing, so the
//! element is still mounted in the same pass; `!open && mounted` → `'ending'` unless
//! already ending or `deferEndingState`; `!open && !mounted` with `'ending'` → back to
//! `undefined`); and three `useIsoLayoutEffect`s — the one-frame-deferred `'ending'` when
//! `deferEndingState` is set (`:44-56`), the one-frame-deferred clear to `undefined` when
//! idle state is off (`:58-72`, deliberately not `flushSync` — the Firefox note at `:64`),
//! and the idle machine that emits `'starting'` then `'idle'` one frame after open
//! (`:74-90`).
//!
//! Spec: `specs/library/internals/implementation.md` ("Element rendering
//! (`useRenderElement`) and transition state" — the mechanism summary, verified against the
//! source) and `specs/library/internals/behavior.md` ("State model" — the observable
//! matrix). Upstream has no unit test for this hook (implementation spec, "Anything in
//! source not explained by any test" item 2 — it is exercised only indirectly through the
//! popup component tests outside this unit); the port mirrors the mechanism the spec
//! documents and pins it with its own matrix.
//!
//! ## Rust adaptations
//!
//! - The four `useState` slots port to [`RwSignal`]s returned to the consumer (upstream's
//!   returned `setMounted` is the signal's `Set` impl — the `use_controlled` port's
//!   convention, where the exposed state handle is both readable and writable).
//! - The render-phase conditionals (`:31-42`) port to one [`use_iso_layout_effect`]
//!   tracking every input the block reads (`open`, `deferEndingState`, and the two
//!   states): the effect's setup run is the first render, and each re-run is a later
//!   render pass. In Leptos the reactive graph *is* the commit — every write lands before
//!   the flush yields to the browser, which is the observable content of upstream's
//!   "re-renders before committing" note (`:26-28`).
//! - React's same-value `setState` bail-out (an `Object.is`-equal write skips the
//!   re-render, so an effect whose deps include the state it writes does not re-fire) is
//!   reproduced by guarding every write with an inequality check. Without it the idle
//!   effect's own frame write (`:84`, always requested) would re-run the effect forever:
//!   reactive_graph notifies on every set, equal or not.
//! - The three layout effects (`:44-56`, `:58-72`, `:74-90`) port to
//!   [`use_iso_layout_effect`]s whose tracked reads are the upstream dep arrays; the
//!   `AnimationFrame.cancel` cleanups (`:50-52`, `:70-71`, `:87-89`) port to
//!   `on_cleanup` inside the effect body — each effect re-run first runs the cleanups of
//!   the previous run (the reactive-graph `Owner::with_cleanup` order), which is the
//!   upstream per-rerun cleanup semantics.
//! - `AnimationFrame.request`/`AnimationFrame.cancel` (`:46`, `:63`, `:83`) are the
//!   static-class form upstream, ported as the free functions
//!   [`leptos_ui_utils::use_animation_frame::request_animation_frame`]
//!   /`cancel_animation_frame` (not the [`AnimationFrame`](leptos_ui_utils::use_animation_frame::AnimationFrame)
//!   hook instance — upstream does not use the hook here).
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.

use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};

use leptos_ui_utils::use_animation_frame::{cancel_animation_frame, request_animation_frame};
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;

/// `TransitionStatus` (`useTransitionStatus.ts:6`) — `'starting' | 'ending' | 'idle'`.
/// `undefined` is `None` at the read sites (`Option<TransitionStatus>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionStatus {
    /// `'starting'` — the mount/open frame.
    Starting,
    /// `'ending'` — the close frame, before unmount.
    Ending,
    /// `'idle'` — one frame after open, when `enableIdleState` is set.
    Idle,
}

/// The upstream return shape (`useTransitionStatus.ts:92-96`): `{ mounted, setMounted,
/// transitionStatus }` — the port's [`RwSignal`]s are both (reading is `Get`, writing is
/// `Set`, so upstream's `setMounted` is the signal itself).
pub struct UseTransitionStatus {
    /// `mounted` (`:29`, `:32`) — whether the element should stay in the DOM for the
    /// exit-transition frames.
    pub mounted: RwSignal<bool>,
    /// `transitionStatus` (`:23-25`) — `None` is upstream's `undefined`.
    pub transition_status: RwSignal<Option<TransitionStatus>>,
}

/// The upstream `useTransitionStatus` (`useTransitionStatus.ts:17-97`). `open` (and the
/// two mode flags the effects re-read, `enableIdleState`/`deferEndingState`) are reactive
/// sources — the render-prop analog; `animateInitialOpen` is only read at hook time
/// (`:29`'s `useState` initializer), so it is a plain `bool`. Must be called inside a
/// reactive owner (a component).
pub fn use_transition_status<O, I, D>(
    open: O,
    enable_idle_state: I,
    defer_ending_state: D,
    animate_initial_open: bool,
) -> UseTransitionStatus
where
    O: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    I: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
    D: Get<Value = bool> + GetUntracked<Value = bool> + Clone + 'static,
{
    // The `useState` initializers (`:23-29`), read untracked — the first render's value.
    let mounted = RwSignal::new(open.get_untracked() && !animate_initial_open);
    let transition_status = RwSignal::new(
        if open.get_untracked() && enable_idle_state.get_untracked() {
            Some(TransitionStatus::Idle)
        } else {
            None
        },
    );

    // The render-phase conditionals (`:31-42`). Tracking every input the block reads is
    // the "runs on every render" semantics — including `mounted`/`transitionStatus`
    // themselves, so a consumer unmount (`setMounted(false)`) still re-runs the
    // `'ending'` → `undefined` cleanup; the setup run is the first render. The
    // same-value bail-out guards keep the tracked writes from looping (module docs).
    use_iso_layout_effect({
        let open = open.clone();
        let defer_ending_state = defer_ending_state.clone();
        move || {
            let open = open.get();
            let defer_ending_state = defer_ending_state.get();
            let mounted_now = mounted.get();
            let status_now = transition_status.get();

            if open && !mounted_now {
                mounted.set(true);
                set_status(&transition_status, Some(TransitionStatus::Starting));
            }

            if !open
                && mounted_now
                && status_now != Some(TransitionStatus::Ending)
                && !defer_ending_state
            {
                set_status(&transition_status, Some(TransitionStatus::Ending));
            }

            if !open && !mounted_now && status_now == Some(TransitionStatus::Ending) {
                set_status(&transition_status, None);
            }
        }
    });

    // The `deferEndingState` effect (`:44-56`): one frame later, `'ending'` — unless the
    // state moved on before the frame (the cleanup cancels the pending write, `:50-52`).
    // The tracked reads are the upstream dep array
    // (`[open, mounted, transitionStatus, deferEndingState]`).
    use_iso_layout_effect({
        let open = open.clone();
        move || {
            let open = open.get();
            if !open
                && mounted.get()
                && transition_status.get() != Some(TransitionStatus::Ending)
                && defer_ending_state.get()
            {
                let frame = request_animation_frame({
                    let transition_status = transition_status.clone();
                    move |_| set_status(&transition_status, Some(TransitionStatus::Ending))
                });
                on_cleanup(move || cancel_animation_frame(frame));
            }
        }
    });

    // The idle-off clear (`:58-72`): while open with idle state disabled, one frame after
    // any status the frame clears back to `undefined` (`:63-67` — deliberately not
    // `flushSync`, the Firefox note). Deps: `[enableIdleState, open]`.
    use_iso_layout_effect({
        let open = open.clone();
        let enable_idle_state = enable_idle_state.clone();
        move || {
            let open = open.get();
            if !open || enable_idle_state.get() {
                return;
            }

            let frame = request_animation_frame({
                let transition_status = transition_status.clone();
                move |_| set_status(&transition_status, None)
            });
            on_cleanup(move || cancel_animation_frame(frame));
        }
    });

    // The idle machine (`:74-90`): while open with idle state enabled, re-emit
    // `'starting'` when the current status is anything else (the keep-mounted reopen path:
    // `'ending'` → `'starting'` again), then `'idle'` one frame later. Deps:
    // `[enableIdleState, open, mounted, transitionStatus]`.
    use_iso_layout_effect({
        let open = open.clone();
        let enable_idle_state = enable_idle_state.clone();
        move || {
            let open = open.get();
            if !open || !enable_idle_state.get() {
                return;
            }

            if open && mounted.get() && transition_status.get() != Some(TransitionStatus::Idle) {
                set_status(&transition_status, Some(TransitionStatus::Starting));
            }

            let frame = request_animation_frame({
                let transition_status = transition_status.clone();
                move |_| set_status(&transition_status, Some(TransitionStatus::Idle))
            });
            on_cleanup(move || cancel_animation_frame(frame));
        }
    });

    UseTransitionStatus {
        mounted,
        transition_status,
    }
}

/// The React same-value bail-out analog (module docs): a write only when the value
/// differs. Reads untracked — the write sites are not re-deriving from the status, they
/// are committing a decided value.
fn set_status(status: &RwSignal<Option<TransitionStatus>>, next: Option<TransitionStatus>) {
    if status.get_untracked() != next {
        status.set(next);
    }
}

#[cfg(test)]
mod tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Port-owned pins for the `useState` initializers (`useTransitionStatus.ts:23-29`) —
    // the only part of the hook that runs on a host build (the layout effects are DOM
    // effects; the full matrix is the wasm suite's).
    #[test]
    fn initial_state_seeds_from_open_and_the_mode_flags() {
        let owner = in_owner();

        // Open at mount with idle state: `idle` + mounted (`:23-25`, `:29`).
        let status = use_transition_status(
            RwSignal::new(true),
            RwSignal::new(true),
            RwSignal::new(false),
            false,
        );
        assert!(status.mounted.get_untracked());
        assert_eq!(
            status.transition_status.get_untracked(),
            Some(TransitionStatus::Idle)
        );

        // Open at mount without idle state: no status, still mounted (no animation for
        // content open on the first render — the `animateInitialOpen` default).
        let status = use_transition_status(
            RwSignal::new(true),
            RwSignal::new(false),
            RwSignal::new(false),
            false,
        );
        assert!(status.mounted.get_untracked());
        assert_eq!(status.transition_status.get_untracked(), None);

        // Closed at mount: unmounted, no status.
        let status = use_transition_status(
            RwSignal::new(false),
            RwSignal::new(true),
            RwSignal::new(false),
            false,
        );
        assert!(!status.mounted.get_untracked());
        assert_eq!(status.transition_status.get_untracked(), None);

        // `animateInitialOpen` makes a first-render-open element go through `'starting'`:
        // mounted starts `false` (`open && !animateInitialOpen`).
        let status = use_transition_status(
            RwSignal::new(true),
            RwSignal::new(false),
            RwSignal::new(false),
            true,
        );
        assert!(!status.mounted.get_untracked());
        assert_eq!(status.transition_status.get_untracked(), None);

        owner.cleanup();
    }

    #[test]
    fn the_returned_signal_is_the_setmounted_handle() {
        let owner = in_owner();

        // Upstream's returned `setMounted` (`:93`) is the signal's `Set` impl: consumers
        // unmount by writing the signal directly.
        let status = use_transition_status(
            RwSignal::new(false),
            RwSignal::new(false),
            RwSignal::new(false),
            false,
        );
        status.mounted.set(true);
        assert!(status.mounted.get_untracked());

        owner.cleanup();
    }
}
