# Stage 2: Batch synthesis (template) — for needs-batched-mining units only

Parameterized by `{{unit}}`. Runs after every batch in `specs/library/{{unit}}/parts/PLAN.json`
has a matching `parts/<batchName>.implementation.md`. This is a deliberately LIGHT final pass —
its own process, separate from the batches themselves.

---

Read `specs/library/{{unit}}/parts/PLAN.json` and every `parts/*.implementation.md` it names.

Write `specs/library/{{unit}}/implementation.md` as an index: one short paragraph + citation per
part (pointing into `parts/<batchName>.implementation.md`, e.g. "see `parts/root.implementation.md`
for the state machine"), plus any TRULY cross-cutting detail that only makes sense at the
whole-unit level — how batches' state/context coordinate with each other. Keep this short; the
parts carry the depth, this file is a map to them, not a restatement.

RULES:
- Do not re-derive content already in a part — cite it by file, don't copy it.
- Any cross-cutting claim not already covered by a part still needs its own
  `` `path:line` `` citation, full repo-relative path.
- After writing, run
  `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}/implementation.md`
  to record citation baselines.
