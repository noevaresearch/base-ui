//! `AvatarRoot` — `AvatarRoot.tsx` (the whole component body) plus the
//! `avatarStateAttributesMapping` (`root/stateAttributesMapping.ts:1-3`) and the
//! state/props types.

use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Set;

use leptos_ui_internals::state_attributes::StateAttributeProps;
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use web_sys::Element;

use super::context::{
    AvatarRootContextValue, ImageLoadingStatus, provide_avatar_root_context,
};

/// `avatarStateAttributesMapping` (`stateAttributesMapping.ts:1-3`): `imageLoadingStatus`
/// maps to `null` on every part — no generic status attribute ever leaks to the DOM
/// (implementation.md "State → attribute mapping"); the transition key (Image's extra
/// mapping) is *not* handled here — the root's only state member is
/// `imageLoadingStatus` (`AvatarRoot.tsx:22-24`).
pub fn avatar_state_attributes_mapping<'a>()
-> impl Fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>> + 'a {
    |_key, _value| {
        if _key == "imageLoadingStatus" {
            // The suppression: `imageLoadingStatus: () => null` (`:2`).
            Some(None)
        } else {
            // No mapping for this key — the default walk applies (a no-op here: the
            // root state has no other members).
            None
        }
    }
}

/// `AvatarRootState` (`AvatarRoot.tsx:46-51`) — the one member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarRootState {
    /// `imageLoadingStatus` (`:50`).
    pub image_loading_status: ImageLoadingStatus,
}

impl AvatarRootState {
    /// The state record `useRenderElement`'s custom mapping consumes
    /// (`AvatarRoot.tsx:22-24`). The suppression mapping declines the only member, so
    /// the walk emits nothing — carried as the record for exactness.
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
/// body-time facade: `className`/`style` through the engine's component-props
/// vocabulary, arbitrary DOM props as `(name, value)` pairs (the `...elementProps`
/// rest), the forwarded ref.
pub struct AvatarRootProps {
    /// `className`/`style` (`:18`) — the engine's class/style resolution.
    pub class_style: leptos_ui_internals::use_render_element::UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:18`) — spread onto the element.
    pub element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:16` — `HTMLSpanElement`; behavior.md *DOM structure*:
    /// the `refInstanceof: window.HTMLSpanElement` conformance).
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

/// `AvatarRoot` (`AvatarRoot.tsx:14-42`) — owns the root-mirrored status
/// (`React.useState('idle')`, `:20`), publishes the context (`:26-41`), and renders
/// the `span` inside the provider (`:41`). Returns the materialized root element;
/// must be called inside a reactive owner.
///
/// The context value's two members are the same signal read two ways (upstream's
/// `useMemo` over `[imageLoadingStatus, setImageLoadingStatus]`, `:26-32`): Fallback
/// reads `image_loading_status`, Image writes through `set_image_loading_status` —
/// in the port they are the *same* `RwSignal`, so the mirror is exact: one write,
/// two observable members, no staleness window.
pub fn use_avatar_root(props: AvatarRootProps) -> web_sys::Element {
    // `const [imageLoadingStatus, setImageLoadingStatus] = React.useState('idle')`
    // (`:20`) — the root-mirrored status; `RwSignal` is both members of the context.
    let status = RwSignal::new(ImageLoadingStatus::Idle);

    // The context (`:26-41`) — provided before the element so the parts subtree reads
    // it (upstream renders `element` INSIDE the Provider, `:41`).
    provide_avatar_root_context(AvatarRootContextValue {
        image_loading_status: status.clone(),
        set_image_loading_status: status.clone(),
    });

    // `const state = { imageLoadingStatus }` (`:22-24`).
    let state = AvatarRootState {
        image_loading_status: ImageLoadingStatus::Idle,
    };

    // `useRenderElement('span', …)` (`:34-39`) — the suppression mapping, the plain
    // `elementProps` bag.
    let state_map = state.to_state_map();
    let mapping = avatar_state_attributes_mapping();
    let mut intrinsic =
        leptos_ui_internals::use_render_element::RenderElementProps::default();
    for (name, value) in &props.element_attributes {
        intrinsic
            .handlers
            .attributes
            .push((
                name.clone(),
                leptos_ui_internals::use_render_element::static_attr(value.clone()),
            ));
    }
    let props_bags = vec![leptos_ui_internals::merge_props::PropsSource::Static(
        intrinsic,
    )];
    let refs: Vec<InputRef<Element>> = match props.ref_callback {
        Some(callback) => vec![InputRef::Callback(callback)],
        None => Vec::new(),
    };

    let rendered = leptos_ui_internals::use_render_element::use_render_element(
        "span",
        props.class_style,
        leptos_ui_internals::use_render_element::UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: Some(&mapping),
        },
    )
    .expect("the AvatarRoot renders unconditionally");

    let (element, _cleanup) = rendered.create_element();
    // The root has no dynamic state on this surface (the status member is suppressed
    // from the DOM and the root body never re-derives), so the listener cleanup is
    // forgotten per the harness convention; the element rides the caller's tree.
    element
}

/// Writes the root status — the context setter from Image's side. `Set` on the same
/// signal the context carries.
pub(crate) fn set_root_status(context: &AvatarRootContextValue, status: ImageLoadingStatus) {
    reactive_graph::traits::Set::set(&context.set_image_loading_status, status);
}

/// The status read the Fallback gate uses (tracked).
pub(crate) fn root_status(
    context: &AvatarRootContextValue,
) -> reactive_graph::signal::RwSignal<ImageLoadingStatus> {
    context.image_loading_status.clone()
}

// Silence the unused-set warning on non-test builds where only Image writes.
#[allow(unused)]
fn _assert_set_available<T>(signal: &RwSignal<T>, value: T)
where
    T: PartialEq,
{
    Set::set(signal, value);
}
