//! Small shared helpers of the field unit — the id generator and the
//! transition-status bridge.
//!
//! The internals' `use_transition_status` is the real ported hook; its inputs/outputs
//! are rg-0.2 sources, so the leptos-side callers feed it through the bridge pattern
//! the direction-provider page verified: run the hook inside a dedicated rg-0.2 owner
//! and mirror the status into a leptos signal with an rg-0.2 effect (the
//! leptos→rg-0.2 open mirror is a leptos effect writing the rg-0.2 writable — one
//! effect per runtime, each created inside its own owner).

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get as _, GetUntracked as _, Set as _};

use leptos_ui_internals::use_transition_status::{UseTransitionStatus, use_transition_status};

/// The generated-id fallback — the same generator shape the accordion item uses
/// (`new_base_ui_id` in `accordion/mod.rs`), so ids stay unique across units.
pub fn new_base_ui_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("base-ui-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// The transition-status derivation over a leptos-tracked `open` source: runs the
/// real [`use_transition_status`] inside a dedicated rg-0.2 owner (kept alive with
/// the returned signal) and mirrors the status into a leptos `RwSignal` the views
/// track.
pub fn transition_status_signal(
    open: impl Fn() -> bool + Clone + 'static,
) -> LeptosTransitionStatus {
    // The rg-0.2 open mirror: written by a leptos effect, read by the hook.
    let open_mirror: RwSignal<bool> = RwSignal::new(open());
    // The leptos status mirror: written by the rg-0.2 effect, read by the views.
    let status_mirror: RwSignal<
        Option<leptos_ui_internals::use_transition_status::TransitionStatus>,
    > = RwSignal::new(None);

    // The hook's rg-0.2 scope: a fresh owner per derivation, forgotten to outlive the
    // page (the docs-app consumer convention).
    let hook_owner = reactive_graph::owner::Owner::new();
    let UseTransitionStatus { transition_status, .. } = hook_owner.with(|| {
        use_transition_status(
            open_mirror.clone(),
            true,  // enable_idle_state
            true,  // defer_ending_state
            false, // animate_initial_open
        )
    });

    // leptos → rg-0.2: the open mirror follows the tracked source.
    Effect::new({
        let open_mirror = open_mirror.clone();
        move |_| {
            open_mirror.set(open());
        }
    });
    // rg-0.2 → leptos: the status mirror follows the hook's signal.
    reactive_graph::effect::Effect::new({
        let status_mirror = status_mirror.clone();
        move |_| {
            let status = transition_status.get();
            status_mirror.set(status);
        }
    });

    std::mem::forget(hook_owner);
    LeptosTransitionStatus { inner: status_mirror }
}

/// The leptos-side handle the parts track.
#[derive(Clone)]
pub struct LeptosTransitionStatus {
    inner: leptos::prelude::RwSignal<
        Option<leptos_ui_internals::use_transition_status::TransitionStatus>,
    >,
}

impl LeptosTransitionStatus {
    /// The tracked read.
    pub fn get(
        &self,
    ) -> Option<leptos_ui_internals::use_transition_status::TransitionStatus> {
        self.inner.get()
    }
}
