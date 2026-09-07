# `platform` — behavior spec

Unit: `packages/utils/src/platform` (Phase A util → crate `leptos-ui-utils`).
Source of truth: **none** — this unit has no test file. `ralph/generated/utils.json` lists
`testFiles: []` for this unit, and no `platform.test.ts(x)` exists under
`packages/utils/src/platform/`. Every behavioral claim below is therefore marked UNVERIFIED
per the mining rules, EXCEPT (a) facts read directly off the unit's own source lines, and
(b) "consumption facts": import lines and module-mock blocks in consumer test files that
prove which properties of the namespace consumers rely on. No consumer test asserts the
_values_ the detection logic produces; they only assert behavior under mocked flag values.

## Public API surface (props, parts, subcomponents)

- Not a component: no props, no parts, no subcomponents. A static, module-load-time
  detection namespace. Sole export is one named export `platform`
  (`packages/utils/src/platform/index.ts:32`). UNVERIFIED as a tested contract — inferred
  from `packages/utils/src/platform/index.ts:32`, no test asserts the export list.
- Publicly reachable as `@base-ui/utils/platform` via the explicit export map entry
  `"./platform": "./src/platform/index.ts"` (`packages/utils/package.json:14`). UNVERIFIED
  as a consumer-facing resolution — inferred from `packages/utils/package.json:14`; consumer
  test files do import that exact specifier (see consumption facts below).
- The namespace is an aggregate of five group namespaces re-exported by
  `packages/utils/src/platform/parts.ts:1-5`: `os`, `engine`, `screenReader`, `env`,
  `mediaQuery`. Internal helpers in `shared.ts` (`lowerUserAgent`, `lowerPlatform`,
  `maxTouchPoints`) are NOT re-exported through `parts.ts`, so they are not part of the
  public `platform` namespace (`packages/utils/src/platform/parts.ts:1-5`).
  UNVERIFIED — inferred from `packages/utils/src/platform/parts.ts:1-5`, no test asserts
  the absence of a `shared` group on the namespace.
- Group `os` — six boolean flags:
  - `ios`: `/^i(os$|p)/` on lowercased `navigator.platform`, or `platform === 'macintel'`
    with `maxTouchPoints > 1` (iPadOS 13+ reports as macOS) —
    `packages/utils/src/platform/os.ts:7-8`. UNVERIFIED — inferred from
    `packages/utils/src/platform/os.ts:7-8`, no test asserts the regex or the
    touch-point disambiguation.
  - `android`: `lowerPlatform === 'android'` or the UA contains `'android'` —
    `packages/utils/src/platform/os.ts:11-12`. UNVERIFIED — inferred from
    `packages/utils/src/platform/os.ts:11-12`, no test asserts it.
  - `mac`: `!ios && lowerPlatform.startsWith('mac')` — explicitly excludes iPadOS —
    `packages/utils/src/platform/os.ts:15`. UNVERIFIED — inferred from
    `packages/utils/src/platform/os.ts:15`, no test asserts it.
  - `windows`: `lowerPlatform.startsWith('win')` — `packages/utils/src/platform/os.ts:18`.
    UNVERIFIED — inferred from `packages/utils/src/platform/os.ts:18`; no test and no
    consumer mock anywhere references `windows`.
  - `linux`: `!android && /^(linux|chrome os)/.test(lowerPlatform)` —
    `packages/utils/src/platform/os.ts:21`. UNVERIFIED — inferred from
    `packages/utils/src/platform/os.ts:21`; no test and no consumer mock anywhere
    references `linux`.
  - `apple`: `mac || ios` — `packages/utils/src/platform/os.ts:24`. UNVERIFIED as a
    computed value — inferred from `packages/utils/src/platform/os.ts:24`; consumer tests
    only ever mock `apple` _together with_ its inputs (`ios: true, apple: true`,
    `mac: true, apple: true`, `mac: false, apple: false` — see test-level evidence below),
    so no test asserts that `apple` derives from `mac || ios`.
