#!/usr/bin/env node
/* eslint-disable no-console */

// verify-citation-drift.mjs — is a drifted citation a CHANGED ASSERTION, or the same words re-wrapped?
//
// WHY THIS EXISTS
// ---------------
// `check-citations.mjs check` stores only a HASH per cited window, so when a cited file is reflowed (lines
// joined or split) it can only say "content has drifted" — it cannot tell a re-wrap from an edit, because it
// has nothing to compare against. Its own recovery path (`findUniqueNearbyDrift`) searches ±40 LINES for the
// recorded window, which a reflow defeats: the window moved by a few lines and changed its line count.
//
// This recovers the MISSING ORACLE from git: the recorded hash was computed at the cited range ± WINDOW_MARGIN
// on some historical revision of the cited file, so walk that file's history until a revision hashes to the
// recorded value. That revision holds the exact text the spec cited, and the question "did the assertion
// change?" becomes mechanical — the recorded text, whitespace-normalised, either is still present at the cited
// range (a re-wrap: safe to re-record) or it is not (a genuine change: the spec must be checked by hand).
//
// Usage: node ralph/scripts/verify-citation-drift.mjs [--cited TODO.md] [--max-revs 80]

import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const WINDOW_MARGIN = 2; // must match check-citations.mjs
const arg = (n, d) => {
  const i = process.argv.indexOf(`--${n}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : d;
};
const CITED = arg('cited', 'TODO.md');
const MAX_REVS = Number(arg('max-revs', '80'));

const hashWindow = (lines, start, end) =>
  crypto.createHash('sha256').update(lines.slice(Math.max(0, start - 1 - WINDOW_MARGIN), Math.min(lines.length, end + WINDOW_MARGIN)).join('\n')).digest('hex');
const norm = (s) => s.replace(/\s+/g, ' ').trim();

// ---- every sidecar under specs/ ------------------------------------------------------------------------
const sidecars = [];
(function walk(dir) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p);
    else if (e.name.endsWith('.citations.json')) sidecars.push(p);
  }
})(path.join(ROOT, 'specs'));

const current = fs.readFileSync(path.join(ROOT, CITED), 'utf8').split('\n');
const drifted = [];
for (const sidecar of sidecars) {
  const recorded = JSON.parse(fs.readFileSync(sidecar, 'utf8'));
  for (const [key, hash] of Object.entries(recorded)) {
    const m = key.match(/^(.*):(\d+)-(\d+)$/);
    if (!m || m[1] !== CITED) continue;
    const start = Number(m[2]);
    const end = Number(m[3]);
    if (hashWindow(current, start, end) === hash) continue; // not drifted
    drifted.push({ sidecar: path.relative(ROOT, sidecar), key, start, end, hash });
  }
}
console.log(`${drifted.length} drifted ${CITED} citation(s)`);

// ---- recover the recorded text from the revision that recorded it --------------------------------------
const revs = execFileSync('git', ['log', `--max-count=${MAX_REVS}`, '--format=%H', '--', CITED], { cwd: ROOT, encoding: 'utf8' })
  .split('\n').filter(Boolean);
const cache = new Map();
const linesAt = (rev) => {
  if (!cache.has(rev)) cache.set(rev, execFileSync('git', ['show', `${rev}:${CITED}`], { cwd: ROOT, encoding: 'utf8' }).split('\n'));
  return cache.get(rev);
};

let safe = 0;
let changed = 0;
for (const d of drifted) {
  let oracleText = null;
  let oracleRev = null;
  for (const rev of revs) {
    let lines;
    try { lines = linesAt(rev); } catch { continue; }
    if (d.end > lines.length) continue;
    if (hashWindow(lines, d.start, d.end) === d.hash) { oracleText = lines.slice(d.start - 1, d.end).join('\n'); oracleRev = rev.slice(0, 9); break; }
  }
  const currentWindow = norm(current.slice(d.start - 1, d.end).join('\n'));
  if (oracleText === null) {
    changed += 1;
    console.log(`  CHANGED? ${d.key} — recorded revision not found in ${revs.length} rev(s); needs a human`);
    continue;
  }
  const oracle = norm(oracleText);
  // A re-wrap keeps every word; the range covers the same text (possibly with different line breaks), so the
  // recorded text must still be a substring of the cited range, or the cited range a substring of it.
  const sameText = currentWindow.includes(oracle) || oracle.includes(currentWindow);
  if (sameText) {
    safe += 1;
    console.log(`  RE-WRAP  ${d.key} (recorded at ${oracleRev}) — same words, ${Math.abs(currentWindow.length - oracle.length)} char diff in wrapping; safe to re-record`);
  } else {
    changed += 1;
    console.log(`  CHANGED  ${d.key} (recorded at ${oracleRev})\n    recorded: ${oracle.slice(0, 150)}\n    now:      ${currentWindow.slice(0, 150)}`);
  }
}
console.log(`\n${safe} re-wrap(s) — safe to re-record; ${changed} possible change(s) — read these by hand`);
process.exit(changed ? 1 : 0);
