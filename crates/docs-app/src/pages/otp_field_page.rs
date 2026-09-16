//! The docs page for `OTP Field`, mirroring
//! `docs/src/app/(docs)/react/components/otp-field/page.mdx`
//! (`specs/docs-content/otp-field/page.md`) — the
//! `docs-content: components/otp-field` TODO item.
//!
//! Page structure per the spec's "Page structure (headings, in order)" section:
//! `# OTP Field` h1, the `<Subtitle>` ("A one-time password input composed of
//! individual character slots."), the hero demo before the first heading,
//! `## Usage guidelines`, `## Anatomy` with its fenced snippet, `## Examples`
//! over "Labeling an OTP field" (`page.mdx:33-54`), "Form integration"
//! (`:56-78`), "Alphanumeric verification codes" (`:80-87`), "Grouped layouts"
//! (`:89-96`), "Placeholder hints" (`:98-105`), "Custom normalization"
//! (`:107-118`) and "Masked entry" (`:120-126`), and `## API reference`
//! (`:128-142`) over the three generated `TypesOTPField` references
//! (`docs/src/app/(docs)/react/components/otp-field/types.md`) echoed as static
//! prose per the accordion/button/meter/field/checkbox/checkbox-group page
//! precedent: the port has no docs generator, so each table's summary line,
//! props and data attributes are rendered as text, never fabricated as
//! executable machinery.
//!
//! Page furniture mirrored in module docs (the checkbox/checkbox-group
//! precedent): the `<Meta name="description">` content — "A high-quality,
//! unstyled React OTP field component for one-time password and verification
//! code entry." (`page.mdx:5-8`) — and the trailing `export const metadata` SEO
//! keywords block (12 keywords, `page.mdx:144-159`: 'React OTP Field',
//! 'One-Time Password Input', 'Verification Code Input', 'OTP Input Component',
//! 'Pin Input', 'Passcode Input', 'Two-Factor Authentication Input', '2FA Input',
//! 'Multi-Slot Input', 'Accessible OTP Field', 'Headless React Components',
//! 'Base UI').
//!
//! ## The pair's composition root (the surface the owner crate did not have)
//!
//! `crates/leptos-ui/src/otp_field.rs` ports all three parts, but the Root is
//! element-only: `use_otp_field_root` returns the root's
//! [`RenderedElement`] description and its props struct carries no `children`
//! (`OtpFieldRootProps`, `otp_field.rs:236-296`), while upstream's Root is a
//! PROVIDER wrapped around its subtree — every demo on this page nests its
//! `OTPField.Input` slots INSIDE `<OTPField.Root>`
//! (`OTPFieldRoot.tsx:394-400`: the `CompositeList` + `OTPFieldRootContext`
//! providers wrap `element`). The composition is assembled here, out of the
//! crate's own public seams, in the order the port's internals require:
//!
//! 1. [`use_otp_field_root`] — runs the whole Root body, which PROVIDES the
//!    root context before returning (`otp_field.rs:830-832`; the parts' required
//!    read, `otp_field.rs:348-353`).
//! 2. [`provide_otp_composite_list`] — the slot registry the Inputs register
//!    into, which is the ROOT's own `inputRefs`: `CompositeList`'s `elementsRef`
//!    IS `OTPFieldRoot.tsx:103`'s array, the ordered list `focusInput` (`:163-168`)
//!    reads, so the port hands that handle over (`INPUT_REFS`) and the view layer
//!    provides it here rather than creating a second, detached list. The ORDER is
//!    upstream's: the provide sits above the slots because the Input's index is
//!    claimed at hook-call time (`use_composite_list_item.rs`: "hook-call order
//!    standing in for render order"). With the right identity, the focus queue an
//!    accepted character leaves behind (`:206-212`) lands on the next slot.
//! 3. `RenderedElement::create_element()` on the root, then each slot's own
//!    `create_element()` appended INSIDE it — the DOM nesting upstream's JSX
//!    expresses. `create_element` (`use_render_element.rs:536-575`) materializes
//!    the merged bag (class/style/attrs), attaches the handler slots and fires
//!    the ref fork, which is where the port hangs the Input's `input`/`paste`
//!    write path (`otp_field.rs:1341`, `:1407`).
//!
//! This is the same PAIR-PORTABILITY GAP the checkbox-group pair hit, resolved
//! the same way: real parts only, no demo-side reimplementation of the
//! component. Nothing here re-derives an OTP behavior — the slots are the
//! crate's `use_otp_field_input`, the divider is the crate's shared Separator
//! (which is what `OTPField.Separator` IS: `otp_field_separator` is a re-export
//! of it, `otp_field.rs:1529-1534`).
//!
//! ## Documented adaptations (never silent)
//!
//! - **`React.useId()` → `use_base_ui_id`.** Every demo upstream calls
//!   `React.useId()` for the label/description ids and passes it as the Root's
//!   `id`. The port's equivalent is the internals crate's `use_base_ui_id` (the
//!   checkbox-group page's convention). The slot ids then derive exactly as
//!   types.md documents ("Subsequent inputs derive their ids from it
//!   (`{id}-2`, `{id}-3`, ...)", `types.md:33`), via the port's own
//!   `get_input_id` (`otp_field.rs:760-772`).
//! - **`OTPField.Separator className`.** The grouped demo passes the separator a
//!   `className`; `otp_field_separator()` takes no props and renders the shared
//!   Separator's defaults (`otp_field.rs:1532-1534`), so the demo calls the same
//!   real part through `separator_element` with the demo's class — the identical
//!   element, one layer of convenience removed (the separator page's own call
//!   shape).
//! - **The custom-sanitize demo's consumer handlers.** That demo passes
//!   `onFocus` per slot and computes each slot's `className` from its own
//!   `useInvalidFeedback` state. The port's `OtpFieldInputProps` carries the
//!   consumer's static `...elementProps` rest but no per-slot handler slots, so
//!   the demo attaches its own `focus` listener to the materialized slot node and
//!   writes the class with an effect — the consumer's prop, wired one layer out
//!   at the node it belongs to. Recorded in `ralph/logs/spec-discrepancies.md`.
//! - **CSS-modules variants.** `custom-sanitize` ships only a `css-modules`
//!   variant (`demos.json` entry 2's own discrepancy note; the other five ship
//!   Tailwind variants), so its module class names (`Field`, `Label`, `Root`,
//!   `Input`, `InvalidPulseA`/`InvalidPulseB`, `Description`, `ScreenReaderOnly`)
//!   are carried as literal class strings the way the Tailwind variants' class
//!   strings are — docs-app ships no stylesheet at all, so the DEMO's
//!   element-for-element DOM shape is what these ports pin (the checkbox-group
//!   page precedent).
//! - **`Form`/`Field` in the Form-integration section.** That section renders an
//!   inline snippet (no demo component upstream), so it is mirrored as the
//!   verbatim fenced snippet — no stub `Form` machinery is fabricated.

