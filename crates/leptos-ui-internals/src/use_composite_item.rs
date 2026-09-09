//! Port of `packages/react/src/internals/composite/item/useCompositeItem.ts` — the
//! item-side hook that registers with the composite list and participates in the roving
//! highlight (`TODO.md`, item `infra: internals`; the checkpoint sequence recorded in
//! that entry's note).
//!
//! Upstream (`useCompositeItem.ts:16-48`) composes the root context with the list-item
//! registration: `highlightedIndex === index` decides the roving `tabIndex` (`:21`,
//! `:27`); the item's `onFocus` moves the highlight to itself (`:28-30`); `onMouseMove`
//! focuses the hovered item when `highlightItemOnHover` is set, the item is not already
//! highlighted, and it is not disabled — `disabled` attribute or `aria-disabled="true"`
//! (`:31-41`). The merged ref puts the list-registration ref first (`:24`) — shared-node
//! ownership follows ref attachment order.
//!
//! Spec: `specs/library/internals/implementation.md` ("Context providers/consumers" —
//! the `CompositeRootContext` row: "useCompositeItem (highlight + hover)"; "Anything in
//! source not explained by any test" item 6 — `highlightItemOnHover` hover-focus is
//! untested upstream) and `specs/library/internals/behavior.md` ("Focus management" —
//! the roving tabindex matrix asserted through the `CompositeRoot` suite). Every claim
//! was verified against the source before porting.
//!
//! ## Rust adaptations
//!
//! - `highlightedIndex === index` (`:21`) is a reactive [`Memo`] over both sides, and the
//!   roving `tabIndex` (`:27`) is a derived `Memo<i32>` — the view layer binds it as the
//!   `tabindex` attribute.
//! - `itemRef` (`:23`) ports to a `StoredValue` slot; the merged ref pairs it with the
//!   list-registration ref via [`use_merged_refs`] — the same merge upstream performs,
//!   registration side first.
//! - The `disabled` read (`:37`) uses `ariaDisabled === 'true'` upstream (the ARIA
//!   reflection property); the port reads the `aria-disabled` attribute — the same
//!   source the reflection property projects, and the spelling
//!   `leptos_ui_utils::is_element_disabled` already uses.
//! - The required context accessor panics with the upstream error when no root is in
//!   scope (`CompositeRootContext.ts:25-29`) — see
//!   [`crate::composite_root_context`].

use std::rc::Rc;

use reactive_graph::computed::Memo;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, WithValue};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, FocusEvent, HtmlElement, MouseEvent};

use leptos_ui_utils::use_merged_refs::{InputRef, MergedRefCallback, use_merged_refs};
use leptos_ui_utils::use_ref_with_init::use_ref_with_init;

use crate::composite_root_context::use_composite_root_context_required;
use crate::use_composite_list_item::{
    UseCompositeListItem, UseCompositeListItemParams, use_composite_list_item,
};

/// `UseCompositeItemParameters` (`useCompositeItem.ts:11-14`) — the metadata pass-through
/// to the list registration; every other list-item parameter keeps its upstream default.
pub struct UseCompositeItemParams<M> {
    /// `metadata` (`:13`).
    pub metadata: Option<M>,
}

/// Upstream's `compositeProps` object (`useCompositeItem.ts:26-42`) — the three members
/// the view layer spreads onto the item element.
pub struct CompositeItemProps {
    /// `tabIndex` (`:27`) — `0` for the highlighted item, `-1` otherwise.
    pub tab_index: Memo<i32>,
    /// `onFocus` (`:28-30`) — moves the highlight to this item.
    pub on_focus: Rc<dyn Fn(&FocusEvent)>,
    /// `onMouseMove` (`:31-41`) — the `highlightItemOnHover` focus arm.
    pub on_mouse_move: Rc<dyn Fn(&MouseEvent)>,
}

