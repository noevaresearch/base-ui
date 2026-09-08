//! Port of `packages/react/src/floating-ui-react/components/FloatingTreeStore.ts` and
//! `components/FloatingTree.tsx` — the logical floating tree backing store and its
//! React-context plumbing (`FloatingTreeContext`/`FloatingNodeContext`,
//! `FloatingTree.tsx:10-11`).
//!
//! ## Rust adaptations
//!
//! - `nodesRef: { current: Array<FloatingNodeType> }` (`FloatingTreeStore.ts:9`) ports
//!   to a `RefCell<Vec<Rc<FloatingNodeType>>>`: the node objects the tree, the tree
//!   walkers, and `useFloatingNodeId`'s cleanup closure share must stay
//!   identity-comparable (`removeNode` finds the exact node object, `:17-22`), which the
//!   `Rc` handles give; `node.context` is patched through its own `RefCell`
//!   (`hooks/useFloating.ts:187`).
//! - `FloatingTree`/`FloatingNode` are React context-provider components. This crate is
//!   view-free (the `csp_provider`/`direction_provider` realm convention), so the
//!   providers port to [`provide_floating_tree`] / [`provide_floating_node`] — the same
//!   `provide_context` calls the component bodies make (`FloatingTree.tsx:94`,
//!   `FloatingTree.tsx:67-71`) — and the view layer composes them when Phase B lands.
//!   `FloatingTree`'s once-per-mount store creation (`:93`, `useRefWithInit`) is the
//!   caller's choice of store handle here.
//! - `useFloatingNodeId`'s registration effect (`:37-47`) ports to
//!   [`use_floating_node_id`] over the ported `use_iso_layout_effect` (the
//!   `RenderEffect` decision, `specs/architecture.md`, "Layout effect") and the `useId`
//!   port. Upstream re-registers on `[tree, id, parentId]` changes (`:47`) — the id is
//!   stable per hook instance (the deterministic counter) and the parent id is
//!   call-time context, so the effect's single registration run plus owner-disposal
//!   cleanup expresses the same lifecycle. Upstream bails when `id` is falsy (`:38-40`);
//!   the ported `use_id` always yields a non-empty string, so the guard reduces to the
//!   optional-chained no-tree case.
//! - `useFloatingTree(externalTree)` (`:23-26`): `externalTree ?? contextTree` — the
//!   explicit tree wins over the ambient context.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::owner::LocalStorage;
use reactive_graph::owner::on_cleanup;
use reactive_graph::owner::provide_context;
use reactive_graph::owner::use_context;
use reactive_graph::traits::GetUntracked;
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;

use leptos_ui_utils::use_id;
use leptos_ui_utils::use_iso_layout_effect;

use crate::floating_ui::types::FloatingNodeType;
use crate::floating_ui::types::FloatingTreeEvents;

/// Port of `FloatingTreeStore` (`FloatingTreeStore.ts:8-23`): "stores and manages
/// floating elements in a tree structure" — the backing store for the `FloatingTree`
/// provider.
#[derive(Default)]
pub struct FloatingTreeStore {
    /// `nodesRef` (`:9`) — the registered nodes, shared by `Rc` handle (see the module
    /// docs).
    pub nodes: RefCell<Vec<Rc<FloatingNodeType>>>,
    /// `events` (`:11`) — the shared bus carrying `'floating.closed'` and
    /// `'virtualfocus'` (see the types module docs for the citations).
    pub events: FloatingTreeEvents,
}

/// Convenience alias matching upstream's `FloatingTreeType` (`types.ts:145`). The `Rc`
/// handle sits behind a `SendWrapper` because the tree travels through
/// `provide_context`, which requires `Send + Sync` (`reactive_graph`'s context
/// contract) — the same bridge the crate's other non-thread-safe handles use.
pub type SharedFloatingTreeStore = SendWrapper<Rc<FloatingTreeStore>>;

impl FloatingTreeStore {
    /// Upstream constructor (the class instance shape, `FloatingTreeStore.ts:8-23`).
    pub fn new() -> Self {
        Self::default()
    }

