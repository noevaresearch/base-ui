# `internals` — implementation spec (why/how behind the behavior)

Ground truth for WHAT happens is "Public API surface" and the other sections of this unit's
behavior spec; this document explains the mechanisms that produce it. Unit: `infra: internals`
(no `wraps-external:` field in TODO.md, so nothing here delegates to an external package — the
vendored `floating-ui-react` tree is in-repo source, covered under "Dependencies").

## State machine / hooks used

### Composite registry (`CompositeList` + `useCompositeListItem`)

`CompositeList` is a two-phase registry. Source of truth is a `Map<Element, registration>`
(`map`, `packages/react/src/internals/composite/list/CompositeList.tsx:30`) fed synchronously by
callback refs (`register`/`unregister`, `packages/react/src/internals/composite/list/CompositeList.tsx:48-58`).
Publications are then coalesced: `scheduleMapUpdate` sets an `isDirtyRef` flag and bumps a dummy
`useState` tick (`packages/react/src/internals/composite/list/CompositeList.tsx:39-46`), and a
`useIsoLayoutEffect` with no deps runs `flush()` on every commit when dirty
(`packages/react/src/internals/composite/list/CompositeList.tsx:187-191`). This is why
"one publication happens per commit where registration changes" (behavior spec, "State model"):
the dirty flag dedupes the N individual ref registrations that fire within a single commit into
one synchronous re-render, so refs are rebuilt before paint and while the originating event is
still inside `act()` (`packages/react/src/internals/composite/list/CompositeList.tsx:36-38`).

`flush()` derives the ordered snapshot via `getCompositeListSnapshot`
(`packages/react/src/internals/composite/list/CompositeList.tsx:142-170` and
`packages/react/src/internals/composite/list/CompositeList.tsx:227-272`): connected nodes only,
explicit non-negative indexes reserve their slots, automatic items sort by document position and
fill the gaps left of the reserved slots, then a final numeric sort orders everything. `syncRefs`
rebuilds `elementsRef`/`labelsRef` from that snapshot — including the label fallback chain
(`label` → `textRef.textContent` → `element.textContent`,
`packages/react/src/internals/composite/list/CompositeList.tsx:76-81`) — and `nextIndexRef` is
re-seeded from the final length (`packages/react/src/internals/composite/list/CompositeList.tsx:84`),
which is what makes `guess: true` seeding (below) roughly correct for flat lists. A change
detection pass over the previous snapshot gates the actual publish
(`packages/react/src/internals/composite/list/CompositeList.tsx:146-168`), so identity-stable
registrations republish nothing.

Item-side, `useCompositeListItem` seeds its index two ways. Explicit `index` short-circuits
everything (`index = externalIndex ?? internalIndex`,
`packages/react/src/internals/composite/list/useCompositeListItem.ts:55`). Without one, a
`guess: true` item claims `nextIndexRef.current` during the *first render* via a `useState`
initializer (`packages/react/src/internals/composite/list/useCompositeListItem.ts:42-55`) —
that is the mechanism behind "assigns correct indexes during the first render without a
corrective re-render" (behavior spec, "State model"): the index is known at render time so
`data-index` is right on the first client paint, and SSR renders `-1` because the initializer
runs on the server too but the registry is empty there. Non-guessed items instead subscribe via
`subscribeMapChange` and copy their published index into state
(`packages/react/src/internals/composite/list/useCompositeListItem.ts:84-96`), which is the
corrective re-render path.

Two subtle mechanisms in the callback ref:

- The ref is *deliberately identity-sensitive* on `metadata`/`label`/`textRef`
  (`packages/react/src/internals/composite/list/useCompositeListItem.ts:59-82`). Detaching and
  reattaching the callback ref is the re-registration signal, so changing one of those values
  re-registers the item; this is also why the JSDoc asks callers to keep those referentially
  stable (`packages/react/src/internals/composite/list/useCompositeListItem.ts:15-21`). The
  edge-case test for "changing an explicit index re-registers … while automatic index shifts do
  NOT detach" (behavior spec, "Edge cases") follows directly: `externalIndex` changes the ref's
  captured closure (re-register at the new slot), while a pure published-index change only flows
  through the subscription state update, not the ref.
- Registration ordering decides ownership when two items share one DOM node: the ref unregisters
  the previous node then registers the new one
  (`packages/react/src/internals/composite/list/useCompositeListItem.ts:62-82`), and since
  callback refs attach outer-component-first, the outer registration wins — the comment rules out
  replacing this with an effect-based publish. `CompositeItem` additionally places `compositeRef`
 *first* in its merged ref array for the same reason
  (`packages/react/src/internals/composite/item/CompositeItem.tsx:27-33`).

`subscribeMapChange` is a plain listener `Set` on the list instance
(`packages/react/src/internals/composite/list/CompositeList.tsx:202-207`), separate from the
`onMapChange` prop so items can observe without forcing the root to re-render per item.

Out-of-React DOM moves are handled by `observe()`
(`packages/react/src/internals/composite/list/CompositeList.tsx:89-140`): one `MutationObserver`
per *adjacent-pair common ancestor* — a reorder must invert at least one adjacent pair, so
observing each pair's common ancestor catches both direct item moves and wrapper moves with
exactly the roots needed (`packages/react/src/internals/composite/list/CompositeList.tsx:128-139`),
which is the "one `observe` call" claim in behavior spec ("DOM structure & portal behavior"). The
callback filters out pure add/remove batches via `hasMovedNode`
(`packages/react/src/internals/composite/list/CompositeList.tsx:286-296`) and only fires when a
connected node now sits before its previously-connected predecessor; ordering itself is
`compareDocumentPosition` with the `DOCUMENT_POSITION_FOLLOWING` bit
(`packages/react/src/internals/composite/list/CompositeList.tsx:298-302`), which handles sibling
and nested (shared-node) items uniformly.

