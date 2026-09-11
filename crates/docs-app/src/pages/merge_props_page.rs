//! The docs page for `mergeProps`, mirroring
//! `docs/src/app/(docs)/react/utils/merge-props/page.mdx`
//! (`specs/docs-content/merge-props/page.md`) — the `docs-content: utils/merge-props`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# mergeProps` h1,
//! `<Subtitle>` ("A utility to merge multiple sets of React props."), the intro
//! prose, `## How merging works` with the four `ts` snippet fences (echoed as
//! static code blocks per the toggle/csp-provider page precedent — the snippets
//! are documentation, and the port's merge_props unit tests already pin their
//! asserted outputs), `### Preventing Base UI's default behavior` with the
//! page's one live demo, `## Passing a function instead of an object` with the
//! `tsx` snippet, and `## API reference` over the two generated reference
//! components (echoed as static prose per the toggle page's `TypesToggle`
//! precedent — the port has no docs generator).
//!
//! The live demo is the upstream `DemoPreventBaseUIHandler`
//! (`docs/src/app/(docs)/react/utils/merge-props/demos/prevent-base-ui-handler/
//! css-modules/index.tsx`, the demos.json entry) ported onto the REAL
//! `leptos_ui::toggle_element` + `leptos_ui_internals::merge_props` machinery:
//! a controlled `Toggle` (`pressed` wired through the `on_pressed_change`
//! mirror — the demo's `useState` + `onPressedChange={setPressed}` pair)
//! whose consumer `onClick` handler (`ToggleHandlers.on_click`, the
//! `elementProps` rest bag) calls `event.prevent_base_ui_handler()` while the
//! demo is locked. The merged composition is the real `merge_event_handlers`
//! fold: the consumer's handler is the later bag's handler, so it runs first
//! and its prevention mark gates Toggle's own click machine — the demo's
//! observable contract ("the heart stays in its current state while locked"),
//! the exact behavior merge_props.rs's prevention tests pin at the unit level.
//!
//! The demo's render prop is the function form
//! (`render={(props, state) => <button …>}`) with the heart icon per
//! `state.pressed` — the same shape as the toggle hero (the toggle_page
//! `hero_render_prop` precedent), demonstrating the page's claim that props
//! are NOT merged automatically in render functions and that the consumer
//! chains them (here: the `type="button"` attribute plus the spread props bag).

use leptos::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set as _, Update as _};
use leptos::web_sys::MouseEvent;

use leptos_ui::{ToggleHandlers, ToggleProps, toggle_element};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderProp, RenderedElement, UseRenderElementComponentProps,
};

use crate::pages::use_render_page::RawElementView;

const HEART_FILLED: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path d="M7.99961 13.8667C7.88761 13.8667 7.77561 13.8315 7.68121 13.7611C7.43321 13.5766 1.59961 9.1963 1.59961 5.8667C1.59961 3.80856 3.27481 2.13336 5.33294 2.13336C6.59054 2.13336 7.49934 2.81176 7.99961 3.3131C8.49988 2.81176 9.40868 2.13336 10.6663 2.13336C12.7244 2.13336 14.3996 3.80803 14.3996 5.8667C14.3996 9.1963 8.56601 13.5766 8.31801 13.7616C8.22361 13.8315 8.11161 13.8667 7.99961 13.8667Z" /></svg>"#;

const HEART_OUTLINE: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path fill-rule="evenodd" clip-rule="evenodd" d="m7.99961 4.8232-.75505-.75666c-.40333-.40419-1.0559-.86651-1.91162-.86651-1.46903 0-2.66666 1.19764-2.66666 2.66667 0 .5412.24648 1.2356.75339 2.04713.49581.79376 1.17682 1.59861 1.89311 2.33647 1.06989 1.1022 2.1604 1.9962 2.68705 2.4102.52751-.4149 1.61735-1.3085 2.68657-2.4101.7163-.73792 1.3973-1.54278 1.8932-2.33656.5069-.81154.7533-1.50594.7533-2.04714 0-1.46947-1.1975-2.66667-2.6666-2.66667-.85574 0-1.50831.46232-1.91164.86651zm-.01387-1.52394c-.5031-.49988-1.40673-1.1659-2.6528-1.1659-2.05813 0-3.73333 1.6752-3.73333 3.73334 0 3.3296 5.8336 7.7099 6.0816 7.8944a.532.532 0 0 0 .3184.1056c.112 0 .224-.0352.3184-.1051.248-.185 6.08159-4.5653 6.08159-7.8949 0-2.05867-1.6752-3.73334-3.7333-3.73334-1.24617 0-2.14985.66611-2.65293 1.166q-.0069.00686-.0137.01367c.00002-.00003-.00002.00002 0 0-.00459-.0046-.00927-.00914-.01393-.01377" /></svg>"#;

