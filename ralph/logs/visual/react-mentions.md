# React mentions — rendered pages

Generated 2026-09-16T12:57:26.640Z by check-react-mentions.mjs.

**Totals: 2 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `base-ui-leptos`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/collapsible

fail 2, warn 0, mentions of `base-ui-leptos`: 0, attributed (credits to upstream): 0

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at base-ui-leptos
  - `import { Collapsible } from '@base-ui/react/collapsible'; <Collapsible.Root> <Collapsible.Trigger /> <Collapsible.Panel /> </Collapsible.Root>`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at base-ui-leptos
  - `import { Collapsible } from '@base-ui/react/collapsible'; <Collapsible.Root> <Collapsible.Trigger /> <Collapsible.Panel /> </Collapsible.Root>`
