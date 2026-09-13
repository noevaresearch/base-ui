//! Port of the Base UI Progress — the `library: progress` TODO item
//! (`specs/library/progress/behavior.md`, `specs/library/progress/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **No state machine, no events, no focus** (implementation.md "Dependencies":
//!   "Explicitly not used by this unit: `floating-ui-react`, `use-render`,
//!   `useControlled`, `useStableCallback`, portals, or the owner utilities —
//!   Progress has no events, focus, or positioning. For dependency-graph purposes
//!   this is the lightest tier of Base UI component"): the only React state in the
//!   unit is the label id (`ProgressRoot.tsx:36`), and every value-related output
//!   (`aria-valuenow`, formatted text, valuetext, indicator width) is derived inline
//!   during render from the props. The port mirrors this: the derivation runs once
//!   in the component body (the run-once component model), and the label id is the
//!   only reactive state — the meter's twin (the `meter.rs` port, done-marked
//!   4312186f1, same `valueToPercent → clamp → formatNumber` pipeline shape).
//! - **The value derivation pipeline** (`ProgressRoot.tsx:38-60`): the guarded arm —
//!   `value != null && Number.isFinite(value)` — computes `rawPercentage =
//!   valueToPercent(value, min, max)`, `percentageValue = clamp(NaN ? 0 :
//!   rawPercentage, 0, 100)`, `clampedValue = clamp(value, min, max)`, `status =
//!   clampedValue === max ? 'complete' : 'progressing'`, and `formattedValue` =
//!   the **clamped** value through `format` or the **percentage** as
//!   `{ style: 'percent' }` (`:56-58`). Note the pipeline difference from meter:
//!   progress clamps `value` **without** a NaN interception arm (the guarded arm
//!   excludes non-finite values upstream of the clamp, so meter's `NaN → min`
//!   fallback is unreachable here), and progress derives a three-state `status`
//!   (`ProgressRoot.tsx:42-47,53`) where meter had none.
//! - **Three statuses drive the data-attributes** (`ProgressRoot.test.tsx:64-118`):
//!   `data-indeterminate`, `data-progressing`, `data-complete` — exactly one
//!   present at a time, on every part (root, label, value, track, indicator) via
//!   `progressStateAttributesMapping` (`stateAttributesMapping.ts:7-21`). The port
//!   walks the same custom mapping through the ported
//!   [`leptos_ui_internals::state_attributes::get_state_attributes_props`] engine —
//!   the real state→attribute walk upstream's `useRenderElement` performs — instead
//!   of a hardcoded attribute list.
//! - **The root carries the entire ARIA surface** (`ProgressRoot.tsx:64-72`):
//!   `role="progressbar"`, `aria-valuemin`/`aria-valuemax` always, `aria-valuenow`
//!   the **clamped** value (omitted entirely when indeterminate — `:68`'s
//!   `clampedValue ?? undefined`), `aria-valuetext` the `getAriaValueText(formatted,
//!   raw)` override or the default (`'indeterminate progress'` when indeterminate,
//!   the formatted value otherwise), and `aria-labelledby` the state-lifted label
//!   id. The user's own `aria-valuetext`/`aria-labelledby` props (the rightmost
//!   `elementProps` bag) override the internal values — the meter's
//!   `aria_valuetext`/`aria_labelledby` prop convention.
//! - **The hidden NVDA workaround span** (`ProgressRoot.tsx:76-78`,
//!   mui/base-ui#4184): a `role="presentation"` visually-hidden `<span>` containing
//!   the text `x` appended after the user's children — the meter's precedent, the
//!   port renders it from the ported [`leptos_ui_utils::VISUALLY_HIDDEN`].
//! - **The context carries exactly five members** (`ProgressRootContext.tsx:9-24`):
//!   `formattedValue`, `percentageValue`, `value` (the raw prop), `setLabelId`, and
//!   `state` (the `{ status }` record). Consumption: Label reads `setLabelId` +
//!   `state`, Indicator `percentageValue` + `state`, Value `value` +
//!   `formattedValue` + `state`, and Track `state` only (`ProgressTrack.tsx:19` —
//!   unlike meter's context-free passthrough, progress's Track DOES read context,
//!   `ProgressTrack.tsx:19`, and so throws outside a Root like the other parts,
//!   behavior.md's parts-outside-Root item — proven for Label
//!   `ProgressLabel.test.tsx:49-59`, the shared context means all parts share the
//!   guard).
//! - **Indicator sizing is inline CSS** (`ProgressIndicator.tsx:24-30`):
//!   `insetInlineStart: 0`, `height: 'inherit'`, `width: {percentage}%` — but only
//!   when determinate; indeterminate carries NO inline width
//!   (`indicator.style.width === ''`, `ProgressRoot.test.tsx:86`).
//! - **`ProgressValue` marks its visible text `aria-hidden: true`**
//!   (`ProgressValue.tsx:34`) and follows `status` rather than re-deriving
//!   indeterminacy (`:28-30`): the render function receives `('indeterminate',
//!   rawValue)` for both `value=null` and `value=NaN` (`ProgressValue.test.tsx:66-79`
//!   — the fixed first-argument string, NOT the empty formatted value), while the
//!   no-children arm renders the formatted value or nothing when indeterminate
//!   (`:30-32`).
//!
//! ## Rust adaptations
//!
//! - **`value: number | null` is `Option<f64>`.** The required-with-null prop
//!   (`ProgressRootProps.value`, `:146`) is the unit's indeterminacy sentinel; the
//!   non-finite arm (`NaN`, `±Infinity` — `ProgressRoot.test.tsx:246-258`) rides
//!   the same `Number.isFinite` guard.
//! - **The pure prop re-derivation is fixed at body time.** Upstream re-derives on
//!   every render because React re-renders on prop change; Leptos runs the body
//!   once, so a changing `value` rides a rebuild of the subtree (the meter/
//!   toggle-hero-demo convention — the ported `use_composite_root`'
//!   "reactive re-runs discharged to the view layer" rule).
//! - **`formatNumber` rides the already-ported [`leptos_ui_utils::format_number`]**
//!   — the wasm `Intl` wrapper with the module-level formatter cache. The `format`
//!   prop is the JSON object of `Intl.NumberFormatOptions`; the default arm builds
//!   the `{ style: 'percent' }` object literal (the meter's `derive_formatted_value`).
//! - **The context value crosses `provide_context` inside a [`SendWrapper`]** (the
//!   meter/composite_root_context precedent) because `LabelIdSetter` is an `Rc`.
//!   The read goes through the **same runtime the tree provides under** — leptos
//!   0.7's `use_context` (the cross-crate owner-bridge trap meter's first wasm run
//!   caught, fixed in b9b107c12).
//! - The discarded `'use client'` directives are N/A.

