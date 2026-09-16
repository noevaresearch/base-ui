//! The docs page for `Button`, mirroring
//! `docs/src/app/(docs)/react/components/button/page.mdx`
//! (`specs/docs-content/button/page.md`) — the `docs-content: components/button`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Button` h1,
//! `<Subtitle>` ("A button component that can be rendered as another tag or
//! focusable when disabled."), the hero demo before the first heading,
//! `## Usage guidelines` (two bullets), `## Anatomy` with the single fenced
//! snippet, `## Examples` over "Rendering as another tag" (inline snippet,
//! upstream's `nativeButton={false}`), "Rendering links as buttons" (prose), and
//! "Loading states" (`./demos/loading`), and `## API reference` over the
//! generated `TypesButton` reference
//! (`docs/src/app/(docs)/react/components/button/types.md`) rendered through
//! the ported reference primitives (`crate::reference`) — upstream's own
//! section shape: the `Prop | Type | Default` header row, five `<details>`
//! prop rows carrying their anchors, short types and defaults, and the
//! generated data-attributes table. The port has no docs generator, so the
//! generated content is carried as data, never fabricated as executable
//! machinery.
//!
//! Both of the page's embedded snippets are TRANSLATED to the port (upstream's
//! `.mdx` ships them as JSX against `@base-ui/react/button`) and both are
//! compiled and language-checked by the `snippet_language_guard` module at the
//! foot of this file — `specs/docs-content/CONTRACT.md` requirement 1, whose
//! per-example obligations are tabulated in
//! `specs/docs-content/button/page.md` § Snippet & behaviour contract.
//!
//! Page furniture mirrored in module docs (the separator page precedent): the
//! `<Meta name="description">` content — "A high-quality, unstyled React button
//! component that can be rendered as another tag or focusable when disabled."
//! — and the trailing `export const metadata` SEO keywords block (13 keywords:
//! 'React Button', 'Button Component', 'Focusable Disabled Button', 'Custom
//! Element Button', 'Clickable Element', 'Action Button', 'Submit Button',
//! 'Accessible Button', 'Loading State Button', 'Button as Link', 'Link
//! Button', 'Headless React Components', 'Base UI').
//!
//! The two live demos are the upstream Tailwind demos
//! (`docs/src/app/(docs)/react/components/button/demos/{hero,loading}/tailwind/index.tsx`,
//! the two `specs/docs-content/button/demos.json` entries) ported onto the REAL
//! `leptos_ui::button_element` port. The hero is fully static (demos.json:
//! `stateManaged: "none"` — a native `<button>` labeled "Submit" with only
//! `className` exercised), so it materializes once with no reactive machinery:
//! the element the port describes is exactly the React demo's DOM, including
//! the upstream class string verbatim and the `type="button"` default the
//! render engine forces (the demo's contract from behavior.md "DOM
//! structure": a native `<button>` with the form-submit default overridden).
//! The loading demo is the "controlled — demo-level useState" entry: the
//! demo's `loading` state wraps the `disabled` prop and derives the label
//! text (demos.json `stateManaged`), with `focusableWhenDisabled` keeping the
//! disabled button keyboard-focusable and `aria-labelledby` (the real
//! `useBaseUiId` id) pointing at the inner label span — the page's
//! loading-state guidance made live. Leptos runs the body once, so the React
//! re-render analog is the merge-props-page's dynamic-child rebuild: the
//! tracked view-tree closure re-runs on each `loading` flip and
//! `RawElementView::rebuild` REPLACES the materialized button in place
//! (hand-rolled `reactive_graph::effect::Effect`s lost their first-run
//! subscription in the wasm suite — the view-tree closure tracks reliably).
//! The 4-second re-enable rides the ported dual-target `TimeoutManager` (the
//! AGENTS.md rule: never raw `window.setTimeout`), and the demo's consumer
//! `onClick` rides the real merged handler bag — it fires only while enabled
//! (the internal disabled guard runs before the consumer's handler;
//! behavior.md "Events", the later-bag-runs-first merge rule).

use std::rc::Rc;

use leptos::prelude::*;

use leptos_ui::button_element;
use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::timeout_manager::TimeoutManager;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderedElement, UseRenderElementComponentProps,
};

