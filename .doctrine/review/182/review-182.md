# Review RV-182 — code-review of SL-167

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-167 extends the IMP-189 quick-fix pattern (slice `parse_ref` + `parse_cli_id`
wired to `value_parser`) to the four remaining governance kinds that take raw
`id: u32` CLI args: ADR, policy, standard, RFC.

Lines of attack:

1. **Correctness** — does each kind accept both `PREFIX-NNN` and bare `NNN`?
   Does zero-padding work? Are error messages helpful?
2. **DRY / cohesion** — four per-kind `parse_ref` functions (D1 choice). Copy-paste
   errors? Mechanical divergence? Maintenance burden?
3. **Tests** — VT-1 coverage. Are assertions sufficient? Edge cases?
4. **Doc integrity** — doc comments match their functions? Paste artifacts?
5. **Design fidelity** — does the implementation match SL-167 design D1/D2?

## Synthesis

### Overall: **acceptable**

The implementation is mechanically correct and complete. Four `parse_ref` +
`parse_cli_id` pairs are wired to their respective `Status` variants via
`#[arg(value_parser = parse_cli_id)]`, tests cover the expected input spectrum
(prefixed uppercase, prefixed lowercase, bare, zero-padded, invalid), and the
existing test suite passes unmodified. The design's two decisions — per-kind
functions (D1) and no change to `integrity::parse_canonical_ref` (D2) — are
faithfully followed. Clippy is silent.

### Synopsis

SL-167 is a small, well-scoped slice that does exactly what it says on the tin.
The code works. The audit uncovered three findings, all resolved:

- **F-1 (major → fixed):** `adr::parse_ref`'s doc-comment carried a copy-paste
  artifact from `run_status` — first line claimed it was the status verb. Fixed
  by removing the stray line. The other three modules' doc comments were clean.
  This is the only functional-documentation defect.
- **F-2 (minor → tolerated):** The four per-kind implementations are mechanically
  identical. Design D1 chose this explicitly — each kind "controls its own error
  message formatting" — and the tradeoff (28 lines of near-duplicate code in
  exchange for per-kind ownership of error strings) was consciously accepted.
  The cost is future maintenance: any logic change must be applied across five
  modules (slice + four governance kinds).
- **F-3 (nit → fixed):** Invalid-input test assertions were shallow — `is_err()`
  proved rejection but didn't verify the error message named the expected kind.
  Strengthened to `unwrap_err().to_string().contains("ADR")` (and analogously
  for POL/STD/RFC), catching potential prefix-string copy-paste mistakes at
  test time.

### Standing risks

- **Low:** A future cross-kind refactor that touches `parse_ref` logic must
  remember to replicate across all five modules. The strengthened error-message
  tests (F-3 fix) provide a partial safety net — if a kind's prefix string is
  accidentally swapped, the test will catch it.

### Haiku

Canonical ids  
now accepted everywhere —  
copy-paste doc fixed.
