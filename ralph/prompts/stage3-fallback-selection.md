# Stage 3: Fallback selection (used only when the mechanical picker finds nothing)

`ralph/scripts/pick-next-todo.mjs` is the default, mechanical item selector — deterministic,
dependency-aware, and it's what keeps 50+ independent stateless iterations from having to each
correctly re-derive phase ordering and the docs-pairing rule on their own judgment. This prompt is
the fallback for its one failure mode: it found no `not-started` item whose `blocked-by` deps are
all `done`. That has three possible causes, and they need different responses — do not assume
which one it is, determine it.

---

You are one stateless iteration of a Ralph port loop. The mechanical item picker found nothing
pickable in `TODO.md`. Read `CONTEXT.md`, then all of `TODO.md`, then determine which case this is:

**(a) The port is actually complete.** Every item is `status: done`. If so: do nothing, change
nothing, and report that clearly — do not invent work.

**(b) Everything remaining is genuinely, correctly blocked.** E.g. all Phase B items are waiting
on a Phase A item that is legitimately still `not-started`. If so: do nothing, change nothing, and
report which item(s) are the actual bottleneck, so a human knows what to prioritize.

**(c) The `blocked-by` graph is wrong or overly conservative.** This is the case most worth
checking for, because `generate-todo.mjs` deliberately used a coarse default
(`blocked-by: [Phase A complete]` on every Phase B item, meaning "every Phase A item," even though
most components only depend on a handful of specific utils/infra crates). If a Phase B item's
`specs/library/<name>/implementation.md` (once it exists) already documents its real dependencies
under "Dependencies on other Base UI internals," and those specific dependencies are all `done`
even though *other*, unrelated Phase A items aren't, this item is safe to work on despite the
mechanical block.

If and only if you determine (c):
1. Pick exactly ONE such item.
2. Before implementing anything, edit its `TODO.md` entry: replace `blocked-by: [Phase A
   complete]` with the precise list of items it actually depends on (cite the
   implementation.md section that proves this in a trailing comment on the line). Do not simply
   delete or blank the `blocked-by` field — narrowing it must be justified and visible.
3. Only after that edit, proceed exactly as `ralph/prompts/stage3-forward-loop.md` describes from
   its step 2 onward, for this item.
4. In your commit message, state explicitly that this item was picked via the fallback path and
   why the narrower dependency is correct, so a human reviewing `git log` can see this wasn't the
   mechanical picker's normal path.

Do not use this fallback to route around the docs-pairing rule itself (`docs-pair:` /
`check-todo-schema.mjs`) — that rule is not a scheduling artifact, it's the actual definition of
done, and narrowing `blocked-by` never touches it.
