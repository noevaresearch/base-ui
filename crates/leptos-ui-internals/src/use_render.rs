//! Port of `packages/react/src/use-render/useRender.ts` — a typed, defaults-only facade over
//! the shared render engine (`use_render_element`).
//!
//! Upstream `useRender(params)` is a one-line delegation:
//! ```ts
//! return useRenderElement(params.defaultTagName ?? 'div', params, params);
//! ```
//! (`specs/library/use-render/implementation.md`, "State machine / hooks used"):
//! - Default tag resolution
//! - The double-pass `params, params`
//! - Render selection happens downstream (`evaluateRenderProp` prefers `render` over
//!   the default tag)
//!
//! ## Rust adaptations
//!
//! - **The double-pass signature** (`use_render_element(element, componentProps, params)`) —
//!   Rust's lack of TypeScript's overloaded function signatures means the
//!   `params` argument must be wrapped in a structure that carries its own copy of
//!   `componentProps`'s `render`, `className`, and `style` fields for the engine to read,
//!   which the double-pass exposes through the empty `HTMLProps` wrapper in
//!   `crate::use_render_element::UseRenderElementParams`. The Rust version accepts
//!   explicit `default_tag_name: &str` (instead of defaulting to `'div'`), which matches
//!   upstream's optional parameter (`params.defaultTagName ?? 'div'` — `useRender.ts:19`).
//! - **The `enabled` field** (`useRender.ts:73-78`) is preserved as `UseRenderParams::enabled`,
//!   but unlike upstream where it's typed as `Enabled extends boolean | undefined = undefined`,
//!   Rust defaults it to `true` and only allows `false` at compile time (`Enabled = false`).
//!   The spec notes this is a latent feature (behavior.md, "Edge cases") not exercised
//!   by the upstream suite (`useRender.test.tsx`), so the Rust port implements it as an
//!   optional `bool` parameter with a default of `true`.
//! - **No `className`/`style` exposed** (`useRender.ts:69-70`) — the hook's `props` field is
//!   explicitly typed as `Record<string, unknown>`, so `className` and `style` are read as
//!   `undefined` through the engine (`useRenderElement.tsx:63`, `:71`). The Rust wrapper
//!   conforms to this: its `UseRenderParameters` type omits `class_name` and `style`.
//! - **`enabled: false`** — the conditional return type (`Enabled extends false ? null :
//!   ReactElement`) maps to `Option<RenderedElement>`, where `None` represents
//!   `null` (`useRenderElement.rs:392-395`).
//!
//! ## Dependencies on other Base UI internals
//!
//! This unit re-exports the same types that `useRender` does:
//! - `ComponentRenderFn`, `HTMLProps`, `BaseUIEvent` — from `crate::types`
//! - `UseRenderElementComponentProps`, `UseRenderElementParams`, `RenderedElement`,
//!   `ClassNameSource`, `StyleSource` — from `crate::use_render_element`
//! - `StateAttributesMapping`, `get_state_attributes_props` — from `crate::state_attributes`
//! - `PropsSource`, `merge_class_names`, `merge_styles`, `resolve_source` — from
//!   `crate::merge_props`
//!
//! All of these are already ported in prior Phase A items (`infra: types`,
//! `infra: internals`, `infra: merge-props`, `infra: use-render-element`).

use std::rc::Rc;

use crate::merge_props::{PropsSource, merge_class_names, merge_styles, resolve_source};
use crate::state_attributes::{StateAttributeProps, StateAttributesMapping, get_state_attributes_props};
use crate::types::{BaseUIEvent, ComponentRenderFn, HTMLProps};
use crate::use_render_element::{
    ClassNameSource, RenderElementComponentProps, RenderElementParams,
    RenderElementProps, RenderProp, RenderedElement, StyleSource,
};

/// Port of `useRender.Parameters<State, RenderedElementType, Enabled>`
/// (`packages/react/src/use-render/useRender.ts:42-84`): the parameter bag passed to the hook.
///
/// The Rust version differs from upstream in two ways:
/// - `default_tag_name: &str` is explicit (not optional with default `'div'`), matching
///   the Rust signature pattern over TypeScript's nullish coalescing.
/// - `class_name` and `style` are omitted — the hook exposes no props bag, so these are
///   `undefined` through the engine (`useRenderElement.tsx:63`, `:71`). This matches
///   upstream's `UseRenderParameters` exposure: `className`/`style` are read as `undefined`
///   (`useRenderElement.tsx:63-71`), and `useRender`'s own parameter types omit them
///   (`useRender.ts:42-84` — line `?` is not a valid position in Rust type aliases).
#[derive(Clone, Default)]
pub struct UseRenderParameters<
    State: serde::Serialize + 'static,
    RenderedElement: From<web_sys::Element> + Into<web_sys::Element> + 'static,
> {
    /// The `render` prop — a render function or a render element to override the default tag.
    pub render: Option<RenderProp<RenderedElement, State>>,
    /// The ref(s) to attach to the rendered element.
    pub refs: Vec<crate::use_render_element::InputRef<web_sys::Element>>,
    /// The component's internal state, automatically converted to `data-*` attributes.
    pub state: State,
    /// Custom mapping for converting state properties to `data-*` attributes.
    ///
    /// The mapping receives `&str` (key) and `&serde_json::Value` (value), returning
    /// `Some(None)` to omit the attribute, `Some(Some(props))` to add custom props, or
    /// `None` to use the default handling.
    pub state_attributes_mapping: Option<Rc<dyn Fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>>>>,
    /// Props to be spread on the rendered element (empty bag for this hook).
    pub props: Vec<PropsSource>,
    /// If `false`, the hook returns `None`.
    pub enabled: bool,
}

