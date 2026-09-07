//! Shared test helpers for the crate's test modules, so each unit's tests don't re-implement
//! the same capture stubs.

use std::cell::RefCell;

use crate::create_log_once::LogOnce;

/// Records the outputs the logger emits, stubbing the console dispatch the way upstream's
/// `vi.spyOn(console, ...)` mocks do; the real dispatch is asserted by the wasm tests.
pub(crate) fn capture_log(log_once: &LogOnce, messages: &[&str]) -> Vec<String> {
    let emitted = RefCell::new(Vec::new());
    log_once.log_to(messages, &mut |output| {
        emitted.borrow_mut().push(output.to_string());
    });
    emitted.into_inner()
}
