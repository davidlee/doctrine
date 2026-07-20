# Design SL-223: Publication seam

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Deliver the **publication seam** — the first implementation slice of RFC-021 C1 /
Contract B (PRD-017 product, SPEC-026 tech, ADR-019 root). Publication is the
declared exposure policy that says *what framework-owned material is public, how it
is addressed, and under what licence* — the source of truth both later surfaces
(the `library list|tree|show` read UX and federated search) consume. Neither can be
built until publication exists, so it is peeled off first.

This slice builds the **mechanism**: a runtime-parsed publication declaration that
is the sole authority for the public set (REQ-373 / REQ-359), a storage-independent
logical-address resolver over a source-adapter interface (REQ-374), a fail-closed
{MIT, GPL} licence field with recorded provenance (REQ-379 / REQ-367), and a
neutral byte-level asset-read seam extracted from install's text-only helper
(REQ-376 / D3). It deliberately ships **no `doctrine library` command** — the read
UX is the next slice — and verifies at the engine layer plus a build-time admission
gate (locked decision D-A below).

## 2. Current State

- **Asset reads are install-owned and text-only.** `src/install.rs` embeds the
  `install/` tree via `#[derive(RustEmbed)] #[folder = "install/"] struct Assets`.
  Two accessors sit over it: `asset_text(name) -> Result<String>` (`install.rs:861`,
  `from_utf8`, used by ~30 call sites across `slice.rs`, `spec.rs`, `backlog.rs`,
  `knowledge.rs`, `requirement.rs`, `standard.rs`, `concept_map.rs`) and
  `embedded_asset(rel) -> Option<Cow<[u8]>>` (a raw-byte accessor already present,
  but `pub(crate)` in and semantically owned by install). There is no neutral,
  non-install owner of asset bytes.
- **Projection is selective, not blanket.** Despite ADR-019's framing ("projects
  nearly every embedded file"), `install::run` projects a *selected* set through
  named forward steps — directories from `manifest.toml [dirs].create`, seeded
  memory items, hymn starter twins, agent defs, skills — not the whole embed tree.
  **Consequence for this design:** a new file under `install/` is embedded and
  runtime-readable but is *not* auto-projected into client repos. This is what lets
  the publication manifest live under the existing embed root without leaking into
  projection.
- **No publication anything.** The manifest, the logical-address resolver, and the
  library surface are greenfield. SPEC-026's only live anchor is entity search
  (`src/search.rs`), which is out of scope here (federation is a later slice).
- **Module layering (ADR-001)** is enforced leaf ← engine ← command, acyclic, with
  per-module tier assignments at `.doctrine/adr/001/layering.toml`.

## 3. Forces & Constraints

- **ADR-019 (root).** Seven independent asset properties; `published` is an
  *exposure* policy orthogonal to `embedded`/`runtime-loaded` (storage). Embedding
  is a storage default, never publication. `owned` is an ownership property, never
  a licence identifier.
- **PRD-017 invariants.** No library operation writes to the repo; every published
  asset has a declared licence and customization status; a logical address is
  stable across storage change; publication tracks the declaration, not the embed.
- **SPEC-026 decisions** (this slice realises D1, D2, D3, D6, D9; D4/D5/D7/D8/D10
  are federation/library, out of scope).
- **ADR-001 layering.** The byte-read is a leaf seam; declaration/admission/
  resolver are engine (pure-first); no command tier this slice.
- **STD-001.** Licence and customization vocabularies are closed enums / named
  constants, never free-text magic strings.
- **Behaviour-preservation gate** (AGENTS.md): changing the shared asset-read
  machinery must keep install's existing suites green *unchanged* — that is the
  proof the extraction is behaviour-preserving.
- **rust-embed re-embed footgun** (`mem.pattern.build.rust-embed-no-rerun`): a lone
  edit to an embedded asset does not re-embed on `cargo build`; `touch
  src/asset_source.rs` (the module holding the `#[folder]` derive after the move)
  forces recompile. Relevant to implementation + any test that reads the embedded
  manifest.
