# SL-158 Design — Trinary actionability

> Governing canon: **ADR-017** (gating = inbound `needs` on an unsettled record).
> This design formalises the implementation of ADR-017 and corrects one false
> premise in its prose (see D2). Decisions locked with the user in the `/design`
> pass (2026-06-26). `notes.md` carries the exploration trail.

## 1. Design decisions

### D1 — `Gating`: a third status class (the sole *partition* delta)

`priority::partition::StatusClass` gains a third variant, `Gating` — **non-`Workable`,
non-`Terminal`**. It splits the two predicates the binary model fused:

| predicate | reads | effect |
|---|---|---|
| `eligible` (worklist) | `== Workable` | a `Gating` node never surfaces as work |
| `blocks` (dep overlay) | `!= Terminal`  | a `Gating` node blocks its dependents |

A node settling from `Gating` → `Terminal` stops blocking and unblocks its dependent
— **for free**, because `channels::blocked_by` already keeps only `!= Terminal`
predecessors. `channels.rs` takes **no code change**; the new variant slots into the
existing pole reads.

`KindPartition` grows a third set:

```rust
struct KindPartition {
    prefix: &'static str,
    workable: &'static [&'static str],
    gating:   &'static [&'static str],   // NEW — non-workable, non-terminal
    terminal: &'static [&'static str],
}
```

`status_class` checks `workable → Workable`, `gating → Gating`, `terminal → Terminal`,
else `Unrecognised`. The four knowledge rows move their unsettled states into `gating`:

| kind | `gating` (unsettled) | `terminal` (settled) |
|---|---|---|
| ASM | `held`, `testing` | `validated`, `invalidated`, `obsolete` |
| DEC | `proposed` | `accepted`, `rejected`, `superseded` |
| QUE | `open` | `answered`, `obsolete` |
| CON | `active` | `waived`, `superseded`, `retired` |

Every **non-knowledge** row keeps `gating: &[]` (empty) — its behaviour is byte-identical
to today (the three-way cover reduces to the old binary wherever `gating` is empty).

The VT-1 drift canary generalises: `workable ∪ gating ∪ terminal == <KIND>_STATUSES`.

### D2 — Target-admissibility gate widening (the delta ADR-017 missed)

**ADR-017's premise that the `needs` work-like gate is *source-only* is false in the
current code.** `commands/dep_seq.rs::resolve_dep_seq_src` gates the **target** as
work-like too (`is_work_like(tkref.kind)`), so `doctrine needs SL-x QUE-1` is refused
today (*"cross-tier dep/seq is not yet allowed"*). Without this change, no dependent
can author the inbound gate ADR-017 mandates.

Split the one predicate into two:

```rust
// SOURCE gate — UNCHANGED. Records still cannot AUTHOR dep/seq (ADR-017 §3:
// no record-as-dep/seq-author surface, is_work_like not widened).
fn is_work_like(kind) -> bool      // { SL, ISS, IMP, CHR, RSK, IDE, REV }

// TARGET gate — NEW, wider. A work item may now `needs`/`after` a record.
fn is_admissible_dep_target(kind) -> bool   // is_work_like(kind) ∨ is_record(kind)
//                                            records = { ASM, DEC, QUE, CON }
```

- **Governance stays excluded** (SPEC/ADR/POL/STD) — depending on canon routes through
  a Revision (the SL-060 invariant). Target set = `work-like ∪ records`, *not* all kinds.
- **record→record `needs` is excluded for free** — the *source* gate (`is_work_like`)
  already refuses a record as author. ADR-017 §5's "gating-inert" becomes
  "un-authorable" — strictly stronger, consistent.
- Admissibility is **structural, not status-keyed**: `needs → settled-record` is just an
  already-satisfied prereq (mirrors `needs → done-slice`). No status filter on the gate.
- The refusal message updates to reflect records-now-allowed; governance still refused.

The edge then rides the **existing** kind-agnostic `graph.rs` build (`needs` resolves
any scanned target, emits `prereq→src`) — `graph.rs` is untouched.

**`after` rides the same gate.** `resolve_dep_seq_src` is shared by `needs` *and*
`after`, so widening the target predicate admits records as `after` targets too. This
is **inert, not harmful**: `after` is soft sequence among workable items, and a record
is never `eligible`, so it never participates in the seq ordering — `SL after QUE` writes
a seq edge that orders nothing. We accept the shared widening (one predicate, minimal
surface) rather than split a per-verb gate; the meaningful gating path is `needs`.

### D3 — estimate/value admissible on records (confirmatory; no code)

`estimate` / `value` are already kind-agnostic (`estimate.rs`; no kind gate in
`facet.rs`; `id_path` resolves the knowledge path). Writing `[estimate]`/`[value]` to a
record round-trips — `RawRecordToml` has no `deny_unknown_fields`, so the table is
ignored by the knowledge reader, not rejected. **Confirmed in the CLI.** This design
records the intent; a VT pins the round-trip.

