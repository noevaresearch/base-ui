# Stage 3: forward loop (per-iteration prompt template)

Rendered by `ralph/scripts/classralph.sh` with `{{todo-id}}` filled in. One invocation of this
prompt = one Ralph iteration = one bounded objective (Principle 2). You have no memory of any
other iteration.

`{{todo-id}}` is either a real item id — what `ralph/scripts/pick-next-todo.mjs` (deterministic,
dependency-aware) suggests as the next item — or the literal string `NONE`, meaning the mechanical
picker found no `not-started` item whose `blocked-by` deps are all `done`.

Either way, **treat it as a suggestion, not a mandate.** The mechanical picker exists so that most
iterations don't have to re-derive phase ordering and the docs-pairing rule from scratch. Read
`TODO.md` and the specs yourself and pick the highest-priority task you can think of — it will
usually be the suggestion, but not always. Do Step 0 below before committing to any item.

---

## Step 0: confirm or override the suggested item

Read `CONTEXT.md`, then all of `TODO.md`.

**If `{{todo-id}}` is a real item id:** read its entry and the specs it cites. Sanity-check it:
is it genuinely `not-started`, are its `blocked-by` deps actually `done`, and is there not some
more important unblocked item the picker's simpler rules missed? If it holds up, proceed with it
in "Do the work" below — you do not need to write anything about this check, just proceed.

**Also check, regardless of whether `{{todo-id}}` is a real id or `NONE`:** is the `blocked-by`
graph itself wrong or overly conservative? `generate-todo.mjs` deliberately used a coarse default
(`blocked-by: [Phase A complete]` on every Phase B item, meaning "every Phase A item," even though
most components only depend on a handful of specific utils/infra crates). If some Phase B item's
`specs/library/<name>/implementation.md` (once it exists) already documents its real dependencies
under "Dependencies on other Base UI internals," and those specific dependencies are all `done`
even though *other*, unrelated Phase A items aren't, that item is safe to work on despite the
mechanical block — and may be more important than `{{todo-id}}`.

**Also scan for any `status: blocked` item.** This loop never halts on a failure — a prior
iteration's regression failure or false start becomes a `blocked` item with a one-line reason,
committed as real state, precisely so a *later* iteration (you, right now) can pick it up rather
than it silently rotting. If a `blocked` item's recorded reason looks resolved by work already
`done` since — or if the reason itself was wrong — that item may be the actual highest-priority
task, above `{{todo-id}}`: fixing something broken usually outweighs starting something new. Read
its blocked-note, verify the blocker's real status yourself (don't just trust the note), and if
it's genuinely resolved, treat it as picked (set `status:` back to `not-started` or straight to
`done` as appropriate, and note in your commit why the block no longer applies) instead of
starting fresh work elsewhere.

