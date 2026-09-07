#!/usr/bin/env node
/* eslint-disable no-console */

// Forcibly flips one TODO.md item back to `status: blocked` (unchecking its box if needed) and
// records why, in a `note:` field. Used by classralph.sh when its OWN independent regression
// re-run (not the agent's self-reported one) fails for an item the agent just marked `done` —
// per the loop's core principle, a caught failure must become durable TODO.md/git state for the
// next stateless iteration to react to, not something the driver silently overrides or halts on.
// See classralph.sh step 5/its header comment, and stage3-forward-loop.md's "scan for any
// status: blocked item" guidance in Step 0.
//
// Deliberately does NOT touch other fields (crate, specs, done-when, blocked-by, docs-pair,
// commit) — the false "done" commit sha is left in place as an audit trail of what was reverted.
//
// Usage: node ralph/scripts/mark-todo-blocked.mjs "<todo-id>" "<one-line reason>" [path/to/TODO.md]
// Exits 0 and rewrites the file if the item was found, or exits 1 (no changes) if not.

import fs from 'fs/promises';
import path from 'path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');

const ITEM_HEADER_RE = /^- \[( |x)\] (.+)$/;
const FIELD_RE = /^(\s{4,})([a-z-]+): (.*)$/;

async function main() {
  const [, , todoId, reasonArg, todoPathArg] = process.argv;
  if (!todoId || !reasonArg) {
    console.error('Usage: mark-todo-blocked.mjs "<todo-id>" "<one-line reason>" [path/to/TODO.md]');
    process.exitCode = 1;
    return;
  }
  // Field values are single-line; flatten any newlines a caller passed in.
  const reason = reasonArg.replace(/\s+/g, ' ').trim();
  const todoPath = todoPathArg ? path.resolve(PROJECT_ROOT, todoPathArg) : path.join(PROJECT_ROOT, 'TODO.md');

  const content = await fs.readFile(todoPath, 'utf8');
  const lines = content.split('\n');

  let itemStart = -1;
  let itemEnd = lines.length;
  for (let i = 0; i < lines.length; i += 1) {
    const headerMatch = lines[i].match(ITEM_HEADER_RE);
    if (headerMatch) {
      if (itemStart !== -1) {
        itemEnd = i;
        break;
      }
      if (headerMatch[2].trim() === todoId) {
        itemStart = i;
      }
    }
  }

  if (itemStart === -1) {
    console.error(`mark-todo-blocked: no item "${todoId}" found in ${todoPath}`);
    process.exitCode = 1;
    return;
  }

  const headerMatch = lines[itemStart].match(ITEM_HEADER_RE);
  const indent = lines[itemStart + 1]?.match(FIELD_RE)?.[1] ?? '      ';
  lines[itemStart] = `- [ ] ${headerMatch[2].trim()}`;

  let statusLine = -1;
  let noteLine = -1;
  for (let i = itemStart + 1; i < itemEnd; i += 1) {
    const fieldMatch = lines[i].match(FIELD_RE);
    if (!fieldMatch) continue;
    if (fieldMatch[2] === 'status') statusLine = i;
    if (fieldMatch[2] === 'note') noteLine = i;
  }

  if (statusLine === -1) {
    console.error(`mark-todo-blocked: item "${todoId}" has no status field — refusing to guess where to insert one`);
    process.exitCode = 1;
    return;
  }
  lines[statusLine] = `${indent}status: blocked`;

  if (noteLine !== -1) {
    lines[noteLine] = `${indent}note: ${reason}`;
  } else {
    lines.splice(statusLine + 1, 0, `${indent}note: ${reason}`);
  }

  await fs.writeFile(todoPath, lines.join('\n'), 'utf8');
  console.log(`mark-todo-blocked: "${todoId}" set to status: blocked (${reason})`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
