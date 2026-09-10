//! Port of `packages/react/src/utils/collapsibleOpenStateMapping.ts` — the
//! state-attribute mappings Collapsible/Accordion pass to the render layer
//! (implementation.md, "Downstream consumers": "Collapsible/Accordion
//! (`collapsibleOpenStateMapping`)").
//!
//! The two mappings read the collapsible data-attribute constants from the component's
//! own modules (`:2-3`), which are Phase B units not yet ported — the constants are
//! inlined here with their upstream definitions cited:
//!
//! - `CollapsiblePanelDataAttributes.open` = `'data-open'`,
//!   `CollapsiblePanelDataAttributes.closed` = `'data-closed'`
//!   (`packages/react/src/collapsible/panel/CollapsiblePanelDataAttributes.ts:6,10`).
//! - `CollapsibleTriggerDataAttributes.panelOpen` = `'data-panel-open'`
//!   (`packages/react/src/collapsible/trigger/CollapsibleTriggerDataAttributes.ts:14`).
//!
//! The port follows the crate's [`crate::state_attributes::StateAttributesMapping`]
//! convention (see the `popup_state_mapping` module docs for the three-way return).

use std::collections::BTreeMap;

use serde_json::Value;

use crate::state_attributes::{StateAttributeProps, is_truthy};

/// `CollapsiblePanelDataAttributes.open` (`CollapsiblePanelDataAttributes.ts:6`).
pub const PANEL_OPEN: &str = "data-open";
/// `CollapsiblePanelDataAttributes.closed` (`CollapsiblePanelDataAttributes.ts:10`).
pub const PANEL_CLOSED: &str = "data-closed";
/// `CollapsibleTriggerDataAttributes.panelOpen`
/// (`CollapsibleTriggerDataAttributes.ts:14`).
pub const TRIGGER_PANEL_OPEN: &str = "data-panel-open";

/// The two panel hooks (`:5-11`): both are bare `data-*` attributes (the empty-string
/// value).
fn panel_hook(open: bool) -> StateAttributeProps {
    BTreeMap::from([(
        (if open { PANEL_OPEN } else { PANEL_CLOSED }).to_string(),
        String::new(),
    )])
}

/// `triggerOpenStateMapping` (`collapsibleOpenStateMapping.ts:13-24`) — on the
/// *trigger*: open emits [`TRIGGER_PANEL_OPEN`], closed returns `null`
/// (`Some(None)` — no attributes).
pub fn trigger_open_state_mapping(key: &str, value: &Value) -> Option<Option<StateAttributeProps>> {
    if key != "open" {
        return None;
    }

    if is_truthy(value) {
        Some(Some(BTreeMap::from([(
            TRIGGER_PANEL_OPEN.to_string(),
            String::new(),
        )])))
    } else {
        Some(None)
    }
}

/// `collapsibleOpenStateMapping` (`collapsibleOpenStateMapping.ts:26-34`) — on the
/// *panel*: open emits [`PANEL_OPEN`], closed emits [`PANEL_CLOSED`] (both branches
/// return a hook, unlike the trigger mapping).
pub fn collapsible_open_state_mapping(
    key: &str,
    value: &Value,
) -> Option<Option<StateAttributeProps>> {
    if key != "open" {
        return None;
    }

    Some(Some(panel_hook(is_truthy(value))))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // The trigger mapping's two branches (`:16-23`): open emits `data-panel-open`,
    // closed returns null.
    #[test]
    fn the_trigger_mapping_emits_panel_open_only_while_open() {
        assert_eq!(
            trigger_open_state_mapping("open", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-panel-open".to_string(), String::new())])
        );
        assert_eq!(
            trigger_open_state_mapping("open", &json!(false)),
            Some(None)
        );
    }

    // The panel mapping's two branches (`:26-32`): both emit a hook.
    #[test]
    fn the_panel_mapping_emits_open_and_closed_hooks() {
        assert_eq!(
            collapsible_open_state_mapping("open", &json!(true))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-open".to_string(), String::new())])
        );
        assert_eq!(
            collapsible_open_state_mapping("open", &json!(false))
                .expect("mapped")
                .expect("props"),
            BTreeMap::from([("data-closed".to_string(), String::new())])
        );
    }

    // Unhandled keys fall through to the default handling (`getStateAttributesProps.ts:15`).
    #[test]
    fn unhandled_keys_fall_through() {
        assert_eq!(
            trigger_open_state_mapping("transitionStatus", &json!("idle")),
            None
        );
        assert_eq!(collapsible_open_state_mapping("payload", &json!(1)), None);
    }
}
