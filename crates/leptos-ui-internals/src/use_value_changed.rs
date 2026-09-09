//! Port of `packages/react/src/internals/useValueChanged.ts:1-17` — the change-notification
//! hook (`TODO.md`, item `infra: internals`).
//!
//! Upstream is a 17-line hook: a previous-value ref seeded with the first render's value
//! (`useValueChanged.ts:7`), the callback wrapped in `useStableCallback` (`:8`), and a
//! `useIsoLayoutEffect` that fires `onChange(previousValue)` whenever the tracked value
//! fails strict `!==` against the ref, then stores the new value (`:10-16`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The tracked value is a reactive source ([`Get`]), the `use_controlled` port's
//!   convention; the layout effect ports to a [`reactive_graph::effect::Effect`] tracking
//!   it — re-running when the source changes, the committed-effect analog. The effect must
//!   run inside a reactive owner (a component), like the other hook ports.
//! - The strict `!==` detection (`useValueChanged.ts:11`) ports to `PartialEq`: JS's
//!   strict equality treats `+0` and `-0` as equal (so a `0 → -0` transition never fires),
//!   which `f64`'s `==` shares, while `NaN` is always a change, which `!=` also shares.
//!   This is deliberately *not* the [`leptos_ui_utils::are_arrays_equal::ObjectIs`]
//!   semantics — the implementation spec calls out that the module's strictness is the
//!   *opposite* of `itemEquality`'s `Object.is` (spec "Small hooks").
//! - The previous-value ref (`useValueChanged.ts:7`) keeps the *raw* value, so the payload
//!   of the next real change preserves the `-0` bit pattern — the upstream test pins
//!   `Object.is(onChange.mock.calls[0][0], -0)`
//!   (`packages/react/src/internals/useValueChanged.test.tsx:27-29`).
//! - `useStableCallback` (`:8`) is N/A: the callback is bound once per hook call, so its
//!   identity is stable by construction — the concern `useStableCallback` solves (a fresh
//!   function identity per render joining effect deps) has no Rust counterpart here.
//! - The effect body reads only the tracked value; the `onChange` invocation runs
//!   untracked (`reactive_graph::graph::subscriber::untrack`), matching React's
//!   layout-effect semantics where the callback's own signal reads do not join the
//!   effect's dependency list (`useValueChanged.ts:16` deps `[value, onChangeCallback]`).
//! - Upstream's Strict-Mode double-invocation variant
//!   (`useValueChanged.test.tsx:7`) has no analog: the port's effect runs once, and the
//!   double-run is unobservable upstream anyway (the ref-equality guard makes both
//!   invocations no-ops).
//! - `'use client'` (`useValueChanged.ts:1`) is N/A — no React Server Components boundary
//!   in Rust.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::effect::Effect;
use reactive_graph::graph::untrack;
use reactive_graph::traits::{Get, GetUntracked};

/// The upstream `useValueChanged` (`packages/react/src/internals/useValueChanged.ts:6-16`):
/// calls `on_change` with the value seen before the current one whenever the tracked value
/// changes. The first read never fires (the ref seeds from the initial value), and a
/// `0.0 → -0.0` transition is not a change — but is retained, so the *next* real change's
/// payload is the retained `-0.0`.
pub fn use_value_changed<T, V, C>(value: V, on_change: C)
where
    T: PartialEq + Clone + 'static,
    V: Get<Value = T> + GetUntracked<Value = T> + 'static,
    C: Fn(T) + 'static,
{
    // Upstream's ref seed (`useValueChanged.ts:7`): the first render's value, read
    // untracked so the seed does not join the effect's dependencies.
    let value_ref = Rc::new(RefCell::new(value.get_untracked()));

    Effect::new(move |_| {
        let current = value.get();
        untrack(|| {
            let mut value_ref = value_ref.borrow_mut();
            // Strict `!==` (`useValueChanged.ts:11-13`).
            if *value_ref != current {
                on_change(value_ref.clone());
            }
            *value_ref = current;
        });
    });
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::Set;

    use super::*;

    fn init_executor() {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            Executor::init_futures_executor();
        });
    }

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Mirrors `packages/react/src/internals/useValueChanged.test.tsx:7-31` (both the strict
    // and non-strict variants — the port's single effect run subsumes them): a `0 → -0`
    // rerender is not a change, but the retained `-0` is the payload of the next real
    // change, bit-for-bit (`Object.is(payload, -0)`).
    #[test]
    fn retains_negative_zero_as_the_previous_value_without_treating_it_as_a_change() {
        init_executor();
        let owner = in_owner();

        let source: RwSignal<f64> = RwSignal::new(0.0);
        let calls: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_callback = Rc::clone(&calls);

        use_value_changed(source, move |previous_value| {
            calls_for_callback.borrow_mut().push(previous_value);
        });

        // The initial effect run: the ref seeds, nothing fires.
        Executor::poll_local();
        assert_eq!(calls.borrow().len(), 0);

        // `0 → -0`: strict `!==` is false, so no change — and the ref retains the `-0`.
        source.set(-0.0);
        Executor::poll_local();
        assert_eq!(calls.borrow().len(), 0);

        // `→ 1`: a real change, firing once with the retained `-0` payload.
        source.set(1.0);
        Executor::poll_local();
        assert_eq!(calls.borrow().len(), 1);
        let payload = calls.borrow()[0];
        assert!(
            leptos_ui_utils::are_arrays_equal::ObjectIs::object_is(&payload, &-0.0),
            "the payload must be the retained -0, got {payload}"
        );

        owner.cleanup();
    }

    // Port-owned pin for the ordinary change path
    // (`packages/react/src/internals/useValueChanged.ts:10-16`): each real change fires the
    // callback once with the value from before it, in order.
    #[test]
    fn fires_once_per_real_change_with_the_previous_value() {
        init_executor();
        let owner = in_owner();

        let source: RwSignal<&'static str> = RwSignal::new("first");
        let calls: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_callback = Rc::clone(&calls);

        use_value_changed(source, move |previous_value| {
            calls_for_callback.borrow_mut().push(previous_value);
        });

        Executor::poll_local();
        assert!(calls.borrow().is_empty(), "the seed never fires");

        source.set("second");
        Executor::poll_local();
        source.set("third");
        Executor::poll_local();
        assert_eq!(&*calls.borrow(), &["first", "second"]);

        // A same-value write is not a change (strict `!==` is false) — upstream's dep
        // comparison would not even re-run the effect; the port's re-run is a no-op.
        source.set("third");
        Executor::poll_local();
        assert_eq!(&*calls.borrow(), &["first", "second"]);

        owner.cleanup();
    }
}
