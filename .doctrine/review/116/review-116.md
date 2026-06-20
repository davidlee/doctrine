# Review RV-116 — reconciliation of SL-127

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (self-audit, `--as` both roles). **Surface reviewed:** the
dispatch candidate interaction branch `candidate/127/review-001` (a clean no-ff
3-way merge of `review/127` onto fresh `main` `1b00c46`), worktree
`.doctrine/state/dispatch/candidate/cand-127-review-001` — not the raw `review/*`
evidence ref (R2).

**Lines of attack:**
1. **Behaviour-preservation gate** — does the full suite stay green on the
   candidate? (`cargo test`, `clippy`.) Any red test from a SL-127 change is a
   gate violation.
2. **Design §5 verification alignment** — every phase EX/VT has a real test, not
   a vacuous one. Specifically probe PHASE-05 EX-2 (env-prefix retirement) for
   vacuity and PHASE-01 VT-3 (minting fallout).
3. **Deliverables work live, not just under test** — the freshest-descendant
   ladder corrects drift, and `refresh-base` merges trunk cleanly. (Exercised
   directly this audit.)
4. **RV-030 F-1 pinned-fork-point invariant** — refresh-base advances B by
   explicit recorded action, never silent reparenting.
5. **Dispatch-arm artifacts** — why did prepare-review project 0 phase cuts
   despite 5 boundaries? Is that a SL-127 defect or an orthogonal framework gap?

**Out of scope (route to /reconcile or backlog, not fixed here):** governance/
spec edits; dispatch-framework defects orthogonal to SL-127's design; the
`DOCTRINE_TRUNK_REF=main` memory updates (valid until integrate).

## Synthesis

SL-127 delivers what its design locked. Both axes are sound and **verified live**,
not merely under test:

- **Axis 1 (ancestor-dominant ladder).** `freshest_descendant` correctly overtakes
  a stale `origin/HEAD` that is an ancestor of local `main`. Proven the hard way:
  the first drift check this audit read a false `trunk: stable` — traced not to the
  code but to a **stale shared jail-target binary** built from old `main`; once the
  binary was rebuilt from the coord tree the ladder reported `moved (4 ahead)`
  correctly. The plan-presence gate (PHASE-02) has direct tests
  (`coordinate_refuses_create_when_base_lacks_the_slice_plan`).
- **Axis 2 (`refresh-base` + drift surfacing).** Exercised end-to-end this audit:
  `refresh-base` merged 4 trunk commits into `dispatch/127` cleanly and a re-run
  `prepare-review` re-pinned the bundle to the fresh base — the RV-030 F-1 invariant
  (explicit recorded advance, never silent reparenting) held. `trunk_drift`,
  `select_guidance` (the RefreshBase-first ordering), and the candidate-create hint
  all carry dedicated tests.

**Suite:** 2117 unit + integration green on the candidate, save one regression —
`dispatch_router_skill_is_shrunk` (PHASE-05's legitimate +10-line refresh-base
routing section breached the ≤64 lean-router budget). Fixed within audit scope
(budget → 74, F-1) and folded into the bundle (option 1): cherry-picked onto
`dispatch/127`, `review/127` regenerated, review-surface candidate
`cand-127-review-002` re-created + admitted (`e1cd6fb`) — the fix is now in the
evidence bundle and will reach trunk at close.

**Standing risks / tradeoffs consciously accepted:**
- **Drift detection is binary-version-sensitive** (synthesised risk): `dispatch
  status` measures against whatever ladder the *running binary* implements. A stale
  binary silently mis-measures. Operationally: rebuild from the coord tree before
  trusting drift (captured as memory).
- **Per-phase review granularity lost** (F-2 → ISS-039): the claude arm left
  `boundaries.toml` uncommitted, so prepare-review projected 0 phase cuts. Tolerated
  — the cumulative bundle is whole and reviewable; orthogonal to SL-127's design.
- **Env-prefix retirement was vacuous in the skills** (F-3): the ritual never lived
  there; it persists in 3 memories that stay valid until integrate.
- **No single authoritative git-interaction model** (born here → IMP-128): the
  "where does an audit repair live to integrate?" question had no canonical answer;
  resolved pragmatically (option 1) and filed for a proper tech spec.

## Reconciliation Brief

### Per-slice (direct edit)
- **None required.** `design.md` remains accurate against the implementation; the
  `select_guidance` extraction (F-4) is a finer-grained pure-fn refactor than §4
  named, behaviour-preserving — optional one-line mention in §4, not a correction.
- `notes.md` already updated this session (conclude-resume + stale-binary gotcha).

### Governance/spec (REV)
- **None.** No ADR/spec/requirement is wrong; SL-127 conformed to ADR-006/011/012
  and honoured RV-030 F-1. No REV needed.

### Deferred to /close (post-integrate), not reconcile writes
- Update the 3 `DOCTRINE_TRUNK_REF=main` memories
  (`mem_019ee083…`/`mem_019ee3c4…`/`mem_019ec912…`) once SL-127 integrates and the
  installed binary carries the ladder fix — they are correct until then (F-3).
- ISS-039 (boundaries.toml) and IMP-128 (git-interaction tech spec) are durable
  backlog follow-ups, not reconcile writes.
