# Design SL-148: Git-ref reservation backend

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Make a freshly allocated entity identity trustworthy across **every clone of a
shared remote**, not just one working tree. Today the only cross-machine
arbitration is the git push race at merge time; two agents in separate clones
each locally `mkdir` the same number and both believe they own it (the collision
surfaces late, as corrupted history). Implement the specified-but-deferred
shared-backend reservation reach (PRD-005, SPEC-008 § Trunk-aware fork safety):
a second `Claim` backend that linearizes at the remote, reach selectable by
config, plus a survey of held claims — realising REQ-021 and REQ-022, leaving
leasing (IDE-021) and git-ref content storage out by governance.

## 2. Current State

- **One claim backend.** `entity.rs::Claim { fn claim(&self, claim: &Path) }`;
  `LocalFs` = `fs::create_dir` (the mkdir *is* the atomic claim). Hardcoded
  `&LocalFs` at ~12 materialise sites (slice/spec/adr/requirement/backlog/
  knowledge/concept_map). Only `claim_fresh_id` calls `.claim()`.
- **Allocation** (`claim_fresh_id`): `next_id(scan(), trunk_ids)` =
  `max(local_dirs ∪ trunk_ids) + 1`; `trunk_ids` read once at the shell edge
  (`git::trunk_entity_ids` → local `ls-tree` of the trunk tree-ish), held
  constant; the local `scan` re-reads each retry to recover a lost race. Bounded
  128-retry loop; exhaustion is a hard `bail!`.
- **Git is local-only.** `git.rs` exposes `update_ref_cas` (`git update-ref <ref>
  <new> <old>`, the 3-arg CAS), `resolve_ref`, `tree_with_file`, `commit_tree`,
  `merge_base`, `ScratchIndex`, `ZERO_OID`, remote *selection* (`select_remote`,
  `doctrine.repo.preferredremote`). **No `push`, no `fetch`, no
  `--force-with-lease`.** All existing coordination (dispatch, trunk-union) is
  single-machine local-ref.
- **Config** is one shared `dtoml::parse(doctrine.toml)`; each consumer projects
  its section (`.dispatch`, `.conduct`, `.estimation`, …). No `[reservation]`.
- **Repair** exists: `reseat` renumbers an entity's canonical-id quad to the next
  free id (SPEC-008 § reseat), keyed on the canonical ref.

## 3. Forces & Constraints

- **PRD-005 invariant — single linearization point.** "An identity is held only
  once the claim is accepted, never on optimistic local state alone." For
  cross-clone reach the accept MUST be the remote's, so this slice introduces
  doctrine's first **remote** git mutation.
- **PRD-005 — coordination-only, content-free.** A claim references a name, never
  holds the entity's bytes (REQ-024); entities stay ordinary working-tree files.
- **Behaviour-preservation gate** (identity-claim-seam memory). The numeric
  callers' suites are the proof; `LocalFs` behaviour and observable CLI output
  stay unchanged. Signatures may change mechanically.
- **POL-002 / back-compat.** A repo with no `[reservation]` and no remote behaves
  bit-for-bit as today.
- **Jail blocks network push** (env). No test may depend on a network remote;
  the cross-clone guarantee is proven against a **local bare repo** used as the
  remote.
- **ADR-006 D3** — local-only degradation is silent reach loss, not incorrectness;
  but the loss must be *made visible* (PRD-005 measure 3).
- **SPEC-022** — git interaction model: ref CAS, zero-oid creation, report-not-
  clobber on moved targets. The reservation push must follow the same posture.
- **No parallel implementation** — ride `git.rs`, `integrity::KINDS`, `dtoml`.

## 4. Guiding Principles

- One materialise loop, one claim seam — generalise only as far as the second
  backend forces (identity-claim-seam memory).
- The remote is the arbiter; local state is a cache that never advances past what
  the remote accepted (SPEC-022 linearization posture).
- Default-safe: the capability ships off; turning it on is a deliberate config
  act, and the default-flip to `auto` is its own gated step.
- Degrade loudly, fail honestly: `auto` falls back to local with a one-time
  signal; `shared` hard-fails. Never a silent cross-team downgrade.
- Reuse the user's git config; never mutate `.git/config`.

## 5. Proposed Design

### 5.1 System Model

