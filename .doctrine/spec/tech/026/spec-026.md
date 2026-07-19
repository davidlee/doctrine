# SPEC-026: Publication & library

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

> **Status — forward-intent (RFC-021 C1 / ADR-019).** Almost nothing described
> here is shipped. The publication manifest, the logical-address resolver, and the
> `doctrine library` surface are **greenfield**. Federated search is a
> **generalisation of the shipped entity-corpus search** into a provider model —
> the entity-search module (`doctrine::search`, `src/search.rs`, today BM25 over
> entity kinds only) is the one live anchor, and it anchors *only* that existing
> mechanism, not the greenfield container around it. The provider interface,
> coordinator, grouping, and envelope are target architecture. Every requirement
> below stands `pending` until an implementation slice lands it and reconciliation
> verifies coverage. Do not read this spec as current behaviour.

Publication & library is the container that **exposes** framework-owned material
without projecting it, and makes the whole corpus **discoverable** from one entry
point. It is the sibling of Install & distribution (SPEC-009): where SPEC-009
decides what minimal base fileset lands *in* a project, this decides what
framework-owned material is reachable *without* landing — the other half of the
embed/publish/project separation ADR-019 draws. Both ride the same compile-time
embed as their storage; neither owns it as a policy.

It realises PRD-017. Three mechanisms compose the container:

- **Publication policy** — a manifest (embedded by default, parsed at runtime)
  that declares the public set and each asset's exposure metadata. It is a
  *second, independent* policy that may describe the same bytes SPEC-009 projects
  from: one shared backing source, two independent manifests, two orthogonal
  exposure decisions (project vs publish). Embedding is the production default,
  not part of the contract.
- **Library read surface** — `doctrine library list | tree | show`, a strictly
  read-only API over a logical-address namespace decoupled from physical storage.
- **Search federation** — a provider registry and coordinator that turns the
  current single-corpus entity search into one federated query across every
  corpus, grouped by source.

It sits beneath the whole-system root (SPEC-003), beside SPEC-009. Its peer
dependencies are authored as durable `spec interactions` edges on this spec: it
**uses** the memory engine (SPEC-007) and entity search as federation providers,
and it **uses** SPEC-013's CLI grammar and JSON-envelope conventions. It also
**uses** the byte-level asset-read seam and embed storage today owned by SPEC-009
(generalised from text-only `asset_text`) — but that seam is being extracted into
a *neutral* asset-source layer owned by neither projection nor publication (D3);
the edge records the present dependency and D3 records the intended extraction, so
the graph is truthful now without baking in permanent coupling to install's
projection path. The composable-guidance corpus this publishes over is the hymn
cascade (SPEC-023).

Two boundaries are deliberate. **The library never materializes.** Semantic
materialization of a supported customization (a hymn override, an entity root)
stays with its owning feature (SPEC-009, SPEC-023); the library only reads and
emits bytes. **Publication is not activation.** Exposing a behaviour fragment's
bytes for inspection says nothing about whether that fragment is active
instruction or trusted project authority — those are RFC-021 C2/C3 and Stage 5,
explicitly downstream.

## Responsibilities

Mirrors the structured `responsibilities` list: own the publication manifest as a
policy independent of install's projection policy (sharing a backing source,
embedded by default); map a stable logical-address namespace onto backing storage
through a resolver over source adapters; serve the read-only
`library list | tree | show` surface over that resolver through a neutral
byte-level asset-read seam; carry each asset's licence as an explicit declared
field validated against an allowed policy (rule as an authoring aid only);
generalise entity search into a provider-federation model with a fixed common
request; assemble results grouped by corpus with native semantics preserved, a
typed terminal status per group, and no cross-provider score comparison; bound and
isolate each provider; and keep every corpus's specialized query surface behind
its native command.

### The publication manifest

A manifest — supplied with the tool (embedded by default) and parsed at
runtime — is the sole authority for what is public. Each entry binds a **logical
address** to a **backing source** and carries the exposure metadata: content
kind, human title or summary, licence, and customization status. (The
customization-status *vocabulary* — its allowed values and their meaning — is
settled with the supported-customization model of RFC-021 C2, which C1 does not
deliver; C1 carries the field with a minimal provisional meaning, PRD-017 OQ-5.)
An embedded
asset absent from the manifest is not public, however reachable it is in the
embed tree; publication tracks the manifest, never the embed root. This is the
same manifest-as-policy shape SPEC-009 uses for projection, applied to the
orthogonal exposure axis — the two manifests are independent surfaces even when
they describe the same underlying bytes, and the sole-authority contract does not
require embedding as the storage. A **duplicate logical address is rejected at
manifest admission**, not silently resolved by source precedence; the hymn
cascade's own overlay (SPEC-023), where an override deliberately shadows its base
at one address, is not such a collision — that precedence is the cascade's, owned
by SPEC-023, not the manifest's to arbitrate.

