//! Port of the Base UI Meter — the `library: meter` TODO item
//! (`specs/library/meter/behavior.md`, `specs/library/meter/implementation.md`).
//!
//! Upstream's structural facts this port follows (implementation.md):
//!
//! - **No state machine** ("State machine / hooks used", implementation.md:11-16): the
//!   meter is a display-only, fully-controlled component; the only React state in the
//!   unit is the label id (`MeterRoot.tsx:35`), and every value-related output (aria
//!   attributes, formatted text, indicator width) is derived inline during render from
//!   the props. The port mirrors this: the derivation runs once in the component body
//!   (the run-once component model), and the label id is the only reactive state.
//! - **The value derivation pipeline** (`MeterRoot.tsx:37-51`): `rawPercentage =
//!   valueToPercent(value, min, max)`, `percentageValue = clamp(NaN ? 0 :
//!   rawPercentage, 0, 100)` (the `:37` comment: `clamp` handles infinity, NaN must be
//!   intercepted), `clampedValue = clamp(NaN ? min : value, min, max)` (NaN falls back
//!   to `min`, implementation.md untested item 6), `formattedValue` = the **clamped**
//!   value through `format` or the **percentage** as `{ style: 'percent' }` (`:44-46`),
//!   and `ariaValuetext` = `getAriaValueText(formattedValue, valueProp)` — the formatted
//!   *clamped* value first, the **raw** prop second (`:48-51`), overriding only the aria
//!   string, never the visible text.
//! - **The context carries exactly four members** (`MeterRootContext.ts:4-12`):
//!   `formattedValue`, `percentageValue`, `setLabelId`, and `value` (the raw,
//!   unclamped prop — `MeterRoot.tsx:75`). Consumption: Label reads only `setLabelId`,
//!   Indicator only `percentageValue`, Value `value` + `formattedValue`, and Track
//!   reads **nothing** (implementation.md untested item 4 — Track is the only part that
//!   never calls `useMeterRootContext`, so it renders outside a Root without throwing).
//! - **The root carries the entire ARIA surface** (`MeterRoot.tsx:53-59`): `role="meter"`
//!   plus `aria-valuemin`/`aria-valuemax`/`aria-valuenow`/`aria-valuetext`/
//!   `aria-labelledby` all on the single root element; the parts contribute no aria of
//!   their own. The user's `aria-valuetext` prop (in `elementProps`, the rightmost bag)
//!   overrides the internal value (mergeProps rightmost-wins; implementation.md
//!   untested item 7) — the port spells that as the optional `aria_valuetext` prop.
//! - **Label association by state-lifted id** (`useRegisteredLabelId.ts:12-17`): the
//!   label registers its own id upward through `setLabelId` and the root renders
//!   `aria-labelledby` from that state — the already-ported
//!   [`leptos_ui_internals::use_registered_label_id`], whose registration cycle and
//!   clear-if-current unmount are suite-tested there (the labelable-provider
//!   checkpoint).
//! - **The hidden NVDA workaround span** (`MeterRoot.tsx:63-65`, mui/base-ui#4184): a
//!   `role="presentation"` visually-hidden `<span>` containing the text `x` appended
//!   after the user's children — asserted by no test (implementation.md untested item
//!   1) but load-bearing for screen readers; the port renders it from the ported
//!   [`leptos_ui_utils::VISUALLY_HIDDEN`] constants.
//! - **Indicator sizing is inline CSS, not measurement** (`MeterIndicator.tsx:26-30`):
//!   `width: {percentage}%"` anchored with `insetInlineStart: 0` and `height: inherit`
//!   — the logical inline-start property is RTL-correct by construction.
//! - **`MeterValue` marks its visible text `aria-hidden: true`** (`MeterValue.tsx:26`)
//!   because the root's `aria-valuetext` already exposes the value; its children accept
//!   only `null` or a render function (`:36-41`) — any non-function children are
//!   replaced with the formatted string (`:27`; behavior.md over-claims "a node",
//!   implementation.md untested item 5). The port's optional `children` prop is the
//!   render-function arm; omission renders the formatted value.
//!
//! ## Rust adaptations
//!
//! - **The pure prop re-derivation is fixed at body time.** Upstream re-derives
//!   everything on every render because React re-renders on prop change; Leptos runs
//!   the component body once, so a changing `value` rides a rebuild of the subtree (the
//!   `toggle_hero_demo` Effect-driven rebuild precedent — the
//!   `use_composite_root`/`set_disabled_indices` "reactive re-runs discharged to the
//!   view layer" convention). The pure derivation is extracted into
//!   [`derive_meter_values`] so the clamping matrix is host-testable.
//! - **`formatNumber` rides the already-ported [`leptos_ui_utils::format_number`]** —
//!   the wasm `Intl` wrapper with the module-level formatter cache
//!   (`packages/utils/src/formatNumber.ts:3-17`). The `format` prop is the JSON object
//!   of `Intl.NumberFormatOptions`; the default arm builds the `{ style: 'percent' }`
//!   object literal.
//! - **The context value crosses `provide_context` inside a [`SendWrapper`]** (the
//!   `composite_root_context.rs` precedent) because `LabelIdSetter` is an `Rc`.
//! - The discarded `'use client'` directives are N/A. No events, no focus, no portals,
//!   no state attributes (all `Meter*State` interfaces are empty — implementation.md
//!   untested item 10), no `stateAttributesMapping` — `useRenderElement`'s state walk
//!   never fires for this unit.
//! - The empty-state render-callback `state` argument (item 10) has no Rust arm: the
//!   parts expose no render-prop here — the port's `render_class_style` vocabulary is
//!   accepted on the root facade only through the `class` prop; the `render` prop's
//!   element-override surface is the docs-page/demo layer's concern (the
//!   `use_render` page precedent).

