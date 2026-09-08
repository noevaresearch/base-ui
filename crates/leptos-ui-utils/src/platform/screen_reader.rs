//! Port of `packages/utils/src/platform/screen-reader.ts` — the `screenReader` flag group of
//! the `platform` namespace.

/// Upstream `screenReader` namespace (`packages/utils/src/platform/screen-reader.ts:8-12`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenReader {
    /// The user *may* be using VoiceOver — actual activation is not detectable. True on any
    /// Apple platform (macOS, iOS, iPadOS) (`screen-reader.ts:8-12`).
    pub voice_over: bool,
}

impl ScreenReader {
    /// `voiceOver = apple` (`packages/utils/src/platform/screen-reader.ts:12`) — a pure OS
    /// check: whether a screen reader is *actually* running cannot be detected, and
    /// engine-specific quirks are meant to be gated at call sites (`screen-reader.ts:3-7`).
    ///
    /// Takes the derived `apple` flag directly, mirroring upstream's import of `os.apple`
    /// (`screen-reader.ts:1`).
    pub(crate) fn detect(apple: bool) -> ScreenReader {
        ScreenReader { voice_over: apple }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implementation-derived (`packages/utils/src/platform/screen-reader.ts:12`): the flag is
    // purely `apple` re-exported — no test upstream asserts the derivation (the consumer test
    // mocks `voiceOver` directly,
    // `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:15`), so
    // the port pins it here for each OS classification.
    #[test]
    fn voice_over_is_exactly_the_apple_flag() {
        // macOS
        assert!(ScreenReader::detect(true).voice_over);
        // Windows / Linux / Android / SSR
        assert!(!ScreenReader::detect(false).voice_over);
    }
}
