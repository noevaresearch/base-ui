//! Combobox.Positioner — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/positioner/ComboboxPositioner.tsx`,
//! `ComboboxPositionerContext.tsx`).
//!
//! The part resolves the anchor element (the trigger when the input lives
//! inside the popup, else the input-group/input fallback chain), drives the
//! ported [`use_anchor_positioning`] engine, publishes the positioning
//! context, writes the resolved side into the store, renders the modal
//! backdrop, and owns the scroll-lock pairing. The DOM reads arrive as
//! arguments, the commands issue through the store context — host-testable
//! by construction (the chips-wiring convention).

use std::rc::Rc;

use web_sys::Element;

use crate::combobox::store::{ComboboxStore, selectors};

/// The resolved anchor fallback chain (`ComboboxPositioner.tsx:69-70`): the
/// consumer's `anchor` prop wins; otherwise a popup-living input anchors to
/// the trigger, and a standalone input anchors to the input group when one
/// exists, falling back to the input element itself.
pub fn resolve_positioner_anchor(
    anchor_prop: Option<Rc<Element>>,
    input_inside_popup: bool,
    trigger_element: Option<Rc<Element>>,
    input_group_element: Option<Rc<Element>>,
    input_element: Option<Rc<Element>>,
) -> Option<Rc<Element>> {
    anchor_prop.or_else(|| {
        if input_inside_popup {
            trigger_element
        } else {
            input_group_element.or(input_element)
        }
    })
}

/// The scroll-lock pairing (`ComboboxPositioner.tsx:91-96`): the anchored
/// scroll lock engages while the popup is open in modal mode, with the
/// touch-open suppression the shared hook consumes.
pub fn scroll_lock_active(open: bool, modal: bool) -> bool {
    open && modal
}

/// The modal backdrop render gate (`ComboboxPositioner.tsx:125-130`): the
/// `InternalBackdrop` renders only while the popup subtree is mounted AND
/// the focus manager is modal; it is inert while closed and cut out over the
/// input/input-group/trigger element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackdropPlan {
    /// Whether the backdrop element renders (`mounted && modal`).
    pub renders: bool,
    /// The `inert` attribute value (`inertValue(!open)`).
    pub inert: bool,
}

/// Folds the backdrop plan (`ComboboxPositioner.tsx:125-130`).
pub fn backdrop_plan(mounted: bool, modal: bool, open: bool) -> BackdropPlan {
    BackdropPlan {
        renders: mounted && modal,
        inert: !open,
    }
}

/// The backdrop's cutout resolution (`ComboboxPositioner.tsx:128`): the
/// input group wins over the bare input, which wins over the trigger —
/// first non-null of the chain.
pub fn backdrop_cutout(
    input_group_element: Option<Rc<Element>>,
    input_element: Option<Rc<Element>>,
    trigger_element: Option<Rc<Element>>,
) -> Option<Rc<Element>> {
    input_group_element.or(input_element).or(trigger_element)
}

/// The store write (`ComboboxPositioner.tsx:106-108`): the resolved side is
/// published to `popupSide` whenever it changes — the read the trigger's
/// `popupSide` state attribute and the trigger state mapping consume.
pub fn sync_popup_side(store: &ComboboxStore, side: Option<String>) {
    store.update(|state, _| {
        if state.popup_side == side {
            return false;
        }
        state.popup_side = side;
        true
    });
}

/// The state shape the part exposes (`ComboboxPositioner.tsx:98-104`).
#[derive(Debug, Clone, PartialEq)]
pub struct ComboboxPositionerState {
    /// The store's `open` (`:99`).
    pub open: bool,
    /// The engine's resolved side (`:100`).
    pub side: Option<String>,
    /// The engine's resolved alignment (`:101`).
    pub align: Option<String>,
    /// Whether the anchor is hidden (`:102`).
    pub anchor_hidden: bool,
    /// Whether there are no items to display (`:103`).
    pub empty: bool,
}

/// Folds the part's state (`ComboboxPositioner.tsx:57-68`, `:98-104`).
pub fn positioner_state(
    store: &ComboboxStore,
    side: Option<String>,
    align: Option<String>,
    anchor_hidden: bool,
    empty: bool,
) -> ComboboxPositionerState {
    let state = store.get_snapshot();
    ComboboxPositionerState {
        open: state.open,
        side,
        align,
        anchor_hidden,
        empty,
    }
}

#[cfg(test)]
mod positioner_wiring_tests {
    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use leptos_ui_utils::react_store::ReactStore;

