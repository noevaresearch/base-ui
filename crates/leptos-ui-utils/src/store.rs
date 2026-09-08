//! Port of `packages/utils/src/store/Store.ts` and the store-observation half of
//! `packages/utils/src/store/ReactStore.ts` (Base UI Phase A util), the plain
//! observable state container the component port hangs per-component state off of.
//! The React-render-phase members of the unit live in [`crate::react_store`].
//!
//! Upstream behavior claims are cited to the unit's own test suite
//! (`packages/utils/src/store/Store.test.ts` and the `observeSelector` suite in
//! `packages/utils/src/store/ReactStore.test.tsx`), which
//! `specs/utils/store.md` names as the source of truth.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream state is a mutable object property whose identity is compared with
//!   `===`/`Object.is` (`packages/utils/src/store/Store.ts:66`,
//!   `packages/utils/src/store/Store.ts:93`). The port stores the state as an
//!   [`Rc`] handle: identity comparisons become [`Rc::ptr_eq`], listeners and
//!   readers receive a cheap handle clone, and `notifyAll`'s "renew the state
//!   reference" (`packages/utils/src/store/Store.ts:115-118`) becomes wrapping a
//!   clone of the contents in a fresh `Rc`.
//! - Upstream `set(key, value)` addresses one property by name
//!   (`packages/utils/src/store/Store.ts:106-110`). Rust struct fields are not
//!   runtime-addressable, so [`Store::set_field`] takes a field accessor closure
//!   returning a `&mut` to the written field and derives the same no-op skip from
//!   the field's own `PartialEq` — the same "compare the written value, commit
//!   once, skip identical writes" contract, checked per field instead of per key.
//!   Upstream compares with `Object.is`, under which `NaN === NaN` skips the
//!   write; Rust's `PartialEq` reports `NaN != NaN`, so a `NaN` write commits
//!   where upstream would skip it — a strictly more-notifying direction that
//!   cannot drop a legitimate notification.
//! - Upstream `update(changes)` iterates the given keys with `Object.is` and
//!   commits at most once if any differs (`packages/utils/src/store/Store.ts:91-98`).
//!   [`Store::update`] hands the caller a mutable clone and the current state and
//!   expects it to report whether anything changed, keeping the single-commit /
//!   no-op-skip contract while the per-field comparisons live at the call site.
//! - Upstream iterates its `listeners` `Set` live, so a listener subscribed during
//!   a notification pass would receive that same pass. The port snapshots the
//!   listener list before dispatching: a `RefCell` cannot be re-borrowed while a
//!   listener it is iterating calls back into the store, and the re-entrancy test
//!   (`packages/utils/src/store/Store.test.ts:112-131`) pins that the observable
//!   outcome — final state delivered once to later listeners, stale pass aborted —
//!   is unchanged. A listener subscribing mid-pass sees only later passes, which
//!   upstream's live iteration would have delivered into the current one.
//! - Upstream dedups subscribers through the `Set`: subscribing the identical
//!   function twice yields one entry. The port appends every subscriber (an
//!   `Rc<dyn Fn>` has no usable equality), so subscribing the same closure object
//!   twice notifies twice; each unsubscribe handle still removes only its own
//!   registration.
//! - `Store.create` constructs "the class it is called on" so a subclass
//!   instantiation test passes (`packages/utils/src/store/Store.test.ts:25-37`).
//!   Rust has no class hierarchy to dispatch on; [`Store::create`] is a plain
//!   alias of [`Store::new`] and the subclass case is N/A.
//! - Upstream `observe(keyOrSelector, listener)` resolves named selectors through
//!   the `ReactStore`'s selectors registry (`packages/utils/src/store/ReactStore.ts:233-243`).
//!   The port passes selector functions directly at the call site (the registry
//!   exists only to bundle functions for JS call sites), so [`Store::observe`]
//!   lives on the plain store and accepts any `Fn(&State) -> V`. The listener's
//!   third argument, the store instance (`packages/utils/src/store/ReactStore.test.tsx:511-515`),
//!   has no Rust equivalent on a borrowed `self`; callers capture the store handle
//!   they already hold — the same object, so the identity the test asserts holds
//!   trivially.
//! - The `use(selector, ...args)` render-phase member
//!   (`packages/utils/src/store/Store.ts:120-125`) is ported as the reactive
//!   bridge in [`crate::react_store`], which is where the React hook layer landed.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The upstream `Listener<State>` (`packages/utils/src/store/Store.ts:3`): called
/// with the new state whenever a notifying update lands
/// (`packages/utils/src/store/Store.test.ts:47-48`). The argument is the state
/// handle so identity-sensitive listeners can compare it.
pub type StoreListener<State> = Rc<dyn Fn(Rc<State>)>;

