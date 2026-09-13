//! Tests for the Avatar port — mirrors of the three upstream suites behavior.md
//! mines (`AvatarRoot/AvatarImage/AvatarFallback.test.tsx`), over the pure
//! builders in a host suite and the real mounted tree in a wasm-browser suite
//! (the meter/button dual-target convention).
//!
//! Host suite: the state machines without a DOM — the Tier-1 status machine's
//! short-circuits (via the serialized source-config key, the dep-array
//! vocabulary), the delay-latch machine, the enabled gate, the suppression
//! mapping, the presence gate.
//! Wasm suite: the full mounted tree — the probe cycle with a deterministic
//! fake (the upstream `window.Image` stub), the keepMounted attribute matrix,
//! the fallback gating, the aria-hidden contract, and the exact
//! image-or-fallback exclusivity (behavior.md *Edge cases* regression).
//! The missing-Root panic contract is pinned host-side (a wasm panic is an
//! uncatchable trap — the meter b9b107c12 precedent).

// The wasm-only harness items are dead code on the host target — the dual-target
// test-module convention (the meter_tests.rs precedent).
#![allow(unused_imports, dead_code)]

use super::*;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use crate::avatar::source_config_key_for_tests;

    fn in_owner() -> reactive_graph::owner::Owner {
        let _ = any_spawner::Executor::init_futures_executor();
        let owner = reactive_graph::owner::Owner::new();
        owner.set();
        owner
    }

    // The Tier-1 dep-array vocabulary (`useImageLoadingStatus.ts:68`): the
    // serialized source-config key is lossless and stable — the keepMounted
    // arm is a dedicated sentinel, an empty config is distinct from it, and
    // each field's presence/absence survives the round trip. The key is the
    // reactive source the scheduling effect tracks, so "any field changes →
    // the key changes → the effect re-runs" is the upstream dep-array
    // restart (the reset-on-source-change transition, behavior.md
    // *Transitions*; implementation.md untested item 4: config-prop-only
    // resets too).
    #[test]
    fn the_source_config_key_is_lossless_and_stable() {
        // keepMounted: the sentinel regardless of config.
        assert_eq!(
            source_config_key_for_tests(None, None, None, None, None, false),
            "\u{1}\u{1}keepMounted"
        );
        assert_eq!(
            source_config_key_for_tests(Some("/a.png"), None, None, None, None, false),
            "\u{1}\u{1}keepMounted"
        );
        // Empty config vs sentinel: distinct.
        let empty = source_config_key_for_tests(None, None, None, None, None, true);
        assert_ne!(empty, "\u{1}\u{1}keepMounted");
        // Round-trip: five fields.
        let full = source_config_key_for_tests(
            Some("/a.png"),
            Some("a.png 2x"),
            Some("50vw"),
            Some("origin"),
            Some("use-credentials"),
            true,
        );
        let parts: Vec<&str> = full.split('\u{1}').collect();
        assert_eq!(parts[0], "/a.png");
        assert_eq!(parts[1], "a.png 2x");
        assert_eq!(parts[2], "50vw");
        assert_eq!(parts[3], "origin");
        assert_eq!(parts[4], "use-credentials");
        // Changing any one field changes the key (the dep-array restart).
        for changed in [
            source_config_key_for_tests(
                Some("/b.png"),
                Some("a.png 2x"),
                Some("50vw"),
                Some("origin"),
                Some("use-credentials"),
                true,
            ),
            source_config_key_for_tests(
                Some("/a.png"),
                Some("b.png 2x"),
                Some("50vw"),
                Some("origin"),
                Some("use-credentials"),
                true,
            ),
            source_config_key_for_tests(
                Some("/a.png"),
                Some("a.png 2x"),
                Some("100vw"),
                Some("origin"),
                Some("use-credentials"),
                true,
            ),
            source_config_key_for_tests(
                Some("/a.png"),
                Some("a.png 2x"),
                Some("50vw"),
                Some("no-referrer"),
                Some("use-credentials"),
                true,
            ),
            source_config_key_for_tests(
                Some("/a.png"),
                Some("a.png 2x"),
                Some("50vw"),
                Some("origin"),
                Some("anonymous"),
                true,
            ),
        ] {
            assert_ne!(full, changed, "a changed field changes the key");
        }
    }

    // The presence gate (`AvatarImage.tsx:153`): keepMounted || mounted.
    #[test]
    fn should_render_is_keep_mounted_or_mounted() {
        let base = |keep: bool, mounted: bool| AvatarImageStateSnapshot {
            image_loading_status: ImageLoadingStatus::Loading,
            transition_status: None,
            keep_mounted: keep,
            mounted,
        };
        assert!(should_render(&base(true, false)));
        assert!(should_render(&base(false, true)));
        assert!(!should_render(&base(false, false)));
    }

    // The fallback enabled gate (`AvatarFallback.tsx:46`): status !== 'loaded'
    // && (delay === 0 || delayPassed). The core contract
    // (`AvatarFallback.test.tsx:38-51`: loaded → never shown), the pending-
    // delay arms, and the `delay === 0` render-cycle OR (`:47-49`).
    #[test]
    fn the_fallback_gate_is_status_and_delay() {
        // Not loaded, no delay → shown (all three non-loaded statuses).
        assert!(avatar_fallback_state(ImageLoadingStatus::Idle, 0.0, false));
        assert!(avatar_fallback_state(ImageLoadingStatus::Loading, 0.0, false));
        assert!(avatar_fallback_state(ImageLoadingStatus::Error, 0.0, false));
        // Loaded → never shown, whatever the latch says.
        assert!(!avatar_fallback_state(ImageLoadingStatus::Loaded, 0.0, false));
        assert!(!avatar_fallback_state(ImageLoadingStatus::Loaded, 0.0, true));
        // Pending delay, latch not fired → hidden.
        assert!(!avatar_fallback_state(ImageLoadingStatus::Loading, 500.0, false));
        // Pending delay, latch fired → shown.
        assert!(avatar_fallback_state(ImageLoadingStatus::Loading, 500.0, true));
    }

    // The suppression mapping (`stateAttributesMapping.ts:1-3`): the status
    // member maps to null on every part; a transition key falls through to
    // the image mapping's second member; a foreign key gets no mapping at
    // all (the default walk's `data-<key>` output is the engine's tested
    // behavior, the progress host suite's mapping-decline twin).
    #[test]
    fn the_status_member_is_suppressed_from_the_dom() {
        let mapping = avatar_state_attributes_mapping();
        let loaded = serde_json::Value::String("loaded".to_string());
        assert_eq!(mapping("imageLoadingStatus", &loaded), Some(None));
        assert_eq!(mapping("transitionStatus", &loaded), None);
        assert_eq!(mapping("unrelated", &loaded), None);
    }

    // The image's combined mapping (`AvatarImage.tsx:16-19`): the root's
    // suppression for the status member PLUS the transition mapping's
    // style-hook emission (`starting`/`ending` → data-starting-style /
    // data-ending-style; anything else on a transition key → nothing).
    #[test]
    fn the_image_mapping_layers_the_transition_hooks() {
        let mapping = avatar_image_state_attributes_mapping();
        let starting = serde_json::Value::String("starting".to_string());
        let ending = serde_json::Value::String("ending".to_string());
        let idle = serde_json::Value::String("idle".to_string());
        let loaded = serde_json::Value::String("loaded".to_string());

        // The suppression still applies to the status member.
        assert_eq!(mapping("imageLoadingStatus", &loaded), Some(None));
        // Transition hooks emit exactly their own attribute.
        match mapping("transitionStatus", &starting) {
            Some(Some(props)) => {
                assert_eq!(props.len(), 1, "one hook: {props:?}");
                assert!(props.contains_key("data-starting-style"));
            }
            other => panic!("the starting hook maps to a bag: {other:?}"),
        }
        match mapping("transitionStatus", &ending) {
            Some(Some(props)) => {
                assert_eq!(props.len(), 1, "one hook: {props:?}");
                assert!(props.contains_key("data-ending-style"));
            }
            other => panic!("the ending hook maps to a bag: {other:?}"),
        }
        // `idle`/null on a transition key emits nothing.
        assert_eq!(mapping("transitionStatus", &idle), Some(None));
    }

    // The context shell: the missing-root accessor panics the upstream
    // message (AvatarRootContext.ts:14-18; implementation.md untested item 1
    // pins the port's decision to reproduce the throw — the meter
    // use_meter_root_context precedent).
    #[test]
    fn the_missing_root_context_is_the_upstream_error() {
        let owner = in_owner();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::avatar::use_avatar_root_context();
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
            message.starts_with("Base UI: AvatarRootContext is missing."),
            "the missing-provider error reads {message:?}"
        );
    }

    // The status vocabulary (`AvatarRoot.tsx:44`) — the strings the fan-out
    // reports (behavior.md *Events*: "a single string status argument").
    #[test]
    fn the_status_strings_match_upstream() {
        assert_eq!(ImageLoadingStatus::Idle.as_str(), "idle");
        assert_eq!(ImageLoadingStatus::Loading.as_str(), "loading");
        assert_eq!(ImageLoadingStatus::Loaded.as_str(), "loaded");
        assert_eq!(ImageLoadingStatus::Error.as_str(), "error");
    }
}

