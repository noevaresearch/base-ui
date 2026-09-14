//! The combobox store — `packages/react/src/combobox/store.ts:1-205` ported over the
//! shared [`ReactStore`] engine (`leptos_ui_utils::react_store`).
//!
//! Upstream shape (store.ts):
//! - [`ComboboxState`] ports `State` (`store.ts:10-63`), grouped by role — identity/labels,
//!   selection, popup lifecycle, structure, elements, pre-computed prop bags, config
//!   passthroughs — with every field the selectors read. Upstream element fields are
//!   `HTMLElement | null`; the port carries `Option<web_sys::Element>` (the
//!   popover/parts.rs element-slot precedent).
//! - The two-tier split (`store.ts:69-120`): reactive [`ComboboxState`] read through
//!   selectors vs the non-reactive [`ComboboxStoreContext`] carrying the refs and the
//!   NOOP-seeded command slots every part registers into.
//! - [`selectors`] (`store.ts:122-203`) port as free functions over `&ComboboxState`:
//!   the derived predicates `has_selection_chips` (`:129-132`), `has_selected_value`
//!   (`:134-143`, where `[]` reads as "no value" in multiple mode), `is_active`
//!   (`:157`), and `is_selected` (`:158-167`), which fans an array `selectedValue`
//!   through [`compare_item_equality`].
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Items/values are the crate's dynamic [`serde_json::Value`] representation (the
//!   `resolve_value_label` precedent): upstream's `any`-shaped `selectedValue` and
//!   `items` are one runtime shape in JS. `selectedValue`'s "unset" state ports to
//!   `Value::Null` — a JS `null` selection is a *value*, and the `multiple` array
//!   branch is what distinguishes the shapes.
//! - Upstream selectors are named functions addressable by string key from
//!   `store.useState('key')`; Rust call sites take the field directly or call these
//!   closures, so the port keeps the function table as free functions (the
//!   floating_root_store registry convention) and the string-keyed indirection is N/A.
//! - The store is [`ReactStore<ComboboxState, ComboboxStoreContext>`] — upstream
//!   `ComboboxStore = ReactStore<State, ComboboxStoreContext, typeof selectors>`
//!   (`store.ts:205`); the third type argument is the JS selector registry, which the
//!   port's ReactStore does not carry (see its module docs).
//! - The element-typed fields (`positionerElement`, `triggerElement`, …) port to
//!   `Option<web_sys::Element>`; HTML-input-specific ones lose their subtype (the port
//!   has no HTMLInputElement wrapper need at this layer) and stay `Element`.
//! - The pre-computed prop bags (`popupProps`, `listProps`, `inputProps`,
//!   `triggerProps`, `itemProps`) port to [`HTMLProps`] — upstream `HTMLProps`
//!   (`internals/types.ts`) is the ref-carrying bag shape the root fills and the parts
//!   spread; the port's bag is the same single-`node_ref` shape.
//! - Command slots seed with a shared [`noop_command`] (`NOOP`,
//!   `AriaCombobox.tsx:516-525`) and are bound to the real implementations during the
//!   root's construction; the port's slots are `Rc<dyn Fn>` fields on the context,
//!   replaceable at any time (the `useContextCallback` binding mechanism is a React
//!   render-phase artifact — Leptos components run once, so the root assigns the
//!   closures directly after creating the context).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde_json::Value;
use web_sys::Element;

use leptos_ui_internals::item_equality::compare_item_equality;
use leptos_ui_internals::resolve_value_label::has_null_item_label;
use leptos_ui_internals::types::HTMLProps;
use leptos_ui_utils::react_store::ReactStore;

/// The transition status union — upstream `TransitionStatus` from
/// `internals/useTransitionStatus` (`store.ts:5`), ported as the string values the
/// state attribute mapping writes (`starting` / `ending` / `indeterminate`).
pub type TransitionStatus = String;

/// The popup side union — upstream `Side` from `internals/useAnchorPositioning`
/// (`store.ts:6`): `top`/`right`/`bottom`/`left`.
pub type Side = String;

/// The selection mode union (`store.ts:53`): `'single' | 'multiple' | 'none'`.
pub type SelectionMode = String;

/// The `autoHighlight` union (`store.ts:62`): `false | 'always' | 'input-change'`.
pub type AutoHighlight = String;

/// The opening interaction type — upstream `InteractionType` from
/// `useEnhancedClickHandler` (`store.ts:2`): `'mouse' | 'touch' | 'keyboard'`.
pub type InteractionType = String;

