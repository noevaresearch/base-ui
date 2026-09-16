//! Port of the Base UI Avatar — the `library: avatar` TODO item
//! (`specs/library/avatar/behavior.md`, `specs/library/avatar/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **Two-tier status machine** ("State machine / hooks used"): `useImageLoadingStatus`
//!   (`useImageLoadingStatus.ts:14-71`) owns the canonical image-local status
//!   (`'idle' | 'loading' | 'loaded' | 'error'`, seeded `'idle'`); `AvatarRoot` mirrors
//!   it through `AvatarRootContext` (`AvatarRoot.tsx:20`, `AvatarRootContext.ts:5-8`)
//!   and `AvatarImage` is the only writer — the fan-out
//!   (`AvatarImage.tsx:120-129`) reports every status *except* `'idle'` to both the
//!   user's `onLoadingStatusChange` and the root, and the unmount cleanup resets
//!   the root to `'idle'` (`:131-133`) — the whole mechanism behind "unmounting a
//!   loaded image makes the fallback reappear" (behavior.md *Edge cases*).
//! - **The probe (default mode)**: with `enabled = !keepMounted`, a detached
//!   `window.Image()` carries the load — `'loading'` set synchronously, request-config
//!   props applied (`referrerPolicy`/`crossOrigin ?? null`/`sizes`/`srcset`/`src`, in
//!   that order), then the cached fast path resolves immediately from
//!   `complete`/`naturalWidth` (`useImageLoadingStatus.ts:33-63`); no `src`/`srcSet`
//!   short-circuits to `'error'` without constructing a probe (`:27-30`); late events
//!   after teardown are guarded by the `isMounted` flag (`:35-41`, `:65-67`). The reset
//!   on source-config change falls out of the effect's dependency array (`:68`) —
//!   the port's scheduling effect re-runs on the serialized source-config signal
//!   (the dep-array analog), draining the probe slot first.
//! - **keepMounted in-place sync**: the probe is disabled and the status is read off
//!   the rendered `<img>` — not `complete` → `'loading'`, `complete` →
//!   `'loaded'`/`'error'` from `naturalWidth` (`AvatarImage.tsx:61-99`); a dropped
//!   (never-fired) ref bails without overwriting anything (`:69-74`); the first
//!   commit with an already-loaded element pre-seeds `mounted` so the enter animation
//!   is not replayed (`:84-88`).
//! - **Presence + transition**: `useTransitionStatus(isVisible)` with `isVisible =
//!   status === 'loaded'` (defaults: no idle state, no deferred ending), and
//!   `shouldRender = keepMounted || mounted` (`:53`, `:153`, `:171`) — the element
//!   exists only while `mounted` in default mode. `useOpenChangeComplete` scoped to
//!   the closing direction (`enabled: !isVisible`) defers `setMounted(false)` until
//!   exit animations finish (`:135-144`).
//! - **keepMounted's state attributes**: `data-loading`/`data-error` while not
//!   displayable and `aria-hidden` until loaded, scoped to keepMounted mode so the
//!   default mode's exiting-but-loaded element can't pick them up (`:101-118`); the
//!   element's own `onLoad`/`onError` update the status (`:111-116`). The
//!   `transitionStatus: keepMounted && 'ending' ? undefined : transitionStatus` mask
//!   (`:150`) keeps the never-unmounting element from playing a reversing exit.
//! - **Bag order is the compatibility shim** (`:154-169`):
//!   `[renderedStatusProps, elementProps, sourceProps]` merges right-to-left
//!   later-wins, so `loading`/`sizes`/`srcSet` land before `src` (the Safari/Firefox
//!   fetch-order note at `:37-44`), user `elementProps` override the internal status
//!   attributes (an explicit `aria-hidden` survives — behavior.md *Accessibility*),
//!   and chained event handlers run the user's first with
//!   `preventBaseUIHandler()` cancelling the internal update (behavior.md *Events*).
//! - **State-attribute suppression** (`stateAttributesMapping.ts:1-3`): every part
//!   maps `imageLoadingStatus` to `null` — no generic status attribute ever leaks;
//!   the image's mapping adds `transitionStatusMapping`
//!   (`AvatarImage.tsx:16-19`), so its public attribute surface is exactly
//!   `data-loading`/`data-error` (keepMounted) plus the transition attributes.
//! - **Fallback delay latch** (`AvatarFallback.tsx:20-47`): `delayPassed` seeds
//!   `delay === 0` and only ever moves `false → true`; `delay > 0` starts the
//!   (ported) `useTimeout`; the enabled gate is
//!   `imageLoadingStatus !== 'loaded' && (delay === 0 || delayPassed)`.
//!
//! ## Rust adaptations
//!
//! - **Dual-runtime law** (the field-bridge/direction-provider documented trap):
//!   the machinery (probe effect, fan-out, transition hook, `useOpenChangeComplete`)
//!   runs under a dedicated **rg-0.2 owner** — its effects must be scoped to an rg
//!   owner, not the ambient leptos one. The views track **leptos mirrors** built
//!   with lockstep rg→leptos effects (`transition_status_signal` precedent). The
//!   root status is a leptos signal (the fallback gate reads it in the view tree);
//!   the fan-out writes it through a lockstep effect.
//! - **The unmount reset is a leptos-side `on_cleanup`** (the context signal is
//!   leptos-runtime; an rg-0.2 `on_cleanup` under the leptos mount owner would
//!   never fire). Also drains the probe slot (the `isMounted` teardown).
//! - **The probe is the default factory + scoped override** (`with_probe_factory`):
//!   upstream constructs `new window.Image()` inside the layout effect; the port's
//!   factory is injectable so the wasm suite drives the deterministic fake (the
//!   upstream `window.Image` stub, `AvatarImage.test.tsx:27-73`). A real probe is a
//!   detached `document.createElement("img")` (behaviorally equivalent for load
//!   probing). The slot drain at each effect run / unmount is the upstream
//!   cleanup's `isMounted` flip (dropping the closures unsets the IDL attributes —
//!   late events disconnected).
//! - **`onLoad`/`onError` ride the engine's handler slots** — the materialization
//!   seam attaches them as real listeners; the merge contract (the user's handler
//!   first, `prevent_base_ui_handler()` cancels the internal update,
//!   mergeProps.ts:221-274) is the engine's own tested behavior.
//! - **Dynamic parts re-materialize through the engine** — the image/fallback
//!   views wrap the element builder in the dynamic-view-child convention (the
//!   progress-hero precedent): tracked leptos mirror reads re-invoke the builder
//!   and replace the node in place. Attributes are always current at each
//!   materialization; the churn is the port's re-render analog. `None` renders
//!   nothing (the upstream `null`), with no wrapper element.
//! - **Props are static per body run** (the meter precedent): a runtime prop change
//!   (a different `delay`, a new `src`) is the caller's rebuild — re-invoking the
//!   component/hook re-runs the body, re-seeds the config signal, and the
//!   scheduling effect restarts the probe: upstream's dep-array restart.
//! - The render prop rides the engine's `RenderProp` union (element and callback
//!   forms). The missing-context error reproduces upstream's `'Base UI:
//!   AvatarRootContext is missing. Avatar parts must be placed within
//!   <Avatar.Root>.'` panic (`AvatarRootContext.ts:12-19`) through the required
//!   accessor (the meter precedent).
//! - The discarded `'use client'` directives are N/A. No focus, no keyboard, no
//!   portal (behavior.md — all N/A sections; implementation.md "Explicitly not
//!   used": floating-ui-react and use-render).

