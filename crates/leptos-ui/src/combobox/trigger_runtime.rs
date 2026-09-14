//! Combobox Trigger behavior layer — the port of
//! `packages/react/src/combobox/trigger/ComboboxTrigger.tsx` (the part's
//! computed state, aria wiring, and event-handler logic, as host-testable
//! plan functions over the store spine's shapes, per the input-runtime
//! batch's convention).
//!
//! The DOM reads (pointer types, event targets, `relatedTarget` containment,
//! client coordinates) are plain arguments; the commands the part issues are
//! returned as plan structs the wasm wiring executes against the
//! already-ported root runtime ([`crate::combobox::root_runtime`]) and parts
//! utils ([`crate::combobox::parts_util`]).
//!
//! Upstream reference: `ComboboxTrigger.tsx` — the read-side state composition
//! and disabled fold (:39-88), the `aria-controls` resolution (:90-98), the
//! pointer-type tracking (:100-104), the typeahead gate and match commit
//! (:106-119), the click hook's gate (:121-124), the state object
//! (:131-139), the `onFocus`/`onBlur` handlers (:166-188), the `onMouseDown`
//! handler with the drag-selection mouseup pairing (:189-243), the
//! `onKeyDown` opener (:244-253), and the state-attributes mapping
//! (`utils/stateAttributesMapping.ts:10-23`).

use serde_json::{Map, Value};
use web_sys::Element;

use crate::combobox::root_utils::get_combobox_popup_id;
use crate::combobox::store::ComboboxStore;

// `REASONS` entries this layer issues beyond the ones the root runtime
// carries (`packages/react/src/internals/reason-parts.ts:34` — defined at the
// consuming layer per the root runtime's placement note, until the
// `infra: internals` unit ports `reason-parts.ts` behind a re-export).
/// `REASONS.cancelOpen`.
pub const REASON_CANCEL_OPEN: &str = "cancel-open";
/// `REASONS.none`.
pub use crate::combobox::value_chips::REASON_NONE;
/// `REASONS.listNavigation` (`floating_ui/reasons.rs`).
pub use leptos_ui_internals::floating_ui::reasons::LIST_NAVIGATION as REASON_LIST_NAVIGATION;
/// `REASONS.triggerPress` (`floating_ui/reasons.rs`).
pub use leptos_ui_internals::floating_ui::reasons::TRIGGER_PRESS as REASON_TRIGGER_PRESS;

/// The mouseup-drag boundary offset (`getPseudoElementBounds.ts:14`) — a
/// release within 5px of the trigger's bounds still counts as "on" it.
pub const MOUSEUP_BOUNDARY_OFFSET: f64 = 5.0;

// ---------------------------------------------------------------------------
// The read-side state composition (ComboboxTrigger.tsx:39-88)
// ---------------------------------------------------------------------------

/// The `disabled` fold (`:82`): `fieldDisabled || comboboxDisabled ||
/// disabledProp`.
pub fn trigger_disabled(field: bool, combobox: bool, prop: bool) -> bool {
    field || combobox || prop
}

/// The `id` resolution (`:86-87`): the labelable-id registration receives the
/// prop only when the input is inside the popup; the element id is the prop,
/// falling back to the root id only in that same mode.
#[derive(Clone, Debug, PartialEq)]
pub enum TriggerId {
    /// A concrete id — the prop, or the root id in the inside-popup mode.
    Resolved(String),
    /// Nothing supplied — `useBaseUiId` generates one at wiring time.
    Generated,
}

pub fn resolve_trigger_id(
    id_prop: Option<String>,
    input_inside_popup: bool,
    root_id: Option<String>,
) -> TriggerId {
    if let Some(id) = id_prop {
        return TriggerId::Resolved(id);
    }
    if input_inside_popup {
        if let Some(id) = root_id {
            return TriggerId::Resolved(id);
        }
    }
    TriggerId::Generated
}

