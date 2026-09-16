//! The docs page for `Avatar`, mirroring
//! `docs/src/app/(docs)/react/components/avatar/page.mdx`
//! (`specs/docs-content/avatar/page.md`) — the `docs-content: components/avatar`
//! TODO item.
//!
//! Page structure per the spec's "Page structure (headings, in order)" section:
//! `# Avatar` h1, the `<Subtitle>` ("An easily stylable avatar component."), the
//! hero demo before the first heading, `## Anatomy` with its fenced snippet,
//! `## Optimized and lazy-loaded images` (`page.mdx:26`) with its two prose
//! paragraphs and the `keepMounted` snippet (upstream's fence is titled for
//! `next/image`, a JavaScript library this port has no counterpart for; the
//! example teaches the prop it is actually about, and the mirrored prose above
//! it is upstream's), `### Stacking` (`:41`) with the
//! three prose paragraphs and the "Stacked image and fallback" css snippet,
//! `### Server rendering` (`:68`), `## API reference` (`:72`) over the three
//! generated `TypesAvatar` reference tables (`### Root`/`### Image`/
//! `### Fallback`), and `## Additional types` (`:90`, preceded by the
//! `@exclude-table-of-contents` marker in the source, `:88`) documenting
//! `ImageLoadingStatus`. The tables are echoed as static prose over the real
//! generated `types.md` per the accordion/button/meter/field/checkbox page
//! precedent: the port has no docs generator, so no fabricated executable
//! machinery.
//!
//! Page furniture mirrored in module docs (the accordion/field/checkbox page
//! precedent): the `<Meta name="description">` content — "A high-quality,
//! unstyled React avatar component that is easy to customize." (`page.mdx:4-7`) —
//! and the trailing `export const metadata` SEO keywords block (13 keywords,
//! `page.mdx:94-110`: 'React Avatar', 'Avatar Component', 'Profile Image UI',
//! 'User Image', 'User Avatar', 'Profile Picture', 'Account Image',
//! 'Profile Icon', 'Initials Fallback', 'Accessible Avatar', 'Headless React
//! Components', 'Customizable Avatar', 'Base UI').
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/avatar/demos/hero/tailwind/index.tsx`,
//! the single `specs/docs-content/avatar/demos.json` entry, whose recorded
//! citation window covers `:1-22`) ported onto the REAL `leptos_ui::avatar`
//! parts: two `<Avatar.Root>` spans side by side — the first nesting
//! `Avatar.Image` (the remote `src`, `width`/`height` 48, `object-cover`) above
//! `Avatar.Fallback` (`delay={600}`, the initials "LT"), the second carrying a
//! bare `LT` TEXT child — exactly upstream's markup order (`:7-15`, `:17-19`).
//! This is the composition the port could not express before
//! `leptos_ui::avatar_root_view` existed: `use_avatar_root` returns a childless
//! materialized span (which rendered the parts as SIBLINGS), while the demo
//! nests them inside the root — the PAIR-PORTABILITY GAP this item's ledger note
//! recorded, now closed in the owner crate. Every upstream `className` string is
//! carried verbatim so the DOM the Leptos port produces matches the React
//! demo's element-for-element.
//!
//! The demo is uncontrolled (`demos.json` "stateManaged": "none — purely
//! declarative; the image loading status and delayed fallback are handled
//! internally by the component"): the image's own status machine decides whether
//! the `<img>` is in the DOM at all (default mode: it is mounted only once
//! loaded — `specs/library/avatar/behavior.md` *DOM structure & portal
//! behavior*) and the 600 ms fallback delay latches the initials behind
//! `useTimeout` (*State model*), exactly as upstream's do.

use crate::code_block::{Lang, code_block};
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use leptos_ui::avatar_fallback_view;
use leptos_ui::avatar_image_view;
use leptos_ui::avatar_root_view;
use leptos_ui::use_avatar_fallback;
use leptos_ui::use_avatar_image;
use leptos_ui::{AvatarFallbackProps, AvatarImageProps, AvatarRootViewProps};
use leptos_ui_internals::use_render_element::{ClassNameSource, UseRenderElementComponentProps};

/// The demo's remote source (`hero/tailwind/index.tsx:10`), carried verbatim.
const DEMO_IMAGE_SRC: &str =
    "https://images.unsplash.com/photo-1543610892-0b1f7e6d8ac1?w=128&h=128&dpr=2&q=80";

/// The upstream Root `className` (`hero/tailwind/index.tsx:7`, repeated at `:16`)
/// — carried verbatim, including the `dark:` variant halves.
const DEMO_ROOT_CLASS: &str = "inline-flex size-8 items-center justify-center overflow-hidden rounded-full bg-neutral-200 align-middle text-sm leading-none font-normal text-neutral-950 select-none dark:bg-neutral-800 dark:text-white";

/// The upstream Image `className` (`hero/tailwind/index.tsx:13`).
const DEMO_IMAGE_CLASS: &str = "size-full object-cover";

/// The upstream Fallback `className` (`hero/tailwind/index.tsx:14`).
const DEMO_FALLBACK_CLASS: &str = "flex size-full items-center justify-center text-sm";

/// The upstream Fallback `delay` (`hero/tailwind/index.tsx:14`): 600 ms, the
/// flash-avoidance latch `demos.json` records under `propsExercised`.
const DEMO_FALLBACK_DELAY_MS: f64 = 600.0;

/// The `## Anatomy` snippet (`page.mdx:17-24`) — "import the component and assemble its parts".
///
/// Upstream's block imports `{ Avatar }` from `@base-ui/react/avatar` and assembles
/// `Avatar.Root > Avatar.Image + Avatar.Fallback`. The port's spelling is the same tree with Rust's
/// path separator (`specs/docs-content/CONTRACT.md` requirement 1's mapping table):
/// `<Avatar::Root>` over `<Avatar::Image>` and `<Avatar::Fallback>`, all three public items of
/// `leptos_ui::Avatar` usable directly in `view!` markup — the surface
/// `crates/leptos-ui/tests/part_surface.rs:90-104` pins and `crates/leptos-ui/src/avatar/mod.rs`
/// documents (the flattened `avatar_root_view(..)`/`avatar_image_view(..)` helpers stay for callers
/// that drive the parts themselves; they are not the teaching surface).
///
/// Two spellings differ from upstream's listing for the port's own reasons, both shown rather than
/// hidden: `Avatar.Fallback`'s content is upstream's element child (`<Avatar.Fallback>LT</…>`) and
/// the engine writes it as the element's HTML content, so the port spells it `inner_html`; and
/// upstream's `src=""` is a listing placeholder, so the example carries a real path (the one
/// upstream's own demo and its `Optimized and lazy-loaded images` example use).
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::Avatar;

