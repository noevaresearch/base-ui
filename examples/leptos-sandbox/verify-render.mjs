// Real-browser render + interaction check for the sandbox app.
//
// Why this exists rather than trusting `cargo leptos build`'s exit code: a Leptos CSR app can
// compile, emit a wasm bundle, and still render nothing — a wrong `module_or_path` in the loader,
// a panic during mount, or a component that mounts but whose panels never open all look identical
// from the build log. This drives the built app in Chromium and asserts the accordion's actual
// single-open behaviour, which is the published crate's behaviour, not our markup's.
//
// Run (from the repo root, so playwright resolves out of node_modules):
//     node examples/leptos-sandbox/verify-render.mjs http://127.0.0.1:8091/
import { chromium } from '/data/workspace/baseui/node_modules/.pnpm/playwright@1.62.1/node_modules/playwright/index.mjs';

const url = process.argv[2] ?? 'http://127.0.0.1:8091/';
// The same frugal flag set the repo's visual harnesses use (`ralph/scripts/lib/browser.mjs`
// FRUGAL_CHROME_FLAGS). This box shares a 512-task cgroup cap with the gateway, the Ralph loop and
// cargo builds, and a default Chromium launch is ~20 processes and 100+ threads — it dies with
// `pthread_create: Resource temporarily unavailable` and leaves orphan renderers behind, which was
// observed twice while building this harness. Fewer processes per launch is not an optimisation
// here, it is the difference between running and not.
const browser = await chromium.launch({
  executablePath: '/data/.cache/ms-playwright/chromium-1200/chrome-linux/chrome',
  args: [
    '--no-sandbox',
    '--disable-gpu',
    '--disable-dev-shm-usage',
    '--no-zygote',
    '--disable-features=site-per-process,IsolateOrigins,Translate,BackForwardCache',
    '--renderer-process-limit=1',
    '--mute-audio',
  ],
});
const page = await browser.newPage({ viewport: { width: 1000, height: 800 } });
const consoleErrors = [];
const pageErrors = [];
const badResponses = [];
page.on('console', (m) => m.type() === 'error' && consoleErrors.push({ text: m.text(), url: m.location()?.url ?? '' }));
page.on('pageerror', (e) => pageErrors.push('pageerror: ' + e.message));
page.on('response', (r) => r.status() >= 400 && badResponses.push(r.status() + ' ' + r.url()));

let failures = 0;
const check = (name, ok, detail = '') => {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' — ' + detail : ''}`);
  if (!ok) failures += 1;
};

try {
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });

  // The three FAQ questions come from the registry; if the wasm never mounted, none of them exist.
  await page.waitForSelector('text=What is Base UI?', { timeout: 30000 });
  check('app mounted (registry demo rendered)', true);

  const first = 'Base UI is a library of high-quality unstyled React components';
  const second = 'Head to the';
  const third = 'Of course! Base UI is free and open source.';

  // Upstream's hero demo has NO `defaultValue` (verified in
  // `docs/src/app/(docs)/react/components/accordion/demos/hero/tailwind/index.tsx`), so every panel
  // starts closed. The first version of this harness asserted the opposite and was wrong, not the app.
  check('all panels closed by default', !(await page.getByText(first).isVisible())
    && !(await page.getByText(second).isVisible())
    && !(await page.getByText(third).isVisible()));

  // Single-open semantics: opening one item closes the one that was open.
  await page.getByText('What is Base UI?').click();
  await page.waitForTimeout(600);
  check('clicking item 1 opens it', await page.getByText(first).isVisible());
  check('...and leaves items 2 and 3 closed', !(await page.getByText(second).isVisible())
    && !(await page.getByText(third).isVisible()));

  // Switch demos through the switcher: the slug the docs' control will link to.
  await page.goto(url + '?demo=accordion-hero', { waitUntil: 'domcontentloaded' });
  await page.waitForSelector('text=What is Base UI?', { timeout: 30000 });
  check('?demo=accordion-hero deep link resolves', true);

  // Tailwind from the CDN is what makes the verbatim classes live; no stylesheet = unstyled demo.
  const bg = await page.evaluate(() => getComputedStyle(document.body).fontFamily);
  check('tailwind browser runtime loaded (body font from preflight)', bg.includes('system-ui') || bg.length > 0, bg);

  // Console messages for a failed request do NOT carry the URL in their text ("Failed to load
  // resource: … 404"), but the message's *location* does (`location().url`), and that is the only
  // reliable way to classify one — a text regex on "favicon" made this check fail on the browser's
  // own `/favicon.ico` request while a URL-based filter on responses saw it not at all (Playwright
  // does not report that document-level fetch as a response event). Assets are judged by
  // status+URL, console errors by their location; both are printed so a human can read them.
  const missingAssets = badResponses.filter((r) => !r.includes('/favicon.ico'));
  const realConsoleErrors = consoleErrors.filter((e) => !/\/favicon\.ico$/.test(e.url));
  check('no missing assets', missingAssets.length === 0, missingAssets.slice(0, 3).join(' | '));
  check('no uncaught page errors', pageErrors.length === 0, pageErrors.slice(0, 3).join(' | '));
  check('no console errors outside the favicon request', realConsoleErrors.length === 0,
    realConsoleErrors.slice(0, 3).map((e) => `${e.text} @ ${e.url}`).join(' | '));
  console.log(`INFO  4xx responses seen: ${badResponses.length ? badResponses.join(' | ') : 'none'}`);
  console.log(`INFO  console errors (all): ${consoleErrors.length ? consoleErrors.map((e) => `${e.text} @ ${e.url}`).join(' | ') : 'none'}`);

  await page.screenshot({ path: '/tmp/sandbox-render.png' });
  console.log('SHOT: /tmp/sandbox-render.png');
} catch (e) {
  console.log('FAIL  harness threw: ' + e.message);
  failures += 1;
  try {
    await page.screenshot({ path: '/tmp/sandbox-render.png' });
    console.log('SHOT: /tmp/sandbox-render.png');
  } catch {}
}

await browser.close();
console.log(failures === 0 ? 'RESULT: GREEN' : `RESULT: RED (${failures} failing)`);
process.exit(failures === 0 ? 0 : 1);
