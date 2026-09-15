//! `AvatarRoot` — `packages/react/src/avatar/root/AvatarRoot.tsx` (the whole
//! component body) plus `avatarStateAttributesMapping`
//! (`root/stateAttributesMapping.ts:1-3`) and the state/props types.
//!
//! Split into the pure builder [`avatar_root_element`] (host-testable, the
//! `separator_element`/`button_element` facade convention) and the full body
//! [`use_avatar_root`] (the context provision + materialization; wasm-only).

use std::rc::Rc;

use leptos::children::Children;
use leptos::html::{Custom, custom};
use leptos::prelude::{AnyView, Effect, Get, IntoView, NodeRef, RwSignal};
use leptos::tachys::html::element::ElementChild;
use leptos::tachys::html::node_ref::NodeRefAttribute;
use send_wrapper::SendWrapper;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::StateAttributeProps;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderElementProps, RenderProp, RenderedElement, StyleSource,
    UseRenderElementComponentProps, UseRenderElementParams, static_attr, use_render_element,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use web_sys::{Element, HtmlElement};

use super::context::{AvatarRootContextValue, ImageLoadingStatus, provide_avatar_root_context};
use super::views::update_element;

/// `avatarStateAttributesMapping` (`stateAttributesMapping.ts:1-3`):
/// `imageLoadingStatus` maps to `null` on every part — no generic status attribute
/// ever leaks to the DOM (implementation.md "State → attribute mapping"). The
/// root's only state member is `imageLoadingStatus` (`AvatarRoot.tsx:22-24`), so
/// this is the whole mapping for Root and Fallback; the image's mapping adds
/// `transitionStatusMapping` on top (the image module).
pub fn avatar_state_attributes_mapping<'a>()
-> impl Fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>> + 'a {
    move |key: &str, _value: &serde_json::Value| {
        if key == "imageLoadingStatus" {
            // The suppression: `imageLoadingStatus: () => null` (`:2`).
            Some(None)
        } else {
            // No mapping for this key — the default walk applies (a no-op here:
            // the root state has no other members).
            None
        }
    }
}

/// The `AvatarRootState` record (`AvatarRoot.tsx:46-51`) — the one member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarRootState {
    /// `imageLoadingStatus` (`:50`).
    pub image_loading_status: ImageLoadingStatus,
}

impl AvatarRootState {
    /// The state record `useRenderElement`'s custom mapping consumes
    /// (`AvatarRoot.tsx:22-24`). The suppression mapping declines the only member,
    /// so the walk emits nothing — the record is carried for exactness (the
    /// `AvatarRoot` render-prop `state` argument, implementation.md untested
    /// item 2).
    pub fn to_state_map(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "imageLoadingStatus".to_string(),
            serde_json::Value::String(self.image_loading_status.as_str().to_string()),
        );
        map
    }
}

/// The `AvatarRoot` props (`AvatarRootProps`, `AvatarRoot.tsx:53`) — the port's
/// body-time facade: `className`/`style`/`render` through the engine's
/// component-props vocabulary, arbitrary DOM props as `(name, value)` pairs (the
/// `...elementProps` rest), the forwarded ref.
pub struct AvatarRootProps {
    /// `className`/`style` (`:18`).
    pub class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:18`) — spread onto the element.
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:16` — `HTMLSpanElement`; behavior.md *DOM
    /// structure*: the `refInstanceof: window.HTMLSpanElement` conformance).
    pub ref_callback: Option<RefCallback<Element>>,
}

impl Default for AvatarRootProps {
    fn default() -> Self {
        Self {
            class_style: Default::default(),
            element_attributes: Vec::new(),
            ref_callback: None,
        }
    }
}

/// The component-props clone the element builders need (`use_render_element`
/// consumes `UseRenderElementComponentProps` by value; the builders receive
/// `&props` so a per-call clone is the move). `RenderProp` is `Clone`; the
/// class/style sources clone through their `Rc` arms.
pub(crate) fn clone_class_style(
    props: &UseRenderElementComponentProps,
) -> UseRenderElementComponentProps {
    use leptos_ui_internals::use_render_element::{ClassNameSource, StyleSource};
    UseRenderElementComponentProps {
        class_name: props.class_name.as_ref().map(|source| match source {
            ClassNameSource::Static(class) => ClassNameSource::Static(class.clone()),
            ClassNameSource::Function(function) => ClassNameSource::Function(Rc::clone(function)),
        }),
        render: props.render.clone(),
        style: props.style.as_ref().map(|source| match source {
            StyleSource::Static(style) => StyleSource::Static(style.clone()),
            StyleSource::Function(function) => StyleSource::Function(Rc::clone(function)),
        }),
    }
}