/// The demo's render prop (`css-modules/index.tsx:22-35`): the function form of
/// `render` — `(props, state) => <button type="button" {...getToggleProps(props)}>
/// {state.pressed ? filled : outline}</button>`. The port receives the merged
/// bag and state map from the real `use_render_element` evaluation and returns
/// the per-state `<button type="button">` description: the consumer-supplied
/// `type="button"` (the demo's explicit attribute), the merged bag's remaining
/// props (including the merged handler composition), and the icon per
/// `state.pressed`.
fn demo_render_prop() -> RenderProp {
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
                    handlers: {
                        let mut handlers = props.handlers;
                        handlers.attributes.push((
                            "type".to_string(),
                            Rc::new(|| Some("button".to_string()))
                                as leptos_ui_internals::floating_ui::element_props::ElementAttributeFn,
                        ));
                        handlers
                    },
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

/// The upstream demo (`css-modules/index.tsx:8-35`) on the real port. Two
/// pieces of React state, the demo's `useState` pair:
///
/// - `pressed` (initially `true`): controlled Toggle state, mirrored through
///   `on_pressed_change` — the demo's `pressed={pressed}
///   onPressedChange={setPressed}` wiring. The mirror is the re-render driver:
///   the rebuild effect reads it tracked, standing in for React re-rendering
///   the demo component when `setPressed` fires (the toggle_page precedent).
/// - `locked` (initially `true`): gates whether the consumer handler calls
///   `event.prevent_base_ui_handler()`, and drives the Lock/Unlock button's
///   label. It is ALSO read tracked by the rebuild effect — flipping it
///   re-renders the demo exactly like the React `setLocked` does.
///
/// The prevention contract itself needs no rebuild: the consumer handler is
/// the merged composition's later handler, so it runs first on every click
/// and its mark gates Toggle's machine live. The rebuild-on-locked exists to
/// reproduce the upstream demo's re-render (the "(locked)"/"(unlocked)" label
/// and a fresh machine seeded to the current pressed state), not because the
/// prevention depends on it.
pub fn prevent_base_ui_handler_demo_with(
    pressed_mirror: RwSignal<Option<bool>>,
    locked: RwSignal<bool>,
) -> impl IntoView {
    let container = document().create_element("span").unwrap();
    container.set_attribute("data-merge-props-demo", "").unwrap();

    let build = move || {
        let seeded_pressed = pressed_mirror.get_untracked().unwrap_or(true);
        let locked_read = locked.clone();
        let rendered = toggle_element(ToggleProps {
            // Controlled (the demo passes `pressed`); the mirror seeds
            // `defaultPressed` so each rebuild starts in the state the
            // previous machine ended in — the controlled write is the no-op
            // commit path (the port's useControlled tri-state).
            pressed: None,
            default_pressed: seeded_pressed,
            disabled: false,
            on_pressed_change: {
                let mirror = pressed_mirror.clone();
                Some(Arc::new(
                    move |next: bool, _details: &BaseUIChangeEventDetails<(), MouseEvent>| {
                        mirror.set(Some(next));
                    },
                ))
            },
            value: None,
            native_button: true,
            render_class_style: UseRenderElementComponentProps {
                // The demo's `className={styles.Toggle}` — the css-modules
                // local name carried as the class attribute value (the port
                // has no css-modules pipeline; the class identity is what the
                // demo passes, documented here rather than restyled).
                class_name: Some(ClassNameSource::Static("Toggle".to_string())),
                render: Some(demo_render_prop()),
                style: None,
            },
            // `aria-label="Favorite"` (`:9`) rides the elementProps rest.
            element_attributes: vec![("aria-label".to_string(), "Favorite".to_string())],
            // The demo's `getToggleProps` consumer handler
            // (`:11-19`): `mergeProps<'button'>(props, { onClick })` — while
            // locked, `event.preventBaseUIHandler()`. In the port this is the
            // `elementProps` rest bag's handler; the real `merge_event_handlers`
            // composition runs it FIRST (rightmost-first) and its prevention
            // mark gates Toggle's own machine (the unit tests' pin).
            handlers: ToggleHandlers {
                on_click: Some(Rc::new(move |event: &leptos_ui_internals::types::BaseUIEvent<MouseEvent>| {
                    if locked_read.get_untracked() {
                        event.prevent_base_ui_handler();
                    }
                })),
            },
        })
        .expect("standalone Toggle renders (no group context)");
        rendered
    };

    let build_container = container.clone();
    let pressed_for_effect = pressed_mirror.clone();
    let locked_for_effect = locked.clone();
    Effect::new(move |_| {
        // Tracked reads — the re-render analog (React re-renders the demo
        // component on either state change; the effect re-runs on either
        // signal). Deliberately `.get()`, not `get_untracked()`: an untracked
        // read subscribes to nothing and the rebuild would never fire.
        let current_pressed = pressed_for_effect.get();
        let current_locked = locked_for_effect.get();
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
        // Keep the tracked reads alive past the early returns above.
        let _ = (current_pressed, current_locked);
    });

    let label_locked = locked.clone();
    let unlock_click = locked.clone();
    view! {
        <div class="merge-props-demo-container">
            <div class="merge-props-demo-toggle-row">
                <span data-merge-props-toggle-slot>{RawElementView { element: container }}</span>
                <span class="merge-props-demo-label">
                    "Favorite "
                    {move || {
                        if label_locked.get() { "(locked)" } else { "(unlocked)" }
                    }}
                </span>
            </div>
            <button
                class="merge-props-demo-button"
                data-merge-props-lock-button=""
                on:click=move |_| unlock_click.update(|l| *l = !*l)
            >
                {move || if locked.get() { "Unlock" } else { "Lock" }}
            </button>
        </div>
    }
}

/// The demo with its own state mirrors — the standalone form
/// (`prevent_base_ui_handler_demo_with` documents the state contract). The
/// demo's initial state: pressed `true`, locked `true` (`css-modules/index.tsx:9-10`).
pub fn prevent_base_ui_handler_demo() -> impl IntoView {
    prevent_base_ui_handler_demo_with(RwSignal::new(None), RwSignal::new(true))
}

/// The `docs/src/app/(docs)/react/utils/merge-props/page.mdx` page.
#[component]
pub fn MergePropsPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"mergeProps"</h1>
            <p class="subtitle">"A utility to merge multiple sets of React props."</p>

            <p>
                "`mergeProps` helps you combine multiple prop objects (for example, internal "
                "props + user props) into a single set of props you can spread onto an element. "
                "It behaves like `Object.assign` (rightmost wins) with a few special cases, so "
                "common React patterns work as expected."
            </p>

            <h2>"How merging works"</h2>
            <ul>
                <li>
                    "For most keys (everything except `className`, `style`, and event handlers), "
                    "the value from the rightmost object wins:"
                    <pre><code>"mergeProps({ id: 'a', dir: 'ltr' }, { id: 'b' });"
                    </code></pre>
                </li>
                <li>
                    "`ref` is not merged. Only the rightmost ref is kept:"
                    <pre><code>"mergeProps({ ref: refA }, { ref: refB });"
                    </code></pre>
                </li>
                <li>
                    "`className` values are concatenated right-to-left (rightmost first):"
                    <pre><code>"mergeProps({ className: 'a' }, { className: 'b' });"
                    </code></pre>
                </li>
                <li>
                    "`style` objects are merged, with keys from the rightmost style overwriting "
                    "earlier ones."
                </li>
                <li>
                    "Event handlers are merged and executed right-to-left (rightmost first):"
                    <pre><code>"mergeProps({ onClick: a }, { onClick: b });"
                    </code></pre>
                    <ul>
                        <li>
                            "For React synthetic events, Base UI adds "
                            "`event.preventBaseUIHandler()`. Calling it prevents Base UI's "
                            "internal logic from running. This does not call `preventDefault()` "
                            "or `stopPropagation()`."
                        </li>
                        <li>
                            "For non-synthetic events (custom events with primitive/object "
                            "values), this mechanism isn't available and all handlers always "
                            "execute."
                        </li>
                    </ul>
                </li>
            </ul>

            <h3>"Preventing Base UI's default behavior"</h3>
            <p>
                "When using the function form of the `render` prop, props are not merged "
                "automatically. You can use `mergeProps` to combine Base UI's props with your "
                "own, and call `preventBaseUIHandler()` to stop Base UI's internal logic from "
                "running:"
            </p>

            <div class="docs-demo">{prevent_base_ui_handler_demo()}</div>

            <h2>"Passing a function instead of an object"</h2>
            <p>
                "Each argument can be a props object or a function that receives the merged "
                "props up to that point (left to right) and returns a props object. This is "
                "useful when you need to compute the next props from whatever has already been "
                "merged."
            </p>
            <p>
                "Note that the function's return value completely replaces the accumulated "
                "props up to that point. If you want to chain event handlers from the previous "
                "props, you must call them manually:"
            </p>
            <pre><code>
"const merged = mergeProps(
  {
    onClick(event) {
      // Handler from previous props
    },
  },
  (props) => ({
    onClick(event) {
      // Manually call the previous handler
      props.onClick?.(event);
      // Your logic here
    },
  }),
);"
            </code></pre>

            <h2>"API reference"</h2>
            <h3>"mergeProps"</h3>
            <p>
                "This function accepts up to 5 arguments, each being either a props object or "
                "a function that returns a props object. If you need to merge more than 5 sets "
                "of props, use `mergePropsN` instead."
            </p>
            <p>
                "`mergeProps` merges plain attributes (rightmost wins per key), `className` "
                "(concatenated rightmost-first), `style` (per-key later-wins), event handlers "
                "(executed right-to-left with `preventBaseUIHandler()` gating), and `ref` "
                "(never merged; only the rightmost ref is kept)."
            </p>
            <h3>"mergePropsN"</h3>
            <p>
                "This function accepts an array of props objects or functions that return "
                "props objects. It is slightly less efficient than `mergeProps`, so only use "
                "it when you need to merge more than 5 sets of props."
            </p>
        </article>
    }
}
