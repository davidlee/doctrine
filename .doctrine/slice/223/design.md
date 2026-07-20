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
UX is the next slice — and verifies at the engine layer via a command-free
`emit(addr, writer)` consumer path plus a source-level admission gate (locked
decision D-A below).

## 2. Current State

- **Asset reads are install-owned and text-only.** `src/install.rs` embeds the
  `install/` tree via `#[derive(RustEmbed)] #[folder = "install/"] struct Assets`.
  Two accessors sit over it: `asset_text(name) -> Result<String>` (`install.rs:861`,
  `from_utf8`, used by ~30 call sites across `slice.rs`, `spec.rs`, `backlog.rs`,
  `knowledge.rs`, `requirement.rs`, `standard.rs`, `concept_map.rs`) and
  `embedded_asset(rel) -> Option<Cow<[u8]>>` (a raw-byte accessor already present,
  but `pub(crate)` in and semantically owned by install). There is no neutral,
  non-install owner of asset bytes.
- **Projection is BLANKET, not selective.** `build_plan` step 2
  (`src/install.rs:1395–1410`) installs **every** embedded file returned by
  `embedded_filenames()` (`:1431–1438`), which excludes only *root* `manifest.toml`.
  ADR-019's "projects nearly every embedded file" is literal. **Consequence for this
  design (corrected after codex RV-286 finding 1):** a new file placed anywhere
  under the `install/` embed root — including a nested `install/publication/…` —
  **is** projected into `.doctrine/…` of the client repo. Reusing the `install/`
  root for the publication manifest would therefore project it and violate ADR-019 /
  PRD-017 / REQ-380. The manifest must live in a **separate embed root** that
  `build_plan` never walks (see D-C).
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
- **Requirement surface.** In scope: REQ-373 (sole authority), REQ-374 (resolver),
  REQ-376 (neutral seam), REQ-379 (licence), REQ-380 (manifest independence, NF-001),
  and the **seam foundation** of REQ-381 (read-only structural, NF-002). Out of
  scope: REQ-375 (library verbs), REQ-377/378 (federation), REQ-382/383 (federation
  NF-003/004).
- **ADR-001 layering.** The byte-read + embed ownership is a leaf seam;
  declaration/admission/resolver/emit are engine (pure-first); no command tier this
  slice.
- **STD-001.** Licence and customization vocabularies are closed enums / named
  constants, never free-text magic strings.
- **Behaviour-preservation gate** (AGENTS.md): changing the shared asset-read
  machinery must keep install's existing suites green *unchanged* — that is the
  proof the extraction is behaviour-preserving.
- **rust-embed re-embed footgun** (`mem.pattern.build.rust-embed-no-rerun`): a lone
  edit to an embedded asset does not re-embed on `cargo build`; `touch
  src/asset_source.rs` (the module holding the `#[folder]` derives after the move)
  forces recompile. The admission gate therefore validates the manifest **from disk
  source**, not embedded bytes, so staleness cannot mask a defect (§5.4).
- **flake.nix / crane embed-strip** (AGENTS.md): crane's `cleanCargoSource` keeps
  only `.rs`/`.toml`/`.lock`; a *new* RustEmbed `#[folder]` root must be grafted
  into `srcWithDist` or the nix binary ships hollow. The new `publication/` root
  (D-C) **must** be grafted, validated with `just nix-build` (host-only —
  `doCheck = false`, so nix skips tests; the graft check is presence-of-asset, and
  the authoritative admission gate runs in `just gate`/CI, which does run tests).
- **Spec-prose drift** (`mem.pattern.doctrine.spec-prose-requirement-drift`):
  mechanism nouns are tech-tier; this design (per-slice) is the correct home for
  them.

## 4. Guiding Principles

- **Publication is deliberate, not incidental.** Only manifest-declared assets are
  public; embedded-but-undeclared is invisible by construction.
- **The seam reads, never writes.** The read path exposes no write capability; the
  `SourceAdapter` trait has only `read`, and the sole `EmbeddedAdapter` reads
  immutable compiled bytes by key — no filesystem path, no mutation reachable.
- **Licence fails closed.** An asset whose licence cannot be established from the
  allowed set fails admission — no runtime default, no guessed licence.
- **Address is the contract; storage is free.** The resolver is written against an
  adapter interface, so a source-root reorganisation moves bytes, never addresses.
- **Neutral ownership.** The byte-read seam is owned by neither projection nor
  publication, so publication is not born coupled to install's projection path.
