# Dialog — Behavior Spec

Mined from tests only. Every non-trivial claim cites the test file/line that proves it.

## Public API surface (props, parts, subcomponents)

Parts: `Dialog.Root`, `Dialog.Trigger`, `Dialog.Portal`, `Dialog.Backdrop`, `Dialog.Popup`,
`Dialog.Viewport`, `Dialog.Title`, `Dialog.Description`, `Dialog.Close`.

- `Dialog.Root` props exercised by tests: `open`, `defaultOpen`, `onOpenChange`, `onOpenChangeComplete`,
  `modal` (`true` / `false` / `'trap-focus'`), `disablePointerDismissal`, `actionsRef`, `handle`,
  `defaultTriggerId`, `triggerId`, a render-prop children function receiving `{ payload }`
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:29-40`,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:864-923`,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:614-646`,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:1195-1234`,
  `packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:19-41`,
  `packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:126-157`).
  `Dialog.Root.Actions` exposes `unmount()` and `close()`
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1196-1233`).
- `Dialog.createHandle<TPayload>()` creates a detached handle object with `isOpen`, `open(triggerId)`,
  `openWithPayload(payload)`, `close()` methods, usable across multiple `Dialog.Trigger`/`Dialog.Root`
  pairs that are not nested in the same JSX subtree
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:19-41,186-228,1921-1953`).
- `Dialog.Trigger` props: `disabled`, `render`, `nativeButton`, `handle`, `id`, `payload` (value or
  function-returning-value) (`packages/react/src/dialog/trigger/DialogTrigger.test.tsx:34-83`,
  `packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1836-1858`). Requires a
  `Dialog.Root` ancestor OR a `handle` prop, else throws
  (`packages/react/src/dialog/trigger/DialogTrigger.test.tsx:22-32`).
- `Dialog.Portal` — has no props asserted beyond `keepMounted` and `container`; is required to wrap
  any portaled part (`Dialog.Viewport`, `Dialog.Popup`, `Dialog.Backdrop`) or a descriptive error is
  thrown (`packages/react/src/dialog/portal/DialogPortal.test.tsx:17-31`).
- `Dialog.Backdrop` props: `forceRender` (boolean) controls whether nested backdrops render when not
  the outermost open dialog (`packages/react/src/dialog/backdrop/DialogBackdrop.test.tsx:31-148`).
