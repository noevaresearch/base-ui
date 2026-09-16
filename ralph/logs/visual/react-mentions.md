# React mentions — rendered pages

Generated 2026-09-16T10:33:00.724Z by check-react-mentions.mjs.

**Totals: 4 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `@noevaresearch/base-ui`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/fieldset

fail 4, warn 0, mentions of `@noevaresearch/base-ui`: 0, attributed (credits to upstream): 2

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Fieldset } from '@base-ui/react/fieldset'; <Fieldset.Root> <Fieldset.Legend /> </Fieldset.Root>;`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Fieldset } from '@base-ui/react/fieldset'; <Fieldset.Root> <Fieldset.Legend /> </Fieldset.Root>;`
- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: className (string | ((state: Fieldset.Root.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSProperties | ((state: Fieldset.Root.Stat`
- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: className (string | ((state: Fieldset.Legend.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSProperties | ((state: Fieldset.Legend.`
