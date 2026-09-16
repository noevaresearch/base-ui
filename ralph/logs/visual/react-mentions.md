# React mentions — rendered pages

Generated 2026-09-16T15:07:20.644Z by check-react-mentions.mjs.

**Totals: 1 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `base-ui-leptos`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/meter

fail 1, warn 0, mentions of `base-ui-leptos`: 0, attributed (credits to upstream): 0

- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: children ((formattedValue: string, value: number) => React.ReactNode | null — the render-function form; omission renders the formatted value), className, style, render.`
