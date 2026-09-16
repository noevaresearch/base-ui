//! Port of the Base UI Form — the `library: form` TODO item
//! (`specs/library/form/behavior.md`, `specs/library/form/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **Form is deliberately thin** (implementation.md header; `Form.tsx` is 251 lines):
//!   it owns only the submit orchestration, the error mirror, and the field registry
//!   handle — per-field state lives in the `field` package, which pulls Form's refs
//!   down through `FormContext` and pushes validity back up into the registry.
//! - **The field registry is a ref, not state** (`Form.tsx:36-38`): mount/unmount and
//!   validity updates must never re-render the `<form>`. The port's registry is
//!   [`leptos_ui_internals::form_context::FormRef`] — the insertion-ordered
//!   `Rc<RefCell<FormState>>` the internals unit already defines and the field package
//!   already writes ([`crate::field`], done-marked) — so the port's Form is the
//!   *provider* side of an existing, integration-tested contract rather than a second
//!   registry implementation.
//! - **The error mirror** (`:73-77`): `errors` is "controlled with an internal mirror"
//!   — `React.useState(externalErrors)` seeds local state from the prop,
//!   `useValueChanged(externalErrors, …)` re-syncs on prop *identity* change, and
//!   `clearErrors` (`:145-157`) is the only local writer (`Object.hasOwn` so
//!   prototype-less records work; an absent key returns `previousErrors` unchanged).
//! - **`submittedRef` + post-submit effect** (`:79-86`): the focus-on-invalid-submit
//!   behavior is split in two — `focusFirstInvalid()` runs synchronously inside the
//!   submit handler for validator failures, while `submittedRef` is set right before
//!   `onSubmit` fires (`:124`) and a `useEffect` keyed on `[errors, focusFirstInvalid]`
//!   consumes the flag exactly once for errors that commit *after* submit (async server
//!   errors). That split is what makes external-error focus happen "only on submit,
//!   never on value change" (behavior.md "Focus management").
//! - **`focusFirstInvalid`** (`:43-71`): iterates the registry in insertion order, and
//!   among invalid fields picks the control first by **document position** via
//!   `comesBeforeInSameTree` (`:244-251`, `compareDocumentPosition` with the
//!   `DOCUMENT_POSITION_DISCONNECTED` bit masked out, so disconnected trees degrade to
//!   registration order). It returns *tri-state-ish*: `true` when it focused, otherwise
//!   `hasInvalid` — so submission stays blocked even when no invalid field has a usable
//!   control (`:63-70`). `select()` is applied only when `tagName === 'INPUT'`
//!   (`:65-67`).
//! - **The submit pipeline** (`:111-138`): bump `submitCountRef` → run every
//!   registered field's `validate()` synchronously → if `focusFirstInvalid()` returns
//!   truthy, `preventDefault()` and stop → else set `submittedRef`, call
//!   `onSubmit` → if `onFormSubmit` is present, `preventDefault()`, project
//!   `{ [name]: field.getValue() }` from the registry (unnamed fields skipped,
//!   `:130-135`), and call it with
//!   [`createGenericEventDetails`]`(REASONS.none, event.nativeEvent)` (`:137`) — which
//!   is why `eventDetails.event.defaultPrevented` is `true` in behavior.md's Events
//!   section.
//! - **`actionsRef`** (`:88-104`): the imperative handle is allocated once over an
//!   empty dep array; `validate(name)` scans the registry by `field.name === name`
//!   with `Array.find` (first match) and `validate()` iterates all entries. Its
//!   closures call the registry entries' own `validate`, so the handle stays correct
//!   across rename/unmount/replacement without re-binding.
//! - **`useRenderElement('form', …)`** (`:106-143`): the standard pipeline with
//!   `[forwardedRef, elementRef]` as the ref fork and `[{ noValidate: true, onSubmit },
//!   elementProps]` as the props bags — the Form-injected props first, so a
//!   user-supplied `noValidate={false}` overrides the default (behavior.md "DOM
//!   structure").
//! - **`FormContext` is provided, never `useControlled`** (implementation.md
//!   "Context providers/consumers", `:159-171`): six members cross the boundary
//!   (`formRef`, `errors`, `clearErrors`, `elementRef`, `validationMode`,
//!   `submitCountRef`), and Form renders `element` *inside* the provider.
//! - **No scroll-into-view anywhere** (implementation.md "DOM/portal strategy", and
//!   behavior.md's UNVERIFIED focus note): the port likewise never scrolls.
//!
//! ## Rust adaptations
//!
//! - **The provider rides the internals' rg-0.2 owner bridge** (the
//!   `field_root.rs`/`field_parts.rs` bridge-window precedent): `FormContextValue` is
//!   internals' bag read through [`leptos_ui_internals::form_context::use_form_context`]
//!   — an rg-0.2 `use_context` — so Form publishes it inside a fresh
//!   `reactive_graph::owner::Owner` window (`Owner::with`, forgotten so the context
//!   outlives the subtree) and builds the children *inside* that window. `Owner::with`
//!   swaps only the reactive-graph thread-local, so the leptos owner chain — and every
//!   leptos `Effect` the subtree creates — stays live.
//! - **The `<form>` element is hand-rolled, not engine-materialized** (the `progress.rs`
//!   convention): `useRenderElement`'s port has **no children channel** (its
//!   [`leptos_ui_internals::use_render_element::RenderedElement`] carries `inner_html`
//!   only) and its handler vocabulary has **no `onSubmit` slot**, while Form needs both
//!   — it renders exactly one element with the whole parts subtree *inside* it, and it
//!   injects the submit handler ahead of `elementProps` (`:108-142`). The port keeps the
//!   precedence contract by hand: the `novalidate` default is rendered statically and
//!   the user's `...elementProps` rest rides the mount effect as the later-wins bag (the
//!   `field_root.rs` `element_attributes` channel). The submit listener has no engine
//!   handler slot at all, so it rides the ref-attach — the same documented seam the
//!   `otp_field.rs` port uses for `input`/`paste`.
//! - **`errors` is a plain prop plus a documented late-writer handle.** Upstream's
//!   prop-identity `useValueChanged` sync (`:75-77`) has no observable analog in the
//!   port's static-prop model (a prop change is a subtree rebuild — the `progress.rs`
//!   note), so the mirror is seeded from the prop at body time and
//!   [`FormErrorsHandle`] is the port's channel for the *asynchronous* case
//!   (behavior.md's server-error-arrives-after-submit path): it writes the same mirror
//!   `clearErrors` mutates **and** runs the post-submit focus hook (`:79-86`). Both
//!   mirror writers discharge that hook directly rather than through a reactive effect:
//!   an rg-0.2 `Effect` needs a global executor this workspace does not initialize, and a
//!   leptos `Effect` tracks only the leptos (rg-0.1) graph, so it could never observe a
//!   mirror write — the same dual-runtime wall the meter/button iterations recorded.
//! - **`focus()`/`select()` ride the host seam** (the `combobox::clear` convention): an
//!   `Element` focus is wasm-only, so the host arm no-ops and the host tests observe the
//!   *decision* (blocked / has-invalid); the wasm suite asserts the real
//!   `document.activeElement`.
//! - **Validators are snapshotted before they run.** A registered entry's `validate()`
//!   re-enters the registry through `form_ref.borrow_mut()` to write its merged
//!   validity back (`field/validation.rs:409-413`, mirroring
//!   `useFieldValidation.ts:136-149`), so the port collects the closures out of the
//!   borrow before invoking them — holding the borrow across the call would panic on
//!   the first real validation.
//! - **`Form.Values`/`Errors` port to insertion-ordered vecs of pairs** (the internals'
//!   `FormErrors` convention — JS string-key object order), and `getValue`'s JS
//!   `unknown` is [`serde_json::Value`] (`undefined` collapsed onto `Null`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::owner::Owner;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get as RgGet, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, HtmlFormElement};