use std::rc::Rc;
use std::sync::Arc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal;
use reactive_graph::traits::{GetUntracked, Set, Update};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsValue;

use leptos_ui_internals::use_registered_label_id::{
    LabelIdSetter, LabelIdUpdate, use_registered_label_id,
};
use leptos_ui_internals::value_to_percent::value_to_percent;
use leptos_ui_utils::clamp::clamp;
use leptos_ui_utils::format_number::format_number;
use leptos_ui_utils::visually_hidden::VISUALLY_HIDDEN;

/// The `MeterRootContext` members (`MeterRootContext.ts:4-12`): `formattedValue`,
/// `percentageValue`, `setLabelId`, `value` (the raw, unclamped prop).
#[derive(Clone)]
pub struct MeterRootContextValue {
    /// `formattedValue` — the formatted clamped value (default arm: the percent of the
    /// normalized ratio).
    pub formatted_value: String,
    /// `percentageValue` — the value normalized to a `0`–`100` percentage of the range,
    /// clamped to those bounds (`:6-8`).
    pub percentage_value: f64,
    /// `setLabelId` — the write side of the label-id lift (`:9`).
    pub set_label_id: LabelIdSetter,
    /// `value` — the raw, unclamped prop (`MeterRoot.tsx:75`).
    pub value: f64,
}

/// `useMeterRootContext` (`MeterRootContext.ts:16-25`): the context read plus the
/// missing-root guard — panics with the upstream `Base UI:`-prefixed error when no
/// `MeterRoot` is an ancestor (`:19-21`; the `composite_root_context.rs` required-
/// accessor precedent).
///
/// The read goes through the **same runtime the component tree provides under** —
/// leptos 0.7's context API (which this workspace's `reactive_graph = "0.2"`
/// dependency does NOT share: the internals crate runs a second, independent
/// reactive-graph runtime in the same process, the cross-crate owner bridge
/// documented on the direction-provider docs page). The first wasm run caught the
/// mismatch: `MeterRoot` provided through `leptos::prelude::provide_context`
/// (leptos runtime) while this accessor read through
/// `reactive_graph::owner::use_context` (internals runtime), so every part outside
/// a `MeterRoot` saw "context missing" — and rg-0.2's `provide_context` is a silent
/// no-op without a current rg-0.2 owner, which is why no variant of the split
/// runtime pair could ever connect provider to consumer under a leptos mount.
pub fn use_meter_root_context() -> MeterRootContextValue {
    use_context::<SendWrapper<MeterRootContextValue>>()
        .expect(
            "Base UI: MeterRootContext is missing. Meter parts must be placed within <Meter.Root>.",
        )
        .take()
}

