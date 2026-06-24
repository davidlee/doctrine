# Notes SL-148: Git-ref reservation backend

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Stage

**PHASE-01 (behaviour gate) COMPLETE — T1–T4 done, green.** Seam enriched + named
path split; zero observable behaviour change. Design revised (D9) + plan tightened;
external adversarial pass (codex/GPT-5.5) integrated F-7..F-13 (design §10); D8
governance call taken. Next: PHASE-02.

**Code-verification pass (this session, commits `341809e8` design / `18e1e637` plan).**
Read the seam, git.rs primitives, config idiom, every call-site. 8/8 handover Qs
resolved (§10 "Code-verification pass" disposition). Findings F-V1..F-V6:
- **F-V2 → D9 (design change, user-approved).** Shared `Claim` trait forced the named
  (memory) path to fabricate `id`/`stem`. Resolution: split the named path off `Claim`
  (inline `fs::create_dir`-or-bail); `Claim`/`ClaimCtx`/`reserve::backend` are now
  Fresh-numeric-only. **Supersedes SL-005 D7** (the named+numeric unification in
  `mem.system.engine.identity-claim-seam` §2) → memory update at /reconcile (R8).
  Checked against `scratch/memory-contract.local.md`: memory's remote future
  (the external decision register HTTP, server-side idempotency) is a **separate storage seam** at the
  `materialise_named` write body — D9 *enables* it, doesn't foreclose it (OQ-6).
- **F-V1 → 11 Fresh sites**, not ~10: the design omitted the `materialise_fresh_prebuilt`
  family (review/rec×2/revision). EN-1/EX-2 corrected.
- **F-V3 → `ClaimCtx{dir,id}`** (stem/root/remote/holder backend-captured).
- **F-V4/5/6 → PHASE-03 detail:** env-aware commit helper + empty-tree oid net-new;
  `$DOCTRINE_AGENT_ID` resolution net-new; the re-fetching scan source must be injected
  through `materialise` (so `ReservedIds` is a closure, not a static `Vec`).
- **R9:** new `reserve` module needs an ADR-001 `layering.toml` entry (PHASE-01 EX-6).
- **Confirmed sound:** `next_id` sig + scan closure (F-6); `RefCas`→Won/AlreadyHeld;
  `run_git` raw `Output` → separable streams for `--porcelain` (R2 viable);
  `install::prompt_confirm`/`tty` reuse for D8.

## Context-building map (read order for a fresh reviewer / planner / designer)

1. **`slice-148.md`** — scope, in/out, A1–A3 assumptions, R1–R3 risks,
   verification intent. The three governance cuts (leasing OUT → IDE-021; git-ref
   *content* storage OUT; local backend unchanged) are the load-bearing scope
   boundaries.
2. **`design.md`** — the design. §7 Decisions (D1–D7) is the fastest orientation;
   §10 Review Notes lists the internal adversarial findings (F-1..F-6) and what's
   still open for a hostile pass.
3. **Governing intent** (read via `doctrine spec show`): **PRD-005**
   (Reservation & Leasing) — esp. §2 scope (leasing explicitly deferred), the
   invariants (single-linearization-point), §8 open questions (OQ-2 probe cost,
   OQ-3 ref volume). **SPEC-008** (Id lifecycle) § Trunk-aware fork safety + D1/D2
   — the shipped half this extends. **SPEC-022** (git interaction model) — the
   ref-taxonomy/CAS posture the new remote ops must match.
4. **Reference**: lazyspec **RFC-035** (`/workspace/lazyspec/docs/rfcs/`) — the
   parity source. Doctrine adopts only its *reservation-over-git-ref* half;
   diverges deliberately (no `.git/config` mutation; metadata-as-data not
   `lease.json`; leasing split out). **Working prior art** (MIT) digested in
   `scratch/lazyspec.git.research.md` — `engine/git_ref.rs` (GitRefOps trait +
   `GitCli` + `MockGitRefClient`), `engine/lease.rs`, `engine/agent.rs`. It ships
   the exact zero-oid `--force-with-lease=ref:0` create-CAS, which is what
   de-risked codex's first blocker (OQ-3/D2).
