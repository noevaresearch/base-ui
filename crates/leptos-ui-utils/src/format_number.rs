//! Port of `packages/utils/src/formatNumber.ts` (Base UI Phase A util).
//!
//! Upstream is a two-export pure formatting module over the platform `Intl.NumberFormat`:
//! `getFormatter(locale, options)` returns the cached formatter for the (locale, options)
//! pair, and `formatNumber(value, locale, options)` returns `''` for a `null` value or the
//! formatter's `format(value)` otherwise (`packages/utils/src/formatNumber.ts:5-27`). The
//! sole source of truth for behavior is `packages/utils/src/formatNumber.test.ts`
//! (46 lines, 5 tests) (`specs/utils/formatNumber.md`).
//!
//! The cache is a module-level `Map` keyed by
//! `JSON.stringify({ locale: stringifyLocale(locale), options })`
//! (`packages/utils/src/formatNumber.ts:3-6`), so identity is keyed by *content*, not object
//! identity: two separately constructed but structurally equal options objects share one
//! entry (`packages/utils/src/formatNumber.test.ts:13-17`), while distinct `Intl.Locale`
//! inputs (`fr-FR` vs `en-US`) with equal options produce distinct formatters
//! (`packages/utils/src/formatNumber.test.ts:19-26`). The key's `locale` half comes from the
//! ported [`stringify_locale`] (upstream imports `./stringifyLocale`,
//! `packages/utils/src/formatNumber.ts:1`); it is reused, not duplicated here.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The optional `locale?: Intl.LocalesArgument` / `options?: Intl.NumberFormatOptions`
//!   parameters become required `&JsValue` parameters; the upstream omitted-argument call
//!   maps to [`JsValue::UNDEFINED`] — the same convention as [`stringify_locale`]
//!   (`packages/utils/src/stringifyLocale.test.ts:6`). `JSON.stringify` omits an
//!   `undefined`-valued property, so omitted and explicit `null` options land on different
//!   cache keys exactly as in JS.
//! - The module-level `Map` becomes a [`thread_local!`] `HashMap`: wasm execution is
//!   single-threaded, so this mirrors upstream's shared module state (unbounded, no
//!   eviction — cache limits are unproven upstream, `specs/utils/formatNumber.md`).
//! - `value: number | null` becomes [`Option<f64>`]: [`None`] is the upstream
//!   `value == null` branch (`packages/utils/src/formatNumber.ts:24-26`, which in JS matches
//!   both `null` and `undefined`) and maps to the empty string; a present value goes
//!   straight to `Intl.NumberFormat.prototype.format`
//!   (`packages/utils/src/formatNumber.ts:27`).
//! - `js-sys` does not bind `Intl.NumberFormat`, so this module declares its own minimal
//!   import of the platform type (`packages/utils/src/formatNumber.ts:13`), like
//!   [`stringify_locale`] does for `Intl.Locale`.

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use wasm_bindgen::UnwrapThrowExt;

use crate::stringify_locale::stringify_locale;

#[wasm_bindgen]
extern "C" {
    /// The platform `Intl.NumberFormat`, the type upstream caches and formats with
    /// (`packages/utils/src/formatNumber.ts:13`).
    #[derive(Clone)]
    #[wasm_bindgen(extends = js_sys::Object, js_namespace = Intl, typescript_type = "Intl.NumberFormat")]
    pub type NumberFormat;

    #[wasm_bindgen(constructor, js_namespace = Intl, js_class = "NumberFormat")]
    fn new(locale: &JsValue, options: &JsValue) -> NumberFormat;

    /// `Intl.NumberFormat.prototype.format`
    /// (`packages/utils/src/formatNumber.ts:27`).
    #[wasm_bindgen(method, js_name = format)]
    pub fn format(this: &NumberFormat, value: f64) -> js_sys::JsString;
}

thread_local! {
    /// The upstream module-level `cache` (`packages/utils/src/formatNumber.ts:3`).
    static CACHE: RefCell<HashMap<String, NumberFormat>> = RefCell::new(HashMap::new());
}

