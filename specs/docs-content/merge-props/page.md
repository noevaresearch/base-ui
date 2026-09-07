# mergeProps docs page content spec

Mined from `docs/src/app/(docs)/react/utils/merge-props/page.mdx` only. The utility's own
behavior is covered by `specs/library/merge-props/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/utils/merge-props/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/merge-props/demos.json`).

## Page structure (headings, in order)

- `# mergeProps` (h1) — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:1`
- `## How merging works` — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:12`
  - `### Preventing Base UI's default behavior` — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:37`
- `## Passing a function instead of an object` — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:46`
- `## API reference` — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:71`
  - `### mergeProps` — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:75`
  - `### mergePropsN` — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:82`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A utility to merge multiple sets of React props." — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:3`
- `<Meta name="description">` — "A utility to merge multiple sets of React props, handling event
  handlers, className, and style props intelligently." — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:4-7`
- Intro prose (two sentences) before the first `##` heading — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:9-10`
- Four short `ts` snippet fences interleaved with the "How merging works" bullet list — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:15-31`
- Mid-page demo import and render: `import { DemoPreventBaseUIHandler } from './demos/prevent-base-ui-handler';`
  under the "Preventing Base UI's default behavior" heading — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:39`, `docs/src/app/(docs)/react/utils/merge-props/page.mdx:44`
- One `tsx` snippet fence under "Passing a function instead of an object" — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:54-69`
- `import { TypesMergeProps, TypesMergePropsN } from './types';` at the top of the API reference
  section — `docs/src/app/(docs)/react/utils/merge-props/page.mdx:73`
- Two generated reference components with `hideDescription`: `<TypesMergeProps hideDescription />` —
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:80`, `<TypesMergePropsN hideDescription />` —
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:87`

## Prose claims about component behavior

- Purpose claim: `mergeProps` "helps you combine multiple prop objects (for example, internal props
  + user props) into a single set of props you can spread onto an element." The combine-props-objects
  framing matches behavior.md "Public API surface (props, parts, subcomponents)" (variadic props
  objects and/or props-getter functions); "spread onto an element" is usage guidance — behavior.md
  "DOM structure & portal behavior" is N/A (the unit never renders). Consistent, no mismatch.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:9`
- Analogy claim: "It behaves like `Object.assign` (rightmost wins) with a few special cases."
  Consistent with behavior.md "Events" opening precedence rule (with three arguments the LAST
  argument has highest precedence: scalars win outright, style per key, className placed first,
  handlers execute first). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:10`
- Precedence claim: "For most keys (everything except `className`, `style`, and event handlers),
  the value from the rightmost object wins," with an example titled `returns { id: 'b', dir: 'ltr' }`.
  Consistent with behavior.md "Events" precedence rule (scalar props of the last argument win
  outright). The page names the three exception categories: `className`, `style`, event handlers —
  the same four behaviors (scalars, ref, className, style, handlers) enumerated in behavior.md,
  except `ref` (see next bullet). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:14-17`
- Ref claim: "`ref` is not merged. Only the rightmost ref is kept," with an example titled
  `only refB is used`. Not covered anywhere in behavior.md — no section and no test addresses ref
  handling. Docs states it as fact; behavior.md neither confirms nor contradicts. See Discrepancies.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:18-21`
- className claim: "`className` values are concatenated right-to-left (rightmost first)," with an
  example titled `className is 'b a'` for `mergeProps({ className: 'a' }, { className: 'b' })`.
  Consistent with behavior.md "Events" precedence rule ("its `className` is placed first in the
  merged string"). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:22-25`
- style claim: "`style` objects are merged, with keys from the rightmost style overwriting earlier
  ones." Consistent with behavior.md "Events" precedence rule ("its `style` values win per key").
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:26`
- Handler-order claim: "Event handlers are merged and executed right-to-left (rightmost first),"
  with an example titled `b runs before a`. Consistent with behavior.md "Events" (execution order is
  last-argument-first; every underlying handler invoked exactly once per merged invocation).
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:27-31`
- Synthetic-event prevention claim: "For React synthetic events, Base UI adds
  `event.preventBaseUIHandler()`. Calling it prevents Base UI's internal logic from running."
  Consistent with behavior.md "Events" prevention semantics (the event argument exposes
  `preventBaseUIHandler()`; once called, every handler merged earlier in the argument list is
  skipped). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:33`
- Non-interference claim: prevention "does not call `preventDefault()` or `stopPropagation()`."
  Not covered by behavior.md — "Events" explicitly marks the relationship to native
  `event.preventDefault()` as UNVERIFIED (no test asserts it). Docs states it affirmatively. See
  Discrepancies. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:34`
- Non-synthetic-events claim: "For non-synthetic events (custom events with primitive/object
  values), this mechanism isn't available and all handlers always execute." Not covered by
  behavior.md — "Events" marks whether `preventBaseUIHandler()` applies to non-standard handlers
  as untested, and the synthetic-vs-non-standard classification criterion as unasserted. Docs
  asserts both. See Discrepancies. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:35`
- render-prop claim: "When using the function form of the `render` prop, props are not merged
  automatically. You can use `mergeProps` to combine Base UI's props with your own, and call
  `preventBaseUIHandler()` to stop Base UI's internal logic from running." The first half concerns
  component `render`-prop behavior, which is outside behavior.md's scope entirely (pure-utility
  spec, no component); the second half is consistent with behavior.md "Events" prevention semantics.
  See Discrepancies. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:41-42`
