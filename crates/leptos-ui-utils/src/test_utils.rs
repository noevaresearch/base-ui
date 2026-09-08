//! Port of `packages/utils/src/testUtils.ts` (Base UI Phase A util).
//!
//! Upstream is a three-export test-harness module (`packages/utils/src/testUtils.ts:1-21`)
//! with no runtime-behavior test of its own: the only upstream test files touching it are five
//! type-test `.spec.ts` files whose `expectType` usage is compile-time assertion only
//! (`packages/utils/src/addEventListener.spec.ts:1`, `packages/utils/src/useControlled.spec.ts:2`,
//! `packages/utils/src/store/Store.spec.ts:1`, `packages/utils/src/store/ReactStore.spec.ts:1`,
//! `packages/utils/src/store/createSelector.spec.ts:1`), so every behavioral claim in
//! `specs/utils/testUtils.md` is marked UNVERIFIED there and is pinned by this module's own
//! tests instead (`specs/utils/testUtils.md`, "Source of truth"). The three exports:
//!
//! - `isJSDOM` — `/jsdom/.test(window.navigator.userAgent)` evaluated once at module load
//!   (`packages/utils/src/testUtils.ts:4`); see [`is_jsdom`].
//! - `IfEquals<T, U, Y = unknown, N = never>` — the exact-identity conditional
//!   (`packages/utils/src/testUtils.ts:6-8`); see [`TypeEq`].
//! - `expectType<Expected, Actual>(_actual)` — a compile-time-only type assertion with an
//!   empty body (`packages/utils/src/testUtils.ts:21`); see [`expect_type`].
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream evaluates `isJSDOM` eagerly at module-import time; Rust has no module-load
//!   hook, so the snapshot is taken on first access and frozen for the life of the program
//!   (a [`thread_local!`] `OnceCell<bool>`, the crate's established module-state pattern —
//!   wasm execution is single-threaded). Every reader observes the same frozen boolean and
//!   later changes to the user-agent string are not observed; both properties are pinned by
//!   this module's wasm tests, since a real browser is the only realm where the global read
//!   succeeds.
//! - Upstream assumes `window` exists at import time — importing the module in a bare Node
//!   environment without a DOM global would throw (`specs/utils/testUtils.md`, "DOM structure
//!   & portal behavior"). The port preserves the failure mode: the first [`is_jsdom`] call in
//!   a realm without a global window panics (`UnwrapThrowExt`, the same convention as
//!   [`owner_document`](crate::owner_document) / [`owner_window`](crate::owner_window)).
//! - The `/jsdom/` predicate is factored into a private pure function over the user-agent
//!   string (`str::contains("jsdom")` — the same unanchored, case-sensitive substring test) so
//!   both branches are testable on the host, where no real navigator exists.
//! - `IfEquals` maps to the [`TypeEq`] trait: its blanket impl is the only possible impl
//!   (coherence), so `This` always normalizes to the implementing type itself and an equality
//!   bound `Actual: TypeEq<This = Expected>` holds exactly when the two types are identical.
//!   Upstream's `Y`/`N` branches collapse structurally: the parameter type plays
//!   `Y = Actual` and the unsatisfiable bound plays `N = never` — a mismatched call is a
//!   compile error, which is the entire contract.
//! - `expectType` maps to [`expect_type`]: an empty-bodied function with zero runtime cost,
//!   exactly like upstream. The five upstream `.spec.ts` consumers are compile-time pins with
//!   no runtime assertions; the analogous pins here are this module's doc tests, including a
//!   `compile_fail` example — the Rust mechanism for asserting that a wrongly-typed call does
//!   not compile.

use std::cell::OnceCell;

use wasm_bindgen::UnwrapThrowExt;

thread_local! {
    /// The `isJSDOM` snapshot (`packages/utils/src/testUtils.ts:4`): computed on first
    /// access, then frozen — every later reader observes the same boolean.
    static IS_JSDOM: OnceCell<bool> = const { OnceCell::new() };
}

/// The unanchored, case-sensitive `/jsdom/.test(...)` substring check
/// (`packages/utils/src/testUtils.ts:4`), factored out as a pure function so both branches
/// are host-testable (`specs/utils/testUtils.md`, "Edge cases": any user-agent string
/// containing `jsdom` anywhere classifies the run as jsdom).
fn user_agent_is_jsdom(user_agent: &str) -> bool {
    user_agent.contains("jsdom")
}

