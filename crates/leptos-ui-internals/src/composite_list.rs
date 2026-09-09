//! Port of `packages/react/src/internals/composite/list/CompositeList.tsx` +
//! `CompositeListContext.ts` — the two-phase composite item registry the composite cluster
//! (`useCompositeListItem` now; `useCompositeRoot`/`CompositeItem` as later checkpoints of this
//! unit) builds on (`TODO.md`, item `infra: internals`; the checkpoint sequence recorded in
//! that entry's note).
//!
//! Upstream is a React component with no DOM of its own
//! (`packages/react/src/internals/composite/list/CompositeList.tsx:22-217`): the source of
//! truth is a `Map<Element, registration>` fed synchronously by the items' callback refs
//! (`register`/`unregister`, `:48-58`); publications are coalesced — `scheduleMapUpdate` sets
//! an `isDirtyRef` flag and bumps a dummy state tick (`:39-46`), and a no-deps
//! `useIsoLayoutEffect` runs `flush()` on every commit when dirty (`:187-191`), which is why
//! "one publication happens per commit where registration changes" and the refs are rebuilt
//! before paint (`specs/library/internals/implementation.md`, "Composite registry").
//! `flush()` derives the ordered snapshot via `getCompositeListSnapshot` (`:142-170`,
//! `:227-272`: connected nodes only, explicit non-negative indexes reserve their slots,
//! automatic items sort by document position and fill the gaps left of the reserved slots,
//! then a final numeric sort), `syncRefs` rebuilds `elementsRef`/`labelsRef` from it — with
//! the label fallback chain `label` → `textRef.textContent` → `element.textContent`
//! (`:76-81`) — and re-seeds `nextIndexRef` from the final length (`:84`). A change pass over
//! the previous snapshot gates the actual publish (`:146-168`). Out-of-React DOM moves are
//! caught by `observe()` (`:89-140`): one `MutationObserver` per *adjacent-pair common
//! ancestor* (`:128-139`), whose callback filters pure add/remove batches via `hasMovedNode`
//! (`:286-296`) and fires only when a connected node now sits before its previously-connected
//! predecessor, ordered by `compareDocumentPosition`'s `DOCUMENT_POSITION_FOLLOWING` bit
//! (`:298-302`).
//!
//! Spec: `specs/library/internals/behavior.md` ("State model" — registry state, indexes,
//! publication alignment; "DOM structure & portal behavior" — the MutationObserver claims;
//! "Edge cases" — churn, out-of-React surgery, labels) and
//! `specs/library/internals/implementation.md` ("Composite registry (`CompositeList` +
//! `useCompositeListItem`)" — the mechanisms above). Every claim was verified against the
//! source before porting.
//!
//! ## Rust adaptations
//!
//! - The `Map<Element, registration>` ports to an insertion-ordered
//!   `Vec<(Element, registration)>` keyed by element identity (`JsValue` `==` is reference
//!   equality) — `Map.set` on an existing key replaces the value in place and keeps its
//!   position, `Map.delete` removes, iteration is insertion order (the
//!   `request_queue.rs` Vec-registry precedent).
//! - The React commit machinery — dirty flag + dummy-state tick + no-deps layout effect —
//!   dissolves into a dirty flag plus a coalesced **microtask** flush (`window.queueMicrotask`,
//!   the `queue_microtask` helper precedent): registrations that happen in one synchronous
//!   stack land in one publication, and the flush still runs before paint, which is the
//!   contract the layout-effect timing exists for. Two deliberate divergences, both
//!   unobservable upstream except through publication counts with no items: the port does not
//!   publish the phantom empty map on mount-with-no-items (upstream's always-run effect
//!   flushes once even when nothing registered), and a flush is scheduled only by
//!   `register`/`unregister`, never by a render. Host builds have no microtask queue: the
//!   default flush is synchronous there, and the coalescing contract is pinned by the
//!   wasm/browser suite (plus a host scheduler override for deterministic counting, the
//!   `timeout_manager.rs` dispatch-override precedent).
//! - The published `Map<Element, CompositeMetadata<Metadata>>` ports to
//!   [`CompositeListMap`] — an index-ordered `Vec` of pairs (upstream builds the published
//!   map by iterating the index-sorted snapshot, so `Array.from(map.keys())` is index order).
//! - `elementsRef`/`labelsRef` (caller-owned `Array<HTMLElement | null>` /
//!   `Array<string | null>` ref objects) port to shared `Rc<RefCell<Vec<Option<_>>>>`
//!   handles — sparse explicit indexes leave `None` slots exactly like the JS array holes
//!   (`elementsRef.current[item.index] = item.element`, `:74`).
//! - `registration.label: string | null | undefined` ports to
//!   `Option<Option<String>>` — outer `None` is `undefined` (fall back to the text chain),
//!   `Some(None)` is the explicit `null` ("no label", no fallback — behavior spec
//!   "labels": the explicit-null arm).
//! - `textRef: RefObject<HTMLElement | null>` ports to [`TextRef`], a shared mutable slot
//!   read at flush time (`:80`), the crate's ref-object mapping.
//! - The change-detection pass compares `registration.metadata` by *identity* upstream
//!   (`:156`); the port compares with `PartialEq`, so an equal-valued re-registered metadata
//!   no longer counts as a change — the gate's purpose (don't republish when nothing
//!   meaningfully changed) is preserved, and the identity gate's only observable difference
//!   is *extra* upstream publications for recreated-but-equal metadata objects.
//! - The unmount effects (`:172-200`: ref-object re-copy, observer disconnect, dirty
//!   marking, ref-array clearing) collapse into [`Drop`] — the reactive owner drops the
//!   provided context value, which drops the last registry handle, which runs the cleanup.
//!   The ref-object re-copy arm is N/A: the port's ref handles are fixed at construction
//!   (there is no prop change to replay).
//! - React context ports to reactive-graph's owner-scoped
//!   [`provide_context`]/[`use_context`] behind [`SharedCompositeListContext`] — the
//!   `SendWrapper` bridge required by `provide_context`'s `Send + Sync` contract (the
//!   `floating_delay_group.rs` precedent). The module-singleton no-op default
//!   (`CompositeListContext.ts:18-23`) is a shared thread-local: provider-less items'
//!   `register`/`unregister`/`subscribeMapChange` are no-ops and their guess claims draw from
//!   one shared counter, exactly like upstream's module-scope default object.
//! - `sortByDocumentPosition`'s bitwise document-position test (`:298-302`) ports to
//!   [`compare_document_position_following`] over the raw `u16` flags.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use reactive_graph::owner::{provide_context, use_context};
use send_wrapper::SendWrapper;
use web_sys::Element;
use web_sys::wasm_bindgen::JsCast;

use leptos_ui_utils::RefCallback;

// ---------------------------------------------------------------------------
// Shared ref handles
// ---------------------------------------------------------------------------

/// Upstream `elementsRef: RefObject<Array<HTMLElement | null>>`
/// (`packages/react/src/internals/composite/list/CompositeList.tsx:313`): the list of
/// registered elements ordered by index — explicit indexes can leave empty (`None`) slots.
/// The same shape the floating-ui `useListNavigation`/`useTypeahead` ports consume as their
/// `ListRef`.
pub type CompositeListElementsRef = Rc<RefCell<Vec<Option<Element>>>>;

/// Upstream `labelsRef: RefObject<Array<string | null>>` (`:318`): the label per index,
/// `None` where the item resolved no label.
pub type CompositeListLabelsRef = Rc<RefCell<Vec<Option<String>>>>;

/// Upstream `textRef: React.RefObject<HTMLElement | null>`
/// (`packages/react/src/internals/composite/list/CompositeListContext.ts:8`): a shared
/// mutable element slot read at flush time for the label fallback chain.
pub type TextRef = Rc<RefCell<Option<Element>>>;

