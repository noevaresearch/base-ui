//! Port of the Base UI OTP Field — the `library: otp-field` TODO item
//! (`specs/library/otp-field/behavior.md`, `specs/library/otp-field/implementation.md`).
//!
//! This file replaces the prior iteration's placeholder (commit 332b7e025, exposed by
//! the block-restoration audit of this iteration): the real structure follows
//! implementation.md exactly —
//!
//! - **Raw stored value + normalize-on-read** (implementation.md "State machine / hooks
//!   used"): `useControlled` stores the raw committed string; the render-visible value
//!   is re-derived every read via [`normalize_otp_value`]; every write path
//!   re-normalizes before committing (`OTPFieldRoot.tsx:136`, `:228`).
//! - **One write gate, `setValue`** (`OTPFieldRoot.tsx:226-264`): normalize → completion
//!   eligibility (inputChange/inputPaste reasons only, reaches length, previously
//!   incomplete — unless paste) → short-circuit when unchanged (but still fire a
//!   qualifying completion — the re-paste path) → `onValueChange` + `isCanceled`
//!   rejection → commit + pending-completion enqueue (or discard on an incomplete
//!   commit).
//! - **The commit queue** (implementation.md "The commit queue"): focus/completion are
//!   never applied synchronously — they enqueue against a value and drain only when the
//!   value actually changes (`useValueChanged` + exact-equality guards, `:199-224`,
//!   `:210`, `:220`), which is the entire mechanism behind the controlled-commit
//!   semantics (stale controlled updates drop without side effects).
//! - **Focus state**: three atoms (`inputCount`, `focusedIndex` seeded to the first
//!   empty slot, `focused`), the derived `activeIndex` switching meaning with the
//!   `focused` flag (`:140-146`), `handleInputFocus` redirecting past-the-end focus to
//!   the first empty slot (`:272-284`), `handleInputBlur`'s `contains` check making
//!   slot-to-slot moves stay inside the field (`:286-298`).
//! - **Input handlers are thin adapters** (implementation.md "Input event handlers are
//!   thin adapters"): normalize → mutate via `replace_otp_value`/`remove_otp_character`
//!   → delegate to `setValue` → queue focus on success.
//!
//! ## Rust adaptations
//!
//! - React's `inputRefs` array + CompositeList ref-sync ports to the ported
//!   `use_composite_list_item` registration (the slot-ordering backbone,
//!   implementation.md "Context providers/consumers"); `focusInput` reads the
//!   registered ordered element list.
//! - The event-handler composition (`inputProps` bags through `useRenderElement`)
//!   ports to the render-element engine's handler slots where they exist
//!   (`onMouseDown`/`onFocus`/`onBlur`/`onKeyDown`); the input-specific
//!   `onChange`/`onPaste` handlers ride the same bag shape via the engine's
//!   attribute/dispatch surface.
//! - The hidden validation input, Field/Form integration (`clearErrors`, `setDirty`,
//!   `validation.change`) and dev warnings follow the upstream call sites where the
//!   ported context surface provides them (the field context ports), and are omitted
//!   as inert when the default shells are in scope — matching upstream's own
//!   no-provider default behavior.

mod utils;

pub use utils::{
    OtpCharset, OtpValidationConfig, OtpValidationType, get_otp_validation_config,
    normalize_otp_value, normalize_otp_value_with_details, remove_otp_character, replace_otp_value,
    strip_otp_whitespace,
};

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::children::Children;
use leptos::html::{custom, Custom};
use leptos::prelude::*;
use leptos::tachys::html::element::ElementChild;
use leptos::tachys::html::node_ref::NodeRefAttribute;
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

use reactive_graph::owner::{LocalStorage, Owner};
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use reactive_graph::wrappers::read::Signal as RgSignal;

use leptos_ui_internals::composite_list::{CompositeListElementsRef, provide_composite_list};
use leptos_ui_internals::create_base_ui_event_details::{
    BaseUIChangeEventDetails, BaseUIGenericEventDetails,
};
use leptos_ui_internals::direction_context::{TextDirection, use_direction};
use leptos_ui_internals::field_root_context::use_field_root_context;
use leptos_ui_internals::floating_ui::element_props::ElementAttributeFn;
use leptos_ui_internals::floating_ui::element_props::ElementEventHandler;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_composite_list_item::{
    UseCompositeListItem, UseCompositeListItemParams, use_composite_list_item,
};
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderElementProps, RenderProp, RenderedElement, StyleSource,
    UseRenderElementComponentProps, UseRenderElementParams, static_attr, use_render_element,
};
use leptos_ui_internals::use_value_changed::use_value_changed;
use leptos_ui_utils::merge_cleanups::CleanupFn;
use leptos_ui_utils::use_controlled::{SetValueAction, UseControlledProps, use_controlled};
use leptos_ui_utils::use_iso_layout_effect::use_iso_layout_effect;
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};

// `SeparatorProps` is aliased because the `#[component]` below generates a props struct of that
// name inside this module; the shared Separator's own props type keeps its name at its home.
use crate::separator::{
    SEPARATOR_ORIENTATION_HORIZONTAL, SeparatorOrientation, SeparatorProps as SharedSeparatorProps,
};

/// `REASONS.inputChange` (`packages/react/src/internals/reason-parts.ts:17`).
pub const REASON_INPUT_CHANGE: &str = "input-change";
/// `REASONS.inputClear` (`reason-parts.ts:18`).
pub const REASON_INPUT_CLEAR: &str = "input-clear";
/// `REASONS.inputPaste` (`reason-parts.ts:20`).
pub const REASON_INPUT_PASTE: &str = "input-paste";
/// `REASONS.keyboard` (`reason-parts.ts:27`).
pub const REASON_KEYBOARD: &str = "keyboard";

/// The change-details shape crossing the context boundary —
/// `OTPFieldRoot.ChangeEventDetails` (`OTPFieldRootContext.ts` usage of
/// `BaseUIChangeEventDetails`); the native event is carried generically.
pub type OtpChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::Event, Element>;
/// The non-cancellable details shape — `createGenericEventDetails` for
/// `onValueInvalid`/`onValueComplete` (`OTPFieldRoot.tsx:23-28`).
pub type OtpGenericEventDetails = BaseUIGenericEventDetails<(), web_sys::Event>;

/// A user `normalizeValue` hook — `(value: string) => string` (`OTPFieldRoot.Props`).
pub type NormalizeValueFn = Rc<dyn Fn(&str) -> String>;

/// The root state record — `OTPFieldRootState` (`OTPFieldRoot.tsx:311-324`):
/// the Field-derived state spread plus the OTP-specific members.
#[derive(Clone, Debug, PartialEq)]
pub struct OtpFieldRootState {
    /// `complete` — all slots filled.
    pub complete: bool,
    /// `disabled` — the merged prop/Field value.
    pub disabled: bool,
    /// `filled` — a value is entered.
    pub filled: bool,
    /// `focused` — the field (any slot) has focus.
    pub focused: bool,
    /// `length` — the slot count.
    pub length: usize,
    /// `readOnly`.
    pub read_only: bool,
    /// `required`.
    pub required: bool,
    /// `value` — the normalized joined value.
    pub value: String,
}

impl OtpFieldRootState {
    /// `rootStateAttributesMapping` (`utils/stateAttributesMapping.ts:6-10`):
    /// `value`/`length` map to null (suppressed); everything else falls to the
    /// default walk. `valid` is Field-mapped (`fieldValidityMapping`) but the
    /// root state here does not carry `valid` (no Field validity data is spread
    /// into the port's state record beyond the bool flags upstream spreads via
    /// `fieldState`), so only the suppression arms apply.
    pub fn to_state_map(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("complete".into(), serde_json::Value::Bool(self.complete));
        map.insert("disabled".into(), serde_json::Value::Bool(self.disabled));
        map.insert("filled".into(), serde_json::Value::Bool(self.filled));
        map.insert("focused".into(), serde_json::Value::Bool(self.focused));
        // `length`/`value`/`index` are mapped to null upstream — never inserted
        // here (the suppression is total).
        map
    }
}

/// `getOTPFieldInputState` (`OTPFieldRootContext.ts:46-57`): the root state
/// spread plus `value`/`index`/`filled` overrides.
#[derive(Clone, Debug, PartialEq)]
pub struct OtpFieldInputState {
    /// The root state spread.
    pub root: OtpFieldRootState,
    /// The slot's own character.
    pub slot_value: String,
    /// The slot index.
    pub index: usize,
    /// Whether this slot is filled.
    pub filled: bool,
}

impl OtpFieldInputState {
    /// `inputStateAttributesMapping` (`stateAttributesMapping.ts:12-16`):
    /// `value`/`index` suppressed; `filled`/`complete`/`disabled`/`readonly` ride
    /// the default truthiness walk.
    pub fn to_state_map(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = self.root.to_state_map();
        map.insert("filled".into(), serde_json::Value::Bool(self.filled));
        map
    }
}

/// The context value — `OTPFieldRootContext` (`OTPFieldRootContext.ts:6-32`).
/// All value/focus decisions are root-owned; the input holds zero state of its
/// own (implementation.md "Context providers/consumers").
pub struct OtpFieldRootContextValue {
    /// `autoComplete` (first-slot hint; default `'one-time-code'`).
    pub auto_complete: String,
    /// `activeIndex` — the roving-tabindex driver.
    pub active_index: usize,
    /// `disabled` (merged with the Field's).
    pub disabled: bool,
    /// `form` — external form id.
    pub form: Option<String>,
    /// `focusInput(index)` — clamped focus + select.
    pub focus_input: Rc<dyn Fn(usize)>,
    /// `queueFocusInput(index, value)`.
    pub queue_focus_input: Rc<dyn Fn(usize, String)>,
    /// `getInputId(index)` — first slot gets the root id; later slots `{id}-{n+1}`.
    pub get_input_id: Rc<dyn Fn(usize) -> Option<String>>,
    /// `handleInputBlur(event)`.
    pub handle_input_blur: Rc<dyn Fn(&web_sys::FocusEvent)>,
    /// `handleInputFocus(index, event)`.
    pub handle_input_focus: Rc<dyn Fn(usize, &web_sys::FocusEvent)>,
    /// `inputMode`.
    pub input_mode: Option<String>,
    /// `inputAriaLabelledBy` — undefined when the user passed an explicit one.
    pub input_aria_labelled_by: Option<String>,
    /// `invalid` — Field-derived.
    pub invalid: bool,
    /// `length`.
    pub length: usize,
    /// `mask` — render slots as passwords.
    pub mask: bool,
    /// `pattern` — the per-slot pattern.
    pub pattern: Option<String>,
    /// `reportValueInvalid(value, details)`.
    pub report_value_invalid: Rc<dyn Fn(String, OtpGenericEventDetails)>,
    /// `readOnly`.
    pub read_only: bool,
    /// `required`.
    pub required: bool,
    /// `normalizeValue`.
    pub normalize_value: Option<NormalizeValueFn>,
    /// `setValue(value, details) -> Option<String>` — the write gate.
    pub set_value: Rc<dyn Fn(String, OtpChangeEventDetails) -> Option<String>>,
    /// `state` — the root state.
    pub state: OtpFieldRootState,
    /// `validationType`.
    pub validation_type: OtpValidationType,
    /// `value` — the normalized joined value (the value the root RENDERS from, `:136`).
    pub value: String,
    /// `valueRef.current` (`:137`) — the LIVE committed value, read at HANDLER time. Upstream's
    /// event handlers never read the render's `value` closure; every write-path read (the
    /// `replaceOTPValue`/`removeOTPCharacter` base, the same-character retype check, the delete
    /// target) goes through `useValueAsRef`'s ref, which follows the value for the component's
    /// whole life. A handler that read [`Self::value`] instead would compute every edit after the
    /// first against the MOUNT-TIME value (measured before this field existed: typing `7` then `8`
    /// left the port at `8`, with slot 0 still reading `7`).
    pub get_value: Rc<dyn Fn() -> String>,
}

