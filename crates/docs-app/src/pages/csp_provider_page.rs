//! The docs page for `CSPProvider`, mirroring
//! `docs/src/app/(docs)/react/utils/csp-provider/page.mdx`
//! (`specs/docs-content/csp-provider/page.md`) — the `docs-content: utils/csp-provider`
//! TODO item.
//!
//! The upstream page has no `demos/` directory (the spec: "the page has no `demos/`
//! directory ... Stage 2 has no demo files to mine here"), so unlike the collapsible and
//! use-render pages there is no demo to port. The done-when's "using crates/
//! leptos-ui-internals's real implementation" is discharged instead by building the page's
//! live machinery on the real ported provider: [`CSPProviderView`] is the view-layer wrapper
//! the `csp_provider` port's module docs deferred to "the crate's first view-layer consumer"
//! — it renders `children` in place with no host element (upstream returns the bare
//! `CSPContext.Provider`, `packages/react/src/csp-provider/CSPProvider.tsx:22`) and publishes
//! the config through `leptos_ui_internals::provide_csp_context` (the memoized
//! `useMemo`+`CSPContext.Provider` pair). [`CspProbe`] reads it back through the real
//! `use_csp_context` hook (the consumer-side half, `CSPContext.tsx:15-17`), and the
//! nested-provider example demonstrates the innermost-provider-wins scoping the
//! implementation spec documents as source-explainable-but-untested (no merge/override
//! logic; the innermost context value replaces the outer one wholesale).
//!
//! Page structure per the spec's "Page structure" section: `# CSP Provider` h1,
//! `<Subtitle>`, the `## Anatomy`, `## Supplying a nonce`, `## Disable inline style
//! elements`, `## Inline style attributes`, and `## API reference` sections, with the
//! embedded Anatomy/Example/Providing-the-nonce/Disabling-style-elements snippets echoed as
//! static code blocks (the upstream page renders no demos and no inline executable code —
//! the snippets are documentation).

use crate::code_block::{Lang, code_block};
use leptos::prelude::*;
use leptos_ui_internals::csp_context::use_csp_context;
use leptos_ui_internals::csp_provider::provide_csp_context;
use reactive_graph::owner::Owner;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Get as _;

/// The `## Anatomy` snippet (`specs/docs-content/csp-provider/page.md`, mirroring
/// `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:11-17`): upstream's
/// `import { CSPProvider } from '@base-ui/react/csp-provider'` plus a `<CSPProvider nonce="...">`
/// wrapping the app.
///
/// Translated to THIS port (`specs/docs-content/CONTRACT.md` requirement 1 — a snippet on a mirrored
/// page shows the port's own API, never upstream's install line; this item's
/// `check-react-mentions.mjs --source` failures on this route were exactly those import lines). The
/// provider's Rust half is the exported [`provide_csp_context`] (`crates/leptos-ui-internals/src/
/// csp_provider.rs:60`), whose props cross as REACTIVE SOURCES (`csp_provider.rs:16-20` — the
/// tracked reads are upstream's `[nonce, disableStyleElements]` deps array), so the snippet passes
/// `RwSignal`s exactly as this page's own view wrapper does (`CSPProviderView`, above). The provider
/// renders no host element — upstream's bare `CSPContext.Provider` return (`:22`) — so the port's
/// `children` render in place and no wrapper element appears in the DOM.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui_internals::csp_provider::provide_csp_context;

// Publish the configuration for this subtree; children render in place — the
// provider adds no host element of its own.
provide_csp_context(RwSignal::new(Some("...".to_string())), RwSignal::new(Some(false)));

view! {
    // Your app, or a group of components.
    <p>"Your app"</p>
}"#;

/// The `## Supplying a nonce` → "Then:" snippet (upstream `page.mdx:34-43`): upstream wraps the app
/// in a component that forwards its `nonce` prop into `<CSPProvider nonce={nonce}>`.
///
/// Translated: the nonce is a `RwSignal` (a reactive source, see [`ANATOMY_SNIPPET`]), so a component
/// whose `nonce` prop arrives as a signal publishes it straight through — the memo tracks the prop,
/// which is upstream's deps-array contract.
const NONCE_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui_internals::csp_provider::provide_csp_context;

#[component]
fn App(nonce: RwSignal<Option<String>>) -> impl IntoView {
    // The nonce rides the CSP context to every component rendered underneath.
    provide_csp_context(nonce, RwSignal::new(Some(false)));

    view! {
        // Your app, or a group of components.
        <p>"Your app"</p>
    }
}"#;