use crate::pages::use_render_page::RawElementView;
use crate::reference::{self, DataAttributeRow, ReferenceProp};

/// The upstream demo `Button` `className` — shared verbatim by both demos
/// (`hero/tailwind/index.tsx:5`, `loading/tailwind/index.tsx:9`).
const DEMO_BUTTON_CLASS: &str = "flex h-8 items-center justify-center gap-2 rounded-none border border-neutral-950 bg-white px-3 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 active:not-data-disabled:bg-neutral-200 focus-visible:outline-2 focus-visible:-outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white data-disabled:border-neutral-500 data-disabled:text-neutral-500 disabled:border-neutral-500 disabled:text-neutral-500 dark:border-white dark:bg-neutral-950 dark:text-white dark:hover:not-data-disabled:bg-neutral-800 dark:active:not-data-disabled:bg-neutral-700 dark:data-disabled:border-neutral-400 dark:data-disabled:text-neutral-400";

/// The page's embedded snippets, TRANSLATED to the port (`specs/docs-content/CONTRACT.md`
/// requirement 1). The upstream `.mdx` ships both blocks as JSX against `@base-ui/react/button`,
/// which a mirrored page must not teach.
///
/// The port's surface for this component is an element DESCRIPTION, not a `#[component]`:
/// `leptos_ui::button_element` (`crates/leptos-ui/src/button.rs:155`) answers with a
/// `RenderedElement` that the caller materializes through `RenderedElement::create_element()`
/// (`crates/leptos-ui-internals/src/use_render_element.rs:536`) — the same two-step the crate's own
/// button tests and this page's demos use. The snippets show that idiom, and the
/// `snippet_language_guard` module at the foot of this file compiles each one, so a snippet cannot
/// name a prop, field or path the port lacks.
///
/// The `## Anatomy` snippet (`page.mdx:24-28`) — "import the component". Upstream's bare
/// `<Button />;` becomes the build-then-materialize pair, because there is no component wrapper to
/// mount; `specs/docs-content/button/page.md`'s contract table is the citation for what each block
/// must show.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{ButtonProps, button_element};

// @highlight-text "button_element"
// Button is an element description: build it, then materialize it.
let rendered = button_element(ButtonProps::default())
    .expect("Button always renders (a leaf with no enabled gate)");

let (element, cleanup) = rendered.create_element();
// Append `element` where it belongs; the listeners live until `cleanup` drops."#;

/// The "Rendering as another tag" snippet (`page.mdx:36-43`, upstream directive
/// `@highlight-text "nativeButton={false}"` on `:39`) — upstream's
/// `<Button render={<div />} nativeButton={false}>`. Translated: `native_button` is the port's
/// snake_case prop, and `render={<div />}` is the port's element-form `render` prop —
/// `RenderProp::Element { tag: "div", .. }`, the same shape the checkbox page's native-button
/// snippet teaches (contract row "Rendering as another tag").
const CUSTOM_TAG_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{ButtonProps, button_element};
use leptos_ui_internals::use_render_element::{
    RenderElementProps, RenderProp, UseRenderElementComponentProps,
};

// @highlight-text "native_button" "render"
let rendered = button_element(ButtonProps {
    // The rendered tag is not a <button>, so the engine supplies button
    // semantics (role="button", tabindex="0", Enter/Space activation).
    native_button: false,
    render_class_style: UseRenderElementComponentProps {
        render: Some(RenderProp::Element {
            tag: "div".into(),
            props: RenderElementProps::default(),
        }),
        ..UseRenderElementComponentProps::default()
    },
    ..ButtonProps::default()
})
.expect("Button always renders");

rendered.props.inner_html = Some("Button that can contain complex children".to_string());"#;

