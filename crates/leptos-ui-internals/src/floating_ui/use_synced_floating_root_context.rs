//! Port of `packages/react/src/floating-ui-react/hooks/useSyncedFloatingRootContext.ts`
//! — the popup-store adapter, the second of the two construction paths feeding the same
//! `FloatingRootStore` class (`specs/library/floating-ui-react/implementation.md`,
//! "Store backbone": "Two construction paths feed the same store class"; the public
//! `useFloating` path ports in [`crate::floating_ui::use_floating_root_context`]).
//!
//! ## Rust adaptations
//!
//! - Upstream narrows the popup store through
//!   `SyncedFloatingRootContextStore<State> = Pick<ReactStore<…>, 'context' | 'state' |
//!   'useState' | 'useSyncedValue'>` (`useSyncedFloatingRootContext.ts:14-17`) so
//!   consumers need not provide unrelated store capabilities. The structural Pick is a
//!   TypeScript assignability device: the port takes the concrete
//!   `Rc<ReactStore<PopupStoreState<P>, PopupStoreContext<D>>>` handle and the compiler
//!   checks the members actually used.
//! - The generic `OpenChangeEventDetails extends BaseUIChangeEventDetails<string>`
//!   parameter (`useSyncedFloatingRootContext.ts:21,40`) collapses to the port's
//!   `OnOpenChangeFn` convention — `Rc<dyn Fn(bool, &BaseUIChangeEventDetails<String>)>`
//!   — the same collapse the floating store context made
//!   (`FloatingRootStore.ts:55-56`, ported in
//!   [`crate::floating_ui::floating_root_store`]'s context). Upstream's own cast
//!   `onOpenChange as (open, eventDetails: BaseUIChangeEventDetails<string>) => void`
//!   (`:58-61`) is this erasure in source form.
//! - The internal-store construction (`:63-76` — a `useRef` filled on the first render
//!   when no `floatingRootContext` option exists) ports to
//!   [`leptos_ui_utils::use_ref_with_init`] with the same option-presence gate: the
//!   factory runs once and creates the internal store only when the option is absent.
//! - `popupStore.useSyncedValue('floatingId', floatingId)` (`:80`) ports to
//!   [`ReactStore::use_synced_value`] with the option value wrapped in a derived
//!   signal — upstream re-applies the per-render value through the hook's dependency
//!   array; the port's hook body runs once, so the value syncs on the effect's first
//!   run (the per-component reactive floatingId is a popup-components concern; the
//!   store-side vocabulary accepts any `Get` source).
//! - The sync effect (`:82-107`) tracks the three `useState` selections
//!   (`open`, `activeTriggerElement`, and `popupElement`/`positionerElement` per the
//!   `treatPopupAsFloatingElement` flag) — the reactive analog of the dependency array
//!   `[open, floatingId, referenceElement, floatingElement, store]` (`:107`). The
//!   `positionReference` mirror check (`:102-104` — mirror only while
//!   `positionReference === referenceElement`, upstream's one-time mirror that stops
//!   once positioning diverges them, implementation.md "Two construction paths")
//!   compares against the pre-update state, exactly where upstream reads
//!   `store.state` before applying `valuesToSync`.
//! - The render-phase context patch (`:110-111` — `store.context.onOpenChange` and
//!   `store.context.nested` re-assigned every render) ports to the context's setter
//!   methods, which the port's floating store context exposes for exactly this
//!   (`hooks/useFloatingRootContext.ts:76-77` adaptation, floating_root_store module
//!   docs).
//! - Upstream has no test file for this hook (implementation.md "Anything in source not
//!   explained by any test", point 2: "zero test references … it only runs under the
//!   real popup components"). The `popupId` selector claims of
//!   `popupStoreUtils.test.tsx:764-796` are the one upstream test exercising it (in
//!   `infra: utils`' suite); they are mirrored here alongside port-owned pins for the
//!   mirroring behavior itself.

use std::rc::Rc;

