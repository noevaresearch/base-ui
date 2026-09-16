#!/usr/bin/env bash
# release-watchdog.sh — the crate's feedback loop, mirroring the website's deploy watchdog.
#
# WHY: the docs site gets its feedback from CI deploy + baseui-deploy-watchdog. Publishing had none: the
# publish workflow ran several times (failures, one in flight) and nothing anywhere said so — it was found by
# accident while answering an unrelated question, because PUBLISHING IS NOT A PARITY AXIS and no harness gate
# could have caught it. This is that missing channel: it reports the registry's real state (not what the repo
# believes), and it stays silent when there is nothing to say.
#
# It reports, in one line, only when something is worth knowing:
#   * the version the registry actually serves vs the version the repo intends to release (drift/mismatch);
#   * whether a publish attempt failed or is stuck (a red or hung workflow run);
#   * a first publication that just appeared (so it is never a surprise).
#
# Usage: release-watchdog.sh            # silent when healthy
set -uo pipefail
cd /data/workspace/baseui
export PATH=/data/bin:$PATH
STATE=ralph/generated/release-watchdog.json
mkdir -p ralph/generated
REPO=noevaresearch/base-ui

# What the repo intends to release (the crate's declared version).
# The crate inherits its version (`version.workspace = true`), so read the WORKSPACE manifest and, if the
# crate overrides it, prefer that. Reading only the crate manifest produced the nonsense "version.workspace =
# true" as a version — the kind of wrong value that then gets reported as drift.
LOCAL=$(grep -m1 -A 2 '^\[package\]' crates/leptos-ui/Cargo.toml 2>/dev/null | grep -m1 '^version' | cut -d'"' -f2)
case "$LOCAL" in
  ""|*workspace*) LOCAL=$(grep -m1 -A 8 '^\[workspace.package\]' Cargo.toml 2>/dev/null | grep -m1 '^version' | cut -d'"' -f2)
esac
[ -z "$LOCAL" ] && LOCAL="?"
CRATE=$(grep -m1 '^name' crates/leptos-ui/Cargo.toml 2>/dev/null | cut -d'"' -f2 || echo "?")
[ "$LOCAL" = "?" ] && { echo "release-watchdog: cannot read the crate manifest — skipping"; exit 0; }

# What the registry actually serves. Authoritative and EXPLICIT: the API returns 404 for a crate that has
# never been published, and a network hiccup must not be reported as either presence or absence — so the three
# outcomes (version / not-published / unknown) are distinct, and only the first can produce drift.
HTTP=$(curl -s -o /tmp/rw-crate.json -w '%{http_code}' -A 'release-watchdog (noevaresearch)' --max-time 20 "https://crates.io/api/v1/crates/$CRATE" 2>/dev/null || echo 000)
case "$HTTP" in
  200) SERVED=$(node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{try{console.log(JSON.parse(s).crate.max_version)}catch{console.log("unknown")}})' < /tmp/rw-crate.json 2>/dev/null);;
  404) SERVED="not-published";;
  *)   SERVED="unknown";;
esac
[ -z "$SERVED" ] && SERVED="unknown"

# Last publish workflow run: a red or stuck run is the thing a human needs told.
RUN=$(gh run list --repo "$REPO" --workflow=publish-crates.yml --limit 1 --json status,conclusion,createdAt,url 2>/dev/null | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{try{const r=JSON.parse(s)[0]||{};console.log(`${r.status}|${r.conclusion || "-"}|${(r.createdAt||"").slice(11,16)}|${r.url||""}`)}catch{console.log("-|-|-|-")}})')

PREV="{}"
[ -f "$STATE" ] && PREV=$(cat "$STATE" 2>/dev/null || echo "{}")
python3 - "$STATE" "$LOCAL" "$SERVED" "$RUN" "$PREV" "$CRATE" <<'PY'
import json, sys, datetime, re
state_path, local, served, run, prev, crate = sys.argv[1:7]
status, conclusion, when, url = (run.split('|') + ['-', '-', '-'])[:4]
prev = json.loads(prev or '{}')
now = datetime.datetime.now(datetime.timezone.utc).isoformat()
lines = []
if prev.get('served') != served:
    if served == 'not-published':
        lines.append(f"release-watchdog: nothing on crates.io for `{crate}` yet (repo declares {local}) — first publish still pending")
    elif served == 'unknown':
        lines.append(f"release-watchdog: could not read crates.io for `{crate}` this run (unknown state, not a verdict)")
    else:
        lines.append(f"release-watchdog: crates.io now serves {crate} {served} (repo declares {local})")
semver = re.compile(r'^\d+\.\d+\.\d+')
# Asymmetric on purpose. The pipeline stamps the release version at publish time and does not commit it back,
# so the registry is normally AHEAD of the declared version — reporting that as drift would be noise every
# 30 minutes. The direction that matters is the repo claiming a version the registry does not serve: pages
# and docs would then tell readers to install something that does not exist.
def semtuple(v):
    try: return tuple(int(x) for x in str(v).split('.')[:3])
    except Exception: return None
if served not in ('not-published', 'unknown', '-') and semver.match(local or ''):
    a, b = semtuple(local), semtuple(served)
    if a and b and a > b:
        lines.append(f"release-watchdog: DRIFT — the repo declares {local} but crates.io serves only {served}: nothing published that version, so any doc naming it is wrong")
    elif a and b and a < b:
        if prev.get('lastNote') != 'ahead':      # report the transition once, then stay quiet
            lines.append(f"release-watchdog: published release ahead of the repo (registry {served} > declared {local}) — normal for this pipeline, no action")
        prev['lastNote'] = 'ahead'
    else:
        prev['lastNote'] = None
if status == 'completed' and conclusion == 'failure':
    lines.append(f"release-watchdog: last publish run FAILED at {when} — {url}")
if status == 'in_progress' and prev.get('inProgressSince') and prev.get('inProgressSince') != when:
    lines.append(f"release-watchdog: a publish run has been in flight since {prev.get('inProgressSince')} — {url}")
json.dump({'served': served, 'local': local, 'lastRun': run,
           'inProgressSince': when if status == 'in_progress' else None,
           # lastNote MUST be persisted: it was computed but never written, so the "release ahead of the repo"
           # line repeated on every 30-minute run — a watchdog that repeats itself is noise, and noise is how a
           # real alert gets ignored.
           'lastNote': prev.get('lastNote'), 'updatedAt': now}, open(state_path, 'w'))
print("\n".join(lines))
PY
