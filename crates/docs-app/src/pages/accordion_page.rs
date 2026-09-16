//! The docs page for `Accordion`, mirroring
//! `docs/src/app/(docs)/react/components/accordion/page.mdx`
//! (`specs/docs-content/accordion/page.md`) — the `docs-content: components/accordion (prose +
//! snippet completion)` TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Accordion` h1,
//! `<Subtitle>` ("A set of collapsible panels with headings."), the hero demo before
//! the first heading, `## Anatomy` with the single fenced snippet, `## Examples` over
//! "Open multiple panels" (`./demos/multiple`) and "Hidden until found"
//! (`./demos/hidden-until-found`), and `## API reference` over the five generated
//! `TypesAccordion` reference blocks.
//!
//! The `## API reference` section is the whole of upstream's `<TypesAccordion.Root />` …
//! `<TypesAccordion.Panel />` (`page.mdx:52-74`): each part's summary line, its generated props
//! table (one `<details>` row per prop, `Name`/`Description`/`Type`/`Default`), its data-attributes
//! table (`Accordion.Panel` also has the CSS-variables table), and the `Additional Types` panels the
//! part carries (`Accordion.Root.Props`, `Accordion.Root.State`, …). The generated content lives in
//! `crate::pages::accordion_reference` (transcribed from
//! `docs/src/app/(docs)/react/components/accordion/types.md` and upstream's live render, with its
//! provenance recorded there) and is rendered through the ported reference primitives
//! (`crate::reference`) — never fabricated as executable machinery.
//!
//! The `### <Part>` headings carry upstream's own ids (`id="root"`, `id="item"`, …) so the generated
//! `Re-Export of …` lines' `#root`/`#item` links resolve, exactly as they do upstream.
//!
//! Page furniture mirrored in module docs (the separator page precedent): the
//! `<Meta name="description">` content — "A high-quality, unstyled React accordion
//! component that displays a set of collapsible panels with headings." — and the
//! trailing `export const metadata` SEO keywords block (10 keywords: 'React
//! Accordion', 'Accordion Component', 'Collapsible Content', 'Expandable Panel',
//! 'Accessible Accordion', 'Multi-Panel UI', 'Disclosure Widget', 'FAQ Component',
//! 'Headless React Components', 'Base UI').
//!
//! The three live demos are the upstream Tailwind demos
//! (`docs/src/app/(docs)/react/components/accordion/demos/{hero,multiple,hidden-until-found}/tailwind/index.tsx`,
//! the three `specs/docs-content/accordion/demos.json` entries) ported onto the REAL
//! `leptos_ui::accordion` parts — `AccordionRoot`/`AccordionItem`/`AccordionHeader`/
//! `AccordionTrigger`/`AccordionPanel` — with every upstream `className` string
//! carried verbatim so the DOM the Leptos port produces matches the React demo's
//! element-for-element (the parts' `class` props take the demo classes directly; no
//! wrapper-div workaround, unlike the collapsible page whose components predated
//! class forwarding). All three demos are uncontrolled (demos.json:
//! `stateManaged: "uncontrolled"` — "no value props; Accordion.Root manages state
//! internally with all panels initially closed"): the real port's `handleValueChange`
//! algebra + cancel protocol own the open state, exactly the machinery
//! `specs/library/accordion/behavior.md` documents. The hero exercises the default
//! single-open toggle; the multiple demo adds `multiple` so panels open
//! independently (upstream `index.tsx:6-9`); the hidden-until-found demo adds
//! `hiddenUntilFound` on Root (`index.tsx:6-9`) so closed panels stay mounted with
//! `hidden="until-found"` — the port's panel mount/unmount gate + hidden-attribute
//! walk (`AccordionPanel` in `crates/leptos-ui/src/accordion/mod.rs`). The rotating
//! plus icon rides the trigger's real `data-panel-open` attribute through the
//! upstream `group-data-panel-open:rotate-45` Tailwind variant class.
//!
//! Snippet language (`specs/docs-content/CONTRACT.md` requirement 1; the spec carries the contract
//! table): the Anatomy block teaches the PORT's composition — `leptos_ui`'s parts in `view!` syntax,
//! the same tree upstream teaches with `::`-style module paths — instead of upstream's JSX, which is
//! what it carried until this item.

