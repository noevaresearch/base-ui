# Form docs page content spec

Mined from `docs/src/app/(docs)/react/components/form/page.mdx` only. The component's own
behavior is covered by `specs/library/form/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/form/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/form/demos.json`).
The page's generated API-reference backing files (`types.ts`/`types.md`, same directory) are
summarized under "API tables referenced" because the .mdx delegates its API tables to them.

## Page structure (headings, in order)

- `# Form` (h1) — `docs/src/app/(docs)/react/components/form/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/form/page.mdx:14`
- `## Examples` — `docs/src/app/(docs)/react/components/form/page.mdx:31`
  - `### Submit with a Server Function` — `docs/src/app/(docs)/react/components/form/page.mdx:33`
  - `### Submit form values as a JavaScript object` — `docs/src/app/(docs)/react/components/form/page.mdx:41`
  - `### Using with Zod` — `docs/src/app/(docs)/react/components/form/page.mdx:64`
- `## API reference` — `docs/src/app/(docs)/react/components/form/page.mdx:72`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A native form element with consolidated error handling." — `docs/src/app/(docs)/react/components/form/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React form component with consolidated error handling." — `docs/src/app/(docs)/react/components/form/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/form/page.mdx:10-12`
- Demo imports interleaved with the Examples subsections (`./demos/form-action` at `docs/src/app/(docs)/react/components/form/page.mdx:37`, `./demos/zod` at `docs/src/app/(docs)/react/components/form/page.mdx:68`)
- `TypesForm` import for the API reference — `docs/src/app/(docs)/react/components/form/page.mdx:74`
- Trailing `export const metadata` SEO keywords block (10 keywords, e.g. 'React Form Component', 'Form Error Handling') — `docs/src/app/(docs)/react/components/form/page.mdx:78-91`

## Prose claims about component behavior

- Page describes the component as "A native form element with consolidated error handling." and,
  in the meta description, as "high-quality, unstyled". The "native form element" claim matches
  behavior.md "Public API surface (props, parts, subcomponents)" (Form renders a native
  `<form>` element); "consolidated error handling" is a naming-level characterization consistent
  with the externally-owned `errors` record in behavior.md "State model (controlled/uncontrolled,
  defaults, transitions)", though no test asserts a concept literally called "consolidated".
  `docs/src/app/(docs)/react/components/form/page.mdx:3`, `docs/src/app/(docs)/react/components/form/page.mdx:5-8`
- Anatomy usage guidance: "Form is composed together with Field", linking to the Field docs page,
  with instruction to "Import the components and place them together". Matches behavior.md
  "Public API surface (props, parts, subcomponents)": Form has no subcomponents of its own and
  composes with Field parts as registered value/validation participants.
  `docs/src/app/(docs)/react/components/form/page.mdx:16`
- Server Function usage guidance: "Forms using `useActionState` can be submitted with a
  Server Function instead of `onSubmit`." This is React platform integration guidance; behavior.md
  "Events (names, payload shape, bubbling, preventDefault semantics)" documents only the native
  `submit` event received via `onSubmit` and does not cover `action`/`useActionState`/Server
  Functions — uncovered, not contradicted (see Discrepancies).
  `docs/src/app/(docs)/react/components/form/page.mdx:35`
- `onFormSubmit` claim: "You can use `onFormSubmit` instead of the native `onSubmit` to access
  form values as a JavaScript object. This is useful when you need to transform the values before
  submission, or integrate with 3rd party APIs." Matches behavior.md "Events (names, payload
  shape, bubbling, preventDefault semantics)": `onFormSubmit` receives `(formValues,
  eventDetails)` where `formValues` is a plain record keyed by field name, and behavior.md
  "State model (controlled/uncontrolled, defaults, transitions)" records values aligned with
  native submission semantics. The page does not state that `onFormSubmit` fires only on
  successful submit and not at all when the form is invalid (behavior.md records this); see
  Discrepancies.
  `docs/src/app/(docs)/react/components/form/page.mdx:43`
- "When used, `preventDefault` is called on the native submit event." Matches behavior.md
  "Events (names, payload shape, bubbling, preventDefault semantics)": the Form handles/prevents
  the native default itself, and `eventDetails.event.defaultPrevented` is `true`.
  `docs/src/app/(docs)/react/components/form/page.mdx:62`
- Zod integration guidance: "When parsing the schema using `schema.safeParse()`, the
  `z.flattenError(result.error).fieldErrors` data can be used to map the errors to each field's
  `name`." The map-errors-by-field-name convention is consistent with behavior.md "State model
  (controlled/uncontrolled, defaults, transitions)" (the `errors` prop is a record keyed by
  field name, and fields without an entry are not marked invalid). The Zod API specifics
  (`safeParse`, `z.flattenError`) are third-party usage guidance with no counterpart in
  behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/form/page.mdx:66`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section renders a single generated component `<TypesForm />`; unlike
  multi-part components, there are no per-part headings and no inline prop tables in the .mdx
  source of this page.
  `docs/src/app/(docs)/react/components/form/page.mdx:72-76`
- `TypesForm` is imported from `./types`, where it is created as
  `createTypes(import.meta.url, Form)` — i.e. the reference is generated from the `Form`
  component itself.
  `docs/src/app/(docs)/react/components/form/page.mdx:74`, `docs/src/app/(docs)/react/components/form/types.ts:4`
