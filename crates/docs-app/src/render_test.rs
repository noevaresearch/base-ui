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

/// Await one turn of the browser event loop (`setTimeout` 0) — the flush that
/// lets a reactive Effect (async first-run and rebuild) land before the test
/// re-reads the DOM. The docs-page demos rebuild inside `Effect::new` (the
/// React re-render analog), so any assertion about a REBUILT element must
/// wait out the executor's turn; direct machine mutations on the same node
/// (the crate suites' shape) don't need this.
async fn flush_one_turn() {
    use wasm_bindgen::JsValue;
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        web_sys::window()
            .expect("window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .expect("set_timeout");
    });
    wasm_bindgen_futures::JsFuture::from(promise).await;
    let _ = JsValue::UNDEFINED;
}

/// One real animation frame followed by a settled macrotask turn. Some completions are
/// *frame*-driven rather than timer-driven — the transition/animation completion that
/// unmounts a closed `Checkbox.Indicator` resolves on a requested frame
/// (`useAnimationsFinished`'s `await_frame` idiom, the checkbox crate's own
/// `settle_frames` helper), which `flush_one_turn`'s timer turn does not guarantee has
/// run. Note: this is the frame counterpart of `flush_one_turn`, added by the
/// `docs-content: components/checkbox` iteration; the field page's demo needed only the
/// timer turn.
async fn flush_one_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        web_sys::window()
            .expect("window")
            .request_animation_frame(&resolve)
            .expect("request_animation_frame");
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("animation frame");
    // Then the crate-local settle (the checkbox crate's `settle()`): drive the futures
    // executor, yield to leptos's own scheduler, drive it again, and give the browser a
    // real macrotask turn — the unmount effect is scheduled through both executors.
    for _ in 0..32 {
        any_spawner::Executor::poll_local();
    }
    leptos::task::tick().await;
    for _ in 0..32 {
        any_spawner::Executor::poll_local();
    }
    flush_one_turn().await;
    for _ in 0..32 {
        any_spawner::Executor::poll_local();
    }
}

#[wasm_bindgen_test]
async fn app_mounts_and_renders_the_shell() {
    // Mount the app for real in the test browser; App installs its own Router, and the
    // parent route renders the ported chrome (`crate::chrome::DocsLayout`) around the
    // outlet, so the shell is what a fresh document body gets at a minimum.
    //
    // The URL is pinned to a route this crate serves first: the shell lives INSIDE the parent
    // route, so an unmatched path renders the router's fallback ("Not found") with no chrome at
    // all — and this test browser's URL is whatever an earlier test left behind.
    pin_url("/");

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

    // The Router resolves its location asynchronously, so unlike the pre-chrome shell (which sat
    // OUTSIDE the router and rendered synchronously) the chrome arrives after a turn.
    let mut html = String::new();
    for _ in 0..20 {
        flush_one_turn().await;
        html = container.inner_html();
        if html.contains("RootLayout") {
            break;
        }
    }

    // Every layer of upstream's layout shell (`docs/src/app/(docs)/layout.tsx`), plus the
    // header (`Header.tsx`) and the side nav (`SideNav.tsx`) inside it.
    for expected in [
        "RootLayout",
        "RootLayoutContainer",
        "RootLayoutContent",
        "ContentLayoutRoot",
        "HeaderInner",
        "HeaderLogoLink",
        "SkipNav",
        "Skip to contents",
        "SideNavRoot",
        "SideNavHeading",
        "SideNavLink",
        "ContentLayoutMain",
        "main-content",
        "data-side-nav-viewport",
    ] {
        assert!(
            html.contains(expected),
            "the ported chrome is missing {expected}; html was: {html}"
        );
    }
    // The placeholder shell this item replaced (a `docs-title` h1 and a demo-areas div) is
    // gone: it rendered on every route and matched nothing upstream.
    assert!(
        !html.contains("docs-title") && !html.contains("Collapsible panel demo area"),
        "the old placeholder chrome still renders; html was: {html}"
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
        leptos::prelude::document()
            .get_element_by_id("test-mount-root-2")
            .is_some(),
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

    let _guard = mount_to({ container.clone() }, move || {
        view! {
            <CSPProviderView nonce=Some("test-nonce".to_string()) disable_style_elements=Some(false)>
                <CspProbe />
            </CSPProviderView>
            <CSPProviderView nonce=None disable_style_elements=Some(true)>
                <CspProbe />
            </CSPProviderView>
        }
    });

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
async fn toggle_page_renders_the_hero_demo_through_the_real_port() {
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
    // Effect and mounts a fresh button — await the effect's flush, then
    // re-query after dispatch.
    {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_view(Some(&web_sys::window().expect("window")));
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
            .expect("click event");
        html_element
            .dispatch_event(event.as_ref())
            .expect("dispatch click");
    }

    // The rebuild Effect flushes on a later microtask/turn — wait one turn
    // before re-reading the DOM for the replaced button.
    flush_one_turn().await;

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

#[wasm_bindgen_test]
async fn merge_props_page_renders_and_the_locked_toggle_prevents_the_base_ui_handler() {
    // The merge-props docs page's live demo is the upstream
    // DemoPreventBaseUIHandler (demos/prevent-base-ui-handler/css-modules/
    // index.tsx) ported onto the real toggle_element + merge_props
    // composition: a controlled Toggle whose consumer onClick (the
    // elementProps rest bag) calls prevent_base_ui_handler() while locked,
    // so the merged composition gates Toggle's own click machine — the
    // page's prevention claim, exercised end-to-end.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-merge-props");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::merge_props_page::{MergePropsPage, prevent_base_ui_handler_demo};
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    let _guard = mount_to({ container.clone() }, move || {
        view! { <MergePropsPage /> }
    });

    let html = container.inner_html();
    assert!(
        html.contains("mergeProps"),
        "page h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("How merging works"),
        "page sections did not render; html was: {html}"
    );

    // Demo initial state: pressed=true (filled heart), locked=true.
    let button = container
        .query_selector("[data-merge-props-toggle-slot] button")
        .expect("query")
        .expect("the demo's toggle button rendered");
    let html_element = button
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button element");
    assert_eq!(
        html_element.get_attribute("aria-pressed").as_deref(),
        Some("true"),
        "the demo starts pressed=true (useState(true))"
    );
    assert!(
        button.inner_html().contains("fill-rule") == false,
        "the filled heart renders while pressed"
    );
    let label = container
        .query_selector(".merge-props-demo-label")
        .expect("query label")
        .expect("label element");
    assert!(
        label
            .text_content()
            .unwrap_or_default()
            .contains("(locked)"),
        "the demo starts locked"
    );

    // Click 1 while locked: the consumer handler runs first and marks the
    // event; Toggle's machine is gated and must NOT flip. The mirror never
    // changes, so no rebuild — the same button must still read pressed=true.
    {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_view(Some(&web_sys::window().expect("window")));
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
            .expect("click event");
        html_element
            .dispatch_event(event.as_ref())
            .expect("dispatch click while locked");
    }
    assert_eq!(
        html_element.get_attribute("aria-pressed").as_deref(),
        Some("true"),
        "while locked, preventBaseUIHandler() must veto Toggle's machine — pressed stays true"
    );

    // Unlock: the Lock/Unlock button flips the demo's `locked` state (the
    // React setLocked(l => !l) analog). The rebuild re-seeds the machine from
    // the mirror (still pressed=true).
    let lock_button = container
        .query_selector("[data-merge-props-lock-button]")
        .expect("query lock button")
        .expect("lock button element");
    lock_button
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .expect("lock button as HtmlElement")
        .click();
    // `HtmlElement::click` is infallible in web-sys (returns `()`) — the
    // dispatch itself cannot fail; the label assertion below verifies it.
    // Effect timing: the rebuild effect lands on a later event-loop turn —
    // wait one turn out, then assert on the label and re-query the rebuilt
    // button for the click below.
    flush_one_turn().await;
    {
        let html = container.inner_html();
        assert!(
            html.contains("(unlocked)"),
            "unlocking flips the label; html after the unlock click was: {html}"
        );
    }

    // Click 2 while unlocked: no prevention, the machine runs and commits
    // pressed=false (the mirror flips, rebuilding the button).
    let unlocked_button = container
        .query_selector("[data-merge-props-toggle-slot] button")
        .ok()
        .flatten()
        .expect("the rebuilt demo button");
    {
        let init = web_sys::MouseEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(true);
        init.set_view(Some(&web_sys::window().expect("window")));
        let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
            .expect("click event");
        unlocked_button
            .dyn_into::<web_sys::HtmlElement>()
            .expect("button element")
            .dispatch_event(event.as_ref())
            .expect("dispatch click while unlocked");
    }
    // The mirror flip schedules the rebuild Effect — wait its turn out before
    // re-querying for the replaced button.
    flush_one_turn().await;
    assert_eq!(
        container
            .query_selector("[data-merge-props-toggle-slot] button")
            .ok()
            .flatten()
            .and_then(|b| b.dyn_into::<web_sys::HtmlElement>().ok())
            .and_then(|b| b.get_attribute("aria-pressed"))
            .as_deref(),
        Some("false"),
        "while unlocked, the click commits pressed=false (setPressed via onPressedChange)"
    );

    // Sanity: the standalone demo form also mounts (the demos.json entry's
    // public contract) — pressed=true, locked=true seeds.
    let probe = leptos::prelude::document()
        .create_element("div")
        .expect("create probe")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    probe.set_id("test-mount-root-merge-props-standalone");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&probe)
        .expect("append probe");
    let _guard2 = mount_to({ probe.clone() }, || prevent_base_ui_handler_demo());
    assert!(
        probe
            .query_selector("[data-merge-props-toggle-slot] button")
            .ok()
            .flatten()
            .is_some(),
        "the standalone demo form renders its toggle"
    );
}

#[wasm_bindgen_test]
fn separator_page_renders_the_hero_demo_through_the_real_port() {
    // The separator docs page's live demo is the upstream hero
    // (`docs/src/app/(docs)/react/components/separator/demos/hero/tailwind/index.tsx`,
    // the demos.json entry's `orientation` prop exercise) ported onto the real
    // `leptos_ui::separator_element`: a static nav row of six links split by a
    // vertical Separator. The demo is fully static (demos.json:
    // `stateManaged: "none"`), so the assertions pin the materialized DOM
    // contract end-to-end: the container, six anchors with the demo class and
    // href, and the Separator's role/aria-orientation/data-orientation
    // intrinsics plus the demo's className merge.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-separator");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::separator_page::separator_hero_demo;
    use leptos::mount::mount_to;

    any_spawner::Executor::init_futures_executor();
    let _guard = mount_to({ container.clone() }, move || separator_hero_demo());

    // The demo root: the `flex gap-4 text-nowrap` container (`:4`).
    let demo_root = container
        .query_selector("[data-separator-hero]")
        .expect("query")
        .expect("the hero demo root rendered");
    assert_eq!(
        demo_root.get_attribute("class").as_deref(),
        Some("flex gap-4 text-nowrap"),
        "the demo container carries the upstream flex class"
    );

    // Six anchors (`:5-14, :33-46`), each with href + the shared demo class.
    let anchors: Vec<web_sys::Element> = {
        let list = demo_root
            .query_selector_all("a[href='#']")
            .expect("query anchors");
        let mut out = Vec::new();
        for i in 0..list.length() {
            out.push(
                list.get(i)
                    .expect("anchor at index")
                    .dyn_into::<web_sys::Element>()
                    .expect("anchor element"),
            );
        }
        out
    };
    assert_eq!(
        anchors.len(),
        6,
        "the hero demo renders exactly the upstream six links"
    );
    let labels: Vec<String> = anchors
        .iter()
        .map(|a| a.text_content().unwrap_or_default())
        .collect();
    assert_eq!(
        labels,
        vec!["Home", "Pricing", "Blog", "Support", "Log in", "Sign up"],
        "the links render in the upstream source order"
    );

    // The Separator: the vertical orientation (`:31`) through the real port's
    // merge — role/aria-orientation from the intrinsic bag, data-orientation
    // from the DEFAULT state walk, the demo className merged on.
    let separator = demo_root
        .query_selector("[role='separator']")
        .expect("query")
        .expect("the Separator element rendered inside the demo");
    assert_eq!(
        separator.get_attribute("role").as_deref(),
        Some("separator"),
        "the port renders role=separator (the intrinsic bag)"
    );
    assert_eq!(
        separator.get_attribute("aria-orientation").as_deref(),
        Some("vertical"),
        "the demo's orientation=vertical prop is reflected to aria-orientation"
    );
    assert_eq!(
        separator.get_attribute("data-orientation").as_deref(),
        Some("vertical"),
        "the demo's orientation rides the DEFAULT state walk onto data-orientation"
    );
    assert_eq!(
        separator.get_attribute("class").as_deref(),
        Some("w-px bg-neutral-300 dark:bg-neutral-700"),
        "the demo className is merged onto the element (the mergeClassNames path)"
    );
    assert_eq!(
        separator.tag_name().to_uppercase(),
        "DIV",
        "the Separator renders as the default div (refInstanceof: HTMLDivElement)"
    );

    // Separator is a single part with no children of its own.
    assert!(
        separator.children().length() == 0,
        "the Separator element is a leaf (the no-props anatomy renders it bare)"
    );
}

#[wasm_bindgen_test]
fn separator_page_component_renders_the_full_page_structure() {
    // Mount the page component directly under an owner chain (the real mount
    // path for the route's view) and assert the mirrored page structure
    // rendered: h1, the subtitle, the Anatomy snippet, the API reference
    // prose, and the live hero demo.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-separator-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::separator_page::SeparatorPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _guard = mount_to({ container.clone() }, || {
        view! { <SeparatorPage /> }
    });

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Separator</h1>"),
        "page h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A separator element accessible to screen readers."),
        "page subtitle did not render; html was: {html}"
    );
    assert!(
        html.contains("Anatomy"),
        "the Anatomy section did not render; html was: {html}"
    );
    assert!(
        html.contains("@base-ui/react/separator"),
        "the Anatomy import snippet did not render; html was: {html}"
    );
    assert!(
        html.contains("API reference"),
        "the API reference section did not render; html was: {html}"
    );
    assert!(
        html.contains("role=\"separator\"") || html.contains("data-orientation=\"vertical\""),
        "the live hero demo did not render; html was: {html}"
    );
}

#[wasm_bindgen_test]
async fn view_dynamic_children_update_reactively_in_the_harness() {
    // The controlled experiment for the reactive-update path the merge-props
    // test exercises: a `view!` dynamic child closure reading a signal must
    // re-run when the signal flips (after one executor turn). If this fails,
    // the break is in the harness/app's reactive wiring, not the demo.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-reactive-probe");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let value = RwSignal::new(1i32);
    let value_for_view = value;
    let _guard = mount_to({ container.clone() }, move || {
        view! { <p data-reactive-probe>{move || value_for_view.get().to_string()}</p> }
    });

    let probe = container
        .query_selector("[data-reactive-probe]")
        .expect("query")
        .expect("probe rendered");
    assert_eq!(
        probe.text_content().as_deref(),
        Some("1"),
        "the dynamic child's initial evaluation reads the signal"
    );

    value.set(2);
    flush_one_turn().await;

    let probe = container
        .query_selector("[data-reactive-probe]")
        .expect("query")
        .expect("probe still present");
    assert_eq!(
        probe.text_content().as_deref(),
        Some("2"),
        "the dynamic child re-ran after the signal flip (the reactive subscription)"
    );

    // Phase 2: the DELEGATED on:click path — the merge-props lock button uses
    // a leptos `on:click` (event delegation), not a native listener. Click it
    // and assert the handler ran (the signal flipped -> label updated).
    let value_for_click = value;
    let flip = move |_| value_for_click.update(|v| *v = *v + 1);
    // Rebuild the mount content with the click button (a fresh mount keeps
    // the probe simple; the same document/session).
    let click_container = leptos::prelude::document()
        .create_element("div")
        .expect("create click container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    click_container.set_id("test-mount-root-click-probe");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&click_container)
        .expect("append click container");

    let value_in_view = value;
    let _guard2 = mount_to({ click_container.clone() }, move || {
        view! {
            <div>
                <em data-click-probe>{move || value_in_view.get().to_string()}</em>
                <button data-click-btn on:click=flip>"bump"</button>
            </div>
        }
    });

    let btn = click_container
        .query_selector("[data-click-btn]")
        .expect("query")
        .expect("click button rendered")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button as HtmlElement");
    btn.click();
    flush_one_turn().await;

    let readout = click_container
        .query_selector("[data-click-probe]")
        .expect("query")
        .expect("click probe present");
    assert_eq!(
        readout.text_content().as_deref(),
        Some("3"),
        "the delegated on:click handler ran and the dynamic child re-rendered — \
         if this fails, delegated events are dead in this harness and the \
         merge-props test must dispatch a bubbling native event instead"
    );
}

#[wasm_bindgen_test]
async fn nested_view_child_closures_stay_tracked() {
    // The SECOND controlled experiment: the merge-props demo's closures live
    // in a view! nested INSIDE another view! (the page calls the demo
    // function as a child expression, not a component). If a signal-backed
    // closure in the nested view stops tracking (the runtime warning shape:
    // "outside a reactive tracking context"), this reproduces it — the fix
    // belongs at the demo/page structure level, not the test.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-nested-probe");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    fn nested_demo(flag: RwSignal<bool>) -> impl IntoView {
        let flag_for_closure = flag;
        view! {
            <span data-nested-probe>
                {move || if flag_for_closure.get() { "on" } else { "off" }}
            </span>
        }
    }

    // The flip handle, passed INTO the nested view (the demo-function shape:
    // signal created by the caller, closure inside the nested view!).
    let flag = RwSignal::new(false);
    let flag_mirror = RwSignal::new(false);
    let flag_mirror_for_view = flag_mirror;
    let nested_flag = flag.clone();
    let _guard = mount_to({ container.clone() }, move || {
        view! {
            <div>
                {nested_demo(nested_flag)}
                <em data-mirror-probe>
                    {move || if flag_mirror_for_view.get() { "M-on" } else { "M-off" }}
                </em>
            </div>
        }
    });

    // The nested span's closure is the exact shape under test.
    let nested = container
        .query_selector("[data-nested-probe]")
        .expect("query")
        .expect("nested probe rendered");
    assert_eq!(
        nested.text_content().as_deref(),
        Some("off"),
        "the nested-view dynamic child's initial evaluation"
    );

    flag.set(true);
    flush_one_turn().await;

    let nested = container
        .query_selector("[data-nested-probe]")
        .expect("query")
        .expect("nested probe still present");
    assert_eq!(
        nested.text_content().as_deref(),
        Some("on"),
        "the NESTED-view dynamic child re-ran after the signal flip — if this fails, \
         closures inside a child-expression view lose tracking and the merge-props \
         demo needs a structural fix (component boundary), not a test change"
    );

    // Phase 2: THE STATIC-SIBLING HYPOTHESIS — the merge-props label is
    // `"Favorite " {move || ...}` (a static text node followed by a dynamic
    // child in the SAME element). If a dynamic child after a static sibling
    // loses its reactive update, this reproduces it; the fix is to wrap the
    // dynamic text in its own element.
    let label_flag = RwSignal::new(false);
    let label_flag_for_view = label_flag;
    let label_container = leptos::prelude::document()
        .create_element("div")
        .expect("create label container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    label_container.set_id("test-mount-root-static-sibling-probe");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&label_container)
        .expect("append label container");
    let _guard3 = mount_to({ label_container.clone() }, move || {
        view! {
            <span data-static-sibling-probe>
                "static "
                {move || if label_flag_for_view.get() { "on" } else { "off" }}
            </span>
        }
    });
    let probe = label_container
        .query_selector("[data-static-sibling-probe]")
        .expect("query")
        .expect("static-sibling probe rendered");
    assert_eq!(
        probe.text_content().as_deref(),
        Some("static off"),
        "the mixed static+dynamic children's initial evaluation"
    );
    label_flag.set(true);
    flush_one_turn().await;
    let probe = label_container
        .query_selector("[data-static-sibling-probe]")
        .expect("query")
        .expect("static-sibling probe still present");
    assert_eq!(
        probe.text_content().as_deref(),
        Some("static on"),
        "the dynamic child AFTER a static text node re-ran after the flip — if this \
         fails, the merge-props label's dynamic text must be wrapped in its own \
         element (the React text-interpolation shape has no direct Leptos \
         equivalent at this nesting)"
    );

    // The nested demo's signal is internal, so drive the MIRROR closure (same
    // nesting depth, same closure shape) and read back through it.
    let mirror = container
        .query_selector("[data-mirror-probe]")
        .expect("query")
        .expect("mirror probe rendered");
    assert_eq!(
        mirror.text_content().as_deref(),
        Some("M-off"),
        "the nested-view dynamic child's initial evaluation"
    );

    flag_mirror.set(true);
    flush_one_turn().await;

    let mirror = container
        .query_selector("[data-mirror-probe]")
        .expect("query")
        .expect("mirror probe still present");
    assert_eq!(
        mirror.text_content().as_deref(),
        Some("M-on"),
        "the nested-view dynamic child re-ran after the signal flip — if this fails, \
         nested-view closures lose tracking and the merge-props demo needs a \
         structural fix (component boundary), not a test change"
    );
}

#[wasm_bindgen_test]
async fn direction_provider_page_renders_probes_through_the_real_provider_and_hook() {
    // The direction-provider docs page's live machinery mirrors the upstream hero's
    // observable contract (`demos/hero/tailwind/index.tsx:1-21`) on the real ported
    // implementation: `DirectionProviderView` publishes through
    // `leptos_ui_internals::provide_direction_context` under a real reactive-graph
    // owner, and the child `DirectionProbe` reads it back through the real
    // `use_direction` hook. Mounting the rtl demo and a bare probe pins the
    // end-to-end context flow: the rtl provider's probe reads `direction: rtl`,
    // the bare probe outside any provider reads the fallback `direction: ltr`
    // (`DirectionContext.tsx:15`), and the demo's outer div carries the native
    // `dir="rtl"` attribute — the docs caveat that DirectionProvider does not
    // affect HTML and CSS (`page.mdx:26`).
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-direction");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::direction_provider_page::{DirectionProbe, DirectionProviderRtlDemo};
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _guard = mount_to({ container.clone() }, move || {
        view! {
            <DirectionProviderRtlDemo />
            <DirectionProbe />
        }
    });

    let html = container.inner_html();
    assert!(
        html.contains("docs-direction-probe"),
        "no probe rendered; html was: {html}"
    );

    let probes: Vec<web_sys::Element> = {
        let list = container
            .query_selector_all(".docs-direction-probe")
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
        "the demo's probe plus the bare probe should have rendered two probes; html was: {html}"
    );

    let rtl = probes[0].text_content().unwrap_or_default();
    assert!(
        rtl.contains("direction: rtl"),
        "the rtl provider's probe did not read the provided direction; it read: {rtl}"
    );

    let bare = probes[1].text_content().unwrap_or_default();
    assert!(
        bare.contains("direction: ltr"),
        "the no-provider probe did not read the 'ltr' fallback — context leaked across the provider boundary (the Owner::set thread-local overwrite); it read: {bare}"
    );

    let demo_div = container
        .query_selector(".docs-direction-demo")
        .expect("query")
        .expect("the demo's outer div rendered");
    assert_eq!(
        demo_div.get_attribute("dir").as_deref(),
        Some("rtl"),
        "the demo's outer div must carry the native dir attribute the provider does not set"
    );
}

#[wasm_bindgen_test]
fn direction_provider_route_renders_without_panicking() {
    // Mount the page component directly under an owner chain (the real mount
    // path for the route's view) and assert the mirrored page structure
    // rendered: h1, subtitle, the Anatomy section, and the live demo.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-direction-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::direction_provider_page::DirectionProviderPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _guard = mount_to({ container.clone() }, || {
        view! { <DirectionProviderPage /> }
    });

    let html = container.inner_html();
    assert!(
        html.contains("Direction Provider"),
        "page h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("Enables RTL behavior for Base UI components."),
        "page subtitle did not render; html was: {html}"
    );
    assert!(
        html.contains("Anatomy"),
        "page sections did not render; html was: {html}"
    );
    assert!(
        html.contains("direction: rtl"),
        "the live provider demo did not render; html was: {html}"
    );
}

