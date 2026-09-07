//! Port of `packages/utils/src/fastHooks.ts` (Base UI Phase A util).
//!
//! Upstream is a render-plumbing infra util with no dedicated test suite
//! (`ralph/generated/utils.json:62-66` records `testFiles: []`); the only test exercising any of
//! its API is the store unit's suite, which renders a `fastComponent`-wrapped component
//! (`packages/utils/src/store/ReactStore.test.tsx:6`,
//! `packages/utils/src/store/ReactStore.test.tsx:364-391`). The behavior claims below are cited
//! to the unit's own source per `specs/utils/fastHooks.md`.
//!
//! The protocol: a wrapper around a component function creates one per-component instance,
//! installs it as the module-level current instance, runs every registered hook's `before`
//! callback, calls the wrapped function, runs every registered hook's `after` callback, flips
//! `didInitialize` to `true`, and always clears the current instance afterwards — the clear sits
//! in a `finally`, so it happens even when the wrapped function throws
//! (`packages/utils/src/fastHooks.ts:70-88`). Specialized hooks (upstream: `useStore`)
//! registered at module load read the current instance during render via `getInstance()`
//! (`packages/utils/src/fastHooks.ts:17-19`,
//! `packages/utils/src/store/useStore.ts:136`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The module-level `hooks` registry and `currentInstance` slot become `thread_local`s:
//!   wasm is single-threaded, and per-thread isolation keeps native tests independent of each
//!   other — the same adaptation `create_log_once` made.
//! - JS objects are shared mutable references, so the current instance is stored and handed out
//!   as an [`InstanceHandle`] (`Rc<RefCell<Instance>>`): hooks, the wrapped function, and
//!   `get_instance()` all observe mutations made through the same object identity, which is what
//!   upstream's hook protocol relies on (upstream's `before` hook mutates the instance fields the
//!   render-phase consumer later reads, `packages/utils/src/store/useStore.ts:80-86`).
//! - The internal, structurally-typed `HookType` (`packages/utils/src/fastHooks.ts:8-11`, not
//!   exported) becomes the public [`Hook`] struct with [`HookCallback`] fields — Rust needs a
//!   named type for [`register`]'s parameter.
//! - Upstream creates the per-component instance lazily once per mounted component and reuses it
//!   across renders via `useRefWithInit` (`packages/utils/src/fastHooks.ts:67`;
//!   lazy-once semantics in `packages/utils/src/useRefWithInit.ts:16-22`). Leptos component
//!   bodies execute once per mount rather than on every reactive update
//!   (`specs/architecture.md`, "React ↔ Leptos state-management quick reference"), so the port
//!   creates the instance once per invocation of the wrapped function and the
//!   `didInitialize: false → true` transition happens within that single invocation, exactly as
//!   upstream's first render observes it (`packages/utils/src/fastHooks.ts:83`,
//!   `packages/utils/src/fastHooks.ts:124-128`).
//! - [`fast_component`] has no `fastComponentRef` counterpart: upstream's variant only combines
//!   the wrapper with `React.forwardRef` (`packages/utils/src/fastHooks.ts:118-122`), and Leptos
//!   has no forward-ref concept — components receive a `node_ref: NodeRef<T>` prop like any
//!   other prop (`specs/architecture.md`, "Refs / DOM access"), so the wrapped function already
//!   receives everything through its single props argument. The upstream wrapper's ref pass-through
//!   (`packages/utils/src/fastHooks.ts:77`) collapses with it.
//! - The `displayName` copy (`packages/utils/src/fastHooks.ts:90`) and the babel display-name
//!   allowlist (`babel.config.mjs:29-32`) are React devtooling concerns with no Rust/Leptos
//!   equivalent — N/A.
//! - The `finally` clear becomes a drop guard, so the slot is cleared on both normal return and
//!   unwind, matching upstream's exception behavior: `after` hooks and the `didInitialize`
//!   assignment are skipped while the current instance is still cleared
//!   (`packages/utils/src/fastHooks.ts:84-86`; UNVERIFIED upstream — inferred, no test asserts
//!   it — but implemented and pinned by the port's own tests).
//! - Upstream's `finally` assigns `currentInstance = undefined` rather than restoring a previous
//!   value (`packages/utils/src/fastHooks.ts:85`), so a nested wrapped call leaves the slot
//!   empty for the remainder of the outer call; the port preserves that quirk deliberately.

use std::cell::RefCell;
use std::rc::Rc;

