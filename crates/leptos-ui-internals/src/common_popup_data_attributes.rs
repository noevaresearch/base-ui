//! Port of `packages/react/src/utils/CommonPopupDataAttributes.ts` — the data
//! attributes every popup family (Dialog, Popover, Menu, Tooltip, PreviewCard) emits on
//! its popup element. Constants only, exactly as upstream (JSDoc-commented exports).
//!
//! `startingStyle`/`endingStyle` alias the `TransitionStatusDataAttributes` constants
//! (`CommonPopupDataAttributes.ts:1,17,23` — re-exported from the internals
//! state-attributes module, the crate's port of that upstream file).

/// The `TransitionStatusDataAttributes` constants the aliases point at
/// (`CommonPopupDataAttributes.ts:1`).
pub use crate::state_attributes::{ENDING_STYLE, STARTING_STYLE};

/// `open` (`CommonPopupDataAttributes.ts:7`) — present when the popup is open.
pub const OPEN: &str = "data-open";

/// `closed` (`CommonPopupDataAttributes.ts:13`) — present when the popup is closed.
pub const CLOSED: &str = "data-closed";

/// `anchorHidden` (`CommonPopupDataAttributes.ts:29`) — present when the anchor is
/// hidden.
pub const ANCHOR_HIDDEN: &str = "data-anchor-hidden";

/// `side` (`CommonPopupDataAttributes.ts:35`) — which side the popup is positioned
/// relative to the trigger (`'top' | 'bottom' | 'left' | 'right' | 'inline-end' |
/// 'inline-start'`).
pub const SIDE: &str = "data-side";

/// `align` (`CommonPopupDataAttributes.ts:41`) — how the popup is aligned relative to
/// the specified side (`'start' | 'center' | 'end'`).
pub const ALIGN: &str = "data-align";
