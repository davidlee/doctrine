# Review RV-266 — reconciliation of SL-213

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed (F-2 record):** candidate interaction branch
`candidate/213/review-001` (cand-213-review-001, tip 935cb4cf), the no-ff merge
of impl-bundle `review/213` (8563566c) onto base `refs/heads/main` (ba5cf0be —
the dispatch base). Evidence refs `review/213` + `phase/213-02..06` are
immutable (R2); audit runs against the candidate, per /audit dispatched-slice
rule. All six phases were dispatch-driven (claude arm, worker self-commit).

Lines of attack:

1. **Mechanical conformance** — `slice conformance 213` cells: two undeclared
   leads (`src/priority/config.rs`, `.doctrine/slice/213/slice-213.toml`),
   zero undelivered, 14 conformant. Each undeclared dispositioned.
2. **Design fidelity** — implementation vs design.md normative rules R1–R6,
   C1–C8, P1–P15, S1–S4; carried assumptions resolved in-drive (local
   Kosaraju vs cordage reuse; R6 vacuous; GAUGE_STEP homing) confirmed
   against ADR-001 layering and the plan's EX pins.
3. **Known tensions harvested by workers** (notes.md): P8 gauge-scope
   (prototype global-H vs design per-component wording); 10th `malformed`
   display token vs design S2's nine; `Resolution.unknown_supersedes`
   un-plumbed; PHASE-06 fixture correction on
   `compare_list_orders_by_total_key`.
4. **Verification evidence** — verify-vt all PASS post-prepare-review
   (PHASE-01..06, every VT); full suite 3306→3390 green each phase;
   regression funnel diffs clean; `doctrine check gate` re-run on the
   candidate worktree.
5. **VH-1** — human eyeball of list/explain/findings output on a real
   mini-ledger: agent pre-screen recorded here, human pass pending with the
   worker-noted candidates (tombstoned/[withdrawn] redundancy,
   AnchorConflict phrasing, P7-vs-P8 gauge line).

## Synthesis

SL-213 shipped the comparison constraint layer whole: resolution (R1–R6),
compilation with degradation (C1–C8), projection with gauge (P1–P15), priority
wiring under the authored > projected > gauge > default chain (D11), and the
read surfaces (list status column, --active-only, explain value-source shapes,
four findings with exit hints, determinism e2e). Six phases, all
dispatch-driven; suite 3306 → 3390, green at every boundary; per-phase
regression funnel diffs clean; `doctrine check gate` re-verified green on the
candidate worktree (cand-213-review-001); `slice verify-vt 213` all-PASS
across every VT of all six phases post-prepare-review.

**Closure story.** The mechanical conformance delta reduced to a single
understood cell: `src/priority/config.rs` was a registry gap (the drive-time
selector fix landed on the coordination branch's copy, not the primary's) —
fixed in-audit by the registry verb (F-1); the remaining undeclared
`slice-213.toml` cell is the selector fix itself riding PHASE-05's recorded
range, tolerated with rationale (F-2). Design fidelity checks confirmed the
three carried assumptions resolved in-drive: local Kosaraju over private
cordage machinery (deliberate-duplication note in place, cordage API
untouched), R6 "decomposed" vacuous (F-6, confirmed and closed), GAUGE_STEP
homed in priority/config.rs at 0.25 with a config override. An agent VH-1
pre-screen on a synthesized mini-ledger exercised all four findings, the
explain shapes, and --json parity — everything renders per design §4.

**Standing risks.** (1) The P8 gauge-scope tension (F-3) is the one genuine
normative conflict: design §3 prose reads per-component, the implementation
follows the prototype's global-H semantics pinned by plan EX-1. It is
transparently pinned (s2_partial_order_gauge golden + module-header note) and
diverges only with ≥2 disjoint anchor-free components; adjudication is
delegated to reconcile with a recommendation to bless the implemented
semantics. (2) `Resolution.unknown_supersedes` has no surface consumer —
disclosed nowhere; captured as IDE-036, low risk (load-time warning data,
design mandates warnings-not-errors and that holds). (3) The human VH-1 pass
is still pending; all known candidates are cosmetic and carried by CHR-041.

**Tradeoffs consciously accepted.** Worker e2e tests hand-author wire-shape
session TOML instead of shelling the write verbs (jail confinement — the A5
STOP condition's anticipated adaptation, documented in the test module doc).
The PHASE-06 fixture correction (F-5) re-stamped dates to match D13's
resolution-keyed ordering — an honest fixture, not a masked regression.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §3 (P8, P12) — gauge scope wording** [F-3]: adjudicate global-H
  vs per-component. Recommended: bless the implemented prototype semantics —
  reword P8 to "gauge spread is computed over the global height H; the
  anchored branch is taken when any anchor exists in the corpus", scope P12's
  locality claim to the anchored regime, and note per-component as a
  considered-and-rejected variant (rejected because plan EX-1 pins the
  validated prototype verbatim). If adjudication instead mandates
  per-component semantics, that is new sliced work (code + goldens), not a
  reconcile edit — stop and re-scope.
- **design.md §4 S2 + §1 enum inventory — tenth status token** [F-4]: add
  `malformed` (ResolutionStatus::Malformed, R2 supersession-cycle
  participants) to the S2 token enumeration and the §1
  ResolutionStatus/RowState inventory. Implementation is correct; prose
  predates the variant.

### Governance/spec (REV)

- None. No ADR, policy, standard, or spec drift surfaced — ADR-001 layering
  held (StatusMap/AnchorMap builders homed in priority/graph.rs above the
  comparison tier), ADR-015/REV-022 value-provenance semantics implemented as
  amended.

### Registry (already applied in-audit)

- `slice selector add 213 src/priority/config.rs --intent design-target`
  [F-1] — applied to the primary registry during this audit; recorded here so
  reconcile can confirm the conformance re-run stays at
  undeclared=1 (the tolerated F-2 cell) / undelivered=0.
