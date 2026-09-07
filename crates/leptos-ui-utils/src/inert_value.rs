//! Port of `packages/utils/src/inertValue.ts` (Base UI Phase A util).
//!
//! Upstream computes the value to assign to a DOM element's `inert` attribute from an
//! optional boolean flag — call sites pass the inverse of an open/active state, e.g.
//! `inert={inertValue(!open)}`
//! (`packages/react/src/dialog/portal/DialogPortal.tsx:37`,
//! `packages/react/src/tabs/panel/TabsPanel.tsx:73`). It has no test file upstream —
//! `ralph/generated/utils.json:126-131` lists `testFiles: []` — so every behavioral claim is
//! inferred from the unit's own source (`specs/utils/inertValue.md`) and pinned by this
//! module's own tests rather than a reference suite.
//!
//! Upstream behavior is gated on the React major version
//! (`packages/utils/src/inertValue.ts:3-9`): on React ≥ 19 the input passes through unchanged
//! (`packages/utils/src/inertValue.ts:4-5`); on React < 19 truthy input returns the string
//! `'true'` — so React < 19, which lacked native boolean `inert` prop support, still renders
//! the attribute — and falsy input returns `undefined` so the attribute is omitted
//! (`packages/utils/src/inertValue.ts:7-8`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The React < 19 compatibility branch (`packages/utils/src/inertValue.ts:7-8`) is
//!   deliberately not replicated: it exists to satisfy a React-runtime constraint that Leptos
//!   does not have. With no React runtime to check, the version gate
//!   (`isReactVersionAtLeast(19)`, `packages/utils/src/inertValue.ts:1`,
//!   `packages/utils/src/reactVersion.ts:3`, `packages/utils/src/reactVersion.ts:7-8`) and its
//!   module-load-time `React.version` snapshot collapse away entirely — this module has no
//!   counterpart dependency on a future `reactVersion` port, and [`inert_value`] implements
//!   the passthrough branch's mapping for every call (`specs/utils/inertValue.md`, "Edge
//!   cases").
//! - The optional `value?: boolean` parameter and the declared `boolean | undefined` return
//!   (`packages/utils/src/inertValue.ts:3`) both map to [`Option<bool>`]: `None` stands for
//!   the omitted argument and for upstream's falsy-input `undefined` return — the
//!   omitted-attribute case both upstream branches produce.

/// Returns the value to assign to a DOM element's `inert` attribute — the port of upstream
/// `inertValue` (`packages/utils/src/inertValue.ts:3-9`): the input passes through unchanged.
///
/// Compute it from an open/active flag the way upstream call sites do, e.g.
/// `inert={inertValue(!open)}`
/// (`packages/react/src/dialog/portal/DialogPortal.tsx:37`,
/// `packages/react/src/tabs/panel/TabsPanel.tsx:73`). [`Some`] values flow through as-is and
/// `None` (upstream's omitted argument) flows through as `None`. See the module docs for why
/// the React < 19 string-`'true'` branch is not replicated.
pub fn inert_value(value: Option<bool>) -> Option<bool> {
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pins the passthrough mapping (`packages/utils/src/inertValue.ts:4-5`,
    // `specs/utils/inertValue.md` "Edge cases"): truthy input comes back unchanged. Upstream
    // has no test file (`ralph/generated/utils.json:126-131`), so this is the port's own
    // pinning of the inferred contract, not a mirror of a reference suite.
    #[test]
    fn passes_some_true_through() {
        assert_eq!(inert_value(Some(true)), Some(true));
    }

    // `inertValue(false)` → `false` on the passthrough branch — NOT `undefined`: the
    // attribute-omission for a `false` value is the caller's falsy-attribute semantics, not
    // this function's (`packages/utils/src/inertValue.ts:4-5`).
    #[test]
    fn passes_some_false_through() {
        assert_eq!(inert_value(Some(false)), Some(false));
    }

    // `inertValue()` → `undefined` (`packages/utils/src/inertValue.ts:4-5`): the omitted
    // argument is the port's `None`.
    #[test]
    fn passes_none_through() {
        assert!(inert_value(None).is_none());
    }
}
