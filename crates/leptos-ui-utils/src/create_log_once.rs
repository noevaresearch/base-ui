//! Port of `packages/utils/src/createLogOnce.ts` (Base UI Phase A util).
//!
//! Upstream is a dev-facing logging factory: `createLogOnce(severity, prefix?)` returns a logger
//! that joins its message arguments with a single space
//! (`packages/utils/src/createLogOnce.ts:10`), prepends the prefix as `` `${prefix}: ${message}` ``
//! when one was given (`packages/utils/src/createLogOnce.ts:11`), and writes the resulting output
//! to `console[severity]` at most once per distinct output, deduplicated in a module-level set
//! keyed by `` `${severity}:${output}` `` (`packages/utils/src/createLogOnce.ts:12-20`). Two
//! different severities never suppress each other
//! (`packages/utils/src/createLogOnce.test.ts:36-43`), and `reset()` clears the registry so an
//! already-emitted output is logged again (`packages/utils/src/createLogOnce.test.ts:45-52`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The `'warn' | 'error'` severity union becomes the [`Severity`] enum; its string form is used
//!   verbatim in the dedup key, as upstream does (`packages/utils/src/createLogOnce.ts:12`).
//! - The optional `prefix?` parameter becomes the separate [`create_log_once_with_prefix`]
//!   factory, matching how upstream's other optional parameters are ported in this crate.
//! - The returned logger function becomes the owned [`LogOnce`] handle with a [`LogOnce::log`]
//!   method taking the message arguments as a slice. The handle owns its prefix, so a logger
//!   outlives any borrowed prefix string.
//! - The module-level dedup set becomes a `thread_local` registry: wasm is single-threaded, and
//!   per-thread isolation keeps native tests independent of each other.
//! - Upstream gates the whole body on `process.env.NODE_ENV !== 'production'`
//!   (`packages/utils/src/createLogOnce.ts:2-9`), making the logger a no-op in production builds.
//!   That gating has no test-observable contract (the upstream test file runs ungated), and the
//!   release-build decision is explicitly deferred (`specs/architecture.md`, "Log-once dedup" /
//!   "Not yet covered"), so the port implements the ungated behavior unconditionally.

use std::cell::RefCell;
use std::collections::HashSet;

use wasm_bindgen::JsValue;

/// The upstream `severity` argument (`packages/utils/src/createLogOnce.ts:7`), restricted to the
/// two values the upstream test file exercises: `'warn'`
/// (`packages/utils/src/createLogOnce.test.ts:15`) and `'error'`
/// (`packages/utils/src/createLogOnce.test.ts:24`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Dispatches the output to `console.warn`
    /// (`packages/utils/src/createLogOnce.ts:15-16`).
    Warn,
    /// Dispatches the output to `console.error`
    /// (`packages/utils/src/createLogOnce.ts:17-18`).
    Error,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

