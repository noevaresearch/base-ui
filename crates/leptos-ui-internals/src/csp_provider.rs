//! Port of `packages/react/src/csp-provider/CSPProvider.tsx` — the provider half of the
//! `infra: csp-provider` TODO item (`specs/library/csp-provider/behavior.md`,
//! `specs/library/csp-provider/implementation.md`).
//!
//! Upstream is a ~20-line stateless context provider with no state machine and no effects
//! (`packages/react/src/csp-provider/CSPProvider.tsx:11-23`): it destructures
//! `{ children, nonce, disableStyleElements }` from its props, memoizes the two config props
//! into a `CSPContextValue` against a `[nonce, disableStyleElements]` deps array (`:14-20`),
//! and returns a bare `CSPContext.Provider` wrapping `children` (`:22`) — it renders no DOM
//! node of its own (resolving behavior.md's UNVERIFIED wrapper-element question: there is no
//! wrapper; children mount in place with no host element added).
//!
//! The memo's purpose is referential stability of the context value: without it, every
//! re-render of the provider (triggered by any parent re-render, even with unchanged props)
//! would produce a fresh context object and re-render every context consumer in the subtree;
//! the memo makes consumer re-renders fire only when the two config props actually change
//! (`specs/library/csp-provider/implementation.md`, "State machine / hooks used"). The provider
//! applies no defaults itself — the `disableStyleElements: false` default lives in
//! [`use_csp_context`]'s fallback (`packages/react/src/internals/csp-context/CSPContext.tsx:
//! 11-17`).
//!
//! ## Rust adaptations
//!
//! - The two config props are reactive sources ([`Get`] of `Option<…>`) rather than plain
//!   per-render values: that is what Leptos component props are (`MaybeProp<String>` /
//!   `MaybeProp<bool>` both satisfy these bounds, so the future `#[component]` view wrapper
//!   forwards its props straight through), and it is what lets the memo below track them —
//!   the tracked reads are the `[nonce, disableStyleElements]` deps array (`:14-20`).
//! - The `useMemo` + `CSPContext.Provider` pair dissolves into one [`Memo`] handed to
//!   reactive-graph's owner-scoped [`provide_context`]: same publish-subscribe shape, keyed by
//!   type instead of a context object (`specs/architecture.md`, "Context passing").
//! - The children rendering half (upstream `:22`) is not part of this primitive: this crate is
//!   view-free (no `leptos` dependency — see `use_media_query`'s realm convention), so the
//!   `#[component]` wrapper that renders `children` in place with no host element lands with
//!   the crate's first view-layer consumer (Phase C's docs-app). Everything observable about
//!   the provider — the value's shape, memoization, and subtree visibility — is complete and
//!   pinned here.
//! - The omitted `disableStyleElements` prop (`None`) normalizes to `false` in the provided
//!   value where upstream passes `undefined` through verbatim — observably identical, since
//!   every consumer reads the field only for truthiness and the hook's no-provider fallback
//!   already pins `false` (see the `csp_context` module docs).
//! - `'use client'` (`:1`) is N/A — there is no React Server Components boundary in Rust.
//!
//! Must be called inside a reactive owner (a component), like the other hook ports.

use reactive_graph::computed::Memo;
use reactive_graph::owner::provide_context;
use reactive_graph::traits::Get;

use crate::csp_context::CSPContextValue;

