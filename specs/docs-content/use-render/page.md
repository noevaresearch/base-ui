# useRender docs page content spec

Mined from `docs/src/app/(docs)/react/utils/use-render/page.mdx` only. The hook's own behavior is
covered by `specs/library/use-render/behavior.md` and is referenced here by section name instead of
being restated. Demo source files under `docs/src/app/(docs)/react/utils/use-render/demos/` are out
of scope for this file (Stage 2 mines them into `specs/docs-content/use-render/demos.json`).

## Page structure (headings, in order)

- `# useRender` (h1) — `docs/src/app/(docs)/react/utils/use-render/page.mdx:1`
- `## Examples` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:9`
- `## Merging props` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:23`
- `## Merging refs` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:52`
- `## TypeScript` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:100`
- `## Migrating from Radix UI` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:129`
- `## Render prop and polymorphism` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:166`
- `## API reference` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:172`

No `###` subheadings exist anywhere on the page — every section after the h1 is a flat `##`.
`docs/src/app/(docs)/react/utils/use-render/page.mdx:1-198`

Non-heading page furniture, in document order:

- `<Subtitle>` — "Hook for enabling a render prop in custom components." —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:3`
- `<Meta name="description">` — "Hook for enabling a render prop in custom components." (identical
  text to the Subtitle) — `docs/src/app/(docs)/react/utils/use-render/page.mdx:5`
- Intro prose (one sentence) before the first `##` heading —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:7`
- Two demo imports and renders under "Examples": `import { DemoUseRenderRender } from
  './demos/render';` + `<DemoUseRenderRender />` —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:13-15` — and `import {
  DemoUseRenderRenderCallback } from './demos/render-callback';` + `<DemoUseRenderRenderCallback />`
  — `docs/src/app/(docs)/react/utils/use-render/page.mdx:19-21` (the demo code itself is Stage 2's
  scope)
- Seven titled snippet fences interleaved with the prose (enumerated under "Code snippets embedded
  directly in the .mdx") — `docs/src/app/(docs)/react/utils/use-render/page.mdx:33-50`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:58-73`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:79-98`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:107-127`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:135-147`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:151-164`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:176-180`
- API-reference import `import { TypesUseRender } from './types';` —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:174` — and the generated reference component
  `<TypesUseRender />` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:182`
- Trailing `export const metadata` block with 11 SEO keywords ("Base UI useRender", "React Render
  Prop Hook", "Merge Props Helper", "Merge Refs Hook", "Radix asChild Migration", "Component
  Composition", "Props Spreading", "Ref Forwarding", "Custom Components", "asChild Pattern", "Slot
  Pattern") — `docs/src/app/(docs)/react/utils/use-render/page.mdx:184-198`

## Prose claims about component behavior

- Purpose claim: "The `useRender` hook lets you build custom components that provide a `render` prop
  to override the default rendered element." The override framing matches behavior.md "State model
  (controlled/uncontrolled, defaults, transitions)" (a `render` element overrides `defaultTagName`)
  and "DOM structure & portal behavior" (a `render` element replaces the fallback element entirely);
  the element/function duality implied by "render prop" matches behavior.md "Public API surface
  (props, parts, subcomponents)" (`render` is a React element or a `(props, state) => ReactElement`
  function). Consistent, no mismatch. `docs/src/app/(docs)/react/utils/use-render/page.mdx:7`
- Text-component claim: "A `render` prop for a custom Text component lets consumers use it to
  replace the default rendered `p` element with a different tag or component." Consistent with
  behavior.md "DOM structure & portal behavior" (a `render` element replaces the fallback
  entirely); the `p` default is the component's own `defaultTagName: 'p'` set in the page's
  snippets — `docs/src/app/(docs)/react/utils/use-render/page.mdx:63`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:87` — not the hook's built-in fallback, which
  behavior.md "State model (controlled/uncontrolled, defaults, transitions)" records as DIV when
  neither `defaultTagName` nor `render` is given. No mismatch, but the page never states the DIV
  fallback (see Discrepancies). `docs/src/app/(docs)/react/utils/use-render/page.mdx:11`
