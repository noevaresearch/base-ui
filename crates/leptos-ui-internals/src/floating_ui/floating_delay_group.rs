//! Port of `packages/react/src/floating-ui-react/components/FloatingDelayGroup.tsx` —
//! the shared-delay group provider and its `useDelayGroup` consumer
//! (`specs/library/floating-ui-react/behavior.md`, "Public API surface": "`FloatingDelayGroup`
//! (`delay`, `timeoutMs`) + `useDelayGroup` (`delayRef`, `isInstantPhase`)";
//! "State model": "the group only mediates delays (instant member-to-member switch,
//! `timeoutMs` re-open window, `isInstantPhase` marker, unmount-safe group timeout)").
//!
//! ## Upstream shape being ported
//!
//! - `ContextValue` (`FloatingDelayGroup.tsx:14-25`) — the shared group state: `hasProvider`,
//!   `timeoutMs`, `delayRef`/`initialDelayRef` (the live and seed `Delay` values), the shared
//!   `Timeout`, `currentIdRef` (the owning member's floating id), and `currentContextRef`
//!   (`{ onOpenChange, setIsInstantPhase }` — the owning member's callback pair). All the
//!   mutable members are refs, so membership changes never re-render other members
//!   (implementation.md, "Notable per-hook state machines" → "DelayGroup": "All shared group
//!   state lives in refs inside context ... so membership changes never re-render other
//!   members; only `isInstantPhase` is component-local React state", `:161`).
//! - The module-level `createContext` default (`:27-35`) — `hasProvider: false` with inert
//!   zero-delay refs and an unowned `Timeout`.
//! - `FloatingDelayGroup` (`:66-107`) — creates the refs and the group `Timeout` (`:69-73`),
//!   re-syncs `delayRef` on `delay` changes (`:75-87`: the seed ref always follows the prop,
//!   while an active membership keeps its zero open delay and only adopts the new close
//!   delay), and provides the context.
//! - `useDelayGroup(context, { open })` (`:141-281`) — the `openRef` mirror (`:162-166`)
//!   plus three layout effects:
//!   - the close/reset effect (`:168-217`): when this member closes while it owns the group,
//!     the instant phase clears immediately (`:182`), and `timeoutMs` schedules the group
//!     reset — skipped if another member took over or the member re-opened (`:186-193`, the
//!     race guard) — with the re-run cleanup clearing a superseded window unless the member
//!     re-opened or another took over while it was closed (`:196-200`); without `timeoutMs`
//!     the reset is immediate (`:203`);
//!   - the ownership effect (`:219-254`): on open, cancel any pending reset (`:229`),
//!     publish this member's `{ onOpenChange: store.setOpen, setIsInstantPhase }` pair
//!     (`:230`), zero the open delay (`:232-235`), and — when a different member was active —
//!     flip both members' instant phases and force-close the previous owner with
//!     `REASONS.none` (`:237-244`);
//!   - the unmount effect (`:256-270`): a member unmounting while it owns the group clears
//!     the context pair, and — only if it is still open — the ownership and delay state plus
//!     any pending reset; a closed owner unmounting preserves the `timeoutMs` window
//!     (`:261-263`).
//!
//! ## Rust adaptations
//!
//! - `FloatingDelayGroup` is a React provider component; this crate is view-free (the
//!   `csp_provider`/`direction_provider` realm convention), so it ports to
//!   [`provide_floating_delay_group`] — the same provider-body work (refs, `useTimeout`, the
//!   sync layout effect, `provide_context`) without the view layer. The context travels
//!   through `provide_context` behind [`SharedFloatingDelayGroupContext`], the crate's
//!   `SendWrapper` bridge for non-thread-safe handles (the `SharedFloatingTreeStore`
//!   precedent).
//! - Upstream's module-level context default (`:27-35`) is one shared object every
//!   provider-less consumer receives; the port mirrors that with a thread-local default
//!   (a Rust module has no import-time initializer), so provider-less members still share
//!   one group state the way upstream's do.
//! - The `React.RefObject` members port to `Rc<RefCell<_>>` handles shared by reference;
//!   clones of the group context (and of `Timeout`) share the same slots.
//! - The `delay`/`timeoutMs` props port to resolver closures (`impl Fn() -> Delay` /
//!   `impl Fn() -> u32`): a Leptos component body runs once, so a static prop is `|| value`
//!   and a reactive one reads its signal — the `provide_direction_context` Get-source
//!   convention in plain-closure form. The provider's sync effect calls the resolver inside
//!   the effect body, so reactive reads drive the re-runs that upstream's `[delay, ...]`
//!   dependency array scheduled (`:87`).
//! - `isInstantPhase` (`:161`) ports to a local [`RwSignal`]; the setter shared through
//!   `currentContextRef` is the signal setter behind an `Rc<dyn Fn(bool)>`.
//! - `currentContextRef`'s `onOpenChange: store.setOpen` (`:230`) stores a closure over the
//!   member's [`FloatingRootStore`] handle — the bound method's identity is not observable
//!   here, and the handle keeps the store alive as long as the group context does.
//! - `createChangeEventDetails(REASONS.none)` (`:240`) has no native event argument; the
//!   ported details type carries one by value, so the factory's own omitted-argument default
//!   (`event ?? new Event('base-ui')`,
//!   `packages/react/src/internals/createBaseUIEventDetails.ts:131`) is supplied.
//! - The layout effects port to [`use_iso_layout_effect`] (the `RenderEffect` decision,
//!   `specs/architecture.md`, "Layout effect"): reactive reads replace the dependency
//!   arrays, and the two effects' returned cleanups (`:196-200`, `:257-269`) port to
//!   `on_cleanup` registrations inside the effect bodies (per-run cleanups that also fire on
//!   disposal) over `SendWrapper` bridges for the non-`Send` captures.
//! - `timeoutMs` truthiness (`if (timeoutMs)`, `:184`) ports to `!= 0` — the default is `0`
//!   (`:67`), and upstream's test suite only ever passes non-negative integers.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, on_cleanup, provide_context, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, Set};
use send_wrapper::SendWrapper;
use web_sys::Event;

