# IMP-294: Project belt design-target declarations into the selector registry

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at SL-220 audit (RV-277 F-3). Mid-phase design-target declarations
made through the dispatch belt (e.g. 8e0d7699, cb8c45b5, 33851ebf) live in the
coord tree and never project back into the primary tree's selector registry
(`slice-NNN.toml`), so `slice conformance` at audit reports every adjudicated
extension as `undeclared` — 21 cells for SL-220, each re-derived by hand from
phase sheets. Either project belt declarations at the funnel/prepare-review
beat, or reconcile them into the registry at conclude. Related: IMP-162
(selector glob lint), IDE-025 (selector-sourced write allowlist).
