//! The docs page for `Checkbox Group`, mirroring
//! `docs/src/app/(docs)/react/components/checkbox-group/page.mdx`
//! (`specs/docs-content/checkbox-group/page.md`) — the
//! `docs-content: components/checkbox-group` TODO item.
//!
//! Page structure per the spec's "Page structure (headings, in order)" section:
//! `# Checkbox Group` h1, the `<Subtitle>` ("Provides shared state to a series of
//! checkboxes."), the hero demo before the first heading, `## Usage guidelines`,
//! `## Anatomy` with its fenced snippet, `## Examples` over "Labeling a checkbox
//! group" (`page.mdx:32-50`), "Rendering as a native button" (`:52-87`), "Form
//! integration" (`:89-120`), the "Parent checkbox" recipe (`:122-134`) and the
//! "Nested parent checkbox" demo (`:136-140`), and `## API reference`
//! (`page.mdx:142-146`) over the generated `TypesCheckboxGroup` tables
//! (`types.md`) echoed as static prose per the accordion/button/meter/field/checkbox
//! page precedent: the port has no docs generator, so the table's documented props,
//! data attributes and types are rendered as text, never fabricated as executable
//! machinery.
//!
//! Page furniture mirrored in module docs (the checkbox/field/accordion page
//! precedent): the `<Meta name="description">` content — "A high-quality, unstyled
//! React checkbox group component that provides a shared state for a series of
//! checkboxes." (`page.mdx:4-7`) — and the trailing `export const metadata` SEO
//! keywords block (11 keywords, `page.mdx:148-162`: 'React Checkbox Group',
//! 'Checkbox Group Component', 'Grouped Checkboxes', 'Multiple Selection',
//! 'Checkbox List', 'Multi-Select Checkboxes', 'Parent Child Checkbox', 'Accessible
//! Checkbox Group', 'Form Fieldset Control', 'Headless React Components',
//! 'Base UI').
//!
//! ## The three live demos and the surface they needed
//!
//! All three `specs/docs-content/checkbox-group/demos.json` entries use the same
//! three parts (`CheckboxGroup`, `Checkbox.Root`, `Checkbox.Indicator`), nested the
//! upstream way — the checkboxes INSIDE the group element, which is what the group's
//! context provider wraps (`CheckboxGroup.tsx:173-177`). The crate's element-only
//! builder returns an element description and takes no children, so this pair's
//! PAIR-PORTABILITY GAP was the composition root; it landed in the owner crate as
//! [`leptos_ui::checkbox_group_view`] (with the two live-source props the controlled
//! recipes need — `CheckboxGroupProps::value_source` and
//! `CheckboxRootViewProps::indeterminate_source`, both cited to the demos below).
//! The demos here compose ONLY real parts: no demo-side machinery beyond the state
//! the upstream demo itself holds.
//!
//! - **hero** (`demos/hero/tailwind/index.tsx`): uncontrolled, `defaultValue=['fuji-apple']`,
//!   an `aria-labelledby` link to a sibling caption div, three enclosing `<label>`
//!   items each wrapping a `Checkbox.Root` (with `name`/`value`) and its
//!   `Checkbox.Indicator` checkmark. Static — demos.json `stateManaged` is
//!   "uncontrolled — group state held internally".
//! - **parent** (`demos/parent/css-modules/index.tsx`): controlled
//!   (`useState<string[]>([])` + `onValueChange={setValue}`), `allValues=fruits`, a
//!   `parent` `Checkbox.Root` whose `Checkbox.Indicator` uses the state-driven
//!   `render` callback to swap the check for a horizontal rule while
//!   `state.indeterminate`.
//! - **nested** (`demos/nested/css-modules/index.tsx`): two controlled groups with
//!   their values manually synced in both `onValueChange` handlers, and the outer
//!   parent's `indeterminate` computed from the INNER group's value (the outer group
//!   only sees `manage-users` as one flat value).
//!
//! ## The reactive machinery (React's `useState` mirror)
//!
//! The two controlled demos hold their state in leptos `RwSignal`s (the docs-app
//! convention — the progress page's `useState` mirror) and the `onValueChange`
//! handlers are the `setValue` writes. `CheckboxGroup`'s controlled source, however,
//! is typed over the internals crate's rg-0.2 runtime (`use_controlled`'s source and
//! the group context the checkbox parts mirror), while the page's state is a leptos
//! signal: [`mirror_leptos_to_rg`] keeps an rg-0.2 mirror in lockstep, one leptos
//! effect per source — the same direction the crate's own `transition_status_signal`
//! documents ("the rg-0.2 open mirror: written by a leptos effect, read by the
//! hook"). Nothing rebuilds on a change: the group's context value updates and every
//! child's live derivation (its `group_value` mirror, the parent engine's tri-state)
//! re-fires, exactly as React's re-render re-reads the context — the subtree mounts
//! once, so the hidden inputs and focus are not churned (the fresh-node-rebuild
//! hazard the crate records for stateful elements).
//!
//! ## Documented adaptation (never silent)
//!
//! The `parent`/`nested` demos' `Checkbox.Indicator` `render` callbacks return their
//! ELEMENT upstream (`<span {...props}>…</span>`, `demos/parent/css-modules/index.tsx:26-28`);
//! leptos `view!` has no attribute spread, so the port's Indicator renders its own
//! `<span>` (the element whose attribute surface it owns, `CheckboxIndicator.tsx:60-64`)
//! and hands the callback the STATE, whose members the callback selects on — the
//! `Field.Validity` state-callback precedent. The state object itself is upstream's
//! full `CheckboxIndicatorState` (`CheckboxIndicator.tsx:33-36` = the root state plus
//! `transitionStatus`). Recorded in `ralph/logs/spec-discrepancies.md`.
//!
//! ## CSS-modules variants
//!
//! `parent` and `nested` ship only a `css-modules` variant
//! (`demos/{parent,nested}/index.ts` register `{ CssModules }` alone), so the demos
//! have no Tailwind class strings to carry: the module's own class names
//! (`CheckboxGroup`, `Item`, `Checkbox`, `Indicator`) are carried verbatim as literal
//! class strings, the way the Tailwind variants' class strings are. docs-app ships no
//! stylesheet at all (the Tailwind classes are equally inert here), so the DEMO's
//! element-for-element DOM shape is what these ports pin.