/// Upstream `label: string | null | undefined` (`CompositeListContext.ts:7`): outer [`None`]
/// is `undefined` (resolve through the fallback chain), `Some(None)` is the explicit `null`
/// ("no label" — no fallback), `Some(Some(label))` is the explicit label.
pub type RegistrationLabel = Option<Option<String>>;

// ---------------------------------------------------------------------------
// Registration + published types
// ---------------------------------------------------------------------------

/// Upstream `CompositeListRegistration` (`packages/react/src/internals/composite/list/
/// CompositeListContext.ts:4-9`): the per-item registration the item hooks hand to
/// [`CompositeList::register`].
#[derive(Clone, Debug)]
pub struct CompositeListRegistration<M> {
    /// `metadata` (`:5`) — published with the item, `None` when absent (upstream `null`).
    pub metadata: Option<M>,
    /// `index` (`:6`) — the explicit slot reservation; [`None`] (upstream `null`) means the
    /// index is automatic. Negative values are ignored by the snapshot
    /// (`CompositeList.tsx:246-251`).
    pub index: Option<i32>,
    /// `label` (`:7`) — see [`RegistrationLabel`].
    pub label: RegistrationLabel,
    /// `textRef` (`:8`) — the text slot consulted before the element's own text.
    pub text_ref: Option<TextRef>,
}

/// Upstream `CompositeMetadata<CustomMetadata>` (`CompositeList.tsx:9-11`): the published
/// per-item record — `{ index: number } & CustomMetadata`. The Rust port carries the custom
/// metadata beside the index (an intersection type has no Rust spelling).
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeMetadata<M> {
    /// The resolved index (explicit or automatic).
    pub index: i32,
    /// The item's custom registration metadata, [`None`] when the item registered none.
    pub metadata: Option<M>,
}

/// The published snapshot — upstream's `Map<Element, CompositeMetadata<Metadata>>`
/// (`CompositeList.tsx:319`), built by iterating the index-sorted items, so iteration order
/// is index order and `get` keys by element identity.
#[derive(Clone, Debug)]
pub struct CompositeListMap<M> {
    entries: Vec<(Element, CompositeMetadata<M>)>,
}

impl<M> CompositeListMap<M> {
    /// `map.size` (`CompositeList.tsx:111`).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `map.size === 0` (`useCompositeRoot.ts:111` consumes it this way).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `map.get(node)` — the element-identity lookup.
    pub fn get(&self, node: &Element) -> Option<&CompositeMetadata<M>> {
        self.entries
            .iter()
            .find(|(element, _)| element == node)
            .map(|(_, metadata)| metadata)
    }

    /// `map.keys()` — elements in index order.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.entries.iter().map(|(element, _)| element)
    }

    /// `map.entries()` — the (element, metadata) pairs in index order.
    pub fn iter(&self) -> impl Iterator<Item = &(Element, CompositeMetadata<M>)> {
        self.entries.iter()
    }
}

// ---------------------------------------------------------------------------
// Listeners
// ---------------------------------------------------------------------------

/// The map-publication listener — upstream's `subscribeMapChange` callback and
/// `onMapChange` prop (`CompositeList.tsx:202-207`, `:319`).
pub type CompositeListListener<M> = Rc<dyn Fn(&CompositeListMap<M>)>;

/// The `subscribeMapChange` cleanup (`CompositeList.tsx:204-206`): an explicit unsubscribe —
/// drop semantics would silently differ from upstream's returned function.
#[derive(Clone)]
pub struct CompositeListUnsubscribe {
    unsubscribe: Rc<dyn Fn()>,
}

impl CompositeListUnsubscribe {
    /// Runs the cleanup (removes the listener from the list's registry).
    pub fn unsubscribe(&self) {
        (self.unsubscribe)();
    }
}

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

/// One snapshot entry — upstream's `CompositeListItem` (`CompositeList.tsx:13-17`) plus the
/// registration the change pass and label chain read. Owned (registrations cloned out of the
/// map) so the map borrow can drop before the publishes re-enter.
struct SnapshotItem<M> {
    index: i32,
    element: Element,
    registration: CompositeListRegistration<M>,
}

struct ListInner<M> {
    /// `map` (`CompositeList.tsx:30`) — insertion-ordered element-identity registry.
    map: RefCell<Vec<(Element, CompositeListRegistration<M>)>>,
    /// `listeners` (`:29`, the `createListeners` `Set`) — `subscribeMapChange` subscribers,
    /// id-tagged for removal.
    listeners: RefCell<Vec<(u64, CompositeListListener<M>)>>,
    next_listener_id: Cell<u64>,
    /// `nextIndexRef` (`:31`) — the shared guess-claim counter, re-seeded by `syncRefs`
    /// (`:84`).
    next_index_ref: Rc<Cell<i32>>,
    /// `isDirtyRef` (`:32`) — "a flush is pending" in the port (see the module docs for the
    /// initial-value adaptation).
    is_dirty: Cell<bool>,
    /// `itemsRef` (`:33`) — the previous snapshot, for the change pass (`:146-158`).
    items: RefCell<Option<Vec<SnapshotItem<M>>>>,
    /// `mutationObserverRef` (`:34`) — the observer and its kept closure.
    observer: RefCell<
        Option<(
            web_sys::MutationObserver,
            wasm_bindgen::closure::Closure<dyn FnMut(js_sys::Array, web_sys::MutationObserver)>,
        )>,
    >,
    elements_ref: CompositeListElementsRef,
    labels_ref: Option<CompositeListLabelsRef>,
    on_map_change: Box<dyn Fn(&CompositeListMap<M>)>,
}

impl<M> Drop for ListInner<M> {
    // The upstream unmount cleanups (`CompositeList.tsx:179-184`, `:193-200`): disconnect
    // the observer, clear the ref arrays, mark dirty so a Strict-Mode-style replay would
    // rebuild (the port has no replay — the mark is parity).
    fn drop(&mut self) {
        if let Some((observer, _closure)) = self.observer.borrow_mut().take() {
            observer.disconnect();
        }
        self.elements_ref.borrow_mut().clear();
        if let Some(labels) = &self.labels_ref {
            labels.borrow_mut().clear();
        }
        self.listeners.borrow_mut().clear();
        self.is_dirty.set(true);
    }
}

/// The composite item registry — upstream's `CompositeList` component state
/// (`packages/react/src/internals/composite/list/CompositeList.tsx:22-217`) as a handle.
/// Clones share one registry the way the React context value is shared; the registry is
/// torn down when the last handle drops (the upstream unmount cleanups).
pub struct CompositeList<M> {
    inner: Rc<ListInner<M>>,
}

