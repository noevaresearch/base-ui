# Accordion implementation spec

Companion to `behavior.md` (same directory), which documents WHAT the accordion does; this file
documents WHY/HOW. The `TODO.md` entry (`TODO.md:301-307`) has no `wraps-external:` field, so
there is no external package to delegate to — everything below is derived from the component's
own source. The single most important structural fact: **Accordion is a thin composition layer
over the internal `collapsible` component.** Accordion owns the state (a `Value[]` array on the
root); `Collapsible` is instantiated per item in a permanently controlled mode and supplies the
mount/unmount, transition, measurement, and `hidden="until-found"` machinery.

The only file in the given list not mined here is `packages/react/src/accordion/root/AccordionRoot.spec.tsx`,
which is a TypeScript type test (compile-time assertions about the `Value` generic, no runtime
behavior); per the Stage 2 citation rules it is excluded from citation, and the gap it leaves is
flagged in the last section.

## State machine / hooks used

### Root: one `useControlled` array is the entire state machine

- `useControlled` (`packages/react/src/accordion/root/AccordionRoot.tsx:60-65`) holds the open
  item values, uncontrolled defaulting to the shared frozen `EMPTY_ARRAY`
  (`packages/react/src/accordion/root/AccordionRoot.tsx:44`). `useControlled` returns a setter
  that is a no-op when the prop is controlled (`packages/utils/src/useControlled.ts:82-91`), so
  uncontrolled mode updates internal state while controlled mode relies on the consumer
  re-rendering with a new `value` (see "State model" in behavior.md).
- All array algebra lives in one `useStableCallback`, `handleValueChange`
  (`packages/react/src/accordion/root/AccordionRoot.tsx:67-97`):
  - non-`multiple`: toggle by value identity — `[]` if `value[0] === newValue`, else `[newValue]`
    (`packages/react/src/accordion/root/AccordionRoot.tsx:73-79`);
  - `multiple` + open: append to a copy; `multiple` + close: filter out
    (`packages/react/src/accordion/root/AccordionRoot.tsx:80-95`).
  - Cancel protocol ordering (what makes `eventDetails.cancel()` work as documented in the
    Events section of behavior.md): the user's `onValueChange` is invoked with the *attempted*
    next value **before** any state write, and `setValue` is skipped when
    `details.isCanceled` (`packages/react/src/accordion/root/AccordionRoot.tsx:75-79`,
    `packages/react/src/accordion/root/AccordionRoot.tsx:83-87`,
    `packages/react/src/accordion/root/AccordionRoot.tsx:90-94`).
- Dev-only `warn` for the `hiddenUntilFound` + `keepMounted={false}` conflict
  (`packages/react/src/accordion/root/AccordionRoot.tsx:46-56`).
- `useRenderElement` renders the root `div` with the state and a mapping that suppresses the raw
  `value` array from becoming a data attribute
  (`packages/react/src/accordion/root/AccordionRoot.tsx:120-125`, mapping at
  `packages/react/src/accordion/root/AccordionRoot.tsx:14-16`).
- `orientation`/`loopFocus` are accepted but are documented deprecated no-ops that no longer
  affect keyboard focus (`packages/react/src/accordion/root/AccordionRoot.tsx:196-202`,
  `packages/react/src/accordion/root/AccordionRoot.tsx:216-223`); `orientation` only flows into
  state (and thence `data-orientation`) (`packages/react/src/accordion/root/AccordionRoot.tsx:99-106`).

### Item: derived open state, delegation into collapsible

- `useCompositeListItem()` registers the item DOM node with the root's `CompositeList` and
  returns the DOM-position `index`; the registration ref is merged with the forwarded ref via
  `useMergedRefs` (`packages/react/src/accordion/item/AccordionItem.tsx:42-43`). Registration is
  a callback ref into a shared map kept by `CompositeList`
  (`packages/react/src/internals/composite/list/useCompositeListItem.ts:62-82`), and indexes are
  assigned by `document` position order (`packages/react/src/internals/composite/list/CompositeList.tsx:227-272`).
- A missing `value` falls back to a generated id from `useBaseUiId`
  (`packages/react/src/accordion/item/AccordionItem.tsx:52-54`), which wraps `useId` with a
  `base-ui-` prefix (`packages/react/src/internals/useBaseUiId.ts:9-11`) — stable across
  SSR/hydration (matches the hydration association tests referenced in the Accessibility section
  of behavior.md).
