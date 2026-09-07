# Avatar docs page content spec

Mined from `docs/src/app/(docs)/react/components/avatar/page.mdx` only. The component's own
behavior is covered by `specs/library/avatar/behavior.md` and is referenced here by section
name instead of being restated. Demo source files under `docs/src/app/(docs)/react/components/avatar/demos/`
are out of scope for this file (Stage 2 mines them into `specs/docs-content/avatar/demos.json`).

## Page structure (headings, in order)

- `# Avatar` (h1) — `docs/src/app/(docs)/react/components/avatar/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/avatar/page.mdx:13`
- `## Optimized and lazy-loaded images` — `docs/src/app/(docs)/react/components/avatar/page.mdx:26`
  - `### Stacking` — `docs/src/app/(docs)/react/components/avatar/page.mdx:41`
  - `### Server rendering` — `docs/src/app/(docs)/react/components/avatar/page.mdx:68`
- `## API reference` — `docs/src/app/(docs)/react/components/avatar/page.mdx:72`
  - `### Root` — `docs/src/app/(docs)/react/components/avatar/page.mdx:76`
  - `### Image` — `docs/src/app/(docs)/react/components/avatar/page.mdx:80`
  - `### Fallback` — `docs/src/app/(docs)/react/components/avatar/page.mdx:84`
- `## Additional types` — `docs/src/app/(docs)/react/components/avatar/page.mdx:90` (preceded by
  the `[//]: # '@exclude-table-of-contents'` marker, which removes this section from the page's
  table of contents) — `docs/src/app/(docs)/react/components/avatar/page.mdx:88`

Non-heading page furniture, in document order:

- `<Subtitle>` — "An easily stylable avatar component." — `docs/src/app/(docs)/react/components/avatar/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React avatar component that is easy to customize." — `docs/src/app/(docs)/react/components/avatar/page.mdx:4-7`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/avatar/page.mdx:9-11`
- `TypesAvatar` / `TypesAvatarAdditional` import for the API reference and Additional types
  sections — `docs/src/app/(docs)/react/components/avatar/page.mdx:74`
- Trailing `export const metadata` SEO keywords block (13 keywords, e.g. 'React Avatar',
  'Initials Fallback', 'Accessible Avatar') — `docs/src/app/(docs)/react/components/avatar/page.mdx:94-110`

## Prose claims about component behavior

- Anatomy usage guidance: "Import the component and assemble its parts", showing the assembly
  `Avatar.Root` > `Avatar.Image` (with an empty `src`) alongside `Avatar.Fallback` ("LT"),
  imported from the `@base-ui/react/avatar` namespace. The part set and namespace import match
  behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/avatar/page.mdx:15-24`
- "By default, `<Avatar.Image>` preloads `src` and renders the image only once it has loaded."
  Matches behavior.md "DOM structure & portal behavior" (default mode: a detached `window.Image`
  probe carries the load and the rendered `<img>` is not in the DOM while loading) and
  "State model (controlled/uncontrolled, defaults, transitions)" (`'loading'` in default mode →
  image element unmounted). The follow-on clause — "This doesn't compose with image optimizers
  such as `next/image`, which serve a different URL than the raw `src`, or with
  `loading="lazy"`" — is a docs-only compatibility claim with no counterpart in behavior.md; see
  Discrepancies. `docs/src/app/(docs)/react/components/avatar/page.mdx:28`
- "Add the `keepMounted` prop to render the image element right away and let it load in place.
  Only the image that is actually displayed is requested". The keepMounted semantics match
  behavior.md "DOM structure & portal behavior" (with `keepMounted` the `<img>` is in the DOM
  with its `src` while loading, alongside the visible fallback, and no detached probe image is
  ever constructed). The "only the image that is actually displayed is requested" clause implies
  the default-mode probe request may not be reused for the displayed image; behavior.md does not
  address network request reuse — see Discrepancies.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:30`
