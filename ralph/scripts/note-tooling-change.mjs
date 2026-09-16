#!/usr/bin/env node
/* eslint-disable no-console */
// note-tooling-change.mjs — make an iteration's edits to the measurement tooling VISIBLE in the ledger.
//
// A post-condition can only revert UNCOMMITTED tooling edits. An iteration that edits a gate and COMMITS it
// under its own item message passes that check silently — which happened (2026-09-16, three harness scripts
// in one item commit). The edit itself was sound that time (the surface gate never ran for `library:` items
// because it sat inside a docs-app guard), but "sound this time" is not a process: a gate change must be
// attributable and reviewable, so the driver calls this to append a note the next iteration will read.
//
// Usage: note-tooling-change.mjs "<todo id>" "<sha>" "<files, comma separated>"
import fs from 'node:fs';
const [id, sha, files] = process.argv.slice(2);
if (!id || !sha) { console.error('usage: note-tooling-change.mjs "<todo id>" "<sha>" "<files>"'); process.exit(2); }
const p = 'TODO.md';
let todo = fs.readFileSync(p, 'utf8');
const start = todo.indexOf(`- [ ] ${id}`);
const alt = start < 0 ? todo.indexOf(`- [x] ${id}`) : start;
if (alt < 0) { console.error(`note-tooling-change: no ledger item "${id}"`); process.exit(1); }
let end = todo.indexOf('\n- [', alt + 5);
if (end < 0) end = todo.length;
const stamp = new Date().toISOString().slice(0, 16).replace('T', ' ');
const note = `\n      review-note: MEASUREMENT TOOLING CHANGED in this iteration's own commit ${sha.slice(0, 10)} — ${files || '(see the commit)'}. A gate edit is not self-authorising: it needs review as a tooling change (what it now measures, and whether the bar it enforces moved). Recorded by the driver so the next iteration sees it rather than inheriting a quietly different gate.`;
todo = todo.slice(0, end) + note + todo.slice(end);
fs.writeFileSync(p, todo);
console.log(`noted tooling change for "${id}" at ${sha.slice(0, 10)}`);
