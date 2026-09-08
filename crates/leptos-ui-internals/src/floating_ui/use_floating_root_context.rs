//! Port of `packages/react/src/floating-ui-react/hooks/useFloatingRootContext.ts` — the
//! store-construction path behind the public `useFloating`
//! (`specs/library/floating-ui-react/implementation.md`, "Store backbone": "Two
//! construction paths feed the same store class").

use std::rc::Rc;

use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::GetValue;
use reactive_graph::wrappers::read::Signal;
use web_sys::Element;

use leptos_ui_utils::use_id;
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_ref_with_init;

use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
use crate::floating_ui::tree::use_floating_parent_node_id;
use crate::floating_ui::types::{OnOpenChangeFn, ReferenceType, TransitionStatus};

/// Port of `UseFloatingRootContextOptions` (`useFloatingRootContext.ts:15-24`).
///
/// The `elements` members keep upstream's `undefined` vs `null` distinction: the outer
/// `None` is `undefined` (not provided — the store keeps its current value), the inner
/// `None` is an explicit `null` (clears the store value). The layout-effect sync is
/// gated on provided-ness (`:64-71`).
pub struct UseFloatingRootContextOptions {
    /// `open` (`:16`) — default `false` (`:27`).
    pub open: Option<Signal<bool, LocalStorage>>,
    /// `onOpenChange` (`:17`).
    pub on_open_change: Option<OnOpenChangeFn>,
    /// `elements.reference` (`:20`) — must be a real DOM element when provided; a
    /// virtual element here triggers the dev-only error (`:32-41`).
    pub elements_reference: Option<Option<ReferenceType>>,
    /// `elements.floating` (`:21`).
    pub elements_floating: Option<Option<Element>>,
}

impl UseFloatingRootContextOptions {
    /// The upstream defaults (`:27` — `open = false`, `elements = {}`).
    pub fn new() -> Self {
        Self {
            open: None,
            on_open_change: None,
            elements_reference: None,
            elements_floating: None,
        }
    }
}

impl Default for UseFloatingRootContextOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Port of `useFloatingRootContext(options)` (`useFloatingRootContext.ts:26-79`):
/// creates the `FloatingRootStore` once (`useRefWithInit`, `:43-56`), syncs the
/// option-driven state in a layout effect (`:58-74`), and re-patches the context's
/// `onOpenChange`/`nested` (`:76-77`). Must be called inside a reactive owner.
pub fn use_floating_root_context(options: UseFloatingRootContextOptions) -> Rc<FloatingRootStore> {
    let UseFloatingRootContextOptions {
        open,
        on_open_change,
        elements_reference,
        elements_floating,
    } = options;

    let open = open.unwrap_or_else(|| Signal::derive_local(|| false));
    let open_value = open.get_untracked();

    let floating_id_signal = use_id(Signal::<Option<String>, LocalStorage>::from(None), None);
    let floating_id = floating_id_signal.get_untracked();
    let nested = use_floating_parent_node_id().is_some();

    // The dev-only virtual-element guard (`:32-41`).
    #[cfg(debug_assertions)]
    if let Some(Some(reference)) = elements_reference.as_ref() {
        if !reference.is_element() {
            leptos_ui_utils::error().log(&[
                "Cannot pass a virtual element to the `elements.reference` option, \
                 as it must be a real DOM element. Use `context.setPositionReference()` \
                 instead.",
            ]);
        }
    }

    let store: Rc<FloatingRootStore> = use_ref_with_init({
        let elements_reference = elements_reference.clone();
        let elements_floating = elements_floating.clone();
        let on_open_change = on_open_change.clone();
        move || {
            FloatingRootStore::new(FloatingRootStoreOptions {
                open: open_value,
                transition_status: None::<TransitionStatus>,
                reference_element: elements_reference.unwrap_or(None),
                floating_element: elements_floating.unwrap_or(None),
                trigger_elements: PopupTriggerMap::new(),
                floating_id: Some(floating_id),
                sync_only: false,
                nested,
                on_open_change,
            })
        }
    })
    .get_value();

    // The option-sync layout effect (`:58-74`): `open`/`floatingId` always sync;
    // `referenceElement`/`floatingElement` only when provided (the `undefined` vs
    // `null` distinction — see the options docs). `open` is tracked so a reactive
    // open change re-syncs the store, mirroring the upstream dependency array.
    {
        let store = Rc::clone(&store);
        let elements_reference = elements_reference.clone();
        let elements_floating = elements_floating.clone();
        use_iso_layout_effect(move || {
            let open_value = open.get();
            let floating_id_value = floating_id_signal.get_untracked();

            store.update(|state, _| {
                state.open = open_value;
                state.floating_id = Some(floating_id_value.clone());
                true
            });

            if let Some(reference) = elements_reference.as_ref() {
                let dom_reference = reference
                    .as_ref()
                    .and_then(ReferenceType::as_element)
                    .cloned();
                store.update(|state, _| {
                    state.reference_element = reference.clone();
                    state.dom_reference_element = dom_reference;
                    true
                });
            }

            if let Some(floating) = elements_floating.as_ref() {
                store.set_field(|state| &mut state.floating_element, floating.clone());
            }
        });
    }

    // The context re-patch (`:76-77`) — non-reactive values kept fresh.
    store.context.set_on_open_change(on_open_change);
    store.context.set_nested(nested);

    store
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use std::cell::Cell;

    use crate::floating_ui::types::OnOpenChangeFn;

    // Pins the construction seeding (`useFloatingRootContext.ts:43-56`): the store is
    // built once with the option values — `open: false` default, provided elements
    // seeded — and the context patch (`:76-77`) lands synchronously.
    #[test]
    fn seeds_the_store_and_patches_the_context() {
        reactive_graph::owner::Owner::new().with(|| {
            let calls: Rc<Cell<Vec<bool>>> = Rc::new(Cell::new(Vec::new()));
            let calls_handle = Rc::clone(&calls);
            let on_open_change: OnOpenChangeFn = Rc::new(move |open: bool, _details| {
                let mut log = calls_handle.take();
                log.push(open);
                calls_handle.set(log);
            });

            let store = use_floating_root_context(UseFloatingRootContextOptions {
                on_open_change: Some(on_open_change),
                ..UseFloatingRootContextOptions::new()
            });

            assert!(!store.get_snapshot().open, "open defaults to false");
            assert!(
                store.get_snapshot().floating_id.is_some(),
                "the floatingId is generated (the useId port)"
            );
            assert!(
                store.context.on_open_change().is_some(),
                "the context onOpenChange is patched"
            );
            assert!(
                !store.context.nested(),
                "not nested without an ambient tree node"
            );
        });
    }

    // Pins the provided-elements seeding (`:64-67` — `elements.reference` provided):
    // a real element reference seeds both the reference and the DOM reference. Uses the
    // virtual-arm rejection implicitly — a virtual element in `elements.reference`
    // panics the dev guard under debug builds, so only the absent case is host-tested;
    // the element-carrying case is pinned in the wasm suite.
    #[test]
    fn absent_elements_leave_the_store_references_empty() {
        reactive_graph::owner::Owner::new().with(|| {
            let store = use_floating_root_context(UseFloatingRootContextOptions::new());
            assert!(store.get_snapshot().reference_element.is_none());
            assert!(store.get_snapshot().dom_reference_element.is_none());
            assert!(store.get_snapshot().floating_element.is_none());
        });
    }
}
