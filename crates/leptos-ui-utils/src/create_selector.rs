//! Port of `packages/utils/src/store/createSelector.ts` and
//! `packages/utils/src/store/createSelectorMemoized.ts` — the selector-combinator
//! half of the store unit. `specs/utils/store.md` names the modules' own test suites
//! as the source of truth.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - Upstream `createSelector` is variadic, dispatches through unrolled fixed arities
//!   for one to seven input selectors plus a trailing combiner
//!   (`packages/utils/src/store/createSelector.ts:148-228`), and throws
//!   `'Unsupported number of selectors'` beyond seven
//!   (`packages/utils/src/store/createSelector.test.ts:68-75`). Rust has no
//!   variadics: [`create_selector_1`] through [`create_selector_7`] provide the same
//!   composition per arity, the single-function identity form is the closure itself
//!   (`createSelector.test.ts:5-11`), and the arity overflow is a compile error
//!   instead of a runtime throw.
//! - Upstream forwards up to three extra arguments positionally to every input
//!   selector and the combiner (`createSelector.test.ts:77-88`). Rust call sites bind
//!   such values by capturing them in the closures instead; the extra-argument slots
//!   are not ported for the non-memoized combinators.
//! - Upstream `createSelectorMemoized` memoizes on reselect machinery: a per-state
//!   object `WeakMap` of memo instances whose `lruMemoize` (max size 1, `Object.is`
//!   equality) caches the combiner result keyed on the input selector results, with
//!   the state object carrying a `__cacheKey__`
//!   (`packages/utils/src/store/createSelectorMemoized.ts:61-151`). The port models
//!   this with [`MemoizedSelector`]: a cache keyed on the state handle's identity
//!   (the port's equivalent of the `__cacheKey__` identity, see
//!   [`crate::store`] on identity) holding the extra arguments and the result, with
//!   entries removed when their state handle is dropped (the `WeakMap` behavior).
//!   For pure input selectors this is equivalent to comparing the input results,
//!   because they are functions of the state alone.
//! - `createSelectorMemoized` throws `'Unsupported number of arguments'` when the
//!   combiner's `Function.length` reports more than three extra arguments or
//!   under-reports them via rest parameters
//!   (`packages/utils/src/store/createSelectorMemoized.test.ts:132-151`). Rust's
//!   static arity makes both N/A: the extra arguments are a tuple capped at three
//!   slots by the [`SelectorArgs`] implementors.
//! - `createSelectorMemoizedWithOptions` forwards reselect creator options
//!   (`argsMemoize`, `devModeChecks`, custom `memoize`rs, `resultEqualityCheck`) —
//!   reselect-interop surface with no Rust counterpart, not ported. The underlying
//!   memoization semantics of the default configuration (`maxSize: 1`,
//!   `Object.is` equality, per-state caches) are what [`MemoizedSelector`] ports.
//!   Note one consequence: upstream `Object.is` treats `NaN` as equal to itself, so
//!   a memoized comparison involving `NaN` can hit the cache where Rust's `PartialEq`
//!   (`NaN != NaN`) re-runs the combiner — re-running is the safe direction.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

/// The extra-argument tuples the memoized selector's fixed dispatch slots support —
/// upstream caps the combiner at three arguments beyond the input selector results
/// (`packages/utils/src/store/createSelectorMemoized.ts:73-77`).
pub trait SelectorArgs: PartialEq + Clone {}
impl SelectorArgs for () {}
impl<A: PartialEq + Clone> SelectorArgs for (A,) {}
impl<A: PartialEq + Clone, B: PartialEq + Clone> SelectorArgs for (A, B) {}
impl<A: PartialEq + Clone, B: PartialEq + Clone, C: PartialEq + Clone> SelectorArgs for (A, B, C) {}

