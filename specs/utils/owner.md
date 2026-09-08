# `owner` — behavior spec

Unit: `packages/utils/src/owner` (Phase A util → crate `leptos-ui-utils`).

Source of truth: **none** — this unit has no test file. The generated unit manifest records
`testFiles: []` with a single srcFile for it (`ralph/generated/utils.json:168-174`), and a
repo-wide search finds no `*.test.*` file importing `@base-ui/utils/owner` directly. The unit's
`TODO.md` entry has no `wraps-external:` field (`TODO.md:99-103`). Every behavioral claim below
is therefore UNVERIFIED by tests: implementation-derived claims cite the unit's own source, and
consumer-behavior claims cite production call sites framed as indirect evidence. Stage 3 should
not treat this spec as test-proven behavior; for `ownerDocument` it should encode the fallback
semantics as its own Rust tests rather than binding to a reference suite.

## Public API surface (props, parts, subcomponents)

- Single module with exactly two named exports and no default export: `ownerWindow` and
  `ownerDocument`. No components, no props objects, no parts, no subcomponents.
  UNVERIFIED — inferred from `packages/utils/src/owner.ts:1-5`, no test asserts this.
- `ownerWindow` is a pure re-export alias of the third-party `getWindow` from
  `@floating-ui/utils/dom` — one line, no local logic
  (`packages/utils/src/owner.ts:1`). The underlying algorithm is delegated to that external
  package; its public type signature is `(node: any) => typeof window`
  (`node_modules/.pnpm/@floating-ui+utils@0.2.12/node_modules/@floating-ui/utils/dist/floating-ui.utils.dom.d.ts:23`).
  Nothing in this unit's own source states or asserts its runtime behavior. For Stage 3: the
  Rust equivalent crate already named for the floating-ui family is `floating-ui-leptos`
  (`TODO.md:314-314`), a natural bind target for the window-resolution semantics instead of
  reimplementing them — note this unit's own TODO entry has no `wraps-external:` field
  (`TODO.md:99-103`), so this is an observation from the code, not a recorded delegation.
- `ownerDocument(node: Element | null)` — a locally defined function taking one positional
  argument typed `Element | null` and returning a `Document`. The `?.` in the body also makes
  `undefined` safe at runtime despite the declared type. UNVERIFIED — inferred from
  `packages/utils/src/owner.ts:3-4`, no test asserts this.
- Import surface: consumed in-repo as `@base-ui/utils/owner`, which resolves through the
  wildcard entry `"./*": "./src/*.ts"` in the `@base-ui/utils` export map
  (`packages/utils/package.json:12-16`; the same map nulls out `./*.test` imports).
  It is a widely-used shared utility — roughly 50 in-repo source files import from it; two
  representative import sites: `packages/react/src/tabs/tab/TabsTab.tsx:3`,
  `packages/react/src/drawer/root/DrawerRoot.tsx:7`. Not re-exported from `@base-ui/react`;
  no test imports the module by name (UNVERIFIED for the "not re-exported" claim — inferred
  from the unit's 5-line source having no barrel, `packages/utils/src/owner.ts:1-5`).

## State model (controlled/uncontrolled, defaults, transitions)

N/A — two stateless pure functions with no controlled/uncontrolled duality, defaults, or
transitions, and no module-level mutable state (contrast `generateId`, which keeps a counter).
Both calls are resolved per-invocation from their argument. UNVERIFIED — inferred from
`packages/utils/src/owner.ts:1-5`, no test asserts this.

## Keyboard interactions

N/A — non-visual DOM-realm resolution utility. No keyboard behavior is implemented, and no
test or call site suggests any.

## Focus management

N/A — the unit performs no focusing. Indirect evidence of its role in focus flows: consumers
use it to resolve the correct document before comparing active elements, e.g.
`activeElement(ownerDocument(target)) === target`
(`packages/react/src/drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.tsx:792`).
That focus-logic behavior is the consumer's, not this unit's.

## Accessibility (roles, aria-*, id linking)

N/A — sets no roles, aria attributes, or id links. Its accessibility relevance is indirect:
it exists so component code resolves `document`/`window` through a DOM node (correct realm
under iframes/shadow roots) instead of touching browser globals — e.g. event/portal code
resolving the document from the event target or popup element
(`packages/react/src/utils/useSwipeDismiss.ts:923`,
`packages/react/src/toast/root/ToastRoot.tsx:286`). No test asserts any accessibility
behavior of this unit; the realm-safety motivation itself is UNVERIFIED — inferred from the
call-site pattern, no test asserts this.

## DOM structure & portal behavior

N/A — creates no DOM, renders nothing, performs no portal behavior. Indirect: it is the
document/window resolver used by portal-rendered popup components. Caution for Stage 3:
several component test suites read the *native DOM* `element.ownerDocument` property directly
(e.g. `packages/react/src/menu/root/MenuRoot.test.tsx:1767`) — those exercise the browser
property, not this util, and must not be counted as coverage of it.

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — emits, dispatches, and handles no events. Indirect evidence of call-site usage shapes:
`ownerWindow(element).getComputedStyle(element)` for style measurement
(`packages/react/src/tabs/indicator/TabsIndicator.tsx:258`), `ownerWindow(event.currentTarget)`
passed into an event-classification helper (`packages/react/src/drawer/viewport/DrawerViewport.tsx:1126`),
and `const win = ownerWindow(popupElement)` for popup-window access
(`packages/react/src/drawer/root/DrawerRoot.tsx:452`). None of these are asserted by tests to
go through this util's behavior specifically.

## Edge cases (rapid interactions, unmount, nesting)

All claims in this section are UNVERIFIED — inferred from
`packages/utils/src/owner.ts:1-5`, no test asserts any of them:

- Null/undefined argument: `ownerDocument(null)` falls back to the global `document` via the
  `|| document` branch (`packages/utils/src/owner.ts:4`). This is the branch that makes the
  util safe for `ownerDocument(ref.current)` before mount.
- Falsy `node.ownerDocument`: a node whose `ownerDocument` is missing (e.g. a detached or
  mock node) also falls back to the global `document` (`packages/utils/src/owner.ts:4`).
- Cross-realm resolution: for a node inside an iframe or shadow root, `node.ownerDocument`
  resolves to that node's own document by native DOM semantics; the unit adds no logic beyond
  the fallback (`packages/utils/src/owner.ts:4`). No test in the repo exercises this util
  under iframes or shadow DOM.
- `ownerWindow` edge cases (null/undefined input, cross-realm windows) are owned entirely by
  `@floating-ui/utils/dom` via the re-export (`packages/utils/src/owner.ts:1`) and are
  UNVERIFIED here; do not re-derive them from Base UI's tests (there are none).
- SSR: `ownerDocument` can touch the browser-global `document` at call time
  (`packages/utils/src/owner.ts:4`) and the unit contains no SSR guard; server behavior is
  UNVERIFIED.
- Rapid interactions, unmount, nesting: N/A — stateless functions, so interleaved, repeated,
  or nested calls cannot interfere with each other
  (`packages/utils/src/owner.ts:3-5`).

## Shared harness dependencies

None. The unit has no test file at all, so no test harness is involved: no `#test-utils`
import, no file under `packages/react/test/`, and no shared utils-test harness. The module's
only import is the third-party `@floating-ui/utils/dom`
(`packages/utils/src/owner.ts:1`).