// ---------------------------------------------------------------------------
// Wasm suite
// ---------------------------------------------------------------------------
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use leptos::prelude::view;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::Event;

    use super::*;
    use crate::avatar::{
        Probe, ProbeFactory, avatar_fallback_view, avatar_image_view, with_probe_factory,
    };

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// The deterministic fake probe — the upstream `window.Image` stub
    /// (`AvatarImage.test.tsx:27-73`): `complete_on_set` + `natural_width`
    /// decide the cached fast path; the exposed element lets tests fire
    /// `load`/`error` and assert the request-config assignments.
    struct FakeProbe {
        image: web_sys::HtmlImageElement,
        complete_on_set: bool,
        natural_width: u32,
    }

    impl Probe for FakeProbe {
        fn complete(&self) -> bool {
            self.complete_on_set
        }

        fn natural_width(&self) -> u32 {
            self.natural_width
        }

        fn set_on_load(&mut self, callback: Rc<dyn Fn()>) {
            let closure: wasm_bindgen::closure::Closure<dyn FnMut()> =
                wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()));
            self.image
                .set_onload(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        fn set_on_error(&mut self, callback: Rc<dyn Fn()>) {
            let closure: wasm_bindgen::closure::Closure<dyn FnMut()> =
                wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()));
            self.image
                .set_onerror(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        // Deliberately NO image surface (`None`): `configure_probe` skips its
        // assignments, so no real `src` is ever set and no real network fetch
        // races the tests — the load/error events are purely test-driven, the
        // exact determinism contract of the upstream `window.Image` stub
        // (`AvatarImage.test.tsx:27-73`, which replaces the constructor
        // wholesale). The config-assignment contract has its own test below
        // with a surfacing probe.
        fn as_image(&self) -> Option<web_sys::HtmlImageElement> {
            None
        }
    }

    /// A factory whose probes register themselves in `sink` so the test can
    /// fire events on the live probe (the factory is called from the rg
    /// effect run inside the machinery owner, so the sink crosses as an
    /// `Rc<RefCell>` — the machinery is single-threaded wasm).
    fn recording_factory(
        sink: Rc<RefCell<Vec<web_sys::HtmlImageElement>>>,
    ) -> ProbeFactory {
        Rc::new(move || -> Box<dyn Probe> {
            let image: web_sys::HtmlImageElement = document()
                .create_element("img")
                .unwrap()
                .dyn_into()
                .unwrap();
            sink.borrow_mut().push(image.clone());
            Box::new(FakeProbe {
                image,
                complete_on_set: false,
                natural_width: 0,
            })
        })
    }

    /// The factory for the request-config-assignment contract (upstream
    /// `AvatarImage.test.tsx:135-147`: the probe receives `sizes`, `srcset`,
    /// `src`): the probe EXPOSES its element so `configure_probe`'s
    /// assignments are observable. `complete_on_set: false` keeps the
    /// machine in `'loading'`; the one fetch this test provable starts is
    /// harmless — the assertions read attributes set synchronously.
    fn surfacing_factory(sink: Rc<RefCell<Vec<web_sys::HtmlImageElement>>>) -> ProbeFactory {
        Rc::new(move || -> Box<dyn Probe> {
            let image: web_sys::HtmlImageElement = document()
                .create_element("img")
                .unwrap()
                .dyn_into()
                .unwrap();
            sink.borrow_mut().push(image.clone());
            Box::new(SurfacingProbe { image })
        })
    }

    /// The surfacing variant — `as_image` returns the element (the default
    /// `RealProbe` shape), so the config assignments land on it.
    struct SurfacingProbe {
        image: web_sys::HtmlImageElement,
    }

    impl Probe for SurfacingProbe {
        fn complete(&self) -> bool {
            false
        }

        fn natural_width(&self) -> u32 {
            0
        }

        fn set_on_load(&mut self, callback: Rc<dyn Fn()>) {
            let closure: wasm_bindgen::closure::Closure<dyn FnMut()> =
                wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()));
            self.image
                .set_onload(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        fn set_on_error(&mut self, callback: Rc<dyn Fn()>) {
            let closure: wasm_bindgen::closure::Closure<dyn FnMut()> =
                wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()));
            self.image
                .set_onerror(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        fn as_image(&self) -> Option<web_sys::HtmlImageElement> {
            Some(self.image.clone())
        }
    }

    fn cached_factory() -> ProbeFactory {
        Rc::new(move || -> Box<dyn Probe> {
            let image: web_sys::HtmlImageElement = document()
                .create_element("img")
                .unwrap()
                .dyn_into()
                .unwrap();
            image.set_src("/cached.png");
            Box::new(FakeProbe {
                image,
                complete_on_set: true,
                natural_width: 24,
            })
        })
    }

    fn container() -> web_sys::Element {
        let container = document().create_element("div").unwrap();
        container.set_attribute("data-avatar-test", "1").unwrap();
        document().body().unwrap().append_child(&container).unwrap();
        container
    }

    /// Every `<span>` under the host in document order: [0] is always the
    /// ROOT span (mounted first), [1] the fallback span when it is mounted.
    /// A bare `query_selector("span")` matches the root — never the fallback.
    fn spans(host: &web_sys::Element) -> Vec<web_sys::Element> {
        let list = host.query_selector_all("span").unwrap();
        let mut out: Vec<web_sys::Element> = Vec::new();
        for index in 0..list.length() {
            let node = list.get(index).unwrap();
            out.push(node.dyn_into::<web_sys::Element>().unwrap());
        }
        out
    }

    /// Mounts `build`'s view into a fresh container and leaks the handle —
    /// the `std::mem::forget(mount_to(...))` convention (dropping the
    /// UnmountHandle would unmount the tree).
    fn mount_view<V: leptos::prelude::IntoView + 'static>(
        build: impl FnOnce() -> V + 'static,
    ) -> web_sys::Element {
        let _ = any_spawner::Executor::init_futures_executor();
        let host = container();
        std::mem::forget(leptos::mount::mount_to(host.clone().unchecked_into(), build));
        host
    }

    /// The full three-part mount (`AvatarRoot.test.tsx`'s topology: Image +
    /// Fallback under Root). The subtree is CONSTRUCTED inside Root's view —
    /// a body runs when its view is built, so the context is provided before
    /// the parts read it (the meter_tests harness trap). The root element
    /// bridges into the view through `AvatarDocView` (the raw-element view);
    /// the parts ride their dynamic views.
    fn mount_avatar(
        image_props: crate::avatar::AvatarImageProps,
        fallback_props: crate::avatar::AvatarFallbackProps,
        factory: Option<crate::avatar::ProbeFactory>,
    ) -> web_sys::Element {
        let run = move || {
            let build = move || {
                // Root first (provides the context, returns the span), then
                // the parts (read it). Same scope → same owner.
                let root_element = use_avatar_root(crate::avatar::AvatarRootProps::default());
                let image_handle = use_avatar_image(&image_props);
                let fallback_handle = use_avatar_fallback(&fallback_props);
                view! {
                    {crate::avatar::AvatarDocView { element: root_element }}
                    // The parts ride REAL dynamic-view children: the closure
                    // INVOKED per reactive run (the progress-hero shape). A
                    // bare `avatar_image_view(...)` here binds the never-
                    // invoked closure itself — the type checks, the tree
                    // never exists.
                    {dynamic(avatar_image_view(image_handle, image_props))}
                    {dynamic(avatar_fallback_view(fallback_handle, fallback_props))}
                }
            };
            match factory {
                Some(factory) => with_probe_factory(factory, || mount_view(build)),
                None => mount_view(build),
            }
        };
        run()
    }

    /// Binds a dynamic-part closure as a leptos dynamic-view child — the
    /// actual `{move || …}` invocation. A closure handed to `view!` bare is a
    /// never-invoked value (the type checks, the tree never exists — this
    /// harness's first wasm run's exact failure). The handle and props ride
    /// each invocation through the SendWrapper-capturing bodies in views.rs.
    fn dynamic<V: leptos::prelude::IntoView + 'static>(
        body: impl Fn() -> V + Send + 'static,
    ) -> impl leptos::prelude::IntoView + 'static {
        move || body()
    }

    fn flush() {
        for _ in 0..32 {
            any_spawner::Executor::poll_local();
        }
    }

    // The keepMounted status-attribute bag (`renderedStatusProps`,
    // `AvatarImage.tsx:101-118`) through the pure builder: while loading the
    // element carries data-loading (bare) + aria-hidden=true and neither
    // data-error nor a generic status attribute; while errored, data-error +
    // aria-hidden; while loaded, none of the three. In default (non-
    // keepMounted) mode the bag contributes nothing (`:101`'s conditional).
    #[wasm_bindgen_test]
    fn the_keep_mounted_status_attributes_follow_the_status() {
        let attrs_of = |status: ImageLoadingStatus, keep: bool| {
            let snapshot = AvatarImageStateSnapshot {
                image_loading_status: status,
                transition_status: None,
                keep_mounted: keep,
                mounted: true,
            };
            let rendered = crate::avatar::avatar_image_element(
                snapshot,
                &crate::avatar::AvatarImageProps {
                    src: Some("/x.png".to_string()),
                    keep_mounted: keep,
                    ..Default::default()
                },
                Vec::new(),
            )
            .expect("mounted → the element exists");
            let (element, cleanup) = rendered.create_element();
            std::mem::forget(cleanup);
            let mut out = std::collections::BTreeMap::new();
            for name in ["data-loading", "data-error", "aria-hidden", "src", "alt"] {
                out.insert(name.to_string(), element.get_attribute(name));
            }
            out
        };

        // Loading (keepMounted): data-loading present, data-error absent,
        // aria-hidden="true", no generic status attribute.
        let loading = attrs_of(ImageLoadingStatus::Loading, true);
        assert_eq!(loading.get("data-loading"), Some(&Some(String::new())));
        assert!(!loading.contains_key("data-error"));
        assert_eq!(loading.get("aria-hidden"), Some(&Some("true".to_string())));
        assert!(!loading.contains_key("data-image-loading-status"));

        // Error (keepMounted): data-error present, data-loading absent.
        let error = attrs_of(ImageLoadingStatus::Error, true);
        assert_eq!(error.get("data-error"), Some(&Some(String::new())));
        assert!(!error.contains_key("data-loading"));
        assert_eq!(error.get("aria-hidden"), Some(&Some("true".to_string())));

        // Loaded (keepMounted): all three status attributes drop.
        let loaded = attrs_of(ImageLoadingStatus::Loaded, true);
        assert!(!loaded.contains_key("data-loading"));
        assert!(!loaded.contains_key("data-error"));
        assert!(!loaded.contains_key("aria-hidden"));

        // Default mode: no status attributes at all, whatever the status
        // (`:101`'s keepMounted scope — the element only exists once loaded,
        // so `data-loading` would be meaningless there).
        let default_loaded = attrs_of(ImageLoadingStatus::Loaded, false);
        assert!(!default_loaded.contains_key("data-loading"));
        assert!(!default_loaded.contains_key("data-error"));
        assert!(!default_loaded.contains_key("aria-hidden"));
    }

    // The three-bag merge order (`:169`'s
    // `[renderedStatusProps, elementProps, sourceProps]`, right-to-left
    // later-wins): a user `alt` (elementProps) survives, and the element
    // carries the sourceProps `src` — plus the bag-2-over-bag-1 precedence
    // that keeps an explicit `aria-hidden` (behavior.md *Accessibility*,
    // `AvatarImage.test.tsx:643-665`).
    #[wasm_bindgen_test]
    fn the_bag_order_puts_user_props_middle_and_source_last() {
        let snapshot = AvatarImageStateSnapshot {
            image_loading_status: ImageLoadingStatus::Loading,
            transition_status: None,
            keep_mounted: true,
            mounted: true,
        };
        let rendered = crate::avatar::avatar_image_element(
            snapshot,
            &crate::avatar::AvatarImageProps {
                src: Some("/final.png".to_string()),
                element_attributes: vec![
                    ("alt".to_string(), "the user".to_string()),
                    ("aria-hidden".to_string(), "focus".to_string()),
                ],
                keep_mounted: true,
                ..Default::default()
            },
            Vec::new(),
        )
        .expect("mounted → the element exists");
        let (element, cleanup) = rendered.create_element();
        std::mem::forget(cleanup);
        // The user's alt lands.
        assert_eq!(
            element.get_attribute("alt").as_deref(),
            Some("the user"),
            "elementProps ride through"
        );
        // The user's explicit aria-hidden (bag 2) overrides the internal
        // status attribute (bag 1) — the explicit-aria-hidden contract.
        assert_eq!(
            element.get_attribute("aria-hidden").as_deref(),
            Some("focus"),
            "the explicit aria-hidden survives (:643-665)"
        );
        // The sourceProps src lands last.
        assert_eq!(element.get_attribute("src").as_deref(), Some("/final.png"));
    }

    // The cached fast path (`AvatarImage.test.tsx:829-837`): a complete
    // probe resolves synchronously inside the machinery effect — the image
    // mounts with its source, and the fallback never renders (no 'loading'
    // report in between; the gate never opened).
    #[wasm_bindgen_test]
    fn a_cached_load_resolves_synchronously_and_hides_the_fallback() {
        let host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/cached.png".to_string()),
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps {
                inner_html: Some("AB".to_string()),
                ..Default::default()
            },
            Some(cached_factory()),
        );
        flush();

        let image = host
            .query_selector("img")
            .unwrap()
            .expect("the loaded image element mounts");
        assert_eq!(
            image.get_attribute("src").as_deref(),
            Some("/cached.png"),
            "the source lands on the element"
        );
        // Spans: the ROOT always remains; the loaded image means the
        // fallback is gone — a bare `query_selector("span")` matched the
        // root, which is always there.
        assert_eq!(
            spans(&host).len(),
            1,
            "only the root span — the fallback never rendered"
        );
        // The suppression mapping (implementation.md untested item 3): no
        // generic status attribute anywhere.
        assert!(
            host.query_selector("[data-image-loading-status]")
                .unwrap()
                .is_none(),
            "no generic status attribute leaks to the DOM"
        );
    }

    // The keepMounted attribute matrix (behavior.md *DOM structure* +
    // *Accessibility*): the element stays in the DOM while loading with
    // data-loading + aria-hidden, and flips to exposed after the load event
    // (`AvatarImage.test.tsx:221-234`, `:595-613` reversed).
    //
    // keepMounted DISABLES the probe (`useImageLoadingStatus.ts:22-25` bails
    // before `new window.Image()` — "no probe is ever constructed"), so the
    // recording sink stays empty; the load is driven on the ELEMENT itself —
    // the upstream keepMounted source of truth (`AvatarImage.tsx:111-116`'s
    // element `onLoad`), wired by the port's element seam. Firing a synthetic
    // `load` on the img dispatches to that listener exactly like the real
    // event.
    #[wasm_bindgen_test]
    fn keep_mounted_carries_the_state_attributes_until_load() {
        let host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/pending.png".to_string()),
                keep_mounted: true,
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps::default(),
            Some(recording_factory(Rc::new(RefCell::new(Vec::new())))),
        );
        flush();

        let image = host
            .query_selector("img")
            .unwrap()
            .expect("keepMounted keeps the img in the DOM while loading");
        assert_eq!(
            image.get_attribute("data-loading").as_deref(),
            Some(""),
            "data-loading is present (bare) while loading"
        );
        assert_eq!(image.get_attribute("data-error"), None);
        assert_eq!(
            image.get_attribute("aria-hidden").as_deref(),
            Some("true"),
            "aria-hidden until loaded — the fallback owns the name"
        );

        // Fire the load ON THE ELEMENT → 'loaded' → the attributes drop
        // (the tracked status read re-materializes the element — the
        // upstream re-render analog).
        image
            .unchecked_ref::<web_sys::HtmlImageElement>()
            .dispatch_event(&Event::new("load").unwrap())
            .unwrap();
        flush();

        let image = host.query_selector("img").unwrap().unwrap();
        assert_eq!(
            image.get_attribute("aria-hidden"),
            None,
            "the loaded image loses aria-hidden and is exposed to AT"
        );
        assert_eq!(image.get_attribute("data-loading"), None);
    }

    // The keepMounted fan-out contract (`AvatarImage.test.tsx:149-174`): the
    // onLoadingStatusChange callback observes 'loading' then 'loaded' — the
    // user callback is a fan-out consumer next to the root mirror. Event
    // driven on the element (keepMounted constructs no probe — see the
    // attribute-matrix test above).
    #[wasm_bindgen_test]
    fn the_status_fan_out_reports_loading_then_loaded() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static SEQ: AtomicU32 = AtomicU32::new(0);

        // The sink is filled from the fan-out effect (machinery owner,
        // single-threaded wasm) AND read from the test — a Cell, not a
        // RefCell borrowed across the call boundary.
        thread_local! {
            static REPORTED: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
        }

        let image_props = crate::avatar::AvatarImageProps {
            src: Some("/reported.png".to_string()),
            keep_mounted: true,
            on_loading_status_change: Some(Rc::new(|status: ImageLoadingStatus| {
                REPORTED.with(|slot| {
                    slot.borrow_mut()
                        .push(Box::leak(status.as_str().to_string().into_boxed_str()));
                    SEQ.fetch_add(1, Ordering::SeqCst);
                });
            })),
            ..Default::default()
        };
        let host = mount_avatar(
            image_props,
            crate::avatar::AvatarFallbackProps::default(),
            Some(recording_factory(Rc::new(RefCell::new(Vec::new())))),
        );
        flush();

        // The initial 'loading' report.
        {
            let reported = REPORTED.with(|slot| slot.borrow().clone());
            assert_eq!(
                reported,
                vec!["loading"],
                "the first report is 'loading' (:160-173)"
            );
        }

        let image = host.query_selector("img").unwrap().unwrap();
        image
            .unchecked_ref::<web_sys::HtmlImageElement>()
            .dispatch_event(&Event::new("load").unwrap())
            .unwrap();
        flush();

        let reported = REPORTED.with(|slot| slot.borrow().clone());
        assert_eq!(
            reported,
            vec!["loading", "loaded"],
            "'loading' then 'loaded' on load (:168-173)"
        );
        let _ = SEQ.load(Ordering::SeqCst);
        // The aria contract rode the same transition (keepMounted).
        let image = host.query_selector("img").unwrap().unwrap();
        assert_eq!(image.get_attribute("aria-hidden"), None);
    }

    // The delay latch through the real timer (`AvatarFallback.test.tsx
    // :100-201`): `delay: 50` hides the fallback past the synchronous flush,
    // shows it once the `useTimeout` fires; `delay: 0` renders it
    // synchronously (`:120-132`). Async wasm test over the browser's real
    // `setTimeout` (the delay path is user-perceived timing, not
    // pre-paint-critical — upstream uses the plain `useEffect` there).
    #[wasm_bindgen_test]
    async fn the_delay_latch_gates_the_fallback_through_the_real_timer() {
        let sleep = |ms: i32| {
            let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(resolve.unchecked_ref(), ms)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise)
        };

        // delay: 0 → mounted synchronously (root + fallback spans).
        let zero_host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/zero.png".to_string()),
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps {
                delay: 0.0,
                inner_html: Some("AB".to_string()),
                ..Default::default()
            },
            Some(recording_factory(Rc::new(RefCell::new(Vec::new())))),
        );
        flush();
        assert_eq!(
            spans(&zero_host).len(),
            2,
            "delay 0 renders the fallback synchronously (:120-132)"
        );

        // delay: 50 → hidden at mount time, shown once the timeout fires
        // (`:100-118`).
        let host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/delayed.png".to_string()),
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps {
                delay: 50.0,
                inner_html: Some("CD".to_string()),
                ..Default::default()
            },
            Some(recording_factory(Rc::new(RefCell::new(Vec::new())))),
        );
        flush();
        assert_eq!(
            spans(&host).len(),
            1,
            "a pending delay keeps the fallback hidden at mount"
        );
        sleep(120).await;
        flush();
        assert_eq!(
            spans(&host).len(),
            2,
            "the fallback shows once the delay elapses (:105-118)"
        );
        let all_spans = spans(&host);
        assert_eq!(all_spans[1].text_content().as_deref(), Some("CD"));
    }

    // The request-config assignment on the probe (`AvatarImage.test.tsx
    // :135-147`): the probe receives `sizes`, `srcset`, `src` — the
    // configure order the port copies verbatim (`useImageLoadingStatus.ts
    // :46-58`). Driven through the SURFACING fake (an element exposed back
    // to `configure_probe`); the src assignment starts a harmless fetch in
    // the test page but the assertions read attributes set synchronously.
    #[wasm_bindgen_test]
    fn the_probe_receives_the_request_configuration() {
        let probes = Rc::new(RefCell::new(Vec::new()));
        let _host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/configured.png".to_string()),
                sizes: Some("50vw".to_string()),
                src_set: Some("a.png 1x, b.png 2x".to_string()),
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps::default(),
            Some(surfacing_factory(probes.clone())),
        );
        flush();

        let probes = probes.borrow();
        assert_eq!(
            probes.len(),
            1,
            "exactly one probe constructed for one source"
        );
        let probe = &probes[0];
        assert_eq!(
            probe.get_attribute("sizes").as_deref(),
            Some("50vw"),
            "sizes lands on the probe (:50-52)"
        );
        assert_eq!(
            probe.get_attribute("srcset").as_deref(),
            Some("a.png 1x, b.png 2x"),
            "srcset lands on the probe (:53-55)"
        );
        assert_eq!(
            probe.get_attribute("src").as_deref(),
            Some("/configured.png"),
            "src lands on the probe, LAST (:56-58)"
        );
    }

    // The default-mode fallback cycle (`AvatarFallback.test.tsx:203-238`):
    // while the probe is pending, the fallback is mounted and the image
    // element is absent (the detached probe carries the load); an error
    // keeps exactly that shape (`:277-285`), and the fallback's children
    // render (`:53-66`).
    //
    // Default (probe) mode never constructs the rendered img, so the probe
    // events must be driven through the recording sink (the fake's element)
    // — the element-level fire below belongs to keepMounted only.
    #[wasm_bindgen_test]
    fn loading_shows_the_fallback_and_the_default_mode_image_stays_absent() {
        let probes = Rc::new(RefCell::new(Vec::new()));
        let host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/slow.png".to_string()),
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps {
                inner_html: Some("AB".to_string()),
                ..Default::default()
            },
            Some(recording_factory(probes.clone())),
        );
        flush();

        // Spans: [0] the ROOT span, [1] the fallback — the bare
        // `query_selector("span")` matched the root.
        let all_spans = spans(&host);
        assert_eq!(
            all_spans.len(),
            2,
            "root + fallback mount while loading (img absent)"
        );
        assert_eq!(
            all_spans[1].text_content().as_deref(),
            Some("AB"),
            "the fallback's children render (the :53-66 contract)"
        );
        assert!(
            host.query_selector("img").unwrap().is_none(),
            "the img element is absent while loading (the probe carries it)"
        );

        // Fire the error on the live probe → status 'error' → the fallback
        // stays, and still no img element (the image never mounted).
        let probe = probes.borrow()[0].clone();
        probe.dispatch_event(&Event::new("error").unwrap()).unwrap();
        flush();

        let all_spans = spans(&host);
        assert_eq!(
            all_spans.len(),
            2,
            "the fallback remains after the error"
        );
        assert!(
            host.query_selector("img").unwrap().is_none(),
            "an errored default-mode image never enters the DOM"
        );
    }

    // The exclusivity regression (behavior.md *Edge cases*,
    // `AvatarFallback.test.tsx:240-296`): switching to the loaded image
    // unmounts the fallback — exactly one of image or fallback is in the
    // DOM after the load, no double exposure. The root span always remains.
    #[wasm_bindgen_test]
    fn exactly_one_of_image_or_fallback_after_the_load() {
        let probes = Rc::new(RefCell::new(Vec::new()));
        let host = mount_avatar(
            crate::avatar::AvatarImageProps {
                src: Some("/eventual.png".to_string()),
                ..Default::default()
            },
            crate::avatar::AvatarFallbackProps {
                inner_html: Some("AB".to_string()),
                ..Default::default()
            },
            Some(recording_factory(probes.clone())),
        );
        flush();

        assert_eq!(
            spans(&host).len(),
            2,
            "root + fallback while loading"
        );
        assert!(host.query_selector("img").unwrap().is_none());

        let probe = probes.borrow()[0].clone();
        probe.dispatch_event(&Event::new("load").unwrap()).unwrap();
        flush();

        assert!(
            host.query_selector("img").unwrap().is_some(),
            "the image mounts once loaded"
        );
        assert_eq!(
            spans(&host).len(),
            1,
            "only the root span remains — the fallback unmounted, exactly one of the two"
        );
    }

    // The unmount reset (`:131-133`, behavior.md *Edge cases*
    // `AvatarFallback.test.tsx:68-98`): unmounting a loaded image resets the
    // root to 'idle' — the fallback reappears.
    //
    // The port's reset rides a leptos-side `on_cleanup` under the image's
    // mount owner, so the contract is exercised by dropping the mount's
    // `UnmountHandle` (leptos-0.7.8 mount.rs:230-235: drop → unmount +
    // owner cleanup). The mount body provides the ROOT CONTEXT itself (not
    // `use_avatar_root`, which would provide its own fresh 'idle' signal and
    // shadow this one) over a status signal the test holds — the upstream
    // root component's state, surviving its children's unmount. After the
    // image loads (fallback hides), dropping the mount must (a) remove the
    // tree and (b) write 'idle' back into the shared signal before it dies —
    // exactly `:133`'s `setRootImageLoadingStatus('idle')` on cleanup.
    //
    // keepMounted keeps the img element in the tree so one mount owns both
    // parts; the load fires on the element (keepMounted constructs no probe
    // — `useImageLoadingStatus.ts:22-25`).
    #[wasm_bindgen_test]
    fn unmounting_the_image_makes_the_fallback_reappear() {
        let _ = any_spawner::Executor::init_futures_executor();
        let host = container();

        // The root status — the test's window into the context the parts
        // read and the image machinery writes.
        let root_status: leptos::prelude::RwSignal<ImageLoadingStatus> =
            leptos::prelude::RwSignal::new(ImageLoadingStatus::Idle);

        let image_props = crate::avatar::AvatarImageProps {
            src: Some("/doomed.png".to_string()),
            keep_mounted: true,
            ..Default::default()
        };
        // Two independent copies of the same config: one for the body's
        // `use_avatar_image` read, one moved into the view closure (the
        // props are static per body run — the port's documented adaptation).
        let image_props_for_build = crate::avatar::AvatarImageProps {
            src: Some("/doomed.png".to_string()),
            keep_mounted: true,
            ..Default::default()
        };
        let fallback_props = crate::avatar::AvatarFallbackProps {
            inner_html: Some("AB".to_string()),
            ..Default::default()
        };
        let fallback_props_for_build = crate::avatar::AvatarFallbackProps {
            inner_html: Some("AB".to_string()),
            ..Default::default()
        };

        let handle = with_probe_factory(recording_factory(Rc::new(RefCell::new(Vec::new()))), || {
            leptos::mount::mount_to(host.clone().unchecked_into(), move || {
                // The context over the SHARED signal — no `use_avatar_root`
                // here, so this provide is the one the parts resolve.
                crate::avatar::provide_avatar_root_context(
                    crate::avatar::AvatarRootContextValue {
                        image_loading_status: root_status,
                        set_image_loading_status: root_status,
                    },
                );
                let image_handle = use_avatar_image(&image_props_for_build);
                let fallback_handle = use_avatar_fallback(&fallback_props_for_build);
                view! {
                    // Real dynamic-view children (the closure invoked per
                    // reactive run) — see mount_avatar's comment.
                    {dynamic(avatar_image_view(image_handle, image_props))}
                    {dynamic(avatar_fallback_view(fallback_handle, fallback_props))}
                }
            })
        });
        flush();

        // While loading: the keepMounted img is mounted with its status
        // attributes, the fallback span is beside it (status not loaded).
        let image = host
            .query_selector("img")
            .unwrap()
            .expect("keepMounted keeps the img mounted while loading");
        assert_eq!(spans(&host).len(), 1, "the fallback shows while loading");

        // Load the image on the element → 'loaded' → the fallback hides.
        image
            .unchecked_ref::<web_sys::HtmlImageElement>()
            .dispatch_event(&Event::new("load").unwrap())
            .unwrap();
        flush();
        assert_eq!(spans(&host).len(), 0, "loaded → the fallback hides");
        assert_eq!(
            leptos::prelude::GetUntracked::get_untracked(&root_status),
            ImageLoadingStatus::Loaded,
            "the fan-out mirrored 'loaded' into the shared root status"
        );

        // Drop the mount: the tree unmounts (the img leaves the DOM) and
        // the leptos-side on_cleanup reset writes 'idle'.
        drop(handle);
        flush();

        assert!(
            host.query_selector("img").unwrap().is_none(),
            "the unmount removed the tree (UnmountHandle::drop → unmount)"
        );
        assert_eq!(
            leptos::prelude::GetUntracked::get_untracked(&root_status),
            ImageLoadingStatus::Idle,
            "the unmount reset (:131-133) wrote 'idle' into the root status"
        );
    }
}