- `isOpen` is a pure derivation: membership test of the item value in the root array
  (`packages/react/src/accordion/item/AccordionItem.tsx:58`).
- The item's `onOpenChange` wrapper chains two cancel checks: it first calls the item-level
  `onOpenChange` prop, returns if that cancelled, then forwards to the root's
  `handleValueChange` (`packages/react/src/accordion/item/AccordionItem.tsx:60-70`). Combined
  with the root ordering above, this produces the two-layer cancellation documented in the
  Events section of behavior.md (item-level cancel blocks even `onValueChange`).
- `useCollapsibleRoot` is called with `open: isOpen` **as a controlled prop**
  (`packages/react/src/accordion/item/AccordionItem.tsx:72-76`). Inside, collapsible keeps its
  own `useControlled` but since `controlled` is always defined here, its `setOpen` can never
  mutate anything (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:16-21`,
  `packages/utils/src/useControlled.ts:82-91`). Every state mutation is therefore routed:
  event → collapsible `handleTrigger` (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:30-41`,
  which stamps `REASONS.triggerPress` and the native event into the details) → item
  `onOpenChange` wrapper → root `handleValueChange` → `setValue`. The collapsible layer
  contributes `mounted` and `transitionStatus` from `useTransitionStatus`
  (`packages/react/src/collapsible/root/useCollapsibleRoot.ts:23`) — the shared
  starting→idle→ending machine where `mounted` flips true synchronously during the opening
  render and `ending` is deferred a frame so exit styles can be observed
  (`packages/react/src/internals/useTransitionStatus.ts:17-34`,
  `packages/react/src/internals/useTransitionStatus.ts:44-56`).
- Item state merges root state with per-item fields; `hidden` is `!isOpen && !collapsible.mounted`,
  so a closing panel remains "not hidden" until the exit transition finishes — the mechanism
  behind the closing-panel assertions in the DOM structure section of behavior.md
  (`packages/react/src/accordion/item/AccordionItem.tsx:96-105`).
- Trigger-id registry: `defaultTriggerId` from `useBaseUiId` plus a tri-state
  `registeredTriggerId` (`undefined` = keep fallback, string = manual id registered, `null` =
  trigger unmounted) resolved into `triggerId`
  (`packages/react/src/accordion/item/AccordionItem.tsx:107-111`). This is what makes the
  panel's `aria-labelledby` follow dynamic trigger ids and drop on unmount (Accessibility
  section of behavior.md).

### Trigger: `useButton` supplies activation semantics

- `useButton({ disabled, focusableWhenDisabled: true, native: nativeButton })`
  (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:37-41`). `focusableWhenDisabled`
  is what keeps non-native/disabled triggers tabbable (`tabindex="0"` asserted in the Focus
  management section of behavior.md) via `useFocusableWhenDisabled`
  (`packages/react/src/internals/use-button/useButton.ts:28-34`).
- Space-vs-Enter timing documented in the Keyboard interactions section of behavior.md is entirely
  `useButton`'s doing: Enter on a non-native trigger dispatches a synthetic click on **keydown**
  (`packages/react/src/internals/use-button/useButton.ts:154-176`), Space dispatches it on
  **keyup** (`packages/react/src/internals/use-button/useButton.ts:177-217`); native `<button>`s
  get both from the browser. The synthetic click then flows through the `onClick: handleTrigger`
  prop (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:54-59`).
- Disabled resolution `disabledProp || contextDisabled`
  (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:35`) is why a root/item-disabled
  accordion beats a trigger's own `disabled={false}` (Edge cases section of behavior.md).
- The manual-id registration effect mirrors the item's tri-state protocol: register `idProp`
  (empty string treated as absent), restore the fallback on change, mark `null` on unmount
  (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:47-52`).