The three cleanup/Strict-Mode effects matter for the Suspense/Strict behavior in the behavior
spec ("Edge cases"): the ref-objects effect re-copies the last committed snapshot when ref
objects change or effects replay without ref reattach
(`packages/react/src/internals/composite/list/CompositeList.tsx:172-185`), and the unmount effect
marks the map dirty so the Strict replay rebuilds everything
(`packages/react/src/internals/composite/list/CompositeList.tsx:193-200`).

### Highlight state machine (`useCompositeRoot`)

`highlightedIndex` is `externalHighlightedIndex ?? internalHighlightedIndex` with `useState(0)`
default (`packages/react/src/internals/composite/root/useCompositeRoot.ts:90` and
`packages/react/src/internals/composite/root/useCompositeRoot.ts:100`) — the controlled/uncontrolled
behavior in the behavior spec. Every index change funnels through one stable
`onHighlightedIndexChange` that snapshots the highlighted element into `highlightedElementRef`
and scrolls it into view on request
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:101-108`). That ref is the
identity anchor for the whole reconciliation model: indexes are positions in a shifting array,
elements are stable.

`onMapChange` (fed by `CompositeList`) is the reconciliation pump and runs in two phases
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:110-164`):

- First population (`hasSetDefaultIndexRef` false): adopt an item carrying
  `data-composite-item-active` as the initial tab stop (attribute constant in
  `packages/react/src/internals/composite/constants.ts:1`), else move off a disabled index-0 to
  the first enabled index (`packages/react/src/internals/composite/root/useCompositeRoot.ts:137-163`).
