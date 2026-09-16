//! The namespaced view surface's browser suite — `OTPField::Root` / `::Input` / `::Separator`
//! (`crates/leptos-ui/src/otp_field.rs`, "The namespaced view surface").
//!
//! WHY THIS SUITE EXISTS, SEPARATELY FROM THE PAGE'S
//! ------------------------------------------------
//! The docs page's otp-field suite (`crates/docs-app/src/render_test.rs`) mounts the page's
//! composition root — the composition assembled by hand in
//! `crates/docs-app/src/pages/otp_field_page.rs:306-375`, precisely because the owner crate had no
//! view layer. That suite therefore proves the *hooks* work; it cannot prove the *surface* works,
//! because it never calls it. The surface is new machinery with two seams that fail silently when
//! wrong: (1) the slot registry is provided in an rg-0.2 owner window the children construct inside
//! (`provide_otp_composite_list` + `use_composite_list_item`, both owner-scoped), and (2) the merged
//! bag, the engine's handler slots and the ref fork are written by a commit effect onto a
//! leptos-created node. A wrong window gives every slot the wrong index (wrong id, wrong value slice,
//! wrong roving tabindex); a dropped description unregisters the `input`/`paste` write path. Both are
//! invisible to a compile pin, so they are measured here in Chrome for Testing: the mounted DOM's own
//! attribute surface, and one real keystroke through the write path.
//!
//! Every test flushes one settled turn after mounting, because the bag/ref commit is an effect (the
//! `update_element` writer) rather than part of the synchronous tree — the same reason the sibling
//! suites flush before asserting on attributes.
//!
//! Runtime behaviour beyond this (the full keystroke/focus matrix) is the unit's own business:
//! `specs/library/otp-field/behavior.md` is mined from upstream's tests and is already pinned by the
//! page's suite. This file pins what only the namespaced surface can be wrong about.

#![allow(unused_imports, dead_code)]