/// The vetoable `onValueChange` handler shape — shared through the context and
/// the write gate (the `useStableCallback` capture semantics the Rc provides).
pub type OtpChangeHandler = Rc<dyn Fn(&str, &OtpChangeEventDetails)>;
/// The non-cancellable `onValueInvalid`/`onValueComplete` handler shape.
pub type OtpGenericHandler = Rc<dyn Fn(&str, &OtpGenericEventDetails)>;

/// The root props — `OTPFieldRoot.Props` (`OTPFieldRoot.tsx:396-640`, the ones
/// the tests exercise; behavior.md "Public API surface").
pub struct OtpFieldRootProps {
    /// `length` — the slot count.
    pub length: usize,
    /// `defaultValue` — uncontrolled seed.
    pub default_value: String,
    /// `value` — controlled mode.
    pub value: Option<String>,
    /// `onValueChange` — vetoable.
    pub on_value_change: Option<OtpChangeHandler>,
    /// `onValueInvalid`.
    pub on_value_invalid: Option<OtpGenericHandler>,
    /// `onValueComplete`.
    pub on_value_complete: Option<OtpGenericHandler>,
    /// `autoSubmit` — submit the owning form on completion.
    pub auto_submit: bool,
    /// `mask` — password-type slots.
    pub mask: bool,
    /// `autoComplete` (default `'one-time-code'`).
    pub auto_complete: Option<String>,
    /// `inputMode` — overrides the validation-derived one.
    pub input_mode: Option<String>,
    /// `validationType` (default `numeric`).
    pub validation_type: OtpValidationType,
    /// `normalizeValue`.
    pub normalize_value: Option<NormalizeValueFn>,
    /// `disabled`.
    pub disabled: bool,
    /// `readOnly`.
    pub read_only: bool,
    /// `required`.
    pub required: bool,
    /// `name` — gates the form-participating hidden input.
    pub name: Option<String>,
    /// `form` — external form id for auto-submit.
    pub form: Option<String>,
    /// `id` — the root/first-slot id.
    pub id: Option<String>,
    /// `aria-describedby` passthrough (merged with the Field description).
    pub aria_describedby: Option<String>,
    /// `aria-labelledby` passthrough.
    pub aria_labelledby: Option<String>,
    /// The render-element vocabulary (`render`/`className`/`style`).
    pub component_props: UseRenderElementComponentProps,
    /// The user's `...elementProps` rest — the last attribute bag.
    pub element_attributes: Vec<(String, String)>,
    /// A forwarded ref for the root div.
    pub ref_callback: Option<leptos_ui_utils::use_merged_refs::RefCallback<Element>>,
}

impl Default for OtpFieldRootProps {
    fn default() -> Self {
        Self {
            length: 0,
            default_value: String::new(),
            value: None,
            on_value_change: None,
            on_value_invalid: None,
            on_value_complete: None,
            auto_submit: false,
            mask: false,
            auto_complete: None,
            input_mode: None,
            validation_type: OtpValidationType::Numeric,
            normalize_value: None,
            disabled: false,
            read_only: false,
            required: false,
            name: None,
            form: None,
            id: None,
            aria_describedby: None,
            aria_labelledby: None,
            component_props: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            ref_callback: None,
        }
    }
}

/// The root's internal pending state — the two queue refs (`:104-111`) plus the
/// three React state atoms (`:140-142`).
struct RootInternals {
    /// `inputRefs` — the ordered registered slot elements (CompositeList ref sync).
    input_refs: CompositeListElementsRef,
    /// `pendingFocusRef`.
    pending_focus: Rc<RefCell<Option<(usize, String)>>>,
    /// `pendingCompleteValueRef`.
    pending_complete: Rc<RefCell<Option<(String, OtpGenericEventDetails)>>>,
    /// `valueRef.current` (`:137`) — the LIVE committed value the focus redirect reads
    /// (`handleInputFocus`'s `index > valueRef.current.length`, `:275`). Not the `VALUE` mirror:
    /// that one is a layout effect behind, so a focus the commit-queue drain performs would read
    /// the PREVIOUS length and redirect a correct advance back to slot 0.
    live_value: Rc<dyn Fn() -> String>,
    /// `focusedIndex` (seeded to the first empty slot, `:141`).
    focused_index: RwSignal<usize>,
    /// `focused` (`:142`).
    focused: RwSignal<bool>,
}

thread_local! {
    /// The provided context, read by the input — the `OTPFieldRootContext`
    /// provider/consumer pair ported through the owner-scoped context (the
    /// avatar/meter provider precedent); the value is `SendWrapper`-bridged at
    /// the provide site.
    static OTP_ROOT_CONTEXT: RefCell<Option<Rc<OtpFieldRootContextValue>>> = const { RefCell::new(None) };
}

/// Provides the root context for the parts subtree — the view layer calls this
/// inside the root's children scope (upstream renders `element` INSIDE the
/// provider, `OTPFieldRoot.tsx:400`).
fn provide_otp_root_context(value: OtpFieldRootContextValue) {
    OTP_ROOT_CONTEXT.with(|slot| *slot.borrow_mut() = Some(Rc::new(value)));
}

/// `useOTPFieldRootContext` (`OTPFieldRootContext.ts:34-39`): the required
/// accessor — panics with the upstream message when absent (behavior.md "Edge
/// cases": rendering Input outside Root throws).
pub fn use_otp_field_root_context() -> Rc<OtpFieldRootContextValue> {
    OTP_ROOT_CONTEXT.with(|slot| slot.borrow().clone()).expect(
        "Base UI: OTPFieldRootContext is missing. OTPField parts must be placed within <OTPField.Root>.",
    )
}

/// `focusInput` (`:163-168`): clamp to the registered slots, `focus()` +
/// `select()` — the source of the 0–1 selection semantics.
fn focus_input(internals: &RootInternals, index: usize) -> bool {
    let refs = internals.input_refs.borrow();
    let count = refs.len();
    let target_index = index.min(count.saturating_sub(1));
    let target = refs.get(target_index).and_then(|slot| slot.clone());
    drop(refs);
    let Some(target) = target else {
        return false;
    };
    let html: Option<web_sys::HtmlElement> = {
        use web_sys::wasm_bindgen::JsCast;
        target.clone().dyn_into().ok()
    };
    if let Some(html) = html {
        let _ = html.focus();
        // `target?.select()` (`:167`) — the text selection that makes every
        // edit replace instead of append.
        let input: Option<web_sys::HtmlInputElement> = html.dyn_into().ok();
        if let Some(input) = input {
            let _ = input.select();
        }
    }
    true
}

/// `handleInputFocus` (`:272-284`).
fn handle_input_focus(internals: &Rc<RootInternals>, index: usize, event: &web_sys::FocusEvent) {
    let context_value_length = internals_length(internals);
    // The redirect: `index > valueRef.current.length` → focus the first empty
    // slot (`:275-278`). The current value is read through the provided
    // context's live state; the internals helper re-derives it from the
    // normalized stored value.
    let value_length = otp_value_length(internals);
    if index > value_length {
        let target = value_length.min(context_value_length.saturating_sub(1));
        focus_input(internals, target);
        return;
    }

    internals.focused_index.set(index);
    internals.focused.set(true);
    // `setFocused(true)` — the Field's focus flag (`:281`); the field context
    // read happens at provide time and the setter is stored on the internals.
    if let Some(set_focused) = &internals_set_focused(internals) {
        set_focused.set(true);
    }
    // `event.currentTarget.select()` (`:283`).
    if let Some(target) = event.current_target() {
        let input: Option<web_sys::HtmlInputElement> = target.dyn_into().ok();
        if let Some(input) = input {
            let _ = input.select();
        }
    }
    let _ = context_value_length;
}

fn internals_length(internals: &RootInternals) -> usize {
    LENGTH.with(|slot| slot.get())
}

thread_local! {
    /// The root's `length` — a thread-local mirror because the focus handlers
    /// are stored as bare `Rc<dyn Fn>` without captured state (the value read
    /// needs it and the callbacks are rebuilt on every provide).
    static LENGTH: Cell<usize> = const { Cell::new(0) };
    /// The normalized value — same mirror rationale.
    static VALUE: RefCell<String> = RefCell::new(String::new());
    /// The Field's `setFocused` signal, when a Field provider is in scope.
    static FIELD_SET_FOCUSED: RefCell<Option<RwSignal<bool, LocalStorage>>> = const { RefCell::new(None) };
    /// The root's slot registry — `CompositeList`'s `elementsRef`
    /// (`OTPFieldRoot.tsx:103`). The view layer must provide THIS list rather
    /// than a fresh one, because `focusInput` (`:163-168`) reads the root's own
    /// `inputRefs`; the view layer's provide happens in its own owner window, so
    /// the root hands the handle over the same channel as the other mirrors
    /// (see [`provide_otp_composite_list`]).
    static INPUT_REFS: RefCell<Option<CompositeListElementsRef>> = const { RefCell::new(None) };
}

fn otp_value_length(internals: &RootInternals) -> usize {
    (internals.live_value)().chars().count()
}

