# Review RV-284 — reconciliation of SL-222

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed (F-2 record).** Dispatched slice — this audit runs against
the candidate interaction branch `refs/heads/candidate/222/review-001`
(`cand-222-review-001`, worktree at
`.doctrine/state/dispatch/candidate/cand-222-review-001`, tip `bafaa3a88e51`),
built from base `refs/heads/main` (`051367407` — main == edge at audit time)
plus the `impl_bundle` payload from evidence ref `refs/heads/review/222`
(`949baca4279d`). Evidence refs `review/222` / `dispatch/222` are immutable;
the ledger itself is driven from the parent tree.

**Lines of attack** (design.md is canon; RV-282's nine integrated findings are
part of it):

1. **Conformance algebra** — 216 undeclared paths and 3 undelivered selectors
   reported by `slice conformance SL-222`; disposition every cell (bulk
   backlog-TOML strips are expected migration output; `src/main.rs`,
   `src/priority/config.rs`, `.doctrine/adr/015/**` undelivered need
   explanation).
2. **The flip's correctness spine** — E6 ladder order (pin > human > projection
   > agent > migrated > [transitional facet, dead at PHASE-09] > bare anchor);
   E7 row-sourced bare anchor domination-by-construction incl. the F-4
   in-pipeline re-siting one-site pin; E4 per-field mean + affine linearity
   with floor-after-aggregation; E5 anti-laundering (`anchor_map()` ≡ Pin/Human
   operative costs only).
3. **Value-pass refactor gate (E3/RV-282 F-3)** — assertion semantics + goldens
   unchanged; accessor re-path the sole churn class.
4. **The deletion (Q4/F-5)** — widened grep-gate: no parse/consumption of
   `value`/`estimate` top-level TOML keys outside the scan-seam tripwire;
   magnitude-free `UnmigratedFacet` (F-9); NF-001 symbol-substring tripwire.
5. **Migration hygiene (E9/F-7)** — truly non-mutating `--check`; census
   reconciliation; never emits pins; idempotency key (path, lower, upper);
   `[risk]` untouched.
6. **Governance gate ordering (E11)** — REV against ADR-015 + SPEC-020
   REQ-269..277/310 disposition map approved before the flip landed; ADR-015
   text updated on apply.
7. **Evidence cadence (R1)** — four snapshots (pre-flip, post-flip,
   post-migration, final) present, separable, committed.
8. **Suites + gate** — full suite and `doctrine check gate` green on the
   candidate surface; VT keyword mandates hold per phase.

## Synthesis

SL-222 delivered the estimate half of the RFC-020 ledgered-claims transition
essentially as designed, and the design it conformed to was already
battle-hardened (RV-282's nine findings were integrated pre-implementation).
The audit ran against candidate surface `candidate/222/review-001`
(base `main` @ 051367407 + impl bundle `review/222` @ 949baca42, audit repair
3c456029e on top): full suite green, `doctrine check gate` green.

**Closure story.** The correctness spine held at every probed seam. The E6
ladder is literal in `graph.rs::est_cost` with per-rung isolation tests; E5
anti-laundering is exhaustively property-tested (16³ tier-mask sweep over
`anchor_map()`); the E7/F-4 bare anchor is derived in-pipeline with the
one-site pin (`CostCtx.absent == Pipeline.bare_anchor == est gauge centre`);
the migration executed with a fully reconciled census (185 = 185 + 0 + 0) and
a provably non-mutating `--check`; the deletion's widened grep-gate holds (no
`value`/`estimate` key consumption outside the parse-free scan tripwire); the
SPEC-020 REQ disposition map is applied. The three PHASE-09 worker-laundered
stubs the orchestrator repaired (a2775657a) were re-verified cold and are
genuinely repaired (F-6).

**What the audit changed.** Two fix-now findings: the missing E8 write-class
regression test for the estimate pin family (F-1 — the one real hole in the
verification plan's delivery; repaired at 3c456029e) and a PHASE-05 registry
range that missed the REV-026 approve/apply commits (F-2 — re-recorded, the
false "undelivered ADR-015" cell dissolved).

**Standing risks.** Low. The known residue is deliberate and behaviour-inert
(F-7 → CHR-047). The registry carries one honest wrinkle: the audit-repair
commit lives outside any phase delta, so `src/main.rs` reads undelivered
until integration (F-1 response). Conformance's ~190 undeclared `.doctrine/**`
paths are the census-bounded migration strip — accepted drift, selector
algebra can't express a data-dependent path set (F-5).

**Tradeoffs consciously accepted.** The R1 evidence came back null — four
byte-identical snapshots — where the design predicted a divisor-wide re-rank.
Verified genuine (fresh reproduction on the candidate matches modulo corpus
drift) and explained: operative-cost collapse is source-stable across
facet→claim motion and this corpus had zero est-domain projections to lose
(F-9). The flip's headline consequence (facets stop anchoring projection,
permanently) therefore has an empty extension *on this corpus* — the loud
findings + explain provenance remain the re-assertion prompts for any corpus
where it does not.

## Reconciliation Brief

### Per-slice (direct edit)

- **[F-3] Selector registry**: post-integration, `doctrine slice selector rm
  SL-222 src/priority/config.rs` (spurious undelivered — the SL-220-era
  demote-knob doc text already reads domain-generically; no edit was ever
  needed). Mirror: annotate design §3 code-impact line for config.rs.
