use leptos::prelude::*;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::use_render::{UseRenderParameters, use_render};
use leptos_ui_internals::use_render_element::{
    RenderElementHandlers, RenderElementProps, RenderFn, RenderProp, RenderedElement,
};
use serde_json::json;
use std::rc::Rc;

use ::leptos::tachys::renderer::CastFrom;
use ::leptos::tachys::html::attribute::Attribute;
use ::leptos::tachys::renderer::dom;
use ::leptos::tachys::renderer::types as renderer_types;
use ::leptos::tachys::view::Mountable;
use ::leptos::tachys::view::Render;
use ::leptos::tachys::view::add_attr::AddAnyAttr;
use ::leptos::tachys::view::Position;
use ::leptos::tachys::view::PositionState;
use ::leptos::tachys::hydration::Cursor;
use reactive_graph::owner::Owner as _OwnerTrait;

/// Bridges the internals crate's [`RenderedElement`] — the Rust shape of
/// React's `useRender` return value — into the Leptos view tree.
///
/// `create_element()` (the seam standing in for React rendering the returned
/// `ReactElement` upstream) materializes the description into a real DOM node:
/// class, style, the lazy attributes, the `inner_html` content slot, the
/// attached handler slots, and the forked ref firing. This wrapper implements
/// tachys' `Render`/`Mountable` so the materialized node mounts as a view;
/// `rebuild` swaps in a freshly materialized node, mirroring React's
/// per-render replacement of the `useRender` return value.
pub struct RawElementView {
    pub element: dom::Element,
}

/// The retained view state: the mounted DOM node.
pub struct RawElementState(dom::Element);

impl Mountable for RawElementState {
    fn unmount(&mut self) {
        let _ = self.0.remove();
    }

    fn mount(
        &mut self,
        parent: &renderer_types::Element,
        marker: Option<&renderer_types::Node>,
    ) {
        ::leptos::tachys::renderer::Rndr::insert_node(parent, self.0.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        if let Some(parent) = self.0.parent_node() {
            if let Some(element) = renderer_types::Element::cast_from(parent) {
                child.mount(&element, Some(self.0.as_ref()));
                return true;
            }
        }
        false
    }
}

impl Render for RawElementView {
    type State = RawElementState;

    fn build(self) -> Self::State {
        RawElementState(self.element)
    }

    fn rebuild(self, state: &mut Self::State) {
        // A rebuilt RawElementView is a new materialization: replace the node
        // in place (the per-render replacement of the useRender return).
        if let Some(parent) = state.0.parent_node() {
            let _ = parent.replace_child(&self.element, &state.0);
        }
        state.0 = self.element;
    }
}

// CSR-only view: server-side rendering has no meaning for the docs app.
impl ::leptos::tachys::view::RenderHtml for RawElementView {
    type AsyncOutput = Self;

    const MIN_LENGTH: usize = 0;

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }

    fn to_html_with_buf(
        self,
        _buf: &mut String,
        _position: &mut Position,
        _escape: bool,
        _mark_branches: bool,
    ) {
        // No server-side HTML for a live-materialized node (CSR-only).
    }

    fn hydrate<const FROM_SERVER: bool>(
        self,
        _cursor: &Cursor,
        _position: &PositionState,
    ) -> Self::State {
        self.build()
    }
}

impl AddAnyAttr for RawElementView {
    type Output<SomeNewAttr: Attribute> = RawElementView;

