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
                       └─ --record-integration → record row: payload ⊑ trunk       ── NEW (SL-211)
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

**Earned-surface resolution (R2 — one seam, shared with integrate).** The recorder
sources the **model trunk payload**, identical to `integrate`'s trunk planning —
SPEC-022 is normative: `review/<N>` is a *review surface*, **never** a trunk
payload, and a candidate-active slice **requires** a `close_target` admission (no
raw-evidence fallback). So:

```rust
fn resolve_trunk_payload(
    root: &Path, slice3: &str, journal: &Journal, candidates: &Candidates,
) -> anyhow::Result<String /* oid */> {
    if candidates.rows.is_empty() {
        // legacy: the .doctrine-stripped cumulative code cut (NOT review/<N>).
        let phase_ref = phase_chain_tip(journal, slice3)
            .with_context(|| format!("no phase/{slice3}-NN code units"))?;
        resolve_commit(root, &phase_ref)?.with_context(|| format!("{phase_ref} unresolved"))
    } else {
        // candidate-active: the admitted close_target — REFUSE if none (mirrors
        // plan_candidate_trunk_row / SPEC-022; the fix for RV-263 finding 1).
        candidates.current_admission.close_target.as_ref()
            .context("record-integration: candidate workflow active but no close_target \
                      admission — run `dispatch candidate admit --role close_target` (or \
                      supersede the conflicted candidate); will not fall back to raw review")
            .map(|a| a.admitted_oid.clone())
    }
}
```
This is `integrate()`'s inline branch (`dispatch.rs:2044–2055`) extracted; the
recorder and the two advancing planners (`plan_trunk_row` /
`plan_candidate_trunk_row`) all consume it. The refusal *behaviour* (candidate-
active without admission) is preserved byte-for-behaviour; the extraction's only
observable churn is that the two planners' no-admission message text becomes the
shared string — a trivial-implementation detail, existing suites updated to match.

**The sanctioned recovery land is therefore the payload, not `review/<N>`** —
`git merge --no-ff phase/<N>-NN` (legacy) or the admitted candidate ref
(candidate-active), so the payload is genuinely an ancestor of trunk. This revises
the oracle memories' merge step (§ Doc/memory, §10 F-1).

**Planner (pure over OIDs; is_ancestor is the thin-shell git seam):**
```rust
fn plan_recorded_trunk_row(
    root: &Path, slice3: &str, journal: &Journal,
    candidates: &Candidates, trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let payload = resolve_trunk_payload(root, slice3, journal, candidates)?;
    let tip = resolve_commit(root, trunk_ref)?
        .with_context(|| format!(
            "record-integration: {trunk_ref} does not resolve — no trunk to record onto"))?;
    // EARNED CHECK (R1 negative): the payload must already be on trunk.
    // is_ancestor proves *integration occurred* (a commit in trunk's history), the
    // same standard the gate holds — not tree-survival-at-tip (SPEC-022; F-1b).
    anyhow::ensure!(
        git::is_ancestor(root, &payload, &tip)?,
        "record-integration: trunk payload {payload} is not an ancestor of {trunk_ref} \
         (at {tip}) — land it (`git merge --no-ff phase/{slice3}-NN` or the admitted \
         candidate) before recording"
    );
    Ok(recorded_row(trunk_ref, payload))
}
```