    /// `addNode(node)` (`:13-15`).
    pub fn add_node(&self, node: Rc<FloatingNodeType>) {
        self.nodes.borrow_mut().push(node);
    }

    /// `removeNode(node)` (`:17-22`): removes the exact node object (`findIndex` by
    /// identity), no-op when absent.
    pub fn remove_node(&self, node: &Rc<FloatingNodeType>) {
        let mut nodes = self.nodes.borrow_mut();
        if let Some(index) = nodes
            .iter()
            .position(|registered| Rc::ptr_eq(registered, node))
        {
            nodes.remove(index);
        }
    }
}

/// `FloatingNodeContext` (`FloatingTree.tsx:10`) — the `{ id, parentId }` context the
/// nested-float parent chain is published through (`FloatingTree.tsx:68`).
#[derive(Clone, Default)]
pub struct FloatingNodeContext {
    pub id: Option<String>,
    pub parent_id: Option<String>,
}

/// `FloatingTreeContext` (`FloatingTree.tsx:11`) — the tree-store context.
#[derive(Clone)]
pub struct FloatingTreeContext(pub SharedFloatingTreeStore);

/// Port of `useFloatingParentNodeId` (`FloatingTree.tsx:17-18`): the parent node id from
/// the ambient [`FloatingNodeContext`], `None` for top-level floats
/// (`?.id || null`).
pub fn use_floating_parent_node_id() -> Option<String> {
    use_context::<FloatingNodeContext>().and_then(|node| node.id)
}

/// Port of `useFloatingTree(externalTree)` (`FloatingTree.tsx:23-26`): the nearest
/// floating tree context, unless an explicit external tree is given.
pub fn use_floating_tree(
    external_tree: Option<SharedFloatingTreeStore>,
) -> Option<SharedFloatingTreeStore> {
    external_tree.or_else(|| use_context::<FloatingTreeContext>().map(|context| context.0))
}

/// Port of `useFloatingNodeId(externalTree)` (`FloatingTree.tsx:32-50`): generates the
/// node id (the `useId` port), registers `{ id, parentId }` into the tree in a layout
/// effect, and unregisters on cleanup. Must be called inside a reactive owner.
pub fn use_floating_node_id(
    external_tree: Option<SharedFloatingTreeStore>,
) -> Signal<String, LocalStorage> {
    let id = use_id(Signal::derive(|| None::<String>), None);
    let tree = use_floating_tree(external_tree);
    let parent_id = use_floating_parent_node_id();

    let tree_for_effect = tree.clone();
    let id_value = id.get_untracked();
    use_iso_layout_effect(move || {
        let node = Rc::new(FloatingNodeType {
            id: Some(id_value.clone()),
            parent_id: parent_id.clone(),
            context: RefCell::new(None),
        });
        if let Some(tree) = tree_for_effect.as_ref() {
            tree.add_node(Rc::clone(&node));
        }
        // The cleanup captures the exact node handle `add_node` stored, so
        // `remove_node`'s identity match removes precisely this registration
        // (`FloatingTree.tsx:44-46`). The handle rides behind a `SendWrapper` because
        // cleanup closures cross `reactive_graph`'s Send+Sync bound — the same bridge
        // the `ReactStore` port uses for its cleanup closures.
        let node_for_cleanup = SendWrapper::new(node);
        on_cleanup({
            let tree = tree_for_effect.clone();
            move || {
                if let Some(tree) = tree.as_ref() {
                    tree.remove_node(&*node_for_cleanup);
                }
            }
        });
    });

    id
}

/// Provides the tree context — the body of the `FloatingTree` provider component
/// (`FloatingTree.tsx:90-95`) without the view layer (see the module docs). Returns the
/// store that is now ambient for the current reactive owner's subtree.
pub fn provide_floating_tree(tree: SharedFloatingTreeStore) -> SharedFloatingTreeStore {
    provide_context(FloatingTreeContext(tree.clone()));
    tree
}

