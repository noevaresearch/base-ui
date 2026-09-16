#!/usr/bin/env bash
# ralph-baseui-hermes.sh — ONE iteration of the Base UI -> Leptos port loop, driven by `hermes chat`.
#
# This file is the VERSIONED driver: /data/scripts/ralph-baseui-hermes.sh is a thin wrapper that execs it,
# so what runs is what is reviewable in git. State channel: TODO.md + specs/** + git.
#
# POST-CONDITIONS (why they exist)
# --------------------------------
# An iteration once exited rc=0 having left 16 modified files — INCLUDING TWO MODIFIED HARNESS SCRIPTS —
# uncommitted, with its ledger item still `not-started`. The driver only compared `before_sha` to
# `after_sha` and printed "no commit produced this iteration — continuing next tick". That is not a safety
# property:
#   * Ralph doctrine is that state lives in the repo. Work left in the working tree is state that rots the
#     moment anything resets the tree, and it is invisible to every later iteration.
#   * uncommitted edits to the MEASUREMENT TOOLING are worse: the gate that graded the work then differs
#     from the gate anyone can reproduce from the repository, and the graded party wrote it.
# So every iteration now ends with three post-conditions: revert tooling edits (unless the item IS a
# tooling item), land uncommitted content as an explicit CHECKPOINT (never a done-marking), and report
# untracked residue.
set -uo pipefail
cd /data/workspace/baseui
set -a; source /data/.env; set +a
export OPENROUTER_API_KEY="${OPENROUTER_API_KEY_BASEUI:-$OPENROUTER_API_KEY}"
export NODE_OPTIONS="--max-old-space-size=3072"
export CARGO_TARGET_DIR="/data/cargo-target"
export HERMES_RALPH_MODEL="${RALPH_MODEL:-z-ai/glm-5.3-flash}"
HERMES=/opt/venv/bin/hermes
LOG_DIR=ralph/logs/stage3
mkdir -p "$LOG_DIR"

suggested_id="$(node ralph/scripts/pick-next-todo.mjs TODO.md 2>/dev/null || true)"
[ -z "$suggested_id" ] && suggested_id="NONE"
safe_id="$(echo "$suggested_id" | tr -c 'A-Za-z0-9._-' '-')"
ts="$(date +%Y%m%d-%H%M%S)"
log_file="${LOG_DIR}/hermes-${safe_id}-${ts}.log"

# RALPH_TEST_BEFORE_SHA lets the post-conditions be tested against a range that already contains a commit
# (a dry run otherwise has no new commit, so the "committed tooling change" guard could never be exercised).
before_sha="${RALPH_TEST_BEFORE_SHA:-$(git rev-parse HEAD)}"

PROMPT_TEXT="$(sed "s#{{todo-id}}#${suggested_id}#g" ralph/prompts/stage3-forward-loop.md)"

echo "ralph-baseui-hermes: iteration start — suggested: $suggested_id (log: $log_file)"
if [ "${RALPH_SKIP_ITERATION:-0}" = "1" ]; then
  # Testability: exercise the POST-CONDITIONS without spending a model iteration. Used to prove that a
  # dirty tree is landed and a tooling edit is reverted, which is otherwise only observable by burning
  # 20 minutes of model time and hoping the failure recurs.
  echo "ralph-baseui-hermes: RALPH_SKIP_ITERATION=1 — post-conditions only"
  rc=0
else
"$HERMES" chat -q "$PROMPT_TEXT" \
  --model "${RALPH_MODEL:-deepseek-v4-flash}" --provider "${RALPH_PROVIDER:-deepseek}" \
  ${RALPH_REASONING:+--reasoning "$RALPH_REASONING"} \
  --max-turns 150 > "$log_file" 2>&1