/// The upstream `isJSDOM` export (`packages/utils/src/testUtils.ts:4`): whether the test runs
/// in a JSDOM environment, decided by testing the user-agent string for the substring
/// `jsdom` (unanchored, case-sensitive).
///
/// Upstream computes the value once at module load; the port snapshots on first access and
/// freezes it for the program's lifetime, so later changes to the user-agent string are not
/// observed (see the module docs). Panics in a realm without a global `window`, preserving
/// upstream's import-time `ReferenceError` in a bare Node environment (module docs, "Rust
/// adaptations").
pub fn is_jsdom() -> bool {
    IS_JSDOM.with(|snapshot| {
        *snapshot.get_or_init(|| {
            let window = web_sys::window().unwrap_throw();
            let user_agent = window.navigator().user_agent().unwrap_throw();
            user_agent_is_jsdom(&user_agent)
        })
    })
}

/// The port of upstream's `IfEquals<T, U, Y = unknown, N = never>` exact-identity conditional
/// (`packages/utils/src/testUtils.ts:6-8`, attributed upstream to the linked Stack Overflow
/// technique for testing exact type identity).
///
/// The blanket impl is the only possible impl (coherence), so `This` always normalizes to the
/// implementing type itself; an equality bound `T: TypeEq<This = U>` is therefore satisfiable
/// exactly when `T` and `U` are the *same* type — the strict-identity semantics the upstream
/// conditional's `Y`/`N` selection provides (`specs/utils/testUtils.md`, "Edge cases": mutual
/// assignability in one direction is not enough). [`expect_type`] is defined in terms of this
/// bound.
pub trait TypeEq {
    /// The type itself — the `Y`-branch result of upstream's conditional, normalized back to
    /// the implementing type by the blanket impl.
    type This: ?Sized;
}

impl<T: ?Sized> TypeEq for T {
    type This = T;
}

