//! Port of `packages/utils/src/useControlled.ts` (Base UI Phase A util).
//!
//! Upstream is a single React hook implementing the standard controlled/uncontrolled duality:
//! the mode is decided once from the first render's `controlled` prop
//! (`packages/utils/src/useControlled.ts:41`), an internal `useState` seeded with the initial
//! `default` holds the uncontrolled value (`packages/utils/src/useControlled.ts:42`), the
//! exposed value reads the controlled prop whenever the hook is controlled and the prop is
//! defined, falling back to the internal state otherwise
//! (`packages/utils/src/useControlled.ts:45`), and the returned setter only reaches the internal
//! state while the hook is uncontrolled (`packages/utils/src/useControlled.ts:82-89`). In
//! development builds two effects emit diagnostics: a mode-switch warning comparing the live
//! `controlled` prop's defined-ness against the fixed initial mode
//! (`packages/utils/src/useControlled.ts:48-63`) and a default-change warning comparing the
//! live `default` against the initially captured one while uncontrolled
//! (`packages/utils/src/useControlled.ts:65-79`), both dispatched through the `error` binding
//! (`packages/utils/src/useControlled.ts:5`) — a `createLogOnce` logger whose per-output dedup
//! is what produces the warn-only-on-first-change behavior the suite observes
//! (`packages/utils/src/useControlled.test.tsx:222-242`, `:244-278`).
//!
//! The proven contract comes from `packages/utils/src/useControlled.test.tsx` (runtime) and
//! `packages/utils/src/useControlled.spec.ts` (type-level only); see `specs/utils/useControlled.md`.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - Upstream's two TypeScript overloads (`packages/utils/src/useControlled.ts:28-33`) share a
//!   single implementation body (`packages/utils/src/useControlled.ts:34-92`) and differ only in
//!   whether `default` may be `undefined`. Rust has no `undefined`, so the port ships the
//!   defined-`default` shape (`default` yields a plain `T`); callers wanting the optional shape
//!   instantiate `T = Option<U>`, which routes through the same runtime behavior.
//! - Upstream re-renders with new prop values; Leptos components run once and props are reactive
//!   sources. `controlled` becomes `C: Get<Value = Option<T>>` and `default` becomes
//!   `D: Get<Value = T>`. The first-render snapshot is an untracked read at hook-call time
//!   (`isControlled`, `packages/utils/src/useControlled.ts:41`; the `useState` seed and the
//!   `defaultValue` ref, `packages/utils/src/useControlled.ts:42` and `:65`), and the dev effects
//!   observe the sources reactively the way upstream's dependency arrays
//!   (`packages/utils/src/useControlled.ts:63`, `:79`) did. The hook must be called inside a
//!   reactive owner (a component).
//! - The exposed value is a derived local `Signal` recomputing the upstream render-phase ternary
//!   (`packages/utils/src/useControlled.ts:45`) on each read. It is deliberately not a
//!   `Memo`: the branch is two signal reads, and `Memo::new` requires `Send + Sync` internals,
//!   which would exclude local-storage signal types (see `specs/architecture.md`, the
//!   `useMemo` row's "derived signal by default" rule).
//! - Upstream's setter type `React.Dispatch<React.SetStateAction<T>>`
//!   (`packages/utils/src/useControlled.spec.ts:13`) becomes the [`SetValueAction`] enum
//!   (plain value or functional updater — the updater form's runtime behavior is UNVERIFIED
//!   upstream per the spec, `specs/utils/useControlled.md`, "Events") consumed by the returned
//!   closure. The closure is created exactly once per hook call and the identity-stability
//!   concern of upstream's `useCallback` (`packages/utils/src/useControlled.ts:88`) is a React
//!   render-phase artifact, N/A in Leptos — same as the `react_store` port's
//!   `use_state_setter`.
//! - The dev-gated block (`process.env.NODE_ENV !== 'production'`,
//!   `packages/utils/src/useControlled.ts:47`) becomes `cfg!(debug_assertions)` around the
//!   effects' creation, so release builds skip the effects entirely like upstream does; the
//!   gating convention matches the crate's `react_store`/`use_animation_frame` ports.
//! - Upstream's `serializeToDevModeString` comparison
//!   (`packages/utils/src/useControlled.ts:70`, `:94-124`) exists to make JS's exotic value
//!   semantics non-warning: `JSON.stringify(NaN)` is `'null'`, so NaN is self-equal, and
//!   element/function-bearing objects normalize to equal strings. Rust values cannot throw on
//!   comparison and have no element/function duals, so the port compares with `PartialEq` plus
//!   one canonicalization preserving the one observable quirk the suite pins — NaN is
//!   self-equivalent (`packages/utils/src/useControlled.test.tsx:150-154`), resolved by
//!   `f32`/`f64` downcast, the only Rust analogs of a self-unequal value.
//! - `state` and `name` appear in upstream's effect dependency arrays
//!   (`packages/utils/src/useControlled.ts:63`, `:79`) but are static developer labels; the port
//!   captures them once at hook-call time.
//! - The warnings dispatch through the ported [`crate::error`] binding exactly as upstream
//!   imports it (`packages/utils/src/useControlled.ts:5`), inheriting the log-once registry's
//!   dedup — the mechanism behind upstream's observed warn-once semantics.

