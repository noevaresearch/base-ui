//! Port of `packages/utils/src/store/ReactStore.ts` — the React-render-phase half of
//! the store unit (`ReactStore`, its context field, and its hook integrations), plus
//! the reactive bridge `useStore.ts` backs `Store.use`/`ReactStore.useState` with.
//! The plain observable container lives in [`crate::store`].
//!
//! Upstream behavior claims are cited to `ReactStore.test.tsx`, which
//! `specs/utils/store.md` names as the unit's source of truth.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream `ReactStore` extends `Store`; the port composes and derefs
//!   ([`ReactStore`] derefs to [`Store`]), so every `Store` method (including
//!   [`Store::observe`]) is available on the wrapper as inheritance would give.
//! - Upstream's third constructor argument registers named selectors addressable by
//!   string key from `select`/`useState`/`observe`. The registry exists to bundle
//!   selector functions for JS call sites; Rust call sites pass selector closures
//!   directly, so [`ReactStore::select`] and [`crate::store::Store::observe`] take
//!   closures and the registry is not ported.
//! - Upstream hooks run in React's render phase and sync through
//!   `useIsoLayoutEffect`. The port's hooks create a
//!   `reactive_graph::effect::Effect` scoped to the current reactive owner, which
//!   re-runs when the reactive values they read change — the reactive equivalent of
//!   upstream's dependency-array effects. The first run happens on the next executor
//!   tick, so the store state is updated asynchronously after the hook call (React
//!   likewise applies layout effects after the render pass). Components create these
//!   hooks inside their own reactive owner, whose disposal plays the role of
//!   unmounting.
//! - Upstream hooks address one state property by string key. Rust struct fields are
//!   not runtime-addressable, so the hooks take a field accessor closure returning a
//!   `&mut` to the written field, like [`Store::set_field`].
//! - `useControlledProp`'s dev-mode controlled/uncontrolled switch warning is keyed
//!   upstream by the property name and cached across renders
//!   (`packages/utils/src/store/ReactStore.ts:133-147`). The port emits the same
//!   message when the controlled value's defined-ness flips while the effect re-runs;
//!   the message wording is upstream's, which is crossed relative to the switch
//!   direction (`ReactStore.test.tsx:80-83`, `ReactStore.test.tsx:97-100` — the spec
//!   records the strings as asserted). It logs through `console.error` in debug
//!   builds; the message itself is [`controlled_state_switch_message`].
//! - `useSyncedValues(props)` applies a props object's entries with a dev check that
//!   the same keys are passed every render
//!   (`packages/utils/src/store/ReactStore.ts:90-114`). Rust call sites have no
//!   dynamic key objects; N [`ReactStore::use_synced_value`] calls (one per field)
//!   give the same per-entry dependency behavior, and the key-stability check is N/A.
//! - `useStateSetter`'s identity stability across forced re-renders
//!   (`ReactStore.test.tsx:258-271`) is a React render-phase artifact: Leptos
//!   components run once, so the returned closure is created exactly once per call
//!   and holds no per-render state.
//! - The observation listener's third argument (the store instance,
//!   `ReactStore.test.tsx:511-515`) has no equivalent on a borrowed `self`; callers
//!   capture the store handle they already hold (same object).
//! - `useContextCallback` (stable callbacks assigned into the store context) is not
//!   ported: it depends on `useStableCallback`, which is its own unported unit, and
//!   no test in the unit's suite exercises it.

use std::ops::Deref;
use std::rc::Rc;

use reactive_graph::effect::Effect;
use reactive_graph::graph::untrack;
use reactive_graph::owner::LocalStorage;
use reactive_graph::owner::on_cleanup;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::Get;
use reactive_graph::traits::Set;
use send_wrapper::SendWrapper;

use crate::store::Store;
use crate::store::StoreUnsubscribe;

/// A [`Store`] with a non-reactive `context`, controlled-key support and the reactive
/// hook integrations — the port of upstream `ReactStore<State, Context>`
/// (`packages/utils/src/store/ReactStore.ts:14-18`). Create it inside a reactive
/// owner and hold it as an `Rc` so the hooks can outlive the call that created them.
pub struct ReactStore<State, Context = ()> {
    store: Store<State>,
    /// Non-reactive values such as refs, callbacks, etc.
    /// (`packages/utils/src/store/ReactStore.ts:35`).
    pub context: Context,
}