/// Builds one `button_element` description — the shared spine of both demos:
/// the upstream class string, `focusableWhenDisabled` (set by the loading
/// demo; the hero passes neither `disabled` nor `focusableWhenDisabled`), the
/// optional `aria-labelledby` override into the elementProps rest, the
/// optional consumer `onClick` (the loading demo's), and the label span as
/// the button's content. The consumer handler enters through
/// `ButtonProps.handlers` — the real rest-bag consumer slot — so the internal
/// disabled guard composes around it inside `use_button` exactly as upstream
/// composes around the consumer's handler.
fn demo_button_element(
    disabled: bool,
    focusable_when_disabled: bool,
    label: &str,
    label_id: Option<&str>,
    on_click: Option<ElementEventHandler<leptos::web_sys::MouseEvent>>,
) -> RenderedElement {
    let mut rendered = button_element(leptos_ui::ButtonProps {
        disabled,
        focusable_when_disabled,
        native_button: true,
        render_class_style: UseRenderElementComponentProps {
            class_name: Some(ClassNameSource::Static(DEMO_BUTTON_CLASS.to_string())),
            render: None,
            style: None,
        },
        // `aria-labelledby={labelId}` (loading/tailwind/index.tsx:17) — the
        // rest-bag attribute override (the earlier-bag-wins rule). The hero
        // passes none (it carries only className).
        element_attributes: label_id
            .map(|id| vec![("aria-labelledby".to_string(), id.to_string())])
            .unwrap_or_default(),
        // The consumer's `onClick` (`loading/tailwind/index.tsx:19-25`) —
        // `ButtonHandlers.on_click` is the wrapped rest-bag slot.
        handlers: leptos_ui::ButtonHandlers {
            on_click,
            ..leptos_ui::ButtonHandlers::default()
        },
    })
    .expect("the demo Button renders (a leaf with no enabled gate)");

    // The demo's label content: the hero's bare-text child ("Submit",
    // `hero/tailwind/index.tsx:7`) or the loading demo's
    // `<span id={labelId}>{loading ? 'Submitting' : 'Submit'}</span>`
    // (`loading/tailwind/index.tsx:22-24`). `inner_html` is the bag's
    // content slot at materialization (both label strings are static
    // literals — no escaping concern).
    rendered.props.inner_html = Some(match label_id {
        Some(id) => format!("<span id=\"{id}\">{label}</span>"),
        None => label.to_string(),
    });

    rendered
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): a
/// native `<button>` labeled "Submit" with only `className` exercised
/// (demos.json `propsExercised.Button: ["className"]`,
/// `stateManaged: "none"`). Fully static — one materialization inside its own
/// owner (the toggle-page seed: the build clobbers the thread-local owner,
/// and `Owner::with` saves/restores the surrounding one), no reactive
/// machinery.
pub fn button_hero_demo() -> RawElementView {
    let container = document().create_element("span").unwrap();
    container.set_attribute("data-button-hero", "").unwrap();

    let seed_owner = reactive_graph::owner::Owner::new();
    seed_owner.with(|| {
        let rendered = demo_button_element(false, false, "Submit", None, None);
        let (element, cleanup) = rendered.create_element();
        let _ = container.append_child(&element);
        std::mem::forget(cleanup);
    });
    // The seed's reactive scope stays alive for the page's lifetime (the
    // direction-provider owner-bridge precedent).
    std::mem::forget(seed_owner);

    RawElementView { element: container }
}