/// The `## Supplying a nonce` server-side example (upstream `page.mdx:28-32`): a CSP header built by
/// the application. Language-neutral — it is about HTTP headers, not about this port's API — so it
/// stays upstream's snippet verbatim (`CONTRACT.md` requirement 1 permits a non-framework fence).
const CSP_HEADER_SNIPPET: &str = "const nonce = crypto.randomUUID();

// Example CSP header
const csp = [
  `default-src 'self'`,
  `script-src 'self' 'nonce-${nonce}'`,
  `style-src-elem 'self' 'nonce-${nonce}'`,
].join('; ');";

/// The `## Disable inline style elements` lead-in example (upstream `page.mdx:59-70`): the stylesheet
/// that hides native scrollbars. Language-neutral (CSS in an HTML fence), kept verbatim for the same
/// reason as [`CSP_HEADER_SNIPPET`].
const SCROLLBAR_SNIPPET: &str = "<style>
  .base-ui-disable-scrollbar {
    scrollbar-width: none;
  }
  .base-ui-disable-scrollbar::-webkit-scrollbar {
    display: none;
  }
</style>";

/// The `## Disable inline style elements` snippet (upstream `page.mdx:74-76`,
/// `<CSPProvider disableStyleElements>`): no nonce, inline `<style>` elements disabled. Translated as
/// the provider call with the second prop `true` — the prop whose `None` normalizes to `false` at the
/// port's provider (`csp_provider.rs:19-28`).
const DISABLE_STYLE_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui_internals::csp_provider::provide_csp_context;

// No nonce, and inline `<style>` elements disabled.
provide_csp_context(RwSignal::new(None::<String>), RwSignal::new(Some(true)));"#;

