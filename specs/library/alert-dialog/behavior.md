# Alert Dialog behavior spec

Mined from the single alert-dialog test file. The unit's `TODO.md` entry (`TODO.md:304-310`) has
no `wraps-external:` field, so the behavior below is derived entirely from the component's own
tests; there is no third-party package to delegate to. The unit is not on the
`needs-batched-mining: true` list, so this single file covers the whole unit.

Files mined:
- `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx`

## Public API surface (props, parts, subcomponents)

- All parts are imported from the `@base-ui/react/alert-dialog` namespace.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:5`
- Parts exercised by the tests: `AlertDialog.Root`, `AlertDialog.Trigger`,
  `AlertDialog.Portal`, `AlertDialog.Popup`, `AlertDialog.Backdrop`, `AlertDialog.Title`,
  `AlertDialog.Description`, `AlertDialog.Close`, `AlertDialog.Viewport`.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:17-30`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:37-40`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:125`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:144`
- `AlertDialog.createHandle()` creates a handle object used to wire detached triggers to a root,
  and exposes `isOpen` plus imperative `open`/`openWithPayload`/`close` methods (see State model
  and Edge cases). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:79`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:932`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:949-957`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1030`
- `AlertDialog.Root` props exercised:
  - `open` (controlled). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:34`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1072`
  - `defaultOpen` (uncontrolled initial). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:82`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:200`
  - `onOpenChange(open, details)`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:140`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:246`
  - `onOpenChangeComplete(open)`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1072`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1155`
  - `handle` (binds the root to a `createHandle()` handle for detached triggers).
    `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:105`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:608`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:874`
  - `triggerId` / `defaultTriggerId` (select which trigger is the "active" trigger for ARIA
    sync when the dialog opens outside a trigger press). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:59`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:82`
  - `actionsRef` (ref to `AlertDialog.Root.Actions` with `unmount()` and `close()` methods).
    `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:282`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:367`
- `AlertDialog.Trigger` props exercised: `handle` (detached mode), `id`, `payload`.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:102`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:60`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:441`
- `AlertDialog.Root` accepts a children-as-function render prop receiving `{ payload }`, which
  re-renders with the active trigger's payload. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:439`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:807`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:976`
- `AlertDialog.Trigger` accepts a `payload` prop of any type, surfaced through that render prop.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:441-442`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:799-802`
- Popup conformance is registered with `expectedPopupRole: 'alertdialog'`,
  `expectedAriaHasPopupValue: 'dialog'`, and `triggerMouseAction: 'click'`.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:27-29`

## State model (controlled/uncontrolled, defaults, transitions)

- Open state is controlled via `open` or uncontrolled via `defaultOpen`; with neither, the dialog
  starts closed and the popup is absent from the DOM.
  `packages/react/test/popupConformanceTests.tsx:35-45`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:410`
- Uncontrolled transition: clicking any trigger opens the popup; clicking `AlertDialog.Close`
  closes it. `packages/react/test/popupConformanceTests.tsx:50-64`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:412-428`
- `onOpenChange` receives the new open state: `true` on open, `false` on close, once per
  transition (0 calls before any interaction). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:150-163`
- Backdrop click does not close the dialog and does not call `onOpenChange` — pointer dismissal
  is disabled for alert dialogs (the root store exposes `disablePointerDismissal: true`).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:230-233`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:883-887`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:895-899`
- Escape closes with reason `REASONS.escapeKey`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:210-213`
- Controlled veto: when `onOpenChange` ignores a close request (never sets `open=false`), the
  popup stays in the DOM with `data-open`, the trigger keeps `data-popup-open`, and
  `handle.isOpen` stays `true`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:236-277`
- `details.preventUnmountOnClose()` in `onOpenChange` keeps the popup mounted after a close
  request; the popup is only removed when the consumer calls `actionsRef.current.unmount()`.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:287-291`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:303-311`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:313-317`
