#!/usr/bin/env bash
# ralph2 — Stage 2 (implementation mining) fan-out driver.
#
# Same isolation contract as ralph1: one BRAND-NEW `opencode` process per unit, per Huntley's Ralph
# technique — nothing carries between units except what is on disk (specs/**, the unit's citation
# sidecar). `ralph/prompts/stage2-implementation-mining.md` is explicit that it runs "after Stage 1
# has produced specs/library/{{unit}}/behavior.md ... dispatched per-unit as a bounded fan-out
# task ... NOT a loop" — this script is that fan-out, in bash, with a fresh process per iteration.
# --parallel below only controls how many of those independent invocations run at once; it does
# NOT introduce cross-unit state, since each unit still only reads/writes its own spec files.
#
# A unit is only processed once its Stage 1 output (specs/library/{{unit}}/behavior.md) exists;
# units without one are skipped with a note, since implementation.md would have no ground truth to
# build on.
#
# Usage:
#   bash ralph/scripts/ralph2.sh                        # all library units with a behavior.md
#   bash ralph/scripts/ralph2.sh dialog menu             # only these units
#   bash ralph/scripts/ralph2.sh --parallel 5            # all units, 5 opencode processes at once
#
# With --parallel > 1, each unit's own `opencode` transcript still streams to stdout (via `tee`),
# so concurrent units' output WILL interleave in the terminal — an accepted tradeoff of real
# parallelism. Each unit's own log file under ralph/logs/stage2/ stays clean regardless.
#
# Units with `hasTests: false` in components.json (measured 2026-09-07: types,
# unstable-use-media-query) never get a behavior.md from ralph1.sh — Stage 1 mines from tests,
# and there are none to mine. Rather than skip them here too (they'd be silently dropped
# entirely, with neither script ever attempting them — the bug this comment is here to prevent
# regressing to), this driver routes them through `stage-source-only-mining.md` instead, which
# produces both behavior.md AND implementation.md from source in one pass.
#
# Env overrides:
#   OPENCODE_BIN     opencode executable (default: opencode)
#   OPENCODE_FLAGS   extra flags passed to every invocation (default: --auto)
#   FORCE=1          re-mine units that already have an implementation.md (default: skip them)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

OPENCODE_BIN="${OPENCODE_BIN:-opencode}"
OPENCODE_FLAGS="${OPENCODE_FLAGS:---auto}"
TEMPLATE="ralph/prompts/stage2-implementation-mining.md"
SOURCE_ONLY_TEMPLATE="ralph/prompts/stage-source-only-mining.md"
PLAN_TEMPLATE="ralph/prompts/stage2-plan-batches.md"
BATCH_TEMPLATE="ralph/prompts/stage2-batch-mine.md"
SYNTHESIS_TEMPLATE="ralph/prompts/stage2-synthesis.md"
LOG_DIR="ralph/logs/stage2"
STATUS_DIR="${LOG_DIR}/.exitcodes"
mkdir -p "$LOG_DIR" "$STATUS_DIR"

# ---- argument parsing: --parallel N, plus optional unit-name filters ----
PARALLEL=1
unit_filters=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --parallel)
      PARALLEL="$2"
      shift 2
      ;;
    --parallel=*)
      PARALLEL="${1#--parallel=}"
      shift
      ;;
    *)
      unit_filters+=("$1")
      shift
      ;;
  esac
done
if ! [[ "$PARALLEL" =~ ^[0-9]+$ ]] || [ "$PARALLEL" -lt 1 ]; then
  echo "ralph2: --parallel must be a positive integer, got \"$PARALLEL\"" >&2
  exit 1
fi

want_unit() {
  local name="$1"
  if [ "${#unit_filters[@]}" -eq 0 ]; then
    return 0
  fi
  for u in "${unit_filters[@]}"; do
    if [ "$u" = "$name" ]; then
      return 0
    fi
  done
  return 1
}