macro_rules! create_selector_arity {
    ($name:ident, $($selector:ident : $Type:ident),+) => {
        /// Creates a selector combining the given input selectors with a trailing
        /// combiner — the fixed-arity form of upstream `createSelector`
        /// (`packages/utils/src/store/createSelector.ts:148-228`). The single-function
        /// form is the identity: pass the closure itself.
        pub fn $name<State, $($Type,)* V>(
            $($selector: impl Fn(&State) -> $Type + 'static,)*
            combiner: impl Fn($($Type,)+) -> V + 'static,
        ) -> impl Fn(&State) -> V + 'static
        where
            State: 'static,
        {
            move |state| combiner($($selector(state),)+)
        }
    };
}

create_selector_arity!(create_selector_1, input_a: A);
create_selector_arity!(create_selector_2, input_a: A, input_b: B);
create_selector_arity!(create_selector_3, input_a: A, input_b: B, input_c: C);
create_selector_arity!(create_selector_4, input_a: A, input_b: B, input_c: C, input_d: D);
create_selector_arity!(
    create_selector_5,
    input_a: A,
    input_b: B,
    input_c: C,
    input_d: D,
    input_e: E
);
create_selector_arity!(
    create_selector_6,
    input_a: A,
    input_b: B,
    input_c: C,
    input_d: D,
    input_e: E,
    input_f: F
);
create_selector_arity!(
    create_selector_7,
    input_a: A,
    input_b: B,
    input_c: C,
    input_d: D,
    input_e: E,
    input_f: F,
    input_g: G
);

/// A memoized derived selector — the port of what upstream's reselect configuration
/// computes for `createSelectorMemoized` (`packages/utils/src/store/
/// createSelectorMemoized.ts:58-155`): the combiner runs at most once per distinct
/// state identity and argument set, and the cache entry for a state dies with the
/// state (upstream's `WeakMap`).
pub struct MemoizedSelector<State, Args, V> {
    compute: Box<dyn Fn(&State, Args) -> V>,
    // Keyed on the state handle's address; the `Weak` lets the entry die with the
    // state, like upstream's `WeakMap<__cacheKey__, ...>`.
    cache: RefCell<HashMap<usize, (Weak<State>, Args, Rc<V>)>>,
}

impl<State: 'static, Args: SelectorArgs + 'static, V: 'static> MemoizedSelector<State, Args, V> {
    /// Calls the selector with extra arguments, running the combiner only when the
    /// state identity or the arguments differ from the cached call
    /// (`packages/utils/src/store/createSelectorMemoized.test.ts:29-44`,
    /// `:83-130`). Two distinct states with equal content each run the combiner
    /// (per-state-identity caching, `:68-81`).
    pub fn call_with(&self, state: &Rc<State>, args: Args) -> V
    where
        V: Clone,
    {
        self.prune();
        let state_key = Rc::as_ptr(state) as usize;
        if let Some((_, cached_args, cached_value)) = self.cache.borrow().get(&state_key) {
            if *cached_args == args {
                return (**cached_value).clone();
            }
        }

        let value = Rc::new((self.compute)(state, args.clone()));
        let result = (*value).clone();
        self.cache.borrow_mut().insert(
            state_key,
            (Rc::downgrade(state), args, value),
        );
        result
    }

    /// Drops cache entries whose state handle is gone — the garbage-collection
    /// upstream gets for free from `WeakMap`.
    fn prune(&self) {
        self.cache
            .borrow_mut()
            .retain(|_, (state, _, _)| state.strong_count() > 0);
    }
}

impl<State: 'static, V: 'static> MemoizedSelector<State, (), V> {
    /// Calls a selector without extra arguments
    /// (`packages/utils/src/store/createSelectorMemoized.test.ts:46-66`).
    pub fn call(&self, state: &Rc<State>) -> V
    where
        V: Clone,
    {
        self.call_with(state, ())
    }
}