- `Dialog.Popup` props: `initialFocus` (ref | function | `false`), `finalFocus` (ref | function |
  `false`), `keepMounted` is actually a `Dialog.Portal` prop, but popup mount visibility depends on it
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:34-61`). Must be rendered within
  `<Dialog.Root>` (`packages/react/src/dialog/popup/DialogPopup.test.tsx:22-32`).
- `Dialog.Viewport` — wraps `Dialog.Popup`; renders only while the dialog is mounted, unless the
  containing `Dialog.Portal` has `keepMounted`
  (`packages/react/src/dialog/viewport/DialogViewport.test.tsx:24-65`).
- `Dialog.Close` props: `disabled`, `render`, `nativeButton`, `onClick` (can call
  `event.preventBaseUIHandler()` to stop the built-in close behavior)
  (`packages/react/src/dialog/close/DialogClose.test.tsx:24-135`).
- `Dialog.Title`, `Dialog.Description` — only conformance-tested (render as `<h2>`/`<p>` by default
  via `refInstanceof`) (`packages/react/src/dialog/title/DialogTitle.test.tsx:8-19`,
  `packages/react/src/dialog/description/DialogDescription.test.tsx:8-19`).

## State model (controlled/uncontrolled, defaults, transitions)

- Supports both controlled (`open`/`onOpenChange`) and uncontrolled (`defaultOpen`) usage; clicking
  trigger/close toggles open state in uncontrolled mode
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:389-409`).
- `onOpenChange` is called with `(open, eventDetails)`; `eventDetails.reason` is one of
  `REASONS.triggerPress`, `REASONS.closePress`, `REASONS.escapeKey`, `REASONS.outsidePress`,
  `REASONS.imperativeAction`
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:411-483`,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:431-457`).
- `eventDetails.cancel()` on open prevents the open state change while uncontrolled
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:535-553`) and on close prevents the internal
  `openchange` event from firing (`packages/react/src/dialog/root/DialogRoot.test.tsx:582-611`).
- `eventDetails.preventUnmountOnClose()` keeps the dialog DOM mounted after a close, and
  `actionsRef.current.unmount()` force-unmounts it later
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1195-1233`).
- `onOpenChangeComplete(open)` fires once transitions/animations finish: called with `true` after
  open completes and `false` after close completes, including waiting out CSS animations/transitions,
  and is not called for closes canceled mid-animation in a way that contradicts final state
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1318-1619`).
- Handle-backed (detached) root state: a `Dialog.Root` can attach to a `Dialog.createHandle()` handle;
  imperative `handle.open(id)`, `handle.openWithPayload(value)`, `handle.close()` work once a root is
  attached, and are no-ops (with a console warning "no root using this handle is mounted") before
  attach or after detach in dev; the warning is suppressed in production
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:186-247,249-323`).
- When a handle-backed root remounts, previously-stale payload/open state resets to defaults ("No
  payload", closed) rather than carrying over
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1389-1442,1518-1577`).
- Switching the `handle` prop on a live, still-mounted `Dialog.Root` re-attaches state without
  resetting it (the store belongs to the Root, not the handle)
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:490-547`).
- If two `Dialog.Root`s stay mounted while sharing one handle, a deferred check warns "more than one
  mounted root"; a transient overlap during a route-transition handoff (old root unmounts before the
  deferred check runs) does not warn, and the newer root wins control; if the newer, overlapping root
  unmounts first (a canceled transition), control reverts to the older, still-mounted root without a
  warning (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:690-933`).
- React StrictMode effect replay does not cause spurious "more than one mounted root" warnings
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:820-843`).
- Nested dialog/drawer counts: `Dialog.Popup` exposes a `--nested-dialogs` CSS custom property counting
  currently-open descendant dialogs (including cross-type nesting with `AlertDialog`), incrementing on
  open and decrementing on close or on unmount of an open nested dialog; unmounting an already-closed
  nested dialog does not change the count
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:745-938`).
- `data-nested` / `data-nested-dialog-open` style-hook attributes are applied to a `Dialog.Popup` that
  has a parent `Dialog`/`AlertDialog`, and to the parent respectively, reflecting nesting state
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:941-1000`).
- Multiple `Dialog.Trigger`s within one `Dialog.Root` (contained or detached) can each open the shared
  dialog; the active trigger gets `aria-expanded="true"`/`aria-controls`, inactive ones do not
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:936-1202,1204-1918`). The
  popup DOM node is reused across trigger switches rather than remounted
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1020-1046,1802-1833`).
- With a controlled `triggerId`, opening programmatically sets the associated trigger's ARIA state and
  the payload of that specific trigger (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1078-1155`).
- Trigger `payload` stays reactive: updating the value backing a mounted trigger's `payload` (without
  remounting `Dialog.Root`) updates the currently displayed payload
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1157-1201,1835-1889`).

## Keyboard interactions

- `Escape` closes the dialog (`onOpenChange` called with `reason: REASONS.escapeKey`)
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:459-470`), including from an inactive detached
  trigger that currently has focus (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1891-1917`).
  Only one internal `openchange` event fires per Escape close
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:555-580`).
  Escape inside a nested dialog opened from within a `Menu` does not close the parent menu
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1151-1192`).
- `Escape` then `Enter` reopens the dialog and re-invokes `initialFocus` with interaction type
  `'keyboard'` on keyboard-driven opens, `'touch'` on touch-driven opens
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:165-238`).
- `Tab` / `Shift+Tab` cycle focus within the popup when `modal="trap-focus"`, keeping focus off
  elements outside the popup, even across a `display: contents` wrapping ancestor
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:410-452`,
  `packages/react/src/dialog/root/DialogRoot.test.tsx:2058-2104`, Chromium-only).
