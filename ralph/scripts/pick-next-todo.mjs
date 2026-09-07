#!/usr/bin/env node
/* eslint-disable no-console */

// Picks the next TODO.md item for forward-loop.sh to work on: the first item, in file order
// (which is phase order — Phase A before B before C before D), that is `status: not-started`
// and whose `blocked-by` dependencies are all `status: done` (or is "Phase A complete", which
// resolves to "every Phase A item is done").
//
// This is deliberately deterministic and dependency-aware rather than random or purely
// sequential, per Principle 2 (bounded objective) — an iteration should never be handed an item
// whose prerequisites don't exist yet.
//
// Usage: node ralph/scripts/pick-next-todo.mjs [path/to/TODO.md]
// Prints the chosen item's id to stdout and exits 0, or prints nothing and exits 1 if no item
// is currently pickable (either everything is done, or everything remaining is blocked).

import fs from 'fs/promises';
import path from 'path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const DEFAULT_TODO_PATH = path.join(PROJECT_ROOT, 'TODO.md');

const ITEM_HEADER_RE = /^- \[( |x)\] (.+)$/;
const FIELD_RE = /^\s{4,}([a-z-]+): (.*)$/;

function parseTodo(content) {
  const lines = content.split('\n');
  const items = [];
  let current = null;
  for (const line of lines) {
    const headerMatch = line.match(ITEM_HEADER_RE);
    if (headerMatch) {
      if (current) items.push(current);
      current = { id: headerMatch[2].trim(), checked: headerMatch[1] === 'x', fields: {} };
      continue;
    }
    if (!current) continue;
    const fieldMatch = line.match(FIELD_RE);
    if (fieldMatch) {
      const [, key, rawValue] = fieldMatch;
      current.fields[key] = rawValue.replace(/\s+#.*$/, '').trim();
    }
  }
  if (current) items.push(current);
  return items;
}

function parseIdList(rawValue) {
  if (!rawValue) return [];
  const trimmed = rawValue.trim();
  if (!trimmed.startsWith('[') || !trimmed.endsWith(']')) return [trimmed];
  return trimmed.slice(1, -1).split(',').map((s) => s.trim()).filter(Boolean);
}

async function main() {
  const todoPath = process.argv[2] ? path.resolve(PROJECT_ROOT, process.argv[2]) : DEFAULT_TODO_PATH;
  const content = await fs.readFile(todoPath, 'utf8');
  const items = parseTodo(content);
  const byId = new Map(items.map((item) => [item.id, item]));

  const allPhaseADone = items
    .filter((item) => item.id.startsWith('utils: ') || item.id.startsWith('infra: '))
    .every((item) => item.fields.status === 'done');

  function depSatisfied(depId) {
    if (depId === 'Phase A complete') return allPhaseADone;
    const dep = byId.get(depId);
    return dep ? dep.fields.status === 'done' : false;
  }

  for (const item of items) {
    if (item.fields.status !== 'not-started') continue;
    const deps = parseIdList(item.fields['blocked-by']);
    if (deps.every(depSatisfied)) {
      process.stdout.write(`${item.id}\n`);
      return;
    }
  }

  process.exitCode = 1;
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
