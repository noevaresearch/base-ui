//! Combobox Input behavior layer — the port of
//! `packages/react/src/combobox/input/ComboboxInput.tsx` (the part's computed
//! state and event-handler logic, as host-testable plan functions over the
//! store spine's shapes, per the value-chips batch's convention).
//!
//! The DOM reads (key names, modifiers, caret/selection state, the
//! chip-highlight context, the composing flag) are plain arguments; the
//! commands the part issues are returned as plan structs the wasm wiring
//! executes against the already-ported root runtime
//! ([`crate::combobox::root_runtime`]) and parts utils
//! ([`crate::combobox::parts_util`]).
//!
//! Upstream reference: `ComboboxInput.tsx` — the read-side state composition
//! (:51-102), the `setInputElement` ref callback (:104-117), `clearHighlight`
//! (:119-124), the chip navigation matrix (`handleKeyDown`, :150-200), the
//! composition handlers (:304-322), the `onChange` orchestration (:323-379),
//! and the `onKeyDown` matrix (:361-457).

use serde_json::Value;

use crate::combobox::parts_util::{get_chip_navigation_keys, get_index_after_chip_removal};
use crate::combobox::store::ComboboxStore;

// `REASONS` entries this layer issues (`packages/react/src/internals/reasons.ts`)
// beyond the ones the root runtime carries.
/// `REASONS.escapeKey`.
pub const REASON_ESCAPE_KEY: &str = "escape-key";
/// `REASONS.inputPress`.
pub const REASON_INPUT_PRESS: &str = "input-press";
/// `REASONS.none`.
pub use crate::combobox::value_chips::{REASON_KEYBOARD, REASON_NONE, REASON_POINTER};

// ---------------------------------------------------------------------------
// The read-side state composition (ComboboxInput.tsx:51-102)
// ---------------------------------------------------------------------------

/// `isInsidePopup` (`:92`): `hasPositionerParent || inline` — the input mounts
/// inside the popup positioner or the combobox is inline.
pub fn is_inside_popup(has_positioner_parent: bool, inline: bool) -> bool {
    has_positioner_parent || inline
}

/// `focusManagerModal` (`:93`): `!isInsidePopup || modal` — a standalone input
/// always gets the modal focus manager; an inside-popup input only when the
/// root is modal.
pub fn focus_manager_modal(is_inside_popup: bool, modal: bool) -> bool {
    !is_inside_popup || modal
}

/// The `disabled` fold (`:94`): `fieldDisabled || comboboxDisabled || disabledProp`.
pub fn input_disabled(field: bool, combobox: bool, prop: bool) -> bool {
    field || combobox || prop
}

/// `inputOwnsFormValue` (`:102`): `selectionMode === 'none' && !hasPositionerParent`
/// — only a standalone `none`-mode input carries the hidden form value.
pub fn input_owns_form_value(selection_mode: &str, has_positioner_parent: bool) -> bool {
    selection_mode == "none" && !has_positioner_parent
}

/// The `id` resolution outcome (`:95`):
/// `useBaseUiId(idProp ?? (!isInsidePopup ? rootId : undefined))`.
#[derive(Clone, Debug, PartialEq)]
pub enum InputId {
    /// A concrete id — the prop, or the root id when the input is standalone.
    Resolved(String),
    /// Nothing supplied — `useBaseUiId` generates one at wiring time.
    Generated,
}

pub fn resolve_input_id(
    id_prop: Option<String>,
    is_inside_popup: bool,
    root_id: Option<String>,
) -> InputId {
    if let Some(id) = id_prop {
        return InputId::Resolved(id);
    }
    if !is_inside_popup {
        if let Some(id) = root_id {
            return InputId::Resolved(id);
        }
    }
    InputId::Generated
}

/// `fieldStateForInput` (`:96`): inside a positioner the Field state is reset
/// to `DEFAULT_FIELD_STATE_ATTRIBUTES` (the provider is swapped at :451-455),
/// so the state attributes fold only the standalone Field state.
pub fn field_state_for_input(
    has_positioner_parent: bool,
    standalone: &FieldStateFold,
) -> FieldStateFold {
    if has_positioner_parent {
        FieldStateFold::default()
    } else {
        standalone.clone()
    }
}