use std::rc::Rc;
use std::sync::Arc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set, Update};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsValue;

use leptos_ui_internals::state_attributes::{
    StateAttributeProps, StateAttributesMapping, get_state_attributes_props,
};
use leptos_ui_internals::use_registered_label_id::{
    LabelIdSetter, LabelIdUpdate, use_registered_label_id,
};
use leptos_ui_internals::value_to_percent::value_to_percent;
use leptos_ui_utils::clamp::clamp;
use leptos_ui_utils::format_number::format_number;
use leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN;

/// `ProgressStatus` (`ProgressRoot.tsx:106`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProgressStatus {
    /// `'indeterminate'`.
    Indeterminate,
    /// `'progressing'`.
    Progressing,
    /// `'complete'`.
    Complete,
}

impl ProgressStatus {
    /// The `ProgressRootDataAttributes` name (`ProgressRootDataAttributes.ts:1-11`,
    /// via `stateAttributesMapping.ts:8-19`) — the `data-*` attribute the mapping
    /// emits for the status.
    pub fn data_attribute(self) -> &'static str {
        match self {
            ProgressStatus::Indeterminate => "data-indeterminate",
            ProgressStatus::Progressing => "data-progressing",
            ProgressStatus::Complete => "data-complete",
        }
    }

    /// The serde_json encoding the ported state walk consumes (`ProgressRoot.tsx:62`'s
    /// `{ status }` state record).
    pub fn as_json_value(self) -> serde_json::Value {
        match self {
            ProgressStatus::Indeterminate => "indeterminate".into(),
            ProgressStatus::Progressing => "progressing".into(),
            ProgressStatus::Complete => "complete".into(),
        }
    }
}

