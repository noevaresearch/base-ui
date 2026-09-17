#!/usr/bin/env bash
# ralph-watchdog.sh — keep the Base UI → Leptos forward loop moving without a human relaunching it.
#
# WHY THIS EXISTS
# ---------------
# `ralph-baseui-hermes.sh` runs exactly ONE iteration and exits; the continuous driver that used to
# call it in a loop was removed (2026-09-15, by request). Between then and now, iterations only
# started because the agent relaunched them one by one from Discord completion pings — so the whole
# experiment stopped dead if the agent was not there, and nothing in the repo recorded that fact.
# This script is the missing driver: cron calls it, and it starts an iteration when it is safe to.
#
# WHAT IT CHECKS BEFORE STARTING (each guard exists because its failure was observed)
#   1. an iteration is already running -> do nothing (two concurrent iterations fight over the same
#      working tree, TODO.md and the cargo lock)
#   2. cgroup task pressure too high   -> skip; the box hit 512/512 tasks and could not fork anything,
#      which starves the gateway and makes every tool call fail in their pre-call hook. Clear stale
#      harness Chrome first: it is the debris that fills the ceiling.
#   3. no commit in the last N attempts -> back off and report, so a wedged loop is visible rather
#      than silently burning model spend
#   4. docs server down                -> start it, since every fidelity/ergonomics measurement needs it
#
# SILENCE IS THE CONTRACT (2026-09-17)
# ------------------------------------
# Cron runs this with no_agent=true, so ANYTHING on stdout is delivered to the user verbatim, and an
# empty stdout is a silent tick. Every routine line that used to reach stdout did so ~96 times a day:
# the memory-guard preflight (called without redirecting its stdout) prints `memory: N MB / 4096MB
# (P%)` on EVERY call, and the numbers move every tick, so the user got a message per tick that said
# nothing and could never dedupe. That is noise, and noise trains the user to ignore the alert that
# matters. The rules now:
#   * routine status -> the log ($WATCHDOG_LOG), never stdout
#   * stdout         -> only a state CHANGE: memory crossing into CRITICAL/TIGHT, a stalled-commit
#                       warning appearing, or an explicit --dry / --reap-only run asking for output
#   * routine "started iteration pid N" is logged, not announced: at one iteration per tick it is the
#     normal case, not news. A watchdog that speaks only when it intervenes stays worth reading.
# The memory level is remembered in $LEVEL_STATE so a persistent condition is said once, not every 15min.
#
# THIS FILE IS THE CANONICAL COPY. Cron resolves its `script:` field under /data/scripts, so
# /data/scripts/ralph-watchdog.sh is a symlink to this file. The point of the move is that this script
# spent its first day OUTSIDE version control (/data/scripts is not a git repo), which left the loop's
# driver — the single piece of infrastructure the whole experiment depends on — invisible to `git log`
# and to review.
#
# ENV: TASK_CEILING (default 420), MAX_TASKS (512), REAP_MIN_AGE (600s) and REAP_ORPHAN_AGE (60s) are
# overridable, so the guards can be exercised deliberately instead of only being read.

set -uo pipefail

# Overridable so the guards — and the silence contract — can be exercised against a stub repo in a
# temp dir instead of only being read (same spirit as the deploy watchdog's WATCH_FAKE_* hooks).
REPO="${RALPH_REPO:-/data/workspace/baseui}"
LOGDIR="$REPO/ralph/logs/stage3"
DRIVER_LOG="$LOGDIR/driver.log"
WATCHDOG_LOG="$LOGDIR/watchdog.log"
STATE="${RALPH_STATE:-/data/run/ralph-watchdog.state}"
LEVEL_STATE="${LEVEL_STATE:-/data/run/ralph-watchdog.level}"
HARNESS="${RALPH_HARNESS:-/data/scripts/ralph-baseui-hermes.sh}"
mkdir -p "$LOGDIR" "$(dirname "${RALPH_STATE:-/data/run/x}")" 2>/dev/null
stamp() { date -u +%Y-%m-%dT%H:%M:%SZ; }
log() { echo "$(stamp) $*" >> "$WATCHDOG_LOG"; }

