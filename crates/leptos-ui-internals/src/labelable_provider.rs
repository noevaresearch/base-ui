//! Port of `packages/react/src/internals/labelable-provider/` — the context unit providing
//! labelable elements with an accessible name (label) and description
//! (`specs/library/internals/implementation.md`, "Context providers/consumers" — the
//! `LabelableContext` row and the cross-boundary state notes; all claims verified against the
//! source before porting).
//!
//! The upstream directory is five modules, consolidated here in upstream's own order:
//! [`ControlIdSource`] (the `Symbol()` ownership token),
//! `LabelableContext.ts` ([`LabelableContextValue`], the no-op default, and the
//! provide/use accessors), `LabelableProvider.tsx` ([`provide_labelable_context`]),
//! `useLabelableId.ts` ([`use_labelable_id`]), `useLabel.ts` ([`use_label`]), and
//! `useAriaLabelledBy.ts` ([`use_aria_labelled_by`]). The whole directory has no tests in
//! the upstream unit (`specs/library/internals/implementation.md`, "Anything in source not
//! explained by any test" item 4), so the suites below pin the written mechanics per the
//! PrehydrationScript precedent.
//!
//! ## Behavior carried over
//!
//! - `controlId` is a *last-registered-wins with sticky selection* reducer over
//!   symbol-keyed control registrations: an empty registration map preserves the current id
//!   (React Activity/Suspense hidden subtrees keep their DOM but lose effects,
//!   `LabelableProvider.tsx:36-58`), a still-registered current id is kept over newer
//!   registrations (`:46-50`), an explicit `null` registration deliberately suppresses
//!   `htmlFor` (`LabelableContext.ts:8-11`), and `resetControlId` restores the default id
//!   when nothing is registered (`:62-66`).
//! - `useLabelableId` returns `controlId ?? id ?? defaultId` — the provider's
//!   pre-registration state wins so SSR `htmlFor` pairs up
//!   (`useLabelableId.ts:79-82`) — and it unregisters in the layout phase so a replacement
//!   control's layout effect never observes the outgoing registration (`:73-77`).
//! - `useLabel` resolves the control id, skips the interaction pipeline when the pointer
//!   lands on a nested form control (`useLabel.ts:46-50`), suppresses text selection on
//!   double clicks (`:52-55`), and focuses the resolved control on non-native labels
//!   (`:57-62`, `:30-44`) with the `focusVisible: true` focus options (`:103-108`).
//! - `useAriaLabelledBy` resolves `explicit ?? labelId ?? fallback` (`useAriaLabelledBy.ts:16`)
//!   and its fallback discovers the associated native `<label>` through the three fast paths:
//!   wrapping parent, preceding sibling matched by `htmlFor`, and the `.labels` property
//!   (`:49-70`), assigning the generated id to an id-less label (`:42-44`).
//!
//! ## Rust adaptations
//!
//! - React context ports to reactive-graph's owner-scoped
//!   [`provide_context`]/[`use_context`] behind [`SharedLabelableContext`] — the
//!   `SendWrapper` bridge (the `composite_list.rs` precedent). The all-`NOOP` default context
//!   (`LabelableContext.ts:32-41`) is built fresh per provider-less access with inert
//!   signals: within this unit every consumer reads `controlId` through `??` chains and
//!   falsy guards, so the default's `undefined` `controlId` and a provider-level `null`
//!   behave identically at the accessors — the port's context surface keeps one
//!   `Option<String>` where `None` is "no `htmlFor`" (the React-17 `undefined` window the
//!   provider guards at `:18-20` dissolves: the `use_id` port's supported path always yields
//!   a string).
//! - The `Symbol()` ownership token (`useLabelableId.ts:19`,
//!   `useRefWithInit(() => Symbol())`) ports to [`ControlIdSource`], a process-unique
//!   counter — Rust has no `Symbol`, and the token is only ever compared for equality as a
//!   map key. The registration map is an insertion-ordered vec keyed by the token (JS `Map`
//!   semantics: `set` on an existing key keeps its position, `delete` removes it).
//! - `useStableCallback`/`useCallback` handlers (`LabelableProvider.tsx:26-81`) port to plain
//!   `Rc` closures — the hook runs once, so the identity stabilization has nothing to
//!   stabilize (the `composite_root_context.rs` convention). The `registerControlId === NOOP`
//!   identity checks (`useLabelableId.ts:24`, `:33`) port to an `Rc::ptr_eq` against the
//!   shared no-op registration the default context hands out.
//! - `setDescriptionProps`-style props plumbing: upstream `getDescriptionProps` takes the
//!   whole `HTMLProps` and returns it with `aria-describedby` merged
//!   (`LabelableProvider.tsx:68-81`). The port's attribute surface is the crate's
//!   `(name, lazy-value)` bag (`specs/architecture.md`, the mergeProps decision), and the
//!   only member the function touches is `aria-describedby`, so the port is a function over
//!   that bag: it reads the external `aria-describedby` eagerly at call time and installs a
//!   lazy entry re-reading `messageIds` at attribute-read time — the roving-tabIndex
//!   lazy-attribute convention. The upstream `split(' ')` (not `split_whitespace`) and
//!   first-occurrence `Set` dedupe are preserved (`:70-77`).
//! - The label/`messageIds` setters (`setLabelId`/`setMessageIds`) port to the
//!   [`RwSignal`] handles themselves — a React state setter is a write; consumers compose
//!   the shared [`LabelIdUpdate`] contract of `useRegisteredLabelId` on top.
//! - `useLabel`'s returned props bag ports to [`LabelProps`]: the `id` attribute (the
//!   registered label id) and `htmlFor` (reactive over the resolved control id) travel as
//!   signal reads, the interaction handlers as native-event handler slots — internal hooks
//!   return native handlers and the view layer adapts them into the [`BaseUIEvent`]-typed
//!   bags (`use_render_element.rs` convention). `useLabelableId`'s `id`/`enabled` params are
//!   static per instance: the effect's reactive re-runs a changing prop would drive are
//!   discharged to the view layer (the `use_composite_root.rs` `set_disabled_indices`
//!   precedent).
//! - `useAriaLabelledBy`'s no-deps layout effect (`useAriaLabelledBy.ts:22-31` — "run after
//!   every commit so DOM association changes are reflected") has no reactive counterpart for
//!   the commit edge: the port re-runs on the tracked `labelId` changes and the per-commit
//!   re-check is discharged to the view layer, the same divergence `use_composite_list_item`
//!   documents for ref-driven re-registration. The DOM reads ride the ref slot type of the
//!   crate's `HTMLProps` vocabulary (crate::types).
//! - `'use client'` is N/A — no React Server Components boundary in Rust.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use reactive_graph::owner::{LocalStorage, provide_context, use_context};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use web_sys::Element;
use web_sys::wasm_bindgen::JsCast;

use floating_ui_dom::dom::is_html_element;
use leptos_ui_utils::owner::owner_document;
use leptos_ui_utils::shadow_dom::get_target;
use leptos_ui_utils::use_iso_layout_effect;

use crate::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use crate::use_base_ui_id::use_base_ui_id;
use crate::use_registered_label_id::{LabelIdSetter, LabelIdUpdate, use_registered_label_id};

// ---------------------------------------------------------------------------
// The Symbol() stand-in (useLabelableId.ts:19)
// ---------------------------------------------------------------------------

thread_local! {
    /// The process-unique token source behind [`ControlIdSource::new`].
    static NEXT_CONTROL_SOURCE: Cell<u64> = const { Cell::new(0) };
}

/// The per-control ownership token — upstream's `useRefWithInit(() => Symbol())`
/// (`useLabelableId.ts:19`; the field-register-control cross-note in
/// `specs/library/internals/implementation.md` records the same shape). The
/// `registerControlId`/registration map pair keys ownership by this token, so a replaced
/// control's registration is dropped instead of leaking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ControlIdSource(u64);

impl ControlIdSource {
    /// Allocates a fresh token — the `Symbol()` call.
    pub fn new() -> Self {
        Self(NEXT_CONTROL_SOURCE.with(|next| {
            let token = next.get() + 1;
            next.set(token);
            token
        }))
    }
}

