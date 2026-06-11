# SPEC-007: Memory engine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Memory is the scope-aware durable-knowledge store: a typed, scoped, ranked unit of
recalled knowledge an agent or human would otherwise rediscover each session. This
container sits beneath the whole-system root (SPEC-003) and rides the shared entity
engine (SPEC-004) for materialisation, the storage rule, and edit-preserving writes;
it restates none of that. What it owns is everything specific to *memory*: the
named (UUID-identity, no-reservation) entity shape, the memory domain vocabulary,
the `record` producer with its git-frame capture, the scope-aware `find`/`retrieve`
reader with its deterministic ranking and git-anchored staleness, the security
render contract with its non-bypassable trust holdback, and the global/derived
orientation class. The append-only lifecycle ledger, NDJSON interchange, and
event-store backend adapter are designed reserved seams, not shipped, and are not
owned here.

Memory is the entity that *forced* the engine's second identity shape: it is the
first kind not numbered by the `mkdir` reservation primitive, taking a client-minted
content-addressable UUID instead. That generalisation lives in the parent engine
(the named materialiser); this container owns the memory-specific reasons for it and
the model that rides it.

## Responsibilities

Mirrors the structured `responsibilities` list: own the named memory entity and its
store layout; carry the memory domain vocabulary; provide the `record` producer;
build and freeze the git context frame; provide the scope-aware reader; compute
git-anchored staleness; enforce the security render contract and trust holdback; and
carry `verify` plus the global/derived orientation class.

### The named memory entity and its store

A memory is the first Doctrine entity addressed by a client-minted UUID rather than
a reserved number — interop demands an id that is stable across retries and clones
and content-addressable, and a UUID is collision-free across clones by construction,
so memory sidesteps the distributed-id-collision risk entirely (no reservation
namespace, no `acquire`). The `<memory_uid>` directory is canonical; tooling
resolves memories by uid. The store applies the storage rule:

```text
.doctrine/memory/
  items/<memory_uid>/
    memory.toml          # canonical current state — owned, edit-preserving
    memory.md            # prose body
    events.toml          # RESERVED: append-only lifecycle ledger (deferred)
  mem.<type>.<domain>.<subject> -> <memory_uid>   # slug alias symlink (convenience)
  index/ embeddings/ state/    # derived / ephemeral — gitignored
  shipped/                     # materialised global/derived corpus — gitignored
```

The `memory_key` slug (`mem.<type>.<domain>.<subject>[.<purpose>]`) is a human
handle aliased by the symlink — mutable, never the directory's identity; the uid is
the stable identity, so a hand-edited key that leaves a stale symlink does not
confuse the tooling. `items/` is committed and reviewable; `index/`, `embeddings/`,
`state/`, and `shipped/` are derived/ephemeral and gitignored.

### The memory domain vocabulary

Three orthogonal axes, each a closed enum in the payload (interop widens kinds in
the payload, never in the substrate schema):

- **Six memory types** — `concept`, `fact`, `pattern`, `signpost`, `system`,
  `thread`. The rule is to use the narrowest type that fits; a durable `thread` is
  *promoted* to a durable type, not left to linger.
- **Six lifecycle statuses** — `active`, `draft`, `superseded`, `retracted`,
  `archived`, `quarantined` — each with a default-retrieval visibility (`active`
  included; the rest excluded by default, `quarantined` and `retracted` never
  reaching agent context).
- **A separate review/verification axis** — `unverified` / `verified` / `stale` /
  `disputed`. Verification is not lifecycle and not approval: a memory can be
  `active` + `stale` + `unverified` at once.

### The `record` producer and git-frame capture

`record` mints the uid, constructs the git context frame, writes the scope
(`paths`/`globs`/`commands`/`tags`) and the captured anchor, and scaffolds the item;
`memory new` is the uniform-grammar alias dispatching the identical handler. The git
frame is built by Doctrine, never inferred by any backend, and the algorithm is
frozen and shared with the interop counterparty (`the external decision register`): `src/git.rs`
reproduces `GitContextFrameV1` (normalizer tags `forget.remote.v1` /
`forget.checkout.v1`) byte-for-byte, so a Doctrine record and a backend claim derive
identical ids and dedup at the seam. The frame derives `repo_id` by precedence
explicit → remote → local-root (ambiguous multi-remote is an error, not a guess),
hashes a content-bearing `checkout_state_id` for a dirty tree (so distinct edits to
one fileset do not collide), keeps born / unborn / non-repo as three distinct
states, and stores `anchor_kind` rather than a separate dirty flag (dirty is
derived). A repo-scoped memory requires a born frame; an unborn or non-git context
yields an `unanchored` memory, permitted only for unscoped memory.

### The scope-aware reader and deterministic ranking

Retrieval is scope-first and lexical-first. Scope matching is an OR across
dimensions with specificity weights — `paths` (exact/prefix, weight 3), `globs`
(glob, 2), `commands` (token-prefix, 1), `tags` (set intersection, 0); a memory with
no scope cannot be found by a scope-filtered query. Results order by a nine-key
deterministic sort tuple — hard filters (workspace, repo, lifecycle status,
trust/quarantine, git visibility) first, then lexical relevance and exact key match,
scope specificity, verification state, trust/provenance, severity, weight, review
recency, and a uid/key tiebreaker — so the same query yields the same order for
agent reproducibility. `find` and `retrieve` share this ranking; they differ only in
the holdback and the render contract.

### Git-anchored staleness