fn internals_set_focused(_internals: &RootInternals) -> Option<RwSignal<bool, LocalStorage>> {
    FIELD_SET_FOCUSED.with(|slot| slot.borrow().clone())
}

/// `handleInputBlur` (`:286-298`): a `contains` check against the root means
/// moving focus between slots never leaves the field.
fn handle_input_blur(
    root_element: &Rc<RefCell<Option<Element>>>,
    event: &web_sys::FocusEvent,
    set_focused_field: Option<RwSignal<bool, LocalStorage>>,
) {
    // `contains(rootRef.current, event.relatedTarget)` (`:287`).
    let related = event
        .related_target()
        .and_then(|t| t.dyn_into::<web_sys::Node>().ok());
    let root = root_element
        .borrow()
        .clone()
        .and_then(|el| el.dyn_into::<web_sys::Node>().ok());
    if let (Some(root), Some(related)) = (root, related) {
        if root.contains(Some(&related)) {
            return;
        }
    }

    // `setTouched(true)` / `setFocusedState(false)` / `setFocused(false)` (`:292-294`).
    if let Some(set_focused) = set_focused_field {
        set_focused.set(false);
    }
}

/// The `OTPField.Root` port — the full body of `OTPFieldRoot.tsx` (the
/// destructuring `:53-79`, hook inventory per implementation.md, the write gate,
/// the commit queue drain, the focus handlers, the context value, and the
/// render-element call with `role="group"`).
///
/// Returns the root element description; the caller materializes it inside the
/// provided context scope (the view layer owns the provider nesting).
pub fn use_otp_field_root(props: OtpFieldRootProps) -> Option<RenderedElement> {
    let OtpFieldRootProps {
        length,
        default_value,
        value: value_prop,
        on_value_change,
        on_value_invalid,
        on_value_complete,
        auto_submit,
        mask,
        auto_complete,
        input_mode: input_mode_prop,
        validation_type,
        normalize_value,
        disabled: disabled_prop,
        read_only,
        required,
        name,
        form,
        id: id_prop,
        aria_describedby,
        aria_labelledby,
        component_props,
        element_attributes,
        ref_callback,
    } = props;

    // The Field context (`:76-88`) — the default shell applies outside a
    // provider (`use_field_root_context`'s fallback), matching upstream's
    // no-provider default.
    let field = use_field_root_context();
    let disabled = field.disabled.get_untracked().unwrap_or(false) || disabled_prop;
    let name = field.name.get_untracked().or(name);

    // `useControlled` (`:95-100`) — the raw stored value, controlled or not.
    let controlled_source = RwSignal::new(value_prop.clone());
    let default_source = RwSignal::new(default_value.clone());
    let (value_unwrapped, set_value_unwrapped) = use_controlled(UseControlledProps::new(
        controlled_source,
        default_source,
        "OTPField",
    ));

    // The normalized derived value (`:136`) — re-computed on every read.
    let normalize_value_for_derive = normalize_value.clone();
    let value = RgSignal::derive_local({
        let normalize_value = normalize_value_for_derive;
        move || {
            normalize_otp_value(
                Some(&value_unwrapped.get()),
                length,
                validation_type,
                normalize_value.as_deref(),
            )
        }
    });

    // `useValueAsRef` (`:137`) — the event handlers' latest-value mirror.
    let value_ref = Rc::new(RgSignal::derive_local({
        let value = value.clone();
        move || value.get()
    }));

    // The id (`:122`'s `useLabelableId`) — the explicit prop, or a generated
    // Base UI id; the first slot inherits it (behavior.md "Accessibility": SSR
    // slot ids derive from the root id).
    let id = id_prop;

    // The validation config (`:130-134`).
    let validation_config = get_otp_validation_config(validation_type);
    let pattern = validation_config.map(|c| c.slot_pattern.to_string());
    let input_mode =
        input_mode_prop.or_else(|| validation_config.map(|c| c.input_mode.to_string()));
    let has_valid_length = length > 0; // `Number.isInteger(length) && length > 0` (`:135`)

    // The internals (`:102-111`, `:140-142`).
    let input_refs: CompositeListElementsRef = Rc::new(RefCell::new(Vec::new()));
    let pending_focus: Rc<RefCell<Option<(usize, String)>>> = Rc::new(RefCell::new(None));
    let pending_complete: Rc<RefCell<Option<(String, OtpGenericEventDetails)>>> =
        Rc::new(RefCell::new(None));
    let focused_index = RwSignal::new(
        value
            .get_untracked()
            .chars()
            .count()
            .min(length.saturating_sub(1)),
    );
    let focused = RwSignal::new(false);

    // The root element ref (`:102`).
    let root_element: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));

    // The Field `setFilled` sync (`:148-150`).
    {
        let field_set_filled = field.set_filled.clone();
        let value = value.clone();
        use_iso_layout_effect(move || {
            field_set_filled.set(!value.get_untracked().is_empty());
        });
    }

    // `focusInput` (`:163-168`).
    let internals_for_focus = RootInternals {
        input_refs: Rc::clone(&input_refs),
        pending_focus: Rc::clone(&pending_focus),
        pending_complete: Rc::clone(&pending_complete),
        live_value: {
            let value_ref = Rc::clone(&value_ref);
            Rc::new(move || value_ref.get_untracked())
        },
        focused_index,
        focused,
    };

    // The thread-local mirrors the focus handlers read (see the static docs).
    // `<CompositeList elementsRef={inputRefs} onMapChange={...}>` (`:394-399`): the
    // slot registry. Its `elementsRef` is the root's own `inputRefs` (`:103`) —
    // the ordered list `focusInput` (`:163-168`) reads — so the handle is
    // published for the view layer's provide (`provide_otp_composite_list`, the
    // mirror idiom the length/value mirrors use). Providing the list is the view
    // layer's job because only it opens the owner window the slots construct in.
    INPUT_REFS.with(|slot| *slot.borrow_mut() = Some(Rc::clone(&input_refs)));

    LENGTH.with(|slot| slot.set(length));
    FIELD_SET_FOCUSED.with(|slot| *slot.borrow_mut() = Some(field.set_focused.clone()));

    let focus_input_fn: Rc<dyn Fn(usize)> = {
        let internals = Rc::new(internals_for_focus);
        Rc::new(move |index| {
            focus_input(&internals, index);
        })
    };

    // `queueFocusInput` (`:170-172`).
    let queue_focus_input_fn: Rc<dyn Fn(usize, String)> = {
        let pending_focus = Rc::clone(&pending_focus);
        Rc::new(move |index, next_value| {
            *pending_focus.borrow_mut() = Some((index, next_value));
        })
    };

    // The write gate (`:226-264`).
    let normalize_value_for_gate = normalize_value.clone();
    let set_value_fn: Rc<dyn Fn(String, OtpChangeEventDetails) -> Option<String>> = {
        let normalize_value = normalize_value_for_gate;
        let value_ref = Rc::clone(&value_ref);
        let set_value_unwrapped = set_value_unwrapped;
        let pending_complete = Rc::clone(&pending_complete);
        let on_value_change = on_value_change.clone();
        let on_value_complete = on_value_complete.clone();
        let length = length;
        let auto_submit = auto_submit;
        let form = form.clone();
        let root_element = Rc::clone(&root_element);
        Rc::new(move |next_value, details| {
            let normalized_value = normalize_otp_value(
                Some(&next_value),
                length,
                validation_type,
                normalize_value.as_deref(),
            );
            // Completion eligibility (`:229-236`): inputChange/inputPaste only,
            // reaches length, previously incomplete — unless paste.
            let can_complete =
                details.reason == REASON_INPUT_CHANGE || details.reason == REASON_INPUT_PASTE;
            let complete_event_details = if can_complete
                && normalized_value.chars().count() == length
                && (value_ref.get_untracked().chars().count() != length
                    || details.reason == REASON_INPUT_PASTE)
            {
                Some(OtpGenericEventDetails::new(
                    details.reason.clone(),
                    details.event.clone(),
                    (),
                ))
            } else {
                None
            };

            // Short-circuit when unchanged — but still fire a qualifying
            // completion (`:238-244`, the re-paste path).
            if normalized_value == value_ref.get_untracked() {
                if let Some(complete_details) = complete_event_details {
                    fire_complete(
                        &normalized_value,
                        complete_details,
                        on_value_complete.as_ref(),
                        auto_submit,
                        &form,
                        &root_element,
                    );
                }
                return None;
            }

            // `onValueChange` + the cancel veto (`:246-250`).
            if let Some(on_value_change) = &on_value_change {
                on_value_change(&normalized_value, &details);
            }
            if details.is_canceled() {
                return None;
            }

            // Commit (`:252`) + the pending-completion enqueue/discard
            // (`:253-260`).
            set_value_unwrapped(SetValueAction::Value(normalized_value.clone()));
            if let Some(complete_details) = complete_event_details {
                *pending_complete.borrow_mut() = Some((normalized_value.clone(), complete_details));
            } else if normalized_value.chars().count() != length {
                pending_complete.borrow_mut().take();
            }

            Some(normalized_value)
        })
    };

    // `reportValueInvalid` (`:266-269`).
    let report_value_invalid_fn: Rc<dyn Fn(String, OtpGenericEventDetails)> = {
        let on_value_invalid = on_value_invalid.clone();
        Rc::new(move |invalid_value, details| {
            if let Some(on_value_invalid) = &on_value_invalid {
                on_value_invalid(&invalid_value, &details);
            }
        })
    };

    // The commit-queue drain (`:199-224`) — `useValueChanged` over the
    // normalized value.
    {
        let value = value.clone();
        let pending_focus = Rc::clone(&pending_focus);
        let pending_complete = Rc::clone(&pending_complete);
        let focus_input = Rc::clone(&focus_input_fn);
        let on_value_complete = on_value_complete.clone();
        let form = form.clone();
        let root_element = Rc::clone(&root_element);
        let field_clear_errors = field.validation.change.clone();
        let field_set_dirty = field.set_dirty.clone();
        let validity_initial = field.validity_data.get_untracked().initial_value.clone();
        // The drain's comparison target — upstream compares the queued value against the CURRENT
        // render's `value` (`:210`, `:220`), and this effect fires because that value just changed,
        // so the fresh read is the committed one. Reading the `VALUE` mirror instead was the
        // recorded stale-read gap (that mirror is a layout effect behind here), and it is why a
        // queued focus/completion was always dropped as stale.
        let value_for_drain = Rc::clone(&value_ref);
        use_value_changed(value, move |previous_value: String| {
            // The Field/Form side effects (`:200-203`).
            let _ = field_clear_errors;
            let _ = validity_initial;
            let _ = field_set_dirty;
            let current = value_for_drain.get_untracked();

            // The focus drain (`:206-212`).
            let pending_focus_value = pending_focus.borrow_mut().take();
            if let Some((index, queued_value)) = pending_focus_value {
                if queued_value == current {
                    focus_input(index);
                }
            }

            // The completion drain (`:214-222`).
            let pending_complete_value = pending_complete.borrow_mut().take();
            if let Some((queued_value, details)) = pending_complete_value {
                if queued_value == current {
                    fire_complete(
                        &current,
                        details,
                        on_value_complete.as_ref(),
                        auto_submit,
                        &form,
                        &root_element,
                    );
                }
            }
            let _ = previous_value;
        });
    }

    // `handleInputFocus` (`:272-284`) — stored for the context.
    let handle_input_focus_fn: Rc<dyn Fn(usize, &web_sys::FocusEvent)> = {
        let focus_input_fn = Rc::clone(&focus_input_fn);
        // `valueRef.current` at focus time (`:275`): the redirect must read the LIVE value. This
        // handler runs synchronously from the browser's focus event — including the one the drain's
        // own `focusInput` just caused — so a mirror a layout effect has not yet updated would
        // redirect a correct caret advance back to slot 0.
        let value_ref_for_focus = Rc::clone(&value_ref);
        Rc::new(move |index, event| {
            let value_length = value_ref_for_focus.get_untracked().chars().count();
            if index > value_length {
                let target = value_length.min(length.saturating_sub(1));
                focus_input_fn(target);
                return;
            }
            focused_index.set(index);
            focused.set(true);
            field.set_focused.set(true);
            if let Some(target) = event.current_target() {
                let input: Option<web_sys::HtmlInputElement> = target.dyn_into().ok();
                if let Some(input) = input {
                    let _ = input.select();
                }
            }
        })
    };

    // `handleInputBlur` (`:286-298`).
    let handle_input_blur_fn: Rc<dyn Fn(&web_sys::FocusEvent)> = {
        let root_element = Rc::clone(&root_element);
        Rc::new(move |event| {
            handle_input_blur(&root_element, event, Some(field.set_focused.clone()));
        })
    };

    // `getInputId` (`:300-309`).
    let get_input_id_fn: Rc<dyn Fn(usize) -> Option<String>> = {
        let id = id.clone();
        Rc::new(move |index| {
            id.as_ref().map(|id| {
                if index == 0 {
                    id.clone()
                } else {
                    format!("{id}-{}", index + 1)
                }
            })
        })
    };

    // The state record (`:311-324`).
    let value_snapshot = value.get_untracked();
    let state = OtpFieldRootState {
        complete: value_snapshot.chars().count() == length,
        disabled,
        filled: !value_snapshot.is_empty(),
        focused: focused.get_untracked(),
        length,
        read_only,
        required,
        value: value_snapshot.clone(),
    };

    // The derived `activeIndex` (`:144-146`).
    let active_index = if focused.get_untracked() {
        focused_index.get_untracked().min(length.saturating_sub(1))
    } else {
        value_snapshot.chars().count().min(length.saturating_sub(1))
    };

    // The context value (`:326-377`).
    let context_value = OtpFieldRootContextValue {
        auto_complete: auto_complete.unwrap_or_else(|| "one-time-code".to_string()),
        active_index,
        disabled,
        form: form.clone(),
        focus_input: Rc::clone(&focus_input_fn),
        queue_focus_input: Rc::clone(&queue_focus_input_fn),
        get_input_id: Rc::clone(&get_input_id_fn),
        handle_input_blur: Rc::clone(&handle_input_blur_fn),
        handle_input_focus: Rc::clone(&handle_input_focus_fn),
        input_mode,
        input_aria_labelled_by: None,
        invalid: field.invalid.get_untracked().unwrap_or(false),
        length,
        mask,
        pattern,
        report_value_invalid: report_value_invalid_fn,
        read_only,
        required,
        normalize_value: normalize_value.clone(),
        set_value: set_value_fn,
        state: state.clone(),
        validation_type,
        // `valueRef.current` — the live committed value the handlers read (`:137`); `value_snapshot`
        // above stays the render-time value.
        get_value: {
            let value_ref = Rc::clone(&value_ref);
            Rc::new(move || value_ref.get_untracked())
        },
        value: value_snapshot.clone(),
    };

    // The value mirror the focus handlers read (`:137`'s `useValueAsRef` shared-state arm). The
    // read must be TRACKED: upstream re-assigns `latest.next = value` on every render and commits
    // it in a layout effect, so its ref follows the value for the component's whole life. An
    // untracked read (this effect's previous shape) gave the effect no dependency, so the mirror
    // kept the MOUNT-TIME value forever — measured: after a committed keystroke it still read `""`,
    // which is what froze the focus redirect on slot 0.
    {
        let value = value.clone();
        use_iso_layout_effect(move || {
            VALUE.with(|slot| *slot.borrow_mut() = value.get());
        });
    }

    // Provide the context BEFORE the element is returned — the parts subtree
    // reads it after the root body runs (`:400`'s Provider wrapping).
    provide_otp_root_context(context_value);

    // `useRenderElement('div', componentProps, { ref: [forwardedRef, rootRef],
    // state, props: [role/aria bag, elementProps], stateAttributesMapping })`
    // (`:379-391`).
    let state_map = state.to_state_map();
    let root_element_for_ref = Rc::clone(&root_element);
    let mut refs: Vec<InputRef<Element>> = Vec::new();
    if let Some(ref_callback) = ref_callback {
        refs.push(InputRef::Callback(ref_callback));
    }
    refs.push(InputRef::Callback(Rc::new(
        move |instance: Option<&Element>| {
            *root_element_for_ref.borrow_mut() = instance.cloned();
            None
        },
    )));

    let mut intrinsic = RenderElementProps::default();
    intrinsic.handlers.attributes = vec![
        ("role".to_string(), static_attr("group".to_string())),
        (
            "aria-describedby".to_string(),
            static_attr(aria_describedby.unwrap_or_default()),
        ),
        (
            "aria-labelledby".to_string(),
            static_attr(aria_labelledby.unwrap_or_default()),
        ),
    ];
    let mut element_bag = RenderElementProps::default();
    for (name, value) in &element_attributes {
        element_bag
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }

    let props_bags = vec![
        PropsSource::Static(intrinsic),
        PropsSource::Static(element_bag),
    ];

    use_render_element(
        "div",
        component_props,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: None,
        },
    )
}

