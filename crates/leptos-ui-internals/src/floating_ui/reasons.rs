//! Port of the `REASONS` string registry the floating-ui unit's hooks pass to
//! `setOpen` — upstream `packages/react/src/internals/reasons.ts` re-exporting
//! `./reason-parts` (`specs/library/floating-ui-react/behavior.md`, "Events": every
//! interaction hook funnels state changes through `onOpenChange(nextOpen, details)`
//! with a `details.reason` drawn from `REASONS`).
//!
//! The registry belongs to the `infra: internals` unit (not yet ported — its TODO item
//! is still `not-started`), so the constants the floating-ui hooks reference live here
//! until that unit lands, exactly like `FloatingUIOpenChangeDetails` lives in
//! [`crate::floating_ui::types`] ("defined here until the `infra: internals` unit ports
//! `internals/types`"). When the internals unit ports `reason-parts.ts`, these constants
//! move behind a re-export of that port; the string values are upstream's verbatim, so
//! the move is a pointer change, not a behavior change.
//!
//! Only the reasons the floating-ui unit itself emits are defined here — the full
//! `reason-parts.ts` registry (input-change, item-press, …) belongs to the components
//! that emit them.

/// `REASONS.none` (`reason-parts.ts:1`).
pub const NONE: &str = "none";

/// `REASONS.triggerPress` (`reason-parts.ts:3`) — the default reason for
/// [`crate::floating_ui::use_click`] opens/closes.
pub const TRIGGER_PRESS: &str = "trigger-press";

/// `REASONS.triggerHover` (`reason-parts.ts:4`).
pub const TRIGGER_HOVER: &str = "trigger-hover";

/// `REASONS.triggerFocus` (`reason-parts.ts:5`) — the reason [`crate::floating_ui::
/// use_focus`] opens and closes with.
pub const TRIGGER_FOCUS: &str = "trigger-focus";

/// `REASONS.outsidePress` (`reason-parts.ts:7`).
pub const OUTSIDE_PRESS: &str = "outside-press";

/// `REASONS.inputPress` (`reason-parts.ts:19`) — [`crate::floating_ui::use_click`]'s
/// alternate `reason` for typeable (input) references.
pub const INPUT_PRESS: &str = "input-press";

/// `REASONS.escapeKey` (`reason-parts.ts:22`).
pub const ESCAPE_KEY: &str = "escape-key";