- Subsequent populations: `elements.indexOf(highlightedElementRef.current)` decides. Same element
  at a shifted index → follow it (the "removing an item before the highlighted one keeps the tab
  stop on the same element" behavior). Element gone → keep the replacement at the same index if
  eligible, else `getFallbackIndex` (active item if focusable, else first enabled, else 0 so an
  all-disabled list stays in range; `packages/react/src/internals/composite/root/useCompositeRoot.ts:342-365`).

Because `disabledIndices` can arrive a render late (the comment cites Toolbar deriving them from
metadata), a separate layout effect re-validates the default tab stop whenever
`disabledIndices`/`highlightedIndex` change, gated to uncontrolled mode after first population
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:166-193`).

Keydown (`packages/react/src/internals/composite/root/useCompositeRoot.ts:206-317`) is a filter
pipeline: key whitelist (`COMPOSITE_KEYS`, `packages/react/src/internals/composite/composite.ts:22`)
with opt-in Home/End → modifier gate via `event.getModifierState` minus the allow-list
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:367-377`, giving the
`modifierKeys` allow-list semantics) → native-input yield (arrow within a non-collapsed selection
or mid-text returns control to the textbox, `packages/react/src/internals/composite/root/useCompositeRoot.ts:229-246`)
→ target computation. Target computation differs by mode: grid mode delegates entirely to the
injected `grid` navigator with cell-map math (`packages/react/src/internals/composite/root/useCompositeRoot.ts:252-265`);
list mode computes forward/backward keys with RTL mapping
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:221-226`) and, when the neighbor
is the same index, either loops (min↔max swap, invoking `onLoop` whose return value overrides the
target — hence `onLoop`'s "return prevIndex to stay" contract) or searches the next non-disabled
index (`packages/react/src/internals/composite/root/useCompositeRoot.ts:282-300`). Commit is
guarded to an actual index change and then: optional `stopPropagation`, `preventDefault` for
in-orientation keys, `onHighlightedIndexChange(next, true)` (which scrolls), and `focus()` inside
`queueMicrotask` — deliberately deferred so Floating UI FocusManager's `returnFocus` runs first
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:302-316`). This microtask
deferral plus the stable callback exposed as `relayKeyboardEvent`
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:338`) is the whole external-key
relay story documented in `CompositeRootContext`
(`packages/react/src/internals/composite/root/CompositeRootContext.ts:8-14`).

The root's own `onFocus` selects the entire value of a focused native input
(`setSelectionRange(0, value.length)`, `packages/react/src/internals/composite/root/useCompositeRoot.ts:321-328`)
— that is what makes the "first arrow key returns control to the textbox" test pass: the
full-selection state is what the keydown yield-check (three conditions above) reads.

Grid math lives in a closure built by `gridNavigation(config)`
(`packages/react/src/internals/composite/root/gridNavigation.ts:50-126`): importing/calling it is
the tree-shaking opt-in for grid support (its doc comment says so explicitly). It converts item
indices to hypothetical 1×1 cell indices, treats gaps as disabled, picks the corner of the
spanning item closest to the movement direction, and lets floating-ui's `getGridNavigatedIndex`
do the actual step.

`CompositeRoot` itself is thin wiring: `useDirection()` → `useCompositeRoot` → `useRenderElement`
→ `CompositeRootContext.Provider` wrapping a `CompositeList`, fanning `onMapChange` to both the
consumer prop and the reconciliation callback
(`packages/react/src/internals/composite/root/CompositeRoot.tsx:42-95`). The wrapper renders
whatever `tag`/`render` says, which is why it adds no ARIA of its own.

### `useButton`

Stateless except refs. Composition:

- Composite detection is *context-inferred*: `composite ?? (useCompositeRootContext(true) !== undefined)`
  (`packages/react/src/internals/use-button/useButton.ts:25-26`) — the optional-context accessor
  returns `undefined` outside a root (`packages/react/src/internals/composite/root/CompositeRootContext.ts:21-32`).
  That single line produces the "behaves as composite inside a CompositeRoot even when `composite`
  is unspecified" behavior in the behavior spec ("Keyboard interactions").
- Disabled/focusability attribute policy is delegated to `useFocusableWhenDisabled`
  (`packages/react/src/internals/use-button/useButton.ts:28-34`), which decides
  `disabled` vs `aria-disabled` vs `tabIndex` from `native`/`composite`/`focusableWhenDisabled`
  (`packages/react/src/utils/useFocusableWhenDisabled.ts:20-58`).
- `getButtonProps` merges: interaction guards → `type: 'button'` (native) or `role: 'button'`
  (non-native, `packages/react/src/internals/use-button/useButton.ts:226`) → disabled policy →
  consumer props (`packages/react/src/internals/use-button/useButton.ts:102-229`).

The keyboard click synthesis is the interesting state machine. Space/Enter never call
`element.click()`; they `dispatchClickWithModifiers` — a synthetic untrusted `click` PointerEvent
carrying the source event's modifier state (which `click()` cannot express), `composed: true` so
it crosses shadow boundaries, `detail: 0` per native keyboard-click convention
(`packages/react/src/utils/dispatchClickWithModifiers.ts:19-36`). Branches:

- Composite + Space on keydown: prevented default only suppresses the click for text-navigation
  roles (`menuitem*`, `option`, `gridcell`), a prevented keydown cancels, otherwise
  `preventBaseUIHandler()` + dispatch unless it's a non-native non-button element
  (`packages/react/src/internals/use-button/useButton.ts:138-152`).
- Non-composite: Enter dispatches on keydown; Space dispatches on keyup (the DOM's own rule);
  both honor `defaultPrevented` and `preventBaseUIHandler` as cancellation
  (`packages/react/src/internals/use-button/useButton.ts:155-176` and
  `packages/react/src/internals/use-button/useButton.ts:177-217`). The keyup handler also
  swallows Space keyup on native composite buttons (`preventDefault` + return) so the keydown
  click isn't doubled — the dedup mechanism behind "a full Space press fires exactly one click".
- `preventBaseUIHandler` is honored via the `mergeProps` event pipeline: handlers are
  `makeEventPreventable`d and later handlers are skipped when
  `event.baseUIHandlerPrevented` is set (`packages/react/src/merge-props/mergeProps.ts:221-274`),
  and `useButton` re-checks the flag after running the consumer's keydown
  (`packages/react/src/internals/use-button/useButton.ts:121-125`).

`updateDisabled` force-clears a native `disabled` attribute on disabled *composite* buttons
(`packages/react/src/internals/use-button/useButton.ts:72-89`) for the disabled-renders-button
case, re-run from the stable `buttonRef` on every ref attach
(`packages/react/src/internals/use-button/useButton.ts:234-237`) — the "even after the button's
ref changes" clause of the focus-management behavior. Dev-only mismatch warnings
(`nativeButton` vs actual tag) run in an effect with owner-stack capture
(`packages/react/src/internals/use-button/useButton.ts:36-66`).

### `useAnchorPositioning`

A middleware *builder* around the vendored floating-ui `useFloating`
(`packages/react/src/internals/useAnchorPositioning.ts:128-132`; the public entry injects
`useBaseUIFloating`, and the test-visible `useAnchorPositioningWithHook` accepts any
`useFloating`-shaped hook — that injection is the seams the tests use). Only two pieces of
hook state exist: `mountSide` for `lazyFlip` sticky-side locking
(`packages/react/src/internals/useAnchorPositioning.ts:164-168` and
`packages/react/src/internals/useAnchorPositioning.ts:588-592`), and the `arrowRef` ref. Everything
else is derived per render.

Middleware pipeline, in order: caller `inline` middleware (Preview Card's line-box override) →
`offset` (function-based so `sideOffset`/`alignOffset` callbacks read live rects; logical→physical
side mapping is RTL-aware, `packages/react/src/internals/useAnchorPositioning.ts:52-61` and
`packages/react/src/internals/useAnchorPositioning.ts:250-278`) → `flip`/`shift` in a
collision-avoidance-dependent order (`shift` preferences and centered alignment put shift first,
per the floating-ui combining note, `packages/react/src/internals/useAnchorPositioning.ts:339-348`)
→ `size` (writes `--available-width/height` vars plus DPR-snapped `--anchor-width/height`,
`packages/react/src/internals/useAnchorPositioning.ts:350-371`) → `arrow` (substituting a fake
element when no arrow is rendered so transform-origin still has an anchor,
`packages/react/src/internals/useAnchorPositioning.ts:372-382`) → a custom `transformOrigin`
middleware writing the CSS var from arrow/alignment/shift geometry
(`packages/react/src/internals/useAnchorPositioning.ts:383-446`) → `hide` → `adaptiveOrigin`.
The flip bias of 1px on the preferred side exists for the iOS software-keyboard centering bug
(`packages/react/src/internals/useAnchorPositioning.ts:223-232`).

Anchor registration is two effects because refs from parent components populate *after* layout
effects: a `useIsoLayoutEffect` resolves function/ref anchors immediately, and a `useEffect`
re-checks plain refs once their `.current` is populated; both dedupe against a
`registeredPositionReferenceRef` (`packages/react/src/internals/useAnchorPositioning.ts:537-578`).
`keepMounted` popups get their floating-root context nulled while closed so positioning doesn't
run on hidden elements (`packages/react/src/internals/useAnchorPositioning.ts:451-462`), and
`whileElementsMounted: autoUpdate` is only passed in the mount/unmount mode — `keepMounted` mode
starts `autoUpdate` from a separate effect (`packages/react/src/internals/useAnchorPositioning.ts:486-495`
and `packages/react/src/internals/useAnchorPositioning.ts:573-578`).

`floatingStyles` is memoized output with deliberate pre-positioning behavior: `position: fixed`
until `isPositioned` (prevents `autoFocus` scroll jumps), `top/left: 0` + `opacity: 0` so stale
coordinates from a previous open can't overflow the mobile viewport, and the `--available-*`
seeds are set unconditionally so React's style diff never removes the imperative values `size()`
writes (`packages/react/src/internals/useAnchorPositioning.ts:499-533`).

### Element rendering (`useRenderElement`) and transition state

`useRenderElement` = props merge + element evaluation. Props: `getStateAttributesProps` turns
state into `data-*` attributes with per-key custom mapping override
(`packages/react/src/internals/getStateAttributesProps.ts:5-31` — truthy → attribute, `true` →
empty value; `transitionStatusMapping` is the canonical custom mapping, emitting
`data-starting-style`/`data-ending-style` hooks and `null` (no attributes) for idle/undefined,
`packages/react/src/internals/stateAttributesMapping.ts:7-20`). Refs are merged with the render
element's own ref via `useMergedRefs(N)` — conditionally called but hook-order-stable by design,
skipped on the server (`packages/react/src/internals/useRenderElement.tsx:94-104`). Evaluation
(`evaluateRenderProp`, `packages/react/src/internals/useRenderElement.tsx:158-206`): render
function → `render(props, state)` (with the uppercase-name hook-safety warning,
`packages/react/src/internals/useRenderElement.tsx:208-230`); render element → `cloneElement`
with merged props and the internal ref; else default tag — with `button` forced to
`type="button"` and `img` forced to `alt=""` at the JSX level
(`packages/react/src/internals/useRenderElement.tsx:232-240`). `enabled: false` returns `null`
and short-circuits prop computation while still calling the ref hook to keep order stable
(`packages/react/src/internals/useRenderElement.tsx:40-47`). Lazy/Flight-shaped render props are
unwrapped via `React.Children.toArray` before `.props`/`.ref` are read, only while enabled
(`packages/react/src/internals/useRenderElement.tsx:32-36` and
`packages/react/src/internals/useRenderElement.tsx:145-156`).

`useTransitionStatus` is a render-phase-derived state machine
(`packages/react/src/internals/useTransitionStatus.ts:31-42`): `open && !mounted` → mount +
`'starting'` in the same render pass (React re-renders before commit, which is why `mounted`
starts `false`); `!open && mounted` → `'ending'` (deferred one frame via `AnimationFrame` when
`deferEndingState`, `packages/react/src/internals/useTransitionStatus.ts:44-56`); unmounted+ending
→ back to `undefined`. `enableIdleState` adds `starting → idle` one frame after open
(`packages/react/src/internals/useTransitionStatus.ts:58-90`). `getDisabledMountTransitionStyles`
maps `'starting'` to the shared `transition: none` style constant
(`packages/react/src/internals/getDisabledMountTransitionStyles.ts:5-9` and
`packages/react/src/internals/constants.ts:5`).

`useAnimationsFinished` returns a stable runner built on `getAnimations()`
(`packages/react/src/internals/useAnimationsFinished.ts:51-158`): wait one animation frame (or a
`MutationObserver` on the `data-starting-style` attribute when `waitForStartingStyleRemoved`), await
all `animation.finished` promises, retry the whole `exec()` when animations are replaced
mid-flight (the cancellation-then-replacement case), and run the callback in `flushSync` — unless
`batch: true`, in which case callbacks are collected into a module-level `pendingCallbacks` array
flushed once per microtask checkpoint inside a single `flushSync`
(`packages/react/src/internals/useAnimationsFinished.ts:9-32`). The module-level batching is why
"batched callbacks finishing in the same microtask commit once" while default mode gives each
callback its own commit (later completions observe earlier updates). `useOpenChangeComplete` is a
thin effect wrapper adding per-run `AbortController` cancellation
(`packages/react/src/internals/useOpenChangeComplete.tsx:15-27`).

### Small hooks

- `useValueChanged`: previous-value ref + `useIsoLayoutEffect` with strict `!==` (so `0 → -0`
  never fires, `Object.is` semantics not applied — the *opposite* of `itemEquality`,
  `packages/react/src/internals/useValueChanged.ts:6-17`).
- `useBaseUiId`: `useId(idOverride, 'base-ui')` prefix wrapper
  (`packages/react/src/internals/useBaseUiId.ts:9-11`).
- `usePressAndHold`: pointer-type-aware hold-repeat machine — immediate `tick()` on
  pointerdown, then `useTimeout` (start delay) → `useInterval` (tick delay) — with a
  touch-intent check (fewer than 3 pointer moves within 50ms and still pressed), a global
  `pointerup` listener so holds ending off-element still stop, a global `contextmenu` preventer
  for touches outside the hit area, and `shouldSkipClick` for mouse (`detail !== 0`) vs touch
  (synthesized-click suppression) (`packages/react/src/internals/usePressAndHold.ts:80-286`).

### Pure units

- `RequestQueue` (`packages/react/src/internals/RequestQueue.ts:26-126`): two maps (`queued` FIFO
  via `Map` insertion order, `pending`), `processQueue` fills spare concurrency slots from
  `pickEntries` (a protected method — the documented subclass override point for ordering, not
  exercised by tests), and a rejected `fetchFn` is caught to just delete the pending entry. Keys
  are stringified through `getKeyId ?? String`.
- `TimeoutManager` (`packages/react/src/internals/TimeoutManager.ts:4-29`): `Map<key, id>`;
  `start` clears first (replacement semantics), callbacks delete their own entry, class-internal
  raw `setTimeout` (not the `useTimeout` hook — this is a non-React utility).
- `getFilter` (`packages/react/src/internals/filter.ts:4-65`): module-level `Map` cache keyed by
  `stringifyLocale(locale)|JSON(options)` wrapping one `Intl.Collator` (search usage, base
  sensitivity, punctuation-insensitive) with sliding-window `contains` and slice-based
  `startsWith`/`endsWith`.
- `itemEquality` (`packages/react/src/internals/itemEquality.ts`): null-safe comparison wrapper
  over a caller comparer (`compareItemEquality`), `Object.is`-semantics default
  (`packages/react/src/internals/itemEquality.ts:9-10`), array-aware dirty check via
  `areArraysEqual`, and the `findSelectionIndex` fast path that builds a `Set` only for the
  default comparer with an explicit `+0/-0` re-check because `Set` collapses them
  (`packages/react/src/internals/itemEquality.ts:71-84`) — that is the mechanism behind the
  "`+0` and `-0` are distinct" behavior and the read-once selection anchoring.
- `resolveValueLabel` (`packages/react/src/internals/resolveValueLabel.tsx`): group detection by
  actual `items` array (`packages/react/src/internals/resolveValueLabel.tsx:22-28`), record lookup
  guarded by `Object.hasOwn` (prototype-member safety), and stringification via `serializeValue`
  (`packages/react/src/internals/serializeValue.ts:1-13`, JSON with `String` fallback).
- `createBaseUIEventDetails` (`packages/react/src/internals/createBaseUIEventDetails.ts:118-166`):
  closure-based `canceled`/`allowPropagation` flags surfaced as getters, defaulting the event to
  a bare `Event('base-ui')`; `reasons.ts`/`reason-parts.ts` are the reason-string registry, and
  `ReasonToEventMap` type-narrows `details.event` per reason. The `.spec.ts` files for this and
  `useRenderElement` are compile-time `expectType` type specs, not runtime tests.
- Temporal adapters are thin `TemporalAdapter` implementations over `date-fns` + `@date-fns/tz`
  (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:102-218`) and
  `luxon` (`packages/react/src/internals/temporal-adapter-luxon/TemporalAdapterLuxon.ts:50-109`).
  The date-fns `date()` is where the date-only-vs-datetime parsing rule comes from: date-only
  strings are read back with UTC getters (spec parses them as UTC midnight) and rebuilt
  face-value in the target zone via the `TZDate` multi-arg constructor, datetime strings use
  local getters (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:125-165`).
  `setTimezone` duck-types any object with `withTimeZone` before falling back to `new TZDate`
  (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:198-211`) —
  cross-copy TZDate compatibility. The shared type vocabulary is `temporal/temporal.ts`
  (module-augmentation registry `TemporalSupportedObjectLookup`,
  `packages/react/src/internals/temporal/temporal.ts:13-21`) and the ~60-method
  `temporal/temporal-adapter.ts` interface.