impl<M> Clone for CompositeList<M> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<M> CompositeList<M>
where
    M: Clone + PartialEq + 'static,
{
    /// Builds the registry — upstream's component setup (`CompositeList.tsx:25-34`): the
    /// caller-owned ref handles and the `onMapChange` prop come in, the map/listeners/
    /// counters start at their initial values (`isDirtyRef` starts `true` upstream; the
    /// port's `is_dirty` starts `false` because the flush is scheduled, not commit-driven —
    /// module docs).
    pub fn new(
        elements_ref: CompositeListElementsRef,
        labels_ref: Option<CompositeListLabelsRef>,
        on_map_change: impl Fn(&CompositeListMap<M>) + 'static,
    ) -> Self {
        Self {
            inner: Rc::new(ListInner {
                map: RefCell::new(Vec::new()),
                listeners: RefCell::new(Vec::new()),
                next_listener_id: Cell::new(0),
                next_index_ref: Rc::new(Cell::new(0)),
                is_dirty: Cell::new(false),
                items: RefCell::new(None),
                observer: RefCell::new(None),
                elements_ref,
                labels_ref,
                on_map_change: Box::new(on_map_change),
            }),
        }
    }

    /// Upstream `register` (`CompositeList.tsx:48-53`): `map.set(node, registration)` +
    /// schedule. An existing entry's value is replaced in place (position kept).
    pub fn register(&self, node: &Element, registration: CompositeListRegistration<M>) {
        let mut map = self.inner.map.borrow_mut();
        if let Some(entry) = map.iter_mut().find(|(element, _)| element == node) {
            entry.1 = registration;
        } else {
            map.push((node.clone(), registration));
        }
        drop(map);
        schedule_map_update(&self.inner);
    }

    /// Upstream `unregister` (`CompositeList.tsx:55-58`): `map.delete(node)` + schedule.
    pub fn unregister(&self, node: &Element) {
        let mut map = self.inner.map.borrow_mut();
        map.retain(|(element, _)| element != node);
        drop(map);
        schedule_map_update(&self.inner);
    }

    /// Upstream `subscribeMapChange` (`CompositeList.tsx:202-207`): adds the listener and
    /// returns its removal handle. Called on every publication where the snapshot changed.
    pub fn subscribe_map_change(
        &self,
        listener: impl Fn(&CompositeListMap<M>) + 'static,
    ) -> CompositeListUnsubscribe {
        self.subscribe_map_change_listener(Rc::new(listener))
    }

    /// `nextIndexRef.current` (`CompositeListContext.ts:15`) — read without claiming.
    pub fn next_index(&self) -> i32 {
        self.inner.next_index_ref.get()
    }

    /// The guess claim — `useCompositeListItem`'s initializer body
    /// (`useCompositeListItem.ts:46-50`): read `nextIndexRef.current` and increment it. The
    /// shared `Rc<Cell>` keeps the re-seed from `syncRefs` (`:84`) visible to claimants.
    pub fn claim_next_index(&self) -> i32 {
        let claimed = self.inner.next_index_ref.get();
        self.inner.next_index_ref.set(claimed + 1);
        claimed
    }

    /// The elements/labels handles the root and item consumers read — upstream passes
    /// `elementsRef` down through props (`useCompositeRoot.ts:96` owns it; `CompositeRoot`
    /// hands it to `CompositeList`).
    pub fn elements_ref(&self) -> CompositeListElementsRef {
        Rc::clone(&self.inner.elements_ref)
    }

    pub fn labels_ref(&self) -> Option<CompositeListLabelsRef> {
        self.inner.labels_ref.as_ref().map(Rc::clone)
    }
}

/// Upstream `scheduleMapUpdate` (`CompositeList.tsx:39-46`) with the scheduling
/// adaptation (module docs): mark dirty and schedule one coalesced flush. Free function so
/// the observer callback can reach it through the upgraded `Rc`.
fn schedule_map_update<M: Clone + PartialEq + 'static>(inner: &Rc<ListInner<M>>) {
    if inner.is_dirty.get() {
        return;
    }
    inner.is_dirty.set(true);

    let weak = Rc::downgrade(inner);
    schedule_flush(Rc::new(move || {
        if let Some(inner) = weak.upgrade() {
            flush(&inner);
        }
    }));
}

/// Upstream `flush` (`CompositeList.tsx:142-170`): snapshot → syncRefs → change pass →
/// observe → store → publish (listeners first, then `onMapChange`).
fn flush<M: Clone + PartialEq + 'static>(inner: &Rc<ListInner<M>>) {
    if !inner.is_dirty.get() {
        return;
    }

    let (items, automatic_nodes) = {
        let map = inner.map.borrow();
        get_composite_list_snapshot(&map)
    };
    let published = sync_refs(inner, &items);

    let changed = {
        let previous = inner.items.borrow();
        match previous.as_ref() {
            None => true,
            Some(previous) => {
                previous.len() != items.len()
                    || items.iter().zip(previous.iter()).any(|(item, prev)| {
                        item.index != prev.index
                            || item.element != prev.element
                            || item.registration.index != prev.registration.index
                            || item.registration.metadata != prev.registration.metadata
                    })
            }
        }
    };

    observe(inner, &automatic_nodes);
    *inner.items.borrow_mut() = Some(items);
    inner.is_dirty.set(false);

    if !changed {
        return;
    }

    let listeners: Vec<_> = inner
        .listeners
        .borrow()
        .iter()
        .map(|(_, l)| Rc::clone(l))
        .collect();
    for listener in listeners {
        listener(&published);
    }
    (inner.on_map_change)(&published);
}