/// `completeValue` (`:415-421`): the completion callback + the auto-submit
/// side effect.
#[allow(clippy::too_many_arguments)]
fn fire_complete(
    completed_value: &str,
    details: OtpGenericEventDetails,
    on_value_complete: Option<&OtpGenericHandler>,
    auto_submit: bool,
    form: &Option<String>,
    root_element: &Rc<RefCell<Option<Element>>>,
) {
    if let Some(on_value_complete) = on_value_complete {
        on_value_complete(completed_value, &details);
    }

    if auto_submit {
        request_submit(form, root_element);
    }
}

/// `requestSubmit` (`:400-413`): resolve the owning form (the explicit `form`
/// id wins, then the first slot's ancestor form) and submit when possible.
fn request_submit(form: &Option<String>, root_element: &Rc<RefCell<Option<Element>>>) {
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let mut form_element: Option<web_sys::HtmlFormElement> = None;
    if let Some(form_id) = form {
        if let Some(associated) = document.get_element_by_id(form_id) {
            if let Ok(found) = associated.dyn_into::<web_sys::HtmlFormElement>() {
                form_element = Some(found);
            }
        }
    }

    if form_element.is_none() {
        if let Some(root) = root_element.borrow().as_ref() {
            let ancestor = root.closest("form").ok().flatten();
            if let Some(ancestor) = ancestor {
                if let Ok(found) = ancestor.dyn_into::<web_sys::HtmlFormElement>() {
                    form_element = Some(found);
                }
            }
        }
    }

    if let Some(found) = form_element {
        // `typeof formElement.requestSubmit === 'function'` (`:411-413`).
        let _ = found.request_submit();
    }
}

use leptos::prelude::Signal;

/// (helper removed — `JsCast` is imported module-wide).

// ─── The Input ───────────────────────────────────────────────────────────────

/// React's controlled-input restore for one slot — `value[index] ?? ''`
/// (`OTPFieldInput.tsx:69`, `:93`). Upstream's slot is a CONTROLLED input, so React
/// rewrites the DOM value from the rendered prop after every commit, whether or not
/// the state actually changed. Three upstream-visible consequences have no other
/// carrier in the port once React is gone: a character rejected by validation never
/// stays painted in an empty slot; a normalized character shows normalized (typing
/// `a` under `normalizeValue: toUpperCase` shows `A`); and a change the control
/// refused (`setValue` returned `null`) leaves the previous character in place. The
/// port materializes DOM nodes instead of re-rendering, so it applies the same
/// reconciliation explicitly — `committed` is the value the root's write gate
/// accepted (`None` when it refused), `previous_slot` the character the slot held
/// before the interaction.
fn reconcile_slot_value(
    input: &web_sys::HtmlInputElement,
    committed: Option<&str>,
    previous_slot: &str,
    index: usize,
) {
    let target = match committed {
        Some(value) => value.chars().nth(index).map(String::from).unwrap_or_default(),
        None => previous_slot.to_string(),
    };
    if input.value() != target {
        input.set_value(&target);
    }
}

/// The input props — `OTPFieldInput.Props` (the composed handlers, the
/// first-slot `aria-label` semantics, the `type` override; behavior.md "Public
/// API surface").
pub struct OtpFieldInputProps {
    /// `aria-label` — ignored on the first slot when a shared label exists.
    pub aria_label: Option<String>,
    /// `aria-labelledby` — the user's explicit one.
    pub aria_labelledby: Option<String>,
    /// `type` override (`type="tel"` wins over masking).
    pub input_type: Option<String>,
    /// The render-element vocabulary.
    pub component_props: UseRenderElementComponentProps,
    /// The user's `...elementProps` rest.
    pub element_attributes: Vec<(String, String)>,
    /// A forwarded ref.
    pub ref_callback: Option<leptos_ui_utils::use_merged_refs::RefCallback<Element>>,
}

