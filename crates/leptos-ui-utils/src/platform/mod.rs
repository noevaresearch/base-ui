//! Port of `packages/utils/src/platform/` — static platform detection, evaluated once.
//!
//! Upstream is a directory of five group modules aggregated by `parts.ts` and exposed as the
//! single named export `platform` (`packages/utils/src/platform/index.ts:32`,
//! `packages/utils/src/platform/parts.ts:1-5`): `os`, `engine`, `screenReader`, `env`, and
//! `mediaQuery`. The internal `shared.ts` helpers are not re-exported (`parts.ts:1-5`), so
//! they stay crate-private here too (`specs/utils/platform.md`, "Public API surface").
//!
//! The Rust shape mirrors the upstream access path (`platform.os.mac`, `platform.engine.webkit`,
//! …) as `platform().os.mac` — one accessor returning a shared, immutable aggregate:
//!
//! ```ignore
//! use leptos_ui_utils::platform;
//! if platform().os.mac { … }
//! if platform().engine.webkit { … }
//! ```
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream computes every flag in `const` module initializers at import time
//!   (`packages/utils/src/platform/index.ts:2`,
//!   `packages/utils/src/platform/shared.ts:48-51`). Rust has no module-load-time evaluation,
//!   so the aggregate is built once inside a `LazyLock` on first access. That preserves the
//!   observable contract the upstream consumers rely on — the flags are stable for the whole
//!   session and later mutations of `navigator` have no effect on them
//!   (`specs/utils/platform.md`, "State model") — and is the standard Rust equivalent of a
//!   module-load-time constant.
//! - On a non-wasm host target (no JS realm at all — the SSR situation,
//!   `packages/utils/src/platform/shared.ts:24-26`) the aggregate evaluates to all-false flags
//!   with `mediaQuery` unchanged, mirroring the documented SSR safety
//!   (`packages/utils/src/platform/index.ts:4-7`).
//! - Upstream keeps each category in its own module so unused namespaces tree-shake away
//!   (`index.ts:22-23`); that concern does not exist in Rust, so the groups collapse into one
//!   crate-private module tree re-exported through this one.

use std::sync::LazyLock;

mod engine;
mod env;
mod media_query;
mod os;
mod screen_reader;
mod shared;

pub use engine::Engine;
pub use env::Env;
pub use media_query::MediaQuery;
pub use os::Os;
pub use screen_reader::ScreenReader;

/// The aggregate behind the upstream `platform` namespace import
/// (`packages/utils/src/platform/index.ts:32`): the five groups of `parts.ts:1-5`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    /// OS-specific behavior (keyboard shortcuts, native menus) (`index.ts:25`).
    pub os: Os,
    /// Rendering-engine bugs (CSS, layout, focus) (`index.ts:26`).
    pub engine: Engine,
    /// AT-specific accessibility workarounds (`index.ts:27`).
    pub screen_reader: ScreenReader,
    /// Test-environment gating (`index.ts:28`).
    pub env: Env,
    /// CSS-side platform detection (`@supports`/`@media` query strings) (`index.ts:29`).
    pub media_query: MediaQuery,
}

/// The upstream `platform` export (`packages/utils/src/platform/index.ts:32`): the static
/// detection namespace, built once from the real navigator (or the SSR-empty shape where no
/// realm exists) and shared for the rest of the session.
pub fn platform() -> &'static Platform {
    static PLATFORM: LazyLock<Platform> = LazyLock::new(|| {
        let raw = shared::read_raw_data();
        // `screenReader` derives from `os` (`packages/utils/src/platform/screen-reader.ts:1,12`)
        // and `engine`'s `gecko`/`blink` anchor to its own CSS-supports probe
        // (`packages/utils/src/platform/engine.ts:7-22`); each group is computed exactly once
        // (`specs/utils/platform.md`, "State model").
        let os = Os::detect(&raw);
        Platform {
            os,
            engine: Engine::detect(&raw, engine::supports_webkit_backdrop_filter()),
            screen_reader: ScreenReader::detect(os.apple),
            env: Env::detect(&raw),
            media_query: MediaQuery::detect(),
        }
    });
    &PLATFORM
}

#[cfg(test)]
mod tests {
    use super::*;

