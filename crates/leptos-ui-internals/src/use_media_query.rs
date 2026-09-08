//! Port of `packages/react/src/unstable-use-media-query/index.ts` (Base UI Phase A infra unit,
//! the `infra: unstable-use-media-query` TODO item).
//!
//! Upstream is a single 90-line module exporting one hook, `useMediaQuery(query, options)`
//! (`packages/react/src/unstable-use-media-query/index.ts:5`), which reports whether a CSS
//! media query currently matches and tracks the browser's `change` events on the underlying
//! `MediaQueryList`. It renders no UI of its own and has no upstream test file
//! (`specs/library/unstable-use-media-query/behavior.md`, "Shared harness dependencies") — the
//! tests below pin the ported contract itself, the established precedent for source-only units
//! (the `use_scroll_lock`/`use_interval` ports).
//!
//! ## Upstream mechanics being ported
//!
//! - The `@media` prefix strip (`packages/react/src/unstable-use-media-query/index.ts:13`):
//!   `/^@media( ?)/m` removes the first `@media` (plus at most one following space) found at a
//!   line start, so both `"(min-width: 600px)"` and `"@media (min-width: 600px)"` are accepted
//!   forms; every consumer of the query value — the custom implementations and the ambient one
//!   alike — sees the stripped form (`:26,30,41`).
//! - The effective `matchMedia` resolution (`:15-20`): the `matchMedia` option overrides the
//!   ambient `window.matchMedia` when provided (documented for iframe content windows,
//!   `:66-70`); without a window or `window.matchMedia` — the defensive `supportMatchMedia`
//!   check (`:6-11`) — the effective implementation is `null`.
//! - The snapshot tiers (`:24-34`, the `getServerSnapshot` selection): under `noSsr` with a
//!   working `matchMedia` the server snapshot is a live read per call (`:25-27`); otherwise an
//!   `ssrMatchMedia` stand-in's result, captured once per machinery run (`:29-32`); otherwise
//!   `defaultMatches` (default `false`, `:16,60-65`).
//! - The client machinery (`:36-47`): with a `matchMedia`, one `MediaQueryList` per machinery
//!   run (`:41`), the snapshot re-reads `mediaQueryList.matches` (`:44`), and `subscribe` wires
//!   change notifications through the `addEventListener` util (`:45`); with
//!   `matchMedia === null` the result is pinned to `defaultMatches` and never changes, with a
//!   no-op subscription (`:37-39`). The deps array `[getDefaultSnapshot, matchMedia, query]`
//!   (`:47`) re-creates the machinery when the query changes; the previous list is simply
//!   dropped and React re-subscribes (`specs/library/unstable-use-media-query/behavior.md`,
//!   "Edge cases").
//!
//! ## Rust adaptations (behavior-preserving where the upstream contract is defined)
//!
//! - The hook returns a [`Signal<bool>`] instead of a bare `boolean`: React re-renders on every
//!   `change` notification and re-reads `mediaQueryList.matches`, while a Leptos component body
//!   runs once, so the observable state must live in a signal the consumer can track. The
//!   signal is written at setup and on every `change` event.
//! - `query` is a reactive source (a [`Get`] of `String`) rather than a per-render argument: a
//!   change re-runs the machinery — fresh `MediaQueryList`, fresh subscription, previous list's
//!   subscription torn down — the deps-array contract (`:47`). The remaining deps entries
//!   (`defaultMatches` via `getDefaultSnapshot`, the `matchMedia` identity) are plain values
//!   here: [`UseMediaQueryOptions`] is fixed for the hook call, and re-creating the machinery
//!   because an *option's identity* changed has no Leptos consumer analog.
//! - The snapshot-tier split maps onto the port's two execution targets, following the crate's
//!   realm convention (`platform`'s "SSR empty shape", the `use_scroll_lock` port's no-realm
//!   binding): a target with a JS realm (wasm in a browser) is the client analog — the value is
//!   the live `MediaQueryList.matches`, read synchronously during setup and re-read on every
//!   `change` event; a target with no JS realm (the host build) is the SSR analog — the value
//!   is the server-snapshot tier exactly as upstream's SSR render resolves it (`:24-34`):
//!   `ssrMatchMedia`'s captured result when provided, else `defaultMatches`, static forever.
//!   Upstream's post-hydration client re-render has no equivalent in a non-hydrating target.
//! - Consequently [`UseMediaQueryOptions::no_ssr`] and
//!   [`UseMediaQueryOptions::ssr_match_media`] are realm-gated: `noSsr` exists to pick the
//!   hydration server snapshot (`:71-78`), and the port has no hydration pass on either target
//!   (the host never has a `matchMedia`, and the browser target always reads live), so it is
//!   accepted for API-shape parity and inert; `ssrMatchMedia` is consulted only on the no-realm
//!   target — upstream consulted it only via `getServerSnapshot` too, so a pure client-side
//!   React render also never observes it (`packages/react/src/unstable-use-media-query/
//!   index.ts:49` with a non-hydrating mount).
//! - Upstream's `matchMedia?: typeof window.matchMedia | undefined` admits an explicit `null`
//!   in JS despite the type, which forces the static, never-subscribing behavior of `:37-39`
//!   even in a real browser (the destructured default `:17` only fills `undefined`).
//!   [`MatchMediaSource`] makes all three cases representable: [`MatchMediaSource::Ambient`]
//!   (the default; `window.matchMedia` when the realm has one), [`MatchMediaSource::Custom`]
//!   (the option form), and [`MatchMediaSource::Disabled`] (the explicit-`null` form).
//! - The custom `matchMedia` type is [`MatchMediaFn`] = `Rc<dyn Fn(&str) -> MediaQueryList>`
//!   (upstream: a function returning a `MediaQueryList`, `:70`), receiving the *stripped* query
//!   (`:26,41`), and [`SsrMatchMediaFn`] is `Rc<dyn Fn(&str) -> bool>` — upstream's
//!   `(query) => { matches: boolean }` stand-in (`:82`) flattened to the only field the hook
//!   reads. The ambient path goes through `web_sys`'s `Window::match_media` binding, whose
//!   `Ok(None)` result (a `MediaQueryList` that per the DOM spec is never null) is treated as a
//!   hard error rather than silently degrading to the static behavior.
//! - `useSyncExternalStore` (`:49`) dissolves into the signal plus the
//!   [`use_iso_layout_effect`] machinery: the effect's first run is synchronous during setup
//!   (`specs/architecture.md`, "Layout effect"), so the live read is in place before the hook
//!   returns, mirroring `getSnapshot`'s render-time read; re-runs are driven by tracked reads
//!   of `query`; and [`on_cleanup`] plays the role of the store-subscription teardown, removing
//!   the `change` listener before each re-run and on owner disposal.
//! - The dev-only `useDebugValue` (`:51-54`) is N/A — there is no React DevTools in Rust.
//! - `'use client'` (`:1`) is N/A — there is no React Server Components boundary in Rust.
//!
//! Must be called inside a reactive owner (a component), like the other hook ports.

