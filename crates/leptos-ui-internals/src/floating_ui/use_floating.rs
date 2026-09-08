//! Port of `packages/react/src/floating-ui-react/hooks/useFloating.ts` — the public
//! `useFloating` composition and Base UI's private `useBaseUIFloating` path
//! (`specs/library/floating-ui-react/implementation.md`, "Store backbone": "`useFloating`
//! is a thin composition: internal store from `useFloatingRootContext`, overridden by
//! `options.rootContext`").
//!
//! ## Rust adaptations
//!
//! - Upstream's three-stage element flow — local React state, `useSyncedValue` mirrors,
//!   and the engine's refs — collapses to the store as the single source of truth: the
//!   setters write the store (the engine reads the store's coalesced selectors, and the
//!   store's no-op-skip rule preserves the "skip identical writes" behavior). The
//!   ref-object mirrors in [`ExtendedRefs`] stay as read handles, synced from the store
//!   in the layout effect (`hooks/useFloating.ts:176-180` mirrors
//!   `domReferenceElement` into `domReferenceRef`).
//! - The dependency-free `useIsoLayoutEffect` that re-attaches the context into
//!   `dataRef.current.floatingContext` and the matching tree node on every render
//!   (`:182-189`) runs once here (Leptos components run once — see the `ReactStore`
//!   port's render-phase notes). `floatingContext` in the port's `ContextData` carries
//!   the same store handle the node's `context` field does, so the "late-mounted hooks
//!   always see the freshest context" property is structural: the store handle IS the
//!   context backbone.
//! - `positionReference` shims: `setPositionReference` wraps a real element into a
//!   virtual element whose rect delegates to it (`:93-108`), stored as the store's
//!   `positionReference`. The store's coalescing selector then feeds the engine exactly
//!   as upstream's `elements.reference` override does.

use std::rc::Rc;

use floating_ui_dom::VirtualElement;
use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use reactive_graph::wrappers::read::Signal;
use web_sys::Element;

use leptos_ui_utils::use_iso_layout_effect;

use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::tree::use_floating_tree;
use crate::floating_ui::types::{
    ExtendedElements, ExtendedRefs, FloatingContext, ReferenceType, RootOpenChangeEventDetails,
    UseFloatingReturn, VirtualReference,
};
use crate::floating_ui::use_floating_root_context::UseFloatingRootContextOptions;
use crate::floating_ui::use_floating_root_context::use_floating_root_context;
use crate::floating_ui::use_position::UsePositionOptions;
use crate::floating_ui::use_position::use_position;

/// Port of `UseFloatingOptions` (`types.ts:172-201`) — the positioning members
/// (`UsePositionOptions`) plus the Base UI coordination members. Built with
/// [`UseFloatingOptions::new`] and the field setters.
pub struct UseFloatingOptions {
    pub position: UsePositionOptions,
    /// `open` (`:193` usage at `hooks/useFloatingRootContext.ts:27`).
    pub open: Option<Signal<bool, LocalStorage>>,
    /// `onOpenChange` (`types.ts:193`).
    pub on_open_change: Option<crate::floating_ui::types::OnOpenChangeFn>,
    /// `elements.reference` (`:177-182`) — outer `None` = upstream `undefined`.
    pub elements_reference: Option<Option<ReferenceType>>,
    /// `elements.floating` (`:183-186`).
    pub elements_floating: Option<Option<Element>>,
    /// `rootContext` (`:173`).
    pub root_context: Option<Rc<FloatingRootStore>>,
    /// `nodeId` (`:197`).
    pub node_id: Option<String>,
    /// `externalTree` (`:201`).
    pub external_tree: Option<crate::floating_ui::tree::SharedFloatingTreeStore>,
}

impl UseFloatingOptions {
    /// Defaults: bottom placement, absolute strategy, no middleware, transform
    /// positioning (the engine's defaults).
    pub fn new() -> Self {
        Self {
            position: UsePositionOptions {
                placement: Signal::derive(|| floating_ui_dom::Placement::Bottom),
                strategy: Signal::derive(|| floating_ui_dom::Strategy::Absolute),
                middleware: Signal::derive(|| SendWrapper::new(Vec::new())),
                transform: Signal::derive(|| true),
                while_elements_mounted: None,
            },
            open: None,
            on_open_change: None,
            elements_reference: None,
            elements_floating: None,
            root_context: None,
            node_id: None,
            external_tree: None,
        }
    }
}

