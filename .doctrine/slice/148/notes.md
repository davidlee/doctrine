# Notes SL-148: Git-ref reservation backend

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Stage

**Design internally + externally reviewed; awaiting user lock approval → `/plan`.**
No code yet. `just check` N/A (nothing modified). External adversarial pass (codex
/ GPT-5.5) integrated as F-7..F-13 (design §10); all three originally-open items
(R1/R2/E5) closed. One governance call taken by the User: D8 — `auto` fail-closes
on a configured-remote failure, operator opt-in fallback (`y/N` prompt /
`DOCTRINE_RESERVATION_FALLBACK=1`). On lock: `doctrine slice status 148 plan`.

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
