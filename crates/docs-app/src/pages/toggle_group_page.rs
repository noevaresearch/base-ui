//! The docs page for `ToggleGroup`, mirroring
//! `docs/src/app/(docs)/react/components/toggle-group/page.mdx`
//! (`specs/docs-content/toggle-group/page.md`) — the `docs-content: components/toggle-group`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Toggle Group` h1, the
//! `<Subtitle>` one-liner ("Provides a shared state to a series of toggle buttons."), the
//! `<Meta name="description">` copy, the hero demo rendered immediately after the header
//! block, `## Anatomy` with its single fenced snippet, `## Examples` with the `### Multiple`
//! subsection and its demo, and `## API reference` over the generated `TypesToggleGroup`
//! content (echoed as static prose in the PORT's type names, per the input/csp-provider page
//! precedent — the port has no docs generator and the generator's `React.CSSProperties` /
//! `ReactElement` type cells would be defects under
//! `ralph/scripts/check-react-mentions.mjs --source`).
//!
//! THE DEMOS ARE THE REAL PORT. Both mirror the upstream Tailwind variants
//! (`demos/hero/tailwind/index.tsx`, `demos/multiple/tailwind/index.tsx`): a `ToggleGroup`
//! with upstream's `className` strings verbatim, its `defaultValue`, its `aria-label` (which
//! the port carries through `element_attributes` — upstream's `...elementProps` rest), and
//! three `Toggle` children built from the `toggle` unit's own description builder
//! ([`toggle_element`]) so the group's composite root, its value snapshot and each child's
//! `aria-pressed`/`data-pressed` are the real machine's (behavior.md → "State model",
//! "Accessibility").
//!
//! THE CHILDREN ARE REBUILT PER VALUE SNAPSHOT, which is the port's re-render analog: the
//! landed `ToggleGroupContext` hands children a value *snapshot* at body time, and
//! `toggle_group_view` re-runs the subtree closure whenever the group's value changes
//! (`crates/leptos-ui/src/toggle_group.rs`, module docs) — so a click on a child really does
//! re-derive every child's pressed state here, not just the clicked one.
//!
//! GAPS CARRIED OPEN, recorded rather than papered over:
//!
//! - There is no `Toggle` **component** in the port (the toggle unit ships the element
//!   description [`toggle_element`] plus `create_element`), so a grouped child has to be
//!   materialized through a view bridge. The bridge used here is the docs app's own
//!   [`RawElementView`] (the crate-side twin is `leptos_ui::AvatarDocView`, avatar-named).
//!   Upstream's teaching shape is `<ToggleGroup><Toggle /></ToggleGroup>`; the difference is
//!   owned by the ergonomics item `docs-ergonomics: mirrored snippets must read like
//!   upstream's (namespaced components, size parity)` and is listed in the page spec's
//!   contract table as a carried-open gap.
//! - `render` and the forwarded `ref` are not exposed on the `ToggleGroup` component: its
//!   view path builds a fixed `<div>`, so the element form of `render` (which replaces the
//!   tag, `useRenderElement.tsx:164-196`) would be silently dropped — the unit's own
//!   recorded decision (`crates/leptos-ui/src/toggle_group.rs`, module docs), also in the
//!   contract table.
//! - The `Toolbar`-nesting branch is structurally absent (upstream reads the toolbar
//!   contexts purely to OR `disabled`; `library: toolbar` has no ledger item), recorded in
//!   the unit's spec and TODO entry, not dropped here.

use leptos::prelude::*;
use std::rc::Rc;

use leptos_ui::{ToggleGroup, ToggleProps, toggle_element};
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderElementProps, RenderProp, RenderedElement, UseRenderElementComponentProps,
};

use crate::code_block::{Lang, code_block};
use crate::pages::use_render_page::RawElementView;

/// The `## Anatomy` snippet (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:17-21`
/// — upstream's `import { ToggleGroup } from '@base-ui/react/toggle-group'` followed by a bare
/// `<ToggleGroup />`).
///
/// Translated to THIS port (`specs/docs-content/CONTRACT.md` requirement 1 — a snippet on a
/// mirrored page shows the port's own API, never upstream's import line). Two adaptations the
/// port's shape forces, both recorded in the module docs and the spec's contract table: the
/// `aria-label` upstream passes as a prop rides the `element_attributes` rest (that IS
/// upstream's `...elementProps` spread), and the bare render becomes a real composed child
/// because `ToggleGroup` requires `children` (`crates/leptos-ui/src/toggle_group.rs:692`,
/// `children: ChildrenFn`).
const ANATOMY_SNIPPET: &str = r#"use leptos::prelude::*;
use leptos_ui::{ToggleGroup, ToggleProps, toggle_element};
use docs_app::pages::use_render_page::RawElementView;

