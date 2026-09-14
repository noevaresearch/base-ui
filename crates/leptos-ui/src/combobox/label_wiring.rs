//! Combobox.Label — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/label/ComboboxLabel.tsx`).
//!
//! The part derives its association targets from the root store — the trigger
//! element's id (or the root id when the input lives inside the popup) as the
//! fallback control id, and the `{rootId}-label` default as its own id — and
//! registers itself through `useLabel`'s `setLabelId` dispatch, which is the
//! already-ported [`LabelIdUpdate`] shape the store's `labelId` field answers
//! (`ComboboxLabel.tsx:30-56`).
//!
//! The DOM reads arrive as arguments, the registration issues through the
//! store — host-testable by construction (the group-wiring convention). The
//! reactive `useLabel` call itself runs at the view layer inside a reactive
//! owner (the field-parts precedent); this module carries the derivations the
//! part feeds it and the store dispatch it wires.

use crate::combobox::store::{ComboboxState, ComboboxStore};
use leptos_ui_internals::resolve_aria_labelled_by::get_default_label_id;
use leptos_ui_internals::use_registered_label_id::LabelIdUpdate;

/// The dev-only warning message (`ComboboxLabel.tsx:38-43`): emitted when an
/// external `<Combobox.Input>` is the form control — `<Combobox.Label>` labels
/// the trigger only. The upstream message carries the captured owner stack
/// suffix; the port carries the message body (the stack is a dev-tools affordance,
/// not part of the contract — the upstream test pins
/// `stringContaining('<Combobox.Label> labels <Combobox.Trigger> only.')`).
pub const EXTERNAL_INPUT_WARNING: &str = "<Combobox.Label> labels <Combobox.Trigger> only. \
When <Combobox.Input> is the form control, use a native <label> or <Field.Label> instead.";

/// The dev warning's gate (`ComboboxLabel.tsx:35-44`): the effect body — warn
/// only when an input element exists AND it is not inside the popup.
pub fn should_warn_external_input(input_element_present: bool, input_inside_popup: bool) -> bool {
    input_element_present && !input_inside_popup
}

/// The fallback control id (`ComboboxLabel.tsx:32`): the trigger's own id when
/// present, else the root id when the input is inside the popup, else nothing.
pub fn label_local_control_id(
    trigger_element_id: Option<&str>,
    input_inside_popup: bool,
    root_id: Option<&str>,
) -> Option<String> {
    trigger_element_id.map(str::to_string).or_else(|| {
        input_inside_popup
            .then(|| root_id)
            .flatten()
            .map(str::to_string)
    })
}

/// The label element's default id (`ComboboxLabel.tsx:31`): the
/// `getDefaultLabelId(rootId)` derivation — `{rootId}-label`, absent without a
/// root id (the ported [`get_default_label_id`], resolve_aria_labelled_by.rs:13-15).
pub fn label_default_id(root_id: Option<&str>) -> Option<String> {
    get_default_label_id(root_id)
}

/// The `setLabelId` dispatch the part wires into `useLabel`
/// (`ComboboxLabel.tsx:47-53`): the function-form resolution first —
/// `typeof nextLabelId === 'function' ? nextLabelId(store.state.labelId) :
/// nextLabelId` — then the plain store write. The port's [`LabelIdUpdate`]
/// already carries the function-form as the `ClearIfCurrent` arm, so the
/// dispatch is a straight passthrough onto the store's `labelId` field; this
/// executor is the seam the view-layer `useLabel` params call.
pub fn set_store_label_id(store: &ComboboxStore, update: LabelIdUpdate) {
    store.set_field(
        |state| &mut state.label_id,
        resolve_label_id_update(store, update),
    );
}

/// The dispatch's resolution (`ComboboxLabel.tsx:48-50`) — the current
/// `store.state.labelId` handed to the function-form (the `ClearIfCurrent` arm),
/// the plain value passed through (`Set`).
fn resolve_label_id_update(store: &ComboboxStore, update: LabelIdUpdate) -> Option<String> {
    match update {
        LabelIdUpdate::Set(next) => next,
        LabelIdUpdate::ClearIfCurrent(id) => {
            let current = store.get_snapshot().label_id.clone();
            if current.as_deref() == Some(id.as_str()) {
                None
            } else {
                current
            }
        }
    }
}