/// The React re-render analog — the merge-props demo's exact mechanism,
/// proven in this harness (its wasm test passes at this tree): the seed
/// materialization is the initial render, and an `Effect::new` created under
/// the component's real reactive owner re-runs the build on each `loading`
/// flip, REPLACING the container's child in place. The effect's FIRST act is
/// the tracked read — `.get()`, never `get_untracked()`: an untracked read
/// subscribes to nothing and the rebuild would never fire.
///
/// THE ROOT CAUSE this demo's history pins (recorded for the audit loop):
/// every earlier failure — the prior session's hand-rolled Effect, the
/// dynamic-view-child rewrite, and the first Effect-based attempts here —
/// shared one defect: the demo's reactive state was created through the
/// DIRECT `reactive_graph` dependency (rg-0.2, the internals crate's
/// runtime) while the `Effect::new` and the mount owner belong to leptos's
/// runtime (rg-0.1 — the workspace holds BOTH, Cargo.lock reactive_graph
/// 0.1.8 + 0.2.14; the meter b9b107c12 cross-crate runtime split, resurfaced
/// in docs-app). A signal on one runtime is invisible to effects on the
/// other: the read warns "outside a reactive tracking context" (from the
/// signal's runtime's perspective), subscribes to nothing, and `set` wakes
/// no one — while the whole rg-0.1 machinery (mount owner, view tree, text
/// closures) keeps working, which is why the harness characterization tests
/// passed beside the failing demo. The rule: a docs-app page's reactive
/// state and effects ride the SAME leptos runtime the view tree mounts
/// under; the direct rg-0.2 dependency exists for the internals crate's own
/// machinery, not for page state.
fn demo_build(
    loading: RwSignal<bool>,
    timeouts: &send_wrapper::SendWrapper<TimeoutManager>,
    label_id: &str,
    reset_ms: i32,
) -> RenderedElement {
    let is_loading = loading.get_untracked();
    // The demo's consumer onClick (`:19-25`): setLoading(true) then the
    // reset. It enters through ButtonProps.handlers.on_click (the real
    // rest-bag consumer slot), so it fires only while enabled.
    let on_click: Option<ElementEventHandler<leptos::web_sys::MouseEvent>> = (!is_loading).then({
        let loading_mirror = loading;
        let timeouts_for_click = timeouts.clone();
        move || {
            Rc::new(move |_event: &leptos::web_sys::MouseEvent| {
                loading_mirror.set(true);
                timeouts_for_click.start("button-loading-demo-reset", reset_ms, {
                    let loading_mirror = loading_mirror.clone();
                    move || loading_mirror.set(false)
                });
            }) as ElementEventHandler<leptos::web_sys::MouseEvent>
        }
    });
    demo_button_element(
        is_loading,
        true,
        if is_loading { "Submitting" } else { "Submit" },
        Some(label_id),
        on_click,
    )
}

/// The loading demo (`demos/loading/tailwind/index.tsx`, demos.json entry 2):
/// a Button that enters a loading state on click — disabled yet still
/// keyboard-focusable (`focusableWhenDisabled`), its label swapping from
/// "Submit" to "Submitting", re-enabled after `reset_ms` milliseconds
/// (`setTimeout(…, 4000)` through the ported `TimeoutManager`). The demo's
/// `loading` useState is the mirror signal. The consumer's `onClick` rides
/// the real merged handler bag and fires only while enabled — the internal
/// disabled guard runs before the consumer's handler (behavior.md
/// "Events").
/// The component-tag form of the loading demo (what the page mounts).
#[component]
pub fn ButtonLoadingDemo(reset_ms: i32) -> impl IntoView {
    button_loading_demo_with(reset_ms)
}

pub fn button_loading_demo_with(reset_ms: i32) -> RawElementView {
    // The demo's `labelId = React.useId()` — the real ported id generator
    // (the `base-ui-…` prefixed useId wrapper), created once at construction:
    // the id is stable across rebuilds exactly as the React demo's useId is.
    // The rg-0.2 `GetUntracked` is imported scoped: the wrapper's return type
    // is an rg-0.2 `Signal`, whose `.get_untracked` is the rg-0.2 trait's —
    // the page's own rg-0.1 signals use the leptos-prelude trait, and a
    // top-level import of both same-named traits muddies method resolution.
    let label_id = {
        use reactive_graph::traits::GetUntracked as _;
        use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>)).get_untracked()
    };

    // The demo's `loading` state — the mirror the React demo's useState
    // re-renders on. RwSignal from `leptos::prelude` — the runtime the view
    // tree mounts under and the only runtime whose effects this demo's
    // rebuild subscribes to (the two-runtimes trap in `demo_build`'s docs).
    let loading = RwSignal::new(false);
    // The demo's `setTimeout(…, 4000)` reset — the ported dual-target
    // TimeoutManager (the AGENTS.md rule: never raw window.setTimeout).
    // `start` replaces any pending reset under the same key, so a re-click
    // cannot stack resets (the TimeoutManager.start contract). SendWrapper
    // because the Effect's closure must be Send while the registry is an
    // Rc<RefCell<…>> — the wasm target is single-threaded, so the wrapper
    // is the honest adapter (the use-render page's SendWrapper'd cleanup
    // precedent).
    let timeouts = send_wrapper::SendWrapper::new(TimeoutManager::default());

    let container = document().create_element("span").unwrap();
    container.set_attribute("data-button-loading", "").unwrap();

    let build = move || demo_build(loading, &timeouts, &label_id, reset_ms);

    // The initial build runs synchronously at construction — the React
    // demo's first render happens before the event loop turns. The
    // build+materialize chain runs inside its OWN leptos-runtime owner
    // (`Owner::with` saves and restores the surrounding one — the
    // merge-props/direction-provider seed convention), so the seed's
    // internals-crate reads resolve without disturbing the mount's owner.
    let seed_owner = Owner::new();
    seed_owner.with(|| {
        let rendered = build();
        let (element, cleanup) = rendered.create_element();
        let _ = container.append_child(&element);
        std::mem::forget(cleanup);
    });
    // The seed's reactive scope stays alive for the page's lifetime (the
    // direction-provider owner-bridge precedent).
    std::mem::forget(seed_owner);

    // The rebuild Effect: tracked read first (the subscription), then the
    // build, then replace the previous materialization wholesale. The
    // listener cleanup is forgotten per materialization — the live element
    // must keep its handlers for the page's lifetime (`create_element`'s
    // own hold-or-forget contract), and the replaced node leaves its
    // listeners behind with it.
    let build_container = container.clone();
    let loading_for_effect = loading;
    Effect::new(move |_| {
        // Tracked read — deliberately `.get()`, not `get_untracked()`:
        // an untracked read subscribes to nothing and the rebuild would
        // never fire (the merge-props Effect's rule). `demo_build` reads
        // the value untracked, so this body is the only subscriber.
        let current_loading = loading_for_effect.get();
        let rendered = build();
        let container = &build_container;
        while let Some(child) = container.first_child() {
            let _ = container.remove_child(&child);
        }
        let (element, cleanup) = rendered.create_element();
        let _ = container.append_child(&element);
        std::mem::forget(cleanup);
        // Keep the tracked read alive past the early paths above.
        let _ = current_loading;
    });

    RawElementView { element: container }
}

