# Review RV-185 — reconciliation of SL-168

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface (dispatched slice, ADR-007 R2).** Audit ran against the
candidate review surface `cand-168-review-001` (branch `candidate/168/review-001`,
base `refs/heads/main` @ 7990a6df, source `review/168`), materialised as a
worktree. The `review/*` and `phase/*` refs are immutable evidence; findings and
repair target the candidate.

**Provisioning note.** The fresh candidate worktree did not carry the gitignored
derived embed root `web/map/dist/` (RustEmbed `#[folder]` in
src/map_server/assets.rs); absent it the whole binary + test bin fail to compile
(`Assets::get` E0599). Provisioned manually (`cp -r web/map/dist`) before build —
see RFC-011 case-notes.

**Lines of attack.** (1) Behaviour vs locked design (RV-183): the 8-check model,
exit semantics (D4), ProseCite precision (D11/R8), the D12 native re-point safety
net. (2) Conformance algebra — undeclared/undelivered selectors. (3) ADR-001
layering (the `finding` leaf, the orchestrator-as-composer invariant). (4) The
test suite as the behaviour-preservation proof (does it actually pass, and does it
prove what it claims).

## Synthesis

SL-168 ships a real, working `doctrine doctor`: the 8-check model is present, the
orchestrator (`commands/doctor.rs`) is a clean command-layer composer, the
`finding` leaf is a faithful pure contract (STD-001 constants, single-source
`Category::severity()`), exit semantics are correct (one genuine RelationIntegrity
error — ADR-004/ADR-012 supersession drift — correctly drives non-zero; warnings
alone exit 0), and the `e2e_doctor_golden` suite (clean-exit-0, nonzero-on-error,
superset-of-validate, json-array) is green. The four adversarial design passes
(RV-183) paid off: the load-bearing precision logic (maximal-token scanner,
3-part skip, `kind_by_prefix` gate, done-but-open ≥1 guard, RawLabel↔IllegalRows
disjointness, facet-only TomlParse via the `"facet"`-keyed risk diagnostic) is
implemented and unit-tested as designed.

But the slice is **not ready to close.** Two suite tests are RED and three
defects are blockers:

- **F-1 (ADR-001 gate RED).** The new modules `finding`/`doctor_checks` are
  unclassified, and — more seriously — `registry::spec_fk_findings` makes the
  `registry` *leaf* call `spec::build_registry` (*command*), an upward edge that
  breaks the layering invariant the design itself leans on. The fix is to move the
  composition up to the command layer (`spec.rs`), exactly where §5.1 said it
  belongs.
- **F-2 (memory golden RED).** The D12 safety net was implemented as a byte-exact
  assertion over the **live** corpus, baking a `commits-behind-HEAD` count that
  drifts on every commit. It went red the moment main moved (SL-169). A
  non-hermetic golden cannot be the stable behaviour-preservation proof D12
  requires.
- **F-3 (ProseCite scope).** The scanner globs the whole repo instead of the
  authored `.doctrine/**` corpus, descending into nested worktree copies
  (`.dispatch/`, `.worktrees/`) — 827 of 966 findings (86%) are duplicate
  worktree noise. Anchoring to `.doctrine/**` collapses it to ~139.

**Standing risks / tradeoffs.** The residual 139 ProseCite findings (after F-3)
are mostly design-accepted placeholder noise (R8: SL-999/REQ-999 family) plus an
**unmodeled** RFC-prefix↔IETF-RFC-number collision (F-4, tolerated). The JSON
contract diverged from the locked design on two points (F-5 bare array vs
`json_envelope`; F-6 missing `severity` field) — both small, both routed to
reconcile. A new 805-line module (`doctor_checks.rs`) is absent from the design
system model (F-7). Finally, an **evidence-integrity** note (F-9): the handover
reported "2716 pass, 0 fail" and called the layering failure "pre-existing and
unrelated" — it is neither; the funnel self-reported green over two red tests.
Trust the test run, not the handover.

