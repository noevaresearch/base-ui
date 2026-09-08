//! Port of `packages/react/src/floating-ui-react/utils/nodes.ts` — the logical
//! (non-DOM) floating-tree walking helpers over the registered [`FloatingNodeType`]s
//! (`specs/library/floating-ui-react/behavior.md`, "DOM structure & portal behavior":
//! "`nodes` is a logical (non-DOM) flat tree").
//!
//! Nodes are shared `Rc` handles: `removeNode` removes the exact node object by
//! identity (`components/FloatingTreeStore.ts:17-22`), and `node.context` is patched in
//! after registration (`hooks/useFloating.ts:185-188`), so the port keeps one
//! allocation per node and walks the same handles the tree stores.

use std::rc::Rc;

use crate::floating_ui::types::FloatingNodeType;

/// `getNodeChildren(nodes, id, onlyOpenChildren)` (`nodes.ts:5-16`): the direct
/// children of `id` — recursively including their children — filtered to open nodes
/// when `onlyOpenChildren` (`:13` — `child.context?.open`; the port reads the node's
/// root-store handle's `open` state). Order is depth-first, children before their own
/// descendants (`flatMap`).
pub fn get_node_children(
    nodes: &[Rc<FloatingNodeType>],
    id: Option<&str>,
    only_open_children: bool,
) -> Vec<Rc<FloatingNodeType>> {
    let direct_children: Vec<_> = nodes
        .iter()
        .filter(|node| node.parent_id.as_deref() == id)
        .cloned()
        .collect();

    let mut result = Vec::new();
    for child in direct_children {
        let is_open = child
            .context
            .borrow()
            .as_ref()
            .map(|store| store.get_snapshot().open)
            .unwrap_or(false);
        if !only_open_children || is_open {
            result.push(Rc::clone(&child));
        }
        result.extend(get_node_children(
            nodes,
            child.id.as_deref(),
            only_open_children,
        ));
    }
    result
}

/// `getDeepestNode(nodes, id)` (`nodes.ts:18-38`): the node in `id`'s subtree at the
/// maximum depth, starting the count at `id` itself (`findDeepest(id, 0)`). Returns the
/// node from `nodes`, or `None` when `id` has no registered subtree match.
pub fn get_deepest_node(
    nodes: &[Rc<FloatingNodeType>],
    id: Option<&str>,
) -> Option<Rc<FloatingNodeType>> {
    let mut deepest_node_id: Option<String> = None;
    let mut max_depth: i64 = -1;

    fn find_deepest(
        nodes: &[Rc<FloatingNodeType>],
        node_id: Option<&str>,
        depth: i64,
        deepest_node_id: &mut Option<String>,
        max_depth: &mut i64,
    ) {
        if depth > *max_depth {
            *deepest_node_id = node_id.map(str::to_owned);
            *max_depth = depth;
        }

        for child in get_node_children(nodes, node_id, true) {
            find_deepest(
                nodes,
                child.id.as_deref(),
                depth + 1,
                deepest_node_id,
                max_depth,
            );
        }
    }

    find_deepest(
        nodes,
        id,
        0,
        &mut deepest_node_id,
        &mut max_depth,
    );

    nodes
        .iter()
        .find(|node| node.id == deepest_node_id)
        .cloned()
}

