# React mentions — source scan

Generated 2026-09-16T08:46:13.264Z by check-react-mentions.mjs --source.

Scope: the port's own reader-facing source — `crates/docs-app/src/**/*.rs`, test files excluded. Mirror analyses (`specs/docs-content/*/page.md`, `specs/library/**`) are deliberately NOT scanned: they document upstream React by design.

The package this port points readers at must be `@noevaresearch/base-ui`; React APIs in prose/snippets are defects.

## crates/docs-app/src/pages/accordion_reference.rs

- **react-api** L167: short_ty: "React.CSSProperties | function",
- **react-api** L168: ty: "| React.CSSProperties\n| ((\n    state: Accordion.Root.State<Value>,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L192: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Accordion.Root.State<Value>,\n  ) => ReactElement)\n| undefined",
- **react-api** L302: short_ty: "React.CSSProperties | function",
- **react-api** L303: ty: "| React.CSSProperties\n| ((\n    state: Accordion.Item.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L313: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Accordion.Item.State,\n  ) => ReactElement)\n| undefined",
- **react-api** L394: short_ty: "React.CSSProperties | function",
- **react-api** L395: ty: "| React.CSSProperties\n| ((\n    state: Accordion.Header.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L405: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Accordion.Header.State,\n  ) => ReactElement)\n| undefined",
- **react-api** L494: short_ty: "React.CSSProperties | function",
- **react-api** L495: ty: "| React.CSSProperties\n| ((\n    state: Accordion.Trigger.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L505: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Accordion.Trigger.State,\n  ) => ReactElement)\n| undefined",
- **react-api** L586: short_ty: "React.CSSProperties | function",
- **react-api** L587: ty: "| React.CSSProperties\n| ((\n    state: Accordion.Panel.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L611: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Accordion.Panel.State,\n  ) => ReactElement)\n| undefined",

## crates/docs-app/src/pages/checkbox_page.rs

- **react-api** L495: short_ty: "React.Ref<HTMLInputElement>",
- **react-api** L496: ty: "React.Ref<HTMLInputElement> | undefined",
- **react-api** L525: short_ty: "React.CSSProperties | function",
- **react-api** L526: ty: "| React.CSSProperties\n| ((\n    state: Checkbox.Root.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L536: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Checkbox.Root.State,\n  ) => ReactElement)\n| undefined",
- **react-api** L636: short_ty: "React.CSSProperties | function",
- **react-api** L637: ty: "| React.CSSProperties\n| ((\n    state: Checkbox.Indicator.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L657: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Checkbox.Indicator.State,\n  ) => ReactElement)\n| undefined",
- **package-react** L872: const UPSTREAM_ANATOMY: &str = "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root>\n  <Checkbox.Indicator />\n</Checkbox.Root>;";
- **react-api** L1214: "React.Ref<HTMLInputElement>",
- **react-api** L1217: "React.CSSProperties | function",
- **react-api** L1229: "React.CSSProperties | function",
- **react-word** L878: "the classifier no longer recognises upstream's React source — the assertions below \
- **react-word** L892: let (mut leptos, mut react, mut other) = (0, 0, 0);
- **react-word** L897: react += 1;
- **react-word** L898: panic!("the '{name}' snippet still carries React source");
- **react-word** L902: panic!("the '{name}' snippet identifies as neither port nor React source");
- **react-word** L907: (leptos, react, other),
- **react-word** L909: "the probe must read {{total: 5, leptos: 5, react: 0}} for this page"

## crates/docs-app/src/pages/button_page.rs

- **react-api** L437: short_ty: "React.CSSProperties | function",
- **react-api** L438: ty: "| React.CSSProperties\n| ((\n    state: Button.State,\n  ) => React.CSSProperties | undefined)\n| undefined",
- **react-api** L448: ty: "| ReactElement\n| ((\n    props: HTMLProps,\n    state: Button.State,\n  ) => ReactElement)\n| undefined",
- **package-react** L567: const UPSTREAM_ANATOMY: &str = "import { Button } from '@base-ui/react/button';\n\n<Button />;";
- **package-react** L571: const UPSTREAM_CUSTOM_TAG: &str = "import { Button } from '@base-ui/react/button';\n\n// @highlight-text \"nativeButton={false}\"\n<Button render={<div />} nativeButton={false}>\n  Button that can con
- **react-api** L677: "React.CSSProperties | function",
- **react-word** L577: "the classifier no longer recognises upstream's React source — the assertions below \
- **react-word** L582: "the classifier no longer recognises upstream's React source — the assertions below \
- **react-word** L593: let (mut leptos, mut react, mut other) = (0, 0, 0);
- **react-word** L598: react += 1;
- **react-word** L599: panic!("the '{name}' snippet still carries React source");
- **react-word** L603: panic!("the '{name}' snippet identifies as neither port nor React source");
- **react-word** L608: (leptos, react, other),
- **react-word** L610: "the probe must read {{total: 2, leptos: 2, react: 0}} for this page"