```
 slice/spec/adr/... run_new (shell)                      git.rs (impure)
   resolve reach (doctrine.toml [reservation])           ┌─ fetch_refspec ──┐
   resolve backend ──────────────┐                       │  push_ref_cas    │ NEW
   trunk_ids = trunk_entity_ids   │                       │  for_each_ref    │
        │                         ▼                       └──────────────────┘
        ▼                  ┌──────────────┐ claim(ctx)         ▲
   entity::materialise ───▶│ dyn Claim    │────────────────────┘
        │  (+ reserved_ids  │  LocalFs     │  mkdir
        │   scan source)    │  GitRef ─────┼─ push refs/doctrine/reservation/<prefix>/<NNN>
        ▼                   └──────────────┘    under zero-oid CAS, then mkdir
   claim_fresh_id loop:
     id = next_id(scan(), trunk_ids)          scan() = local_dirs (LocalFs)
     claim.claim(ctx)                                  local_dirs ∪ remote_reservation_ids (GitRef)
       Won        → build + write_fileset
       AlreadyHeld→ re-fetch + recompute + retry  (exhaust → bail w/ reseat hint)
```

`reservation list` is a sibling read path: fetch refs → `for_each_ref` → render.

### 5.2 Interfaces & Contracts

**Claim seam (enriched — D1; Fresh-numeric-only — D9).** `&Path` → a minimal
descriptor. The seam now serves the **Fresh numbered path only** (D9 splits the
named/memory path off it, below). Only `id` varies per retry, so it alone rides
`claim()`; the per-kind/per-repo constants (`prefix`, repo `root`, `remote`,
`holder`) are **captured by the backend at construction** (`reserve::backend`),
not threaded through the ctx — keeping `ClaimCtx` to the two values already live
at the claim site (`entity.rs` `claim_fresh_id`):

```rust
pub(crate) struct ClaimCtx<'a> {
    pub dir: &'a Path,      // entity dir — LocalFs/GitRef mkdir this
    pub id:  u32,           // candidate id — GitRef's ref segment; recomputed each retry
}
pub(crate) trait Claim {
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired>;
}
```

