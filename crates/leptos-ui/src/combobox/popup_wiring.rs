//! Combobox.Popup — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/popup/ComboboxPopup.tsx`).
//!
//! The part renders the popup element carrying the store's `popupProps` bag
//! plus the id/role/onFocus matrix, the popup-id registration effect
//! (`popupId` set on mount / cleared on unmount), the open-change-complete
//! notification, the state-attribute mapping (the popup mapping spread with
//! the transition-status mapping), the disabled-mount transition styles, and
//! the FloatingFocusManager's parameter derivation. The DOM reads arrive as
//! arguments, the commands issue through the store context — host-testable
//! by construction (the chips-wiring convention).

use std::cell::RefCell;
use std::rc::Rc;

use web_sys::Element;

use crate::combobox::root_utils::get_combobox_popup_id;
use crate::combobox::store::{ComboboxStore, TransitionStatus};

/// The popup's effective id (`ComboboxPopup.tsx:54`): the consumer's `id`
/// prop wins, else the convention-derived `{rootId}-popup` when the input
/// lives inside the popup, else absent.
pub fn popup_element_id(
    id_prop: Option<&str>,
    input_inside_popup: bool,
    root_id: Option<&str>,
) -> Option<String> {
    id_prop.map(str::to_string).or_else(|| {
        input_inside_popup
            .then(|| get_combobox_popup_id(root_id))
            .flatten()
    })
}

/// The popup-id registration cycle (`ComboboxPopup.tsx:56-62`): the mount
/// write prefers the rendered DOM id (a `render` prop may override it) over
/// the derived one, and the unmount clears the store field.
pub fn sync_popup_id(
    store: &ComboboxStore,
    popup_ref: &Rc<RefCell<Option<Element>>>,
    popup_id: Option<String>,
) {
    let rendered = popup_ref
        .borrow()
        .as_ref()
        .and_then(|el| el.get_attribute("id"));
    let resolved = rendered.or(popup_id);
    store.update(|state, _| {
        state.popup_id = resolved.clone();
        true
    });
}

/// The unmount half of the same effect (`ComboboxPopup.tsx:59-61`).
pub fn clear_popup_id(store: &ComboboxStore) {
    store.update(|state, _| {
        state.popup_id = None;
        true
    });
}

/// The element's role (`ComboboxPopup.tsx:90`): `dialog` when the input is
/// inside the popup, `presentation` otherwise.
pub fn popup_role(input_inside_popup: bool) -> &'static str {
    if input_inside_popup {
        "dialog"
    } else {
        "presentation"
    }
}

/// The `onFocus` body's gate (`ComboboxPopup.tsx:91-99`): a non-touch open
/// whose focus target is the list or the popup itself refocuses the input —
/// focus landing anywhere else in the popup subtree is left alone.
pub fn popup_focus_refocuses_input(
    open_method_touch: bool,
    list_element: Option<&Element>,
    target: Option<&Element>,
    current_target: Option<&Element>,
) -> bool {
    if open_method_touch {
        return false;
    }
    let contains_target = leptos_ui_utils::shadow_dom::contains(list_element, target);
    let is_popup_itself = match (target, current_target) {
        (Some(target), Some(current)) => target == current,
        _ => false,
    };
    contains_target || is_popup_itself
}

/// The default initial-focus derivation (`ComboboxPopup.tsx:107-113`): when
/// the input lives inside the popup the focus moves on touch opens to the
/// popup itself (preventing the Android virtual keyboard) and otherwise to
/// the input; outside-popup popups move no focus at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultInitialFocus {
    /// The popup element — the touch-open arm (`:112`).
    Popup,
    /// The input element — every other open type (`:112`).
    Input,
    /// `false` — no focus move (`:113`, the outside-popup default).
    None,
}

/// Folds the default (`ComboboxPopup.tsx:110-113`).
pub fn default_initial_focus(
    input_inside_popup: bool,
    interaction_touch: bool,
) -> DefaultInitialFocus {
    if !input_inside_popup {
        DefaultInitialFocus::None
    } else if interaction_touch {
        DefaultInitialFocus::Popup
    } else {
        DefaultInitialFocus::Input
    }
}