use crate::code_block::{Lang, code_block};
use crate::pages::accordion_reference::{
    HEADER_ADDITIONAL_TYPES, HEADER_DATA_ATTRIBUTES, HEADER_PROPS, HEADER_SUMMARY,
    ITEM_ADDITIONAL_TYPES, ITEM_DATA_ATTRIBUTES, ITEM_PROPS, ITEM_SUMMARY, PANEL_ADDITIONAL_TYPES,
    PANEL_CSS_VARIABLES, PANEL_DATA_ATTRIBUTES, PANEL_PROPS, PANEL_SUMMARY, ROOT_ADDITIONAL_TYPES,
    ROOT_DATA_ATTRIBUTES, ROOT_PROPS, ROOT_SUMMARY, TRIGGER_ADDITIONAL_TYPES,
    TRIGGER_DATA_ATTRIBUTES, TRIGGER_PROPS, TRIGGER_SUMMARY,
};
use crate::reference::{self, AdditionalType, DataAttributeRow, ReferenceProp, Segment};
use leptos::prelude::*;

use leptos_ui::{AccordionHeader, AccordionItem, AccordionPanel, AccordionRoot, AccordionTrigger};

/// The Anatomy snippet (`page.mdx:17-28`), translated to the port.
///
/// Upstream's block imports `{ Accordion }` from `@base-ui/react/accordion` and assembles
/// `Accordion.Root > Accordion.Item > Accordion.Header > Accordion.Trigger` with `Accordion.Panel`
/// as the Header's sibling inside the Item. The port's parts are `#[component]` functions in
/// `leptos_ui`, so the same four-part tree is the same four elements in `view!` markup with the
/// crate's own names — `specs/docs-content/CONTRACT.md` requirement 1's "same names, same hierarchy":
/// upstream's `Accordion.Root` is this port's `AccordionRoot`, and so on down the tree. The shape is
/// compiled by `snippet_language_guard::anatomy_snippet_shape` below, so the snippet cannot name a
/// part, prop or path the port does not have.
///
/// One difference in the leaves, forced by the port's types and therefore shown rather than hidden:
/// upstream's `<Accordion.Trigger />` and `<Accordion.Panel />` are self-closing (React's `children`
/// is optional), while this port's `AccordionTrigger`/`AccordionPanel` take a required `children`
/// prop, so the example passes their content inline. Upstream's own demo does the same — its
/// triggers and panels carry the question and the answer.
///
/// (The namespaced `Accordion::Root` spelling the contract's mapping table describes is
/// `library: namespaced part surface (ported batch)`'s surface, which does not exist yet; this is the
/// port's current public API, per the snippet-translation queue's own instruction to translate to it
/// now rather than wait.)
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{AccordionHeader, AccordionItem, AccordionPanel, AccordionRoot, AccordionTrigger};

view! {
    <AccordionRoot>
        <AccordionItem>
            <AccordionHeader>
                <AccordionTrigger>"Trigger"</AccordionTrigger>
            </AccordionHeader>
            <AccordionPanel>"Panel"</AccordionPanel>
        </AccordionItem>
    </AccordionRoot>
}"#;

/// The upstream demo root `className`
/// (`hero/tailwind/index.tsx:4`, shared verbatim by all three demos).
const DEMO_ROOT_CLASS: &str = "flex w-full max-w-80 flex-col border border-neutral-950 text-neutral-950 dark:border-white dark:text-white";

/// The upstream non-first Item `className`
/// (`hero/tailwind/index.tsx:21` — "border-t border-neutral-950 dark:border-white",
/// repeated on items 2..n in every demo).
const DEMO_ITEM_BORDER_CLASS: &str = "border-t border-neutral-950 dark:border-white";

/// The upstream Trigger `className`
/// (`hero/tailwind/index.tsx:9` — the `group` hover/focus treatment, shared verbatim
/// by every trigger in every demo).
const DEMO_TRIGGER_CLASS: &str = "group flex w-full items-center justify-between gap-4 bg-transparent px-3 py-2 text-left text-sm font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 focus-visible:relative focus-visible:z-1 focus-visible:outline-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white dark:text-white dark:hover:not-data-disabled:bg-neutral-800";