- **flake.nix / crane embed-strip** (AGENTS.md): crane's `cleanCargoSource` keeps
  only `.rs`/`.toml`/`.lock`; a *new* RustEmbed `#[folder]` root must be grafted
  into `srcWithDist` or the nix binary ships hollow. **This design keeps the
  manifest under the existing `install/` root precisely to avoid a new embed root
  and this graft.**
- **Spec-prose drift** (`mem.pattern.doctrine.spec-prose-requirement-drift`):
  mechanism nouns are tech-tier; this design (per-slice) is the correct home for
  them.

## 4. Guiding Principles

- **Publication is deliberate, not incidental.** Only manifest-declared assets are
  public; embedded-but-undeclared is invisible by construction.
- **The seam reads, never writes.** No mutating capability is reachable; read-only
  is structural (the engine holds read-only source capabilities only).
- **Licence fails closed.** An asset whose licence cannot be established from the
  allowed set fails admission — no runtime default, no guessed licence.
- **Address is the contract; storage is free.** The resolver is written against an
  adapter interface, so a source-root reorganisation moves bytes, never addresses.
- **Neutral ownership.** The byte-read seam is owned by neither projection nor
  publication, so publication is not born coupled to install's projection path.
- **Mechanism, not policy.** This slice earns its keep by proving the seam with a
  minimal, correctly-licensed corpus; *which* collections are ultimately published
  (OQ-1) is separable and deferred.

## 5. Proposed Design

### 5.1 System Model

Three components across two ADR-001 tiers; no command tier this slice.

```text
                 ┌────────────────────────── engine ──────────────────────────┐
  build-time     │  publication.rs                                             │
  test  ───────► │   PublicationManifest::load ──(admission: fail-closed)──►   │
                 │   Resolver { adapters }                                      │
  engine  ─────► │     resolve(LogicalAddress) ──► entry.backing ──►           │
  integ. tests   │       SourceAdapter::read(backing) ──► bytes                │
                 └───────────────────────────────┬─────────────────────────────┘
                                                 │ reads bytes through
                 ┌───────────────── leaf ────────▼─────────────────────────────┐
                 │  asset_source.rs   (owns `Assets` RustEmbed, moved here)     │
                 │   read_bytes(key) -> Option<Cow<[u8]>>   (binary-safe)       │
                 │   read_text(key)  -> Result<String>      (from_utf8 on top)  │
                 └───────────────────────────────▲─────────────────────────────┘
                                                 │ delegates (signature preserved)
                 ┌─────────────── command/engine (existing) ───────────────────┐
                 │  install.rs :: asset_text / embedded_asset  ──► delegate     │
                 └─────────────────────────────────────────────────────────────┘
```

- **`src/asset_source.rs` (new, leaf).** Owns `#[derive(RustEmbed)] #[folder =
  "install/"] struct Assets` (kept *private* to the module — the embed type does
  not leak across the seam), *relocated from `install.rs`*. Exposes
  `read_bytes(key) -> Option<Cow<'static, [u8]>>` (binary-safe), `read_text(key) ->
  anyhow::Result<String>` (`from_utf8` on `read_bytes`, preserving the current
  error text), and `iter() -> impl Iterator<Item = Cow<'static, str>>` (wrapping
  `Assets::iter()` for enumeration callers). This is the D3 neutral seam — owned by
  neither projection nor publication.
- **`src/publication.rs` (new, engine).** The manifest schema + admission, the
  `SourceAdapter` trait, the `EmbeddedAdapter`, and the `Resolver`. Pure-first: all
  IO is the leaf seam behind the adapter. `resolve(addr) -> Result<Cow<[u8]>>` is
  the byte-emit path a later `library show` wraps unchanged (decision D-A).
- **`install.rs`** keeps `asset_text` / `embedded_asset` signatures (delegating to
  `asset_source`), so their **external** callers (~30 sites across `slice.rs`,
  `spec.rs`, `backlog.rs`, etc.) are untouched. Install's own **internal** `Assets::`
  sites — `Assets::get` (manifest load `:1363`, hymn body `:1580`) and
  `Assets::iter()` (hymn/agent enumeration `:899,:914,:1433,:2191`) — rewire to the
  `asset_source` accessors. This is behaviour-preserving (same embed, same bytes);
  install's suites staying green unchanged is the proof.

