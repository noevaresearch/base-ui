//! Port of `packages/react/src/internals/composite/root/CompositeRootContext.ts` — the
//! context bag items use to participate in the composite's roving highlight without any
//! props (`TODO.md`, item `infra: internals`; the checkpoint sequence recorded in that
//! entry's note).
//!
//! Upstream (`CompositeRootContext.ts:4-19`) carries `highlightedIndex`,
//! `onHighlightedIndexChange`, `highlightItemOnHover`, and `relayKeyboardEvent` — the
//! last one exists so detached triggers (e.g. a Menubar whose `Menu.Root` sits outside
//! `CompositeRoot`) can forward keyboard events the root would otherwise never see
//! (`:8-14`). The context default is `undefined`, and the accessor's non-optional
//! overload throws the "Composite parts must be placed within <Composite.Root>" error
//! (`:25-29`).
//!
//! Spec: `specs/library/internals/implementation.md` ("Context providers/consumers" —
//! the `CompositeRootContext` row and the `useButton` optional-consumer note). Every
//! claim was verified against the source before porting.
//!
//! ## Rust adaptations
//!
//! - React context ports to reactive-graph's owner-scoped
//!   [`provide_context`]/[`use_context`] behind [`SharedCompositeRootContext`] — the
//!   `SendWrapper` bridge required by `provide_context`'s `Send + Sync` contract (the
//!   `composite_list.rs` precedent).
//! - `useCompositeRootContext()`'s two overloads (optional returning `undefined`,
//!   required throwing) port to [`use_composite_root_context`] returning
//!   [`Option`] and [`use_composite_root_context_required`] panicking with the upstream
//!   message — a thrown error's only non-view behavior is unwinding to the nearest
//!   boundary, which a panic preserves for a missing-provider developer error.
//! - `onHighlightedIndexChange`'s optional second argument
//!   (`(index, shouldScrollIntoView?) => void`, `:6`) ports to a fixed
//!   `(i32, bool)` signature — `false` for the default (no scroll) call sites.
//! - `highlightedIndex` (`:5`) is the root's reactive [`Memo`]`<i32>`: the context value
//!   is handed out once, so per-render freshness (upstream's `useMemo` per commit) is
//!   carried by the reactive read instead.

use std::rc::Rc;

use reactive_graph::computed::Memo;
use reactive_graph::owner::{provide_context, use_context};
use send_wrapper::SendWrapper;

use crate::floating_ui::element_props::ElementEventHandler;
use web_sys::KeyboardEvent;

/// The context bag — upstream's `CompositeRootContext`
/// (`packages/react/src/internals/composite/root/CompositeRootContext.ts:4-15`).
#[derive(Clone)]
pub struct CompositeRootContextValue {
    /// `highlightedIndex` (`:5`) — the root's resolved (external ?? internal) index.
    pub highlighted_index: Memo<i32>,
    /// `onHighlightedIndexChange` (`:6`) — `(index, shouldScrollIntoView)`.
    pub on_highlighted_index_change: Rc<dyn Fn(i32, bool)>,
    /// `highlightItemOnHover` (`:7`) — whether `mousemove` focuses items (the
    /// `useCompositeItem` hover arm).
    pub highlight_item_on_hover: bool,
    /// `relayKeyboardEvent` (`:14`) — forwards out-of-tree keyboard events into the
    /// root's navigation pipeline; the same handler as the root's `onKeyDown`
    /// (`useCompositeRoot.ts:338`).
    pub relay_keyboard_event: ElementEventHandler<KeyboardEvent>,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `composite_list.rs`
/// precedent).
pub type SharedCompositeRootContext = SendWrapper<CompositeRootContextValue>;

/// Publishes the composite root context for the current subtree — upstream's
/// `CompositeRootContext.Provider` (`CompositeRoot.tsx:84`, value built at `:73-81`).
/// Must be called inside a reactive owner (a component), like the other hook ports.
pub fn provide_composite_root_context(
    value: CompositeRootContextValue,
) -> CompositeRootContextValue {
    provide_context(SharedCompositeRootContext::new(value.clone()));
    value
}

/// The optional accessor — upstream's `useCompositeRootContext(true)` overload
/// (`CompositeRootContext.ts:21-23`): `None` when no root is in scope. This is the
/// variant `useButton`'s composite inference consumes
/// (`useButton.ts:25-26`).
pub fn use_composite_root_context() -> Option<CompositeRootContextValue> {
    use_context::<SharedCompositeRootContext>().map(|shared| (*shared).clone())
}

/// The required accessor — upstream's `useCompositeRootContext()` overload
/// (`CompositeRootContext.ts:25-29`): panics with the upstream error when the context is
/// missing ("Composite parts must be placed within <Composite.Root>").
pub fn use_composite_root_context_required() -> CompositeRootContextValue {
    use_composite_root_context().unwrap_or_else(|| {
        panic!(
            "Base UI: CompositeRootContext is missing. Composite parts must be placed within \
             <Composite.Root>."
        )
    })
}
