//! Port of `packages/utils/src/error.ts` (Base UI Phase A util).
//!
//! Upstream is a two-export module with no unit-local algorithm: it creates one logger at module
//! scope — `error`, the fixed binding `createLogOnce('error', 'Base UI')`
//! (`packages/utils/src/error.ts:3`) — and re-exports `reset` from `./createLogOnce`
//! (`packages/utils/src/error.ts:5`). The unit has no upstream test file, so every behavioral
//! claim is UNVERIFIED in `specs/utils/error.md`, and the port is deliberately a thin binding
//! over the ported [`create_log_once`](crate::create_log_once) machinery: the once-logging
//! algorithm is specified and tested there (`specs/utils/createLogOnce.md`) and no test in this
//! unit constrains it.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream's module-scope singleton (`packages/utils/src/error.ts:3`) becomes the [`error`]
//!   function, returning a fresh [`LogOnce`] handle per call. The two are observationally
//!   equivalent: the once-per-output dedup state lives in the module-level registry, not in the
//!   handle (`packages/utils/src/createLogOnce.ts:1-4`,
//!   `packages/utils/src/createLogOnce.ts:12-14`), so every handle with the same severity and
//!   prefix deduplicates against the same shared state — the only state-bearing aspect the spec
//!   identifies for this unit.
//! - The `reset` re-export is preserved as [`reset`], re-exported from
//!   [`crate::create_log_once`] exactly as upstream re-exports it from `./createLogOnce`
//!   (`packages/utils/src/error.ts:5`).

use crate::create_log_once::{LogOnce, Severity, create_log_once_with_prefix};

/// The upstream `error` export (`packages/utils/src/error.ts:3`): a logger fixed to the
/// `'error'` severity with the `'Base UI'` prefix — the module-scope binding of
/// [`create_log_once_with_prefix`](crate::create_log_once_with_prefix).
///
/// Sole upstream consumer: `packages/utils/src/useControlled.ts:5`.
pub fn error() -> LogOnce {
    create_log_once_with_prefix(Severity::Error, "Base UI")
}

pub use crate::create_log_once::reset;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::capture_log;

    // The fixed binding upstream performs at module scope
    // (`packages/utils/src/error.ts:3`): the emitted output carries the `Base UI` prefix, and the
    // severity is `error` — proven end-to-end against `console.error` by the createLogOnce wasm
    // tests, since `Severity::Error` selects that channel.
    #[test]
    fn binds_the_error_severity_with_the_base_ui_prefix() {
        reset();

        assert_eq!(capture_log(&error(), &["boom"]), ["Base UI: boom"]);
    }

    // The once-per-output state lives in the shared registry rather than in the handle, so two
    // handles from separate `error()` calls dedup against the same state — the observable
    // content of upstream's module-scope singleton (`packages/utils/src/error.ts:3`).
    #[test]
    fn dedups_across_separately_constructed_handles() {
        reset();
        let first = error();

        assert_eq!(capture_log(&first, &["boom"]), ["Base UI: boom"]);

        assert_eq!(capture_log(&error(), &["boom"]), Vec::<String>::new());
    }

    // The re-exported `reset` (`packages/utils/src/error.ts:5`) is the shared registry's reset,
    // so it clears the dedup state the `error` binding writes into and the same output is logged
    // again — the singleton/reset interaction the spec marks UNVERIFIED upstream
    // (specs/utils/error.md), which the ported mechanics make direct.
    #[test]
    fn reset_clears_the_shared_registry() {
        reset();

        assert_eq!(capture_log(&error(), &["boom"]), ["Base UI: boom"]);

        reset();

        assert_eq!(capture_log(&error(), &["boom"]), ["Base UI: boom"]);
    }
}
