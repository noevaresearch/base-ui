//! Tests for the Fieldset port — mirrors of the upstream suites behavior.md mines
//! (`FieldsetRoot.test.tsx`, `FieldsetLegend.test.tsx`), over the real mounted tree
//! in a wasm-browser suite and the description-level contracts in a host suite.
//!
//! Host suite: the two element descriptions (tag, bag order, the state walk's
//! `data-disabled`), the `...elementProps` override rule (implementation.md untested
//! item 2), and the missing-root throw.
//! Wasm suite: the mounted association lifecycle — the legend's registration landing
//! on the root (`FieldsetLegend.test.tsx:17-28`), the custom-id passthrough
//! (`:30-38`), the withdrawal on unmount (`:59-67`), the no-legend case (`:81-85`),
//! and the effective-disabled rule across nested roots
//! (`FieldsetRoot.test.tsx:32-84`).
//!
//! The engine's own conformance surface (prop forwarding, ref forwarding, render
//! props, className merging — implementation.md "Dependencies" makes them the shared
//! engine's) is pinned by `use_render_element`'s own suite; this facade suite pins
//! that the parts wire it with the documented bag and the documented state record.

// The wasm-only harness items are dead code on the host target — the dual-target
// test-module convention (the button/separator/field precedent).
#![allow(unused_imports, dead_code)]