- **[F-4] Selector registry**: post-integration, `doctrine slice selector add
  SL-222 tests/e2e_compare_elicit.rs` and `tests/e2e_priority_golden.rs`
  (orchestrator fixture-conversion intervention a2775657a), note-matched to
  the existing mid-flight declarations.
- **[F-8(4)] design.md prose mirrors**: §2/§3/§5 code-impact and deletion
  inventories undercount the shipped blast radius — add the 20 mid-flight
  ripple selector families (comparison initializer ripple; flip verb/guard/
  golden surfaces; show-line seam across memory/rec/retrieve/review/revision/
  spec; PHASE-09 deletion ripple across doctor/knowledge/map/reconcile/
  relation/estimate-display) so canon matches the registry.
- **[F-1 residual] slice-222 registry note**: after integration lands
  3c456029e, either re-run conformance to confirm `src/main.rs` conformant
  via the landed edit, or record the audit-repair range if the registry still
  reads undelivered.

### Governance/spec (REV)

- **[F-8(1)] RFC-020**: Phase-2 row → delivered-by-SL-222 (Phase 3 stays
  open); record the E1 payload deviation ({lower,upper} only — skew/unit/
  confidence deliberately not columns), the E5 records-anchor narrowing, and
  the F-9 null re-rank result (projection-free corpus ⇒ byte-identical
  snapshots; class-(b) evidence empty by construction).
- **[F-8(2)] spec-014 (PRD)**: prose still describes authored `[estimate]`
  tables; amend to claims-era wording (facet retired at SL-222 PHASE-09;
  tripwire + migration script are the remedy path). REV modify.
- **[F-8(3)] PRD-014/SPEC-020 retention REQs, PRD-011/SPEC-001 descent
  prose**: named non-contradicting at design §7 — verify and mark reconciled,
  no text change expected.

No plan.toml or criterion edits are proposed (all `PHASE-NN`/`EN-/EX-/VT-`
ids untouched). ADR-015 itself needs no further motion — REV-026 applied its
text pre-flip (verified conformant after the F-2 re-record).

## Reconciliation Outcome

Reconcile ran on the **integrated `main` tree** (dispatched slice), not bare
edge. Integration ritual: promoted `edge → main` (brought RV-284 onto main),
built + admitted a `close_target` candidate, `dispatch sync --integrate --trunk
refs/heads/main` landed the slice's code + authored governance (REV-026,
PRD-014/SPEC-020 amendments, requirement dispositions, slice deltas, notes,
phase snapshots) onto main. Reconcile leftovers authored on a `main` worktree;
primary tree stayed on `edge` throughout.

**Audit-surface split (key context).** The audit ran on the primary tree, blind
to the candidate's authored `.doctrine/` state, so its brief flagged items
already satisfied on the candidate. Confirmed post-integration:
- **F-8(2)** PRD-014 §1/§2 already claims-era (REV-026); only §6 primary/
  validation flow lagged — completed here (see REVs).
- **F-8(3)** SPEC-020 disposition map (REQ-269..277, 310) + the PRD-014/SPEC-020
  retention REQs already applied by REV-026; PRD-011/SPEC-001 descent prose
  verified non-contradicting. `spec validate` clean for SPEC-020, PRD-014,
  PRD-011, SPEC-001. No text change (F-8(3) satisfied).

### Direct edits applied (per-slice)
- `slice-222.toml` selector registry: removed `src/priority/config.rs`
  (spurious-undelivered, F-3); added `tests/e2e_compare_elicit.rs` +
  `tests/e2e_priority_golden.rs` as design-targets, note-matched to the
  orchestrator fixture-conversion `a2775657a` (F-4).
- `design.md`: §3 config.rs code-impact line annotated (F-3); Reconciliation
  addendum recording the 20 mid-flight ripple selector families so canon
  matches the registry (F-8(4)).
- `rfc/020/rfc-020.md`: Phase-2 row → delivered-by-SL-222, with the E1 payload
  deviation, E5 records-anchor narrowing, and F-9 null re-rank recordings
  (F-8(1); direct edit — the CLI/ADR-014 refuses an RFC as a REV `revises`
  target).

### REVs completed
- **REV-027** (`reconcile-sl-222`): done/approved — PRD-014 §6 estimate primary
  + validation flow amended to claims-era wording, mirroring the ratified value
  half (covers RV-284 F-8(2) residual). Rationale + before/after in
  revision-027.md.

### F-1 residual (audit repair on trunk)
The F-1 fix (`estimate_pin_is_orchestrator` write-class test, main.rs) was
committed **on the review candidate** (`3c456029e`), on top of `review/222`,
never folded into the bundle. The close_target sourced `review/222` and so
landed main **without** it — caught at the conformance re-check (`src/main.rs`
read undelivered). Repair cherry-picked onto trunk at close; conformance's
residual `.doctrine/**` undeclared set is the F-5-tolerated wrinkle (selector
algebra cannot express a data-dependent path set), unchanged.

### Withdrawn / tolerated
- **F-5** tolerated — conformance `.doctrine/**` undeclared set; rationale in
  finding disposition. **F-7** follow-up → CHR-047 (deliberate code residue).
  F-6/F-9 aligned; F-3/F-4/F-8 verified.

Reconcile pass complete — handoff to /close.