// ---------------------------------------------------------------------------
// Meter docs page (`docs-content: components/meter`)
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn meter_hero_demo_renders_the_real_part_tree_at_value_24() {
    // The upstream hero's observable DOM contract
    // (`demos/hero/tailwind/index.tsx:3-14`, demos.json entry 1) on the real
    // ported parts: role="meter" with the full ARIA tuple derived from
    // value=24 between the 0/100 defaults (aria-valuenow=24, the percent
    // aria-valuetext "24%" from the port's formatNumber pipeline, the
    // aria-labelledby link the Label registration fills), the Label's
    // role="presentation" span carrying "Storage Used", the Track div, and
    // the Indicator's inline width: 24% fill inside it.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-meter-demo");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::meter_page::MeterHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || {
        view! { <MeterHeroDemo /> }
    }));

    let html = container.inner_html();
    assert!(
        html.contains("Storage Used"),
        "the hero demo did not render; html was: {html}"
    );

    // The root: role=meter, the ARIA tuple, and the demo class verbatim.
    let root = container
        .query_selector("[role='meter']")
        .expect("query")
        .expect("the meter root rendered");
    assert_eq!(root.get_attribute("role").as_deref(), Some("meter"));
    assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("24"));
    assert_eq!(
        root.get_attribute("aria-valuemin").as_deref(),
        Some("0"),
        "the min default reaches the ARIA surface"
    );
    assert_eq!(
        root.get_attribute("aria-valuemax").as_deref(),
        Some("100"),
        "the max default reaches the ARIA surface"
    );
    assert_eq!(
        root.get_attribute("aria-valuetext").as_deref(),
        Some("24%"),
        "the default percent aria-valuetext from the format pipeline"
    );
    assert!(
        root.get_attribute("class")
            .expect("root class")
            .contains("grid-cols-2"),
        "the upstream demo className rides the real root"
    );

    // The label association: aria-labelledby points at the rendered Label,
    // whose registration filled the root's signal (behavior.md
    // "Accessibility").
    let labelledby = root
        .get_attribute("aria-labelledby")
        .expect("the root's aria-labelledby is set by the Label's registration");
    let label = container
        .query_selector(&format!("#{labelledby}"))
        .expect("query")
        .unwrap_or_else(|| panic!("no element under #{labelledby}; html was: {html}"));
    assert_eq!(label.get_attribute("role").as_deref(), Some("presentation"));
    assert_eq!(label.text_content().as_deref(), Some("Storage Used"));

    // The Track + Indicator: the fill's inline width is the context-derived
    // 24% (the part's own style, not demo machinery).
    let indicator = container
        .query_selector("[role='meter'] > div > div")
        .or_else(|_| container.query_selector("div[style*='width']"))
        .expect("query")
        .expect("the indicator rendered");
    let style = indicator.get_attribute("style").unwrap_or_default();
    assert!(
        style.contains("width: 24%"),
        "the indicator fill must carry the derived 24% width; style was: {style}"
    );

    // The Value: aria-hidden text with the formatted default "24%".
    let value = container
        .query_selector("span[aria-hidden='true']")
        .expect("query")
        .expect("the value span rendered");
    assert_eq!(value.text_content().as_deref(), Some("24%"));
}

#[wasm_bindgen_test]
fn meter_page_route_renders_the_mirrored_structure() {
    // Mount the page component directly (the real mount path for the route's
    // view) and assert the mirrored page structure: the h1, the Subtitle
    // line, the Anatomy snippet, all five API-reference part headings in
    // document order, both hero-demo slots, and the single demo slot.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-meter-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::meter_page::MeterPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || {
        view! { <MeterPage /> }
    }));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Meter</h1>"),
        "page h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A graphical display of a numeric value within a range."),
        "page subtitle did not render; html was: {html}"
    );
    for heading in [
        "Anatomy",
        "API reference",
        "Root",
        "Track",
        "Indicator",
        "Value",
        "Label",
    ] {
        assert!(
            html.contains(&format!(">{heading}</")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    assert!(
        html.contains("@base-ui/react/meter"),
        "the Anatomy import snippet did not render"
    );
    // The page's single (hero) demo mounted on the real parts: the meter
    // role, the label text, and the derived percent value all present.
    assert!(
        container
            .query_selector("[role='meter']")
            .expect("query")
            .is_some(),
        "the hero demo slot did not render the real meter; html was: {html}"
    );
    assert!(html.contains("Storage Used"));
    assert!(html.contains("24%"));
    assert_eq!(
        container
            .query_selector_all("[data-demo]")
            .expect("query")
            .length(),
        1,
        "the page mirrors upstream's single demo slot"
    );
}

// ---------------------------------------------------------------------------
// docs-content: components/accordion — the page's three demos on the real
// leptos_ui::accordion port (specs/docs-content/accordion/demos.json).
// ---------------------------------------------------------------------------

/// All `<button>`s in mount order — the three/four demo triggers.
fn buttons_of(container: &web_sys::HtmlElement) -> Vec<web_sys::HtmlButtonElement> {
    let list = container
        .query_selector_all("button")
        .expect("query buttons");
    let mut out = Vec::new();
    for i in 0..list.length() {
        out.push(
            list.get(i)
                .expect("button at index")
                .dyn_into::<web_sys::HtmlButtonElement>()
                .expect("button element"),
        );
    }
    out
}

/// All open/closed panel regions (`[role=region]`) in mount order.
fn regions_of(container: &web_sys::HtmlElement) -> Vec<web_sys::Element> {
    let list = container
        .query_selector_all("[role='region']")
        .expect("query regions");
    let mut out = Vec::new();
    for i in 0..list.length() {
        out.push(
            list.get(i)
                .expect("region at index")
                .dyn_into::<web_sys::Element>()
                .expect("region element"),
        );
    }
    out
}

fn click_button(button: &web_sys::HtmlButtonElement) {
    let init = web_sys::MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
    button
        .dispatch_event(event.dyn_ref::<web_sys::Event>().unwrap())
        .expect("dispatch click");
}

#[wasm_bindgen_test]
async fn accordion_hero_demo_toggles_through_the_real_port() {
    // demos.json entry "hero": default single-open accordion, all panels
    // initially closed (stateManaged: uncontrolled, Root propsExercised: []).
    // The demo classes ride the REAL parts' class props; the open/close
    // machine is the port's root value algebra.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-accordion-hero");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::accordion_page::AccordionHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    // The test realm is SHARED across the suite: the mounted DOM stays in the
    // document after the test ends, so the reactive owner backing it must stay
    // alive too — dropping the mount guard disposes the owner while a queued
    // rebuild effect (from the last flush) still references it, and the next
    // test's executor turn panics on the disposed read. Same trade the page
    // code makes for its owner bridges (std::mem::forget).
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <AccordionHeroDemo /> },
    ));

    // Initial: three triggers closed, zero panels in the DOM (closed
    // non-keepMounted panels unmount — behavior.md "State model").
    let buttons = buttons_of(&container);
    assert_eq!(buttons.len(), 3, "the hero renders three triggers");
    for b in &buttons {
        assert_eq!(
            b.get_attribute("aria-expanded").as_deref(),
            Some("false"),
            "every trigger starts closed"
        );
        assert!(
            b.get_attribute("data-panel-open").is_none(),
            "no data-panel-open while closed"
        );
    }
    assert_eq!(
        regions_of(&container).len(),
        0,
        "closed non-keepMounted panels are absent from the DOM"
    );

    // The demo classes ride the real parts: root, trigger 1, and the first
    // item carry the upstream Tailwind strings (item 1 without border-t).
    let root = container
        .query_selector("[class*='max-w-80']")
        .expect("query")
        .expect("the demo root rendered");
    assert!(
        root.get_attribute("class")
            .as_deref()
            .unwrap_or_default()
            .starts_with("flex w-full max-w-80 flex-col"),
        "the demo root carries the upstream class"
    );
    assert!(
        buttons[0]
            .get_attribute("class")
            .as_deref()
            .unwrap_or_default()
            .contains("group flex w-full items-center justify-between"),
        "the trigger carries the upstream group class"
    );

    // Open item 1: aria-expanded flips, data-panel-open appears, the panel
    // mounts with the hero answer, aria-controls resolves.
    click_button(&buttons[0]);
    flush_one_turn().await;

    let buttons = buttons_of(&container);
    let regions = regions_of(&container);
    assert_eq!(regions.len(), 1, "exactly one panel is open");
    assert_eq!(
        buttons[0].get_attribute("aria-expanded").as_deref(),
        Some("true"),
        "the activated trigger reports aria-expanded=true"
    );
    assert_eq!(
        buttons[0].get_attribute("data-panel-open").as_deref(),
        Some("true"),
        "the activated trigger carries data-panel-open"
    );
    let region = &regions[0];
    assert!(
        region.get_attribute("data-open").is_some(),
        "the open panel carries data-open"
    );
    assert!(
        region
            .text_content()
            .unwrap_or_default()
            .contains("high-quality unstyled React components"),
        "the open panel shows the first FAQ answer"
    );
    let controls = buttons[0]
        .get_attribute("aria-controls")
        .expect("aria-controls resolves while open");
    assert_eq!(
        region.get_attribute("id").as_deref(),
        Some(controls.as_str()),
        "aria-controls points at the open panel"
    );

    // Single-open: activating item 2 closes item 1 (the root's non-multiple
    // algebra — accordion_next_value's identity toggle).
    click_button(&buttons[1]);
    flush_one_turn().await;

    let buttons = buttons_of(&container);
    let regions = regions_of(&container);
    assert_eq!(regions.len(), 1, "still exactly one panel open");
    assert_eq!(
        buttons[0].get_attribute("aria-expanded").as_deref(),
        Some("false"),
        "item 1 closed when item 2 opened (single-open)"
    );
    assert_eq!(
        buttons[1].get_attribute("aria-expanded").as_deref(),
        Some("true"),
        "item 2 is the open one"
    );
    assert!(
        regions[0]
            .text_content()
            .unwrap_or_default()
            .contains("Quick start"),
        "the open panel is item 2's"
    );

    // Close item 2: back to zero panels.
    click_button(&buttons[1]);
    flush_one_turn().await;
    assert_eq!(
        regions_of(&container).len(),
        0,
        "the second activation closes the panel"
    );
}

#[wasm_bindgen_test]
async fn accordion_multiple_demo_keeps_independent_panels_open() {
    // demos.json entry "multiple": Root multiple=true — several panels open at
    // once, toggling one does not close the others.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-accordion-multiple");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::accordion_page::MultipleDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    // See the hero test: the shared-realm owner must outlive the test.
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <MultipleDemo /> },
    ));

    let buttons = buttons_of(&container);
    assert_eq!(buttons.len(), 3);
    click_button(&buttons[0]);
    flush_one_turn().await;
    click_button(&buttons[1]);
    flush_one_turn().await;

    let buttons = buttons_of(&container);
    let regions = regions_of(&container);
    assert_eq!(regions.len(), 2, "two panels are open simultaneously");
    assert_eq!(
        buttons[0].get_attribute("aria-expanded").as_deref(),
        Some("true"),
        "item 1 stayed open through item 2's activation (multiple)"
    );
    assert_eq!(
        buttons[1].get_attribute("aria-expanded").as_deref(),
        Some("true")
    );

    // Closing item 1 leaves item 2 open — the multiple append/filter algebra.
    click_button(&buttons[0]);
    flush_one_turn().await;

    let buttons = buttons_of(&container);
    let regions = regions_of(&container);
    assert_eq!(regions.len(), 1);
    assert_eq!(
        buttons[0].get_attribute("aria-expanded").as_deref(),
        Some("false")
    );
    assert_eq!(
        buttons[1].get_attribute("aria-expanded").as_deref(),
        Some("true"),
        "item 2 is unaffected by item 1's close"
    );
}

#[wasm_bindgen_test]
async fn accordion_hidden_until_found_demo_keeps_closed_panels_mounted() {
    // demos.json entry "hidden-until-found": Root hiddenUntilFound=true —
    // closed panels stay in the DOM with hidden="until-found" (the port's
    // panel mount gate + hidden-attribute walk).
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-accordion-huf");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::accordion_page::HiddenUntilFoundDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    // See the hero test: the shared-realm owner must outlive the test.
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <HiddenUntilFoundDemo /> },
    ));

    // Initially: all four panels stay MOUNTED, hidden="until-found", closed.
    let buttons = buttons_of(&container);
    assert_eq!(buttons.len(), 4, "the shipping FAQ renders four items");
    let regions = regions_of(&container);
    assert_eq!(
        regions.len(),
        4,
        "hiddenUntilFound keeps every closed panel in the DOM"
    );
    for r in &regions {
        assert_eq!(
            r.get_attribute("hidden").as_deref(),
            Some("until-found"),
            "a closed panel hides with until-found"
        );
        assert!(r.get_attribute("data-open").is_none());
    }

    // Opening the return-policy item removes its hidden attribute and shows
    // the "restocking fee" content (the find-in-page target text).
    click_button(&buttons[1]);
    flush_one_turn().await;

    let regions = regions_of(&container);
    assert_eq!(regions.len(), 4, "the panels never unmount");
    let open_panel = &regions[1];
    assert!(
        open_panel.get_attribute("hidden").is_none(),
        "the opened panel drops the hidden attribute"
    );
    assert!(open_panel.get_attribute("data-open").is_some());
    assert!(
        open_panel
            .text_content()
            .unwrap_or_default()
            .contains("restocking fee"),
        "the open panel shows the restocking answer"
    );
    // The other three stay closed under until-found.
    for (i, r) in regions.iter().enumerate() {
        if i != 1 {
            assert_eq!(
                r.get_attribute("hidden").as_deref(),
                Some("until-found"),
                "panel {i} stays until-found-hidden"
            );
        }
    }
}

#[wasm_bindgen_test]
fn accordion_page_component_renders_the_full_page_structure() {
    // The whole page: H1 + Subtitle, hero demo before the first heading,
    // Anatomy snippet, both Examples subsections with their demos, and the
    // five-part API reference — mirroring page.mdx's document order per
    // specs/docs-content/accordion/page.md.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-accordion-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::accordion_page::AccordionPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    // The page mounts three accordion demos; `<Show>`-driven panel gates spawn
    // effects whose executor must exist before the first mount (the test order
    // is not fixed — this test must not depend on a prior test having init'd).
    let _ = any_spawner::Executor::init_futures_executor();
    // See the hero test: the shared-realm owner must outlive the test.
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <AccordionPage /> },
    ));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Accordion</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A set of collapsible panels with headings."),
        "the subtitle did not render"
    );
    // The hero demo mounts before the first h2 (three triggers on the page
    // from the hero alone; the full page has 3+3+4=10 across the three demos).
    assert_eq!(
        buttons_of(&container).len(),
        10,
        "the page renders all three demos (3 + 3 + 4 triggers)"
    );
    for demo in ["hero", "multiple", "hidden-until-found"] {
        assert!(
            container
                .query_selector(&format!("[data-demo='{demo}']"))
                .expect("query")
                .is_some(),
            "the {demo} demo slot did not render"
        );
    }
    assert!(
        html.contains("@base-ui/react/accordion"),
        "the Anatomy snippet did not render"
    );
    for heading in [
        "Anatomy",
        "Examples",
        "Open multiple panels",
        "Hidden until found",
        "API reference",
        "Root",
        "Item",
        "Header",
        "Trigger",
        "Panel",
    ] {
        assert!(
            html.contains(&format!(">{heading}</")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
}

// ---------------------------------------------------------------------------
// docs-content: components/button — the page's two demos on the real
// leptos_ui::button_element port (specs/docs-content/button/demos.json).
// ---------------------------------------------------------------------------

/// The single `<button>` under the given container.
fn button_in(container: &web_sys::Element) -> web_sys::HtmlElement {
    container
        .query_selector("button")
        .expect("query button")
        .expect("the demo button rendered")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("button element")
}

fn click_with_bubbles(element: &web_sys::HtmlElement) {
    let init = web_sys::MouseEventInit::new();
    init.set_bubbles(true);
    init.set_cancelable(true);
    let event = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init).unwrap();
    element
        .dispatch_event(event.dyn_ref::<web_sys::Event>().unwrap())
        .expect("dispatch click");
}

#[wasm_bindgen_test]
fn button_hero_demo_renders_the_static_native_button() {
    // demos.json entry "hero": a native <button> labeled "Submit" with only
    // className exercised (propsExercised.Button: ["className"],
    // stateManaged: "none"). The assertions pin the materialized DOM
    // contract: tag BUTTON, the form-submit default overridden to
    // type="button" (behavior.md "DOM structure", the render engine's forced
    // default), the upstream class string verbatim, the label, and none of
    // the state furniture the demo never passes.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-button-hero");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::button_page::button_hero_demo;
    use leptos::mount::mount_to;

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || button_hero_demo()));

    let button = button_in(&container);
    assert_eq!(
        button.tag_name(),
        "BUTTON",
        "the hero renders the native <button> root"
    );
    let native = button
        .clone()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("native button");
    assert_eq!(
        native.type_(),
        "button",
        "the form-submit default is overridden by the render engine (useRenderElement.tsx:232-240)"
    );
    assert_eq!(
        button.get_attribute("class").as_deref(),
        Some(
            "flex h-8 items-center justify-center gap-2 rounded-none border border-neutral-950 bg-white px-3 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 active:not-data-disabled:bg-neutral-200 focus-visible:outline-2 focus-visible:-outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white data-disabled:border-neutral-500 data-disabled:text-neutral-500 disabled:border-neutral-500 disabled:text-neutral-500 dark:border-white dark:bg-neutral-950 dark:text-white dark:hover:not-data-disabled:bg-neutral-800 dark:active:not-data-disabled:bg-neutral-700 dark:data-disabled:border-neutral-400 dark:data-disabled:text-neutral-400"
        ),
        "the upstream className is carried verbatim"
    );
    assert_eq!(
        button.text_content().as_deref(),
        Some("Submit"),
        "the hero's label"
    );
    assert!(
        !button.has_attribute("disabled"),
        "the hero button is enabled"
    );
    assert!(
        !button.has_attribute("data-disabled"),
        "no data-disabled while enabled"
    );
    assert!(
        button.get_attribute("aria-labelledby").is_none(),
        "the hero carries no aria-labelledby (only className is exercised)"
    );
}

