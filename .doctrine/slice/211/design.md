# Design SL-211: Split-lineage close recovery

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

A dispatched slice whose reviewed code reaches trunk by a sanctioned route that
leaves **no journal trunk row** — operator direct-land after a candidate
conflict, a manual merge when pre-dispatch `edge→main` promotion was skipped,
external integration — cannot reach `done`. The close-gate
(`ledger.rs::trunk_integration`) demands a trunk row that no verb writes for this
lineage shape, so the slice is shipped-but-lifecycle-incomplete (SL-147 stranded
at `reconcile`; SL-190 hand-edited `journal.toml` to force `done`).

Make the variation **recordable** by a verb, and let the existing gate
**recognise** the recorded row — without touching ADR-012 FF-only, and without a
`--force` bypass of the integrity the gate exists to give.

Governing frame: **RFC-016 §C** (operator-carried contract → refusal-with-
prescription) + **§D** (every legal variation gets a first-class recorded row at
the point of variation; the contrapositive — a variation that cannot get a row is
refused *there*, not discovered at close). SL-211 is the §C/§D beachhead, not the
whole zero-rescue machine.

## 2. Current State

The close chain is a ladder: `slice status done` refuses without a journal trunk
row → only `sync --integrate` writes one → integrate refuses without an admitted
`close_target` (candidate path) or a fast-forwardable phase tip (legacy path).

**The gate already accepts the shape we need.** `trunk_integration`
(`ledger.rs:444`) passes iff the journal has a trunk row whose `planned_new_oid`
is an **ancestor of the live trunk tip** (`ledger.rs:497`,
`is_ancestor(planned, tip)`) — it does *not* require planned to *be* the tip. So
"the reviewed commit is somewhere in trunk's history" already satisfies the gate.
The missing piece is a verb that writes such a row.

**Why neither existing planner writes it.** Both *advance* trunk fast-forward-only
— they assert `is_ancestor(trunk_tip, planned)`, i.e. planned must **descend**
trunk:

- `plan_trunk_row` (`dispatch.rs:2352`, legacy) → phase-chain tip.
- `plan_candidate_trunk_row` (`dispatch.rs:2392`, candidate) → admitted
  `close_target.admitted_oid`.

In a manual-merge land, trunk holds a **merge commit** and the reviewed tip is an
**ancestor, not descendant** → non-ff → refused. That is the deadlock SL-190 broke
by hand.

`close-preff-trunk-absorbs-repair` covers only the *linear-FF* subset
(`git branch -f main <tip>` where the reviewed tip descends trunk); it cannot move
trunk backward to an ancestor, so it does **not** cover the split-lineage merge
shape (resolves OQ-3: negative).

## 3. Forces & Constraints

- **ADR-012 D2/D4 — FF-only, expected-tip, never auto-non-ff, report-not-resolve.**
  The recorder must not advance trunk at all; it records that integration already
  occurred out-of-band. Orthogonal to FF-only, not an amendment of it.
- **SPEC-022 — "coverage/lineage is reconciled, never inferred."** The row is a
  *recorded fact*, earned at record time and re-verifiable at the gate.
- **STD-001** — no magic strings (ref names, stage flag via named constants where
  the module already defines them, e.g. `DISPATCH_REF_PREFIX`).
- **ADR-001** — layering leaf←engine←command, no cycles. `ledger` stays
  ref-agnostic (unchanged); the trunk-ref literal and refusal copy live in the
  `slice`/`dispatch` shells.
- **Behaviour-preservation gate** — the entity engine / dispatch suites are the
  proof; native clean-base integrate+close must stay byte-unchanged.
- **A1 (pure/imperative split)** — ref/merge ops in the thin shell; row
  planning/validation takes OIDs in.
- **No parallel implementation (R2)** — one recorder/gate seam; the earned-surface
  resolution is *shared* with the existing planners, not duplicated.

## 4. Guiding Principles

