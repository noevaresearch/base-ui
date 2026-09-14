//! Combobox value/chips/clear parts runtime — the behavior layer of
//! `Combobox.Value`, `Combobox.Chips`, `Combobox.Chip`, `Combobox.ChipRemove`,
//! and `Combobox.Clear` (`packages/react/src/combobox/{value,chips,chip,chip-remove,clear}`),
//! mined from `specs/library/combobox/parts/value-chips.md` and its upstream test
//! citations.
//!
//! Like the root's mutator layer ([`crate::combobox::root_runtime`]), the behavior is
//! ported as host-testable free functions over the store spine's shapes
//! ([`crate::combobox::store`]): the DOM reads (key names, modifiers, the
//! keyboard-active flag) are plain arguments, the commands the parts issue are
//! returned as plan structs the wasm wiring executes, and the label resolution rides
//! the already-ported `resolve_value_label` internals.

use std::rc::Rc;

use serde_json::Value;

use crate::combobox::parts_util::{get_chip_navigation_keys, get_index_after_chip_removal};
use crate::combobox::root_runtime::stringify_value_label;
use leptos_ui_internals::item_equality::{compare_item_equality, find_item_index};
use leptos_ui_internals::resolve_value_label::{
    has_null_item_label, resolve_multiple_labels, resolve_selected_label,
};

// `REASONS` entries this layer issues (`packages/react/src/internals/reason-parts.ts`)
// that the floating-ui reasons registry does not carry.
/// `REASONS.keyboard`.
pub const REASON_KEYBOARD: &str = "keyboard";
/// `REASONS.pointer`.
pub const REASON_POINTER: &str = "pointer";
/// `REASONS.none`.
pub const REASON_NONE: &str = "none";
/// `REASONS.listNavigation`.
pub const REASON_LIST_NAVIGATION: &str = "list-navigation";
/// `REASONS.clearPress`.
pub const REASON_CLEAR_PRESS: &str = "clear-press";
/// `REASONS.chipRemovePress`.
pub const REASON_CHIP_REMOVE_PRESS: &str = "chip-remove-press";

// ---------------------------------------------------------------------------
// Combobox.Value — the display resolution
// (`packages/react/src/combobox/value/ComboboxValue.tsx:15-38`)
// ---------------------------------------------------------------------------

/// The `children` prop shape the Value part accepts
/// (`ComboboxValue.tsx:15-17`): a render function over the selected value, a
/// static node, or nothing.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueChildren {
    /// No `children` prop.
    None,
    /// A static node — rendered verbatim.
    Node,
    /// A render function — called with the selected value.
    RenderFunction,
}

/// What the Value part renders — the branch taken by the precedence chain
/// (`ComboboxValue.tsx:25-36`).
#[derive(Clone, Debug, PartialEq)]
pub enum ValueDisplay {
    /// `children` was `null` and nothing else resolved — renders nothing
    /// (the `let children = null` seed).
    Nothing,
    /// The static `children` node — wins over everything
    /// (`ComboboxValue.test.tsx:760-802` claim).
    Node,
    /// The render function — the caller invokes it with `selectedValue`.
    RenderFunction,
    /// The placeholder — shown only when there is no selected value, a
    /// placeholder was given, and the items carry no null-value item with a
    /// non-null label (`ComboboxValue.test.tsx:831-856` claim).
    Placeholder,
    /// Multiple mode: the per-value labels folded with `", "` separators
    /// (`ComboboxValue.test.tsx:525-592` claim).
    Multiple(Vec<Value>),
    /// Single mode: the resolved label value.
    Single(Value),
}

/// The Combobox.Value resolution (`ComboboxValue.tsx:17-37`).
///
/// `items` is the store's raw `items` state (the record/array/grouped shapes the
/// resolver handles), `selected_value` the store's `selectedValue`,
/// `item_to_string_label` the store-carried `itemToStringLabel`.
/// `has_null_item_label_raw` is the items-derived fact (the selector's computed
/// value); the `shouldCheckNullItemLabel` gate (`:23-24`) is computed here from
/// the same inputs upstream uses, so callers cannot desynchronize it.
pub fn resolve_value_display(
    children: &ValueChildren,
    placeholder_present: bool,
    has_selected_value: bool,
    has_null_item_label_raw: bool,
    selection_mode: &str,
    selected_value: &Value,
    items: Option<&Value>,
    item_to_string_label: Option<&Rc<dyn Fn(&Value) -> String>>,
) -> ValueDisplay {
    // `shouldCheckNullItemLabel` (`:23`): the hasNullItemLabel selector only runs
    // its scan when nothing else can satisfy the display.
    let should_check_null_item_label =
        !has_selected_value && placeholder_present && *children == ValueChildren::None;
    // `store.useState('hasNullItemLabel', shouldCheckNullItemLabel)` (`:24`): the
    // disabled selector reads as false.
    let has_null_label = should_check_null_item_label && has_null_item_label_raw;

    if *children == ValueChildren::RenderFunction {
        return ValueDisplay::RenderFunction;
    }
    if *children == ValueChildren::Node {
        return ValueDisplay::Node;
    }
    // `!hasSelectedValue && placeholder != null && !hasNullLabel` (`:28`).
    if !has_selected_value && placeholder_present && !has_null_label {
        return ValueDisplay::Placeholder;
    }
    if selection_mode == "multiple" {
        if let Some(values) = selected_value.as_array() {
            return ValueDisplay::Multiple(resolve_multiple_labels(
                values,
                items,
                item_to_string_label.as_ref().map(|f| f.as_ref()),
            ));
        }
    }
    ValueDisplay::Single(resolve_selected_label(
        selected_value,
        items,
        item_to_string_label.as_ref().map(|f| f.as_ref()),
    ))
}

