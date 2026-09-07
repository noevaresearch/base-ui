# Stage 1 (docs): Docs-content page mining (template)

Parameterized by `{{unit}}` (component/util name, e.g. `accordion`), `{{route}}` (docs route,
e.g. `components/accordion`), and `{{pagePath}}` (the page's `.mdx` file). Filled in and
dispatched per-page as a bounded fan-out task, one subagent per page — this is NOT a loop.
Mirrors `ralph/prompts/stage1-behavior-mining.md`'s framing exactly, scoped to docs content
instead of component tests.

---

You are mining DOCS CONTENT ONLY, not implementation, and not the component's own behavior
(that's already covered by `specs/library/{{unit}}/behavior.md` — read it first, don't
re-derive it; cite it by section name instead of restating it).

Read: {{pagePath}}

Produce/update `specs/docs-content/{{unit}}/page.md` with these exact sections:

## Page structure (headings, in order)
## Prose claims about component behavior
Only claims the page text makes (props described, usage guidance, caveats) — cross-check each
against `specs/library/{{unit}}/behavior.md` and flag any mismatch under "Discrepancies" rather
than silently trusting either source.
## API tables referenced (props/parts documented on this page)
## Code snippets embedded directly in the .mdx (not pulled from demos/)
## Discrepancies (docs page vs. behavior.md)
## Cross-links to other docs pages

RULES:
- Every non-trivial claim MUST end with a citation in the exact form
  `` `docs/src/app/(docs)/react/{{route}}/page.mdx:123` `` (or a range `:123-145`) —
  backtick-wrapped, this exact shape, because `ralph/scripts/check-citations.mjs` parses it
  mechanically.
- Do not describe what you think the page *should* say — only what it actually says.
- Do not open or reference demo source files here — that's Stage 2 (docs)'s job
  (`specs/docs-content/{{unit}}/demos.json`), scoped separately so page prose and demo code
  don't get mined by the same pass.
- Do NOT write any Rust or Leptos code. Do NOT touch `crates/` or `docs-app/`.
- Sections that don't apply should say "N/A" rather than being omitted, so downstream tooling
  can rely on the section always existing.
- After writing the spec, run
  `node ralph/scripts/check-citations.mjs record --scope specs/docs-content/{{unit}}`
  to record citation baselines.
