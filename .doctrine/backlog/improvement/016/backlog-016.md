# IMP-016: Structural relation surface for product-to-product and spec-to-ADR links — cross-corpus relations are prose-only

## Problem

Doctrine anticipates cross-entity relations everywhere (reserved seams in PRD-001,
PRD-008, PRD-010) but the wiring is mostly unbuilt, so most cross-corpus links can
only be expressed as **prose citations** — which dangle silently (`reseat` reports
danglers, it never rewrites prose).

Current structural relation surfaces:

- **tech SPEC → PRD** — `descends_from` (single-valued, real). The one solid edge.
- **tech SPEC → tech SPEC** — `interactions.toml` (outbound peer edges, tech-only,
  hand-authored, no verb).
- **ADR → ADR** — `[relationships]` block (supersedes / superseded_by / related /
  tags) but **inert in v1 and no relation-set CLI**; and **no `amends` kind**, so
  ADR-009→ADR-003 (an amendment) cannot be stored (ADR-009 documents this debt).
- **backlog → slice / spec / drift** — `[relationships]` arrays (this item uses
  the `specs` array).

Missing entirely, with no home:

- **product PRD → product PRD** — product specs have **no relation surface at all**
  (e.g. PRD-013 → PRD-002 / PRD-010 / PRD-001 is prose-only).
- **any spec → ADR** — no spec→ADR edge kind exists anywhere; even the backlog
  `[relationships]` block has `slices`/`specs`/`drift` but **no `adrs`**.
- **ADR `amends` kind** — needed for the amendment lineage ADR-009 carries in prose.

## Why it matters

PRD-013 (requirement-reconciliation) is governed by ADR-003/ADR-009 and is a peer
of PRD-002/PRD-010/PRD-001, but none of that is queryable — it lives only in the
§Relations prose block and inline citations. The graph-derived priority engine
(PRD-011 / SPEC-001) and any future traceability or coverage tooling can only see
edges that are structurally stored; prose links are invisible to them.

## Scope sketch (not a design)

- A relation surface for product specs (outbound, ADR-004), and a spec→ADR edge
  kind (or an `adrs` array on the relevant `[relationships]` blocks).
- The ADR `amends` relation kind + a relation-set CLI (ADR-009 debt).
- Likely overlaps PRD-012 (`technical-specifications`, draft) which already owns
  "peer architectural relations between specs" + code anchors — reconcile scope
  with it rather than duplicate.

## References

- ADR-004 — relations stored outbound-only; reciprocity derived (the constraint
  any surface must honour).
- ADR-009 References — the `amends`-kind / relation-set-CLI debt, named there.
- PRD-012 — overlapping spec-relation + code-anchor machinery (draft).
- SL-021 non-goal — "cross-corpus linkage beyond tech-only interactions is later
  work."