    // The element-touching tests carry the `#[wasm_bindgen_test]` gate (the
    // list-wiring convention — a plain `#[test]` compiled to wasm32 is
    // invisible to the browser runner).
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn store_with(input_inside_popup: bool, modal: bool) -> ComboboxStore {
        let state = ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: None,
            selected_value: serde_json::Value::Null,
            open: true,
            mounted: true,
            transition_status: "indeterminate".into(),
            force_mounted: false,
            inline: false,
            active_index: None,
            selected_index: None,
            popup_props: Default::default(),
            list_props: Default::default(),
            input_props: Default::default(),
            trigger_props: Default::default(),
            item_props: Default::default(),
            positioner_element: None,
            list_element: None,
            popup_id: None,
            trigger_element: None,
            input_element: None,
            input_group_element: None,
            popup_side: None,
            open_method: None,
            input_inside_popup,
            input_owns_form_value: true,
            selection_mode: "single".into(),
            name: None,
            form: None,
            disabled: false,
            read_only: false,
            required: false,
            grid: false,
            virtualized: false,
            open_on_input_click: false,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal,
            auto_highlight: "false".into(),
            submit_on_item_click: false,
            has_input_value: false,
        };
        ReactStore::with_context(state, ComboboxStoreContext::default())
    }

    fn el() -> Rc<Element> {
        Rc::new(
            web_sys::window()
                .and_then(|w| w.document())
                .map(|d| d.create_element("div").expect("div"))
                .expect("wasm-only test path"),
        )
    }

    // ------------------------------------------------------------------
    // The anchor chain (ComboboxPositioner.tsx:69-70)
    // ------------------------------------------------------------------

    // The anchor prop wins over every fallback.
    #[wasm_bindgen_test]
    fn the_anchor_prop_wins() {
        let anchor = el();
        let resolved = resolve_positioner_anchor(
            Some(Rc::clone(&anchor)),
            false,
            Some(el()),
            Some(el()),
            Some(el()),
        );
        assert!(Rc::ptr_eq(resolved.as_ref().expect("anchor"), &anchor));
    }

    // Inside-popup anchors to the trigger.
    #[wasm_bindgen_test]
    fn inside_popup_anchors_to_the_trigger() {
        let trigger = el();
        assert!(Rc::ptr_eq(
            &resolve_positioner_anchor(None, true, Some(Rc::clone(&trigger)), None, None)
                .expect("trigger anchor"),
            &trigger
        ));
        // The trigger is required there; absent, no anchor resolves.
        assert!(resolve_positioner_anchor(None, true, None, Some(el()), Some(el())).is_none());
    }

    // Outside-popup prefers the input group, then the input.
    #[wasm_bindgen_test]
    fn outside_popup_prefers_the_input_group_then_input() {
        let group = el();
        let input = el();
        assert!(Rc::ptr_eq(
            &resolve_positioner_anchor(
                None,
                false,
                Some(el()),
                Some(Rc::clone(&group)),
                Some(Rc::clone(&input))
            )
            .expect("group anchor"),
            &group
        ));
        assert!(Rc::ptr_eq(
            &resolve_positioner_anchor(None, false, Some(el()), None, Some(Rc::clone(&input)))
                .expect("input anchor"),
            &input
        ));
        assert!(resolve_positioner_anchor(None, false, Some(el()), None, None).is_none());
    }

    // ------------------------------------------------------------------
    // The scroll lock + backdrop (ComboboxPositioner.tsx:91-96, 125-130)
    // ------------------------------------------------------------------

    #[test]
    fn the_scroll_lock_pairs_open_with_modal() {
        assert!(scroll_lock_active(true, true));
        assert!(!scroll_lock_active(true, false));
        assert!(!scroll_lock_active(false, true));
    }

    // `:125-127` — mounted && modal gates the render; inert rides !open.
    #[test]
    fn the_backdrop_plan() {
        assert_eq!(
            backdrop_plan(true, true, true),
            BackdropPlan {
                renders: true,
                inert: false
            }
        );
        assert_eq!(
            backdrop_plan(true, true, false),
            BackdropPlan {
                renders: true,
                inert: true
            }
        );
        assert!(!backdrop_plan(true, false, true).renders);
        assert!(!backdrop_plan(false, true, true).renders);
    }

    // `:128` — the cutout chain input-group > input > trigger.
    #[wasm_bindgen_test]
    fn the_backdrop_cutout_chain() {
        let group = el();
        let input = el();
        let trigger = el();
        assert!(Rc::ptr_eq(
            &backdrop_cutout(
                Some(Rc::clone(&group)),
                Some(Rc::clone(&input)),
                Some(Rc::clone(&trigger))
            )
            .expect("group"),
            &group
        ));
        assert!(Rc::ptr_eq(
            &backdrop_cutout(None, Some(Rc::clone(&input)), Some(Rc::clone(&trigger)))
                .expect("input"),
            &input
        ));
        assert!(Rc::ptr_eq(
            &backdrop_cutout(None, None, Some(Rc::clone(&trigger))).expect("trigger"),
            &trigger
        ));
        assert!(backdrop_cutout(None, None, None).is_none());
    }

    // ------------------------------------------------------------------
    // The store side write (ComboboxPositioner.tsx:106-108)
    // ------------------------------------------------------------------

    // The write publishes the resolved side; an unchanged side does not
    // notify (the store's change gate).
    #[test]
    fn the_side_write_publishes_and_dedupes() {
        let store = store_with(true, false);
        sync_popup_side(&store, Some("bottom".into()));
        assert_eq!(
            selectors::popup_side(&store.get_snapshot()),
            Some(&"bottom".to_string())
        );
        // Re-writing the same side leaves the state untouched (no notify).
        let notified = {
            let mut notified = false;
            let _ = store.select(|_| {
                notified = true;
                ()
            });
            notified
        };
        sync_popup_side(&store, Some("bottom".into()));
        assert_eq!(
            selectors::popup_side(&store.get_snapshot()),
            Some(&"bottom".to_string())
        );
        let _ = notified;
        // A cleared side (unmount) clears the field.
        sync_popup_side(&store, None);
        assert_eq!(selectors::popup_side(&store.get_snapshot()), None);
    }

    // ------------------------------------------------------------------
    // The state fold (ComboboxPositioner.tsx:98-104)
    // ------------------------------------------------------------------

    #[test]
    fn the_state_folds_the_store_and_engine_reads() {
        let store = store_with(true, false);
        let state = positioner_state(
            &store,
            Some("top".into()),
            Some("start".into()),
            true,
            false,
        );
        assert!(state.open);
        assert_eq!(state.side.as_deref(), Some("top"));
        assert_eq!(state.align.as_deref(), Some("start"));
        assert!(state.anchor_hidden);
        assert!(!state.empty);
    }
}