- Callback claim: "The callback version of the `render` prop enables more control of how props are
  spread, and also passes the internal `state` of a component." The props-spread half is consistent
  with behavior.md "Public API surface (props, parts, subcomponents)" (render functions receive
  `props` and spread it onto their output element) and "DOM structure & portal behavior" (the hook
  does not inject or overwrite `className` for render functions, so the function controls the final
  props). The `state` half: a second `state` argument is proven to exist, but behavior.md "Public
  API surface (props, parts, subcomponents)" explicitly records that its contents are never
  asserted, and "State model (controlled/uncontrolled, defaults, transitions)" establishes the hook
  is stateless with caller-supplied `state` — the "internal `state` of a component" framing is a
  component-level claim (see Discrepancies). `docs/src/app/(docs)/react/utils/use-render/page.mdx:17`
- mergeProps scope claim: "The `mergeProps` function merges two or more sets of React props
  together. It safely merges three types of props: 1. Event handlers, so that all are invoked
  2. `className` strings 3. `style` properties." These are behavior claims about a different export
  — `specs/library/use-render/behavior.md` never mentions `mergeProps`, so they cannot be
  cross-checked against it (a sibling spec `specs/library/merge-props/behavior.md` exists for that
  utility; cross-checking it is outside this file's mandate). See Discrepancies.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:25-29`
- mergeProps direction claim: "`mergeProps` merges objects from left to right, so that subsequent
  objects' properties in the arguments overwrite previous ones." Same out-of-scope situation as the
  previous bullet. `docs/src/app/(docs)/react/utils/use-render/page.mdx:31`
- mergeProps usage-guidance claim: "Merging props is useful when creating custom components, as well
  as inside the callback version of the `render` prop for any Base UI component." Usage guidance
  tying `mergeProps` to the render callback; no counterpart in behavior.md, not contradicted.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:31`
- Ref-merging claim: "you often need to control a ref internally while still letting external
  consumers pass their own—merging refs lets both parties have access to the underlying DOM element.
  The `ref` option in `useRender` enables this, which holds an array of refs to be merged together."
  The array framing is consistent with behavior.md "Public API surface (props, parts,
  subcomponents)" (`ref` accepts an array of refs; every ref in the array is attached to the
  rendered DOM element after mount) and "Edge cases (rapid interactions, unmount, nesting)"
  (multi-ref attachment: all refs end up pointing at the same rendered element).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:54`
- React 19 claim: "In React 19, `React.forwardRef()` is not needed when building primitive
  components, as the external ref prop is already contained inside `props`. Your internal ref can be
  passed to `ref` to be merged with `props.ref`." React-version guidance with no counterpart in
  behavior.md; the "merged with `props.ref`" wording asserts an automatic merging behavior the
  behavior spec never records (see Discrepancies).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:56`
- Older-React claim: "In older versions of React, you need to use `React.forwardRef()` and add the
  forwarded ref to the `ref` array along with your own internal ref." The array usage is consistent
  with behavior.md "Public API surface (props, parts, subcomponents)"; the `forwardRef` requirement
  itself is React-version guidance outside the spec's scope.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:75`
- React-19-assumption claim: "The [examples](#examples) above assume React 19, and should be
  modified to use `React.forwardRef()` to support React 18 and 17." Usage guidance; docs-only. Also
  the page's only anchor link (same-page, see Cross-links).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:77`
- TypeScript claims: "To type props, there are two interfaces: `useRender.ComponentProps` for a
  component's external (public) props. It types the `render` prop and HTML attributes.
  `useRender.ElementProps` for the element's internal (private) props. It types HTML attributes
  alone." behavior.md records only a `useRender.Parameters<{}, Element, undefined>` type as
  exercised in tests ("Public API surface (props, parts, subcomponents)"); `ComponentProps` and
  `ElementProps` are neither confirmed nor contradicted there. See Discrepancies.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:102-105`
- Radix contrast claim: "Radix UI uses an `asChild` prop, while Base UI uses a `render` prop."
  Comparative claim about an external library — outside behavior.md's scope entirely.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:131`
- Radix Slot claim: "In Radix UI, the `Slot` component lets you implement an `asChild` prop."
  External-library claim, outside behavior.md's scope.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:133`
- Equivalence claim: "In Base UI, `useRender` lets you implement a `render` prop. The example below
  is the equivalent implementation to the Radix example above." The first sentence restates the
  purpose claim (consistent with behavior.md "Public API surface (props, parts, subcomponents)");
  the Radix equivalence itself is a docs assertion with no counterpart in behavior.md.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:149`
- Render-prop design claim: "The `render` prop is primarily designed for composing event handlers
  and behavioral props. In most cases it should render the same tag as the default element."
  Design/usage guidance; behavior.md "Events (names, payload shape, bubbling, preventDefault
  semantics)" is N/A for the hook, so the event-handler-composition framing has no test counterpart
  there (generic `props` forwarding is what is proven). Not contradicted.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:168`
- Polymorphism claim: "Using `render` for polymorphism (rendering a different tag) requires more
  care, as some default props may not be valid on the new element. For example, `type=\"button\"` is
  only valid on a `<button>`. Since the component can't know what element `render` will produce at
  render time and before hydration, props like these need an explicit signal. This is why Base UI's
  [Button](/react/components/button) provides a `nativeButton` prop to control which defaults are
  applied." Rendering a different tag via `render` is proven possible (behavior.md "State model
  (controlled/uncontrolled, defaults, transitions)" and "DOM structure & portal behavior"); the
  props-validity caveat, the render-time/hydration rationale, and the `nativeButton` attribution to
  Button are all docs-only (see Discrepancies).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:170`

## API tables referenced (props/parts documented on this page)

- No literal Markdown/JSX API tables are written on the page. The `## API reference` section
  documents the single export via a generated reference component: `import { TypesUseRender } from
  './types';` — `docs/src/app/(docs)/react/utils/use-render/page.mdx:174` — preceded by a `Usage`
  fence showing only the call shape `const element = useRender({ /* Input parameters */ })` —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:176-180` — followed by `<TypesUseRender />` —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:182`.
- No prop names, prop types, defaults, or prop descriptions are written inline in the .mdx source;
  the generated table is rendered without modifier flags (no `hideDescription` or similar is
  passed). `docs/src/app/(docs)/react/utils/use-render/page.mdx:182`
- Export documented on this page: `useRender` — its package import path is shown in the migration
  snippet as `import { useRender } from '@base-ui/react/use-render';` —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:152`. `mergeProps` is exercised in two
  snippets via `import { mergeProps } from '@base-ui/react/merge-props';` —
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:34` — but has no API-reference entry on this
  page (it has its own docs page).
- No parts or subcomponents documented — matches behavior.md "Public API surface (props, parts,
  subcomponents)" ("No subcomponents or parts: the entire public surface is the single hook plus its
  parameter types"). `docs/src/app/(docs)/react/utils/use-render/page.mdx:172-182`
