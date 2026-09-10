//! Port of `packages/react/src/utils/CommonViewportDataAttributes.ts` — the one data
//! attribute the four Viewport parts (Popover, Menu, Tooltip, PreviewCard) expose on
//! the viewport element, consumed by the `popupViewportStateMapping`
//! (`usePopupViewport.tsx:21-30`).

/// Indicates the direction from which the popup was activated. This can be used to
/// create directional animations based on how the popup was triggered. Contains
/// space-separated values for both horizontal and vertical axes.
/// (`CommonViewportDataAttributes.ts:6` — the type is `` `${'left' | 'right' | ''}
/// ${'down' | 'up' | ''}` ``.)
pub const ACTIVATION_DIRECTION: &str = "data-activation-direction";
