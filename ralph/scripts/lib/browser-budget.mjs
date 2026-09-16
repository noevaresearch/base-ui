// browser-budget.mjs — the 4 GB rule, enforced where the browser is actually started.
//
// WHY THIS EXISTS
// ---------------
// The first attempt at this put the guard in `run-regression.sh`, guarding the gates that the ORCHESTRATOR runs.
// The loop doesn't always go through the orchestrator: an iteration that calls `check-react-mentions.mjs --all` or
// `snippet-ergonomics.mjs` directly still started a browser, and the box went straight back to 10 chromium processes
// and 81% of its 4 GB. A guard in one caller is a suggestion; every caller shares the same scripts, so the guard has
// to live in the scripts.
//
// The numbers this encodes (measured 2026-09-16): cgroup memory.max 4096 MB, memory.peak 4100 MB, oom_kill 51, 13
// iteration logs carrying fork/alloc failures. One chromium instance is ~1.4 GB across ~10 processes; a rust/wasm
// build is ~1.5 GB. Together they exceed the box, and the process the kernel kills is usually the iteration's own
// tool call — which reads as "the loop stopped moving" and cost hours of misdiagnosis.
//
// EXIT CODE 2 IS THE CONTRACT. Callers already map exit 2 (and the word UNMEASURABLE) to UNMEASURED rather than
// FAIL, which is the honest verdict: a measurement that did not happen is not a failed page. Never change this to a
// non-zero-but-successful code, and never print a score you did not measure.
//
// CI is allowed because CI has room: 4 vCPU / 16 GB, free for a public repository. `measure-port.yml` sets
// RALPH_BROWSER_GATES=1 explicitly — the allowance is named, not inferred, so a runner and a laptop are never
// confused for one another.

export const BROWSER_GATE_ALLOWED_ENV = 'RALPH_BROWSER_GATES';

/** True when the caller has explicitly allowed browser work on this machine. */
export function browserWorkAllowed() {
  return process.env[BROWSER_GATE_ALLOWED_ENV] === '1';
}

/**
 * Refuse to start a browser unless explicitly allowed. Returns false when work may proceed; otherwise prints the
 * reason, the alternative, and exits 2 (UNMEASURED for every caller).
 *
 * @param {string} gate      the script name asking for a browser
 * @param {string} instead   what the caller should read INSTEAD of measuring here
 */
export function refuseBrowserWork(gate, instead) {
  if (browserWorkAllowed()) return false;
  console.log(`UNMEASURABLE: ${gate} needs a real browser and this box cannot afford one right now.`);
  console.log('  why: the sandbox is a 4096 MB cgroup; one chromium is ~1.4 GB across ~10 processes, a rust build');
  console.log('       ~1.5 GB. Building and measuring together has hit 95% and produced 51 oom_kill events today —');
  console.log('       and the process the kernel kills is usually an iteration\'s own tool call, which reads as');
  console.log('       "the loop stopped" rather than "the box was out of memory".');
  console.log(`  instead: ${instead}`);
  console.log(`  to run it here anyway (at real risk of an OOM kill): ${BROWSER_GATE_ALLOWED_ENV}=1 node ralph/scripts/${gate}`);
  process.exit(2);
  return true; // unreachable; keeps linters honest about the return type
}
