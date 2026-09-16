//! The demo registry — the contract between the port and the sandbox.
//!
//! One entry per demo a reader can open here. Each `mount` fn is the *same composition the docs
//! page renders*, written against the **published** crate: `use leptos_ui::Accordion;` and
//! `<Accordion::Root>` etc. — the spelling the docs snippets teach, since a reader who copies code
//! out of here must get something that compiles against crates.io.
//!
//! Adding a demo is the whole cost of extending the sandbox: a `#[component]` fn plus one entry
//! here. Per the owner's decision (SANDBOX-PLAN.md §0), a ported demo is not fully delivered until
//! it is also reachable from this registry — so a demo that exists only on the docs page is
//! unfinished work, and the ledger carries it as visible debt rather than as silence.

use leptos::prelude::*;
use leptos_ui::Accordion;

/// One demo the sandbox can mount.
pub struct Demo {
    /// Stable identifier used in `?demo=<slug>` and in the docs' "Open in CodeSandbox" link.
    pub slug: &'static str,
    /// Human title for the switcher.
    pub title: &'static str,
    /// The upstream demo this port renders, for the footer's provenance line.
    pub upstream: &'static str,
    /// Mounts the demo.
    pub mount: fn() -> AnyView,
}

/// Every registered demo, in switcher order.
pub const ALL: &[Demo] = &[Demo {
    slug: "accordion-hero",
    title: "Accordion (hero)",
    upstream: "docs/src/app/(docs)/react/components/accordion/demos/hero/tailwind/index.tsx",
    mount: || view! { <AccordionHero /> }.into_any(),
}];

/// The registry entry for `slug`, if one exists.
pub fn find(slug: &str) -> Option<&'static Demo> {
    ALL.iter().find(|demo| demo.slug == slug)
}

// The class strings are upstream's, carried verbatim (the docs page stores the same three
// constants); the demo's layout/visual semantics depend on them, and the Tailwind browser runtime
// in `index.html` is what makes them live here.
const DEMO_ROOT_CLASS: &str = "flex w-full max-w-80 flex-col border border-neutral-950 text-neutral-950 dark:border-white dark:text-white";
const DEMO_ITEM_BORDER_CLASS: &str = "border-t border-neutral-950 dark:border-white";
const DEMO_TRIGGER_CLASS: &str = "group flex w-full items-center justify-between gap-4 bg-transparent px-3 py-2 text-left text-sm font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 focus-visible:relative focus-visible:z-1 focus-visible:outline-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white dark:text-white dark:hover:not-data-disabled:bg-neutral-800";
const DEMO_PANEL_CLASS: &str = "h-[var(--accordion-panel-height)] overflow-hidden text-sm transition-[height] duration-150 ease-[ease-out] data-ending-style:h-0 data-starting-style:h-0";

/// The upstream PlusIcon (`hero/tailwind/index.tsx:46-58`), copied from the port's docs page
/// (`crates/docs-app/src/pages/accordion_page.rs:plus_icon`) rather than re-invented: a 16x16 plus
/// that the `group` trigger's `group-data-panel-open:rotate-45` variant rotates into an X while the
/// panel is open.
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

/// One FAQ item, mirroring the docs page's `faq_item` exactly: Item > Header > Trigger/Panel, no
/// `value` prop (upstream's hero demo passes none either), and `first` only drops the Border-t
/// class because upstream's first item carries no `className`.
fn faq_item(question: &'static str, answer: &'static str, first: bool) -> impl IntoView {
    view! {
        <Accordion::Item class=(!first).then(|| DEMO_ITEM_BORDER_CLASS.to_string())>
            <Accordion::Header>
                <Accordion::Trigger class=Some(DEMO_TRIGGER_CLASS.to_string())>
                    {question}
                    {plus_icon()}
                </Accordion::Trigger>
            </Accordion::Header>
            <Accordion::Panel class=Some(DEMO_PANEL_CLASS.to_string())>
                <div class="px-3 py-2">{answer}</div>
            </Accordion::Panel>
        </Accordion::Item>
    }
}

/// The hero demo: a default single-open accordion of three FAQ items, no props on Root
/// (upstream `demos/hero/tailwind/index.tsx`, `propsExercised.Accordion.Root: []`).
#[component]
pub fn AccordionHero() -> impl IntoView {
    view! {
        <Accordion::Root class=Some(DEMO_ROOT_CLASS.to_string())>
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
        </Accordion::Root>
    }
}
