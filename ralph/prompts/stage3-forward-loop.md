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

7. Run the FULL workspace regression:
   `bash ralph/scripts/run-regression.sh "<your item's id>"`
   This runs the citation check, `cargo test --workspace` (not just your crate), TODO schema
   validation, and the docs-app check from step 6 if applicable. If it fails for ANY reason —
   including a failure in a crate you didn't touch — you have regressed prior work or the item
   isn't actually done yet. Do not check the item off. Set its `status:` to `blocked` in
   `TODO.md`, with a one-line note of what failed and which command reported it, and **commit
   that status update** (message `[ralph][<your item's id>] blocked: <one-line reason>`). This
   loop never halts on a failure — the failure has to become durable state instead, so the next
   stateless iteration inherits it from git/`TODO.md` and can act on it (fix the real problem,
   pick something else, or narrow scope), rather than everyone just quietly re-discovering the
   same failure from scratch. Do not leave a `blocked` status uncommitted.

8. Only if step 7 exits 0: update `TODO.md` for this item — `status: done`,
   `commit: <sha-you-are-about-to-create>` — and commit with message
   `[ralph][<your item's id>] <one-line summary>`. Then stop. Do not start another item.

Reminder of the objective (see `CONTEXT.md`): a Phase B/A-with-docs item is not truly finished
just because its own crate's tests pass — `ralph/scripts/check-todo-schema.mjs` (run inside step
7's regression) will refuse to let you mark it `done` while its `docs-pair` item is still
unfinished. Do not try to satisfy that by fabricating a stub docs page or demo.
