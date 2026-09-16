// browser.mjs — shared Chrome lifecycle for the visual harness.
//
// WHY THIS EXISTS
// ---------------
// This box runs a 512-task cgroup cap (/sys/fs/cgroup/pids.max) shared with the Hermes gateway,
// the Ralph loop and cargo builds. A Chrome launch is ~20 processes and 100+ threads, and if a
// harness run is killed mid-flight (its own timeout, an OOM, a fork failure) Chrome survives as an
// orphan with all its children. Enough orphans and the box cannot fork at all: `fork: retry:
// Resource temporarily unavailable`, every Hermes tool call timing out in its pre-call hook, and
// git's credential helper dying with EAGAIN — which presents as a bogus auth failure. That state
// was reached once (512/512 tasks); this module exists so it cannot be reached again.
//
// Two defences:
//   1. reapStaleHarnessChromium() — kills any harness Chrome still alive from an earlier run
//      (identified by its --user-data-dir, which always sits under a harness temp prefix), before a
//      new one starts.
//   2. killChrome() — kills the whole PROCESS GROUP, not just the direct child, so renderers and
//      crashpad handlers cannot outlive the run. launchChrome() spawns detached so the group exists.
//
// Deliberately uses no dependencies: /proc scanning is enough, and this must work when the box is
// too starved to fork anything useful.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';

// Every harness temp dir prefix. A Chrome whose cmdline carries one of these is ours.
const HARNESS_TMP_PREFIXES = [
  '/tmp/visdiff-',
  '/tmp/gaprep-',
  '/tmp/cdp-diff-',
  '/tmp/visdiff',
  '/tmp/ralph-',
];

const CHROME_MARKER = 'chrome-' + 'linux64'; // split so this file's own path can never self-match

/** Kill any Chrome left over from an earlier harness run. Returns the number reaped. */
export function reapStaleHarnessChromium() {
  let reaped = 0;
  let entries;
  try {
    entries = fs.readdirSync('/proc');
  } catch {
    return 0;
  }
  for (const entry of entries) {
    if (!/^\d+$/.test(entry)) continue;
    if (Number(entry) === process.pid) continue;
    let cmdline = '';
    try {
      cmdline = fs.readFileSync(`/proc/${entry}/cmdline`, 'utf8');
    } catch {
      continue;
    }
    if (!cmdline.includes(CHROME_MARKER)) continue;
    const ours = HARNESS_TMP_PREFIXES.some((p) => cmdline.includes(p));
    if (!ours) continue;
    try {
      process.kill(Number(entry), 'SIGKILL');
      reaped++;
    } catch {
      /* already gone */
    }
  }
  return reaped;
}


// ---- cross-process harness mutex -------------------------------------------------------------
// WHY: two harness runs at once do not merely slow this box down, they BREAK it. A Chrome launch is
// ~20 processes inside a 512-task cgroup shared with the gateway, cargo and the loop, and the observed
// failure was an item being marked BLOCKED on a false regression: a concurrent run ate the task budget,
// a second Chrome could not start, and check-visual-budget reported "FAIL: could not measure
// react/components/toggle … chrome devtools port 9888" — a resource failure dressed up as a parity
// failure. Reaping on launch does not help, because the run holding the resources is alive.
// So every launch takes a pid-file mutex and waits its turn. Staleness is checked by liveness, not by
// age, so a hard-killed holder cannot wedge the queue.
const LOCK_PATH = process.env.RALPH_HARNESS_LOCK || '/tmp/ralph-harness.lock';

