# Source-only mining (template) — for units with no tests

Parameterized by `{{unit}}` and `{{srcFiles}}` (non-test source files for that unit). Used ONLY
for units where `ralph/generated/components.json` records `hasTests: false` (measured
2026-09-07: `types`, `unstable-use-media-query` — each a single `index.ts` with no test file at
all). Every other unit goes through the normal two-stage split
(`stage1-behavior-mining.md` then `stage2-implementation-mining.md`); this template exists
because that split's whole premise — mine observable behavior FROM TESTS, mine WHY/HOW from
source second — has nothing to mine in its first half when there are no tests to read. Rather
than fabricate a `behavior.md` with invented test citations, this produces both specs in one pass,
directly from source, with behavior claims marked as source-derived instead of test-verified.
Filled in and dispatched per-unit as a bounded fan-out task, one subagent per unit — NOT a loop.

---

You are given `{{unit}}`, a unit with NO test files. Read every file in: {{srcFiles}}

Produce/update `specs/library/{{unit}}/behavior.md` with the same section structure Stage 1 uses:

## Public API surface (props, parts, subcomponents)
## State model (controlled/uncontrolled, defaults, transitions)
## Keyboard interactions
## Focus management
## Accessibility (roles, aria-*, id linking)
## DOM structure & portal behavior
## Events (names, payload shape, bubbling, preventDefault semantics)
## Edge cases (rapid interactions, unmount, nesting)
## Shared harness dependencies

Then produce `specs/library/{{unit}}/implementation.md` with the same section structure Stage 2
uses:

## State machine / hooks used
## Context providers/consumers
## DOM/portal strategy and why
## Dependencies on other Base UI internals
## Anything in source not explained by any test

RULES:
- Every claim in EITHER file must end with a citation to source, same exact backtick-wrapped
  `` `packages/react/src/{{unit}}/X.ts:123` `` form `check-citations.mjs` parses.
- Because there are no tests, every claim in `behavior.md` is inherently source-derived, not
  test-verified — do not write "UNVERIFIED" the way Stage 1 does for untested-but-inferrable
  behavior on a normally-tested unit; instead, add ONE line at the top of `behavior.md`:
  "This unit has no test files (`hasTests: false` in `ralph/generated/components.json`) — every
  claim below is derived directly from source, not confirmed by a test." Sections that don't
  apply should still say "N/A" rather than being omitted.
- `implementation.md`'s "Anything in source not explained by any test" section is trivially "N/A
  — this unit has no tests at all" here; don't skip the section, just say that.
- If this unit's `TODO.md` entry has a `wraps-external:` field, follow the same delegation rule
  Stage 1/2 use: do not derive the third-party library's own algorithm, just note the delegation
  and the named Rust-equivalent crate.
- Do NOT write any Rust or Leptos code. Do NOT touch `crates/`.
- After writing both specs, run
  `node ralph/scripts/check-citations.mjs record --scope specs/library/{{unit}}`
  to record citation baselines.
