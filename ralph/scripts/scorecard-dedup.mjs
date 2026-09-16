#!/usr/bin/env node
/* eslint-disable no-console */

// scorecard-dedup.mjs — ONE measurement per route, and the count is routes, not lines.
//
// WHY THIS EXISTS
// ---------------
// The CI aggregate step appended each shard's raw output to `ralph/generated/scorecard.jsonl` and then reported
// "N routes measured" where N was a LINE count. The file is an append-log, so the second run re-appended everything
// and the report said "1037 route(s) measured" for a repository with 17 mirrored routes. A number that cannot be
// wrong in the direction of "more" is not a measurement — and this is the same defect class the whole day has been
// about: a measure printing something other than what it claims.
//
// The rules:
//   * a route's NEWEST measurement wins (later generatedAt; ties broken by keeping the last one seen, since the
//     append order is the measurement order);
//   * entries that are not a well-formed scorecard record are DROPPED and counted, never silently kept — an
//     unparseable line is evidence of a broken writer, not a route;
//   * the output is stable (sorted by route) so a re-run of an unchanged input produces an identical file, which
//     is what lets CI decide "no change to write back" honestly.
//
// Usage: node ralph/scripts/scorecard-dedup.mjs [--check]
//   --check  exit 1 if the file is not already deduplicated (for a gate), without writing.

import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const FILE = path.join(ROOT, 'ralph/generated/scorecard.jsonl');
const checkOnly = process.argv.includes('--check');

if (!fs.existsSync(FILE)) {
  console.log('scorecard-dedup: no scorecard.jsonl yet — nothing to do');
  process.exit(0);
}

const raw = fs.readFileSync(FILE, 'utf8').split('\n').filter((l) => l.trim());
const byRoute = new Map();
const malformed = [];

for (const line of raw) {
  let rec;
  try {
    rec = JSON.parse(line);
  } catch {
    malformed.push(line.slice(0, 80));
    continue;
  }
  const route = rec.route;
  if (!route || !Array.isArray(rec.axes)) {
    malformed.push(JSON.stringify(rec).slice(0, 80));
    continue;
  }
  const prev = byRoute.get(route);
  // Newest wins. `>=` (not `>`) because equal timestamps mean the later line is the later measurement.
  if (!prev || String(rec.generatedAt ?? '') >= String(prev.generatedAt ?? '')) byRoute.set(route, rec);
}

const out = [...byRoute.keys()].sort().map((r) => JSON.stringify(byRoute.get(r))).join('\n') + '\n';
const current = fs.readFileSync(FILE, 'utf8');

console.log(`scorecard-dedup: ${raw.length} line(s) → ${byRoute.size} route(s)`);
if (malformed.length) console.log(`  dropped ${malformed.length} malformed line(s), first: ${malformed[0]}`);
console.log(`  newest measurement: ${[...byRoute.values()].map((r) => r.generatedAt).sort().pop()}`);

if (checkOnly) {
  if (current !== out) {
    console.error('scorecard-dedup: FILE IS NOT DEDUPLICATED (or not stable) — run without --check to fix');
    process.exit(1);
  }
  console.log('  file is deduplicated and stable');
  process.exit(0);
}
if (current === out) {
  console.log('  already clean — no write');
  process.exit(0);
}
fs.writeFileSync(FILE, out);
console.log(`  written: ${out.split('\n').filter(Boolean).length} record(s), one per route`);
