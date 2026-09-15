# Visual-baseline rebake, 2026-09-15 (the content-recall formula changed under the recorded scores)

`ralph/generated/visual-baseline.json` recorded its first three routes at 19:46-19:49 under a
SIX-term content-recall formula. Commit 3873d8a3e (20:22:38, the `docs-spec` iteration) added a
seventh term — `snippetLanguage` = `leptos / total` snippets, i.e. PURITY — so a page that embeds
upstream's React source now scores 0 on that term instead of being credited for the copied text.
Every ported page currently carries React snippets, so the formula change alone lowers every
route's content recall by roughly a seventh, and a measurement taken after it can no longer be
compared with the recorded baseline. The baselines were rebaked in the `docs-chrome: layout shell`
iteration for exactly that reason, with the proof below rather than a bare `--update`.

## Method (reproduce, then compare)

The pre-chrome raw `visual-diff.mjs` reports were still on disk (`/tmp/visual-diff-{checkbox,button,meter}/report.json`,
19:32-19:40 — the same build the baseline was measured on), so the recorded baseline can be
REPRODUCED term-by-term rather than trusted: recomputing `mean(headings, demos, codeBlocks, tables,
links, textLen)` on those reports returns 65.53 / 71.60 for checkbox / button, identical to the
recorded `score` to the hundredth, which confirms what formula produced the baseline.

| route | recorded baseline (6-term) | pre-chrome, 6-term (reproduced) | pre-chrome, 7-term | post-chrome, 7-term | pre→post, same formula | visual proximity pre→post |
| --- | --- | --- | --- | --- | --- | --- |
| react/components/checkbox | 65.53 / visual 85.41 / content 35.70 | 65.53 / 35.70 ✅ exact | 63.49 / 30.60 | 64.78 / content 29.59 / visual 88.24 | **+1.29** (6-term: +1.23) | 85.41 → **88.24** |
| react/components/button | 71.60 / visual 89.97 / content 44.06 | 71.60 / 44.06 ✅ exact | 69.09 / 37.76 | 68.85 / content 35.90 / visual 90.81 | **-0.24** (6-term: -0.36) | 89.97 → **90.81** |
| react/components/meter | 69.91 / visual 93.38 / content 34.71 | 67.60 / 28.92 (stored report is 19:40, the 19:49 baseline run overwrote its own copy) | 65.95 / 24.79 | 65.80 / content 23.85 / visual 93.76 | **-0.15** (6-term: -0.21) | 93.38 → **93.76** |

So the like-for-like movement of the chrome work is +1.29 / -0.24 / -0.15 on the blended score, and
+2.83 / +0.84 / +0.38 on VISUAL PROXIMITY — the surface this item owns (its `done-when` is measured
on visual proximity). The two small negative deltas are not chrome defects: they are the recall
CORRECTION of removing the placeholder `Base UI Documentation` `<h1>` the old shell rendered on
every page (headings 0.90 → 0.80 on button, 0.50 → 0.44 on meter) plus live-upstream text-length
drift (upstream is re-rendered for every measurement: its `textLen` moved 13317 → 13675 between the
pre- and post-chrome checkbox runs, which alone costs ~0.8 of a textLen ratio).

What the rebake records is therefore the CURRENT formula's measurement of the CURRENT tree, with
the superseded numbers kept here so the change is auditable:

- superseded (6-term): checkbox 65.53 (85.41 / 35.70), button 71.60 (89.97 / 44.06), meter 69.91 (93.38 / 34.71)

A future iteration must not read the rebake as "fidelity dropped by 0.75 / 2.75 / 2.52": those
deltas are a metric change plus the loss of a heading that was never in upstream's page. The
comparison that matters is pre-chrome vs post-chrome under ONE formula, listed above.

## Tool defects fixed in the same iteration (the gate could not run at all without them)

Both were introduced by the same commit sequence that added the purity term, and both are in
`ralph/scripts/check-visual-budget.mjs`:

1. The per-route report line referenced a variable that does not exist in `main`'s scope
   (`(l.snippets||{})` — `l` is local to `scoreReport`), so EVERY run of the script crashed with
   `ReferenceError: l is not defined` before printing anything. Fixed to `measured.leptos.snippets`.
2. The baseline-record block read `measured.measuredBuild.bytes` before `measured.measuredBuild` was
   assigned (the assignment sat three lines BELOW the block), so `--update` and every first-time
   route recording crashed with `TypeError: Cannot read properties of undefined (reading 'bytes')`.
   `--all-done` hid it: with no improving route the record block never executed. Fixed by reading
   the served build before the record block.

## Caveat for the next iteration

The recorded baseline is only comparable across runs of the SAME `check-visual-budget.mjs`
formula, and the upstream half of every measurement is a live Next.js render whose probe counts
drift slightly between runs. When a scoring change lands (any edit to `scoreReport`'s
`recallParts`/weights), rebake the baseline in the same iteration and reproduce the old numbers
first, as done here — otherwise the gate fails every docs item against a metric that no longer
exists.