impl<State: 'static> ReactStore<State, ()> {
    /// Direct construction with an empty context — upstream `new ReactStore(state)`
    /// (`ReactStore.test.tsx:287-291` passes `undefined` as the context).
    pub fn new(state: State) -> Self {
        Self {
            store: Store::new(state),
            context: (),
        }
    }

    /// Upstream `ReactStore.create(initialState)`
    /// (`ReactStore.test.tsx:21-24`): state seeded, `context` empty.
    pub fn create(state: State) -> Self {
        Self::new(state)
    }
}

impl<State: 'static, Context: 'static> ReactStore<State, Context> {
    /// Upstream `new ReactStore(state, context)` — non-reactive context values
    /// alongside the state.
    pub fn with_context(state: State, context: Context) -> Self {
        Self {
            store: Store::new(state),
            context,
        }
    }
}

impl<State, Context> Deref for ReactStore<State, Context> {
    type Target = Store<State>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl<State: Clone + 'static, Context: 'static> ReactStore<State, Context> {
    /// Synchronizes a single external value into the store whenever the reactive
    /// `value` changes — upstream `useSyncedValue(key, value)`
    /// (`packages/utils/src/store/ReactStore.ts:45-54`): the write is skipped when
    /// the field already holds the value ([`Store::set_field`]'s no-op rule,
    /// `ReactStore.test.tsx:112-117`), and re-applied when the store instance is
    /// swapped, because the new instance gets its own effect
    /// (`ReactStore.test.tsx:128-133` — swapping is expressed by calling the hook
    /// against the other store handle).
    pub fn use_synced_value<V, M>(self: &Rc<Self>, field: impl Fn(&mut State) -> &mut V + 'static, value: M)
    where
        V: PartialEq + 'static,
        M: Get<Value = V> + 'static,
    {
        let store = Rc::clone(self);
        Effect::new(move || {
            store.set_field(|state| (field)(state), value.get());
        });
    }

    /// Synchronizes a single optional external value into the store and resets the
    /// field to `None` when the owning reactive scope is disposed — upstream
    /// `useSyncedValueWithCleanup(key, value)`
    /// (`packages/utils/src/store/ReactStore.ts:63-78`): synced on mount and on
    /// change, `undefined` on unmount (`ReactStore.test.tsx:221-232`).
    pub fn use_synced_value_with_cleanup<V, M>(
        self: &Rc<Self>,
        field: impl Fn(&mut State) -> &mut Option<V> + 'static,
        value: M,
    ) where
        V: PartialEq + 'static,
        M: Get<Value = Option<V>> + 'static,
    {
        let field = Rc::new(field);
        let store = Rc::clone(self);
        Effect::new({
            let field = Rc::clone(&field);
            move || {
                store.set_field(|state| (field)(state), value.get());
            }
        });

        // Registered on the *calling* owner (not inside the effect), so the reset runs
        // only on disposal — not before every effect re-run.
        let store = Rc::clone(self);
        let cleanup = SendWrapper::new(move || {
            store.set_field(|state| (field)(state), None);
        });
        on_cleanup(move || (*cleanup)());
    }

    /// Registers a controllable prop for a specific field: while the reactive
    /// `controlled` source yields `Some`, the field is kept in sync with it — upstream
    /// `useControlledProp(key, controlled)`
    /// (`packages/utils/src/store/ReactStore.ts:120-148`). Updates to *other* keys
    /// still apply while this one is controlled (`ReactStore.test.tsx:39-43`), and a
    /// later change of the controlled value syncs into state
    /// (`ReactStore.test.tsx:46-49`). Switching between controlled and uncontrolled
    /// emits the dev warning (`ReactStore.test.tsx:69-101`).
    pub fn use_controlled_prop<V, M>(
        self: &Rc<Self>,
        key: &'static str,
        field: impl Fn(&mut State) -> &mut V + 'static,
        controlled: M,
    ) where
        V: PartialEq + 'static,
        M: Get<Value = Option<V>> + 'static,
    {
        let store = Rc::clone(self);
        Effect::new(move |previous: Option<bool>| -> bool {
            let controlled = controlled.get();
            let is_controlled = controlled.is_some();
            if let Some(previous) = previous {
                if previous != is_controlled {
                    warn_controlled_state_switch(key, is_controlled);
                }
            }
            if let Some(value) = controlled {
                store.set_field(|state| (field)(state), value);
            }
            is_controlled
        });
    }

    /// Returns a stable setter closure writing one field — upstream
    /// `useStateSetter(key)` (`packages/utils/src/store/ReactStore.ts:203-211`),
    /// commonly used as a ref callback (`ReactStore.test.tsx:258-271`).
    pub fn use_state_setter<V>(
        self: &Rc<Self>,
        field: impl Fn(&mut State) -> &mut V + 'static,
    ) -> impl Fn(V) + 'static
    where
        V: PartialEq + 'static,
    {
        let store = Rc::clone(self);
        move |value| store.set_field(|state| (field)(state), value)
    }

    /// Subscribes to a derived slice of the state and returns a reactive signal of
    /// the selected value — the port of `Store.use(selector)`/`ReactStore.useState`
    /// (`packages/utils/src/store/Store.ts:120-125`,
    /// `packages/utils/src/store/ReactStore.ts:171-179` via
    /// `packages/utils/src/store/useStore.ts`). The returned signal updates when the
    /// store notifies with a changed selection, and also when the selector reads any
    /// reactive value of its own (the selector-argument reactivity of
    /// `ReactStore.test.tsx:384-390`). Reads before the first executor tick see the
    /// selection at hook-call time, mirroring upstream's "the value returned by
    /// `useState` is updated after the component renders" note
    /// (`packages/utils/src/store/Store.ts:22`). Unsubscribes when the owning
    /// reactive scope is disposed.
    pub fn use_state<V, S>(self: &Rc<Self>, selector: S) -> RwSignal<V, LocalStorage>
    where
        V: PartialEq + Clone + 'static,
        S: Fn(&State) -> V + 'static,
    {
        let store = Rc::clone(self);
        // The initial selection is computed once, outside any tracking context —
        // upstream applies it during the first render
        // (`packages/utils/src/store/useStore.ts:153`).
        let value = RwSignal::new_local(untrack(|| selector(&store.get_snapshot())));

        // Every store notification nudges the effect, which then re-applies the
        // selector to the latest state snapshot; the selector's own reactive reads
        // (a selector argument, for instance) also re-run the effect.
        let bump = RwSignal::new_local(());
        let unsubscribe: StoreUnsubscribe = {
            let bump = bump.clone();
            self.subscribe(move |_| bump.set(()))
        };
        let unsubscribe = SendWrapper::new(unsubscribe);
        on_cleanup(move || (*unsubscribe)());

        Effect::new(move |previous: Option<V>| -> V {
            bump.get();
            let next = selector(&store.get_snapshot());
            if previous.as_ref().is_none_or(|previous| *previous != next) {
                value.set(next.clone());
            }
            next
        });

        value
    }
}

impl<State: 'static, Context> ReactStore<State, Context> {
    /// Reads the current state through a selector without subscribing — upstream
    /// `select(key)` (`packages/utils/src/store/ReactStore.ts:154-162`).
    pub fn select<V, S>(&self, selector: S) -> V
    where
        S: FnOnce(&State) -> V,
    {
        selector(&self.get_snapshot())
    }
}

/// The dev-mode controlled/uncontrolled switch warning, with upstream's exact (crossed)
/// wording — `packages/utils/src/store/ReactStore.ts:141-145`. Exposed for tests;
/// `console.error` dispatch is wasm-only, so host tests assert this string directly.
#[doc(hidden)]
pub fn controlled_state_switch_message(key: &str, now_controlled: bool) -> String {
    format!(
        "A component is changing the {}controlled state of {} to be {}controlled. Elements should not switch from uncontrolled to controlled (or vice versa).",
        if now_controlled { "" } else { "un" },
        key,
        if now_controlled { "un" } else { "" },
    )
}

fn warn_controlled_state_switch(key: &str, now_controlled: bool) {
    if cfg!(debug_assertions) {
        let message = controlled_state_switch_message(key, now_controlled);
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&message));
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;