- `risk` stays **excluded**: kind-gated to risk-items, *and* its `[facet]` table name
  collides with knowledge records' typed kind-facet `[facet]`. Do not widen risk.
- **Scoring wiring (corrected — two paths, opposite verdicts):**
  - *via leverage — INERT.* The leverage DP flows *dependent → prereq* (`graph.rs:513`);
    a record is always a prereq with **no dep predecessors** (it can't author `needs`), so
    its base never propagates to anyone through the dep overlay.
  - *via optionality — LIVE (with D6).* A record's base **does** propagate to the targets
    it `references` — `optionality(N) = ref_coeff · Σ base(referrers)` over
    `CONSEQUENCE_LABELS`, which includes `References` (`graph.rs:163`, filtered by label,
    role-blind). Once D6 lets records author `references`, an estimated record raises its
    referenced target's optionality. **This is the purpose** estimate/value on records earns.
  - *Visibility (codex correction).* A record's own `score` **is** surfaced — `survey --all`
    bypasses the eligibility filter (`surface.rs:136`) and `explain`/`inspect` render score
    + the raw `StatusClass` for any minted id (`render.rs:184`). So this is **not** a hidden
    quantity; the earlier "never displayed" claim was wrong.
- Surfacing estimate/value in `show`/`inspect` → **IMP-183** (out of scope).

### D6 — Records may author `references` (concerns)

A knowledge record cannot author `references` today — `link QUE-171 references SL-158`
is refused (*"QUE may not author `references`; legal: shapes, spawns, governed_by"*).
`RECORD` is absent from every `references` source-set (`relation.rs:331-366`). That is a
gratuitous limit: a record should be able to declare it is *about* an artefact.

Add `RECORD` (ASM/DEC/QUE/CON) as a legal `references` source under the **`concerns`**
role (aboutness/relevance; target `AnyNumbered`) — the role that already serves
work→any-numbered (`relation.rs:355-366`). `shapes` (epistemic influence) stays distinct;
`concerns` is plain aboutness. This is **not** `shapes`-roles (IDE-022) — a different,
independent edit to the same file.

- **Layer:** `src/relation.rs` `RELATION_RULES` only (authoring permission). No gating,
  no dep/seq — `references` is a ref/consequence overlay, never the dep overlay, so **no
  cycle or blocking effect**.
- **Scoring interaction (intended):** a record's `references` edge feeds its target's
  optionality (see D3). With most records carrying no `estimate`/`value` (base 0), the
  contribution is 0 by default; it activates only when a record is deliberately sized.
- **Inbound render:** `concerns` inbound name is "concerned by" (existing).

### D4 — Canon moves first

SPEC-001 / PRD-011 gain a D-decision + requirement for: the third status class; the
`eligible`-vs-`blocks` split; records as admissible `needs` targets. SPEC-019
D7 / NF-003 / OQ-2 revise — records become `Gating` (unsettled) / `Terminal` (settled),
no longer all-inert. Authored through design→reconcile, **not** hand-edited ahead of
the engine. ADR-017's source-only premise (D2) is reconciled in ADR prose at close.

### D5 — Split out of scope

- **`shapes`-roles** (semantic disambiguation, ADR-016) → **IDE-022**. Different layer
  (semantic `shapes`, graph-inert), carries its own open "do they earn their keep"
  question; bundling a settled slice behind an unsettled question is the re-park trap.
- Outbound gating authoring stays a derived hub-view + deferred batch sugar (ADR-017 §3).

## 2. Current vs target behaviour

| scenario | today | target |
|---|---|---|
| `status_class(QUE, "open")` | `Terminal` | `Gating` |
| `status_class(QUE, "answered")` | `Terminal` | `Terminal` |
| `status_class(SL, "design")` | `Workable` | `Workable` (unchanged) |
| `doctrine needs SL-1 QUE-1` | **refused** (cross-tier) | accepted; emits `QUE-1→SL-1` |
| `SL-1 needs QUE-1`, QUE-1 `open` | n/a (un-authorable) | SL-1 **blocked** by QUE-1 |
| QUE-1 → `answered` | n/a | SL-1 **unblocked** |
| `doctrine needs SL-1 ADR-1` | refused | **still refused** (governance) |
| `doctrine needs QUE-1 SL-1` | refused | **still refused** (record can't author) |
| `link QUE-1 references SL-1` | **refused** (illegal source) | accepted (concerns role) — D6 |
| `estimate set QUE-1 …` | writes (works) | writes; now feeds optionality of what QUE-1 `references` |

## 3. Code impact (design-target touch-set)

- **`src/priority/partition.rs`** — add `Gating` variant; add `gating` field to
  `KindPartition`; `status_class` checks the `gating` set; move the four knowledge rows'
  unsettled states into `gating`; every other row `gating: &[]`. Generalise the `vocab`
  test helper + the VT-1 canaries to the three-way cover.
- **`src/commands/dep_seq.rs`** — add `is_record` + `is_admissible_dep_target`; swap the
  *target* check in `resolve_dep_seq_src` to the wider predicate; update the refusal
  message; add admission + refusal tests.
- **`src/relation.rs`** — add `RECORD` to a `references`/`concerns` source-set (D6);
  authoring-test for `link QUE references SL`.
- **`src/knowledge.rs`** — D3 confirmatory test (VT-7): estimate round-trip on a record
  (auxiliary test infrastructure, not a production seam).
- **`src/relation_graph.rs`** — fixture updates: add `references(concerns)` edges to
  knowledge-kind seeds in per-kind edge-label tests (auxiliary test infrastructure).
- **`.doctrine/spec/tech/001/` (SPEC-001) + PRD-011** — D-decision + requirement (D4).
- **`.doctrine/spec/tech/019/` (SPEC-019)** — D7 / NF-003 / OQ-2 revision (D4).

**No *code* change, but observable OUTPUT flips (codex):** `channels.rs`, `graph.rs`,
`surface.rs`, `view.rs`, `render.rs` are not edited — predicates use `== Workable` /
`!= Terminal` (not exhaustive `match`es) and `render.rs:184` prints the class via
`{:?}` (Debug), so a new `Gating` variant compiles untouched. **But** `survey --all`,
`explain`, and `inspect` render `StatusClass` for any minted node, so an unsettled
record's eligibility/score output flips `Terminal → Gating`. Intended; pinned by a
knowledge-in-priority golden (VT-8). `relation.rs` IS edited (D6); `shapes`-roles still
split to IDE-022.

## 4. Verification

- **VT-1 (canary, generalised):** `workable ∪ gating ∪ terminal == <KIND>_STATUSES` per
  partitioned kind. Non-knowledge rows: `gating == ∅`.
- **VT-2 (class boundary):** each knowledge kind — unsettled status → `Gating`, settled
  → `Terminal`, never `Workable`.
- **VT-3 (gate, blocks):** `SL needs QUE` with QUE `open` → SL blocked; SL not actionable.
- **VT-4 (gate, settle→unblock):** flip QUE → `answered` → SL unblocked, actionable.
- **VT-5 (record never eligible):** a `Gating` record is not in `eligible`/the worklist.
- **VT-6 (admissibility, the ADR-017 VT that fails today):** `doctrine needs SL-1 QUE-1`
  resolves + writes the edge; a governance target (ADR/POL/STD/SPEC) still refused; a
  record *source* still refused.
- **VT-7 (estimate round-trip):** `estimate set ASM-1 …` writes; `knowledge show ASM-1`
  reads back clean (table ignored, not rejected).
- **VT-8 (knowledge-in-priority golden, codex):** an e2e golden over a corpus including an
  unsettled + a settled record — `explain`/`inspect`/`survey --all` render `Gating` for the
  unsettled record and `Terminal` for the settled one. Pins the intended output flip the
  existing priority goldens (work/backlog/review only) don't cover.
- **VT-9 (references authoring, D6):** `link QUE-1 references SL-1 [--role concerns]` is
  accepted (was refused); the edge renders inbound as "concerned by" on SL-1; a record with
  an `[estimate]`/`[value]` raises SL-1's optionality by `ref_coeff · base(record)`.

### Tests that flip **by design** (consumer revision, not regression)

- `partition::tests::every_knowledge_status_classifies_terminal_never_workable` — now
  asserts unsettled → `Gating`, settled → `Terminal`.
- `partition::tests::knowledge_partitions_cover_the_real_vocabularies` — canary form
  becomes the three-way cover.
- `partition::tests::decision_accepted_diverges_hidden_from_status_class` — **stays
  green** (`accepted` ∈ DEC terminal → still `Terminal`).

## 5. Invariants & behaviour-preservation

- **Reduce-to-binary:** wherever `gating == ∅` (every non-knowledge kind), the three-way
  cover is byte-identical to today. The existing priority suites are the proof and stay
  green unchanged (the entity-engine behaviour-preservation gate).
- **No new relation vocabulary, no new overlay** (ADR-017). `graph.rs`/`channels.rs`
  untouched.
- **Source gate unchanged** → no record becomes a dep/seq author; `is_work_like` not
  widened (ADR-017 §3 negative consequence honoured).
- **Cycle safety:** record→record `needs` is un-authorable (source gate); the dep overlay
  `CyclePolicy::Reject` remains the backstop for any cross-kind cycle.

## 6. Open questions

None blocking. Transitive canon gating (an `active` CON gating work through the spec it
shapes) is **deferred** (ADR-017 §4) — gates here are direct `needs → record` only.
