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
//! `nativeButton={false}`), "Rendering links as buttons" (prose), and
//! "Loading states" (`./demos/loading`), and `## API reference` over the
//! generated `TypesButton` reference
//! (`docs/src/app/(docs)/react/components/button/types.md`) echoed as static
//! prose per the toggle/separator page precedent — the port has no docs
//! generator, so the table's documented props/data-attributes/state type are
//! rendered as text, never fabricated as executable machinery.
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
//! re-render analog is the toggle-page's Effect-driven rebuild: the mirror
//! signal flips → the demo rebuilds through the real `button_element`
//! pipeline → the fresh button mounts disabled (or re-enabled) with the new
//! label. The 4-second re-enable rides the ported dual-target
//! `TimeoutManager` (the AGENTS.md rule: never raw `window.setTimeout`), and
//! the demo's consumer `onClick` rides the real merged handler bag — it fires
//! only while enabled (the internal disabled guard runs before the consumer's
//! handler; behavior.md "Events", the later-bag-runs-first merge rule).

use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};

use leptos_ui::button_element;
use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::timeout_manager::TimeoutManager;
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderedElement, UseRenderElementComponentProps,
};

use crate::pages::use_render_page::RawElementView;

/// The lib code names DOM types through leptos's `web_sys` re-export —
/// docs-app carries web-sys only as a dev-dependency (the toggle-page
/// convention).

/// The upstream demo `Button` `className` — shared verbatim by both demos
/// (`hero/tailwind/index.tsx:5`, `loading/tailwind/index.tsx:9`).
const DEMO_BUTTON_CLASS: &str = "flex h-8 items-center justify-center gap-2 rounded-none border border-neutral-950 bg-white px-3 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 active:not-data-disabled:bg-neutral-200 focus-visible:outline-2 focus-visible:-outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white data-disabled:border-neutral-500 data-disabled:text-neutral-500 disabled:border-neutral-500 disabled:text-neutral-500 dark:border-white dark:bg-neutral-950 dark:text-white dark:hover:not-data-disabled:bg-neutral-800 dark:active:not-data-disabled:bg-neutral-700 dark:data-disabled:border-neutral-400 dark:data-disabled:text-neutral-400";

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

/// Materializes one demo button into the container — the
/// `create_element` + `std::mem::forget(cleanup)` harness convention of the
/// toggle page (the page outlives any single materialization; the listener
/// cleanups ride the forgotten owner chain).
fn mount_rendered(container: &leptos::web_sys::Element, rendered: RenderedElement) {
    let (element, cleanup) = rendered.create_element();
    let _ = container.append_child(&element);
    std::mem::forget(cleanup);
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
        mount_rendered(&container, rendered);
    });
    // The seed's reactive scope stays alive for the page's lifetime (the
    // direction-provider owner-bridge precedent).
    std::mem::forget(seed_owner);

    RawElementView { element: container }
}

/// The loading demo at the upstream reset delay (4000 ms —
/// `loading/tailwind/index.tsx:23`). See [`button_loading_demo_with`] for the
/// machinery; the parameter exists so the wasm suite can exercise the
/// re-enable with a short delay instead of waiting four real seconds (the
/// `toggle_hero_demo_with(pressed_source)` testable-parameter precedent).
pub fn button_loading_demo() -> RawElementView {
    button_loading_demo_with(4000)
}