### 5.2 Interfaces & Contracts

```rust
// ── leaf: src/asset_source.rs ────────────────────────────────────────────
pub(crate) fn read_bytes(key: &str) -> Option<std::borrow::Cow<'static, [u8]>>;
pub(crate) fn read_text(key: &str) -> anyhow::Result<String>;
pub(crate) fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>>;

// ── engine: src/publication.rs ───────────────────────────────────────────
pub(crate) struct PublicationManifest { entries: Vec<PublicationEntry> }

pub(crate) struct PublicationEntry {
    address: LogicalAddress,   // stable POSIX-like contract
    backing: String,           // physical embed key — decoupled from address
    kind: ContentKind,
    title: String,             // title-or-summary, non-empty
    licence: Licence,
    licence_provenance: LicenceProvenance,
    customization: CustomizationStatus,
}

pub(crate) enum Licence { Mit, Gpl }                 // D6 — fixed allowed set
pub(crate) enum LicenceProvenance { Declared, RuleSuggestedConfirmed }
pub(crate) enum CustomizationStatus { Customizable, Fixed }  // OQ-5 provisional

/// A stable POSIX-like logical path. Rejects absolute, empty, `.`/`..`, and
/// backslash segments at construction (traversal-safe by type).
pub(crate) struct LogicalAddress(String);

pub(crate) trait SourceAdapter {
    fn read(&self, backing: &str) -> Result<std::borrow::Cow<'static, [u8]>, ResolveError>;
}
pub(crate) struct EmbeddedAdapter;   // reads via asset_source::read_bytes

pub(crate) struct Resolver { /* address -> entry, + adapter registry */ }

impl PublicationManifest {
    /// Parse the embedded manifest and run fail-closed admission.
    pub(crate) fn load() -> Result<Self, AdmissionError>;
    /// Parse + admit an explicit TOML string (test seam; no embed dependency).
    pub(crate) fn admit(toml: &str) -> Result<Self, AdmissionError>;
}
impl Resolver {
    pub(crate) fn resolve(&self, addr: &LogicalAddress)
        -> Result<std::borrow::Cow<'static, [u8]>, ResolveError>;
}
```

Manifest on disk (`install/publication/manifest.toml`), mirroring the projection
manifest's TOML-policy shape but an **independent surface** (D1):

```toml
# SPDX-License-Identifier: MIT
[[entry]]
address       = "templates/slice.toml"   # logical contract
backing       = "templates/slice.toml"    # physical embed key (may differ)
kind          = "template"
title         = "Slice metadata template"
licence       = "MIT"
provenance    = "declared"
customization = "customizable"
```

### 5.3 Data, State & Ownership

- **Ownership.** `asset_source` owns the embed and the byte-read; `publication`
  owns the declaration schema, admission rules, address namespace, and resolver;
  `install` owns projection only and now *borrows* the byte-read. This is the D3
  neutral cut made concrete.
- **State.** All state is embedded-at-build (the manifest + the assets) and parsed
  read-only at runtime; no runtime mutation, no disk writes. Read-only is
  structural — no adapter or engine type exposes a write.
- **The address namespace** is the durable contract (`LogicalAddress`); `backing`
  is the mutable physical key. Keeping them separate fields (even when equal for
  templates today) is what makes storage-independence structural rather than
  coincidental.

### 5.4 Lifecycle, Operations & Dynamics

- **Admission (`load`/`admit`), fail-closed.** Parse TOML → validate each entry
  (all fields present, `title` non-empty, `licence ∈ {MIT, GPL}`, `customization ∈
  {customizable, fixed}`, `LogicalAddress` well-formed) → reject **duplicate
  logical addresses** (D9) → build the address→entry index. Any failure returns a
  typed `AdmissionError`; nothing is silently defaulted.
