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

pub use context::{
    AvatarRootContext, AvatarRootContextValue, ImageLoadingStatus, provide_avatar_root_context,
    use_avatar_root_context,
};
pub use fallback::{
    AvatarFallbackProps, UseAvatarFallback, avatar_fallback_element, avatar_fallback_state,
    use_avatar_fallback,
};
pub use image::{
    AvatarImageProps, AvatarImageStateSnapshot, Probe, ProbeConfig, ProbeFactory, RealProbe,
    UseAvatarImage, LOADING_DATA_ATTR, ERROR_DATA_ATTR, avatar_image_element,
    avatar_image_state_attributes_mapping, should_render, use_avatar_image,
    with_probe_factory,
};
pub use root::{
    AvatarRootProps, AvatarRootState, avatar_root_element, avatar_state_attributes_mapping,
    use_avatar_root,
};
pub use views::{AvatarDocView, avatar_fallback_view, avatar_image_view};
