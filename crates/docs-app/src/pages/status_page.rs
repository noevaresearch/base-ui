//! /status — the port's progress table, on the site rather than only in a chat message.
//!
//! WHY IT EXISTS
//! -------------
//! "How far along is the port?" had two answers nobody could reproduce: items marked done in the ledger, and
//! pages passing the scorecard. Neither said WHICH component was behind or WHY. This page renders
//! `status_data.rs` — generated from the gates' own outputs before each deploy — so the breakdown is visible
//! where the port lives.
//!
//! The two headline numbers are kept visibly distinct on purpose: items-done is a planning count, and
//! pages-passing is the quality count (a route passes only when EVERY axis passes, and an unmeasured axis is
//! never a pass). They are far apart, and pretending otherwise is how "60% done" coexisted with React code on
//! every mirrored page.

use crate::status_data::{
    COMPONENTS, EXPLANATION, GENERATED_AT, ITEMS_DONE, ITEMS_PCT, ITEMS_TOTAL, MIRRORED_ROUTES,
    PAGES_MEASURED, PAGES_PASSING,
};
use leptos::prelude::*;

fn opt<T: std::fmt::Display>(v: Option<T>) -> String {
    match v {
        Some(x) => format!("{}", x),
        None => "unmeasured".to_string(),
    }
}

/// A measured axis: unmeasured is visually distinct from a bad number, because they mean different things.
fn cell(v: Option<f64>, bar: f64, suffix: &str) -> impl IntoView {
    let (text, class) = match v {
        None => ("unmeasured".to_string(), "status-cell status-unmeasured"),
        Some(x) if x >= bar => (format!("{:.1}{}", x, suffix), "status-cell status-ok"),
        Some(x) => (format!("{:.1}{}", x, suffix), "status-cell status-low"),
    };
    view! { <td class=class>{text}</td> }
}

#[component]
pub fn StatusPage() -> impl IntoView {
    let rows = COMPONENTS
        .iter()
        .map(|c| {
            let parts = match (c.parts_exposed, c.parts_documented) {
                (Some(e), Some(d)) => format!("{}/{}", e, d),
                _ => "unmeasured".to_string(),
            };
            let snips = match (c.snippets_leptos, c.snippets_react) {
                (Some(l), Some(r)) => format!("{}/{}", l, r),
                _ => "unmeasured".to_string(),
            };
            let verdict = c.page_verdict.unwrap_or("unmeasured");
            let verdict_class = if verdict == "PASS" {
                "status-cell status-ok"
            } else {
                "status-cell status-low"
            };
            view! {
                <tr>
                    <td class="status-cell status-name">{c.component}</td>
                    <td class="status-cell">{parts}</td>
                    <td class="status-cell">{opt(c.namespaced_path)}</td>
                    <td class="status-cell">{snips}</td>
                    {cell(c.example_length_pct, 80.0, "%")}
                    {cell(c.attribute_ratio, 0.8, "x")}
                    {cell(c.copy_coverage, 95.0, "%")}
                    {cell(c.page_parity, 90.0, "")}
                    {cell(c.widget_parity, 97.0, "%")}
                    <td class=verdict_class>{verdict}</td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <article class="status-page">
            <h1>"Port status"</h1>
            <p class="status-lead">
                "Every component, every axis that decides its migration, generated from the gates' own output at "
                {GENERATED_AT} ". A field that could not be measured reads "
                <strong>"unmeasured"</strong>
                " — it is never counted as a pass."
            </p>

            <div class="status-headline">
                <div class="status-metric">
                    <span class="status-metric-value">{format!("{:.0}%", ITEMS_PCT)}</span>
                    <span class="status-metric-label">{format!("items done ({} / {})", ITEMS_DONE, ITEMS_TOTAL)}</span>
                    <span class="status-metric-note">"planning count — ledger entries marked done"</span>
                </div>
                <div class="status-metric">
                    <span class="status-metric-value">{format!("{} / {}", PAGES_PASSING, MIRRORED_ROUTES)}</span>
                    <span class="status-metric-label">"pages passing every axis"</span>
                    <span class="status-metric-note">
                        {format!("quality count — {} of {} mirrored routes measured so far; an unmeasured axis never passes", PAGES_MEASURED, MIRRORED_ROUTES)}
                    </span>
                </div>
            </div>

            <p class="status-explain">{EXPLANATION}</p>

            <div class="status-table-wrap">
                <table class="status-table">
                    <thead>
                        <tr>
                            <th>"component"</th>
                            <th>"parts"</th>
                            <th>"namespaced path"</th>
                            <th>"snippets leptos/react"</th>
                            <th>"example length (bar 80%)"</th>
                            <th>"attributes (bar 0.8x)"</th>
                            <th>"copy (bar 95%)"</th>
                            <th>"page parity (bar 90)"</th>
                            <th>"widget (bar 97%)"</th>
                            <th>"verdict"</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </article>
    }
}