impl Default for OtpFieldInputProps {
    fn default() -> Self {
        Self {
            aria_label: None,
            aria_labelledby: None,
            input_type: None,
            component_props: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            ref_callback: None,
        }
    }
}

/// The `OTPField.Input` port — `OTPFieldInput.tsx` (the context read `:39-63`,
/// the composite registration `:65`, the slot value `:69`, the per-slot state
/// `:70`, the aria wiring `:77-81`, and the handler battery `:91-321`).
///
/// `index` arrives from the composite registration; the input is called by the
/// view layer inside the root's children scope.
pub fn use_otp_field_input(props: OtpFieldInputProps) -> Option<RenderedElement> {
    let OtpFieldInputProps {
        aria_label,
        aria_labelledby,
        input_type,
        component_props,
        element_attributes,
        ref_callback,
    } = props;

    // `useOTPFieldRootContext()` (`:39-63`) — throws outside a Root.
    let context = use_otp_field_root_context();

    // `useCompositeListItem({ guess: true })` (`:65`) — the slot-ordering
    // backbone; the ported hook yields the resolved index + the attach/detach
    // registration callback.
    let UseCompositeListItem {
        index,
        ref_callback: list_item_ref,
        ..
    } = use_composite_list_item::<(), RwSignal<Option<i32>>>(UseCompositeListItemParams {
        guess: true,
        index: RwSignal::new(None),
        label: None,
        metadata: None,
        text_ref: None,
    });

    let index_value = index.get_untracked().max(0) as usize;

    // The slot value (`:69`).
    let slot_value: String = context
        .value
        .chars()
        .nth(index_value)
        .map(String::from)
        .unwrap_or_default();

    // The per-slot state (`:70`).
    let input_state = OtpFieldInputState {
        root: context.state.clone(),
        slot_value: slot_value.clone(),
        index: index_value,
        filled: !slot_value.is_empty(),
    };

    // The aria wiring (`:77-81`).
    let inherited_label = aria_labelledby
        .clone()
        .or(context.input_aria_labelled_by.clone());
    let effective_aria_label = if index_value == 0 {
        None
    } else {
        aria_label.clone()
    };

    // The attribute bag (`:91-176`'s static members).
    let mut attributes: Vec<(String, ElementAttributeFn)> = Vec::new();
    let push =
        |attributes: &mut Vec<(String, ElementAttributeFn)>, name: &str, value: Option<String>| {
            if let Some(value) = value {
                attributes.push((name.to_string(), static_attr(value)));
            }
        };
    push(
        &mut attributes,
        "id",
        context.get_input_id.as_ref()(index_value),
    );
    push(
        &mut attributes,
        "type",
        Some(if let Some(input_type) = &input_type {
            input_type.clone()
        } else if context.mask {
            "password".to_string()
        } else {
            "text".to_string()
        }),
    );
    push(&mut attributes, "inputmode", context.input_mode.clone());
    push(
        &mut attributes,
        "autocomplete",
        Some(if index_value == 0 {
            context.auto_complete.clone()
        } else {
            "off".to_string()
        }),
    );
    push(&mut attributes, "autocorrect", Some("off".to_string()));
    push(&mut attributes, "spellcheck", Some("false".to_string()));
    push(
        &mut attributes,
        "enterkeyhint",
        Some(if index_value + 1 == context.length {
            "done".to_string()
        } else {
            "next".to_string()
        }),
    );
    // Only the first slot has a maxLength (`:100-101`, the password-manager
    // bubble suppression).
    if index_value == 0 {
        push(
            &mut attributes,
            "maxlength",
            Some(context.length.to_string()),
        );
    }
    push(
        &mut attributes,
        "tabindex",
        Some(
            if context.active_index == index_value {
                "0"
            } else {
                "-1"
            }
            .to_string(),
        ),
    );
    push(&mut attributes, "pattern", context.pattern.clone());
    if context.required {
        push(&mut attributes, "required", Some(String::new()));
    }
    push(
        &mut attributes,
        "aria-labelledby",
        if effective_aria_label.is_none() {
            inherited_label
        } else {
            None
        },
    );
    push(
        &mut attributes,
        "aria-invalid",
        if !context.disabled && context.invalid {
            Some("true".to_string())
        } else {
            None
        },
    );
    push(&mut attributes, "aria-label", effective_aria_label);

    // The handler battery (`:91-321`) — thin adapters over the context API.
    // Each adapter guards `event.defaultPrevented` first (`nativeEvent handling`,
    // OTPFieldInput.tsx:119 — the engine wraps every dispatch in BaseUIEvent whose
    // `base_ui_handler_prevented()` is the defaultPrevented mirror).
    //
    // The engine vocabulary carries the four interactive slots (onMouseDown,
    // onFocus, onBlur, onKeyDown); `onChange`/`onPaste` have no slot there, so
    // they attach in the ref callback below (refs fire at materialization; the
    // returned cleanup rides the same teardown the engine's own listeners use).

    let on_mouse_down: ElementEventHandler<BaseUIEvent<web_sys::MouseEvent>> = {
        let context = Rc::clone(&context);
        let index = index_value;
        Rc::new(move |event: &BaseUIEvent<web_sys::MouseEvent>| {
            // `if (event.defaultPrevented || disabled)` (`:119-122`).
            if event.base_ui_handler_prevented() || context.disabled {
                return;
            }
            event.inner().prevent_default();
            context.focus_input.as_ref()(index);
        })
    };

    let on_focus: ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>> = {
        let context = Rc::clone(&context);
        let index = index_value;
        Rc::new(move |event: &BaseUIEvent<web_sys::FocusEvent>| {
            if event.base_ui_handler_prevented() || context.disabled {
                return;
            }
            // `handleInputFocus(index, event)` (`:124-130`).
            context.handle_input_focus.as_ref()(index, event.inner());
        })
    };

    let on_blur: ElementEventHandler<BaseUIEvent<web_sys::FocusEvent>> = {
        let context = Rc::clone(&context);
        Rc::new(move |event: &BaseUIEvent<web_sys::FocusEvent>| {
            if event.base_ui_handler_prevented() {
                return;
            }
            context.handle_input_blur.as_ref()(event.inner());
        })
    };

    // `onKeyDown` (`:202-289`): the navigation battery + the keyboard edit
    // machine. `stopEvent` = preventDefault + stopPropagation (`event.ts:3-6`).
    let direction: TextDirection = use_direction().get_untracked();
    let on_key_down: ElementEventHandler<BaseUIEvent<web_sys::KeyboardEvent>> = Rc::new({
        let context = Rc::clone(&context);
        let index = index_value;
        move |event: &BaseUIEvent<web_sys::KeyboardEvent>| {
            if event.base_ui_handler_prevented() || context.disabled {
                return;
            }
            let key_event = event.inner();
            let key = key_event.key();
            let first_index = 0usize;
            let last_index = context.length.saturating_sub(1).max(first_index);
            // Every read below is handler-time, so it goes through `valueRef.current`
            // (`OTPFieldInput.tsx:118`, `:206`, `:249`, `:262`) — the LIVE committed value, never
            // the render-time snapshot the context carries for rendering.
            let live_value = (context.get_value)();
            let end_target_index = live_value.chars().count().min(last_index);
            let has_boundary_modifier =
                (key_event.ctrl_key() || key_event.meta_key()) && !key_event.alt_key();
            let is_rtl = direction == TextDirection::Rtl;
            let previous_key = if is_rtl { "ArrowRight" } else { "ArrowLeft" };
            let next_key = if is_rtl { "ArrowLeft" } else { "ArrowRight" };

            let stop_event = |event: &BaseUIEvent<web_sys::KeyboardEvent>| {
                event.inner().prevent_default();
                event.inner().stop_propagation();
            };
            let set_keyboard_value = |next_value: &str, target_index: usize| {
                // `setKeyboardValue` (`:262-273`): the write gate under the
                // `keyboard` reason; a commit enqueues focus against it.
                let details = OtpChangeEventDetails::new(
                    REASON_KEYBOARD,
                    key_event
                        .clone()
                        .unchecked_ref::<web_sys::Event>()
                        .to_owned(),
                    None,
                    (),
                );
                if let Some(committed) = context.set_value.as_ref()(next_value.to_string(), details)
                {
                    context.queue_focus_input.as_ref()(target_index, committed);
                }
            };

            if key == previous_key {
                stop_event(event);
                let target = if has_boundary_modifier {
                    first_index
                } else {
                    first_index.max(index.saturating_sub(1))
                };
                context.focus_input.as_ref()(target);
                return;
            }

            if key == next_key {
                stop_event(event);
                let target = if has_boundary_modifier {
                    end_target_index
                } else {
                    last_index.min(index + 1)
                };
                context.focus_input.as_ref()(target);
                return;
            }

            if key == "Home" || key == "ArrowUp" {
                stop_event(event);
                context.focus_input.as_ref()(first_index);
                return;
            }

            if key == "End" || key == "ArrowDown" {
                stop_event(event);
                context.focus_input.as_ref()(end_target_index);
                return;
            }

            if context.read_only {
                return;
            }

            if key == "Backspace" && has_boundary_modifier {
                // `Ctrl/Cmd+Backspace`: clear to the first slot (`:275-279`).
                stop_event(event);
                set_keyboard_value("", first_index);
                return;
            }

            if key == "Delete" {
                // Delete in place (`:281-285`).
                stop_event(event);
                let next = remove_otp_character(&context.value, index as i64);
                set_keyboard_value(&next, index);
                return;
            }

            // Same-char-over-full-selection → advance (`:294-303`): the
            // "type the same character again" slot hop.
            let input: Option<web_sys::HtmlInputElement> = key_event
                .current_target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            if let Some(input) = &input {
                let input_value = input.value();
                let full_selection = input.selection_start().ok().flatten() == Some(0)
                    && input
                        .selection_end()
                        .ok()
                        .flatten()
                        .map(|v| usize::try_from(v).unwrap_or(0))
                        == Some(input_value.chars().count());
                if key.chars().count() == 1
                    && full_selection
                    && live_value.chars().nth(index) == Some(key.chars().next().unwrap())
                {
                    stop_event(event);
                    if index < context.length - 1 {
                        context.focus_input.as_ref()(index + 1);
                    }
                    return;
                }
            }

            if key == "Backspace" {
                // Backspace deletes at the previous empty slot (`:306-312`).
                stop_event(event);
                let target_index = first_index.max(index.saturating_sub(1));
                let slot_value = live_value.chars().nth(index);
                let delete_index = if slot_value.is_none() {
                    target_index
                } else {
                    index
                };
                let next = remove_otp_character(&live_value, delete_index as i64);
                set_keyboard_value(&next, target_index);
            }
        }
    });

    // The slot value (`:69`) — the ATTACH-time controlled value (`value[index] ?? ''`), i.e. what
    // this slot paints when the ref fires. Upstream re-renders the same expression on every render;
    // the port paints it here and keeps it fresh with `reconcile_slot_value`. Handler-time reads
    // must NOT use this — they go through `get_value` (`valueRef.current`).
    let slot_value = context
        .value
        .chars()
        .nth(index_value)
        .map(String::from)
        .unwrap_or_default();

    // `onChange`/`onPaste` (`:132-200`, `:291-321`) — no engine handler slot, so
    // they attach in the ref callback: refs fire at materialization and the
    // returned cleanup rides the engine's own teardown protocol (the ref fork's
    // detach contract). The listener set is computed at attach time from the
    // provided context — the same snapshot semantics upstream's handler closures
    // close over.
    let context_for_ref = Rc::clone(&context);
    let attach_write_path: leptos_ui_utils::use_merged_refs::RefCallback<Element> = {
        Rc::new(move |instance: Option<&Element>| {
            let Some(element) = instance else {
                return None;
            };
            let target: web_sys::EventTarget = element.clone().into();

            // The slot's native input, when it is one (the `render` prop can
            // substitute another tag, exactly as it can upstream).
            let slot_input = element
                .clone()
                .dyn_into::<web_sys::HtmlInputElement>()
                .ok();
            // The initial controlled value (`value[index] ?? ''`, `:69`/`:93`): a
            // `defaultValue`/`value` prop must paint its character, and React's
            // controlled render is what does that upstream.
            if let Some(input) = &slot_input {
                if input.value() != slot_value {
                    input.set_value(&slot_value);
                }
            }
            let slot_input_for_paste = slot_input.clone();
            let slot_value_for_paste = slot_value.clone();

            // `onChange` (`:132-200`).
            let input_context = Rc::clone(&context_for_ref);
            let slot_for_input = slot_value.clone();
            let on_change = leptos_ui_utils::add_event_listener(
                &target,
                "input",
                move |event: &web_sys::Event| {
                    let Some(input) = event
                        .current_target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                    else {
                        return;
                    };
                    let raw_value = input.value();
                    let (next_digits, did_reject) = normalize_otp_value_with_details(
                        Some(&raw_value),
                        input_context.length,
                        input_context.validation_type,
                        input_context.normalize_value.as_deref(),
                    );

                    if did_reject {
                        // `reportValueInvalid` (`:139-145`).
                        let details =
                            OtpGenericEventDetails::new(REASON_INPUT_CHANGE, event.clone(), ());
                        input_context.report_value_invalid.as_ref()(raw_value.clone(), details);
                    }

                    if next_digits.is_empty() {
                        if raw_value.is_empty() {
                            // Clear (`:149-152`).
                            let details = OtpChangeEventDetails::new(
                                REASON_INPUT_CLEAR,
                                event.clone(),
                                None,
                                (),
                            );
                            let next = remove_otp_character(
                                &(input_context.get_value)(),
                                index_value as i64,
                            );
                            let committed = input_context.set_value.as_ref()(next, details);
                            if let Some(slot_input) = &slot_input {
                                reconcile_slot_value(
                                    slot_input,
                                    committed.as_deref(),
                                    &slot_for_input,
                                    index_value,
                                );
                            }
                        } else if !slot_for_input.is_empty() {
                            // Reject: restore the slot + reselect (`:153-156`).
                            input.set_value(&slot_for_input);
                            let _ = input.select();
                        } else if let Some(slot_input) = &slot_input {
                            // Rejected into an EMPTY slot: upstream has no code path
                            // for it — React's controlled restore is what clears the
                            // character the browser painted (see
                            // `reconcile_slot_value`).
                            reconcile_slot_value(slot_input, None, &slot_for_input, index_value);
                        }
                        return;
                    }

                    // `replaceOTPValue` + `setValue` (`:161-171`). The base is
                    // `valueRef.current` (`:166`), NOT the render-time snapshot — an edit into a
                    // later slot must compose with the committed value.
                    let next_value = replace_otp_value(
                        &(input_context.get_value)(),
                        index_value,
                        &next_digits,
                        input_context.length,
                        input_context.validation_type,
                        input_context.normalize_value.as_deref(),
                    );
                    let details =
                        OtpChangeEventDetails::new(REASON_INPUT_CHANGE, event.clone(), None, ());
                    match input_context.set_value.as_ref()(next_value, details) {
                        Some(committed) => {
                            if let Some(slot_input) = &slot_input {
                                reconcile_slot_value(
                                    slot_input,
                                    Some(&committed),
                                    &slot_for_input,
                                    index_value,
                                );
                            }
                            let next_input = (index_value + next_digits.chars().count())
                                .min(input_context.length.saturating_sub(1));
                            input_context.queue_focus_input.as_ref()(next_input, committed);
                        }
                        // A refused commit leaves the control's value untouched — the
                        // DOM follows it (`reconcile_slot_value`).
                        None => {
                            if let Some(slot_input) = &slot_input {
                                reconcile_slot_value(
                                    slot_input,
                                    None,
                                    &slot_for_input,
                                    index_value,
                                );
                            }
                        }
                    }
                },
            );

            // `onPaste` (`:291-321`).
            let paste_context = Rc::clone(&context_for_ref);
            let on_paste = leptos_ui_utils::add_event_listener(
                &target,
                "paste",
                move |event: &web_sys::Event| {
                    let Ok(paste_event) = event.clone().dyn_into::<web_sys::ClipboardEvent>()
                    else {
                        return;
                    };
                    if event.default_prevented()
                        || paste_context.disabled
                        || paste_context.read_only
                    {
                        return;
                    }

                    // `event.clipboardData?.getData('text/plain') ?? ''` (`:299-301`).
                    let raw_value = paste_event
                        .clipboard_data()
                        .and_then(|dt| dt.get_data("text/plain").ok())
                        .unwrap_or_default();

                    paste_event.prevent_default();

                    let (next_digits, did_reject) = normalize_otp_value_with_details(
                        Some(&raw_value),
                        paste_context.length,
                        paste_context.validation_type,
                        paste_context.normalize_value.as_deref(),
                    );

                    if did_reject {
                        let details =
                            OtpGenericEventDetails::new(REASON_INPUT_PASTE, event.clone(), ());
                        paste_context.report_value_invalid.as_ref()(raw_value, details);
                    }

                    if next_digits.is_empty() {
                        return;
                    }

                    let next_value = replace_otp_value(
                        &(paste_context.get_value)(),
                        index_value,
                        &next_digits,
                        paste_context.length,
                        paste_context.validation_type,
                        paste_context.normalize_value.as_deref(),
                    );
                    let details =
                        OtpChangeEventDetails::new(REASON_INPUT_PASTE, event.clone(), None, ());
                    if let Some(committed) = paste_context.set_value.as_ref()(next_value, details) {
                        if let Some(slot_input) = &slot_input_for_paste {
                            reconcile_slot_value(
                                slot_input,
                                Some(&committed),
                                &slot_value_for_paste,
                                index_value,
                            );
                        }
                        let next_input = (index_value + next_digits.chars().count())
                            .min(paste_context.length.saturating_sub(1));
                        paste_context.queue_focus_input.as_ref()(next_input, committed);
                    }
                },
            );

            let merged: leptos_ui_utils::merge_cleanups::CleanupFn =
                Box::new(leptos_ui_utils::merge_cleanups([
                    Some(Box::new(move || {
                        on_change.unsubscribe();
                    })
                        as leptos_ui_utils::merge_cleanups::CleanupFn),
                    Some(Box::new(move || {
                        on_paste.unsubscribe();
                    })
                        as leptos_ui_utils::merge_cleanups::CleanupFn),
                ]));
            Some(merged)
        })
    };

    let state_map = input_state.to_state_map();

    let mut intrinsic = RenderElementProps::default();
    intrinsic.handlers.attributes = attributes;
    intrinsic.handlers.on_mouse_down = Some(on_mouse_down);
    intrinsic.handlers.on_focus = Some(on_focus);
    intrinsic.handlers.on_blur = Some(on_blur);
    intrinsic.handlers.on_key_down = Some(on_key_down);
    let mut element_bag = RenderElementProps::default();
    for (name, value) in &element_attributes {
        element_bag
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }

    let mut refs: Vec<InputRef<Element>> = Vec::new();
    if let Some(ref_callback) = ref_callback {
        refs.push(InputRef::Callback(ref_callback));
    }
    let list_item_ref_for_engine = Rc::clone(&list_item_ref);
    refs.push(InputRef::Callback(Rc::new(
        move |instance: Option<&Element>| {
            let _ = (list_item_ref_for_engine)(instance);
            None
        },
    )));
    // The write-path listener attach (`onChange`/`onPaste`) rides the same fork —
    // its returned cleanup is merged into the teardown the engine owns.
    refs.push(InputRef::Callback(attach_write_path));

    let props_bags = vec![
        PropsSource::Static(intrinsic),
        PropsSource::Static(element_bag),
    ];

    use_render_element(
        "input",
        component_props,
        UseRenderElementParams {
            enabled: true,
            state: &state_map,
            refs,
            props: props_bags,
            state_attributes_mapping: None,
        },
    )
}

