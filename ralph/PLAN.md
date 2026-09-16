# PLAN — closing the port's remaining gaps (written 2026-09-16, after 101/169 items)

This is the plan for the last stretch of the Base UI → Leptos port: what is left, in what order, and what
"done" is going to mean from here. It exists because the measurements now say something specific — and
because two failures this week were process failures, not engineering ones.

## 1. What the measurements actually say

| Axis | Where we are | Bar |
|---|---|---|
| Widget/component parity | 9 routes ≥97 (six at exactly 100), 5 unmeasured (demo-layout faults) | ≥97 |
| Page parity | best 87.5 (accordion), button 87.4, checkbox 85.9; rest 60–74 | ≥90 |
| Snippet language | 53 React defects across 21 page sources; 18 pages tell readers to install `@base-ui/react` | 0 |
| Snippet ergonomics | 8–31/100; five accordion blocks are 12 lines against upstream's 474; 0 namespaced parts anywhere | see §3 |
| Copy (prose) | 7 routes 100%, button 87%, checkbox 84.7%, avatar 43.2% | ≥95 |
| Part surface | 0/185 parts exposed as namespaced components across 31 components | 100% |

**The diagnosis that matters:** every gap above has the same root — the library's public surface. A page
cannot teach `<Accordion::Root>` if the port only offers `accordion_root_view(...)`, so a page that wants to
look finished either pastes upstream's React (what 53 defects are) or shows a hollow flattened stub (what
accordion's 12-line example is). Pages are downstream of the API.

A second diagnosis, from the accordion iteration: its `done-when` (`copy ≥95` + `react=0`) **passed on the
hollow stub**. Necessary conditions, not sufficient ones. Nothing may be marked done on a page whose example
teaches less than upstream's.

## 2. Lanes, in dependency order

**Lane 0 — measurement integrity (DONE, keep it that way).** Harness mutex; resource failures report
UNMEASURABLE rather than FAIL; React-to-React excluded from every metric and fails; one classifier
definition (`ralph/scripts/lib/snippet-lang.mjs`) instead of three drifting ones; the driver versioned in
`ralph/driver/` with post-conditions (revert tooling edits, land uncommitted content as a CHECKPOINT,
report residue). An uncommitted iteration can no longer pass as success — that is what let two harness
edits and 697 lines of accordion work sit unlanded.

**Lane 1 — the part surface (critical path).** Three batches already in the ledger (ported / menus /
inputs). For each component: `pub mod <Component> { pub fn Root/Indicator/… }` usable as `view!` markup,
plus a compile pin in the style of `crates/leptos-ui/tests/ns_component_path.rs`. Done-when:
`check-part-surface.mjs --components <batch> --strict` reports 0 missing **and** the pin compiles.
*Why first:* it unblocks namespaced snippet spelling, attribute density and the ergonomics score. Nothing
in Lane 2 can reach its bar before this exists.

**Lane 2 — page snippets, ONE page per iteration (18 pages).** Translate each page's examples to
`<Component::Part />` markup. Done-when per page: purity 1.0 (react 0, no react-to-react), **length
similarity ≥80%**, **attribute density ≥0.8× upstream**, namespaced spelling for ≥90% of component tags,
gap-report structure ≥80%. *Anti-rework rule:* a page's snippet work is blocked-by its component's surface
batch — translating twice (flattened now, namespaced later) is the rework this lane order prevents.

**Lane 3 — everything that does not depend on the surface (no rework risk, user-visible).** Install/alias
lines on the 18 pages (mechanical: `crates/docs-app/src/install_ref.rs` constants); React type columns
(`ReactElement` → the Rust type the port accepts); prose copy to ≥95% on the thin routes (avatar 43%,
button 87%, checkbox 85%, accordion 100%); mentions 0.

**Lane 4 — page-level visual parity to ≥90.** Sidebar/typography/API tables/code chrome/demo layout, driven
by `visual-gap-report.mjs` (it already names each gap with a severity and a fix). Five routes have an
unmeasured widget region — those demo-layout faults are part of this lane, and an unmeasured axis is not a
pass.

**Lane 5 — the 15 extra mirrored pages** (handbook/forms/utilities) that gate nothing yet but are visible on
the live site.

**Lane 6 — final verification on the DEPLOYED site.** All gates run `--all` against
`https://baseui.noevaresearch.com`, not localhost: a page is not done because a local build is green.
Publication of the crate/npm package stays the user's decision (the alias is `private: true`).

## 3. "Done" is a page scorecard, not a checklist of axes

Build `ralph/scripts/check-page.mjs --route <route> [--strict]`: run every axis for one route and print a
single verdict (structure, page parity, widget parity, snippet purity, snippet ergonomics axes, copy
coverage, React mentions, install alias). Anything unmeasured is reported as UNMEASURED, never passed. Then:

* the ledger's page items reference the scorecard in their `done-when` instead of naming two conditions;
* progress is reported as **pages passing the scorecard**, because that is the number that answers "is the
  site good" — item counts cannot (101/169 with a hollow example counted as progress is exactly the
  failure mode to avoid);
* a scheduled all-routes sweep (idle-only, mutex-serialised, ~1 route per invocation to stay inside the
  cgroup budget) writes `ralph/logs/scorecard.md` and posts a digest, so drift is visible without a human
  asking.

## 4. Sequencing summary

```
Lane 0  integrity      ✓ done
Lane 1  part surface   → 3 batches (ported → menus → inputs)      [critical path]
Lane 2  page snippets  → 18 pages, one per iteration, after their batch
Lane 3  alias/prose    → 18 pages + 3 thin-copy pages, in parallel with Lane 1/2
Lane 4  page parity    → gap reports, 5 unmeasured widget regions
Lane 5  extra pages    → 15
Lane 6  live-site sweep → all axes, deployed URL, then publish decision
```

## 5. Risks and what is already done about them

| Risk | Mitigation |
|---|---|
| Mega-items stall on the turn ceiling | bounded items (one page / one surface batch); part surface split into 3 |
| A gate that silently never runs | `routes:` field verified end-to-end — an empty route made a hard gate report green |
| Copying upstream scoring as parity | language gate: excluded from every metric + P0 + fail |
| A false FAIL blocking complete work | resource failures are UNMEASURABLE, not FAIL |
| An iteration leaving unlanded work | driver post-conditions: checkpoint content, revert tooling, report residue |
| Self-graded tooling edits | driver reverts harness changes unless the item is a tooling item |
| A page marked done on a hollow example | scorecard in §3, with length and attribute density as required axes |
| Box starvation killing the loop (and the watchdog) | mutex + janitor + preflight guard; never run a manual harness sweep alongside the loop |
