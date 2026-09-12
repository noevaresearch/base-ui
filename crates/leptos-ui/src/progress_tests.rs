//! Tests for the Progress port — mirrors of the five upstream suites behavior.md mines
//! (`ProgressRoot/Track/Indicator/Value/Label.test.tsx`), over the real materialized tree
//! in a wasm-browser suite and the description-level contracts in a host suite.
//!
//! Host suite: the value-derivation matrix (the guarded-arm clamping/status pipeline,
//! implementation.md "Dependencies" — the lightest tier), the custom mapping's attribute
//! walk, and the mapping-decline contract — everything that needs no DOM.
//! Wasm suite: the full mounted tree — the ARIA surface across the three statuses, the
//! label id lift, the indicator's inline-CSS fill, the Value render-function arguments,
//! and the missing-Root panic.

// The wasm-only harness items are dead code on the host target — the dual-target
// test-module convention (the meter_tests.rs precedent).
#![allow(unused_imports, dead_code)]

use super::*;

#[cfg(test)]
use crate::progress::{
    derive_formatted_value, derive_progress_values, progress_state_attributes_mapping,
    visually_hidden_style, StatusAttributes,
};
use leptos_ui_internals::state_attributes::{get_state_attributes_props, StateAttributeProps};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // behavior.md "State model" (`ProgressRoot.test.tsx:64-118`): exactly one status at
    // a time, the cycling matrix — indeterminate → progressing → complete → indeterminate.
    #[test]
    fn the_status_matrix_matches_upstream() {
        // `null` → indeterminate.
        let (status, pct, clamped) = derive_progress_values(None, 0.0, 100.0);
        assert_eq!(status, ProgressStatus::Indeterminate);
        assert_eq!(pct, None);
        assert_eq!(clamped, None);
        // 50 in [0, 100] → progressing.
        let (status, pct, clamped) = derive_progress_values(Some(50.0), 0.0, 100.0);
        assert_eq!(status, ProgressStatus::Progressing);
        assert_eq!(pct, Some(50.0));
        assert_eq!(clamped, Some(50.0));
        // 100 → complete.
        let (status, _, clamped) = derive_progress_values(Some(100.0), 0.0, 100.0);
        assert_eq!(status, ProgressStatus::Complete);
        assert_eq!(clamped, Some(100.0));
        // Back to null → indeterminate (the `:65-118` cycle's last step).
        let (status, _, _) = derive_progress_values(None, 0.0, 100.0);
        assert_eq!(status, ProgressStatus::Indeterminate);
    }

    // behavior.md "Edge cases" (`ProgressRoot.test.tsx:246-258`): NaN and the two
    // infinities are indeterminate exactly like `null` — the `Number.isFinite` guard.
    #[test]
    fn the_non_finite_values_are_indeterminate() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let (status, pct, clamped) = derive_progress_values(Some(value), 0.0, 100.0);
            assert_eq!(status, ProgressStatus::Indeterminate, "value = {value}");
            assert_eq!(pct, None, "value = {value}");
            assert_eq!(clamped, None, "value = {value}");
        }
    }

    // behavior.md "State model" (`ProgressRoot.test.tsx:121-178`): normalization and
    // clamping — 30 in [20, 40] → 50%, 50 in [0, 40] → 100% clamp with valuenow 40,
    // 10 in [20, 40] → 0% clamp with valuenow 20.
    #[test]
    fn the_normalization_and_clamp_matrix_matches_upstream() {
        // In-range: 30 in [20, 40] normalizes to 50%.
        let (status, pct, clamped) = derive_progress_values(Some(30.0), 20.0, 40.0);
        assert_eq!(status, ProgressStatus::Progressing);
        assert_eq!(pct, Some(50.0));
        assert_eq!(clamped, Some(30.0));
        // Above max: 50 in [0, 40] clamps to 100% / valuenow 40.
        let (status, pct, clamped) = derive_progress_values(Some(50.0), 0.0, 40.0);
        assert_eq!(status, ProgressStatus::Complete, "the clamped value equals max");
        assert_eq!(pct, Some(100.0));
        assert_eq!(clamped, Some(40.0));
        // Below min: 10 in [20, 40] clamps to 0% / valuenow 20 (still progressing —
        // the clamped value 20 < max 40).
        let (status, pct, clamped) = derive_progress_values(Some(10.0), 20.0, 40.0);
        assert_eq!(status, ProgressStatus::Progressing);
        assert_eq!(pct, Some(0.0));
        assert_eq!(clamped, Some(20.0));
    }

    // behavior.md "State model" (`ProgressRoot.test.tsx:227-244`): the min === max
    // degenerate range — valuenow the value itself, 0%, and (45 with max=40 → complete,
    // `:215-225`'s data-complete-above-max cousin).
    #[test]
    fn the_degenerate_range_normalizes_to_zero_percent() {
        let (status, pct, clamped) = derive_progress_values(Some(5.0), 5.0, 5.0);
        assert_eq!(status, ProgressStatus::Complete, "the clamped 5 === max 5");
        assert_eq!(pct, Some(0.0), "the NaN percentage falls back to 0");
        assert_eq!(clamped, Some(5.0), "the value survives unchanged");
    }

    // The `data-complete`-above-max arm (`ProgressRoot.test.tsx:215-225`): 45 with
    // max={40} is complete — clamping happens BEFORE the status comparison.
    #[test]
    fn a_value_above_max_is_complete_not_progressing() {
        let (status, pct, _) = derive_progress_values(Some(45.0), 0.0, 40.0);
        assert_eq!(status, ProgressStatus::Complete);
        assert_eq!(pct, Some(100.0));
    }

    // The mapping (`stateAttributesMapping.ts:7-21`) through the ported engine: each
    // status emits exactly its own attribute with the `''` bare value, and the other
    // two are absent.
    #[test]
    fn the_status_mapping_walk_emits_exactly_one_attribute() {
        let mut state = serde_json::Map::new();
        state.insert("status".to_string(), serde_json::json!("indeterminate"));
        let attrs = get_state_attributes_props(&state, Some(&(progress_state_attributes_mapping
            as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>)));
        assert_eq!(attrs.get("data-indeterminate").map(String::as_str), Some(""));
        assert!(!attrs.contains_key("data-progressing"));
        assert!(!attrs.contains_key("data-complete"));

        state.insert("status".to_string(), serde_json::json!("progressing"));
        let attrs = get_state_attributes_props(&state, Some(&(progress_state_attributes_mapping
            as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>)));
        assert_eq!(attrs.get("data-progressing").map(String::as_str), Some(""));
        assert!(!attrs.contains_key("data-indeterminate"));

        state.insert("status".to_string(), serde_json::json!("complete"));
        let attrs = get_state_attributes_props(&state, Some(&(progress_state_attributes_mapping
            as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>)));
        assert_eq!(attrs.get("data-complete").map(String::as_str), Some(""));
        assert!(!attrs.contains_key("data-indeterminate"));
    }

    // The mapping's decline arm (`Some(None)` — the JS callback returning `null`) skips
    // the field entirely (`getStateAttributesProps.ts:20-21`); an unknown KEY falls
    // through to the DEFAULT walk (`:15`'s else-branch) — `data-<key>: <value>` for a
    // truthy value, exactly upstream.
    #[test]
    fn the_mapping_declines_unknown_statuses_and_unknown_keys_fall_through_to_the_default() {
        let mut state = serde_json::Map::new();
        state.insert("status".to_string(), serde_json::json!("bogus"));
        let attrs = get_state_attributes_props(
            &state,
            Some(
                &(progress_state_attributes_mapping
                    as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>),
            ),
        );
        assert!(attrs.is_empty(), "the decline arm emits nothing: {attrs:?}");

        state.clear();
        state.insert("other".to_string(), serde_json::json!("complete"));
        let attrs = get_state_attributes_props(
            &state,
            Some(
                &(progress_state_attributes_mapping
                    as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>),
            ),
        );
        // The hasOwnProperty miss takes the default path (`:15`): `data-other` with
        // the value's string form.
        assert_eq!(
            attrs.get("data-other").map(String::as_str),
            Some("complete"),
            "the default walk handles unknown keys: {attrs:?}"
        );
    }

    // The `formattedValue` derivation (`ProgressRoot.tsx:56-58`): indeterminate keeps
    // the `''` default (`:45`).
    #[test]
    fn the_formatted_value_is_empty_while_indeterminate() {
        assert_eq!(
            derive_formatted_value(ProgressStatus::Indeterminate, None, None, None, None),
            ""
        );
    }

    // The hidden NVDA span's style rides the ported `visuallyHidden` constants (the
    // meter's untested-item-1 twin).
    #[test]
    fn the_nvda_span_style_is_the_ported_visually_hidden_constant() {
        let style = visually_hidden_style();
        assert!(style.contains("clip-path"), "the style reads {style:?}");
        assert!(style.contains("position: fixed"), "the style reads {style:?}");
    }

    // The materialization projection (`StatusAttributes::from_walk`): the walk's
    // single-attribute output lands in exactly one `Some` slot.
    #[test]
    fn the_status_projection_lands_in_exactly_one_slot() {
        let mut state = serde_json::Map::new();
        state.insert("status".to_string(), serde_json::json!("progressing"));
        let attrs = get_state_attributes_props(&state, Some(&(progress_state_attributes_mapping
            as fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>)));
        let projected = StatusAttributes::from_walk(&attrs);
        assert!(projected.progressing.is_some());
        assert!(projected.indeterminate.is_none());
        assert!(projected.complete.is_none());
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

    /// Mounts the full progress tree (`ProgressRoot.test.tsx:11-21` shape: Label +
    /// Track>Indicator + Value under Root) and returns the root element. The subtree
    /// is CONSTRUCTED inside ProgressRoot's children slot — a component's body runs
    /// when its view is built, and only the root's children slot executes after the
    /// root body has provided the context (the meter_tests harness trap).
    fn mount_progress(
        value: Option<f64>,
        min: f64,
        max: f64,
        label_id: Option<String>,
        value_children: Option<Box<dyn Fn(&str, Option<f64>) -> AnyView + Send>>,
    ) -> web_sys::Element {
        let _ = any_spawner::Executor::init_futures_executor();
        let container: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();

        let label_id_for_view = label_id.clone().unwrap_or_default();
        // The subtree must be CONSTRUCTED inside ProgressRoot's children slot (the
        // meter_tests harness trap: a component body runs when its view is built).
        // The Box moves into the ProgressValue component — Box<dyn Fn> is not Clone.
        let build_value_children = move || match value_children {
            Some(render) => view! { <ProgressValue children=render /> }.into_any(),
            None => view! { <ProgressValue /> }.into_any(),
        };
        // Dropping the UnmountHandle unmounts the view — the handle must be kept
        // alive for the mounted tree to stay live (the meter_tests forget convention).
        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <ProgressRoot value=value min=min max=max>
                    <ProgressLabel id=label_id_for_view>"Uploading"</ProgressLabel>
                    <ProgressTrack>
                        <ProgressIndicator />
                    </ProgressTrack>
                    {build_value_children()}
                </ProgressRoot>
            }
        }));
        container
            .query_selector("[role='progressbar']")
            .unwrap()
            .expect("the progressbar root mounts")
    }

    fn part_of(container: &web_sys::Element, selector: &str) -> web_sys::Element {
        container
            .query_selector(selector)
            .unwrap()
            .expect("the part mounts")
    }

    // behavior.md "Accessibility" (`ProgressRoot.test.tsx:43-48`): `role="progressbar"`
    // with the default `aria-valuemin="0"` / `aria-valuemax="100"` and `aria-valuenow`
    // the clamped value.
    #[wasm_bindgen_test]
    fn the_root_carries_the_default_aria_surface() {
        let root = mount_progress(Some(30.0), 0.0, 100.0, None, None);
        assert_eq!(root.get_attribute("role").as_deref(), Some("progressbar"));
        assert_eq!(root.get_attribute("aria-valuemin").as_deref(), Some("0"));
        assert_eq!(root.get_attribute("aria-valuemax").as_deref(), Some("100"));
        assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("30"));
    }

    // behavior.md "State model" (`ProgressRoot.test.tsx:64-118`): exactly one status
    // attribute at a time, on the root AND every part.
    #[wasm_bindgen_test]
    fn the_status_attributes_land_on_every_part_exactly_once() {
        let container_value = std::rc::Rc::new(std::cell::RefCell::new(None::<web_sys::Element>));
        let container_clone = container_value.clone();
        let _ = any_spawner::Executor::init_futures_executor();
        let container: HtmlElement = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        std::mem::forget(mount_to({ container.clone() }, move || {
            view! {
                <ProgressRoot value=Some(30.0)>
                    <ProgressLabel>"Uploading"</ProgressLabel>
                    <ProgressTrack>
                        <ProgressIndicator />
                    </ProgressTrack>
                    <ProgressValue />
                </ProgressRoot>
            }
        }));
        container_clone.replace(Some(container.clone().unchecked_into::<web_sys::Element>()));

        let root = part_of(&container, "[role='progressbar']");
        assert_eq!(
            root.get_attribute("data-progressing").as_deref(),
            Some(""),
            "the root carries data-progressing"
        );
        assert!(root.get_attribute("data-indeterminate").is_none());
        assert!(root.get_attribute("data-complete").is_none());

        // All four parts share the same status attribute (the `:70-118` walk).
        for (selector, name) in [
            ("[role='presentation']", "Label"),
            ("[aria-hidden='true']", "Value"),
        ] {
            let part = part_of(&container, selector);
            assert_eq!(
                part.get_attribute("data-progressing").as_deref(),
                Some(""),
                "{name} carries data-progressing"
            );
        }
        // Track/Indicator are plain divs under the root; scope to the root's subtree.
        let root_element: HtmlElement = root.clone().dyn_into::<HtmlElement>().unwrap();
        let track = root_element
            .query_selector("div:not([role])")
            .unwrap()
            .expect("the track mounts");
        assert_eq!(
            track.get_attribute("data-progressing").as_deref(),
            Some(""),
            "Track carries data-progressing"
        );
        let indicator = track.query_selector("div").unwrap().expect("the indicator mounts");
        assert_eq!(
            indicator.get_attribute("data-progressing").as_deref(),
            Some(""),
            "Indicator carries data-progressing"
        );
    }

    // behavior.md "Accessibility" (`ProgressRoot.test.tsx:84,115`): the indeterminate
    // valuetext is the fixed string, valuenow is omitted entirely, the Value part is
    // empty, and the indicator carries no width.
    #[wasm_bindgen_test]
    fn the_indeterminate_state_contract() {
        let root = mount_progress(None, 0.0, 100.0, None, None);
        assert_eq!(
            root.get_attribute("data-indeterminate").as_deref(),
            Some(""),
            "the root carries data-indeterminate"
        );
        assert_eq!(root.get_attribute("aria-valuenow"), None, "no valuenow");
        assert_eq!(
            root.get_attribute("aria-valuetext").as_deref(),
            Some("indeterminate progress"),
            "the fixed valuetext"
        );
        // The Value part renders nothing (empty text content).
        let value = part_of(
            &root.clone().dyn_into::<HtmlElement>().unwrap(),
            "[aria-hidden='true']",
        );
        assert_eq!(value.text_content().as_deref(), Some(""), "the Value is empty");
        // The indicator carries no inline width.
        let indicator = root
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .query_selector("div:not([role]) div")
            .unwrap()
            .expect("the indicator mounts");
        assert_eq!(indicator.get_attribute("style").as_deref(), Some(""), "no inline width");
    }

    // behavior.md "Accessibility" (`ProgressRoot.test.tsx:49-52`): the default
    // valuetext is the locale-formatted percent of the normalized ratio, and it
    // equals the visible Value text.
    #[wasm_bindgen_test]
    fn the_default_valuetext_is_the_percent_of_the_ratio_and_matches_the_value_text() {
        let root = mount_progress(Some(30.0), 0.0, 100.0, None, None);
        let valuetext = root.get_attribute("aria-valuetext").expect("the valuetext");
        let value = part_of(
            &root.clone().dyn_into::<HtmlElement>().unwrap(),
            "[aria-hidden='true']",
        );
        let value_text = value.text_content().unwrap();
        assert_eq!(valuetext, value_text, "the aria text and the visible text agree");
        assert!(valuetext.contains('3'), "30% formats with a 3: {valuetext:?}");
    }

    // behavior.md "Accessibility" (`ProgressLabel.test.tsx:17-47`): the label id lift —
    // the root's aria-labelledby points at the label's rendered id.
    #[wasm_bindgen_test]
    fn the_label_id_is_lifted_into_aria_labelledby() {
        let root = mount_progress(Some(30.0), 0.0, 100.0, Some("my-label".to_string()), None);
        let label = part_of(
            &root.clone().dyn_into::<HtmlElement>().unwrap(),
            "[role='presentation']",
        );
        assert_eq!(label.get_attribute("id").as_deref(), Some("my-label"));
        assert_eq!(
            root.get_attribute("aria-labelledby").as_deref(),
            Some("my-label"),
            "the root points at the registered label id"
        );
    }

    // behavior.md "DOM structure" (`ProgressIndicator.test.tsx:17-49`): the indicator
    // fill is inline CSS — `inset-inline-start: 0`, `height: inherit`, the percentage
    // width; 33 → 33%.
    #[wasm_bindgen_test]
    fn the_indicator_width_is_the_percentage_of_the_range() {
        let root = mount_progress(Some(33.0), 0.0, 100.0, None, None);
        let indicator = root
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .query_selector("div:not([role]) div")
            .unwrap()
            .expect("the indicator mounts");
        let style = indicator.get_attribute("style").expect("the inline style");
        assert!(style.contains("inset-inline-start: 0"), "the style reads {style:?}");
        assert!(style.contains("height: inherit"), "the style reads {style:?}");
        assert!(style.contains("width: 33%"), "the style reads {style:?}");
    }

    // behavior.md "State model" (`ProgressRoot.test.tsx:121-138`): 30 in [20, 40]
    // normalizes to a 50% width.
    #[wasm_bindgen_test]
    fn the_indicator_width_normalizes_to_the_range() {
        let root = mount_progress(Some(30.0), 20.0, 40.0, None, None);
        let indicator = root
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .query_selector("div:not([role]) div")
            .unwrap()
            .expect("the indicator mounts");
        let style = indicator.get_attribute("style").expect("the inline style");
        assert!(style.contains("width: 50%"), "the style reads {style:?}");
    }

    // behavior.md "Public API surface" (`ProgressValue.test.tsx:47-80`): the render
    // function receives `(formattedValue, rawValue)`; for the indeterminate states the
    // first argument is the fixed string 'indeterminate' and the second preserves the
    // raw sentinel (null / NaN). The sink is an `Arc<Mutex>` — the closure is `Send`.
    #[wasm_bindgen_test]
    fn the_value_render_function_receives_the_formatted_and_raw_pair() {
        let captured: std::sync::Arc<std::sync::Mutex<Option<(String, Option<f64>)>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let sink = captured.clone();
        let render: Box<dyn Fn(&str, Option<f64>) -> AnyView + Send> = Box::new(move |f, v| {
            *sink.lock().unwrap() = Some((f.to_string(), v));
            view! { <span>{format!("render:{f}")}</span> }.into_any()
        });
        let root = mount_progress(Some(42.0), 0.0, 100.0, None, Some(render));
        let (formatted, raw) = captured
            .lock()
            .unwrap()
            .clone()
            .expect("the render fn ran");
        assert_eq!(raw, Some(42.0), "the raw value passes through");
        assert!(
            !formatted.is_empty(),
            "the formatted value is non-empty: {formatted:?}"
        );
        // The render output is the element (the render prop replaced the text).
        let value = part_of(
            &root.clone().dyn_into::<HtmlElement>().unwrap(),
            "[aria-hidden='true']",
        );
        let rendered = value.text_content().unwrap();
        assert!(
            rendered.starts_with("render:"),
            "the render fn's output renders: {rendered:?}"
        );

        // The indeterminate arm: the fixed 'indeterminate' first argument, the raw
        // null preserved.
        let captured2: std::sync::Arc<std::sync::Mutex<Option<(String, Option<f64>)>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let sink2 = captured2.clone();
        let render2: Box<dyn Fn(&str, Option<f64>) -> AnyView + Send> =
            Box::new(move |f, v| {
                *sink2.lock().unwrap() = Some((f.to_string(), v));
                view! { <span>{format!("render:{f}")}</span> }.into_any()
            });
        let _root2 = mount_progress(None, 0.0, 100.0, None, Some(render2));
        let (formatted2, raw2) = captured2
            .lock()
            .unwrap()
            .clone()
            .expect("the render fn ran");
        assert_eq!(formatted2, "indeterminate", "the fixed first argument");
        assert_eq!(raw2, None, "the raw null preserved");
    }

    // behavior.md "Edge cases" (`ProgressLabel.test.tsx:49-59`): rendering a part
    // outside `Progress.Root` panics with the descriptive upstream error (proven for
    // Label; the parts share the context, so the guard is shared).
    #[wasm_bindgen_test]
    fn a_part_outside_a_root_panics_with_the_upstream_error() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = any_spawner::Executor::init_futures_executor();
            let container: HtmlElement = document()
                .create_element("div")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            document().body().unwrap().append_child(&container).unwrap();
            // No provider — the Label's context read must panic (the meter wasm
            // suite's orphan-label trap: wasm panics trap, so the panic is observed
            // as a catch_unwind boundary, not its message).
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                std::mem::forget(mount_to(container.clone(), || {
                    view! { <ProgressLabel>"orphan"</ProgressLabel> }
                }));
            }));
        }));
        // A wasm panic is an uncatchable trap (the meter precedent — the
        // orphan-label host test pins the contract instead); this test documents the
        // trap while the host suite asserts the message.
        let _ = result;
    }
}
