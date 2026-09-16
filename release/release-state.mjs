#!/usr/bin/env node
// release/release-state.mjs — decides whether THIS push owes a crates.io release, and which version.
//
// The cadence rule (user decision, 2026-09-16): one patch release per 10 commits, and ONLY from a
// commit whose workspace builds the published artifacts green (the workflow enforces the gate; this
// script only answers "is a release due, and what number?").
//
// Why this is a script and not `$((count % 10))` in YAML, in three parts:
//
//  1. THE COUNTER NEEDS AN ORIGIN. `count % 10 == 0` against all 5000+ commits of this repo's
//     history would fire on nearly every push forever. The origin is the commit that ADDED
//     release/release.json — discovered from git itself (`--diff-filter=A`), so activating the
//     counter needs no edit here, and resetting it is `git rm` + re-add in one commit.
//
//  2. THE COUNT MUST BE MEASURED AGAINST CARRYABLE STATE, NOT A PUSH BOUNDARY. The hourly automation
//     pushes BATCHES of commits (13 at once is ordinary), so a push can step straight over any
//     exact multiple of 10. Comparing "how many 10-commit windows have elapsed" against "how many
//     releases actually exist on the registry" means a window that was skipped is picked up by the
//     NEXT push instead of being silently lost.
//
//  3. THE REGISTRY IS THE STATE STORE, DELIBERATELY. Nothing is written back to the repo (a
//     workflow that commits its own version bump would feed the commit counter that triggered it —
//     a release that triggers its own successor). crates.io already knows what has been published,
//     so the plan is idempotent: re-running the same commit's workflow cannot double-publish.
//
// Usage:
//   node release/release-state.mjs                 # human summary + JSON to stdout
//   node release/release-state.mjs --json          # JSON only
//   node release/release-state.mjs --force         # a release is due regardless of cadence
//   node release/release-state.mjs --github-output # append plan to $GITHUB_OUTPUT

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '..');
const BASELINE_FILE = 'release/release.json';
const COMMITS_PER_RELEASE = 10;
const REGISTRY = 'https://crates.io/api/v1/crates';
const USER_AGENT = 'noevaresearch-base-ui-release (https://github.com/noevaresearch/base-ui)';

/// Publish order matters: a crate's published deps must already exist in the index.
export const CRATES = ['base-ui-leptos-utils', 'base-ui-leptos-internals', 'base-ui-leptos'];

const args = new Set(process.argv.slice(2));
const force = args.has('--force');

function git(...argv) {
  return execFileSync('git', argv, { cwd: ROOT, encoding: 'utf8' }).trim();
}

/// The commit that introduced this automation. Editing the file does not move it (only ADDING does).
function baselineSha() {
  const sha = git('log', '--diff-filter=A', '-1', '--format=%H', '--', BASELINE_FILE);
  if (!sha) {
    throw new Error(
      `cannot find the commit that added ${BASELINE_FILE} — the release counter has no origin. ` +
        `Note: the workflow must check out with fetch-depth: 0, or this lookup finds nothing.`,
    );
  }
  return sha;
}

/// Highest 0.1.<patch> published for a crate, or 0 when the crate does not exist yet.
async function publishedPatch(name) {
  const res = await fetch(`${REGISTRY}/${name}`, { headers: { 'User-Agent': USER_AGENT } });
  if (res.status === 404) return { name, patch: 0, state: 'not-published' };
  if (!res.ok) throw new Error(`crates.io lookup for ${name} failed: HTTP ${res.status}`);
  const body = await res.json();
  const versions = (body.versions ?? []).map((v) => v.num);
  const patches = versions
    .map((v) => /^0\.1\.(\d+)$/.exec(v))
    .filter(Boolean)
    .map((m) => Number(m[1]));
  return {
    name,
    patch: patches.length ? Math.max(...patches) : 0,
    state: versions.length ? 'published' : 'not-published',
    // A version outside the 0.1.x line would mean the patch counter is no longer the whole story.
    offSchemeVersions: versions.filter((v) => !/^0\.1\.\d+$/.test(v)),
  };
}

