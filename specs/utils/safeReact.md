# `safeReact` — behavior spec

Unit: `packages/utils/src/safeReact` (Phase A util → crate `leptos-ui-utils`).
Source of truth: none — this unit has no test file. The generated unit manifest records
`testFiles: []` for it (`ralph/generated/utils.json:199-205`), and no
`safeReact.test.*` exists anywhere in the repo. The unit's `TODO.md` entry has no
`wraps-external:` field (`TODO.md:114-118`), so there is no runtime delegation to a
third-party npm package and no external crate for Stage 3 to bind against; the Rust port
must implement the behavior itself.

The unit does have **indirect** test coverage: three dedicated "React 17 fallback" suites
mock this module via `vi.mock` and stub out two of its properties, and several more suites
mutate or probe `SafeReact.captureOwnerStack` at runtime. Those suites are cited below,
clearly framed as indirect evidence — they prove the consumers' contracts with
`SafeReact`, not the clone's internals.

## Public API surface (props, parts, subcomponents)

- Single module with a single named export `SafeReact`: a plain object created by
  shallow-spreading the entire React namespace at module-evaluation time and cast to
  `typeof React` (`packages/utils/src/safeReact.ts:11`). No components, no props, no
  parts, no subcomponents. UNVERIFIED — inferred from the full 11-line file
  (`packages/utils/src/safeReact.ts:1-11`); no test asserts the export surface directly.
- Documented intent (docstring, not test-proven): the clone exists so newer React APIs can
  be read from a plain object; bundlers rewrite direct `React.someNewApi` reads into named
  imports, which breaks React 17, while property reads on the clone stay optional
  (`packages/utils/src/safeReact.ts:3-10`).
- Import specifier: all in-repo `packages/react` consumers import
  `import { SafeReact } from '@base-ui/utils/safeReact'` (e.g.
  `packages/react/src/otp-field/root/OTPFieldRoot.tsx:3`,
  `packages/react/src/drawer/popup/DrawerPopup.tsx:4`,
  `packages/react/src/field/label/FieldLabel.tsx:4`); the two `packages/utils` consumers
  use the relative `./safeReact` (`packages/utils/src/useId.ts:3`,
  `packages/utils/src/useStableCallback.ts:2`). The specifier resolves through the
  wildcard `./*": "./src/*.ts"` entry of `@base-ui/utils`'s export map, with test files
  explicitly blocked from being exported (`packages/utils/package.json:12-17`).
