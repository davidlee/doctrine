# Design SL-060: Cross-kind dep/seq capture: extend needs/after authoring beyond backlog

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

Realises IMP-033. Sequenced before IMP-047 (which rides this slice's seam).

## 1. Design Problem

The dep/sequence axis — hard `needs` (prerequisite, payload-free) and soft `after`
(sequence, per-edge `rank`) — is authored only on backlog items today. The intent
is cross-kind: a slice genuinely wants to sequence after another slice, or be
blocked until a prerequisite slice lands. There is no surface to author it, so the
priority engine's cross-kind `blockers`/`next` view never sees a slice-level
dependency.

The narrow goal: let a **work-like entity** (slices, in this slice) author its own
`needs`/`after`, and make the existing cross-kind consumer surface those edges —
without duplicating the backlog machinery, and without compromising the shipped
library to fix this repo's pre-existing data.

## 2. Current State

**The consumer is already cross-kind.** `priority/graph.rs` (SL-047) builds the
`dep_overlay` (`Reject`) and `seq_overlay` (`Evict`) over `EntityKey`
(corpus-wide prefix+id) and **emits dep/seq edges kind-agnostically** — DD-2 in
that module: *"today only backlog authors needs/after, so emitted
kind-agnostically."* The overlays, EntityKey identity, blocker chain, cycle
policy, `consequence` tally are all already kind-blind.

**The single backlog-bound point** is the dep/seq *read* loop
(`priority/graph.rs` §3b): it reads `backlog::dep_seq_for` only behind
`backlog::kind_from_prefix`. Non-backlog kinds silently contribute zero dep/seq
edges. That one gate is the consumer-side gap.

**The capture surface is welded to backlog.** The typed `Relationships{needs,
after}` schema, the edit-preserving `append_relationship` (`toml_edit`) write
seam, and the `dep_seq_for` read accessor all live inside `src/backlog.rs`,
keyed on `ItemKind` and backlog file paths.

**Slices carry no typed `[relationships]` table.** SL-048 "the cut" moved slice
tier-1 relations to `[[relation]]` array-of-tables rows; a slice TOML ends with
those rows. There is no typed relation table, and a naive tail-insert of
`needs`/`after` keys would land them *inside* the last `[[relation]]` element
(the F-1 corruption hazard `append_relationship` already guards).

**Canon ownership has a gap.** PRD-009 owns the `needs`/`after` authored schema
but scopes it to *a backlog item* (FR-010 / REQ-097, "item→item"), with a firm
boundary (backlog "does not own the change contract — that is a slice").
PRD-011 (cross-kind derived priority) explicitly disclaims the authored capture
seam as "PRD-009's … it does not redefine or duplicate it." So cross-kind dep/seq
*capture* is owned by neither spec.

**Distinct axis from `link` (ADR-010 D1).** `needs`/`after` are typed Tier-2
**payload** edges (`after.rank`), deliberately kept off `RELATION_RULES` / the
generic `[[relation]]` block. This slice does **not** extend `link`/`RelationLabel`.

## 3. Forces & Constraints

- **ADR-010 D1** — dep/seq stays typed, off the relation vocabulary (the `rank`
  payload is the reason). `needs`/`after` are NOT `link` labels.
- **ADR-004** — relations outbound-only; no reverse edge authored on the target.
- **ADR-001** — layering: the new schema/write/read is a leaf; the dispatch is
  engine; the verb is command. No cycle.
- **No parallel implementation** (user standard) — the dep/seq schema+write+read
  must be *lifted* into one shared unit, not copied per kind.
- **Don't compromise the product for project-local ops**
  (`mem.pattern.design.product-not-compromised-by-project-local-ops`) — durable
  code stays strict; the pre-existing-slices-lack-table gap is fixed out-of-band.
- **Behaviour-preservation gate** — backlog `needs`/`after`, `priority`, and
  `backlog order` suites stay green unchanged (the lift is internal).
- **Storage rule** — typed data in TOML; the seeded `[relationships]` table
  precedes all `[[relation]]` arrays (F-1 / SL-048 R2-m1 ordering).
- **Determinism** — BTree only; no clock/RNG/HashMap iteration order.

## 4. Guiding Principles

- Generalise the seam, do not widen the blast radius. The consumer is one gate
  away from cross-kind; change the gate, not the overlay machinery.
- Lift once, dispatch per kind — mirror the established `outbound_for` /
  `status_and_title_for` per-kind dispatch shape; introduce no new pattern.
- Keep two producers convergent but unentangled: SL-060 owns the typed-needs/after
  producer into `dep_overlay`; IMP-047 later adds the labelled-`gates` producer
  into the same overlay. Design the overlay open; do not unify the producers here.
- Canon moves first: the cross-kind product (PRD-011) claims the cross-kind
  capture intent before the design is built on it.

## 5. Proposed Design

### 5.1 System Model

```
author:  doctrine needs/after <SRC> <TGT>     (command)
            │  validate (resolve, authoring-kind, self-edge) — shell
            ▼
         dep_seq::append(toml_path, edit)       (leaf — edit-preserving, F-1 strict)
            │
   ┌────────┴─────────┐  storage: <kind>/<id>/<stem>.toml  [relationships].needs/.after
 backlog/033/…       slice/060/…

read/consume:
  relation_graph::scan_entities ──▶ priority/graph.rs §3b
                                       dep_seq_for(root, kind, id)   (engine dispatch)
                                          ├ backlog → leaf.read
                                          ├ slice   → leaf.read
                                          └ else    → empty
                                       ▼
                                    dep_overlay (Reject) / seq_overlay (Evict)
                                       — edge emission ALREADY kind-agnostic (DD-2)
                                       ▼
                                survey / next / blockers / explain
```

### 5.2 Interfaces & Contracts

**New leaf `src/dep_seq.rs`:**

```rust
pub(crate) struct AfterEdge { pub to: String, pub rank: i32 }      // lifted from backlog
pub(crate) struct DepSeq    { pub needs: Vec<String>, pub after: Vec<AfterEdge> }
//  NOTE: `promoted` does NOT move here — it is a backlog projection detail
//  (resolution == Promoted), read backlog-only in the priority attrs pass.

pub(crate) enum RelEdit<'a> { Needs(&'a [String]), After { to: &'a str, rank: i32 } }

/// Read the typed [relationships] dep/seq block from an entity's TOML.
/// Absent table → empty DepSeq (a kind that does not author dep/seq).
pub(crate) fn read(toml_path: &Path) -> anyhow::Result<DepSeq>;

/// Edit-preserving append into [relationships].{needs|after} (the lifted
/// `append_relationship` body). STRICT F-1 refuse if the seeded table/array is
/// absent — never creates it (scaffold guarantees it; out-of-band backfill seeds
/// pre-existing entities). Idempotent.
pub(crate) fn append(toml_path: &Path, edit: &RelEdit<'_>) -> anyhow::Result<()>;
```

**Engine dispatch (mirrors `outbound_for`), in `relation_graph.rs` or beside it:**

```rust
/// Map (kind, id) → its TOML path and read its DepSeq via the leaf. Authoring
/// kinds: backlog (5) + slice. Every other kind returns the empty DepSeq.
pub(crate) fn dep_seq_for(root, kind: &entity::Kind, id: u32) -> anyhow::Result<DepSeq>;
```

**Command (`doctrine needs` / `doctrine after`, top-level, sibling to `link`):**

```
needs  <SRC> <TGT>            append TGT to SRC.needs
after  <SRC> <TGT> [--rank N] append { to: TGT, rank: N (default 0) } to SRC.after
```

`backlog needs`/`backlog after` retained as thin delegates to the same leaf path.

### 5.3 Data, State & Ownership

- **Storage** — unchanged shape, new home: `[relationships].needs = [ids]`,
  `[relationships].after = [{to, rank}]` in the entity's own
  `<kind>/<id>/<stem>.toml`. Slices gain a seeded `[relationships]` table
  (scaffold) positioned **before** any `[[relation]]` rows.
- **Ownership** — each authoring entity owns its outbound dep/seq (ADR-004
  outbound-only; PRD-011 "each entity is the source of its own intent"). The
  priority engine *derives* blockers/order; never writes back.
- **`promoted`** — backlog-only; remains in the priority adapter's attrs pass,
  read for backlog kinds only (every other kind: `false`).

### 5.4 Lifecycle, Operations & Dynamics

- Author-time validation (command shell, before the leaf write):
  1. SRC resolves and is a dep/seq-authoring kind (else refuse).
  2. TGT resolves against `integrity::KINDS` (forward-edge resolvability; **no
     kind-pair gate** — OQ-2 / D2). Free-text targets rejected (unlike backlog
     `drift`).
  3. Self-edge (SRC == TGT) refused.
- No author-time cycle check — cordage diagnoses cycles at read (`dep` Reject /
  `seq` Evict), matching backlog today.
- Read/consume: `priority` build reads each scanned entity's DepSeq via the
  dispatch; emits `needs`→`dep_overlay` (B→A flip, `EdgeAttrs::new(0,0)`),
  `after`→`seq_overlay` (B→A flip, `EdgeAttrs::new(rank, age)`, age = array
  index). Unchanged emission logic.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** seeded `[relationships]` table precedes every `[[relation]]` array in
  authored TOML (F-1; SL-048 R2-m1). Pinned by a round-trip golden.
- **INV-2** the lift is byte-identical for backlog: same reads, same writes, same
  `dep_seq_for` results, same goldens.
- **INV-3** outbound-only — the verb writes only on SRC; never touches TGT.
- **Edge** dangling target (TGT later deleted) — contributes no edge at read
  (resolve-only), same as backlog today; not an author-time error beyond initial
  resolvability.
- **Edge** a slice `needs` a terminal-status slice — surfaces per the priority
  partition (terminal predecessor does not block); SL-060 changes no partition
  semantics (that is IMP-047).
- **Assumption** `promoted` has no slice analogue; slices are never `promoted`.

## 6. Open Questions & Unknowns

- **OQ-1 — DISSOLVED.** "Which projector carries cross-kind dep/seq" — the
  `priority` graph already does, kind-agnostically (DD-2). `backlog_order.rs`
  (legacy `backlog order`, ItemKind-bound) is untouched.
- **OQ-2 — RESOLVED (D2).** No kind-pair legal table; targets validated for
  resolvability only. Gradient inversion (e.g. work depending on governance) is
  not a needs/after concern — it is the future Revision kind (IDE-010).
- **PARKED — semantic edge labels (forelobe).** Whether dep edges should carry a
  distinguishing label (`needs` vs IMP-047 `gates` vs `blocks`) rather than
  collapsing into one unlabelled `dep_overlay`. Not decided here; IMP-047 forces
  the question when it adds `gates`. SL-060 keeps `needs`/`after` unlabelled.

## 7. Decisions, Rationale & Alternatives

- **D1 — Lift the dep/seq schema+write+read into a shared leaf `src/dep_seq.rs`.**
  Rejected: per-kind copy (parallel implementation); generic over `entity::Kind`
  (fights "Kind is data, not a trait"). The leaf is the smallest honest DRY move.
- **D2 — Unrestricted targets (resolvability only), no kind-pair gate.** Backlog's
  prior art has no target-kind gate; a legal table would be new asymmetric policy
  invented here, and IMP-047's intended topology is deliberately cross-tier.
  Gradient is convention, not enforcement.
- **D3 — Slices only as the source kind this slice activates.** Specs/ADRs as
  dep/seq sources model the wrong thing — depending on governance is *pending
  revise-intent*, the future Revision kind (IDE-010), not a `needs` edge.
- **D4 — Generic top-level verbs `doctrine needs`/`after`** (sibling to `link`),
  backlog verbs delegate. Rejected: per-kind subcommands (more surface, hides the
  cross-kind generality).
- **D5 — Strict leaf + scaffold seed + out-of-band backfill** (SL-048 precedent;
  `mem.pattern.design.product-not-compromised-by-project-local-ops`). Rejected:
  create-on-absent leniency in the leaf (bends the product for a transient local
  gap); a shipped `migrate` verb (over-build for dogfood-only).
- **D6 — Canon moves first: amend PRD-011** to claim cross-kind dep/seq capture
  intent. Rejected: widen PRD-009 FR-010 (fights its backlog boundary); judge it
  already-permitted (leaves PRD-009's "backlog item" wording as a latent
  contradiction).
- **D7 — `dep_overlay` left open for IMP-047.** SL-060 generalises the typed
  producer only; the labelled-`gates` producer is IMP-047's. No producer
  unification here.

## 8. Risks & Mitigations

- **R1 — toml_edit positioning of a seeded table before `[[relation]]` rows.** The
  one genuine impl risk. Mitigate: scaffold authors the order directly (template);
  the leaf never *creates* the table (strict), so positioning is a
  scaffold/backfill concern, not a runtime-write concern. Pin with a round-trip
  golden (INV-1) + SL-048-style storage post-check on backfilled files.
- **R2 — the lift perturbs backlog behaviour.** Mitigate: INV-2 byte-identical
  gate; backlog verbs/goldens run unchanged; the lift is mechanical (move + thin
  delegate).
- **R3 — `promoted` accidentally folded into the leaf.** Mitigate: explicit D-note
  that it stays backlog-only; the leaf `DepSeq` has no `promoted` field.
- **R4 — scope creep into IMP-047.** Mitigate: D7 + §5.5 edge — no partition /
  `Gating` / `gates`-label work; SL-060 ends at "typed cross-kind needs/after
  surfaces in the existing binary-partition blocker view."

## 9. Quality Engineering & Validation

- **Behaviour preservation:** backlog `needs`/`after`, `priority` (survey/next/
  blockers/explain), `backlog order` goldens byte-identical.
- **New behaviour:**
  - slice→slice `needs`/`after` authorable via `doctrine needs`/`after`;
    round-trips `slice show` / `show --json` (black-box golden).
  - an authored slice→slice `needs` surfaces as a cross-kind blocker in
    `priority blockers <SL>` and holds the dependent in `next`.
  - `after` rank/age ordering across slices behaves as the backlog seq overlay.
- **Validation refusals:** unresolvable TGT, free-text TGT, self-edge, non-authoring
  SRC kind — each refused with a clear message; tested.
- **Migration oracle (out-of-band backfill):** SL-048-style storage post-check —
  seeded `[relationships]` precedes all `[[relation]]` arrays; round-trip `show` +
  `validate` clean across every backfilled slice.
- `just gate` clean; clippy zero warnings.

## 10. Review Notes

(adversarial pass pending — §6 PARKED + R1 are the prime targets.)