- The gate stays mechanical and unchanged; earnedness is enforced by the **verb**,
  and independently **re-verified** by the gate (belt-and-suspenders).
- One verb, two sources, unified through the resolution the integrate path already
  uses (`candidate_active ? admitted close_target : phase-chain tip`).
- The recorder is *terminal* for the trunk leg — it exists precisely because
  integrate cannot run for this lineage.

## 5. Proposed Design

### 5.1 System Model

One new **sync stage** — `--record-integration` — beside `--prepare-review` and
`--integrate`. It resolves the *earned surface* OID from the ledger (shared
resolution), asserts that OID is already an ancestor of live trunk, and commits a
single **Verified** trunk row to `dispatch/<N>`. It mutates no external ref. The
unchanged gate then reads that row and passes the slice to `done`.

```
                       ┌─ --prepare-review → review/<N> + phase/<N>-NN (no trunk)
 dispatch sync --slice ┼─ --integrate      → advance trunk ff-only (descendant)   ── existing
                       └─ --record-integration → record row: reviewed_oid ⊑ trunk  ── NEW (SL-211)
                                                     │
 slice status … done → trunk_integration ──────────┘ (unchanged; accepts ancestor row)
```

### 5.2 Interfaces & Contracts

**CLI** — new variant in the `Sync` `stage` group (`dispatch.rs:49`):
```
dispatch sync --slice N --record-integration --trunk <ref>
```
- In the mutually-exclusive `stage` group (`conflicts_with` prepare_review /
  integrate / show_journal_trunk_oid).
- `--trunk <ref>` required for this stage.
- Orchestrator-classed — refused under worker-mode (same as the other stages).

**Shared source resolution (R2 — one seam).** Extract the earned-surface lookup
`integrate()` does inline (`dispatch.rs:2044–2055`):
```rust
fn resolve_trunk_payload(
    root: &Path, slice3: &str, journal: &Journal, candidates: &Candidates,
) -> anyhow::Result<String /* oid */> {
    if candidates.rows.is_empty() {
        let phase_ref = phase_chain_tip(journal, slice3)
            .with_context(|| format!("no phase/{slice3}-NN code units"))?;
        resolve_commit(root, &phase_ref)?
            .with_context(|| format!("{phase_ref} does not resolve"))
    } else {
        candidates.current_admission.close_target.as_ref()
            .context("candidate workflow active but no close_target admission — \
                      run `dispatch candidate admit --role close_target` first")
            .map(|a| a.admitted_oid.clone())
    }
}
```
`plan_trunk_row` and `plan_candidate_trunk_row` are refactored to call it (they
then only add `expected_old` + the *descendant* assertion). One resolution, three
consumers.

**Planner (pure over OIDs; is_ancestor is the thin-shell git seam):**
```rust
fn plan_recorded_trunk_row(
    root: &Path, slice3: &str, journal: &Journal,
    candidates: &Candidates, trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let reviewed = resolve_trunk_payload(root, slice3, journal, candidates)?;
    let tip = resolve_commit(root, trunk_ref)?
        .with_context(|| format!(
            "record-integration: {trunk_ref} does not resolve — no trunk to record onto"))?;
    // EARNED CHECK (R1 negative): the reviewed surface must already be on trunk.
    anyhow::ensure!(
        git::is_ancestor(root, &reviewed, &tip)?,
        "record-integration: reviewed surface {reviewed} is not an ancestor of \
         {trunk_ref} (at {tip}) — the reviewed code has not landed on trunk; land \
         it (manual merge / integrate) before recording"
    );
    Ok(recorded_row(trunk_ref, reviewed))
}
```

**Handler:** `run_record_integration(path, slice, trunk_ref)` → `root::find` →
worker-mode class check (as the other sync stages) → resolve `dispatch/<N>` tip,
tree-read `journal.toml` + candidates → `fresh` guard (target already journaled ⇒
"already recorded", no-op) → `plan_recorded_trunk_row` → append → `commit_journal`
onto `dispatch/<N>`. **No `with_journaled_projection`** (no advance).

