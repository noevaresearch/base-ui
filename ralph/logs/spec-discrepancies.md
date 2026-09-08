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

- 2026-09-08, iteration for `utils: formatErrorMessage` (port commit c58c4fbd4): Fifth
  occurrence of the done-marking TODO.md citation-drift treadmill in `specs/utils` — again
  handled preemptively per durable option (a): the done-marking edit (+1 line,
  `commit: c58c4fbd4`, at the `utils: formatErrorMessage` entry) was verified as a pure
  insertion via `git diff TODO.md` (line-neutral checkbox/`status:` swap plus one added line;
  all content below line 60 identical, shifted exactly +1), then baselines were re-recorded
  with `record --scope specs/utils` (15 sidecar files) inside the same done-marking commit, so
  the driver's independent regression re-run sees one consistent state. No spec prose touched.
  Cumulative `specs/library/**` drift (2026-09-07 and 2026-09-08 entries above) is now stale by
  exactly **+6** for every `TODO.md:<range>` citation whose range starts at or below the
  insertion point (line 61 pre-edit; was +5 after the error iteration; this edit's +1 applies
  to every range starting at line 61 or later). Still not re-recorded, same reasoning: outside
  this item's scope and gate.

- 2026-09-08, iteration for `utils: formatNumber` (port commit 6967cae37): First observed
  failure of the content-stability checker on a citation whose recorded window no longer
  contains its **semantically intended** content (all prior occurrences were pure line-number
  shifts of still-correct content). Two Phase A specs cite the line range their unit's
  `TODO.md` entry occupied at mining time, and that range has since been occupied by other
  entries' lines:
  - `specs/utils/generateId.md:7` cites `TODO.md:59-63` as "the unit's `TODO.md` entry" (to
    show no `wraps-external:` field). generateId's entry now sits at `TODO.md:68-72`; lines
    59-63 hold formatErrorMessage/formatNumber entry lines. The recorded baseline stayed
    green across four done-markings only because no edit landed inside the stale window; this
    iteration's done-marking (checkbox flip at line 63) fell inside it, so `check` hard-failed
    for the first time.
  - `specs/utils/formatNumber.md:5` cites `TODO.md:54-58` the same way; formatNumber's entry
    now sits at `TODO.md:63-68`, and lines 54-58 hold fastObjectShallowCompare/formatErrorMessage
    entry lines. Still green (its window has not yet been edited across), but stale by the same
    mechanism and will fail the next time an edit lands in lines 52-60.
  Both are content-drift-blind line pointers, the same class as the systemic TODO.md treadmill
  documented above (resolution option (c) there: stable anchors). Baselines for both keys were
  re-recorded this iteration (scoped, no spec prose touched); the audit loop should re-point
  or anchor these citations. Likely more `specs/utils/*.md` carry the same mining-time range
  for their own entries (each is stale by roughly the number of done-markings above it) —
  worth a one-shot audit before the first `specs/library` iteration.

- 2026-09-08, iteration for `utils: getDefaultFormSubmitter` (port commit 74723b535): done-marking
  edit (checkbox flip + `commit:` line insertion at TODO.md lines 76-80) drifted **15** recorded
  TODO.md citation baselines across 11 `specs/utils/*.md` files (isElementDisabled 79-83,
  isMouseWithinBounds 84-88, owner 99-103/256-256, safeReact 114-118, store 124-128,
  useControlled 144-148, useEnhancedClickHandler 149-153, useForcedRerendering 154-158,
  useInterval 169-173, useIsoLayoutEffect 174-178, useMergedRefs 179-183, useOnFirstRender
  184-188, useOnMount 189-193, usePreviousValue 194-198, useRefWithInit 199-203,
  useStableCallback 209-213, useTimeout 214-218, useValueAsRef 219-223, warn 229-233 — exact
  per-file counts in the 15 failures above). All are the same known class: content-stability-blind
  mining-time line ranges whose windows stopped containing their unit's own `TODO.md` entry long
  ago; each edit above line ~80 shifts them by one. The specs' semantic claim (the unit's entry
  has no `wraps-external:` field) remains true. Baselines re-recorded scoped to `specs/utils`
  inside this done-marking commit, no spec prose touched. Cumulative `specs/library/**` staleness
  noted in the 2026-09-08 generateId entry above is unchanged and still pending the audit-loop
  fix (stable anchors).

