# Avatar — behavior spec (Stage 1: behavior mining)

Mined from the unit's own test suite only:

- `packages/react/src/avatar/root/AvatarRoot.test.tsx`
- `packages/react/src/avatar/image/AvatarImage.test.tsx`
- `packages/react/src/avatar/fallback/AvatarFallback.test.tsx`

The unit has no `wraps-external:` field in its TODO entry (no third-party package is delegated to).

## Public API surface (props, parts, subcomponents)

- All parts are imported from a single `Avatar` barrel (`@base-ui/react/avatar`): `Avatar.Root`, `Avatar.Image`, `Avatar.Fallback`. `packages/react/src/avatar/root/AvatarRoot.test.tsx:2`, `packages/react/src/avatar/image/AvatarImage.test.tsx:3`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:3`
- `Avatar.Root`: container element that forwards its ref to an `HTMLSpanElement` (shared conformance suite asserts `refInstanceof: window.HTMLSpanElement`). `packages/react/src/avatar/root/AvatarRoot.test.tsx:8-11`
- `Avatar.Image` props proven by tests:
  - Native `<img>` props are forwarded to the rendered element as attributes: `crossOrigin` → `crossorigin`, `referrerPolicy` → `referrerpolicy`, `sizes`, `srcSet` → `srcset`, plus `src`. `packages/react/src/avatar/image/AvatarImage.test.tsx:102-121`
  - `srcSet` alone (no `src`) is sufficient for the loaded image to render and the fallback to be absent. `packages/react/src/avatar/image/AvatarImage.test.tsx:123-133`
  - `sizes` / `srcSet` / `src` are also forwarded to the detached loading-probe image (see DOM structure). `packages/react/src/avatar/image/AvatarImage.test.tsx:135-147`
  - `keepMounted` (boolean) — keeps the `<img>` element mounted across not-loaded statuses. `packages/react/src/avatar/image/AvatarImage.test.tsx:220-234`
  - `onLoadingStatusChange(status)` — status-change callback. `packages/react/src/avatar/image/AvatarImage.test.tsx:149-174`
  - `onLoad`, `onError` — user event handlers forwarded to the rendered element. `packages/react/src/avatar/image/AvatarImage.test.tsx:290-306`, `packages/react/src/avatar/image/AvatarImage.test.tsx:308-326`
  - `render` — accepts a replacement element (`render={<img ... />}`) or a render callback (`render={(props) => <img {...props} />}`). `packages/react/src/avatar/image/AvatarImage.test.tsx:351-364`, `packages/react/src/avatar/image/AvatarImage.test.tsx:691-709`
  - `alt` — used as the accessible name of the loaded image. `packages/react/src/avatar/image/AvatarImage.test.tsx:575-593`
- `Avatar.Fallback` props proven by tests:
  - `children` — rendered text content. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:53-66`
  - `delay` (number | undefined) — defers fallback appearance. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:100-201`
  - Forwards ref to an `HTMLSpanElement`. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:31-36`
- `Avatar.Image` forwards ref to an `HTMLImageElement`. `packages/react/src/avatar/image/AvatarImage.test.tsx:95-100`

## State model (controlled/uncontrolled, defaults, transitions)