/// The Field-state attributes the input state spreads
/// (`ComboboxInput.tsx:231-238`, the `...fieldStateForInput` spread).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FieldStateFold {
    /// `data-touched` from the Field state.
    pub touched: bool,
    /// `data-valid` from the Field state — `None` when not yet validated.
    pub valid: Option<bool>,
}

// ---------------------------------------------------------------------------
// The setInputElement ref callback (ComboboxInput.tsx:104-117)
// ---------------------------------------------------------------------------

/// The ref callback's plan: which store writes the wiring executes.
#[derive(Clone, Debug, PartialEq)]
pub struct SetInputElementPlan {
    /// `store.context.setInputValue('', createChangeEventDetails(REASONS.none))`
    /// — when the input mounts inside the popup and no input value is held.
    pub reset_input_value: bool,
    /// The `inputInsidePopup` value written with the element.
    pub input_inside_popup: bool,
}

pub fn plan_set_input_element(
    store: &ComboboxStore,
    has_positioner_parent: bool,
) -> SetInputElementPlan {
    let next_is_inside_popup = has_positioner_parent || store.select(|state| state.inline);
    SetInputElementPlan {
        reset_input_value: next_is_inside_popup && !store.select(|state| state.has_input_value),
        input_inside_popup: next_is_inside_popup,
    }
}

// ---------------------------------------------------------------------------
// clearHighlight (ComboboxInput.tsx:119-124)
// ---------------------------------------------------------------------------

/// The `setIndices` command `clearHighlight` issues: both indices nulled with
/// the keyboard-vs-pointer reason read from `keyboardActiveRef` at the call
/// site (`:121-123`).
#[derive(Clone, Debug, PartialEq)]
pub struct ClearHighlightPlan {
    pub active_index: Option<usize>,
    pub selected_index: Option<usize>,
    pub reason: &'static str,
}

pub fn plan_clear_highlight(keyboard_active: bool) -> ClearHighlightPlan {
    ClearHighlightPlan {
        active_index: None,
        selected_index: None,
        reason: if keyboard_active {
            REASON_KEYBOARD
        } else {
            REASON_POINTER
        },
    }
}

// ---------------------------------------------------------------------------
// The chip navigation matrix (handleKeyDown, ComboboxInput.tsx:150-200)
// ---------------------------------------------------------------------------

/// The chip-navigation outcome of `handleKeyDown` (`:150-200`) — the next chip
/// index (or `undefined`), mirroring the function's return.
#[derive(Clone, Debug, PartialEq)]
pub enum ChipNav {
    /// No chips context — `handleKeyDown` returns `undefined` immediately
    /// (`:151-154`) without consuming the event.
    NoChipsContext,
    /// A returned index: `Some(i)` navigates to chip `i`; `None` is the
    /// upstream `undefined` return (drop the highlight).
    Index(Option<usize>),
}

