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

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::owner::Owner;
use reactive_graph::traits::Get as _;
use leptos_ui_internals::csp_context::use_csp_context;
use leptos_ui_internals::csp_provider::provide_csp_context;

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
            <pre><code>
"import { CSPProvider } from '@base-ui/react/csp-provider';

// prettier-ignore
<CSPProvider nonce=\"...\">
  {/* Your app or a group of components */}
</CSPProvider>"
            </code></pre>
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
            <pre><code>
"const nonce = crypto.randomUUID();

// Example CSP header
const csp = [
  `default-src 'self'`,
  `script-src 'self' 'nonce-${nonce}'`,
  `style-src-elem 'self' 'nonce-${nonce}'`,
].join('; ');"
            </code></pre>
            <p>"Then:"</p>
            <pre><code>
"import { CSPProvider } from '@base-ui/react/csp-provider';

function App({ nonce }) {
  return <CSPProvider nonce={nonce}>{/* ... */}</CSPProvider>;
}"
            </code></pre>
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
            <pre><code>
"<style>
  .base-ui-disable-scrollbar {
    scrollbar-width: none;
  }
  .base-ui-disable-scrollbar::-webkit-scrollbar {
    display: none;
  }
</style>"
            </code></pre>
            <p>"Specify `disableStyleElements` to remove these tags:"</p>
            <pre><code>"<CSPProvider disableStyleElements>{/* ... */}</CSPProvider>"</code></pre>
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