/// The `useLabelableId` registration input (`:86`): the prop id is forwarded
/// only when the input is inside the popup; otherwise the trigger does not
/// register a labelable id at all (`undefined`).
pub fn labelable_id_input(id_prop: Option<&str>, input_inside_popup: bool) -> Option<String> {
    if input_inside_popup {
        id_prop.map(str::to_string)
    } else {
        None
    }
}

/// The `aria-controls` resolution (`:90-98`): `None` while closed. Open with
/// the input inside the popup falls back to the default popup id while the
/// popup registers its own (custom ids are stored once the popup mounts), so
/// the attribute is set on the same commit `open` becomes `true`. Open with a
/// standalone input points at the list element's id.
pub fn resolve_aria_controls(
    open: bool,
    input_inside_popup: bool,
    stored_popup_id: Option<&str>,
    root_id: Option<&str>,
    list_element: Option<&Element>,
) -> Option<String> {
    if !open {
        return None;
    }
    if input_inside_popup {
        stored_popup_id
            .map(str::to_string)
            .or_else(|| get_combobox_popup_id(root_id))
    } else {
        list_element.and_then(|el| el.get_attribute("id"))
    }
}

/// The typeahead hook's enabled gate (`:106-109`): typeahead on a closed
/// trigger commits a value rather than moving a highlight, so it stays gated
/// on `readOnly` and on the single selection mode.
pub fn typeahead_enabled(
    open: bool,
    read_only: bool,
    combobox_disabled: bool,
    selection_mode: &str,
) -> bool {
    !open && !read_only && !combobox_disabled && selection_mode == "single"
}

/// The typeahead `onMatch` commit plan (`:113-118`): the match commits only
/// when `valuesRef[index]` carries a defined value — a sparse label entry
/// (`labelsRef` holds `Some(label)` where `valuesRef` holds nothing) is
/// skipped without touching the selection.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeaheadMatchPlan {
    /// `store.context.setSelectedValue(nextSelectedValue, …REASONS.none)`.
    Commit(Value),
    /// The matched index has no value registered — no commit.
    Skip,
}

pub fn plan_typeahead_match(values_at_index: Option<Value>) -> TypeaheadMatchPlan {
    match values_at_index {
        Some(value) if !value.is_null() => TypeaheadMatchPlan::Commit(value),
        // JS `undefined` at the index ports to `None`; a literal `null` is a
        // real value upstream and commits.
        Some(value) => TypeaheadMatchPlan::Commit(value),
        None => TypeaheadMatchPlan::Skip,
    }
}

/// The click hook's gate (`:121-124`): enabled unless the combobox is
/// disabled; the interaction is a `mousedown`, not a click.
pub fn click_enabled(combobox_disabled: bool) -> bool {
    !combobox_disabled
}

/// The state object's `placeholder` field (`:138`): `none`-mode never carries
/// the placeholder hook; otherwise it tracks "no selected value".
pub fn trigger_placeholder(selection_mode: &str, has_selected_value: bool) -> bool {
    if selection_mode == "none" {
        false
    } else {
        !has_selected_value
    }
}

// ---------------------------------------------------------------------------
// The aria wiring (ComboboxTrigger.tsx:154-163)
// ---------------------------------------------------------------------------

/// The element-attribute plan for the aria block (`:154-163`). The
/// `aria-expanded`/`aria-labelledby`/id/tabIndex fields are unconditional;
/// the `aria-required`/`aria-readonly` pair is emitted only alongside the
/// `combobox` role (the input-inside-popup mode — without it the trigger is a
/// plain button and a standalone `Combobox.Input` already carries
/// `aria-readonly`).
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerAriaPlan {
    pub id: Option<String>,
    pub tab_index: i32,
    pub role: Option<String>,
    pub aria_expanded: bool,
    pub aria_haspopup: String,
    pub aria_controls: Option<String>,
    pub aria_required: Option<bool>,
    pub aria_readonly: Option<bool>,
    pub aria_labelledby: Option<String>,
}