use super::*;

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use std::rc::Rc;

    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement, HtmlInputElement};

    use super::*;
    // The capitalised alias, i.e. exactly the path a consumer writes.
    use crate::OTPField;
    use leptos::mount::mount_to;
    use leptos::prelude::*;
    use leptos_ui_utils::use_merged_refs::RefCallback;

    wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().expect("window").document().expect("document")
    }

    /// One settled macrotask turn (the docs-app `render_test.rs` convention), so the mount's commit
    /// effect has written the element's bag and fired its ref fork.
    async fn flush_one_turn() {
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            web_sys::window()
                .expect("window")
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                .expect("set_timeout");
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.ok();
    }

    /// One animation frame plus the crate-local settle (the docs-app `render_test.rs`
    /// `flush_one_frame`): an effect that RE-RUNS after a signal write is scheduled through
    /// leptos's own task queue, so it lands after `leptos::task::tick()` has drained it — a bare
    /// macrotask turn is not enough to observe a post-commit re-run (`flush_one_turn` alone made
    /// the commit-queue drain look inert).
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
        for _ in 0..32 {
            any_spawner::Executor::poll_local();
        }
        leptos::task::tick().await;
        for _ in 0..32 {
            any_spawner::Executor::poll_local();
        }
        flush_one_turn().await;
    }

    /// The settle sequence every post-interaction assertion uses.
    async fn settle() {
        flush_one_frame().await;
        flush_one_turn().await;
    }

    /// The upstream-style keystroke: set the input's value and dispatch a bubbling `input` event —
    /// the port's write path reads it off the element (`otp_field.rs`'s ref-fork handlers).
    fn type_into(input: &HtmlInputElement, text: &str) {
        input.set_value(text);
        let init = web_sys::EventInit::new();
        init.set_bubbles(true);
        let event = web_sys::Event::new_with_event_init_dict("input", &init).expect("input event");
        input.dispatch_event(&event).expect("dispatch input");
    }

    fn mount_container() -> HtmlElement {
        let container = document()
            .create_element("div")
            .expect("create container")
            .dyn_into::<HtmlElement>()
            .expect("div as HtmlElement");
        document().body().expect("body").append_child(&container).expect("append container");
        container
    }

    /// Mounts three slots with a separator between the second and the third, through the NAMESPACED
    /// parts — the tree upstream's docs teach (`OTPFieldRoot.test.tsx:94-118`).
    fn mount_named_parts() -> HtmlElement {
        let _ = any_spawner::Executor::init_futures_executor();
        let container = mount_container();
        // The mount guard is forgotten rather than dropped: dropping it unmounts (the
        // `std::mem::forget(mount_to(…))` convention of the sibling wasm suites).
        std::mem::forget(mount_to({ container.clone() }, || {
            view! {
                <OTPField::Root length=3 id="code".to_string()>
                    <OTPField::Input />
                    <OTPField::Input />
                    <OTPField::Separator>"-"</OTPField::Separator>
                    <OTPField::Input aria_label="Character 3 of 3".to_string() />
                </OTPField::Root>
            }
        }));
        container
    }

    fn slots(container: &HtmlElement) -> Vec<HtmlInputElement> {
        let list = container.query_selector_all("input").expect("query slots");
        (0..list.length())
            .map(|index| {
                list.item(index)
                    .expect("slot at index")
                    .dyn_into::<HtmlInputElement>()
                    .expect("slot as input")
            })
            .collect()
    }

    fn query(container: &HtmlElement, selector: &str) -> Element {
        container
            .query_selector(selector)
            .expect("query")
            .unwrap_or_else(|| panic!("expected a `{selector}` element"))
    }

    fn active_element() -> Option<Element> {
        document().active_element()
    }

    /// THE NESTING CLAIM: the slots are real children of the `role="group"` root element — the
    /// provider-wrapped subtree upstream's JSX expresses and the element-only path could not
    /// (`OtpFieldRootProps` carries no `children`). A view layer that appended the slots as siblings
    /// of the root would fail here, and `getByRole('group')`-shaped queries would stop containing
    /// them (`behavior.md:82`).
    #[wasm_bindgen_test]
    async fn the_namespaced_root_nests_its_slots_inside_the_group_element() {
        let container = mount_named_parts();
        flush_one_turn().await;

        let group = query(&container, "[role=group]");
        let slots = slots(&container);
        assert_eq!(slots.len(), 3, "three Inputs are three slots, the separator is not one");
        for (index, slot) in slots.iter().enumerate() {
            assert_eq!(
                slot.parent_element().as_ref(),
                Some(&group),
                "slot {index} is a direct child of the group, not a sibling of it"
            );
        }
    }

    /// THE REGISTRY CLAIM: the slots carry the slot indexes their render order implies, which is the
    /// observable signature of the owner window the registry is provided in. Wrong indexes show up as
    /// duplicated ids, a missing first-slot `autocomplete`/`maxlength`, or every slot on
    /// `tabindex=0`. Each assertion mirrors a behavior-spec claim.
    #[wasm_bindgen_test]
    async fn the_namespaced_slots_carry_the_documented_slot_attribute_surface() {
        let container = mount_named_parts();
        flush_one_turn().await;
        let slots = slots(&container);
        assert_eq!(slots.len(), 3, "three Inputs are three slots");

        // `behavior.md:96` — the ids derive from the root id (`{id}` then `{id}-{index+1}`).
        assert_eq!(slots[0].id(), "code", "the first slot inherits the root id");
        assert_eq!(slots[1].id(), "code-2", "later slots derive their ids from the root id");

        // `behavior.md:89` — first slot `one-time-code`, the rest `off`.
        assert_eq!(slots[0].get_attribute("autocomplete").as_deref(), Some("one-time-code"));
        assert_eq!(slots[1].get_attribute("autocomplete").as_deref(), Some("off"));

        // `behavior.md:103` — only the first slot carries `maxlength`.
        assert_eq!(slots[0].get_attribute("maxlength").as_deref(), Some("3"));
        assert_eq!(slots[1].get_attribute("maxlength"), None, "later slots carry no maxlength");

        // `behavior.md:91` — the built-in `numeric` validation type sets a single-character pattern.
        assert!(
            slots[0].get_attribute("pattern").is_some(),
            "the validation pattern rides the slots"
        );

        // The roving tabindex (`activeIndex`, `OTPFieldInput.tsx:102`): exactly one slot is tabbable.
        let tabbable: Vec<String> =
            slots.iter().filter_map(|slot| slot.get_attribute("tabindex")).collect();
        assert_eq!(
            tabbable.iter().filter(|value| value.as_str() == "0").count(),
            1,
            "exactly one slot is tabbable, got {tabbable:?}"
        );
    }

    /// THE SEPARATOR CLAIM: its children render inside the separator element and it consumes no slot
    /// index — upstream's "supports grouped layouts without affecting slot counting"
    /// (`OTPFieldRoot.test.tsx:94-118`, `behavior.md:20`).
    #[wasm_bindgen_test]
    async fn the_namespaced_separator_renders_its_children_without_consuming_a_slot() {
        let container = mount_named_parts();
        flush_one_turn().await;

        let separator = query(&container, "[role=separator]");
        assert_eq!(separator.text_content().as_deref(), Some("-"), "its own child text renders");
        assert_eq!(
            slots(&container).len(),
            3,
            "three Inputs are three slots — the separator consumes none"
        );
    }

    /// THE WRITE PATH AND THE REGISTRY, END TO END, THROUGH THE NAMESPACED COMPOSITION.
    ///
    /// Two claims, both of them the surface's own:
    ///
    /// 1. A real keystroke reaches the port's write path and commits — the Input's `input` listener
    ///    lives in the ref fork the commit effect fires, so a dropped description or a detached node
    ///    turns this into a no-op instead of an error.
    /// 2. The registry the Root provided is the one the port's focus API reads: calling the provided
    ///    context's own `focusInput(index)` moves focus to the slot at that index. That is the
    ///    allocation the root view owns (`provide_otp_composite_list` handing over the root's own
    ///    `inputRefs`, "The three seams" step 2) — a wrong or detached list makes this land nowhere.
    ///
    /// NOT asserted here, deliberately: the caret advancing to the next slot *implicitly* when a
    /// character is accepted (`behavior.md:55`). The port drops that queued focus at its stale-mirror
    /// guard — a DIAGNOSED, still-open gap owned by the library item and recorded in
    /// `ralph/logs/spec-discrepancies.md:442-454` and in-line at `otp_field.rs:724-731` ("the drains
    /// below compare a queued value against `VALUE`, … so the comparison sees the PREVIOUS value and a
    /// queued focus/completion is dropped as stale"). This suite asserts the registry directly
    /// instead, rather than pinning a behavior the port does not perform — the surface must not paper
    /// over the crate's own gap, and this test is not the place that gap gets quietly declared fixed.
    #[wasm_bindgen_test]
    async fn a_keystroke_through_the_namespaced_parts_commits_and_the_registry_drives_focus() {
        let container = mount_named_parts();
        flush_one_turn().await;

        let slots = slots(&container);
        slots[0].focus().expect("focus slot 0");
        type_into(&slots[0], "7");
        for _ in 0..3 {
            flush_one_turn().await;
        }
        settle().await;
        settle().await;
        assert_eq!(
            slots[0].value(),
            "7",
            "the keystroke reached the port's write path through the namespaced parts and committed"
        );

        // Claim 1 — `behavior.md:55`: "Typing a character into a slot fills it and moves focus to
        // the next slot." The advance is the commit-queue drain running `focusInput(index + 1)`
        // through the Root's registry, so this is also the registry's end-to-end proof: a detached
        // or empty list makes the queue land nowhere and the caret stays on slot 0.
        assert_eq!(
            active_element().map(|element| element.id()),
            Some(slots[1].id()),
            "the caret advanced to the second slot after the commit"
        );

        // Claim 2 — `behavior.md:29`/`:55`: the next character lands in the slot the caret reached,
        // composed with the committed value (upstream's write base is `valueRef.current`, `:166`).
        type_into(&slots[1], "8");
        settle().await;
        settle().await;
        assert_eq!(slots[0].value(), "7", "the first character is still in slot 0");
        assert_eq!(slots[1].value(), "8", "the second character landed in slot 1");
        let context = use_otp_field_root_context();
        assert_eq!(
            (context.get_value)(),
            "78",
            "the committed value accumulates across slots"
        );
        assert_eq!(
            active_element().map(|element| element.id()),
            Some(slots[2].id()),
            "the caret advanced again"
        );

        // Claim 3 — `behavior.md:70`: "Focusing a later empty slot redirects focus to the first
        // empty slot." The value is two characters long, so slots 2 and 3 are the empty tail:
        // `focusInput(3)` must land on slot 2, and that move IS the registry read (`focusInput`
        // resolves its target from the root's registered list).
        (context.focus_input)(3);
        assert_eq!(
            active_element().map(|element| element.id()),
            Some(slots[2].id()),
            "focusInput(index) reads the root view's registry and the redirect clamps to the first \
             empty slot"
        );
        (context.focus_input)(0);
        assert_eq!(
            active_element().map(|element| element.id()),
            Some(slots[0].id()),
            "a slot inside the value length is adopted as-is"
        );
    }

    /// TEMPORARY MEASUREMENT (this iteration only, removed before the done-marking): the two
    /// leftover diagnostics are replaced by one measurement of the registry seam the suite's own
    /// claim rests on — how many slots the root's published list actually holds after the mount
    /// settles, and where `focusInput(2)` lands.
    #[wasm_bindgen_test]
    async fn probe_registry_and_caret() {
        let container = mount_named_parts();
        flush_one_turn().await;
        let slots = slots(&container);
        let registry_len = crate::otp_field::debug_registry_len();
        let registry_ids = crate::otp_field::debug_registry_ids();
        let dom_ids: Vec<String> = slots.iter().map(|slot| slot.id()).collect();
        slots[0].focus().expect("focus slot 0");
        type_into(&slots[0], "7");
        for _ in 0..3 {
            flush_one_turn().await;
        }
        let after_type = active_element().map(|element| element.id());
        let registry_len_after_type = crate::otp_field::debug_registry_len();
        let mirror_after_type = crate::otp_field::debug_value_mirror();
        let counts = crate::otp_field::debug_counts();
        settle().await;
        settle().await;
        let mirror_after_settle = crate::otp_field::debug_value_mirror();
        let counts_after_settle = crate::otp_field::debug_counts();
        let active_after_settle = active_element().map(|element| element.id());
        let context = use_otp_field_root_context();
        let live_value_read = (context.get_value)();
        (context.focus_input)(2);
        let after_focus2 = active_element().map(|element| element.id());
        // And the same move requested directly on the DOM node, to separate "the registry entry is
        // not focusable" from "the port's own focus handler redirected the focus".
        let _ = slots[2].focus();
        let after_dom_focus2 = active_element().map(|element| element.id());
        panic!(
            "PROBE registry_len={registry_len:?} registry_ids={registry_ids:?} dom_ids={dom_ids:?} \
             after_type={after_type:?} registry_len_after_type={registry_len_after_type:?} \
             mirror_after_type={mirror_after_type:?} length_mirror={} after_focus2={after_focus2:?} \
             after_dom_focus2={after_dom_focus2:?} counts={counts:?} live_value={live_value_read:?} \
             mirror_after_settle={mirror_after_settle:?} counts_after_settle={counts_after_settle:?} \
             active_after_settle={active_after_settle:?}",
            crate::otp_field::debug_length_mirror(),
        );
    }

    /// The forwarded `ref` fires with the root node — the surface's third seam (the commit effect
    /// fires the description's ref fork, which carries both the caller's ref and the root-element
    /// recorder `requestSubmit`'s ancestor-form lookup reads, `otp_field.rs:933-955`).
    #[wasm_bindgen_test]
    async fn the_root_forwards_its_ref_to_the_mounted_group_element() {
        let seen: Rc<std::cell::RefCell<Option<String>>> = Rc::new(std::cell::RefCell::new(None));
        let seen_for_ref = Rc::clone(&seen);

        let _ = any_spawner::Executor::init_futures_executor();
        let container = mount_container();
        std::mem::forget(mount_to({ container.clone() }, move || {
            let ref_callback: RefCallback<Element> = Rc::new(move |element: Option<&Element>| {
                *seen_for_ref.borrow_mut() =
                    element.map(|element| element.get_attribute("role").unwrap_or_default());
                None
            });
            view! {
                <OTPField::Root length=1 ref_callback=ref_callback>
                    <OTPField::Input />
                </OTPField::Root>
            }
        }));
        flush_one_turn().await;

        assert_eq!(
            seen.borrow().as_deref(),
            Some("group"),
            "the forwarded ref received the `role=\"group\"` root element"
        );
    }
}