/// The `hasNullItemLabel` selector's items-scan
/// (`packages/react/src/internals/resolveValueLabel.tsx:55-71`), re-exported for
/// the Value part's gate.
pub fn items_have_null_value_label(items: Option<&Value>) -> bool {
    has_null_item_label(items)
}

// ---------------------------------------------------------------------------
// Combobox.Chips — the container
// (`packages/react/src/combobox/chips/ComboboxChips.tsx:38-49`)
// ---------------------------------------------------------------------------

/// The `role` the Chips container carries (`ComboboxChips.tsx:42-44`): none while
/// empty, `toolbar` once at least one chip renders — the NVDA browse-mode rule
/// (`ComboboxChips.test.tsx:17-39` claim).
pub fn chips_role(has_selection_chips: bool) -> Option<&'static str> {
    has_selection_chips.then_some("toolbar")
}

/// The Chips container's highlighted-chip state (`ComboboxChips.tsx:28-35`):
/// cleared whenever the popup opens (`ComboboxChips.test.tsx:277-309` claim —
/// reopening clears the highlighted chip and leaves it unfocused).
#[derive(Default)]
pub struct ChipsRuntime {
    /// The `highlightedChipIndex` React state.
    pub highlighted_chip_index: std::cell::Cell<Option<usize>>,
}

impl ChipsRuntime {
    /// The render-time reset (`ComboboxChips.tsx:31-33`): when the popup is open
    /// and a chip is highlighted, clear it. Returns `true` when the state was
    /// reset (the caller re-renders/focuses nothing — upstream only sets state).
    pub fn sync_open(&self, open: bool) -> bool {
        if open && self.highlighted_chip_index.get().is_some() {
            self.highlighted_chip_index.set(None);
            return true;
        }
        false
    }
}

/// What a keydown on a focused chip does — the `handleKeyDown` outcome
/// (`packages/react/src/combobox/chip/ComboboxChip.tsx:39-96`) plus the wrapper's
/// focus/handlers, as a plan the caller executes.
#[derive(Clone, Debug, PartialEq)]
pub struct ChipKeyDownPlan {
    /// The next highlighted chip index, or `None` for upstream `undefined` — the
    /// caller focuses the input (`ComboboxChip.tsx:98-104`).
    pub next_index: Option<usize>,
    /// Whether the event is stopped (`stopEvent` / `preventDefault`).
    pub stop_event: bool,
    /// Whether `setOpen(true, listNavigation)` fires.
    pub open_popup: bool,
    /// Whether the chip's own value is removed (Backspace/Delete).
    pub remove_chip: bool,
}

