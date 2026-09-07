# CSPProvider docs page content spec

Mined from `docs/src/app/(docs)/react/utils/csp-provider/page.mdx` only. The component's own
behavior is covered by `specs/library/csp-provider/behavior.md` and is referenced here by section
name instead of being restated. The page has no `demos/` directory (the only page-directory files
are `page.mdx`, `types.md`, and `types.ts`), so there are no demo imports or demo code on this
page; Stage 2 (`specs/docs-content/csp-provider/demos.json`) has no demo files to mine here.

## Page structure (headings, in order)

- `# CSP Provider` (h1) — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:9`
- `## Supplying a nonce` — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:26`
- `## Disable inline style elements` — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:57`
- `## Inline style attributes` — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:80`
- `## API reference` — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:92`

Non-heading page furniture, in document order:

- `<Subtitle>` — "Configures CSP-related behavior for inline tags rendered by Base UI components." — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:3`
- `<Meta name="description">` — "A CSP provider component that applies a nonce to inline <style> and <script> tags rendered by Base UI components, and can disable inline <style> elements." — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:4-7`
- `TypesCSPProvider` import for the API reference — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:94`
- Trailing `export const metadata` SEO keywords block (6 keywords, e.g. 'Base UI CSP Provider', 'Content Security Policy', 'CSP nonce', 'React CSP', 'Inline script nonce', 'Inline style nonce') — `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:98-107`

## Prose claims about component behavior

- Motivation claim: "Some Base UI components render inline `<style>` or `<script>` tags for
  functionality such as removing scrollbars or pre-hydration behavior. Under a strict Content
  Security Policy (CSP), these tags may be blocked unless they include a matching [nonce]
  attribute" (MDN link on the word "nonce"). The `<style>` side matches behavior.md
  "DOM structure & portal behavior" (document-level `.base-ui-disable-scrollbar` style injection
  for scrollbar removal); the `<script>` tags and "pre-hydration behavior" rationale are not
  exercised by any test recorded in behavior.md.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:22`
- "`CSPProvider` allows configuring this behavior globally for all Base UI components within its
  tree." Matches behavior.md "DOM structure & portal behavior": the provider governs inline
  `<style>` elements present in the document, and suppression reaches content rendered through
  `Select.Portal`.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:24`
- Server setup steps for a nonce-blocking CSP: (1) generate a random nonce per request, (2)
  include it in the CSP header via `style-src-elem`/`script-src`, (3) pass the same nonce into
  `CSPProvider` during rendering. The pass-the-nonce step matches behavior.md "Public API surface
  (props, parts, subcomponents)" (prop `nonce` (string)); the server/header steps are general CSP
  guidance with no test counterpart.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:28-32`
- Effect claim: "This will ensure that all inline `<style>` and `<script>` tags rendered by Base
  UI components include the correct nonce attribute, allowing them to function under your CSP."
  behavior.md "DOM structure & portal behavior" verifies the nonce lands on the injected
  `<style>` element only; the script-tag half is not covered by any test recorded there.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:55`
- Which components inject a style tag: "You can avoid supplying a `nonce` if you disable inline
  `<style>` elements entirely and rely on external stylesheets only. The relevant components are
  `<ScrollArea.Viewport>` and `<Select.Popup>` or `<Select.List>` when `alignItemWithTrigger` is
  enabled, which inject a style tag to disable native scrollbars." The ScrollArea and Select
  emitters match behavior.md "DOM structure & portal behavior"; the `alignItemWithTrigger` gating
  condition and the `<Select.List>` alternative are not asserted by behavior.md (its Select test
  renders a `defaultOpen` Root with Portal/Positioner/Popup and does not mention
  `alignItemWithTrigger` or `Select.List`).
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:59`
- Injected stylesheet content (html block, echoed here because the page presents it as what Base
  UI renders): a `<style>` tag defining `.base-ui-disable-scrollbar { scrollbar-width: none; }`
  and `.base-ui-disable-scrollbar::-webkit-scrollbar { display: none; }`. Matches behavior.md
  "DOM structure & portal behavior", which identifies the injected style by the
  `.base-ui-disable-scrollbar` class name.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:61-70`
- "Specify `disableStyleElements` to remove these tags". Matches behavior.md "Public API surface
  (props, parts, subcomponents)" (prop `disableStyleElements` (boolean) suppresses injection of
  the inline `<style>` elements) and "State model (controlled/uncontrolled, defaults,
  transitions)" (the flag is off by default).
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:72`
- "`<script>` tags across all components are opt-in, so they are not affected by this prop and
  don't have their own disable flag. A `nonce` is required if any component uses inline scripts."
  Not covered by behavior.md (no test touches script tags); no contradiction, but a docs-only
  claim — see Discrepancies.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:78`
- Scope claim: "`CSPProvider` covers inline `<style>` and `<script>` tags rendered as elements,
  but it does not cover inline style attributes (for example, `<div style="...">`)." Also:
  "The `style-src-attr` directive in CSP governs inline style attributes encountered when parsing
  HTML from server pre-rendered components (it does not affect client-side JavaScript that sets
  styles)." The element/attribute scope split is not covered by behavior.md (attributes are never
  tested); the `style-src-attr` semantics are general CSP guidance, not component behavior.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:82`
- "In CSP, `style-src` applies to both `<style>` elements and `style=""` attributes. If you only
  want to control `<style>` elements, use `style-src-elem` instead." General CSP guidance, not
  component behavior; N/A for behavior.md.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:84`