- Consumed API subset: only three React APIs are ever read through the clone in this repo —
  `useId` (`packages/utils/src/useId.ts:24`),
  `useInsertionEffect`/`useLayoutEffect` (`packages/utils/src/useStableCallback.ts:5-12`),
  and `captureOwnerStack` (12 call sites across 9 files, always optional-chained with a
  fallback: `packages/react/src/internals/use-button/useButton.ts:47`,
  `packages/react/src/internals/use-button/useButton.ts:56`,
  `packages/react/src/field/label/FieldLabel.tsx:59`,
  `packages/react/src/field/label/FieldLabel.tsx:67`,
  `packages/react/src/combobox/label/ComboboxLabel.tsx:47`,
  `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.tsx:109`,
  `packages/react/src/drawer/popup/DrawerPopup.tsx:171`,
  `packages/react/src/number-field/input/NumberFieldInput.tsx:420`,
  `packages/react/src/otp-field/root/OTPFieldRoot.tsx:655`,
  `packages/react/src/otp-field/root/OTPFieldRoot.tsx:668`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:83`,
  `packages/react/src/otp-field/input/OTPFieldInput.tsx:281`). The clone carries every
  other React namespace member too, but nothing reads them through `SafeReact`.
  UNVERIFIED — inferred from `packages/utils/src/safeReact.ts:11` plus repo-wide
  `SafeReact.` usage search; no test asserts the consumed subset.
- It is a *fresh plain object*, not the real React namespace object: tests reconfigure its
  `captureOwnerStack` property with `Object.defineProperty` and later restore it without
  breaking React itself (`packages/react/src/combobox/label/ComboboxLabel.test.tsx:28-54`),
  and the React 17 suites spread-and-override it wholesale at the module boundary
  (`packages/react/src/field/root/FieldRoot.react17.test.tsx:8-18`,
  `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:9-19`,
  `packages/react/src/otp-field/root/OTPFieldRoot.react17.test.tsx:6-16`). This
  operationally proves the properties are plain/configurable and the object is spreadable.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — not a stateful component (no controlled/uncontrolled concepts, no defaults, no
transitions). Module-lifecycle facts that matter instead:

- The clone is created once at module evaluation, so its object identity and property set
  are fixed for the module's lifetime (`packages/utils/src/safeReact.ts:11`). UNVERIFIED —
  inferred from that line; no production source writes to the clone (only test files do),
  and no test asserts production immutability.
- Property-read timing is split, with test backing for the call-time side:
  - Import-time snapshots: `useId` (`packages/utils/src/useId.ts:24`) and
    `useInsertionEffect`/`useLayoutEffect` (`packages/utils/src/useStableCallback.ts:5-12`)
    read the clone at module scope, so binding values are fixed when those modules are
    first evaluated. UNVERIFIED — inferred from those lines; no test mutates the clone
    after evaluation for these two APIs.
  - Call-time reads: every `captureOwnerStack` consumer reads it at warning time
    (`packages/react/src/internals/use-button/useButton.ts:47`), so mutating the clone
    between renders changes warning behavior — proven by tests that swap it per-test and
    restore it in `finally` (`packages/react/src/combobox/label/ComboboxLabel.test.tsx:28-54`,
    `packages/react/src/drawer/popup/DrawerPopup.test.tsx:68-95`).

## Keyboard interactions

N/A — non-visual module; no keyboard behavior exists to assert.

## Focus management

N/A — no focus behavior exists to assert.

## Accessibility (roles, aria-*, id linking)

N/A — the clone has no accessibility surface of its own. Indirectly, the React 17 fallback
it enables (no `SafeReact.useId`) has test-proven a11y consequences in consumers: label
`for` association and parent `aria-controls` wiring are deferred until fallback ids arrive
during hydration (`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:83-113`,
`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:115-150`) — that is
`useId`/consumer behavior, not behavior of this module.

## DOM structure & portal behavior

N/A — renders nothing, creates no DOM, uses no portal.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — emits no events. Its only output consumed at runtime is the string returned by
`SafeReact.captureOwnerStack?.()`, which is appended to development warning messages
(`packages/react/src/internals/use-button/useButton.ts:47`); when the API is missing the
expression resolves to `''`, so warnings are emitted without an owner stack.

## Edge cases (rapid interactions, unmount, nesting)

- React 17 (no `SafeReact.useId`): the direct consumer falls back to a state+effect
  id generator instead of React's (`packages/utils/src/useId.ts:24-41`). The three react17
  suites mock this module (spreading the original and setting `useId: undefined`,
  `captureOwnerStack: undefined`) and prove the consumer-side fallback behavior: a Field
  control whose explicit id is removed gets a generated id that matches its label's `for`
  (`packages/react/src/field/root/FieldRoot.react17.test.tsx:24-57`); a Checkbox drops an
  explicit id once the prop is removed and never reuses an unmounted keyed checkbox's id
  (`packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:51-81`); server-rendered
  OTP inputs omit generated ids entirely
  (`packages/react/src/otp-field/root/OTPFieldRoot.react17.test.tsx:21-33`).
- `captureOwnerStack` absent (module-level stub): development warnings still fire with the
  same messages, just without an owner stack — Field label-mismatch warnings are asserted
  under the stubbed module
  (`packages/react/src/field/root/FieldRoot.react17.test.tsx:79-113`).
- `captureOwnerStack` present but returning `null`: consumers produce the identical message
  with no stack appended — proven via `vi.spyOn(SafeReact, 'captureOwnerStack')`
  returning `null` in Drawer.Popup's exact-string warning
  (`packages/react/src/drawer/popup/DrawerPopup.test.tsx:68-95`), NumberField.Input's paste
  warning (`packages/react/src/number-field/input/NumberFieldInput.test.tsx:1424-1455`),
  OTPField.Root's singular-input and invalid-length warnings
  (`packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1552-1572`,
  `packages/react/src/otp-field/root/OTPFieldRoot.test.tsx:1574-1595`), and OTPField.Input's
  paste and first-slot `aria-label` warnings
  (`packages/react/src/otp-field/input/OTPFieldInput.test.tsx:531-558`,
  `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:815-842`).
- Mutability + module singleton: swapping `SafeReact.captureOwnerStack` with
  `Object.defineProperty` inside one test changes consumer behavior within that test and is
  restored afterwards (`packages/react/src/combobox/label/ComboboxLabel.test.tsx:28-54`,
  `packages/react/src/combobox/label/ComboboxLabel.test.tsx:56-85`) — indirect proof that
  all importers share one object instance whose properties can be reconfigured. That the
  restore fully isolates later tests is UNVERIFIED — inferred from the `finally` restore
  pattern; no test asserts cross-test state isolation.
- Preact compatibility branch: `useStableCallback` rejects `useInsertionEffect` when React
  itself aliases it to `useLayoutEffect` (Preact fires it too late)
  (`packages/utils/src/useStableCallback.ts:5-12`). UNVERIFIED — inferred from those lines;
  no test simulates Preact.
- Production `NODE_ENV`: consumer warnings are skipped entirely and `captureOwnerStack` is
  never called (`packages/react/src/combobox/label/ComboboxLabel.test.tsx:56-85`) —
  consumer-side guards, cited as indirect evidence only.
- Rapid interactions / unmount / nesting: N/A — the module has no interaction surface. The
  mount/unmount/id-lifecycle flows exercised around the React 17 fallback (e.g. keyed
  remounts getting fresh generated ids,
  `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:67-81`) are consumer
  behavior, not module behavior.

## Shared harness dependencies

- The unit itself has no test file, so it imposes no harness dependency of its own.
- The three react17 suites import `createRenderer` from the shared `#test-utils` harness
  (resolving to `packages/react/test/index.ts`) plus `@mui/internal-test-utils`
  (`packages/react/src/field/root/FieldRoot.react17.test.tsx:5-6`,
  `packages/react/src/checkbox/root/CheckboxRoot.react17.test.tsx:6-7`,
  `packages/react/src/otp-field/root/OTPFieldRoot.react17.test.tsx:3-4`).
- The captureOwnerStack-probing suites do the same:
  `packages/react/src/combobox/label/ComboboxLabel.test.tsx:4` (also `describeConformance`,
  `isJSDOM`), `packages/react/src/drawer/popup/DrawerPopup.test.tsx:8-9`,
  `packages/react/src/number-field/input/NumberFieldInput.test.tsx:3`,
  `packages/react/src/number-field/input/NumberFieldInput.test.tsx:7`,
  `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:5`,
  `packages/react/src/otp-field/input/OTPFieldInput.test.tsx:9`,
  `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.test.tsx:2-3`.
- The direct `useId` consumer suite (whose primary branch runs through
  `SafeReact.useId` when defined, `packages/utils/src/useId.ts:34-37`) uses only
  `@mui/internal-test-utils`, not `#test-utils` (`packages/utils/src/useId.test.tsx:3`);
  its generation cases (e.g. `packages/utils/src/useId.test.tsx:28`) and its React 18
  server case (`packages/utils/src/useId.test.tsx:89`) exercise the clone-present path
  against the installed React. That the tests intentionally pin `SafeReact`'s clone
  behavior is UNVERIFIED — they target `useId`, not this module.
