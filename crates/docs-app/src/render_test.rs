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
    std::mem::forget(mount_to({ container.clone() }, || view! { <AccordionHeroDemo /> }));

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
    std::mem::forget(mount_to({ container.clone() }, || view! { <MultipleDemo /> }));

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
    std::mem::forget(mount_to({ container.clone() }, || view! { <HiddenUntilFoundDemo /> }));

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
    std::mem::forget(mount_to({ container.clone() }, || view! { <AccordionPage /> }));

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
        Some("flex h-8 items-center justify-center gap-2 rounded-none border border-neutral-950 bg-white px-3 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none hover:not-data-disabled:bg-neutral-100 active:not-data-disabled:bg-neutral-200 focus-visible:outline-2 focus-visible:-outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white data-disabled:border-neutral-500 data-disabled:text-neutral-500 disabled:border-neutral-500 disabled:text-neutral-500 dark:border-white dark:bg-neutral-950 dark:text-white dark:hover:not-data-disabled:bg-neutral-800 dark:active:not-data-disabled:bg-neutral-700 dark:data-disabled:border-neutral-400 dark:data-disabled:text-neutral-400"),
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
    // The component form the page mounts (the orphan refactor's target): the
    // loading demo lives inside a real mount so its dynamic-child rebuild
    // has a reactive scope to subscribe under.
    let _guard = mount_to({ container.clone() }, || view! {
        <ButtonLoadingDemo reset_ms=250 />
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
        html.contains("A button component that can be rendered as another tag or focusable when disabled."),
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
        html.contains("@base-ui/react/button"),
        "the Anatomy import snippet did not render"
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
