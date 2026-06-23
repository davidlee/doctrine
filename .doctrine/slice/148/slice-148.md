# Git-ref reservation backend

## Context

Identity allocation today (SPEC-008, the shipped half) is local-only: the `Claim`
seam in `src/entity.rs` has exactly one backend, `LocalFs`, whose `mkdir` is the
atomic claim, and every materialisation site (`backlog.rs`, `governance.rs`,
`knowledge.rs`, `concept_map.rs`, `memory.rs`) passes `&LocalFs`. Fork safety is
achieved separately, by unioning the trunk's id listing into the candidate scan
(`max(local ∪ trunk) + 1`, read once via `git ls-tree`). The *only* cross-machine
coordination is therefore the **git push race**: two agents in separate clones can
each locally `mkdir` the same number and commit; the remote rejects the second
push as non-fast-forward. There is no pre-author reservation that reaches across
clones — the remote is the conflict-resolution point, not the claim mechanism.

PRD-005 and SPEC-008 already specify the fix and call it **deferred**: a
permanent-claim-over-a-shared-backend reach, selectable by configuration, that
makes a reserved identity unique across every clone of a shared remote — without
changing what callers ask for (REQ-021), plus a survey of held claims (REQ-022).
The `entity.rs` comment naming "the future `git-ref` backend" marks the drop-in
seam. This slice implements that backend.

Reference: lazyspec RFC-035 (`/workspace/lazyspec/docs/rfcs/`). It bundles three
capabilities; doctrine governs them apart (see Non-Goals) — only its
**reservation-over-git-ref** half maps here.

Governing intent: **PRD-005** (Reservation & Leasing), **SPEC-008** (Id lifecycle,
§ Trunk-aware fork safety, D1/D2), requirements **REQ-021** (reach by config) and
**REQ-022** (survey held claims). Constrained by **ADR-006** (worktree posture,
D3 — local degradation) and **SPEC-022** (git interaction model).

## Scope & Objectives

What changes, and why: add a second `Claim` backend that reserves identities over
git refs so a claim reaches every clone of a shared remote, closing the
single-tree blind spot that today only the push race catches.

In scope:

- **A git-ref `Claim` backend** alongside `LocalFs`, implementing the existing
  `Claim` seam (`Won` / `AlreadyHeld`), so the materialise loop and the
  `max + 1` retry interpretation (SPEC-008 D1) are reused unchanged. The claim is
  a ref under a reserved namespace (e.g. `refs/doctrine/claims/<kind>/<id>`),
  created with compare-and-swap against the remote so exactly one contender wins
  (PRD-005 invariant: the accepted claim is the single linearization point).
- **Reach selection by configuration** — local vs shared, transparent to callers
  (REQ-021). An auto mode uses the shared backend when the remote is reachable and
  falls back to single-tree reach with a **one-time visible signal** that
  cross-team reach is off (PRD-005 §6, success measure 3); never a silent
  downgrade.
- **Survey of held claims** (REQ-022) — a read that lists held identities under a
  namespace with holder and acquisition time, across the active store.
- **Claim references a name only, never content** (REQ-024 invariant) — the ref
  records the reservation, never the entity's bytes; entities stay working-tree
  files (PRD-005 principle).
- Wiring the backend selection through the materialisation call sites without
  changing their request shape (`MaterialiseRequest` unchanged).

Affected surface (concrete, to be confirmed in `/design`):

- `src/entity.rs` — the `Claim` seam and its callers; the new backend likely lives
  beside it or in a new `src/` module (cf. the `git.rs` pattern for the impure git
  shell). Pure/imperative split: ref I/O is impure-shell, behind a trait mockable
  in tests (mirror the existing git seam).
- `src/git.rs` / git-interaction layer (SPEC-022) — ref CAS, fetch, push, list,
  read commit timestamp. Reuse, do not fork, the existing git plumbing
  (`ScratchIndex`, hash-object/mktree/commit-tree per the dispatch tooling memory).
- Configuration surface — where reach is selected (`.doctrine/doctrine.toml`, per
  SL-146; confirm in design).
- A CLI verb for the survey (REQ-022) — shape TBD in `/design`.
- `integrity::KINDS` is the id table the allocator already iterates; confirm the
  backend reuses it rather than re-deriving (numbered-kind-identity-table memory).

## Non-Goals

- **Lease-based edit-exclusion** — TTL, heartbeat, release, force-acquire, crash
  recovery, clock-skew handling, and per-kind write-gating. PRD-005 §2 explicitly
  defers this as a separate capability "specified elsewhere"; no such spec exists
  yet. It is RFC-035's coordination half and needs its own PRD/tech spec before any
  slice. Out of scope here — see Follow-Ups.