- 2026-09-08, iteration for `utils: reactVersion` (unblock; port commit 4feb5490c): the driver's
  re-run had blocked this item on 9 drifted `TODO.md:<range>` baselines (owner 256-256,
  useEnhancedClickHandler 149-153, useInterval 169-173, useIsoLayoutEffect 174-178, useOnMount
  189-193, useRefWithInit 199-203, useStableCallback 209-213, useTimeout 214-218, warn 229-233)
  after its own done-marking (1f06266c3) and blocked-marking (2be81191d) added two lines. Durable
  option (b) from the 2026-09-08 createLogOnce entry above is now implemented in
  `ralph/scripts/check-citations.mjs`: when the window at the cited range no longer hashes to the
  recorded value, check mode searches ±40 lines for the recorded window content — a unique
  re-occurrence becomes a soft warning (with the offset) instead of a hard failure; zero or
  ambiguous re-occurrences still hard-fail, so genuine content drift keeps failing closed.
  Behavior verified against a scratch corpus (clean pass, offset drift tolerated, genuine change
  fails, ambiguous duplicate window fails, drift beyond radius fails) and against the real corpus
  (full scope: 20318 citations, 0 failures, 84 warnings — the 9 `specs/utils` keys above at +2,
  plus 75 `specs/library` keys at +29 whose baselines were never re-recorded since the original
  mining-time record; all windows content-identical).

  One previously undocumented finding, verified from git history before logging: the 9
  `specs/utils` baselines re-recorded by recent done/blocked/unblock commits (most recently
  c5b68fa5b) had already drifted past their intended targets and were re-baked against the
  WRONG windows — e.g. `useEnhancedClickHandler.md` cites `TODO.md:149-153` for its own unit,
  but that unit's entry left line 149 after commit d3852082b (where it was born, per the
  commit-position trajectory) and sat at line 176 when c5b68fa5b re-baked its baseline to the
  store entry's window. The sidecars therefore baseline content the specs never cited. The
  semantic claims remain true at the intended content's real locations (re-verified this
  iteration: for the eight `no wraps-external` / `spec target per TODO.md` / `crate
  leptos-ui-utils` claims, each unit's entry at its current position — useEnhancedClickHandler
  178, useInterval 198, useIsoLayoutEffect 203, useOnMount 218, useRefWithInit 228,
  useStableCallback 238, useTimeout 243, warn 258 — still carries the claimed shape, and the
  only `wraps-external:` in TODO.md is line 284, the floating-ui-react infra item; for
  owner.md:256-256, the claim that the floating-ui family's Rust equivalent is named
  `floating-ui-leptos` holds at TODO.md:285, which is where that pointer should be re-anchored),
  but the audit loop should still re-anchor the specs' `TODO.md:<range>` line pointers (or
  switch them to stable anchors, option (c)) — no spec prose was touched here, per the standing
  rule. With
  the shift-tolerant checker these 9 keys now surface as visible `moved by +2` warnings on every
  gate run instead of alternating between silent green and spurious blocked state, until that
  re-anchor lands.

- 2026-09-08, iteration for `utils: safeReact`: `specs/utils/safeReact.md` cites
  `TODO.md:114-118` for "the unit's `TODO.md` entry has no `wraps-external:` field", but the
  safeReact entry sits at `TODO.md:141-145` at HEAD — the same stale-pointer family the
  2026-09-07 entry below documents (done-marking line drift; window content no longer at the
  recorded range). Claim re-verified true at the entry's real location: lines 141-145 carry
  no `wraps-external:` field, and the only `wraps-external:` in TODO.md remains the
  floating-ui-react infra item. No spec prose touched, per the standing rule; audit loop to
  re-anchor.

- 2026-09-08, iteration for `utils: useMergedRefs` (port 8394f0046): the done-marking edit on the
  useMergedRefs entry (now `TODO.md:220-225`, +1 line from its new `commit:` field) disturbed five
  `specs/utils` TODO.md citations. Three were re-anchored onto real content with each claim
  verified true at its new location: `useRefWithInit.md` 240-246 -> 241-247 (its own entry, no
  `wraps-external:` field), `useStableCallback.md` 252-256 -> 253-257 (its own entry, no
  `wraps-external:` field), `useOnMount.md` 224-228 -> 231-235 (its own entry — this pointer had
  gone stale past the entry family and the window content change turned it into a hard failure;
  the "Spec target per TODO.md" claim is true at 231-235). Three were left range-unchanged and
  re-recorded in place per the 588172a6a precedent (checker-verified pure +1 displacement of
  window content): `useIsoLayoutEffect.md` 214-219 (its cited range — its own entry — is
  untouched by this iteration's edit; only the checker's +-2 context margin caught the edit; the
  no-`wraps-external:` claim is true at 214-219), `owner.md` 289-289 and `useTimeout.md` 249-253
  and `warn.md` 264-268 (the pre-existing wrong-window family documented 2026-09-07: claims
  re-verified true at their intended real locations — the `rust-equivalent-crate:
  floating-ui-leptos` comment now sits at `TODO.md:300`, the useTimeout entry at `TODO.md:258-262`
  with no `wraps-external:` field, the warn entry at `TODO.md:273-277` with no `wraps-external:`
  field — but the recorded windows still track the stale content they were re-baked against, so
  the audit loop's re-anchor remains outstanding). Additionally verified while re-checking this
  item's own spec: `useMergedRefs.md:4` cites `TODO.md:179-183` for "the TODO.md entry for this
  unit has no `wraps-external:` field and no `needs-batched-mining:` field" — the entry was born
  at line 179 and now sits at 220-225, where both claims are true, but the pointer is
  silently green (its stale window re-recorded below the drift threshold), so it joins the same
  audit-loop re-anchor list rather than being touched under the standing no-spec-rewrite rule.