impl<
    State: serde::Serialize + 'static,
    RenderedElement: From<web_sys::Element> + Into<web_sys::Element> + 'static,
> Default for UseRenderParameters<State, RenderedElement> {
    fn default() -> Self {
        Self {
            render: None,
            refs: Vec::new(),
            state: State::default(),
            state_attributes_mapping: None,
            props: Vec::new(),
            enabled: true,
        }
    }
}

/// Port of `useRender.ReturnValue<Enabled>`
/// (`packages/react/src/use-render/useRender.ts:86-88`): the return type.
///
/// The conditional return type (`Enabled extends false ? null : ReactElement`) maps to
/// `Option<RenderedElement>`, where `None` represents `null`.
pub type UseRenderReturnValue<RenderedElement> = Option<RenderedElement>;

/// Port of `useRender` (`packages/react/src/use-render/useRender.ts:12-20`):
/// renders a Base UI element using the provided parameters.
///
/// The function delegates to `use_render_element` with a double-pass
/// (`params, params`) pattern, where the `params` bag is passed both as
/// `componentProps` and `params` because the Rust signature cannot be overloaded.
///
/// ## Parameters
///
/// - `default_tag_name` — the default HTML element to render if `render` is not provided.
///   Matches upstream's `params.defaultTagName ?? 'div'` behavior.
/// - `params` — the render parameters.
///
/// ## Returns
///
/// Returns `Some(RenderedElement)` when `enabled` is true, or `None` when `enabled` is false.
pub fn use_render<
    State: serde::Serialize + 'static,
    RenderedElement: From<web_sys::Element> + Into<web_sys::Element> + 'static,
>(
    default_tag_name: &str,
    params: &UseRenderParameters<State, RenderedElement>,
) -> UseRenderReturnValue<RenderedElement> {
    if !params.enabled {
        return None;
    }

    let component_props = RenderElementComponentProps::default();

    let state_map = match serde_json::to_value(&params.state) {
        Ok(value) => value,
        Err(_) => return None,
    };

    let mut state_map = match state_map {
        serde_json::Value::Object(map) => map,
        _ => return None,
    };

    let element_params = RenderElementParams {
        state: &state_map,
        refs: params.refs.clone(),
        props: params.props.clone(),
        state_attributes_mapping: params.state_attributes_mapping.as_ref().map(|mapping| mapping.as_ref()),
    };

    use_render_element::use_render_element(default_tag_name, component_props, element_params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComponentRenderFn;
    use crate::use_render_element::RenderProp;

    #[test]
    fn use_render_defaults_to_div() {
        let params = UseRenderParameters::<serde_json::Value, web_sys::Element>::default();
        let element = use_render("div", &params);
        assert!(element.is_some());
        assert_eq!(element.unwrap().tag, "div");
    }

    #[test]
    fn use_render_accepts_render_function() {
        let params = UseRenderParameters::<serde_json::Value, web_sys::Element> {
            render: Some(RenderProp::Fn {
                component_props: HTMLProps::default(),
                state: serde_json::json!({}),
            }),
            ..Default::default()
        };
        let element = use_render("div", &params);
        assert!(element.is_some());
    }

    #[test]
    fn use_render_enabled_false_returns_none() {
        let params = UseRenderParameters::<serde_json::Value, web_sys::Element> {
            enabled: false,
            ..Default::default()
        };
        let element = use_render("div", &params);
        assert!(element.is_none());
    }

    #[test]
    fn use_render_with_state_converts_to_data_attributes() {
        let state = serde_json::json!({
            "active": true,
            "disabled": false,
            "count": 0,
        });
        let params = UseRenderParameters {
            state: state.clone(),
            enabled: true,
            ..Default::default()
        };
        let element = use_render("div", &params);
        assert!(element.is_some());
        let rendered = element.unwrap();
        let props = &rendered.props;
        assert!(props.handlers.attributes.contains_key("data-active"));
        assert_eq!(props.handlers.attributes.get("data-active").unwrap()(&serde_json::Value::Null), Some("".to_string()));
        assert!(!props.handlers.attributes.contains_key("data-disabled"));
        assert!(!props.handlers.attributes.contains_key("data-count"));
    }

    #[test]
    fn use_render_with_state_attributes_mapping() {
        let state = serde_json::json!({
            "isActive": true,
        });
        let mapping = Rc::new(|key: &str, value: &serde_json::Value| -> Option<Option<StateAttributeProps>> {
            if value.is_boolean() && value.as_bool().unwrap() {
                Some((
                    "data-is-active".to_string(),
                    "".to_string(),
                ))
            } else {
                None
            }
        });
        let params = UseRenderParameters {
            state,
            state_attributes_mapping: Some(mapping),
            enabled: true,
            ..Default::default()
        };
        let element = use_render("div", &params);
        assert!(element.is_some());
        let rendered = element.unwrap();
        let props = &rendered.props;
        assert!(props.handlers.attributes.contains_key("data-is-active"));
    }
}