/// The reactive state shape — upstream `State` (`store.ts:10-63`). `Clone` only: the
/// state carries non-comparable handles (element refs, Rc'd accessors, prop bags) —
/// upstream's per-field `Object.is` change detection ports to the field accessor
/// `set_field` writes, whose `V: PartialEq` bounds apply field-by-field.
#[derive(Clone)]
pub struct ComboboxState {
    pub id: Option<String>,
    pub label_id: Option<String>,

    /// The root `items` prop — `undefined` (no items prop at all) ports to `None`;
    /// a present-but-empty list is `Some([])`.
    pub items: Option<Vec<Value>>,

    /// The selected value. A JS `null` selection is a *value* and ports to
    /// `Value::Null`; the multiple-mode shape is an array.
    pub selected_value: Value,

    pub open: bool,
    pub mounted: bool,
    pub transition_status: TransitionStatus,
    pub force_mounted: bool,

    pub inline: bool,

    /// Filtered-list coordinates, not DOM order (`store.ts:14`).
    pub active_index: Option<usize>,
    pub selected_index: Option<usize>,

    pub popup_props: HTMLProps,
    pub list_props: HTMLProps,
    pub input_props: HTMLProps,
    pub trigger_props: HTMLProps,
    pub item_props: HTMLProps,

    pub positioner_element: Option<Element>,
    pub list_element: Option<Element>,
    pub popup_id: Option<String>,
    pub trigger_element: Option<Element>,
    pub input_element: Option<Element>,
    pub input_group_element: Option<Element>,
    pub popup_side: Option<Side>,

    pub open_method: Option<InteractionType>,

    pub input_inside_popup: bool,
    pub input_owns_form_value: bool,

    pub selection_mode: SelectionMode,

    pub name: Option<String>,
    pub form: Option<String>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub grid: bool,
    pub virtualized: bool,
    pub open_on_input_click: bool,
    pub item_to_string_label: Option<Rc<dyn Fn(&Value) -> String>>,
    pub is_item_equal_to_value: Rc<dyn Fn(&Value, &Value) -> bool>,
    pub modal: bool,
    pub auto_highlight: AutoHighlight,
    pub submit_on_item_click: bool,
    pub has_input_value: bool,
}

impl ComboboxState {
    /// The upstream default comparer — `isItemEqualToValue` defaults to
    /// `compareItemEquality`'s `defaultItemEquality` (the `Object.is` semantics) when
    /// the prop is absent; the port carries the function always and seeds this.
    pub fn default_is_item_equal_to_value() -> Rc<dyn Fn(&Value, &Value) -> bool> {
        Rc::new(|a: &Value, b: &Value| compare_item_equality(Some(a), Some(b), |x, y| x == y))
    }
}

/// The non-reactive shared values — upstream `ComboboxStoreContext`
/// (`store.ts:69-120`). Nothing here is observable through the state: writing to a
/// ref never notifies subscribers.
pub struct ComboboxStoreContext {
    /// Item elements in list order, owned by `Combobox.List`.
    pub list_ref: Rc<RefCell<Vec<Option<Element>>>>,
    /// Item text labels in list order, used for typeahead.
    pub labels_ref: Rc<RefCell<Vec<Option<String>>>>,
    /// The popup element.
    pub popup_ref: Rc<RefCell<Option<Element>>>,
    /// The empty-state element.
    pub empty_ref: Rc<RefCell<Option<Element>>>,
    /// The input element that owns the combobox role.
    pub input_ref: Rc<RefCell<Option<Element>>>,
    /// Internal dismiss button rendered before the popup content.
    pub start_dismiss_ref: Rc<RefCell<Option<Element>>>,
    /// Internal dismiss button rendered after the popup content.
    pub end_dismiss_ref: Rc<RefCell<Option<Element>>>,
    /// Whether the last interaction came from the keyboard.
    pub keyboard_active_ref: Rc<Cell<bool>>,
    /// Container holding the selection chips.
    pub chips_container_ref: Rc<RefCell<Option<Element>>>,
    /// The clear button.
    pub clear_ref: Rc<RefCell<Option<Element>>>,
    /// Item values in list order.
    pub values_ref: Rc<RefCell<Vec<Value>>>,
    /// Item element that received the last pointerdown, to pair it with a mouseup.
    pub pointer_down_item_ref: Rc<RefCell<Option<Element>>>,
    /// Native event that triggered the in-flight selection.
    pub selection_event_ref: Rc<RefCell<Option<web_sys::Event>>>,

