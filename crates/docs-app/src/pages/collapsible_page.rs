//! The docs page for `Collapsible`, mirroring
//! `docs/src/app/(docs)/react/components/collapsible/page.mdx`
//! (`specs/docs-content/collapsible/page.md`).
//!
//! Page structure per the spec's "Page structure (headings, in order)" section: `# Collapsible`
//! h1, the `<Subtitle>` ("A collapsible panel controlled by a button."), the hero demo before the
//! first heading, `## Anatomy` with its fenced snippet, `## Examples` over "Hidden until found",
//! and `## API reference` over the three generated `TypesCollapsible` tables (Root, Trigger,
//! Panel — echoed as the three headings the mirrored page carries, the accordion/meter page
//! precedent: the port has no docs generator, so the tables' documented props are not fabricated
//! as executable machinery).
//!
//! Page furniture mirrored in module docs (the checkbox/meter page precedent): the
//! `<Meta name="description">` content — "A high-quality, unstyled React collapsible component
//! that displays a panel controlled by a button." (`page.mdx:4-7`) — and the trailing
//! `export const metadata` SEO keywords block (11 keywords, `page.mdx:60-74`: 'React Collapsible',
//! 'Collapsible Component', 'Expandable Panel', 'Toggle Panel Button', 'Disclosure Widget',
//! 'Show/Hide Content', 'Accordion Item', 'Accessible Disclosure', 'Headless React Components',
//! 'Collapsible Trigger Panel', 'Base UI').
//!
//! ## The snippets, TRANSLATED to this port's API
//!
//! `specs/docs-content/CONTRACT.md` requirement 1: every code block on a mirrored page is
//! expressed against THIS port — `leptos_ui` parts in `view!` markup, never upstream's
//! `@base-ui/react` source. Before this iteration both blocks on this page were upstream's JSX
//! verbatim (the probe read `snippets {total: 2, leptos: 0, react: 2}`), i.e. the live page taught
//! the wrong framework while the structural checks passed. The spelling is the CONTRACT's own
//! mapping (`<Collapsible.Root>` -> `<Collapsible::Root>`, the three parts of
//! `crates/leptos-ui/src/collapsible/mod.rs:104,111,117`); the page moved out of `lib.rs` in the
//! same iteration so its snippets live beside the guard that proves them, like every other page.
//!
//! ## Documented adaptation: `hiddenUntilFound` (never silent)
//!
//! Upstream's "Searchable hidden panel" example teaches `hiddenUntilFound` on
//! `Collapsible.Panel` — the closed panel stays mounted with `hidden="until-found"`, so
//! find-in-page can search it and a native `beforematch` event opens it
//! (`specs/library/collapsible/behavior.md`, "State model" and "Events"; cited by the page spec at
//! `specs/docs-content/collapsible/page.md:28`). The port's `CollapsiblePanel` accepts
//! `keep_mounted` only (`crates/leptos-ui/src/collapsible/mod.rs:57-62`) and hides the panel by
//! class, not by the `hidden` attribute — so the example shows the port's real prop and the prose
//! states the difference rather than repeating upstream's rationale for behaviour this port does
//! not reproduce (requirement 3: "if an obligation cannot be proved by an observable in this port
//! yet, say so explicitly … do not write a weaker claim to make the row look filled"). The gap is
//! logged in `ralph/logs/spec-discrepancies.md` and is a `library:`-side surface question, not a
//! copy edit.

use crate::code_block::{Lang, code_block};
use leptos::prelude::*;
use leptos_ui::{CollapsiblePanel, CollapsibleRoot, CollapsibleTrigger};

/// The `docs/src/app/(docs)/react/components/collapsible/demos/hero/tailwind/index.tsx`
/// demo, ported to Leptos on the real `leptos_ui` Collapsible component.
/// The upstream demo passes Tailwind classes as component props; the ported
/// components do not yet forward arbitrary props, so the same classes ride on
/// wrapper elements with identical layout/visual semantics.
#[component]
pub fn CollapsibleHeroDemo() -> impl IntoView {
    view! {
        <div class="flex min-h-36 w-48 flex-col justify-center text-neutral-950 dark:text-white">
            <CollapsibleRoot>
                <CollapsibleTrigger>
                    <span class="flex h-8 w-full items-center justify-between gap-2 rounded-none border border-neutral-950 bg-white pl-3 pr-2 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none">
                        "Recovery keys"
                        <svg
                            class="transition-transform duration-100"
                            width="16"
                            height="16"
                            viewBox="0 0 16 16"
                            fill="currentColor"
                            style="display: block"
                        >
                            <path d="M6 12V4l4.5 4z" />
                        </svg>
                    </span>
                </CollapsibleTrigger>
                <CollapsiblePanel>
                    <div class="flex h-9 flex-col justify-end overflow-hidden text-sm">
                        <div class="flex flex-col gap-2 px-3.5 py-2">
                            <div>"alien-bean-pasta"</div>
                            <div>"wild-irish-burrito"</div>
                            <div>"horse-battery-staple"</div>
                        </div>
                    </div>
                </CollapsiblePanel>
            </CollapsibleRoot>
        </div>
    }
}

/// The `## Anatomy` snippet (`page.mdx:17-24`), translated: upstream's
/// `<Collapsible.Root>` / `<Collapsible.Trigger />` / `<Collapsible.Panel />` tree in the port's
/// namespaced spelling. The port's parts take their content as `children` (a required prop on
/// `Trigger`/`Panel`, unlike upstream where children are optional), so the anatomy carries the
/// minimum each part needs instead of being self-closing.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::Collapsible;

