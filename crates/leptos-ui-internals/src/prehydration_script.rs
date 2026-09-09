//! Port of `packages/react/src/internals/PrehydrationScript.tsx` — the inline-script
//! emitter that positions server-rendered content ahead of hydration.
//!
//! Upstream (59 lines) renders an inline `<script>` that runs before React hydrates,
//! used by components that need to style server-rendered content before the client
//! boots — Tabs.Indicator (`packages/react/src/tabs/indicator/TabsIndicator.tsx:170`)
//! and Slider.Thumb (`packages/react/src/slider/thumb/SliderThumb.tsx:469`); the script
//! bodies are the `#prehydration/*` subpath modules (`packages/react/package.json:112-118`).
//! The component itself is three lines of logic (`:32-49`):
//!
//! - `useCSPContext()` for the `nonce` (`:34`) — the CSP contract covers script nonces
//!   too, matching the provider's JSDoc "inline `<style>` or `<script>` tags"
//!   (`specs/library/internals/implementation.md`, "Context providers/consumers" row for
//!   `CSPContext`: PrehydrationScript "reads only `nonce`").
//! - `useIsHydrating()` gates the whole return (`:35`, `:37-39`): not hydrating → `null`.
//!   The element therefore exists exactly during server rendering and the hydration
//!   render, and unmounts once hydration completes ("Once `isHydrating` flips to `false`
//!   the element unmounts", `:20-21`).
//! - The rendered element is `<script nonce dangerouslySetInnerHTML={{ __html: script }}
//!   suppressHydrationWarning />` (`:41-48`): the script body becomes the element's HTML
//!   content (`:45`), and the browser executes inline scripts from server HTML as it
//!   parses them — that parse-time execution *is* the feature.
//!
//! The long doc comment (`:6-31`) is the authoritative rationale for the bundle/hydration
//! strategy: the script body is imported through a `#prehydration/*` subpath whose
//! `browser` condition resolves to the shared stub exporting an empty string
//! (`packages/react/src/internals/prehydrationScript.stub.ts:6`), so the body only ever
//! exists in server bundles; the component must still render an empty script element
//! during the hydration pass so the React tree matches the server markup, with
//! `suppressHydrationWarning` bridging the content difference — returning `null` on the
//! client wholesale would drop an element the server emitted and trigger a recoverable
//! hydration error (React #418).
//!
//! ## Rust adaptations
//!
//! - **The component returns an element description**: `None` for the `null` branch
//!   (`:37-39`), [`Some`] of a `script`-tag [`RenderedElement`] otherwise — the crate's
//!   element-vocabulary convention (`use_render_element.rs` module docs; the component
//!   is *not* a `useRenderElement` call site upstream, it returns a bare `<script>`, so
//!   the description is built directly rather than through the merge machinery).
//!   [`RenderedElement::create_element`] materializes it, standing in for React
//!   rendering the return value.
//! - **`dangerouslySetInnerHTML` rides the shared bag** as the new
//!   [`RenderElementProps::inner_html`] member (`:45`); materialization applies it as
//!   the element's HTML content. For a `script` element that content is the program text
//!   the browser executes when the element connects.
//! - **The `nonce` rides the lazy-attribute convention** (the `composite_view.rs`
//!   roving-`tabIndex` precedent): the [`crate::use_csp_context`] memo is captured once
//!   (upstream's hook call, `:34`) and re-read at attribute-read time, so a context-side
//!   nonce change re-renders the attribute the way upstream's context re-render does. An
//!   absent nonce yields `None` from the closure — no `nonce` attribute, upstream's
//!   `undefined` prop (`:43`).
//! - **The `#prehydration/*` browser-condition stub is N/A** — there is no JS bundler to
//!   exclude the body from client bundles. The body is inert data in Rust until a script
//!   element carrying it connects to a document: execution happens from server HTML
//!   parsing (upstream's only real execution path) or from a client render inserting the
//!   element, which only occurs during a hydration pass by the flag's contract — the
//!   fresh-client-mount default stays `false`, so a plain client mount emits nothing.
//!   Keeping server-HTML execution correct (body present in the server pass) while the
//!   client hydration pass reuses the already-executed server node is Phase C's
//!   hydration-render contract, the seam the React runtime owned upstream.
//! - **`suppressHydrationWarning` is N/A** — there is no reconciler comparing server and
//!   client content; the description always carries the real body (upstream carried the
//!   stub's empty body client-side and suppressed the resulting mismatch warning).
//! - **React #418 / "must stay in client bundles" (`:23-26`) is N/A** — the constraint
//!   it protects (the hydration-render tree must match the server markup) is expressed
//!   here by the gate itself: the element exists in every hydration pass and is absent
//!   only for fresh mounts.
//! - `'use client'` (`:1`) is N/A — no React Server Components boundary in Rust.
//! - **No upstream tests exist for this unit** (`specs/library/internals/implementation.md`,
//!   "Anything in source not explained by any test", item 5) — the suite below pins the
//!   written mechanics only.

use std::rc::Rc;

use reactive_graph::traits::{Get, With};

