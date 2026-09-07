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
- Before reading source, check this unit's `TODO.md` entry for a `wraps-external:` field. If
  present, this unit primarily wraps a third-party npm package, and a Rust/Leptos equivalent of
  that package already exists (named in `rust-equivalent-crate:`) — do NOT attempt to derive the
  third-party library's own algorithm from its behavior. Scope entirely to files inside this
  unit's own directory (its wrapper hooks/components), and in the spec's relevant section note
  that the underlying algorithm is delegated to the named external package, with the Rust
  equivalent crate named for Stage 3 to bind against instead of reimplementing.
- Every non-trivial claim MUST end with a citation in the exact form
  `` `packages/react/src/{{unit}}/X.test.tsx:123` `` (or a range `:123-145`) — backtick-wrapped,
  this exact shape, because `ralph/scripts/check-citations.mjs` parses it mechanically. This is
  the FULL repo-relative path, EVERY time, even the second/third mention of a file already named
  earlier in the same sentence or paragraph — `` `X.test.tsx:45` `` is not valid shorthand once
  `` `packages/react/src/{{unit}}/X.test.tsx:12` `` established the path; the checker resolves
  each citation independently and has no memory of prior ones (measured 2026-09-07: this exact
  shorthand pattern produced 100+ hard failures each in `number-field` and `toast`'s
  `leaf-parts` batch — don't repeat it).
- Do not describe what you think the component *should* do — only what the tests actually prove.
  If behavior is only informally asserted (no test covers it), write:
  "UNVERIFIED — inferred from `path:line`, no test asserts this."
- Do not open or reference files outside the given test files, except shared harness files.
- Do NOT write any Rust or Leptos code. Do NOT touch `crates/`.
- Sections that don't apply (e.g. a non-visual util has no "Keyboard interactions") should say
  "N/A" rather than being omitted, so downstream tooling can rely on the section always existing.
- After writing the spec, run `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}`
  (or the matching `specs/utils/{{unit}}.md` path) to record citation baselines.

## Batching note (large units only)

If this unit's `TODO.md` entry has `needs-batched-mining: true` (measured 2026-09-07:
combobox, drawer, floating-ui-react, menu, number-field, select — each ≥8,000 test lines or
≥20 test files; combobox's `root/` subdirectory alone is over 2x Dialog's entire suite), do NOT
dispatch one subagent to read every test file in `{{testFiles}}` at once. Instead:

1. Group `{{testFiles}}` by their immediate subdirectory under the unit (e.g. `combobox/root/`,
   `combobox/input/`, `combobox/items/`, ...; small/related subdirectories may be grouped into one
   batch, e.g. `arrow`+`backdrop`+`icon`, but never split a single subdirectory's tests across
   batches — its tests are usually testing one cohesive piece of behavior).
2. Dispatch one subagent per batch, each producing `specs/library/{{unit}}/parts/<batch-name>.md`
   using the same section structure and citation rules as above, scoped only to its batch's files.
3. Once every batch is done, a final lightweight synthesis task reads all `parts/*.md` files and
   writes `specs/library/{{unit}}/behavior.md` as an index (one paragraph + citation per part,
   pointing into `parts/<batch-name>.md`) plus any TRULY cross-cutting behavior that only makes
   sense at the whole-unit level (e.g. how the root state machine and the items list coordinate) —
   this synthesis step should stay short; the parts files carry the depth.