/// Upstream `getCompositeListSnapshot` (`CompositeList.tsx:227-272`): connected nodes only,
/// explicit non-negative indexes reserve their slots (negative explicit indexes are dropped
/// entirely — neither reserved nor automatic), automatic items sort by document position and
/// fill the gaps left of the reserved slots, and a final numeric sort orders everything when
/// any reservation exists. Returns the ordered items and the automatic elements (document
/// order) for the observer.
fn get_composite_list_snapshot<M>(
    map: &[(Element, CompositeListRegistration<M>)],
) -> (Vec<SnapshotItem<M>>, Vec<Element>)
where
    M: Clone,
{
    let mut reserved_indices: HashSet<i32> = HashSet::new();
    let mut items: Vec<SnapshotItem<M>> = Vec::new();
    let mut automatic_items: Vec<(Element, CompositeListRegistration<M>)> = Vec::new();

    for (node, registration) in map {
        if !node.is_connected() {
            continue;
        }

        match registration.index {
            None => automatic_items.push((node.clone(), registration.clone())),
            Some(index) if index >= 0 => {
                reserved_indices.insert(index);
                items.push(SnapshotItem {
                    index,
                    element: node.clone(),
                    registration: registration.clone(),
                });
            }
            // `index >= 0` is the only reserved arm (`CompositeList.tsx:248`) — a negative
            // explicit index falls out of the snapshot entirely.
            Some(_) => {}
        }
    }

    let mut next_automatic_index = 0i32;
    automatic_items.sort_by(|(a, _), (b, _)| {
        if compare_document_position_following(a, b) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    for (element, registration) in &automatic_items {
        while reserved_indices.contains(&next_automatic_index) {
            next_automatic_index += 1;
        }

        items.push(SnapshotItem {
            index: next_automatic_index,
            element: element.clone(),
            registration: registration.clone(),
        });
        next_automatic_index += 1;
    }

    if !reserved_indices.is_empty() {
        items.sort_by_key(|item| item.index);
    }

    let automatic_nodes = automatic_items
        .into_iter()
        .map(|(element, _)| element)
        .collect();
    (items, automatic_nodes)
}

/// Upstream `syncRefs` (`CompositeList.tsx:60-87`): rebuild `elementsRef` (sparse — explicit
/// indexes leave `None` holes, and the length is the highest filled slot + 1, the JS sparse
/// array's `.length`), rebuild `labelsRef` through the fallback chain
/// (`label` → `textRef.current.textContent` → `element.textContent`, `:76-81`), re-seed
/// `nextIndexRef` from the final length (`:84`), and build the published map in index order.
fn sync_refs<M: Clone>(inner: &Rc<ListInner<M>>, items: &[SnapshotItem<M>]) -> CompositeListMap<M> {
    let mut entries = Vec::with_capacity(items.len());

    {
        let mut elements = inner.elements_ref.borrow_mut();
        elements.clear();
        for item in items {
            let index = item.index as usize;
            if elements.len() <= index {
                elements.resize(index + 1, None);
            }
            elements[index] = Some(item.element.clone());
            entries.push((
                item.element.clone(),
                CompositeMetadata {
                    index: item.index,
                    metadata: item.registration.metadata.clone(),
                },
            ));
        }
        inner.next_index_ref.set(elements.len() as i32);
    }

    if let Some(labels_ref) = &inner.labels_ref {
        let mut labels = labels_ref.borrow_mut();
        labels.clear();
        for item in items {
            let index = item.index as usize;
            if labels.len() <= index {
                labels.resize(index + 1, None);
            }
            labels[index] = resolve_label(item);
        }
    }

    CompositeListMap { entries }
}

/// The label fallback chain (`CompositeList.tsx:76-81`): an explicit label wins (including
/// the explicit `null` — no fallback), then the text ref's current text, then the element's
/// own text.
fn resolve_label<M>(item: &SnapshotItem<M>) -> Option<String> {
    match &item.registration.label {
        Some(explicit) => explicit.clone(),
        None => item
            .registration
            .text_ref
            .as_ref()
            .and_then(|text_ref| {
                text_ref
                    .borrow()
                    .as_ref()
                    .and_then(|text_element| text_element.text_content())
            })
            .or_else(|| item.element.text_content()),
    }
}

/// Upstream `observe` (`CompositeList.tsx:89-140`): (re)arm the MutationObserver over the
/// automatic items. The observer watches each adjacent pair's common ancestor — a reorder
/// must invert at least one adjacent pair, so the pair roots catch both direct item moves
/// and ancestor wrapper moves — and its callback re-sorts only after a *move*.
fn observe<M: Clone + PartialEq + 'static>(inner: &Rc<ListInner<M>>, sorted_nodes: &[Element]) {
    // `mutationObserverRef.current?.disconnect()` (`CompositeList.tsx:90-91`). The taken
    // tuple is dropped after the disconnect: the closure must outlive any in-flight
    // invocation, and no further invocation can arrive once disconnected.
    if let Some((observer, _closure)) = inner.observer.borrow_mut().take() {
        observer.disconnect();
    }

    // `typeof MutationObserver !== 'function' || sortedNodes.length < 2` (`:94`): a single
    // item can't reorder, and every environment this crate runs in has MutationObserver.
    if sorted_nodes.len() < 2 {
        return;
    }

    let on_mutations = {
        let inner = Rc::downgrade(inner);
        let sorted_nodes = sorted_nodes.to_vec();
        move |records: js_sys::Array, _observer: web_sys::MutationObserver| {
            let Some(inner) = inner.upgrade() else {
                return;
            };

            // Only verify the order after a move (`:99-105`): additions and removals alone
            // can't change the relative order of the remaining items.
            if !has_moved_node(&records) {
                return;
            }

            let mut previous_connected_node: Option<Element> = None;
            for node in &sorted_nodes {
                if !node.is_connected() {
                    continue;
                }

                if let Some(previous) = &previous_connected_node {
                    if compare_document_position_following(previous, node) {
                        // A connected node now appears before the previous connected node —
                        // wrappers/items moved and the index map needs to be rebuilt
                        // (`:109-120`).
                        if let Some((observer, _closure)) = inner.observer.borrow_mut().take() {
                            observer.disconnect();
                        }
                        schedule_map_update(&inner);
                        return;
                    }
                }

                previous_connected_node = Some(node.clone());
            }
        }
    };

    let closure =
        wasm_bindgen::closure::Closure::wrap(Box::new(on_mutations) as Box<dyn FnMut(_, _)>);
    let mutation_observer = web_sys::MutationObserver::new(closure.as_ref().unchecked_ref())
        .expect("MutationObserver constructor failed");

    // One root per adjacent pair's common ancestor (`:128-139`).
    let mut roots: Vec<Element> = Vec::new();
    for pair in sorted_nodes.windows(2) {
        if let Some(root) = get_common_ancestor(&pair[0], &pair[1]) {
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
    }
    for root in &roots {
        let init = web_sys::MutationObserverInit::new();
        init.set_child_list(true);
        mutation_observer
            .observe_with_options(root, &init)
            .expect("MutationObserver.observe failed");
    }

    *inner.observer.borrow_mut() = Some((mutation_observer, closure));
}

/// Upstream `getCommonAncestor` (`CompositeList.tsx:274-284`): walk `parentElement` up from
/// the first node until the walk reaches an ancestor containing the second. The
/// `parentElement` walk cannot cross shadow boundaries, so the native `contains` is
/// sufficient (upstream's comment).
fn get_common_ancestor(first_node: &Element, last_node: &Element) -> Option<Element> {
    let mut ancestor = first_node.parent_element();
    while let Some(current) = ancestor {
        if current.contains(Some(last_node)) {
            return Some(current);
        }
        ancestor = current.parent_element();
    }
    None
}

/// Upstream `hasMovedNode` (`CompositeList.tsx:286-296`): a batch moves a node when any
/// removed node is still connected (removed-then-re-added in the same batch).
fn has_moved_node(records: &js_sys::Array) -> bool {
    for record in records.iter() {
        let record: web_sys::MutationRecord = record.unchecked_into();
        let removed = record.removed_nodes();
        for index in 0..removed.length() {
            let Some(node) = removed.item(index) else {
                continue;
            };
            let node: web_sys::Node = node.unchecked_into();
            if node.is_connected() {
                return true;
            }
        }
    }
    false
}

/// Whether `b` follows `a` in document order — `DOCUMENT_POSITION_FOLLOWING` is always
/// reported alongside `CONTAINED_BY`, so testing that one bit orders siblings and nested
/// items alike. Returns `true` when `b` follows `a` (the sort's "a before b" answer).
pub fn compare_document_position_following(a: &Element, b: &Element) -> bool {
    const DOCUMENT_POSITION_FOLLOWING: u16 = 4;
    a.compare_document_position(b) & DOCUMENT_POSITION_FOLLOWING != 0
}

// ---------------------------------------------------------------------------
// Flush scheduling
// ---------------------------------------------------------------------------

/// The flush work item — a coalesced one-shot the scheduler owns until it runs.
type FlushWork = Rc<dyn Fn()>;

// The host scheduler override — the `timeout_manager.rs` dispatch-override precedent: host
// builds have no microtask queue, so tests install a manual scheduler to pin the
// coalescing contract deterministically. Absent an override the host flush runs
// synchronously at the end of `register`/`unregister`.
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static FLUSH_SCHEDULER_OVERRIDE: RefCell<Option<Rc<dyn Fn(FlushWork)>>> =
        const { RefCell::new(None) };
}

/// Installs (or clears) the host flush scheduler override. wasm builds schedule through
/// `window.queueMicrotask` and have no override.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_flush_scheduler_override(scheduler: Option<Rc<dyn Fn(FlushWork)>>) {
    FLUSH_SCHEDULER_OVERRIDE.with(|cell| *cell.borrow_mut() = scheduler);
}

/// Schedules the coalesced flush (module docs): one microtask on wasm, the override or an
/// immediate run on host.
fn schedule_flush(work: FlushWork) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let function =
                js_sys::Function::from(wasm_bindgen::closure::Closure::once_into_js(move || {
                    work()
                }));
            window.queue_microtask(&function);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let scheduler = FLUSH_SCHEDULER_OVERRIDE.with(|cell| cell.borrow().clone());
        match scheduler {
            Some(scheduler) => scheduler(work),
            None => work(),
        }
    }
}

// ---------------------------------------------------------------------------
// React context (CompositeListContext.ts)
// ---------------------------------------------------------------------------