- Stacking a11y claim: "With `keepMounted`, the image and the fallback are both present until the
  image loads. The image is hidden from assistive technology until then, so the fallback provides
  the accessible name on its own." Matches behavior.md "Accessibility (roles, aria-*, id
  linking)" (while not loaded the `<img>` is `aria-hidden="true"`, no `img` role is exposed, and
  the fallback owns the avatar's accessible name) and "DOM structure & portal behavior"
  (`keepMounted` keeps both elements present while loading).
  `docs/src/app/(docs)/react/components/avatar/page.mdx:43`
- Stacking order guidance: "place `<Avatar.Image>` after `<Avatar.Fallback>`. Both are
  positioned, so whichever comes later in the DOM paints on top. The fallback then shows through
  until the image covers it." Pure CSS stacking guidance; behavior.md records no positioning or
  paint-order requirements (its DOM coverage is element types, mounting, and status attributes
  only). Not contradicted; see the Anatomy-order note under Discrepancies.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:45`
- "A loading image paints nothing, so the fallback shows through on its own. An image that failed
  to load paints a broken-image icon on top of it. Hide the image in either state with the
  `data-loading` and `data-error` attributes". The `data-loading`/`data-error` attributes match
  behavior.md "State model (controlled/uncontrolled, defaults, transitions)" (`keepMounted` keeps
  the `<img>` mounted while loading and after an error, with the status attributes carrying the
  state) and "Accessibility (roles, aria-*, id linking)" (after an error the image stays
  `aria-hidden="true"` with `data-error` set). The "paints nothing" / "broken-image icon"
  rendering claims are browser-behavior assertions with no test counterpart in behavior.md; see
  Discrepancies. `docs/src/app/(docs)/react/components/avatar/page.mdx:47`
- "Avoid `display: none` here: an element without a box never intersects the viewport, so
  `loading="lazy"` would never fetch the image. `visibility` and `opacity` both keep lazy loading
  working." CSS authoring guidance constraining user styles, tied to the same `loading="lazy"`
  composition claim flagged above; no counterpart in behavior.md.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:66`
- Server rendering claim: "With `keepMounted`, the image is part of the server-rendered HTML and
  starts loading before hydration. So is the fallback, which stays visible until hydration
  resolves the loading status. A cached image is displayed immediately, without an enter
  animation." Matches behavior.md "Accessibility (roles, aria-*, id linking)" (SSR: the server
  HTML contains the `aria-hidden` image plus the visible fallback; after hydration the layout
  effect resolves the status before paint, so the fallback is removed without a flash) and
  "Edge cases (rapid interactions, unmount, nesting)" (SSR hydration with a cached image: no
  fallback flash and the enter animation is not replayed — no `data-starting-style`).
  `docs/src/app/(docs)/react/components/avatar/page.mdx:70`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents the three parts, each as its own heading rendering a
  generated reference component: `### Root` → `<TypesAvatar.Root />`,
  `### Image` → `<TypesAvatar.Image />`, `### Fallback` → `<TypesAvatar.Fallback />`.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:72-86`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesAvatar, TypesAvatarAdditional } from './types';`); no props, prop types,
  defaults, or prop descriptions are written inline in the .mdx source of this page, and the page
  contains no literal Markdown API tables.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:74`
- Parts documented on this page: Root, Image, Fallback — the same three-part set recorded in
  behavior.md "Public API surface (props, parts, subcomponents)".
  `docs/src/app/(docs)/react/components/avatar/page.mdx:76-86`
- The `## Additional types` section renders
  `<TypesAvatarAdditional showAdditionalTypes={['imageloadingstatus']} />`, documenting the image
  loading status union. behavior.md "State model (controlled/uncontrolled, defaults,
  transitions)" records the status values observed in tests: `'loading'`, `'loaded'`, `'error'`,
  and `'idle'`. `docs/src/app/(docs)/react/components/avatar/page.mdx:90-92`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- **"Anatomy"** (jsx): imports `{ Avatar }` from `@base-ui/react/avatar` and assembles
  `<Avatar.Root>` containing `<Avatar.Image src="" />` followed by
  `<Avatar.Fallback>LT</Avatar.Fallback>`.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:17-24`
- **"Using next/image"** (jsx): imports `Image` from `next/image` and assembles `<Avatar.Root>`
  with `<Avatar.Fallback>LT</Avatar.Fallback>` placed before
  `<Avatar.Image keepMounted render={<Image src="/avatar.png" width={32} height={32} alt="" />} />`
  (Fallback-before-Image order, consistent with the Stacking guidance).
  `docs/src/app/(docs)/react/components/avatar/page.mdx:32-39`
- **"Stacked image and fallback"** (css): `.Root` gets `position: relative`; `.Image` and
  `.Fallback` are both `position: absolute; inset: 0`; `.Image[data-loading]` and
  `.Image[data-error]` get `visibility: hidden`.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:49-64`
