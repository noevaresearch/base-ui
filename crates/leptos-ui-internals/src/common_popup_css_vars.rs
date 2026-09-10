//! Port of `packages/react/src/utils/CommonPopupCssVars.ts` — the runtime-computed CSS
//! custom properties the viewport morphing writes onto the popup's previous-content
//! container (the `--`-prefixed contract; `specs/architecture.md`, "CSS custom
//! properties"). Constants only, exactly as upstream.

/// `popupWidth` (`CommonPopupCssVars.ts:9`) — the popup's width when the previous
/// content was rendered; used to freeze the dimensions of the popup when animating
/// between different content.
pub const POPUP_WIDTH: &str = "--popup-width";

/// `popupHeight` (`CommonPopupCssVars.ts:17`) — the popup's height when the previous
/// content was rendered; used to freeze the dimensions of the popup when animating
/// between different content.
pub const POPUP_HEIGHT: &str = "--popup-height";
