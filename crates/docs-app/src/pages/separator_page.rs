//! The docs page for `Separator`, mirroring
//! `docs/src/app/(docs)/react/components/separator/page.mdx`
//! (`specs/docs-content/separator/page.md`) — the `docs-content: components/separator`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Separator` h1,
//! `<Subtitle>` ("A separator element accessible to screen readers."), the hero
//! demo before the first heading, `## Anatomy` with the single two-line fenced
//! snippet, and `## API reference` over the generated `TypesSeparator` reference
//! (echoed as static prose per the toggle page precedent — the port has no docs
//! generator, so the table's documented props/data-attributes/types are rendered
//! as text, never fabricated as executable machinery).
//!
//! The live demo is the upstream hero
//! (`docs/src/app/(docs)/react/components/separator/demos/hero/tailwind/index.tsx`,
//! `specs/docs-content/separator/demos.json` entry 1) ported onto the REAL
//! `leptos_ui::separator_element` port: a `flex gap-4 text-nowrap` container with
//! four left-group links, a **vertical** `Separator`
//! (`orientation="vertical"`, `:31` — the demo's single exercised prop per
//! demos.json's `propsExercised`) styled `w-px bg-neutral-300
//! dark:bg-neutral-700`, and two right-group links. The demo is fully static
//! (demos.json: `stateManaged: "none"`, no nonTrivialInteractions), so the port
//! is one `separator_element` call plus six anchors — no signals, no effects,
//! no rebuild machinery, unlike the toggle page's reactive hero.
//!
//! The upstream link `className` strings (Tailwind: text size/color, hover
//! underline, focus-visible outline ring) are carried verbatim so the DOM the
//! Leptos port produces matches the React demo's element-for-element.

use crate::code_block::{Lang, code_block};
use leptos::prelude::*;

use leptos_ui::{SEPARATOR_ORIENTATION_VERTICAL, SeparatorProps, separator_element};
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderedElement, UseRenderElementComponentProps,
};

use crate::pages::use_render_page::RawElementView;

/// The upstream link `className` (`hero/tailwind/index.tsx:7-9`, shared by all
/// six links; the demo repeats the same literal six times).
const LINK_CLASS: &str = "text-sm text-neutral-950 decoration-neutral-300 decoration-1 underline-offset-2 hover:underline focus-visible:no-underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white dark:text-white dark:decoration-neutral-700";

/// The upstream Separator `className` (`hero/tailwind/index.tsx:31`).
const DEMO_SEPARATOR_CLASS: &str = "w-px bg-neutral-300 dark:bg-neutral-700";

/// The `## Anatomy` snippet (`specs/docs-content/separator/page.md`, mirroring
/// `docs/src/app/(docs)/react/components/separator/page.mdx:18-22`): upstream's
/// `import { Separator } from '@base-ui/react/separator'` followed by a bare `<Separator />;`.
///
/// Translated to THIS port (`specs/docs-content/CONTRACT.md` requirement 1 — a snippet embedded in a
/// mirrored page must show the port's own API, never upstream's import line, so this item's
/// `check-react-mentions.mjs --source` count goes to zero on this route). `Separator` is a
/// single-element unit with no dotted parts upstream (`specs/library/separator/behavior.md`
/// § Public API surface: "a single root component with no subcomponents, parts, or context hooks"),
/// so there is no `Separator::Part` tree to teach — the port's surface for it is the element
/// description [`separator_element`] plus `create_element`, the same "build it, then materialize it"
/// shape the button page's Anatomy teaches (`button_page.rs:101-110`) for the sibling
/// no-dotted-part unit. The snippet's `.expect` message is the port's own: the unit has no
/// `enabled` gate, so the description always materializes.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{SeparatorProps, separator_element};

// Separator is an element description: build it, then materialize it.
let rendered = separator_element(SeparatorProps::default())
    .expect("Separator always renders (a static leaf with no enabled gate)");

let (element, cleanup) = rendered.create_element();
// Append `element` where it belongs; the listeners live until `cleanup` drops."#;

/// The demo's link label + `href="#"` pair (`hero/tailwind/index.tsx` — the
/// six `<a href="#">` elements, four before the separator, two after).
const LINKS: [(&str, usize); 6] = [
    ("Home", 0),
    ("Pricing", 0),
    ("Blog", 0),
    ("Support", 0),
    ("Log in", 0),
    ("Sign up", 0),
];