- **Publication ≠ projection, structurally.** The publication manifest lives in its
  own embed root that install's projection never walks (REQ-380).
- **Mechanism, not policy.** This slice earns its keep by proving the seam with a
  minimal, correctly-licensed corpus; *which* collections are ultimately published
  (OQ-1) is separable and deferred.

## 5. Proposed Design

### 5.1 System Model

Components across two ADR-001 tiers; no command tier this slice.

```text
                 ┌────────────────────────── engine ──────────────────────────┐
  source-level   │  publication.rs                                             │
  admission ───► │   PublicationManifest::admit(bytes) ─(fail-closed)─►        │
  gate (disk)    │   PublicationManifest::load()  ── embedded convenience ─┐   │
                 │   Resolver { manifest, EmbeddedAdapter }                │   │
  engine  ─────► │     resolve(LogicalAddress) ─► entry.backing ─►         │   │
  integ. tests   │       EmbeddedAdapter::read(backing) ─► bytes           │   │
                 │     emit(LogicalAddress, &mut dyn Write) ─► framing-free │   │
                 └───────┬───────────────────────────────────────────┬─────────┘
             reads       │ manifest bytes            template bytes    │
             through     ▼                                            ▼
                 ┌───────────────── leaf: src/asset_source.rs ──────────────────┐
                 │  PublicationAssets  (#[folder="publication/"])  manifest      │
                 │  InstallAssets      (#[folder="install/"])      general seam  │
                 │   read_bytes(key)/read_text(key)/iter()  (over InstallAssets) │
                 │   publication_manifest_bytes()           (over PublicationA.) │
                 └───────────────────────────────▲─────────────────────────────┘
                                                 │ delegates (signature preserved)
                 ┌─────────────── install.rs (existing) ───────────────────────┐
                 │  asset_text / embedded_asset ─► asset_source (InstallAssets) │
                 │  build_plan walks InstallAssets ONLY — never PublicationA.   │
                 └─────────────────────────────────────────────────────────────┘
```

- **`src/asset_source.rs` (new, leaf).** Owns **two** private RustEmbed roots:
  `InstallAssets` (`#[folder = "install/"]`, relocated from `install.rs`) and
  `PublicationAssets` (`#[folder = "publication/"]`, new). Both structs stay private
  to the module — the embed type does not leak across the seam. Exposes over
  InstallAssets: `read_bytes(key) -> Option<Cow<'static, [u8]>>` (binary-safe),
  `read_text(key) -> anyhow::Result<String>` (`from_utf8`, preserving current error
  text), `iter() -> impl Iterator<Item = Cow<'static, str>>`; and over
  PublicationAssets: `publication_manifest_bytes() -> Option<Cow<'static, [u8]>>`.
  This is the D3 neutral seam — owned by neither projection nor publication.
- **`src/publication.rs` (new, engine).** The manifest schema + source-agnostic
  admission, the `SourceAdapter` trait, the one `EmbeddedAdapter`, the `Resolver`,
  and `emit`. Pure-first: all IO is the leaf seam behind the adapter.
- **`install.rs`** keeps `asset_text` / `embedded_asset` signatures (delegating to
  `asset_source`), so their **external** callers (~30 sites across `slice.rs`,
  `spec.rs`, `backlog.rs`, etc.) are untouched. Install's own **internal** `Assets::`
  sites — `Assets::get` (manifest load `:1363`, hymn body `:1580`) and
  `Assets::iter()` (hymn/agent enumeration `:899,:914,:1433,:2191`) — rewire to the
  `asset_source` accessors. Behaviour-preserving (same embed, same bytes); install's
  suites staying green unchanged is the proof. `build_plan`/`embedded_filenames`
  continue to walk **only** `InstallAssets` — never `PublicationAssets` — so the
  manifest is structurally outside projection.

### 5.2 Interfaces & Contracts