5. **Code surface** (current state): `src/entity.rs` (`Claim`/`LocalFs`,
   `claim_fresh_id` loop ~L372, `next_id` ~L203, `materialise*`); `src/git.rs`
   (has `update_ref_cas`/`RefCas`/`ZERO_OID`, `commit_tree`, `resolve_ref`,
   `select_remote`; **lacks** push/fetch/`--force-with-lease`); `src/dtoml.rs`
   (shared config reader) + a consumer module pattern (`dispatch_config.rs`,
   `conduct.rs` `ConductConfig`); `integrity::KINDS` (stem table).

## What the planner needs to know (phasing drivers)

- **Behaviour gate is the PHASE-01 boundary.** The `ClaimCtx` seam enrichment (D1)
  + the ~10 Fresh call-site swap to a `reserve::backend(root)?` helper must land
  with `LocalFs` behaviour-identical and the full existing suite green **before**
  any GitRef code. That swap (not the seam signature) is the bulk of the change
  (F-3) — touches slice/spec×2/adr/requirement/backlog×5/knowledge/concept_map.
- **GitRef + new git.rs remote ops** (`fetch_refspec`, `push_ref_cas`,
  `for_each_ref`) behind a mock seam — second phase. CAS-rejection vs transport-
  error classification is load-bearing (R2): a transport error must NOT read as
  `AlreadyHeld`. **Classification is `git push --porcelain`-based** (F-9), not
  stderr parsing — only the explicit lease/create-CAS rejection retries; auth/hook/
  namespace-policy → hard error. **Crib mechanics from lazyspec** (research file):
  push **by oid** of a **dangling** commit (no local `update-ref` pre-push — avoids
  phantom-create, F-7/I4); glob-fetch `+refs/.../*:refs/.../*`; `read_commit_timestamp`
  via `cat-file -p` committer line; FIFO-queue mock (`MockGitRefClient`) for the seam.
- **Survey verb** (`reservation list`) — can parallel or follow GitRef.
- **Default flip `local`→`auto`** — the FINAL, isolated, gated phase (D5). It is
  behaviour-gate-sensitive (stdout must stay byte-identical; signal is stderr-only
  + one-time). Keep it reversible and last.
- Test substrate: a `bare-remote` helper (`git init --bare` temp + two clones) —
  build it early; it underwrites every cross-clone VT and is jail-safe.

## Open questions still live (don't let them get lost)

- **E5/R6 (RESOLVED by external pass):** uniform reach IS a sound v1 posture —
  under uniform `shared`/`auto`, every authored id reserves a ref at author time, so
  branch-only-authored ids are already covered; the gap is purely a `local`/mixed
  phenomenon. `shared`-unions-branch-heads rejected as unnecessary. F-8 closed the
  separate `auto`-internal transient hole (D8). Mixed-reach remains the documented
  A3/E5 limit + `validate`/`reseat` backstop.
- **OQ-3 (design):** exact `--force-with-lease` create-flag form (`:<zero>` vs
  `:`) — confirm against the bare-repo substrate in the GitRef phase.
- **PRD-005 OQ-2/OQ-3:** auto-probe round-trip amortisation; permanent-ref volume
  — both deferred, no v1 work, but cite them if they resurface.

## Follow-ups captured (not stranded)

- **IDE-021** — lease-based edit-exclusion coordination (the deferred RFC-035
  half). Needs its own PRD/spec before slicing.
- **Spec reconcile (R7):** at `/reconcile`, add SPEC-008 prose for the remote
  reservation ref class + `git.rs` remote ops, and a SPEC-022 cross-ref. No
  conflict; the prose just needs to record the widened ref surface. **Plus a
  PRD-005 §6 note** (D8): `auto` fail-closes on a configured-remote transient
  failure — stricter than PRD's literal "fall back + signal" — operator opts into
  fallback explicitly. Records the deliberate tightening.
- **Jail relaxation:** dev-only, for network e2e against a real remote. Not a CI
  dependency (bare-repo substrate covers the mechanism).

## Durable facts worth a memory (harvest at wrap-up, not yet)

- **Jail blocks agent `git push`** — env gotcha; test distributed git via a local
  bare repo, not a network remote. (Record at wrap-up.)
- **`git.rs` was local-ref-only before this slice** — SL-148 introduces doctrine's
  first remote git mutation (`push`/`fetch`); future remote-coordination work
  builds on `push_ref_cas`/`fetch_refspec`. (Record once the seam lands.)

## Commits (design stage)

- `b6fc75b6` slice scope · `33f5c8cd` design v1 · `626f8118` internal adversarial
  pass integrated + slice reconcile. All `.doctrine` committed promptly; no
  pending authored changes; no code yet.

