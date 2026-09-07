# Stage 2: Implementation mining (template)

Parameterized by `{{unit}}` and `{{srcFiles}}` (non-test source files for that unit). Run after
Stage 1 has produced `specs/library/{{unit}}/behavior.md` (or `specs/utils/{{unit}}.md`) for the
same unit. Filled in and dispatched per-unit as a bounded fan-out task, one subagent per unit —
NOT a loop.

---

You are given `{{unit}}`'s behavior spec at `specs/library/{{unit}}/behavior.md` (already
written — treat it as ground truth of WHAT happens) and its source files: {{srcFiles}}.

Your job is WHY/HOW: explain the state machine, hook composition, context usage, and DOM
decisions that produce the behavior already documented. Do not restate what behavior.md already
says — cite it by section name if you need to refer back to it, don't re-describe it.

Produce `specs/library/{{unit}}/implementation.md` with:

## State machine / hooks used
Name each hook (e.g. `useControlled`, `useIsoLayoutEffect`), cite its call site.

## Context providers/consumers
What crosses the boundary, and to which child components.

## DOM/portal strategy and why

## Dependencies on other Base UI internals
Other components/utils this unit imports from (e.g. `floating-ui-react`, `internals/`,
`use-render`) — this section is what `ralph/scripts/generate-todo.mjs`'s coarse "blocked-by:
[Phase A complete]" default should eventually be replaced with, once enough units have this
section filled in to compute precise per-component dependencies.

## Anything in source not explained by any test
Flag explicitly — this is exactly the gap the golden-fixture stage and the backward-looking
audit loop need to know about independently; do not silently paper over it.

RULES:
- If this unit's `TODO.md` entry has a `wraps-external:` field, the "Dependencies on other Base
  UI internals" section must explicitly state the delegation (which external package, which Rust
  crate replaces it — both named in the TODO.md fields) rather than treating the external
  package's internals as something this spec needs to derive.
- Same citation format as Stage 1 (backtick-wrapped `` `path:line` ``), citing the SOURCE files
  given above, not test files.
- Do not propose Rust/Leptos architecture here — that's `specs/architecture.md`, a separate later
  synthesis step, out of scope for this task.
- After writing the spec, run
  `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}`
  to record citation baselines for the new citations you added.