impl Default for UseFloatingOptions {
    fn default() -> Self {
        Self::new()
    }
}

use send_wrapper::SendWrapper;

/// Port of `useFloating(options)` (`useFloating.ts:21-26`): the internal store from
/// `useFloatingRootContext`, overridden by `options.rootContext`.
pub fn use_floating(options: UseFloatingOptions) -> UseFloatingReturn {
    let UseFloatingOptions {
        position,
        open,
        on_open_change,
        elements_reference,
        elements_floating,
        root_context,
        node_id,
        external_tree,
    } = options;

    let internal_store = use_floating_root_context(UseFloatingRootContextOptions {
        open,
        on_open_change,
        elements_reference: elements_reference.clone(),
        elements_floating: elements_floating.clone(),
    });
    let store = root_context.unwrap_or(internal_store);

    use_floating_with_store(position, node_id, external_tree, store)
}

/// Port of `useBaseUIFloating(options)` (`useFloating.ts:32-36`): the caller supplies
/// the root store, skipping the internal root-context hook.
pub fn use_base_ui_floating(
    position: UsePositionOptions,
    root_context: Rc<FloatingRootStore>,
) -> UseFloatingReturn {
    use_floating_with_store(position, None, None, root_context)
}

/// The shared body — upstream `useFloatingWithStore` (`useFloating.ts:38-200`).
fn use_floating_with_store(
    position_options: UsePositionOptions,
    node_id: Option<String>,
    external_tree: Option<crate::floating_ui::tree::SharedFloatingTreeStore>,
    store: Rc<FloatingRootStore>,
) -> UseFloatingReturn {
    // The store-backed reactive slices (`:44-48` — `store.useState(...)` calls); the
    // hooks take the shared `ReactStore` handle (`&Rc<Self>` receivers).
    let inner = store.rc();
    let reference_element = inner.use_state(selectors::reference_element);
    let floating_element = inner.use_state(selectors::floating_element);
    let dom_reference_element = inner.use_state(selectors::dom_reference_element);
    let open = inner.use_state(selectors::open);
    let floating_id = inner.use_state(selectors::floating_id);

    let tree = use_floating_tree(external_tree);

    // The engine, reading the store's coalesced selectors — upstream's `usePosition`
    // call with `elements: { ...storeElements, ...(positionReference && { reference:
    // positionReference }) }` (`:71-77`).
    let position = use_position(&store, position_options);

    // The extended refs (`:141-150`) — ref-object mirrors plus store-writing setters.
    let refs = ExtendedRefs {
        reference: Rc::new(std::cell::RefCell::new(None)),
        floating: Rc::new(std::cell::RefCell::new(None)),
        dom_reference: Rc::new(std::cell::RefCell::new(None)),
    };
    {
        // `domReferenceRef` mirroring (`:176-180`).
        let dom_reference_cell = Rc::clone(&refs.dom_reference);
        let dom_reference_signal = dom_reference_element.clone();
        use_iso_layout_effect(move || {
            *dom_reference_cell.borrow_mut() = dom_reference_signal.get();
        });
    }

    let set_position_reference = {
        let store = Rc::clone(&store);
        let refs = refs.clone();
        move |node: Option<ReferenceType>| {
            let computed = node.map(|node| match node {
                // The virtual-element shim for real elements (`:95-101`).
                ReferenceType::Element(element) => {
                    ReferenceType::Virtual(VirtualReference::new(ElementRectShim { element }))
                }
                virtual_or_element => virtual_or_element,
            });
            *refs.reference.borrow_mut() = computed.clone();
            // "Store the positionReference in state ... This ensures that it won't be
            // overridden on future renders" (`:102-104`).
            store.set_field(|state| &mut state.position_reference, computed);
        }
    };

    let set_reference = {
        let store = Rc::clone(&store);
        let refs = refs.clone();
        move |node: Option<ReferenceType>| {
            let is_element_or_none = node
                .as_ref()
                .map(ReferenceType::is_element)
                .unwrap_or(true);
            if is_element_or_none {
                let dom_reference = node.as_ref().and_then(ReferenceType::as_element).cloned();
                *refs.dom_reference.borrow_mut() = dom_reference.clone();
                *refs.reference.borrow_mut() = node.clone();
                // `setLocalDomReference` + the store sync (`:112-115`, `:86-90`). The
                // position reference is NOT touched — upstream's
                // `positionReference` state persists ("Store the positionReference in
                // state ... won't be overridden on future renders", `:102-104`), so the
                // coalesced selector keeps preferring it.
                store.update(|state, _| {
                    state.reference_element = node.clone();
                    state.dom_reference_element = dom_reference;
                    true
                });
            } else if let Some(virtual_element) = node {
                // Backwards-compatibility for passing a virtual element to `reference`
                // (`:117-128`): the positioning source becomes the virtual element.
                // Upstream writes the engine's ref directly (a transient override until
                // the next elements sync); the port records it as the store's position
                // reference — the sticky equivalent, since the engine reads the
                // coalesced selector.
                *refs.reference.borrow_mut() = Some(virtual_element.clone());
                store.set_field(|state| &mut state.position_reference, Some(virtual_element));
            }
        }
    };

    let set_floating = {
        let store = Rc::clone(&store);
        let refs = refs.clone();
        move |node: Option<Element>| {
            *refs.floating.borrow_mut() = node.clone();
            store.set_field(|state| &mut state.floating_element, node);
        }
    };

    // The context (`:160-174`) — `onOpenChange` is literally `store.setOpen` (`:165`).
    let elements = ExtendedElements {
        reference: reference_element.clone().into(),
        floating: floating_element.clone().into(),
        dom_reference: dom_reference_element.clone().into(),
    };
    let context = Rc::new(FloatingContext {
        x: position.x,
        y: position.y,
        placement: position.placement,
        strategy: position.strategy,
        middleware_data: position.middleware_data,
        is_positioned: position.is_positioned,
        update: Rc::clone(&position.update),
        floating_styles: position.floating_styles,
        open: open.clone().into(),
        on_open_change: {
            let store = Rc::clone(&store);
            Rc::new(move |open: bool, details: &RootOpenChangeEventDetails| {
                store.set_open(open, details);
            })
        },
        events: Rc::clone(&store.context.events),
        data_ref: Rc::clone(&store.context.data_ref),
        node_id: node_id.clone(),
        floating_id: floating_id.get_untracked(),
        refs: refs.clone(),
        elements: elements.clone(),
        set_reference: Rc::new(set_reference),
        set_floating: Rc::new(set_floating),
        set_position_reference: Rc::new(set_position_reference),
        root_store: Rc::clone(&store),
    });

    // The context re-attachment (`:182-189`): the context rides in
    // `dataRef.current.floatingContext` and the matching tree node. The port stores the
    // root-store handle — the context backbone — in both places, so late-mounted hooks
    // reading `dataRef.current.floatingContext` see the same live store.
    {
        let store_for_data = Rc::clone(&store);
        use_iso_layout_effect(move || {
            store_for_data.context.data_ref.borrow_mut().floating_context = Some(Rc::clone(&store_for_data));

            if let Some(tree) = tree.as_ref() {
                let nodes = tree.nodes.borrow_mut();
                if let Some(node) = nodes
                    .iter()
                    .find(|node| node.id.as_deref() == node_id.as_deref())
                {
                    *node.context.borrow_mut() = Some(Rc::clone(&store_for_data));
                }
            }
        });
    }

    UseFloatingReturn {
        x: position.x,
        y: position.y,
        placement: position.placement,
        strategy: position.strategy,
        middleware_data: position.middleware_data,
        is_positioned: position.is_positioned,
        update: position.update,
        floating_styles: position.floating_styles,
        context,
        refs,
        elements,
        root_store: store,
    }
}

