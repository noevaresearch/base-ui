//! Port of `packages/utils/src/empty.ts` (Base UI Phase A util).
//!
//! Upstream exports three shared "nothing" values: a no-op function (`NOOP`,
//! `packages/utils/src/empty.ts:1`), a frozen empty array (`EMPTY_ARRAY`,
//! `packages/utils/src/empty.ts:6`), and a frozen empty object (`EMPTY_OBJECT`,
//! `packages/utils/src/empty.ts:8`). The only upstream test file
//! (`packages/utils/src/empty.test.ts`) asserts that the two singletons are frozen, and that
//! `EMPTY_ARRAY` is zero-length; everything else is marked unverified in `specs/utils/empty.md`.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - JS values are dynamically typed, so one `NOOP` is assignable to any callback signature and
//!   its arguments are simply ignored. Rust signatures are fixed, so [`NOOP`] covers the
//!   zero-argument (`() => void`) shapes upstream actually stores it in (cleanup refs, seeded
//!   store commands); callers needing a no-op for an argument-taking signature write the
//!   argument-ignoring closure directly (`|_| {}`). Identity comparison against `NOOP` still
//!   works: `fn` pointers implement `PartialEq`, mirroring the `=== NOOP` checks upstream
//!   (e.g. `packages/react/src/internals/labelable-provider/useLabelableId.ts:24`).
//! - `EMPTY_ARRAY` is typed upstream as mutable `never[]` so it is assignable to any `T[]`
//!   fallback without widening `T`, and frozen so a write through a widened alias throws instead
//!   of mutating the shared singleton (`packages/utils/src/empty.ts:3-6`). Rust consts cannot be
//!   generic, so the same contract is expressed as the generic [`empty_array`] function: it
//!   returns an empty `&'static [T]` for any element type, and the immutability (frozen-ness) is
//!   enforced by the type system — no caller can push to or mutate the shared slice — rather
//!   than by a runtime `Object.isFrozen` flag.
//! - `EMPTY_OBJECT` is a shared record with zero own properties, frozen
//!   (`packages/utils/src/empty.ts:8`). Rust's unit type is exactly the empty product type: a
//!   single shared value with no fields, immutable by construction. For typed records, Rust
//!   struct literals / `Default::default()` play the same empty-fallback role downstream
//!   (`EMPTY_OBJECT as CustomProperties` in
//!   `packages/react/src/internals/createBaseUIEventDetails.ts:129`), so [`EMPTY_OBJECT`] exists
//!   as the untyped shared empty value.

/// The upstream no-op function (`packages/utils/src/empty.ts:1`): callable, does nothing, returns
/// nothing.
///
/// Usable directly as a `fn()` value — for the seeded-command and default-callback shapes
/// upstream stores `NOOP` in (`packages/react/src/tooltip/store/TooltipStore.ts:115`,
/// `packages/react/src/floating-ui-react/utils/enqueueFocus.ts:26`) — and comparable by
/// identity with `==`, mirroring upstream's `=== NOOP` checks.
pub const NOOP: fn() = noop;

/// A no-op function; see [`NOOP`].
fn noop() {}

/// A shared, empty, immutable slice of `T` — the upstream `EMPTY_ARRAY`
/// (`packages/utils/src/empty.ts:6`).
///
/// Returns an empty `&'static [T]` for any element type, so it can serve as the
/// `defaultValue ?? EMPTY_ARRAY` fallback without widening the element type
/// (`packages/utils/src/empty.ts:3-5`). The slice cannot be mutated through the returned
/// reference — the frozen singleton property upstream asserts with `Object.isFrozen`
/// (`packages/utils/src/empty.test.ts:6`) holds structurally.
pub const fn empty_array<T>() -> &'static [T] {
    &[]
}

/// The shared empty record — the upstream `EMPTY_OBJECT`
/// (`packages/utils/src/empty.ts:8`): a single frozen value with zero own properties
/// (`packages/utils/src/empty.test.ts:11` asserts the frozen-ness half).
///
/// In Rust that is the unit type; typed records use their own `Default::default()` or empty
/// struct literal for the same fallback role.
pub const EMPTY_OBJECT: () = ();

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors `packages/utils/src/empty.test.ts:5-8`: the shared array is frozen and
    // zero-length. Frozen-ness is a type-system property here — `empty_array` only ever hands
    // out `&'static [T]`, so no caller can push to or mutate the shared slice — so the
    // runtime-provable halves are: empty, and assignable to any element type without widening
    // (the upstream mutable-`never[]` trick, `packages/utils/src/empty.ts:3-5`).
    #[test]
    fn keeps_the_shared_empty_array_frozen_and_empty() {
        let numbers: &[i32] = empty_array();
        assert!(numbers.is_empty());
        let words: &[String] = empty_array();
        assert!(words.is_empty());
    }

    // The upstream typing trick (`packages/utils/src/empty.ts:3-5`) makes `EMPTY_ARRAY` usable
    // as a fallback for any `T[]` option without widening `T`; the generic signature preserves
    // exactly that at the Rust call sites that will replace `?? EMPTY_ARRAY`.
    #[test]
    fn works_as_an_option_fallback_for_any_element_type() {
        let maybe_numbers: Option<&[u8]> = None;
        assert!(maybe_numbers.unwrap_or_else(empty_array::<u8>).is_empty());
        let maybe_words: Option<Vec<String>> = None;
        assert!(maybe_words.unwrap_or_default().is_empty());
    }

    // Mirrors `packages/utils/src/empty.test.ts:10-12`: the shared object is frozen. `()` is
    // the empty product type — one shared value, zero fields, immutable by construction — so
    // the assertion below pins the export to exactly that empty record. (The spec marks "zero
    // own properties" as UNVERIFIED upstream, inferred from `packages/utils/src/empty.ts:8`;
    // the Rust representation makes it structural.)
    #[test]
    fn keeps_the_shared_empty_object_frozen() {
        assert_eq!(EMPTY_OBJECT, ());
    }

    // Upstream has no test for `NOOP` (specs/utils/empty.md marks it UNVERIFIED, inferred from
    // `packages/utils/src/empty.ts:1`). This covers the Rust-adapted contract only: callable
    // as a `fn()` value, and compared by identity — a different function is not `NOOP` — which
    // is the `=== NOOP` semantics downstream ports rely on
    // (`packages/react/src/internals/labelable-provider/useLabelableId.ts:24`).
    #[test]
    fn noop_is_callable_and_comparable_by_identity() {
        fn some_other_function() {}
        let f: fn() = NOOP;
        f();
        assert_eq!(f, NOOP);
        assert_ne!(some_other_function as fn(), NOOP);
    }
}
