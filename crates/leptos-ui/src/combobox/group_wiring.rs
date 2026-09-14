//! Combobox.Group / Combobox.GroupLabel — the parts' DOM wiring over the store
//! spine (`packages/react/src/combobox/group/ComboboxGroup.tsx`,
//! `group-label/ComboboxGroupLabel.tsx`, `group/ComboboxGroupContext.ts`,
//! `collection/GroupCollectionContext.tsx`).
//!
//! The association seam: Group owns the `labelId` state and provides it (plus
//! its optional `items`) through the group context; GroupLabel registers its
//! id into that state for the element's lifetime and renders hidden. The
//! registration cycle is the already-ported [`LabelIdUpdate`] dispatch shape —
//! upstream's plain `setLabelId(id)` mount write is the `Set` arm and the
//! cleanup's `(currentId) => currentId === id ? undefined : currentId`
//! function-form (`ComboboxGroupLabel.tsx:22-26`) is exactly the
//! `ClearIfCurrent` arm, so a stale cleanup cannot clear a newer label.
//!
//! The DOM reads arrive as arguments, the state lives on a shared handle —
//! host-testable by construction (the chips-wiring convention). React's
//! context provider ports to the handle being passed explicitly at the call
//! sites (the combobox parts' convention, `mod.rs` docs).

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::Value;

use leptos_ui_internals::use_registered_label_id::LabelIdUpdate;

/// The group context value (`ComboboxGroupContext.ts:5-11`): the registered
/// label id, its setter, and the group's optional `items`.
#[derive(Clone)]
pub struct ComboboxGroupContextValue {
    /// `labelId` (`:6`) — the id of the currently mounted GroupLabel, if any.
    pub label_id: Rc<RefCell<Option<String>>>,
    /// `setLabelId` (`:7`) — dispatches [`LabelIdUpdate`] over [`Self::label_id`].
    pub set_label_id: LabelIdSetterHandle,
    /// `items` (`:8-10`) — the group's scoped items, when the prop is set.
    pub items: Option<Vec<Value>>,
}

impl Default for ComboboxGroupContextValue {
    fn default() -> Self {
        Self::new(None)
    }
}

/// The `setLabelId` dispatch (`ComboboxGroupContext.ts:7`) — the same
/// [`LabelIdUpdate`] shape the labelable-provider's setter answers, because
/// GroupLabel's mount write and conditional-clear cleanup map onto its two
/// arms exactly (module docs).
pub type LabelIdSetterHandle = Rc<dyn Fn(LabelIdUpdate)>;

impl std::fmt::Debug for ComboboxGroupContextValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComboboxGroupContextValue")
            .field("label_id", &self.label_id.borrow())
            .field("items.set", &self.items.is_some())
            .finish()
    }
}

impl ComboboxGroupContextValue {
    /// Builds the context value the upstream `useMemo` constructs
    /// (`ComboboxGroup.tsx:19-27`): one shared `labelId` cell, its setter,
    /// and the `items` passthrough.
    pub fn new(items: Option<Vec<Value>>) -> Self {
        let label_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let writer = Rc::clone(&label_id);
        ComboboxGroupContextValue {
            label_id,
            set_label_id: Rc::new(move |update| match update {
                LabelIdUpdate::Set(next) => *writer.borrow_mut() = next,
                LabelIdUpdate::ClearIfCurrent(id) => {
                    let mut current = writer.borrow_mut();
                    if current.as_deref() == Some(id.as_str()) {
                        *current = None;
                    }
                }
            }),
            items,
        }
    }
}

/// Executes the `setLabelId` dispatch (`ComboboxGroupContext.ts:7`) — the
/// free-function form the parts call.
pub fn set_group_label_id(context: &ComboboxGroupContextValue, update: LabelIdUpdate) {
    (context.set_label_id)(update);
}