## Context providers/consumers

| Context | Provider in this unit | Consumers in this unit | Boundary contents |
|---|---|---|---|
| `CompositeListContext` | `CompositeList` (`packages/react/src/internals/composite/list/CompositeList.tsx:209-216`) | `useCompositeListItem` (`packages/react/src/internals/composite/list/useCompositeListItem.ts:37`) | `register`/`unregister`/`subscribeMapChange`/`nextIndexRef` (`packages/react/src/internals/composite/list/CompositeListContext.ts:11-23`); defaults are no-ops so orphan items render with `data-index="-1"` instead of crashing |
| `CompositeRootContext` | `CompositeRoot` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:83-95`) | `useCompositeItem` (highlight + hover, `packages/react/src/internals/composite/item/useCompositeItem.ts:17-18`); `useButton` optional (composite inference, `packages/react/src/internals/use-button/useButton.ts:25-26`) | `highlightedIndex`, `onHighlightedIndexChange`, `highlightItemOnHover`, `relayKeyboardEvent` — items need no props to participate |
| `DirectionContext` | (provider lives elsewhere, e.g. `DirectionProvider`) | `useDirection` in `useCompositeRoot` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:42`) and `useAnchorPositioning` (`packages/react/src/internals/useAnchorPositioning.ts:182-183`) | `'ltr' | 'rtl'`, defaulted to `'ltr'` in the accessor (`packages/react/src/internals/direction-context/DirectionContext.tsx:12-16`) |
| `CSPContext` | (provider lives at app level) | `PrehydrationScript` for the inline-script `nonce` (`packages/react/src/internals/PrehydrationScript.tsx:34`), `useCSPContext` defaulting `disableStyleElements: false` (`packages/react/src/internals/csp-context/CSPContext.tsx:11-17`) | `{ nonce, disableStyleElements }` |
| `FieldRootContext` | (provider is `Field.Root`, not here) | `useRegisterFieldControl` (`packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:15`) | Full field state + `registerFieldControl(source, registration)`; context default is an all-`NOOP` shell so hooks work outside a field (`packages/react/src/internals/field-root-context/FieldRootContext.ts:32-63`) |
| `FormContext` | (provider is `Form`) | `useFieldControlRegistration` for `formRef` (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:30`) | `formRef.current.fields: Map<id, {getValue, name, controlRef, validityData, validate}>` (`packages/react/src/internals/form-context/FormContext.ts:13-28`) — the imperative registry Form submits against |
| `LabelableContext` | `LabelableProvider` (`packages/react/src/internals/labelable-provider/LabelableProvider.tsx:106-108`), nestable | `useLabelableId`, `useLabel`, `useAriaLabelledBy`; nested providers read parent `messageIds` (`packages/react/src/internals/labelable-provider/LabelableProvider.tsx:24`) | `controlId` (multi-source: symbol-keyed registration map, `packages/react/src/internals/labelable-provider/LabelableProvider.tsx:22-60`), `labelId`, `messageIds`, `getDescriptionProps` |

Cross-boundary state notes:

- `LabelableProvider.controlId` is a *last-registered-wins with sticky selection* reducer over
  symbol-keyed control registrations: an empty registration map preserves the current id (React
  Activity/Suspense hidden subtrees keep their DOM but lose effects,
  `packages/react/src/internals/labelable-provider/LabelableProvider.tsx:36-58`), and an explicit
  `null` registration deliberately suppresses `htmlFor` (aria-labelledby cases).
- `useLabelableId` returns `controlId ?? id ?? defaultId` — provider pre-registration state wins
  so SSR `htmlFor` pairs up (`packages/react/src/internals/labelable-provider/useLabelableId.ts:79-82`),
  and it unregisters in a layout-phase cleanup so a replacement control's layout effect never
  observes the outgoing registration (`packages/react/src/internals/labelable-provider/useLabelableId.ts:73-77`).
- `useFieldControlRegistration.register` keys ownership by a per-control `Symbol` source
  (`useRefWithInit(() => Symbol())` in `useRegisterFieldControl`,
  `packages/react/src/internals/field-register-control/useRegisterFieldControl.ts:16-22`); a
  replaced control's registration is dropped and `change(undefined, true)` cancels pending work,
  re-registration updates the Form's Map entry in place (delete+re-add would reorder the Map and
  thus submit ordering), and the field-level `initialValue` baseline is captured exactly once per
  field, not per control instance
  (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:86-103`).