use reactive_graph::traits::Get;
use reactive_graph::traits::GetUntracked;
use reactive_graph::traits::GetValue;
use reactive_graph::wrappers::read::Signal;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use leptos_ui_utils::react_store::ReactStore;
use leptos_ui_utils::use_iso_layout_effect;
use leptos_ui_utils::use_ref_with_init;

use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
use crate::floating_ui::popup_store::PopupStoreContext;
use crate::floating_ui::popup_store::PopupStoreState;
use crate::floating_ui::popup_store::selectors;
use crate::floating_ui::types::{OnOpenChangeFn, ReferenceType};

/// Port of `UseSyncedFloatingRootContextOptions`
/// (`useSyncedFloatingRootContext.ts:19-32`).
pub struct UseSyncedFloatingRootContextOptions<Payload, ChangeEventDetails> {
    /// The popup store to keep synced (`:23`) — see the module docs on the
    /// structural-narrowing adaptation.
    pub popup_store: Rc<ReactStore<PopupStoreState<Payload>, PopupStoreContext<ChangeEventDetails>>>,
    /// Whether the Popup element is passed to Floating UI as the floating element
    /// instead of the default Positioner (`:24-27`; default `false`, `:44`).
    pub treat_popup_as_floating_element: bool,
    /// An externally owned floating root context to sync into instead of the internal
    /// store (`:28`).
    pub floating_root_context: Option<Rc<FloatingRootStore>>,
    /// The floating element's id (`:29`).
    pub floating_id: Option<String>,
    /// Whether the popup is nested (`:30`).
    pub nested: bool,
    /// The open-change callback the synced store forwards to (`:31`).
    pub on_open_change: OnOpenChangeFn,
}

