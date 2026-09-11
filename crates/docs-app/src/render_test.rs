//! Wasm-browser render tests for the docs-app pages, per the Stage 3 prompt's
//! step 6 ("confirm the relevant route renders without panicking"). The
//! Playwright differential check (`ralph/scripts/playwright-diff.mjs`) does not
//! exist yet, so this in-browser render assertion is the available verification.
//!
//! Run with:
//! `CHROME=... CHROMEDRIVER=... cargo test -p docs-app --target wasm32-unknown-unknown`

#![cfg(target_arch = "wasm32")]

use crate::App;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn app_mounts_and_renders_the_shell() {
    // Mount the app for real in the test browser; App installs its own Router,
    // so a fresh document body gets the header shell at a minimum.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    let _guard = leptos::mount::mount_to({ container.clone() }, App);

    let html = container.inner_html();
    assert!(
        html.contains("docs-title"),
        "app shell did not render; html was: {html}"
    );
}

#[wasm_bindgen_test]
fn app_mounts_without_panicking() {
    // The real done-when gate is "renders without panicking"; mounting the
    // full App (Router + routes + the collapsible hero demo's component tree)
    // and surviving any reactive effect flush is that assertion at its core.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-2");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    let _guard = leptos::mount::mount_to({ container.clone() }, App);

    assert!(
        leptos::prelude::document().get_element_by_id("test-mount-root-2").is_some(),
        "mount target vanished — the app unmounted its own root"
    );
}

#[wasm_bindgen_test]
fn use_render_page_renders_the_text_demo_through_the_real_hook() {
    // The use-render docs page builds its demos through the real
    // leptos_ui_internals::use_render (the ported hook), so mounting the App
    // and navigating to the route must produce the two Text elements: the
    // default `<p>` and the render-prop-overridden `<strong>`.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-use-render");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::use_render_page::{TextProps, text_element};
    use wasm_bindgen::JsCast;

    let default_paragraph = text_element(TextProps {
        render_tag: None,
        children: "Text component rendered as a paragraph tag".to_string(),
    });
    let strong_override = text_element(TextProps {
        render_tag: Some("strong".to_string()),
        children: "Text component rendered as a strong tag".to_string(),
    });
    container
        .append_child(&default_paragraph.element)
        .expect("append p");
    container
        .append_child(&strong_override.element)
        .expect("append strong");

    let html = container.inner_html();
    assert!(
        html.contains("<p class=\"docs-use-render-text\">"),
        "the defaultTagName element did not render; html was: {html}"
    );
    assert!(
        html.contains("<strong class=\"docs-use-render-text\">"),
        "the render-prop element override did not replace the default tag; html was: {html}"
    );

    // The merged props/children flow into the overriding element: the class
    // came from the component's merged bag, the children text from the
    // consumer's remaining props.
    let strong = container
        .query_selector("strong.docs-use-render-text")
        .expect("query")
        .expect("strong present")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("element");
    assert_eq!(
        strong.text_content().as_deref(),
        Some("Text component rendered as a strong tag"),
        "the children did not flow into the render-prop element"
    );
}

#[wasm_bindgen_test]
fn csp_provider_page_renders_probes_through_the_real_provider_and_hook() {
    // The csp-provider docs page has no demos (the upstream page.mdx has no
    // demos/ directory), so the real-implementation half of the done-when is
    // the page's live provider machinery: `CSPProviderView` publishes the
    // config through the ported `provide_csp_context` under a real
    // reactive-graph owner, and the child `CspProbe` reads it back through the
    // real `use_csp_context` hook. Mounting the page's two provider instances
    // and asserting the probes' text pins the end-to-end context flow —
    // including the innermost-provider-wins nesting (the second provider's
    // `disableStyleElements: true` must not leak into the first probe, and its
    // own probe must see it).
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-csp");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::csp_provider_page::{CSPProviderView, CspProbe};
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _guard = mount_to(
        { container.clone() },
        move || {
            view! {
                <CSPProviderView nonce=Some("test-nonce".to_string()) disable_style_elements=Some(false)>
                    <CspProbe />
                </CSPProviderView>
                <CSPProviderView nonce=None disable_style_elements=Some(true)>
                    <CspProbe />
                </CSPProviderView>
            }
        },
    );

    let html = container.inner_html();
    assert!(
        html.contains("docs-csp-probe"),
        "no probe rendered; html was: {html}"
    );

    let probes: Vec<web_sys::Element> = {
        let list = container
            .query_selector_all(".docs-csp-probe")
            .expect("query all probes");
        let mut out = Vec::new();
        for i in 0..list.length() {
            out.push(
                list.get(i)
                    .expect("item at index")
                    .dyn_into::<web_sys::Element>()
                    .expect("element"),
            );
        }
        out
    };
    assert_eq!(
        probes.len(),
        2,
        "both provider instances should have rendered a probe; html was: {html}"
    );

    let first = probes[0].text_content().unwrap_or_default();
    assert!(
        first.contains("nonce: test-nonce"),
        "first probe did not see the provided nonce; it read: {first}"
    );
    assert!(
        first.contains("disableStyleElements: false"),
        "first probe did not see disableStyleElements=false; it read: {first}"
    );

    let second = probes[1].text_content().unwrap_or_default();
    assert!(
        second.contains("nonce: (none)"),
        "second (no-nonce) provider did not override wholesale — the innermost provider wins with no per-prop inheritance; it read: {second}"
    );
    assert!(
        second.contains("disableStyleElements: true"),
        "second probe did not see disableStyleElements=true; it read: {second}"
    );
}