view! {
    <ToggleGroup
        default_value=vec!["left".to_string()]
        element_attributes=vec![("aria-label".to_string(), "Text alignment".to_string())]
    >
        {move || {
            // A `Toggle` is an element description: build it, then materialize it.
            let rendered = toggle_element(ToggleProps {
                value: Some("left".to_string()),
                ..Default::default()
            })
            .expect("a grouped Toggle renders through the group's composite root");
            let (element, cleanup) = rendered.create_element();
            std::mem::forget(cleanup);
            RawElementView { element }
        }}
    </ToggleGroup>
}"#;

/// The hero demo's panel class (`demos/hero/tailwind/index.tsx:10`).
const PANEL_CLASS: &str = "flex gap-px p-px border border-neutral-950 dark:border-white";

/// The hero demo's button class (`demos/hero/tailwind/index.tsx:15`) — verbatim.
const HERO_BUTTON_CLASS: &str = "flex size-8 items-center justify-center border-none rounded-none bg-transparent text-neutral-950 dark:text-white select-none hover:not-data-disabled:bg-neutral-100 dark:hover:not-data-disabled:bg-neutral-800 active:not-data-disabled:not-data-pressed:bg-neutral-200 dark:active:not-data-disabled:not-data-pressed:bg-neutral-700 data-pressed:bg-neutral-950 data-pressed:text-white dark:data-pressed:bg-white dark:data-pressed:text-neutral-950 data-pressed:hover:not-data-disabled:bg-neutral-950 data-pressed:hover:not-data-disabled:text-white dark:data-pressed:hover:not-data-disabled:bg-white dark:data-pressed:hover:not-data-disabled:text-neutral-950 focus-visible:relative focus-visible:z-1 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white";

/// The multiple demo's button class (`demos/multiple/tailwind/index.tsx:10`) — verbatim.
const MULTIPLE_BUTTON_CLASS: &str = "flex size-8 items-center justify-center border-none rounded-none bg-transparent text-neutral-950 dark:text-white select-none hover:bg-neutral-100 dark:hover:bg-neutral-800 data-pressed:bg-neutral-950 data-pressed:text-white dark:data-pressed:bg-white dark:data-pressed:text-neutral-950 data-pressed:hover:bg-neutral-950 data-pressed:hover:text-white dark:data-pressed:hover:bg-white dark:data-pressed:hover:text-neutral-950 focus-visible:relative focus-visible:z-1 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-neutral-950 dark:focus-visible:outline-white";

/// `AlignLeftIcon` (`demos/hero/tailwind/index.tsx:37-51`) as the RENDERED element — the icon
/// is the button's content, and the port's `ToggleProps` has no children slot, so it travels
/// through the render prop's `inner_html` (the toggle page's heart-icon precedent).
const ALIGN_LEFT_ICON: &str = r#"<svg width="16" height="16" fill="none" viewBox="0 0 16 16" stroke="currentColor" style="display: block"><path stroke-linecap="square" stroke-linejoin="round" d="M2.5 4.5h11m-11 7h9M2.5 8h5"/></svg>"#;

/// `AlignCenterIcon` (`demos/hero/tailwind/index.tsx:53-66`).
const ALIGN_CENTER_ICON: &str = r#"<svg width="16" height="16" fill="none" viewBox="0 0 16 16" stroke="currentColor" style="display: block"><path stroke-linecap="square" stroke-linejoin="round" d="M2.5 4.5h11m-10 7h9M5.5 8h5"/></svg>"#;

/// `AlignRightIcon` (`demos/hero/tailwind/index.tsx:68-81`).
const ALIGN_RIGHT_ICON: &str = r#"<svg width="16" height="16" fill="none" viewBox="0 0 16 16" stroke="currentColor" style="display: block"><path stroke-linecap="square" stroke-linejoin="round" d="M2.5 4.5h11m-9 7h9M8.5 8h5"/></svg>"#;

