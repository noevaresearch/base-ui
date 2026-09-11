//! The docs page for `DirectionProvider`, mirroring
//! `docs/src/app/(docs)/react/utils/direction-provider/page.mdx`
//! (`specs/docs-content/direction-provider/page.md`) — the
//! `docs-content: utils/direction-provider` TODO item.
//!
//! The upstream page renders one hero demo (`DemoDirectionProviderHero`), whose component
//! parts are `DirectionProvider` + `Slider.Root/Control/Track/Indicator/Thumb`
//! (`specs/docs-content/direction-provider/demos.json`,
//! `docs/src/app/(docs)/react/utils/direction-provider/demos/hero/tailwind/index.tsx:1-21`).
//! `library: slider` is a Phase B component not yet ported, so the demo's Slider subtree
//! cannot be built on real ported machinery — and the loop's rules forbid a fabricated
//! stub demo standing in for it. Per the csp-provider precedent (the page whose spec
//! records "no demos/ to mine"), the done-when's "using crates/leptos-ui-internals's real
//! implementation" is discharged instead by building the page's live machinery on the real
//! ported provider + hook:
//!
//! - [`DirectionProviderView`] is the view-layer wrapper the `direction_provider` port's
//!   module docs deferred to "the crate's first view-layer consumer" — it renders
//!   `children` in place with no host element and no `dir` attribute (upstream returns the
//!   bare `DirectionContext.Provider`, `packages/react/src/direction-provider/
//!   DirectionProvider.tsx:18-20`; behavior.md's UNVERIFIED wrapper/dir-attribute question
//!   resolves to "no wrapper") and publishes the direction through
//!   `leptos_ui_internals::provide_direction_context` (the writer-side-default + memoize
//!   pair, `:16-17`).
//! - [`DirectionProbe`] reads it back through the real `use_direction` hook
//!   (`packages/react/src/internals/direction-context/DirectionContext.tsx:12-15`) and
//!   displays the ambient direction — the shape every real consumer takes.
//! - The page carries the demo's `dir="rtl"` native-attribute div (the upstream hero's
//!   outer `<div dir="rtl">`, `demos/hero/tailwind/index.tsx:6`) to illustrate the docs
//!   caveat that `<DirectionProvider>` does not affect HTML and CSS — applications set
//!   `dir`/CSS themselves (`page.mdx:26`).
//!
//! Page structure per the spec's "Page structure" section: `# Direction Provider` h1,
//! `<Subtitle>`, the hero-demo slot (its real-machinery analog below), `## Anatomy` with
//! the embedded snippet, the HTML/CSS caveat paragraph, and `## API reference` with
//! `### DirectionProvider` and `### useDirection` (the generated `Types*` tables are
//! documentation furniture — echoed as the inline prop/hook summaries the spec mines from
//! `types.md:7-42`, since docs-app has no generated-props pipeline).

use leptos::prelude::*;
use reactive_graph::owner::Owner;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Get as _;
use leptos_ui_internals::direction_context::{TextDirection, use_direction};
use leptos_ui_internals::direction_provider::provide_direction_context;

/// The view-layer half of the ported `DirectionProvider`
/// (`packages/react/src/direction-provider/DirectionProvider.tsx:13-21`): publishes the
/// configured direction as the subtree's direction context through the real
/// `provide_direction_context` port and renders `children` in place — no host element,
/// no `dir` attribute, matching upstream's bare `DirectionContext.Provider` return
/// (`:18-20`).
///
/// An omitted `direction` stays absent across the boundary: the port's writer-side default
/// (`:16`) resolves it to `'ltr'` inside `provide_direction_context`, exactly as upstream's
/// `const { direction = 'ltr' } = props` does — `undefined` never enters the context.
#[component]
pub fn DirectionProviderView(
    direction: Option<TextDirection>,
    children: Children,
) -> impl IntoView {
    let owner = Owner::new();
    // Cross-crate owner bridge (documented adaptation, the csp_provider_page
    // precedent): the internals crate is reactive-graph-0.2-only while this view
    // crate runs on leptos 0.7 (reactive-graph 0.1) — two independent reactive
    // runtimes in one process. The provider's context publish and the hook's
    // context lookup must see the SAME owner chain, so this wrapper creates a
    // real reactive-graph-0.2 owner and runs the real `provide_direction_context`
    // and the children build inside it — exactly the window in which a child
    // component body executes and `use_direction` resolves its context.
    //
    // `Owner::with` (not a manual `set()` guard): `Owner::set` OVERWRITES the
    // thread-local current owner without saving the previous one
    // (reactive_graph-0.2.14 owner.rs:265-266; only `with` saves and restores it,
    // :280-288). With a bare guard the thread-local stays pointing at this
    // provider's owner after the view builds, so a LATER sibling view mounted
    // outside any provider would resolve through the leaked provider chain and
    // wrongly inherit this provider's direction instead of the 'ltr' fallback —
    // caught by the wasm suite (the bare probe read `direction: rtl`). `with`
    // restores the previous owner, matching upstream's React context scoping:
    // siblings of the provider are outside it.
    let provided = owner.with(|| {
        provide_direction_context(
            // The prop crosses the boundary as a reactive source — the tracked
            // read is upstream's `[direction]` deps array (`:17`). Static demo
            // props, so a constant read is their full reactive surface; a
            // dynamic caller (the live rtl→ltr transition demo) would pass a
            // real rg-0.2 signal — and the memo tracks it, which is the entire
            // mechanism behind the verified live transition.
            RwSignal::new(direction),
        )
    });

    // Build the children under the provider owner so a child's
    // `use_direction()` finds this provider's value (upstream: React context
    // scoping — the provider's `DirectionContext.Provider` wraps `children`,
    // :18-20).
    let view = owner.with(|| children());

    // Keep the owner (and thus the provided memo) alive for the life of the
    // mounted view: the page's providers are static (upstream's provider props
    // never change here either — the memo's deps array never re-fires), so the
    // value is immutable in practice. An rg-0.2 owner has no leptos-visible
    // disposal hook, so it is dropped only with the process (the docs app is
    // the process-long consumer, the prehydration_script.rs ArcRwSignal
    // app-long-lived contract).
    std::mem::forget(owner);

    view
}

