//! Port of `packages/react/src/floating-ui-react/hooks/useHoverInteractionSharedState.ts`
//! — the mutable hover-interaction instance the split hover hooks share so the
//! reference-side and floating-side handlers agree on pointer type, rest state, and the
//! safePolygon handler without a shared React provider
//! (`specs/library/floating-ui-react/implementation.md`, "Notable per-hook state
//! machines": "Hover (3 implementations, one shared instance)").
//!
//! ## Rust adaptations
//!
//! - The `HoverInteraction` class's mutable fields port to `Cell`/`RefCell` members on
//!   a `Rc`-shared struct: upstream shares the instance by reference through
//!   `dataRef.current.hoverInteractionState`
//!   (`useHoverInteractionSharedState.ts:114-131`), and the port's
//!   [`crate::floating_ui::types::ContextData::hover_interaction_state`] field carries
//!   the same `Rc` (the dataRef bag grew this key when this unit ported — the
//!   per-key-field convention of the `ContextData` port).
//! - `pointerEventsMutationOwnerByScopeElement` (`:62-65`, a `WeakMap` keyed by the
//!   scope element) ports to a thread-local registry `Vec` compared by element
//!   identity (JS `===`). The `WeakMap` weakens both sides so abandoned owners are
//!   collectable; the registry holds strong references until the owner clears its
//!   mutation (every apply/clear path and the takeover branch removes entries) — the
//!   only residue is an owner that unmounts without clearing, which retains the small
//!   state struct. `Rc` has no weak counterpart, so the strong registry is the
//!   available approximation.
//! - `useHoverInteractionSharedState(store)` (`:118-131`): the `useRefWithInit` seed +
//!   `dataRef` back-fill + `useOnMount(disposeEffect)` sequence ports one-to-one over
//!   the ported hooks ([`leptos_ui_utils::use_ref_with_init`],
//!   [`leptos_ui_utils::use_on_mount`]).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::traits::GetValue;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;
use web_sys::{CssStyleDeclaration, Element};

use leptos_ui_utils::use_on_mount;
use leptos_ui_utils::use_ref_with_init;
use leptos_ui_utils::use_timeout::Timeout;

use crate::floating_ui::element_props::FloatingContextSource;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::safe_polygon::SafePolygonOptions;
use crate::floating_ui::use_hover_shared::MouseMoveHandler;

/// Port of the `HoverInteraction` class (`useHoverInteractionSharedState.ts:11-52`):
/// the per-popup mutable hover state shared between the reference-side and
/// floating-side hover hooks through the store's `dataRef`.
pub struct HoverInteraction {
    /// `pointerType` (`:12`) — the last observed pointer type.
    pub pointer_type: RefCell<Option<String>>,
    /// `interactedInside` (`:13`) — whether the pointer interacted with an interactive
    /// element inside the popup (the click-like-open-event ingredient).
    pub interacted_inside: Cell<bool>,
    /// `handler` (`:14`) — the active safePolygon `mousemove` handler, when one is
    /// running.
    pub handler: RefCell<Option<MouseMoveHandler>>,
    /// `blockMouseMove` (`:15`) — `true` until the pointer enters the reference,
    /// suppressing rest-engine opens while closing.
    pub block_mouse_move: Cell<bool>,
    /// `performedPointerEventsMutation` (`:16`) — whether this instance currently owns
    /// a pointer-events mutation.
    pub performed_pointer_events_mutation: Cell<bool>,
    /// `pointerEventsScopeElement` (`:17`) — the element carrying
    /// `pointer-events: none`.
    pub pointer_events_scope_element: RefCell<Option<Element>>,
    /// `pointerEventsReferenceElement` (`:18`).
    pub pointer_events_reference_element: RefCell<Option<Element>>,
    /// `pointerEventsFloatingElement` (`:19`).
    pub pointer_events_floating_element: RefCell<Option<Element>>,
    /// `restTimeoutPending` (`:20`).
    pub rest_timeout_pending: Cell<bool>,
    /// `openChangeTimeout` (`:21`).
    pub open_change_timeout: Timeout,
    /// `restTimeout` (`:22`).
    pub rest_timeout: Timeout,
    /// `handleCloseOptions` (`:23`) — the active `handleClose`'s `__options`, synced by
    /// the active trigger's hook call (`useHoverReferenceInteraction.ts:158-161`) and
    /// read by the floating-side pointer-events effect
    /// (`useHoverFloatingInteraction.ts:101-107`).
    pub handle_close_options: RefCell<Option<SafePolygonOptions>>,
}

