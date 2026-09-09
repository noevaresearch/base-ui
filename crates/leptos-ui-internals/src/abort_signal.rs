//! Port of the platform `AbortController`/`AbortSignal` pair as consumed by
//! `packages/react/src/internals/useOpenChangeComplete.tsx:20` and
//! `packages/react/src/internals/useAnimationsFinished.ts` (`TODO.md`, item `infra:
//! internals`; the transition/animation checkpoint recorded in that entry's note).
//!
//! Upstream does not define these — they are the web platform's, and the only surface this
//! unit consumes is:
//!
//! - `new AbortController()` + `controller.abort()` on cleanup
//!   (`useOpenChangeComplete.tsx:20-26`) — the per-run cancellation handle;
//! - `signal.aborted` reads (`useAnimationsFinished.ts:85`, `:102`, `:107`);
//! - one `abort` listener with `{ once: true }`, used to disconnect the starting-style
//!   `MutationObserver` (`useAnimationsFinished.ts:152`).
//!
//! ## Rust adaptation
//!
//! The controller and the signal collapse into one clonable handle ([`AbortSignal`]):
//! `abort()` is the controller side, `aborted()`/`add_abort_listener` the signal side, and
//! `Clone` shares one registration (the `PopupTriggerMap` shared-identity precedent) — a
//! consumer that "creates a controller, passes the signal around, aborts on cleanup"
//! clones the handle and calls `abort()` on its own copy. A `web_sys::AbortSignal` was
//! rejected because the host test builds have no JS realm (the host suite pins the
//! abort-recheck and once-listener semantics deterministically), and every later consumer
//! in this crate gets one consistent type on both targets.
//!
//! Listeners fire at most once (upstream's `{ once: true }` is the only subscription shape
//! this unit uses), and a listener registered after `abort()` never fires — matching the
//! platform, where the `abort` event has already dispatched.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Default)]
struct Inner {
    aborted: Cell<bool>,
    listeners: RefCell<Vec<Rc<dyn Fn()>>>,
}

/// The collapsed `AbortController`/`AbortSignal` handle (module docs). Clones share the
/// aborted flag and the listener registry.
#[derive(Clone, Default)]
pub struct AbortSignal {
    inner: Rc<Inner>,
}

impl AbortSignal {
    /// `new AbortController().signal` (`useOpenChangeComplete.tsx:20`) — the port has one
    /// constructor for the collapsed pair.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner {
                aborted: Cell::new(false),
                listeners: RefCell::new(Vec::new()),
            }),
        }
    }

    /// `signal.aborted` (`useAnimationsFinished.ts:85`, `:102`, `:107`).
    pub fn aborted(&self) -> bool {
        self.inner.aborted.get()
    }

    /// `controller.abort()` (`useOpenChangeComplete.tsx:25`). Idempotent: the platform's
    /// `abort()` on an already-aborted controller does not re-dispatch the event.
    pub fn abort(&self) {
        if self.inner.aborted.replace(true) {
            return;
        }
        for listener in self.inner.listeners.borrow_mut().drain(..) {
            listener();
        }
    }

    /// `signal.addEventListener('abort', fn, { once: true })`
    /// (`useAnimationsFinished.ts:152`). A listener registered after `abort()` never fires —
    /// the platform event has already dispatched.
    pub fn add_abort_listener(&self, listener: impl Fn() + 'static) {
        if self.inner.aborted.get() {
            return;
        }
        self.inner.listeners.borrow_mut().push(Rc::new(listener));
    }
}

impl std::fmt::Debug for AbortSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbortSignal")
            .field("aborted", &self.aborted())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    // Port-owned pins for the consumed surface (`useOpenChangeComplete.tsx:20-26`,
    // `useAnimationsFinished.ts:85`, `:102`, `:107`, `:152`): flag reads, idempotent abort,
    // once-only listeners, and the too-late registration rule.
    #[test]
    fn abort_sets_the_flag_and_fires_listeners_exactly_once() {
        let signal = AbortSignal::new();
        assert!(!signal.aborted());

        let calls = Rc::new(RefCell::new(0));
        let calls_for_listener = Rc::clone(&calls);
        signal.add_abort_listener(move || {
            *calls_for_listener.borrow_mut() += 1;
        });

        signal.abort();
        assert!(signal.aborted());
        signal.abort();
        assert_eq!(
            *calls.borrow(),
            1,
            "abort() is idempotent; the listener fires once"
        );

        // A clone shares the aborted flag (the collapsed controller/signal pair).
        let clone = signal.clone();
        assert!(clone.aborted());
    }

    #[test]
    fn a_listener_registered_after_abort_never_fires() {
        let signal = AbortSignal::new();
        signal.abort();

        let calls = Rc::new(RefCell::new(0));
        let calls_for_listener = Rc::clone(&calls);
        signal.add_abort_listener(move || {
            *calls_for_listener.borrow_mut() += 1;
        });

        assert_eq!(*calls.borrow(), 0, "the abort event has already dispatched");
    }
}