```rust
// ── leaf: src/asset_source.rs ────────────────────────────────────────────
pub(crate) fn read_bytes(key: &str) -> Option<std::borrow::Cow<'static, [u8]>>;
pub(crate) fn read_text(key: &str) -> anyhow::Result<String>;
pub(crate) fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>>;
pub(crate) fn publication_manifest_bytes() -> Option<std::borrow::Cow<'static, [u8]>>;

// ── engine: src/publication.rs ───────────────────────────────────────────
pub(crate) struct PublicationManifest { entries: Vec<PublicationEntry> }

pub(crate) struct PublicationEntry {
    address: LogicalAddress,   // stable POSIX-like contract
    backing: String,           // physical embed key — decoupled from address
    kind: ContentKind,
    title: String,             // title-or-summary, non-empty
    licence: Licence,
    provenance: LicenceProvenance,   // TOML + Rust field name unified (F7)
    customization: CustomizationStatus,
}

#[derive(Deserialize)] #[serde(rename_all = "UPPERCASE")]  // MIT / GPL in TOML
pub(crate) enum Licence { Mit, Gpl }                        // D6 — fixed allowed set
#[derive(Deserialize)] #[serde(rename_all = "kebab-case")]
pub(crate) enum LicenceProvenance { Declared, RuleSuggestedConfirmed }
#[derive(Deserialize)] #[serde(rename_all = "kebab-case")]
pub(crate) enum CustomizationStatus { Customizable, Fixed } // OQ-5 provisional

/// A stable POSIX-like logical path. Rejects absolute, empty, `.`/`..`, and
/// backslash segments at construction (traversal-safe by type).
pub(crate) struct LogicalAddress(String);

/// Read-only by construction: the ONLY method is `read`; no write capability.
pub(crate) trait SourceAdapter {
    fn read(&self, backing: &str) -> Result<std::borrow::Cow<'static, [u8]>, ResolveError>;
}
pub(crate) struct EmbeddedAdapter;   // reads via asset_source::read_bytes (embed key only)

/// Holds the admitted manifest + exactly one injected adapter this slice. NOT a
/// multi-source registry — that arrives with the runtime-loaded adapter (OQ-4).
pub(crate) struct Resolver { manifest: PublicationManifest, source: EmbeddedAdapter }

impl PublicationManifest {
    /// Source-AGNOSTIC admission — the sole-authority contract (REQ-373) is a
    /// property of these bytes, not of where they came from. Any manifest source
    /// (embedded now; runtime-loaded later, OQ-4) feeds the same admission.
    pub(crate) fn admit(bytes: &[u8]) -> Result<Self, AdmissionError>;
    /// Embedded convenience: read the shipped manifest and admit it.
    pub(crate) fn load() -> Result<Self, AdmissionError>;  // = admit(publication_manifest_bytes()?)
}
impl Resolver {
    pub(crate) fn resolve(&self, addr: &LogicalAddress)
        -> Result<std::borrow::Cow<'static, [u8]>, ResolveError>;
    /// The real consumer shape a later `library show` wraps: stream the resolved
    /// asset's bytes framing-free to any writer, propagating IO errors (D-A/F6).
    pub(crate) fn emit(&self, addr: &LogicalAddress, out: &mut dyn std::io::Write)
        -> Result<(), EmitError>;
}
```