/// Port of `useSyncedFloatingRootContext(options)`
/// (`useSyncedFloatingRootContext.ts:38-114`): keeps a `FloatingRootStore` in sync
/// with the provided popup store, using the provided floating root context when one
/// exists and creating an internal `syncOnly` store otherwise. Must be called inside a
/// reactive owner.
pub fn use_synced_floating_root_context<Payload, ChangeEventDetails>(
    options: UseSyncedFloatingRootContextOptions<Payload, ChangeEventDetails>,
) -> Rc<FloatingRootStore>
where
    Payload: Clone + 'static,
    ChangeEventDetails: 'static,
{
    let UseSyncedFloatingRootContextOptions {
        popup_store,
        treat_popup_as_floating_element,
        floating_root_context,
        floating_id,
        nested,
        on_open_change,
    } = options;

    // The three popup-store selections (`:51-55`) — the named selectors
    // `open`/`activeTriggerElement`/`popupElement | positionerElement` at the port's
    // call sites.
    let open = popup_store.use_state(selectors::open);
    let reference_element = popup_store.use_state(selectors::active_trigger_element);
    let floating_element_selector: fn(&PopupStoreState<Payload>) -> Option<HtmlElement> =
        if treat_popup_as_floating_element {
            selectors::popup_element
        } else {
            selectors::positioner_element
        };
    let floating_element = popup_store.use_state(floating_element_selector);

    // The shared trigger registry (`:56` — the popup store's map lands in the synced
    // floating store's context; `PopupTriggerMap` clones share one registry).
    let trigger_elements = popup_store.context.trigger_elements.clone();

    // The internal-store construction (`:63-76`): created once, only when no
    // floating root context was provided, seeded with the hook-time popup state and
    // `syncOnly: true` — the popup store owns the real state in this mode.
    let has_provided_context = floating_root_context.is_some();
    let floating_id_for_init = floating_id.clone();
    let on_open_change_for_init = Rc::clone(&on_open_change);
    let internal_store: Option<Rc<FloatingRootStore>> = use_ref_with_init({
        let open = open.get_untracked();
        let reference_element = reference_element.get_untracked();
        let floating_element = floating_element.get_untracked();
        move || {
            if has_provided_context {
                return None;
            }
            Some(FloatingRootStore::new(FloatingRootStoreOptions {
                open,
                transition_status: None,
                reference_element: reference_element.map(ReferenceType::Element),
                // The floating store's `floatingElement` arm carries the base
                // `Element` (the port's `FloatingRootState`); the positioner/popup
                // selectors hand back `HtmlElement`s.
                floating_element: floating_element.map(|element| element.unchecked_into()),
                trigger_elements,
                floating_id: floating_id_for_init,
                sync_only: true,
                nested,
                on_open_change: Some(on_open_change_for_init),
            }))
        }
    })
    .get_value();

    // `const store = floatingRootContextProp ?? internalStoreRef.current!` (`:78`).
    let store = floating_root_context.or(internal_store).expect(
        "the internal store is created whenever no floating root context is provided",
    );

    // `popupStore.useSyncedValue('floatingId', floatingId)` (`:80`).
    let floating_id_signal = Signal::derive_local({
        let floating_id = floating_id.clone();
        move || floating_id.clone()
    });
    popup_store.use_synced_value(
        |state: &mut PopupStoreState<Payload>| &mut state.floating_id,
        floating_id_signal,
    );

    // The sync effect (`:82-107`).
    {
        let store = Rc::clone(&store);
        use_iso_layout_effect(move || {
            let open = open.get();
            let reference_element = reference_element.get();
            let floating_element = floating_element.get();

            // `isElement(referenceElement)` (`:98-100`): the popup store's active
            // trigger is a real element whenever present.
            let dom_reference_element = reference_element.clone();

            // The one-time positionReference mirror (`:102-104`): evaluated against
            // the pre-update state — once positioning diverges the two, the mirror
            // stops.
            let mirror_position_reference = {
                let state = store.get_snapshot();
                state.position_reference == state.reference_element
            };

            store.update(
                |state, _| {
                    state.open = open;
                    state.floating_id = floating_id.clone();
                    state.reference_element = reference_element
                        .as_ref()
                        .map(|element| ReferenceType::Element(element.clone()));
                    state.floating_element =
                        floating_element.as_ref().map(|element| element.clone().unchecked_into());
                    state.dom_reference_element = dom_reference_element;
                    if mirror_position_reference {
                        state.position_reference = reference_element
                            .as_ref()
                            .map(|element| ReferenceType::Element(element.clone()));
                    }
                    true
                },
            );
        });
    }

    // Keep non-reactive context values fresh for interactions that call
    // `store.setOpen` (`:109-111`).
    store.context.set_on_open_change(Some(on_open_change));
    store.context.set_nested(nested);

    store
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::cell::RefCell;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;

    use super::*;
    use crate::floating_ui::popup_store::create_initial_popup_store_state;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    type TestStore = Rc<ReactStore<PopupStoreState<()>, PopupStoreContext<()>>>;

    /// The upstream suite's store fixture (`popupStoreUtils.test.tsx:34-55`):
    /// `createInitialPopupStoreState` + the context carrying the same trigger map.
    fn create_store(mutate: impl FnOnce(&mut PopupStoreState<()>)) -> TestStore {
        let trigger_elements = PopupTriggerMap::new();
        let mut state =
            create_initial_popup_store_state(&trigger_elements, None, false);
        mutate(&mut state);
        Rc::new(ReactStore::with_context(
            state,
            PopupStoreContext {
                trigger_elements,
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change: None,
                on_open_change_complete: None,
            },
        ))
    }

    fn on_open_change_recorder(log: Rc<RefCell<Vec<bool>>>) -> OnOpenChangeFn {
        Rc::new(move |open: bool, _details| log.borrow_mut().push(open))
    }

    // Pins the construction-time half (`useSyncedFloatingRootContext.ts:63-76`,
    // `:110-111`): with no floating root context option, an internal store is
    // created — seeded with the hook-time popup state and the floatingId — and the
    // context patch lands synchronously. (The `syncOnly` forwarding half is pinned in
    // the wasm suite, which can construct the event details.)
    #[test]
    fn constructs_an_internal_sync_only_store_and_patches_its_context() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let log = Rc::new(RefCell::new(Vec::new()));
        let popup_store = create_store(|state| state.open = true);
        let store = use_synced_floating_root_context(UseSyncedFloatingRootContextOptions {
            popup_store: Rc::clone(&popup_store),
            treat_popup_as_floating_element: false,
            floating_root_context: None,
            floating_id: Some("popup-id".to_owned()),
            nested: true,
            on_open_change: on_open_change_recorder(Rc::clone(&log)),
        });

        // The internal store seeds from the hook-time popup state (`:65-75`).
        assert!(
            store.get_snapshot().open,
            "the internal store seeds open from the popup store"
        );
        assert!(
            store.get_snapshot().reference_element.is_none(),
            "no active trigger seeds a null reference"
        );
        assert_eq!(
            store.get_snapshot().floating_id,
            Some("popup-id".to_owned()),
            "the floatingId option seeds the internal store"
        );

        // The context patch (`:110-111`).
        assert!(
            store.context.on_open_change().is_some(),
            "onOpenChange is patched into the context"
        );
        assert!(store.context.nested(), "nested is patched into the context");
    }

    // Pins the provided-context branch (`:28`, `:78`): the returned store is the
    // provided floating root context itself — the same object identity upstream's
    // `??` preserves — and the context patch lands on it.
    #[test]
    fn uses_the_provided_floating_root_context_when_one_exists() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let popup_store = create_store(|_| {});
        let provided = FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: PopupTriggerMap::new(),
            floating_id: None,
            sync_only: false,
            nested: false,
            on_open_change: None,
        });

        let log = Rc::new(RefCell::new(Vec::new()));
        let store = use_synced_floating_root_context(UseSyncedFloatingRootContextOptions {
            popup_store: Rc::clone(&popup_store),
            treat_popup_as_floating_element: false,
            floating_root_context: Some(Rc::clone(&provided)),
            floating_id: None,
            nested: true,
            on_open_change: on_open_change_recorder(Rc::clone(&log)),
        });

        assert!(
            Rc::ptr_eq(&store, &provided),
            "the provided floating root context is used as-is"
        );
        assert!(store.context.nested(), "the context patch lands on it too");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::cell::RefCell;

    use reactive_graph::owner::Owner;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use wasm_bindgen::JsCast;
    use web_sys::Element;
    use crate::floating_ui::popup_store::create_initial_popup_store_state;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    type TestStore = Rc<ReactStore<PopupStoreState<()>, PopupStoreContext<()>>>;

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// Drains the executor so the tracked effects re-run after their inputs changed —
    /// the wasm analog of React's re-render flushing the layout effect (the
    /// `floating_delay_group` suite's `flush`).
    fn flush() {
        for _ in 0..8 {
            any_spawner::Executor::poll_local();
        }
    }

    /// The upstream suite's store fixture (`popupStoreUtils.test.tsx:34-55`).
    fn create_store(mutate: impl FnOnce(&mut PopupStoreState<()>)) -> TestStore {
        let trigger_elements = PopupTriggerMap::new();
        let mut state =
            create_initial_popup_store_state(&trigger_elements, None, false);
        mutate(&mut state);
        Rc::new(ReactStore::with_context(
            state,
            PopupStoreContext {
                trigger_elements,
                popup_ref: Rc::new(Cell::new(None)),
                on_open_change: None,
                on_open_change_complete: None,
            },
        ))
    }

    fn element() -> web_sys::HtmlElement {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .unchecked_into::<HtmlElement>()
    }

    fn popup_element_with_id(id: &str) -> web_sys::HtmlElement {
        let element = element();
        element.set_id(id);
        element
    }

    fn on_open_change_recorder(log: Rc<RefCell<Vec<bool>>>) -> OnOpenChangeFn {
        Rc::new(move |open: bool, _details| log.borrow_mut().push(open))
    }

    fn run_hook(
        popup_store: TestStore,
        options_tweaks: impl FnOnce(&mut UseSyncedFloatingRootContextOptions<(), ()>),
    ) -> Rc<FloatingRootStore> {
        let owner = Owner::new();
        owner.set();

        let mut options = UseSyncedFloatingRootContextOptions {
            popup_store: Rc::clone(&popup_store),
            treat_popup_as_floating_element: false,
            floating_root_context: None,
            floating_id: None,
            nested: false,
            on_open_change: on_open_change_recorder(Rc::new(RefCell::new(Vec::new()))),
        };
        options_tweaks(&mut options);
        let store = use_synced_floating_root_context(options);
        std::mem::forget(owner);
        store
    }

    // Mirrors `popupStoreUtils.test.tsx:765-772`: the hook syncs the floating id into
    // the popup store (the `useSyncedValue` half, `:80`) and into the floating root
    // context (the sync effect's `valuesToSync.floatingId`, `:83-96`), where the
    // `popupId` selector reads it.
    #[wasm_bindgen_test]
    fn syncs_the_floating_id_into_the_popup_store_for_trigger_ownership_selectors() {
        init_executor();

        let popup_store = create_store(|_| {});
        let seeded_context = Rc::clone(&popup_store.get_snapshot().floating_root_context);
        let store = run_hook(Rc::clone(&popup_store), |options| {
            options.floating_root_context = Some(seeded_context);
            options.floating_id = Some("popup-id".to_owned());
        });
        flush();

        assert_eq!(
            store.get_snapshot().floating_id,
            Some("popup-id".to_owned()),
            "the floating id syncs into the floating root context"
        );
        assert_eq!(
            popup_store.get_snapshot().floating_id,
            Some("popup-id".to_owned()),
            "the floating id syncs into the popup store"
        );
        assert_eq!(
            popup_store.select(selectors::popup_id),
            Some("popup-id".to_owned()),
            "the popupId selector reads the synced id"
        );
    }

    // Mirrors `popupStoreUtils.test.tsx:774-781`: an empty floating id is synced
    // as-is (the state carries `''`), and the `popupId` selector resolves it to
    // `undefined`.
    #[wasm_bindgen_test]
    fn omits_the_popup_id_when_the_floating_id_is_empty() {
        init_executor();

        let popup_store = create_store(|_| {});
        let seeded_context = Rc::clone(&popup_store.get_snapshot().floating_root_context);
        let _store = run_hook(Rc::clone(&popup_store), |options| {
            options.floating_root_context = Some(seeded_context);
            options.floating_id = Some(String::new());
        });
        flush();

        assert_eq!(
            popup_store.get_snapshot().floating_id,
            Some(String::new()),
            "the empty floating id is synced as-is"
        );
        assert_eq!(
            popup_store.select(selectors::popup_id),
            None,
            "the popupId selector omits it"
        );
    }

    // Port-owned pin for the internal store's `syncOnly: true` seed (`:73`): a
    // `setOpen` call on the synced store forwards to the hook's `onOpenChange`
    // without dispatching an `'openchange'` emission — the popup store owns the real
    // state in this mode (the `FloatingRootStore.ts:126-130` behavior).
    #[wasm_bindgen_test]
    fn the_internal_store_forwards_set_open_sync_only() {
        init_executor();

        let log = Rc::new(RefCell::new(Vec::new()));
        let popup_store = create_store(|_| {});
        let store = run_hook(popup_store, |options| {
            options.on_open_change = on_open_change_recorder(Rc::clone(&log));
        });

        let emissions = Rc::new(Cell::new(0u8));
        let emissions_handle = Rc::clone(&emissions);
        store.context.events.on(
            "openchange",
            Rc::new(move |_| emissions_handle.set(emissions_handle.get() + 1)),
        );
        let details = crate::floating_ui::types::RootOpenChangeEventDetails::new(
            crate::floating_ui::reasons::TRIGGER_PRESS,
            web_sys::Event::new("keydown").unwrap(),
            None,
            String::new(),
        );
        store.set_open(true, &details);

        assert_eq!(*log.borrow(), vec![true], "the callback received the change");
        assert_eq!(emissions.get(), 0, "no openchange emission in syncOnly mode");
    }

    // Port-owned pin for the sync effect (`:82-107`): the popup store's
    // open/activeTriggerElement/positionerElement mirror into the internal floating
    // store — open, the reference (and its DOM-reference arm), the floating element,
    // and the position reference (the mirror is active while the two are identical,
    // which the constructor seeding establishes).
    #[wasm_bindgen_test]
    fn mirrors_the_popup_store_state_into_the_internal_floating_store() {
        init_executor();

        let popup_store = create_store(|_| {});
        let store = run_hook(Rc::clone(&popup_store), |_| {});

        let trigger = element();
        let positioner = element();
        popup_store.update(
            |state, _| {
                state.open = true;
                state.mounted = true;
                state.active_trigger_element = Some(trigger.clone().unchecked_into());
                state.positioner_element = Some(positioner.clone());
                true
            },
        );
        flush();

        let state = store.get_snapshot();
        assert!(state.open, "open mirrors");
        assert_eq!(
            state.reference_element,
            Some(ReferenceType::Element(trigger.clone().unchecked_into())),
            "the active trigger mirrors as the reference"
        );
        assert_eq!(
            state.dom_reference_element,
            Some(trigger.clone().unchecked_into()),
            "the DOM reference mirrors from the real trigger"
        );
        assert_eq!(
            state.floating_element,
            Some(positioner.clone().unchecked_into()),
            "the positioner mirrors as the floating element by default"
        );
        assert_eq!(
            state.position_reference,
            state.reference_element,
            "the position reference keeps mirroring the reference while identical"
        );
    }

    // Port-owned pin for the `treatPopupAsFloatingElement` branch (`:53-55`): the
    // floating element source flips from the positioner to the popup element.
    #[wasm_bindgen_test]
    fn treat_popup_as_floating_element_selects_the_popup_element() {
        init_executor();

        let popup_store = create_store(|_| {});
        let store = run_hook(Rc::clone(&popup_store), |options| {
            options.treat_popup_as_floating_element = true;
        });

        let popup = element();
        let positioner = element();
        popup_store.update(
            |state, _| {
                state.popup_element = Some(popup.clone());
                state.positioner_element = Some(positioner.clone());
                true
            },
        );
        flush();

        assert_eq!(
            store.get_snapshot().floating_element,
            Some(popup.clone().unchecked_into()),
            "the popup element is the floating element under the flag"
        );
        assert_ne!(
            store.get_snapshot().floating_element,
            Some(positioner.clone().unchecked_into()),
            "the positioner is not"
        );
    }

    // Port-owned pin for the one-time mirror guard (`:102-104`): once
    // `positionReference` diverges from `referenceElement` (upstream
    // `setPositionReference`), later reference changes stop mirroring.
    #[wasm_bindgen_test]
    fn the_position_reference_mirror_stops_once_diverged() {
        init_executor();

        let popup_store = create_store(|_| {});
        let store = run_hook(Rc::clone(&popup_store), |_| {});

        let trigger = element();
        popup_store.update(
            |state, _| {
                state.open = true;
                state.mounted = true;
                state.active_trigger_element = Some(trigger.clone().unchecked_into());
                true
            },
        );
        flush();
        assert_eq!(
            store.get_snapshot().position_reference,
            store.get_snapshot().reference_element,
            "the mirror is active while the two are identical"
        );

        // Diverge: upstream `useFloating().setPositionReference(virtualElement)`
        // (`hooks/useFloating.ts:95-101`) — a different positioning anchor.
        let diverged = element();
        store.set_field(
            |state| &mut state.position_reference,
            Some(ReferenceType::Element(diverged.clone().unchecked_into())),
        );

        let second_trigger = element();
        popup_store.update(
            |state, _| {
                state.active_trigger_element = Some(second_trigger.clone().unchecked_into());
                true
            },
        );
        flush();

        assert_eq!(
            store.get_snapshot().reference_element,
            Some(ReferenceType::Element(second_trigger.unchecked_into())),
            "the reference keeps following the active trigger"
        );
        assert_eq!(
            store.get_snapshot().position_reference,
            Some(ReferenceType::Element(diverged.unchecked_into())),
            "the diverged position reference is left alone"
        );
    }
}
