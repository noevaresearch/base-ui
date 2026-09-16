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
//   0. `status: blocked` OR `status: reopened`, deps satisfied — broken state first. A blocked note
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

// LIVELOCK GUARD STATE. Measured 2026-09-16: an item whose done-when contained a clause its lane could not
// reach was blocked honestly — and then re-picked as "broken state" every single iteration. Five picks
// produced four ledger-only commits, React types on the pages went 51 -> 51, and the commit rate fell from
// 22/hour to 2/hour. Re-reading a genuinely broken item is right; re-reading an item that has ALREADY been
// tried and re-blocked with its status unchanged is a decision waiting to be made (narrow the done-when, or
// split the item), and it costs a whole iteration per attempt.
const ATTEMPTS_PATH = path.join(PROJECT_ROOT, 'ralph/generated/pick-attempts.json');
const ATTEMPT_LIMIT = 3;
function loadAttempts() {
  try { return JSON.parse(fs.readFileSync(ATTEMPTS_PATH, 'utf8')); } catch { return {}; }
}
function saveAttempts(state) {
  try { fs.mkdirSync(path.dirname(ATTEMPTS_PATH), { recursive: true }); fs.writeFileSync(ATTEMPTS_PATH, JSON.stringify(state, null, 1)); } catch { /* best effort */ }
}

function stalled(item, attempts) {
  const rec = attempts[item.id];
  if (!rec) return false;
  // A status CHANGE means something happened (a real attempt, a new block reason) — the counter restarts.
  return rec.status === item.fields.status && rec.picks >= ATTEMPT_LIMIT;
}

function tier(item, attempts = {}) {
  const status = item.fields.status;
  const prio = (item.fields.priority || '').trim().toLowerCase();
  if (stalled(item, attempts)) return 4;   // tried, unchanged, needs a human decision — last
  // reopened == a false done recorded in the ledger (the item is checked-done-shaped state but the
  // ledger says otherwise); both are broken state and outrank new work.
  if (status === 'blocked' || status === 'reopened') return 0;
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
    if (status !== 'not-started' && status !== 'blocked' && status !== 'reopened') return false;
    return parseIdList(item.fields['blocked-by']).every(depSatisfied);
  });

  if (!pickable.length) {
    if (explain) console.error('nothing pickable: every remaining item is done, in progress, or blocked on unfinished deps');
    process.exitCode = 1;
    return;
  }

  const attempts = loadAttempts();
  const ordered = pickable
    .map((item, index) => ({ item, index, tier: tier(item, attempts) }))
    .sort((a, b) => (a.tier - b.tier) || (a.index - b.index));

  const chosen = ordered[0];
  // record the pick: consecutive picks with no status change are what the guard above counts
  {
    const prev = attempts[chosen.item.id];
    attempts[chosen.item.id] = {
      picks: prev && prev.status === chosen.item.fields.status ? prev.picks + 1 : 1,
      status: chosen.item.fields.status,
      lastPickedAt: new Date().toISOString(),
    };
    saveAttempts(attempts);
  }
  if (explain) {
    const counts = ordered.reduce((acc, o) => { acc[o.tier] = (acc[o.tier] || 0) + 1; return acc; }, {});
    console.error(`pickable ${pickable.length} — tier counts ${JSON.stringify(counts)} (0 blocked-resolved, 1 priority:high, 2 normal, 3 mega-unit, 4 stalled-needs-decision)`);
    console.error(`chosen tier ${chosen.tier}: ${chosen.item.id}`);
    const stalledItems = ordered.filter((o) => o.tier === 4).slice(0, 4).map((o) => `${o.item.id} (${attempts[o.item.id].picks} picks, status unchanged)`);
    if (stalledItems.length) console.error(`STALLED — these need a decision, not another iteration: ${stalledItems.join('; ')}`);
  }
  process.stdout.write(`${chosen.item.id}\n`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
