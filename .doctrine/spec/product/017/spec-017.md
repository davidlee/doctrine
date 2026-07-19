# PRD-017: Library & search

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

Doctrine ships a large body of framework-owned material — entity and
configuration templates, composable guidance fragments, curated reference,
reusable scripts and workflow modules, the shipped memory corpus. Historically
the only way to get at any of it was for the installer to *project* it into the
client repo, because embedding and projection were one accidental policy
(ADR-019 § Context). RFC-021 / ADR-019 pull those policies apart: install now
provisions a **minimal, justified base** and leaves the rest framework-owned.
That is the right cut — but it removes something real. An asset that no longer
lands in the repo is an asset a user can no longer open, copy, fork, or consult
offline. Minimal projection, on its own, *reduces user control*.

The need is to **restore that control without reintroducing projection**: let a
human or agent see what framework-owned material exists, read any of it, and
copy the parts meant to be copied — while it continues to live nowhere in their
repository. The value is twofold. First, publication decouples "inspectable and
reusable" from "installed", so the project surface stays minimal while nothing
becomes hidden. Second, one **federated search** makes the whole corpus —
published library, memories, RFCs, entities — discoverable from a single entry
point, so a user does not have to know which store holds a thing before they can
find it.

This capability is the sibling PRD-006 (Install) explicitly depends on and
deliberately does not deliver: install owns *getting the justified base into a
project*; this owns *exposing everything install leaves behind, and making the
whole corpus discoverable*. ADR-019 is the shared root — its position that
**publication is independent of projection** (an asset can be public, auditable,
and copyable through a declared seam without being written into the project) is
the principle this capability realises.

## 2. Scope

In scope:

- **Publication** — a declared seam that exposes chosen framework-owned material
  for inspection and copy, without projecting it. Which assets are public, and
  the per-asset metadata that governs that exposure (stable logical address,
  content kind, title/summary, licence, customization status), is declared
  deliberately, not inherited from the embed root.
- **Library** — a read-only surface to enumerate, browse, and emit published
  material by stable logical address: list a path, show the tree, print an
  asset's bytes cleanly for piping.
- **Federated search** — a single search entry point that queries every
  discoverable corpus (published library, memory, RFCs, entities, and other
  registered providers), returning results grouped by corpus with each
  provider's own trust, severity, and staleness preserved.
- **Licence declaration and visibility** for published material — every
  published asset carries an explicit declared licence, drawn from a fixed
  allowed set (MIT for copyable authoring material, GPL for owned internal
  content — RFC-021), so a user knows what terms govern what they copy or adapt.

Out of scope:

- **Projecting or materializing** anything into a client repo — the library only
  reads and emits; semantic materialization of a supported customization stays
  with the owning feature (install, the hymn cascade), not here.
- **Deciding which physical source roots hold which material**, and the embed /
  runtime-load storage choice — physical storage policy is governed by ADR-019's
  semantic-ownership cut and the descending technical contracts, not by this
  capability. (SPEC-009 owns projection and today's embedded source; it does not
  own the whole publication storage choice.)
- **The specialized query languages** of individual corpora — federated search
  exposes only common discovery arguments; a corpus keeps its own rich query
  surface behind its native command (`memory find`, entity search).
- **Behaviour delivery, resolution, activation, and trust acceptance** — RFC-021
  C2/C3 and Stage 5. Publication exposes bytes for inspection; it does not make
  any published fragment *active instruction*, and it does not establish which
  repository-supplied content a user has accepted.
