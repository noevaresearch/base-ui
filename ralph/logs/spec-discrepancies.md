## Spec Citation Discrepancies - library: form

### implementation.md citation issues:
- **Line 56**: Claims `TODO.md:407-413` has no `wraps-external:` field for the form component
  - **Actual content at TODO.md:407-413**: Avatar component (done) with exemption-from-docs-pairing note and extensive commit notes
  - **Correct location**: Form component is at TODO.md:502-508, which indeed has no `wraps-external:` field
  - **Impact**: The citation is pointing to the wrong line numbers but the actual claim (no `wraps-external:` field) is correct for the form item

### Resolution:
The citation contains correct line numbers but points to the wrong TODO.md entry. The form component at TODO.md:502-508 correctly has no `wraps-external:` field, so the spec's claim is factually correct even though the citation reference is stale. The implementation can proceed.

**Date**: 2026-09-13  
**Item**: library: form

## Spec Citation Discrepancies - library: drawer

### behavior.md citation issues:
- **Line 393**: Claims `TODO.md:393-393` says `library: drawer` is flagged `needs-batched-mining: true`
  - **Actual content at TODO.md:393**: `- [ ] library: autocomplete`
  - **Correct location**: `needs-batched-mining: true` is at TODO.md:484 for the drawer item
  - **Impact**: The citation hash doesn't match current content

### implementation.md citation issues:
- **Lines 385-392**: Claims these contain `wraps-external:` field for drawer
  - **Actual content at TODO.md:385-392**: Alert-dialog item (done) with notes about citation drift
  - **Correct location**: Drawer item is at TODO.md:477-484, no `wraps-external:` field present
  - **Impact**: The citation hash doesn't match current content

- **Line 388**: Claims this contains precise per-unit requirements
  - **Actual content at TODO.md:388**: Note about alert-dialog citation drift resolution
  - **Correct location**: No specific per-unit requirements found at this line
  - **Impact**: The citation hash doesn't match current content

### Resolution:
These appear to be stale citations from previous iterations that weren't updated when TODO.md content changed. The drawer item at TODO.md:477-484 is correctly structured and the implementation should proceed based on the actual TODO entry, not the stale citations.

**Date**: 2026-09-13
**Item**: library: drawer
# Appended by the `library: field` part-surface iteration.

## run-regression.sh's docs-rendering gate cannot see Phase D items

- **Where**: `ralph/scripts/run-regression.sh:51-52` — step 4 keys off the item's own
  `docs-pair:` field (`get-todo-field.mjs "$TODO_ID" docs-pair`).
- **Problem**: Phase D items (`docs-content: components/*`) carry `owner: library: X` and
  **no** `docs-pair:` field (see `TODO.md`'s Phase D entries and
  `ralph/scripts/check-todo-schema.mjs`, whose pairing rule is oriented the other way:
  the *library* item points at the docs item). So for exactly the items whose `done-when`
  IS "docs-app renders <page>.mdx with all its demos using crates/leptos-ui's real
  component", the gate skips the `cargo leptos build` check entirely and passes on the
  crate tests + TODO schema alone.
- **Impact**: a Phase D item can be marked `done` with the docs-app build broken or the
  route missing. The Stage 3 prompt's own step 6 covers the gap ("it has a `docs-pair:`
  field, **or its `done-when` mentions docs-app**") — the runner does not implement the
  second half of that disjunction.
- **Not fixed here**: this iteration's item (`library: field`) does carry `docs-pair:`, so
  the gate ran the docs-app build; changing `run-regression.sh` is loop-tooling scope, not
  the item's crate. Recording it so the audit loop can align the runner with the prompt.

**Date**: 2026-09-15
**Item**: library: field (observed while assessing docs-content: components/field)

# Appended by the `library: form` iteration.

## `specs/library/form/implementation.md`'s TODO.md line citation has drifted out of range

- **Where**: `specs/library/form/implementation.md:56` cites "The `library: form` entry in
  `TODO.md:502-508` has no `wraps-external:` field"; the recorded baseline key is
  `TODO.md:407-413` (`specs/library/form/implementation.citations.json:52`).
- **Problem**: the assertion itself still holds — the `library: form` entry carries no
  `wraps-external:` field — but the entry has moved far from both cited ranges: it now
  lives at `TODO.md:619-643` (the file grew as entry notes accumulated above it).
- **Impact**: none for this iteration's work: the citation checker's shift tolerance
  reports it rather than blocking, and neither cited range (`407-413`, `502-508`) falls
  inside the window this iteration's `blocked-by`/`note` edit touched, so no baseline was
  invalidated. Recording it so the audit loop can re-anchor the range to the entry's true
  position (the `context-menu`/`accordion` re-anchoring precedent) instead of this
  iteration rewriting a spec citation — the specs are not mine to silently "fix".

**Date**: 2026-09-15
**Item**: library: form (citation check by inspection, not by failure)