view! {
    <Collapsible::Root>
        <Collapsible::Trigger>"Toggle"</Collapsible::Trigger>
        <Collapsible::Panel>"Panel content"</Collapsible::Panel>
    </Collapsible::Root>
}"#;

/// The "Searchable hidden panel" snippet (`page.mdx:34-40`), translated to the port's real prop
/// (`keep_mounted`); upstream's `hiddenUntilFound` is not on the port's `CollapsiblePanel` — see
/// the module docs' "Documented adaptation" section.
const HIDDEN_UNTIL_FOUND_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::Collapsible;

view! {
    <Collapsible::Root>
        <Collapsible::Trigger>"Shipping details"</Collapsible::Trigger>
        // @highlight-text "keep_mounted"
        <Collapsible::Panel keep_mounted=true>
            "Standard shipping takes 3–5 business days."
        </Collapsible::Panel>
    </Collapsible::Root>
}"#;

/// The `docs/src/app/(docs)/react/components/collapsible/page.mdx` page.
#[component]
pub fn CollapsiblePage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Collapsible"</h1>
            <p class="subtitle">"A collapsible panel controlled by a button."</p>

            <div class="docs-demo" data-demo="hero"><CollapsibleHeroDemo /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Examples"</h2>
            <h3>"Hidden until found"</h3>
            <p>
                "The closed panel can be kept mounted with the `keep_mounted` prop, so its "
                "contents stay in the DOM and remain indexable by search engines. Upstream's "
                "`hiddenUntilFound` prop — which hides the closed panel with "
                "`hidden=\"until-found\"` so find-in-page can search it and reveal it on a match — "
                "is not exposed by this port's `Collapsible.Panel` yet; the panel is hidden by "
                "class. The ported example below shows the prop the port does have."
            </p>
            {code_block(Lang::Rust, "Searchable hidden panel", HIDDEN_UNTIL_FOUND_SNIPPET)}
            <p>
                <a href="/react/components/accordion#hidden-until-found">
                    "See the Accordion page's hidden-until-found example for the interactive version."
                </a>
            </p>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            <h3>"Trigger"</h3>
            <h3>"Panel"</h3>
        </article>
    }
}

// ---------------------------------------------------------------------------
// The page's snippet guard (`specs/docs-content/CONTRACT.md` requirement 1)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};
    use leptos_ui::Collapsible;

    /// Upstream's Anatomy block (`page.mdx:17-24`), kept as the classifier's positive control so
    /// the assertions below cannot pass vacuously. The `@base-ui/react/collapsible` import line is
    /// deliberately NOT carried: this is page-source, not reader-facing, and
    /// `check-react-mentions.mjs --source` counts a bare package string wherever it appears (the
    /// `field_page.rs` precedent).
    const UPSTREAM_ANATOMY_SHAPE: &str = "<Collapsible.Root>\n  <Collapsible.Trigger />\n  <Collapsible.Panel />\n</Collapsible.Root>;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert_eq!(
            classify(UPSTREAM_ANATOMY_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertions below \
             would be vacuous"
        );
    }

    #[test]
    fn every_snippet_on_this_page_teaches_the_port() {
        let snippets = [
            ("Anatomy", ANATOMY_SNIPPET),
            ("Searchable hidden panel", HIDDEN_UNTIL_FOUND_SNIPPET),
        ];
        let mut counts = (0, 0, 0);
        for (name, text) in snippets {
            match classify(text) {
                SnippetLanguage::Leptos => counts.0 += 1,
                SnippetLanguage::React => {
                    counts.1 += 1;
                    panic!("the '{name}' snippet still carries upstream's React source");
                }
                SnippetLanguage::Other => {
                    counts.2 += 1;
                    panic!("the '{name}' snippet identifies as neither the port nor upstream");
                }
            }
        }
        assert_eq!(
            counts,
            (2, 0, 0),
            "the probe must read {{total: 2, leptos: 2, react: 0}} for this page"
        );
    }

    /// The snippets' compositions, verbatim, compiled: the compiler checks the part spellings and
    /// the `keep_mounted` prop this page teaches. Never called.
    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        view! {
            <Collapsible::Root>
                <Collapsible::Trigger>"Toggle"</Collapsible::Trigger>
                <Collapsible::Panel>"Panel content"</Collapsible::Panel>
            </Collapsible::Root>
        }
    }

    #[allow(dead_code)]
    fn hidden_until_found_snippet_shape() -> impl IntoView {
        view! {
            <Collapsible::Root>
                <Collapsible::Trigger>"Shipping details"</Collapsible::Trigger>
                <Collapsible::Panel keep_mounted=true>
                    "Standard shipping takes 3–5 business days."
                </Collapsible::Panel>
            </Collapsible::Root>
        }
    }

    #[test]
    fn every_snippet_compiles_against_the_ports_surface() {
        let _ = (anatomy_snippet_shape, hidden_until_found_snippet_shape);
    }

    /// The teaching spelling is the namespaced one (`<Collapsible::Root>` — `CONTRACT.md`
    /// requirement 1's mapping table), and no flattened `*_view(..)` helper may appear: both
    /// blocks were upstream's JSX before this iteration, and a helper call would be just as far
    /// from the table's Rust side.
    #[test]
    fn the_snippets_teach_the_namespaced_parts() {
        for (name, text) in [
            ("Anatomy", ANATOMY_SNIPPET),
            ("Searchable hidden panel", HIDDEN_UNTIL_FOUND_SNIPPET),
        ] {
            assert!(
                text.contains("<Collapsible::Root>"),
                "the '{name}' snippet does not teach <Collapsible::Root>"
            );
            assert!(
                !text.contains("_view("),
                "the '{name}' snippet uses a flattened *_view helper"
            );
        }
    }
}