**If you conclude a different item than `{{todo-id}}` is the right one to work on** (whether
because `{{todo-id}}` is `NONE`, or because you're overriding it for a documented reason):
1. Pick exactly ONE such item. Treat its id as `{{todo-id}}` for every step below.
2. Before implementing anything, edit its `TODO.md` entry: replace `blocked-by: [Phase A
   complete]` with the precise list of items it actually depends on (cite the
   implementation.md section that proves this in a trailing comment on the line) — or, if you're
   overriding a real suggestion rather than working around a `NONE`, add a one-line note
   explaining why this item takes priority. Never simply delete or blank a `blocked-by` field —
   narrowing it must be justified and visible, and never rewrite a spec to make your choice look
   more justified than it is.
3. In your commit message, state explicitly that this item was chosen over the mechanical
   suggestion (or in place of a `NONE`) and why, so a human reviewing `git log` can see this
   wasn't the default path.
4. Proceed to "Do the work" below.

**If `{{todo-id}}` was `NONE` and you find no case (c) opportunity either,** determine which
remaining case applies and stop — do not proceed to "Do the work":

- **The port is actually complete.** Every item is `status: done`. Do nothing, change nothing,
  and report that clearly — do not invent work.
- **Everything remaining is genuinely, correctly blocked.** E.g. all Phase B items are waiting on
  a Phase A item that is legitimately still `not-started`. Do nothing, change nothing, and report
  which item(s) are the actual bottleneck, so a human knows what to prioritize.

Do not use any of this to route around the docs-pairing rule itself (`docs-pair:` /
`check-todo-schema.mjs`) — that rule is not a scheduling artifact, it's the actual definition of
done, and no override of item selection ever touches it.

---

## Do the work

You are one stateless iteration of a Ralph port loop, porting Base UI (React) to Leptos. You have
NO memory of prior iterations beyond what is committed to git and written in `TODO.md`/`specs/`.

1. Your item is whichever id Step 0 above left you with (either `{{todo-id}}` as suggested, or the
   one you selected instead). Do not start a second item after finishing this one.

2. Read every file listed under that item's `specs:` field, and `specs/architecture.md`, if you
   have not already done so in Step 0.

3. Follow every citation in those specs into the ORIGINAL file/line it names, to confirm it still
   says what the spec claims. You are also a citation check, in addition to the automated one
   (`ralph/scripts/check-citations.mjs`) — do not trust the spec's paraphrase blindly.

4. Implement ONLY this item, in the crate named in its `crate:` field. Do not touch any other
   crate. Do not touch `specs/**` except to append a note to `ralph/logs/spec-discrepancies.md`
   if you find a spec is wrong or incomplete relative to upstream — never silently rewrite a spec
   to agree with your implementation, and never delete or contradict an existing citation.

   Budget your time roughly 80% porting the actual behavior, 20% writing tests for it. This is a
   forward-porting loop, not a test-authoring loop — the goal is faithful behavior parity with
   upstream, verified by a proportionate amount of new test code, not maximal coverage. If you
   notice you're spending most of an iteration expanding test cases rather than translating
   behavior, that's a signal to stop adding tests and finish the port.

   You may make intermediate commits as you reach a safe checkpoint within this item (a state
   where the crate you're working in — and anything that depends on it — still compiles), so the
   audit trail shows real progress rather than one large diff. This is not "commit after every
   edit" — only after checkpoints that leave the workspace compiling. These intermediate commits
   do NOT mark the item done and are not a substitute for step 8: `status:` stays whatever it was
   (not `done`) until the full regression in step 7 passes.

5. Verify locally: `cargo check -p <crate>`, `cargo test -p <crate>`.

6. If this item's `done-when` references docs-app rendering (it has a `docs-pair:` field, or its
   `done-when` mentions docs-app), also build/run `crates/docs-app` against your change and
   confirm the relevant route renders without panicking, and — if
   `ralph/scripts/playwright-diff.mjs` exists yet — passes its differential check against the
   original React docs page for the same demo.

6b. **Visual fidelity (Phase E items — anything under `docs-chrome:` or `docs-fidelity:`, and
   any docs page you touched).** Mounting is not parity: a page can render perfectly and still
   look nothing like upstream. Two tools exist, and they are a loop, not a verdict:

   * `node ralph/scripts/visual-gap-report.mjs --todo-id "<your item's id>"` — **read this
     before implementing, and again after.** It renders your route on both apps and writes
     `ralph/logs/visual/<component>.md` listing the *named* gaps, each with a suggested fix and a
     severity: missing sidebar/header, zero coloured code tokens, missing demo file tabs,
     API-props-as-prose instead of tables, typography and column-width drift. It also writes
     `<component>-mask.png` (red = differs from upstream) and `<component>-overlay.png` — look at
     them if you can; they show *where* the page diverges. Fix the highest-severity gap first.
     If the report says the route rendered shell-only, fix the mount before any styling work: the
     fidelity numbers are meaningless until the page actually renders.
   * `node ralph/scripts/check-part-surface.mjs --strict` — the repo-wide ergonomic gate: it walks
     every mined spec (specs/library/**) for the `Component.Part` names upstream documents and asks
     whether the crate exposes `Component::Part`. Use it when you work
     `library: namespaced part surface (Checkbox::Root form)` or anything under `docs-ergonomics:` —
     a part that exists only as `<component>_<part>_view(..)` is behaviour without the ergonomics, and
     this is the number that must reach 100%.
   * `node ralph/scripts/snippet-ergonomics.mjs --route <route>` — **does your example code READ
     like upstream's?** A snippet can be Leptos and still be ergonomically alien: upstream teaches
     `<Checkbox.Root><Checkbox.Indicator /></Checkbox.Root>`, so the port's examples must use the same
     shape — namespaced components usable as view! markup (the `<ui::Button />` idiom), props as
     attributes, and a size class within 20% of upstream (the `--length-floor 0.8` bar; both lines and
     characters are compared). It reports AST shape/naming/depth via tree-sitter plus the raw-call
     smells (`*_view(...)`, `…Props { .. }`) that never appear upstream, and writes
     `ralph/logs/visual/<component>-snippets.md` with a named fix per gap. If the parts have no
     same-shaped public surface yet, that surface is the work — not a workaround in the snippet.
   * `node ralph/scripts/check-visual-budget.mjs --route <route> --update` — two numbers, and they
     are held to different bars. **Component widget parity** (bar 97%): the demo's own rendered
     control+label, cropped per side to a COMMON rect and compared — the component must look the same
     even though the framework differs, so a widget below 97 is a real fidelity defect in the
     component's own markup. **Page score** (bar 90): the blended page number, deliberately looser
     because mirrored prose and code are Leptos and *should* differ from upstream's React. Both print
     on every run; the -widget.png crops in `ralph/logs/visual/` show what to fix. The score
     (0.6 x pixel proximity + 0.4 x content recall, best-known per route in
     `ralph/generated/visual-baseline.json`). Run it with `--update` **only when the score went
     up**; it records your improvement so later iterations must beat it. Never use `--update` to
     paper over a drop — a regression is a fact about your change, not a number to reset.
     By default this is a REGRESSION gate: it fails on a drop against the recorded page score or
     widget parity, and on losing the measurability of a region that was measurable. The ABSOLUTE
     bars are enforced when you pass them — `--target 90 --target-component 97` is what the Phase E
     parity item is measured with — while a below-bar widget is reported on every run (and raised as
     a P0 by the gap report) either way. A component region the two sides cannot be compared on is a
     reported FAULT, never a scored number, and it is fatal when the bar is being enforced.

   Work a gap, re-run the score, and put both numbers (before -> after, and which gaps closed) in
   your commit message — that is the loop improving itself instead of guessing at "looks better".

6c. **Spec-level feedback: the mirrored-page contract (specs/docs-content/CONTRACT.md).** Both tool
   checks above watch a page's *shape* and *looks*; neither can see that a page teaches the wrong
   framework — the checkbox page passed every structural gate while all five of its code blocks
   held upstream's React source. That obligation is spec-level, so:

   * before mirroring or repairing a docs page, read that page's spec for its
     `## Snippet & behaviour contract` table: it names, per example, the Leptos snippet you must
     show, the behavioural obligations (cited to `specs/library/<component>/behavior.md`) and the
     **observable** that proves each one. Snippets show the port's own API (`leptos_ui` parts in
     `view!` syntax) — never upstream's JSX; a demo must reproduce upstream's *behaviour*, not just
     its markup.
   * if the page's spec has no such table, that is a spec gap, not something to improvise: append a
     finding to `ralph/logs/spec-discrepancies.md` and pick the Phase E item
     `docs-spec: snippet & behaviour contract on every mirrored page` (or the page's own item) rather
     than inventing a contract mid-implementation.
   * **exception to the specs-are-read-only rule:** when the item you picked is itself a
     `docs-spec:` item, authoring/repairing the spec files *is* the work — edit
     `specs/docs-content/<name>/page.md` directly (still never deleting or contradicting an existing
     citation, and still logging contradictions to the discrepancy log).
   * `node ralph/scripts/check-docs-contract.mjs` lists which mirrored pages lack a contract
     (`--strict` exits 1) — that list is your queue when you pick a docs-spec item.

6c. **Website copy — the prose, which nothing measured until now.** A docs page is mostly sentences.
   The existing content-recall term only counted characters, so a page could score 85 with half its
   prose missing, and missing/paraphrased/invented sentences all moved that number in unhelpful
   directions. `node ralph/scripts/check-copy-fidelity.mjs --route <route>` renders both apps, takes
   the prose blocks (code excluded, navigation excluded — only leaf prose and table cells), and reports
   coverage with the `missing` and `changed` blocks named so you can read upstream's sentence next to
   yours. Measured 2026-09-16 it is not uniform: field, fieldset, form, meter, checkbox-group and
   collapsible are at 100%; button 87%; checkbox 84.7% (its gaps are upstream's reference-section
   headings like `Checkbox.Root.Props`/`Checkbox.Root.State` and its `Re-Export of Root props as
   CheckboxRootProps` notes — sections this port does not render at all); avatar 43.2%; accordion 22.2%
   (45 of upstream's 63 blocks absent — that is a page to finish, not a snippet to translate).
   If it prints `UNMEASURABLE`, the render did not finish: it is refusing to score, not scoring zero —
   re-run it rather than believing the number. When your item's done-when names copy coverage, that
   number is part of done.

6d. **The port's own name — no React in the generated code.** Crediting the original work is allowed;
   shipping upstream's framework inside the port is not. `node ralph/scripts/check-react-mentions.mjs
   --route <route>` (rendered) and `--source` (this port's own page content) enforce it:
   * the package a reader is told to install is **`@noevaresearch/base-ui`** — mapped locally at
     `packages/leptos/` (`private: true`, NOT published; its README says publication is a separate
     reviewed step). `@base-ui/react`, `from 'react'`, an npmjs/react.dev link, or an `import { X } from
     '@base-ui/react/<part>'` snippet is a DEFECT. Do not invent an install command that would 404 —
     describe what exists today (the crate path `crates/leptos-ui` plus the alias).
   * React APIs in prose or API tables are DEFECTS: `useState`, `useRef`, `React.memo`, `forwardRef`,
     `React.*`, `HTMLProps`, `props.children`, `JSX`. A type column that says `ReactElement` must say the
     Rust type the port accepts (`Callback`, `Children`, `Rc<dyn Fn...>`, …).
   * a credited reference ("ported from the React implementation", "upstream", "based on") is allowed and
     is counted as `attribution`, never failed. Any other bare "React" is a warn: reword it to Leptos, or
     list it in `specs/docs-content/<name>/react-allow.json` with a reason.
   Measured 2026-09-16: the port's own source has 53 defects across 21 files (worst: `ReactElement` in the
   API type columns of checkbox/button, React import strings in `code_block.rs`), and rendered checkbox
   shows 13 defects with **0 mentions of `@noevaresearch/base-ui`** — the alias is used nowhere yet.
   The install line a page shows comes from `crates/docs-app/src/install_ref.rs` (`INSTALL_SNIPPET`,
   `PACKAGE_ALIAS`, `RUST_CRATE`, `PROVENANCE`): render those constants instead of writing your own text,
   and `node ralph/scripts/check-package-alias.mjs` verifies the whole chain — manifest identity ⇄
   install_ref agreement ⇄ the bare specifier actually resolving from the repo root and from
   `test/node-resolution` ⇄ no page naming upstream's package. It currently fails on **18 page sources**,
   each telling the reader to install `@base-ui/react`. Owning item:
   `docs-copy: Leptos-only mentions + the @noevaresearch/base-ui alias`.

7. Run the FULL workspace regression:
   `bash ralph/scripts/run-regression.sh "<your item's id>"`
   This runs the citation check, `cargo test --workspace` (not just your crate), TODO schema
   validation, and the docs-app check from step 6 if applicable. If it fails for ANY reason —
   including a failure in a crate you didn't touch — you have regressed prior work or the item
   isn't actually done yet. Do not check the item off. Set its `status:` to `blocked` in
   `TODO.md`, with a one-line note of what failed and which command reported it, and **commit
   that status update** (message `[<your item's id>] blocked: <one-line reason> [model: <name>]`). This
   loop never halts on a failure — the failure has to become durable state instead, so the next
   stateless iteration inherits it from git/`TODO.md` and can act on it (fix the real problem,
   pick something else, or narrow scope), rather than everyone just quietly re-discovering the
   same failure from scratch. Do not leave a `blocked` status uncommitted.

8. Only if step 7 exits 0: update `TODO.md` for this item — `status: done`,
   `commit: <sha-you-are-about-to-create>` — and commit with message
   `[<your item's id>] <one-line summary> [model: <the model name you are
   running as — it is in the $HERMES_RALPH_MODEL env var if set, otherwise check
   your invocation>]`. Do NOT prefix commit subjects with `[ralph]` — the item id
   is the scope. Every commit you create in this loop (intermediate checkpoints
   included) MUST carry the `[model: <name>]` tag at the end of its subject line,
   so the audit trail shows which model produced which work.
   Then stop. Do not start another item.

Reminder of the objective (see `CONTEXT.md`): a Phase B/A-with-docs item is not truly finished
just because its own crate's tests pass — `ralph/scripts/check-todo-schema.mjs` (run inside step
7's regression) will refuse to let you mark it `done` while its `docs-pair` item is still
unfinished. Do not try to satisfy that by fabricating a stub docs page or demo.
