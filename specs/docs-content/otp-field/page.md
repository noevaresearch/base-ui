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

## Snippet & behaviour contract

Per `specs/docs-content/CONTRACT.md`: every snippet on this page must show the **port's** API and
every example must reproduce upstream's behaviour. Authored 2026-09-16 by reopening
`docs-content: components/otp-field`, which was marked `done` while all three of its embedded code
blocks carried upstream's JSX/TSX verbatim (the pre-change tree's `code_block(Lang::Jsx, "Anatomy",
ANATOMY_SNIPPET)` call and its two `Lang::Tsx` siblings: `import { OTPField } from
'@base-ui/react/otp-field'` plus JSX) and while `node ralph/scripts/check-docs-contract.mjs` listed
it among the pages "Marked done without a snippet & behaviour contract" — every structural gate
green, the page teaching the wrong framework.

The port's real surface is the namespaced module `leptos_ui::OTPField` (`crates/leptos-ui/src/otp_field.rs`):
`OTPField::Root`, `OTPField::Input`, `OTPField::Separator` as `view!` components, with the `Root`'s
`length`/`id`/`aria_describedby`/`validation_type`/`normalize_value`/`mask`/`name` props
(`crates/leptos-ui/src/otp_field.rs:1849-1916`) and the `Input`'s `aria_label` (`crates/leptos-ui/src/otp_field.rs:1988-1993`). The
checkbox/button/field/checkbox-group pages are the translation precedent; `crates/leptos-ui/tests/part_surface.rs`
pins the three namespaced parts.

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:22-29` (Anatomy) | `use leptos_ui::OTPField;` then assemble `<OTPField::Root length=6>` wrapping `<OTPField::Input />` and `<OTPField::Separator />` in `view!` markup | `specs/library/otp-field/behavior.md:14` (the namespace export's three parts), `specs/library/otp-field/behavior.md:17` (`length` takes the slot count), `specs/library/otp-field/behavior.md:102` (root `div[role=group]` containing N slots, Separator children inline) | the page renders the three-part assembly — `crates/docs-app/src/render_test.rs` `otp_field_page_component_renders_the_full_page_structure` asserts the rendered page contains `OTPField::Root` and the three-part anatomy; the crate's own `crates/leptos-ui/src/otp_field_view_tests.rs` `the_namespaced_root_nests_its_slots_inside_the_group_element` and `the_namespaced_separator_renders_its_children_without_consuming_a_slot` prove the parts themselves |
| Hero demo (`docs/src/app/(docs)/react/components/otp-field/page.mdx:10-12`, source cited in `demos.json` entry `hero`) | the hero's composition against the port: a native `<label for=…>` plus `<OTPField::Root id=… length=6 aria_describedby=…>` holding six `<OTPField::Input>` slots (first without `aria_label`, the rest announcing their position) and the supporting `<p id=…>` | `specs/library/otp-field/behavior.md:82` (root is `role="group"`), `specs/library/otp-field/behavior.md:86` (a native label associated via the root id gives every slot the same accessible name), `specs/library/otp-field/behavior.md:87` (root `aria-describedby` forwarded to the group), `specs/library/otp-field/behavior.md:94` (first slot's `aria-label` ignored when a shared label exists, later slots kept), `specs/library/otp-field/behavior.md:96` (slot ids derive from the root id), `specs/library/otp-field/behavior.md:89` (first slot `autocomplete="one-time-code"`, later slots `off`), `specs/library/otp-field/behavior.md:91` (the built-in `validationType` supplies the slots' `inputMode`/`pattern` — the port's default is `OtpValidationType::Numeric`) | `render_test.rs` `otp_field_hero_demo_renders_the_real_part_composition` — the live DOM: `div[role='group']`, six real slots, the label's `for` equal to the first slot's id, derived slot ids, the per-slot `aria-label` rule, `autocomplete` and `inputmode`; **interaction observable** `render_test.rs` `otp_field_slots_accumulate_characters_across_slots` — real keystrokes through the port's write path leave the earlier character committed (`'A'` survives the `'b'` typed into the next slot), which is the slot-entry behaviour every demo stands on, with `otp_field_composition_attaches_the_ports_write_path` proving the write path is attached at all |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:41-54` (Labeling an OTP field) | the port's equivalent of upstream's block: native `<label for="verification-code">`, `<OTPField::Root id="verification-code".to_string() length=6 aria_describedby="verification-code-description".to_string()>`, six `<OTPField::Input>` slots (first bare, the rest with `aria_label="Character N of 6".to_string()`), and the `<p id="verification-code-description">` | `specs/library/otp-field/behavior.md:86` (native label + matching root id = one shared accessible name), `specs/library/otp-field/behavior.md:96` (slot ids `verification-code` … `verification-code-6`), `specs/library/otp-field/behavior.md:94` (the first slot's `aria-label` is ignored when a shared label is associated — hence the docs' choice to label slots 2+ only), `specs/library/otp-field/behavior.md:87` (the root's `aria-describedby` is forwarded to the group) | `render_test.rs` `otp_field_hero_demo_renders_the_real_part_composition` is this same composition with the port's generated ids (label `for` = root id, description `<p>` id = `<root-id>-description`, slot ids derived, first slot carries no `aria-label`); the static `verification-code` literal in the snippet is the demo's generated id made readable |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:60-75` (Form integration, upstream `{2}` highlight on the `Field.Root` line) | `<Form>` wrapping `<Field::Root name="verificationCode".to_string()>` with `<Field::Label>`/`<Field::Description>` and `<OTPField::Root length=6>` holding the six slots — the composition `docs-content: components/checkbox`'s form row and the checkbox-group form snippet already teach | `specs/library/otp-field/behavior.md:83` (`Field.Label` associates with the FIRST slot), `specs/library/otp-field/behavior.md:84` (`Field.Description` applies to the group: the group gets `aria-labelledby` plus an `aria-describedby` naming the Field description), `specs/library/otp-field/behavior.md:85` (wrapping a whole field in `Field.Label`/`Field.Description` gives every slot the label's id), `specs/library/otp-field/behavior.md:131` (a `required` hidden input blocks `form.checkValidity()` while incomplete) | **NO port observable yet — carried open, not claimed** (see the gaps below): nothing in the tree mounts a `Form`/`Field`-wrapped OTP field, so this row's obligations are proven for UPSTREAM by `specs/library/otp-field/behavior.md:83-85`/`:131` and for the port only indirectly (the Field suite's own label-association tests, the crate's `required` coverage in `crates/leptos-ui/src/otp_field.rs`'s hidden-input path) |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:80-87` (Alphanumeric verification codes) | the demo's own props, expressed through the port: `<OTPField::Root length=6 validation_type=OtpValidationType::Alphanumeric …>` with the slots inside | `specs/library/otp-field/behavior.md:30` (typing/pasting filtered per `validationType`; `alphanumeric` keeps only `[a-zA-Z0-9]`), `specs/library/otp-field/behavior.md:91` (the built-in type supplies the slot `pattern`/`inputMode`) | `render_test.rs` `otp_field_slots_accumulate_characters_across_slots` types `'A'` and `'b'` through the alphanumeric demo and asserts the committed slot values — an accepted alphanumeric character really lands, and the earlier one survives |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:89-96` (Grouped layouts) | upstream's demo has no fenced snippet; the port's obligation is that its own layout wrappers hold the slots and `<OTPField::Separator>` sits between the groups | `specs/library/otp-field/behavior.md:102` (arbitrary wrapper divs between slots do not affect slot counting; Separator renders its children inline and does not consume a slot), `specs/library/otp-field/behavior.md:20` (Separator renders its children between groups) | `render_test.rs` `otp_field_grouped_demo_keeps_slot_registration_across_wrapper_elements` — two wrapper layouts of three slots each under the group, the shared Separator rendered between them with its class carried verbatim, and slot registration intact |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:98-105` (Placeholder hints) | the demo's own props through the port: `OTPField::Input` with a native `placeholder` riding the element-attribute rest | `specs/library/otp-field/behavior.md:19` (`OTPField.Input` is a real input: composed handlers, `aria-label`, per-slot `type`), `specs/library/otp-field/behavior.md:16` (plain-div/plain-input props are accepted) | `render_test.rs` `otp_field_password_and_placeholder_demos_follow_their_props` — the placeholder attribute is present on the rendered slots of that demo |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:107-118` (Custom normalization) | the demo's own props through the port: `normalize_value` (and `validation_type=OtpValidationType::None` for the full-custom rule) plus `on_value_invalid` for rejected characters | `specs/library/otp-field/behavior.md:31-32` (`validationType: 'none'` disables built-in filtering and uses `normalizeValue`; with a non-`none` type custom normalization composes AFTER built-in validation), `specs/library/otp-field/behavior.md:113` (`onValueInvalid` fires with the raw rejected characters) | `render_test.rs` `otp_field_custom_sanitize_demo_normalizes_and_reports_rejected_characters` — a real keystroke, the uppercasing on the committed value, and the rejected-character report through the demo's `aria-live` message and alternating highlight class |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:120-126` (Masked entry) | the demo's own props through the port: `<OTPField::Root mask=true …>` (a per-slot `input_type` overrides it) | `specs/library/otp-field/behavior.md:106` (`mask` renders every slot as `input[type="password"]`; a per-slot `type` overrides it), `specs/library/otp-field/behavior.md:19` (`type` wins over internal masking) | `render_test.rs` `otp_field_password_and_placeholder_demos_follow_their_props` — the masked demo's slots really are password-typed inputs |
| `docs/src/app/(docs)/react/components/otp-field/page.mdx:14-16` (Usage guidelines) | no snippet (prose guidance); its two claims are the native-`<label>` and `Field` routes to an accessible name, both taught by the rows above | `specs/library/otp-field/behavior.md:86` (native `<label htmlFor={rootId}>` gives every slot the same accessible name), `specs/library/otp-field/behavior.md:83` (`Field.Label` associates with the first slot), `specs/library/otp-field/behavior.md:95` (the port warns when the first slot carries `aria-label` with no associated label — the caveat the guideline encodes) | covered by the hero row's `otp_field_hero_demo_renders_the_real_part_composition` (native label) and by the Field suite's label-association tests (`crates/leptos-ui/src/field_tests.rs`), which is what the checkbox page's form row cites for the same association |

Gaps carried open against this contract (do not mark the page's snippet work done over them):

* **the Form-integration row has no observable in this port.** No test in the tree mounts a
  `Form`/`Field`-wrapped OTP field, so `specs/library/otp-field/behavior.md:83-85` and `:131` are proven for the port only
  by the Field suite's own tests, not by an OTP-field observable. Per `CONTRACT.md` requirement 3
  this is stated rather than papered over, and it is logged to `ralph/logs/spec-discrepancies.md`
  for whoever owns the next OTP-field test pass. The snippet's composition is compile-checked
  (below), which is a weaker claim and is labelled as such.
* **`.to_string()` on the string props.** The port's `Root`/`Input` string props take `String`, so
  every example carries `.to_string()` where upstream writes a bare literal. That is the crate's
  real API today, not a snippet workaround (the field page carries the same shape for `class=`),
  but it is an ergonomics gap the `docs-ergonomics:` lane owns — recorded here so the divergence is
  deliberate and visible.
* **the upstream `{2}` highlight directive on the form block is not reproduced.** Upstream's fence
  marks line 2 (`<Field.Root name="verificationCode">`). The port's `code_block` renders no
  highlight affordance, which is `docs-chrome: code blocks`' scope, not this contract's; the
  composition itself is faithful to that line.
* **the five demo sections embed no fenced snippet** — upstream renders demo components there and
  so does this page, so their rows above name the demo's props and the observable, not a snippet.
  Their source lives in the demo files (`specs/docs-content/otp-field/demos.json`), out of this
  page's snippet scope.
