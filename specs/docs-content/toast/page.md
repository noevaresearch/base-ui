# Toast docs page content spec

Mined from `docs/src/app/(docs)/react/components/toast/page.mdx` only. The component's own
behavior is meant to be covered by `specs/library/toast/behavior.md` and referenced here by
section name; see the note below — that file did not exist at mining time. Demo source files
under `docs/src/app/(docs)/react/components/toast/demos/` are out of scope for this file
(Stage 2 mines them into `specs/docs-content/toast/demos.json`).

**Note on cross-checking:** `specs/library/toast/behavior.md` was not present when this spec
was mined (the `specs/library/toast/` directory exists but is empty). No claim below could be
cross-checked against it, so all cross-checks are recorded as "unverified" rather than asserted
consistent. This file must be revisited once the behavior spec is generated.

## Page structure (headings, in order)

- `# Toast` (h1) — `docs/src/app/(docs)/react/components/toast/page.mdx:1`
- `## Anatomy` — `docs/src/app/(docs)/react/components/toast/page.mdx:14`
- `## General usage` — `docs/src/app/(docs)/react/components/toast/page.mdx:51`
- `## Global manager` — `docs/src/app/(docs)/react/components/toast/page.mdx:58`
- `## Stacking and animations` — `docs/src/app/(docs)/react/components/toast/page.mdx:73`
- `## Examples` — `docs/src/app/(docs)/react/components/toast/page.mdx:160`
  - `### Anchored toasts` — `docs/src/app/(docs)/react/components/toast/page.mdx:162`
  - `### Custom position` — `docs/src/app/(docs)/react/components/toast/page.mdx:222`
  - `### Undo action` — `docs/src/app/(docs)/react/components/toast/page.mdx:233`
  - `### Promise` — `docs/src/app/(docs)/react/components/toast/page.mdx:241`
  - `### Custom` — `docs/src/app/(docs)/react/components/toast/page.mdx:251`
  - `### Deduplicated toast` — `docs/src/app/(docs)/react/components/toast/page.mdx:260`
  - `### Varying heights` — `docs/src/app/(docs)/react/components/toast/page.mdx:268`
- `## API reference` — `docs/src/app/(docs)/react/components/toast/page.mdx:277`
  - `### Provider` — `docs/src/app/(docs)/react/components/toast/page.mdx:281`
  - `### Portal` — `docs/src/app/(docs)/react/components/toast/page.mdx:285`
  - `### Viewport` — `docs/src/app/(docs)/react/components/toast/page.mdx:289`
  - `### Root` — `docs/src/app/(docs)/react/components/toast/page.mdx:293`
  - `### Content` — `docs/src/app/(docs)/react/components/toast/page.mdx:297`
  - `### Title` — `docs/src/app/(docs)/react/components/toast/page.mdx:301`
  - `### Description` — `docs/src/app/(docs)/react/components/toast/page.mdx:305`
  - `### Action` — `docs/src/app/(docs)/react/components/toast/page.mdx:309`
  - `### Close` — `docs/src/app/(docs)/react/components/toast/page.mdx:313`
  - `### Positioner` — `docs/src/app/(docs)/react/components/toast/page.mdx:317`
  - `### Arrow` — `docs/src/app/(docs)/react/components/toast/page.mdx:321`
- `## useToastManager` — `docs/src/app/(docs)/react/components/toast/page.mdx:325`
  - ``### `add` method`` — `docs/src/app/(docs)/react/components/toast/page.mdx:335`
  - ``### `update` method`` — `docs/src/app/(docs)/react/components/toast/page.mdx:373`
  - ``### `close` method`` — `docs/src/app/(docs)/react/components/toast/page.mdx:391`
  - ``### `promise` method`` — `docs/src/app/(docs)/react/components/toast/page.mdx:405`
- `## Additional types` — `docs/src/app/(docs)/react/components/toast/page.mdx:457`

Non-heading page furniture, in document order:

- `<Subtitle>` — "Generates toast notifications." — `docs/src/app/(docs)/react/components/toast/page.mdx:3`
- `<Meta name="description">` — "A high-quality, unstyled React toast component to generate notifications." — `docs/src/app/(docs)/react/components/toast/page.mdx:5-8`
- Hero demo import and render (`./demos/hero`) before the first heading — `docs/src/app/(docs)/react/components/toast/page.mdx:10-12`
- `TypesToast` and `TypesToastAdditional` imports for the API reference — `docs/src/app/(docs)/react/components/toast/page.mdx:279`
- Demo imports interleaved with the Examples subsections (`./demos/anchored` at `docs/src/app/(docs)/react/components/toast/page.mdx:218`, `./demos/position` at `docs/src/app/(docs)/react/components/toast/page.mdx:229`, `./demos/undo` at `docs/src/app/(docs)/react/components/toast/page.mdx:237`, `./demos/promise` at `docs/src/app/(docs)/react/components/toast/page.mdx:247`, `./demos/custom` at `docs/src/app/(docs)/react/components/toast/page.mdx:256`, `./demos/deduplicate` at `docs/src/app/(docs)/react/components/toast/page.mdx:264`, `./demos/varying-heights` at `docs/src/app/(docs)/react/components/toast/page.mdx:273`)
- `[//]: # '@exclude-table-of-contents'` marker after the `promise` method section — `docs/src/app/(docs)/react/components/toast/page.mdx:455`
- Trailing `export const metadata` SEO keywords block (24 keywords, e.g. 'React Toast', 'Toast Manager', 'Live Region') — `docs/src/app/(docs)/react/components/toast/page.mdx:461-488`

## Prose claims about component behavior

All claims below are unverified against `specs/library/toast/behavior.md` (file absent at
mining time); none is asserted consistent with it.

- Page describes the component as "Generates toast notifications." and, in the meta
  description, as "unstyled". `docs/src/app/(docs)/react/components/toast/page.mdx:3`, `docs/src/app/(docs)/react/components/toast/page.mdx:5-8`
- Anatomy usage guidance: "Import the component and assemble its parts". The snippet shows two
  assemblies from the `@base-ui/react/toast` namespace: stacked toasts
  (`Provider` > `Portal` > `Viewport` > `Root` > `Content` containing `Title`, `Description`,
  `Action`, `Close`) and anchored toasts (`Provider` > `Portal` > `Viewport` > `Positioner` >
  `Root` containing `Arrow` then `Content` with the same inner parts).
  `docs/src/app/(docs)/react/components/toast/page.mdx:16-49`
- "`<Toast.Provider>` can be wrapped around your entire app, ensuring all toasts are rendered in
  the same viewport." `docs/src/app/(docs)/react/components/toast/page.mdx:53`
- "<kbd>F6</kbd> lets users jump into the toast viewport landmark region to navigate toasts with
  keyboard focus." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:54-55`
- "The `data-base-ui-swipe-ignore` attribute can be manually added to elements inside of a toast
  to prevent swipe-to-dismiss gestures on them. Interactive elements are automatically
  prevented." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:56`
- Global manager: "A global toast manager can be created by passing the `toastManager` prop to
  the `<Toast.Provider>`" — enables queuing a toast from anywhere in the app (such as in
  functions outside the React tree) while still using the same toast renderer.
  `docs/src/app/(docs)/react/components/toast/page.mdx:60-61`
- "The created `toastManager` exposes the same `add`, `close`, `update`, and `promise` methods
  as the `Toast.useToastManager()` hook. Unlike the hook, it does not return the reactive
  `toasts` array, since it lives outside the React tree."
  `docs/src/app/(docs)/react/components/toast/page.mdx:63`
- "The `--toast-index` CSS variable can be used to determine the stacking order of the toasts.
  The 0th index toast appears at the front."
  `docs/src/app/(docs)/react/components/toast/page.mdx:75-76`