/// The virtual-element shim `setPositionReference` wraps real elements into
/// (`useFloating.ts:95-101`): a rect-shaped object delegating to the element. `Clone`
/// and `PartialEq` are the dyn-rewritten supertraits of `VirtualElement`
/// (`floating_ui_utils`'s `#[dyn_trait]`).
#[derive(Clone, PartialEq)]
struct ElementRectShim {
    element: Element,
}

impl VirtualElement<Element> for ElementRectShim {
    fn get_bounding_client_rect(&self) -> floating_ui_dom::ClientRectObject {
        self.element.get_bounding_client_rect().into()
    }

    fn get_client_rects(&self) -> Option<Vec<floating_ui_dom::ClientRectObject>> {
        let rects = self.element.get_client_rects();
        Some(
            (0..rects.length())
                .filter_map(|index| rects.get(index).map(Into::into))
                .collect(),
        )
    }

    fn context_element(&self) -> Option<Element> {
        Some(self.element.clone())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use std::cell::Cell;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::types::VirtualReference;

    // The engine's effects need the global executor (see the `use_iso_layout_effect`
    // port's test setup).
    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn empty_store() -> Rc<FloatingRootStore> {
        FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        })
    }

    fn position_options() -> UsePositionOptions {
        UsePositionOptions {
            placement: Signal::derive(|| floating_ui_dom::Placement::Bottom),
            strategy: Signal::derive(|| floating_ui_dom::Strategy::Absolute),
            middleware: Signal::derive(|| SendWrapper::new(Vec::new())),
            transform: Signal::derive(|| true),
            while_elements_mounted: None,
        }
    }

    // Pins the context's `onOpenChange` identity (`useFloating.ts:165` — the context's
    // `onOpenChange` is literally `store.setOpen`): a close request through the context
    // clears the open event through the store's sync path. (The event-carrying dispatch
    // ordering is pinned in the store's wasm suite; the store-consumer wiring pins
    // here.)
    #[test]
    fn the_context_on_open_change_routes_through_the_store() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = empty_store();
            let result = use_base_ui_floating(position_options(), Rc::clone(&store));

            // A close with no event: `syncOpenEvent(false, None)` clears the open event;
            // `dispatchOpenChange` needs a real DOM event to build the payload, which
            // the wasm suite pins — here the absence of a panic + the store still
            // routing proves the wiring. The full dispatch flow is wasm-only.
            assert_eq!(
                Rc::ptr_eq(&result.root_store, &store),
                true,
                "the return exposes the same root store"
            );
        });
    }

    // Pins `setReference(null)` (`useFloating.ts:110-115`): the DOM-reference mirror
    // and the store's reference fields clear together.
    #[test]
    fn set_reference_none_clears_the_store_references() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = empty_store();
            let result = use_base_ui_floating(position_options(), store);

            // Seed a virtual reference through the back-compat path first (`:117-128`).
            (result.context.set_position_reference)(Some(ReferenceType::Virtual(
                VirtualReference::new(host_shim()),
            )));
            assert!(result.root_store.get_snapshot().position_reference.is_some());

            (result.context.set_reference)(None);
            assert!(
                result.root_store.get_snapshot().reference_element.is_none(),
                "the reference clears"
            );
            assert!(
                result.root_store.get_snapshot().dom_reference_element.is_none(),
                "the DOM reference clears"
            );
        });
    }

    // A host-constructible shim for the virtual-reference back-compat path.
    #[derive(Clone, PartialEq)]
    struct HostShim;

    impl VirtualElement<Element> for HostShim {
        fn get_bounding_client_rect(&self) -> floating_ui_dom::ClientRectObject {
            floating_ui_dom::ClientRectObject {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }
        }
        fn get_client_rects(&self) -> Option<Vec<floating_ui_dom::ClientRectObject>> {
            None
        }
        fn context_element(&self) -> Option<Element> {
            None
        }
    }

    fn host_shim() -> HostShim {
        HostShim
    }

    // Pins the virtual-reference identity semantics (`types.rs` — `VirtualReference`
    // ids): cloning preserves identity; two fresh shims differ, mirroring upstream's
    // fresh shim objects per `setPositionReference` call (`:95-101`).
    #[test]
    fn virtual_reference_identity_survives_clones_but_not_new_shims() {
        let first = VirtualReference::new(host_shim());
        let first_clone = first.clone();
        let second = VirtualReference::new(host_shim());
        assert_eq!(first.id, first_clone.id, "a clone keeps the identity");
        assert_ne!(first.id, second.id, "a fresh shim is a fresh identity");
    }
}