/// The upstream Panel `className`
/// (`hero/tailwind/index.tsx:15` — the height-animated panel:
/// `--accordion-panel-height` sizing, `data-starting-style`/`data-ending-style`
/// h-0 transitions).
const DEMO_PANEL_CLASS: &str = "h-[var(--accordion-panel-height)] overflow-hidden text-sm transition-[height] duration-150 ease-[ease-out] data-ending-style:h-0 data-starting-style:h-0";

/// The upstream PlusIcon (`hero/tailwind/index.tsx:46-58`): a 16x16 plus that the
/// `group` trigger's `group-data-panel-open:rotate-45` variant rotates into an X
/// while the panel is open. The icon class rides the real trigger's
/// `data-panel-open` attribute (the port flips it from the root's value array).
fn plus_icon() -> impl IntoView {
    view! {
        <svg
            class="shrink-0 transition-transform duration-100 ease-[ease-out] group-data-panel-open:rotate-45"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-linecap="square"
            stroke-linejoin="round"
            style="display: block"
        >
            <path d="M1.5 8h13M8 14.5v-13" />
        </svg>
    }
}

/// One FAQ item of the demos (`hero/tailwind/index.tsx:6-20` shape): Header wrapping
/// Trigger, Panel as a sibling inside Item — the upstream Anatomy assembly. `first`
/// drops the border-t class (upstream item 1 has no `className`).
fn faq_item(question: &'static str, answer: &'static str, first: bool) -> impl IntoView {
    view! {
        <AccordionItem class=(!first).then(|| DEMO_ITEM_BORDER_CLASS.to_string())>
            <AccordionHeader>
                <AccordionTrigger class=Some(DEMO_TRIGGER_CLASS.to_string())>
                    {question}
                    {plus_icon()}
                </AccordionTrigger>
            </AccordionHeader>
            <AccordionPanel class=Some(DEMO_PANEL_CLASS.to_string())>
                <div class="px-3 py-2">{answer}</div>
            </AccordionPanel>
        </AccordionItem>
    }
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): a default
/// single-open accordion of three FAQ-style items, no props on Root
/// (demos.json `propsExercised.Accordion.Root: []`).
#[component]
pub fn AccordionHeroDemo() -> impl IntoView {
    view! {
        <AccordionRoot class=Some(DEMO_ROOT_CLASS.to_string())>
            {faq_item(
                "What is Base UI?",
                "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
                true,
            )}
            {faq_item(
                "How do I get started?",
                "Head to the \u{201c}Quick start\u{201d} guide in the docs. If you\u{2019}ve used unstyled libraries before, you\u{2019}ll feel at home.",
                false,
            )}
            {faq_item("Can I use it for my project?", "Of course! Base UI is free and open source.", false)}
        </AccordionRoot>
    }
}

/// The "Open multiple panels" demo (`demos/multiple/tailwind/index.tsx`, demos.json
/// entry 3): the hero FAQ accordion with `multiple` set on Root
/// (`propsExercised.Accordion.Root: ["multiple"]`), so several panels can be open at
/// the same time.
#[component]
pub fn MultipleDemo() -> impl IntoView {
    view! {
        <AccordionRoot class=Some(DEMO_ROOT_CLASS.to_string()) multiple=true>
            {faq_item(
                "What is Base UI?",
                "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
                true,
            )}
            {faq_item(
                "How do I get started?",
                "Head to the \u{201c}Quick start\u{201d} guide in the docs. If you\u{2019}ve used unstyled libraries before, you\u{2019}ll feel at home.",
                false,
            )}
            {faq_item("Can I use it for my project?", "Of course! Base UI is free and open source.", false)}
        </AccordionRoot>
    }
}