- The autogenerated backing markdown (`types.md`) documents exactly one part, `Form` ("A native
  form element with consolidated error handling. Renders a `<form>` element."), with a props
  table listing: `errors` (externally returned validation errors keyed by `<Field.Root>` `name`),
  `actionsRef` (imperative `validate` actions), `onFormSubmit` (with "preventDefault() is called
  on the native submit event when used"), `validationMode` (default `'onSubmit'`; `onBlur` /
  `onChange` modes; a `<Field.Root>` `validationMode` takes precedence), plus the standard
  `className`, `style`, and `render` props.
  `docs/src/app/(docs)/react/components/form/types.md:5-22`
- The backing markdown also documents an `actionsRef` usage example (`validate()` for all
  fields, `validate('email')` for one) and type sections `Form.Props`, `Form.State`,
  `Form.Actions`, `Form.SubmitEventDetails`, `Form.SubmitEventReason`, `Form.ValidationMode`,
  `Form.Values`, plus a Canonical Types alias list.
  `docs/src/app/(docs)/react/components/form/types.md:24-32`, `docs/src/app/(docs)/react/components/form/types.md:34-77`, `docs/src/app/(docs)/react/components/form/types.md:79-88`
- Cross-check: the single-part set (Form only, no sub-parts) matches behavior.md "Public API
  surface (props, parts, subcomponents)". The documented `actionsRef.validate(name)` /
  `validate()` signature matches behavior.md "Focus management"; the `validationMode` modes and
  Field-level precedence match behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)"; the `errors`-keyed-by-name description matches the same section. No mismatch.

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Two fenced code blocks:
  1. Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ Field }` from
     `@base-ui/react/field` and `{ Form }` from `@base-ui/react/form`, assembling `<Form>`
     wrapping `Field.Root` containing `Field.Label`, `Field.Control`, and `Field.Error`.
     `docs/src/app/(docs)/react/components/form/page.mdx:18-29`
  2. "Submission using onFormSubmit" (` ```tsx title="Submission using onFormSubmit" `):
     `<Form onFormSubmit={async (formValues: { id: string; quantity: number }) => ...}>`
     transforming values into a `{ product_id, order_quantity }` payload and POSTing it as JSON
     via `fetch` to `https://api.example.com`.
     `docs/src/app/(docs)/react/components/form/page.mdx:45-60`
- No other code blocks exist in the page. The Server Function and Zod examples render demo
  components (`<DemoFormAction />`, `<DemoFormZod />`, plus `<DemoFormHero />` at the top)
  imported from `./demos/*`; their code lives in demo files and is out of scope here (Stage 2).
  `docs/src/app/(docs)/react/components/form/page.mdx:10-12`, `docs/src/app/(docs)/react/components/form/page.mdx:37-39`, `docs/src/app/(docs)/react/components/form/page.mdx:68-70`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/form/behavior.md`.
Items flagged for lack of coverage or omission:

- Omission: behavior.md "Events (names, payload shape, bubbling, preventDefault semantics)"
  records that `onFormSubmit` fires once per successful submit and not at all when the form is
  invalid (submission is gated on all registered fields being valid). The page presents
  `onFormSubmit` as a drop-in "instead of the native `onSubmit`" but never mentions this
  invalid-submit gating difference. Omission of a developer-facing caveat, not a contradiction.
  `docs/src/app/(docs)/react/components/form/page.mdx:43`
- Docs-only claim, not covered by behavior.md: `useActionState`/Server Function submission as an
  alternative to `onSubmit`. behavior.md "Events (names, payload shape, bubbling,
  preventDefault semantics)" covers only the native `submit` event via `onSubmit`; no test
  exercises React action/Server Function submission.
  `docs/src/app/(docs)/react/components/form/page.mdx:35`
- Docs-only claim, not covered by behavior.md: Zod integration guidance (`schema.safeParse()`,
  `z.flattenError(result.error).fieldErrors` mapped to field `name`s). behavior.md covers the
  external `errors` record keyed by field name ("State model (controlled/uncontrolled, defaults,
  transitions)") but no test involves Zod.
  `docs/src/app/(docs)/react/components/form/page.mdx:66`
- Omission (page prose vs. the page's own API reference): the generated reference documents
  `validationMode` (default `'onSubmit'`, `onBlur`/`onChange`, Field-root precedence) and
  `actionsRef.validate()`, but the page prose never mentions either; behavior.md proves both
  ("State model (controlled/uncontrolled, defaults, transitions)" for `validationMode`, "Focus
  management" for `actionsRef`). A prose-vs-reference completeness gap, not a contradiction.
  `docs/src/app/(docs)/react/components/form/types.md:19`, `docs/src/app/(docs)/react/components/form/types.md:24-32`
- Note (not a mismatch): the page's "consolidated error handling" phrasing appears only at the
  naming level in prose; behavior.md grounds error handling in the `errors` prop, per-field
  `validate`, and native constraint validation ("State model (controlled/uncontrolled, defaults,
  transitions)") without using the word "consolidated".
  `docs/src/app/(docs)/react/components/form/page.mdx:3`

## Cross-links to other docs pages

- Internal link: `[Field](/react/components/field)` in the Anatomy intro, pointing to the Field
  component docs page — the only internal docs cross-link on the page.
  `docs/src/app/(docs)/react/components/form/page.mdx:16`
- One external link: React documentation for handling form submission with a Server Function
  (`https://react.dev/reference/react-dom/components/form#handle-form-submission-with-a-server-function`),
  used to explain the `useActionState` example.
  `docs/src/app/(docs)/react/components/form/page.mdx:35`
- Demo components (`./demos/hero`, `./demos/form-action`, `./demos/zod`) and the `TypesForm`
  reference (`./types`) are same-page imports rendered on this page itself; they are same-page
  imports, not cross-page links.
  `docs/src/app/(docs)/react/components/form/page.mdx:10`, `docs/src/app/(docs)/react/components/form/page.mdx:37`, `docs/src/app/(docs)/react/components/form/page.mdx:68`, `docs/src/app/(docs)/react/components/form/page.mdx:74`