// ---------------------------------------------------------------------------
// The generated `## API reference` content.
//
// Source: `docs/src/app/(docs)/react/components/button/types.md` (3.1 kB, generated by
// `pnpm docs:api` from the component's type definitions), which is what upstream's page renders
// through `<TypesButton />` (`page.mdx:59-63`). The port has no docs generator, so the generated
// content is carried here as data — the same approach the page's snippets take — and rendered by
// `crate::reference`, whose element shape and class names are upstream's.
//
// The descriptions' inline code spans are transcribed where the generated table has them
// (upstream keeps them in the DOM, and the fidelity gate's `codeBlocks` recall term counts them),
// the type strings keep upstream's rendered multi-line union form, and each row's summary carries
// the SHORT type label upstream shows there (`boolean`, `string | function`) — all read off
// upstream's own render of this route at 1280px (2026-09-16) rather than derived.
//
// The `### Button.Props` / `### Button.State` additional-type panels upstream renders after the
// tables are NOT here: they sit behind `display: none` until a row's type is clicked, and their
// body is the generated TypeScript type block, which `docs-chrome: code blocks` owns (see
// `ralph/logs/spec-discrepancies.md`).
// ---------------------------------------------------------------------------

/// `types.md:14-20` — the generated `**Button Props:**` rows, in order, with each prop's
/// `#Button-<name>` anchor as upstream renders it and the short type its summary shows.
const BUTTON_PROPS: &[ReferenceProp] = &[
    ReferenceProp {
        name: "focusableWhenDisabled",
        anchor: "Button-focusableWhenDisabled",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("false"),
        description: &[reference::text(
            "Whether the button should be focusable when disabled.",
        )],
    },
    ReferenceProp {
        name: "nativeButton",
        anchor: "Button-nativeButton",
        short_ty: "boolean",
        ty: "boolean | undefined",
        default_value: Some("true"),
        description: &[
            reference::text("Whether the component renders a native "),
            reference::code("<button>"),
            reference::text(" element when replacing it\nvia the "),
            reference::code("render"),
            reference::text(" prop.\nSet to "),
            reference::code("false"),
            reference::text(" if the rendered element is not a button (for example, "),
            reference::code("<div>"),
            reference::text(")."),
        ],
    },
    ReferenceProp {
        name: "className",
        anchor: "Button-className",
        short_ty: "string | function",
        ty: "| string\n| ((state: Button.State) => string | undefined)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "CSS class applied to the element, or a function that\nreturns a class based on the component\u{2019}s state.",
        )],
    },
    ReferenceProp {
        name: "style",
        anchor: "Button-style",
        short_ty: "React.CSSProperties | function",
        ty: "| React.CSSProperties\n| ((\n    state: Button.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
        default_value: None,
        description: &[reference::text(
            "Style applied to the element, or a function that\nreturns a style object based on the component\u{2019}s state.",
        )],
    },
    ReferenceProp {
        name: "render",
        anchor: "Button-render",
        short_ty: "ReactElement | function",
        ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Button.State,\n  ) => ReactElement)\n| undefined",
        default_value: None,
        description: &[
            reference::text(
                "Allows you to replace the component\u{2019}s HTML element\nwith a different tag, or compose it with another component.\nAccepts a ",
            ),
            reference::code("ReactElement"),
            reference::text(" or a function that returns the element to render."),
        ],
    },
];

