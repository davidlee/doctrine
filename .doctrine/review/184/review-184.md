# Review RV-184 — reconciliation of SL-169

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface audited.** SL-169 was driven by `/dispatch`; per the audit lens this
review targets the **candidate interaction branch**, not the raw evidence refs.
The reviewed surface is `candidate/169/review-001` — initially the no-ff merge of
the impl bundle `review/169` (57cf3d3f) onto `main` (7990a6df) at `5544ba94`, then
the audit repair commit `a842e5b6` (RV-184 F-1/F-2). The admitted OID is
`a842e5b6` (`candidate admit --review RV-184`).

**Lines of attack.**
1. **Behaviour-preservation gate.** SL-169 changes shared list machinery; the
   full suite must be green. Probe every kind whose default column set gained a
   conditional `tags` splice — especially the governance dispatch (adr/policy/
   standard share `governance.rs`), where one edit changes three kinds' output.
2. **Conformance algebra.** Run `slice conformance 169` and disposition every
   undeclared (scope creep / missed selector) and undelivered (dropped work /
   stale design) cell against `design.md` + `plan.toml` VT criteria.
3. **VT-1 completeness.** The plan's PHASE-05 VT-1 has two halves — the columns
   golden *and* the parse-conformance matrix. Verify both, not just the louder one.
4. **False-green pressure.** The dispatch handover claimed all-green with "3
   pre-existing DOCTRINE_WORKER" failures; verify that claim against an actual
   re-run rather than trusting it.

## Synthesis

**Closure story.** SL-169 wires the columns/tags read surface — `--columns` on
`relation list`/`census`, a `tags` column in `--columns` for the taggable kinds, a
conditional default `tags` column, concept-map header lowercasing, and REC/review
taggability. The behaviour is correct and broadly test-covered (the
`e2e_list_columns_golden` + `e2e_adr_cli_golden` suites pass), but the dispatch
landed it **false-green**: the handover reported "2677 pass, 3 pre-existing
DOCTRINE_WORKER failures, 1 SL-168 TOML failure". A clean re-run of the candidate
shows **no DOCTRINE_WORKER failures exist** — the 3 "environmental" failures were
SL-169's own `e2e_standard_cli_golden` regressions (F-1). The standard golden was
left asserting the pre-tags shape while the adr golden beside it was updated; the
shared `governance.rs` dispatch renders the new column for all three governance
kinds, and the tagged STD-001 fixture made standard the one that tripped. This is
the audit's headline catch: a completed-and-handed-off slice was not actually
green.

**Conformance.** The algebra flagged 4 undeclared paths and 1 undelivered
selector. The undeclared edits (F-3) are all correct, compile-necessary fallout
of the Rec `tags` field and the relation columns-arg wiring — benign selector
under-declaration, not scope creep. The undelivered selector (F-2) was real
dropped work: `e2e_list_conformance.rs` was mandated by `design.md` and plan VT-1
to gain relation/census coverage and never did, leaving VT-1 half-satisfied.

**Repairs (fix-now, in audit scope).** F-1: regenerated the 3 standard goldens to
the new tags-column shape. F-2: added `relation_list_and_census_accept_columns` —
a focused `--columns` parse-conformance test, the correct shape because relation
list/census do **not** ride the `CommonListArgs` spine and must not be added as
`SPINE_KINDS` rows. Both committed at `a842e5b6`. Post-repair the suite is green
except one pre-existing, out-of-scope failure.

**Standing risks / tradeoffs accepted.**
- **Latent golden fragility.** Only `standard` tripped because only its fixture
  carried a tagged row; `slice`/`spec`/`revision`/`rfc`/`knowledge` goldens stayed
  byte-identical because their fixtures are untagged. The conditional-default gate
  is sound, but any future tagged fixture in those suites will regenerate the
  golden — expected, not a defect.
- **Pre-existing SL-168 corpus failure** (`relation_rows_of_one_label_are_contiguous`
  on `slice-168.toml`) is unrelated to SL-169, inherited from `main`, and out of
  scope — noted, not dispositioned against this slice.
- **Pre-existing fmt debt** (F-4) in `policy.rs`/`standard.rs`/`governance.rs`
  `parse_ref` tests pre-exists on `main`; SL-169's own code is fmt-clean. The edge
  working tree already carries the uncommitted fix — commit it independently.

## Reconciliation Brief

### Per-slice (direct edit)
- **`design.md` §Touch-table / `slice-169.toml` selectors** — widen the
  `design-target` selectors so the four correct-but-undeclared edits are declared:
  `src/commands/cli.rs` (relation `--columns` dispatch wiring), `src/mcp_server/tools.rs`
  and `src/reconcile.rs` (Rec `tags` field fallout), `tests/e2e_adr_cli_golden.rs`
  (adr golden regen). Closes the F-3 conformance `undeclared` gap so canon matches
  the delivered edit set. (Cosmetic; no behaviour implication.)

### Governance/spec (REV)
- **None.** No spec or governance finding surfaced. All findings were per-slice
  code/test (F-1/F-2 fixed in audit scope) or per-slice metadata (F-3). F-4 is
  out-of-scope base debt, not an SL-169 reconciliation item.

### Integration notes for /reconcile → /close (not findings)
- The reviewed/admitted surface is `candidate/169/review-001` @ `a842e5b6`; it is
  **not yet integrated** to `main`/`edge`. `main` is at the plan commit 7990a6df;
  the bundle sits directly on it.
- `edge` carries a **duplicate PHASE-01 commit** `c838bb71` (the same
  `listing::default_with_tags` helper the bundle already contains) — reconcile/land
  must resolve this so the helper isn't double-applied.
- Promote `edge → main` before integrating (the dispatch landing-zone ritual), then
  land the admitted candidate.

## Reconciliation Outcome

### Direct edits applied
- `slice-169.toml` selectors: added 4 `design-target` rows — `src/commands/cli.rs`,
  `src/mcp_server/tools.rs`, `src/reconcile.rs`, `tests/e2e_adr_cli_golden.rs`.
  `slice conformance 169` now reports `undeclared (0)`. (RV-184 F-3)
- `design.md` §Code-impact: added matching 4 rows so prose canon equals the
  delivered edit set. (RV-184 F-3)

### REVs completed
- None. Brief surfaced no governance/spec finding; F-1/F-2 were fixed in audit
  scope at `a842e5b6`, F-3 is per-slice metadata (above), F-4 is out-of-scope base
  debt.

### Withdrawn / tolerated
- None. All 4 findings `verified`; remediation recorded here, dispositions
  unchanged.

### Deferred to /close (integration, not reconcile writes)
- `undelivered (1) tests/e2e_list_conformance.rs`: F-2's parse-conformance test
  landed in that exact file at `a842e5b6`. The cell is stale only because the
  admitted candidate is not yet on `edge`/`main`; it self-resolves on integration.
- Promote `edge → main`, resolve the duplicate PHASE-01 `c838bb71` (same
  `listing::default_with_tags` helper the bundle carries), then land `a842e5b6`.

Reconcile pass complete — handoff to /close.