- Three mitigation options when a CSP blocks inline style _attributes_ in addition to _elements_:
  (1) relax the CSP by adding `'unsafe-inline'` to `style-src-attr` (or use only
  `style-src-elem` instead of `style-src`), with the caveat that style attributes "pose a less
  severe security risk than style elements" but "may not be acceptable in high-security
  environments"; (2) render the affected components only on the client so no inline styles appear
  in the initial HTML; (3) manually unset inline styles and specify them in CSS — "Any component
  can have its inline styles unset, such as `<ScrollArea.Viewport style={{ overflow: undefined
  }}>`. Note that you'll need to ensure you vet upgrades for any new inline styles added by Base
  UI components." Docs-only guidance; none of it is covered by behavior.md.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:86-90`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section renders a single generated reference component,
  `<TypesCSPProvider />`, imported from `./types`. Unlike multi-part components, there are no
  per-part headings or per-part tables. `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:92-96`
- The reference table itself is a generated component (`import { TypesCSPProvider } from
  './types';`); no props, prop types, defaults, or prop descriptions are written inline in the
  .mdx source of this page. `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:94`
- Parts documented on this page: none — there is exactly one export, the `CSPProvider` component
  itself, consistent with behavior.md "Public API surface (props, parts, subcomponents)" ("no
  parts/subcomponents are observed in tests").
- Props documented on this page (in prose, not tables): `nonce` (string, e.g. a per-request
  random value) and `disableStyleElements` (boolean). The two-prop surface matches behavior.md
  "Public API surface (props, parts, subcomponents)" exactly.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:28-32`, `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:72-78`
- Import path documented: `@base-ui/react/csp-provider`, matching behavior.md "Public API surface
  (props, parts, subcomponents)". `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:14`, `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:48`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

- ` ```jsx title="Anatomy" ` snippet: imports `{ CSPProvider }` from `@base-ui/react/csp-provider`
  and wraps the app — `<CSPProvider nonce="...">{/* Your app or a group of components
  */}</CSPProvider>` (with a `// prettier-ignore` comment).
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:13-20`
- ` ```ts title="Example" ` snippet: `const nonce = crypto.randomUUID();` followed by an example
  CSP header built from an array joined with `'; '` — `default-src 'self'`,
  `script-src 'self' 'nonce-${nonce}'`, `style-src-elem 'self' 'nonce-${nonce}'`.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:34-43`
- ` ```jsx title="Providing the nonce" ` snippet: `function App({ nonce })` returning
  `<CSPProvider nonce={nonce}>{/* ... */}</CSPProvider>`.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:47-53`
- Un-titled ` ```html ` block showing the injected stylesheet content: `.base-ui-disable-scrollbar`
  with `scrollbar-width: none;` and a `::-webkit-scrollbar { display: none; }` rule.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:61-70`
- ` ```jsx title="Disabling style elements" ` snippet: `<CSPProvider
  disableStyleElements>{/* ... */}</CSPProvider>`.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:74-76`
- There are no `<Demo... />` component renders on this page; all example code above is embedded
  directly in the .mdx.

## Discrepancies (docs page vs. behavior.md)

No direct contradictions found between the page prose and `specs/library/csp-provider/behavior.md`.
Items flagged for lack of coverage or omission:

- Docs-only claim, not covered by behavior.md: inline `<script>` tags exist across Base UI
  components, are opt-in, would receive the nonce under `CSPProvider`, are unaffected by
  `disableStyleElements`, have no disable flag of their own, and require a nonce when used.
  behavior.md's tests exercise only `<style>` elements; the nonce is verified on the injected
  style element only ("DOM structure & portal behavior").
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:22`, `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:55`, `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:78`
- Docs-only claim, not covered by behavior.md: the style-tag emitters are exactly
  `<ScrollArea.Viewport>` and `<Select.Popup>`/`<Select.List>` gated on `alignItemWithTrigger`.
  behavior.md "DOM structure & portal behavior" confirms ScrollArea (Root + Viewport) and Select
  (portaled popup content) emit `.base-ui-disable-scrollbar`, but does not mention
  `alignItemWithTrigger` or `<Select.List>`.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:59`
- Docs-only claim, not covered by behavior.md: the entire "Inline style attributes" section —
  that the provider does not cover inline style attributes, the `style-src-attr` directive
  semantics, and the three mitigation options (`'unsafe-inline'` relaxation, client-only
  rendering, manually unsetting inline styles such as
  `<ScrollArea.Viewport style={{ overflow: undefined }}>` plus the upgrade-vetting caveat).
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:82`, `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:86-90`
- Omission (behavior.md fact not surfaced on the page): behavior.md "Edge cases (rapid
  interactions, unmount, nesting)" records that injected inline styles persist in the document
  across renders under React 19's style hoisting, and marks provider-unmount and multi-provider
  nesting behavior as UNVERIFIED; the page never mentions style persistence, unmount, or nesting
  multiple providers. These are test-environment/unverified observations rather than
  developer-facing caveats, so the omission is informational, not a contradiction.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:24`

## Cross-links to other docs pages

- N/A — the page contains no internal links to other Base UI documentation pages.
- One external link: MDN reference for the HTML `nonce` global attribute
  (`https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/nonce`), used
  to explain why inline tags need a nonce. `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:22`
- `<ScrollArea.Viewport>`, `<Select.Popup>`, and `<Select.List>` are referenced by name as inline
  code, not as hyperlinks to their docs pages.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:59`, `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:90`
- The page imports no demo components (`./demos/*` does not exist for this page); its only import
  besides metadata is the generated `TypesCSPProvider` reference component.
  `docs/src/app/(docs)/react/utils/csp-provider/page.mdx:94`