use leptos_ui_utils::use_timeout::Timeout;
use leptos_ui_utils::{use_iso_layout_effect, use_timeout};

use crate::floating_ui::element_props::FloatingContextSource;
use crate::floating_ui::floating_root_store::FloatingRootStore;
use crate::floating_ui::floating_root_store::selectors;
use crate::floating_ui::reasons;
use crate::floating_ui::types::{Delay, OnOpenChangeFn, RootOpenChangeEventDetails};
use crate::floating_ui::use_hover_shared::{DelayInput, get_delay};

/// The `currentContextRef` payload (`FloatingDelayGroup.tsx:21-24`): the owning member's
/// open-change callback and its local `isInstantPhase` setter.
#[derive(Clone)]
pub struct DelayGroupMemberContext {
    /// `onOpenChange` — upstream stores `store.setOpen` (`:230`); the port wraps the
    /// member's store handle (see the module docs).
    pub on_open_change: OnOpenChangeFn,
    /// `setIsInstantPhase` (`:23`) — the member's local `isInstantPhase` setter.
    pub set_is_instant_phase: Rc<dyn Fn(bool)>,
}

/// Port of `ContextValue` (`FloatingDelayGroup.tsx:14-25`) — the shared group state.
pub struct FloatingDelayGroupContext {
    /// `hasProvider` (`:15`).
    pub has_provider: bool,
    /// `timeoutMs` (`:16`) — read fresh by the members' close effect each run (upstream
    /// re-memos the context value on `timeoutMs` changes, `:101`).
    pub timeout_ms: Rc<dyn Fn() -> u32>,
    /// `delayRef` (`:17`) — the live delay the members read per event.
    pub delay_ref: Rc<RefCell<Delay>>,
    /// `initialDelayRef` (`:18`) — the seed delay resets fall back to.
    pub initial_delay_ref: Rc<RefCell<Delay>>,
    /// `timeout` (`:19`) — the group's single reset window.
    pub timeout: Timeout,
    /// `currentIdRef` (`:20`) — the owning member's floating id.
    pub current_id_ref: Rc<RefCell<Option<String>>>,
    /// `currentContextRef` (`:21-24`) — the owning member's callback pair.
    pub current_context_ref: Rc<RefCell<Option<DelayGroupMemberContext>>>,
}

/// The context handle — the `Rc` group state rides behind a `SendWrapper` because
/// `provide_context` requires `Send + Sync` (`reactive_graph`'s context contract); the same
/// bridge the `SharedFloatingTreeStore` handle uses.
pub type SharedFloatingDelayGroupContext = SendWrapper<Rc<FloatingDelayGroupContext>>;

thread_local! {
    /// The module-level context default (`FloatingDelayGroup.tsx:27-35`): upstream's
    /// `createContext({...})` default is one shared object every provider-less consumer
    /// receives, so the port's default is one shared thread-local instance — provider-less
    /// members still share one group state the way upstream's do. Its `Timeout` is unowned
    /// (never disposed), like upstream's module-scope `new Timeout()`.
    static DEFAULT_CONTEXT: SharedFloatingDelayGroupContext = {
        SharedFloatingDelayGroupContext::new(Rc::new(FloatingDelayGroupContext {
            has_provider: false,
            timeout_ms: Rc::new(|| 0),
            delay_ref: Rc::new(RefCell::new(Delay::Value(0))),
            initial_delay_ref: Rc::new(RefCell::new(Delay::Value(0))),
            timeout: Timeout::create(),
            current_id_ref: Rc::new(RefCell::new(None)),
            current_context_ref: Rc::new(RefCell::new(None)),
        }))
    };
}

/// `resetDelayRef` (`FloatingDelayGroup.tsx:37-39`): restore the live delay from the seed.
fn reset_delay_ref(delay_ref: &Rc<RefCell<Delay>>, initial_delay_ref: &Rc<RefCell<Delay>>) {
    *delay_ref.borrow_mut() = initial_delay_ref.borrow().clone();
}

