# Button implementation spec

Companion to `specs/library/button/behavior.md` (WHAT); this file is the WHY/HOW of the React
implementation in `packages/react/src/button/`. Source files mined: `Button.tsx`,
`ButtonDataAttributes.tsx`, `index.ts`, `Button.spec.tsx` (plus the internals they call into,
cited where load-bearing). No `wraps-external:` field exists in the button `TODO.md` entry
(`TODO.md:325-331`), so there is no external-package delegation to account for — everything below
derives from Base UI's own source.

## State machine / hooks used

`Button` itself contains **no state machine and no `useState`**. The entire runtime behavior is
two hook calls plus a state descriptor:

1. **`useButton`** — the behavior engine, called at `packages/react/src/button/Button.tsx:27-31`
   with `{ disabled, focusableWhenDisabled, native: nativeButton }`. Defined at
   `packages/react/src/internals/use-button/useButton.ts:14-243`. Internally it composes:
   - `React.useRef` holding the DOM element (`packages/react/src/internals/use-button/useButton.ts:23`).
   - `useFocusableWhenDisabled` (`packages/react/src/internals/use-button/useButton.ts:28-34` →
     `packages/react/src/utils/useFocusableWhenDisabled.ts:4-61`) — a pure prop-derivation hook
     (memoized, no effects). It decides, per the native/non-native × disabled ×
     focusableWhenDisabled matrix, whether to emit `disabled` attribute
     (`packages/react/src/utils/useFocusableWhenDisabled.ts:45-47`), `aria-disabled`
     (`packages/react/src/utils/useFocusableWhenDisabled.ts:38-43`), and which `tabIndex`
     (`packages/react/src/utils/useFocusableWhenDisabled.ts:30-36` — `-1` only for non-native
     disabled buttons without `focusableWhenDisabled`). It also injects an `onKeyDown` that
     `preventDefault()`s every non-Tab key on a focusable-when-disabled button
     (`packages/react/src/utils/useFocusableWhenDisabled.ts:23-28`) — that is the mechanism behind
     "keyboard activation fully suppressed" while the button stays Tab-focusable (behavior.md,
     "Keyboard interactions"). The subtle reason `aria-disabled` is emitted even for a *native*
     button with `focusableWhenDisabled` (`packages/react/src/utils/useFocusableWhenDisabled.ts:38-43`)
     is that the native `disabled` attribute must be *omitted* there (it would block focus), so
     AT still needs the disabled signal from ARIA.
   - A dev-only `React.useEffect` guarding `nativeButton`/element-tag mismatches with
     `error(...)` + `SafeReact.captureOwnerStack`
     (`packages/react/src/internals/use-button/useButton.ts:36-66`) — DX assertion, no production
     effect.
   - `updateDisabled` + `useIsoLayoutEffect`
     (`packages/react/src/internals/use-button/useButton.ts:68-89`) — an imperative DOM fix-up for
     the nested case `<Toolbar.Button disabled render={<Menu.Trigger />}>`: when two `useButton`
     layers stack, the inner layer's `element.disabled` write would leak the native `disabled`
     attribute onto a focusable-when-disabled composite item, so the effect force-clears it. The
     same callback is re-run from the ref callback
     (`packages/react/src/internals/use-button/useButton.ts:234-237`) so a re-mounted element is
     corrected without an effect pass.
   - `getButtonProps`, a memoized prop-resolver
     (`packages/react/src/internals/use-button/useButton.ts:91-232`). This is where the
     event semantics of behavior.md live: `onClick`/`onMouseDown`/`onPointerDown` are gated on
     `disabled` (preventDefault + swallow the external handler,
     `packages/react/src/internals/use-button/useButton.ts:104-115,218-224`); `onKeyDown`/`onKeyUp`
     are `makeEventPreventable`-wrapped so the caller's handler runs first and can veto Base UI's
     own activation via `event.preventBaseUIHandler()`
     (`packages/react/src/internals/use-button/useButton.ts:121-125,184-200`). The Enter/Space
     split mirrors native `<button>` semantics: **Enter dispatches the synthetic click on
     keydown** (`packages/react/src/internals/use-button/useButton.ts:172-175`), **Space on
     keyup** (`packages/react/src/internals/use-button/useButton.ts:207-216`), with a
     keydown-defaultPrevented check cancelling activation like a native button would
     (`packages/react/src/internals/use-button/useButton.ts:165-168`), and Space prevented on link
     roots to kill page scroll (`packages/react/src/internals/use-button/useButton.ts:159-161`).
     The `shouldClick` guard restricts synthetic dispatch to the current target and skips real
     `<button>`s / links where the browser already handles it
     (`packages/react/src/internals/use-button/useButton.ts:127-131,154-163`).
   - The closing `mergeProps` line sets the semantic fork in one expression:
     `isNativeButton ? { type: 'button' } : { role: 'button' }`
     (`packages/react/src/internals/use-button/useButton.ts:226`), then layers
     `focusableWhenDisabledProps`, then the caller's other props last so user values win
     (`packages/react/src/internals/use-button/useButton.ts:226-229`).
   - `buttonRef` is a `useStableCallback` ref that stores the element and re-runs `updateDisabled`
     (`packages/react/src/internals/use-button/useButton.ts:234-237`).
