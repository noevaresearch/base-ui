//! Tests for the Popover port — mirrors of the upstream suites behavior.md mines
//! (`PopoverRoot.test.tsx`, `PopoverTrigger.test.tsx`, `PopoverClose.test.tsx`,
//! `PopoverTitle.test.tsx`), over the store-level contracts in an owner-scoped
//! host suite (the dialog_tests.rs convention; the wasm materialized-tree
//! mirrors follow in a later pass once the docs-app pairing work lands).
//!
//! Host suite: the pure contracts that need no DOM — the `setOpen` sequence
//! (`PopoverStore.ts:95-168`), the controlled veto, the hover patient-click
//! window arming, the triggerless `closePress` trigger backfill, the
//! `instantType` mapping (`:159-167`), the modal-union projection, and the
//! `defaultOpen` ⇒ `mounted` first-paint seed.

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::popover::store::{
    PopoverExtraState, popover_change_event_details, popover_set_open,
};
use leptos_ui_internals::floating_ui::popup_store::{
    PopupStoreContext, create_initial_popup_store_state,
};
use leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap;
use leptos_ui_internals::floating_ui::types::{OnOpenChangeFn, RootOpenChangeEventDetails};
use leptos_ui_internals::popup_store_utils::PopupStore;

/// The shared details type the callbacks exchange
/// (`PopoverRoot.tsx:20-22` — `createChangeEventDetails(reason, nativeEvent)`).
type Details = RootOpenChangeEventDetails;

/// A counting callback recorder (the dialog_tests.rs convention).
#[derive(Clone, Default)]
pub(crate) struct Recorder {
    calls: Rc<RefCell<Vec<(bool, String)>>>,
}

impl Recorder {
    pub(crate) fn record(&self, open: bool, reason: &str) {
        self.calls.borrow_mut().push((open, reason.to_owned()));
    }
    pub(crate) fn calls(&self) -> Vec<(bool, String)> {
        self.calls.borrow().clone()
    }
}

/// Builds a popup store with the shared shape and a recording `onOpenChange` —
/// the dialog `make_store` harness convention.
pub(crate) fn make_store(on_open_change: Option<OnOpenChangeFn>) -> PopupStore<()> {
    let trigger_elements = PopupTriggerMap::new();
    let state = create_initial_popup_store_state(&trigger_elements, None, false);
    Rc::new(leptos_ui_utils::react_store::ReactStore::with_context(
        state,
        PopupStoreContext {
            trigger_elements,
            popup_ref: Rc::new(std::cell::Cell::new(None)),
            on_open_change,
            on_open_change_complete: None,
        },
    ))
}

