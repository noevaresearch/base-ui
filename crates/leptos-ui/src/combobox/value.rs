//! Combobox.Value — the part's DOM wiring over the store spine
//! (`packages/react/src/combobox/value/ComboboxValue.tsx`).
//!
//! Upstream Value renders a fragment — no element of its own; its entire
//! observable surface is the children resolution. The port mirrors that: this
//! module is the wiring seam that reads the store selections the part makes
//! (`ComboboxValue.tsx:19-24`) and resolves them through the already-ported
//! [`resolve_value_display`] runtime; the caller (the wasm view layer) renders
//! the result. Host-testable by construction — no DOM reads.

use serde_json::Value;

use crate::combobox::store::{ComboboxStore, selectors};
use crate::combobox::value_chips::{ValueChildren, ValueDisplay, resolve_value_display};

/// The part's props (`ComboboxValue.Props`, `ComboboxValue.tsx:63-76`):
/// the children shape and the placeholder.
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxValueProps {
    /// The `children` prop shape (`ComboboxValue.tsx:15-17`).
    pub children: ValueChildren,
    /// The `placeholder` prop — `None` is upstream `undefined`.
    pub placeholder: Option<String>,
}

impl Default for ComboboxValueProps {
    fn default() -> Self {
        Self {
            children: ValueChildren::None,
            placeholder: None,
        }
    }
}

/// The `ComboboxValue` render body (`ComboboxValue.tsx:17-37`) over the store:
/// reads the part's five store selections, computes the
/// `shouldCheckNullItemLabel` gate exactly as the part does, and resolves the
/// display through the runtime's precedence chain.
pub fn combobox_value_display(store: &ComboboxStore, props: &ComboboxValueProps) -> ValueDisplay {
    // The selections (`:19-24`). `select` clones the state once — the part reads
    // a consistent snapshot, matching React's render-time store read.
    let state = store.select(|s| s.clone());

    let has_selected_value = selectors::has_selected_value(&state);
    // The raw items-scan: the runtime computes the `shouldCheckNullItemLabel`
    // gate from the same inputs, so the raw value is what upstream's disabled
    // selector would scan with (the gate belongs to the precedence chain, not
    // the read).
    let has_null_item_label_raw = selectors::has_null_item_label(&state, true);

    resolve_value_display(
        &props.children,
        props.placeholder.is_some(),
        has_selected_value,
        has_null_item_label_raw,
        &state.selection_mode,
        &state.selected_value,
        state
            .items
            .as_ref()
            .map(|items| Value::Array(items.clone()))
            .as_ref(),
        state.item_to_string_label.as_ref(),
    )
}

