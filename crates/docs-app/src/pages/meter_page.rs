//! The docs page for `Meter`, mirroring
//! `docs/src/app/(docs)/react/components/meter/page.mdx`
//! (`specs/docs-content/meter/page.md`) — the `docs-content: components/meter`
//! TODO item.
//!
//! Page structure per the spec's "Page structure" section: `# Meter` h1,
//! `<Subtitle>` ("A graphical display of a numeric value within a range."), the
//! hero demo before the first heading, `## Anatomy` with the single fenced
//! snippet, and `## API reference` over the five generated `TypesMeter`
//! reference tables (Root, Track, Indicator, Value, Label — echoed as static
//! prose per the accordion/separator page precedent: the port has no docs
//! generator, so the tables' documented props are rendered as text, never
//! fabricated as executable machinery; the meter parts expose no data
//! attributes — `MeterRootState = {}` — so there is no data-attribute list).
//!
//! Page furniture mirrored in module docs (the accordion page precedent): the
//! `<Meta name="description">` content — "A high-quality, unstyled React meter
//! component that provides a graphical display of a numeric value." — and the
//! trailing `export const metadata` SEO keywords block (14 keywords: 'React
//! Meter', 'Meter Component', 'Progress Meter', 'Gauge', 'Level Indicator',
//! 'Measurement Display', 'Capacity Indicator', 'Value Indicator', 'Rating
//! Meter', 'Fuel Gauge', 'Accessible Meter', 'Headless React Components',
//! 'Graphical Value Display', 'Base UI').
//!
//! The live demo is the upstream Tailwind hero
//! (`docs/src/app/(docs)/react/components/meter/demos/hero/tailwind/index.tsx`,
//! the single `specs/docs-content/meter/demos.json` entry) ported onto the REAL
//! `leptos_ui` meter parts — `MeterRoot`/`MeterLabel`/`MeterValue`/`MeterTrack`/
//! `MeterIndicator` — with every upstream `className` string carried verbatim so
//! the DOM the Leptos port produces matches the React demo's element-for-element.
//! The demo is fully static (demos.json `stateManaged: "none"` — "Meter.Root
//! receives a literal `value={24}` prop with no React state, no event handlers,
//! and no user interaction"): the port's derivation pipeline
//! (valueToPercent → clamp → formatNumber) owns the rendered surface exactly as
//! upstream's does — the indicator's inline `width: 24%` fill, the Value's
//! default percent text, and the root's full ARIA tuple all derive from
//! `value=24` with zero demo-side machinery. No reactive plumbing was added.

use leptos::prelude::*;

use leptos_ui::{MeterIndicator, MeterLabel, MeterRoot, MeterTrack, MeterValue};

/// The upstream demo Root `className` (`hero/tailwind/index.tsx:6`): the two
/// column grid that right-aligns the Value against the Label above the Track.
const DEMO_ROOT_CLASS: &str = "grid max-w-full w-60 grid-cols-2 gap-y-2";

/// The upstream Label `className` (`hero/tailwind/index.tsx:7`).
const DEMO_LABEL_CLASS: &str = "text-sm font-normal text-neutral-950 dark:text-white";

/// The upstream Value `className` (`hero/tailwind/index.tsx:8`).
const DEMO_VALUE_CLASS: &str = "text-right text-sm text-neutral-950 dark:text-white";

/// The upstream Track `className` (`hero/tailwind/index.tsx:10`): spans both
/// grid columns under the Label/Value row.
const DEMO_TRACK_CLASS: &str = "col-span-2 h-3 overflow-hidden bg-neutral-200 dark:bg-neutral-800";

/// The upstream Indicator `className` (`hero/tailwind/index.tsx:11`): the fill
/// color + the width transition; the width itself is the part's inline
/// `width: {percentage}%` style from the context read.
const DEMO_INDICATOR_CLASS: &str = "bg-neutral-950 transition-[width] duration-500 dark:bg-white";

/// The hero demo (`demos/hero/tailwind/index.tsx`, demos.json entry 1): a
/// static storage-used meter at `value=24` (`propsExercised.Meter.Root:
/// ["value"]`), Label and Value as direct Root children (the flat placement the
/// Anatomy snippet shows), Track spanning the grid with the Indicator inside.
#[component]
pub fn MeterHeroDemo() -> impl IntoView {
    view! {
        <MeterRoot class=DEMO_ROOT_CLASS.to_string() value=24.0>
            <MeterLabel class=DEMO_LABEL_CLASS.to_string()>
                "Storage Used"
            </MeterLabel>
            <MeterValue class=DEMO_VALUE_CLASS.to_string() />
            <MeterTrack class=DEMO_TRACK_CLASS.to_string()>
                <MeterIndicator class=DEMO_INDICATOR_CLASS.to_string() />
            </MeterTrack>
        </MeterRoot>
    }
}

/// One API-reference part block: the generated `TypesMeter.<Part />` tables
/// (`docs/src/app/(docs)/react/components/meter/types.md`) echoed as static
/// prose — the summary line and the props list. The meter parts document no
/// data attributes (`types.md`'s state types are all empty), so unlike the
/// accordion page there is no third line.
fn api_part(summary: &'static str, props: &'static str) -> impl IntoView {
    view! {
        <p class="api-summary">{summary}</p>
        <p class="api-props">{props}</p>
    }
}

/// The `docs/src/app/(docs)/react/components/meter/page.mdx` page.
#[component]
pub fn MeterPage() -> impl IntoView {
    view! {
        <article class="docs-page">
            <h1>"Meter"</h1>
            <p class="subtitle">"A graphical display of a numeric value within a range."</p>

            <div class="docs-demo" data-demo="hero"><MeterHeroDemo /></div>

            <h2>"Anatomy"</h2>
            <p>"Import the component and assemble its parts:"</p>
            <pre><code>
"import { Meter } from '@base-ui/react/meter';

<Meter.Root>
  <Meter.Label />
  <Meter.Track>
    <Meter.Indicator />
  </Meter.Track>
  <Meter.Value />
</Meter.Root>;"
            </code></pre>

            <h2>"API reference"</h2>
            <h3>"Root"</h3>
            {api_part(
                "Groups all parts of the meter and provides the value for screen readers. Renders a <div> element.",
                "Props: value (number, required — the current value), aria-valuetext (string — a user-friendly name for aria-valuenow; overrides the derived default), getAriaValueText ((formattedValue: string, value: number) => string — a human-readable text alternative for aria-valuenow), locale (Intl.LocalesArgument — the Intl.NumberFormat locale, defaulting to the user's runtime locale), min (number, 0), max (number, 100), format (Intl.NumberFormatOptions), className, style, render.",
            )}
            <h3>"Track"</h3>
            {api_part(
                "Contains the meter indicator and represents the entire range of the meter. Renders a <div> element.",
                "Props: className, style, render.",
            )}
            <h3>"Indicator"</h3>
            {api_part(
                "Visualizes the position of the value along the range. Renders a <div> element.",
                "Props: className, style, render. The fill width is the part's inline style (width: {percentage}%), derived from the Root's value between min and max.",
            )}
            <h3>"Value"</h3>
            {api_part(
                "A text element displaying the current value. Renders a <span> element.",
                "Props: children ((formattedValue: string, value: number) => React.ReactNode | null — the render-function form; omission renders the formatted value), className, style, render.",
            )}
            <h3>"Label"</h3>
            {api_part(
                "An accessible label for the meter. Renders a <span> element.",
                "Props: className, style, render.",
            )}
        </article>
    }
}