use reactive_graph::effect::Effect;
use reactive_graph::owner::LocalStorage;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set, Update};
use reactive_graph::wrappers::read::Signal;

use crate::error::error;

/// The upstream `UseControlledProps` interface
/// (`packages/utils/src/useControlled.ts:7-24`), parameterized by the two reactive sources
/// instead of the value type: `C` yields `Option<T>` (the `controlled` prop) and `D` yields
/// `T` (the `default` prop). See the module docs on the optional-default overload.
pub struct UseControlledProps<C, D> {
    /// Holds the component value when it's controlled
    /// (`packages/utils/src/useControlled.ts:8-10`): `None` while uncontrolled.
    pub controlled: C,

    /// The default value when uncontrolled, and the fallback if a controlled value later
    /// becomes absent (`packages/utils/src/useControlled.ts:12-14`).
    pub default: D,

    /// The component name displayed in warnings
    /// (`packages/utils/src/useControlled.ts:16-18`).
    pub name: &'static str,

    /// The name of the state variable displayed in warnings
    /// (`packages/utils/src/useControlled.ts:20-22`); `None` is upstream's omitted field,
    /// resolved to `'value'` by the default parameter
    /// (`packages/utils/src/useControlled.ts:38`).
    pub state: Option<&'static str>,
}

impl<C, D> UseControlledProps<C, D> {
    /// Constructs the props with the upstream required fields
    /// (`packages/utils/src/useControlled.ts:34-37`); `state` defaults to omitted
    /// (`packages/utils/src/useControlled.ts:38`) and can be overridden on the returned struct.
    pub fn new(controlled: C, default: D, name: &'static str) -> Self {
        Self {
            controlled,
            default,
            name,
            state: None,
        }
    }
}

/// The Rust form of React's `SetStateAction<T>` — the argument type of upstream's returned
/// setter (`packages/utils/src/useControlled.ts:83`): either a new value or a functional
/// updater over the previous value.
pub enum SetValueAction<T> {
    /// A replacement value (`packages/utils/src/useControlled.test.tsx:45`,
    /// `setValueState(2)`).
    Value(T),

    /// A functional updater over the previous value — accepted by upstream's
    /// `SetStateAction` setter type (`packages/utils/src/useControlled.spec.ts:13`); its
    /// runtime behavior is UNVERIFIED upstream (`specs/utils/useControlled.md`, "Events").
    Update(Box<dyn FnOnce(&T) -> T>),
}

impl<T> From<T> for SetValueAction<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

