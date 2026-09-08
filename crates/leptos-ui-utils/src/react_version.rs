//! Port of `packages/utils/src/reactVersion.ts` (Base UI Phase A util).
//!
//! Upstream is a single-export pure predicate, `isReactVersionAtLeast(reactVersionToCheck:
//! SupportedVersions): boolean` (`packages/utils/src/reactVersion.ts:7-8`), whose argument is
//! constrained to the union `SupportedVersions = 17 | 18 | 19`
//! (`packages/utils/src/reactVersion.ts:5`) — callers can only ask about React major versions
//! 17, 18, or 19; any other major is unrepresentable at the type level. The comparison is
//! against a module-load-time snapshot, `parseInt(React.version, 10)`
//! (`packages/utils/src/reactVersion.ts:3`), so every call reads the same frozen value and the
//! predicate's answers are fixed per module instance. It has no test file upstream
//! (`ralph/generated/utils.json:190-197` lists `testFiles: []`), so every behavioral claim is
//! inferred from the unit's own source (`specs/utils/reactVersion.md`) and pinned by this
//! module's own tests rather than a reference suite.
//!
//! All three observed upstream call sites pass the literal `19` and use the result to gate
//! React-19-only behavior: ref extraction (`packages/utils/src/getReactElementRef.ts:15`), the
//! `inert` attribute's value shape (`packages/utils/src/inertValue.ts:4`), and whether
//! `useSyncExternalStore`'s native "get version" semantics can be relied on
//! (`packages/utils/src/store/useStore.ts:12`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - There is no React runtime here to read a version from. The port implements the React ≥ 19
//!   behavior branches everywhere — the branch all three upstream call sites take — so the
//!   module-load snapshot maps to the public compile-time constant [`REACT_MAJOR_VERSION`]
//!   pinned to `19`. This preserves the snapshot contract in its strongest form: the value is
//!   frozen before any call can observe it (upstream: frozen at module load,
//!   `packages/utils/src/reactVersion.ts:3`), every call reads the same value, and it cannot
//!   track later runtime mutation (upstream already never sees one in practice —
//!   `specs/utils/reactVersion.md`, "State model").
//! - Upstream's `parseInt(React.version, 10)` parse behaviors have no counterpart to port:
//!   there is no version string to parse, so the truncation-at-first-non-numeric-segment
//!   behavior (`'19.1.0'` → `19`) never arises, and the NaN fail-closed behavior (a missing or
//!   unparseable version answering `false` for every input) cannot occur for a compile-time
//!   constant. Documented, not replicated (`specs/utils/reactVersion.md`, "State model" and
//!   "Edge cases").
//! - The `SupportedVersions = 17 | 18 | 19` union maps to the [`SupportedVersion`] enum, keeping
//!   the same type-level restriction: a caller cannot ask about any other major.
//! - The comparison stays inclusive, `majorVersion >= reactVersionToCheck`
//!   (`packages/utils/src/reactVersion.ts:7-8`).
//! - The predicate is kept at all — rather than collapsed into the two already-ported caller
//!   modules' hardcoded branches — so the port keeps the upstream API shape and has one place
//!   pinning "which React-version behavior does this port implement", the same
//!   API-shape-consistency philosophy as the shared-`Store` decision in
//!   `specs/architecture.md` ("Shared store"). [`inert_value`](crate::inert_value) and
//!   [`get_react_element_ref`](crate::get_react_element_ref) already took the React ≥ 19
//!   branches and documented this module as their would-be counterpart dependency.

/// The React major version whose behavior this port implements — the counterpart of upstream's
/// module-load-time snapshot `parseInt(React.version, 10)`
/// (`packages/utils/src/reactVersion.ts:3`).
///
/// Upstream reads the real React runtime's version once when its module is first imported; the
/// Leptos port has no React runtime to read, so the snapshot is pinned at compile time to `19`,
/// the major whose behavior branches this port implements everywhere (the
/// `isReactVersionAtLeast(19)` branches of `packages/utils/src/getReactElementRef.ts:15`,
/// `packages/utils/src/inertValue.ts:4`, and `packages/utils/src/store/useStore.ts:12` — see
/// the module docs).
pub const REACT_MAJOR_VERSION: u8 = 19;

