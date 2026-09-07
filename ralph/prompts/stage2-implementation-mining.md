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
  given above, not test files. FULL repo-relative path EVERY time, even re-citing a file already
  named earlier in the same paragraph — `check-citations.mjs` resolves each citation
  independently with no memory of prior ones; a bare filename like `` `Foo.tsx:45` `` fails as
  "cited file does not exist" (measured 2026-09-07: this exact shorthand produced 100+ hard
  failures each in `number-field` and a batched `toast` part — don't repeat it).
- Do not propose Rust/Leptos architecture here — that's `specs/architecture.md`, a separate later
  synthesis step, out of scope for this task.
- After writing the spec, run
  `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}`
  to record citation baselines for the new citations you added.

## Batching note (large units only)

If this unit's `TODO.md` entry has `needs-batched-mining: true` (the same flag Stage 1 uses —
measured 2026-09-07: combobox, drawer, floating-ui-react, menu, number-field, select, toast —
each large enough that a single-shot pass risks an upstream idle-timeout before anything gets
written), do NOT dispatch one subagent to read every file in `{{srcFiles}}` at once. Instead:

1. Check whether `specs/library/{{unit}}/parts/` already exists from Stage 1's own batching.
   If so, mirror its partition exactly (same subdirectory groupings, same batch names) — keeping
   behavior parts and implementation parts aligned by name is what lets a reader/future spec-
   consumer go from `parts/<batch>.md` to `parts/<batch>.implementation.md` for the same slice of
   the component, rather than the two stages inventing different boundaries. If no `parts/`
   directory exists yet (Stage 1 wasn't batched for this unit), group `{{srcFiles}}` by their
   immediate subdirectory under the unit the same way Stage 1 does (small/related subdirectories
   may be grouped, e.g. `arrow`+`backdrop`+`icon`; never split one subdirectory's files across
   batches).
2. Dispatch one subagent per batch, each producing `specs/library/{{unit}}/parts/<batch-name>.implementation.md`
   using the same section structure and citation rules as above, scoped only to that batch's
   source files plus whatever it explicitly imports from outside the unit (cite those imports by
   path — do not inline-explain another unit's internals; that belongs to that unit's own spec).
3. Once every batch is done, a final lightweight synthesis task reads all
   `parts/*.implementation.md` files and writes `specs/library/{{unit}}/implementation.md` as an
   index (one paragraph + citation per part, pointing into `parts/<batch-name>.implementation.md`)
   plus any TRULY cross-cutting implementation detail that only makes sense at the whole-unit
   level (e.g. how the root state machine and a shared store/manager coordinate across parts) —
   this synthesis step should stay short; the parts files carry the depth.
