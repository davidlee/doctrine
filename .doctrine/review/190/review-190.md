# Review RV-190 — reconciliation of SL-171

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject / surface reviewed.** SL-171 (read-surface upgrade to `doctrine next`:
facet columns + `--columns` + pagination). Dispatched slice (`/dispatch`,
pi/subprocess + confined arm). Evidence refs are immutable: `review/171`
(impl-bundle tip `11b83373`, = `main`@`66bee51a` + one bundle commit),
`dispatch/171` (coordination, ledger `5d24e444`). No `dispatch candidate` was
created — `review/171` *is* the reviewable projection (0 ahead / 1 behind `main`,
merge-base = `main`). Audit reviews `review/171`; checks run in a throwaway
worktree `.dispatch/audit-171` (the main tree stays on `edge`).

**Lines of attack.**
1. **Conformance algebra** — `slice conformance SL-171`: 8/8 src conformant, 0
   undelivered, 2 *undeclared* test edits. Confirm the undeclared edits are
   design-sanctioned, not scope creep.
2. **Design ↔ code** — every EX-1…EX-6 (both phases) realised; no scope drift
   beyond the locked design (§3–§6, D1–D7, F1–F13).
3. **Verification integrity** — do the VTs have real automated coverage? The
   `verify-vt` gate reports all 10 VTs UNCHECKABLE (no structured mandate);
   separate the *mandate gap* (plan-authoring) from actual *test coverage*.
4. **Invariants held** — behaviour-preservation (the `format_truncation_notice`
   lift + unchanged `--json` payload proved by unmodified goldens); STD-001 (no
   magic strings — `ABSENT_CELL`, `NEXT_LIMIT_DEFAULT`); ADR-001 layering (no new
   edge from the `listing` lift); the SL-047 render-source-of-truth (surface
   reads facets, never recomputes).
5. **Dispatch funnel integrity** — the NF-001 facet-allowlist tripwire that
   PHASE-01's worker tripped + misreported green; the confined-arm self-commit
   failure (orchestrator imported the working-tree diff). Did the orchestrator's
   catch leave the surface correct?

## Synthesis

**Verdict: SL-171 is sound and ready to reconcile.** The read-surface upgrade to
`doctrine next` (facet columns + `--columns` + pagination) realises the locked
design EX-1…EX-6 across both phases, with no scope drift beyond the eight declared
src files. The conformance algebra was clean (8/8 conformant, 0 undelivered); the
only signals were two *undeclared* edits — both test files outside the src-only
selectors, both design-sanctioned (F-1 golden update per §10 F4; F-2 NF-001
allowlist required by the EX-3/EX-5 facet exposure). Neither is scope creep.

**Independent verification** (review surface, fresh worktree `.dispatch/audit-171`):
e2e_priority_golden 17 passed, e2e_estimate_non_blocking 2 passed (NF-001 tripwire
satisfied), priority::render unit 12 passed. The design's headline invariants hold:
behaviour-preservation (the `format_truncation_notice` lift + unchanged `--json`
payload proved by the unmodified retrieve/next-json goldens), STD-001 (`ABSENT_CELL`,
`NEXT_LIMIT_DEFAULT` named consts — no magic strings), ADR-001 layering (the listing
lift adds no edge; both callers already import `listing`), and the SL-047
render-source-of-truth (surface projects facets from `NodeAttr.facets`, never
recomputes). The F1 division-by-zero guard is doubly defended (call-site `limit != 0`
intent + fn-internal `page_size == 0 → ""`).

**The one real gap (F-3, major): PHASE-02 pagination shipped with no automated
coverage at the `next` surface.** PHASE-01 was genuinely well-tested; PHASE-02's
limit/offset slice, footer guard, CLI page→offset resolution + `--page` validation,
and the D7 visible-slice tags gate were entirely unexercised — `format_truncation_notice`
was covered only transitively via retrieve goldens. Disposed **fix-now** (user
decision): the audit added 11 tests (7 unit + 4 black-box) on the candidate repair
surface `candidate/171/review-001` (commit 47123c58, admitted at the same OID,
linked to this RV). Post-fix: render unit 19 passed, e2e_priority_golden 21 passed,
clippy + fmt clean.

**Standing risk / accepted tradeoff (F-4, tolerated): the verify-vt gate is inert.**
All 10 plan VTs are prose-only, so the conclude-S6 / audit `verify-vt` reports them
UNCHECKABLE — zero mechanical S3 coverage signal. It is non-halting (INV-4) but
masked F-3 from automated detection (caught here by hand instead). SL-171's plan is
left prose-only (retroactive backfill is marginal); the systemic fix — the `/plan`
skill authoring structured VT mandates — is captured as **IMP-209**.

**Dispatch funnel.** The funnel held despite two worker-side failures: PHASE-01's
worker misreported green and tripped the NF-001 facet allowlist; the orchestrator's
coord-tree verify caught both and completed the allowlist (F-2, design-sanctioned).
The confined arm's worker couldn't self-commit (linked-worktree object store under a
ro-bound `.git`) and the orchestrator imported the working-tree diff; two
confined-script bugs were found and fixed in trunk (66bee51a). Net: the admitted
surface is correct; the failures were absorbed by the orchestrator-sole-writer
discipline, not leaked into the bundle.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md line 3 (F-5)** — status reads `drafted (pre-lock)`; the design is
  locked, both phases implemented + dispatched, slice in audit. Update the status
  line to reflect the locked-and-implemented state. Pure prose-truth fix.

### Governance/spec (REV)
- *None.* No finding touches a spec, requirement, ADR, policy, or standard. F-4's
  systemic remediation is tooling/process work tracked as the backlog item IMP-209,
  not a governance REV.

### Integration note (for /close, not reconcile)
- The audited+admitted surface is `cand-171-review-001` @ `47123c58` (review_surface,
  base `main`, source `review/171` + the F-3 test commit). `/close` integrates from
  the admitted candidate, not raw `review/171` — the pagination tests live only on
  the candidate tip.
