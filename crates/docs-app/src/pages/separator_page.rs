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

use leptos::prelude::*;

use leptos_ui::{separator_element, SeparatorProps, SEPARATOR_ORIENTATION_VERTICAL};
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderedElement, UseRenderElementComponentProps,
};

use crate::pages::use_render_page::RawElementView;

/// The upstream link `className` (`hero/tailwind/index.tsx:7-9`, shared by all
/// six links; the demo repeats the same literal six times).
const LINK_CLASS: &str = "text-sm text-neutral-950 decoration-neutral-300 decoration-1 underline-offset-2 hover:underline focus-visible:no-underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white dark:text-white dark:decoration-neutral-700";

/// The upstream Separator `className` (`hero/tailwind/index.tsx:31`).
const DEMO_SEPARATOR_CLASS: &str = "w-px bg-neutral-300 dark:bg-neutral-700";

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
            <pre><code>
"import { Separator } from '@base-ui/react/separator';

<Separator />;"
            </code></pre>

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