- Hook parameters the page actually surfaces (in prose and snippets, never in a table): `render`
  (`docs/src/app/(docs)/react/utils/use-render/page.mdx:7`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:11`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:17`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:40`), `ref`
  (`docs/src/app/(docs)/react/utils/use-render/page.mdx:54`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:56`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:65`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:90`), `defaultTagName` (snippets only:
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:63`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:87`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:120`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:156`), `props` (snippets only:
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:67`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:92`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:122`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:158`).
- The `state` input parameter and `stateAttributesMapping` are never named in the page text; `state`
  appears only as the render callback's second argument
  (`docs/src/app/(docs)/react/utils/use-render/page.mdx:17`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:40`).

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- **"Using mergeProps in the render callback"** (tsx): imports `mergeProps` from
  `@base-ui/react/merge-props` and `styles` from `./index.module.css`; a `Button` renders a
  `<Component>` element with `render={(props, state) => (<button {...mergeProps<'button'>(props, {
  className: styles.Button })} />)}`, demonstrating spreading the callback's `props` through
  `mergeProps` to add a class. Factual note: `Component` is referenced but not imported within the
  snippet (its only import lines are `mergeProps` and `styles`).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:33-50`
- **"React 19"** (tsx): a `Text({ render, ...props })` component calling `useRender` with
  `defaultTagName: 'p'`, an `internalRef` passed as the single (non-array) `ref` value alongside
  `props` and `render`; uses inline `{/* @highlight-text "internalRef" */}` JSX-comment markers to
  highlight the ref lines. `docs/src/app/(docs)/react/utils/use-render/page.mdx:58-73`
- **"React 18 and 17"** (tsx): the same `Text` component wrapped in
  `React.forwardRef(function Text({ render, ...props }: TextProps, forwardedRef:
  React.ForwardedRef<HTMLElement>) ...)`, merging via the array form `ref: [forwardedRef,
  internalRef]`; uses `// @highlight` line-comment markers.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:79-98`
- **"Typing props"** (tsx): `interface ButtonProps extends useRender.ComponentProps<'button'> {}`,
  a `defaultProps: useRender.ElementProps<'button'>` object carrying `className`, `type: 'button'`,
  and `children`, and `useRender({ defaultTagName: 'button', render, props:
  mergeProps<'button'>(defaultProps, props) })` — demonstrating the two documented interfaces plus
  `mergeProps` composition. Factual note: `mergeProps` is used but not imported within this
  snippet. `docs/src/app/(docs)/react/utils/use-render/page.mdx:107-127`
- **"Radix UI Slot component"** (jsx): `import { Slot } from 'radix-ui'`; a `Button` choosing
  `const Comp = asChild ? Slot.Root : 'button'`; usage `<Button asChild><MyButton
  className="primary">Submit</MyButton></Button>`.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:135-147`
- **"Base UI render prop"** (jsx): `import { useRender } from '@base-ui/react/use-render'`; a
  `Button` returning `useRender({ defaultTagName: 'button', render, props })`; usage `<Button
  render={<MyButton className="primary" />}>Submit</Button>` — the page's only demonstration of the
  element form of `render`. `docs/src/app/(docs)/react/utils/use-render/page.mdx:151-164`
- **"Usage"** (tsx): placeholder `const element = useRender({ /* Input parameters */ })` introducing
  the generated API reference. `docs/src/app/(docs)/react/utils/use-render/page.mdx:176-180`
- Two rendered demo components appear on the page (`<DemoUseRenderRender />` and
  `<DemoUseRenderRenderCallback />`, imported from `./demos/render` and `./demos/render-callback`);
  their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:13-15`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:19-21`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and
`specs/library/use-render/behavior.md`. Items flagged for lack of coverage, unverified-status
mismatch, or wording nuance:

- Unverified-form mismatch: the React 19 snippet passes `ref: internalRef` — a single ref, not an
  array — as the primary recommended pattern. behavior.md "Public API surface (props, parts,
  subcomponents)" records that only the array form is tested and explicitly marks the single-ref
  form UNVERIFIED. The docs present the single-ref form affirmatively; behavior.md neither confirms
  nor refutes it. `docs/src/app/(docs)/react/utils/use-render/page.mdx:58-73`
