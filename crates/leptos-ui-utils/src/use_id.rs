//! Port of `packages/utils/src/useId.ts` (Base UI Phase A util).
//!
//! Upstream is a two-path hook (`packages/utils/src/useId.ts:32-42`). When React's built-in
//! `useId` is available (`packages/utils/src/useId.ts:24` — the React 18+ path the upstream
//! test suite actually exercises), it is called unconditionally on every render
//! (`packages/utils/src/useId.ts:35`) and the export returns the override or React's
//! per-instance id, prefix-wrapped when a prefix is given —
//! `idOverride ?? (prefix ? `${prefix}-${reactId}` : reactId)`
//! (`packages/utils/src/useId.ts:36`). Otherwise it falls back to `useGlobalId`
//! (`packages/utils/src/useId.ts:8-22`): state seeded with the override
//! (`packages/utils/src/useId.ts:9`), `idOverride || defaultId` exposed
//! (`packages/utils/src/useId.ts:10`), and a client-only effect generating
//! `` `${prefix}-${globalId}` `` from a module-level counter, incrementing before the read and
//! defaulting the prefix to `'mui'` (`packages/utils/src/useId.ts:5`, `:8`, `:17-18`).
//!
//! The port has no `React.useId` to delegate to, so per `specs/architecture.md`
//! ("ID generation (`useId`)") the generation mechanism is the fallback's deterministic
//! counter, formatted the way the fallback formats it (`` `${prefix ?? 'mui'}-{n}` ``), while
//! the timing is the modern path's: the counter is consumed eagerly at hook-call time, in
//! both render passes. That is what makes the SSR behavior proven by
//! `packages/utils/src/useId.test.tsx:89-100` hold — the fallback path itself cannot provide
//! an SSR id (its counter only runs in a client effect, `packages/utils/src/useId.ts:11-20`),
//! which is exactly why the upstream suite gates its server test on `React.useId` being
//! defined (`packages/utils/src/useId.test.tsx:90-92`). The counter is a [`thread_local!`]
//! `Cell<u64>` (the crate's convention for upstream module state — same as the
//! [`crate::generate_id`] port), so its sequence restarts per process/thread and reproduces
//! the same sequence for the same component order on the "server" and "client" passes — a
//! plain implementation detail, not a hydration concern (Leptos 0.7 does not match elements
//! by ID).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Upstream re-renders and recomputes the export's expression every render; Leptos
//!   components run once and props are reactive sources. `idOverride` becomes
//!   `C: Get<Value = Option<String>>` (the shape a component's `id?: string` prop takes as a
//!   `MaybeProp<String>`) and the return value is a derived local [`Signal`]
//!   re-evaluating the override-or-generated choice on every read — the same reactivity
//!   convention as the [`crate::use_controlled`] port. The generated core is consumed once
//!   per hook call and never changes, matching upstream's per-instance id stability (React's
//!   `useId`; on the fallback path the one-time `setDefaultId`,
//!   `packages/utils/src/useId.ts:12-19`).
//! - `prefix` is read once at call time and baked into the generated core. The modern path
//!   re-applies the prefix on every render, but every upstream call site passes a literal
//!   (`packages/react/src/internals/useBaseUiId.ts:10` passes `'base-ui'`; no dynamic-prefix
//!   call site exists upstream), and the counter-bearing fallback reads the prefix only at
//!   generation (`packages/utils/src/useId.ts:18`). Post-call prefix changes are UNVERIFIED
//!   upstream (`specs/utils/useId.md`, "State model") and unsupported by the port.
//! - The default prefix is `'mui'` (`packages/utils/src/useId.ts:8`) — the only generated-id
//!   format the counter-bearing fallback defines. A bare `useId()` on the modern path returns
//!   React's own id format instead, which is not portable.
//! - The counter is consumed on every hook call, even when an override will win, matching
//!   the modern path's unconditional `React.useId()` call (`packages/utils/src/useId.ts:35`);
//!   the fallback increments only while uncontrolled (`packages/utils/src/useId.ts:12-19`).
//!   The divergence is unobservable — ids only need uniqueness, and while an override is
//!   present the generated core is never exposed.
//! - Upstream's return type is `string | undefined` (`packages/utils/src/useId.ts:32`), but
//!   `undefined` only escapes on the fallback path before its first client effect (SSR on
//!   React 17); the supported path always yields a string, and the spec's public API surface
//!   states `string` (`specs/utils/useId.md`). The port returns [`String`].
//! - Nullish-coalescing semantics are preserved, not the fallback path's `||` semantics: a
//!   *provided* override is returned verbatim even when empty (modern path,
//!   `packages/utils/src/useId.ts:36` — `'' ?? x` is `''`), so
//!   [`Option::unwrap_or_else`] substitutes only on `None`. The fallback's
//!   `idOverride || defaultId` (`packages/utils/src/useId.ts:10`) would generate instead — a
//!   pre-React-18 divergence the supported path eliminated.
//! - The `useState` seed + client-only `useEffect` mechanics
//!   (`packages/utils/src/useId.ts:9-20`) are replaced by eager generation plus the derived
//!   signal: no effect is registered and no cleanup is needed, matching upstream's
//!   cleanup-free hook.
//! - `'use client'` (`packages/utils/src/useId.ts:1`) is N/A — there is no React Server
//!   Components boundary in Rust.

