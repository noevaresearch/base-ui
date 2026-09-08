//! Port of `packages/react/src/internals/direction-context/DirectionContext.tsx` — the context
//! half of the direction-provider unit (the `infra: direction-provider` TODO item's only
//! internal dependency, per `specs/library/direction-provider/implementation.md`,
//! "Dependencies on other Base UI internals": "Exactly one internal import: `DirectionContext`
//! and `TextDirection` from `internals/direction-context/DirectionContext`. That one file
//! supplies the unit's entire mechanism — context object, consumer hook, union type.").
//!
//! Upstream (16 lines) defines four things (`packages/react/src/internals/direction-context/
//! DirectionContext.tsx:4-15`):
//!
//! - [`TextDirection`] (`:4`): the closed `'ltr' | 'rtl'` union — the entire reading-direction
//!   vocabulary Base UI consumes. The type contract pins it closed
//!   (`packages/react/src/direction-provider/DirectionProvider.spec.tsx:25-27`,
//!   `@ts-expect-error` on `direction="vertical"`); the Rust enum makes that closure
//!   unrepresentable-to-violate rather than merely compiler-asserted.
//! - [`DirectionContextValue`] (`:6-8`): the context value shape `{ direction: TextDirection }`.
//!   The only field crossing the boundary is `direction` — the implementation spec's
//!   "Context providers/consumers" table: "The only field crossing the boundary is `direction`".
//! - `DirectionContext` (`:10`): a React context whose default is `undefined` — i.e. "no
//!   provider" is distinguishable from any provided value.
//! - [`use_direction`] (`:12-15`): the consumer hook, applying the fallback
//!   `context?.direction ?? 'ltr'`. This hook — not the provider — is where the behavior
//!   spec's verified default ("outside any `<DirectionProvider>`, `useDirection()` returns
//!   `'ltr'`", `specs/library/direction-provider/behavior.md`, "State model",
//!   `packages/react/src/direction-provider/DirectionProvider.test.tsx:27-31`) is implemented.
//!
//! ## Rust adaptations
//!
//! - There is no explicit `DirectionContext` object: reactive-graph's owner-scoped context
//!   store keys values by type, so the [`Memo<DirectionContextValue>`] type plays the role of
//!   the context object's identity. Upstream's `undefined` default maps to [`use_context`]
//!   returning [`None`] — the lookup walks the owner parent chain exactly like React context
//!   walks the element tree (`specs/architecture.md`, "Context passing").
//! - [`use_direction`] returns a [`Memo`] of the scalar direction rather than the bare value,
//!   for the same reason the `use_media_query` port returns a `Signal<bool>`: a Leptos
//!   component body runs once, so the observable state must live in a reactive value the
//!   consumer can track. Upstream's hook re-runs on every context change because React
//!   re-renders subscribers; here the returned memo re-computes under the same trigger. The
//!   scalar return (not the context value struct) mirrors upstream exactly: `useDirection()`
//!   returns the direction itself, never the `{ direction }` wrapper — the compile-time
//!   contract at `packages/react/src/direction-provider/DirectionProvider.spec.tsx:10-12`
//!   asserts the return is exactly `TextDirection`, never `undefined`.
//! - [`DirectionContextValue::default`] yields `{ direction: Ltr }`. This is the
//!   hook-side observable (the `?? 'ltr'` fallback), not the context default — upstream's
//!   context default is absence (`:10`, `undefined`), which stays [`use_context`]'s [`None`]
//!   here. The `Default` impl exists so the no-provider memo and tests can name the same
//!   value the fallback produces.
//! - [`TextDirection::as_str`] pins the exact lowercase tokens the upstream string union
//!   carries — the text the docs probe renders directly as content
//!   (`packages/react/src/direction-provider/DirectionProvider.test.tsx:11-14`, asserting
//!   `'ltr'`/`'rtl'` at `:30`/`:36`/`:40`). `Display` delegates to it so the future view layer
//!   renders the same text.
//!
//! Must be called inside a reactive owner (a component), like the other hook ports.

use core::fmt;

use reactive_graph::computed::Memo;
use reactive_graph::owner::use_context;
use reactive_graph::traits::Get;