- Docs-only claim, not covered by behavior.md: the internal ref "can be passed to `ref` to be merged
  with `props.ref`" (and, per the same line, the external ref is "already contained inside
  `props`" in React 19). behavior.md proves refs listed in the `ref` array attach to the rendered
  element ("Public API surface (props, parts, subcomponents)", "Edge cases (rapid interactions,
  unmount, nesting)") and that `props` are forwarded, but records no test asserting that `props.ref`
  is automatically merged with the `ref` option. `docs/src/app/(docs)/react/utils/use-render/page.mdx:56`
- Wording nuance: the callback version "passes the internal `state` of a component." behavior.md
  "Public API surface (props, parts, subcomponents)" confirms a second `state` argument exists but
  records that its contents are never asserted, and "State model (controlled/uncontrolled, defaults,
  transitions)" establishes the hook is stateless with `state` being caller-supplied data rather
  than hook-managed state. The "internal state of a component" framing is a component-level claim
  the hook-level spec cannot corroborate (and not refuted).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:17`
- Docs-only claims, not covered by behavior.md: the two TypeScript interfaces
  `useRender.ComponentProps` (external/public props; types `render` + HTML attributes) and
  `useRender.ElementProps` (internal/private props; HTML attributes alone). behavior.md "Public API
  surface (props, parts, subcomponents)" records only `useRender.Parameters<{}, Element, undefined>`
  as exercised via tests; `ComponentProps`/`ElementProps` are neither confirmed nor contradicted.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:102-105`