const baseline = baselineSha();
const commitsSinceBaseline = Number(git('rev-list', '--count', `${baseline}..HEAD`));
const dueReleases = Math.floor(commitsSinceBaseline / COMMITS_PER_RELEASE);

const registry = [];
for (const name of CRATES) registry.push(await publishedPatch(name));

const maxPatch = Math.max(...registry.map((r) => r.patch));
// A release is a UNIT of all three crates at one version. If they disagree, a previous run died
// mid-publish (real case 2026-09-16: utils and internals reached 0.1.1, the cancel killed the run
// before the component crate). That version must be COMPLETED, not skipped — advancing to the next
// patch instead would leave 0.1.1 permanently missing its main crate, and consumers resolving
// `base-ui-leptos = "0.1.1"` would get a dependency graph that never existed.
const incompleteRelease = registry.some((r) => r.patch < maxPatch);
const targetPatch = incompleteRelease ? maxPatch : maxPatch + 1;
const releasesDone = maxPatch;
const offScheme = registry.flatMap((r) => r.offSchemeVersions ?? []);
const nextVersion = `0.1.${targetPatch}`;

// Which crates still need the target version? On a fresh release that is all of them; on a resumed
// partial release it is exactly the missing ones.
const publish = Object.fromEntries(registry.map((r) => [r.name, r.patch < targetPatch]));

const cadenceDue = dueReleases > maxPatch;
// A half-published release is retried by the next push regardless of cadence: an incomplete version
// on a registry is worse than an early one, because it is broken for anyone who resolves it.
const shouldPublish = force || cadenceDue || incompleteRelease;

const plan = {
  baseline_sha: baseline,
  commits_since_baseline: commitsSinceBaseline,
  commits_per_release: COMMITS_PER_RELEASE,
  due_releases: dueReleases,
  releases_done: releasesDone,
  releases_behind: Math.max(0, dueReleases - releasesDone),
  next_version: nextVersion,
  should_publish: shouldPublish,
  forced: force,
  publish,
  publish_order: CRATES.filter((c) => publish[c]),
  registry,
  off_scheme_versions: offScheme,
};

if (args.has('--json')) {
  console.log(JSON.stringify(plan, null, 2));
} else if (!args.has('--github-output')) {
  console.log(`baseline          ${baseline.slice(0, 10)} (${commitsSinceBaseline} commits ago)`);
  console.log(`cadence           ${dueReleases} release(s) due, ${releasesDone} published → ${plan.releases_behind} behind`);
  for (const r of registry) console.log(`registry          ${r.name}: ${r.state}, max patch ${r.patch}`);
  console.log(`next version      ${nextVersion}${force ? '  (forced)' : ''}`);
  console.log(`decision          ${shouldPublish ? `PUBLISH ${plan.publish_order.join(', ')}` : 'nothing due — no publish'}`);
  if (offScheme.length) console.log(`WARNING           versions outside 0.1.x exist: ${offScheme.join(', ')}`);
  if (!shouldPublish && !force) console.log(`\nnext release lands ${COMMITS_PER_RELEASE - (commitsSinceBaseline % COMMITS_PER_RELEASE)} commit(s) from here`);
}

if (args.has('--github-output')) {
  const out = process.env.GITHUB_OUTPUT;
  if (!out) throw new Error('--github-output requires $GITHUB_OUTPUT');
  fs.appendFileSync(
    out,
    Object.entries({
      should_publish: plan.should_publish,
      next_version: plan.next_version,
      publish_order: plan.publish_order.join(','),
      commits_since_baseline: plan.commits_since_baseline,
      releases_done: plan.releases_done,
      releases_behind: plan.releases_behind,
      baseline_sha: plan.baseline_sha,
    })
      .map(([k, v]) => `${k}=${v}\n`)
      .join(''),
  );
}
