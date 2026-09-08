//! Port of `packages/react/src/direction-provider/DirectionProvider.tsx` — the provider half
//! of the `infra: direction-provider` TODO item (`specs/library/direction-provider/behavior.md`,
//! `specs/library/direction-provider/implementation.md`).
//!
//! Upstream is a ~25-line stateless context provider with no state machine and no effects
//! (`packages/react/src/direction-provider/DirectionProvider.tsx:13-21`): it applies a
//! writer-side default to the one prop (`const { direction = 'ltr' } = props`, `:16`),
//! memoizes `{ direction }` against `[direction]` (`:17`), and returns a bare
//! `DirectionContext.Provider` wrapping `children` (`:18-20`) — it renders no DOM node of its
//! own and sets no `dir` attribute (resolving behavior.md's UNVERIFIED "Accessibility" and
//! "DOM structure & portal behavior" sections: "there is no wrapper element and no `dir`
//! attribute is ever set — DOM directionality is entirely the application's concern",
//! implementation.md, "DOM/portal strategy and why").
//!
//! Two mechanisms matter:
//!
//! - The memo's purpose is referential stability of the context value: any provider re-render
//!   with an unchanged `direction` reuses the same context object, and React's context
//!   identity bail-out skips re-rendering every consumer below; when the prop does change, a
//!   fresh object fans out to all consumers — "this is the entire mechanism behind the live
//!   `rtl → ltr` transition verified in behavior.md's 'State model' section"
//!   (implementation.md, "State machine / hooks used").
//! - The writer-side default (`:16`) means under any provider the value is always a
//!   fully-formed `{ direction }` — `undefined` never enters the context. Unlike CSPProvider,
//!   which forwards props verbatim including `undefined`, this unit defaults at the writer
//!   (implementation.md, "State machine / hooks used"); the hook's `?? 'ltr'` fallback
//!   ([`crate::direction_context::use_direction`]) is belt-and-suspenders — unreachable under
//!   any provider, firing only with no provider in scope (implementation.md's "Double-default
//!   note"). Behavior.md flags the writer-side default as UNVERIFIED (no test renders the
//!   provider without `direction`); this port pins it.
//!
//! ## Rust adaptations
//!
//! - The prop is a reactive source ([`Get`] of `Option<TextDirection>`) rather than a plain
//!   per-render value: that is what Leptos component props are (`MaybeProp<TextDirection>`
//!   satisfies the bound, so the future `#[component]` view wrapper forwards its prop straight
//!   through), and it is what lets the memo below track it — the tracked read is the
//!   `[direction]` deps array (`:17`).
//! - The `useMemo` + `DirectionContext.Provider` pair dissolves into one [`Memo`] handed to
//!   reactive-graph's owner-scoped [`provide_context`]: same publish-subscribe shape, keyed by
//!   type instead of a context object (`specs/architecture.md`, "Context passing").
//! - The children rendering half (upstream `:18-20`) is not part of this primitive: this crate
//!   is view-free (no `leptos` dependency — the same convention as the `csp_provider` module),
//!   so the `#[component]` wrapper that renders `children` in place with no host element lands
//!   with the crate's first view-layer consumer (Phase C's docs-app). Everything observable
//!   about the provider — the value's shape, the writer-side default, memoization, and subtree
//!   visibility — is complete and pinned here.
//! - `'use client'` (`:1`) is N/A — there is no React Server Components boundary in Rust. The
//!   `DirectionProviderState` empty interface and namespace (`:23`, `:34-37`) are dead part-API
//!   surface with no runtime meaning (implementation.md, "Anything in source" bullet 4) — no
//!   Rust counterpart is warranted.
//!
//! Must be called inside a reactive owner (a component), like the other hook ports.

use reactive_graph::computed::Memo;
use reactive_graph::owner::provide_context;
use reactive_graph::traits::Get;

use crate::direction_context::{DirectionContextValue, TextDirection};