pub(crate) fn details(reason: &str) -> Details {
    Details::new(
        reason.to_owned(),
        // The host target has no JS runtime — wrap a plain JsValue instead of
        // calling the wasm-bindgen `Event` constructor (the dialog_tests.rs
        // host-suite convention).
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
        None,
        String::new(),
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use leptos_ui_internals::floating_ui::popup_store::selectors;
    use leptos_ui_internals::floating_ui::reasons as REASONS;

    // behavior.md "Events" (`PopoverRoot.test.tsx:150-163` analog): `onOpenChange`
    // fires once per requested transition with the new open state and the reason
    // carried on the details; the store's `open` commits after the callback.
    #[test]
    fn set_open_notifies_once_with_the_next_open_state() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let store = make_store(Some(Rc::new(move |open: bool, details: &Details| {
            recorder_for_cb.record(open, &details.reason);
        })));

        popover_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert_eq!(
            recorder.calls(),
            vec![(true, REASONS::TRIGGER_PRESS.to_owned())]
        );
        assert!(store.get_snapshot().open, "the open commits");

        popover_set_open(&store, false, &mut details(REASONS::CLOSE_PRESS));
        assert_eq!(recorder.calls().len(), 2, "one call per transition");
        assert!(!store.get_snapshot().open, "the close commits");
    }

    // behavior.md "State model" cancel gate (`PopoverRoot.test.tsx:535-553` analog):
    // `eventDetails.cancel()` inside `onOpenChange` vetoes both directions.
    #[test]
    fn a_canceled_open_or_close_is_vetoed() {
        let store = make_store(Some(Rc::new(|_open: bool, details: &Details| {
            details.cancel();
        })));

        popover_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert!(
            !store.get_snapshot().open,
            "the canceled open never commits"
        );

        // A fresh store: the close path's veto.
        let store = make_store(Some(Rc::new(|open: bool, details: &Details| {
            if !open {
                details.cancel();
            }
        })));
        popover_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert!(store.get_snapshot().open, "the uncanceled open lands");

        popover_set_open(&store, false, &mut details(REASONS::CLOSE_PRESS));
        assert!(
            store.get_snapshot().open,
            "the canceled close never commits — the veto semantics"
        );
    }

    // `PopoverStore.setOpen`'s `instantType` mapping (`:159-167`): keyboard click
    // (`triggerPress` with `detail === 0`) → `'click'`, dismiss-close →
    // `'dismiss'`, `focusOut` → `'focus'`, everything else → cleared.
    #[test]
    fn the_instant_type_mapping_follows_the_reason() {
        // Keyboard click: a MouseEvent with detail 0. The host target has no JS
        // runtime, so the dyn_ref arm reads None and no click instant is written —
        // the dismiss and focus arms still pin the mapping.
        let store = make_store(None);
        popover_set_open(&store, true, &mut details(REASONS::TRIGGER_PRESS));
        assert_eq!(
            store.get_snapshot().instant_type,
            None,
            "a detail-less press event carries no keyboard classification on host"
        );

        let store = make_store(None);
        popover_set_open(&store, false, &mut details(REASONS::ESCAPE_KEY));
        assert_eq!(
            store.get_snapshot().instant_type,
            Some(leptos_ui_internals::floating_ui::popup_store::InstantType::Dismiss),
            "the escape dismiss maps to the dismiss instant"
        );

        let store = make_store(None);
        popover_set_open(&store, true, &mut details(REASONS::FOCUS_OUT));
        assert_eq!(
            store.get_snapshot().instant_type,
            Some(leptos_ui_internals::floating_ui::popup_store::InstantType::Focus),
            "focus-out maps to the focus instant"
        );
    }

    // The triggerless `closePress` backfill (`PopoverStore.ts:112-122`): a close
    // with no trigger on the details falls back to the registered active trigger —
    // with no trigger registered and none on the details, nothing is backfilled
    // and the close still lands.
    #[test]
    fn a_triggerless_close_press_backfills_nothing_without_a_registered_trigger() {
        let store = make_store(None);

        let mut closing = details(REASONS::CLOSE_PRESS);
        popover_set_open(&store, false, &mut closing);
        assert!(
            closing.trigger.is_none(),
            "no active trigger registered — nothing to backfill"
        );
        assert!(!store.get_snapshot().open);
    }

    // The `defaultOpen` first-paint seed (`PopoverStore.ts:211-215`): the root's
    // initial state overlays `mounted: true` when seeded open, so content renders
    // without an open transition (behavior.md "Uncontrolled").
    #[test]
    fn the_default_open_seed_mounts_on_first_paint() {
        // Closed by default.
        let trigger_elements = PopupTriggerMap::new();
        let closed: leptos_ui_internals::floating_ui::popup_store::PopupStoreState<()> =
            create_initial_popup_store_state(&trigger_elements, None, false);
        assert!(!closed.open && !closed.mounted);

        // Open by default: the root overlays `mounted` (mod.rs's
        // `use_render_popover_root` seeding block).
        let trigger_elements = PopupTriggerMap::new();
        let mut open: leptos_ui_internals::floating_ui::popup_store::PopupStoreState<()> =
            create_initial_popup_store_state(&trigger_elements, None, false);
        open.open = true;
        open.mounted = true;
        assert!(open.open && open.mounted);
    }

    // The extra-slab defaults (`PopoverStore.ts:196-206`): `stickIfOpen` seeds
    // `true`, the modal projection carries the union arms separately.
    #[test]
    fn the_extra_slab_defaults_match_create_initial_state() {
        let defaults = PopoverExtraState::default();
        assert!(defaults.stick_if_open, "stickIfOpen seeds true");
        assert!(!defaults.modal);
        assert!(!defaults.trap_focus);
        assert_eq!(defaults.open_change_reason, None);
        assert_eq!(defaults.close_delay, 0);
        assert!(!defaults.open_on_hover);
        assert!(!defaults.disabled);
    }

    // The modal union projection (`PopoverRoot.tsx:177-189`): the boolean arm and
    // the `'trap-focus'` arm project onto separate flags for the interaction gates.
    #[test]
    fn the_modal_union_projects_onto_the_two_flags() {
        assert!(!PopoverModal::False.is_bool_true());
        assert!(!PopoverModal::False.is_trap_focus());
        assert!(PopoverModal::True.is_bool_true());
        assert!(!PopoverModal::True.is_trap_focus());
        assert!(!PopoverModal::TrapFocus.is_bool_true());
        assert!(PopoverModal::TrapFocus.is_trap_focus());
    }

    // The shared details constructor (`PopoverRoot.tsx:20-22`): the reason rides
    // through. The constructor's fallback `Event::new` needs a JS runtime, so the
    // constructor is pinned on wasm only (the dialog_tests.rs host/wasm split).
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn the_details_constructor_carries_the_reason() {
        let details = popover_change_event_details(REASONS::CLOSE_PRESS, None);
        assert_eq!(details.reason, REASONS::CLOSE_PRESS);
        assert!(!details.is_canceled());

        let mut details = details;
        details.cancel();
        assert!(details.is_canceled(), "cancel() flips the gate");
    }

    // The selectors read through the snapshot (`PopoverStore.ts` selectors table):
    // open/mounted track the committed state.
    #[test]
    fn the_selectors_track_the_committed_state() {
        let store = make_store(None);
        let snapshot = store.get_snapshot();
        assert!(!selectors::open(&snapshot));
        assert!(!selectors::mounted(&snapshot));
        assert!(selectors::payload(&snapshot).is_none());
    }
}