    use super::*;

    /// The upstream suite's shared state shape (`ReactStore.test.tsx:9`).
    #[derive(Clone, Debug, PartialEq)]
    struct ValueLabelState {
        value: i64,
        label: String,
    }

    fn state(value: i64, label: &str) -> ValueLabelState {
        ValueLabelState {
            value,
            label: label.to_string(),
        }
    }

    /// A notification counter standing in for upstream's call-count assertions.
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

    // Mirrors `ReactStore.test.tsx:18-25`: create() constructs a fully wired store —
    // state seeded, context empty. The `toBeInstanceOf` half is structural in Rust
    // (see the crate::store module docs on `create`).
    #[test]
    fn create_constructs_a_fully_wired_react_store_instance() {
        let store = ReactStore::create(state(1, "a"));

        assert_eq!(*store.get_snapshot(), state(1, "a"));
        assert_eq!(store.context, ());
    }

    // Mirrors `ReactStore.test.tsx:27-50`: with a controlled value, the initial
    // render syncs it into state; updates to other keys still apply; a later change
    // of the controlled value syncs again.
    #[test]
    fn use_controlled_prop_syncs_internal_state_from_the_controlled_value() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let store = Rc::new(ReactStore::create(state(0, "")));
        let controlled: RwSignal<Option<i64>> = RwSignal::new(Some(1));
        store.use_controlled_prop("value", |s: &mut ValueLabelState| &mut s.value, controlled);

