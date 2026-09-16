# React mentions — rendered pages

Generated 2026-09-16T08:19:06.240Z by check-react-mentions.mjs.

**Totals: 5 defect(s), 0 tolerated-reference candidate(s) across 1 route(s).**

Rule: this port points readers at `@noevaresearch/base-ui`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/avatar

fail 5, warn 0, mentions of `@noevaresearch/base-ui`: 0, attributed (credits to upstream): 1

- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Avatar } from '@base-ui/react/avatar'; <Avatar.Root> <Avatar.Image src="" /> <Avatar.Fallback>LT</Avatar.Fallback> </Avatar.Root>;`
- FAIL **package-react** — ships upstream's package/import inside the generated port — the reader must be pointed at @noevaresearch/base-ui
  - `import { Avatar } from '@base-ui/react/avatar'; <Avatar.Root> <Avatar.Image src="" /> <Avatar.Fallback>LT</Avatar.Fallback> </Avatar.Root>;`
- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: className (string | ((state: Avatar.Root.State) => string | undefined) — CSS class applied to the element, or a function that returns a class based on the component's state), style (React.CSSProperties | ((state: Avatar.Root.State) =`
- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: onLoadingStatusChange (((status: ImageLoadingStatus) => void) — callback fired when the loading status changes), className (string | ((state: Avatar.Image.State) => string | undefined)), style (React.CSSProperties | ((state: Avatar.I`
- FAIL **react-api** — a React API where this port uses Leptos (signals, props, view!)
  - `Props: delay (number, 0 — how long to wait before showing the fallback, specified in milliseconds), className (string | ((state: Avatar.Fallback.State) => string | undefined)), style (React.CSSProperties | ((state: Avatar.Fallback.State) =>`
