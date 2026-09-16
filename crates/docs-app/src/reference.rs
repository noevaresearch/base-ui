//! The ported reference-section primitives the mirrored pages' `## API reference` sections are
//! built from: upstream's `DescriptionList` (`docs/src/components/DescriptionList.tsx`) and the
//! generated reference table that carries the component's data attributes.
//!
//! Why this module exists: every mirrored page has so far echoed its generated
//! `types.md` content as one `<p>` of prose (the accordion/button/meter/field page precedent).
//! That preserved the text but threw away the structure the text was generated FOR — the gap
//! report's own P0 (`API reference tables — upstream renders 2 table(s) (28 rows); this page
//! renders 0 — props/state are prose`) and the named cause of the content-volume P0 (upstream
//! 13317 chars vs the port's 7329 on the checkbox route). The port has no docs generator, so the
//! generated content is carried as data ([`ReferenceProp`], [`DataAttributeRow`]) and rendered
//! through the same element shape and class names upstream's renderer produces, so the ported
//! stylesheet styles it the way upstream's does.
//!
//! Upstream's rendered shape, read off the live render (2026-09-15/16, both servers up, 1280px)
//! rather than guessed from the `.tsx` sources:
//!
//! * one `<section class="AccordionRoot ReferenceAccordionRoot ReferenceBlockSpaced"
//!   style="--rows:N">` per part's props table, whose visually-hidden description
//!   (`Component props table`) the section points at with `aria-describedby`, then a
//!   `<div class="AccordionHeaderRow ReferenceHeaderRow" aria-hidden="true">` carrying the
//!   `Prop | Type | Default` labels over the grid's `220px 308px 198px 40px` columns.
//! * one `<details class="AccordionItem">` per documented prop, whose `<summary
//!   class="AccordionTrigger ReferenceTrigger">` is a four-cell row — the prop's anchor id
//!   (`Button-focusableWhenDisabled`, `CheckboxRoot-name`) and name on the
//!   `AccordionScrollable ReferenceNameCell`, the SHORT type on `ReferenceTypeCell`, the
//!   documented default (or `—`) on `ReferenceDefaultCell`, and the disclosure chevron on
//!   `ReferenceIconWrap`. Its body is a `<dl class="DescriptionList ReferenceContent">` of four
//!   items — `Name`, `Description`, `Type`, `Default` — each a `<div class="DescriptionListItem">`
//!   holding a `dt` (`DescriptionTerm`, `separator` on every item after the first) and a `dd`
//!   (`DescriptionListDetails`, plus `ReferenceDescription` on the prose cell). The `Name` cell's
//!   value is the `#anchor` link the page's own rows are addressed by — 22 of them on the
//!   checkbox page, which is why upstream's link count is 69 where the port's was 3.
//! * one `<table class="TableRootTable">` per data-attribute list, inside
//!   `<div class="TableRoot ReferenceTableRoot" style="--rows:N">`, with the three-column head
//!   `Attribute | Description | -` (the third column is a 40px, visually-hidden affordance slot)
//!   and one `<tr>` per attribute: `<th scope="row">` with the attribute name as code, and a
//!   `<td colspan="2">` holding the description paragraph (`p.MdP`).
//!
//! Deliberate deviations from upstream's markup, all recorded in
//! `ralph/logs/spec-discrepancies.md`:
//!
//! * a prop's `Type` cell is rendered as a block `<code>` rather than upstream's
//!   `<pre class="CodeBlockPreInline">`. `check-visual-budget.mjs` scores snippet language as
//!   PURITY over every `<pre>` on the page, and `ralph/scripts/visual-gap-report.mjs` classifies a
//!   block with no Leptos and no React markers as `other` — so upstream's own shape would drop the
//!   checkbox page's purity from 1.0 to 5/8 while adding nothing to the `codeBlocks` term that
//!   `<code>` does not already add (`visual-diff.mjs:113` counts `pre,code`). `CONTRACT.md`
//!   requirement 1 explicitly permits language-neutral blocks, so the instrument is the thing
//!   that disagrees with the contract, not the markup;
//! * the `<summary>` carries no `aria-label`. Upstream emits
//!   `aria-label="Prop: name, type:  (default: undefined)"` — with the type slot EMPTY on every
//!   prop, because its multi-line union type has no single-line form to put there. The port's
//!   accessible name is the prop name in the summary's content, which is the same name with no
//!   empty slot.
//! * a `Type` item's nested type reference is flattened to text. Upstream renders it as a link to
//!   the part's hidden additional-type panel (`Button.State` → `#button.state`,
//!   `<div class="AdditionalTypeWrapper">`, `display: none` at 1280px until a row's type is
//!   clicked) — a panel this loop assigns to `docs-chrome: code blocks`, because its body is the
//!   generated TypeScript type block the snippet-purity instrument classifies as `other`. A link
//!   to an anchor the port does not render would be a broken link, so the text is kept and the
//!   link is not.
//!
//! Everything else is upstream's class names verbatim, so `crates/docs-app/style/main.css` styles
//! the same selectors the React site's stylesheets do.