/// `types.md:22-26` — the generated `**Button Data Attributes:**` table's single row.
const BUTTON_DATA_ATTRIBUTES: &[DataAttributeRow] = &[DataAttributeRow {
    name: "data-disabled",
    description: &[reference::text("Present when the button is disabled.")],
}];

/// The `docs/src/app/(docs)/react/components/button/page.mdx` page.
#[component]
pub fn ButtonPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Button"</h1>
            <p class="subtitle">"A button component that can be rendered as another tag or focusable when disabled."</p>

            <div class="docs-demo" data-demo="hero">{button_hero_demo()}</div>

            <h2>"Usage guidelines"</h2>
            <ul>
                <li>
                    <strong>"Submit buttons"</strong>
                    ": Unlike the native button element, `type=\"submit\"` must be specified on Button for it to act as a submit button."
                </li>
                <li>
                    <strong>"Links"</strong>
                    ": The Button component enforces button semantics (`role=\"button\"`, keyboard interaction, disabled state). It should not be used for links. See "
                    <a href="#rendering-links-as-buttons">"Rendering links as buttons"</a>
                    " below."
                </li>
            </ul>

            <h2>"Anatomy"</h2>
            <p>"Import the component:"</p>
            <pre><code>{ANATOMY_SNIPPET}</code></pre>

            <h2>"Examples"</h2>
            <h3>"Rendering as another tag"</h3>
            <p>
                "The button can remain keyboard accessible while being rendered as another tag, "
                "such as a `<div>`, by specifying `nativeButton={false}`."
            </p>
            <pre><code>{CUSTOM_TAG_SNIPPET}</code></pre>

            <h3>"Rendering links as buttons"</h3>
            <p>
                "The Button component enforces button semantics. `nativeButton={false}` signals "
                "that the rendered tag is not a `<button>`, but it must still be a tag that can "
                "receive button semantics (`role=\"button\"`, keyboard interaction handlers). "
                "Links (`<a>`) have their own semantics and should not be rendered as buttons "
                "through the `render` prop."
            </p>
            <p>
                "If a link needs to look like a button visually, style the `<a>` element "
                "directly with CSS rather than using the Button component."
            </p>

            <h3>"Loading states"</h3>
            <p>
                "For buttons that enter a loading state after activation, specify "
                "`focusableWhenDisabled` so focus remains on the button while it is disabled. "
                "Because some browser and screen reader combinations do not reliably announce "
                "changes to a focused button's descendant text, use "
                <a href="https://www.w3.org/TR/accname-1.2/#computation-steps">"aria-labelledby"</a>
                " to make the changing text the button's explicit accessible name."
            </p>
            <div class="docs-demo" data-demo="loading">
                <ButtonLoadingDemo reset_ms=4000 />
            </div>

            <h2>"API reference"</h2>
            {reference::props_section(BUTTON_PROPS, "button-props-table")}
            {reference::data_attributes_table(BUTTON_DATA_ATTRIBUTES)}

        </article>
    }
}

