//! Port of `packages/utils/src/platform/engine.ts` — the `engine` flag group of the `platform`
//! namespace.

use super::shared::RawNavigatorData;

/// Upstream `engine` namespace (`packages/utils/src/platform/engine.ts:7-22`): three booleans,
/// with `gecko`/`blink` anchored to `!webkit` so engines are mutually exclusive by
/// construction (`engine.ts:10`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Engine {
    /// WebKit: Safari, all iOS browsers, GNOME Web. Excludes Blink (`engine.ts:6-8`).
    pub webkit: bool,
    /// Gecko: Firefox (`engine.ts:14-15`).
    pub gecko: bool,
    /// Blink: Chrome, Edge, Opera, Brave, and other Chromium-based browsers (`engine.ts:21-22`).
    pub blink: bool,
}

/// `typeof CSS !== 'undefined' && !!CSS.supports?.('-webkit-backdrop-filter:none')`
/// (`packages/utils/src/platform/engine.ts:7-8`) — WebKit is distinguished from Blink by the
/// legacy `-webkit-backdrop-filter` name, which Blink (forked 2013) does not ship
/// (`engine.ts:3-5`).
///
/// The `web_sys::css::supports` binding mirrors the whole upstream expression: a realm without
/// a `CSS` global throws inside the generated binding (the caught `Err`), the optional-call
/// `?.` and truthiness coercion map to `unwrap_or(false)`.
pub(crate) fn supports_webkit_backdrop_filter() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return false;

    #[cfg(target_arch = "wasm32")]
    web_sys::css::supports("-webkit-backdrop-filter:none").unwrap_or(false)
}

impl Engine {
    /// Derives the group from raw navigator data plus the one-shot `CSS.supports` probe result.
    /// Pure, so the port's tests can drive the `!webkit` anchoring with synthetic inputs.
    pub(crate) fn detect(raw: &RawNavigatorData, webkit: bool) -> Engine {
        // Upstream lowers the UA once at module scope
        // (`packages/utils/src/platform/shared.ts:49`).
        let lower_user_agent = raw.user_agent.to_lowercase();

        Engine {
            webkit,
            // `!webkit && lowerUserAgent.includes('firefox')` — anchored to `!webkit` so
            // Firefox-on-iOS (WebKit-based, UA marker `FxiOS/`) classifies as WebKit
            // (`packages/utils/src/platform/engine.ts:10-15`).
            gecko: !webkit && lower_user_agent.contains("firefox"),
            // `!webkit && lowerUserAgent.includes('chrom')` — one substring covers
            // Chrome/Chromium/Edge/Opera/Brave; Chrome-on-iOS `CriOS/` stays WebKit; the
            // positive UA check also makes this SSR-safe (`engine.ts:17-22`).
            blink: !webkit && lower_user_agent.contains("chrom"),
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

    // Implementation-derived (`packages/utils/src/platform/engine.ts:7-8`): `webkit` is exactly
    // the CSS-supports probe result, nothing else.
    #[test]
    fn webkit_is_taken_from_the_css_supports_probe() {
        assert!(Engine::detect(&raw(""), true).webkit);
        assert!(!Engine::detect(&raw(""), false).webkit);
    }

    // Implementation-derived (`packages/utils/src/platform/engine.ts:14-15`): a Firefox UA with
    // WebKit absent is Gecko (and not Blink).
    #[test]
    fn a_firefox_user_agent_without_webkit_is_gecko() {
        let engine = Engine::detect(&raw("mozilla/5.0 … firefox/140.0"), false);
        assert!(engine.gecko);
        assert!(!engine.blink);
        assert!(!engine.webkit);
    }

    // Implementation-derived (`packages/utils/src/platform/engine.ts:10-13`): Firefox-on-iOS
    // uses WebKit; even if a WebKit-based UA ever injected `Firefox`, the `!webkit` anchor
    // keeps it classified as WebKit.
    #[test]
    fn a_firefox_marker_cannot_overcome_webkit() {
        let engine = Engine::detect(&raw("… fxcios/… firefox/…"), true);
        assert!(engine.webkit);
        assert!(!engine.gecko);
        assert!(!engine.blink);
    }

    // Implementation-derived (`packages/utils/src/platform/engine.ts:17-22`): the shared `chrom`
    // substring covers every Chromium-based browser's UA marker.
    #[test]
    fn chromium_markers_without_webkit_are_blink() {
        for user_agent in [
            "… chrome/140.0.0.0 …",
            "… chromium/140.0.0.0 …",
            "… edg/140.0.0.0 … chrome/140.0.0.0 …",
        ] {
            let engine = Engine::detect(&raw(user_agent), false);
            assert!(engine.blink, "{user_agent} should be blink");
            assert!(!engine.gecko);
        }
    }

    // Implementation-derived (`packages/utils/src/platform/engine.ts:17-22`): Chrome-on-iOS
    // uses `CriOS/` and stays WebKit.
    #[test]
    fn a_chromium_marker_cannot_overcome_webkit() {
        let engine = Engine::detect(&raw("… crios/140.0.0.0 …"), true);
        assert!(engine.webkit);
        assert!(!engine.blink);
        assert!(!engine.gecko);
    }

    // Spec, "Edge cases" (`packages/utils/src/platform/engine.ts:15,22`): `gecko` and `blink`
    // are NOT mutually exclusive with each other by construction — a UA containing both markers
    // sets both.
    #[test]
    fn gecko_and_blink_are_not_mutually_exclusive_with_each_other() {
        let engine = Engine::detect(&raw("… firefox/140.0 … chrom/140.0 …"), false);
        assert!(engine.gecko);
        assert!(engine.blink);
    }

    // Implementation-derived (`packages/utils/src/platform/engine.ts:20-22`): an empty UA (SSR)
    // matches nothing, so without WebKit every flag stays false.
    #[test]
    fn an_empty_user_agent_yields_no_gecko_or_blink() {
        let engine = Engine::detect(&raw(""), false);
        assert!(!engine.webkit);
        assert!(!engine.gecko);
        assert!(!engine.blink);
    }

    // Spec, "Edge cases": WebKit anchors the other two, so a WebKit browser can never classify
    // as Gecko or Blink regardless of the UA (`packages/utils/src/platform/engine.ts:10-22`).
    #[test]
    fn webkit_excludes_gecko_and_blink_by_construction() {
        let engine = Engine::detect(&raw("chrome/140.0.0.0 firefox/140.0"), true);
        assert!(engine.webkit);
        assert!(!engine.gecko);
        assert!(!engine.blink);
    }
}