/// Upstream's return object (`useCompositeItem.ts:44-48`) plus the reactive
/// `is_highlighted` the roving `tabIndex` derives from.
pub struct UseCompositeItem {
    /// `compositeProps` (`:45`).
    pub composite_props: CompositeItemProps,
    /// `compositeRef` (`:46`) — the list-registration ref merged with the internal item
    /// slot; registration side first (shared-node ownership, `:24`).
    pub composite_ref: Option<MergedRefCallback<Element>>,
    /// `index` (`:47`) — the resolved registry index.
    pub index: Memo<i32>,
    /// The `highlightedIndex === index` comparison (`:21`) as a reactive value.
    pub is_highlighted: Memo<bool>,
}

/// Port of `useCompositeItem` (`useCompositeItem.ts:16-48`). Must be called inside a
/// reactive owner (a component) and within a `CompositeRoot`'s context (the required
/// accessor panics otherwise, matching upstream's throw).
pub fn use_composite_item<M>(params: UseCompositeItemParams<M>) -> UseCompositeItem
where
    M: Clone + PartialEq + 'static,
{
    let UseCompositeItemParams { metadata } = params;

    // `useCompositeRootContext()` (`:17-18`) — the required overload.
    let context = use_composite_root_context_required();

    // `useCompositeListItem(params)` (`:19`) — only `metadata` is forwarded; the index
    // stays automatic (`externalIndex: null`) and unguessed, the upstream defaults.
    let UseCompositeListItem {
        index,
        ref_callback,
        ..
    } = use_composite_list_item::<M, RwSignal<Option<i32>>>(UseCompositeListItemParams {
        guess: false,
        index: RwSignal::new(None),
        label: None,
        metadata,
        text_ref: None,
    });

    // `const isHighlighted = highlightedIndex === index` (`:21`).
    let is_highlighted = {
        let highlighted = context.highlighted_index.clone();
        let index = index.clone();
        Memo::new(move |_| highlighted.get() == index.get())
    };

    // `itemRef` (`:23`) + the merged ref (`:24`).
    let item_ref = use_ref_with_init(|| None::<Element>);
    let composite_ref = use_merged_refs(
        InputRef::Callback(Rc::clone(&ref_callback)),
        InputRef::Object(Rc::new(item_ref.clone())),
    );

    let composite_props = CompositeItemProps {
        // `tabIndex: isHighlighted ? 0 : -1` (`:27`).
        tab_index: {
            let is_highlighted = is_highlighted.clone();
            Memo::new(move |_| if is_highlighted.get() { 0 } else { -1 })
        },
        // `onFocus() { onHighlightedIndexChange(index) }` (`:28-30`) — the default
        // `shouldScrollIntoView` (`false`).
        on_focus: {
            let on_highlighted_index_change = context.on_highlighted_index_change.clone();
            let index = index.clone();
            Rc::new(move |_event: &FocusEvent| {
                on_highlighted_index_change(index.get_untracked(), false);
            })
        },
        // `onMouseMove` (`:31-41`).
        on_mouse_move: {
            let item_ref = item_ref.clone();
            let is_highlighted = is_highlighted.clone();
            let highlight_item_on_hover = context.highlight_item_on_hover;
            Rc::new(move |_event: &MouseEvent| {
                let Some(item) = item_ref.with_value(|slot| slot.clone()) else {
                    return;
                };
                if !highlight_item_on_hover {
                    return;
                }

                // `item.hasAttribute('disabled') || item.ariaDisabled === 'true'` (`:37`).
                let disabled = item.has_attribute("disabled")
                    || item.get_attribute("aria-disabled").as_deref() == Some("true");
                if !is_highlighted.get_untracked() && !disabled {
                    if let Ok(html) = item.dyn_into::<HtmlElement>() {
                        let _ = html.focus();
                    }
                }
            })
        },
    };

    UseCompositeItem {
        composite_props,
        composite_ref,
        index,
        is_highlighted,
    }
}