2. **`useRenderElement`** — the render engine, called at `packages/react/src/button/Button.tsx:37-41`
   as `useRenderElement('button', componentProps, { state, ref: [forwardedRef, buttonRef], props:
   [elementProps, getButtonProps] })`. Defined at
   `packages/react/src/internals/useRenderElement.tsx:22-48`. It:
   - resolves `className`/`style` against the state object and generates `data-*` state
     attributes (`packages/react/src/internals/useRenderElement.tsx:73-78`);
   - merges the forwarded ref, the internal `buttonRef`, and any ref on the `render` element via
     `useMergedRefs`/`useMergedRefsN` plus `getReactElementRef`
     (`packages/react/src/internals/useRenderElement.tsx:94-104`) — this is why refs attach on
     both default and render-prop roots (behavior.md, "Shared harness dependencies");
   - evaluates the `render` prop: functions are invoked with `(props, state)`
     (`packages/react/src/internals/useRenderElement.tsx:164-170`), elements are cloned with
     `mergeProps(props, render.props)` so the render element's own handlers compose (theirs first)
     rather than being clobbered (`packages/react/src/internals/useRenderElement.tsx:172,196`);
   - special-cases the string tag `'button'` to default `type="button"` before spreading props
     (`packages/react/src/internals/useRenderElement.tsx:232-240`) — a second, independent guard
     against the form-submit default, complementing `useButton`'s `{ type: 'button' }`.

The only "state" object is the plain `{ disabled }` literal at
`packages/react/src/button/Button.tsx:33-35` (`ButtonState`,
`packages/react/src/button/Button.tsx:44-49`). It exists solely to drive `data-disabled`: the
generic state→attribute mapper turns `disabled: true` into `data-disabled=""`
(`packages/react/src/internals/getStateAttributesProps.ts:24-28`). This confirms behavior.md's
UNVERIFIED note in "State model": there is genuinely no internal state, controlled/uncontrolled
machinery, or transition logic anywhere in the unit — `useControlled` is not used.

No portal, no `floating-ui-react`, no `use-render` involvement.

## Context providers/consumers

`Button` **provides no context**. It **consumes** exactly one, optionally:

- `CompositeRootContext` (`packages/react/src/internals/composite/root/CompositeRootContext.ts:17-19`),
  read non-throwingly via `useCompositeRootContext(true)`
  (`packages/react/src/internals/use-button/useButton.ts:25`). Presence alone flips
  `isCompositeItem` (`packages/react/src/internals/use-button/useButton.ts:26`), which changes:
  Space activation to fire on **keydown** instead of keyup
  (`packages/react/src/internals/use-button/useButton.ts:138-152`), suppression of the duplicate
  native Space keyup click (`packages/react/src/internals/use-button/useButton.ts:187-196`), and
  the nested-disabled DOM fix-up described above. None of this is reachable through `Button`'s own
  props (the `composite` parameter of `useButton` is never passed by `Button.tsx`); it is purely
  ambient — `Button` becomes a composite item by being rendered inside `Composite.Root`.

