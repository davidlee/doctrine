# IDE-068: Typst as a render target for design docs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Idea

Use Typst (typesetting language with an embeddable Rust crate, `typst`) to
render doctrine documents — design docs first — into polished PDF/SVG, with
diagrams better-looking than mermaid.

## Assessment (2026-09-30, quick, unverified claims marked)

**As an authoring format: no.** Markdown is the storage rule's prose tier
across every kind, every template, `materialise`, `show`, the library, and
client repos (POL-002). Agents are far more fluent in markdown than Typst;
a format switch costs tokens and syntax errors on every write for a
presentation gain nobody reads raw.

**As a render target: plausible.** Markdown stays canonical; a
`doctrine export`-style verb renders to PDF. Typst can ingest CommonMark via
the `cmarker` package (unverified from memory) and embed SVG. Costs of the
crate: implement its `World` trait (fonts, files, package fetch — network),
0.x API churn, binary growth from bundled fonts. A shell-out to the `typst`
CLI avoids all three and is the cheaper first probe.

**diagram-design fit: orthogonal.** diagram-design is an agent skill that
hand-lays inline SVG per diagram; it is not a renderer. Its SVG embeds in
markdown (`![](x.svg)`) or the map explorer just as well — Typst is not
needed to get prettier diagrams. Tradeoff vs mermaid: hand-laid SVG is not
regenerable from a diffable source, so it drifts from the prose and is
token-heavy to revise. The Typst-native diagrams-as-code option is `fletcher`
(unverified), but it only renders inside Typst.

## If taken

Cheapest probe: markdown design doc → `typst` CLI via cmarker → PDF, mermaid
blocks swapped for pre-rendered SVG. Judge whether anyone reads the PDF
before touching the crate.

Related: IMP-385 (diagram rendering for the spec anchor report; DOT chosen
by DEC-115), SL-245 (inline terminal diagram rendering).
