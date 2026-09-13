//! The docs page for `Progress`, mirroring
//! `docs/src/app/(docs)/react/components/progress/page.mdx`
//! (`specs/docs-content/progress/page.md`) — the
//! `docs-content: components/progress` TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Progress` h1,
//! `<Subtitle>` ("Displays the status of a task that takes a long time."), the
//! hero demo before the first heading, `## Anatomy` with the single fenced
//! snippet, and `## API reference` over the five generated `TypesProgress`
//! reference tables (Root, Track, Indicator, Value, Label — echoed as static
//! prose per the meter page precedent: the port has no docs generator, so the
//! tables' documented props are rendered as text, never fabricated as
//! executable machinery; unlike meter, the progress parts DO document data
//! attributes — `data-complete`/`data-indeterminate`/`data-progressing`, the
//! three-state status mapping — so each part block carries the third line).
//!
//! Page furniture mirrored in module docs (the meter page precedent): the
//! `<Meta name="description">` content — "A high-quality, unstyled React
//! progress bar component that displays the status of a task that takes a long
//! time." — and the trailing `export const metadata` SEO keywords block
//! (14 keywords: 'React Progress Bar', 'Progress Component', 'Progress
//! Indicator', 'Loader', 'Loading Bar', 'Upload Progress', 'Download
//! Progress', 'Task Status Indicator', 'Determinate Progress', 'Indeterminate
//! Progress', 'Accessible Progress', 'Headless React Components', 'Loading
//! Indicator', 'Base UI').
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/progress/demos/hero/tailwind/index.tsx`,
//! the single `specs/docs-content/progress/demos.json` entry) ported onto the
//! REAL `leptos_ui` progress parts —
//! `ProgressRoot`/`ProgressLabel`/`ProgressValue`/`ProgressTrack`/
//! `ProgressIndicator` — with every upstream `className` string carried
//! verbatim so the DOM the Leptos port produces matches the React demo's
//! element-for-element.
//!
//! ## The reactive demo machinery (the React re-render analog)
//!
//! demos.json `stateManaged`: "controlled — useState(20) wrapping
//! Progress.Root's `value`, updated by a 1s setInterval simulation that adds
//! up to 25 (clamped at 100) and is cleared on unmount". Leptos runs the
//! component body once, so the demo mirrors the React state on a leptos
//! `RwSignal` and reproduces the re-render with the mechanism PROVEN reactive
//! in this harness (`render_test.rs`
//! `view_dynamic_children_update_reactively_in_the_harness`): a tracked
//! dynamic-view child closure reading the signal re-runs on each tick and
//! rebuilds the whole part subtree — the per-render replacement semantics of
//! React's re-render (the merge-props page's rebuild convention; progress's
//! parts are plain Leptos components, so no `RawElementView` seed +
//! materialization is involved). The tracked read is the closure's own
//! `value.get()` — the whole closure is the subscriber.
//!
//! The simulation's `setInterval` rides the ported
//! [`leptos_ui_utils::use_interval::use_interval`] hook (the AGENTS.md rule:
//! never raw `window.setInterval`; upstream's demo uses the raw global inside
//! `React.useEffect`, and the port's hook adds the owner-disposal cleanup the
//! React effect's `clearInterval` return provides). The hook is created once
//! in the demo body under the route's reactive owner; the rebuild does not
//! re-create it, and `Interval.start`'s replace-not-stack contract makes each
//! restart cancel the previous schedule instead of stacking ticks. The
//! increment is `Math.random() * 25` through `js_sys::Math::random()` on
//! wasm, with a deterministic host fallback so the page compiles on both
//! targets (the sequence itself is non-deterministic upstream, so nothing can
//! pin it — the tests assert the invariants: start at 20, strictly advance,
//! every derived surface agrees on the same number).

use leptos::prelude::*;

use leptos_ui::{ProgressIndicator, ProgressLabel, ProgressRoot, ProgressTrack, ProgressValue};
use leptos_ui_internals::use_base_ui_id::use_base_ui_id;
use leptos_ui_utils::use_interval::use_interval;
use send_wrapper::SendWrapper;

/// The upstream demo Root `className` (`hero/tailwind/index.tsx:17`): the two
/// column grid that right-aligns the Value against the Label above the Track.
const DEMO_ROOT_CLASS: &str = "grid max-w-full w-60 grid-cols-2 gap-y-2";

/// The upstream Label `className` (`hero/tailwind/index.tsx:18`).
const DEMO_LABEL_CLASS: &str = "text-sm font-normal text-neutral-950 dark:text-white";

/// The upstream Value `className` (`hero/tailwind/index.tsx:21`).
const DEMO_VALUE_CLASS: &str = "text-right text-sm text-neutral-950 dark:text-white";

/// The upstream Track `className` (`hero/tailwind/index.tsx:22`): spans both
/// grid columns under the Label/Value row and clips the fill.
const DEMO_TRACK_CLASS: &str = "col-span-2 h-1 overflow-hidden bg-neutral-200 dark:bg-neutral-800";

/// The upstream Indicator `className` (`hero/tailwind/index.tsx:23`): the fill
/// color + the width transition; the width itself is the part's inline
/// `width: {percentage}%` style from the context read.
const DEMO_INDICATOR_CLASS: &str = "bg-neutral-950 transition-[width] duration-500 dark:bg-white";

/// One tick's increment — the demo's `Math.random() * 25`
/// (`hero/tailwind/index.tsx:11`). `Math.random()` resolves per call: the
/// ambient `js_sys::Math::random()` on wasm, a deterministic xorshift-style
/// LCG fallback on the host (which has no `Math` global; the sequence is
/// non-deterministic upstream, so only the range contract matters).
#[cfg(target_arch = "wasm32")]
fn random_unit() -> f64 {
    js_sys::Math::random()
}

/// The host arm — see [`random_unit`].
#[cfg(not(target_arch = "wasm32"))]
fn random_unit() -> f64 {
    use std::cell::Cell;
    thread_local! {
        static LCG: Cell<u64> = const { Cell::new(0x2545_F491_4F6C_DD1D) };
    }
    LCG.with(|state| {
        let next = state
            .get()
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state.set(next);
        // Top 31 bits → [0, 1).
        (next >> 33) as f64 / (1u64 << 31) as f64
    })
}

/// The tick body — `Math.min(100, Math.round(current + Math.random() * 25))`
/// (`hero/tailwind/index.tsx:11`): a random 0–25 increment, rounded, clamped
/// at 100 (upstream never clamps at min; the value starts at 20 and only
/// rises).
fn next_value(current: f64) -> f64 {
    (100.0f64).min((current + random_unit() * 25.0).round())
}

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): the
/// controlled `value` mirror starts at 20, the 1s interval simulation drives
/// it toward 100, and every derived surface (root ARIA tuple, Indicator's
/// inline width, Value's formatted text) re-derives from the mirrored prop on
/// each rebuild — the port's derivation pipeline owns them exactly as
/// upstream's render does. `interval_ms` parameterizes the real interval
/// delay (upstream 1000; the wasm suite passes a short value so the cycle
/// lands inside the test window).
///
/// Must run under a reactive owner (a component body — the `use_interval`
/// hook registers its disposal cleanup there, and the tracked closure below
/// subscribes under the route's owner).
pub fn progress_hero_demo_with(interval_ms: u32) -> impl IntoView {
    // The demo's `useState(20)` — the mirror the React demo re-renders on.
    let value = RwSignal::new(20.0_f64);

    // The demo's implicit stable Label id. React's `useId` ids are stable
    // across re-renders, so the Label ASSOCIATION survives the rebuilds; the
    // port's generated id churns per rebuild (a fresh `useBaseUiId` per
    // construction), so the demo mints the id ONCE here — through the real
    // ported `useBaseUiId` generator — and passes it to every rebuild's
    // `ProgressLabel id=…` (the button-loading demo's exact convention;
    // a user-supplied id wins over the registered one, ProgressLabel.tsx:20).
    // The id is LEAKED to `'static`: the rebuild closure captures it, and the
    // `view!` expansion was observed to capture String seeds BY VALUE (the
    // E0525 FnOnce trap — two borrow-derived attempts still moved); a
    // `&'static str` is Copy, so every capture is a copy and the closure
    // stays FnMut no matter the expansion's capture mode.
    let label_id: &'static str = {
        use reactive_graph::traits::GetUntracked as _;
        let generated = use_base_ui_id(reactive_graph::signal::RwSignal::new_local(None::<String>))
            .get_untracked();
        Box::leak(generated.into_boxed_str())
    };

    // The demo's `setInterval` — the ported `useInterval` hook (the AGENTS.md
    // rule), created once under the component's owner so its pending tick is
    // cleared when the owning scope is disposed (upstream's effect-returned
    // `clearInterval`). `SendWrapper` because the rebuild Effect's closure
    // must be Send while the hook's handle holds an `Rc` slot registry — the
    // wasm target is single-threaded, so the wrapper is the honest adapter
    // (the button page's TimeoutManager precedent).
    let interval = SendWrapper::new(use_interval());
    let interval_for_effect = interval.clone();
    let value_for_effect = value;
    Effect::new(move |_| {
        // Tracked read first — `.get()`, never `get_untracked()`: an
        // untracked read subscribes to nothing. Upstream's demo effect body
        // runs ONCE (`[]` deps) and only the render re-runs on the ticks;
        // this effect stays subscribed so the schedule survives each rebuild
        // through the same restart (each re-run replaces the previous
        // registration under the hook's replace-not-stack contract — one
        // live interval per instance, matching upstream).
        let _subscription = value_for_effect.get();
        interval_for_effect.start(interval_ms, {
            let value_for_tick = value_for_effect;
            move || value_for_tick.update(|current| *current = next_value(*current))
        });
        // The cleanup: the per-run cancel standing in for React's
        // `clearInterval` return. The handle CLONES into the cleanup closure
        // (clones share the pending-id slot) — moving it would make this
        // effect's closure FnOnce, and effects must be FnMut.
        let interval_for_cleanup = interval_for_effect.clone();
        move || interval_for_cleanup.clear()
    });

    // The tracked dynamic-view child — the React re-render analog (the
    // merge-props page's rebuild convention, proven reactive in this harness
    // by view_dynamic_children_update_reactively_in_the_harness): the closure
    // reads the signal, so each tick's write re-runs it and the fresh part
    // subtree REPLACES the previous one — per-render replacement semantics.
    // The value is read ONCE per evaluation and passed as the controlled
    // prop, exactly as the React demo's single `value={value}` prop feeds the
    // whole derivation; the demo-minted stable Label id rides every rebuild.
    view! {
        {move || {
            let current = value.get();
            view! {
                <ProgressRoot class=DEMO_ROOT_CLASS.to_string() value=Some(current)>
                    // `id=` rides the skill's known call-site rule: the
                    // `#[prop(optional)] Option<String>` unwraps a layer at
                    // call sites — pass the String, not Some(...) (E0308).
                    // The id is the demo-minted leaked `&'static str` (see
                    // above): captures of Copy types never move.
                    <ProgressLabel
                        class=DEMO_LABEL_CLASS.to_string()
                        id=label_id.to_string()
                    >"Export data"</ProgressLabel>
                    <ProgressValue class=DEMO_VALUE_CLASS.to_string() />
                    <ProgressTrack class=DEMO_TRACK_CLASS.to_string()>
                        <ProgressIndicator class=DEMO_INDICATOR_CLASS.to_string() />
                    </ProgressTrack>
                </ProgressRoot>
            }
        }}
    }
}