- Group `engine` — three boolean flags:
  - `webkit`: true iff `CSS.supports('-webkit-backdrop-filter:none')` (distinguishes
    WebKit from Blink, which only ships the unprefixed property) —
    `packages/utils/src/platform/engine.ts:7-8`. UNVERIFIED — inferred from
    `packages/utils/src/platform/engine.ts:7-8`, no test asserts the CSS-supports probe.
  - `gecko`: `!webkit && lowerUserAgent.includes('firefox')` — anchored to `!webkit` so
    Firefox-on-iOS (WebKit-based, UA marker `FxiOS/`) classifies as WebKit —
    `packages/utils/src/platform/engine.ts:15`. UNVERIFIED — inferred from
    `packages/utils/src/platform/engine.ts:10-15`, no test asserts it.
  - `blink`: `!webkit && lowerUserAgent.includes('chrom')` (covers Chrome/Chromium/Edge/
    Opera/Brave via the shared substring; Chrome-on-iOS `CriOS/` stays WebKit) —
    `packages/utils/src/platform/engine.ts:22`. UNVERIFIED — inferred from
    `packages/utils/src/platform/engine.ts:17-22`, no test asserts it.
- Group `screenReader` — one flag:
  - `voiceOver`: `=== apple` (pure OS check; actual screen-reader activation is
    undetectable, and engine-specific quirks are meant to be gated at call sites) —
    `packages/utils/src/platform/screen-reader.ts:12`. UNVERIFIED — inferred from
    `packages/utils/src/platform/screen-reader.ts:3-12`, no test asserts the derivation
    from `apple`; the consumer test mocks `voiceOver` directly
    (`packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:15`).
- Group `env` — one flag:
  - `jsdom`: `/jsdom|happydom/.test(lowerUserAgent)` — `packages/utils/src/platform/env.ts:4`.
    UNVERIFIED — inferred from `packages/utils/src/platform/env.ts:4`; no test asserts its
    value in jsdom/HappyDOM runners or otherwise.
- Group `mediaQuery` — one constant string:
  - `iOS`: `'@supports (-webkit-touch-callout: none)'`, a CSS query matching iOS/iPadOS
    WebKit browsers — `packages/utils/src/platform/media-query.ts:4`. UNVERIFIED — inferred
    from `packages/utils/src/platform/media-query.ts:4`; no test asserts it and no file in
    the repo consumes `platform.mediaQuery.iOS`.
- Raw-data source (`shared.ts`, internal): `readRawData()` returns
  `{ userAgent, platform, maxTouchPoints }`; SSR returns empty/zero values; in non-production
  builds it prefers `navigator.userAgentData` (Chromium) formatting brands as
  `brand/version` joined by spaces to avoid DevTools deprecation warnings; production builds
  always read the legacy `navigator.userAgent`/`navigator.platform` —
  `packages/utils/src/platform/shared.ts:23-46`. UNVERIFIED — inferred from
  `packages/utils/src/platform/shared.ts:23-46`, no test asserts either branch.
