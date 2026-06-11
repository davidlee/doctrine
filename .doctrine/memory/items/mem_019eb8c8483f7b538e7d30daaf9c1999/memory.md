# SPEC-002 Reconciliation Engine slice roadmap (A→B)

**SPEC-002** (Requirement Reconciliation Engine, descends PRD-013) — the
observe→reconcile→close machinery of ADR-003 — decomposes into **two slices**,
B depends on A. Maps onto ADR-003 observe → reconcile → close.

## Slice A — observe substrate — `SL-042` (DONE)
Store evidence, derive coverage/drift, **never** derive authored status from coverage.
- **A·P1** REC entity kind (`REC-NNN`, `.doctrine/rec/NNN/`, status-less/immutable, 3 moves accept/revise/redesign) — D1, REQ-108.
- **A·P2** coverage substrate — slice-side mode-discriminated entries `(slice, req, contributing_change, mode ∈ VT/VA/VH, status, git_anchor, [attested])` in `.doctrine/slice/NNN/coverage.toml` (authored tier) — D3, REQ-109.
- **A·P3** composite view (`composite()` pure fold) + drift surfacer (`drift(authored, composite) → Verdict{Coherent,Divergent(reason),Indeterminate}`, read-only, no write) — D4/D6, REQ-110/111.
- **A·P4** VH/VA staleness decay on the `src/git.rs` git-anchor seam, surface-never-demote — D5, REQ-115.

## Slice B — reconcile + close — `SL-044` (IN DESIGN, resolves IMP-030)
Author reconciled truth through one writer; gate closure on coherence.
- **B·P1** write seam — `spec req status <REQ> --to <state>` edit-preserving setter (mirrors `slice status`). Both accept & revise reuse it (revise = structural status only; material spec/ADR prose → a Revision, IDE-003) — FR-005 precond.
- **B·P2** reconcile writer — sole author; per divergence applies accept (status catches up) / revise (status corrected) / redesign (escalate reconcile→design, no write); emits exactly one REC/act; NF-001 coverage→status-writer import-edge structural proof — D7, REQ-112, REQ-114, REQ-116.
- **B·P3** closure gate — predicate on `slice status reconcile→done` (extends the existing D-C9b RV-blocker gate in `slice::run_status`); default-refuse residual drift on the closing slice's coverage reqs; override = an `accept` REC (`from==to` "affirm") owned by the slice — D8, REQ-113.

## Cross-cutting (proven across both, not phases)
- **NF-001 (REQ-114)** no coverage→authored-status derivation — structural; the load-bearing import-edge guard lands in **B** (no status-writer existed in A to wall off).
- **NF-003 (REQ-116)** diffable/reconstructable from the authored tier alone — asserted at B·P2/B·P3.

## Key decisions locked (SL-044 design, in progress)
- accept reframed as **"affirm authored status against evidence"** (moves iff authored lagged; `from==to` if already correct) — dissolves the accept-overload for the override case.
- closure-gate override = `accept` REC with `from==to`, `owning_slice == closing slice`; each slice's gate discharges only its **own** drift (no stale cross-slice override).
- gate match + NF-003 reconstruction use an **on-demand REC corpus scan** (max-id, owning-slice-scoped, naming R), NOT a stored `req→rec` link — ADR-004 outbound-only, avoids denormalization desync. Perf escalation = RSK-006.

Realises REQ-108..116; descends PRD-013. Sibling record family: RV ledger (SL-040, ADR-007). Drift Ledger (IMP-022) is a separate mass-divergence kind, out of scope.
