//! Provisional port of `packages/react/src/utils/popups/store.ts` — the popup-store
//! state shape, initial-state factory, context shape, and selector table.
//!
//! The full `popups` unit belongs to the `infra: utils` TODO item (not yet ported);
//! `useSyncedFloatingRootContext` reads this exact state shape, and the floating-ui
//! unit's spec carries the dependency — "`PopupStoreState` for
//! `useSyncedFloatingRootContext`" (`specs/library/floating-ui-react/implementation.md`,
//! "src/utils/" section) — so the store.ts piece ports here first, following the
//! [`crate::floating_ui::popup_trigger_map`] and [`crate::floating_ui::focus_guard`]
//! provisional precedents. It moves behind a re-export of the popups port when
//! `infra: utils` lands. Only the members the floating-ui unit itself calls are
//! behavior-tested here; the rest port for shape parity.
//!
//! ## Rust adaptations
//!
//! - Upstream selectors are a named registry (`popupStoreSelectors`,
//!   `store.ts:168-206`) addressed by string key from `useState('open')`-style calls.
//!   The port follows the [`crate::floating_ui::floating_root_store`] convention: the
//!   registry becomes the free selector functions in [`selectors`], passed at call
//!   sites (see `crates/leptos-ui-utils/src/react_store.rs` module docs).
//! - Upstream's `type S = PopupStoreState<unknown>` fixes the selector-table's state
//!   type so the table is payload-agnostic. Rust free functions are generic over the
//!   payload parameter instead — same payload-agnosticism, structural rather than
//!   nominal.
//! - `PopupStoreState` derives `Clone` because the store hooks require it
//!   (`ReactStore::use_state`/`use_synced_value`'s `State: Clone` bound; the store
//!   snapshots through `Rc<State>`). Upstream's `Payload` type parameter is
//!   unconstrained — the bound is a Rust artifact, not a semantic narrowing.
//! - `popupRef` (`store.ts:127` — `React.RefObject<HTMLElement | null>`) ports to the
//!   crate's ref-object shape `Rc<Cell<Option<E>>>` (the `HTMLProps` ref-slot
//!   precedent, `crate::types` module docs).
//! - The trigger-props bags (`activeTriggerProps`/`inactiveTriggerProps`/`popupProps`,
//!   `store.ts:72-80`) use [`crate::types::HTMLProps`] — the port's attribute-prop
//!   vocabulary for upstream's `HTMLProps` (Leptos's native attribute system carries
//!   the attribute surface; see `specs/architecture.md`, "Prop / class / style
//!   merging").
//! - `PopupTriggerStoreKeys`/`PopupTriggerDataStore` (`store.ts:217`,
//!   `store.ts:223-226`) are TypeScript `Pick<…>` narrowings over the store's members,
//!   letting detached-handle views accept a subset of the store type. Rust call sites
//!   pass the concrete store handle and the compiler checks the members actually used,
//!   so the narrowings have no port-side shape to preserve.

use std::cell::Cell;
use std::rc::Rc;

use web_sys::{Element, HtmlElement};

use crate::floating_ui::floating_root_store::{FloatingRootStore, FloatingRootStoreOptions};
use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
use crate::floating_ui::types::TransitionStatus;
use crate::types::HTMLProps;