/// The chip keydown logic (`ComboboxChip.tsx:39-96`). `index` is the chip's own
/// index, `chips_len` the number of *rendered* chips (`chipsRef.current.length` —
/// navigation is bounded by rendered chips, `ComboboxChip.test.tsx:452-492`),
/// `selected_len` the selected array's length, `direction` the
/// `ltr`/`rtl` direction.
pub fn plan_chip_key_down(
    key: &str,
    ctrl: bool,
    meta: bool,
    alt: bool,
    index: usize,
    chips_len: usize,
    selected_len: usize,
    direction: &str,
) -> ChipKeyDownPlan {
    let (previous_chip_key, next_chip_key) = get_chip_navigation_keys(direction);
    // Upstream seeds `nextIndex = index` and only navigation sets it back to a
    // number; `undefined` always ends in focusing the input.
    let mut next_index: Option<usize> = Some(index);
    let mut plan = ChipKeyDownPlan {
        next_index,
        stop_event: false,
        open_popup: false,
        remove_chip: false,
    };

    if key == previous_chip_key {
        // `ComboboxChip.tsx:44-51`.
        plan.stop_event = true;
        next_index = if index > 0 { Some(index - 1) } else { None };
    } else if key == next_chip_key {
        // `ComboboxChip.tsx:52-59`.
        plan.stop_event = true;
        next_index = if index < chips_len.saturating_sub(1) {
            Some(index + 1)
        } else {
            None
        };
    } else if key == "Backspace" || key == "Delete" {
        // `ComboboxChip.tsx:60-77`: remove the chip's value and clear both
        // indices with the keyboard reason.
        next_index = get_index_after_chip_removal(index, selected_len);
        plan.stop_event = true;
        plan.remove_chip = true;
    } else if key == "Enter" || key == " " {
        // `ComboboxChip.tsx:78-81`.
        plan.stop_event = true;
        next_index = None;
    } else if key == "ArrowDown" || key == "ArrowUp" {
        // `ComboboxChip.tsx:82-88`: hand focus to the input by opening the popup.
        plan.stop_event = true;
        plan.open_popup = true;
        next_index = None;
    } else if key.chars().count() == 1 && !ctrl && !meta && !alt {
        // `ComboboxChip.tsx:89-96`: plain printable characters hand focus to the
        // input; modified printables intentionally leave focus on the chip
        // (`ComboboxChip.test.tsx:365-386`) — the fall-through keeps
        // `nextIndex = index`.
        next_index = None;
    }

    plan.next_index = next_index;
    plan
}

/// The disabled/readOnly gate on the chip's `onKeyDown`
/// (`ComboboxChip.tsx:109-112`) — focus stays on the chip, no removal, no
/// navigation (`ComboboxChip.test.tsx:36-90`, `:171-196`).
pub fn chip_key_down_blocked(disabled: bool, read_only: bool) -> bool {
    disabled || read_only
}

// ---------------------------------------------------------------------------
// Combobox.ChipRemove — the removal plan
// (`packages/react/src/combobox/chip-remove/ComboboxChipRemove.tsx:64-118`)
// ---------------------------------------------------------------------------

/// What a ChipRemove activation (click, Enter, or Space) does — the
/// `removeChip`/`clearActiveIndexForRemovedItem` outcome as a plan
/// (`ComboboxChipRemove.tsx:64-118`).
#[derive(Clone, Debug, PartialEq)]
pub struct ChipRemovePlan {
    /// Whether the active-index clear command fires (`:64-85`): only when the
    /// removed item is the currently active one in the visible list.
    pub clear_active_index: bool,
    /// The indices-clear reason (`:80-83`): keyboard vs pointer per the
    /// `keyboardActiveRef`.
    pub active_index_reason: &'static str,
    /// The new selected value — the array minus the removed chip's entry
    /// (`:88-91`).
    pub next_selected_value: Value,
    /// Whether the originating event's propagation is blocked (`:111-114`):
    /// upstream blocks unless the user's `onValueChange` called
    /// `details.allowPropagation()` — the port carries the details-driven
    /// decision as this flag's input (see [`plan_chip_remove_propagation`]).
    pub propagation_blocked_by_default: bool,
}

/// The ChipRemove activation logic. `index` is the chip's index into the
/// selected array, `selected_value` the store's array-shaped `selectedValue`,
/// `active_index` the store's `activeIndex`, `values` the `valuesRef.current`
/// list, `is_item_equal_to_value` the store-carried comparer,
/// `keyboard_active` the `keyboardActiveRef` flag.
pub fn plan_chip_remove(
    index: usize,
    selected_value: &[Value],
    active_index: Option<usize>,
    values: &[Value],
    is_item_equal_to_value: &dyn Fn(&Value, &Value) -> bool,
    keyboard_active: bool,
) -> ChipRemovePlan {
    // `clearActiveIndexForRemovedItem` (`:64-85`).
    let clear_active_index = match active_index {
        None => false,
        Some(active_index) => {
            let removed_item = selected_value.get(index);
            // `findItemIndex` over the visible list; a miss (filtered out) means
            // no clear — the highlight can't equal the removed index.
            let removed_index = values.iter().position(|value| {
                removed_item.is_some_and(|removed_item| {
                    compare_item_equality(Some(value), Some(removed_item), is_item_equal_to_value)
                })
            });
            removed_index.is_some_and(|removed_index| removed_index == active_index)
        }
    };

    ChipRemovePlan {
        clear_active_index,
        active_index_reason: if keyboard_active {
            REASON_KEYBOARD
        } else {
            REASON_POINTER
        },
        next_selected_value: Value::Array(
            selected_value
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != index)
                .map(|(_, value)| value.clone())
                .collect(),
        ),
        // Upstream default: a ChipRemove-triggered change blocks propagation of
        // the originating click/keydown to the parent Chip
        // (`ComboboxChipRemove.test.tsx:283-335`).
        propagation_blocked_by_default: true,
    }
}

