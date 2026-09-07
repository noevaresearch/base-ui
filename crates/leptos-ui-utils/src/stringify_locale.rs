//! Port of `packages/utils/src/stringifyLocale.ts` (Base UI Phase A util).
//!
//! Upstream is a single-export pure function that stringifies an `Intl.LocalesArgument`
//! (`string | Intl.Locale | Array<string | Intl.Locale>`) into the string form used for
//! cache keys (`packages/utils/src/stringifyLocale.test.ts:5`): arrays are stringified by
//! mapping the same function over each element and joining with `,`
//! (`packages/utils/src/stringifyLocale.ts:2-3`), a `null`/`undefined` argument becomes `''`
//! (`packages/utils/src/stringifyLocale.ts:6-8`), and everything else goes through the JS
//! `String()` coercion — a string passes through unchanged and
//! `new Intl.Locale('fr-FR')` stringifies to `'fr-FR'`
//! (`packages/utils/src/stringifyLocale.ts:10`, `packages/utils/src/stringifyLocale.test.ts:7-9`).
//! The sole source of truth for behavior is `packages/utils/src/stringifyLocale.test.ts`
//! (11 lines, 1 test / 4 assertions), which pins all four output shapes this port is tested
//! against (`specs/utils/stringifyLocale.md`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - `Intl.LocalesArgument` is represented as an arbitrary [`JsValue`], the same way the
//!   `js_sys` bindings represent JS union types: a primitive string, an `Intl.Locale`
//!   instance, or a JS `Array` of either. Array detection mirrors upstream's
//!   `Array.isArray` ([`js_sys::Array::is_array`]), so a plain string never reaches
//!   [`js_sys::Array::from`] (which would split it into single characters).
//! - The optional `locale?: Intl.LocalesArgument` parameter becomes a required `&JsValue`;
//!   the upstream no-argument call maps to [`JsValue::UNDEFINED`]
//!   (`packages/utils/src/stringifyLocale.test.ts:6`). Upstream's `locale == null` check is
//!   carried over as [`JsValue::is_null_or_undefined`], which covers `null` and `undefined`
//!   exactly like JS `==`.
//! - Upstream's per-element recursion `locale.map((value) => stringifyLocale(value))`
//!   (`packages/utils/src/stringifyLocale.ts:3`) is kept as a real recursion into
//!   [`stringify_locale`], so nested arrays flatten the same way (untested upstream —
//!   `specs/utils/stringifyLocale.md`, "Not covered by any test").
//! - The JS `String(value)` global coercion (`packages/utils/src/stringifyLocale.ts:10`)
//!   has no `js-sys` binding, so this module declares its own minimal import of the global
//!   `String` function.

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
    /// The JS `String(value)` global coercion
    /// (`packages/utils/src/stringifyLocale.ts:10`).
    #[wasm_bindgen(js_name = "String")]
    fn js_string_coercion(value: &JsValue) -> String;
}

/// The upstream `stringifyLocale` export
/// (`packages/utils/src/stringifyLocale.ts:1-11`): stringifies an `Intl.LocalesArgument`
/// into the cache-key string.
///
/// Pass [`JsValue::UNDEFINED`] for the upstream no-argument call
/// (`packages/utils/src/stringifyLocale.test.ts:6`). See the module docs for the accepted
/// input shapes and their outputs.
pub fn stringify_locale(locale: &JsValue) -> String {
    if js_sys::Array::is_array(locale) {
        let elements = js_sys::Array::from(locale);
        let mut parts: Vec<String> = Vec::with_capacity(elements.length() as usize);
        elements.for_each(&mut |element: JsValue, _index: u32, _array: js_sys::Array| {
            parts.push(stringify_locale(&element));
        });
        return parts.join(",");
    }

    if locale.is_null_or_undefined() {
        return String::new();
    }

    js_string_coercion(locale)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    // The tests only exercise JS language globals (`Intl`, `Array`, `String`), but the
    // crate's other wasm test modules run in the browser and `.cargo/config.toml` wires the
    // wasm32 test runner to chromedriver.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen]
    extern "C" {
        /// `new Intl.Locale(tag)` — `js-sys` does not bind the `Intl.Locale` type
        /// (`packages/utils/src/stringifyLocale.test.ts:8`).
        #[wasm_bindgen(extends = js_sys::Object, js_namespace = Intl, typescript_type = "Intl.Locale")]
        type IntlLocale;

        #[wasm_bindgen(constructor, js_namespace = Intl, js_class = "Locale")]
        fn new(tag: &str) -> IntlLocale;
    }

    // Mirrors `packages/utils/src/stringifyLocale.test.ts:5-10` — the unit's entire test
    // suite: no argument, a plain string, an `Intl.Locale` instance, and a mixed array of
    // both element types.
    #[wasm_bindgen_test]
    fn stringifies_intl_locale_arguments_for_cache_keys() {
        assert_eq!(stringify_locale(&JsValue::UNDEFINED), "");
        assert_eq!(stringify_locale(&JsValue::from_str("en-US")), "en-US");
        assert_eq!(
            stringify_locale(IntlLocale::new("fr-FR").as_ref()),
            "fr-FR"
        );

        let mixed = js_sys::Array::of2(
            &JsValue::from_str("fr-FR"),
            IntlLocale::new("en-US").as_ref(),
        );
        assert_eq!(stringify_locale(mixed.as_ref()), "fr-FR,en-US");
    }

    // Unproven upstream but direct from the implementation: `locale == null` covers an
    // explicit `null` argument, not just the omitted one
    // (`packages/utils/src/stringifyLocale.ts:6-8`).
    #[wasm_bindgen_test]
    fn stringifies_an_explicit_null_to_the_empty_string() {
        assert_eq!(stringify_locale(&JsValue::NULL), "");
    }

    // Unproven upstream but direct from the implementation: `map` over zero elements
    // followed by `join(',')` yields the empty string for `[]`
    // (`packages/utils/src/stringifyLocale.ts:3`).
    #[wasm_bindgen_test]
    fn stringifies_an_empty_array_to_the_empty_string() {
        assert_eq!(stringify_locale(js_sys::Array::new().as_ref()), "");
    }

    // Unproven upstream but direct from the implementation: elements go through the full
    // `stringifyLocale` recursion (`packages/utils/src/stringifyLocale.ts:3`), so a nested
    // array flattens completely rather than being coerced with `String()`.
    #[wasm_bindgen_test]
    fn flattens_nested_arrays() {
        let nested = js_sys::Array::of2(
            js_sys::Array::of2(&JsValue::from_str("fr-FR"), &JsValue::from_str("en-US")).as_ref(),
            &JsValue::from_str("ja-JP"),
        );
        assert_eq!(stringify_locale(nested.as_ref()), "fr-FR,en-US,ja-JP");
    }
}