use std::cell::Cell;

use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;

thread_local! {
    /// The upstream module-level `globalId` (`packages/utils/src/useId.ts:5`): shared across
    /// every generating call for the thread's lifetime, never reset, not readable from
    /// outside.
    static COUNTER: Cell<u64> = const { Cell::new(0) };
}

/// The fallback signature's default prefix (`packages/utils/src/useId.ts:8`) — the only
/// generated-id format the counter-bearing upstream path defines.
const DEFAULT_PREFIX: &str = "mui";

/// The upstream client-effect body's `globalId += 1; ...globalId` sequence
/// (`packages/utils/src/useId.ts:17-18`): increment before the read, so the first generated
/// id carries `1`. `wrapping_add` mirrors JS's never-overflow number semantics rather than
/// panicking a debug build after 2⁶⁴ calls, same as the [`crate::generate_id`] port.
fn next_id_number() -> u64 {
    COUNTER.with(|counter| {
        counter.set(counter.get().wrapping_add(1));
        counter.get()
    })
}

/// The upstream `useId` export (`packages/utils/src/useId.ts:32-42`): returns the externally
/// supplied id verbatim while provided, otherwise a deterministic generated id —
/// `` `{prefix}-{n}` `` when a prefix is given, `` `mui-{n}` `` when not. The generated core
/// is stable for the hook instance's lifetime, so repeated reads and derived compositions
/// (`` `${id}-label` ``) stay coherent, and each hook call consumes its own counter value, so
/// multiple calls in one component never collide
/// (`packages/utils/src/useId.test.tsx:42-62`, `:64-87`).
///
/// Must be called inside a reactive owner (a component): the returned derived signal is
/// arena-allocated to the calling owner and disposed with it. The signal tracks the override
/// source, so a component whose id prop changes reactively picks up the new value —
/// upstream's re-render semantics (`packages/utils/src/useId.test.tsx:13-26`, `:28-40`) —
/// without the component re-running.
pub fn use_id<C>(id_override: C, prefix: Option<&str>) -> Signal<String, LocalStorage>
where
    C: Get<Value = Option<String>> + 'static,
{
    // The modern path's unconditional `React.useId()` call
    // (`packages/utils/src/useId.ts:35`), ported as the architecture decision's deterministic
    // counter (see the module docs): consumed eagerly at call time, in both render passes.
    let n = next_id_number();

    // The fallback's generation format (`packages/utils/src/useId.ts:17-18`, with its `'mui'`
    // parameter default at `packages/utils/src/useId.ts:8`), applied to the caller's prefix.
    // Baked once: the prefix is call-time configuration (module docs).
    let generated = format!("{}-{n}", prefix.unwrap_or(DEFAULT_PREFIX));

    // The export's return expression (`packages/utils/src/useId.ts:36`) —
    // `idOverride ?? (prefix ? `${prefix}-${reactId}` : reactId)` — with the ported reactId
    // analog being the generated core above: the override wins while provided, the generated
    // id applies otherwise. `unwrap_or_else` preserves the modern path's nullish semantics: a
    // provided-but-empty string is returned verbatim (module docs).
    Signal::derive_local(move || id_override.get().unwrap_or_else(|| generated.clone()))
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

    fn none_source() -> RwSignal<Option<String>> {
        RwSignal::new(None)
    }

    // Mirrors `packages/utils/src/useId.test.tsx:13-26`: the provided id is returned
    // verbatim, and an update to the override source is picked up — the port's analog of the
    // `setProps({ id: 'another-id' })` re-render.
    #[test]
    fn returns_the_provided_id_and_tracks_updates() {
        let owner = in_owner();

        let id_override: RwSignal<Option<String>> = RwSignal::new(Some("some-id".to_string()));
        let id = use_id(id_override, None);

        assert_eq!(id.get_untracked(), "some-id");

        id_override.set(Some("another-id".to_string()));
        assert_eq!(id.get_untracked(), "another-id");

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useId.test.tsx:28-40`: with no override the hook generates
    // a non-empty id, and a later override takes over — the generated → provided transition
    // the spec pins (`specs/utils/useId.md`, "State model").
    #[test]
    fn generates_a_non_empty_id_when_none_is_provided_and_tracks_a_later_override() {
        let owner = in_owner();

        let id_override: RwSignal<Option<String>> = RwSignal::new(None);
        let id = use_id(id_override, None);

        assert!(!id.get_untracked().is_empty());

        id_override.set(Some("another-id".to_string()));
        assert_eq!(id.get_untracked(), "another-id");

        owner.cleanup();
    }

    // Pins the modern-path `??` semantics in both directions (module docs; the reverse
    // transition and the empty-string case are UNVERIFIED upstream —
    // `specs/utils/useId.md`, "State model" — and the upstream paths diverge on `''`):
    // dropping the override reverts to the generated id, and a provided-but-empty string is
    // returned verbatim rather than regenerating (`packages/utils/src/useId.ts:36`, where
    // `'' ?? x` is `''`; the fallback's `||` at `packages/utils/src/useId.ts:10` would
    // generate instead).
    #[test]
    fn a_provided_id_returning_to_absent_falls_back_and_empty_string_is_verbatim() {
        let owner = in_owner();

        let id_override: RwSignal<Option<String>> = RwSignal::new(Some("first".to_string()));
        let id = use_id(id_override, None);

        assert_eq!(id.get_untracked(), "first");

        id_override.set(None);
        assert!(
            !id.get_untracked().is_empty(),
            "reverting to no override restores the generated id"
        );

        id_override.set(Some(String::new()));
        assert_eq!(
            id.get_untracked(),
            "",
            "a provided empty string is returned verbatim (nullish, not falsy, semantics)"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useId.test.tsx:42-62` at the hook level: the same instance
    // returns the same string on every read (the stability the spec marks UNVERIFIED upstream,
    // `specs/utils/useId.md`, "State model"), so a caller composing `` `${id}-label` `` gets a
    // coherent, wired-up pair.
    #[test]
    fn repeated_reads_are_stable_and_support_suffix_composition() {
        let owner = in_owner();

        let id = use_id(none_source(), None);

        let first = id.get_untracked();
        assert!(!first.is_empty());
        assert_eq!(id.get_untracked(), first, "the generated core is consumed once");

        let label_id = format!("{first}-label");
        assert_eq!(
            format!("{}-label", id.get_untracked()),
            label_id,
            "suffix composition is stable across reads"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useId.test.tsx:64-87`: two hook calls in one component
    // yield distinct ids that compose into a single space-separated IDREF list —
    // `aria-labelledby={`${labelPartA} ${labelPartB}`}`.
    #[test]
    fn multiple_calls_in_one_component_produce_distinct_ids() {
        let owner = in_owner();

        let source: RwSignal<Option<String>> = RwSignal::new(None);
        let label_part_a = use_id(source, None);
        let label_part_b = use_id(source, None);

        let part_a = label_part_a.get_untracked();
        let part_b = label_part_b.get_untracked();
        assert_ne!(part_a, part_b, "each hook call consumes its own counter value");

        assert_eq!(
            format!("{part_a} {part_b}"),
            format!("{} {}", label_part_a.get_untracked(), label_part_b.get_untracked()),
            "the combined IDREF list stays coherent with the parts"
        );

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useId.test.tsx:89-100` (the SSR case the upstream suite
    // gates on `React.useId`, `packages/utils/src/useId.test.tsx:90-92`): the port generates
    // eagerly at call time, so a non-empty id exists with no executor, no effect, and no
    // flush — the analog of rendering on the server (`specs/architecture.md`, "ID
    // generation").
    #[test]
    fn an_id_is_available_immediately_at_call_time() {
        let owner = in_owner();

        let id = use_id(none_source(), None);

        assert!(!id.get_untracked().is_empty());

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useId.test.tsx:102-123`: a prefix produces an id beginning
    // with `{prefix}-`, valid for the same ARIA linking pattern.
    #[test]
    fn can_be_prefixed() {
        let owner = in_owner();

        let id = use_id(none_source(), Some("base-ui"));

        let value = id.get_untracked();
        assert!(value.starts_with("base-ui-"), "unprefixed id: {value:?}");

        owner.cleanup();
    }

    // Pins the architecture decision's determinism (`specs/architecture.md`, "ID
    // generation": the counter must produce the same sequence on server and client for a
    // given render) plus the increment-before-read order (`packages/utils/src/useId.ts:17-18`)
    // and the shared-across-prefixes counter (`packages/utils/src/useId.ts:5`): a fresh
    // thread has a cold counter (the analog of a fresh render pass), where the sequence is
    // exactly `mui-1`, `mui-2`, then `base-ui-3` for a prefixed call.
    #[test]
    fn the_generated_sequence_is_deterministic_from_a_cold_counter() {
        let ids = std::thread::spawn(|| {
            let owner = Owner::new();
            owner.set();

            let source: RwSignal<Option<String>> = RwSignal::new(None);
            let first = use_id(source, None).get_untracked();
            let second = use_id(source, None).get_untracked();
            let prefixed = use_id(source, Some("base-ui")).get_untracked();
            (first, second, prefixed)
        })
        .join()
        .unwrap();

        assert_eq!(ids.0, "mui-1");
        assert_eq!(ids.1, "mui-2");
        assert_eq!(ids.2, "base-ui-3", "the counter is shared across prefixes");
    }
}