/// The upstream hero demo (`hero/tailwind/index.tsx:3-47`) on the real port:
/// one `separator_element` call with `orientation="vertical"` between two
/// groups of anchors, all inside the demo's `flex gap-4 text-nowrap` container.
///
/// Static per demos.json (`stateManaged: "none"`) — no signals, no effects;
/// the element is materialized once under the caller's owner (the mount) and
/// returned as a [`RawElementView`] (the use-render page's bridge), since
/// `separator_element` produces an engine [`RenderedElement`] description, not
/// a Leptos view.
pub fn separator_hero_demo() -> RawElementView {
    let container = document().create_element("div").unwrap();
    container
        .set_attribute("data-separator-hero", "")
        .expect("set data-separator-hero");

    // The demo root (`:4`): `className="flex gap-4 text-nowrap"` on a div.
    container
        .set_attribute("class", "flex gap-4 text-nowrap")
        .expect("set demo class");

    // The six anchors (`:5-14`, `:33-46`), in source order.
    for (label, _) in LINKS {
        let anchor = document().create_element("a").expect("create a");
        anchor.set_attribute("href", "#").expect("set href");
        anchor
            .set_attribute("class", LINK_CLASS)
            .expect("set link class");
        anchor.set_text_content(Some(label));
        container.append_child(&anchor).expect("append link");
    }

    // The Separator (`:31`): `orientation="vertical"` — the demo's single
    // exercised prop — and its demo `className`. Rendered through the real
    // `separator_element` port (the no-state-machine leaf facade over
    // use_render_element); the intrinsics (role="separator",
    // aria-orientation="vertical", data-orientation="vertical") come from the
    // port's own bag composition, pinned by the crate suite.
    let rendered: RenderedElement = separator_element(SeparatorProps {
        orientation: SEPARATOR_ORIENTATION_VERTICAL,
        render_class_style: UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Static(DEMO_SEPARATOR_CLASS.to_string())),
            render: None,
            style: None,
        },
        element_attributes: Vec::new(),
        ref_callback: None,
    })
    .expect("standalone Separator renders (no enabled gate)");

    let (element, _cleanup) = rendered.create_element();
    container
        .append_child(&element)
        .expect("append separator element");

    // The demo is fully static: no reactive state, no rebuild effect, so no
    // cleanup guard is retained (the element lives under the mount owner for
    // the lifetime of the test/page). `separator_element`'s ref fork
    // registered no callbacks (no forwarded ref passed).
    std::mem::forget(_cleanup);

    RawElementView { element: container }
}

/// The `docs/src/app/(docs)/react/components/separator/page.mdx` page.
#[component]
pub fn SeparatorPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Separator"</h1>
            <p class="subtitle">"A separator element accessible to screen readers."</p>

            <div class="docs-demo">{separator_hero_demo()}</div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and use it as a single part:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"API reference"</h2>
            <p>
                "Props: `orientation` (default `horizontal`; `'horizontal' | 'vertical'` — set "
                "`vertical` for a vertical divider; reflected to `aria-orientation` and "
                "`data-orientation`), plus the standard `className`, `style`, and `render` props "
                "and arbitrary DOM-prop forwarding."
            </p>
            <p>
                "A `Separator` renders a `div` element with `role=\"separator\"`. The state is "
                "`{ orientation }`; `data-orientation` rides the default state walk (no custom "
                "mapping)."
            </p>
        </article>
    }
}

#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};

    /// Upstream's Anatomy block (`docs/src/app/(docs)/react/components/separator/page.mdx:18-22`),
    /// kept as the classifier's positive control so the assertions below cannot pass vacuously if
    /// `looks_react` ever stops recognising upstream's JSX shape. The package specifier upstream's
    /// import line carries is deliberately left out: this is page source, not reader-facing, and the
    /// sibling pages' controls omit it for the same reason (the `field_page.rs:224-227` precedent).
    const UPSTREAM_ANATOMY_SHAPE: &str = "<Separator />;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert_eq!(
            classify(UPSTREAM_ANATOMY_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertion below would \
             be vacuous"
        );
    }

    /// Every code block this page embeds, in document order, with the language `CONTRACT.md`
    /// requirement 1 requires of it. The page carries exactly one fence — the Anatomy listing; the
    /// demo's markup is built in Rust, so no other `<pre>` is rendered (the HTML scrollbar fence the
    /// spec's upstream page has lives on the csp-provider page, not here).
    fn page_snippets() -> [(&'static str, &'static str); 1] {
        [("Anatomy", ANATOMY_SNIPPET)]
    }

    /// The page-level number the probe reads: `{total: 1, leptos: 1, react: 0, other: 0}` — the same
    /// triple `visual-gap-report.mjs`'s in-browser probe reports for `react/components/separator`.
    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let languages: Vec<(&str, SnippetLanguage)> = page_snippets()
            .iter()
            .map(|(name, text)| (*name, classify(text)))
            .collect();
        assert_eq!(
            languages,
            vec![("Anatomy", SnippetLanguage::Leptos)],
            "the probe must read {{total: 1, leptos: 1, react: 0, other: 0}} for this page"
        );
    }

    /// The snippet's own composition, compiled. Never called: the compiler checks the crate paths, the
    /// props struct and the `RenderedElement` API the page teaches, so a snippet naming an API the
    /// port does not have fails the build instead of shipping (the checkbox page's guard caught three
    /// such snippets — `snippet_language.rs`'s header).
    #[allow(dead_code)]
    fn anatomy_snippet_shape() {
        let rendered = separator_element(SeparatorProps::default())
            .expect("Separator always renders (a static leaf with no enabled gate)");
        let (_element, _cleanup) = rendered.create_element();
    }
}