/// `BoldIcon` (`demos/multiple/tailwind/index.tsx:37-50`).
const BOLD_ICON: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path d="M3.73353 2.13333C3.4386 2.13333 3.2002 2.37226 3.2002 2.66666C3.2002 2.96106 3.4386 3.2 3.73353 3.2H4.26686V12.8H3.73353C3.4386 12.8 3.2002 13.0389 3.2002 13.3333C3.2002 13.6277 3.4386 13.8667 3.73353 13.8667H9.86686C11.7783 13.8667 13.3335 12.3115 13.3335 10.4C13.3335 8.9968 12.4945 7.78881 11.2929 7.24375C11.8897 6.70615 12.2669 5.93066 12.2669 5.06666C12.2669 3.44906 10.9506 2.13333 9.33353 2.13333H3.73353ZM6.93353 3.2H8.26686C9.29619 3.2 10.1335 4.03733 10.1335 5.06666C10.1335 6.096 9.29619 6.93333 8.26686 6.93333H6.93353V3.2ZM6.93353 8H7.73353H8.26686C9.59006 8 10.6669 9.0768 10.6669 10.4C10.6669 11.7232 9.59006 12.8 8.26686 12.8H6.93353V8Z"/></svg>"#;

/// `ItalicIcon` (`demos/multiple/tailwind/index.tsx:52-65`).
const ITALIC_ICON: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path d="M8.52599 2.12186C8.48583 2.12267 8.44578 2.1265 8.4062 2.13332H6.93328C6.86261 2.13232 6.79244 2.14538 6.72686 2.17173C6.66127 2.19808 6.60158 2.23721 6.55125 2.28683C6.50092 2.33646 6.46096 2.39559 6.43368 2.46079C6.4064 2.526 6.39235 2.59597 6.39235 2.66665C6.39235 2.73733 6.4064 2.80731 6.43368 2.87251C6.46096 2.93772 6.50092 2.99685 6.55125 3.04647C6.60158 3.0961 6.66127 3.13522 6.72686 3.16157C6.79244 3.18793 6.86261 3.20099 6.93328 3.19999H7.70099L6.69057 12.8H5.86661C5.79594 12.799 5.72577 12.812 5.66019 12.8384C5.59461 12.8648 5.53492 12.9039 5.48459 12.9535C5.43425 13.0031 5.39429 13.0623 5.36701 13.1275C5.33973 13.1927 5.32568 13.2626 5.32568 13.3333C5.32568 13.404 5.33973 13.474 5.36701 13.5392C5.39429 13.6044 5.43425 13.6635 5.48459 13.7131C5.53492 13.7628 5.59461 13.8019 5.66019 13.8282C5.72577 13.8546 5.79594 13.8677 5.86661 13.8667H9.06661C9.13729 13.8677 9.20745 13.8546 9.27304 13.8282C9.33862 13.8019 9.39831 13.7628 9.44864 13.7131C9.49897 13.6635 9.53894 13.6044 9.56622 13.5392C9.5935 13.474 9.60754 13.404 9.60754 13.3333C9.60754 13.2626 9.5935 13.1927 9.56622 13.1275C9.53894 13.0628 9.49897 13.0031 9.44864 12.9535C9.39831 12.9039 9.33862 12.8648 9.27304 12.8384C9.20745 12.812 9.13729 12.799 9.06661 12.8H8.2989L9.30932 3.19999H10.1333C10.204 3.20099 10.2741 3.18793 10.3397 3.16157C10.4053 3.13522 10.465 3.0961 10.5153 3.04647C10.5656 2.99685 10.6056 2.93772 10.6329 2.87251C10.6602 2.80731 10.6742 2.73733 10.6742 2.66665C10.6742 2.59597 10.6602 2.526 10.6329 2.46079C10.6056 2.39559 10.5656 2.33646 10.5153 2.28683C10.465 2.23721 10.4053 2.19808 10.3397 2.17173C10.2741 2.14538 10.204 2.13232 10.1333 2.13332H8.66349C8.61807 2.12555 8.57207 2.12171 8.52599 2.12186Z"/></svg>"#;

