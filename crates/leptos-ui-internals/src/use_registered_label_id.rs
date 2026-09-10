//! Port of `packages/react/src/utils/useRegisteredLabelId.ts` — the label-id satellite the
//! `useLabel` half of the labelable-provider unit consumes
//! (`packages/react/src/internals/labelable-provider/useLabel.ts:7`; the
//! stringifyLocale-over-formatNumber precedent: a private `packages/react/src/utils` module
//! pulled in as a required leaf dependency of the checkpoint that ports its consumer).
//!
//! Upstream is 20 lines: `id = useBaseUiId(idProp)` (`:10`), one layout effect registering the
//! id with the setter and clearing it on unmount (`:12-17`), and the return (`:19`). The
//! registration is what makes the provider's `labelId` track the rendered label so a control's
//! `aria-describedby` can point at it (`specs/library/internals/implementation.md`,
//! "Context providers/consumers" — the `LabelableContext` row's `labelId` member).
//!
//! ## Rust adaptations
//!
//! - The `setLabelId` parameter — upstream `React.Dispatch<React.SetStateAction<string |
//!   undefined>>` (`:8`) — is a union of two call shapes: the plain value dispatch
//!   (`setLabelId(id)`, `:13`) and the function-updater dispatch in the cleanup
//!   (`setLabelId((currentId) => currentId === id ? undefined : currentId)`, `:15`). The port
//!   collapses the union into one enum, [`LabelIdUpdate`]: [`LabelIdUpdate::Set`] carries the
//!   plain value, [`LabelIdUpdate::ClearIfCurrent`] carries the updater's captured id and asks
//!   the setter to clear only when its current value still equals it — the conditional that
//!   keeps a *later* label's registration from being clobbered by this hook's unmount cleanup.
//!   A React state setter answers the updater against its own current value; a consumer store
//!   implementing the setter does the same against its state.
//! - The layout effect's dep array `[id, setLabelId]` (`:17`) becomes the reactive tracking of
//!   [`use_iso_layout_effect`]: the body reads the id signal (tracked), so an id change re-runs
//!   the cycle — cleanup (clear-if-current) first, then the fresh registration, the same order
//!   React's cleanup-then-effect produces.
//! - The unmount cleanup ports to [`on_cleanup`] registered inside the effect: it fires before
//!   each re-run and once at owner disposal — upstream's cleanup-on-unmount for the stable
//!   `setLabelId` dep (`:14-16`).

use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, on_cleanup};
use reactive_graph::traits::Get;
use reactive_graph::wrappers::read::Signal;

use leptos_ui_utils::use_iso_layout_effect;

use crate::use_base_ui_id::use_base_ui_id;

/// One dispatch of the `setLabelId` parameter — the port's collapse of upstream's
/// `React.SetStateAction<string | undefined>` union (module docs).
#[derive(Clone, Debug, PartialEq)]
pub enum LabelIdUpdate {
    /// `setLabelId(next)` — the plain value dispatch (`useRegisteredLabelId.ts:13`).
    Set(Option<String>),
    /// `setLabelId((currentId) => currentId === id ? undefined : currentId)` — the cleanup's
    /// conditional clear (`:15`): clear only when the setter's current value still equals the
    /// carried id, so a later label's registration survives this hook's unmount.
    ClearIfCurrent(String),
}

/// The `setLabelId` parameter (`useRegisteredLabelId.ts:8`) — see [`LabelIdUpdate`].
pub type LabelIdSetter = Rc<dyn Fn(LabelIdUpdate)>;