use crate::code_block::{Lang, code_block};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{Element, Event};
use leptos_ui::{
    NormalizeValueFn, OtpChangeEventDetails, OtpFieldInputProps, OtpFieldRootProps,
    OtpGenericEventDetails, OtpValidationType, SeparatorProps, provide_otp_composite_list,
    separator_element, use_otp_field_input, use_otp_field_root,
};
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderedElement, UseRenderElementComponentProps,
};
use leptos_ui_utils::CleanupFn;

use crate::pages::use_render_page::RawElementView;

/// The demo slot count (`const OTP_LENGTH = 6` / `CODE_LENGTH = 6` in every
/// demo).
const OTP_LENGTH: usize = 6;

/// The demos' field container `className` (`hero/tailwind/index.tsx:9`, shared
/// by the hero, alphanumeric, focused-placeholder and password demos).
const FIELD_CLASS: &str = "flex w-full max-w-80 flex-col items-start gap-1";

/// The demos' `<label>` `className` (`hero/tailwind/index.tsx:10`).
const LABEL_CLASS: &str = "text-sm font-bold text-neutral-950 dark:text-white";

/// The demos' supporting-text `className` (`hero/tailwind/index.tsx:26`).
const DESCRIPTION_CLASS: &str = "m-0 text-sm text-neutral-600 dark:text-neutral-400";

/// The Root `className` of the row-layout demos (`hero/tailwind/index.tsx:16`).
const ROOT_CLASS: &str = "flex w-full gap-2";

/// The Root `className` of the grouped demo (`grouped/tailwind/index.tsx:11`,
/// which also centres its children).
const GROUPED_ROOT_CLASS: &str = "flex w-full items-center gap-2";

/// The per-slot `className` (`hero/tailwind/index.tsx:21`, shared verbatim by
/// the hero, alphanumeric, grouped and password demos).
const INPUT_CLASS: &str = "m-0 h-10 w-10 rounded-none border border-neutral-950 bg-white dark:bg-neutral-950 text-center font-inherit text-base font-normal text-neutral-950 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white dark:border-white dark:text-white";

/// The placeholder demo's per-slot `className`
/// (`focused-placeholder/tailwind/index.tsx:21`) — the input class plus the
/// placeholder colour and the `focus:placeholder:text-transparent` rule the
/// page's prose describes.
const PLACEHOLDER_INPUT_CLASS: &str = "m-0 h-10 w-10 rounded-none border border-neutral-950 bg-white dark:bg-neutral-950 text-center font-inherit text-base font-normal text-neutral-950 placeholder:text-neutral-500 focus:outline-2 focus:-outline-offset-1 focus:outline-neutral-950 dark:focus:outline-white focus:placeholder:text-transparent dark:border-white dark:text-white dark:placeholder:text-neutral-400";

/// The placeholder demo's slot hint (`focused-placeholder/tailwind/index.tsx:22`).
const PLACEHOLDER: &str = "•";

/// The grouped demo's inner group `className` (`grouped/tailwind/index.tsx:12`).
const GROUP_CLASS: &str = "flex gap-2";

/// The grouped demo's Separator `className` (`grouped/tailwind/index.tsx:25`) —
/// the 1px dash the page presents the code around.
const SEPARATOR_CLASS: &str = "h-px w-4 bg-current text-neutral-950 dark:text-white";

/// The custom-sanitize demo's module class names
/// (`custom-sanitize/css-modules/index.module.css`, carried verbatim as literal
/// class strings — module docs, "CSS-modules variants").
const SANITIZE_FIELD_CLASS: &str = "Field";
/// See [`SANITIZE_FIELD_CLASS`].
const SANITIZE_LABEL_CLASS: &str = "Label";
/// See [`SANITIZE_FIELD_CLASS`].
const SANITIZE_ROOT_CLASS: &str = "Root";
/// See [`SANITIZE_FIELD_CLASS`].
const SANITIZE_INPUT_CLASS: &str = "Input";
/// The first of the two alternating pulse classes the demo swaps to retrigger
/// its highlight animation (`useInvalidFeedback.ts:22-30` — `getInvalidClassName`
/// returns the even class for an even pulse and the odd class otherwise;
/// `index.tsx:44-48` passes `styles.InputInvalidB` as the even one).
const SANITIZE_PULSE_EVEN_CLASS: &str = "InputInvalidB";
/// See [`SANITIZE_PULSE_EVEN_CLASS`].
const SANITIZE_PULSE_ODD_CLASS: &str = "InputInvalidA";
/// See [`SANITIZE_FIELD_CLASS`].
const SANITIZE_DESCRIPTION_CLASS: &str = "Description";
/// See [`SANITIZE_FIELD_CLASS`].
const SANITIZE_SR_ONLY_CLASS: &str = "ScreenReaderOnly";

/// The `## Anatomy` snippet (`page.mdx:22-29`) — translated to the port's namespaced surface
/// (`leptos_ui::OTPField`): the same tree with Rust's path separator
/// (`specs/docs-content/CONTRACT.md`, requirement 1's React→Rust mapping table), the way
/// `docs-content: components/checkbox` and `docs-content: components/button` were translated.
/// One spelling differs from upstream's listing for the port's own reason: `OTPField::Root` takes
/// `length` as a required prop (`otp_field.rs:1849-1851`; `specs/library/otp-field/behavior.md`
/// § Public API surface — "`length` takes the slot count"), so the port shows it where upstream's
/// listing shorthand shows no props at all. The snippet was `Lang::Jsx` carrying upstream's import
/// line verbatim until this item; the guard below now pins the language.
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::OTPField;

view! {
    <OTPField::Root length=6>
        <OTPField::Input />
        <OTPField::Separator />
    </OTPField::Root>
}"#;

/// The "Labeling an OTP field" snippet (`page.mdx:41-54`) — the native label, six slots and the
/// supporting text, expressed against the port: `id`, `length` and `aria_describedby` are
/// `OTPField::Root` props (`otp_field.rs:1900-1916`) and the per-slot announcement rides
/// `OTPField::Input`'s `aria_label` (`otp_field.rs:1988-1993`). The port's string props take
/// `String`, hence the `.to_string()` the examples carry (recorded as a contract gap: upstream
/// writes the bare literal).
const LABELING_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::OTPField;

view! {
    <div>
        <label for="verification-code">"Verification code"</label>
        <OTPField::Root
            id="verification-code".to_string()
            length=6
            aria_describedby="verification-code-description".to_string()
        >
            <OTPField::Input />
            <OTPField::Input aria_label="Character 2 of 6".to_string() />
            <OTPField::Input aria_label="Character 3 of 6".to_string() />
            <OTPField::Input aria_label="Character 4 of 6".to_string() />
            <OTPField::Input aria_label="Character 5 of 6".to_string() />
            <OTPField::Input aria_label="Character 6 of 6".to_string() />
        </OTPField::Root>
        <p id="verification-code-description">
            "Enter the 6-character code we sent to your device."
        </p>
    </div>
}"#;