/// The "Hidden until found" demo (`demos/hidden-until-found/tailwind/index.tsx`,
/// demos.json entry 2): a shipping-FAQ accordion with `hiddenUntilFound` set once on
/// Root (`propsExercised.Accordion.Root: ["hiddenUntilFound"]`), so closed panels
/// stay mounted with `hidden="until-found"` and browser find-in-page (searching
/// "restocking") reveals the matching panel.
#[component]
pub fn HiddenUntilFoundDemo() -> impl IntoView {
    view! {
        <AccordionRoot class=Some(DEMO_ROOT_CLASS.to_string()) hidden_until_found=true>
            {faq_item(
                "How long does shipping take?",
                "Standard shipping takes 3\u{2013}5 business days. Express delivery arrives in 1\u{2013}2 business days.",
                true,
            )}
            {faq_item(
                "What is your return policy?",
                "You can return any item within 30 days of delivery. Opened items may be subject to a 10% restocking fee.",
                false,
            )}
            {faq_item(
                "Do you ship internationally?",
                "Yes, we ship to over 40 countries. International orders typically arrive within 7\u{2013}14 business days.",
                false,
            )}
            {faq_item(
                "How can I track my order?",
                "Once your order ships, you\u{2019}ll receive a tracking link by email. Tracking updates can take up to 24 hours to appear.",
                false,
            )}
        </AccordionRoot>
    }
}

/// One `### <Part>` section of the `## API reference`, in upstream's `<TypesAccordion.<Part> />`
/// shape: the part's generated summary, its props table, its data-attributes table (plus the
/// CSS-variables table when the part has one — only `Accordion.Panel` does), and the part's
/// `Additional Types` panels.
fn api_part(
    part: &'static str,
    summary: &'static [Segment],
    props: &'static [ReferenceProp],
    table_id: &'static str,
    data_attributes: &'static [DataAttributeRow],
    css_variables: &'static [DataAttributeRow],
    additional_types: &'static [AdditionalType],
) -> impl IntoView {
    view! {
        {reference::part_section_heading(part)}
        {reference::part_summary(summary)}
        {reference::props_section(props, table_id)}
        {reference::data_attributes_table(data_attributes)}
        {(!css_variables.is_empty())
            .then(|| reference::css_variables_table(css_variables))}
        {reference::additional_types(additional_types)}
    }
}

/// The `docs/src/app/(docs)/react/components/accordion/page.mdx` page.
#[component]
pub fn AccordionPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Accordion"</h1>
            <p class="subtitle">"A set of collapsible panels with headings."</p>

            <div class="docs-demo" data-demo="hero"><AccordionHeroDemo /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Examples"</h2>
            <h3>"Open multiple panels"</h3>
            <p>
                "You can set up the accordion to allow multiple panels to be open at the "
                "same time using the `multiple` prop."
            </p>
            <div class="docs-demo" data-demo="multiple"><MultipleDemo /></div>

            <h3>"Hidden until found"</h3>
            <p>
                "The `hiddenUntilFound` prop hides closed panels with `hidden=\"until-found\"` "
                "so the browser can search their contents and reveal the matching panel "
                "automatically. It can be set on each `Accordion.Panel`, or once on "
                "`Accordion.Root` to apply to all panels."
            </p>
            <p>
                "To try it, press Ctrl+F (Cmd+F on macOS) and search for \"restocking\" — "
                "the browser opens the closed panel containing the match. When "
                "`hiddenUntilFound` is enabled, closed panels always remain mounted in the "
                "DOM, which also makes their contents indexable by search engines."
            </p>
            <p>
                "Older browsers that don't support `hidden=\"until-found\"` keep panels hidden "
                "until their trigger opens them, and find-in-page skips over the contents."
            </p>
            <div class="docs-demo" data-demo="hidden-until-found"><HiddenUntilFoundDemo /></div>

            <h2 id="api-reference">"API reference"</h2>
            {api_part(
                "Root",
                ROOT_SUMMARY,
                ROOT_PROPS,
                "accordion-root-props-table",
                ROOT_DATA_ATTRIBUTES,
                &[],
                ROOT_ADDITIONAL_TYPES,
            )}
            {api_part(
                "Item",
                ITEM_SUMMARY,
                ITEM_PROPS,
                "accordion-item-props-table",
                ITEM_DATA_ATTRIBUTES,
                &[],
                ITEM_ADDITIONAL_TYPES,
            )}
            {api_part(
                "Header",
                HEADER_SUMMARY,
                HEADER_PROPS,
                "accordion-header-props-table",
                HEADER_DATA_ATTRIBUTES,
                &[],
                HEADER_ADDITIONAL_TYPES,
            )}
            {api_part(
                "Trigger",
                TRIGGER_SUMMARY,
                TRIGGER_PROPS,
                "accordion-trigger-props-table",
                TRIGGER_DATA_ATTRIBUTES,
                &[],
                TRIGGER_ADDITIONAL_TYPES,
            )}
            {api_part(
                "Panel",
                PANEL_SUMMARY,
                PANEL_PROPS,
                "accordion-panel-props-table",
                PANEL_DATA_ATTRIBUTES,
                PANEL_CSS_VARIABLES,
                PANEL_ADDITIONAL_TYPES,
            )}
        </article>
    }
}