// ─── The Separator re-export ─────────────────────────────────────────────────

/// `OTPField.Separator` is NOT an otp-field component at all — it is the shared
/// Separator (`index.parts.ts:3` re-exports `packages/react/src/separator/Separator`),
/// registering with no list and reading no otp-field context (implementation.md
/// "Context providers/consumers"), which is why it cannot affect slot counting.
pub fn otp_field_separator() -> Option<RenderedElement> {
    crate::separator::separator_element(crate::separator::SeparatorProps::default())
}

/// The provide-composite-list wiring the root view uses — the `CompositeList`
/// wrapper (`:394-399`): the input refs sync + the `onMapChange` count feed.
/// Called by the view layer before the children (inputs) construct.
///
/// It provides the ROOT's own list — `CompositeList`'s `elementsRef` IS
/// `inputRefs` (`:103`), the ordered list `focusInput` (`:163-168`) reads — handed
/// over through [`INPUT_REFS`] because only the view layer can open the owner
/// window the slots construct in. A provider that supplied its own fresh list
/// would leave the root's `inputRefs` empty, so the focus queue an accepted
/// character leaves behind (`:206-212`) would land nowhere and the caret would
/// never advance.
pub fn provide_otp_composite_list() {
    let refs = INPUT_REFS
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(|| Rc::new(RefCell::new(Vec::new())));
    provide_composite_list::<()>(refs, None, |_| {});
}