## DOM/portal strategy and why

- **No portal in this unit.** `CompositeRoot` renders one wrapper element via `useRenderElement`
  (`packages/react/src/internals/composite/root/CompositeRoot.tsx:66-71`); items render wherever
  the consumer puts them, possibly through `ReactDOM.createPortal` into a detached shadow root —
  the registry doesn't care because everything is keyed by element identity and verified with
  `isConnected`/`compareDocumentPosition`, which is what makes the portal test in behavior spec
  ("DOM structure & portal behavior") pass without special handling. `PrehydrationScript` renders
  an inline `<script>` server-side; portals for popups belong to the popup components that
  consume `useAnchorPositioning`, not to this unit.
- **`MutationObserver` instead of React reconciliation for ordering** — described under the
  composite registry above; the design constraint is that DOM moves (reorders, portal reparents,
  out-of-React surgery) don't produce React commits, so the observer on adjacent-pair common
  ancestors is the only reliable signal. `getCommonAncestor` uses native `contains` because the
  `parentElement` walk cannot cross shadow boundaries anyway
  (`packages/react/src/internals/composite/list/CompositeList.tsx:274-284`).
- **Manual scroll math instead of `scrollIntoView`**: `scrollIntoViewIfNeeded` computes offsets by
  walking `offsetParent` up to the scroll container and reads `scroll-margin-*`/`scroll-padding-*`
  from computed styles, then issues one `scrollTo({behavior: 'auto'})`
  (`packages/react/src/internals/composite/composite.ts:44-173`). This avoids scrolling *all*
  ancestor scroll chains (native `scrollIntoView` scrolls every ancestor), gives exact RTL
  semantics (the RTL branch checks the left edge first because scrollLeft is negative in RTL), and
  respects both CSS scroll snap margins/paddings that the native API handles differently per
  browser.
