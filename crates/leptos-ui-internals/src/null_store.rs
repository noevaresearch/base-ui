//! Port of `packages/react/src/utils/NullStore.ts` — the inert `ReactStore` fallback.
//!
//! "A `ReactStore` whose state never changes. Useful for fallback stores that need to
//! support normal store reads while detached from the component that owns real state.
//! Context values may still contain mutable refs or maps." (`NullStore.ts:5-9` — the
//! five popup families use it as the detached/fallback store their handles and triggers
//! read from before (or after) a live root exists.)
//!
//! All four mutators are overridden explicitly upstream, with the rationale carried
//! here verbatim in spirit (`NullStore.ts:16-19`): "`update`/`set`/`notifyAll` funnel
//! through `setState` in the base `Store`, so overriding `setState` alone would
//! neutralize them today. They are overridden explicitly so the store stays inert even
//! if a future base-class change stops routing a mutator through `setState`." The
//! read surface (`getSnapshot`, `subscribe`, `observe`, the context bag, the reactive
//! hooks) keeps working — only writes are neutralized.
//!
//! Rust adaptations:
//! - Upstream subclasses `ReactStore`; the port wraps an `Rc<ReactStore<State,
//!   Context>>` behind `Deref`, following the `FloatingRootStore` convention. The
//!   no-op mutators are inherent methods, which shadow the deref-target mutators in
//!   method resolution — the inertness contract holds at the type's own call sites.
//!   Where a consumer expects `&ReactStore`, `Deref` supplies it; an `Rc<NullStore>`
//!   is not an `Rc<ReactStore>` (no object-subclass coercion), so consumers share it
//!   through the [`NullStore`] handle or an enum at the family level, the same shape
//!   the `PopupHandle` attachment port will have to pick.

use std::ops::Deref;
use std::rc::Rc;

use leptos_ui_utils::react_store::ReactStore;

/// Port of `NullStore<State, Context, Selectors>` (`NullStore.ts:11-25`). The selectors
/// table is a type-level concern upstream; the port's selector functions are free
/// functions over the state (the `FloatingRootStore` port convention), so no third
/// parameter is needed here.
pub struct NullStore<State, Context = ()> {
    inner: Rc<ReactStore<State, Context>>,
}

impl<State: 'static> NullStore<State, ()> {
    /// Upstream `new NullStore(state)` — state seeded, empty context.
    pub fn new(state: State) -> Self {
        Self {
            inner: Rc::new(ReactStore::new(state)),
        }
    }
}

impl<State: 'static, Context: 'static> NullStore<State, Context> {
    /// Upstream `new NullStore(state, context)` — the context bag (mutable refs/maps)
    /// stays live and shared; only state writes are inert (`NullStore.ts:8-9`).
    pub fn with_context(state: State, context: Context) -> Self {
        Self {
            inner: Rc::new(ReactStore::with_context(state, context)),
        }
    }

    /// Overridden no-op (`NullStore.ts:19`): `setState(_newState: State) {}`.
    pub fn set_state(&self, _new_state: Rc<State>) {}

    /// Overridden no-op (`NullStore.ts:21`): `update(_changes: Pick<State, Key>) {}`.
    /// The port's [`leptos_ui_utils::store::Store::update`] takes the apply closure;
    /// the argument is discarded without invoking it.
    pub fn update(&self, _apply: impl FnOnce(&mut State, &State) -> bool) {}

    /// Overridden no-op (`NullStore.ts:23`): `set(_key, _value) {}` — the base
    /// `set_field` mutator, neutralized.
    pub fn set_field<V: PartialEq>(&self, _accessor: impl FnOnce(&mut State) -> &mut V, _value: V) {
    }

    /// Overridden no-op (`NullStore.ts:25`): `notifyAll() {}`.
    pub fn notify_all(&self) {}
}

impl<State, Context> Deref for NullStore<State, Context> {
    type Target = ReactStore<State, Context>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The inert-store contract (implementation.md, "Submodules with no test anywhere":
    // "all mutators no-oped deliberately"): every write leaves the snapshot untouched
    // and fires no listener, while reads and the context bag keep working.
    #[test]
    fn mutators_are_no_ops_and_reads_still_work() {
        let store =
            NullStore::with_context(vec!["initial".to_string()], std::cell::RefCell::new(0u32));

        let notified = Rc::new(std::cell::Cell::new(0u32));
        let counter = Rc::clone(&notified);
        let unsubscribe = store.subscribe(move |_| counter.set(counter.get() + 1));

        store.set_state(Rc::new(vec!["written".to_string()]));
        store.update(|_, _| true);
        store.set_field(|state| &mut state[0], "written".to_string());
        store.notify_all();

        assert_eq!(*store.get_snapshot(), vec!["initial".to_string()]);
        assert_eq!(notified.get(), 0);

        // The context bag stays live (`NullStore.ts:8-9`).
        store.context.replace_with(|n| *n + 1);
        assert_eq!(*store.context.borrow(), 1);

        unsubscribe();
    }

    // A detached fallback store still supports the read path the handles use —
    // `select` over the state and the unsubscribe-returning `subscribe`.
    #[test]
    fn selects_over_the_seeded_state() {
        #[derive(PartialEq)]
        struct State {
            open: bool,
        }

        let store = NullStore::new(State { open: false });
        let open = store.select(|state: &State| state.open);
        assert!(!open);
    }
}
