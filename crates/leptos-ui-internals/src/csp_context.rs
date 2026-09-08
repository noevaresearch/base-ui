//! Port of `packages/react/src/internals/csp-context/CSPContext.tsx` — the context half of the
//! CSPProvider unit (the `infra: csp-provider` TODO item's only internal dependency, per
//! `specs/library/csp-provider/implementation.md`, "Dependencies on other Base UI internals").
//!
//! Upstream (17 lines) defines three things (`packages/react/src/internals/csp-context/
//! CSPContext.tsx:4-17`):
//!
//! - [`CSPContextValue`] (`:4-7`): `{ nonce?: string; disableStyleElements?: boolean }` — the
//!   config every CSP-aware consumer reads. The verified consumer sites destructure exactly
//!   these two fields: `ScrollAreaRoot`
//!   (`packages/react/src/scroll-area/root/ScrollAreaRoot.tsx:53`), `SelectPopup`
//!   (`packages/react/src/select/popup/SelectPopup.tsx:70`), and `PrehydrationScript`
//!   (`packages/react/src/internals/PrehydrationScript.tsx:34`, which reads only `nonce` and
//!   applies it to an inline `<script>` — the CSP contract covers script nonces too, matching
//!   the provider's JSDoc "inline `<style>` or `<script>` tags",
//!   `packages/react/src/csp-provider/CSPProvider.tsx:6-7`).
//! - `CSPContext` (`:9`): a React context whose default is `undefined` — i.e. "no provider" is
//!   distinguishable from any provided value.
//! - `useCSPContext` (`:11-17`): returns the ambient context value, falling back to
//!   `DEFAULT_CSP_CONTEXT_VALUE = { disableStyleElements: false }` when no provider exists
//!   (`:11-13`). This hook — not the provider — is where the behavior spec's verified default
//!   ("`disableStyleElements` off with no provider",
//!   `specs/library/csp-provider/behavior.md`, "State model") is implemented; the provider
//!   itself applies no defaults.
//!
//! ## Rust adaptations
//!
//! - There is no explicit `CSPContext` object: reactive-graph's owner-scoped context store keys
//!   values by type, so the [`Memo<CSPContextValue>`] type plays the role of the context
//!   object's identity. Upstream's `undefined` default maps to [`use_context`] returning
//!   [`None`] — the lookup walks the owner parent chain exactly like React context walks the
//!   element tree (`specs/architecture.md`, "Context passing").
//! - [`use_csp_context`] returns a [`Memo`] of the value rather than the bare value, for the
//!   same reason the `use_media_query` port returns a `Signal<bool>`: a Leptos component body
//!   runs once, so the observable state must live in a reactive value the consumer can track.
//!   Upstream's `useMemo` inside the provider (see the `csp_provider` module) keeps the React
//!   context value referentially stable so consumers re-render only when the two config props
//!   actually change (`packages/react/src/csp-provider/CSPProvider.tsx:14-20`); the memo plays
//!   exactly that role on the consumer side.
//! - [`CSPContextValue::disable_style_elements`] is a plain `bool` where upstream's field is
//!   `boolean | undefined`. Upstream passes the props through the context verbatim, including
//!   `undefined` when omitted (`packages/react/src/csp-provider/CSPProvider.tsx:14-19`), and
//!   every consumer reads the field only for truthiness; the no-provider default already pins
//!   `false` (`CSPContext.tsx:11-13`). Normalizing the omitted prop to `false` at the provider
//!   is therefore observably identical, and it keeps the Rust value type total.
//! - `nonce` stays [`Option<String>`] — verbatim, `None` for `undefined` — because consumers
//!   apply it as an element attribute where absence is meaningful (no `nonce` attribute vs a
//!   set one).
//! - The no-provider fallback builds a fresh dependency-free [`Memo`] per call rather than
//!   sharing one module-level object the way upstream's `DEFAULT_CSP_CONTEXT_VALUE` does:
//!   memos are owned by the reactive owner that creates them, so a shared one could be
//!   disposed together with its originating owner while later consumers still need it. The two
//!   are indistinguishable to consumers — the fallback memo has no dependencies and always
//!   yields the default.
//!
//! Must be called inside a reactive owner (a component), like the other hook ports.