pub fn plan_trigger_aria(
    id: TriggerId,
    input_inside_popup: bool,
    open: bool,
    required: bool,
    read_only: bool,
    aria_controls: Option<String>,
    aria_labelledby: Option<String>,
) -> TriggerAriaPlan {
    TriggerAriaPlan {
        id: match id {
            TriggerId::Resolved(id) => Some(id),
            TriggerId::Generated => None, // generated at wiring time
        },
        tab_index: if input_inside_popup { 0 } else { -1 },
        role: if input_inside_popup {
            Some("combobox".into())
        } else {
            None
        },
        aria_expanded: open,
        aria_haspopup: if input_inside_popup {
            "dialog".into()
        } else {
            "listbox".into()
        },
        aria_controls,
        aria_required: input_inside_popup.then_some(required),
        aria_readonly: input_inside_popup.then_some(read_only),
        aria_labelledby,
    }
}

// ---------------------------------------------------------------------------
// The event handlers (ComboboxTrigger.tsx:166-253)
// ---------------------------------------------------------------------------

/// The `onFocus` plan (`:166-174`): the Field is focused; a disabled trigger
/// stops there, otherwise the pending mount is scheduled (`focusTimeout`
/// starts `forceMount` at 0).
#[derive(Clone, Debug, PartialEq)]
pub struct FocusPlan {
    pub set_focused: bool,
    pub force_mount: bool,
}

pub fn plan_focus(disabled: bool) -> FocusPlan {
    FocusPlan {
        set_focused: true,
        force_mount: !disabled,
    }
}

/// The `onBlur` plan (`:175-188`): a focus move into the popup positioner is
/// not a blur; otherwise the Field is touched and unfocused, and `onBlur`
/// validation commits the mode's value — the input value in `none` mode, the
/// selection otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct BlurPlan {
    /// Return early — the popup has the focus.
    pub is_popup_focus_move: bool,
    pub set_touched: bool,
    pub set_focused: bool,
    /// `validation.commit(valueToValidate)` when the mode is `onBlur`.
    pub commit_value: Option<Value>,
}

pub fn plan_blur(
    related_target_in_positioner: bool,
    validation_mode_on_blur: bool,
    selection_mode: &str,
    input_value: &str,
    selected_value: &Value,
) -> BlurPlan {
    if related_target_in_positioner {
        return BlurPlan {
            is_popup_focus_move: true,
            set_touched: false,
            set_focused: true,
            commit_value: None,
        };
    }
    BlurPlan {
        is_popup_focus_move: false,
        set_touched: true,
        set_focused: false,
        commit_value: if validation_mode_on_blur {
            if selection_mode == "none" {
                Some(Value::String(input_value.to_string()))
            } else {
                Some(selected_value.clone())
            }
        } else {
            None
        },
    }
}

/// The `onMouseDown` plan (`:189-243`). Upstream order: the disabled bail
/// (`:190-192`), the floating dom-reference write outside the popup
/// (`:194-196`), the item-registration mount (`:198-199`), the input focus +
/// preventDefault for non-touch pointers (`:201-207`, the default is
/// suppressed outside the popup so the standalone input keeps the focus),
/// the open bail (`:209-211`), and — only with the input inside the popup —
/// the one-shot `mouseup` pairing that closes a drag released off the
/// trigger (`:240-242`).
#[derive(Clone, Debug, PartialEq)]
pub struct MouseDownPlan {
    /// `floatingRootContext.set('domReferenceElement', currentTarget)` —
    /// outside the popup only.
    pub set_dom_reference: bool,
    /// `store.context.forceMount()` — items register for the initial
    /// selection highlight.
    pub force_mount: bool,
    /// `store.context.inputRef.current?.focus()` — non-touch pointers only.
    pub focus_input: bool,
    /// `event.preventDefault()` — only outside the popup.
    pub prevent_default: bool,
    /// The one-shot `mouseup` listener the inside-popup mode installs.
    pub track_mouseup: bool,
}