Manifest on disk at the **new** root `publication/manifest.toml` (repo root, its own
embed — an **independent surface** from install's projection manifest, D1/REQ-380):

```toml
# SPDX-License-Identifier: MIT
[[entry]]
address       = "templates/slice.toml"   # logical contract
backing       = "templates/slice.toml"    # physical InstallAssets key (may differ)
kind          = "template"
title         = "Slice metadata template"
licence       = "MIT"
provenance    = "declared"
customization = "customizable"
```

Note the backing key references the `install/` embed (templates stay physically at
`install/templates/` — D-B, no relocation); the manifest itself lives in the
`publication/` root. Two roots, both leaf-owned.

### 5.3 Data, State & Ownership

- **Ownership.** `asset_source` owns both embeds and the byte-read; `publication`
  owns the declaration schema, admission rules, address namespace, resolver, and
  emit; `install` owns projection only and now *borrows* the InstallAssets byte-read.
  This is the D3 neutral cut made concrete.
- **State.** All state is embedded-at-build (the two embeds) and parsed read-only at
  runtime; no runtime mutation, no disk writes.
- **Read-only is structural at the seam (REQ-381 foundation, not full).** The
  `SourceAdapter` trait exposes only `read`; `EmbeddedAdapter` reads immutable
  compiled bytes addressed by embed key — it takes no filesystem path and holds no
  write handle, so no code path from resolve/emit to a mutation exists *by
  construction*. The **full** REQ-381 obligation (the entire library surface, incl.
  failure paths, leaves every protected root byte-for-byte unchanged) is the library
  slice's to close once the verbs exist; this slice proves the seam foundation and
  that resolve+emit over a clean repo write nothing (§9). A future disk-backed
  adapter (OQ-4) must preserve the structural property — noted as its obligation.
- **The address namespace** is the durable contract (`LogicalAddress`); `backing`
  is the mutable physical key. Keeping them separate fields (even when equal for
  templates today) is what makes storage-independence structural, not coincidental.

### 5.4 Lifecycle, Operations & Dynamics

- **Admission (`admit(bytes)`), fail-closed, source-agnostic.** Parse TOML →
  validate each entry (all fields present, `title` non-empty, `licence ∈ {MIT, GPL}`,
  `provenance` a known value, `customization ∈ {customizable, fixed}`,
  `LogicalAddress` well-formed) → reject **duplicate logical addresses** (D9) → build
  the address→entry index. Any failure returns a typed `AdmissionError`; nothing is
  silently defaulted. `load()` is the embedded convenience over `admit`.
- **Resolution.** `resolve(addr)`: look up the entry (miss → `UnknownAddress`), read
  its bytes through the single `EmbeddedAdapter` (absent → `BackingSourceMissing`).
  `emit(addr, writer)` resolves then streams the bytes framing-free, propagating a
  writer IO error as `EmitError::Io`. Never a silent empty result. (Multi-adapter
  routing and `UnsupportedSourceType` arrive with the runtime-loaded adapter, §5.5.)
- **Admission gate (REQ-379), staleness-proof.** The authoritative gate is a test
  that reads `publication/manifest.toml` **from disk** (`env!("CARGO_MANIFEST_DIR")`)
  and runs `admit` on those bytes — so a mis-classified or unlicensed shipped asset
  fails `just gate` / CI *regardless of embed staleness* (the rust-embed footgun
  cannot mask it). A companion test runs `load()` on the embedded bytes to prove the
  shipped embed itself admits. `just nix-build` (host-only) confirms the
  `publication/` graft landed in the release artifact (nix `doCheck = false` skips
  tests, so the graft check is presence-of-asset).

### 5.5 Invariants, Assumptions & Edge Cases

Invariants: (1) no write capability reachable from resolve/emit (structural, §5.3);
(2) every admitted entry carries licence ∈ {MIT, GPL} and a customization status;
(3) a logical address resolves independent of the physical layout; (4) an
embedded-but-undeclared asset is unreachable through the resolver; (5) the
publication manifest is never projected into a client repo (structural — outside the
`InstallAssets` root `build_plan` walks).

Edge cases → typed errors **constructed this slice**: unknown/undeclared address →
`UnknownAddress`; duplicate address → `DuplicateAddress` at admission; malformed
TOML → `MalformedManifest`; missing field / out-of-set licence → `MissingField` /
`LicenceMissingOrOutOfSet`; unknown provenance value → `MissingField` (fail-closed);
absent backing bytes → `BackingSourceMissing`; writer IO failure → `EmitError::Io`;
traversal-like address → rejected at `LogicalAddress` construction
(`TraversalRejected`). **Deferred** (would be dead code with one adapter):
`UnsupportedSourceType` + a per-entry `source` discriminator, both additive when the
runtime-loaded adapter (OQ-4) lands. The hymn-cascade overlay is **not** a duplicate
(D9) — but this slice declares no hymns, so no overlay arises here; the note is
carried for the manifest schema's future.

## 6. Open Questions & Unknowns

- **OQ-1 (deferred, [[QUE-172]]).** Whether the shipped memory corpus is
  published-for-copy is *not* settled here. Templates-only this slice; the
  collection-set decision waits for the library slice that consumes it.
- **OQ-5 (provisional, [[ASM-003]]).** `CustomizationStatus` ships as the closed
  enum `{customizable, fixed}` with the provisional meaning "intended to be
  customized". Its stable vocabulary is C2's; widening it is a controlled additive
  change.
- **REQ-373 runtime-loaded coverage is partial.** Admission is source-agnostic
  (`admit(bytes)`), so the sole-authority contract holds for *any* source by
  construction; but only the **embedded** source is wired this slice. A working
  runtime-loaded manifest source is deferred (OQ-4). Coverage recorded as
  embedded-covered / runtime-loaded pending at reconcile.
- **Author-facing `publication validate` verb** — deferred. The disk-source
  admission gate covers CI; a real verb can land with the library slice if authors
  need it.
- **Runtime-loaded adapter (OQ-4)** — the `SourceAdapter` trait + source-agnostic
  `admit` exist (that is what makes the resolver and admission storage-independent),
  but only `EmbeddedAdapter` / the embedded manifest source are implemented.

## 7. Decisions, Rationale & Alternatives

- **D-A — Verify at the engine layer via a command-free `emit` consumer path +
  source-level admission gate; ship no `library` CLI verb.** `emit(addr, &mut dyn
  Write)` streams framing-free bytes and propagates IO errors — the exact shape a
  later `library show` wraps, minus CLI parsing — so the seam is proven against a
  *real* consumer, not merely asserted-on by `resolve`. *Rationale:* resolves the
  speculative-generality risk while honouring the route's decision to defer the
  library UX. *Alternatives:* (a) test `resolve()->bytes` only — still an engine
  assertion, proves no framing-free emission or output-failure handling (codex F6);
  (c) ship a thin `show` verb — overlaps the library slice's boundary. (REQ-375
  stays `pending`.)