- `Enter` on a focused trigger opens the dialog (used to test `initialFocus` interaction type)
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:219-226`).

## Focus management

- `initialFocus`: by default focuses the first focusable element inside the popup on open
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:64-90`). Accepts a ref
  (`:92-125`), a function returning an element (`:127-163`), a function receiving the interaction type
  (`'keyboard'`/`'touch'`) and returning an element or falsy for default
  (`:165-238`). On touch-opened dialogs, default focus goes to the popup element itself (not inner
  content), to avoid triggering the virtual keyboard (`:240-261`). `initialFocus={false}` disables
  moving focus at all — the trigger retains focus (`:263-285`). A function returning `true` or `null`
  defaults to normal initial-focus behavior (`:287-331`). The `initialFocus` function/ref accessor is
  NOT invoked again when the dialog closes (`:333-380`).
- `finalFocus`: by default returns focus to the trigger on close
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:455-481`). Accepts a ref (`:483-518`), a
  function returning an element (`:520-546`), `false` to disable moving focus (`:548-572`), a function
  returning `true` to force focus to the trigger (`:574-598`), and a function receiving the close type
  (`'keyboard'` vs. other) to choose between an element or trigger default (`:600-644`). If
  `initialFocus` pointed outside the popup and `finalFocus` is unset, final focus goes to the trigger,
  not wherever `initialFocus` pointed (`:682-716`); if both `initialFocus` and `finalFocus` point
  outside the popup, `finalFocus` wins on close (`:646-680`). A function returning `null` for
  `finalFocus` falls back to default behavior (`:718-742`).
- Chromium-only: focus trap cycling and initial focus continue to work correctly when the popup is
  wrapped in a `display: contents` ancestor element
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:383-452`).
- Chromium-only: focus stays trapped inside the popup even when its content includes a non-scrollable
  `ScrollArea` (`packages/react/src/dialog/root/DialogRoot.test.tsx:2058-2104`).
- Chromium-only: if a focused child element inside the popup is removed on `pointerdown`, focus moves
  to the popup itself, and a subsequent outside click still dismisses the dialog
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1236-1272`).
- Chromium-only: after a `NumberField` scrub-area pointer-lock interaction inside the dialog, the next
  outside click still dismisses the dialog (`packages/react/src/dialog/root/DialogRoot.test.tsx:1274-1315`).
- Chromium-only: focus returns to a menu trigger (not lost) when a detached dialog trigger housed
  inside a `Menu.Item` unmounts as part of opening the dialog and later Escape-closes it
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1999-2056`).
- Chromium-only: an `AlertDialog` confirmation opened from an outside-press cancel restores focus back
  into the original dialog's field once the confirmation closes
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:2106-2176`).
- Chromium-only: a non-modal dialog's field blur does not leak into an unrelated dialog's return-focus
  stack — a non-modal dialog does not feed the shared modal focus-return stack
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:2178-2243`).

## Accessibility (roles, aria-*, id linking)

- Popup has `role="dialog"` (`packages/react/src/dialog/root/DialogRoot.test.tsx:39`, verified via
  `popupConformanceTests` — see Shared harness section).
- `Dialog.Backdrop` has `role="presentation"`
  (`packages/react/src/dialog/backdrop/DialogBackdrop.test.tsx:21-29`).
- Trigger gets `aria-controls` pointing at the popup's `id`, and `aria-expanded` toggling
  `'false'`/`'true'` (verified via `popupConformanceTests`, and directly at
  `packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:531-534,590-592`).