- No other code blocks exist in the page. The hero demo at the top is a rendered component
  (`<DemoAvatarHero />` imported from `./demos/hero`); its code lives in demo files and is out of
  scope here (Stage 2). `docs/src/app/(docs)/react/components/avatar/page.mdx:9-11`

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/avatar/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: default-mode `<Avatar.Image>` "doesn't compose
  with image optimizers such as `next/image`, which serve a different URL than the raw `src`, or
  with `loading="lazy"`. behavior.md records no test touching next/image, image optimizers, or
  native lazy loading. `docs/src/app/(docs)/react/components/avatar/page.mdx:28`
- Docs-only claim, not covered by behavior.md: with `keepMounted`, "Only the image that is
  actually displayed is requested". behavior.md "DOM structure & portal behavior" verifies the
  detached probe exists in default mode and is never constructed with `keepMounted`, but no test
  asserts anything about network request reuse or duplicate fetching.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:30`
- Docs-only claims, not covered by behavior.md: "A loading image paints nothing" and "an image
  that failed to load paints a broken-image icon on top of it". behavior.md only verifies the
  `data-loading`/`data-error` attributes and the aria/visibility semantics; no test asserts
  browser rendering of empty or broken images. `docs/src/app/(docs)/react/components/avatar/page.mdx:47`
- Docs-only claim, not covered by behavior.md: `display: none` prevents `loading="lazy"` from
  ever fetching the image because a box-less element never intersects the viewport, while
  `visibility`/`opacity` keep lazy loading working. behavior.md has no lazy-loading viewport
  coverage. `docs/src/app/(docs)/react/components/avatar/page.mdx:66`
- Omission (behavior.md documents, page never mentions): the `delay` prop on `Avatar.Fallback`;
  the `onLoadingStatusChange` callback; user `onLoad`/`onError` forwarding and the
  `preventBaseUIHandler` cancellation mechanism ("Events (names, payload shape, bubbling,
  preventDefault semantics)"); preservation of an explicitly provided `aria-hidden` value;
  `data-starting-style`/`data-ending-style` animation hooks; and `srcSet`/`sizes` source
  handling ("Public API surface (props, parts, subcomponents)"). Omissions of documented API and
  caveats, not contradictions of anything the page does say.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:26-70`
- Note (not a mismatch): the Anatomy snippet places `<Avatar.Image>` before `<Avatar.Fallback>`
  while the Stacking guidance instructs placing `<Avatar.Image>` after `<Avatar.Fallback>`. The
  latter is scoped to the `keepMounted` stacked-layout pattern, so the two orderings address
  different scenarios; behavior.md prescribes neither order.
  `docs/src/app/(docs)/react/components/avatar/page.mdx:20-23`, `docs/src/app/(docs)/react/components/avatar/page.mdx:45`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages, and no external
  links either (the `next/image` references are package mentions inside prose and a code snippet,
  not hyperlinks). `docs/src/app/(docs)/react/components/avatar/page.mdx:28`, `docs/src/app/(docs)/react/components/avatar/page.mdx:32-39`
- The demo component (`./demos/hero`) is imported and rendered on this page itself; it is a
  same-page import, not a cross-page link. `docs/src/app/(docs)/react/components/avatar/page.mdx:9-11`