- **Addressing below whole-asset granularity** — C1 exposes whole published
  assets; addressable *fragments* within an asset (e.g. one hymn section) are a
  C2-triggered concern (RFC-021's exposed-fragment / composition model), not
  reintroduced here.
- **Migrating or cleaning up existing installs** (RFC-021 non-goal).

Boundary: this capability owns *what framework-owned material is public, how it
is addressed and read, and how the whole corpus is searched*. It does not own
what any published asset *does* once copied, nor whether it is projected, nor
whether it is trusted as instruction.

## 3. Principles

- **Publication is not projection.** A published asset is exposed for
  inspection and copy through a declared seam; it is never, by virtue of being
  published, written into a client repo. Visibility and residence are different
  properties (ADR-019 position 2).
- **Publication is deliberate, not incidental.** An asset is public because it
  is deliberately declared public — never because the binary happens to embed it.
  Embedded ≠ published; Doctrine-owned ≠ opaque.
- **Publication is independent of storage.** Whether an asset is stored one way
  or delivered another does not change whether, or how, it is published (ADR-019
  seven-property vocabulary).
- **Licence is declared and visible.** Every published asset carries an explicit
  declared licence, drawn from a fixed allowed set, that a user can read before
  copying — not a semantic category they must guess, and not an appearance that
  may mislead. Prose that looks copyable may still encode owned process logic; the
  declared licence, not appearances, governs, and an asset whose licence cannot be
  established is not published.
- **The library reads, it never writes.** No library operation mutates the
  project, installs a file, or overwrites anything. Emitting an asset is
  emitting bytes.
- **Search federates without flattening.** One entry point searches every
  corpus, but results stay grouped by their source and keep that source's trust
  and staleness semantics. Scores from different providers are not presented as
  globally comparable.

## 4. Requirements

The functional and quality requirements this capability must satisfy are
recorded as requirement entities and appear under the synthesized Requirements
section below. This section carries only the constraints and invariants that
bound every valid implementation.

Constraints:

- Publication status, licence, and customization status are the exposure
  properties from ADR-019's seven-property vocabulary; they must stay
  orthogonal to the storage/delivery properties (owned, versioned, distributed,
  embedded, runtime-loaded). No implementation may make publication a function
  of embedding.
- The federated search entry point must accept only discovery arguments common
  to every provider; a provider's specialized query syntax stays behind its
  native command.
- Logical addresses must be stable POSIX-like paths independent of physical
  embed or source-root layout, so a published asset's address survives a storage
  reorganisation.

Invariants:

- No library operation ever writes to, installs into, or overwrites the client
  repo.
- Every published asset has a declared licence and customization status; nothing
  is published with those unclassified.
- A published asset's logical address is stable across changes to how or where
  it is stored.
- Federated results are never presented as a single globally-ranked list;
  provider scores are not asserted to be mutually comparable.

## 5. Success Measures

- From a project with a minimal base install, a user can enumerate and read any
  framework-owned published asset without that asset existing anywhere in their
  repo, and without network access beyond the installed binary.
- Before copying an asset, a user can read its declared licence — one of the
  fixed allowed set — and its customization status directly, without inferring
  either from the asset's category or appearance.
- A single `search` over a plain query returns hits from more than one corpus,
  visibly grouped by corpus, each hit resolvable through that corpus's native
  show command.
- Removing every non-base file from a fresh install still leaves onboarding,
  inspection, and offline reference available through the library — the material
  a user can read and copy is preserved without projection. (Materializing a
  supported customization is its owning feature's job — SPEC-009 / the hymn
  cascade — not the library's; the complete Stage-1 user-control story is a
  cross-contract scenario, not the library's sole measure.)
- No library invocation changes a byte of the project; running the full library
  surface over a clean repo leaves it clean.

## 6. Behaviour

Primary flow — browse the library: a user asks for the published material at a
logical path; the system returns the declared entries there (list) or the subtree
beneath (tree), each with its title/summary, content kind, licence, and
customization status. Nothing is written.

Primary flow — read a published asset: a user names a logical address; the
system emits the asset published at that address as clean bytes to stdout,
suitable for piping or redirect. An unknown or unpublished address is refused,
not guessed at; an address that is published but whose backing material is
unavailable is a distinct, clearly reported failure.

Primary flow — federated search: a user issues one query with only common
discovery arguments; the system runs it across every registered corpus provider
and returns results grouped by corpus. Each group keeps its provider's native
semantics — memory hits retain trust, severity, and staleness; library hits
carry logical address and licence — and each hit remains retrievable through its
native show command. Machine-readable output wraps the groups in one common
envelope with typed result groups.

Alternate flow — scoped search: a user narrows the federation to specific
corpora through the common arguments; providers not selected are not queried.
Guard: narrowing is by common argument only — it never smuggles a provider's
specialized query language into the federated entry point.

Edge cases and failure modes: a query that matches nothing in any corpus reports
an empty grouped result, not an error. A provider that is unavailable is
reported as such without failing the whole search. A logical address that would
collide across independent sources is rejected at publication admission, not
silently resolved by precedence — the hymn cascade's own overlay is not such a
collision, its precedence being owned by the cascade (SPEC-023). An asset present
in a source but absent from the publication declaration is *not* published — it
is invisible to the library, by design, until it is declared public.

## 7. Verification

Verification confirms that publication exposes without projecting, that the
library only ever reads, that every published asset's licence is declared and
visible, and that search federates without flattening — without binding the spec
to a particular declaration schema or storage mechanism.

The non-projection invariant is proven by exercising the full library surface
(list, tree, show over a representative set of published addresses) against a
clean project and confirming not one byte of the repo changes. Publication
determinism is proven by confirming an asset absent from the publication
declaration is not reachable through the library, and that a declared asset is —
i.e. publication tracks the declaration, not the embed root. Storage independence
is proven by changing how or where a published asset is stored or delivered and
confirming its logical address and its publication are unchanged. Licence is
proven by confirming every published asset carries a declared licence from the
fixed allowed set, that the licence is visible in list, tree, and show before any
copy, that an asset whose licence cannot be established fails publication, and
that each entry records its licence provenance. Federation is proven by a query
that hits multiple corpora, asserting the results are grouped (not interleaved),
that memory hits retain trust/severity/staleness, and that the machine envelope
carries typed groups; and by an unavailable provider degrading to a reported gap
rather than a failed search.

Where a check must cite a specific obligation, it references the durable
requirement entity (REQ-NNN), never a mobile membership label. Coverage of the
functional and quality requirements is tracked against those entities, not
duplicated here.

## 8. Open Questions

- OQ-1 — Does the published library include the shipped **memory corpus** as a
  copyable collection, or only templates, hymns, curated reference, and reusable
  scripts/modules? The memory corpus is already exposed for *search* and
  *retrieval* through its own native surface; whether it is also *published for
  copy* through the library is a distinct classification decision. Blocks the
  published library's initial collection set and how far the licence
  classification reaches.
- OQ-2 — How far can a licence *classification rule* reach for material that
  *looks* like a copyable template but encodes owned process logic (RFC-021
  § "Licence boundaries can surprise")? Licence itself is not blocked on this —
  it is an explicit declared field that fails closed if unestablished — so this
  bounds only how much of the classification a rule can automate for authors
  before per-asset human judgement is needed.
- OQ-3 — Are logical addresses permanent, or versioned / deprecatable / allowed
  to alias after a source reorganisation? The namespace is a durable contract
  users may script against; its change policy is unspecified. Blocks the
  address-stability contract's edges.
- OQ-4 — When runtime-loaded backing sources are supported, how is an externally
  supplied publication package's version and integrity established before its
  assets are trusted as published? Blocks the runtime-loaded delivery story.
- OQ-5 — What is the allowed vocabulary of **customization status**, and what may
  a user infer from each value? Every published asset must carry the field, but
  its meaning is bound up with the supported-customization model (RFC-021 C2),
  which C1 does not deliver. Until C2 settles it, C1 declares the field with a
  minimal, provisional meaning (whether an asset is intended to be customized).
  Blocks the field's stable value set and its product interpretation.
