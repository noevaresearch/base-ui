# CHECKS — the ordered registry

**What this file is.** Every check in this project, numbered in the order it should run, cheapest and most
fundamental first. The numbering IS the execution order. It exists because the checks had grown organically:
several were wired in one place and not another, a few could not fail, one was invoked by nothing at all, and
nobody could see the shape of the whole. Ordering them made the redundancy visible — which is the point.

**Reading the columns.**
`cost` — 🖥 browser (heavy: ~1.4 GB chromium) · 🧮 cpu (build/test) · ⚡ cheap (seconds, no browser)
`wired` — where it runs today: `reg` = run-regression.sh (per iteration) · `CI` = a GitHub workflow · `prompt` = the
loop is told about it · `watchdog` = the 15-minute cron · `—` = nothing runs it (a defect unless it is a library)
`sev` — `hard` (can fail the run) · `advisory` (prints; `|| true`) · `report` (informational by design)

| # | check | cost | wired | sev | why it exists |
|---|---|---|---|---|---|
| **1** | `env-health.mjs` | ⚡ | prompt, watchdog | report | Is the environment lying? Read before judging your own work: memory, browser allowance, both servers, scorecard freshness, instrument health, ledger deps. BROKEN ⇒ do not edit the port. |
| **2** | `memory-guard.sh` | ⚡ | watchdog | report | The box is a 4 GB cgroup; reaps browsers orphaned to PID 1 and refuses new work at ≥90%. |
| **3** | `check-todo-schema.mjs` | ⚡ | reg, prompt, CI | hard | The ledger is the plan; a malformed entry is unworkable. |
| **4** | `audit-instruments.mjs --strict-phantoms` | ⚡ | reg | hard | A `blocked-by` that resolves to no item parks its item forever. 33 items were parked by exactly that. |
| **5** | `gate-selftest.mjs` (runs `check-naming-parity.mjs`) | ⚡ | CI | report | Instruments tested like code: must-flag and must-pass fixtures, then the real reports. An invariant that cannot fail is not an invariant. |
| **6** | `scorecard-dedup.mjs` → `scorecard-latest.mjs` | ⚡ | CI / reg, prompt | hard | Measurement hygiene and the read path: one record per route, newest wins, and the loop reads CI's numbers instead of booting a browser. |
| **7** | `check-citations.mjs` | ⚡ | reg, prompt, CI | hard | Claims about behaviour must cite the spec that proves them. |
| **8** | `check-docs-contract.mjs` | ⚡ | reg, prompt | hard | The docs spec (`specs/docs-content/CONTRACT.md`) is what makes "parity" checkable. |
| **9** | `check-package-alias.mjs` | ⚡ | reg, prompt | hard | Install lines must name this port's package, and the bare specifier must resolve. |
| **10** | `check-react-mentions.mjs --source` | ⚡ | reg, prompt | hard | No React APIs in the port's own content (attribution is fine; React code is not). |
| **11** | `check-component-strict.mjs` | ⚡ | reg, prompt | hard | Parts, props, sections, hygiene, namespaced path — against the upstream behaviour spec. |
| **12** | `check-part-surface.mjs` | ⚡ | reg, prompt | hard | Every documented upstream part exists as `Component::Part`. |
| **13** | `check-unpassable.mjs` | ⚡ | reg, prompt | report | Names the case where an item CANNOT pass (a bar claimed by nobody) and gives the re-scope recipe. |
| **14** | `check-sandbox-parity.mjs` | ⚡ | reg | hard | The sandbox mirror must match what the docs show. |
| **15** | `cargo test --workspace` + wasm32 build | 🧮 | reg, CI | hard | The code compiles and its tests pass. |
| **16** | `check-page.mjs` (aggregates 17–20) | 🖥 | reg, prompt, CI | hard | One verdict per route across every axis; an unmeasured axis never passes. |
| **17** | `check-visual-budget.mjs` | 🖥 | reg, prompt | hard | Rendered layout against upstream's. |
| **18** | `snippet-ergonomics.mjs` | 🖥 | reg, prompt | hard | The port's example code must read as this library's (size, shape, naming, density). |
| **19** | `check-copy-fidelity.mjs` | 🖥 | reg, prompt | hard | Prose parity: the page must SAY what upstream says. |
| **20** | `check-react-mentions.mjs --all` | 🖥 | reg, prompt | hard | The rendered page must not ship upstream's React. |
| **21** | `build-status.mjs` + `build-status-rust.mjs` | ⚡ | CI, prompt | report | The `/status` page is generated from the gates' own output, never hand-written. |
| **22** | publish + deploy, with published-bytes smoke tests | 🧮 | CI | hard | What ships is what was verified: crates.io upload checked against the registry, site checked by fetching it. |

## Order rationale

1. **Environment (1–2) before everything.** Every failure tonight that looked like a port failure was the
   environment or an instrument lying. Two cheap seconds of "is the thing that will judge me working?" prevents hours.
2. **Ledger (3–4) before work.** An item that cannot be satisfied, or a plan entry that is malformed, wastes the
   whole iteration regardless of how good the code is.
3. **Instruments (5–6) before their output (16–20).** Untested rulers produced five false verdicts today.
4. **Cheap source checks (7–15) before browser checks (16–20).** Seconds before gigabytes: a failure that a grep can
   find must never be found by a browser.
5. **Rendered parity is 16–20 and runs in CI**, on a 16 GB runner, because this box cannot hold a browser and a
   build at once (measured: 51 oom_kill events).
6. **Delivery last** — never publish what the cheap and rendered checks have not already passed.

## Flagged for removal or merge (the reason this list was reordered)

| item | flag | evidence |
|---|---|---|
| `scorecard-sweep.sh` | **remove candidate** | superseded by `measure-port.yml`, which measures all 17 routes on six parallel runners every 2 h. Its cron is paused; the script is now dead weight that can disagree with CI. |
| `check-naming-parity.mjs` | **merge (done)** | it is an instrument self-test, not a codebase gate; it now runs inside check #5 instead of nowhere. |
| `build-status-rust.mjs` | **merge candidate** | two generators for one artefact (`status_data.rs`); could be one entry point (#21). |
| `memory-guard.sh` | **keep, but note** | correct that it lives in the watchdog (it gates NEW work); it must not move into the per-iteration regression. |
| `check-unpassable.mjs` | **keep, advisory** | diagnostic by design; it prints a recipe, and hard-failing would block work on a condition it exists to describe. |
| `check-sandbox-parity.mjs` | **verify** | wired hard in the regression but nothing references it in the prompt or CI; confirm it is catching real drift rather than sitting green. |

## The operating rule this file is part of

**The loop owns itself.** It runs, it fixes what it finds, and it reports through the repo. The agent's role on the
outside is **view access** — read state, run read-only checks, and report — plus **surgical edits to the plan**
(`TODO.md`, this file, `ralph/PLAN.md`) when a defect is found that the loop cannot see: an unsatisfiable dependency
(4), a mis-placed guard, a stale deploy trigger. Not restructuring the machinery. Anything else is the agent creating
work for itself in the loop's name — which is what most of one long night was.