/// `handleKeyDown` (`ComboboxInput.tsx:150-200`): the chip-highlight arrow
/// walk with the at-boundary drop to `undefined`, the Backspace/Delete removal
/// walk, and the unhighlighted previous-key walk gated on
/// `selectionStart ?? 0 === 0` and a non-empty selection.
///
/// `direction` is the DOM `direction` context; `selection_start` is the
/// `event.currentTarget.selectionStart` read (`null` reads as `0`, the
/// `:187` claim); `selected_len` is `selectedValue.length` (multiple mode).
pub fn plan_chip_navigation(
    key: &str,
    direction: &str,
    has_chips_context: bool,
    highlighted_chip_index: Option<usize>,
    rendered_chips_count: usize,
    selected_len: usize,
    selection_start: Option<usize>,
) -> ChipNav {
    if !has_chips_context {
        return ChipNav::NoChipsContext;
    }

    let (previous_chip_key, next_chip_key) = get_chip_navigation_keys(direction);

    if let Some(highlighted) = highlighted_chip_index {
        if key == previous_chip_key {
            // `:157-164`: preventDefault always; at index 0 drop the highlight.
            return ChipNav::Index(if highlighted > 0 {
                Some(highlighted - 1)
            } else {
                None
            });
        }
        if key == next_chip_key {
            // `:165-172`: preventDefault always; past the last chip drop it.
            return ChipNav::Index(if highlighted + 1 < rendered_chips_count {
                Some(highlighted + 1)
            } else {
                None
            });
        }
        if key == "Backspace" || key == "Delete" {
            // `:173-178`: removal moves the highlight via
            // getIndexAfterChipRemoval (the highlight-clear itself happens in
            // the caller — `clearHighlight()` at :178).
            return ChipNav::Index(get_index_after_chip_removal(highlighted, selected_len));
        }
        return ChipNav::Index(None);
    }

    // Navigation when no chip is highlighted (`:183-189`): the previous-chip
    // key with the caret at the start and a non-empty selection focuses the
    // last rendered chip.
    if key == previous_chip_key && selection_start.unwrap_or(0) == 0 && selected_len > 0 {
        return ChipNav::Index(if rendered_chips_count > 0 {
            Some(rendered_chips_count - 1)
        } else {
            None
        });
    }

    ChipNav::Index(None)
}

// ---------------------------------------------------------------------------
// The onKeyDown matrix (ComboboxInput.tsx:361-457)
// ---------------------------------------------------------------------------

/// The keydown inputs the DOM wiring reads (`event.key`, the modifier flags,
/// the caret/selection state, the store snapshot) — everything the plan needs
/// so the host tests stay DOM-free.
#[derive(Clone, Debug)]
pub struct InputKeyDownInput<'a> {
    pub key: &'a str,
    /// `event.which === 229` — the IME composition keydown (`:445-447`).
    pub ime_composing: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    /// The resolved `disabled` (`:94` fold).
    pub disabled: bool,
    pub read_only: bool,
    /// `store.state.open` / `mounted`.
    pub open: bool,
    pub mounted: bool,
    /// `store.state.inline`.
    pub inline: bool,
    /// The selection mode (`'none' | 'single' | 'multiple'`).
    pub selection_mode: &'a str,
    /// `store.state.selectedValue` (for the Escape clear and its `isClear`).
    pub selected_value: &'a Value,
    /// `input.value === ''` — the DOM read (`:416`, `:433`).
    pub input_value_empty: bool,
    /// Whether the chips context exists (multiple mode with a Chips part).
    pub has_chips_context: bool,
    /// `comboboxChipsContext.highlightedChipIndex`.
    pub highlighted_chip_index: Option<usize>,
    /// `comboboxChipsContext.chipsRef.current.length`.
    pub rendered_chips_count: usize,
    /// The direction context (`getChipNavigationKeys`).
    pub direction: &'a str,
    /// `event.currentTarget.selectionStart` (the unhighlighted-chip walk).
    pub selection_start: Option<usize>,
    /// Whether `details.isPropagationAllowed` held on the Escape path — the
    /// floating layer's propagation decision the plan receives.
    pub escape_propagation_allowed: bool,
    /// `platform.engine.gecko` — the Home/End caret cursor swap (`:390-401`).
    pub gecko: bool,
    /// The direction is rtl (`:389`).
    pub rtl: bool,
    /// `input.value.length` — the Home/End cursor math.
    pub input_value_len: usize,
    /// `store.state.activeIndex`.
    pub active_index: Option<usize>,
}

