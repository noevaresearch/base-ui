//! Wasm-browser render tests for the docs-app pages, per the Stage 3 prompt's
//! step 6 ("confirm the relevant route renders without panicking"). The
//! Playwright differential check (`ralph/scripts/playwright-diff.mjs`) does not
//! exist yet, so this in-browser render assertion is the available verification.
//!
//! Run with:
//! `CHROME=... CHROMEDRIVER=... cargo test -p docs-app --target wasm32-unknown-unknown`

#![cfg(target_arch = "wasm32")]

use docs_app::App;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn app_mounts_and_renders_the_shell() {
    // Mount the app for real in the test browser; App installs its own Router,
    // so a fresh document body gets the header shell at a minimum.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container");
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
        .expect("create container");
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
