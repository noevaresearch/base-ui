# Stage 2 (docs): Demo mining (template)

Parameterized by `{{unit}}` (component/util name) and `{{demos}}` (JSON array of
`{name, dir, tailwind, cssModules, isHero}` from `ralph/generated/docs-content.json`'s entry for
this unit's route). Run after Stage 1 (docs) has produced `specs/docs-content/{{unit}}/page.md`
for the same unit. Filled in and dispatched per-unit as a bounded fan-out task, one subagent per
unit — NOT a loop. Mirrors `ralph/prompts/stage2-implementation-mining.md`'s framing, scoped to
docs demos instead of component source.

---

You are given `{{unit}}`'s docs page spec at `specs/docs-content/{{unit}}/page.md` (already
written) and its demo variants: {{demos}}.

For EACH demo, read both its `tailwind` and `cssModules` variant files (they should be
behaviorally identical — implementing the same demo against the same headless component with two
different styling approaches; note under "Discrepancies" if they diverge in anything other than
styling).

Your job is to produce a machine-readable manifest, not prose — `docs-app`'s Stage 3
implementation reads this mechanically to know what each demo must reproduce, so keep it in the
exact JSON shape below rather than the section-heading Markdown format Stage 1 uses.

Produce `specs/docs-content/{{unit}}/demos.json`:

```json
[
  {
    "name": "<demo name>",
    "isHero": true,
    "componentPartsUsed": ["Accordion.Root", "Accordion.Item", "..."],
    "propsExercised": { "Accordion.Root": ["defaultValue", "onValueChange"], "...": [] },
    "whatItDemonstrates": "one sentence, plain language",
    "stateManaged": "none | uncontrolled | controlled — and how (e.g. useState wrapping value)",
    "nonTrivialInteractions": ["e.g. keyboard nav between items", "..."],
    "citations": ["`docs/src/app/(docs)/react/components/{{unit}}/demos/<name>/tailwind/index.tsx:12-30`"]
  }
]
```

RULES:
- One array entry per demo listed in {{demos}}; do not add or drop demos.
- `citations` entries MUST be backtick-wrapped INSIDE the JSON string, exactly as shown above
  (backticks are ordinary characters in a JSON string, so this is valid JSON) — `check-citations.mjs`
  also validates `*.json` spec files, using the same backtick-delimited `` `path:line` `` pattern
  it uses for `.md` specs; an unwrapped citation string will silently not be checked. Cite the
  `tailwind` variant file by convention (both variants are expected to match; only cite
  `cssModules` additionally if this demo is one of the divergent cases you flagged).
- Do not propose Rust/Leptos component APIs or `docs-app` architecture here — that belongs to
  `specs/architecture.md`, a separate later synthesis step, out of scope for this task.
- After writing the spec, run
  `node ralph/scripts/check-citations.mjs record --scope specs/docs-content/{{unit}}`
  to record citation baselines for the new citations you added.