/// `useRenderElement('span', …)` (`AvatarRoot.tsx:34-39`) as a pure function —
/// the suppression mapping, the plain `elementProps` bag. The description the
/// caller materializes.
pub fn avatar_root_element(state: AvatarRootState, props: &AvatarRootProps) -> RenderedElement {
    let state_map = state.to_state_map();
    let mapping = avatar_state_attributes_mapping();

    let mut intrinsic = RenderElementProps::default();
    for (name, value) in &props.element_attributes {
        intrinsic
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }
    let props_bags = vec![PropsSource::Static(intrinsic)];
    let refs: Vec<InputRef<Element>> = match &props.ref_callback {
        Some(callback) => vec![InputRef::Callback(Rc::clone(callback))],
        None => Vec::new(),
    };

    use_render_element(
        "span",
        clone_class_style(&props.class_style),
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: Some(&mapping),
        },
    )
    .expect("the AvatarRoot renders unconditionally (no enabled gate)")
}

/// `AvatarRoot` (`AvatarRoot.tsx:14-42`) — owns the root-mirrored status
/// (`React.useState('idle')`, `:20`), publishes the context (`:26-41`), and
/// renders the `span` inside the provider (`:41`). Must be called inside a
/// reactive owner (a component body / mount scope).
///
/// The context value's two members are the same signal read two ways (upstream's
/// `useMemo` over `[imageLoadingStatus, setImageLoadingStatus]`, `:26-32`):
/// Fallback reads `image_loading_status`, Image writes through
/// `set_image_loading_status` — in the port they are the *same* `RwSignal`, so
/// the mirror is exact: one write, two observable members, no staleness window.
/// The root status is the *leptos-runtime* signal (the views track it); the
/// image's rg-side machinery bridges writes with the lockstep effect.
///
/// Note the root element's materialization is a one-time snapshot of
/// `imageLoadingStatus: 'idle'` — the status member is suppressed from the DOM
/// by the mapping, so no attribute ever depends on it (upstream re-renders the
/// span with a fresh state object on every status change and the suppression
/// maps it to null both times; the port's single materialization renders the
/// identical DOM).
pub fn use_avatar_root(props: AvatarRootProps) -> Element {
    // `const [imageLoadingStatus, setImageLoadingStatus] = React.useState('idle')`
    // (`:20`) — the root-mirrored status; one signal is both context members.
    let status: RwSignal<ImageLoadingStatus> = RwSignal::new(ImageLoadingStatus::Idle);

    // The context (`:26-41`) — provided before the element so the parts subtree
    // reads it (upstream renders `element` INSIDE the Provider, `:41`).
    provide_avatar_root_context(AvatarRootContextValue {
        image_loading_status: status.clone(),
        set_image_loading_status: status.clone(),
    });

    // `const state = { imageLoadingStatus }` (`:22-24`).
    let state = AvatarRootState {
        image_loading_status: ImageLoadingStatus::Idle,
    };

    let rendered = avatar_root_element(state, &props);
    let (element, cleanup) = rendered.create_element();
    // The root has no dynamic DOM state (the status member is suppressed from
    // the DOM and the root body never re-derives), but the listener cleanup —
    // the ref fork's registration — belongs to this owner, not leaked forever.
    // The cleanup FnOnce crosses as a leaked Box (a wasm-rooted owner's
    // on_cleanup is effectively never-run-anywhere-else anyway).
    if let Some(cleanup) = cleanup {
        std::mem::forget(cleanup);
    }
    element
}

/// The `AvatarRoot` **view** props — the composition surface the docs pages
/// need ([`avatar_root_view`]). `children` is upstream's own prop
/// (`AvatarRoot.tsx:18` — `...elementProps`, which carries `children` through
/// `useRenderElement`'s props onto the span, `:34-39`); this struct is the
/// view-layer facade over the same `className`/`style`/`render`/`...elementProps`
/// vocabulary [`AvatarRootProps`] exposes to the engine.
pub struct AvatarRootViewProps {
    /// `className` (`AvatarRoot.tsx:18`, `:52`).
    pub class: Option<String>,
    /// `style` (`:18`) — ordered declarations.
    pub style: Vec<(String, String)>,
    /// The `...elementProps` rest's plain attributes (`:18`), merged last-but-one
    /// (the consumer wins over the internal values — here the internal bag is the
    /// empty `elementProps`, so they are the only source).
    pub element_attributes: Vec<(String, String)>,
    /// `render` (`:18`) — the engine's replacement-element union (the element
    /// form's tag is what the view renders, the callback form is resolved by
    /// [`avatar_root_element`] like it is for every other part).
    pub render: Option<RenderProp>,
    /// The forwarded `ref` (`:16` — `HTMLSpanElement`).
    pub ref_callback: Option<RefCallback<Element>>,
    /// `children` (`:18`) — the parts subtree (`Avatar.Image`/`Avatar.Fallback`)
    /// and/or the root's own text content; upstream renders both INSIDE the
    /// root span (`:41`).
    pub children: Option<Children>,
}

