//! Port of `packages/utils/src/warn.ts` (Base UI Phase A util).
//!
//! Upstream is a two-export module with no unit-local algorithm: it creates one logger at module
//! scope — `warn`, the fixed binding `createLogOnce('warn', 'Base UI')`
//! (`packages/utils/src/warn.ts:3`) — and re-exports `reset` from `./createLogOnce`
//! (`packages/utils/src/warn.ts:5`). The unit has no upstream test file, so the `'Base UI'`
//! prefix output is UNVERIFIED in `specs/utils/warn.md`, and the port is deliberately a thin
//! binding over the ported [`create_log_once`](crate::create_log_once) machinery: the
//! once-logging algorithm is specified and tested there (`specs/utils/createLogOnce.md`) and no
//! test in this unit constrains it.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream's module-scope singleton (`packages/utils/src/warn.ts:3`) becomes the [`warn`]
//!   function, returning a fresh [`LogOnce`] handle per call. The two are observationally
//!   equivalent: the once-per-output dedup state lives in the module-level registry, not in the
//!   handle (`packages/utils/src/createLogOnce.ts:1-4`,
//!   `packages/utils/src/createLogOnce.ts:12-14`), so every handle with the same severity and
//!   prefix deduplicates against the same shared state — the only state-bearing aspect the spec
//!   identifies for this unit.
//! - The `reset` re-export is preserved as [`reset`], re-exported from
//!   [`crate::create_log_once`] exactly as upstream re-exports it from `./createLogOnce`
//!   (`packages/utils/src/warn.ts:5`).

use crate::create_log_once::{LogOnce, Severity, create_log_once_with_prefix};

/// The upstream `warn` export (`packages/utils/src/warn.ts:3`): a logger fixed to the `'warn'`
/// severity with the `'Base UI'` prefix — the module-scope binding of
/// [`create_log_once_with_prefix`](crate::create_log_once_with_prefix).
///
/// Upstream consumers live in `packages/react/src` (e.g.
/// `packages/react/src/internals/useRenderElement.tsx:5`).
pub fn warn() -> LogOnce {
    create_log_once_with_prefix(Severity::Warn, "Base UI")
}

pub use crate::create_log_once::reset;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::capture_log;

    // The fixed binding upstream performs at module scope (`packages/utils/src/warn.ts:3`): the
    // emitted output carries the `Base UI` prefix, and the severity is `warn` — proven
    // end-to-end against `console.warn` by this module's wasm tests, since `Severity::Warn`
    // selects that channel.
    #[test]
    fn binds_the_warn_severity_with_the_base_ui_prefix() {
        reset();

        assert_eq!(capture_log(&warn(), &["boom"]), ["Base UI: boom"]);
    }

    // The once-per-output state lives in the shared registry rather than in the handle, so two
    // handles from separate `warn()` calls dedup against the same state — the observable
    // content of upstream's module-scope singleton (`packages/utils/src/warn.ts:3`).
    #[test]
    fn dedups_across_separately_constructed_handles() {
        reset();
        let first = warn();

        assert_eq!(capture_log(&first, &["boom"]), ["Base UI: boom"]);

        assert_eq!(capture_log(&warn(), &["boom"]), Vec::<String>::new());
    }

    // The re-exported `reset` (`packages/utils/src/warn.ts:5`) is the shared registry's reset,
    // so it clears the dedup state the `warn` binding writes into and the same output is logged
    // again — the singleton/reset interaction the spec marks UNVERIFIED upstream
    // (specs/utils/warn.md), which the ported mechanics make direct.
    #[test]
    fn reset_clears_the_shared_registry() {
        reset();

        assert_eq!(capture_log(&warn(), &["boom"]), ["Base UI: boom"]);

        reset();

        assert_eq!(capture_log(&warn(), &["boom"]), ["Base UI: boom"]);
    }

    // The spec marks UNVERIFIED upstream the combination of the fixed prefix with multiple
    // message arguments (specs/utils/warn.md, Events) — the multi-message upstream test creates
    // its logger without a prefix (packages/utils/src/createLogOnce.test.ts:31). The ported
    // mechanics make it direct: the prefix wraps the joined message.
    #[test]
    fn joins_multiple_messages_under_the_base_ui_prefix() {
        reset();

        assert_eq!(
            capture_log(&warn(), &["first", "second"]),
            ["Base UI: first second"]
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Replaces `console.warn` with a recording spy for the spy's lifetime, restoring the
    /// original on drop — the wasm equivalent of upstream's `vi.spyOn(console, 'warn')`
    /// (`packages/utils/src/createLogOnce.test.ts:14`).
    struct WarnSpy {
        console: js_sys::Object,
        original: JsValue,
        calls: Rc<RefCell<Vec<String>>>,
        _closure: Closure<dyn FnMut(JsValue)>,
    }

    impl WarnSpy {
        fn install() -> WarnSpy {
            let global: js_sys::Object = js_sys::global();
            let console: js_sys::Object = js_sys::Reflect::get(&global, &"console".into())
                .unwrap_throw()
                .unchecked_into();
            let original = js_sys::Reflect::get(&console, &"warn".into()).unwrap_throw();
            let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
            let calls_in_spy = calls.clone();
            let closure = Closure::new(move |message: JsValue| {
                calls_in_spy
                    .borrow_mut()
                    .push(message.as_string().unwrap_or_default());
            });
            js_sys::Reflect::set(&console, &"warn".into(), closure.as_ref().unchecked_ref())
                .unwrap_throw();
            WarnSpy {
                console,
                original,
                calls,
                _closure: closure,
            }
        }
    }

    impl Drop for WarnSpy {
        fn drop(&mut self) {
            js_sys::Reflect::set(&self.console, &"warn".into(), &self.original).unwrap_throw();
        }
    }

    // The spec's UNVERIFIED claim, proven end-to-end in a browser (specs/utils/warn.md): the
    // fixed `'Base UI'` prefix is part of the emitted output, the joined output reaches
    // `console.warn` as a single string argument, and a repeated call through the shared
    // registry is suppressed (`packages/utils/src/createLogOnce.test.ts:16-18`).
    #[wasm_bindgen_test]
    fn dispatches_the_base_ui_prefixed_output_to_console_warn_once() {
        reset();
        let spy = WarnSpy::install();

        warn().log(&["boom"]);
        warn().log(&["boom"]);

        assert_eq!(
            spy.calls.borrow().as_slice(),
            ["Base UI: boom".to_string()]
        );
    }
}