/// The textual form of a resolved display — the string the fragment renders
/// for the non-function branches. The `RenderFunction` arm has no textual form
/// (the caller invokes it with the selected value); `Nothing` renders nothing.
pub fn display_to_text(display: &ValueDisplay) -> Option<String> {
    match display {
        ValueDisplay::Nothing => None,
        ValueDisplay::RenderFunction => None,
        ValueDisplay::Node => None, // static nodes carry their own content
        ValueDisplay::Placeholder => None, // resolved by the caller's prop value
        ValueDisplay::Multiple(values) => Some(
            // Upstream renders the folded labels as sibling nodes with `", "`
            // separator nodes between them — the textual form is the plain
            // concatenation, not a re-join (a re-join double-counts the
            // separators already present as nodes).
            values
                .iter()
                .map(|value| match value {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<String>(),
        ),
        ValueDisplay::Single(value) => match value {
            Value::String(s) => Some(s.clone()),
            Value::Null => None,
            other => Some(other.to_string()),
        },
    }
}

#[cfg(test)]
mod value_wiring_tests {
    use serde_json::{Value, json};
    use std::rc::Rc;

    use super::*;
    use crate::combobox::store::ComboboxState;
    use crate::combobox::value_chips::ValueChildren;

    fn state(selection_mode: &str) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: Some(vec![json!("Apple"), json!("Banana")]),
            selected_value: Value::Null,
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
            input_element: None,
            input_group_element: None,
            popup_side: None,
            open_method: None,
            input_inside_popup: false,
            input_owns_form_value: true,
            selection_mode: selection_mode.into(),
            name: None,
            form: None,
            disabled: false,
            read_only: false,
            required: false,
            grid: false,
            virtualized: false,
            open_on_input_click: true,
            item_to_string_label: None,
            is_item_equal_to_value: ComboboxState::default_is_item_equal_to_value(),
            modal: false,
            auto_highlight: "true".into(),
            submit_on_item_click: false,
            has_input_value: false,
        }
    }

    fn store(selection_mode: &str) -> ComboboxStore {
        ComboboxStore::with_context(state(selection_mode), Default::default())
    }

    // `ComboboxValue.test.tsx:620-683` — the label resolves for the selection.
    #[test]
    fn single_selection_resolves_the_label_over_the_store() {
        let store = store("single");
        store.set_field(|s| &mut s.selected_value, json!("Apple"));
        let display = combobox_value_display(
            &store,
            &ComboboxValueProps {
                children: ValueChildren::None,
                placeholder: Some("Pick one".into()),
            },
        );
        assert_eq!(display, ValueDisplay::Single(json!("Apple")));
        assert_eq!(display_to_text(&display).as_deref(), Some("Apple"));
    }

    // `ComboboxValue.test.tsx:525-592` — multiple mode folds labels with ", ".
    #[test]
    fn multiple_selection_folds_the_labels() {
        let store = store("multiple");
        store.set_field(|s| &mut s.selected_value, json!(["Apple", "Banana"]));
        let display = combobox_value_display(
            &store,
            &ComboboxValueProps {
                children: ValueChildren::None,
                placeholder: None,
            },
        );
        assert_eq!(
            display,
            ValueDisplay::Multiple(vec![
                Value::String("Apple".into()),
                Value::String(", ".into()),
                Value::String("Banana".into()),
            ]),
            "the labels fold with the \", \" separator nodes upstream renders between them"
        );
        assert_eq!(display_to_text(&display).as_deref(), Some("Apple, Banana"));
    }

    // `ComboboxValue.test.tsx:760-802` — static children win over everything.
    #[test]
    fn children_node_wins_over_the_selection() {
        let store = store("single");
        store.set_field(|s| &mut s.selected_value, json!("Apple"));
        let display = combobox_value_display(
            &store,
            &ComboboxValueProps {
                children: ValueChildren::Node,
                placeholder: Some("Pick one".into()),
            },
        );
        assert_eq!(display, ValueDisplay::Node);
    }

    // `ComboboxValue.test.tsx:831-856` — placeholder only when the items carry
    // no null-value label and nothing is selected.
    #[test]
    fn placeholder_shows_only_when_nothing_selected_and_no_null_label() {
        let store = store("single");
        let props = ComboboxValueProps {
            children: ValueChildren::None,
            placeholder: Some("Pick one".into()),
        };
        let display = combobox_value_display(&store, &props);
        assert_eq!(display, ValueDisplay::Placeholder);

        // A null-value item WITH a non-null label: the hasNullItemLabel gate
        // fires, the placeholder is suppressed, and the null selection matches
        // that item — its label is the display (the value-chips spec's
        // `ComboboxValue.test.tsx:831-856` claim, the runtime test's mirror).
        store.set_field(
            |s| &mut s.items,
            Some(vec![json!({"value": null, "label": "Nothing selected"})]),
        );
        let display = combobox_value_display(&store, &props);
        assert_eq!(
            display,
            ValueDisplay::Single(Value::String("Nothing selected".into()))
        );
        assert_eq!(
            display_to_text(&display).as_deref(),
            Some("Nothing selected")
        );
    }

    // The render-function arm is flagged for the caller, not resolved here
    // (`ComboboxValue.tsx:18-20` — `childrenProp(selectedValue)`).
    #[test]
    fn render_function_arm_is_flagged_for_the_caller() {
        let store = store("single");
        let display = combobox_value_display(
            &store,
            &ComboboxValueProps {
                children: ValueChildren::RenderFunction,
                placeholder: None,
            },
        );
        assert_eq!(display, ValueDisplay::RenderFunction);
        assert_eq!(display_to_text(&display), None);
    }

    // The store-carried stringifier feeds the label resolution
    // (`ComboboxValue.tsx:19` — `itemToStringLabel`).
    #[test]
    fn item_to_string_label_feeds_the_resolution() {
        let mut st = state("single");
        st.selected_value = json!("anything");
        st.item_to_string_label = Some(Rc::new(|_| "LABEL".to_string()));
        let store = ComboboxStore::with_context(st, Default::default());
        let display = combobox_value_display(
            &store,
            &ComboboxValueProps {
                children: ValueChildren::None,
                placeholder: None,
            },
        );
        assert_eq!(display, ValueDisplay::Single(json!("LABEL")));
    }
}
