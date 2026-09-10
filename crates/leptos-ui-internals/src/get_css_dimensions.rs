//! Port of `packages/react/src/utils/getCssDimensions.ts` — the element's CSS width/
//! height read with the offset-dimension fallback the positioning consumers rely on
//! (the Tabs indicator and `usePopupAutoResize` per the unit's implementation spec).
//!
//! Upstream reads `getComputedStyle(element).width/height` and falls back to
//! `offsetWidth`/`offsetHeight` when the rounded CSS size disagrees with the integer
//! offset size (`getCssDimensions.ts:10-18`) — subpixel CSS widths (`100.5px`) round
//! differently than the layout's integer report — and SVG elements in test environments
//! report empty strings, which the `parseFloat(...) || 0` guard turns into `0`
//! (`getCssDimensions.ts:6-9`).

use floating_ui_dom::dom::is_html_element;
use floating_ui_utils::Dimensions;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{CssStyleDeclaration, Element, HtmlElement, Window};

use leptos_ui_utils::owner::owner_window;

/// JS `parseFloat(value) || 0` over a computed CSS length: the leading numeric prefix
/// parses (`"100.5px"` → `100.5`), and the falsy results (`0`, `NaN` — `""`/`"auto"`)
/// collapse to `0` through the `|| 0` guard. The exponent notation `parseFloat` also
/// accepts cannot occur in a computed `width`/`height` value, so the scan is digits,
/// the decimal point, and a leading sign only.
fn parse_css_length(value: &str) -> f64 {
    let trimmed = value.trim_start();
    let prefix_end = trimmed
        .find(|c: char| !matches!(c, '0'..='9' | '.' | '+' | '-'))
        .unwrap_or(trimmed.len());
    let parsed = trimmed[..prefix_end].parse::<f64>().unwrap_or(f64::NAN);
    if parsed == 0.0 || parsed.is_nan() {
        0.0
    } else {
        parsed
    }
}

/// Port of `getCssDimensions` (`getCssDimensions.ts:4-24`): the element's computed
/// `width`/`height`, replaced wholesale by the `offsetWidth`/`offsetHeight` integers
/// when `round(css)` disagrees with the offset report.
pub fn get_css_dimensions(element: &Element) -> Dimensions {
    let window: Window = owner_window(Some(element.as_ref() as &web_sys::Node));
    let css: Option<CssStyleDeclaration> = window.get_computed_style(element).ok().flatten();

    let css_length = |name: &str| -> f64 {
        css.as_ref()
            .and_then(|style| style.get_property_value(name).ok())
            .map(|value| parse_css_length(&value))
            .unwrap_or(0.0)
    };

    // In testing environments, the `width` and `height` properties are empty strings
    // for SVG elements, returning NaN — the `|| 0` fallback (`getCssDimensions.ts:6-9`).
    let mut width = css_length("width");
    let mut height = css_length("height");

    // `isHTMLElement(element)` (`getCssDimensions.ts:10`).
    let has_offset = is_html_element(element);
    let html: Option<HtmlElement> = has_offset
        .then(|| element.clone().dyn_into().ok())
        .flatten();
    let offset_width = html.as_ref().map_or(width, |e| f64::from(e.offset_width()));
    let offset_height = html
        .as_ref()
        .map_or(height, |e| f64::from(e.offset_height()));
    let should_fallback = width.round() != offset_width || height.round() != offset_height;

    if should_fallback {
        width = offset_width;
        height = offset_height;
    }

    Dimensions { width, height }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    // Mirrors `getCssDimensions.test.ts` — no upstream test file in this unit
    // (implementation.md, "Submodules with no test anywhere": "getCssDimensions.ts —
    // no direct test in this unit; only indirect numeric assertions via TabsIndicator
    // tests"), so the two fallback branches are pinned against real layout.
    #[wasm_bindgen_test]
    fn keeps_the_computed_size_when_it_matches_the_offset_size() {
        let element = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        element
            .style()
            .set_css_text("position: absolute; width: 100px; height: 50px;");
        document().body().unwrap().append_child(&element).unwrap();

        let dimensions = get_css_dimensions(&element);

        assert_eq!(dimensions.width, 100.0);
        assert_eq!(dimensions.height, 50.0);

        document().body().unwrap().remove_child(&element).unwrap();
    }

    // The `shouldFallback` branch (`getCssDimensions.ts:13-18`): the computed width of
    // a content-box element excludes its padding while `offsetWidth` includes it, so
    // `round(css)` disagrees with the offset report and the offset integers replace
    // both dimensions.
    #[wasm_bindgen_test]
    fn falls_back_to_the_offset_integers_when_the_rounded_css_size_disagrees() {
        let element = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        element
            .style()
            .set_css_text(
                "position: absolute; box-sizing: content-box; width: 100px; height: 50px; padding: 5px;",
            );
        document().body().unwrap().append_child(&element).unwrap();

        let dimensions = get_css_dimensions(&element);

        assert_eq!(dimensions.width, 110.0);
        assert_eq!(dimensions.height, 60.0);

        document().body().unwrap().remove_child(&element).unwrap();
    }

    // The `parseFloat(...) || 0` guard (`getCssDimensions.ts:6-9`) over an element
    // whose computed sizes resolve to nothing: a detached element has no layout box,
    // so the computed `width`/`height` are `"auto"`-shaped and the offsets are 0.
    #[wasm_bindgen_test]
    fn collapses_unresolvable_sizes_to_zero() {
        let element = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        let dimensions = get_css_dimensions(&element);

        assert_eq!(dimensions.width, 0.0);
        assert_eq!(dimensions.height, 0.0);
    }
}
