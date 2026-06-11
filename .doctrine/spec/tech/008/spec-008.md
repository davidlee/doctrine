# SPEC-008: Id lifecycle

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Id lifecycle is the container that governs a numbered entity's id from allocation
to integrity repair: how a fresh `SL-031` / `REQ-150` / `ADR-009` is chosen and
claimed without collision, how the corpus is scanned for id-integrity violations,
and how a collided or misnumbered entity is renumbered. It sits beneath the
whole-system root (SPEC-003) and rides the shared entity engine (SPEC-004) for the
atomic claim primitive and edit-preserving writes; it restates none of that. What
it owns is everything specific to the *id*: the next-free allocation algorithm
that interprets the engine's claim seam, the trunk-aware fork-safety that algorithm
folds in, the corpus-wide `KINDS` table a generic scan keys on, and the
detect-versus-repair split (`validate` then `reseat`) that is the ADR-006 D3
posture for fork-safe ids.

Ids are **per-namespace**: `SL-001`, `ADR-001`, and `REQ-001` coexist legitimately
because each numbered kind is its own namespace, so every allocation scan and every
integrity rule is *intra-kind*. Memory is out of this lifecycle entirely — it is a
named UUID entity with no reserved number (its identity reasons live in SPEC-007),
so it neither allocates here nor is scanned by `validate`.

## Responsibilities

Mirrors the structured `responsibilities` list: own next-free allocation; make it
fork-safe via trunk-id union; hold the corpus-wide `KINDS` table; provide `validate`
(detect); provide `reseat` (repair); and exclude memory's named identity.

### Next-free allocation

A numbered id is chosen by one backend-agnostic algorithm: scan the namespace for
the ids present, take `max + 1` (or `001` when empty), and atomically claim that
candidate's directory. Gaps are never backfilled — `max + 1` ignores a hole left by
an abandoned reservation, which is a harmless gap, not a fault. The claim is the
engine's `Claim` seam (SPEC-004): a `Won` outcome means the dir is this caller's and
scaffolding proceeds; an `AlreadyHeld` outcome means a sibling won the race between
the scan and the claim, so the loop recomputes the candidate and retries. The loop
is bounded (a fixed retry ceiling); exhausting it is a hard error, never a silent
duplicate id. *Reservation* — this `max + 1` arbitration — is one caller's
interpretation of the generic claim primitive owned by the parent engine; this
container owns the arbitration, not the primitive.

A `Won` claim that then fails to scaffold removes its own directory, so a partial
write never survives as a ghost entity occupying an id (the parent engine's
materialise loop owns that cleanup; the allocation contract here is that a returned
id is always a fully-materialised entity).

### Trunk-aware fork safety

The local working tree alone cannot see an id a sibling fork has already authored
but not yet merged, so a naive local `max + 1` would re-mint a colliding id across
forks. Allocation closes this by unioning the **trunk's** id listing into the
candidate scan: `max(local ∪ trunk) + 1`. The trunk ids are read once at the
imperative shell edge (a `git ls-tree` of the kind's directory on the resolved
trunk tree-ish) and held constant across retries — only the local scan re-reads to
recover from a lost race. A repo with no trunk (fresh, no remote, detached) is a
defined terminus that yields an empty trunk listing and degrades to local-only
allocation, correct and lock-free on one tree but without cross-fork reach
(ADR-006 D3). This is the shipped half of the reservation primitive; the
permanent-claim-over-a-shared-backend generalisation (`git-ref`, leasing) is a
specified-but-deferred extension of the same algorithm.

### The corpus-wide `KINDS` table

A generic id scan needs three facts the per-kind engine `Kind` descriptor does not
carry together: the canonical `prefix`, the tree `dir`, the metadata-file `stem`
(`slice-007.toml`), and the gitignored runtime phase-state dir a kind owns (`Some`
only for slice today, which `reseat` refuses to strand). `integrity::KINDS` is the
single table that travels that quad for every numbered kind in canonical order. It
is a deliberate drift surface: a new numbered kind absent from this table silently
escapes `validate`, a cost this container accepts in exchange for not threading a
registry through every kind-owning module.

### `validate` — the detect half

`validate` is a pure check layer (facts in, findings out, no disk — the
pure/imperative split) over an impure scan that reads each kind's namespace into a
snapshot. The check enforces three intra-kind rules per numbered kind:

- **(a) dir/toml agreement** — a directory's basename id equals the id its sister
  toml declares.
- **(b) no intra-kind duplicate** — no two directories of one kind declare the same
  id.
