//! Port of `packages/utils/src/platform/media-query.ts` — the `mediaQuery` group of the
//! `platform` namespace.

/// `@supports (-webkit-touch-callout: none)` — a CSS query matching iOS/iPadOS WebKit browsers
/// (`packages/utils/src/platform/media-query.ts:3-4`).
///
/// A plain constant string in every environment (`packages/utils/src/platform/index.ts:6-7`):
/// it is evaluated by the browser at style-resolution time, not by JavaScript, so there is no
/// detection logic to port.
pub(crate) const IOS_QUERY: &str = "@supports (-webkit-touch-callout: none)";

/// Upstream `mediaQuery` namespace (`packages/utils/src/platform/media-query.ts`).
///
/// Unlike the boolean groups, this group exposes CSS query *strings* the consumer matches
/// against (`packages/utils/src/platform/index.ts:6-7`). No file in the repo consumes
/// `platform.mediaQuery.iOS` today (`specs/utils/platform.md`, "Public API surface") — the
/// port keeps it for API-shape parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaQuery {
    /// CSS `@supports` query matching iOS/iPadOS WebKit browsers (`media-query.ts:3-4`).
    pub ios: &'static str,
}

impl MediaQuery {
    /// The group is one constant — same value in every environment
    /// (`packages/utils/src/platform/index.ts:6-7`).
    pub(crate) fn detect() -> MediaQuery {
        MediaQuery { ios: IOS_QUERY }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implementation-derived (`packages/utils/src/platform/media-query.ts:4`): the exact
    // upstream string, byte for byte.
    #[test]
    fn the_ios_query_is_the_exact_upstream_constant() {
        assert_eq!(
            MediaQuery::detect().ios,
            "@supports (-webkit-touch-callout: none)"
        );
    }
}
