//! The docs page for `Toggle`, mirroring
//! `docs/src/app/(docs)/react/components/toggle/page.mdx`
//! (`specs/docs-content/toggle/page.md`) — the `docs-content: components/toggle`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Toggle` h1,
//! `<Subtitle>` ("A two-state button that can be on or off."), the hero demo
//! before the first heading, `## Anatomy` with the single fenced snippet, and
//! `## API reference` over the generated `TypesToggle` reference (echoed as
//! static prose per the csp-provider page precedent — the port has no docs
//! generator, so the table's documented props/data-attributes/types are
//! rendered as text, never fabricated as executable machinery).
//!
//! The live demo is the upstream hero
//! (`docs/src/app/(docs)/react/components/toggle/demos/hero/tailwind/index.tsx`,
//! `specs/docs-content/toggle/demos.json` entry 1) ported onto the REAL
//! `leptos_ui::toggle_element` port: an uncontrolled `Toggle` (no
//! `pressed`/`defaultPressed`), `aria-label="Favorite"`, the Tailwind class
//! string, and a render prop returning a different `<button type="button">`
//! per `state.pressed` — filled heart when pressed, outline heart otherwise —
//! exactly the demo's `(props, state) => button` shape. `data-pressed` styling
//! rides the real state mapping (the port's tests pin the attribute; the demo
//! carries the upstream class string whose `data-pressed:` variants key off
//! it).

use leptos::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

use leptos::web_sys::MouseEvent;

use leptos_ui::{toggle_element, ToggleProps};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderProp, RenderedElement, UseRenderElementComponentProps,
};

use crate::pages::use_render_page::RawElementView;

const HERO_CLASS: &str = "flex size-8 items-center justify-center border-none rounded-none bg-transparent text-neutral-950 dark:text-white select-none hover:not-data-disabled:bg-neutral-100 dark:hover:not-data-disabled:bg-neutral-800 active:not-data-disabled:bg-neutral-200 dark:active:not-data-disabled:bg-neutral-700 data-pressed:text-neutral-950 dark:data-pressed:text-white focus-visible:outline-2 focus-visible:-outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white";

const HEART_FILLED: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path d="M7.99961 13.8667C7.88761 13.8667 7.77561 13.8315 7.68121 13.7611C7.43321 13.5766 1.59961 9.1963 1.59961 5.8667C1.59961 3.80856 3.27481 2.13336 5.33294 2.13336C6.59054 2.13336 7.49934 2.81176 7.99961 3.3131C8.49988 2.81176 9.40868 2.13336 10.6663 2.13336C12.7244 2.13336 14.3996 3.80803 14.3996 5.8667C14.3996 9.1963 8.56601 13.5766 8.31801 13.7616C8.22361 13.8315 8.11161 13.8667 7.99961 13.8667Z" /></svg>"#;

const HEART_OUTLINE: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path fill-rule="evenodd" clip-rule="evenodd" d="m7.99961 4.8232-.75505-.75666c-.40333-.40419-1.0559-.86651-1.91162-.86651-1.46903 0-2.66666 1.19764-2.66666 2.66667 0 .5412.24648 1.2356.75339 2.04713.49581.79376 1.17682 1.59861 1.89311 2.33647 1.06989 1.1022 2.1604 1.9962 2.68705 2.4102.52751-.4149 1.61735-1.3085 2.68657-2.4101.7163-.73792 1.3973-1.54278 1.8932-2.33656.5069-.81154.7533-1.50594.7533-2.04714 0-1.46947-1.1975-2.66667-2.6666-2.66667-.85574 0-1.50831.46232-1.91164.86651zm-.01387-1.52394c-.5031-.49988-1.40673-1.1659-2.6528-1.1659-2.05813 0-3.73333 1.6752-3.73333 3.73334 0 3.3296 5.8336 7.7099 6.0816 7.8944a.532.532 0 0 0 .3184.1056c.112 0 .224-.0352.3184-.1051.248-.185 6.08159-4.5653 6.08159-7.8949 0-2.05867-1.6752-3.73334-3.7333-3.73334-1.24617 0-2.14985.66611-2.65293 1.166q-.0069.00686-.0137.01367c.00002-.00003-.00002.00002 0 0-.00459-.0046-.00927-.00914-.01393-.01377" /></svg>"#;

/// The upstream hero demo's render prop (`hero/tailwind/index.tsx:10-24`):
/// `(props, state) => state.pressed ? <button>…filled…</button> :
/// <button>…outline…</button>`, spreading Toggle's merged props onto each
/// rendered button. The port receives the merged bag and the state map from
/// the real `use_render_element` evaluation and returns the per-state
/// `<button type="button">` description carrying those props.
fn hero_render_prop() -> RenderProp {
    RenderProp::Function(Rc::new(
        |props: leptos_ui_internals::use_render_element::RenderElementProps,
         state: &serde_json::Map<String, serde_json::Value>| {
            let pressed = state
                .get("pressed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            RenderedElement {
                tag: "button".to_string(),
                props: leptos_ui_internals::use_render_element::RenderElementProps {
                    // `type="button"` on each rendered button (`:12, :20`).
                    handlers: {
                        let mut handlers = props.handlers;
                        handlers.attributes.push((
                            "type".to_string(),
                            Rc::new(|| Some("button".to_string()))
                                as leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
                        ));
                        handlers
                    },
                    // The icon is the button's content — the render prop's JSX
                    // child, delivered as HTML (both icons are static SVGs).
                    inner_html: Some(
                        if pressed { HEART_FILLED } else { HEART_OUTLINE }.to_string(),
                    ),
                    class: props.class,
                    style: props.style,
                    ref_callback: props.ref_callback,
                },
            }
        },
    ))
}

