#!/usr/bin/env bash
# memory-guard.sh — the loop's real constraint is 4 GB, and it does not know that.
#
# MEASURED 2026-09-16 (cgroup v2, this sandbox):
#   memory.max      4096 MB
#   memory.current  3602 MB at rest          (88% with nothing running)
#   memory.peak     4100 MB                  → the ceiling HAS been hit
#   memory.events   oom_kill 51              → 51 kills, and 13 iteration logs show fork/alloc failures
# The budget, per unit:
#   chromium (one instance, 10 processes)  ~1400 MB
#   rustc/wasm build                       ~1500 MB
#   node harness + shell                   ~ 500 MB
#   -----------------------------------------------
#   total                                  ~3400 MB  against 4096 MB
# So a build overlapping a measurement is an OOM kill, the killed command is usually the agent's own tool call,
# and the iteration dies mid-step — which looks exactly like "the loop stopped moving" and was misdiagnosed as
# model slowness, gate strictness and picker livelock before anyone looked at memory.
#
# RULE: never build and measure at the same time. Serialize them, keep ONE chromium, and check the budget before
# starting new work. Usage:
#   memory-guard.sh            # report + reap orphans; exit 0 ok, 3 critical
#   memory-guard.sh --preflight # exit 3 if an iteration must NOT start now
set -uo pipefail

CG=/sys/fs/cgroup
read_mb() { awk -v f="$1" 'BEGIN{printf "%.0f", f/1048576}'; }
LIMIT=$(cat $CG/memory.max 2>/dev/null || echo 0)
CUR=$(cat $CG/memory.current 2>/dev/null || echo 0)
[ "$LIMIT" = "max" ] && LIMIT=0
if [ "${LIMIT:-0}" -gt 0 ] 2>/dev/null; then
  PCT=$(( CUR * 100 / LIMIT ))
else
  PCT=0
fi

# Reap orphaned browsers: a Chrome reparented to PID 1 has no harness left to kill it, so it holds ~1.4 GB until
# something does. This is safe by construction — the harness mutex means only one browser is ever live, and a
# live one always has a parent.
REAPED=0
for p in $(ps -eo pid,ppid,cmd --no-headers 2>/dev/null | grep '[c]hrome-linux64/chrome' | awk '$2==1 {print $1}'); do
  kill -9 "$p" 2>/dev/null && REAPED=$((REAPED+1))
done
sleep 1
CUR=$(cat $CG/memory.current 2>/dev/null || echo 0)
PCT_AFTER=$( [ "${LIMIT:-0}" -gt 0 ] && echo $(( CUR * 100 / LIMIT )) || echo 0 )

printf 'memory: %sMB / %sMB (%s%%)' "$(read_mb "$CUR")" "$(read_mb "$LIMIT")" "$PCT_AFTER"
[ "$REAPED" -gt 0 ] && printf '  [reaped %s orphan browser process(es)]' "$REAPED"
printf '\n'

if [ "${PCT_AFTER:-0}" -ge 90 ]; then
  echo "memory-guard: CRITICAL — no new iteration should start (a build plus a measurement cannot fit)"
  exit 3
fi
if [ "${PCT_AFTER:-0}" -ge 75 ]; then
  echo "memory-guard: TIGHT — proceed only if nothing is building; keep exactly one browser"
fi
exit 0
