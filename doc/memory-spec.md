# Memory specification — umbrella

**Status: direction. Decisions locked, build sequenced, most deferred.** This is
the umbrella for doctrine's memory subsystem. It locks the architecture and the
schema so the v1 entity can land without restructuring, and reserves the seams
(lifecycle ledger, event interchange, pluggable event-store backend, vector/graph
retrieval) so each attaches later additively. It does **not** start a build; it
pins the decisions the build will lean on. Sits under
[entity-model](entity-model.md) and reuses the slice/drift entity machinery
([slices-spec](slices-spec.md), [drift-spec](drift-spec.md)).

doctrine has no memory subsystem today; the installer already reserves
`.doctrine/memory`. The design below resolves what that directory holds.

## Thesis

A **memory** is a durable, scoped, typed unit of recalled knowledge — a fact,
pattern, signpost, or working thread that an agent or human would otherwise
rediscover each session. Two things must stay separate and are conflated by naïve
designs:

1. **The shape of meaning** — typed, scoped, linked, ranked knowledge with
   provenance and review state. This is what doctrine authors and queries.
2. **The persistence substrate** — how the bytes are stored and made durable.

doctrine owns the first and treats the second as **pluggable**. The default
substrate is the doctrine-native file model (mutable owned TOML + prose, the
storage rule every other entity follows). A second substrate — an **append-only
event-store backend** for shared, durable, multi-client memory — is an
*interoperability target*, not v1. The model is shaped so that adopting it later
is additive (§ Interoperability constraints, § Backend abstraction).

## What v1 is, and what is reserved

The same "the apparatus arrives with the caller" line every doctrine spec draws
([reservation-spec](reservation-spec.md), [drift-spec](drift-spec.md)):

- **v1 (native current-state memory).** A memory entity with the storage rule:
  per-memory `memory.toml` (owned, edit-preserving) + `memory.md` (prose).
  `record` / `show` / `list` / `find` / `retrieve`, scope-first lexical
  retrieval, deterministic ranking, git-anchored staleness. Committed to git,
  reviewable, greppable. Reuses the slice/drift entity engine — memory is its
  next caller, not a parallel implementation.
- **Reserved seam (designed here, built when its caller exists).** The
  append-only **lifecycle ledger** (`events.toml`), the **NDJSON event
  interchange**, the **event-store backend adapter**, and **vector/graph**
  retrieval. v1's schema and identity rules are chosen so these bolt on without a
  rewrite. The current-state files are lossless-exportable to the event stream
  the moment the ledger is switched on.

The interoperability constraints (§ next) **shape v1 now** — UUID identity,
integer-only-encodable numerics, scope-as-coordinate, no floats in payload,
hostile-input rendering — even though the backend itself is deferred. That is the
whole point of locking them early.

## Interoperability constraints

To stay swappable onto an append-only event-store backend without a rewrite, the
model obeys these constraints from day one. They are backend-neutral
requirements, stated against an abstract substrate.

1. **Lifecycle is facts, not field-mutation — at the seam.** The native backend
   edits `memory.toml` in place (its grain). But every mutating operation
   (`supersede`, `retract`, `review`, `re-tag`, `reanchor`) is *also* expressible
   as an appended **event**, because the interop backend has no update or delete —
   only append. The current-state file is the native authority; the event is the
   portable fact. The reserved ledger (§ Lifecycle ledger) is where the native
   side records those facts so export is lossless.
2. **Ride payloads, never reshape the substrate.** All domain meaning — type,
   scope, trust, provenance, links — lives in self-describing record/event
   *fields*, never in substrate-specific columns. The backend stores opaque
   payloads; doctrine never asks it for a new column.
3. **Identity is client-minted and content-addressed.** A memory's id and every
   event's id are deterministic and stable across retries and clients (§ Identity).
   This is what makes append idempotent and retries free on any backend.
4. **Anchor before consume.** No repo-scoped memory is written or read without an
   explicit git context frame that doctrine builds (§ Scope & anchoring). The
   backend never infers a git coordinate; doctrine constructs it.
5. **Integer-only numerics in payload.** Any field that crosses to the backend
   carries no fractional or exponent numbers — scores, weights, confidences, and
   (eventually) embeddings encode as scaled integers, strings, or out-of-band
   bytes. Floats in canonical payload are a hard error on the target substrate, so
   doctrine never emits them (§ Schema, § Known risks).
