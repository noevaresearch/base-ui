//! Port of `packages/react/src/utils/CommonTriggerDataAttributes.ts` — the data
//! attributes every popup family emits on its trigger elements. Constants only, exactly
//! as upstream.

/// `popupOpen` (`CommonTriggerDataAttributes.ts:5`) — present when the popup is open.
pub const POPUP_OPEN: &str = "data-popup-open";

/// `pressed` (`CommonTriggerDataAttributes.ts:11`) — present when a pressable trigger
/// is pressed.
pub const PRESSED: &str = "data-pressed";