- `Dialog.Title` sets `aria-labelledby` and `Dialog.Description` sets `aria-describedby` on the popup,
  linked by generated `id`s; these stay in sync when the title/description parts change or unmount
  (removing the corresponding `aria-*` attribute)
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:306-386`).
- `disabled` on `Dialog.Trigger`/`Dialog.Close`: native `<button>` gets the `disabled` attribute; a
  custom (non-native) `render` element instead gets `aria-disabled="true"` (no `disabled` attribute),
  and both get `data-disabled`
  (`packages/react/src/dialog/trigger/DialogTrigger.test.tsx:34-83`,
  `packages/react/src/dialog/close/DialogClose.test.tsx:24-86`). A disabled trigger is also excluded
  from Tab order (`packages/react/src/dialog/trigger/DialogTrigger.test.tsx:55-56,80-81`).

## DOM structure & portal behavior

- A part meant to be portaled (e.g. `Dialog.Viewport`) throws
  "Base UI: `<Dialog.Portal>` is missing." if not wrapped in `<Dialog.Portal>`
  (`packages/react/src/dialog/portal/DialogPortal.test.tsx:17-31`).
- `Dialog.Popup` throws "Base UI: DialogRootContext is missing. Dialog parts must be placed within
  <Dialog.Root>." if rendered outside `<Dialog.Root>`
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:22-32`).
- `Dialog.Trigger` throws "Base UI: <Dialog.Trigger> must be used within <Dialog.Root> or provided
  with a handle." without a root ancestor or a `handle` prop
  (`packages/react/src/dialog/trigger/DialogTrigger.test.tsx:22-32`).
- `Dialog.Portal` supports rendering into a custom `container`, including a Shadow DOM `shadowRoot`,
  and outside-press/backdrop-click dismissal works correctly for portals rendered into shadow roots
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:694-816`).
- `keepMounted` on `Dialog.Portal` keeps the popup in the DOM (marked inaccessible via
  `toBeInaccessible()`) even when `open=false`; without it (or `keepMounted={false}`/`undefined`), the
  dialog element is absent from the DOM when closed
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:34-61`).
  `Dialog.Viewport` similarly stays mounted while its enclosing `Dialog.Portal` has `keepMounted`, even
  after `open` becomes false (`packages/react/src/dialog/viewport/DialogViewport.test.tsx:49-65`); by
  default the viewport (and its popup) render only once the dialog opens
  (`packages/react/src/dialog/viewport/DialogViewport.test.tsx:24-47`).
- Internal `Dialog.Backdrop` (rendered automatically when `modal={true}`) sits as a sibling before the
  popup, after a focus guard element: `popup.previousElementSibling.previousElementSibling` has
  `role="presentation"` when `modal={true}`, and is absent (`null`) when `modal={false}`
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:864-922`).
- By default (`forceRender` unset/false), a nested `Dialog.Backdrop` does not render if there is an
  ancestor dialog whose own backdrop is rendered — only the outermost/root backdrop renders, at any
  nesting depth; `forceRender` overrides this and forces every level's backdrop to render
  (`packages/react/src/dialog/backdrop/DialogBackdrop.test.tsx:31-148`).
- React `Suspense` boundaries placed outside `Dialog.Portal` do not cause "Maximum update depth
  exceeded" errors when content inside the portal is lazy-loaded
  (`packages/react/src/dialog/portal/DialogPortal.test.tsx:33-71`).
- A dialog can render with `Dialog.Trigger` and `Dialog.Root` NOT sharing a JSX ancestor ("detached
  triggers") when both reference the same `Dialog.createHandle()` handle; the trigger can appear
  before or after the root in the tree, at any level of DOM nesting/wrapper depth, and remains
  clickable/functional across dynamic reparenting (adding/removing wrapper `<div>`s) and across the
  handle being replaced (Fast-Refresh-like recreation)
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:612-639,1204-1918`).

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange(open: boolean, eventDetails)` — `eventDetails.reason` identifies the trigger cause:
  `REASONS.triggerPress`, `REASONS.closePress`, `REASONS.escapeKey`, `REASONS.outsidePress`,
  `REASONS.imperativeAction` (`packages/react/src/dialog/root/DialogRoot.test.tsx:411-457,459-498`).
  `eventDetails.cancel()` can prevent the pending state change
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:535-611`).
  `eventDetails.preventUnmountOnClose()` and `eventDetails.trigger` (undefined when no trigger is
  associated) are also part of the payload
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:431-457,1195-1233`).
- `onOpenChangeComplete(open: boolean)` fires after the open/close transition (including CSS
  animpark/transition completion) finishes
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1318-1619`).
- An internal `openchange` event (observed via the floating-ui `floatingRootContext.context.events`)
  fires with `{ open, reason }`; exactly one fires per Escape-close, and none fires if the close is
  canceled (`packages/react/src/dialog/root/DialogRoot.test.tsx:555-611,2262-2281`).