/// Publishes the reading direction for the current subtree — upstream's `DirectionProvider`
/// body (`packages/react/src/direction-provider/DirectionProvider.tsx:13-21`): applies the
/// writer-side default (`:16`), memoizes `{ direction }` against the prop (`:17`), and
/// provides the value to the reactive owner scope. Returns the provided memo (the memoized
/// context value).
///
/// An omitted direction ([`None`]) resolves to [`TextDirection::Ltr`] — the writer-side
/// default (`:16`). The provided value updates reactively when the prop changes and only then
/// — the deps array contract (`:17`).
pub fn provide_direction_context<D>(direction: D) -> Memo<DirectionContextValue>
where
    D: Get<Value = Option<TextDirection>> + Send + Sync + 'static,
{
    let value = Memo::new(move |_| DirectionContextValue {
        direction: direction.get().unwrap_or(TextDirection::Ltr),
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
    use crate::direction_context::use_direction;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the provided value's shape (`packages/react/src/direction-provider/
    // DirectionProvider.tsx:17`): a single-field `{ direction }` carrying the configured
    // value — the verified "provides the configured direction to descendants" case
    // (`packages/react/src/direction-provider/DirectionProvider.test.tsx:33-36`).
    #[test]
    fn the_provided_value_carries_the_configured_direction() {
        let _owner = owner();

        let provided = provide_direction_context(RwSignal::new(Some(TextDirection::Rtl)));
        assert_eq!(
            provided.get_untracked(),
            DirectionContextValue {
                direction: TextDirection::Rtl,
            },
            "the configured direction crosses the boundary as given"
        );
    }

    // Pins the writer-side default (`packages/react/src/direction-provider/
    // DirectionProvider.tsx:16`): an omitted `direction` — upstream's `undefined` — never
    // enters the context; under any provider the value is a fully-formed `{ direction: 'ltr' }`.
    // Behavior.md flags exactly this gap as UNVERIFIED ("no test renders `<DirectionProvider>`
    // without a `direction` prop", implementation.md, "Anything in source" bullet 1); the port
    // pins the source-explainable contract.
    #[test]
    fn an_omitted_direction_defaults_to_ltr_in_the_provided_value() {
        let _owner = owner();

        let provided = provide_direction_context(RwSignal::new(None));
        assert_eq!(
            provided.get_untracked(),
            DirectionContextValue {
                direction: TextDirection::Ltr,
            },
            "the writer-side default resolves an omitted prop to 'ltr'"
        );
    }

    // Pins the memoization contract (`:17`): the value recomputes when the direction prop
    // changes and only then — the tracked read of the prop is the `[direction]` deps array.
    // This is the reactive shape of the verified live `rtl → ltr` transition
    // (`packages/react/src/direction-provider/DirectionProvider.test.tsx:38-41`).
    #[test]
    fn the_provided_value_updates_when_the_direction_changes() {
        let _owner = owner();

        let direction_signal = RwSignal::new(Some(TextDirection::Rtl));
        let provided = provide_direction_context(direction_signal);
        assert_eq!(
            provided.get_untracked().direction,
            TextDirection::Rtl,
            "starts at the configured 'rtl'"
        );

        direction_signal.set(Some(TextDirection::Ltr));
        assert_eq!(
            provided.get_untracked().direction,
            TextDirection::Ltr,
            "a direction change recomputes the value"
        );
    }

    // Pins that a value provided under one direction does not bleed into an independent owner:
    // the provider scopes to the reactive owner, the way `DirectionContext.Provider` scopes to
    // its React subtree (`:18-20`).
    #[test]
    fn the_provided_value_is_scoped_to_the_provider_subtree() {
        let _outside = owner();
        let outside_read = use_direction().get_untracked();

        {
            let _inside = owner();
            let _provided = provide_direction_context(RwSignal::new(Some(TextDirection::Rtl)));
            assert_eq!(
                use_direction().get_untracked(),
                TextDirection::Rtl,
                "the provider's own scope reads the provided value"
            );
        }

        assert_eq!(
            use_direction().get_untracked(),
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
    use crate::direction_context::use_direction;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // The same contract as the host writer-side-default test, exercised under the browser's
    // reactive-graph runtime.
    #[wasm_bindgen_test]
    fn an_omitted_direction_defaults_to_ltr_in_the_provided_value() {
        let _owner = owner();

        let _provided = provide_direction_context(RwSignal::new(None));
        assert_eq!(
            use_direction().get_untracked(),
            TextDirection::Ltr,
            "the writer-side default resolves an omitted prop to 'ltr'"
        );
    }

    // The same contract as the host reactivity test, exercised under the browser's
    // reactive-graph runtime.
    #[wasm_bindgen_test]
    fn the_provided_value_updates_when_the_direction_changes() {
        let _owner = owner();

        let direction_signal = RwSignal::new(Some(TextDirection::Rtl));
        let provided = provide_direction_context(direction_signal);
        assert_eq!(
            provided.get_untracked().direction,
            TextDirection::Rtl,
            "starts at the configured 'rtl'"
        );

        direction_signal.set(Some(TextDirection::Ltr));
        assert_eq!(
            provided.get_untracked().direction,
            TextDirection::Ltr,
            "a direction change recomputes the provided value"
        );
    }
}