rc=$?
fi
MODEL_TAG="model:${RALPH_MODEL:-unknown} via ${RALPH_PROVIDER:-unknown}"
# Fallback: if primary provider failed (quota exhausted, 429/5xx, auth), retry once on fallback provider
if [ "$rc" -ne 0 ] && [ -n "${RALPH_FALLBACK_PROVIDER:-}" ]; then
  echo "ralph-baseui-hermes: primary ${RALPH_PROVIDER} failed rc=$rc — falling back to ${RALPH_FALLBACK_PROVIDER}/${RALPH_FALLBACK_MODEL}"
  "$HERMES" chat -q "$PROMPT_TEXT" \
    --model "${RALPH_FALLBACK_MODEL}" --provider "${RALPH_FALLBACK_PROVIDER}" \
    ${RALPH_FALLBACK_REASONING:+--reasoning "$RALPH_FALLBACK_REASONING"} \
    --max-turns 150 > "$log_file" 2>&1
  rc=$?
  MODEL_TAG="model:${RALPH_FALLBACK_MODEL} via ${RALPH_FALLBACK_PROVIDER}-fallback"
fi
export HERMES_RALPH_MODEL="${RALPH_MODEL} via ${RALPH_PROVIDER}"
echo "ralph-baseui-hermes: iteration done rc=$rc"

after_sha="$(git rev-parse HEAD)"
if [ "$after_sha" = "$before_sha" ]; then
  echo "ralph-baseui-hermes: no commit produced this iteration"
fi

# =============================== POST-CONDITIONS =====================================================
is_tooling_item=0
case "$suggested_id" in
  infra:*|*harness*|*tooling*|*driver*) is_tooling_item=1 ;;
esac

# (1) MEASUREMENT INTEGRITY — an iteration must not rewrite the tools that score it.
# `git status --porcelain` gives two columns: XY <path>. New harness files show as `??` and CANNOT be
# reverted by checkout — and a brand-new gate script changes behaviour just as much as an edited one, so
# both cases must be handled. Paths are handled ONE AT A TIME: a single untracked path in the pathspec made
# `git checkout -- ralph/scripts ralph/prompts ralph/driver` fail wholesale, so a planted tamper in
# snippet-lang.mjs survived a revert that had already announced success.
TOOLING_DIRTY="$(git status --porcelain ralph/scripts ralph/prompts ralph/driver 2>/dev/null || true)"
if [ -n "$TOOLING_DIRTY" ]; then
  if [ "$is_tooling_item" -eq 1 ]; then
    echo "ralph-baseui-hermes: tooling changed by a TOOLING item ($suggested_id) — left for the driver's own verification:"
    echo "$TOOLING_DIRTY" | sed 's/^/    /'
  else
    echo "ralph-baseui-hermes: MEASUREMENT INTEGRITY — iteration '$suggested_id' modified the harness that grades it; reverting:"
    echo "$TOOLING_DIRTY" | sed 's/^/    /'
    QUARANTINE=/tmp/ralph-tooling-quarantine/$(date +%Y%m%d-%H%M%S)
    echo "$TOOLING_DIRTY" | while IFS= read -r entry; do
      [ -z "$entry" ] && continue
      xy="$(printf '%s' "$entry" | cut -c1-2)"
      path="$(printf '%s' "$entry" | cut -c4-)"
      [ -z "$path" ] && continue
      if [ "$xy" = "??" ]; then
        mkdir -p "$QUARANTINE"
        mv "$path" "$QUARANTINE/" 2>/dev/null && echo "ralph-baseui-hermes:   quarantined NEW tooling file $path -> $QUARANTINE"
      else
        git checkout -- "$path" 2>/dev/null && echo "ralph-baseui-hermes:   reverted $path"
      fi
    done
    echo "ralph-baseui-hermes: harness restored to HEAD — a tooling change must be its own reviewed tooling item"
  fi
fi