use leptos_ui_internals::create_base_ui_event_details::BaseUIGenericEventDetails;
use leptos_ui_internals::floating_ui::reasons::NONE as REASONS_NONE;
use leptos_ui_internals::form_context::{FormContextValue, FormRef, FormState, SharedFormContext};
use leptos_ui_utils::add_event_listener::{EventListenerUnsubscribe, add_event_listener};
use leptos_ui_utils::use_merged_refs::RefCallback;

/// The context vocabulary this unit provides/**consumes** — re-exported so a caller can
/// build the `errors` record and pick a validation mode without reaching into the
/// internals crate (the `field` module's re-export convention).
pub use leptos_ui_internals::form_context::{FormErrorValue, FormErrors, FormValidationMode};

// ---------------------------------------------------------------------------
// Public vocabulary (`Form.Values`, `Form.Actions`, `Form.SubmitEventDetails`)
// ---------------------------------------------------------------------------

/// Upstream's `Form.Values<FormValues>` (`Form.tsx:241`) — the submitted values record
/// keyed by field name. Ported as an insertion-ordered vec of pairs (the internals'
/// `FormErrors` convention: JS string-key object order is observable through
/// `onFormSubmit`'s iteration order).
pub type FormValues = Vec<(String, Value)>;

/// `Form.SubmitEventDetails` (`Form.tsx:181`): `BaseUIGenericEventDetails` over the
/// native submit event. Its `reason` is always `REASONS.none` — the only submit reason
/// Form emits (`:137`, re-exported at `:180`).
///
/// The event type is [`web_sys::Event`], not `SubmitEvent`: upstream hands the consumer
/// React's `FormEvent<HTMLFormElement>`, which likewise exposes no `submitter` — and
/// carrying the plain event keeps the injected handler working for *any* dispatched
/// submit (a `requestSubmit()`/button submit and an explicitly dispatched event alike),
/// which is what upstream's `onSubmit` observes.
pub type FormSubmitEventDetails = BaseUIGenericEventDetails<(), Event>;

