#!/usr/bin/env node
/* eslint-disable no-console */

// Structural validator for TODO.md. Parses every `- [ ]`/`- [x]` item block and its indented
// fields, then enforces the invariants the Ralph loop depends on:
//
//   1. Checkbox state must agree with `status:` (`[x]` <=> done, `[ ]` <=> anything else).
//   2. Every id in a `blocked-by:` list must resolve to either an existing item id or the
//      synthetic dependency "Phase A complete" (meaning: every Phase A item is done).
//   3. Every `docs-pair:` value must resolve to an existing item id.
//   4. THE core rule from the approved plan: an item with a `docs-pair:` (and without
//      `exempt-from-docs-pairing: true`) cannot be `status: done` unless the item it points to
//      is ALSO `status: done`. This is what makes "ported AND renders in the Leptos docs site"
//      structurally impossible to skip — a forward-loop iteration cannot check off a component
//      by only making its Rust tests pass.
//   5. No duplicate item ids.
//
// Usage:
//   node ralph/scripts/check-todo-schema.mjs [path/to/TODO.md]
//
// Exit code is non-zero on any violation, so this is usable as a loop/CI gate (wired into
// ralph/scripts/run-regression.sh before any TODO checkoff is accepted).

import fs from 'fs/promises';
import path from 'path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');
const DEFAULT_TODO_PATH = path.join(PROJECT_ROOT, 'TODO.md');

const VALID_STATUSES = new Set(['not-started', 'in-progress', 'done', 'blocked', 'reopened']);
const SYNTHETIC_DEPS = new Set(['Phase A complete']);

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
      current = {
        id: headerMatch[2].trim(),
        checked: headerMatch[1] === 'x',
        fields: {},
        lineNumber: null,
      };
      continue;
    }
    if (!current) continue;

    const fieldMatch = line.match(FIELD_RE);
    if (fieldMatch) {
      const [, key, rawValue] = fieldMatch;
      // Strip trailing inline comments (`  # ...`) that aren't part of the value.
      const value = rawValue.replace(/\s+#.*$/, '').trim();
      current.fields[key] = value;
    }
  }
  if (current) items.push(current);

  return items;
}

function parseIdList(rawValue) {
  if (!rawValue) return [];
  const trimmed = rawValue.trim();
  if (!trimmed.startsWith('[') || !trimmed.endsWith(']')) {
    return [trimmed];
  }
  return trimmed
    .slice(1, -1)
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
}

function main() {
  const todoPath = process.argv[2] ? path.resolve(PROJECT_ROOT, process.argv[2]) : DEFAULT_TODO_PATH;

  return fs.readFile(todoPath, 'utf8').then((content) => {
    const items = parseTodo(content);
    const byId = new Map(items.map((item) => [item.id, item]));
    const errors = [];

    const seenIds = new Set();
    for (const item of items) {
      if (seenIds.has(item.id)) {
        errors.push(`Duplicate item id: "${item.id}"`);
      }
      seenIds.add(item.id);

      const status = item.fields.status;
      if (status === undefined) {
        errors.push(`"${item.id}": missing status field`);
      } else if (!VALID_STATUSES.has(status)) {
        errors.push(`"${item.id}": invalid status "${status}" (expected one of ${[...VALID_STATUSES].join(', ')})`);
      } else if (item.checked && status !== 'done') {
        errors.push(`"${item.id}": checkbox is [x] but status is "${status}" (expected "done")`);
      } else if (!item.checked && status === 'done') {
        errors.push(`"${item.id}": status is "done" but checkbox is [ ] (should be [x])`);
      }

      for (const depId of parseIdList(item.fields['blocked-by'])) {
        if (!SYNTHETIC_DEPS.has(depId) && !byId.has(depId)) {
          errors.push(`"${item.id}": blocked-by references unknown item "${depId}"`);
        }
      }

      const docsPair = item.fields['docs-pair'];
      if (docsPair) {
        const pairedItem = byId.get(docsPair);
        if (!pairedItem) {
          errors.push(`"${item.id}": docs-pair references unknown item "${docsPair}"`);
        } else if (
          item.fields['exempt-from-docs-pairing'] !== 'true' &&
          item.fields.status === 'done' &&
          pairedItem.fields.status !== 'done'
        ) {
          errors.push(
            `"${item.id}": marked done but its docs-pair "${docsPair}" is not done — ` +
              'a component/util is not complete until its docs page also renders against it ' +
              '(see the approved plan\'s objective). Do not mark this done until the pair is done, ' +
              'or add exempt-from-docs-pairing: true if this pairing is genuinely wrong.',
          );
        }
      }
    }

    console.log(`Parsed ${items.length} TODO items from ${path.relative(PROJECT_ROOT, todoPath)}`);

    if (errors.length > 0) {
      console.error(`\n${errors.length} schema violation(s):`);
      errors.forEach((e) => console.error(`  - ${e}`));
      process.exitCode = 1;
    } else {
      console.log('Schema OK.');
    }
  });
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
