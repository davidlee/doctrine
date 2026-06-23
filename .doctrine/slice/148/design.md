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
        │   scan source)    │  GitRef ─────┼─ push refs/doctrine/reservation/<stem>/<NNN>
        ▼                   └──────────────┘    under zero-oid CAS, then mkdir
   claim_fresh_id loop:
     id = next_id(scan(), trunk_ids)          scan() = local_dirs (LocalFs)
     claim.claim(ctx)                                  local_dirs ∪ remote_reservation_ids (GitRef)
       Won        → build + write_fileset
       AlreadyHeld→ re-fetch + recompute + retry  (exhaust → bail w/ reseat hint)
```

`reservation list` is a sibling read path: fetch refs → `for_each_ref` → render.

### 5.2 Interfaces & Contracts

**Claim seam (enriched — D1).** `&Path` → a descriptor carrying the ref identity:

```rust
pub(crate) struct ClaimCtx<'a> {
    pub dir: &'a Path,      // entity dir (LocalFs claims this)
    pub root: &'a Path,     // repo root (GitRef runs git here)
    pub stem: &'a str,      // KINDS stem — the reservation ref segment ("slice")
    pub id: u32,            // the candidate id
}
pub(crate) trait Claim {
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired>;
}
```

- `LocalFs::claim` = `create_dir(ctx.dir)` — behaviour-identical to today.
- `GitRef::claim` = build `refs/doctrine/reservation/{stem}/{id:03}` → empty-tree
  commit (`commit_tree`) → `push_ref_cas(remote, ref, new, ZERO_OID)`. `Updated`
  ⇒ also `create_dir(ctx.dir)` (local mkdir; same-machine exclusion + keeps the
  loop's H2 cleanup valid) ⇒ `Won`. `Moved` ⇒ `AlreadyHeld`. A push/network error
  propagates (not swallowed as `AlreadyHeld`).

**New `git.rs` remote ops** (thin, shell-out, mirror existing helpers):

```rust
pub(crate) fn fetch_refspec(root, remote, refspec) -> Result<(), CaptureError>;
// push <new>:<ref> with --force-with-lease=<ref>:<expected_old>; classify
// rejection (CAS mismatch) distinctly from a transport error.
pub(crate) fn push_ref_cas(root, remote, refname, new_oid, expected_old)
    -> Result<RefCas, CaptureError>;          // reuses the RefCas enum
pub(crate) fn for_each_ref(root, pattern)      // (refname, oid, author, date, msg)
    -> Result<Vec<RefRow>, CaptureError>;
```

**Backend selection blast radius (F-3).** Shared reach applies to *any* numbered
kind (PRD-005), so **every Fresh-allocating materialise site** (slice / spec ×2 /
adr / requirement / backlog ×5 / knowledge / concept_map) swaps its literal
`&LocalFs` for a single resolved-backend helper `reserve::backend(root)?`.
`InExisting` sub-entity sites (design, phases) take no claim and are untouched.
This caller swap — not the seam signature — is the bulk of the change and drives
phasing (seam + LocalFs parity first, then GitRef behind the helper, then the
default flip).

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
performs the reachability fetch, and decides degradation. Reservation-ref ids
fetched here seed the per-retry scan source.

**Survey.** `doctrine reservation list [--kind <stem>] [--remote <name>]` →
fetch `refs/doctrine/reservation/*` → table `{canonical, holder, acquired}`.

### 5.3 Data, State & Ownership

- **Reservation ref** `refs/doctrine/reservation/<stem>/<NNN>` → empty-tree
  commit (the well-known empty tree). **No blobs** (REQ-024). Holder =
  `$DOCTRINE_AGENT_ID` else git committer identity. The commit is built with
  `GIT_AUTHOR_NAME/EMAIL` + `GIT_COMMITTER_NAME/EMAIL` set **explicitly** from
  the resolved holder (F-2) — never relying on ambient `git config user.*`, so a
  human with unset git identity does not fail. Acquired = committer date.
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
   `auto` + fetch fails / no remote ⇒ fall back to `LocalFs` + one-time signal.
   `shared` + fetch fails ⇒ hard error.
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

- **I1** At most one holder per id: guaranteed by zero-oid `--force-with-lease`
  at the remote — exactly one creating push lands; rivals' CAS expectation
  (absent) no longer holds and they are rejected.
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
  platforms (zero-oid create rejection semantics). Confirm in PHASE-01 against
  the local-bare-repo substrate.
- **OQ-4** — agent identity beyond `$DOCTRINE_AGENT_ID` + git committer (session
  id?) — minimal for v1; revisit with leasing (IDE-021).

## 7. Decisions, Rationale & Alternatives

- **D1 — enrich the claim seam to `ClaimCtx`, keep one loop.** Smallest change
  that gives `GitRef` the ref identity while preserving `LocalFs` exactly.
  Rejected: derive-ref-from-`&Path` (fragile, fs-coupled); a second trait
  (breaks the one-loop principle).
- **D2 — claim linearizes at the remote via `push --force-with-lease=<ref>:0`.**
  The only shape meeting PRD-005's single-linearization invariant for cross-clone
  reach. Rejected: survey-only / push-race (fails REQ-021); local-ref-now-push-
  later (no real guarantee until follow-up).
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
- **D7 — reuse `integrity::KINDS` stem as the ref segment.** One id table; no
  second mapping to drift (numbered-kind-identity-table memory).

## 8. Risks & Mitigations

- **R1 — distributed CAS correctness is the load-bearing risk.** Mitigation:
  test the lost-race→re-fetch→retry path against a local bare repo with two
  clones racing the same id, not just the happy path (VT, §9). I1 proof.
- **R2 — first remote surface (auth, transport, partial failure).** Mitigation:
  all remote ops behind `git.rs` helpers with a mock seam; classify CAS-rejection
  vs transport error distinctly (a transport error must NOT read as `AlreadyHeld`
  → would corrupt the retry). `shared` hard-fails on transport error.
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
  cross-reference from SPEC-022. Tracked for /reconcile, not a blocker.
- **R4 — default-flip breaks stdout-asserting suites.** Mitigation: D5 isolates
  the flip; signal is stderr-only + one-time; final phase sweeps the suite.
- **R5 — jail prevents network e2e.** Mitigation: local-bare-repo substrate
  covers the mechanism; network e2e is a manual dev affordance (jail-relax
  follow-up), never a CI dependency.

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

**Open for external/inquisition pass:** R1 (CAS correctness proof depth), R2
(transport-vs-CAS error classification), E5 (is uniform-reach an acceptable v1
posture, or must shared reach union branch heads?).