thread_local! {
    /// The upstream module-level `loggedMessages` set
    /// (`packages/utils/src/createLogOnce.ts:1-4`), holding one `` `${severity}:${output}` ``
    /// key per already-emitted output (`packages/utils/src/createLogOnce.ts:12-14`).
    static LOGGED_MESSAGES: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// A logger created by [`create_log_once`] or [`create_log_once_with_prefix`]: the Rust form of
/// upstream's returned `logOnce` function (`packages/utils/src/createLogOnce.ts:8`).
#[derive(Clone, Debug)]
pub struct LogOnce {
    severity: Severity,
    prefix: Option<String>,
}

impl LogOnce {
    /// Logs the messages, joined with a single space
    /// (`packages/utils/src/createLogOnce.test.ts:29-34`) and prefixed as
    /// `` `${prefix}: ${message}` `` when the logger was created with a prefix
    /// (`packages/utils/src/createLogOnce.test.ts:13-20`), at most once per distinct output
    /// until [`reset`] is called (`packages/utils/src/createLogOnce.test.ts:16-18`).
    pub fn log(&self, messages: &[&str]) {
        self.log_to(messages, &mut |output| dispatch_console(self.severity, output));
    }

    /// The dedup and formatting logic shared by every test target; `emit` receives the final
    /// output string exactly when upstream would call the console method
    /// (`packages/utils/src/createLogOnce.ts:13-20`).
    fn log_to(&self, messages: &[&str], emit: &mut dyn FnMut(&str)) {
        let message = messages.join(" ");
        let output = match self.prefix.as_deref() {
            Some(prefix) => format!("{prefix}: {message}"),
            None => message,
        };
        let key = format!("{}:{output}", self.severity.as_str());
        let unseen = LOGGED_MESSAGES.with_borrow_mut(|seen| seen.insert(key));
        if unseen {
            emit(&output);
        }
    }
}

/// Creates a logger writing each unique message to the console channel selected by `severity`,
/// with no prefix (`packages/utils/src/createLogOnce.ts:7-23`).
pub fn create_log_once(severity: Severity) -> LogOnce {
    LogOnce {
        severity,
        prefix: None,
    }
}

/// Same as [`create_log_once`], with the upstream optional second parameter filled in: the
/// prefix is prepended to each emitted output as `` `${prefix}: ${message}` ``
/// (`packages/utils/src/createLogOnce.ts:11`, `packages/utils/src/createLogOnce.test.ts:19`).
pub fn create_log_once_with_prefix(severity: Severity, prefix: &str) -> LogOnce {
    LogOnce {
        severity,
        prefix: Some(prefix.to_string()),
    }
}

/// Clears the dedup registry, so every previously emitted output is logged again on the next
/// call (`packages/utils/src/createLogOnce.test.ts:45-52`).
pub fn reset() {
    LOGGED_MESSAGES.with_borrow_mut(|seen| seen.clear());
}

fn dispatch_console(severity: Severity, output: &str) {
    let value = JsValue::from_str(output);
    match severity {
        Severity::Warn => web_sys::console::warn_1(&value),
        Severity::Error => web_sys::console::error_1(&value),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    /// Records the outputs the logger emits, stubbing the console dispatch the way upstream's
    /// `vi.spyOn(console, ...)` mocks do; the real dispatch is asserted by the wasm tests below.
    fn log_and_capture(log_once: &LogOnce, messages: &[&str]) -> Vec<String> {
        let emitted = RefCell::new(Vec::new());
        log_once.log_to(messages, &mut |output| {
            emitted.borrow_mut().push(output.to_string());
        });
        emitted.into_inner()
    }

    // Mirrors `packages/utils/src/createLogOnce.test.ts:13-20` (the `beforeEach` reset included).
    #[test]
    fn creates_a_logger_with_a_custom_prefix() {
        reset();
        let log_once = create_log_once_with_prefix(Severity::Warn, "My Library");

        assert_eq!(log_and_capture(&log_once, &["message"]), ["My Library: message"]);

        // The identical repeat call is deduplicated to a single console emission
        // (`packages/utils/src/createLogOnce.test.ts:16-18`).
        assert_eq!(log_and_capture(&log_once, &["message"]), Vec::<String>::new());
    }

    // Mirrors `packages/utils/src/createLogOnce.test.ts:22-27`: without a prefix the message is
    // passed through unchanged, with no leading/trailing separator
    // (`packages/utils/src/createLogOnce.test.ts:26`).
    #[test]
    fn creates_a_logger_without_a_prefix() {
        reset();
        let log_once = create_log_once(Severity::Error);

        assert_eq!(log_and_capture(&log_once, &["message"]), ["message"]);
    }

    // Mirrors `packages/utils/src/createLogOnce.test.ts:29-34`.
    #[test]
    fn joins_multiple_messages_with_a_space() {
        reset();
        let log_once = create_log_once(Severity::Warn);

        assert_eq!(
            log_and_capture(&log_once, &["first", "second"]),
            ["first second"]
        );
    }

    // Mirrors `packages/utils/src/createLogOnce.test.ts:36-43`: loggers of different severities
    // keep independent dedup state, so neither suppresses the other.
    #[test]
    fn deduplicates_each_severity_independently() {
        reset();
        let warn_once = create_log_once(Severity::Warn);
        let error_once = create_log_once(Severity::Error);

        let warn_emitted = log_and_capture(&warn_once, &["message"]);
        let error_emitted = log_and_capture(&error_once, &["message"]);

        assert_eq!(warn_emitted, ["message"]);
        assert_eq!(error_emitted, ["message"]);
    }

    // Mirrors `packages/utils/src/createLogOnce.test.ts:45-52`.
    #[test]
    fn logs_again_after_reset() {
        reset();
        let log_once = create_log_once(Severity::Warn);

        assert_eq!(log_and_capture(&log_once, &["message"]), ["message"]);

        reset();

        let emitted_again = log_and_capture(&log_once, &["message"]);
        assert_eq!(emitted_again.len(), 1, "emitted again after reset");
    }

    // Unproven upstream but direct from the implementation: the dedup key is the formatted
    // output (`packages/utils/src/createLogOnce.ts:12`), so the prefix participates in it —
    // the same message through loggers with different prefixes is emitted from each.
    #[test]
    fn includes_the_prefix_in_the_dedup_key() {
        reset();
        let first = create_log_once_with_prefix(Severity::Warn, "First");
        let second = create_log_once_with_prefix(Severity::Warn, "Second");

        assert_eq!(log_and_capture(&first, &["message"]), ["First: message"]);
        assert_eq!(log_and_capture(&second, &["message"]), ["Second: message"]);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Replaces `console[method]` with a recording spy for the spy's lifetime, restoring the
    /// original on drop — the wasm equivalent of upstream's `vi.spyOn(console, ...).mockImplementation`
    /// (`packages/utils/src/createLogOnce.test.ts:14`, `packages/utils/src/createLogOnce.test.ts:23`).
    struct ConsoleSpy {
        console: js_sys::Object,
        method: &'static str,
        original: JsValue,
        calls: Rc<RefCell<Vec<String>>>,
        _closure: Closure<dyn FnMut(JsValue)>,
    }

    impl ConsoleSpy {
        fn install(method: &'static str) -> ConsoleSpy {
            let global: js_sys::Object = js_sys::global();
            let console: js_sys::Object = js_sys::Reflect::get(&global, &"console".into())
                .unwrap_throw()
                .unchecked_into();
            let original = js_sys::Reflect::get(&console, &method.into()).unwrap_throw();
            let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
            let calls_in_spy = calls.clone();
            let closure = Closure::new(move |message: JsValue| {
                calls_in_spy
                    .borrow_mut()
                    .push(message.as_string().unwrap_or_default());
            });
            js_sys::Reflect::set(
                &console,
                &method.into(),
                closure.as_ref().unchecked_ref(),
            )
            .unwrap_throw();
            ConsoleSpy {
                console,
                method,
                original,
                calls,
                _closure: closure,
            }
        }
    }

    impl Drop for ConsoleSpy {
        fn drop(&mut self) {
            js_sys::Reflect::set(&self.console, &self.method.into(), &self.original)
                .unwrap_throw();
        }
    }

    // Asserts the dispatch selection the upstream tests observe through their console spies:
    // `'warn'` reaches `console.warn` (`packages/utils/src/createLogOnce.test.ts:14-19`) and
    // `'error'` reaches `console.error` (`packages/utils/src/createLogOnce.test.ts:23-26`), with
    // the formatted output as the sole argument (`packages/utils/src/createLogOnce.ts:16-18`).
    #[wasm_bindgen_test]
    fn dispatches_to_the_console_method_matching_the_severity() {
        reset();
        let warn_spy = ConsoleSpy::install("warn");
        let error_spy = ConsoleSpy::install("error");

        create_log_once_with_prefix(Severity::Warn, "My Library").log(&["message"]);
        create_log_once(Severity::Error).log(&["message"]);

        assert_eq!(
            warn_spy.calls.borrow().as_slice(),
            ["My Library: message".to_string()]
        );
        assert_eq!(error_spy.calls.borrow().as_slice(), ["message".to_string()]);
    }

    // The public `log` path shares the module-level registry, so a repeated call through the
    // real dispatch is suppressed exactly as the stubbed path deduplicates it
    // (`packages/utils/src/createLogOnce.test.ts:16-18`).
    #[wasm_bindgen_test]
    fn deduplicates_repeated_calls_through_the_real_dispatch() {
        reset();
        let spy = ConsoleSpy::install("warn");
        let log_once = create_log_once(Severity::Warn);

        log_once.log(&["message"]);
        log_once.log(&["message"]);

        assert_eq!(spy.calls.borrow().as_slice(), ["message".to_string()]);
    }
}