# MEMORY PREFLIGHT (measurement: 2026-09-16 — 4096MB cap, 3602MB at rest, oom_kill 51, peak 4100MB).
# One chromium is ~1.4GB and a rust/wasm build ~1.5GB, so a build overlapping a measurement is an OOM kill,
# and the killed process is usually the iteration's own tool call — which presents as 'the loop stopped'.
# Check the budget before starting work; the guard also reaps orphaned browsers that hold 1.4GB each.
# The guard's own stdout is captured, never inherited: see SILENCE IS THE CONTRACT above.
MG_OUT=""
MG_RC=0
if [ -x "$REPO/ralph/scripts/memory-guard.sh" ]; then
  MG_OUT="$("$REPO/ralph/scripts/memory-guard.sh" --preflight 2>&1)"
  MG_RC=$?
fi
[ -n "$MG_OUT" ] && printf '%s\n' "$MG_OUT" >> "$WATCHDOG_LOG"
LEVEL=OK
case "$MG_OUT" in
  *"memory-guard: CRITICAL"*) LEVEL=CRITICAL ;;
  *"memory-guard: TIGHT"*)    LEVEL=TIGHT ;;
esac
[ "$MG_RC" = "3" ] && LEVEL=CRITICAL
MEM_LINE="$(printf '%s' "$MG_OUT" | grep -m1 '^memory:')"
PREV_LEVEL="$(cat "$LEVEL_STATE" 2>/dev/null || echo none)"

if [ "$LEVEL" = "CRITICAL" ]; then
  log "skip: memory critical (previous state: $PREV_LEVEL)"
  printf 'CRITICAL' > "$LEVEL_STATE" 2>/dev/null
  if [ "$PREV_LEVEL" != "CRITICAL" ]; then
    echo "ralph loop IDLE on memory — ${MEM_LINE:-memory: (unavailable)}"
    echo "no new iteration starts above 90% (a build plus a measurement cannot fit); the loop resumes by itself when it drops"
  fi
  exit 0
fi
if [ "$LEVEL" = "TIGHT" ] && [ "$PREV_LEVEL" != "TIGHT" ]; then
  echo "ralph loop: ${MEM_LINE:-memory: (unavailable)} — iterations continue, one browser at a time"
fi
printf '%s' "$LEVEL" > "$LEVEL_STATE" 2>/dev/null

# Two rustc/cargo jobs fit in the remaining budget; four do not.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"

# Refresh the environment verdict the prompt tells the loop to read FIRST (ralph/scripts/env-health.mjs). Cheap, no
# browser, no build: it checks memory headroom, whether a browser is allowed here, both reference servers, whether the
# committed measurement is parseable and fresh, whether the instruments pass their own fixtures, and whether the ledger
# has an unsatisfiable dependency. Written to ralph/generated/env-health.json before the iteration starts, because a
# verdict the loop reads is only useful if it is current.
if [ -x "$REPO/ralph/scripts/env-health.mjs" ]; then
  node "$REPO/ralph/scripts/env-health.mjs" --quiet || true   # 3 = degraded, 4 = broken; both are FOR the loop to read
fi

TASK_CEILING="${TASK_CEILING:-420}"
MAX_TASKS="${MAX_TASKS:-512}"
REAP_MIN_AGE="${REAP_MIN_AGE:-600}"
REAP_ORPHAN_AGE="${REAP_ORPHAN_AGE:-60}"
DRY=0
REAP_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --dry) DRY=1 ;;
    --reap-only) REAP_ONLY=1 ;;
  esac
done

tasks_now() { cat /sys/fs/cgroup/pids.current 2>/dev/null || echo 0; }

