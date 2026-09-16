# React mentions — rendered pages

Generated 2026-09-16T12:46:47.304Z by check-react-mentions.mjs.

**Totals: 4 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `base-ui-leptos`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/utils/csp-provider

fail 4, warn 0, mentions of `base-ui-leptos`: 0, attributed (credits to upstream): 0

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at base-ui-leptos
  - `import { CSPProvider } from '@base-ui/react/csp-provider'; // prettier-ignore <CSPProvider nonce="..."> {/* Your app or a group of components */} </CSPProvider>`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at base-ui-leptos
  - `import { CSPProvider } from '@base-ui/react/csp-provider'; // prettier-ignore <CSPProvider nonce="..."> {/* Your app or a group of components */} </CSPProvider>`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at base-ui-leptos
  - `import { CSPProvider } from '@base-ui/react/csp-provider'; function App({ nonce }) { return <CSPProvider nonce={nonce}>{/* ... */}</CSPProvider>; }`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at base-ui-leptos
  - `import { CSPProvider } from '@base-ui/react/csp-provider'; function App({ nonce }) { return <CSPProvider nonce={nonce}>{/* ... */}</CSPProvider>; }`