mod context;
mod fallback;
mod image;
mod root;
mod views;

// The element-bag writer (`views::update_element`) is the crate's ONE implementation of
// "replay a `RenderedElement`'s merged bag onto a live node", used by the view layers that
// materialize their own leptos node rather than a `RenderedElement` description
// (`avatar_root_view`, and the otp-field parts). Re-exported at the module so those callers do not
// need `views` public.
pub(crate) use views::update_element;

pub use context::{
    AvatarRootContext, AvatarRootContextValue, ImageLoadingStatus, provide_avatar_root_context,
    use_avatar_root_context,
};
pub use fallback::{
    AvatarFallbackProps, UseAvatarFallback, avatar_fallback_element, avatar_fallback_state,
    use_avatar_fallback,
};
#[cfg(test)]
pub use image::source_config_key_for_tests;
pub use image::{
    AvatarImageProps, AvatarImageStateSnapshot, ERROR_DATA_ATTR, LOADING_DATA_ATTR, LocalRwSignal,
    Probe, ProbeConfig, ProbeFactory, RealProbe, UseAvatarImage, avatar_image_element,
    avatar_image_state_attributes_mapping, should_render, use_avatar_image, with_probe_factory,
};
pub use root::{
    AvatarRootProps, AvatarRootState, AvatarRootViewProps, avatar_root_element,
    avatar_root_view, avatar_state_attributes_mapping, use_avatar_root,
};
pub use views::{AvatarDocView, avatar_fallback_view, avatar_image_view};