- The unit is driven by an image-loading status with values observed in tests: `'loading'`, `'loaded'`, `'error'`, and `'idle'` (the mocked internal hook's default return; the `ImageLoadingStatus` type is imported from `../root/AvatarRoot`). `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:7-16`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:24`
- Status is internal state reported through `onLoadingStatusChange`; no test passes any status-controlling prop, so no controlled-status API is exercised. UNVERIFIED — inferred from the whole `onLoadingStatusChange` suite at `packages/react/src/avatar/image/AvatarImage.test.tsx:149-218`, no test asserts a controlled API.
- Initial status resolution:
  - A mount with a pending source first reports `'loading'` (probe mode). `packages/react/src/avatar/image/AvatarImage.test.tsx:160-173`
  - A cached image (`complete` + `naturalWidth > 0`) resolves to `'loaded'` synchronously in the initial layout effect and never reports `'loading'`; the cached-src JSDOM test shows the image immediately with no fallback. `packages/react/src/avatar/image/AvatarImage.test.tsx:829-837`, `packages/react/src/avatar/image/AvatarImage.test.tsx:1136-1146`
  - A cached error (`complete` + `naturalWidth === 0`) resolves to `'error'` without ever emitting `'idle'`. `packages/react/src/avatar/image/AvatarImage.test.tsx:202-217`
  - A source-less image is `complete` with `naturalWidth === 0` and is resolved to `'error'` without waiting for an event. `packages/react/src/avatar/image/AvatarImage.test.tsx:515-535`
- Transitions:
  - `'loading'` → `'loaded'` on the load event (both probe `onload` and rendered-element `load`). `packages/react/src/avatar/image/AvatarImage.test.tsx:168-173`, `packages/react/src/avatar/image/AvatarImage.test.tsx:251-259`
  - `'loading'` → `'error'` on the error event. `packages/react/src/avatar/image/AvatarImage.test.tsx:194-199`, `packages/react/src/avatar/image/AvatarImage.test.tsx:277-285`
  - Changing the source (via `src` prop or the rendered element's `src`) resets status to `'loading'` (fallback reappears), and the new load can resolve it again. `packages/react/src/avatar/image/AvatarImage.test.tsx:351-393`, `packages/react/src/avatar/image/AvatarImage.test.tsx:395-427`, `packages/react/src/avatar/image/AvatarImage.test.tsx:797-838`
  - Status is preserved across unrelated re-renders (e.g. a `className` change on the rendered element). `packages/react/src/avatar/image/AvatarImage.test.tsx:537-573`
- Fallback visibility derives from status:
  - `'loaded'` → fallback children not rendered. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:38-51`
  - `'error'` → fallback rendered. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:53-66`
  - `'loading'` (default mode) → fallback mounted, image element unmounted. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:203-238`
  - `'idle'` → fallback visible, subject to `delay`. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:105-118`
- `delay` gates fallback appearance only (it never re-hides a visible fallback): hidden until the delay elapses, `delay={0}` renders synchronously on mount, and changing a pending delay to `0` shows it immediately. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:105-118`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:120-132`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:134-153`
- `keepMounted` keeps the `<img>` element mounted while loading and after an error (status attributes carry the state instead). `packages/react/src/avatar/image/AvatarImage.test.tsx:221-234`, `packages/react/src/avatar/image/AvatarImage.test.tsx:262-288`

## Keyboard interactions

N/A — no test exercises keyboard behavior on any Avatar part.

## Focus management

N/A — no test asserts focus behavior; the shared conformance suite only checks ref forwarding (`packages/react/src/avatar/root/AvatarRoot.test.tsx:8-11`, `packages/react/src/avatar/image/AvatarImage.test.tsx:95-100`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:31-36`).

## Accessibility (roles, aria-*, id linking)

- While the image is not loaded, the `<img>` is `aria-hidden="true"`, no `img` role is exposed, and the fallback owns the avatar's accessible name. `packages/react/src/avatar/image/AvatarImage.test.tsx:575-593`
- After load, `aria-hidden` is removed and the image is exposed as `role="img"` named by its `alt`. `packages/react/src/avatar/image/AvatarImage.test.tsx:587-593`
- After an error the image stays `aria-hidden="true"` (with `data-error` set) and the fallback remains visible. `packages/react/src/avatar/image/AvatarImage.test.tsx:595-613`
- The image becomes `aria-hidden="true"` again when its source changes after having been loaded. `packages/react/src/avatar/image/AvatarImage.test.tsx:615-641`
- An explicitly provided `aria-hidden` value is preserved and never overridden by the internal logic (even after load). `packages/react/src/avatar/image/AvatarImage.test.tsx:643-665`
- SSR: the server HTML contains the `aria-hidden` image plus the visible fallback (fallback owns the name until hydration); after hydration the layout effect sees `image.complete` and resolves the status before paint, so the fallback is removed without a flash and the image is exposed as `role="img"` named by `alt`. `packages/react/src/avatar/image/AvatarImage.test.tsx:444-478`
- No id-based aria linking between Root/Image/Fallback is asserted by any test (naming comes from the fallback's text content and the image's `alt`). UNVERIFIED — inferred from the suite, no test asserts id linking.

## DOM structure & portal behavior

- Element mapping: Root renders a `<span>`, Fallback renders a `<span>`, Image renders an `<img>` (ref conformance). `packages/react/src/avatar/root/AvatarRoot.test.tsx:8-11`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:31-36`, `packages/react/src/avatar/image/AvatarImage.test.tsx:95-100`
- Default mode (no `keepMounted`): while loading, the `<img>` element is not in the DOM — a detached `window.Image` probe carries the load instead, and it receives `sizes`, `srcset`, and `src`. `packages/react/src/avatar/image/AvatarImage.test.tsx:135-147`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:228-236`
- With `keepMounted`: the `<img>` is in the DOM with its `src` while loading, alongside the visible fallback, and no detached probe image is ever constructed. `packages/react/src/avatar/image/AvatarImage.test.tsx:221-234`, `packages/react/src/avatar/image/AvatarImage.test.tsx:480-513`
- No portal: every test renders the parts as children of `Avatar.Root` and queries them via `screen`; no test exercises or references portal behavior. UNVERIFIED — inferred from the test structure, no test asserts absence of portaling.
- Animation hooks (Chromium-only tests, `BASE_UI_ANIMATIONS_DISABLED = false`):
  - Mounting applies `data-starting-style` to the image; `getAnimations` is not consulted for the enter transition. `packages/react/src/avatar/image/AvatarImage.test.tsx:846-909`
  - Unmounting applies `data-ending-style` before the element is removed. `packages/react/src/avatar/image/AvatarImage.test.tsx:911-963`
  - Without `keepMounted`, the not-loaded attributes (`data-loading` / `data-error`) are not applied while the image animates out (the element only exists once loaded). `packages/react/src/avatar/image/AvatarImage.test.tsx:965-1015`
  - With `keepMounted` the element never unmounts, so `data-ending-style` is never applied; `data-loading` carries the state instead. `packages/react/src/avatar/image/AvatarImage.test.tsx:1017-1063`

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onLoadingStatusChange` is a callback prop (not a DOM event): it receives a single string status argument. Observed payloads are exactly `'loading'` then `'loaded'` on load, and `'loading'` then `'error'` on error. `packages/react/src/avatar/image/AvatarImage.test.tsx:160-173`, `packages/react/src/avatar/image/AvatarImage.test.tsx:194-199`
- On the cached-error path no `'idle'` value is ever emitted. `packages/react/src/avatar/image/AvatarImage.test.tsx:202-217`
- Bubbling: N/A — the status change is a React callback; no test asserts DOM-event bubbling for it.
- Cancellation semantics: the user's `onLoad` can call `event.preventBaseUIHandler()` to cancel the internal status update — the image keeps `data-loading`, the fallback stays mounted, and `'loaded'` is never reported. `packages/react/src/avatar/image/AvatarImage.test.tsx:328-349`
- User `onError` / `onLoad` handlers are still invoked (once each) in addition to the status machinery. `packages/react/src/avatar/image/AvatarImage.test.tsx:290-306`, `packages/react/src/avatar/image/AvatarImage.test.tsx:308-326`

## Edge cases (rapid interactions, unmount, nesting)

- Unmounting a loaded image makes the fallback reappear. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:68-98`
- Exactly one of image or fallback is mounted when switching to the image, even while a 2s exit animation runs on the fallback (regression test). `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:240-296`
- `delay` lifecycle: changing a pending delay to `undefined` shows the fallback immediately; restoring a number afterwards must not re-hide the already-visible fallback; `undefined → number` also keeps a visible fallback visible. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:176-200`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:155-174`
- Rapid source swaps reset and re-resolve the status: a cached first source reports only `['loaded']`, then a swap to an unloaded source reports `['loaded', 'loading', 'error']`. `packages/react/src/avatar/image/AvatarImage.test.tsx:797-838`
- Repeated status resets across re-renders: after a rendered-element src change resets to `'loading'`, the new load still resolves to `'loaded'`. `packages/react/src/avatar/image/AvatarImage.test.tsx:351-393`
- A rendered element that drops the ref: no crash, and a previously reported `'loaded'` status is never overwritten (no spurious `'loading'` that would flash the fallback). `packages/react/src/avatar/image/AvatarImage.test.tsx:747-795`
- SSR hydration with a cached image: no fallback flash — status resolves on the first post-hydration render (asserted synchronously, no `waitFor`), and the enter animation is not replayed (no `data-starting-style`). `packages/react/src/avatar/image/AvatarImage.test.tsx:1095-1133`, `packages/react/src/avatar/image/AvatarImage.test.tsx:1065-1092`
- Source-props ordering in the render callback: request-configuring props (`loading`, `sizes`, `srcSet`) are applied before `src` so browsers start fetching with the full request configuration. `packages/react/src/avatar/image/AvatarImage.test.tsx:717-745`
- Source props supplied inside a render callback are not overridden by the component. `packages/react/src/avatar/image/AvatarImage.test.tsx:691-715`

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts` (packages/react/package.json:110). Avatar tests use `createRenderer` (providing `render`, `renderToString`/`hydrate`, `user`, `setProps`, and the fake-timer `clock` with `clock.withFakeTimers()`), `describeConformance`, and `isJSDOM` (used to split JSDOM-only from Chromium-only tests via `it.skipIf`). `packages/react/src/avatar/root/AvatarRoot.test.tsx:3`, `packages/react/src/avatar/image/AvatarImage.test.tsx:5`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:5`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:101-103`
- `@mui/internal-test-utils` (npm package) provides `act`, `fireEvent`, `screen`, `waitFor`. `packages/react/src/avatar/image/AvatarImage.test.tsx:4`, `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:4`
- The Fallback tests mock the unit-internal hook `useImageLoadingStatus` (`vi.mock`) to drive statuses deterministically; the mock reveals the hook returns a `[status, setter]` tuple. `packages/react/src/avatar/fallback/AvatarFallback.test.tsx:9-16`
- Image-loading tests stub `window.Image` with a configurable mock (`completeOnSet`, `naturalWidth`) to simulate cached vs. async image loading. `packages/react/src/avatar/image/AvatarImage.test.tsx:27-73`