    // Spec, "Edge cases" (`packages/utils/src/platform/shared.ts:24-26`,
    // `packages/utils/src/platform/index.ts:4-7`): with no navigator, every derived flag is
    // false while `mediaQuery` stays a constant string. A non-wasm host target has no JS realm
    // at all — exactly that SSR situation — so the host build can pin the whole contract.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_realm_without_a_navigator_yields_all_false_flags_and_the_constant_media_query() {
        let platform = platform();

        assert!(!platform.os.ios);
        assert!(!platform.os.android);
        assert!(!platform.os.mac);
        assert!(!platform.os.windows);
        assert!(!platform.os.linux);
        assert!(!platform.os.apple);

        assert!(!platform.engine.webkit);
        assert!(!platform.engine.gecko);
        assert!(!platform.engine.blink);

        assert!(!platform.screen_reader.voice_over);
        assert!(!platform.env.jsdom);

        assert_eq!(
            platform.media_query.ios,
            "@supports (-webkit-touch-callout: none)"
        );
    }

    // Spec, "State model" (`packages/utils/src/platform/index.ts:2`,
    // `packages/utils/src/platform/shared.ts:48-51`): the namespace is static module data —
    // computed once, stable for the session, immune to later `navigator` changes. The
    // `LazyLock`'s returned reference is shared, so every access observes the same snapshot.
    #[test]
    fn repeated_accesses_return_the_same_stable_snapshot() {
        assert_eq!(platform(), platform());
    }

    // The namespace's documented invariants (`packages/utils/src/platform/os.ts:24`,
    // `packages/utils/src/platform/screen-reader.ts:12`,
    // `packages/utils/src/platform/engine.ts:10`): `apple` derives from `mac || ios`,
    // `voiceOver` from `apple`, and WebKit excludes Gecko/Blink by construction. Asserting them
    // on the real aggregate checks the static-init wiring end to end.
    #[test]
    fn the_aggregate_satisfies_the_upstream_invariants() {
        let platform = platform();
        assert_eq!(platform.os.apple, platform.os.mac || platform.os.ios);
        assert_eq!(platform.screen_reader.voice_over, platform.os.apple);
        if platform.engine.webkit {
            assert!(!platform.engine.gecko);
            assert!(!platform.engine.blink);
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // The behavior under test is the real read path — a browser realm's `navigator` and
    // `CSS.supports` — so like the crate's other wasm test modules this runs in a real browser
    // (`.cargo/config.toml` wires the wasm32 runner to chromedriver).
    //
    // Spec, "State model" (`packages/utils/src/platform/index.ts:2`): the namespace is built
    // once from real data and stays stable; two accesses observe the identical snapshot.
    #[wasm_bindgen_test]
    fn the_browser_aggregate_is_stable_across_accesses() {
        assert_eq!(platform(), platform());
    }

    // The aggregate's invariants must hold for the real environment's data too — this checks
    // the static-init wiring (`os` feeding `screenReader`, the probe feeding `engine`), not
    // just the pure derivations.
    #[wasm_bindgen_test]
    fn the_browser_aggregate_satisfies_the_upstream_invariants() {
        let platform = platform();
        assert_eq!(platform.os.apple, platform.os.mac || platform.os.ios);
        assert_eq!(platform.screen_reader.voice_over, platform.os.apple);
        if platform.engine.webkit {
            assert!(!platform.engine.gecko);
            assert!(!platform.engine.blink);
        }
    }

    // Environment-specific (chromedriver runs Chrome): Blink classifies as Blink and not
    // WebKit. Upstream's own Chromium-runner suites depend on exactly these values — e.g.
    // `skipIf(isJSDOM || !platform.engine.blink)` passes in Chromium
    // (`packages/react/src/dialog/root/DialogRoot.test.tsx:42`) and the WebKit-gated suites
    // skip there (`packages/react/src/navigation-menu/root/NavigationMenuRoot.webkit.test.tsx:8-19`).
    #[wasm_bindgen_test]
    fn the_chromium_test_browser_classifies_as_blink() {
        let platform = platform();
        assert!(platform.engine.blink);
        assert!(!platform.engine.webkit);
        assert!(!platform.engine.gecko);
    }
}