// ---------------------------------------------------------------------------
// The namespaced part surface (`Avatar::Root`, `Avatar::Image`, `Avatar::Fallback`)
// ---------------------------------------------------------------------------
//
// Upstream teaches `<Avatar.Root><Avatar.Image /><Avatar.Fallback /></Avatar.Root>`; this port's
// spelling is the same tree with Rust's path separator (`specs/docs-content/CONTRACT.md`, the
// React→Rust mapping table; the macro-level pin is `crates/leptos-ui/tests/ns_component_path.rs`).
// `Avatar` documents three parts (behavior.md "Public API surface": Root, Image, Fallback) and all
// three were, until now, reachable only as view functions — `avatar_root_view(..)`,
// `avatar_image_view(handle, ..)`, `avatar_fallback_view(handle, ..)` — behaviour without the
// ergonomics (`check-part-surface.mjs`: "3 exist only in the flattened form").
//
// Each component below is the composition surface over those functions: it builds the part's
// engine props from upstream's own prop names, calls the part's hook in the component body, and
// renders the part's dynamic view through [`dynamic_part_view`] — no element, attribute or handler
// logic is duplicated. The `avatar_root_view`/`avatar_*_view` functions and their handles stay for
// callers that drive the parts themselves (the docs page and the wasm suite do).
//
// `Av.Root`'s children are invoked ONCE, synchronously, inside the provider (see
// [`AvatarRootViewProps`]) — upstream's parts subtree, built in the same reactive scope.

use leptos::children::Children;
use leptos::prelude::*;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderProp, StyleSource, UseRenderElementComponentProps,
};
use leptos_ui_utils::use_merged_refs::RefCallback;

use crate::avatar::image::{OnLoadingStatusChange, UserImageEventHandler};
use crate::avatar::views::dynamic_part_view;

/// The engine's `className`/`style`/`render` bag from the port's attribute-level spelling of them
/// (`className` -> `class`, the ordered `style` declarations, the `render` union).
fn class_style_bag(
    class: Option<String>,
    style: Vec<(String, String)>,
    render: Option<RenderProp>,
) -> UseRenderElementComponentProps {
    UseRenderElementComponentProps {
        class_name: class.map(ClassNameSource::Static),
        render,
        // `None` when the caller declared no style: upstream's `style === undefined` (the engine
        // distinguishes "no style" from "an empty style record").
        style: (!style.is_empty()).then_some(StyleSource::Static(style)),
    }
}