/// Port of `useRegisteredLabelId` (`useRegisteredLabelId.ts:6-19`): resolves the label id
/// (the override wins verbatim, else a `base-ui-`-prefixed generated id), registers it with
/// `set_label_id` while mounted, and clears it — conditionally — on unmount. Must be called
/// inside a reactive owner (a component).
///
/// The `id_prop` parameter is the reactive source standing in for upstream's `idProp` prop
/// (`:6`) — a static override rides a derived signal (the `floating_portal.rs` convention),
/// and a changing override drives the re-registration cycle the dep array's `id` member
/// (`:17`) produces.
pub fn use_registered_label_id<I>(
    id_prop: I,
    set_label_id: LabelIdSetter,
) -> Signal<String, LocalStorage>
where
    I: Get<Value = Option<String>> + 'static,
{
    // `const id = useBaseUiId(idProp)` (`:10`).
    let id = use_base_ui_id(id_prop);

    // The registration effect (`:12-17`), tracked on the id signal (module docs).
    {
        let set_label_id = send_wrapper::SendWrapper::new(set_label_id);
        let id_signal = id.clone();
        use_iso_layout_effect(move || {
            let current = id_signal.get();
            set_label_id(LabelIdUpdate::Set(Some(current.clone())));
            let set_label_id = set_label_id.clone();
            on_cleanup(move || set_label_id(LabelIdUpdate::ClearIfCurrent(current)));
        });
    }

    // `return id` (`:19`).
    id
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::RefCell;

    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{GetUntracked, Set};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the id resolution (`:10` over `useBaseUiId.ts:9-11`): with no override the
    // generated id carries the `base-ui-` prefix. The registration dispatch itself runs
    // through the layout effect, which the host binding suppresses — the dispatch sequence
    // is pinned by the wasm suite.
    #[test]
    fn the_generated_id_carries_the_base_ui_prefix() {
        let owner = in_owner();

        let updates = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&updates);
        let setter: LabelIdSetter = Rc::new(move |update| sink.borrow_mut().push(update));

        let id = use_registered_label_id(Signal::derive_local(|| None::<String>), setter);

        assert!(
            id.get_untracked().starts_with("base-ui-"),
            "the generated id carries the base-ui prefix"
        );
        assert!(
            updates.borrow().is_empty(),
            "the host has no document: the registration effect is suppressed"
        );

        owner.cleanup();
    }

    // Pins the override passthrough (`:10`): an explicit id prop wins verbatim.
    #[test]
    fn an_explicit_id_prop_overrides_the_generated_id() {
        let owner = in_owner();

        let updates = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&updates);
        let setter: LabelIdSetter = Rc::new(move |update| sink.borrow_mut().push(update));

        let id = use_registered_label_id(
            Signal::derive_local(|| Some("my-label".to_string())),
            setter,
        );

        assert_eq!(id.get_untracked(), "my-label");

        owner.cleanup();
    }

    // Pins the non-DOM binding's suppression on the host (`packages/utils/src/
    // useIsoLayoutEffect.ts:6` — the port's environment selection): with no `document`
    // global the registration effect is discarded, so no dispatch reaches the setter.
    #[test]
    fn the_host_binding_does_not_run_the_registration() {
        let _owner = in_owner();

        let updates = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&updates);
        let setter: LabelIdSetter = Rc::new(move |update| sink.borrow_mut().push(update));

        let _ = use_registered_label_id(Signal::derive_local(|| None::<String>), setter);

        assert!(
            updates.borrow().is_empty(),
            "the host has no document: the layout-effect registration is suppressed"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::{Get, GetUntracked, Set};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // A settable stand-in for the consumer side of the contract: a signal store answering
    // both dispatch shapes the way a React state setter would.
    #[derive(Clone)]
    struct LabelIdStore {
        value: RwSignal<Option<String>, LocalStorage>,
    }

    impl LabelIdStore {
        fn new() -> (Self, LabelIdSetter) {
            let store = Self {
                value: RwSignal::new_local(None),
            };
            let handle = store.clone();
            (store, Rc::new(move |update| match update {
                LabelIdUpdate::Set(next) => handle.value.set(next),
                LabelIdUpdate::ClearIfCurrent(id) => {
                    if handle.value.get_untracked() == Some(id) {
                        handle.value.set(None);
                    }
                }
            }))
        }
    }

    // Pins the registration (`:12-14`) in a real DOM realm: the layout effect runs
    // synchronously during the hook call, so the id is registered by the time the hook
    // returns.
    #[wasm_bindgen_test]
    fn the_id_is_registered_synchronously() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (store, setter) = LabelIdStore::new();
        let id = use_registered_label_id(Signal::derive_local(|| None::<String>), setter);

        assert_eq!(store.value.get_untracked(), Some(id.get_untracked()));

        owner.cleanup();
    }

    // Pins the unmount cleanup's conditional (`:14-16`): disposal clears the registration
    // when the current value is still this hook's id. The store's signal is created before
    // the owner is entered — an owner-registered signal is disposed with its owner and
    // panics every later reader (the `use_is_hydrating.rs` ArcRwSignal note).
    #[wasm_bindgen_test]
    fn disposal_clears_the_registration_when_it_is_still_current() {
        let _ = Executor::init_futures_executor();

        let (store, setter) = LabelIdStore::new();
        let owner = in_owner();

        let id = use_registered_label_id(Signal::derive_local(|| None::<String>), setter);
        assert_eq!(store.value.get_untracked(), Some(id.get_untracked()));

        owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            store.value.get_untracked(),
            None,
            "the current registration is cleared on unmount"
        );
    }

    // Pins the conditional's guard: a later label overwrote the registration, so this hook's
    // unmount cleanup must NOT clobber it — the behavior the `currentId === id` updater
    // arm (`:15`) exists for.
    #[wasm_bindgen_test]
    fn disposal_keeps_a_later_labels_registration() {
        let _ = Executor::init_futures_executor();

        let (store, setter) = LabelIdStore::new();
        let owner = in_owner();

        let _ = use_registered_label_id(Signal::derive_local(|| None::<String>), setter);

        // A later label registered its own id (upstream: a second useRegisteredLabelId
        // instance set the shared labelId after this one).
        let later = "base-ui-later".to_string();
        store.value.set(Some(later.clone()));

        owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            store.value.get_untracked(),
            Some(later),
            "the cleanup leaves a non-current registration in place"
        );
    }

    // Pins the reactive re-registration cycle (the dep array's `id` member, `:17`): an id
    // change re-runs the cycle — the old id's ClearIfCurrent fires first, then the fresh id
    // is registered.
    #[wasm_bindgen_test]
    fn an_id_change_re_registers_through_the_cleanup_then_set_order() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let (store, setter) = LabelIdStore::new();
        let override_source: RwSignal<Option<String>, LocalStorage> = RwSignal::new_local(None);
        let id = use_registered_label_id(override_source, setter);

        let first = store.value.get_untracked();
        assert_eq!(first, Some(id.get_untracked()), "registered once");

        override_source.set(Some("renamed".to_string()));
        Executor::poll_local();

        assert_eq!(
            store.value.get_untracked(),
            Some("renamed".to_string()),
            "the fresh id is registered after the cleanup cleared the old one"
        );
        assert_eq!(id.get_untracked(), "renamed");

        owner.cleanup();
    }
}