/// The GroupLabel registration effect body (`ComboboxGroupLabel.tsx:21-27`):
/// the mount write `setLabelId(id)` and the returned cleanup
/// `setLabelId((currentId) => currentId === id ? undefined : currentId)`,
/// paired. The caller runs the mount half at layout-effect time and the
/// returned closure at unmount.
pub fn register_group_label(context: &ComboboxGroupContextValue, id: &str) -> impl Fn() + use<> {
    (context.set_label_id)(LabelIdUpdate::Set(Some(id.to_string())));
    let context = context.clone();
    let id = id.to_string();
    move || {
        (context.set_label_id)(LabelIdUpdate::ClearIfCurrent(id.clone()));
    }
}

/// The group element's `role` attribute (`ComboboxGroup.tsx:44-48`):
/// `rowgroup` when the root renders a grid — `group` is not a valid owned
/// element of `grid`, and `row` must be owned by `grid`, `rowgroup`, or
/// `treegrid` — else `group`.
pub fn group_element_role(grid: bool) -> &'static str {
    if grid { "rowgroup" } else { "group" }
}

/// The group element's `aria-labelledby` (`ComboboxGroup.tsx:49`) — the
/// registered label id, absent until a GroupLabel mounts.
pub fn group_aria_labelledby(context: &ComboboxGroupContextValue) -> Option<String> {
    context.label_id.borrow().clone()
}

/// The GroupLabel element attribute plan (`ComboboxGroupLabel.tsx:29-33`):
/// the resolved `id` (the `useBaseUiId` result — override wins verbatim,
/// otherwise the `base-ui-` prefix — the ported [`leptos_ui_internals::
/// use_base_ui_id`] contract) and `aria-hidden: true`, which the consumer's
/// explicit `undefined` overrides away (the `elementProps` bag lands after
/// the part's own props, the merge order `useRenderElement` defines). The
/// `Option<Option<bool>>` models exactly that: `None` = not specified (the
/// part's `true` stands), `Some(v)` = the consumer's value wins verbatim.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupLabelAttrs {
    /// The resolved `id`.
    pub id: String,
    /// The effective `aria-hidden` after the override resolution.
    pub aria_hidden: Option<bool>,
}

/// Resolves the GroupLabel element attributes (`ComboboxGroupLabel.tsx:29-33`
/// with the `useBaseUiId` override rule, `useBaseUiId.ts:9-11`).
pub fn group_label_attrs(
    id_override: Option<&str>,
    generated_id: &str,
    aria_hidden_override: Option<Option<bool>>,
) -> GroupLabelAttrs {
    GroupLabelAttrs {
        id: id_override.unwrap_or(generated_id).to_string(),
        aria_hidden: match aria_hidden_override {
            // Not specified: the part's own `aria-hidden: true` stands.
            None => Some(true),
            // Consumer value wins verbatim — including the explicit
            // `aria-hidden={undefined}` that removes the attribute.
            Some(explicit) => explicit,
        },
    }
}

/// The `GroupCollectionContext` value (`GroupCollectionContext.tsx:5-7`).
#[derive(Clone, Debug, PartialEq)]
pub struct GroupCollectionContextValue {
    /// The group's items — the provider's whole payload.
    pub items: Vec<Value>,
}

/// The conditional provider gate (`ComboboxGroup.tsx:52-60`): when the group
/// received `items`, the wrapped element is additionally provided through the
/// `GroupCollectionProvider` so nested `Collection` components render
/// group-specific items; without `items`, no collection context appears.
/// The port returns the context value the provider would carry, `None` when
/// the provider is skipped — the view layer provides it conditionally.
pub fn group_collection_provider(
    items: Option<&Vec<Value>>,
) -> Option<GroupCollectionContextValue> {
    items.map(|items| GroupCollectionContextValue {
        items: items.clone(),
    })
}

#[cfg(test)]
mod group_wiring_tests {
    use super::*;
    use crate::combobox::store::{ComboboxState, ComboboxStore, ComboboxStoreContext};
    use serde_json::json;

    // ------------------------------------------------------------------
    // Group — role derivation (ComboboxGroup.test.tsx / ComboboxGroup.tsx:44-48)
    // ------------------------------------------------------------------

    // `ComboboxGroup.tsx:44-48` — `group` is not a valid owned element of
    // `grid`, so a grid root renders the group as `rowgroup`.
    #[test]
    fn group_renders_rowgroup_when_the_root_is_a_grid() {
        assert_eq!(group_element_role(true), "rowgroup");
    }