/// The element-props slot the part strips (`ComboboxLabel.tsx:21-24`): the
/// runtime `id` override is deleted from the element props — the label id is
/// derived from the root, and an untyped consumer's `id` must not win.
/// The port models the strip as the pair (kept id source = the
/// `useLabel`-registered id, dropped = the consumer's) — the attribute plan
/// the view layer renders from.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelAttrs {
    /// The element's rendered `id` — the `useLabel`-registered id (the
    /// override-aware `useBaseUiId` result), never the consumer's raw prop.
    pub id: String,
}

/// Resolves the element's `id` attribute (`ComboboxLabel.tsx:21-24` +
/// `useLabel`'s registered id): the consumer's `id` prop is dropped, the
/// derived default (`{rootId}-label`) stands, and when even that is absent
/// (no root id) the generated `useBaseUiId` id carries — the fallback the
/// registration cycle guarantees.
pub fn label_element_attrs(default_label_id: Option<&str>, generated_id: &str) -> LabelAttrs {
    LabelAttrs {
        id: default_label_id.unwrap_or(generated_id).to_string(),
    }
}

/// The store-state read the part makes (`ComboboxLabel.tsx:26-31`) — the four
/// values the derivations consume, folded over one snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelState {
    /// `store.useState('inputInsidePopup')`.
    pub input_inside_popup: bool,
    /// `store.useState('triggerElement')`'s id, when the element is mounted.
    pub trigger_element_id: Option<String>,
    /// `store.useState('inputElement')`'s presence.
    pub input_element_present: bool,
    /// `store.useState('id')`.
    pub root_id: Option<String>,
}

/// Reads [`LabelState`] from the store (`ComboboxLabel.tsx:26-31`).
pub fn label_state(store: &ComboboxStore) -> LabelState {
    let state = store.get_snapshot();
    LabelState {
        input_inside_popup: state.input_inside_popup,
        trigger_element_id: state
            .trigger_element
            .as_ref()
            .map(|el| el.get_attribute("id").unwrap_or_default()),
        input_element_present: state.input_element.is_some(),
        root_id: state.id.clone(),
    }
}

/// The dev-warning probe over a state snapshot (`ComboboxLabel.tsx:35-44`).
pub fn label_state_warning(state: &LabelState) -> Option<&'static str> {
    should_warn_external_input(state.input_element_present, state.input_inside_popup)
        .then_some(EXTERNAL_INPUT_WARNING)
}

/// The state shape the part's render consumes — re-exported for the view layer's
/// `stateAttributesMapping: fieldValidityMapping` walk (the port's
/// `field_state_attributes`, the field-parts precedent; the Label state adds no
/// combobox-specific members, `ComboboxLabel.tsx:76-78`).
pub type LabelFieldState = ComboboxState;

