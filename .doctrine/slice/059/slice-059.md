# Knowledge records: standalone four-kind entity surface

## Context

Forward-intent realisation of **SPEC-019** (knowledge-record entity surface,
amended in `6643f4c`), itself descending from **PRD-010**. No code exists yet.

SPEC-019 is sliced in three (the cut, per the spec's pinned structure):

- **Slice A — this slice (SL-059): the standalone entity surface.** The four
  record kinds on the engine, their lifecycles/facets/evidence, prefix→kind
  resolution, the priority-partition declaration, and the `knowledge`
  capture/inspect/survey/transition CLI. **No cross-kind deps — ships alone.**
- **Slice B — the relation seam** (FR-005) → backlog (`see Follow-Ups`).
- **Slice C — supersession** (FR-006, IMP-006-gated) → backlog.

This slice rides the shared entity scaffold unchanged (SPEC-004 / the backlog's
single-entity discipline, SPEC-015) — it is the structural sibling of the backlog
surface. The variation it adds is **data** keyed by `record_kind`, not a parallel
implementation.

## Scope & Objectives

Deliver SPEC-019 **FR-001/002/003/004**, **FR-007** (the four shared verbs only),
**NF-001/002/003** — i.e. REQ-239/240/241/242, REQ-253, REQ-245/246/247.

1. **Four kinds on the engine.** Bind `assumption`/`decision`/`question`/
   `constraint` (prefixes ASM/DEC/QUE/CON) as four data-valued engine `Kind`s
   over one `knowledge_record` entity discriminated by `record_kind` — each its
   own tree (`.doctrine/knowledge/<kind>/`), reservation namespace, and
   `record-NNN.{toml,md}` fileset + `NNN-slug` symlink. One kind-blind
   materialiser; never parallel per-kind schemas (NF-001).
2. **`integrity::KINDS` rows** for the four kinds (+ their stateful status sets).
   Mind the **ordered** golden `kinds_table_covers_the_numbered_kinds` and
   `scanned_kinds()`.
3. **Per-kind lifecycle vocabularies** keyed by `record_kind`, the transition
   verb validating against the record's own vocab and refusing a foreign-kind
   state; a per-kind `is_terminal` predicate driving the `listing::retain`
   hide-set (FR-002). One `*_STATUSES` const + known-set guard per kind.
4. **Per-kind typed `[facet]` blocks + shared `[evidence]` structure** through
   the `"" → None` optional seam; the closed enums (`confidence`, `basis`,
   constraint `source`) each with a drift guard mirroring their variant set
   (FR-003, NF-001). `confidence` is assumption-only.
5. **Prefix→kind read-path resolution** — `show`/`status` resolve `record_kind`
   from the id prefix; identity is permanent, `record_kind` fixed at capture
   (FR-004, NF-003).
6. **Four `priority::partition` entries** — one per kind, `workable: &[]` /
   all-`Terminal`, plus four VT-1 drift canaries (NF-003; never `Workable`). This
   is the status-ful declaration, **not** REC's status-less `None → Terminal`
   path.
7. **The `doctrine knowledge` CLI** — `new <record_kind>` / `show` / `list` /
   `status`, riding SPEC-013's `<kind> <verb>` grammar, `CommonListArgs`,
   kind-relative `--status` known-set, canonical-id/JSON/columns (FR-007).
   Black-box per-verb goldens + the SPEC-013 parse-conformance matrix.
8. **DEC dual-namespacing (SPEC-019 D8).** The numbered kind is 2-part `DEC-NNN`.
   Disambiguate the live `decision_ref` code sites that the new kind makes
   misleading: the stale `src/rec.rs:318` comment (*"a DEC is … not a numbered
   entity kind"* — becomes false), the `relation_graph.rs`/`rec.rs` test
   fixtures, and the `src/main.rs:1537` `--decision` doc example. **Decide** the
   `DecisionRef` Unvalidated label's posture (keep free-text so external
   forgettable `DEC-NNN-XX` refs survive — recommended — vs validate numbered
   DEC). External forgettable citations stay 3-part prose, untouched.

## Non-Goals

- **The relation/spawn seam (FR-005)** — Slice B. No RECORD `RELATION_RULES`
  rows, no minted labels, no record `relation_edges` reader/accessor, no edges
  here. (The generic `[[relation]]` EOF-append attachment point exists once the
  KINDS rows land; B wires the rules + reader.) **Exception (design §4 L7, F-A1):**
  Slice A DOES add a four-prefix *empty* `outbound_for` arm
  (`"ASM"|"DEC"|"QUE"|"CON" => Ok(vec![])`) — pure routing so the KINDS-driven
  dispatch stays total and debug-safe (the `debug_assert!(false)` fallthrough
  would otherwise panic every debug-build graph scan once a record exists). This
  is routing, not a relation: no rules, no labels, no reader. B swaps the empty
  arm for the real accessor.
- **Supersession (FR-006)** — Slice C; gated on the unbuilt IMP-006 verb.
- **Direct gating** — IMP-047 (priority-engine `Gating` class). Interim is
  all-`Terminal`-inert; gating only via a spawned backlog proxy (out of scope
  here — that's a relation, Slice B).
- **The memory↔record seam** — OQ-1 / PRD-010 OQ-006/007, v2.
- **Renaming external forgettable `DEC-NNN-XX` citations** — provenance, never
  renumbered (D8).

## Affected surface

- `src/` — a new `knowledge`/`record` module (the one `knowledge_record`
  entity + the four `Kind` descriptors + per-kind status/facet enums + the
  `knowledge` command), riding the shared scaffold/render/transition seam.
- `src/integrity.rs` — `KINDS` rows (ordered golden).
- `src/priority/partition.rs` — four `KindPartition` entries + canaries.
- `src/main.rs` — the `knowledge` subcommand; the `--decision` doc example (D8).
- `src/rec.rs`, `src/relation.rs` — `decision_ref`/`Unvalidated` comments (D8).
- `src/relation_graph.rs` — `outbound_for` four-prefix empty arm (L7, F-A1:
  total-dispatch / debug-safe); D8 comment.
- `install/` + `.gitignore` — authored-entity wiring (manifest dir + negation,
  per `mem.pattern.install.authored-entity-wiring`).

## Risks / Assumptions / Open Questions

- **R1 — behaviour preservation (NF-001 / REQ-245).** Riding the shared scaffold must leave
  the slice / ADR / spec / backlog / memory suites green unchanged. The engine
  `Kind` is data, not a trait — the verb seam is not abstracted; variation is the
  kind table.
- **R2 — ordered-golden churn.** `KINDS` insertion position is load-bearing
  (`RELATION_RULES` is enum-`Ord`-ordered; the kinds-table golden is ordered).
  Pick the insertion point deliberately.
- **OQ1 — facet typed shapes. RESOLVED** (design §9): each field fixed as
  text/enum/list/date with `""`/`[]`→absent; plural fields (`alternatives`,
  `consequences`, `applies_to`) → list; `…_by` → text attribution; `…_on` → date
  (review finding M6).
- **OQ2 — `DecisionRef` posture under a numbered DEC. RESOLVED** (design §4 L3):
  keep free-text `Unvalidated` (external 3-part `DEC-NNN-XX` cites survive); D8
  work is comment/example disambiguation only, no behaviour change.
- **A1 — naming. RESOLVED** (design §4 L4): accepted as SPEC-019 proposes —
  `knowledge` namespace, `record-NNN` fileset, `.doctrine/knowledge/<kind>/`.
- **KINDS insertion point. RESOLVED** (design §4 L5): append after `REC`
  (`[…,"RV","REC","ASM","DEC","QUE","CON"]`), zero churn to other goldens.
- **Status modeling. RESOLVED** (design §4 L1): data-driven `&[&str]` per kind +
  `record_kind` lookup (not a typed enum). Facet is a typed enum-of-structs (L2).

## Summary

The first and independent slice of SPEC-019: stand up the four knowledge-record
kinds as one engine-discriminated entity with per-kind lifecycles, typed facets,
evidence, prefix→kind resolution, the never-`Workable` partition declaration, and
the `knowledge` CLI — no relations, no supersession, no gating. Ships alone.

## Follow-Ups

- **Slice B — relation seam (FR-005).** Captured as a backlog item. RECORD
  source-group + two minted labels (record→backlog-item relate, `spawns`) +
  `outbound_for` arm + record reader + coverage-invariant extension. Rides the
  shipped `link`/`unlink` (IMP-048 done). **Coordinates with SL-058** (relation
  surface tooling) and IMP-016/IMP-035 — shared `RELATION_RULES`/`outbound_for`
  sites; sequence, don't collide.
- **Slice C — supersession (FR-006).** Captured as a backlog item. The IMP-006
  transactional verb + the §6 cross-kind matrix + the `Supersedes` RECORD
  `LifecycleOnly` rule row. Gated on IMP-006.
- **IMP-053 — record↔record associative relation class** (surfaced at `/design`).
  No current SPEC-019 label covers QUE↔ASM / ASM→DEC associative intent; a new
  `RelationLabel` is a SPEC-019 amendment feeding Slice B.
- **IDE-006 — constraint owner + immutability/enforceability facet axis.**
- **IDE-007 — guidance: when a DEC record vs an ADR vs a governance surface.**