- **Floating positioning**: the positioner element receives `positionerStyles` (see
  `useAnchorPositioning` above); the anchor is registered via `refs.setPositionReference` rather
  than the DOM reference so virtual elements and function anchors work; `hide` middleware exposes
  `anchorHidden` for consumers. CSS custom properties (`--available-width/height`,
  `--anchor-width/height`, transform-origin var) are the contract between this hook and consumer
  stylesheets.
- **`PrehydrationScript` browser-bundle strategy**: the script body is imported through a
  `#prehydration/*` subpath whose `browser` condition resolves to the shared empty stub
  (`packages/react/src/internals/prehydrationScript.stub.ts:1-6`), so the script source only ever
  exists in server bundles; the component must still render an empty script element during the
  hydration pass (`suppressHydrationWarning` bridges the content difference) or React reports a
  hydration mismatch against the server markup — the long doc comment at
  `packages/react/src/internals/PrehydrationScript.tsx:6-31` is the authoritative rationale. The
  `isHydrating` signal is `useSyncExternalStore` with `getServerSnapshot: true` /
  `getSnapshot: false` (imported from `packages/react/src/utils/useIsHydrating.ts`), which
  intrinsically returns true only during hydration of server markup.
- **Event synthesis over `element.click()`**: `dispatchClickWithModifiers` (see `useButton`
  above) — `composed: true` keeps shadow-DOM hosts working, modifier preservation keeps
  modifier-aware consumers working, and `detail: 0` marks it as keyboard-generated.