/// The value derivation pipeline (`MeterRoot.tsx:37-40`), extracted verbatim so the
/// degenerate-input matrix is host-testable: `rawPercentage = valueToPercent(value,
/// min, max)`, then `clamp(NaN ? 0 : rawPercentage, 0, 100)` and
/// `clamp(NaN ? min : value, min, max)`.
pub(crate) fn derive_meter_values(value: f64, min: f64, max: f64) -> (f64, f64) {
    // `clamp` handles infinity, but NaN needs an explicit fallback before normalizing
    // range outputs (`:36-37`'s comment).
    let raw_percentage = value_to_percent(value, min, max);
    let percentage_value = if raw_percentage.is_nan() {
        0.0
    } else {
        clamp(raw_percentage, 0.0, 100.0)
    };
    let clamped_value = if value.is_nan() {
        min
    } else {
        clamp(value, min, max)
    };
    (percentage_value, clamped_value)
}

/// The `formattedValue` derivation (`MeterRoot.tsx:43-46`): with `format` the **clamped**
/// value goes through `formatNumber(clampedValue, locale, format)`; without it the
/// **percentage** is formatted as `{ style: 'percent' }`. Formatting the clamped value is
/// what keeps the visible text, `aria-valuetext`, `aria-valuenow`, and the indicator
/// consistent.
pub(crate) fn derive_formatted_value(
    percentage_value: f64,
    clamped_value: f64,
    locale: Option<&str>,
    format: Option<&serde_json::Value>,
) -> String {
    let locale_value = locale.map(JsValue::from_str).unwrap_or(JsValue::UNDEFINED);
    match format {
        Some(options) => {
            let options_value = js_sys::JSON::parse(
                &serde_json::to_string(options).expect("the format options serialize"),
            )
            .expect("the format options parse as JSON");
            format_number(Some(clamped_value), &locale_value, &options_value)
        }
        None => {
            let options = js_sys::Object::new();
            js_sys::Reflect::set(&options, &"style".into(), &"percent".into())
                .expect("the style option sets");
            format_number(
                Some(percentage_value / 100.0),
                &locale_value,
                &options.into(),
            )
        }
    }
}

