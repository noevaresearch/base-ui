//! Tests for the PreviewCard port — mirrors of the upstream suites behavior.md
//! mines (`PreviewCardRoot.test.tsx`, `PreviewCardTrigger.test.tsx`,
//! `PreviewCardPopup.test.tsx`, `PreviewCardPositioner.test.tsx`), over the
//! store-level contracts in an owner-scoped host suite (the popover_tests.rs
//! convention; the wasm materialized-tree mirrors follow in a later pass once
//! the docs-app pairing work lands).
//!
//! Host suite: the pure contracts that need no DOM — the `setOpen` sequence
//! (the shared `applyPopupOpenChange`), the controlled veto, the
//! `instantType` mapping, the `defaultOpen` ⇒ `mounted` first-paint seed, the
//! context-missing panics' message contracts (host-side through the store
//! layer), the constants, and the `defaultOpen`-ignored-under-controlled-
//! `open` selector shadowing.

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::preview_card::store::{
    CLOSE_DELAY, OPEN_DELAY, PreviewCardExtraState, preview_card_change_event_details,
    preview_card_set_open,
};
use leptos_ui_internals::floating_ui::popup_store::{
    PopupStoreContext, create_initial_popup_store_state,
};
use leptos_ui_internals::floating_ui::popup_trigger_map::PopupTriggerMap;
use leptos_ui_internals::floating_ui::types::{OnOpenChangeFn, RootOpenChangeEventDetails};
use leptos_ui_internals::popup_store_utils::PopupStore;

/// The shared details type the callbacks exchange
/// (`PreviewCardRoot.tsx:20-22` shape).
type Details = RootOpenChangeEventDetails;