#[wasm_bindgen_test]
async fn button_loading_demo_runs_the_full_state_cycle_through_the_real_port() {
    // demos.json entry "loading": click while enabled → disabled +
    // focusableWhenDisabled + label "Submitting" (aria-labelledby at the
    // inner span); the shortened timeout re-enables with the label back to
    // "Submit". The disabled-phase click must be swallowed by the internal
    // guard — the consumer onClick rides the real merged bag and the guard
    // runs before it (behavior.md "Events") — which is exactly why the
    // mirror does not flip again and the cycle terminates.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-button-loading");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::button_page::ButtonLoadingDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    // The component form the page mounts: the loading demo lives inside a
    // real mount so its rebuild Effect subscribes under the mount's owner.
    let _guard = mount_to({ container.clone() }, || {
        view! {
            <ButtonLoadingDemo reset_ms=250 />
        }
    });

    // The rebuild Effect's first run is deferred to the executor: it is what
    // subscribes the effect to the loading mirror, and a click that lands
    // before it races the subscription. Settle one turn before interacting
    // (the initial DOM itself comes from the synchronous seed build — the
    // assert below pins that).
    flush_one_turn().await;
    let button = button_in(&container);
    assert_eq!(button.text_content().as_deref(), Some("Submit"));
    assert!(!button.has_attribute("disabled"));
    assert!(!button.has_attribute("data-disabled"));
    // upstream sets `aria-disabled = disabled` unconditionally for the
    // native + focusableWhenDisabled combination
    // (useFocusableWhenDisabled.ts:38-44), and React stringifies aria-*:
    // the enabled button carries aria-disabled="false".
    assert_eq!(
        button.get_attribute("aria-disabled").as_deref(),
        Some("false"),
        "the enabled focusableWhenDisabled button signals aria-disabled=false (the upstream conditional)"
    );
    let labelledby = button
        .get_attribute("aria-labelledby")
        .expect("the demo sets aria-labelledby");
    assert!(
        labelledby.starts_with("base-ui-"),
        "the labelId comes from the real useBaseUiId generator; got: {labelledby}"
    );
    let span = container
        .query_selector(&format!("span#{labelledby}"))
        .expect("query label span")
        .expect("the label span exists");
    assert_eq!(span.text_content().as_deref(), Some("Submit"));

    // Click while enabled: loading=true → rebuild mounts the disabled,
    // focusable-when-disabled button labeled "Submitting".
    click_with_bubbles(&button);
    flush_one_turn().await;

    let loading_button = button_in(&container);
    assert_eq!(
        loading_button.text_content().as_deref(),
        Some("Submitting"),
        "the label swapped with the loading state"
    );
    // focusableWhenDisabled's whole contract: the NATIVE disabled attribute
    // is NOT set (it would block focus) — useFocusableWhenDisabled.ts:43-47
    // sets `disabled` only when `!focusableWhenDisabled` — and the disabled
    // signal rides aria-disabled="true" + data-disabled instead (the port's
    // use_button wasm test focusable_when_disabled_native_button_uses_aria_
    // disabled_and_stays_focusable pins the same).
    let native = loading_button
        .clone()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("native button");
    assert!(
        !native.disabled(),
        "no native disabled attribute under focusableWhenDisabled (it would block focus)"
    );
    assert_eq!(
        loading_button.get_attribute("aria-disabled").as_deref(),
        Some("true"),
        "the disabled signal rides aria-disabled (useFocusableWhenDisabled.ts:38-44)"
    );
    assert_eq!(
        loading_button.get_attribute("data-disabled").as_deref(),
        Some(""),
        "data-disabled present (the state mapping)"
    );
    assert_eq!(
        loading_button.get_attribute("tabindex").as_deref(),
        Some("0"),
        "focusableWhenDisabled keeps the disabled button keyboard-focusable"
    );
    assert_eq!(
        loading_button.get_attribute("aria-labelledby").as_deref(),
        Some(labelledby.as_str()),
        "the labelId is stable across the rebuild (the React useId contract)"
    );
    let loading_span = container
        .query_selector(&format!("span#{labelledby}"))
        .expect("query label span")
        .expect("the label span exists while loading");
    assert_eq!(loading_span.text_content().as_deref(), Some("Submitting"));

    // Click while disabled: the consumer handler must NOT run (the internal
    // guard runs first) — the mirror stays true and no second reset stacks.
    click_with_bubbles(&loading_button);
    flush_one_turn().await;
    assert_eq!(
        button_in(&container).text_content().as_deref(),
        Some("Submitting"),
        "the disabled-phase click was swallowed by the guard (no re-entry)"
    );

    // The reset timer fires: loading=false → re-enabled "Submit". The 250 ms
    // timer is a real macrotask, so POLL for the flip with generous retries
    // instead of a fixed wait — a fixed wait races the debug build's
    // turn latency (the wasm suite caught that: the re-enable landed
    // nondeterministically before/after a single fixed sleep).
    let mut re_enabled = false;
    for _ in 0..40 {
        flush_one_turn().await;
        {
            let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                web_sys::window()
                    .expect("window")
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50)
                    .expect("set_timeout");
            });
            wasm_bindgen_futures::JsFuture::from(promise).await;
        }
        if button_in(&container).text_content().as_deref() == Some("Submit") {
            re_enabled = true;
            break;
        }
    }
    assert!(
        re_enabled,
        "the timeout re-enabled the button with the original label"
    );
    // The re-enable is a full state-walk rerun: the disabled signal rides it.
    let reenabled = button_in(&container);
    assert_eq!(
        reenabled.get_attribute("aria-disabled").as_deref(),
        Some("false"),
        "aria-disabled returns to false after the reset"
    );
    assert!(
        !reenabled.has_attribute("data-disabled"),
        "data-disabled cleared after the reset"
    );
}

#[wasm_bindgen_test]
fn button_page_component_renders_the_full_page_structure() {
    // The whole page: H1 + Subtitle, hero demo before the first heading,
    // the two Usage guidelines bullets, Anatomy, the three Examples
    // subsections with the loading demo, and the API reference — mirroring
    // page.mdx's document order per specs/docs-content/button/page.md.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-button-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::button_page::ButtonPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || view! { <ButtonPage /> }));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Button</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains(
            "A button component that can be rendered as another tag or focusable when disabled."
        ),
        "the subtitle did not render"
    );
    for heading in [
        "Usage guidelines",
        "Anatomy",
        "Examples",
        "Rendering as another tag",
        "Rendering links as buttons",
        "Loading states",
        "API reference",
    ] {
        assert!(
            html.contains(&format!(">{heading}</")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    assert!(
        html.contains("leptos_ui::{ButtonProps, button_element}"),
        "the Anatomy import snippet did not render — the page must teach the port's API \
         (specs/docs-content/CONTRACT.md requirement 1)"
    );
    assert!(
        html.contains("nativeButton={false}"),
        "the custom-tag example did not render"
    );
    // Both demos mounted: the static hero "Submit" button and the loading
    // demo's initial "Submit" (its loading button has no onClick attached
    // until clicked, so the page shows two enabled buttons).
    assert_eq!(
        buttons_of(&container).len(),
        2,
        "the page renders both demos (hero + loading)"
    );
    for demo in ["hero", "loading"] {
        assert!(
            container
                .query_selector(&format!("[data-demo='{demo}']"))
                .expect("query")
                .is_some(),
            "the {demo} demo slot did not render"
        );
    }
}

/// `specs/docs-content/CONTRACT.md` requirement 1 at the browser level: both of the page's rendered
/// code blocks show the PORT's API. The browser-free `snippet_language_guard` in the page module
/// classifies the two constants at host-test time; this is the other half — what the REAL
/// `ButtonPage` mount actually puts in the DOM, classified with the same rules the
/// `ralph/scripts/visual-gap-report.mjs` probe uses. Before this page's translation that probe
/// raised a P0 on this route — "2 of 2 code block(s) still contain React source (JSX, hooks, or
/// `@base-ui/react` imports) instead of the Leptos port's own API" — while every structural check
/// stayed green, so the assertion is on the DOM, not on the constants.
#[wasm_bindgen_test]
fn button_page_snippets_teach_the_port_not_upstream() {
    use crate::pages::button_page::ButtonPage;
    use crate::snippet_language::{looks_leptos, looks_react};
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-button-snippets");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || view! { <ButtonPage /> }));

    let blocks = container.query_selector_all("pre").expect("query pre");
    let texts: Vec<String> = (0..blocks.length())
        .map(|i| {
            blocks
                .get(i)
                .expect("pre")
                .text_content()
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(
        texts.len(),
        2,
        "the page renders its two embedded snippets; found: {texts:?}"
    );

    let (mut leptos, mut react) = (0, 0);
    for text in &texts {
        match (looks_leptos(text), looks_react(text)) {
            (true, false) => leptos += 1,
            (_, true) => react += 1,
            _ => panic!("a snippet identifies as neither port nor React source: {text}"),
        }
    }
    assert_eq!(
        (leptos, react),
        (2, 0),
        "the probe must read {{total: 2, leptos: 2, react: 0}} for this page; blocks were: {texts:?}"
    );
    for text in &texts {
        assert!(
            !text.contains("@base-ui/react"),
            "a rendered snippet still carries upstream's runtime: {text}"
        );
    }
}

/// The button page's generated `## API reference` section, rendered through the ported reference
/// primitives: upstream's props section (header row + one anchored `<details>` per prop carrying
/// its short type and its default) and the generated data-attributes table.
///
/// ASSERTED AGAINST UPSTREAM'S OWN RENDER, not the port's shape alone: the section's
/// `--rows`, the four header labels, the five row anchors, and the cells each row's summary shows
/// were read off `http://127.0.0.1:3005/react/components/button` at 1280px (2026-09-16). Before
/// this item the route rendered the whole reference as three prose paragraphs — the gap report's
/// P0 read `upstream renders 1 table(s) (2 rows); this page renders 0`.
#[wasm_bindgen_test]
fn button_page_api_reference_renders_upstreams_section_and_table() {
    use crate::pages::button_page::ButtonPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-button-api-reference");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || view! { <ButtonPage /> }));

    // The props section, in upstream's shape and order: the described-by table id, the
    // `--rows` count, and the four header cells over the row grid.
    let section = container
        .query_selector("section.ReferenceAccordionRoot[aria-describedby='button-props-table']")
        .expect("query section")
        .expect("the props section rendered with its described-by table id");
    assert_eq!(
        section.get_attribute("style").unwrap_or_default(),
        "--rows:5",
        "the section does not carry upstream's row count"
    );
    let heads: Vec<String> = {
        let list = section
            .query_selector_all(".AccordionHeaderCellInner")
            .expect("query header cells");
        (0..list.length())
            .map(|i| {
                list.get(i)
                    .expect("cell")
                    .text_content()
                    .unwrap_or_default()
            })
            .collect()
    };
    assert_eq!(
        heads,
        vec!["Prop", "Type", "Default", ""],
        "the section's header row does not carry upstream's four labels"
    );

    // One `<details>` row per documented prop (types.md:16-20), each addressed by upstream's
    // anchor, each with a Name cell linking to it, and each carrying the summary cells upstream
    // renders: name, short type, default (an em dash where the generated table documents none).
    let rows = container
        .query_selector_all("details.AccordionItem")
        .expect("query prop rows");
    assert_eq!(rows.length(), 5, "the five Button props (types.md:16-20)");
    for (name, short_ty, default_cell) in [
        ("focusableWhenDisabled", "boolean", "false"),
        ("nativeButton", "boolean", "true"),
        ("className", "string | function", "\u{2014}"),
        ("style", "React.CSSProperties | function", "\u{2014}"),
        ("render", "ReactElement | function", "\u{2014}"),
    ] {
        let anchor = format!("Button-{name}");
        let summary = container
            .query_selector(&format!("#{anchor}"))
            .expect("query summary")
            .unwrap_or_else(|| panic!("the {name} prop row is not addressed by its anchor"));
        assert!(
            container
                .query_selector(&format!("a[href='#{anchor}']"))
                .expect("query name link")
                .is_some(),
            "the {name} row's Name cell does not link to its own anchor"
        );
        let cells: Vec<String> = {
            let list = summary
                .query_selector_all("code")
                .expect("query summary cells");
            (0..list.length())
                .map(|i| {
                    list.get(i)
                        .expect("cell")
                        .text_content()
                        .unwrap_or_default()
                })
                .collect()
        };
        assert_eq!(
            cells,
            vec![
                name.to_string(),
                short_ty.to_string(),
                default_cell.to_string()
            ],
            "the {name} row's summary drifted from upstream's render"
        );
    }

    // The four-item description list one row carries, and the generated description text.
    let labels: Vec<String> = {
        let row = container
            .query_selector("#Button-nativeButton")
            .expect("query summary")
            .expect("the nativeButton row rendered")
            .parent_element()
            .expect("the summary's `<details>` row");
        let list = row.query_selector_all("dt").expect("query dt");
        (0..list.length())
            .map(|i| list.get(i).expect("dt").text_content().unwrap_or_default())
            .collect()
    };
    assert_eq!(
        labels,
        vec!["Name", "Description", "Type", "Default"],
        "a documented prop row carries upstream's four description items"
    );
    let html = container.inner_html();
    assert!(
        html.contains("Whether the component renders a native"),
        "the generated description text did not render"
    );
    assert!(
        html.contains("Set to"),
        "the description's multi-line tail did not render"
    );

    // The generated data-attributes table: upstream's three-column head and its single row
    // (`types.md:24-26`), whose name cell is the row's `<th scope='row'>`.
    let table = container
        .query_selector("div.ReferenceTableRoot > table.TableRootTable")
        .expect("query table")
        .expect("the data-attributes table rendered");
    let heads: Vec<String> = {
        let list = table
            .query_selector_all("thead th")
            .expect("query head cells");
        (0..list.length())
            .map(|i| {
                list.get(i)
                    .expect("head cell")
                    .text_content()
                    .unwrap_or_default()
            })
            .collect()
    };
    assert_eq!(
        heads,
        vec!["Attribute", "Description", "-"],
        "the table does not carry upstream's head columns"
    );
    assert_eq!(
        table
            .query_selector_all("tbody tr")
            .expect("query rows")
            .length(),
        1,
        "the Button data-attributes table carries its single documented row"
    );
    assert!(
        html.contains("data-disabled") && html.contains("Present when the button is disabled."),
        "the generated data-attribute row did not render"
    );
}

// ============================== Progress docs page (`docs-content: components/progress`) ==============================

/// The first `[role='progressbar']` under the container.
fn progressbar_in(container: &web_sys::Element) -> web_sys::Element {
    container
        .query_selector("[role='progressbar']")
        .expect("query")
        .expect("the progress bar rendered")
}

#[wasm_bindgen_test]
async fn progress_hero_demo_drives_the_real_part_tree_through_the_interval_simulation() {
    // demos.json entry "hero": `useState(20)` + a 1s `setInterval` simulation
    // (`Math.min(100, Math.round(current + Math.random() * 25))`) re-rendering
    // `Progress.Root value={value}` — the demo mounted with a SHORT interval
    // (50 ms) so the cycle lands inside the test window; the contract under
    // test is the upstream one: the value starts at 20, strictly advances
    // toward 100, and every derived surface (root ARIA tuple, the Indicator's
    // inline width fill, the Value's formatted text) re-derives from the same
    // mirrored prop — the port's derivation pipeline, not demo machinery.
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-progress-demo");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::progress_page::ProgressHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || {
        view! {
            <ProgressHeroDemo interval_ms=50 />
        }
    }));

    // The initial surface — the seed render at value=20 (the demo's
    // useState(20)), the real derivation pipeline's output.
    let root = progressbar_in(&container);
    assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("20"));
    assert_eq!(
        root.get_attribute("aria-valuetext").as_deref(),
        Some("20%"),
        "the default percent aria-valuetext from the format pipeline"
    );
    assert_eq!(root.get_attribute("aria-valuemin").as_deref(), Some("0"));
    assert_eq!(root.get_attribute("aria-valuemax").as_deref(), Some("100"));
    assert!(
        root.get_attribute("class")
            .expect("root class")
            .contains("grid-cols-2"),
        "the upstream demo className rides the real root"
    );
    // The Label association (behavior.md "Accessibility"): aria-labelledby
    // points at the rendered role=presentation Label carrying "Export data",
    // and the id is the demo-minted `base-ui-…` one.
    let labelledby = root
        .get_attribute("aria-labelledby")
        .expect("the root's aria-labelledby is set by the Label's registration");
    assert!(
        labelledby.starts_with("base-ui-"),
        "the labelId comes from the real useBaseUiId generator; got: {labelledby}"
    );
    let label = container
        .query_selector(&format!("#{labelledby}"))
        .expect("query")
        .unwrap_or_else(|| panic!("no element under #{labelledby}"));
    assert_eq!(label.get_attribute("role").as_deref(), Some("presentation"));
    assert_eq!(label.text_content().as_deref(), Some("Export data"));

    // The interval effect's first run is deferred to the executor (the
    // button-page precedent): settle one turn before polling, then POLL for
    // the first advance instead of a fixed wait — the ticks are real
    // 50 ms macrotasks and a fixed sleep races them.
    flush_one_turn().await;
    let mut advanced_to: Option<i64> = None;
    for _ in 0..60 {
        flush_one_turn().await;
        {
            let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                web_sys::window()
                    .expect("window")
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 100)
                    .expect("set_timeout");
            });
            wasm_bindgen_futures::JsFuture::from(promise).await;
        }
        let now = progressbar_in(&container)
            .get_attribute("aria-valuenow")
            .expect("aria-valuenow present");
        if now != "20" {
            advanced_to = now.parse::<i64>().ok();
            break;
        }
    }
    let now = advanced_to.expect("the interval simulation advanced the value past the initial 20");

    // The advance contract (`hero/tailwind/index.tsx:11`): strictly rising,
    // clamped at 100.
    assert!(
        (21..=100).contains(&now),
        "the simulation moved 20 strictly toward the 100 clamp; got {now}"
    );

    // Every derived surface agrees on the SAME value (the React demo's single
    // `value={value}` prop feeding the whole derivation):
    // - the root's valuetext is the formatted percent of the new value, AND
    //   the Label association SURVIVED the rebuild — the same id resolving
    //   after the replacement (the React useId stability contract; the
    //   first wasm run caught the port's fresh-generated id churning per
    //   rebuild, fixed by the demo minting the id once through the real
    //   useBaseUiId generator),
    let root = progressbar_in(&container);
    assert_eq!(
        root.get_attribute("aria-valuetext").as_deref(),
        Some(format!("{now}%").as_str()),
        "aria-valuetext re-derived from the advanced value"
    );
    assert_eq!(
        root.get_attribute("aria-labelledby").as_deref(),
        Some(labelledby.as_str()),
        "the labelId is stable across the rebuild (the React useId contract)"
    );
    // - the Indicator's inline width fill tracks the new percentage,
    let indicator = container
        .query_selector("[role='progressbar'] > div > div")
        .or_else(|_| container.query_selector("div[style*='width']"))
        .expect("query")
        .expect("the indicator rendered");
    let style = indicator.get_attribute("style").unwrap_or_default();
    assert!(
        style.contains(&format!("width: {now}%")),
        "the indicator fill must carry the advanced width; style was: {style}"
    );
    // - the Value's aria-hidden text carries the formatted value,
    let value = container
        .query_selector("span[aria-hidden='true']")
        .expect("query")
        .expect("the value span rendered");
    assert_eq!(
        value.text_content().as_deref(),
        Some(format!("{now}%").as_str()),
        "the Value's formatted text re-derived"
    );
    // - the subtree was REPLACED per-render (one progressbar, not an
    //   accumulating list), and
    assert_eq!(
        container
            .query_selector_all("[role='progressbar']")
            .expect("query all")
            .length(),
        1,
        "each rebuild replaces the previous subtree (per-render replacement)"
    );
    // - exactly one status attribute rides every PART (the five parts the
    //   state walk covers — root, label, value, track, indicator — NOT the
    //   hardcoded NVDA workaround span, which upstream's
    //   `defaultProps.children` renders without the part walk):
    //   data-complete at the 100 clamp, data-progressing strictly below it.
    let expected = if now == 100 {
        "data-complete"
    } else {
        "data-progressing"
    };
    let label = container
        .query_selector(&format!("#{labelledby}"))
        .expect("query")
        .expect("the label part");
    let value_span = container
        .query_selector("span[aria-hidden='true']")
        .expect("query")
        .expect("the value part");
    let track = container
        .query_selector("[role='progressbar'] > div")
        .expect("query")
        .expect("the track part");
    let indicator = container
        .query_selector("[role='progressbar'] > div > div")
        .expect("query")
        .expect("the indicator part");
    for (name, part) in [
        ("root", &root),
        ("label", &label),
        ("value", &value_span),
        ("track", &track),
        ("indicator", &indicator),
    ] {
        assert!(
            part.has_attribute(expected),
            "the {name} part must carry {expected} at value {now}"
        );
    }
}

