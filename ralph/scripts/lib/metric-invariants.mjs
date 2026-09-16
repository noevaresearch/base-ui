// metric-invariants.mjs — states that must be IMPOSSIBLE in a trustworthy report, in one place.
//
// Both consumers import this: gate-selftest.mjs (does the checker work?) and check-page.mjs (may this number be
// quoted as a page verdict?). Two copies of an invariant drift apart, and a checker that disagrees with its
// subject is worse than none — that is the failure class this file exists to end.

export const INVARIANTS = [
  {
    id: 'snippets-scored-not-dropped',
    why: 'a report that found N>=1 Leptos snippets but counted 0 lines has dropped OUR blocks from scoring, so ' +
         'every size-derived axis (length, attribute density, tree shape, props) reads 0 BY CONSTRUCTION and the ' +
         'page can never pass no matter what the loop does.',
    violates: (r) => (r.leptosSnippets ?? 0) > 0 && (r.metrics?.leptosLines ?? null) === 0,
    // `?? null`, not `?? 0`: a report whose leptosLines is NULL is a deliberate refusal by the gate
    // ("blocks found but none scorable" — snippet-ergonomics.mjs:449-452), not a dropped count. Reading
    // null as 0 fired this invariant on `react/utils/merge-props` and `react/utils/use-render` — the two
    // shapes look identical to a `?? 0` and opposite to a reader. MEASURED 2026-09-16, gate-selftest.
    detail: (r) => `leptosSnippets=${r.leptosSnippets} but leptosLines=${r.metrics?.leptosLines}`,
  },
  {
    id: 'no-react-to-react-scoring',
    why: 'React-to-React comparison is not fidelity evidence: pasting upstream would score high, so the measure ' +
         'would reward copying — a defect by the owner\'s rule.',
    violates: (r) => (r.metrics?.reactToReactBlocks ?? 0) > 0,
    detail: (r) => `reactToReactBlocks=${r.metrics?.reactToReactBlocks}`,
  },
  {
    id: 'length-similarity-consistency',
    why: 'if lines were counted on both sides, similarity must be non-zero. A 0 means the ratio was taken ' +
         'against an empty side: a broken measurement, not a failed page.',
    violates: (r) => (r.metrics?.upstreamLines ?? 0) > 0 && (r.metrics?.leptosLines ?? 0) > 0 && (r.metrics?.lengthSimilarity ?? 0) === 0,
    detail: (r) => `upstreamLines=${r.metrics?.upstreamLines} leptosLines=${r.metrics?.leptosLines} lengthSimilarity=${r.metrics?.lengthSimilarity}`,
  },
  {
    id: 'upstream-side-present',
    why: 'a report with no upstream snippets cannot measure parity at all; it must report UNMEASURABLE rather ' +
         'than a score, or the loop chases a page whose reference was never loaded.',
    violates: (r) => (r.upstreamSnippets ?? 0) === 0 && (r.score ?? 0) > 0,
    detail: (r) => `upstreamSnippets=${r.upstreamSnippets} yet score=${r.score}`,
  },
  {
    id: 'no-blocks-scored',
    // MEASURED 2026-09-16 on the CI runner (run 35136883087): every route's report carried
    // `upstreamSnippets 0, leptosSnippets 0, snippetLanguages.total 0` AND a score of 45, because every
    // ratio in the comparison defaults to 1 on empty input — naming 100, props 100, brevity 100,
    // namespaceStyle 100. An empty measurement therefore read as a page with a perfect API shape, and
    // `snippet language` reported PASS (`react = 0`) on a page from which NOTHING had been extracted.
    why: 'a report that extracted ZERO blocks from this port\'s own page cannot score shape, naming, ' +
         'attribute density or language: the empty-input defaults are all 1 (100%), so the score is an ' +
         'artefact of arithmetic on nothing, and a `react = 0` PASS on a page with no blocks is a false ' +
         'green rather than a language verdict.',
    violates: (r) => (r.leptosSnippets ?? 0) === 0 && (r.score ?? 0) > 0,
    detail: (r) => `leptosSnippets=${r.leptosSnippets} yet score=${r.score} (every ratio defaults to 100% on empty input)`,
  },
  {
    id: 'report-not-refused',
    // A gate that could not take a measurement must not leave a file that a consumer can score: a stale
    // report (from a previous run, or from this box's own last good sweep) read on the runner is how a
    // local green becomes a CI number. Refusals are written EXPLICITLY so the report can never be
    // mistaken for a measurement.
    why: 'a report marked as a refusal carries no measurement, and any consumer that scores it would be ' +
         'reporting a number nobody took.',
    violates: (r) => r.refused === true && (r.score ?? null) !== null,
    detail: (r) => `refused=true but score=${r.score}`,
  },
];

export function evaluate(report) {
  const fired = [];
  for (const inv of INVARIANTS) {
    let bad = false;
    try { bad = !!inv.violates(report); } catch { bad = false; }
    if (bad) fired.push(inv);
  }
  return fired;
}