- **Resolution.** `resolve(addr)`: look up the entry (miss → `UnknownAddress`),
  read its bytes through the single `EmbeddedAdapter` (absent → `BackingSourceMissing`).
  Never a silent empty result. (Multi-adapter routing and `UnsupportedSourceType`
  arrive with the runtime-loaded adapter — see §5.5.)
- **Build-time gate.** A `#[test]` calls `PublicationManifest::load()` on the
  *shipped embedded* manifest and asserts it admits — so a mis-classified or
  unlicensed shipped asset fails CI (`just gate`). This is the "validate/build
  time" gate (REQ-379) without a CLI verb.

### 5.5 Invariants, Assumptions & Edge Cases

Invariants: (1) no reachable write over any protected root; (2) every admitted
entry carries licence ∈ {MIT, GPL} and a customization status; (3) a logical
address resolves independent of the physical layout; (4) an embedded-but-undeclared
asset is unreachable through the resolver.

Edge cases → typed errors **constructed this slice**: unknown/undeclared address →
`UnknownAddress`; duplicate address → `DuplicateAddress` at admission; malformed
TOML → `MalformedManifest`; missing field / out-of-set licence → `MissingField` /
`LicenceMissingOrOutOfSet`; absent backing bytes → `BackingSourceMissing`;
traversal-like address → rejected at `LogicalAddress` construction
(`TraversalRejected`). **Deferred** (would be dead code with one adapter):
`UnsupportedSourceType`, which lands with the runtime-loaded adapter (OQ-4). The
hymn-cascade overlay is **not** a duplicate (D9) — but this slice declares no hymns,
so no overlay arises here; the note is carried for the manifest schema's future.

## 6. Open Questions & Unknowns

- **OQ-1 (deferred, QUE).** Whether the shipped memory corpus is published-for-copy
  is *not* settled here. This slice declares templates only; the collection-set
  decision waits for the library slice that consumes it. Recorded as a knowledge
  QUE, linked to this slice.
- **OQ-5 (provisional, ASM).** `CustomizationStatus` ships as the closed enum
  `{customizable, fixed}` with the provisional meaning "intended to be customized".
  Its stable vocabulary is C2's; widening it is a controlled additive change.
- **Author-facing `publication validate` verb** — deferred. The build-time test
  covers CI; a real verb can land with the library slice if authors need it.
- **Runtime-loaded adapter (OQ-4)** — the `SourceAdapter` trait exists (that is what
  makes the resolver storage-independent), but only `EmbeddedAdapter` is
  implemented; a working runtime-loaded adapter is out of scope.

## 7. Decisions, Rationale & Alternatives

- **D-A — Verify at the engine layer + build-time gate; ship no `library` CLI verb,
  but build the real byte-emit consumer path.** `resolve(addr) -> bytes` is written
  and tested as the exact function `library show` will wrap; no command is
  registered. *Rationale:* resolves the speculative-generality risk (a real
  consumer shape drives the interface) while honouring the route's decision to defer
  the library UX. *Alternatives:* (a) engine-tests only — leaves the resolver
  interface unproven against a consumer; (c) ship a thin `show` verb — overlaps the
  library slice's boundary. (SPEC-026 read-surface, REQ-375, stays `pending`.)
- **D-B — Extract the neutral asset-source seam now (D3), minimally.** `asset_text`
  is refactored to delegate to a new leaf `asset_source` that owns the embed and a
  binary-safe byte read; **no physical file relocation** of published material.
  *Rationale:* keeps REQ-376's neutral-ownership criterion honestly met and stops
  publication being born coupled to projection; the file-relocation into
  semantically-owned roots (ADR-019 "exact names downstream") is genuinely later.
  *Alternative:* ride `embedded_asset` provisionally — ships REQ-376 unmet and hands
  the coupling to the next slice.
- **D-C — Manifest under the existing `install/` embed root
  (`install/publication/manifest.toml`), not a new root.** *Rationale:* install
  projects selectively, so the file is runtime-readable but not projected; reusing
  the root avoids a new RustEmbed `#[folder]` and the flake.nix `srcWithDist` graft
  footgun. *Alternative:* a dedicated `publication/` embed root — cleaner semantic
  ownership but pulls in the graft + `just nix-build` gate for no C1 benefit;
  recorded as a follow-up when relocation happens anyway.