use leptos::prelude::*;

use leptos_ui::checkbox_group_view;
use leptos_ui::checkbox_indicator_view;
use leptos_ui::checkbox_root_view;
use leptos_ui::{
    CheckboxGroupChangeEventDetails, CheckboxGroupViewProps, CheckboxIndicatorRenderState,
    CheckboxIndicatorViewProps, CheckboxRootViewProps,
};
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::Get as RgGet;
use reactive_graph::traits::GetUntracked as RgGetUntracked;
use reactive_graph::traits::Set as RgSet;
use reactive_graph::wrappers::read::Signal as RgSignal;

/// The hero demo's group `className` (`demos/hero/tailwind/index.tsx:12`), carried
/// verbatim.
const HERO_GROUP_CLASS: &str = "flex flex-col items-start gap-1 text-neutral-950 dark:text-white";

/// The hero demo's caption `className` (`hero/tailwind/index.tsx:14`).
const HERO_CAPTION_CLASS: &str = "text-sm font-bold";

/// The hero demo's per-item `<label>` `className` (`:18`, `:31`, `:44`).
const HERO_ITEM_CLASS: &str =
    "flex items-center gap-2 text-sm font-normal text-neutral-950 dark:text-white";

/// The hero demo's checkbox `className` (`:22`, `:35`, `:48`) — including the
/// `data-checked:`/`focus-visible:` variants the port's real state attributes feed
/// through the stylesheet.
const HERO_CHECKBOX_CLASS: &str = "flex size-4 shrink-0 items-center justify-center border rounded-none p-0 border-neutral-950 bg-white text-white dark:border-white dark:bg-neutral-950 dark:text-neutral-950 data-checked:bg-neutral-950 data-checked:text-white dark:data-checked:bg-white dark:data-checked:text-neutral-950 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-950 dark:focus-visible:outline-white";

/// The hero demo's Indicator `className` (`:24`, `:37`, `:50`) — its
/// `data-unchecked:hidden` variant is the demo's hiding mechanism.
const HERO_INDICATOR_CLASS: &str = "flex data-unchecked:hidden";

