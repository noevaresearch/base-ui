//! Port of `packages/react/src/utils/useIsHydrating.ts` — the hydration-pass flag
//! `PrehydrationScript` gates on (the module's only internal dependency, per
//! `specs/library/internals/implementation.md`, "Dependencies on other Base UI internals":
//! `utils/useIsHydrating` (`packages/react/src/internals/PrehydrationScript.tsx:3`)).
//!
//! Upstream (22 lines) is one hook built on the `useSyncExternalStore` shim
//! (`:1`, `:20-22`):
//!
//! - `subscribe` is a no-op — there is no external store to subscribe to (`:4-6`), so the
//!   value never changes on its own; only React's own render phases read the two
//!   snapshots.
//! - `getServerSnapshot` returns `true` (`:12-14`) — the value React uses for server
//!   rendering *and* for the hydration render, whose output must match the server markup.
//! - `getSnapshot` returns `false` (`:8-10`) — the value for every fresh client-only
//!   mount. After hydration React's store shim detects the server/client snapshot
//!   mismatch and forces one post-mount re-render, so the observable sequence during
//!   hydration is exactly one `true` render pass followed by `false` — the flip the
//!   `PrehydrationScript` doc comment describes ("Once `isHydrating` flips to `false`
//!   the element unmounts", `packages/react/src/internals/PrehydrationScript.tsx:20-21`).
//! - The contract, verbatim from the doc comment (`:16-19`): "`true` while React is
//!   hydrating server-rendered markup and `false` for fresh client-only mounts."
//!
//! ## Rust adaptations
//!
//! - **React's runtime becomes an explicit global flag.** Upstream the value is supplied
//!   by React itself: `useSyncExternalStore`'s two snapshots are resolved by the
//!   reconciler at the right moments, and no user code toggles anything. The
//!   reactive-graph-only crate has no reconciler, so the same observable contract is
//!   carried by a module-level [`ArcRwSignal<bool>`] that the future SSR/hydration
//!   runtime (the Phase C `docs-app`/`leptos-ui` shell) drives: `true` around a
//!   server-render pass or a hydration pass, `false` otherwise. The flag is
//!   app-long-lived, like the runtime it stands in for — the
//!   `DISPATCH_OVERRIDE`/flush-scheduler thread-local precedent
//!   (`timeout_manager.rs`/`composite_list.rs`). [`set_is_hydrating`] is that runtime's
//!   seam; nothing in the crate calls it, exactly as nothing upstream calls the
//!   snapshots. The Arc-backed storage is deliberate: a plain `RwSignal` registers its
//!   storage with whichever owner is current at first access, so a lazily created
//!   global would be disposed together with that owner and panic every later reader
//!   (observed as a wasm-only latent failure — the host's Arc storage has no
//!   owner-registered `ArenaItem` to dispose, so only the browser suite caught it).
//!   `ArcRwSignal` is pure `Arc`/`RwLock` state with no owner registration — the global
//!   outlives every test owner and every component, which is the contract.
//! - **The flip is reactive instead of a re-render.** Upstream's post-hydration
//!   `false` arrives as one forced re-render; here the flag is a real signal, so
//!   the same flip is a signal write — consumers reading it inside a reactive scope
//!   re-run, which is what unmounts the prehydration script element (the port of the
//!   `if (!isHydrating) return null` branch re-evaluating). [`use_is_hydrating`] hands
//!   out a read-only [`Signal<bool>`] so a view-layer conditional can track it without
//!   being able to write it.
//! - **The default is `false`.** The crate currently runs in the client-only world —
//!   upstream's `getSnapshot` (`:8-10`) — and every test and consumer today mounts fresh.
//!   A default of `true` would make every consumer render its prehydration element on
//!   plain client mounts, which is precisely the behavior the upstream contract rules
//!   out.
//! - `useSyncExternalStore`'s subscribe half is dropped entirely: there is no external
//!   store, and reactive-graph subscriptions ride the signal reads themselves.
//! - `'use client'` (the shim import side) is N/A — no React Server Components boundary
//!   in Rust.

use reactive_graph::signal::ArcRwSignal;
use reactive_graph::traits::Set;
use reactive_graph::wrappers::read::Signal;

thread_local! {
    /// The module-level hydration flag — upstream's server/client snapshot pair, resolved
    /// by the future SSR/hydration runtime through [`set_is_hydrating`]. Default `false`:
    /// the fresh-client-mount world (`useIsHydrating.ts:8-10`); see the module docs. The
    /// Arc-backed storage keeps the flag out of every owner's disposal set (see the
    /// module docs).
    static IS_HYDRATING: ArcRwSignal<bool> = ArcRwSignal::new(false);
}

