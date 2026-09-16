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
  # Every entry of the `specs:` field is checked AS ITSELF — a file scope for a file entry, a directory
  # scope for a directory entry — and never as the PARENT DIRECTORY of a file entry. Expanding a file to
  # its parent was a shorthand that happened to give the whole unit directory for the usual
  # `specs/library/<name>/behavior.md` shape, but for an item whose first spec is a top-level file it
  # swallowed the entire surrounding tree: `docs-parity: page scorecard` cites `ralph/PLAN.md`, so its
  # scope became all 94 markdown/JSON files under `ralph/` — including `ralph/prompts/` (templates whose
  # `X.ts` / `Foo.tsx` placeholders are illustrations, not citations) and `ralph/logs/` (narrative records
  # that name files in shorthand). The step then failed on 39 citations in files the item never named and
  # could not fix, so the item could not pass this gate at all.
  # MEASURED BEFORE SHIPPING (157 items; old scope vs the item's own entries): 0 items newly blocked, and
  # 31 items — 28 of them already `done` — became gate-able, every one of the 28 having failed on a
  # SIBLING's spec file that its first spec's directory happened to contain.
  echo "--- citation check (scope: this item's own specs field) ---"
  IFS=',' read -ra SPEC_ENTRIES <<< "$SPECS_FIELD"
  for SPEC_ENTRY in "${SPEC_ENTRIES[@]}"; do
    SPEC_ENTRY="$(echo "$SPEC_ENTRY" | xargs)"
    if [ -z "$SPEC_ENTRY" ]; then continue; fi
    echo "    scope: $SPEC_ENTRY"
    node ralph/scripts/check-citations.mjs check --scope "$SPEC_ENTRY" || fail "citation check failed (scope: $SPEC_ENTRY)"
  done
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
        fail "visual fidelity gate failed — the FAIL line above names which rule: a page score or component-widget parity dropped beyond the tolerance, or a component region could not be compared (see ralph/generated/visual-baseline.json)"
      fi
    elif [ "$ITEM_CRATE" = "docs-app" ]; then
      echo "--- Visual fidelity budget (every recorded route: \"$TODO_ID\" has no route of its own) ---"
      if ! node ralph/scripts/check-visual-budget.mjs --all-done; then
        fail "visual fidelity gate failed on a recorded route — the FAIL line above names which rule (see ralph/generated/visual-baseline.json)"
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
  # 7. Snippet ergonomics: does the port's example code READ like upstream's? A page can be Leptos
  #    (not React) and still be ergonomically alien — internal-shaped calls and props structs where
  #    upstream teaches <Checkbox.Root>. Reported every run; the length floor (snippet size within
  #    20% of upstream) and the score target are the Phase E docs-ergonomics item's measurement.
  # 8. Part surface, repo-wide: for every `Component.Part` upstream's mined specs document, does the
  #    crate expose `Component::Part`? That is the port's ergonomic claim checked across ALL specs at
  #    once (an LSP would show it per file; a gate needs it deterministic and repo-wide). Advisory
  #    here — the library item that provides the surface is the thing gated with --strict.
  # (The part-surface check MOVED below the docs-pairing conditional — see the note there. It is keyed
  #  off the item id, so nesting it inside this guard meant the surface batches' own HARD check never
  #  ran for the one item whose done-when names it as its verification.)

  # Routes this item owns: an explicit `routes:` field, else derived from the id. Rationale: the four
  # snippet-translation batches name their routes only in prose ("(batch 1)"), so an id-derived route
  # was empty and the whole snippet/copy check block silently skipped for exactly the items that had
  # been made hard-gated for it — a gate that never ran is worse than no gate, because it reports green.
  ROUTES_FIELD="$(node ralph/scripts/get-todo-field.mjs "$TODO_ID" routes 2>/dev/null || true)"
  ITEM_ROUTES=""
  if [ -n "$ROUTES_FIELD" ]; then
    ITEM_ROUTES="$(tr ',' ' ' <<< "$ROUTES_FIELD" | tr -s ' ')"
  elif grep -qE 'components/[a-z0-9-]+' <<< "$TODO_ID"; then
    ITEM_ROUTES="$(grep -oE 'components/[a-z0-9-]+' <<< "$TODO_ID" | head -1)"
  fi

  if [ -f "ralph/scripts/snippet-ergonomics.mjs" ] && [ -n "$ITEM_ROUTES" ]; then
    if [[ "$TODO_ID" == docs-ergonomics:* || "$TODO_ID" == docs-parity:* || "$TODO_ID" == "docs-chrome: snippet translation"* ]]; then
      for r in $ITEM_ROUTES; do
        node ralph/scripts/snippet-ergonomics.mjs --route "react/$r" --length-floor 0.8 || \
          fail "Snippet ergonomics: react/$r still teaches APIs this port does not expose (or under the 80% size floor)"
      done
    else
      for r in $ITEM_ROUTES; do
        node ralph/scripts/snippet-ergonomics.mjs --route "react/$r" --length-floor 0.8 || true
      done
    fi
  fi

  # --- Website copy (the prose half, code excluded) ---
  if [ -f "ralph/scripts/check-copy-fidelity.mjs" ] && [ -n "$ITEM_ROUTES" ]; then
    COPY_BAR=95
    for r in $ITEM_ROUTES; do
      if [[ "$TODO_ID" == "docs-chrome: snippet translation"* || "$TODO_ID" == docs-copy:* || "$TODO_ID" == docs-content:*accordion* ]]; then
        node ralph/scripts/check-copy-fidelity.mjs --route "react/$r" --target "$COPY_BAR" || \
          fail "Copy fidelity: react/$r is below ${COPY_BAR}% prose coverage against upstream"
      else
        node ralph/scripts/check-copy-fidelity.mjs --route "react/$r" || true
      fi
    done
  fi

  # (The component-strict check MOVED below the docs-pairing conditional — see the note there; same
  #  reason as the part-surface check: it is keyed off the item id.)

  # --- The port's own name: no React leakage, always `@noevaresearch/base-ui` ---
  if [ -f "ralph/scripts/check-react-mentions.mjs" ]; then
    echo "--- React mentions / package alias (CONTRACT.md requirement 6) ---"
    # WHO OWNS THIS BAR: the items whose own `done-when` names these commands as their verification —
    # `docs-copy: install lines …` ("check-package-alias.mjs and check-react-mentions.mjs --source both go
    # from failing to 0 defects") and `docs-copy: Leptos-only mentions …` (both commands named verbatim,
    # plus `--all`). The `docs-parity:` lane was in this clause by name only: the scorecard item MEASURES
    # both axes per route (check-page.mjs) but its done-when claims neither as a bar, so gating it on them
    # made a measurement item un-done-markable until two other items' work landed — the same
    # "the per-item gate and the acceptance bar are different questions" defect the visual-budget bar was
    # moved for. Nothing is relaxed: the bar stays HARD, on the items that claim it.
    if [[ "$TODO_ID" == docs-copy:* || "$TODO_ID" == docs-ergonomics:* ]]; then
      node ralph/scripts/check-react-mentions.mjs --source ||         fail "the port's own page content still names React APIs or upstream's package (CONTRACT.md requirement 6)"
      if [[ "$TODO_ID" == "docs-copy: Leptos-only"* ]]; then
        node ralph/scripts/check-react-mentions.mjs --all ||           fail "rendered routes still show React APIs or upstream's package name"
      fi
    else
      node ralph/scripts/check-react-mentions.mjs --source >/dev/null 2>&1 ||         echo "NOTE: React mentions / package alias defects exist in the port's page content (advisory for this item — see ralph/logs/visual/react-mentions-source.md)"
      for r in $ITEM_ROUTES; do
        node ralph/scripts/check-react-mentions.mjs --route "react/$r" || true
      done
    fi
  fi

  # --- The port's name resolves locally, and pages install the right thing ---
  if [ -f "ralph/scripts/check-package-alias.mjs" ]; then
    # Same ownership rule as the mentions check above: HARD for the docs-copy items that name
    # `check-package-alias.mjs` in their own done-when, not for the parity lane that merely measures it.
    if [[ "$TODO_ID" == docs-copy:* || "$TODO_ID" == docs-ergonomics:* ]]; then
      node ralph/scripts/check-package-alias.mjs ||         fail "package alias: the docs name upstream's package, or @noevaresearch/base-ui no longer resolves (CONTRACT.md requirement 6)"
    else
      node ralph/scripts/check-package-alias.mjs >/dev/null 2>&1 ||         echo "NOTE: package-alias defects exist (advisory for this item — run ralph/scripts/check-package-alias.mjs)"
    fi
  fi

  if [ -f "ralph/scripts/check-docs-contract.mjs" ]; then
    echo "--- Mirrored-page snippet & behaviour contracts (specs/docs-content/CONTRACT.md) ---"
    # HARD for the item that owns the contract work; advisory elsewhere.
    if [[ "$TODO_ID" == docs-spec:* ]]; then
      node ralph/scripts/check-docs-contract.mjs --strict || \
        fail "mirrored pages still lack a snippet & behaviour contract"
    else
      node ralph/scripts/check-docs-contract.mjs --todo-id "$TODO_ID" || true
    fi
  fi