### Logical addressing and the resolver

Published material is addressed by a stable POSIX-like logical path independent of
where or how the asset is stored. A resolver maps each logical address to its
backing source **through a source adapter** — the embedded-asset adapter is the
production default; a runtime-loaded adapter (development or external-package
source, the storage alternative ADR-019 names) resolves the same addresses. The
resolver is written against the adapter interface, not one fixed storage type, so
storage independence is structural. The namespace is the contract; the physical
layout is free to change beneath it. Addressing exposes no embed key, crate path,
or filesystem detail, so a source-root reorganisation moves bytes without moving
addresses.

### The read-only library surface

`doctrine library list [path]` and `tree [path]` read manifest metadata to
enumerate or nest the published entries at a logical path. `doctrine library show
<logical-path>` resolves the address and streams the asset's **raw bytes** to
stdout — clean, pipeable, no decorative framing in the content stream — through a
**neutral byte-level asset-read seam** (generalising install's text-only
`asset_text`, binary-safe, owned by the shared asset-source layer, not the
library). The surface has *no* mutating verb by construction: read-only is
structural over every protected root (client project, runtime state, backing
sources), a capability the library holds rather than a write call it declines to
make (D3). Failures are distinct, reported error classes, never a silent empty
result: an unknown, unpublished, or traversal-like address is refused with a
stable stderr diagnostic and non-zero status; a published address whose backing
source is missing, whose bytes are absent though metadata is present, whose
publication policy is malformed, or whose source type is unsupported by the
running binary each reports its own class (a broken build/package, not a user
error).

### Licence classification

Each manifest entry's licence is an **explicit declared field**, validated at
validate/build time against the fixed allowed set **{MIT, GPL}** — MIT for
copyable authoring material (schemas, examples, editable starters, selectors,
declared seams), GPL for owned internal content (process bodies, engine
contracts, registry definitions, orchestration knowledge), per RFC-021. A
classification rule over that same copyable-vs-owned distinction *assists
authoring* by suggesting which of the two applies, but the declared field, not
the rule, is the source of truth; each entry records its provenance (declared, or
rule-suggested and confirmed) so the licence is auditable. An asset whose licence
cannot be established **fails publication at validate/build** rather than being
silently defaulted; because publication fails closed, no runtime licence fallback
exists (and `owned` — an ADR-019 *ownership* property — is never used as a
licence). Because licence fails closed, the open question of *how far the rule can
reach* (OQ-2 on the product spec) does not block publication — it only bounds how
much authoring the rule can automate.

### Search federation

A `SearchProvider`-style interface fronts each corpus; a federation coordinator
dispatches one query — carrying only the **fixed common request** (query text,
optional provider selection, per-provider result limit) — to every registered
provider and assembles their results **grouped by corpus**. The shipped entity-corpus BM25
search becomes one provider; memory (SPEC-007) another; the library another (the
library provider indexing published *metadata* — title/summary, kind, address,
licence — for C1; content indexing is a later option). Providers run **concurrently under one
federation-wide deadline**, each provider's errors and panics isolated, and each
group carries a **typed terminal status** — success-with-hits, success-empty,
unavailable, failed, or timed-out — so a slow or failing provider degrades to a
reported gap, never stalls or aborts the whole search; total search time is
bounded by the federation deadline (not the sum of per-provider deadlines), and
the coordinator returns the groups that completed once that deadline expires. Each group keeps its provider's native
result semantics — memory retains trust, severity, and staleness — and each hit
stays resolvable through that corpus's native `show`. Provider scores are not
asserted mutually comparable, so results are never flattened into one global
ranking. Machine output is one common outer envelope with typed per-corpus groups,
per SPEC-013's conventions, and partial success has a deterministic, documented
exit contract. A provider's rich, specialized query language stays behind its
native command; the federated entry point is discovery-only.

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
  under the wrong licence, so licence is an explicit declared field validated
  against an allowed policy, and an asset that cannot be classified **fails
  publication** rather than shipping under a guessed licence.
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
  loss.** `doctrine::search` can be refactored behind the provider interface
  while its observable entity-search contract — ranking order, kind filtering,
  human and machine output schema, empty-query behaviour, exit status — is
  preserved. Existing suites staying green is necessary but not sufficient
  evidence; gaps in that contract are covered before the refactor. If the
  abstraction distorts that contract, the provider boundary is drawn in the wrong
  place.