/// Reads the hydration flag — upstream's `useIsHydrating`
/// (`packages/react/src/utils/useIsHydrating.ts:20-22`). `true` during a server-render or
/// hydration pass, `false` for fresh client-only mounts. The returned signal is tracked:
/// a reactive scope reading it re-runs when the runtime flips the flag, which is the
/// post-hydration unmount path ([`crate::prehydration_script`]). Unlike the other hook
/// ports this needs no reactive owner — it reads a module-level signal, the way upstream
/// needs no component state.
pub fn use_is_hydrating() -> Signal<bool> {
    IS_HYDRATING.with(|flag| Signal::from(flag.clone()))
}

/// Drives the hydration flag — the runtime seam standing in for React resolving
/// `useSyncExternalStore`'s snapshot pair (`useIsHydrating.ts:8-14`). The future
/// SSR/hydration shell sets `true` around a server-render or hydration pass and `false`
/// once the pass is done; the write notifies subscribers, which is the port of
/// upstream's forced post-hydration re-render. Nothing in the crate calls this — the
/// caller is the app shell, the same way nothing upstream toggles the snapshots.
pub fn set_is_hydrating(hydrating: bool) {
    IS_HYDRATING.with(|flag| flag.set(hydrating));
}

/// Resets the global flag on drop — the test-only guard keeping the app-long-lived
/// module state from leaking between wasm cases (each host test case guards itself
/// locally; the wasm suites share this one).
#[cfg(test)]
pub(crate) struct ResetIsHydrating;

#[cfg(test)]
impl ResetIsHydrating {
    pub(crate) fn new() -> Self {
        set_is_hydrating(false);
        Self
    }
}

#[cfg(test)]
impl Drop for ResetIsHydrating {
    fn drop(&mut self) {
        set_is_hydrating(false);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{GetUntracked, Track};

    use super::*;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Resets the global flag around each test so the module-level state cannot leak
    // between cases (the flag is app-long-lived by design).
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            set_is_hydrating(false);
        }
    }

    // Pins the default (`useIsHydrating.ts:8-10`): with no runtime pass in flight the
    // hook reports `false` — the fresh client-only mount world every current consumer
    // runs in.
    #[test]
    fn the_default_is_false() {
        let _reset = Reset;

        assert!(
            !use_is_hydrating().get_untracked(),
            "no hydration pass in flight — the flag is false"
        );
    }

    // Pins the contract (`useIsHydrating.ts:16-19`) end to end through the runtime seam:
    // the flag reads `true` during a pass and `false` once it ends, and the flip is
    // observable — upstream's forced post-hydration re-render.
    #[test]
    fn the_runtime_flips_the_flag_through_a_pass() {
        let _reset = Reset;

        set_is_hydrating(true);
        assert!(
            use_is_hydrating().get_untracked(),
            "true while the hydration pass is in flight"
        );

        set_is_hydrating(false);
        assert!(
            !use_is_hydrating().get_untracked(),
            "false once the pass ends"
        );
    }

    // Pins the read-only surface: `use_is_hydrating` hands out a signal consumers can
    // track (a reactive scope re-runs on the flip — the post-hydration unmount path) but
    // the returned handle is the read half only.
    #[test]
    fn the_returned_signal_is_read_only_and_tracks_the_flag() {
        // Plain-effect runs land on the `futures_executor` `LocalPool`, which only
        // advances when polled (the use_anchor_positioning.rs host-suite pattern).
        let _ = any_spawner::Executor::init_futures_executor();
        let _owner = owner();
        let _reset = Reset;

        let reads = std::rc::Rc::new(std::cell::Cell::new(0));
        {
            let signal = use_is_hydrating();
            let reads = std::rc::Rc::clone(&reads);
            let _effect = Effect::new(move |_| {
                signal.track();
                reads.set(reads.get() + 1);
            });
            any_spawner::Executor::poll_local();
        }

        assert_eq!(reads.get(), 1, "the effect ran once on the initial value");

        set_is_hydrating(true);
        any_spawner::Executor::poll_local();
        assert_eq!(reads.get(), 2, "the flip to true re-ran the tracked effect");

        set_is_hydrating(false);
        any_spawner::Executor::poll_local();
        assert_eq!(reads.get(), 3, "the flip back re-ran the tracked effect");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::traits::GetUntracked;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // The same contract as the host tests, exercised under the browser's reactive-graph
    // runtime (the realm where the real components will run — the loop's wasm-only
    // latent failures are why both targets are pinned).
    #[wasm_bindgen_test]
    fn the_default_is_false_and_the_runtime_flips_the_flag() {
        let _guard = ResetIsHydrating::new();

        assert!(
            !use_is_hydrating().get_untracked(),
            "fresh client-only mount: false"
        );

        set_is_hydrating(true);
        assert!(
            use_is_hydrating().get_untracked(),
            "true while the hydration pass is in flight"
        );

        set_is_hydrating(false);
        assert!(!use_is_hydrating().get_untracked(), "false after the pass");
    }
}