/// The upstream `Instance` type (`packages/utils/src/fastHooks.ts:4-6`): the per-component
/// instance object specialized hooks hang their per-component state off and branch on for
/// one-time initialization (`packages/utils/src/store/useStore.ts:82-85`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instance {
    /// Upstream `didInitialize` (`packages/utils/src/fastHooks.ts:5`): `false` until the wrapped
    /// function's first invocation completes (`packages/utils/src/fastHooks.ts:83`).
    pub did_initialize: bool,
}

/// A shared handle to the per-component [`Instance`]: the Rust form of upstream passing the same
/// instance object to the hooks, the wrapped function (via `getInstance()`), and the wrapper's
/// own bookkeeping (`packages/utils/src/fastHooks.ts:70-88`).
pub type InstanceHandle = Rc<RefCell<Instance>>;

/// One half of the upstream `HookType` pair (`packages/utils/src/fastHooks.ts:8-11`): a callback
/// receiving the current per-component instance, able to read it (`borrow`), mutate it
/// (`borrow_mut`), or keep the handle for later, the same way upstream hooks keep and mutate the
/// instance object (`packages/utils/src/store/useStore.ts:80-86`).
pub type HookCallback = Rc<dyn Fn(&InstanceHandle)>;

/// The upstream `HookType` (`packages/utils/src/fastHooks.ts:8-11`): a `before`/`after` callback
/// pair run around every invocation of every [`fast_component`]-wrapped function, each receiving
/// the per-component instance as payload (`packages/utils/src/fastHooks.ts:73-75`,
/// `packages/utils/src/fastHooks.ts:79-81`). Upstream declares the payload loosely (`any`) and
/// concretizes it at the consumer.
#[derive(Clone)]
pub struct Hook {
    /// Runs immediately before the wrapped function (`packages/utils/src/fastHooks.ts:73-75`).
    pub before: HookCallback,
    /// Runs immediately after the wrapped function
    /// (`packages/utils/src/fastHooks.ts:79-81`).
    pub after: HookCallback,
}

thread_local! {
    /// The upstream module-level, append-only `hooks` registry
    /// (`packages/utils/src/fastHooks.ts:13`): populated only by [`register`], which has no
    /// unregister counterpart (`packages/utils/src/fastHooks.ts:25-27`).
    static HOOKS: RefCell<Vec<Hook>> = const { RefCell::new(Vec::new()) };

    /// The upstream module-level `currentInstance` slot
    /// (`packages/utils/src/fastHooks.ts:15`): set around each wrapped invocation and always
    /// cleared afterwards (`packages/utils/src/fastHooks.ts:70-88`).
    static CURRENT_INSTANCE: RefCell<Option<InstanceHandle>> = const { RefCell::new(None) };
}

/// Reads the module-level current instance, `None` outside a wrapped invocation
/// (`packages/utils/src/fastHooks.ts:17-19`). Specialized hooks call this during render
/// (`packages/utils/src/store/useStore.ts:136`).
pub fn get_instance() -> Option<InstanceHandle> {
    CURRENT_INSTANCE.with_borrow(Clone::clone)
}

/// Overwrites the module-level current instance (`packages/utils/src/fastHooks.ts:21-23`),
/// intended for code rendering outside a [`fast_component`] wrapper to install an instance
/// context (UNVERIFIED — inferred from the cited lines, no test asserts this upstream).
pub fn set_instance(instance: Option<InstanceHandle>) {
    CURRENT_INSTANCE.with_borrow_mut(|slot| *slot = instance);
}

/// Appends a [`Hook`] to the module-level, append-only registry
/// (`packages/utils/src/fastHooks.ts:25-27`); there is no unregister counterpart. In practice
/// the registry is populated once at module load by the specialized hooks
/// (`packages/utils/src/store/useStore.ts:78`).
pub fn register(hook: Hook) {
    HOOKS.with_borrow_mut(|hooks| hooks.push(hook));
}

/// Snapshots the registry for one phase's iteration. The snapshot means a hook registering
/// another hook mid-run cannot observe itself the way JS's live-array iteration would, but the
/// registry is populated once at module load in practice
/// (`packages/utils/src/store/useStore.ts:78`), so the difference is unobservable.
fn registered_hooks() -> Vec<Hook> {
    HOOKS.with_borrow(Vec::clone)
}