- `actionsRef.current.unmount()` clears the manual-unmount state after use: a subsequent
  open/close cycle unmounts the popup normally on close.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:320-364`
- `actionsRef.current.close()` closes the dialog imperatively.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:379-383`
- Handle state: `handle.isOpen` tracks open state across trigger press, close, and vetoed closes.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:118`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:270`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:893`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:899`
- Handle imperative API: `handle.open(triggerId)` opens the dialog as if that trigger activated
  it (payload of that trigger is rendered, that trigger's ARIA is synced);
  `handle.openWithPayload(payload)` opens with a programmatic payload and no active trigger;
  `handle.close()` closes. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:949-962`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:989-1003`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1030-1042`
- The root store surfaces state to descendants via `useDialogRootContext()`; tests assert
  `modal: true`, `disablePointerDismissal: true`, and `role: 'alertdialog'` for alert dialogs.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:875`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:883-888`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1230-1244`
- Multiple triggers within one Root (or detached via a handle): any trigger opens the dialog;
  each open/close cycle works with every trigger. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:412-433`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:625-646`

## Keyboard interactions

- Escape while the dialog is open requests a close; `onOpenChange` is called once with reason
  `REASONS.escapeKey`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:210-213`
- No other keyboard behavior is asserted: Tab order, Enter/Space trigger activation, and focus
  trapping keys have no tests in this suite. UNVERIFIED — inferred from
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:196-214`, no test asserts any key
  other than Escape.

## Focus management

- No test in this suite asserts any focus behavior: initial focus placement, focus trapping,
  focus return to the trigger, or focus movement on open/close are all uncovered.
  UNVERIFIED — inferred, no test asserts focus movement anywhere in
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1-1244`.
- The popup conformance suite used by this unit does not register focus assertions either (its
  suites cover controlled/uncontrolled open, ARIA attributes, and animations).
  `packages/react/test/popupConformanceTests.tsx:33-213`

## Accessibility (roles, aria-*, id linking)

- The popup has `role="alertdialog"`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:28`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:46-47`, `packages/react/test/popupConformanceTests.tsx:71-76`
- The popup's `aria-labelledby` is set to the `AlertDialog.Title` element's id, and
  `aria-describedby` to the `AlertDialog.Description` element's id.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:49-54`
- The trigger exposes open state via `aria-expanded` (`'true'`/`'false'`).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:72-75`, `packages/react/test/popupConformanceTests.tsx:87-105`
- The active trigger's `aria-controls` equals the popup's id; with multiple triggers only the
  active trigger gets `aria-expanded="true"` and `aria-controls`, others stay `aria-expanded="false"`
  with no `aria-controls`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:72-75`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:514-526`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:995-996`, `packages/react/test/popupConformanceTests.tsx:80-85`
- The trigger has `aria-haspopup="dialog"`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:29`, `packages/react/test/popupConformanceTests.tsx:107-113`
- A custom popup `id` is honored and `aria-controls` follows the custom id.
  `packages/react/test/popupConformanceTests.tsx:115-120`
- `triggerId`/`defaultTriggerId` on the root select which trigger receives the open-state ARIA
  when the dialog opens without a trigger press (e.g. initially open, or opened via handle).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:57-76`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:82-95`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:97-119`
- Detached triggers (`Trigger handle={...}` outside the Root) sync their ARIA the same way, and
  the handle's `isOpen` mirrors the open state.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:97-119`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:687-699`
- The backdrop renders a `role="presentation"` element hidden from the accessibility tree (tests
  query it with `{ hidden: true }`). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:230`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:923`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1059`

## DOM structure & portal behavior

- The popup is rendered through `AlertDialog.Portal`; when closed (and not kept mounted), the
  popup is removed from the DOM entirely rather than merely hidden.
  `packages/react/test/popupConformanceTests.tsx:35-45`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:410`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:417-419`
- `AlertDialog.Viewport` renders a wrapper that contains the popup element.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:121-133`
- Open-state data attributes: the popup gets `data-open` while open, and the trigger gets
  `data-popup-open`. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:269`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:274-275`
- Switching triggers while open reuses the same popup DOM node (no remount).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:469-495`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:833-864`
- Animations are disabled by default in this suite via the `BASE_UI_ANIMATIONS_DISABLED` global.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:13-15`
- The popup exposes `data-starting-style` / `data-ending-style` hooks that tests target with CSS
  animations to verify transition timing. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1107`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1121`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1190`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1208`
- With animations enabled and no exit animation defined, the popup is removed from the DOM on
  close; the exit-animation-finish removal path is covered by the conformance suite but is
  currently skipped there. `packages/react/test/popupConformanceTests.tsx:134-153`, `packages/react/test/popupConformanceTests.tsx:155-157`
- Nested structure beyond Viewport/Portal (e.g. nested alert dialogs) is not tested.
  UNVERIFIED — no test covers nesting.

## Events (names, payload shape, bubbling, preventDefault semantics)

- `onOpenChange(open: boolean, details)` fires once per requested transition.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:150-163`
- `details.reason` values asserted: `REASONS.triggerPress` (trigger click),
  `REASONS.closePress` (Close click), `REASONS.escapeKey` (Escape).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:183`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:191`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:213`
- `details.trigger` is the trigger element that opened the dialog (including its `id`), and it
  still points at the original trigger on a close press — not the close button.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:184-185`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:192-193`
- `details.preventUnmountOnClose()` opts a close transition out of automatic popup unmount; the
  consumer must then call `actionsRef.current.unmount()` to remove it.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:287-291`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:313-317`
- Backdrop clicks produce no `onOpenChange` call at all (pointer dismissal disabled).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:230-232`
- `onOpenChangeComplete(open)` fires when the open/close transition fully settles:
  - On close with no exit animation, it fires with `false` (the initial mount with
    `defaultOpen`/`open` first fires it with `true`).
    `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1090-1091`
  - On close, it fires only after the exit animation finishes.
    `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1144`
  - On open with no enter animation, it fires with `true` (the test records two total calls and
    asserts the first is `true`). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1173-1174`
  - On open, it fires after the enter animation finishes.
    `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1221-1223`
- Event bubbling past the dialog parts (e.g. to ancestor handlers) is not asserted. UNVERIFIED —
  no test covers bubbling.

## Edge cases (rapid interactions, unmount, nesting)

- Repeated open/close cycles across several triggers (in-Root and detached) keep working.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:412-433`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:625-646`
- A controlled close veto preserves the open visual state (`data-open`, `data-popup-open`) and
  `handle.isOpen` until the consumer actually updates `open`.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:236-277`
- After `preventUnmountOnClose`, `actionsRef.current.unmount()` removes the popup, and the
  manual-unmount state is cleared so later close flows unmount normally again.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:281-364`
- Unmounting the root while open (detached-trigger setup), then remounting it: the popup stays
  gone after remount, the detached trigger still opens it, `aria-controls` re-links to the new
  popup id, and backdrop clicks still do not close it (`handle.isOpen` stays `true`).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:649-706`
- Detached triggers remain clickable when reparented — moving them between 0-3 wrapper `<div>`
  levels in both directions keeps open/close working.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:708-742`
- Replacing the handle object (Fast Refresh-like recreation) keeps detached triggers working,
  standalone and combined with reparenting.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:744-793`
- Multiple trigger suites, detached-trigger suites, modality, and `onOpenChangeComplete` suites
  are browser-only (`describe.skipIf(isJSDOM)`).
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:387`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:530`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1046`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1063`
- Modality: when open, a `role="presentation"` backdrop is present (the suite asserts its
  presence as the modality signal; inertness of outside elements itself is not directly
  asserted). `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1047-1060`

## Shared harness dependencies

- `#test-utils` resolves to `packages/react/test/index.ts` (alias defined at
  `packages/react/package.json:110`); this test imports `createRenderer`, `isJSDOM`, and
  `popupConformanceTests` from it. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:6`
- `popupConformanceTests` (`packages/react/test/popupConformanceTests.tsx:6-214`) runs the popup
  conformance suites for this unit with the config passed at
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:17-30`: controlled `open`
  mounting (`packages/react/test/popupConformanceTests.tsx:35-45`), uncontrolled trigger-click
  opening (`packages/react/test/popupConformanceTests.tsx:50-64`), popup role
  (`packages/react/test/popupConformanceTests.tsx:71-76`), trigger `aria-controls`/
  `aria-expanded`/`aria-haspopup` and custom popup `id`
  (`packages/react/test/popupConformanceTests.tsx:80-120`), and animation-driven removal
  (`packages/react/test/popupConformanceTests.tsx:125-212`, with the animation-finish case
  skipped at `packages/react/test/popupConformanceTests.tsx:156-157`).
- `createRenderer` (`packages/react/test/createRenderer.ts:27-48`) wraps the
  `@mui/internal-test-utils` renderer in an awaited `act`; `render` is awaited and exposes
  `rerender`, `setProps`, and the user-event `user` instance.
  `packages/react/test/createRenderer.ts:31-43`
- `isJSDOM` gates the browser-only suites and is re-exported from `@base-ui/utils/testUtils` via
  the harness index. `packages/react/test/index.ts:1`
- `act`, `flushMicrotasks`, `screen`, `waitFor`, and `within` come from the external
  `@mui/internal-test-utils` package. `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:4`
- `REASONS` is imported from the source tree (`../../internals/reasons`), not a harness; tests
  only use it to assert `details.reason` values.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:7`
- `useDialogRootContext` is imported from `../../dialog/root/DialogRootContext` (also not a
  harness) and used by a test-local helper to read the root store state (`modal`,
  `disablePointerDismissal`, `role`) as data attributes.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:8`, `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:1230-1244`
- The `BASE_UI_ANIMATIONS_DISABLED` global is set to `true` before each test in this file; the
  conformance animations suite toggles it off/on around its own tests.
  `packages/react/src/alert-dialog/root/AlertDialogRoot.test.tsx:13-15`, `packages/react/test/popupConformanceTests.tsx:126-132`
