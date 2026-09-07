# OTP Field docs page content spec

Mined from `docs/src/app/(docs)/react/components/otp-field/page.mdx` only. The component's own
behavior is covered by `specs/library/otp-field/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/otp-field/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/otp-field/demos.json`).

## Page structure (headings, in order)

- `# OTP Field` (h1) — `docs/src/app/(docs)/react/components/otp-field/page.mdx:1`
- `## Usage guidelines` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:14`
- `## Anatomy` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:18`
- `## Examples` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:31`
  - `### Labeling an OTP field` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:33`
  - `### Form integration` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:56`
  - `### Alphanumeric verification codes` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:80`
  - `### Grouped layouts` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:89`
  - `### Placeholder hints` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:98`
  - `### Custom normalization` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:107`
  - `### Masked entry` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:120`
- `## API reference` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:128`
  - `### Root` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:132`
  - `### Input` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:136`
  - `### Separator` — `docs/src/app/(docs)/react/components/otp-field/page.mdx:140`

Non-heading page furniture, in document order:

- `<Subtitle>` — "A one-time password input composed of individual character slots." — `docs/src/app/(docs)/react/components/otp-field/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React OTP field component for one-time password and verification code entry." — `docs/src/app/(docs)/react/components/otp-field/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/otp-field/page.mdx:10-12`
- Demo imports interleaved with the Examples subsections (`./demos/alphanumeric` at `docs/src/app/(docs)/react/components/otp-field/page.mdx:85`, `./demos/grouped` at `docs/src/app/(docs)/react/components/otp-field/page.mdx:94`, `./demos/focused-placeholder` at `docs/src/app/(docs)/react/components/otp-field/page.mdx:103`, `./demos/custom-sanitize` at `docs/src/app/(docs)/react/components/otp-field/page.mdx:116`, `./demos/password` at `docs/src/app/(docs)/react/components/otp-field/page.mdx:124`)
- `TypesOTPField` import for the API reference — `docs/src/app/(docs)/react/components/otp-field/page.mdx:130`
- Trailing `export const metadata` SEO keywords block (12 keywords, e.g. 'React OTP Field', 'One-Time Password Input', 'Pin Input', '2FA Input') — `docs/src/app/(docs)/react/components/otp-field/page.mdx:144-159`

## Prose claims about component behavior

- Page describes the component as "A one-time password input composed of individual character
  slots." and, in the meta description, as "unstyled". Naming-level claim only; consistent with
  the three-part API (Root/Input/Separator) in behavior.md "Public API surface (props, parts,
  subcomponents)".
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:3`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:5-8`
- Usage guideline: "Form controls must have an accessible name", creatable "using a `<label>`
  element or the `Field` component". Matches behavior.md "Accessibility (roles, aria-*, id
  linking)": a native `<label htmlFor={rootId}>` gives every slot the same accessible name, and
  `Field.Label` associates with the first slot.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:16`
- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `OTPField.Root` containing `OTPField.Input` and `OTPField.Separator`, imported from the
  `@base-ui/react/otp-field` namespace. The part set and namespace import match behavior.md
  "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:20-29`
- Labeling guidance: "Pass an `id` to `<OTPField.Root>` and use a native `<label>` with a
  matching `htmlFor`. Let the first input use the field label, and add `aria-label` to the
  remaining inputs so assistive technology can announce which slot is focused." Consistent with
  behavior.md "Accessibility (roles, aria-*, id linking)": a native label associated via the root
  id gives every slot the same accessible name, `aria-label` on the first slot is ignored when a
  shared label is associated (hence the docs only add it to slots 2+), and `aria-label` on later
  slots is kept. The root-id-derived slot ids recorded in the same section support the
  id/htmlFor association.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:35-37`
- "Optionally, add `aria-describedby` when supporting text should be announced with the field."
  Consistent with behavior.md "Accessibility (roles, aria-*, id linking)": root
  `aria-describedby` is forwarded to the group.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:39`
