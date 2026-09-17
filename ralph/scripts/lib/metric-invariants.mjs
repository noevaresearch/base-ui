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
    // WHAT THIS FIRES ON, and why it is not "does the report mention such a block": the rule is that a
    // React-to-React pair must never be ADMITTED TO SCORING. snippet-ergonomics.mjs already excludes such
    // blocks from every metric (`scoringTexts`, the language gate), and it REPORTS the count as its own P0
    // finding — so firing on mere presence made this invariant unsatisfiable by correct behaviour: a page
    // that excluded its React block and said so was flagged exactly like a page that scored the paste, and
    // the checker stayed red no matter what the loop did. A checker that cannot be satisfied by the correct
    // behaviour is the "checker that disagrees with its subject" this file exists to end (see the header).
    // The defect is the PUBLISHED NUMBER: `score` is the fidelity verdict every consumer reads, so a report
    // that carries a score while its own counters say a pair was React-to-React is the one that lies.
    why: 'React-to-React comparison is not fidelity evidence: pasting upstream would score high, so the measure ' +
         'would reward copying — a defect by the owner\'s rule. The violation is a report that PUBLISHED a score ' +
         'over such a pair: exclude the blocks AND withhold the number, or the page reads as a measured port.',
    violates: (r) => (r.metrics?.reactToReactBlocks ?? 0) > 0 && (r.score ?? null) !== null,
    detail: (r) => `reactToReactBlocks=${r.metrics?.reactToReactBlocks} yet score=${r.score}`,
  },
  {
    id: 'no-score-from-excluded-extraction',
    // The general form of the rule above, and the reason the two stale reports on disk were quotable at all:
    // a number is a CLAIM about a measurement, so if every block the extractor found was excluded from
    // scoring, the ratios behind that number were computed from an empty input — and every ratio in the
    // comparison defaults to 1 (100%) on empty input, which is how an empty measurement reads as a page with
    // a perfect API shape (`no-blocks-scored`'s finding, and the CI run recorded there). `blocksExcludedFromScoring >= leptosSnippets`
    // with a non-zero block count means NOTHING was admitted, so no size or shape verdict exists to publish.
    // snippet-ergonomics.mjs writes UNMEASURED (`null`) instead, which is the honest shape and the one the
    // consumers already handle (`check-page.mjs`: a null ratio never passes and never fails).
    why: 'a report that excluded EVERY extracted block from scoring has no size/shape measurement to publish, so ' +
         'a score beside it is arithmetic on an empty input — the degenerate case that let a page nobody could ' +
         'measure read as measured. Withhold the number; report UNMEASURED.',
    violates: (r) => (r.score ?? null) !== null && (r.leptosSnippets ?? 0) > 0 &&
                     (r.metrics?.blocksExcludedFromScoring ?? 0) >= (r.leptosSnippets ?? 0),
    detail: (r) => `blocksExcludedFromScoring=${r.metrics?.blocksExcludedFromScoring} of leptosSnippets=${r.leptosSnippets} yet score=${r.score}`,
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

// ---- the PRODUCER's side of the same two rules ---------------------------------------------------------
// `no-react-to-react-scoring` and `no-score-from-excluded-extraction` above say which reports may not carry a
// number. This function is the producer's half of exactly those rules — the reason `snippet-ergonomics.mjs`
// withholds `score` — and it lives here, beside them, for the reason this file's header gives: two copies of
// a rule drift apart, and a producer whose rule disagrees with its checker is worse than no checker at all.
// It is exported so `gate-selftest.mjs` can unit-test it without a browser (the producer itself needs one).
export function scoreWithheldReason({ reactToReactBlocks = 0, extractedBlocks = 0, scorableBlocks = 0 } = {}) {
  if (reactToReactBlocks > 0) {
    return `${reactToReactBlocks} of ${extractedBlocks} block(s) on this page are upstream's own React code (or ≥90% character-identical to it), so nothing here is a fidelity measurement of this port`;
  }
  if (extractedBlocks > 0 && scorableBlocks === 0) {
    return `every one of the ${extractedBlocks} block(s) extracted from this page was excluded from scoring, so every ratio in this score was computed from an empty input`;
  }
  return null;
}