/// The component-tag form of the hero demo (what the page mounts).
#[component]
pub fn ProgressHeroDemo(interval_ms: u32) -> impl IntoView {
    progress_hero_demo_with(interval_ms)
}

/// One API-reference part block: the generated `TypesProgress.<Part />` tables
/// (`docs/src/app/(docs)/react/components/progress/types.md`) echoed as static
/// prose — the summary line, the props list, and the data-attributes list
/// (the accordion page's three-line shape: unlike meter, the progress parts
/// document the three-state status attributes).
fn api_part(summary: &'static str, props: &'static str, data_attrs: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
        <p class="api-data-attrs">{data_attrs}</p>
    }
}

/// The status data attributes shared by every part — the three-way mapping the
/// generated tables document per part (types.md's `Data Attributes` blocks).
const STATUS_DATA_ATTRS: &str = "Data attributes: data-complete (present when the progress has completed), data-indeterminate (present when the progress is in indeterminate state), data-progressing (present while the progress is progressing).";

/// The `docs/src/app/(docs)/react/components/progress/page.mdx` page.
#[component]
pub fn ProgressPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Progress"</h1>
            <p class="subtitle">"Displays the status of a task that takes a long time."</p>

            <div class="docs-demo" data-demo="hero"><ProgressHeroDemo interval_ms=1000 /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            <pre><code>