- **A classification rule can assist licence authoring for most assets.** Licence
  itself is an explicit declared field (D6) that fails closed, so this hypothesis
  is not load-bearing on correctness — it only governs how much of the
  copyable/owned classification a rule can automate before per-asset human
  judgement is needed (OQ-2). If the rule reaches little, authors declare more by
  hand; publication is unaffected either way.

## Decisions

- **D1 — Two independent manifests over a shared backing source.** Publication
  policy lives in a manifest distinct from SPEC-009's projection manifest.
  Projected and published are orthogonal exposure axes that *may share* one
  distributed backing source — embedded by default, not by contract (ADR-019
  positions 1–2); conflating them into one manifest, or fixing "one embed" as an
  invariant, would recreate the embed/install coupling C1 exists to break. An
  asset may be projected, published, both, or neither.
- **D2 — Logical addressing decoupled from storage via a resolver over adapters.**
  The public contract is a stable logical namespace; a resolver maps it to the
  backing source through a source adapter. Writing the resolver against the
  adapter interface (not one fixed storage type) is what makes publication
  storage-independent — embedded and runtime-loaded adapters resolve the same
  address — and address-stable across the source-root reorganisation ADR-019
  anticipates.
- **D3 — The library is read-only by construction, over a neutral asset-source
  seam.** No mutating verb and no reachable write capability over any protected
  root (client project, runtime state, backing sources); library-domain code
  holds read-only source capabilities only, so the guarantee is structural and
  provable once. The read path is a **neutral byte-level asset-read seam** —
  extracted from install's text-only `asset_text`, owned by the shared
  asset-source layer rather than by projection or publication — so binary assets
  emit faithfully and neither policy owns the other's storage. Materialization
  stays with the owning feature, never the library.
- **D4 — Search is a provider registry, not a bigger search command.** Federation
  is an open set of corpus providers behind a common interface plus a coordinator,
  not special-cased corpora bolted onto entity search. Adding a corpus needs only
  a provider implementation plus a registration declaration; existing providers
  and the coordinator's dispatch are untouched (open/closed). The shipped entity
  search is refactored into the first provider, preserving its observable contract
  (not merely keeping suites green).
- **D5 — Results grouped, never globally ranked.** Per-corpus grouping is the
  output contract; cross-provider scores are not comparable and are never merged
  into one ranking. Native trust/severity/staleness ride through each group. The
  machine envelope carries typed groups (SPEC-013), not a flat list.
- **D6 — Licence is an explicit declared field, validated against the allowed set
  {MIT, GPL}.** Licence is declared per manifest entry and validated at
  validate/build against the fixed allowed set **{MIT, GPL}** — MIT for copyable
  authoring material, GPL for owned internal content (RFC-021). Rule-based
  classification assists authoring but is not the source of truth, and provenance
  is recorded per entry. An asset whose licence cannot be established **fails
  publication at validate/build**; because publication fails closed there is no
  runtime licence fallback (`owned` is an ADR-019 ownership property, not a licence
  identifier, and is never used as one). This keeps the library auditable before
  it invites copying (RFC-021 licence-surprise principle) without resting the
  contract on an unproven inference rule.
- **D7 — Federation entry point is discovery-only.** The bare `search` surface
  accepts only arguments common to all providers; each provider's specialized
  query language stays behind its native command. This bounds the federated
  surface and preserves the rich native searches (`memory find`, entity `-k`).
- **D8 — The federation request and response contracts are fixed and typed.** The
  common request is exactly query text, optional provider selection, and a
  **per-provider** result limit (each group is independently ranked, so a single
  federation-wide cap across incomparable scores would be incoherent). Providers
  run **concurrently under one federation-wide deadline**, each provider's errors
  and panics isolated, so total search time is bounded by that deadline, not the
  sum of per-provider deadlines; each response group carries a typed terminal
  status (success-with-hits, success-empty, unavailable, failed, timed-out); the
  machine envelope is one outer envelope of typed groups; and partial success has a
  deterministic, documented exit contract. This is what makes fail-soft
  (D-concern) falsifiable rather than aspirational.
- **D9 — Duplicate logical addresses are rejected, not silently resolved.**
  Collisions across independent sources are rejected at manifest admission so
  publication never depends on source ordering. The hymn cascade's overlay
  precedence (SPEC-023) — an override deliberately shadowing its base at one
  address — is not a collision; it is resolved by the cascade, which owns that
  precedence, not by the publication manifest.
- **D10 — Library search indexes published metadata for C1.** The library
  provider indexes title/summary, kind, address, and licence — not asset content.
  Content indexing is a deferred option, not a C1 obligation. There is no secrecy
  boundary at stake (every published asset is open-source licensed under the
  declared set — D6); the library corpus is simply the manifest, so unpublished
  bytes are out of search by construction, not by a guard.
