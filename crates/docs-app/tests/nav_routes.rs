//! The side-nav → route drift guard, run in a test page of its own.
//!
//! WHY THIS IS NOT IN `src/render_test.rs` WITH THE OTHER docs-app TESTS: `wasm-bindgen-test`
//! runs every test of a crate in ONE shared page, and this guard mounts the real `App` once per
//! side-nav href — 18 mounts, including the AVATAR page. Mounting the avatar page is currently
//! unsafe to follow with anything else: its image probe's late callback reads a status mirror the
//! unmount has already disposed, and the resulting panic lands on whatever test happens to be
//! running when the callback fires. Measured 2026-09-15, guard enabled in the shared page vs
//! guard disabled:
//!
//! * guard disabled — 55 passed, 0 failed;
//! * guard enabled — the guard itself passes, but four tests it never asserts on go red:
//!   `checkbox_hero_demo_toggles_through_the_real_port` (the panic itself: "At
//!   crates/leptos-ui/src/avatar/image.rs:303:8, you tried to access a reactive value which was
//!   defined at crates/leptos-ui/src/avatar/image.rs:747:9, but it has already been disposed"),
//!   the two checkbox-group interaction tests (their clicks stop reaching the state machine), and
//!   `avatar_hero_demo_renders_the_real_root_composition` (the root renders `<!----><!---->` — no
//!   Image/Fallback at all).
//!
//! Neither of the two lazy answers is acceptable: deleting the guard loses the only check that the
//! sidebar and `App`'s route table agree, and disabling it (an early `return`) hides a real defect
//! behind a green suite. A separate target is a separate wasm binary, hence a separate page: the
//! guard keeps its full strength — it still drives the real `Router` to every href the sidebar
//! renders — and the leak it exposes can no longer be inherited by the tests that share the other
//! page. The underlying defect is recorded in the ledger as its own item (search TODO.md for
//! "avatar image probe") rather than being papered over here.

#![cfg(target_arch = "wasm32")]

use docs_app::App;
use docs_app::chrome::NAV_SECTIONS;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

// Without this the runner executes the target in Node, where there is no `window`, no DOM and no
// `history` — measured: `web_sys::window()` returns `None` and `pin_url` panics on its first line
// ("panicked at tests/nav_routes.rs:90:10: window"), because the runner only prints its
// "Running headless tests in Chrome on …" banner for a browser-configured target. Same
// declaration as the lib page (`src/render_test.rs:15`).
wasm_bindgen_test_configure!(run_in_browser);

/// One settled macrotask turn — `src/render_test.rs`'s `flush_one_turn`, kept identical so both
/// pages wait for the router's asynchronous location resolution the same way.
async fn flush_one_turn() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        web_sys::window()
            .expect("window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .expect("set_timeout");
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("timeout turn");
}

/// A fresh child of `body` — `src/render_test.rs`'s `fresh_container`, kept identical.
fn fresh_container(id: &str) -> web_sys::HtmlElement {
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id(id);
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");
    container
}

/// Every element matching `selector` inside `root` — `src/render_test.rs`'s `els`.
fn els(root: &web_sys::Element, selector: &str) -> Vec<web_sys::Element> {
    let list = root
        .query_selector_all(selector)
        .expect("query_selector_all");
    (0..list.length())
        .map(|index| {
            list.item(index)
                .expect("element at index")
                .dyn_into::<web_sys::Element>()
                .expect("element")
        })
        .collect()
}

/// Pins the test page's URL so the next `Router` mount resolves to `path`.
///
/// `replaceState`, NOT `pushState` + `popstate`: dispatching `popstate` in a shared test document
/// also notifies the listeners routers from earlier mounts left behind, and a listener whose
/// reactive owner is gone panics inside `RwSignal<Option<String>>::get`. `replaceState` fires no
/// event, and a router that mounts afterwards reads the URL at mount — the same state a navigation
/// would have produced. (Same reasoning as `src/render_test.rs`'s `pin_url`.)
fn pin_url(path: &str) {
    web_sys::window()
        .expect("window")
        .history()
        .expect("history")
        .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
        .expect("replace_state_with_url");
}

/// The drift guard for the nav tree: the sidebar is data (`chrome::NAV_SECTIONS`) while the routes
/// are declared separately in `App`, so this drives the real `Router` to every href the nav renders
/// and asserts the outlet renders that page rather than the router's fallback.
///
/// Each href gets its own mount with the URL pinned BEFORE it (see `pin_url`): navigating a mounted
/// router needs a `popstate` dispatch, which a router from an earlier mount would also receive.
#[wasm_bindgen_test]
async fn every_side_nav_href_resolves_to_a_real_page() {
    // The suite's convention for the `any_spawner` global executor (`src/render_test.rs`): the
    // mounted pages carry real component effects that spawn futures, and this page has no earlier
    // test to have initialized it. Without this the accordion page's `use_button` effect panics —
    // "tried to spawn a Future with Executor::spawn_local() before a global executor was
    // initialized" — and the chrome never renders, i.e. the guard would fail for a reason that is
    // about the harness, not about the nav tree.
    let _ = any_spawner::Executor::init_futures_executor();

    let items: Vec<(&'static str, &'static str)> = NAV_SECTIONS
        .iter()
        .flat_map(|section| section.items.iter())
        .map(|item| (item.title, item.href))
        .collect();
    assert_eq!(items.len(), 18, "14 components + 4 utils are ported");

    for (title, href) in items {
        pin_url(href);
        let container = fresh_container(&format!("test-nav-route-{}", href.replace('/', "-")));
        let _guard = leptos::mount::mount_to(container.clone(), App);

        // The router resolves its location asynchronously; wait for the shell before asserting.
        let mut mounted = false;
        for _ in 0..20 {
            flush_one_turn().await;
            if container.inner_html().contains("RootLayout") {
                mounted = true;
                break;
            }
        }
        assert!(mounted, "the chrome never rendered at {href}");

        let main = els(container.as_ref(), "main.ContentLayoutMain");
        assert_eq!(main.len(), 1, "the outlet's main is missing at {href}");
        let text = main[0].text_content().unwrap_or_default();
        assert!(
            !text.contains("Not found"),
            "{href} resolved to the router's fallback"
        );
        assert!(
            text.contains(title),
            "{href} did not render the {title} page; main was: {text:?}"
        );

        // The nav marked the page we are on — and only it.
        let active = els(container.as_ref(), "a.SideNavLink[data-active]");
        assert_eq!(active.len(), 1, "expected one active item at {href}");
        assert_eq!(active[0].get_attribute("href").as_deref(), Some(href));
    }
}
