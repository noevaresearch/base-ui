# Stage 1: Behavior mining (template)

Parameterized by `{{unit}}` (component/util/infra-dir name), `{{testFiles}}` (list), and
`{{sharedHarnessFiles}}` (list, may be empty). Filled in and dispatched per-unit as a bounded
fan-out task, one subagent per unit — this is NOT a loop.

---

You are mining BEHAVIOR ONLY, not implementation. You are given exactly one unit: `{{unit}}`.

Read every file in: {{testFiles}}

If any of those files import a shared test-utils harness (e.g. `#test-utils`, or a file under
`packages/react/test/`), also read that harness file and note it under "Shared harness
dependencies" below — do not read any other component's test files.

Produce/update `specs/library/{{unit}}/behavior.md` (or `specs/utils/{{unit}}.md` for a
packages/utils/src unit) with these exact sections:

## Public API surface (props, parts, subcomponents)
## State model (controlled/uncontrolled, defaults, transitions)
## Keyboard interactions
## Focus management
## Accessibility (roles, aria-*, id linking)
## DOM structure & portal behavior
## Events (names, payload shape, bubbling, preventDefault semantics)
## Edge cases (rapid interactions, unmount, nesting)
## Shared harness dependencies

RULES:
- Every non-trivial claim MUST end with a citation in the exact form
  `` `packages/react/src/{{unit}}/X.test.tsx:123` `` (or a range `:123-145`) — backtick-wrapped,
  this exact shape, because `ralph/scripts/check-citations.mjs` parses it mechanically.
- Do not describe what you think the component *should* do — only what the tests actually prove.
  If behavior is only informally asserted (no test covers it), write:
  "UNVERIFIED — inferred from `path:line`, no test asserts this."
- Do not open or reference files outside the given test files, except shared harness files.
- Do NOT write any Rust or Leptos code. Do NOT touch `crates/`.
- Sections that don't apply (e.g. a non-visual util has no "Keyboard interactions") should say
  "N/A" rather than being omitted, so downstream tooling can rely on the section always existing.
- After writing the spec, run `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}`
  (or the matching `specs/utils/{{unit}}.md` path) to record citation baselines.