/// The `MeterRoot` props (`MeterRootProps`, `MeterRoot.tsx:117-141`) — `value` is
/// required and uncontrolled mode does not exist (the deliberate `useControlled`
/// absence, implementation.md "Hooks"). The component macro expands a struct of its
/// own from these fields (the accordion's individual-`#[prop]` convention), so this
/// expansion holds the upstream prop set one field per prop.
///
/// - `value` (`:140`) — the current value; the only required prop.
/// - `min` (`:130-132` — upstream default `0`), `max` (`:125-127` — `100`).
/// - `locale` (`:135-137`) — the `Intl.NumberFormat` locale; `None` is the runtime
///   default locale.
/// - `format` (`:121-123`) — the `Intl.NumberFormatOptions` object; `None` selects the
///   default percent arm.
/// - `get_aria_value_text` (`:128-130`) — receives `(formattedValue, value)` (the
///   formatted *clamped* value first, the **raw** prop second) and overrides only the
///   aria string, never the visible `Meter.Value` text.
/// - `aria_valuetext` (`:94-96`) — the user's `'aria-valuetext'` prop; sits in the
///   `elementProps` rightmost bag upstream, so it overrides the internal
///   `ariaValuetext` (implementation.md untested item 7).
/// - `aria_labelledby` (`:98` — a documented prop) — overrides the label-registered id
///   per the same rightmost-bag rule.
/// - `class` — the `className` passthrough.
#[leptos::component]
pub fn MeterRoot(
    /// `value` (`:140`).
    value: f64,
    /// `min` (`:132` — upstream default `0`).
    #[prop(default = 0.0, optional)]
    min: f64,
    /// `max` (`:127` — upstream default `100`).
    #[prop(default = 100.0, optional)]
    max: f64,
    /// `locale` (`:137`).
    #[prop(default = None, optional)]
    locale: Option<String>,
    /// `format` (`:123`).
    #[prop(default = None, optional)]
    format: Option<serde_json::Value>,
    /// `getAriaValueText` (`:130`).
    #[prop(default = None, optional)]
    get_aria_value_text: Option<Arc<dyn Fn(&str, f64) -> String + Send + Sync>>,
    /// The user's `aria-valuetext` prop (`:96`).
    #[prop(default = None, optional)]
    aria_valuetext: Option<String>,
    /// The user's `aria-labelledby` prop (`:98`).
    #[prop(default = None, optional)]
    aria_labelledby: Option<String>,
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The user's children — the parts subtree. Optional (upstream `children` is a
    /// normal optional React prop; behavior.md "Public API surface" pins
    /// `MeterRoot.test.tsx:216-221`: `Meter.Root` with no children still exposes the
    /// meter role).
    #[prop(default = None, optional)]
    children: Option<leptos::children::Children>,
) -> impl leptos::IntoView {
    // The destructuring (`:21-33`). The only React state in the unit is the label id
    // (`:35`) — the port mirrors it as the unit's only signal.
    let label_id = RwSignal::new(None::<String>);
    let set_label_id: LabelIdSetter = {
        let label_id = label_id.clone();
        Rc::new(move |update: LabelIdUpdate| match update {
            LabelIdUpdate::Set(next) => label_id.set(next),
            LabelIdUpdate::ClearIfCurrent(id) => {
                // The functional-updater guard (`useRegisteredLabelId.ts:15`): only
                // reset when the current value still equals the cleaning-up id.
                label_id.update(|current| {
                    if current.as_deref() == Some(id.as_str()) {
                        *current = None;
                    }
                });
            }
        })
    };

    // The derivation pipeline (`:37-51`).
    let (percentage_value, clamped_value) = derive_meter_values(value, min, max);
    let formatted_value = derive_formatted_value(
        percentage_value,
        clamped_value,
        locale.as_deref(),
        format.as_ref(),
    );

    // `ariaValuetext` (`:48-51`): the callback receives the formatted *clamped* value
    // and the raw prop.
    let internal_aria_valuetext = match &get_aria_value_text {
        Some(get) => get(&formatted_value, value),
        None => formatted_value.clone(),
    };
    // The user's `aria-valuetext` (the rightmost `elementProps` bag) overrides the
    // internal value (implementation.md untested item 7).
    let aria_valuetext = aria_valuetext.unwrap_or(internal_aria_valuetext);

    // The context value (`:70-78`) — the four members, provided before the element so
    // the parts' subtree reads it.
    provide_context(SendWrapper::new(MeterRootContextValue {
        formatted_value: formatted_value.clone(),
        percentage_value,
        set_label_id,
        value,
    }));

    // The ARIA surface (`:53-59`) — all on the single root element; the user's
    // `aria-labelledby` prop overrides the label-registered id.
    let aria_labelledby_attr = move || aria_labelledby.clone().or_else(|| label_id.get_untracked());
    let valuenow = if clamped_value.fract() == 0.0 {
        format!("{}", clamped_value as i64)
    } else {
        format!("{clamped_value}")
    };
    let valymin = format_number_default_locale(min);
    let valymax = format_number_default_locale(max);

    view! {
        <div
            class={class}
            role="meter"
            aria-labelledby={aria_labelledby_attr}
            aria-valuemin={valymin}
            aria-valuemax={valymax}
            aria-valuenow={valuenow}
            aria-valuetext={aria_valuetext}
        >
            {children.map(|children| children())}
            // The hidden NVDA workaround span (`:63-65`, mui/base-ui#4184): NVDA reads
            // the label only when a presentational text node follows it inside the
            // meter. No test observes this node (implementation.md untested item 1).
            <span role="presentation" style={visually_hidden_style()}>
                "x"
            </span>
        </div>
    }
}