/// Single-function form of `createSelectorMemoized`: the only argument is the
/// combiner, receiving the state — memoized on the state identity
/// (`packages/utils/src/store/createSelectorMemoized.test.ts:9-27`).
pub fn create_selector_memoized<State, V>(combiner: impl Fn(&State) -> V + 'static) -> MemoizedSelector<State, (), V>
where
    State: 'static,
    V: 'static,
{
    MemoizedSelector {
        compute: Box::new(move |state, (): ()| combiner(state)),
        cache: RefCell::new(HashMap::new()),
    }
}

/// Single-function form with extra arguments: the combiner receives the state
/// followed by the arguments, memoized on all of them
/// (`packages/utils/src/store/createSelectorMemoized.test.ts:29-44`,
/// `:83-130`).
pub fn create_selector_memoized_with_args<State, Args, V>(
    combiner: impl Fn(&State, Args) -> V + 'static,
) -> MemoizedSelector<State, Args, V>
where
    State: 'static,
    Args: SelectorArgs + 'static,
    V: 'static,
{
    MemoizedSelector {
        compute: Box::new(combiner),
        cache: RefCell::new(HashMap::new()),
    }
}

/// Input-selector form with one input
/// (`packages/utils/src/store/createSelectorMemoized.test.ts:68-81`).
pub fn create_selector_memoized_1<State, A, V>(
    input_a: impl Fn(&State) -> A + 'static,
    combiner: impl Fn(A) -> V + 'static,
) -> MemoizedSelector<State, (), V>
where
    State: 'static,
    A: 'static,
    V: 'static,
{
    MemoizedSelector {
        compute: Box::new(move |state, (): ()| combiner(input_a(state))),
        cache: RefCell::new(HashMap::new()),
    }
}

/// Input-selector form with one input and up to three extra arguments
/// (`packages/utils/src/store/createSelectorMemoized.test.ts:83-130`): the combiner
/// receives the input result followed by the argument tuple.
pub fn create_selector_memoized_1_with_args<State, A, Args, V>(
    input_a: impl Fn(&State) -> A + 'static,
    combiner: impl Fn(A, Args) -> V + 'static,
) -> MemoizedSelector<State, Args, V>
where
    State: 'static,
    A: 'static,
    Args: SelectorArgs + 'static,
    V: 'static,
{
    MemoizedSelector {
        compute: Box::new(move |state, args| combiner(input_a(state), args)),
        cache: RefCell::new(HashMap::new()),
    }
}

/// Input-selector form with two inputs
/// (`packages/utils/src/store/createSelectorMemoized.test.ts:46-66`).
pub fn create_selector_memoized_2<State, A, B, V>(
    input_a: impl Fn(&State) -> A + 'static,
    input_b: impl Fn(&State) -> B + 'static,
    combiner: impl Fn(A, B) -> V + 'static,
) -> MemoizedSelector<State, (), V>
where
    State: 'static,
    A: 'static,
    B: 'static,
    V: 'static,
{
    MemoizedSelector {
        compute: Box::new(move |state, (): ()| combiner(input_a(state), input_b(state))),
        cache: RefCell::new(HashMap::new()),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct SumState {
        a: i64,
        b: i64,
    }

    /// A call counter standing in for upstream's `vi.fn()` combiners.
    struct Counted<V> {
        calls: Cell<usize>,
        compute: Box<dyn Fn() -> V>,
    }

    impl<V> Counted<V> {
        fn new(compute: impl Fn() -> V + 'static) -> Self {
            Self {
                calls: Cell::new(0),
                compute: Box::new(compute),
            }
        }

        fn call(&self) -> V {
            self.calls.set(self.calls.get() + 1);
            (self.compute)()
        }

        fn count(&self) -> usize {
            self.calls.get()
        }
    }

    // Mirrors `createSelector.test.ts:13-40`: input selectors plus a combiner
    // produce the combined value.
    #[test]
    fn create_selector_combines_input_selectors() {
        let selector = create_selector_2(
            |state: &SumState| state.a,
            |state: &SumState| state.b,
            |a, b| a + b,
        );

        let state = SumState { a: 1, b: 2 };
        assert_eq!(selector(&state), 3);
    }

    // Mirrors `createSelector.test.ts:25-66`: six and seven input selectors work.
    #[test]
    fn create_selector_supports_six_and_seven_inputs() {
        let selector_6 = create_selector_6(
            |s: &SumState| s.a,
            |s: &SumState| s.b,
            |s: &SumState| s.a * 2,
            |s: &SumState| s.b * 2,
            |s: &SumState| s.a * 4,
            |s: &SumState| s.b * 4,
            |a, b, c, d, e, f| a + b + c + d + e + f,
        );
        let state = SumState { a: 1, b: 2 };
        assert_eq!(selector_6(&state), 1 + 2 + 2 + 4 + 4 + 8);

        let selector_7 = create_selector_7(
            |s: &SumState| s.a,
            |s: &SumState| s.b,
            |s: &SumState| s.a * 2,
            |s: &SumState| s.b * 2,
            |s: &SumState| s.a * 4,
            |s: &SumState| s.b * 4,
            |s: &SumState| s.a * 8,
            |a, b, c, d, e, f, g| a + b + c + d + e + f + g,
        );
        assert_eq!(selector_7(&state), 1 + 2 + 2 + 4 + 4 + 8 + 8);
    }

    // Mirrors `createSelectorMemoized.test.ts:9-27`: the single-function form wires
    // the state into the combiner and memoizes on the state identity.
    #[test]
    fn memoized_single_function_form_memoizes_per_state_identity() {
        let combiner = Rc::new(Counted::new(|| 42));

        let selector = create_selector_memoized({
            let combiner = Rc::clone(&combiner);
            move |_: &SumState| combiner.call()
        });

        let state = Rc::new(SumState { a: 4, b: 2 });
        assert_eq!(selector.call(&state), 42);
        assert_eq!(selector.call(&state), 42);
        assert_eq!(combiner.count(), 1);
    }

    // Mirrors `createSelectorMemoized.test.ts:29-44`: extra arguments reach the
    // combiner alongside the state and participate in memoization.
    #[test]
    fn memoized_single_function_form_memoizes_on_extra_arguments() {
        let combiner = Rc::new(Counted::new(|| ()));

        let state = Rc::new(SumState { a: 10, b: 0 });
        let selector = create_selector_memoized_with_args({
            let combiner = Rc::clone(&combiner);
            move |state: &SumState, x1: (i64,)| {
                let _ = combiner.call();
                state.a + x1.0
            }
        });

        assert_eq!(selector.call_with(&state, (5,)), 15);
        assert_eq!(selector.call_with(&state, (5,)), 15);
        assert_eq!(combiner.count(), 1);

        assert_eq!(selector.call_with(&state, (6,)), 16);
        assert_eq!(combiner.count(), 2);
    }

    // Mirrors `createSelectorMemoized.test.ts:46-66`: the input-selector form re-runs
    // the combiner when the inputs change and memoizes otherwise.
    #[test]
    fn memoized_input_selector_form_reruns_when_inputs_change() {
        let combiner = Rc::new(Counted::new(|| ()));

        let selector = create_selector_memoized_2(
            |state: &SumState| state.a,
            |state: &SumState| state.b,
            {
                let combiner = Rc::clone(&combiner);
                move |a, b| {
                    let _ = combiner.call();
                    a + b
                }
            },
        );

        let state = Rc::new(SumState { a: 1, b: 2 });
        assert_eq!(selector.call(&state), 3);
        assert_eq!(selector.call(&state), 3);
        assert_eq!(combiner.count(), 1);

        let next = Rc::new(SumState { a: 5, b: 2 });
        assert_eq!(selector.call(&next), 7);
        assert_eq!(combiner.count(), 2);
    }

    // Mirrors `createSelectorMemoized.test.ts:68-81`: two distinct states with equal
    // content each run the combiner.
    #[test]
    fn memoized_selector_caches_separately_per_state_identity() {
        let combiner = Rc::new(Counted::new(|| ()));

        let selector = create_selector_memoized_1(
            |state: &SumState| state.a,
            {
                let combiner = Rc::clone(&combiner);
                move |value| {
                    let _ = combiner.call();
                    value
                }
            },
        );

        let a = Rc::new(SumState { a: 1, b: 0 });
        let b = Rc::new(SumState { a: 1, b: 0 });

        selector.call(&a);
        selector.call(&b);

        assert_eq!(combiner.count(), 2);
    }

    // Mirrors `createSelectorMemoized.test.ts:83-130`: one, two and three extra
    // arguments are passed to the combiner and memoized on.
    #[test]
    fn memoized_selector_memoizes_on_up_to_three_extra_arguments() {
        let state = Rc::new(SumState { a: 10, b: 0 });

        let one = Rc::new(Counted::new(|| ()));
        let selector_one = create_selector_memoized_1_with_args(
            |state: &SumState| state.a,
            {
                let one = Rc::clone(&one);
                move |value, x1: (i64,)| {
                    let _ = one.call();
                    value + x1.0
                }
            },
        );
        assert_eq!(selector_one.call_with(&state, (1,)), 11);
        assert_eq!(selector_one.call_with(&state, (1,)), 11);
        assert_eq!(one.count(), 1);
        assert_eq!(selector_one.call_with(&state, (2,)), 12);
        assert_eq!(one.count(), 2);

        let two = Rc::new(Counted::new(|| ()));
        let selector_two = create_selector_memoized_1_with_args(
            |state: &SumState| state.a,
            {
                let two = Rc::clone(&two);
                move |value, args: (i64, i64)| {
                    let _ = two.call();
                    value + args.0 + args.1
                }
            },
        );
        assert_eq!(selector_two.call_with(&state, (1, 2)), 13);
        assert_eq!(selector_two.call_with(&state, (1, 2)), 13);
        assert_eq!(two.count(), 1);
        assert_eq!(selector_two.call_with(&state, (1, 5)), 16);
        assert_eq!(two.count(), 2);

        let three = Rc::new(Counted::new(|| ()));
        let selector_three = create_selector_memoized_1_with_args(
            |state: &SumState| state.a,
            {
                let three = Rc::clone(&three);
                move |value, args: (i64, i64, i64)| {
                    let _ = three.call();
                    value + args.0 + args.1 + args.2
                }
            },
        );
        assert_eq!(selector_three.call_with(&state, (1, 2, 3)), 16);
        assert_eq!(selector_three.call_with(&state, (1, 2, 3)), 16);
        assert_eq!(three.count(), 1);
        assert_eq!(selector_three.call_with(&state, (1, 2, 7)), 20);
        assert_eq!(three.count(), 2);
    }

    // The per-state cache is weak: entries die with their state, like upstream's
    // `WeakMap` (`packages/utils/src/store/createSelectorMemoized.ts:61`).
    #[test]
    fn memoized_selector_cache_entries_die_with_their_state() {
        let combiner = Rc::new(Counted::new(|| ()));

        let selector = create_selector_memoized_1(
            |state: &SumState| state.a,
            {
                let combiner = Rc::clone(&combiner);
                move |value| {
                    let _ = combiner.call();
                    value
                }
            },
        );

        let state = Rc::new(SumState { a: 1, b: 0 });
        selector.call(&state);
        selector.call(&state);
        assert_eq!(combiner.count(), 1);

        drop(state);
        selector.prune();

        // The pruned entry is gone; nothing holds the old state alive through the
        // selector. (Weak::strong_count on a dead handle is 0, and the next call
        // recomputes for a fresh state.)
        let fresh = Rc::new(SumState { a: 1, b: 0 });
        assert_eq!(selector.call(&fresh), 1);
        assert_eq!(combiner.count(), 2);
    }
}