6. **Carry the workspace coordinate.** `workspace` is a real coordinate on every
   memory from the first record, even while single-tenant deployments pin it to a
   default. Identity and scope never bake in a single-workspace assumption.
7. **Stored memory is hostile input.** Memory text is untrusted data, never
   trusted instruction, on every backend (§ Security).

These map cleanly: a native `edit` becomes an event `append`; a native `query`
becomes a projection `read`. Anywhere the file model mutates, the substrate
models a new fact and folds it.

## Storage layout

The storage rule ([entity-model](entity-model.md)) — identity+facets in owned
TOML, prose in markdown — applied to memory:

```text
.doctrine/memory/
  items/
    <memory_uid>/
      memory.toml          # canonical current state — owned, edit-preserving
      memory.md            # prose body; [[...]] wikilinks for cross-refs
      events.toml          # RESERVED: append-only lifecycle ledger (deferred)
    mem.pattern.cli.skinny -> <memory_uid>   # slug alias symlink (convenience)
  index/                   # derived lexical + backlink index — gitignored, rebuildable
  embeddings/              # RESERVED: optional vector sidecars — gitignored
  state/                   # ephemeral working overlays — gitignored
```

- The **`<memory_uid>` directory is canonical**; tools resolve memories by uid.
- The **slug symlink** (`<memory_key> -> <memory_uid>`) is a human convenience
  alias, maintained by the tooling, not authoritative — the same pattern slices
  use for slugs (slices-spec § On-disk structure).
- `items/<uid>/` is **git-tracked** (authored, diffable, reviewable). `index/`,
  `embeddings/`, `state/` are **derived/ephemeral and gitignored** — the install
  manifest's blanket `.doctrine/memory/*` ignore is split accordingly
  (§ Install changes).

### Identity departs from numeric reservation

