# Alert Dialog implementation spec

Stage 2 mining for the `alert-dialog` unit. Companion to `specs/library/alert-dialog/behavior.md`
(behavior.md is the WHAT record; this file only explains HOW/WHY and cites source). The unit's
`TODO.md` entry (`TODO.md:369-374`) has no `wraps-external:` field — confirmed by reading the
entry — so no external-package delegation applies and no replacement crate is named in the TODO
fields. The delegation that does exist is entirely *internal*: alert-dialog is a thin facade over
Base UI's own `dialog` unit plus shared popup utilities.

## Facade shape (what alert-dialog itself contributes)

The unit's own code contains almost no logic. It contributes exactly three things:

1. **Mode override values.** `AlertDialogRoot` is a one-line call to the shared dialog root
   renderer with mode `'alert-dialog'` (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:14-16`).
   Its props interface strips the dialog props that must not be configurable here — `modal`,
   `disablePointerDismissal`, `handle`, `actionsRef`, `onOpenChange` are re-declared with
   alert-dialog-specific types (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:20-42`).
2. **A nominally-branded handle.** `AlertDialogHandle<Payload>` extends `DialogHandle<Payload>`
   adding only a type-level brand field with no runtime presence
   (`packages/react/src/alert-dialog/handle.ts:11-15`); the factory just constructs it
   (`packages/react/src/alert-dialog/handle.ts:20-22`). All state, attachment, and imperative
   behavior is inherited from `DialogHandle`.
3. **Type narrowing + re-exports.** `AlertDialogTrigger` is literally `DialogTrigger` re-exported
   under a narrowed interface whose `handle` prop only accepts `AlertDialogHandle`
   (`packages/react/src/alert-dialog/trigger/AlertDialogTrigger.tsx:16-31`). Seven of the nine
   parts are direct re-exports of dialog parts with no wrapper component:
   `packages/react/src/alert-dialog/index.parts.ts:1-10`. Public prop/state types for those parts
   are aliased from the dialog types in `packages/react/src/alert-dialog/index.ts:5-33`.

Everything below is therefore implemented in the delegation targets; call sites are cited there
because that is where the behavior-producing code lives.

## State machine / hooks used

