#!/usr/bin/env node
/* eslint-disable no-console */

// Picks the next TODO.md item for the forward loop to work on.
//
// ORDERING (highest priority first). File order alone was measured to be structurally blind to the
// work that closes the loop's own gates: on 2026-09-16 it kept returning `library: drawer` (line 643)
// while the fidelity and ergonomics items sat at 1973-2160 — every Phase E item that landed had been
// picked by the model OVERRIDING the picker, which costs reasoning every iteration and is not
// guaranteed to keep happening. So the order is now explicit:
//
//   0. `status: blocked` with its `blocked-by` deps satisfied — broken state first. A blocked note
//      may be stale (a later iteration resolved the blocker); an iteration re-checks rather than
//      trusting it, and either clears the block or re-records it. This is the ledger's own doctrine.
//   1. `priority: high` — work that closes a MEASURED gate that is currently open: the ergonomics /
//      fidelity / part-surface items whose numbers are below their bars (check-visual-budget,
//      snippet-ergonomics, check-part-surface). These are prioritized because they are the items
//      whose absence the gates keep reporting.
//   2. everything else, in file order (phase order), as before.
//   3. `priority: low` — the needs-batched-mining mega-units (drawer and friends): the ledger records
//      them as unclosable in one bounded iteration, so a pick there yields no done-ness and unblocks
//      nothing but its own docs pair. They sort LAST so they are only reached when nothing else is
//      pickable; when they are reached, that is a signal to batch-mine them rather than to try.
//
// An item is pickable when its `status` is `not-started` or `blocked` AND every `blocked-by` id
// resolves to a `done` item (or to "Phase A complete", meaning every Phase A item is done).
//
// Usage: node ralph/scripts/pick-next-todo.mjs [path/to/TODO.md] [--explain]
// Prints the chosen item's id, or nothing and exit 1 when nothing is pickable.

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
      current = { id: headerMatch[2].trim(), checked: headerMatch[1] === 'x', fields: {}, line: items.length };
      continue;
    }
    if (!current) continue;
    const fieldMatch = line.match(FIELD_RE);
    if (fieldMatch) {
      const [, key, rawValue] = fieldMatch;
      if (!(key in current.fields)) current.fields[key] = rawValue.replace(/\s+#.*$/, '').trim();
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

function tier(item) {
  const status = item.fields.status;
  const prio = (item.fields.priority || '').trim().toLowerCase();
  if (status === 'blocked') return 0;   // broken state first (note re-verified by the iteration)
  if (prio === 'high') return 1;        // closes a measured gate that is currently open
  if (prio === 'low') return 3;         // needs-batched-mining mega-unit: last resort
  return 2;                             // normal file/phase order
}

async function main() {
  const args = process.argv.slice(2).filter((a) => !a.startsWith('--'));
  const explain = process.argv.includes('--explain');
  const todoPath = args[0] ? path.resolve(PROJECT_ROOT, args[0]) : DEFAULT_TODO_PATH;
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

  const pickable = items.filter((item) => {
    const status = item.fields.status;
    if (status !== 'not-started' && status !== 'blocked') return false;
    return parseIdList(item.fields['blocked-by']).every(depSatisfied);
  });

  if (!pickable.length) {
    if (explain) console.error('nothing pickable: every remaining item is done, in progress, or blocked on unfinished deps');
    process.exitCode = 1;
    return;
  }

  const ordered = pickable
    .map((item, index) => ({ item, index, tier: tier(item) }))
    .sort((a, b) => (a.tier - b.tier) || (a.index - b.index));

  const chosen = ordered[0];
  if (explain) {
    const counts = ordered.reduce((acc, o) => { acc[o.tier] = (acc[o.tier] || 0) + 1; return acc; }, {});
    console.error(`pickable ${pickable.length} — tier counts ${JSON.stringify(counts)} (0 blocked-resolved, 1 priority:high, 2 normal, 3 mega-unit)`);
    console.error(`chosen tier ${chosen.tier}: ${chosen.item.id}`);
  }
  process.stdout.write(`${chosen.item.id}\n`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
