//! Port of `packages/react/src/internals/composite/list/useCompositeListItem.ts` — the
//! item-side registration hook (`TODO.md`, item `infra: internals`; the checkpoint sequence
//! recorded in that entry's note).
//!
//! Upstream (`packages/react/src/internals/composite/list/useCompositeListItem.ts:32-99`)
//! seeds the item's index two ways: an explicit `index` short-circuits everything
//! (`:55`, `externalIndex ?? internalIndex`); without one, a `guess: true` item claims
//! `nextIndexRef.current` during the first render via the `useState` initializer (`:42-55`)
//! — so the index is known at render time and `data-index` is right on the first paint,
//! while the server render (empty registry) reports `-1`. Non-guessed items subscribe via
//! `subscribeMapChange` and copy their published index into state (`:84-96`) — the
//! corrective path. The callback ref is *deliberately identity-sensitive* on
//! `metadata`/`label`/`textRef` (`:59-82`): detaching and reattaching it is the
//! re-registration signal, and since callback refs attach child-first, the LAST
//! registration for a shared DOM node wins — which is how a nested outer item's metadata
//! keeps ownership when both items register one node (the `CompositeRoot.test.tsx:1400-1415`
//! rule; `specs/library/internals/implementation.md`, "Two subtle mechanisms").
//!
//! Spec: `specs/library/internals/behavior.md` ("State model" — the `-1` default, guess
//! seeding, index defaults; "Edge cases" — the re-registration and no-detach pins) and
//! `specs/library/internals/implementation.md` ("Composite registry"). Every claim was
//! verified against the source before porting.
//!
//! ## Rust adaptations
//!
//! - The internal index is a signal behind a [`Memo`] — `externalIndex ?? internalIndex`
//!   (`:55`) becomes `index_source.get().unwrap_or(internal)` with both reads tracked, so
//!   the view layer can bind `data-index` reactively; the corrective re-render path becomes
//!   the subscription's signal write. The `-1` initial (`:42`) covers the SSR arm.
//! - The guess claim (`:46-50`) runs once at hook-call time — the analog of the
//!   `useState` initializer's single first-render run, with hook-call order standing in for
//!   render order.
//! - The identity-sensitive callback ref ports to the [`CompositeItemRefCallback`] attach/
//!   detach protocol: attach registers the node, detach unregisters it (`:62-82` verbatim),
//!   and registration order still decides shared-node ownership. Because Rust closures have
//!   no identity to recreate, the "params changed → the ref is recreated → React cycles it"
//!   path (`:59-82` deps) is an explicit [`UseCompositeListItem::set_params`] that re-runs
//!   the same unregister/register cycle with the fresh params when the item is attached.
//!   The reactive-prop-driven re-registration a real component needs is discharged by the
//!   future view layer wiring, which forwards prop changes into `set_params` (the same
//!   view-free deferral the `csp_provider` port records); the upstream warning against
//!   effect-based *publishing* (`:59-62`) is untouched — re-registration still goes through
//!   the same synchronous registration calls.
//! - The subscription (`:84-96`) is taken unconditionally with the external-index guard
//!   checked inside the listener: upstream skips subscribing when `externalIndex != null`,
//!   but the only thing the port's listener writes is the internal signal, which the memo
//!   ignores while the external source is `Some` — observably identical, and it keeps the
//!   dynamic `None → Some` external transitions working without re-subscribing.
//! - Registration metadata is generic `M: Clone + PartialEq` (the registry's change-gate
//!   bounds); the attach-time registration carries `index: index_source.get_untracked()`
//!   — upstream's `externalIndex ?? null` (`:75`).

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::computed::Memo;
use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;
use web_sys::Element;

use leptos_ui_utils::CleanupFn;

use crate::composite_list::{
    CompositeItemRefCallback, CompositeListContextValue, CompositeListMap,
    CompositeListRegistration, RegistrationLabel, TextRef, use_composite_list_context,
};

/// Upstream `UseCompositeListItemParameters` (`useCompositeListItem.ts:6-22`). The index
/// source is a tracked read (`Get<Option<i32>>`) — the port's reactive component prop; a
/// static value arrives as a signal holding it.
pub struct UseCompositeListItemParams<M, I> {
    /// `guess` (`:7-12`) — claim the initial index from render order.
    pub guess: bool,
    /// `index` (`:13`) — the explicit external index, `None` when automatic.
    pub index: I,
    /// `label` (`:14`) — see [`RegistrationLabel`].
    pub label: RegistrationLabel,
    /// `metadata` (`:15-19`).
    pub metadata: Option<M>,
    /// `textRef` (`:20-21`).
    pub text_ref: Option<TextRef>,
}

