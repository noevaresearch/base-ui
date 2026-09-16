# React mentions — rendered pages

Generated 2026-09-16T09:22:59.040Z by check-react-mentions.mjs.

**Totals: 2 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `@noevaresearch/base-ui`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/checkbox-group

fail 2, warn 0, mentions of `@noevaresearch/base-ui`: 0, attributed (credits to upstream): 0

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Checkbox } from '@base-ui/react/checkbox'; import { CheckboxGroup } from '@base-ui/react/checkbox-group'; <CheckboxGroup> <Checkbox.Root /> </CheckboxGroup>;`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Checkbox } from '@base-ui/react/checkbox'; import { CheckboxGroup } from '@base-ui/react/checkbox-group'; <CheckboxGroup> <Checkbox.Root /> </CheckboxGroup>;`