- Props-getter claim: "Each argument can be a props object or a function that receives the merged
  props up to that point (left to right) and returns a props object," and "This is useful when you
  need to compute the next props from whatever has already been merged." Broadly consistent with
  behavior.md "Public API surface (props, parts, subcomponents)" props-getter description (called
  exactly once per invocation, receives the accumulated merge of all props-object arguments that
  PRECEDE it); a wording nuance is flagged under Discrepancies.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:48-49`
- Getter-replace claim: "the function's return value completely replaces the accumulated props up
  to that point." Consistent with behavior.md "Public API surface (props, parts, subcomponents)"
  ("its return value REPLACES that accumulated object") and "Edge cases (rapid interactions,
  unmount, nesting)" (a getter positioned last replaces the accumulated props; earlier props do
  not survive). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:51`
- Manual-chaining claim: "If you want to chain event handlers from the previous props, you must
  call them manually." Consistent with behavior.md "Edge cases (rapid interactions, unmount,
  nesting)" (a getter can capture the props accumulated before it, including handlers, and return
  new handlers that call the captured ones; manual re-invocation also bypasses automatic
  prevention). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:52`
- Arity claim: "`mergeProps` accepts up to 5 arguments, each being either a props object or a
  function that returns a props object. If you need to merge more than 5 sets of props, use
  `mergePropsN` instead." behavior.md "Public API surface (props, parts, subcomponents)" describes
  `mergeProps` as variadic with no arity cap and records no 5-argument limit — the cap is a
  docs-only claim. See Discrepancies. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:77-78`
- `mergePropsN` claims: "accepts an array of props objects or functions that return props objects"
  — consistent with behavior.md "Public API surface (props, parts, subcomponents)" (`mergePropsN`:
  same merging applied to a single array argument; proven equivalent for the lone-handler
  prevention behavior). "It is slightly less efficient than `mergeProps`, so only use it when you
  need to merge more than 5 sets of props" — a performance/usage-guidance claim with no
  counterpart in behavior.md (no perf assertions exist). See Discrepancies.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:84-85`

## API tables referenced (props/parts documented on this page)

- No literal Markdown API tables exist on the page. The `## API reference` section documents the
  two exports via generated reference components: `### mergeProps` → `<TypesMergeProps
  hideDescription />` and `### mergePropsN` → `<TypesMergePropsN hideDescription />`.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:75-87`
- The reference tables are imported from `./types` (`import { TypesMergeProps,
  TypesMergePropsN } from './types';`); no prop names, prop types, defaults, or prop descriptions
  are written inline in the .mdx source of this page. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:73`
- Both reference components receive the `hideDescription` flag, meaning the descriptive prose for
  each export is supplied by the page text itself (lines 77-78 for `mergeProps`, lines 84-85 for
  `mergePropsN`) rather than by the generated tables. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:80`, `docs/src/app/(docs)/react/utils/merge-props/page.mdx:87`
- Exports documented on this page: `mergeProps`, `mergePropsN` — the same two exports recorded in
  behavior.md "Public API surface (props, parts, subcomponents)". There is no "Additional types"
  section and no parts/subcomponents (the unit is headless with no parts).
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:71-87`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- **"returns { id: 'b', dir: 'ltr' }"** (ts): `mergeProps({ id: 'a', dir: 'ltr' }, { id: 'b' });`
  — scalar rightmost-wins example; title asserts the exact expected output.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:15-17`
- **"only refB is used"** (ts): `mergeProps({ ref: refA }, { ref: refB });` — ref non-merge
  example; title asserts only the rightmost ref survives.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:19-21`
