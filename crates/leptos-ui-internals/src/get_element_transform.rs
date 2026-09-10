//! Port of `packages/react/src/utils/getElementTransform.ts` — the 2D translation and
//! scale extracted from the element's computed `transform` matrix (consumed by the
//! Tabs indicator and Drawer per the unit's implementation spec).
//!
//! Upstream parses both the 6-value `matrix(a, b, c, d, tx, ty)` form — translation in
//! the last two entries, scale as `sqrt(a² + b²)` — and the 16-value `matrix3d(...)`
//! form — translation in entries 12/13, scale as the m11 entry (`getElementTransform.ts:19-28`).
//!
//! The module doc carries upstream's caveat verbatim (`:3-6`): the `translate`, `rotate`,
//! and `scale` *longhands* are separate properties and are not reflected in the computed
//! `transform` value — a documented-but-untested limitation (implementation.md,
//! "Anything in source not explained by any test").

use web_sys::wasm_bindgen::JsCast;
use web_sys::{CssStyleDeclaration, HtmlElement};

use leptos_ui_utils::owner::owner_window;

/// The `{ x, y, scale }` return shape (`getElementTransform.ts:32`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementTransform {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

/// JS `parseFloat` per matrix entry: computed `matrix()`/`matrix3d()` values are always
/// plain comma-space-separated numbers, so a plain parse with the `NaN` fallback covers
/// the contract (a non-numeric entry would propagate `NaN` upstream too).
fn parse_entry(value: &str) -> f64 {
    value.trim().parse::<f64>().unwrap_or(f64::NAN)
}

/// The `matrix(?:3d)?\(([^)]+)\)` capture (`getElementTransform.ts:17`) as a string scan:
/// the `matrix` keyword, optional `3d`, then the parenthesized argument list.
fn extract_matrix_args(transform: &str) -> Option<&str> {
    let keyword = transform.find("matrix")? + "matrix".len();
    let rest = &transform[keyword..];
    let rest = rest.strip_prefix("3d").unwrap_or(rest);
    let open = rest.find('(')? + 1;
    let close = rest[open..].find(')')? + open;
    Some(&rest[open..close])
}

/// Port of `getElementTransform` (`getElementTransform.ts:10-33`). The optional
/// `computedStyle` parameter (`:8`) avoids a second lookup when the caller has already
/// resolved the style; `None` reads it through the element's owner window (`:11`).
pub fn get_element_transform(
    element: &HtmlElement,
    computed_style: Option<&CssStyleDeclaration>,
) -> ElementTransform {
    let transform = match computed_style {
        Some(style) => style.get_property_value("transform").unwrap_or_default(),
        None => {
            let window = owner_window(Some(element.as_ref() as &web_sys::Node));
            window
                .get_computed_style(element)
                .ok()
                .flatten()
                .and_then(|style| style.get_property_value("transform").ok())
                .unwrap_or_default()
        }
    };

    let mut result = ElementTransform {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };

    if !transform.is_empty() && transform != "none" {
        if let Some(args) = extract_matrix_args(&transform) {
            let values: Vec<f64> = args.split(", ").map(parse_entry).collect();
            if values.len() == 6 {
                result.x = values[4];
                result.y = values[5];
                result.scale = (values[0] * values[0] + values[1] * values[1]).sqrt();
            } else if values.len() == 16 {
                result.x = values[12];
                result.y = values[13];
                result.scale = values[0];
            }
        }
    }

    result
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    fn element() -> HtmlElement {
        document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap()
    }

    // No `transform` → the `{ x: 0, y: 0, scale: 1 }` defaults
    // (`getElementTransform.ts:12-14`).
    #[wasm_bindgen_test]
    fn returns_the_defaults_without_a_transform() {
        assert_eq!(
            get_element_transform(&element(), None),
            ElementTransform {
                x: 0.0,
                y: 0.0,
                scale: 1.0
            }
        );
    }

    // The 6-value matrix path (`:20-23`): `translate(10px, 20px)` computes to
    // `matrix(1, 0, 0, 1, 10, 20)` — translation from entries 4/5, scale
    // `sqrt(1² + 0²) = 1`.
    #[wasm_bindgen_test]
    fn parses_a_6_value_matrix() {
        let el = element();
        el.style()
            .set_property("transform", "translate(10px, 20px)")
            .unwrap();
        document().body().unwrap().append_child(&el).unwrap();

        assert_eq!(
            get_element_transform(&el, None),
            ElementTransform {
                x: 10.0,
                y: 20.0,
                scale: 1.0
            }
        );

        document().body().unwrap().remove_child(&el).unwrap();
    }

    // The scale extraction `sqrt(a² + b²)` (`:23`): `scale(2)` computes to
    // `matrix(2, 0, 0, 2, 0, 0)`.
    #[wasm_bindgen_test]
    fn extracts_the_scale_from_a_6_value_matrix() {
        let el = element();
        el.style().set_property("transform", "scale(2)").unwrap();
        document().body().unwrap().append_child(&el).unwrap();

        assert_eq!(
            get_element_transform(&el, None),
            ElementTransform {
                x: 0.0,
                y: 0.0,
                scale: 2.0
            }
        );

        document().body().unwrap().remove_child(&el).unwrap();
    }

    // The 16-value matrix3d path (`:24-28`): translation from entries 12/13, scale
    // from the m11 entry.
    #[wasm_bindgen_test]
    fn parses_a_16_value_matrix3d() {
        let el = element();
        el.style()
            .set_property(
                "transform",
                "matrix3d(2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 1, 0, 30, 40, 0, 1)",
            )
            .unwrap();
        document().body().unwrap().append_child(&el).unwrap();

        assert_eq!(
            get_element_transform(&el, None),
            ElementTransform {
                x: 30.0,
                y: 40.0,
                scale: 2.0
            }
        );

        document().body().unwrap().remove_child(&el).unwrap();
    }

    // The documented longhand caveat (`:3-6`): the `translate` longhand is a separate
    // property and is NOT reflected in the computed `transform`.
    #[wasm_bindgen_test]
    fn does_not_read_the_translate_longhand() {
        let el = element();
        el.style().set_property("translate", "10px 20px").unwrap();
        document().body().unwrap().append_child(&el).unwrap();

        assert_eq!(
            get_element_transform(&el, None),
            ElementTransform {
                x: 0.0,
                y: 0.0,
                scale: 1.0
            }
        );

        document().body().unwrap().remove_child(&el).unwrap();
    }

    // The `computedStyle` parameter (`:8`, `:11`): a caller-resolved style is read
    // instead of performing a second lookup.
    #[wasm_bindgen_test]
    fn reads_the_caller_resolved_computed_style() {
        let el = element();
        el.style()
            .set_property("transform", "translate(10px, 20px)")
            .unwrap();
        document().body().unwrap().append_child(&el).unwrap();

        let style = web_sys::window()
            .unwrap()
            .get_computed_style(&el)
            .unwrap()
            .unwrap();

        assert_eq!(
            get_element_transform(&el, Some(&style)),
            ElementTransform {
                x: 10.0,
                y: 20.0,
                scale: 1.0
            }
        );

        document().body().unwrap().remove_child(&el).unwrap();
    }
}