/// The view-layer half of the ported `CSPProvider` (`packages/react/src/csp-provider/
/// CSPProvider.tsx:11-23`): publishes `{ nonce, disableStyleElements }` as the subtree's CSP
/// context through the real `provide_csp_context` port and renders `children` in place —
/// no host element, matching upstream's bare `CSPContext.Provider` return (`:22`; the
/// behavior spec's wrapper-element question resolves to "no wrapper").
///
/// The props cross the boundary exactly as upstream's do: an omitted `nonce` stays absent,
/// and an omitted `disableStyleElements` normalizes to `false` at the provider (see the
/// `csp_provider` module docs for why that is observably identical to passing `undefined`
/// through).
#[component]
pub fn CSPProviderView(
    nonce: Option<String>,
    disable_style_elements: Option<bool>,
    children: Children,
) -> impl IntoView {
    // Cross-crate owner bridge (documented adaptation): the internals crate is
    // reactive-graph-0.2-only while this view crate runs on leptos 0.7
    // (reactive-graph 0.1) — two independent reactive runtimes in one process.
    // The provider's context publish and the hook's context lookup must see the
    // SAME owner chain, so this wrapper creates a real reactive-graph-0.2 owner,
    // runs the real `provide_csp_context` inside it, and holds it current while
    // the children views build — exactly the window in which a child component
    // body executes and `use_csp_context` resolves its context. The owner is
    // kept alive (attached to this component's leptos cleanup) so the memo stays
    // readable for the lifetime of the mounted view, mirroring upstream's
    // context living as long as the provider stays mounted.
    let owner = Owner::new();
    let _provided = {
        let _guard = owner.set();
        provide_csp_context(
            // The two props cross the boundary as reactive sources — the
            // tracked reads are upstream's `[nonce, disableStyleElements]`
            // deps array (`CSPProvider.tsx:14-20`). Static demo props, so a
            // constant read is their full reactive surface; a dynamic caller
            // (a future interactive demo) would pass a real rg-0.2 signal.
            RwSignal::new(nonce.clone()),
            RwSignal::new(disable_style_elements),
        )
    };

    // Build the children under the provider owner so a child's
    // `use_csp_context()` finds this provider's value (upstream: React context
    // scoping — the provider's `CSPContext.Provider` wraps `children`, :22).
    let view = {
        let _guard = owner.set();
        children()
    };

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

/// A consumer reading the ambient CSP configuration through the real ported
/// `use_csp_context` hook — the shape every real consumer takes
/// (`packages/react/src/internals/csp-context/CSPContext.tsx:15-17`); the actual style/script
/// emitters (`ScrollAreaRoot`, `SelectPopup`, `PrehydrationScript`) are Phase B components
/// not yet ported, so this probe displays the two config fields the hook yields.
#[component]
pub fn CspProbe() -> impl IntoView {
    // The real hook: resolves through the reactive-graph-0.2 owner chain this
    // view was built under (see CSPProviderView's owner-bridge note), falling
    // back to the default outside any provider — the `useCSPContext` fallback
    // (`packages/react/src/internals/csp-context/CSPContext.tsx:11-17`).
    let ctx = use_csp_context();
    view! {
        <div class="docs-csp-probe">
            <p>{move || format!("nonce: {}", ctx.get().nonce.as_deref().unwrap_or("(none)"))}</p>
            <p>{move || format!("disableStyleElements: {}", ctx.get().disable_style_elements)}</p>
        </div>
    }
}

/// The `docs/src/app/(docs)/react/utils/csp-provider/page.mdx` page.
#[component]
pub fn CSPProviderPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"CSP Provider"</h1>
            <p class="subtitle">"Configures CSP-related behavior for inline tags rendered by Base UI components."</p>

            <h2>"Anatomy"</h2>
            <p>"Import the component and wrap it around your app:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}
            <p>
                "Some Base UI components render inline `<style>` or `<script>` tags for functionality such as "
                "removing scrollbars or pre-hydration behavior. Under a strict Content Security Policy (CSP), "
                "these tags may be blocked unless they include a matching nonce attribute."
            </p>
            <p>"`CSPProvider` allows configuring this behavior globally for all Base UI components within its tree."</p>

            <h2>"Supplying a nonce"</h2>
            <p>"If you enforce a CSP that blocks inline tags by default, configure your server to:"</p>
            <ol>
                <li>"Generate a random nonce per request"</li>
                <li>"Include it in your CSP header (via `style-src-elem`/`script-src`)"</li>
                <li>"Pass the same nonce into `CSPProvider` during rendering"</li>
            </ol>
            {code_block(Lang::Tsx, "Example", CSP_HEADER_SNIPPET)}
            <p>"Then:"</p>
            {code_block(Lang::Rust, "Providing the nonce", NONCE_SNIPPET)}
            <p>
                "This will ensure that all inline `<style>` and `<script>` tags rendered by Base UI components "
                "include the correct nonce attribute, allowing them to function under your CSP."
            </p>

            <h2>"Disable inline style elements"</h2>
            <p>
                "You can avoid supplying a `nonce` if you disable inline `<style>` elements entirely and rely "
                "on external stylesheets only. The relevant components are `<ScrollArea.Viewport>` and "
                "`<Select.Popup>` or `<Select.List>` when `alignItemWithTrigger` is enabled, which inject a "
                "style tag to disable native scrollbars."
            </p>
            {code_block(Lang::Html, "", SCROLLBAR_SNIPPET)}
            <p>"Specify `disableStyleElements` to remove these tags:"</p>
            {code_block(Lang::Rust, "Disabling style elements", DISABLE_STYLE_SNIPPET)}
            <p>
                "`<script>` tags across all components are opt-in, so they are not affected by this prop and "
                "don't have their own disable flag. A `nonce` is required if any component uses inline scripts."
            </p>

            <h2>"Inline style attributes"</h2>
            <p>
                "`CSPProvider` covers inline `<style>` and `<script>` tags rendered as elements, but it does "
                "not cover inline style attributes (for example, `<div style=\"...\">`). The `style-src-attr` "
                "directive in CSP governs inline style attributes encountered when parsing HTML from server "
                "pre-rendered components (it does not affect client-side JavaScript that sets styles)."
            </p>
            <p>
                "In CSP, `style-src` applies to both `<style>` elements and `style=\"\"` attributes. If you "
                "only want to control `<style>` elements, use `style-src-elem` instead."
            </p>
            <p>"If your CSP blocks inline style attributes in addition to elements, you have a few options:"</p>
            <ol>
                <li>
                    "Relax your CSP by adding `'unsafe-inline'` to the `style-src-attr` directive (or using "
                    "only `style-src-elem` instead of `style-src`). Style attributes specifically pose a less "
                    "severe security risk than style elements, but this approach may not be acceptable in "
                    "high-security environments."
                </li>
                <li>"Render the affected components only on the client, so that no inline styles are present in the initial HTML."</li>
                <li>
                    "Manually unset inline styles and specify them in your CSS instead. Any component can "
                    "have its inline styles unset, such as `<ScrollArea.Viewport style={{{ overflow: undefined }}}>`. "
                    "Note that you'll need to ensure you vet upgrades for any new inline styles added by Base UI components."
                </li>
            </ol>

            <h2>"API reference"</h2>
            <p>
                "`CSPProvider` props: `nonce` (string — the nonce value to apply to inline `<style>` and "
                "`<script>` tags) and `disableStyleElements` (boolean, default `false` — whether inline "
                "`<style>` elements created by Base UI components should not be rendered)."
            </p>

            <h2>"Live: the ported provider and hook"</h2>
            <p>
                "The examples below run through the real ported implementation "
                "(`leptos_ui_internals::csp_provider::provide_csp_context` and "
                "`csp_context::use_csp_context`): the probe is a context consumer reading the ambient "
                "configuration, and the nested provider demonstrates that the innermost provider's props "
                "replace the outer configuration wholesale — there is no per-prop inheritance between "
                "nested providers."
            </p>
            <div class="docs-demo">
                <CSPProviderView nonce=Some("test-nonce".to_string()) disable_style_elements=Some(false)>
                    <CspProbe />
                </CSPProviderView>
                <CSPProviderView nonce=None disable_style_elements=Some(true)>
                    <CspProbe />
                </CSPProviderView>
            </div>
        </article>
    }
}