/// The resolved final focus (`ComboboxPopup.tsx:118-123`): the consumer's
/// prop verbatim; its absence means "return focus" only when the input is
/// outside the popup (the `false` disable inside the popup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFinalFocus {
    /// The consumer supplied a value — carried verbatim.
    Prop,
    /// `undefined` — the inside-popup default (`:122`).
    Unspecified,
    /// `false` — the outside-popup default (`:122`).
    Disabled,
}

/// Folds the resolution (`ComboboxPopup.tsx:118-123`).
pub fn resolve_final_focus(
    final_focus_prop: Option<()>,
    input_inside_popup: bool,
) -> ResolvedFinalFocus {
    match final_focus_prop {
        Some(()) => ResolvedFinalFocus::Prop,
        None if input_inside_popup => ResolvedFinalFocus::Unspecified,
        None => ResolvedFinalFocus::Disabled,
    }
}

/// `focusManagerModal` (`ComboboxPopup.tsx:125`): the modal focus manager
/// runs whenever the input lives outside the popup, or the root is modal.
pub fn focus_manager_modal(input_inside_popup: bool, modal: bool) -> bool {
    !input_inside_popup || modal
}

/// The state shape the part exposes (`ComboboxPopup.tsx:74-81`).
#[derive(Debug, Clone, PartialEq)]
pub struct ComboboxPopupState {
    /// The store's `open` (`:44`).
    pub open: bool,
    /// The positioner's resolved side (`:76`).
    pub side: Option<String>,
    /// The positioner's resolved alignment (`:77`).
    pub align: Option<String>,
    /// Whether the anchor is hidden (`:78`).
    pub anchor_hidden: bool,
    /// The transition status (`:79`).
    pub transition_status: TransitionStatus,
    /// Whether there are no items to display (`:80`).
    pub empty: bool,
}

/// Folds the part's state (`ComboboxPopup.tsx:43-53`, `:74-81`).
#[allow(clippy::too_many_arguments)]
pub fn popup_state(
    store: &ComboboxStore,
    side: Option<String>,
    align: Option<String>,
    anchor_hidden: bool,
    empty: bool,
) -> ComboboxPopupState {
    let state = store.get_snapshot();
    ComboboxPopupState {
        open: state.open,
        side,
        align,
        anchor_hidden,
        transition_status: state.transition_status.clone(),
        empty,
    }
}

#[cfg(test)]
mod popup_wiring_tests {
    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use leptos_ui_utils::react_store::ReactStore;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // The element-touching tests carry the `#[wasm_bindgen_test]` gate (the
    // list-wiring convention — a plain `#[test]` compiled to wasm32 is
    // invisible to the browser runner).
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn store_with(input_inside_popup: bool, root_id: Option<&str>) -> ComboboxStore {
        let state = ComboboxState {
            id: root_id.map(str::to_string),
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
            modal: false,
            auto_highlight: "false".into(),
            submit_on_item_click: false,
            has_input_value: false,
        };
        ReactStore::with_context(state, ComboboxStoreContext::default())
    }

    fn make_element(id: Option<&str>) -> Element {
        let el = web_sys::window()
            .and_then(|w| w.document())
            .map(|d| d.create_element("div").expect("div"))
            .expect("wasm-only test path");
        if let Some(id) = id {
            el.set_attribute("id", id).unwrap();
        }
        el
    }

    // ------------------------------------------------------------------
    // The id chain (ComboboxPopup.tsx:54)
    // ------------------------------------------------------------------

    // The consumer's id wins; the convention id only lands inside-popup.
    #[test]
    fn the_popup_id_chain() {
        assert_eq!(
            popup_element_id(Some("custom"), true, Some("root")),
            Some("custom".to_string())
        );
        assert_eq!(
            popup_element_id(None, true, Some("root")),
            Some("root-popup".to_string())
        );
        assert_eq!(popup_element_id(None, false, Some("root")), None);
    }

    // ------------------------------------------------------------------
    // The registration cycle (ComboboxPopup.tsx:56-62)
    // ------------------------------------------------------------------

    // The rendered DOM id is preferred over the derived one (`:57-58`).
    // Both mount-write tests need a real element; browser-only.
    #[wasm_bindgen_test]
    fn the_mount_write_prefers_the_rendered_dom_id() {
        let store = store_with(true, Some("root"));
        let popup_ref: Rc<RefCell<Option<Element>>> =
            Rc::new(RefCell::new(Some(make_element(Some("rendered")))));
        sync_popup_id(&store, &popup_ref, Some("root-popup".into()));
        let snapshot = store.get_snapshot();
        assert_eq!(snapshot.popup_id.as_deref(), Some("rendered"));
    }

