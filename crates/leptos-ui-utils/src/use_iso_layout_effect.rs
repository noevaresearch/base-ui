//! Port of `packages/utils/src/useIsoLayoutEffect.ts` (Base UI Phase A util).
//!
//! Upstream is a 6-line module: a `'use client'` directive, the React import, a private
//! module-local `noop` (`packages/utils/src/useIsoLayoutEffect.ts:4`), and a single
//! module-level constant export selecting between them
//! (`packages/utils/src/useIsoLayoutEffect.ts:6`):
//!
//! ```ts
//! export const useIsoLayoutEffect = typeof document !== 'undefined' ? React.useLayoutEffect : noop;
//! ```
//!
//! The binding is evaluated exactly once, when the module is first imported, and frozen for the
//! lifetime of the process — never re-evaluated per render or per call, so the binding cannot
//! switch at runtime (`specs/utils/useIsoLayoutEffect.md`, "State model"). In a browser realm it
//! is a full passthrough of `React.useLayoutEffect` with no wrapper logic; in a realm without a
//! `document` global (server rendering, non-DOM test environments) it is the private `noop`,
//! whose callback — and any cleanup it would have returned — is silently discarded
//! (`specs/utils/useIsoLayoutEffect.md`, "Public API surface", "Edge cases").
//!
//! Source of truth: **none.** The upstream repo has no test file for this unit
//! (`specs/utils/useIsoLayoutEffect.md`, "Source of truth"), so every upstream claim in this
//! module is a source-derived description of current implementation behavior, not test-proven
//! behavior. The tests below pin the ported contract itself.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The environment selection (`packages/utils/src/useIsoLayoutEffect.ts:6`) becomes a runtime
//!   probe of the global `document` property (`Reflect.get(globalThis, "document")`), resolved
//!   on first use and frozen in a [`thread_local!`] — the crate's established module-state
//!   pattern (the `test_utils` `is_jsdom` precedent), since Rust has no module-load hook. The
//!   probe mirrors the upstream `typeof document !== 'undefined'` semantics: an `undefined`
//!   value (the result for a missing property) selects the noop binding, while any other value —
//!   including `null`, whose `typeof` is `'object'` — selects the effect binding, exactly as the
//!   upstream ternary would. The host/SSR-server target has no JS realm at all, so its probe is
//!   constant `false`: the analog of upstream's bare-Node SSR environment. Because the snapshot
//!   is frozen on first resolution, injecting or removing a `document` global afterwards does
//!   not switch the binding (`specs/utils/useIsoLayoutEffect.md`, "Edge cases").
//! - The browser binding — upstream a passthrough of `React.useLayoutEffect`
//!   (`specs/utils/useIsoLayoutEffect.md`, "Public API surface") — becomes reactive_graph's
//!   [`RenderEffect`], the equivalent resolved **empirically**, not from docs alone
//!   (`specs/architecture.md`, "Layout effect"): its first run takes place immediately and
//!   synchronously during creation (the defining property of a render effect), so the callback
//!   runs by the time [`use_iso_layout_effect`] returns, matching `useLayoutEffect`'s
//!   run-before-paint contract for measurement/positioning. **Recorded distinction**
//!   (`specs/architecture.md`, "Layout effect"): subsequent re-runs are NOT same-call-stack
//!   synchronous — they resolve as earlier-queued microtasks through the ambient executor —
//!   unlike React's `useLayoutEffect`. If a specific consumer behavior ever turns out to depend
//!   on true same-call-stack synchronicity (not just pre-paint timing), revisit that case.
//! - Upstream's optional dependency array has no analog parameter: reactive tracking replaces
//!   it. The callback body runs inside the effect's observer, so reactive values it reads are
//!   tracked automatically and drive re-runs — the analog of listing them in the deps array. A
//!   callback that reads no reactive values runs exactly once, the analog of `[]` (upstream's
//!   "missing array ⇒ run every render" behavior has no port counterpart: a Leptos component
//!   body runs once, so there is no per-render re-invocation to mirror).
//! - Upstream's cleanup-function contract (a callback may return a cleanup, e.g.
//!   `packages/utils/src/store/ReactStore.ts:74-76`) becomes reactive_graph's [`on_cleanup`]
//!   idiom: cleanups registered inside the callback fire before each re-run (each re-run enters
//!   through the effect owner's cleanup) and once more when the effect is finally canceled. Note
//!   [`on_cleanup`] requires a `Send + Sync` closure — non-`Send` captures go through
//!   `SendWrapper`, the crate's `use_interval` precedent.
//! - Unmount: React owns the effect until unmount; the port keeps the [`RenderEffect`] alive in
//!   a [`StoredValue`] arena-allocated to the calling owner. Owner disposal drops it, which
//!   drops the effect's notification channel sender, which ends the executor task and drops the
//!   callback (`reactive_graph`'s channel wakes its receiver a final time when the sender
//!   drops). Must be called inside a reactive owner (a component): without one the
//!   [`StoredValue`] registers nowhere and the effect is never disposed — the port of React's
//!   invalid-hook-call error in a system that cannot throw.
//! - The noop binding (`packages/utils/src/useIsoLayoutEffect.ts:4`) is the non-DOM branch's
//!   body: no effect is created, no executor is touched, and the callback is dropped — the
//!   standard suppression for environments where layout effects cannot run
//!   (`specs/utils/useIsoLayoutEffect.md`, "Edge cases").
//! - The ambient executor: the [`RenderEffect`] re-run loop is spawned through `any_spawner`'s
//!   ambient executor, which must be initialized before the first browser-branch call
//!   (`leptos::mount` does this in production; tests use `Executor::init_futures_executor`).
//!   The first run itself is synchronous and does not wait on the executor.
//! - `'use client'` (`packages/utils/src/useIsoLayoutEffect.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.

