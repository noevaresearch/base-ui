//! Port of `packages/utils/src/platform/shared.ts` — the internal raw-data reader feeding every
//! `platform` group.
//!
//! This module is internal in upstream too: `shared.ts` exports `lowerUserAgent`,
//! `lowerPlatform`, and `maxTouchPoints`, but `parts.ts` re-exports only the five group
//! namespaces, so none of these are part of the public `platform` namespace
//! (`packages/utils/src/platform/parts.ts:1-5`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream lowers the raw strings once at module scope and re-exports the lowered copies
//!   (`packages/utils/src/platform/shared.ts:48-51`). The port keeps the raw values in
//!   `RawNavigatorData` and lowers per derivation call in each group's `detect` — same inputs,
//!   same results, but lets the derivation stay a pure function the tests can drive with
//!   synthetic data.
//! - `readRawData` is cfg-split: on `wasm32` it reads the real navigator; on a non-wasm host
//!   target there is no JS realm at all — the exact situation upstream's SSR guard handles —
//!   so it returns the same empty/zero shape instead of panicking inside a wasm-bindgen import.

#[cfg(target_arch = "wasm32")]
use js_sys::Reflect;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::Navigator;

/// Upstream `RawNavigatorData` (`packages/utils/src/platform/shared.ts:7-11`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawNavigatorData {
    pub user_agent: String,
    pub platform: String,
    pub max_touch_points: u32,
}

impl RawNavigatorData {
    /// The SSR branch's shape (`packages/utils/src/platform/shared.ts:24-26`):
    /// `{ userAgent: '', platform: '', maxTouchPoints: 0 }` — substring checks match nothing,
    /// so every derived flag is `false` (`packages/utils/src/platform/index.ts:4-7`).
    pub(crate) const SSR_EMPTY: RawNavigatorData = RawNavigatorData {
        user_agent: String::new(),
        platform: String::new(),
        max_touch_points: 0,
    };
}

