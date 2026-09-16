# React mentions — rendered pages

Generated 2026-09-16T11:39:37.030Z by check-react-mentions.mjs.

**Totals: 2 defect(s), 1 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `@noevaresearch/base-ui`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/otp-field

fail 2, warn 1, mentions of `@noevaresearch/base-ui`: 0, attributed (credits to upstream): 0

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { OTPField } from '@base-ui/react/otp-field'; <OTPField.Root> <OTPField.Input /> <OTPField.Separator /> </OTPField.Root>;`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { OTPField } from '@base-ui/react/otp-field'; <OTPField.Root> <OTPField.Input /> <OTPField.Separator /> </OTPField.Root>;`
- warn **react-word** — the bare word "React" — allowed only as a recorded, reasoned reference to upstream
  - `Pass `autoSubmit` to submit the owning form automatically when all slots are filled, or use `onValueComplete` to react to completion without submitting.`