- `data-panel-open` (asserted in the DOM structure section of behavior.md) comes from
  `triggerOpenStateMapping`, which maps `open: true` → `data-panel-open`
  (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:61-66`,
  `packages/react/src/utils/collapsibleOpenStateMapping.ts:13-24`).

### Panel: `useCollapsiblePanel` owns mount/measure/transition sequencing

- Panel-level `hiddenUntilFound`/`keepMounted` default to the root's values and can override
  them, including overriding a root-level `hiddenUntilFound={false}`
  (`packages/react/src/accordion/panel/AccordionPanel.tsx:38-39`,
  `packages/react/src/accordion/panel/AccordionPanel.tsx:52-53`); a second dev-only `warn`
  covers the panel-level conflict (`packages/react/src/accordion/panel/AccordionPanel.tsx:57-67`).
- The panel id registry effect is the trigger-side counterpart: it registers the manual id into
  the collapsible root's `setPanelIdState`, whose resolved `panelId` (registered ?? generated)
  is what the trigger's `aria-controls` points at
  (`packages/react/src/accordion/panel/AccordionPanel.tsx:69-74`,
  `packages/react/src/collapsible/root/useCollapsibleRoot.ts:25-28`).
- `useCollapsiblePanel` does the heavy lifting
  (`packages/react/src/accordion/panel/AccordionPanel.tsx:76-95`):
  - `hidden = !open && !mounted` and the `shouldRender` gate (`keepMounted || hiddenUntilFound ||
    mounted || open`) — returning `null` below is the non-kept-mounted unmount behavior
    (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:73`,
    `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:384`,
    `packages/react/src/accordion/panel/AccordionPanel.tsx:136-140`).
  - One layout effect classifies the author's CSS into `css-transition` / `css-animation` / `none`
    by inspecting computed styles (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:410-445`)
    and then, per phase, measures `scrollHeight/scrollWidth`, resets layout styles, temporarily
    neutralizes motion, or unmounts when there is nothing to wait for
    (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:148-277`). Measured dimensions
    are fed back as the `--accordion-panel-height/width` CSS variables, `auto` when unmeasured
    (`packages/react/src/accordion/panel/AccordionPanel.tsx:119-124`, constants in
    `packages/react/src/accordion/panel/AccordionPanelCssVars.ts:5-9`) — the px-then-`hidden`
    closing behavior in the DOM structure section of behavior.md.
  - Close completion waits one animation frame after `data-ending-style` lands before watching
    for animation finish (Chrome registers the exit transition a frame late when one item closes
    while another opens) (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:297-344`).
  - `hidden="until-found"` is forced via direct DOM `setAttribute` because React cannot render
    that string value for the boolean `hidden` attribute
    (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:346-357`), and a `beforematch`
    listener opens the panel with `REASONS.none` details when browser find-in-page matches the
    hidden content (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:359-382`).
  - Open-animation suppression (SSR + `React.Activity` reveal, DOM structure / Edge cases
    sections of behavior.md) is driven by `shouldPreventOpenAnimation`, ref-based one-shot flags
    read during render by documented intent
    (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:75-80`).
- Panel style composition order matters: the user's `style` prop is stripped from the component
  props, resolved via `resolveStyle` (so `style` may be a function of panel state), and merged
  before the final `{ animationName: 'none' }` entry so suppression always wins
  (`packages/react/src/accordion/panel/AccordionPanel.tsx:105-134`,
  `packages/react/src/utils/resolveStyle.ts:8-13`).
- The panel exposes `transitionStatus` (with a `forcePanelIdle` override for motion-skipped open
  paths) in its state
  (`packages/react/src/accordion/panel/AccordionPanel.tsx:99-102`,
  `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:65`,
  `packages/react/src/collapsible/panel/useCollapsiblePanel.ts:74`).

### Header: stateless passthrough

`AccordionHeader` only reads the item state from context and renders an `h3` through
`useRenderElement` with the shared mapping — it contributes no behavior
(`packages/react/src/accordion/header/AccordionHeader.tsx:21-28`).

### Data-attribute layer

`accordionStateAttributesMapping` composes the collapsible open/closed mapping, `data-index`,
and the transition-status mapping, and suppresses `value` from becoming an attribute
(`packages/react/src/accordion/item/stateAttributesMapping.ts:7-12`). The constant string
definitions live in `packages/react/src/accordion/item/AccordionItemDataAttributes.ts:5-13`,
`packages/react/src/accordion/panel/AccordionPanelDataAttributes.ts:7-27`,
`packages/react/src/accordion/root/AccordionRootDataAttributes.ts:4-6`, and
`packages/react/src/accordion/header/AccordionHeaderDataAttributes.ts:5-13`.

## Context providers/consumers

Three contexts cross component boundaries (all created with `undefined` defaults and guarded by
hook accessors that throw a `Base UI:` error naming the required ancestor —
`packages/react/src/accordion/root/AccordionRootContext.ts:18-30`,
`packages/react/src/accordion/item/AccordionItemContext.ts:13-25`):