- **D-B — Extract the neutral asset-source seam now (D3), minimally.** `asset_text`
  delegates to a new leaf `asset_source` owning the embed and a binary-safe byte
  read; **no physical file relocation** of published material. *Rationale:* keeps
  REQ-376's neutral-ownership criterion honestly met and stops publication being born
  coupled to projection; file-relocation into semantically-owned roots (ADR-019
  "exact names downstream") is genuinely later. *Alternative:* ride `embedded_asset`
  provisionally — ships REQ-376 unmet.
- **D-C — Manifest lives in a NEW `publication/` embed root, not under `install/`
  (revised after codex RV-286 F1).** *Rationale:* `build_plan` projects the entire
  `install/` embed except root `manifest.toml` (`install.rs:1395–1438`), so a
  manifest under `install/` **would** project into `.doctrine/publication/…` —
  violating ADR-019 / PRD-017 / REQ-380. A separate root that `build_plan` never
  walks makes non-projection *structural* and directly serves REQ-380 (manifest
  independence). *Cost accepted:* the flake.nix `srcWithDist` graft + `just
  nix-build` presence check (the crane embed-strip footgun). *Alternative rejected:*
  keep the manifest under `install/` and add an exclusion to install's projection
  filter — couples publication into install's projection policy, the exact coupling
  REQ-380 / D1 forbid, and is fragile.
- **D-D — Templates-only shipped declaration (OQ-1 deferred).** *Rationale:*
  mechanism/policy split; templates are the unambiguous MIT copyable case and give
  the licence gate real assets to check. *Alternative:* settle OQ-1 and enumerate
  the full set now — front-loads a licence-entangled classification with no consumer.
- **D-E — Licence and customization are closed enums (STD-001), licence fails closed
  (D6).** No free-text, no runtime default. Provenance recorded per entry (field name
  unified as `provenance` across TOML + Rust, codex F7) for auditability.
- **D-F — Address and backing are separate fields; the resolver is written against
  `SourceAdapter` with exactly one injected `EmbeddedAdapter` this slice — not a
  multi-source registry (codex F3).** Storage-independence is structural, proven by a
  test resolving one address through a relocated backing / second (in-memory) adapter
  (D2/REQ-374). The trait earns its keep as the REQ-374 storage-independence contract,
  exercised by that second-adapter test; source-identity dispatch is deferred (OQ-4).

## 8. Risks & Mitigations

- **R1 — Extraction regresses install's asset reads.** *Mitigation:* signatures
  preserved; delegation only; install's existing suites must stay green unchanged
  (behaviour-preservation gate). Add a binary round-trip test the old text-only path
  could not express.
- **R2 — Seam with no CLI consumer builds the wrong abstraction.** *Mitigation:*
  D-A's `emit(addr, writer)` is the real consumer path (framing-free stream + IO
  error), driving the interface from a real shape, not speculation.
- **R3 — Stale embed masks manifest/asset changes in tests.** *Mitigation:* the
  authoritative admission gate reads the manifest **from disk source**, not embedded
  bytes (§5.4), so staleness cannot hide a defect; `touch src/asset_source.rs` before
  any rebuild that must re-embed; never trust `Finished`.
- **R4 — Publication manifest leaks into projection (codex F1, was a real defect).**
  *Mitigation:* the manifest lives in the separate `publication/` embed root;
  `build_plan`/`embedded_filenames` walk only `InstallAssets`. A regression test
  installs into a clean temp repo and asserts `.doctrine/publication/` is **absent**
  (REQ-380 + non-projection, structural).
- **R5 — Address/backing coincide for templates, so decoupling looks cosmetic.**
  *Mitigation:* a dedicated storage-independence test uses `address ≠ backing`
  (relocated key) to prove the resolver is not bound to one layout.