view! {
    <Avatar::Root>
        <Avatar::Image src="/avatar.png".to_string() />
        <Avatar::Fallback inner_html="LT".to_string() />
    </Avatar::Root>
}"#;

/// The page's second example (`page.mdx:32-39`), upstream's `jsx title="Using next/image"`.
///
/// Upstream's point is `keepMounted`: the image element is rendered right away and loads in place,
/// which is what composes with an image optimizer. The port exposes exactly that
/// (`Avatar::Image`'s `keep_mounted`, `crates/leptos-ui/src/avatar/mod.rs`), so the example teaches
/// it. Two honest deviations, recorded rather than papered over: the title says what the example now
/// demonstrates (upstream's names `next/image`, a JavaScript library this port has no counterpart
/// for — the mirrored prose above the block still carries upstream's sentence about it, per the
/// page's copy contract), and the width/height/alt attributes upstream passes to the optimizer ride
/// the port's `element_attributes` rest (the same `...elementProps` bag upstream spreads).
const KEEP_MOUNTED_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::Avatar;

view! {
    <Avatar::Root>
        <Avatar::Fallback inner_html="LT".to_string() />
        <Avatar::Image
            keep_mounted=true
            src="/avatar.png".to_string()
            element_attributes=vec![
                ("width".to_string(), "32".to_string()),
                ("height".to_string(), "32".to_string()),
                ("alt".to_string(), "".to_string()),
            ]
        />
    </Avatar::Root>
}"#;