/// Upstream `UseCompositeListItemReturnValue` (`useCompositeListItem.ts:24-27`) plus the
/// explicit re-registration the port's non-recreatable closures need (module docs).
pub struct UseCompositeListItem<M> {
    /// `index` (`:26`) — the resolved index, reactive: the external source when `Some`,
    /// otherwise the internal (guessed or subscription-corrected) index.
    pub index: Memo<i32>,
    /// `ref` (`:25`) — the attach/detach registration callback.
    pub ref_callback: CompositeItemRefCallback,
    /// The identity-sensitive-ref analog: replaces the stored `label`/`metadata`/`textRef`
    /// and re-runs the unregister/register cycle if the item is attached. The external
    /// index comes from the same reactive source the hook was built with.
    pub set_params: Rc<dyn Fn(RegistrationLabel, Option<M>, Option<TextRef>)>,
}

struct ItemState<M> {
    node: Option<Element>,
    label: RegistrationLabel,
    metadata: Option<M>,
    text_ref: Option<TextRef>,
}

/// Port of `useCompositeListItem` (`useCompositeListItem.ts:32-99`). Must be called inside a
/// reactive owner (a component) — the subscription and detach cleanup register there.
pub fn use_composite_list_item<M, I>(
    params: UseCompositeListItemParams<M, I>,
) -> UseCompositeListItem<M>
where
    M: Clone + PartialEq + 'static,
    I: Clone + Get<Value = Option<i32>> + GetUntracked<Value = Option<i32>> + 'static,
{
    let UseCompositeListItemParams {
        guess,
        index: index_source,
        label,
        metadata,
        text_ref,
    } = params;

    // The index source rides a `SendWrapper` because the memo's compute closure must be
    // `Send + Sync` (reactive_graph's contract); the same bridge the context handles use.
    let index_source = SendWrapper::new(index_source);

    let context: CompositeListContextValue<M> = use_composite_list_context();

    // The guess claim (`useCompositeListItem.ts:42-55`): one read+increment of
    // `nextIndexRef.current` at first-render time, only for un-indexed guessed items.
    let internal = RwSignal::new(if guess && index_source.get_untracked().is_none() {
        let claimed = context.next_index_ref.get();
        context.next_index_ref.set(claimed + 1);
        claimed
    } else {
        -1
    });

    let index = Memo::new({
        let index_source = index_source.clone();
        move |_| index_source.get().unwrap_or_else(|| internal.get())
    });

    let state = Rc::new(RefCell::new(ItemState {
        node: None,
        label,
        metadata,
        text_ref,
    }));

    let build_registration =
        |state: &ItemState<M>, external_index: Option<i32>| CompositeListRegistration {
            metadata: state.metadata.clone(),
            index: external_index,
            label: state.label.clone(),
            text_ref: state.text_ref.clone(),
        };

    // The deliberately identity-sensitive callback ref (`:59-82`): detach unregisters the
    // previous node, attach registers the current one — attach order decides shared-node
    // ownership.
    let ref_callback: CompositeItemRefCallback = {
        let state = Rc::clone(&state);
        let index_source = index_source.clone();
        let register = Rc::clone(&context.register);
        let unregister = Rc::clone(&context.unregister);
        Rc::new(move |node: Option<&Element>| -> Option<CleanupFn> {
            let (previous, registration) = {
                let mut state = state.borrow_mut();
                let previous = state.node.take();
                let registration =
                    node.map(|_| build_registration(&state, index_source.get_untracked()));
                (previous, registration)
            };

            if let Some(previous) = previous {
                unregister(&previous);
            }

            if let (Some(node), Some(registration)) = (node, registration) {
                state.borrow_mut().node = Some(node.clone());
                register(node, registration);
            }

            None
        })
    };

    // The ref-identity-change analog (`:59-82` deps): fresh params replace the stored ones
    // and, when attached, the registration cycles once (unregister → register) — the exact
    // sequence React's detach/reattach produces.
    let set_params: Rc<dyn Fn(RegistrationLabel, Option<M>, Option<TextRef>)> = {
        let state = Rc::clone(&state);
        let index_source = index_source.clone();
        let register = Rc::clone(&context.register);
        let unregister = Rc::clone(&context.unregister);
        Rc::new(move |label, metadata, text_ref| {
            let (attached, registration) = {
                let mut state = state.borrow_mut();
                state.label = label;
                state.metadata = metadata;
                state.text_ref = text_ref;
                let attached = state.node.clone();
                let registration = build_registration(&state, index_source.get_untracked());
                (attached, registration)
            };

            if let Some(node) = attached {
                unregister(&node);
                register(&node, registration);
            }
        })
    };

    // The corrective subscription (`:84-96`): published indexes flow into the internal
    // signal while the item has no external index (module docs for the always-on
    // adaptation). The publication reaches a subscriber whose node is gone without
    // crashing — the lookup simply misses.
    {
        let state = Rc::clone(&state);
        let subscription =
            (context.subscribe_map_change)(Rc::new(move |map: &CompositeListMap<M>| {
                if index_source.get_untracked().is_some() {
                    return;
                }
                let node = state.borrow().node.clone();
                if let Some(node) = node {
                    if let Some(published) = map.get(&node) {
                        internal.set(published.index);
                    }
                }
            }));
        let subscription = SendWrapper::new(subscription);
        on_cleanup(move || subscription.unsubscribe());
    }

    // React's unmount ref detach (`:66-70`): the node is unregistered when the owner dies
    // even if no detach call ever arrived.
    {
        let cleanup = SendWrapper::new((Rc::clone(&state), Rc::clone(&context.unregister)));
        on_cleanup(move || {
            let (state, unregister) = &*cleanup;
            if let Some(node) = state.borrow_mut().node.take() {
                unregister(&node);
            }
        });
    }

    UseCompositeListItem {
        index,
        ref_callback,
        set_params,
    }
}

