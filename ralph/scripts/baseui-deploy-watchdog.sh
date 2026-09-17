#!/usr/bin/env bash
# baseui-deploy-watchdog.sh — alerts ONLY when the docs-site deploy pipeline is broken.
#
# Prints nothing while healthy: the cron runs this with no_agent, so its stdout is delivered
# verbatim and an empty stdout is a silent tick. It watches the pipeline that publishes the
# Leptos port's docs site (https://baseui.noevaresearch.com, built and uploaded by
# .github/workflows/deploy-docs-app.yml). Three failure modes, every one of them otherwise
# silent — a green build yesterday and a broken pipeline today look identical from outside:
#
#   1. the newest run of the deploy workflow FAILED
#   2. the live site is serving an older crates-touching commit than the branch has, with no
#      deploy in flight for the newer one — i.e. a deploy that never ran
#   3. the live site does not answer 200
#
# It alerts once per distinct cause (fingerprint kept in a state file) rather than repeating the
# same line every tick, and prints one line when the pipeline recovers. Read-only with respect to
# the port loop: a `git fetch`, never a repo write.
#
# WHY THE DEDUPE DID NOT ACTUALLY DEDUPE (fixed 2026-09-17)
# --------------------------------------------------------
# The fingerprint WAS the alert text, and problem 2 embeds its own age — "... (227m old)" — so the
# text differed on every tick, `current != previous` was always true, and the same standing failure
# was re-delivered every 30 minutes to two channels (deliver was `origin,all`). A dedupe key that
# contains a clock never matches twice. The fingerprint is now the text with every elapsed-minutes
# count normalised to `(Nm)`, so the same cause hashes stable for as long as it lasts; the message
# the user reads still carries the real age.
#
# Test hooks (used to verify the alert paths without breaking anything real):
#   WATCH_FAKE_RUN='id|status|conclusion|headSha|createdAt|event'   WATCH_FAKE_HTTP=503
#   WATCH_FAKE_LIVE='<sha>'   WATCH_STATE=/tmp/x.state   STALE_MINUTES=90
set -uo pipefail

GH=/data/bin/gh
REPO=noevaresearch/base-ui
WORKFLOW=deploy-docs-app.yml
BRANCH=migration-to-rust
LIVE=https://baseui.noevaresearch.com
REPO_DIR=/data/workspace/baseui
STATE="${WATCH_STATE:-/data/.baseui-deploy-watchdog.state}"
STALE_MINUTES="${STALE_MINUTES:-90}"

problems=()

# ---- 3. is the site up? ---------------------------------------------------------------------
if [ -n "${WATCH_FAKE_HTTP:-}" ]; then
  http="$WATCH_FAKE_HTTP"
else
  http="$(curl -sS -m 20 -o /dev/null -w '%{http_code}' "$LIVE/" 2>/dev/null || echo 000)"
fi
if [ "$http" != "200" ]; then
  problems+=("the live site did not answer 200 (got '$http') — $LIVE")
fi

if [ -n "${WATCH_FAKE_LIVE:-}" ]; then
  live_sha="$WATCH_FAKE_LIVE"
else
  live_sha="$(curl -sS -m 20 "$LIVE/version.json" 2>/dev/null \
    | sed -n 's/.*"commit"[[:space:]]*:[[:space:]]*"\([0-9a-f]\{7,40\}\)".*/\1/p')"
fi

# ---- 1. did the newest deploy run fail? ------------------------------------------------------
if [ -n "${WATCH_FAKE_RUN:-}" ]; then
  run="$WATCH_FAKE_RUN"
else
  run="$("$GH" run list --repo "$REPO" --workflow "$WORKFLOW" --limit 1 \
    --json databaseId,status,conclusion,headSha,createdAt,event \
    --jq '.[0] | "\(.databaseId)|\(.status)|\(.conclusion)|\(.headSha)|\(.createdAt)|\(.event)"' 2>/dev/null)"