/// The parent/nested demos' module class names (`demos/{parent,nested}/css-modules/index.module.css`),
/// carried verbatim as literals (see the module docs' CSS-modules note).
const MODULE_GROUP_CLASS: &str = "CheckboxGroup";
const MODULE_ITEM_CLASS: &str = "Item";
const MODULE_CHECKBOX_CLASS: &str = "Checkbox";
const MODULE_INDICATOR_CLASS: &str = "Indicator";

/// The parent demo's `const fruits = ['fuji-apple', 'gala-apple',
/// 'granny-smith-apple']` (`demos/parent/css-modules/index.tsx:7`) — the group's
/// `allValues`, in upstream's order.
const PARENT_FRUITS: [&str; 3] = ["fuji-apple", "gala-apple", "granny-smith-apple"];

/// The nested demo's `mainPermissions` (`demos/nested/css-modules/index.tsx:7`).
const NESTED_MAIN_PERMISSIONS: [&str; 3] = ["view-dashboard", "manage-users", "access-reports"];

/// The nested demo's `userManagementPermissions` (`:8`) — the length is what the
/// demo's two sync handlers and the outer parent's `indeterminate` compare against.
const NESTED_USER_PERMISSIONS: [&str; 4] =
    ["create-user", "edit-user", "delete-user", "assign-roles"];

/// The `## Anatomy` snippet (`page.mdx:21-28`), carried verbatim.
const ANATOMY_SNIPPET: &str = "import { Checkbox } from '@base-ui/react/checkbox';\nimport { CheckboxGroup } from '@base-ui/react/checkbox-group';\n\n<CheckboxGroup>\n  <Checkbox.Root />\n</CheckboxGroup>;";

/// The "Using aria-labelledby to label a checkbox group" snippet (`page.mdx:36-39`).
const LABELLEDBY_SNIPPET: &str = "<div id=\"protocols-label\">Allowed network protocols</div>\n<CheckboxGroup aria-labelledby=\"protocols-label\">{/* ... */}</CheckboxGroup>";

/// The "Using an enclosing label to label a checkbox" snippet (`page.mdx:43-50`) —
/// including its `// @highlight` directives, which are comments in the source.
const ENCLOSING_LABEL_SNIPPET: &str = "// @highlight\n<label>\n  <Checkbox.Root value=\"http\" />\n  HTTP\n  {/* @highlight */}\n</label>";

/// The "Sibling label pattern with a native button" snippet (`page.mdx:56-67`).
const NATIVE_BUTTON_SNIPPET: &str = "<div id=\"protocols-label\">Allowed network protocols</div>\n<CheckboxGroup aria-labelledby=\"protocols-label\">\n  <div>\n    <label htmlFor=\"protocol-http\">HTTP</label>\n    {/* @highlight-text \"nativeButton\" \"render={<button />}\" */}\n    <Checkbox.Root id=\"protocol-http\" value=\"http\" nativeButton render={<button />}>\n      <Checkbox.Indicator />\n    </Checkbox.Root>\n  </div>\n</CheckboxGroup>";

/// The "Render callback" snippet (`page.mdx:71-87`) — the invalid-HTML rationale the
/// page spec flags under Discrepancies as unproven by the unit's behavior spec.
const RENDER_CALLBACK_SNIPPET: &str = "<div id=\"protocols-label\">Allowed network protocols</div>\n<CheckboxGroup aria-labelledby=\"protocols-label\">\n  <Checkbox.Root\n    value=\"http\"\n    nativeButton\n    // @highlight-start\n    render={(buttonProps) => (\n      <label>\n        <button {...buttonProps} />\n        HTTP\n      </label>\n    )}\n    {/* @highlight-end */}\n  />\n</CheckboxGroup>";

/// The "Using Checkbox Group in a form" snippet (`page.mdx:93-120`).
const FORM_SNIPPET: &str = "<Form>\n  {/* @highlight */}\n  <Field.Root name=\"allowedNetworkProtocols\">\n    <Fieldset.Root render={<CheckboxGroup />}>\n      <Fieldset.Legend>Allowed network protocols</Fieldset.Legend>\n      <Field.Item>\n        <Field.Label>\n          <Checkbox.Root value=\"http\" />\n          HTTP\n        </Field.Label>\n      </Field.Item>\n      <Field.Item>\n        <Field.Label>\n          <Checkbox.Root value=\"https\" />\n          HTTPS\n        </Field.Label>\n      </Field.Item>\n      <Field.Item>\n        <Field.Label>\n          <Checkbox.Root value=\"ssh\" />\n          SSH\n        </Field.Label>\n      </Field.Item>\n    </Fieldset.Root>\n  </Field.Root>\n</Form>";