/// Browser-free guard for `specs/docs-content/CONTRACT.md` requirement 1: the snippet embedded in
/// this page demonstrates the PORT's API, in the port's own shape.
///
/// Why it exists: the obligation was unenforced in the host gate — `run-regression.sh` runs
/// `cargo test --workspace`, and the snippet-language probe lives in
/// `ralph/scripts/visual-gap-report.mjs`, which needs BOTH dev servers up. This page carried
/// upstream's JSX for the whole of its life behind every green gate, and because transcribed text
/// counted toward content recall, keeping it there *raised* the fidelity score. This module is the
/// cheap half of that obligation: it runs in the ordinary host suite, and it fails the moment the
/// snippet teaches upstream instead of the port.
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify, looks_leptos, looks_react};

    /// The upstream Anatomy block (`page.mdx:17-28`), kept here as the classifier's positive control:
    /// if `looks_react` ever stops recognising upstream's source, the assertions below would pass
    /// vacuously, and this test would say so instead.
    const UPSTREAM_ANATOMY: &str = "import { Accordion } from '@base-ui/react/accordion';\n\n<Accordion.Root>\n  <Accordion.Item>\n    <Accordion.Header>\n      <Accordion.Trigger />\n    </Accordion.Header>\n    <Accordion.Panel />\n  </Accordion.Item>\n</Accordion.Root>;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert!(
            looks_react(UPSTREAM_ANATOMY) && !looks_leptos(UPSTREAM_ANATOMY),
            "the classifier no longer recognises upstream's React source — the assertions below \
             would be vacuous"
        );
        assert_eq!(
            classify(UPSTREAM_ANATOMY),
            SnippetLanguage::React,
            "upstream's source must classify as React"
        );
    }

    /// The port's snippet is `view!` markup over `#[component]` functions — `<AccordionRoot>` is
    /// spelled exactly like the JSX tag `looks_react` hunts for, so this is also the regression test
    /// for the exclusive rule that keeps an idiomatic Leptos snippet from being read as upstream's
    /// React (see `crate::snippet_language::looks_leptos_exclusive`).
    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let snippets = [("Anatomy", ANATOMY_SNIPPET)];
        let (mut leptos, mut react, mut other) = (0, 0, 0);
        for (name, text) in snippets {
            match classify(text) {
                SnippetLanguage::Leptos => leptos += 1,
                SnippetLanguage::React => {
                    react += 1;
                    panic!("the '{name}' snippet still carries React source");
                }
                SnippetLanguage::Other => {
                    other += 1;
                    panic!("the '{name}' snippet identifies as neither port nor React source");
                }
            }
        }
        assert_eq!(
            (leptos, react, other),
            (1, 0, 0),
            "the probe must read {{total: 1, leptos: 1, react: 0}} for this page"
        );
    }

    // --- the snippet's shape, compiled -------------------------------------------------------
    // Mirrors `ANATOMY_SNIPPET` verbatim (its imports are the `leptos_ui` ones at the top of this
    // file). Never called: the compiler checks the parts, props and paths the page teaches.

    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        view! {
            <AccordionRoot>
                <AccordionItem>
                    <AccordionHeader>
                        <AccordionTrigger>"Trigger"</AccordionTrigger>
                    </AccordionHeader>
                    <AccordionPanel>"Panel"</AccordionPanel>
                </AccordionItem>
            </AccordionRoot>
        }
    }
}

