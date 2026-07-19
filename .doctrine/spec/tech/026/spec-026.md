# SPEC-026: Publication & library

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

> **Status — forward-intent (RFC-021 C1 / ADR-019).** Almost nothing described
> here is shipped. The publication manifest, the logical-address resolver, and the
> `doctrine library` surface are **greenfield**. Federated search is a
> **generalisation of the shipped entity-corpus search** (`src/search.rs`, today
> BM25 over entity kinds only) into a provider model — that code is the one live
> anchor; the federation, grouping, and envelope are target architecture. Every
> requirement below stands `pending` until an implementation slice lands it and
> reconciliation verifies coverage. Do not read this spec as current behaviour.

Publication & library is the container that **exposes** framework-owned material
without projecting it, and makes the whole corpus **discoverable** from one entry
point. It is the sibling of Install & distribution (SPEC-009): where SPEC-009
decides what minimal base fileset lands *in* a project, this decides what
framework-owned material is reachable *without* landing — the other half of the
embed/publish/project separation ADR-019 draws. Both ride the same compile-time
embed as their storage; neither owns it as a policy.

It realises PRD-017. Three mechanisms compose the container:

- **Publication policy** — a manifest, embedded and parsed at runtime, that
  declares the public set and each asset's exposure metadata. It is a *second,
  independent* policy over the same embedded bytes SPEC-009 projects from: one
  embed, two manifests, two orthogonal exposure decisions (project vs publish).
- **Library read surface** — `doctrine library list | tree | show`, a strictly
  read-only API over a logical-address namespace decoupled from physical storage.
- **Search federation** — a provider registry and coordinator that turns the
  current single-corpus entity search into one federated query across every
  corpus, grouped by source.

It sits beneath the whole-system root (SPEC-003), beside SPEC-009. It **uses**
the embed storage and the shared asset-read seam that SPEC-009 owns (generalised
from text-only `asset_text` to a byte-level seam), and it **uses** the memory
engine (SPEC-007) and entity search as federation providers. Those peer `uses`
edges are architectural but are authored as durable `spec interactions` edges by
the implementing slice, not asserted here — following SPEC-009's own posture with
its sibling edge. The CLI grammar and JSON-envelope conventions come from
SPEC-013; the composable-guidance corpus this publishes over is the hymn cascade
(SPEC-023).

Two boundaries are deliberate. **The library never materializes.** Semantic
materialization of a supported customization (a hymn override, an entity root)
stays with its owning feature (SPEC-009, SPEC-023); the library only reads and
emits bytes. **Publication is not activation.** Exposing a behaviour fragment's
bytes for inspection says nothing about whether that fragment is active
instruction or trusted project authority — those are RFC-021 C2/C3 and Stage 5,
explicitly downstream.

## Responsibilities

Mirrors the structured `responsibilities` list: own the publication manifest as a
policy independent of install's projection policy; map a stable logical-address
namespace onto physical storage through a resolver; serve the read-only
`library list | tree | show` surface over that resolver; derive each asset's
licence from a crisp manifest-encoded rule; generalise entity search into a
provider-federation model; assemble results grouped by corpus with native
semantics preserved and no cross-provider score comparison; and keep every
corpus's specialized query surface behind its native command.

### The publication manifest

A manifest — embedded and parsed at runtime exactly as SPEC-009's
`install/manifest.toml` is — is the sole authority for what is public. Each entry
binds a **logical address** to a **backing source** and carries the exposure
metadata: content kind, human title or summary, licence, and customization
status. An embedded asset absent from the manifest is not public, however
reachable it is in the embed tree; publication tracks the manifest, never the
embed root. This is the same manifest-as-policy shape SPEC-009 uses for
projection, applied to the orthogonal exposure axis — the two manifests are
independent surfaces even when they describe the same underlying bytes.

### Logical addressing and the resolver

Published material is addressed by a stable POSIX-like logical path independent of
where or how the asset is stored. A resolver maps each logical address to its
backing source — an embedded asset key today, a runtime-loaded path under a
development or external-package source tomorrow (the storage alternative ADR-019
names). The namespace is the contract; the physical layout is free to change
beneath it. Addressing exposes no embed key, crate path, or filesystem detail, so
a source-root reorganisation moves bytes without moving addresses.

### The read-only library surface

`doctrine library list [path]` and `tree [path]` read manifest metadata to
enumerate or nest the published entries at a logical path. `doctrine library show
<logical-path>` resolves the address and streams the asset's **raw bytes** to
stdout — clean, pipeable, no decorative framing in the content stream — through a
byte-level generalisation of SPEC-009's `asset_text` seam (text-only today;
binary-safe here). The surface has *no* mutating verb by construction: there is no
code path from a library command to a project write. An unknown or unpublished
address is refused; a published address whose backing source is missing is a
distinct, reported failure (a broken build/package, not a user error).

### Licence classification

Each manifest entry's licence is derived by a crisp, encoded rule rather than
chosen per asset: copyable authoring material (schemas, examples, editable
starters, selectors, declared seams) versus owned internal content (process
bodies, engine contracts, registry definitions, orchestration knowledge). The
rule is asserted at validate/build time so no asset can publish with an
unclassified licence, and a reader can predict an asset's licence from the rule
without inspecting its content.