impl Default for ControlIdSource {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LabelableContext.ts
// ---------------------------------------------------------------------------

/// One registration of a control id — the `registerControlId(source, nextId)` parameter
/// union (`LabelableContext.ts:13`): `string | null | undefined`, where `undefined` removes
/// the source's registration, `null` registers "deliberately suppress `htmlFor`", and a
/// string registers the concrete id.
pub type ControlIdRegistration = Option<Option<String>>;

/// The registration call — `registerControlId` (`LabelableContext.ts:13`).
pub type RegisterControlIdFn = Rc<dyn Fn(ControlIdSource, ControlIdRegistration)>;

/// The `aria-describedby` merger — the port's `HTMLProps`-passing (crate::types)
/// `getDescriptionProps(externalProps) => HTMLProps` slice (module docs): reads the
/// external value eagerly, installs a lazy merged entry re-reading `messageIds`.
pub type DescriptionPropsFn = Rc<dyn Fn(&mut Vec<(String, ElementAttributeFn)>)>;

/// The context bag — upstream's `LabelableContext`
/// (`packages/react/src/internals/labelable-provider/LabelableContext.ts:6-26`).
#[derive(Clone)]
pub struct LabelableContextValue {
    /// `controlId` (`:8-12`) — `Some(id)` pairs the label with `htmlFor`; `None` omits it
    /// (the `null` suppression, or the no-provider default; see the module docs).
    pub control_id: Signal<Option<String>, LocalStorage>,
    /// `registerControlId` (`:13`).
    pub register_control_id: RegisterControlIdFn,
    /// `resetControlId` (`:14`).
    pub reset_control_id: Rc<dyn Fn()>,
    /// `labelId` (`:16-19`) — the registered label id; the `setLabelId` setter is the
    /// signal write.
    pub label_id: RwSignal<Option<String>, LocalStorage>,
    /// `messageIds` (`:21-24`) — the accessible-description ids; `setMessageIds` is the
    /// signal write.
    pub message_ids: RwSignal<Vec<String>, LocalStorage>,
    /// `getDescriptionProps` (`:25`) — the `aria-describedby` merger (module docs).
    pub get_description_props: DescriptionPropsFn,
}

/// The context type provided through the reactive owner — the `SendWrapper` bridge
/// `provide_context`'s `Send + Sync` contract requires (the `composite_list.rs` precedent).
pub type SharedLabelableContext = SendWrapper<LabelableContextValue>;

thread_local! {
    /// The no-op registration the default context hands out — shared so the
    /// `registerControlId === NOOP` identity checks (`useLabelableId.ts:24`, `:33`) port to
    /// an `Rc::ptr_eq` against this instance.
    static NOOP_REGISTER_CONTROL_ID: RegisterControlIdFn = Rc::new(|_, _| {});
}

/// Whether `register` is the default context's no-op — upstream's
/// `registerControlId === NOOP` (`useLabelableId.ts:24`, `:33`).
fn is_noop_register_control_id(register: &RegisterControlIdFn) -> bool {
    NOOP_REGISTER_CONTROL_ID.with(|noop| Rc::ptr_eq(register, noop))
}

/// Upstream's default context value (`LabelableContext.ts:32-41`): `controlId: undefined`,
/// the two registration calls and both setters `NOOP`, `messageIds: []`, and an identity
/// `getDescriptionProps`. Built fresh per provider-less access inside the caller's owner —
/// the inert signals make the setter no-ops observational (module docs).
fn noop_labelable_context() -> LabelableContextValue {
    LabelableContextValue {
        control_id: Signal::derive_local(|| None),
        register_control_id: NOOP_REGISTER_CONTROL_ID.with(Rc::clone),
        reset_control_id: Rc::new(|| {}),
        label_id: RwSignal::new_local(None),
        message_ids: RwSignal::new_local(Vec::new()),
        get_description_props: Rc::new(|_| {}),
    }
}

/// The accessor — upstream's `useLabelableContext` (`LabelableContext.ts:43-45`), falling
/// back to the no-op default outside a provider.
pub fn use_labelable_context() -> LabelableContextValue {
    use_context::<SharedLabelableContext>()
        .map(|shared| (*shared).clone())
        .unwrap_or_else(noop_labelable_context)
}

// ---------------------------------------------------------------------------
// LabelableProvider.tsx
// ---------------------------------------------------------------------------

/// The `last-registered-wins with sticky selection` reducer
/// (`LabelableProvider.tsx:36-58`): an empty map preserves `prev`; a still-registered
/// `prev` wins wherever it sits; otherwise the first registered value (insertion order,
/// which may be a `null` suppression) becomes the control id.
fn reduce_control_id(
    registrations: &[(ControlIdSource, Option<String>)],
    prev: Option<String>,
) -> Option<String> {
    if registrations.is_empty() {
        // A hidden subtree (React Activity, a re-suspending Suspense) destroys effects but
        // keeps its DOM, so preserve its selected control (`:37-41`).
        return prev;
    }

    for (_, id) in registrations {
        // Keep the current selection while it is still registered, so rapid
        // unmount/remount cycles don't churn it (`:46-50`).
        if *id == prev {
            return prev;
        }
    }

    registrations
        .first()
        .map(|(_, id)| id.clone())
        .expect("the empty-map branch returned above")
}

/// Upserts one registration, preserving the insertion position of an existing key — JS
/// `Map.set`/`delete` semantics over the insertion-ordered vec (module docs).
fn upsert_registration(
    registrations: &mut Vec<(ControlIdSource, Option<String>)>,
    source: ControlIdSource,
    id: Option<String>,
) {
    match registrations.iter_mut().find(|(key, _)| *key == source) {
        Some(entry) => entry.1 = id,
        None => registrations.push((source, id)),
    }
}

/// Port of `LabelableProvider` (`packages/react/src/internals/labelable-provider/LabelableProvider.tsx:9-109`):
/// builds the context state and publishes it for the current subtree — the component's
/// `<LabelableContext.Provider>` render (`:106-108`) without the React runtime. Nestable:
/// the description merge reads the parent context's `messageIds` (`:24`). Must be called
/// inside a reactive owner (a component), like the other hook ports.
pub fn provide_labelable_context() -> LabelableContextValue {
    // `const defaultId = useBaseUiId()` (`:12`) — no override.
    let default_id = use_base_ui_id(Signal::derive_local(|| None::<String>));

    // `useState<string | null | undefined>(defaultId)` (`:14`); the React-17 `undefined`
    // window the `=== undefined` guard (`:18-20`) handles dissolves (module docs), so the
    // state starts on the resolved default id and `None` is the `null` suppression.
    let control_id_state: RwSignal<Option<String>, LocalStorage> =
        RwSignal::new_local(Some(default_id.get_untracked()));

    // `useState<string | undefined>()` / `useState<string[]>([])` (`:15-16`).
    let label_id: RwSignal<Option<String>, LocalStorage> = RwSignal::new_local(None);
    let message_ids: RwSignal<Vec<String>, LocalStorage> = RwSignal::new_local(Vec::new());

    // `useRefWithInit(() => new Map<symbol, string | null>())` (`:22`).
    let registrations: Rc<RefCell<Vec<(ControlIdSource, Option<String>)>>> =
        Rc::new(RefCell::new(Vec::new()));

    // `const { messageIds: parentMessageIds } = useLabelableContext()` (`:24`) — nested
    // providers read the parent's ids; the signal handle is captured so the merge stays
    // fresh (module docs).
    let parent_message_ids: Option<RwSignal<Vec<String>, LocalStorage>> =
        use_context::<SharedLabelableContext>().map(|shared| shared.message_ids.clone());

    // `registerControlId` (`:26-60`).
    let register_control_id: RegisterControlIdFn = {
        let registrations = Rc::clone(&registrations);
        let control_id_state = control_id_state.clone();
        Rc::new(move |source, next_id| {
            {
                let mut registrations = registrations.borrow_mut();
                match next_id {
                    None => registrations.retain(|(key, _)| *key != source),
                    Some(id) => upsert_registration(&mut registrations, source, id),
                }
            }

            // `setControlIdState((prev) => ...)` (`:36-58`) with React's same-value
            // bail-out reproduced as the inequality guard (the reactive_graph notify-on-
            // every-set adaptation, the `use_transition_status.rs` precedent).
            let prev = control_id_state.get_untracked();
            let next = reduce_control_id(&registrations.borrow(), prev.clone());
            if next != prev {
                control_id_state.set(next);
            }
        })
    };

    // `resetControlId` (`:62-66`).
    let reset_control_id: Rc<dyn Fn()> = {
        let registrations = Rc::clone(&registrations);
        let control_id_state = control_id_state.clone();
        let default_id = default_id.clone();
        Rc::new(move || {
            if registrations.borrow().is_empty() {
                let target = Some(default_id.get_untracked());
                if control_id_state.get_untracked() != target {
                    control_id_state.set(target);
                }
            }
        })
    };

    // `const controlId = controlIdState === undefined ? defaultId : controlIdState`
    // (`:18-20`) — the guard dissolves (module docs), so the context read is the state.
    let control_id: Signal<Option<String>, LocalStorage> =
        Signal::derive_local(move || control_id_state.get());

    // `getDescriptionProps` (`:68-81`) — the eager external read + the lazy merged entry
    // (module docs).
    let get_description_props: DescriptionPropsFn = {
        let message_ids = message_ids.clone();
        Rc::new(move |attributes: &mut Vec<(String, ElementAttributeFn)>| {
            let external = attributes
                .iter()
                .find(|(name, _)| name == "aria-describedby")
                .and_then(|(_, value)| value());

            let parent_message_ids = parent_message_ids.clone();
            let message_ids = message_ids.clone();
            let merged: ElementAttributeFn = Rc::new(move || {
                let mut ids: Vec<String> = match &external {
                    Some(value) => value.split(' ').map(str::to_string).collect(),
                    None => Vec::new(),
                };
                if let Some(parent) = &parent_message_ids {
                    ids.extend(parent.get());
                }
                ids.extend(message_ids.get());

                // `Array.from(new Set(ids)).join(' ') || undefined` (`:77`) — the
                // first-occurrence dedupe; an empty join means the prop is omitted.
                let mut seen = Vec::new();
                for id in ids {
                    if !seen.contains(&id) {
                        seen.push(id);
                    }
                }
                let joined = seen.join(" ");
                (!joined.is_empty()).then_some(joined)
            });

            match attributes
                .iter_mut()
                .find(|(name, _)| name == "aria-describedby")
            {
                Some((_, value)) => *value = merged,
                None => attributes.push(("aria-describedby".to_string(), merged)),
            }
        })
    };

    let value = LabelableContextValue {
        control_id,
        register_control_id,
        reset_control_id,
        label_id,
        message_ids,
        get_description_props,
    };
    provide_context(SharedLabelableContext::new(value.clone()));
    value
}

// ---------------------------------------------------------------------------
// useLabelableId.ts
// ---------------------------------------------------------------------------

/// The parameters — upstream's `UseLabelableIdParameters`
/// (`packages/react/src/internals/labelable-provider/useLabelableId.ts:85-96`).
#[derive(Default)]
pub struct UseLabelableIdParams {
    /// `id` (`:87-90`) — the control's `id`; `None` is both `undefined` (generate) and the
    /// JSDoc's `null` (an `aria-labelledby` control), which behave identically here.
    pub id: Option<String>,
    /// `enabled` (`:91-95`) — whether the control owns the label association of its
    /// labelable scope. Defaults to `true`.
    pub enabled: bool,
}

/// Port of `useLabelableId` (`useLabelableId.ts:10-83`): registers the control's id with
/// the labelable scope and returns the resolved id — the provider's pre-registration state
/// wins (`:79-82`). Must be called inside a reactive owner (a component).
///
/// The `id`/`enabled` params are static per instance; the reactive re-runs a changing prop
/// would drive are discharged to the view layer (module docs).
pub fn use_labelable_id(params: UseLabelableIdParams) -> Signal<String, LocalStorage> {
    let UseLabelableIdParams { id, enabled } = params;

    let context = use_labelable_context();

    // `const defaultId = useBaseUiId()` (`:17`) — deliberately not seeded with `id` (`:15-16`).
    let default_id = use_base_ui_id(Signal::derive_local(|| None::<String>));

    // `useRefWithInit(() => Symbol())` (`:19`), plus the two guard refs (`:20-21`).
    let control_source = ControlIdSource::new();
    let has_registered = Rc::new(Cell::new(false));
    let had_explicit_id = Rc::new(Cell::new(false));

    // `unregisterControlId` (`:23-30`).
    let unregister_control_id: Rc<dyn Fn()> = {
        let has_registered = Rc::clone(&has_registered);
        let register_control_id = context.register_control_id.clone();
        Rc::new(move || {
            if !has_registered.get() || is_noop_register_control_id(&register_control_id) {
                return;
            }
            has_registered.set(false);
            register_control_id(control_source, None);
        })
    };

    // The registration effect (`:32-71`).
    {
        let unregister_control_id = Rc::clone(&unregister_control_id);
        let register_control_id = context.register_control_id.clone();
        let reset_control_id = context.reset_control_id.clone();
        let has_registered = Rc::clone(&has_registered);
        let had_explicit_id = Rc::clone(&had_explicit_id);
        let default_id = default_id.clone();
        let id = id.clone();
        use_iso_layout_effect(move || {
            if !enabled || is_noop_register_control_id(&register_control_id) {
                unregister_control_id();
                return;
            }

            let next_id: Option<String> = if let Some(explicit) = &id {
                had_explicit_id.set(true);
                Some(explicit.clone())
            } else if had_explicit_id.get() {
                // An id-less replacement after an explicit id keeps the fallback.
                Some(default_id.get())
            } else {
                // An id-less replacement must claim the provider's fallback so a previously
                // registered explicit id is not retained after its control unmounts
                // (`:46-49`).
                reset_control_id();
                return;
            };

            // Either the control never had an explicit `id`, or React 17 has not assigned
            // the fallback id yet (`:52-57`) — the second arm never fires here (module
            // docs).
            let Some(next_id) = next_id else {
                unregister_control_id();
                return;
            };

            has_registered.set(true);
            register_control_id(control_source, Some(Some(next_id)));
        });
    }

    // The layout-phase unregistration (`:73-77`): a replacement control's layout effect
    // would otherwise run first and still see the outgoing control's registration.
    {
        let unregister_control_id = SendWrapper::new(unregister_control_id);
        use_iso_layout_effect(move || {
            let unregister_control_id = unregister_control_id.clone();
            reactive_graph::owner::on_cleanup(move || unregister_control_id());
        });
    }

    // The return (`:82`): the provider's id wins until registration runs.
    Signal::derive_local(move || {
        if !enabled {
            return id.clone().unwrap_or_else(|| default_id.get());
        }
        match context.control_id.get() {
            Some(control_id) => control_id,
            None => id.clone().unwrap_or_else(|| default_id.get()),
        }
    })
}

// ---------------------------------------------------------------------------
// useLabel.ts
// ---------------------------------------------------------------------------

/// The parameters — upstream's `UseLabelParameters`
/// (`packages/react/src/internals/labelable-provider/useLabel.ts:79-99`).
#[derive(Default)]
pub struct UseLabelParams {
    /// `id` (`:80`).
    pub id: Option<String>,
    /// `fallbackControlId` (`:81-84`) — the control id used when no labelable context
    /// control id exists.
    pub fallback_control_id: Option<String>,
    /// `native` (`:85-89`) — whether the rendered element is a native `<label>`. Defaults
    /// to `false`.
    pub native: bool,
    /// `setLabelId` (`:90-93`) — an additional callback syncing the current label id with a
    /// local component state/store; receives the same [`LabelIdUpdate`] dispatches the
    /// context setter answers (module docs).
    pub set_label_id: Option<LabelIdSetter>,
    /// `focusControl` (`:94-98`) — a custom focus handler for non-native labels; when
    /// omitted, focus behavior targets the resolved control id.
    pub focus_control: Option<Rc<dyn Fn(&web_sys::MouseEvent, Option<String>)>>,
}

/// The returned props bag — upstream's
/// `React.HTMLAttributes<any> & React.LabelHTMLAttributes<any>` return
/// (`useLabel.ts:101`) narrowed to the members the two return branches build (`:64-76`).
/// The view layer maps the handler slots onto the element (module docs).
pub struct LabelProps {
    /// The label element's `id` — the registered id (the `useRegisteredLabelId` return).
    pub id: Signal<String, LocalStorage>,
    /// `htmlFor` — the resolved control id, reactive over the context's `controlId`
    /// (`:28`, `:67`).
    pub for_control: Signal<Option<String>, LocalStorage>,
    /// `onMouseDown` — the native-label interaction slot (`:68`).
    pub on_mouse_down: Option<ElementEventHandler<web_sys::MouseEvent>>,
    /// `onClick` — the non-native interaction slot (`:72`).
    pub on_click: Option<ElementEventHandler<web_sys::MouseEvent>>,
    /// `onPointerDown` — the non-native text-selection suppression slot (`:73-75`).
    pub on_pointer_down: Option<ElementEventHandler<web_sys::PointerEvent>>,
}

/// `focusElementWithVisible` (`useLabel.ts:103-108`): focus with the `focusVisible: true`
/// option (Chrome 144+; Safari and Firefox already support it).
pub fn focus_element_with_visible(element: &web_sys::HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_focus_visible(true);
    let _ = element.focus_with_options(&options);
}

/// Port of `useLabel` (`useLabel.ts:10-77`). Must be called inside a reactive owner (a
/// component) — the id registration registers an effect there.
pub fn use_label(params: UseLabelParams) -> LabelProps {
    let UseLabelParams {
        id: id_prop,
        fallback_control_id,
        native,
        set_label_id: set_label_id_prop,
        focus_control: focus_control_prop,
    } = params;

    let context = use_labelable_context();

    // `syncLabelId` (`:21-24`): the context write plus the optional store sync, both
    // receiving the same dispatch shape.
    let sync_label_id: LabelIdSetter = {
        let context_label_id = context.label_id.clone();
        Rc::new(move |update| match update {
            LabelIdUpdate::Set(next) => {
                context_label_id.set(next.clone());
                if let Some(prop) = &set_label_id_prop {
                    prop(LabelIdUpdate::Set(next));
                }
            }
            LabelIdUpdate::ClearIfCurrent(id) => {
                if context_label_id.get_untracked() == Some(id.clone()) {
                    context_label_id.set(None);
                }
                if let Some(prop) = &set_label_id_prop {
                    prop(LabelIdUpdate::ClearIfCurrent(id));
                }
            }
        })
    };

    // `const id = useRegisteredLabelId(idProp, syncLabelId)` (`:26`).
    let id = use_registered_label_id(Signal::derive_local(move || id_prop.clone()), sync_label_id);

    // `const resolvedControlId = contextControlId ?? fallbackControlId` (`:28`).
    let resolved_control_id: Signal<Option<String>, LocalStorage> = {
        let context_control_id = context.control_id.clone();
        Signal::derive_local(move || {
            context_control_id
                .get()
                .or_else(|| fallback_control_id.clone())
        })
    };

    // `focusControl` (`:30-44`).
    let focus_control: Rc<dyn Fn(&web_sys::MouseEvent)> = {
        let resolved_control_id = resolved_control_id.clone();
        Rc::new(move |event: &web_sys::MouseEvent| {
            let resolved = resolved_control_id.get_untracked();

            if let Some(focus_control_prop) = &focus_control_prop {
                focus_control_prop(event, resolved);
                return;
            }

            let Some(control_id) = resolved else {
                return;
            };

            let document = owner_document(
                event
                    .current_target()
                    .as_ref()
                    .and_then(|target| target.dyn_ref::<web_sys::Node>()),
            );
            if let Some(control_element) = document.get_element_by_id(&control_id) {
                if is_html_element(&control_element) {
                    if let Some(element) = control_element.dyn_ref::<web_sys::HtmlElement>() {
                        focus_element_with_visible(element);
                    }
                }
            }
        })
    };

    // `handleInteraction` (`:46-62`).
    let handle_interaction: Rc<dyn Fn(&web_sys::MouseEvent)> = {
        let focus_control = Rc::clone(&focus_control);
        Rc::new(move |event: &web_sys::MouseEvent| {
            let target = get_target(event);
            if let Some(element) = target
                .as_ref()
                .and_then(|target| target.dyn_ref::<Element>())
            {
                if let Ok(Some(_)) = element.closest("button,input,select,textarea") {
                    return;
                }
            }

            // Prevent text selection when double clicking label (`:52-55`).
            if !event.default_prevented() && event.detail() > 1 {
                event.prevent_default();
            }

            if native {
                return;
            }

            focus_control(event);
        })
    };

    // The two return branches (`:64-76`).
    if native {
        LabelProps {
            id,
            for_control: resolved_control_id,
            on_mouse_down: Some(handle_interaction),
            on_click: None,
            on_pointer_down: None,
        }
    } else {
        LabelProps {
            id,
            for_control: resolved_control_id,
            on_mouse_down: None,
            on_click: Some(handle_interaction),
            on_pointer_down: Some(Rc::new(|event: &web_sys::PointerEvent| {
                event.prevent_default();
            })),
        }
    }
}

// ---------------------------------------------------------------------------
// useAriaLabelledBy.ts
// ---------------------------------------------------------------------------

/// Port of `useAriaLabelledBy`
/// (`packages/react/src/internals/labelable-provider/useAriaLabelledBy.ts:6-34`): resolves
/// `explicit ?? labelId ?? fallback`, where the fallback discovers the native `<label>`
/// associated with the control element the ref points at. Must be called inside a reactive
/// owner (a component).
///
/// `label_source_ref` is the control element ref in the crate's `HTMLProps` (crate::types) slot shape.
/// The per-commit fallback re-check (`:22-31`) re-runs on the tracked `label_id` changes;
/// the label-mount/unmount edges of the upstream every-commit semantics are discharged to
/// the view layer (module docs).
pub fn use_aria_labelled_by<L>(
    explicit_aria_labelled_by: Option<String>,
    label_id: L,
    label_source_ref: Rc<Cell<Option<Element>>>,
    enable_fallback: bool,
    label_source_id: Option<String>,
) -> Signal<Option<String>, LocalStorage>
where
    L: Get<Value = Option<String>> + Clone + 'static,
{
    // `useBaseUiId(labelSourceId ? `${labelSourceId}-label` : undefined)` (`:15`).
    let generated_label_id = use_base_ui_id(Signal::derive_local(move || {
        label_source_id
            .as_ref()
            .map(|source| format!("{source}-label"))
    }));

    // `useState<string | undefined>()` (`:13`).
    let fallback_aria_labelled_by: RwSignal<Option<String>, LocalStorage> =
        RwSignal::new_local(None);

    // The fallback effect (`:22-31`).
    {
        let explicit_aria_labelled_by = explicit_aria_labelled_by.clone();
        let label_id = label_id.clone();
        let fallback_aria_labelled_by = fallback_aria_labelled_by.clone();
        let generated = generated_label_id.clone();
        use_iso_layout_effect(move || {
            // The `||` gate is truthiness upstream (`:23-26`), not nullishness — an empty
            // string lets the fallback proceed.
            let has_explicit_or_label_id = explicit_aria_labelled_by
                .as_ref()
                .is_some_and(|id| !id.is_empty())
                || label_id.get().is_some_and(|id| !id.is_empty());
            let next_aria_labelled_by = if has_explicit_or_label_id || !enable_fallback {
                None
            } else {
                let label_source = label_source_ref.replace(None);
                label_source_ref.set(label_source.clone());
                get_aria_labelled_by(
                    label_source.as_ref(),
                    Some(generated.get_untracked()).as_deref(),
                )
            };

            if fallback_aria_labelled_by.get_untracked() != next_aria_labelled_by {
                fallback_aria_labelled_by.set(next_aria_labelled_by);
            }
        });
    }

    // The return (`:16`, `:33`) — nullish `??`, so an empty string rides through.
    Signal::derive_local(move || {
        explicit_aria_labelled_by
            .clone()
            .or_else(|| label_id.get())
            .or_else(|| fallback_aria_labelled_by.get())
    })
}

/// `getAriaLabelledBy` (`useAriaLabelledBy.ts:36-47`).
fn get_aria_labelled_by(
    label_source: Option<&Element>,
    generated_label_id: Option<&str>,
) -> Option<String> {
    let label = find_associated_label(label_source)?;

    if label.id().is_empty() {
        if let Some(generated_label_id) = generated_label_id {
            label.set_id(generated_label_id);
        }
    }

    let id = label.id();
    (!id.is_empty()).then_some(id)
}

/// `findAssociatedLabel` (`useAriaLabelledBy.ts:49-70`): the fast path before the expensive
/// `.labels` read — the wrapping parent `<label>`, then the following sibling matched by
/// `htmlFor`, then `.labels`. The upstream unchecked `as HTMLLabelElement` casts port to
/// checked casts that fall through to the next path (module docs).
fn find_associated_label(label_source: Option<&Element>) -> Option<web_sys::HtmlLabelElement> {
    let label_source = label_source?;

    // Fast path before the expensive `.labels` read (`:54-58`).
    if let Some(parent) = label_source.parent_element() {
        if parent.tag_name() == "LABEL" {
            if let Some(label) = parent.dyn_ref::<web_sys::HtmlLabelElement>() {
                return Some(label.clone());
            }
        }
    }

    // The following sibling registered for this control's id (`:60-66`).
    let control_id = label_source.id();
    if !control_id.is_empty() {
        if let Some(next_sibling) = label_source.next_element_sibling() {
            if let Some(label) = next_sibling.dyn_ref::<web_sys::HtmlLabelElement>() {
                if label.html_for() == control_id {
                    return Some(label.clone());
                }
            }
        }
    }

    // The `.labels` property read (`:68-69`) — a dynamic lookup so elements without the
    // property (upstream's duck-typed `labels?` member) yield `undefined`.
    let labels = js_sys::Reflect::get(
        label_source.as_ref(),
        &wasm_bindgen::JsValue::from_str("labels"),
    )
    .ok()
    .filter(|value| !value.is_undefined() && !value.is_null())?;
    let labels: web_sys::NodeList = labels.dyn_into().ok()?;
    let first = labels.get(0)?;
    first.dyn_into().ok()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use std::cell::RefCell;

    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{Get, GetUntracked, Set};

    use super::*;

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn static_attr(value: Option<&str>) -> ElementAttributeFn {
        let value = value.map(str::to_string);
        Rc::new(move || value.clone())
    }

    fn described_by_of(attributes: &[(String, ElementAttributeFn)]) -> Option<String> {
        attributes
            .iter()
            .find(|(name, _)| name == "aria-describedby")
            .and_then(|(_, value)| value())
    }

    // Pins the default context (`LabelableContext.ts:32-41`): the no-op registration keeps
    // a stray control inert, and the accessor falls back to it outside a provider.
    #[test]
    fn the_no_op_default_context_keeps_a_stray_control_inert() {
        let owner = in_owner();

        let context = use_labelable_context();
        assert!(
            is_noop_register_control_id(&context.register_control_id),
            "the default context hands out the shared no-op registration"
        );
        assert_eq!(context.control_id.get_untracked(), None);
        assert_eq!(context.label_id.get_untracked(), None);
        assert_eq!(context.message_ids.get_untracked(), Vec::<String>::new());

        (context.register_control_id)(ControlIdSource::new(), Some(Some("a".to_string())));
        assert_eq!(
            context.control_id.get_untracked(),
            None,
            "the no-op registration leaves the control id untouched"
        );

        owner.cleanup();
    }

    // Pins the reducer's last-registered-wins base case
    // (`LabelableProvider.tsx:43-57`): with no sticky match, the first registered value in
    // insertion order becomes the control id.
    #[test]
    fn the_first_registration_wins_until_it_leaves() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let a = ControlIdSource::new();
        let b = ControlIdSource::new();

        (context.register_control_id)(a, Some(Some("a".to_string())));
        assert_eq!(context.control_id.get_untracked(), Some("a".to_string()));

        (context.register_control_id)(b, Some(Some("b".to_string())));
        assert_eq!(
            context.control_id.get_untracked(),
            Some("a".to_string()),
            "the later registration does not churn the selection"
        );

        (context.register_control_id)(a, None);
        assert_eq!(
            context.control_id.get_untracked(),
            Some("b".to_string()),
            "the surviving registration takes over"
        );

        owner.cleanup();
    }