Nothing crosses from `Button` downward: it is a leaf component that only forwards merged props and
refs to whatever element `render` produces.

## DOM/portal strategy and why

- **In-place rendering, no portal.** `useRenderElement('button', …)` clones/creates a single root
  element; nothing wraps it, so a render-prop `<a>`/`<span>` keeps its tag (behavior.md, "DOM
  structure & portal behavior"). A portal would be meaningless for a leaf trigger with no
  popup layer.
- **Two-layer element identity.** The default tag is fixed at `'button'`
  (`packages/react/src/button/Button.tsx:37`); `render` swaps identity inside
  `evaluateRenderProp` while keeping the computed props and refs
  (`packages/react/src/internals/useRenderElement.tsx:158-206`). Element props always lose to
  explicit user props in conflicts because `otherExternalProps` is merged last
  (`packages/react/src/internals/use-button/useButton.ts:228`) — which is how
  `<Button type="submit">` can override the internal `type: 'button'`
  (`packages/react/src/button/Button.spec.tsx:4`).
- **Imperative DOM surgery as an escape hatch**: `updateDisabled` writes `element.disabled = false`
  directly (`packages/react/src/internals/use-button/useButton.ts:79-87`) because the "disabled
  composite button rendering another button" case cannot be expressed through React props alone
  (two layers both writing `disabled`).

## Dependencies on other Base UI internals

Everything `Button`'s folder imports, in dependency order (this is the precise per-component
dependency list the coarse `blocked-by: [Phase A complete]` default should be replaced with):

- **`internals/use-button/useButton`** — behavior engine
  (`packages/react/src/button/Button.tsx:3`).
- **`internals/useRenderElement`** — render/merge engine
  (`packages/react/src/button/Button.tsx:4`), which itself pulls
  `@base-ui/utils/useMergedRefs`, `@base-ui/utils/getReactElementRef`,
  `@base-ui/utils/mergeObjects`, `@base-ui/utils/warn`, `@base-ui/utils/empty`, plus sibling
  internals `internals/getStateAttributesProps`, `utils/resolveClassName`, `utils/resolveStyle`,
  `merge-props` (`packages/react/src/internals/useRenderElement.tsx:2-11`).
- **`internals/types`** — `BaseUIComponentProps<'button', ButtonState>` and `NativeButtonProps`
  type contract (`packages/react/src/button/Button.tsx:5,51-52`; definitions at
  `packages/react/src/internals/types.ts:36-61,63-71`).
- **`merge-props`** — `mergeProps`/`mergePropsN` (props-getter composition and handler-chaining
  with the `preventBaseUIHandler` veto, `packages/react/src/merge-props/mergeProps.ts:42-115,210-250`)
  and `makeEventPreventable` (`packages/react/src/merge-props/mergeProps.ts:268-274`), used by
  `useButton` (`packages/react/src/internals/use-button/useButton.ts:8`).
- **`utils/useFocusableWhenDisabled`** — disabled/focusability prop derivation
  (`packages/react/src/internals/use-button/useButton.ts:11`).
- **`utils/dispatchClickWithModifiers`** — the synthetic click for keyboard activation; dispatches
  an untrusted `click` via `ownerWindow(target).PointerEvent` carrying the source event's modifier
  keys, with `detail: 0` (keyboard convention) so it still triggers native activation like link
  navigation (`packages/react/src/utils/dispatchClickWithModifiers.ts:19-36`; imported at
  `packages/react/src/internals/use-button/useButton.ts:12`). This is the direct mechanism behind
  behavior.md's "preserves modifier keys" and "link navigates" assertions.
- **`internals/composite/root/CompositeRootContext`** — optional ambient consumer
  (`packages/react/src/internals/use-button/useButton.ts:9`).