/// `UnderlineIcon` (`demos/multiple/tailwind/index.tsx:67-80`).
const UNDERLINE_ICON: &str = r#"<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" style="display: block"><path d="M3.73331 2.13332C3.66264 2.13232 3.59247 2.14538 3.52689 2.17173C3.46131 2.19809 3.40161 2.23721 3.35128 2.28684C3.30095 2.33646 3.26099 2.39559 3.23371 2.4608C3.20643 2.526 3.19238 2.59598 3.19238 2.66666C3.19238 2.73734 3.20643 2.80731 3.23371 2.87252C3.26099 2.93772 3.30095 2.99685 3.35128 3.04648C3.40161 3.0961 3.46131 3.13523 3.52689 3.16158C3.59247 3.18793 3.66264 3.20099 3.73331 3.19999V7.99999C3.73331 10.224 5.55144 12.2667 7.99998 12.2667C10.4485 12.2667 12.2666 10.224 12.2666 7.99999V3.19999C12.3373 3.20099 12.4075 3.18793 12.4731 3.16158C12.5386 3.13523 12.5983 3.0961 12.6487 3.04648C12.699 2.99685 12.739 2.93772 12.7662 2.87252C12.7935 2.80731 12.8076 2.73734 12.8076 2.66666C12.8076 2.59598 12.7935 2.526 12.7662 2.4608C12.739 2.39559 12.699 2.33646 12.6487 2.28684C12.5983 2.23721 12.5386 2.19809 12.4731 2.17173C12.4075 2.14538 12.3373 2.13232 12.2666 2.13332H10.1333C10.0626 2.13232 9.99247 2.14538 9.92689 2.17173C9.8613 2.19809 9.80161 2.23721 9.75128 2.28684C9.70095 2.33646 9.66099 2.39559 9.63371 2.4608C9.60643 2.526 9.59238 2.59598 9.59238 2.66666C9.59238 2.73734 9.60643 2.80731 9.63371 2.87252C9.66099 2.93772 9.70095 2.99685 9.75128 3.04648C9.80161 3.0961 9.8613 3.13523 9.92689 3.16158C9.99247 3.18793 10.0626 3.20099 10.1333 3.19999V8.97187C10.1333 10.0855 9.32179 11.0818 8.21352 11.1896C6.94152 11.3138 5.86665 10.3136 5.86665 9.06666V3.19999C5.93732 3.20099 6.00748 3.18793 6.07307 3.16158C6.13865 3.13523 6.19834 3.0961 6.24867 3.04648C6.299 2.99685 6.33897 2.93772 6.36625 2.87252C6.39353 2.80731 6.40757 2.73734 6.40757 2.66666C6.40757 2.59598 6.39353 2.526 6.36625 2.4608C6.33897 2.39559 6.299 2.33646 6.24867 2.28684C6.19834 2.23721 6.13865 2.19809 6.07307 2.17173C6.00748 2.14538 5.93732 2.13232 5.86665 2.13332H3.73331ZM3.73331 13.3333C3.66264 13.3323 3.59247 13.3454 3.52689 13.3717C3.46131 13.3981 3.40161 13.4372 3.35128 13.4868C3.30095 13.5365 3.26099 13.5956 3.23371 13.6608C3.20643 13.726 3.19238 13.796 3.19238 13.8667C3.19238 13.9373 3.20643 14.0073 3.23371 14.0725C3.26099 14.1377 3.30095 14.1969 3.35128 14.2465C3.40161 14.2961 3.46131 14.3352 3.52689 14.3616C3.59247 14.3879 3.66264 14.401 3.73331 14.4H12.2666C12.3373 14.401 12.4075 14.3879 12.4731 14.3616C12.5386 14.3352 12.5983 14.2961 12.6487 14.2465C12.699 14.1969 12.739 14.1377 12.7662 14.0725C12.7935 14.0073 12.8076 13.9373 12.8076 13.8667C12.8076 13.796 12.7935 13.726 12.7662 13.6608C12.739 13.5956 12.699 13.5365 12.6487 13.4868C12.5983 13.4372 12.5386 13.3981 12.4731 13.3717C12.4075 13.3454 12.3373 13.3323 12.2666 13.3333H3.73331Z"/></svg>"#;