/// The context bag — upstream's `CompositeListContextValue`
/// (`packages/react/src/internals/composite/list/CompositeListContext.ts:11-16`): the two
/// registration calls, the publication subscription, and the shared guess counter. Clones
/// share the underlying registry.
pub struct CompositeListContextValue<M> {
    /// `register` (`:12`).
    pub register: Rc<dyn Fn(&Element, CompositeListRegistration<M>)>,
    /// `unregister` (`:13`).
    pub unregister: Rc<dyn Fn(&Element)>,
    /// `subscribeMapChange` (`:14`).
    pub subscribe_map_change: Rc<dyn Fn(CompositeListListener<M>) -> CompositeListUnsubscribe>,
    /// `nextIndexRef` (`:15`) — the shared claim counter.
    pub next_index_ref: Rc<Cell<i32>>,
}

impl<M> Clone for CompositeListContextValue<M> {
    fn clone(&self) -> Self {
        Self {
            register: Rc::clone(&self.register),
            unregister: Rc::clone(&self.unregister),
            subscribe_map_change: Rc::clone(&self.subscribe_map_change),
            next_index_ref: Rc::clone(&self.next_index_ref),
        }
    }
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `floating_delay_group.rs`
/// precedent).
pub type SharedCompositeListContext<M> = SendWrapper<CompositeListContextValue<M>>;

thread_local! {
    /// The no-op default's `nextIndexRef` (`CompositeListContext.ts:22`): upstream's default
    /// context object is one module-level singleton, so every provider-less guess claimant
    /// draws from one shared counter. Shared per thread — the crate's realm.
    static NOOP_NEXT_INDEX_REF: Rc<Cell<i32>> = Rc::new(Cell::new(0));
}

/// Upstream's default context value (`CompositeListContext.ts:18-23`): no-op
/// `register`/`unregister`, a no-op subscription, and the shared counter — "the default
/// context no-ops keep a stray item inert rather than throwing" (behavior spec, "without a
/// parent list", `CompositeList.test.tsx:1140-1153`).
fn noop_composite_list_context<M>() -> CompositeListContextValue<M> {
    CompositeListContextValue {
        register: Rc::new(|_, _| {}),
        unregister: Rc::new(|_| {}),
        subscribe_map_change: Rc::new(|_listener| CompositeListUnsubscribe {
            unsubscribe: Rc::new(|| {}),
        }),
        next_index_ref: NOOP_NEXT_INDEX_REF.with(Rc::clone),
    }
}

/// Publishes the composite list registry for the current subtree — upstream's
/// `CompositeListContext.Provider` (`CompositeList.tsx:214-216`): builds the registry from
/// the caller-owned ref handles and `onMapChange`, provides the context bag, and returns it.
/// The registry tears down with the providing owner (the [`Drop`] cleanups).
///
/// Must be called inside a reactive owner (a component), like the other hook ports.
pub fn provide_composite_list<M>(
    elements_ref: CompositeListElementsRef,
    labels_ref: Option<CompositeListLabelsRef>,
    on_map_change: impl Fn(&CompositeListMap<M>) + 'static,
) -> CompositeListContextValue<M>
where
    M: Clone + PartialEq + 'static,
{
    let list = CompositeList::new(elements_ref, labels_ref, on_map_change);
    let value = list.context_value();
    provide_context(SharedCompositeListContext::new(value.clone()));
    value
}

impl<M> CompositeList<M>
where
    M: Clone + PartialEq + 'static,
{
    /// The context bag over this registry (`CompositeList.tsx:209-212`).
    pub fn context_value(&self) -> CompositeListContextValue<M> {
        let list = self.clone();
        let register = Rc::new(
            move |node: &Element, registration: CompositeListRegistration<M>| {
                list.register(node, registration);
            },
        );

        let list = self.clone();
        let unregister = Rc::new(move |node: &Element| {
            list.unregister(node);
        });

        let list = self.clone();
        let subscribe_map_change = Rc::new(move |listener: CompositeListListener<M>| {
            list.subscribe_map_change_listener(listener)
        });

        CompositeListContextValue {
            register,
            unregister,
            subscribe_map_change,
            next_index_ref: Rc::clone(&self.inner.next_index_ref),
        }
    }

    /// [`CompositeList::subscribe_map_change`] over an already-erased listener.
    pub fn subscribe_map_change_listener(
        &self,
        listener: CompositeListListener<M>,
    ) -> CompositeListUnsubscribe {
        let id = self.inner.next_listener_id.get();
        self.inner.next_listener_id.set(id + 1);
        self.inner.listeners.borrow_mut().push((id, listener));

        let inner = Rc::downgrade(&self.inner);
        CompositeListUnsubscribe {
            unsubscribe: Rc::new(move || {
                if let Some(inner) = inner.upgrade() {
                    inner
                        .listeners
                        .borrow_mut()
                        .retain(|(entry_id, _)| *entry_id != id);
                }
            }),
        }
    }
}

/// Reads the ambient composite list registry — upstream's `useCompositeListContext`
/// (`CompositeListContext.ts:25-27`): the provided value, or the shared no-op default when
/// no list is in scope.
pub fn use_composite_list_context<M>() -> CompositeListContextValue<M>
where
    M: Clone + PartialEq + 'static,
{
    use_context::<SharedCompositeListContext<M>>()
        .map(|shared| (*shared).clone())
        .unwrap_or_else(noop_composite_list_context)
}

/// The item-side ref callback type — [`leptos_ui_utils::RefCallback`] over the registry's
/// element type, so the future view layer can merge it with consumer refs through the
/// crate's `use_merged_refs` vocabulary (upstream's `ref: (node: HTMLElement | null) => void`
/// is the same no-cleanup-callback protocol, `useCompositeListItem.ts:25`).
pub type CompositeItemRefCallback = RefCallback<Element>;

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{GetUntracked, Set};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::HtmlElement;

    use super::*;

    use crate::use_composite_list_item::{
        UseCompositeListItem, UseCompositeListItemParams, use_composite_list_item,
    };

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type Metadata = String;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// A connected item element carrying its label as text content — the analog of the
    /// upstream `Item` fixture's `<div data-testid={label}>{label}</div>`
    /// (`CompositeList.test.tsx:13-20`).
    fn item_element(label: &str) -> HtmlElement {
        let element = document()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        element.set_text_content(Some(label));
        document().body().unwrap().append_child(&element).unwrap();
        element
    }

    /// Lets the queued flush (and, when an out-of-band DOM move is involved, the observer
    /// callback that schedules it) run: one awaited resolved promise per microtask tick —
    /// the flush microtask was queued before the await's continuation.
    async fn run_microtasks(ticks: usize) {
        for _ in 0..ticks {
            JsFuture::from(js_sys::Promise::resolve(&JsValue::undefined()))
                .await
                .unwrap();
        }
    }

    struct Harness {
        _owner: Owner,
        _context: CompositeListContextValue<Metadata>,
        elements: CompositeListElementsRef,
        labels: CompositeListLabelsRef,
        publications: Rc<Cell<usize>>,
        last_map: Rc<std::cell::RefCell<Option<CompositeListMap<Metadata>>>>,
    }

    /// The upstream render fixture — `provide_composite_list` under a fresh owner with
    /// caller-owned ref handles and a counting `onMapChange`
    /// (`CompositeList.test.tsx:22-35`).
    fn harness() -> Harness {
        let owner = owner();
        let elements: CompositeListElementsRef = Rc::new(RefCell::new(Vec::new()));
        let labels: CompositeListLabelsRef = Rc::new(RefCell::new(Vec::new()));
        let publications = Rc::new(Cell::new(0));
        let last_map = Rc::new(std::cell::RefCell::new(None));

        let list = provide_composite_list(Rc::clone(&elements), Some(Rc::clone(&labels)), {
            let publications = Rc::clone(&publications);
            let last_map = Rc::clone(&last_map);
            move |map| {
                publications.set(publications.get() + 1);
                *last_map.borrow_mut() = Some(map.clone());
            }
        });

        Harness {
            _owner: owner,
            _context: list,
            elements,
            labels,
            publications,
            last_map,
        }
    }