pub fn plan_mouse_down(
    disabled: bool,
    input_inside_popup: bool,
    pointer_type: &str,
    open: bool,
) -> MouseDownPlan {
    if disabled {
        return MouseDownPlan {
            set_dom_reference: false,
            force_mount: false,
            focus_input: false,
            prevent_default: false,
            track_mouseup: false,
        };
    }
    let focus_input = pointer_type != "touch";
    MouseDownPlan {
        set_dom_reference: !input_inside_popup,
        force_mount: true,
        focus_input,
        prevent_default: focus_input && !input_inside_popup,
        track_mouseup: input_inside_popup && !open,
    }
}

/// The paired-`mouseup` handler's plan (`:215-238`). A release on the
/// trigger, inside the positioner, or inside the list never cancels; a
/// release within [`MOUSEUP_BOUNDARY_OFFSET`] of the trigger's bounds is
/// still "on" the trigger (`isMouseWithinBounds`,
/// `getPseudoElementBounds.ts:20-29`); anything else closes the popup with
/// the `cancel-open` reason. A trigger that already unmounted stops the
/// handler before any read (`:216-219` — the pending-mouseup test,
/// `ComboboxTrigger.test.tsx:172-201`).
#[derive(Clone, Debug, PartialEq)]
pub enum MouseUpPlan {
    /// The trigger unmounted — the pending listener exits silently.
    TriggerGone,
    /// The release landed on the trigger, the positioner, the list, or
    /// within the trigger's 5px-inflated bounds — the popup stays open.
    Keep,
    /// `store.context.setOpen(false, …REASONS.cancelOpen, mouseEvent)`.
    Close,
}

pub fn plan_mouse_up(
    trigger_mounted: bool,
    has_mouse_up_target: bool,
    trigger_contains_target: bool,
    positioner_contains_target: bool,
    list_contains_target: bool,
    mouse_within_bounds: bool,
) -> MouseUpPlan {
    if !trigger_mounted {
        return MouseUpPlan::TriggerGone;
    }
    if !has_mouse_up_target
        || trigger_contains_target
        || positioner_contains_target
        || list_contains_target
    {
        return MouseUpPlan::Keep;
    }
    if mouse_within_bounds {
        return MouseUpPlan::Keep;
    }
    MouseUpPlan::Close
}

/// The `isMouseWithinBounds` rectangle test
/// (`getPseudoElementBounds.ts:20-29`): the client point must sit within the
/// element's bounds inflated by [`MOUSEUP_BOUNDARY_OFFSET`] on every side.
pub fn is_mouse_within_bounds(
    client_x: f64,
    client_y: f64,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
) -> bool {
    let o = MOUSEUP_BOUNDARY_OFFSET;
    client_x >= left - o && client_x <= right + o && client_y >= top - o && client_y <= bottom + o
}

/// The `onKeyDown` plan (`:244-253`): the arrow keys stop the event and open
/// the popup with the `list-navigation` reason, moving the focus to the
/// input. Every other key falls through untouched. (The textarea case in
/// `ComboboxTrigger.test.tsx:440-455` never reaches here — the click hook's
/// reference typing keeps a textarea trigger from receiving the role the
/// keydown is wired to; the plan mirrors the handler's own body, which has
/// no element-type gate.)
#[derive(Clone, Debug, PartialEq)]
pub enum KeyDownPlan {
    /// `stopEvent(event)` + `setOpen(true, …REASONS.listNavigation)` +
    /// `inputRef.current?.focus()`.
    OpenWithListNavigation,
    /// Any other key — no intercept.
    Inert,
}

pub fn plan_key_down(key: &str) -> KeyDownPlan {
    if key == "ArrowDown" || key == "ArrowUp" {
        KeyDownPlan::OpenWithListNavigation
    } else {
        KeyDownPlan::Inert
    }
}

// ---------------------------------------------------------------------------
// The state-attributes mapping (utils/stateAttributesMapping.ts:10-23)
// ---------------------------------------------------------------------------

