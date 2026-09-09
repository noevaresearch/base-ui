//! Port of `packages/react/src/floating-ui-react/utils/constants.ts` — the unit's shared
//! string constants, used by `utils/element.ts` (`FOCUSABLE_ATTRIBUTE`,
//! `TYPEABLE_SELECTOR`), the composite/grid navigation (`ACTIVE_KEY`, `SELECTED_KEY` —
//! `utils/composite.ts:101-283` reads them off middleware data), and list navigation
//! (`ARROW_*` key names, `hooks/useListNavigation.ts`).

/// `FOCUSABLE_ATTRIBUTE` (`constants.ts:1`) — marks the element carrying the floating
/// element's event handlers/aria props so `getFloatingFocusElement` can find it
/// (`utils/element.ts:83-96`).
pub const FOCUSABLE_ATTRIBUTE: &str = "data-base-ui-focusable";

/// `ACTIVE_KEY` (`constants.ts:2`) — the composite middleware-data key.
pub const ACTIVE_KEY: &str = "active";

/// `SELECTED_KEY` (`constants.ts:3`) — the composite middleware-data key.
pub const SELECTED_KEY: &str = "selected";

/// `TYPEABLE_SELECTOR` (`constants.ts:4-6`) — the selector matching typeable elements
/// (inputs, contenteditables, textareas).
pub const TYPEABLE_SELECTOR: &str = "input:not([type='hidden']):not([disabled]),\
[contenteditable]:not([contenteditable='false']),textarea:not([disabled])";

/// `ARROW_LEFT` (`constants.ts:7`).
pub const ARROW_LEFT: &str = "ArrowLeft";

/// `ARROW_RIGHT` (`constants.ts:8`).
pub const ARROW_RIGHT: &str = "ArrowRight";

/// `ARROW_UP` (`constants.ts:9`).
pub const ARROW_UP: &str = "ArrowUp";

/// `ARROW_DOWN` (`constants.ts:10`).
pub const ARROW_DOWN: &str = "ArrowDown";

/// `CLICK_TRIGGER_IDENTIFIER` (`packages/react/src/internals/constants.ts:7`) — marks a
/// nested element as a click trigger; the FocusManager's outside-pointer tracking resets
/// its focus-out suppression on the next tick after pressing one
/// (`FloatingFocusManager.tsx:374-381`). Provisionally hosted here until the
/// `infra: internals` constants unit ports (the `focus_guard` precedent).
pub const CLICK_TRIGGER_IDENTIFIER: &str = "data-base-ui-click-trigger";