    fn add_any_attr<NewAttr: Attribute>(self, _attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml,
    {
        self
    }
}

impl RawElementView {
    /// Materializes the description now; the view renders exactly this node.
    pub fn new(rendered: RenderedElement) -> Self {
        let (element, cleanup) = rendered.create_element();
        // The listener/ref teardown rides the app owner: registered under the
        // current reactive owner (the component body that built the view), the
        // same owner scope upstream React unmount cleanup would run in.
        if let Some(cleanup) = cleanup {
            if let Some(owner) = Owner::current() {
                owner.with(|| {
                    // SendWrapper satisfies on_cleanup's Send+Sync bound on the
                    // wasm single thread, the same wrapper tachys' `on` uses.
                    let cleanup = send_wrapper::SendWrapper::new(cleanup);
                    on_cleanup(move || cleanup.take()());
                });
            }
        }
        Self { element }
    }
}

/// The `render` demo — `docs/src/app/(docs)/react/utils/use-render/demos/render/css-modules/index.tsx`
/// ported onto the real `leptos_ui_internals::use_render`. The custom `Text`
/// component defaults to a `<p>`; the consumer overrides it with the element
/// form of the render prop (`<strong>`).
///
/// The element form of the render prop is expressed the way the engine sees
/// it: a render function returning the consumer's `<strong>` element with
/// exactly the props it declared (upstream `cloneElement` semantics), which
/// `use_render_element`'s evaluate step then merges with the component props.
pub struct TextProps {
    /// The consumer's element-form render: the tag it overrides `<p>` with.
    pub render_tag: Option<String>,
    /// The consumer's remaining props (here just the children text).
    pub children: String,
}

pub fn text_element(props: TextProps) -> RawElementView {
    // `mergeProps({ className: styles.Text }, otherProps)` — the component's
    // own default class first, the consumer's remaining props last.
    let defaults = RenderElementProps {
        class: Some("docs-use-render-text".to_string()),
        ..RenderElementProps::default()
    };
    let children = props.children;
    let merged = leptos_ui_internals::merge_props::merge_props_n(vec![
        PropsSource::Static(defaults),
        // The consumer's remaining props: here just the children content.
        PropsSource::Getter(Rc::new(move |previous: &RenderElementProps| {
            let mut merged = previous.clone();
            merged.inner_html = Some(html_escape(&children));
            merged
        })),
    ]);

    // The element-form render prop: `<Text render={<strong />}>` overrides the
    // default tag entirely while the merged props/children still flow into it.
    // The render element rides `RenderProp::Element` — the port's shape for the
    // JSX element (`mergeProps(props, render.props)` + `cloneElement`,
    // `useRenderElement.tsx:172-196`): the merged bag folds into the element's
    // own (empty) props, so the class and the escaped children flow through.
    let render = props
        .render_tag
        .map(|tag| RenderProp::Element { tag, props: RenderElementProps::default() });

    let params = UseRenderParameters {
        render,
        props: vec![PropsSource::Static(merged)],
        enabled: true,
        ..UseRenderParameters::default()
    };
    let rendered = use_render("p", &params).expect("useRender enabled");
    RawElementView::new(rendered)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The `render-callback` demo — `docs/src/app/(docs)/react/utils/use-render/demos/render-callback/css-modules/index.tsx`.
/// The Counter owns `count` (uncontrolled), derives `state = { odd }`, merges
/// its defaultProps via `mergeProps`, and the consumer's render callback
/// spreads the merged props onto its own `<button>` and appends the
/// 👍/👎 suffix from `state.odd`.
pub fn counter_demo() -> impl IntoView {
    let count = RwSignal::new(0i32);

    let container = document().create_element("span").unwrap();
    container.set_attribute("data-use-render-counter", "").unwrap();

    // The default props bag (the component's own defaults), rebuilt per
    // reactive run — children carry the live count, onClick increments,
    // aria-label tracks the count, exactly as the demo's defaultProps do.
    let build = move || {
        let current = count.get_untracked();
        let odd = current % 2 == 1;

        let label = format!("Count is {current}, click to increase.");
        let label_attr = label.clone();
        let count_signal = count;

        let state = json!({ "odd": odd }).as_object().cloned().unwrap();

        let defaults = RenderElementProps {
            class: Some("docs-use-render-counter".to_string()),
            handlers: {
                let mut handlers = RenderElementHandlers::default();
                handlers.on_click = Some(Rc::new(move |_event| {
                    count_signal.update(|value| *value += 1);
                }));
                handlers.attributes = vec![
                    (
                        "aria-label".to_string(),
                        Rc::new(move || Some(label_attr.clone())) as _,
                    ),
                    (
                        "type".to_string(),
                        Rc::new(|| Some("button".to_string())) as _,
                    ),
                ];
                handlers
            },
            inner_html: Some(format!(
                "Counter: <span class=\"docs-use-render-count\">{current}</span>"
            )),
            ..RenderElementProps::default()
        };

        let merged =
            leptos_ui_internals::merge_props::merge_props_n(vec![PropsSource::Static(defaults)]);

        // The consumer's render callback: receives (props, state), owns the
        // output element, appends the odd/even suffix from state.odd.
        let render_callback: RenderFn =
            Rc::new(move |mut props: RenderElementProps, state: &_| {
                let odd = state
                    .get("odd")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                // The callback spreads the merged props onto its own button:
                // the class/handlers/attributes stay, and the suffix span is
                // appended after the children content.
                let suffix = if odd { "👎" } else { "👍" };
                props.inner_html = Some(match props.inner_html.take() {
                    Some(existing) => format!(
                        "{existing}<span class=\"docs-use-render-suffix\">{suffix}</span>"
                    ),
                    None => format!("<span class=\"docs-use-render-suffix\">{suffix}</span>"),
                });
                RenderedElement {
                    tag: "button".to_string(),
                    props,
                }
            });

        let params = UseRenderParameters {
            render: Some(RenderProp::Function(render_callback)),
            state,
            props: vec![PropsSource::Static(merged)],
            enabled: true,
            ..UseRenderParameters::default()
        };
        use_render("button", &params)
    };

    let build_container = container.clone();
    Effect::new(move |_| {
        let rendered = build().expect("useRender enabled");
        let container = &build_container;
        // Remove the previous button before mounting the fresh one —
        // the per-render replacement semantics of React's useRender.
        while let Some(child) = container.first_child() {
            let _ = container.remove_child(&child);
        }
        let (element, cleanup) = rendered.create_element();
        let _ = container.append_child(&element);
        // Hold the listener cleanup for this effect's lifetime; the effect
        // re-runs before the owner disposes, and the app teardown drops the
        // container with the listeners on it.
        std::mem::forget(cleanup);
    });

    RawElementView {
        element: container,
    }
}

/// The docs page for the `useRender` util, mirroring
/// `docs/src/app/(docs)/react/utils/use-render/page.mdx`
/// (H1 + Subtitle, intro, Examples with the two demos, Merging props,
/// Merging refs, TypeScript, Migrating from Radix UI, Render prop and
/// polymorphism, API reference sections).
#[component]
pub fn UseRenderPage() -> impl IntoView {
    let text_default = text_element(TextProps {
        render_tag: None,
        children: "Text component rendered as a paragraph tag".to_string(),
    });
    let text_strong = text_element(TextProps {
        render_tag: Some("strong".to_string()),
        children: "Text component rendered as a strong tag".to_string(),
    });

    view! {
        <article class="docs-page">
            <h1>"useRender"</h1>
            <p class="subtitle">"Hook for enabling a render prop in custom components."</p>

            <p>
                "The `useRender` hook lets you build custom components that provide a "
                "`render` prop to override the default rendered element."
            </p>

            <h2>"Examples"</h2>
            <p>
                "A `render` prop for a custom Text component lets consumers use it to replace "
                "the default rendered `p` element with a different tag or component."
            </p>
            <div class="docs-demo">
                {text_default}
                {text_strong}
            </div>
            <p>
                "The callback version of the `render` prop enables more control of how props are "
                "spread, and also passes the internal `state` of a component."
            </p>
            <div class="docs-demo">{counter_demo()}</div>

            <h2>"Merging props"</h2>
            <p>
                "The `mergeProps` function merges two or more sets of React props together, "
                "combining class names, styles, and event handlers."
            </p>
            <pre><code>
"const element = useRender({
  defaultTagName: 'p',
  props: mergeProps({ className: styles.Text }, otherProps),
});"
            </code></pre>

            <h2>"Merging refs"</h2>
            <pre><code>
"useRender({
  defaultTagName: 'p',
  refs: [ref],
  props: otherProps,
});"
            </code></pre>

            <h2>"TypeScript"</h2>
            <pre><code>
"interface TextProps extends useRender.ComponentProps<'p'> {}"
            </code></pre>

            <h2>"Migrating from Radix UI"</h2>
            <pre><code>
"// Radix
<Slot.Slot>
// Base UI
useRender({ render })"
            </code></pre>

            <h2>"Render prop and polymorphism"</h2>
            <pre><code>
"<Text render={<strong />}>…</Text>"
            </code></pre>

            <h2>"API reference"</h2>
            <p>"Hook for enabling a render prop in custom components."</p>
        </article>
    }
}