impl HoverInteraction {
    /// Port of the constructor (`:25-38`): every field starts at its falsy value
    /// except `blockMouseMove` (`true`) and the two timers.
    pub fn create() -> Rc<Self> {
        Rc::new(Self {
            pointer_type: RefCell::new(None),
            interacted_inside: Cell::new(false),
            handler: RefCell::new(None),
            block_mouse_move: Cell::new(true),
            performed_pointer_events_mutation: Cell::new(false),
            pointer_events_scope_element: RefCell::new(None),
            pointer_events_reference_element: RefCell::new(None),
            pointer_events_floating_element: RefCell::new(None),
            rest_timeout_pending: Cell::new(false),
            open_change_timeout: Timeout::create(),
            rest_timeout: Timeout::create(),
            handle_close_options: RefCell::new(None),
        })
    }

    /// Port of `dispose` (`:44-47`): clears both timers. Registered as the
    /// mount-cleanup by [`use_hover_interaction_shared_state`].
    pub fn dispose(&self) {
        self.open_change_timeout.clear();
        self.rest_timeout.clear();
    }
}

thread_local! {
    /// `pointerEventsMutationOwnerByScopeElement` (`useHoverInteractionSharedState.ts:
    /// 62-65`) — which instance owns the pointer-events mutation on each scope element.
    /// See the module docs for the `WeakMap` → registry adaptation.
    static POINTER_EVENTS_MUTATION_OWNER_BY_SCOPE_ELEMENT:
        RefCell<Vec<(Element, Rc<HoverInteraction>)>> = const { RefCell::new(Vec::new()) };
}

/// The `element.style` accessor (`HTMLElement`/`SVGElement`'s
/// `ElementCSSInlineStyle`), read through `Reflect` so the port does not depend on the
/// per-element web-sys bindings.
fn inline_style(element: &Element) -> Option<CssStyleDeclaration> {
    let style = js_sys::Reflect::get(element.as_ref(), &JsValue::from_str("style")).ok()?;
    if style.is_undefined() || style.is_null() {
        return None;
    }
    Some(style.unchecked_into::<CssStyleDeclaration>())
}

fn set_pointer_events(element: Option<&Element>, value: &str) {
    if let Some(style) = element.and_then(inline_style) {
        style.set_property("pointer-events", value).unwrap();
    }
}

fn remove_pointer_events(element: &Element) {
    if let Some(style) = inline_style(element) {
        style.remove_property("pointer-events").unwrap();
    }
}

/// Port of `clearSafePolygonPointerEventsMutation`
/// (`useHoverInteractionSharedState.ts:67-85`): undoes the instance's pointer-events
/// mutation — removing the inline properties only when the instance still owns the
/// registry entry (a takeover may have reassigned them) — and resets the tracked
/// elements either way.
pub fn clear_safe_polygon_pointer_events_mutation(instance: &Rc<HoverInteraction>) {
    if !instance.performed_pointer_events_mutation.get() {
        return;
    }

    let scope_element = instance.pointer_events_scope_element.borrow().clone();
    let reference_element = instance.pointer_events_reference_element.borrow().clone();
    let floating_element = instance.pointer_events_floating_element.borrow().clone();

    let owns_entry = scope_element
        .as_ref()
        .map(|scope| {
            POINTER_EVENTS_MUTATION_OWNER_BY_SCOPE_ELEMENT.with_borrow_mut(|registry| {
                if let Some(position) = registry.iter().position(|(scope_candidate, owner)| {
                    scope_candidate == scope && Rc::ptr_eq(owner, instance)
                }) {
                    registry.remove(position);
                    true
                } else {
                    false
                }
            })
        })
        .unwrap_or(false);

    if owns_entry {
        for element in [&scope_element, &reference_element, &floating_element]
            .into_iter()
            .flatten()
        {
            remove_pointer_events(&element);
        }
    }

    instance.performed_pointer_events_mutation.set(false);
    *instance.pointer_events_scope_element.borrow_mut() = None;
    *instance.pointer_events_reference_element.borrow_mut() = None;
    *instance.pointer_events_floating_element.borrow_mut() = None;
}