The alert-dialog unit itself declares zero React hooks. The state machine is the shared dialog
one, entered through a single call site: `useRenderDialogRoot('alert-dialog', props)` at
`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:15`. The renderer accepts three modes —
`'dialog' | 'drawer' | 'alert-dialog'` — and the alert mode's only special treatment is computed
before the store is created: `modal` is forced to `true`, `disablePointerDismissal` is forced to
`true`, and `role` becomes `'alertdialog'` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:35-39`).
These three constants are the entire alert-dialog delta, and they are exactly the values the
behavior spec records as asserted store state (behavior.md, "State model" and "Accessibility").

Hook chain inside `useRenderDialogRoot` (all call sites
`packages/react/src/dialog/root/useRenderDialogRoot.tsx` unless noted):

- `usePopupRootStore` (`:49-63`) — creates the single `DialogStore<Payload>` owned by this Root
  instance (initial `open` from `defaultOpen`, `openProp`, `activeTriggerId` from
  `defaultTriggerId`, `triggerIdProp`, plus the mode-forced `rootState`). The comment block at
  `:45-48` records the ownership rules: the store is not tied to the handle (swapping a handle
  re-attaches rather than recreates state — the Fast-Refresh handle-replacement case in
  behavior.md, "Edge cases"), and the popup element is handed to Floating UI as the floating
  element via `treatPopupAsFloatingElement: true`, the second argument (`:63`), whose parameter is
  declared at `packages/react/src/utils/popups/popupStoreUtils.ts:70-88`.
- `store.useControlledProp('openProp', openProp)` and `useControlledProp('triggerIdProp', ...)`
  (`:65-66`) — live syncing of controlled props into the store; this is the mechanism behind the
  controlled-veto semantics in behavior.md, "State model" (store stays open until the consumer
  actually updates `open`).
- `store.useSyncedValues(rootState)` (`:68`) — keeps `modal`/`disablePointerDismissal`/`nested`/
  `role` current in the store.
- `store.useContextCallback('onOpenChange' | 'onOpenChangeComplete', ...)` (`:69-70`) — registers
  the consumer callbacks used by every open/close transition.
- `store.useState('open' | 'mounted' | 'payload')` (`:72-74`) — re-render subscriptions; `payload`
  feeds the children-as-function call at `:101` (behavior.md, "Public API surface").
- `usePopupRootSync(store, open)` (`:76`) — pushes the controlled `open` value back into the store
  (`packages/react/src/utils/popups/popupStoreUtils.ts:626`).
- `useImplicitActiveTrigger(store)` (`:77`) — maintains `activeTriggerId` from registered triggers
  so ARIA sync works without an explicit `triggerId` (`packages/react/src/utils/popups/popupStoreUtils.ts:416`).
- `useOpenStateTransitions(open, store)` (`:78`) — drives open→mounted→unmounted transition state
  and returns `forceUnmount` (`packages/react/src/utils/popups/popupStoreUtils.ts:554`), the
  function behind `actionsRef.current.unmount()` in behavior.md, "State model".
- `React.useImperativeHandle(actionsRef, ...)` (`:80-87`) — builds `AlertDialogRoot.Actions`:
  `unmount: forceUnmount` and `close: () => store.setOpen(false, createChangeEventDetails(REASONS.imperativeAction))`.
- Handle lifecycle (inherited, not re-implemented): `DialogHandle` extends `BasePopupHandle`
  starting attached to a null store (`packages/react/src/dialog/store/DialogHandle.ts:14-20`);
  `open(triggerId)` → `openByTrigger` (`:30-32`), `openWithPayload(payload)` → `set('payload')` +
  `setOpen(true, REASONS.imperativeAction)` with a dev-mode warning when no root is attached
  (`:41-56`), `close()` → `closePopup` (`:63-65`), and the `isOpen` getter reads the attached
  store's `open` (`:70-72`). The `preventUnmountOnClose()` flag itself is attached to change-event
  details by `attachPreventUnmountOnClose` (`packages/react/src/utils/popups/popupStoreUtils.ts:223`).

Trigger-side hook chain (the alert trigger is `DialogTrigger` verbatim; call sites
`packages/react/src/dialog/trigger/DialogTrigger.tsx`):

- `useDialogRootContext(true)` + `usePopupHandleStore(handle)` — store resolution: the handle's
  attached store wins over the context store, and a missing both throws
  (`:38-45`; `packages/react/src/utils/popups/usePopupHandleStore.ts:17`). This dual resolution is
  what lets the same component serve both in-Root and detached triggers (behavior.md, "State model").
- `useBaseUiId(idProp)` (`:47`) — trigger element id.
- `useTriggerDataForwarding(thisTriggerId, triggerElementRef, store, { payload })` (`:54-61`) —
  registers the trigger element + payload into the store and reports whether the popup is mounted
  by this trigger (`packages/react/src/utils/popups/popupStoreUtils.ts:318`).
- `useButton({ disabled, native })` (`:63-66`) — native/non-native button props.
- `useClick(floatingContext)` (`:68`) — Floating UI click interaction; the trigger element becomes
  the Floating UI reference.
- `useOpenMethodTriggerProps` (`:69-74`) — records the open method (touch/keyboard) into
  `store.openMethod`.
- `useRenderElement('button', componentProps, ...)` (`:83-101`) — final render, merging Floating UI
  reference props, store-provided `triggerProps`, and the static ARIA block
  (`aria-haspopup: 'dialog'`, `aria-expanded`, `aria-controls`) at `:90-97`.

## Context providers/consumers

One context: `DialogRootContext`, holding the reactive `DialogStore` itself (not a derived props
object), created at `packages/react/src/dialog/root/DialogRootContext.ts:5`. The root provides it
at `packages/react/src/dialog/root/useRenderDialogRoot.tsx:92`. Consumers:

- The seven re-exported dialog parts (Portal, Popup, Backdrop, Title, Description, Close, Viewport
  — `packages/react/src/alert-dialog/index.parts.ts:2-7`) read the store through
  `useDialogRootContext()`, whose required variant throws if the part is used outside a Root
  (`packages/react/src/dialog/root/DialogRootContext.ts:9-18`). Because the context value is the
  store, every part subscribes to exactly the state slices it needs (e.g. popup element, open,
  transition status) — this is how `data-open`, `aria-labelledby`/`aria-describedby` id linking,
  and portal mounting emerge without prop drilling.
- `AlertDialogTrigger` consumes the context in its *optional* form (`useDialogRootContext(true)`,
  `packages/react/src/dialog/trigger/DialogTrigger.tsx:38`) so it can render detached with only a
  `handle` prop.
- Nesting coordination crosses one more boundary: the root reads its own parent store via
  `useDialogRootContext(true)` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:41-42`),
  derives the `nested` flag into root state (`:43`), and passes the parent's callback context into
  `DialogInteractions` (`:94-100`), which notifies parents of open/close through
  `onNestedDialogOpen` (`packages/react/src/dialog/root/useDialogRoot.ts:88-112`).