/// `Avatar.Root` — upstream's `<Avatar.Root>` (`AvatarRoot.tsx`), the provider-wrapped span.
#[allow(non_snake_case)]
#[component]
pub fn Root(
    /// `className` (`AvatarRoot.tsx:18`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` (`:18`) — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest's plain attributes (`:18`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// `render` (`:18`) — the element/callback replacement union.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// The forwarded `ref` (`:16`).
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<web_sys::Element>>,
    /// The parts subtree (`Avatar.Image` / `Avatar.Fallback`), rendered INSIDE the root span.
    children: Children,
) -> impl IntoView {
    avatar_root_view(AvatarRootViewProps {
        class,
        style,
        element_attributes,
        render,
        ref_callback,
        children: Some(children),
    })
}

/// `Avatar.Image` — upstream's `<Avatar.Image>` (`AvatarImage.tsx`).
///
/// The image is mounted through [`use_avatar_image`] and rendered by [`avatar_image_view`]: the
/// element exists only while the part is displayable (`shouldRender`), and each re-render
/// materializes it.
#[allow(non_snake_case)]
#[component]
pub fn Image(
    /// `src` (`:42` — a `sourceProps` member, applied last).
    #[prop(default = None, optional)]
    src: Option<String>,
    /// `sizes` (`:40` — `sourceProps`, applied before `src`).
    #[prop(default = None, optional)]
    sizes: Option<String>,
    /// `srcSet` (`:41` — `sourceProps`).
    #[prop(default = None, optional)]
    src_set: Option<String>,
    /// `className` (`:31-44` destructuring).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style`.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// `render` (`:31-44`) — the element or callback form.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// The `...elementProps` rest (the MIDDLE bag: beats the internal status attributes, loses to
    /// `sourceProps` on `src`/`sizes`/`srcSet`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// `keepMounted` (`:202`, default `false`).
    #[prop(default = false, optional)]
    keep_mounted: bool,
    /// The user's `onLoad` (`:111-113`).
    #[prop(default = None, optional)]
    on_load: Option<UserImageEventHandler>,
    /// The user's `onError` (`:114-116`).
    #[prop(default = None, optional)]
    on_error: Option<UserImageEventHandler>,
    /// `onLoadingStatusChange` (`:196`).
    #[prop(default = None, optional)]
    on_loading_status_change: Option<OnLoadingStatusChange>,
    /// The forwarded `ref` (`:29`).
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<web_sys::Element>>,
) -> impl IntoView {
    let props = AvatarImageProps {
        class_style: class_style_bag(class, style, render),
        element_attributes,
        on_load,
        on_error,
        on_loading_status_change,
        keep_mounted,
        sizes,
        src_set,
        src,
        ref_callback,
    };
    let handle = use_avatar_image(&props);
    dynamic_part_view(avatar_image_view(handle, props))
}

/// `Avatar.Fallback` — upstream's `<Avatar.Fallback>` (`AvatarFallback.tsx`), shown while the image
/// is not displayable (after `delay`).
#[allow(non_snake_case)]
#[component]
pub fn Fallback(
    /// `delay` (`:22`, default `0`) — how long to wait before showing the fallback, in ms.
    #[prop(default = 0.0, optional)]
    delay: f64,
    /// `className` (`:20`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style`.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// `render`.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// The `...elementProps` rest (`:20`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The fallback's content — upstream's element children (`:41`; behavior.md "Public API
    /// surface": "children — rendered text content"). The engine writes it as the element's HTML
    /// content at materialization (its `dangerouslySetInnerHTML` slot), so it is spelled as the
    /// inner HTML here.
    #[prop(default = None, optional)]
    inner_html: Option<String>,
    /// The forwarded `ref` (`:19`).
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<web_sys::Element>>,
) -> impl IntoView {
    let props = AvatarFallbackProps {
        class_style: class_style_bag(class, style, render),
        element_attributes,
        inner_html,
        delay,
        ref_callback,
    };
    let handle = use_avatar_fallback(&props);
    dynamic_part_view(avatar_fallback_view(handle, props))
}
