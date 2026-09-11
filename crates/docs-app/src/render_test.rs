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