/// The demos' checkmark (`demos/*/…/index.tsx`, the `CheckIcon` helper): a 16×16
/// stroke svg with `display: block` inline, exactly as upstream renders it inside the
/// Indicator.
fn check_icon_view() -> AnyView {
    view! {
        <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            style="display:block"
        >
            <path d="m2.5 8.5 4 4 7-9" />
        </svg>
    }
    .into_any()
}

/// The demos' horizontal rule (`demos/{parent,nested}/…/index.tsx`, the
/// `HorizontalRuleIcon` helper): a 12×12 `currentColor` fill with the
/// `vectorEffect="non-scaling-stroke"` line — the indicator content the `render`
/// callback swaps in while `state.indeterminate`.
fn horizontal_rule_icon_view() -> AnyView {
    view! {
        <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="currentColor"
            stroke-width="1"
            style="display:block"
        >
            <line
                x1="3"
                y1="12"
                x2="21"
                y2="12"
                stroke="currentColor"
                vector-effect="non-scaling-stroke"
            />
        </svg>
    }
    .into_any()
}

/// The state-driven Indicator content the parent/nested demos use
/// (`render={(props, state) => … state.indeterminate ? <HorizontalRuleIcon /> : <CheckIcon />}`):
/// the port's Indicator owns the `<span>` (no attribute spread in `view!` — the module
/// docs' documented adaptation), so the callback returns the ICON the state selects.
fn indeterminate_indicator_content() -> AnyView {
    checkbox_indicator_view(CheckboxIndicatorViewProps {
        class: Some(MODULE_INDICATOR_CLASS.to_string()),
        render: Some(std::rc::Rc::new(|state: CheckboxIndicatorRenderState| {
            if state.indeterminate {
                horizontal_rule_icon_view()
            } else {
                check_icon_view()
            }
        })),
        ..CheckboxIndicatorViewProps::default()
    })
    .into_any()
}

/// One item of a module-variant demo: `<label className={styles.Item}>` wrapping the
/// checkbox and the item text (the parent demo's item shape,
/// `demos/parent/css-modules/index.tsx:34-41`) — the `value` is the group-membership
/// value the checkbox reports (`<Checkbox.Root value=…>`).
fn module_item(value: &'static str, text: &'static str) -> AnyView {
    view! {
        <label class=MODULE_ITEM_CLASS>
            {checkbox_root_view(CheckboxRootViewProps {
                value: Some(value.to_string()),
                class: Some(MODULE_CHECKBOX_CLASS.to_string()),
                children: Some(Box::new(|| {
                    checkbox_indicator_view(CheckboxIndicatorViewProps {
                        class: Some(MODULE_INDICATOR_CLASS.to_string()),
                        children: Some(std::sync::Arc::new(check_icon_view)),
                        ..CheckboxIndicatorViewProps::default()
                    })
                    .into_any()
                })),
                ..CheckboxRootViewProps::default()
            })}
            {text}
        </label>
    }
    .into_any()
}

