//! Port of `packages/utils/src/useMergedRefs.ts` (Base UI Phase A util).
//!
//! Upstream merges several refs into a single memoized callback ref — "This makes sure multiple
//! refs are updated together and have the same value" (`packages/utils/src/useMergedRefs.ts:15-21`).
//! The fixed-arity hook `useMergedRefs(a, b[, c[, d]])` (`packages/utils/src/useMergedRefs.ts:30-41`)
//! and the array hook `useMergedRefsN(refs)` (`packages/utils/src/useMergedRefs.ts:48-54`) both
//! create the fork box once via `useRefWithInit(createForkRef)` (`:36`, `:49`), and — whenever the
//! render body re-runs with a changed set of branch identities (`didChange` `:64-78`,
//! `didChangeN` `:80-85`) — rebuild the setter (`update`, `:87-153`), returning
//! `forkRef.callback` (`:40`, `:53`), a `React.RefCallback<I> | null` (`:6`).
//!
//! The setter's own invocation protocol (`update`, `packages/utils/src/useMergedRefs.ts:95-152`)
//! is the ported contract:
//!
//! - When every branch is nullish the setter itself is `null` (`:90-93`).
//! - An invocation first runs and drops any pending detach cleanup (`:96-99`).
//! - With a non-null instance it forks the instance to every live branch: function branches are
//!   invoked with the instance and a returned function is captured as that branch's attach-time
//!   cleanup (`:110-116`), object branches get `ref.current = instance` (`:117-119`), and
//!   nullish branches are skipped silently (`:106-108`). It then stores a detach cleanup that,
//!   per branch, invokes the captured attach-time cleanup if there was one (`:133-135`), else
//!   invokes the function branch again with `null` — the legacy no-cleanup detach protocol
//!   (`:136-140`) — and nulls object branches (`:143-146`).
//! - A `null` instance invocation runs only the pending cleanup and never touches the branches
//!   (`:101`) — a detached callback branch is not re-invoked with `null` once it returned a
//!   cleanup.
//!
//! Test-proven behavior (`packages/utils/src/useMergedRefs.test.tsx`, per
//! `specs/utils/useMergedRefs.md`): mounting forks the node to both an object branch and a
//! `useState`-setter callback branch without dev warnings (`:15-30`); a single live branch among
//! nullish ones still fires (`:33-50`); all-nullish branches render cleanly with the null setter
//! (`:57-75`); a callback branch is invoked exactly once at attach with the mounted node and its
//! returned cleanup is not invoked at attach (`:157-159`); on unmount an attach-cleanup branch's
//! cleanup runs exactly once and its callback is never re-invoked (`:164-167`) while a
//! no-cleanup branch is invoked once with `null` and not again (`:169-172`); the branch-change
//! behaviors (`:91-124`) dissolve, see below.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The `InputRef<I>` union — `React.Ref<I> | null | undefined` (`:4-5`) — becomes the
//!   [`InputRef`] enum: [`InputRef::Object`], [`InputRef::Callback`], [`InputRef::Empty`]. The
//!   `typeof ref` dispatch in `update` (`:109`, `:131`) is the same discrimination in type form.
//! - The object-ref flavor is generic over the [`RefObject`] trait — anything with a settable
//!   `current` slot, upstream's `ref.current = instance` / `ref.current = null` writes
//!   (`:117-119`, `:143-146`). The [`InputRef::Object`] variant holds the handle as a
//!   `Rc<dyn RefObject<T>>` trait object rather than a concrete type parameter, for the same
//!   reason `get_react_element_ref`'s ref handle is generic (the port's concrete ref type is
//!   chosen by callers per `specs/architecture.md`, "Refs / DOM access" — components use
//!   `NodeRef<T>`, which the `leptos-ui` layer can implement [`RefObject`] for because the trait
//!   is local to this crate) and so that branches of different object flavors merge in one call
//!   (upstream's `React.Ref` union does not discriminate flavors at the type level either) and
//!   that callback-only merges need no flavor named. The tested flavor is the crate's confirmed
//!   `useRef`-as-mutable-box mapping, `StoredValue<Option<T>, LocalStorage>`
//!   (`specs/architecture.md`, the `useRef` row; the `use_ref_with_init` port).
//!   Two consequences: the JS reference share `ref.current = instance` becomes an owned store
//!   write, so the `StoredValue` impl clones the node handle (Leptos node handles are
//!   ref-counted clones); and a write after the box's owner is disposed is silently dropped
//!   (`try_set_value` returns the value back instead of writing) rather than panicking — the
//!   recorded dispose-panic deviation applies to boxes this crate itself owns (the
//!   `use_ref_with_init` port), while here the box is caller-owned input the merged contract
//!   must not be able to break (every covered upstream render asserts `not.toErrorDev()`,
//!   `packages/utils/src/useMergedRefs.test.tsx:26-28` and siblings).
//! - The callback-ref flavor [`RefCallback`] models the full protocol in one signature: invoked
//!   with `Some(&instance)` on attach and `None` on the legacy detach path; returning
//!   `Some(CleanupFn)` is the React-19 cleanup protocol (upstream's
//!   `typeof refCleanup === 'function'` capture, `:112-114`), `None` is the legacy shape whose
//!   detach is the `null` invocation (`:136-140`). [`CleanupFn`] is the crate's shared cleanup
//!   type (the `merge_cleanups` port).
//! - The arity overloads (`:22-29`) collapse into the two-argument [`use_merged_refs`] — the
//!   only test-proven arity (`packages/utils/src/useMergedRefs.test.tsx:19`, `:86`, `:151`) —
//!   plus the array form [`use_merged_refs_n`] mirroring `useMergedRefsN` (`:48-54`); three- and
//!   four-argument merge sites take the array form. The overloads are TS typing ergonomics.
//! - `didChange`/`didChangeN`/`update` re-running per render (`:36-38`, `:49-51`) dissolves:
//!   they exist because React re-runs the render body and must detect branch-identity changes
//!   between renders, while a Leptos component body runs once (`specs/architecture.md`, the
//!   quick-reference `useCallback` row) — the same dissolution as the `useRefWithInit` sentinel
//!   in the crate's hook ports (`use_interval.rs`, `use_idle_callback.rs`, `fast_hooks.rs`).
//!   The setter is therefore built exactly once from the branches given at call time, and
//!   upstream's "changing refs" behaviors (`packages/utils/src/useMergedRefs.test.tsx:91-124` —
//!   a branch added after mount picks the node up, a swapped branch detaches only itself while
//!   untouched branches are unaffected) have no counterpart at this hook: in the port those
//!   re-renders are signal-driven re-attach logic built at the use site, not re-executed hook
//!   bodies. What stays live is the setter's own attach/detach protocol across invocations —
//!   including re-attach on a second non-null invocation, which upstream's pending-cleanup guard
//!   (`:96-99`) also serves when the same callback ref is committed to a new element.
//! - The `useRefWithInit` fork box (`:36`, `:49`) dissolves into the returned closure itself:
//!   React needs the `ForkRef` struct (`:9-13`, `:56-62`) to persist across re-renders, which is
//!   why upstream reaches for `useRefWithInit` (ported first per the `utils: useRefWithInit`
//!   dependency note in `TODO.md`); the port's `'static` `Rc` closure is itself the persistent
//!   box, and the only mutable piece left — the pending-cleanup slot (`:11`, `:59`) — is an
//!   `Rc<RefCell<..>>` shared between the setter and the detach closure it stores. Nothing is
//!   owner-arena allocated, so the hook needs no reactive owner (upstream's hook-rules
//!   requirement has no runtime counterpart in a plain constructor); and it registers no
//!   `on_cleanup` — detach-on-unmount is the ref protocol's job in React (`commitDetachRef`
//!   invokes the old callback with `null`), which stays the invoking view layer's responsibility
//!   here, exactly as upstream leaves it to React itself.
//! - `Result<I> | null` (`:6`) becomes [`Option<MergedRefCallback<T>>`]: `None` when every
//!   branch is [`InputRef::Empty`] (`:90-93`, pinned by
//!   `packages/utils/src/useMergedRefs.test.tsx:57-75`).
//! - The stored detach closure needs its own branch handles (upstream's cleanup closure reads
//!   the same `refs` array, `:126-127`); the port clones the branch list — an `Rc`-handle clone
//!   is the same underlying object upstream shares by reference.
//! - `'use client'` (`:1`) is N/A — there is no React Server Components boundary in Rust.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, StoredValue};
use reactive_graph::traits::SetValue;