- **(c) alias target equality** — every `NNN-slug` alias symlink targets the
  directory whose toml id equals the alias's encoded id (target *equality*, not mere
  resolvability — a link that resolves to the wrong-numbered dir is a finding).

A malformed metadata toml is a hard error propagated out of the scan, kept distinct
from an integrity finding: `validate` reports inconsistency, it does not paper over
corruption. A clean corpus exits zero; any finding exits non-zero with the count.

### `reseat` — the repair half

`reseat` renumbers an entity's canonical-id quad — the directory name, the
`<stem>-NNN.{toml,md}` filenames, the toml `id` field (written edit-preservingly so
comments and unknown keys survive), and the `NNN-slug` alias — to the next-free
trunk-aware id, or to an explicit `--to`. It keys on the canonical ref (`SL-031`),
never a bare number, because the kind disambiguates the per-namespace id. Two guards
fire *before* any mutation: an occupied target is refused (no clobber), and an id
with live gitignored runtime phase state is refused (reseat does not own the
disposable tier — clear it first). Inbound prose citations of the old ref are
reported as danglers and force a non-zero exit *even on a fully-completed reseat* —
the rename succeeded, but the citations are the human's to rewrite by hand, because
prose relations are outbound-only (ADR-004); reseat never rewrites prose.

## Concerns

- **Fork collision is the headline hazard.** Two agents in separate clones minting
  the same id is the immediate coordination failure this container exists to
  prevent; the trunk-id union is its shipped mitigation, and `validate`/`reseat` are
  the detect/repair backstop when a collision lands anyway (e.g. a pre-merge
  reservation race the local backend could not see).
- **Local-only degradation is silent reach loss, not incorrectness.** Without a
  reachable trunk, allocation is correct and lock-free on one tree but cannot see
  other forks; the cross-fork guarantee requires a trunk to union against.
- **`KINDS` is a drift surface.** A numbered kind not registered there escapes every
  integrity check — an accepted cost of one flat table over a threaded registry.
- **`reseat` is non-transactional.** Its post-guard filesystem ops are a sequence,
  not a transaction; a mid-sequence failure leaves a half-reseated entity that
  `validate` will flag. It targets freshly-minted pre-execution collisions where
  that blast radius is acceptable.

## Hypotheses

- **`max + 1` over a per-namespace scan is sufficient allocation.** A namespace scan
  plus an atomic claim is preferred over a stored high-water-mark counter; a counter
  would be a second coordination primitive and break the reservation-is-a-claim
  unification to dodge a ref-count cliff no Doctrine kind approaches.
- **Trunk union closes the fork gap without a shared store.** Folding trunk ids into
  the local scan is preferred over requiring a network reservation backend for the
  common case, so single-tree and forked work are both collision-free offline; the
  shared-backend (`git-ref`) reach is deferred until a caller needs cross-team
  claims.
- **Detect and repair are deliberately separate verbs.** `validate` only reports and
  `reseat` only repairs one entity at a time, preferred over an auto-fixing scan, so
  renumbering is always a deliberate human-driven act and never silently rewrites the
  committed corpus.

## Decisions

- **D1 — reservation is an interpretation of the engine's claim seam, not a second
  primitive.** The `max + 1` arbitration and retry loop are owned here; the atomic
  `Claim` primitive (`mkdir` is the local backend) is owned by the parent engine
  (SPEC-004) and not restated. A `claim` is the generic "this dir is mine"; a
  *reservation* is what the numbered callers build on it.
- **D2 — trunk ids are a constant input read once at the shell edge.** Fork safety
  is `max(local ∪ trunk) + 1`; trunk is read once (impure `ls-tree`) and held across
  retries, only the local scan re-reads to recover a lost race — keeping the pure
  allocation decision a function of its inputs.
- **D3 — `validate` detects, `reseat` repairs (ADR-006 D3).** The two halves are
  split: a pure-check corpus scan that only reports, and a guarded single-entity
  renumber that only repairs and never rewrites inbound prose (ADR-004).
- **D4 — `KINDS` is one flat table, accepting the registration drift surface.** The
  `(prefix, dir, stem, state_dir)` quad lives in one place rather than being threaded
  through every kind-owning module; the cost is that an unregistered numbered kind
  escapes the scan.
- **D5 — memory's named identity is out of scope.** A `mem_<uid>` entity has no
  reserved number, so it is excluded from both next-free allocation and `validate`;
  its alias integrity is a deferred key-based variant, not part of the numbered
  lifecycle.
