#!/usr/bin/env node
/* eslint-disable no-console */

// Compares two TODO.md snapshots and prints the ids of items that became `status: done` in the
// "after" snapshot but were not `done` in the "before" snapshot. Used by every classralph.sh
// iteration: the agent is only ever handed a suggested item id (see stage3-forward-loop.md's
// "Step 0"), and may work on a different one instead, so the driver never knows in advance which
// item(s) to independently re-verify — it has to detect it from the diff.
//
// Usage: node ralph/scripts/diff-todo-done.mjs <before-file> <after-file>
// Prints one id per line.

import fs from 'fs/promises';

const ITEM_HEADER_RE = /^- \[( |x)\] (.+)$/;
const FIELD_RE = /^\s{4,}([a-z-]+): (.*)$/;

function parseStatuses(content) {
  const lines = content.split('\n');
  const statuses = new Map();
  let currentId = null;
  for (const line of lines) {
    const headerMatch = line.match(ITEM_HEADER_RE);
    if (headerMatch) {
      currentId = headerMatch[2].trim();
      continue;
    }
    if (!currentId) continue;
    const fieldMatch = line.match(FIELD_RE);
    if (fieldMatch && fieldMatch[1] === 'status') {
      statuses.set(currentId, fieldMatch[2].replace(/\s+#.*$/, '').trim());
    }
  }
  return statuses;
}

async function main() {
  const [, , beforePath, afterPath] = process.argv;
  if (!beforePath || !afterPath) {
    console.error('Usage: diff-todo-done.mjs <before-file> <after-file>');
    process.exitCode = 1;
    return;
  }
  const before = parseStatuses(await fs.readFile(beforePath, 'utf8'));
  const after = parseStatuses(await fs.readFile(afterPath, 'utf8'));

  for (const [id, status] of after) {
    if (status === 'done' && before.get(id) !== 'done') {
      process.stdout.write(`${id}\n`);
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