    // Pins the upsert's position preservation (JS `Map.set` on an existing key): a
    // re-registration updates in place and the insertion order still decides the fallback.
    #[test]
    fn a_re_registration_updates_in_place() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let a = ControlIdSource::new();
        let b = ControlIdSource::new();

        (context.register_control_id)(a, Some(Some("a".to_string())));
        (context.register_control_id)(b, Some(Some("b".to_string())));
        (context.register_control_id)(a, Some(Some("a2".to_string())));

        assert_eq!(
            context.control_id.get_untracked(),
            Some("a2".to_string()),
            "the sticky selection still names the updated registration"
        );

        (context.register_control_id)(a, None);
        assert_eq!(
            context.control_id.get_untracked(),
            Some("b".to_string()),
            "the insertion order is unchanged by the in-place update"
        );

        owner.cleanup();
    }

    // Pins the sticky-selection reducer (`:46-50`): the current selection is kept while it
    // is still registered, so newer registrations never churn it.
    #[test]
    fn the_current_selection_is_sticky_while_registered() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let a = ControlIdSource::new();
        let b = ControlIdSource::new();

        (context.register_control_id)(a, Some(Some("a".to_string())));
        assert_eq!(context.control_id.get_untracked(), Some("a".to_string()));

        // "a" is registered; a newer registration must not churn the selection.
        (context.register_control_id)(b, Some(Some("b".to_string())));
        assert_eq!(
            context.control_id.get_untracked(),
            Some("a".to_string()),
            "the sticky selection survives a newer registration"
        );

        // Once "a" leaves, the surviving registration takes over.
        (context.register_control_id)(a, None);
        assert_eq!(context.control_id.get_untracked(), Some("b".to_string()));

        owner.cleanup();
    }

    // Pins the empty-map sticky branch (`:37-41`): an empty registration map preserves the
    // current id, and only `resetControlId` restores the default (`:62-66`).
    #[test]
    fn an_empty_registration_map_preserves_the_selection_until_reset() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let default_id = context.control_id.get_untracked().expect("the default id");
        let a = ControlIdSource::new();

        (context.register_control_id)(a, Some(Some("a".to_string())));
        (context.register_control_id)(a, None);

        assert_eq!(
            context.control_id.get_untracked(),
            Some("a".to_string()),
            "the empty map preserves the selection (the hidden-subtree rule)"
        );

        (context.reset_control_id)();
        assert_eq!(
            context.control_id.get_untracked(),
            Some(default_id),
            "resetControlId restores the default id"
        );

        owner.cleanup();
    }

    // Pins the `null` suppression (`LabelableContext.ts:8-11`,
    // `LabelableProvider.tsx:33`): a registered `null` becomes the control id, and an
    // all-null map resolves to `null` (the reducer's first-value assignment).
    #[test]
    fn a_null_registration_suppresses_the_control_id() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let a = ControlIdSource::new();

        (context.register_control_id)(a, Some(None));
        assert_eq!(
            context.control_id.get_untracked(),
            None,
            "the null registration deliberately suppresses htmlFor"
        );

        owner.cleanup();
    }

    // Pins `useLabelableId`'s return outside a provider (`useLabelableId.ts:79-82` over the
    // default context): `undefined ?? id ?? defaultId`.
    #[test]
    fn a_stray_control_resolves_to_its_explicit_or_generated_id() {
        let owner = in_owner();

        let generated = use_labelable_id(UseLabelableIdParams::default());
        assert!(
            generated.get_untracked().starts_with("base-ui-"),
            "the id-less stray control falls back to the generated id"
        );

        let explicit = use_labelable_id(UseLabelableIdParams {
            id: Some("my-control".to_string()),
            enabled: true,
        });
        assert_eq!(explicit.get_untracked(), "my-control");

        owner.cleanup();
    }

    // Pins the `enabled: false` arm (`:82`): the disabled control returns its own generated
    // id (`undefined ?? id ?? defaultId` over the hook's `defaultId`) and never registers.
    #[test]
    fn a_disabled_control_returns_the_fallback_without_registering() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let default_id = context.control_id.get_untracked().expect("the default id");

        let resolved = use_labelable_id(UseLabelableIdParams {
            id: None,
            enabled: false,
        });

        assert!(
            resolved.get_untracked().starts_with("base-ui-"),
            "the disabled control resolves to its own generated id"
        );
        assert_eq!(
            context.control_id.get_untracked(),
            Some(default_id),
            "no registration happened"
        );

        owner.cleanup();
    }

    // Pins `useLabelableId`'s pre-registration return (`:79-82`): the provider's
    // pre-registration state wins over the explicit id — "the label renders `htmlFor` from
    // the provider's pre-registration state, so preempting it with an explicit `id` here
    // would leave the pair unassociated in server-rendered markup".
    #[test]
    fn the_provider_state_wins_until_registration_runs() {
        let owner = in_owner();

        let context = provide_labelable_context();
        let default_id = context.control_id.get_untracked().expect("the default id");

        let resolved = use_labelable_id(UseLabelableIdParams {
            id: Some("my-control".to_string()),
            enabled: true,
        });

        assert_eq!(
            resolved.get_untracked(),
            default_id,
            "the provider's pre-registration state wins until the registration effect runs"
        );

        owner.cleanup();
    }

    // Pins the `getDescriptionProps` merge (`LabelableProvider.tsx:68-81`): the external
    // value, the parent ids, and the own ids dedupe in first-occurrence order; an empty
    // result omits the prop; the upstream `split(' ')` keeps its exact semantics.
    #[test]
    fn the_description_merge_dedupes_external_parent_and_own_ids() {
        let owner = in_owner();

        let parent = provide_labelable_context();
        parent
            .message_ids
            .set(vec!["p1".to_string(), "shared".to_string()]);

        // A nested provider reads the parent ids (`:24`).
        let child = provide_labelable_context();
        child
            .message_ids
            .set(vec!["c1".to_string(), "shared".to_string()]);

        let mut attributes = vec![(
            "aria-describedby".to_string(),
            static_attr(Some("ext shared")),
        )];
        (child.get_description_props)(&mut attributes);

        assert_eq!(
            described_by_of(&attributes).as_deref(),
            Some("ext shared p1 c1"),
            "external first, then parent, then own; duplicates dropped at first occurrence"
        );

        owner.cleanup();
    }

    // Pins the empty-result omission (`:77`'s `|| undefined`) and the fresh-install path
    // when the external bag carried no `aria-describedby`.
    #[test]
    fn the_merge_omits_the_prop_when_nothing_resolves() {
        let owner = in_owner();

        let context = provide_labelable_context();

        let mut without_external: Vec<(String, ElementAttributeFn)> = Vec::new();
        (context.get_description_props)(&mut without_external);
        assert_eq!(
            described_by_of(&without_external),
            None,
            "no ids and no external value: the prop is omitted"
        );

        let mut blank_external = vec![("aria-describedby".to_string(), static_attr(Some(" ")))];
        (context.get_description_props)(&mut blank_external);
        assert_eq!(
            described_by_of(&blank_external),
            None,
            "a blank external split leaves only empty strings: the join is empty, the prop omitted"
        );

        owner.cleanup();
    }

    // Pins the noop default's identity `getDescriptionProps`
    // (`LabelableContext.ts:40`): the external bag passes through untouched.
    #[test]
    fn the_default_description_props_pass_through() {
        let owner = in_owner();

        let context = use_labelable_context();
        let mut attributes = vec![("aria-describedby".to_string(), static_attr(Some("ext")))];
        (context.get_description_props)(&mut attributes);

        assert_eq!(
            described_by_of(&attributes).as_deref(),
            Some("ext"),
            "the identity default does not merge anything"
        );

        owner.cleanup();
    }

    // Pins the label-store sync contract (`useLabel.ts:21-24` over `useLabel.ts:90-93`):
    // the prop setter receives the same dispatches the context setter answers — here the
    // ClearIfCurrent passthrough is exercised through the context signal directly.
    #[test]
    fn the_label_store_receives_the_context_syncs() {
        let owner = in_owner();

        let context = provide_labelable_context();

        let prop_updates = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&prop_updates);

        let sync: LabelIdSetter = {
            let context_label_id = context.label_id.clone();
            Rc::new(move |update| match update {
                LabelIdUpdate::Set(next) => {
                    context_label_id.set(next.clone());
                    sink.borrow_mut().push(("set".to_string(), next));
                }
                LabelIdUpdate::ClearIfCurrent(id) => {
                    if context_label_id.get_untracked() == Some(id.clone()) {
                        context_label_id.set(None);
                    }
                    sink.borrow_mut().push(("clear".to_string(), Some(id)));
                }
            })
        };

        sync(LabelIdUpdate::Set(Some("l1".to_string())));
        assert_eq!(context.label_id.get_untracked(), Some("l1".to_string()));

        sync(LabelIdUpdate::ClearIfCurrent("l1".to_string()));
        assert_eq!(context.label_id.get_untracked(), None);

        assert_eq!(
            prop_updates.borrow().as_slice(),
            [
                ("set".to_string(), Some("l1".to_string())),
                ("clear".to_string(), Some("l1".to_string())),
            ],
        );

        owner.cleanup();
    }

    // Pins `ControlIdSource` uniqueness — the `Symbol()` stand-in's contract.
    #[test]
    fn control_sources_are_unique() {
        let a = ControlIdSource::new();
        let b = ControlIdSource::new();
        assert_ne!(a, b);
        assert_eq!(a, a);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::Cell;

    use any_spawner::Executor;
    use reactive_graph::owner::Owner;
    use reactive_graph::traits::{Get, GetUntracked, Set};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::wasm_bindgen::JsCast;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn in_owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn make_element(tag: &str, id: &str) -> Element {
        let element = document().create_element(tag).unwrap();
        if !id.is_empty() {
            element.set_id(id);
        }
        element
    }

    // Pins the registration effect (`useLabelableId.ts:32-71`) in a real DOM realm: the
    // layout effect runs synchronously during the hook call, so the control id is
    // registered — and the return switches onto it — by the time the hook returns.
    #[wasm_bindgen_test]
    fn the_control_registers_synchronously_and_the_return_follows_the_provider() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        provide_labelable_context();
        let resolved = use_labelable_id(UseLabelableIdParams {
            id: Some("control-1".to_string()),
            enabled: true,
        });

        assert_eq!(
            resolved.get_untracked(),
            "control-1",
            "the registration ran during the hook call"
        );

        owner.cleanup();
    }

    // Pins the layout-phase unregistration (`:73-77`): owner disposal unregisters, the
    // empty map preserves the selection (the sticky rule), and resetControlId restores the
    // default.
    #[wasm_bindgen_test]
    fn disposal_unregisters_and_the_sticky_rule_holds() {
        let _ = Executor::init_futures_executor();

        let context_owner = Owner::new();
        context_owner.set();
        let context = provide_labelable_context();
        let default_id = context.control_id.get_untracked().expect("the default id");

        let control_owner = Owner::new();
        control_owner.set();
        let resolved = use_labelable_id(UseLabelableIdParams {
            id: Some("control-2".to_string()),
            enabled: true,
        });
        assert_eq!(
            context.control_id.get_untracked(),
            Some("control-2".to_string())
        );
        assert_eq!(resolved.get_untracked(), "control-2");

        control_owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            context.control_id.get_untracked(),
            Some("control-2".to_string()),
            "the empty map preserves the selection"
        );

        (context.reset_control_id)();
        assert_eq!(
            context.control_id.get_untracked(),
            Some(default_id),
            "resetControlId restores the default"
        );

        context_owner.cleanup();
    }

    // Pins the replacement handover: the sticky rule keeps the selection on the still-
    // registered outgoing control, and the replacement takes over once that registration
    // leaves (upstream's replacement commit runs the outgoing layout-phase cleanup before
    // the replacement's registration, `useLabelableId.ts:73-77`'s note).
    #[wasm_bindgen_test]
    fn a_replacement_control_takes_over_the_registration() {
        let _ = Executor::init_futures_executor();

        let context_owner = Owner::new();
        context_owner.set();
        let context = provide_labelable_context();

        let first_owner = Owner::new();
        first_owner.set();
        let _ = use_labelable_id(UseLabelableIdParams {
            id: Some("first".to_string()),
            enabled: true,
        });
        assert_eq!(
            context.control_id.get_untracked(),
            Some("first".to_string())
        );

        // The replacement registers while the first is still registered: the sticky rule
        // keeps the selection on "first".
        let second_owner = Owner::new();
        second_owner.set();
        let _ = use_labelable_id(UseLabelableIdParams {
            id: Some("second".to_string()),
            enabled: true,
        });
        assert_eq!(
            context.control_id.get_untracked(),
            Some("first".to_string()),
            "the sticky selection survives the replacement's registration"
        );

        // The outgoing registration's cleanup then hands the selection over.
        first_owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            context.control_id.get_untracked(),
            Some("second".to_string()),
            "the replacement takes over once the outgoing registration leaves"
        );

        second_owner.cleanup();
        context_owner.cleanup();
    }

    // Pins `useLabel`'s native branch (`useLabel.ts:64-69`): `htmlFor` resolves the
    // registered control, and the mousedown interaction returns before the focus pipeline
    // (`:57-59`) — the browser's own label behavior owns native focusing.
    #[wasm_bindgen_test]
    fn a_native_label_mousedown_defers_to_the_browser_and_resolves_htmlfor() {
        let _ = Executor::init_futures_executor();

        let owner = in_owner();
        provide_labelable_context();

        let control = make_element("input", "the-control");
        document().body().unwrap().append_child(&control).unwrap();

        // The control registers its id with the provider.
        let _ = use_labelable_id(UseLabelableIdParams {
            id: Some("the-control".to_string()),
            enabled: true,
        });

        let params = UseLabelParams {
            native: true,
            ..UseLabelParams::default()
        };
        let props = use_label(params);
        assert_eq!(
            props.for_control.get_untracked(),
            Some("the-control".to_string()),
            "htmlFor resolves the registered control id"
        );
        assert!(
            props.on_click.is_none(),
            "the native branch fills only mousedown"
        );

        // Focus the control, then run the interaction with a cancelable event: the native
        // branch must leave focus untouched.
        control
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .focus()
            .unwrap();

        let mut init = web_sys::MouseEventInit::new();
        init.set_cancelable(true);
        let mouse_event =
            web_sys::MouseEvent::new_with_mouse_event_init_dict("mousedown", &init).unwrap();

        let handler = props
            .on_mouse_down
            .expect("the native branch fills mousedown");
        handler(&mouse_event);

        assert_eq!(
            document().active_element().map(|el| el.id()),
            Some("the-control".to_string()),
            "the native branch returns before the focus pipeline (the browser owns it)"
        );

        owner.cleanup();
    }

    // Pins the non-native branch (`useLabel.ts:70-76`): clicking focuses the control
    // through the real dispatch path (attached listener, `currentTarget` set), and the
    // pointerdown slot prevents the default (the text-selection suppression).
    #[wasm_bindgen_test]
    fn a_non_native_label_focuses_on_click_and_prevents_pointerdown() {
        let _ = Executor::init_futures_executor();

        let owner = in_owner();

        let control = make_element("input", "other-control");
        document().body().unwrap().append_child(&control).unwrap();

        // Outside a provider: the fallback control id resolves the focus target
        // (`useLabel.ts:28`, `:81-84`).
        let params = UseLabelParams {
            fallback_control_id: Some("other-control".to_string()),
            ..UseLabelParams::default()
        };
        let props = use_label(params);
        assert!(props.on_mouse_down.is_none());
        assert!(props.on_click.is_some());
        assert!(props.on_pointer_down.is_some());

        let span = document()
            .create_element("span")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&span).unwrap();

        // Attach the click interaction and dispatch a real click so `currentTarget` is the
        // span (the `ownerDocument(currentTarget)` control lookup).
        let handler = props.on_click.clone().unwrap();
        let unsubscribe = leptos_ui_utils::add_event_listener(&span, "click", move |event| {
            if let Some(mouse_event) = event.dyn_ref::<web_sys::MouseEvent>() {
                handler(mouse_event);
            }
        });

        let click = web_sys::MouseEvent::new("click").unwrap();
        span.dispatch_event(click.as_ref()).unwrap();

        assert_eq!(
            document().active_element().map(|el| el.id()),
            Some("other-control".to_string()),
            "the click focused the fallback control"
        );
        unsubscribe.unsubscribe();

        // The pointerdown slot prevents the default on a cancelable event.
        let mut init = web_sys::PointerEventInit::new();
        init.set_cancelable(true);
        if let Some(handler) = &props.on_pointer_down {
            let raw =
                web_sys::PointerEvent::new_with_event_init_dict("pointerdown", &init).unwrap();
            handler(&raw);
            assert!(raw.default_prevented(), "the slot prevents the default");
        }

        owner.cleanup();
    }

    // Pins the nested-form-control yield (`useLabel.ts:46-50`): an interaction whose real
    // dispatch target sits inside a button/input/select/textarea returns before the focus
    // pipeline — exercised through a real dispatch so `getTarget` sees the inner target.
    #[wasm_bindgen_test]
    fn an_interaction_on_a_nested_form_control_yields() {
        let _ = Executor::init_futures_executor();

        let owner = in_owner();

        let control = make_element("input", "target-control");
        document().body().unwrap().append_child(&control).unwrap();
        control
            .dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .focus()
            .unwrap();

        let params = UseLabelParams {
            fallback_control_id: Some("target-control".to_string()),
            ..UseLabelParams::default()
        };
        let props = use_label(params);

        // A wrapper label carrying the handler, with a button between it and the target:
        // the dispatch target is the inner span, whose `closest` hits the button.
        let wrapper = document()
            .create_element("label")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        let button = make_element("button", "");
        let inner = make_element("span", "");
        button.append_child(&inner).unwrap();
        wrapper.append_child(&button).unwrap();
        document().body().unwrap().append_child(&wrapper).unwrap();

        let handler = props.on_click.clone().unwrap();
        let unsubscribe = leptos_ui_utils::add_event_listener(&wrapper, "click", move |event| {
            if let Some(mouse_event) = event.dyn_ref::<web_sys::MouseEvent>() {
                handler(mouse_event);
            }
        });

        let mut init = web_sys::MouseEventInit::new();
        init.set_cancelable(true);
        init.set_bubbles(true);
        let click = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
        inner.dispatch_event(click.as_ref()).unwrap();

        assert!(
            !click.default_prevented(),
            "the nested-control target returned before the pipeline"
        );
        assert_eq!(
            document().active_element().map(|el| el.id()),
            Some("target-control".to_string()),
            "focus did not move"
        );

        unsubscribe.unsubscribe();
        owner.cleanup();
    }

    // Pins the double-click text-selection suppression (`useLabel.ts:52-55`): a
    // non-prevented event with `detail > 1` gets its default prevented.
    #[wasm_bindgen_test]
    fn a_double_click_gets_its_default_prevented() {
        let _ = Executor::init_futures_executor();

        let owner = in_owner();

        let params = UseLabelParams {
            native: true,
            ..UseLabelParams::default()
        };
        let props = use_label(params);

        if let Some(handler) = &props.on_mouse_down {
            let mut init = web_sys::MouseEventInit::new();
            init.set_cancelable(true);
            init.set_detail(2);
            let raw =
                web_sys::MouseEvent::new_with_mouse_event_init_dict("mousedown", &init).unwrap();
            handler(&raw);
            assert!(raw.default_prevented(), "detail > 1 prevents the default");

            let mut single_init = web_sys::MouseEventInit::new();
            single_init.set_cancelable(true);
            let single =
                web_sys::MouseEvent::new_with_mouse_event_init_dict("mousedown", &single_init)
                    .unwrap();
            handler(&single);
            assert!(!single.default_prevented(), "detail == 1 is left alone");
        }

        owner.cleanup();
    }

    // Pins the wrapping-parent fast path of the `useAriaLabelledBy` fallback
    // (`useAriaLabelledBy.ts:54-58`): a control span inside a native `<label>` resolves the
    // label's id; an id-less label receives the generated `${labelSourceId}-label` id
    // (`:15`, `:42-44`).
    #[wasm_bindgen_test]
    fn the_fallback_discovers_a_wrapping_label() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let label = document()
            .create_element("label")
            .unwrap()
            .dyn_into::<web_sys::HtmlLabelElement>()
            .unwrap();
        let control = make_element("span", "the-span");
        label.append_child(&control).unwrap();
        document().body().unwrap().append_child(&label).unwrap();

        let label_source_ref: Rc<Cell<Option<Element>>> = Rc::new(Cell::new(None));
        label_source_ref.set(Some(control.clone()));

        let aria_labelled_by = use_aria_labelled_by(
            None,
            Signal::derive_local(|| None::<String>),
            label_source_ref,
            true,
            Some("my-control".to_string()),
        );

        assert_eq!(
            aria_labelled_by.get_untracked(),
            Some("my-control-label".to_string()),
            "the id-less wrapping label received the generated id"
        );
        assert_eq!(label.id(), "my-control-label");

        owner.cleanup();
    }

    // Pins the sibling fast path (`useAriaLabelledBy.ts:60-66`) and the id-preserving arm
    // (`:46`): a following `<label for=...>` sibling resolves by its `htmlFor`, and a label
    // that already has an id keeps it.
    #[wasm_bindgen_test]
    fn the_fallback_discovers_a_sibling_label_with_htmlfor() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let control = make_element("span", "the-span");
        let label = document()
            .create_element("label")
            .unwrap()
            .dyn_into::<web_sys::HtmlLabelElement>()
            .unwrap();
        label.set_html_for("the-span");
        label.set_id("sibling-label");
        document().body().unwrap().append_child(&control).unwrap();
        document().body().unwrap().append_child(&label).unwrap();

        let label_source_ref: Rc<Cell<Option<Element>>> = Rc::new(Cell::new(None));
        label_source_ref.set(Some(control.clone()));

        let aria_labelled_by = use_aria_labelled_by(
            None,
            Signal::derive_local(|| None::<String>),
            label_source_ref,
            true,
            None,
        );

        assert_eq!(
            aria_labelled_by.get_untracked(),
            Some("sibling-label".to_string()),
            "the sibling label's own id wins over the generated one"
        );

        owner.cleanup();
    }

    // Pins the precedence chain (`useAriaLabelledBy.ts:16`, `:23-26`) and the disable arm:
    // an explicit `aria-labelledby` or a context label id short-circuits the fallback, and
    // `enableFallback: false` forces `undefined`.
    #[wasm_bindgen_test]
    fn the_explicit_and_label_ids_short_circuit_the_fallback() {
        let _ = Executor::init_futures_executor();

        let explicit_owner = Owner::new();
        explicit_owner.set();
        let control = make_element("span", "");
        document().body().unwrap().append_child(&control).unwrap();
        let label_source_ref: Rc<Cell<Option<Element>>> = Rc::new(Cell::new(None));
        label_source_ref.set(Some(control.clone()));
        let explicit = use_aria_labelled_by(
            Some("explicit".to_string()),
            Signal::derive_local(|| None::<String>),
            label_source_ref,
            true,
            None,
        );
        assert_eq!(explicit.get_untracked(), Some("explicit".to_string()));
        explicit_owner.cleanup();

        let disabled_owner = Owner::new();
        disabled_owner.set();
        let label_source_ref: Rc<Cell<Option<Element>>> = Rc::new(Cell::new(None));
        label_source_ref.set(Some(control.clone()));
        let disabled = use_aria_labelled_by(
            None,
            Signal::derive_local(|| None::<String>),
            label_source_ref,
            false,
            None,
        );
        assert_eq!(
            disabled.get_untracked(),
            None,
            "enableFallback: false suppresses the DOM discovery"
        );
        disabled_owner.cleanup();
    }

    // Pins the tracked re-run of the fallback effect (module docs): a `labelId` change
    // re-runs the resolution, and the return follows `explicit ?? labelId ?? fallback`.
    #[wasm_bindgen_test]
    fn a_label_id_change_re_runs_the_resolution() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let control = make_element("span", "");
        document().body().unwrap().append_child(&control).unwrap();
        let label_source_ref: Rc<Cell<Option<Element>>> = Rc::new(Cell::new(None));
        label_source_ref.set(Some(control.clone()));

        let label_id: RwSignal<Option<String>, LocalStorage> = RwSignal::new_local(None);
        let aria_labelled_by = use_aria_labelled_by(None, label_id, label_source_ref, true, None);
        assert_eq!(
            aria_labelled_by.get_untracked(),
            None,
            "no associated label in the DOM: the fallback is None"
        );

        label_id.set(Some("from-context".to_string()));
        Executor::poll_local();
        assert_eq!(
            aria_labelled_by.get_untracked(),
            Some("from-context".to_string()),
            "the context label id takes over"
        );

        owner.cleanup();
    }

    // Pins the `useRegisteredLabelId` → context `labelId` wiring end-to-end
    // (`useLabel.ts:26`): rendering a label registers its id with the provider, so the
    // control's `aria-labelledby` can resolve through the label-id chain.
    #[wasm_bindgen_test]
    fn a_rendered_label_registers_its_id_with_the_provider() {
        let _ = Executor::init_futures_executor();

        let context_owner = Owner::new();
        context_owner.set();
        let context = provide_labelable_context();

        let label_owner = Owner::new();
        label_owner.set();
        let params = UseLabelParams {
            ..UseLabelParams::default()
        };
        let props = use_label(params);

        assert_eq!(
            context.label_id.get_untracked(),
            Some(props.id.get_untracked()),
            "the label id is registered with the provider"
        );

        label_owner.cleanup();
        Executor::poll_local();
        assert_eq!(
            context.label_id.get_untracked(),
            None,
            "the unmount cleanup cleared the registration"
        );

        context_owner.cleanup();
    }

    // Pins the `aria-describedby` merger against the lazy attribute read in a real bag
    // (`LabelableProvider.tsx:68-81`): the entry re-reads `messageIds` at attribute-read
    // time.
    #[wasm_bindgen_test]
    fn the_description_entry_re_reads_message_ids_lazily() {
        let _ = Executor::init_futures_executor();
        let owner = in_owner();

        let context = provide_labelable_context();

        let mut attributes: Vec<(
            String,
            crate::floating_ui::element_props::ElementAttributeFn,
        )> = Vec::new();
        (context.get_description_props)(&mut attributes);
        assert_eq!(attributes.len(), 1);

        context.message_ids.set(vec!["m1".to_string()]);
        let (_, value) = &attributes[0];
        assert_eq!(value(), Some("m1".to_string()));

        context
            .message_ids
            .set(vec!["m1".to_string(), "m2".to_string()]);
        assert_eq!(value(), Some("m1 m2".to_string()), "the lazy entry re-read");

        owner.cleanup();
    }
}
