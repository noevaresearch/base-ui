#!/usr/bin/env bash
# classralph — the canonical Ralph loop (Stage 3: forward loop / port execution).
#
# This is "classic Ralph" in Geoffrey Huntley's original sense: a `while` loop where every single
# iteration is a brand-new, disposable `opencode run` process with NO memory of any previous
# iteration. The only channel between iterations is what got committed to git: TODO.md's status
# fields, the specs/** the item cites, and the crate code itself. That is the entire point of the
# technique — unlike ralph1.sh/ralph2.sh (bounded fan-outs over a fixed unit list), this loop is
# open-ended: it keeps going, one bounded objective per iteration (Principle 2), until there is
# nothing left that is unblocked, or something tells it to stop.
#
# Each iteration does, in order:
#   1. `pick-next-todo.mjs` suggests one TODO.md item (deterministic, dependency-aware) — or the
#      sentinel "NONE" if it finds nothing pickable.
#   2. `stage3-forward-loop.md` is rendered with that suggestion. The prompt is explicit that this
#      is a suggestion, not a mandate (see its "Step 0" section): the agent reads TODO.md/specs
#      itself and may confirm it, or pick a different item instead — including narrowing an
#      overly conservative `blocked-by` (generate-todo.mjs's coarse default blocks every Phase B
#      item on "Phase A complete" even though most only need a handful of specific deps), or
#      determining nothing is actually pickable. This mirrors Huntley's original pattern of
#      letting the agent itself judge the most important next task, rather than always deferring
#      to a mechanical rule — the picker exists to save most iterations the trouble, not to
#      constrain the ones with a legitimate reason to differ.
#   3. A brand-new `opencode run` process gets that prompt and does the work end-to-end, INCLUDING
#      running `run-regression.sh` itself before checking the item off (see that prompt, step 7).
#   4. Because the agent may have worked on a different item than suggested, this driver never
#      assumes which item(s) it completed — it diffs TODO.md before/after to detect that, then
#      independently re-runs run-regression.sh for each (see run-regression.sh's own header): an
#      iteration cannot fake a "done" past this loop by only self-reporting success.
#   5. Progress (or a `blocked` status, always committed — see stage3-forward-loop.md step 7) is
#      the only channel back to future iterations. This loop never halts on a failure: per classic
#      Ralph, a failure just becomes durable TODO.md/git state for the next fresh, stateless
#      iteration to react to (fix it, work around it, or pick something else) — not a reason for
#      the driver itself to stop and wait for a human. If the independent re-verification in step
#      4 catches a false "done" (the agent's own regression passed but this driver's independent
#      rerun didn't), the driver itself corrects TODO.md back to `blocked` with the failure noted,
#      commits that correction, and continues — so a corrupted "done" is never trusted going
#      forward. Only `STOP_FILE` and `MAX_ITERATIONS` stop the loop.
#
# Usage:
#   bash ralph/scripts/classralph.sh
#
# Env overrides:
#   OPENCODE_BIN        opencode executable (default: opencode)
#   OPENCODE_FLAGS      extra flags passed to every invocation (default: --auto)
#   MAX_ITERATIONS      safety cap on iterations (default: 500)
#   TODO_PATH           path to TODO.md (default: TODO.md)
#   STOP_FILE           if this file appears, halt after the current iteration (default: .ralph-stop)
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
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

  suggested_id="$(node ralph/scripts/pick-next-todo.mjs "$TODO_PATH")"
  pick_status=$?
  if [ "$pick_status" -ne 0 ] || [ -z "$suggested_id" ]; then
    # The mechanical picker's one failure mode: nothing not-started has all blocked-by deps done.
    # Don't halt outright — hand the agent "NONE" and let it judge for itself (per the prompt's
    # "Step 0") whether the port is complete, everything is genuinely blocked, or the blocked-by
    # graph is just overly conservative and something is actually workable.
    suggested_id="NONE"
  fi

  iteration=$((iteration + 1))
  before_sha="$(git rev-parse HEAD)"
  ts="$(date +%Y%m%d-%H%M%S)"
  # suggested_id may contain slashes/spaces; make a filesystem-safe log name.
  safe_id="$(echo "$suggested_id" | tr -c 'A-Za-z0-9._-' '-')"
  log_file="${LOG_DIR}/${iteration}-${safe_id}-${ts}.log"

  echo "classralph: iteration $iteration/$MAX_ITERATIONS — suggested: \"$suggested_id\" (log: $log_file)"

  prompt_file="$(mktemp)"
  sed "s#{{todo-id}}#${suggested_id}#g" "$TEMPLATE" > "$prompt_file"

  # Fresh process. This session has no memory of any prior iteration beyond TODO.md/specs/git.
  # `opencode run` takes the prompt as a positional message, not via -f (which attaches a file
  # alongside a message rather than supplying the message itself) — so pass the rendered
  # template's contents directly.
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" > "$log_file" 2>&1
  agent_status=$?
  rm -f "$prompt_file"

  if [ "$agent_status" -ne 0 ]; then
    echo "classralph: iteration (suggested \"$suggested_id\") FAILED (opencode exit $agent_status) \
— see $log_file. Not halting — this loop never stops on a failure; the next fresh iteration gets \
another try. If this repeats every iteration, it's likely a real bug (e.g. in this driver's own \
opencode invocation) worth a human's attention, not something more retries will fix." >&2
    continue
  fi

  after_sha="$(git rev-parse HEAD)"
  if [ "$after_sha" = "$before_sha" ]; then
    echo "classralph: iteration (suggested \"$suggested_id\") made no commit — it likely determined \
the port is complete, everything remaining is genuinely blocked, or only reported findings. \
See $log_file. Continuing." >&2
    continue
  fi

  # The agent may have worked on a different item than suggested (Step 0 lets it override), so
  # never assume — detect which item(s) actually became `done` from the TODO.md diff itself.
  before_todo="$(mktemp)"
  git show "$before_sha:$TODO_PATH" > "$before_todo"
  changed_ids="$(node ralph/scripts/diff-todo-done.mjs "$before_todo" "$TODO_PATH")"
  rm -f "$before_todo"

  if [ -z "$changed_ids" ]; then
    # No item newly marked `done` — most likely the agent committed a `blocked` status update
    # (per stage3-forward-loop.md step 7, that's now always committed) or a fallback narrowing of
    # blocked-by. Either way that IS the durable state this loop relies on; nothing more to verify.
    echo "classralph: iteration (suggested \"$suggested_id\") committed but marked no item 'done' \
— likely a 'blocked' status update or a blocked-by narrowing. See $log_file. Continuing."
    continue
  fi

  while IFS= read -r changed_id; do
    [ -z "$changed_id" ] && continue
    if [ "$changed_id" != "$suggested_id" ]; then
      echo "classralph: note — agent worked on \"$changed_id\", overriding the suggestion \"$suggested_id\"."
    fi
    echo "classralph: \"$changed_id\" marked done — re-running regression independently..."
    if ! bash ralph/scripts/run-regression.sh "$changed_id" >> "$log_file" 2>&1; then
      echo "classralph: independent regression check FAILED for \"$changed_id\" after commit \
$after_sha — see $log_file. This 'done' status cannot be trusted, so correcting it to 'blocked' \
now (rather than halting) so the next iteration inherits the real state." >&2
      node ralph/scripts/mark-todo-blocked.mjs "$changed_id" \
        "driver's independent regression re-run failed after commit ${after_sha}; see ${log_file}" \
        "$TODO_PATH" >> "$log_file" 2>&1
      git add "$TODO_PATH"
      git commit -m "[ralph][${changed_id}] blocked: independent regression re-run failed (driver correction)" \
        >> "$log_file" 2>&1
    else
      echo "classralph: \"$changed_id\" verified OK at $after_sha."
    fi
  done <<< "$changed_ids"
done

echo "classralph: stopped after $iteration iteration(s)."