## crates/docs-app/src/code_block.rs

- **package-react** L842: "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root>\n  <Checkbox.Indicator />\n</Checkbox.Root>;",
- **package-react** L883: let code = "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root id=\"x\" render={<button />}>\n  <label>Text</label>\n</Checkbox.Root>;";
- **package-react** L889: assert_eq!(class_of(&lines, "'@base-ui/react/checkbox'"), Some("pl-s"));
- **package-react** L933: "import { Checkbox } from '@base-ui/react/checkbox';\n\n<Checkbox.Root id=\"x\">\n  <label>Text</label>\n</Checkbox.Root>;",

## crates/docs-app/src/pages/avatar_page.rs

- **package-react** L85: const ANATOMY_SNIPPET: &str = r#"import { Avatar } from '@base-ui/react/avatar';
- **react-api** L296: "Props: className (string | ((state: Avatar.Root.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSP
- **react-api** L303: "Props: onLoadingStatusChange (((status: ImageLoadingStatus) => void) — callback fired when the loading status changes), className (string | ((state: Avatar.Image.State) => string | undefined)), style
- **react-api** L310: "Props: delay (number, 0 — how long to wait before showing the fallback, specified in milliseconds), className (string | ((state: Avatar.Fallback.State) => string | undefined)), style (React.CSSProper

## crates/docs-app/src/pages/field_page.rs

- **package-react** L134: "import { Field } from '@base-ui/react/field';
- **react-api** L150: "Props: name (string — identifies the field when a form is submitted; takes precedence over the name prop on <Field.Control>), actionsRef (React.RefObject<Field.Root.Actions | null> — a ref to imperat
- **react-api** L186: "Props: children ((state: Field.Validity.State) => React.ReactNode, required — a function that accepts the field validity state as an argument; the state carries validity, value, error, errors, initia

## crates/docs-app/src/pages/fieldset_page.rs

- **react-api** L146: "Props: className (string | ((state: Fieldset.Root.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CS
- **react-api** L160: "Props: className (string | ((state: Fieldset.Legend.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.
- **package-react** L184: "import { Fieldset } from '@base-ui/react/fieldset';

## crates/docs-app/src/pages/form_page.rs

- **package-react** L168: const ANATOMY_SNIPPET: &str = "import { Field } from '@base-ui/react/field';\nimport { Form } from '@base-ui/react/form';\n\n<Form>\n  <Field.Root>\n    <Field.Label />\n    <Field.Control />\n    <Fi
- **react-api** L656: "Props: errors (Errors — validation errors returned externally, typically after submission by a server or a form action; this should be an object where keys correspond to the name attribute on <Field.
- **package-react** L749: <a href="https://react.dev/reference/react-dom/components/form#handle-form-submission-with-a-server-function">

## crates/docs-app/src/chrome.rs

- **package-react** L150: href: "https://www.npmjs.com/package/@base-ui/react",
- **package-react** L183: href="https://www.npmjs.com/package/@base-ui/react"

## crates/docs-app/src/pages/csp_provider_page.rs

- **package-react** L130: "import { CSPProvider } from '@base-ui/react/csp-provider';
- **package-react** L167: "import { CSPProvider } from '@base-ui/react/csp-provider';

## crates/docs-app/src/pages/direction_provider_page.rs

- **package-react** L169: "import { DirectionProvider } from '@base-ui/react/direction-provider';
- **react-api** L188: "(`React.ReactNode`). Additional types: `type TextDirection = 'ltr' | 'rtl'`."

## crates/docs-app/src/pages/meter_page.rs

- **package-react** L109: "import { Meter } from '@base-ui/react/meter';
- **react-api** L139: "Props: children ((formattedValue: string, value: number) => React.ReactNode | null — the render-function form; omission renders the formatted value), className, style, render.",

## crates/docs-app/src/pages/progress_page.rs