# --- what counts as stale Chrome? ---------------------------------------------------------------
# The FIRST version of this reaped a hardcoded directory allowlist:
#     user-data-dir=/tmp/(visdiff|gaprep|sniperg|live|cdp|anc)
# and that allowlist was the defect. Measured 2026-09-16 06:42 with the box pinned at the ceiling:
# EIGHT chrome processes held the cgroup and this pattern matched ZERO of them — every one lived
# under /tmp/diag-J8zbNN, a name no branch of the pattern contains — so the "reap stale harness
# Chrome first" guard reaped nothing and the starvation it exists to clear went uncleared. A new
# harness directory silently escapes an allowlist.
#
# What actually identifies debris is OWNERSHIP, not the directory's name. ralph/scripts/lib/browser.mjs
# launches Chrome with `detached: true` (browser.mjs:135, "so it leads its own process group"), so a
# Chrome whose launcher has exited is reparented to init and shows ppid 1: nobody is left to read its
# output. Age is the second signal, for a profile under /tmp that has simply outlived any plausible run.
#
# SAFETY — the only shape a real, user-owned browser can have on this box is a live parent AND a
# profile outside /tmp, and that shape is never reaped. Two consequences, both accepted and both
# small: a live run's `chrome_crashpad_handler` children are detached by Chrome itself (observed
# ppid 1 while their parent was still alive), so they are reapable; and reaping only ever happens
# under real pressure (guard 2's ceiling branch) or when an operator asks for it explicitly.
chrome_procs() {
  ps -eo pid=,ppid=,etimes=,args= | awk '
    /[c]hrome-linux64/ {
      d = ""
      for (i = 4; i <= NF; i++) if ($i ~ /^--user-data-dir=/) d = substr($i, 17)
      print $1, $2, $3, (d == "" ? "-" : d)
    }'
}