    // Commands. Seeded with the NOOP and assigned during the root's construction
    // (`AriaCombobox.tsx:516-525`, `:1437-1444`), so parts can call them on first
    // render before the root's callbacks exist.
    /// Opens or closes the popup.
    pub set_open: Rc<dyn Fn(bool, &ChangeCommandDetails)>,
    /// Sets the input value.
    pub set_input_value: Rc<dyn Fn(String, &ChangeCommandDetails)>,
    /// Sets the selected value.
    pub set_selected_value: Rc<dyn Fn(Value, &ChangeCommandDetails)>,
    /// Sets the active and/or selected index.
    pub set_indices: Rc<dyn Fn(SetIndicesInput)>,
    /// Mounts the popup subtree without opening it, to resolve derived item labels.
    pub force_mount: Rc<dyn Fn()>,
    /// Applies a selection originating from an item.
    pub handle_selection: Rc<dyn Fn(&web_sys::Event, Value)>,
    /// Requests submission of the owning form.
    pub request_submit: Rc<dyn Fn()>,
    /// Called when the open state change animation completes.
    pub on_open_change_complete: Rc<dyn Fn(bool)>,
}

/// The details argument the command slots take — the change-event details type the
/// root's mutators construct (`AriaCombobox.ChangeEventDetails`). The command slots
/// only pass it through to the user callbacks, so the port carries the shared
/// `BaseUIChangeEventDetails` shape (the popover `RootOpenChangeEventDetails`
/// precedent) with an empty custom payload and the generic event default.
pub type ChangeCommandDetails =
    leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails<(), web_sys::Event>;

/// The `setIndices` input shape (`store.ts:102-108`): either index optional, plus the
/// highlight reason tag. Upstream `undefined` means "leave unchanged"; the port keeps
/// `None` for that.
#[derive(Default)]
pub struct SetIndicesInput {
    pub active_index: Option<Option<usize>>,
    pub selected_index: Option<Option<usize>>,
    pub reason: Option<String>,
}

impl Default for ComboboxStoreContext {
    fn default() -> Self {
        /// The NOOP command seed (`AriaCombobox.tsx:516-525`).
        fn noop() {}

        Self {
            list_ref: Rc::new(RefCell::new(Vec::new())),
            labels_ref: Rc::new(RefCell::new(Vec::new())),
            popup_ref: Rc::new(RefCell::new(None)),
            empty_ref: Rc::new(RefCell::new(None)),
            input_ref: Rc::new(RefCell::new(None)),
            start_dismiss_ref: Rc::new(RefCell::new(None)),
            end_dismiss_ref: Rc::new(RefCell::new(None)),
            keyboard_active_ref: Rc::new(Cell::new(false)),
            chips_container_ref: Rc::new(RefCell::new(None)),
            clear_ref: Rc::new(RefCell::new(None)),
            values_ref: Rc::new(RefCell::new(Vec::new())),
            pointer_down_item_ref: Rc::new(RefCell::new(None)),
            selection_event_ref: Rc::new(RefCell::new(None)),
            set_open: Rc::new(|_, _| noop()),
            set_input_value: Rc::new(|_, _| noop()),
            set_selected_value: Rc::new(|_, _| noop()),
            set_indices: Rc::new(|_: SetIndicesInput| noop()),
            force_mount: Rc::new(noop),
            handle_selection: Rc::new(|_, _| noop()),
            request_submit: Rc::new(noop),
            on_open_change_complete: Rc::new(|_| noop()),
        }
    }
}

/// The store type — upstream `ComboboxStore = ReactStore<State, ComboboxStoreContext,
/// typeof selectors>` (`store.ts:205`).
pub type ComboboxStore = ReactStore<ComboboxState, ComboboxStoreContext>;

/// The selector table (`store.ts:122-203`), ported as free functions over the state.
pub mod selectors {
    use super::*;

    pub fn id(state: &ComboboxState) -> Option<&String> {
        state.id.as_ref()
    }

    pub fn label_id(state: &ComboboxState) -> Option<&String> {
        state.label_id.as_ref()
    }

    pub fn items(state: &ComboboxState) -> Option<&Vec<Value>> {
        state.items.as_ref()
    }

    pub fn selected_value(state: &ComboboxState) -> &Value {
        &state.selected_value
    }

    /// `hasSelectionChips` (`store.ts:129-132`): the selection is an array with at
    /// least one entry.
    pub fn has_selection_chips(state: &ComboboxState) -> bool {
        state
            .selected_value
            .as_array()
            .is_some_and(|values| !values.is_empty())
    }

    /// `hasSelectedValue` (`store.ts:134-143`): `[]` reads as "no value" in multiple
    /// mode; any non-nullish value counts otherwise.
    pub fn has_selected_value(state: &ComboboxState) -> bool {
        let selected_value = &state.selected_value;
        if selected_value.is_null() {
            return false;
        }
        if state.selection_mode == "multiple" {
            if let Some(values) = selected_value.as_array() {
                return !values.is_empty();
            }
        }
        true
    }