#[wasm_bindgen_test]
fn progress_page_route_renders_the_mirrored_structure() {
    // The whole page: H1 + Subtitle, the hero demo before the first heading,
    // the Anatomy snippet, and the API reference over the five parts —
    // mirroring page.mdx's document order per
    // specs/docs-content/progress/page.md. Mounted synchronously (the page's
    // own demo interval is 1000 ms), so the initial value=20 surface is
    // deterministic — no other test's timer can interleave (the suite's
    // tests run one at a time on the single JS thread).
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-progress-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::progress_page::ProgressPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <ProgressPage /> },
    ));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Progress</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("Displays the status of a task that takes a long time."),
        "the subtitle did not render"
    );
    for heading in [
        "Anatomy",
        "API reference",
        "Root",
        "Track",
        "Indicator",
        "Value",
        "Label",
    ] {
        assert!(
            html.contains(&format!(">{heading}</")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    assert!(
        html.contains("@base-ui/react/progress"),
        "the Anatomy import snippet did not render"
    );
    // The hero demo slot mounted the real part tree at the initial value.
    let hero = container
        .query_selector("[data-demo='hero']")
        .expect("query")
        .expect("the hero demo slot rendered");
    let root = progressbar_in(&hero);
    assert_eq!(root.get_attribute("aria-valuenow").as_deref(), Some("20"));
    assert_eq!(root.get_attribute("aria-valuetext").as_deref(), Some("20%"));
    // The demo precedes the first heading (page.mdx document order).
    let demo_at = html.find("data-demo").expect("demo slot in html");
    let anatomy_at = html
        .find("<h2>Anatomy</h2>")
        .expect("Anatomy heading in html");
    assert!(
        demo_at < anatomy_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );
    // The API reference prose echoes the generated tables' content (static
    // prose, never fabricated machinery).
    assert!(
        html.contains("getAriaValueText"),
        "the Root props prose did not render"
    );
    assert!(
        html.contains("data-progressing"),
        "the parts' data-attributes prose did not render"
    );
}

// ---------------------------------------------------------------------------
// The field docs page (`docs-content: components/field`)
// ---------------------------------------------------------------------------

/// The upstream hero demo's class strings, carried verbatim by the page
/// (`docs/src/app/(docs)/react/components/field/demos/hero/tailwind/index.tsx:5-16`).
const FIELD_DEMO_ROOT_CLASS: &str = "flex w-full max-w-64 flex-col items-start gap-1";
const FIELD_DEMO_LABEL_CLASS: &str = "text-sm font-bold text-neutral-950 dark:text-white";
const FIELD_DEMO_ERROR_CLASS: &str = "text-sm text-red-700 dark:text-red-400";
const FIELD_DEMO_DESCRIPTION_CLASS: &str = "text-sm text-neutral-600 dark:text-neutral-400";

/// The field hero demo's interaction, pinned end-to-end through the REAL parts
/// (`specs/docs-content/field/demos.json`: `stateManaged: "uncontrolled"` — the
/// demo holds no state; "Field.Error renders only when the native valueMissing
/// constraint matches (empty required input), surfacing once validation is
/// triggered"). The label↔control association, the `...elementProps` rest
/// (`required`, `placeholder`), the helper description, and the valueMissing
/// error gate are all the port's own machinery — the page adds no state.
#[wasm_bindgen_test]
async fn field_hero_demo_renders_the_real_part_composition() {
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-field-hero");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::field_page::FieldHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FieldHeroDemo /> },
    ));
    // The `elementProps` rest bag lands through the control's post-mount Effect
    // (the field module's bag-writer), so the attribute asserts follow one turn.
    flush_one_turn().await;

    // Field.Root — the upstream stack class + the pristine (untouched) state.
    let root = container
        .query_selector("div")
        .expect("query root")
        .expect("Field.Root rendered a <div>");
    assert_eq!(
        root.get_attribute("class").as_deref(),
        Some(FIELD_DEMO_ROOT_CLASS),
        "the demo Root carries the upstream className verbatim"
    );
    assert_eq!(
        root.get_attribute("data-touched"),
        None,
        "the uncontrolled demo starts untouched (no data-touched)"
    );

    // Field.Label — the children text + the class, associated with the control.
    let label = container
        .query_selector("label")
        .expect("query label")
        .expect("Field.Label rendered a <label>");
    assert_eq!(
        label.text_content().as_deref(),
        Some("Name"),
        "Field.Label renders upstream's children text"
    );
    assert_eq!(
        label.get_attribute("class").as_deref(),
        Some(FIELD_DEMO_LABEL_CLASS),
        "the demo Label carries the upstream className verbatim"
    );

    // Field.Control — the upstream input class plus the elementProps rest.
    let input = container
        .query_selector("input")
        .expect("query input")
        .expect("Field.Control rendered an <input>")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input as HtmlInputElement");
    assert!(
        input
            .get_attribute("class")
            .expect("input class")
            .starts_with("h-8 self-stretch border border-neutral-950"),
        "the demo Control carries the upstream className verbatim"
    );
    assert!(
        input.has_attribute("required"),
        "the elementProps rest reaches the input's required attribute"
    );
    assert_eq!(
        input.get_attribute("placeholder").as_deref(),
        Some("Required"),
        "the elementProps rest reaches the input's placeholder"
    );
    assert_eq!(
        label.get_attribute("for").as_deref(),
        input.get_attribute("id").as_deref(),
        "the label is automatically associated with the field control"
    );
    assert_eq!(
        input.value(),
        "",
        "the uncontrolled control starts empty (upstream passes no value/defaultValue)"
    );

    // Field.Description — a <p> with the helper text.
    let description = container
        .query_selector("p")
        .expect("query description")
        .expect("Field.Description rendered a <p>");
    assert_eq!(
        description.text_content().as_deref(),
        Some("Visible on your profile"),
        "Field.Description renders upstream's helper text"
    );
    assert_eq!(
        description.get_attribute("class").as_deref(),
        Some(FIELD_DEMO_DESCRIPTION_CLASS),
        "the demo Description carries the upstream className verbatim"
    );

    // Field.Error — gated on valueMissing, which the pristine empty control has
    // not surfaced yet (no validation has run). The port keeps the error element
    // MOUNTED and marks it `hidden` while unrendered (leptos has no
    // return-null-unmount arm here; the field crate's own adaptation), so the
    // unrendered state is pinned on the `hidden` attribute rather than on
    // absence — the deviation from upstream's `return null` (FieldError.tsx
    // :130-134) is recorded in ralph/logs/spec-discrepancies.md.
    let error = container
        .query_selector(".text-red-700")
        .expect("query error")
        .expect("the error slot is mounted in the pristine tree");
    assert!(
        error.has_attribute("hidden"),
        "the unrendered Field.Error must be hidden; html was: {}",
        container.inner_html()
    );
    assert_eq!(
        error.get_attribute("class").as_deref(),
        Some(FIELD_DEMO_ERROR_CLASS),
        "the demo Error carries the upstream className verbatim"
    );
}

/// The demo's error slot is WIRED to the native `valueMissing` constraint, but
/// nothing in the demo TRIGGERS validation — pinned here so the page's real
/// behavior is documented rather than assumed.
///
/// `specs/docs-content/field/demos.json`'s `nonTrivialInteractions[1]` says the
/// error surfaces "per Field.Root's default onBlur validation mode". That is
/// wrong on both counts against upstream source: Field.Root's default mode is
/// `'onSubmit'` (`internals/form-context/FormContext.ts:42`,
/// `internals/field-root-context/FieldRootContext.ts:48`) and the demo has no
/// `<Form>` and no submit control at all. Upstream's own FieldError tests drive
/// the error through a `<Form>` + `<button type="submit">`
/// (`FieldError.test.tsx:32-53`): the submit is what commits `state.valid = false`,
/// and only from that state does the change path refresh validity
/// (`useFieldValidation.ts:244-246` — `revalidate` returns early while
/// `state.valid !== false`, which the pristine field's `null` always is). So the
/// typed-then-cleared input leaves the slot hidden — in the React docs and in the
/// port alike. Recorded in `ralph/logs/spec-discrepancies.md`.
#[wasm_bindgen_test]
async fn field_hero_demo_keeps_the_error_hidden_until_validation_is_triggered() {
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-field-hero-dirty");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::field_page::FieldHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FieldHeroDemo /> },
    ));
    flush_one_turn().await;

    let input = container
        .query_selector("input")
        .expect("query input")
        .expect("Field.Control rendered an <input>")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input as HtmlInputElement");

    let type_into = |value: &str| {
        input.set_value(value);
        let init = web_sys::EventInit::new();
        init.set_bubbles(true);
        let event = web_sys::Event::new_with_event_init_dict("input", &init).expect("input event");
        input.dispatch_event(&event).expect("dispatch input");
    };
    let error = || {
        container
            .query_selector(".text-red-700")
            .expect("query error")
            .expect("the error slot is mounted")
    };

    assert!(
        error().has_attribute("hidden"),
        "the pristine error slot is hidden; html was: {}",
        container.inner_html()
    );

    // The demo's own interaction path: type, then clear (the required control is
    // empty again).
    type_into("A");
    flush_one_turn().await;
    assert!(
        error().has_attribute("hidden"),
        "a filled required control is valid — the error stays hidden"
    );
    type_into("");
    flush_one_turn().await;
    assert!(
        error().has_attribute("hidden"),
        "the demo has no Form/submit, so nothing commits validity — the valueMissing \
         slot stays hidden (upstream's revalidate early-return); html was: {}",
        container.inner_html()
    );

    // The wiring itself is real and is what the page renders: the demo class list,
    // the message text, and the control's elementProps rest.
    assert_eq!(
        error().get_attribute("class").as_deref(),
        Some(FIELD_DEMO_ERROR_CLASS),
        "the error slot carries the upstream className verbatim"
    );
    let html = container.inner_html();
    assert!(
        html.contains("Please enter your name"),
        "the error slot carries the demo's message text; html was: {html}"
    );
    assert!(
        input.has_attribute("required"),
        "the control is the real required input the constraint is wired to"
    );
}

/// The whole mirrored page: H1 + Subtitle, the hero demo before the first
/// heading, the Anatomy snippet, and the API reference over the seven parts —
/// `specs/docs-content/field/page.md` document order.
#[wasm_bindgen_test]
fn field_page_component_renders_the_full_page_structure() {
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-field-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use crate::pages::field_page::FieldPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || view! { <FieldPage /> }));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Field</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A component that provides labeling and validation for form controls."),
        "the subtitle did not render"
    );
    for heading in [
        "Anatomy",
        "API reference",
        "Root",
        "Label",
        "Control",
        "Description",
        "Item",
        "Error",
        "Validity",
    ] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    assert!(
        html.contains("@base-ui/react/field"),
        "the Anatomy import snippet did not render"
    );
    // The hero demo slot mounted the real part tree (the required empty input).
    let hero = container
        .query_selector("[data-demo='hero']")
        .expect("query")
        .expect("the hero demo slot rendered");
    assert!(
        hero.query_selector("input").expect("query input").is_some(),
        "the live hero demo did not render its Field.Control"
    );
    // The demo precedes the first heading (page.mdx document order).
    let demo_at = html.find("data-demo").expect("demo slot in html");
    let anatomy_at = html
        .find("<h2>Anatomy</h2>")
        .expect("Anatomy heading in html");
    assert!(
        demo_at < anatomy_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );
    // The API reference prose echoes the generated tables' content (static
    // prose, never fabricated machinery).
    assert!(
        html.contains("validationMode"),
        "the Root props prose did not render"
    );
    assert!(
        html.contains("data-starting-style"),
        "the Error data-attributes prose did not render"
    );
}

// The checkbox docs page (`docs-content: components/checkbox`)
// ---------------------------------------------------------------------------

/// The upstream hero demo's class strings, carried verbatim by the page
/// (`docs/src/app/(docs)/react/components/checkbox/demos/hero/tailwind/index.tsx:6-11`).
const CHECKBOX_DEMO_LABEL_CLASS: &str =
    "flex items-center gap-2 text-sm font-normal text-neutral-950 dark:text-white";
const CHECKBOX_DEMO_ROOT_CLASS: &str = "flex size-4 shrink-0 items-center justify-center border rounded-none p-0 border-neutral-950 bg-white text-white dark:border-white dark:bg-neutral-950 dark:text-neutral-950 data-checked:bg-neutral-950 data-checked:text-white dark:data-checked:bg-white dark:data-checked:text-neutral-950 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white";
const CHECKBOX_DEMO_INDICATOR_CLASS: &str = "flex data-unchecked:hidden";

/// A fresh mount container for a checkbox-page test.
fn checkbox_container(id: &str) -> web_sys::HtmlElement {
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

/// The hero demo (`demos/hero/tailwind/index.tsx`, the single demos.json entry):
/// upstream's exact element composition rendered through the REAL
/// `leptos_ui::checkbox` view functions — the enclosing label, the `span` control
/// (ticked by `defaultChecked`, so `data-checked` is the initial hook), the hidden
/// input beside it, and the mounted Indicator wrapping the checkmark svg. The page
/// adds no state of its own (demos.json `stateManaged`).
#[wasm_bindgen_test]
async fn checkbox_hero_demo_renders_the_real_part_composition() {
    let container = checkbox_container("test-mount-root-checkbox-hero");

    use crate::pages::checkbox_page::CheckboxHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxHeroDemo /> },
    ));
    // The visible control's attribute bag lands through a post-mount writer
    // (the field/demo convention), so the asserts follow one real turn.
    flush_one_turn().await;

    // The enclosing <label> — upstream's "simplest labeling pattern" (page.mdx:33),
    // with the label text after the checkbox.
    let label = container
        .query_selector("label")
        .expect("query label")
        .expect("the demo renders an enclosing <label>");
    assert_eq!(
        label.get_attribute("class").as_deref(),
        Some(CHECKBOX_DEMO_LABEL_CLASS),
        "the demo label carries the upstream className verbatim"
    );
    assert!(
        label
            .text_content()
            .expect("label text")
            .contains("Enable notifications"),
        "the label renders upstream's text"
    );

    // The visible control: a <span> (the documented default, page.mdx:46) with the
    // upstream class and the checked state the `defaultChecked` prop seeds.
    let control = container
        .query_selector("[role=\"checkbox\"]")
        .expect("query control")
        .expect("the demo renders the real Checkbox.Root control");
    assert_eq!(control.tag_name(), "SPAN", "the default control is a span");
    assert_eq!(
        control.get_attribute("class").as_deref(),
        Some(CHECKBOX_DEMO_ROOT_CLASS),
        "the demo control carries the upstream className verbatim"
    );
    assert_eq!(
        control.get_attribute("aria-checked").as_deref(),
        Some("true"),
        "defaultChecked seeds the checked state"
    );
    assert!(
        control.has_attribute("data-checked"),
        "the checked hook is present for the demo's data-checked: variants"
    );
    assert!(
        !control.has_attribute("data-unchecked"),
        "a checked box carries no data-unchecked"
    );

    // The hidden input beside it (the port's real DOM contract).
    let input = container
        .query_selector("input[type=\"checkbox\"]")
        .expect("query input")
        .expect("the hidden input rendered")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input as HtmlInputElement");
    assert!(
        input.checked(),
        "the hidden input mirrors the checked state"
    );
    assert_eq!(input.get_attribute("tabindex").as_deref(), Some("-1"));
    assert_eq!(input.get_attribute("aria-hidden").as_deref(), Some("true"));

    // The Indicator: mounted because the box is ticked, carrying the upstream class
    // (whose data-unchecked:hidden variant is the demo's actual hiding mechanism)
    // and the checkmark svg (`hero/tailwind/index.tsx:20-34`). Selected by that
    // variant — the control's own class also begins with `flex`.
    let indicator = container
        .query_selector("span[class*=\"data-unchecked:hidden\"]")
        .expect("query indicator")
        .expect("the Indicator is mounted while checked");
    assert_eq!(
        indicator.get_attribute("class").as_deref(),
        Some(CHECKBOX_DEMO_INDICATOR_CLASS),
        "the demo Indicator carries the upstream className verbatim"
    );
    let svg = indicator
        .query_selector("svg")
        .expect("query svg")
        .expect("the Indicator wraps the checkmark svg");
    assert_eq!(svg.get_attribute("width").as_deref(), Some("16"));
    assert_eq!(svg.get_attribute("viewBox").as_deref(), Some("0 0 16 16"));
    assert_eq!(svg.get_attribute("stroke").as_deref(), Some("currentColor"));
    assert_eq!(
        svg.get_attribute("style").as_deref(),
        Some("display:block"),
        "upstream's inline display:block on the svg"
    );
    let path = svg
        .query_selector("path")
        .expect("query path")
        .expect("the checkmark path rendered");
    assert_eq!(path.get_attribute("d").as_deref(), Some("m2.5 8.5 4 4 7-9"));
}

/// The demo's one interaction, driven end-to-end through the port's real machine:
/// a click on the control funnels through the hidden input's `change` (the unit's
/// single state funnel), flipping `aria-checked`/the `data-*` hooks, the input's
/// `checked` property, and unmounting the Indicator (the mount gate).
#[wasm_bindgen_test]
async fn checkbox_hero_demo_toggles_through_the_real_port() {
    let container = checkbox_container("test-mount-root-checkbox-hero-toggle");

    use crate::pages::checkbox_page::CheckboxHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxHeroDemo /> },
    ));
    flush_one_turn().await;

    let control = container
        .query_selector("[role=\"checkbox\"]")
        .expect("query control")
        .expect("the control rendered")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("control as HtmlElement");
    let input = container
        .query_selector("input[type=\"checkbox\"]")
        .expect("query input")
        .expect("the hidden input rendered")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input as HtmlInputElement");
    assert!(input.checked(), "the demo starts ticked");

    // A real activating click (the browser runs the element's activation behavior, so
    // the port's re-dispatch onto the hidden input happens for real).
    control.click();
    // The exit-completion that unmounts the indicator is frame-driven, so wait real
    // frames rather than timer turns (the crate's own `settle_frames(2)`).
    flush_one_frame().await;
    flush_one_frame().await;

    assert_eq!(
        control.get_attribute("aria-checked").as_deref(),
        Some("false"),
        "the click unticked the checkbox"
    );
    assert!(
        control.has_attribute("data-unchecked"),
        "the unchecked hook is present after the click"
    );
    assert!(
        !control.has_attribute("data-checked"),
        "the checked hook is gone after the click"
    );
    assert!(
        !input.checked(),
        "the hidden input's checked property follows the state"
    );
    assert!(
        container
            .query_selector("span[class*=\"data-unchecked:hidden\"]")
            .expect("query indicator")
            .is_none(),
        "unchecking unmounts the Indicator (the mount gate), leaving the checkmark hidden"
    );
}

/// The mirrored page structure (`page.mdx`): the h1, the `<Subtitle>`, the hero
/// demo before the first heading, every heading in document order, the Anatomy and
/// Examples snippets, and the API-reference prose echoing the generated tables.
#[wasm_bindgen_test]
fn checkbox_page_component_renders_the_full_page_structure() {
    let container = checkbox_container("test-mount-root-checkbox-page");

    use crate::pages::checkbox_page::CheckboxPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxPage /> },
    ));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Checkbox</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("An easily stylable checkbox component."),
        "the subtitle did not render"
    );
    for heading in [
        "Usage guidelines",
        "Anatomy",
        "Examples",
        "Labeling a checkbox",
        "Rendering as a native button",
        "Form integration",
        "API reference",
        "Root",
        "Indicator",
    ] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    assert!(
        html.contains("checkbox_root_view"),
        "the Anatomy snippet (the port's own composition) did not render"
    );
    assert!(
        html.contains("Accept terms and conditions"),
        "the labeling snippet did not render"
    );
    assert!(
        html.contains("native_button"),
        "the native-button snippet did not render"
    );
    assert!(
        html.contains("stayLoggedIn"),
        "the form-integration snippet did not render"
    );
    // The page must teach the PORT, not upstream (`specs/docs-content/CONTRACT.md` requirement 1).
    // The host-side guard in `pages/checkbox_page.rs` asserts this over the five constants; this
    // asserts it over the DOM the route actually renders, since the obligation is about what the
    // page shows. The markers below are the JSX spellings these blocks carried verbatim until the
    // snippet-translation pass (`@base-ui/react/checkbox`, `nativeButton`, `htmlFor`).
    let blocks = container
        .query_selector_all("pre")
        .expect("query pre blocks");
    assert_eq!(
        blocks.length(),
        5,
        "the page mirrors upstream's five embedded code blocks"
    );
    for i in 0..blocks.length() {
        let text = blocks
            .get(i)
            .expect("pre block")
            .text_content()
            .unwrap_or_default();
        for marker in [
            "@base-ui/react",
            "className=",
            "<Checkbox",
            "htmlFor",
            "nativeButton",
        ] {
            assert!(
                !text.contains(marker),
                "code block {i} still carries React source ({marker}); it read: {text}"
            );
        }
    }
    // The hero demo slot mounted the real part tree. The sync test cannot see the
    // post-mount attribute bag (the control's `role` lands in an Effect), so this
    // asserts on the statically-rendered hidden input instead — the demo's own
    // render tests above cover the attribute surface.
    let hero = container
        .query_selector("[data-demo='hero']")
        .expect("query")
        .expect("the hero demo slot rendered");
    assert!(
        hero.query_selector("input[type=\"checkbox\"]")
            .expect("query input")
            .is_some(),
        "the live hero demo did not render the real Checkbox control"
    );
    // The demo precedes the first heading (page.mdx document order).
    let demo_at = html.find("data-demo").expect("demo slot in html");
    let guidelines_at = html
        .find("<h2>Usage guidelines</h2>")
        .expect("Usage guidelines heading in html");
    assert!(
        demo_at < guidelines_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );
    // The API reference section renders the generated `types.md` content as upstream renders it
    // (real tables + one `<details>` row per prop, `docs-chrome: API reference tables`); this
    // structure test only asserts the section mounted with its content present, and the
    // dedicated table test below asserts the generated shape.
    assert!(
        html.contains("uncheckedValue"),
        "the Root props did not render"
    );
    assert!(
        html.contains("data-starting-style"),
        "the Indicator data-attributes table did not render"
    );
}

