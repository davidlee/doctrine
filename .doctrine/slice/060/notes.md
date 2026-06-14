# SL-060 — durable implementation notes

## PHASE-01 (canon-first spec amendment) — DONE, VH-1 signed 2026-06-14

Canon outcomes are authored canon now (durable): PRD-011 §1/§2/§3 + REQ-258
(FR-009) + REQ-097 cross-ref. Commits fa38369 (amendment) + 3bf5b99 (boundary
tightening). Read them via `doctrine spec show PRD-011` / `requirement-258.{toml,md}`.

Durable carry-forwards (the gitignored phase sheet is not a safe home):

- **Endpoint rule is a CLOSED ALLOWLIST.** Valid `needs`/`after` endpoints = slice +
  the backlog kinds; **every other admitted kind refused** (governance, spec,
  requirement, knowledge, drift, review, reconciliation). Stated positively on
  purpose — a denylist-by-example rots as the corpus admits more kinds. REQ-258 #3
  enumerates the full refused set.

- **PHASE-03 test-coverage delta (not a logic change).** The D2 predicate is already
  allowlist-by-construction (design §5.2 lines 198–199, D2 line 273), so drift/review/
  rec are refused for free. But plan VT-2 names only 4 refusal examples (gov/spec/req/
  knowledge); REQ-258 #3 now widens the *verification* expectation. PHASE-03 refusal
  tests must cover a representative beyond the 4 — pick a resolvable non-work entity
  not in the original list (RV/REC if they're in `integrity::KINDS`; else that hits the
  unresolvable-target path, also refused). Free-text and self-edge refusals unchanged.