/// The "Using OTP Field in a form" snippet (`page.mdx:60-75`), including its `{2}` line-highlight
/// marker (upstream highlights the `<Field.Root name="verificationCode">` line; the port keeps
/// that element at the same position in the tree). Translated to the port's namespaced surface:
/// `Form` wraps `Field::Root`, and the slots nest inside `OTPField::Root`
/// (`specs/docs-content/CONTRACT.md` requirement 1; the composition the `docs-content:
/// components/checkbox` form row and the checkbox-group form snippet already teach).
const FORM_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{Field, Form, OTPField};

view! {
    <Form>
        <Field::Root name="verificationCode".to_string()>
            <Field::Label>"Verification code"</Field::Label>
            <Field::Description>"Enter the 6-character code we sent to your device."</Field::Description>
            <OTPField::Root length=6>
                <OTPField::Input />
                <OTPField::Input aria_label="Character 2 of 6".to_string() />
                <OTPField::Input aria_label="Character 3 of 6".to_string() />
                <OTPField::Input aria_label="Character 4 of 6".to_string() />
                <OTPField::Input aria_label="Character 5 of 6".to_string() />
                <OTPField::Input aria_label="Character 6 of 6".to_string() />
            </OTPField::Root>
        </Field::Root>
    </Form>
}"#;

/// The demos' `React.useId()` analog — the internals crate's Base UI id, i.e.
/// `use_base_ui_id` over an empty override (the checkbox-group page's
/// convention). The override signal is the rg-0.2 runtime's own
/// (`use_base_ui_id`'s signature), not the prelude's re-export, hence the
/// fully-qualified path and the scoped trait import.
fn demo_id() -> String {
    use reactive_graph::traits::GetUntracked as RgGetUntracked;
    use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>)).get_untracked()
}

/// A `className`-only render-element bag (`ClassNameSource::Static` — the
/// docs-page convention, e.g. `separator_page.rs:96-99`).
fn class_props(class: &str) -> UseRenderElementComponentProps {
    UseRenderElementComponentProps {
        class_name: Some(ClassNameSource::Static(class.to_string())),
        render: None,
        style: None,
    }
}

/// Adopts a materialized element's listener/ref teardown on the current owner —
/// the [`RawElementView::new`] convention, so a mounted demo's cleanups run with
/// the component that built it rather than leaking for the page's lifetime.
fn adopt_cleanup(cleanup: Option<CleanupFn>) {
    if let Some(cleanup) = cleanup {
        if let Some(owner) = Owner::current() {
            owner.with(|| {
                // SendWrapper satisfies on_cleanup's Send+Sync bound on the wasm
                // single thread, the wrapper `RawElementView::new` uses.
                let cleanup = send_wrapper::SendWrapper::new(cleanup);
                on_cleanup(move || cleanup.take()());
            });
        }
    }
}

/// Retains a materialized description's **ref fork** for as long as the element it
/// materialized is mounted — the sibling of [`adopt_cleanup`]: that one adopts the
/// teardown, this one adopts the live ref.
///
/// `RenderedElement::create_element` fires the forked ref with the node and keeps
/// nothing (`use_render_element.rs:571-573`); the fork itself — and every attach-time
/// cleanup its branches returned — lives in the description's `props.ref_callback`.
/// For the Input that is the whole behaviour seam of this pair: `onChange`/`onPaste`
/// have no engine handler slot, so the port attaches them **inside the ref callback**
/// (`otp_field.rs:1329-1476`), and the `EventListenerUnsubscribe` handles that hold
/// those listeners live inside the fork's pending-cleanup slot.
/// `EventListenerUnsubscribe::drop` *removes* the listener
/// (`add_event_listener.rs:92-96`), so a page that lets the description drop while the
/// node stays in the DOM silently unregisters the write path — the field then keeps
/// the browser's own text and never commits (`set_value`/`onValueChange` never run).
///
/// Registering the description on the current owner ties the fork's lifetime to the
/// page's, the lifetime `adopt_cleanup` already gives this subtree's handler
/// cleanups; a call with no owner in scope leaks the description deliberately rather
/// than detaching a ref whose element is still mounted.
fn retain_ref_fork(rendered: RenderedElement) {
    if let Some(owner) = Owner::current() {
        owner.with(|| {
            let rendered = send_wrapper::SendWrapper::new(rendered);
            on_cleanup(move || drop(rendered.take()));
        });
    } else {
        std::mem::forget(rendered);
    }
}

/// Creates an element with its `class` set.
fn element_with_class(tag: &str, class: &str) -> Element {
    let element = document().create_element(tag).expect("create element");
    element.set_attribute("class", class).expect("set class");
    element
}

/// The demos' native `<label htmlFor={id}>` (`hero/tailwind/index.tsx:10-12`).
fn label_for(for_id: &str, text: &str) -> Element {
    let label = element_with_class("label", LABEL_CLASS);
    label.set_attribute("for", for_id).expect("set htmlFor");
    label.set_text_content(Some(text));
    label
}

/// The demos' supporting `<p id={descriptionId}>` (`hero/tailwind/index.tsx:26-28`).
fn description_with_id(id: &str, text: &str) -> Element {
    let paragraph = element_with_class("p", DESCRIPTION_CLASS);
    paragraph
        .set_attribute("id", id)
        .expect("set description id");
    paragraph.set_text_content(Some(text));
    paragraph
}

/// Upstream's per-slot `aria-label={index === 0 ? undefined : ...}`
/// (`hero/tailwind/index.tsx:23`): the first slot relies on the shared label
/// (`page.mdx:35-37`), the rest announce their position.
fn character_label(index: usize, length: usize) -> Option<String> {
    (index > 0).then(|| format!("Character {} of {}", index + 1, length))
}

/// The pair's composition root (module docs, "The pair's composition root"):
/// run the real Root, provide the slot registry, materialize the root node, and
/// hand it to `build_slots` so every slot is created INSIDE the root — the
/// nesting upstream's Provider wraps.
/// The pair's composition root (module docs, "The pair's composition root"):
/// run the real Root, provide the slot registry, materialize the root node, and
/// hand it to `build_slots` so every slot is created INSIDE the root — the
/// nesting upstream's Provider wraps. `pub(crate)` so the docs-app wasm suite can
/// exercise the exact composition the page mounts (`render_test.rs`).
pub(crate) fn otp_root(
    root_props: OtpFieldRootProps,
    build_slots: impl FnOnce(&Element),
) -> Element {
    // 1. The whole Root body runs here: contexts provided, write gate, commit
    //    queue, state record, and the element description.
    let rendered = use_otp_field_root(root_props).expect("OTPField.Root always renders");
    // 2. The registry the slots register into — provided BEFORE they construct
    //    (`otp_field.rs:1536-1543`), since their index is claimed at hook-call
    //    time. The provide and the slots' `use_composite_list_item` reads are
    //    both rg-0.2 OWNER-SCOPED context operations, while a docs-app page
    //    renders under leptos's own (0.1.x) owner — so the pair needs an rg-0.2
    //    window, exactly the `checkbox_group_view` bridge (`crates/leptos-ui/
    //    src/checkbox_group/view.rs:137-138,215-216`). Without it the slots read
    //    `use_composite_list_context`'s provider-less default
    //    (`composite_list.rs:790-799`) and claim their index from the SHARED
    //    thread-local counter — every slot then has the wrong index, so its id,
    //    its `value` slice and its tabindex all point at the wrong place.
    let bridge_owner = reactive_graph::owner::Owner::new();
    let (root_node, rendered) = bridge_owner.with(move || {
        // 2. The slot registry — the ROOT's own list, provided inside this window
        //    because that is where the slots construct (`provide_otp_composite_list`
        //    hands over the root's `inputRefs`, module docs step 2).
        provide_otp_composite_list();
        // 3. The root node, then its slots.
        let (root_node, cleanup) = rendered.create_element();
        build_slots(&root_node);
        adopt_cleanup(cleanup);
        (root_node, rendered)
    });
    // The bridge owner (and with it the provided registry) outlives the subtree
    // — the `checkbox_group_view` precedent.
    std::mem::forget(bridge_owner);
    // The root's own fork is retained on the CALLER's owner rather than the
    // deliberately-forgotten bridge, so the page's rule is one rule: every
    // description this pair materializes is kept alive by [`retain_ref_fork`].
    retain_ref_fork(rendered);
    root_node
}

