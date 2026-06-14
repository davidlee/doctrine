# SL-057 implementation notes (durable)

Dispatch drive of SL-057 via the dispatch skill (serial, one worker per phase,
orchestrator sole-writer funnel). PHASES 01–05 LANDED on `main` (coordination).
Conclusion (`doctrine slice status 57 audit` → `/audit` → reconcile → `/close`) REMAINS.

## Landed phase chain (coordination commits)
- **PHASE-01** `d005879` — coverage.rs pure VT model: `VtCheck/Matcher/MatchSource/
  RunOutcome`, `derive_status`, `evaluate_matcher`, `valid`/`ValidError`, additive
  `check: Option<VtCheck>` on `CoverageEntry`. **MatchSource serde = TOML string scalar**
  `stdout`/`stderr`/`file:<glob>` (cross-phase byte contract — PHASE-05 goldens pin it).
- **PHASE-02** `8afd0ae` — `dtoml.rs` single doctrine.toml reader (`DoctrineToml{conduct,
  verification}`); `conduct::parse` now `Ok(dtoml::parse(t)?.conduct)` (conduct suite
  byte-green, R2). `verify.rs`: `VerificationConfig`+`timeout_secs()`(300), `resolve(cfg,
  &check)->Resolved{argv,source}|ResolveError`.
- **PHASE-03** `f41d90e` — `coverage_store.rs` impure load/save (`fsutil::write_atomic`);
  `record(root, slice_id, RecordInput{key,status,check,touched_paths}, cfg, today:&str,
  attested_override)` (valid+resolve gate, INJECTED date F-VI, `head_sha` anchor); `forget(
  root, slice_id, &key)->Option<(key,status)>` + pure `withdrawal_line` (F-IV). coverage.rs
  blanket dead_code → per-symbol.
- **PHASE-04** `cb3f71c` — `coverage_verify.rs` impure `run(root, slice_ids:&[u32])->Report`
  (GLOBAL argv dedup, one run/argv, cwd=root, std-only wall-clock timeout→Unobtainable,
  matcher eval incl. canonical-contained File-glob any-match, `derive_status`, re-stamp
  `git_anchor` only on Ran F-VIII, per-slice save). `Report{verified,backfill}` +
  `exit_code_only_count()`/`backfill_count()`. NF-001 e2e guard drives `run()`.

- **PHASE-05** `e3f28c0` — CLI `coverage` subcommand group (D4): `Command::Coverage{
  #[command(subcommand)] CoverageCommand{Show,Record,Verify,Forget}}` (mirror `MemoryCommand`).
  `show` = relocated bare view (`coverage_view::run` unchanged). `record` = args-struct
  `CoverageRecordArgs`/`run_record` → `coverage_store::record` (clock read in shell, injected
  `today` F-VI). `verify <slice>|--all` → `coverage_verify::run` + `print_report` (transition /
  exit-code-only / backfill lines). `forget` → `coverage_store::forget` + `withdrawal_line`.
  Per-verb access classifier (main.rs:1820): Show=Read, Record/Verify/Forget=Write. **DRY cfg
  reader**: `load_config` hosted in `coverage_store` (LOWER module than coverage_verify → no
  cycle, ADR-001), verifier calls down. New `canonical_slice_ref` so `--slice 57` ≡ `--slice
  SL-057` (keying fix). Goldens: bare `coverage` → `coverage show` churn (gate b) + 11 new
  record/verify/forget/show black-box goldens. Removed 9+ now-fulfilled dead_code expects.

**Funnel re-anchor incident (PHASE-05 import)**: worker forked from B=0846800; during the long
worker run a concurrent session merged+closed SL-056 (and a version bump), moving coordination
HEAD to 00cef94 — caught the tree mid-merge (`.git/MERGE_HEAD`, `UU` on slice-056/design.md)
on first funnel attempt → HALTED, did NOT commit. After the merge finished, re-anchored B'=
00cef94. External move touched `src/main.rs` (shared with my delta) — NOT file-disjoint, so a
blind checkout-import would clobber SL-056's main.rs. Imported via `git cherry-pick --no-commit
S` (3-way merge, old-B as base): 7 disjoint files applied trivially, main.rs auto-merged CLEAN
(no conflict). Verified combined tree green (clippy zero, 25 coverage goldens), branch-point
stationary, committed staging only the delta. Lesson: on a moved HEAD mid-funnel, cherry-pick
--no-commit of the single S is the clean 3-way tool (S^==B makes it == net-diff-B..S apply).

**For audit scrutiny (emergent worker decisions, not pre-planned)**: (1) `load_config` RELOCATED
to coverage_store rather than promoted-in-place at coverage_verify — sound (avoids cycle) but
verify it reads identically. (2) `canonical_slice_ref` normalization is NEW behaviour the worker
added on its own initiative (claimed a real keying bug) — confirm it's correct + covered, not
silent scope creep.

All current dead_code `#[expect(not(test))]` annotations across coverage.rs /
coverage_store.rs / coverage_verify.rs / verify.rs / dtoml.rs are ahead-of-consumer;
they SELF-CLEAR when PHASE-05 wires the CLI. PHASE-05 worker must REMOVE the now-unfulfilled
expects (clippy fires "unfulfilled expectation" → delete it).

