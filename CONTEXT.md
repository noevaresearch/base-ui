# Ralph Port: Orientation

You are one stateless iteration of a Ralph loop porting Base UI (React) to Leptos. You have no
memory of prior iterations — everything you need is on disk. Read in this order:

1. This file.
2. `TODO.md` — pick (or resume) exactly one unchecked, unblocked item.
3. Every file under `specs:` for that item, plus `specs/architecture.md`.
4. Only then, if a spec's citation is ambiguous, follow it into the original file it names —
   the spec is the interface; the citation is the drill for when the spec alone isn't enough.

## The objective

The Leptos port of a component is not "done" when its Rust code compiles and its own unit tests
pass. It is done when **the docs site itself renders that component's docs page, with its real
demos, using the ported `leptos-ui` crate** — see each Phase B/D pairing in `TODO.md`. A component
whose Phase B item is checked off but whose paired Phase D item isn't is not finished.

## Rules for every iteration

- Do exactly one `TODO.md` item. Do not start a second one in the same iteration.
- Never mark an item `done` without running the full verification gate for it
  (`ralph/scripts/run-regression.sh <todo-id>`), including `cargo test --workspace`, not just the
  crate you touched — a passing local test for your item does not mean you haven't broken
  something else.
- Never edit `specs/**` to make a citation agree with your implementation. If a spec looks wrong,
  append a note to `ralph/logs/spec-discrepancies.md` instead — that's for the backward-looking
  audit loop to resolve, not for you to silently "fix."
- You may not create a fabricated docs page or stub demo just to satisfy the Phase D pairing rule
  — that defeats the entire point of the objective above.
- **Release infrastructure needs the owner's authorisation — an iteration does not grant it to
  itself.** `release/**` and `.github/workflows/**` publish to public registries, where a version is
  permanent (crates.io allows yank, never delete). Two legitimate shapes:
  * **Report only** (the default): append the concern to `ralph/logs/spec-discrepancies.md`, or open
    a `TODO.md` item. Never edit those paths, never cancel a run in flight — and never summarise
    their state from a single artifact (see below).
  * **Act, with authorisation**: if the owner has explicitly told you to change a release path, do it
    as a NAMED COMMIT that quotes their instruction verbatim, so the authorisation is auditable.
    Both the disarm and the re-arm on 2026-09-16 were done this way, and the driver records such
    commits as a measurement review rather than reverting them.
  The driver reverts *uncommitted* edits to these paths (like the harness scripts, because a release
  workflow is an instrument too), so an authorised change must be committed to survive — and an
  unauthorised one should not.
- **When you check whether something shipped, check EVERY artifact, not the first one.** A partial
  publication looks exactly like no publication if you query one name: on 2026-09-16 an iteration
  checked `base-ui-leptos` alone (404), wrote "nothing is published" into a commit message, and
  disarmed the pipeline — while two of the three crates were already LIVE at 0.1.1. The cancel
  accompanying that disarm cut a release in half and nobody noticed for an hour. Use
  `node release/release-state.mjs`, which prints per-crate state for exactly this reason.

## Directory map

- `specs/` — the durable knowledge base (behavior + implementation + fixtures, all cited).
- `crates/` — the Rust/Leptos workspace (does not exist until build-order step 10 — see
  `Ralph-BaseUI-Leptos-Methodology.md` and the approved plan for why).
- `ralph/` — all loop tooling: `prompts/` (stage templates), `scripts/` (enumeration, citation
  checker, regression runner, audit tooling), `generated/` (enumeration manifests, regenerable).
- `packages/react/`, `packages/utils/`, `docs/` — the original, untouched React source and docs
  site. Never modify these; they're the oracle every citation and Playwright differential test
  points back into.
