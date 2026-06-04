# Memory entity v1

## Context

The memory umbrella ([memory-spec.md](../../../doc/memory-spec.md)) locked the
architecture and schema and sequenced the build: v1 is a **native current-state
memory entity** (per-memory `memory.toml` + `memory.md` under the storage rule),
with the append-only ledger, event interchange, pluggable backend, and
vector/graph retrieval all reserved as later seams. This slice is the first build
step — and deliberately the thinnest useful one, mirroring slices-spec's own first
cut (`new` + `list`, everything else deferred).

The slice exists where memory **first breaks the entity engine's identity
assumption**. slice-003/004 built and exercised the scaffold engine (`src/entity.rs`:
`Kind` descriptor, fileset-as-function, `acquire` seam, two `MaterialiseMode`s),
but every caller so far — slice, design doc, IP, phases — is **numeric-identity**:
`candidate_id` is `max + 1`, `scan_ids` collects numeric dirs, the id is a `u32`
rendered `{:03}`, and `AllocateFreshEntity` reserves it through a race-retry loop
([entity-model.md](../../../doc/entity-model.md) § Identity and references;
reservation-spec § The unification).

Memory is the **first client-supplied, string-identity, non-reserved caller**.
Interop constraint 3 (memory-spec § Interoperability constraints) mandates a
client-minted, stable, content-addressable `memory_uid`; a UUID is collision-free
across clones by construction, so memory **needs no reservation** — the `mkdir` on
the uid directory is creation, not race-arbitration, and an `AlreadyHeld` is a
genuine duplicate (a hard error), not a lost race to retry. So memory fits neither
existing `MaterialiseMode`. Generalising the engine to admit string identity —
driven by memory as the proving caller, not speculatively — is the architectural
core of this slice (design.md § 5).

## Scope & Objectives

- **Memory schema (parse layer).** `RawMemoryToml` (tolerant serde, unknown keys
  preserved in an `extra` catch-all) → `Memory` (validated: closed `memory_type`
  and `status` enums, normalized fields), plus the `memory.md` prose body. The
  three-layer model ([entity-model.md](../../../doc/entity-model.md) § Rust
  implementation model) minus the registry layer (no relations resolved in v1).
  Round-trip tested; unknown-key preservation asserted. The provisional type/status
  vocabularies are memory-spec §§ Memory types / Lifecycle status.

- **Engine string-identity generalisation.** `src/entity.rs` grows a third
  placement that materialises an entity under a **caller-supplied name** with **no
  id reservation** — the uid names the directory, `acquire` claims it once, an
  `AlreadyHeld` is a hard duplicate error (no retry, no `candidate_id`, no
  `{:03}`). The numeric callers (slice/drift/design/phases) are **behaviour-
  preserving** — their suites gate every step. The exact mechanism (a third
  `MaterialiseMode` variant + an `EntityId` widening of `ScaffoldCtx`/`Materialised`,
  vs a fuller identity strategy) is the design doc's central decision (design.md
  § 5.1, § 7); the principle is *generalise only as far as the second identity
  shape forces* (reservation-spec § "apparatus arrives with the caller").

- **`doctrine memory record`.** Mint a `memory_uid` (in the imperative shell, an
  input to the pure layer like the date — no clock/rng in pure code), accept an
  optional `--key` (`mem.<type>.<domain>.<subject>`, shorthand normalized), and
  scaffold `.doctrine/memory/items/<uid>/{memory.toml,memory.md}` with a
  `<key> -> <uid>` slug-alias symlink when a key is given (the slices-spec slug
  symlink pattern — convenience, not authority; tool resolves by uid). `--type`
  required; `--status` (default `active`), `--summary`, `--tag` optional.

- **`doctrine memory show <uid|key>` and `memory list`.** `show` resolves the
  argument as a uid (directory under `items/`) or a key (via the slug symlink only —
  no `memory_key` scan; the registry that re-keys safely is SL-008) and renders the
  record. `list [--type --status --tag]` reads
  each `items/*/memory.toml` and formats id/type/status/key/title rows — the
  AND-filtered metadata pipeline, **no scope matching or ranking yet** (those are
  SL-006). Reuses the pure-format / IO-seam split of `slice list`.

- **Install manifest split.** Replace the placeholder blanket
  `.doctrine/memory/*` gitignore with: create `.doctrine/memory/items`; gitignore
  `index/`, `embeddings/`, `state/` only — so authored memory items are committed,
  diffable, reviewable, while derived/ephemeral subtrees stay ignored
  (memory-spec § Install changes; install-spec § Manifest).