/// One real `OTPField.Input` slot, appended inside its root. `element_attributes`
/// carries the consumer's `...elementProps` rest — the port's path for a static
/// attribute like the placeholder demo's `placeholder`. `decorate` receives the
/// slot's index and its materialized node, for the one demo that owns per-slot
/// consumer wiring.
pub(crate) fn append_slot(
    parent: &Element,
    index: usize,
    class: &str,
    aria_label: Option<String>,
    element_attributes: Vec<(String, String)>,
    decorate: Option<&dyn Fn(usize, &Element)>,
) {
    let rendered = use_otp_field_input(OtpFieldInputProps {
        aria_label,
        component_props: class_props(class),
        element_attributes,
        ..OtpFieldInputProps::default()
    })
    .expect("OTPField.Input renders inside a Root");
    let (node, cleanup) = rendered.create_element();
    parent.append_child(&node).expect("append slot");
    adopt_cleanup(cleanup);
    // The Input's `input`/`paste` write path IS its ref callback — the fork must
    // outlive this call, see [`retain_ref_fork`].
    retain_ref_fork(rendered);
    if let Some(decorate) = decorate {
        decorate(index, &node);
    }
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, `specs/docs-content/otp-field/demos.json`
/// entry 5) — the canonical minimal field: a native label, six slots, and the
/// supporting description, all on the real `OTPField.Root`/`OTPField.Input`
/// parts. Uncontrolled (`demos.json`: "static markup; only React.useId for the
/// label/description ids").
pub fn otp_field_hero_demo() -> RawElementView {
    let id = demo_id();
    let description_id = format!("{id}-description");

    let container = element_with_class("div", FIELD_CLASS);
    container
        .append_child(&label_for(&id, "Verification code"))
        .expect("append label");

    let root = otp_root(
        OtpFieldRootProps {
            length: OTP_LENGTH,
            id: Some(id),
            aria_describedby: Some(description_id.clone()),
            component_props: class_props(ROOT_CLASS),
            ..OtpFieldRootProps::default()
        },
        |root_node| {
            for index in 0..OTP_LENGTH {
                append_slot(
                    root_node,
                    index,
                    INPUT_CLASS,
                    character_label(index, OTP_LENGTH),
                    Vec::new(),
                    None,
                );
            }
        },
    );
    container.append_child(&root).expect("append root");
    container
        .append_child(&description_with_id(
            &description_id,
            "Enter the 6-character code we sent to your device.",
        ))
        .expect("append description");

    RawElementView { element: container }
}

/// The alphanumeric demo (`demos/alphanumeric/tailwind/index.tsx`, demos.json
/// entry 1) — `validationType="alphanumeric"`, which the port filters to
/// `[a-zA-Z0-9]` (`behavior.md` "State model"; `OtpValidationType::Alphanumeric`).
/// The description carries the upstream `<code>A7C9XZ</code>` example.
pub fn otp_field_alphanumeric_demo() -> RawElementView {
    let id = demo_id();
    let description_id = format!("{id}-description");

    let container = element_with_class("div", FIELD_CLASS);
    container
        .append_child(&label_for(&id, "Recovery code"))
        .expect("append label");

    let root = otp_root(
        OtpFieldRootProps {
            length: OTP_LENGTH,
            id: Some(id),
            validation_type: OtpValidationType::Alphanumeric,
            aria_describedby: Some(description_id.clone()),
            component_props: class_props(ROOT_CLASS),
            ..OtpFieldRootProps::default()
        },
        |root_node| {
            for index in 0..OTP_LENGTH {
                append_slot(
                    root_node,
                    index,
                    INPUT_CLASS,
                    character_label(index, OTP_LENGTH),
                    Vec::new(),
                    None,
                );
            }
        },
    );
    container.append_child(&root).expect("append root");

    let paragraph = description_with_id(
        &description_id,
        "Accept letters and numbers for backup codes such as ",
    );
    let code = document().create_element("code").expect("create code");
    code.set_attribute("class", "font-mono")
        .expect("set code class");
    code.set_text_content(Some("A7C9XZ"));
    paragraph.append_child(&code).expect("append code");
    paragraph.append_with_str_1(".").expect("append period");
    container
        .append_child(&paragraph)
        .expect("append description");

    RawElementView { element: container }
}

/// The grouped demo (`demos/grouped/tailwind/index.tsx`, demos.json entry 4) —
/// two layout `<div>`s of three slots each around the real Separator, proving
/// "arbitrary wrapper elements between slots do not affect slot counting"
/// (`behavior.md` "DOM structure & portal behavior"). The first slot still
/// relies on the shared label; the second group's slots carry their own
/// announced positions (`:33`).
pub fn otp_field_grouped_demo() -> RawElementView {
    let id = demo_id();

    let container = element_with_class("div", FIELD_CLASS);
    container
        .append_child(&label_for(&id, "Verification code"))
        .expect("append label");

    let root = otp_root(
        OtpFieldRootProps {
            length: OTP_LENGTH,
            id: Some(id),
            component_props: class_props(GROUPED_ROOT_CLASS),
            ..OtpFieldRootProps::default()
        },
        |root_node| {
            // The first group (`:12-22`): three slots, the first slot unnamed.
            let first_group = element_with_class("div", GROUP_CLASS);
            for index in 0..3 {
                append_slot(
                    &first_group,
                    index,
                    INPUT_CLASS,
                    character_label(index, OTP_LENGTH),
                    Vec::new(),
                    None,
                );
            }
            root_node
                .append_child(&first_group)
                .expect("append first group");

            // The divider (`:25`) — the crate's shared Separator, which is what
            // `OTPField.Separator` re-exports.
            let separator = separator_element(SeparatorProps {
                render_class_style: class_props(SEPARATOR_CLASS),
                ..SeparatorProps::default()
            })
            .expect("standalone Separator renders (no enabled gate)");
            let (separator_node, separator_cleanup) = separator.create_element();
            root_node
                .append_child(&separator_node)
                .expect("append separator");
            adopt_cleanup(separator_cleanup);

            // The second group (`:26-37`): three more slots, each announced as
            // "Character 4|5|6 of 6".
            let second_group = element_with_class("div", GROUP_CLASS);
            for index in 3..OTP_LENGTH {
                append_slot(
                    &second_group,
                    index,
                    INPUT_CLASS,
                    Some(format!("Character {} of {}", index + 1, OTP_LENGTH)),
                    Vec::new(),
                    None,
                );
            }
            root_node
                .append_child(&second_group)
                .expect("append second group");
        },
    );
    container.append_child(&root).expect("append root");

    RawElementView { element: container }
}

/// The placeholder demo (`demos/focused-placeholder/tailwind/index.tsx`,
/// demos.json entry 3) — the native `placeholder` prop reaches the real input
/// through the port's `...elementProps` rest, and the demo's own CSS hides the
/// hint on the focused slot (`page.mdx:100-101`).
pub fn otp_field_focused_placeholder_demo() -> RawElementView {
    let id = demo_id();
    let description_id = format!("{id}-description");

    let container = element_with_class("div", FIELD_CLASS);
    container
        .append_child(&label_for(&id, "Verification code"))
        .expect("append label");

    let root = otp_root(
        OtpFieldRootProps {
            length: OTP_LENGTH,
            id: Some(id),
            aria_describedby: Some(description_id.clone()),
            component_props: class_props(ROOT_CLASS),
            ..OtpFieldRootProps::default()
        },
        |root_node| {
            for index in 0..OTP_LENGTH {
                append_slot(
                    root_node,
                    index,
                    PLACEHOLDER_INPUT_CLASS,
                    character_label(index, OTP_LENGTH),
                    vec![("placeholder".to_string(), PLACEHOLDER.to_string())],
                    None,
                );
            }
        },
    );
    container.append_child(&root).expect("append root");
    container
        .append_child(&description_with_id(
            &description_id,
            "Placeholder hints can stay visible until the active slot is focused.",
        ))
        .expect("append description");

    RawElementView { element: container }
}

/// The masked-entry demo (`demos/password/tailwind/index.tsx`, demos.json entry
/// 6) — `mask` renders every slot as `input[type="password"]` through the port's
/// own attribute bag (`behavior.md` "DOM structure & portal behavior", types.md
/// `mask`).
pub fn otp_field_password_demo() -> RawElementView {
    let id = demo_id();
    let description_id = format!("{id}-description");

    let container = element_with_class("div", FIELD_CLASS);
    container
        .append_child(&label_for(&id, "Access code"))
        .expect("append label");

    let root = otp_root(
        OtpFieldRootProps {
            length: OTP_LENGTH,
            id: Some(id),
            mask: true,
            aria_describedby: Some(description_id.clone()),
            component_props: class_props(ROOT_CLASS),
            ..OtpFieldRootProps::default()
        },
        |root_node| {
            for index in 0..OTP_LENGTH {
                append_slot(
                    root_node,
                    index,
                    INPUT_CLASS,
                    character_label(index, OTP_LENGTH),
                    Vec::new(),
                    None,
                );
            }
        },
    );
    container.append_child(&root).expect("append root");

    let paragraph = description_with_id(&description_id, "Use ");
    let code = document().create_element("code").expect("create code");
    code.set_attribute("class", "font-mono")
        .expect("set code class");
    code.set_text_content(Some("mask"));
    paragraph.append_child(&code).expect("append code");
    paragraph
        .append_with_str_1(" to obscure the code on shared screens.")
        .expect("append tail");
    container
        .append_child(&paragraph)
        .expect("append description");

    RawElementView { element: container }
}

/// The custom-normalization demo
/// (`demos/custom-sanitize/css-modules/index.tsx`, demos.json entry 2 — the
/// demo's only variant) — `normalizeValue` uppercases accepted characters, and
/// rejected ones are reported through `onValueInvalid`, which the demo's own
/// `useInvalidFeedback` hook turns into an `aria-live` status message plus an
/// alternating highlight class on the focused slot (`useInvalidFeedback.ts:38-51`).
///
/// The hook's state is a `useState` triple upstream; here it is three signals,
/// and the two consumer callbacks are the port's `on_value_change`/
/// `on_value_invalid` slots. The per-slot `onFocus` and the state-derived
/// `className` ride the materialized node directly (module docs, "Documented
/// adaptations").
pub fn otp_field_custom_sanitize_demo() -> RawElementView {
    let id = demo_id();
    let description_id = format!("{id}-description");

    // `useInvalidFeedback`'s state (`useInvalidFeedback.ts:5-9`).
    let focused_index = RwSignal::new(0usize);
    let invalid_pulse = RwSignal::new(0u32);
    let status_message = RwSignal::new(String::new());
    // The hook's two refs: the "don't clear on the next change" flag and the
    // pending 400ms reset. The reset is generation-guarded instead of handle-
    // cleared — a stale timer cannot clear a newer pulse.
    let pulse_generation = Rc::new(Cell::new(0u32));
    let skip_clear_on_next_value_change = Rc::new(Cell::new(false));

    // `clearInvalidFeedback` (`:21-31`).
    let clear_invalid_feedback = {
        let pulse_generation = Rc::clone(&pulse_generation);
        move || {
            pulse_generation.set(pulse_generation.get().wrapping_add(1));
            invalid_pulse.set(0);
            status_message.set(String::new());
        }
    };

    // The materialized slot nodes, so the committed value can be reflected onto
    // them (see the mirror note in `handleValueChange`).
    let slots: Rc<RefCell<Vec<Element>>> = Rc::new(RefCell::new(Vec::new()));

    // `handleValueChange` (`:33-40`).
    let on_value_change = {
        let skip_clear = Rc::clone(&skip_clear_on_next_value_change);
        let clear = clear_invalid_feedback.clone();
        let slots = Rc::clone(&slots);
        Rc::new(move |value: &str, _details: &OtpChangeEventDetails| {
            // The port's slots materialize their state ONCE and are not re-rendered
            // from state afterwards (upstream's Input renders `value={slotValue}`;
            // the port's `RenderedElement::create_element` resolves the attribute bag
            // a single time), so the demo reflects the committed value onto the slot
            // nodes here — the DOM write React's controlled input performs for
            // itself, and the only way `normalizeValue`'s uppercasing becomes
            // visible. The port gap is recorded in `ralph/logs/spec-discrepancies.md`.
            for (index, slot) in slots.borrow().iter().enumerate() {
                let next = value
                    .chars()
                    .nth(index)
                    .map(String::from)
                    .unwrap_or_default();
                if let Ok(input) = slot.clone().dyn_into::<leptos::web_sys::HtmlInputElement>() {
                    if input.value() != next {
                        input.set_value(&next);
                    }
                }
            }

            if skip_clear.get() {
                skip_clear.set(false);
                return;
            }
            clear();
        }) as leptos_ui::OtpChangeHandler
    };

    // `handleValueInvalid` (`:42-60`).
    let on_value_invalid = {
        let skip_clear = Rc::clone(&skip_clear_on_next_value_change);
        let pulse_generation = Rc::clone(&pulse_generation);
        Rc::new(move |value: &str, _details: &OtpGenericEventDetails| {
            skip_clear.set(true);
            invalid_pulse.update(|pulse| *pulse += 1);
            status_message.set(format!("Unsupported characters were ignored from {value}."));

            let generation = pulse_generation.get().wrapping_add(1);
            pulse_generation.set(generation);
            let generation_source = Rc::clone(&pulse_generation);
            set_timeout(
                move || {
                    if generation_source.get() == generation {
                        invalid_pulse.set(0);
                    }
                },
                Duration::from_millis(400),
            );
        }) as leptos_ui::OtpGenericHandler
    };

    let normalize: NormalizeValueFn = Rc::new(|value: &str| value.to_uppercase());

    let container = element_with_class("div", SANITIZE_FIELD_CLASS);
    container
        .append_child(&label_for(&id, "Recovery code"))
        .expect("append label");

    let decorate = {
        let decorate_slots = Rc::clone(&slots);
        move |slot_index: usize, node: &Element| {
            // The consumer's `onFocus` (`index.tsx:63-65`), wired onto the slot node.
            decorate_slots.borrow_mut().push(node.clone());
            let focus_node = node.clone();
            let focus_listener = leptos::wasm_bindgen::closure::Closure::<dyn Fn(Event)>::new(
                move |_event: Event| {
                    focused_index.set(slot_index);
                },
            );
            focus_node
                .add_event_listener_with_callback("focus", focus_listener.as_ref().unchecked_ref())
                .expect("attach focus listener");
            focus_listener.forget();

            // The state-derived `className` (`index.tsx:44-48`, `:57`): the base
            // input class plus — while the pulse is live and this is the focused
            // slot — the alternating highlight class.
            let class_node = node.clone();
            Effect::new(move |_| {
                let pulse = invalid_pulse.get();
                let active = pulse > 0 && focused_index.get() == slot_index;
                let class = if active {
                    let pulse_class = if pulse % 2 == 0 {
                        SANITIZE_PULSE_EVEN_CLASS
                    } else {
                        SANITIZE_PULSE_ODD_CLASS
                    };
                    format!("{SANITIZE_INPUT_CLASS} {pulse_class}")
                } else {
                    SANITIZE_INPUT_CLASS.to_string()
                };
                let _ = class_node.set_attribute("class", &class);
            });
        }
    };

    let root = otp_root(
        OtpFieldRootProps {
            length: OTP_LENGTH,
            id: Some(id),
            validation_type: OtpValidationType::Alphanumeric,
            normalize_value: Some(normalize),
            on_value_change: Some(on_value_change),
            on_value_invalid: Some(on_value_invalid),
            aria_describedby: Some(description_id.clone()),
            component_props: class_props(SANITIZE_ROOT_CLASS),
            ..OtpFieldRootProps::default()
        },
        |root_node| {
            for index in 0..OTP_LENGTH {
                append_slot(
                    root_node,
                    index,
                    SANITIZE_INPUT_CLASS,
                    character_label(index, OTP_LENGTH),
                    Vec::new(),
                    Some(&decorate as &dyn Fn(usize, &Element)),
                );
            }
        },
    );
    container.append_child(&root).expect("append root");

    container
        .append_child(&description_with_id(
            &description_id,
            "Letters and digits only. Letters are converted to uppercase.",
        ))
        .expect("append description");

    // The screen-reader status line (`:70-72`).
    let status = element_with_class("span", SANITIZE_SR_ONLY_CLASS);
    status
        .set_attribute("aria-live", "polite")
        .expect("set aria-live");
    let status_for_effect = status.clone();
    Effect::new(move |_| {
        status_for_effect.set_text_content(Some(&status_message.get()));
    });
    container.append_child(&status).expect("append status span");

    RawElementView { element: container }
}

/// One API-reference block: a generated `TypesOTPField` table
/// (`docs/src/app/(docs)/react/components/otp-field/types.md`) echoed as static
/// prose — the summary line, the props list and the data-attributes list.
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
    }
}

