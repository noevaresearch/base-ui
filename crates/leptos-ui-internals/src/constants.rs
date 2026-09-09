//! Port of `packages/react/src/internals/constants.ts` — the `internals` unit's shared
//! constants (`TODO.md`, item `infra: internals`).
//!
//! Upstream is a 41-line, dependency-free constants module (`packages/react/src/internals/
//! constants.ts:1-41`) consumed across the library: the typeahead/patient-click thresholds
//! by popup triggers, the swipe-ignore attribute/selector pair by slider-like components,
//! the collision-avoidance presets by `useAnchorPositioning`, the disabled-transitions style
//! by `getDisabledMountTransitionStyles`
//! (`packages/react/src/internals/getDisabledMountTransitionStyles.ts:5-9`), and the
//! `ownerVisuallyHidden` style by `FloatingPortal`'s `aria-owns` owner span.
//!
//! Rust adaptations (behavior-preserving):
//!
//! - `DISABLED_TRANSITIONS_STYLE` is upstream a props object `{ style: { transition: 'none' } }`
//!   (`constants.ts:5`). The port is view-free (`specs/architecture.md`, the crate-workspace
//!   decision), and the crate's established shape for declaration maps is the
//!   `&[(&str, &str)]` slice (the `floating_portal` precedent this port replaces), so the
//!   constant carries the inner `style` declaration map; the `{ style: ... }` wrapper key is
//!   the consumer's props-slot shape, reconstructed at the view layer.
//! - `ownerVisuallyHidden` (`constants.ts:36-41`, a `React.CSSProperties`) ports to the same
//!   declaration-map shape as [`OWNER_VISUALLY_HIDDEN`]. React's numeric-to-pixel conversion
//!   for `top`/`left` (`constants.ts:38-40`, the numbers `0`) is pre-applied as `"0px"`, the
//!   value React actually puts in the DOM.
//! - `DROPDOWN_COLLISION_AVOIDANCE`/`POPUP_COLLISION_AVOIDANCE` (`constants.ts:18-28`) port
//!   their single `fallbackAxisSide` field onto a [`CollisionAvoidance`] struct, preserving
//!   the `as const` literal types as `&'static str` values.
//! - The two swipe selectors (`constants.ts:11-12`) are derived from their attribute
//!   constants with `format!`-free string literals — the upstream templates are static, so
//!   the port keeps them as independent `&'static str` consts mirroring the definitions.

/// `TYPEAHEAD_RESET_MS` (`packages/react/src/internals/constants.ts:3`) — how long a
/// typeahead sequence remembers its buffer before resetting.
pub const TYPEAHEAD_RESET_MS: u32 = 500;

/// `PATIENT_CLICK_THRESHOLD` (`packages/react/src/internals/constants.ts:4`) — the maximum
/// gap between the two presses of a patient (double) click.
pub const PATIENT_CLICK_THRESHOLD: u32 = 500;

/// `DISABLED_TRANSITIONS_STYLE` (`packages/react/src/internals/constants.ts:5`) — the
/// `transition: none` declaration map applied while a component is in its `'starting'`
/// transition state (`packages/react/src/internals/getDisabledMountTransitionStyles.ts:5-9`).
/// See the module docs for the dropped `{ style: ... }` wrapper.
pub const DISABLED_TRANSITIONS_STYLE: &[(&str, &str)] = &[("transition", "none")];

/// `CLICK_TRIGGER_IDENTIFIER` (`packages/react/src/internals/constants.ts:7`) — marks a
/// nested element as a click trigger; the FocusManager's outside-pointer tracking resets
/// its focus-out suppression on the next tick after pressing one
/// (`FloatingFocusManager.tsx:374-381`). Previously hosted provisionally in
/// `floating_ui/constants.rs` (the `focus_guard` precedent); re-exported there for the
/// FocusManager's `closest` lookup.
pub const CLICK_TRIGGER_IDENTIFIER: &str = "data-base-ui-click-trigger";

/// `BASE_UI_SWIPE_IGNORE_ATTRIBUTE` (`packages/react/src/internals/constants.ts:8`) — marks
/// an element whose swipes must not trigger the ambient dismiss/swipe gestures.
pub const BASE_UI_SWIPE_IGNORE_ATTRIBUTE: &str = "data-base-ui-swipe-ignore";

/// `LEGACY_SWIPE_IGNORE_ATTRIBUTE` (`packages/react/src/internals/constants.ts:9`) — the
/// pre-rename attribute, still honored for compatibility.
pub const LEGACY_SWIPE_IGNORE_ATTRIBUTE: &str = "data-swipe-ignore";