/// `triggerStateAttributesMapping`
/// (`packages/react/src/combobox/utils/stateAttributesMapping.ts:10-23`):
/// the pressable-trigger open-state mapping (`data-popup-open` +
/// `data-pressed` while open), the field-validity mapping (`data-valid` /
/// `data-invalid`, nothing before the first validation), the popup-side hook
/// (the retained side, present only while the positioner mounts), the
/// list-empty hook, and the default truthy walk for the remaining state
/// fields (`data-readonly`, `data-disabled`, `data-placeholder`, …).
pub fn trigger_state_attributes(state: &Map<String, Value>) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    for (key, value) in state {
        let mapped = match key.as_str() {
            "open" => {
                leptos_ui_internals::popup_state_mapping::pressable_trigger_open_state_mapping(
                    key, value,
                )
            }
            "valid" => leptos_ui_internals::state_attributes::field_validity_mapping(key, value),
            "popupSide" => match value.as_str() {
                Some(side) if !side.is_empty() => Some(Some(
                    [(("data-popup-side").to_string(), side.to_string())]
                        .into_iter()
                        .collect(),
                )),
                _ => Some(None),
            },
            "listEmpty" => match value.as_bool() {
                Some(true) => Some(Some(
                    [(("data-list-empty").to_string(), String::new())]
                        .into_iter()
                        .collect(),
                )),
                _ => Some(None),
            },
            // The default walk (`getStateAttributesProps.ts:24-28`): a
            // truthy field emits `data-<kebab-key>`, `true` in the bare
            // empty-valued form.
            _ => {
                let truthy = match value {
                    Value::Bool(b) => *b,
                    Value::String(s) => !s.is_empty(),
                    Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                    _ => false,
                };
                if truthy {
                    let kebab = key
                        .chars()
                        .flat_map(|c| {
                            if c.is_ascii_uppercase() {
                                vec!['-', c.to_ascii_lowercase()]
                            } else {
                                vec![c]
                            }
                        })
                        .collect::<String>();
                    Some(Some(
                        [(format!("data-{kebab}"), String::new())]
                            .into_iter()
                            .collect(),
                    ))
                } else {
                    Some(None)
                }
            }
        };
        if let Some(Some(props)) = mapped {
            for (name, attr_value) in props {
                attributes.push((name, attr_value));
            }
        }
    }
    attributes
}

// ---------------------------------------------------------------------------
// The store-facing composition (the render body's reads, :60-88)
// ---------------------------------------------------------------------------

/// The state-object snapshot the wiring feeds to
/// [`trigger_state_attributes`] — the part's state keys
/// (`:131-139`), with the Field-validity `valid` carried through as `null`
/// before the first validation.
pub fn trigger_state_map(
    store: &ComboboxStore,
    field_disabled: bool,
    disabled_prop: bool,
    field_touched: bool,
    field_valid: Option<bool>,
    list_empty: bool,
) -> Map<String, Value> {
    let state = store.select(|s| s.clone());
    let disabled = trigger_disabled(field_disabled, state.disabled, disabled_prop);
    let popup_side = crate::combobox::parts_util::use_popup_side(
        state.mounted,
        state.positioner_element.as_ref(),
        state.popup_side.clone(),
    );
    let has_selected_value = crate::combobox::store::selectors::has_selected_value(&state);
    let mut map = Map::new();
    map.insert("touched".into(), Value::Bool(field_touched));
    map.insert(
        "valid".into(),
        field_valid.map(Value::Bool).unwrap_or(Value::Null),
    );
    map.insert("readOnly".into(), Value::Bool(state.read_only));
    map.insert("open".into(), Value::Bool(state.open));
    map.insert("disabled".into(), Value::Bool(disabled));
    map.insert(
        "popupSide".into(),
        popup_side.map(Value::String).unwrap_or(Value::Null),
    );
    map.insert("listEmpty".into(), Value::Bool(list_empty));
    map.insert(
        "placeholder".into(),
        Value::Bool(trigger_placeholder(
            &state.selection_mode,
            has_selected_value,
        )),
    );
    map
}

#[cfg(test)]
mod trigger_wiring_tests;