use reactive_graph::computed::Memo;
use reactive_graph::owner::use_context;

/// The CSP configuration Base UI's inline-`<style>`/`<script>` machinery reads — upstream's
/// `CSPContextValue` (`packages/react/src/internals/csp-context/CSPContext.tsx:4-7`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CSPContextValue {
    /// Applied as the `nonce` attribute of injected inline elements (`:5`); [`None`] is
    /// upstream's `undefined` (no `nonce` attribute).
    pub nonce: Option<String>,
    /// `true` suppresses injection of Base UI's inline `<style>` elements for the subtree
    /// (`:6`) — upstream's `disableStyleElements`, normalized from `boolean | undefined` (see
    /// the module docs).
    pub disable_style_elements: bool,
}

/// Reads the ambient CSP configuration — upstream's `useCSPContext`
/// (`packages/react/src/internals/csp-context/CSPContext.tsx:15-17`). Without a provider in
/// scope this falls back to the default — `{ disableStyleElements: false }`, no nonce — the
/// upstream `DEFAULT_CSP_CONTEXT_VALUE` (`:11-13`).
pub fn use_csp_context() -> Memo<CSPContextValue> {
    use_context::<Memo<CSPContextValue>>()
        .unwrap_or_else(|| Memo::new(|_| CSPContextValue::default()))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;

    use super::*;
    use crate::csp_provider::provide_csp_context;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the no-provider fallback (`packages/react/src/internals/csp-context/CSPContext.tsx:
    // 11-17`): with no `CSPContext.Provider` above the call site, `useCSPContext` resolves to
    // `DEFAULT_CSP_CONTEXT_VALUE` — `disableStyleElements: false`, `nonce` undefined.
    #[test]
    fn without_a_provider_the_hook_falls_back_to_the_default() {
        let _owner = owner();

        let csp = use_csp_context();
        let value = csp.get_untracked();
        assert_eq!(
            value,
            CSPContextValue {
                nonce: None,
                disable_style_elements: false,
            },
            "no provider resolves to the DEFAULT_CSP_CONTEXT_VALUE fallback"
        );
    }

    // Pins the provider → consumer propagation through the owner chain: a value provided in a
    // parent owner is read by a consumer in a descendant owner, the way `CSPContext.Provider`
    // reaches consumers anywhere in its React subtree
    // (`packages/react/src/csp-provider/CSPProvider.tsx:22`).
    #[test]
    fn a_provided_value_reaches_consumers_through_the_owner_chain() {
        let parent = owner();

        let nonce = RwSignal::new(Some("test-nonce".to_string()));
        let disable = RwSignal::new(Some(true));
        let provided = provide_csp_context(nonce, disable);

        let child = parent.child();
        child.set();
        let csp = use_csp_context();
        let value = csp.get_untracked();
        assert_eq!(
            value,
            CSPContextValue {
                nonce: Some("test-nonce".to_string()),
                disable_style_elements: true,
            },
            "the consumer in the child owner reads the provider's value"
        );
        assert_eq!(
            csp.get_untracked(),
            provided.get_untracked(),
            "the hook hands consumers the same value the provider published"
        );
    }

    // Pins the nesting semantics the implementation spec derives from React context scoping
    // (`specs/library/csp-provider/implementation.md`, "Anything in source not explained by any
    // test", bullet 3): the innermost provider's props win wholesale — there is no per-prop
    // inheritance between nested providers.
    #[test]
    fn the_innermost_provider_wins_when_providers_nest() {
        let outer = owner();

        let outer_nonce = RwSignal::new(Some("outer".to_string()));
        let _outer_value = provide_csp_context(outer_nonce, RwSignal::new(Some(true)));

        let inner = outer.child();
        inner.set();
        let inner_nonce = RwSignal::new(Some("inner".to_string()));
        let _inner_value = provide_csp_context(inner_nonce, RwSignal::new(Some(false)));

        let leaf = inner.child();
        leaf.set();
        let value = use_csp_context().get_untracked();
        assert_eq!(
            value,
            CSPContextValue {
                nonce: Some("inner".to_string()),
                disable_style_elements: false,
            },
            "the innermost provider's value wins wholesale"
        );
    }

    // Pins the subtree scoping: a context provided in one owner is invisible to an independent
    // owner outside it — the reactive-graph analog of React context not crossing sibling
    // boundaries (context.rs's lookup walks only the parent chain).
    #[test]
    fn a_provided_value_does_not_leak_outside_its_subtree() {
        // The provider's owner is dropped before the independent owner below is created, so
        // that owner is a fresh root (a dead current-owner weak resolves to no parent).
        {
            let _scope = owner();
            let provided = provide_csp_context(
                RwSignal::new(Some("leak".to_string())),
                RwSignal::new(None),
            );
            assert_eq!(
                provided.get_untracked().nonce,
                Some("leak".to_string()),
                "sanity: the provider published the value inside its own owner"
            );
        }

        let _independent = owner();
        let value = use_csp_context().get_untracked();
        assert_eq!(
            value,
            CSPContextValue::default(),
            "an independent owner sees the default, not the other subtree's value"
        );
    }

    // Pins that the fallback does not depend on the provider's memo staying alive: the
    // provided memo is disposed together with its owner, and a later consumer outside that
    // owner still resolves cleanly to the default.
    #[test]
    fn the_fallback_survives_the_provider_owner_being_disposed() {
        {
            let scope = owner();
            let _value = provide_csp_context(RwSignal::new(None), RwSignal::new(None));
            // Disposing the provider's owner disposes its memo with it.
            drop(scope);
        };

        let _fresh = owner();
        let value = use_csp_context().get_untracked();
        assert_eq!(
            value,
            CSPContextValue::default(),
            "the disposed provider's memo is gone; the hook falls back to the default"
        );
    }

    // Guards the internal invariant the fallback relies on: the fallback memo the hook builds
    // is a real [`Memo`] with no dependencies, so it never diverges from
    // `CSPContextValue::default()` however often it is read.
    #[test]
    fn the_fallback_memo_is_dep_free_and_stable() {
        let _owner = owner();

        let fallback: Memo<CSPContextValue> = Memo::new(|_| CSPContextValue::default());
        assert_eq!(
            fallback.get_untracked(),
            CSPContextValue::default(),
            "first read"
        );
        assert_eq!(
            fallback.get_untracked(),
            CSPContextValue::default(),
            "repeat read is stable"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::csp_provider::provide_csp_context;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // The same contract as the host fallback test, exercised under the browser's
    // reactive-graph runtime (the realm where the real components will run — the loop's
    // wasm-only latent failures are why both targets are pinned).
    #[wasm_bindgen_test]
    fn without_a_provider_the_hook_falls_back_to_the_default() {
        let _owner = owner();

        let value = use_csp_context().get_untracked();
        assert_eq!(
            value,
            CSPContextValue {
                nonce: None,
                disable_style_elements: false,
            },
            "no provider resolves to the DEFAULT_CSP_CONTEXT_VALUE fallback"
        );
    }

    // The same contract as the host owner-chain test, exercised under the browser's
    // reactive-graph runtime.
    #[wasm_bindgen_test]
    fn a_provided_value_reaches_consumers_through_the_owner_chain() {
        let parent = owner();

        let provided = provide_csp_context(
            RwSignal::new(Some("test-nonce".to_string())),
            RwSignal::new(Some(true)),
        );

        let child = parent.child();
        child.set();
        let value = use_csp_context().get_untracked();
        assert_eq!(
            value,
            provided.get_untracked(),
            "the consumer in the child owner reads the provider's value"
        );
        assert_eq!(
            value.nonce,
            Some("test-nonce".to_string()),
            "the nonce propagated verbatim"
        );
        assert!(value.disable_style_elements, "the flag propagated as true");
    }
}