/// Browser-free guard for the `## API reference` content: the generated tables are the one part of
/// this page whose content comes from a file outside this repo's Rust test path
/// (`docs/src/app/(docs)/react/components/accordion/types.md`, regenerated by `pnpm docs:api`), and
/// every structural check passes for any content — `render_test.rs` asserts the shape of the tables,
/// which does not notice a mistyped prop name, a dropped row or a reordered list. The row sets and
/// their order are asserted here instead, so the transcription is checked without a browser and a
/// future regeneration of the documented surface fails loudly rather than silently.
#[cfg(test)]
mod reference_content_guard {
    use super::*;

    /// `types.md:14-31` — the generated `**Root Props:**` rows, in order.
    const ROOT_PROP_NAMES: [&str; 12] = [
        "defaultValue",
        "value",
        "onValueChange",
        "hiddenUntilFound",
        "loopFocus",
        "multiple",
        "disabled",
        "orientation",
        "className",
        "style",
        "keepMounted",
        "render",
    ];

    /// `types.md:156-165` — the generated `**Item Props:**` rows, in order.
    const ITEM_PROP_NAMES: [&str; 6] = [
        "value",
        "onOpenChange",
        "disabled",
        "className",
        "style",
        "render",
    ];

    /// `types.md:249-255` — the generated `**Header Props:**` rows, in order.
    const HEADER_PROP_NAMES: [&str; 3] = ["className", "style", "render"];

    /// `types.md:101-108` — the generated `**Trigger Props:**` rows, in order.
    const TRIGGER_PROP_NAMES: [&str; 4] = ["nativeButton", "className", "style", "render"];

    /// `types.md:304-312` — the generated `**Panel Props:**` rows, in order.
    const PANEL_PROP_NAMES: [&str; 5] = [
        "hiddenUntilFound",
        "className",
        "style",
        "keepMounted",
        "render",
    ];

    /// `types.md:35-38` — the generated `**Root Data Attributes:**` rows, in order.
    const ROOT_DATA_ATTRIBUTE_NAMES: [&str; 2] = ["data-orientation", "data-disabled"];

    /// `types.md:176-182` — the generated `**Item Data Attributes:**` rows, in order.
    const ITEM_DATA_ATTRIBUTE_NAMES: [&str; 3] = ["data-open", "data-disabled", "data-index"];

    /// `types.md:257-263` — the generated `**Header Data Attributes:**` rows, in order.
    const HEADER_DATA_ATTRIBUTE_NAMES: [&str; 3] = ["data-open", "data-disabled", "data-index"];

    /// `types.md:110-115` — the generated `**Trigger Data Attributes:**` rows, in order.
    const TRIGGER_DATA_ATTRIBUTE_NAMES: [&str; 2] = ["data-panel-open", "data-disabled"];

    /// `types.md:314-323` — the generated `**Panel Data Attributes:**` rows, in order.
    const PANEL_DATA_ATTRIBUTE_NAMES: [&str; 6] = [
        "data-open",
        "data-orientation",
        "data-disabled",
        "data-index",
        "data-starting-style",
        "data-ending-style",
    ];

    /// `types.md:325-330` — the generated `**Panel CSS Variables:**` rows, in order.
    const PANEL_CSS_VARIABLE_NAMES: [&str; 2] =
        ["--accordion-panel-height", "--accordion-panel-width"];