/// `BASE_UI_SWIPE_IGNORE_SELECTOR` (`packages/react/src/internals/constants.ts:11`) — the
/// attribute-selector form of [`BASE_UI_SWIPE_IGNORE_ATTRIBUTE`].
pub const BASE_UI_SWIPE_IGNORE_SELECTOR: &str = "[data-base-ui-swipe-ignore]";

/// `LEGACY_SWIPE_IGNORE_SELECTOR` (`packages/react/src/internals/constants.ts:12`) — the
/// attribute-selector form of [`LEGACY_SWIPE_IGNORE_ATTRIBUTE`].
pub const LEGACY_SWIPE_IGNORE_SELECTOR: &str = "[data-swipe-ignore]";

/// The shape shared by the two collision-avoidance presets
/// (`packages/react/src/internals/constants.ts:18-28`): where an axis-collision fallback
/// should move the popup.
pub struct CollisionAvoidance {
    /// `fallbackAxisSide` — `'none'` (never fall back on the cross axis) or `'end'`.
    pub fallback_axis_side: &'static str,
}

/// `DROPDOWN_COLLISION_AVOIDANCE` (`packages/react/src/internals/constants.ts:14-20`) — for
/// dropdowns that strictly prefer top/bottom placements and use `var(--available-height)`
/// to limit their height.
pub const DROPDOWN_COLLISION_AVOIDANCE: CollisionAvoidance = CollisionAvoidance {
    fallback_axis_side: "none",
};

/// `POPUP_COLLISION_AVOIDANCE` (`packages/react/src/internals/constants.ts:22-28`) — for
/// regular popups that usually aren't scrollable and may freely flip to any axis.
pub const POPUP_COLLISION_AVOIDANCE: CollisionAvoidance = CollisionAvoidance {
    fallback_axis_side: "end",
};

/// `ownerVisuallyHidden` (`packages/react/src/internals/constants.ts:30-41`) — the special
/// visually hidden styles for the `aria-owns` owner element to ensure owned-element
/// accessibility in iOS/Safari/VoiceControl (the owner element is an empty span, so most
/// common visually hidden styles are not needed; see the linked floating-ui issue in the
/// upstream doc comment). Previously hosted provisionally in `floating_portal.rs`.
pub const OWNER_VISUALLY_HIDDEN: &[(&str, &str)] = &[
    ("clip-path", "inset(50%)"),
    ("position", "fixed"),
    ("top", "0px"),
    ("left", "0px"),
];

#[cfg(test)]
mod tests {
    use super::*;

    // Port-owned pins (`packages/react/src/internals/constants.ts` has no test file — the
    // implementation spec lists the module's exports under "Anything in source not explained
    // by any test", item 10): the constant values are pinned against the upstream literals
    // so a transcription slip cannot slip through silently.
    #[test]
    fn thresholds_match_upstream() {
        assert_eq!(TYPEAHEAD_RESET_MS, 500);
        assert_eq!(PATIENT_CLICK_THRESHOLD, 500);
    }

    #[test]
    fn the_disabled_transitions_style_carries_the_transition_none_declaration() {
        assert_eq!(DISABLED_TRANSITIONS_STYLE, &[("transition", "none")]);
    }

    #[test]
    fn the_swipe_ignore_attributes_and_selectors_match_upstream() {
        assert_eq!(BASE_UI_SWIPE_IGNORE_ATTRIBUTE, "data-base-ui-swipe-ignore");
        assert_eq!(LEGACY_SWIPE_IGNORE_ATTRIBUTE, "data-swipe-ignore");
        assert_eq!(BASE_UI_SWIPE_IGNORE_SELECTOR, "[data-base-ui-swipe-ignore]");
        assert_eq!(LEGACY_SWIPE_IGNORE_SELECTOR, "[data-swipe-ignore]");
    }

    #[test]
    fn the_collision_avoidance_presets_match_upstream() {
        assert_eq!(DROPDOWN_COLLISION_AVOIDANCE.fallback_axis_side, "none");
        assert_eq!(POPUP_COLLISION_AVOIDANCE.fallback_axis_side, "end");
    }

    #[test]
    fn the_owner_visually_hidden_style_matches_upstream() {
        assert_eq!(
            OWNER_VISUALLY_HIDDEN,
            &[
                ("clip-path", "inset(50%)"),
                ("position", "fixed"),
                ("top", "0px"),
                ("left", "0px"),
            ]
        );
    }
}