/// `progressStateAttributesMapping` (`stateAttributesMapping.ts:7-21`): the custom
/// mapping keyed on `status` — `progressing` → `data-progressing`, `complete` →
/// `data-complete`, `indeterminate` → `data-indeterminate`, anything else → `null`
/// (the `Some(None)` decline arm the engine skips).
pub fn progress_state_attributes_mapping(
    key: &str,
    _value: &serde_json::Value,
) -> Option<Option<StateAttributeProps>> {
    if key != "status" {
        return None;
    }
    // The `status` string re-derives the enum (the mapping's `value ===` chain).
    let attribute = match _value.as_str() {
        Some("progressing") => ProgressStatus::Progressing,
        Some("complete") => ProgressStatus::Complete,
        Some("indeterminate") => ProgressStatus::Indeterminate,
        _ => return Some(None),
    };
    let mut props = StateAttributeProps::new();
    props.insert(attribute.data_attribute().to_string(), String::new());
    Some(Some(props))
}

/// The `ProgressRootContext` members (`ProgressRootContext.tsx:9-24`).
#[derive(Clone)]
pub struct ProgressRootContextValue {
    /// `formattedValue` — the formatted clamped value; `''` while indeterminate.
    pub formatted_value: String,
    /// `percentageValue` — the value normalized to a `0`–`100` percentage of the
    /// range, clamped to those bounds; `None` while indeterminate (`:13-16`).
    pub percentage_value: Option<f64>,
    /// `value` — the raw prop, `None` when indeterminate (`:19-21`).
    pub value: Option<f64>,
    /// `setLabelId` — the write side of the label-id lift (`:22`).
    pub set_label_id: LabelIdSetter,
    /// `state` — the `{ status }` record (`:23`).
    pub state: ProgressStatus,
}

/// `useProgressRootContext` (`ProgressRootContext.tsx:30-42`): the context read plus
/// the missing-root guard — panics with the upstream `Base UI:`-prefixed error when
/// no `ProgressRoot` is an ancestor (`:34-39`; proven for Label,
/// `ProgressLabel.test.tsx:49-59`).
///
/// The read goes through the **same runtime the component tree provides under** —
/// leptos 0.7's context API (the meter's cross-crate owner-bridge note: the
/// internals crate's rg-0.2 runtime is a second, independent runtime, and only the
/// leptos-runtime read connects provider to consumer under a leptos mount).
pub fn use_progress_root_context() -> ProgressRootContextValue {
    use_context::<SendWrapper<ProgressRootContextValue>>()
        .expect(
            "Base UI: ProgressRootContext is missing. Progress parts must be placed within <Progress.Root>.",
        )
        .take()
}

/// The guarded-arm derivation (`ProgressRoot.tsx:42-60`), extracted so the
/// degenerate-input matrix is host-testable: `None` value or any non-finite value
/// stays indeterminate (`status = 'indeterminate'`, percentage `None`, clamped
/// `None`); a finite value runs `valueToPercent → clamp` and picks the status.
/// Returns `(status, percentage_value, clamped_value)`.
pub(crate) fn derive_progress_values(
    value: Option<f64>,
    min: f64,
    max: f64,
) -> (ProgressStatus, Option<f64>, Option<f64>) {
    match value {
        // The guard arm: `value != null && Number.isFinite(value)` (`:49`).
        Some(v) if v.is_finite() => {
            let raw_percentage = value_to_percent(v, min, max);
            let percentage_value = if raw_percentage.is_nan() {
                0.0
            } else {
                clamp(raw_percentage, 0.0, 100.0)
            };
            let clamped_value = clamp(v, min, max);
            let status = if clamped_value == max {
                ProgressStatus::Complete
            } else {
                ProgressStatus::Progressing
            };
            (status, Some(percentage_value), Some(clamped_value))
        }
        // `null`/`NaN`/`±Infinity` — the indeterminate defaults (`:42-47`).
        _ => (ProgressStatus::Indeterminate, None, None),
    }
}