- `PopupHandleAttachment` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:93`,
  component at `packages/react/src/utils/popups/popupStoreUtils.ts:108`) is the store↔handle
  bridge rendered as a child when the `handle` prop is present — it attaches the Root's store to
  the handle, making detached triggers and the imperative handle API work.

## DOM/portal strategy and why

- The Root renders no DOM element. Its output is: the context provider, an optional
  `PopupHandleAttachment`, an optional `DialogInteractions`, and children (render-prop or plain;
  `packages/react/src/dialog/root/useRenderDialogRoot.tsx:91-103`).
- Portal/DOM ownership belongs to `DialogPortal`/`DialogPopup` (re-exported dialog parts). The
  popup element is registered into the store and passed to Floating UI as the floating element
  (`treatPopupAsFloatingElement: true`, `packages/react/src/dialog/root/useRenderDialogRoot.tsx:63`;
  declared at `packages/react/src/utils/popups/popupStoreUtils.ts:70-88`; the design note is the
  comment at `:45-48`). Rationale: Floating UI is used for the *interaction* machinery (reference
  registration, `useClick`, `useDismiss`, floating root context) rather than for positioning —
  dialogs are centered/positioned with CSS, so the popup is a "floating element" only to reuse the
  dismissal/focus plumbing shared with positioned popups.
- The trigger renders a native `<button>` by default through `useRenderElement('button', ...)`,
  merging Floating UI reference props, store trigger props, and the static ARIA block
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:83-101`); `nativeButton={false}` reroutes
  through `useButton`'s non-button path (`:63-66`, `:98`).
- Mount gating: `DialogInteractions` is only rendered while `open || mounted`
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:89-100`). It is a renderless component
  (`packages/react/src/dialog/root/useDialogRoot.ts:126`) that carries the modality machinery:
  `useDismiss` with an outside-press policy that only honors presses on the dialog's own
  backdrop/internal backdrop for modal dialogs (`:29-84`), `useScrollLock(open && modal === true)`
  (`:86`), and nested-dialog counting. For alert-dialog, `disablePointerDismissal` is forced
  `true` (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:38`), so `outsidePress` returns
  `false` at `packages/react/src/dialog/root/useDialogRoot.ts:66` — the mechanism behind
  behavior.md's "Backdrop click does not close" rows.
- Trigger/popup open-state data attributes are produced by the shared mapping layer:
  `stateAttributesMapping: triggerOpenStateMapping` on the trigger
  (`packages/react/src/dialog/trigger/DialogTrigger.tsx:100`), which emits
  `data-popup-open` when its `open` state is `true`
  (`packages/react/src/utils/popupStateMapping.ts:9-11,30-37`); the unit's own data-attribute
  module just re-declares those names for docs/typing
  (`packages/react/src/alert-dialog/trigger/AlertDialogTriggerDataAttributes.ts:1-10`).

## Dependencies on other Base UI internals

No `wraps-external:` field exists on this unit's TODO entry (`TODO.md:369-374`), so per the
stage rules this is stated explicitly: **no external npm package is wrapped and no replacement
Rust crate is named in the TODO fields** — the delegation target is Base UI's own `dialog` unit,
which will itself be ported; alert-dialog should be treated as a configuration of it, not a
separate implementation.

Precise internal dependency list (what a per-component dependency graph needs):

- **`dialog/` unit (component-level dependency, the big one):**
  - `dialog/root/useRenderDialogRoot` — the entire root implementation
    (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:5,15`).
  - `dialog/root/DialogRootContext` / `useDialogRootContext` — the shared context
    (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:4` type import;
    consumed indirectly by the trigger and all re-exported parts).
  - `dialog/store/DialogHandle` — base class for `AlertDialogHandle`
    (`packages/react/src/alert-dialog/handle.ts:1`).
  - `dialog/trigger/DialogTrigger` — the trigger component itself, re-exported
    (`packages/react/src/alert-dialog/trigger/AlertDialogTrigger.tsx:3-7,16`).
  - `dialog/{backdrop,close,description,portal,popup,title,viewport}` — re-exported parts
    (`packages/react/src/alert-dialog/index.parts.ts:2-7`), plus their prop/state types aliased in
    `packages/react/src/alert-dialog/index.ts:5-33`. `DialogRoot.Actions`, `DialogRoot.Actions`,
    and the change-event reason/details types are likewise aliased
    (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:44-50`).
- **`utils/popups/` (shared popup machinery, via `useRenderDialogRoot` and `DialogTrigger`):**
  `usePopupRootStore`, `PopupHandleAttachment`, `useTriggerRegistration`,
  `useTriggerDataForwarding`, `useImplicitActiveTrigger`, `useOpenStateTransitions`,
  `usePopupInteractionProps`, `usePopupRootSync` (`packages/react/src/utils/popups/popupStoreUtils.ts:73,108,144,318,416,554,605,626`),
  `usePopupHandleStore` (`packages/react/src/utils/popups/usePopupHandleStore.ts:17`),
  `BasePopupHandle` (handle base class, `packages/react/src/dialog/store/DialogHandle.ts:4`),
  `createInitialPopupStoreState` (`packages/react/src/utils/popups/popupStoreUtils.ts` via
  `DialogStore`), and the data-attribute mappings from `packages/react/src/utils/popupStateMapping.ts:7,30-37`.
- **`floating-ui-react`:** `useClick` (`packages/react/src/dialog/trigger/DialogTrigger.tsx:13,68`),
  `useDismiss` (`packages/react/src/dialog/root/useDialogRoot.ts:5,29`), and the DOM-safe helpers
  `contains`/`getTarget` (`packages/react/src/dialog/root/useDialogRoot.ts:6`).
- **`internals/`:** `useButton` (`packages/react/src/dialog/trigger/DialogTrigger.tsx:5`),
  `useRenderElement` (`:6`), `useBaseUiId` (`:12`), `CLICK_TRIGGER_IDENTIFIER` (`:9`),
  `createBaseUIEventDetails` (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:3`;
  `packages/react/src/dialog/store/DialogHandle.ts:2`), `REASONS`
  (`packages/react/src/dialog/store/DialogHandle.ts:3`).
