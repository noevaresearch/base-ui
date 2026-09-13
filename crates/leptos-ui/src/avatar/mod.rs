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
//!   user's `onLoadingStatusChange` and the root, and the unmount-only cleanup resets
//!   the root to `'idle'` (`:131-133`) — the whole mechanism behind "unmounting a
//!   loaded image makes the fallback reappear" (behavior.md *Edge cases*).
//! - **The probe (default mode)**: with `enabled = !keepMounted`, a detached
//!   `window.Image()` carries the load — `'loading'` set synchronously, request-config
//!   props applied (`referrerPolicy`/`crossOrigin ?? null`/`sizes`/`srcset`/`src`, in
//!   that order), then the cached fast path resolves immediately from
//!   `complete`/`naturalWidth` (`useImageLoadingStatus.ts:33-63`); no `src`/`srcSet`
//!   short-circuits to `'error'` without constructing a probe (`:27-30`); late events
//!   after teardown are guarded by the `isMounted` flag (`:35-41`, `:65-67`). The reset
//!   on source-config change falls out of the effect's dependency array (`:68`).
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
//! - **Canonical status lives in rg-0.2 signals** — the machinery's runtime (the
//!   internals' hooks are rg-0.2). The probe scheduling effect, the fan-out, the
//!   loaded derivation, and the transition-status bridge all read/write them on the
//!   rg side; the views track the *leptos mirrors* built with the field unit's
//!   [`mirror_rg_to_leptos`] pattern (`field/validation_helpers.rs` — the
//!   dual-runtime law: a leptos attribute closure never tracks an rg read directly).
//! - **The probe is effect-scheduled over a pluggable factory** — upstream constructs
//!   `new window.Image()` inside the layout effect; the port's
//!   [`use_avatar_image`] takes an optional [`ProbeFactory`] so the wasm suite can
//!   inject the deterministic fake (the upstream `window.Image` stub,
//!   `AvatarImage.test.tsx:27-73`) while the default factory builds a real
//!   `HtmlImageElement` via `document().create_element("img")`. The scheduling
//!   effect's re-run on source-config change (`:68`'s dependency array) stands in
//!   for the effect-restart: each run tears down the previous probe (the
//!   `isMounted` guard) before starting the next.
//! - **`onLoad`/`onError` ride the element seam.** The engine's handler bag has no
//!   `load`/`error` slots (it mirrors `useRenderElement`'s
//!   `React.HTMLAttributes` delegation set); the keepMounted listeners attach at
//!   the materialization seam — the ref-fork callback, which is this unit's
//!   layout-effect analog — as a chain: the user's handler first (the
//!   [`BaseUIEvent`]-wrapped dispatch, `prevent_base_ui_handler()` cancels the
//!   internal update, the mergeProps.ts:221-274 contract), then the internal status
//!   write unless prevented.
//! - **Dynamic parts re-materialize through the engine.** `create_element`
//!   evaluates attributes once (a snapshot), so the Image/Fallback views wrap the
//!   engine description in the established rebuild convention: a tracked leptos
//!   read (the status/mounted mirrors) re-invokes the description builder and
//!   materializes the fresh element on every state change — the
//!   progress-hero/`view_dynamic_children` precedent. Attributes are therefore
//!   always current at each materialization; the churn is the port's accepted
//!   re-render analog.
//! - **Props are static per body run** (the meter precedent): React re-runs the
//!   body on prop change; Leptos runs it once, so runtime prop *changes* (a
//!   different `delay`, a new `src`) are the caller's rebuild — the delay-lifecycle
//!   and src-swap edges that upstream covers by re-running the body resolve here by
//!   re-invoking the component. The source-config probe restart (dep-array
//!   behavior) is still faithfully modeled *within* one body via the scheduling
//!   effect.
//! - **The render prop** rides the engine's [`RenderProp`] union on Image (the
//!   element and callback forms; the callback receives the fully merged bag —
//!   behavior.md *Source-props ordering* pins the merged order, which the engine's
//!   three-bag fold produces); Root/Fallback accept `class`/`style` through
//!   [`UseRenderElementComponentProps`]. `render` on the root/fallback facades is
//!   the docs-layer concern (the meter note).
//! - The missing-context error reproduces upstream's `'Base UI:
//!   AvatarRootContext is missing. Avatar parts must be placed within
//!   <Avatar.Root>.'` panic (`AvatarRootContext.ts:12-19`) through the required
//!   accessor (the meter `use_meter_root_context` precedent) — pinned host-side (a
//!   wasm panic is an uncatchable trap).
//! - The discarded `'use client'` directives are N/A. No focus, no keyboard, no
//!   portal (behavior.md — all N/A sections; implementation.md "Explicitly not
//!   used": floating-ui-react and use-render).

mod context;
mod fallback;
mod image;
mod root;

pub use context::{
    AvatarRootContextValue, ImageLoadingStatus, provide_avatar_root_context,
    use_avatar_root_context,
};
pub use fallback::{
    AvatarFallbackProps, avatar_fallback_element, avatar_fallback_state, use_avatar_fallback,
};
pub use image::{
    AvatarImageProps, AvatarImageState, Probe, ProbeConfig, ProbeFactory, avatar_image_element,
    avatar_image_state_attributes_mapping, use_avatar_image,
};
pub use root::{AvatarRootProps, avatar_root_element, use_avatar_root};