/// The "Stacked image and fallback" snippet (`page.mdx:49-64`), carried
/// verbatim.
const STACKING_SNIPPET: &str = r#".Root {
  position: relative;
}

.Image,
.Fallback {
  position: absolute;
  inset: 0;
}

.Image[data-loading],
.Image[data-error] {
  visibility: hidden;
}"#;

/// Binds a dynamic-part closure as a leptos dynamic-view child — the actual
/// `{move || …}` invocation (`Avatar.Image`/`Avatar.Fallback` in this port are
/// view functions returning closures, so a bare `avatar_image_view(...)` handed
/// to `view!` would bind a never-invoked value; the crate's own wasm suite
/// records that trap).
fn dynamic<V: IntoView + 'static>(
    body: impl Fn() -> V + Send + 'static,
) -> impl IntoView + 'static {
    move || body()
}

/// The hero's image props (`hero/tailwind/index.tsx:8-13`): the remote `src`, the
/// `width`/`height` 48 attributes (they ride the `...elementProps` rest — the
/// engine writes plain DOM attributes, so they stay strings exactly as JSX
/// renders them), and the `object-cover` fill class.
fn hero_image_props() -> AvatarImageProps {
    AvatarImageProps {
        class_style: UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Static(DEMO_IMAGE_CLASS.to_string())),
            render: None,
            style: None,
        },
        element_attributes: vec![
            ("width".to_string(), "48".to_string()),
            ("height".to_string(), "48".to_string()),
        ],
        src: Some(DEMO_IMAGE_SRC.to_string()),
        ..AvatarImageProps::default()
    }
}

/// The hero's fallback props (`hero/tailwind/index.tsx:14-16`): the 600 ms delay
/// and the initials child ("children — rendered text content", behavior.md
/// *Public API surface*, `AvatarFallback.test.tsx:53-66`).
fn hero_fallback_props() -> AvatarFallbackProps {
    AvatarFallbackProps {
        class_style: UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Static(DEMO_FALLBACK_CLASS.to_string())),
            render: None,
            style: None,
        },
        delay: DEMO_FALLBACK_DELAY_MS,
        inner_html: Some("LT".to_string()),
        ..AvatarFallbackProps::default()
    }
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, the single demos.json entry):
/// upstream's exact JSX shape — `<div className="flex gap-4">` over the two
/// avatars, the first nesting Image + Fallback inside its Root, the second a bare
/// text child of its Root (`:17-19`).
///
/// The parts are built INSIDE the root view's children closure, which
/// `avatar_root_view` invokes after providing the root context — so
/// `Avatar.Image`/`Avatar.Fallback` resolve the very root they are nested in
/// (the `AvatarRoot.tsx:41` contract), rather than a sibling root. The props
/// carry `Rc`-backed slots (the ref/event vocabulary) and leptos's `Children` is
/// `Send`, so each crosses into the closure through `SendWrapper` (the crate's
/// own view convention); one copy per part serves both `use_avatar_*` and its
/// dynamic view (the props are static per body run — the port's documented
/// adaptation).
#[component]
pub fn AvatarHeroDemo() -> impl IntoView {
    let image_props = SendWrapper::new(hero_image_props());
    let fallback_props = SendWrapper::new(hero_fallback_props());

    let first_root = avatar_root_view(AvatarRootViewProps {
        class: Some(DEMO_ROOT_CLASS.to_string()),
        children: Some(Box::new(move || {
            // Unwrapped inside the closure: `SendWrapper<T>` is `Send` whatever
            // `T` is, which is what lets the `Rc`-backed props cross into a
            // leptos `Children` closure at all (the crate's view convention).
            let image_props = image_props.take();
            let fallback_props = fallback_props.take();
            let image_handle = use_avatar_image(&image_props);
            let fallback_handle = use_avatar_fallback(&fallback_props);
            view! {
                {dynamic(avatar_image_view(image_handle, image_props))}
                {dynamic(avatar_fallback_view(fallback_handle, fallback_props))}
            }
            .into_any()
        })),
        ..AvatarRootViewProps::default()
    });

    let second_root = avatar_root_view(AvatarRootViewProps {
        class: Some(DEMO_ROOT_CLASS.to_string()),
        children: Some(Box::new(|| ("LT").into_any())),
        ..AvatarRootViewProps::default()
    });

    view! {
        <div class="flex gap-4">
            {first_root}
            {second_root}
        </div>
    }
}

