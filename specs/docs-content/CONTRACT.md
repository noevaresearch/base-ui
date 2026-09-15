# Mirrored docs pages — snippet & behaviour contract

Status: binding convention for every `docs-content: components/<name>` and `docs-content: utils/<name>`
item. Written 2026-09-15 after the checkbox page was found carrying five code blocks of **upstream's
React source** (JSX and `import { Checkbox } from '@base-ui/react/checkbox'`) and **zero** Leptos
snippets, while every structural check passed and the fidelity score was *inflated* by the
transcribed text.

## Why this exists

A mirrored page has two distinct obligations, and the existing gates only watched one of them:

1. **Structure** — the headings, sections and demo placement match upstream. `playwright-diff.mjs`
   and `check-visual-budget.mjs` watch this.
2. **Behaviour** — the page *teaches the port*: every example shows the Leptos API, and every demo
   behaves the way the upstream demo behaved. Nothing watched this, and the failure is silent:
   transcription of upstream's source produces a page that looks right, quotes the wrong framework,
   and demonstrates behaviour nobody ever re-checked.

So behaviour fidelity is a **spec obligation**, not a script's opinion. Scripts corroborate it
(`visual-gap-report.mjs` classifies snippet language; `check-visual-budget.mjs` scores snippet
language as purity), but the definition of done lives here.

## Requirement 1 — snippets demonstrate the port

Every code block embedded in a mirrored page must be expressed against **this port's** API:

* imports/uses of `leptos_ui` / `leptos-ui-internals` parts, e.g. `use leptos::prelude::*;` and
  `leptos_ui::checkbox::{root, indicator}` style composition, not `@base-ui/react/*`;
* markup in `view!` syntax over the port's view functions, not JSX;
* props/state shown as the port expresses them (`Signal<T>`, `RwSignal`, `cx(...)`, callbacks),
  not React signatures;
* a snippet may legitimately be language-neutral (a shell command, a file tree, a CSS rule) —
  those are `other` and are fine. **Any block that shows upstream's runtime source is a defect.**

Corroboration: `node ralph/scripts/visual-gap-report.mjs --route <route>` reports
`snippets: {total, leptos, react, other}` and raises a P0 when `react > 0`.

## Requirement 2 — demos reproduce upstream behaviour

Each mirrored page's demos must reproduce the behaviour of the upstream demo they mirror — not just
its markup. Behaviour is already mined and test-proven elsewhere in the repo; the contract cites it
rather than restating it:

* `specs/library/<component>/behavior.md` — the test-mined behaviour sections (Public API surface,
  State model, Keyboard interactions, Focus management, Accessibility, DOM structure, Events, Edge
  cases, Shared harness dependencies).
* `specs/docs-content/<component>/demos.json` — the per-demo source citations for this page.

The page spec must name, per demo, the **observable** that proves the behaviour (e.g. "a real click
funnels through the hidden input's change event and flips `aria-checked` plus the input property" —
not "the checkbox works").

## Requirement 3 — the contract table (format)

Each mirrored page's spec carries a section, `## Snippet & behaviour contract`, with a table of one
row per example the page teaches:

| example (upstream citation) | Leptos snippet to show | behavioural obligations (cited) | observable that proves it |
| --- | --- | --- | --- |
| `docs/src/app/(docs)/react/components/checkbox/page.mdx:19-27` (Anatomy) | import + assemble the port's parts | `specs/library/checkbox/behavior.md` § Public API surface (only Root + Indicator are public) | rendered tree contains the port's root/indicator elements |

Rules for the table:

* cite the upstream line range for the example, and the behaviour spec section for each obligation
  — never paraphrase the behaviour spec without a citation (the citation checker enforces that
  specs stay stable against upstream);
* include an explicit row for the hero demo with its interaction observable;
* if an obligation cannot be proved by an observable in this port yet, say so explicitly and log it
  to `ralph/logs/spec-discrepancies.md` — do not write a weaker claim to make the row look filled.

## Requirement 4 — gaps go to the discrepancy log

Where upstream's behaviour is not yet reproduced, or the spec's claim is wrong or incomplete, the
finding goes to `ralph/logs/spec-discrepancies.md` (one entry per finding, with the upstream
citation that contradicts it). The page's `done-when` in `TODO.md` must not be marked satisfied
over an open discrepancy.

## Requirement 5 — this contract is part of done

A `docs-content:` item is not done unless its spec carries the contract table, its snippets are all
`react: 0` per the gap report probe, its demo observables were exercised, and snippet-language
purity in `check-visual-budget.mjs` is 1.0 (or the page has no snippets at all). The Phase E item
`docs-spec: snippet & behaviour contract on every mirrored page` is the ledger's home for bringing
already-mirrored pages up to this contract.

Lint: `node ralph/scripts/check-docs-contract.mjs` lists pages missing the contract section
(`--strict` exits 1). It is deliberately advisory in `run-regression.sh`: the sections must be
authored deliberately, so the gate reports rather than blocks.