/// The upstream `useControlled` hook (`packages/utils/src/useControlled.ts:34-92`), returning
/// the `[value, setValue]` tuple (`packages/utils/src/useControlled.ts:91`) as a derived
/// reactive read plus a guarded setter. Must be called inside a reactive owner (a component):
/// the dev diagnostics effects attach to the calling owner and are disposed with it.
///
/// The mode is fixed from the `controlled` source's value at call time
/// (`packages/utils/src/useControlled.ts:41`); a later defined-ness change only affects the
/// exposed value's fallback branch and emits the dev warning, it never re-opens the setter.
pub fn use_controlled<T, C, D>(
    props: UseControlledProps<C, D>,
) -> (Signal<T, LocalStorage>, impl Fn(SetValueAction<T>))
where
    T: PartialEq + Clone + 'static,
    C: Get<Value = Option<T>> + GetUntracked<Value = Option<T>> + Clone + 'static,
    D: Get<Value = T> + GetUntracked<Value = T> + 'static,
{
    let UseControlledProps {
        controlled,
        default,
        name,
        state,
    } = props;

    let state = state.unwrap_or("value");

    // Upstream's `isControlled` ref capture (`packages/utils/src/useControlled.ts:41`): the
    // first render's defined-ness, fixed for the hook's lifetime.
    let is_controlled = controlled.get_untracked().is_some();

    // Upstream's `useState(defaultProp)` seed (`packages/utils/src/useControlled.ts:42`).
    let value_state = RwSignal::new_local(default.get_untracked());

    // Upstream's exposed value ternary (`packages/utils/src/useControlled.ts:45`): while
    // controlled, a defined controlled value wins; otherwise the internal state — which is
    // also the fallback when a controlled value later becomes absent.
    let controlled_for_value = controlled.clone();
    let value = Signal::derive_local(move || {
        if is_controlled {
            match controlled_for_value.get() {
                Some(value) => value,
                None => value_state.get(),
            }
        } else {
            value_state.get()
        }
    });

    // The dev-gated block (`packages/utils/src/useControlled.ts:47-80`): release builds skip
    // the effects entirely, like upstream.
    if cfg!(debug_assertions) {
        // The mode-switch warning (`packages/utils/src/useControlled.ts:48-63`): the condition
        // compares the live defined-ness against the FIXED initial mode, so a controlled prop
        // that merely flickers defined→absent on an initially-uncontrolled hook is silent
        // (`isControlled !== (controlled !== undefined)`).
        Effect::new(move || {
            if controlled.get().is_some() != is_controlled {
                error().log(&[&controlled_state_switch_message(is_controlled, state, name)]);
            }
        });

        // The default-change warning (`packages/utils/src/useControlled.ts:65-79`): the
        // comparison is against the initially captured default (the ref at
        // `packages/utils/src/useControlled.ts:65`), and only while uncontrolled. The
        // warn-only-on-first-change behavior the suite observes is the shared `error`
        // registry's log-once dedup, not this effect's condition.
        let initial_default = default.get_untracked();
        Effect::new(move || {
            let current_default = default.get();
            if !is_controlled && !dev_mode_equivalent(&initial_default, &current_default) {
                error().log(&[&default_state_change_message(state, name)]);
            }
        });
    }

    // Upstream's `setValueIfUncontrolled` (`packages/utils/src/useControlled.ts:82-89`): the
    // write reaches the internal state only while the hook is uncontrolled.
    let set_value = move |action: SetValueAction<T>| {
        if !is_controlled {
            match action {
                SetValueAction::Value(value) => value_state.set(value),
                SetValueAction::Update(update) => {
                    value_state.update(|previous| {
                        let next = update(previous);
                        *previous = next;
                    });
                }
            }
        }
    };

    // Upstream's tuple return (`packages/utils/src/useControlled.ts:91`).
    (value, set_value)
}