        Executor::poll_local();
        assert_eq!(store.get_snapshot().value, 1);

        store.update(|next, _| {
            next.label = "y".to_string();
            true
        });
        assert_eq!(store.get_snapshot().label, "y");

        controlled.set(Some(7));
        Executor::poll_local();
        assert_eq!(store.get_snapshot().value, 7);

        owner.cleanup();
    }

    // Mirrors `ReactStore.test.tsx:52-67`: syncing through a second store instance
    // wires the same controlled value into that instance.
    #[test]
    fn use_controlled_prop_syncs_internal_state_when_the_store_changes() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let first_store = Rc::new(ReactStore::create(state(0, "")));
        let second_store = Rc::new(ReactStore::create(state(0, "")));
        let controlled: RwSignal<Option<i64>> = RwSignal::new(Some(1));

        first_store.use_controlled_prop("value", |s: &mut ValueLabelState| &mut s.value, controlled.clone());
        Executor::poll_local();
        assert_eq!(first_store.get_snapshot().value, 1);
        assert_eq!(second_store.get_snapshot().value, 0);

        second_store.use_controlled_prop("value", |s: &mut ValueLabelState| &mut s.value, controlled);
        Executor::poll_local();
        assert_eq!(second_store.get_snapshot().value, 1);

        owner.cleanup();
    }

    // Mirrors `ReactStore.test.tsx:69-101`: switching between controlled and
    // uncontrolled produces the dev error. The wording is crossed relative to the
    // switch direction upstream; the port records the strings as asserted.
    #[test]
    fn warns_on_switching_between_controlled_and_uncontrolled() {
        assert_eq!(
            controlled_state_switch_message("value", false),
            "A component is changing the uncontrolled state of value to be controlled. Elements should not switch from uncontrolled to controlled (or vice versa).",
        );
        assert_eq!(
            controlled_state_switch_message("value", true),
            "A component is changing the controlled state of value to be uncontrolled. Elements should not switch from uncontrolled to controlled (or vice versa).",
        );
    }

    // Mirrors `ReactStore.test.tsx:103-117`: useSyncedValue updates a single key when
    // the passed value changes.
    #[test]
    fn use_synced_value_updates_a_single_key_when_the_passed_value_changes() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let store = Rc::new(ReactStore::create(state(0, "")));
        let value: RwSignal<i64> = RwSignal::new(1);
        store.use_synced_value(|s: &mut ValueLabelState| &mut s.value, value);

        Executor::poll_local();
        assert_eq!(store.get_snapshot().value, 1);

        value.set(2);
        Executor::poll_local();
        assert_eq!(store.get_snapshot().value, 2);

        owner.cleanup();
    }

    // Mirrors `ReactStore.test.tsx:208-233`: useSyncedValueWithCleanup syncs on mount
    // and on change, and resets the key to None when the component unmounts.
    #[test]
    fn use_synced_value_with_cleanup_synchronizes_value_and_resets_on_cleanup() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let first: Rc<str> = Rc::from("first");
        let second: Rc<str> = Rc::from("second");
        let store = Rc::new(ReactStore::create(HolderState { node: None }));
        // `Rc<str>` is not `Send`, so the signal must be the local-storage variant.
        let value = RwSignal::new_local(Some(first.clone()));
        store.use_synced_value_with_cleanup(|s: &mut HolderState| &mut s.node, value);

        Executor::poll_local();
        assert!(Rc::ptr_eq(store.get_snapshot().node.as_ref().unwrap(), &first));

        value.set(Some(second.clone()));
        Executor::poll_local();
        assert!(Rc::ptr_eq(store.get_snapshot().node.as_ref().unwrap(), &second));

        // Unmount: the owner disposal resets the key to None.
        owner.cleanup();
        assert!(store.get_snapshot().node.is_none());
    }

    // Mirrors `ReactStore.test.tsx:235-272`: the setter writes the value into the
    // store state. Upstream's identity-stability assertion is N/A (see the module
    // docs on `use_state_setter`).
    #[test]
    fn use_state_setter_updates_the_store_state() {
        let element: Rc<str> = Rc::from("element");
        let store = Rc::new(ReactStore::create(ElementState { element: None }));
        let setter = store.use_state_setter(|s: &mut ElementState| &mut s.element);

        setter(Some(element.clone()));
        assert!(Rc::ptr_eq(store.get_snapshot().element.as_ref().unwrap(), &element));

        setter(None);
        assert_eq!(store.get_snapshot().element, None);
    }

    // Mirrors `ReactStore.test.tsx:326-345` and `ReactStore.test.tsx:376-390`: the
    // reactive bridge re-renders (updates the signal) when the selected slice
    // changes, including through a selector argument, and unsubscribes on cleanup.
    #[test]
    fn use_state_bridge_tracks_the_selected_slice_and_selector_arguments() {
        let _ = Executor::init_futures_executor();
        let owner = Owner::new();
        owner.set();

        let store = Rc::new(ReactStore::create(ValuesState {
            values: [("first", "one"), ("second", "two")]
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }));

        // The selector reads a reactive argument (ReactStore.test.tsx:375-378).
        let value_key: RwSignal<&'static str> = RwSignal::new("first");
        let output = store.use_state(move |state: &ValuesState| {
            state.values.get(value_key.get()).cloned().unwrap_or_default()
        });
        assert_eq!(output.get_untracked(), "one");

        // A store change that alters the selection updates the bridge.
        store.update(|next, _| {
            next.values.insert("first".to_string(), "uno".to_string());
            true
        });
        Executor::poll_local();
        assert_eq!(output.get_untracked(), "uno");

        // A selector-argument change alone re-applies the selector.
        value_key.set("second");
        Executor::poll_local();
        assert_eq!(output.get_untracked(), "two");

        // A store change that leaves the selection unchanged does not re-run readers
        // of the bridge (upstream dedups on the selector result).
        let runs = Rc::new(CellCount::default());
        {
            let runs = Rc::clone(&runs);
            Effect::new(move |_: Option<()>| {
                let _ = output.get();
                runs.increment();
            });
        }
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        store.update(|next, _| {
            next.values.insert("other".to_string(), "x".to_string());
            true
        });
        Executor::poll_local();
        assert_eq!(runs.get(), 1);

        // Cleanup unsubscribes the bridge from the store and disposes its signal;
        // later store changes are simply not observed (the unsubscribe contract is
        // pinned at the Store level in crate::store's tests).
        owner.cleanup();
        store.update(|next, _| {
            next.values.insert("first".to_string(), "changed".to_string());
            true
        });
        Executor::poll_local();
    }

    // Mirrors the store-level half of `ReactStore.test.tsx:274-362` (nested stores):
    // observing the parent key subscribes/unsubscribes to the parent on attach and
    // removal, local writes propagate into the parent, and parent updates flow back
    // into the derived view while raw child state stays put. The rendered-output
    // assertions of the upstream test are covered by the bridge test above.
    #[test]
    fn supports_nested_stores_as_state_values() {
        let parent_store = Rc::new(ReactStore::create(CountState { count: 0 }));
        let child_store = Rc::new(ReactStore::create(ChildState {
            count: 10,
            parent: None,
        }));

        // Observing the `parent` key: subscribe to the parent (with a
        // notifyAll-forwarding callback) when a parent arrives, unsubscribe when it
        // becomes undefined (ReactStore.test.tsx:299-313).
        let parent_subscription: Rc<RefCell<Option<StoreUnsubscribe>>> = Rc::new(RefCell::new(None));
        {
            let child = Rc::clone(&child_store);
            let subscription = Rc::clone(&parent_subscription);
            // The selector's result proxies the store reference by pointer identity:
            // `observe` compares selected values with `PartialEq`, which the store
            // handle itself does not implement.
            child_store.observe(
                |state: &ChildState| state.parent.as_ref().map(|parent| Rc::as_ptr(parent) as usize),
                move |new_parent, _old_parent| {
                    let mut subscription = subscription.borrow_mut();
                    if let Some(existing) = subscription.take() {
                        existing();
                    }
                    if new_parent.is_some() {
                        let parent = child.get_snapshot().parent.clone().unwrap();
                        let child = Rc::clone(&child);
                        *subscription = Some(parent.subscribe(move |_| child.notify_all()));
                    }
                },
            );
        }

        // An observer on the local count selector propagates local writes into the
        // parent — only while a parent is attached, matching upstream's
        // `store.state.parent?.set('count', newCount)` (ReactStore.test.tsx:315-324).
        {
            let child = Rc::clone(&child_store);
            child_store.observe(
                |state: &ChildState| state.count,
                move |new_count, _old_count| {
                    if let Some(parent) = child.get_snapshot().parent.as_ref() {
                        parent.set_field(|state: &mut CountState| &mut state.count, *new_count);
                    }
                },
            );
        }

        // Derived view: through the parent when attached, local state otherwise
        // (the upstream selectors `count` and `parent` play this role).
        let selected_count = |state: &ChildState| {
            state
                .parent
                .as_ref()
                .map_or(state.count, |parent| parent.get_snapshot().count)
        };

        child_store.set_field(|s: &mut ChildState| &mut s.count, 5);
        assert_eq!(child_store.get_snapshot().count, 5);
        assert_eq!(child_store.select(selected_count), 5);

        child_store.update(|next, _| {
            next.parent = Some(Rc::clone(&parent_store));
            true
        });
        assert_eq!(child_store.get_snapshot().count, 5);
        assert_eq!(child_store.select(selected_count), 0);

        child_store.set_field(|s: &mut ChildState| &mut s.count, 20);
        assert_eq!(child_store.get_snapshot().count, 20);
        assert_eq!(parent_store.get_snapshot().count, 20);
        assert_eq!(child_store.select(selected_count), 20);

        parent_store.set_field(|s: &mut CountState| &mut s.count, 15);
        assert_eq!(parent_store.get_snapshot().count, 15);
        assert_eq!(child_store.get_snapshot().count, 20);
        assert_eq!(child_store.select(selected_count), 15);
    }

    #[derive(Clone, Debug, PartialEq)]
    struct HolderState {
        node: Option<Rc<str>>,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct ElementState {
        element: Option<Rc<str>>,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct ValuesState {
        values: std::collections::HashMap<String, String>,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct CountState {
        count: i64,
    }

    #[derive(Clone)]
    struct ChildState {
        count: i64,
        parent: Option<Rc<ReactStore<CountState>>>,
    }
}