/// One API-reference block: the generated `TypesAvatar` tables
/// (`docs/src/app/(docs)/react/components/avatar/types.md`) echoed as static
/// prose — the summary line, the props list, the data-attributes list, and the
/// documented state type.
fn api_part(
    summary: &'static str,
    props: &'static str,
    data_attrs: &'static str,
    state: &'static str,
) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
        <p class="api-state">{state}</p>
    }
}

/// The `docs/src/app/(docs)/react/components/avatar/page.mdx` page.
#[component]
pub fn AvatarPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Avatar"</h1>
            <p class="subtitle">"An easily stylable avatar component."</p>

            <div class="docs-demo" data-demo="hero"><AvatarHeroDemo /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Optimized and lazy-loaded images"</h2>
            <p>
                "By default, `<Avatar.Image>` preloads `src` and renders the image only once it has loaded. "
                "This doesn't compose with image optimizers such as `next/image`, which serve a different URL "
                "than the raw `src`, or with `loading=\"lazy\"`."
            </p>
            <p>
                "Add the `keepMounted` prop to render the image element right away and let it load in place. "
                "Only the image that is actually displayed is requested:"
            </p>
            {code_block(Lang::Rust, "Using keepMounted", KEEP_MOUNTED_SNIPPET)}

            <h3>"Stacking"</h3>
            <p>
                "With `keepMounted`, the image and the fallback are both present until the image loads. The "
                "image is hidden from assistive technology until then, so the fallback provides the accessible "
                "name on its own."
            </p>
            <p>
                "Stack the two in the same box, and place `<Avatar.Image>` after `<Avatar.Fallback>`. Both are "
                "positioned, so whichever comes later in the DOM paints on top. The fallback then shows through "
                "until the image covers it."
            </p>
            <p>
                "A loading image paints nothing, so the fallback shows through on its own. An image that failed "
                "to load paints a broken-image icon on top of it. Hide the image in either state with the "
                "`data-loading` and `data-error` attributes:"
            </p>
            {code_block(Lang::Css, "Stacked image and fallback", STACKING_SNIPPET)}
            <p>
                "Avoid `display: none` here: an element without a box never intersects the viewport, so "
                "`loading=\"lazy\"` would never fetch the image. `visibility` and `opacity` both keep lazy "
                "loading working."
            </p>

            <h3>"Server rendering"</h3>
            <p>
                "With `keepMounted`, the image is part of the server-rendered HTML and starts loading before "
                "hydration. So is the fallback, which stays visible until hydration resolves the loading status. "
                "A cached image is displayed immediately, without an enter animation."
            </p>

            <h2>"API reference"</h2>

            <h3>"Root"</h3>
            {api_part(
                "Displays a user's profile picture, initials, or fallback icon. Renders a <span> element.",
                "Props: className (string | ((state: Avatar.Root.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSProperties | ((state: Avatar.Root.State) => React.CSSProperties | undefined) — style applied to the element, or a function that returns a style object based on the component's state), render (ReactElement | ((props: HTMLProps, state: Avatar.Root.State) => ReactElement) — allows you to replace the component's HTML element with a different tag, or compose it with another component; accepts a ReactElement or a function that returns the element to render).",
                "Data attributes: none — this part maps its state member imageLoadingStatus to null (avatarStateAttributesMapping), so no generic status attribute ever reaches the DOM.",
                "State: type AvatarRootState = { imageLoadingStatus: ImageLoadingStatus } — the image loading status.",
            )}
            <h3>"Image"</h3>
            {api_part(
                "The image to be displayed in the avatar. Renders an <img> element.",
                "Props: onLoadingStatusChange (((status: ImageLoadingStatus) => void) — callback fired when the loading status changes), className (string | ((state: Avatar.Image.State) => string | undefined)), style (React.CSSProperties | ((state: Avatar.Image.State) => React.CSSProperties | undefined)), keepMounted (boolean, false — whether the image element stays mounted and loads in place instead of being preloaded; supports loading=\"lazy\" and optimized image components such as next/image), render (ReactElement | ((props: img props, state: Avatar.Image.State) => ReactElement)).",
                "Data attributes: data-error (present when the image failed to load), data-loading (present while the image is loading), data-starting-style (present when the image begins animating in), data-ending-style (present when the image is animating out).",
                "State: type AvatarImageState = { transitionStatus: TransitionStatus; imageLoadingStatus: ImageLoadingStatus }.",
            )}
            <h3>"Fallback"</h3>
            {api_part(
                "Rendered when the image fails to load or when no image is provided. Renders a <span> element.",
                "Props: delay (number, 0 — how long to wait before showing the fallback, specified in milliseconds), className (string | ((state: Avatar.Fallback.State) => string | undefined)), style (React.CSSProperties | ((state: Avatar.Fallback.State) => React.CSSProperties | undefined)), render (ReactElement | ((props: HTMLProps, state: Avatar.Fallback.State) => ReactElement)).",
                "Data attributes: none — the fallback shares Root's state mapping (imageLoadingStatus to null); the fallback's presence itself carries the not-loaded state.",
                "State: type AvatarFallbackState = { imageLoadingStatus: ImageLoadingStatus }.",
            )}

            <h2>"Additional types"</h2>
            <p>
                "ImageLoadingStatus: type ImageLoadingStatus = 'idle' | 'loading' | 'loaded' | 'error';"
            </p>
        </article>
    }
}