/// The mode-switch warning body (`packages/utils/src/useControlled.ts:51-60`), joined with
/// newlines. `initially_controlled` is the hook's fixed first-render mode, so the wording is
/// anchored to it — mirroring the template's fixed `isControlled` interpolation.
fn controlled_state_switch_message(initially_controlled: bool, state: &str, name: &str) -> String {
    [
        format!(
            "A component is changing the {}controlled {state} state of {name} to be {}controlled.",
            if initially_controlled { "" } else { "un" },
            if initially_controlled { "un" } else { "" },
        ),
        "Elements should not switch from uncontrolled to controlled (or vice versa).".to_string(),
        format!(
            "Decide between using a controlled or uncontrolled {name} \
             element for the lifetime of the component."
        ),
        "The nature of the state is determined during the first render. It's considered \
         controlled if the value is not `undefined`."
            .to_string(),
        "More info: https://fb.me/react-controlled-components".to_string(),
    ]
    .join("\n")
}

/// The default-change warning body (`packages/utils/src/useControlled.ts:73-77`), a single
/// line — upstream concatenates two template literals including the trailing space after
/// `initialized. `.
fn default_state_change_message(state: &str, name: &str) -> String {
    format!(
        "A component is changing the default {state} state of an uncontrolled {name} after \
         being initialized. To suppress this warning opt to use a controlled {name}."
    )
}

/// The Rust analog of `serializeToDevModeString(a) !== serializeToDevModeString(b)`
/// (`packages/utils/src/useControlled.ts:70`, `:94-124`): structural equality plus the NaN
/// canonicalization the upstream serializer gets from `JSON.stringify(NaN) === 'null'` —
/// upstream renders with a NaN default without warning
/// (`packages/utils/src/useControlled.test.tsx:150-154`), which naive equality would break
/// since NaN is self-unequal in Rust too. `T: 'static` enables the `f32`/`f64` downcast; no
/// other Rust type is self-unequal.
fn dev_mode_equivalent<T: PartialEq + 'static>(initial: &T, current: &T) -> bool {
    if initial == current {
        return true;
    }
    is_nan_value(initial) && is_nan_value(current)
}