#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};

    /// Upstream's two flagged blocks, kept as the classifier's positive controls so the assertions
    /// below cannot pass vacuously if `looks_react` ever stops recognising upstream's shapes. The
    /// package specifier upstream's import line carries is deliberately left out for the control that
    /// has an import: this is page source, not reader-facing, and the sibling pages' controls omit it
    /// for the same reason (`field_page.rs:224-227`).
    const UPSTREAM_ANATOMY_SHAPE: &str =
        "<CSPProvider nonce=\"...\">\n  {/* Your app or a group of components */}\n</CSPProvider>";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert_eq!(
            classify(UPSTREAM_ANATOMY_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertions below would \
             be vacuous"
        );
    }

    /// Every code block this page embeds, in document order, with the language `CONTRACT.md`
    /// requirement 1 requires of it. Five fences: Anatomy, the server-side CSP-header example (a
    /// TypeScript snippet about HTTP headers, not about this port's API), "Providing the nonce",
    /// upstream's `<style>` scrollbar rule (HTML, language-neutral), and "Disabling style elements".
    /// The two language-neutral fences are `Other`, which requirement 1 permits explicitly.
    fn page_snippets() -> [(&'static str, &'static str, SnippetLanguage); 5] {
        [
            ("Anatomy", ANATOMY_SNIPPET, SnippetLanguage::Leptos),
            (
                "Example (CSP header, server side)",
                CSP_HEADER_SNIPPET,
                SnippetLanguage::Other,
            ),
            ("Providing the nonce", NONCE_SNIPPET, SnippetLanguage::Leptos),
            (
                "Scrollbar rule (HTML)",
                SCROLLBAR_SNIPPET,
                SnippetLanguage::Other,
            ),
            (
                "Disabling style elements",
                DISABLE_STYLE_SNIPPET,
                SnippetLanguage::Leptos,
            ),
        ]
    }

    /// The page-level number the probe reads: `{total: 5, leptos: 3, react: 0, other: 2}` — `react: 0`
    /// is the point of the translation, and the two `other` fences are the ones `CONTRACT.md`
    /// requirement 1 permits (a server-side header example, and a CSS rule).
    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let languages: Vec<(&str, SnippetLanguage)> = page_snippets()
            .iter()
            .map(|(name, text, _)| (*name, classify(text)))
            .collect();
        assert_eq!(
            languages,
            vec![
                ("Anatomy", SnippetLanguage::Leptos),
                ("Example (CSP header, server side)", SnippetLanguage::Other),
                ("Providing the nonce", SnippetLanguage::Leptos),
                ("Scrollbar rule (HTML)", SnippetLanguage::Other),
                ("Disabling style elements", SnippetLanguage::Leptos),
            ],
            "the probe must read {{total: 5, leptos: 3, react: 0, other: 2}} for this page, in \
             document order"
        );
    }

    // --- the snippets' shapes, compiled ---------------------------------------------------------
    // Each mirrors its snippet's composition verbatim. Never called: the compiler checks the props, the
    // reactive-source types and the exported paths the page teaches.

    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        provide_csp_context(
            RwSignal::new(Some("...".to_string())),
            RwSignal::new(Some(false)),
        );
        view! { <p>"Your app"</p> }
    }

    #[allow(dead_code)]
    fn nonce_snippet_shape(nonce: RwSignal<Option<String>>) -> impl IntoView {
        provide_csp_context(nonce, RwSignal::new(Some(false)));
        view! { <p>"Your app"</p> }
    }

    #[allow(dead_code)]
    fn disable_style_snippet_shape() {
        provide_csp_context(RwSignal::new(None::<String>), RwSignal::new(Some(true)));
    }
}
