# `visuallyHidden` — behavior spec

Unit: `packages/utils/src/visuallyHidden` (Phase A util → crate `leptos-ui-utils`).
Canonical test coverage: **none of its own** — `ralph/generated/utils.json` lists
`testFiles: []` for it (`ralph/generated/utils.json:408-413`), and there is no
`visuallyHidden.test.*` file anywhere in the repo. The only test in the repository that
exercises this unit at all lives in *another unit's* suite:
`packages/react/src/floating-ui-react/utils/tabbable.test.ts` (imports both exports at
`packages/react/src/floating-ui-react/utils/tabbable.test.ts:3`). That file is outside this
unit's own test list, so per the mining rules it would normally be out of scope; it is cited
here explicitly and narrowly because it contains the *only* test-proven behavior available for
this unit. Everything not covered by those two cases is UNVERIFIED and inferred from the
unit's own 24-line source. Stage 3 must not treat untested claims as test-proven behavior.

## Public API surface (props, parts, subcomponents)

- File-style module `packages/utils/src/visuallyHidden.ts` — the manifest lists exactly one
  src file for the unit (`ralph/generated/utils.json:411-413`).
- Exactly two named exports, both style-constant objects typed `React.CSSProperties`
  (`packages/utils/src/visuallyHidden.ts:14`, `packages/utils/src/visuallyHidden.ts:21`):
  - `visuallyHidden` (`packages/utils/src/visuallyHidden.ts:14-19`)
  - `visuallyHiddenInput` (`packages/utils/src/visuallyHidden.ts:21-24`)
- Both spread a single private (unexported) base object `visuallyHiddenBase`
  (`packages/utils/src/visuallyHidden.ts:3-12`) whose properties are:
  `clipPath: 'inset(50%)'`, `overflow: 'hidden'`, `whiteSpace: 'nowrap'`, `border: 0`,
  `padding: 0`, `width: 1`, `height: 1`, `margin: -1`. UNVERIFIED as a tested contract —
  inferred from `packages/utils/src/visuallyHidden.ts:3-12`, no test asserts the values.
- The two exports differ **only** in `position`: `visuallyHidden` adds
  `position: 'fixed', top: 0, left: 0` (`packages/utils/src/visuallyHidden.ts:14-19`);
  `visuallyHiddenInput` adds `position: 'absolute'` and sets no `top`/`left`
  (`packages/utils/src/visuallyHidden.ts:21-24`). UNVERIFIED — inferred from
  `packages/utils/src/visuallyHidden.ts:14-24`, no test asserts the delta.
- No default export, no functions, no hooks, no components, no props. UNVERIFIED — inferred
  from `packages/utils/src/visuallyHidden.ts:1-24` (the file contains only two object
  literals and a type-only React import).