use crate::merge_cleanups::CleanupFn;

/// The object-ref flavor of [`InputRef`] — upstream's object `React.Ref` branch, the
/// `ref.current = instance` / `ref.current = null` writes
/// (`packages/utils/src/useMergedRefs.ts:117-119`, `:143-146`). Any type with a settable
/// `current` slot implements this; the default flavor is [`StoredValue`]`<Option<T>, LocalStorage>`
/// (the crate's `useRef`-as-mutable-box mapping). The `leptos-ui` layer can implement it for
/// `NodeRef<T>` — the trait is local to this crate, the handle type is foreign.
pub trait RefObject<T>: 'static {
    /// Points the ref's `current` slot at `current` (`Some`) or detaches it (`None`).
    /// Implementations must not panic on a disposed box — the write is silently dropped there.
    fn set_current(&self, current: Option<&T>);
}

impl<T: Clone + 'static> RefObject<T> for StoredValue<Option<T>, LocalStorage> {
    fn set_current(&self, current: Option<&T>) {
        // The JS reference share becomes an owned store write (a clone of the node handle);
        // after owner disposal the write is silently dropped — see the module docs.
        let _ = self.try_set_value(current.cloned());
    }
}

/// The callback-ref flavor of [`InputRef`] — upstream's `React.RefCallback<I>` branch with the
/// full attach/detach protocol: invoked with `Some(&instance)` on attach and `None` on the
/// legacy detach path (`packages/utils/src/useMergedRefs.ts:136-140`); returning
/// [`Some`](Some)`([`CleanupFn`])` captures a React-19-style attach-time cleanup
/// (`packages/utils/src/useMergedRefs.ts:112-114`), returning `None` is the legacy no-cleanup
/// shape.
pub type RefCallback<T> = Rc<dyn Fn(Option<&T>) -> Option<CleanupFn>>;

