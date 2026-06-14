# Review RV-029 — reconciliation of SL-066

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciles the SL-066 deliverable (the REV change-axis kind, ADR-013) against its
locked `design.md`, the authored `plan.toml` EX/VT criteria, and governance
(ADR-003/004/009/010/013, the SL-060 dep/seq invariant). The code bundle lives on
`review/066` @525a74f (15 files / 3273 ins), NOT on `main`; phase units
`phase/066-02..05`; coordination tip `dispatch/066` @18c3db3. PHASE-01 (ADR-013 +
doc relocations) already landed on `main`.

**Evidence gathered (parent-tree audit, bundle built in a read-only worktree on
`review/066` with an isolated `CARGO_TARGET_DIR` to defeat the shared-target
false-RED):**
- 20/20 `e2e_revision*` suites green; 1279 lib/bins tests green; clippy zero-warning
  (bins/lib, no `--all-targets` per convention).
- Behaviour-preservation holds: the only existing-test edits are the
  corpus-enumeration canaries (integrity prefix table, relation ALL-label list +
  overlay count 13→14, partition vocab, is_work_like allowlist) that MUST update
  when a kind is added — not behaviour changes.
- Integration safety: `review/066` cut from base 6062e28; `git merge-tree` onto
  current `main` (incl. the 3 post-base ISS-012 commits + SL-064 caaef20) reports
  **0 conflicts** — ISS-012 touched `.doctrine/agents`, disjoint from the bundle's
  `!.doctrine/revision/` negation.

**Lines of attack (the invariants the slice is held to):** the three corpus-walk
arms (G1/G2/G3) co-landing with the KINDS row; approval orthogonality
(approval-blind transitions, ADR-009); `revises` TypedVerbOnly + inbound-on-inspect
(ADR-004 §3); governance excluded as dep/seq target while REV is admitted both ways
(SL-060/IDE-010); apply's all-or-nothing from-guard, approval checkpoint, and
status-only→done / mixed→started disposition; REC schema untouched.

**External code-review findings** (provided by the user) are folded in below as
ledger findings alongside the auditor's own.

## Synthesis

**Outcome: reconciled, ship-ready, integrate sealed.** SL-066 delivers the REV
change-axis kind (ADR-013) faithfully to its locked design. The independent audit
pass and the external code review agree on the headline: the code is *correct* — no
blocker, no behaviour-preservation breach, no design deviation. All 10 findings are
quality concerns, not defects, and every one is terminal.

**The closure story.** The five-phase delivery landed all three KINDS-consumer
corpus-walk arms (G1 partition, G2 dep_seq, G3 outbound) atomically with the KINDS
row — the one hazard that could panic a debug-build scan the moment a REV is minted —
and the e2e suite proves a minted REV neither panics nor mis-classifies. The
load-bearing invariants hold under test: approval is orthogonal and transitions are
approval-blind (ADR-009); `revises` is `TypedVerbOnly` so the `[[change]]` payload is
the sole edge writer (no `doctrine link` footgun); inbound surfaces on `inspect`, not
`show` (ADR-004 §3); governance stays excluded as a dep/seq target while REV is
admitted both ways (SL-060 invariant intact / IDE-010 payoff realised); apply is
all-or-nothing behind the from-guard with the approval checkpoint, and `done` never
lies (status-only→done, mixed→started). REC schema is untouched. 20/20 revision e2e +
1279 lib/bins green, clippy zero-warning.

**Standing risks — none blocking.** The only risk on the domain map (integration
base) is retired: `review/066` was cut from 6062e28, but `git merge-tree` onto current
`main` — including the post-base ISS-012 and SL-064 commits — is **0-conflict**, because
ISS-012's `.gitignore` edit (`.doctrine/agents`) is disjoint from the bundle's
`!.doctrine/revision/` negation. Integration via `/close --integrate`'s 3-way is safe.
F9's "branch lags / rebase first" is a diff-base artifact (diffing `main...review/066`
renders main's newer commits as reverse-noise), not a bundle problem.

**Tradeoffs consciously accepted.** Six actionable quality findings (F2 test-harness
DRY, F4 `settle_disposition` `_=>` trapdoor, F5 1478-line module decomposition, F6
unit-testing the row-build validation, F7 magic-`0` placeholder, F8 TOCTOU doc-comment)
are deferred to **IMP-073** rather than applied at close. The deliverable is a
verified-green *sealed dispatch bundle*; the user elected to integrate it untouched and
land the cleanup as a focused follow-up slice — avoiding perturbing the bundle and
re-exercising the ISS-015 funnel import path for non-blocking quality work. IMP-073
scopes each item concretely and notes F5 can also split the over-grown REC module it
cites as precedent. Three findings are tolerated as design-intended or precedent-aligned
(F1 `allocated` is the operator-hand-fill anchor for the deferred creation-apply path,
design.md:228; F3 whole-file parse rides the REC precedent; F10 dup-slug is the
consistent all-kinds posture — identity is the id, not the slug). F9 is aligned (no
action).

The drift-surface posture (F8) is the right anti-grain: doctrine surfaces drift, never
silently clobbers — the from-guard aborts the *whole* apply on any stale row, writing
nothing. That property, and "done never lies" (M1), are the two correctness subtleties
both reviewers independently flagged as well-handled.