/// Port of `applySafePolygonPointerEventsMutation`
/// (`useHoverInteractionSharedState.ts:87-112`): claims the pointer-events mutation on
/// `scope_element` for `instance` — clearing any previous owner first (a second
/// claimant evicts the first, `:97-100`) — and sets the scope to `pointer-events:
/// none` with the reference and floating elements back at `auto`.
pub fn apply_safe_polygon_pointer_events_mutation(
    instance: &Rc<HoverInteraction>,
    scope_element: &Element,
    reference_element: &Element,
    floating_element: &Element,
) {
    let existing_owner = POINTER_EVENTS_MUTATION_OWNER_BY_SCOPE_ELEMENT.with_borrow(|registry| {
        registry
            .iter()
            .find(|(scope, _)| scope == scope_element)
            .map(|(_, owner)| Rc::clone(owner))
    });
    if let Some(existing_owner) = existing_owner {
        if !Rc::ptr_eq(&existing_owner, instance) {
            clear_safe_polygon_pointer_events_mutation(&existing_owner);
        }
    }

    clear_safe_polygon_pointer_events_mutation(instance);
    instance.performed_pointer_events_mutation.set(true);
    *instance.pointer_events_scope_element.borrow_mut() = Some(scope_element.clone());
    *instance.pointer_events_reference_element.borrow_mut() = Some(reference_element.clone());
    *instance.pointer_events_floating_element.borrow_mut() = Some(floating_element.clone());
    POINTER_EVENTS_MUTATION_OWNER_BY_SCOPE_ELEMENT
        .with_borrow_mut(|registry| registry.push((scope_element.clone(), Rc::clone(instance))));

    set_pointer_events(Some(scope_element), "none");
    set_pointer_events(Some(reference_element), "auto");
    set_pointer_events(Some(floating_element), "auto");
}