use std::rc::Rc;

use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;

use leptos_ui_utils::add_event_listener;
use leptos_ui_utils::use_iso_layout_effect;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsValue, UnwrapThrowExt};

use web_sys::MediaQueryList;

/// A custom `matchMedia` implementation — upstream's `matchMedia?: typeof window.matchMedia`
/// option (`packages/react/src/unstable-use-media-query/index.ts:70`): a function from the
/// (already `@media`-stripped) query string to a live `MediaQueryList`.
pub type MatchMediaFn = Rc<dyn Fn(&str) -> MediaQueryList>;

/// A server-side `matchMedia` stand-in — upstream's
/// `ssrMatchMedia?: (query: string) => { matches: boolean }` option
/// (`packages/react/src/unstable-use-media-query/index.ts:82`), flattened to the only field the
/// hook reads. Receives the stripped query; its result is captured once per machinery run
/// (`:29-32`).
pub type SsrMatchMediaFn = Rc<dyn Fn(&str) -> bool>;

/// Where the hook's `MediaQueryList` comes from — upstream's `matchMedia` option plus its
/// ambient default (`packages/react/src/unstable-use-media-query/index.ts:15-20`):
///
/// - [`MatchMediaSource::Ambient`]: `window.matchMedia` when the realm has a window with a
///   `matchMedia` function, `null` otherwise — the upstream default expression
///   `supportMatchMedia ? window.matchMedia : null` (`:17`).
/// - [`MatchMediaSource::Custom`]: the `matchMedia` option (`:66-70`), documented upstream for
///   handling an iframe content window.
/// - [`MatchMediaSource::Disabled`]: the explicit-`null` form JS callers can pass despite the
///   type — the static, never-subscribing behavior of `:37-39` even in a real browser.
pub enum MatchMediaSource {
    Ambient,
    Custom(MatchMediaFn),
    Disabled,
}