/// The unsubscribe function `subscribe`/`observe` return
/// (`packages/utils/src/store/Store.ts:46-51`,
/// `packages/utils/src/store/ReactStore.test.tsx:423-427`). Callable repeatedly:
/// calls past the first are no-ops, matching upstream's `Set.delete` idempotence.
pub type StoreUnsubscribe = Rc<dyn Fn()>;

/// A data store implementation that allows subscribing to state changes and
/// updating the state — the port of upstream `Store<State>`
/// (`packages/utils/src/store/Store.ts:9-126`). It uses an observer pattern to
/// notify subscribers when the state changes.
pub struct Store<State> {
    /// The current state, held by handle so identity stays observable
    /// (`packages/utils/src/store/Store.ts:27`).
    state: RefCell<Rc<State>>,
    /// The upstream `listeners` set (`packages/utils/src/store/Store.ts:29`), shared
    /// with the unsubscribe handles so they can remove their own registration after
    /// the store handle itself is gone.
    listeners: Rc<RefCell<Vec<StoreListener<State>>>>,
    /// The upstream `updateTick` recursion guard (`packages/utils/src/store/Store.ts:32`).
    update_tick: Cell<u64>,
}

impl<State: 'static> Store<State> {
    /// Direct construction — upstream `new Store(state)`
    /// (`packages/utils/src/store/Store.test.ts:40-45`).
    pub fn new(state: State) -> Self {
        Self {
            state: RefCell::new(Rc::new(state)),
            listeners: Rc::new(RefCell::new(Vec::new())),
            update_tick: Cell::new(0),
        }
    }

    /// Upstream `Store.create(initialState)`
    /// (`packages/utils/src/store/Store.test.ts:8-13`). Upstream's subclass
    /// dispatch has no Rust counterpart (see the module docs), so this is an
    /// alias of [`Store::new`].
    pub fn create(state: State) -> Self {
        Self::new(state)
    }

    /// Returns the current state — upstream `getSnapshot()`
    /// (`packages/utils/src/store/Store.ts:56-58`), which reads the `state`
    /// property (`packages/utils/src/store/Store.test.ts:12`). Callers can use
    /// the handle directly (to avoid subscribing) in effects or event handlers.
    pub fn get_snapshot(&self) -> Rc<State> {
        self.state.borrow().clone()
    }

    /// Registers a listener called whenever the store's state changes and returns
    /// its unsubscribe function (`packages/utils/src/store/Store.ts:46-51`).
    pub fn subscribe(&self, listener: impl Fn(Rc<State>) + 'static) -> StoreUnsubscribe {
        let listener: StoreListener<State> = Rc::new(listener);
        self.listeners.borrow_mut().push(Rc::clone(&listener));
        let listeners = Rc::clone(&self.listeners);
        Rc::new(move || {
            listeners.borrow_mut().retain(|l| !Rc::ptr_eq(l, &listener));
        })
    }

    /// Observes changes derived through `selector` and calls `listener` when the
    /// selected value changes — the port of upstream `observe`
    /// (`packages/utils/src/store/ReactStore.ts:233-257`). The listener fires once
    /// immediately with the current selector result in both slots
    /// (`packages/utils/src/store/ReactStore.test.tsx:412-413`,
    /// `packages/utils/src/store/ReactStore.test.tsx:440-442`), then only when the
    /// selector result changes, with the previous result as the second argument
    /// (`packages/utils/src/store/ReactStore.test.tsx:457-462`); a change that
    /// leaves the result unchanged fires nothing
    /// (`packages/utils/src/store/ReactStore.test.tsx:477-479`). Unsubscribing
    /// stops all further calls (`packages/utils/src/store/ReactStore.test.tsx:530-536`).
    pub fn observe<V: PartialEq + Clone + 'static>(
        &self,
        selector: impl Fn(&State) -> V + 'static,
        listener: impl Fn(&V, &V) + 'static,
    ) -> StoreUnsubscribe {
        let prev_value = selector(&self.get_snapshot());
        listener(&prev_value, &prev_value);

        // Upstream assigns `prevValue = nextValue` *before* calling the listener
        // (`packages/utils/src/store/ReactStore.ts:249-256`), so the previous result is
        // never mutated while the listener runs and a listener that re-enters the store
        // sees the already-updated value. The borrow is dropped before the call for the
        // same reason: the `RefCell` here only guards the captured slot, never the call.
        let prev_value = Rc::new(RefCell::new(prev_value));
        self.subscribe(move |state| {
            let next_value = selector(&state);
            let should_fire = next_value != *prev_value.borrow();
            if should_fire {
                let old_value = (*prev_value.borrow()).clone();
                *prev_value.borrow_mut() = next_value.clone();
                listener(&next_value, &old_value);
            }
        })
    }
}