/// The rg-0.2 mirror of a leptos-held controlled `value` — the group's `use_controlled`
/// source is typed over the internals crate's runtime while the page's state is a leptos
/// signal, so a leptos effect keeps an rg-0.2 signal in lockstep and the returned derived
/// signal is what `CheckboxGroupProps::value_source` reads (the crate's own sanctioned
/// direction for this bridge — `transition_status_signal`'s "rg-0.2 open mirror: written
/// by a leptos effect, read by the hook"). One per controlled group.
fn mirror_leptos_to_rg(
    source: RwSignal<Vec<String>>,
) -> RgSignal<Option<Vec<String>>, LocalStorage> {
    let mirror = reactive_graph::signal::RwSignal::new_local(Vec::<String>::new());
    let mirror_for_effect = mirror.clone();
    Effect::new(move |_| {
        let next = source.get();
        if RgGetUntracked::get_untracked(&mirror_for_effect) != next {
            RgSet::set(&mirror_for_effect, next);
        }
    });
    RgSignal::derive_local(move || Some(RgGet::get(&mirror)))
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): upstream's
/// exact JSX shape, in upstream's element order — the caption div first, then the three
/// enclosing `<label>` items, each wrapping the checkbox and its Indicator checkmark.
/// Uncontrolled, so nothing here is reactive: `defaultValue` pre-ticks `fuji-apple`.
#[component]
pub fn CheckboxGroupHeroDemo() -> impl IntoView {
    let id =
        use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>)).get_untracked();

    let caption_id = id.clone();
    let children = Box::new(move || {
        let items = [
            ("apple", "fuji-apple", "Fuji"),
            ("apple", "gala-apple", "Gala"),
            ("apple", "granny-smith-apple", "Granny Smith"),
        ];
        let mut nodes: Vec<AnyView> = vec![
            view! { <div class=HERO_CAPTION_CLASS id=caption_id.clone()>"Apples"</div> }.into_any(),
        ];
        for (name, value, text) in items {
            nodes.push(
                view! {
                    <label class=HERO_ITEM_CLASS>
                        {checkbox_root_view(CheckboxRootViewProps {
                            name: Some(name.to_string()),
                            value: Some(value.to_string()),
                            class: Some(HERO_CHECKBOX_CLASS.to_string()),
                            children: Some(Box::new(|| {
                                checkbox_indicator_view(CheckboxIndicatorViewProps {
                                    class: Some(HERO_INDICATOR_CLASS.to_string()),
                                    children: Some(std::sync::Arc::new(check_icon_view)),
                                    ..CheckboxIndicatorViewProps::default()
                                })
                                .into_any()
                            })),
                            ..CheckboxRootViewProps::default()
                        })}
                        {text}
                    </label>
                }
                .into_any(),
            );
        }
        nodes.into_iter().collect_view().into_any()
    });

    view! {
        {checkbox_group_view(CheckboxGroupViewProps {
            element_attributes: vec![("aria-labelledby".to_string(), id)],
            default_value: Some(vec!["fuji-apple".to_string()]),
            class: Some(HERO_GROUP_CLASS.to_string()),
            children: Some(children),
            ..CheckboxGroupViewProps::default()
        })}
    }
}

/// The parent demo (`demos/parent/css-modules/index.tsx`, demos.json entry 2): the
/// controlled recipe — `value`/`onValueChange` (the leptos state mirror + its rg-0.2
/// bridge), `allValues={fruits}`, and the `parent` tri-state checkbox whose Indicator
/// swaps icons on `state.indeterminate`. Upstream's inline `marginLeft` on the group
/// and the parent label's `-1rem` counter-shift are carried as declarations.
#[component]
pub fn CheckboxGroupParentDemo() -> impl IntoView {
    let value = RwSignal::new(Vec::<String>::new());
    let value_source = mirror_leptos_to_rg(value);

    let id =
        use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>)).get_untracked();

    let parent_id = id.clone();
    let children = Box::new(move || {
        let mut nodes: Vec<AnyView> = vec![
            view! {
                <label class=MODULE_ITEM_CLASS id=parent_id.clone() style="margin-left:-1rem">
                    {checkbox_root_view(CheckboxRootViewProps {
                        parent: true,
                        class: Some(MODULE_CHECKBOX_CLASS.to_string()),
                        children: Some(Box::new(indeterminate_indicator_content)),
                        ..CheckboxRootViewProps::default()
                    })}
                    "Apples"
                </label>
            }
            .into_any(),
        ];
        for (value_, text) in [
            (PARENT_FRUITS[0], "Fuji"),
            (PARENT_FRUITS[1], "Gala"),
            (PARENT_FRUITS[2], "Granny Smith"),
        ] {
            nodes.push(module_item(value_, text));
        }
        nodes.into_iter().collect_view().into_any()
    });

    view! {
        {checkbox_group_view(CheckboxGroupViewProps {
            element_attributes: vec![("aria-labelledby".to_string(), id)],
            value_source: Some(value_source),
            on_value_change: Some(std::rc::Rc::new(
                move |next: Vec<String>, _details: &CheckboxGroupChangeEventDetails| {
                    // `onValueChange={setValue}` (`:17`) — React's state setter verbatim.
                    value.set(next);
                },
            )),
            all_values: Some(PARENT_FRUITS.iter().map(|v| v.to_string()).collect()),
            class: Some(MODULE_GROUP_CLASS.to_string()),
            style: vec![("margin-left".to_string(), "1rem".to_string())],
            children: Some(children),
            ..CheckboxGroupViewProps::default()
        })}
    }
}

