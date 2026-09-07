# Stage 2: Batch planning (template) — for needs-batched-mining units only

Parameterized by `{{unit}}` and `{{srcFiles}}`. This is a PLANNING-ONLY task — fast and shallow
by design, because the failure mode it exists to prevent is a single long-lived process
accumulating context across many batches until it stalls (measured 2026-09-07: toast and
number-field's Stage 2 mining each stalled twice, silently, ~12-15 minutes in, after an internal
subagent-per-batch fan-out inside ONE process — real process isolation per batch is the fix, and
this plan is what makes that possible without an LLM re-deciding the grouping on every retry).

---

Do NOT mine any implementation content. Do NOT read file bodies beyond skimming names/directory
structure. Your only job: partition `{{srcFiles}}` into named batches and write
`specs/library/{{unit}}/parts/PLAN.json`.

Rules for the partition:
- If `specs/library/{{unit}}/parts/*.md` already exists (Stage 1's own behavior batching), mirror
  that partition's batch names and groupings exactly — do not invent a different split.
- Otherwise, group by immediate subdirectory under the unit; small/related subdirectories (and
  loose top-level files) may be merged into one batch, but never split one subdirectory's files
  across batches. Aim for roughly 4-10 files per batch.
- Every file in `{{srcFiles}}` must appear in exactly one batch.

Write `specs/library/{{unit}}/parts/PLAN.json`:

```json
{
  "batches": [
    { "name": "root", "files": ["packages/react/src/{{unit}}/root/X.tsx", "..."] },
    { "name": "...", "files": ["..."] }
  ]
}
```

That's it — no other output, no other files written.