/// `getNodeAncestors(nodes, id)` (`nodes.ts:40-53`): the ancestor chain of `id`, nearest
/// parent first, excluding the node itself. Upstream appends each found ancestor to the
/// result in parent→grandparent order (`:46-50`).
pub fn get_node_ancestors(nodes: &[Rc<FloatingNodeType>], id: Option<&str>) -> Vec<Rc<FloatingNodeType>> {
    let mut all_ancestors: Vec<Rc<FloatingNodeType>> = Vec::new();
    let mut current_parent_id = nodes
        .iter()
        .find(|node| node.id.as_deref() == id)
        .and_then(|node| node.parent_id.clone());

    while let Some(parent_id) = current_parent_id {
        let current_node = nodes
            .iter()
            .find(|node| node.id.as_deref() == Some(parent_id.as_str()));
        current_parent_id = current_node.and_then(|node| node.parent_id.clone());

        if let Some(node) = current_node {
            all_ancestors.push(Rc::clone(node));
        }
    }

    all_ancestors
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::floating_ui::floating_root_store::FloatingRootStore;
    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    fn node(id: &str, parent_id: Option<&str>, store: Option<Rc<FloatingRootStore>>) -> Rc<FloatingNodeType> {
        Rc::new(FloatingNodeType {
            id: Some(id.to_owned()),
            parent_id: parent_id.map(str::to_owned),
            context: std::cell::RefCell::new(store),
        })
    }

    fn root_store(open: bool) -> Rc<FloatingRootStore> {
        let store = FloatingRootStore::new(FloatingRootStoreOptions {
            open,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        });
        store.update(|state, _| {
            state.open = open;
            true
        });
        store
    }

    // Fixture shape (behavior.md, "Edge cases": "onlyOpenChildren not pruning
    // closed-intermediary descendants"):
    //
    //   root
    //   ├── child (open) ── gc (closed)
    //   └── ci (closed) ── cic (open)
    fn fixture() -> Vec<Rc<FloatingNodeType>> {
        vec![
            node("root", None, None),
            node("child", Some("root"), Some(root_store(true))),
            node("gc", Some("child"), Some(root_store(false))),
            node("ci", Some("root"), Some(root_store(false))),
            node("cic", Some("ci"), Some(root_store(true))),
        ]
    }

    fn ids(nodes: &[Rc<FloatingNodeType>]) -> Vec<String> {
        nodes.iter().map(|node| node.id.clone().unwrap()).collect()
    }

    // `getNodeChildren` (`nodes.ts:5-16`): with `onlyOpenChildren`, open children are
    // kept, closed children pruned — but the walk still descends through closed
    // intermediaries, so their open descendants surface (`:12-15` recurses
    // unconditionally).
    #[test]
    fn only_open_children_keeps_open_nodes_but_descends_closed_intermediaries() {
        let nodes = fixture();
        assert_eq!(
            ids(&get_node_children(&nodes, Some("root"), true)),
            vec!["child".to_owned(), "cic".to_owned()],
            "open child kept, closed `ci` pruned, but its open descendant `cic` surfaces"
        );
    }

    #[test]
    fn without_only_open_children_every_descendant_is_returned_depth_first() {
        let nodes = fixture();
        assert_eq!(
            ids(&get_node_children(&nodes, Some("root"), false)),
            vec![
                "child".to_owned(),
                "gc".to_owned(),
                "ci".to_owned(),
                "cic".to_owned()
            ],
            "flatMap order: each child followed by its own subtree"
        );
    }

    // `getDeepestNode` (`nodes.ts:18-38`): the walk iterates `getNodeChildren`'s
    // FLATTENED subtree, examining every descendant at a uniform `depth + 1` — so the
    // first open descendant of `root` (`child`) wins the max-depth tie against `cic`.
    // This quirk is upstream's own (`children.forEach((child) => findDeepest(child.id,
    // depth + 1))` over the flat list), and the port preserves it.
    #[test]
    fn get_deepest_node_returns_the_first_maximum_depth_node() {
        let nodes = fixture();
        assert_eq!(
            get_deepest_node(&nodes, Some("root"))
                .and_then(|node| node.id.clone()),
            Some("child".to_owned()),
            "the flattened walk examines all descendants at depth+1; the first (`child`) \
             wins the tie"
        );
    }

    // `getNodeAncestors` (`nodes.ts:40-53`): nearest parent first, excluding the node.
    #[test]
    fn get_node_ancestors_walks_the_parent_chain() {
        let nodes = fixture();
        assert_eq!(
            ids(&get_node_ancestors(&nodes, Some("cic"))),
            vec!["ci".to_owned(), "root".to_owned()]
        );
        assert!(get_node_ancestors(&nodes, Some("root")).is_empty());
    }
}