#[cfg(test)]
mod label_wiring_tests {
    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStoreContext};
    use leptos_ui_utils::react_store::ReactStore;

    fn store_with(
        input_inside_popup: bool,
        input_present: bool,
        root_id: Option<&str>,
    ) -> ComboboxStore {
        let state = ComboboxState {
            id: root_id.map(str::to_string),
            label_id: None,
            items: None,
            selected_value: serde_json::Value::Null,
            open: false,
            mounted: false,
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
            input_element: input_present.then(|| {
                // A stand-in element; the host tests only read its presence.
                web_sys::window()
                    .and_then(|w| w.document())
                    .map(|d| d.create_element("input").unwrap())
                    .expect("wasm-only test path")
            }),
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

    // ------------------------------------------------------------------
    // The fallback control id (ComboboxLabel.tsx:32)
    // ------------------------------------------------------------------

    // `ComboboxLabel.tsx:32` — the trigger's own id wins over everything.
    #[test]
    fn the_trigger_id_is_the_fallback_control_id() {
        assert_eq!(
            label_local_control_id(Some("trigger-1"), true, Some("root")),
            Some("trigger-1".into())
        );
    }

    // `ComboboxLabel.tsx:32` — no trigger id: the root id counts only when
    // the input is inside the popup (the input-inside-popup form's
    // `getComboboxPopupId`-rooted id).
    #[test]
    fn the_root_id_falls_through_only_when_the_input_is_inside_the_popup() {
        assert_eq!(
            label_local_control_id(None, true, Some("root")),
            Some("root".into())
        );
        assert_eq!(label_local_control_id(None, false, Some("root")), None);
        assert_eq!(label_local_control_id(None, true, None), None);
    }

    // ------------------------------------------------------------------
    // The default label id (ComboboxLabel.tsx:31, resolveAriaLabelledBy.ts:3-5)
    // ------------------------------------------------------------------

    // `{rootId}-label` — the default the upstream suites' label queries
    // resolve against.
    #[test]
    fn the_default_label_id_is_the_root_id_with_the_label_suffix() {
        assert_eq!(
            label_default_id(Some("root-1")),
            Some("root-1-label".into())
        );
        // No root id — the `id == null` check yields `undefined`.
        assert_eq!(label_default_id(None), None);
    }

    // ------------------------------------------------------------------
    // The dev warning (ComboboxLabel.tsx:35-44)
    // ------------------------------------------------------------------

    // `ComboboxLabel.test.tsx:37-59` ("warns without relying on
    // React.captureOwnerStack when labeling an external input") — the gate is
    // input-present AND outside the popup, and the message body is stable.
    #[test]
    fn warns_only_for_an_external_input() {
        assert!(should_warn_external_input(true, false));
        assert!(!should_warn_external_input(true, true));
        assert!(!should_warn_external_input(false, false));
        assert!(
            EXTERNAL_INPUT_WARNING.starts_with("<Combobox.Label> labels <Combobox.Trigger> only.")
        );
    }

    // The state-fold probe (`ComboboxLabel.tsx:26-31` + `:35-44`).
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn the_state_fold_probes_the_warning_through_the_store() {
        let external = store_with(false, true, Some("root"));
        assert_eq!(
            label_state_warning(&label_state(&external)),
            Some(EXTERNAL_INPUT_WARNING)
        );
        let inside = store_with(true, true, Some("root"));
        assert_eq!(label_state_warning(&label_state(&inside)), None);
    }

    // ------------------------------------------------------------------
    // The setLabelId dispatch (ComboboxLabel.tsx:47-53)
    // ------------------------------------------------------------------

    // `ComboboxLabel.tsx:47-53` — the plain `Set` dispatch writes the id into
    // the store's `labelId` field (the same field the Root's aria-labelledby
    // resolution reads).
    #[test]
    fn the_set_dispatch_writes_the_store_label_id() {
        let store = store_with(false, false, Some("root"));
        set_store_label_id(&store, LabelIdUpdate::Set(Some("base-ui-1".into())));
        assert_eq!(store.get_snapshot().label_id.as_deref(), Some("base-ui-1"));
    }

    // The function-form (`ComboboxLabel.tsx:48-50` — `nextLabelId(store.state.labelId)`)
    // is the `ClearIfCurrent` arm: the current value is cleared only when it
    // still equals the carried id.
    #[test]
    fn the_clear_if_current_dispatch_clears_only_the_matching_id() {
        let store = store_with(false, false, Some("root"));
        set_store_label_id(&store, LabelIdUpdate::Set(Some("old".into())));
        set_store_label_id(&store, LabelIdUpdate::Set(Some("new".into())));
        // The stale clear sees a foreign id and leaves it.
        set_store_label_id(&store, LabelIdUpdate::ClearIfCurrent("old".into()));
        assert_eq!(store.get_snapshot().label_id.as_deref(), Some("new"));
        // The matching clear removes it.
        set_store_label_id(&store, LabelIdUpdate::ClearIfCurrent("new".into()));
        assert_eq!(store.get_snapshot().label_id.as_deref(), None);
    }

    // ------------------------------------------------------------------
    // The element-attrs plan (ComboboxLabel.tsx:21-24)
    // ------------------------------------------------------------------

    // The consumer's `id` is stripped (`:21-24`) — the derived default stands.
    #[test]
    fn the_element_id_is_the_derived_default_not_the_consumer_override() {
        let attrs = label_element_attrs(Some("root-1-label"), "base-ui-generated");
        assert_eq!(attrs.id, "root-1-label");
    }

    // Without a root id the generated `useBaseUiId` id carries — the
    // registration cycle's own id, still not the consumer's.
    #[test]
    fn without_a_root_id_the_generated_id_carries() {
        let attrs = label_element_attrs(None, "base-ui-generated");
        assert_eq!(attrs.id, "base-ui-generated");
    }
}
