//! Port of `packages/utils/src/platform/os.ts` — the `os` flag group of the `platform`
//! namespace.

use super::shared::RawNavigatorData;

/// Upstream `os` namespace (`packages/utils/src/platform/os.ts:7-24`): six booleans derived
/// once from the raw navigator data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Os {
    /// iPhone, iPad (including iPadOS 13+ reporting as macOS), iPod (`os.ts:6-8`).
    pub ios: bool,
    /// Android phones, tablets, and embedded Android browsers (`os.ts:10-12`).
    pub android: bool,
    /// macOS desktop. Excludes iPadOS, which reports as `MacIntel` (`os.ts:14-15`).
    pub mac: bool,
    /// Windows desktop (`os.ts:17-18`).
    pub windows: bool,
    /// Linux desktop (including Chrome OS) (`os.ts:20-21`).
    pub linux: bool,
    /// Any Apple OS (`mac || ios`) (`os.ts:23-24`).
    pub apple: bool,
}

impl Os {
    /// Derives the group from raw navigator data — pure, so the port's tests can drive it with
    /// synthetic inputs the way upstream consumer tests mock the whole group
    /// (`specs/utils/platform.md`, "Test-level evidence").
    ///
    /// The upstream JS regexes translate exactly:
    /// - `/^i(os$|p)/` matches iff the platform is exactly `"ios"` or starts with `"ip"`
    ///   (the `os$` alternative must end right there; the `p` alternative continues).
    /// - `/^(linux|chrome os)/` matches iff the platform starts with `"linux"` or
    ///   `"chrome os"`.
    pub(crate) fn detect(raw: &RawNavigatorData) -> Os {
        // Upstream lowers both strings once at module scope
        // (`packages/utils/src/platform/shared.ts:49-50`).
        let lower_user_agent = raw.user_agent.to_lowercase();
        let lower_platform = raw.platform.to_lowercase();
        let max_touch_points = raw.max_touch_points;

        // `/^i(os$|p)/.test(lowerPlatform) || (lowerPlatform === 'macintel' &&
        //  maxTouchPoints > 1)` (`packages/utils/src/platform/os.ts:7-8`). iPadOS 13+ reports
        // `MacIntel`; disambiguated via `maxTouchPoints` so iPad is classified as iOS, not
        // macOS (mui/base-ui#1309 — `os.ts:3-5`).
        let ios = lower_platform == "ios"
            || lower_platform.starts_with("ip")
            || (lower_platform == "macintel" && max_touch_points > 1);

        // `lowerPlatform === 'android' || lowerUserAgent.includes('android')` (`os.ts:11-12`).
        let android = lower_platform == "android" || lower_user_agent.contains("android");

        // `!ios && lowerPlatform.startsWith('mac')` (`os.ts:15`).
        let mac = !ios && lower_platform.starts_with("mac");

        // `lowerPlatform.startsWith('win')` (`os.ts:18`).
        let windows = lower_platform.starts_with("win");

        // `!android && /^(linux|chrome os)/.test(lowerPlatform)` (`os.ts:21`).
        let linux = !android
            && (lower_platform.starts_with("linux") || lower_platform.starts_with("chrome os"));

        // `mac || ios` (`os.ts:24`).
        let apple = mac || ios;

        Os {
            ios,
            android,
            mac,
            windows,
            linux,
            apple,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(user_agent: &str, platform: &str, max_touch_points: u32) -> RawNavigatorData {
        RawNavigatorData {
            user_agent: user_agent.to_string(),
            platform: platform.to_string(),
            max_touch_points,
        }
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:7-8`): the iOS regex and the
    // touch-point disambiguation. Upstream consumer tests override this group together with
    // `apple` (`packages/react/src/combobox/status/ComboboxStatus.iOS.test.tsx:7-18`,
    // `packages/react/src/number-field/root/NumberFieldRoot.iOS.test.tsx:6-17`), so `ios` must
    // imply `apple`.
    #[test]
    fn iphone_style_platforms_are_ios_and_apple() {
        for platform in ["iphone", "ipad", "ipod", "iPad Simulator"] {
            let os = Os::detect(&raw("", platform, 0));
            assert!(os.ios, "{platform} should be ios");
            assert!(os.apple, "{platform} should be apple");
        }
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:3-8`): iPadOS 13+ reports
    // `MacIntel` with touch support and must classify as iOS, not macOS (mui/base-ui#1309).
    #[test]
    fn macintel_with_multiple_touch_points_is_ios_not_mac() {
        let os = Os::detect(&raw("", "macintel", 2));
        assert!(os.ios);
        assert!(os.apple);
        assert!(!os.mac);
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:7-8`): the disambiguation is
    // strictly `maxTouchPoints > 1` — exactly one touch point does not make a Mac an iPad.
    #[test]
    fn macintel_with_at_most_one_touch_point_is_mac() {
        for max_touch_points in [0, 1] {
            let os = Os::detect(&raw("", "macintel", max_touch_points));
            assert!(!os.ios, "{max_touch_points} touch points should not be ios");
            assert!(os.mac);
            assert!(os.apple);
        }
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:15`): `mac` requires the
    // platform prefix and the absence of iOS.
    #[test]
    fn macintosh_platforms_are_mac() {
        for platform in ["mac", "macintel", "macintosh"] {
            let os = Os::detect(&raw("", platform, 0));
            assert!(os.mac, "{platform} should be mac");
        }
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:11-12`): Android matches the
    // platform string or the UA substring.
    #[test]
    fn android_matches_the_platform_or_the_user_agent() {
        let by_platform = Os::detect(&raw("", "android", 0));
        assert!(by_platform.android);

        let by_user_agent = Os::detect(&raw("mozilla… android 14; …", "linux x86_64", 0));
        assert!(by_user_agent.android);
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:21`): `linux` is anchored to
    // `!android`, so an Android UA on a Linux platform string classifies as Android only.
    #[test]
    fn android_suppresses_linux() {
        let os = Os::detect(&raw("android", "linux x86_64", 0));
        assert!(os.android);
        assert!(!os.linux);
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:20-21`): plain Linux and
    // Chrome OS both classify as Linux.
    #[test]
    fn linux_and_chrome_os_platforms_are_linux() {
        for platform in [
            "linux",
            "linux x86_64",
            "linux armv8l",
            "chrome os",
            "Chrome OS",
        ] {
            let os = Os::detect(&raw("", platform, 0));
            assert!(os.linux, "{platform} should be linux");
        }
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:18`): the `win` prefix.
    #[test]
    fn windows_platforms_are_windows() {
        for platform in ["win", "win32", "windows"] {
            let os = Os::detect(&raw("", platform, 0));
            assert!(os.windows, "{platform} should be windows");
            assert!(!os.apple);
            assert!(!os.linux);
        }
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:7-8`): the regex's `os$`
    // alternative must end right there — longer strings starting with "io" do not match.
    #[test]
    fn longer_io_strings_are_not_ios() {
        for platform in ["irix", "ios like", "ion"] {
            let os = Os::detect(&raw("", platform, 0));
            assert!(!os.ios, "{platform} should not be ios");
        }
    }

    // Spec, "Edge cases" (`packages/utils/src/platform/shared.ts:24-26`,
    // `packages/utils/src/platform/index.ts:4-7`): SSR's empty reads match nothing, so every
    // flag is false. The consumer mock contract additionally pairs `os` flags with `apple`
    // (`packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:12-23` mocks the
    // false variant `mac: false, apple: false`).
    #[test]
    fn ssr_empty_inputs_yield_all_flags_false() {
        let os = Os::detect(&RawNavigatorData::SSR_EMPTY);
        assert!(!os.ios);
        assert!(!os.android);
        assert!(!os.mac);
        assert!(!os.windows);
        assert!(!os.linux);
        assert!(!os.apple);
    }

    // Implementation-derived (`packages/utils/src/platform/os.ts:24`): `apple` derives from
    // `mac || ios` — no test upstream asserts the derivation (consumer tests only mock the
    // pairs), so the port pins it here.
    #[test]
    fn apple_derives_from_mac_or_ios() {
        assert!(Os::detect(&raw("", "macintel", 0)).apple);
        assert!(Os::detect(&raw("", "iphone", 0)).apple);
        assert!(!Os::detect(&raw("", "win32", 0)).apple);
        assert!(!Os::detect(&raw("", "linux x86_64", 0)).apple);
    }
}
