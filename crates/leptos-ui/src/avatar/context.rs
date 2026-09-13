//! The Avatar context — `packages/react/src/avatar/root/AvatarRootContext.ts` (the
//! whole file) plus the `ImageLoadingStatus` union
//! (`packages/react/src/avatar/root/AvatarRoot.tsx:44`).
//!
//! Rust adaptation (the dual-runtime law this workspace's docs pages
//! document): the canonical status signal is a **leptos** `RwSignal` — the
//! runtime the views track — while the image machinery bridges it from the
//! rg-0.2 side with lockstep effects (the field `transition_status_signal`
//! precedent). A single signal per member (upstream's `imageLoadingStatus`
//! and `setImageLoadingStatus` are the same `React.useState` pair read two
//! ways; the meter root's single-signal `setLabelId` precedent).

use leptos::prelude::RwSignal;
use send_wrapper::SendWrapper;

/// `ImageLoadingStatus` (`AvatarRoot.tsx:44`) — `'idle' | 'loading' | 'loaded' |
/// 'error'`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLoadingStatus {
    /// `'idle'` — the seed, and the root's reset value when the image unmounts.
    Idle,
    /// `'loading'`.
    Loading,
    /// `'loaded'`.
    Loaded,
    /// `'error'`.
    Error,
}

impl ImageLoadingStatus {
    /// The string the fan-out reports to `onLoadingStatusChange`
    /// (behavior.md *Events*: "a single string status argument").
    pub fn as_str(self) -> &'static str {
        match self {
            ImageLoadingStatus::Idle => "idle",
            ImageLoadingStatus::Loading => "loading",
            ImageLoadingStatus::Loaded => "loaded",
            ImageLoadingStatus::Error => "error",
        }
    }
}

/// `AvatarRootContext` (`AvatarRootContext.ts:5-8`): `imageLoadingStatus` (read by
/// Fallback's gate and the parts' state objects) and `setImageLoadingStatus`
/// (written by Image only) — the same signal in the port (upstream's state pair,
/// read two ways).
#[derive(Clone)]
pub struct AvatarRootContextValue {
    /// `imageLoadingStatus` (`:6`).
    pub image_loading_status: RwSignal<ImageLoadingStatus>,
    /// `setImageLoadingStatus` (`:7`).
    pub set_image_loading_status: RwSignal<ImageLoadingStatus>,
}

/// The context crosses the view/machinery boundary through a `SendWrapper` (the
/// meter `MeterRootContextValue` precedent — the workspace's context convention).
pub type AvatarRootContext = SendWrapper<AvatarRootContextValue>;

/// `AvatarRootContext.Provider` (`AvatarRoot.tsx:41`) — publishes the value for the
/// parts subtree. The root renders its own element inside the provider, so the
/// boundary is strictly one-directional (implementation.md "Context
/// providers/consumers").
pub fn provide_avatar_root_context(value: AvatarRootContextValue) -> AvatarRootContext {
    let wrapped = SendWrapper::new(value);
    leptos::prelude::provide_context(wrapped.clone());
    wrapped
}

/// `useAvatarRootContext` (`AvatarRootContext.ts:12-19`) — the context read plus
/// the missing-root guard: panics with the upstream `Base UI:`-prefixed message
/// (`:14-18`; implementation.md untested item 1 pins the port's decision to
/// reproduce the throw — the meter `use_meter_root_context` precedent).
pub fn use_avatar_root_context() -> AvatarRootContextValue {
    let wrapped = leptos::prelude::use_context::<AvatarRootContext>()
        .expect(
            "Base UI: AvatarRootContext is missing. Avatar parts must be placed within <Avatar.Root>.",
        );
    // Move the signal handles out so repeated reads share the same signals
    // (`AvatarRootContextValue` is `Clone`; the wrapper derefs to it).
    (*wrapped).clone()
}