- `LocalFs::claim` = `create_dir(ctx.dir)` — behaviour-identical to today; ignores `id`.
- `GitRef { root, prefix, remote, holder }` (captured in `reserve::backend`).
  `GitRef::claim` = build `refs/doctrine/reservation/{self.prefix}/{ctx.id:03}` →
  empty-tree commit (`commit_tree`, **dangling** — no local `update-ref` before the
  push; pushed **by oid** so a failed push never advances a local ref past the
  remote, reinforcing I4; lazyspec prior-art pattern) →
  `push_ref_cas(remote, ref, new, ZERO_OID)`. `Updated`
  ⇒ also `create_dir(ctx.dir)` (local mkdir; same-machine exclusion + keeps the
  loop's H2 cleanup valid) ⇒ `Won`. `Moved` ⇒ `AlreadyHeld`. A push/network error
  propagates (not swallowed as `AlreadyHeld`).

**Named path off the seam (D9).** `materialise_named` (memory's only caller, ×2)
no longer takes `&dyn Claim`; its directory claim is an inline
`fs::create_dir(entity_dir)` → `Ok` / `AlreadyExists` ⇒ `bail!("… already exists")`,
with the existing H2 won-dir cleanup retained verbatim. Memory's uid is a
client-minted v7 UUID (minted-once-stored, never numbered, never reserved
cross-clone — SPEC-008 D5), so it has no `id`/`prefix` to give a `ClaimCtx` and
never selects `GitRef`. The seam is therefore honestly Fresh-numeric-only;
memory's *remote* future is a **separate storage seam** at this same
`materialise_named` write body (see §6, OQ-6). This supersedes the SL-005 D7
named+numeric claim-seam unification (`mem.system.engine.identity-claim-seam` §2 —
update at /reconcile, R8).

**New `git.rs` remote ops** (thin, shell-out, mirror existing helpers):

```rust
pub(crate) fn fetch_refspec(root, remote, refspec) -> Result<(), CaptureError>;
// Push the *oid* (new_oid:<ref>), not a local ref, with
// --force-with-lease=<ref>:<expected_old>. Classify via `git push --porcelain`
// (F-9/F-10): ONLY the explicit lease/create-CAS rejection ⇒ Moved/AlreadyHeld;
// `remote rejected`/auth/hook/namespace-policy (e.g. host forbids refs/doctrine/*)
// ⇒ HARD error surfacing the remote reason — never AlreadyHeld, never silent retry.
pub(crate) fn push_ref_cas(root, remote, refname, new_oid, expected_old)
    -> Result<RefCas, CaptureError>;          // reuses the RefCas enum
pub(crate) fn for_each_ref(root, pattern)      // (refname, oid, author, date, msg)
    -> Result<Vec<RefRow>, CaptureError>;
```

**Backend selection blast radius (F-3, corrected F-V1).** Shared reach applies to
*any* numbered kind (PRD-005), so **every Fresh-allocating materialise site** swaps
its literal `&LocalFs` for the resolved-backend helper `reserve::backend(root, prefix)?`.
Code-verification (F-V1) corrected the count to **11 production sites in two
families** — the original enumeration omitted the entire `materialise_fresh_prebuilt`
family:
- via `entity::materialise(.., Fresh)` (7): `slice`, `spec`, `governance`/adr,
  `requirement`, `backlog`, `knowledge`, `concept_map`.
- via `entity::materialise_fresh_prebuilt` (4): **`review` (RV), `rec` ×2,
  `revision` (REV)** — also `claim_fresh_id → .claim()`, also numbered, also in scope.

`InExisting` sub-entity sites (design, phases) take no claim and are untouched. The
**named** sites (memory ×2) are *not* "untouched" — they drop the `&LocalFs`
argument entirely (D9). This caller swap — not the seam signature — is the bulk of
the change and drives phasing (seam + LocalFs parity first, then GitRef behind the
helper, then the default flip).

`reserve` is a **new module** (engine tier — reaches `git`/`entity`/`dtoml`, no
command module): it needs an ADR-001 `layering.toml` classification entry or
`just gate`'s `MixedUmbrella` assertion goes red
(`mem.pattern.lint.module-split-needs-layering-entry`) — a PHASE-01 exit criterion.

**Reach config.** New section, parsed via the shared reader, consumed by a
`reservation_config` module (never eagerly validated in `dtoml::parse` —
dtoml-shared-reader memory):

```toml
[reservation]
reach  = "local"   # local | shared | auto   (v1 ships local; flips to auto, D5)
remote = "origin"  # optional; else select_remote / preferredremote
```

```rust
enum Reach { Local, Shared, Auto }
fn resolve_backend(root, cfg) -> anyhow::Result<(Box<dyn Claim>, ReservedIds)>;
```

`resolve_backend` is the **sole** site that selects `LocalFs` vs `GitRef`,
performs the reachability fetch, and decides degradation (contract: D8 — `auto`
fail-closes on a configured-remote failure, with an explicit operator opt-in to
local fallback). Reservation-ref ids fetched here seed the per-retry scan source.

**Survey.** `doctrine reservation list [--kind <prefix>] [--remote <name>]` →
fetch `refs/doctrine/reservation/*` → table `{canonical, holder, acquired}`.

### 5.3 Data, State & Ownership

- **Reservation ref** `refs/doctrine/reservation/<prefix>/<NNN>` (`<prefix>` =
  the canonical id-space prefix, e.g. `SL`/`ASM`/`IMP` — F-V7, **not** the shared
  file-stem) → empty-tree
  commit (the well-known empty tree). **No blobs** (REQ-024). Holder =
  `$DOCTRINE_AGENT_ID` else git committer identity. The commit is built with
  `GIT_AUTHOR_NAME/EMAIL` + `GIT_COMMITTER_NAME/EMAIL` set **explicitly** from
  the resolved holder (F-2) — never relying on ambient `git config user.*`, so a
  human with unset git identity does not fail. Acquired = committer date —
  **best-effort client-declared metadata** (`GIT_COMMITTER_DATE` is client-set); the
  survey (REQ-022) reports it as the holder's declared time, not a server clock (F-12).
  Message = canonical ref (`SL-148`). Permanent — never deleted, never reissued;
  one ref per reserved id (no GC, PRD-005 OQ-3 open).
- **Scan source (GitRef, F-6).** The per-retry `scan()` closure fetches
  `+refs/doctrine/reservation/*:refs/doctrine/reservation/*`, then reads the
  fetched **local** refs (`for_each_ref`) ∪ local dirs → ids. It re-fetches each
  iteration (the iter-1 fetch overlaps `resolve_backend`'s reachability probe — a
  harmless double-fetch; retries are rare). `next_id`'s signature is unchanged —
  the remote union rides the `local` argument.
- **Refspec** passed explicitly per fetch/push
  (`+refs/doctrine/reservation/*:refs/doctrine/reservation/*`). **`.git/config`
  is never mutated** (diverges from RFC-035; no setup step).
- **Candidate set:** `next_id(scan(), trunk_ids)` unchanged in shape. For GitRef,
  `scan()` returns `local_dirs ∪ remote_reservation_ids`; `trunk_ids` stays the
  constant trunk-union input. On `AlreadyHeld`, the loop re-fetches and the next
  `scan()` reflects the rival's new reservation.
- **Degradation signal:** one-time per-process stderr note on `auto`→local
  fallback. Never stdout (protects byte-identical CLI output / the behaviour gate).

### 5.4 Lifecycle, Operations & Dynamics

Reserve (shared/auto, reachable remote):
1. `resolve_backend`: fetch reservation refs (this *is* the reachability probe).
   Degradation contract (D8):
   - `auto` + **no remote configured** (structurally single-tree) ⇒ `LocalFs` +
     one-time stderr signal (the genuine PRD-005 §6 fallback case).
   - `auto` + **configured remote that fails** (transient/auth/transport) ⇒
     **hard error by default** (fail-closed — a silent transient downgrade would
     mint a colliding local id, B2). Operator opts into local fallback per
     allocation via an interactive `y/N` prompt (TTY) or, non-interactively,
     `DOCTRINE_RESERVATION_FALLBACK=1` (config `[reservation] allow_local_fallback`);
     on accept ⇒ `LocalFs` + one-time stderr signal. Prompt/signal are stderr-only
     (behaviour gate).
   - `shared` + fetch fails ⇒ hard error; no fallback, no prompt (shared means shared).
2. `next_id(local ∪ remote_reservation_ids, trunk_ids)`.
3. `GitRef::claim`: empty-tree commit → `push_ref_cas(..., ZERO_OID)`.
   - `Updated` ⇒ local mkdir ⇒ `Won` ⇒ scaffold + `write_fileset` (H2 cleanup
     unchanged: a build failure `remove_dir_all`s the dir; the **ref stays** — a
     reserved-but-unauthored id is a harmless permanent gap, PRD-005).
   - `Moved` ⇒ `AlreadyHeld` ⇒ re-fetch, recompute, retry.
4. Exhaust 128 retries ⇒ `bail!` whose message prints the remediation:
   `doctrine reseat <canonical> [--to <id>]` (D6).

Survey: fetch → `for_each_ref` → render holder/acquired per held id.

Degraded (local / no remote): identical to today's path; `trunk_ids` union still
gives offline fork-safety at single-tree+trunk reach.

### 5.5 Invariants, Assumptions & Edge Cases

- **I1** At most one holder per id: guaranteed by `--force-with-lease=<ref>:<zero>`,
  whose lease receive-pack checks **against the remote's current ref state**, not a
  local remote-tracking ref (F-11) — stale local fetch state cannot satisfy it.
  Exactly one creating push finds the ref absent and lands; thereafter every rival's
  zero-oid expectation no longer holds and receive-pack rejects them. Rivals build
  *different* commit oids (identity/date differ), but the CAS guards the ref **name**,
  not object identity, so only one create lands regardless (F-13). Proven in
  production by lazyspec's lease engine (same primitive).
- **I2** A claim never holds entity content (empty tree) and never appears in the
  entity's working-tree record (REQ-024).
- **I3** `LocalFs` path and all existing suites are observably unchanged (gate).
- **I4** Local never advances past the remote: the local mkdir happens only after
  the remote push is `Updated`.
- **E1** Push won but local mkdir fails (id dir already exists locally, foreign):
  treat as hard error with the reseat hint — do not orphan a remote reservation
  silently. (Rare; flagged for review — see R3.)
- **E2** No trunk + no remote = defined terminus: empty unions, first id, local
  reach (SPEC-008 D2). 
- **E3** Pre-existing remote reservation refs created out-of-band: tolerated —
  they widen the candidate set; unparposeable ref names under the namespace are
  ignored, not fatal.
- **E4** Ref existence checks use `for-each-ref`/`rev-parse --verify`, never
  `cat-file -e` (rtk false-positive memory).
- **E5 — reach is a team-wide agreement (F-1), an assumption, not a guarantee.**
  The candidate set is `local_dirs ∪ remote_reservation_ids ∪ trunk_ids`. It does
  **not** cover an id authored under `local` reach on an unmerged branch (no
  reservation ref, not on trunk). So a team mixing `local` and `shared` clones can
  still collide. Supported posture: a repo's reach is uniform across its
  clones (all `shared`/`auto`, or all `local`). Mixing is unsupported and
  documented, not defended in code. Single-tree collision (separate clones, no
  shared backend) remains the visible accepted limit (PRD-005 §6).

## 6. Open Questions & Unknowns

- **OQ-1 (PRD-005 OQ-2)** — `auto` costs one fetch per reservation. Caching /
  amortising the reachability+ref view is deferred; reservations are infrequent.
- **OQ-2 (PRD-005 OQ-3)** — permanent ref accumulation at very large volumes;
  no GC in v1.
- **OQ-3** — `--force-with-lease` portability across git versions / hosted
  platforms (zero-oid create rejection semantics). **De-risked** by lazyspec prior
  art (ships the form); still confirm in PHASE-01 against the local-bare-repo
  substrate.
- **OQ-4** — agent identity beyond `$DOCTRINE_AGENT_ID` + git committer (session
  id?) — minimal for v1; revisit with leasing (IDE-021).
- **OQ-5 (F-10)** — should `shared`/`auto` run a one-time *push-capability* probe
  when first selecting `GitRef` (a host may allow fetch but forbid creating
  `refs/doctrine/*`)? v1 relies on the first create-push's porcelain classification
  to hard-error with the remote reason; an up-front probe is a latency-vs-early-
  failure trade left open.
- **OQ-6 (memory remote storage — out of scope, recorded so D9 isn't re-litigated)**
  — memory wants a *remote storage/coordination* backend in future
  (`scratch/memory-contract.local.md`: forgettable / `forgetd`, an HTTP+JSON
  append-only event log; idempotency is **server-side** via a deterministic
  `event_id` + `409 duplicate_event`=success, not a reservation CAS). That is a
  **separate seam** from this slice's reservation `Claim` — it slots into the
  `materialise_named` *write body* (`fs::create_dir` + `write_fileset` → a memory
  storage backend `FsMarkdown | ForgettableHttp`), not into the numbered-id claim
  loop. D9 (splitting the named path off `Claim`) **enables** this cleanly — it does
  not foreclose it; forcing memory onto the reservation seam would misfit
  HTTP-append-with-server-idempotency onto mkdir-or-`AlreadyHeld`. No v1 work.

## 7. Decisions, Rationale & Alternatives

- **D1 — enrich the claim seam to `ClaimCtx`, keep one loop.** Smallest change
  that gives `GitRef` the ref identity while preserving `LocalFs` exactly.
  Rejected: derive-ref-from-`&Path` (fragile, fs-coupled); a second trait
  (breaks the one-loop principle). **(D9 refines):** `ClaimCtx` carries only
  `{dir, id}` — `prefix`/`root`/`remote`/`holder` are backend-captured at
  construction (only `id` varies per retry), and the seam is Fresh-numeric-only.
- **D2 — claim linearizes at the remote via `push --force-with-lease=<ref>:0`.**
  The only shape meeting PRD-005's single-linearization invariant for cross-clone
  reach. The explicit lease `<ref>:<zero-oid>` is compared by receive-pack against
  the **remote** ref, so correctness does not depend on a fresh remote-tracking ref
  (F-11). Validated by lazyspec prior art (MIT; `engine/git_ref.rs`
  `push_ref_with_lease`, `expected_old=ZERO_SHA`) — the zero-oid create form ships
  and works, de-risking OQ-3 from a blocker to a substrate confirmation. Rejected:
  survey-only / push-race (fails REQ-021); local-ref-now-push-later (no real
  guarantee until follow-up).
- **D3 — empty-tree commit, metadata-as-data.** Most content-free reading of
  REQ-024; holder/acquired are exactly git commit fields. Rejected: a
  `reservation.toml` blob (reintroduces a content object).
- **D4 — explicit per-command refspec, never mutate `.git/config`.** No setup
  step, user git config untouched. Diverges from RFC-035 deliberately.
- **D5 — ship default `local`; flip default to `auto` in the final gated phase.**
  Delivers OOTB team coordination (user intent) while de-risking the behaviour-
  gate-sensitive default-flip into its own reversible step.
- **D6 — failure prints the reseat remediation command.** Retry exhaustion /
  detected collision is actionable: cite `doctrine reseat <canonical>` (SPEC-008
  repair verb), never a bare "could not reserve".
- **D7 — reuse `integrity::KINDS` `prefix` as the ref segment (corrected F-V7).**
  One id table; no second mapping to drift (numbered-kind-identity-table memory).
  **The segment must key the id-space**, and `prefix` (`SL`/`ASM`/`IMP`…) is the
  canonical, enforced-unique per-kind identity — `refs/doctrine/reservation/SL/148`
  mirrors `SL-148`. The originally-written `stem` is the **file-stem**
  (`record-NNN.toml`), deliberately *shared* across sub-kinds (knowledge ×4 →
  `record`, backlog ×5 → `backlog`), so it is many-to-one with the id-space and
  would collapse independent namespaces (ASM-001 and DEC-001 both → `record/001`).
  `prefix` is also already in scope at all 11 sites (the prebuilt callers already
  pass it). Rejected: `stem` (wrong granularity); `dir` (correct but slashy/long).
- **D8 — `auto` fail-closes on a configured-remote failure; local fallback is an
  explicit operator opt-in (B2).** PRD-005 §6's "fall back + one-time signal" governs
  only the **structurally single-tree** case (no remote configured). A configured
  remote that fails transiently under `auto` is a hard error, because a silent
  transient downgrade would mint a local id that collides with another clone's
  accepted remote reservation — violating I-ONEHOLDER under *uniform* `auto` config,
  not just mixed reach (E5). The operator accepts reduced reach per allocation via an
  interactive `y/N` prompt (TTY) or `DOCTRINE_RESERVATION_FALLBACK=1` (non-interactive);
  both routes emit the one-time stderr signal — satisfying PRD-005's "made visible,
  never silently assumed." Stricter than PRD-005's literal wording; a one-line PRD-005
  reconcile note records the tightening (R7). Rejected: PRD-literal
  degrade-on-any-unreachability (the B2 hole).
- **D9 — split the named (memory) path off the `Claim` seam; the seam is
  Fresh-numeric-only (supersedes SL-005 D7).** Code-verification (F-V2) found the
  shared seam forces the named path to construct a `ClaimCtx` with no numeric
  `id`/`prefix`. SL-005 D7 deliberately *unified* named+numeric on one generic claim
  seam (`mem.system.engine.identity-claim-seam` §2: "Reservation is one caller's
  interpretation of the generic claim"). But SL-148's enrichment makes the ctx
  numeric-shaped (`id` is a reservation concept), so the named caller no longer fits
  a "generic" seam. Resolution: `materialise_named` drops `&dyn Claim` and inlines
  its mkdir-or-bail; `Claim`/`ClaimCtx`/`reserve::backend` serve only numbered
  allocation. Faithful to §4's "generalise only as far as the second backend
  forces" — GitRef forces generality on the *numbered* path only. Loses zero test
  capability (no mock-`Claim` is ever injected into the named path — only Fresh
  uses `AlwaysHeld`). Memory's remote future is a distinct storage seam (OQ-6).
  Rejected: sentinel `id:0`/`prefix:""` (a numeric field on a non-numeric path);
  `id: Option<u32>` (a `.expect()` that re-encodes the very invariant the type
  can't prove). Requires a /reconcile update to the SL-005 memory (R8).

## 8. Risks & Mitigations

- **R1 — distributed CAS correctness is the load-bearing risk.** Mitigation:
  test the lost-race→re-fetch→retry path against a local bare repo with two
  clones racing the same id, not just the happy path (VT, §9). I1 proof.
- **R2 — first remote surface (auth, transport, partial failure).** Mitigation:
  all remote ops behind `git.rs` helpers with a mock seam; classification is
  **machine-stable via `git push --porcelain`** (F-9), not English-stderr parsing —
  ONLY the explicit lease/create-CAS rejection maps to `AlreadyHeld`; `remote
  rejected`/auth/hook/namespace-policy failures (incl. a host forbidding
  `refs/doctrine/*`, F-10) are hard errors that surface the remote reason, never a
  silent 128-retry. `shared` hard-fails on transport error; `auto` fail-closes on a
  configured-remote failure (D8).
- **R3 — E1 (remote-won / local-mkdir-failed) split state. RESOLVED.** Leaving
  the remote reservation ref is correct: PRD-005's "an abandoned reservation is a
  harmless gap, not a fault to recover from" governs. The orphaned ref is a
  permanent gap; the operator gets a hard error + reseat hint and picks another
  id. No rollback of the remote ref (rollback would itself need a second remote
  round-trip and contradict permanence).
- **R6 — mixed-reach unsoundness (E5/F-1).** Mitigation: documented team-wide
  uniform-reach assumption; `validate`/`reseat` remain the cross-fork collision
  backstop (SPEC-008) when a mixed-reach or pre-merge race lands a collision
  anyway. Out of scope to defend in code this slice.
- **R7 — SPEC-022 ref-taxonomy extension (F-4).** `refs/doctrine/reservation/*`
  and the new remote git ops widen git's ref surface SPEC-022 enumerates (it
  currently scopes coordination/evidence refs as local). No conflict — PRD-005 /
  SPEC-008 ratify the reservation reach — but reconciliation must add a prose note
  to SPEC-008 (the remote reservation ref class + remote ops in `git.rs`) and a
  cross-reference from SPEC-022. Tracked for /reconcile, not a blocker. **(D8
  addendum):** also add a PRD-005 §6 reconcile note that `auto` fail-closes on a
  configured-remote transient failure (stricter than the literal "fall back +
  signal"), the operator opting into fallback explicitly.
- **R4 — default-flip breaks stdout-asserting suites.** Mitigation: D5 isolates
  the flip; signal is stderr-only + one-time; final phase sweeps the suite.
- **R5 — jail prevents network e2e.** Mitigation: local-bare-repo substrate
  covers the mechanism; network e2e is a manual dev affordance (jail-relax
  follow-up), never a CI dependency.
- **R8 — D9 supersedes a recorded SL-005 decision.** D9 reverses the SL-005 D7
  named+numeric claim-seam unification documented in
  `mem.system.engine.identity-claim-seam` §2. Mitigation: at /reconcile, update
  that memory's §2 to record the seam is now Fresh-numeric-only and why (the
  enrichment made the ctx numeric-shaped); note memory's directory claim is now an
  inline mkdir, not a `Claim` backend. Tracked for /reconcile, not a blocker.
- **R9 — new `reserve` module needs an ADR-001 layering entry.** A new `src` unit
  under the layering gate requires a `layering.toml` classification or `just gate`'s
  `MixedUmbrella` assertion goes red (recurring design omission,
  `mem.pattern.lint.module-split-needs-layering-entry`). Mitigation: PHASE-01 exit
  criterion; `reserve` is engine tier (reaches `git`/`entity`/`dtoml`, no command
  module). Regenerate authoritatively, don't hand-guess.

## 9. Quality Engineering & Validation

Behaviour gate first: full existing suite green, unchanged, after the seam
enrichment (D1) — proven before any GitRef code (PHASE-01 boundary).

New coverage (REQ-cited, mode-discriminated):
- **VT — collision-freedom under contention (REQ-020/021, I1):** two clones of a
  local bare repo compute the same candidate; exactly one push lands, the loser
  re-fetches, recomputes, and lands the next id. No duplicate.
- **VT — reach selection (REQ-021):** `local` never touches the remote; `shared`
  uses it and hard-fails when absent; `auto` uses it when reachable and falls
  back to local + one-time signal when not. Explicit choice overrides.
- **VT — content-free claim (REQ-024, I2):** the reservation ref's commit tree is
  empty; the entity record carries no coordination bytes.
- **VT — survey (REQ-022):** `reservation list` reports holder + acquired per held
  id, kind-filterable.
- **VT — failure remediation (D6):** retry-exhaustion error text contains the
  `doctrine reseat <canonical>` command.
- **VT — degradation (ADR-006 D3):** no-remote `auto` resolves to local with
  byte-identical stdout and a single stderr signal.
- Unit: `push_ref_cas` CAS-rejection vs transport-error classification (R2);
  `next_id` over `local ∪ remote_reservation_ids`.

Substrate: a `bare-remote` test helper (`git init --bare` temp + two clones),
extending existing git test fixtures — no network, jail-safe (R5).

## 10. Review Notes

**Internal adversarial pass (self) — integrated:**
- **F-1 → E5/R6** — mixed-reach is unsound; the candidate set misses branch-only
  authored ids. Stated as a team-wide uniform-reach assumption; `validate`/`reseat`
  backstop. Integrated §5.5, §8.
- **F-2 → §5.3** — holder must not depend on ambient `git config user.*`; set
  `GIT_AUTHOR_*`/`GIT_COMMITTER_*` explicitly from the resolved holder.
- **F-3 → §5.2** — real blast radius is the ~10 Fresh materialise call sites
  swapping `&LocalFs` → `reserve::backend(root)?`, not the seam signature. Drives
  phasing.
- **F-4 → R7** — reservation refs + remote ops extend SPEC-022's ref taxonomy;
  reconcile-time spec note required (not a blocker).
- **F-5 → R3** — split-state (remote-won/local-failed) resolved by PRD-005's
  harmless-gap principle; no remote rollback.
- **F-6 → §5.3** — GitRef `scan()` reads fetched local reservation refs, re-fetched
  per iteration; `next_id` signature unchanged.

**Deferred to plan-level detail:** exact `--force-with-lease` create flag form
(`:<zero>` vs `:`) confirmed against the local-bare-repo substrate (OQ-3);
empty-tree oid via the well-known constant or `mktree`.

**Code-verification pass (source-read, pre-plan-lock) — disposition:**
- **F-V1 → §5.2/blast radius.** Original ~10-by-kind enumeration omitted the
  `materialise_fresh_prebuilt` family — true count is **11 production Fresh sites**
  (+`review`, `rec`×2, `revision`). Corrected.
- **F-V2 → D9.** Shared seam forces the named path to fabricate `id`/`prefix`. Split
  the named path off `Claim` (supersedes SL-005 D7). See D9, R8.
- **F-V3 → D1/§5.2.** `ClaimCtx.root` (and `prefix`) are redundant — backend-captured
  at construction; ctx reduced to `{dir, id}`. Folded.
- **F-V7 → D7/§5.2/§5.3 (caught at PHASE-01 execution).** The ref segment was keyed
  on `integrity::KINDS` `stem`, but `stem` is the **file-stem** and is *shared* across
  sub-kinds (knowledge ×4 → `record`, backlog ×5 → `backlog`, spec ×2), each of which
  has its **own tree + id-space** ("Own tree + reservation namespace" per KINDS). So
  `<stem>` collapses independent id-spaces. Corrected to `prefix` (canonical,
  enforced-unique per kind, already in scope at all 11 sites). `reserve::backend(root,
  prefix)`; ref = `refs/doctrine/reservation/{prefix}/{id:03}`.
- **F-V4 → PHASE-03 detail.** `commit_tree` routes through `run_git` with **empty
  env**; the `GIT_AUTHOR_*`/`GIT_COMMITTER_*` (F-2) commit needs a small env-aware
  helper over the existing private `run_git_env` seam (`git.rs`). Empty-tree oid has
  no existing constant (only unrelated `ZERO_OID`) → net-new (`4b825dc…` or
  `mktree`). Confirms OQ-3 scope; no design change.
- **F-V5 → §5.3.** No `$DOCTRINE_AGENT_ID` resolution exists in `src` today — holder
  resolution is fully net-new (as F-2 already implies). Confirmed, no change.
- **F-V6 → §5.3/plan.** The per-retry scan closure is built *inside* `materialise`
  (hardcoded `scan_ids`); GitRef's re-fetch means the scan **source** must be
  injected from `reserve::backend` through `materialise`/`materialise_fresh_prebuilt`
  into `claim_fresh_id`'s `scan` param — so `ReservedIds` must be a *re-fetching
  closure*, not a static `Vec` (a snapshot would miss a rival's post-`AlreadyHeld`
  ref). Behaviour already in §5.3 F-6; the signature-change wiring is a /plan +
  phase-plan concern.
- **Confirmed sound (no change):** `next_id(local,trunk)` signature + re-invoked
  `scan` closure (F-6); `RefCas::{Updated,Moved}` → `Won`/`AlreadyHeld` (Q4);
  `run_git`→raw `Output` gives separable stdout/stderr/exit-code for `--porcelain`
  (R2 viable, Q5); config lazy-projection idiom + `install::prompt_confirm`/`tty`
  reuse for the D8 prompt (Q7).

**External adversarial pass (codex / GPT-5.5) — disposition (R1/R2/E5 closed):**
- **F-7 (codex B1) → OQ-3/D2** — zero-oid `--force-with-lease=ref:0` is *not* an
  unverified spell: lazyspec ships it (MIT prior art). Downgraded from blocker;
  bare-repo confirmation stays in PHASE-01. Adopted lazyspec's push-by-oid +
  dangling-commit pattern (§5.2, reinforces I4).
- **F-8 (codex B2) → D8/§5.4** — `auto` could self-collide: a transient fetch
  failure under *uniform* `auto` silently mints a colliding local id. Accepted;
  `auto` now fail-closes on a configured-remote failure, operator opt-in fallback via
  `y/N` prompt or `DOCTRINE_RESERVATION_FALLBACK=1` (user decision). PRD-005 reconcile
  note added (R7).
- **F-9 (codex M1) → §5.2/R2** — `push --porcelain` machine-stable classification;
  only explicit lease/create-CAS rejection ⇒ `AlreadyHeld`. Accepted.
- **F-10 (codex M2) → R2/OQ-5** — fetch-reachable ≠ push-capable; policy/namespace
  rejection is a hard error surfacing the remote reason; up-front capability probe
  left as OQ-5. Accepted.
- **F-11 (codex m1) → I1/D2** — proof rewritten: the explicit lease compares the
  *remote* ref via receive-pack; stale local-tracking refs irrelevant. Accepted.
- **F-12 (codex m2) → §5.3** — `acquired` documented as best-effort client-declared
  metadata (forgeable `GIT_COMMITTER_DATE`). Accepted.
- **F-13 (codex m3) → D3** — confirmed content-free per REQ-024: empty tree, no
  blobs, differing commit oids irrelevant (CAS guards the ref name). No change.

**Resolution of the three originally-open items:**
- **R1 (CAS proof depth)** — closed by F-7/F-11 + lazyspec prior art: the zero-oid
  explicit lease is a remote-checked create-CAS; exactly one creating push lands.
- **R2 (classification)** — closed by F-9/F-10: porcelain-based, only the CAS
  rejection retries; every other failure hard-errors.
- **E5 (uniform reach)** — under *uniform* `shared`/`auto`, every authored id reserves
  a ref at author time, so `remote_reservation_ids` already covers branch-only-authored
  ids; the E5 gap is purely a `local`/mixed-reach phenomenon and uniform reach (A3) is
  a sound v1 posture. F-8 additionally closes the `auto`-internal transient hole.
  codex's "shared unions branch heads" alternative is unnecessary under uniform reach
  and rejected for v1. The mixed-reach collision remains the documented A3/E5 limit,
  backstopped by `validate`/`reseat`.
