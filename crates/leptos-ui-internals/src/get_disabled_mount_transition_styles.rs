//! Port of `packages/react/src/internals/getDisabledMountTransitionStyles.ts` — the
//! mount-transition style suppressor (`TODO.md`, item `infra: internals`; the
//! transition/animation checkpoint recorded in that entry's note).
//!
//! Upstream maps `'starting'` to the shared `DISABLED_TRANSITIONS_STYLE` constant
//! (`getDisabledMountTransitionStyles.ts:5-9` over `constants.ts:5`) and everything else
//! to `EMPTY_OBJECT` — the props object a consumer spreads so a mounting element does not
//! run its own CSS transitions during the starting frame.
//!
//! The upstream return shape is `{ style?: CSSProperties | undefined }`; the port's
//! constants module already documents `DISABLED_TRANSITIONS_STYLE` as the style pair list
//! (`crate::constants`, where upstream's `{ style: { transition: 'none' } }` object
//! becomes `[("transition", "none")]`), so the return is the style pairs behind an
//! [`Option`] — `Some` is the `style` prop present, `None` is `EMPTY_OBJECT`.

use crate::constants::DISABLED_TRANSITIONS_STYLE;
use crate::use_transition_status::TransitionStatus;

/// The upstream `getDisabledMountTransitionStyles`
/// (`getDisabledMountTransitionStyles.ts:5-9`): `Some` (the `transition: none` pairs) only
/// while the transition status is `'starting'`.
pub fn get_disabled_mount_transition_styles(
    transition_status: Option<TransitionStatus>,
) -> Option<&'static [(&'static str, &'static str)]> {
    if transition_status == Some(TransitionStatus::Starting) {
        Some(DISABLED_TRANSITIONS_STYLE)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors the upstream mapping (`getDisabledMountTransitionStyles.ts:8`) across every
    // status the machine can emit (`useTransitionStatus.ts:6`, including `undefined`).
    #[test]
    fn only_the_starting_status_suppresses_transitions() {
        assert_eq!(
            get_disabled_mount_transition_styles(Some(TransitionStatus::Starting)),
            Some(&[("transition", "none")][..])
        );
        assert_eq!(
            get_disabled_mount_transition_styles(Some(TransitionStatus::Ending)),
            None
        );
        assert_eq!(
            get_disabled_mount_transition_styles(Some(TransitionStatus::Idle)),
            None
        );
        assert_eq!(get_disabled_mount_transition_styles(None), None);
    }
}
