//! The docs page for `Accordion`, mirroring
//! `docs/src/app/(docs)/react/components/accordion/page.mdx`
//! (`specs/docs-content/accordion/page.md`) — the `docs-content: components/accordion`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Accordion` h1,
//! `<Subtitle>` ("A set of collapsible panels with headings."), the hero demo before
//! the first heading, `## Anatomy` with the single fenced snippet, `## Examples` over
//! "Open multiple panels" (`./demos/multiple`) and "Hidden until found"
//! (`./demos/hidden-until-found`), and `## API reference` over the five generated
//! `TypesAccordion` reference tables (echoed as static prose per the toggle/separator
//! page precedent — the port has no docs generator, so the tables' documented
//! props/data-attributes/CSS variables are rendered as text, never fabricated as
//! executable machinery).
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

use leptos::prelude::*;

use leptos_ui::{AccordionHeader, AccordionItem, AccordionPanel, AccordionRoot, AccordionTrigger};

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

/// One API-reference part block: the generated `TypesAccordion.<Part />` tables
/// (`docs/src/app/(docs)/react/components/accordion/types.md`) echoed as static
/// prose — the summary line, the props list, and the data-attributes list.
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
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
            <pre><code>
"import { Accordion } from '@base-ui/react/accordion';

<Accordion.Root>
  <Accordion.Item>
    <Accordion.Header>
      <Accordion.Trigger />
    </Accordion.Header>
    <Accordion.Panel />
  </Accordion.Item>
</Accordion.Root>;"
            </code></pre>

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

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                "Groups all parts of the accordion. Renders a <div> element.",
                "Props: defaultValue (Value[] — the uncontrolled initially-expanded item(s); use value for controlled), value (Value[] — controlled expanded item(s)), onValueChange (value, eventDetails), hiddenUntilFound (boolean, false — find-in-page reveal via hidden=\"until-found\"; overrides keepMounted), loopFocus (deprecated no-op), multiple (boolean, false), disabled (boolean, false), orientation ('horizontal' | 'vertical', default 'vertical' — deprecated no-op), keepMounted (boolean, false — ignored when hiddenUntilFound is used), className, style, render.",
                "Data attributes: data-orientation (the accordion's orientation), data-disabled (present when the accordion is disabled).",
            )}
            <h3>"Item"</h3>
            {api_part(
                "Groups an accordion header with the corresponding panel. Renders a <div> element.",
                "Props: value (a unique value identifying this item; a unique ID is generated when omitted — set it to control the accordion programmatically or give an item an initial open state), onOpenChange (open, eventDetails), disabled (boolean, false), className, style, render.",
                "Data attributes: data-open, data-disabled, data-index (the item's index).",
            )}
            <h3>"Header"</h3>
            {api_part(
                "A heading that labels the corresponding panel. Renders an <h3> element.",
                "Props: className, style, render.",
                "Data attributes: data-open, data-disabled, data-index.",
            )}
            <h3>"Trigger"</h3>
            {api_part(
                "A button that opens and closes the corresponding panel. Renders a <button> element.",
                "Props: nativeButton (boolean, true — set false when the render prop replaces the button element), className, style, render.",
                "Data attributes: data-panel-open (present when the panel is open), data-disabled (present when the item is disabled).",
            )}
            <h3>"Panel"</h3>
            {api_part(
                "A collapsible panel with the accordion item contents. Renders a <div> element.",
                "Props: hiddenUntilFound (boolean, false — overrides keepMounted, hides with hidden=\"until-found\"), keepMounted (boolean, false — keep the element in the DOM while closed; ignored when hiddenUntilFound is used), className, style, render.",
                "Data attributes: data-open, data-orientation, data-disabled, data-index, data-starting-style (animating in), data-ending-style (animating out). CSS variables: --accordion-panel-height, --accordion-panel-width.",
            )}
        </article>
    }
}