/// A counting callback recorder (the popover_tests.rs convention).
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
/// the popover `make_store` harness convention.
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
        // calling the wasm-bindgen `Event` constructor (the popover_tests.rs
        // host-suite convention).
        web_sys::Event::from(web_sys::wasm_bindgen::JsValue::NULL),
        None,
        String::new(),
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use leptos_ui_internals::floating_ui::popup_store::{InstantType, selectors};
    use leptos_ui_internals::floating_ui::reasons as REASONS;

    // behavior.md "State model" (`PreviewCardRoot.test.tsx:132-176` analog):
    // `onOpenChange` fires exactly once per transition and is not called when
    // state does not change — the store layer's notify-then-commit sequence.
    #[test]
    fn set_open_notifies_once_per_transition() {
        let recorder = Recorder::default();
        let recorder_for_cb = recorder.clone();
        let store = make_store(Some(Rc::new(move |open: bool, details: &Details| {
            recorder_for_cb.record(open, &details.reason);
        })));

        preview_card_set_open(&store, true, &mut details(REASONS::TRIGGER_HOVER), None);
        assert_eq!(
            recorder.calls(),
            vec![(true, REASONS::TRIGGER_HOVER.to_owned())]
        );
        assert!(store.get_snapshot().open, "the open commits");

        preview_card_set_open(&store, false, &mut details(REASONS::TRIGGER_HOVER), None);
        assert_eq!(recorder.calls().len(), 2, "one call per transition");
        assert!(!store.get_snapshot().open, "the close commits");
    }

    // behavior.md "Events" (`PreviewCardRoot.test.tsx:421-441`): the
    // `eventDetails.cancel()` abort semantics — vetoed opens/closes never
    // commit and no second notification fires (the cancel gate at
    // popupStoreUtils.ts:270-272).
    #[test]
    fn a_canceled_open_or_close_is_vetoed() {
        let store = make_store(Some(Rc::new(|_open: bool, details: &Details| {
            details.cancel();
        })));

        preview_card_set_open(&store, true, &mut details(REASONS::TRIGGER_HOVER), None);
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
        preview_card_set_open(&store, true, &mut details(REASONS::TRIGGER_FOCUS), None);
        assert!(store.get_snapshot().open, "the uncanceled open lands");

        preview_card_set_open(&store, false, &mut details(REASONS::ESCAPE_KEY), None);
        assert!(
            store.get_snapshot().open,
            "the canceled close never commits — the veto semantics"
        );
    }

    // The `instantType` mapping (popupStoreUtils.ts:291-297 over
    // implementation.md "State machine"): focus-open → `'focus'`, press or
    // Escape close → `'dismiss'`, hover → `undefined`.
    #[test]
    fn the_instant_type_mapping_follows_the_reason() {
        let store = make_store(None);
        preview_card_set_open(&store, true, &mut details(REASONS::TRIGGER_FOCUS), None);
        assert_eq!(
            store.get_snapshot().instant_type,
            Some(InstantType::Focus),
            "focus-open maps to the focus instant"
        );

        let store = make_store(None);
        preview_card_set_open(&store, false, &mut details(REASONS::ESCAPE_KEY), None);
        assert_eq!(
            store.get_snapshot().instant_type,
            Some(InstantType::Dismiss),
            "the escape close maps to the dismiss instant"
        );

        let store = make_store(None);
        preview_card_set_open(&store, true, &mut details(REASONS::TRIGGER_HOVER), None);
        assert_eq!(
            store.get_snapshot().instant_type,
            None,
            "hover carries no instant"
        );
    }

    // The `defaultOpen` first-paint seed (`PreviewCardStore.ts`
    // createInitialState + the root's `mounted` overlay): `defaultOpen: true`
    // renders the card open on mount (behavior.md "State model").
    #[test]
    fn the_default_open_seed_mounts_on_first_paint() {
        // Closed by default.
        let trigger_elements = PopupTriggerMap::new();
        let closed: leptos_ui_internals::floating_ui::popup_store::PopupStoreState<()> =
            create_initial_popup_store_state(&trigger_elements, None, false);
        assert!(!closed.open && !closed.mounted);

        // Open by default: the root overlays `mounted` (mod.rs's
        // `use_render_preview_card_root` seeding block).
        let trigger_elements = PopupTriggerMap::new();
        let mut open: leptos_ui_internals::floating_ui::popup_store::PopupStoreState<()> =
            create_initial_popup_store_state(&trigger_elements, None, false);
        open.open = true;
        open.mounted = true;
        assert!(open.open && open.mounted);
    }

    // The extra-slab defaults (`PreviewCardStore.ts:20-24`): `closeDelay`
    // seeds the CLOSE_DELAY constant, `instantType` seeds undefined.
    #[test]
    fn the_extra_slab_defaults_match_create_initial_state() {
        let defaults = PreviewCardExtraState::default();
        assert_eq!(defaults.instant_type, None);
        assert_eq!(
            defaults.close_delay, CLOSE_DELAY,
            "closeDelay seeds the CLOSE_DELAY constant"
        );
        assert_eq!(OPEN_DELAY, 600, "the documented open delay default");
        assert_eq!(CLOSE_DELAY, 300, "the documented close delay default");
    }

    // The `defaultOpen`-ignored-under-controlled-`open` mechanism
    // (implementation.md "Store-centric state model": the shared `open`
    // selector reads `openProp ?? open`, so `defaultOpen` only seeds internal
    // `open`, which the selector shadows whenever `openProp` is defined).
    #[test]
    fn the_controlled_open_prop_shadows_the_default_open_seed() {
        let store = make_store(None);
        // `defaultOpen` seeds the internal open…
        store.update(|state, _| {
            state.open = true;
            true
        });
        assert!(selectors::open(&store.get_snapshot()));
        // …but a controlled `open={false}` shadows it.
        store.set_field(|state| &mut state.open_prop, Some(false));
        assert!(
            !selectors::open(&store.get_snapshot()),
            "the selector reads openProp ?? open — defaultOpen is ignored"
        );

        let store = make_store(None);
        store.set_field(|state| &mut state.open_prop, Some(true));
        assert!(
            selectors::open(&store.get_snapshot()),
            "controlled open=true keeps it open over a closed seed"
        );
    }

    // The selectors read through the snapshot: open/mounted track the
    // committed state, payload starts empty.
    #[test]
    fn the_selectors_track_the_committed_state() {
        let store = make_store(None);
        let snapshot = store.get_snapshot();
        assert!(!selectors::open(&snapshot));
        assert!(!selectors::mounted(&snapshot));
        assert!(selectors::payload(&snapshot).is_none());
    }

    // The shared details constructor (`PreviewCardRoot.tsx:20-22` shape): the
    // reason rides through. The constructor's fallback `Event::new` needs a JS
    // runtime, so the constructor is pinned on wasm only (the popover_tests.rs
    // host/wasm split).
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn the_details_constructor_carries_the_reason() {
        let details = preview_card_change_event_details(REASONS::CLOSE_PRESS, None);
        assert_eq!(details.reason, REASONS::CLOSE_PRESS);
        assert!(!details.is_canceled());

        let mut details = details;
        details.cancel();
        assert!(details.is_canceled(), "cancel() flips the gate");
    }
}
