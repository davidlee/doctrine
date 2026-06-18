# Review RV-076 — plan of SL-095

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Correction (F-3):** The plan DOES exist at `.doctrine/slice/095/plan.toml` + `.doctrine/slice/095/plan.md`. F-1 and F-2 were raised in error before discovering the artefacts.

Lines of interrogation:
1. **SL-097 gate** — `src/supersede.rs` is MISSING from the working tree. PHASE-03 assumes it exists but EN-1 doesn't gate on it. The plan's SL-097 coexistence section is intent-level — implementers need a concrete decision.
2. **Read/write disconnect** — PHASE-02 switches readers to `[[relation]]` but the verb still writes to the typed array until PHASE-03. What happens to a supersede operation in the gap?
3. **StorageTarget timing** — the enum doesn't exist yet. Which phase introduces it?

## Synthesis

**Verdict: the plan exists (missed on first pass) and is well-structured, but has two gaps at its critical seam — the SL-097 gate and the PHASE-02↔03 read/write disconnect.**

### Correction

- **F-1, F-2 (withdrawn in error):** The plan is at `.doctrine/slice/095/plan.toml` + `.doctrine/slice/095/plan.md`. Both findings were raised before discovering the artefacts. F-3 corrects the record.

### Real findings

- **F-4 (major, PHASE-03 EN-1 insufficient):** `src/supersede.rs` is MISSING from the working tree. PHASE-03's objective says "extends supersede_policy (in src/supersede.rs — SL-097's extracted home)" but EN-1 only gates on PHASE-02. Either EN-1 needs `.after SL-097`, or the plan must specify the interim code location (adr.rs) with a note to extract after SL-097 lands.

- **F-5 (major, read/write disconnect window):** PHASE-02 switches readers to `[[relation]]` but the verb's write path still targets the typed `[relationships].supersedes` array until PHASE-03. A supersede operation between the two phases writes to a field nothing reads. Either merge PHASE-02+03 into one phase, or document the gap invariant explicitly.

### What the plan gets right

- Three-phase breakdown is logical: PHASE-01 (low-risk `related` row), PHASE-02 (structural migration), PHASE-03 (verb extension)
- Risk decreases across phases — PHASE-01 is the smallest blast radius
- SL-097 coexistence intent is articulated (both merge orders)
- All exit/verification criteria are testable
- Behaviour-preservation gate (VT-7: "existing ADR supersede tests green") is explicit

### Gate

F-4 and F-5 are must-fix before PHASE-02 or PHASE-03 can execute. The plan is otherwise approved.