    // The non-grid default: the accessible `group` role the upstream
    // suites' `screen.getByRole('group')` queries resolve against
    // (ComboboxGroup.test.tsx:23).
    #[test]
    fn group_renders_group_role_by_default() {
        assert_eq!(group_element_role(false), "group");
    }

    // ------------------------------------------------------------------
    // Group — the context value + aria-labelledby (ComboboxGroup.tsx:19-27, :49)
    // ------------------------------------------------------------------

    fn context() -> ComboboxGroupContextValue {
        ComboboxGroupContextValue::new(None)
    }

    // `ComboboxGroup.tsx:49` — before any GroupLabel mounts, the
    // `aria-labelledby` slot is empty (upstream renders `undefined`).
    #[test]
    fn group_aria_labelledby_is_absent_before_a_label_registers() {
        let group = context();
        assert_eq!(group_aria_labelledby(&group), None);
    }

    // `ComboboxGroup.test.tsx:35-53` ("should associate label with group") —
    // after the label mounts, the group's `aria-labelledby` carries the
    // label's id.
    #[test]
    fn group_aria_labelledby_carries_the_registered_label_id() {
        let group = context();
        let cleanup = register_group_label(&group, "test-group");
        assert_eq!(group_aria_labelledby(&group), Some("test-group".into()));
        cleanup();
    }

    // ------------------------------------------------------------------
    // GroupLabel — the registration cycle (ComboboxGroupLabel.tsx:21-27)
    // ------------------------------------------------------------------

    // `ComboboxGroupLabel.test.tsx:22-39` ("wires to group aria-labelledby") —
    // the mount write publishes the label's id to the group.
    #[test]
    fn group_label_registration_publishes_the_id() {
        let group = context();
        assert_eq!(group.label_id.borrow().as_deref(), None);
        let cleanup = register_group_label(&group, "base-ui-1");
        assert_eq!(group.label_id.borrow().as_deref(), Some("base-ui-1"));
        cleanup();
    }

    // `ComboboxGroupLabel.test.tsx:42-58` ("is hidden from the accessibility
    // tree by default") — covered at the attribute layer below; the
    // registration itself is orthogonal.
    //
    // `ComboboxGroupLabel.tsx:23-26` — the unmount cleanup clears the id it
    // registered: the group's `aria-labelledby` returns to absent.
    #[test]
    fn group_label_cleanup_clears_the_registration() {
        let group = context();
        let cleanup = register_group_label(&group, "base-ui-2");
        assert!(group_aria_labelledby(&group).is_some());
        cleanup();
        assert_eq!(group_aria_labelledby(&group), None);
    }

    // `ComboboxGroupLabel.test.tsx:97-129` ("does not let an older label
    // cleanup clear a newer label") — the conditional-clear dispatch: an
    // older label's cleanup running after a newer label registered must not
    // clear the newer id.
    #[test]
    fn an_older_label_cleanup_does_not_clear_a_newer_label() {
        let group = context();
        let cleanup_old = register_group_label(&group, "old-label");
        assert_eq!(group_aria_labelledby(&group), Some("old-label".into()));

        // The newer label mounts (the "both" rerender).
        let cleanup_new = register_group_label(&group, "new-label");
        assert_eq!(group_aria_labelledby(&group), Some("new-label".into()));

        // The old label unmounts AFTER the new one registered ("new") —
        // its cleanup sees a foreign id and leaves it alone.
        cleanup_old();
        assert_eq!(group_aria_labelledby(&group), Some("new-label".into()));

        // The new label's own cleanup still clears its own id.
        cleanup_new();
        assert_eq!(group_aria_labelledby(&group), None);
    }

