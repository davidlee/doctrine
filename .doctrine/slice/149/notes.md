# SL-149 implementation notes (durable)

Durable decisions + deviations harvested during the build. Runtime progress lives in
`.doctrine/state/.../phases/`; this file is the authored record that survives close-out.

## Execution deviation from plan.toml — hybrid additive build (user-approved via /consult)

**What changed.** The plan authored P2 to *remove* the `Specs`/`Requirements`
`RelationLabel` variants and P4 to *remove* the named `specs`/`requirements` show keys.
During P2 this proved impossible to land green: removing the enum variants + re-keying
`lookup`/`inbound_name`/`validate_link` is a **workspace-wide hard cut** (7 consumer files
+ 5 callers), so no leaf-only phase can both remove them and keep `just check` green.
Decisively, the **P5 corpus migration must still read the live `specs`/`requirements`
edges** to rewrite them — so the variant-parsing code must exist until migration time.

**Resolution (user-approved).** P2–P4 are **ADDITIVE**: `References` + role grammar ship
alongside the retained `Specs`/`Requirements` variants/rows and the legacy `specs`/
`requirements` show keys. The corpus is fixture-only for these phases (no live
`references` edges until P5). The **removal of the variants/rows/legacy keys moves to
PHASE-05**, co-located with the corpus migration where it is actually safe — exactly where
the design's hard-cut (§2.9) lives.

**Plan criteria affected (reinterpreted, not abandoned):**
- P2 EX-1 "Specs/Requirements variants removed" → satisfied in **P5**.
- P4 EX-3 "specs/requirements keys removed" → satisfied in **P5**.
- P5 EX-4 "No specs/requirements rows remain" → now also owns the **code** removal
  (variants + rows + legacy show keys), after the migration has read the old edges.

The transient dual-vocabulary (P2–P4) is internal/fixture-only and harmless; it is **not**
the corpus dual-read §2.9 prohibits (that is about on-disk state, addressed by P5's
hard-cut single-commit).

## PHASE-04 executed as 3 sub-batches (cost/quality, user pref)

P4a rendering surfaces (inbound/outbound/list/census/web-graph data) · P4b show + show
--json references-by-role object · P4c link/unlink `--role` CLI. Each independently green.
PHASE id is immutable; this is runtime batching only.

## Carried follow-up — web-graph TS frontend (capture as backlog at reconcile)

P4a wired role through the **backend** web-graph data (`catalog/graph.rs` serialises
`role`), but the **TS frontend** (`web/map/`) does not yet read `edge.role` to render
`references(<role>)` in the dot label. Out of the design's named seam (§2.7 points at
`catalog/graph.rs`, not the JS). File a backlog item at reconcile.

## Key shipped seam (for P5/P6/audit)

- `Role` enum (closed, Ord) + `RelationLabel::References` in `src/relation.rs`; rules
  re-keyed `(source,label,role)`; `lookup`/`legal_roles`/`inbound_name`/`validate_link`
  (MissingRole/IllegalRole/RoleNotApplicable) role-aware; pure `targets_for_role`.
- Source sets (pinned, live census 2026-06-24): `implements`={SL}, `scoped_from`={SL},
  `concerns`={SL,RFC,ISS,IMP,CHR,RSK,IDE}. Inbound verbs: implemented by / scoped into /
  concerned by.
- Storage: `RelationEdge`/`RelationRow` carry `role`; `[[relation]] role` serde-skipped
  when None (label-only rows byte-identical on disk); idempotency on the
  `(label,role,target)` triple.
- `references(implements) → Kinds(SPEC,PRD,REQ)`; `references(scoped_from) →
  Kinds(BACKLOG)`; `references(concerns) → AnyNumbered`.