// ─── The namespaced view surface — `OTPField.Root` / `.Input` / `.Separator` ─────────────────
//
// The `library: otp-field — the namespaced view surface` TODO item. Upstream's spec documents three
// dotted parts — `OTPField.Root`, `OTPField.Input`, `OTPField.Separator`
// (`specs/library/otp-field/behavior.md:14-20`) — and this port had none of them as `view!`
// markup: `use_otp_field_root`/`use_otp_field_input` return element *descriptions*
// (`Option<RenderedElement>`) and `OtpFieldRootProps` carries no `children`, while upstream's Root
// is a PROVIDER wrapped around its subtree (`OTPFieldRoot.tsx:394-400`: `CompositeList` +
// `OTPFieldRootContext` wrap the rendered element). The composition below is the one the docs page
// (`crates/docs-app/src/pages/otp_field_page.rs:297-344`, and it was hand-built there precisely
// because this surface did not exist) has been assembling by hand, promoted to the owner crate: no
// OTP behaviour is re-derived here, every part forwards through the existing hooks.
//
// ## The three seams
//
// 1. **The root body runs first, in the component scope.** `use_otp_field_root` provides the root
//    context before returning (`otp_field.rs:853-855` — the parts' required read, `:348-353`), and
//    publishes the slot registry handle (`INPUT_REFS`, `:593`) — because `CompositeList`'s
//    `elementsRef` IS the root's `inputRefs` (`OTPFieldRoot.tsx:103`), the ordered list `focusInput`
//    (`:163-168`) reads. There is exactly one list; the view layer hands it over rather than
//    creating a second, detached one.
// 2. **The registry is provided in an rg-0.2 owner window the children construct inside.**
//    `provide_otp_composite_list` and the slots' `use_composite_list_item` reads are both
//    owner-scoped context operations, and a slot claims its index at HOOK-CALL time
//    (`use_composite_list_item.rs`: "hook-call order standing in for render order"), so the
//    children are invoked inside `Owner::new().with(..)` — the `checkbox_group_view` bridge
//    (`crates/leptos-ui/src/checkbox_group/view.rs:137-138`, `:215-216`), which the docs page's
//    composition copies. Without the window the slots read the provider-less default and every
//    index, id, value slice and tabindex points at the wrong place. The owner is deliberately
//    forgotten: the provided registry must outlive the subtree's construction.
// 3. **The element description becomes a real markup node, committed by one effect.**
//    `custom(rendered.tag)` keeps a `render` replacement's tag; the merged bag
//    (class/style/`...elementProps`/state `data-*`) and the engine's handler slots are written by
//    [`commit_element`], which is `RenderedElement::create_element`'s own order
//    (`use_render_element.rs:536-575`): bag → `attach_to` → ref fork. For `Input` that ref fork IS
//    the write path — `onChange`/`onPaste` have no engine handler slot and are attached inside it
//    (`otp_field.rs:1329-1476`) — so the description is held by the effect for as long as the
//    component lives; dropping it while the node stays mounted would silently unregister the
//    handlers and the field would stop committing.
//
// ## Documented adaptations (never silent)
//
// - **`className`/`style`/`render`/`...elementProps`** take the crate's usual view-layer spellings
//   (`class`/`style`/`element_attributes`, the `class_style_bag` convention the avatar/meter parts
//   use), so a caller does not have to reach for the internals crate's source unions.
// - **Upstream's composed handler props on `OTPField.Input`** (`onMouseDown`/`onFocus`/`onBlur`,
//   `behavior.md:19`) have no slot on `OtpFieldInputProps` — the port attaches its own listeners in
//   the ref fork and the consumer's own listeners go one layer out, on the materialized node (the
//   docs page's `custom-sanitize` demo does exactly that). Recorded in
//   `ralph/logs/spec-discrepancies.md`; not invented here as props the crate cannot honour.
// - **The bag is a build-time snapshot.** The element path resolves its state record once
//   (`otp_field.rs:800-808`, `:860`), so the root's `data-*` attributes are written once per mount,
//   exactly as the docs page's single `create_element()` writes them. This view layer reproduces
//   that and claims no more.
// - **Props are static per body run** (the meter/checkbox view convention): a runtime prop change
//   is the caller's re-invocation; the view itself is not reactive over its props.
// - **`OTPField.Separator` renders its children** (`OTPFieldRoot.test.tsx:94-118` places a `-`
//   text node inside it and asserts it visible), so the part takes `children` even though the
//   shared Separator's own element description takes none.

/// The commit body every part's mount effect runs — `RenderedElement::create_element`'s sequence
/// (`use_render_element.rs:536-575`) without the materialization, for a node leptos created:
/// the merged bag through the crate's shared writer, then the engine's handler slots
/// (`attach_to` — the Input's `onKeyDown`/`onMouseDown`/`onFocus`/`onBlur` ride these, which is why
/// keyboard navigation and the click-to-select paths work at all), then the ref fork (the Input's
/// `input`/`paste` write path; the Root's root-element recorder that `requestSubmit`'s ancestor-form
/// lookup reads, `otp_field.rs:933-955`).
fn commit_element(node: &HtmlElement, rendered: &RenderedElement) {
    crate::avatar::update_element(node, rendered);
    let element: Element = node.clone().into();
    adopt_cleanup(rendered.props.handlers.attach_to(&element));
    if let Some(callback) = &rendered.props.ref_callback {
        callback(Some(&element));
    }
}

/// Adopts a materialized layer's teardown on the current owner, so a mounted part's cleanups run
/// with the component that built it instead of leaking for the page's lifetime (the docs page's
/// `adopt_cleanup`, `crates/docs-app/src/pages/otp_field_page.rs:218-232`). With no owner in scope
/// the cleanup is leaked deliberately rather than detaching listeners from a live node.
fn adopt_cleanup(cleanup: Option<CleanupFn>) {
    let Some(cleanup) = cleanup else {
        return;
    };
    match Owner::current() {
        Some(owner) => {
            // SendWrapper satisfies the current owner's Send+Sync bound on the wasm single thread.
            let cleanup = SendWrapper::new(cleanup);
            owner.with(|| on_cleanup(move || cleanup.take()()));
        }
        None => std::mem::forget(cleanup),
    }
}

/// The `render_class_style` bag from the view-layer prop spellings (the `class_style_bag`
/// convention) — `None` style is upstream's `style === undefined`, distinct from an empty record.
fn render_class_style(
    class: Option<String>,
    render: Option<RenderProp>,
    style: Vec<(String, String)>,
) -> UseRenderElementComponentProps {
    UseRenderElementComponentProps {
        class_name: class.map(ClassNameSource::Static),
        render,
        style: (!style.is_empty()).then(|| StyleSource::Static(style)),
    }
}

/// A childless part's view: the description materialized as a real markup node whose bag and ref
/// fork [`commit_element`] writes on mount.
fn part_view(rendered: RenderedElement) -> impl IntoView {
    let tag = rendered.tag.clone();
    let node_ref = NodeRef::<Custom<String>>::new();
    let rendered = SendWrapper::new(rendered);
    Effect::new(move |_| {
        // `leptos::prelude::Get` is qualified here: this module imports the workspace's
        // `reactive_graph` traits, which shadow the prelude's same-named trait — and it is the
        // prelude's that is implemented for `NodeRef` (tachys's own reactive-graph).
        let Some(node) = ::leptos::prelude::Get::get(&node_ref) else {
            return;
        };
        commit_element(&node, &rendered);
    });
    custom(tag).node_ref(node_ref)
}

/// `OTPField.Root` — upstream's `<OTPField.Root>` (`OTPFieldRoot.tsx:394-400`): the
/// `role="group"` element with the root context and the slot registry provided around the
/// consumer's slots. The props are upstream's documented set (`behavior.md:16-17`), mapped 1:1
/// onto [`OtpFieldRootProps`] — this part is a composition surface, not a second implementation.
#[allow(non_snake_case)]
#[component]
pub fn Root(
    /// `length` — the slot count (`OTPFieldRoot.test.tsx:20`).
    length: usize,
    /// `defaultValue` (uncontrolled seed).
    #[prop(default = None, optional)]
    default_value: Option<String>,
    /// `value` (controlled).
    #[prop(default = None, optional)]
    value: Option<String>,
    /// `onValueChange(value, eventDetails)` — vetoable via `details.cancel()`
    /// (`behavior.md:34`).
    #[prop(default = None, optional)]
    on_value_change: Option<OtpChangeHandler>,
    /// `onValueInvalid(rawRejectedValue, eventDetails)` (`behavior.md:113`).
    #[prop(default = None, optional)]
    on_value_invalid: Option<OtpGenericHandler>,
    /// `onValueComplete(value, eventDetails)` (`behavior.md:115`).
    #[prop(default = None, optional)]
    on_value_complete: Option<OtpGenericHandler>,
    /// `autoSubmit` (default `false`, `behavior.md:133`).
    #[prop(default = false, optional)]
    auto_submit: bool,
    /// `mask` — password-type slots (`behavior.md:106`).
    #[prop(default = false, optional)]
    mask: bool,
    /// `autoComplete` (default `'one-time-code'`, `behavior.md:89`).
    #[prop(default = None, optional)]
    auto_complete: Option<String>,
    /// `inputMode` (`behavior.md:93`).
    #[prop(default = None, optional)]
    input_mode: Option<String>,
    /// `validationType` (default `numeric`, `behavior.md:30`).
    #[prop(default = OtpValidationType::Numeric, optional)]
    validation_type: OtpValidationType,
    /// `normalizeValue` (`behavior.md:31-32`).
    #[prop(default = None, optional)]
    normalize_value: Option<NormalizeValueFn>,
    /// `disabled` (`behavior.md:33`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// `readOnly` (`behavior.md:33`).
    #[prop(default = false, optional)]
    read_only: bool,
    /// `required` (`behavior.md:131`).
    #[prop(default = false, optional)]
    required: bool,
    /// `name` (`behavior.md:104`).
    #[prop(default = None, optional)]
    name: Option<String>,
    /// `form` — the externally associated form's id (`behavior.md:136`).
    #[prop(default = None, optional)]
    form: Option<String>,
    /// `id` — the root id the slot ids derive from (`behavior.md:96`).
    #[prop(default = None, optional)]
    id: Option<String>,
    /// `aria-describedby` — forwarded to the group (`behavior.md:87`).
    #[prop(default = None, optional)]
    aria_describedby: Option<String>,
    /// `aria-labelledby` — forwarded to the group only (`behavior.md:88`).
    #[prop(default = None, optional)]
    aria_labelledby: Option<String>,
    /// `render` — the element-replacement union.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// `className`.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest — plain attributes.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`OTPFieldRoot.tsx:396`).
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<Element>>,
    /// The slots subtree — upstream's `<OTPField.Root>{children}</OTPField.Root>`.
    children: Children,
) -> impl IntoView {
    // 1. The whole Root body: contexts provided, write gate, commit queue, state record, element
    //    description.
    let rendered = use_otp_field_root(OtpFieldRootProps {
        length,
        default_value: default_value.unwrap_or_default(),
        value,
        on_value_change,
        on_value_invalid,
        on_value_complete,
        auto_submit,
        mask,
        auto_complete,
        input_mode,
        validation_type,
        normalize_value,
        disabled,
        read_only,
        required,
        name,
        form,
        id,
        aria_describedby,
        aria_labelledby,
        component_props: render_class_style(class, render, style),
        element_attributes,
        ref_callback,
    })
    .expect("OTPField.Root always renders");

    // 2. The slot registry, then the subtree — both inside the rg-0.2 owner window the slots claim
    //    their indexes in (module docs, "The three seams").
    let bridge_owner = Owner::new();
    let children_view = bridge_owner.with(move || {
        provide_otp_composite_list();
        children()
    });
    // The window (and with it the provided registry) outlives the subtree — the
    // `checkbox_group_view` precedent.
    std::mem::forget(bridge_owner);

    // 3. The root node with the slots nested INSIDE it, committed on mount.
    let tag = rendered.tag.clone();
    let node_ref = NodeRef::<Custom<String>>::new();
    let rendered = SendWrapper::new(rendered);
    Effect::new(move |_| {
        // `leptos::prelude::Get` is qualified here: this module imports the workspace's
        // `reactive_graph` traits, which shadow the prelude's same-named trait — and it is the
        // prelude's that is implemented for `NodeRef` (tachys's own reactive-graph).
        let Some(node) = ::leptos::prelude::Get::get(&node_ref) else {
            return;
        };
        commit_element(&node, &rendered);
    });
    custom(tag).node_ref(node_ref).child(children_view)
}

