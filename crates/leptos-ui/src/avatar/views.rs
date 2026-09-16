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
use leptos::tachys::renderer::CastFrom;
use leptos::tachys::renderer::types as renderer_types;
use leptos::tachys::view::add_attr::AddAnyAttr;
use leptos::tachys::view::{Mountable, Render, RenderHtml};
use leptos::tachys::view::{Position, PositionState};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

use leptos_ui_internals::use_render_element::RenderedElement;
use leptos_ui_internals::use_transition_status::TransitionStatus;
use leptos_ui_utils::use_merged_refs::InputRef;

use crate::avatar::fallback::{
    AvatarFallbackProps, UseAvatarFallback, avatar_fallback_element, avatar_fallback_state,
};
use crate::avatar::image::{
    AvatarImageProps, AvatarImageStateSnapshot, UseAvatarImage, avatar_image_element, should_render,
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
        let transition_status =
            if handle.keep_mounted && transition_status == Some(TransitionStatus::Ending) {
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

        // The presence gate closes (`!shouldRender → null`, `:174-176`):
        // release the retained node (React's unmount — the element leaves
        // the tree with its listeners) and render nothing.
        if !should_render(&snapshot) {
            *handle.materialized.borrow_mut() = None;
            return None;
        }

        // The ref fork (`:168`'s `[forwardedRef, imageRef]`): the caller's
        // ref rides the props; the seam is this port's imageRef slot.
        let seam = handle.element_seam.clone();
        let rendered = avatar_image_element(snapshot, props, vec![InputRef::Callback(seam)]);

        // The React commit analog. Upstream's same-type re-render RETAINS
        // the DOM element: React diffs the props onto the existing node and
        // never re-fires refs. A fresh `create_element` per rebuild would
        // (a) restart the browser's fetch on every re-render (the fresh
        // `src` starts a new request), (b) drop the element's load/error
        // listeners with the replaced node, and (c) re-fire the ref fork —
        // whose keepMounted sync then re-reads the INCOMPLETE fresh element
        // and regresses the status to `'loading'`. So: the FIRST
        // materialization creates the node and fires the ref fork
        // (mounting); every LATER run updates the retained node in place.
        // (The retained-node read is hoisted out of the `match` scrutinee:
        // a scrutinee-temporary borrow guard would still be held when the
        // creating arm writes the slot — "RefCell already borrowed".)
        let previous = handle.materialized.borrow().clone();
        match (previous, rendered) {
            // The re-render commit: same node, refreshed attributes, refs
            // untouched.
            (Some(previous), Some(rendered)) => {
                update_element(previous.unchecked_ref(), &rendered);
                Some(AvatarDocView { element: previous })
            }
            // The first commit: create, fire the fork, retain.
            (None, Some(rendered)) => {
                let (element, _cleanup) = rendered.create_element();
                // The (single) element's listeners live on the retained
                // node for the view's lifetime; the cleanup is deliberately
                // not retained (the separator-page static precedent).
                std::mem::forget(_cleanup);
                *handle.materialized.borrow_mut() = Some(element.clone());
                Some(AvatarDocView { element })
            }
            // Unreachable (the gate returned above) — upstream's null.
            (_, None) => None,
        }
    }
}

/// The in-place commit — React's prop diff onto a retained element,
/// mirroring `create_element`'s write set. Attributes present in the fresh
/// description are set; attributes absent from it (but set by an earlier
/// commit) are removed — exactly what React does when a prop's value goes
/// from defined to `undefined`. Class/style replace wholesale (the only
/// avatar call site passes neither).
pub(crate) fn update_element(node: &web_sys::HtmlElement, rendered: &RenderedElement) {
    if let Some(class) = &rendered.props.class {
        node.set_attribute("class", class).expect("set class");
    }
    if !rendered.props.style.is_empty() {
        let style = rendered
            .props
            .style
            .iter()
            .map(|(property, value)| format!("{property}: {value};"))
            .collect::<Vec<_>>()
            .join(" ");
        node.set_attribute("style", &style).expect("set style");
    }
    let mut seen: Vec<String> = Vec::new();
    for (name, value) in &rendered.props.handlers.attributes {
        if let Some(value) = value() {
            node.set_attribute(name, &value).expect("set attribute");
        } else {
            let _ = node.remove_attribute(name);
        }
        seen.push(name.clone());
    }
    if let Some(inner_html) = &rendered.props.inner_html {
        node.set_inner_html(inner_html);
    }
    // The lazy `None` arms of PREVIOUS commits are not re-derived here (the
    // fresh description enumerates its own props), but a status attribute
    // that DISAPPEARED between commits must be removed — walk the DOM's own
    // attribute set for names this description no longer carries.
    let dom_attrs = node.get_attribute_names();
    for name_value in dom_attrs.iter() {
        let Some(name) = name_value.as_string() else {
            continue;
        };
        let managed = seen.iter().any(|seen_name| seen_name == &name)
            || matches!(name.as_str(), "class" | "style");
        if !managed {
            let _ = node.remove_attribute(&name);
        }
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

/// Binds a dynamic-part closure as a leptos dynamic-view child — the actual `{move || …}`
/// invocation. [`avatar_image_view`]/[`avatar_fallback_view`] return *closures* (each re-run
/// materializes a fresh element, `None` renders nothing), so handing one to `view!` bare would bind
/// a never-invoked value: the type checks and the tree never exists (the crate's own wasm suite
/// recorded exactly that trap). `Avatar.Image`/`Avatar.Fallback` (the namespaced part surface in
/// `avatar/mod.rs`) are what let a caller skip the wrapping entirely.
///
/// The docs page and the wasm harness each carry a private copy of this one-liner
/// (`crates/docs-app/src/pages/avatar_page.rs:123`, `avatar_tests.rs:557`); those are separate
/// compilation targets with their own local helpers, and this is the crate-side canonical one — a
/// third copy in the crate would be the defect, so the component surface uses this.
pub fn dynamic_part_view<V: leptos::prelude::IntoView + 'static>(
    body: impl Fn() -> V + Send + 'static,
) -> impl leptos::prelude::IntoView + 'static {
    move || body()
}