/// The nested demo (`demos/nested/css-modules/index.tsx`, demos.json entry 3): two
/// controlled groups whose values are synced by hand in both handlers, with the outer
/// parent's `indeterminate` computed from the inner group's value (upstream's
/// `managementValue.length > 0 && managementValue.length !== userManagementPermissions.length`,
/// `:35-38`) — the live read `CheckboxRootViewProps::indeterminate_source` exists for,
/// since a one-shot flag could not follow the inner group's changes.
#[component]
pub fn CheckboxGroupNestedDemo() -> impl IntoView {
    let main_value = RwSignal::new(Vec::<String>::new());
    let management_value = RwSignal::new(Vec::<String>::new());
    let main_source = mirror_leptos_to_rg(main_value);
    let management_source = mirror_leptos_to_rg(management_value);

    let id =
        use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>)).get_untracked();

    // The outer group's `onValueChange` (`:19-26`): checking `manage-users` ticks every
    // inner permission; unchecking it clears the inner group; the main value follows.
    let main_for_change = main_value;
    let management_for_change = management_value;
    let outer_on_change = std::rc::Rc::new(
        move |next: Vec<String>, _details: &CheckboxGroupChangeEventDetails| {
            if next.iter().any(|v| v == "manage-users") {
                management_for_change.set(
                    NESTED_USER_PERMISSIONS
                        .iter()
                        .map(|v| v.to_string())
                        .collect(),
                );
            } else if management_for_change.get().len() == NESTED_USER_PERMISSIONS.len() {
                management_for_change.set(Vec::new());
            }
            main_for_change.set(next);
        },
    );

    let parent_id = id.clone();
    let children = Box::new(move || {
        let mut nodes: Vec<AnyView> = vec![
            view! {
                <label class=MODULE_ITEM_CLASS id=parent_id.clone() style="margin-left:-1rem">
                    {checkbox_root_view(CheckboxRootViewProps {
                        parent: true,
                        class: Some(MODULE_CHECKBOX_CLASS.to_string()),
                        indeterminate_source: Some(Signal::derive(move || {
                            let current = management_value.get();
                            !current.is_empty() && current.len() != NESTED_USER_PERMISSIONS.len()
                        })),
                        children: Some(Box::new(indeterminate_indicator_content)),
                        ..CheckboxRootViewProps::default()
                    })}
                    "User Permissions"
                </label>
            }
            .into_any(),
            module_item(NESTED_MAIN_PERMISSIONS[0], "View Dashboard"),
            module_item(NESTED_MAIN_PERMISSIONS[2], "Access Reports"),
        ];

        // The inner group (`:68-95`): a nested `CheckboxGroup` providing its own context,
        // syncing upward into the main value.
        let management_source_for_inner = send_wrapper::SendWrapper::new(management_source);
        let main_for_inner = main_value;
        let management_for_inner = management_value;
        let inner_on_change = std::rc::Rc::new(
            move |next: Vec<String>, _details: &CheckboxGroupChangeEventDetails| {
                if next.len() == NESTED_USER_PERMISSIONS.len() {
                    let mut merged = main_for_inner.get();
                    if !merged.iter().any(|v| v == "manage-users") {
                        merged.push("manage-users".to_string());
                    }
                    main_for_inner.set(merged);
                } else {
                    let filtered: Vec<String> = main_for_inner
                        .get()
                        .into_iter()
                        .filter(|v| v != "manage-users")
                        .collect();
                    main_for_inner.set(filtered);
                }
                management_for_inner.set(next);
            },
        );
        let inner_children = Box::new(move || {
            let mut inner_nodes: Vec<AnyView> = vec![
                view! {
                    <label class=MODULE_ITEM_CLASS id="manage-users-caption" style="margin-left:-1rem">
                        {checkbox_root_view(CheckboxRootViewProps {
                            parent: true,
                            class: Some(MODULE_CHECKBOX_CLASS.to_string()),
                            children: Some(Box::new(indeterminate_indicator_content)),
                            ..CheckboxRootViewProps::default()
                        })}
                        "Manage Users"
                    </label>
                }
                .into_any(),
            ];
            for (value_, text) in [
                (NESTED_USER_PERMISSIONS[0], "Create User"),
                (NESTED_USER_PERMISSIONS[1], "Edit User"),
                (NESTED_USER_PERMISSIONS[2], "Delete User"),
                (NESTED_USER_PERMISSIONS[3], "Assign Roles"),
            ] {
                inner_nodes.push(module_item(value_, text));
            }
            inner_nodes.into_iter().collect_view().into_any()
        });

        nodes.push(
            view! {
                {checkbox_group_view(CheckboxGroupViewProps {
                    element_attributes: vec![(
                        "aria-labelledby".to_string(),
                        "manage-users-caption".to_string(),
                    )],
                    value_source: Some((*management_source_for_inner).clone()),
                    on_value_change: Some(inner_on_change),
                    all_values: Some(
                        NESTED_USER_PERMISSIONS.iter().map(|v| v.to_string()).collect(),
                    ),
                    class: Some(MODULE_GROUP_CLASS.to_string()),
                    style: vec![("margin-left".to_string(), "1rem".to_string())],
                    children: Some(inner_children),
                    ..CheckboxGroupViewProps::default()
                })}
            }
            .into_any(),
        );
        nodes.into_iter().collect_view().into_any()
    });

    view! {
        {checkbox_group_view(CheckboxGroupViewProps {
            element_attributes: vec![("aria-labelledby".to_string(), id)],
            value_source: Some(main_source),
            on_value_change: Some(outer_on_change),
            all_values: Some(NESTED_MAIN_PERMISSIONS.iter().map(|v| v.to_string()).collect()),
            class: Some(MODULE_GROUP_CLASS.to_string()),
            style: vec![("margin-left".to_string(), "1rem".to_string())],
            children: Some(children),
            ..CheckboxGroupViewProps::default()
        })}
    }
}

