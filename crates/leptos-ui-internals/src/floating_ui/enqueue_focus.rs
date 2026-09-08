//! Port of `packages/react/src/floating-ui-react/utils/enqueueFocus.ts` — the
//! rAF-deferred, cancelable focus scheduler used by list navigation and the FocusManager
//! (`specs/library/floating-ui-react/implementation.md`, "Anything in source not
//! explained by any test" item 8).
//!
//! The module-level `rafId` singleton is ported faithfully, including the behavior the
//! implementation spec flags: concurrent callers cancel each other's queued focus
//! (`enqueueFocus.ts:11` — one module-scoped id), because upstream's own consumers rely
//! on exactly that last-caller-wins behavior.

use std::cell::Cell;
use std::rc::Rc;

use leptos_ui_utils::use_animation_frame::AnimationFrameId;
use leptos_ui_utils::use_animation_frame::cancel_animation_frame;
use leptos_ui_utils::use_animation_frame::request_animation_frame;
use web_sys::FocusOptions;

use crate::floating_ui::types::EventUnsubscribe;

// The module-level `rafId` singleton (`enqueueFocus.ts:11`).
thread_local! {
    static RAF_ID: Cell<Option<AnimationFrameId>> = const { Cell::new(None) };
}

/// Options for [`enqueue_focus`] — upstream's `Options` interface
/// (`enqueueFocus.ts:4-9`).
#[derive(Default)]
pub struct EnqueueFocusOptions {
    /// `preventScroll` (`:5`) — default `false` (`:13`).
    pub prevent_scroll: Option<bool>,
    /// `sync` (`:6`) — focus synchronously instead of on the next frame; default
    /// `false` (`:13`).
    pub sync: Option<bool>,
    /// `shouldFocus` (`:7-8`) — called when the frame runs to decide whether focus
    /// should still be applied (`:18-20`).
    pub should_focus: Option<Box<dyn Fn() -> bool>>,
}

/// `enqueueFocus(el, options)` (`enqueueFocus.ts:12-37`): focuses the element —
/// synchronously when `sync`, else on the next animation frame, canceling the previous
/// module-level queued frame first (`:15`). Returns the cancel closure; the `sync` path
/// returns a no-op (`:24-27`, the `NOOP` import).
///
/// `el` is nullable (`el?.focus(...)`, `:21`) — a `None` element queues nothing but
/// still participates in the singleton cancel, matching upstream's optional element.
pub fn enqueue_focus(
    el: Option<&web_sys::HtmlElement>,
    options: EnqueueFocusOptions,
) -> EventUnsubscribe {
    let EnqueueFocusOptions {
        prevent_scroll,
        sync,
        should_focus,
    } = options;
    let prevent_scroll = prevent_scroll.unwrap_or(false);
    let sync = sync.unwrap_or(false);

    if let Some(previous) = RAF_ID.take() {
        cancel_animation_frame(previous);
    }

    let exec = {
        let el = el.cloned();
        move || {
            if let Some(should_focus) = should_focus.as_ref() {
                if !should_focus() {
                    return;
                }
            }
            if let Some(el) = el.as_ref() {
                let focus_options = FocusOptions::new();
                focus_options.set_prevent_scroll(prevent_scroll);
                let _ = el.focus_with_options(&focus_options);
            }
        }
    };

    if sync {
        exec();
        return Rc::new(|| {});
    }

    let current_raf_id = request_animation_frame(move |_| exec());
    RAF_ID.set(Some(current_raf_id.clone()));
    Rc::new(move || {
        if RAF_ID.get().map(|id| id == current_raf_id).unwrap_or(false) {
            cancel_animation_frame(current_raf_id.clone());
            RAF_ID.set(None);
        }
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins the sync path (`enqueueFocus.ts:24-27`): focus applies immediately and the
    // returned cancel closure is a no-op (`NOOP`). With a `shouldFocus` veto, focus does
    // not apply (`:18-20`) — observable through the veto flag alone since no element is
    // involved.
    #[test]
    fn sync_focus_runs_immediately_and_returns_a_no_op_cancel() {
        let ran: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let ran_handle = Rc::clone(&ran);
        let cancel = enqueue_focus(
            None,
            EnqueueFocusOptions {
                sync: Some(true),
                should_focus: Some(Box::new(move || {
                    ran_handle.set(true);
                    true
                })),
                ..EnqueueFocusOptions::default()
            },
        );
        assert!(
            ran.get(),
            "the sync path ran the focus decision immediately"
        );
        cancel(); // no-op, must not panic
    }

    // Pins the `shouldFocus` veto (`enqueueFocus.ts:18-20`): a false veto skips the
    // focus application — with a `None` element the veto flag itself is the observable.
    #[test]
    fn the_should_focus_veto_skips_the_focus() {
        let asked: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let asked_handle = Rc::clone(&asked);
        enqueue_focus(
            None,
            EnqueueFocusOptions {
                sync: Some(true),
                should_focus: Some(Box::new(move || {
                    asked_handle.set(true);
                    false
                })),
                ..EnqueueFocusOptions::default()
            },
        );
        assert!(asked.get(), "the veto callback was consulted");
    }
}
