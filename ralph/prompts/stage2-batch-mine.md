# Stage 2: Single-batch implementation mining (template)

Parameterized by `{{unit}}`, `{{batchName}}`, `{{batchFiles}}` (explicit file list — from
`specs/library/{{unit}}/parts/PLAN.json`, computed by `stage2-plan-batches.md`). This process
mines EXACTLY ONE batch and nothing else — real process isolation per batch, dispatched by
`ralph2.sh` itself (not an internal subagent fan-out inside one long-lived process, which is what
previously stalled: measured 2026-09-07, toast and number-field each stalled silently ~12-15
minutes into an internal multi-batch pass). One batch, one bounded task, one disposable process —
same principle as every other Ralph stage.

---

You are given `{{unit}}`'s behavior spec at `specs/library/{{unit}}/behavior.md` (or, if it
exists, the matching batch at `specs/library/{{unit}}/parts/{{batchName}}.md`) as ground truth of
WHAT happens, and this batch's source files: {{batchFiles}}.

Your job is WHY/HOW, for THIS BATCH ONLY: explain the state machine, hook composition, context
usage, and DOM decisions that produce the documented behavior for these files. Do not restate
behavior already documented — cite it by section name instead.

Produce `specs/library/{{unit}}/parts/{{batchName}}.implementation.md` with:

## State machine / hooks used
## Context providers/consumers
## DOM/portal strategy and why
## Dependencies on other Base UI internals
Other components/utils this batch imports from OUTSIDE this unit — cite those imports by path,
do not inline-explain another unit's internals.
## Anything in source not explained by any test

RULES:
- Scope strictly to `{{batchFiles}}` plus whatever they explicitly import from outside the unit
  (cite those imports, don't re-derive their internals). Do not read other batches' files unless
  one of your batch's files imports from them.
- If this unit's `TODO.md` entry has a `wraps-external:` field and it's relevant to this batch's
  files, state the delegation explicitly rather than deriving the third-party algorithm.
- Citation format: backtick-wrapped `` `path:line` ``, FULL repo-relative path EVERY time, even
  re-citing a file already named earlier in the same paragraph — `check-citations.mjs` resolves
  each citation independently with no memory of prior ones; a bare filename like `` `Foo.tsx:45` ``
  fails as "cited file does not exist" (measured 2026-09-07: this exact shorthand produced 100+
  hard failures — don't repeat it).
- Do not propose Rust/Leptos architecture here — that's `specs/architecture.md`, out of scope.
- After writing, run
  `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}/parts/{{batchName}}.implementation.md`
  to record citation baselines.
