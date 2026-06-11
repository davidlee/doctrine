# Reconcile writer + closure gate (SPEC-002 B)

## Context

**SPEC-002** (Requirement Reconciliation Engine, draft, descends PRD-013) is the
observe→reconcile→close machinery of ADR-003. **SL-042** built the **observe** half
(the two-tier coverage substrate, REC kind, derived composite + drift reads,
staleness decay) and deliberately shipped **no write path to authored truth** — that
is the NF-001 line. This slice builds the **reconcile + close** half: author
reconciled requirement/spec truth through one writer, and gate closure on coherence.

The hard line stays: observed evidence and authored truth never touch through a
function (`REQ-105`/NF-001). Slice B is where authored truth is finally *written* —
so it is the slice that must prove the no-derivation invariant structurally at the
seam where coverage is read and status is written (the reconcile writer authors an
explicit value; it never computes status from coverage). That structural proof —
the coverage→status-writer import-edge enforcement — is the load-bearing test
(IMP-030).

Foundations already ship: SL-042's coverage store + drift surfacer (the reads this
writer consumes), the ADR-009 slice FSM (`slice status`, the F12 closure seam, and
the existing D-C9b close-gate that already refuses `→reconcile`/`→done` while an RV
carries an unresolved `blocker`), the edit-preserving authored-TOML status
transition pattern (`mem.pattern.entity.edit-preserving-status-transition`), and the
SL-040 close-gate-as-corpus-scan topology (no reverse index).

**Governing decisions** (no re-deciding here; tensions go back through `/consult`):
SPEC-002 (**D7** sole writer, **D8** closure gate, **D9** CLI shapes deferred),
ADR-003 (explicit-authorship-not-derivation), ADR-009 (FSM, F12 closure seam, the
`reconcile → design` escalation). Realises requirements **REQ-112** (FR-005, sole
writer) and **REQ-113** (FR-006, closure gate). Strengthens the cross-cutting
**REQ-114** (NF-001, no derivation — proven structurally at the write seam) and
**REQ-116** (NF-003, diffable/reconstructable from the authored tier). Descends
product **PRD-013**.

**Relations** (prose — no structural slice-relation surface in v1; IMP-016):
- *Realises* — SPEC-002 (its reconcile + close half): REQ-112, REQ-113;
  cross-cutting REQ-114, REQ-116.
- *Governed by* — ADR-003, ADR-009.
- *Descends product* — PRD-013.
- *Depends on (hard)* — **SL-042**. It owns the coverage store shape, the REC kind,
  the derived composite + drift reads this writer consumes, and the
  status-less/immutable REC contract (SL-042 D-Q3). Slice B writes **no** new
  observed-tier machinery; it reads SL-042's reads and writes the authored tier.
- *Sibling / shared seam* — **SL-040** (RV review-ledger). B·P3's closure gate
  reuses SL-040 #3/#6 — the outbound-edge + reverse close-gate-as-corpus-scan
  pattern (no reverse index) — and *extends the same `slice status` close-gate seam*
  the D-C9b RV-blocker check already hooks. Reuse, do not re-implement.
- *Resolves backlog* — **IMP-030**.
- *Related backlog* — **RSK-006** (coverage scan/staleness perf — the reconcile
  writer is the at-scale coverage reader RSK-006 flags; revisit the
  reverse-index/batching trigger here). **IMP-008** (the `/reconcile` skill + the
  audit/reconcile seam that *drives* this writer — skill wiring is downstream, but
  the writer's CLI shape is what it will call). **IMP-006** (uniform
  lifecycle-transition verbs across kinds — B·P1's `spec req status` must ride the
  `slice status` transition shape so it folds into that uniformity, not a parallel
  one). **IMP-023** (rewire the RV `reconciliation` *facet* — an adversarial
  *review of* a reconciliation — onto RV; **distinct** from this slice's reconcile
  *writer*, which performs the act. Design must not conflate the two).
- *Adjacent surface* — **ISS-004** (`spec req add` aborts on an overlong
  title-derived slug, no `--slug` escape). B·P1 extends the same `spec req` command
  tree; it transitions existing requirements by id (no slug derivation) so it should
  dodge the defect, but must not reintroduce the unescaped-slug footgun.

## Scope & Objectives

Build the reconcile + close half as three phases (B·P1–B·P3), ending green with
exactly one REC per reconciliation act and closure refusing unreconciled drift.