/// Port of `PopupStoreState<Payload>` (`store.ts:12-81`): the state common to all
/// popup stores. Each popup family extends this shape with its own fields (behavior.md,
/// "Public API surface" — the popups barrel exports it as the base of every family's
/// store).
#[derive(Clone)]
pub struct PopupStoreState<Payload> {
    /// Whether the popup is open (internal state) (`store.ts:16`).
    pub open: bool,
    /// Whether the popup is open (external prop) (`store.ts:20`).
    pub open_prop: Option<bool>,
    /// Whether the popup is mounted in the DOM — usually follows `open` but can differ
    /// during exit transitions (`store.ts:25`).
    pub mounted: bool,
    /// The current enter/exit transition status of the popup (`store.ts:29`).
    pub transition_status: Option<TransitionStatus>,
    /// The floating root context the popup store syncs into (`store.ts:31`;
    /// `FloatingRootContext` is `FloatingRootStore`, `types.ts:122`).
    pub floating_root_context: Rc<FloatingRootStore>,
    /// The ID of the floating element (`store.ts:32`).
    pub floating_id: Option<String>,
    /// Number of trigger elements currently registered for this popup
    /// (`store.ts:36`).
    pub trigger_count: u32,
    /// Whether to prevent unmounting the popup when closed — for JS animation
    /// libraries that control unmounting themselves (`store.ts:41`).
    pub prevent_unmounting_on_close: bool,
    /// Optional payload set by the trigger (`store.ts:46`).
    pub payload: Option<Payload>,
    /// ID of the currently active trigger (`store.ts:51`).
    pub active_trigger_id: Option<String>,
    /// The currently active trigger DOM element (`store.ts:55`).
    pub active_trigger_element: Option<Element>,
    /// ID of the trigger (external prop) (`store.ts:59`).
    pub trigger_id_prop: Option<String>,
    /// The popup DOM element (`store.ts:63`).
    pub popup_element: Option<HtmlElement>,
    /// The positioner DOM element (`store.ts:67`).
    pub positioner_element: Option<HtmlElement>,
    /// Props to spread onto the active trigger element (`store.ts:72`).
    pub active_trigger_props: HTMLProps,
    /// Props to spread onto inactive trigger elements (`store.ts:76`).
    pub inactive_trigger_props: HTMLProps,
    /// Props to spread onto the popup element (`store.ts:80`).
    pub popup_props: HTMLProps,
}

/// Port of `createInitialPopupStoreState` (`store.ts:83-117`): the initial state a
/// popup Root seeds its store with. The trigger map is shared — the same registry lands
/// in the seeded floating root context (`store.ts:93-103`, constructed `syncOnly: true`
/// with no `onOpenChange`) and in the caller's [`PopupStoreContext`], upstream one
/// object by reference (see [`PopupTriggerMap`]'s module docs on shared identity).
pub fn create_initial_popup_store_state<Payload>(
    trigger_elements: &PopupTriggerMap,
    floating_id: Option<String>,
    nested: bool,
) -> PopupStoreState<Payload> {
    PopupStoreState {
        open: false,
        open_prop: None,
        mounted: false,
        transition_status: None,
        floating_root_context: FloatingRootStore::new(FloatingRootStoreOptions {
            open: false,
            transition_status: None,
            reference_element: None,
            floating_element: None,
            trigger_elements: trigger_elements.clone(),
            floating_id: floating_id.clone(),
            sync_only: true,
            nested,
            on_open_change: None,
        }),
        floating_id,
        trigger_count: 0,
        prevent_unmounting_on_close: false,
        payload: None,
        active_trigger_id: None,
        active_trigger_element: None,
        trigger_id_prop: None,
        popup_element: None,
        positioner_element: None,
        active_trigger_props: HTMLProps::default(),
        inactive_trigger_props: HTMLProps::default(),
        popup_props: HTMLProps::default(),
    }
}

/// Port of `PopupStoreContext<ChangeEventDetails>` (`store.ts:119-136`): the store's
/// non-reactive context — the mutable state shared by a Root with its
/// Trigger/Popup/Positioner/Viewport parts (implementation spec, "Context
/// providers/consumers": each popup family creates its own React context for the
/// store; this shape is what crosses it).
pub struct PopupStoreContext<ChangeEventDetails> {
    /// Map of registered trigger elements (`store.ts:123`).
    pub trigger_elements: PopupTriggerMap,
    /// Reference to the popup element (`store.ts:127`).
    pub popup_ref: Rc<Cell<Option<HtmlElement>>>,
    /// Callback fired when the open state changes (`store.ts:131`).
    pub on_open_change: Option<Rc<dyn Fn(bool, &ChangeEventDetails)>>,
    /// Callback fired when the open state change animation completes
    /// (`store.ts:135`).
    pub on_open_change_complete: Option<Rc<dyn Fn(bool)>>,
}

/// Port of the `popupStoreSelectors` table (`store.ts:138-206`) as free functions —
/// the port's call-site equivalent of the string-keyed registry (see the module docs).
/// Each selector layers the external-prop override over the internal state
/// (`openProp ?? open`, `store.ts:142`; `triggerIdProp ?? activeTriggerId`,
/// `store.ts:140`; `popupElement?.id ?? floatingId`, `store.ts:144-147`).
pub mod selectors {
    use super::*;

