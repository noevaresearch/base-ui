# `getDefaultFormSubmitter` — behavior spec

Unit: `packages/utils/src/getDefaultFormSubmitter` (Phase A util → crate `leptos-ui-utils`).
Source of truth: `packages/utils/src/getDefaultFormSubmitter.test.ts` (the only test file for this unit).

## Public API surface (props, parts, subcomponents)

- Single named export `getDefaultFormSubmitter`. No components, no props object, no parts.
- Signature as exercised: takes one argument — the `HTMLFormElement` to inspect, obtained via
  `document.querySelector<HTMLFormElement>('#test-form')` in every test
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:5-8`) — and returns either a DOM element
  or `null`, asserted by strict identity (`toBe`) against the node itself, not a copy
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:23`, `packages/utils/src/getDefaultFormSubmitter.test.ts:68`).
- Return type narrowing to `HTMLButtonElement | HTMLInputElement` (exported as
  `DefaultFormSubmitter`): UNVERIFIED — inferred from
  `packages/utils/src/getDefaultFormSubmitter.ts:1`, no test asserts the static type; tests only
  compare against `<button>` and `<input>` nodes.
- Accepting `null` as the form argument: UNVERIFIED — inferred from
  `packages/utils/src/getDefaultFormSubmitter.ts:14-17`, no test passes a null form.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — stateless pure function. Each call independently inspects the given form; no state,
defaults, or transitions exist (`packages/utils/src/getDefaultFormSubmitter.test.ts:5-69`).

## Keyboard interactions

N/A — the utility performs no keyboard handling and no test asserts any. Its stated purpose is
to let callers mirror native Enter-key implicit submission by clicking the returned submitter
(UNVERIFIED — inferred from `packages/utils/src/getDefaultFormSubmitter.ts:4-8`, no test asserts
keyboard behavior or click semantics).

## Focus management

N/A — no focus behavior is implemented or asserted by any test.

## Accessibility (roles, aria-*, id linking)

N/A — no accessibility behavior is implemented or asserted by any test.

## DOM structure & portal behavior

- Renders nothing and creates no DOM; tests build the DOM via `document.body.innerHTML` and the
  function only reads it (`packages/utils/src/getDefaultFormSubmitter.test.ts:5-8`).
- Submitters associated through the `form` attribute from outside the form element are found: a
  `<button form="test-form" type="submit">` placed before the form in the document is returned
  over an internal submit button (`packages/utils/src/getDefaultFormSubmitter.test.ts:14-24`).
  This matches traversal of the form's associated-controls collection including form-attribute
  association (UNVERIFIED beyond this markup — inferred from
  `packages/utils/src/getDefaultFormSubmitter.ts:19`, no test covers an external submitter placed
  after the form in document order).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are dispatched, emitted, or asserted by any test. That clicking the returned
submitter preserves browser semantics (`SubmitEvent.submitter`, submitter attributes) is
UNVERIFIED — inferred from `packages/utils/src/getDefaultFormSubmitter.ts:6-8`, no test asserts
event behavior.

## Edge cases (rapid interactions, unmount, nesting)

Proven behaviors:

- First submitter wins: with an external `form`-associated submit button preceding the form and
  an internal submit button, the earlier one is returned
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:14-24`).
- Disabled submitters are not filtered out: a `<button type="submit" disabled>` listed first is
  returned (`packages/utils/src/getDefaultFormSubmitter.test.ts:26-35`).
- A `<button>` with no `type` attribute (implicit submit type) counts as a submitter and wins
  over a later explicit `type="submit"` button
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:37-46`).
- `<input type="submit">` counts as a submitter, while non-submit controls are skipped: a
  `<button type="button">` and `<input type="reset">` preceding it do not match
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:48-58`).
- No submitter present (checkbox + `type="button"` button only) → returns `null`
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:60-69`).

Unproven behaviors (callers in this repo must not rely on them being specified here):

- A disabled `<input type="submit">` submitter: UNVERIFIED — inferred from
  `packages/utils/src/getDefaultFormSubmitter.ts:22-29`, no test asserts this (the disabled test
  uses a `<button>` only, `packages/utils/src/getDefaultFormSubmitter.test.ts:26-35`).
- `<input type="image">` is intentionally unsupported (Chromium omits it from `form.elements`):
  UNVERIFIED — inferred from the comment at `packages/utils/src/getDefaultFormSubmitter.ts:25-26`,
  no test covers image inputs.
- Order guarantees beyond the tested markup (e.g. submitter ordering when form-attribute and
  in-form controls interleave differently): UNVERIFIED — inferred from
  `packages/utils/src/getDefaultFormSubmitter.ts:19-31`, no test asserts this.
- Rapid repeated calls, nesting, or unmount lifecycle interaction: N/A — pure function over an
  existing DOM node with no lifecycle; no test covers call-pattern edge cases
  (`packages/utils/src/getDefaultFormSubmitter.test.ts:5-69`).

## Shared harness dependencies

None. The test file imports only `describe`/`it`/`expect`/`afterEach` from `vitest` and
`getDefaultFormSubmitter` from its sibling module
(`packages/utils/src/getDefaultFormSubmitter.test.ts:1-2`). There is no `#test-utils` or
`packages/react/test` harness dependency; DOM setup uses the raw `document` global directly
(`packages/utils/src/getDefaultFormSubmitter.test.ts:5-12`).
