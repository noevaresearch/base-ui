//! Port of `packages/utils/src/useValueAsRef.ts` (Base UI Phase A util).
//!
//! Upstream is a 30-line hook: a public hook that seeds a stable ref object via
//! `useRefWithInit(createLatestRef, value)` (`packages/utils/src/useValueAsRef.ts:11`),
//! unconditionally writes the newest value into its `next` slot during render
//! (`packages/utils/src/useValueAsRef.ts:13`), registers a no-dependency layout effect that
//! copies `next` into `current` after every commit (`packages/utils/src/useValueAsRef.ts:15-16`
//! — the disabled `react-hooks/exhaustive-deps` lint is upstream's evidence that the
//! always-run sync is intentional), and returns the object
//! (`packages/utils/src/useValueAsRef.ts:18`); plus the module-private factory producing
//! `{ current: value, next: value, effect }` with an `effect` closure performing the
//! `next` → `current` copy (`packages/utils/src/useValueAsRef.ts:21-30`). The JSDoc states the
//! purpose: "Untracks the provided value by turning it into a ref to remove its reactivity" —
//! used to access the passed value inside `React.useEffect` without causing the effect to
//! re-run when the value changes (`packages/utils/src/useValueAsRef.ts:5-9`).
//!
//! The result is a one-commit lag between two slots of the same object: `next` is "the value
//! as of the latest render" and `current` is "the value as of the latest commit"
//! (`specs/utils/useValueAsRef.md`, "State model"). Every detected upstream call site reads
//! `current` inside event handlers and effects for exactly that untracked-read property —
//! e.g. `packages/react/src/internals/useAnchorPositioning.ts:179-180,262-268`,
//! `packages/react/src/tooltip/trigger/TooltipTrigger.tsx:133,219,226`,
//! `packages/react/src/slider/control/SliderControl.tsx:136` — and SliderControl additionally
//! *writes* `current` to pin it ahead of the next sync
//! (`packages/react/src/slider/control/SliderControl.tsx:224,302`).
//!
//! Source of truth: **none.** This unit has no dedicated test file — the generated manifest
//! lists `testFiles: []` (`ralph/generated/utils.json:399-406`) — so every upstream claim in
//! this module is a source-derived description of current implementation behavior, not
//! test-proven behavior (`specs/utils/useValueAsRef.md`, "Source of truth"). The spec
//! explicitly directs Stage 3 to encode these expectations as its own Rust tests rather than
//! binding to a reference suite; the tests below pin the ported contract itself.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The tracked value becomes a reactive source `V: Get<Value = T> + GetUntracked<Value = T>`
//!   — Leptos components run once and props are reactive sources, so "the value on every
//!   render" is a tracked read on every change (the `use_previous_value` port's convention,
//!   `crates/leptos-ui-utils/src/use_previous_value.rs`). The seed is an untracked read at
//!   hook-call time (`packages/utils/src/useValueAsRef.ts:11` — the factory argument, the
//!   first render's value).
//! - The upstream ref object's three own properties (`packages/utils/src/useValueAsRef.ts:22-28`)
//!   become the [`ValueAsRef`] handle (a `Copy` box handle, the crate's established ref-handle
//!   shape): `current` is a [`StoredValue`] — the crate's confirmed `useRef`-as-mutable-box
//!   mapping (`specs/architecture.md`, the state-management quick reference's `useRef` row;
//!   the `use_ref_with_init` port) — and the accessors `current()`/`next()`/`effect()`/
//!   `set_current()` mirror reading the properties, invoking `effect`, and assigning
//!   `current`. JS's shared-reference property assignment becomes an owned store write
//!   (`T: Clone`) — the `use_merged_refs` `RefObject` precedent ("the JS reference share
//!   becomes an owned store write").
//! - The two-phase write becomes two chained reactive steps, with `next` carried by an
//!   internal [`RwSignal`]: the render-phase write is a plain reactive_graph [`Effect`] that
//!   tracks the source and writes `next`; the commit-phase sync is a [`RenderEffect`] (through
//!   the ported `use_iso_layout_effect`) that tracks `next` and writes `current`. Chaining on
//!   the signal is what preserves upstream's freshness invariant — the sync can only ever
//!   re-run *after* the render-phase write of the same change has landed, so `next` is never
//!   staler than `current` and a manual `effect()` copy can only advance `current`, never
//!   regress it (`specs/utils/useValueAsRef.md`, "State model" one-commit lag, "Events"
//!   manual-invocation copy). Consumers read `next` untracked, so it stays as
//!   non-subscribing as upstream's plain property read. (The signal lives outside the
//!   `current` box deliberately: arena access is a process-wide `RwLock`, so a signal read
//!   inside a box-update closure would self-deadlock.)
//! - Render-phase timing: upstream writes `next` synchronously in the render body; the port's
//!   writer effect is a plain reactive_graph `Effect` whose runs are deferred to the ambient
//!   executor's next tick (reactive_graph spawns it with `spawn_local`). The write is
//!   unconditional either way — no equality check gates it
//!   (`packages/utils/src/useValueAsRef.ts:13`, spec "Edge cases") — and the first run is
//!   redundant with the seed exactly as upstream's first render is redundant with its factory.
//!   Unlike the sync below, the writer runs in *every* environment, matching upstream where
//!   the render-phase write also happens where layout effects do not.
//! - Commit-phase timing: the sync runs through [`use_iso_layout_effect`] — synchronous first
//!   run, re-runs as earlier-queued microtasks through the ambient executor (the empirically
//!   resolved `RenderEffect` decision and its recorded microtask-vs-synchronous distinction,
//!   `specs/architecture.md`, "Layout effect"). Its tracked read of `next` is the mechanism:
//!   upstream's no-dependency array means "run after every commit", and in the port "every
//!   commit" is every landed render-phase write (the disabled lint on
//!   `packages/utils/src/useValueAsRef.ts:15` marks that always-run intent). The public
//!   [`ValueAsRef::effect`] performs the identical copy but reads `next` **untracked**:
//!   upstream's effect body is not a tracking scope, and a tracked manual copy would silently
//!   subscribe any consumer effect invoking it to value changes — violating the JSDoc
//!   guarantee the hook exists for.
//! - Non-DOM environments: `use_iso_layout_effect` resolves to its noop binding when there is
//!   no `document` global (`packages/utils/src/useIsoLayoutEffect.ts:6`), so the per-commit
//!   sync never runs and `current` stays at the initial value while only `next` tracks updates
//!   (`specs/utils/useValueAsRef.md`, "Edge cases"). The host (native) target is such an
//!   environment.
//! - Consumer writes of `current` (the SliderControl pattern — pinning it ahead of the next
//!   sync) become [`ValueAsRef::set_current`]; the next commit-phase sync supersedes the
//!   manual write, as upstream's next commit does.
//! - Unmount: upstream registers no cleanup and tears nothing down
//!   (`packages/utils/src/useValueAsRef.ts:15-18`). The port's writer effect, sync effect, and
//!   boxes are all arena-allocated to the calling owner and canceled at its disposal; access
//!   through the handle afterwards panics rather than returning a still-referenced JS object
//!   (there is no GC to keep it alive) — the recorded reactive_graph boundary (the
//!   `use_ref_with_init` precedent). Must be called inside a reactive owner (a component):
//!   without one nothing registers anywhere and is never disposed — the port of React's
//!   invalid-hook-call error in a system that cannot throw. The ambient executor must be
//!   initialized before the writer's first deferred run (`leptos::mount` does this in
//!   production; tests use `Executor::init_futures_executor`).
//! - StrictMode double render and discarded concurrent renders
//!   (`specs/utils/useValueAsRef.md`, "Edge cases"): N/A — Leptos has no StrictMode and no
//!   re-running render bodies; the seed's synchronous single invocation replaces both
//!   concerns (the `use_ref_with_init` dissolution).
//! - `'use client'` (`packages/utils/src/useValueAsRef.ts:1`) is N/A — there is no React
//!   Server Components boundary in Rust.