    #[test]
    fn the_transcribed_rows_match_the_generated_types_content() {
        assert_eq!(
            ROOT_PROPS.iter().map(|prop| prop.name).collect::<Vec<_>>(),
            ROOT_PROP_NAMES.to_vec()
        );
        assert_eq!(
            ITEM_PROPS.iter().map(|prop| prop.name).collect::<Vec<_>>(),
            ITEM_PROP_NAMES.to_vec()
        );
        assert_eq!(
            HEADER_PROPS
                .iter()
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            HEADER_PROP_NAMES.to_vec()
        );
        assert_eq!(
            TRIGGER_PROPS
                .iter()
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            TRIGGER_PROP_NAMES.to_vec()
        );
        assert_eq!(
            PANEL_PROPS.iter().map(|prop| prop.name).collect::<Vec<_>>(),
            PANEL_PROP_NAMES.to_vec()
        );
        assert_eq!(
            ROOT_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            ROOT_DATA_ATTRIBUTE_NAMES.to_vec()
        );
        assert_eq!(
            ITEM_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            ITEM_DATA_ATTRIBUTE_NAMES.to_vec()
        );
        assert_eq!(
            HEADER_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            HEADER_DATA_ATTRIBUTE_NAMES.to_vec()
        );
        assert_eq!(
            TRIGGER_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            TRIGGER_DATA_ATTRIBUTE_NAMES.to_vec()
        );
        assert_eq!(
            PANEL_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            PANEL_DATA_ATTRIBUTE_NAMES.to_vec()
        );
        assert_eq!(
            PANEL_CSS_VARIABLES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            PANEL_CSS_VARIABLE_NAMES.to_vec()
        );
    }

    /// The anchors are what upstream's `#AccordionRoot-<name>` deep links address, and the port's
    /// rows carry the same ones — a mistyped anchor is a silently broken link, which no structural
    /// check would see.
    #[test]
    fn every_prop_row_carries_upstreams_anchor() {
        for prop in ROOT_PROPS {
            assert_eq!(prop.anchor, format!("AccordionRoot-{}", prop.name));
        }
        for prop in ITEM_PROPS {
            assert_eq!(prop.anchor, format!("AccordionItem-{}", prop.name));
        }
        for prop in HEADER_PROPS {
            assert_eq!(prop.anchor, format!("AccordionHeader-{}", prop.name));
        }
        for prop in TRIGGER_PROPS {
            assert_eq!(prop.anchor, format!("AccordionTrigger-{}", prop.name));
        }
        for prop in PANEL_PROPS {
            assert_eq!(prop.anchor, format!("AccordionPanel-{}", prop.name));
        }
    }

    /// The `Additional Types` panels: 15 headings and the five `Re-Export of …` lines upstream's
    /// `<TypesAccordion.* />` components render, i.e. the part sets and their order.
    #[test]
    fn the_additional_type_panels_match_the_generated_types_content() {
        let counts = [
            (ROOT_ADDITIONAL_TYPES, 5),
            (ITEM_ADDITIONAL_TYPES, 4),
            (HEADER_ADDITIONAL_TYPES, 2),
            (TRIGGER_ADDITIONAL_TYPES, 2),
            (PANEL_ADDITIONAL_TYPES, 2),
        ];
        for (types, expected) in counts {
            assert_eq!(types.len(), expected);
        }
        assert_eq!(
            ROOT_ADDITIONAL_TYPES
                .iter()
                .map(|ty| ty.name)
                .collect::<Vec<_>>(),
            vec![
                "Accordion.Root.Props",
                "Accordion.Root.State",
                "Accordion.Root.ChangeEventReason",
                "Accordion.Root.ChangeEventDetails",
                "Accordion.Root.Value",
            ]
        );
        assert_eq!(
            PANEL_ADDITIONAL_TYPES
                .iter()
                .map(|ty| ty.name)
                .collect::<Vec<_>>(),
            vec!["Accordion.Panel.Props", "Accordion.Panel.State"]
        );
        // Upstream's `Re-Export of <part> props as <Alias>` line, one per part.
        let re_exports = [
            (ROOT_ADDITIONAL_TYPES, "AccordionRootProps"),
            (ITEM_ADDITIONAL_TYPES, "AccordionItemProps"),
            (HEADER_ADDITIONAL_TYPES, "AccordionHeaderProps"),
            (TRIGGER_ADDITIONAL_TYPES, "AccordionTriggerProps"),
            (PANEL_ADDITIONAL_TYPES, "AccordionPanelProps"),
        ];
        for (types, alias) in re_exports {
            let re_export = types
                .iter()
                .find(|ty| ty.re_export_of.is_some())
                .expect("a .Props panel");
            assert_eq!(
                re_export.name,
                format!("Accordion.{}.Props", {
                    let part = types[0].name.split('.').nth(1).unwrap();
                    part
                })
            );
            assert_eq!(re_export.re_export_of.unwrap().1, alias);
        }
    }
}
