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
use leptos_ui_internals::use_transition_status::{UseTransitionStatus, use_transition_status};

/// The generated-id fallback — the same generator shape the accordion item uses
/// (`new_base_ui_id` in `accordion/mod.rs`), so ids stay unique across units.
pub fn new_base_ui_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("base-ui-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// The transition-status derivation over a leptos-tracked `open` source: runs the
/// real [`use_transition_status`] inside a dedicated rg-0.2 owner (kept alive with
/// the returned handle) and mirrors the status into a leptos signal the views
/// track. The rg-0.2 `mounted` mirror rides along — the `FieldError` unmount gate
/// writes it (upstream's `setMounted`).
pub fn transition_status_signal(
    open: impl Fn() -> bool + Clone + 'static,
) -> LeptosTransitionStatus {
    // The rg-0.2 open mirror: written by a leptos effect, read by the hook. The
    // constructor is fully qualified — the prelude glob's `RwSignal` is leptos's.
    let open_mirror = reactive_graph::signal::RwSignal::new(open());
    // The leptos status mirror: written by the rg-0.2 effect, read by the views.
    let status_mirror: leptos::prelude::RwSignal<
        Option<leptos_ui_internals::use_transition_status::TransitionStatus>,
    > = RwSignal::new(None);
    // The leptos mounted mirror: written through the returned handle (the
    // `setMounted` slot), read by the views' mount gate.
    let mounted_mirror: leptos::prelude::RwSignal<bool> = RwSignal::new(open());

    // The hook's rg-0.2 scope: a fresh owner per derivation, forgotten to outlive the
    // page (the docs-app consumer convention). The `open` closure rides through the
    // generic bound — the hook's `use_iso_layout_effect` reads it tracked inside the
    // rg effects.
    let hook_owner = reactive_graph::owner::Owner::new();
    let hook = hook_owner.with(|| {
        // The mode flags ride rg-0.2 signals (the hook's generic bounds demand
        // reactive sources — the `bool` literals the prior cut passed cannot satisfy
        // `Get<Value = bool>`).
        let enable_idle_state = reactive_graph::signal::RwSignal::new(true);
        let defer_ending_state = reactive_graph::signal::RwSignal::new(false);
        use_transition_status(
            open_mirror.clone(),
            enable_idle_state,
            defer_ending_state,
            false, // animate_initial_open
        )
    });
    let UseTransitionStatus {
        mounted,
        transition_status,
    } = hook;

    // Seed the leptos mirrors from the hook's initializer values (the hook's
    // `useState` initializers read untracked at hook time).
    let seed_status = reactive_graph::traits::GetUntracked::get_untracked(&transition_status);
    status_mirror.set(seed_status);
    let seed_mounted = reactive_graph::traits::GetUntracked::get_untracked(&mounted);
    mounted_mirror.set(seed_mounted);

    // leptos → rg-0.2: the open mirror follows the tracked source.
    Effect::new({
        let open_mirror = open_mirror.clone();
        move |_| {
            reactive_graph::traits::Set::set(&open_mirror, open());
        }
    });
    // rg-0.2 → leptos: the status mirror follows the hook's signal.
    reactive_graph::effect::Effect::new({
        let status_mirror = status_mirror.clone();
        move |_| {
            let status = reactive_graph::traits::Get::get(&transition_status);
            status_mirror.set(status);
        }
    });
    // rg-0.2 → leptos: the mounted mirror follows the hook (the hook flips `mounted`
    // itself when `open` goes true — upstream `useTransitionStatus`'s mount run).
    reactive_graph::effect::Effect::new({
        let mounted_mirror = mounted_mirror.clone();
        move |_| {
            let next = reactive_graph::traits::Get::get(&mounted);
            mounted_mirror.set(next);
        }
    });

    std::mem::forget(hook_owner);
    LeptosTransitionStatus {
        inner: status_mirror,
        mounted_rg: mounted,
        mounted_leptos: mounted_mirror,
    }
}

/// The leptos-side handle the parts track: the status mirror (kept in lockstep with
/// the rg hook by the mirror effects) plus the two-way `mounted` slot.
#[derive(Clone)]
pub struct LeptosTransitionStatus {
    inner: leptos::prelude::RwSignal<
        Option<leptos_ui_internals::use_transition_status::TransitionStatus>,
    >,
    /// The hook's rg-0.2 `mounted` writable — the `setMounted` target.
    mounted_rg: reactive_graph::signal::RwSignal<bool>,
    /// The leptos-side `mounted` mirror the views' mount gate tracks.
    mounted_leptos: leptos::prelude::RwSignal<bool>,
}

impl LeptosTransitionStatus {
    /// The tracked status read.
    pub fn get(&self) -> Option<leptos_ui_internals::use_transition_status::TransitionStatus> {
        self.inner.get()
    }

    /// The tracked `mounted` read — the view's render gate.
    pub fn mounted(&self) -> bool {
        self.mounted_leptos.get()
    }

    /// The untracked `mounted` read — effect-internal guards.
    pub fn mounted_untracked(&self) -> bool {
        self.mounted_leptos.get_untracked()
    }

    /// `setMounted(next)` — writes the rg hook's `mounted` writable; its own effects
    /// drive the status transitions off it, and the mirrors (the rg→leptos mounted
    /// effect above) keep every read in lockstep.
    pub fn set_mounted(&self, value: bool) {
        reactive_graph::traits::Set::set(&self.mounted_rg, value);
    }
}