    /// `open` (`store.ts:142`) — the controlled prop wins when present.
    pub fn open<P>(state: &PopupStoreState<P>) -> bool {
        state.open_prop.unwrap_or(state.open)
    }

    /// `mounted` (`store.ts:170`).
    pub fn mounted<P>(state: &PopupStoreState<P>) -> bool {
        state.mounted
    }

    /// `transitionStatus` (`store.ts:171`).
    pub fn transition_status<P>(state: &PopupStoreState<P>) -> Option<TransitionStatus> {
        state.transition_status
    }

    /// `floatingRootContext` (`store.ts:172`).
    pub fn floating_root_context<P>(state: &PopupStoreState<P>) -> Rc<FloatingRootStore> {
        Rc::clone(&state.floating_root_context)
    }

    /// `triggerCount` (`store.ts:173`).
    pub fn trigger_count<P>(state: &PopupStoreState<P>) -> u32 {
        state.trigger_count
    }

    /// `preventUnmountingOnClose` (`store.ts:174`).
    pub fn prevent_unmounting_on_close<P>(state: &PopupStoreState<P>) -> bool {
        state.prevent_unmounting_on_close
    }

    /// `payload` (`store.ts:175`).
    pub fn payload<P: Clone>(state: &PopupStoreState<P>) -> Option<P> {
        state.payload.clone()
    }

    /// `activeTriggerId` (`store.ts:140`, table entry `store.ts:177`) — the external
    /// trigger-id prop wins when present.
    pub fn active_trigger_id<P>(state: &PopupStoreState<P>) -> Option<String> {
        state
            .trigger_id_prop
            .clone()
            .or_else(|| state.active_trigger_id.clone())
    }

    /// `activeTriggerElement` (`store.ts:178`) — gated on `mounted` so an exiting
    /// popup stops reporting its trigger.
    pub fn active_trigger_element<P>(state: &PopupStoreState<P>) -> Option<Element> {
        if state.mounted {
            state.active_trigger_element.clone()
        } else {
            None
        }
    }

    /// `popupId` (`store.ts:144-147`, table entry `store.ts:179`): the popup
    /// element's own id, else the generated floating id; an empty candidate
    /// (`''` is not nullish, so an id-less popup element does *not* fall through to
    /// the floating id — `??` only skips `null`/`undefined`) resolves to `undefined`.
    pub fn popup_id<P>(state: &PopupStoreState<P>) -> Option<String> {
        let candidate = match &state.popup_element {
            Some(popup_element) => Some(popup_element.id()),
            None => state.floating_id.clone(),
        };
        candidate.filter(|id| !id.is_empty())
    }

    /// `triggerOwnsOpenPopup` (`store.ts:149-153`): the popup is open and the
    /// (coalesced) active trigger is the queried one.
    fn trigger_owns_open_popup<P>(state: &PopupStoreState<P>, trigger_id: Option<&str>) -> bool {
        trigger_id.is_some()
            && open(state)
            && active_trigger_id(state).as_deref() == trigger_id
    }

    /// `triggerOwnsOpenPopupOrIsOnlyTrigger` (`store.ts:155-166`): ownership, or the
    /// "only trigger" fallback — open with no active trigger and exactly one
    /// registered trigger.
    fn trigger_owns_open_popup_or_is_only_trigger<P>(
        state: &PopupStoreState<P>,
        trigger_id: Option<&str>,
    ) -> bool {
        if trigger_owns_open_popup(state, trigger_id) {
            return true;
        }
        trigger_id.is_some()
            && open(state)
            && active_trigger_id(state).is_none()
            && state.trigger_count == 1
    }

    /// `isTriggerActive` (`store.ts:183-184`): the trigger is the (coalesced) active
    /// one, regardless of open state.
    pub fn is_trigger_active<P>(state: &PopupStoreState<P>, trigger_id: Option<&str>) -> bool {
        trigger_id.is_some() && active_trigger_id(state).as_deref() == trigger_id
    }

    /// `isOpenedByTrigger` (`store.ts:188-189`): the popup is open and was activated
    /// by the trigger with the given ID.
    pub fn is_opened_by_trigger<P>(
        state: &PopupStoreState<P>,
        trigger_id: Option<&str>,
    ) -> bool {
        trigger_owns_open_popup(state, trigger_id)
    }

