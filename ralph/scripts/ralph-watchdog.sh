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
#      which starves the gateway and makes every tool call fail in its pre-call hook
#   3. no commit in the last N attempts -> back off and report, so a wedged loop is visible rather
#      than silently burning model spend
#   4. docs server down                -> start it, since every fidelity/ergonomics measurement needs it
#
# Usage: bash /data/scripts/ralph-watchdog.sh          (cron: every 15 minutes)
#        bash /data/scripts/ralph-watchdog.sh --dry    (report what it would do, start nothing)

set -uo pipefail
REPO=/data/workspace/baseui
LOGDIR="$REPO/ralph/logs/stage3"
DRIVER_LOG="$LOGDIR/driver.log"
WATCHDOG_LOG="$LOGDIR/watchdog.log"
STATE=/data/run/ralph-watchdog.state
HARNESS=/data/scripts/ralph-baseui-hermes.sh
TASK_CEILING=420
MAX_TASKS=512
DRY=0
[ "${1:-}" = "--dry" ] && DRY=1

mkdir -p "$LOGDIR" /data/run
stamp() { date -u +%Y-%m-%dT%H:%M:%SZ; }

log() { echo "$(stamp) $*" >> "$WATCHDOG_LOG"; }

# --- 1. is an iteration already running? -------------------------------------------------------
if pgrep -f "ralph-baseui-hermes.sh" >/dev/null 2>&1; then
  log "skip: an iteration is already running (pid $(pgrep -f 'ralph-baseui-hermes.sh' | head -1))"
  exit 0
fi

# --- 2. resource pressure ----------------------------------------------------------------------
TASKS=$(cat /sys/fs/cgroup/pids.current 2>/dev/null || echo 0)
if [ "$TASKS" -ge "$TASK_CEILING" ]; then
  log "skip: cgroup tasks ${TASKS}/${MAX_TASKS} >= ceiling ${TASK_CEILING} — reaping stale harness Chrome first"
  # the harness reaps its own leftovers; do the same here so the next tick can run
  ps -eo pid,args | awk '/[c]hrome-linux64/ && /user-data-dir=\/tmp\/(visdiff|gaprep|sniperg|live|cdp|anc)/{print $1}' | xargs -r kill -9 2>/dev/null
  sleep 2
  TASKS=$(cat /sys/fs/cgroup/pids.current 2>/dev/null || echo 0)
  if [ "$TASKS" -ge "$TASK_CEILING" ]; then
    log "skip: still ${TASKS}/${MAX_TASKS} after reaping"
    exit 0
  fi
  log "reaped; now ${TASKS}/${MAX_TASKS}"
fi

# --- 3. back off if recent iterations produced nothing -----------------------------------------
NO_COMMIT=$(grep -c 'no commit produced' "$DRIVER_LOG" 2>/dev/null || echo 0)
STARTS=$(grep -c 'iteration start' "$DRIVER_LOG" 2>/dev/null || echo 0)
if [ "$STARTS" -gt 0 ] && [ "$NO_COMMIT" -gt 0 ]; then
  RATIO=$(( NO_COMMIT * 100 / STARTS ))
  if [ "$RATIO" -ge 40 ]; then
    log "warn: ${NO_COMMIT}/${STARTS} iterations produced no commit (${RATIO}%) — starting anyway, but check the last logs in $LOGDIR"
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
echo "started iteration pid $! (tasks ${TASKS}/${MAX_TASKS})"