impl<State: Clone + 'static> Store<State> {
    /// Updates the entire store's state and notifies all registered listeners —
    /// upstream `setState` (`packages/utils/src/store/Store.ts:65-82`). Passing
    /// the current state handle notifies no one
    /// (`packages/utils/src/store/Store.test.ts:52-60`); a listener that triggers
    /// a nested `set_state` makes the outer pass abort instead of delivering the
    /// stale state, because the nested call already notified every listener
    /// (`packages/utils/src/store/Store.test.ts:112-131`).
    pub fn set_state(&self, next: Rc<State>) {
        if Rc::ptr_eq(&self.get_snapshot(), &next) {
            return;
        }

        *self.state.borrow_mut() = Rc::clone(&next);
        self.update_tick.set(self.update_tick.get() + 1);

        let current_tick = self.update_tick.get();
        let listeners = self.listeners.borrow().clone();
        for listener in listeners {
            if current_tick != self.update_tick.get() {
                // If the tick has changed, a recursive `set_state` call has been
                // made, and it has already notified all listeners.
                return;
            }
            listener(Rc::clone(&next));
        }
    }

    /// Writes one field and notifies listeners if the value has changed —
    /// upstream `set(key, value)` (`packages/utils/src/store/Store.ts:106-110`):
    /// one write, one notification, and a second write with the same value is
    /// skipped (`packages/utils/src/store/Store.test.ts:73-84`).
    pub fn set_field<V: PartialEq>(&self, accessor: impl FnOnce(&mut State) -> &mut V, value: V) {
        let current = self.get_snapshot();
        let mut next = (*current).clone();
        let field = accessor(&mut next);
        if *field == value {
            return;
        }
        *field = value;
        self.set_state(Rc::new(next));
    }

    /// Merges changes into the current state and notifies listeners if there are
    /// changes — upstream `update(changes)` (`packages/utils/src/store/Store.ts:91-98`):
    /// merges changed fields, notifies once, and skips no-op updates
    /// (`packages/utils/src/store/Store.test.ts:86-97`). `apply` receives the
    /// mutable clone and the current state and reports whether any field it wrote
    /// differs, standing in for upstream's per-key `Object.is` comparisons.
    pub fn update(&self, apply: impl FnOnce(&mut State, &State) -> bool) {
        let current = self.get_snapshot();
        let mut next = (*current).clone();
        if apply(&mut next, &current) {
            self.set_state(Rc::new(next));
        }
    }

    /// Gives the state a new reference and updates all registered listeners —
    /// upstream `notifyAll()` (`packages/utils/src/store/Store.ts:115-118`): the
    /// state handle becomes a new `Rc` with equal content
    /// (`packages/utils/src/store/Store.test.ts:99-110`).
    pub fn notify_all(&self) {
        let current = self.get_snapshot();
        self.set_state(Rc::new((*current).clone()));
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    /// The upstream suite's shared state shape (`packages/utils/src/store/Store.test.ts:4`).
    #[derive(Clone, Debug, PartialEq)]
    struct State {
        value: i64,
        label: String,
    }

    fn state(value: i64, label: &str) -> State {
        State {
            value,
            label: label.to_string(),
        }
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:8-13`: the factory returns a
    // store seeded with the given state. The `toBeInstanceOf` half is structural in
    // Rust (see the module docs on `create`).
    #[test]
    fn create_returns_a_store_seeded_with_the_given_state() {
        let store = Store::create(state(1, "a"));

        assert_eq!(*store.get_snapshot(), state(1, "a"));
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:15-23`: each call produces an
    // independent instance; mutating one does not affect another.
    #[test]
    fn produces_an_independent_instance_per_call() {
        let first = Store::create(state(0, "a"));
        let second = Store::create(state(0, "a"));

        first.set_field(|s| &mut s.value, 1);

        assert_eq!(first.get_snapshot().value, 1);
        assert_eq!(second.get_snapshot().value, 0);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:40-50`: `setState` notifies
    // every subscriber once with the new state.
    #[test]
    fn notifies_subscribers_with_the_new_state() {
        let store = Store::new(state(0, "a"));
        let calls: Rc<RefCell<Vec<Rc<State>>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&calls);
        store.subscribe(move |state| sink.borrow_mut().push(state));

        store.set_state(Rc::new(state(1, "a")));

        let calls = calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(**calls.first().unwrap(), state(1, "a"));
        assert_eq!(store.get_snapshot().value, 1);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:52-60`: `setState` receiving
    // the current state reference (the same handle) notifies no one.
    #[test]
    fn does_not_notify_when_set_state_receives_the_current_state_reference() {
        let store = Store::new(state(0, "a"));
        let count = Rc::new(CellCount::default());
        let sink = Rc::clone(&count);
        store.subscribe(move |_| sink.increment());

        store.set_state(store.get_snapshot());

        assert_eq!(count.get(), 0);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:62-71`: unsubscribing stops
    // notifications.
    #[test]
    fn unsubscribing_stops_notifications() {
        let store = Store::new(state(0, "a"));
        let count = Rc::new(CellCount::default());
        let sink = Rc::clone(&count);
        let unsubscribe = store.subscribe(move |_| sink.increment());

        unsubscribe();
        store.set_field(|s| &mut s.value, 1);

        assert_eq!(count.get(), 0);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:73-84`: `set` writes a single
    // field, notifies once, and skips a same-value write.
    #[test]
    fn set_field_writes_a_single_field_and_skips_same_value_writes() {
        let store = Store::new(state(0, "a"));
        let count = Rc::new(CellCount::default());
        let sink = Rc::clone(&count);
        store.subscribe(move |_| sink.increment());

        store.set_field(|s| &mut s.value, 1);
        assert_eq!(*store.get_snapshot(), state(1, "a"));
        assert_eq!(count.get(), 1);

        store.set_field(|s| &mut s.value, 1);
        assert_eq!(count.get(), 1);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:86-97`: `update` merges
    // changed fields, notifies once, and skips no-op updates.
    #[test]
    fn update_merges_changed_fields_and_skips_no_op_updates() {
        let store = Store::new(state(0, "a"));
        let count = Rc::new(CellCount::default());
        let sink = Rc::clone(&count);
        store.subscribe(move |_| sink.increment());

        store.update(|next, current| {
            next.value = 2;
            next.label = "b".to_string();
            *next != *current
        });
        assert_eq!(*store.get_snapshot(), state(2, "b"));
        assert_eq!(count.get(), 1);

        store.update(|next, current| {
            next.value = 2;
            next.label = "b".to_string();
            *next != *current
        });
        assert_eq!(count.get(), 1);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:99-110`: `notifyAll` renews
    // the state reference (a fresh `Rc`) and notifies.
    #[test]
    fn notify_all_renews_the_state_reference_and_notifies() {
        let store = Store::new(state(0, "a"));
        let count = Rc::new(CellCount::default());
        let sink = Rc::clone(&count);
        store.subscribe(move |_| sink.increment());
        let previous = store.get_snapshot();

        store.notify_all();

        assert_eq!(count.get(), 1);
        assert!(!Rc::ptr_eq(&store.get_snapshot(), &previous));
        assert_eq!(*store.get_snapshot(), *previous);
    }

    // Mirrors `packages/utils/src/store/Store.test.ts:112-131`: a nested `set` from
    // a listener stops the outer notification pass — the nested call delivered the
    // final state to every listener and the outer pass aborts before delivering the
    // stale state to `second`.
    #[test]
    fn a_nested_set_state_from_a_listener_stops_the_outer_notification_pass() {
        let store = Rc::new(Store::new(state(0, "a")));

        let first_calls = Rc::new(CellCount::default());
        let first_sink = Rc::clone(&first_calls);
        let nested_store = Rc::clone(&store);
        let first_seen_final = Rc::new(RefCell::new(Vec::new()));
        let first_final_sink = Rc::clone(&first_seen_final);
        store.subscribe(move |state| {
            first_sink.increment();
            if state.value == 1 {
                nested_store.set_field(|s| &mut s.value, 2);
            }
            first_final_sink.borrow_mut().push(state.value);
        });

        let second_calls = Rc::new(CellCount::default());
        let second_sink = Rc::clone(&second_calls);
        let second_seen = Rc::new(RefCell::new(Vec::new()));
        let second_seen_sink = Rc::clone(&second_seen);
        store.subscribe(move |state| {
            second_sink.increment();
            second_seen_sink.borrow_mut().push((state.value, state.label.clone()));
        });

        store.set_field(|s| &mut s.value, 1);

        // The nested set() notified every listener with the final state; the outer
        // pass detected it and did not deliver the stale state to `second`.
        assert_eq!(store.get_snapshot().value, 2);
        assert_eq!(first_calls.get(), 2);
        assert_eq!(second_calls.get(), 1);
        assert_eq!(*second_seen.borrow(), vec![(2, "a".to_string())]);
    }

    // The unsubscribe handle removes only its own registration: two subscriptions
    // of behaviorally identical listeners unsubscribe independently
    // (`packages/utils/src/store/Store.test.ts:62-71`, extended to two listeners).
    #[test]
    fn an_unsubscribe_handle_removes_only_its_own_registration() {
        let store = Store::new(state(0, "a"));
        let first = Rc::new(CellCount::default());
        let second = Rc::new(CellCount::default());
        let first_sink = Rc::clone(&first);
        let second_sink = Rc::clone(&second);
        let unsubscribe = store.subscribe(move |_| first_sink.increment());
        store.subscribe(move |_| second_sink.increment());

        unsubscribe();
        store.set_field(|s| &mut s.value, 1);

        assert_eq!(first.get(), 0);
        assert_eq!(second.get(), 1);
    }

    // Mirrors `packages/utils/src/store/ReactStore.test.tsx:401-427` ("accepts
    // selector functions"): the listener fires immediately with the current
    // selector result in both slots, fires on changes with the previous result as
    // `oldValue`, and unsubscribing stops all further calls.
    #[test]
    fn observe_accepts_selector_functions_and_stops_after_unsubscribe() {
        let store = Store::new(CounterState { count: 0, multiplier: 1 });
        let calls: Rc<RefCell<Vec<(bool, bool)>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&calls);

        let unsubscribe = store.observe(
            |state: &CounterState| state.count > 1,
            move |new_value, old_value| sink.borrow_mut().push((*new_value, *old_value)),
        );

        assert_eq!(*calls.borrow(), vec![(false, false)]);

        store.set_field(|s| &mut s.count, 2);
        store.set_field(|s| &mut s.count, 1);
        assert_eq!(*calls.borrow(), vec![(false, false), (true, false), (false, true)]);

        unsubscribe();

        store.set_field(|s| &mut s.count, 3);
        assert_eq!(calls.borrow().len(), 3);
    }

    // Mirrors `packages/utils/src/store/ReactStore.test.tsx:445-463`: the listener
    // fires only when the selector result changes, delivering the old value; a
    // change that leaves the result unchanged fires nothing
    // (`packages/utils/src/store/ReactStore.test.tsx:465-480`).
    #[test]
    fn observe_fires_on_selector_changes_and_skips_unchanged_results() {
        let store = Store::new(CounterState { count: 5, multiplier: 1 });
        let calls: Rc<RefCell<Vec<(i64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&calls);

        store.observe(
            |state: &CounterState| state.count * 2,
            move |new_value, old_value| sink.borrow_mut().push((*new_value, *old_value)),
        );

        store.set_field(|s| &mut s.count, 10);
        store.set_field(|s| &mut s.count, 7);
        assert_eq!(
            *calls.borrow(),
            vec![(10, 10), (20, 10), (14, 20)],
        );

        // Changing `multiplier` leaves the `count * 2` selector result unchanged, so
        // nothing fires — the upstream case changes a non-dependency of the selector
        // (`store.set('multiplier', 5)`).
        store.set_field(|s| &mut s.multiplier, 9);
        assert_eq!(calls.borrow().len(), 3);
    }

    // Mirrors `packages/utils/src/store/ReactStore.test.tsx:482-501`: a change to
    // *any* dependency of the selector fires the listener — `multiplied` fires for
    // a `count` change and again for a `multiplier` change.
    #[test]
    fn observe_fires_when_any_dependency_of_the_selector_changes() {
        let store = Store::new(CounterState { count: 5, multiplier: 3 });
        let calls: Rc<RefCell<Vec<(i64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&calls);

        store.observe(
            |state: &CounterState| state.count * state.multiplier,
            move |new_value, old_value| sink.borrow_mut().push((*new_value, *old_value)),
        );

        store.set_field(|s| &mut s.count, 10);
        store.set_field(|s| &mut s.multiplier, 2);

        assert_eq!(*calls.borrow(), vec![(15, 15), (30, 15), (20, 30)]);
    }

    // Mirrors `packages/utils/src/store/ReactStore.test.tsx:539-559`: multiple
    // observers on the same selector each receive the sequence of values.
    #[test]
    fn observe_supports_multiple_observers_on_the_same_selector() {
        let store = Store::new(CounterState { count: 5, multiplier: 1 });
        let first: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
        let second: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
        let first_sink = Rc::clone(&first);
        let second_sink = Rc::clone(&second);

        store.observe(
            |state: &CounterState| state.count * 2,
            move |new_value, _| first_sink.borrow_mut().push(*new_value),
        );
        store.observe(
            |state: &CounterState| state.count * 2,
            move |new_value, _| second_sink.borrow_mut().push(*new_value),
        );

        store.set_field(|s| &mut s.count, 10);

        assert_eq!(*first.borrow(), vec![10, 20]);
        assert_eq!(*second.borrow(), vec![10, 20]);
    }

    // Mirrors `packages/utils/src/store/ReactStore.test.tsx:562-584`: observers on
    // different selectors fire independently of each other.
    #[test]
    fn observe_supports_observers_on_different_selectors() {
        let store = Store::new(CounterState { count: 5, multiplier: 3 });
        let doubled: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
        let multiplied: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
        let doubled_sink = Rc::clone(&doubled);
        let multiplied_sink = Rc::clone(&multiplied);

        store.observe(
            |state: &CounterState| state.count * 2,
            move |new_value, _| doubled_sink.borrow_mut().push(*new_value),
        );
        store.observe(
            |state: &CounterState| state.count * state.multiplier,
            move |new_value, _| multiplied_sink.borrow_mut().push(*new_value),
        );

        store.set_field(|s| &mut s.count, 10);
        store.set_field(|s| &mut s.multiplier, 2);

        assert_eq!(*doubled.borrow(), vec![10, 20]);
        assert_eq!(*multiplied.borrow(), vec![15, 30, 20]);
    }

    /// The upstream observe suite's counter state
    /// (`packages/utils/src/store/ReactStore.test.tsx:394`).
    #[derive(Clone, Debug, PartialEq)]
    struct CounterState {
        count: i64,
        multiplier: i64,
    }

    /// A plain notification counter standing in for upstream's `vi.fn()` call
    /// counts (the port's Vitest-only stand-in; see `fast_hooks.rs` tests).
    #[derive(Default)]
    struct CellCount(std::cell::Cell<usize>);

    impl CellCount {
        fn increment(&self) {
            self.0.set(self.0.get() + 1);
        }

        fn get(&self) -> usize {
            self.0.get()
        }
    }
}
