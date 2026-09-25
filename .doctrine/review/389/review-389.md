# Review RV-389 — code-review of SL-264

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Pre-audit code review of SL-264's implementation delta (PHASE-01..PHASE-05).

**Delta** — the SL-264 commits only, not the `d128f79..HEAD` range (other slices
interleave): `37d26a5ff` (PHASE-01), `a67060fe5` (PHASE-02), `24b38cd61`
(PHASE-03), `6db9eb66b` + `b91be7168` (PHASE-04), `e37cd62ab` (PHASE-05),
`1f8c21d10` (selectors). ~2.9k+/0.7k-, 26 files; ~1.3k non-test `src/design_run/*`.

**Authority** — the locked design (`sec-1`..`sec-7`), `DEC-300`/`DEC-301`/`DEC-302`,
the `RV-386` amendments (`F-1`..`F-18`), ADR-001, STD-001, STD-003. Deviations
already disclosed in `notes.md` (PHASE-02 EX file list, PHASE-01 VT-2 weak red,
PHASE-03→04 coverage hole, `sec-5` under-declaration, PHASE-01 missing boundary
row) are checked, not re-raised.

**Passes** — one per dimension; each finding title is prefixed with its dimension:

1. `[correctness]` — staleness per `Coverage` (`InquiryMap` vs `ReviewedGraph`);
   per-node legacy fallback; the derived blocking condition still fires (R1:
   loosening invalidation is a truthfulness change); stored snapshots parse and keep
   their verdicts (R2); `Sparse<bool>` + two-home key null/omit/create; change log
   agrees with the gate.
2. `[conformance]` — code vs design sections and DECs; VT/EX criteria actually
   evidenced; layering and standards; `install/*.md` mirrors vs code.
3. `[dry]` — parallel implementations, cohesion, naming, function length, legacy
   read-only surface area.
4. `[tests]` — brittleness, theatre, fixture duplication, golden pins, red-first
   honesty of each VT.

**Invariants held** — no condition that should invalidate silently stops doing so;
no stored snapshot becomes unreadable or changes verdict except where the design
says so; `needs: null` ≡ `needs: []`.