/// The `## API reference` section renders the generated `types.md` content in the shape upstream
/// renders it (`docs-chrome: API reference tables`): two real `<table>`s — the Root and Indicator
/// data-attribute lists, `docs/src/app/(docs)/react/components/checkbox/types.md:22-33` and
/// `:126-141`, which upstream measures as 2 tables / 28 rows — and one `<details>` row per
/// documented prop carrying the prop's `#CheckboxRoot-<name>` anchor, so the generated content is
/// a table and an addressed tree instead of one paragraph of prose.
#[wasm_bindgen_test]
fn checkbox_page_api_reference_renders_the_generated_tables() {
    let container = checkbox_container("test-mount-root-checkbox-api-reference");

    use crate::pages::checkbox_page::CheckboxPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxPage /> },
    ));

    // Two tables, in upstream's order, with upstream's row counts and head columns.
    let tables: Vec<web_sys::Element> = {
        let list = container.query_selector_all("table").expect("query tables");
        let mut out = Vec::new();
        for i in 0..list.length() {
            out.push(
                list.get(i)
                    .expect("table at index")
                    .dyn_into::<web_sys::Element>()
                    .expect("element"),
            );
        }
        out
    };
    assert_eq!(
        tables.len(),
        2,
        "the API reference renders the Root and Indicator data-attribute tables"
    );
    let mut row_counts = Vec::new();
    for (i, table) in tables.iter().enumerate() {
        row_counts.push(
            table
                .query_selector_all("tbody tr")
                .expect("query rows")
                .length(),
        );
        let heads: Vec<String> = {
            let list = table
                .query_selector_all("thead th")
                .expect("query head cells");
            (0..list.length())
                .map(|j| {
                    list.get(j)
                        .expect("head cell")
                        .text_content()
                        .unwrap_or_default()
                })
                .collect()
        };
        assert_eq!(
            heads,
            vec!["Attribute", "Description", "-"],
            "table {i} does not carry upstream's head columns"
        );
    }
    assert_eq!(
        row_counts,
        vec![12, 14],
        "the Root table carries its 12 data attributes and the Indicator table its 14 \
         (types.md:22-33 / :126-141)"
    );

    // One `<details>` row per documented prop, each addressed by upstream's anchor, and each
    // carrying the four-item `dl` the expanded row shows.
    let rows = container
        .query_selector_all("details.AccordionItem")
        .expect("query prop rows");
    assert_eq!(
        rows.length(),
        22,
        "18 Root props + 4 Indicator props (types.md:14-31 / :117-122)"
    );
    for (prop, prefix) in [
        ("name", "CheckboxRoot"),
        ("render", "CheckboxRoot"),
        ("keepMounted", "CheckboxIndicator"),
    ] {
        let anchor = format!("#{prefix}-{prop}");
        assert!(
            container
                .query_selector(&format!("[id='{prefix}-{prop}']"))
                .expect("query anchor")
                .is_some(),
            "the prop row for {prop} is not addressed by its upstream anchor id"
        );
        assert!(
            container
                .query_selector(&format!("a[href='{anchor}']"))
                .expect("query name link")
                .is_some(),
            "the {prop} row's Name cell does not link to its own anchor"
        );
    }
    let labels: Vec<String> = {
        // The `<summary>` is the `<details>`'s first child; the four `dt`s live in the `dl`
        // beside it (upstream's row shape), so the query starts from the row, not the summary.
        let row = container
            .query_selector("#CheckboxRoot-name")
            .expect("query summary")
            .expect("the name prop row rendered")
            .parent_element()
            .expect("the summary's `<details>` row");
        let list = row.query_selector_all("dt").expect("query dt");
        (0..list.length())
            .map(|j| list.get(j).expect("dt").text_content().unwrap_or_default())
            .collect()
    };
    assert_eq!(
        labels,
        vec!["Name", "Description", "Type", "Default"],
        "a documented prop row carries upstream's four description items"
    );
    assert!(
        container
            .inner_html()
            .contains("Identifies the field when a form is submitted."),
        "the generated description text did not render"
    );
}

// ---------------------------------------------------------------------------
// The avatar docs page (`docs-content: components/avatar`)
// ---------------------------------------------------------------------------

/// The upstream hero demo's class strings, carried verbatim by the page
/// (`docs/src/app/(docs)/react/components/avatar/demos/hero/tailwind/index.tsx:7-15`).
const AVATAR_DEMO_ROOT_CLASS: &str = "inline-flex size-8 items-center justify-center overflow-hidden rounded-full bg-neutral-200 align-middle text-sm leading-none font-normal text-neutral-950 select-none dark:bg-neutral-800 dark:text-white";
const AVATAR_DEMO_IMAGE_CLASS: &str = "size-full object-cover";
const AVATAR_DEMO_FALLBACK_CLASS: &str = "flex size-full items-center justify-center text-sm";

/// A fresh mount container for an avatar-page test (the checkbox-page shape).
fn avatar_container(id: &str) -> web_sys::HtmlElement {
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

/// Drives BOTH reaction timings the port needs: the futures executor (`poll_local`,
/// which runs the components' rg-0.2 machinery effects — including the fallback's
/// `useTimeout` start) and leptos's own scheduler (`task::tick` plus a real
/// macrotask turn).
async fn settle_machinery() {
    for _ in 0..32 {
        any_spawner::Executor::poll_local();
    }
    leptos::task::tick().await;
    for _ in 0..32 {
        any_spawner::Executor::poll_local();
    }
    flush_one_turn().await;
    for _ in 0..32 {
        any_spawner::Executor::poll_local();
    }
}

/// Settles the machinery, awaits a REAL `ms`-long timeout (the Fallback's `delay`
/// latch runs on a real `useTimeout` timer — behavior.md *State model*: "hidden
/// until the delay elapses"), then settles again so the latch reaches the view
/// through the port's rg→leptos mirror and the dynamic-view re-run.
///
/// The leading settle matters: the latch's timer is STARTED inside an rg-0.2
/// effect, which only runs when the futures executor is polled — wait first and
/// the 600 ms window begins only after it has already elapsed.
async fn flush_after_ms(ms: i32) {
    settle_machinery().await;
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        web_sys::window()
            .expect("window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .expect("set_timeout");
    });
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .expect("await the delay window");
    settle_machinery().await;
}

/// The element children of `parent`, in document order. `HtmlCollection`'s
/// indexed accessors are not generated in this workspace's `web-sys` feature
/// set, so the walk uses the `Node`/`Element` cursors (`first_element_child` /
/// `next_element_sibling`) instead.
fn element_children(parent: &web_sys::Element) -> Vec<web_sys::Element> {
    let mut out: Vec<web_sys::Element> = Vec::new();
    let mut current = parent.first_element_child();
    while let Some(child) = current {
        current = child.next_element_sibling();
        out.push(child);
    }
    out
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, the single demos.json entry):
/// upstream's exact element composition rendered through the REAL
/// `leptos_ui::avatar` parts — two `<Avatar.Root>` spans side by side, the first
/// NESTING `Avatar.Image` (remote `src`, `width`/`height` 48, `object-cover`)
/// and `Avatar.Fallback` (`delay={600}`, "LT"), the second carrying a bare `LT`
/// text child.
///
/// This is the assertion the pair was blocked on: the parts are CHILDREN of the
/// root span, and the root itself is the only child of the wrapper — a
/// sibling-mounted port (what `use_avatar_root` alone produces) would put three
/// nodes beside each other and fail here. The image-or-fallback exclusivity is
/// asserted after the delay window, so it holds whether the remote image loads
/// (the `<img>` replaces the fallback) or fails/stays pending (the fallback
/// remains) — the network never decides whether the test passes, only which of
/// the two upstream-legal DOM states is on screen (behavior.md *Edge cases*).
#[wasm_bindgen_test]
async fn avatar_hero_demo_renders_the_real_root_composition() {
    let container = avatar_container("test-mount-root-avatar-hero");

    use crate::pages::avatar_page::AvatarHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <AvatarHeroDemo /> },
    ));
    // The Root's className/attribute bag is written by the root view's commit
    // EFFECT (the `view!`-has-no-attribute-spread convention in the port), so the
    // first turn has to land before the DOM carries it.
    flush_one_turn().await;

    let wrapper = container
        .query_selector("div.flex")
        .expect("query the wrapper")
        .expect("the demo's `flex gap-4` wrapper rendered");

    let roots: Vec<web_sys::Element> = element_children(&wrapper);
    assert_eq!(
        roots.len(),
        2,
        "the wrapper holds exactly the two Root spans — a part mounted as a SIBLING would show up here"
    );
    for root in &roots {
        assert_eq!(root.tag_name(), "SPAN", "Avatar.Root renders a <span>");
        assert_eq!(
            root.get_attribute("class").as_deref(),
            Some(AVATAR_DEMO_ROOT_CLASS),
            "the upstream Root className is carried verbatim"
        );
    }

    // The second avatar: `<Avatar.Root>LT</Avatar.Root>` (`:17-19`) — the bare
    // text child no part could render (only the root view can).
    let second = &roots[1];
    assert_eq!(
        second.text_content().as_deref(),
        Some("LT"),
        "the second avatar's text child renders inside its root"
    );
    assert_eq!(
        second.children().length(),
        0,
        "the second avatar carries no element children"
    );

    // The delay window (600 ms) plus slack: the parts have settled.
    flush_after_ms(1000).await;

    let first = &roots[0];
    let parts: Vec<web_sys::Element> = element_children(first);
    assert_eq!(
        parts.len(),
        1,
        "exactly one of Image/Fallback is mounted inside the root (behavior.md *Edge cases*); \
         the root's inner HTML was: {}",
        first.inner_html()
    );
    match parts[0].tag_name().as_str() {
        "IMG" => {
            assert_eq!(
                parts[0].get_attribute("class").as_deref(),
                Some(AVATAR_DEMO_IMAGE_CLASS),
                "the loaded image keeps the upstream Image className"
            );
            assert_eq!(
                parts[0].get_attribute("width").as_deref(),
                Some("48"),
                "the width prop rides the engine's attribute bag"
            );
            assert_eq!(parts[0].get_attribute("height").as_deref(), Some("48"));
            assert!(
                parts[0]
                    .get_attribute("src")
                    .unwrap_or_default()
                    .contains("images.unsplash.com"),
                "the image renders the demo's remote src"
            );
        }
        "SPAN" => {
            assert_eq!(
                parts[0].get_attribute("class").as_deref(),
                Some(AVATAR_DEMO_FALLBACK_CLASS),
                "the fallback keeps the upstream Fallback className"
            );
            assert_eq!(
                parts[0].text_content().as_deref(),
                Some("LT"),
                "the fallback renders its initials child"
            );
        }
        other => panic!("unexpected part element: {other}"),
    }
}

/// The mirrored page structure (`page.mdx`): the h1, the `<Subtitle>`, the hero
/// demo before the first heading, every heading in document order, the three
/// embedded snippets, and the API-reference prose echoing the generated
/// `types.md` tables.
#[wasm_bindgen_test]
fn avatar_page_component_renders_the_full_page_structure() {
    let container = avatar_container("test-mount-root-avatar-page");

    use crate::pages::avatar_page::AvatarPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || view! { <AvatarPage /> }));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Avatar</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("An easily stylable avatar component."),
        "the subtitle did not render"
    );
    for heading in [
        "Anatomy",
        "Optimized and lazy-loaded images",
        "Stacking",
        "Server rendering",
        "API reference",
        "Root",
        "Image",
        "Fallback",
        "Additional types",
    ] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    assert!(
        html.contains("@base-ui/react/avatar"),
        "the Anatomy import snippet did not render"
    );
    assert!(
        html.contains("next/image"),
        "the 'Using next/image' snippet did not render"
    );
    assert!(
        html.contains(".Image[data-loading]"),
        "the stacking css snippet did not render"
    );
    assert!(
        html.contains("data-starting-style"),
        "the Image data-attributes prose did not render"
    );
    assert!(
        html.contains("'idle' | 'loading' | 'loaded' | 'error'"),
        "the ImageLoadingStatus additional-type prose did not render"
    );
    // The demo precedes the first heading (page.mdx document order).
    let demo_at = html.find("data-demo").expect("demo slot in html");
    let anatomy_at = html
        .find("<h2>Anatomy</h2>")
        .expect("Anatomy heading in html");
    assert!(
        demo_at < anatomy_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );
    // The hero demo slot mounted the real part tree (the roots are spans with
    // the upstream className — asserted in full by the demo render test above).
    let hero = container
        .query_selector("[data-demo='hero']")
        .expect("query")
        .expect("the hero demo slot rendered");
    assert_eq!(
        hero.query_selector_all("span")
            .expect("query the demo's spans")
            .length(),
        2,
        "the live hero demo did not render the two real Root spans"
    );
}

// The checkbox-group docs page (`docs-content: components/checkbox-group`)
// ---------------------------------------------------------------------------

/// The hero demo's class strings, carried verbatim by the page
/// (`docs/src/app/(docs)/react/components/checkbox-group/demos/hero/tailwind/index.tsx:12-24`).
const CHECKBOX_GROUP_HERO_GROUP_CLASS: &str =
    "flex flex-col items-start gap-1 text-neutral-950 dark:text-white";
const CHECKBOX_GROUP_HERO_ITEM_CLASS: &str =
    "flex items-center gap-2 text-sm font-normal text-neutral-950 dark:text-white";
const CHECKBOX_GROUP_HERO_INDICATOR_CLASS: &str = "flex data-unchecked:hidden";

/// A fresh mount container for a checkbox-group-page test.
fn checkbox_group_container(id: &str) -> web_sys::HtmlElement {
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

/// Every element matching `selector` under `root`, in document order (the `NodeList`
/// walk the crate's own wasm suites use — `NodeList` exposes `length`/`item`, not an
/// iterator, on this web-sys build).
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

/// Settles a chain of Effect hops (the demo's own leptos→rg-0.2 mirror, the parts'
/// rg→leptos mirrors, and the attribute writers) and then the frame-driven Indicator
/// mount/unmount — the checkbox suite's `settle_frames` shape, twice.
async fn settle_checkbox_group() {
    flush_one_frame().await;
    flush_one_frame().await;
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): upstream's
/// exact element composition rendered through the REAL `leptos_ui::checkbox_group_view`
/// — the group `div` (role=group, the `aria-labelledby` link to the sibling caption),
/// then the three enclosing `<label>` items, each with a real Checkbox control, its
/// hidden input and its Indicator. Uncontrolled: `defaultValue={['fuji-apple']}` seeds
/// the group, so only the fuji checkbox starts ticked.
#[wasm_bindgen_test]
async fn checkbox_group_hero_demo_renders_the_real_part_composition() {
    let container = checkbox_group_container("test-mount-root-checkbox-group-hero");

    use crate::pages::checkbox_group_page::CheckboxGroupHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxGroupHeroDemo /> },
    ));
    flush_one_turn().await;

    // The group element: upstream's `<CheckboxGroup>` is a real `<div role="group">`
    // with the demo className and the `aria-labelledby` link to the caption.
    let group = container
        .query_selector("[role=\"group\"]")
        .expect("query group")
        .expect("the demo renders the real CheckboxGroup element");
    assert_eq!(group.tag_name(), "DIV");
    assert_eq!(
        group.get_attribute("class").as_deref(),
        Some(CHECKBOX_GROUP_HERO_GROUP_CLASS),
        "the group carries the upstream className verbatim"
    );
    let labelled_by = group
        .get_attribute("aria-labelledby")
        .expect("the demo labels the group with aria-labelledby (page.mdx:34)");
    assert!(
        labelled_by.starts_with("base-ui-"),
        "the demo id rides the real useBaseUiId generator: {labelled_by:?}"
    );

    // The sibling caption div the group points at (`:14-16`).
    let caption = container
        .query_selector(&format!("#{labelled_by}"))
        .expect("query caption")
        .expect("the caption the group is labelled by rendered");
    assert_eq!(
        caption.get_attribute("class").as_deref(),
        Some("text-sm font-bold")
    );
    assert_eq!(caption.text_content().as_deref(), Some("Apples"));

    // The three items, in upstream's order, each an enclosing <label>.
    let labels = els(&group, "label");
    assert_eq!(labels.len(), 3, "the hero demo renders three items");
    for label in &labels {
        assert_eq!(
            label.get_attribute("class").as_deref(),
            Some(CHECKBOX_GROUP_HERO_ITEM_CLASS)
        );
    }
    assert_eq!(
        labels
            .iter()
            .map(|label| label.text_content().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["Fuji", "Gala", "Granny Smith"],
        "items render in upstream's order with their text"
    );

    // The three real controls, ticked per the group's `defaultValue`.
    let controls = els(&group, "[role=\"checkbox\"]");
    assert_eq!(controls.len(), 3, "the hero demo renders three checkboxes");
    assert_eq!(
        controls[0].get_attribute("aria-checked").as_deref(),
        Some("true"),
        "the group's defaultValue seeds the first checkbox as ticked"
    );
    assert!(controls[0].has_attribute("data-checked"));
    for control in &controls[1..] {
        assert_eq!(
            control.get_attribute("aria-checked").as_deref(),
            Some("false")
        );
        assert!(
            control.has_attribute("data-unchecked"),
            "an unticked box carries the data-unchecked hook"
        );
    }

    // The hidden inputs carry the demo's `name`, and the `value` follows upstream's
    // in-group rule (`CheckboxRoot.tsx:256-260`): `(groupContext ? checked && valueProp :
    // valueProp) || ''` — inside a CheckboxGroup the hidden input's value is set only
    // while the box is ticked, so an unticked member's input value is the empty string.
    let inputs: Vec<web_sys::HtmlInputElement> = els(&group, "input[type=\"checkbox\"]")
        .into_iter()
        .map(|node| node.dyn_into::<web_sys::HtmlInputElement>().expect("input"))
        .collect();
    assert_eq!(inputs.len(), 3);
    for input in &inputs {
        assert_eq!(input.get_attribute("name").as_deref(), Some("apple"));
    }
    assert_eq!(
        inputs[0].get_attribute("value").as_deref(),
        Some("fuji-apple"),
        "the ticked member's input carries its group value"
    );
    assert_eq!(
        inputs[1].get_attribute("value").as_deref(),
        Some(""),
        "an unticked member's input value is empty (upstream's in-group rule)"
    );
    assert_eq!(inputs[2].get_attribute("value").as_deref(), Some(""));
    assert!(inputs[0].checked(), "the pre-ticked input is checked");

    // The Indicator: mounted only where the box is ticked (the checkbox unit's mount
    // gate), carrying the upstream class whose `data-unchecked:hidden` variant is the
    // demo's hiding mechanism, and wrapping the checkmark svg.
    let indicators = els(
        &group,
        &format!("span[class=\"{CHECKBOX_GROUP_HERO_INDICATOR_CLASS}\"]"),
    );
    assert_eq!(
        indicators.len(),
        1,
        "only the ticked checkbox mounts its Indicator"
    );
    let path = indicators[0]
        .query_selector("path")
        .expect("query path")
        .expect("the Indicator wraps the checkmark svg");
    assert_eq!(path.get_attribute("d").as_deref(), Some("m2.5 8.5 4 4 7-9"));
}

/// The hero demo's group state sharing: checking a second box ticks only that child
/// (each checkbox owns its own state; the group's shared value is what ties them
/// together), and re-clicking untick it — the real port's funnel driven by a real
/// activating click.
#[wasm_bindgen_test]
async fn checkbox_group_hero_demo_shares_state_through_the_group() {
    let container = checkbox_group_container("test-mount-root-checkbox-group-hero-toggle");

    use crate::pages::checkbox_group_page::CheckboxGroupHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxGroupHeroDemo /> },
    ));
    flush_one_turn().await;

    let controls = els(&container, "[role=\"checkbox\"]");
    let gala = controls[1]
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    let fuji = controls[0].clone();

    gala.click();
    settle_checkbox_group().await;

    assert_eq!(
        controls[1].get_attribute("aria-checked").as_deref(),
        Some("true"),
        "the click ticked the gala checkbox"
    );
    assert!(controls[1].has_attribute("data-checked"));
    assert_eq!(
        controls[0].get_attribute("aria-checked").as_deref(),
        Some("true"),
        "the pre-ticked fuji checkbox is untouched"
    );
    assert_eq!(
        container
            .query_selector_all("span[class=\"flex data-unchecked:hidden\"]")
            .expect("query indicators")
            .length(),
        2,
        "both ticked checkboxes mount their Indicator"
    );

    // Toggling back: the Control is a span, so the click funnels through the hidden
    // input's change event (the unit's single state funnel).
    gala.click();
    settle_checkbox_group().await;
    assert_eq!(
        controls[1].get_attribute("aria-checked").as_deref(),
        Some("false"),
        "the second click unticked it again"
    );
    assert!(controls[1].has_attribute("data-unchecked"));
}

/// The parent demo's controlled recipe end to end — the demo state the page holds as a
/// leptos signal, its rg-0.2 mirror (the group's controlled source), the group's parent
/// engine, and the state-driven Indicator `render` callback:
///
/// 1. with nothing ticked the parent is `aria-checked="false"` (0 of 3) and its
///    Indicator is unmounted (the checkbox mount gate);
/// 2. ticking ONE child makes the group's aggregate partial → the parent becomes
///    `aria-checked="mixed"` and its Indicator renders the horizontal rule
///    (`state.indeterminate`, `demos/parent/css-modules/index.tsx:26-28`);
/// 3. ticking the remaining two → all → the parent becomes `aria-checked="true"` and
///    the callback renders the checkmark again.
#[wasm_bindgen_test]
async fn checkbox_group_parent_demo_drives_the_parent_tri_state() {
    let container = checkbox_group_container("test-mount-root-checkbox-group-parent");

    use crate::pages::checkbox_group_page::CheckboxGroupParentDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxGroupParentDemo /> },
    ));
    flush_one_turn().await;

    let slot = container
        .query_selector("[data-demo=\"parent\"]")
        .expect("query slot");
    let controls = els(&container, "[role=\"checkbox\"]");
    // The parent is the demo's first checkbox (`:22-32`).
    assert_eq!(
        controls.len(),
        4,
        "the parent demo renders parent + 3 items"
    );
    assert_eq!(
        controls[0].get_attribute("aria-checked").as_deref(),
        Some("false"),
        "an empty group leaves the parent unchecked"
    );
    let _ = &slot;
    assert!(
        container
            .query_selector("span[class=\"Indicator\"]")
            .expect("query indicator")
            .is_none(),
        "an unchecked, non-indeterminate parent mounts no Indicator"
    );

    // One child ticked → the group's aggregate goes partial.
    controls[1]
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .click();
    settle_checkbox_group().await;

    assert_eq!(
        controls[0].get_attribute("aria-checked").as_deref(),
        Some("mixed"),
        "one of three ticked makes the parent's aria-checked mixed"
    );
    assert!(
        controls[0].has_attribute("data-indeterminate"),
        "the indeterminate hook is present for the demo's stylesheet"
    );
    let parent_indicator = container
        .query_selector("span[class=\"Indicator\"]")
        .expect("query indicator")
        .expect("the mixed parent mounts its Indicator");
    let line = parent_indicator
        .query_selector("line")
        .expect("query line")
        .expect("the render callback swapped in the horizontal rule while indeterminate");
    assert_eq!(line.get_attribute("x1").as_deref(), Some("3"));
    assert_eq!(line.get_attribute("x2").as_deref(), Some("21"));

    // All three ticked → the parent goes to checked and the callback swaps back.
    controls[2]
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .click();
    settle_checkbox_group().await;
    controls[3]
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .click();
    settle_checkbox_group().await;

    assert_eq!(
        controls[0].get_attribute("aria-checked").as_deref(),
        Some("true"),
        "all children ticked makes the parent checked"
    );
    assert!(!controls[0].has_attribute("data-indeterminate"));
    let parent_indicator = container
        .query_selector("span[class=\"Indicator\"]")
        .expect("query indicator")
        .expect("the checked parent still mounts its Indicator");
    assert!(
        parent_indicator
            .query_selector("path")
            .expect("query path")
            .is_some(),
        "the callback renders the checkmark again once the parent is no longer mixed"
    );
}