- Observed consumer convention (UNVERIFIED — inferred from import statements, no test asserts
  usage semantics): component packages apply these constants to elements to hide them visually
  while keeping them functional — e.g. Switch (`packages/react/src/switch/root/SwitchRoot.tsx:6`),
  Checkbox (`packages/react/src/checkbox/root/CheckboxRoot.tsx:7`),
  Radio (`packages/react/src/radio/root/RadioRoot.tsx:5`),
  Select (`packages/react/src/select/root/SelectRoot.tsx:3`),
  NumberField (`packages/react/src/number-field/root/NumberFieldRoot.tsx:10`),
  OTPField (`packages/react/src/otp-field/root/OTPFieldRoot.tsx:8`),
  AriaCombobox (`packages/react/src/combobox/root/AriaCombobox.tsx:9`),
  Meter (`packages/react/src/meter/root/MeterRoot.tsx:3`),
  Progress (`packages/react/src/progress/root/ProgressRoot.tsx:3`),
  SliderThumb (`packages/react/src/slider/thumb/SliderThumb.tsx:6`),
  FocusGuard (`packages/react/src/utils/FocusGuard.tsx:5`),
  ToastViewport (`packages/react/src/toast/viewport/ToastViewport.tsx:6`),
  ComboboxInternalDismissButton
  (`packages/react/src/combobox/utils/ComboboxInternalDismissButton.tsx:4`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — two module-level constant object literals. No state, no props, no controlled/uncontrolled
duality, no defaults, no transitions. The objects are plain mutable literals (not
`Object.freeze`d), so a caller mutating the exported object would affect every consumer
process-wide. UNVERIFIED — inferred from `packages/utils/src/visuallyHidden.ts:3-24`, no test
asserts freeze/mutation semantics.

## Keyboard interactions

The unit implements no keyboard handling of its own (no listeners, no key logic) — N/A as
implemented behavior. However, its keyboard-relevant contract **is test-proven** (the only
test-proven behavior of this unit, via the cross-unit tabbable suite):

- A `<button>` with `visuallyHidden` applied via `Object.assign(button.style, visuallyHidden)`
  remains in the tab order: `isTabbable(button)` is `true` and `tabbable(document.body)`
  includes it (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-315`).
- A checkbox `<input>` with `visuallyHiddenInput` applied the same way likewise remains
  tabbable (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:317-326`).
- By contrast, the same suite proves that `visibility: hidden`
  (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:123-131`) and
  `display: none` subtrees
  (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:224-234`) remove elements from
  the tab order — so the constants demonstrably avoid the hiding mechanisms that break keyboard
  reachability. UNVERIFIED as an intent statement — inferred by comparing
  `packages/react/src/floating-ui-react/utils/tabbable.test.ts:123-234` with
  `:307-326`; no test asserts the constants' *purpose*.
- These two visually-hidden tabbability tests are not `it.skipIf(isJSDOM)`-gated, so they are
  expected to pass in both the jsdom and Chromium environments
  (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:307`,
  `packages/react/src/floating-ui-react/utils/tabbable.test.ts:317`).

## Focus management

N/A — the unit performs no focus management (no focus calls, no focus traps, no redirects).
The tabbability proven above implies keyboard focusability in principle, but no test asserts
that a styled element can actually receive focus via `.focus()` or Tab. UNVERIFIED — inferred
from `packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-326`, which only asserts
membership in the tabbable set.

## Accessibility (roles, aria-*, id linking)

- Sets no roles, aria attributes, or id links — the unit is pure CSS-in-JS constants. N/A
  beyond that.
- The property values constitute the standard CSS "visually hidden" technique: `clip-path:
  inset(50%)` renders the box invisible without removing it from the accessibility tree, with a
  1×1 px box, zero border/padding, `overflow: hidden`, `white-space: nowrap`, and
  `margin: -1` to neutralize layout side effects. UNVERIFIED as intent — inferred from
  `packages/utils/src/visuallyHidden.ts:3-12`; no test asserts screen-reader or a11y-tree
  behavior.

## DOM structure & portal behavior

N/A — creates no DOM, renders nothing, no portals. The constants are applied by consumers to
their own elements (UNVERIFIED — inferred from the import lines cited in
"Public API surface").

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, or handled.

## Edge cases (rapid interactions, unmount, nesting)

- The fixed-vs-absolute split is the unit's only behavioral fork: `visuallyHidden` is
  `position: fixed` anchored at `top: 0; left: 0` (taking the element out of flow relative to
  the viewport), while `visuallyHiddenInput` is `position: absolute` with no offsets, leaving
  it at its static position within its parent. UNVERIFIED — inferred from
  `packages/utils/src/visuallyHidden.ts:14-24`; no test asserts positioning outcomes.
- Rapid interactions, unmount, nesting: N/A — stateless constants with no lifecycle, listeners,
  or DOM involvement of their own; they cannot leak, race, or require cleanup.
  UNVERIFIED — inferred from `packages/utils/src/visuallyHidden.ts:1-24`, no test asserts this.
- Because both exports are shared module-level objects, per-element customization (e.g. the
  test's own `Object.assign(button.style, visuallyHidden)` pattern,
  `packages/react/src/floating-ui-react/utils/tabbable.test.ts:310`) copies properties onto a
  style object rather than sharing state — no aliasing hazard when used this way. UNVERIFIED as
  a general contract — inferred from `packages/utils/src/visuallyHidden.ts:3-24`; only this
  copy-on-assign usage is exercised by
  `packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-326`.

## Shared harness dependencies

- The unit has no test file of its own, so it has no direct harness. No `#test-utils` import,
  no file under `packages/react/test/`, and no shared utils-test harness is associated with it
  via its (empty) `testFiles` list (`ralph/generated/utils.json:408-413`).
- For completeness: the cross-unit test that exercises this unit imports `isJSDOM` from
  `#test-utils` (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:2`), which maps
  to `packages/react/test/index.ts` (`packages/react/package.json:110`) and re-exports
  `@base-ui/utils/testUtils` (`packages/react/test/index.ts:1`); `isJSDOM` is a UA sniff,
  `/jsdom/.test(window.navigator.userAgent)` (`packages/utils/src/testUtils.ts:4`). The flag is
  used only to env-gate other tests in that file — the visuallyHidden assertions are not gated
  (`packages/react/src/floating-ui-react/utils/tabbable.test.ts:307-326`).
