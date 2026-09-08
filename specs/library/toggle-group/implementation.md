# toggle-group — implementation spec

Mined from this unit's non-test source files:

- `packages/react/src/toggle-group/index.ts`
- `packages/react/src/toggle-group/ToggleGroup.tsx`
- `packages/react/src/toggle-group/ToggleGroupContext.ts`
- `packages/react/src/toggle-group/ToggleGroupDataAttributes.ts`
- `packages/react/src/toggle-group/ToggleGroup.spec.tsx` (type-level only: proves `value`/`defaultValue` accept `readonly Value[]` while `onValueChange` receives a mutable `Value[]` — `packages/react/src/toggle-group/ToggleGroup.spec.tsx:4-8`, `packages/react/src/toggle-group/ToggleGroup.spec.tsx:19-27`)

This is the WHY/HOW companion to `specs/library/toggle-group/behavior.md` (the WHAT). Behavior claims are referenced by that doc's section names and not restated. The public surface is the `ToggleGroup` component plus its types, exported by `packages/react/src/toggle-group/index.ts:1-3`; there are no subcomponents in this unit.

## State machine / hooks used

The whole component is one state slot plus one reducer-shaped handler. There is no per-item state, no registration of item elements, and no effect in this unit at all — the group is purely: derive `disabled` → hold `value` → expose a transition callback → render.

**`useControlled`** — `packages/react/src/toggle-group/ToggleGroup.tsx:48-53`. The single source of truth for the pressed set (`groupValue`). It is controlled iff `value` is defined on the *first* render (the hook captures `controlled !== undefined` in a ref once, `packages/utils/src/useControlled.ts:41`); afterwards the setter it returns is a no-op in controlled mode (`packages/utils/src/useControlled.ts:82-89`), which is the mechanism behind behavior.md's UNVERIFIED "controlled-mode clicks" note: clicks do run the reducer and fire `onValueChange`, they just can't move state — only the parent's prop can. Uncontrolled state is seeded with `defaultValue` coerced to a shared frozen `EMPTY_ARRAY` when omitted (`packages/react/src/toggle-group/ToggleGroup.tsx:41`, `packages/utils/src/useControlled.ts:42`). The hook also owns the dev-only warnings for controlled↔uncontrolled flips and `defaultValue` mutation (`packages/utils/src/useControlled.ts:47-80`).

**`useStableCallback`** — wraps `setGroupValue` at `packages/react/src/toggle-group/ToggleGroup.tsx:55-81`. `setGroupValue(newValue, nextPressed, eventDetails)` is the reducer. The wrapper is what makes it legal to put the handler in the context memo's dependency list without invalidating the memo: the returned trampoline has a permanent identity while an insertion effect swaps in the latest render's closure (`packages/utils/src/useStableCallback.ts:35-51`), so the handler reads fresh `groupValue`/`multiple` at call time even though it was created in an earlier render. It also satisfies the repo rule that context-carried callbacks be stable (the call sites are Toggle click handlers, never render).

The transition table (all in `packages/react/src/toggle-group/ToggleGroup.tsx:61-71`):

| `multiple` | `nextPressed` | result |
|---|---|---|
| `false` | `true` | replace set with `[newValue]` (`packages/react/src/toggle-group/ToggleGroup.tsx:70`) |
| `false` | `false` | clear set to `[]` (`packages/react/src/toggle-group/ToggleGroup.tsx:70`) |
| `true` | `true` | `push` onto a `slice()` of the current set (`packages/react/src/toggle-group/ToggleGroup.tsx:63-65`) |
| `true` | `false` | `splice(indexOf(newValue), 1)` on that copy (`packages/react/src/toggle-group/ToggleGroup.tsx:66-67`) |

Two structural consequences:

- `multiple` is read at event time, not transformed eagerly — which is why behavior.md's "multiple transitions" bullet works: flipping the prop never touches the stored array, it only changes how *subsequent* events reduce it. The stored selection survives, and the next single-mode press replaces it wholesale.
- The commit is two-phase: `onValueChange` fires first (`packages/react/src/toggle-group/ToggleGroup.tsx:73`), then `setValueState` only if `!eventDetails.isCanceled` (`packages/react/src/toggle-group/ToggleGroup.tsx:75-79`). This is the entire veto machinery behind behavior.md's "Cancellation" bullets — there is no separate "cancel" path, just an early return between callback and commit. Note the callback receives the *computed* next array, not the current one.

