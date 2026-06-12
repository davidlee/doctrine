# IMP-032: Governance superseded_by is a stored reciprocal — derive it from supersedes, don't store it (ADR-004)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Governance kinds (ADR/POL/STD) store `superseded_by` as an authored
`Vec<String>` field (`src/governance.rs:160`; written on the adr/policy/standard
paths — `src/adr.rs:231`, `src/policy.rs:227`, `src/standard.rs:232`). It is the
**reciprocal** of `supersedes`: `A supersedes B ⟺ B superseded_by A`. Storing
both directions is a stored inbound/reverse field — the exact pattern **ADR-004**
forbids (relations outbound-only; reciprocity is derived).

No code derives or cross-checks it today — it is hand-maintained and inert
(parsed only for `show`).

## Why it matters

Surfaced during SL-046 design (the cross-kind relation reader). SL-046's reader
rule projects only the canonical **outbound** direction (`supersedes`) and
**derives** the reciprocal via cordage `in_edges`, deliberately **not** projecting
`superseded_by` as edges (projecting both would double-count the same fact in the
derived inbound view). So the stored field is already redundant to the reader.

This leaves a latent drift: a hand-maintained `superseded_by` that disagrees with
the derived inbound. The clean fix is to **stop storing it** and render it as
derived inbound everywhere (matching how spec `descends_from` already derives its
children view, and slice `supersedes` already stores outbound-only).

## Scope of the fix (NOT SL-046 — capture-side)

- Remove `superseded_by` from the governance authored schema + migrate existing
  TOML (a closure/edit-preserving migration).
- Render the reciprocal as derived inbound in `show` / `inspect`.
- Belongs with **SL-048** (structural cross-corpus relation edges) under the
  relation-governance ADR that slice needs. Asymmetric reciprocal only — `related`
  is genuinely symmetric and legitimately appears on both sides.

Related: [[SL-048]] · IMP-016 (cross-corpus relations) · ADR-004 (outbound-only).
