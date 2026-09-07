# Stage 3: Forward loop (per-iteration prompt template)

Rendered by `ralph/scripts/forward-loop.sh` with `{{todo-id}}` filled in to the item
`ralph/scripts/pick-next-todo.mjs` selected. One invocation of this prompt = one Ralph iteration
= one bounded objective (Principle 2). You have no memory of any other iteration.

---

You are one stateless iteration of a Ralph port loop, porting Base UI (React) to Leptos. You have
NO memory of prior iterations beyond what is committed to git and written in `TODO.md`/`specs/`.

1. Read `CONTEXT.md`, then `TODO.md`. Your assigned item is: `{{todo-id}}` (already selected for
   you — do not pick a different item, and do not start a second item after finishing this one).

2. Read every file listed under that item's `specs:` field, and `specs/architecture.md`.

3. Follow every citation in those specs into the ORIGINAL file/line it names, to confirm it still
   says what the spec claims. You are also a citation check, in addition to the automated one
   (`ralph/scripts/check-citations.mjs`) — do not trust the spec's paraphrase blindly.

4. Implement ONLY this item, in the crate named in its `crate:` field. Do not touch any other
   crate. Do not touch `specs/**` except to append a note to `ralph/logs/spec-discrepancies.md`
   if you find a spec is wrong or incomplete relative to upstream — never silently rewrite a spec
   to agree with your implementation, and never delete or contradict an existing citation.

5. Verify locally: `cargo check -p <crate>`, `cargo test -p <crate>`.

6. If this item's `done-when` references docs-app rendering (it has a `docs-pair:` field, or its
   `done-when` mentions docs-app), also build/run `crates/docs-app` against your change and
   confirm the relevant route renders without panicking, and — if
   `ralph/scripts/playwright-diff.mjs` exists yet — passes its differential check against the
   original React docs page for the same demo.

7. Run the FULL workspace regression:
   `bash ralph/scripts/run-regression.sh "{{todo-id}}"`
   This runs the citation check, `cargo test --workspace` (not just your crate), TODO schema
   validation, and the docs-app check from step 6 if applicable. If it fails for ANY reason —
   including a failure in a crate you didn't touch — you have regressed prior work or the item
   isn't actually done yet. Do not check the item off. Set its `status:` to `blocked` in
   `TODO.md` with a one-line note of what failed, and stop without committing.

8. Only if step 7 exits 0: update `TODO.md` for this item — `status: done`,
   `commit: <sha-you-are-about-to-create>` — and commit with message
   `[ralph][{{todo-id}}] <one-line summary>`. Then stop. Do not start another item.

Reminder of the objective (see `CONTEXT.md`): a Phase B/A-with-docs item is not truly finished
just because its own crate's tests pass — `ralph/scripts/check-todo-schema.mjs` (run inside step
7's regression) will refuse to let you mark it `done` while its `docs-pair` item is still
unfinished. Do not try to satisfy that by fabricating a stub docs page or demo.