### 5.3 Data, State & Ownership

**Row shape** — `recorded_row(trunk_ref, reviewed_oid)`:

| field | value | why |
|---|---|---|
| `target_ref` | `trunk_ref` | the row the gate filters for |
| `source_oid` | `reviewed_oid` | earned surface (evidence) |
| `planned_new_oid` | `reviewed_oid` | the gate re-checks `is_ancestor(this, trunk)` |
| `applied_new_oid` | `reviewed_oid` | already applied — terminal, not pending |
| `expected_old_oid` | `reviewed_oid` | **= planned**, *not* the trunk tip |
| `status` | `Verified` | statement of fact, not intent |

`expected_old = planned` (not `trunk_tip`) is load-bearing for replay-safety: a
stray later `sync --integrate --trunk` runs `advance_row` on this row; with
`current(trunk_tip) != planned` and `current != expected_old`, it classifies as
`Refused` (non-destructive — Refused mutates nothing). Had `expected_old` been the
trunk tip, replay would try to advance trunk *backward* to the ancestor — the one
outcome we must forbid.

Ownership: the row lives in `journal.toml` on `dispatch/<N>` (authored-adjacent
run ledger, object-db sourced), written by the recorder alone.

### 5.4 Lifecycle, Operations & Dynamics

Recovery flow (replaces the SL-190 hand-edit):
1. Operator lands the reviewed code on trunk out-of-band (manual `merge --no-ff
   review/<N>`, or a direct-land) — the reviewed tip becomes an ancestor of trunk.
2. `dispatch sync --slice N --record-integration --trunk <ref>` — earned check
   passes; Verified trunk row committed.
3. `slice status N done` — `trunk_integration` reads the row, re-verifies
   `is_ancestor(reviewed, trunk)` → `Integrated` → passes.

The recorder is terminal for the trunk leg; `--edge` aggregation, if wanted, stays
the separate `--integrate --edge` path (not gated by `done`).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** Recorder writes **no external ref** (only the `dispatch/<N>` journal
  commit). FF-only invariant untouched.
- **INV-2** A recorded row's `planned_new_oid` is always an ancestor of the trunk
  tip at record time (the earned check); the gate re-asserts it at `done`.
- **INV-3** `ledger.rs::trunk_integration` is **unchanged**.
- **EDGE — trunk ref absent** → refuse ("no trunk to record onto").
- **EDGE — reviewed surface not on trunk** → refuse (R1 negative); no row written.
- **EDGE — already recorded** (`fresh` false) → no-op, no duplicate row.
- **EDGE — candidate-active but no `close_target` admission** → refuse (inherited
  from `resolve_trunk_payload`, same message the integrate path gives).
- **EDGE — trunk tip == reviewed tip** (clean ff already happened elsewhere) →
  planned == tip → row still valid; a later replay is a clean NoOp.

## 6. Open Questions & Unknowns

- **OQ-1 (housekeeping, not design-blocking)** — stale IMP-169 reservation
  `refs/doctrine/reservation/IMP/169` (2026-06-24, empty tree). Confirm no active
  drive and reap before execute.
- **OQ-2 — RESOLVED:** one verb, two sources unified through
  `resolve_trunk_payload` (§5.2). Direct-land vs external-merge are *scenarios*,
  not distinct sources.
- **OQ-3 — RESOLVED (negative):** `close-preff-trunk-absorbs-repair` covers only
  the linear-FF subset; the recorder is still required for the ancestor-not-tip
  merge shape (§2). The recorder also subsumes the pre-FF dance, but does not
  retire it.

## 7. Decisions, Rationale & Alternatives

