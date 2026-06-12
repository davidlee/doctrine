# Review RV-005 — reconciliation of SL-045

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation of SL-045 (two read surfaces: `doctrine
coverage <ref>` + `doctrine spec req list <SPEC>`) against the LOCKED `design.md`
(§5.5 invariants, §10 review charges) and governance (ADR-001 layering, F4/SL-025
uniform list contract). Lines of attack — the design's load-bearing invariants:

- **INV-1 / F1 wall** — rendered `status` cell sourced ONLY from
  `requirement::load`, never the coverage fold (no data edge fold→authored).
- **E2 canonicalize** — `member_reqs` runs `canonicalize_fk` on every member FK
  before batching, else `scan_coverage` (exact-string) silently reads `none`.
- **E1 / INV-4 dangling** — `CoverageRow` enum{Healthy|Dangling}; dangling shape
  carries no fabricated authored cells; spec fan AND roster degrade-and-continue;
  bare single-REQ load failure fatal.
- **INV-2 one walk** — spec fan = exactly one `scan_coverage_batch`.
- **INV-3 authored-only roster** — `spec req list`: no scan, no coverage import,
  no observed/verdict column.
- **E6** — `observed_state` an independent total partition over 5 `CoverageStatus`,
  not 1:1 onto `drift`.
- **D5** — `Verdict::label` the single cell-text source; reconcile `build_prompt`
  not merged. **F6** — coverage/coverage_scan/reconcile suites green unchanged.

Evidence: read the four-commit delta seams directly (not just test green), ran the
full bin suite (977 baseline) + all integration suites + `cargo clippy`, plus an
external adversarial pass (codex gpt-5.5).

## Synthesis

**Verdict: audit-ready, closing.** One major finding raised and reconciled
fix-now (F-1); all other invariants verified holding at the seam.

**Findings.**
- **F-1 (major → fix-now → verified).** `spec req list` skipped the
  `validate_statuses` known-set guard every sibling list surface performs, so
  `--status bogus` silently emptied the roster instead of erroring — a F4/SL-025
  uniform-contract breach. Surfaced by the external pass (codex E#1), confirmed at
  the seam. Reconciled inside the slice (commit 03a0d7a): `REQ_STATUSES` const +
  drift canary in `requirement.rs`, `validate_statuses` call in `req_list_rows`
  mirroring `list_rows:1170`, red/green test. `just check` green.

**Invariants verified holding (read at the seam, not inferred from green tests).**
- INV-1 wall — `coverage_view::rows` reads `status` from `requirement::load` and
  ONLY there; the fold feeds observed/verdict only. Pinned at the COMMAND seam by
  `e2e_coverage_view_golden::req_status_cell_is_fixed_while_verdict_moves`.
- E2 — `spec::member_reqs` canonicalizes every FK (spec.rs:557) before batching.
- E1/INV-4 — `CoverageRow` is the enum; dangling JSON forbidden-keys asserted
  absent (`status`/`observed`/`verdict`/`kind`); fan + roster both degrade-and-
  continue; bare single-REQ fatal. Both `coverage` and `spec req list` symmetric.
- INV-2 — `rows` issues exactly one `scan_coverage_batch` over the whole fan.
- INV-3 — no `coverage`/`coverage_scan` import in `spec.rs`; roster never scans.
- E6 — `observed_state` is the four-predicate partition; a test asserts it does
  NOT track `drift` 1:1.
- ADR-001 — `coverage_view`→`spec` only; `spec` imports no `coverage_view` (no
  back-edge). D5 — `Verdict::label` single source; `build_prompt` untouched.
- F6 — 977 baseline bin + all integration suites green UNCHANGED; the SL-044
  NF-001 residency proof (`e2e_coverage_authored_residency`) still green.

**Rejected charge (recorded, not raised).** Codex E#2 alleged the module-level
`#![cfg_attr(not(test), expect(dead_code,…))]` at `coverage_view.rs:39` would be
an unfulfilled lint expectation now that `main.rs` wires `coverage_view::run`,
breaking the zero-warning gate. Empirically FALSE: `cargo clippy` recompiles the
crate clean with zero warnings — the expectation is still fulfilled (a residual
not-yet-wired item keeps `dead_code` firing under the bins/lib build). Static
reasoning without building; rejected, no finding.

**Standing risk / dogfood signal (not a code defect — captured as follow-up).**
SL-045's whole purpose was to make REQ↔coverage drift *visible*; with the read
surface now shipped, every SPEC-002 requirement (REQ-108..116) still reads
authored `pending` while the engine + its read surface ship — the drift is now
observable via `doctrine coverage SPEC-002` but unreconciled. That is corpus
reconciliation work, out of SL-045's scope (a surfacing slice). Captured as
backlog, not a finding against this slice's deliverable.

**Relaxed golden — consciously accepted.** Two dangling-row TABLE asserts in
`e2e_coverage_view_golden` were loosened from byte-exact to whitespace-split cell
checks because the inline `load_error` embeds an absolute tempdir path that floats
per run and widens the column. Healthy-row cells stay byte-exact (the wall proof);
the dangling JSON forbidden-keys contract is asserted exactly. Acceptable.