use super::*;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::fieldset::legend::{
        FieldsetLegendElementProps, FieldsetLegendViewProps, fieldset_legend_element,
        fieldset_legend_view,
    };
    use crate::fieldset::root::{
        FieldsetRootElementProps, FieldsetRootViewProps, fieldset_element,
    };
    use leptos::prelude::Signal;
    use leptos_ui_internals::use_render_element::{
        ClassNameSource, RenderedElement, StyleSource, UseRenderElementComponentProps,
    };

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    fn static_legend_id(id: Option<&'static str>) -> Signal<Option<String>> {
        Signal::derive(move || id.map(str::to_string))
    }

    fn attribute_value(
        rendered: &RenderedElement,
        name: &str,
    ) -> Option<Option<String>> {
        rendered
            .props
            .handlers
            .attributes
            .iter()
            .find(|(attribute, _)| attribute == name)
            .map(|(_, value)| value())
    }

    // FieldsetRoot.tsx:34 — the default tag is the native `<fieldset>` (behavior.md
    // "DOM structure"; the implicit `group` role comes free from it).
    #[test]
    fn the_root_description_is_a_native_fieldset() {
        let _owner = in_owner();
        let rendered = fieldset_element(FieldsetRootElementProps {
            legend_id: static_legend_id(None),
            ..FieldsetRootElementProps::default()
        })
        .expect("the root always renders");
        assert_eq!(rendered.tag, "fieldset");
    }

    // FieldsetRoot.tsx:30-32 + :37-43, with getStateAttributesProps.ts:24-28 — the
    // `{ disabled }` state record walks to `data-disabled=""` on the element itself
    // (implementation.md untested item 1: no upstream test asserts this member), and
    // the intrinsic bag carries the native `disabled` attribute plus the derived
    // `aria-labelledby`.
    #[test]
    fn the_disabled_member_drives_the_attribute_and_the_state_hook() {
        let _owner = in_owner();

        let enabled = fieldset_element(FieldsetRootElementProps {
            disabled: false,
            legend_id: static_legend_id(None),
            ..FieldsetRootElementProps::default()
        })
        .expect("renders");
        assert_eq!(
            attribute_value(&enabled, "disabled"),
            Some(None),
            "an enabled fieldset omits the native disabled attribute (React's false projection)"
        );
        assert_eq!(
            attribute_value(&enabled, "data-disabled"),
            None,
            "the default walk emits NOTHING for a false member — the name is absent from the bag \
             entirely (getStateAttributesProps.ts:24-28: a false state value produces no attribute)"
        );

        let disabled = fieldset_element(FieldsetRootElementProps {
            disabled: true,
            legend_id: static_legend_id(Some("legend-1")),
            ..FieldsetRootElementProps::default()
        })
        .expect("renders");
        assert_eq!(
            attribute_value(&disabled, "disabled"),
            Some(Some(String::new())),
            "disabled=true is the empty-string attribute"
        );
        assert_eq!(
            attribute_value(&disabled, "data-disabled"),
            Some(Some(String::new())),
            "the state record's true member becomes data-disabled=\"\""
        );
        assert_eq!(
            attribute_value(&disabled, "aria-labelledby"),
            Some(Some("legend-1".to_string())),
            "the association is the registered legend id"
        );
    }

    // FieldsetLegend.tsx:28 — a plain `<div>` (implementation.md "DOM/portal
    // strategy": a native `<legend>` is hostile to styling, so the association is
    // programmatic), carrying the resolved id and the context's disabled member.
    #[test]
    fn the_legend_description_is_a_div_with_the_resolved_id() {
        let _owner = in_owner();
        let rendered = fieldset_legend_element(FieldsetLegendElementProps {
            id: "base-ui-fieldset-legend".to_string(),
            disabled: true,
            ..FieldsetLegendElementProps::default()
        })
        .expect("the legend always renders");
        assert_eq!(rendered.tag, "div");
        assert_eq!(
            attribute_value(&rendered, "id"),
            Some(Some("base-ui-fieldset-legend".to_string()))
        );
        assert_eq!(
            attribute_value(&rendered, "data-disabled"),
            Some(Some(String::new()))
        );
    }

    // implementation.md untested item 2 / FieldsetRoot.tsx:37-43 + mergeProps.ts:14-15:
    // `elementProps` is the LAST bag, so a user-supplied `aria-labelledby` or `disabled`
    // silently overrides the internally managed value (rightmost-wins for plain values).
    #[test]
    fn the_element_props_rest_overrides_the_managed_members() {
        let _owner = in_owner();
        let rendered = fieldset_element(FieldsetRootElementProps {
            disabled: true,
            legend_id: static_legend_id(Some("managed")),
            element_attributes: vec![
                ("aria-labelledby".to_string(), "user-supplied".to_string()),
                ("data-foobar".to_string(), "1".to_string()),
            ],
            ..FieldsetRootElementProps::default()
        })
        .expect("renders");

        assert_eq!(
            attribute_value(&rendered, "aria-labelledby"),
            Some(Some("user-supplied".to_string())),
            "the rest bag wins over the managed association"
        );
        assert_eq!(
            attribute_value(&rendered, "data-foobar"),
            Some(Some("1".to_string())),
            "arbitrary DOM props are forwarded (conformance propForwarding.tsx:23-36)"
        );
    }

    // FieldsetRoot.tsx:18-20 — `className`/`style` reach the engine's merge, not a
    // view-side re-derivation.
    #[test]
    fn the_class_and_style_ride_the_engine_merge() {
        let _owner = in_owner();
        let rendered = fieldset_element(FieldsetRootElementProps {
            render_class_style: UseRenderElementComponentProps {
                class_name: Some(ClassNameSource::Static("billing".to_string())),
                render: None,
                style: Some(StyleSource::Static(vec![(
                    "color".to_string(),
                    "red".to_string(),
                )])),
            },
            ..FieldsetRootElementProps::default()
        })
        .expect("renders");
        assert_eq!(rendered.props.class.as_deref(), Some("billing"));
        assert_eq!(
            rendered.props.style,
            vec![("color".to_string(), "red".to_string())]
        );
    }

    // FieldsetRootContext.ts:14-21 / FieldsetLegend.test.tsx:69-79 — the required
    // overload throws the upstream message when there is no root ancestor (the
    // avatar `use_avatar_root_context` panic-pin precedent).
    #[test]
    fn the_legend_requires_a_root_context() {
        let owner = in_owner();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = fieldset_legend_view(FieldsetLegendViewProps::default());
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
            message.contains(
                "Base UI: FieldsetRootContext is missing. Fieldset parts must be placed within \
                 <Fieldset.Root>."
            ),
            "the port reproduces the upstream throw verbatim, got: {message}"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// One real browser turn (a `setTimeout(0)` drain) — the avatar/field
    /// `flush_one_turn` convention: leptos-side effects are scheduled on the
    /// browser's microtask/timer queue, which a synchronous assert never sees.
    async fn flush_one_turn() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .expect("window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    resolve.unchecked_ref(),
                    0,
                )
                .expect("setTimeout");
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .expect("await");
    }

    fn legend_view(id: Option<String>) -> AnyView {
        fieldset_legend_view(FieldsetLegendViewProps {
            id,
            children: Some(Box::new(|| "Billing details".into_any())),
            ..FieldsetLegendViewProps::default()
        })
        .into_any()
    }

    /// Mounts `FieldsetRoot` (the view fn) whose children closure builds the given
    /// subtree at the root's body time — the real composition path, so parts resolve
    /// the root's context exactly as component nesting does. The `UnmountHandle` is
    /// forgotten (the meter/field convention): dropping it unmounts the view and
    /// cancels its owner before any assertion runs.
    fn mount_fieldset_root(
        build_children: impl Fn() -> AnyView + Send + 'static,
        disabled: bool,
    ) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = document()
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        container.set_id("test-fieldset-root");
        document().body().unwrap().append_child(&container).unwrap();

        std::mem::forget(mount_to({ container.clone() }, move || {
            fieldset_root_view(FieldsetRootViewProps {
                disabled,
                children: Some(Box::new(move || build_children())),
                ..FieldsetRootViewProps::default()
            })
        }));

        container
    }

    fn fieldset_of(container: &HtmlElement) -> HtmlElement {
        container
            .query_selector("fieldset")
            .unwrap()
            .expect("the fieldset element rendered")
            .dyn_into::<HtmlElement>()
            .unwrap()
    }

    fn legend_of(container: &HtmlElement) -> HtmlElement {
        container
            .query_selector("div")
            .unwrap()
            .expect("the legend div rendered")
            .dyn_into::<HtmlElement>()
            .unwrap()
    }

    // FieldsetLegend.test.tsx:17-28 / :87-108 — the legend's generated id (present in
    // render, from `useBaseUiId`) is registered with the root, and the root's
    // `aria-labelledby` points at it. behavior.md "Accessibility": no `aria-labelledby`
    // before the registration lands, which is exactly the post-mount write this pins.
    #[wasm_bindgen_test]
    async fn the_legend_registers_its_generated_id_on_the_root() {
        let container = mount_fieldset_root(|| legend_view(None), false);
        flush_one_turn().await;

        let fieldset = fieldset_of(&container);
        let legend = legend_of(&container);
        let legend_id = legend.get_attribute("id").expect("the legend has an id");

        assert!(
            legend_id.starts_with("base-ui-"),
            "the generated id carries the base-ui prefix (useBaseUiId.ts:9-11), got {legend_id}"
        );
        assert_eq!(
            fieldset.get_attribute("aria-labelledby").as_deref(),
            Some(legend_id.as_str()),
            "the root's association points at the legend"
        );
    }

    // FieldsetLegend.test.tsx:30-38 — an explicit id is used verbatim.
    #[wasm_bindgen_test]
    async fn an_explicit_legend_id_is_used_verbatim() {
        let container = mount_fieldset_root(|| legend_view(Some("my-legend".to_string())), false);
        flush_one_turn().await;

        assert_eq!(
            legend_of(&container).get_attribute("id").as_deref(),
            Some("my-legend")
        );
        assert_eq!(
            fieldset_of(&container)
                .get_attribute("aria-labelledby")
                .as_deref(),
            Some("my-legend"),
            "the custom id is what the association points at"
        );
    }

    // FieldsetLegend.test.tsx:81-85 — with no legend rendered the attribute is absent
    // (React's `undefined` drops it; the port's `None` attribute resolution removes it).
    #[wasm_bindgen_test]
    async fn a_root_without_a_legend_has_no_association() {
        let container = mount_fieldset_root(|| "".into_any(), false);
        flush_one_turn().await;

        assert!(
            !fieldset_of(&container).has_attribute("aria-labelledby"),
            "no legend, no aria-labelledby"
        );
    }

    // FieldsetRoot.test.tsx:21-30 + :32-84 — the native `disabled` attribute lands on
    // the fieldset, the state hook follows it, and the effective-disabled rule ORs an
    // ancestor's value into a nested root whose own prop is false.
    #[wasm_bindgen_test]
    async fn the_effective_disabled_or_reaches_a_nested_root() {
        let container = mount_fieldset_root(
            || {
                fieldset_root_view(FieldsetRootViewProps {
                    disabled: false,
                    children: Some(Box::new(|| legend_view(None))),
                    ..FieldsetRootViewProps::default()
                })
                .into_any()
            },
            true,
        );
        flush_one_turn().await;

        let inner = fieldset_of(&container);
        assert!(
            inner.has_attribute("disabled"),
            "the ancestor's disabled reaches the nested fieldset"
        );
        assert_eq!(
            inner.get_attribute("data-disabled").as_deref(),
            Some(""),
            "the nested root's effective state is disabled"
        );
    }

    // FieldsetLegend.test.tsx:59-67 — unmounting the legend withdraws the association.
    // The legend is built inside its own leptos owner so it can be disposed while the
    // root keeps running; disposal runs the rg-0.2 owner cleanup that fires the ported
    // hook's guarded ClearIfCurrent dispatch.
    #[wasm_bindgen_test]
    async fn unmounting_the_legend_withdraws_the_association() {
        let _ = any_spawner::Executor::init_futures_executor();
        let legend_owner = send_wrapper::SendWrapper::new(leptos::reactive::owner::Owner::new());
        let dispose_handle = legend_owner.clone();

        let container = mount_fieldset_root(move || legend_owner.with(|| legend_view(None)), false);
        flush_one_turn().await;

        let fieldset = fieldset_of(&container);
        assert!(
            fieldset.has_attribute("aria-labelledby"),
            "the legend registered before the unmount"
        );

        dispose_handle.take().cleanup();
        let _ = any_spawner::Executor::poll_local();
        flush_one_turn().await;
        let _ = any_spawner::Executor::poll_local();
        flush_one_turn().await;

        assert!(
            !fieldset.has_attribute("aria-labelledby"),
            "the withdrawn registration clears the root's association"
        );
    }

    // FieldsetRoot.test.tsx:75-83 / FieldRoot.tsx:44,48 (the cross-unit consumer):
    // a Field nested in the fieldset inherits the effective disabled — the context
    // shape this port preserves for `field`.
    #[wasm_bindgen_test]
    async fn a_nested_field_inherits_the_disabled_state() {
        use crate::field::field_control::field_control_view;
        use crate::field::field_root::{FieldRootViewProps, field_root_view};

        let container = mount_fieldset_root(
            || {
                field_root_view(FieldRootViewProps {
                    children: Some(Box::new(|| {
                        field_control_view(Default::default()).into_any()
                    })),
                    ..FieldRootViewProps::default()
                })
                .into_any()
            },
            true,
        );
        flush_one_turn().await;

        let input = container
            .query_selector("input")
            .unwrap()
            .expect("the control rendered")
            .dyn_into::<HtmlElement>()
            .unwrap();
        assert!(
            input.has_attribute("disabled"),
            "the fieldset's native disabled reaches the control"
        );
    }
}