/// The command the onKeyDown wiring executes — the branch taken through the
/// upstream matrix, in its evaluation order (`:361-457`).
#[derive(Clone, Debug, PartialEq)]
pub enum KeyDownPlan {
    /// The modifier guard bailed (`:362-364`): nothing happens — the browser
    /// keeps the event (modified navigation stays on the input,
    /// `:705-727` test claim).
    ModifiedPassthrough,
    /// `readOnly` browsing Enter with an open popup and a highlight must not
    /// submit the form (`:369-374`): the wiring stops the event.
    ReadOnlyEnterStop,
    /// The disabled/readOnly early return (`:371-374`) without the Enter case.
    Inert,
    /// Home/End caret management (`:382-405`): the wiring stops the event and
    /// applies `setSelectionRange(cursor, cursor)` + `scrollLeft`.
    Caret {
        /// The `setSelectionRange` position.
        cursor: usize,
        /// The `scrollLeft` write (`0` for Home; the scroll amount for End,
        /// negated in rtl).
        scroll_left: isize,
    },
    /// Escape on a closed popup (`:406-429`): clear the input value and the
    /// selection (`null` single / `[]` multiple), stopping propagation unless
    /// the value was already clear, the combobox is inline, or the details
    /// allow propagation.
    EscapeClosed { stop_propagation: bool },
    /// Backspace on an empty input with no highlighted chip and a non-empty
    /// multiple selection (`:432-455`): remove the last rendered chip — or
    /// the last selected value when no chips render — and clear the
    /// highlight.
    ChipRemoveLast { removal_index: usize },
    /// The chip-navigation branch ran (`:457-467`): focus moves to the next
    /// chip (when some index resolved) or back to the input (when a chip had
    /// the highlight and the walk dropped it).
    ChipNav {
        next_index: Option<usize>,
        restore_input_focus: bool,
    },
    /// The IME guard (`:469-471`): `event.which === 229` bails without
    /// selection (`:728-750` test claim — no item selected for an IME keydown
    /// without a highlight).
    ImeGuard,
    /// Enter with an open popup and no highlight (`:473-485`): an inline
    /// combobox does nothing; otherwise the popup closes with reason `none`
    /// (form submission stays allowed).
    EnterNoHighlightClose,
    /// Enter with an open popup and a highlight (`:487-490`): the wiring
    /// stops the event and `clickHighlightedItem(store, activeIndex)`
    /// selects.
    EnterHighlightClick { active_index: usize },
    /// Nothing matched — the browser keeps the event.
    Passthrough,
}

/// The Escape-on-closed plan (`:406-429`): `isClear` decides propagation — a
/// value that was already empty lets the event escape (`:785-834` test
/// claims: stop when a value clears, propagate when already empty or inline).
fn escape_closed_plan(input: &InputKeyDownInput) -> KeyDownPlan {
    let is_clear = if input.selection_mode == "multiple" {
        input
            .selected_value
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(false)
    } else {
        input.selected_value.is_null()
    };
    let stop_propagation = !is_clear && !input.inline && !input.escape_propagation_allowed;
    KeyDownPlan::EscapeClosed { stop_propagation }
}