impl Default for AvatarRootViewProps {
    fn default() -> Self {
        Self {
            class: None,
            style: Vec::new(),
            element_attributes: Vec::new(),
            render: None,
            ref_callback: None,
            children: None,
        }
    }
}

/// `AvatarRoot` as a **composition view** — the missing surface `library: avatar`
/// was marked done without (the `AvatarRoot.tsx:41` contract: the root renders
/// its `children` INSIDE the provider, inside its own span). [`use_avatar_root`]
/// returns a bare materialized `span` with no place for children, which is why
/// the crate's own harness mounts the parts as siblings
/// (`avatar_tests.rs` `mount_avatar`) — fine for the machine contracts, but
/// unable to produce upstream's nesting, which every docs demo and every real
/// composition relies on.
///
/// Must be called inside a reactive owner (a component body or mount scope); the
/// children closure is invoked synchronously here, after
/// [`provide_avatar_root_context`], so a parts subtree built inside it resolves
/// the root the same way `Avatar.Image`/`Avatar.Fallback` do upstream. Because
/// the context is provided into the CURRENT owner (exactly as
/// [`use_avatar_root`] does), two siblings each rendering a root with parts must
/// be built in separate reactive scopes (or sequentially, parts first) — a
/// second root in the same scope re-provides the context for anything created
/// after it.
///
/// Documented adaptations (never silent), following the crate's view convention
/// (`checkbox_root_view`):
///
/// 1. The element is a real leptos markup node (`custom(rendered.tag)`, so a
///    `render` replacement element keeps its tag) whose `className`/`style`/
///    attribute bag is written by ONE commit effect — `view!`/`HtmlElement` has
///    no dynamic attribute spread, and the bag is dynamic (`update_element`, the
///    image re-render commit's own writer).
/// 2. The forwarded `ref` fires in that same commit, after the attributes and
///    after the children are in place — React's parent-ref-after-children commit
///    order. Its cleanup (the image's ref-fork contract) is leaked like every
///    other part's is (the view's lifetime owns the node).
/// 3. A runtime prop change is the caller's re-invocation (the meter/`Props are
///    static per body run` precedent); the view itself is not reactive over its
///    props.
pub fn avatar_root_view(props: AvatarRootViewProps) -> impl IntoView {
    let AvatarRootViewProps {
        class,
        style,
        element_attributes,
        render,
        ref_callback,
        children,
    } = props;

    // `const [imageLoadingStatus, setImageLoadingStatus] = React.useState('idle')`
    // + the provider (`AvatarRoot.tsx:20-41`) — the same single-signal pair
    // `use_avatar_root` publishes, provided BEFORE the children run so the parts
    // subtree reads it.
    let status: RwSignal<ImageLoadingStatus> = RwSignal::new(ImageLoadingStatus::Idle);
    provide_avatar_root_context(AvatarRootContextValue {
        image_loading_status: status.clone(),
        set_image_loading_status: status,
    });

    // `const element = useRenderElement('span', componentProps, …)` (`:34-39`) —
    // the SAME builder the engine path uses, so class/style/`render`/attribute
    // merge semantics are the engine's, not a view-side re-derivation. The
    // status member is suppressed from the DOM by the mapping, so the state
    // record's `'idle'` snapshot renders the identical DOM upstream's
    // per-status re-render does (the `use_avatar_root` note).
    let engine_props = AvatarRootProps {
        class_style: UseRenderElementComponentProps {
            class_name: class.map(ClassNameSource::Static),
            render,
            style: (!style.is_empty()).then_some(StyleSource::Static(style)),
        },
        element_attributes,
        // Fired by this view's commit effect (below), in React's
        // ref-after-children order, not by `create_element`.
        ref_callback: None,
    };
    let rendered = avatar_root_element(
        AvatarRootState {
            image_loading_status: ImageLoadingStatus::Idle,
        },
        &engine_props,
    );

    let tag = rendered.tag.clone();
    // `children` (`:41`) — invoked once, in this scope, after the provider.
    let children_view: Option<AnyView> = children.map(|children| children());
    let node_ref = NodeRef::<Custom<String>>::new();

    let rendered = SendWrapper::new(rendered);
    let ref_callback = SendWrapper::new(ref_callback);
    Effect::new(move |_| {
        let Some(node) = node_ref.get() else {
            return;
        };
        // The element's bag (class/style/`...elementProps`) — the React commit.
        update_element(&node, &rendered);
        if let Some(callback) = ref_callback.as_ref().as_ref() {
            let element: Element = node.clone().into();
            if let Some(cleanup) = callback(Some(&element)) {
                // The node outlives the mount; the cleanup is leaked with it
                // (the parts' own held-or-forgotten listener convention).
                std::mem::forget(cleanup);
            }
        }
    });

    custom(tag).node_ref(node_ref).child(children_view)
}
