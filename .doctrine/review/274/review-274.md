# Review RV-274 — reconciliation of SL-211

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-211 (split-lineage close recovery — the
`dispatch sync --record-integration` verb). Self-audit, conformance mode.

**Surface reviewed (F-2 record).** The dispatch code bundle `review/211`
@ `5d3687df` ("review(211): impl bundle") — phases 01–03, built + tested in an
isolated worktree off that ref. PHASE-04 (doc/memory reconciliation) is an
empty-code phase delivered **main-thread on edge** (the 3 recovery memories +
close skill + dispatch-mechanics), verified out-of-band via VA-1 (agent
inspection) and VH-1 (operator acceptance) — not carried in the review bundle by
design.

**Lines of attack / invariants held:**
- Design conformance — the shipped verb matches design §5.2/§5.4/§5.5 and honours
  INV-1 (no external ref), INV-4 (never `review/<N>` as payload), D2
  (`expected_old = planned`), D6 (payload = phase-chain tip / admitted
  `close_target`), D7 (replace a non-applied trunk row), F-4 (`--trunk` ==
  `deliver_to` guard).
- RV-263 (external codex) findings actually implemented — candidate-active
  refuses without a `close_target` admission (F-1); a stale Failed/Pending trunk
  row is *replaced*, not refused (F-2).
- No parallel implementation (R2 / D3) — `resolve_trunk_payload` is the single
  seam shared by all three planners.
- VT coverage — every design VT-1..8 has a passing test on the review surface.
- Path conformance — `slice conformance SL-211` clean on the code surface.
- Behaviour-preservation (VT-7) — the existing dispatch/ledger suites stay green
  unchanged.

## Synthesis

**Clean audit.** The implementation on `review/211` faithfully realises the
design and passes on the actual review surface:

- **Design ↔ code conformance is exact.** `recorded_row` sets
  `expected_old = planned = payload`, all OIDs = payload, status Verified
  (§5.3 / D2), with an asserting test. `resolve_trunk_payload` returns the
  phase-chain tip (legacy) or the admitted `close_target` (candidate-active),
  refusing when candidate-active without an admission — never `review/<N>`
  (INV-4 / D6 / RV-263 F-1), via the `NO_CLOSE_TARGET_ADMISSION` named constant
  (STD-001). The seam is shared by `plan_recorded_trunk_row`, `plan_trunk_row`,
  and `plan_candidate_trunk_row` (D3 / R2 — no parallel implementation).
  `run_record_integration` commits only the `dispatch/<N>` journal row (INV-1,
  no `with_journaled_projection`), guards `--trunk` == `deliver_to` (F-4), and
  replaces a non-applied trunk row while no-op-ing a Verified one (D7 / F-2). The
  slice-shell Blocked prescription names `--record-integration` (VT-8).
- **Green on the review surface.** SL-211 unit tests 11/0; `e2e_dispatch_sync`
  44/0 including `vt1_record_integration_closes_split_lineage_slice`,
  `vt2_close_integration_honours_deliver_to_override`, and the `vt7_*`
  behaviour-preservation cases; `cargo clippy --workspace` clean. Every design
  VT-1..8 maps to a passing test. (The initial worktree build failed on the
  RustEmbed `web/map/dist/` asset — a gitignored embed folder a plain
  `git worktree add` doesn't provision, in a file SL-211 never touches;
  resolved by copying the asset. Not a slice defect — captured as harvest.)
- **Conformance clean.** 4/4 code paths conformant, 0 undelivered; the lone
  undeclared path is the slice's own `slice-211.toml` lifecycle metadata (F-1,
  aligned).
- **Handover concern discharged.** The `≈ UNATTRIBUTABLE` verify-vt classification
  was purely the pre-derivation state; after `prepare-review` derived the
  registry, conformance attributes all four code files. It did not persist into
  audit.

**Tradeoffs / standing notes (no material risk):**
- SL-211 *builds* `record-integration` but its **own** close is the ordinary
  integrate path — the slice code lives only on `dispatch/211`/`review/211`; main
  is deliberately behind edge and is advanced by `dispatch sync --integrate` at
  close (promote `edge→main` first). It is not a self-referential split-lineage
  close.
- The implementation settled both of design §10's "still open (minor)" questions
  (hard-refuse on `--trunk`≠`deliver_to` with default-when-absent; emit the
  payload OID in the success line). Sound choices; only the §10 prose is stale
  (F-2 → reconcile).

No blocker findings; the close-gate is clear.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §10** (finding F-2): the two "Still open (minor, non-blocking)"
  questions are resolved in the shipped verb — update §10 to record the
  resolution rather than leaving them framed as open. (1) `--trunk` ≠ `deliver_to`
  → hard `bail!` naming both refs; absent `--trunk` → defaults to `deliver_to`
  (VT-6 / `run_record_integration_refuses_trunk_deliver_to_mismatch` +
  `vt2_close_integration_honours_deliver_to_override`). (2) the success line emits
  the payload OID (`record-integration: recorded Verified trunk row … (payload …)`).
  No code change; prose-only, so canon tells the truth.

### Governance/spec (REV)
- None. No ADR, spec, policy, or standard divergence surfaced — the design
  already routed its governance frame (RFC-016 §C/§D, SPEC-022, ADR-012 FF-only)
  and the code preserves each invariant.

## Reconciliation Outcome

### Direct edits applied
- **design.md §10** (RV-274 F-2): rewrote the "Still open (minor, non-blocking)"
  paragraph to record the resolution the shipped verb actually settled — (1)
  `--trunk` ≠ `deliver_to` hard-refuses (`bail!` naming both refs; absent `--trunk`
  defaults to `deliver_to`, VT-6); (2) the success line emits the payload OID.
  Preserved the still-open retroactive SL-147/SL-190 re-land note. Prose-only; no
  code change. Canon now tells the truth.

### REVs completed
- None. Reconciliation brief carried no governance/spec item — no ADR, spec,
  policy, or standard divergence surfaced.

### Withdrawn / tolerated
- RV-274 F-1 (nit): aligned — the lone conformance `undeclared` is the slice's own
  `slice-211.toml` lifecycle metadata, not a code path. No write needed.

Reconcile pass complete — every brief item resolved. Handoff to /close.