/// The `formattedValue` derivation (`ProgressRoot.tsx:56-58`): with `format` the
/// **clamped** value goes through `formatNumber(clampedValue, locale, format)`;
/// without it the **percentage** is formatted as `{ style: 'percent' }`. Indeterminate
/// keeps the `''` default (`:45`).
pub(crate) fn derive_formatted_value(
    status: ProgressStatus,
    percentage_value: Option<f64>,
    clamped_value: Option<f64>,
    locale: Option<&str>,
    format: Option<&serde_json::Value>,
) -> String {
    match (status, percentage_value, clamped_value) {
        (ProgressStatus::Indeterminate, _, _) => String::new(),
        (_, Some(pct), Some(clamped)) => {
            let locale_value = locale.map(JsValue::from_str).unwrap_or(JsValue::UNDEFINED);
            match format {
                Some(options) => {
                    let options_value = js_sys::JSON::parse(
                        &serde_json::to_string(options).expect("the format options serialize"),
                    )
                    .expect("the format options parse as JSON");
                    format_number(Some(clamped), &locale_value, &options_value)
                }
                None => {
                    let options = js_sys::Object::new();
                    js_sys::Reflect::set(&options, &"style".into(), &"percent".into())
                        .expect("the style option sets");
                    format_number(Some(pct / 100.0), &locale_value, &options.into())
                }
            }
        }
        // Unreachable by construction (progressing/complete always carry both), kept
        // total for the match.
        _ => String::new(),
    }
}