/// Clears the current-instance slot when dropped — the Rust form of upstream's `finally` block
/// (`packages/utils/src/fastHooks.ts:84-86`), running on both normal return and unwind.
struct ClearCurrentInstanceOnDrop;

impl Drop for ClearCurrentInstanceOnDrop {
    fn drop(&mut self) {
        CURRENT_INSTANCE.with_borrow_mut(|slot| *slot = None);
    }
}

/// Wraps a component function to enable performance optimizations for internal hooks — the port
/// of upstream `fastComponent` (`packages/utils/src/fastHooks.ts:63-92`), keeping its JSDoc
/// contract: the wrapper creates a stable instance object, sets it as the current context, calls
/// registered hooks before and after the wrapped function, then clears the context
/// (`packages/utils/src/fastHooks.ts:70-88`). The primary beneficiary is `useStore`, which
/// registers its hooks at module load and reads the current instance during render
/// (`packages/utils/src/store/useStore.ts:78`, `packages/utils/src/store/useStore.ts:136`).
///
/// Per invocation:
/// 1. create the per-component instance as `{ didInitialize: false }`
///    (`packages/utils/src/fastHooks.ts:124-128`),
/// 2. set it as the current instance (`packages/utils/src/fastHooks.ts:70-71`),
/// 3. run every registered hook's `before`
///    (`packages/utils/src/fastHooks.ts:73-75`),
/// 4. call the wrapped function and take its result
///    (`packages/utils/src/fastHooks.ts:77`, `packages/utils/src/fastHooks.ts:88`),
/// 5. run every registered hook's `after`
///    (`packages/utils/src/fastHooks.ts:79-81`),
/// 6. flip `did_initialize` to `true` (`packages/utils/src/fastHooks.ts:83`),
/// 7. clear the current instance, including when step 4 unwinds
///    (`packages/utils/src/fastHooks.ts:84-86`).
///
/// See the module docs for the `fastComponentRef` and display-name adaptations.
pub fn fast_component<P, R>(render: impl Fn(P) -> R + 'static) -> impl Fn(P) -> R + 'static {
    move |props| {
        let instance: InstanceHandle = Rc::new(RefCell::new(Instance::default()));

        CURRENT_INSTANCE.with_borrow_mut(|slot| *slot = Some(instance.clone()));
        let _clear_current_instance = ClearCurrentInstanceOnDrop;

        for hook in registered_hooks() {
            (hook.before)(&instance);
        }

        let result = render(props);

        for hook in registered_hooks() {
            (hook.after)(&instance);
        }

        instance.borrow_mut().did_initialize = true;

        result
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;

    use super::*;

    /// A recorder shared between a test and its hooks: each test's hooks capture their own
    /// recorder, so registrations left behind by other tests (the registry is append-only and
    /// never cleared, mirroring upstream) cannot pollute a test's recorded events.
    type Events = Rc<RefCell<Vec<&'static str>>>;

    fn hook_for(events: &Events, before_label: &'static str, after_label: &'static str) -> Hook {
        let before_events = Rc::clone(events);
        let after_events = Rc::clone(events);
        Hook {
            before: Rc::new(move |_: &InstanceHandle| {
                before_events.borrow_mut().push(before_label);
            }),
            after: Rc::new(move |_: &InstanceHandle| {
                after_events.borrow_mut().push(after_label);
            }),
        }
    }

    // The wrapper returns whatever the wrapped component function returns
    // (`packages/utils/src/fastHooks.ts:88`) with the same call signature as the input
    // (`packages/utils/src/fastHooks.ts:63-65`), so props flow through unchanged.
    #[test]
    fn returns_the_wrapped_function_result_with_props_passed_through() {
        let component = fast_component(|props: u32| props * 2);

        assert_eq!(component(21), 42);
    }

    // `currentInstance` is set immediately before the wrapped render call
    // (`packages/utils/src/fastHooks.ts:70-71`) and always cleared afterwards in a `finally`
    // block (`packages/utils/src/fastHooks.ts:84-86`); specialized hooks read it during render
    // via `getInstance()` (`packages/utils/src/fastHooks.ts:17-19`).
    #[test]
    fn exposes_the_instance_during_the_call_and_clears_it_afterwards() {
        let seen_during_call = Rc::new(Cell::new(false));
        let seen = Rc::clone(&seen_during_call);

        let component = fast_component(move |_: ()| {
            seen.set(get_instance().is_some());
        });
        component(());

        assert!(seen_during_call.get(), "instance exposed during the call");
        assert!(get_instance().is_none(), "instance cleared after the call");
    }

    // `before(instance)` runs immediately before the wrapped render function and `after(instance)`
    // runs immediately after it, each receiving the per-component `Instance` object as payload
    // (`packages/utils/src/fastHooks.ts:73-75`, `packages/utils/src/fastHooks.ts:79-81`).
    #[test]
    fn runs_before_and_after_hooks_around_the_call_with_the_instance_as_payload() {
        let events: Events = Rc::new(RefCell::new(Vec::new()));
        let handle_seen_in_before = Rc::new(RefCell::new(None::<InstanceHandle>));
        let handle_seen_in_call = Rc::new(RefCell::new(None::<InstanceHandle>));
        let handle_seen_in_after = Rc::new(RefCell::new(None::<InstanceHandle>));

        let before_events = Rc::clone(&events);
        let after_events = Rc::clone(&events);
        let before_handle = Rc::clone(&handle_seen_in_before);
        let call_handle = Rc::clone(&handle_seen_in_call);
        let after_handle = Rc::clone(&handle_seen_in_after);
        register(Hook {
            before: Rc::new(move |instance: &InstanceHandle| {
                before_events.borrow_mut().push("before");
                *before_handle.borrow_mut() = Some(instance.clone());
            }),
            after: Rc::new(move |instance: &InstanceHandle| {
                after_events.borrow_mut().push("after");
                *after_handle.borrow_mut() = Some(instance.clone());
            }),
        });

        let call_events = Rc::clone(&events);
        let component = fast_component(move |_: ()| {
            call_events.borrow_mut().push("call");
            *call_handle.borrow_mut() = get_instance();
        });
        component(());

        assert_eq!(
            events.borrow().as_slice(),
            ["before", "call", "after"],
            "hooks run around the wrapped call"
        );
        let before = handle_seen_in_before.borrow().clone().unwrap();
        let call = handle_seen_in_call.borrow().clone().unwrap();
        let after = handle_seen_in_after.borrow().clone().unwrap();
        assert!(
            Rc::ptr_eq(&before, &call) && Rc::ptr_eq(&call, &after),
            "hooks and the call observe the same instance object"
        );
    }

    // The instance is created as `{ didInitialize: false }` (`packages/utils/src/fastHooks.ts:124-128`)
    // and only flipped to `true` after the `after` hooks ran
    // (`packages/utils/src/fastHooks.ts:79-83`), so every hook on the first invocation observes
    // `false` — the flag specialized hooks branch on for one-time initialization
    // (`packages/utils/src/store/useStore.ts:82-85`).
    #[test]
    fn flips_did_initialize_to_true_only_after_the_after_hooks() {
        let events: Events = Rc::new(RefCell::new(Vec::new()));

        let before_events = Rc::clone(&events);
        let after_events = Rc::clone(&events);
        register(Hook {
            before: Rc::new(move |instance: &InstanceHandle| {
                before_events.borrow_mut().push(if instance.borrow().did_initialize {
                    "before:true"
                } else {
                    "before:false"
                });
            }),
            after: Rc::new(move |instance: &InstanceHandle| {
                after_events.borrow_mut().push(if instance.borrow().did_initialize {
                    "after:true"
                } else {
                    "after:false"
                });
            }),
        });

        let handle = Rc::new(RefCell::new(None::<InstanceHandle>));
        let handle_in_call = Rc::clone(&handle);
        let component = fast_component(move |_: ()| {
            *handle_in_call.borrow_mut() = get_instance();
        });
        component(());

        assert_eq!(
            events.borrow().as_slice(),
            ["before:false", "after:false"],
            "first-invocation hooks observe didInitialize: false"
        );
        assert!(
            handle.borrow().as_ref().unwrap().borrow().did_initialize,
            "the wrapper flips didInitialize after the after hooks"
        );
    }

    // Every registered hook runs, in registration order, around the wrapped call
    // (`packages/utils/src/fastHooks.ts:73-75`, `packages/utils/src/fastHooks.ts:79-81`); the
    // registry is append-only (`packages/utils/src/fastHooks.ts:13`,
    // `packages/utils/src/fastHooks.ts:25-27`).
    #[test]
    fn runs_every_registered_hook_in_registration_order() {
        let events: Events = Rc::new(RefCell::new(Vec::new()));

        register(hook_for(&events, "first:before", "first:after"));
        register(hook_for(&events, "second:before", "second:after"));
        let call_events = Rc::clone(&events);

        let component = fast_component(move |_: ()| {
            call_events.borrow_mut().push("call");
        });
        component(());

        assert_eq!(
            events.borrow().as_slice(),
            ["first:before", "second:before", "call", "first:after", "second:after"],
        );
    }

    // If the wrapped render function throws, the `after` hooks and the `didInitialize = true`
    // assignment are skipped and the error propagates, while the current instance is still
    // cleared because the cleanup sits in `finally` (`packages/utils/src/fastHooks.ts:70-86`;
    // UNVERIFIED upstream — inferred, no test asserts it — pinned here so the port keeps the
    // exception behavior the `finally` implies).
    #[test]
    fn clears_the_instance_and_skips_after_hooks_when_the_call_unwinds() {
        let events: Events = Rc::new(RefCell::new(Vec::new()));
        let after_events = Rc::clone(&events);
        register(Hook {
            before: Rc::new(|_: &InstanceHandle| {}),
            after: Rc::new(move |_: &InstanceHandle| {
                after_events.borrow_mut().push("after");
            }),
        });

        let handle = Rc::new(RefCell::new(None::<InstanceHandle>));
        let handle_in_before = Rc::clone(&handle);
        register(Hook {
            before: Rc::new(move |instance: &InstanceHandle| {
                *handle_in_before.borrow_mut() = Some(instance.clone());
            }),
            after: Rc::new(|_: &InstanceHandle| {}),
        });

        let result = catch_unwind(AssertUnwindSafe(|| {
            let component = fast_component(|_: ()| -> u32 { panic!("render failure") });
            component(())
        }));

        assert!(result.is_err(), "the panic propagates out of the wrapper");
        assert!(get_instance().is_none(), "the finally clear still ran");
        assert!(
            events.borrow().is_empty(),
            "the after hooks were skipped"
        );
        assert!(
            !handle.borrow().as_ref().unwrap().borrow().did_initialize,
            "didInitialize was not flipped"
        );
    }

    // `setInstance` overwrites the module-level current instance; intended for code rendering
    // outside a `fastComponent` wrapper to install an instance context (UNVERIFIED — inferred
    // from `packages/utils/src/fastHooks.ts:21-23`, no test asserts this upstream).
    #[test]
    fn set_installs_an_instance_that_get_reads_back() {
        let instance: InstanceHandle = Rc::new(RefCell::new(Instance::default()));

        set_instance(Some(instance.clone()));
        let read_back = get_instance().unwrap();
        assert!(Rc::ptr_eq(&read_back, &instance), "same object identity");

        set_instance(None);
        assert!(get_instance().is_none());
    }

    // The per-component instance is created lazily exactly once per mounted wrapper component
    // (`packages/utils/src/fastHooks.ts:67`), so two components never share one instance; in the
    // port each invocation of the wrapped function is one component body execution (see module
    // docs).
    #[test]
    fn gives_each_invocation_its_own_instance() {
        let first_handle = Rc::new(RefCell::new(None::<InstanceHandle>));
        let second_handle = Rc::new(RefCell::new(None::<InstanceHandle>));

        let first = Rc::clone(&first_handle);
        let first_component = fast_component(move |_: ()| {
            *first.borrow_mut() = get_instance();
        });
        first_component(());

        let second = Rc::clone(&second_handle);
        let second_component = fast_component(move |_: ()| {
            *second.borrow_mut() = get_instance();
        });
        second_component(());

        let first = first_handle.borrow().clone().unwrap();
        let second = second_handle.borrow().clone().unwrap();
        assert!(!Rc::ptr_eq(&first, &second));
    }

    // Upstream's `finally` assigns `currentInstance = undefined` rather than restoring a previous
    // value (`packages/utils/src/fastHooks.ts:85`), so a nested wrapped call leaves the slot
    // empty for the remainder of the outer call; pinned so the port doesn't silently "improve"
    // the quirk.
    #[test]
    fn nested_invocation_clears_the_current_instance_for_the_outer_call() {
        let outer_after_inner = Rc::new(Cell::new(true));
        let observed = Rc::clone(&outer_after_inner);

        let component = fast_component(move |_: ()| {
            let inner = fast_component(|_: ()| {});
            inner(());
            observed.set(get_instance().is_some());
        });
        component(());

        assert!(
            !outer_after_inner.get(),
            "the inner finally cleared the slot for the still-running outer call"
        );
    }
}