fi
IFS='|' read -r run_id run_status run_conc run_sha run_at run_event <<<"${run:-|||||}"
if [ "${run_conc:-}" = "failure" ]; then
  failed_step="$("$GH" run view "${run_id:-0}" --repo "$REPO" --json jobs \
    --jq '[.jobs[].steps[] | select(.conclusion=="failure") | .name] | join(", ")' 2>/dev/null)"
  problems+=("deploy run $run_id FAILED at [${failed_step:-unknown step}] — https://github.com/$REPO/actions/runs/$run_id")
fi

# ---- 2. is the site serving the branch's newest deployable commit? ---------------------------
# "Deployable" = the paths the workflow triggers on (crates/**, Cargo.toml, Cargo.lock, and the
# workflow file itself). Comparing against crates/ alone raises a false alarm whenever the last
# deployable commit was the workflow file.
git -C "$REPO_DIR" fetch -q nr "$BRANCH" 2>/dev/null
trigger_line="$(git -C "$REPO_DIR" log -1 --format='%H|%ct' "nr/$BRANCH" \
  -- crates Cargo.toml Cargo.lock .github/workflows/deploy-docs-app.yml 2>/dev/null)"
IFS='|' read -r head_sha head_ct <<<"${trigger_line:-|}"
age_min=""
if [ -n "${head_sha:-}" ] && [ -n "${live_sha:-}" ]; then
  age_min=$(( ( $(date -u +%s) - head_ct ) / 60 ))
  # A deploy for that exact commit explains the gap — in flight, queued, or just finished. Only
  # no deploy at all is a problem.
  if [ "$live_sha" != "$head_sha" ] && [ "$age_min" -ge "$STALE_MINUTES" ] && [ "${run_sha:-}" != "$head_sha" ]; then
    problems+=("the site serves ${live_sha:0:9} but $BRANCH's newest deployable commit is ${head_sha:0:9} (${age_min}m old) with no deploy run for it")
  fi
fi

# A deploy that reported SUCCESS but did not reach the site: an upload, edge-cache or cert
# mismatch is invisible from the Actions tab, and it is exactly how a green pipeline can serve a
# stale page for days. Allow a few minutes for propagation before calling it.
if [ "${run_conc:-}" = "success" ] && [ -n "${run_sha:-}" ] && [ -n "${live_sha:-}" ] && [ "$run_sha" != "$live_sha" ]; then
  run_age_min=$(( ( $(date -u +%s) - $(date -u -d "${run_at:-1970-01-01T00:00:00Z}" +%s 2>/dev/null || echo 0) ) / 60 ))
  if [ "$run_age_min" -ge 15 ]; then
    problems+=("deploy run $run_id reported SUCCESS for ${run_sha:0:9} but the site still serves ${live_sha:0:9} (${run_age_min}m later)")
  fi
fi

# ---- report -----------------------------------------------------------------------------------
current="$(printf '%s\n' "${problems[@]+"${problems[@]}"}" | sed '/^$/d')"
# The dedupe key, not the message: elapsed minutes are a clock, and a clock in the key means the key
# never repeats. Normalise them away; the text below keeps them for the human reading it.
key="$(printf '%s' "$current" | sed -E 's/\([0-9]+m (old|later)\)/(Nm \1)/g')"
previous="$(cat "$STATE" 2>/dev/null || true)"

if [ -z "$current" ]; then
  if [ -n "$previous" ]; then
    echo "Base UI docs deploy RECOVERED — $LIVE"
    echo "live commit: ${live_sha:0:9} | last run: ${run_id:-?} ${run_status:-?}/${run_conc:-?}"
  fi
  : > "$STATE"
  exit 0
fi

if [ "$key" = "$previous" ]; then
  exit 0   # same cause as the last alert: do not repeat it every tick
fi

echo "Base UI docs deploy needs attention — $LIVE"
printf '%s\n' "$current" | sed 's/^/  - /'
echo "live commit: ${live_sha:0:9} (${live_sha:+ok}) | branch: $BRANCH @ ${head_sha:0:9} (${age_min:-?}m old)"
echo "last run: ${run_id:-?} ${run_status:-?}/${run_conc:-?} (${run_at:-?})"
printf '%s' "$key" > "$STATE"
exit 0