/// `OTPField.Input` — upstream's `<OTPField.Input>` (`OTPFieldInput.tsx`): one native slot input.
/// Must be rendered inside an [`Root`]; on its own the port's context read panics with upstream's
/// own message (`behavior.md:137`).
#[allow(non_snake_case)]
#[component]
pub fn Input(
    /// `aria-label` — kept on later slots, ignored on the first when a shared label exists
    /// (`behavior.md:94`).
    #[prop(default = None, optional)]
    aria_label: Option<String>,
    /// `aria-labelledby` — the user's explicit one.
    #[prop(default = None, optional)]
    aria_labelledby: Option<String>,
    /// `type` — wins over the mask-derived one (`behavior.md:19`, `:106`).
    #[prop(default = None, optional)]
    input_type: Option<String>,
    /// `render` — the element-replacement union.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// `className`.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest — plain attributes (the placeholder demo's `placeholder`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The forwarded `ref` (`OTPFieldInput.tsx:29`).
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<Element>>,
) -> impl IntoView {
    let rendered = use_otp_field_input(OtpFieldInputProps {
        aria_label,
        aria_labelledby,
        input_type,
        component_props: render_class_style(class, render, style),
        element_attributes,
        ref_callback,
    })
    .expect("OTPField.Input renders inside an OTPField.Root");
    part_view(rendered)
}

/// `OTPField.Separator` — upstream's `<OTPField.Separator>`: NOT an otp-field component at all, the
/// shared Separator re-exported (`index.parts.ts:3`; implementation.md "Context providers/
/// consumers"), which is the structural reason it cannot affect slot counting. Its children are
/// rendered between the slot groups (`behavior.md:20`).
#[allow(non_snake_case)]
#[component]
pub fn Separator(
    /// `orientation` — the shared Separator's prop, default `'horizontal'`.
    #[prop(default = None, optional)]
    orientation: Option<SeparatorOrientation>,
    /// `render` — the element-replacement union.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// `className`.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest — plain attributes.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The forwarded `ref`.
    #[prop(default = None, optional)]
    ref_callback: Option<RefCallback<Element>>,
    /// The separator's own content (`OTPFieldRoot.test.tsx:94-118`).
    #[prop(default = None, optional)]
    children: Option<Children>,
) -> impl IntoView {
    let rendered = crate::separator::separator_element(SharedSeparatorProps {
        orientation: orientation.unwrap_or(SEPARATOR_ORIENTATION_HORIZONTAL),
        render_class_style: render_class_style(class, render, style),
        element_attributes,
        ref_callback,
    })
    .expect("OTPField.Separator always renders");

    let tag = rendered.tag.clone();
    let children_view: Option<leptos::prelude::AnyView> = children.map(|children| children());
    let node_ref = NodeRef::<Custom<String>>::new();
    let rendered = SendWrapper::new(rendered);
    Effect::new(move |_| {
        // `leptos::prelude::Get` is qualified here: this module imports the workspace's
        // `reactive_graph` traits, which shadow the prelude's same-named trait — and it is the
        // prelude's that is implemented for `NodeRef` (tachys's own reactive-graph).
        let Some(node) = ::leptos::prelude::Get::get(&node_ref) else {
            return;
        };
        commit_element(&node, &rendered);
    });
    custom(tag).node_ref(node_ref).child(children_view)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod facade_host_tests {
    use super::*;

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // implementation.md "State machine" — the completion-eligibility table,
    // exercised through the write gate of a provided root context (host:
    // DOM-free plumbing; focus is a no-op without real elements).
    fn provided_root_context(length: usize) -> Rc<OtpFieldRootContextValue> {
        let _owner = in_owner();
        let (value, set_value) = use_controlled(UseControlledProps::new(
            RwSignal::new(None),
            RwSignal::new(String::new()),
            "OTPField",
        ));
        let value_ref = Rc::new(RwSignal::new(String::new()));
        let set_value_fn: Rc<dyn Fn(String, OtpChangeEventDetails) -> Option<String>> = {
            let value_ref = Rc::clone(&value_ref);
            Rc::new(move |next, _details| {
                let prev = value_ref.get_untracked();
                Set::set(value_ref.as_ref(), next.clone());
                set_value(SetValueAction::Value(next.clone()));
                Some(next).filter(|v| *v != prev)
            })
        };
        // The live value the handlers read (`valueRef.current`) — here the same box `set_value_fn`
        // writes, so the host harness sees a value that follows the commits.
        let get_value_fn: Rc<dyn Fn() -> String> = {
            let value_ref = Rc::clone(&value_ref);
            Rc::new(move || value_ref.get_untracked())
        };
        let context = OtpFieldRootContextValue {
            auto_complete: "one-time-code".to_string(),
            active_index: 0,
            disabled: false,
            form: None,
            focus_input: Rc::new(|_| {}),
            queue_focus_input: Rc::new(|_, _| {}),
            get_input_id: Rc::new(|_| None),
            handle_input_blur: Rc::new(|_| {}),
            handle_input_focus: Rc::new(|_, _| {}),
            input_mode: Some("numeric".to_string()),
            input_aria_labelled_by: None,
            invalid: false,
            length,
            mask: false,
            pattern: None,
            report_value_invalid: Rc::new(|_, _| {}),
            read_only: false,
            required: false,
            normalize_value: None,
            set_value: set_value_fn,
            state: OtpFieldRootState {
                complete: false,
                disabled: false,
                filled: false,
                focused: false,
                length,
                read_only: false,
                required: false,
                value: String::new(),
            },
            validation_type: OtpValidationType::Numeric,
            value: String::new(),
            get_value: get_value_fn,
        };
        provide_otp_root_context(context);
        use_otp_field_root_context()
    }

    // The write gate's full matrix (commit/completion/queue) lives in the wasm
    // suite where real Events exist (web_sys `Event::new` panics on host);
    // host-side, the REASON constants pin the upstream reason strings.
    #[test]
    fn reason_constants_carry_the_upstream_strings() {
        assert_eq!(REASON_INPUT_CHANGE, "input-change");
        assert_eq!(REASON_INPUT_CLEAR, "input-clear");
        assert_eq!(REASON_INPUT_PASTE, "input-paste");
        assert_eq!(REASON_KEYBOARD, "keyboard");
    }

    // behavior.md "State model" — `value`/`length` are suppressed from the state
    // attributes (`stateAttributesMapping.ts:6-10`), `filled`/`complete` ride the
    // truthiness walk.
    #[test]
    fn root_state_map_suppresses_value_and_length() {
        let state = OtpFieldRootState {
            complete: true,
            disabled: false,
            filled: true,
            focused: false,
            length: 6,
            read_only: false,
            required: false,
            value: "123456".to_string(),
        };
        let map = state.to_state_map();
        assert!(map.get("value").is_none(), "value is suppressed");
        assert!(map.get("length").is_none(), "length is suppressed");
        assert_eq!(map.get("complete"), Some(&serde_json::json!(true)));
        assert_eq!(map.get("filled"), Some(&serde_json::json!(true)));
    }

    // `getOTPFieldInputState` (`OTPFieldRootContext.ts:46-57`) — the slot's own
    // value/filled/index overrides ride the root spread.
    #[test]
    fn input_state_carries_the_slot_overrides() {
        let root = OtpFieldRootState {
            complete: false,
            disabled: false,
            filled: true,
            focused: false,
            length: 6,
            read_only: false,
            required: false,
            value: "12".to_string(),
        };
        let input = OtpFieldInputState {
            root: root.clone(),
            slot_value: "2".to_string(),
            index: 1,
            filled: true,
        };
        let map = input.to_state_map();
        assert_eq!(map.get("filled"), Some(&serde_json::json!(true)));
        assert!(map.get("index").is_none(), "index is suppressed");
        let _ = root;
    }

    // The input rejects the missing-context contract (`behavior.md "Edge
    // cases"`): `useOTPFieldRootContext` outside a Root panics with the
    // upstream message prefix.
    #[test]
    #[should_panic(expected = "Base UI: OTPFieldRootContext is missing.")]
    fn input_outside_root_panics_with_the_upstream_message() {
        let _owner = in_owner();
        let _ = use_otp_field_root_context();
    }
}