# Mines exactly one unit's implementation.md. Runs inline (PARALLEL=1) or backgrounded
# (PARALLEL>1) — identical body either way. Writes its own exit code to STATUS_DIR/<name> so the
# final summary can report failures even for backgrounded jobs.
mine_unit() {
  local name="$1" src_dir="$2"
  local impl_path="specs/library/${name}/implementation.md"

  local src_files
  src_files="$(find "$src_dir" -type f \( -name '*.ts' -o -name '*.tsx' \) -not -name '*.test.tsx' | sort | sed 's/^/- /')"

  local prompt_file
  prompt_file="$(mktemp)"
  {
    sed \
      -e "s#{{unit}}#${name}#g" \
      -e "s#{{srcFiles}}#see list below#g" \
      "$TEMPLATE"
    echo
    echo "{{srcFiles}} for this unit (non-test files under ${src_dir}):"
    echo "$src_files"
  } > "$prompt_file"

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph2: mining '$name' -> $impl_path (log: $log_file)"

  # set +e/-e is scoped to this call only. When mine_unit runs backgrounded (`&`), bash forks a
  # subshell for it, so this toggle never leaks into the parent script's own -e setting.
  set +e
  # Same two fixes as ralph1.sh: `-f` ATTACHES a file rather than sending it as the message
  # (confirmed via `opencode run --help`), so the prompt must be the positional message argument;
  # and </dev/null is required so opencode can't inherit and drain this loop's row-pipe stdin
  # (`done < <(jq ...)`), which is what made the loop appear to finish early with no error.
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph2: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph2: '$name' done."
  fi
  return "$status"
}

# Mines exactly one hasTests:false unit's behavior.md AND implementation.md in one pass, via
# stage-source-only-mining.md. Same isolation/logging contract as mine_unit above.
mine_source_only_unit() {
  local name="$1" src_dir="$2"
  local impl_path="specs/library/${name}/implementation.md"

  local src_files
  src_files="$(find "$src_dir" -type f \( -name '*.ts' -o -name '*.tsx' \) -not -name '*.test.tsx' | sort | sed 's/^/- /')"

  local prompt_file
  prompt_file="$(mktemp)"
  {
    sed \
      -e "s#{{unit}}#${name}#g" \
      -e "s#{{srcFiles}}#see list below#g" \
      "$SOURCE_ONLY_TEMPLATE"
    echo
    echo "{{srcFiles}} for this unit (non-test files under ${src_dir}):"
    echo "$src_files"
  } > "$prompt_file"

  local log_file="${LOG_DIR}/${name}.log"
  echo "ralph2: mining '$name' (source-only, no tests) -> $impl_path (log: $log_file)"

  set +e
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  echo "$status" > "${STATUS_DIR}/${name}"

  if [ "$status" -ne 0 ]; then
    echo "ralph2: '$name' FAILED (exit $status) — see $log_file" >&2
  else
    echo "ralph2: '$name' done."
  fi
  return "$status"
}

# Runs one opencode invocation from a rendered template. Returns opencode's exit status.
# Shared by mine_unit_batched's plan/batch/synthesis calls below — each is its own real process
# (not an internal subagent fan-out inside one long-lived process), which is the actual fix for
# the stall pattern measured 2026-09-07: a single process mining toast or number-field via
# internal batching accumulated context across batches and stalled silently ~12-15 minutes in,
# twice each, with no error — just dead air. One bounded task per process, same as every other
# Ralph stage, is what prevents that from compounding.
run_one_opencode() {
  local prompt_file="$1" log_file="$2"
  set +e
  "$OPENCODE_BIN" run $OPENCODE_FLAGS "$(cat "$prompt_file")" < /dev/null > "$log_file" 2>&1
  local status=$?
  set -e
  rm -f "$prompt_file"
  return "$status"
}