    fn unindexed_params() -> UseCompositeListItemParams<Metadata, RwSignal<Option<i32>>> {
        UseCompositeListItemParams {
            guess: false,
            index: RwSignal::new(None),
            label: None,
            metadata: None,
            text_ref: None,
        }
    }

    fn explicit_params(index: i32) -> UseCompositeListItemParams<Metadata, RwSignal<Option<i32>>> {
        UseCompositeListItemParams {
            guess: false,
            index: RwSignal::new(Some(index)),
            label: None,
            metadata: None,
            text_ref: None,
        }
    }

    fn attach(item: &UseCompositeListItem<Metadata>, node: &HtmlElement) {
        (item.ref_callback)(Some(node));
    }

    fn detach(item: &UseCompositeListItem<Metadata>) {
        (item.ref_callback)(None);
    }

    fn index_of(map: &CompositeListMap<Metadata>, node: &HtmlElement) -> i32 {
        map.get(node).expect("node is published").index
    }

    // Mirrors `CompositeList.test.tsx:22-43` — "cleans up refs on unmount": the refs sync
    // to the mounted items in order, and the teardown (the upstream unmount cleanup,
    // `CompositeList.tsx:179-184`) empties both arrays.
    #[wasm_bindgen_test]
    async fn registers_items_in_dom_order_and_clears_the_refs_on_teardown() {
        let elements_ref = Rc::new(RefCell::new(Vec::new()));
        let labels_ref = Rc::new(RefCell::new(Vec::new()));
        {
            let _owner = owner();
            let list = provide_composite_list::<Metadata>(
                Rc::clone(&elements_ref),
                Some(Rc::clone(&labels_ref)),
                |_| {},
            );

            let a = item_element("a");
            let b = item_element("b");
            let c = item_element("c");
            let (i_a, i_b, i_c) = (
                use_composite_list_item::<Metadata, _>(unindexed_params()),
                use_composite_list_item::<Metadata, _>(unindexed_params()),
                use_composite_list_item::<Metadata, _>(unindexed_params()),
            );
            attach(&i_a, &a);
            attach(&i_b, &b);
            attach(&i_c, &c);
            run_microtasks(1).await;

            assert_eq!(elements_ref.borrow().len(), 3);
            assert_eq!(labels_ref.borrow().len(), 3);
            assert_eq!(
                elements_ref
                    .borrow()
                    .iter()
                    .map(|e| e.as_ref().unwrap().text_content())
                    .collect::<Vec<_>>(),
                vec![Some("a".into()), Some("b".into()), Some("c".into())],
            );
            assert_eq!(
                labels_ref.borrow().clone(),
                vec![Some("a".into()), Some("b".into()), Some("c".into())],
            );

            drop(list);
        }
        assert_eq!(
            elements_ref.borrow().len(),
            0,
            "the teardown clears elements"
        );
        assert_eq!(labels_ref.borrow().len(), 0, "the teardown clears labels");
    }