use std::cell::Cell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use reactive_graph::effect::RenderEffect;
use reactive_graph::owner::StoredValue;

thread_local! {
    /// The frozen environment selection (`packages/utils/src/useIsoLayoutEffect.ts:6`):
    /// upstream's module-import-time ternary, resolved on first use and frozen for the life of
    /// the program (see the module docs). `None` is "not yet resolved".
    static DOCUMENT_PRESENCE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// The upstream `useIsoLayoutEffect` export
/// (`packages/utils/src/useIsoLayoutEffect.ts:6`): runs `callback` like a layout effect —
/// synchronously during setup, then again whenever reactive values the callback reads change —
/// in a DOM environment, and discards it entirely otherwise. Must be called inside a reactive
/// owner (a component) when the DOM binding is selected; see the module docs. UNVERIFIED
/// upstream — no test asserts the hook.
pub fn use_iso_layout_effect(mut callback: impl FnMut() + 'static) {
    if !document_global_exists() {
        return;
    }
    let effect = RenderEffect::new(move |_: Option<()>| callback());
    StoredValue::new(effect);
}

/// The frozen `typeof document !== 'undefined'` selection
/// (`packages/utils/src/useIsoLayoutEffect.ts:6`): resolves the probe once, freezes the answer,
/// and returns it for every later call.
fn document_global_exists() -> bool {
    DOCUMENT_PRESENCE.with(|snapshot| match snapshot.get() {
        Some(frozen) => frozen,
        None => {
            let resolved = probe_document_global();
            snapshot.set(Some(resolved));
            resolved
        }
    })
}

/// The runtime probe behind the frozen selection — upstream's `typeof document` read
/// (`packages/utils/src/useIsoLayoutEffect.ts:6`). A missing property reads as `undefined`,
/// which selects the noop binding; every other value — including `null`, whose `typeof` is
/// `'object'` — selects the effect binding, matching the upstream ternary exactly.
#[cfg(target_arch = "wasm32")]
fn probe_document_global() -> bool {
    matches!(
        js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("document")),
        Ok(value) if !value.is_undefined()
    )
}