- **Git-ref document storage** — storing entity content in refs (RFC-035 stores
  iterations there). Conflicts with PRD-005's principle that entities remain
  ordinary working-tree files; doctrine puts only *claims* in refs, never content.
  Not in scope, and not planned without a governance Revision.
- **Changing the local backend or the trunk-union algorithm** — `LocalFs` and
  `max(local ∪ trunk) + 1` stay as-is; this adds a sibling backend, it does not
  replace the shipped half.
- **Renumber/repair** (`validate` / `reseat`) — unchanged.

## Summary

Implement the specified-but-deferred git-ref reservation backend: a second `Claim`
backend reserving identities over remote-reachable git refs, reach selected by
config with visible local fallback, plus a survey of held claims — realising
REQ-021/REQ-022 under PRD-005/SPEC-008, leaving leasing and git-ref content
storage out by governance.

### Risks & assumptions

- **R1 — distributed CAS correctness.** The cross-clone uniqueness guarantee rests
  on atomic compare-and-swap at the remote (`push --force-with-lease` against an
  all-zeros expected ref for creation). Getting the linearization point right, and
  proving it under contention, is the load-bearing risk. Verification must exercise
  a lost-race → recompute → retry path, not just the happy path.
- **R2 — git-interaction reuse.** Must ride the existing git seam (SPEC-022) and
  not fork a parallel git layer (no-parallel-implementation). Confirm the seam
  exposes (or cheaply gains) ref CAS / fetch / push / list / timestamp ops.
- **A1 — assumes a `git push` remote is the coordination substrate.** No daemon, no
  central lock (PRD-005 constraint). Single-tree reach remains correct offline.
- **A2 — behaviour-preservation gate.** Existing entity-engine suites must stay
  green unchanged; numeric callers' observable behaviour is the proof
  (identity-claim-seam memory invariant).
- **A3 — reach is a team-wide agreement** (design E5/F-1). The candidate set does
  not cover ids authored under `local` reach on an unmerged branch, so a team
  mixing `local` and `shared` clones can still collide. A repo's reach is assumed
  uniform across clones; mixing is unsupported and documented, not defended in
  code (`validate`/`reseat` stay the cross-fork backstop).
- **R3 — jail blocks network push.** The cross-clone guarantee is proven against a
  local bare repo; no test depends on a network remote (jail-relax is a dev-only
  follow-up).

### Open questions (for `/design`)

- OQ-1 — ref namespace layout and what the claim commit/blob carries (holder +
  acquired-at for the survey, REQ-022) while still being content-free per REQ-024.
- OQ-2 — config shape for reach selection (local | shared | auto) and where it
  lives; interaction with the existing trunk-union path.
- OQ-3 — fetch refspec / setup requirements for custom refs (RFC-035 § Fetch
  Refspecs analog) and how `auto` probes remote reachability without a costly
  round-trip per reservation (PRD-005 OQ-2).
- OQ-4 — the survey CLI verb shape and output.

### Verification / closure intent

- Collision-freedom under contention proven at the backend boundary: a claim that
  loses the atomic race drives recompute-and-retry to the next free id; no two
  holders ever come away with the same number (REQ-020 reused, REQ-021).
- Reach behaviour proven: `auto` resolves to shared when the remote is reachable,
  falls back to single-tree with the reduced reach surfaced; an explicit choice
  overrides selection (REQ-021).
- Coordination-only boundary proven: a claim references an entity by name, holds no
  content, never appears in the entity's record (REQ-024).
- Survey proven: held claims report holder and acquisition time (REQ-022).
- Behaviour-preservation: full existing suite green, unchanged.
- Closure tracks coverage against the durable REQ entities, never membership labels.

## Follow-Ups

- **Lease-based coordination** (RFC-035 coordination half) — author the deferred
  "specified elsewhere" capability: a PRD/tech spec for transient edit-exclusion
  leases, then slice(s). Tracked as IDE-021.
- **Spec reconcile (design R7)** — at /reconcile, add a prose note to SPEC-008
  (the remote reservation ref class `refs/doctrine/reservation/*` + the new remote
  git ops in `git.rs`) and a cross-reference from SPEC-022 (git interaction model
  ref taxonomy). No conflict — PRD-005/SPEC-008 ratify the reach — but the spec
  prose should record the widened ref surface.
- **Jail relaxation for network e2e** — relax the bubblewrap jail's git-push block
  to dev/test the backend against a real network remote (e.g. GitHub). Dev-only;
  not a CI dependency.