/// `Form.Actions` (`Form.tsx:185-187`): the imperative handle behind `actionsRef`.
#[derive(Clone)]
pub struct FormActions {
    /// `validate(fieldName?)` (`:91-100`) — the live registry scan, captured once.
    validate: Rc<dyn Fn(Option<&str>)>,
}

impl FormActions {
    /// Builds the handle over the registry scan (`:88-104`).
    pub fn new(validate: Rc<dyn Fn(Option<&str>)>) -> Self {
        Self { validate }
    }

    /// `actionsRef.current?.validate()` / `validate('email')` (`:91-100`): with a name,
    /// the first registry entry whose `name` matches (insertion order — `Array.find`);
    /// without one, every entry.
    pub fn validate(&self, field_name: Option<&str>) {
        (self.validate)(field_name);
    }
}

/// `actionsRef` (`:229`) — upstream's `React.RefObject<Form.Actions | null>`. The crate's
/// writable ref-slot shape (`types.rs` convention): Form writes the handle into the slot
/// at materialization and the caller reads it from its own copy.
pub type FormActionsRef = Rc<Cell<Option<Rc<FormActions>>>>;

/// The port's channel for errors that arrive **after** submit (behavior.md "Events",
/// "Focus management": external errors set asynchronously still trigger the focus).
///
/// Upstream delivers those by changing the `errors` prop, which the `useValueChanged`
/// sync (`Form.tsx:75-77`) mirrors and whose commit the `[errors]` effect (`:79-86`)
/// observes. The port's props are plain values, so the mirror is seeded from the prop at
/// body time (the `progress.rs` static-prop note) and this handle is the delivery point
/// for late errors: it writes the same mirror `clearErrors` mutates **and** runs the
/// post-submit focus hook, so a late commit behaves exactly like upstream's.
///
/// It is a documented seam, not hidden behavior: the handle is inert until [`Form`] binds
/// its mirror and hook to it.
#[derive(Clone, Default)]
pub struct FormErrorsHandle(Rc<RefCell<Option<FormErrorsBinding>>>);

/// The write side [`FormErrorsHandle`] carries once bound: the mirror to write, and the
/// post-submit commit hook (`:79-86`) that runs with it.
#[derive(Clone)]
struct FormErrorsBinding {
    /// `:73`'s local error state — the record `FormContext.errors` publishes.
    mirror: RwSignal<FormErrors>,
    /// The binding's write path: set the mirror, then run the commit hook.
    write: Rc<dyn Fn(FormErrors)>,
}

impl FormErrorsHandle {
    /// Creates an unbound handle (binding happens when it is passed to [`Form`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds the Form's mirror and its post-submit commit hook to this handle.
    pub(crate) fn bind(&self, mirror: RwSignal<FormErrors>, write: Rc<dyn Fn(FormErrors)>) {
        *self.0.borrow_mut() = Some(FormErrorsBinding { mirror, write });
    }

    /// Whether a mounted [`Form`] has bound its mirror.
    pub fn is_bound(&self) -> bool {
        self.0.borrow().is_some()
    }

    /// Delivers externally-produced errors — the port's stand-in for upstream's `errors`
    /// prop change (the server-action case). A no-op before binding.
    pub fn set(&self, errors: FormErrors) {
        if let Some(binding) = self.0.borrow().clone() {
            (binding.write)(errors);
        }
    }