- **D1 — the recorded row carries the reviewed OID (planned = reviewed), and the
  gate re-verifies.** Alternative (B) recorded the live trunk tip → gate check
  `is_ancestor(tip, tip)` is trivially true → rubber stamp; all integrity resting
  on one record-time assertion. Rejected per R1. (A) keeps the gate a live,
  re-checkable invariant and records *which* reviewed commit landed.
- **D2 — `expected_old = planned`, not the trunk tip.** Makes a stray later
  integrate a non-destructive `Refused` rather than a backward trunk advance
  (§5.3).
- **D3 — one verb, source resolution shared with the integrate planners** (R2).
  No parallel implementation; existing planner tests prove the extraction.
- **D4 — earnedness in the verb; the gate leaf and the prescription copy stay
  mechanical.** The slice-shell prescription (§ below) is an unconditional
  signpost; it does not re-derive earnedness (no is_ancestor duplicated into
  `slice.rs`).
- **D5 — `ledger.rs` unchanged.** The gate already accepts an ancestor row;
  behaviour-preservation by construction.

**Prescription (IMP-169, RFC-016 §C)** — copy edit in the slice close-gate shell
(`slice.rs:1016–1020`). The current Blocked refusal points only at
`dispatch sync --integrate`, which is exactly what cannot run for split lineage.
Augment to name both remedies:
```
slice N → done: refused — dispatched code not integrated to trunk: {reason}
  • if trunk can still fast-forward: `dispatch sync --integrate --trunk <ref>`, verify, retry
  • if the reviewed code already landed out-of-band (manual merge / direct-land):
    `dispatch sync --slice N --record-integration --trunk <ref>`
```

**Non-goals** — IMP-127 (ingest a hand-resolved 3-way merge; SL-212, reverses
ADR-012 D2/D4, RFC-006-gated); split-lineage *prevention* (IMP-201 / IMP-174); the
broader RFC-016 machine (`dispatch next`, auto-sourcing, bundle export/ingest); any
`--force` / row-earned bypass.

## 8. Risks & Mitigations

- **R1 — un-earned row rubber-stamps the gate.** Mitigated by the earned check
  (§5.2 `plan_recorded_trunk_row`) *and* the gate's independent `is_ancestor`
  re-verification (D1). Negative test VT-2.
- **R2 — parallel implementation of the two sources.** Mitigated by
  `resolve_trunk_payload` extraction shared with both existing planners (D3).
- **R3 — a later stray integrate corrupts trunk.** Mitigated by `expected_old =
  planned` → `Refused`, never a backward advance (D2). Documented as unsupported;
  non-destructive.
- **A1 — pure/imperative split holds** (git in the shell; planners take OIDs).

## 9. Quality Engineering & Validation

Red/green/refactor; behaviour-preservation is the existing suites staying green.

- **VT-1** Record writes a Verified trunk row (`planned = reviewed_oid`);
  `trunk_integration` → `Integrated`; `slice status … done` passes. (SL-190 shape.)
- **VT-2 (negative, R1)** reviewed_oid *not* an ancestor of trunk → refuse; no row.
- **VT-3** Manual-merge lineage (trunk = merge commit, reviewed tip = ancestor) →
  recorded + `done`. (SL-147 shape.)
- **VT-4** Source resolution: candidate-active → admitted `close_target`; legacy →
  phase-chain tip; candidate-active without admission → refuse.
- **VT-5** Idempotent re-record → no duplicate row ("already recorded").
- **VT-6 (behaviour-preservation)** existing dispatch + ledger suites green
  unchanged; `resolve_trunk_payload` refactor proven by the existing
  `plan_trunk_row` / `plan_candidate_trunk_row` tests.
- **VT-7 (prescription)** Blocked `slice status` message names
  `--record-integration` (mirrors `slice.rs:6469`).
- **VH** Replay the SL-147 / SL-190 shapes end-to-end to `done`, no hand-edited
  journal, no forfeited integrity.

## 10. Review Notes

(Adversarial pass pending — §6 Adversarial Review of the design skill.)