- "The `--toast-offset-y` CSS variable can be used to determine the vertical offset of the
  toasts when positioned absolutely with a translation offset — this is usually used with the
  `data-expanded` attribute, present when the toast viewport is being hovered or has focus."
  Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:85`
- "While the stack is collapsed, each toast's height can be clamped to the frontmost toast's
  height using the `--toast-frontmost-height` CSS variable, and `<Toast.Content>` is used to
  hide the content of the toasts behind it. The `data-behind` attribute marks content that sits
  behind the frontmost toast and pairs with the `data-expanded` attribute so the content fades
  back in when the viewport expands." Unverified.
  `docs/src/app/(docs)/react/components/toast/page.mdx:93-94`
- "The `--toast-swipe-movement-x` and `--toast-swipe-movement-y` CSS variables are used to
  determine the swipe movement of the toasts in order to add a translation offset."
  `docs/src/app/(docs)/react/components/toast/page.mdx:116`
- "The `data-swipe-direction` attribute can be used to determine the swipe direction of the
  toasts to add a translation offset upon dismissal."
  `docs/src/app/(docs)/react/components/toast/page.mdx:127`
- "The `data-limited` attribute indicates that the toast exceeded the `limit` option. Limited
  toasts remain mounted with the HTML `inert` attribute, so this is useful for hiding them or
  animating them differently." Unverified.
  `docs/src/app/(docs)/react/components/toast/page.mdx:154-155`
- "The `updateKey` property increments when a toast is updated or upserted. This can be used to
  replay attention-grabbing styles by switching animation names or, when remounting is
  acceptable, by including it in a React `key`."
  `docs/src/app/(docs)/react/components/toast/page.mdx:157-158`
- Anchored toasts: "Toasts can be anchored to a specific element using `<Toast.Positioner>` and
  the `positionerProps` option when adding a toast. This is useful for showing contextual
  feedback like transient 'Copied' toasts that appear near the button that triggered the
  action." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:164`
- "Anchored toasts should be rendered in a separate `<Toast.Provider>` from stacked toasts. A
  global toast manager can be created for each to manage them separately throughout your app."
  `docs/src/app/(docs)/react/components/toast/page.mdx:166`
- Custom position: "The position of the toasts is controlled by your own CSS. To change the
  toasts' position, you can modify the `.Viewport` and `.Root` styles. A more general component
  could accept a `data-position` attribute, which the CSS handles for each variation."
  `docs/src/app/(docs)/react/components/toast/page.mdx:224-227`
- Undo action: "When adding a toast, the `actionProps` option can be used to define props for an
  action button inside of it—this enables the ability to undo an action associated with the
  toast." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:235`
- Promise states: "An asynchronous toast can be created with three possible states: `loading`,
  `success`, and `error`. The `type` string matches these states to change the styling. Each of
  the states also accepts the method options object for more granular control."
  `docs/src/app/(docs)/react/components/toast/page.mdx:243-245`
- Custom data: "A toast with custom data can be created by passing any typed object interface to
  the `data` option. This enables you to pass any data (including functions) you need to the
  toast and access it in the toast's rendering logic."
  `docs/src/app/(docs)/react/components/toast/page.mdx:253-254`
- Deduplication: "When you upsert the same toast by `id`, the `updateKey` property increments so
  a custom renderer can replay a visual animation." The demo description adds that alternating
  CSS animation names from `updateKey` keeps the same toast mounted while replaying the pulse.
  `docs/src/app/(docs)/react/components/toast/page.mdx:262`
- Varying heights: "Toasts with varying heights are stacked by clamping every toast's height to
  the frontmost toast at index 0 using the `--toast-frontmost-height` CSS variable, while the
  `data-behind` attribute hides the content of the toasts behind it." Caveat: "Avoid sizing
  `<Toast.Content>` to the root's height (such as `height: 100%`), as resizing it alongside the
  root cancels the root's height transition." Unverified.
  `docs/src/app/(docs)/react/components/toast/page.mdx:270-271`
- Hook description: "`useToastManager` … Manages toasts, called inside of a
  `<Toast.Provider>`." `docs/src/app/(docs)/react/components/toast/page.mdx:327`
- `add` method: "Creates a toast by adding it to the toast list." Upsert semantics: "If you pass
  an `id` that already exists, the existing toast is updated in place instead of creating a
  duplicate." Return value: "Returns a `toastId` that can be used to update or close the toast
  later." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:337-341`