/// One demo item: the child's `value`, its `aria-label`, and the icon that is its content.
type DemoItem = (&'static str, &'static str, &'static str);

/// The group's children — one [`Toggle`](toggle_element) per item, built inside the
/// `ChildrenFn` so each value-snapshot rebuild re-derives every child's membership
/// (`toggle_group.rs`, module docs).
///
/// The icon rides a render prop: upstream puts it in the JSX children
/// (`<Toggle><AlignLeftIcon /></Toggle>`), and the port's `ToggleProps` carries no children
/// slot, so the same markup is delivered as the rendered button's `inner_html` (the toggle
/// page's hero precedent). The render prop keeps the merged props it is handed — class,
/// style, the composite bag's `tabindex`/handlers — so the composite item stays intact.
fn icon_render_prop(icon: &'static str) -> RenderProp {
    RenderProp::Function(Rc::new(
        move |props: RenderElementProps,
              _state: &serde_json::Map<String, serde_json::Value>| RenderedElement {
            tag: "button".to_string(),
            props: RenderElementProps {
                handlers: props.handlers,
                inner_html: Some(icon.to_string()),
                class: props.class,
                style: props.style,
                ref_callback: props.ref_callback,
            },
        },
    ))
}

/// One demo (`demos/hero/tailwind/index.tsx` / `demos/multiple/tailwind/index.tsx`) on the
/// real port: upstream's panel `className`, `defaultValue`, `aria-label` and child set.
///
/// The children are a `move ||` closure so the `ToggleGroup` component's `ChildrenFn` prop
/// receives the per-value-snapshot rebuild the unit documents: `toggle_group_view` re-runs its
/// subtree closure whenever the group's value changes (`crates/leptos-ui/src/toggle_group.rs`,
/// module docs), which is what makes each click re-derive every child's pressed state.
fn group_demo(
    multiple: bool,
    default_value: Vec<String>,
    aria_label: &'static str,
    marker: &'static str,
    button_class: &'static str,
    items: &'static [DemoItem],
) -> impl IntoView {
    view! {
        <div class="docs-demo" data-demo=marker>
            <ToggleGroup
                multiple=multiple
                default_value=default_value
                class=PANEL_CLASS.to_string()
                element_attributes=vec![
                    ("aria-label".to_string(), aria_label.to_string()),
                ]
            >
                {move || {
                    let mut toggles: Vec<AnyView> = Vec::with_capacity(items.len());
                    for &(value, label, icon) in items.iter() {
                        let rendered = toggle_element(ToggleProps {
                            value: Some(value.to_string()),
                            render_class_style: UseRenderElementComponentProps {
                                class_name: Some(ClassNameSource::Static(
                                    (*button_class).to_string(),
                                )),
                                render: Some(icon_render_prop(icon)),
                                style: None,
                            },
                            // `aria-label="Align left"` (`:19`) — upstream's `...elementProps` rest.
                            element_attributes: vec![(
                                "aria-label".to_string(),
                                (*label).to_string(),
                            )],
                            ..Default::default()
                        })
                        .expect("a grouped Toggle renders through the group's composite root");
                        let (element, cleanup) = rendered.create_element();
                        // The node outlives the mount; the listeners' cleanup is leaked with it
                        // (the use-render/toggle page convention).
                        std::mem::forget(cleanup);
                        toggles.push(RawElementView { element }.into_any());
                    }
                    toggles.into_iter().collect_view().into_any()
                }}
            </ToggleGroup>
        </div>
    }
}

/// The hero demo (`demos/hero/tailwind/index.tsx:6-41`): a single-selection group seeded with
/// `['left']`, its three alignment toggles, and `aria-label="Text alignment"`.
pub fn toggle_group_hero_demo() -> impl IntoView {
    group_demo(
        false,
        vec!["left".to_string()],
        "Text alignment",
        "hero",
        HERO_BUTTON_CLASS,
        &[
            ("left", "Align left", ALIGN_LEFT_ICON),
            ("center", "Align center", ALIGN_CENTER_ICON),
            ("right", "Align right", ALIGN_RIGHT_ICON),
        ],
    )
}

/// The `### Multiple` demo (`demos/multiple/tailwind/index.tsx:5-30`): `multiple` with
/// `['bold', 'italic']` seeded, and `aria-label="Text formatting options"`.
pub fn toggle_group_multiple_demo() -> impl IntoView {
    group_demo(
        true,
        vec!["bold".to_string(), "italic".to_string()],
        "Text formatting options",
        "multiple",
        MULTIPLE_BUTTON_CLASS,
        &[
            ("bold", "Bold", BOLD_ICON),
            ("italic", "Italic", ITALIC_ICON),
            ("underline", "Underline", UNDERLINE_ICON),
        ],
    )
}

/// The `docs/src/app/(docs)/react/components/toggle-group/page.mdx` page.
#[component]
pub fn ToggleGroupPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Toggle Group"</h1>
            <p class="subtitle">"Provides a shared state to a series of toggle buttons."</p>

            {toggle_group_hero_demo()}

            <h2>"Anatomy"</h2>
            <p>"Import the component and use it as a single part:"</p>
            {code_block(Lang::Rust, "Anatomy", ANATOMY_SNIPPET)}

            <h2>"Examples"</h2>

            <h3>"Multiple"</h3>
            <p>"Add the `multiple` prop to allow pressing more than one toggle at a time."</p>
            {toggle_group_multiple_demo()}

            <h2>"API reference"</h2>
            <p>
                "The group is a single part: it renders one `div` with `role=\"group\"` and "
                "exposes no subcomponents of its own. Its children are `Toggle`s from the "
                "`toggle` unit, identified by their `value` prop, which read the group's value "
                "through the toggle-group context."
            </p>
            <p>
                "Props: `defaultValue` (the pressed state as an array of the pressed toggles' "
                "values; the uncontrolled counterpart of `value`), `value` (the controlled "
                "counterpart of `defaultValue`), `onValueChange` "
                "(`(groupValue: Vec<String>, eventDetails: ToggleGroupChangeEventDetails) -> ()`), "
                "`loopFocus` (default `true`; whether arrow-key focus loops back to the first "
                "item), `multiple` (default `false`; when `false` only one item can be pressed), "
                "`disabled` (default `false`), `orientation` (default `\"horizontal\"`), plus "
                "`class` (the `className` prop), `style` (ordered declarations) and "
                "`element_attributes` (upstream's `...elementProps` rest)."
            </p>
            <p>
                "Data attributes: `data-orientation` (the orientation string), `data-disabled` "
                "(present when the group is disabled) and `data-multiple` (present when the "
                "group allows several pressed items)."
            </p>
            <p>
                "`ToggleGroup.State` is `{ disabled: boolean, multiple: boolean, orientation: "
                "Orientation }`. `ToggleGroup.ChangeEventDetails` exposes `reason`, `event`, "
                "`cancel()`, `allowPropagation()`, `isCanceled`, `isPropagationAllowed` and "
                "`trigger`; `ToggleGroup.ChangeEventReason` is `'none'`. `Orientation` is "
                "`'horizontal' | 'vertical'`. Canonical spellings: `ToggleGroupState`, "
                "`ToggleGroupProps`, `ToggleGroupChangeEventReason`, "
                "`ToggleGroupChangeEventDetails`."
            </p>
        </article>
    }
}