- `Dialog.Close`'s `onClick` handler: if it calls `event.preventBaseUIHandler()`, the dialog does not
  close (`packages/react/src/dialog/close/DialogClose.test.tsx:118-135`). Passing
  `onClick={undefined}` still closes the dialog via the built-in handler
  (`packages/react/src/dialog/close/DialogClose.test.tsx:89-116`). Clicking `Dialog.Close` while the
  dialog is already closed (`open=false`, `keepMounted`) still invokes the user's own `onClick` but
  does not call `onOpenChange` again
  (`packages/react/src/dialog/close/DialogClose.test.tsx:137-155`).
- Outside-press dismissal distinguishes intentional press from incidental `mousedown`: for both a
  user-provided backdrop and the internal (`modal={true}`) backdrop, `mousedown` alone does not close
  the dialog, only the subsequent `click` does
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:648-692`).
- Right-button ("non-main") clicks/pointerdowns on the backdrop or outside the dialog do not trigger
  close (`packages/react/src/dialog/root/DialogRoot.test.tsx:518-532,1622-1646`, Chromium-only for the
  backdrop pointer variant).
  A native trusted click whose `pointerdown` opened the dialog is ignored for outside-press purposes —
  only the gesture that opened it is suppressed; a fresh press while open still dismisses
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:42-168`, Chromium-only).
- Touch-based outside dismissal (Chromium-only): a tap (`touchstart`/small `touchmove`/`touchend`)
  outside closes both non-modal and `trap-focus` dialogs with `reason: REASONS.outsidePress`
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1648-1867`). Multi-touch edge cases do not
  close the dialog: when another touch is still active on `touchend`, when two touches lift
  simultaneously, or when multiple touches move but none crosses the drag-dismissal threshold
  individually; a single touch crossing the ~10px drag threshold does close it immediately, before
  `touchend` (`:1714-1866`).
- `disablePointerDismissal` prop controls whether an outside mousedown+click closes the dialog: `true`
  → does not close; `false`/`undefined` → closes (default is to close)
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:614-646`).

## Edge cases (rapid interactions, unmount, nesting)

- Dismiss interactions (Escape, outside click) correctly "rewire" after a dialog is closed and
  reopened — i.e. they keep working on subsequent open/close cycles, for contained, detached, and
  multiple-detached-trigger configurations
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:275-304`).
- Nested non-modal dialogs: opening a new modal dialog does not dismiss a previous modal dialog
  underneath it (`packages/react/src/dialog/root/DialogRoot.test.tsx:925-966`); multiple non-nested
  (sibling, not JSX-nested) dialogs opened in a chain dismiss one at a time via their own backdrops,
  from outermost/last-opened to first
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:968-1032`).
- Chromium-only nested-popup interop: dismissing outside a nested modal `Menu`, `Select`, or the
  parent dialog's own backdrop closes only the intended layer, not the enclosing dialog
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1034-1149`).
- Unmounting an open nested dialog decrements `--nested-dialogs`; unmounting an already-closed nested
  dialog does not change the count
  (`packages/react/src/dialog/popup/DialogPopup.test.tsx:808-900`).
- A handle-backed root that unmounts while open and later remounts starts fresh (closed, no payload,
  triggers show `aria-expanded="false"`, no stale `aria-controls`), and persistent (still-mounted)
  detached triggers are notified of that reset
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1323-1517`).
- A layout effect that opens/closes a handle-backed root's dialog in the very same commit the root
  first attaches (before detached triggers have finished re-registering) still correctly resolves and
  associates the intended trigger, without a spurious "no root using this handle is mounted" or "No
  trigger found" warning
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:99-228,641-688,871-933`).
- An abandoned/canceled `React.useTransition` render that would have replaced a live handle-backed
  root does not attach a replacement store nor affect the currently open dialog
  (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:325-410`).