/// The reading direction Base UI components consume — upstream's `TextDirection`
/// (`packages/react/src/internals/direction-context/DirectionContext.tsx:4`), the closed
/// `'ltr' | 'rtl'` union.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextDirection {
    /// Left-to-right. The default at every defaulting site: the provider's writer-side default
    /// (`packages/react/src/direction-provider/DirectionProvider.tsx:16`) and the hook's
    /// no-provider fallback (`packages/react/src/internals/direction-context/
    /// DirectionContext.tsx:15`).
    #[default]
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl TextDirection {
    /// The exact lowercase token of the upstream string union — the text a consumer renders
    /// for the value (`packages/react/src/direction-provider/DirectionProvider.test.tsx:11-14`).
    pub fn as_str(self) -> &'static str {
        match self {
            TextDirection::Ltr => "ltr",
            TextDirection::Rtl => "rtl",
        }
    }
}

impl fmt::Display for TextDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The value crossing the context boundary — upstream's `DirectionContext` type
/// (`packages/react/src/internals/direction-context/DirectionContext.tsx:6-8`), the
/// single-field `{ direction: TextDirection }`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectionContextValue {
    /// The reading direction for the provider's subtree (`:7`). The only field crossing the
    /// boundary — the provider is the codebase's only writer and every reader goes through
    /// [`use_direction`] (`specs/library/direction-provider/implementation.md`,
    /// "Context providers/consumers").
    pub direction: TextDirection,
}