/// The `visuallyHidden` style declarations serialized for the `style` attribute — the
/// `style={visuallyHidden}` object spread (`:64`) over the ported constants.
pub(crate) fn visually_hidden_style() -> String {
    VISUALLY_HIDDEN
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `aria-valuemin`/`aria-valuemax` ride React's number-to-string serialization
/// (`:54-55`); the port formats through the same integer/float split
/// `aria-valuenow` uses.
fn format_number_default_locale(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// The `Meter.Label` component (`MeterLabel.tsx` body): reads only `setLabelId`
/// (`:21`), registers its id through the ported [`use_registered_label_id`] (`:23`),
/// and renders the `role="presentation"` span (`:30` — implementation.md untested
/// item 2) carrying `id` + the user's props.
#[leptos::component]
pub fn MeterLabel(
    /// The `id` prop (`useRegisteredLabelId.ts:6` — a user-supplied id wins, otherwise
    /// a `base-ui-`-prefixed id is generated).
    #[prop(default = None, optional)]
    id: Option<String>,
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    children: leptos::children::Children,
) -> impl leptos::IntoView {
    let MeterRootContextValue { set_label_id, .. } = use_meter_root_context();

    // `const id = useRegisteredLabelId(idProp, setLabelId)` (`:23`) — the registration
    // effect and its clear-if-current cleanup are the ported hook's suite-tested
    // contracts. The id signal is read untracked into the attribute closure (the
    // dialog parts' `get_untracked` convention for rg-0.2 LocalStorage signals).
    let id_signal = use_registered_label_id(RwSignal::new(id), set_label_id);
    let registered_id = move || id_signal.get_untracked();

    view! {
        <span class={class} role="presentation" id={registered_id}>
            {children()}
        </span>
    }
}

/// The `Meter.Track` component (`MeterTrack.tsx` body): a pure structural passthrough
/// with **no context access** (`:13-23` — implementation.md untested item 4: the only
/// part that renders outside `Meter.Root` without throwing).
#[leptos::component]
pub fn MeterTrack(
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    children: leptos::children::Children,
) -> impl leptos::IntoView {
    view! {
        <div class={class}>
            {children()}
        </div>
    }
}

/// The `Meter.Indicator` component (`MeterIndicator.tsx` body): reads only
/// `percentageValue` (`:20`) and turns it into the inline-CSS fill —
/// `insetInlineStart: 0`, `height: 'inherit'`, `width: {percentage}%` (`:26-30`).
#[leptos::component]
pub fn MeterIndicator(
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
) -> impl leptos::IntoView {
    let MeterRootContextValue {
        percentage_value, ..
    } = use_meter_root_context();

    let style = format!(
        "inset-inline-start: 0; height: inherit; width: {}%;",
        percentage_value
    );

    view! {
        <div class={class} style={style} />
    }
}

/// The `Meter.Value` component (`MeterValue.tsx` body): reads `value` +
/// `formattedValue` (`:20`), marks the visible text `aria-hidden: true` (`:26`), and
/// renders the formatted value unless the render-function `children` replaces it
/// (`:27` — the `(formattedValue, rawValue)` arguments; behavior.md's "a node"
/// over-claim, implementation.md untested item 5).
///
/// The render function takes the pair as a `(&str, f64)` argument — the Leptos
/// spelling of upstream's `children(formattedValue, value)` (`:27`): a `ChildrenFn`
/// receives no arguments, so the port uses the two-argument closure type directly,
/// wrapped in the `ChildrenFn`-shaped `Option` (the render-function arm; omission
/// renders the formatted value).
#[leptos::component]
pub fn MeterValue(
    /// The `className` passthrough.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// The render-function children arm (`:36-41`) — receives fresh
    /// `(formattedValue, value)` on every derivation; omission renders the formatted
    /// value.
    #[prop(default = None, optional)]
    children: Option<Box<dyn Fn(&str, f64) -> AnyView + Send>>,
) -> impl leptos::IntoView {
    let MeterRootContextValue {
        value,
        formatted_value,
        ..
    } = use_meter_root_context();

    // `children(formattedValue, value)` (`:27`) — the closure receives the pair fresh
    // from the context read on every derivation (the display-only unit re-derives in
    // the body; a value change rides a subtree rebuild, and a rebuilt subtree re-reads
    // the provided context).
    let text = move || match &children {
        Some(render) => render(formatted_value.as_str(), value).into_any(),
        None => formatted_value.clone().into_any(),
    };

    view! {
        <span class={class} aria-hidden="true">
            {text}
        </span>
    }
}

// The `JsValue`/`JsCast` imports serve the JSON/Reflect boundary above; `js_sys` rides
// the internals crate's re-exports through `format_number` on both targets.
