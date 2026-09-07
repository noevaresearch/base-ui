# CSPProvider — behavior spec

Mined Stage 1 from test files only. Unit: `csp-provider` (infra unit; React source in `packages/react/src/csp-provider/`). The TODO.md entry (`TODO.md:237-242`) has no `wraps-external:` field, so no third-party delegation applies — behavior below is fully scoped to this unit's own test file.

Suite size: one test file, 4 tests, all DOM-query based (`packages/react/src/csp-provider/CSPProvider.test.tsx`).

## Public API surface (props, parts, subcomponents)

- `CSPProvider` is exported from `@base-ui/react/csp-provider` and is a component that wraps children. `packages/react/src/csp-provider/CSPProvider.test.tsx:4`, `packages/react/src/csp-provider/CSPProvider.test.tsx:19-23`
- Prop `disableStyleElements` (boolean): suppresses injection of Base UI's inline `<style>` elements for the wrapped subtree — asserted for the `.base-ui-disable-scrollbar` stylesheet emitted by ScrollArea (`packages/react/src/csp-provider/CSPProvider.test.tsx:17-27`) and by Select (`packages/react/src/csp-provider/CSPProvider.test.tsx:29-48`).
- Prop `nonce` (string): value is placed on the injected inline `<style>` element as a `nonce` attribute (`nonce="test-nonce"` asserted). `packages/react/src/csp-provider/CSPProvider.test.tsx:50-62`
- No parts/subcomponents are observed in tests; the unit's public surface exercised by tests is the single `CSPProvider` component with the two props above. UNVERIFIED — additional props (if any) are inferred from nothing in the suite; no test asserts them.

## State model (controlled/uncontrolled, defaults, transitions)

N/A — no observable state, controlled/uncontrolled props, or state transitions are asserted anywhere in the suite. The only default proven by tests: `disableStyleElements` is off by default (inline styles are injected when no provider or a plain provider wraps the tree). `packages/react/src/csp-provider/CSPProvider.test.tsx:64-75`

## Keyboard interactions

N/A — the component is a style-injection context; no keyboard behavior exists or is tested.

## Focus management

N/A — no focus behavior exists or is tested.

## Accessibility (roles, aria-*, id linking)

N/A — no roles, `aria-*` attributes, or id linking are asserted for the provider or its injected styles. The provider itself is never queried as a DOM node by any test, so whether it renders a wrapper element is UNVERIFIED — inferred from `packages/react/src/csp-provider/CSPProvider.test.tsx:14-76`, no test asserts this.

## DOM structure & portal behavior

- The provider governs inline `<style>` elements present in the document (tests query `document.querySelectorAll('style')` and look for text content containing the class name `.base-ui-disable-scrollbar`). `packages/react/src/csp-provider/CSPProvider.test.tsx:7-12`
- Default behavior: rendering `ScrollArea.Root` + `ScrollArea.Viewport` produces a document-level `<style>` element containing `.base-ui-disable-scrollbar`. `packages/react/src/csp-provider/CSPProvider.test.tsx:64-75`
- With `disableStyleElements`, that `<style>` element is entirely absent from the document. `packages/react/src/csp-provider/CSPProvider.test.tsx:17-27`
- The suppression reaches content rendered through `Select.Portal` (Root `defaultOpen`, Trigger/Value, Portal/Positioner/Popup/Item): the `.base-ui-disable-scrollbar` style is still absent from the document even though popup content is portaled. `packages/react/src/csp-provider/CSPProvider.test.tsx:29-48`
- With `nonce`, the injected `<style>` carries that nonce as an attribute — relevant for CSP headers that require a nonce on inline styles. `packages/react/src/csp-provider/CSPProvider.test.tsx:50-62`
- React 19 hoists inline `<style>` elements; the suite comment notes the style "already exists from previous test due to React 19's hoisting", i.e. injected styles persist in the document across renders. `packages/react/src/csp-provider/CSPProvider.test.tsx:71-74`

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the component emits no events and none are asserted.

## Edge cases (rapid interactions, unmount, nesting)

- Rapid interactions: N/A (no interactive behavior under test).
- Style hoisting persistence: inline styles injected once remain in the document across subsequent renders (observed as cross-test persistence under React 19 hoisting). `packages/react/src/csp-provider/CSPProvider.test.tsx:71-74`
- UNVERIFIED — behavior on unmount of the provider (whether injected styles are removed) is inferred from nothing in `packages/react/src/csp-provider/CSPProvider.test.tsx:14-76`; no test asserts this.
- UNVERIFIED — nesting multiple providers (inheritance/override of `nonce` and `disableStyleElements`) is inferred from nothing in the same suite; no test asserts this.

## Shared harness dependencies

- The suite imports `createRenderer` from `#test-utils`, which maps to `packages/react/test/index.ts` (`packages/react/package.json:110`); that entry re-exports `@base-ui/utils/testUtils` plus local harness modules (`packages/react/test/index.ts:1-14`).
- `createRenderer` is defined in `packages/react/test/createRenderer.ts:27-49`: it wraps `@mui/internal-test-utils`'s `createRenderer`, running each `render` inside `act` and adding act-wrapped `rerender`/`setProps` helpers. The CSPProvider suite uses only the `render` helper (`packages/react/src/csp-provider/CSPProvider.test.tsx:15`).