#[cfg(test)]
mod snippet_language_guard {
    use super::*;
    use crate::snippet_language::{SnippetLanguage, classify};

    /// Upstream's Anatomy block (`docs/src/app/(docs)/react/components/toggle-group/page.mdx:17-21`),
    /// kept as the classifier's positive control so the assertion below cannot pass vacuously if
    /// `looks_react` ever stops recognising upstream's JSX shape. The package specifier upstream's
    /// import line carries is deliberately left out: this is page source, not reader-facing, and the
    /// sibling pages' controls omit it for the same reason (`toggle_page.rs:256`, `field_page.rs:224-227`).
    const UPSTREAM_ANATOMY_SHAPE: &str = "// prettier-ignore\n<ToggleGroup />;";

    #[test]
    fn the_classifier_recognises_upstream_source() {
        assert_eq!(
            classify(UPSTREAM_ANATOMY_SHAPE),
            SnippetLanguage::React,
            "the classifier no longer recognises upstream's source shape — the assertion below would \
             be vacuous"
        );
    }

    /// Every code block this page embeds, in document order. The page carries exactly one fence —
    /// the Anatomy listing (upstream's page inlines exactly one too; the two demos and the
    /// generated types are component imports there, and real demos here).
    fn page_snippets() -> [(&'static str, &'static str); 1] {
        [("Anatomy", ANATOMY_SNIPPET)]
    }

    /// The page-level number the probe reads: `{total: 1, leptos: 1, react: 0, other: 0}`.
    #[test]
    fn the_pages_snippets_all_teach_the_port() {
        let languages: Vec<(&str, SnippetLanguage)> = page_snippets()
            .iter()
            .map(|(name, text)| (*name, classify(text)))
            .collect();
        assert_eq!(
            languages,
            vec![("Anatomy", SnippetLanguage::Leptos)],
            "the probe must read {{total: 1, leptos: 1, react: 0, other: 0}} for this page"
        );
    }

    /// The snippet's own composition, compiled. Never called: the compiler checks the crate
    /// paths, the props structs and the `RenderedElement` API the page teaches, so a snippet
    /// naming an API the port does not have fails the build instead of shipping.
    #[allow(dead_code)]
    fn anatomy_snippet_shape() {
        let rendered = toggle_element(ToggleProps {
            value: Some("left".to_string()),
            ..Default::default()
        })
        .expect("a grouped Toggle renders through the group's composite root");
        let (_element, _cleanup) = rendered.create_element();
        let _ = ToggleGroup;
    }
}