- **package-react** L265: "import { Progress } from '@base-ui/react/progress';
- **react-api** L298: "Props: children ((formattedValue: string | null, value: number | null) => React.ReactNode | null — the render-function form; omission renders the formatted value), className, style, render.",

## crates/docs-app/src/lib.rs

- **package-react** L132: "import { Collapsible } from '@base-ui/react/collapsible';
- **react-word** L49: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("accordion"))
- **react-word** L53: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("button"))
- **react-word** L56: <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("avatar")) view=AvatarPage />
- **react-word** L58: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("checkbox"))
- **react-word** L62: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("checkbox-group"))
- **react-word** L66: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("collapsible"))
- **react-word** L69: <Route path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("use-render")) view=UseRenderPage />
- **react-word** L71: path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("csp-provider"))
- **react-word** L74: <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("toggle")) view=TogglePage />
- **react-word** L76: path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("direction-provider"))
- **react-word** L80: path=(StaticSegment("react"), StaticSegment("utils"), StaticSegment("merge-props"))
- **react-word** L84: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("separator"))
- **react-word** L87: <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("meter")) view=MeterPage />
- **react-word** L88: <Route path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("field")) view=FieldPage />
- **react-word** L90: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("fieldset"))
- **react-word** L94: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("otp-field"))
- **react-word** L98: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("progress"))
- **react-word** L102: path=(StaticSegment("react"), StaticSegment("components"), StaticSegment("form"))

## crates/docs-app/src/pages/accordion_page.rs

- **package-react** L380: const UPSTREAM_ANATOMY: &str = "import { Accordion } from '@base-ui/react/accordion';\n\n<Accordion.Root>\n  <Accordion.Item>\n    <Accordion.Header>\n      <Accordion.Trigger />\n    </Accordion.Head
- **react-word** L178: "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
- **react-word** L201: "Base UI is a library of high-quality unstyled React components for design systems and web apps.",
- **react-word** L386: "the classifier no longer recognises upstream's React source — the assertions below \
- **react-word** L391: SnippetLanguage::React,
- **react-word** L392: "upstream's source must classify as React"
- **react-word** L403: let (mut leptos, mut react, mut other) = (0, 0, 0);
- **react-word** L407: SnippetLanguage::React => {
- **react-word** L408: react += 1;
- **react-word** L409: panic!("the '{name}' snippet still carries React source");
- **react-word** L413: panic!("the '{name}' snippet identifies as neither port nor React source");
- **react-word** L418: (leptos, react, other),
- **react-word** L420: "the probe must read {{total: 1, leptos: 1, react: 0}} for this page"

## crates/docs-app/src/pages/checkbox_group_page.rs

- **package-react** L153: const ANATOMY_SNIPPET: &str = "import { Checkbox } from '@base-ui/react/checkbox';\nimport { CheckboxGroup } from '@base-ui/react/checkbox-group';\n\n<CheckboxGroup>\n  <Checkbox.Root />\n</CheckboxGr

## crates/docs-app/src/pages/otp_field_page.rs

- **package-react** L188: const ANATOMY_SNIPPET: &str = "import { OTPField } from '@base-ui/react/otp-field';\n\n<OTPField.Root>\n  <OTPField.Input />\n  <OTPField.Separator />\n</OTPField.Root>;";
- **react-word** L899: "filled, or use `onValueComplete` to react to completion without submitting."

## crates/docs-app/src/pages/separator_page.rs

- **package-react** L136: "import { Separator } from '@base-ui/react/separator';

## crates/docs-app/src/pages/toggle_page.rs

- **package-react** L197: "import { Toggle } from '@base-ui/react/toggle';

## crates/docs-app/src/pages/use_render_page.rs

- **react-api** L163: let children = props.children;
- **react-word** L351: "The `mergeProps` function merges two or more sets of React props together, "

## crates/docs-app/src/snippet_language.rs

- **package-react** L70: has("@base-ui/react")
- **react-word** L134: React,
- **react-word** L147: SnippetLanguage::React

## crates/docs-app/src/install_ref.rs

- **react-word** L28: pub const PROVENANCE: &str = "Ported from the React implementation of Base UI — the same behaviour and anatomy, expressed with Leptos signals and view! markup.";

## crates/docs-app/src/pages/merge_props_page.rs

- **react-word** L261: <p class="subtitle">"A utility to merge multiple sets of React props."</p>
- **react-word** L267: "common React patterns work as expected."
- **react-word** L294: "For React synthetic events, Base UI adds "