/// One API-reference block: the generated `TypesCheckboxGroup` table
/// (`docs/src/app/(docs)/react/components/checkbox-group/types.md`) echoed as static
/// prose — the summary line, the props list, and the data-attributes list.
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
    }
}

/// The `docs/src/app/(docs)/react/components/checkbox-group/page.mdx` page.
#[component]
pub fn CheckboxGroupPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Checkbox Group"</h1>
            <p class="subtitle">"Provides shared state to a series of checkboxes."</p>

            <div class="docs-demo" data-demo="hero"><CheckboxGroupHeroDemo /></div>

            <h2>"Usage guidelines"</h2>
            <ul>
                <li>
                    <strong>"Form controls must have an accessible name"</strong>
                    ": It can be created using `<label>` elements, or the `Field` and `Fieldset` "
                    "components. See "
                    <a href="#labeling-a-checkbox-group">"Labeling a checkbox group"</a>
                    " and the "
                    <a href="/react/handbook/forms">"forms guide"</a>
                    "."
                </li>
            </ul>

            <h2>"Anatomy"</h2>
            <p>
                "Checkbox Group is composed together with "
                <a href="/react/components/checkbox">"Checkbox"</a>
                ". Import the components and place them together:"
            </p>
            <pre><code>{ANATOMY_SNIPPET}</code></pre>

            <h2>"Examples"</h2>

            <h3>"Labeling a checkbox group"</h3>
            <p>"Label the group with `aria-labelledby` and a sibling label element:"</p>
            <pre><code>{LABELLEDBY_SNIPPET}</code></pre>
            <p>"An enclosing `<label>` is the simplest labeling pattern for each checkbox:"</p>
            <pre><code>{ENCLOSING_LABEL_SNIPPET}</code></pre>

            <h3>"Rendering as a native button"</h3>
            <p>
                "By default, `<Checkbox.Root>` renders a `<span>` element to support enclosing "
                "labels. Prefer rendering each checkbox as a native button when using sibling "
                "labels (`htmlFor`/`id`)."
            </p>
            <pre><code>{NATIVE_BUTTON_SNIPPET}</code></pre>
            <p>
                "Native buttons with wrapping labels are supported by using the `render` callback "
                "to avoid invalid HTML, so the hidden input is placed outside the label:"
            </p>
            <pre><code>{RENDER_CALLBACK_SNIPPET}</code></pre>

            <h3>"Form integration"</h3>
            <p>
                "Use "
                <a href="/react/components/field">"Field"</a>
                " and "
                <a href="/react/components/fieldset">"Fieldset"</a>
                " for group labeling and form integration:"
            </p>
            <pre><code>{FORM_SNIPPET}</code></pre>

            <h3>"Parent checkbox"</h3>
            <p>"A checkbox that controls other checkboxes within a `<CheckboxGroup>` can be created:"</p>
            <ol>
                <li>"Make `<CheckboxGroup>` a controlled component"</li>
                <li>"Pass an array of all the child checkbox values to the `allValues` prop on the `<CheckboxGroup>` component"</li>
                <li>"Add the `parent` boolean prop to the parent `<Checkbox.Root>`"</li>
            </ol>
            <p>
                "The group controls the parent checkbox's "
                <a href="/react/components/checkbox#CheckboxRoot-indeterminate">"indeterminate"</a>
                " state when some, but not all, child checkboxes are checked."
            </p>
            <div class="docs-demo" data-demo="parent"><CheckboxGroupParentDemo /></div>

            <h3>"Nested parent checkbox"</h3>
            <div class="docs-demo" data-demo="nested"><CheckboxGroupNestedDemo /></div>

            <h2>"API reference"</h2>
            <h3>"CheckboxGroup"</h3>
            {api_part(
                "Provides a shared state to a series of checkboxes.",
                "Props: defaultValue (string[] — names of the checkboxes in the group that should be initially ticked; to render a controlled checkbox group, use the value prop instead), value (string[] — names of the checkboxes in the group that should be ticked; to render an uncontrolled checkbox group, use the defaultValue prop instead), onValueChange ((value: string[], eventDetails: CheckboxGroup.ChangeEventDetails) => void — event handler called when a checkbox in the group is ticked or unticked; provides the new value as an argument), allValues (string[] — names of all checkboxes in the group; use this when creating a parent checkbox), disabled (boolean, false — whether the component should ignore user interaction), className, style, render.",
                "Data attributes: data-disabled (present when the checkbox group is disabled).",
            )}
            <h3>"CheckboxGroup.Props"</h3>
            <p class="api-summary">"Re-export of CheckboxGroup props."</p>
            <h3>"CheckboxGroup.State"</h3>
            <p class="api-summary">"State: disabled (boolean — whether the component should ignore user interaction), touched (boolean — whether the field has been touched), dirty (boolean — whether the field value has changed from its initial value), valid (boolean | null — whether the field is valid), filled (boolean — whether the field has a value), focused (boolean — whether the field is focused)."</p>
            <h3>"CheckboxGroup.ChangeEventReason"</h3>
            <p class="api-summary">"type CheckboxGroupChangeEventReason = 'none'."</p>
            <h3>"CheckboxGroup.ChangeEventDetails"</h3>
            <p class="api-summary">"Details: reason ('none' — the reason for the event), event (Event — the native event associated with the custom event), cancel (() => void — cancels Base UI from handling the event), allowPropagation (() => void — allows the event to propagate in cases where Base UI will stop the propagation), isCanceled (boolean — indicates whether the event has been canceled), isPropagationAllowed (boolean — indicates whether the event is allowed to propagate), trigger (Element | undefined — the element that triggered the event, if applicable)."</p>
            <h3>"Canonical types"</h3>
            <p class="api-summary">"Maps Canonical: Alias — use Canonical when its namespace is already imported; otherwise use Alias. CheckboxGroup.State: CheckboxGroupState; CheckboxGroup.Props: CheckboxGroupProps; CheckboxGroup.ChangeEventReason: CheckboxGroupChangeEventReason; CheckboxGroup.ChangeEventDetails: CheckboxGroupChangeEventDetails."</p>
        </article>
    }
}
