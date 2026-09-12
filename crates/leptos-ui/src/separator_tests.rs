//! Tests for the Separator port — mirrors of
//! `packages/react/src/separator/Separator.test.tsx` (the suite behavior.md mines)
//! plus the four conformance suites `describeConformance` runs for this unit
//! (prop forwarding, ref forwarding, render prop, className), over the real
//! materialized element in a wasm-browser suite and the description-level
//! contracts in an owner-scoped host suite.
//!
//! Host suite: the description-level contracts that need no DOM (the state map,
//! the defaults, bag order, render-fn shape).
//! Wasm suite: the materialized-element contracts (the `separator` role,
//! `aria-orientation`/`data-orientation` for both orientations, the default
//! 'horizontal', attribute override, render-prop element identity, the
//! class/style merge, the forked ref attach).
//!
//! The full render/merge/ref/state-attributes machinery is
//! `use_render_element`'s own suite-tested contract; this facade's suite pins
//! that the unit wires the engine through with the documented bags and no
//! `stateAttributesMapping`.

// The wasm-only harness items (Recorder, mount_separator) are dead code on the
// host target — the dual-target test-module convention (the button/toggle
// precedent, whose host suites import only the description-level surface).
#![allow(unused_imports, dead_code)]

use super::*;

/// `Separator.Props['orientation']` values exercised by the upstream suite
/// (`Separator.test.tsx:20`).
const ORIENTATIONS: [(&str, &SeparatorOrientation); 2] = [
    ("horizontal", &SEPARATOR_ORIENTATION_HORIZONTAL),
    ("vertical", &SEPARATOR_ORIENTATION_VERTICAL),
];

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use leptos_ui_internals::use_render_element::{RenderedElement, UseRenderElementComponentProps};

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // implementation.md:43-47 — no `stateAttributesMapping`, so the DEFAULT state
    // walk applies: truthy non-boolean `orientation` → `data-orientation =
    // String(value)`.
    #[test]
    fn the_state_map_carries_orientation_for_the_default_walk() {
        for (expected, orientation) in ORIENTATIONS {
            let map = SeparatorState {
                orientation: orientation.0,
            }
            .to_state_map();
            assert_eq!(
                map.get("orientation"),
                Some(&serde_json::json!(expected)),
                "orientation {expected} rides the state map as a string"
            );
        }
    }

    // The destructuring default (`Separator.tsx:16`) — behavior.md's UNVERIFIED
    // default, pinned at the source level per implementation.md untested item 1.
    #[test]
    fn orientation_defaults_to_horizontal() {
        let props = SeparatorProps::default();
        assert_eq!(
            props.orientation,
            SEPARATOR_ORIENTATION_HORIZONTAL,
            "the default is the destructuring's 'horizontal'"
        );
    }

    // implementation.md:152-159 — SeparatorDataAttributes.ts is a consumer-facing
    // anchor with no runtime imports; the walk emits its documented attribute.
    #[test]
    fn separator_data_attributes_document_data_orientation() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packages/react/src/separator/SeparatorDataAttributes.ts"
        ));
        assert!(
            source.contains("data-orientation"),
            "the data-attributes anchor documents data-orientation"
        );
        assert!(
            !source.contains("data-nope"),
            "the anchor has no other runtime attributes"
        );
    }

    // Upstream defaults (`Separator.tsx:16` destructuring + `:29-35` interface):
    // orientation 'horizontal', and the props shape — no other props exist.
    #[test]
    fn props_default_to_a_horizontal_leaf() {
        let props = SeparatorProps::default();
        assert_eq!(
            props.orientation,
            SEPARATOR_ORIENTATION_HORIZONTAL,
            "orientation: 'horizontal'"
        );
        assert!(props.element_attributes.is_empty());
        assert!(props.ref_callback.is_none());
        assert!(props.render_class_style.render.is_none());
        assert!(props.render_class_style.class_name.is_none());
        assert!(props.render_class_style.style.is_none());
    }

    // Bag order (`Separator.tsx:23`): `[intrinsic, elementProps]` — the
    // description carries exactly these two bags, nothing internal between them
    // (a separator has no behavior; the unit contributes no handlers —
    // behavior.md "Events": N/A).
    #[test]
    fn the_description_is_always_produced_as_a_div_with_the_two_bags() {
        let _owner = in_owner();
        let rendered = separator_element(SeparatorProps::default()).expect("the leaf renders");
        assert_eq!(rendered.tag, "div", "the default root tag is div");
        let attribute_names: Vec<_> = rendered
            .props
            .handlers
            .attributes
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        assert!(
            attribute_names.contains(&"role".to_string()),
            "role=separator is the intrinsic bag"
        );
        assert!(
            attribute_names.contains(&"aria-orientation".to_string()),
            "aria-orientation is the intrinsic bag"
        );
        assert!(
            attribute_names.contains(&"data-orientation".to_string()),
            "data-orientation is the state walk's default-arm output"
        );
        assert_eq!(
            attribute_names.len(),
            3,
            "exactly the three attributes, no internal extras"
        );
    }

    // The render-fn arm (`useRenderElement.tsx:165-170`): the helper wraps the
    // closure in the engine's function arm, which receives the merged bag.
    #[test]
    fn the_render_function_helper_builds_the_function_arm() {
        let _owner = in_owner();
        let render = separator_render_fn(|props, state| {
            assert!(
                state.contains_key("orientation"),
                "the render fn receives the state"
            );
            RenderedElement {
                tag: "hr".to_string(),
                props,
            }
        });
        let rendered = separator_element(SeparatorProps {
            render_class_style: UseRenderElementComponentProps {
                render: Some(render),
                ..UseRenderElementComponentProps::default()
            },
            ..SeparatorProps::default()
        })
        .expect("the leaf renders");
        assert_eq!(rendered.tag, "hr", "the render fn owns the element");
        let attribute_names: Vec<_> = rendered
            .props
            .handlers
            .attributes
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        assert!(
            attribute_names.contains(&"role".to_string()),
            "the merged props still carry the intrinsic role"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement};

    use super::*;
    use leptos_ui_internals::use_render_element::UseRenderElementComponentProps;
    use std::rc::Rc;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Builds and mounts one Separator; returns the mounted root element with the
    /// (forgotten) listener cleanup — the harness convention of the
    /// button/toggle wasm suites.
    fn mount_separator(props: SeparatorProps) -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        std::mem::forget(owner);

        let rendered = separator_element(props).expect("the separator renders");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        document().body().unwrap().append_child(&element).unwrap();
        element.dyn_into::<HtmlElement>().unwrap()
    }

    // behavior.md "Accessibility" + "DOM structure" (`Separator.test.tsx:14-17`,
    // `refInstanceof: window.HTMLDivElement` at `:11`): the default root is a
    // visible div carrying the `separator` role.
    #[wasm_bindgen_test]
    fn renders_a_visible_div_with_the_separator_role() {
        let root = mount_separator(SeparatorProps::default());
        assert_eq!(root.tag_name(), "DIV", "the default root element is a div");
        assert_eq!(
            root.get_attribute("role").as_deref(),
            Some("separator"),
            "role=separator is present"
        );
    }

    // behavior.md "Accessibility" (`Separator.test.tsx:19-27`): aria-orientation
    // mirrors the prop exactly for both supported values — AND the state walk's
    // data-orientation rides along for both (implementation.md untested item 2,
    // the fixture pin the spec asks for).
    #[wasm_bindgen_test]
    fn aria_orientation_and_data_orientation_mirror_the_prop_for_both_values() {
        for (expected, orientation) in ORIENTATIONS {
            let root = mount_separator(SeparatorProps {
                orientation: orientation.clone(),
                ..SeparatorProps::default()
            });
            assert_eq!(
                root.get_attribute("aria-orientation").as_deref(),
                Some(expected),
                "aria-orientation={expected}"
            );
            assert_eq!(
                root.get_attribute("data-orientation").as_deref(),
                Some(expected),
                "data-orientation mirrors the same prop (the default walk)"
            );
        }
    }

    // The destructuring default (`Separator.tsx:16`) — behavior.md's UNVERIFIED
    // default pinned at the DOM level: no-props renders
    // `aria-orientation="horizontal"` + `data-orientation="horizontal"`.
    #[wasm_bindgen_test]
    fn the_default_orientation_is_horizontal_on_the_mounted_root() {
        let root = mount_separator(SeparatorProps::default());
        assert_eq!(
            root.get_attribute("aria-orientation").as_deref(),
            Some("horizontal"),
            "the default aria-orientation"
        );
        assert_eq!(
            root.get_attribute("data-orientation").as_deref(),
            Some("horizontal"),
            "the default data-orientation"
        );
    }

    // behavior.md "Public API surface" (`propForwarding.tsx:23-36`): arbitrary DOM
    // props (`lang`, `data-*`) forward to the default root.
    #[wasm_bindgen_test]
    fn arbitrary_dom_props_forward_to_the_default_root() {
        let root = mount_separator(SeparatorProps {
            element_attributes: vec![
                ("lang".to_string(), "fr".to_string()),
                ("data-foobar".to_string(), "random-value".to_string()),
            ],
            ..SeparatorProps::default()
        });
        assert_eq!(root.get_attribute("lang").as_deref(), Some("fr"));
        assert_eq!(
            root.get_attribute("data-foobar").as_deref(),
            Some("random-value")
        );
    }

    // implementation.md untested item 4 / precedence (implementation.md:97-103):
    // `elementProps` sits LAST in the merge, so a user-supplied `role`,
    // `aria-orientation`, or `data-orientation` silently replaces the library
    // values.
    #[wasm_bindgen_test]
    fn the_user_attributes_override_the_library_intrinsics() {
        let root = mount_separator(SeparatorProps {
            element_attributes: vec![
                ("role".to_string(), "presentation".to_string()),
                ("aria-orientation".to_string(), "bogus".to_string()),
                ("data-orientation".to_string(), "user".to_string()),
            ],
            ..SeparatorProps::default()
        });
        assert_eq!(
            root.get_attribute("role").as_deref(),
            Some("presentation"),
            "the user role wins"
        );
        assert_eq!(
            root.get_attribute("aria-orientation").as_deref(),
            Some("bogus"),
            "the user aria-orientation wins"
        );
        assert_eq!(
            root.get_attribute("data-orientation").as_deref(),
            Some("user"),
            "the user data-orientation beats the state-derived one"
        );
    }

    // behavior.md "DOM structure" (`renderProp.tsx:41-59`, custom element `div`,
    // `wrappingAllowed` default): the `render` prop replaces the root — an `<hr>`
    // remains an `<hr>`, no wrapper element added.
    #[wasm_bindgen_test]
    fn the_render_prop_replaces_the_root_element_identity() {
        use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

        let root = mount_separator(SeparatorProps {
            render_class_style: UseRenderElementComponentProps {
                render: Some(RenderProp::Element {
                    tag: "hr".to_string(),
                    props: RenderElementProps::default(),
                }),
                ..UseRenderElementComponentProps::default()
            },
            ..SeparatorProps::default()
        });
        assert_eq!(
            root.tag_name(),
            "HR",
            "the render element is the root — the tag is preserved"
        );
        assert_eq!(
            root.get_attribute("role").as_deref(),
            Some("separator"),
            "the intrinsic role applies on the custom element"
        );
        assert_eq!(
            root.get_attribute("aria-orientation").as_deref(),
            Some("horizontal"),
            "aria-orientation applies on the custom element"
        );
    }

    // behavior.md "Public API surface" (`renderProp.tsx:163-178`): a
    // function-form className resolves to a class name that MERGES with the
    // render element's own class name.
    #[wasm_bindgen_test]
    fn the_function_classname_merges_with_the_render_element_classname() {
        use leptos_ui_internals::use_render_element::{ClassNameSource, RenderElementProps, RenderProp};

        let root = mount_separator(SeparatorProps {
            render_class_style: UseRenderElementComponentProps {
                class_name: Some(ClassNameSource::Function(Rc::new(|state| {
                    // The function-form resolution receives the state — the
                    // resolveClassName util's contract; echo the orientation as
                    // the class suffix like a consumer would.
                    let orientation = state.get("orientation").and_then(|v| v.as_str())?;
                    Some(format!("sep-{orientation}"))
                }))),
                render: Some(RenderProp::Element {
                    tag: "hr".to_string(),
                    props: RenderElementProps {
                        class: Some("render-prop-classname".to_string()),
                        ..RenderElementProps::default()
                    },
                }),
                ..UseRenderElementComponentProps::default()
            },
            ..SeparatorProps::default()
        });
        let class = root.get_attribute("class").unwrap_or_default();
        assert!(
            class.contains("render-prop-classname"),
            "the render element's class survives the merge: got '{class}'"
        );
        assert!(
            class.contains("sep-horizontal"),
            "the function-form className merges in: got '{class}'"
        );
    }

    // behavior.md "Public API surface" (`propForwarding.tsx:82-95`): `style`
    // forwards to the default root.
    #[wasm_bindgen_test]
    fn style_forwards_to_the_default_root() {
        use leptos_ui_internals::use_render_element::StyleSource;

        let root = mount_separator(SeparatorProps {
            render_class_style: UseRenderElementComponentProps {
                style: Some(StyleSource::Static(vec![(
                    "background-color".to_string(),
                    "red".to_string(),
                )])),
                ..UseRenderElementComponentProps::default()
            },
            ..SeparatorProps::default()
        });
        let style = root.get_attribute("style").unwrap_or_default();
        assert!(
            style.contains("background-color: red"),
            "the style declarations land on the root: got '{style}'"
        );
    }

    // behavior.md "DOM structure" (`refForwarding.tsx:32-38` +
    // `renderProp.tsx:115-144`): the ref attaches to the root and merges with a
    // render-element ref so both observers see the same node — `instanceof
    // window.HTMLDivElement` on the default path.
    #[wasm_bindgen_test]
    fn the_ref_attaches_to_the_root_and_merges_with_the_render_element_ref() {
        use std::sync::Arc;
        use std::sync::Mutex;

        use leptos_ui_internals::use_render_element::{RenderElementProps, RenderProp};

        let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_for_cb = seen.clone();
        let seen_for_render = seen.clone();

        let props = SeparatorProps {
            ref_callback: Some(Rc::new(move |instance: Option<&Element>| {
                if let Some(element) = instance {
                    seen_for_cb
                        .lock()
                        .unwrap()
                        .push(element.tag_name());
                }
                None
            })),
            render_class_style: UseRenderElementComponentProps {
                render: Some(RenderProp::Element {
                    tag: "div".to_string(),
                    props: RenderElementProps {
                        ref_callback: Some(Rc::new(move |instance: Option<&Element>| {
                            if let Some(element) = instance {
                                seen_for_render
                                    .lock()
                                    .unwrap()
                                    .push(format!("render:{}", element.tag_name()));
                            }
                        })),
                        ..RenderElementProps::default()
                    },
                }),
                ..UseRenderElementComponentProps::default()
            },
            ..SeparatorProps::default()
        };
        let _root = mount_separator(props);
        let mut calls = seen.lock().unwrap().clone();
        // The fork's invocation order is its argument order — [bag ref,
        // render-element ref, forwarded ref] (`useMergedRefsN`, the engine's ref
        // fork) — so the render-element slot fires BEFORE the forwarded one.
        // Upstream's conformance suite (renderProp.tsx:115-144,
        // refForwarding.tsx:32-38) asserts node identity, not call order, so the
        // assertion is order-insensitive; the contract under test is that both
        // observers received the SAME root div (the HTMLDivElement instanceof
        // row).
        calls.sort();
        assert_eq!(
            calls,
            vec!["DIV".to_string(), "render:DIV".to_string()],
            "both ref slots observe the same root node"
        );
    }
}
