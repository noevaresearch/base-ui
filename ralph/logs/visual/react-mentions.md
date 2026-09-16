# React mentions — rendered pages

Generated 2026-09-16T16:13:16.236Z by check-react-mentions.mjs.

**Totals: 0 defect(s) — 0 gated (react-api, package-react), 0 re-homed to another item (snippet-react); 1 tolerated-reference candidate(s) across 1 route(s).**

Source scan this run: 0 gated defect(s) (see react-mentions-source.md).

Rule: this port points readers at `base-ui-leptos`; React APIs in prose or snippets are defects; the bare word "React" is tolerated only when a page lists it in `specs/docs-content/<name>/react-allow.json` with a reason.

## react/components/form

fail 0 (0 gated: react-api, package-react), warn 1, mentions of `base-ui-leptos`: 0, attributed (credits to upstream): 1

- warn **react-word** — the bare word "React" — allowed only as a recorded, reasoned reference to upstream
  - `Upstream's React docs submit this demo with a server function, through React DOM's `useActionState`, instead of `onSubmit`. Server functions are a React DOM feature with no counterpart in this Rust/Leptos port, so this mirror keeps the demo`