/// The mirrored page structure (`page.mdx`): the h1, the `<Subtitle>`, the hero demo
/// before the first heading, every heading in document order (including the two demo
/// sections), the inline snippets, the three live demo slots in page order, and the
/// API-reference prose echoing the generated `TypesCheckboxGroup` table.
#[wasm_bindgen_test]
fn checkbox_group_page_component_renders_the_full_page_structure() {
    let container = checkbox_group_container("test-mount-root-checkbox-group-page");

    use crate::pages::checkbox_group_page::CheckboxGroupPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <CheckboxGroupPage /> },
    ));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Checkbox Group</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("Provides shared state to a series of checkboxes."),
        "the subtitle did not render"
    );
    for heading in [
        "Usage guidelines",
        "Anatomy",
        "Examples",
        "Labeling a checkbox group",
        "Rendering as a native button",
        "Form integration",
        "Parent checkbox",
        "Nested parent checkbox",
        "API reference",
        "CheckboxGroup",
        "CheckboxGroup.Props",
        "CheckboxGroup.State",
        "CheckboxGroup.ChangeEventReason",
        "CheckboxGroup.ChangeEventDetails",
        "Canonical types",
    ] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    // The Anatomy and Examples snippets, verbatim.
    assert!(
        html.contains("@base-ui/react/checkbox-group"),
        "the Anatomy import snippet did not render"
    );
    assert!(
        html.contains("protocols-label"),
        "the labelling snippet did not render"
    );
    assert!(
        html.contains("allValues"),
        "the parent-checkbox recipe didn't reference allValues"
    );
    assert!(
        html.contains("allowedNetworkProtocols"),
        "the form-integration snippet did not render"
    );

    // All three live demos mounted real parts, in page order.
    let hero_at = html.find("data-demo=\"hero\"").expect("hero slot");
    let parent_at = html.find("data-demo=\"parent\"").expect("parent slot");
    let nested_at = html.find("data-demo=\"nested\"").expect("nested slot");
    let guidelines_at = html
        .find("<h2>Usage guidelines</h2>")
        .expect("Usage guidelines heading");
    assert!(
        hero_at < guidelines_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );
    assert!(
        hero_at < parent_at && parent_at < nested_at,
        "the demos render in page order"
    );
    for demo in ["hero", "parent", "nested"] {
        let slot = container
            .query_selector(&format!("[data-demo='{demo}']"))
            .expect("query slot")
            .expect("the demo slot rendered");
        // The control's `role` lands in a post-mount writer Effect, so a synchronous
        // structure test asserts on the statically-rendered hidden input (the checkbox
        // page's structure test does the same); the async demo tests above cover the
        // attribute surface.
        assert!(
            slot.query_selector("input[type=\"checkbox\"]")
                .expect("query input")
                .is_some(),
            "the '{demo}' demo did not render real Checkbox controls"
        );
    }
}

// ─── The OTP Field docs page (`docs-content: components/otp-field`) ──────────
//
// The page's six demos are built as real DOM under the components' owners (the
// separator/button page convention), so these tests mount the demo fns directly
// and assert the DOM the port produced. `specs/docs-content/otp-field/page.md`
// is the page-shape oracle and `demos.json` the demo oracle; the assertions
// below cite the upstream lines they mirror.

/// Types `text` into `input` as a real keystroke: set the value, then dispatch
/// the `input` event the port's write path listens for (`OTPFieldInput.tsx`'s
/// onChange attaches to `input`).
fn type_into(input: &web_sys::HtmlInputElement, text: &str) {
    input.set_value(text);
    let init = web_sys::EventInit::new();
    init.set_bubbles(true);
    let event = web_sys::Event::new_with_event_init_dict("input", &init).expect("input event");
    input.dispatch_event(&event).expect("dispatch input");
}

/// The OTP slots of a mounted page demo, as real inputs.
fn otp_slots(container: &web_sys::HtmlElement) -> Vec<web_sys::HtmlInputElement> {
    let list = container.query_selector_all("input").expect("query slots");
    (0..list.length())
        .map(|index| {
            list.item(index)
                .expect("slot at index")
                .dyn_into::<web_sys::HtmlInputElement>()
                .expect("slot as input")
        })
        .collect()
}

/// The OTP Field page's structure: the whole mirrored page, in document order.
#[wasm_bindgen_test]
fn otp_field_page_component_renders_the_full_page_structure() {
    use crate::pages::otp_field_page::OtpFieldPage;
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-otp-field-page");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <OtpFieldPage /> },
    ));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>OTP Field</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A one-time password input composed of individual character slots."),
        "the subtitle did not render"
    );
    for heading in [
        "Usage guidelines",
        "Anatomy",
        "Examples",
        "Labeling an OTP field",
        "Form integration",
        "Alphanumeric verification codes",
        "Grouped layouts",
        "Placeholder hints",
        "Custom normalization",
        "Masked entry",
        "API reference",
        "Root",
        "Root.Props",
        "Root.State",
        "Root.ValidationType",
        "Root.ChangeEventReason",
        "Root.ChangeEventDetails",
        "Root.InvalidEventReason",
        "Root.InvalidEventDetails",
        "Root.CompleteEventReason",
        "Root.CompleteEventDetails",
        "Input",
        "Input.Props",
        "Input.State",
        "Separator",
        "Separator.Props",
        "Separator.State",
        "Canonical types",
    ] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    // The three embedded snippets, verbatim (`page.mdx:22-29`, `:41-54`, `:60-75`).
    assert!(
        html.contains("@base-ui/react/otp-field"),
        "the Anatomy import snippet did not render"
    );
    assert!(
        html.contains("verification-code-description"),
        "the labeling snippet did not render"
    );
    assert!(
        html.contains("Enter the 6-character code we sent to your device."),
        "the form-integration snippet's description did not render"
    );

    // The hero renders BEFORE the first heading (`page.mdx:10-12`), and the six
    // demo slots appear in page order.
    let hero_at = html.find("data-demo=\"hero\"").expect("hero slot");
    let guidelines_at = html
        .find("<h2>Usage guidelines</h2>")
        .expect("Usage guidelines heading");
    assert!(
        hero_at < guidelines_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );
    let order: Vec<usize> = [
        "hero",
        "alphanumeric",
        "grouped",
        "focused-placeholder",
        "custom-sanitize",
        "password",
    ]
    .iter()
    .map(|demo| {
        html.find(&format!("data-demo=\"{demo}\""))
            .unwrap_or_else(|| panic!("the {demo} demo slot did not render"))
    })
    .collect();
    assert!(
        order.windows(2).all(|pair| pair[0] < pair[1]),
        "the six demos render in page order; positions were {order:?}"
    );

    // Every demo slot mounted real OTP slots (the port's single input part).
    for demo in [
        "hero",
        "alphanumeric",
        "grouped",
        "focused-placeholder",
        "custom-sanitize",
        "password",
    ] {
        let slot = container
            .query_selector(&format!("[data-demo='{demo}']"))
            .expect("query slot")
            .expect("the demo slot rendered");
        assert_eq!(
            slot.query_selector_all("input")
                .expect("query inputs")
                .length(),
            6,
            "the '{demo}' demo did not render six real OTP slots"
        );
    }
}

/// The hero demo's real composition: the root's own bag (role, class, id, the
/// description link), the derived slot ids, and the first-slot aria rule
/// (`hero/tailwind/index.tsx:15-29`).
#[wasm_bindgen_test]
fn otp_field_hero_demo_renders_the_real_part_composition() {
    use crate::pages::otp_field_page::otp_field_hero_demo;
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-otp-field-hero");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, otp_field_hero_demo));

    let root = container
        .query_selector("div[role='group']")
        .expect("query root")
        .expect("the port's root rendered (role=group comes from its own bag)");
    // The `id` prop is the FIRST INPUT's id (types.md:33), not the root's: the
    // root's own bag carries role/aria-* only (`otp_field.rs:849-858`).
    let slots = otp_slots(&container);
    assert_eq!(slots.len(), 6, "the hero renders six slots");
    let root_id = slots[0]
        .get_attribute("id")
        .expect("the first slot carries the root's id prop");
    assert_eq!(
        root.get_attribute("class").as_deref(),
        Some("flex w-full gap-2"),
        "the root's demo className is carried verbatim"
    );
    let described_by = root.get_attribute("aria-describedby");
    assert!(
        described_by.as_deref() == Some(format!("{root_id}-description").as_str()),
        "the root forwards aria-describedby (its own bag, OTPFieldRoot.tsx:379-391); \
         root_id={root_id:?} actual={described_by:?} \
         label_for={:?}",
        container
            .query_selector("label")
            .expect("query label")
            .and_then(|label| label.get_attribute("for"))
    );

    let label = container
        .query_selector("label")
        .expect("query label")
        .expect("the demo's native label rendered");
    assert_eq!(
        label.get_attribute("for").as_deref(),
        Some(root_id.as_str()),
        "the label's htmlFor is the root id (`page.mdx:35`: the shared accessible name)"
    );
    assert_eq!(label.text_content().as_deref(), Some("Verification code"));

    let description = container
        .query_selector(&format!("p[id='{root_id}-description']"))
        .expect("query description")
        .expect("the supporting description rendered with the generated id");
    assert_eq!(
        description.text_content().as_deref(),
        Some("Enter the 6-character code we sent to your device.")
    );

    let slots = otp_slots(&container);
    assert_eq!(slots.len(), 6, "the hero renders six slots");

    for (index, slot) in slots.iter().enumerate() {
        let expected_id = if index == 0 {
            root_id.clone()
        } else {
            format!("{root_id}-{}", index + 1)
        };
        assert_eq!(
            slot.get_attribute("id").as_deref(),
            Some(expected_id.as_str()),
            "slot {index} derives its id from the root id (types.md:33, `get_input_id`)"
        );
        // The first slot relies on the shared label; later slots announce their
        // position (`hero/tailwind/index.tsx:23` + behavior.md "Accessibility").
        let expected_label = if index == 0 {
            None
        } else {
            Some(format!("Character {} of 6", index + 1))
        };
        assert_eq!(
            slot.get_attribute("aria-label"),
            expected_label,
            "slot {index}'s aria-label follows the first-slot rule"
        );
    }
    // The port's own input bag (`OTPFieldInput.tsx:91-176`): only the first slot
    // advertises the one-time-code autocomplete; the rest turn it off.
    assert_eq!(
        slots[0].get_attribute("autocomplete").as_deref(),
        Some("one-time-code"),
        "the first slot carries the port's default autoComplete"
    );
    assert_eq!(
        slots[1].get_attribute("autocomplete").as_deref(),
        Some("off"),
        "later slots disable autocomplete (the port's own bag)"
    );
    assert_eq!(
        slots[0].get_attribute("inputmode").as_deref(),
        Some("numeric"),
        "`validationType=\"numeric\"` (the default) supplies the numeric inputMode"
    );
}

/// The grouped demo: wrapper elements around subsets of slots do not disturb the
/// slot registry, and the shared Separator sits between the groups
/// (`grouped/tailwind/index.tsx:11-38`; behavior.md "DOM structure").
#[wasm_bindgen_test]
fn otp_field_grouped_demo_keeps_slot_registration_across_wrapper_elements() {
    use crate::pages::otp_field_page::otp_field_grouped_demo;
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-otp-field-grouped");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, otp_field_grouped_demo));

    let root = container
        .query_selector("div[role='group']")
        .expect("query root")
        .expect("the grouped demo's root rendered");
    assert_eq!(
        root.get_attribute("class").as_deref(),
        Some("flex w-full items-center gap-2"),
        "the grouped root's className is carried verbatim"
    );

    // The two layout wrappers, each holding three slots.
    let group_list = root
        .query_selector_all("div.flex.gap-2")
        .expect("query groups");
    assert_eq!(
        group_list.length(),
        2,
        "the demo wraps its slots in two layouts"
    );
    for index in 0..group_list.length() {
        let group = group_list
            .item(index)
            .expect("group at index")
            .dyn_into::<web_sys::Element>()
            .expect("group as element");
        assert_eq!(
            group.query_selector_all("input").expect("inputs").length(),
            3,
            "each layout holds three slots"
        );
    }

    let separator = root
        .query_selector("[role='separator']")
        .expect("query separator")
        .expect("the real Separator rendered between the groups");
    assert_eq!(
        separator.get_attribute("class").as_deref(),
        Some("h-px w-4 bg-current text-neutral-950 dark:text-white"),
        "the Separator's demo className is carried verbatim"
    );

    // The slots still number 0-5 in source order despite the wrappers, so their
    // derived ids are unbroken (`get_input_id`), and the second group's slots
    // announce positions 4-6 (`:33`).
    let slots = otp_slots(&container);
    assert_eq!(slots.len(), 6, "six slots total");
    let root_id = slots[0]
        .get_attribute("id")
        .expect("the first slot carries the root's id prop (types.md:33)");

    for (index, slot) in slots.iter().enumerate() {
        let expected_id = if index == 0 {
            root_id.clone()
        } else {
            format!("{root_id}-{}", index + 1)
        };
        let actual_id = slot.get_attribute("id");
        assert!(
            actual_id.as_deref() == Some(expected_id.as_str()),
            "wrapper elements must not disturb slot {index}'s registration; \
             expected={expected_id:?} actual={actual_id:?}"
        );
    }
    assert_eq!(
        slots[4].get_attribute("aria-label").as_deref(),
        Some("Character 5 of 6"),
        "the second group's slots announce their positions"
    );
}

/// The mask and placeholder demos: `mask` renders every slot as
/// `input[type="password"]`, and the native `placeholder` reaches the real input
/// through the port's `...elementProps` rest (demos.json entries 6 and 3).
#[wasm_bindgen_test]
fn otp_field_password_and_placeholder_demos_follow_their_props() {
    use crate::pages::otp_field_page::{
        otp_field_focused_placeholder_demo, otp_field_password_demo,
    };

    for (name, demo) in [
        (
            "password",
            otp_field_password_demo as fn() -> crate::pages::use_render_page::RawElementView,
        ),
        (
            "focused-placeholder",
            otp_field_focused_placeholder_demo
                as fn() -> crate::pages::use_render_page::RawElementView,
        ),
    ] {
        let container = leptos::prelude::document()
            .create_element("div")
            .expect("create container")
            .dyn_into::<web_sys::HtmlElement>()
            .expect("div as HtmlElement");
        container.set_id(&format!("test-mount-root-otp-field-{name}"));
        leptos::prelude::document()
            .body()
            .expect("body")
            .append_child(&container)
            .expect("append container");

        use leptos::mount::mount_to;
        use leptos::prelude::*;

        any_spawner::Executor::init_futures_executor();
        std::mem::forget(mount_to({ container.clone() }, demo));

        let slots = otp_slots(&container);
        assert_eq!(slots.len(), 6, "the '{name}' demo renders six slots");
        for slot in slots.iter() {
            match name {
                "password" => assert_eq!(
                    slot.get_attribute("type").as_deref(),
                    Some("password"),
                    "`mask` renders every slot as a password input (types.md `mask`)"
                ),
                _ => assert_eq!(
                    slot.get_attribute("placeholder").as_deref(),
                    Some("•"),
                    "the native placeholder rode the port's elementProps rest"
                ),
            }
        }
    }
}

/// The port's write path accumulates characters across slots: a keystroke in a
/// later slot keeps the earlier ones (`OTPFieldInput.tsx:161-171`'s
/// `replaceOTPValue(previousValue, index, digits)`). This is the slot-entry
/// behavior every demo on the page stands on.
#[wasm_bindgen_test]
async fn otp_field_slots_accumulate_characters_across_slots() {
    use crate::pages::otp_field_page::otp_field_alphanumeric_demo;
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-otp-field-accumulate");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, otp_field_alphanumeric_demo));
    flush_one_turn().await;

    let slots = otp_slots(&container);
    type_into(&slots[0], "A");
    flush_one_turn().await;
    type_into(&slots[1], "b");
    flush_one_turn().await;

    // Upstream's committed value after two single-character entries is "Ab"
    // (slot 0 = A, slot 1 = b) — so the completed-value probe the page uses
    // reports both characters. The port's write path reads the value it holds,
    // so a stale read would drop "A".
    assert_eq!(
        slots[0].value(),
        "A",
        "the first character must survive the second keystroke"
    );
}

/// The custom-normalization demo end to end: a real keystroke through the port's
/// write path, `normalizeValue`'s uppercasing on the committed value, and the
/// rejected-character report through `onValueInvalid` — the demo's `aria-live`
/// message plus its alternating highlight class on the focused slot
/// (`custom-sanitize/css-modules/index.tsx`, `useInvalidFeedback.ts:38-51`).
#[wasm_bindgen_test]
async fn otp_field_custom_sanitize_demo_normalizes_and_reports_rejected_characters() {
    use crate::pages::otp_field_page::otp_field_custom_sanitize_demo;
    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-otp-field-sanitize");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        otp_field_custom_sanitize_demo,
    ));
    flush_one_turn().await;

    let status = || {
        container
            .query_selector("[aria-live]")
            .expect("query status")
            .expect("the demo's aria-live status span rendered")
    };
    assert_eq!(
        status().text_content().as_deref(),
        Some(""),
        "the demo starts with no feedback (`useInvalidFeedback.ts:5-9`)"
    );

    // A valid alphanumeric character, typed into the FOCUSED slot (the demo's
    // pulse targets the focused index, so focus it first): accepted, and
    // `normalizeValue` uppercases it before the state update
    // (`page.mdx:109-112`).
    let slots = otp_slots(&container);
    slots[0].focus().expect("focus slot 0");
    flush_one_turn().await;

    // The rejected-character arm FIRST, on a pristine field: `_` is outside
    // `[a-zA-Z0-9]`, so the port's own write path reports the attempted string
    // through `onValueInvalid` (`OTPFieldRoot.tsx`'s reportValueInvalid) and the
    // demo publishes the message plus the pulse. This is the direct evidence that
    // the port attached its `input` listener at all.
    type_into(&slots[0], "_");
    flush_one_turn().await;
    let invalid_status = status().text_content();
    assert!(
        invalid_status.as_deref() == Some("Unsupported characters were ignored from _."),
        "the rejected character is announced with the attempted value; \
         actual={invalid_status:?} slot0={:?} html={}",
        slots[0].value(),
        container.inner_html()
    );
    assert_eq!(
        slots[0].get_attribute("class").as_deref(),
        Some("Input InputInvalidA"),
        "the focused slot carries the odd-pulse highlight class (`index.tsx:44-48`)"
    );
    // The rejected keystroke left the value alone (the port restores the slot's
    // own character on the reject arm, `OTPFieldInput.tsx:153-156`).
    assert_eq!(
        slots[0].value(),
        "",
        "a rejected character must not enter the value"
    );

    // Now an accepted character: `normalizeValue`'s uppercasing is what reaches
    // the committed value, and the demo reflects it back onto the slot.
    type_into(&slots[0], "a");
    flush_one_turn().await;
    let after_first = slots[0].value();
    assert!(
        after_first == "A",
        "normalizeValue uppercased the committed value onto the slot; \
         actual={after_first:?} status={:?} html={}",
        status().text_content(),
        container.inner_html()
    );
    // The demo's hook KEEPS the message across this one change: the invalid report
    // ARMS `skipClearOnNextValueChangeRef` and the next value change consumes it
    // without clearing (`useInvalidFeedback.ts:29-36`, armed at `:38-41`) — so the
    // first accepted character after a rejection does not clear the feedback.
    assert_eq!(
        status().text_content().as_deref(),
        Some("Unsupported characters were ignored from _."),
        "the armed skip swallows the first accepted change's clear"
    );

    // The next accepted change is the one that clears it — the same contract seen
    // from the other side, and the direct evidence that the port's `onValueChange`
    // reaches the demo at all (`useInvalidFeedback.ts:35`).
    type_into(&slots[1], "b");
    flush_one_turn().await;
    assert_eq!(
        status().text_content().as_deref(),
        Some(""),
        "the following accepted change clears the feedback; slot1={:?}",
        slots[1].value()
    );
}