fn is_nan_value(value: &dyn std::any::Any) -> bool {
    if let Some(value) = value.downcast_ref::<f64>() {
        value.is_nan()
    } else if let Some(value) = value.downcast_ref::<f32>() {
        value.is_nan()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;

    use super::*;
    use crate::create_log_once::{Severity, has_logged, logged_outputs, reset};

    /// The full `console.error` output for the mode-switch warnings, with the `Base UI` prefix
    /// the `error` binding adds (`packages/utils/src/error.ts:3`) — the exact strings upstream
    /// asserts (`packages/utils/src/useControlled.test.tsx:72-74`, `:106-108`).
    const UNCONTROLLED_TO_CONTROLLED_OUTPUT: &str = concat!(
        "Base UI: A component is changing the uncontrolled value state of TestComponent to be controlled.\n",
        "Elements should not switch from uncontrolled to controlled (or vice versa).\n",
        "Decide between using a controlled or uncontrolled TestComponent element for the lifetime of the component.\n",
        "The nature of the state is determined during the first render. It's considered controlled if the value is not `undefined`.\n",
        "More info: https://fb.me/react-controlled-components",
    );

    const CONTROLLED_TO_UNCONTROLLED_OUTPUT: &str = concat!(
        "Base UI: A component is changing the controlled value state of TestHook to be uncontrolled.\n",
        "Elements should not switch from uncontrolled to controlled (or vice versa).\n",
        "Decide between using a controlled or uncontrolled TestHook element for the lifetime of the component.\n",
        "The nature of the state is determined during the first render. It's considered controlled if the value is not `undefined`.\n",
        "More info: https://fb.me/react-controlled-components",
    );

    /// What a previous-state (instead of fixed-initial-mode) comparison would emit when the
    /// defined controlled value of the initially-uncontrolled `TestComponent` hook returns to
    /// absent — pinned absent in the fixed-mode test below.
    const CONTROLLED_TO_UNCONTROLLED_TESTCOMPONENT_OUTPUT: &str = concat!(
        "Base UI: A component is changing the controlled value state of TestComponent to be uncontrolled.\n",
        "Elements should not switch from uncontrolled to controlled (or vice versa).\n",
        "Decide between using a controlled or uncontrolled TestComponent element for the lifetime of the component.\n",
        "The nature of the state is determined during the first render. It's considered controlled if the value is not `undefined`.\n",
        "More info: https://fb.me/react-controlled-components",
    );

    /// The default-change warning output (`packages/utils/src/useControlled.test.tsx:129-131`)
    /// with the `Base UI` prefix.
    const DEFAULT_CHANGE_OUTPUT: &str = "Base UI: A component is changing the default value state of an uncontrolled TestComponent after being initialized. To suppress this warning opt to use a controlled TestComponent.";

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:30-49`: rendering with only `default`
    // exposes the default, and the setter updates the exposed value — including the functional
    // updater form the setter type accepts (`packages/utils/src/useControlled.spec.ts:13`).
    #[test]
    fn uncontrolled_mode_initializes_to_the_default_and_its_setter_updates_it() {
        reset();
        let owner = in_owner();

        let controlled: RwSignal<Option<i64>> = RwSignal::new(None);
        let default: RwSignal<i64> = RwSignal::new(1);
        let (value, set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        assert_eq!(value.get_untracked(), 1);

        set_value(SetValueAction::Value(2));
        assert_eq!(value.get_untracked(), 2);

        set_value(SetValueAction::Update(Box::new(|previous| previous + 10)));
        assert_eq!(value.get_untracked(), 12);

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:51-62`: rendering with a `controlled`
    // value exposes it, and later controlled values flow through reactively. The setter is a
    // no-op while controlled (`packages/utils/src/useControlled.ts:84-86`).
    #[test]
    fn controlled_mode_exposes_the_controlled_value_and_the_setter_no_ops() {
        reset();
        let owner = in_owner();

        let controlled: RwSignal<Option<i64>> = RwSignal::new(Some(1));
        let default: RwSignal<i64> = RwSignal::new(0);
        let (value, set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        assert_eq!(value.get_untracked(), 1);

        controlled.set(Some(7));
        assert_eq!(value.get_untracked(), 7);

        set_value(SetValueAction::Value(2));
        assert_eq!(value.get_untracked(), 7);

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:64-75`: an initially uncontrolled
    // instance renders without a warning, and re-rendering with a controlled value emits the
    // dev warning with the exact upstream message.
    #[test]
    fn warns_when_switching_from_uncontrolled_to_controlled() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<String>> = RwSignal::new(None);
        let default: RwSignal<String> = RwSignal::new(String::new());
        let (value, _set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        Executor::poll_local();
        assert!(!has_logged(
            Severity::Error,
            UNCONTROLLED_TO_CONTROLLED_OUTPUT
        ));

        controlled.set(Some("foobar".to_string()));
        Executor::poll_local();
        assert!(has_logged(
            Severity::Error,
            UNCONTROLLED_TO_CONTROLLED_OUTPUT
        ));

        // The switch only warns: the exposed value is anchored to the fixed initial mode
        // (`packages/utils/src/useControlled.ts:45` — `isControlled` is false, so the value
        // stays the internal state even though the prop is now defined; upstream's test
        // asserts only the warning, `useControlled.test.tsx:70-74`).
        assert_eq!(value.get_untracked(), "");

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:77-117`: switching a controlled
    // instance to uncontrolled emits the dev warning, the exposed value falls back to the
    // default, and a subsequent setter call leaves it unchanged — the mode is fixed at first
    // call, so the setter no-ops even though the prop is now absent
    // (`packages/utils/src/useControlled.ts:84-86`).
    #[test]
    fn controlled_to_uncontrolled_warns_falls_back_to_default_and_setter_no_ops() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<&'static str>> = RwSignal::new(Some("foobar"));
        let default: RwSignal<&'static str> = RwSignal::new("default");
        let (value, set_value) =
            use_controlled(UseControlledProps::new(controlled, default, "TestHook"));

        assert_eq!(value.get_untracked(), "foobar");

        controlled.set(None);
        Executor::poll_local();
        assert!(has_logged(
            Severity::Error,
            CONTROLLED_TO_UNCONTROLLED_OUTPUT
        ));

        assert_eq!(value.get_untracked(), "default");

        set_value("next".into());
        assert_eq!(value.get_untracked(), "default");

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:119-132`: changing `default` after
    // initialization while uncontrolled emits the dev warning with the exact upstream message.
    #[test]
    fn warns_when_the_default_changes_after_initialization_while_uncontrolled() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<i64>> = RwSignal::new(None);
        let default: RwSignal<i64> = RwSignal::new(0);
        let (_value, _set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        Executor::poll_local();
        assert!(!has_logged(Severity::Error, DEFAULT_CHANGE_OUTPUT));

        default.set(1);
        Executor::poll_local();
        assert!(has_logged(Severity::Error, DEFAULT_CHANGE_OUTPUT));

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:134-148`: with `controlled`
    // provided, supplying and then changing `default` emits no warning at any point.
    #[test]
    fn does_not_warn_when_the_default_changes_while_controlled() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<i64>> = RwSignal::new(Some(1));
        let default: RwSignal<i64> = RwSignal::new(0);
        let (_value, _set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        Executor::poll_local();
        assert!(!has_logged(Severity::Error, DEFAULT_CHANGE_OUTPUT));

        default.set(1);
        Executor::poll_local();
        assert!(!has_logged(Severity::Error, DEFAULT_CHANGE_OUTPUT));

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:150-154`: a NaN default must not
    // warn on initialization — the case the upstream serializer's `JSON.stringify(NaN) ===
    // 'null'` canonicalization exists for, since NaN is self-unequal under naive equality.
    #[test]
    fn does_not_warn_when_the_default_is_nan() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<f64>> = RwSignal::new(None);
        let default: RwSignal<f64> = RwSignal::new(f64::NAN);
        let (_value, _set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        Executor::poll_local();
        assert!(!has_logged(Severity::Error, DEFAULT_CHANGE_OUTPUT));

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useControlled.test.tsx:222-242`: only the first
    // post-initialization default change warns; later changes to other values and back to the
    // initial default emit no further console output. The first change exhausts the shared
    // `error` registry's tolerance for that exact output (log-once dedup), and the change back
    // to the initial value also fails the comparison condition itself
    // (`packages/utils/src/useControlled.ts:70`). Asserted as registry deltas — the analog of
    // upstream's per-act `console.error` spy scope — since an already-emitted output stays in
    // the registry.
    #[test]
    fn warns_only_when_the_default_changes_for_the_first_time() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<i64>> = RwSignal::new(None);
        let default: RwSignal<i64> = RwSignal::new(0);
        let (_value, _set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        Executor::poll_local();
        assert!(logged_outputs(Severity::Error).is_empty());

        default.set(1);
        Executor::poll_local();
        assert_eq!(
            logged_outputs(Severity::Error),
            vec![DEFAULT_CHANGE_OUTPUT.to_string()]
        );

        default.set(2);
        Executor::poll_local();
        assert_eq!(
            logged_outputs(Severity::Error),
            vec![DEFAULT_CHANGE_OUTPUT.to_string()],
            "the repeated identical warning is suppressed by the log-once dedup"
        );

        default.set(0);
        Executor::poll_local();
        assert_eq!(
            logged_outputs(Severity::Error),
            vec![DEFAULT_CHANGE_OUTPUT.to_string()],
            "returning to the initial default also fails the change condition itself"
        );

        owner.cleanup();
    }

    // Mirrors the exotic-default initialization tests
    // (`packages/utils/src/useControlled.test.tsx:156-169` arrays, `:171-186` elements,
    // `:188-205` functions, `:207-220` bigint, `:280-288` null): exotic default values
    // initialize without spurious warnings or failures. Rust values cannot throw on
    // comparison (module docs), so the ported contract is the no-spurious-warning half; each
    // scenario runs in its own owner like a fresh upstream render.
    #[test]
    fn exotic_defaults_initialize_without_warning() {
        let scenarios: Vec<Box<dyn Fn() -> Owner>> = vec![
            // `default: []` (array, useControlled.test.tsx:156-169)
            Box::new(|| {
                let owner = in_owner();
                let controlled: RwSignal<Option<Vec<String>>> = RwSignal::new(None);
                let default: RwSignal<Vec<String>> = RwSignal::new(Vec::new());
                let _ = use_controlled(UseControlledProps::new(
                    controlled,
                    default,
                    "TestComponent",
                ));
                owner
            }),
            // `default: { value: <span/> }` / `{ value: fn }` (elements/functions,
            // useControlled.test.tsx:171-205) — a Rust value of an arbitrary structured type
            Box::new(|| {
                #[derive(Clone, PartialEq)]
                struct Structured {
                    inner: Option<Vec<u8>>,
                }
                let owner = in_owner();
                let controlled: RwSignal<Option<Structured>> = RwSignal::new(None);
                let default: RwSignal<Structured> = RwSignal::new(Structured {
                    inner: Some(vec![1]),
                });
                let _ = use_controlled(UseControlledProps::new(
                    controlled,
                    default,
                    "TestComponent",
                ));
                owner
            }),
            // `default: 1n` (bigint, useControlled.test.tsx:207-220)
            Box::new(|| {
                let owner = in_owner();
                let controlled: RwSignal<Option<i128>> = RwSignal::new(None);
                let default: RwSignal<i128> = RwSignal::new(1);
                let _ = use_controlled(UseControlledProps::new(
                    controlled,
                    default,
                    "TestComponent",
                ));
                owner
            }),
            // `default: null` (null, useControlled.test.tsx:280-288) — `T = Option<_>`
            Box::new(|| {
                let owner = in_owner();
                let controlled: RwSignal<Option<Option<&'static str>>> = RwSignal::new(None);
                let default: RwSignal<Option<&'static str>> = RwSignal::new(None);
                let _ = use_controlled(UseControlledProps::new(
                    controlled,
                    default,
                    "TestComponent",
                ));
                owner
            }),
        ];

        for scenario in scenarios {
            reset();
            let _ = Executor::init_futures_executor();
            let owner = scenario();
            Executor::poll_local();
            assert!(!has_logged(Severity::Error, DEFAULT_CHANGE_OUTPUT));
            owner.cleanup();
        }
    }

    // Pins the fixed-mode wording anchor (`packages/utils/src/useControlled.ts:52-54`): the
    // message is built from the INITIAL mode, not the live prop — the same initially
    // uncontrolled hook going back to absent after having been defined does not warn (the
    // condition `isControlled !== (controlled !== undefined)` is false again), which a
    // previous-state comparison would get wrong.
    #[test]
    fn a_defined_controlled_value_returning_to_absent_on_an_uncontrolled_hook_is_silent() {
        reset();
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let controlled: RwSignal<Option<i64>> = RwSignal::new(None);
        let default: RwSignal<i64> = RwSignal::new(0);
        let (_value, _set_value) = use_controlled(UseControlledProps::new(
            controlled,
            default,
            "TestComponent",
        ));

        controlled.set(Some(1));
        Executor::poll_local();
        assert!(has_logged(
            Severity::Error,
            UNCONTROLLED_TO_CONTROLLED_OUTPUT
        ));

        controlled.set(None);
        Executor::poll_local();
        assert!(!has_logged(
            Severity::Error,
            CONTROLLED_TO_UNCONTROLLED_TESTCOMPONENT_OUTPUT
        ));

        owner.cleanup();
    }
}
