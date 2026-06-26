# Review RV-165 — reconciliation of SL-138

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-138 — relation-transitive walk for `inspect`. 3 phases, all
landed on edge as single non-merge feat commits (P1 `f4104f55`, P2 `42de85c4`,
P3 `1ed4c750`); merge `3f9a368b`. Reviewed surface: the landed `edge` tree
(solo funnel, not dispatched — `review/*`/`phase/*` immutability caveat N/A).

**Invariants this audit holds the slice to:**
1. **Behaviour-preservation (the #1 design risk).** `reachable` re-expressed
   over `reachable_bounded(.., None)` must be byte-identical; bare `inspect <ID>`
   output byte-unchanged (existing `e2e_inspect_golden` 16/16, cordage + blockers
   suites green UNCHANGED). Design §6 gate.
2. **Layering (ADR-001).** Traversal in cordage leaf; engine `TransitiveDir`
   defined in `relation_graph`, never `cli.rs`; transitive branch relation-only —
   no `priority`/actionability up-call.
3. **Output contract (C4).** `kind=inspect-transitive`, inbound-before-outbound,
   view-level `truncated`, `max_depth: null` unbounded, non-requested direction
   key OMITTED — golden-pinned.
4. **Table-derived overlay predicate (C2).** no-overlay set
   `{contextualizes, drift, decision_ref}` rejected from `--labels` + absent from
   default; predicate via `OverlayMap`, never a hardcoded list.
5. **Direction polarity (design §5 L188).** `up`=Inbound=blast-radius,
   `down`=Outbound=derivation — sheet's guess was inverted; verify the landed
   code + goldens match the design, not the sheet.
6. **Memory-ref gate (F2).** `mem_*`/`mem.key` + `--transitive` rejected BEFORE
   the memory early-return, pointing at `retrieve --expand`.
7. **EX/VT evidence.** Every EX-1..4 + VT per phase evidenced against landed code.

**Lines of attack / where bodies are buried:**
- **Conformance registry accuracy** — the boundaries ledger start-oids: are the
  recorded ranges the actual per-phase deltas, or do they sweep foreign commits?
  (P3 was recorded manually per the handoff — verify it is exactly one feat
  commit; scrutinise P1/P2 the same way.)
- **Dead-code expects** — all 11 PHASE-02 `expect(dead_code)` on the transitive
  subgraph must be retired by P3 (else `unfulfilled_lint_expectations` under
  not(test)); confirm none linger.
- **`_root` unused param** — design §5 / plan EX-1 list `root: &Path` but the
  relation-only walk reads nothing per-entity from disk; confirm it is signature
  symmetry, not a dropped existence-gate path.
- **Scope discipline** — undeclared conformance paths: scope creep, or
  registry-range pollution from interleaved edge history?

## Synthesis

**Closure story.** SL-138 lands its design cleanly. The transitive-walk surface
matches the locked design contract end-to-end: behaviour-preservation held
byte-for-byte (bare `inspect` 16/16 golden unchanged; `reachable` re-expressed
over `reachable_bounded(.., None)` with the cordage + blockers suites green
unchanged), the C4 output contract is golden-pinned (`kind=inspect-transitive`,
inbound-before-outbound, view-level `truncated`, `max_depth: null` unbounded,
non-requested direction omitted), the C2 overlay predicate is table-derived, the
direction polarity matches design §5 L188 (`up`=Inbound, the sheet's inverted
guess corrected and golden-pinned), the F2 memory-ref gate sits before the memory
early-return, layering holds (engine `TransitiveDir` never in `cli.rs`; the
transitive branch is relation-only with zero `priority::` calls), and all 11
PHASE-02 `expect(dead_code)` markers were retired by PHASE-03. Every EX-1..4 and
VT per phase is evidenced against the landed code; `just check` green. Five
findings, no blocker.

**The one substantive finding (F-1, major, fixed in-audit).** The conformance
registry was materially wrong: the solo binding stamped each phase's
`code_start_oid` at its in_progress flip, but P1 and P2 landed rebased onto an
advanced `edge` tip, so the recorded ranges swept ~30 foreign commits (SL-154,
SL-156, IMP-174, RFC-005) into the phase deltas — 49 undeclared paths burying the
true scope. The true deltas are exactly one non-merge feat commit each; I
re-recorded P1/P2 with `slice record-delta`, dropping undeclared to 4 (all
legitimate: 2 doctrine authored-state files + 2 test files, outside the §9
source-selector granularity). P3 was already clean (manually re-recorded during
execute). The root cause (F-2) is a pre-existing SL-147 mechanism limitation, not
SL-138 code — captured as IMP-175.

**Standing risks / tradeoffs consciously accepted.**
- `depths` is returned by `reachable_bounded` but unconsumed by display today
  (design D6) — a deliberate seam for a future path/tree view, not dead weight.
- Transitive `references` collapses roles into one section (F3) — accepted; a
  per-role transitive walk would need payload-aware traversal, out of scope.
- `--max-depth 0 == unbounded` is a documented footgun (F6, design); `all` is the
  primary spelling.
- `transitive_from(_root)` is an unused param kept for call-site symmetry with
  the locked signature — verified benign (existence gate works without it).
- The conformance auditor was the last line of defence against the registry
  pollution; IMP-175 exists to move that correctness upstream into the automatic
  capture so a future audit isn't relied on to catch it.

## Reconciliation Brief

No governance/spec (REV) changes and no per-slice artefact edits are required —
the design is accurate to the implementation and no finding contradicts canon.

### Per-slice (direct edit)
- None. design.md, plan.toml, slice-138.toml are all coherent with the landed
  code. (Notes harvest is complete; see `notes.md`.)

### Governance/spec (REV)
- None.

### Already actioned in-audit (recorded for the reconciler's awareness)
- **F-1** — conformance registry corrected via `slice record-delta` (P1
  `5cb84f3a..f4104f55`, P2 `83b9cea2..42de85c4`). Runtime-state correction only;
  nothing to re-write downstream. Verify conformance still reads 5/0/4 at close.
- **F-2** — root cause captured as **IMP-175** (no SL-138 action).

`/reconcile` is effectively a pass-through to `/close` here: consume this brief,
confirm there is no write surface to action, and proceed.

## Reconciliation Outcome

Pass-through. The reconciliation brief maps to no write surface.

### Direct edits applied
- None. design.md, plan.toml, slice-138.toml coherent with landed code.

### REVs completed
- None. No finding contradicts canon.

### Already actioned in-audit (recorded for traceability)
- RV-165 F-1: conformance registry corrected via `slice record-delta` (P1
  `5cb84f3a..f4104f55`, P2 `83b9cea2..42de85c4`) — runtime state only.
- RV-165 F-2: root cause captured as IMP-175 — no SL-138 action.

All 5 findings terminal (`verified`); RV-165 `done · await=none`. No REV, no
per-slice edit, no half-applied change blocks close. Reconcile complete — handoff
to /close.
