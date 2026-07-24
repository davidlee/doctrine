# REV REV-033 — reconcile SL-226

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile pass for SL-226 (CLI graph emitter + ego-view). One governance item
from RV-301's reconciliation brief.

## Reconcile narrative (SL-226)

- **[RV-301 finding F-4]** — `modify` PRD-016. PRD-016 §2's out-of-scope bullet
  demotes *static graph file interchange (GraphML/Cypher/DOT-file export) for
  external tools* to on-demand (RFC-002). SL-226 ships `doctrine graph`, an
  on-demand, pipe-composable DOT/JSON stdout consumption surface (RFC-001) — a
  different thing from the demoted static-file interchange pipeline, but close
  enough in wording ("DOT export") to be misread as excluded. This was flagged
  as a pre-declared reconcile deferral at design time (design §3; scope Context).

  **Target:** PRD-016 §2, the "Static graph file interchange …" out-of-scope
  bullet (spec-016.md ~l.64-67).

  **Before (end of bullet):**
  > … this capability's interchange surface is the navigable JSON/SVG contract,
  > not a file export pipeline.

  **After (appended):**
  > … not a file export pipeline. This demotion covers *static file interchange
  > for external tools* only; it does not reach `doctrine graph`'s on-demand,
  > pipe-composable DOT/JSON stdout emission (RFC-001, delivered by SL-226), an
  > in-workflow consumption surface rather than a static export channel.

  Surfaced-for-manual landing (a `modify` row is not auto-landed by
  `revision apply`).
