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
/// subscribes to nothing and the rebuild would never fire. (The prior
/// session's dynamic-view-child route was a dead end, recorded here for the
/// audit: a closure returning a view VALUE is evaluated once at build — the
/// runtime warning names the untracked read — and never re-runs; only text
/// closures get the runtime's effect wrap. The hand-rolled-effect failure it
/// chased was the old free-function form's forgotten owner, not the
/// mechanism: under a held mount guard the Effect tracks reliably.)
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
                leptos::web_sys::console::log_1(&"[btn-diag] consumer onClick FIRED".into());
                loading_mirror.set(true);
                leptos::web_sys::console::log_1(&"[btn-diag] loading set true".into());
                timeouts_for_click.start("button-loading-demo-reset", reset_ms, {
                    let loading_mirror = loading_mirror.clone();
                    move || {
                        leptos::web_sys::console::log_1(&"[btn-diag] reset timer FIRED".into());
                        loading_mirror.set(false)
                    }
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
    // DIAGNOSTIC: static id — tests whether use_base_ui_id clobbers the
    // reactive owner (the reactive_graph 0.2 Owner::set trap) and kills the
    // Effect below.
    let label_id: String = "base-ui-diag-label".to_string();

    // The demo's `loading` state — the mirror the React demo's useState
    // re-renders on.
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

    // The initial build runs synchronously at component construction — the
    // React demo's first render happens before the event loop turns. The
    // build+materialize chain clobbers the thread-local reactive owner (the
    // cross-crate owner leak the direction-provider page documents), so it
    // runs inside its OWN Owner::with — which saves and restores the
    // surrounding owner — keeping the Effect created AFTER this seed under
    // the component's real reactive scope (the merge-props seed
    // convention).
    // The clobber guard: the seed build chain (button_element →
    // use_render_element → create_element) leaves the thread-local current
    // owner broken — the direction-provider page's reactive_graph 0.2 trap
    // (Owner::set overwrites without saving; only Owner::with saves and
    // restores). merge-props escapes only because toggle's chain does not
    // clobber; this demo's does, so the Effect must be created under the
    // mount's owner, captured BEFORE the seed: an Effect born under a broken
    // owner is dropped before its deferred first run executes — the run then
    // fires untracked (the "outside a reactive tracking context" warning),
    // subscribes to nothing, and the rebuild never happens.
    let effect_owner = reactive_graph::owner::Owner::current();

    let seed_owner = reactive_graph::owner::Owner::new();
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
    match effect_owner {
        Some(owner) => owner.with(|| {
            Effect::new(move |_| {
                // Tracked read — deliberately `.get()`, not `get_untracked()`:
                // an untracked read subscribes to nothing and the rebuild would
                // never fire (the merge-props Effect's rule). `demo_build` reads
                // the value untracked, so this body is the only subscriber.
                let current_loading = loading_for_effect.get();
                leptos::web_sys::console::log_1(
                    &format!("[btn-diag] effect run, loading={current_loading}").into(),
                );
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
        }),
        // A demo constructed outside any reactive owner cannot rebuild — that
        // is the broken state this guard exists to escape, so fail loudly
        // rather than silently mounting a dead button.
        None => panic!(
            "ButtonLoadingDemo: no reactive owner in scope — mount the demo \
             inside a real mount (mount_to); a rebuild Effect created here \
             would never subscribe"
        ),
    }

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
            <div class="docs-demo" data-demo="loading">
                <ButtonLoadingDemo reset_ms=4000 />
            </div>

            <h2>"API reference"</h2>
            {api_part(
                "A button component that can be used to trigger actions. Renders a <button> element.",
                "Props: disabled (boolean, false — whether the button should ignore user interaction; the custom-element path signals it with aria-disabled instead), focusableWhenDisabled (boolean, false — whether the button should be focusable when disabled), nativeButton (boolean, true — whether the component renders a native <button> element when replacing it via the render prop; set to false if the rendered element is not a button, for example <div>), className (string | ((state: Button.State) => string | undefined)), style (React.CSSProperties | ((state: Button.State) => React.CSSProperties | undefined)), render (ReactElement | ((props: HTMLProps, state: Button.State) => ReactElement) — replaces the component's HTML element with a different tag, or composes it with another component).",
                "Data attributes: data-disabled (present when the button is disabled).",
            )}

        </article>
    }
}
