# Review RV-157 — reconciliation of SL-147

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed (F-2).** Dispatched slice. Reviewed the **candidate
interaction branch** `candidate/147/review-001` (`ad0a75f5`), created via
`dispatch candidate create` from the impl-bundle ref `review/147` (`186c2ef5`) —
a single squashed source-delta commit rooted exactly on `main` (`a9580f02`).
Coordination/evidence refs (`dispatch/147`, `review/147`) are immutable (R2);
not edited.

**Lines of attack.** Conformance of the shipped delta to design.md (RFC-004
v0.1) and governance (POL-002, ADR-001, ADR-006/012): (1) does the recorded
source-delta topology match design D5; (2) is the ADR-001 module layering
honoured by the new leaf modules; (3) is the staleness re-point /
domain_map burn behaviour-preserving; (4) base freshness / integration
readiness; (5) the two carried design-prose drifts (primary_worktree home;
double-write topology).

**Verify path.** `just check` GREEN in the candidate worktree (compiles +
tests `conformance.rs`, `globmatch.rs`, `e2e_slice_record_delta.rs`,
`e2e_dispatch_sync.rs`). Live `slice conformance 147` not runnable this
session (F-5).

## Synthesis

SL-147 ships its RFC-004 v0.1 scope: an accreting `[[selector]]` list on the
slice (D2), the arm-neutral recorded source-delta registry (D5), the pure
conformance algebra + `slice conformance` verb (D6), the `domain_map` burn with
the staleness reader re-pointed onto selectors (D4), and the lifecycle-skill
wiring + dogfood (D8/F-8). The behaviour-preservation gate holds — `just check`
is green, the review staleness computation survived its input swap, and the new
leaf modules (`boundary`, `conformance`, `globmatch`) are registered in ADR-001
layering. No code-correctness defect surfaced.

The closure story is **clean code, drifted design prose, and a deferred
integration hazard.** Three of five findings are confirmed-and-routed (none a
code defect); two are aligned (sanctioned process / evidence limitation).

The two design-prose drifts are both *home/topology* drift with the
*load-bearing decision intact* — the implementation is the better truth in each
case and the lifecycle skills already track it; only `design.md` lags:

- **F-1 (R-D5)** — the dispatch recording is arm-asymmetric in shipped code
  (claude `record-boundary` double-writes ref-cut + conformance registry in one
  call; codex/pi uses the separate `record-delta`), not the symmetric
  separate-beat topology design D5 describes.
- **F-2** — the cross-worktree resolver lives at `git::primary_worktree` (an
  ADR-001-clean leaf), not the design's `worktree::subagent::primary_worktree`
  (which would be an engine→command upward edge).

The standing risk consciously carried forward is **F-3**: the impl bundle was
cut from a stale `main` (28 commits behind `edge`, including RFC-005/ISS-025
edits to the very files SL-147 rewrote). Integration is `/close` stage-2 work,
but it must promote `edge`→`main` and merge onto current `edge` first, expecting
conflicts in `review.rs`/`dispatch.rs`/`state.rs`. Recorded so close does not
integrate blind.

## Reconciliation Brief

### Per-slice (direct edit to design.md)
- **design.md D5 (dispatch-arm bullet, ~L212-226) [F-1]** — rewrite to the
  shipped **arm-asymmetric double-write**: the claude arm's `dispatch
  record-boundary` writes BOTH the committed ref-cut ledger AND the arm-neutral
  conformance registry in one call (`dispatch.rs:552-560`); the codex/pi arm
  (no `record-boundary`) uses the separate `slice record-delta` at funnel step
  8. Drop the "we do not touch record-boundary / both arms call record-delta"
  framing. Load-bearing decision (both arms populate the registry off
  coordination oids) unchanged. Lifecycle skills already reconciled in P06.
- **design.md D5/R5/F-5/OQ-conf-3/D7 [F-2]** — correct the resolver home
  pointer from `worktree::subagent::primary_worktree` to
  `git::primary_worktree` (relocated in P02 to fix an ADR-001 engine→command
  upward-edge violation; user-confirmed). Decision (reuse the existing
  resolver, do not reinvent) unchanged.

### Governance/spec (REV)
- None. Both prose drifts are per-slice design-doc edits; no ADR/spec/policy
  change is implicated (ADR-001 layering.toml was correctly updated in-slice).

### Close-gate note (for /close stage-2, not reconcile)
- **F-3** — before `dispatch sync --integrate`: `git fetch . edge:main` to
  promote edge, then merge/rebase the bundle onto current `edge`. Expect
  conflicts in `src/review.rs` (-568 lines), `src/dispatch.rs`, `src/state.rs`
  against RFC-005 / ISS-025. Do not integrate from the stale `main` base.