/// The upstream hero demo (`hero/tailwind/index.tsx:7-25`) on the real port:
/// uncontrolled `Toggle`, `aria-label="Favorite"`, the demo's Tailwind class,
/// and the per-state render prop above.
///
/// Rebuilt on `pressed_source`'s change — React re-renders the demo component
/// when Toggle's state flips (the render prop re-runs with the new
/// `state.pressed`); Leptos runs the body once, so this Effect-driven
/// rebuild (the `counter_demo` use-render precedent) is the re-render
/// analog. The demo's own "no React state" contract is preserved: the
/// source is Toggle's internal pressed value, read back through the real
/// machine.
pub fn toggle_hero_demo_with(pressed_source: RwSignal<Option<bool>>) -> RawElementView {
    let container = document().create_element("span").unwrap();
    container.set_attribute("data-toggle-hero", "").unwrap();

    let build = move || {
        let seeded_pressed = pressed_source.get_untracked().unwrap_or(false);
        let rendered = toggle_element(ToggleProps {
            // Uncontrolled (the demo passes no `pressed` prop). The mirror
            // seeds `defaultPressed` so each rebuild starts in the state the
            // previous machine ended in; `onPressedChange` below records the
            // flip that re-runs this build.
            pressed: None,
            default_pressed: seeded_pressed,
            disabled: false,
            on_pressed_change: {
                let mirror = pressed_source;
                Some(std::sync::Arc::new(move |next: bool, _details: &BaseUIChangeEventDetails<(), MouseEvent>| {
                    mirror.set(Some(next));
                }))
            },
            value: None,
            native_button: true,
            render_class_style: UseRenderElementComponentProps {
                class_name: Some(ClassNameSource::Static(HERO_CLASS.to_string())),
                render: Some(hero_render_prop()),
                style: None,
            },
            // `aria-label="Favorite"` (`:8`) rides the elementProps rest.
            element_attributes: vec![("aria-label".to_string(), "Favorite".to_string())],
            handlers: Default::default(),
        })
        .expect("standalone Toggle renders (no group context)");
        rendered
    };

    // The initial build runs synchronously at creation — the React demo's
    // first render happens before the event loop turns. The build+materialize
    // chain (toggle_element -> use_render_element -> create_element) clobbers
    // the thread-local reactive owner (the cross-crate owner leak the
    // direction-provider page documents), so it runs inside its OWN
    // Owner::with — which saves and restores the surrounding owner — keeping
    // the closures built AFTER this seed (the page's label/button text) under
    // the page's real tracking scope. (Effect::new defers its first run to
    // the executor's flush, which left the container empty for any
    // synchronous observer — the wasm suite's first contact with a real
    // browser caught that; the Effect below handles only the reactive
    // REBUILDS, replacing the seeded element wholesale.)
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

    let build_container = container.clone();
    Effect::new(move |_| {
        let rendered = build();
        let container = &build_container;
        // Remove the previous button before mounting the fresh one —
        // the per-render replacement semantics of React's re-render.
        while let Some(child) = container.first_child() {
            let _ = container.remove_child(&child);
        }
        let (element, cleanup) = rendered.create_element();
        let _ = container.append_child(&element);
        std::mem::forget(cleanup);
    });

    RawElementView { element: container }
}

/// The hero demo seeded with its own pressed mirror — the standalone form
/// (`toggle_hero_demo_with` documents the reactivity contract).
pub fn toggle_hero_demo() -> RawElementView {
    toggle_hero_demo_with(RwSignal::new(None))
}

/// The `docs/src/app/(docs)/react/components/toggle/page.mdx` page.
#[component]
pub fn TogglePage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Toggle"</h1>
            <p class="subtitle">"A two-state button that can be on or off."</p>

            <div class="docs-demo">{toggle_hero_demo()}</div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and use it as a single part:"</p>
            <pre><code>
"import { Toggle } from '@base-ui/react/toggle';

// prettier-ignore
<Toggle />"
            </code></pre>

            <h2>"API reference"</h2>
            <p>
                "Props: `value` (unique string identifying the toggle inside a toggle group), "
                "`defaultPressed` (default `false`; uncontrolled counterpart of `pressed`), "
                "`pressed` (controlled counterpart of `defaultPressed`), "
                "`onPressedChange` (`(pressed: boolean, eventDetails: Toggle.ChangeEventDetails) => void`), "
                "`nativeButton` (default `true`; set `false` when replacing the element with a "
                "non-button via `render`), `disabled` (default `false`), plus the standard "
                "`className`, `style`, and `render` props."
            </p>
            <p>
                "A `Toggle` renders a `button` element. Data attributes: `data-pressed` "
                "(present when pressed) and `data-disabled` (present when disabled)."
            </p>
            <p>
                "`Toggle.State` is `{ pressed: boolean, disabled: boolean }`. "
                "`Toggle.ChangeEventDetails` exposes `reason`, `event`, `cancel()`, "
                "`allowPropagation()`, `isCanceled`, `isPropagationAllowed`, and `trigger`."
            </p>
        </article>
    }
}