#[wasm_bindgen_test]
fn csp_provider_route_renders_without_panicking() {
    // Mount the full App and drive the router to the csp-provider route by
    // dispatching a click on a navigation link — or, since the shell exposes no
    // links yet, mount the page component directly under an owner chain (the
    // real mount path for the route's view).
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-csp-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::csp_provider_page::CSPProviderPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _guard = mount_to({ container.clone() }, || {
        view! { <CSPProviderPage /> }
    });

    let html = container.inner_html();
    assert!(
        html.contains("CSP Provider"),
        "page h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("Supplying a nonce"),
        "page sections did not render; html was: {html}"
    );
    assert!(
        html.contains("test-nonce"),
        "the live provider demo did not render; html was: {html}"
    );
}

#[wasm_bindgen_test]
fn toggle_page_renders_the_hero_demo_through_the_real_port() {
    // The toggle docs page's live demo is the upstream hero
    // (`docs/src/app/(docs)/react/components/toggle/demos/hero/tailwind/index.tsx`)
    // ported onto the real `leptos_ui::toggle_element`: an uncontrolled
    // Toggle whose render prop swaps the icon on `state.pressed`. Materialize
    // the element, click it through the real handler bag, and assert the
    // state machine end-to-end: aria-pressed flips false->true, data-pressed
    // appears (the state mapping), and the inner HTML swaps to the filled
    // heart — the demo's observable contract.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-toggle");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::toggle_page::toggle_hero_demo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;
    use wasm_bindgen::JsCast;

    // Build + materialize the demo under a real mount (mount_to initializes
    // the reactive owner chain; the any_spawner global executor is the
    // wasm-suite convention — the toggle_tests.rs mount_toggle precedent —
    // for the effects the port spawns).
    any_spawner::Executor::init_futures_executor();
    let _guard = mount_to({ container.clone() }, move || toggle_hero_demo());

    let button = container
        .query_selector("[data-toggle-hero] button")
        .expect("query")
        .expect("the hero demo's toggle button rendered");
    let html_element = button
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button element");

    // Uncontrolled start: not pressed, outline heart, aria-pressed=false.
    assert_eq!(
        html_element.get_attribute("aria-pressed").as_deref(),
        Some("false"),
        "uncontrolled Toggle must start aria-pressed=false"
    );
    assert!(
        !html_element.has_attribute("data-pressed"),
        "no data-pressed before the first click (the state mapping)"
    );
    assert!(
        button.inner_html().contains("fill-rule"),
        "the outline heart must render while unpressed"
    );

    // Click through the real handler: the merged bag's on_click machine
    // (a bubbling cancelable MouseEvent, per the crate suite's click()).
    // The click flips the pressed mirror, which re-runs the demo's build
    // Effect and mounts a fresh button — re-query after dispatch.
    {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_view(Some(&web_sys::window().expect("window")));
        let event =
            web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
                .expect("click event");
        html_element
            .dispatch_event(event.as_ref())
            .expect("dispatch click");
    }

    let pressed_button = container
        .query_selector("[data-toggle-hero] button")
        .ok()
        .flatten()
        .expect("the re-rendered hero button");
    let pressed_html = pressed_button
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button element");

    assert_eq!(
        pressed_html.get_attribute("aria-pressed").as_deref(),
        Some("true"),
        "after the click the machine commits pressed=true (setPressedState)"
    );
    assert!(
        pressed_html.has_attribute("data-pressed"),
        "data-pressed present after pressing (ToggleDataAttributes.pressed)"
    );
    assert!(
        pressed_html.inner_html().contains("fill-rule") == false,
        "the filled heart replaces the outline one when pressed"
    );
}