/// Publishes the CSP configuration for the current subtree — upstream's `CSPProvider` body
/// (`packages/react/src/csp-provider/CSPProvider.tsx:11-23`): memoizes the two config props
/// into a [`CSPContextValue`] and provides it to the reactive owner scope. Returns the
/// provided memo (the memoized context value).
///
/// The provided value updates reactively when either prop changes and only then — the deps
/// array contract (`:14-20`). `nonce` passes through verbatim (`None` stays `None`, `:16`);
/// an omitted `disableStyleElements` normalizes to `false` (see the module docs).
pub fn provide_csp_context<N, D>(nonce: N, disable_style_elements: D) -> Memo<CSPContextValue>
where
    N: Get<Value = Option<String>> + Send + Sync + 'static,
    D: Get<Value = Option<bool>> + Send + Sync + 'static,
{
    let value = Memo::new(move |_| CSPContextValue {
        nonce: nonce.get(),
        disable_style_elements: disable_style_elements.get().unwrap_or(false),
    });
    provide_context(value.clone());
    value
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};

    use super::*;
    use crate::csp_context::use_csp_context;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the memoized value's shape (`packages/react/src/csp-provider/CSPProvider.tsx:
    // 14-19`): both props cross the boundary verbatim — `nonce` as given, including `None`
    // when omitted upstream; behavior.md's nonce test (`:50-62`) asserts the provided nonce
    // reaches the injected `<style>` attribute.
    #[test]
    fn the_provided_value_carries_the_props() {
        let _owner = owner();

        let provided = provide_csp_context(
            RwSignal::new(Some("test-nonce".to_string())),
            RwSignal::new(Some(true)),
        );
        assert_eq!(
            provided.get_untracked(),
            CSPContextValue {
                nonce: Some("test-nonce".to_string()),
                disable_style_elements: true,
            },
            "both props cross the boundary as given"
        );
    }

    // Pins the omitted-prop normalization (see the module docs): an omitted
    // `disableStyleElements` — upstream's `undefined`, falsy at every consumer — normalizes to
    // `false`, and an omitted `nonce` stays `None` verbatim. This is the "plain provider"
    // case of behavior.md's default test (`:64-75`): styles stay injected.
    #[test]
    fn an_omitted_disable_style_elements_normalizes_to_false() {
        let _owner = owner();

        let provided = provide_csp_context(RwSignal::new(None), RwSignal::new(None));
        assert_eq!(
            provided.get_untracked(),
            CSPContextValue {
                nonce: None,
                disable_style_elements: false,
            },
            "omitted props resolve to no-nonce and styles-injected"
        );
    }

    // Pins the memoization contract (`:14-20`): the value recomputes when either config prop
    // changes and only then — tracked reads of the two props are the deps array.
    #[test]
    fn the_memoized_value_updates_when_the_props_change() {
        let _owner = owner();

        let nonce = RwSignal::new(None);
        let disable = RwSignal::new(None);
        let provided = provide_csp_context(nonce, disable);
        assert_eq!(
            provided.get_untracked(),
            CSPContextValue::default(),
            "the value starts at the normalized default"
        );

        nonce.set(Some("rotated".to_string()));
        assert_eq!(
            provided.get_untracked().nonce,
            Some("rotated".to_string()),
            "a nonce change recomputes the value"
        );

        disable.set(Some(true));
        assert!(
            provided.get_untracked().disable_style_elements,
            "a disableStyleElements change recomputes the value"
        );
    }

    // Pins that a value provided under one config does not bleed into an independent owner:
    // the provider scopes to the reactive owner, the way `CSPContext.Provider` scopes to its
    // React subtree (`:22`).
    #[test]
    fn the_provided_value_is_scoped_to_the_provider_subtree() {
        let _outside = owner();
        let outside_read = use_csp_context().get_untracked();

        {
            let _inside = owner();
            let _provided =
                provide_csp_context(RwSignal::new(Some("inner".to_string())), RwSignal::new(None));
            assert_eq!(
                use_csp_context().get_untracked().nonce,
                Some("inner".to_string()),
                "the provider's own scope reads the provided value"
            );
        }

        assert_eq!(
            use_csp_context().get_untracked(),
            outside_read,
            "outside the provider's scope the ambient value is unchanged"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::csp_context::use_csp_context;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // The same contract as the host reactivity test, exercised under the browser's
    // reactive-graph runtime.
    #[wasm_bindgen_test]
    fn the_memoized_value_updates_when_the_props_change() {
        let _owner = owner();

        let nonce = RwSignal::new(None);
        let provided = provide_csp_context(nonce, RwSignal::new(None));
        assert_eq!(
            provided.get_untracked(),
            CSPContextValue::default(),
            "the value starts at the normalized default"
        );

        nonce.set(Some("browser-nonce".to_string()));
        assert_eq!(
            use_csp_context().get_untracked().nonce,
            Some("browser-nonce".to_string()),
            "a nonce change recomputes the provided value"
        );
    }
}
