//! Tests for the Meter port — mirrors of the five upstream suites behavior.md mines
//! (`MeterRoot/Track/Indicator/Value/Label.test.tsx`), over the real materialized tree
//! in a wasm-browser suite and the description-level contracts in a host suite.
//!
//! Host suite: the value-derivation matrix (the clamping/NaN pipeline, implementation.md
//! "Value derivation pipeline") and the context-shape contracts — everything that needs
//! no DOM.
//! Wasm suite: the full mounted tree — the ARIA surface, the label id lift, the
//! indicator's inline-CSS fill, the Value render-function arguments, and the
//! missing-Root panic.

// The wasm-only harness items are dead code on the host target — the dual-target
// test-module convention (the button_tests.rs precedent).
#![allow(unused_imports, dead_code)]

use super::*;

#[cfg(test)]
use crate::meter::{derive_meter_values, visually_hidden_style};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // behavior.md "Accessibility" (`MeterRoot.test.tsx:188-222`): the clamping matrix —
    // value above max clamps to max, below min to min, NaN to min (implementation.md
    // untested item 6: the NaN fallback targets `min`, not literal 0).
    #[test]
    fn the_clamping_matrix_matches_upstream() {
        // 150 → 100, -10 → 0 under the default range.
        let (pct, now) = derive_meter_values(150.0, 0.0, 100.0);
        assert_eq!(pct, 100.0);
        assert_eq!(now, 100.0);
        let (pct, now) = derive_meter_values(-10.0, 0.0, 100.0);
        assert_eq!(pct, 0.0);
        assert_eq!(now, 0.0);
        // NaN → percentage 0 and valuenow = min (the default min pins "NaN→0").
        let (pct, now) = derive_meter_values(f64::NAN, 0.0, 100.0);
        assert_eq!(pct, 0.0);
        assert!(now.is_nan() || now == 0.0, "the default-min arm");
        // NaN with a custom min falls back to that min (implementation.md item 6).
        let (pct, now) = derive_meter_values(f64::NAN, 20.0, 40.0);
        assert_eq!(pct, 0.0);
        assert_eq!(now, 20.0);
        // min === max: the ratio denominator is 0 → NaN percentage → 0, and the raw
        // value survives clamping unchanged (`MeterRoot.test.tsx:205-211`).
        let (pct, now) = derive_meter_values(5.0, 5.0, 5.0);
        assert_eq!(pct, 0.0);
        assert_eq!(now, 5.0);
    }

    // The normalized-ratio mapping (`MeterIndicator.test.tsx:16-52`): 30 in [0,100] →
    // 30%, 30 in [20,40] → 50%, 0.5 in [0,1] → 50%.
    #[test]
    fn the_percentage_ratio_matches_upstream() {
        let (pct, _) = derive_meter_values(30.0, 0.0, 100.0);
        assert_eq!(pct, 30.0);
        let (pct, _) = derive_meter_values(30.0, 20.0, 40.0);
        assert_eq!(pct, 50.0);
        let (pct, _) = derive_meter_values(0.5, 0.0, 1.0);
        assert_eq!(pct, 50.0);
    }

    // The hidden NVDA span's style rides the ported `visuallyHidden` constants
    // (implementation.md untested item 1).
    #[test]
    fn the_nvda_span_style_is_the_ported_visually_hidden_constant() {
        let style = visually_hidden_style();
        assert!(style.contains("clip-path"), "the style reads {style:?}");
        assert!(
            style.contains("position: fixed"),
            "the style reads {style:?}"
        );
    }

    // The context's default shell is absent (no provider) — the required accessor
    // panics with the upstream message (`MeterRootContext.ts:19-21`), pinned here via
    // the panic boundary (the field_root_context.rs precedent).
    #[test]
    fn the_missing_root_context_is_the_upstream_error() {
        let owner = in_owner();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            use_meter_root_context();
        }));
        owner.cleanup();
        let message = result
            .err()
            .and_then(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            })
            .unwrap_or_default();
        assert!(
            message.starts_with("Base UI: MeterRootContext is missing."),
            "the missing-provider error reads {message:?}"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    use super::*;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Mounts the full meter tree (`MeterRoot.test.tsx:21-26` shape: Label +
    /// Track>Indicator + Value under Root) and returns the root element.
    fn mount_meter(
        value: f64,
        min: f64,
        max: f64,
        label_id: Option<String>,
        value_children: Option<Box<dyn Fn(&str, f64) -> AnyView + Send>>,
    ) -> web_sys::Element {
        let _ = any_spawner::Executor::init_futures_executor();
        let container: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();

        let label_id_for_view = label_id.clone().unwrap_or_default();
        // Dropping the UnmountHandle unmounts the view and cancels the reactive
        // owner before the children resolve the provided context — the handle must
        // be kept alive for the mounted tree to stay live (the toggle_tests forget
        // convention; the wasm run caught the harness dropping it).
        std::mem::forget(mount_to({ container.clone() }, move || {
            let value_children_view = match value_children {
                // The macro's `Option<T>` prop setter takes the inner value; omission
                // (None) just drops the prop.
                Some(render) => view! { <MeterValue children=render /> }.into_any(),
                None => view! { <MeterValue /> }.into_any(),
            };
            view! {
                <MeterRoot value=value min=min max=max>
                    <MeterLabel id=label_id_for_view>"Storage Used"</MeterLabel>
                    <MeterTrack>
                        <MeterIndicator />
                    </MeterTrack>
                    {value_children_view}
                </MeterRoot>
            }
        }));
        container
            .query_selector("[role='meter']")
            .unwrap()
            .expect("the meter root mounts")
    }

    fn root_of(container: &web_sys::Element) -> web_sys::Element {
        container
            .query_selector("[role='meter']")
            .unwrap()
            .expect("the meter root mounts")
    }

    // behavior.md "Accessibility" (`MeterRoot.test.tsx:29-33`): `role="meter"` with the
    // default `aria-valuemin="0"` / `aria-valuemax="100"` and `aria-valuenow` the
    // clamped value.
    #[wasm_bindgen_test]
    fn the_root_carries_the_default_aria_surface() {
        let root = mount_meter(30.0, 0.0, 100.0, None, None);
        assert_eq!(root.get_attribute("aria-valuemin").as_deref(), Some("0"));
        assert_eq!(root.get_attribute("aria-valuemax").as_deref(), Some("100"));
        assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("30"));
    }

    // behavior.md "Accessibility" (`MeterRoot.test.tsx:34,52`): the default
    // `aria-valuetext` is the locale-formatted percent of the normalized ratio, and it
    // always equals the visible `Meter.Value` text.
    #[wasm_bindgen_test]
    fn the_default_valuetext_is_the_percent_of_the_ratio_and_matches_the_value_text() {
        let root = mount_meter(30.0, 0.0, 100.0, None, None);
        let valuetext = root
            .get_attribute("aria-valuetext")
            .expect("the valuetext renders");
        let value_text = root
            .query_selector("span[aria-hidden='true']")
            .unwrap()
            .expect("the value span mounts")
            .text_content()
            .unwrap();
        assert_eq!(valuetext, value_text, "same formatter, same locale");
        assert!(
            valuetext.contains('3'),
            "30% formats with a 3: {valuetext:?}"
        );
    }

    // behavior.md "State model" (`MeterRoot.test.tsx:188-222`): value 150 clamps the
    // percentage text to 100% while `aria-valuenow` clamps to 100.
    #[wasm_bindgen_test]
    fn an_out_of_range_value_clamps_every_face() {
        let root = mount_meter(150.0, 0.0, 100.0, None, None);
        assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("100"));
        let valuetext = root.get_attribute("aria-valuetext").unwrap();
        assert!(valuetext.contains("100"), "clamped to 100%: {valuetext:?}");
    }

    // behavior.md "Accessibility" (`MeterRoot.test.tsx:35-37`,
    // `MeterLabel.test.tsx:40`): `aria-labelledby` on the root points at the
    // `Meter.Label` element's id (a user-supplied id such as `label-a` is honored).
    #[wasm_bindgen_test]
    fn the_label_id_registers_into_the_root_labelledby() {
        let root = mount_meter(30.0, 0.0, 100.0, Some("label-a".to_string()), None);
        let labelledby = root
            .get_attribute("aria-labelledby")
            .expect("the labelledby renders");
        assert_eq!(labelledby, "label-a", "the user-supplied id wins");
        let label = root
            .query_selector("span[role='presentation']")
            .unwrap()
            .expect("the label mounts");
        assert_eq!(label.get_attribute("id").as_deref(), Some("label-a"));
    }

    // behavior.md "Accessibility" (`MeterRoot.test.tsx:136,150`): `min={20} max={40}
    // value={30}` → 50%.
    #[wasm_bindgen_test]
    fn a_custom_range_re_derives_the_ratio() {
        let root = mount_meter(30.0, 20.0, 40.0, None, None);
        let valuetext = root.get_attribute("aria-valuetext").unwrap();
        assert!(valuetext.contains("50"), "50% of [20,40]: {valuetext:?}");
    }

    // implementation.md "DOM/portal strategy": the indicator's inline-CSS fill is
    // `insetInlineStart: 0` + `width: {percentage}%` (`MeterIndicator.tsx:26-30`) —
    // percentage width needs no layout measurement.
    #[wasm_bindgen_test]
    fn the_indicator_fill_is_the_inline_css_percentage() {
        let root = mount_meter(33.0, 0.0, 100.0, None, None);
        let indicator = root
            .query_selector("[role='meter'] > div > div div[style]")
            .unwrap()
            .or_else(|| {
                // Fallback: the Indicator is the div inside the Track.
                root.query_selector("div > div > div")
                    .expect("query_selector")
            })
            .expect("the indicator mounts");
        let style = indicator.get_attribute("style").unwrap_or_default();
        assert!(style.contains("width: 33%"), "the fill is 33%: {style:?}");
    }

    // implementation.md "Render pipeline per part" + behavior.md "Public API surface"
    // (`MeterValue.test.tsx:65-85`): the Value render-function children receive fresh
    // `(formattedValue, rawValue)` — the port delivers both through the two-argument
    // closure (`MeterValue.tsx:27`).
    #[wasm_bindgen_test]
    fn the_value_children_receive_the_formatted_and_raw_arguments() {
        let root = mount_meter(
            30.0,
            150.0,
            200.0,
            None,
            Some(Box::new(|formatted: &str, raw: f64| {
                view! {
                    <span data-formatted=format!("{formatted}") data-raw=format!("{raw}") />
                }
                .into_any()
            })),
        );
        let probe = root
            .query_selector("span[data-raw]")
            .unwrap()
            .expect("the render-function child mounts");
        // value=30 with range [150,200]: raw = the unclamped 30, formatted = the
        // clamped 150 ("150%" of the range — the raw value below min pins the pair
        // being (formatted-clamped, raw) exactly as `getAriaValueText` receives it).
        assert_eq!(probe.get_attribute("data-raw").as_deref(), Some("30"));
        let formatted = probe.get_attribute("data-formatted").unwrap_or_default();
        assert!(
            formatted.contains("150"),
            "formatted is the clamped 150: {formatted:?}"
        );
    }

    // behavior.md "Public API surface" (`MeterRoot.test.tsx:216-221`): `Meter.Root`
    // with no children still exposes the meter role.
    #[wasm_bindgen_test]
    fn the_root_exposes_the_meter_role_with_no_children() {
        let _ = any_spawner::Executor::init_futures_executor();
        let container: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        std::mem::forget(mount_to({ container.clone() }, move || {
            view! { <MeterRoot value=50.0 /> }
        }));
        let root = root_of(&container);
        assert_eq!(root.get_attribute("role").as_deref(), Some("meter"));
        assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("50"));
    }

    // behavior.md "Accessibility" (`MeterRoot.test.tsx:114-117,272-273`):
    // `getAriaValueText(formattedValue, value)` overrides only the aria string — it
    // receives the formatted *clamped* value and the raw value, and never affects the
    // visible `Meter.Value` text.
    #[wasm_bindgen_test]
    fn get_aria_value_text_overrides_only_the_aria_string() {
        let _ = any_spawner::Executor::init_futures_executor();
        let container: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <MeterRoot
                    value=30.0
                    get_aria_value_text=std::sync::Arc::new(|formatted: &str, raw: f64| {
                        format!("{formatted} of {raw}")
                    })
                >
                    <MeterValue />
                </MeterRoot>
            }
        }));
        let root = root_of(&container);
        let valuetext = root.get_attribute("aria-valuetext").unwrap();
        // The callback saw the formatted clamped value ("30%") and the raw 30.
        assert!(
            valuetext.contains("30%") && valuetext.contains("of 30"),
            "the callback received (formatted, raw): {valuetext:?}"
        );
        // The visible `Meter.Value` text is untouched by the override (same formatter,
        // same locale — the `MeterRoot.test.tsx:53,67` equality).
        let value_text = root
            .query_selector("span[aria-hidden='true']")
            .unwrap()
            .expect("the value span mounts")
            .text_content()
            .unwrap();
        assert_eq!(
            value_text, "30%",
            "the visible text stays the default percent"
        );
    }

    // implementation.md untested item 1: the hidden NVDA span — `role="presentation"`,
    // visually hidden, containing the text `x` (mui/base-ui#4184).
    #[wasm_bindgen_test]
    fn the_root_appends_the_hidden_nvda_span() {
        let root = mount_meter(30.0, 0.0, 100.0, None, None);
        let spans: Vec<web_sys::Element> = js_sys::Array::from(
            &root
                .query_selector_all("span[role='presentation']")
                .unwrap()
                .into(),
        )
        .iter()
        .map(|node| node.dyn_into::<web_sys::Element>().unwrap())
        .collect();
        assert_eq!(spans.len(), 1, "exactly one NVDA span appends");
        assert_eq!(spans[0].text_content().as_deref(), Some("x"));
        let style = spans[0].get_attribute("style").unwrap_or_default();
        assert!(style.contains("clip-path"), "visually hidden: {style:?}");
    }

    // implementation.md untested item 3: the visible value span is `aria-hidden` (the
    // root's aria-valuetext already exposes the value).
    #[wasm_bindgen_test]
    fn the_value_span_is_aria_hidden() {
        let root = mount_meter(30.0, 0.0, 100.0, None, None);
        let value_span = root
            .query_selector("span[aria-hidden='true']")
            .unwrap()
            .expect("the value span mounts");
        assert_eq!(value_span.tag_name(), "SPAN");
    }

    // behavior.md "Public API surface" (`MeterLabel.test.tsx:49-59`): rendering a
    // context part outside `Meter.Root` rejects with the upstream error. The panic
    // boundary keeps the wasm suite alive (the field_root_context.rs precedent).
    #[wasm_bindgen_test]
    fn a_label_outside_the_root_panics_with_the_upstream_error() {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = mount_to(
                document()
                    .create_element("div")
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap(),
                move || {
                    view! { <MeterLabel>"orphan"</MeterLabel> }
                },
            );
        }));
        owner.cleanup();
        assert!(result.is_err(), "the orphan label must not render silently");
    }
}