# Mines a needs-batched-mining:true unit via real per-batch process isolation: one process to
# plan the partition, one process PER BATCH, one process to synthesize. Each step is independently
# skip-if-exists, so a stall/failure on one batch never forces redoing already-done work — same
# property the internal-fan-out approach was supposed to have but didn't, because it was all one
# process's context.
mine_unit_batched() {
  local name="$1" src_dir="$2"
  local parts_dir="specs/library/${name}/parts"
  local plan_path="${parts_dir}/PLAN.json"
  local impl_path="specs/library/${name}/implementation.md"
  mkdir -p "$parts_dir"

  if [ ! -f "$plan_path" ]; then
    local src_files
    src_files="$(find "$src_dir" -type f \( -name '*.ts' -o -name '*.tsx' \) -not -name '*.test.tsx' | sort | sed 's/^/- /')"
    local prompt_file
    prompt_file="$(mktemp)"
    { sed -e "s#{{unit}}#${name}#g" -e "s#{{srcFiles}}#see list below#g" "$PLAN_TEMPLATE"
      echo; echo "{{srcFiles}} for this unit (non-test files under ${src_dir}):"; echo "$src_files"
    } > "$prompt_file"
    local plan_log="${LOG_DIR}/${name}-plan.log"
    echo "ralph2: planning batches for '$name' -> $plan_path (log: $plan_log)"
    if ! run_one_opencode "$prompt_file" "$plan_log"; then
      echo "ralph2: '$name' plan step FAILED — see $plan_log" >&2
      echo 1 > "${STATUS_DIR}/${name}"; return 1
    fi
    if [ ! -f "$plan_path" ]; then
      echo "ralph2: '$name' plan step produced no $plan_path — see $plan_log" >&2
      echo 1 > "${STATUS_DIR}/${name}"; return 1
    fi
  fi

  # Mines exactly one batch, backgroundable. Writes its own exit code to a per-batch status file
  # (distinct from STATUS_DIR/${name}, which is this whole unit's final status) so the dispatch
  # loop below can tell which batch(es) failed even when several ran concurrently.
  mine_one_batch() {
    local unit="$1" batch_name="$2" batch_files_json="$3" parts_dir="$4"
    local batch_impl_path="${parts_dir}/${batch_name}.implementation.md"
    local batch_files
    batch_files="$(echo "$batch_files_json" | jq -r '.[]' | sed 's/^/- /')"
    local prompt_file
    prompt_file="$(mktemp)"
    { sed \
        -e "s#{{unit}}#${unit}#g" \
        -e "s#{{batchName}}#${batch_name}#g" \
        -e "s#{{batchFiles}}#see list below#g" \
        "$BATCH_TEMPLATE"
      echo; echo "{{batchFiles}} for this batch:"; echo "$batch_files"
    } > "$prompt_file"
    local batch_log="${LOG_DIR}/${unit}-${batch_name}.log"
    echo "ralph2: mining '$unit/$batch_name' -> $batch_impl_path (log: $batch_log)"
    if run_one_opencode "$prompt_file" "$batch_log"; then
      echo 0 > "${STATUS_DIR}/${unit}-${batch_name}"
      echo "ralph2: '$unit/$batch_name' done."
    else
      echo 1 > "${STATUS_DIR}/${unit}-${batch_name}"
      echo "ralph2: '$unit/$batch_name' FAILED — see $batch_log" >&2
    fi
  }

  # BATCH_PARALLEL controls how many of THIS unit's batches run concurrently — separate from
  # --parallel, which controls how many different UNITS run concurrently. Each batch is now a
  # fully independent process (that's the whole point of this design), so there's no structural
  # reason they must run one at a time; default 3 keeps concurrent opencode processes per unit
  # bounded rather than firing all batches at once regardless of count.
  local batch_count
  batch_count="$(jq '.batches | length' "$plan_path")"
  local batch_parallel="${BATCH_PARALLEL:-3}"
  local batch_active=0
  local dispatched_batches=()
  local batch_failed=0

  for i in $(seq 0 $((batch_count - 1))); do
    local batch_name batch_files_json batch_impl_path
    batch_name="$(jq -r ".batches[$i].name" "$plan_path")"
    batch_files_json="$(jq -c ".batches[$i].files" "$plan_path")"
    batch_impl_path="${parts_dir}/${batch_name}.implementation.md"

    if [ -f "$batch_impl_path" ] && [ "${FORCE:-0}" != "1" ]; then
      echo "ralph2: skip '$name/$batch_name' — $batch_impl_path already exists"
      continue
    fi

    # Don't start NEW batches once one has failed (same reasoning as the old single-file break:
    # a killed/failed batch must actually stop this unit's driver, not let it wander into more
    # work) — but batches already dispatched before the failure are left to finish, not killed.
    if [ "$batch_failed" -ne 0 ]; then
      echo "ralph2: '$name' — not starting '$batch_name', an earlier batch in this run failed"
      continue
    fi

    dispatched_batches+=("$batch_name")
    mine_one_batch "$name" "$batch_name" "$batch_files_json" "$parts_dir" &
    batch_active=$((batch_active + 1))
    if [ "$batch_active" -ge "$batch_parallel" ]; then
      wait -n || true
      batch_active=$((batch_active - 1))
      # Check whether the batch that just finished failed, without knowing which one it was —
      # scan all dispatched-so-far status files rather than tracking PIDs to job identity.
      for b in "${dispatched_batches[@]}"; do
        if [ -f "${STATUS_DIR}/${name}-${b}" ] && [ "$(cat "${STATUS_DIR}/${name}-${b}")" != "0" ]; then
          batch_failed=1
        fi
      done
    fi
  done
  wait || true
  for b in "${dispatched_batches[@]}"; do
    if [ -f "${STATUS_DIR}/${name}-${b}" ] && [ "$(cat "${STATUS_DIR}/${name}-${b}")" != "0" ]; then
      batch_failed=1
    fi
  done

  if [ "$batch_failed" -ne 0 ]; then
    echo "ralph2: '$name' had failed batch(es) — skipping synthesis until they're fixed" >&2
    echo 1 > "${STATUS_DIR}/${name}"; return 1
  fi

  # Re-synthesize if this run actually mined any batch (dispatched_batches non-empty), even if
  # implementation.md already exists — a stale synthesis written before a since-fixed batch was
  # re-mined must not be trusted (measured 2026-09-07: toast's synthesis referenced a version of
  # 'manager' that had since been deleted for bad citations; without this, re-mining just that one
  # part would leave the top-level index silently pointing at outdated content).
  if [ -f "$impl_path" ] && [ "${FORCE:-0}" != "1" ] && [ "${#dispatched_batches[@]}" -eq 0 ]; then
    echo "ralph2: '$name' done (synthesis already existed, no batches were re-mined this run)."
    echo 0 > "${STATUS_DIR}/${name}"; return 0
  fi

  local prompt_file
  prompt_file="$(mktemp)"
  sed -e "s#{{unit}}#${name}#g" "$SYNTHESIS_TEMPLATE" > "$prompt_file"
  local synth_log="${LOG_DIR}/${name}-synthesis.log"
  echo "ralph2: synthesizing '$name' -> $impl_path (log: $synth_log)"
  if ! run_one_opencode "$prompt_file" "$synth_log"; then
    echo "ralph2: '$name' synthesis FAILED — see $synth_log" >&2
    echo 1 > "${STATUS_DIR}/${name}"; return 1
  fi
  if [ ! -f "$impl_path" ]; then
    echo "ralph2: '$name' synthesis produced no $impl_path — see $synth_log" >&2
    echo 1 > "${STATUS_DIR}/${name}"; return 1
  fi
  echo "ralph2: '$name' done."
  echo 0 > "${STATUS_DIR}/${name}"
  return 0
}