# PIDs this box would reap right now. Detection only — prints, kills nothing, so `--dry` can show the
# same set the reap would act on.
stale_chrome_pids() {
  chrome_procs | while read -r pid ppid age dir; do
    if [ "$ppid" = "1" ] && [ "$age" -ge "$REAP_ORPHAN_AGE" ]; then
      printf '%s\n' "$pid"          # launcher gone: orphaned, and past any launch still settling
      continue
    fi
    case "$dir" in
      /tmp/*) [ "$age" -ge "$REAP_MIN_AGE" ] && printf '%s\n' "$pid" ;;
    esac
  done
}

# Reap, in passes: killing a Chrome root reparents its children to init, which makes them orphans —
# the very signal this tests for — so the next pass catches them without any process-group guessing.
reap_stale_chromes() {
  local pass n killed=0
  for pass in 1 2 3 4; do
    n=0
    while read -r pid; do
      [ -n "$pid" ] || continue
      kill -9 "$pid" 2>/dev/null && n=$((n + 1))
    done < <(stale_chrome_pids)
    killed=$((killed + n))
    [ "$n" -eq 0 ] && break
    sleep 0.3
  done
  printf '%s\n' "$killed"
}

# --- --reap-only: clear debris and stop ----------------------------------------------------------
# Deliberately BEFORE guard 1: it acts on ownership, never on whether an iteration is running, and
# that is what makes it safe to clear debris while the loop is working (a live run's own Chrome has a
# live parent, so it is not a candidate). It starts nothing and touches no server.
if [ "$REAP_ONLY" = "1" ]; then
  TASKS=$(tasks_now)
  if [ "$DRY" = "1" ]; then
    echo "dry run: would reap $(stale_chrome_pids | grep -c .) stale chrome process(es) (tasks ${TASKS}/${MAX_TASKS})"
    stale_chrome_pids | while read -r pid; do
      ps -p "$pid" -o pid=,ppid=,etimes=,args= 2>/dev/null | cut -c1-150
    done
    exit 0
  fi
  KILLED=$(reap_stale_chromes)
  sleep 1
  NOW=$(tasks_now)
  echo "reaped ${KILLED} stale chrome process(es); cgroup tasks ${TASKS} -> ${NOW} (ceiling ${TASK_CEILING}/${MAX_TASKS})"
  exit 0
fi

# --- 1. is an iteration already running? -------------------------------------------------------
if pgrep -f "ralph-baseui-hermes.sh" >/dev/null 2>&1; then
  log "skip: an iteration is already running (pid $(pgrep -f 'ralph-baseui-hermes.sh' | head -1))"
  exit 0
fi

# --- 2. resource pressure ----------------------------------------------------------------------
TASKS=$(tasks_now)
if [ "$TASKS" -ge "$TASK_CEILING" ]; then
  CANDIDATES=$(stale_chrome_pids | grep -c .)
  if [ "$DRY" = "1" ]; then
    # --dry promises to change nothing, so it must not reap either: it reports the same candidate
    # count the real run would act on and the decision it would reach.
    log "dry run: ${TASKS}/${MAX_TASKS} >= ceiling ${TASK_CEILING} — would reap ${CANDIDATES} stale chrome process(es), then start an iteration if tasks drop below the ceiling"
    echo "dry run: would reap ${CANDIDATES} stale chrome process(es), then start an iteration if tasks drop below the ceiling"
    exit 0
  fi
  log "skip: cgroup tasks ${TASKS}/${MAX_TASKS} >= ceiling ${TASK_CEILING} — reaping stale harness Chrome first (${CANDIDATES} candidate(s))"
  KILLED=$(reap_stale_chromes)
  sleep 2
  TASKS=$(tasks_now)
  if [ "$TASKS" -ge "$TASK_CEILING" ]; then
    log "skip: still ${TASKS}/${MAX_TASKS} after reaping ${KILLED}"
    exit 0
  fi
  log "reaped ${KILLED}; now ${TASKS}/${MAX_TASKS}"
fi

# --- 3. back off if recent iterations produced nothing -----------------------------------------
# The ratio is over the whole driver log, so it does not move tick to tick: it is announced when the
# band it falls in changes (STATE_STALL holds the last band spoken), not on every tick it is true.
NO_COMMIT=$(grep -c 'no commit produced' "$DRIVER_LOG" 2>/dev/null || true)
STARTS=$(grep -c 'iteration start' "$DRIVER_LOG" 2>/dev/null || true)
[[ "$NO_COMMIT" =~ ^[0-9]+$ ]] || NO_COMMIT=0
[[ "$STARTS" =~ ^[0-9]+$ ]] || STARTS=0
STATE_STALL=/data/run/ralph-watchdog.stall
if [ "$STARTS" -gt 0 ] && [ "$NO_COMMIT" -gt 0 ]; then
  RATIO=$(( NO_COMMIT * 100 / STARTS ))
  BAND=$(( RATIO / 20 ))          # 0-19 -> 0, 20-39 -> 1, 40-59 -> 2, ...
  if [ "$RATIO" -ge 40 ]; then
    log "warn: ${NO_COMMIT}/${STARTS} iterations produced no commit (${RATIO}%) — starting anyway, but check the last logs in $LOGDIR"
    if [ "$(cat "$STATE_STALL" 2>/dev/null || echo none)" != "$BAND" ]; then
      echo "ralph loop warning: ${NO_COMMIT}/${STARTS} recent iterations produced NO commit (${RATIO}%) — see $LOGDIR"
      printf '%s' "$BAND" > "$STATE_STALL" 2>/dev/null
    fi
  else
    printf '%s' "$BAND" > "$STATE_STALL" 2>/dev/null
  fi
fi

# --- 4. the docs server every measurement depends on -------------------------------------------
if ! curl -s -o /dev/null --max-time 8 http://127.0.0.1:3177/; then
  log "docs server on 3177 is down — starting it"
  if [ "$DRY" = "0" ]; then
    (cd "$REPO" && setsid python3 ralph/scripts/serve-docs-app.py --port 3177 >> /tmp/serve-docs-app.log 2>&1 &)
    sleep 2
  fi
fi

# --- start one iteration ------------------------------------------------------------------------
if [ "$DRY" = "1" ]; then
  log "dry run: would start one iteration (tasks ${TASKS}/${MAX_TASKS})"
  echo "dry run: would start one iteration (tasks ${TASKS}/${MAX_TASKS})"
  exit 0
fi

log "starting an iteration (tasks ${TASKS}/${MAX_TASKS})"
echo "iteration start (watchdog) $(stamp)" >> "$DRIVER_LOG"
setsid bash "$HARNESS" >> "$DRIVER_LOG" 2>&1 &
echo "$! $(stamp)" > "$STATE"
# Logged, not announced: one iteration per tick is this watchdog doing its normal job, and a message
# saying so every 15 minutes is the noise the contract above exists to stop. $DRIVER_LOG has the record.
log "started iteration pid $! (tasks ${TASKS}/${MAX_TASKS})"