    // Mirrors `CompositeList.test.tsx:898-917` — "resolves each label source": explicit
    // label → explicit `null` (no fallback) → textRef → element text.
    #[wasm_bindgen_test]
    async fn the_label_fallback_chain_holds() {
        let harness = harness();

        let explicit = item_element("explicit-ignored");
        let null_label = item_element("not a label");
        let text_ref_item = item_element("from text ref-ignored");
        let element_text = item_element("from element");

        let text_slot: TextRef = Rc::new(RefCell::new(None));
        let span = document()
            .create_element("span")
            .unwrap()
            .unchecked_into::<HtmlElement>();
        span.set_text_content(Some("from text ref"));
        text_ref_item.append_child(&span).unwrap();

        let make = |label: RegistrationLabel, text_ref: Option<TextRef>| {
            use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
                guess: false,
                index: RwSignal::new(None),
                label,
                metadata: None,
                text_ref,
            })
        };

        attach(&make(Some(Some("explicit label".into())), None), &explicit);
        attach(&make(Some(None), None), &null_label);
        *text_slot.borrow_mut() = Some(span.unchecked_into());
        attach(&make(None, Some(Rc::clone(&text_slot))), &text_ref_item);
        attach(&make(None, None), &element_text);
        run_microtasks(1).await;

        assert_eq!(
            harness.labels.borrow().clone(),
            vec![
                Some("explicit label".into()),
                None,
                Some("from text ref".into()),
                Some("from element".into()),
            ],
        );
    }

    // Mirrors `CompositeList.test.tsx:119-140` — "registers explicitly indexed items in
    // their index-addressed slots": the published map is index order regardless of
    // registration order.
    #[wasm_bindgen_test]
    async fn explicit_indexes_reserve_their_slots() {
        let harness = harness();

        let two = item_element("two");
        let zero = item_element("zero");
        let one = item_element("one");

        let (i_two, i_zero, i_one) = (
            use_composite_list_item::<Metadata, _>(explicit_params(2)),
            use_composite_list_item::<Metadata, _>(explicit_params(0)),
            use_composite_list_item::<Metadata, _>(explicit_params(1)),
        );
        attach(&i_two, &two);
        attach(&i_zero, &zero);
        attach(&i_one, &one);
        run_microtasks(1).await;

        let map = harness.last_map.borrow().clone().unwrap();
        assert_eq!(
            map.iter()
                .map(|(_, metadata)| metadata.index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2],
        );
        assert_eq!(
            harness.elements.borrow().clone(),
            vec![
                Some(zero.unchecked_into()),
                Some(one.unchecked_into()),
                Some(two.unchecked_into()),
            ],
        );
    }

    // Mirrors `CompositeList.test.tsx:142-162` — "reserves explicit slots when assigning
    // automatic indexes": automatic items fill around the reservation and their published
    // indexes correct through the subscription.
    #[wasm_bindgen_test]
    async fn automatic_items_fill_around_reserved_slots() {
        let harness = harness();

        let auto_one = item_element("automatic one");
        let explicit_zero = item_element("explicit zero");
        let auto_two = item_element("automatic two");

        let i_auto_one = use_composite_list_item::<Metadata, _>(unindexed_params());
        let i_explicit = use_composite_list_item::<Metadata, _>(explicit_params(0));
        let i_auto_two = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&i_auto_one, &auto_one);
        attach(&i_explicit, &explicit_zero);
        attach(&i_auto_two, &auto_two);
        run_microtasks(1).await;

        assert_eq!(
            harness.elements.borrow().clone(),
            vec![
                Some(explicit_zero.unchecked_into()),
                Some(auto_one.unchecked_into()),
                Some(auto_two.unchecked_into()),
            ],
        );
        assert_eq!(i_auto_one.index.get_untracked(), 1);
        assert_eq!(i_auto_two.index.get_untracked(), 2);
    }

    // Mirrors `CompositeList.test.tsx:294-307` — "does not register negative explicit
    // indexes": the item falls out of the snapshot entirely.
    #[wasm_bindgen_test]
    async fn negative_explicit_indexes_are_ignored() {
        let harness = harness();

        let node = item_element("item");
        let item = use_composite_list_item::<Metadata, _>(explicit_params(-1));
        attach(&item, &node);
        run_microtasks(1).await;

        assert_eq!(harness.elements.borrow().len(), 0);
    }

    // Mirrors `CompositeList.test.tsx:80-117` — "only publishes maps that are aligned with
    // the element registry": N registrations in one stack coalesce into one publication,
    // and each publication's keys equal the elements ref contents.
    #[wasm_bindgen_test]
    async fn registrations_in_one_stack_publish_once_aligned() {
        let harness = harness();

        let nodes: Vec<_> = ["a", "b", "c"].iter().map(|l| item_element(l)).collect();
        for node in &nodes {
            let item = use_composite_list_item::<Metadata, _>(unindexed_params());
            attach(&item, node);
        }
        run_microtasks(1).await;

        assert_eq!(
            harness.publications.get(),
            1,
            "one publication for the batch"
        );
        let map = harness.last_map.borrow().clone().unwrap();
        let keys: Vec<_> = map.elements().cloned().collect();
        let elements: Vec<_> = harness
            .elements
            .borrow()
            .iter()
            .map(|e| e.clone().unwrap())
            .collect();
        assert_eq!(
            keys, elements,
            "the published map is aligned with the registry"
        );

        let d = item_element("d");
        let item = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&item, &d);
        run_microtasks(1).await;

        assert_eq!(
            harness.publications.get(),
            2,
            "the growth publishes once more"
        );
    }

    // Mirrors `CompositeList.test.tsx:199-250` — "syncs replacement refs without publishing
    // an unchanged map": a re-registration whose metadata/index/element all compare equal
    // syncs the refs but republishes nothing.
    #[wasm_bindgen_test]
    async fn re_registration_with_unchanged_params_does_not_republish() {
        let harness = harness();

        let node = item_element("item");
        let item = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&item, &node);
        run_microtasks(1).await;
        assert_eq!(harness.publications.get(), 1);

        (item.set_params)(None, None, None);
        run_microtasks(1).await;

        assert_eq!(
            harness.publications.get(),
            1,
            "no republication for equal params"
        );
        assert_eq!(
            harness.elements.borrow().clone(),
            vec![Some(node.unchecked_into())],
        );
    }

    // Mirrors `CompositeList.test.tsx:385-420` — "assigns correct guessed indexes during
    // the first render": the claim happens at hook-call time in hook-call order (the analog
    // of the `useState` initializer's first render, `useCompositeListItem.ts:42-55`).
    #[wasm_bindgen_test]
    async fn guessed_items_claim_render_order_at_hook_call_time() {
        let _harness = harness();

        let context = use_composite_list_context::<Metadata>();
        let before = context.next_index_ref.get();

        let guessed = |index: RwSignal<Option<i32>>| {
            use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
                guess: true,
                index,
                label: None,
                metadata: None,
                text_ref: None,
            })
        };
        let a = guessed(RwSignal::new(None));
        let b = guessed(RwSignal::new(None));
        let c = guessed(RwSignal::new(None));
        assert_eq!(
            (
                a.index.get_untracked(),
                b.index.get_untracked(),
                c.index.get_untracked()
            ),
            (before, before + 1, before + 2),
            "the guesses are correct at hook-call time"
        );
    }

    // Mirrors `CompositeList.test.tsx:164-197` — "does not consume an index guess for an
    // explicitly indexed item": in a fresh list, an explicitly indexed guessed item claims
    // nothing and the automatic sibling claims the first slot.
    #[wasm_bindgen_test]
    async fn explicit_items_consume_no_guess() {
        let harness = harness();

        let explicit = use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
            guess: true,
            index: RwSignal::new(Some(1)),
            label: None,
            metadata: None,
            text_ref: None,
        });
        let automatic = use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
            guess: true,
            index: RwSignal::new(None),
            label: None,
            metadata: None,
            text_ref: None,
        });
        assert_eq!(explicit.index.get_untracked(), 1);
        assert_eq!(
            automatic.index.get_untracked(),
            0,
            "the explicit sibling consumed no guess"
        );

        let explicit_node = item_element("explicit");
        let automatic_node = item_element("automatic");
        attach(&explicit, &explicit_node);
        attach(&automatic, &automatic_node);
        run_microtasks(1).await;

        let map = harness.last_map.borrow().clone().unwrap();
        assert_eq!(index_of(&map, &automatic_node), 0);
        assert_eq!(index_of(&map, &explicit_node), 1);
    }

    // Mirrors `CompositeList.test.tsx:422-471` — "re-registers an item when its explicit
    // index changes or is removed": the re-registration lands at the new slot with the old
    // one empty, and dropping the explicit index falls back to an automatic index through
    // the subscription.
    #[wasm_bindgen_test]
    async fn set_params_re_registers_an_item_at_its_new_explicit_slot() {
        let harness = harness();

        let tracked = item_element("tracked");
        let index_source = RwSignal::new(Some(0));
        let item = use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
            guess: false,
            index: index_source.clone(),
            label: None,
            metadata: None,
            text_ref: None,
        });
        attach(&item, &tracked);
        run_microtasks(1).await;
        assert_eq!(
            harness.elements.borrow()[0].as_ref().unwrap(),
            tracked.unchecked_ref::<Element>(),
        );

        index_source.set(Some(2));
        (item.set_params)(None, None, None);
        run_microtasks(1).await;

        let elements = harness.elements.borrow();
        assert!(elements[0].is_none(), "the old slot is empty");
        assert_eq!(
            elements[2].as_ref().unwrap(),
            tracked.unchecked_ref::<Element>()
        );
        drop(elements);
        assert_eq!(item.index.get_untracked(), 2);

        index_source.set(None);
        (item.set_params)(None, None, None);
        run_microtasks(1).await;

        assert_eq!(
            item.index.get_untracked(),
            0,
            "the automatic fallback resolves"
        );
        assert_eq!(
            harness.elements.borrow().clone(),
            vec![Some(tracked.unchecked_into())],
        );
    }

    // Mirrors `CompositeList.test.tsx:494-531` — "does not detach item refs when an index
    // shifts": a sibling registered before the item shifts its published index through the
    // subscription, without a re-attach cycle.
    #[wasm_bindgen_test]
    async fn an_index_shift_does_not_re_attach_the_item() {
        let harness = harness();

        let tracked = item_element("tracked");
        let item = use_composite_list_item::<Metadata, _>(unindexed_params());
        let calls = Rc::new(Cell::new(0));
        let tracked_ref = {
            let calls = Rc::clone(&calls);
            let inner = Rc::clone(&item.ref_callback);
            Rc::new(move |node: Option<&Element>| {
                calls.set(calls.get() + 1);
                inner(node)
            })
        };
        (tracked_ref)(Some(&tracked));
        run_microtasks(1).await;
        assert_eq!(calls.get(), 1);

        let before = item_element("before");
        let before_item = use_composite_list_item::<Metadata, _>(unindexed_params());
        tracked
            .parent_element()
            .unwrap()
            .insert_before(&before, Some(tracked.unchecked_ref::<web_sys::Node>()))
            .unwrap();
        attach(&before_item, &before);
        run_microtasks(2).await;

        assert_eq!(item.index.get_untracked(), 1, "the shift propagated");
        assert_eq!(
            calls.get(),
            1,
            "no re-attach cycle for a published-index shift"
        );
        assert_eq!(
            harness.elements.borrow().clone(),
            vec![
                Some(before.unchecked_into()),
                Some(tracked.unchecked_into()),
            ],
        );
    }

    // Mirrors `CompositeList.test.tsx:533-573` — "excludes items detached outside React
    // from the registry": a node removed without a ref callback stays registered but is
    // skipped by the snapshot, leaving the other items' order intact.
    #[wasm_bindgen_test]
    async fn detached_items_are_excluded_from_the_registry() {
        let harness = harness();

        let a = item_element("a");
        let b = item_element("b");
        let c = item_element("c");
        let (i_a, i_b, i_c) = (
            use_composite_list_item::<Metadata, _>(unindexed_params()),
            use_composite_list_item::<Metadata, _>(unindexed_params()),
            use_composite_list_item::<Metadata, _>(unindexed_params()),
        );
        attach(&i_a, &a);
        attach(&i_b, &b);
        attach(&i_c, &c);
        run_microtasks(1).await;

        b.remove();
        let d = item_element("d");
        let i_d = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&i_d, &d);
        run_microtasks(1).await;

        assert_eq!(
            harness
                .elements
                .borrow()
                .iter()
                .map(|e| e.as_ref().unwrap().text_content())
                .collect::<Vec<_>>(),
            vec![Some("a".into()), Some("c".into()), Some("d".into())],
        );
        let map = harness.last_map.borrow().clone().unwrap();
        assert_eq!(
            map.elements().map(|e| e.text_content()).collect::<Vec<_>>(),
            vec![Some("a".into()), Some("c".into()), Some("d".into())],
        );
    }

    // Mirrors `CompositeList.test.tsx:650-680` — "updates indexes when a leaf item moves
    // outside React": the observer's move detection re-sorts and republishes.
    #[wasm_bindgen_test]
    async fn out_of_band_moves_reindex_through_the_observer() {
        let harness = harness();

        let container = item_element("container");
        let a = item_element("a");
        let b = item_element("b");
        let c = item_element("c");
        container.append_child(&a).unwrap();
        container.append_child(&b).unwrap();
        container.append_child(&c).unwrap();

        let (i_a, i_b, i_c) = (
            use_composite_list_item::<Metadata, _>(unindexed_params()),
            use_composite_list_item::<Metadata, _>(unindexed_params()),
            use_composite_list_item::<Metadata, _>(unindexed_params()),
        );
        attach(&i_a, &a);
        attach(&i_b, &b);
        attach(&i_c, &c);
        run_microtasks(1).await;
        assert_eq!(
            a.get_attribute("data-index"),
            None,
            "the port owns no DOM attributes"
        );
        assert_eq!(index_of(&harness.last_map.borrow().clone().unwrap(), &a), 0);

        // `container.appendChild(a)` (`CompositeList.test.tsx:668`) — a direct DOM move.
        container.append_child(&a).unwrap();
        run_microtasks(2).await;

        let map = harness.last_map.borrow().clone().unwrap();
        assert_eq!(index_of(&map, &a), 2);
        assert_eq!(index_of(&map, &b), 0);
        assert_eq!(index_of(&map, &c), 1);
        assert_eq!(
            harness
                .elements
                .borrow()
                .iter()
                .map(|e| e.as_ref().unwrap().text_content())
                .collect::<Vec<_>>(),
            vec![Some("b".into()), Some("c".into()), Some("a".into())],
        );
    }

    // Mirrors `CompositeList.test.tsx:804-841` — "ignores mutations for unrelated leaf
    // nodes": adding/removing a non-item leaf under the observed root never republishes.
    #[wasm_bindgen_test]
    async fn unrelated_leaf_mutations_do_not_republish() {
        let harness = harness();

        let list_div = item_element("list");
        let a = item_element("a");
        let b = item_element("b");
        list_div.append_child(&a).unwrap();
        list_div.append_child(&b).unwrap();

        let (i_a, i_b) = (
            use_composite_list_item::<Metadata, _>(unindexed_params()),
            use_composite_list_item::<Metadata, _>(unindexed_params()),
        );
        attach(&i_a, &a);
        attach(&i_b, &b);
        run_microtasks(1).await;
        assert_eq!(harness.publications.get(), 1);

        let badge = document().create_element("span").unwrap();
        list_div.append_child(&badge).unwrap();
        badge.remove();
        run_microtasks(2).await;

        assert_eq!(
            harness.publications.get(),
            1,
            "pure add/remove batches never republish"
        );
    }

    // Mirrors `CompositeList.test.tsx:607-648` — "updates the registry when a mounted item
    // stops rendering an element": the item stays subscribed while its node is gone, the
    // publication misses cleanly, and the tail reindexes both ways.
    #[wasm_bindgen_test]
    async fn vanishing_items_reindex_and_survive_publications_to_gone_nodes() {
        let harness = harness();

        let tail = item_element("tail");
        let i_tail = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&i_tail, &tail);
        run_microtasks(1).await;
        assert_eq!(harness.elements.borrow().len(), 1);

        let vanishing = item_element("vanishing");
        let i_vanishing = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&i_vanishing, &vanishing);
        run_microtasks(1).await;
        assert_eq!(harness.elements.borrow().len(), 2);

        detach(&i_vanishing);
        run_microtasks(1).await;
        assert_eq!(
            index_of(&harness.last_map.borrow().clone().unwrap(), &tail),
            0,
            "the tail reindexes"
        );

        attach(&i_vanishing, &vanishing);
        run_microtasks(1).await;
        assert_eq!(harness.elements.borrow().len(), 2);
        assert_eq!(
            index_of(&harness.last_map.borrow().clone().unwrap(), &vanishing),
            1,
            "the re-attached node takes its DOM position"
        );
    }

    // Mirrors `CompositeList.test.tsx:968-991` — "publishes item metadata alongside the
    // index".
    #[wasm_bindgen_test]
    async fn metadata_is_published_alongside_the_index() {
        let harness = harness();

        let first = item_element("first");
        let second = item_element("second");
        let metadata_item = |metadata: Option<Metadata>| {
            use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
                guess: false,
                index: RwSignal::new(None),
                label: None,
                metadata,
                text_ref: None,
            })
        };
        let i_first = metadata_item(Some("alpha".into()));
        let i_second = metadata_item(Some("beta".into()));
        attach(&i_first, &first);
        attach(&i_second, &second);
        run_microtasks(1).await;

        let map = harness.last_map.borrow().clone().unwrap();
        assert_eq!(map.get(&first).unwrap().metadata, Some("alpha".to_string()));
        assert_eq!(map.get(&first).unwrap().index, 0);
        assert_eq!(map.get(&second).unwrap().metadata, Some("beta".to_string()));
        assert_eq!(map.get(&second).unwrap().index, 1);
    }

    // Mirrors `CompositeRoot.test.tsx:1400-1415` — "keeps the outer registration when items
    // share a DOM node": the LAST registration for a node owns the entry (upstream: the
    // outer ref attaches last; the port: the same attach order, driven explicitly). The
    // "inner update must not steal ownership" half is discharged by the future merged-refs
    // wiring, which cycles every merged ref in attach order — the hook-level mechanism
    // (registration order wins) is what this pin covers.
    #[wasm_bindgen_test]
    async fn the_last_registration_for_a_shared_node_owns_the_entry() {
        let harness = harness();

        let shared = item_element("shared");
        let metadata_item = |metadata: Option<Metadata>| {
            use_composite_list_item::<Metadata, _>(UseCompositeListItemParams {
                guess: false,
                index: RwSignal::new(None),
                label: None,
                metadata,
                text_ref: None,
            })
        };
        let inner = metadata_item(Some("inner".into()));
        let outer = metadata_item(Some("outer".into()));
        attach(&inner, &shared);
        attach(&outer, &shared);
        run_microtasks(1).await;

        let map = harness.last_map.borrow().clone().unwrap();
        assert_eq!(
            map.get(&shared).unwrap().metadata,
            Some("outer".to_string()),
            "the outer registration owns the shared entry"
        );
        assert_eq!(map.len(), 1, "one node, one entry");
    }

    // Mirrors `CompositeList.test.tsx:1140-1153` — "renders an item that is not wrapped in
    // a list": the default context no-ops keep a stray item inert rather than throwing.
    #[wasm_bindgen_test]
    async fn an_orphan_item_over_the_default_context_stays_inert() {
        let _owner = owner();

        let orphan = item_element("orphan");
        let item = use_composite_list_item::<Metadata, _>(unindexed_params());
        attach(&item, &orphan);
        detach(&item);
        run_microtasks(1).await;

        assert_eq!(item.index.get_untracked(), -1);
    }
}