- Screen-reader announcement: "For high priority toasts, the `title` and `description` strings
  are what are used to announce the toast to screen readers. Screen readers do not announce any
  extra content rendered inside `<Toast.Root>`, including the `<Toast.Title>` or
  `<Toast.Description>` components, unless they intentionally navigate to the toast viewport."
  Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:370-371`
- `update` method: "Updates the toast with new options." Semantics: "Options replace the
  corresponding values of the toast, including custom `data`, which is replaced as a whole. To
  derive the update from the current state of the toast, pass a function instead. It receives
  the current toast and returns the options to apply. Custom `data` is `undefined` when the
  toast has none yet." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:375`, `docs/src/app/(docs)/react/components/toast/page.mdx:383`
- `close` method: "Closes the toast, removing it from the toast list after any animations
  complete." Close-all: "you can close all toasts at once by not passing an ID." Unverified.
  `docs/src/app/(docs)/react/components/toast/page.mdx:393`, `docs/src/app/(docs)/react/components/toast/page.mdx:399`
- `promise` method: "Creates an asynchronous toast with three possible states: `loading`,
  `success`, and `error`." The description-configuration snippet comments that the
  `loading`/`success`/`error` string values are "a shortcut for the `description` option", and
  the prose adds that "Each state also accepts the method options object to granularly control
  the toast for each state." Unverified. `docs/src/app/(docs)/react/components/toast/page.mdx:407`, `docs/src/app/(docs)/react/components/toast/page.mdx:409-421`, `docs/src/app/(docs)/react/components/toast/page.mdx:423`

## API tables referenced (props/parts documented on this page)

- The `## API reference` section documents eleven parts, each as its own heading rendering a
  generated reference component: `### Provider` → `<TypesToast.Provider />`,
  `### Portal` → `<TypesToast.Portal />`, `### Viewport` → `<TypesToast.Viewport />`,
  `### Root` → `<TypesToast.Root />`, `### Content` → `<TypesToast.Content />`,
  `### Title` → `<TypesToast.Title />`, `### Description` → `<TypesToast.Description />`,
  `### Action` → `<TypesToast.Action />`, `### Close` → `<TypesToast.Close />`,
  `### Positioner` → `<TypesToast.Positioner />`, `### Arrow` → `<TypesToast.Arrow />`.
  `docs/src/app/(docs)/react/components/toast/page.mdx:277-323`
- The `## useToastManager` section documents the hook with
  `<TypesToast.useToastManager hideDescription />` — `docs/src/app/(docs)/react/components/toast/page.mdx:325-333`
- The `## Additional types` section renders
  `<TypesToastAdditional showAdditionalTypes={['toastmanagerupdateoptions']} />`, i.e. the
  `toastmanagerupdateoptions` type is the additional type documented on this page (the
  `[method options](#toastmanagerupdateoptions)` anchors in the Promise section point here).
  `docs/src/app/(docs)/react/components/toast/page.mdx:457-459`
- The reference tables themselves are generated components imported from `./types`
  (`import { TypesToast, TypesToastAdditional } from './types';`); no props, prop types,
  defaults, or prop descriptions are written inline in the .mdx source of this page.
  `docs/src/app/(docs)/react/components/toast/page.mdx:279`

## Code snippets embedded directly in the .mdx (not pulled from demos/)

Nineteen fenced code blocks plus one trailing JS export:

1. Anatomy snippet (` ```jsx title="Anatomy" `): imports `{ Toast }` from
   `@base-ui/react/toast`; assembles stacked and anchored toast trees (see Prose claims).
   `docs/src/app/(docs)/react/components/toast/page.mdx:18-49`
2. ` ```tsx title="Creating a manager instance" `: `const toastManager = Toast.createToastManager();`
   `docs/src/app/(docs)/react/components/toast/page.mdx:65-67`
3. ` ```jsx title="Using the instance" `: `<Toast.Provider toastManager={toastManager}>`
   `docs/src/app/(docs)/react/components/toast/page.mdx:69-71`
4. ` ```css title="z-index stacking" `: `.Toast` with
   `z-index: calc(1000 - var(--toast-index))` and
   `transform: scale(calc(max(0, 1 - (var(--toast-index) * 0.1))))`.
   `docs/src/app/(docs)/react/components/toast/page.mdx:78-83`
5. ` ```css title="Expanded offset" `: `.Toast[data-expanded]` with
   `transform: translateY(var(--toast-offset-y))`.
   `docs/src/app/(docs)/react/components/toast/page.mdx:87-91`
6. ` ```css title="Collapsed content" `: height clamp
   `height: var(--toast-frontmost-height, var(--toast-height))`, a `.ToastContent` opacity
   transition, `.ToastContent[data-behind] { opacity: 0 }`, and
   `.ToastContent[data-expanded] { opacity: 1 }` (with `@highlight-text` markers).
   `docs/src/app/(docs)/react/components/toast/page.mdx:96-114`
7. ` ```css title="Swipe offset" `: `.Toast` transform combining the scale, the
   `--toast-swipe-movement-x` translation, and a `--toast-swipe-movement-y` plus
   `--toast-index` percentage translation. `docs/src/app/(docs)/react/components/toast/page.mdx:118-125`
8. ` ```css title="Swipe direction" `: `&[data-ending-style]` with per-direction overrides for
   `data-swipe-direction='up' | 'down' | 'left' | 'right'`; the left/right branches reference a
   locally defined `--offset-y` (an inline comment notes it derives from `--toast-offset-y`,
   `--toast-index`, and swipe movement values).
   `docs/src/app/(docs)/react/components/toast/page.mdx:129-152`
9. ` ```tsx title="Mixing stacked and anchored toasts" `: creates
   `anchoredToastManager` and `stackedToastManager` via `Toast.createToastManager()`; an `App`
   renders two `<Toast.Provider>`s (one per manager); `AnchoredToasts` maps `toasts` from
   `Toast.useToastManager()` into `Toast.Positioner key={toast.id} toast={toast}` wrapping
   `<Toast.Root toast={toast}>`; `StackedToasts` maps them into plain `<Toast.Root>`.
   `docs/src/app/(docs)/react/components/toast/page.mdx:168-216`
10. ` ```tsx title="Usage" `: `const toastManager = Toast.useToastManager();`
    `docs/src/app/(docs)/react/components/toast/page.mdx:329-331`
11. ` ```jsx title="Usage" ` (add): `const toastId = toastManager.add({ description: 'Hello, world!' });`
    `docs/src/app/(docs)/react/components/toast/page.mdx:343-347`
12. ` ```jsx title="Example" ` (add): a button whose `onClick` calls
    `toastManager.add({ description: 'Hello, world!' })`, with `@highlight` markers.
    `docs/src/app/(docs)/react/components/toast/page.mdx:349-368`
13. ` ```jsx title="Usage" ` (update): `toastManager.update(toastId, { description: 'New description' });`
    `docs/src/app/(docs)/react/components/toast/page.mdx:377-381`
14. ` ```jsx title="Deriving the update from the current toast" `:
    `toastManager.update(toastId, (prevToast) => ({ data: prevToast.data && { ...prevToast.data, progress: 100 } }))`
    `docs/src/app/(docs)/react/components/toast/page.mdx:385-389`
15. ` ```jsx title="Usage" ` (close): `toastManager.close(toastId);`
    `docs/src/app/(docs)/react/components/toast/page.mdx:395-397`
16. ` ```jsx title="Close all toasts" `: `toastManager.close();`
    `docs/src/app/(docs)/react/components/toast/page.mdx:401-403`
17. ` ```tsx title="Description configuration" ` (promise): a promise resolved after a timeout
    with `loading`/`success`/`error` given as strings; inline comment "Each are a shortcut for
    the `description` option". `docs/src/app/(docs)/react/components/toast/page.mdx:409-421`
18. ` ```tsx title="Method options configuration" ` (promise): the same promise with
    `loading`/`success`/`error` given as option objects (`title` + `description`, plus
    `actionProps` with a "Contact support" button on the error state).
    `docs/src/app/(docs)/react/components/toast/page.mdx:425-451`
19. Trailing `export const metadata = { keywords: [...] }` (JS, not fenced): 24 SEO keywords.
    `docs/src/app/(docs)/react/components/toast/page.mdx:461-488`

The eight Examples subsections themselves render demo components
(`<DemoToastAnchored />`, `<DemoToastPosition />`, `<DemoToastUndo />`, `<DemoToastPromise />`,
`<DemoToastCustom />`, `<DemoToastDeduplicate />`, `<DemoToastVaryingHeights />`, plus
`<DemoToastHero />` at the top) imported from `./demos/*`; their code lives in demo files and is
out of scope here (Stage 2).
`docs/src/app/(docs)/react/components/toast/page.mdx:12`, `docs/src/app/(docs)/react/components/toast/page.mdx:220`, `docs/src/app/(docs)/react/components/toast/page.mdx:231`, `docs/src/app/(docs)/react/components/toast/page.mdx:239`, `docs/src/app/(docs)/react/components/toast/page.mdx:249`, `docs/src/app/(docs)/react/components/toast/page.mdx:258`, `docs/src/app/(docs)/react/components/toast/page.mdx:266`, `docs/src/app/(docs)/react/components/toast/page.mdx:275`

## Discrepancies (docs page vs. behavior.md)

No comparison was performed: `specs/library/toast/behavior.md` does not exist at mining time
(the `specs/library/toast/` directory is empty), so no contradiction can be confirmed and none
can be ruled out. Every claim in "Prose claims about component behavior" above is unverified.
Once the behavior spec is generated, this section must be revisited. Docs-only assertions that
will specifically need a cross-check against it:

- F6 keyboard access to the toast viewport landmark region.
  `docs/src/app/(docs)/react/components/toast/page.mdx:54-55`
- Automatic swipe-to-dismiss prevention on interactive elements (vs. manual
  `data-base-ui-swipe-ignore` opt-out). `docs/src/app/(docs)/react/components/toast/page.mdx:56`
- Limited toasts staying mounted with the HTML `inert` attribute.
  `docs/src/app/(docs)/react/components/toast/page.mdx:154-155`
- Screen-reader announcement using only the `title`/`description` strings of the add/update
  options, with extra `<Toast.Root>` content unannounced unless navigated to.
  `docs/src/app/(docs)/react/components/toast/page.mdx:370-371`
- `<Toast.Content>` sized to the root's height (e.g. `height: 100%`) canceling the root's
  height transition. `docs/src/app/(docs)/react/components/toast/page.mdx:270-271`
- Upsert-by-`id` semantics of `add`, the `toastId` return value, whole-replacement semantics of
  `update` (including function-form updates and `data` being `undefined` when unset), and
  animation-completion-gated removal on `close`.
  `docs/src/app/(docs)/react/components/toast/page.mdx:337-341`, `docs/src/app/(docs)/react/components/toast/page.mdx:383`, `docs/src/app/(docs)/react/components/toast/page.mdx:393`

## Cross-links to other docs pages

- N/A — the page contains no links to other Base UI documentation pages and no external URLs.
- In-page anchor links only: `[method options](#toastmanagerupdateoptions)` appears twice in
  the Promise prose/snippet area and targets the `## Additional types` section rendering the
  `toastmanagerupdateoptions` type; a trailing
  `[`promise` method](#ToastuseToastManager-promise)` link targets the
  ``### `promise` method`` heading. `docs/src/app/(docs)/react/components/toast/page.mdx:245`, `docs/src/app/(docs)/react/components/toast/page.mdx:423`, `docs/src/app/(docs)/react/components/toast/page.mdx:453`, `docs/src/app/(docs)/react/components/toast/page.mdx:457-459`
- Demo components (`./demos/hero`, `./demos/anchored`, `./demos/position`, `./demos/undo`,
  `./demos/promise`, `./demos/custom`, `./demos/deduplicate`, `./demos/varying-heights`) and
  `./types` are imported and rendered on this page itself; they are same-page imports, not
  cross-page links. `docs/src/app/(docs)/react/components/toast/page.mdx:10`, `docs/src/app/(docs)/react/components/toast/page.mdx:218`, `docs/src/app/(docs)/react/components/toast/page.mdx:229`, `docs/src/app/(docs)/react/components/toast/page.mdx:237`, `docs/src/app/(docs)/react/components/toast/page.mdx:247`, `docs/src/app/(docs)/react/components/toast/page.mdx:256`, `docs/src/app/(docs)/react/components/toast/page.mdx:264`, `docs/src/app/(docs)/react/components/toast/page.mdx:273`, `docs/src/app/(docs)/react/components/toast/page.mdx:279`