    /// `isMountedByTrigger` (`store.ts:193-194`): the popup is mounted and was
    /// activated by the trigger with the given ID.
    pub fn is_mounted_by_trigger<P>(
        state: &PopupStoreState<P>,
        trigger_id: Option<&str>,
    ) -> bool {
        trigger_id.is_some()
            && active_trigger_id(state).as_deref() == trigger_id
            && state.mounted
    }

    /// `triggerProps` (`store.ts:195-196`): the active or inactive props bag.
    pub fn trigger_props<P>(state: &PopupStoreState<P>, is_active: bool) -> HTMLProps {
        if is_active {
            state.active_trigger_props.clone()
        } else {
            state.inactive_trigger_props.clone()
        }
    }

    /// `triggerPopupId` (`store.ts:200-201`): the popup id for the trigger that
    /// currently owns the open popup (including the only-trigger fallback).
    pub fn trigger_popup_id<P>(
        state: &PopupStoreState<P>,
        trigger_id: Option<&str>,
    ) -> Option<String> {
        if trigger_owns_open_popup_or_is_only_trigger(state, trigger_id) {
            popup_id(state)
        } else {
            None
        }
    }

    /// `popupProps` (`store.ts:202`).
    pub fn popup_props<P>(state: &PopupStoreState<P>) -> HTMLProps {
        state.popup_props.clone()
    }

    /// `popupElement` (`store.ts:204`).
    pub fn popup_element<P>(state: &PopupStoreState<P>) -> Option<HtmlElement> {
        state.popup_element.clone()
    }