/// The `docs/src/app/(docs)/react/components/otp-field/page.mdx` page.
#[component]
pub fn OtpFieldPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"OTP Field"</h1>
            <p class="subtitle">"A one-time password input composed of individual character slots."</p>

            <div class="docs-demo" data-demo="hero">{otp_field_hero_demo()}</div>

            <h2>"Usage guidelines"</h2>
            <ul>
                <li>
                    <strong>"Form controls must have an accessible name"</strong>
                    ": It can be created using a `<label>` element or the `Field` component. See "
                    <a href="#labeling-an-otp-field">"Labeling an OTP field"</a>
                    " and the "
                    <a href="/react/handbook/forms">"forms guide"</a>
                    "."
                </li>
            </ul>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Examples"</h2>

            <h3>"Labeling an OTP field"</h3>
            <p>
                "Pass an `id` to `<OTPField.Root>` and use a native `<label>` with a matching "
                "`htmlFor`. Let the first input use the field label, and add `aria-label` to the "
                "remaining inputs so assistive technology can announce which slot is focused."
            </p>
            <p>
                "Optionally, add `aria-describedby` when supporting text should be announced with "
                "the field."
            </p>
            {code_block(Lang::Rust, "OTP Field with a native label and description", LABELING_SNIPPET)}

            <h3>"Form integration"</h3>
            <p>
                "Use "
                <a href="/react/components/field">"Field"</a>
                " to handle label associations and form integration:"
            </p>
            {code_block(Lang::Rust, "Using OTP Field in a form", FORM_SNIPPET)}
            <p>
                "Pass `autoSubmit` to submit the owning form automatically when all slots are "
                "filled, or use `onValueComplete` to react to completion without submitting."
            </p>

            <h3>"Alphanumeric verification codes"</h3>
            <p>
                "Use `validationType=\"alphanumeric\"` for recovery, backup, or invite codes that "
                "mix letters and numbers."
            </p>
            <div class="docs-demo" data-demo="alphanumeric">{otp_field_alphanumeric_demo()}</div>

            <h3>"Grouped layouts"</h3>
            <p>
                "Wrap subsets of inputs in your own layout elements and use `<OTPField.Separator>` "
                "when you want the code presented in smaller visual chunks such as `123-456`."
            </p>
            <div class="docs-demo" data-demo="grouped">{otp_field_grouped_demo()}</div>

            <h3>"Placeholder hints"</h3>
            <p>
                "`<OTPField.Input>` is a real input, so native `placeholder` props and CSS work as "
                "usual. This example keeps placeholder hints visible until the active slot receives "
                "focus."
            </p>
            <div class="docs-demo" data-demo="focused-placeholder">
                {otp_field_focused_placeholder_demo()}
            </div>

            <h3>"Custom normalization"</h3>
            <p>
                "Use `normalizeValue` to normalize accepted values before state updates, such as "
                "converting alphanumeric codes to uppercase. It runs after `validationType` "
                "filtering, and the result is filtered against `validationType` again. Use "
                "`validationType=\"none\"` when the normalizer should provide the full validation "
                "rule."
            </p>
            <p>
                "Pair custom rules with `inputMode` for keyboard hints and `onValueInvalid` for "
                "rejected characters."
            </p>
            <div class="docs-demo" data-demo="custom-sanitize">
                {otp_field_custom_sanitize_demo()}
            </div>

            <h3>"Masked entry"</h3>
            <p>"Use `mask` when the code should be obscured while it is being typed."</p>
            <div class="docs-demo" data-demo="password">{otp_field_password_demo()}</div>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                "Groups all OTP field parts and manages their state. Renders a <div> element.",
                "Props: name (string — identifies the field when a form is submitted), defaultValue (string — the uncontrolled OTP value when the component is initially rendered), value (string — the OTP value), onValueChange ((value: string, eventDetails: OTPField.Root.ChangeEventDetails) => void — fired when the OTP value changes; eventDetails.reason is 'input-change', 'input-clear', 'input-paste' or 'keyboard'), autoComplete (string, 'one-time-code' — the input autocomplete attribute, applied to the first slot and hidden validation input), autoSubmit (boolean, false — whether to submit the owning form when the OTP becomes complete), form (string — the id of the form element the hidden input is associated with), inputMode ('none' | 'text' | 'tel' | 'url' | 'email' | 'numeric' | 'decimal' | 'search' — the virtual keyboard hint applied to the slot inputs), length (number, required — the number of OTP input slots; required so the root can clamp values, detect completion and generate consistent validation markup before all slots hydrate), mask (boolean, false — whether the slot inputs should mask entered characters; pass type directly to individual Input parts for a custom input type), normalizeValue ((value: string) => string — normalizes the OTP value after whitespace and validationType filtering; the returned value is filtered by validationType again, then clamped to length; characters rejected while normalizing typed or pasted text are reported through onValueInvalid), onValueComplete ((value: string, eventDetails: OTPField.Root.CompleteEventDetails) => void — fired when the OTP value becomes complete, or when a complete value is pasted while already complete), onValueInvalid ((value: string, eventDetails: OTPField.Root.InvalidEventDetails) => void — fired when entered text contains characters rejected before the OTP value updates; the value argument is the attempted string before normalization), validationType (OTPField.Root.ValidationType, 'numeric' — the type of input validation applied to the OTP value), disabled (boolean, false), readOnly (boolean, false), required (boolean, false), id (string — the id of the first input element; subsequent inputs derive their ids from it, {id}-2, {id}-3, and so on), className, style, render.",
                "Data attributes: data-disabled, data-readonly, data-required, data-valid (present when the OTP field is in a valid state — when wrapped in Field.Root), data-invalid (present when the OTP field is in an invalid state — when wrapped in Field.Root), data-dirty (present when the OTP field's value has changed — when wrapped in Field.Root), data-touched (present when the OTP field has been touched — when wrapped in Field.Root), data-complete (present when all slots are filled), data-filled (present when the OTP field contains at least one character), data-focused (present when one of the OTP field inputs is focused).",
            )}
            <h3>"Root.Props"</h3>
            <p>"Re-export of Root props."</p>
            <h3>"Root.State"</h3>
            <p>
                "type OTPFieldRootState = { complete: boolean; disabled: boolean; length: number; "
                "readOnly: boolean; required: boolean; value: string; touched: boolean; dirty: "
                "boolean; valid: boolean | null; filled: boolean; focused: boolean };"
            </p>
            <h3>"Root.ValidationType"</h3>
            <p>"type OTPFieldRootValidationType = 'numeric' | 'alpha' | 'alphanumeric' | 'none';"</p>
            <h3>"Root.ChangeEventReason"</h3>
            <p>
                "type OTPFieldRootChangeEventReason = 'input-change' | 'input-clear' | "
                "'input-paste' | 'keyboard';"
            </p>
            <h3>"Root.ChangeEventDetails"</h3>
            <p>
                "type OTPFieldRootChangeEventDetails = ({ reason: 'input-change'; event: InputEvent "
                "| Event } | { reason: 'input-clear'; event: InputEvent | Event | FocusEvent } | "
                "{ reason: 'input-paste'; event: ClipboardEvent } | { reason: 'keyboard'; event: "
                "KeyboardEvent }) & { cancel: () => void; allowPropagation: () => void; "
                "isCanceled: boolean; isPropagationAllowed: boolean; trigger: Element | "
                "undefined };"
            </p>
            <h3>"Root.InvalidEventReason"</h3>
            <p>"type OTPFieldRootInvalidEventReason = 'input-change' | 'input-paste';"</p>
            <h3>"Root.InvalidEventDetails"</h3>
            <p>
                "type OTPFieldRootInvalidEventDetails = { reason: 'input-change'; event: InputEvent "
                "| Event } | { reason: 'input-paste'; event: ClipboardEvent };"
            </p>
            <h3>"Root.CompleteEventReason"</h3>
            <p>"type OTPFieldRootCompleteEventReason = 'input-change' | 'input-paste';"</p>
            <h3>"Root.CompleteEventDetails"</h3>
            <p>
                "type OTPFieldRootCompleteEventDetails = { reason: 'input-change'; event: "
                "InputEvent | Event } | { reason: 'input-paste'; event: ClipboardEvent };"
            </p>
            <h3>"Input"</h3>
            {api_part(
                "An individual OTP character input. Renders an <input> element.",
                "Props: className, style, render — the standard render-element vocabulary; the native input attributes (aria-label, type, placeholder, inputMode and the rest) forward through to the element.",
                "Data attributes: data-disabled, data-readonly, data-required, data-valid (when wrapped in Field.Root), data-invalid (when wrapped in Field.Root), data-dirty (when wrapped in Field.Root), data-touched (when wrapped in Field.Root), data-complete (present when all slots are filled), data-filled (present when the input contains a character), data-focused (present when any OTP field input is focused).",
            )}
            <h3>"Input.Props"</h3>
            <p>"Re-export of Input props."</p>
            <h3>"Input.State"</h3>
            <p>
                "type OTPFieldInputState = { filled: boolean; index: number; value: string; "
                "disabled: boolean; length: number; required: boolean; readOnly: boolean; "
                "complete: boolean; touched: boolean; dirty: boolean; valid: boolean | null; "
                "focused: boolean };"
            </p>
            <h3>"Separator"</h3>
            {api_part(
                "A separator element accessible to screen readers. Renders a <div> element.",
                "Props: orientation (Orientation, 'horizontal' — the orientation of the separator), className, style, render.",
                "Data attributes: data-orientation rides the shared Separator's default state walk.",
            )}
            <h3>"Separator.Props"</h3>
            <p>"Re-export of Separator props."</p>
            <h3>"Separator.State"</h3>
            <p>"type OTPFieldSeparatorState = { orientation: Orientation };"</p>
            <h3>"Canonical types"</h3>
            <p>
                "OTPField.Root.State: OTPFieldRootState, OTPField.Root.Props: OTPFieldRootProps, "
                "OTPField.Root.ChangeEventReason: OTPFieldRootChangeEventReason, "
                "OTPField.Root.ChangeEventDetails: OTPFieldRootChangeEventDetails, "
                "OTPField.Root.InvalidEventReason: OTPFieldRootInvalidEventReason, "
                "OTPField.Root.InvalidEventDetails: OTPFieldRootInvalidEventDetails, "
                "OTPField.Root.CompleteEventReason: OTPFieldRootCompleteEventReason, "
                "OTPField.Root.CompleteEventDetails: OTPFieldRootCompleteEventDetails, "
                "OTPField.Input.State: OTPFieldInputState, OTPField.Input.Props: "
                "OTPFieldInputProps."
            </p>
        </article>
    }
}