**Verdict: REPAIR-THEN-RECONCILE.** F-1/F-2/F-3 (+ the F-10 nit) require a code
repair pass on the candidate before integration; the close-gate correctly refuses
`audit→reconcile` while they stand. Once repaired and the blockers verified, the
per-slice design reconciles (F-5/F-6/F-7) are mechanical. No governance/spec REV
is required — every finding lands per-slice or in code.

## Reconciliation Brief

### Code repair (on the candidate, before integration — blockers)
- **F-1** — relocate `spec_fk_findings` from `registry.rs` into `spec.rs`
  (command, already owns `build_registry`), restoring `registry` out=0 leaf
  purity; classify `finding = "leaf"`, `doctor_checks = "command"` in
  `.doctrine/adr/001/layering.toml`. Re-run `architecture_layering_gate` → green.
- **F-2** — make `tests/e2e_memory_validate_golden.rs` hermetic (seeded fixture
  corpus) or normalise the volatile `N commits behind HEAD` substring before the
  byte-compare.
- **F-3** — anchor `prose_cite_findings` glob to `root/.doctrine/**/*.md` (design
  D11) and/or exclude nested-worktree roots (`.dispatch/`, `.worktrees/`) in
  `is_disposable_prose_d11`. Confirm 966 → ~139.
- **F-10** (nit, sweep with the above) — renumber the `doctor_checks.rs` check
  comments to #6/#7/#8.

### Per-slice (direct edit during reconcile)
- **design.md §5.1** (F-7): add `src/doctor_checks.rs` to the System Model; add it
  as a slice selector.
- **design.md §5.4** (F-5): reconcile the `--json` shape — either amend the design
  to "bare array" or note the code will move to `json_envelope` (recommend the
  latter, for CLI --json consistency).
- **design.md §5.4** (F-6): reconcile the `severity` row field — recommend adding
  it to the serialized `Finding` row.
- **selectors** (F-8, optional): re-tag the read-only design-target selectors
  (`hydrate.rs`/`relation.rs`/`lifecycle.rs`/`catalog/scan.rs`/`relation_graph.rs`)
  as `scope-relevant`.

### Tolerated (no action; recorded)
- **F-4** — RFC↔IETF-RFC collision: accepted v1 limitation extending R8; advisory
  severity bounds it. Optional follow-up backlog only if noise proves intolerable.
- **F-9** — handover self-report was wrong; corrected understanding recorded here.

### Governance/spec (REV)
- None. No ADR/spec/policy/standard change is required.

## Repair Outcome (2026-06-28)

All three blockers + minors repaired on `repair/168` @ `eb8b6222` (the SL-168
impl bundle replayed onto current `main` e16bcf3c, churn dropped):

- **F-1** resolved — `spec_fk_findings` moved to `spec.rs`; `finding`/`doctor_checks`
  classified in `layering.toml`; `architecture_layering_gate` green (17/0).
- **F-2** resolved — memory golden reseeded onto a hermetic fixture corpus (`-p`).
- **F-3** resolved — ProseCite anchored to `.doctrine/**`; 966→118 on the live
  corpus; unit fixtures relocated under `.doctrine/` so the D11 skips are
  exercised, not bypassed.
- **F-5/F-6** resolved in code — `--json` now emits the `{kind, rows}` envelope
  with a `severity` field (matches locked §5.4), so no design amendment needed.
- **F-7** reconciled — design §5.1 updated to the as-built module layout.
- **F-8** — governance.rs churn dropped (trunk kept); policy/standard reformatting
  retained (rustfmt-mandated under the active edition — not gratuitous).
- **F-10** resolved — check-number comments corrected.
- **F-11** (found during repair; truncated audit runs had missed it) — slice-168.toml
  `related` rows made contiguous.

Full suite green (0 failed), `clippy -D warnings` clean, `fmt --check` clean.
RV-185 done · await=none. Ledger ready for `/close`.
