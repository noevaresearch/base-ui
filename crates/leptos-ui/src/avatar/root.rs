//! `AvatarRoot` — `packages/react/src/avatar/root/AvatarRoot.tsx` (the whole
//! component body) plus `avatarStateAttributesMapping`
//! (`root/stateAttributesMapping.ts:1-3`) and the state/props types.
//!
//! Split into the pure builder [`avatar_root_element`] (host-testable, the
//! `separator_element`/`button_element` facade convention) and the full body
//! [`use_avatar_root`] (the context provision + materialization; wasm-only).

use std::rc::Rc;

use leptos::prelude::RwSignal;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::StateAttributeProps;
use leptos_ui_internals::use_render_element::{
    RenderElementProps, RenderedElement, UseRenderElementComponentProps, UseRenderElementParams,
    static_attr, use_render_element,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use web_sys::Element;

use super::context::{
    AvatarRootContextValue, ImageLoadingStatus, provide_avatar_root_context,
};

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

/// `useRenderElement('span', …)` (`AvatarRoot.tsx:34-39`) as a pure function —
/// the suppression mapping, the plain `elementProps` bag. The description the
/// caller materializes.
pub fn avatar_root_element(
    state: AvatarRootState,
    props: &AvatarRootProps,
) -> RenderedElement {
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
        props.class_style.clone(),
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
    if let Some(cleanup) = cleanup {
        let cleanup = send_wrapper::SendWrapper::new(cleanup);
        leptos::prelude::on_cleanup(move || (*cleanup)());
    }
    element
}