/// Browser-free guard for `specs/docs-content/CONTRACT.md` requirement 1: all three of this page's
/// snippets demonstrate the PORT's API.
///
/// Why it exists: this page sat in the repo marked `done` while all three of its embedded code
/// blocks carried upstream's source — the React library's own import line plus JSX on
/// `## Anatomy`, upstream's TSX for "Labeling an OTP field" and "Using OTP Field in a form"
/// (the package specifier is deliberately not spelled out here: this is page-source, and
/// `check-package-alias.mjs` reads a quoted occurrence as the page telling the reader what to
/// install) — and every structural gate stayed green, because `playwright-diff.mjs` and
/// `check-visual-budget.mjs` watch a page's shape and looks, not which framework it teaches. The
/// snippet-language probe that does (`ralph/scripts/visual-gap-report.mjs:233-242`) needs BOTH dev
/// servers up and reports a NOTE when the upstream reference is down; transcribed snippet text even
/// counted toward content recall, so leaving upstream's source there *raised* the fidelity score.
/// This module is the cheap half of that obligation: it runs in the ordinary host suite
/// (`cargo test -p docs-app --lib`) and fails the moment a snippet teaches React again.
///
/// Deliberately three-part, matching the checkbox/button/field pages' guards:
///   * `the_classifier_recognises_upstream_source` is the positive control — upstream's Anatomy
///     shape WITHOUT its package specifier (this is page-source, not reader-facing, and
///     `check-react-mentions.mjs --source` counts a bare `@base-ui/react/…` string wherever it
///     appears), so `the_pages_snippets_all_teach_the_port` cannot pass vacuously;
///   * `the_pages_snippets_all_teach_the_port` classifies each constant with the same rules as the
///     probe (mirrored in `crate::snippet_language`, shared by every mirrored page), asserting the
///     triple the probe reports: `{total: 3, leptos: 3, react: 0}`;
///   * the `_shape` functions compile the composition each snippet teaches, so a snippet cannot
///     name a prop, part or path the port does not actually have. They are never called (the
///     page's real compositions are exercised by `render_test.rs`); the compiler is the assertion.
#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};
    use leptos_ui::{Field, Form, OTPField};

    /// Upstream's Anatomy block (`page.mdx:22-29`), package specifier removed.
    const UPSTREAM_ANATOMY_SHAPE: &str =
        "<OTPField.Root>\n  <OTPField.Input />\n  <OTPField.Separator />\n</OTPField.Root>;";

    /// Upstream's form-integration block (`page.mdx:60-75`), package specifier removed.
    const UPSTREAM_FORM_SHAPE: &str = "<Form>\n  <Field.Root name=\"verificationCode\">\n    <Field.Label>Verification code</Field.Label>\n    <OTPField.Root length={6}>\n      <OTPField.Input />\n    </OTPField.Root>\n  </Field.Root>\n</Form>";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert_eq!(
            classify(UPSTREAM_ANATOMY_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertion below \
             would be vacuous"
        );
        assert_eq!(
            classify(UPSTREAM_FORM_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertion below \
             would be vacuous"
        );
    }

    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let snippets = [
            ("Anatomy", ANATOMY_SNIPPET),
            ("OTP Field with a native label and description", LABELING_SNIPPET),
            ("Using OTP Field in a form", FORM_SNIPPET),
        ];
        let (mut leptos, mut react) = (0, 0);
        for (name, text) in snippets {
            match classify(text) {
                SnippetLanguage::Leptos => leptos += 1,
                SnippetLanguage::React => {
                    react += 1;
                    panic!("the '{name}' snippet still carries React source");
                }
                SnippetLanguage::Other => {}
            }
        }
        assert_eq!(
            (leptos, react),
            (3, 0),
            "the page's snippets must all teach the port (probe triple: {{total: 3, leptos: 3, \
             react: 0}})"
        );
    }

    /// The `## Anatomy` snippet's composition, verbatim.
    #[allow(dead_code)]
    fn anatomy_snippet_shape() -> impl IntoView {
        view! {
            <OTPField::Root length=6>
                <OTPField::Input />
                <OTPField::Separator />
            </OTPField::Root>
        }
    }

    /// The "Labeling an OTP field" snippet's composition, verbatim.
    #[allow(dead_code)]
    fn labeling_snippet_shape() -> impl IntoView {
        view! {
            <div>
                <label for="verification-code">"Verification code"</label>
                <OTPField::Root
                    id="verification-code".to_string()
                    length=6
                    aria_describedby="verification-code-description".to_string()
                >
                    <OTPField::Input />
                    <OTPField::Input aria_label="Character 2 of 6".to_string() />
                    <OTPField::Input aria_label="Character 3 of 6".to_string() />
                    <OTPField::Input aria_label="Character 4 of 6".to_string() />
                    <OTPField::Input aria_label="Character 5 of 6".to_string() />
                    <OTPField::Input aria_label="Character 6 of 6".to_string() />
                </OTPField::Root>
                <p id="verification-code-description">
                    "Enter the 6-character code we sent to your device."
                </p>
            </div>
        }
    }

    /// The "Using OTP Field in a form" snippet's composition, verbatim.
    #[allow(dead_code)]
    fn form_snippet_shape() -> impl IntoView {
        view! {
            <Form>
                <Field::Root name="verificationCode".to_string()>
                    <Field::Label>"Verification code"</Field::Label>
                    <Field::Description>
                        "Enter the 6-character code we sent to your device."
                    </Field::Description>
                    <OTPField::Root length=6>
                        <OTPField::Input />
                        <OTPField::Input aria_label="Character 2 of 6".to_string() />
                        <OTPField::Input aria_label="Character 3 of 6".to_string() />
                        <OTPField::Input aria_label="Character 4 of 6".to_string() />
                        <OTPField::Input aria_label="Character 5 of 6".to_string() />
                        <OTPField::Input aria_label="Character 6 of 6".to_string() />
                    </OTPField::Root>
                </Field::Root>
            </Form>
        }
    }

    /// Every snippet's composition must compile against the crate's real surface.
    #[test]
    fn every_snippet_compiles_against_the_ports_surface() {
        let _ = (
            anatomy_snippet_shape,
            labeling_snippet_shape,
            form_snippet_shape,
        );
    }
}