/// A consumer reading the ambient reading direction through the real ported
/// `use_direction` hook (`packages/react/src/internals/direction-context/
/// DirectionContext.tsx:12-15`) — the shape every real consumer takes. Displays the
/// direction so the page's live examples are observable; under no provider it reads the
/// fallback `'ltr'`.
#[component]
pub fn DirectionProbe() -> impl IntoView {
    // The real hook: resolves through the reactive-graph-0.2 owner chain this
    // view was built under (see DirectionProviderView's owner-bridge note),
    // falling back to 'ltr' outside any provider.
    let direction = use_direction();
    view! {
        <div class="docs-direction-probe">
            <p>{move || format!("direction: {}", direction.get())}</p>
        </div>
    }
}

/// The live analog of the upstream hero demo's observable contract
/// (`demos/hero/tailwind/index.tsx:1-21`), built on the real ported machinery: an outer
/// `<div dir="rtl">` carrying the NATIVE attribute (the demo's illustration that
/// `<DirectionProvider>` does not affect HTML and CSS — `page.mdx:26`), a
/// [`DirectionProviderView`] with `direction="rtl"` publishing through the real
/// provider, and a [`DirectionProbe`] reading it back through the real hook.
///
/// The demo's Slider subtree is deferred to `library: slider` (a Phase B component not
/// yet ported — a fabricated stub would violate the loop's no-fabricated-demo rule); the
/// provider/consumer context flow the demo demonstrates is pinned live here instead.
#[component]
pub fn DirectionProviderRtlDemo() -> impl IntoView {
    view! {
        <div class="docs-direction-demo" dir="rtl">
            <DirectionProviderView direction=Some(TextDirection::Rtl)>
                <DirectionProbe />
            </DirectionProviderView>
        </div>
    }
}

/// The `docs/src/app/(docs)/react/utils/direction-provider/page.mdx` page.
#[component]
pub fn DirectionProviderPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Direction Provider"</h1>
            <p class="subtitle">"Enables RTL behavior for Base UI components."</p>

            <div class="docs-demo">
                <DirectionProviderRtlDemo />
            </div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and wrap it around your app:"</p>
            <pre><code>
"import { DirectionProvider } from '@base-ui/react/direction-provider';

// prettier-ignore
<DirectionProvider>
  {/* Your app or a group of components */}
</DirectionProvider>"
            </code></pre>
            <p>
                "`<DirectionProvider>` enables child Base UI components to adjust behavior based on RTL text "
                "direction, but does not affect HTML and CSS. The `dir=\"rtl\"` HTML attribute or "
                "`direction: rtl` CSS style must be set additionally by your own application code."
            </p>

            <h2>"API reference"</h2>

            <h3>"DirectionProvider"</h3>
            <p>
                "Enables RTL behavior for Base UI components. Props: `direction` (`TextDirection`, "
                "default `'ltr'` — \"The reading direction of the text\") and `children` "
                "(`React.ReactNode`). Additional types: `type TextDirection = 'ltr' | 'rtl'`."
            </p>

            <h3>"useDirection"</h3>
            <p>
                "Use this hook to read the current text direction. This is useful for wrapping portaled "
                "components that may be rendered outside your application root and are unaffected by the "
                "`dir` attribute set within. Return value: `TextDirection`."
            </p>

            <h2>"Live: the ported provider and hook"</h2>
            <p>
                "The example below runs through the real ported implementation "
                "(`leptos_ui_internals::direction_provider::provide_direction_context` and "
                "`direction_context::use_direction`): the provider publishes its configured direction "
                "through the real memoized context (an omitted direction resolves to the writer-side "
                "'ltr' default, and the memo tracks the prop so a dynamic caller gets the live "
                "rtl → ltr transition), and the probe reads it back through the real hook. The outer "
                "div carries the native `dir=\"rtl\"` attribute separately — illustrating the docs "
                "caveat that DirectionProvider does not affect HTML and CSS. The upstream hero demo's "
                "Slider subtree lands with library: slider (a Phase B component not yet ported)."
            </p>
            <div class="docs-demo">
                <DirectionProviderRtlDemo />
                <DirectionProbe />
            </div>
        </article>
    }
}