/// The upstream `expectType` export (`packages/utils/src/testUtils.ts:21`): issues a type
/// error if `Expected` is not identical to `Actual`, with an empty body — a pure compile-time
/// assertion with zero runtime cost or checking (`specs/utils/testUtils.md`, "Public API
/// surface").
///
/// `Expected` should be declared when invoking `expect_type`; `Actual` is almost always
/// inferred from the value argument — the shape every real upstream call site uses, e.g.
/// `expectType<PointerEvent, typeof event>(event)`
/// (`packages/utils/src/addEventListener.spec.ts:5`). A call whose argument type is not
/// exactly `Expected` fails the [`TypeEq`] equality bound and does not compile; the empty
/// body means a mismatch can never "pass at runtime" (see the `compile_fail` example below —
/// the Rust analog of upstream's compile-time-only `.spec.ts` consumers).
///
/// # Examples
///
/// Pinning an exact type compiles and does nothing at runtime:
///
/// ```
/// use leptos_ui_utils::expect_type;
///
/// let value: Vec<u8> = vec![1, 2, 3];
/// expect_type::<Vec<u8>, _>(value);
/// ```
///
/// A value whose type is not *exactly* the expected type fails to compile — the port of
/// upstream's worked JSDoc example (`packages/utils/src/testUtils.ts:16`), where
/// `expectType<number | string, typeof value>(value)` errors because `typeof value` is not
/// identical to the union even though it is assignable to it:
///
/// ```compile_fail
/// use leptos_ui_utils::expect_type;
///
/// let value: u32 = 5;
/// expect_type::<i64, _>(value);
/// ```
pub fn expect_type<Expected, Actual>(_actual: Actual)
where
    Actual: TypeEq<This = Expected>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    // The predicate is the `/jsdom/.test(...)` regex test
    // (`packages/utils/src/testUtils.ts:4`): unanchored, so any position in the string
    // matches (`specs/utils/testUtils.md`, "Edge cases"). Upstream has no runtime test file,
    // so these host tests are the port's own pinning of the inferred contract.
    #[test]
    fn detects_jsdom_anywhere_in_the_user_agent_string() {
        assert!(user_agent_is_jsdom("jsdom/24.0.0"));
        assert!(user_agent_is_jsdom("Mozilla/5.0 (jsdom; Linux) AppleWebKit/537.36"));
        assert!(user_agent_is_jsdom("Mozilla/5.0 (X11; Linux x86_64) jsdom"));
    }

    // A real-browser user agent (and an empty string) does not contain the substring, so the
    // answer is false outside jsdom.
    #[test]
    fn does_not_detect_jsdom_in_real_browser_user_agents() {
        assert!(!user_agent_is_jsdom(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"
        ));
        assert!(!user_agent_is_jsdom(""));
    }

    // The upstream regex `/jsdom/` is case-sensitive (`packages/utils/src/testUtils.ts:4`),
    // so an upper-case variant must not classify the run as jsdom.
    #[test]
    fn detection_is_case_sensitive_like_the_upstream_regex() {
        assert!(!user_agent_is_jsdom("JSDOM"));
        assert!(!user_agent_is_jsdom("JsDom"));
    }

    // `expectType` is a pure compile-time assertion with an empty body
    // (`packages/utils/src/testUtils.ts:21`): calling it with identically-typed values is a
    // no-op that returns `()`. The positive half of the contract is pinned by these calls
    // compiling and running; the negative half is pinned by the `compile_fail` doc test.
    #[test]
    fn expect_type_is_a_runtime_noop_for_identical_types() {
        expect_type::<u32, _>(7u32);
        expect_type::<String, _>(String::from("value"));
        expect_type::<Vec<u8>, _>(vec![1, 2, 3]);
        expect_type::<Option<&str>, _>(Some("x"));
    }

    // The mechanism `expect_type` relies on: the blanket impl normalizes `This` to the
    // implementing type itself, so a `TypeEq<This = Expected>` bound is satisfiable exactly
    // when the types are identical — the strict identity `IfEquals` provides
    // (`specs/utils/testUtils.md`, "Edge cases"). Pinned via `TypeId` on the normalized
    // associated type for types of different shapes and sizes.
    #[test]
    fn type_eq_normalizes_this_to_the_implementing_type() {
        fn this_of<T: ?Sized + TypeEq + 'static>() -> std::any::TypeId {
            std::any::TypeId::of::<<T as TypeEq>::This>()
        }

        assert_eq!(this_of::<String>(), std::any::TypeId::of::<String>());
        assert_eq!(this_of::<Vec<u8>>(), std::any::TypeId::of::<Vec<u8>>());
        assert_eq!(this_of::<u32>(), std::any::TypeId::of::<u32>());
        assert_eq!(this_of::<str>(), std::any::TypeId::of::<str>());
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use js_sys::Object;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    // The behavior under test is a real read of `window.navigator.userAgent`
    // (`packages/utils/src/testUtils.ts:4`), so the tests run in a real browser via the
    // wasm32 test runner (`.cargo/config.toml` wires it to chromedriver), like the crate's
    // other wasm test modules.
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn navigator_object() -> Object {
        web_sys::window()
            .unwrap_throw()
            .navigator()
            .unchecked_into::<Object>()
    }

    fn user_agent() -> String {
        web_sys::window()
            .unwrap_throw()
            .navigator()
            .user_agent()
            .unwrap_throw()
    }

    // Implementation-derived (`packages/utils/src/testUtils.ts:4`): a real browser's
    // user-agent string does not contain the `jsdom` substring, so the answer is false — the
    // only value observable in a browser realm. (The true branch is unobservable here by
    // definition; its substring logic is pinned by the host tests.)
    #[wasm_bindgen_test]
    fn is_jsdom_is_false_in_a_real_browser() {
        assert!(!is_jsdom());
    }

    // Snapshot semantics (`specs/utils/testUtils.md`, "State model" — UNVERIFIED upstream,
    // implementation-derived): `isJSDOM` is computed once at module load, so every reader
    // observes the same frozen boolean and later changes to the user-agent string are not
    // observed. Mutating `navigator.userAgent` (an own value property shadowing the
    // prototype's getter) after the snapshot exists must not change the answer; deleting the
    // own property restores the real user agent.
    #[wasm_bindgen_test]
    fn later_user_agent_changes_are_not_observed_after_the_snapshot() {
        let original = user_agent();

        // Force the snapshot to exist before mutating the environment.
        let before = is_jsdom();

        let navigator = navigator_object();
        let descriptor = Object::new();
        js_sys::Reflect::set(
            &descriptor,
            &JsValue::from_str("value"),
            &JsValue::from_str("jsdom mutated agent"),
        )
        .unwrap_throw();
        js_sys::Reflect::set(
            &descriptor,
            &JsValue::from_str("configurable"),
            &JsValue::from(true),
        )
        .unwrap_throw();
        js_sys::Reflect::define_property(&navigator, &JsValue::from_str("userAgent"), &descriptor)
            .unwrap_throw();

        assert_eq!(is_jsdom(), before);

        js_sys::Reflect::delete_property(&navigator, &JsValue::from_str("userAgent")).unwrap_throw();
        assert_eq!(user_agent(), original);
    }

    // Repeated reads observe the same frozen value (`specs/utils/testUtils.md`, "State
    // model"): repeated or interleaved calls cannot interfere with each other.
    #[wasm_bindgen_test]
    fn repeated_calls_read_the_same_frozen_value() {
        let first = is_jsdom();
        for _ in 0..3 {
            assert_eq!(is_jsdom(), first);
        }
    }
}
