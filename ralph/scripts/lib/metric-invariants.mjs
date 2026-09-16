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
    violates: (r) => (r.leptosSnippets ?? 0) > 0 && (r.metrics?.leptosLines ?? 0) === 0,
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
