# React mentions — rendered pages

Generated 2026-09-16T11:37:20.548Z by check-react-mentions.mjs.

**Totals: 4 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `@noevaresearch/base-ui`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/form

fail 4, warn 0, mentions of `@noevaresearch/base-ui`: 0, attributed (credits to upstream): 0

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Field } from '@base-ui/react/field'; import { Form } from '@base-ui/react/form'; <Form> <Field.Root> <Field.Label /> <Field.Control /> <Field.Error /> </Field.Root> </Form>;`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Field } from '@base-ui/react/field'; import { Form } from '@base-ui/react/form'; <Form> <Field.Root> <Field.Label /> <Field.Control /> <Field.Error /> </Field.Root> </Form>;`
- FAIL **package-react** — link target points at upstream's package/site instead of @noevaresearch/base-ui
  - `Server Function`
- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: errors (Errors — validation errors returned externally, typically after submission by a server or a form action; this should be an object where keys correspond to the name attribute on <Field.Root>, and values correspond to error(s) `