"import { Progress } from '@base-ui/react/progress';

<Progress.Root>
  <Progress.Label />
  <Progress.Track>
    <Progress.Indicator />
  </Progress.Track>
  <Progress.Value />
</Progress.Root>;"
            </code></pre>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                "Groups all parts of the progress bar and provides the task completion status to screen readers. Renders a <div> element.",
                "Props: value (number | null, required — the current value; the component is indeterminate when value is null), aria-valuetext (string — a user-friendly name for aria-valuenow, the current value of the progress bar), getAriaValueText ((formattedValue: string, value: number | null) => string — a human-readable text alternative for the current value), locale (Intl.LocalesArgument — the Intl.NumberFormat locale, defaulting to the user's runtime locale), min (number, 0), max (number, 100), format (Intl.NumberFormatOptions), className, style, render.",
                STATUS_DATA_ATTRS,
            )}
            <h3>"Track"</h3>
            {api_part(
                "Contains the progress bar indicator. Renders a <div> element.",
                "Props: className, style, render.",
                STATUS_DATA_ATTRS,
            )}
            <h3>"Indicator"</h3>
            {api_part(
                "Visualizes the completion status of the task. Renders a <div> element.",
                "Props: className, style, render. The fill width is the part's inline style (width: {percentage}%), derived from the Root's value between min and max; indeterminate carries no inline width.",
                STATUS_DATA_ATTRS,
            )}
            <h3>"Value"</h3>
            {api_part(
                "A text element displaying the current value. Renders a <span> element.",
                "Props: children ((formattedValue: string | null, value: number | null) => React.ReactNode | null — the render-function form; omission renders the formatted value), className, style, render.",
                STATUS_DATA_ATTRS,
            )}
            <h3>"Label"</h3>
            {api_part(
                "An accessible label for the progress bar. Renders a <span> element.",
                "Props: className, style, render.",
                STATUS_DATA_ATTRS,
            )}
        </article>
    }
}
