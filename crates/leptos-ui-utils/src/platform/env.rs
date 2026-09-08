//! Port of `packages/utils/src/platform/env.ts` — the `env` flag group of the `platform`
//! namespace.

use super::shared::RawNavigatorData;

/// Upstream `env` namespace (`packages/utils/src/platform/env.ts:3-4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Env {
    /// Running in jsdom or HappyDOM (used by unit tests) (`env.ts:3-4`).
    pub jsdom: bool,
}

impl Env {
    /// `/jsdom|happydom/.test(lowerUserAgent)` (`packages/utils/src/platform/env.ts:4`) — an
    /// unanchored substring test, so the port uses two `contains` checks.
    pub(crate) fn detect(raw: &RawNavigatorData) -> Env {
        // Upstream lowers the UA once at module scope
        // (`packages/utils/src/platform/shared.ts:49`).
        let lower_user_agent = raw.user_agent.to_lowercase();
        Env {
            jsdom: lower_user_agent.contains("jsdom") || lower_user_agent.contains("happydom"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::shared::RawNavigatorData;

    fn raw(user_agent: &str) -> RawNavigatorData {
        RawNavigatorData {
            user_agent: user_agent.to_string(),
            platform: String::new(),
            max_touch_points: 0,
        }
    }

    // Implementation-derived (`packages/utils/src/platform/env.ts:4`): both markers of the
    // alternation match, unanchored, in either case.
    #[test]
    fn jsdom_and_happydom_user_agents_are_detected() {
        assert!(Env::detect(&raw("… jsdom/26.0.0 …")).jsdom);
        assert!(Env::detect(&raw("… happydom/15.0.0 …")).jsdom);
        assert!(Env::detect(&raw("… JSDOM/26.0.0 …")).jsdom);
    }

    // Implementation-derived (`packages/utils/src/platform/env.ts:4`): a real browser UA (and
    // SSR's empty one) matches nothing. Consumers use this flag to short-circuit browser-only
    // code paths under test
    // (`packages/react/src/floating-ui-react/utils/event.ts:26`,
    // `packages/react/src/floating-ui-react/utils/element.ts:73`,
    // `packages/react/src/utils/getPseudoElementBounds.ts:36`).
    #[test]
    fn real_browser_and_empty_user_agents_are_not_detected() {
        assert!(!Env::detect(&raw("mozilla/5.0 … chrome/140.0.0.0 safari/537.36")).jsdom);
        assert!(!Env::detect(&raw("")).jsdom);
    }
}