End state: a memory can be recorded, shown, and listed; it lives as a committed,
greppable, reviewable authored entity under the storage rule. The engine has now
hosted its **first string-identity, reservation-free caller**, so the reserved
seams (ledger, export, backend) and the value slices (retrieval, staleness, links)
attach to a proven entity rather than a theoretical one.

## Non-Goals

- **Scope retrieval and ranking** (`-p`/`-c`/`--match-tag`, the deterministic sort
  tuple). The retrieval primitive — the *point* of memory — is **SL-006**. v1
  `list` is metadata-filter only. Deferring it keeps this slice to the entity and
  its identity decision, exactly as slices-spec deferred everything past `new`/`list`.

- **Git anchoring and staleness.** No git context frame is built and no
  `verified_sha`/commit anchoring is computed; `record` writes `anchor_kind = none`
  and leaves the `[git]` block empty. Frame construction and the three-mode
  staleness computation are **SL-007**. This keeps all `git_context` work out of
  this slice (memory-spec § Scope & anchoring — unscoped/unanchored memory is
  permitted; repo-scoped anchoring arrives with its consumer).

- **Links, backlinks, relation registry.** `[[...]]` resolution, derived
  backlinks, and `[[relation]]` FK validation fold into the relation-index
  registry — **SL-008**. The `[[relation]]` rows may be *authored* in v1 but are
  inert (no resolution), the same posture as slice `[relationships]`.

- **The reserved seam.** Lifecycle ledger (`events.toml`), NDJSON import/export,
  the pluggable event-store backend adapter, and vector/graph retrieval are
  **designed but not built** (memory-spec § What v1 is, and what is reserved). v1
  writes no ledger; the current-state file is the sole authority.

- **`memory edit` / mutation verbs.** v1 records and reads. Lifecycle transitions
  (`supersede`/`retract`/`review`/`promote`) and tag edits are hand-edits in v1;
  the edit-preserving (`toml_edit`) mutating-verb surface arrives with the seam
  that consumes it (the ledger), not here — same staging as slice/drift, which
  also ship scaffold + read before any mutate verb.

- **Vocabulary lock.** `memory_type` and `status` members are **provisional**
  (memory-spec § Locked decisions): the model does not depend on exact membership,
  and the enums may be tuned before they harden.

## Summary

The first memory build step: a native current-state memory entity (`memory.toml` +
`memory.md` under the storage rule) with `record` / `show` / `list`, and the
install-manifest split that commits authored items while ignoring derived subtrees.
Its architectural core is the **entity engine's first string-identity, reservation-
free caller** — memory's UUID identity fits neither existing `MaterialiseMode`, so
the engine generalises (driven by memory, not speculatively) to materialise a
caller-named entity with no id allocation. Numeric callers stay behaviour-
preserving; their suites gate every step. Retrieval, git anchoring/staleness, links,
and the reserved ledger/export/backend seams are explicitly out — they attach to
this proven entity in later slices (SL-006/007/008 + the seam).

The engine-identity mechanism, the schema/parse design, the uid-minting seam, and
the manifest change live in the design doc ([design.md](design.md)) — authored with
this slice, pending adversarial review per the slice-002/003/004 rhythm.

## Follow-Ups

- **SL-006 — retrieval & ranking.** Scope matching (paths/globs/commands/tags, OR
  with specificity) and the deterministic sort tuple (memory-spec § Retrieval).
  The value slice; this entity is its substrate.
- **SL-007 — git anchoring & staleness.** Build the git context frame; compute the
  three-mode staleness; populate `[git]` on record/review. The first `git_context`
  work in doctrine.
- **SL-008 — links & registry.** `[[...]]` resolution, derived backlinks, and
  `[[relation]]` FK validation folded into the relation-index registry.
- **Reserved seam.** Lifecycle ledger → NDJSON export → pluggable backend adapter,
  each behind its own caller (memory-spec § Roadmap). The uid minted here is the
  idempotency anchor those seams key off.
- **Glossary.** Add the memory kind and its `mem.*` referends to
  [glossary.md](../../../doc/glossary.md) when this lands (memory-spec § Follow-ups).
- **Engine.** Track the string-identity / no-reservation placement as a capability
  of the shared engine, not a memory-specific fork — its next non-numeric caller
  (e.g. a future content-addressed artefact) inherits it.
