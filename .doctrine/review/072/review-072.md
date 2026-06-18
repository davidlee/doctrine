# Review RV-072 — reconciliation of SL-094

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-094 (semantic graph zoom, pan, crop-on-bounds).
All three phases implemented; inquisition (RV-071) resolved with 7 findings
dispositioned — 5 integrated, 1 tolerated, 1 aligned.

### Lines of attack

1. **Conformance — design decisions D1–D5.** Does the implementation faithfully
   realise each decision? CSS transform wrapper (D1), app-state viewport (D2),
   minK fit-to-container (D3), focus-change centres (D4), same-focus restores
   (D5).

2. **Conformance — RV-071 findings F1–F7.** All 7 inquisition findings were
   dispositioned pre-implementation. F-1 (deltaMode), F-2 ({once:true}), F-3
   (cross-mode guard), F-4 (depth-change edge case), F-5 (touch-action:none)
   were fix-now/design-wrong — confirm each is integrated. F-6 (dimLegend)
   tolerated — confirm no change needed. F-7 (seq guard) aligned — confirm.

3. **ISS-021 requirements.** Scroll-wheel zoom, drag-to-pan, crop-on-bounds —
   all three are the slice's declared scope.

4. **Evidence gates.** `npx tsc --noEmit`, `npx eslint --max-warnings=0`,
   `npx vitest run` (312 tests expected), `npx vite build`.

5. **Phase sheet harvest.** Disposable findings from phase-01/02/03.md that
   should be promoted to durable notes, memories, or backlog items.

### Held to

- design.md (authoritative — viewport rules table, design decisions,
  container mechanics, event handling)
- plan.toml PHASE-01/02/03 criteria (EN-/EX-/VT-/VA-)
- ISS-021 scope
- RV-071 findings F1–F7
- Domain map invariants I1–I6, risks R1–R3

### Audit mode

Conformance. Surface reviewed: `main` branch head (1db6ecbf), which carries
all 3 phases + the wheel-minK fix. No dispatch — sole-agent implementation.