- **`@base-ui/utils/*`:** `useScrollLock` (`packages/react/src/dialog/root/useDialogRoot.ts:4`),
  `useIsoLayoutEffect` (`packages/react/src/dialog/root/useDialogRoot.ts:3`),
  `fastComponentRef` (`packages/react/src/dialog/trigger/DialogTrigger.tsx:3`).

Port-relevant summary: alert-dialog's own surface reduces to (a) three forced store values, (b) a
branded handle subclass, (c) prop-type narrowing/re-exports. A port needs the `dialog` unit plus
the `utils/popups` stack; nothing else in `alert-dialog/` will require independent reimplementation.

## Anything in source not explained by any test

Flagged gaps — behavior.md's mined suite does not exercise these source behaviors:

- **Handle brand enforcement is compile-time only.** The `__alertDialogBrand` field
  (`packages/react/src/alert-dialog/handle.ts:11-15`) has no runtime presence; the only artifact
  exercising it is the type-level spec `packages/react/src/alert-dialog/root/AlertDialogRoot.spec.tsx:32-42`
  (`@ts-expect-error` assertions that `Dialog.Handle`s are rejected). No runtime test, and the
  behavior suite never attempts a cross-type handle assignment.
- **`data-disabled` on the trigger** is declared in the unit's own attribute module
  (`packages/react/src/alert-dialog/trigger/AlertDialogTriggerDataAttributes.ts:6`) but no
  alert-dialog test or behavior.md section asserts it (behavior.md only records `data-popup-open`).
- **Nesting machinery is dead code as far as this unit's tests go.** `parentStore`/`nested` root
  state and the `onNestedDialogOpen` parent-notification path exist in the shared renderer
  (`packages/react/src/dialog/root/useRenderDialogRoot.tsx:41-43,94-100`;
  `packages/react/src/dialog/root/useDialogRoot.ts:88-112`), and behavior.md's "Edge cases"
  section explicitly records nested alert dialogs as untested (UNVERIFIED there).
- **The modality internals are unobserved.** Scroll lock, the outside-press backdrop-matching
  policy, the sloppy/intentional press-mode selection used for focus-trap `aria-hidden` cleanup,
  and topmost-dialog gating (`packages/react/src/dialog/root/useDialogRoot.ts:29-86`) have no
  assertions in this suite: behavior.md's "Modality" row only checks the backdrop's presence, its
  "Focus management" section records zero focus assertions, and forced `disablePointerDismissal`
  makes most of `outsidePress` unreachable in alert-dialog tests (only the Escape path runs).
- **`nativeButton` trigger prop surface** (non-native button rendering path,
  `packages/react/src/dialog/trigger/DialogTrigger.tsx:30,63-66,98`) is inherited via the type
  re-export but untested here.
- **Drawer-mode plumbing** (`isDrawer`, `packages/react/src/dialog/root/useRenderDialogRoot.tsx:35,98`)
  passes through the shared renderer used by alert-dialog but can never be exercised by this
  unit; it belongs to the dialog/drawer units' specs.
- Minor: `AlertDialogRootState` is deliberately empty
  (`packages/react/src/alert-dialog/root/AlertDialogRoot.tsx:18`) — the Root exposes no state
  attributes; noted so the fixtures stage doesn't go looking for root-level `data-*` output.
