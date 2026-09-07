#!/usr/bin/env bash
# classralph — the canonical Ralph loop (Stage 3: forward loop / port execution).
#
# This is "classic Ralph" in Geoffrey Huntley's original sense: a `while` loop where every single
# iteration is a brand-new, disposable `claude` process with NO memory of any previous iteration.
# The only channel between iterations is what got committed to git: TODO.md's status fields, the
# specs/** the item cites, and the crate code itself. That is the entire point of the technique —
# unlike ralph1.sh/ralph2.sh (bounded fan-outs over a fixed unit list), this loop is open-ended: it
# keeps going, one bounded objective per iteration (Principle 2), until there is nothing left that
# is unblocked, or something tells it to stop.
#
# Each iteration does, in order:
#   1. `pick-next-todo.mjs` selects exactly one TODO.md item (deterministic, dependency-aware).
#   2. `stage3-forward-loop.md` is rendered with that item's id.
#   3. A brand-new `claude` process gets that prompt and does the work end-to-end, INCLUDING
#      running `run-regression.sh` itself before checking the item off (see that prompt, step 7).
#   4. This driver independently re-runs run-regression.sh too — an iteration cannot fake a
#      "done" past this loop by only self-reporting success (see run-regression.sh's own header).
#   5. Progress (or a `blocked` status) is committed by the iteration itself; if it did not commit
#      anything, that is treated as a stall and the loop halts rather than spinning forever.
#
# Usage:
#   bash ralph/scripts/classralph.sh
#
# Env overrides:
#   CLAUDE_BIN        claude executable (default: claude)
#   CLAUDE_FLAGS      extra flags passed to every invocation (default: --dangerously-skip-permissions)
#   MAX_ITERATIONS    safety cap on iterations (default: 500)
#   TODO_PATH         path to TODO.md (default: TODO.md)
#   STOP_FILE         if this file appears, halt after the current iteration (default: .ralph-stop)
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

CLAUDE_BIN="${CLAUDE_BIN:-claude}"
CLAUDE_FLAGS="${CLAUDE_FLAGS:---dangerously-skip-permissions}"
TEMPLATE="ralph/prompts/stage3-forward-loop.md"
TODO_PATH="${TODO_PATH:-TODO.md}"
MAX_ITERATIONS="${MAX_ITERATIONS:-500}"
STOP_FILE="${STOP_FILE:-.ralph-stop}"
LOG_DIR="ralph/logs/stage3"
mkdir -p "$LOG_DIR"

iteration=0
while [ "$iteration" -lt "$MAX_ITERATIONS" ]; do
  if [ -f "$STOP_FILE" ]; then
    echo "classralph: stop file '$STOP_FILE' present — halting after $iteration iteration(s)."
    break
  fi

  todo_id="$(node ralph/scripts/pick-next-todo.mjs "$TODO_PATH")"
  pick_status=$?
  if [ "$pick_status" -ne 0 ] || [ -z "$todo_id" ]; then
    echo "classralph: no pickable TODO.md item remains (everything done or everything blocked) — halting."
    break
  fi

  iteration=$((iteration + 1))
  before_sha="$(git rev-parse HEAD)"
  ts="$(date +%Y%m%d-%H%M%S)"
  # todo_id may contain slashes/spaces; make a filesystem-safe log name.
  safe_id="$(echo "$todo_id" | tr -c 'A-Za-z0-9._-' '-')"
  log_file="${LOG_DIR}/${iteration}-${safe_id}-${ts}.log"

  echo "classralph: iteration $iteration/$MAX_ITERATIONS — item: \"$todo_id\" (log: $log_file)"

  prompt_file="$(mktemp)"
  sed "s#{{todo-id}}#${todo_id}#g" "$TEMPLATE" > "$prompt_file"

  # Fresh process. This session has no memory of any prior iteration beyond TODO.md/specs/git.
  "$CLAUDE_BIN" -p "$(cat "$prompt_file")" $CLAUDE_FLAGS > "$log_file" 2>&1
  claude_status=$?
  rm -f "$prompt_file"

  if [ "$claude_status" -ne 0 ]; then
    echo "classralph: iteration for \"$todo_id\" FAILED (claude exit $claude_status) — see $log_file" >&2
    echo "classralph: halting rather than retrying blindly. Inspect the log, then resume." >&2
    break
  fi

  after_sha="$(git rev-parse HEAD)"
  if [ "$after_sha" = "$before_sha" ]; then
    echo "classralph: iteration for \"$todo_id\" made no commit (item was likely marked 'blocked' \
without committing, per the prompt's step 7/8) — halting so a human can look at $log_file." >&2
    break
  fi

  # Independent re-verification: the loop does not trust the iteration's own self-report.
  echo "classralph: re-running regression independently for \"$todo_id\"..."
  if ! bash ralph/scripts/run-regression.sh "$todo_id" >> "$log_file" 2>&1; then
    echo "classralph: independent regression check FAILED for \"$todo_id\" after commit $after_sha \
— see $log_file. Halting; this item's 'done' status should not be trusted." >&2
    break
  fi

  echo "classralph: \"$todo_id\" verified OK at $after_sha."
done

echo "classralph: stopped after $iteration iteration(s)."