/// The upstream `getFormatter` export
/// (`packages/utils/src/formatNumber.ts:5-17`): returns the cached `Intl.NumberFormat` for
/// the (locale, options) pair, or builds one with `new Intl.NumberFormat(locale, options)`
/// and inserts it on a miss. Pass [`JsValue::UNDEFINED`] for either omitted argument.
pub fn get_formatter(locale: &JsValue, options: &JsValue) -> NumberFormat {
    let key = cache_key(locale, options);

    if let Some(cached) = CACHE.with(|cache| cache.borrow().get(&key).cloned()) {
        return cached;
    }

    let formatter = NumberFormat::new(locale, options);
    CACHE.with(|cache| cache.borrow_mut().insert(key, formatter.clone()));
    formatter
}

/// The upstream `formatNumber` export
/// (`packages/utils/src/formatNumber.ts:19-27`): formats `value` with the cached formatter
/// for `locale`/`options`; [`None`] (upstream `value == null`) returns the empty string.
pub fn format_number(value: Option<f64>, locale: &JsValue, options: &JsValue) -> String {
    let Some(value) = value else {
        return String::new();
    };
    get_formatter(locale, options).format(value).into()
}

/// The upstream cache key
/// (`packages/utils/src/formatNumber.ts:6`): `JSON.stringify({ locale: stringifyLocale(locale),
/// options })`. The object literal serializes own enumerable properties in insertion order
/// (`locale` first, then `options`) and drops an `undefined`-valued property, so the key is
/// exactly upstream's for every input shape.
fn cache_key(locale: &JsValue, options: &JsValue) -> String {
    let key_source = js_sys::Object::new();
    js_sys::Reflect::set(
        &key_source,
        &"locale".into(),
        &stringify_locale(locale).into(),
    )
    .unwrap_throw();
    js_sys::Reflect::set(&key_source, &"options".into(), options).unwrap_throw();
    js_sys::JSON::stringify(&key_source).unwrap_throw().into()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    // The tests exercise the JS `Intl` globals, so they run in a real browser via the wasm32
    // test runner (`.cargo/config.toml` wires it to chromedriver), like the crate's other
    // wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen]
    extern "C" {
        /// Test-local bindings for the platform surface the upstream *test* uses but the
        /// port itself does not: constructing a bare `Intl.NumberFormat` to compute the
        /// expected output (`packages/utils/src/formatNumber.test.ts:31-34`), reading
        /// `resolvedOptions().locale` (`packages/utils/src/formatNumber.test.ts:24-25`),
        /// and constructing `Intl.Locale` instances
        /// (`packages/utils/src/formatNumber.test.ts:20-21`).
        #[wasm_bindgen(extends = js_sys::Object, js_namespace = Intl, typescript_type = "Intl.NumberFormat")]
        type RawNumberFormat;

        #[wasm_bindgen(constructor, js_namespace = Intl, js_class = "NumberFormat")]
        fn new(locale: &JsValue, options: &JsValue) -> RawNumberFormat;

        #[wasm_bindgen(method, js_name = resolvedOptions)]
        fn resolved_options(this: &RawNumberFormat) -> js_sys::Object;

        /// `new Intl.Locale(tag)` — `js-sys` does not bind the `Intl.Locale` type
        /// (`packages/utils/src/formatNumber.test.ts:20-21`).
        #[wasm_bindgen(extends = js_sys::Object, js_namespace = Intl, typescript_type = "Intl.Locale")]
        type IntlLocale;

        #[wasm_bindgen(constructor, js_namespace = Intl, js_class = "Locale")]
        fn new(tag: &str) -> IntlLocale;
    }

    /// Upstream `expect(formatter1).toBe(formatter2)`
    /// (`packages/utils/src/formatNumber.test.ts:16`): the two wrappers must point at the
    /// same JS object (`JsValue` equality is JS `===`, identity for objects).
    fn same_js_object(a: &NumberFormat, b: &NumberFormat) -> bool {
        let a: &JsValue = a.as_ref();
        let b: &JsValue = b.as_ref();
        a == b
    }

    /// Mirrors `packages/utils/src/formatNumber.test.ts:4-9` — a fresh object per call, so
    /// a cache hit is proven to come from option *content*, not object identity.
    fn currency_options() -> js_sys::Object {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"currency".into(), &"USD".into()).unwrap_throw();
        js_sys::Reflect::set(&options, &"style".into(), &"currency".into()).unwrap_throw();
        js_sys::Reflect::set(&options, &"minimumFractionDigits".into(), &2.into()).unwrap_throw();
        js_sys::Reflect::set(&options, &"maximumFractionDigits".into(), &2.into()).unwrap_throw();
        options
    }

    /// Mirrors `packages/utils/src/formatNumber.test.ts:31-34` — the expected formatter's
    /// options in the `formats a number` test.
    fn plain_currency_options() -> js_sys::Object {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"style".into(), &"currency".into()).unwrap_throw();
        js_sys::Reflect::set(&options, &"currency".into(), &"USD".into()).unwrap_throw();
        options
    }

    /// `formatter.resolvedOptions().locale`
    /// (`packages/utils/src/formatNumber.test.ts:24-25`).
    fn resolved_locale(formatter: &NumberFormat) -> String {
        let value: &JsValue = formatter.as_ref();
        let raw: &RawNumberFormat = value.unchecked_ref();
        js_sys::Reflect::get(&raw.resolved_options(), &"locale".into())
            .unwrap_throw()
            .as_string()
            .unwrap_throw()
    }

    // Mirrors `packages/utils/src/formatNumber.test.ts:13-17`.
    #[wasm_bindgen_test]
    fn caches_the_formatter_based_on_options() {
        let formatter1 = get_formatter(&JsValue::UNDEFINED, currency_options().as_ref());
        let formatter2 = get_formatter(&JsValue::UNDEFINED, currency_options().as_ref());
        assert!(same_js_object(&formatter1, &formatter2));
    }

    // Mirrors `packages/utils/src/formatNumber.test.ts:19-26`.
    #[wasm_bindgen_test]
    fn caches_different_intl_locale_objects_separately() {
        let formatter1 = get_formatter(
            IntlLocale::new("fr-FR").as_ref(),
            currency_options().as_ref(),
        );
        let formatter2 = get_formatter(
            IntlLocale::new("en-US").as_ref(),
            currency_options().as_ref(),
        );

        assert!(!same_js_object(&formatter1, &formatter2));
        assert_eq!(resolved_locale(&formatter1), "fr-FR");
        assert_eq!(resolved_locale(&formatter2), "en-US");
    }

    // Mirrors `packages/utils/src/formatNumber.test.ts:30-36`: the output must match the
    // platform's own formatter for equivalent options, computed with the same `Intl` the
    // port delegates to.
    #[wasm_bindgen_test]
    fn formats_a_number() {
        let expected: String =
            NumberFormat::new(&JsValue::UNDEFINED, plain_currency_options().as_ref())
                .format(1234.56)
                .into();
        assert_eq!(
            format_number(
                Some(1234.56),
                &JsValue::UNDEFINED,
                currency_options().as_ref()
            ),
            expected
        );
    }

    // Mirrors `packages/utils/src/formatNumber.test.ts:38-40`.
    #[wasm_bindgen_test]
    fn formats_a_number_with_different_options() {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"style".into(), &"percent".into()).unwrap_throw();
        assert_eq!(
            format_number(Some(0.1234), &"en-US".into(), options.as_ref()),
            "12%"
        );
    }

    // Mirrors `packages/utils/src/formatNumber.test.ts:42-44`.
    #[wasm_bindgen_test]
    fn returns_an_empty_string_for_null() {
        assert_eq!(
            format_number(None, &"en-US".into(), currency_options().as_ref()),
            ""
        );
    }

    // Unproven upstream but direct from the implementation: the cache key stringifies the
    // locale through `stringifyLocale` (`packages/utils/src/formatNumber.ts:6`), so two
    // calls with the same string locale and structurally equal options share one entry —
    // the integration point with the ported `stringify_locale` dependency.
    #[wasm_bindgen_test]
    fn caches_formatters_for_the_same_string_locale() {
        let formatter1 = get_formatter(&"en-US".into(), currency_options().as_ref());
        let formatter2 = get_formatter(&"en-US".into(), currency_options().as_ref());
        assert!(same_js_object(&formatter1, &formatter2));
        assert_eq!(resolved_locale(&formatter1), "en-US");
    }
}
