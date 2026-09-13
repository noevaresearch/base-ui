//! The Avatar view layer — the leptos `Render` bridge over the engine's
//! [`RenderedElement`] descriptions (the docs-app `RawElementView` bridge,
//! re-homed crate-side), plus the dynamic part views.
//!
//! - [`AvatarDocView`]: a (possibly fresh) materialized element as a view —
//!   `rebuild` replaces the node in place (upstream's re-render).
//! - [`avatar_image_view`]/[`avatar_fallback_view`]: closures for the
//!   `{move || …}` dynamic-view child (the progress-hero convention): each
//!   re-run materializes a fresh element — the tracked mirror reads are the
//!   re-render inputs — and `None` renders nothing (upstream's `null`). No
//!   wrapper element ever exists in the DOM.
//!
//! The closures capture the handles through `SendWrapper` (the workspace's
//! cross-runtime convention): the part handles hold `Rc`-typed engine
//! vocabulary, and leptos's dynamic children require `Send`. Everything runs
//! on the single UI thread (wasm), where the wrapper is sound.

use std::rc::Rc;

use leptos::tachys::html::attribute::Attribute;
use leptos::tachys::hydration::Cursor;
use leptos::tachys::renderer::types as renderer_types;
use leptos::tachys::renderer::CastFrom;
use leptos::tachys::view::add_attr::AddAnyAttr;
use leptos::tachys::view::{Mountable, Render, RenderHtml};
use leptos::tachys::view::{Position, PositionState};
use send_wrapper::SendWrapper;

use leptos_ui_internals::use_transition_status::TransitionStatus;
use leptos_ui_utils::use_merged_refs::InputRef;

use crate::avatar::fallback::{
    AvatarFallbackProps, UseAvatarFallback, avatar_fallback_element, avatar_fallback_state,
};
use crate::avatar::image::{
    AvatarImageProps, AvatarImageStateSnapshot, UseAvatarImage, avatar_image_element,
};
use crate::avatar::root::AvatarRootState;

/// A materialized element as a leptos view (the use-render page's
/// `RawElementView` bridge, re-homed crate-side).
pub struct AvatarDocView {
    pub element: web_sys::Element,
}

/// The retained view state: the mounted DOM node.
pub struct AvatarDocViewState(web_sys::Element);

impl Mountable for AvatarDocViewState {
    fn unmount(&mut self) {
        let _ = self.0.remove();
    }

    fn mount(&mut self, parent: &renderer_types::Element, marker: Option<&renderer_types::Node>) {
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

impl Render for AvatarDocView {
    type State = AvatarDocViewState;

    fn build(self) -> Self::State {
        AvatarDocViewState(self.element)
    }

    fn rebuild(self, state: &mut Self::State) {
        // A rebuilt AvatarDocView is a new materialization: replace the node
        // in place (the per-render replacement of upstream's element).
        if let Some(parent) = state.0.parent_node() {
            let _ = parent.replace_child(&self.element, &state.0);
        }
        state.0 = self.element;
    }
}

// CSR-only view plumbing (the use-render page's RawElementView precedent):
// the docs app is client-rendered only, so server-side HTML has no meaning.
impl RenderHtml for AvatarDocView {
    type AsyncOutput = Self;

    const MIN_LENGTH: usize = 0;

    fn dry_resolve(&mut self) {}

    async fn resolve(self) -> Self::AsyncOutput {
        self
    }

    fn to_html_with_buf(
        self,
        _buf: &mut String,
        _position: &mut leptos::tachys::view::Position,
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

impl AddAnyAttr for AvatarDocView {
    type Output<SomeNewAttr: Attribute> = AvatarDocView;

    fn add_any_attr<NewAttr: Attribute>(self, _attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml,
    {
        self
    }
}

/// The dynamic image view body — the closure for the `{move || …}` dynamic
/// child. Tracked reads: the mounted mirror, the masked transition-status
/// mirror, and the image-status mirror (the `AvatarImageState` +
/// `shouldRender` inputs; upstream's per-render derivation). Each re-run
/// materializes the fresh element with the seam riding the ref fork — the
/// ref-fire is the commit (the keepMounted sync runs there); `None` renders
/// nothing (`!shouldRender → null`, `:174-176`).
pub fn avatar_image_view(
    handle: UseAvatarImage,
    props: AvatarImageProps,
) -> impl Fn() -> Option<AvatarDocView> + Send + 'static {
    let handle = SendWrapper::new(handle);
    let props = SendWrapper::new(props);
    move || {
        let handle = &handle;
        let props = &props;
        // Tracked reads — the mirrors (the leptos runtime this view tracks).
        let mounted = leptos::prelude::Get::get(&handle.mounted_mirror);
        let transition_status: Option<TransitionStatus> =
            leptos::prelude::Get::get(&handle.status_mirror);
        // The image-local status is a TRACKED read: keepMounted re-renders the
        // element on every status change (`data-loading`/`data-error`/
        // `aria-hidden` are status-derived, `:101-118`; the `AvatarImage.test
        // .tsx:537-573` re-render contract). Default mode's element DOM is
        // status-independent (it only exists once loaded), but the rebuild is
        // harmless and the idempotent set keeps it to a no-op.
        let image_loading_status = leptos::prelude::Get::get(&handle.image_status_mirror);
        // The masked transition status (`:150`): with keepMounted the element
        // never unmounts, so an `'ending'` phase would play and reverse;
        // `data-loading`/`data-error` carry that state instead.
        let transition_status = if handle.keep_mounted
            && transition_status == Some(TransitionStatus::Ending)
        {
            None
        } else {
            transition_status
        };

        let snapshot = AvatarImageStateSnapshot {
            image_loading_status,
            transition_status,
            keep_mounted: handle.keep_mounted,
            mounted,
        };

        // The ref fork (`:168`'s `[forwardedRef, imageRef]`): the caller's
        // ref rides the props; the seam is this port's imageRef slot.
        let seam = handle.element_seam.clone();
        let rendered =
            avatar_image_element(snapshot, props, vec![InputRef::Callback(seam)])?;
        let (element, _cleanup) = rendered.create_element();
        // The replaced node takes its listeners with it into GC; the cleanup
        // is deliberately not retained across rebuilds (the dynamic
        // re-materialization is upstream's re-render — the separator-page
        // static precedent).
        std::mem::forget(_cleanup);
        Some(AvatarDocView { element })
    }
}

/// The dynamic fallback view body — the closure for the `{move || …}` dynamic
/// child. Tracked reads: the root status and the delay-latch mirror (the
/// `enabled` gate inputs, `:46`). `None` renders nothing (`enabled: false` —
/// upstream's `useRenderElement` null).
pub fn avatar_fallback_view(
    handle: UseAvatarFallback,
    props: AvatarFallbackProps,
) -> impl Fn() -> Option<AvatarDocView> + Send + 'static {
    let handle = SendWrapper::new(handle);
    let props = SendWrapper::new(props);
    move || {
        let handle = &handle;
        let props = &props;
        // Tracked reads.
        let image_loading_status = leptos::prelude::Get::get(&handle.root_status);
        let delay_passed = leptos::prelude::Get::get(&handle.delay_mirror);

        let enabled = avatar_fallback_state(image_loading_status, handle.delay, delay_passed);
        let state = AvatarRootState {
            image_loading_status,
        };

        let rendered = avatar_fallback_element(state, enabled, props)?;
        let (element, _cleanup) = rendered.create_element();
        std::mem::forget(_cleanup);
        Some(AvatarDocView { element })
    }
}
