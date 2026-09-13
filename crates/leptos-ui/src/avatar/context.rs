//! The Avatar context — `AvatarRootContext.ts` (the whole file) plus the
//! `ImageLoadingStatus` union (`AvatarRoot.tsx:44`).

use reactive_graph::signal::RwSignal;
use send_wrapper::SendWrapper;

/// `ImageLoadingStatus` (`AvatarRoot.tsx:44`) — `'idle' | 'loading' | 'loaded' |
/// 'error'`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLoadingStatus {
    /// `'idle'` — the seed, and the root's reset value on image unmount.
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

/// `AvatarRootContext` (`AvatarRootContext.ts:5-8`): the mirrored status (read by
/// Fallback and the parts' state objects) and its setter (written by Image only).
/// The setter is the plain value writer (`SetStateAction` narrowed to the value
/// arm — every upstream call site passes a literal status).
#[derive(Clone)]
pub struct AvatarRootContextValue {
    /// `imageLoadingStatus` (`:6`).
    pub image_loading_status: RwSignal<ImageLoadingStatus>,
    /// `setImageLoadingStatus` (`:7`).
    pub set_image_loading_status: RwSignal<ImageLoadingStatus>,
}

/// The context is shared across runtimes (the views build under leptos's, the
/// machinery under rg-0.2's) — the `SendWrapper` convention of every context value
/// carrying an `Rc`-backed slot in this workspace.
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

/// `useAvatarRootContext` (`AvatarRootContext.ts:12-19`) — the context read plus the
/// missing-root guard, panicking with the upstream `'Base UI: AvatarRootContext is
/// missing. Avatar parts must be placed within <Avatar.Root>.'` when no root is an
/// ancestor (`:14-18`; no test exercises this — implementation.md untested item 1
/// pins the port's decision to reproduce the throw; the meter
/// `use_meter_root_context` precedent).
pub fn use_avatar_root_context() -> AvatarRootContextValue {
    leptos::prelude::use_context::<AvatarRootContext>().expect(
        "Base UI: AvatarRootContext is missing. Avatar parts must be placed within <Avatar.Root>.",
    )
    // `.take()`-style move-out: the SendWrapper derefs to the value; clone the two
    // signal handles out so repeated reads share the same signals.
    .clone_inner()
}

impl AvatarRootContext {
    fn clone_inner(&self) -> AvatarRootContextValue {
        AvatarRootContextValue {
            image_loading_status: self.image_loading_status.clone(),
            set_image_loading_status: self.set_image_loading_status.clone(),
        }
    }
}