# (1b) COMMITTED TOOLING CHANGES — the check above only sees the WORKING TREE, so an iteration that edits a
# gate and commits it under its own item message slips past. Observed 2026-09-16: one item commit carried
# +41 lines to check-component-strict.mjs, +79 to check-part-surface.mjs and +92 to run-regression.sh. That
# instance was sound (the surface gate never ran for `library:` items — it sat inside a docs-app guard) and
# it was found only because the verifier read the diff. A gate change is not self-authorising: name it, and
# record it on the item so the next iteration sees it instead of inheriting a quietly different gate.
COMMITTED_TOOLING="$(git diff --name-only "${before_sha}..${after_sha}" -- ralph/scripts ralph/prompts ralph/driver 2>/dev/null || true)"
if [ -n "$COMMITTED_TOOLING" ] && [ "$is_tooling_item" -eq 0 ]; then
  echo "ralph-baseui-hermes: MEASUREMENT REVIEW — this iteration's commit changed the harness it is graded by:"
  echo "$COMMITTED_TOOLING" | sed 's/^/    /'
  node ralph/scripts/note-tooling-change.mjs "$suggested_id" "$after_sha" "$(echo "$COMMITTED_TOOLING" | tr '\n' ' ')" >> "$log_file" 2>&1 || true
  if ! git diff --quiet TODO.md; then
    git add TODO.md
    git commit -q -m "[${suggested_id}] review-note: iteration changed measurement tooling in ${after_sha:0:10}" >> "$log_file" 2>&1 || true
    after_sha="$(git rev-parse HEAD)"
  fi
  echo "ralph-baseui-hermes: recorded on '$suggested_id' as a review-note"
fi

# (2) NOTHING ROTS — content left uncommitted is landed as an explicit CHECKPOINT (never a done-marking).
CONTENT_DIRTY="$(git status --porcelain crates specs docs TODO.md 2>/dev/null | awk '{print $2}')"
if [ -n "$CONTENT_DIRTY" ]; then
  dirty_count="$(echo "$CONTENT_DIRTY" | wc -l | tr -d ' ')"
  echo "ralph-baseui-hermes: iteration left $dirty_count uncommitted content file(s) — landing as a CHECKPOINT (item stays not-done):"
  echo "$CONTENT_DIRTY" | head -12 | sed 's/^/    /'
  git add -A crates specs docs TODO.md 2>/dev/null || true
  git commit -q -m "CHECKPOINT [${suggested_id}] uncommitted work from iteration ${ts} (${MODEL_TAG}); landed by the driver post-condition so it cannot rot — NOT a done-marking" 2>/dev/null || true
  echo "ralph-baseui-hermes: checkpoint committed at $(git rev-parse --short HEAD)"
  after_sha="$(git rev-parse HEAD)"
fi

# (3) RESIDUE VISIBILITY — untracked scratch is reported, never a surprise.
scratch_count="$(git status --porcelain 2>/dev/null | grep -c '^??' || true)"
if [ "${scratch_count:-0}" -gt 0 ]; then
  echo "ralph-baseui-hermes: note — $scratch_count untracked file(s) in the tree (scratch/measurement artifacts)"
fi
# =====================================================================================================

# independent regression re-run for anything newly marked done
before_todo="$(mktemp)"; git show "$before_sha:TODO.md" > "$before_todo"
changed_ids="$(node ralph/scripts/diff-todo-done.mjs "$before_todo" TODO.md 2>/dev/null || true)"
rm -f "$before_todo"
echo "$changed_ids" | while IFS= read -r changed_id; do
  [ -z "$changed_id" ] && continue
  echo "ralph-baseui-hermes: verifying $changed_id independently..."
  if ! bash ralph/scripts/run-regression.sh "$changed_id" >> "$log_file" 2>&1; then
    node ralph/scripts/mark-todo-blocked.mjs "$changed_id" \
      "hermes-driver regression re-run failed [${MODEL_TAG}] after commit ${after_sha}; see ${log_file}" \
      TODO.md >> "$log_file" 2>&1
    git add TODO.md
    git commit -m "[${changed_id}] blocked: hermes-driver regression re-run failed [${MODEL_TAG}]" >> "$log_file" 2>&1
    echo "ralph-baseui-hermes: $changed_id -> blocked (regression failed)"
  else
    echo "ralph-baseui-hermes: $changed_id verified OK at $after_sha"
  fi
done
exit 0
