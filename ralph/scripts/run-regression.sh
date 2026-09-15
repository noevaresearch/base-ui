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
#    trusting the crate tests alone. Two shapes count:
#      * the item has a `docs-pair:` naming the Phase D page it owns (Phase B/A items), or
#      * the item's own `crate:` IS docs-app (the Phase E chrome items) — those have no route
#        of their own, but every change they make lands on every ported route's page.
ITEM_CRATE="$(node ralph/scripts/get-todo-field.mjs "$TODO_ID" crate 2>/dev/null || true)"
DOCS_PAIR="$(node ralph/scripts/get-todo-field.mjs "$TODO_ID" docs-pair 2>/dev/null || true)"
if [ -n "$DOCS_PAIR" ] || [ "$ITEM_CRATE" = "docs-app" ]; then
  if [ ! -d "crates/docs-app" ]; then
    fail "item is a docs-app item (docs-pair: ${DOCS_PAIR:-none}, crate: $ITEM_CRATE) but \
crates/docs-app does not exist yet — cannot verify docs rendering, so this item is not \
actually done regardless of crate test results."
  fi
  echo "--- docs-app build ---"
  (cd crates/docs-app && cargo leptos build) || fail "docs-app build failed"
  # The differential is a DOCS-PAGE check: it derives its route from a "components/<name>" id
  # (playwright-diff.mjs:26-31). A Phase B item's `docs-pair` names the Phase D item that owns
  # that page, so a route-less id has no page of its own to diff — running it here would fail on
  # the route derivation alone, not on anything about the item, and would self-block every Phase B
  # done-marking now that the script exists (fieldset, 2026-09-15). The pairing RULE is untouched:
  # check-todo-schema.mjs (step 3) still refuses `status: done` while the pair is unfinished
  # (or the item must carry `exempt-from-docs-pairing`).
  if [ -f "ralph/scripts/playwright-diff.mjs" ]; then
    if grep -qE 'components/[a-z0-9-]+' <<< "$TODO_ID"; then
      echo "--- Playwright differential check ---"
      node ralph/scripts/playwright-diff.mjs --todo-id "$TODO_ID" || fail "Playwright differential check failed"
    else
      echo "NOTE: \"$TODO_ID\" is not a docs-page id — its docs page belongs to ${DOCS_PAIR:-the routes of this crate}, which runs \
the differential when it lands. Only the docs-app build above was verified here; this is NOT a pass."
    fi
  else
    echo "NOTE: ralph/scripts/playwright-diff.mjs does not exist yet (build-order step 10) — \
skipping differential check. This item should not be trusted as fully verified until that \
script exists and passes at least once."
  fi

  # 5. Visual fidelity. The mount differential above proves the page *renders*; it says nothing
  #    about whether the page looks like upstream's. This gate scores each route's fidelity
  #    (pixel proximity + content recall) against the live upstream render and FAILS on a
  #    regression against the recorded best-known score, so a docs item can neither ship a
  #    naked page silently nor make an existing page worse. Reported as a NOTE — never as a
  #    pass — when the upstream reference server or the Leptos docs server is not running.
  #
  #    A route-less docs-app item (the Phase E chrome items: layout shell, code blocks, demo
  #    panels, API tables) is checked against the WHOLE recorded baseline instead of one route,
  #    because that is the surface it changes — without this an item that regressed every
  #    ported page would have no gate at all to catch it.
  if [ -f "ralph/scripts/check-visual-budget.mjs" ]; then
    if grep -qE 'components/[a-z0-9-]+' <<< "$TODO_ID"; then
      echo "--- Visual fidelity budget ---"
      if ! node ralph/scripts/check-visual-budget.mjs --todo-id "$TODO_ID"; then
        fail "visual fidelity regressed (see ralph/generated/visual-baseline.json)"
      fi
    elif [ "$ITEM_CRATE" = "docs-app" ]; then
      echo "--- Visual fidelity budget (every recorded route: \"$TODO_ID\" has no route of its own) ---"
      if ! node ralph/scripts/check-visual-budget.mjs --all-done; then
        fail "visual fidelity regressed on a recorded route (see ralph/generated/visual-baseline.json)"
      fi
    fi
  fi

  # 6. Spec-level behaviour feedback. The gates above watch a page's shape and its looks; neither
  #    can see that a mirrored page teaches the WRONG FRAMEWORK (the checkbox page shipped five
  #    React snippets and passed every one of them). The obligation lives in
  #    specs/docs-content/CONTRACT.md, so it is reported every run: advisory by default, because
  #    each contract table carries citations and behavioural observables and must be authored
  #    deliberately. `check-docs-contract.mjs --strict` is the measurement for the Phase E
  #    docs-spec item that brings the already-mirrored pages up to the contract.
  if [ -f "ralph/scripts/check-docs-contract.mjs" ]; then
    echo "--- Mirrored-page snippet & behaviour contracts (specs/docs-content/CONTRACT.md) ---"
    node ralph/scripts/check-docs-contract.mjs --todo-id "$TODO_ID" || true
  fi
fi

echo "=== REGRESSION OK for $TODO_ID ==="
