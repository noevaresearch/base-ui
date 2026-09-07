#!/usr/bin/env node
/* eslint-disable no-console */

// Extracts one field's raw value for one TODO.md item, for shell scripts to consume without
// re-implementing TODO.md parsing in bash.
//
// Usage: node ralph/scripts/get-todo-field.mjs "<todo-id>" <field-name> [path/to/TODO.md]
// Prints the raw field value to stdout (list-shaped fields like `specs`/`blocked-by` print
// comma-separated as written) and exits 0, or prints nothing and exits 1 if the item or field
// doesn't exist.

import fs from 'fs/promises';
import path from 'path';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../..');

const ITEM_HEADER_RE = /^- \[( |x)\] (.+)$/;
const FIELD_RE = /^\s{4,}([a-z-]+): (.*)$/;

async function main() {
  const [, , todoId, fieldName, todoPathArg] = process.argv;
  if (!todoId || !fieldName) {
    console.error('Usage: get-todo-field.mjs "<todo-id>" <field-name> [path/to/TODO.md]');
    process.exitCode = 1;
    return;
  }
  const todoPath = todoPathArg
    ? path.resolve(PROJECT_ROOT, todoPathArg)
    : path.join(PROJECT_ROOT, 'TODO.md');

  const content = await fs.readFile(todoPath, 'utf8');
  const lines = content.split('\n');
  let inTarget = false;

  for (const line of lines) {
    const headerMatch = line.match(ITEM_HEADER_RE);
    if (headerMatch) {
      inTarget = headerMatch[2].trim() === todoId;
      continue;
    }
    if (!inTarget) continue;
    const fieldMatch = line.match(FIELD_RE);
    if (fieldMatch && fieldMatch[1] === fieldName) {
      process.stdout.write(`${fieldMatch[2].replace(/\s+#.*$/, '').trim()}\n`);
      return;
    }
  }
  process.exitCode = 1;
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