- Out-of-scope section: the entire "Merging props" section makes `mergeProps` behavior claims —
  "merges two or more sets of React props together", the three safely-merged types with "Event
  handlers, so that all are invoked" first, and left-to-right rightmost-wins overwrite semantics.
  `specs/library/use-render/behavior.md` never mentions `mergeProps`, so no cross-check against it
  is possible; a sibling spec `specs/library/merge-props/behavior.md` exists for that utility
  (cross-checking it is outside this file's mandate).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:25-31`
- Docs-only rationale, not covered by behavior.md: "the component can't know what element `render`
  will produce at render time and before hydration" — behavior.md records nothing about SSR or
  hydration anywhere (its "Keyboard interactions", "Focus management", and "Events" sections are
  N/A). `docs/src/app/(docs)/react/utils/use-render/page.mdx:170`
- Docs-only component claim, outside behavior.md's scope: Button "provides a `nativeButton` prop to
  control which defaults are applied" — a component-level fact about a different unit.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:170`
- Docs-only usage guidance, no counterpart in behavior.md: "The `render` prop is primarily designed
  for composing event handlers and behavioral props. In most cases it should render the same tag as
  the default element." behavior.md proves only mechanics (element/function forms, element
  replacement); the design-intent guidance is neither confirmed nor contradicted.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:168`
- Omissions (behavior.md documents, page prose never mentions): the `state` input parameter and its
  conversion into `data-*` attributes ("Public API surface (props, parts, subcomponents)", "Edge
  cases (rapid interactions, unmount, nesting)"), `stateAttributesMapping` ("Public API surface
  (props, parts, subcomponents)", "Edge cases (rapid interactions, unmount, nesting)"), the DIV
  fallback when neither `defaultTagName` nor `render` is provided ("State model
  (controlled/uncontrolled, defaults, transitions)", "DOM structure & portal behavior" — every page
  snippet sets `defaultTagName` explicitly), `props` values beating state-derived `data-*`
  attributes ("State model (controlled/uncontrolled, defaults, transitions)"), and `defaultTagName`
  swapping the rendered element on re-render ("State model (controlled/uncontrolled, defaults,
  transitions)"). Omissions of documented behavior, not contradictions of anything the page does
  say. `docs/src/app/(docs)/react/utils/use-render/page.mdx:1-198`

## Cross-links to other docs pages

- `[composition guide](/react/handbook/composition)` — links to the composition handbook page.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:131`
- `[Button](/react/components/button)` — links to the Button component page.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:170`
- Same-page anchor, not a cross-page link: `[examples](#examples)` targets the `## Examples`
  heading on this page. `docs/src/app/(docs)/react/utils/use-render/page.mdx:77`
- No external (non-Base-UI) hyperlinks exist on the page; the Radix snippet's `import { Slot } from
  'radix-ui'` and the CSS-module import are code, not links.
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:135-147`
- Relative imports `./demos/render`, `./demos/render-callback`, and `./types` are same-page imports
  rendered on this page, not cross-page links (the demos are Stage 2's scope).
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:13`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:19`,
  `docs/src/app/(docs)/react/utils/use-render/page.mdx:174`