/// The full onKeyDown matrix (`ComboboxInput.tsx:361-457`), in evaluation
/// order. `selected_len` (the multiple selection's length) rides
/// `input.selected_value`'s array.
pub fn plan_key_down(input: &InputKeyDownInput) -> KeyDownPlan {
    // Modifier guard (`:362-364`) — before the keyboard-active write.
    if input.ctrl || input.shift || input.alt || input.meta {
        return KeyDownPlan::ModifiedPassthrough;
    }

    // The keyboard-active write happens before the disabled/readOnly guards
    // so `readOnly` browsing reports keyboard highlight reasons (`:366-368`
    // comment). The wiring executes `keyboardActiveRef.current = true` when
    // the plan is not `ModifiedPassthrough`.

    if input.disabled || input.read_only {
        // Browsing can highlight an item; Enter there must not submit (`:369-374`).
        if input.read_only && input.key == "Enter" && input.open && input.active_index.is_some() {
            return KeyDownPlan::ReadOnlyEnterStop;
        }
        return KeyDownPlan::Inert;
    }

    // Home/End caret management (`:382-405`).
    if input.key == "Home" {
        let cursor = if input.gecko && input.rtl {
            input.input_value_len
        } else {
            0
        };
        return KeyDownPlan::Caret {
            cursor,
            scroll_left: 0,
        };
    }
    if input.key == "End" {
        let cursor = if input.gecko && input.rtl {
            0
        } else {
            input.input_value_len
        };
        let scroll_left = if input.rtl {
            -(input.input_value_len as isize)
        } else {
            input.input_value_len as isize
        };
        return KeyDownPlan::Caret {
            cursor,
            scroll_left,
        };
    }

    // Escape on a closed popup (`:406-429`).
    if !input.mounted && input.key == "Escape" {
        return escape_closed_plan(input);
    }

    // Backspace on an empty input with no highlighted chip (`:432-455`).
    let selected_len = input
        .selected_value
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    if input.has_chips_context
        && input.key == "Backspace"
        && input.input_value_empty
        && input.highlighted_chip_index.is_none()
        && input.selection_mode == "multiple"
        && selected_len > 0
    {
        let removal_index = if input.rendered_chips_count > 0 {
            input.rendered_chips_count - 1
        } else {
            selected_len - 1
        };
        return KeyDownPlan::ChipRemoveLast { removal_index };
    }

    // The chip navigation walk (`:457-467`) — with the input-focus restore
    // when a highlighted chip existed and the walk dropped it.
    let had_highlighted_chip = input.highlighted_chip_index.is_some();
    if input.has_chips_context {
        match plan_chip_navigation(
            input.key,
            input.direction,
            input.has_chips_context,
            input.highlighted_chip_index,
            input.rendered_chips_count,
            selected_len,
            input.selection_start,
        ) {
            ChipNav::Index(next_index) => {
                if next_index.is_some() {
                    return KeyDownPlan::ChipNav {
                        next_index,
                        restore_input_focus: false,
                    };
                }
                if had_highlighted_chip {
                    return KeyDownPlan::ChipNav {
                        next_index: None,
                        restore_input_focus: true,
                    };
                }
                // No chip had the highlight and none resolves — fall through
                // to the remaining branches with the highlight state cleared
                // (upstream `setHighlightedChipIndex(undefined)` is a no-op
                // write; the flow continues at the IME guard).
            }
            ChipNav::NoChipsContext => unreachable!("has_chips_context is true"),
        }
    }

    // IME guard (`:469-471`).
    if input.ime_composing {
        return KeyDownPlan::ImeGuard;
    }

    // Enter with an open popup (`:473-490`).
    if input.key == "Enter" && input.open {
        match input.active_index {
            None => {
                if input.inline {
                    return KeyDownPlan::Passthrough;
                }
                return KeyDownPlan::EnterNoHighlightClose;
            }
            Some(active_index) => {
                return KeyDownPlan::EnterHighlightClick { active_index };
            }
        }
    }

    KeyDownPlan::Passthrough
}

// ---------------------------------------------------------------------------
// The onChange orchestration (ComboboxInput.tsx:323-379)
// ---------------------------------------------------------------------------

/// The typed-input classification the `onChange` handler performs
/// (`:331-333`): autofill may omit `inputType` (Chrome) or report
/// `insertReplacementText` (Firefox); during composition the input is always
/// considered typed into.
pub fn should_open_on_input(input_type: Option<&str>, is_composing: bool) -> bool {
    let autofill_like_input = match input_type {
        None => true,
        Some(t) => t == "insertReplacementText",
    };
    is_composing || !autofill_like_input
}

/// The empty-input clear plan (`:350-364`): an emptied input outside the
/// popup clears a single-mode selection and — unless the popup was opened by
/// an input click — closes the popup with reason `input-clear`.
#[derive(Clone, Debug, PartialEq)]
pub struct EmptyClearPlan {
    /// `setSelectedValue(null, clearDetails)` — single mode only.
    pub clear_selection: bool,
    /// `setOpen(false, clearDetails)`.
    pub close: bool,
}

pub fn plan_empty_clear(
    selection_mode: &str,
    input_inside_popup: bool,
    open_on_input_click: bool,
) -> EmptyClearPlan {
    if input_inside_popup {
        return EmptyClearPlan {
            clear_selection: false,
            close: false,
        };
    }
    EmptyClearPlan {
        clear_selection: selection_mode == "single",
        close: !open_on_input_click,
    }
}