Staleness has four explicit modes, the metric chosen by what anchoring is available:
scoped + attested counts commits touching scoped paths since `verified_sha`;
scoped-unattested and unscoped fall back to days since `reviewed`; the global/derived
class is evergreen — decay-exempt, rendering an explicit non-decaying `reference`
state, its `reviewed` never re-stamped (that would break idempotent sync). Every
undecidable reachability case (shallow/partial clone, detached HEAD, rebase,
non-ancestor anchor, non-git project) resolves to an *explicit* state — `fresh` /
`stale` / `unknown` / `unanchored` / `reference` — never a silent hide or a silent
over-trust.

### Security render contract and the trust holdback

Memory is a durable *hostile-input* substrate, and the posture is backend-independent
and non-negotiable. `retrieve` renders every memory as a quoted, attributed,
delimited data block carrying uid/key, trust, verification, scope, and anchor —
data, never instruction — and suppresses `quarantined`/`retracted`. The **trust
holdback** is non-bypassable: low-trust high-severity memories are withheld from
agent context on `retrieve`, while `find` and `show` keep them inspectable (the
find surface is holdback-exempt so risk stays visible). Partition by workspace and
repo is a hard filter; a memory whose `repo` is non-empty and differs from the
querying repo is filtered out — that filter *is* the cross-repo boundary.

### `verify` and the global/derived orientation class

`verify` attests a memory against the current working tree, stamping the verification
axis; it refuses a dirty tree so no false attestation is recorded. The
global/derived orientation class (ADR-002) is the one sanctioned *scoped-yet-
unanchored* memory: `repo = ""` + `anchor_kind = none`, minted upstream of any
client repo, so an empty repo-id is admitted in every partition (a real anchor or a
non-empty repo would assert something false about the client or self-exclude it).
`memory sync` materialises the embedded global corpus into the gitignored `shipped/`
tree (a clean no-op outside a Doctrine repo); the class is defined by that signature,
not a new memory type.

## Concerns

- **Hostile-input substrate.** Stored memory text is untrusted data; the render
  contract, quarantine/retract suppression, and the trust holdback exist to keep it
  from acting as latent instruction in agent context.
- **Over-trust of stale or poisoned memory.** High lexical similarity must never
  outrank verification, provenance, scope, and trust — which is why those sit above
  lexical score in all but the relevance tier of the sort tuple.
- **Anchoring on impure git.** The git frame, the current date, and the directory
  listing are inputs to the pure layer; frame construction, file IO, and git
  reachability sit behind the IO seam, never inside the pure model.
- **Interop without a rewrite.** v1 obeys the interop constraints now — UUID
  identity, integer-only numerics in payload, scope-as-coordinate, the always-carried
  `workspace`, hostile-input rendering — so the reserved event-store backend bolts on
  additively even though it is unbuilt.

## Hypotheses

- **Scope is the retrieval key.** A memory's findability is its scope; modelling
  paths/globs/commands/tags as the primary coordinate (with deterministic specificity
  weights) is preferred over free-text search, so context-aware recall is precise and
  reproducible.
- **UUID identity beats a reservation namespace for memory.** Because memory must be
  recordable offline, across clones, and idempotently on retry, a client-minted
  content-addressable UUID is preferred over the `mkdir` reservation primitive every
  numbered kind uses — collisions vanish by construction.
- **A deterministic sort tuple beats a learned ranker.** A fixed nine-key ordering is
  preferred so the same query always yields the same order; any future dense or graph
  signal contributes a *bounded* input into the tuple, never breaking the final
  deterministic order.
- **The trust holdback belongs on the consume path, not the store.** Suppressing
  low-trust high-severity memory at `retrieve` (while leaving it inspectable via
  `find`/`show`) is preferred over deleting or hiding it at rest, so risk stays
  auditable while never silently entering agent context.

## Decisions

- **D1 — memory is a named entity, not a numbered one.** Memory takes a client-minted
  UUID and the engine's named materialiser; this is the caller that forced the second
  identity shape, and the reason lives here while the mechanism lives in the parent
  engine.
- **D2 — the git frame algorithm is frozen and shared.** `src/git.rs` reproduces
  the external decision register's `GitContextFrameV1` byte-for-byte under fixed normalizer tags so a
  Doctrine record and a backend claim derive identical ids; the frame is built by
  Doctrine and never inferred by any backend.
- **D3 — verification is a separate axis from lifecycle status.** A memory's
  `active`/`draft`/… status and its `unverified`/`verified`/`stale`/`disputed`
  verification state are orthogonal; verification is advanced by `verify`/review, never
  folded into `status`.
- **D4 — the trust holdback is non-bypassable on `retrieve`, exempt on `find`.**
  Low-trust high-severity memory is withheld from agent context but kept inspectable,
  so the security boundary and the visibility of risk are both preserved.
- **D5 — the global/derived class is a signature, not a new type.** `repo = ""` +
  `anchor_kind = none` + scoped defines the evergreen, decay-exempt, every-partition
  orientation class (ADR-002); it lives in the gitignored `shipped/` tree, outside the
  committed capture store, leaving the scoped⇒anchored rule unchanged for captured
  memory.
- **D6 — the append-only seam is reserved, not built.** The lifecycle ledger
  (`events.toml`), NDJSON interchange, and event-store backend adapter are designed so
  they bolt on additively when their caller exists; v1's canonical authority is the
  mutable, edit-preserving `memory.toml`.
