//! Port of `packages/utils/src/safeReact.ts` (Base UI Phase A util).
//!
//! Upstream is an 11-line data module: a single named export `SafeReact`, a plain object
//! shallow-spreading the entire React namespace at module-evaluation time and cast to
//! `typeof React` (`packages/utils/src/safeReact.ts:11`). Its documented intent is compat
//! indirection: bundlers can rewrite direct `React.someNewApi` reads into named imports,
//! which breaks React 17, while property reads on the clone stay optional
//! (`packages/utils/src/safeReact.ts:3-10`). It has no test file upstream
//! (`ralph/generated/utils.json:199-205` lists `testFiles: []`), so the behavioral claims
//! below are inferred from the unit's own source and its consumers, per
//! `specs/utils/safeReact.md`.
//!
//! The clone's consumed API subset in this repo is exactly three APIs
//! (`specs/utils/safeReact.md`, "Public API surface"): `useId`
//! (`packages/utils/src/useId.ts:24`), `useInsertionEffect`/`useLayoutEffect`
//! (`packages/utils/src/useStableCallback.ts:5-12`), and `captureOwnerStack` — 12 call
//! sites across 9 files, every one optional-chained with a `|| ''` fallback (e.g.
//! `packages/react/src/internals/use-button/useButton.ts:47`,
//! `packages/react/src/internals/use-button/useButton.ts:56`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - There is no React namespace to clone, and the failure mode the clone exists to avoid —
//!   bundlers rewriting `React.someNewApi` member reads into named imports that React 17
//!   cannot resolve — does not exist in Rust: symbol resolution happens at compile time, and
//!   a missing API is a compile error, not a runtime lookup. The module therefore ports the
//!   clone's *consumed contract*, not its object mechanics, following the same
//!   pin-the-branch-callers-take approach as [`crate::react_version`].
//! - The two import-time hook reads (`useId`, `useInsertionEffect`/`useLayoutEffect`) have no
//!   port counterpart here: on the React 19 behavior this port implements
//!   ([`crate::react_version::REACT_MAJOR_VERSION`]) all three hooks exist, and in the Leptos
//!   port their roles are framework-native (Leptos effects; id generation per
//!   `specs/architecture.md`, "ID generation"), decided by each consumer's own port (the
//!   `useId`/`useStableCallback` TODO items), not by an optional namespace lookup.
//! - `captureOwnerStack` is the one call-time read, and the one whose optionality is
//!   load-bearing: React exposes it only in development builds, so production consumers take
//!   the `?.()` miss branch and append `''`. The port has no React owner-stack machinery to
//!   read from, so [`capture_owner_stack`] implements the absent-API branch — returning
//!   [`None`] — which is the branch every one of the 12 upstream call sites already guards
//!   against. The `|| ''` fallback composes as `capture_owner_stack().unwrap_or_default()`.
//! - Upstream's plain-object mutability — tests reconfigure `SafeReact.captureOwnerStack`
//!   with `Object.defineProperty` and restore it afterwards (e.g.
//!   `packages/react/src/combobox/label/ComboboxLabel.test.tsx:28-54`) — is upstream test
//!   mechanics for stubbing a module singleton, not a production contract; there is no object
//!   here to reconfigure and the port pins one immutable branch. The module-singleton
//!   property (all importers share one instance with a fixed property set,
//!   `specs/utils/safeReact.md`, "State model") maps to a stateless free function, which is
//!   trivially consistent across all callers.

/// Port of upstream `SafeReact.captureOwnerStack`
/// (`packages/utils/src/safeReact.ts:11`, member read e.g. at
/// `packages/react/src/internals/use-button/useButton.ts:47`).
///
/// Upstream, `SafeReact.captureOwnerStack?.()` returns a component-owner stack string in
/// React development builds and misses (`undefined`) in production builds, where every
/// consumer falls back to `''`. This port implements the absent-API branch: there is no
/// React owner-stack machinery to read from, so the answer is always [`None`], and
/// consumers append it to warning messages as `capture_owner_stack().unwrap_or_default()`
/// — the exact composition of upstream's `SafeReact.captureOwnerStack?.() || ''`.
///
/// The function is stateless (upstream: module singleton with a fixed property set,
/// `specs/utils/safeReact.md`, "State model"), so repeated or interleaved calls cannot
/// interfere with each other.
pub fn capture_owner_stack() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pins the absent-API branch (`specs/utils/safeReact.md`, "Edge cases" —
    // `captureOwnerStack` absent ⇒ consumers warn with the same messages, just without an
    // owner stack): the port has no owner-stack machinery, so the answer is `None`.
    #[test]
    fn capture_owner_stack_implements_the_absent_api_branch() {
        assert_eq!(capture_owner_stack(), None);
    }

    // Pins the consumer-side fallback composition upstream writes as
    // `SafeReact.captureOwnerStack?.() || ''` (e.g.
    // `packages/react/src/internals/use-button/useButton.ts:47`): the `None` answer composes
    // to the empty string, so warnings are emitted without an owner stack.
    #[test]
    fn consumer_fallback_composition_yields_the_empty_string() {
        let owner_stack_message: String = capture_owner_stack().unwrap_or_default();
        assert_eq!(owner_stack_message, "");
    }

    // Pins the module-singleton contract (`specs/utils/safeReact.md`, "State model" —
    // identity and property set fixed for the module's lifetime): the stateless function
    // gives the same answer on repeated and interleaved calls.
    #[test]
    fn repeated_and_interleaved_calls_are_consistent() {
        for _ in 0..3 {
            assert_eq!(capture_owner_stack(), None);
            assert_eq!(capture_owner_stack().unwrap_or_default(), "");
        }
    }
}