- **D-D — Templates-only shipped declaration (OQ-1 deferred).** *Rationale:*
  mechanism/policy split; templates are the unambiguous MIT copyable case and give
  the licence gate real assets to check. *Alternative:* settle OQ-1 and enumerate
  the full set now — front-loads a licence-entangled classification with no
  consumer.
- **D-E — Licence and customization are closed enums (STD-001), licence fails
  closed (D6).** No free-text, no runtime default. Provenance recorded per entry
  for auditability.
- **D-F — Address and backing are separate fields; the resolver is written against
  `SourceAdapter` (D2).** Storage-independence is structural, proven by a test that
  resolves one address through a relocated backing / second adapter.

## 8. Risks & Mitigations

- **R1 — Extraction regresses install's asset reads.** *Mitigation:* signatures
  preserved; delegation only; install's existing suites must stay green unchanged
  (behaviour-preservation gate). Add a binary round-trip test the old text-only path
  could not express.
- **R2 — Seam with no CLI consumer builds the wrong abstraction.** *Mitigation:*
  D-A builds and tests the actual byte-emit consumer path; the interface is driven
  by a real shape, not speculation.
- **R3 — Stale embed masks manifest/asset changes in tests.** *Mitigation:* honour
  the rust-embed footgun — `touch src/asset_source.rs` before rebuild; the
  build-time admission test greps real bytes, never trusts `Finished`.
- **R4 — Manifest under `install/` read as projection-owned.** *Mitigation:* it is
  read via the neutral seam and parsed by the publication engine as an independent
  policy (D1); no projection forward-step references it, so it never lands in a
  client repo. Verified against `install::build_plan`'s selective projection.
- **R5 — Address/backing coincide for templates, so decoupling looks cosmetic.**
  *Mitigation:* a dedicated storage-independence test uses `address ≠ backing`
  (relocated key) to prove the resolver is not bound to one layout.

## 9. Quality Engineering & Validation

TDD red/green/refactor. Engine integration tests in `publication.rs`:

- valid manifest admits; declared address resolves to **exact bytes**;
- **undeclared** address → `UnknownAddress` (not silent empty);
- **duplicate** address → `DuplicateAddress` at admission;
- **missing / out-of-set licence** → admission fails (fail-closed);
- **missing customization / malformed TOML** → typed admission error;
- **storage independence** — same `LogicalAddress`, relocated `backing`, second
  (test) adapter → resolves identically (D2/REQ-374);
- **binary round-trip** — an in-memory test `SourceAdapter` holding non-UTF-8 bytes
  resolves byte-for-byte (proves the seam is `Cow<[u8]>`-clean, no `from_utf8`),
  without shipping a binary asset into the projection embed (REQ-376); a companion
  test asserts `read_text` and `read_bytes` agree on a real text asset;
- **traversal** addresses rejected at `LogicalAddress` construction.

Build-time / behaviour-preservation:

- shipped embedded manifest admits cleanly (all template entries MIT / customizable
  / declared);
- install's existing suites green **unchanged** (delegation is behaviour-
  preserving);
- `just gate` clean (clippy zero warnings), `cargo fmt`.

Requirement coverage recorded against REQ-373, REQ-374, REQ-376, REQ-379 (and the
PRD invariants REQ-359, REQ-363, REQ-367, REQ-369) at reconcile. REQ-375 (library
read surface), REQ-377/378 (federation), REQ-380 stay `pending` — later slices.

## 10. Review Notes

Internal adversarial pass + external codex review pending (project default).
Locked decisions D-A…D-F stand from the interactive design loop. Watch items for
review: (a) is engine-layer + build-time verification genuinely sufficient without
a CLI verb; (b) is reusing the `install/` embed root a semantic-ownership smell
worth the flake.nix cost to avoid; (c) does the `SourceAdapter` trait earn its keep
with a single adapter, or is it premature generality (D-A/D2 rationale: it is the
storage-independence contract REQ-374 demands, and a test exercises a second
adapter).