- **`useRenderElement` tag defaults**: `renderTag` force-defaults `type="button"` on rendered
  buttons and `alt=""` on images (`packages/react/src/internals/useRenderElement.tsx:232-240`),
  making safe defaults impossible to forget at the component level.

## Dependencies on other Base UI internals

This unit is the *base* of the dependency graph, but it is not a leaf — the following are the
concrete imports a port must replicate or replace. (This section is the precise per-unit data the
coarse `blocked-by` default in `ralph/scripts/generate-todo.mjs` should eventually be derived
from.)

**Vendored `packages/react/src/floating-ui-react/` (in-repo, largest dependency):**

- `composite/composite.ts` re-exports `stopEvent`, `isIndexOutOfListBounds`,
  `isListIndexDisabled`, `findNonDisabledListIndex`, `getMaxListIndex`, `getMinListIndex` from
  `floating-ui-react/utils` (`packages/react/src/internals/composite/composite.ts:4-11`).
- `gridNavigation.ts` imports `createGridCellMap`, `getGridCellIndexOfCorner`,
  `getGridCellIndices`, `getGridNavigatedIndex`, `isListIndexDisabled` from
  `floating-ui-react/utils/composite` (`packages/react/src/internals/composite/root/gridNavigation.ts:2-8`).
- `useCompositeRoot` imports `getTarget` (event-target resolution,
  `packages/react/src/internals/composite/root/useCompositeRoot.ts:30`); `useLabel` imports the
  same (`packages/react/src/internals/labelable-provider/useLabel.ts:6`).
- `useAnchorPositioning` imports the middleware set (`offset`, `flip`, `shift`, `size`, `limitShift`,
  `autoUpdate`), `arrow` middleware, `useBaseUIFloating`, and types (`FloatingRootContext`,
  `FloatingTreeStore`, …) from `floating-ui-react`
  (`packages/react/src/internals/useAnchorPositioning.ts:8-33`); it also imports `hide` from
  `packages/react/src/utils/hideMiddleware.ts` and constants from
  `packages/react/src/utils/adaptiveOriginConstants.ts` / `CommonPositionerCssVars.ts`.
- `@floating-ui/utils/dom` (`isHTMLElement`) is imported directly by `useButton` and `useLabel`.

**`@base-ui/utils/*` (public shared utils package):** `empty` (`EMPTY_OBJECT`/`EMPTY_ARRAY`/`NOOP`),
`useMergedRefs`(+`useMergedRefsN`), `useIsoLayoutEffect`, `useStableCallback`, `useTimeout`,
`useInterval`, `useAnimationFrame` (+`AnimationFrame`), `useRefWithInit`, `useValueAsRef`,
`useId`, `owner` (`ownerDocument`/`ownerWindow`), `isElementDisabled`, `error`, `warn`, `SafeReact`
(owner stacks), `stringifyLocale`, `areArraysEqual`, `mergeObjects`, `getReactElementRef`.
Module-level import sites are consistent across files; e.g.
`packages/react/src/internals/composite/list/CompositeList.tsx:4-6`,
`packages/react/src/internals/use-button/useButton.ts:3-7`,
`packages/react/src/internals/useAnimationsFinished.ts:4-5`,
`packages/react/src/internals/filter.ts:1`.

**`packages/react/src/` private modules (not in `internals/`):**

- `merge-props` — `mergeProps`/`mergePropsN`/`makeEventPreventable`/`mergeClassNames`: the
  `preventBaseUIHandler` contract implementation used by `useButton` and `useRenderElement`
  (`packages/react/src/internals/use-button/useButton.ts:8`,
  `packages/react/src/internals/useRenderElement.tsx:11`).
- `utils/useFocusableWhenDisabled` + `utils/dispatchClickWithModifiers`
  (`packages/react/src/internals/use-button/useButton.ts:11-12`).
- `utils/resolveClassName`/`resolveStyle` (`packages/react/src/internals/useRenderElement.tsx:9-10`).
- `utils/resolveRef` (`packages/react/src/internals/useAnimationsFinished.ts:6`).
- `utils/useIsHydrating` (`packages/react/src/internals/PrehydrationScript.tsx:3`).
- `utils/useRegisteredLabelId` (`packages/react/src/internals/labelable-provider/useLabel.ts:7`).
- `field/utils/getCombinedFieldValidityData`
  (`packages/react/src/internals/field-register-control/useFieldControlRegistration.ts:5`), plus
  types from `field/root/FieldRoot` and `field/control/FieldControlDataAttributes`
  (`packages/react/src/internals/field-constants/constants.ts:1-2`),
  `field/root/useFieldValidation` (`packages/react/src/internals/field-root-context/FieldRootContext.ts:8`),
  and `form`/`form/Form` types.