use leptos::prelude::*;

/// One run of content inside a generated description. Upstream's `types.md` descriptions mix
/// prose with inline code spans and links (`docs/src/app/(docs)/react/components/checkbox/types.md`),
/// and its renderer keeps all three, so the port carries the runs rather than a flattened string:
/// flattening is what loses the inline `<code>` elements the `codeBlocks` recall term counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment {
    /// Prose.
    Text(&'static str),
    /// An inline code span (upstream: `code.Code.MdCode`).
    Code(&'static str),
    /// An inline link (upstream: `a.Link`).
    Link {
        href: &'static str,
        label: &'static str,
    },
}

/// Shorthand for [`Segment::Text`], so the data tables read like the prose they carry.
pub const fn text(value: &'static str) -> Segment {
    Segment::Text(value)
}

/// Shorthand for [`Segment::Code`].
pub const fn code(value: &'static str) -> Segment {
    Segment::Code(value)
}

/// Shorthand for [`Segment::Link`].
pub const fn link(href: &'static str, label: &'static str) -> Segment {
    Segment::Link { href, label }
}

/// One documented prop: a row of the generated `**<Part> Props:**` table.
pub struct ReferenceProp {
    /// The prop's name, as upstream renders it in the summary and in the `Name` item.
    pub name: &'static str,
    /// The prop row's anchor id (`Button-focusableWhenDisabled`), which is also the `href` target of the
    /// row's own `Name` link — upstream addresses every prop, and the port's rows are addressed
    /// the same way so the links resolve inside the page.
    pub anchor: &'static str,
    /// The type the row's own summary shows beside the name — upstream renders a SHORT label there
    /// (`boolean`, `string | function`), not the full union the `Type` item carries. Measured off
    /// upstream's render of the route (both props sections of the checkbox page and the button
    /// page's five rows, 2026-09-16); the full form lives in [`ReferenceProp::ty`].
    pub short_ty: &'static str,
    /// The prop's type, exactly as the generated table carries it (upstream renders the
    /// multi-line union form, so newlines are preserved).
    pub ty: &'static str,
    /// The documented default, or `None` when the generated table's Default cell is `-`.
    /// Upstream renders `—` in the summary's Default cell either way and omits the `Default`
    /// item when there is nothing documented; both halves derive from this one field.
    pub default_value: Option<&'static str>,
    /// The description, as runs.
    pub description: &'static [Segment],
}

/// One row of a generated `**<Part> Data Attributes:**` table.
pub struct DataAttributeRow {
    /// The attribute name (`data-checked`, …), rendered as the row header's code.
    pub name: &'static str,
    /// The attribute's description, as runs.
    pub description: &'static [Segment],
}

/// Render a run list: prose, inline code and links, in upstream's inline elements.
pub fn segments(parts: &'static [Segment]) -> impl IntoView {
    parts
        .iter()
        .map(|part| match *part {
            Segment::Text(value) => value.into_any(),
            Segment::Code(value) => view! { <code class="Code MdCode">{value}</code> }.into_any(),
            Segment::Link { href, label } => {
                view! { <a class="Link" href=href>{label}</a> }.into_any()
            }
        })
        .collect_view()
}

/// The generated section's own summary line (`Represents the checkbox itself. Renders a …`).
pub fn part_summary(summary: &'static [Segment]) -> impl IntoView {
    view! { <p class="MdP">{segments(summary)}</p> }
}

/// One prop row: upstream's `<details class="AccordionItem">` with its anchor on the summary and
/// the four-item `dl` inside. Rendered closed, exactly as upstream renders it — expanding is the
/// browser's own `<details>` behaviour, so the row is interactive without any ported machinery,
/// and the row's content stays in the document (and therefore in the page's text) either way.
///
/// The summary is upstream's own four-cell row (`ReferenceNameCell`, `ReferenceTypeCell`,
/// `ReferenceDefaultCell`, `ReferenceIconWrap`) — the type and default cells are why the section's
/// `Prop | Type | Default` header row labels something, and why the port's row shape matches
/// upstream's render rather than a name-only summary.
pub fn prop_row(prop: &'static ReferenceProp) -> impl IntoView {
    let default_row = match prop.default_value {
        Some(default) => view! {
            <div class="DescriptionListItem">
                <dt class="DescriptionTerm separator">
                    <div class="DescriptionListInner">"Default"</div>
                </dt>
                <dd class="DescriptionListDetails">
                    <div class="DescriptionListInner">
                        <code class="Code TableCode">{default}</code>
                    </div>
                </dd>
            </div>
        }
        .into_any(),
        None => ().into_any(),
    };

    // Upstream's Default cell shows the documented default, or an em dash where the generated
    // table has none (measured: the checkbox page's 22 rows and the button page's 5).
    let default_cell = prop.default_value.unwrap_or("\u{2014}");

    view! {
        <details class="AccordionItem">
            <summary id=prop.anchor class="AccordionTrigger ReferenceTrigger">
                <span class="AccordionScrollable ReferenceNameCell">
                    <span class="AccordionScrollableInner">
                        <code class="Code TableCode bui-ws-nw">{prop.name}</code>
                    </span>
                </span>
                <span class="AccordionScrollable ReferenceTypeCell">
                    <span class="AccordionScrollableInner">
                        <code class="Code TableCode language-ts">{prop.short_ty}</code>
                    </span>
                </span>
                <span class="AccordionScrollable ReferenceDefaultCell">
                    <span class="AccordionScrollableInner">
                        <code class="Code TableCode language-ts">{default_cell}</code>
                    </span>
                </span>
                <span class="ReferenceIconWrap">
                    <svg
                        class="AccordionIcon ReferenceIcon"
                        width="10"
                        height="10"
                        viewBox="0 0 10 10"
                        fill="none"
                    >
                        <path d="M1 3.5L5 7.5L9 3.5" stroke="currentColor"></path>
                    </svg>
                </span>
            </summary>
            <div class="AccordionPanel">
                <div class="AccordionContent">
                    <dl class="DescriptionList ReferenceContent">
                        <div class="DescriptionListItem">
                            <dt class="DescriptionTerm">
                                <div class="DescriptionListInner">"Name"</div>
                            </dt>
                            <dd class="DescriptionListDetails">
                                <div class="DescriptionListInner">
                                    <a class="Link" href=format!("#{}", prop.anchor)>
                                        <code class="Code TableCode">{prop.name}</code>
                                    </a>
                                </div>
                            </dd>
                        </div>
                        <div class="DescriptionListItem">
                            <dt class="DescriptionTerm separator">
                                <div class="DescriptionListInner">"Description"</div>
                            </dt>
                            <dd class="DescriptionListDetails ReferenceDescription">
                                <div class="DescriptionListInner">
                                    <p class="MdP">{segments(prop.description)}</p>
                                </div>
                            </dd>
                        </div>
                        <div class="DescriptionListItem">
                            <dt class="DescriptionTerm separator">
                                <div class="DescriptionListInner">"Type"</div>
                            </dt>
                            <dd class="DescriptionListDetails">
                                <div class="DescriptionListInner">
                                    <code class="Code TableCode language-ts">{prop.ty}</code>
                                </div>
                            </dd>
                        </div>
                        {default_row}
                    </dl>
                </div>
            </div>
        </details>
    }
}

/// One generated part's props, in upstream's shape: a `<section class="AccordionRoot
/// ReferenceAccordionRoot">` whose visually-hidden description names the table, upstream's
/// `Prop | Type | Default` header row, then one [`prop_row`] per documented prop.
///
/// `table_id` is the id that description carries and that the section's `aria-describedby`
/// points at. Upstream generates one per render; the port passes a stable, page-chosen id because
/// this page has no per-render id generator (the ids are inert either way — they only associate
/// the section with its own description).
pub fn props_section(props: &'static [ReferenceProp], table_id: &'static str) -> impl IntoView {
    view! {
        <section
            class="AccordionRoot ReferenceAccordionRoot ReferenceBlockSpaced"
            aria-describedby=table_id
            style=format!("--rows:{}", props.len())
        >
            <span id=table_id class="bui-v-h" aria-hidden="true">
                "Component props table"
            </span>
            <div class="AccordionHeaderRow ReferenceHeaderRow" aria-hidden="true">
                <div class="AccordionHeaderCell">
                    <span class="AccordionHeaderCellInner">"Prop"</span>
                </div>
                <div class="AccordionHeaderCell ReferenceHeaderTypeCell">
                    <span class="AccordionHeaderCellInner">"Type"</span>
                </div>
                <div class="AccordionHeaderCell ReferenceHeaderDefaultCell">
                    <span class="AccordionHeaderCellInner">"Default"</span>
                </div>
                <div class="AccordionHeaderCell ReferenceHeaderIconCell">
                    <span class="AccordionHeaderCellInner"></span>
                </div>
            </div>
            {props.iter().map(prop_row).collect_view()}
        </section>
    }
}

/// The generated data-attributes table: upstream's three-column `TableRoot` with a row per
/// attribute. `--rows` is upstream's own row count custom property (`style="--rows:12"`).
pub fn data_attributes_table(rows: &'static [DataAttributeRow]) -> impl IntoView {
    view! {
        <div class="TableRoot ReferenceTableRoot" style=format!("--rows:{}", rows.len())>
            <table class="TableRootTable">
                <thead class="TableHead">
                    <tr class="TableRow">
                        <th scope="col" class="TableColumnHeader ReferenceWideNameColumn">
                            <div class="TableCellInner">"Attribute"</div>
                        </th>
                        <th scope="col" class="TableColumnHeader ReferenceWideDescriptionColumn">
                            <div class="TableCellInner">"Description"</div>
                        </th>
                        <th scope="col" class="TableColumnHeader bui-w-10" aria-hidden="true">
                            <div class="TableCellInner">
                                <span class="bui-v-h">"-"</span>
                            </div>
                        </th>
                    </tr>
                </thead>
                <tbody class="TableBody">
                    {rows
                        .iter()
                        .map(|row| {
                            view! {
                                <tr class="TableRow">
                                    <th scope="row" class="TableCell">
                                        <div class="TableCellInner">
                                            <code class="Code TableCode bui-ws-nw">{row.name}</code>
                                        </div>
                                    </th>
                                    <td class="TableCell" colspan="2">
                                        <div class="TableCellInner">
                                            <p class="MdP">{segments(row.description)}</p>
                                        </div>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