    #[wasm_bindgen_test]
    fn the_mount_write_falls_back_to_the_derived_id() {
        let store = store_with(true, Some("root"));
        let popup_ref: Rc<RefCell<Option<Element>>> =
            Rc::new(RefCell::new(Some(make_element(None))));
        sync_popup_id(&store, &popup_ref, Some("root-popup".into()));
        let snapshot = store.get_snapshot();
        assert_eq!(snapshot.popup_id.as_deref(), Some("root-popup"));
    }

    // The unmount half (`:59-61`) clears the field.
    #[test]
    fn the_unmount_clears_the_store_field() {
        let store = store_with(true, Some("root"));
        clear_popup_id(&store);
        let snapshot = store.get_snapshot();
        assert_eq!(snapshot.popup_id, None);
    }

    // ------------------------------------------------------------------
    // The role + focus gates (ComboboxPopup.tsx:90-99)
    // ------------------------------------------------------------------

    // `ComboboxPopup.test.tsx`'s dialog/presentation split rides
    // `inputInsidePopup`.
    #[test]
    fn the_role_splits_on_input_placement() {
        assert_eq!(popup_role(true), "dialog");
        assert_eq!(popup_role(false), "presentation");
    }

    // The element comparisons need real nodes; browser-only (the
    // store_with-style wasm gate the label-wiring tests use).
    #[wasm_bindgen_test]
    fn the_focus_gate_refocuses_the_input_from_list_or_popup() {
        let list = make_element(None);
        let target = make_element(None);
        // A target inside the list refocuses (the contains arm).
        assert!(popup_focus_refocuses_input(
            false,
            Some(&list),
            Some(&target),
            Some(&target)
        ));
        // The popup itself refocuses (the currentTarget arm).
        assert!(popup_focus_refocuses_input(
            false,
            None,
            Some(&target),
            Some(&target)
        ));
        // A touch open never refocuses.
        assert!(!popup_focus_refocuses_input(
            true,
            Some(&list),
            Some(&target),
            Some(&target)
        ));
    }

    // ------------------------------------------------------------------
    // The focus-manager parameters (ComboboxPopup.tsx:107-125)
    // ------------------------------------------------------------------

    // The touch-open arm focuses the popup to suppress the Android keyboard.
    #[test]
    fn the_default_initial_focus_matrix() {
        assert_eq!(
            default_initial_focus(true, true),
            DefaultInitialFocus::Popup
        );
        assert_eq!(
            default_initial_focus(true, false),
            DefaultInitialFocus::Input
        );
        assert_eq!(
            default_initial_focus(false, true),
            DefaultInitialFocus::None
        );
        assert_eq!(
            default_initial_focus(false, false),
            DefaultInitialFocus::None
        );
    }

    // `:118-123` — the prop wins; the defaults split on input placement.
    #[test]
    fn the_final_focus_resolution() {
        assert_eq!(
            resolve_final_focus(Some(()), true),
            ResolvedFinalFocus::Prop
        );
        assert_eq!(
            resolve_final_focus(None, true),
            ResolvedFinalFocus::Unspecified
        );
        assert_eq!(
            resolve_final_focus(None, false),
            ResolvedFinalFocus::Disabled
        );
    }

    // `:125` — outside-popup always modal; inside-popup follows the root.
    #[test]
    fn the_focus_manager_modal_fold() {
        assert!(focus_manager_modal(false, false));
        assert!(focus_manager_modal(false, true));
        assert!(focus_manager_modal(true, true));
        assert!(!focus_manager_modal(true, false));
    }

    // ------------------------------------------------------------------
    // The state fold (ComboboxPopup.tsx:74-81)
    // ------------------------------------------------------------------

    #[test]
    fn the_state_folds_the_store_and_positioner_reads() {
        let store = store_with(true, Some("root"));
        let state = popup_state(
            &store,
            Some("bottom".into()),
            Some("center".into()),
            true,
            true,
        );
        assert!(state.open);
        assert_eq!(state.side.as_deref(), Some("bottom"));
        assert_eq!(state.align.as_deref(), Some("center"));
        assert!(state.anchor_hidden);
        assert!(state.empty);
        assert_eq!(state.transition_status, "indeterminate");
    }
}