/// The pair's write-path regression: the page's composition helpers must leave the
/// port's OWN `input` write path attached — `onChange`/`onPaste` are attached inside
/// the Input's ref callback (`otp_field.rs:1356-1560`), whose keep-alive is the ref
/// fork inside the `RenderedElement` the helper materializes. Mounts the same
/// `otp_root` + `append_slot` helper the page's demos use, with instrumented
/// handlers, and types one accepted character. A control listener on the same node
/// proves the dispatch reaches the node's listeners at all, so an empty
/// `onValueChange` can only mean the port's own listener is gone.
#[wasm_bindgen_test]
async fn otp_field_composition_attaches_the_ports_write_path() {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use leptos_ui::{OtpFieldRootProps, OtpValidationType};

    use crate::pages::otp_field_page::{append_slot, otp_root};

    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-otp-field-probe");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    any_spawner::Executor::init_futures_executor();

    let committed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let rejected: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let dispatched = Rc::new(Cell::new(0u32));

    let committed_for_change = Rc::clone(&committed);
    let rejected_for_invalid = Rc::clone(&rejected);
    let owner = reactive_graph::owner::Owner::new();
    let root = owner.with(move || {
        otp_root(
            OtpFieldRootProps {
                length: 6,
                id: Some("probe-otp".to_string()),
                validation_type: OtpValidationType::Alphanumeric,
                on_value_change: Some(Rc::new(move |value: &str, _| {
                    committed_for_change.borrow_mut().push(value.to_string());
                })),
                on_value_invalid: Some(Rc::new(move |value: &str, _| {
                    rejected_for_invalid.borrow_mut().push(value.to_string());
                })),
                ..OtpFieldRootProps::default()
            },
            |root_node| {
                for index in 0..6 {
                    append_slot(root_node, index, "Input", None, Vec::new(), None);
                }
            },
        )
    });
    std::mem::forget(owner);
    container.append_child(&root).expect("append root");

    let slots = otp_slots(&container);

    // A control listener on the same node proves the dispatched event reaches
    // listeners at all (so a silent port handler is not a dispatch artifact).
    let dispatched_for_probe = Rc::clone(&dispatched);
    let control = leptos::wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
        move |_event: web_sys::Event| dispatched_for_probe.set(dispatched_for_probe.get() + 1),
    );
    slots[0]
        .add_event_listener_with_callback("input", control.as_ref().unchecked_ref())
        .expect("attach control listener");
    control.forget();

    type_into(&slots[0], "a");
    flush_one_turn().await;

    assert_eq!(
        dispatched.get(),
        1,
        "the `input` event reached the node's listeners"
    );
    assert!(
        !committed.borrow().is_empty(),
        "the port's write path must commit through `set_value`; committed={:?} rejected={:?} \
         slot0={:?}",
        committed.borrow(),
        rejected.borrow(),
        slots[0].value()
    );
}

// The fieldset docs page (`docs-content: components/fieldset`)
// ---------------------------------------------------------------------------

/// The upstream hero demo's class strings, carried verbatim by the page
/// (`docs/src/app/(docs)/react/components/fieldset/demos/hero/tailwind/index.tsx:6-27`).
const FIELDSET_HERO_ROOT_CLASS: &str = "flex w-full max-w-64 flex-col gap-4";
const FIELDSET_HERO_LEGEND_CLASS: &str = "border-b border-neutral-950 text-base font-bold text-neutral-950 dark:border-white dark:text-white";
const FIELDSET_HERO_FIELD_CLASS: &str = "flex flex-col items-start gap-1";
const FIELDSET_HERO_LABEL_CLASS: &str = "text-sm font-bold text-neutral-950 dark:text-white";
const FIELDSET_HERO_CONTROL_CLASS: &str = "h-8 w-full border border-neutral-950 bg-white dark:bg-neutral-950 px-2 text-sm any-pointer-coarse:text-base font-normal text-neutral-950 placeholder:text-neutral-500 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white dark:border-white dark:text-white dark:placeholder:text-neutral-400";

/// A fresh mount container for a fieldset-page test.
fn fieldset_container(id: &str) -> web_sys::HtmlElement {
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

/// The hero demo (`demos/hero/tailwind/index.tsx:4-32`, the single demos.json
/// entry): upstream's exact element composition rendered through the REAL
/// `leptos_ui` parts — the native `<fieldset>` root carrying the demo class, the
/// legend `<div>` upstream styles in place of a native `<legend>`, and the two
/// Field.Root wrappers each holding a label and an uncontrolled input with the
/// upstream `placeholder` passed through the part's `...elementProps` rest.
#[wasm_bindgen_test]
async fn fieldset_hero_demo_renders_the_real_part_composition() {
    let container = fieldset_container("test-mount-root-fieldset-hero");

    use crate::pages::fieldset_page::FieldsetHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FieldsetHeroDemo /> },
    ));
    // The root/legend class and id are written by the parts' bag writer (the
    // post-mount effect), so they only exist on the node after an executor turn.
    flush_one_turn().await;

    // `Fieldset.Root` renders a REAL native `<fieldset>` (behavior.md "DOM
    // structure & portal behavior"), with the upstream className verbatim.
    let root = container
        .query_selector("fieldset")
        .expect("query fieldset")
        .expect("the demo renders the real Fieldset.Root as a <fieldset>");
    assert_eq!(root.tag_name(), "FIELDSET");
    assert_eq!(
        root.get_attribute("class").as_deref(),
        Some(FIELDSET_HERO_ROOT_CLASS),
        "the root carries the upstream className verbatim"
    );

    // `Fieldset.Legend` renders upstream's `<div>` — deliberately NOT a native
    // `<legend>` — so it is the root's only `div` child.
    let legend = root
        .query_selector("div")
        .expect("query legend")
        .expect("the demo renders the real Fieldset.Legend");
    assert_eq!(legend.tag_name(), "DIV");
    assert_eq!(
        legend.get_attribute("class").as_deref(),
        Some(FIELDSET_HERO_LEGEND_CLASS),
        "the legend carries the upstream className verbatim (dark: variants included)"
    );
    assert_eq!(legend.text_content().as_deref(), Some("Billing details"));

    // The two fields, in upstream's order, each a Field.Root wrapper with the
    // demo class holding a label and a control.
    let labels = els(&root, "label");
    assert_eq!(labels.len(), 2, "the hero demo renders two labelled fields");
    assert_eq!(
        labels
            .iter()
            .map(|label| label.text_content().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["Company", "Tax ID"],
        "the labels render in upstream's order with their text"
    );
    for label in &labels {
        assert_eq!(
            label.get_attribute("class").as_deref(),
            Some(FIELDSET_HERO_LABEL_CLASS)
        );
        // The Field.Root wrapper around this field.
        let field = label
            .parent_element()
            .expect("the label sits inside its Field.Root");
        assert_eq!(
            field.get_attribute("class").as_deref(),
            Some(FIELDSET_HERO_FIELD_CLASS),
            "each field is wrapped by a Field.Root carrying the upstream className"
        );
        assert_eq!(field.tag_name(), "DIV");
    }

    // The two uncontrolled controls, with the upstream `placeholder`s (upstream
    // writes them as bare JSX attributes — the `...elementProps` rest).
    let inputs = els(&root, "input");
    assert_eq!(inputs.len(), 2, "the hero demo renders two Field.Controls");
    assert_eq!(
        inputs
            .iter()
            .map(|input| input.get_attribute("placeholder").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["Enter company name", "Enter fiscal number"],
        "the demo's placeholders ride the part's elementProps rest"
    );
    for input in &inputs {
        assert_eq!(
            input.get_attribute("class").as_deref(),
            Some(FIELDSET_HERO_CONTROL_CLASS),
            "the control carries the upstream className verbatim"
        );
        // Upstream's hero demo passes no `type` (it is an ordinary text input);
        // the port invents no attribute either, so the DOM property — not an
        // attribute — is what carries the behavior the demo relies on.
        assert!(
            input.get_attribute("type").is_none(),
            "the demo's control sets no type attribute (upstream sets none), got {:?}",
            input.get_attribute("type")
        );
        let input: &web_sys::HtmlInputElement = &input.clone().unchecked_into();
        assert_eq!(
            input.type_(),
            "text",
            "Field.Control behaves as a text input in the browser"
        );
    }
}

/// demos.json's one `nonTrivialInteractions` entry, asserted as a live
/// consequence rather than a static attribute: rendering `Fieldset.Legend`
/// inside `Fieldset.Root` "exercises the automatic aria-labelledby linking
/// between the fieldset root and its legend, so the grouped fields are
/// announced with the 'Billing details' legend". The id is generated by the
/// ported `useBaseUiId` (hence the `base-ui-` prefix) and reaches the root
/// through the legend's `use_registered_label_id` registration.
#[wasm_bindgen_test]
async fn fieldset_hero_demo_links_the_root_to_its_legend() {
    let container = fieldset_container("test-mount-root-fieldset-hero-a11y");

    use crate::pages::fieldset_page::FieldsetHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FieldsetHeroDemo /> },
    ));
    flush_one_turn().await;

    let root = container
        .query_selector("fieldset")
        .expect("query fieldset")
        .expect("the demo renders the real Fieldset.Root");
    let legend = root
        .query_selector("div")
        .expect("query legend")
        .expect("the demo renders the real Fieldset.Legend");

    let legend_id = legend
        .get_attribute("id")
        .expect("the legend rendered an id to be labelled by");
    assert!(
        legend_id.starts_with("base-ui-"),
        "the generated id carries the base-ui prefix, got {legend_id:?}"
    );
    assert_eq!(
        root.get_attribute("aria-labelledby").as_deref(),
        Some(legend_id.as_str()),
        "the root's aria-labelledby points at the legend's registered id"
    );

    // Upstream's element order: the legend is the fieldset's FIRST child
    // (`hero/tailwind/index.tsx:7-11`), so the labelling element precedes the
    // fields it groups.
    assert_eq!(
        root.first_element_child().as_ref(),
        Some(&legend),
        "the legend must be the first child of the fieldset (upstream's element order)"
    );
}

/// The mirrored page structure: the h1 + subtitle, the hero demo BEFORE the
/// first heading (`page.mdx:10-12`), the two headings in order, the single
/// Anatomy snippet, and the API-reference prose echoed from the generated
/// `TypesFieldset` tables.
#[wasm_bindgen_test]
fn fieldset_page_component_renders_the_full_page_structure() {
    let container = fieldset_container("test-mount-root-fieldset-page");

    use crate::pages::fieldset_page::FieldsetPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FieldsetPage /> },
    ));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Fieldset</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A native fieldset element with an easily stylable legend."),
        "the subtitle did not render"
    );
    for heading in ["Anatomy", "API reference", "Root", "Legend"] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    // The full heading set, in order — the page's own headings plus the four
    // additional-type headings the generated reference tables emit on upstream's
    // rendered page (`Fieldset.Root.Props` …`Fieldset.Legend.State`), which
    // `specs/docs-content/fieldset/page.md`'s heading list does not record (see
    // the note appended to ralph/logs/spec-discrepancies.md).
    let headings: Vec<String> = els(container.as_ref(), "h1, h2, h3")
        .into_iter()
        .map(|h| h.text_content().unwrap_or_default())
        .collect();
    assert_eq!(
        headings,
        vec![
            "Fieldset",
            "Anatomy",
            "API reference",
            "Root",
            "Fieldset.Root.Props",
            "Fieldset.Root.State",
            "Legend",
            "Fieldset.Legend.Props",
            "Fieldset.Legend.State",
        ],
        "the page must render the same heading structure upstream's page does"
    );
    // The single fenced Anatomy snippet (`page.mdx:18-24`), asserted through the
    // element's textContent: inner_html escapes `<`/`>` in text nodes, so the
    // serialized form is not the right place to look for the JSX itself.
    let code = container
        .query_selector("pre code")
        .expect("query pre code")
        .expect("the Anatomy snippet rendered");
    let snippet = code.text_content().unwrap_or_default();
    assert!(
        snippet.contains("import { Fieldset } from '@base-ui/react/fieldset';"),
        "the Anatomy import line did not render; snippet was: {snippet:?}"
    );
    assert!(
        snippet.contains("<Fieldset.Root>")
            && snippet.contains("<Fieldset.Legend />")
            && snippet.contains("</Fieldset.Root>;"),
        "the Anatomy snippet's assembly did not render; snippet was: {snippet:?}"
    );

    // The hero demo slot mounted the real part tree (the native fieldset).
    let hero = container
        .query_selector("[data-demo='hero']")
        .expect("query")
        .expect("the hero demo slot rendered");
    assert!(
        hero.query_selector("fieldset")
            .expect("query fieldset")
            .is_some(),
        "the live hero demo did not render its Fieldset.Root"
    );

    // The demo precedes the first heading (page.mdx document order).
    let demo_at = html.find("data-demo").expect("demo slot in html");
    let anatomy_at = html
        .find("<h2>Anatomy</h2>")
        .expect("Anatomy heading in html");
    assert!(
        demo_at < anatomy_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );

    // The API reference prose echoes the generated tables' content (static
    // prose, never fabricated machinery). Read through textContent: inner_html
    // escapes the angle brackets these summaries contain.
    let summaries = els(container.as_ref(), ".api-summary");
    assert_eq!(
        summaries.len(),
        4,
        "one summary per part, plus each additional type's re-export line"
    );
    let summary_text: Vec<String> = summaries
        .iter()
        .map(|node| node.text_content().unwrap_or_default())
        .collect();
    assert!(
        summary_text[0].contains(
            "Groups a shared legend with related controls. Renders a <fieldset> element."
        ),
        "the Root summary prose did not render; got {:?}",
        summary_text[0]
    );
    assert_eq!(
        summary_text[1], "Re-export of Root props.",
        "the Root.Props re-export line did not render (types.md \"Root.Props\")"
    );
    assert!(
        summary_text[2]
            .contains("An accessible label that is automatically associated with the fieldset."),
        "the Legend summary prose did not render; got {:?}",
        summary_text[2]
    );
    assert_eq!(
        summary_text[3], "Re-export of Legend props.",
        "the Legend.Props re-export line did not render (types.md \"Legend.Props\")"
    );
    let props = els(container.as_ref(), ".api-props");
    assert!(
        props[0]
            .text_content()
            .unwrap_or_default()
            .contains("Fieldset.Root.State"),
        "the Root props prose did not render"
    );
    let states = els(container.as_ref(), ".api-state");
    assert!(
        states[1]
            .text_content()
            .unwrap_or_default()
            .contains("Fieldset.Legend.State"),
        "the Legend state prose did not render"
    );
}

// The Form docs page (`docs-content: components/form`)
// ---------------------------------------------------------------------------
//
// The page's three demos are the upstream server-error demos, so the tests
// below drive the real submit pipeline end to end: a dispatched native `submit`
// through the port's injected listener, the field machinery's validity gate, the
// demo's fake server, and the late error record landing on the matching field's
// `Field.Error` slot. The delay is passed as a test-sized value (the button
// page's `reset_ms` precedent) so the pending state is observable.

/// The upstream class strings the page carries verbatim — asserted rather than
/// re-imported so a drift in the page's constants is a test failure, not a
/// silent divergence from the React source.
const FORM_PAGE_FORM_CLASS: &str = "flex w-full max-w-64 flex-col gap-4";
const FORM_PAGE_ERROR_CLASS: &str = "text-sm text-red-700 dark:text-red-400";

/// A fresh mount container for a form-page test.
fn form_container(id: &str) -> web_sys::HtmlElement {
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

/// The demo's mounted `<form>`.
fn form_in(container: &web_sys::HtmlElement) -> web_sys::HtmlFormElement {
    container
        .query_selector("form")
        .expect("query form")
        .expect("the demo rendered a <form>")
        .dyn_into::<web_sys::HtmlFormElement>()
        .expect("form as HtmlFormElement")
}

/// The demo's `Field.Control`, selected by its upstream `placeholder`.
fn form_control(container: &web_sys::HtmlElement, placeholder: &str) -> web_sys::HtmlInputElement {
    container
        .query_selector(&format!("input[placeholder='{placeholder}']"))
        .expect("query input")
        .expect("the demo rendered its Field.Control")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("input as HtmlInputElement")
}

/// The demo's single error slot (upstream's `.text-sm.text-red-700` div).
fn form_error_slot(container: &web_sys::HtmlElement) -> web_sys::Element {
    let slots = els(container.as_ref(), ".text-red-700");
    assert_eq!(slots.len(), 1, "the demo renders exactly one error slot");
    slots.into_iter().next().expect("slot")
}

/// The demo's submit button (the real ported `Button`).
fn form_submit_button(container: &web_sys::HtmlElement) -> web_sys::HtmlButtonElement {
    let buttons = els(container.as_ref(), "button");
    assert_eq!(buttons.len(), 1, "the demo renders exactly one button");
    buttons
        .into_iter()
        .next()
        .expect("button")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("button as HtmlButtonElement")
}

/// Dispatches a cancelable `submit` at the form — the crate suites' own idiom
/// (`Form.test.tsx`'s `fireEvent.submit` analog). Returns whether the event was
/// default-prevented, which is the submit pipeline's observable verdict.
fn dispatch_submit(form: &web_sys::HtmlFormElement) -> bool {
    let init = web_sys::EventInit::new();
    web_sys::EventInit::set_cancelable(&init, true);
    let event =
        web_sys::Event::new_with_event_init_dict("submit", &init).expect("submit event init");
    form.dispatch_event(&event).expect("dispatch submit");
    event.default_prevented()
}

/// The hero demo's composition: the real `Form`, the real `Field` parts and the
/// real ported `Button`, each carrying the upstream `className`/attributes
/// (demos.json `propsExercised`: `Form.errors`/`onSubmit`,
/// `Field.Control.type|required|defaultValue|placeholder|pattern`,
/// `Button.type|disabled|focusableWhenDisabled`).
#[wasm_bindgen_test]
async fn form_hero_demo_renders_the_real_part_composition() {
    let container = form_container("test-mount-root-form-hero");

    use crate::pages::form_page::FormHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FormHeroDemo delay_ms=1000 /> },
    ));
    flush_one_turn().await;

    // The ported Form: the <form> element, the upstream class, and the injected
    // noValidate (`Form.tsx:106-143` — the default ahead of elementProps).
    let form = form_in(&container);
    assert_eq!(
        form.get_attribute("class").as_deref(),
        Some(FORM_PAGE_FORM_CLASS),
        "the Form carries the upstream className verbatim"
    );
    assert!(
        form.has_attribute("novalidate"),
        "the port injects noValidate ahead of elementProps; html was: {}",
        container.inner_html()
    );

    // Field.Root + Field.Label + Field.Control: the label is associated with the
    // control and the control carries the elementProps rest.
    let input = form_control(&container, "https://example.com");
    assert_eq!(
        input.get_attribute("type").as_deref(),
        Some("url"),
        "the control's type rides the elementProps rest"
    );
    assert!(
        input.has_attribute("required"),
        "the control's required rides the elementProps rest"
    );
    assert_eq!(
        input.get_attribute("pattern").as_deref(),
        Some("https?://.*"),
        "the control's pattern rides the elementProps rest"
    );
    assert_eq!(
        input.value(),
        "https://example.com",
        "the uncontrolled control is seeded from defaultValue"
    );
    let label = container
        .query_selector("label")
        .expect("query label")
        .expect("Field.Label rendered a <label>");
    assert_eq!(label.text_content().as_deref(), Some("Homepage"));
    assert_eq!(
        label.get_attribute("for").as_deref(),
        input.get_attribute("id").as_deref(),
        "the label is automatically associated with the field control"
    );

    // Field.Error: mounted, hidden while no error has been reported (the port's
    // documented deviation from upstream's `return null`), and carrying the
    // upstream class.
    let error = form_error_slot(&container);
    assert_eq!(
        error.get_attribute("class").as_deref(),
        Some(FORM_PAGE_ERROR_CLASS),
        "the error slot carries the upstream className verbatim"
    );
    assert!(
        error.has_attribute("hidden"),
        "the pristine error slot is hidden; html was: {}",
        container.inner_html()
    );

    // The real ported Button: a native <button type="submit"> with the upstream
    // class and label.
    let button = form_submit_button(&container);
    assert_eq!(
        button.get_attribute("type").as_deref(),
        Some("submit"),
        "the elementProps rest overrides the engine's type=button default"
    );
    assert_eq!(button.text_content().as_deref(), Some("Submit"));
}

/// The hero demo's native-validity gate: an empty required control blocks the
/// submission (demos.json `nonTrivialInteractions[1]` — "Native constraint
/// validation (`required` plus `pattern=\"https?://.*\"` on a `type=\"url\"`
/// control) gates submission before the async handler runs"), the demo's
/// handler never runs, and the port's `focusFirstInvalid` moves focus to the
/// offending control (behavior.md "Focus management").
#[wasm_bindgen_test]
async fn form_hero_demo_blocks_the_submit_on_native_validity_and_focuses_the_control() {
    let container = form_container("test-mount-root-form-hero-gate");

    use crate::pages::form_page::FormHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FormHeroDemo delay_ms=10 /> },
    ));
    flush_one_turn().await;

    let form = form_in(&container);
    let input = form_control(&container, "https://example.com");
    // Clear the seeded value: `required` + an empty value is `valueMissing`.
    input.set_value("");

    assert!(
        dispatch_submit(&form),
        "the blocked submission must be prevented (Form.tsx:119-121)"
    );
    flush_one_turn().await;

    // The handler did not run: no pending state, no error message.
    let button = form_submit_button(&container);
    assert_ne!(
        button.get_attribute("aria-disabled").as_deref(),
        Some("true"),
        "the demo's handler never ran, so the button never entered its pending state"
    );
    assert!(
        form_error_slot(&container).has_attribute("hidden"),
        "a gated submission reports no server error"
    );
    assert!(
        input.validity().value_missing(),
        "the control really is natively invalid (the gate's premise)"
    );
    assert_eq!(
        leptos::prelude::document()
            .active_element()
            .and_then(|element| element.get_attribute("id")),
        input.get_attribute("id"),
        "the first invalid control took focus (`focusFirstInvalid`)"
    );
}