use reactive_graph::effect::Effect;
use reactive_graph::owner::{LocalStorage, StoredValue};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, GetValue, Set, SetValue};

use crate::use_iso_layout_effect::use_iso_layout_effect;

/// The upstream `useValueAsRef` return object
/// (`packages/utils/src/useValueAsRef.ts:11-18`, factory `:21-30`): a `Copy` handle whose
/// reads and writes land on the same per-call-site instance for the handle's lifetime. `next`
/// is the value as of the latest render-equivalent, `current` the value as of the latest
/// commit-equivalent; both reads are non-subscribing. See the module docs.
#[derive(Clone, Copy)]
pub struct ValueAsRef<T> {
    /// The upstream `current` slot (`packages/utils/src/useValueAsRef.ts:23,26`) — the
    /// committed value.
    current: StoredValue<T, LocalStorage>,
    /// The upstream `next` slot (`packages/utils/src/useValueAsRef.ts:24`) — a signal so the
    /// commit-phase sync can track it (see the module docs); consumer reads are untracked.
    next: RwSignal<T, LocalStorage>,
}

impl<T: Clone + 'static> ValueAsRef<T> {
    /// Reads the upstream `current` slot (`packages/utils/src/useValueAsRef.ts:23,26`) — the
    /// value as of the latest commit-equivalent. Non-subscribing: reading it never causes a
    /// consumer effect to re-run (the JSDoc guarantee,
    /// `packages/utils/src/useValueAsRef.ts:5-9`). Panics after owner disposal — the recorded
    /// reactive_graph boundary (see the module docs).
    pub fn current(&self) -> T {
        self.current.get_value()
    }

    /// Reads the upstream `next` slot (`packages/utils/src/useValueAsRef.ts:24`) — the value
    /// as of the latest render-equivalent. Untracked, like upstream's plain property read.
    pub fn next(&self) -> T {
        self.next.get_untracked()
    }

    /// The upstream `effect` property (`packages/utils/src/useValueAsRef.ts:25-27`): performs
    /// the `next` → `current` copy — invoked automatically by the commit-phase sync, and
    /// invocable manually, where it can only advance `current` (see the module docs). Untracked.
    pub fn effect(&self) {
        let next = self.next.get_untracked();
        self.current.set_value(next);
    }

    /// The upstream `latest.current = value` assignment (consumed by SliderControl's
    /// pin-ahead writes, `packages/react/src/slider/control/SliderControl.tsx:224,302`):
    /// overwrites the committed value until the next commit-phase sync supersedes it.
    pub fn set_current(&self, current: T) {
        self.current.set_value(current);
    }
}