**Handler:** `run_record_integration(path, slice, trunk_ref)` → `root::find` →
resolve `dispatch/<N>` tip, tree-read `journal.toml` + candidates →
**trunk-matches-`deliver_to` guard** (F-4: the row must target the gate's ref) →
**existing-trunk-row guard** (F-2: no row ⇒ record; a *Verified* row ⇒ real prior
integration, gate already passes ⇒ idempotent no-op; a *Failed/Pending* row ⇒
**replace** it — a non-applied row has zero external effect, so overwriting it with
the earned Verified row IS the recovery, no hand-edit) → `plan_recorded_trunk_row`
→ append/replace → `commit_journal` onto `dispatch/<N>`. **No
`with_journaled_projection`** (no advance). The `Sync { .. }` variant is already
`Orchestrator`-classed (`guard.rs:303`) — the new stage inherits worker-mode
refusal, no classifier change (F-3).

### 5.3 Data, State & Ownership

**Row shape** — `recorded_row(trunk_ref, payload)`:

| field | value | why |
|---|---|---|
| `target_ref` | `trunk_ref` | the row the gate filters for |
| `source_oid` | `payload` | earned trunk payload (evidence) |
| `planned_new_oid` | `payload` | the gate re-checks `is_ancestor(this, trunk)` |
| `applied_new_oid` | `payload` | already applied — terminal, not pending |
| `expected_old_oid` | `payload` | **= planned**, *not* the trunk tip |
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
1. Operator lands the **trunk payload** out-of-band — `git merge --no-ff
   phase/<N>-NN` (legacy) or the admitted candidate ref (candidate-active) — so
   the payload becomes an ancestor of trunk. (Landing `review/<N>` is *not* the
   sanctioned move: it is a review surface, a different lineage from the payload;
   the earned check would refuse.)
2. `dispatch sync --slice N --record-integration --trunk <ref>` — earned check
   passes; Verified trunk row committed.
3. `slice status N done` — `trunk_integration` reads the row, re-verifies
   `is_ancestor(payload, trunk)` → `Integrated` → passes.

The recorder is terminal for the trunk leg; `--edge` aggregation, if wanted, stays
the separate `--integrate --edge` path (not gated by `done`).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** Recorder writes **no external ref** (only the `dispatch/<N>` journal
  commit). FF-only invariant untouched.
- **INV-2** A recorded row's `planned_new_oid` is always an ancestor of the trunk
  tip at record time (the earned check); the gate re-asserts it at `done`.
- **INV-3** `ledger.rs::trunk_integration` is **unchanged**.
- **INV-4** The recorder never records `review/<N>` as a trunk payload (SPEC-022
  invariant preserved) — only the phase-chain tip or the admitted `close_target`.
- **EDGE — trunk ref absent** → refuse ("no trunk to record onto").
- **EDGE — payload not on trunk** → refuse (R1 negative); no row written. Covers a
  cherry-pick land or a `review/<N>` merge (payload lineage absent) — the operator
  must land the payload (`merge --no-ff phase/<N>-NN` / admitted candidate) to
  earn the row.
- **EDGE — candidate-active without a `close_target` admission** → **refuse**
  (mirrors SPEC-022 / `plan_candidate_trunk_row`; RV-263 F-1). Prescription: admit
  a close_target reflecting the landed tip, or supersede the conflicted candidate.
  No raw-`review/<N>` fallback.
- **EDGE — `--trunk` ≠ `deliver_to`** (F-4) → refuse: the recorded row would
  target a ref the gate does not read, so `done` would still block. Guard resolves
  both and refuses a mismatch (or defaults `--trunk` to `deliver_to`).
- **EDGE — existing trunk row** (F-2): a *Verified* row ⇒ real prior integration,
  gate already passes ⇒ idempotent no-op; a *Failed/Pending* row (never applied,
  zero external effect — e.g. a prior refused integrate) ⇒ **replace** it with the
  earned Verified row (this is the recovery, not a dead-end refusal).
- **EDGE — trunk tip == payload tip** (clean ff already happened elsewhere) →
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

- **D1 — the recorded row carries the trunk payload (planned = payload), and the
  gate re-verifies.** Alternative (B) recorded the live trunk tip → gate check
  `is_ancestor(tip, tip)` is trivially true → rubber stamp; all integrity resting
  on one record-time assertion. Rejected per R1. (A) keeps the gate a live,
  re-checkable invariant and records *which* payload commit landed.
- **D2 — `expected_old = planned`, not the trunk tip.** Makes a stray later
  integrate a non-destructive `Refused` rather than a backward trunk advance
  (§5.3).
- **D3 — one seam: the recorder's payload resolution == integrate's** (R2). The
  inline branch in `integrate()` (`dispatch.rs:2044–2055`) is extracted to
  `resolve_trunk_payload` and shared by the recorder and the two advancing
  planners. No parallel implementation; existing planner tests prove the extraction
  (the no-admission refusal *behaviour* is preserved; only the message text
  becomes the shared string).
- **D4 — earnedness in the verb; the gate leaf and the prescription copy stay
  mechanical.** The slice-shell prescription (§ below) is an unconditional
  signpost; it does not re-derive earnedness (no is_ancestor duplicated into
  `slice.rs`).
- **D5 — `ledger.rs` unchanged.** The gate already accepts an ancestor row;
  behaviour-preservation by construction.
- **D6 — the earned surface is the model trunk payload (phase-chain tip /
  admitted `close_target`), NEVER `review/<N>`** (RV-263, supersedes the internal
  F-1). SPEC-022 is normative: `review/<N>` is a review surface, never a trunk
  payload, and candidate-active *requires* a `close_target` admission. The
  consequence — the sanctioned recovery land is `merge --no-ff phase/<N>-NN` (or
  the admitted candidate), not `review/<N>`, so the payload is a genuine ancestor.
  This keeps the recorder a thin sibling of integrate (identical source) and
  revises the oracle memories' merge step.
- **D7 — a non-applied trunk row (Pending/Failed) is *replaced*, not refused**
  (RV-263 F-2). Such a row never mutated trunk, so overwriting it with the earned
  Verified row is the recovery — it closes the stuck state a prior refused
  integrate leaves, without a hand-edit. A Verified row means real integration →
  the gate already passes → idempotent no-op.

**Prescription (IMP-169, RFC-016 §C)** — copy edit in the slice close-gate shell
(`slice.rs:1016–1020`). The current Blocked refusal points only at
`dispatch sync --integrate`, which is exactly what cannot run for split lineage.
Augment to name both remedies:
```
slice N → done: refused — dispatched code not integrated to trunk: {reason}
  • if trunk can still fast-forward: `dispatch sync --integrate --trunk <ref>`, verify, retry
  • if the trunk payload already landed out-of-band (manual merge of the payload /
    direct-land): `dispatch sync --slice N --record-integration --trunk <ref>`
```

**Non-goals** — IMP-127 (ingest a hand-resolved 3-way merge; SL-212, reverses
ADR-012 D2/D4, RFC-006-gated); split-lineage *prevention* (IMP-201 / IMP-174); the
broader RFC-016 machine (`dispatch next`, auto-sourcing, bundle export/ingest); any
`--force` / row-earned bypass.

## 8. Risks & Mitigations

- **R1 — un-earned row rubber-stamps the gate.** Mitigated by the earned check
  (§5.2 `plan_recorded_trunk_row`) *and* the gate's independent `is_ancestor`
  re-verification (D1). Negative test VT-2.
- **R2 — parallel implementation of the payload resolution.** Mitigated by the
  `resolve_trunk_payload` extraction shared by the recorder and both advancing
  planners (D3) — the recorder's source is byte-identical to integrate's.
- **R3 — a later stray integrate corrupts trunk.** Mitigated by `expected_old =
  planned` → `Refused`, never a backward advance (D2). Documented as unsupported;
  non-destructive.
- **A1 — pure/imperative split holds** (git in the shell; planners take OIDs).

## 9. Quality Engineering & Validation

Red/green/refactor; behaviour-preservation is the existing suites staying green.

- **VT-1** Record writes a Verified trunk row (`planned = payload`);
  `trunk_integration` → `Integrated`; `slice status … done` passes. (SL-190 shape,
  landing the phase-cut payload.)
- **VT-2 (negative, R1)** payload *not* an ancestor of trunk (incl. a `review/<N>`
  merge — wrong lineage) → refuse; no row.
- **VT-3** Legacy manual-merge lineage (trunk = merge commit of `phase/<N>-NN`,
  payload = ancestor) → recorded + `done`. (SL-147 shape.)
- **VT-4 (D6/RV-263 F-1)** Source resolution == integrate's: candidate-active →
  admitted `close_target`; candidate-active **without** admission → **refuse** (no
  raw-review fallback); legacy → phase-chain tip. Assert `review/<N>` is **never**
  recorded as a payload.
- **VT-5 (D7/F-2)** Re-record over a Verified row → idempotent no-op ("already
  integrated"); over a Failed/Pending row → **replace** it with the earned Verified
  row (not a refusal, not a duplicate).
- **VT-6 (F-4)** `--trunk` ≠ `deliver_to` → refuse (or defaults); a matching ref
  records a row the gate reads.
- **VT-7 (behaviour-preservation)** existing dispatch + ledger suites green
  unchanged; the `resolve_trunk_payload` extraction is proven by the existing
  `plan_trunk_row` / `plan_candidate_trunk_row` tests (refusal behaviour intact;
  message-exact assertions updated to the shared string). `Sync { .. }` stays
  `Orchestrator`-classed with the new stage present.
- **VT-8 (prescription)** Blocked `slice status` message names
  `--record-integration` (mirrors `slice.rs:6469`).
- **VH** Replay the SL-147 / SL-190 shapes end-to-end to `done`, no hand-edited
  journal, no forfeited integrity.

## 10. Review Notes

**Internal adversarial pass (design skill §6) — findings integrated:**

- **F-1 (MAJOR, SUPERSEDED by RV-263 below).** The internal pass mis-diagnosed the
  legacy surface as `review/<N>` (it correctly saw the phase-tip lineage problem
  but drew the wrong conclusion). The external pass showed `review/<N>` is *never*
  a valid trunk payload (SPEC-022) — the real fix is that the sanctioned recovery
  lands the *payload* (`phase/<N>-NN` / admitted candidate), not `review/<N>`. See
  RV-263 finding 1 resolution.
- **F-2 (MEDIUM, integrated).** The `fresh` idempotence guard would silently
  no-op over a *stale Failed/Pending* trunk row from a prior refused integrate,
  leaving the gate reading a non-ancestor row (permanent block). Fixed: the
  existing-trunk-row guard distinguishes a Verified ancestor row (no-op) from a
  stale row (refuse with guidance) (§5.4, §5.5).
- **F-3 (MOOT).** `guard.rs:303` classes `DispatchCommand::Sync { .. }` by the
  variant, so the new stage inherits `Orchestrator("dispatch-sync")` and
  worker-mode refusal automatically. No classifier change; a VT confirms it.
- **F-4 (MEDIUM, integrated).** A recorded row whose `target_ref` ≠ the gate's
  `deliver_to` would not be read at `done`. Fixed: guard/default `--trunk` to
  `deliver_to` (§5.4, §5.5, VT-6).
- **F-1b (MINOR, integrated).** Clarified the earned check is `is_ancestor` =
  "integration occurred", the gate's own SPEC-022 standard, not tree-survival at
  tip (a later revert is out of scope — same property the existing gate has).

**External adversarial pass — codex/GPT-5.5, RV-263 (both findings accepted):**

- **RV-263 finding 1 (blocker) — candidate-active raw-`review/<N>` fallback breaks
  the SPEC-022 `close_target`-admission contract.** SPEC-022:169 is normative:
  candidate-active `integrate` *requires* a `close_target` admission and never
  falls back to raw evidence; `review/<N>` is never a trunk payload. Resolution
  (user-chosen option a): the recorder's payload == integrate's — phase-chain tip
  (legacy) / admitted `close_target` (candidate, refuse if none). The sanctioned
  recovery land is the payload, not `review/<N>`. This also *reverts* the internal
  F-1 and restores the shared `resolve_trunk_payload` seam (D3, D6). §5.2/§5.4/§5.5,
  VT-4.
- **RV-263 finding 2 (major) — the stale-row guard named no clearing verb.** A
  prior refused `--integrate --trunk` can leave a Failed/Pending trunk row; my
  first F-2 fix *refused* it with "clear it" but no verb clears it → still stuck.
  Resolution: the recorder **replaces** a non-applied (Pending/Failed) row with the
  earned Verified row — the row carried zero external effect, so overwriting it is
  the recovery (D7). §5.4/§5.5, VT-5.
- Codex confirmed the parts that hold: `expected_old = planned` replay-safety
  against `advance_row`, and "gate unchanged" against `trunk_integration`
  (ancestor-not-tip passes; ambiguous / empty-planned fail closed).

**Resolved in implementation (minor, non-blocking; RV-274 F-2):** both questions
the design left open are settled by the shipped verb. (1) `--trunk` ≠ `deliver_to`
**hard-refuses** — `run_record_integration` `bail!`s naming both refs; an absent
`--trunk` defaults to `deliver_to` (VT-6 /
`run_record_integration_refuses_trunk_deliver_to_mismatch` +
`vt2_close_integration_honours_deliver_to_override`). (2) the success line **emits
the payload OID** (`record-integration: recorded Verified trunk row … (payload …)`).
Retroactive note (still open): already-stranded SL-147/SL-190 that historically
merged `review/<N>` may need a payload re-land to record (or stay one-offs) — the
verb is forward-correct for the general shape.