/// Publishes the group for the current reactive owner's subtree — the body of the
/// `FloatingDelayGroup` provider component (`FloatingDelayGroup.tsx:66-107`) without the
/// view layer (see the module docs). Returns the now-ambient group context.
///
/// The resolvers are read by the provider's sync layout effect (`:75-87`): a static prop is
/// `|| Delay::Partial { .. }` / `|| 0`; a reactive prop reads its signal inside the closure.
pub fn provide_floating_delay_group(
    delay: impl Fn() -> Delay + 'static,
    timeout_ms: impl Fn() -> u32 + 'static,
) -> SharedFloatingDelayGroupContext {
    // `:69-73` — the refs plus the group window; `useTimeout` registers the unmount-safe
    // disposal (the "unmount-safe group timeout", behavior.md "State model").
    let delay_ref: Rc<RefCell<Delay>> = Rc::new(RefCell::new(delay()));
    let initial_delay_ref: Rc<RefCell<Delay>> = Rc::new(RefCell::new(delay_ref.borrow().clone()));
    let current_id_ref: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let current_context_ref: Rc<RefCell<Option<DelayGroupMemberContext>>> =
        Rc::new(RefCell::new(None));
    let timeout = use_timeout();

    // The delay re-sync (`:75-87`): the seed ref always follows the prop; an active
    // membership keeps its zero open delay and only adopts the new close delay.
    {
        let delay_ref = Rc::clone(&delay_ref);
        let initial_delay_ref = Rc::clone(&initial_delay_ref);
        let current_id_ref = Rc::clone(&current_id_ref);
        use_iso_layout_effect(move || {
            let delay = delay();
            *initial_delay_ref.borrow_mut() = delay.clone();

            if current_id_ref.borrow().is_none() {
                *delay_ref.borrow_mut() = delay;
                return;
            }

            let open = get_delay(&DelayInput::Value(delay_ref.borrow().clone()), "open", None);
            let close = get_delay(&DelayInput::Value(delay), "close", None);
            *delay_ref.borrow_mut() = Delay::Partial {
                open: Some(open),
                close: Some(close),
            };
        });
    }

    let context = Rc::new(FloatingDelayGroupContext {
        has_provider: true,
        timeout_ms: Rc::new(timeout_ms),
        delay_ref,
        initial_delay_ref,
        timeout,
        current_id_ref,
        current_context_ref,
    });
    provide_context(SharedFloatingDelayGroupContext::new(Rc::clone(&context)));
    SharedFloatingDelayGroupContext::new(context)
}

/// The ambient group context: the nearest [`provide_floating_delay_group`] value, or the
/// shared module default (`FloatingDelayGroup.tsx:27-35`).
fn use_delay_group_context() -> SharedFloatingDelayGroupContext {
    use_context::<SharedFloatingDelayGroupContext>()
        .unwrap_or_else(|| DEFAULT_CONTEXT.with(|default| default.clone()))
}

/// Port of `UseDelayGroupOptions` (`FloatingDelayGroup.tsx:109-114`).
pub struct UseDelayGroupOptions<O> {
    /// `open` (`:112`) — whether the trigger this hook is used in has opened the popup.
    pub open: O,
}

impl<O> UseDelayGroupOptions<O> {
    /// Seeds the options — upstream's `{ open }` argument (`:143`).
    pub fn new(open: O) -> Self {
        Self { open }
    }
}

/// Port of `UseDelayGroupReturn` (`FloatingDelayGroup.tsx:116-133`).
pub struct UseDelayGroupReturn {
    /// `activeIdRef` (`:120-121`) — the floating element keeping the group active.
    pub active_id_ref: Rc<RefCell<Option<String>>>,
    /// `hasProvider` (`:131`) — whether a `FloatingDelayGroup` provider is present.
    pub has_provider: bool,
    /// `delayRef` (`:123-124`) — the group's live delay.
    pub delay_ref: Rc<RefCell<Delay>>,
    /// `isInstantPhase` (`:126-127`) — whether animations should be removed.
    pub is_instant_phase: RwSignal<bool, LocalStorage>,
}