- **Why non-work is refused (don't re-litigate).** A non-terminal non-work predecessor
  genuinely blocks (`channels::blocked_by` filters by StatusClass≠Terminal, not by
  kind) — "allowed-but-inert" was refuted. Cross-tier governance→work gating is a
  *distinct* future surface (IMP-047 labelled `gates` + non-actionable status-class),
  deliberately not this `needs`/`after` surface. slice→requirement lineage uses
  `descends_from`, never `needs`. Full rationale: REQ-258.md Rationale.

- **Filed this session:** IMP-058 (render the requirement `.md` prose tier — currently
  unreachable via `spec show`), relates to IMP-057 (requirement authoring skill).

## PHASE-02/03 dispatch carry-forwards (for audit)

- **PHASE-02 landed b0d3e3d** — `src/dep_seq.rs` leaf (DepSeq/AfterEdge/RelEdit/read/
  strict append, non-destructive refuse message). Backlog repointed to leaf type+write;
  kept single `read_item`+`dep_seq_for` for `promoted` (one parse, F3). INV-2 backlog/
  priority/order goldens byte-identical.
- **PHASE-03 landed ec0d14b** — generic `doctrine needs`/`after` ride `link`'s
  cross-kind resolver; work-like gate = `is_work_like(kind)` (slice + SL/ISS/IMP/CHR/
  RSK/IDE) in main.rs, the single widen-later guard. Backlog verbs unchanged at source
  (already delegated to leaf at base; cycle refuse intact). Slice scaffold seeds
  `[relationships]` both arrays before `[[relation]]`; slice show/--json surface dep/seq.
- **AUDIT FLAG — existing golden amended by design.** `tests/e2e_relation_migration_
  storage.rs::assert_slice_shape` asserted (SL-048 "the cut") the slice template emits
  NO `[relationships]` header. SL-060 §5.3/E9 deliberately reinstates the table for the
  dep/seq PAYLOAD axis (structural axes stay cut). Worker rewrote the golden to assert
  the table IS present (needs/after) while structural axes (specs/requirements/
  supersedes/governed_by/related) remain cut + link-guidance comment retained. NOT an
  INV-2 behaviour-preservation golden (those stayed green); design-mandated, transparent.
- **VT-2 widened coverage met** — minted a resolvable RV-001 in the test root; refused
  by the work-like kind assertion (not the unresolvable path) + a unit test sweeps
  `integrity::KINDS` asserting only slice+5-backlog pass. Satisfies REQ-258 #3 closed
  allowlist.
- **PHASE-04 landed 521f12e** — engine `dep_seq_for(root,kind,id)->(DepSeq,promoted)`
  (relation_graph.rs, mirrors `outbound_for`): slice arm via leaf read (promoted=false),
  backlog arm via single read_item (one parse), non-authoring kind short-circuits to
  empty with NO disk read (F5). priority/graph.rs §3b read-gate generalised from
  `backlog::kind_from_prefix` to kind dispatch; emission `:257-277` byte-identical
  (DD-2); channels/partition untouched. INV-2 priority goldens green.

## PHASE-05 backfill (inline, data-only) — landed b5d14f7 + 061 addendum

- All 63 slice TOMLs now carry `[relationships]` { needs=[], after=[] } before the
  first `[[relation]]` row. 001-060 backfilled by b5d14f7; 062/063 by the concurrent
  SL-062/SL-063 agent (new template); 061 by the PHASE-05 addendum (excluded from the
  main commit while it was mid-close, backfilled once committed/clean). VT-1 storage
  post-check: 0 violations. VT-2: `doctrine validate` corpus clean; empty arrays are
  byte-stable in `slice show` (additive design — keys omitted when unauthored).
- **Backfill mechanism note (for audit/repro):** textual insert (no tomlkit/toml_edit
  available inline) after the leading top-level scalar run. INITIAL bug: inserting at
  the first non-scalar line corrupted SL-002 (a `# superseded` comment interrupts its
  scalar block, pushing status/created/updated under `[relationships]`). FIX: insert
  after the *last* top-level scalar that precedes the first `^[` table header. Re-verified
  validate-clean. Only SL-002 was affected; fixed before commit.

## Dispatch / shared-main hazards encountered (durable, for audit)

- Shared `main` had a LIVE concurrent agent (SL-061/SL-062/SL-063, IMP-023) the whole
  run — HEAD moved between every spawn and import; a dirty foreign INDEX once blocked
  the funnel (waited for it to clear, then re-anchored). All 3 worker funnels re-anchored
  to the moved HEAD; deltas were disjoint each time.
- **rtk masks git exit codes / stat-proxies diff** — used `ls-tree --name-only` +
  output-content greps (not `cat-file -e` exit) for funnel guards; imported via
  `git checkout S -- <paths>` (not `git diff|apply`). Recorded as
  [[mem.pattern.tooling.git-cat-file-e-exit-masked-use-ls-tree]].
- **glob `git add` on `.doctrine/slice/*/slice-*.toml` swept a foreign UNTRACKED file**
  (SL-063's uncommitted slice TOML) into the backfill commit; caught via the `create
  mode` line, removed by `--amend` + `git rm --cached`, letting the SL-063 agent commit
  it themselves. Lesson: on shared main, never glob-add authored-entity dirs — stage the
  exact known file set; untracked foreign files match the glob.

## /audit outcome — RV-020 reconciliation (2026-06-14) — DONE

Lifecycle `audit → reconcile`. RV-020 derives `done` (2 findings, both terminal).
Reconciled against design (D1–D7, INV-1/2/3, ASM-1, E1–E10), plan EX/VT, canon
(PRD-011 / REQ-258 closed allowlist).

- **F-1 (blocker → fix-now, verified).** `just check`/`just gate` was RED at HEAD:
  `tests/e2e_relation_migration_storage.rs::slice_corpus_*` still pinned the SL-058
  table-absent slice shape, contradicting the locked §5.3/E9 re-carry + PHASE-05's
  corpus backfill. The *template* oracle (`assert_slice_shape`) was amended in
  PHASE-03; its *corpus-walk* sibling was missed. Fixed in-audit (`589eb11`,
  swept-partial; completed `32dae47`): renamed
  `slice_corpus_relationships_table_holds_only_dep_seq_keys`, now asserts every
  typed `[relationships]` key ∈ {needs, after} (subsumes the structural-leak guard);
  `assert_f1` + `[[relation]]` allowlist retained. Gate green, clippy zero.
  **AUDIT FLAG #1 disposition: the golden amendment IS design-sanctioned; the real
  defect was the un-amended sibling, now reconciled.**
- **F-2 (minor → tolerated, verified).** Root cause: data-only PHASE-05 landed
  without a full-gate re-run, so the corpus-oracle drift escaped the per-phase
  funnel (code phases verified before the backfill changed corpus contents).
  Data correct; only the unrun gate hid the test-side staleness. Captured as
  [[mem.pattern.testing.data-only-phase-must-regate-corpus-walk-oracles]].
- **AUDIT FLAG #2 (SL-061 cross-agent backfill): CLEAN.** SL-061 carries the correct
  seeded block (both empty arrays, before `[[relation]]`); no foreign WIP swept.
- **Acceptance verified:** INV-2 byte-identity (full root suite green incl. backlog
  needs/after, priority survey/next/blockers/explain, `backlog order` goldens);
  new-behaviour + D2 closed-allowlist refusals + F5 no-read via `e2e_dep_seq_verbs`
  + `e2e_priority_cross_kind`; PHASE-05 — every slice carries `[relationships]`,
  `doctrine validate` corpus clean.
- **Shared-main note (role-reversed hazard):** a concurrent actor's broad `git add`
  swept this audit's uncommitted test fix + partial RV-020 ledger into `589eb11`
  (the glob-sweep hazard, with me as the swept party). Fix landed intact + correctly
  scoped; ledger remainder committed in `32dae47`. The data-only-regate memory is
  recorded + committed but **left UNVERIFIED** — `memory verify` refuses a dirty
  tree and foreign untracked files (`slice/063`, `review/021`) keep it dirty; attest
  later on a clean tree.