/// The upstream `useValueAsRef` hook (`packages/utils/src/useValueAsRef.ts:10-19`): turns a
/// reactive value into a stable two-slot ref — `next` tracks the source on every change, and
/// `current` catches up to it through a DOM-environment sync effect — so consumers can read
/// the newest value inside effects without subscribing to it. Seeded synchronously
/// (`current === next === value` before any executor tick). See the module docs for the
/// timing adaptations and the non-DOM behavior. UNVERIFIED upstream — the hook has no
/// dedicated test; the contract is pinned by this module's tests per
/// `specs/utils/useValueAsRef.md`, "Source of truth".
pub fn use_value_as_ref<T, V>(value: V) -> ValueAsRef<T>
where
    T: Clone + 'static,
    V: Get<Value = T> + GetUntracked<Value = T> + 'static,
{
    // The factory seed (`packages/utils/src/useValueAsRef.ts:21-30` — `useRefWithInit`
    // initializes once, on the first render): `current === next === value`, synchronously
    // available before the hook returns. The untracked read is the `use_previous_value`
    // port's seed convention.
    let seed = value.get_untracked();
    let current = StoredValue::new_local(seed.clone());
    let next = RwSignal::new_local(seed);

    // The render-phase write (`packages/utils/src/useValueAsRef.ts:13`): unconditional, on
    // every change of the source, in every environment. Runs are deferred to the executor's
    // next tick; the first run re-writes the seed (see the module docs). The handle is kept
    // in a box owned by the caller so owner disposal cancels the effect — the
    // `use_on_mount` port's unmount pattern.
    let writer = Effect::new(move || {
        let value = value.get();
        next.set(value);
    });
    StoredValue::new(writer);

    // The commit-phase sync (`packages/utils/src/useValueAsRef.ts:15-16`): the no-deps
    // always-run layout effect, through the ported `use_iso_layout_effect`. The tracked read
    // of `next` subscribes this sync to the render-phase writes; in a non-DOM environment the
    // callback is discarded entirely (see the module docs).
    use_iso_layout_effect(move || {
        let next = next.get();
        current.set_value(next);
    });

    ValueAsRef { current, next }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the factory seed (`packages/utils/src/useValueAsRef.ts:21-30` — the object is
    // initialized with `current === next === value`; spec "State model", UNVERIFIED
    // upstream): both slots read the initial value synchronously, before any executor tick.
    #[test]
    fn both_slots_are_seeded_with_the_initial_value_synchronously() {
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        assert_eq!(
            latest.current(),
            "initial",
            "current is seeded with the first value"
        );
        assert_eq!(
            latest.next(),
            "initial",
            "next is seeded with the first value"
        );

        owner.cleanup();
    }

    // Pins the non-DOM edge case (`packages/utils/src/useIsoLayoutEffect.ts:6` — the noop
    // binding; spec "Edge cases" — "the per-commit sync never runs and `current` stays at the
    // initial value after mount while only `next` tracks updates", UNVERIFIED upstream): on
    // the host target there is no `document` global, so a change updates `next` (the
    // render-phase write runs everywhere) but never `current` (the sync is discarded).
    #[test]
    fn on_a_non_dom_environment_next_tracks_updates_while_current_stays_seeded() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        source.set("updated");
        Executor::poll_local();

        assert_eq!(
            latest.next(),
            "updated",
            "the render-phase write runs in every environment"
        );
        assert_eq!(
            latest.current(),
            "initial",
            "without a DOM the commit-phase sync never runs"
        );

        owner.cleanup();
    }

    // Pins the manual `effect` surface (`packages/utils/src/useValueAsRef.ts:25-27` — a
    // consumer-invocable `next` → `current` copy; spec "Public API surface", UNVERIFIED
    // upstream): after `next` has advanced, invoking `effect` copies it into `current`, and
    // the unconditional copy is idempotent. On the host this is also the only way `current`
    // can advance.
    #[test]
    fn invoking_effect_manually_copies_next_into_current() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        source.set("updated");
        Executor::poll_local();
        assert_eq!(latest.current(), "initial", "the sync has not run on host");

        latest.effect();
        assert_eq!(latest.current(), "updated", "the manual copy advances current");

        latest.effect();
        assert_eq!(
            latest.current(),
            "updated",
            "the unconditional copy is idempotent"
        );

        owner.cleanup();
    }

    // Pins the hook's defining guarantee (JSDoc `packages/utils/src/useValueAsRef.ts:5-9` —
    // "access the passed value inside React.useEffect without causing the effect to re-run
    // when the value changes"; spec "State model", UNVERIFIED upstream): an effect reading
    // `current()` and `next()` subscribes to nothing — value changes and manual writes never
    // re-run it.
    #[test]
    fn reading_the_slots_does_not_subscribe_a_consumer_effect() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        let reader = latest;
        Effect::new(move || {
            let _current = reader.current();
            let _next = reader.next();
            counter.set(counter.get() + 1);
        });
        Executor::poll_local();
        assert_eq!(runs.get(), 1, "the initial read runs the effect once");

        source.set("changed");
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "a value change never re-runs an effect reading the slots"
        );

        reader.set_current("pinned");
        assert_eq!(runs.get(), 1, "a manual write notifies nothing either");

        owner.cleanup();
    }

    // Pins the manual `current` write (SliderControl's pin-ahead pattern,
    // `packages/react/src/slider/control/SliderControl.tsx:224,302` — UNVERIFIED upstream
    // through a dedicated test): `set_current` is observable immediately through the handle.
    #[test]
    fn a_manual_current_write_is_immediately_observable() {
        let owner = in_owner();

        let source: RwSignal<i32> = RwSignal::new(0);
        let latest = use_value_as_ref(source);

        latest.set_current(42);
        assert_eq!(latest.current(), 42, "the manual write lands on the slot");

        owner.cleanup();
    }

    // Pins identity stability (`packages/utils/src/useValueAsRef.ts:11-18` — the returned
    // object's identity never changes; spec "Public API surface", UNVERIFIED upstream): the
    // handle is `Copy`, and writes through one copy are observed through another — the
    // detached-closure/subscription pattern.
    #[test]
    fn writes_through_one_handle_copy_are_observed_through_another() {
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        let captured = latest;
        let observe = move || captured.current();
        let pin = move |value| captured.set_current(value);

        pin("pinned");
        assert_eq!(observe(), "pinned", "the copies share one instance");

        owner.cleanup();
    }

    // Pins per-call-site independence and the remount edge case
    // (`packages/utils/src/useValueAsRef.ts:11` — each call site gets an independent object,
    // no module-level state; a fresh component instance allocates a fresh object; spec "Edge
    // cases", UNVERIFIED upstream): two hook calls in one owner are independent, and a fresh
    // owner seeds from its own value.
    #[test]
    fn call_sites_are_independent_and_a_fresh_owner_is_a_fresh_instance() {
        let owner = in_owner();

        let first: RwSignal<i32> = RwSignal::new(1);
        let second: RwSignal<i32> = RwSignal::new(2);
        let first_latest = use_value_as_ref(first);
        let second_latest = use_value_as_ref(second);

        first_latest.set_current(100);
        assert_eq!(
            second_latest.current(),
            2,
            "the other call site is untouched"
        );

        owner.cleanup();

        let fresh_owner = in_owner();
        let fresh: RwSignal<i32> = RwSignal::new(3);
        let fresh_latest = use_value_as_ref(fresh);
        assert_eq!(
            fresh_latest.current(),
            3,
            "a fresh owner seeds from its own value, not the previous instance's"
        );
        fresh_owner.cleanup();
    }

    // Pins the lifetime boundary (the module docs — the recorded reactive_graph deviation):
    // access through the handle after owner disposal panics rather than returning a
    // still-referenced value.
    #[test]
    #[should_panic(expected = "already been disposed")]
    fn reading_current_after_owner_disposal_panics() {
        let owner = in_owner();
        let source: RwSignal<i32> = RwSignal::new(0);
        let latest = use_value_as_ref(source);

        owner.cleanup();
        let _ = latest.current();
    }

    // Pins the rapid-change collapse (`packages/utils/src/useValueAsRef.ts:13-27` —
    // intermediate values collapse and `current` ends at the last rendered value; spec "Edge
    // cases", UNVERIFIED upstream): several changes issued before the executor settles leave
    // `next` at the final value only.
    #[test]
    fn rapid_changes_collapse_to_the_last_value() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        source.set("first");
        source.set("second");
        source.set("third");
        Executor::poll_local();

        assert_eq!(
            latest.next(),
            "third",
            "intermediate values collapse in the render-phase slot"
        );

        owner.cleanup();
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::effect::Effect;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, Set};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    // The behavior under test includes the DOM-environment commit-phase sync, so the tests
    // run in a real browser via the wasm32 test runner (`.cargo/config.toml` wires it to
    // chromedriver), like the crate's other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the factory seed in a real realm (`packages/utils/src/useValueAsRef.ts:21-30`).
    #[wasm_bindgen_test]
    fn both_slots_are_seeded_with_the_initial_value_synchronously() {
        let owner = Owner::new();
        owner.set();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        assert_eq!(latest.current(), "initial");
        assert_eq!(latest.next(), "initial");

        owner.cleanup();
    }

    // Pins the commit-phase sync in a real realm
    // (`packages/utils/src/useValueAsRef.ts:15-16,25-27` — after the commit, `current` has
    // caught up to `next`; spec "State model" — UNVERIFIED upstream): a DOM environment runs
    // the sync, so a change settles with both slots at the new value.
    #[wasm_bindgen_test]
    fn the_commit_phase_sync_catches_current_up_to_next() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        source.set("updated");
        Executor::poll_local();

        assert_eq!(latest.next(), "updated", "the render-phase write landed");
        assert_eq!(
            latest.current(),
            "updated",
            "the commit-phase sync caught current up"
        );

        owner.cleanup();
    }

    // Pins the rapid-change collapse in a real realm
    // (`packages/utils/src/useValueAsRef.ts:13-27`; spec "Edge cases" — UNVERIFIED
    // upstream): values from renders that never settle are never observable in `current`.
    #[wasm_bindgen_test]
    fn rapid_changes_collapse_current_to_the_last_value() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let source: RwSignal<i32> = RwSignal::new(0);
        let latest = use_value_as_ref(source);

        source.set(1);
        source.set(2);
        source.set(3);
        Executor::poll_local();

        assert_eq!(latest.current(), 3, "current ends at the last value");

        owner.cleanup();
    }

    // Pins the hook's defining guarantee in a real realm (JSDoc
    // `packages/utils/src/useValueAsRef.ts:5-9`): an effect reading the slots never re-runs
    // on value changes.
    #[wasm_bindgen_test]
    fn reading_the_slots_does_not_subscribe_a_consumer_effect() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let source: RwSignal<&'static str> = RwSignal::new("initial");
        let latest = use_value_as_ref(source);

        let runs = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&runs);
        let reader = latest;
        Effect::new(move || {
            let _current = reader.current();
            let _next = reader.next();
            counter.set(counter.get() + 1);
        });
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        source.set("changed");
        Executor::poll_local();
        assert_eq!(
            runs.get(),
            1,
            "a value change never re-runs an effect reading the slots"
        );

        owner.cleanup();
    }

    // Pins the manual-write supersession in a real realm (the SliderControl pattern,
    // `packages/react/src/slider/control/SliderControl.tsx:224,302`): a manual `current`
    // write holds until the next commit-phase sync supersedes it — as upstream's next commit
    // does.
    #[wasm_bindgen_test]
    fn the_next_sync_supersedes_a_manual_current_write() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let source: RwSignal<i32> = RwSignal::new(0);
        let latest = use_value_as_ref(source);

        latest.set_current(42);
        Executor::poll_local();
        assert_eq!(latest.current(), 42, "the manual write holds without a change");

        source.set(7);
        Executor::poll_local();
        assert_eq!(
            latest.current(),
            7,
            "the commit-phase sync supersedes the manual write"
        );

        owner.cleanup();
    }
}
