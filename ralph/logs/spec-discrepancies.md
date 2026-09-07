# Spec discrepancies log

Append-only findings where a spec is wrong or incomplete relative to upstream (or where the
spec corpus's recorded state needs audit-loop attention). Never edit or delete prior entries;
the backward-looking audit loop resolves them.

---

- 2026-09-07, iteration for `utils: clamp` (commit 36bbc8da0): A **full-scope** citation check
  (`node ralph/scripts/check-citations.mjs check`, no `--scope`) reports 66 drifted
  `TODO.md:<range>` citations across 60+ files under `specs/library/**` — drift that **predates
  this iteration** and is invisible to the per-item gate, because `run-regression.sh` scopes the
  citation check to the item's own specs directory (`specs/utils` for Phase A utils items).
  Fully verified before logging: all 66 (plus the 18 `specs/utils` ones the gate did flag)
  match their recorded baselines as **pure line-number shifts with content-identical windows**
  — zero genuine content drift at any offset in −4..+4. The library baselines were recorded
  before the loop's two done-marking insertions into TODO.md (commits 680094cd4 and
  3ca0fa8dc, each +1 line above every Phase B/infra entry), so they are stale by exactly +2 at
  HEAD (now +3 after this iteration's clamp done-marking edit). No library iteration has yet run
  a gate over its own scope, so none has re-recorded them. Resolution guidance for whichever
  iteration/library scope picks this up: the same verify-then-re-record procedure used for
  `specs/utils` (see commits dc1ed1ff2 and the clamp re-record) applies verbatim — re-verify the
  shift is content-identical at the current offset, then
  `node ralph/scripts/check-citations.mjs record --scope specs/library`. Not re-recorded here
  because it is outside this item's scope and gate.