/// The composing-empty close plan (`:340-349`): during composition an emptied
/// input closes the popup with reason `input-clear` when input clicks do not
/// open the popup and the input is not inside it (`:835-869` test claim).
pub fn plan_composing_empty_close(
    input_value_empty: bool,
    open_on_input_click: bool,
    input_inside_popup: bool,
) -> bool {
    input_value_empty && !open_on_input_click && !input_inside_popup
}

/// Whether the highlight clears on this change while the popup is open
/// (`:353-357` composing branch, `:366-369` typed branch): the highlight
/// resets so virtual focus returns to the input — unless autoHighlight is
/// enabled and the typed text is non-empty (`:344-346` the
/// `shouldMaintainHighlight` gate, composing branch).
///
/// `auto_highlight_enabled` is `Boolean(autoHighlightMode)`; `trimmed_empty`
/// is whether the new value trims to `''`.
pub fn should_clear_highlight_on_change(
    open: bool,
    active_index: Option<usize>,
    auto_highlight_enabled: bool,
    trimmed_empty: bool,
    is_composing: bool,
) -> bool {
    if !open || active_index.is_none() {
        return false;
    }
    if is_composing {
        // `:347-349`: `!shouldMaintainHighlight` — autoHighlight keeps the
        // highlight while a query stands.
        return !(auto_highlight_enabled && !trimmed_empty);
    }
    !auto_highlight_enabled
}

// ---------------------------------------------------------------------------
// The composition handlers (ComboboxInput.tsx:304-322)
// ---------------------------------------------------------------------------

/// `onCompositionStart` (`:305-311`): Android returns early (`:306-308` — its
/// keyboards treat all text as always-composing, mui/base-ui#2942); the
/// composing flag flips and the composing value mirrors the element's.
pub fn composition_start(android: bool) -> bool {
    !android
}

// ---------------------------------------------------------------------------
// The focus/blur handlers (ComboboxInput.tsx:254-303)
// ---------------------------------------------------------------------------

/// `onFocus` (`:257-276`): the inline highlight restore — only when the
/// restore flag was armed by the blur handler, the stashed index survives,
/// and the values slot still exists (`valuesRef` is sparse,
/// `Object.hasOwn` guard, `:264-268`; the removed-slot test claim
/// `:751-784`).
pub fn plan_focus_restore(
    should_restore: bool,
    last_active_index: Option<usize>,
    values_len: usize,
) -> Option<usize> {
    if !should_restore {
        return None;
    }
    let index = last_active_index?;
    // `Object.hasOwn(valuesRef.current, nextActiveIndex)` — the slot must exist.
    if index >= values_len {
        return None;
    }
    Some(index)
}

/// `onBlur` (`:277-290`): the inline highlight stash — when the combobox is
/// inline, a highlight stands, and autoHighlight is not `'always'`, the index
/// is stashed for the focus restore and the highlight clears.
#[derive(Clone, Debug, PartialEq)]
pub struct BlurPlan {
    /// The index stashed into `lastActiveIndexRef` (and the restore flag armed).
    pub stash_index: Option<usize>,
    /// `store.context.setIndices({ activeIndex: null })`.
    pub clear_highlight: bool,
}

pub fn plan_blur(
    inline: bool,
    active_index: Option<usize>,
    auto_highlight_always: bool,
) -> BlurPlan {
    if inline && active_index.is_some() && !auto_highlight_always {
        BlurPlan {
            stash_index: active_index,
            clear_highlight: true,
        }
    } else {
        BlurPlan {
            stash_index: None,
            clear_highlight: false,
        }
    }
}

// ---------------------------------------------------------------------------
// The rendered output (ComboboxInput.tsx:443-457, 458-476)
// ---------------------------------------------------------------------------

/// Whether the internal dismiss button renders before the input
/// (`:446-448`): `open && focusManagerModal` — the `<ComboboxInternalDismissButton>`
/// wired to `store.context.startDismissRef`.
pub fn shows_dismiss_button(open: bool, focus_manager_modal: bool) -> bool {
    open && focus_manager_modal
}