Memory is the first doctrine entity **not** numbered by the `mkdir` reservation
primitive ([reservation-spec](reservation-spec.md)). Interop constraint 3
mandates a client-minted, stable, content-addressable id, and a UUID is
**collision-free across clones by construction** — so memory sidesteps the
distributed-id-collision risk that slices and drift carry entirely. No
reservation namespace, no `acquire` call. The human-facing handle is the
`memory_key` slug (spec-driver's `mem.<type>.<domain>.<subject>` naming), carried
as a field and aliased by the symlink — not the directory's identity.

## Schema — `memory.toml`

Owned, locked-schema, edit-preserving (the mutating verbs use `toml_edit`, never a
full reserialize — same caveat as drift/spec entities). Closed enums where the
vocabulary is locked; free-text/soft where external producers vary.

```toml
memory_uid = "mem_018f3a..."        # client-minted UUID; the idempotency anchor; stable forever
memory_key = "mem.pattern.cli.skinny"  # optional human slug (mem.<type>.<domain>.<subject>)
schema_version = 1
memory_type = "pattern"             # concept | fact | pattern | signpost | system | thread
status = "active"                   # active | draft | superseded | retracted | archived | quarantined
title = "Skinny CLI Pattern"
summary = "CLI delegates to domain logic and formatters."   # one line; used in match
created = "2026-06-04"
updated = "2026-06-04"

[scope]                             # the retrieval key (§ Retrieval)
paths    = ["src/main.rs"]          # exact/prefix — strongest match
globs    = ["src/**/*.rs"]          # glob match
commands = ["doctrine slice"]       # command token-prefix triggers
tags     = ["cli", "architecture"]  # stable categorization (not overloaded as scope)
workspace = "default"               # coordinate — carried always (interop constraint 6)
repo = "github.com/davidlee/doctrine"
repo_id_kind = "remote"             # explicit | remote | local_root — how repo was derived
repo_id_confidence = "high"         # high | medium | low — convergence across clones

[git]                               # anchor; doctrine builds it, the backend never does
anchor_kind = "commit"              # commit | checkout_state | none (dirty is derived from this, not stored)
commit = "..."                      # set iff anchor_kind = commit (clean tree); empty when dirty
tree = "..."                        # HEAD^{tree}; part of the locked frame (decision 6)
checkout_state_id = ""              # set iff anchor_kind = checkout_state (dirty tree)
base_commit = "..."                 # HEAD the memory sits on
ref_name = "refs/heads/main"        # empty when detached HEAD (still anchored to commit/checkout)
verified_sha = "..."                # SHA at last verification — enables git staleness
normalizer = "forget.checkout.v1"   # frozen frame-algorithm tag (the interop seam, § Backend abstraction)

[review]                            # SEPARATE from lifecycle status (approval ≠ lifecycle)
verification_state = "unverified"   # unverified | verified | stale | disputed
reviewed = ""                       # date of last verification against reality
review_by = ""                      # scheduled review horizon (shorter for volatile)

[trust]
trust_level = "medium"              # low | medium | high
confidence = "medium"               # low | medium | high (spec-driver calibration)
actor_type = "agent"                # user | service | agent | system | external
actor_id = "..."
producer_id = "doctrine-cli"        # provenance — must NOT enter idempotency keys
capture_method = "manual"           # free-text

[ranking]
severity = "high"                   # critical | high | medium | low | none
weight = 8                          # integer; higher = earlier

[[relation]]                        # hand-authored lifecycle edges only (§ Links)
rel = "supersedes"                  # related_to | supersedes | depends_on | contradicts | derived_from | explains | implements
to  = "mem_018e..."

[[source]]                          # provenance — enables trust/verification
kind = "code"                       # code | adr | spec | commit | doc
ref  = "src/main.rs"
note = ""
```

**Derived, never stored** (derive-don't-store, [relation-index](relation-index.md)):
backlinks; `[[...]]` resolution from the body; any "ids owned by this memory"
list. Computed by the registry at query time, not written into the file.

### Memory types (6, provisional)

Lifted from spec-driver — proven, and just enum values in a payload field
(interop widens kind in payload, never in substrate schema). The *set* is
provisional; members may be tuned before implementation. The model does not
depend on the exact membership.

| Type | Purpose | Lifespan |
|---|---|---|
| `concept` | Stable mental model, terminology, taxonomy | Long |
| `fact` | Atomic checkable truth (invariant, default, limit) | Long |
| `pattern` | Repeatable recipe / command sequence / workflow | Medium |
| `signpost` | Navigation pointer set — "start here" for a domain | Long |
| `system` | Subsystem map + architecture pointers | Medium |
| `thread` | Short-lived working set for one task/slice | Days–weeks |

Rule: use the narrowest type that fits. A durable `thread` is **promoted**
(`fact`/`pattern`/`system`), not left to linger (§ Retrieval, thread expiry).

### Lifecycle status (6, provisional) — separate from review

`status ∈ {active, draft, superseded, retracted, archived, quarantined}`. Drops
spec-driver's `deprecated`/`obsolete` (folded into `superseded`/`archived` —
fewer states, family-specific vocab, entity-model § State vocabulary).

| Status | Default retrieval |
|---|---|
| `active` | included |
| `draft` | excluded (unless `--include-draft`) |
| `superseded` | excluded; visible when tracing |
| `retracted` | excluded; audit-visible |
| `archived` | excluded; historical |
| `quarantined` | excluded from agent context; review-only (security) |

**Review/verification state is a separate axis** (`[review].verification_state`):
a memory can be `active` + `stale` + `unverified` simultaneously. Lifecycle is not
verification; verification is not approval (entity-model § State vocabulary).
Advanced by the change/review process, not folded into `status`.

## Lifecycle ledger — `events.toml` (reserved seam)

The append-only half, deferred until its caller (the event-store backend adapter,
or an audit requirement) exists. Designed now so it lands without restructuring —
it **is** the drift-ledger shape ([drift-spec](drift-spec.md)) specialised to
memory lifecycle: an array of tables, appended edit-preservingly, written atomically
beside the `memory.toml` mutation it records.

```toml
[[event]]
event_id = "..."                    # deterministic uuid5 (§ Identity)
event_type = "reviewed"             # recorded | reviewed | superseded | retracted | reanchored | linked | tagged | promoted
occurred_at = "2026-06-04T00:00:00Z"
actor_type = "agent"
actor_id = "..."
# event-specific data inline — integer-only numerics (interop constraint 5)
```

Event families and their fold (replay → current projection, for the backend that
needs it): `recorded` (create), `reviewed` (verification + horizon), `superseded`,
`retracted`, `reanchored` (rebind git/source facts), `linked`, `tagged`,
`promoted` (thread → durable). The native backend does not *replay* to read — the
`memory.toml` file already is the current state; the ledger is the audit trail and
the export source. The interop backend *does* replay; both fold the same event
sequence to the same projection (the conformance contract, § Backend abstraction).

The dual write (`memory.toml` current state + `events.toml` history) is the one
deliberate "store twice" — current-and-queryable vs history-and-portable, not
redundant denormalization. It carries drift-spec's self-drift risk and mitigation
verbatim (§ Known risks).

## Identity

- **`memory_uid`** — client-minted UUID, minted **once** per logical memory, the
  idempotency anchor across the whole lifecycle and all retries. Stored, never
  regenerated.
- **`memory_key`** — optional human slug, `mem.<type>.<domain>.<subject>[.<purpose>]`
  (per-segment `[a-z0-9]+(-[a-z0-9]+)*`, 2–7 segments). Authoring/display/links
  handle; mutable; aliased by the symlink. Not identity.
- **`event_id`** — deterministic `uuid5` over a **fixed namespace**, keyed off the
  event's natural identity (`{memory_uid}:{event_type}:{discriminator}`), so every
  client derives the same id for the same fact. Never random for a re-observable
  fact. The `recorded` event keys identically to the backend's claim id so an
  append and a doctrine record dedup at the seam.
- **`source_uri`** (export/interop) — fixed and versioned (`doctrine://memory/v1`).
  It is part of the substrate's uniqueness key, so machine/install provenance
  goes to event metadata, never here.

## Scope & anchoring

**Scope is the retrieval key** (spec-driver principle 2): a memory with no scope
cannot be found by context-aware query. Every actionable memory carries at least
one of `scope.paths` / `scope.globs` / `scope.commands`.

The **git context frame** is built by doctrine (interop constraint 4); the backend
never shells out to git. Minimum frame for a repo-scoped write/read: `repo` (+
`repo_id_kind`/`repo_id_confidence`), HEAD `commit` + `tree` + `ref_name`, the
dirty distinction (carried as `anchor_kind`, not a separate stored `dirty` flag —
derive-don't-store), `checkout_state_id` when dirty, and `base_commit`. A
repo-scoped memory **requires** a born frame; an unborn/non-git context yields an
`unanchored` memory, permitted only for unscoped memory. `workspace` is carried
always.

**The frame algorithm is frozen, and shared with the interop backend.** The
event-store counterparty (`the external decision register`, § Backend abstraction) already defines this
frame as `GitContextFrameV1` under the normalizer tags `forget.remote.v1`
(`repo_id`) and `forget.checkout.v1` (`checkout_state_id`). doctrine's git seam
(`src/git.rs`) **reproduces that algorithm byte-for-byte** so a doctrine `record`
and a backend claim derive identical ids and dedup at the seam (§ Identity,
interop constraint 3); a shared conformance golden-vector pins the equivalence.
Concretely: `repo_id` is derived by precedence explicit→remote→local-root (ambiguous
multi-remote is an error, not a guess); `checkout_state_id` is a content-bearing hash
(`git write-tree` index + `sha256(git diff HEAD --binary)` + sorted untracked
content-hashes), so distinct edits to the same fileset do not collide; a detached
HEAD is still anchored (`commit`/`checkout_state`, empty `ref_name`); and capture
runs git under config-independent normative flags. doctrine adopts the external decision register's
unstable-frame guards: born/unborn/non-repo are three distinct states, and
submodule/symlink/multi-root trees are rejected rather than anchored unstably.

## Retrieval & ranking

Scope-first, lexical-first (a BM25-class lexical index is a strong baseline; dense
and graph retrieval are deferred sidecars, § Open questions). Lifted from
spec-driver — proven and deterministic.

**Scope matching** — OR across dimensions, with specificity weights; records
without scope are excluded from scope-filtered queries:

| Dimension | Match | Specificity |
|---|---|---:|
| `scope.paths` | exact or prefix | 3 |
| `scope.globs` | glob (`**` aware) | 2 |
| `scope.commands` | token-prefix | 1 |
| `tags` | set intersection | 0 |

**Deterministic sort key** (same query ⇒ same order — agent reproducibility):

1. Hard filters: workspace, repo, lifecycle status, trust/quarantine, git visibility.
2. Retrieval relevance: lexical score, exact `memory_key` match.
3. Scope specificity (above).
4. Verification state.
5. Trust level / provenance quality.
6. Severity.
7. Weight (higher first).
8. Review/verification recency (fewer days first; null last).
9. `memory_uid` / `memory_key` tiebreaker.

Dense reranking or graph expansion, when added, contribute **bounded signals into
this tuple** — they never break the deterministic final ordering.

**Staleness** — three modes, the metric chosen by what anchoring is available:

| Mode | Condition | Metric |
|---|---|---|
| scoped + attested | has scope + `verified_sha` | commits touching scoped paths since the SHA |
| scoped, unattested | has scope, no `verified_sha` | days since `reviewed` |
| unscoped | no path/git scope | days since `reviewed` |

Undecidable reachability (shallow/partial clone, detached HEAD, rebase,
non-ancestor anchor, non-git project) resolves to an **explicit** state —
`fresh` / `stale` / `unknown` / `unanchored` — never a silent hide or a silent
over-trust.

**Thread expiry.** A `thread` additionally requires a scope-matched context **and**
verification within 14 days to surface; otherwise excluded. Durable thread content
is promoted to `fact`/`pattern`/`signpost`/`system`, not left as a thread.

## Links & relations

Two mechanisms, the same split spec-driver and entity-model draw:

- **`[[...]]` wikilinks in the prose body** are the preferred cross-reference —
  cheap to author, and **backlinks + `links.out` are derived** (relation-index),
  never hand-maintained. Links inside fenced/inline code are skipped.
- **`[[relation]]` rows in `memory.toml`** are reserved for **lifecycle
  semantics** (`supersedes`, `depends_on`, `contradicts`, …) — assertions that are
  facts about the memory's place in the graph, not navigation. These are the rows
  that fold to relation **events** on the interop backend (constraint 1).

FK validation (every relation target resolves) is the registry's job
([relation-index](relation-index.md) § Two purposes), lazy and cache-independent —
memory relations fold into the same registry as slice/spec edges.

## Security

Memory is a **durable hostile-input substrate**, not a friendly note store. The
posture is non-negotiable and backend-independent:

- **Render as data, never instruction.** Retrieved memory is inserted into agent
  context as a quoted, delimited, attributed block carrying `memory_uid` /
  `memory_key`, `trust_level`, `verification_state`, scope, and anchor. Stored
  text may never override system, developer, or user instructions — no latent
  prompt injection.
- **Suppress by default:** `quarantined` and `retracted` memories never reach
  agent context; low-trust high-risk memory is held back.
- **Secret hygiene:** secret-scan/redact at ingestion; never place secrets in
  scope/coordinate/anchor fields (`repo`, refs, `source_uri`) — on the interop
  backend those feed hashes and projections and are not encrypted.
- **Partition** by workspace and repo; cross-workspace/cross-repo leakage is a
  modelled threat.
- **Audit survives redaction:** retraction suppresses the body from default recall
  but the ledger/history preserves it for audit (subject to policy-driven local
  redaction of generated projections).
- **Frame trust:** the single-tenant local frame is trusted without verification —
  safe only locally. The moment memory fans across clients/tenants, frame
  authenticity and producer trust become real requirements; the seam is designed,
  not built (§ Open questions).

## Backend abstraction

A backend-neutral interface of **logical commands and queries**, not CRUD —
mutable file edits and append-only event writes must not be presented as
equivalent.

Portable operations: `record`, `get`, `list`, `search`, `review`, `supersede`,
`retract`, `reanchor`, `link`, `tag`, `promote`, `verify`, `project`. Two
backends realise them:

- **`local-text` (v1).** Commands edit `memory.{toml,md}` edit-preservingly;
  derived `index/` rebuilt from the items; reserved `events.toml` records history.
  Source of truth is the file.
- **`event-store` (reserved).** Commands append immutable events on a per-memory
  stream; current state is a folded projection; integrity is a separate explicit
  call. Source of truth is the log.

The concrete `event-store` backend is **the external decision register**, which has accepted this
client role: see the external decision register `ADR-005` (generic append-only event substrate;
clients ride opaque payloads). It binds doctrine's per-memory stream shape —
`stream_type = "doctrine.memory"`, `natural_key = memory_uid`,
`source_uri = "doctrine://memory/v1"`, `event_id` the deterministic `uuid5` —
mandates the adapter use only the **generic event store** (never the external decision register's
first-party `/memory/*` domain — DEC-005-C), and commits the substrate to the
read surfaces this contract's rebuild/export needs (stream catalog + workspace
event feed — DEC-005-D). In agent jails the external decision register lives at
`/workspace/the external decision register`.

Differences the abstraction must keep neutral:

| Concern | `local-text` | `event-store` | Abstraction rule |
|---|---|---|---|
| Canonical store | files | event log | never expose file paths as canonical ids |
| Editing | in-place (append-also) | append-only | model `edit` as a command, never a mutation |
| Anchoring | optional/non-git allowed | required for repo scope | context frame is always explicit |
| Projection | TOML/MD files | folded read model | projection contract is backend-neutral |
| Integrity | optional local check | separate verify call | verification reports differ; don't assume |
| Embeddings | sidecar files | out-of-band, keyed by id | never floats in canonical payload |

**Conformance** proves equivalent *behaviour*, not parse success: deterministic
ids across retries/imports/backends; append-only operations append (never mutate
history); both backends fold the same event sequence to the same projection;
retrieval determinism; scope-matching cases; git-anchoring edge cases; lifecycle
filtering; security (injection, poisoning, secret capture, leakage, quarantine);
import/export round-trip with stable ids; projection rebuild from the log.

## Import / export interchange

The portable interchange — and the format the reserved ledger and the event-store
adapter both speak — is an **NDJSON event bundle** (rebuildable projections are
convenience exports):

```text
doctrine-memory-export/
  manifest.toml
  events.ndjson            # canonical domain events — the source of truth
  bodies/<memory_uid>.md
  projections/<memory_uid>/memory.{toml,md}   # rebuildable convenience
```

`events.ndjson` carries the lifecycle events; numbers are **integer-only**
throughout `data`/`metadata` (interop constraint 5) — embeddings and scores stay
out of the canonical payload, encoded or out-of-band.

## Concurrency & projections

No daemon, no lock — disposability dissolves the concurrency problem
([relation-index](relation-index.md) invariant):

- Mutating verbs are **edit-preserving atomic writes** (`toml_edit`); a
  duplicate-`memory_uid` (or duplicate ledger `event_id`) is a **hard lint at
  load over the merged file**, not a write-time guard — the spec/drift pattern.
  UUID identity makes uid collisions vanishingly unlikely even across clones.
- Derived `index/` (and `embeddings/`) are disposable, gitignored, per-clone,
  rebuilt atomically (temp-then-`rename`). Last-writer-wins on identical derived
  bytes is correct under concurrent agents.
- Content-addressed `event_id` tolerates branch-local ledger appends and validates
  stream order at rebuild — concurrent appends on different branches reconcile by
  content, not by sequence number.

## Architecture (Rust)

The three-layer model ([entity-model](entity-model.md) § Rust implementation), and
the pure/imperative split every doctrine entity uses:

```text
RawMemoryToml   # tolerant parse; preserves unknown keys (extra)
Memory          # validated: typed ids, closed enums, normalized paths
MemoryRegistry  # resolved relations + FK diagnostics (relation-index)
```

The git frame, the current date, and the directory listing are **inputs** to the
pure layer (no clock, no disk, no git inside it); frame construction, file IO, and
git reachability sit behind the same IO seam as `doctrine install` / `slice`.
Mutating verbs write `memory.toml` (+ reserved ledger) atomically and
edit-preservingly.

## Install changes

The manifest's blanket `.doctrine/memory/*` ignore (install-spec § Manifest) is a
placeholder; split it when memory lands:

```toml
[dirs]
create = [".doctrine/memory/items"]   # committed authored memory

[gitignore]
entries = [
  ".doctrine/memory/index/*",
  ".doctrine/memory/embeddings/*",
  ".doctrine/memory/state/*",
]
```

## Locked decisions

The ten must-decide open questions, resolved. Decisions 4 and 5 (type and status
vocabularies) are **provisional** — the shape is locked, the exact enum members
may be tuned before implementation.

| # | Question | Decision |
|---|---|---|
| 1 | Append-only history canonical in the native backend? | **No.** Native canonical = mutable `memory.{toml,md}`; the append-only ledger is the audit/export seam, canonical only on the event-store backend. |
| 2 | `memory_uid` / `event_id` algorithm | Client-minted UUID (uid, minted once); deterministic `uuid5` over a fixed namespace keyed by natural identity (event_id). § Identity. |
| 3 | `memory.toml` projection or editable? | **Editable canonical**, owned, edit-preserving (follows decision 1). |
| 4 | First-class memory types in v1 | *(provisional)* All **6** spec-driver types (§ Memory types). |
| 5 | Lifecycle states in v1 | *(provisional)* **6**: active/draft/superseded/retracted/archived/quarantined; review state a separate axis. |
| 6 | Minimum git context frame | repo (+ repo_id_kind/confidence) + HEAD commit/tree/ref + checkout_state_id (dirty) + base_commit; dirty derived from `anchor_kind` (not stored); algorithm frozen as the external decision register's `GitContextFrameV1` (`forget.remote.v1`/`forget.checkout.v1`), reproduced byte-for-byte; born frame required for repo scope (§ Scope & anchoring). |
| 7 | Commit durable memory files? | **Commit `items/`**; gitignore `index/`/`embeddings/`/`state/` (§ Install changes). |
| 8 | Prompt-injection rendering contract | Quoted, attributed, delimited data block; never instruction; suppress quarantined/retracted (§ Security). |
| 9 | Canonical import/export format | NDJSON event bundle + bodies + rebuildable projections; integer-only payload (§ Import/export). |
| 10 | Concurrent appends / projection rebuild | Edit-preserving atomic write; disposable gitignored derived index; content-addressed ids; duplicate-id hard lint at load (§ Concurrency). |

## Roadmap (sequencing guard)

Each step lands behind its own gate, one at a time — this umbrella does not expand
scope, it sequences it:

1. **Entity engine generalisation** (already roadmapped for slice→drift→spec) gains
   a **UUID-identity, no-reservation** variant — memory is its next caller.
2. **v1 native memory**: schema, `record`/`show`/`list`/`find`/`retrieve`, scope
   retrieval + deterministic ranking + git staleness, commit policy, manifest split.
3. **Links/backlinks** fold into the relation-index registry.
4. **Reserved seam (deferred)**: lifecycle ledger, NDJSON import/export, event-store
   backend adapter, vector/graph retrieval — each when its caller exists.

## Open questions (deferred)

1. **Lexical backend choice** — embedded index vs grep-class scan at v1 scale.
2. **Embedding sidecar contract** — when dense retrieval lands; out-of-band keying.
3. **`memory_key` default** — required vs optional, and the auto-derivation rule.
4. **Pre-hook surfacing** (`visibility = pre`) — whether v1 ships proactive memory
   injection or only on-demand `retrieve`.
5. **Multi-client frame authenticity** — when memory fans across clients/tenants
   (the security seam, § Security).
6. **Graph/hierarchical retrieval, reflection/summarisation, retention/erasure
   policy** — all deferred.

## Known risks

- **Ledger/file self-drift.** `memory.toml` current state and the `events.toml`
  history can desync under hand edits. Mitigation inherited verbatim from
  [drift-spec](drift-spec.md): the atomic mutating verb writes both
  edit-preservingly; a load-time lint flags an event with no corresponding state
  change (and vice versa).
- **Float-in-payload trap.** Any score/weight/confidence/embedding that reaches the
  export or the event-store backend as a JSON float is a hard substrate error.
  Encode as scaled integer / string / out-of-band bytes at the boundary; lint the
  export path.
- **Over-trust of stale or poisoned memory.** High lexical similarity must not
  outrank verification/provenance/scope/trust. Rank by those (§ Retrieval); suppress
  quarantined/retracted; render all memory as data.
- **Projection drift** (event-store backend). Generated projections diverge from the
  folded log. Mitigation: rebuild-from-log conformance tests; CI check that
  projection == replay.
- **`memory_key` mutability vs stale symlink.** Editing `memory_key` by hand does
  not move the slug symlink (slices' stale-symlink risk). v1 accepts drift; a future
  `re-key` reconciles. The uid (and its directory) is the stable identity, so tooling
  is unaffected.
- **Secret capture.** Durable retention of secrets pulled into memory text.
  Mitigation: ingestion secret-scan; never route secrets through coordinate/anchor
  fields.

## Follow-ups

- **Glossary.** Add the memory kind and its `mem.*` referends to
  [glossary](glossary.md) when this leaves direction.
- **Entity engine.** Track the UUID-identity / no-reservation variant as a
  capability of the generalised engine, not a memory-specific fork.
- **Locality recovery CLI.** `doctrine memory show <key-or-uid>` reassembles the
  record, prose, derived backlinks, and staleness into one human view — the read
  analogue of `spec req show`.