    /// Reads the current mirror record — the observable state behind
    /// `FormContext.errors`.
    pub fn get_untracked(&self) -> Option<FormErrors> {
        self.0
            .borrow()
            .as_ref()
            .map(|binding| binding.mirror.get_untracked())
    }
}

/// `clearErrors`' pruning rule (`Form.tsx:149-156`): `None` when the record is unchanged —
/// the source's `previousErrors` identity return, taken when the key is absent — and
/// `Some(next)` with the key deleted otherwise.
///
/// `Object.hasOwn` (`:150`) has no Rust counterpart: a JS own-property check guards
/// against prototype keys, while the port's insertion-ordered record carries only real
/// keys, so "present" is simply "a key equals the name".
pub(crate) fn prune_error(errors: FormErrors, name: &str) -> Option<FormErrors> {
    if !errors.iter().any(|(key, _)| key == name) {
        return None;
    }
    Some(errors.into_iter().filter(|(key, _)| key != name).collect())
}

#[cfg(test)]
thread_local! {
    /// Diagnostic slot: how many times [`Form`]'s mount effect body ran. The wasm suite
    /// reads it to tell "the effect never ran" (a scheduling fact) apart from "the effect
    /// ran but the node was not there yet" (a ref-order fact).
    pub static MOUNT_EFFECT_RUNS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

// ---------------------------------------------------------------------------
// The registry-driven pipeline (extracted so the host suite can exercise it)
// ---------------------------------------------------------------------------

/// `comesBeforeInSameTree(element, reference)` (`Form.tsx:244-251`): the
/// `compareDocumentPosition` probe with the `DOCUMENT_POSITION_DISCONNECTED` bit masked
/// out (nodes in disconnected trees are unorderable, so the caller keeps registration
/// order for them) and the `DOCUMENT_POSITION_FOLLOWING` bit required.
pub(crate) fn comes_before_in_same_tree(element: &Element, reference: &Element) -> bool {
    let position = element.compare_document_position(reference);
    // `Node.DOCUMENT_POSITION_DISCONNECTED` = 1, `DOCUMENT_POSITION_FOLLOWING` = 4.
    const DISCONNECTED: u16 = 1;
    const FOLLOWING: u16 = 4;
    (position & DISCONNECTED) == 0 && (position & FOLLOWING) != 0
}

thread_local! {
    /// The host arm's recording slot for the focus seam (the `combobox::clear` `plan`
    /// convention): an `Element::focus()` is wasm-only, so the host build records which
    /// control the port *would* have focused, and the host suite asserts that decision
    /// without a DOM. The wasm arm performs the real `focus()`/`select()` pair and the
    /// browser suite asserts `document.activeElement`.
    static FOCUS_RECORD: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// The host-side focus record (empty on wasm — the browser synthesizes the real effect).
#[cfg(test)]
pub(crate) fn focus_record() -> Vec<String> {
    FOCUS_RECORD.with(|slot| slot.borrow().clone())
}

/// The host-side focus record's reset (the `plan` suites' per-test hygiene).
#[cfg(test)]
pub(crate) fn clear_focus_record() {
    FOCUS_RECORD.with(|slot| slot.borrow_mut().clear());
}

/// `firstControl.focus()` + the `tagName === 'INPUT'` → `.select()` pair (`Form.tsx:64-67`).
fn focus_control(control: &Element) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(html) = control.dyn_ref::<web_sys::HtmlElement>() {
            let _ = html.focus();
            // `(firstControl as HTMLInputElement).select()` — guarded by the same
            // `tagName === 'INPUT'` check upstream uses (`:65-67`).
            if control.tag_name() == "INPUT" {
                if let Some(input) = control.dyn_ref::<web_sys::HtmlInputElement>() {
                    let _ = input.select();
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        FOCUS_RECORD.with(|slot| {
            slot.borrow_mut().push(
                control
                    .get_attribute("data-testid")
                    .unwrap_or_else(|| control.tag_name()),
            );
        });
    }
}

/// `focusFirstInvalid()` (`Form.tsx:43-71`) — returns `true` when it focused a control,
/// otherwise `hasInvalid` (so a field that is invalid *without* a usable control still
/// blocks the submit, `:63-70`).
///
/// Reads the registry in insertion order and keeps the first control by document
/// position; disconnected trees keep registration order (`comesBeforeInSameTree`'s
/// mask).
pub(crate) fn focus_first_invalid(registry: &FormRef) -> bool {
    let mut has_invalid = false;
    let mut first_control: Option<Element> = None;

    for (_, field) in registry.borrow().fields.iter() {
        // `field.validityData.state.valid !== false → continue` (`:54-56`): only an
        // explicit `valid: false` counts as invalid (unvalidated `null` does not).
        if field.validity_data.state.valid != Some(false) {
            continue;
        }
        has_invalid = true;
        let Some(control) = crate::field::validation::cell_peek(&field.control_ref) else {
            continue;
        };
        let take = match &first_control {
            None => true,
            Some(current) => comes_before_in_same_tree(&control, current),
        };
        if take {
            first_control = Some(control);
        }
    }

    match first_control {
        Some(control) => {
            focus_control(&control);
            true
        }
        None => has_invalid,
    }
}

/// The `submitCountRef` bump + the synchronous validate-all + the gate (`:112-121`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SubmitGate {
    /// `focusFirstInvalid()` was truthy — the caller must `preventDefault()` and stop.
    Blocked,
    /// `focusFirstInvalid()` was falsey — the submit proceeds.
    Allowed,
}

/// Bumps `submitCountRef` (shared through context — `:112`, `:15`) and runs every
/// registered field's `validate()` synchronously (`:115-117`).
///
/// The validators are collected out of the registry borrow first: a registered
/// `validate()` writes its merged validity back through `form_ref.borrow_mut()`
/// (`field/validation.rs:409-413`), so invoking them under the iterator's borrow would
/// panic.
pub(crate) fn run_validation_gate(registry: &FormRef, submit_count: &Cell<u32>) -> SubmitGate {
    submit_count.set(submit_count.get() + 1);

    let validators: Vec<Rc<dyn Fn()>> = registry
        .borrow()
        .fields
        .iter()
        .map(|(_, field)| Rc::clone(&field.validate))
        .collect();
    for validate in validators {
        validate();
    }

    if focus_first_invalid(registry) {
        SubmitGate::Blocked
    } else {
        SubmitGate::Allowed
    }
}

/// The `onFormSubmit` values projection (`:130-135`): every registered entry with a
/// `name`, in registry (registration) order, mapped to its live `getValue()`. Unnamed
/// entries are skipped; a JS `undefined` return collapses onto
/// [`Value::Null`] (the internals' `GetFieldValueFn` convention).
pub(crate) fn project_values(registry: &FormRef) -> FormValues {
    registry
        .borrow()
        .fields
        .iter()
        .filter_map(|(_, field)| {
            let name = field.name.clone()?;
            Some((name, (field.get_value)().unwrap_or(Value::Null)))
        })
        .collect()
}

/// `actionsRef`'s `validate(fieldName?)` (`:91-100`) over the live registry: `Some(name)`
/// re-validates the FIRST entry whose name matches (`Array.find`, insertion order);
/// `None` re-validates every entry.
///
/// Same snapshot rule as [`run_validation_gate`] — no borrow is held across a validator.
pub(crate) fn validate_registry(registry: &FormRef, field_name: Option<&str>) {
    let validators: Vec<Rc<dyn Fn()>> = match field_name {
        Some(name) => registry
            .borrow()
            .fields
            .iter()
            .find(|(_, field)| field.name.as_deref() == Some(name))
            .map(|(_, field)| vec![Rc::clone(&field.validate)])
            .unwrap_or_default(),
        None => registry
            .borrow()
            .fields
            .iter()
            .map(|(_, field)| Rc::clone(&field.validate))
            .collect(),
    };
    for validate in validators {
        validate();
    }
}

/// The shared, `Send`-able state the mount effect and the submit listener need (the
/// `SendWrapper` bridge: leptos `Effect` closures demand `Send + Sync`, and every member
/// here is an `Rc`/`Cell` that only ever crosses that boundary on the single UI thread —
/// the `avatar/views.rs` convention).
struct FormPipelineState {
    registry: FormRef,
    submit_count: Rc<Cell<u32>>,
    submitted: Rc<Cell<bool>>,
    element_ref: Rc<Cell<Option<HtmlFormElement>>>,
    on_submit: Option<Rc<dyn Fn(&Event)>>,
    on_form_submit: Option<Rc<dyn Fn(FormValues, FormSubmitEventDetails)>>,
    /// `actionsRef` (`:229`) — the slot the handle is written into at materialization.
    actions_ref: Option<FormActionsRef>,
    element_attributes: Vec<(String, String)>,
    ref_callback: Option<RefCallback<HtmlFormElement>>,
    /// The injected submit listener's unsubscribe slot (`:111-139`) — attached once, at
    /// materialization.
    subscription: RefCell<Option<EventListenerUnsubscribe>>,
    listener_attached: Cell<bool>,
}

/// The injected submit handler (`:111-139`), end to end:
/// bump + validate + gate → `preventDefault` when blocked → `submittedRef = true` →
/// `onSubmit?.(event)` → when `onFormSubmit` exists: `preventDefault`, project the
/// values, and call it with `createGenericEventDetails(REASONS.none, event)`.
fn handle_submit(state: &FormPipelineState, event: &Event) {
    match run_validation_gate(&state.registry, &state.submit_count) {
        SubmitGate::Blocked => {
            // `:119-121`
            event.prevent_default();
        }
        SubmitGate::Allowed => {
            // `:124` — the flag the post-submit focus effect consumes.
            state.submitted.set(true);
            // The user's native `onSubmit` (`:125`) — upstream's `onSubmit?.(event as any)`.
            if let Some(on_submit) = &state.on_submit {
                on_submit(event);
            }
            // `:127-138` — only when `onFormSubmit` exists does Form prevent the native
            // default itself (implementation.md untested item 2).
            if let Some(on_form_submit) = &state.on_form_submit {
                event.prevent_default();
                let values = project_values(&state.registry);
                on_form_submit(
                    values,
                    BaseUIGenericEventDetails::new(REASONS_NONE, event.clone(), ()),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The component
// ---------------------------------------------------------------------------

/// `Form` (`Form.tsx:21-178`) — the native `<form>` that coordinates registered field
/// controls: submission gating, first-invalid focus, external-error mirroring, value
/// projection, and the `actionsRef` handle.
///
/// Like upstream, this is a *participation coordinator*: membership is decided by
/// context, not DOM ancestry, so portaled fields register normally.
#[component]
pub fn Form(
    /// `errors` (`:210`) — validation errors owned externally (typically after a server
    /// submission): keys are `Field.Root` names, values an error or an error list. Seeds
    /// the internal mirror (`:73`); see [`FormErrorsHandle`] for late-arriving errors.
    #[prop(default = None, optional)]
    errors: Option<FormErrors>,
    /// `validationMode` (`:204`, default `'onSubmit'`) — the default mode for contained
    /// fields; `<Field.Root>`'s own prop takes precedence (the field port reads this
    /// member through the context).
    #[prop(default = FormValidationMode::OnSubmit, optional)]
    validation_mode: FormValidationMode,
    /// `noValidate` (`:110`) — upstream injects `true` ahead of `elementProps`, so the
    /// browser's native constraint UI is suppressed unless the user opts back in with
    /// `noValidate={false}` (behavior.md "DOM structure"). `true` (the default) renders
    /// the bare `novalidate` attribute.
    #[prop(default = true, optional)]
    no_validate: bool,
    /// The user's native `onSubmit` (`:29`, called at `:125`) — receives the native form
    /// submit event; consumers call `preventDefault()` themselves and build `FormData`
    /// from `event.current_target()`.
    #[prop(default = None, optional)]
    on_submit: Option<Rc<dyn Fn(&Event)>>,
    /// `onFormSubmit` (`:215-216`) — called as `(formValues, eventDetails)` on a
    /// successful submit. Form prevents the native default itself when this is present
    /// (`:127-128`), which is why `eventDetails.event.default_prevented()` is `true`.
    #[prop(default = None, optional)]
    on_form_submit: Option<Rc<dyn Fn(FormValues, FormSubmitEventDetails)>>,
    /// `actionsRef` (`:229`) — the imperative `validate()` handle; written at
    /// materialization (`:88-104`).
    #[prop(default = None, optional)]
    actions_ref: Option<FormActionsRef>,
    /// The port's late-external-errors channel — see [`FormErrorsHandle`].
    #[prop(default = None, optional)]
    errors_handle: Option<FormErrorsHandle>,
    /// The `className` passthrough (`:26`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The `...elementProps` rest (`:33`) — the last props bag, so these override the
    /// injected `novalidate` on key conflict (the `field_root.rs` channel).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`:107`'s `[forwardedRef, elementRef]`), fired with the real
    /// `<form>` node at materialization.
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<HtmlFormElement>>,
    /// The parts subtree.
    children: Children,
) -> impl IntoView {
    // The refs (`:36-41`). The registry is the internals' shared shape; `elementRef`,
    // `submittedRef` and `submitCountRef` are the crate's `Rc<Cell<…>>` ref slots.
    let registry: FormRef = Rc::new(RefCell::new(FormState::default()));
    let element_ref: Rc<Cell<Option<HtmlFormElement>>> = Rc::new(Cell::new(None));
    let submitted: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let submit_count: Rc<Cell<u32>> = Rc::new(Cell::new(0));

    // `const [errors, setErrors] = React.useState(externalErrors)` (`:73`) — the mirror,
    // seeded from the prop (`errors ?? EMPTY_OBJECT` at `:164`).
    let errors_mirror: RwSignal<FormErrors> = RwSignal::new(errors.unwrap_or_default());

    // The mirror's commit hook — upstream's post-submit focus effect (`:79-86`), whose
    // dependency is the `errors` value itself. The port has no reactive effect to hang it
    // on (an rg-0.2 `Effect` needs a global executor, and a leptos `Effect` cannot track
    // an rg-0.2 signal — the dual-runtime law), so it is discharged at the mirror's two
    // write sites instead: `clearErrors` below, and the handle's late-errors path. The
    // flag is consumed once (`:84`), which is what keeps the focus on the *submit* commit
    // and never on a plain value change (behavior.md "Focus management").
    let commit_errors: Rc<dyn Fn()> = {
        let submitted = Rc::clone(&submitted);
        let registry = Rc::clone(&registry);
        Rc::new(move || {
            if !submitted.replace(false) {
                return;
            }
            focus_first_invalid(&registry);
        })
    };
    if let Some(handle) = &errors_handle {
        let write = Rc::clone(&commit_errors);
        handle.bind(
            errors_mirror,
            Rc::new(move |next: FormErrors| {
                errors_mirror.set(next);
                write();
            }),
        );
    }

    // `clearErrors` (`:145-157`): the only local writer. The `undefined` name is the
    // early return (`:146-148`); an absent key returns the previous record unchanged
    // (`Object.hasOwn`, `:150-152`) — the port compares the record after the delete so
    // the no-op case leaves the signal's value untouched.
    let clear_errors: leptos_ui_internals::form_context::ClearErrorsFn = {
        let errors_mirror = errors_mirror;
        let commit_errors = Rc::clone(&commit_errors);
        Rc::new(move |name: Option<&str>| {
            let Some(name) = name else {
                return;
            };
            // The unchanged case leaves the signal's value (and its identity) alone.
            if let Some(next) = prune_error(errors_mirror.get_untracked(), name) {
                errors_mirror.set(next);
                commit_errors();
            }
        })
    };

    // The shared state the mount effect and the submit listener capture.
    let state = SendWrapper::new(Rc::new(FormPipelineState {
        registry: Rc::clone(&registry),
        submit_count: Rc::clone(&submit_count),
        submitted: Rc::clone(&submitted),
        element_ref: Rc::clone(&element_ref),
        on_submit,
        on_form_submit,
        actions_ref,
        element_attributes,
        ref_callback,
        subscription: RefCell::new(None),
        listener_attached: Cell::new(false),
    }));

    // The provider + the children, built inside the bridge window (`FormContext.Provider`
    // wraps `element` upstream, `:171`): the window publishes the bag on the rg-0.2 owner
    // the field machinery reads through, and the parts subtree constructs inside it.
    let bridge_owner = Owner::new();
    let children_view = bridge_owner.with(|| {
        reactive_graph::owner::provide_context(SharedFormContext::new(FormContextValue {
            // `errors: errors ?? EMPTY_OBJECT` (`:164`) — a derived read of the mirror,
            // so consumers observe `clearErrors`' pruning and the handle's late writes.
            errors: Signal::derive_local(move || raw_errors(errors_mirror)),
            clear_errors,
            element_ref: Rc::clone(&element_ref),
            form_ref: Rc::clone(&registry),
            validation_mode,
            submit_count_ref: Rc::clone(&submit_count),
        }));

        children()
    });
    // The window owner outlives the subtree (the `field_root.rs` precedent).
    std::mem::forget(bridge_owner);

    // The mount-time wiring (`useRenderElement`'s `[forwardedRef, elementRef]` fork plus
    // the injected props/`onSubmit` — `:106-143`). Leptos `view!` has no ref fork or
    // attribute spread, so the node arrives through a `NodeRef` and one post-mount effect
    // syncs the inert slots (the `field_control.rs` convention) and attaches the submit
    // listener (no engine handler slot exists for `submit` — the `otp_field.rs` seam).
    let form_node: NodeRef<leptos::html::Form> = NodeRef::new();
    {
        let state = state.clone();
        Effect::new(move |_| {
            #[cfg(test)]
            MOUNT_EFFECT_RUNS.with(|runs| runs.set(runs.get() + 1));
            let Some(form) = form_node.get() else {
                return;
            };

            // `elementRef` (`:39`, published at `:161`).
            state.element_ref.set(Some(form.clone()));

            // The `actionsRef` handle (`:88-104`) — allocated over the live registry, so
            // it stays correct across rename/unmount/replacement without re-binding.
            if let Some(slot) = state.actions_ref.clone() {
                let registry = Rc::clone(&state.registry);
                slot.set(Some(Rc::new(FormActions::new(Rc::new(move |name| {
                    validate_registry(&registry, name);
                })))));
            }

            // The `...elementProps` rest bag (`:141`) — the later-wins layer, written
            // post-mount so it overrides the rendered defaults (the `field_root.rs`
            // convention: the empty string is the BARE attribute).
            let element: &Element = form.unchecked_ref();
            for (name, value) in &state.element_attributes {
                let _ = element.set_attribute(name, value);
            }

            // The forwarded ref (`:107`'s `[forwardedRef, elementRef]`).
            if let Some(callback) = &state.ref_callback {
                if let Some(cleanup) = callback(Some(&form)) {
                    // The element's lifetime is the view's; the detach rides the same
                    // teardown the listener does (the engine's fork convention).
                    std::mem::forget(cleanup);
                }
            }

            // The injected `onSubmit` (`:111-139`), attached exactly once.
            if !state.listener_attached.replace(true) {
                let pipeline = Rc::clone(&state);
                let subscription =
                    add_event_listener(&form, "submit", move |event: &web_sys::Event| {
                        handle_submit(&pipeline, event);
                    });
                *state.subscription.borrow_mut() = Some(subscription);
            }
        });
    }

    // The listener teardown (the `field_control.rs` `on_cleanup` convention: the `Rc`
    // slot crosses the `Send` boundary behind a `SendWrapper`).
    {
        let state = state.clone();
        let cleanup = SendWrapper::new(move || {
            let _ = state.subscription.borrow_mut().take();
        });
        reactive_graph::owner::on_cleanup(move || (*cleanup)());
    }

    // `noValidate: true` as the injected first bag (`:110`) — `false` opts back in to the
    // browser's native constraint UI (behavior.md "DOM structure").
    view! {
        <form node_ref=form_node novalidate=no_validate class=class>
            {children_view}
        </form>
    }
}

/// Reads the errors mirror as the context's published record. Upstream publishes
/// `errors ?? EMPTY_OBJECT` (`:164`); the port's mirror is never `null` (it defaults to
/// the empty record), so this is the identity read — kept as a named function so the
/// derivation's reference-stability intent is visible at the provide site.
fn raw_errors(mirror: RwSignal<FormErrors>) -> FormErrors {
    RgGet::get(&mirror)
}

// ---------------------------------------------------------------------------
// The namespaced surface (`Form.Props`, `Form.Values`, `Form.Actions`)
// ---------------------------------------------------------------------------
//
// Upstream's `Form` is a single component — it has no view subcomponents (behavior.md "Public API
// surface": "Parts/subcomponents: Form has no subcomponents of its own") — so what the namespace
// carries is the three dotted names upstream's docs and tests cite as TYPES:
//
//   * `Form.Props['errors']` (`Form.test.tsx:664`)  -> [`Props`] = the `Form` component's props
//   * `Form.Values` (cross-field validator's arg, `Form.test.tsx:213-215`) -> [`Values`]
//   * `Form.Actions` (the `actionsRef` handle, `Form.test.tsx:1045-1112`) -> [`Actions`]
//
// Each is an alias of the port's existing type, not a new one — the same name-to-type mapping the
// docs teach, with Rust's `::` in place of React's `.` (`specs/docs-content/CONTRACT.md`). Nothing
// is renamed: `FormProps`, `FormValues` and `FormActions` keep working.

/// `Form.Props` — the `Form` component's props type (`Form.test.tsx:664`).
pub type Props = FormProps;

/// `Form.Values` — the submitted-values record (`Form.test.tsx:213-215`).
pub type Values = FormValues;

/// `Form.Actions` — the imperative `actionsRef` handle (`Form.test.tsx:1045-1112`).
pub type Actions = FormActions;