- **R6 — New `publication/` embed root ships hollow under nix (crane strip).**
  *Mitigation:* graft `publication/` into `srcWithDist`; `just nix-build` (host-only)
  asserts the manifest asset is present in the release binary; the CI admission gate
  (`just gate`, runs tests) is the authoritative licence check since nix
  `doCheck = false`.

## 9. Quality Engineering & Validation

TDD red/green/refactor. Engine tests in `publication.rs`:

- valid manifest admits; declared address resolves to **exact bytes**;
- `emit(addr, writer)` streams **framing-free** bytes byte-for-byte; a failing writer
  surfaces `EmitError::Io` (D-A/F6 — the real consumer path + output-failure handling);
- **undeclared** address → `UnknownAddress` (not silent empty);
- **duplicate** address → `DuplicateAddress` at admission;
- **missing / out-of-set licence** → admission fails (fail-closed);
- **missing / unknown provenance** → typed admission error (F7);
- **missing customization / malformed TOML** → typed admission error;
- **storage independence** — same `LogicalAddress`, relocated `backing`, second
  (in-memory) adapter → resolves identically (D2/REQ-374);
- **binary round-trip** — an in-memory test `SourceAdapter` holding non-UTF-8 bytes
  emits byte-for-byte (proves the seam is `Cow<[u8]>`-clean, no `from_utf8`), without
  shipping a binary asset into any embed (REQ-376); companion: `read_text` and
  `read_bytes` agree on a real text asset;
- **traversal** addresses rejected at `LogicalAddress` construction;
- **no-write** — `load()` + `resolve` + `emit` over a clean temp repo leave every
  path byte-for-byte unchanged (REQ-381 seam foundation).

Install-integration + build gate:

- **non-projection regression** — `install` into a clean temp repo leaves
  `.doctrine/publication/` **absent**; the projection and publication manifests are
  independent surfaces (REQ-380 / codex F1);
- **source-level admission gate** — `admit` over the on-disk `publication/manifest.toml`
  passes (all template entries MIT / customizable / declared); staleness-proof (REQ-379);
- **embedded admission** — `load()` over the shipped embed admits;
- install's existing suites green **unchanged** (delegation is behaviour-preserving);
- `just gate` clean (clippy zero warnings, tests pass), `cargo fmt`; `just nix-build`
  host-only confirms the `publication/` graft.

Requirement coverage recorded at reconcile: **met** — REQ-374, REQ-376, REQ-379,
REQ-380 (and PRD invariants REQ-359, REQ-363, REQ-367, REQ-369); **partial** —
REQ-373 (embedded source covered; runtime-loaded source pending, OQ-4), REQ-381
(seam foundation + no-write proven; full library-surface guarantee pending the verbs).
**Pending — later slices:** REQ-375 (library verbs), REQ-377/378 (federation),
REQ-382/383 (federation NF).

## 10. Review Notes

Internal adversarial pass integrated (iter accessor + call-site precision; defer
`UnsupportedSourceType`/`source` discriminator; in-memory binary test; embed type
kept private).

**External codex review — RV-286, 7 findings, all adjudicated valid on evidence and
integrated:**
1. **(Blocker) D-C non-projection premise false.** Verified against
   `install.rs:1395–1438` — projection is blanket. Fixed: separate `publication/`
   embed root (§2, D-C, R4, §9 non-projection test). Also serves REQ-380.
2. **(Blocker) build-time gate could validate stale bytes.** Fixed: authoritative
   admission gate reads manifest from **disk source**; `just gate` runs tests; nix
   `doCheck=false` handled via `just nix-build` presence check (§5.4, R3, R6).
3. **(Major) adapter registry couldn't select an adapter.** Fixed: honest single
   injected `EmbeddedAdapter`, no registry; source-identity deferred (§5.2, D-F).
4. **(Major) "structurally read-only" overclaimed.** Narrowed to the seam foundation
   + no-write test; full REQ-381 is the library slice's; trait has only `read` (§5.3,
   §5.5).
5. **(Major) REQ-373 runtime-loaded overclaim.** Reframed: source-agnostic `admit`;
   embedded covered, runtime-loaded pending — recorded partial (§6, §9).
6. **(Major) D-A didn't prove a real consumer.** Added `emit(addr, &mut dyn Write)`
   framing-free + IO-error test (§5.2, D-A, §9).
7. **(Minor) schema field-name mismatch.** Unified `provenance` + serde renames;
   unknown/missing provenance rejection tested (§5.2, D-E, §9).

REQ-381 exists (NF-002) — an earlier truncated `spec show` hid it; codex was right,
the design was corrected. Decisions D-A…D-F stand as amended above.
