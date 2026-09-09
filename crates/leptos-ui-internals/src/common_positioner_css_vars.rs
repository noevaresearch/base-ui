//! Port of `packages/react/src/utils/CommonPositionerCssVars.ts` — the runtime-computed
//! CSS custom properties the positioning layer writes onto the popup positioner element
//! for consumer stylesheets (the `--`-prefixed contract; `specs/architecture.md`,
//! "CSS custom properties").
//!
//! The upstream file is JSDoc-commented constants only; the port keeps the same names
//! and values as `&'static str` consts. Consumers read them through `getComputedStyle`
//! and stylesheets target them directly, exactly as upstream.

/// `availableWidth` (`CommonPositionerCssVars.ts:8`) — the available width between the
/// trigger and the edge of the viewport.
pub const AVAILABLE_WIDTH: &str = "--available-width";

/// `availableHeight` (`CommonPositionerCssVars.ts:13`) — the available height between
/// the trigger and the edge of the viewport.
pub const AVAILABLE_HEIGHT: &str = "--available-height";

/// `anchorWidth` (`CommonPositionerCssVars.ts:18`) — the anchor's width.
pub const ANCHOR_WIDTH: &str = "--anchor-width";

/// `anchorHeight` (`CommonPositionerCssVars.ts:23`) — the anchor's height.
pub const ANCHOR_HEIGHT: &str = "--anchor-height";

/// `transformOrigin` (`CommonPositionerCssVars.ts:28`) — the coordinates this element is
/// anchored to, used for animations and transitions.
pub const TRANSFORM_ORIGIN: &str = "--transform-origin";

/// `positionerWidth` (`CommonPositionerCssVars.ts:33`) — the width of the popup's
/// positioner; set `width` to this value when using CSS to animate size changes.
pub const POSITIONER_WIDTH: &str = "--positioner-width";

/// `positionerHeight` (`CommonPositionerCssVars.ts:38`) — the height of the popup's
/// positioner; set `height` to this value when using CSS to animate size changes.
pub const POSITIONER_HEIGHT: &str = "--positioner-height";