/// Provides the node context — the body of the `FloatingNode` provider component
/// (`FloatingTree.tsx:62-72`) without the view layer: publishes `{ id, parentId }`
/// where `parentId` is the ambient parent node id at provision time (`:65`).
pub fn provide_floating_node(id: Option<String>) {
    let parent_id = use_floating_parent_node_id();
    provide_context(FloatingNodeContext { id, parent_id });
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins `addNode`/`removeNode` (`FloatingTreeStore.ts:13-22`): nodes append in order,
    // and removal takes out the exact node object — a second registration with the same
    // id is a different node object upstream (`{ id, parentId }` is a fresh object per
    // registration, `FloatingTree.tsx:42`), so removing one leaves the other.
    #[test]
    fn removal_is_by_node_identity_not_id() {
        let tree = FloatingTreeStore::new();

        let first = Rc::new(FloatingNodeType {
            id: Some("floating-1".to_owned()),
            parent_id: None,
            context: RefCell::new(None),
        });
        let second = Rc::new(FloatingNodeType {
            id: Some("floating-1".to_owned()),
            parent_id: None,
            context: RefCell::new(None),
        });
        tree.add_node(Rc::clone(&first));
        tree.add_node(Rc::clone(&second));
        assert_eq!(tree.nodes.borrow().len(), 2);

        tree.remove_node(&first);
        assert_eq!(tree.nodes.borrow().len(), 1);
        assert!(
            Rc::ptr_eq(&tree.nodes.borrow()[0], &second),
            "the exact node object was removed, its same-id sibling stays"
        );

        tree.remove_node(&first);
        assert_eq!(
            tree.nodes.borrow().len(),
            1,
            "removal of an absent node is a no-op"
        );
    }
}

// The registration effect's lifecycle needs a DOM environment: the ported
// `use_iso_layout_effect` selects its effect binding by probing for the `document`
// global, which only exists under the wasm/browser target.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Pins the registration effect's lifecycle (`FloatingTree.tsx:37-47`): registering
    // inside a reactive owner adds the node (synchronously — the layout-effect
    // decision, `specs/architecture.md`, "Layout effect"); owner disposal (React
    // unmount) removes the exact node.
    #[wasm_bindgen_test]
    fn use_floating_node_id_registers_and_disposal_unregisters() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        let tree = SharedFloatingTreeStore::new(Rc::new(FloatingTreeStore::new()));

        let node_id = use_floating_node_id(Some(tree.clone()));
        let node_id = node_id.get_untracked();

        assert_eq!(tree.nodes.borrow().len(), 1, "the node registered");
        assert_eq!(
            tree.nodes.borrow()[0].id,
            Some(node_id),
            "the returned id is the registered node's id"
        );
        assert_eq!(tree.nodes.borrow()[0].parent_id, None, "top-level float");

        owner.cleanup();
        assert!(
            tree.nodes.borrow().is_empty(),
            "owner disposal unregisters the node"
        );
    }

    // Pins the parent-chain context (`FloatingTree.tsx:17-18,67-71`): a nested provider
    // scope inherits the ambient node context through the owner chain — the same
    // subtree scoping React's context providers give. (Top-level floats read `None` —
    // pinned by the registration test's `parent_id == None` assertion.)
    #[wasm_bindgen_test]
    fn nested_provision_publishes_the_parent_chain() {
        let _ = any_spawner::Executor::init_futures_executor();
        let outer = reactive_graph::owner::Owner::new();
        outer.set();

        provide_floating_node(Some("parent-node".to_owned()));
        assert_eq!(
            use_floating_parent_node_id(),
            Some("parent-node".to_owned()),
            "the ambient node context feeds the parent id"
        );

        // A nested scope inherits the ambient context.
        let inner = reactive_graph::owner::Owner::new();
        inner.set();
        assert_eq!(
            use_floating_parent_node_id(),
            Some("parent-node".to_owned()),
            "the nested scope reads the parent's node context through the owner chain"
        );
        inner.cleanup();
        outer.cleanup();
    }
}