// ---------------------------------------------------------------------------
// The page's snippets teach the PORT (`specs/docs-content/CONTRACT.md` req 1)
// ---------------------------------------------------------------------------
//
// Same guard as the checkbox/button/accordion/field/fieldset/form/meter pages: this page's Anatomy
// block and its second example were upstream's React fences (the `import { Avatar } from
// '@base-ui/react/avatar'` line and the `next/image` composition) while every structural gate
// passed — a page can render perfectly and still teach another framework. The classifier assertions
// read the same rules the gap report's browser probe reports (`react > 0` is the P0); the `_shape`
// functions compile the compositions the snippets teach (never called — the compiler is the
// assertion), so a snippet naming a prop or path this port does not expose cannot ship as
// documentation.
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};
    use leptos_ui::Avatar;

    /// Upstream's Anatomy block (`page.mdx:17-24`) as the classifier's positive control, so the
    /// assertions below cannot pass vacuously if the classifier stops recognising upstream's TSX.
    /// The package specifier is left out on purpose: this is page SOURCE, not reader-facing content,
    /// and `check-react-mentions.mjs --source` reads this file.
    const UPSTREAM_ANATOMY: &str = "<Avatar.Root>\n  <Avatar.Image src=\"\" />\n  <Avatar.Fallback>LT</Avatar.Fallback>\n</Avatar.Root>;";

    /// Upstream's second example (`page.mdx:32-39`), same rule.
    const UPSTREAM_KEEP_MOUNTED: &str = "<Avatar.Root>\n  <Avatar.Fallback>LT</Avatar.Fallback>\n  <Avatar.Image keepMounted render={<Image src=\"/avatar.png\" width={32} height={32} alt=\"\" />} />\n</Avatar.Root>;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        for (name, upstream) in [
            ("Anatomy", UPSTREAM_ANATOMY),
            ("Using keepMounted", UPSTREAM_KEEP_MOUNTED),
        ] {
            assert_eq!(
                classify(upstream),
                SnippetLanguage::React,
                "the classifier no longer recognises upstream's '{name}' source shape — the \
                 page-snippet assertions would be vacuous"
            );
        }
    }

    /// Every snippet this page embeds, in document order, with the language the contract requires.
    /// `Stacking` is a CSS rule: `CONTRACT.md` requirement 1 states that a language-neutral block
    /// (a shell command, a file tree, a CSS rule) is `other` and is fine, so it is pinned as `other`
    /// rather than smuggled into the Leptos count.
    fn page_snippets() -> [(&'static str, &'static str, SnippetLanguage); 3] {
        [
            ("Anatomy", ANATOMY_SNIPPET, SnippetLanguage::Leptos),
            (
                "Using keepMounted",
                KEEP_MOUNTED_SNIPPET,
                SnippetLanguage::Leptos,
            ),
            (
                "Stacked image and fallback",
                STACKING_SNIPPET,
                SnippetLanguage::Other,
            ),
        ]
    }

    /// The page-level number the item's own done-when names: `{total: 3, leptos: 2, react: 0,
    /// other: 1}` — the same triple `visual-gap-report.mjs`'s in-browser probe reports. Pinned as one
    /// ordered comparison so a re-ordering or a re-classified block fails with both sides visible.
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
                ("Using keepMounted", SnippetLanguage::Leptos),
                ("Stacked image and fallback", SnippetLanguage::Other),
            ],
            "the probe must read {{total: 3, leptos: 2, react: 0, other: 1}} for this page, in document \
             order"
        );
    }

    /// `CONTRACT.md` requirement 1's mapping table: upstream's `Avatar.Root` is this port's
    /// `<Avatar::Root>`, not a flattened `<AvatarRoot>`. The gap report's AST layer counts the
    /// dotted spelling, so it is asserted here rather than left to a browser probe. Only the Leptos
    /// blocks are checked for part tags — the `Stacking` block is a CSS rule and carries none.
    #[test]
    fn every_snippet_uses_the_namespaced_spelling() {
        for (name, text, language) in page_snippets() {
            if language != SnippetLanguage::Leptos {
                continue;
            }
            for tag in ["Root", "Image", "Fallback"] {
                assert!(
                    text.contains(&format!("<Avatar::{tag}")),
                    "the '{name}' snippet does not use the namespaced <Avatar::{tag}> spelling"
                );
            }
            for flattened in ["<AvatarRoot", "<AvatarImage", "<AvatarFallback"] {
                assert!(
                    !text.contains(flattened),
                    "the '{name}' snippet still spells {flattened}> (the flattened form is not the \
                     teaching surface — CONTRACT.md requirement 1)"
                );
            }
        }
    }

    /// The Anatomy snippet's composition, verbatim.
    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        view! {
            <Avatar::Root>
                <Avatar::Image src="/avatar.png".to_string() />
                <Avatar::Fallback inner_html="LT".to_string() />
            </Avatar::Root>
        }
    }

    /// The `Using keepMounted` snippet's composition, verbatim.
    #[allow(dead_code)]
    fn keep_mounted_snippet_shape() -> impl IntoView {
        view! {
            <Avatar::Root>
                <Avatar::Fallback inner_html="LT".to_string() />
                <Avatar::Image
                    keep_mounted=true
                    src="/avatar.png".to_string()
                    element_attributes=vec![
                        ("width".to_string(), "32".to_string()),
                        ("height".to_string(), "32".to_string()),
                        ("alt".to_string(), "".to_string()),
                    ]
                />
            </Avatar::Root>
        }
    }

    #[test]
    fn the_snippets_compile_against_the_ports_surface() {
        let _ = (anatomy_snippet_shape, keep_mounted_snippet_shape);
    }
}