/// Port of `useHoverInteractionSharedState(store)`
/// (`useHoverInteractionSharedState.ts:118-131`): returns the shared
/// [`HoverInteraction`] for the popup's store — the instance already stashed on
/// `dataRef.current.hoverInteractionState` if any hook created one, else a fresh one
/// that is stashed — and registers the timer disposal as the mount cleanup. Must be
/// called inside a reactive owner.
pub fn use_hover_interaction_shared_state(
    context: impl Into<FloatingContextSource>,
) -> Rc<HoverInteraction> {
    let store: Rc<FloatingRootStore> = context.into().root_store();
    let data_ref = Rc::clone(&store.context.data_ref);

    let instance = use_ref_with_init(|| {
        data_ref
            .borrow()
            .hover_interaction_state
            .clone()
            .unwrap_or_else(HoverInteraction::create)
    });
    let instance = instance.get_value();

    if data_ref.borrow().hover_interaction_state.is_none() {
        data_ref.borrow_mut().hover_interaction_state = Some(Rc::clone(&instance));
    }

    // `useOnMount(data.hoverInteractionState.disposeEffect)` (`:128`) — the cleanup
    // `disposeEffect` returns is `dispose` itself (`:49-51`).
    use_on_mount({
        let instance = Rc::clone(&instance);
        move || {
            let instance = Rc::clone(&instance);
            move || instance.dispose()
        }
    });

    data_ref
        .borrow()
        .hover_interaction_state
        .clone()
        .expect("the instance was stashed above")
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use wasm_bindgen::closure::Closure;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    fn store() -> Rc<FloatingRootStore> {
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

    fn div() -> Element {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
    }

    /// The `pointer-events` inline property of an element — `None` when no inline value
    /// is set (CSSOM's `removeProperty` leaves an empty string).
    fn pointer_events(element: &Element) -> Option<String> {
        inline_style(element)
            .map(|style| style.get_property_value("pointer-events").unwrap())
            .filter(|value| !value.is_empty())
    }

    // Pins the shared-instance mechanics (`useHoverInteractionSharedState.ts:118-131`):
    // two hook calls against the same store return the SAME instance (the first one
    // stashed on dataRef), while another store gets its own.
    #[wasm_bindgen_test]
    fn hook_calls_share_one_instance_per_store() {
        init_executor();
        reactive_graph::owner::Owner::new().with(|| {
            let store = store();

            let first = use_hover_interaction_shared_state(Rc::clone(&store));
            assert!(
                data_ref_holds(&store, &first),
                "the first call stashes its instance on dataRef"
            );

            let second = use_hover_interaction_shared_state(Rc::clone(&store));
            assert!(
                Rc::ptr_eq(&first, &second),
                "the second call reuses the stashed instance"
            );

            let other = {
                let other_store = FloatingRootStore::new(FloatingRootStoreOptions {
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
                use_hover_interaction_shared_state(other_store)
            };
            assert!(
                !Rc::ptr_eq(&first, &other),
                "a different store gets a different instance"
            );
        });
    }

    fn data_ref_holds(store: &Rc<FloatingRootStore>, instance: &Rc<HoverInteraction>) -> bool {
        store
            .context
            .data_ref
            .borrow()
            .hover_interaction_state
            .as_ref()
            .map(|stashed| Rc::ptr_eq(stashed, instance))
            .unwrap_or(false)
    }

    // Pins the mount-cleanup (`:44-51,128`): owner disposal runs `dispose`, clearing
    // both timers (verified through a started timer). `poll_local` runs the deferred
    // mount callback so its cleanup registers before the disposal.
    #[wasm_bindgen_test]
    fn owner_disposal_disposes_the_timers() {
        init_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        {
            let store = store();
            let instance = use_hover_interaction_shared_state(Rc::clone(&store));
            instance.rest_timeout.start(50, || {});
            assert!(instance.rest_timeout.is_started(), "the timer runs");

            any_spawner::Executor::poll_local();
            owner.cleanup();
            assert!(
                !instance.rest_timeout.is_started(),
                "owner disposal cleared the timer"
            );
        }
    }

    // Pins `applySafePolygonPointerEventsMutation` (`:87-112`): the scope carries
    // `pointer-events: none`, the reference and floating elements carry `auto`, and the
    // mutation is registered as owned.
    #[wasm_bindgen_test]
    fn apply_sets_the_pointer_events_mutation() {
        let instance = HoverInteraction::create();
        let scope = div();
        let reference = div();
        let floating = div();

        apply_safe_polygon_pointer_events_mutation(&instance, &scope, &reference, &floating);

        assert_eq!(pointer_events(&scope).as_deref(), Some("none"));
        assert_eq!(pointer_events(&reference).as_deref(), Some("auto"));
        assert_eq!(pointer_events(&floating).as_deref(), Some("auto"));
        assert!(instance.performed_pointer_events_mutation.get());
    }

    // Pins `clearSafePolygonPointerEventsMutation` (`:67-85`): the properties are
    // removed and the tracked elements reset.
    #[wasm_bindgen_test]
    fn clear_undoes_the_mutation() {
        let instance = HoverInteraction::create();
        let scope = div();
        let reference = div();
        let floating = div();

        apply_safe_polygon_pointer_events_mutation(&instance, &scope, &reference, &floating);
        clear_safe_polygon_pointer_events_mutation(&instance);

        assert_eq!(pointer_events(&scope), None);
        assert_eq!(pointer_events(&reference), None);
        assert_eq!(pointer_events(&floating), None);
        assert!(!instance.performed_pointer_events_mutation.get());
        assert!(instance.pointer_events_scope_element.borrow().is_none());
        assert!(instance.pointer_events_reference_element.borrow().is_none());
    }

    // Pins the single-writer takeover (`:97-100`): a second instance claiming the same
    // scope clears the first instance's mutation, and the first instance's own clear
    // then removes nothing (the properties belong to the second claimant).
    #[wasm_bindgen_test]
    fn a_second_claimant_takes_over_the_scope() {
        let first = HoverInteraction::create();
        let second = HoverInteraction::create();
        let scope = div();
        let reference = div();
        let floating = div();

        apply_safe_polygon_pointer_events_mutation(&first, &scope, &reference, &floating);
        apply_safe_polygon_pointer_events_mutation(&second, &scope, &reference, &floating);

        assert!(
            !first.performed_pointer_events_mutation.get(),
            "the takeover cleared the first claimant's state"
        );
        assert_eq!(pointer_events(&scope).as_deref(), Some("none"));

        // The second claimant clears its own mutation.
        clear_safe_polygon_pointer_events_mutation(&second);
        assert_eq!(pointer_events(&scope), None);

        // The first claimant's late clear is a no-op (its entry was already gone and
        // its fields were reset by the takeover).
        clear_safe_polygon_pointer_events_mutation(&first);
        assert_eq!(pointer_events(&scope), None, "no double-clear residue");
    }

    fn sleep(ms: i32) -> wasm_bindgen_futures::js_sys::Promise {
        wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    }
}