/// Browser-free guard for `specs/docs-content/CONTRACT.md` requirement 1: both snippets embedded in
/// this page demonstrate the PORT's API.
///
/// Why it exists: this page sat in the repo marked `done` while both of its code blocks carried
/// upstream's React source (`import { Button } from '@base-ui/react/button'` plus JSX), and every
/// structural gate stayed green — `playwright-diff.mjs` and `check-visual-budget.mjs` watch a page's
/// shape and looks, not which framework it teaches. The snippet-language probe that does
/// (`ralph/scripts/visual-gap-report.mjs:233-242`) needs BOTH dev servers up and reports a NOTE and
/// passes when the upstream reference is down, and because transcribed snippet text counted toward
/// content recall, leaving the JSX there *raised* the fidelity score. This module is the cheap half
/// of that obligation: it runs in the ordinary host suite (`cargo test -p docs-app --lib`) and fails
/// the moment a snippet teaches React again.
///
/// Deliberately two-part, matching the checkbox page's guard:
///   * `the_pages_snippets_all_teach_the_port` classifies each constant with the same rules as the
///     probe (mirrored in `crate::snippet_language`, shared by every mirrored page), so the numbers
///     the probe would report are asserted in CI: `{total: 2, leptos: 2, react: 0}`;
///   * the `_shape` functions below compile the composition each snippet teaches, so a snippet
///     cannot name a prop, field or path the port does not actually have. They are never called
///     (the page's real compositions are exercised by `render_test.rs`); the compiler is the
///     assertion.
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{looks_leptos, looks_react};
    use leptos_ui::{ButtonProps, button_element};
    use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

    /// The upstream `## Anatomy` block (`page.mdx:24-28`), kept as the classifier's positive
    /// control: if `looks_react` ever stops recognising upstream's source, the assertions below
    /// would pass vacuously, and this test would say so instead.
    const UPSTREAM_ANATOMY: &str = "import { Button } from '@base-ui/react/button';\n\n<Button />;";

    /// The upstream "Rendering as another tag" block (`page.mdx:36-43`) — the second positive
    /// control, with upstream's `@highlight-text` directive in place.
    const UPSTREAM_CUSTOM_TAG: &str = "import { Button } from '@base-ui/react/button';\n\n// @highlight-text \"nativeButton={false}\"\n<Button render={<div />} nativeButton={false}>\n  Button that can contain complex children\n</Button>;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert!(
            looks_react(UPSTREAM_ANATOMY) && !looks_leptos(UPSTREAM_ANATOMY),
            "the classifier no longer recognises upstream's React source — the assertions below \
             would be vacuous"
        );
        assert!(
            looks_react(UPSTREAM_CUSTOM_TAG) && !looks_leptos(UPSTREAM_CUSTOM_TAG),
            "the classifier no longer recognises upstream's React source — the assertions below \
             would be vacuous"
        );
    }

    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let snippets = [
            ("Anatomy", ANATOMY_SNIPPET),
            ("Custom tag button", CUSTOM_TAG_SNIPPET),
        ];
        let (mut leptos, mut react, mut other) = (0, 0, 0);
        for (name, text) in snippets {
            match (looks_leptos(text), looks_react(text)) {
                (true, false) => leptos += 1,
                (_, true) => {
                    react += 1;
                    panic!("the '{name}' snippet still carries React source");
                }
                _ => {
                    other += 1;
                    panic!("the '{name}' snippet identifies as neither port nor React source");
                }
            }
        }
        assert_eq!(
            (leptos, react, other),
            (2, 0, 0),
            "the probe must read {{total: 2, leptos: 2, react: 0}} for this page"
        );
    }

    // --- the snippets' shapes, compiled ------------------------------------------------------
    // Each mirrors its snippet's composition verbatim (imports included, at the top of this
    // module). Never called: the compiler checks the props, fields and paths the page teaches.

    #[allow(dead_code)]
    fn anatomy_snippet_shape() {
        let rendered = button_element(ButtonProps::default())
            .expect("Button always renders (a leaf with no enabled gate)");
        let (_element, _cleanup) = rendered.create_element();
    }

    #[allow(dead_code)]
    fn custom_tag_snippet_shape() {
        let mut rendered = button_element(ButtonProps {
            native_button: false,
            render_class_style: UseRenderElementComponentProps {
                render: Some(RenderProp::Element {
                    tag: "div".into(),
                    props: RenderElementProps::default(),
                }),
                ..UseRenderElementComponentProps::default()
            },
            ..ButtonProps::default()
        })
        .expect("Button always renders");
        rendered.props.inner_html = Some("Button that can contain complex children".to_string());
        let (_element, _cleanup) = rendered.create_element();
    }
}

