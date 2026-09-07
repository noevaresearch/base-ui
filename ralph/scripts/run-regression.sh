#!/usr/bin/env bash
# Full verification gate for one TODO.md item, per the approved plan's "Full-workspace regression
# before checkoff" (verification pipeline item 3). Called by the Stage 3 forward-loop prompt
# (step 7) AND independently re-run by forward-loop.sh itself before it trusts a "done" status —
# an iteration cannot fake this past the driver by only self-reporting success.
#
# Usage: bash ralph/scripts/run-regression.sh "<todo-id>"
#
# Exit code is non-zero if the item is not actually safe to check off.
set -euo pipefail

TODO_ID="${1:?Usage: run-regression.sh <todo-id>}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

fail() {
  echo "REGRESSION FAILED for \"$TODO_ID\": $1" >&2
  exit 1
}

echo "=== run-regression.sh: $TODO_ID ==="

# 1. Citation check, scoped to this item's specs (falls back to full specs/ if the field can't
#    be read, which only happens for a malformed TODO.md — check-todo-schema below will report why).
SPECS_FIELD="$(node ralph/scripts/get-todo-field.mjs "$TODO_ID" specs 2>/dev/null || true)"
if [ -n "$SPECS_FIELD" ]; then
  FIRST_SPEC="$(echo "$SPECS_FIELD" | cut -d',' -f1 | xargs)"
  SCOPE_DIR="$(dirname "$FIRST_SPEC")"
  echo "--- citation check (scope: $SCOPE_DIR) ---"
  node ralph/scripts/check-citations.mjs check --scope "$SCOPE_DIR" || fail "citation check failed"
else
  echo "--- citation check (scope: specs/, could not resolve item's own specs field) ---"
  node ralph/scripts/check-citations.mjs check || fail "citation check failed"
fi

# 2. Full workspace regression — not just the touched crate. This is what stops a Stage 3
#    iteration from silently breaking earlier work while completing new work.
if [ ! -f "Cargo.toml" ]; then
  fail "no Cargo.toml at repo root — the Rust workspace has not been scaffolded yet (approved \
plan build-order step 10). There is nothing for cargo to check; do not treat this as a pass."
fi
echo "--- cargo test --workspace ---"
cargo test --workspace || fail "cargo test --workspace failed"

# 3. TODO.md structural validation, including the docs-pairing rule.
echo "--- TODO.md schema check ---"
node ralph/scripts/check-todo-schema.mjs || fail "TODO.md schema check failed"

# 4. If this item's done-when references docs-app rendering, verify it for real rather than
#    trusting the crate tests alone.
DOCS_PAIR="$(node ralph/scripts/get-todo-field.mjs "$TODO_ID" docs-pair 2>/dev/null || true)"
if [ -n "$DOCS_PAIR" ]; then
  if [ ! -d "crates/docs-app" ]; then
    fail "item has a docs-pair ($DOCS_PAIR) but crates/docs-app does not exist yet — cannot \
verify docs rendering, so this item is not actually done regardless of crate test results."
  fi
  echo "--- docs-app build ---"
  (cd crates/docs-app && cargo leptos build) || fail "docs-app build failed"
  if [ -f "ralph/scripts/playwright-diff.mjs" ]; then
    echo "--- Playwright differential check ---"
    node ralph/scripts/playwright-diff.mjs --todo-id "$TODO_ID" || fail "Playwright differential check failed"
  else
    echo "NOTE: ralph/scripts/playwright-diff.mjs does not exist yet (build-order step 10) — \
skipping differential check. This item should not be trusted as fully verified until that \
script exists and passes at least once."
  fi
fi

echo "=== REGRESSION OK for $TODO_ID ==="