/// The hero demo's server-error path end to end: a valid submit reaches the
/// demo's `onSubmit` (which prevents the native submission itself,
/// `hero/tailwind/index.tsx:16`), the button enters its disabled-but-focusable
/// pending state, and the fake server's error lands on the field — visible, with
/// the message upstream renders — while the uncontrolled control keeps what the
/// user typed.
#[wasm_bindgen_test]
async fn form_hero_demo_surfaces_the_server_error_and_keeps_the_typed_value() {
    let container = form_container("test-mount-root-form-hero-error");

    use crate::pages::form_page::FormHeroDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FormHeroDemo delay_ms=10 /> },
    ));
    flush_one_turn().await;

    let form = form_in(&container);
    let input = form_control(&container, "https://example.com");
    // A value that passes native validation and resolves to an `example.com`
    // hostname — the fake server's "not allowed" branch.
    input.set_value("https://example.com/product");

    assert!(
        dispatch_submit(&form),
        "the handler prevents the native submission itself when it is reached"
    );
    flush_one_turn().await;

    // Pending: the button is disabled yet focusable — `aria-disabled`, no
    // `disabled` attribute (useFocusableWhenDisabled.ts:38-47).
    let button = form_submit_button(&container);
    assert_eq!(
        button.get_attribute("aria-disabled").as_deref(),
        Some("true"),
        "the submit button is disabled while the fake server is pending"
    );
    assert!(
        button.get_attribute("disabled").is_none(),
        "focusableWhenDisabled keeps the native disabled attribute off"
    );

    // The fake server's 1s response (test-sized delay).
    flush_after_ms(30).await;

    // The port rebuilds the Form subtree when the errors record changes (its
    // static-prop law), so the control is a FRESH node — re-queried here, and
    // checked to be the live one rather than the detached predecessor.
    let input = form_control(&container, "https://example.com");
    assert!(
        input.is_connected(),
        "the re-queried control is the live node in the document"
    );
    let error = form_error_slot(&container);
    assert!(
        !error.has_attribute("hidden"),
        "the field's error slot is visible after the server error landed; html was: {}",
        container.inner_html()
    );
    assert_eq!(
        error.text_content().as_deref(),
        Some("The example domain is not allowed"),
        "the fake server's message renders on the matching field"
    );
    assert_eq!(
        input.value(),
        "https://example.com/product",
        "the uncontrolled control still shows what the user typed (the demo re-seeds \
         defaultValue from the submitted value across the rebuild)"
    );
    assert_ne!(
        form_submit_button(&container)
            .get_attribute("aria-disabled")
            .as_deref(),
        Some("true"),
        "the button leaves its pending state once the response lands"
    );
}

/// The form-action demo (`form-action/tailwind/index.tsx`): the server action's
/// error keyed by `Field.Root name="username"` renders on the field. The demo's
/// seeded value is `admin` (`:28`), which is upstream's deterministic
/// reserved-name branch (`:56-58`) — the 50% "unavailable" branch is left alone.
#[wasm_bindgen_test]
async fn form_action_demo_surfaces_the_reserved_name_error() {
    let container = form_container("test-mount-root-form-action");

    use crate::pages::form_page::FormActionDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FormActionDemo delay_ms=10 /> },
    ));
    flush_one_turn().await;

    let form = form_in(&container);
    let input = form_control(&container, "e.g. alice132");
    assert_eq!(
        input.value(),
        "admin",
        "the demo's control is seeded from its defaultValue"
    );
    assert_eq!(
        input.get_attribute("autocomplete").as_deref(),
        Some("username"),
        "the control's autoComplete rides the elementProps rest"
    );

    assert!(dispatch_submit(&form));
    flush_after_ms(30).await;

    let error = form_error_slot(&container);
    assert!(
        !error.has_attribute("hidden"),
        "the server action's error surfaced on the field; html was: {}",
        container.inner_html()
    );
    assert_eq!(
        error.text_content().as_deref(),
        Some("'admin' is reserved for system use"),
        "the action's message for the username field renders verbatim"
    );
    // The username survives the rebuild (the demo re-seeds defaultValue from the
    // value the fake action received).
    assert_eq!(form_control(&container, "e.g. alice132").value(), "admin");
}

/// The zod demo (`zod/tailwind/index.tsx`): the values record `onFormSubmit`
/// receives is validated against the schema's two rules and the flattened field
/// errors map back to each `Field.Error` by `Field.Root name` — both branches of
/// the schema, then the successful parse that clears every error (`:63`).
#[wasm_bindgen_test]
async fn form_zod_demo_maps_the_schema_errors_to_each_field() {
    let container = form_container("test-mount-root-form-zod");

    use crate::pages::form_page::FormZodDemo;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(
        { container.clone() },
        || view! { <FormZodDemo /> },
    ));
    flush_one_turn().await;

    let form = form_in(&container);
    // Nothing gates this demo natively (neither control is required), so the
    // schema's verdict is the only thing that can populate the errors.
    assert!(
        dispatch_submit(&form),
        "onFormSubmit's presence makes the Form prevent the native default itself"
    );
    flush_one_turn().await;

    // Every assertion below re-queries: the demo rebuilds its Form subtree when
    // the errors record changes (this module's header, adaptation 1), so the
    // nodes from before the submit are detached.
    let slots = els(container.as_ref(), ".text-red-700");
    assert_eq!(slots.len(), 2, "one error slot per field");
    assert!(
        !slots[0].has_attribute("hidden") && !slots[1].has_attribute("hidden"),
        "both fields report their schema error; html was: {}",
        container.inner_html()
    );
    assert_eq!(
        slots[0].text_content().as_deref(),
        Some("Name is required"),
        "`z.string().min(1, 'Name is required')` on the name field"
    );
    assert_eq!(
        slots[1].text_content().as_deref(),
        Some("Age must be a positive number"),
        "`z.coerce.number('').positive('Age must be a positive number')` on the age field"
    );

    // The successful parse: both values valid, so `errors` resets to `{}` and
    // every field error clears.
    let name = form_control(&container, "Enter name");
    let age = form_control(&container, "Enter age");
    // Type them like a user does — set the value AND dispatch the `input` event the
    // port's write path listens for (`type_into`'s contract, the OTP suite's helper).
    // A bare `set_value` leaves the field's own change path unfired, which is not
    // what a visitor can do.
    type_into(&name, "Ada");
    type_into(&age, "30");
    assert!(dispatch_submit(&form));
    flush_one_turn().await;

    let slots = els(container.as_ref(), ".text-red-700");
    assert!(
        slots[0].has_attribute("hidden") && slots[1].has_attribute("hidden"),
        "a successful parse clears every field error; html was: {}",
        container.inner_html()
    );
    // The submitted values survive the rebuild (the demo re-seeds each
    // defaultValue from what it received).
    assert_eq!(form_control(&container, "Enter name").value(), "Ada");
    assert_eq!(form_control(&container, "Enter age").value(), "30");
}

/// The mirrored page: `page.mdx`'s headings in document order, the hero demo
/// before the first heading, the three `## Examples` subsections with their
/// snippets, and the API reference's echoed `TypesForm` content.
#[wasm_bindgen_test]
fn form_page_component_renders_the_full_page_structure() {
    let container = form_container("test-mount-root-form-page");

    use crate::pages::form_page::FormPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to({ container.clone() }, || view! { <FormPage /> }));

    let html = container.inner_html();
    assert!(
        html.contains("<h1>Form</h1>"),
        "the h1 did not render; html was: {html}"
    );
    assert!(
        html.contains("A native form element with consolidated error handling."),
        "the subtitle did not render"
    );
    // The prose headings, verbatim. U+00A0 is spelled as the `&nbsp;` entity here because
    // `inner_html()` SERIALIZES it that way (the live DOM's `textContent` — what the
    // differential's snapshot reads — carries the character itself; asserted below).
    for heading in [
        "Anatomy",
        "Examples",
        "Submit with a Server&nbsp;Function",
        "Submit form values as a JavaScript&nbsp;object",
        "Using with Zod",
        "API reference",
        "Form",
        "Canonical Types",
    ] {
        assert!(
            html.contains(&format!(">{heading}<")),
            "heading '{heading}' missing; html was: {html}"
        );
    }
    // And the character itself, on the same headings the upstream differential compares
    // (`textContent`, not the serialization) — this module's header, fact 1.
    let h3_text: Vec<String> = els(container.as_ref(), "h3")
        .iter()
        .map(|node| node.text_content().unwrap_or_default())
        .collect();
    for heading in [
        "Submit with a Server\u{a0}Function",
        "Submit form values as a JavaScript\u{a0}object",
    ] {
        assert!(
            h3_text.iter().any(|text| text == heading),
            "the rendered h3 textContent '{heading}' (U+00A0 included) is missing; h3 texts were: {h3_text:?}"
        );
    }
    // The generated type sections: upstream's `AdditionalTypeHeading` markup — the name
    // immediately followed by the `AdditionalTypeBackLink` label, so the heading's
    // `textContent` reads `Form.PropsHide` (this module's header, fact 2).
    for name in [
        "Form.Props",
        "Form.State",
        "Form.Actions",
        "Form.SubmitEventDetails",
        "Form.SubmitEventReason",
        "Form.ValidationMode",
        "Form.Values",
    ] {
        assert!(
            html.contains(&format!(">{name}<a")),
            "type heading '{name}' is not followed by upstream's back-link; html was: {html}"
        );
    }
    assert!(
        html.contains("class=\"AdditionalTypeBackLink\">Hide</a>"),
        "upstream's back-link label did not render; html was: {html}"
    );
    assert!(
        html.contains("id=\"form.submiteventdetails\""),
        "the type sections carry upstream's slug wrapper id; html was: {html}"
    );

    // The three demos are mounted (page.mdx's hero + the two Examples demos).
    for demo in ["hero", "form-action", "zod"] {
        assert!(
            container
                .query_selector(&format!("[data-demo='{demo}']"))
                .expect("query demo slot")
                .is_some(),
            "the '{demo}' demo slot did not render"
        );
    }
    // The hero demo precedes the first heading (page.mdx document order).
    let demo_at = html.find("data-demo").expect("demo slot in html");
    let anatomy_at = html
        .find("<h2>Anatomy</h2>")
        .expect("Anatomy heading in html");
    assert!(
        demo_at < anatomy_at,
        "the hero demo must render before the first heading (page.mdx order)"
    );

    // The page's embedded snippets and prose, verbatim from page.mdx.
    assert!(
        html.contains("import { Form } from '@base-ui/react/form';"),
        "the Anatomy snippet did not render"
    );
    assert!(
        html.contains("onFormSubmit={async (formValues: { id: string; quantity: number })"),
        "the onFormSubmit snippet did not render"
    );
    assert!(
        html.contains("`preventDefault` is called on the native submit event."),
        "the preventDefault claim did not render"
    );
    assert!(
        html.contains("z.flattenError(result.error).fieldErrors"),
        "the Zod prose did not render"
    );
    // The API reference echoes the generated table's content as static prose.
    let props = els(container.as_ref(), ".api-props");
    assert!(
        props[0]
            .text_content()
            .unwrap_or_default()
            .contains("validationMode"),
        "the Form props prose did not render"
    );
    assert!(
        html.contains("actionsRef.current?.validate('email')"),
        "the actionsRef example did not render"
    );
}

// ═══ The docs chrome: layout shell, header, side navigation ═══
//
// The `docs-chrome: layout shell` item's done-when has two halves. The DOM half is here:
// the shell's layers, the nav tree's content/grouping, the current-route marking, and — the
// drift guard — that every href the nav renders is a route this crate actually serves. The
// visual half (typography scale, sidebar/header geometry, pixel proximity against the React
// site) is measured by `ralph/scripts/check-visual-budget.mjs` on a real page load, which is
// the only place a stylesheet can be observed.

/// A fresh appended container — the crate's test convention (`app_mounts_and_renders_the_shell`).
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

/// The `(text, href)` pairs of every `a.SideNavLink` under `root`, in document order.
fn nav_link_pairs(root: &web_sys::Element) -> Vec<(String, Option<String>)> {
    els(root, "a.SideNavLink")
        .iter()
        .map(|link| {
            (
                link.text_content().unwrap_or_default().trim().to_string(),
                link.get_attribute("href"),
            )
        })
        .collect()
}

/// Pins the test page's URL so the next `Router` mount resolves to `path`.
///
/// This uses `replaceState`, NOT `pushState` + `popstate`. Both work for a router that mounts
/// afterwards (the router reads the URL at mount), but dispatching `popstate` in this shared test
/// document also notifies the listeners ROUTERS FROM EARLIER TESTS left behind, and a listener
/// whose reactive owner has been dropped panics inside `RwSignal<Option<String>>::get` — measured:
/// the drift guard below killed the whole wasm runner that way, and it was the only test in the
/// suite that navigated a mounted app. `replaceState` fires no event, so the leak cannot bite.
fn pin_url(path: &str) {
    web_sys::window()
        .expect("window")
        .history()
        .expect("history")
        .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path))
        .expect("replace_state_with_url");
}

#[wasm_bindgen_test]
fn side_nav_lists_every_ported_route_grouped_like_upstream() {
    use crate::chrome::{NAV_EXTERNAL, NAV_SECTIONS, SideNav};
    use leptos::prelude::*;

    let container = fresh_container("test-side-nav-structure");
    let _guard = leptos::mount::mount_to({ container.clone() }, || {
        view! { <SideNav current_path="/react/components/checkbox".to_string() /> }
    });

    // Upstream's root: `nav.SideNavRoot[aria-label="Main navigation"]`, with the
    // `data-side-nav-viewport` marker its own active-item scroll-into-view keys on
    // (`SideNav.tsx:10-21,93`).
    let nav = els(container.as_ref(), "nav.SideNavRoot");
    assert_eq!(nav.len(), 1, "the side nav's root element did not render");
    assert_eq!(
        nav[0].get_attribute("aria-label").as_deref(),
        Some("Main navigation"),
        "the nav's accessible name is not upstream's"
    );
    assert_eq!(
        els(container.as_ref(), "[data-side-nav-viewport]").len(),
        1,
        "the viewport marker upstream's active-item scroll logic reads is missing"
    );

    // Section headings, in upstream's sitemap order (Components, then Utils).
    let headings: Vec<String> = els(container.as_ref(), ".SideNavHeading")
        .iter()
        .map(|heading| heading.text_content().unwrap_or_default())
        .collect();
    let expected_headings: Vec<String> = NAV_SECTIONS
        .iter()
        .map(|section| section.heading.to_string())
        .collect();
    assert_eq!(
        headings, expected_headings,
        "the nav's section headings do not match the sitemap sections this crate serves"
    );

    // Every ported route, in order, with upstream's label and href — then upstream's two
    // external links after the separator (`layout.tsx:76-96`).
    let expected: Vec<(String, Option<String>)> = NAV_SECTIONS
        .iter()
        .flat_map(|section| section.items.iter())
        .chain(NAV_EXTERNAL.iter())
        .map(|item| (item.title.to_string(), Some(item.href.to_string())))
        .collect();
    assert_eq!(
        nav_link_pairs(container.as_ref()),
        expected,
        "the nav tree must list every ported route exactly once"
    );

    assert_eq!(
        els(container.as_ref(), "hr.SideNavSeparator").len(),
        1,
        "the separator upstream puts before the external links is missing"
    );
    // The external links are the only ones that open in a new tab.
    let external = els(container.as_ref(), "a.SideNavLink[target=_blank]");
    assert_eq!(
        external.len(),
        NAV_EXTERNAL.len(),
        "the external links must be the only ones marked `target=_blank`"
    );
}

#[wasm_bindgen_test]
fn side_nav_marks_only_the_current_route_active() {
    use crate::chrome::SideNav;
    use leptos::prelude::*;

    // Exact matching matters: "Checkbox" must NOT light up while "Checkbox Group" is open
    // (upstream compares `usePathname() === href`, `SideNav.tsx:88-89`), and an unported
    // route marks nothing (the nav lists only the routes this crate serves).
    for (case, path, expected) in [
        (
            "checkbox",
            "/react/components/checkbox",
            Some("/react/components/checkbox"),
        ),
        (
            "checkbox-group",
            "/react/components/checkbox-group",
            Some("/react/components/checkbox-group"),
        ),
        ("unported", "/react/components/drawer", None),
    ] {
        let container = fresh_container(&format!("test-side-nav-active-{case}"));
        let _guard = leptos::mount::mount_to({ container.clone() }, || {
            view! { <SideNav current_path=path.to_string() /> }
        });

        let active = els(container.as_ref(), "a.SideNavLink[data-active]");
        match expected {
            Some(href) => {
                assert_eq!(
                    active.len(),
                    1,
                    "{path} must mark exactly one item active (case {case})"
                );
                assert_eq!(
                    active[0].get_attribute("href").as_deref(),
                    Some(href),
                    "the wrong item is active for {path}"
                );
                assert_eq!(
                    active[0].get_attribute("aria-current").as_deref(),
                    Some("true"),
                    "upstream's Item sets BOTH `data-active` and `aria-current` (SideNav.tsx:105-115)"
                );
            }
            None => assert!(
                active.is_empty(),
                "{path} is not a route this crate serves, so nothing may be marked active"
            ),
        }

        // No other link carries the aria state either.
        assert_eq!(
            els(container.as_ref(), "a.SideNavLink[aria-current]").len(),
            active.len(),
            "an inactive item carries `aria-current`"
        );
    }
}

// The nav → route drift guard moved to its own test page: `tests/nav_routes.rs`.
//
// It mounts the real `App` once per side-nav href (18 mounts, the avatar page among them), and
// `wasm-bindgen-test` runs every test of this crate in ONE shared page — so while the guard lived
// here, the leak it exposes (the avatar image probe's late callback reading a status mirror the
// unmount already disposed, `crates/leptos-ui/src/avatar/image.rs:303/747`) landed on whichever
// test was running when the callback fired, turning four tests it never asserts on red
// (`checkbox_hero_demo_toggles_through_the_real_port`, the two checkbox-group interaction tests,
// `avatar_hero_demo_renders_the_real_root_composition`) while every guard assertion passed. A
// separate target is a separate wasm binary, hence a separate page: the guard keeps its full
// strength there, and this page keeps its clean run. The defect itself is a ledger item of its own
// (TODO.md, "library: avatar — the image probe writes to a disposed status mirror").

/// The ported code-block chrome (`crate::code_block`), measured on the route that carries the most
/// snippets. `visual-gap-report.mjs` raised this item as the first P0 on every recorded route —
/// "upstream's code carries 41 coloured tokens; this page has 0 — the code is unstyled monochrome"
/// — so the assertion is on the DOM the real `CheckboxPage` mount produces, not on the constants:
/// each of the page's five embedded snippets renders inside upstream's `.CodeBlockRoot` (panel with
/// the fence's own `.mdx` title, a copy control), its code text is preserved character for
/// character (every fidelity probe and the pages' own snippet guards read `pre.textContent`), and
/// its tokens carry the prettylights classes the ported stylesheet colours.
#[wasm_bindgen_test]
fn checkbox_page_code_blocks_render_through_the_ported_chrome() {
    use crate::pages::checkbox_page::CheckboxPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-code-blocks");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(container.clone(), || view! { <CheckboxPage /> }));

    let roots = els(&container, ".CodeBlockRoot");
    assert_eq!(
        roots.len(),
        5,
        "the page's five embedded snippets render as code blocks"
    );

    // The titles are the page's own `.mdx` fence titles (`page.mdx:21-88`), in page order.
    let expected_titles = [
        "Anatomy",
        "Wrapping a label around a checkbox",
        "Sibling label pattern with a native button",
        "Render callback",
        "Using Checkbox in a form",
    ];

    let mut classified_total = 0usize;
    for (index, title) in expected_titles.iter().enumerate() {
        let root = &roots[index];
        let title_el = els(root, ".CodeBlockPanelTitle");
        assert_eq!(
            title_el.len(),
            1,
            "block {index} renders exactly one panel title"
        );
        assert_eq!(
            title_el[0].text_content().unwrap_or_default(),
            *title,
            "block {index}'s panel title must be its fence's title"
        );

        let copy = els(root, "button[aria-label='Copy code']");
        assert_eq!(
            copy.len(),
            1,
            "block {index} renders exactly one copy control"
        );
        assert_eq!(
            els(&copy[0], "svg").len(),
            1,
            "block {index}'s copy control carries the icon"
        );

        let code = els(root, "pre code");
        assert_eq!(code.len(), 1, "block {index} renders one `pre > code`");
        let code = &code[0];
        let total: usize = code
            .get_attribute("data-total-lines")
            .expect("data-total-lines")
            .parse()
            .expect("a line count");
        assert_eq!(
            els(code, ".line").len(),
            total,
            "block {index} declares {total} lines and renders a different number of line spans"
        );
        assert!(
            code.class_list().contains("language-rust"),
            "the port's own snippet is fenced as rust, not as upstream's jsx"
        );
        let text = code.text_content().unwrap_or_default();
        assert!(
            text.starts_with("use leptos::prelude::*;"),
            "block {index}'s text is the port's snippet; it read: {text:?}"
        );

        classified_total += els(
            code,
            ".pl-k, .pl-c1, .pl-s, .pl-en, .pl-ent, .pl-smi, .pl-c, .di-bool, .di-n",
        )
        .len();
    }
    assert!(
        classified_total >= 40,
        "the five blocks carry the prettylights token hooks the stylesheet colours; found {classified_total}"
    );
}