active_jobs=0
attempted_units=()

# Stage 2 is source-only (packages/react/src units); implementation.md's own template only ever
# names specs/library/{{unit}}, so this driver is scoped to library components, not packages/utils.
while IFS= read -r encoded; do
  [ -z "$encoded" ] && continue
  row="$(echo "$encoded" | base64 --decode)"
  name="$(echo "$row" | jq -r '.[0]')"
  src_dir="$(echo "$row" | jq -r '.[1]')"

  want_unit "$name" || continue

  has_tests="$(echo "$row" | jq -r '.[2]')"
  behavior_path="specs/library/${name}/behavior.md"
  impl_path="specs/library/${name}/implementation.md"

  if [ "$has_tests" = "false" ]; then
    # No test files exist for this unit — Stage 1 never ran (see ralph1.sh's own comment on the
    # same filter). Route through the source-only combined template instead of requiring
    # behavior.md to pre-exist.
    if [ -f "$impl_path" ] && [ "${FORCE:-0}" != "1" ]; then
      echo "ralph2: skip '$name' — $impl_path already exists (set FORCE=1 to re-mine)"
      continue
    fi

    attempted_units+=("$name")

    if [ "$PARALLEL" -eq 1 ]; then
      mine_source_only_unit "$name" "$src_dir" || true
    else
      mine_source_only_unit "$name" "$src_dir" &
      active_jobs=$((active_jobs + 1))
      if [ "$active_jobs" -ge "$PARALLEL" ]; then
        wait -n || true
        active_jobs=$((active_jobs - 1))
      fi
    fi
    continue
  fi

  if [ ! -f "$behavior_path" ]; then
    echo "ralph2: skip '$name' — no $behavior_path yet (run ralph1.sh for this unit first)"
    continue
  fi

  # needs-batched-mining units get real per-process batch isolation (mine_unit_batched) instead
  # of one process internally fanning out to subagents (mine_unit) — the latter is what stalled
  # silently on toast/number-field, see mine_unit_batched's own comment for the measured detail.
  needs_batch="$(node ralph/scripts/get-todo-field.mjs "library: ${name}" needs-batched-mining 2>/dev/null || true)"

  if [ -f "$impl_path" ] && [ "${FORCE:-0}" != "1" ]; then
    # For batched units, don't trust impl_path alone — if a part got deleted/invalidated after
    # synthesis ran (e.g. a citation-integrity fix removed one bad part but left the stale
    # synthesis and the other good parts in place), this skip would otherwise silently no-op
    # forever instead of ever regenerating the missing part (measured 2026-09-07: exactly this,
    # on toast's 'manager' part). Verify every batch in PLAN.json still has its file before
    # trusting the top-level skip.
    all_batches_present=1
    if [ "$needs_batch" = "true" ] && [ -f "specs/library/${name}/parts/PLAN.json" ]; then
      plan_batch_count="$(jq '.batches | length' "specs/library/${name}/parts/PLAN.json" 2>/dev/null || echo 0)"
      for pi in $(seq 0 $((plan_batch_count - 1))); do
        pb_name="$(jq -r ".batches[$pi].name" "specs/library/${name}/parts/PLAN.json")"
        if [ ! -f "specs/library/${name}/parts/${pb_name}.implementation.md" ]; then
          all_batches_present=0
        fi
      done
    fi
    if [ "$all_batches_present" -eq 1 ]; then
      echo "ralph2: skip '$name' — $impl_path already exists (set FORCE=1 to re-mine)"
      continue
    else
      echo "ralph2: '$name' has a missing batch part despite $impl_path existing — re-checking rather than skipping"
    fi
  fi

  attempted_units+=("$name")

  # || true on every path below: a single unit failing must not abort the whole batch under
  # set -e — failures are tracked via STATUS_DIR and reported in the summary at the end instead.
  if [ "$PARALLEL" -eq 1 ]; then
    if [ "$needs_batch" = "true" ]; then
      mine_unit_batched "$name" "$src_dir" || true
    else
      mine_unit "$name" "$src_dir" || true
    fi
  else
    if [ "$needs_batch" = "true" ]; then
      mine_unit_batched "$name" "$src_dir" &
    else
      mine_unit "$name" "$src_dir" &
    fi
    active_jobs=$((active_jobs + 1))
    if [ "$active_jobs" -ge "$PARALLEL" ]; then
      wait -n || true
      active_jobs=$((active_jobs - 1))
    fi
  fi
done < <(jq -r '.[] | [.name, .srcDir, .hasTests] | @base64' ralph/generated/components.json)

# Drain any still-running background jobs before summarizing.
wait || true

failed=()
for name in "${attempted_units[@]}"; do
  code_file="${STATUS_DIR}/${name}"
  if [ -f "$code_file" ] && [ "$(cat "$code_file")" != "0" ]; then
    failed+=("$name")
  fi
done

if [ "${#failed[@]}" -gt 0 ]; then
  echo "ralph2: done, but ${#failed[@]} unit(s) FAILED: ${failed[*]} — see ralph/logs/stage2/<name>.log" >&2
else
  echo "ralph2: done."
fi
