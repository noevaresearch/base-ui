//! `AvatarFallback` — `packages/react/src/avatar/fallback/AvatarFallback.tsx`
//! (the whole component) plus the delay latch machine.
//!
//! The fallback renders when the root status is not `'loaded'`, gated by the
//! delay latch. View reactivity: the leptos status signal is tracked by the
//! dynamic-view rebuild; the delay latch is an rg signal in the machinery
//! owner, seeded by the effect and mirrored once. The `enabled: false` arm is
//! upstream's `useRenderElement` `enabled` gate returning `null` — the view
//! renders nothing.

use std::rc::Rc;

use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get as RgGet, Set as RgSet};
use send_wrapper::SendWrapper;
// The leptos-mirror writes use the fully qualified `leptos::prelude::Set`
// (the mirror signals are leptos-runtime; the rg `Set` trait does not apply).

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::StateAttributeProps;
use leptos_ui_internals::use_render_element::{
    RenderElementProps, RenderedElement, UseRenderElementComponentProps, UseRenderElementParams,
    static_attr, use_render_element,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};
use web_sys::Element;

use super::context::{AvatarRootContextValue, ImageLoadingStatus, use_avatar_root_context};
use super::root::{AvatarRootState, avatar_state_attributes_mapping};

/// `AvatarFallbackState` (`AvatarFallback.tsx:51`) — the root state.
pub type AvatarFallbackState = AvatarRootState;

/// The `AvatarFallback` props (`AvatarFallbackProps`, `AvatarFallback.tsx:53`).
pub struct AvatarFallbackProps {
    /// `className`/`style` (`:20`).
    pub class_style: UseRenderElementComponentProps,
    /// `delay` (`:22`, default `0`) — how long to wait before showing the
    /// fallback, in milliseconds.
    pub delay: f64,
    /// The `...elementProps` rest (`:20`).
    pub element_attributes: Vec<(String, String)>,
    /// The fallback's content (`:41`'s element children — behavior.md *Public
    /// API surface*: "children — rendered text content",
    /// `AvatarFallback.test.tsx:53-66`). Rendered as the element's HTML
    /// content at materialization (the engine's `dangerouslySetInnerHTML`
    /// slot — the PrehydrationScript convention).
    pub inner_html: Option<String>,
    /// The forwarded `ref` (`:19` — `HTMLSpanElement`).
    pub ref_callback: Option<RefCallback<Element>>,
}

impl Default for AvatarFallbackProps {
    fn default() -> Self {
        Self {
            class_style: Default::default(),
            delay: 0.0,
            element_attributes: Vec::new(),
            inner_html: None,
            ref_callback: None,
        }
    }
}

/// The delay latch — `delayPassed` (`AvatarFallback.tsx:23`): seeded
/// `delay === 0` and only ever moved `false → true` (nothing ever writes
/// `setDelayPassed(false)` — the "once visible, never re-hidden" semantics,
/// behavior.md *delay lifecycle* fall out of the one-directional state).
/// Seeded/started under the machinery owner (the dual-runtime law).
///
/// Returns `(latch, mirror)` — the latch is the machinery-side truth; the
/// mirror is the leptos signal the views track.
pub(crate) fn use_avatar_fallback_delay_latch(
    delay: f64,
) -> (
    super::image::LocalRwSignal<bool>,
    leptos::prelude::RwSignal<bool>,
) {
    // `React.useState(delay === 0)` (`:23`) — `delay={0}` renders
    // synchronously on mount.
    let latch = RgRwSignal::new_local(delay == 0.0);
    let mirror: leptos::prelude::RwSignal<bool> = leptos::prelude::RwSignal::new(delay == 0.0);

    // The plain `useEffect` (`:26-35`) — NOT a layout effect (delay timing is
    // user-perceived, not pre-paint-critical, `:27-28`). The rg effect's
    // first run is the mount pass.
    reactive_graph::effect::Effect::new({
        let latch = latch.clone();
        let mirror = mirror.clone();
        move |_| {
            // `if (delay > 0) { timeout.start(delay, () =>
            // setDelayPassed(true)); }` (`:27-29`) — the ported `useTimeout`
            // (per repo convention, not `window.setTimeout`).
            let timeout = leptos_ui_utils::use_timeout::use_timeout();
            if delay > 0.0 {
                timeout.start(delay as u32, {
                    let latch = latch.clone();
                    let mirror = mirror.clone();
                    move || {
                        RgSet::set(&latch, true);
                        leptos::prelude::Set::set(&mirror, true);
                    }
                });
            } else {
                // `else { setDelayPassed(true); }` (`:30-33`) — once shown
                // without a delay, keep it visible (a later change from no
                // delay to a number would re-hide an already-visible
                // fallback upstream; the port's static-prop adaptation makes
                // a delay change the caller's rebuild anyway).
                RgSet::set(&latch, true);
                leptos::prelude::Set::set(&mirror, true);
            }
            // `return timeout.clear` (`:34`) — the effect cleanup. The
            // effect's per-run cleanup rides `on_cleanup` inside the
            // machinery owner (the per-rerun cleanup order).
            let timeout = SendWrapper::new(timeout);
            reactive_graph::owner::on_cleanup(move || timeout.clear());
        }
    });

    (latch, mirror)
}