/// The merged ref-setter — upstream's `Result<I> = React.RefCallback<I> | null`
/// (`packages/utils/src/useMergedRefs.ts:6`), the function consumers assign to a host element's
/// ref. `use_merged_refs`/`use_merged_refs_n` return `None` instead of a null setter when every
/// branch is nullish (`packages/utils/src/useMergedRefs.ts:90-93`).
pub type MergedRefCallback<T> = Rc<dyn Fn(Option<&T>)>;

/// One ref branch — the port of upstream's `InputRef<I>` union, `React.Ref<I> | null | undefined`
/// (`packages/utils/src/useMergedRefs.ts:4-5`). `Empty` is the tolerated-silently nullish branch
/// (`packages/utils/src/useMergedRefs.ts:106-108`).
pub enum InputRef<T> {
    /// An object ref — a handle implementing [`RefObject`] (a `current` slot), trait-objected so
    /// flavors mix within one merge (see the module docs).
    Object(Rc<dyn RefObject<T>>),
    /// A callback ref implementing the [`RefCallback`] protocol.
    Callback(RefCallback<T>),
    /// A nullish branch: upstream's `null`/`undefined` argument, skipped silently
    /// (`packages/utils/src/useMergedRefs.ts:106-108`).
    Empty,
}

impl<T> Clone for InputRef<T> {
    fn clone(&self) -> Self {
        match self {
            InputRef::Object(reference) => InputRef::Object(Rc::clone(reference)),
            InputRef::Callback(callback) => InputRef::Callback(Rc::clone(callback)),
            InputRef::Empty => InputRef::Empty,
        }
    }
}

/// Merges two refs into a single ref-setter — the port of upstream `useMergedRefs(a, b)`
/// (`packages/utils/src/useMergedRefs.ts:30-41`), the only test-proven arity
/// (`packages/utils/src/useMergedRefs.test.tsx:19`, `:86`, `:151`). Returns `None` when both
/// branches are [`InputRef::Empty`] (`packages/utils/src/useMergedRefs.ts:90-93`).
pub fn use_merged_refs<T>(a: InputRef<T>, b: InputRef<T>) -> Option<MergedRefCallback<T>>
where
    T: 'static,
{
    use_merged_refs_n([a, b])
}