fi

# ---------------------------------------------------------------------------
# The surface checks — run for EVERY item, not only for docs-paired/docs-app ones.
# ---------------------------------------------------------------------------
# Both are keyed off the ITEM ID (HARD for the surface batches and the ergonomics item, advisory but
# always NAMED otherwise), so they belong outside the `docs-pair`/`crate: docs-app` guard above.
# Nested inside it they could not run for `library: namespaced part surface (ported batch)` at all —
# that item carries no `docs-pair`, because it is not a component — so the gate reported
# "REGRESSION OK" for an item whose own done-when names
# `check-part-surface.mjs --components <batch> --strict` as its verification, while that command
# exits 1. A gate that cannot run is worse than no gate, because its silence reads as green: the same
# defect the routes note above records for the snippet/copy blocks.

# --- Part surface, repo-wide: for every `Component.Part` upstream's mined specs document, does the
#     crate expose `Component::Part`? That is the port's ergonomic claim checked across ALL specs at
#     once (an LSP would show it per file; a gate needs it deterministic and repo-wide).
#     HARD when the item is the surface work itself (its done-when is exactly this check, scoped to
#     its batch) or the ergonomics item that consumes it. ADVISORY otherwise: the surface is not yet
#     built for most units, so a blanket failure would block every unrelated item in the repo.
if [ -f "ralph/scripts/check-part-surface.mjs" ]; then
  echo "--- Part surface across every mined spec (Component::Part form) ---"
  if [[ "$TODO_ID" == "library: namespaced part surface (ported batch)"* ]]; then
    node ralph/scripts/check-part-surface.mjs --components checkbox,checkbox-group,avatar,button,collapsible,field,fieldset,form,meter,otp-field,progress,separator,toggle,accordion --strict || \
      fail "the ported batch's parts are not exposed as Component::Part"
  elif [[ "$TODO_ID" == "library: namespaced part surface (menus batch)"* ]]; then
    node ralph/scripts/check-part-surface.mjs --components menu,menubar,context-menu,navigation-menu,toolbar,dialog,alert-dialog,popover,tooltip,preview-card --strict || \
      fail "the menus batch's parts are not exposed as Component::Part"
  elif [[ "$TODO_ID" == "library: namespaced part surface (inputs batch)"* ]]; then
    node ralph/scripts/check-part-surface.mjs --components input,number-field,radio,radio-group,select,combobox,autocomplete,slider,switch,scroll-area,tabs,toast,drawer,direction-provider,csp-provider --strict || \
      fail "the inputs batch's parts are not exposed as Component::Part"
  elif [[ "$TODO_ID" == docs-ergonomics:* ]]; then
    node ralph/scripts/check-part-surface.mjs --strict || \
      fail "docs-ergonomics depends on the namespaced part surface, which is still incomplete"
  else
    node ralph/scripts/check-part-surface.mjs || true
  fi
fi

# --- Component cycle: strict, spec-based feedback against specs/library/<name>/behavior.md ---
# HARD for the items that own the part surface (they exist to close exactly this); ADVISORY — but always
# printed with NAMED gaps — for other library items, so a component can never be declared done with the
# spec's parts, props or sections unproven, while unrelated work is not blocked by a bar it did not claim.
# For a surface batch the hard axes are `parts` and `namespaced path` (see that script's header): the
# other three axes belong to each component's own `library:` item.
if [ -f "ralph/scripts/check-component-strict.mjs" ] && [[ "$TODO_ID" == library:* ]]; then
  echo "--- Component strict (specs/library/<name>/behavior.md: parts, props, sections, hygiene) ---"
  if [[ "$TODO_ID" == "library: namespaced part surface"* ]]; then
    node ralph/scripts/check-component-strict.mjs --todo-id "$TODO_ID" --strict || \
      fail "component strict: the spec's parts/props/sections are not all proven by the port (see the named gaps above)"
  else
    node ralph/scripts/check-component-strict.mjs --todo-id "$TODO_ID" || true
  fi
fi

echo "=== REGRESSION OK for $TODO_ID ==="