## Implementation (PHASE-01)

- **T1** `284538cc` — `reserve::backend(root,prefix)` seam + 11 Fresh call-site swap;
  6 unused `LocalFs` imports dropped; `reserve=engine` in layering.toml. 2426/0.
- **T2** — `ClaimCtx{dir,id}` + `Claim::claim(&ClaimCtx)`; `LocalFs` reads `ctx.dir`
  (`id` carried for PHASE-03 GitRef ref-segment, `#[expect(dead_code)]`). **D9 split:**
  `claim_and_write_named` dropped the `Claim` param, inlines `fs::create_dir`-or-bail —
  dup-bail msg + H2 `remove_dir_all` byte-identical (VA-1). `materialise_named`,
  `memory.rs` ×2, 5 named tests, `LocalFs` import all updated.
- Gate: entity 27/0, main bin suite 2426/0 (= baseline, zero new failures, VT-1),
  clippy clean, `architecture_layering_gate` green. The 3 `e2e_relation_migration_storage`
  failures are concurrent SL-143/backlog-163 dirty corpus, NOT this slice.
- Durable seam fact for wrap-up harvest: **the numeric claim seam now carries
  `ClaimCtx{dir,id}`; the named path is OFF the trait** (inline mkdir). PHASE-03's
  GitRef backend reads `ctx.id` as the ref segment under `refs/doctrine/reservation/<prefix>/<id>`.

## Implementation (PHASE-02..05 — landed in impl bundle review/148)

All four phases landed as one squashed src-only commit (`0c11259c`, 18 files,
+1860/−63; "0 phase cuts" — knowledge-commit skipped, boundaries.toml gitignored).
Runtime phase markers flipped planned→completed at close (audit is the evidence).
- **PHASE-02** — `git.rs` remote ops: `fetch_refspec`, `push_ref_cas`
  (`--porcelain` CAS-vs-transport classification — only lease/create rejection →
  `Moved`), `for_each_ref`; bare-remote substrate (`git init --bare` + N clones).
- **PHASE-03** — `GitRef` backend (dangling empty-tree commit, push-by-oid ZERO_OID
  CAS, explicit holder identity), `[reservation]` config + `resolve_backend` (sole
  reach selector / reachability probe / D8 fail-closed degradation), re-fetching
  scan source injected through `materialise`. Ships `reach=local`.
- **PHASE-04** — `doctrine reservation list` survey (`{canonical, holder,
  acquired}`; `acquired` documented client-declared best-effort; `--kind`/`--remote`).
- **PHASE-05** — default flip `local`→`auto` (D5). Carried `git::resolve_remote`'s
  non-repo→`Ok(None)` short-circuit — required so default-auto degrades to `LocalFs`
  in the bare-TempDir unit suite instead of hard-erroring (latent PHASE-03 gap
  surfaced by auto-as-default).

## Audit + reconcile + close (RV-152, REV-010)

- **Audit RV-152** — 6 findings, all terminal. Integration gate (bundle cherry-picked
  onto current `main`, clean): **2874 passed / 0 failed**, clippy zero-warning, fmt
  clean. Adversarial pass over the CAS/classification path: no defects;
  collision-freedom (I1) proven by `vt1_two_clones_racing`. VH-1 (ship auto-default)
  accepted by the User 2026-06-24. The inherited `e2e_memory_sync` failure was
  confirmed SL-143/base-staleness (green at integration), not SL-148.
- **Reconcile REV-010** (done) + per-slice direct edits: design.md §5.2 as-built note
  (3 layering-driven deviations — F-1); SPEC-008/SPEC-022/PRD-005 §6 prose;
  `mem.system.engine.identity-claim-seam` §2 (D9 supersedes SL-005 D7); ADR-001
  layering comment. See review-152.md `## Reconciliation Outcome`.
- **Build-substrate gotcha (confirmed, already canon):** auditing in a fresh
  `git worktree` fails to compile until the gitignored `web/map/dist/` RustEmbed
  folder is populated — matches `mem.pattern.dispatch.worker-fork-missing-gitignored-embed`.
- **Integration (stage-2, operator-driven):** the code bundle still needs
  `dispatch sync --slice 148 --integrate --trunk refs/heads/main` after finishing
  the edge→main promotion (re-check `dispatch status --slice 148` for drift first).