impl Default for MatchMediaSource {
    fn default() -> Self {
        MatchMediaSource::Ambient
    }
}

/// Upstream `useMediaQuery.Options`
/// (`packages/react/src/unstable-use-media-query/index.ts:59-83`).
#[derive(Default)]
pub struct UseMediaQueryOptions {
    /// Value used when no real match information is available — upstream `defaultMatches`
    /// (`packages/react/src/unstable-use-media-query/index.ts:60-65`, default `false` at
    /// `:16`).
    pub default_matches: bool,
    /// Where the `MediaQueryList` comes from — see [`MatchMediaSource`].
    pub match_media: MatchMediaSource,
    /// Server-side stand-in — upstream `ssrMatchMedia`
    /// (`packages/react/src/unstable-use-media-query/index.ts:79-82`). Consulted only on a
    /// target with no JS realm (the port's SSR analog); its result is captured once. See the
    /// module docs' realm-gating adaptation.
    pub ssr_match_media: Option<SsrMatchMediaFn>,
    /// Upstream `noSsr` (`packages/react/src/unstable-use-media-query/index.ts:71-78`).
    /// Accepted for API-shape parity and inert in this port — see the module docs'
    /// realm-gating adaptation.
    pub no_ssr: bool,
}

/// The `@media` prefix strip — upstream `query.replace(/^@media( ?)/m, "")`
/// (`packages/react/src/unstable-use-media-query/index.ts:13`): the first `@media` found at a
/// line start (the `m` flag makes `^` match after every newline as well as at the string start)
/// is removed, plus at most one following literal space; a later occurrence is left alone, and
/// the `@media` match is literal (no word boundary: `@mediax` strips to `x`).
fn strip_media_prefix(query: &str) -> String {
    let bytes = query.as_bytes();
    let mut line_start = 0usize;
    loop {
        if bytes[line_start..].starts_with(b"@media") {
            let after = line_start + b"@media".len();
            let after = if bytes.get(after) == Some(&b' ') {
                after + 1
            } else {
                after
            };
            let mut stripped = String::with_capacity(query.len() - (after - line_start));
            stripped.push_str(&query[..line_start]);
            stripped.push_str(&query[after..]);
            return stripped;
        }
        let Some(offset) = bytes[line_start..].iter().position(|&byte| byte == b'\n') else {
            return query.to_string();
        };
        line_start += offset + 1;
    }
}

/// The ambient half of the effective `matchMedia` resolution — upstream's
/// `supportMatchMedia ? window.matchMedia : null` default expression
/// (`packages/react/src/unstable-use-media-query/index.ts:10-11,17`): `null` without a window
/// (`:10-11` — the source's comment names jsdom; all supported browsers have the function
/// built in) or without `window.matchMedia`.
#[cfg_attr(not(target_arch = "wasm32"), allow(clippy::unnecessary_wraps))]
fn ambient_match_media() -> Option<MatchMediaFn> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // No JS realm on the host target: `typeof window === 'undefined'` (`:10-11`).
        None
    }

    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        // The defensive `typeof window.matchMedia !== 'undefined'` half of the check (`:11`).
        let has_match_media = js_sys::Reflect::get(&window, &JsValue::from_str("matchMedia"))
            .map(|value| !value.is_undefined() && !value.is_null())
            .unwrap_or(false);
        if !has_match_media {
            return None;
        }
        Some(Rc::new(move |query: &str| {
            window
                .match_media(query)
                .expect_throw("Base UI: window.matchMedia threw while resolving a media query")
                .expect_throw("Base UI: window.matchMedia returned no MediaQueryList")
        }))
    }
}