- Form integration guidance: "Use [Field](/react/components/field) to handle label associations
  and form integration." Consistent with behavior.md "Accessibility (roles, aria-*, id
  linking)": `Field.Label` associates with the first slot and `Field.Description` is applied to
  the group.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:58`
- "Pass `autoSubmit` to submit the owning form automatically when all slots are filled, or use
  `onValueComplete` to react to completion without submitting." Matches behavior.md "Edge cases
  (rapid interactions, unmount, nesting)" (`autoSubmit` submits the owning form on completion,
  default off) and "Events (names, payload shape, bubbling, preventDefault semantics)"
  (`onValueComplete` fires exactly when the OTP becomes complete).
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:77-78`
- "Use `validationType="alphanumeric"` for recovery, backup, or invite codes that mix letters and
  numbers." The charset part matches behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" (`alphanumeric` keeps only `[a-zA-Z0-9]`); the recovery/backup/invite framing is
  docs-only use-case naming, not a behavioral claim.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:82-83`
- Grouped layouts: "Wrap subsets of inputs in your own layout elements and use
  `<OTPField.Separator>` when you want the code presented in smaller visual chunks such as
  `123-456`." Matches behavior.md "DOM structure & portal behavior" (arbitrary wrapper elements
  between slots do not affect slot counting; `OTPField.Separator` renders its children inline
  between groups and does not affect slot counting).
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:91-92`
- Placeholder hints: "`<OTPField.Input>` is a real input, so native `placeholder` props and CSS
  work as usual." The native-input part matches behavior.md "Public API surface (props, parts,
  subcomponents)" (`OTPField.Input` renders a native `HTMLInputElement`). The claim that the
  example "keeps placeholder hints visible until the active slot receives focus" describes the
  demo's styling behavior and has no counterpart in behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:100-101`