**Derived values** (all plain render-time expressions, no hooks):

- `disabled` — three-way OR: component prop ∥ `ToolbarRootContext.disabled` ∥ `ToolbarGroupContext.disabled` (`packages/react/src/toggle-group/ToggleGroup.tsx:45-46`). The toolbar contexts are read at `packages/react/src/toggle-group/ToggleGroup.tsx:38-39`; the root lookup is deliberately optional (`useToolbarRootContext(true)` — the `true` overload returns `undefined` instead of throwing, `packages/react/src/toolbar/root/ToolbarRootContext.ts:12-23`), because nesting in a toolbar is an optional configuration, not a requirement.
- `isValueInitialized` — `valueProp !== undefined || defaultValueProp !== undefined` computed from the *raw* props, with the comment at `packages/react/src/toggle-group/ToggleGroup.tsx:42-43` explaining why: it distinguishes "no value props at all" from "an explicit empty default". It is exported through context solely so `Toggle` can gate its missing-`value` dev warning (behavior.md's State-model warning bullet) on the group actually having a defined value.
- `state` — `{ disabled, multiple, orientation }` (`packages/react/src/toggle-group/ToggleGroup.tsx:83`), the object that drives both the data-attribute emission and the `CompositeRoot` styling/render pipeline. `orientation` and `multiple` defaults come from the destructuring at `packages/react/src/toggle-group/ToggleGroup.tsx:26-30` (`'horizontal'` / `false`), matching the `@default` JSDoc on the props.

**Why pressed-ness is derivable instead of stored:** the group never learns which buttons exist. Each `Toggle` independently derives `pressed` as membership in `groupValue` (`packages/react/src/toggle/Toggle.tsx:63-68`), and on click sends `(value, nextPressed, details)` up through the context callback (`packages/react/src/toggle/Toggle.tsx:96-98`). The value array is the only shared encoding of "which items are pressed". This is also how value-less toggles work (behavior.md's Edge cases): a `Toggle` without a `value` falls back to its auto-generated id as its value (`packages/react/src/toggle/Toggle.tsx:43-44`), so it participates in the same array math under an ephemeral key.

## Context providers/consumers

**Provided: `ToggleGroupContext`** — created at `packages/react/src/toggle-group/ToggleGroupContext.ts:21-23`, memoized and provided at `packages/react/src/toggle-group/ToggleGroup.tsx:85-93` and `packages/react/src/toggle-group/ToggleGroup.tsx:107`. Payload is `{ value, setGroupValue, disabled, isValueInitialized }` (`packages/react/src/toggle-group/ToggleGroupContext.ts:6-19`), with `value` typed `readonly Value[]` to prevent consumers from mutating group state directly. The memo deps are `[disabled, setGroupValue, groupValue, isValueInitialized]` (`packages/react/src/toggle-group/ToggleGroup.tsx:92`) — the context re-provides exactly when the pressed set or effective disabled changes, which is what re-renders the deriving `Toggle`s.

**Crossing the boundary to children:**

- `value` → each Toggle's pressed derivation (membership test).
- `setGroupValue` → each Toggle's click handler. Critically, the `eventDetails` object is *created by the Toggle* (`createChangeEventDetails(REASONS.none, …)`, `packages/react/src/toggle/Toggle.tsx:86`) and shared through this call: the group's `onValueChange` gets the same object the Toggle's `onPressedChange` got, so a single `cancel()` vetoes both the group commit and the item's own commit (Toggle re-checks `details.isCanceled` after the group call, `packages/react/src/toggle/Toggle.tsx:100-104`). One details object = one veto chain covering item + group.
- `disabled` → merged with the item's own `disabled` prop (`packages/react/src/toggle/Toggle.tsx:48`).
- `isValueInitialized` → gate for the missing-`value` dev warning (`packages/react/src/toggle/Toggle.tsx:50-61`).

The only source-level consumer is `packages/react/src/toggle/Toggle.tsx:8` (`useToggleGroupContext`, defined at `packages/react/src/toggle-group/ToggleGroupContext.ts:25-27`). ToggleGroup itself never consumes its own context.

**Consumed (upstream), also via context:**

- `ToolbarRootContext` and `ToolbarGroupContext` — read at `packages/react/src/toggle-group/ToggleGroup.tsx:38-39`, used *only* for the disabled merge and for branch selection (`toolbarContext` truthiness at `packages/react/src/toggle-group/ToggleGroup.tsx:100` and `packages/react/src/toggle-group/ToggleGroup.tsx:108`).
- **Implicitly provided to children through `CompositeRoot`** (standalone branch only): `CompositeRootContext.Provider` + `CompositeList` wrap the element (`packages/react/src/internals/composite/root/CompositeRoot.tsx:83-95`). ToggleGroup passes nothing explicitly here — each `Toggle`'s `CompositeItem` registers itself with that list and picks up roving-tabindex behavior from the root context. This wrapper is how behavior.md's "Focus management" claims are produced without any focus code in this unit.

## DOM/portal strategy and why

- **One plain, non-portal `div` with `role="group"`** (`packages/react/src/toggle-group/ToggleGroup.tsx:95-97`, `packages/react/src/toggle-group/ToggleGroup.tsx:99`). No portal exists anywhere in the unit — a toggle group is an in-flow layout wrapper around its buttons, so there is nothing to teleport (consistent with behavior.md's "Portal behavior: N/A").
- **Two render branches, same DOM contract** (`packages/react/src/toggle-group/ToggleGroup.tsx:106-123`):
  - *Standalone*: `CompositeRoot` receives `render`/`className`/`style`/`state`/`refs` plus `loopFocus={loopFocus}`, `enableHomeAndEndKeys`, `orientation` (`packages/react/src/toggle-group/ToggleGroup.tsx:111-121`). CompositeRoot renders the same div through its own `useRenderElement` and wraps it in the composite providers (`packages/react/src/internals/composite/root/CompositeRoot.tsx:66-71` and `packages/react/src/internals/composite/root/CompositeRoot.tsx:83-95`). The composite wrapper is invisible in the DOM — it exists only to own the roving tabindex, arrow/Home/End handling, and RTL key mapping (behavior.md's Keyboard interactions and Focus management sections are entirely this machinery: RTL→arrow swap at `packages/react/src/internals/composite/root/useCompositeRoot.ts:221-226`, loop at `packages/react/src/internals/composite/root/useCompositeRoot.ts:282-300`, Home/End at `packages/react/src/internals/composite/root/useCompositeRoot.ts:274-280`, focus commit via microtask at `packages/react/src/internals/composite/root/useCompositeRoot.ts:313-315`).
  - *Toolbar-nested*: the bare `element` from ToggleGroup's own `useRenderElement`, gated by `enabled: Boolean(toolbarContext)` (`packages/react/src/toggle-group/ToggleGroup.tsx:99-104,108-109`). **Why:** when nested, the composite-root role already belongs to `Toolbar.Root`/`Toolbar.Group` — the items must register with *that* ancestor's `CompositeList`, and wrapping again would create a second, disconnected composite. Consequence: `loopFocus` and `enableHomeAndEndKeys` are inert in this branch (they are `CompositeRoot` parameters only); keyboard behavior is inherited from the toolbar. All data attributes still render, because `state` is passed to `useRenderElement` in both branches.
- **State → data attributes by convention, not by hand:** `state = { disabled, multiple, orientation }` flows into `useRenderElement`, which calls `getStateAttributesProps` (`packages/react/src/internals/useRenderElement.tsx:76-78`). That helper maps each truthy state key to `data-<key>` — booleans become empty-string attributes, strings become values (`packages/react/src/internals/getStateAttributesProps.ts:24-28`). No custom `stateAttributesMapping` is passed by this unit, so `data-disabled` / `data-orientation="horizontal|vertical"` / `data-multiple` are produced mechanically — exactly the strings declared in `packages/react/src/toggle-group/ToggleGroupDataAttributes.ts:1-13`. `data-orientation` carrying the orientation is also *why* behavior.md's Accessibility section observes that `aria-orientation` is never emitted: the ARIA `group` role has no orientation property, so orientation is surfaced as data only, and no aria attribute is hardcoded anywhere in the unit.
- `useRenderElement` additionally resolves `render` (function form is called with `(props, state)`, element form is cloned with merged props, `packages/react/src/internals/useRenderElement.tsx:164-196`) and merges the forwarded ref (`packages/react/src/internals/useRenderElement.tsx:99-103`), which is what the conformance-verified `render`/ref behavior in behavior.md's Public API section reduces to.
- The `ToggleGroupDataAttributes.ts` constants are a declared public surface for CSS/attribute selectors; the component source never imports them (see the last section).

## Dependencies on other Base UI internals

**packages/utils** (`@base-ui/utils/*`):

- `useControlled` (`packages/utils/src/useControlled.ts`) — controlled/uncontrolled value slot + dev warnings. `packages/react/src/toggle-group/ToggleGroup.tsx:4` and `packages/react/src/toggle-group/ToggleGroup.tsx:48-53`.
- `useStableCallback` (`packages/utils/src/useStableCallback.ts`) — stable `setGroupValue`. `packages/react/src/toggle-group/ToggleGroup.tsx:3` and `packages/react/src/toggle-group/ToggleGroup.tsx:55`.
- `EMPTY_ARRAY` from `@base-ui/utils/empty` — frozen default for `defaultValue`. `packages/react/src/toggle-group/ToggleGroup.tsx:5` and `packages/react/src/toggle-group/ToggleGroup.tsx:41`.

**packages/react/src/internals:**

- `useRenderElement` (`packages/react/src/internals/useRenderElement.tsx`) — element rendering, `render`/`className`/`style` resolution, ref merging, conditional `enabled` rendering, state→data-attributes. `packages/react/src/toggle-group/ToggleGroup.tsx:6` and `packages/react/src/toggle-group/ToggleGroup.tsx:99-104`; used again inside `CompositeRoot` for the standalone branch.
- `getStateAttributesProps` (`packages/react/src/internals/getStateAttributesProps.ts`) — the state-key → `data-*` convention, consumed via `useRenderElement`.
- `CompositeRoot` / `useCompositeRoot` / `CompositeList` / `CompositeRootContext` (`packages/react/src/internals/composite/`) — the standalone branch's roving tabindex + keyboard navigation engine (arrow keys, RTL mapping, Home/End via `enableHomeAndEndKeys`, loop via `loopFocus`, focus microtask commit). `packages/react/src/toggle-group/ToggleGroup.tsx:8` and `packages/react/src/toggle-group/ToggleGroup.tsx:111-121`. Pulls in `useDirection` for RTL internally.
- `createBaseUIChangeEventDetails` types + `REASONS` (`packages/react/src/internals/createBaseUIEventDetails.ts`, `packages/react/src/internals/reasons.ts`) — typing of the change-event contract. `packages/react/src/toggle-group/ToggleGroup.tsx:12-13`, `packages/react/src/toggle-group/ToggleGroup.tsx:55-60`, `packages/react/src/toggle-group/ToggleGroup.tsx:194-196`. Note this unit only ever exposes `REASONS.none` as its change reason (`packages/react/src/toggle-group/ToggleGroup.tsx:194`); the details *instance* is created by the Toggle child and flows upward through the context callback.
- Types: `BaseUIComponentProps`, `HTMLProps`, `Orientation` from `internals/types` (`packages/react/src/toggle-group/ToggleGroup.tsx:7`).

**Cross-component:**

- *Consumes:* `toolbar/root/ToolbarRootContext` (`packages/react/src/toggle-group/ToggleGroup.tsx:9` and `packages/react/src/toggle-group/ToggleGroup.tsx:38`; context at `packages/react/src/toolbar/root/ToolbarRootContext.ts`), `toolbar/group/ToolbarGroupContext` (`packages/react/src/toggle-group/ToggleGroup.tsx:10` and `packages/react/src/toggle-group/ToggleGroup.tsx:39`; context at `packages/react/src/toolbar/group/ToolbarGroupContext.ts`) — optional ancestors providing only `disabled`.
- *Is consumed by:* `toggle/Toggle.tsx` imports `useToggleGroupContext` from this unit (`packages/react/src/toggle/Toggle.tsx:8`). The context defined in `packages/react/src/toggle-group/ToggleGroupContext.ts` is the entire inter-unit contract — a port must preserve both its shape (`value` membership test + `setGroupValue(itemValue, nextPressed, details)` with the shared veto chain) and the fact that the *child* creates the event details.
- Also hosted by `Toolbar` in docs/demos; no other source consumer exists.

**Not used:** no `floating-ui-react` (no positioning), no `use-render`, no portal utilities, no `useIsoLayoutEffect`, no timers/animation-frame utilities, no `useButton`/`useCompositeItem` (those live on the Toggle side of the boundary).

`wraps-external:` — the unit's `TODO.md` entry has no `wraps-external:` field (`TODO.md:552-558`), and the source confirms it: nothing here delegates to an external npm package; all behavior is in-repo.

## Anything in source not explained by any test

Flagged for the golden-fixture stage and the backward-looking audit loop — none of these are covered by behavior.md's mined claims:

1. **`loopFocus={false}`** — the prop exists, defaults to `true`, and forwards to `CompositeRoot` (`packages/react/src/toggle-group/ToggleGroup.tsx:27`, `packages/react/src/toggle-group/ToggleGroup.tsx:118`, `packages/react/src/toggle-group/ToggleGroup.tsx:179-184`), but this unit's tests only exercise the default wrap-around (behavior.md, Keyboard interactions). Non-looping boundary behavior is untested.
2. **Toolbar-branch rendering** — behavior.md's Toolbar nesting bullet only smoke-tests selection and roving focus inside a toolbar. Nothing asserts the toolbar branch applies `render`/`className` or emits the state data attributes (`packages/react/src/toggle-group/ToggleGroup.tsx:99-104` is never attribute-asserted under a toolbar), nor that `loopFocus`/`enableHomeAndEndKeys` are inert there (structural fact: they're only `CompositeRoot` params, `packages/react/src/toggle-group/ToggleGroup.tsx:118-119`).
3. **Toolbar disabled merging** — `toolbarContext?.disabled || toolbarGroupContext?.disabled` (`packages/react/src/toggle-group/ToggleGroup.tsx:45-46`) is untested: behavior.md's disabled tests cover only the group's own `disabled` prop, never `Toolbar.Root disabled` or `Toolbar.Group disabled` wrapping the group.
4. **`useControlled` dev warnings** — the controlled↔uncontrolled flip warning and the `defaultValue`-mutation warning (`packages/utils/src/useControlled.ts:47-80`) are not asserted by this unit (its only warning test is the Toggle missing-`value` error).
5. **`ToggleGroupDataAttributes.ts` is dead code from the component's perspective** — nothing in the repo imports it (only `ralph/generated/components.json:1391` lists the file). The attribute strings it declares are verified only *indirectly*, through rendered-attribute assertions. A port that renames state keys would silently diverge from these constants.
6. **Value-less toggles write generated ids into the group value** — behavior.md's Edge cases documents that value-less toggles stay independently toggleable, but no test observes what `onValueChange` receives for them. Per the implementation, a `Toggle` without `value` uses its auto-generated id (`packages/react/src/toggle/Toggle.tsx:43-44`), so group-value arrays (and `onValueChange` payloads) can contain ephemeral ids like `:r1:` — unobserved anywhere.
7. **Duplicate values in a controlled array** — the multiple-mode removal does `splice(groupValue.indexOf(newValue), 1)` (`packages/react/src/toggle-group/ToggleGroup.tsx:66-67`), which removes only the first occurrence; a controlled `value={['a','a']}` would need two unpress events to clear it. Untested and undocumented.
8. **Arrow-key `stopPropagation`** — `CompositeRoot` defaults `stopEventPropagation = true` (`packages/react/src/internals/composite/root/CompositeRoot.tsx:33`), so navigation keydowns inside the group stop bubbling (`packages/react/src/internals/composite/root/useCompositeRoot.ts:302-305`). behavior.md marks event bubbling N/A, so this side effect is asserted nowhere; consumers attaching ancestor keydown handlers would notice it first.
9. **Shared `EMPTY_ARRAY` default** — the uncontrolled default is a single frozen singleton (`packages/react/src/toggle-group/ToggleGroup.tsx:5` and `packages/react/src/toggle-group/ToggleGroup.tsx:41`); correctness depends on the reducer's copy-before-mutate discipline (`slice()` at `packages/react/src/toggle-group/ToggleGroup.tsx:63`). There is no test that would catch an in-place mutation regression.