    /// `positionerElement` (`store.ts:205`).
    pub fn positioner_element<P>(state: &PopupStoreState<P>) -> Option<HtmlElement> {
        state.positioner_element.clone()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use std::cell::RefCell;

    fn initial_state(nested: bool) -> PopupStoreState<()> {
        create_initial_popup_store_state(&PopupTriggerMap::new(), Some("fid".to_owned()), nested)
    }

    // Pins the initial-state seeds (`store.ts:88-116`): closed, unmounted, no
    // transition, no triggers, no payload, no active trigger, absent external props,
    // and the `floatingId`/`nested` passthrough into both the state and the seeded
    // floating root context.
    #[test]
    fn create_initial_popup_store_state_seeds_the_defaults() {
        let state: PopupStoreState<()> = initial_state(true);

        assert!(!state.open);
        assert_eq!(state.open_prop, None);
        assert!(!state.mounted);
        assert_eq!(state.transition_status, None);
        assert_eq!(state.trigger_count, 0);
        assert!(!state.prevent_unmounting_on_close);
        assert_eq!(state.payload, None);
        assert_eq!(state.active_trigger_id, None);
        assert_eq!(state.active_trigger_element, None);
        assert_eq!(state.trigger_id_prop, None);
        assert_eq!(state.popup_element, None);
        assert_eq!(state.positioner_element, None);
        assert_eq!(state.floating_id, Some("fid".to_owned()));
        assert!(
            state.floating_root_context.context.nested(),
            "the nested option passes through to the seeded floating root context"
        );
    }

    // Pins the seeded floating root context's construction options
    // (`store.ts:93-103`): `syncOnly: true` with `onOpenChange: undefined` — nothing
    // is registered to forward to, and no `'openchange'` listener exists. (The
    // forwarding/emission halves are pinned in the wasm suites, which can construct
    // the event details.)
    #[test]
    fn the_seeded_floating_root_context_is_a_listenerless_sync_only_store() {
        let state: PopupStoreState<()> = initial_state(false);
        let floating_root_context = Rc::clone(&state.floating_root_context);

        assert!(
            floating_root_context.context.on_open_change().is_none(),
            "the seeded context carries no onOpenChange"
        );
    }

    // Pins the external-prop override layering of the coalescing selectors
    // (`store.ts:140,142`): `openProp ?? open` and `triggerIdProp ?? activeTriggerId`,
    // plus the open-state ownership selectors built on them (`store.ts:149-166`,
    // `store.ts:183-201`).
    #[test]
    fn the_coalescing_selectors_layer_external_props_over_internal_state() {
        let mut state: PopupStoreState<()> = initial_state(false);

        // Uncontrolled: the internal state decides.
        state.open = true;
        assert!(selectors::open(&state), "the internal open is the fallback");

        // Controlled: the prop wins in both directions (`store.ts:142`).
        state.open_prop = Some(false);
        assert!(!selectors::open(&state), "openProp: false wins over open: true");
        state.open_prop = Some(true);
        assert!(selectors::open(&state), "openProp: true wins over open: false");

        // The trigger-id override (`store.ts:140`).
        state.active_trigger_id = Some("internal".to_owned());
        assert_eq!(
            selectors::active_trigger_id(&state),
            Some("internal".to_owned()),
            "the internal active trigger is the fallback"
        );
        state.trigger_id_prop = Some("external".to_owned());
        assert_eq!(
            selectors::active_trigger_id(&state),
            Some("external".to_owned()),
            "triggerIdProp wins over the internal active trigger"
        );

        // Ownership selectors (`store.ts:149-166`, `store.ts:183-201`) — the store
        // is open with the external trigger active; `triggerCount` is written
        // directly (registration is the full popups unit's concern).
        state.trigger_count = 1;

        assert!(
            selectors::is_opened_by_trigger(&state, Some("external")),
            "the owning trigger is reported as having opened the popup"
        );
        assert!(
            !selectors::is_opened_by_trigger(&state, Some("other")),
            "a non-active trigger is not reported"
        );
        assert!(
            selectors::is_trigger_active(&state, Some("external")),
            "isTriggerActive does not require the popup to be open"
        );
        state.open_prop = Some(false);
        assert!(
            !selectors::is_opened_by_trigger(&state, Some("external")),
            "a closed popup is not owned by any trigger"
        );

        // The only-trigger fallback (`store.ts:160-165`): open, no active trigger,
        // exactly one registration.
        state.open_prop = Some(true);
        state.trigger_id_prop = None;
        state.active_trigger_id = None;
        assert!(
            selectors::trigger_popup_id(&state, Some("trigger-id")).is_some(),
            "the only registered trigger falls back to owning the open popup"
        );
        state.trigger_count = 2;
        assert!(
            selectors::trigger_popup_id(&state, Some("trigger-id")).is_none(),
            "two registered triggers disable the fallback"
        );
    }

    // Pins the props-bag selector (`store.ts:195-196`): the active/inactive bags are
    // handed out by the `isActive` flag.
    #[test]
    fn trigger_props_selects_between_the_active_and_inactive_bags() {
        let active_slot = Rc::new(Cell::new(None));
        let inactive_slot = Rc::new(Cell::new(None));

        let mut state: PopupStoreState<()> = initial_state(false);
        state.active_trigger_props = HTMLProps {
            node_ref: Some(Rc::clone(&active_slot)),
        };
        state.inactive_trigger_props = HTMLProps {
            node_ref: Some(Rc::clone(&inactive_slot)),
        };

        assert!(
            Rc::ptr_eq(
                selectors::trigger_props(&state, true).node_ref.as_ref().unwrap(),
                &active_slot
            ),
            "an active trigger gets the active bag"
        );
        assert!(
            Rc::ptr_eq(
                selectors::trigger_props(&state, false)
                    .node_ref
                    .as_ref()
                    .unwrap(),
                &inactive_slot
            ),
            "an inactive trigger gets the inactive bag"
        );
    }

    // Pins the shape of the popup-store context (`store.ts:119-136`): the trigger
    // registry, the popup ref slot, and the two open-change callbacks are all
    // present and defaulted.
    #[test]
    fn the_popup_store_context_carries_the_shared_members() {
        let trigger_elements = PopupTriggerMap::new();
        let context: PopupStoreContext<String> = PopupStoreContext {
            trigger_elements: trigger_elements.clone(),
            popup_ref: Rc::new(Cell::new(None)),
            on_open_change: None,
            on_open_change_complete: None,
        };

        assert_eq!(context.trigger_elements.size(), 0);
        assert!(context.popup_ref.take().is_none());
        assert!(context.on_open_change.is_none());
        assert!(context.on_open_change_complete.is_none());

        let seen: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let seen_handle = Rc::clone(&seen);
        let context_with_callbacks: PopupStoreContext<String> = PopupStoreContext {
            on_open_change: Some(Rc::new(move |open: bool, _details: &String| {
                seen_handle.borrow_mut().push(open);
            })),
            on_open_change_complete: Some(Rc::new(|_open: bool| {})),
            ..context
        };
        (context_with_callbacks.on_open_change.as_ref().unwrap())(true, &"details".to_owned());
        assert_eq!(*seen.borrow(), vec![true], "the callback is invocable");
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// A popup element carrying an explicit id — the upstream fixture's
    /// `popupElement` (`popupStoreUtils.test.tsx:785-786`).
    fn popup_element_with_id(id: &str) -> HtmlElement {
        let element = document().create_element("div").unwrap();
        element.set_id(id);
        element.unchecked_into::<HtmlElement>()
    }

    // Mirrors `popupStoreUtils.test.tsx:783-796`: an explicit popup element id wins
    // over the generated floating id (`store.ts:144-147` — `popupElement?.id ??
    // floatingId`).
    #[wasm_bindgen_test]
    fn popup_id_prefers_an_explicit_popup_element_id_over_the_generated_floating_id() {
        let mut state: PopupStoreState<()> = create_initial_popup_store_state(
            &PopupTriggerMap::new(),
            Some("generated-popup-id".to_owned()),
            false,
        );
        state.open = true;
        state.active_trigger_id = Some("trigger".to_owned());
        state.popup_element = Some(popup_element_with_id("explicit-popup-id"));

        assert_eq!(
            selectors::popup_id(&state),
            Some("explicit-popup-id".to_owned()),
            "the popup element's own id wins"
        );
    }

    // Mirrors `popupStoreUtils.test.tsx:774-781`'s selector half: an empty floating id
    // yields `undefined` (`store.ts:146` — `popupId || undefined`).
    #[wasm_bindgen_test]
    fn popup_id_omits_an_empty_floating_id() {
        let state: PopupStoreState<()> =
            create_initial_popup_store_state(&PopupTriggerMap::new(), Some(String::new()), false);

        assert_eq!(
            selectors::popup_id(&state),
            None,
            "an empty floating id is not a popup id"
        );
    }

    // Pins the `??`-vs-`||` subtlety of `store.ts:144-147`: a popup element whose id
    // is the empty string does NOT fall through to the floating id (`''` is not
    // nullish, so `??` keeps it and `||` then discards it).
    #[wasm_bindgen_test]
    fn an_idless_popup_element_does_not_fall_through_to_the_floating_id() {
        let mut state: PopupStoreState<()> = create_initial_popup_store_state(
            &PopupTriggerMap::new(),
            Some("generated-popup-id".to_owned()),
            false,
        );
        state.popup_element = Some(popup_element_with_id(""));

        assert_eq!(
            selectors::popup_id(&state),
            None,
            "an empty popup element id short-circuits the fallback"
        );
    }

    // Mirrors `popupStoreUtils.test.tsx:765-772`'s selector half: the floating id is
    // the fallback popup id while no popup element is set.
    #[wasm_bindgen_test]
    fn popup_id_falls_back_to_the_floating_id() {
        let state: PopupStoreState<()> = create_initial_popup_store_state(
            &PopupTriggerMap::new(),
            Some("popup-id".to_owned()),
            false,
        );

        assert_eq!(
            selectors::popup_id(&state),
            Some("popup-id".to_owned()),
            "the generated floating id is the fallback"
        );
    }

    // Pins the mounted gating of `activeTriggerElement` (`store.ts:178`): an exiting
    // (unmounted) popup stops reporting its active trigger element.
    #[wasm_bindgen_test]
    fn active_trigger_element_is_gated_on_mounted() {
        let element = document().create_element("button").unwrap();
        let mut state: PopupStoreState<()> =
            create_initial_popup_store_state(&PopupTriggerMap::new(), None, false);
        state.active_trigger_element = Some(element.clone());

        state.mounted = true;
        assert_eq!(
            selectors::active_trigger_element(&state),
            Some(element.clone()),
            "a mounted popup reports its active trigger element"
        );

        state.mounted = false;
        assert_eq!(
            selectors::active_trigger_element(&state),
            None,
            "an unmounted popup reports no active trigger element"
        );
    }
}