- **`@base-ui/utils/*`** — `useStableCallback`, `useIsoLayoutEffect`, `error`, `safeReact`
  (`packages/react/src/internals/use-button/useButton.ts:4-7`), `owner`
  (`packages/react/src/utils/dispatchClickWithModifiers.ts:1`).
- **`@floating-ui/utils/dom`** — only the `isHTMLElement` type-guard
  (`packages/react/src/internals/use-button/useButton.ts:3`); no positioning/floating behavior.
- **Public surface**: `packages/react/src/button/index.ts:1-3` exports the component and types;
  `packages/react/src/button/ButtonDataAttributes.tsx:1-4` documents the `data-disabled`
  attribute as a standalone constant (see the unexplained-source note below).

## Anything in source not explained by any test

Flagged explicitly for the golden-fixture and audit stages; none of these are covered by
`Button.test.tsx` or the conformance suite that behavior.md mined:

1. **Composite integration.** The entire `isCompositeItem` branch set — Space-activates-on-keydown
   (`packages/react/src/internals/use-button/useButton.ts:138-152`), duplicate-click suppression on
   native composite Space keyup
   (`packages/react/src/internals/use-button/useButton.ts:187-196`), the text-navigation-role
   exemptions (`menuitem`/`option`/`gridcell`,
   `packages/react/src/internals/use-button/useButton.ts:134-141`), and the nested-disabled DOM
   fix-up (`packages/react/src/internals/use-button/useButton.ts:68-89`) — is unreachable from
   `Button`'s own tests, which never mount inside `Composite.Root`. It *is* exercised by the
   internals unit's own tests (`packages/react/src/internals/use-button/useButton.test.tsx:135-165,310-462`),
   so the Rust port must treat composite behavior as a `useButton`-level concern, not a `Button`
   fixture.
2. **Dev-only diagnostics with no test coverage in this unit**: the `nativeButton`↔tag mismatch
   warnings (`packages/react/src/internals/use-button/useButton.ts:36-66`) and
   `useRenderElement`'s uppercase-named-render-fn warning plus the invalid-`render`-prop throw
   (`packages/react/src/internals/useRenderElement.tsx:166-168,182-194,208-230`). These are
   production-invisible (all `NODE_ENV`-gated) but are part of the public DX contract.
3. **`ButtonDataAttributes.tsx` is dead code as a mechanism.** The constant
   (`packages/react/src/button/ButtonDataAttributes.tsx:1-4`) is imported nowhere in the repo
   (only listed in the generated inventory `ralph/generated/components.json:107`). The actual
   `data-disabled` output is produced generically from the state key name
   (`packages/react/src/internals/getStateAttributesProps.ts:24-28`), so the documented attribute
   contract and the implementation are coupled only by naming convention — a rename of either side
   would silently diverge. The Rust port should treat `data-*` state attributes as derivable from
   the state object, not as per-component literals.
4. **`onKeyUp` as a public handler.** `getButtonProps` destructures, wraps, and merges
   `onKeyUp` with veto semantics (`packages/react/src/internals/use-button/useButton.ts:96,177-217`),
   but behavior.md's Events section records no `onKeyUp` test. The Space-activation-on-keyup
   machinery depends on this handler, yet its external-composition behavior is untested at the
   `Button` level.
5. **`Button.spec.tsx` is a type-check fixture, not executable.** It contains bare JSX
   expressions with no exports (`packages/react/src/button/Button.spec.tsx:1-10`); it is excluded
   from the declaration build (`packages/react/tsconfig.build.json:18`) and included only by the
   test typecheck project (`packages/react/tsconfig.test.json:16`). Its only assertions are
   compile-time prop typings — notably `nativeButton={false}` combined with form props
   (`packages/react/src/button/Button.spec.tsx:10`).
6. **Reverse gap — a test not explained by source**: behavior.md's "becoming disabled while
   focused retains focus" (its Edge cases section) has *no corresponding implementation code* —
   no refocus/blur logic exists anywhere in `useButton` or `Button`. Retention is incidental
   host/browser behavior, so a Chromium-vs-jsdom divergence is plausible and the fixture stage
   should not assume an engineered guarantee.
