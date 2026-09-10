//! Port of `packages/react/src/utils/popupStateMapping.ts` — the state-attribute
//! mappings every popup family (and its triggers) passes to the render layer to turn
//! the `{ open, anchorHidden, transitionStatus }` state into `data-*` attributes
//! (behavior.md, "Accessibility": popup state is exposed only through data attributes).
//!
//! Upstream exports each mapping as an object with one method per state field
//! (`{ open(value) { … } }`) consumed by `getStateAttributesProps`. The port follows
//! the crate's [`crate::state_attributes::StateAttributesMapping`] convention — one
//! callable per mapping receiving `(key, value)` (see that module's docs): returning
//! `None` mirrors the method being absent for the key (fall through to default
//! handling), `Some(None)` mirrors the method returning `null` (emit nothing), and
//! `Some(Some(props))` mirrors the returned props object.
//!
//! The branch conditions use JS truthiness (`if (value)`) exactly like upstream — the
//! mappings' domain is boolean state fields, but a non-boolean value follows the same
//! coercion rather than being rejected.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::common_popup_data_attributes::{ANCHOR_HIDDEN, CLOSED, OPEN};
use crate::common_trigger_data_attributes::{POPUP_OPEN, PRESSED};
use crate::state_attributes::{StateAttributeProps, is_truthy, transition_status_mapping};

fn hook(attribute: &str) -> Option<Option<StateAttributeProps>> {
    Some(Some(BTreeMap::from([(
        attribute.to_string(),
        String::new(),
    )])))
}

/// `triggerOpenStateMapping` (`popupStateMapping.ts:33-41`) — emits
/// [`common_trigger_data_attributes::POPUP_OPEN`] on the trigger while open, nothing
/// when closed.
pub fn trigger_open_state_mapping(key: &str, value: &Value) -> Option<Option<StateAttributeProps>> {
    if key != "open" {
        return None;
    }

    if is_truthy(value) {
        hook(POPUP_OPEN)
    } else {
        Some(None)
    }
}

/// `pressableTriggerOpenStateMapping` (`popupStateMapping.ts:43-51`) — the pressable
/// trigger variant, additionally emitting [`common_trigger_data_attributes::PRESSED`]
/// while open.
pub fn pressable_trigger_open_state_mapping(
    key: &str,
    value: &Value,
) -> Option<Option<StateAttributeProps>> {
    if key != "open" {
        return None;
    }

    if is_truthy(value) {
        Some(Some(BTreeMap::from([
            (POPUP_OPEN.to_string(), String::new()),
            (PRESSED.to_string(), String::new()),
        ])))
    } else {
        Some(None)
    }
}

/// `popupStateMapping` (`popupStateMapping.ts:53-66`) — on the popup element: open
/// emits [`common_popup_data_attributes::OPEN`], closed emits
/// [`common_popup_data_attributes::CLOSED`] (both branches return a hook, unlike the
/// trigger mappings); a hidden anchor emits [`common_popup_data_attributes::ANCHOR_HIDDEN`].
pub fn popup_state_mapping(key: &str, value: &Value) -> Option<Option<StateAttributeProps>> {
    match key {
        "open" => Some(Some(if is_truthy(value) {
            BTreeMap::from([(OPEN.to_string(), String::new())])
        } else {
            BTreeMap::from([(CLOSED.to_string(), String::new())])
        })),
        "anchorHidden" => {
            if is_truthy(value) {
                hook(ANCHOR_HIDDEN)
            } else {
                Some(None)
            }
        }
        _ => None,
    }
}

/// `popupTransitionStateMapping` (`popupStateMapping.ts:68-73`) — the spread
/// composition `{...popupStateMapping, ...transitionStatusMapping}`: the popup mapping
/// answers first and the transition-status mapping handles the `transitionStatus` key
/// it owns (the two never conflict — `popupStateMapping` has no `transitionStatus`
/// method, so spread order is observationally equivalent to this fallback order).
pub fn popup_transition_state_mapping(
    key: &str,
    value: &Value,
) -> Option<Option<StateAttributeProps>> {
    popup_state_mapping(key, value).or_else(|| transition_status_mapping(key, value))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::state_attributes::{ENDING_STYLE, STARTING_STYLE};

    // Mirrors `packages/react/src/utils/popupStateMapping.test.ts:11-15` — the
    // attribute names are pinned against literals so a rename cannot ship unnoticed
    // (`data-anchor-hidden` only appears in this file upstream).
    #[test]
    fn emits_the_open_and_closed_data_attributes() {
        assert_eq!(
            popup_state_mapping("open", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-open".to_string(), String::new())])
        );
        assert_eq!(
            popup_state_mapping("open", &json!(false))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-closed".to_string(), String::new())])
        );
    }

    // Mirrors `packages/react/src/utils/popupStateMapping.test.ts:17-20`.
    #[test]
    fn emits_the_anchor_hidden_data_attribute_only_when_the_anchor_is_hidden() {
        assert_eq!(
            popup_state_mapping("anchorHidden", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-anchor-hidden".to_string(), String::new())])
        );
        assert_eq!(
            popup_state_mapping("anchorHidden", &json!(false)),
            Some(None)
        );
    }

    // Mirrors `packages/react/src/utils/popupStateMapping.test.ts:22-26`.
    #[test]
    fn emits_the_trigger_data_attributes_only_while_open() {
        assert_eq!(
            trigger_open_state_mapping("open", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-popup-open".to_string(), String::new())])
        );
        assert_eq!(
            trigger_open_state_mapping("open", &json!(false)),
            Some(None)
        );
    }

    // Mirrors `packages/react/src/utils/popupStateMapping.test.ts:28-33`.
    #[test]
    fn emits_the_pressed_data_attribute_on_pressable_triggers_only_while_open() {
        assert_eq!(
            pressable_trigger_open_state_mapping("open", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([
                ("data-popup-open".to_string(), String::new()),
                ("data-pressed".to_string(), String::new()),
            ])
        );
        assert_eq!(
            pressable_trigger_open_state_mapping("open", &json!(false)),
            Some(None)
        );
    }

    // The composition contract (`popupStateMapping.ts:68-73`): the transition keys
    // route to the transition-status mapping while the popup keys keep their hooks.
    #[test]
    fn the_transition_composition_routes_each_key_to_its_own_mapping() {
        assert_eq!(
            popup_transition_state_mapping("transitionStatus", &json!("starting"))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([(STARTING_STYLE.to_string(), String::new())])
        );
        assert_eq!(
            popup_transition_state_mapping("transitionStatus", &json!("ending"))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([(ENDING_STYLE.to_string(), String::new())])
        );
        assert_eq!(
            popup_transition_state_mapping("transitionStatus", &json!("idle")),
            Some(None)
        );
        assert_eq!(
            popup_transition_state_mapping("open", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-open".to_string(), String::new())])
        );
    }

    // Unhandled keys fall through to the default handling (the mapping `None` return,
    // `getStateAttributesProps.ts:15`).
    #[test]
    fn unhandled_keys_fall_through_to_the_default_handling() {
        assert_eq!(popup_state_mapping("side", &json!("top")), None);
        assert_eq!(
            trigger_open_state_mapping("anchorHidden", &json!(true)),
            None
        );
        assert_eq!(
            pressable_trigger_open_state_mapping("anchorHidden", &json!(true)),
            None
        );
        assert_eq!(popup_transition_state_mapping("payload", &json!(1)), None);
    }
}