/// The effective `matchMedia` — upstream's option destructuring
/// (`packages/react/src/unstable-use-media-query/index.ts:15-20`): the option overrides the
/// ambient resolution when provided.
fn resolve_match_media(options: &UseMediaQueryOptions) -> Option<MatchMediaFn> {
    match &options.match_media {
        MatchMediaSource::Custom(match_media) => Some(Rc::clone(match_media)),
        MatchMediaSource::Disabled => None,
        MatchMediaSource::Ambient => ambient_match_media(),
    }
}

/// The initial signal value on the no-realm target — upstream's `getServerSnapshot` tier
/// selection (`packages/react/src/unstable-use-media-query/index.ts:24-34`) as it resolves in
/// the SSR analog environment: the `noSsr` live-read branch needs a working `matchMedia`
/// (`:25`), which a realm-less target never has, so the stand-in tier (`:29-32`, captured once
/// per machinery run, with the stripped query) or the `defaultMatches` fallback (`:33`) decides.
#[cfg(not(target_arch = "wasm32"))]
fn initial_value(stripped_query: &str, options: &UseMediaQueryOptions) -> bool {
    if let Some(ssr_match_media) = &options.ssr_match_media {
        return ssr_match_media(stripped_query);
    }
    options.default_matches
}

/// The initial signal value on a realm target — upstream's client snapshot:
/// `useSyncExternalStore` reads `getSnapshot` at render time
/// (`packages/react/src/unstable-use-media-query/index.ts:49`), and with a working
/// `matchMedia` that is the live `mediaQueryList.matches` (`:44`). The port reads it live in
/// the effect's synchronous first run (see the module docs), so the signal seeds with
/// `defaultMatches` — the value that also ends up permanent when the effective `matchMedia` is
/// `null` (`:37-39`).
#[cfg(target_arch = "wasm32")]
fn initial_value(_stripped_query: &str, options: &UseMediaQueryOptions) -> bool {
    options.default_matches
}