/// Port of `useDelayGroup(context, { open })` (`FloatingDelayGroup.tsx:141-281`). Must be
/// called inside a reactive owner (a component).
pub fn use_delay_group<O>(
    context: impl Into<FloatingContextSource>,
    options: UseDelayGroupOptions<O>,
) -> UseDelayGroupReturn
where
    O: Fn() -> bool + 'static,
{
    let UseDelayGroupOptions { open } = options;

    // `'rootStore' in context ? context.rootStore : context` (`:147`).
    let store: Rc<FloatingRootStore> = context.into().root_store();
    // `store.useState('floatingId')` (`:148`).
    let floating_id = store.rc().use_state(selectors::floating_id);

    let group = use_delay_group_context();
    let context = Rc::clone(&*group);
    let has_provider = context.has_provider;
    let timeout_ms = Rc::clone(&context.timeout_ms);
    let delay_ref = Rc::clone(&context.delay_ref);
    let initial_delay_ref = Rc::clone(&context.initial_delay_ref);
    let current_id_ref = Rc::clone(&context.current_id_ref);
    let current_context_ref = Rc::clone(&context.current_context_ref);
    let timeout = context.timeout.clone();

    // `const [isInstantPhase, setIsInstantPhase] = React.useState(false)` (`:161`).
    let is_instant_phase = RwSignal::new_local(false);
    // `const openRef = React.useRef(open)` seeded + mirrored (`:162-166`).
    let open_ref = Rc::new(Cell::new(open()));
    // The `open` resolver rides an `Rc` so every effect gets its own handle.
    let open: Rc<dyn Fn() -> bool> = Rc::new(open);

    // The `openRef` mirror (`:164-166`).
    {
        let open_ref = Rc::clone(&open_ref);
        let open = Rc::clone(&open);
        use_iso_layout_effect(move || {
            open_ref.set(open());
        });
    }

    // The close/reset effect (`:168-217`).
    {
        let store = Rc::clone(&store);
        let open = Rc::clone(&open);
        let floating_id = floating_id.clone();
        let current_id_ref = Rc::clone(&current_id_ref);
        let current_context_ref = Rc::clone(&current_context_ref);
        let delay_ref = Rc::clone(&delay_ref);
        let initial_delay_ref = Rc::clone(&initial_delay_ref);
        let timeout_ms = Rc::clone(&timeout_ms);
        let timeout = timeout.clone();
        let is_instant_phase = is_instant_phase.clone();
        let open_ref_for_cleanup = Rc::clone(&open_ref);
        use_iso_layout_effect(move || {
            // Tracked reads — upstream's `[open, floatingId, ..., timeoutMs, ...]` deps
            // (`:207-217`); the ref members are stable and read untracked.
            let open_now = open();
            let floating_id_now = floating_id.get();
            let timeout_ms_now = timeout_ms();

            let unset = {
                let current_context_ref = Rc::clone(&current_context_ref);
                let current_id_ref = Rc::clone(&current_id_ref);
                let delay_ref = Rc::clone(&delay_ref);
                let initial_delay_ref = Rc::clone(&initial_delay_ref);
                let timeout = timeout.clone();
                move || {
                    if let Some(current) = current_context_ref.borrow_mut().take() {
                        (current.set_is_instant_phase)(false);
                    }
                    *current_id_ref.borrow_mut() = None;
                    reset_delay_ref(&delay_ref, &initial_delay_ref);
                    timeout.clear();
                }
            };

            let current_id = current_id_ref.borrow().clone();
            if current_id.is_none() {
                return;
            }

            if !open_now && current_id == floating_id_now {
                is_instant_phase.set(false);

                if timeout_ms_now != 0 {
                    let closing_id = floating_id_now.clone();
                    {
                        let current_id_ref = Rc::clone(&current_id_ref);
                        let store = Rc::clone(&store);
                        let unset = unset.clone();
                        let closing_id = closing_id.clone();
                        timeout.start(timeout_ms_now, move || {
                            // If another tooltip has taken over the group, skip resetting
                            // (`:187-193`) — or this member re-opened without the effect
                            // re-running first.
                            if store.select(selectors::open)
                                || current_id_ref
                                    .borrow()
                                    .clone()
                                    .is_some_and(|current| Some(current) != closing_id)
                            {
                                return;
                            }
                            unset();
                        });
                    }
                    // The re-run cleanup (`:196-200`): clear a superseded window unless the
                    // member re-opened or another member took over while it was closed.
                    let cleanup = {
                        let open_ref = Rc::clone(&open_ref_for_cleanup);
                        let current_id_ref = Rc::clone(&current_id_ref);
                        let timeout = timeout.clone();
                        move || {
                            let superseded = current_id_ref
                                .borrow()
                                .clone()
                                .is_some_and(|current| Some(current) != closing_id);
                            if open_ref.get() || superseded {
                                timeout.clear();
                            }
                        }
                    };
                    let cleanup = SendWrapper::new(cleanup);
                    on_cleanup(move || (*cleanup)());
                } else {
                    unset();
                }
            }
        });
    }

    // The ownership effect (`:219-254`).
    {
        let store = Rc::clone(&store);
        let open = Rc::clone(&open);
        let floating_id = floating_id.clone();
        let current_id_ref = Rc::clone(&current_id_ref);
        let current_context_ref = Rc::clone(&current_context_ref);
        let delay_ref = Rc::clone(&delay_ref);
        let initial_delay_ref = Rc::clone(&initial_delay_ref);
        let timeout = timeout.clone();
        let is_instant_phase = is_instant_phase.clone();
        use_iso_layout_effect(move || {
            let open_now = open();
            let floating_id_now = floating_id.get();

            if !open_now {
                return;
            }

            let prev_context = current_context_ref.borrow().clone();
            let prev_id = current_id_ref.borrow().clone();

            // A new tooltip is opening, so cancel any pending timeout that would reset the
            // group's delay back to the initial value (`:227-229`).
            timeout.clear();
            *current_context_ref.borrow_mut() = Some(DelayGroupMemberContext {
                on_open_change: {
                    let store = Rc::clone(&store);
                    Rc::new(move |open, details| store.set_open(open, details))
                },
                set_is_instant_phase: {
                    let is_instant_phase = is_instant_phase;
                    Rc::new(move |value| is_instant_phase.set(value))
                },
            });
            *current_id_ref.borrow_mut() = floating_id_now.clone();
            let close = get_delay(
                &DelayInput::Value(initial_delay_ref.borrow().clone()),
                "close",
                None,
            );
            *delay_ref.borrow_mut() = Delay::Partial {
                open: Some(0),
                close: Some(close),
            };

            if prev_id.is_some() && prev_id != floating_id_now {
                is_instant_phase.set(true);
                if let Some(prev) = prev_context {
                    (prev.set_is_instant_phase)(true);
                    (prev.on_open_change)(
                        false,
                        &RootOpenChangeEventDetails::new(
                            reasons::NONE,
                            // `createChangeEventDetails(REASONS.none)` (`:240`) passes no
                            // native event; the factory's own default stands in
                            // (createBaseUIEventDetails.ts:131).
                            Event::new("base-ui").unwrap(),
                            None,
                            String::new(),
                        ),
                    );
                }
            } else {
                is_instant_phase.set(false);
                if let Some(prev) = prev_context {
                    (prev.set_is_instant_phase)(false);
                }
            }
        });
    }

    // The unmount effect (`:256-270`).
    {
        let floating_id = floating_id.clone();
        let current_id_ref = Rc::clone(&current_id_ref);
        let current_context_ref = Rc::clone(&current_context_ref);
        let open_ref = Rc::clone(&open_ref);
        let delay_ref = Rc::clone(&delay_ref);
        let initial_delay_ref = Rc::clone(&initial_delay_ref);
        let timeout = timeout.clone();
        use_iso_layout_effect(move || {
            // `floatingId` is in the deps array (`:264`), so the registration re-runs if it
            // ever changes and the fresh id is what the cleanup compares against.
            let floating_id_now = floating_id.get();
            let cleanup = {
                let current_id_ref = Rc::clone(&current_id_ref);
                let current_context_ref = Rc::clone(&current_context_ref);
                let open_ref = Rc::clone(&open_ref);
                let delay_ref = Rc::clone(&delay_ref);
                let initial_delay_ref = Rc::clone(&initial_delay_ref);
                let timeout = timeout.clone();
                move || {
                    if current_id_ref.borrow().clone() == floating_id_now {
                        *current_context_ref.borrow_mut() = None;

                        // A closed owner unmounting preserves the group window; only an
                        // owner unmounted while still open resets the group state
                        // (`:261-267`).
                        if !open_ref.get() {
                            return;
                        }

                        *current_id_ref.borrow_mut() = None;
                        reset_delay_ref(&delay_ref, &initial_delay_ref);
                        timeout.clear();
                    }
                }
            };
            let cleanup = SendWrapper::new(cleanup);
            on_cleanup(move || (*cleanup)());
        });
    }

    UseDelayGroupReturn {
        active_id_ref: current_id_ref,
        has_provider,
        delay_ref,
        is_instant_phase,
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Pins the context default's shape (`FloatingDelayGroup.tsx:27-35`): no provider, inert
    // zero delays, no owner. The shared-instance semantics (every provider-less consumer
    // receives the SAME object) is exercised in the wasm suite — the thread-local needs the
    // browser binding for the effects that would observe it.
    #[test]
    fn the_default_context_is_inert_and_unowned() {
        DEFAULT_CONTEXT.with(|default| {
            let context = &**default;
            assert!(!context.has_provider, "no provider");
            assert_eq!((context.timeout_ms)(), 0, "timeoutMs defaults to 0");
            assert_eq!(*context.delay_ref.borrow(), Delay::Value(0));
            assert_eq!(*context.initial_delay_ref.borrow(), Delay::Value(0));
            assert!(context.current_id_ref.borrow().is_none(), "no owner");
            assert!(
                context.current_context_ref.borrow().is_none(),
                "no callback pair"
            );
            assert!(!context.timeout.is_started(), "an idle group window");
        });
    }

    // Pins the seed reset's mechanics (`FloatingDelayGroup.tsx:37-39`): the live ref
    // follows the seed. The provider's re-sync effect (the active-membership half,
    // `:75-87`) needs the layout effect (a browser binding), so it lives in the wasm suite.
    #[test]
    fn the_seed_reset_restores_the_live_delay() {
        let delay_ref = Rc::new(RefCell::new(Delay::Partial {
            open: Some(0),
            close: Some(20),
        }));
        let initial_delay_ref = Rc::new(RefCell::new(Delay::Value(0)));
        reset_delay_ref(&delay_ref, &initial_delay_ref);
        assert_eq!(*delay_ref.borrow(), Delay::Value(0));
    }
}

// The provider's and hook's effects run through the ported `use_iso_layout_effect`, whose
// browser binding only exists under the wasm/browser target — every behavioral pin therefore
// runs in the browser, like the crate's other effect-driven suites.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use std::cell::Cell;

    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use reactive_graph::effect::Effect;

    use crate::floating_ui::floating_root_store::FloatingRootStoreOptions;
    use crate::floating_ui::popup_trigger_map::PopupTriggerMap;
    use crate::floating_ui::types::ReferenceType;
    use crate::floating_ui::use_hover::{UseHoverProps, use_hover};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// The upstream suite's fake-timer numbers, compressed for the real-timer wasm runner —
    /// the ratios are kept (`delay: {open: 1000, close: 200} → {open: 100, close: 20}`,
    /// `timeoutMs: 500 → 50`); the sleeps below wait out the compressed equivalents.
    const OPEN_DELAY: u32 = 100;
    const CLOSE_DELAY: u32 = 20;
    const GROUP_WINDOW: u32 = 50;

    /// Removes the fixture's top-level elements on drop — upstream's `afterEach` body sweep
    /// kills the wasm-bindgen-test harness (its own elements live in the body), so fixtures
    /// track what they append (the `mark_others`/`tabbable` precedent).
    struct Fixture {
        elements: Vec<web_sys::Element>,
    }

    impl Fixture {
        fn track(&mut self, element: web_sys::Element) {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&element)
                .unwrap();
            self.elements.push(element);
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            for element in &self.elements {
                let _ = element.remove();
            }
        }
    }

    fn init_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    /// Drains the executor so the tracked effects re-run after their inputs changed — the
    /// wasm analog of the act() wrappers around upstream's fake-timer advances. It must run
    /// after EVERY dispatch (the RenderEffect re-runs a dispatch schedules only execute on a
    /// poll — an early poll also arms the effect loops so later notifications land on the
    /// next poll) and after every real-timer window in which a `Timeout` callback fired (the
    /// store writes the callback performs are synchronous, but the effects they schedule are
    /// not).
    fn flush() {
        for _ in 0..8 {
            any_spawner::Executor::poll_local();
        }
    }

    async fn sleep(ms: i32) {
        let promise = wasm_bindgen_futures::js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }

    type CallLog = Rc<RefCell<Vec<(bool, String)>>>;

    /// The upstream `Tooltip` fixture (`FloatingDelayGroup.test.tsx:15-63`): `useFloating`
    /// + `useDelayGroup` + `useHover` with the group's live delay. Each member owns its own
    /// reactive owner (the React component instance), so individual members can unmount.
    struct Member {
        owner: reactive_graph::owner::Owner,
        store: Rc<FloatingRootStore>,
        button: web_sys::HtmlElement,
        active_id_ref: Rc<RefCell<Option<String>>>,
        is_instant_phase: RwSignal<bool, LocalStorage>,
        delay: Rc<RefCell<Delay>>,
        log: CallLog,
    }

    impl Member {
        fn is_open(&self) -> bool {
            self.store.select(selectors::open)
        }

        fn calls(&self) -> Vec<(bool, String)> {
            self.log.borrow().clone()
        }
    }

    /// Builds the group (`FloatingDelayGroup.test.tsx:65-79` — the provider wrapping the
    /// members) and one member per label, each in its own owner nested under the group's.
    /// Returns the group owner (dispose it to take down the provider) and the members.
    fn setup_group(
        fixture: &mut Fixture,
        labels: &[&str],
    ) -> (reactive_graph::owner::Owner, Vec<Member>) {
        setup_group_with(fixture, labels, OPEN_DELAY, CLOSE_DELAY, GROUP_WINDOW)
    }

    /// [`setup_group`] with explicit group numbers — upstream's per-test `delay`/`timeoutMs`
    /// props (e.g. the `close: 0` group of `FloatingDelayGroup.test.tsx:206`).
    fn setup_group_with(
        fixture: &mut Fixture,
        labels: &[&str],
        open_delay: u32,
        close_delay: u32,
        group_window: u32,
    ) -> (reactive_graph::owner::Owner, Vec<Member>) {
        let group_owner = reactive_graph::owner::Owner::new();
        group_owner.set();
        provide_floating_delay_group(
            move || Delay::Partial {
                open: Some(open_delay),
                close: Some(close_delay),
            },
            move || group_window,
        );

        let mut members = Vec::new();
        for label in labels {
            // Re-select the group owner so each member nests under it, not under a sibling.
            group_owner.set();
            let owner = reactive_graph::owner::Owner::new();
            owner.set();

            let log: CallLog = Rc::new(RefCell::new(Vec::new()));
            let store = FloatingRootStore::new(FloatingRootStoreOptions {
                open: false,
                transition_status: None,
                reference_element: None,
                floating_element: None,
                trigger_elements: PopupTriggerMap::new(),
                floating_id: Some(format!("floating-{label}")),
                sync_only: false,
                nested: false,
                on_open_change: None,
            });
            // The controlled loop (`FloatingDelayGroup.test.tsx:16-20` —
            // `useFloating({ open, onOpenChange: setOpen })`): the consumer's `open` state
            // mirrors the store through the callback.
            let open_signal = RwSignal::new_local(false);
            {
                let weak = Rc::downgrade(&store);
                let log = Rc::clone(&log);
                store.context.set_on_open_change(Some(Rc::new(
                    move |open: bool, details: &RootOpenChangeEventDetails| {
                        open_signal.set(open);
                        if let Some(store) = weak.upgrade() {
                            store.update(|state, _| {
                                state.open = open;
                                true
                            });
                        }
                        log.borrow_mut().push((open, details.reason.clone()));
                    },
                )));
            }

            let document = web_sys::window().unwrap().document().unwrap();
            let button: web_sys::HtmlElement = document
                .create_element("button")
                .unwrap()
                .dyn_into()
                .unwrap();
            let floating = document.create_element("div").unwrap();
            fixture.track(button.clone().into());
            fixture.track(floating.clone());
            store.set_field(
                |state| &mut state.reference_element,
                Some(ReferenceType::Element(button.clone().into())),
            );
            store.set_field(
                |state| &mut state.dom_reference_element,
                Some(button.clone().into()),
            );
            store.set_field(|state| &mut state.floating_element, Some(floating));

            // `useDelayGroup(context, { open })` before `useHover` — the fixture's call
            // order (`FloatingDelayGroup.test.tsx:23-24`).
            let group_return = use_delay_group(
                Rc::clone(&store),
                UseDelayGroupOptions::new(move || open_signal.get()),
            );
            // `useHover(context, { delay: () => delayRef.current })` (`:24`).
            let hover_delay_ref = Rc::clone(&group_return.delay_ref);
            let hover_props = use_hover(
                Rc::clone(&store),
                UseHoverProps {
                    delay: DelayInput::Resolve(Rc::new(move || hover_delay_ref.borrow().clone())),
                    ..UseHoverProps::default()
                },
            );
            let _cleanup = hover_props
                .reference
                .as_ref()
                .expect("the reference bag")
                .attach_to(button.as_ref())
                .expect("slots attached");

            members.push(Member {
                owner,
                store,
                button,
                active_id_ref: group_return.active_id_ref,
                is_instant_phase: group_return.is_instant_phase,
                delay: group_return.delay_ref,
                log,
            });
        }

        (group_owner, members)
    }

    /// Counts this member's `isInstantPhase` notifications — the reactive_graph analog of
    /// upstream's render-count spans (`FloatingDelayGroup.test.tsx:27-35,376-378`): the
    /// only member-local state the group machinery writes is the instant marker, so an
    /// uninvolved member's counter staying at its initial run pins "membership changes
    /// never re-render other members" (implementation.md, "DelayGroup").
    fn track_instant_runs(member: &Member) -> Rc<Cell<usize>> {
        member.owner.set();
        let is_instant_phase = member.is_instant_phase.clone();
        let runs = Rc::new(Cell::new(0));
        let runs_for_effect = Rc::clone(&runs);
        Effect::new(move |_: Option<usize>| -> usize {
            is_instant_phase.get();
            let next = runs_for_effect.get() + 1;
            runs_for_effect.set(next);
            next
        });
        runs
    }

    fn enter(member: &Member) {
        member
            .button
            .dispatch_event(&web_sys::MouseEvent::new("mouseenter").unwrap())
            .unwrap();
    }

    fn leave(member: &Member) {
        member
            .button
            .dispatch_event(&web_sys::MouseEvent::new("mouseleave").unwrap())
            .unwrap();
    }

    // `groups delays correctly` (`FloatingDelayGroup.test.tsx:86-134`): the first member
    // waits the full open delay; each switch to a new member is instant and force-closes
    // the previous owner with REASONS.none; the last leave waits the close delay.
    #[wasm_bindgen_test(async)]
    async fn groups_delays_correctly() {
        init_executor();
        let mut fixture = Fixture {
            elements: Vec::new(),
        };
        let (_group_owner, members) = setup_group(&mut fixture, &["one", "two", "three"]);
        let [one, two, three] = members.try_into().ok().unwrap();

        enter(&one);
        flush();
        sleep(30).await;
        assert!(
            !one.is_open(),
            "the first member has not opened before the group's open delay elapses"
        );
        sleep(90).await;
        flush();
        assert!(
            one.is_open(),
            "the first member opens at the group's open delay"
        );
        assert_eq!(
            one.delay.borrow().open(),
            0,
            "the open member zeroes the group's open delay"
        );

        enter(&two);
        flush();
        assert!(!one.is_open(), "the switch force-closes the previous owner");
        assert!(two.is_open(), "the next member opens instantly");
        assert_eq!(
            one.calls(),
            vec![
                (true, reasons::TRIGGER_HOVER.to_owned()),
                (false, reasons::NONE.to_owned()),
            ],
            "the force-close rides REASONS.none"
        );
        assert_eq!(
            two.calls(),
            vec![(true, reasons::TRIGGER_HOVER.to_owned())],
            "the takeover itself opens with the hover reason"
        );
        enter(&three);
        flush();
        assert!(!two.is_open());
        assert!(three.is_open());

        leave(&three);
        flush();
        sleep(5).await;
        assert!(three.is_open(), "the close waits the group's close delay");
        sleep(CLOSE_DELAY as i32 + 30).await;
        assert!(!three.is_open(), "the close fires after the close delay");
    }

    // `timeoutMs` (`FloatingDelayGroup.test.tsx:136-201`): the close delay and the group
    // window are independent — after the close delay fires, the window keeps the instant
    // open alive for later members.
    #[wasm_bindgen_test(async)]
    async fn timeout_ms_keeps_the_window_alive_after_the_close() {
        init_executor();
        let mut fixture = Fixture {
            elements: Vec::new(),
        };
        let (_group_owner, members) = setup_group(&mut fixture, &["one", "two", "three"]);
        let [one, two, three] = members.try_into().ok().unwrap();

        enter(&one);
        flush();
        sleep(OPEN_DELAY as i32 + 30).await;
        flush();
        assert!(one.is_open());

        leave(&one);
        flush();
        assert!(one.is_open(), "the close delay defers the close");
        sleep(CLOSE_DELAY as i32 + 5).await;
        flush();
        assert!(!one.is_open(), "the close fires after the close delay");

        // The window (started at the close) is still running; the takeover is instant.
        enter(&two);
        flush();
        assert!(
            two.is_open(),
            "the group window keeps the instant open alive for the next member"
        );

        enter(&three);
        flush();
        assert!(!two.is_open());
        assert!(three.is_open());

        leave(&three);
        flush();
        sleep(5).await;
        assert!(three.is_open(), "the close waits the close delay");
        sleep(CLOSE_DELAY as i32 + 30).await;
        flush();
        assert!(!three.is_open(), "the close fires after the close delay");
    }

    // `resets the instant phase after the group window closes`
    // (`FloatingDelayGroup.test.tsx:203-236`): the close clears the instant marker
    // immediately (`:182`), and the window's reset restores the group state for the next
    // cycle. Upstream also replays the lifecycle effects through StrictMode; reactive_graph
    // has no StrictMode analog, so the port pins the reset the double-invocation protects.
    #[wasm_bindgen_test(async)]
    async fn resets_the_instant_phase_after_the_group_window_closes() {
        init_executor();
        let mut fixture = Fixture {
            elements: Vec::new(),
        };
        // Upstream's group is `delay={{ open: 1000, close: 0 }} timeoutMs={50}`
        // (`FloatingDelayGroup.test.tsx:206`) — a zero close delay makes the close
        // synchronous, which is what lets the test pin the marker clearing at close time,
        // before the window, rather than at the window's expiry.
        let (_group_owner, members) =
            setup_group_with(&mut fixture, &["one", "two"], OPEN_DELAY, 0, GROUP_WINDOW);
        let [one, two] = members.try_into().ok().unwrap();

        enter(&one);
        flush();
        sleep(OPEN_DELAY as i32 + 30).await;
        flush();
        assert!(one.is_open());
        assert_eq!(
            one.delay.borrow().open(),
            0,
            "the open member zeroes the group's open delay"
        );

        enter(&two);
        flush();
        assert!(two.is_open(), "the takeover opens instantly");
        assert!(
            two.is_instant_phase.get(),
            "the takeover marks the incoming member instant"
        );

        leave(&two);
        flush();
        assert!(
            !two.is_instant_phase.get(),
            "the close clears the instant marker immediately (`:182`)"
        );
        assert_eq!(
            two.active_id_ref.borrow().as_deref(),
            Some("floating-two"),
            "the member still owns the group during the window"
        );
        sleep(GROUP_WINDOW as i32 + 40).await;
        flush();
        assert!(
            two.active_id_ref.borrow().is_none(),
            "the window's reset clears the ownership"
        );
        assert_eq!(
            two.delay.borrow().open(),
            OPEN_DELAY,
            "the window's reset restores the seed delay"
        );
    }

    // `keeps the active context when an inactive consumer unmounts`
    // (`FloatingDelayGroup.test.tsx:238-283`): disposing a member that never owned the
    // group leaves the active owner and its context pair untouched.
    #[wasm_bindgen_test(async)]
    async fn keeps_the_active_context_when_an_inactive_consumer_unmounts() {
        init_executor();
        let mut fixture = Fixture {
            elements: Vec::new(),
        };
        let (_group_owner, members) = setup_group(&mut fixture, &["one", "two", "three"]);
        let [one, two, three] = members.try_into().ok().unwrap();

        enter(&one);
        flush();
        sleep(OPEN_DELAY as i32 + 30).await;
        flush();
        assert!(one.is_open());

        // Remove the inactive member (`FloatingDelayGroup.test.tsx:255-257`).
        two.owner.cleanup();
        flush();

        enter(&three);
        flush();
        assert!(!one.is_open(), "the takeover still force-closes the owner");
        assert!(
            three.is_open(),
            "the disposed inactive member did not disturb the group"
        );
        assert_eq!(
            one.calls(),
            vec![
                (true, reasons::TRIGGER_HOVER.to_owned()),
                (false, reasons::NONE.to_owned()),
            ],
            "the force-close still rides REASONS.none"
        );
    }

    // `keeps the timeout active when the last closed consumer unmounts`
    // (`FloatingDelayGroup.test.tsx:285-332`): a closed owner unmounting inside the window
    // preserves the group window for the next member.
    #[wasm_bindgen_test(async)]
    async fn keeps_the_timeout_active_when_the_last_closed_consumer_unmounts() {
        init_executor();
        let mut fixture = Fixture {
            elements: Vec::new(),
        };
        let (_group_owner, members) = setup_group(&mut fixture, &["one", "two"]);
        let [one, two] = members.try_into().ok().unwrap();

        enter(&one);
        flush();
        sleep(OPEN_DELAY as i32 + 30).await;
        flush();
        assert!(one.is_open());

        leave(&one);
        flush();
        sleep(CLOSE_DELAY as i32 + 5).await;
        flush();
        assert!(
            !one.is_open(),
            "the close fired; the group window is running"
        );

        // Remove the closed owner during the window (`FloatingDelayGroup.test.tsx:299-301`).
        one.owner.cleanup();
        flush();

        enter(&two);
        flush();
        assert!(
            two.is_open(),
            "the window survived the closed owner's unmount"
        );
    }

    // `does not re-render unrelated consumers` (`FloatingDelayGroup.test.tsx:334-379`) —
    // port-owned analog: upstream counts React renders; the reactive_graph equivalent of
    // "membership changes never re-render other members" (implementation.md, "DelayGroup")
    // is that a member's local instant marker is only ever notified by its own involvement —
    // never by another member's open, close, or delay changes. The counts below are
    // notifications of each tracked marker: 1 is the tracker's initial run, each marker
    // write notifies once (reactive_graph's `Set` has no same-value bail-out — unlike
    // React's setState, a same-value `setIsInstantPhase(false)` still notifies, `:242-243`),
    // so a member's count only grows when the group machinery writes ITS marker.
    #[wasm_bindgen_test(async)]
    async fn uninvolved_members_local_state_stays_untouched() {
        init_executor();
        let mut fixture = Fixture {
            elements: Vec::new(),
        };
        let (_group_owner, members) = setup_group(&mut fixture, &["one", "two", "three"]);
        let [one, two, three] = members.try_into().ok().unwrap();
        let one_runs = track_instant_runs(&one);
        let three_runs = track_instant_runs(&three);

        enter(&one);
        flush();
        sleep(OPEN_DELAY as i32 + 30).await;
        flush();
        assert!(one.is_open(), "one opened on the group's delay");
        assert_eq!(
            one_runs.get(),
            2,
            "initial + one's own open's instant-clear write (`:242`)"
        );
        assert_eq!(three_runs.get(), 1, "the uninvolved member: initial only");

        enter(&two);
        flush();
        assert!(two.is_open(), "two took over");
        assert_eq!(
            one_runs.get(),
            3,
            "the takeover of two marks the outgoing one instant (`:239`) — one's own involvement"
        );
        assert_eq!(
            three_runs.get(),
            1,
            "the one→two switch never touches the uninvolved member"
        );

        enter(&three);
        flush();
        assert!(three.is_open(), "three took over");
        assert_eq!(
            three_runs.get(),
            2,
            "three's own takeover marks it instant (`:238`)"
        );
        assert_eq!(
            one_runs.get(),
            3,
            "the two→three switch never touches the already-inactive one"
        );
    }
}
