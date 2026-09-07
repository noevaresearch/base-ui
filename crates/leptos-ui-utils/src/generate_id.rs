//! Port of `packages/utils/src/generateId.ts` (Base UI Phase A util).
//!
//! Upstream is a 5-line module with no test file of its own (`testFiles: []`,
//! `ralph/generated/utils.json:97-104`) and no `wraps-external` dependency
//! (`specs/utils/generateId.md`), so the algorithm is implemented here directly from
//! `packages/utils/src/generateId.ts:1-5`: a module-level counter incremented on every
//! call, embedded in the returned id between the caller's prefix and a random
//! 4-character base36 segment — `${prefix}-${Math.random().toString(36).slice(2, 6)}-${counter}`.
//!
//! The only upstream runtime evidence is indirect: the toast subsystem resolves an
//! id-less toast through `generateId('toast')` (`packages/react/src/toast/store.ts:167`,
//! `packages/react/src/toast/createToastManager.ts:30`) and the toast manager test only
//! asserts the result is a string (`specs/utils/generateId.md`). All format-level claims
//! are therefore implementation-derived, matching the spec's UNVERIFIED framing.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The module-level `let counter = 0` becomes a [`thread_local!`] `Cell<u64>`: wasm
//!   execution is single-threaded, so this mirrors upstream's shared module state, the
//!   same convention as [`crate::format_number`]'s cache. The counter increments *before*
//!   the value is read, so the first generated id carries counter `1`
//!   (`packages/utils/src/generateId.ts:3-4`). `wrapping_add` mirrors JS's never-overflow
//!   number semantics rather than panicking a debug build after 2⁶⁴ calls.
//! - `Math.random().toString(36).slice(2, 6)` is delegated to the platform exactly:
//!   [`js_sys::Math::random`] feeds the JS `Number.prototype.toString(36)`
//!   (`packages/utils/src/generateId.ts:4`), then the same `.slice(2, 6)` window is taken
//!   (characters 2..6). Hand-rolling base36 formatting in Rust would diverge from the
//!   engine's shortest-round-trip representation, so the port does what
//!   [`crate::format_number`] does for `Intl` — use the platform's own implementation.
//!   Upstream has no SSR guard and neither does the port: the segment is generated at
//!   call time (`specs/utils/generateId.md`, contrasted with `useId` in
//!   `packages/utils/src/useId.ts:14-18`).

use std::cell::Cell;

use wasm_bindgen::UnwrapThrowExt;

thread_local! {
    /// The upstream module-level `counter` (`packages/utils/src/generateId.ts:1`): shared
    /// across every call for the module's lifetime, never reset, not readable from outside.
    static COUNTER: Cell<u64> = const { Cell::new(0) };
}

/// The upstream `generateId` export (`packages/utils/src/generateId.ts:2-4`): returns
/// `` `{prefix}-{random base36 segment}-{counter}` ``, incrementing the shared counter
/// first so every call in the process yields a distinct id regardless of the random
/// segment.
pub fn generate_id(prefix: &str) -> String {
    let counter = COUNTER.with(|counter| {
        counter.set(counter.get().wrapping_add(1));
        counter.get()
    });
    format!("{prefix}-{}-{counter}", random_base36_segment())
}

/// `Math.random().toString(36).slice(2, 6)` (`packages/utils/src/generateId.ts:4`): the
/// platform's base36 rendering of a random double, minus its leading `"0."`, truncated to
/// 4 characters. A value that renders shorter (up to and including `0` itself, rendering
/// as `"0"`) yields a correspondingly shorter — possibly empty — segment, exactly as the
/// JS slice does.
fn random_base36_segment() -> String {
    let random = js_sys::Math::random();
    let base36: String = js_sys::Number::from(random)
        .to_string_with_radix(36)
        .unwrap_throw()
        .into();
    base36.chars().skip(2).take(4).collect()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    // The tests exercise the JS `Math`/`Number` globals, so they run in a real browser via
    // the wasm32 test runner (`.cargo/config.toml` wires it to chromedriver), like the
    // crate's other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// The trailing `-{counter}` of a generated id, parsed as an integer; the counter is
    /// shared module state, so tests reason about relative values rather than absolutes.
    fn trailing_counter(id: &str) -> u64 {
        let (_, digits) = id.rsplit_once('-').expect("id to contain a '-' separator");
        digits.parse().expect("trailing counter to be numeric")
    }

    /// The id shape derived from `packages/utils/src/generateId.ts:4`: the prefix embedded
    /// verbatim, a base36 random segment of at most 4 characters (the `.slice(2, 6)`
    /// window; it can be shorter when the platform's base36 rendering is, e.g. for 0.5 →
    /// `"0.i"`), and a positive counter. The spec marks the format UNVERIFIED
    /// (`specs/utils/generateId.md`) — this asserts what the cited implementation produces.
    #[wasm_bindgen_test]
    fn embeds_the_prefix_a_random_segment_and_the_counter() {
        let id = generate_id("toast");
        let rest = id.strip_prefix("toast-").expect("prefix embedded verbatim");
        let (segment, counter) = rest.rsplit_once('-').expect("counter separator");
        assert!(segment.len() <= 4, "segment longer than slice(2, 6): {segment:?}");
        assert!(
            segment.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "segment not base36: {segment:?}"
        );
        assert!(
            counter.parse::<u64>().map(|n| n > 0).unwrap_or(false),
            "counter not a positive integer: {counter:?}"
        );
    }

    // Direct from the implementation: the shared counter increments on every call
    // (`packages/utils/src/generateId.ts:3`), so two successive calls produce ids whose
    // trailing counters are exactly 1 apart.
    #[wasm_bindgen_test]
    fn counter_increments_on_every_call() {
        let first = generate_id("a");
        let second = generate_id("a");
        assert_eq!(
            trailing_counter(&second),
            trailing_counter(&first) + 1,
            "consecutive calls' counters not 1 apart: {first:?} then {second:?}"
        );
    }

    // The counter is module-level, not keyed by prefix
    // (`packages/utils/src/generateId.ts:1-4`), so it advances across different prefixes.
    #[wasm_bindgen_test]
    fn counter_is_shared_across_prefixes() {
        let first = generate_id("first");
        let second = generate_id("second");
        assert_eq!(
            trailing_counter(&second),
            trailing_counter(&first) + 1,
            "counter not shared across prefixes: {first:?} then {second:?}"
        );
    }

    // Mirrors the spec's rapid-calls edge case — five id-less toasts added in one burst
    // (`specs/utils/generateId.md`): the counter suffix makes successive calls
    // monotonically distinct regardless of the random segment
    // (`packages/utils/src/generateId.ts:3-4`).
    #[wasm_bindgen_test]
    fn rapid_repeated_calls_produce_distinct_ids() {
        let ids: Vec<String> = (0..5).map(|_| generate_id("toast")).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "ids not distinct: {ids:?}");
    }
}