function lockHolderPid() {
  try { return Number(fs.readFileSync(LOCK_PATH, 'utf8').trim()) || 0; } catch { return 0; }
}
function pidAlive(pid) {
  if (!pid) return false;
  try { process.kill(pid, 0); return true; } catch { return false; }
}
let lockDepth = 0;   // one process may run several launches in sequence; it must not queue on itself
export async function acquireHarnessLock(waitMs = Number(process.env.RALPH_HARNESS_LOCK_WAIT_MS || 420000), pollMs = 2000) {
  if (lockDepth > 0) { lockDepth++; return true; }   // reentrant: same-process launches never deadlock
  const start = Date.now();
  for (;;) {
    try {
      const fd = fs.openSync(LOCK_PATH, 'wx');
      fs.writeSync(fd, String(process.pid));
      fs.closeSync(fd);
      lockDepth++;
      return true;
    } catch (e) {
      if (e.code !== 'EEXIST') throw e;
      const holder = lockHolderPid();
      if (!pidAlive(holder)) { try { fs.unlinkSync(LOCK_PATH); } catch {} continue; }   // dead holder
      if (Date.now() - start > waitMs) return false;
      await new Promise((r) => setTimeout(r, pollMs));
    }
  }
}
export function releaseHarnessLock() {
  if (lockDepth > 0) lockDepth--;
  if (lockDepth === 0 && lockHolderPid() === process.pid) { try { fs.unlinkSync(LOCK_PATH); } catch {} }
}

// A launch failure caused by the BOX (no tasks, no memory) rather than by the page. Callers must treat
// these as UNMEASURABLE — never as a parity verdict. Telling them apart matters: the false FAIL above
// blocked an item that was in fact complete.
export function isResourceFailure(message = '') {
  return /devtools port .* never came up|could not launch|Resource temporarily unavailable|ENOMEM|EAGAIN|cannot allocate|fork/i.test(String(message));
}

/**
 * Launch Chrome detached (so it leads its own process group) and wait for the devtools port.
 * Returns { child, kill } where kill() takes the whole group down. Serialised by the harness mutex.
 */
export async function launchChrome(flags, { tmpDir, port, waitMs = 30000 } = {}) {
  const got = await acquireHarnessLock();
  if (!got) {
    throw new Error(`another harness run held the lock for longer than the wait budget (${LOCK_PATH} held by pid ${lockHolderPid()}) — refusing to start a second Chrome on a box this size`);
  }
  process.once('exit', releaseHarnessLock);
  reapStaleHarnessChromium();
  const child = spawn(process.env.CHROME || '/data/tools/chrome-wrapper.sh', [
    ...flags,
    `--user-data-dir=${tmpDir}`,
    `--remote-debugging-port=${port}`,
    'about:blank',
  ], { stdio: 'ignore', detached: true });

  let kill = () => {
    try { process.kill(-child.pid, 'SIGKILL'); } catch {}
    try { child.kill('SIGKILL'); } catch {}
  };

  const deadline = Date.now() + waitMs;
  for (;;) {
    try {
      const r = await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(2000) });
      if (r.ok) return { child, kill };
    } catch {}
    if (Date.now() > deadline) {
      kill();
      await new Promise((r) => setTimeout(r, 3000));
      reapStaleHarnessChromium();
      throw new Error(`chrome devtools port ${port} never came up (check /sys/fs/cgroup/pids.current — a full task cgroup makes this fail)`);
    }
    spawnSync('sleep', ['0.5']);
  }
}

/** Kill a launched Chrome's whole process group, then reap any stragglers from this run. */
export function killChrome(child) {
  if (child) {
    try { process.kill(-child.pid, 'SIGKILL'); } catch {}
    try { child.kill('SIGKILL'); } catch {}
  }
  reapStaleHarnessChromium();
  releaseHarnessLock();
}

/** Report the box's task pressure — worth printing next to a failure so the cause is obvious. */
export function taskPressure() {
  try {
    return {
      current: Number(fs.readFileSync('/sys/fs/cgroup/pids.current', 'utf8').trim()),
      max: Number(fs.readFileSync('/sys/fs/cgroup/pids.max', 'utf8').trim()),
    };
  } catch {
    return null;
  }
}

// The frugal flags every harness launch should use on this box.
export const FRUGAL_CHROME_FLAGS = [
  '--headless=new', '--no-sandbox', '--disable-gpu', '--disable-dev-shm-usage',
  '--no-zygote', '--disable-features=site-per-process,IsolateOrigins,Translate,BackForwardCache',
  '--renderer-process-limit=1', '--mute-audio',
];