- Historical note: this namespace replaced the former `@base-ui/utils/detectBrowser`
  (#4920) (`packages/utils/CHANGELOG.md:32`). Consumption fact, not a behavior proof.
- Port note (Stage 3 binding): the TODO entry has no `wraps-external:` field — this unit
  wraps no third-party package and there is no external crate to bind; the Rust equivalent
  must reimplement the detection from the inputs documented above (UA string, platform
  string, maxTouchPoints, `CSS.supports` probe, `NODE_ENV`-gated userAgentData branch).
  The `mediaQuery.iOS` entry is a plain constant string, not detection logic.

### Test-level evidence (consumer module mocks)

No test file belongs to this unit, but consumer test files mock `@base-ui/utils/platform`
by spreading the actual namespace and overriding exactly one group. These blocks are the
only test-level proof of the namespace's shape and group structure:

- `os.android` override: `packages/react/src/drawer/root/DrawerRoot.test.tsx:24-34`,
  `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:10-20`,
  `packages/react/src/combobox/input/ComboboxInput.android.test.tsx:6-18`.
- `os.ios` + `os.apple` override:
  `packages/react/src/combobox/status/ComboboxStatus.iOS.test.tsx:7-18`,
  `packages/react/src/number-field/root/NumberFieldRoot.iOS.test.tsx:6-17`.
- `os.mac` + `os.apple` override (true variant):
  `packages/react/src/context-menu/root/ContextMenuRoot.test.tsx:14-25`; (false variant:
  `mac: false, apple: false`)
  `packages/react/src/context-menu/root/ContextMenuRoot.non-mac.test.tsx:12-23`.
- `engine.webkit` override:
  `packages/react/src/navigation-menu/root/NavigationMenuRoot.webkit.test.tsx:8-19`,
  `packages/react/src/floating-ui-react/hooks/useListNavigation.webkit.test.tsx:7-18`.
- `engine.gecko` override:
  `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.gecko.test.tsx:7-18`,
  `packages/react/src/combobox/input/ComboboxInput.gecko.test.tsx:7-19`.
- `screenReader.voiceOver` override:
  `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:8-18`.
- The mocks prove the mock contract "replace the `platform` key of the module, spreading
  `actual.platform` and overriding one group"; two of them return only the `platform` key
  with no top-level `...actual` spread
  (`packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:12-17`,
  `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.talkBack.test.tsx:14-19`),
  which is consistent with `platform` being the module's sole export.
- Direct runtime reads of engine flags as booleans for test gating (unmocked):
  `platform.engine.webkit` at `packages/react/src/slider/root/SliderRoot.test.tsx:23`,
  `packages/react/src/slider/thumb/SliderThumb.test.tsx:10`,
  `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.test.tsx:9`,
  `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:8`;
  `platform.engine.gecko` at
  `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.test.tsx:382`;
  `!platform.engine.blink` at `packages/react/src/dialog/root/DialogRoot.test.tsx:42` and
  `packages/react/src/menu/root/MenuRoot.test.tsx:2798`. These prove the flags are plain
  boolean values consumable in `skipIf`, not thunks or accessors. Consumption facts.
- No test or mock anywhere exercises `os.windows`, `os.linux`, `env.jsdom`, or
  `mediaQuery.iOS`.

## State model (controlled/uncontrolled, defaults, transitions)

N/A as a controlled/uncontrolled component concept. The unit has no mutable state: every
flag is a module-scope `const` computed exactly once at import time (`Static platform
detection, evaluated once at module load` — `packages/utils/src/platform/index.ts:2`;
destructure of the raw data happens at module scope —
`packages/utils/src/platform/shared.ts:48-51`). Consequently there are no defaults, no
transitions, and no reactivity: later mutations of `navigator` (or UA overrides after
import) have no effect on already-computed flags. UNVERIFIED — inferred from
`packages/utils/src/platform/shared.ts:48-51`, no test asserts import-time evaluation or
immunity to post-import UA changes.

## Keyboard interactions

N/A — non-visual, non-DOM detection utility. No test asserts any keyboard behavior (no test
file exists for this unit).

## Focus management

N/A — the unit manages no focus and holds no DOM references
(`packages/utils/src/platform/index.ts:32`, `packages/utils/src/platform/parts.ts:1-5`).

## Accessibility (roles, aria-*, id linking)

N/A — the unit renders nothing and sets no ARIA attributes. Its only accessibility-adjacent
surface is the `screenReader.voiceOver` flag, which downstream components use to gate
AT-specific workarounds (e.g. the VoiceOver submenu-trigger tests mock it:
`packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.voiceOver.test.tsx:8-18`).
The unit itself performs no ARIA linking; whether VoiceOver is actually running is
explicitly documented as undetectable
(`packages/utils/src/platform/screen-reader.ts:3-7`). UNVERIFIED — inferred from
`packages/utils/src/platform/screen-reader.ts:3-7`, no test asserts the call-site-gating
guidance.

## DOM structure & portal behavior

N/A — the utility renders nothing, creates no DOM nodes, and performs no portal behavior
(`packages/utils/src/platform/index.ts:32`, `packages/utils/src/platform/parts.ts:1-5`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — no events are emitted, dispatched, listened for, or prevented anywhere in the unit
(`packages/utils/src/platform/index.ts:32`, `packages/utils/src/platform/os.ts:1-24`,
`packages/utils/src/platform/engine.ts:1-22`). The `mediaQuery.iOS` value is a CSS query
_string_ evaluated by the browser at style-resolution time, not a JS media-query listener —
`packages/utils/src/platform/index.ts:6-7`. UNVERIFIED — inferred from
`packages/utils/src/platform/index.ts:6-7`, no test asserts the absence of listeners.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid interactions / unmount / nesting: N/A — there is no instance, lifecycle, or
  component tree involvement; the namespace is static module data
  (`packages/utils/src/platform/index.ts:2`).
- SSR (`typeof navigator === 'undefined'`): raw reads return `{ userAgent: '', platform:
'', maxTouchPoints: 0 }` (`packages/utils/src/platform/shared.ts:24-26`), so UA/platform
  substring checks match nothing and every derived flag is `false`; the module doc states
  the `os`/`engine`/`screenReader`/`env` groups are SSR-safe with all flags false, while
  `mediaQuery` stays a constant string in every environment
  (`packages/utils/src/platform/index.ts:4-7`). UNVERIFIED — inferred from
  `packages/utils/src/platform/shared.ts:24-26` and
  `packages/utils/src/platform/index.ts:4-7`, no test asserts SSR behavior.
- iPadOS 13+ misreporting: `navigator.platform === 'MacIntel'` with `maxTouchPoints > 1` is
  classified as iOS, not macOS (tracked in mui/base-ui#1309) —
  `packages/utils/src/platform/os.ts:3-8`. UNVERIFIED — inferred from
  `packages/utils/src/platform/os.ts:3-8`, no test asserts it.
- Engine exclusivity: `webkit` is checked first via CSS support, and `gecko`/`blink` are
  both anchored to `!webkit`, so a WebKit browser can never be classified Gecko or Blink
  (`packages/utils/src/platform/engine.ts:10-15`,
  `packages/utils/src/platform/engine.ts:17-22`). `gecko` and `blink` are not mutually
  exclusive with each other by construction (a UA containing both `firefox` and `chrom`
  would set both). UNVERIFIED — inferred from `packages/utils/src/platform/engine.ts:15`
  and `packages/utils/src/platform/engine.ts:22`, no test asserts any combination.
- Test-environment interplay: the WebKit/Gecko-specific consumer suites gate on engine
  flags in combination with the shared harness's `isJSDOM` (e.g. `skipIf(isJSDOM || webkit)`
  — `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.gecko.test.tsx:21`,
  `skipIf(isJSDOM || !platform.engine.blink)` —
  `packages/react/src/dialog/root/DialogRoot.test.tsx:42`), implying the flags hold stable
  values across the repo's Chromium/Firefox/Webkit-WebKit runners. UNVERIFIED as a flag
  value — inferred from the cited gating lines; no assertion checks the jsdom value of any
  flag.
- Consumers of `env.jsdom` short-circuit browser-only code paths under test
  (`packages/react/src/floating-ui-react/utils/event.ts:26`,
  `packages/react/src/floating-ui-react/utils/element.ts:73`,
  `packages/react/src/utils/getPseudoElementBounds.ts:36`). Consumption facts, not behavior
  proofs of this unit.
- In-repo consumers (import-line consumption facts):
  `packages/react/src/drawer/root/DrawerRoot.tsx:8`,
  `packages/react/src/scroll-area/viewport/ScrollAreaViewport.tsx:5`,
  `packages/react/src/select/popup/SelectPopup.tsx:5`,
  `packages/react/src/utils/FocusGuard.tsx:4`,
  `packages/react/src/utils/getPseudoElementBounds.ts:2`,
  `packages/react/src/utils/useOpenInteractionType.ts:5`,
  `packages/react/src/floating-ui-react/components/FloatingFocusManager.tsx:11`,
  `packages/react/src/floating-ui-react/hooks/useDismiss.ts:17`,
  `packages/react/src/floating-ui-react/hooks/useFocus.ts:4`,
  `packages/react/src/floating-ui-react/hooks/useListNavigation.ts:8`,
  `packages/react/src/floating-ui-react/utils/event.ts:1`,
  `packages/react/src/floating-ui-react/utils/element.ts:2`,
  `packages/react/src/number-field/root/NumberFieldRoot.tsx:12`,
  `packages/react/src/number-field/scrub-area/NumberFieldScrubArea.tsx:7`,
  `packages/react/src/number-field/scrub-area-cursor/NumberFieldScrubAreaCursor.tsx:4`,
  `packages/react/src/menu/item/useMenuItemCommonProps.ts:3`,
  `packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.tsx:7`,
  `packages/react/src/combobox/input/ComboboxInput.tsx:4`,
  `packages/react/src/combobox/utils/useInitialLiveRegionTextMutation.ts:3`,
  `docs/src/components/SearchControls.tsx:7`,
  `docs/src/components/Demo/Demo.tsx:20`.

## Shared harness dependencies

None. No test files exist for this unit, so no `#test-utils`, `packages/react/test`, or
other shared-harness dependency is possible, and the shared harness itself contains no
reference to this unit. The consumer tests listed above use the shared `#test-utils`
renderer but only as consumers of _other_ units; this unit neither imports nor is imported
by the harness. Should a test file be added later at
`packages/utils/src/platform/*.test.ts(x)`, this spec must be re-mined.