/// Browser-free drift guard for the generated `## API reference` content this page carries as data.
///
/// Why it exists: `docs/src/app/(docs)/react/components/button/types.md` is regenerated by
/// `pnpm docs:api`, and the page's own tests (`render_test.rs`) assert the SHAPE of the rendered
/// reference — one table, five prop rows, the section's header cells — which notices neither a
/// mistyped prop name nor a dropped row nor a stale short type. The row sets, their order, their
/// anchors and the documented defaults are asserted here instead, so the hand transcription is
/// checked without a browser and a future regeneration of the documented surface fails loudly
/// rather than silently.
///
/// The expected values are transcribed from the generated file (`types.md:14-26`) and from
/// upstream's own render of this route at 1280px (2026-09-16) for the two columns the generated
/// markdown does not carry verbatim — each row's SHORT summary type, and the em dash upstream
/// shows where the generated table has no default.
#[cfg(test)]
mod reference_content_guard {
    use super::*;

    /// `types.md:16-20` — the generated `**Button Props:**` rows, in order.
    const BUTTON_PROP_NAMES: [&str; 5] = [
        "focusableWhenDisabled",
        "nativeButton",
        "className",
        "style",
        "render",
    ];

    /// Upstream's short summary type per row, in the same order (measured off the live render;
    /// the generated markdown's `Type` column carries the full union, not this label).
    const BUTTON_SHORT_TYPES: [&str; 5] = [
        "boolean",
        "boolean",
        "string | function",
        "React.CSSProperties | function",
        "ReactElement | function",
    ];

    /// `types.md:22-26` — the generated `**Button Data Attributes:**` rows, in order.
    const BUTTON_DATA_ATTRIBUTE_NAMES: [&str; 1] = ["data-disabled"];

    #[test]
    fn the_transcribed_rows_match_the_generated_types_content() {
        assert_eq!(
            BUTTON_PROPS.iter().map(|prop| prop.name).collect::<Vec<_>>(),
            BUTTON_PROP_NAMES.to_vec(),
            "the Button prop rows drifted from types.md"
        );
        assert_eq!(
            BUTTON_PROPS
                .iter()
                .map(|prop| prop.short_ty)
                .collect::<Vec<_>>(),
            BUTTON_SHORT_TYPES.to_vec(),
            "the Button rows' short summary types drifted from upstream's render"
        );
        assert_eq!(
            BUTTON_DATA_ATTRIBUTES
                .iter()
                .map(|row| row.name)
                .collect::<Vec<_>>(),
            BUTTON_DATA_ATTRIBUTE_NAMES.to_vec(),
            "the Button data-attribute rows drifted from types.md"
        );
    }

    /// Every row must be addressed by upstream's own anchor (`Button-<name>`) — which is also what
    /// its `Name` cell links to — and carry the content the reference is for. A missing or
    /// duplicate anchor would silently break the section's in-page links, and upstream's per-prop
    /// links are a recall term the fidelity gate scores.
    #[test]
    fn every_prop_row_is_addressed_and_carries_its_generated_content() {
        let mut anchors = std::collections::BTreeSet::new();
        for prop in BUTTON_PROPS {
            assert_eq!(
                prop.anchor,
                format!("Button-{}", prop.name),
                "prop '{}' does not carry its upstream anchor",
                prop.name
            );
            assert!(
                anchors.insert(prop.anchor),
                "duplicate prop anchor '{}'",
                prop.anchor
            );
            assert!(!prop.ty.is_empty(), "prop '{}' has no type", prop.name);
            assert!(
                !prop.short_ty.is_empty(),
                "prop '{}' has no summary type",
                prop.name
            );
            assert!(
                !prop.description.is_empty(),
                "prop '{}' has no description",
                prop.name
            );
        }
    }

    /// Upstream renders a summary Default cell on EVERY row and a `Default` description item only
    /// where the generated table documents one (`types.md`'s `-` rows show `—` above and nothing
    /// below), so the two sets have to stay complementary.
    #[test]
    fn only_props_with_a_documented_default_carry_a_default_row() {
        assert_eq!(
            BUTTON_PROPS
                .iter()
                .filter(|prop| prop.default_value.is_some())
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            vec!["focusableWhenDisabled", "nativeButton"],
            "the Button props carrying a documented default drifted from types.md"
        );
    }
}