1. **`AccordionRootContext`** — Root → Item and Panel
   (`packages/react/src/accordion/root/AccordionRoot.tsx:108-118`,
   `packages/react/src/accordion/root/AccordionRoot.tsx:127-131`). Carries `disabled`,
   `handleValueChange`, `hiddenUntilFound`, `keepMounted`, `state`, `value`
   (`packages/react/src/accordion/root/AccordionRootContext.ts:5-16`). Item uses it for
   `disabled`/`handleValueChange`/`value`/`state`; Panel uses only the `hiddenUntilFound` /
   `keepMounted` defaults (`packages/react/src/accordion/panel/AccordionPanel.tsx:38-39`).
2. **`CollapsibleRootContext`** — Item → Trigger and Panel. The item *provides* this context
   (wrapping everything `useCollapsibleRoot` returned plus the wrapped `onOpenChange` and the
   collapsible state) (`packages/react/src/accordion/item/AccordionItem.tsx:87-94`,
   `packages/react/src/accordion/item/AccordionItem.tsx:132`), so the accordion's Trigger/Panel
   are the same consumers the standalone Collapsible component uses
   (`packages/react/src/collapsible/root/CollapsibleRootContext.ts:6-13`). This is the reuse
   seam: `AccordionTrigger` reads `panelId`/`open`/`handleTrigger`/`disabled`
   (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:33`) and `AccordionPanel` reads
   `mounted`/`onOpenChange`/`open`/`setMounted`/`setOpen`/`setPanelIdState`/`transitionStatus`/
   `defaultPanelId` (`packages/react/src/accordion/panel/AccordionPanel.tsx:41-50`).
3. **`AccordionItemContext`** — Item → Header, Trigger, Panel
   (`packages/react/src/accordion/item/AccordionItem.tsx:113-122`,
   `packages/react/src/accordion/item/AccordionItem.tsx:133`). Carries the item `state`,
   `open`, and the trigger-id registry (`defaultTriggerId`, `triggerId`, `setTriggerId`)
   (`packages/react/src/accordion/item/AccordionItemContext.ts:5-11`). Trigger uses the id
   registry (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:43`); Header uses only
   `state`; Panel uses `state` and `triggerId` for `aria-labelledby`
   (`packages/react/src/accordion/panel/AccordionPanel.tsx:97`,
   `packages/react/src/accordion/panel/AccordionPanel.tsx:117-118`).

Additionally, the root wraps children in `CompositeList`, whose registration context Item
consumes via `useCompositeListItem` (`packages/react/src/accordion/root/AccordionRoot.tsx:129`,
`packages/react/src/accordion/item/AccordionItem.tsx:42`).

## DOM/portal strategy and why

- **No portal anywhere.** Every part renders in place via `useRenderElement` with intrinsic
  elements: Root/Item/Panel `div`, Header `h3`, Trigger `button` (with `type="button"`, or
  `role="button"` when non-native) (`packages/react/src/accordion/root/AccordionRoot.tsx:120-125`,
  `packages/react/src/accordion/item/AccordionItem.tsx:124-129`,
  `packages/react/src/accordion/header/AccordionHeader.tsx:23-28`,
  `packages/react/src/accordion/trigger/AccordionTrigger.tsx:61-66`,
  `packages/react/src/accordion/panel/AccordionPanel.tsx:105-134`,
  `packages/react/src/internals/use-button/useButton.ts:226`). This matches the DOM structure
  section of behavior.md. Rationale: an accordion is a document-flow disclosure widget — the
  panel must sit at its authored position for `role="region"` semantics, the collapsible layer
  measures the panel's layout (`scrollHeight`/`scrollWidth`), and the `hidden="until-found"`
  feature requires the closed panel to remain a rendered, in-place DOM node rather than being
  portaled away.