    // The direct dispatch form (`ComboboxGroupContext.ts:7`) — the free
    // function answers the same two arms the handle's setter does.
    #[test]
    fn the_free_dispatch_answers_both_update_arms() {
        let group = context();
        set_group_label_id(&group, LabelIdUpdate::Set(Some("a".into())));
        assert_eq!(group_aria_labelledby(&group), Some("a".into()));
        set_group_label_id(&group, LabelIdUpdate::ClearIfCurrent("b".into()));
        // A foreign id does not clear.
        assert_eq!(group_aria_labelledby(&group), Some("a".into()));
        set_group_label_id(&group, LabelIdUpdate::ClearIfCurrent("a".into()));
        assert_eq!(group_aria_labelledby(&group), None);
    }

    // ------------------------------------------------------------------
    // GroupLabel — the attribute plan (ComboboxGroupLabel.tsx:29-33)
    // ------------------------------------------------------------------

    // `ComboboxGroupLabel.test.tsx:42-58` — hidden from the accessibility
    // tree by default.
    #[test]
    fn group_label_is_aria_hidden_by_default() {
        let attrs = group_label_attrs(None, "base-ui-3", None);
        assert_eq!(attrs.aria_hidden, Some(true));
        assert_eq!(attrs.id, "base-ui-3");
    }

    // `ComboboxGroupLabel.test.tsx:60-76` — the consumer's explicit
    // `aria-hidden={undefined}` overrides the part's `true` away (the
    // elementProps bag lands after the part's props).
    #[test]
    fn an_explicit_aria_hidden_none_overrides_the_default() {
        let attrs = group_label_attrs(None, "base-ui-4", Some(None));
        assert_eq!(attrs.aria_hidden, None);
    }

    // And an explicit value wins verbatim.
    #[test]
    fn an_explicit_aria_hidden_value_wins_verbatim() {
        let attrs = group_label_attrs(None, "base-ui-5", Some(Some(false)));
        assert_eq!(attrs.aria_hidden, Some(false));
    }

    // `ComboboxGroupLabel.test.tsx:78-95` — the provided id lands in
    // `aria-labelledby` verbatim (the `useBaseUiId` override arm).
    #[test]
    fn the_provided_id_is_used_in_aria_labelledby() {
        let attrs = group_label_attrs(Some("test-group"), "base-ui-6", None);
        assert_eq!(attrs.id, "test-group");

        let group = context();
        let _cleanup = register_group_label(&group, &attrs.id);
        assert_eq!(group_aria_labelledby(&group), Some("test-group".into()));
    }

    // ------------------------------------------------------------------
    // Group — the collection provider gate (ComboboxGroup.tsx:52-60)
    // ------------------------------------------------------------------

    // With `items`, the provider carries them to nested collections.
    #[test]
    fn the_collection_provider_carries_the_group_items() {
        let items = vec![json!("apple"), json!("banana")];
        let provided = group_collection_provider(Some(&items));
        assert_eq!(
            provided,
            Some(GroupCollectionContextValue {
                items: vec![json!("apple"), json!("banana")]
            })
        );
    }

    // Without `items`, no provider — upstream renders the wrapped element
    // bare (`ComboboxGroup.tsx:59-60`).
    #[test]
    fn no_collection_provider_without_items() {
        assert_eq!(group_collection_provider(None), None);
    }

    // ------------------------------------------------------------------
    // The wiring reads the real store shape it is rendered under
    // ------------------------------------------------------------------

    fn grid_state(grid: bool) -> ComboboxState {
        ComboboxState {
            id: Some("root".into()),
            label_id: None,
            items: None,
            selected_value: Value::Null,
            open: true,
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
            selection_mode: "single".into(),
            name: None,
            form: None,
            disabled: false,
            read_only: false,
            required: false,
            grid,
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

    // The role derivation consumes the root store's `grid` selection
    // (`ComboboxGroup.tsx:17`) — the port's selector read drives the same
    // branch.
    #[test]
    fn the_store_grid_selection_drives_the_role() {
        let store = ComboboxStore::with_context(grid_state(true), ComboboxStoreContext::default());
        let grid = store.select(|s| s.grid);
        assert_eq!(group_element_role(grid), "rowgroup");

        let store = ComboboxStore::with_context(grid_state(false), ComboboxStoreContext::default());
        let grid = store.select(|s| s.grid);
        assert_eq!(group_element_role(grid), "group");
    }
}