## PHASE-05 distillation (the remaining phase) — base B = re-capture HEAD at start
CLI `coverage` becomes a SUBCOMMAND GROUP (D4 — clap can't disambiguate bare positional
from subcommand names). Source today: `Command::Coverage { reference, columns, format,
json, path }` (main.rs:220) dispatched at main.rs:2212 → `coverage_view::run(...)`.
- **`coverage show <ref>`** = the relocated CURRENT behaviour (move `reference/columns/
  format/json` under `show`). This is the CONSCIOUS golden churn (gate b): bare
  `coverage <ref>` goldens → `coverage show <ref>`. Update goldens + any skill/doc refs
  by hand (D4).
- **`coverage record`** → `coverage_store::record`. Args-STRUCT handler (R4 clippy
  ceiling): the 4-tuple key (`--slice/--requirement/--change/--mode` or OQ-1 ergonomics),
  the check fields (alias/command/extra-args/matcher source+pattern+regex), `--attested-date`.
  CLI reads `clock::today()` and PASSES it into record (date stays injected — F-VI).
- **`coverage verify <slice> [--all]`** → `coverage_verify::run`; `--all` resolves to ALL
  slice ids (the slice-set), single = `slice::parse_ref`. PRINT the `Report` (per-entry
  `key: old→new`, flag exit-code-only cells, the loud "N VT entries lack a check —
  backfill" line).
- **`coverage forget <key>`** → `coverage_store::forget`; PRINT `withdrawal_line` on a hit.
- **Behaviour-preservation gate (a)** = SL-042/044 read+drift fold suites + the conduct
  suite stay green BYTE-UNCHANGED (no test edits). Gate (b) = the conscious `show` golden
  churn, explicitly NOT part of (a).
- VTs: record black-box goldens (happy + each validity reject: empty-matcher-on-shared-
  base, escaping-glob, bad-regex, both-base); verify/forget surface; relocated show.
- End on `just gate` (--workspace) green + `cargo clippy` ZERO. Tests that MINT doctrine
  entities (e2e/CLI goldens) ⇒ run gate with `DOCTRINE_WORKER` UNSET (mem_019eba2897).

## Dispatch state / gotchas for the resuming orchestrator
- **Funnel cadence per batch**: capture B=HEAD pre-spawn (clean) → worker forks rung-3
  from EXPLICIT B → on return: assert `S^==B` + single non-merge + R-5 (no `.doctrine/`
  touch) → import via `git checkout S -- <files>` (rtk stat-proxies `git diff`,
  mem_019ebf75e2) → `env -u DOCTRINE_WORKER just check` + `cargo clippy` → branch-point
  guard (HEAD still B) → `git commit --no-verify` staging ONLY the delta (NO `-a`) →
  record. PHASE-05 = one serial worker, same funnel.
- **Shared main moves under concurrent SL-056/058 work** (expected, mem_019ebb0f21). If
  HEAD≠B at funnel time, prove the move is file-disjoint from the delta, RE-ANCHOR B to
  the new HEAD, 3-way import (mem_019ebb430f). Happened once (PHASE-01, the skills
  install commit) — disjoint, re-anchored cleanly.
- **Coordination working tree is DIRTY with concurrent WIP — LEAVE IT**: `Cargo.toml` +
  `Cargo.lock` (bumped to 0.3.0), `.doctrine/slice/056/slice-056.toml`,
  `.doctrine/review/016*`. Not ours. Stage only PHASE-05's files; commit without `-a`.
- **Spent worker worktrees** under `.worktrees/sl057-*` — GC at slice conclusion
  (`git worktree remove` + delete fork branches). Removing strands `CARGO_MANIFEST_DIR`-
  baked test binaries → false-RED until recompiled (mem_019ebc8e46).
- Claude Code: spawn workers as PLAIN `Agent` (never `isolation: worktree`) running
  `/worktree mode=worker base=<B>`; pre-distill a self-contained prompt (no governance
  read); mandate `export DOCTRINE_WORKER=1`.

## Conclusion after PHASE-05 lands
`doctrine slice status 57 audit` → `/audit` FROM THE PARENT TREE (RV verbs refuse on a
worktree fork, mem_019eb74153) → reconcile → `/close`. Dogfood the VT machinery on
SL-057's own requirements at /close (design §9): `coverage record` VT checks for
REQ-254/255/256/257/114 then `coverage verify` them green, replacing hand-authored
backfill. Out-of-scope follow-ons (historical backfill, RSK-008 close-gate-on-Failed)
captured for close.