/// Merges any number of refs into a single ref-setter — the port of upstream
/// `useMergedRefsN(refs)` (`packages/utils/src/useMergedRefs.ts:48-54`); three- and four-branch
/// merge sites take this form (upstream's fixed-arity overloads, `:22-29`, are TS typing
/// ergonomics).
pub fn use_merged_refs_n<T, I>(refs: I) -> Option<MergedRefCallback<T>>
where
    T: 'static,
    I: IntoIterator<Item = InputRef<T>>,
{
    let refs: Vec<InputRef<T>> = refs.into_iter().collect();

    // Every branch nullish -> the setter itself is null
    // (`packages/utils/src/useMergedRefs.ts:90-93`).
    if refs.iter().all(|branch| matches!(branch, InputRef::Empty)) {
        return None;
    }

    // The fork box's only mutable residue — the pending-cleanup slot — shared between the
    // setter and the detach closure it stores (`packages/utils/src/useMergedRefs.ts:11`, `:59`,
    // `:96-99`); see the module docs for the `useRefWithInit` dissolution.
    let pending_cleanup: Rc<RefCell<Option<CleanupFn>>> = Rc::new(RefCell::new(None));

    Some(Rc::new(move |instance: Option<&T>| {
        // Run and drop the previous detach cleanup before anything else
        // (`packages/utils/src/useMergedRefs.ts:96-99`).
        if let Some(cleanup) = pending_cleanup.borrow_mut().take() {
            cleanup();
        }

        // A null invocation runs only the pending cleanup and never touches the branches
        // (`packages/utils/src/useMergedRefs.ts:101`).
        let Some(instance) = instance else {
            return;
        };

        // Fork the instance to every live branch, capturing the attach-time cleanups callback
        // branches return (`packages/utils/src/useMergedRefs.ts:102-123`; upstream allocates
        // the array filled with nulls, so object and empty branches occupy their slot too).
        let mut branch_cleanups: Vec<Option<CleanupFn>> = Vec::with_capacity(refs.len());
        for branch in &refs {
            match branch {
                InputRef::Object(reference) => {
                    reference.set_current(Some(instance));
                    branch_cleanups.push(None);
                }
                InputRef::Callback(callback) => branch_cleanups.push(callback(Some(instance))),
                InputRef::Empty => branch_cleanups.push(None),
            }
        }

        // Store the detach cleanup: per branch, the captured attach-time cleanup if there was
        // one, else the legacy null invocation, else the object-null write
        // (`packages/utils/src/useMergedRefs.ts:125-150`). The branch list is cloned into the
        // closure (the `Rc`-handle clone of the module docs).
        let detach_refs = refs.clone();
        *pending_cleanup.borrow_mut() = Some(Box::new(move || {
            for (branch, branch_cleanup) in detach_refs.iter().zip(branch_cleanups) {
                match branch {
                    InputRef::Object(reference) => reference.set_current(None),
                    InputRef::Callback(callback) => match branch_cleanup {
                        Some(cleanup) => cleanup(),
                        None => {
                            callback(None);
                        }
                    },
                    InputRef::Empty => {}
                }
            }
        }));
    }))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use reactive_graph::owner::{Owner, StoredValue};
    use reactive_graph::traits::GetValue;

    use super::*;

    #[derive(Clone)]
    struct TestNode {
        id: &'static str,
    }

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    /// An object branch standing in for `React.createRef()` — a `current` box the test can
    /// read afterwards (`packages/utils/src/useMergedRefs.test.tsx:24`, `:98-108`).
    type ObjectBranch = StoredValue<Option<TestNode>, LocalStorage>;

    fn object_branch() -> ObjectBranch {
        StoredValue::new_local(None)
    }

    fn current(branch: &ObjectBranch) -> Option<&'static str> {
        branch.try_get_value().flatten().map(|node| node.id)
    }

    /// A callback branch recording its invocations like a `vi.fn()` mock
    /// (`packages/utils/src/useMergedRefs.test.tsx:128-131`), returning no cleanup — the legacy
    /// ref shape (`packages/utils/src/useMergedRefs.test.tsx:142-148`).
    fn recording_callback(log: Rc<RefCell<Vec<Option<&'static str>>>>) -> RefCallback<TestNode> {
        Rc::new(move |instance: Option<&TestNode>| {
            log.borrow_mut().push(instance.map(|node| node.id));
            None
        })
    }

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:10-30` — merging an object branch (the
    // `createRef` outer ref) with a callback branch (the `useState` setter): a single attach
    // invocation forks the node to both branches and no cleanup runs at attach
    // (`packages/utils/src/useMergedRefs.test.tsx:159`).
    #[test]
    fn forks_the_node_to_every_live_branch_on_attach() {
        let _owner = in_owner();

        let outer_ref = object_branch();
        let own_ref_log = Rc::new(RefCell::new(Vec::<Option<&'static str>>::new()));
        let handle = use_merged_refs(
            InputRef::Object(Rc::new(outer_ref)),
            InputRef::Callback(recording_callback(Rc::clone(&own_ref_log))),
        )
        .expect("a live branch exists, so the setter is not null");

        handle(Some(&TestNode { id: "test" }));

        assert_eq!(
            current(&outer_ref),
            Some("test"),
            "the object branch holds the node"
        );
        assert_eq!(
            *own_ref_log.borrow(),
            [Some("test")],
            "the callback branch was invoked exactly once with the node"
        );
    }

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:32-50` — one live branch next to a
    // nullish one (the forwarded ref never populated by the caller): the live branch still
    // fires and the empty branch is tolerated silently.
    #[test]
    fn forks_when_only_one_branch_requires_a_ref() {
        let _owner = in_owner();

        let log = Rc::new(RefCell::new(Vec::<Option<&'static str>>::new()));
        let handle = use_merged_refs(
            InputRef::Callback(recording_callback(Rc::clone(&log))),
            InputRef::Empty,
        )
        .expect("one live branch exists");

        handle(Some(&TestNode { id: "test" }));

        assert_eq!(*log.borrow(), [Some("test")]);
    }

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:52-75` — no branch requires a ref
    // (a child with no ref merged with a null forwarded ref): the setter itself is null
    // (`packages/utils/src/useMergedRefs.ts:90-93`) and constructing the merge is error-free.
    #[test]
    fn returns_a_null_setter_when_no_branch_requires_a_ref() {
        let _owner = in_owner();

        let handle = use_merged_refs(InputRef::<TestNode>::Empty, InputRef::Empty);

        assert!(handle.is_none(), "an all-nullish merge has a null setter");
    }

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:127-173` — the cleanup protocol. Two
    // callback branches: one returning an attach-time cleanup (React 19 shape, `:133-140`), one
    // legacy returning none (`:142-148`). Attach invokes each exactly once with the node and no
    // cleanup runs; detach runs the captured cleanup exactly once, re-invokes only the legacy
    // branch with `null`, and never re-invokes the cleanup-returning branch.
    #[test]
    fn runs_attach_time_cleanups_instead_of_the_null_invocation_on_detach() {
        let _owner = in_owner();

        let setup_with_cleanup = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let setup_without_cleanup = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let null_handler = Rc::new(Cell::new(0usize));
        let cleanup_count = Rc::new(Cell::new(0usize));

        let with_cleanup: RefCallback<TestNode> = {
            let log = Rc::clone(&setup_with_cleanup);
            let nulls = Rc::clone(&null_handler);
            let cleanups = Rc::clone(&cleanup_count);
            Rc::new(move |instance: Option<&TestNode>| {
                match instance {
                    Some(node) => log.borrow_mut().push(node.id),
                    None => nulls.set(nulls.get() + 1),
                }
                let cleanups = Rc::clone(&cleanups);
                Some(Box::new(move || cleanups.set(cleanups.get() + 1)))
            })
        };
        let without_cleanup: RefCallback<TestNode> = {
            let log = Rc::clone(&setup_without_cleanup);
            let nulls = Rc::clone(&null_handler);
            Rc::new(move |instance: Option<&TestNode>| {
                match instance {
                    Some(node) => log.borrow_mut().push(node.id),
                    None => nulls.set(nulls.get() + 1),
                }
                None
            })
        };

        let handle = use_merged_refs(
            InputRef::Callback(with_cleanup),
            InputRef::Callback(without_cleanup),
        )
        .unwrap();

        handle(Some(&TestNode { id: "test" }));

        assert_eq!(
            *setup_with_cleanup.borrow(),
            ["test"],
            "the cleanup-returning branch attached exactly once"
        );
        assert_eq!(
            cleanup_count.get(),
            0,
            "no attach-time cleanup runs at attach (useMergedRefs.test.tsx:159)"
        );
        assert_eq!(
            *setup_without_cleanup.borrow(),
            ["test"],
            "the legacy branch attached exactly once"
        );

        handle(None);

        assert_eq!(
            setup_with_cleanup.borrow().len(),
            1,
            "the cleanup-returning branch is not re-invoked with null (:166)"
        );
        assert_eq!(
            cleanup_count.get(),
            1,
            "its attach-time cleanup ran exactly once (:167)"
        );
        assert_eq!(
            setup_without_cleanup.borrow().len(),
            1,
            "the legacy branch was not set up again (:170)"
        );
        assert_eq!(
            null_handler.get(),
            1,
            "the legacy branch took the null-detach path exactly once (:172)"
        );
    }

    // Mirrors the invocation-level content of `packages/utils/src/useMergedRefs.test.tsx:91-124`
    // — the setter's lifecycle across elements: object branches hold the node on attach, are
    // all nulled by a detach invocation, and hold the next node on re-attach; a branch that was
    // never attached stays null (`:117`).
    #[test]
    fn object_branches_detach_to_null_and_reattach_across_invocations() {
        let _owner = in_owner();

        let first = object_branch();
        let second = object_branch();
        let never_used = object_branch();
        let handle = use_merged_refs(
            InputRef::Object(Rc::new(first)),
            InputRef::Object(Rc::new(second)),
        )
        .unwrap();

        handle(Some(&TestNode {
            id: "first-element",
        }));
        assert_eq!(current(&first), Some("first-element"));
        assert_eq!(current(&second), Some("first-element"));
        assert_eq!(
            current(&never_used),
            None,
            "an unattached branch stays null"
        );

        handle(None);
        assert_eq!(
            current(&first),
            None,
            "detach nulls the object branch (:143-146)"
        );
        assert_eq!(current(&second), None, "detach nulls every object branch");

        handle(Some(&TestNode {
            id: "second-element",
        }));
        assert_eq!(current(&first), Some("second-element"));
        assert_eq!(current(&second), Some("second-element"));
    }

    // Pins the pending-cleanup guard on re-attach
    // (`packages/utils/src/useMergedRefs.ts:96-99`; source-derived — no upstream test attaches
    // twice without a detach): a second attach invocation runs the first attach's cleanup
    // before forking the new node, and each cleanup runs exactly once overall.
    #[test]
    fn a_new_attach_runs_the_pending_cleanup_exactly_once_before_forking() {
        let _owner = in_owner();

        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
        let cleanup_count = Rc::new(Cell::new(0usize));
        let callback: RefCallback<TestNode> = {
            let log = Rc::clone(&log);
            let cleanups = Rc::clone(&cleanup_count);
            Rc::new(move |instance: Option<&TestNode>| {
                if let Some(node) = instance {
                    log.borrow_mut().push(node.id);
                }
                let cleanups = Rc::clone(&cleanups);
                let log = Rc::clone(&log);
                Some(Box::new(move || {
                    cleanups.set(cleanups.get() + 1);
                    log.borrow_mut().push("cleanup");
                }))
            })
        };
        let handle = use_merged_refs(InputRef::Callback(callback), InputRef::Empty).unwrap();

        handle(Some(&TestNode { id: "first" }));
        handle(Some(&TestNode { id: "second" }));
        handle(None);

        assert_eq!(
            *log.borrow(),
            ["first", "cleanup", "second", "cleanup"],
            "the first attach's cleanup ran before the second attach's fork"
        );
        assert_eq!(
            cleanup_count.get(),
            2,
            "each attach's cleanup ran exactly once"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{GetValue, WithValue};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Clone)]
    struct TestNode {
        id: &'static str,
    }

    type ObjectBranch = StoredValue<Option<TestNode>, LocalStorage>;

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:10-30` in a real realm: the merged
    // setter forks the node to an object branch and a callback branch on attach.
    #[wasm_bindgen_test]
    fn forks_the_node_to_every_live_branch_on_attach() {
        let owner = Owner::new();
        owner.set();

        let outer_ref: ObjectBranch = StoredValue::new_local(None);
        let log = Rc::new(std::cell::RefCell::new(Vec::<&'static str>::new()));
        let callback_log = Rc::clone(&log);
        let handle = use_merged_refs(
            InputRef::Object(Rc::new(outer_ref)),
            InputRef::Callback(Rc::new(
                move |instance: Option<&TestNode>| -> Option<CleanupFn> {
                    if let Some(node) = instance {
                        callback_log.borrow_mut().push(node.id);
                    }
                    None
                },
            )),
        )
        .unwrap();

        handle(Some(&TestNode { id: "test" }));

        assert_eq!(
            outer_ref.with_value(|slot| slot.as_ref().map(|node| node.id)),
            Some("test")
        );
        assert_eq!(*log.borrow(), ["test"]);

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:32-50` in a real realm: a single live
    // branch among nullish ones still fires.
    #[wasm_bindgen_test]
    fn forks_when_only_one_branch_requires_a_ref() {
        let owner = Owner::new();
        owner.set();

        let count = Rc::new(Cell::new(0usize));
        let counter = Rc::clone(&count);
        let handle = use_merged_refs(
            InputRef::Callback(Rc::new(
                move |instance: Option<&TestNode>| -> Option<CleanupFn> {
                    if instance.is_some() {
                        counter.set(counter.get() + 1);
                    }
                    None
                },
            )),
            InputRef::Empty,
        )
        .unwrap();

        handle(Some(&TestNode { id: "test" }));
        assert_eq!(count.get(), 1, "the lone live branch fired");

        owner.cleanup();
    }

    // Mirrors `packages/utils/src/useMergedRefs.test.tsx:52-75` and `:127-173` in a real realm:
    // all-nullish merges have a null setter, and the detach protocol runs the captured
    // attach-time cleanup exactly once while the legacy branch takes the null path.
    #[wasm_bindgen_test]
    fn all_nullish_merges_are_null_and_detach_runs_the_cleanup_protocol() {
        let owner = Owner::new();
        owner.set();

        assert!(use_merged_refs(InputRef::<TestNode>::Empty, InputRef::Empty).is_none());

        let attach_log = Rc::new(std::cell::RefCell::new(Vec::<&'static str>::new()));
        let null_count = Rc::new(Cell::new(0usize));
        let cleanup_count = Rc::new(Cell::new(0usize));

        let with_cleanup: RefCallback<TestNode> = {
            let log = Rc::clone(&attach_log);
            let cleanups = Rc::clone(&cleanup_count);
            Rc::new(move |instance: Option<&TestNode>| {
                if let Some(node) = instance {
                    log.borrow_mut().push(node.id);
                }
                let cleanups = Rc::clone(&cleanups);
                Some(Box::new(move || cleanups.set(cleanups.get() + 1)))
            })
        };
        let without_cleanup: RefCallback<TestNode> = {
            let nulls = Rc::clone(&null_count);
            Rc::new(move |instance: Option<&TestNode>| {
                if instance.is_none() {
                    nulls.set(nulls.get() + 1);
                }
                None
            })
        };

        let handle = use_merged_refs(
            InputRef::Callback(with_cleanup),
            InputRef::Callback(without_cleanup),
        )
        .unwrap();

        handle(Some(&TestNode { id: "test" }));
        assert_eq!(*attach_log.borrow(), ["test"]);
        assert_eq!(cleanup_count.get(), 0, "no cleanup runs at attach");

        handle(None);
        assert_eq!(
            cleanup_count.get(),
            1,
            "the attach-time cleanup ran once on detach"
        );
        assert_eq!(
            null_count.get(),
            1,
            "the legacy branch took the null path once"
        );

        owner.cleanup();
    }

    // Mirrors the object-branch lifecycle (`packages/utils/src/useMergedRefs.test.tsx:105-124`)
    // in a real realm: detach nulls object branches, re-attach repopulates them.
    #[wasm_bindgen_test]
    fn object_branches_detach_to_null_and_reattach_across_invocations() {
        let owner = Owner::new();
        owner.set();

        let first: ObjectBranch = StoredValue::new_local(None);
        let second: ObjectBranch = StoredValue::new_local(None);
        let handle = use_merged_refs(
            InputRef::Object(Rc::new(first)),
            InputRef::Object(Rc::new(second)),
        )
        .unwrap();

        handle(Some(&TestNode { id: "a" }));
        assert_eq!(
            first.with_value(|slot| slot.as_ref().map(|n| n.id)),
            Some("a")
        );
        assert_eq!(
            second.with_value(|slot| slot.as_ref().map(|n| n.id)),
            Some("a")
        );

        handle(None);
        assert_eq!(first.with_value(|slot| slot.as_ref().map(|n| n.id)), None);
        assert_eq!(second.with_value(|slot| slot.as_ref().map(|n| n.id)), None);

        handle(Some(&TestNode { id: "b" }));
        assert_eq!(
            first.with_value(|slot| slot.as_ref().map(|n| n.id)),
            Some("b")
        );
        assert_eq!(
            second.with_value(|slot| slot.as_ref().map(|n| n.id)),
            Some("b")
        );

        owner.cleanup();
    }
}