use crate::csp_context::use_csp_context;
use crate::floating_ui::element_props::ElementAttributeFn;
use crate::use_is_hydrating::use_is_hydrating;
use crate::use_render_element::{RenderElementProps, RenderedElement};

/// The component props — upstream `PrehydrationScript.Props`
/// (`packages/react/src/internals/PrehydrationScript.tsx:51-59`).
pub struct PrehydrationScriptProps {
    /// The script source — upstream's `script` (`:54-58`), imported through the
    /// `#prehydration/*` subpath upstream and passed verbatim here.
    pub script: String,
}

/// Port of `PrehydrationScript` (`PrehydrationScript.tsx:32-49`). Must be called inside a
/// reactive owner (a component) — the CSP context read needs the owner chain. Returns
/// `None` when the crate is not in a server-render/hydration pass (upstream's `null`,
/// the fresh-client-mount world) and the `script`-element description otherwise; the
/// description is `None` again once [`crate::set_is_hydrating`] flips the pass off,
/// which is the post-hydration unmount.
pub fn prehydration_script(props: PrehydrationScriptProps) -> Option<RenderedElement> {
    let csp = use_csp_context();

    if !use_is_hydrating().get() {
        return None;
    }

    let nonce_attribute: ElementAttributeFn =
        Rc::new(move || csp.with(|value| value.nonce.clone()));

    Some(RenderedElement {
        tag: "script".to_string(),
        props: RenderElementProps {
            inner_html: Some(props.script),
            handlers: crate::use_render_element::RenderElementHandlers {
                attributes: vec![("nonce".to_string(), nonce_attribute)],
                ..Default::default()
            },
            ..RenderElementProps::default()
        },
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;
    use reactive_graph::traits::GetUntracked;

    use super::*;
    use crate::csp_provider::provide_csp_context;
    use crate::use_is_hydrating::{ResetIsHydrating, set_is_hydrating};

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn attributes(description: &RenderedElement) -> Vec<(String, Option<String>)> {
        description
            .props
            .handlers
            .attributes
            .iter()
            .map(|(name, value)| (name.clone(), value()))
            .collect()
    }

    // The hydration gate (`PrehydrationScript.tsx:35,:37-39`): in the fresh-client-mount
    // world (the flag's default) the component renders nothing — "return null".
    #[test]
    fn renders_nothing_when_not_hydrating() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();

        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: "window.__x = 1;".to_string(),
            })
            .is_none(),
            "not hydrating — the null branch"
        );
    }

    // The script-element return (`:41-48`) as a description: tag `script`, the body as
    // the inner-HTML member, emitted whenever a hydration pass is in flight.
    #[test]
    fn the_hydration_pass_yields_the_script_element_description() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();
        set_is_hydrating(true);

        let description = prehydration_script(PrehydrationScriptProps {
            script: "window.__x = 1;".to_string(),
        })
        .expect("hydrating — the element renders");
        assert_eq!(description.tag, "script", "the inline script tag");
        assert_eq!(
            description.props.inner_html.as_deref(),
            Some("window.__x = 1;"),
            "the script body rides the inner-HTML member (dangerouslySetInnerHTML, :45)"
        );
    }

    // The CSP nonce (`:34`, `:43`): without a provider the hook's default has no nonce,
    // so the lazy attribute resolves to None — no `nonce` attribute, upstream's
    // `undefined` prop.
    #[test]
    fn without_a_provider_the_nonce_attribute_is_absent() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();
        set_is_hydrating(true);

        let description = prehydration_script(PrehydrationScriptProps {
            script: String::new(),
        })
        .expect("hydrating");
        let rendered = attributes(&description);
        assert_eq!(rendered.len(), 1, "only the nonce slot is carried");
        assert_eq!(rendered[0].0, "nonce");
        assert_eq!(rendered[0].1, None, "no CSP provider — no nonce attribute");
    }

    // The CSP nonce with a provider: the lazy attribute re-reads the context memo at
    // read time (the composite_view lazy-attribute convention), so the value flows
    // through even though the description was built before the read.
    #[test]
    fn a_provided_nonce_flows_through_the_lazy_attribute() {
        let parent = owner();
        let _value = provide_csp_context(
            RwSignal::new(Some("test-nonce".to_string())),
            RwSignal::new(None),
        );

        let child = parent.child();
        child.set();
        let _guard = ResetIsHydrating::new();
        set_is_hydrating(true);

        let description = prehydration_script(PrehydrationScriptProps {
            script: String::new(),
        })
        .expect("hydrating");
        let rendered = attributes(&description);
        assert_eq!(
            rendered[0],
            ("nonce".to_string(), Some("test-nonce".to_string())),
            "the provider's nonce reaches the description's nonce attribute"
        );
    }

    // The post-hydration unmount (`:20-21`, "Once `isHydrating` flips to `false` the
    // element unmounts"): after the pass ends the component renders nothing again — the
    // same call, re-evaluated in the post-flip world.
    #[test]
    fn renders_nothing_again_once_the_pass_ends() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();

        set_is_hydrating(true);
        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: String::new(),
            })
            .is_some(),
            "during the pass the element renders"
        );

        set_is_hydrating(false);
        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: String::new(),
            })
            .is_none(),
            "after the pass the element unmounts (the null branch re-evaluated)"
        );
    }

    // The default CSP context read stays alive across the early return: a caller that
    // never reaches the element description still observes the fallback memo
    // (`useIsHydrating` gate short-circuiting after the `useCSPContext()` call, `:34-39`).
    #[test]
    fn the_gate_short_circuit_still_resolves_the_csp_fallback() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();

        let csp = use_csp_context();
        assert_eq!(
            csp.get_untracked().disable_style_elements,
            false,
            "the fallback default is observable regardless of the gate"
        );
        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: String::new(),
            })
            .is_none(),
            "the gate still short-circuits"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::csp_provider::provide_csp_context;
    use crate::use_is_hydrating::{ResetIsHydrating, set_is_hydrating};
    use reactive_graph::owner::Owner;
    use reactive_graph::signal::RwSignal;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn owner() -> Owner {
        let owner = Owner::new();
        owner.set();
        owner
    }

    fn document() -> web_sys::Document {
        web_sys::window().expect("no window").document().expect("no document")
    }

    // Deletes a window global the execution test sets, so the app-long-lived global
    // object cannot leak state between cases.
    fn delete_global(name: &str) {
        let window = web_sys::window().expect("no window");
        js_sys::Reflect::delete_property(&window, &name.into())
            .expect("delete global");
    }

    fn global(name: &str) -> wasm_bindgen::JsValue {
        let window = web_sys::window().expect("no window");
        js_sys::Reflect::get(&window, &name.into()).expect("read global")
    }

    // The hydration gate under the browser runtime: nothing materializes outside a pass.
    #[wasm_bindgen_test]
    fn renders_nothing_when_not_hydrating() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();

        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: String::new(),
            })
            .is_none(),
            "not hydrating — the null branch"
        );
    }

    // The description materializes into a real `<script>` element carrying the body as
    // its HTML content — the seam standing in for React rendering the returned element.
    #[wasm_bindgen_test]
    fn the_description_materializes_into_a_script_element_carrying_the_body() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();
        set_is_hydrating(true);

        let description = prehydration_script(PrehydrationScriptProps {
            script: "window.__prehydration_body_marker = 'carried';".to_string(),
        })
        .expect("hydrating");
        let (element, _cleanup) = description.create_element();

        assert_eq!(
            element.tag_name(),
            "SCRIPT",
            "the description's tag materializes"
        );
        assert_eq!(
            element.inner_html(),
            "window.__prehydration_body_marker = 'carried';",
            "the body rides the element's HTML content (dangerouslySetInnerHTML, :45)"
        );
        assert!(
            !element.has_attribute("nonce"),
            "no CSP provider — no nonce attribute"
        );
    }

    // The CSP nonce reaches the materialized element through the provider.
    #[wasm_bindgen_test]
    fn a_provided_nonce_reaches_the_materialized_element() {
        let parent = owner();
        let _value = provide_csp_context(
            RwSignal::new(Some("wasm-nonce".to_string())),
            RwSignal::new(None),
        );

        let child = parent.child();
        child.set();
        let _guard = ResetIsHydrating::new();
        set_is_hydrating(true);

        let description = prehydration_script(PrehydrationScriptProps {
            script: String::new(),
        })
        .expect("hydrating");
        let (element, _cleanup) = description.create_element();

        assert_eq!(
            element.get_attribute("nonce").as_deref(),
            Some("wasm-nonce"),
            "the provider's nonce is the materialized element's nonce attribute"
        );
    }

    // The end-to-end feature (`:6-9`): an inline script "runs before React hydrates" —
    // the browser executes the materialized script's body when the element connects to
    // the document, exactly how server HTML's inline scripts run at parse time.
    #[wasm_bindgen_test]
    fn the_materialized_script_executes_when_it_connects() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();
        set_is_hydrating(true);

        let description = prehydration_script(PrehydrationScriptProps {
            script: "window.__prehydration_exec_marker = 'ran';".to_string(),
        })
        .expect("hydrating");
        let (element, _cleanup) = description.create_element();

        assert!(
            global("__prehydration_exec_marker").is_undefined(),
            "a detached script element has not executed yet"
        );

        document()
            .body()
            .expect("body")
            .append_child(&element)
            .expect("append script");
        assert_eq!(
            global("__prehydration_exec_marker").as_string().as_deref(),
            Some("ran"),
            "connecting the element executes the body"
        );

        element.remove();
        delete_global("__prehydration_exec_marker");
    }

    // The post-hydration unmount under the browser runtime: after the pass ends the
    // component renders nothing again.
    #[wasm_bindgen_test]
    fn renders_nothing_again_once_the_pass_ends() {
        let _owner = owner();
        let _guard = ResetIsHydrating::new();

        set_is_hydrating(true);
        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: String::new(),
            })
            .is_some(),
            "during the pass the element renders"
        );

        set_is_hydrating(false);
        assert!(
            prehydration_script(PrehydrationScriptProps {
                script: String::new(),
            })
            .is_none(),
            "after the pass the element unmounts"
        );
    }
}