**Cross-references inside this unit** (intra-unit coupling a port must preserve): `CompositeItem`
→ `useCompositeItem` → {`CompositeRootContext`, `useCompositeListItem` → `CompositeListContext`};
`CompositeRoot` → {`useCompositeRoot`, `CompositeList`, `useRenderElement`, `DirectionContext`};
`useButton` → `CompositeRootContext` (optional); `useRenderElement` →
`getStateAttributesProps`; `stateAttributesMapping` → `useTransitionStatus` types +
`TransitionStatusDataAttributes`; `useOpenChangeComplete` → `useAnimationsFinished`;
`useRegisterFieldControl` → `FieldRootContext`; `useFieldControlRegistration` → `FormContext`;
`LabelableProvider`/`useLabelableId`/`useLabel`/`useAriaLabelledBy` → `LabelableContext` +
`useBaseUiId`; `PrehydrationScript` → `CSPContext`; `filter` → `resolveValueLabel`.

**External npm packages (non-React):** `date-fns` + `@date-fns/tz` (TZDate) for the date-fns
adapter, `luxon` for the Luxon adapter (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:2-60`,
`packages/react/src/internals/temporal-adapter-luxon/TemporalAdapterLuxon.ts:6`),
`use-sync-external-store/shim` via `utils/useIsHydrating`.

## Anything in source not explained by any test

Behavior spec's "Shared harness dependencies" and its per-section citations cover: composite
(root/list/item/composite.ts), `useButton`, `useAnchorPositioning`, `useAnimationsFinished`,
`useRenderElement`, `useValueChanged`, `RequestQueue`, `TimeoutManager`, `filter`,
`getStateAttributesProps`, `itemEquality`, `resolveValueLabel` (partial), `stateAttributesMapping`,
temporal adapters (via the shared harness). Everything below has **no test coverage in this
unit** — the golden-fixture stage and backward-looking audit should treat these as unverified
behavior:

1. **`usePressAndHold`** (`packages/react/src/internals/usePressAndHold.ts`, all 286 lines): no
   test file exists for it and behavior spec never mentions it. The touch-intent threshold,
   pen-as-touch handling, global contextmenu suppression, `shouldSkipClick` semantics, and
   start/tick delays are entirely unverified here.
2. **`useOpenChangeComplete`** and **`useTransitionStatus`**: no unit tests; both are exercised
   only indirectly through popup component tests outside this unit.
3. **`useBaseUiId`**, **`noop.ts`**, **`getDisabledMountTransitionStyles`**,
   **`serializeValue.ts`**: no dedicated tests.
4. **Context units**: `csp-context`, `direction-context`, `form-context`,
   `field-root-context`, `field-constants` have no tests in this unit; `field-register-control`
   (both hooks, 230 lines) and the whole `labelable-provider` directory
   (`LabelableProvider`, `useAriaLabelledBy`, `useLabel`, `useLabelableId` — ~350 lines including
   the DOM label-discovery fast paths) are untested here.
5. **`PrehydrationScript`**: untested in this unit (its hydration behavior is verified in the
   components that use it, if at all).
6. **Composite features without tests here:** `highlightItemOnHover` hover-focus
   (`packages/react/src/internals/composite/item/useCompositeItem.ts:31-41`), `stopEventPropagation`
   and `rootRef` props, `relayKeyboardEvent` as a public capability, and the
   `PAGE_UP`/`PAGE_DOWN` key constants (`packages/react/src/internals/composite/composite.ts:19-20`,
   exported but not in `COMPOSITE_KEYS` and not handled by `useCompositeRoot`).
7. **`useAnchorPositioning` internals beyond the shift-config forwarding** covered by behavior
   spec ("Events"): the `transformOrigin` middleware, `adaptiveOrigin` passthrough, `lazyFlip`
   sticky-side locking, iOS flip bias, keepMounted context nulling, `positionMethod`/`isPositioned`
   fallback styles, and the arrow fake-element substitution have no assertions in this unit's
   tests.
8. **`RequestQueue.pickEntries`** protected ordering override
   (`packages/react/src/internals/RequestQueue.ts:43-55`): documented extension point, untested
   (only default FIFO order is).
9. **`resolveValueLabel` exports** `resolveMultipleLabels`, `stringifyAsValue`,
   `stringifyAsLabel`, `flattenLeafItems` are not directly asserted (behavior spec's API list
   covers only `isGroupedItems`, `hasNullItemLabel`, `resolveSelectedLabel`).
10. **`internals/constants.ts`** exports beyond `DISABLED_TRANSITIONS_STYLE` —
    `TYPEAHEAD_RESET_MS`, `PATIENT_CLICK_THRESHOLD`, `CLICK_TRIGGER_IDENTIFIER`,
    `BASE_UI_SWIPE_IGNORE_ATTRIBUTE`/selector (and its legacy alias),
    `DROPDOWN_COLLISION_AVOIDANCE`, `POPUP_COLLISION_AVOIDANCE`, `ownerVisuallyHidden` — are
    consumed by other units and untested here
    (`packages/react/src/internals/constants.ts:3-41`).
11. **Type-level machinery**: `types.ts` (`WithBaseUIEvent`, `BaseUIComponentProps`,
    `NativeButtonProps`/`NonNativeButtonProps`, `Simplify`, `RequiredExcept`) and the
    `ReasonToEventMap` narrowing in `createBaseUIEventDetails.ts` are compile-time only; the
    `.spec.ts` files are `expectType` compile specs, not runtime tests.
12. **`useCompositeRoot` late-`disabledIndices` re-validation layout effect**
    (`packages/react/src/internals/composite/root/useCompositeRoot.ts:166-193`): its motivating
    scenario (Toolbar deriving disabled indices after map population) is asserted only in
    Toolbar's own tests, not here.
