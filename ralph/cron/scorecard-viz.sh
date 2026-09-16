#!/usr/bin/env bash
# scorecard-viz.sh — regenerate the port progress dashboard and publish it to progress.baseui.noevaresearch.com
#
# Runs hourly, script-only, NO LLM. Silent by design (cron job is no_agent + deliver=local): a dashboard
# that refreshes hourly must not send an hourly message.
#
# EVERYTHING THIS NEEDS LIVES OUTSIDE THE LOOP'S REPO — deliberately:
#   * the driver's measurement-integrity post-condition (ralph/driver/ralph-baseui-hermes.sh) reverts
#     tooling edits and QUARANTINES newly added files under ralph/scripts, so tooling parked there is
#     moved to /tmp mid-flight (observed 2026-09-16 11:56:30 — the generator was quarantined);
#   * the repo's hourly `git add -- TODO.md ralph/generated ralph/logs specs  # was: git add -A (swept the loop's in-progress source into a bot commit; it now stages only the ledger, generated data, logs and specs it actually owns)` checkpoint cron would otherwise commit dashboard artifacts and
#     this log into the port's history, unattributed and unread.
# So the tooling lives in /data/tools/progress/, the report in /data/progress/, the log in /data/logs/,
# and only the DATA is read from the repo.
#
#   1. regenerate the HTML (pure Python, SVG generated — no Chrome: a headless launch is ~20 processes on
#      a 512-task cgroup and the PNG is only needed when a human looks at it)
#   2. keep the origin alive (the tunnel maps the hostname to the port; publishing IS writing the file)
set -uo pipefail

REPO=/data/workspace/baseui
TOOLS=/data/tools/progress
DOCROOT=/data/progress
GENERATOR=$TOOLS/scorecard-viz.py
PORT=3180
LOG=/data/logs/progress-viz.log
mkdir -p "$DOCROOT" /data/logs

# Same discipline as scorecard-sweep.sh: never become the thing that starves the box.
PIDS_CURRENT=$(cat /sys/fs/cgroup/pids.current 2>/dev/null || echo 0)
PIDS_MAX=$(cat /sys/fs/cgroup/pids.max 2>/dev/null || echo 512)
if [ "$PIDS_CURRENT" -gt $((PIDS_MAX - 60)) ]; then
  echo "$(date -u +%FT%TZ) skip: box under pressure ($PIDS_CURRENT/$PIDS_MAX tasks)" >> "$LOG"
  exit 0
fi

# 1. regenerate — the generator writes /data/progress/index.html atomically (tmp + rename)
OUT=$(cd "$REPO" && timeout 180 python3 "$GENERATOR" 2>&1)
RC=$?
if [ $RC -ne 0 ] || ! echo "$OUT" | grep -q '^wrote '; then
  echo "$(date -u +%FT%TZ) FAILED (rc=$RC): $(echo "$OUT" | tail -3 | tr '\n' ' ')" >> "$LOG"
  # Non-zero ON PURPOSE: with deliver=local nothing is posted, but the run history then shows
  # last_status=error instead of claiming success while the published page quietly goes stale.
  exit 1
fi

# 2. keep the origin alive. The origin is the 7 kB C server, NOT python3 -m http.server: that measured
# 20,028 kB RSS to serve one file, against this box's 515 MB total. This one measures ~1,650 kB, 1 thread.
export PATH="$PATH:/data/.local/bin"
SRV=/data/bin/progress-server
if [ ! -x "$SRV" ]; then
  if command -v cc >/dev/null 2>&1; then
    cc -O2 -s -o "$SRV" "$TOOLS/progress-server.c" \
      && echo "$(date -u +%FT%TZ) rebuilt $SRV from progress-server.c" >> "$LOG" \
      || { echo "$(date -u +%FT%TZ) WARNING: could not build $SRV" >> "$LOG"; exit 0; }
  else
    echo "$(date -u +%FT%TZ) WARNING: $SRV missing and no compiler" >> "$LOG"
    exit 0
  fi
fi

if ! curl -s -o /dev/null --max-time 5 "http://127.0.0.1:$PORT/"; then
  # a stale server of the old shape would hold the port; anchor the pattern so it cannot match this shell
  pkill -f "^python3 -m http\.server $PORT" 2>/dev/null
  sleep 1
  (nohup "$SRV" "$PORT" "$DOCROOT/index.html" >> /data/logs/progress-origin.log 2>&1 &)
  sleep 2
  echo "$(date -u +%FT%TZ) restarted the origin ($SRV) on 127.0.0.1:$PORT" >> "$LOG"
fi

# What the report was drawn from, so a stale dashboard is self-evident from the log alone.
# (scorecard-viz.py already prints this line; the shell greps it rather than re-parsing the JSONL.)
SUMMARY=$(echo "$OUT" | grep -m1 '^routes:' || echo 'routes: ?')
echo "$(date -u +%FT%TZ) published ($SUMMARY) $(echo "$OUT" | head -1)" >> "$LOG"
exit 0
