# Review RV-172 — reconciliation of SL-159

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation audit of SL-159 (EVD + HYP epistemic kinds).
Dispatched slice — reviewed the **candidate review surface** `cand-159-review-001`
(`refs/heads/candidate/159/review-001`), built from the `review/159` impl bundle on
`main`; the `review/*` and `phase/*` refs are immutable evidence (R2). Audit repairs
landed on the candidate and it was admitted at `830cd857` (this RV governs it).

Lines of attack:

1. **Mechanical conformance** — `slice conformance 159`: every undeclared touch and
   undelivered selector accounted for.
2. **Touch-site completeness** — the ~17 hardcoded prefix sites (design §2, R1):
   `kinds.rs`, `integrity.rs` (KINDS +2 / count 21→23 / collision list),
   `partition.rs` rows, `search.rs`, `tag.rs`, `dep_seq.rs`, `supersede.rs`.
2. **Design §5 conformance** — status vocabs + gating partition; EVD `confirmed`
   non-terminal; HYP non-supersedable (D7); the closed `Provenance` enum + its
   mandated VT-3 drift-canary; `tested_by` dropped (D5); the `Supports`/`Disputes`
   relation plumbing + the `format_metadata`/`show_json` renderer wiring (codex F4).
3. **Behaviour-preservation gate** — `just check` green on the candidate.

## Synthesis

**Closure story.** SL-159 lands EVD and HYP as fully-modelled record kinds. The
implementation conforms to the design on every substantive axis: the `RecordKind`
enum and `RECORD`/`ALL` arity grow 4→6; `integrity::KINDS` gains two rows with the
count pin bumped 21→23 and the prefix/collision lists extended; the SL-158 gating
partition carries correct per-kind rows (EVD `captured,disputed` gating /
`confirmed,retracted,superseded` terminal — `confirmed` deliberately non-terminal so
it remains supersedable; HYP `proposed` gating / `confirmed,refuted` terminal);
`search.rs`, `tag.rs`, and `dep_seq.rs` mixed-superset literals all gained EVD/HYP;
supersede admits EVD (D7: HYP excluded). The DRY'd sites (`scan.rs`,
`test_helpers.rs`, partition guard) correctly needed no edit — they appear as
benign `undelivered` selectors, not gaps. The full `just check` gate is green on the
admitted candidate.

**The catalog-vs-conformance noise resolved cleanly.** The two `undeclared`/two
`undelivered` memory paths are a selector-vs-symlink artifact:
`mem.signpost.doctrine.knowledge` is a symlink to `mem_019ed2791edc…`, which *was*
updated to six kinds — the conformance engine just sees the slug and the hash-dir as
two different paths. `tests/e2e_memory_anchoring.rs` (declared but untouched)
references no kind catalog, so the design over-predicted it; the gate confirms it
needed no change.

**Two real findings, both repaired in scope (F-1, F-2).** The `Provenance` closed
enum shipped without the `KNOWN` drift-canary the design (§5.3) and its VT list (§9)
mandated — every sibling facet enum (`Basis`, `ConstraintSource`, `Confidence`)
carries one; its absence would let a future variant edit drift uncaught. Added the
const + `ValueEnum` derive + the `provenance_known_set_matches_variants` VT-3 test.
And `validate_matrix` carried an unreachable `Hypothesis` arm contradicting D7 (HYP
has no supersede policy, so it is rejected before the matrix) — removed.

**Standing risks / consciously accepted tradeoffs.**
- **F-3 (tolerated):** `src/commands/superserde.rs` is a pre-existing dead-duplicate
  of `supersede.rs` (on `main` since `98c75027`, no module decl, zero refs). SL-159's
  comment edit sprayed onto it inertly. Out of slice scope; orphan deletion tracked
  as **CHR-028**.
- **F-4 (follow-up):** a fresh dispatch candidate worktree fails the gate because the
  gitignored generated `web/map/dist/` RustEmbed folder is absent — environmental,
  not an SL-159 defect. Gate validated by staging `dist/` from the main tree. Tracked
  as **IMP-187**.
- **Dispatch hygiene (already remediated):** the PHASE-01 worker committed to the
  `edge` main worktree; the orchestrator reset `edge` and cherry-picked to the fork
  (handover). PHASE-02/03 ran correctly in the fork. Matches the known footgun in
  `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`. No residue on `edge`
  (verified: `edge` = `main` + the two unrelated cordage commits).

No blocker remains; the ledger is `done · await=none`.

## Reconciliation Brief

All four findings are code/process; **none touch a spec or governance entity**, so
there is no REV surface for this slice. The governance axis the design anticipated (a
Revision cut post-design) carries no reconciliation obligation from this audit.

### Per-slice (direct edit)
- **None outstanding.** F-1 and F-2 are already fixed on the admitted candidate
  (`830cd857`); reconcile/close need only integrate the candidate to trunk.
- Design accuracy note (optional, non-blocking): design §9's "three facet-enum drift
  canaries" is now four (Provenance added); `e2e_memory_anchoring.rs` listed in §5/§9
  as catalog-coupled is not. Correct in place if design.md is touched during close.

### Governance/spec (REV)
- **None.** No ADR/policy/standard/spec/REQ finding.

### Tracked follow-up (backlog, no reconcile action)
- **CHR-028** — delete dead orphan `src/commands/superserde.rs` (F-3).
- **IMP-187** — dispatch candidate worktree should stage generated embed assets (F-4).

### Integration note for /reconcile → /close
The reconciled truth is the admitted candidate `cand-159-review-001` @ `830cd857`
(base `main` / source `review/159` + the F-1/F-2 repair commit). Land it on `main`
via the dispatch integration path (`dispatch sync --integrate --trunk
refs/heads/main`, per AGENTS.md), then promote to `edge`.