    /// `hasNullItemLabel` (`store.ts:145-147`): gated by the `enabled` flag the caller
    /// passes (the Value part's "has items" gate).
    pub fn has_null_item_label(state: &ComboboxState, enabled: bool) -> bool {
        if enabled {
            leptos_ui_internals::resolve_value_label::has_null_item_label(
                state
                    .items
                    .as_ref()
                    .map(|items| Value::Array(items.clone()))
                    .as_ref(),
            )
        } else {
            false
        }
    }

    pub fn open(state: &ComboboxState) -> bool {
        state.open
    }

    pub fn mounted(state: &ComboboxState) -> bool {
        state.mounted
    }

    pub fn force_mounted(state: &ComboboxState) -> bool {
        state.force_mounted
    }

    pub fn inline(state: &ComboboxState) -> bool {
        state.inline
    }

    pub fn active_index(state: &ComboboxState) -> Option<usize> {
        state.active_index
    }

    pub fn selected_index(state: &ComboboxState) -> Option<usize> {
        state.selected_index
    }

    /// `isActive` (`store.ts:157`).
    pub fn is_active(state: &ComboboxState, index: usize) -> bool {
        state.active_index == Some(index)
    }

    /// `isSelected` (`store.ts:158-167`): fans an array `selectedValue` through
    /// [`compare_item_equality`] with the state's comparer; a single selection
    /// compares directly.
    pub fn is_selected(state: &ComboboxState, item_value: &Value) -> bool {
        let comparer = |a: &Value, b: &Value| (state.is_item_equal_to_value)(a, b);
        let selected_value = &state.selected_value;
        if let Some(values) = selected_value.as_array() {
            return values.iter().any(|selected_item| {
                compare_item_equality(Some(item_value), Some(selected_item), |a, b| comparer(a, b))
            });
        }
        compare_item_equality(Some(item_value), Some(selected_value), |a, b| {
            comparer(a, b)
        })
    }

    pub fn transition_status(state: &ComboboxState) -> &TransitionStatus {
        &state.transition_status
    }

    pub fn popup_props(state: &ComboboxState) -> &HTMLProps {
        &state.popup_props
    }

    pub fn list_props(state: &ComboboxState) -> &HTMLProps {
        &state.list_props
    }

    pub fn input_props(state: &ComboboxState) -> &HTMLProps {
        &state.input_props
    }

    pub fn trigger_props(state: &ComboboxState) -> &HTMLProps {
        &state.trigger_props
    }

    pub fn item_props(state: &ComboboxState) -> &HTMLProps {
        &state.item_props
    }

    pub fn positioner_element(state: &ComboboxState) -> Option<&Element> {
        state.positioner_element.as_ref()
    }

    pub fn list_element(state: &ComboboxState) -> Option<&Element> {
        state.list_element.as_ref()
    }

    pub fn popup_id(state: &ComboboxState) -> Option<&String> {
        state.popup_id.as_ref()
    }

    pub fn trigger_element(state: &ComboboxState) -> Option<&Element> {
        state.trigger_element.as_ref()
    }

    pub fn input_element(state: &ComboboxState) -> Option<&Element> {
        state.input_element.as_ref()
    }

    pub fn input_group_element(state: &ComboboxState) -> Option<&Element> {
        state.input_group_element.as_ref()
    }

    pub fn popup_side(state: &ComboboxState) -> Option<&Side> {
        state.popup_side.as_ref()
    }

    pub fn open_method(state: &ComboboxState) -> Option<&InteractionType> {
        state.open_method.as_ref()
    }

    pub fn input_inside_popup(state: &ComboboxState) -> bool {
        state.input_inside_popup
    }

    pub fn input_owns_form_value(state: &ComboboxState) -> bool {
        state.input_owns_form_value
    }

    pub fn selection_mode(state: &ComboboxState) -> &SelectionMode {
        &state.selection_mode
    }

    pub fn name(state: &ComboboxState) -> Option<&String> {
        state.name.as_ref()
    }

    pub fn form(state: &ComboboxState) -> Option<&String> {
        state.form.as_ref()
    }

    pub fn disabled(state: &ComboboxState) -> bool {
        state.disabled
    }

    pub fn read_only(state: &ComboboxState) -> bool {
        state.read_only
    }

    pub fn required(state: &ComboboxState) -> bool {
        state.required
    }

    pub fn grid(state: &ComboboxState) -> bool {
        state.grid
    }

    pub fn virtualized(state: &ComboboxState) -> bool {
        state.virtualized
    }

    pub fn modal(state: &ComboboxState) -> bool {
        state.modal
    }

    pub fn auto_highlight(state: &ComboboxState) -> &AutoHighlight {
        &state.auto_highlight
    }
}