/// The `ProgressRoot` props (`ProgressRootProps`, `ProgressRoot.tsx:115-147`). The
/// component macro expands a struct of its own from these fields (the accordion's
/// individual-`#[prop]` convention).
///
/// - `value` (`:146`) — the current value; `None` (upstream `null`) or any
///   non-finite value renders the indeterminate state. The one required prop.
/// - `min` (`:137-139` — upstream default `0`), `max` (`:132-134` — `100`).
/// - `locale` (`:127-129`) — the `Intl.NumberFormat` locale; `None` is the runtime
///   default locale.
/// - `format` (`:118-120`) — the `Intl.NumberFormatOptions` object; `None` selects
///   the default percent arm.
/// - `get_aria_value_text` (`:121-123`) — receives `(formattedValue, value)` (the
///   formatted *clamped* value first, the **raw** prop second, `('', null)` when
///   indeterminate) and overrides only the aria string.
/// - `aria_valuetext` (`:114-116`) — the user's `'aria-valuetext'` prop; the
///   rightmost `elementProps` bag overrides the internal value.
/// - `aria_labelledby` — the user's `'aria-labelledby'` prop; overrides the
///   label-registered id per the same rightmost-bag rule.
/// - `class` — the `className` passthrough.
#[leptos::component]
pub fn ProgressRoot(
    /// `value` (`:146`) — `None` renders the indeterminate state. A REQUIRED prop of
    /// `Option` type (no `#[prop(optional)]` auto-wrap): upstream's
    /// `value: number | null` is likewise a required prop whose `null` is the
    /// indeterminacy sentinel, and a required `Option` prop keeps `value=None`
    /// expressible at the view call site.
    value: Option<f64>,
    /// `min` (`:139` — upstream default `0`).
    #[prop(default = 0.0, optional)]
    min: f64,
    /// `max` (`:134` — upstream default `100`).
    #[prop(default = 100.0, optional)]
    max: f64,
    /// `locale` (`:129`).
    #[prop(default = None, optional)]
    locale: Option<String>,
    /// `format` (`:120`).
    #[prop(default = None, optional)]
    format: Option<serde_json::Value>,
    /// `getAriaValueText` (`:122`).
    #[prop(default = None, optional)]
    get_aria_value_text: Option<Arc<dyn Fn(&str, Option<f64>) -> String + Send + Sync>>,
    /// The user's `aria-valuetext` prop (`:115`).
    #[prop(default = None, optional)]
    aria_valuetext: Option<String>,
    /// The user's `aria-labelledby` prop.
    #[prop(default = None, optional)]
    aria_labelledby: Option<String>,
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The user's children — the parts subtree. Optional.
    #[prop(default = None, optional)]
    children: Option<leptos::children::Children>,
) -> impl leptos::IntoView {
    // The destructuring (`:22-34`). The only React state in the unit is the label id
    // (`:36`) — the port mirrors it as the unit's only signal.
    let label_id = RwSignal::new(None::<String>);
    let set_label_id: LabelIdSetter = {
        let label_id = label_id.clone();
        Rc::new(move |update: LabelIdUpdate| match update {
            LabelIdUpdate::Set(next) => label_id.set(next),
            LabelIdUpdate::ClearIfCurrent(id) => {
                // The functional-updater guard (`useRegisteredLabelId.ts:15`).
                label_id.update(|current| {
                    if current.as_deref() == Some(id.as_str()) {
                        *current = None;
                    }
                });
            }
        })
    };

    // The derivation pipeline (`:42-60`).
    let (status, percentage_value, clamped_value) = derive_progress_values(value, min, max);
    let formatted_value = derive_formatted_value(
        status,
        percentage_value,
        clamped_value,
        locale.as_deref(),
        format.as_ref(),
    );

    // `ariaValuetext` (`:69-71`): the callback receives the formatted value (the ''
    // indeterminate default — `('', null)` per `ProgressRoot.test.tsx:279-282`) and
    // the raw prop; without it the derived default (`:47,59`).
    let internal_aria_valuetext = match &get_aria_value_text {
        Some(get) => get(formatted_value.as_str(), value),
        None => {
            if status == ProgressStatus::Indeterminate {
                "indeterminate progress".to_string()
            } else {
                formatted_value.clone()
            }
        }
    };
    // The user's `aria-valuetext` (the rightmost `elementProps` bag) overrides the
    // internal value.
    let aria_valuetext = aria_valuetext.unwrap_or(internal_aria_valuetext);

    // The context value (`:83-92`) — the five members, provided before the element
    // so the parts' subtree reads it.
    provide_context(SendWrapper::new(ProgressRootContextValue {
        formatted_value: formatted_value.clone(),
        percentage_value,
        set_label_id,
        value,
        state: status,
    }));

    // The state walk (`:94-99`'s `useRenderElement` state + `progressStateAttributesMapping`):
    // the `{ status }` record through the ported `get_state_attributes_props` with
    // the unit's custom mapping — the same engine upstream's render pipeline runs,
    // emitting exactly one status attribute (`stateAttributesMapping.ts:7-21`).
    let mut state_map = serde_json::Map::new();
    state_map.insert("status".to_string(), status.as_json_value());
    let state_attributes = get_state_attributes_props(
        &state_map,
        Some(
            &(progress_state_attributes_mapping
                as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>),
        ),
    );

    // The ARIA surface (`:64-72`) — all on the single root element.
    let aria_labelledby_attr = move || aria_labelledby.clone().or_else(|| label_id.get_untracked());
    let valuenow = clamped_value.map(|v| {
        if v.fract() == 0.0 {
            format!("{}", v as i64)
        } else {
            format!("{v}")
        }
    });
    let valuemin = format_number_default_locale(min);
    let valuemax = format_number_default_locale(max);

    // The materialized state walk (the `:94-99` mapping result) — exactly one of
    // the three slots is `Some`.
    let StatusAttributes {
        indeterminate: data_indeterminate,
        progressing: data_progressing,
        complete: data_complete,
    } = StatusAttributes::from_walk(&state_attributes);

    view! {
        <div
            class={class}
            role="progressbar"
            data-indeterminate={data_indeterminate}
            data-progressing={data_progressing}
            data-complete={data_complete}
            aria-labelledby={aria_labelledby_attr}
            aria-valuemin={valuemin}
            aria-valuemax={valuemax}
            aria-valuenow={valuenow}
            aria-valuetext={aria_valuetext}
        >
            {children.map(|children| children())}
            // The hidden NVDA workaround span (`:76-78`, mui/base-ui#4184): NVDA
            // reads the label only when a presentational text node follows it inside
            // the progress bar. No test observes this node.
            <span role="presentation" style={visually_hidden_style()}>
                "x"
            </span>
        </div>
    }
}

/// Materializes the state walk's output through the three typed attribute slots.
/// Leptos's `view!` has no attribute-spread syntax, so the engine's
/// `StateAttributeProps` result — exactly one of the three fixed-name `data-*`
/// attributes upstream's mapping emits (`stateAttributesMapping.ts:8-19`), with the
/// `''` bare-attribute value — is projected onto `Some('')`-means-present
/// `Option<String>` slots (the names, `ProgressRootDataAttributes.ts:1-11`).
#[derive(Clone, Default)]
pub(crate) struct StatusAttributes {
    /// `data-indeterminate`.
    pub indeterminate: Option<String>,
    /// `data-progressing`.
    pub progressing: Option<String>,
    /// `data-complete`.
    pub complete: Option<String>,
}

