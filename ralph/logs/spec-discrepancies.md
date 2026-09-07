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

- 2026-09-08, iteration for `utils: createLogOnce` (unblock; port commit 23d746646): **Third
  occurrence** of the done-marking TODO.md citation-drift treadmill in `specs/utils` (after
  dc1ed1ff2 and the clamp re-record e7b2b7e4f). The item's port was complete and its own gate
  green, but the done-marking edit itself (1a3bf865f, +1 line for `commit:`) then the driver's
  blocked correction (a77195112, +1 line for `note:`) shifted every backticked
  `TODO.md:<range>` citation below the entry, so the driver's independent regression re-run
  hard-failed the citation check (19 failure instances / 18 unique keys) and the item got marked
  `blocked` — for a failure caused by the loop's own bookkeeping, not the port. Verified before
  re-recording: all 18 keys matched their recorded baselines at exactly +2 at that HEAD (then
  +1 after this iteration's done-marking edit, which removed the `note:` line), zero content
  drift at any offset in −8..+8; re-recorded `record --scope specs/utils` (17 sidecar files, no
  spec prose touched).

  Structural note for the audit loop: this WILL recur on every Phase A/infra done-marking. The
  agent's own gate passes because the drift is caused by the done-marking edit, which by
  definition lands after the gate; the driver's independent re-run then fails. Durable options:
  (a) re-record the item's specs scope inside the done-marking commit itself (single consistent
  state), (b) make check-citations.mjs shift-tolerant — re-find the recorded window content at
  nearby line offsets and hard-fail only on genuine content mismatch, (c) cite TODO.md entries
  by a stable anchor instead of line ranges. None implemented here (out of scope for one item).

  Also: re-verified the `specs/library/**` drift left un-re-recorded by the 2026-09-07 entry —
  now stale by exactly **+4** at this HEAD (the predicted +3 after clamp, plus 1a3bf865f's +1,
  with a77195112's +1 and this iteration's −1 canceling). All 66 unique library keys (75 failure
  instances, some keys cited by multiple files — e.g. TODO.md:565-575 by six tooltip part specs)
  verified content-identical at +4, zero genuine drift in −8..+8. Still not re-recorded, same
  reasoning as before: outside this item's scope and gate. First library iteration whose gate
  covers `specs/library` should verify-then-re-record per the guidance above.

- 2026-09-08, iteration for `utils: error` (port commit ee27c1f29): Fourth occurrence of the
  done-marking TODO.md citation-drift treadmill in `specs/utils` (after dc1ed1ff2, e7b2b7e4f,
  and 780e3abc9) — this time handled preemptively per durable option (a): the done-marking edit
  (+1 line, `commit: ee27c1f29`, at the `utils: error` entry) was verified as a pure insertion
  via `git diff TODO.md` (line-neutral `status:` swap plus one added line; all content below
  line 42 identical, shifted exactly +1), then baselines were re-recorded with
  `record --scope specs/utils` (17 sidecar files) inside the same done-marking commit, so the
  driver's independent regression re-run sees one consistent state. No spec prose touched.
  Cumulative `specs/library/**` drift (2026-09-07 and 2026-09-08 entries above) is now stale by
  exactly **+5** for every `TODO.md:<range>` citation whose range starts below line 42 (was +4
  after the createLogOnce iteration; this edit's +1 applies). Still not re-recorded, same
  reasoning: outside this item's scope and gate.