- Remounting a handle-backed root from controlled to uncontrolled (or vice versa) after an unmount
  does not emit a React "controlled state of openProp"-style warning about switching between
  controlled/uncontrolled (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:1579-1644`).
- After the root switches from handle A to handle B, a trigger still wired to handle A can no longer be
  resolved/associated by handle B's root (its `aria-expanded` stays `false`), and a "No trigger found"
  warning is logged (`packages/react/src/dialog/root/DialogRoot.detached-triggers.test.tsx:2028-2086`).
- `Dialog.Root`'s nested-dialog/drawer count reporting to a parent's `onNestedDialogOpen` callback is
  guaranteed to occur before React passive effects flush, both on mount and on teardown
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:170-237`).
- External (third-party) scroll lockers: if a third-party scroll-lock mechanism (attribute+stylesheet,
  `<body>` overflow shorthand, or `<html>` overflow longhands) is still applying its own lock when the
  dialog opens, Base UI's own scroll lock keeps the page locked continuously across the handoff and
  unlocks correctly on close (Chromium-only)
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1869-1997`). If an external `<body>`-only lock
  cannot actually affect the page (because `<html>` owns the scroll container), Base UI locks
  immediately rather than deferring
  (`packages/react/src/dialog/root/DialogRoot.test.tsx:1969-1996`).
- `data-testid="trigger"`/`"popup"` conventions used throughout are test scaffolding, not the
  component's own API — noted only for clarity when reading citations above.

## Shared harness dependencies

`DialogRoot.test.tsx` imports `popupConformanceTests`, `createRenderer`, `isJSDOM`, and `wait` from
`#test-utils`, which resolves to `packages/react/test/index.ts`. The relevant shared harness files:

- `packages/react/test/popupConformanceTests.tsx` — a reusable suite (`popupConformanceTests(config)`)
  asserting generic "popup" conformance shared by all trigger+popup style components (Dialog, Menu,
  Select, etc.): controlled `open` mounts/unmounts the popup; uncontrolled click-to-open (when
  `triggerMouseAction: 'click'`); the popup has `role={expectedPopupRole}`; the trigger has
  `aria-controls` pointing at the popup id, `aria-expanded` toggling on click, `aria-haspopup` set to
  `expectedAriaHasPopupValue`; a custom popup `id` is respected; and animation-related removal
  behavior (Chromium-only, one test explicitly `skip()`-ed as "revisit after feedback from the team").
- `packages/react/test/createRenderer.ts` — wraps the shared MUI `createRenderer` so that `render`,
  `rerender`, and the new `setProps` helper all execute inside `act()`. Not further customized for
  Dialog; used generically by all Dialog test files (`createRenderer().render`).

### Actual `PopupTestConfig` passed by Dialog (`packages/react/src/dialog/root/DialogRoot.test.tsx:28-40`)

```js
popupConformanceTests({
  createComponent: (props) => (
    <Dialog.Root {...props.root}>
      <Dialog.Trigger {...props.trigger}>Open dialog</Dialog.Trigger>
      <Dialog.Portal {...props.portal}>
        <Dialog.Popup {...props.popup}>Dialog</Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  ),
  render,
  triggerMouseAction: 'click',
  expectedPopupRole: 'dialog',
});
```

This leaves `expectedAriaHasPopupValue` defaulted to `expectedPopupRole` ('dialog'),
`alwaysMounted` defaulted to `false`, and `combobox` defaulted to `false`
(`packages/react/test/popupConformanceTests.tsx:12-15`), so the generic conformance suite additionally
proves for Dialog specifically: popup unmounted (`null`) rather than merely inaccessible when closed
(`open: false`/uncontrolled default), `aria-haspopup="dialog"` on the trigger, and non-combobox
`data-open` attribute checks on ARIA-expanded assertions.