/// The `enabled` gate (`:46`): `imageLoadingStatus !== 'loaded' && (delay ===
/// 0 || delayPassed)` — the render gate of the fallback. The `delay === 0`
/// arm ORs in directly, so changing a pending delay to `0` shows the
/// fallback on that same render rather than waiting an effect cycle (`:47-49`).
pub fn avatar_fallback_state(
    image_loading_status: ImageLoadingStatus,
    delay: f64,
    delay_passed: bool,
) -> bool {
    image_loading_status != ImageLoadingStatus::Loaded && (delay == 0.0 || delay_passed)
}

/// Builds the fallback `<span>` element description — upstream's
/// `useRenderElement('span', …)` call (`AvatarFallback.tsx:41-47`) as a pure
/// function. `None` is the `enabled: false` gate (`:46-47`).
pub fn avatar_fallback_element(
    state: AvatarFallbackState,
    enabled: bool,
    props: &AvatarFallbackProps,
) -> Option<RenderedElement> {
    if !enabled {
        return None;
    }

    let state_map = state.to_state_map();
    let mapping = avatar_state_attributes_mapping();

    let mut intrinsic = RenderElementProps::default();
    for (name, value) in &props.element_attributes {
        intrinsic
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }
    // The children (`:41`'s element children) — the engine's content slot.
    if let Some(inner_html) = &props.inner_html {
        intrinsic.inner_html = Some(inner_html.clone());
    }
    let props_bags = vec![PropsSource::Static(intrinsic)];
    let refs: Vec<InputRef<Element>> = match &props.ref_callback {
        Some(callback) => vec![InputRef::Callback(Rc::clone(callback))],
        None => Vec::new(),
    };

    use_render_element(
        "span",
        super::root::clone_class_style(&props.class_style),
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: Some(&mapping),
        },
    )
    .into()
}

/// The `AvatarFallback` body (`AvatarFallback.tsx:18-49`) — the live part.
/// Must be called inside a reactive owner, under an `AvatarRoot`.
///
/// - `imageLoadingStatus` reads the root signal (the context read, `:22`).
/// - The delay latch runs its effect under the machinery owner (the
///   dual-runtime law) and mirrors to leptos for the views.
/// - Returns the leptos mirror signals the dynamic-view rebuild tracks.
pub struct UseAvatarFallback {
    /// The root status signal (the context member, read tracked by the view).
    pub root_status: leptos::prelude::RwSignal<ImageLoadingStatus>,
    /// The delay-latch mirror (leptos) — `delayPassed`.
    pub delay_mirror: leptos::prelude::RwSignal<bool>,
    /// The `delay` prop (static per body run — a change is the caller's
    /// rebuild, the meter/toggle convention).
    pub delay: f64,
}

pub fn use_avatar_fallback(props: &AvatarFallbackProps) -> UseAvatarFallback {
    // `const { imageLoadingStatus } = useAvatarRootContext()` (`:22`).
    let context: AvatarRootContextValue = use_avatar_root_context();

    // The delay latch — the machinery owner pattern (the image module).
    let latch_owner = reactive_graph::owner::Owner::new();
    let (_latch, delay_mirror) = latch_owner.with(|| use_avatar_fallback_delay_latch(props.delay));
    std::mem::forget(latch_owner);

    UseAvatarFallback {
        root_status: context.image_loading_status.clone(),
        delay_mirror,
        delay: props.delay,
    }
}
