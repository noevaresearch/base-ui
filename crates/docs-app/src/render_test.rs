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
        let event =
            web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
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
        label.text_content().unwrap_or_default().contains("(locked)"),
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

    let _guard = mount_to(
        { container.clone() },
        move || {
            view! {
                <DirectionProviderRtlDemo />
                <DirectionProbe />
            }
        },
    );

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