/// The host/SSR-server probe: no JS realm exists on a native target, so `typeof document` can
/// only be `'undefined'` — the analog of upstream's bare-Node SSR environment.
#[cfg(not(target_arch = "wasm32"))]
fn probe_document_global() -> bool {
    false
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::owner::on_cleanup;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};

    use super::*;

    /// Restores the binding decision to "unresolved" after the test, so a later test on the
    /// same thread resolves it from its own environment. The snapshot is per-thread, so
    /// parallel test threads cannot interfere with each other.
    struct ResetBinding;

    impl Drop for ResetBinding {
        fn drop(&mut self) {
            DOCUMENT_PRESENCE.with(|snapshot| snapshot.set(None));
        }
    }

    /// Pre-resolves the frozen selection with an explicit answer — the host analog of stubbing
    /// the `document` global before the upstream module is first imported.
    fn resolve_document_presence(present: bool) {
        DOCUMENT_PRESENCE.with(|snapshot| snapshot.set(Some(present)));
    }

    fn counting_callback() -> (Rc<Cell<usize>>, impl FnMut() + 'static) {
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        (runs, move || counter.set(counter.get() + 1))
    }

    // Pins the non-DOM binding (`packages/utils/src/useIsoLayoutEffect.ts:4`, spec "Edge
    // cases" — UNVERIFIED upstream): with the environment selecting the noop binding, the
    // callback is silently discarded — nothing runs, no effect machinery is touched, no panic.
    #[test]
    fn the_noop_binding_discards_the_callback_without_running_it() {
        let _reset = ResetBinding;
        resolve_document_presence(false);

        let owner = Owner::new();
        owner.set();

        let (runs, callback) = counting_callback();
        use_iso_layout_effect(callback);
        assert_eq!(
            runs.get(),
            0,
            "the non-DOM binding is upstream's noop: the callback is discarded"
        );
    }

    // Pins the browser binding's defining property (architecture.md, "Layout effect" — the
    // empirically resolved decision): the first run is immediate and synchronous, so the
    // callback has run by the time the hook returns, before any executor poll.
    #[test]
    fn the_effect_binding_runs_the_callback_synchronously_during_setup() {
        let _reset = ResetBinding;
        resolve_document_presence(true);
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let (runs, callback) = counting_callback();
        use_iso_layout_effect(callback);
        assert_eq!(
            runs.get(),
            1,
            "the first run is immediate and synchronous — RenderEffect's defining property"
        );
    }

    // Pins the deps-array replacement (spec "Edge cases" — UNVERIFIED upstream): reactive
    // values read by the callback drive re-runs, and those re-runs are executor-timed rather
    // than same-call-stack synchronous — the recorded RenderEffect distinction
    // (architecture.md, "Layout effect").
    #[test]
    fn the_callback_re_runs_when_a_tracked_signal_changes_and_not_synchronously() {
        let _reset = ResetBinding;
        resolve_document_presence(true);
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let count = RwSignal::new(0);
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_iso_layout_effect(move || {
            let _tracked = count.get();
            counter.set(counter.get() + 1);
        });
        assert_eq!(runs.get(), 1);

        count.set(1);
        assert_eq!(
            runs.get(),
            1,
            "re-runs are executor-timed, not same-call-stack synchronous"
        );
        Executor::poll_local();
        assert_eq!(runs.get(), 2, "the tracked change re-runs the callback");
    }

    // Pins the empty-deps analog (spec "Edge cases" — UNVERIFIED upstream): a callback that
    // reads no reactive values runs exactly once, no matter what else changes.
    #[test]
    fn a_callback_that_reads_no_reactive_values_runs_exactly_once() {
        let _reset = ResetBinding;
        resolve_document_presence(true);
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let count = RwSignal::new(0);
        let (runs, callback) = counting_callback();
        use_iso_layout_effect(callback);
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        count.set(1);
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "no tracked reads, no re-run — the analog of an empty deps array"
        );
    }

    // Pins the cleanup-function contract (spec "Public API surface" — UNVERIFIED upstream;
    // consumer shape from `packages/utils/src/store/ReactStore.ts:69-77`): cleanups registered
    // inside the callback via `on_cleanup` fire before each re-run and once more when the
    // effect is finally canceled.
    #[test]
    fn cleanups_registered_inside_the_callback_run_before_re_runs_and_on_disposal() {
        let _reset = ResetBinding;
        resolve_document_presence(true);
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let cleanups = Arc::new(AtomicUsize::new(0));
        let count = RwSignal::new(0);
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        let cleanup_counter = Arc::clone(&cleanups);
        use_iso_layout_effect(move || {
            let _tracked = count.get();
            counter.set(counter.get() + 1);
            let cleanup_counter = Arc::clone(&cleanup_counter);
            on_cleanup(move || {
                cleanup_counter.fetch_add(1, Ordering::SeqCst);
            });
        });
        assert_eq!(runs.get(), 1);
        assert_eq!(cleanups.load(Ordering::SeqCst), 0);

        count.set(1);
        Executor::poll_local();
        assert_eq!(runs.get(), 2);
        assert_eq!(
            cleanups.load(Ordering::SeqCst),
            1,
            "the previous run's cleanup fires before the re-run"
        );

        owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            cleanups.load(Ordering::SeqCst),
            2,
            "the last run's cleanup fires when the effect is canceled"
        );
    }

    // Pins unmount cancellation (`specs/utils/useIsoLayoutEffect.md`, "Edge cases" — UNVERIFIED
    // upstream): after the owning reactive scope is disposed, the effect never re-runs.
    #[test]
    fn the_effect_stops_running_after_the_owner_is_disposed() {
        let _reset = ResetBinding;
        resolve_document_presence(true);
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let count = RwSignal::new(0);
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_iso_layout_effect(move || {
            let _tracked = count.get();
            counter.set(counter.get() + 1);
        });
        assert_eq!(runs.get(), 1);

        owner.cleanup();
        count.set(2);
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "the canceled effect never re-runs after owner disposal"
        );
    }

    // Pins the frozen selection (spec "State model" and "Edge cases" — UNVERIFIED upstream):
    // once resolved, the snapshot is returned as-is without re-probing the environment. The
    // host probe is constant `false` on this target, so the `true` answer here can only come
    // from the frozen snapshot.
    #[test]
    fn a_resolved_snapshot_is_returned_without_reprobing_the_environment() {
        let _reset = ResetBinding;
        resolve_document_presence(true);
        assert!(document_global_exists());
    }

    // Pins the host/SSR-server resolution: a native target has no JS realm, so the probe
    // resolves to the noop binding, and the resolution is frozen in the snapshot slot.
    #[test]
    fn a_host_environment_without_a_js_realm_resolves_to_the_noop_binding() {
        let _reset = ResetBinding;
        assert!(!document_global_exists());
        DOCUMENT_PRESENCE.with(|snapshot| {
            assert_eq!(
                snapshot.get(),
                Some(false),
                "the resolution is frozen in the snapshot slot"
            );
        });
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    // The behavior under test is a real DOM-environment binding decision and effect timing, so
    // the tests run in a real browser via the wasm32 test runner (`.cargo/config.toml` wires
    // it to chromedriver), like the crate's other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the browser-branch resolution (`packages/utils/src/useIsoLayoutEffect.ts:6` —
    // UNVERIFIED upstream): a real browser realm has a global `document`, so the effect
    // binding is selected, and the frozen snapshot is stable across repeated calls.
    #[wasm_bindgen_test]
    fn a_real_browser_realm_resolves_to_the_effect_binding() {
        assert!(
            document_global_exists(),
            "a browser realm has a global document — the effect binding"
        );
        assert!(document_global_exists(), "the snapshot is stable");
    }

    // Pins the synchronous first run in a real realm (architecture.md, "Layout effect").
    #[wasm_bindgen_test]
    fn the_browser_binding_runs_the_callback_synchronously_during_setup() {
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_iso_layout_effect(move || counter.set(counter.get() + 1));
        assert_eq!(
            runs.get(),
            1,
            "the first run is immediate and synchronous"
        );
    }

    // Pins the re-run timing in a real realm (architecture.md, "Layout effect" — the recorded
    // RenderEffect distinction): tracked changes re-run the callback, but not synchronously.
    #[wasm_bindgen_test]
    fn the_browser_binding_re_runs_on_tracked_changes_via_the_executor() {
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let count = RwSignal::new(0);
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_iso_layout_effect(move || {
            let _tracked = count.get();
            counter.set(counter.get() + 1);
        });
        assert_eq!(runs.get(), 1);

        count.set(1);
        assert_eq!(
            runs.get(),
            1,
            "re-runs are executor-timed, not same-call-stack synchronous"
        );
        Executor::poll_local();
        assert_eq!(runs.get(), 2, "the tracked change re-runs the callback");
    }

    // Pins unmount cancellation in a real realm: after owner disposal, the effect never
    // re-runs (`specs/utils/useIsoLayoutEffect.md`, "Edge cases" — UNVERIFIED upstream).
    #[wasm_bindgen_test]
    fn the_browser_binding_stops_running_after_the_owner_is_disposed() {
        let _ = Executor::init_futures_executor();

        let owner = Owner::new();
        owner.set();

        let count = RwSignal::new(0);
        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        use_iso_layout_effect(move || {
            let _tracked = count.get();
            counter.set(counter.get() + 1);
        });
        assert_eq!(runs.get(), 1);

        owner.cleanup();
        count.set(2);
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "the canceled effect never re-runs after owner disposal"
        );
    }
}