/// The propagation decision (`ComboboxChipRemove.tsx:111-114`): block unless the
/// user's handler allowed it.
pub fn plan_chip_remove_propagation(propagation_allowed: bool) -> bool {
    !propagation_allowed
}

// ---------------------------------------------------------------------------
// Combobox.Clear — visibility and the click plan
// (`packages/react/src/combobox/clear/ComboboxClear.tsx:41-118`)
// ---------------------------------------------------------------------------

/// The `visible` derivation (`ComboboxClear.tsx:47-55`): per selection mode —
/// typed input in `none` mode, any selection in `single` mode, any chip in
/// `multiple` mode.
pub fn clear_visible(
    selection_mode: &str,
    input_value: &str,
    selected_value: &Value,
    has_selection_chips: bool,
) -> bool {
    if selection_mode == "none" {
        !input_value.is_empty()
    } else if selection_mode == "single" {
        !selected_value.is_null()
    } else {
        has_selection_chips
    }
}

/// The mount/keepMounted rendering gate (`ComboboxClear.tsx:119-123`): render
/// when `keepMounted` or the transition machinery has the clear mounted.
pub fn clear_should_render(keep_mounted: bool, mounted: bool) -> bool {
    keep_mounted || mounted
}

/// What a Clear click does — the `onClick` body as a plan
/// (`ComboboxClear.tsx:97-118`).
#[derive(Clone, Debug, PartialEq)]
pub struct ClearClickPlan {
    /// The `disabled || readOnly` gate blocked the click (`:98-100`) — nothing
    /// else fires (`ComboboxClear.test.tsx:180-207`).
    pub blocked: bool,
    /// The input-value command (`:103-106`): always the empty string.
    pub input_value: Option<String>,
    /// The selected-value command (`:107-110`): `[]` from an array selection,
    /// `null` otherwise; `None` in `none` mode.
    pub next_selected_value: Option<Value>,
    /// Whether the indices command also clears the selected index
    /// (`:111-114` — `selectionMode !== 'none'`) versus the active index only.
    pub clear_selected_index: bool,
    /// The indices-clear reason (`:101-102`): keyboard vs pointer per the
    /// `keyboardActiveRef`.
    pub indices_reason: &'static str,
    /// The trailing input focus (`:117`).
    pub focus_input: bool,
}

/// The Clear click logic. `keyboard_active` is the `keyboardActiveRef` flag.
pub fn plan_clear_click(
    disabled: bool,
    read_only: bool,
    selection_mode: &str,
    selected_value: &Value,
    keyboard_active: bool,
) -> ClearClickPlan {
    if disabled || read_only {
        return ClearClickPlan {
            blocked: true,
            input_value: None,
            next_selected_value: None,
            clear_selected_index: false,
            indices_reason: REASON_POINTER,
            focus_input: false,
        };
    }

    let clear_selected_index = selection_mode != "none";
    let next_selected_value = if clear_selected_index {
        Some(if selected_value.is_array() {
            Value::Array(Vec::new())
        } else {
            Value::Null
        })
    } else {
        None
    };

    ClearClickPlan {
        blocked: false,
        input_value: Some(String::new()),
        next_selected_value,
        clear_selected_index,
        indices_reason: if keyboard_active {
            REASON_KEYBOARD
        } else {
            REASON_POINTER
        },
        focus_input: true,
    }
}

/// The clear button's `tabIndex`/focus-stealing guard
/// (`ComboboxClear.tsx:87-95`): the port carries the constant and the
/// mouse-down `preventDefault` as data the wiring applies.
pub const CLEAR_TAB_INDEX: i32 = -1;

/// Convenience: the label of a selected value under the store-carried
/// stringifier — the single-mode `Single` arm's string form, re-exported from
/// the root runtime's `stringifyValueLabel` for callers rendering the display.
pub fn display_label_string(
    value: &Value,
    item_to_string_label: Option<&Rc<dyn Fn(&Value) -> String>>,
) -> String {
    stringify_value_label(value, item_to_string_label)
}

/// Convenience: the default comparer the store seeds
/// (`ComboboxState::default_is_item_equal_to_value`), re-exported so callers
/// build [`plan_chip_remove`]'s comparer argument from the same source.
pub fn default_comparer() -> Rc<dyn Fn(&Value, &Value) -> bool> {
    crate::combobox::store::ComboboxState::default_is_item_equal_to_value()
}

/// The `find_item_index` internals helper, re-exported with the crate's
/// non-optional item shape for callers that want the index directly.
pub fn find_item_index_values(values: &[Value], needle: &Value) -> Option<usize> {
    let wrapped: Vec<Option<Value>> = values.iter().cloned().map(Some).collect();
    find_item_index::<Value>(Some(&wrapped), Some(needle), |a, b| a == b)
}