/// The loading demo (`demos/loading/tailwind/index.tsx`, demos.json entry 2):
/// a Button that enters a loading state on click — disabled yet still
/// keyboard-focusable (`focusableWhenDisabled`), its label swapping from
/// "Submit" to "Submitting", re-enabled after `reset_ms` milliseconds
/// (`setTimeout(…, 4000)` through the ported `TimeoutManager`). The demo's
/// `loading` useState is the mirror signal; the React re-render analog is the
/// toggle-page Effect-driven rebuild through the real `button_element`
/// pipeline. The consumer's `onClick` rides the real merged handler bag and
/// fires only while enabled — the internal disabled guard runs before the
/// consumer's handler (behavior.md "Events").
pub fn button_loading_demo_with(reset_ms: i32) -> RawElementView {
    let container = document().create_element("span").unwrap();
    container.set_attribute("data-button-loading", "").unwrap();

    // The demo's `labelId = React.useId()` — the real ported id generator
    // (the `base-ui-…` prefixed useId wrapper), created once under its own
    // owner so the id is stable across rebuilds exactly as the React demo's
    // useId is.
    let seed_owner = reactive_graph::owner::Owner::new();
    let label_id = seed_owner.with(|| {
        use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>))
            .get_untracked()
    });
    std::mem::forget(seed_owner);

    // The demo's `loading` state — the mirror the React demo's useState
    // re-renders on.
    let loading = RwSignal::new(false);
    // The demo's `setTimeout(…, 4000)` reset — the ported dual-target
    // TimeoutManager (the AGENTS.md rule: never raw window.setTimeout).
    // `start` replaces any pending reset under the same key, so a re-click
    // cannot stack resets (the TimeoutManager.start contract).
    let timeouts = TimeoutManager::default();

    let build = move || {
        let is_loading = loading.get_untracked();
        // The demo's consumer onClick (`:19-25`): setLoading(true) then the
        // 4000 ms reset. It enters through ButtonProps.handlers.on_click (the
        // real rest-bag consumer slot), so it fires only while enabled.
        let on_click: Option<ElementEventHandler<leptos::web_sys::MouseEvent>> = (!is_loading).then({
            let loading_mirror = loading;
            let timeouts_for_click = timeouts.clone();
            move || {
                Rc::new(move |_event: &leptos::web_sys::MouseEvent| {
                    web_sys::console::log_1(
                        &"[button-demo] consumer onClick fired".to_string().into(),
                    );
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
            Some(&label_id),
            on_click,
        )
    };

    // The initial build runs synchronously at creation inside its OWN owner
    // (the toggle-page seed convention).
    let build_owner = reactive_graph::owner::Owner::new();
    build_owner.with(|| {
        let rendered = build();
        mount_rendered(&container, rendered);
    });
    std::mem::forget(build_owner);

    // The React re-render analog: each `loading` flip re-runs the build and
    // replaces the button wholesale (the toggle-page/merge-props-page Effect
    // convention: leptos's executor-backed `Effect::new`, created under the
    // mount owner, tracked read first). The mount owner must stay alive — a
    // dropped mount guard disposes it, the deferred first run lands outside
    // any tracking context, subscribes to nothing, and the rebuild never
    // fires (the wasm suite caught exactly that).
    let build_container = container.clone();
    Effect::new(move |_| {
        // Tracked read — the subscription that re-runs the rebuild on each
        // flip. `build()` reads the value untracked, so the effect body is
        // the only subscriber.
        let value = loading.get();
        web_sys::console::log_1(
            &format!("[button-demo] effect body, read loading={value}").into(),
        );
        let rendered = build();
        let container = &build_container;
        while let Some(child) = container.first_child() {
            let _ = container.remove_child(&child);
        }
        mount_rendered(container, rendered);
    });

    RawElementView { element: container }
}

/// One API-reference block: the generated `TypesButton` table
/// (`docs/src/app/(docs)/react/components/button/types.md`) echoed as static
/// prose — the summary line, the props list, and the data-attributes list.
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
    }
}

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
            <pre><code>
"import { Button } from '@base-ui/react/button';

<Button />;"
            </code></pre>

            <h2>"Examples"</h2>
            <h3>"Rendering as another tag"</h3>
            <p>
                "The button can remain keyboard accessible while being rendered as another tag, "
                "such as a `<div>`, by specifying `nativeButton={false}`."
            </p>
            <pre><code>
"import { Button } from '@base-ui/react/button';

<Button render={<div />} nativeButton={false}>
  Button that can contain complex children
</Button>;"
            </code></pre>

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
            <div class="docs-demo" data-demo="loading">{button_loading_demo()}</div>

            <h2>"API reference"</h2>
            {api_part(
                "A button component that can be used to trigger actions. Renders a <button> element.",
                "Props: disabled (boolean, false — whether the button should ignore user interaction; the custom-element path signals it with aria-disabled instead), focusableWhenDisabled (boolean, false — whether the button should be focusable when disabled), nativeButton (boolean, true — whether the component renders a native <button> element when replacing it via the render prop; set to false if the rendered element is not a button, for example <div>), className (string | ((state: Button.State) => string | undefined)), style (React.CSSProperties | ((state: Button.State) => React.CSSProperties | undefined)), render (ReactElement | ((props: HTMLProps, state: Button.State) => ReactElement) — replaces the component's HTML element with a different tag, or composes it with another component).",
                "Data attributes: data-disabled (present when the button is disabled).",
            )}

        </article>
    }
}