/// The erased hook return for the future view layer's context-free wiring — the same shape
/// with the params already fixed. Unused until the view layer lands; kept `pub` for the
/// component checkpoint to build on.
impl<M> UseCompositeListItem<M> {
    /// The `index` at a point in time (`getCompositeListSnapshot` consumers read the
    /// published value the same way).
    pub fn index_untracked(&self) -> i32 {
        self.index.get_untracked()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;

    use super::*;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn params(
        index: RwSignal<Option<i32>>,
    ) -> UseCompositeListItemParams<String, RwSignal<Option<i32>>> {
        UseCompositeListItemParams {
            guess: false,
            index,
            label: None,
            metadata: None,
            text_ref: None,
        }
    }

    // Pins the `-1` initial index (`useCompositeListItem.ts:42-53`): without a guess and
    // without an external index, the item starts unplaced — the SSR arm of behavior spec
    // "State model" ("`useCompositeListItem` index defaults to `-1` before registration
    // resolves").
    #[test]
    fn an_unindexed_unguessed_item_starts_at_negative_one() {
        let _owner = owner();

        let item = use_composite_list_item(params(RwSignal::new(None)));
        assert_eq!(item.index_untracked(), -1);
    }

    // Pins the external-index short-circuit (`:55`): the external index wins verbatim and
    // reacts to source changes, falling back to the internal index when it clears.
    #[test]
    fn the_external_index_short_circuits_and_tracks_reactively() {
        let _owner = owner();

        let external = RwSignal::new(Some(3));
        let item = use_composite_list_item(params(external));
        assert_eq!(item.index_untracked(), 3);

        external.set(None);
        assert_eq!(
            item.index_untracked(),
            -1,
            "falling back to the internal index"
        );

        external.set(Some(7));
        assert_eq!(item.index_untracked(), 7);
    }

    // Pins the guess claim against the shared no-op default counter
    // (`CompositeListContext.ts:22`): two provider-less guessed items draw consecutive
    // indexes from the module-singleton counter, and an explicit external index consumes
    // none (`useCompositeListItem.ts:44`, the `externalIndex == null && guess` gate).
    #[test]
    fn the_guess_claim_draws_consecutive_indexes_and_explicit_indexes_skip_it() {
        let _owner = owner();

        let context = use_composite_list_context::<String>();
        let before = context.next_index_ref.get();

        let guessed: UseCompositeListItem<String> =
            use_composite_list_item(UseCompositeListItemParams {
                guess: true,
                index: RwSignal::new(None),
                label: None,
                metadata: None,
                text_ref: None,
            });
        let next: UseCompositeListItem<String> =
            use_composite_list_item(UseCompositeListItemParams {
                guess: true,
                index: RwSignal::new(None),
                label: None,
                metadata: None,
                text_ref: None,
            });
        assert_eq!(guessed.index_untracked(), before);
        assert_eq!(next.index_untracked(), before + 1);

        let explicit: UseCompositeListItem<String> =
            use_composite_list_item(UseCompositeListItemParams {
                guess: true,
                index: RwSignal::new(Some(9)),
                label: None,
                metadata: None,
                text_ref: None,
            });
        assert_eq!(explicit.index_untracked(), 9);
        assert_eq!(
            context.next_index_ref.get(),
            before + 2,
            "the explicit index consumed no guess"
        );
    }

    // Pins the orphan contract (behavior spec, "without a parent list",
    // `CompositeList.test.tsx:1140-1153`): the default context's no-ops keep the item
    // inert — attach/detach and param updates are safe, no publication ever corrects the
    // index.
    #[test]
    fn an_orphan_item_stays_inert_over_the_default_context() {
        let _owner = owner();

        let item = use_composite_list_item(params(RwSignal::new(None)));
        (item.ref_callback)(None);
        assert_eq!(item.index_untracked(), -1);
        (item.set_params)(Some(Some("label".into())), None, None);
        assert_eq!(item.index_untracked(), -1);
    }
}