- **ARIA wiring is id-registry based, not slot based.** Trigger sets `aria-controls` to the
  resolved panel id *only while open*, and `aria-expanded` always
  (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:54-59`); Panel sets
  `role="region"` + `aria-labelledby` to the resolved trigger id
  (`packages/react/src/accordion/panel/AccordionPanel.tsx:116-118`). Both directions survive
  custom ids, id changes, unmounts, and hydration because the ids are reconciled through the
  tri-state registries described above (Accessibility section of behavior.md).
- **State reaches the DOM as data attributes, not rendered props**: each part maps its memoized
  state object through `stateAttributesMapping` inside `useRenderElement` — root suppresses
  `value`, items/panels add `data-index`/transition attributes, triggers add `data-panel-open`
  (`packages/react/src/accordion/root/AccordionRoot.tsx:14-16`,
  `packages/react/src/accordion/item/stateAttributesMapping.ts:7-12`,
  `packages/react/src/utils/collapsibleOpenStateMapping.ts:13-24`).
- **SSR/animation interaction is a style-ordering decision**, described under the Panel section
  above (`packages/react/src/accordion/panel/AccordionPanel.tsx:126-130`).

## Dependencies on other Base UI internals

No `wraps-external:` field exists for this unit (`TODO.md:301-307`), so there is no external
package or replacement crate to name; the accordion is self-contained within `leptos-ui`'s
future scope except for shared utilities. Direct dependencies, by origin:

`@base-ui/utils/*` (public shared utils package):

| Utility | Used by | Purpose |
|---|---|---|
| `useControlled` | Root (`packages/react/src/accordion/root/AccordionRoot.tsx:3`) | controlled/uncontrolled `value` array |
| `useStableCallback` | Root (`packages/react/src/accordion/root/AccordionRoot.tsx:4`), Item (`packages/react/src/accordion/item/AccordionItem.tsx:3`) | stable `handleValueChange`/`onOpenChange` |
| `warn` | Root (`packages/react/src/accordion/root/AccordionRoot.tsx:5`), Panel (`packages/react/src/accordion/panel/AccordionPanel.tsx:4`) | dev-only prop-conflict warnings |
| `EMPTY_ARRAY` | Root (`packages/react/src/accordion/root/AccordionRoot.tsx:6`) | stable default `value` |
| `useMergedRefs` | Item (`packages/react/src/accordion/item/AccordionItem.tsx:4`) | forwarded + composite-list refs |
| `useIsoLayoutEffect` | Trigger (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:3`), Panel (`packages/react/src/accordion/panel/AccordionPanel.tsx:3`) | id-registration effects |

`internals/` (private to `packages/react`):

| Internal | Used by | Purpose |
|---|---|---|
| `useRenderElement` | all five parts (`packages/react/src/accordion/root/AccordionRoot.tsx:10`, `packages/react/src/accordion/item/AccordionItem.tsx:18`, `packages/react/src/accordion/header/AccordionHeader.tsx:4`, `packages/react/src/accordion/trigger/AccordionTrigger.tsx:10`, `packages/react/src/accordion/panel/AccordionPanel.tsx:15`) | element rendering, state→data-attribute mapping, prop/refs merging |
| `useBaseUiId` | Item (import at `packages/react/src/accordion/item/AccordionItem.tsx:6`, used at `packages/react/src/accordion/item/AccordionItem.tsx:52` and `packages/react/src/accordion/item/AccordionItem.tsx:107`) | generated fallback value and trigger id |
| `useButton` | Trigger (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:6`) | button semantics, activation timing, focusable-when-disabled (transitively depends on `@floating-ui/utils/dom`) |
| `CompositeList` | Root (`packages/react/src/accordion/root/AccordionRoot.tsx:8`) | item registration/indexing |
| `useCompositeListItem` | Item (`packages/react/src/accordion/item/AccordionItem.tsx:13`) | per-item registration + `index` |
| `createBaseUIEventDetails` / `REASONS` | Root (`packages/react/src/accordion/root/AccordionRoot.tsx:11-12`), Item (`packages/react/src/accordion/item/AccordionItem.tsx:19-20`) | `ChangeEventDetails` type + `triggerPress`/`none` reasons |
| `BaseUIComponentProps`/`NativeButtonProps`/`Orientation` | all parts (types) | shared prop contracts |
| `useTransitionStatus`, `useOpenChangeComplete`, `useAnimationsFinished`, `getStateAttributesProps` | not imported by accordion files directly; pulled in via `useCollapsibleRoot` / `useCollapsiblePanel` / mapping types | transition machine, completion detection, attribute-mapping typing |

`collapsible/` (sibling component, the dominant dependency):

| Module | Used by | Purpose |
|---|---|---|
| `useCollapsibleRoot` | Item (import at `packages/react/src/accordion/item/AccordionItem.tsx:8-10`, call at `packages/react/src/accordion/item/AccordionItem.tsx:72-76`) | controlled `open`, `handleTrigger`, `mounted`/`transitionStatus`, panel-id registry |
| `CollapsibleRootContext` | Item provides (import at `packages/react/src/accordion/item/AccordionItem.tsx:12`, provider at `packages/react/src/accordion/item/AccordionItem.tsx:132`); Trigger/Panel consume (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:7`, `packages/react/src/accordion/panel/AccordionPanel.tsx:7`) | shared trigger/panel seam |
| `useCollapsiblePanel` | Panel (`packages/react/src/accordion/panel/AccordionPanel.tsx:8,76-95`) | mount/unmount sequencing, measurement, animation-type handling, `hidden="until-found"`, `beforematch` |
| `CollapsibleRoot`/`CollapsibleRootState` types | Item (`packages/react/src/accordion/item/AccordionItem.tsx:11`) | typing of the provided context |

In-package shared utils (`packages/react/src/utils/`): `collapsibleOpenStateMapping`
(`triggerOpenStateMapping` for the Trigger, base mapping reused by the item mapping —
`packages/react/src/accordion/trigger/AccordionTrigger.tsx:4`,
`packages/react/src/accordion/item/stateAttributesMapping.ts:2`) and `resolveStyle` (Panel,
`packages/react/src/accordion/panel/AccordionPanel.tsx:6`).

Notably **not** depended on: `floating-ui-react` and `use-render` — accordion imports neither;
its positioning/transient-popup machinery usage is nil, and element rendering goes through
`internals/useRenderElement`, not the public `use-render` util.

## Anything in source not explained by any test

Explicit gaps, per the backward-looking audit's needs:

1. **`accordionItemRefs` is written but never read.** The root allocates the array
   (`packages/react/src/accordion/root/AccordionRoot.tsx:58`) and hands it to `CompositeList`
   (`packages/react/src/accordion/root/AccordionRoot.tsx:129`), but nothing in the accordion consumes it — it is vestigial support for the removed
   roving-focus keyboard navigation (`loopFocus`/`orientation` are deprecated no-ops,
   `packages/react/src/accordion/root/AccordionRoot.tsx:196-202`,
   `packages/react/src/accordion/root/AccordionRoot.tsx:216-223`). behavior.md already flags
   `data-index`/`data-orientation` as never asserted; the item-index machinery
   (`useCompositeListItem`) is therefore observable only through those unasserted attributes.
   A Rust/Leptos port could drop the elementsRef without behavioral loss, but that decision
   belongs to the architecture spec.
2. **Non-multiple "close" algebra assumes the toggled item is `value[0]`.** When not `multiple`,
   `nextValue = value[0] === newValue ? [] : [newValue]`
   (`packages/react/src/accordion/root/AccordionRoot.tsx:73-79`) ignores `nextOpen` entirely. If
   a value mismatch ever occurs (e.g. a controlled consumer holds `[a]` while item `b` — which
   is somehow open — toggles closed), the result is `[b]`: an *open*, not a close. No test
   exercises mismatched or duplicate item values against a controlled root.
3. **`aria-controls` is dropped while closed even for kept-mounted panels.** The trigger only
   renders `aria-controls` when `open` (`packages/react/src/accordion/trigger/AccordionTrigger.tsx:55`).
   Tests assert association loss on panel *unmount* (Accessibility section of behavior.md), but
   never assert the `keepMounted`-closed case where the panel exists in the DOM yet the trigger
   advertises no `aria-controls`.
4. **Empty-string panel id is handled inconsistently.** Registration normalizes
   `registeredId = idProp || undefined` while the rendered `id` uses `idProp ?? defaultPanelId`
   (`packages/react/src/accordion/panel/AccordionPanel.tsx:54-55`), so `id=""` would render an
   empty attribute while the trigger links to the generated id. No test covers empty-string ids
   on Panel or Trigger (Trigger normalizes both paths identically,
   `packages/react/src/accordion/trigger/AccordionTrigger.tsx:44-45`).
5. **`beforematch`-triggered opens are untested.** The panel registers a `beforematch` listener
   that opens via `onOpenChange(true, createChangeEventDetails(REASONS.none, event))`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:359-382`), producing a change
   whose `reason` is `none` — a path behavior.md's Events section (which only documents
   `triggerPress`) and tests never cover, even though the root's reason union admits both
   (`packages/react/src/accordion/root/AccordionRoot.tsx:226-229`).
6. **Generic `Value` typing contract has no runtime coverage.** `AccordionRoot.spec.tsx` pins the
   type-level promise that `onValueChange` receives `Value[]` (with a permissive `any[]`
   default), but no runtime test passes typed custom values through the full open/close cycle;
   behavior.md covers custom values only at the value-identity level.
7. **`getAnimationType` conflict warning is dev-only and untested.** Panels styled with both a
   CSS transition and a CSS animation warn and are treated as `css-transition`
   (`packages/react/src/collapsible/panel/useCollapsiblePanel.ts:424-434`); no accordion test
   renders such a combination.