- Custom normalization: "Use `normalizeValue` to normalize accepted values before state updates…
  It runs after `validationType` filtering, and the result is filtered against `validationType`
  again. Use `validationType="none"` when the normalizer should provide the full validation
  rule." "Runs after `validationType` filtering" matches behavior.md "State model" (custom
  `normalizeValue` composes after built-in validation; `none` disables built-in filtering and
  uses `normalizeValue`). The "result is filtered against `validationType` again" claim is not
  recorded in behavior.md; see Discrepancies.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:109-112`
- "Pair custom rules with `inputMode` for keyboard hints and `onValueInvalid` for rejected
  characters." Consistent with behavior.md "Accessibility (roles, aria-*, id linking)" (a custom
  `inputMode` is honored with `validationType="none"` and can override the built-in one
  otherwise) and "Events (names, payload shape, bubbling, preventDefault semantics)"
  (`onValueInvalid` fires whenever characters are rejected).
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:114`
- Masked entry: "Use `mask` when the code should be obscured while it is being typed."
  Consistent with behavior.md "DOM structure & portal behavior" (`mask` renders all slots as
  `input[type="password"]`, with a per-slot `type` prop able to override it; the override is not
  mentioned on this page).
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:122`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the three parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesOTPField.Root />`,
  `### Input` → `<TypesOTPField.Input />`, `### Separator` → `<TypesOTPField.Separator />`.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:128-142`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesOTPField } from './types';`); no props, prop types, defaults, or prop
  descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:130`
- Parts documented on this page: Root, Input, Separator — the same three-part set recorded in
  behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:132-142`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- Three fenced code blocks exist in the page:
  1. Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ OTPField }` from
     `@base-ui/react/otp-field` and assembles `<OTPField.Root>` containing `<OTPField.Input />`
     and `<OTPField.Separator />`.
     `docs/src/app/(docs)/react/components/otp-field/page.mdx:22-29`
  2. Labeling example (` ```tsx title="OTP Field with a native label and description" `): a
     `<label htmlFor="verification-code">`, `<OTPField.Root id="verification-code" length={6}`
     `aria-describedby="verification-code-description">` with six `<OTPField.Input>` slots — the
     first without `aria-label`, the rest with `aria-label="Character N of 6"` — plus a
     `<p id="verification-code-description">` supporting text.
     `docs/src/app/(docs)/react/components/otp-field/page.mdx:41-54`
  3. Form integration example (` ```tsx title="Using OTP Field in a form" {2} `, carrying a
     `{2}` line-highlight marker): `<Form>` wrapping `<Field.Root name="verificationCode">` with
     `Field.Label`, `Field.Description`, and `<OTPField.Root length={6}>` with six
     `<OTPField.Input>` slots (same aria-label pattern as the labeling example).
     `docs/src/app/(docs)/react/components/otp-field/page.mdx:60-75`
- No other code blocks exist in the page. The Alphanumeric, Grouped layouts, Placeholder hints,
  Custom normalization, and Masked entry examples render demo components
  (`<DemoOTPFieldAlphanumeric />`, `<DemoOTPFieldGrouped />`, `<DemoOTPFieldFocusedPlaceholder />`,
  `<DemoOTPFieldCustomNormalize />`, `<DemoOTPFieldPassword />`, plus `<DemoOTPFieldHero />` at
  the top) imported from `./demos/*`; their code lives in demo files and is out of scope here
  (Stage 2).
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:10-12`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:85-87`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:94-96`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:103-105`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:116-118`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:124-126`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/otp-field/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: after `normalizeValue` runs, "the result is
  filtered against `validationType` again". behavior.md "State model (controlled/uncontrolled,
  defaults, transitions)" records only that custom normalization composes after built-in
  filtering, and its utils summary records filtering → normalization → clamping; no test asserts
  a second filtering pass of the normalized result.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:110-111`
- Docs-only claim, not covered by behavior.md: the placeholder-hints example "keeps placeholder
  hints visible until the active slot receives focus" — a styling behavior of the demo with no
  test counterpart recorded anywhere in behavior.md (no placeholder-related test exists there).
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:101`
- Omission (docs page vs. behavior.md): behavior.md "Accessibility (roles, aria-*, id linking)"
  records a development warning when the first slot has an `aria-label` but no associated label
  ("Base UI: <OTPField.Input> ignores `aria-label` on the first input."); the page instructs
  adding `aria-label` to slots but never mentions this warning. The page's guidance (first input
  uses the field label, later inputs get `aria-label`) follows the warning's semantics, so this
  is an omission of a developer-facing caveat, not a contradiction.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:35-37`
- Omission (docs page vs. behavior.md): behavior.md "Public API surface (props, parts,
  subcomponents)" records that a per-slot `type` prop overrides `mask` (e.g. `type="tel"` wins
  over internal masking); the Masked entry section describes `mask` without mentioning the
  escape hatch. Omission, not a contradiction.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:122`

## Cross-links to other docs pages

- `[Labeling an OTP field](#labeling-an-otp-field)` — same-page anchor link within the Usage
  guidelines bullet. `docs/src/app/(docs)/react/components/otp-field/page.mdx:16`
- `[forms guide](/react/handbook/forms)` — link to the forms handbook page.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:16`
- `[Field](/react/components/field)` — link to the Field component page, in the Form integration
  section. `docs/src/app/(docs)/react/components/otp-field/page.mdx:58`
- No external links exist on the page (no MDN or third-party URLs).
- Demo components (`./demos/hero`, `./demos/alphanumeric`, `./demos/grouped`,
  `./demos/focused-placeholder`, `./demos/custom-sanitize`, `./demos/password`) are imported and
  rendered on this page itself; they are same-page imports, not cross-page links.
  `docs/src/app/(docs)/react/components/otp-field/page.mdx:10`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:85`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:94`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:103`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:116`, `docs/src/app/(docs)/react/components/otp-field/page.mdx:124`