/// Reports whether a CSS media query currently matches and tracks the browser's `change`
/// events — upstream `useMediaQuery`
/// (`packages/react/src/unstable-use-media-query/index.ts:5-57`).
///
/// - `query` — the media query string; a leading `@media` (plus one optional space) at a line
///   start is stripped (`:13`). A reactive source: a change re-runs the machinery — fresh
///   `MediaQueryList`, fresh subscription, the previous one dropped — the deps-array contract
///   (`:47`).
/// - `options` — upstream's `Options` object (`:59-83`); fixed for the hook call.
///
/// Returns a signal holding the current match: the live `MediaQueryList.matches` on a realm
/// with a working `matchMedia` (updated on every `change` event), the `ssrMatchMedia` stand-in's
/// captured result on a target with no realm, or `defaultMatches` when no real match
/// information is available (`:37-39`). Must be called inside a reactive owner (a component).
pub fn use_media_query<Q>(query: Q, options: UseMediaQueryOptions) -> Signal<bool>
where
    Q: Get<Value = String> + GetUntracked<Value = String> + 'static,
{
    // The `@media` strip feeds every consumer of the query value (`:13,26,30,41`); the initial
    // tier selection reads it once, untracked — exactly like upstream reading the prop during
    // render (`:5-20`). The machinery's own read inside the effect stays tracked.
    let stripped = strip_media_prefix(&query.get_untracked());
    let initial = initial_value(&stripped, &options);
    let matches = RwSignal::new(initial);

    let match_media = resolve_match_media(&options);
    if let Some(match_media) = match_media {
        use_iso_layout_effect(move || {
            // Tracked read: a query change re-runs the machinery — the deps-array contract
            // (`:47`) — tearing down the previous subscription through the effect cleanup and
            // creating a fresh `MediaQueryList` (`:41`).
            let query = strip_media_prefix(&query.get());
            let media_query_list = match_media(&query);
            // The snapshot re-reads the live `matches` (`:44`); the change notification is
            // what drives it (`:43-46`).
            matches.set(media_query_list.matches());
            let list_for_listener = media_query_list.clone();
            let unsubscribe = add_event_listener(&media_query_list, "change", move |_| {
                matches.set(list_for_listener.matches());
            });
            let unsubscribe = SendWrapper::new(unsubscribe);
            on_cleanup(move || unsubscribe.take().unsubscribe());
        });
    }

    Signal::from(matches)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;
    use reactive_graph::wrappers::read::Signal;

    use super::*;

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Pins the prefix strip (`packages/react/src/unstable-use-media-query/index.ts:13`):
    // `/^@media( ?)/m` — first line-start occurrence only, at most one literal space stripped,
    // no word boundary.
    #[test]
    fn strips_the_at_media_prefix_exactly_like_the_upstream_regex() {
        #[rustfmt::skip]
        let cases = [
            // Already stripped / never had a prefix.
            ("(min-width: 600px)", "(min-width: 600px)"),
            ("", ""),
            // Prefix with and without the single optional space.
            ("@media (min-width: 600px)", "(min-width: 600px)"),
            ("@media(min-width: 600px)", "(min-width: 600px)"),
            // Only ONE space is consumed — `( ?)` is a single optional space.
            ("@media  (min-width: 600px)", " (min-width: 600px)"),
            // The `m` flag: a line start after a newline also matches.
            ("foo\n@media (min-width: 600px)", "foo\n(min-width: 600px)"),
            // First occurrence only — the second is left alone.
            ("@media a\n@media b", "a\n@media b"),
            // Not at a line start: untouched.
            ("foo (min-width: 600px) @media bar", "foo (min-width: 600px) @media bar"),
            // Literal match, no word boundary.
            ("@mediaonly", "only"),
            ("@mediax (min-width: 600px)", "x (min-width: 600px)"),
        ];
        for (query, expected) in cases {
            assert_eq!(
                strip_media_prefix(query),
                expected,
                "stripping {query:?} must match the upstream regex"
            );
        }
    }

    // Pins the no-matchMedia environment (`packages/react/src/unstable-use-media-query/
    // index.ts:6-11,37-39`): the host target has no realm, so the effective `matchMedia` is
    // `null` and the result is pinned to `defaultMatches` — statically, with no subscription.
    #[test]
    fn a_realm_without_a_window_pins_the_result_to_default_matches() {
        let _owner = owner();

        let default_matches = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions::default(),
        );
        assert_eq!(
            default_matches.get_untracked(),
            false,
            "defaultMatches defaults to false"
        );

        let pinned_true = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions {
                default_matches: true,
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(
            pinned_true.get_untracked(),
            true,
            "defaultMatches is used when no real match information is available"
        );
    }

    // Pins the ssrMatchMedia tier (`packages/react/src/unstable-use-media-query/
    // index.ts:29-32`): the stand-in's result is captured once per machinery run, and it is
    // consulted with the stripped query (`:30` reads the already-replaced local).
    #[test]
    fn the_server_snapshot_tier_captures_ssr_match_media_once_with_the_stripped_query() {
        let _owner = owner();
        let calls = Rc::new(Cell::new(0u32));
        let received = Rc::new(std::cell::RefCell::new(String::new()));

        let calls_for_closure = Rc::clone(&calls);
        let received_for_closure = Rc::clone(&received);
        let ssr_match_media: SsrMatchMediaFn = Rc::new(move |query: &str| {
            calls_for_closure.set(calls_for_closure.get() + 1);
            *received_for_closure.borrow_mut() = query.to_string();
            true
        });

        let matches = use_media_query(
            Signal::derive(|| "@media (max-width: 200px)".to_string()),
            UseMediaQueryOptions {
                ssr_match_media: Some(ssr_match_media),
                ..UseMediaQueryOptions::default()
            },
        );

        assert_eq!(matches.get_untracked(), true, "the captured result is used");
        assert_eq!(
            *received.borrow(),
            "(max-width: 200px)",
            "the stand-in receives the stripped query"
        );
        assert_eq!(calls.get(), 1, "the result is captured once, not per read");
    }

    // Pins the tier ORDER (`packages/react/src/unstable-use-media-query/index.ts:24-34`): the
    // noSsr live-read branch requires a working matchMedia (`:25`); on a realm-less target it
    // is skipped because matchMedia is null, so the stand-in (or the default) decides.
    #[test]
    fn no_ssr_cannot_override_a_missing_match_media() {
        let _owner = owner();

        let ssr_match_media: SsrMatchMediaFn = Rc::new(|_query: &str| true);
        let no_ssr_with_stand_in = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions {
                no_ssr: true,
                ssr_match_media: Some(ssr_match_media),
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(
            no_ssr_with_stand_in.get_untracked(),
            true,
            "with no matchMedia, the noSsr branch is skipped and the stand-in tier decides"
        );

        let no_ssr_alone = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions {
                no_ssr: true,
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(
            no_ssr_alone.get_untracked(),
            false,
            "with no matchMedia and no stand-in, the result falls back to defaultMatches"
        );
    }

    // Pins the Disabled source (the explicit-null form JS callers can pass despite the type —
    // the static behavior of `packages/react/src/unstable-use-media-query/index.ts:37-39`).
    #[test]
    fn the_disabled_source_pins_default_matches() {
        let _owner = owner();

        let disabled = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions {
                default_matches: true,
                match_media: MatchMediaSource::Disabled,
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(
            disabled.get_untracked(),
            true,
            "the disabled source resolves to null and pins defaultMatches"
        );
    }

    // Pins per-call-site independence (`specs/library/unstable-use-media-query/behavior.md`,
    // "Edge cases"): multiple hook instances are independent.
    #[test]
    fn two_call_sites_get_independent_signals() {
        let _owner = owner();

        let first = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions {
                default_matches: true,
                ..UseMediaQueryOptions::default()
            },
        );
        let second = use_media_query(
            Signal::derive(|| "(min-width: 600px)".to_string()),
            UseMediaQueryOptions::default(),
        );

        assert_eq!(first.get_untracked(), true);
        assert_eq!(second.get_untracked(), false);
    }

    // The query is read per machinery run; on the host target (no realm) the effect binding is
    // the noop, so the initial read is the only one — the reactive re-run behavior is pinned by
    // the wasm suite.
    #[test]
    fn the_query_value_is_read_at_call_time() {
        let _owner = owner();
        let query = RwSignal::new("@media (min-width: 1px)".to_string());

        let ssr_match_media: SsrMatchMediaFn = Rc::new(|query: &str| query == "(min-width: 1px)");
        let matches = use_media_query(
            query,
            UseMediaQueryOptions {
                ssr_match_media: Some(ssr_match_media),
                ..UseMediaQueryOptions::default()
            },
        );

        assert_eq!(
            matches.get_untracked(),
            true,
            "the initial read strips and forwards the current query value"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;

    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;
    use reactive_graph::wrappers::read::Signal;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{Event, EventInit, HtmlIFrameElement};

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn owner() -> Owner {
        // The effect branch creates a `RenderEffect`, whose re-run loop is spawned through
        // `any_spawner`'s ambient executor — it must be initialized before the first
        // browser-branch call (`leptos_ui_utils::use_iso_layout_effect`'s module docs).
        // Re-initializing returns `Err`, which every test may therefore ignore.
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();
        owner
    }

    /// Creates an iframe of the given CSS width attached to the document body and returns it
    /// with its content window — the exact setup upstream documents the `matchMedia` option
    /// for (`packages/react/src/unstable-use-media-query/index.ts:66-70`).
    fn attached_iframe(width: &str) -> (HtmlIFrameElement, web_sys::Window) {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let iframe = document
            .create_element("iframe")
            .unwrap_throw()
            .dyn_into::<HtmlIFrameElement>()
            .unwrap_throw();
        iframe.style().set_property("width", width).unwrap_throw();
        iframe
            .style()
            .set_property("height", "150px")
            .unwrap_throw();
        iframe.style().set_property("border", "0").unwrap_throw();
        document
            .body()
            .unwrap_throw()
            .append_child(&iframe)
            .unwrap_throw();
        let content_window = iframe
            .content_window()
            .expect_throw("the attached iframe must have a content window");
        (iframe, content_window)
    }

    fn change_event() -> Event {
        Event::new_with_event_init_dict("change", &EventInit::new()).unwrap_throw()
    }

    fn sleep_ms(ms: i32) -> wasm_bindgen_futures::JsFuture {
        let window = web_sys::window().unwrap_throw();
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap_throw();
        });
        wasm_bindgen_futures::JsFuture::from(promise)
    }

    // Pins the client snapshot (`packages/react/src/unstable-use-media-query/index.ts:44,49`):
    // with a working matchMedia the value is the live `mediaQueryList.matches` from setup, not
    // `defaultMatches`.
    #[wasm_bindgen_test]
    async fn reads_the_live_match_from_the_ambient_window() {
        let _owner = owner();

        let matches = use_media_query(
            Signal::derive(|| "(min-width: 0px)".to_string()),
            UseMediaQueryOptions::default(),
        );
        assert_eq!(
            matches.get_untracked(),
            true,
            "the live read at setup wins over the defaultMatches: false fallback"
        );
    }

    // Pins the Disabled source in a real browser
    // (`packages/react/src/unstable-use-media-query/index.ts:37-39`): an explicit null
    // matchMedia pins the static default even though the ambient window has a working one.
    #[wasm_bindgen_test]
    async fn the_disabled_source_pins_default_matches_even_in_a_browser() {
        let _owner = owner();

        let matches = use_media_query(
            Signal::derive(|| "(min-width: 0px)".to_string()),
            UseMediaQueryOptions {
                match_media: MatchMediaSource::Disabled,
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(
            matches.get_untracked(),
            false,
            "the ambient window's working matchMedia must be bypassed"
        );
    }

    // Pins the ssrMatchMedia tier being hydration-only
    // (`packages/react/src/unstable-use-media-query/index.ts:24-34` vs the client snapshot at
    // `:36-47`): a pure client-side render never consults the stand-in.
    #[wasm_bindgen_test]
    async fn ssr_match_media_is_not_consulted_on_a_realm_target() {
        let _owner = owner();
        let calls = Rc::new(Cell::new(0u32));
        let calls_for_closure = Rc::clone(&calls);

        let matches = use_media_query(
            Signal::derive(|| "(min-width: 0px)".to_string()),
            UseMediaQueryOptions {
                ssr_match_media: Some(Rc::new(move |query: &str| {
                    calls_for_closure.set(calls_for_closure.get() + 1);
                    query.is_empty()
                })),
                ..UseMediaQueryOptions::default()
            },
        );

        assert_eq!(
            matches.get_untracked(),
            true,
            "the live client snapshot is used"
        );
        assert_eq!(calls.get(), 0, "the stand-in is never consulted on a realm");
    }

    // Pins the matchMedia option (`packages/react/src/unstable-use-media-query/index.ts:66-70`)
    // end to end with its documented use case — an iframe content window: the custom
    // implementation overrides the ambient window (whose viewport cannot match the query) and
    // receives the stripped query (`:26,41`).
    #[wasm_bindgen_test]
    async fn the_custom_match_media_source_overrides_the_ambient_window() {
        let _owner = owner();
        let (_iframe, content_window) = attached_iframe("100px");

        let calls = Rc::new(Cell::new(0u32));
        let received = Rc::new(std::cell::RefCell::new(String::new()));
        let calls_for_closure = Rc::clone(&calls);
        let received_for_closure = Rc::clone(&received);
        let match_media: MatchMediaFn = Rc::new(move |query: &str| {
            calls_for_closure.set(calls_for_closure.get() + 1);
            *received_for_closure.borrow_mut() = query.to_string();
            content_window
                .match_media(query)
                .unwrap_throw()
                .unwrap_throw()
        });

        let matches = use_media_query(
            Signal::derive(|| "@media (max-width: 200px)".to_string()),
            UseMediaQueryOptions {
                match_media: MatchMediaSource::Custom(match_media),
                ..UseMediaQueryOptions::default()
            },
        );

        assert_eq!(
            matches.get_untracked(),
            true,
            "the iframe's 100px viewport matches what the ambient window cannot"
        );
        assert_eq!(*received.borrow(), "(max-width: 200px)");
        assert_eq!(calls.get(), 1);
    }

    // Pins the re-subscription machinery (`packages/react/src/unstable-use-media-query/
    // index.ts:36-47`): a query change re-runs the machinery against a fresh MediaQueryList,
    // and the previous list's subscription is dropped — a change on it no longer notifies.
    #[wasm_bindgen_test]
    async fn a_query_change_re_subscribes_and_drops_the_previous_list() {
        let owner = owner();
        let window = web_sys::window().unwrap_throw();
        let list_a = window
            .match_media("(min-width: 0px)")
            .unwrap_throw()
            .unwrap_throw();
        let list_b = window
            .match_media("(min-width: 99999px)")
            .unwrap_throw()
            .unwrap_throw();

        let calls = Rc::new(Cell::new(0u32));
        let calls_for_closure = Rc::clone(&calls);
        let list_a_for_dispatch = list_a.clone();
        let match_media: MatchMediaFn = Rc::new(move |query: &str| {
            calls_for_closure.set(calls_for_closure.get() + 1);
            if query == "(a)" {
                list_a.clone()
            } else {
                list_b.clone()
            }
        });

        let query = RwSignal::new("@media (a)".to_string());
        let matches = use_media_query(
            query,
            UseMediaQueryOptions {
                match_media: MatchMediaSource::Custom(match_media),
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(matches.get_untracked(), true, "list_a matches at setup");
        assert_eq!(calls.get(), 1);

        query.set("(b)".to_string());
        any_spawner::Executor::poll_local();
        assert_eq!(
            matches.get_untracked(),
            false,
            "the machinery re-ran against list_b"
        );
        assert_eq!(
            calls.get(),
            2,
            "the custom implementation consulted per run"
        );

        list_a_for_dispatch
            .dispatch_event(&change_event())
            .unwrap_throw();
        any_spawner::Executor::poll_local();
        assert_eq!(
            matches.get_untracked(),
            false,
            "the previous list's subscription was dropped with the machinery"
        );

        owner.cleanup();
    }

    // Pins the change tracking and the subscription teardown
    // (`packages/react/src/unstable-use-media-query/index.ts:43-46`): a real `change` event on
    // the MediaQueryList re-reads the live matches. The disposal half is asserted
    // differentially: the signal is disposed together with its owner (reading it afterwards
    // panics), and a write to a disposed signal aborts the wasm — so if the subscription had
    // survived the disposal, the reverse change would abort the test during the drain loop
    // instead of letting it complete.
    #[wasm_bindgen_test]
    async fn a_real_change_event_updates_the_signal_and_disposal_stops_it() {
        let owner = owner();
        let (iframe, content_window) = attached_iframe("500px");

        let match_media: MatchMediaFn = Rc::new(move |query: &str| {
            content_window
                .match_media(query)
                .unwrap_throw()
                .unwrap_throw()
        });
        let matches = use_media_query(
            Signal::derive(|| "(max-width: 200px)".to_string()),
            UseMediaQueryOptions {
                match_media: MatchMediaSource::Custom(match_media),
                ..UseMediaQueryOptions::default()
            },
        );
        assert_eq!(
            matches.get_untracked(),
            false,
            "the 500px viewport does not match"
        );

        // Shrink the iframe: the browser fires a real `change` on its MediaQueryList, the
        // subscription re-reads the live matches, and the signal flips.
        iframe.style().set_property("width", "100px").unwrap_throw();
        for _ in 0..100 {
            if matches.get_untracked() {
                break;
            }
            let _ = sleep_ms(10).await;
        }
        assert_eq!(
            matches.get_untracked(),
            true,
            "the change event delivered the flipped live matches"
        );

        // After owner disposal the subscription is torn down: the reverse change must find no
        // listener. Completing this drain without an abort is the teardown assertion — see
        // the test comment.
        owner.cleanup();
        iframe.style().set_property("width", "500px").unwrap_throw();
        for _ in 0..20 {
            let _ = sleep_ms(10).await;
        }
    }
}