### Search federation

A `SearchProvider`-style interface fronts each corpus; a federation coordinator
dispatches one query — carrying only arguments common to all providers — to every
registered provider and assembles their results **grouped by corpus**. The
shipped entity-corpus BM25 search becomes one provider; memory (SPEC-007) another;
the library another. Each group keeps its provider's native result semantics —
memory retains trust, severity, and staleness — and each hit stays resolvable
through that corpus's native `show`. Provider scores are not asserted mutually
comparable, so results are never flattened into one global ranking. Machine output
is one common outer envelope with typed per-corpus groups, per SPEC-013's
envelope conventions. A provider's rich, specialized query language stays behind
its native command; the federated entry point is discovery-only.

## Concerns

- **Read-only integrity is load-bearing.** The container's central safety claim
  is that no library operation mutates the project. This wants a structural
  guarantee (no write capability reachable from the surface), not merely the
  absence of a write call — so an audit can prove it once rather than per verb.
- **Provider isolation.** One unavailable or slow provider must degrade to a
  reported gap in its group, never fail or stall the whole federated search.
  Federation is fail-soft per provider.
- **Licence misclassification is a correctness risk, not a nicety.** Prose that
  looks like an editable template may encode owned process logic (RFC-021
  § "Licence boundaries can surprise"). A wrong classification invites copying
  under the wrong licence, so the rule must be auditable and defaulted
  conservatively (unclassifiable → owned, not copyable).
- **Address stability vs storage churn.** The logical namespace is a durable
  contract exposed to users who may script against it; the physical embed layout
  is expected to churn (ADR-019's source-root split). The resolver is the seam
  that absorbs that churn — a leaky address that encodes storage would break on
  the first reorganisation.
- **Search economy.** Federating every corpus on every bare query has a latency
  and token cost; the coordinator should bound per-provider work and let common
  arguments scope the federation without importing native query complexity.

## Hypotheses

- **A single logical namespace can address every published corpus.** Templates,
  hymns, reference, and scripts share one POSIX-like address space cleanly. If a
  corpus needs fundamentally different addressing, the resolver abstraction
  absorbs it; if many do, the single-namespace assumption is wrong and the
  manifest schema must carry per-collection addressing.
- **Publication metadata is small and static enough to embed in one manifest.**
  The public set changes at release cadence, not runtime, so a single embedded
  manifest (like the projection manifest) suffices — no index, no database.
- **The shipped entity search generalises to a provider without behavioural
  loss.** `src/search.rs` can be refactored behind the provider interface while
  its native entity-search surface stays byte-stable (the behaviour-preservation
  gate). If the abstraction distorts entity search, the provider boundary is
  drawn in the wrong place.
- **Licence classification is rule-decidable.** Every publishable asset falls on
  one side of the copyable/owned rule from manifest-declarable facts. If a
  material class needs human judgement per asset, the "crisp and auditable" claim
  fails and licence becomes an explicit per-entry authored field instead.

## Decisions

- **D1 — Two manifests, one embed.** Publication policy lives in a manifest
  distinct from SPEC-009's projection manifest. Projected and published are
  orthogonal exposure axes over the same embedded storage (ADR-019 positions 1–2);
  conflating them into one manifest would recreate the embed/install coupling C1
  exists to break. An asset may be projected, published, both, or neither.
- **D2 — Logical addressing decoupled from storage via a resolver.** The public
  contract is a stable logical namespace; a resolver maps it to the backing
  source. This is what makes publication storage-independent (embedded or
  runtime-loaded resolve the same address) and address-stable across the
  source-root reorganisation ADR-019 anticipates.
- **D3 — The library is read-only by construction.** No mutating verb, and no
  reachable write capability, on the library surface. The read path generalises
  SPEC-009's `asset_text` into a byte-level seam so binary assets emit faithfully;
  materialization stays with the owning feature, never the library.
- **D4 — Search is a provider registry, not a bigger search command.** Federation
  is an open set of corpus providers behind a common interface plus a coordinator,
  not special-cased corpora bolted onto entity search. Adding a corpus is adding a
  provider; existing providers are untouched (open/closed). The shipped entity
  search is refactored into the first provider under the behaviour-preservation
  gate.
- **D5 — Results grouped, never globally ranked.** Per-corpus grouping is the
  output contract; cross-provider scores are not comparable and are never merged
  into one ranking. Native trust/severity/staleness ride through each group. The
  machine envelope carries typed groups (SPEC-013), not a flat list.
- **D6 — Licence is rule-derived and conservatively defaulted.** Licence follows a
  crisp manifest-encoded classification asserted at validate/build; an asset that
  cannot be classified as copyable defaults to owned. This keeps the library
  predictable and auditable before it invites copying (RFC-021 licence-surprise
  principle).
- **D7 — Federation entry point is discovery-only.** The bare `search` surface
  accepts only arguments common to all providers; each provider's specialized
  query language stays behind its native command. This bounds the federated
  surface and preserves the rich native searches (`memory find`, entity `-k`).