impl StatusAttributes {
    /// Projects the walk output onto the three slots (`Some` = the attribute is
    /// present with the mapping's value; the mapping emits `''`).
    pub fn from_walk(attributes: &StateAttributeProps) -> Self {
        StatusAttributes {
            indeterminate: attributes.get("data-indeterminate").cloned(),
            progressing: attributes.get("data-progressing").cloned(),
            complete: attributes.get("data-complete").cloned(),
        }
    }
}

/// The `visuallyHidden` style declarations serialized for the `style` attribute —
/// the `style={visuallyHidden}` object spread (`ProgressRoot.tsx:76`) over the
/// ported constants (the meter's helper).
pub(crate) fn visually_hidden_style() -> String {
    VISUALLY_HIDDEN
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `aria-valuemin`/`aria-valuemax` ride React's number-to-string serialization
/// (`:66-67`); the port formats through the same integer/float split `aria-valuenow`
/// uses (the meter's helper).
fn format_number_default_locale(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// The `Progress.Label` component (`ProgressLabel.tsx` body): reads `setLabelId` +
/// `state` (`:20`), registers its id through the ported [`use_registered_label_id`]
/// (`:22`), and renders the `role="presentation"` span (`:28-31`) carrying `id` +
/// the user's props and the status attribute from the shared state walk.
#[leptos::component]
pub fn ProgressLabel(
    /// The `id` prop (`ProgressLabel.tsx:20` — a user-supplied id wins, otherwise a
    /// `base-ui-`-prefixed id is generated).
    #[prop(default = None, optional)]
    id: Option<String>,
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    children: leptos::children::Children,
) -> impl leptos::IntoView {
    let ProgressRootContextValue {
        set_label_id,
        state,
        ..
    } = use_progress_root_context();

    // `const id = useRegisteredLabelId(idProp, setLabelId)` (`:22`) — the
    // registration effect and its clear-if-current cleanup are the ported hook's
    // suite-tested contracts. The id signal is read untracked into the attribute
    // closure (the meter's dialog-parts convention).
    let id_signal = use_registered_label_id(RwSignal::new(id), set_label_id);
    let registered_id = move || id_signal.get_untracked();

    // The shared state walk (the root's mapping; `:32-37`'s
    // `stateAttributesMapping: progressStateAttributesMapping`).
    let StatusAttributes {
        indeterminate: data_indeterminate,
        progressing: data_progressing,
        complete: data_complete,
    } = StatusAttributes::from_walk(&progress_state_attributes(state));

    view! {
        <span
            class={class}
            role="presentation"
            id={registered_id}
            data-indeterminate={data_indeterminate}
            data-progressing={data_progressing}
            data-complete={data_complete}
        >
            {children()}
        </span>
    }
}

/// The shared per-part state walk: the `{ status }` record through
/// `get_state_attributes_props` with the unit's custom mapping — the parts'
/// `useRenderElement` calls all pass the same `stateAttributesMapping`
/// (`ProgressLabel.tsx:37`, `ProgressValue.tsx:39`, `ProgressTrack.tsx:21`,
/// `ProgressIndicator.tsx:33`), so all parts carry the same status attribute.
fn progress_state_attributes(status: ProgressStatus) -> StateAttributeProps {
    let mut state_map = serde_json::Map::new();
    state_map.insert("status".to_string(), status.as_json_value());
    get_state_attributes_props(
        &state_map,
        Some(
            &(progress_state_attributes_mapping
                as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>),
        ),
    )
}

/// The `Progress.Track` component (`ProgressTrack.tsx` body): reads `state` from
/// the context (`:19`) and renders the structural `<div>` carrying the status
/// attribute (`:23-27`) — unlike meter's context-free passthrough, progress's
/// Track DOES consume the context, so it throws outside a Root like the other
/// parts.
#[leptos::component]
pub fn ProgressTrack(
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    children: leptos::children::Children,
) -> impl leptos::IntoView {
    let ProgressRootContextValue { state, .. } = use_progress_root_context();

    let StatusAttributes {
        indeterminate: data_indeterminate,
        progressing: data_progressing,
        complete: data_complete,
    } = StatusAttributes::from_walk(&progress_state_attributes(state));

    view! {
        <div
            class={class}
            data-indeterminate={data_indeterminate}
            data-progressing={data_progressing}
            data-complete={data_complete}
        >
            {children()}
        </div>
    }
}

/// The `Progress.Indicator` component (`ProgressIndicator.tsx` body): reads
/// `percentageValue` + `state` (`:21`), and when determinate applies the inline-CSS
/// fill — `insetInlineStart: 0`, `height: 'inherit'`, `width: {percentage}%`
/// (`:24-30`); indeterminate carries **no** inline width (`:24`'s `{}` arm —
/// `indicator.style.width === ''`, `ProgressRoot.test.tsx:86`).
#[leptos::component]
pub fn ProgressIndicator(
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
) -> impl leptos::IntoView {
    let ProgressRootContextValue {
        percentage_value,
        state,
        ..
    } = use_progress_root_context();

    // `indicatorStyle` (`:24-30`): the `{}` empty arm when `percentageValue == null`.
    let style = match percentage_value {
        None => String::new(),
        Some(pct) => format!("inset-inline-start: 0; height: inherit; width: {pct}%;"),
    };
    let StatusAttributes {
        indeterminate: data_indeterminate,
        progressing: data_progressing,
        complete: data_complete,
    } = StatusAttributes::from_walk(&progress_state_attributes(state));

    view! {
        <div
            class={class}
            style={style}
            data-indeterminate={data_indeterminate}
            data-progressing={data_progressing}
            data-complete={data_complete}
        />
    }
}

/// The `Progress.Value` component (`ProgressValue.tsx` body): reads `value` +
/// `formattedValue` + `state` (`:24`), marks the visible text `aria-hidden: true`
/// (`:34`), and follows `status` rather than re-deriving indeterminacy (`:28-30`):
/// the render-function children receive `('indeterminate', rawValue)` while
/// indeterminate (the fixed first-argument string, `ProgressValue.test.tsx:66-79`),
/// the formatted value otherwise; without children the element renders the
/// formatted value or nothing (`:30-32`).
///
/// The render function takes the pair as a `(&str, Option<f64>)` argument — the
/// Leptos spelling of upstream's `children(formattedValueArg, value)` (`:36`): a
/// `ChildrenFn` receives no arguments, so the port uses the two-argument closure
/// type directly (the meter's `MeterValue` convention), wrapped in the
/// `ChildrenFn`-shaped `Option`.
#[leptos::component]
pub fn ProgressValue(
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The render-function children arm (`:41-47`) — receives
    /// `('indeterminate', raw)` or `(formattedValue, raw)`; omission renders the
    /// formatted value (or nothing when indeterminate).
    #[prop(default = None, optional)]
    children: Option<Box<dyn Fn(&str, Option<f64>) -> AnyView + Send>>,
) -> impl leptos::IntoView {
    let ProgressRootContextValue {
        value,
        formatted_value,
        state,
        ..
    } = use_progress_root_context();

    // `const indeterminate = state.status === 'indeterminate'` (`:28`) — following
    // `status` rather than re-deriving from `value` (`:27-29`'s comment).
    let indeterminate = state == ProgressStatus::Indeterminate;
    let formatted_value_arg = if indeterminate {
        "indeterminate".to_string()
    } else {
        formatted_value.clone()
    };
    // `formattedValueDisplay` (`:30`): `null` while indeterminate — the no-children
    // arm renders nothing.
    let display: Option<String> = if indeterminate {
        None
    } else {
        Some(formatted_value.clone())
    };

    // `children(formattedValueArg, value)` (`:36`) — the closure receives the pair
    // fresh from the context read on every derivation.
    let text = move || match &children {
        Some(render) => render(formatted_value_arg.as_str(), value).into_any(),
        None => display.clone().unwrap_or_default().into_any(),
    };

    let StatusAttributes {
        indeterminate: data_indeterminate,
        progressing: data_progressing,
        complete: data_complete,
    } = StatusAttributes::from_walk(&progress_state_attributes(state));

    view! {
        <span
            class={class}
            aria-hidden="true"
            data-indeterminate={data_indeterminate}
            data-progressing={data_progressing}
            data-complete={data_complete}
        >
            {text}
        </span>
    }
}

// The `JsValue` import serves the JSON/Reflect boundary in `derive_formatted_value`
// on both targets; `js_sys` rides the internals crate's re-exports through
// `format_number`.