/// Reads the ambient reading direction — upstream's `useDirection`
/// (`packages/react/src/internals/direction-context/DirectionContext.tsx:12-15`): without a
/// provider in scope this falls back to [`TextDirection::Ltr`], the upstream
/// `context?.direction ?? 'ltr'` (`:15`). Under a provider it returns a memo derived from the
/// provided value, so consumers track direction changes live — the verified `rtl → ltr`
/// transition (`specs/library/direction-provider/behavior.md`, "State model",
/// `packages/react/src/direction-provider/DirectionProvider.test.tsx:38-41`).
///
/// The no-provider fallback builds a fresh dependency-free [`Memo`] per call rather than
/// sharing one module-level object: memos are owned by the reactive owner that creates them,
/// so a shared one could be disposed together with its originating owner while later consumers
/// still need it (the same convention as the `csp_context` fallback). The two are
/// indistinguishable to consumers — the fallback memo has no dependencies and always yields
/// [`TextDirection::Ltr`].
pub fn use_direction() -> Memo<TextDirection> {
    match use_context::<Memo<DirectionContextValue>>() {
        Some(value) => Memo::new(move |_| value.get().direction),
        None => Memo::new(|_| TextDirection::Ltr),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};

    use super::*;
    use crate::direction_provider::provide_direction_context;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the no-provider fallback (`packages/react/src/internals/direction-context/
    // DirectionContext.tsx:12-15`): with no `DirectionContext.Provider` above the call site,
    // `useDirection` resolves to `'ltr'` — the upstream `context?.direction ?? 'ltr'`. This is
    // the verified default of behavior.md's "State model"
    // (`packages/react/src/direction-provider/DirectionProvider.test.tsx:27-31`).
    #[test]
    fn without_a_provider_the_hook_falls_back_to_ltr() {
        let _owner = owner();

        let direction = use_direction().get_untracked();
        assert_eq!(
            direction,
            TextDirection::Ltr,
            "no provider resolves to the hook's 'ltr' fallback"
        );
    }

    // Pins the provider → consumer propagation through the owner chain: a direction provided
    // in a parent owner is read by a consumer in a descendant owner, the way
    // `DirectionContext.Provider` reaches consumers anywhere in its React subtree
    // (`packages/react/src/direction-provider/DirectionProvider.tsx:18-20`). Mirrors the
    // verified "provides the configured direction to descendants" test
    // (`packages/react/src/direction-provider/DirectionProvider.test.tsx:33-36`).
    #[test]
    fn a_provided_direction_reaches_consumers_through_the_owner_chain() {
        let parent = owner();

        let provided = provide_direction_context(RwSignal::new(Some(TextDirection::Rtl)));

        let child = parent.child();
        child.set();
        let direction = use_direction();
        assert_eq!(
            direction.get_untracked(),
            TextDirection::Rtl,
            "the consumer in the child owner reads the provider's direction"
        );
        assert_eq!(
            direction.get_untracked(),
            provided.get_untracked().direction,
            "the hook hands consumers the same value the provider published"
        );
    }

    // Pins the live transition (`packages/react/src/direction-provider/
    // DirectionProvider.test.tsx:38-41`): updating the provider's `direction` prop
    // ('rtl' → 'ltr') updates the value seen by descendants without remounting them —
    // here, the hook's memo recomputes from the same provided value. Context propagation is
    // a re-render of subscribers, never a remount (implementation.md, "State machine / hooks
    // used").
    #[test]
    fn the_hook_value_tracks_the_provided_direction_reactively() {
        let _owner = owner();

        let direction_signal = RwSignal::new(Some(TextDirection::Rtl));
        let _provided = provide_direction_context(direction_signal);
        let direction = use_direction();
        assert_eq!(
            direction.get_untracked(),
            TextDirection::Rtl,
            "starts at the configured 'rtl'"
        );

        direction_signal.set(Some(TextDirection::Ltr));
        assert_eq!(
            direction.get_untracked(),
            TextDirection::Ltr,
            "the 'rtl' → 'ltr' prop update propagates live"
        );
    }

    // Pins the nesting semantics the implementation spec derives from React context scoping
    // (`specs/library/direction-provider/implementation.md`, "Anything in source not explained
    // by any test", bullet 5: "the source contains no merge/override logic, so by React context
    // scoping the innermost provider's value wins wholesale"). Behavior.md flags nesting as
    // UNVERIFIED by tests; the port pins the source-explainable outcome.
    #[test]
    fn the_innermost_provider_wins_when_providers_nest() {
        let outer = owner();

        let _outer_value = provide_direction_context(RwSignal::new(Some(TextDirection::Ltr)));

        let inner = outer.child();
        inner.set();
        let _inner_value = provide_direction_context(RwSignal::new(Some(TextDirection::Rtl)));

        let leaf = inner.child();
        leaf.set();
        assert_eq!(
            use_direction().get_untracked(),
            TextDirection::Rtl,
            "the innermost provider's direction wins wholesale"
        );
    }

    // Pins the string tokens (`specs/library/direction-provider/behavior.md`, "Public API
    // surface": "`useDirection()` returns a string value that renders as text ('ltr' /
    // 'rtl')"): the enum's text form is exactly the upstream union's members — the probe
    // asserts these literal strings
    // (`packages/react/src/direction-provider/DirectionProvider.test.tsx:30`, `:36`, `:40`).
    #[test]
    fn the_text_direction_tokens_render_as_the_upstream_strings() {
        assert_eq!(TextDirection::Ltr.as_str(), "ltr");
        assert_eq!(TextDirection::Rtl.as_str(), "rtl");
        assert_eq!(TextDirection::Ltr.to_string(), "ltr");
        assert_eq!(TextDirection::Rtl.to_string(), "rtl");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::direction_provider::provide_direction_context;

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
    fn without_a_provider_the_hook_falls_back_to_ltr() {
        let _owner = owner();

        assert_eq!(
            use_direction().get_untracked(),
            TextDirection::Ltr,
            "no provider resolves to the hook's 'ltr' fallback"
        );
    }

    // The same contract as the host owner-chain + live-transition tests, exercised under the
    // browser's reactive-graph runtime.
    #[wasm_bindgen_test]
    fn a_provided_direction_reaches_consumers_and_tracks_changes() {
        let parent = owner();

        let direction_signal = RwSignal::new(Some(TextDirection::Rtl));
        let _provided = provide_direction_context(direction_signal);

        let child = parent.child();
        child.set();
        let direction = use_direction();
        assert_eq!(
            direction.get_untracked(),
            TextDirection::Rtl,
            "the consumer in the child owner reads the provider's direction"
        );

        direction_signal.set(Some(TextDirection::Ltr));
        assert_eq!(
            direction.get_untracked(),
            TextDirection::Ltr,
            "the 'rtl' → 'ltr' prop update propagates live"
        );
    }
}