- **B·P1 — Write seam** (FR-005 precondition; R1, pre-decided in SL-042). The
  authored-truth write primitive the reconcile writer needs: **one**
  `spec req status <REQ> --to <state>` edit-preserving transition — a **free any→any**
  setter mirroring `governance::set_status` (the adr precedent), *not* the ordered
  `slice status` FSM (D-B6: `revise` must move any direction to correct a mis-claim).
  Both accept and revise reuse this single setter (D-B4 — the earlier "spec-truth
  revise write path" collapses into it; revise differs only by REC move + direction +
  the human's prose hand-edit, which routes to the IDE-003 Revision vehicle).
  Edit-preserving TOML rewrite; clock/disk in the thin shell. No reconcile logic yet.
- **B·P2 — Reconcile writer** (D7, FR-005/REQ-112, NF-001/REQ-114, NF-003/REQ-116).
  The **sole author** of reconciled truth in the loop. Per divergence it applies one
  move: **accept** (write requirement status via the B·P1 seam to match evidence),
  **revise** (write corrected spec truth), or **redesign** (escalate
  `reconcile → design` via `slice status`, ADR-009 — **no** instance write). Emits
  **exactly one REC per requirement** (D-B8 — forced by the single `move` field;
  composed atomically). The writer authors status values explicitly; NF-001 holds
  **type-level** — the writer *does* import `coverage` (to read `drift` for
  prompting), but `coverage` exposes no `ReqStatus`, so a derived status cannot
  compile (D-B7; a compile-fail VT proves it, IMP-030). Every act is committed +
  REC-cited so it is reconstructable from the authored tier alone (NF-003).
- **B·P3 — Closure gate predicate** (D8, FR-006/REQ-113, H5). A predicate on the
  existing `slice status reconcile → done` edge: default-**refuse** while owning
  specs carry residual unreconciled drift (corpus scan over SL-042's drift read,
  SL-040 #3/#6 pattern — no reverse index). The only admitted override is a
  **recorded reconciliation act** — a REC recording accepted residual drift with
  rationale — so closed-with-*unreconciled*-drift is unrepresentable. The
  `done`-only-from-`reconcile` topology edge stays ADR-009 F12 hard, independent of
  the drift check, and composes with (does not replace) the existing D-C9b
  RV-blocker close-gate on the same seam.

**Closure intent.** Reconciliation produces **exactly one REC per act**, diffable
and reconstructable from the authored tier alone (NF-003 — no recourse to chat or
runtime state). Closure **refuses** unreconciled drift unless an override REC is
present. The NF-001 acceptance proof is **structural** at the write seam: no
function maps coverage → authored status; the import edge is absent by test.

## Non-Goals

- **The observe substrate** (coverage store, REC kind, composite/drift reads,
  staleness) — SL-042, depended on, not rebuilt. Slice B adds **no** observed-tier
  machinery.
- **A coverage→status derivation** — forbidden by NF-001. The writer authors
  explicit values; building any path that computes status from coverage violates the
  load-bearing line and is the thing B·P2's structural test exists to forbid.
- **The `/reconcile` skill + audit/reconcile seam disentanglement** — IMP-008,
  downstream. This slice ships the CLI write/gate surface the skill will *drive*, not
  the skill.
- **PRD-010 `knowledge_record`** — forward dep; REC's evidence sub-structure stays
  inline (SL-042 carry), lifted when knowledge_record lands (OQ-2/H4).
- **Composite precedence rules** (OQ-3) — v1 surfaces all; the writer judges. No
  precedence engine here.
- **A staged draft/approve Revision vehicle** (IDE-003) — B·P2's `revise` move is a
  **direct** edit-preserving spec-truth write. Staged delta drafting/approval of
  requirement+spec-prose deltas (distinct from REC) is IDE-003, deferred by SL-042
  D-Q3. Not built here.
- **Drift Ledger** (multi-artefact mass reconciliation, IMP-022 — carved from
  ADR-007 D-C11) and the **RV review-ledger** itself (SL-040) — sibling families,
  out of scope. B's drift is the per-requirement derived read from SL-042, not a
  mass-divergence ledger.
- **Coverage scan/staleness perf hardening** (RSK-006) — revisited here as a
  decision input, but the conditioned reverse-index/batching is its own follow-up,
  not built blind in v1.

## Summary

Builds SPEC-002's reconcile + close half: the authored-truth write seam (one
`spec req status` setter, reused by accept & revise), the **sole-author** reconcile
writer (accept/revise/redesign → exactly one REC per requirement), and the closure
gate
(default-refuse residual drift, override only via a recorded REC). The NF-001
no-derivation line — built into SL-042 by *absence* of a write path — is proven here
by a structural import-edge test at the one seam that reads coverage and writes
status. Depends on SL-042. Resolves IMP-030.

## Follow-Ups

- **IMP-008** — the `/reconcile` skill + audit/reconcile seam, driving this writer.
- **RSK-006** — coverage scan/staleness perf: decide the reverse-index/batching
  trigger against the reconcile writer's real read pattern; the writer is the
  at-scale reader that conditions it.
- **OQ-3** — composite precedence: still deferred; the writer judges `Indeterminate`
  entries as drift in v1.
- **OQ-2 / H4** — REC evidence sub-structure lifts to the shared PRD-010 type when
  `knowledge_record` lands; neither forks.