/// Upstream `readRawData()` (`packages/utils/src/platform/shared.ts:23-46`).
///
/// - SSR (`typeof navigator === 'undefined'`, `shared.ts:24-26`): empty/zero values. On a
///   non-wasm host target there is no JS realm at all, which is that same situation; on wasm,
///   `web_sys::window()` being `None` covers a wasm runtime without a browser realm.
/// - Dev builds (`process.env.NODE_ENV !== 'production'`, `shared.ts:28-39`): prefer the modern
///   `navigator.userAgentData` API on Chromium — when it is present and its `brands` is an
///   array — formatting brands as `brand/version` joined by spaces to avoid DevTools warnings
///   about the deprecated legacy reads. Upstream gates this branch on the bundler-level
///   `NODE_ENV`; the Rust equivalent of "development build" is `cfg!(debug_assertions)`, so
///   release builds always take the legacy path, exactly like upstream production builds.
///   (Derived flags are unaffected either way: Chromium brand strings like "Google Chrome/…"
///   contain the same `chrom` substring the legacy UA does, and `userAgentData.platform` is
///   macOS/Windows/Linux/Chrome OS — a `mac`/`win`/`linux` prefix or `chrome os` prefix match,
///   same as the legacy `navigator.platform` values.)
/// - Production (`shared.ts:41-45`): always read the legacy `navigator.userAgent` /
///   `navigator.platform` / `navigator.maxTouchPoints`.
pub(crate) fn read_raw_data() -> RawNavigatorData {
    #[cfg(not(target_arch = "wasm32"))]
    return RawNavigatorData::SSR_EMPTY;

    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Array;

        let Some(window) = web_sys::window() else {
            return RawNavigatorData::SSR_EMPTY;
        };
        let navigator = window.navigator();

        if cfg!(debug_assertions) {
            // `navigator.userAgentData` is Chromium-only; in other engines the property is
            // `undefined`, which the Reflect read surfaces as a non-object — falling through to
            // the legacy read the way upstream's `uaData &&` guard does (`shared.ts:32`).
            let ua_data = Reflect::get(navigator.as_ref(), &JsValue::from_str("userAgentData"))
                .unwrap_or(JsValue::UNDEFINED);

            // `uaData && Array.isArray(uaData.brands)` (`shared.ts:32`).
            if ua_data.is_object() {
                let brands = Reflect::get(&ua_data, &JsValue::from_str("brands"))
                    .unwrap_or(JsValue::UNDEFINED);
                if Array::is_array(&brands) {
                    // `uaData.brands.map(({ brand, version }) => `${brand}/${version}`)
                    //  .join(' ')` (`shared.ts:34`).
                    let user_agent = Array::from(&brands)
                        .iter()
                        .map(|brand_version| {
                            format!(
                                "{}/{}",
                                property_string(&brand_version, "brand").unwrap_or_default(),
                                property_string(&brand_version, "version").unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    // `uaData.platform ?? navigator.platform ?? ''` (`shared.ts:35`).
                    let platform = property_string(&ua_data, "platform")
                        .or_else(|| navigator.platform().ok())
                        .unwrap_or_default();
                    // `navigator.maxTouchPoints ?? 0` (`shared.ts:36`).
                    return RawNavigatorData {
                        user_agent,
                        platform,
                        max_touch_points: navigator_max_touch_points(&navigator),
                    };
                }
            }
        }

        // Legacy read (`shared.ts:41-45`). `navigator.userAgent` and `navigator.platform` are
        // WebIDL `DOMString` attributes — always present in a real browser — so
        // `unwrap_or_default` only stands in for the `?? ''`-style fallbacks upstream wrote for
        // robustness (`shared.ts:35-36,43-44`).
        RawNavigatorData {
            user_agent: navigator.user_agent().unwrap_or_default(),
            platform: navigator.platform().unwrap_or_default(),
            max_touch_points: navigator_max_touch_points(&navigator),
        }
    }
}

/// Reads a string property, treating a missing/null one as absent — the property-access
/// equivalent of upstream's `??` chains (`packages/utils/src/platform/shared.ts:35`).
#[cfg(target_arch = "wasm32")]
fn property_string(object: &JsValue, property: &str) -> Option<String> {
    Reflect::get(object, &JsValue::from_str(property))
        .ok()
        .and_then(|value| value.as_string())
}

/// `navigator.maxTouchPoints ?? 0` (`packages/utils/src/platform/shared.ts:36,44`): an absent
/// (undefined/null) property reads as zero.
#[cfg(target_arch = "wasm32")]
fn navigator_max_touch_points(navigator: &Navigator) -> u32 {
    Reflect::get(navigator.as_ref(), &JsValue::from_str("maxTouchPoints"))
        .ok()
        .filter(|value| !value.is_undefined() && !value.is_null())
        .and_then(|value| value.as_f64())
        .map_or(0, |points| points as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implementation-derived (`packages/utils/src/platform/shared.ts:24-26`): the SSR shape is
    // exactly empty strings and zero touch points (`specs/utils/platform.md`, "Edge cases").
    #[test]
    fn ssr_empty_raw_data_is_empty_strings_and_zero_touch_points() {
        assert_eq!(RawNavigatorData::SSR_EMPTY.user_agent, "");
        assert_eq!(RawNavigatorData::SSR_EMPTY.platform, "");
        assert_eq!(RawNavigatorData::SSR_EMPTY.max_touch_points, 0);
    }

    // In a real browser realm the read must see the actual navigator — never the SSR-empty
    // shape (`packages/utils/src/platform/shared.ts:24-26` guard only fires when the realm has
    // no navigator). A real Chrome always carries a user agent (legacy or, in dev builds with
    // `userAgentData` present, the brands string) and a platform string.
    #[cfg(all(test, target_arch = "wasm32"))]
    mod wasm_tests {
        use wasm_bindgen::UnwrapThrowExt;
        use wasm_bindgen_test::wasm_bindgen_test;

        use super::*;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        fn a_real_browser_realm_reads_the_actual_navigator() {
            let raw = read_raw_data();
            assert!(!raw.user_agent.is_empty());
            assert!(!raw.platform.is_empty());
            // Zero touch points is the desktop default, not the absent marker — the real
            // property is always present in Chrome, so the `?? 0` fallback
            // (`packages/utils/src/platform/shared.ts:36,44`) must not have fired on a missing
            // property.
            let navigator = web_sys::window().unwrap_throw().navigator();
            assert_eq!(
                raw.max_touch_points,
                navigator.max_touch_points().max(0) as u32
            );
        }
    }

    // On a non-wasm host target there is no JS realm, which is upstream's SSR situation
    // (`packages/utils/src/platform/shared.ts:24-26`) — the read must return the empty shape
    // rather than panic, so every derived flag is `false`
    // (`packages/utils/src/platform/index.ts:4-7`).
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_host_without_a_js_realm_reads_the_ssr_empty_shape() {
        assert_eq!(read_raw_data(), RawNavigatorData::SSR_EMPTY);
    }
}