- **"className is 'b a'"** (ts): `mergeProps({ className: 'a' }, { className: 'b' });` — className
  concatenation-order example; title asserts the exact expected string.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:23-25`
- **"b runs before a"** (ts): `mergeProps({ onClick: a }, { onClick: b });` — handler
  execution-order example. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:29-31`
- **"Manually chaining handlers in a function"** (tsx): a two-argument `mergeProps` whose second
  argument is a getter `(props) => ({ onClick(event) { props.onClick?.(event); ... } })`
  demonstrating the manual previous-handler invocation required when a getter replaces the
  accumulated props. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:54-69`
- One rendered demo component appears on the page (`<DemoPreventBaseUIHandler />`, imported from
  `./demos/prevent-base-ui-handler`); its code lives in demo files and is out of scope here
  (Stage 2). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:39`, `docs/src/app/(docs)/react/utils/merge-props/page.mdx:44`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and
`specs/library/merge-props/behavior.md`. Items flagged for lack of coverage, omission, or
wording nuance:

- Docs-only claim, not covered by behavior.md: `ref` is never merged and only the rightmost ref is
  kept (prose line 18, snippet lines 19-21). behavior.md has no section or test record addressing
  ref handling at all — "Public API surface (props, parts, subcomponents)", "Events", and
  "Edge cases (rapid interactions, unmount, nesting)" never mention `ref`. The docs assert it as
  fact; behavior.md neither confirms nor contradicts. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:18-21`
- Docs-only claim, not covered by behavior.md: calling `preventBaseUIHandler()` "does not call
  `preventDefault()` or `stopPropagation()`." behavior.md "Events" explicitly marks the
  relationship to native `event.preventDefault()` / `nativeEvent` as UNVERIFIED (no test asserts
  it). The docs state affirmatively what the behavior spec leaves unverified.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:34`
- Docs-only claim, not covered by behavior.md: for non-synthetic events (custom events with
  primitive/object values) "this mechanism isn't available and all handlers always execute."
  behavior.md "Events" marks whether `preventBaseUIHandler()` applies to non-standard handlers as
  untested (no test calls it on one) and marks the criterion that classifies a key as "synthetic"
  vs "non-standard" as unasserted. The docs assert both the unavailability and the always-execute
  outcome. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:35`
- Docs-only claim, outside behavior.md's scope: "When using the function form of the `render`
  prop, props are not merged automatically." behavior.md is scoped to the headless utility and
  records nothing about component `render` props; no test corroborates this.
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:41`
- Docs-only claim, not covered by behavior.md: `mergeProps` "accepts up to 5 arguments" and
  `mergePropsN` should be used "when you need to merge more than 5 sets of props." behavior.md
  "Public API surface (props, parts, subcomponents)" describes `mergeProps` as variadic with no
  arity cap; no test asserts a limit. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:77-78`
- Docs-only claim, not covered by behavior.md: `mergePropsN` "is slightly less efficient than
  `mergeProps`." behavior.md records no performance assertions (out of a behavior spec's scope).
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:85`
- Wording nuance (not a hard contradiction): the docs say a getter function "receives the merged
  props up to that point (left to right)", while behavior.md "Public API surface (props, parts,
  subcomponents)" words the same input as "the accumulated merge of all props-object arguments
  that PRECEDE it in the list". The phrasings differ on whether a preceding getter's return value
  feeds the next getter's input; behavior.md "Edge cases (rapid interactions, unmount, nesting)"
  only test-asserts a getter with no preceding props-object arguments receiving an empty object.
  Neither source covers the consecutive-getters case, so the docs' broader phrasing is
  uncorroborated (and not refuted). `docs/src/app/(docs)/react/utils/merge-props/page.mdx:48`
- Omissions (behavior.md documents, page prose never mentions): `undefined` handler slots being
  skipped without breaking the chain ("Events"), non-standard handler invocation arguments being
  forwarded verbatim to every chained handler ("Events"), source objects not being mutated by the
  merge ("Public API surface (props, parts, subcomponents)"), and the `baseUIHandlerPrevented`
  flag a getter can read to opt into respecting prevention ("Edge cases (rapid interactions,
  unmount, nesting)"). Omissions of documented behavior, not contradictions of anything the page
  does say. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:9-87`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages and no external
  links. `docs/src/app/(docs)/react/utils/merge-props/page.mdx:1-87`
- The only non-framework imports are same-page relative references: the demo
  (`./demos/prevent-base-ui-handler`) and the generated API reference (`./types`). They are
  same-page imports rendered on this page, not cross-page links (the demo is Stage 2's scope).
  `docs/src/app/(docs)/react/utils/merge-props/page.mdx:39`, `docs/src/app/(docs)/react/utils/merge-props/page.mdx:73`
