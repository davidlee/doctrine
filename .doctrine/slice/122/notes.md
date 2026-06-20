# SL-122 — dispatch / implementation notes

Driven via `/dispatch` on the **pi subprocess arm** (config
`[dispatch] claude-force-subprocess-dispatch = true`; `preferred-subprocess-harness = "pi"`),
orchestrator = sole writer, workers = source-delta only.

## Dispatch environment gotchas (apply to the rest of this drive too)

- **Trunk ladder forks the coordination worktree off `origin/HEAD`, not local
  `main`.** `dispatch setup` → `worktree::coordinate` → `git::trunk_commit` →
  `trunk_ladder` prefers `origin/HEAD` (`git.rs:1042`). Here `origin/main` is 6+
  commits behind local `main` and predates SL-122, so the coord tree lacked
  `slice/122/plan.toml` and phase-sheet regen failed ("Plan for slice 122 not
  found"). **Workaround:** prefix every trunk-resolving dispatch command with
  `DOCTRINE_TRUNK_REF=main` (setup, and `dispatch sync --prepare-review` at
  conclude). Candidate ISS: setup should prefer local `main` over stale
  `origin/HEAD`, or fail with a clearer message.
- **`just gate` masks Rust test failures behind `lint-js`.** Recipe order is
  `fmt lint lint-js test-all build`; `lint-js` aborts on missing `@eslint/js`
  (jail node_modules gap) before `test-all` runs. Verify Rust directly:
  `just lint && just test-all && just build`.
- **Pre-existing rustfmt skew at `src/relation.rs:1051`** — the jail rustfmt
  collapses a multi-line `RelationLabel::Related` tuple that `main` carries
  multi-line. `cargo fmt --check` flags only this file; it is NOT in any SL-122
  delta and was excluded from the PHASE-01 commit.

## PHASE-01 — RFC kind foundation (done)

- Code: `06afaad0`; funnel boundary record: `9d8bd307`. Base B2 = `9c6d649d`.
- RFC registered as a Kind-is-data kind via the existing `GovKind`/`entity::Kind`
  scaffold — **no engine abstraction** (VA-2 clean; worker confirmed no
  resistance). `rfc new` mints status=open; `rfc show` uses the status-bearing
  read path (mirrors adr, not rec).
- **Plan Affected-Surface gap (finding for audit):** a new command module must be
  registered in `.doctrine/adr/001/layering.toml` or
  `tests/architecture_layering.rs` completeness assertion fails. The plan did not
  enumerate this surface. Orchestrator added `rfc = "command"` (authored .doctrine
  write — workers cannot touch it; the worker garbled an uncommitted attempt).
- **Verification gaps (finding for audit):** worker satisfied VT-2 (RFC
  validate/reseat) via the existing generic `kinds_table_covers_*` assertions and
  VT-4 (corpus-scan no-panic) by code-inspection of the routed `outbound_for`
  arm rather than a dedicated minted-RFC scan test. VT-1/VT-5 have dedicated
  tests. Audit should confirm VT-2/VT-4 coverage is adequate or add tests.

## PHASE-02 — RFC lifecycle transitions + catalog list (done)

- Code: funnel commit `8d6da66f`; boundary record `18bf45b5`. Base B = `9d8bd307`
  (PHASE-01 boundary). Worker delta = `28a1014f` on `worker/SL-122/PHASE-02`.
- **Scope bleed (finding for audit):** the PHASE-02 delta is +227 lines of
  `src/rfc.rs` that are **ALL TESTS** — no new impl. The lifecycle machine
  (`RfcStatus`, `RFC_STATUSES`, `set_status`, `rfc status`, `rfc list
  --status/--all`) was already pre-shipped inside **PHASE-01's** commit `06afaad0`
  (its worker built the full `RfcCommand{New,List,Show,Status}` enum + handlers,
  beyond its mint/show charter). Functionally complete + now tested + green, but
  the P01↔P02 boundary bled. Audit should reconcile the boundary record against
  what each phase actually delivered.
- **Verify gotcha (orchestrator self-inflicted, resolved):** ran verify with
  `DOCTRINE_TRUNK_REF=main` prefixed onto `just test-all` → 141 failures, all
  `DOCTRINE_TRUNK_REF=main does not resolve to a commit`. The env var leaks into
  every test that spins up its own temp git repo (no `main` ref there). The env
  prefix is for **trunk-resolving dispatch cmds only** (`setup`/`sync`), NOT for
  the verify suite. Plain `just lint && just test-all && just build` is green in
  the markerless coord tree — and the 2 `run_new` tests the worker flagged also
  pass here (markerless coord tree resolves trunk natively).

## PHASE-03 — Relations: RFC own edges + REV→RFC precursor (done)

- Code: funnel commit `fe6d0861`; boundary record `65acca6a`. Base B = `18bf45b5`
  (PHASE-02 boundary). Worker delta = `c705062f` on `worker/SL-122/PHASE-03`.
- Genuine remainder confirmed by pre-spawn recon (coord tree, not main — main
  lacks the PHASE-01 RFC code): all of EX-1..5 were unimplemented; only the
  `outbound_for` RFC arm pre-existed as the `Ok(vec![])` stub (scan.rs:75). No
  PHASE-01 over-reach into PHASE-03 scope (unlike the P01→P02 bleed).
- Delivered (5 src files): RFC added to `related` AnyNumbered sources; `outbound_for`
  RFC arm filled via `governance::relation_edges(&rfc::RFC_KIND, …)`; new
  `originates_from` rule (REV→RFC, Tier::Typed, TypedVerbOnly, inbound "precursor
  of") + `OriginatesFrom` RelationLabel variant (name/from_name arms);
  `revision new --originates-from RFC-NNN` authors a single provenance `[[relation]]`
  row (NOT a [[change]] payload); generic `link … originates_from` refused; derived
  "precursor of" inbound on `inspect RFC`. `relation_graph.rs` label-count comment
  16→17.
- **Verify (funnel):** green in coord tree — 2544 passed, 0 failed. The worker
  flagged 3 `e2e_adr_cli_golden` failures in its fork ("worker fork (signal:
  both): refusing authored write") — that is **worker-marker confinement** blocking
  authored `.doctrine` writes, a DIFFERENT env class from the run_new missing-trunk
  one. Both classes clear in the markerless coord tree at funnel verify. Expected;
  no action.

## Pre-existing fix folded into the dispatch base

- `9c6d649d fix(IMP-122)` (committed on `main`, = dispatch base B2): IMP-122
  carried an illegal `[[relation]] related → SL-121` edge (`related` sources are
  ADR/POL/SL/STD, not backlog kinds), failing
  `e2e_relation_migration_storage::backlog_corpus_keeps_dep_seq_typed_migrates_cross_kind_axes`
  on `main`. Relabeled to the `slices` cross-kind axis (user-approved). Orthogonal
  to SL-122; surfaced because it blocked every phase's green gate.