/// The React major versions a caller may ask about — the port of upstream's
/// `SupportedVersions = 17 | 18 | 19` union (`packages/utils/src/reactVersion.ts:5`).
///
/// Any other major is unrepresentable at the type level, exactly like upstream
/// (`specs/utils/reactVersion.md`, "Public API surface").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportedVersion {
    /// React major 17.
    V17,
    /// React major 18.
    V18,
    /// React major 19.
    V19,
}

impl SupportedVersion {
    /// The numeric major version this variant stands for — the value upstream's number-literal
    /// union member carries (`packages/utils/src/reactVersion.ts:5`).
    pub const fn major(self) -> u8 {
        match self {
            Self::V17 => 17,
            Self::V18 => 18,
            Self::V19 => 19,
        }
    }
}

/// Returns whether the React runtime whose behavior this port implements is at least the
/// requested major — the port of upstream `isReactVersionAtLeast`
/// (`packages/utils/src/reactVersion.ts:7-8`), with the same inclusive comparison
/// `majorVersion >= reactVersionToCheck`.
///
/// The answer is fixed for the life of the program (upstream: fixed per module instance,
/// `packages/utils/src/reactVersion.ts:3`; here: fixed at compile time), and repeated or
/// interleaved calls cannot interfere with each other
/// (`specs/utils/reactVersion.md`, "Edge cases"). On this port every [`SupportedVersion`]
/// variant answers `true`, because the port pins [`REACT_MAJOR_VERSION`] to `19` and the
/// comparison is inclusive — which is exactly the branch all three upstream call sites depend
/// on (`packages/utils/src/getReactElementRef.ts:15`, `packages/utils/src/inertValue.ts:4`,
/// `packages/utils/src/store/useStore.ts:12`).
pub const fn is_react_version_at_least(react_version_to_check: SupportedVersion) -> bool {
    REACT_MAJOR_VERSION >= react_version_to_check.major()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pins the inclusive-comparison edge-case table (`specs/utils/reactVersion.md` "Edge
    // cases") for the port's snapshot: on React 19, requesting 17 returns true, requesting 18
    // returns true, and requesting 19 returns true — the last case is the one all three
    // upstream call sites depend on. Upstream has no test file
    // (`ralph/generated/utils.json:190-197`), so this is the port's own pinning of the
    // inferred contract.
    #[test]
    fn every_supported_version_answers_true_on_the_react_19_port() {
        assert!(is_react_version_at_least(SupportedVersion::V17));
        assert!(is_react_version_at_least(SupportedVersion::V18));
        assert!(is_react_version_at_least(SupportedVersion::V19));
    }

    // Pins the snapshot semantics (`specs/utils/reactVersion.md` "State model" and "Edge
    // cases"): every call reads the same frozen value — the answers are fixed, and repeated or
    // interleaved calls cannot interfere with each other.
    #[test]
    fn repeated_and_interleaved_calls_read_the_same_frozen_value() {
        for _ in 0..3 {
            assert!(is_react_version_at_least(SupportedVersion::V19));
            assert!(is_react_version_at_least(SupportedVersion::V17));
        }
        assert_eq!(
            is_react_version_at_least(SupportedVersion::V19),
            is_react_version_at_least(SupportedVersion::V17)
        );
    }

    // Pins the snapshot constant itself: the port implements the React ≥ 19 branches (the
    // branch `packages/utils/src/getReactElementRef.ts:15`, `packages/utils/src/inertValue.ts:4`,
    // and `packages/utils/src/store/useStore.ts:12` all take), so the module-load snapshot's
    // counterpart must be 19 — not an arbitrary value that happens to satisfy the predicate.
    #[test]
    fn the_snapshot_is_the_react_19_branch_the_port_implements() {
        assert_eq!(REACT_MAJOR_VERSION, 19);
        assert_eq!(
            is_react_version_at_least(SupportedVersion::V19),
            REACT_MAJOR_VERSION >= 19
        );
    }

    // Pins the union mapping (`packages/utils/src/reactVersion.ts:5`): each variant stands for
    // its numeric React major, the value upstream's comparison actually uses
    // (`packages/utils/src/reactVersion.ts:7-8`).
    #[test]
    fn major_reports_the_numeric_major_of_each_variant() {
        assert_eq!(SupportedVersion::V17.major(), 17);
        assert_eq!(SupportedVersion::V18.major(), 18);
        assert_eq!(SupportedVersion::V19.major(), 19);
    }
}
