# Design SL-227: Library read surface and minimal projection

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Deliver RFC-021 Claim C1 as one coherent change: the **library read surface**
(Contract B, SPEC-026) that exposes framework-owned material without projecting
it, and the **minimal-projection install flip** (Contract A mechanism, SPEC-009)
that shrinks the base install from ~80 files to three. They are separate
contracts but a deliberate **pair** — minimal projection removes files; the
library restores access to them (RFC-021 design tension). The pairing invariant
is enforced *intra-slice by phase order*: the read path lands (PHASE-01) before
any file stops landing (PHASE-02).

## 2. Current State

- **Publication substrate (SL-223, `publication.rs`):** `PublicationManifest`
  (admitted, immutable), `PublicationEntry` (private fields; only `address()` +
  `report_line()` exposed), `LogicalAddress` (traversal-safe by construction),
  `Resolver<A: SourceAdapter>` with `resolve`/`emit`/`manifest`, `EmbeddedAdapter`
  over the leaf `asset_source` seam. Read-only by construction — `SourceAdapter`
  exposes only `read`. Shipped `publication/manifest.toml` is **templates-only**.
  `ContentKind` is `Template`-only; the enum is documented to widen additively.
- **Install (`install.rs`, ~4067 lines):** `run` → `build_plan` (`:1384`) →
  `execute_plan`. `build_plan` leg 2 (`embedded_filenames()` → `asset_source::iter()`,
  `:1431`) copies the **entire** `install/` embed verbatim into `.doctrine/` —
  the ~80 files are *not itemized anywhere*; they are whatever the embed holds.
  Of the target base trio only `.gitignore` exists today (via `ensure_gitignored`
  `:1451`); `doctrine.toml` is never written (read-only, defaults if absent,
  `dtoml.rs`); `project-orientation.md` does not exist as an asset.
- **First-use roots:** already lazy for every entity-engine kind — `entity.rs`
  `materialise*` `create_dir_all(tree_root)` at `:321/:395/:499`. The manifest's
  eager 18-dir `[dirs].create` is redundant with this.
- **Harness adapters:** already harness-gated by `detect_agents` (`:1525`) → the
  per-agent forward loop (`:421-546`). Not part of the unconditional base dump.

## 3. Forces & Constraints

- **ADR-001** layering (leaf ← engine ← command, no cycles).
- **ADR-019** — embedding ≠ publication ≠ projection (seven independent
  properties). **NF-001** (SPEC-026): publication and projection are independent
  manifests, neither derived from the other.
- **STD-001** named constants (the closed licence/kind/customization vocabularies).
- **Behaviour-preservation gate** (AGENTS.md): shared-machinery changes keep the
  existing suites green; publication VT-1..6 and install *mechanism* tests must
  not move.
- **POL-002** — platform independence; no host-project literals in the engine.
- **Repo denials** — `print_stdout` (stream via a `Write` handle, not `println!`).

## 4. Guiding Principles

- The library is a **pure read veneer** over the SL-223 `Resolver`; additive
  engine changes only, no resolver redesign.
- The flip changes **one leg's enumeration** (embed → declared base set), nothing
  more invasive.
- Publish the **bounded** set (templates + reference docs, [[DEC-010]]); do not
  grow the licence surface beyond what reachability parity requires.
- Name honest pending items rather than fake coverage.

## 5. Proposed Design

### 5.1 System Model

```
command   library.rs (NEW)            install.rs (flip: build_plan leg 2 + base policy)
              │  depends on                 │  reads
engine    publication.rs (Resolver, manifest — ADDITIVE only)   install/manifest.toml [base]
              │  IO only through
leaf      asset_source.rs (byte reads over embeds; + exists)
```

`library → publication → asset_source`, no cycle. The two independent manifests
are the structural spine (NF-001):

| | Projection policy | Publication policy |
|---|---|---|
| File | `install/manifest.toml` (new `[base]` section) | `publication/manifest.toml` |
| Decides | what lands in `.doctrine/` on install | what is reachable via `library` |
| Consumer | `install::build_plan` | `library` + `Resolver` |

### 5.2 Interfaces & Contracts

**Additive to `publication.rs` (engine):**

```rust
impl PublicationEntry {                 // structured accessors for list/tree rendering
    pub(crate) fn title(&self) -> &str;
    pub(crate) fn kind(&self) -> ContentKind;
    pub(crate) fn licence(&self) -> Licence;
    pub(crate) fn customization(&self) -> CustomizationStatus;
}                                        // `backing` stays private
pub(crate) trait SourceAdapter {
    fn read(&self, backing: &str) -> Result<Cow<'static,[u8]>, ResolveError>;
    fn exists(&self, backing: &str) -> bool;                    // NEW availability probe
}
impl<A: SourceAdapter> Resolver<A> {
    pub(crate) fn available(&self, e: &PublicationEntry) -> bool; // = self.source.exists(&e.backing)
}
enum ContentKind { Template, Reference }                        // widened additively (+ "reference")
```

Leaf `asset_source` gains `exists(key) -> bool` (existence without materializing
bytes).

**New command module `library.rs`:**

```rust
fn load_resolver() -> Result<Resolver<EmbeddedAdapter>, AdmissionError>;
pub(crate) fn run_list(path: Option<&str>, fmt: OutputFormat) -> anyhow::Result<()>;
pub(crate) fn run_tree(path: Option<&str>, fmt: OutputFormat) -> anyhow::Result<()>;
pub(crate) fn run_show(addr: &str) -> anyhow::Result<()>;   // raw byte stream — no format
```

CLI (`main.rs`): `doctrine library {list,tree,show}`. `list`/`tree` carry the
house `--format <table|json>` / `--json` flatten struct (`main.rs:150-199`);
`show` streams raw bytes via `Resolver::emit` into `io::stdout().lock()`.

**`show` error-class mapping (FR-003) — four reachable classes, each distinct
stderr + non-zero, never a silent empty:**

| FR-003 class | Type | Origin |
|---|---|---|
| invalid / traversal address | `AdmissionError::TraversalRejected` | `LogicalAddress::parse` |
| unknown / unpublished address | `ResolveError::UnknownAddress` | `Resolver::resolve` |
| missing backing / bytes-absent | `ResolveError::BackingSourceMissing` | `EmbeddedAdapter::read` |
| malformed / unreadable policy | `AdmissionError::{MalformedManifest,ManifestAssetMissing,…}` | `load_resolver` |
| *unsupported source type* | — **unreachable this slice** (D3) | needs >1 adapter (OQ-4) |

### 5.3 Data, State & Ownership

- **Projection base set** — declared in `install/manifest.toml` (`[base]`),
  owned by install. `build_plan` leg 2 reads it, *not* `embedded_filenames()`
  (FR-007: "not from embed contents"). Three entries → three `Step`s:
  `.gitignore` (`Gitignore`, merge, unchanged); `doctrine.toml` (`Install`,
  write-if-absent, NEW embed = commented example); `project-orientation.md`
  (`Install`, write-if-absent, NEW embed = orientation content).
- **Published set** — `publication/manifest.toml` gains four `reference/*`
  entries (`ContentKind::Reference`), licence per [[DEC-010]]:

  | asset | logical address | licence | provenance |
  |---|---|---|---|
  | glossary.md | `reference/glossary.md` | MIT | declared |
  | using-doctrine.md | `reference/using-doctrine.md` | MIT | declared |
  | review-ledger.md | `reference/review-ledger.md` | GPL | declared |
  | governance.md | `reference/governance.md` | GPL | declared |

- **Standing governance** — `project-governance.md`, a distinct user-owned
  surface (NF-005). The flip simply does not project it; its
  materialize-on-definition command (FR-010) is **out of scope** (D4), FR-010
  stays `pending`.
- **Hymns (FR-009, D6)** — the only hymn path today is the opt-in install-time
  prompt "Project exposed hymn starters? [y/N/a]" (`install.rs:383`, default N)
  over `project_starters`; there is **no** hymn-customization verb. So NF-004
  ("no hymn projected *by default*") already holds — the prompt defaults to N —
  and this slice **leaves the prompt as-is** (D6). FR-009's positive
  "materialize on first customization" needs the unbuilt customization verb; it
  stays `pending`, deferred symmetric with FR-010.
- **Projection base declaration** — a new `[base]` section on the `Manifest`
  struct (`install.rs:60-135`); its deserialization + a test are code-impact
  (F4). `build_plan` leg 2 reads `manifest.base`, not `embedded_filenames()`.
- `backing` remains encapsulated in `publication.rs`; the availability probe
  reaches it within-module via `Resolver::available`.

### 5.4 Lifecycle, Operations & Dynamics

```
library list/tree ─▶ load_resolver ─▶ manifest().entries()
                                        │ filter by LogicalAddress prefix
                                        │ per entry: accessors + resolver.available(e)  → "[unavailable]" mark
                                        └─▶ render (table | --json envelope)
library show <a>  ─▶ LogicalAddress::parse(a) ─▶ resolver.emit(a, stdout.lock())
                        error → distinct stderr diagnostic + non-zero exit
install (flipped) ─▶ build_plan leg 2 = install/manifest.toml [base] (3 files)
                     leg 1 eager [dirs].create trimmed (FR-008 lazy via entity.rs)
                     harness-gated forward legs unchanged (NF-004)
```

FR-008 is satisfied by **deletion** (drop the redundant eager dirs) + a test
proving each authored kind's root appears on first scaffold.

### 5.5 Invariants, Assumptions & Edge Cases

- **NF-002 (structural no-write):** `library.rs`'s entire capability surface is
  the read-only `Resolver` (`SourceAdapter` = `read`+`exists`, both reads) + a
  stdout `Write`. It imports no fs-write, no `install`/`state`/`entity` mutator;
  `EmbeddedAdapter` holds no fs path or write handle. No path from a library verb
  to a project mutation exists *by construction* — proven once (a byte-unchanged
  test over a clean repo across every verb incl. failure paths), not per-verb.
- **Sole authority (FR-001):** an embedded-but-undeclared asset is invisible to
  every library verb.
- **write-if-absent:** `doctrine.toml` / `project-orientation.md` never clobber a
  user copy (`Step::Skip` on present).
- **Availability edge:** an entry whose backing is absent stays *visible* in
  list/tree (metadata is valid) marked `[unavailable]`; only `show` on it fails.
- **[[ASM-003]]:** customization-status stays the provisional `{customizable,
  fixed}` enum; C2 widens it.

## 6. Open Questions & Unknowns

- **OQ-1** — machine output *shape* for `list`/`tree`: adopt the house
  `--format/--json` flatten; the JSON envelope fields (per-entry metadata +
  availability) confirmed against an existing read command during PHASE-01.
- **OQ-2 (deferred)** — `library serve` / HTTP, write/materialize/copy verbs:
  explicitly out of scope; library is strictly read-only here.
- **OQ-3 (deferred → OQ-4)** — multi-source dispatch; until it lands the
  *unsupported-source-type* FR-003 class is unreachable (D3).

## 7. Decisions, Rationale & Alternatives

- **D1 — base set declared in `install/manifest.toml`, not `asset_source::iter()`.**
  Satisfies FR-007 ("not from embed contents") and NF-001 (independence) in one move.
- **D2 — availability on the adapter (`exists`), not trial-`read`.** Keeps
  list/tree cheap; avoids materializing bytes to mark a row. Seam reused by
  [[IMP-309]] (a future `doctor` availability check).
- **D3 — drop *unsupported-source-type* to forward-intent.** Unconstructable with
  one `EmbeddedAdapter`; implementing the reachable four and marking the fifth
  pending (OQ-4) beats a dead stub variant. *Alt rejected:* `cfg_attr` dead stub.
- **D4 — FR-010 projection-only.** The flip stops projecting standing governance;
  the materialize-on-definition command is a follow-up. The pairing invariant
  concerns the *reference* docs (published), not the user's authoring affordance.
  FR-010 stays `pending`. *Alt rejected:* build the define command now (scope creep).
- **D5 — mixed MIT/GPL licence calls.** glossary/using-doctrine are copyable
  reference (MIT); review-ledger/governance encode process (GPL). *Alt:*
  all-GPL-conservative — a one-column change if preferred.
- **D6 — FR-009 leave-prompt + defer verb (adversarial pass).** The install-time
  hymn prompt (default N) already satisfies NF-004; retiring it would open a
  customization gap with no replacement verb. So leave it, defer FR-009's
  materialize-on-customization verb (pending, symmetric with FR-010). *Alt
  rejected:* retire the prompt now (capability regression) or build the verb now
  (scope creep).
- **DEC-010 — bounded published set** (templates + reference docs); [[QUE-172]]
  answered *no*. **REV-031** — base orientation surface renamed
  `boot-project.md` → `project-orientation.md` (SPEC-009 aligned).

## 8. Risks & Mitigations

- **R1 — install-flip regression.** Large module; ~9 tests encode today's
  projection set. *Mitigation:* re-gate the auxiliary legs (don't delete);
  behaviour-preservation gate keeps mechanism tests green; the flip touches only
  `build_plan` leg 2 + a declared base section.
- **R2 — FR-008 first-use seam** (was: new cross-cutting mechanism). *Dissolved
  by the map:* `entity.rs:321/395/499` already lazy; FR-008 = deletion + a
  first-scaffold test.
- **R3 — RustEmbed re-embed / crane strip.** New embeds (`install/doctrine.toml`,
  `install/project-orientation.md`) are invisible until recompile, and crane
  strips non-rust embed assets. *Mitigation:* they ride the existing `install/`
  RustEmbed root (already grafted); a lone edit needs a rebuild before it admits;
  admission tests read from disk source (staleness-proof, publication VT-3 pattern).
- **R4 — reachability hole.** *Mitigation:* the no-silent-unreachable gate (§9)
  binds every un-projected reference doc to `library show`.

## 9. Quality Engineering & Validation

- **Library (new):** list/tree prefix filter + tree grouping + availability mark
  + `--json` shape; `show` byte-exact + the four reachable error classes;
  **NF-002** byte-unchanged over a clean repo across every verb+failure path;
  **FR-001** undeclared-asset-invisible; reference entries admit with
  `ContentKind::Reference`.
- **Install (changed):** the ~9 projection-set assertions flip polarity
  (`glossary_is_shipped`, `using_doctrine_is_shipped`, `review_ledger_is_shipped`,
  `execute_creates_dirs_and_files`, `execute_skips_existing_files`,
  `plan_skips_existing_files`, `seeds_governance_when_missing`, the `[dirs]` tree
  assertions).
- **Install (added):** fresh install = exactly `{.gitignore, doctrine.toml,
  project-orientation.md}` (FR-007); no auxiliary asset (NF-004);
  `project-governance.md` absent (NF-005); write-if-absent non-clobber; FR-008
  first-scaffold; harness-survival with a detected harness.
- **The crux — no-silent-unreachable gate:** each reference doc resolves via
  `doctrine library show reference/<x>`. The pairing invariant, executable.
- **Behaviour preservation:** install mechanism tests
  (`plan_creates_dirs_from_manifest`, `ensure_gitignored_*`, agent/workflow/
  `detect_agents`) green unchanged; publication VT-1/2/4/5/6 unchanged.
  **Exception (F1):** publication **VT-3** (`shipped_manifest_admits_from_disk_source`)
  asserts *every* shipped entry is `licence=MIT` (templates-only) — adding
  `reference`-kind, GPL, `fixed` entries **changes** VT-3 to admit the widened
  vocabulary. A *changed* test, not preserved; called out so the gate is honest.
- **Coverage recorded:** SPEC-026 FR-001/FR-003/NF-002; SPEC-009
  FR-007/FR-008/NF-004/NF-005. **Left pending (honest):** SPEC-009 FR-010 (D4),
  **FR-009 (D6 — no customization verb this slice)**, and the FR-003
  unsupported-source-type bullet (D3).

## 10. Review Notes

Phases (library-first; pairing enforced by `PHASE-01 → PHASE-02` order):

- **PHASE-01 — Library read surface + reference publication.** Additive
  `publication.rs` + `asset_source::exists`; `library.rs` (`list`/`tree`/`show`)
  + `main.rs` wiring + `--format/--json`; four reference entries into
  `publication/manifest.toml`. Delivers the read path.
- **PHASE-02 — Minimal-projection flip.** `install/manifest.toml [base]`; new
  embeds `doctrine.toml` + `project-orientation.md`; `build_plan` leg 2 → base
  set, trim leg 1; flip/add install tests; FR-008 verification; NF-004/005; the
  no-silent-unreachable gate. The cut, structurally after the read path.

`/plan` authors the formal EN/EX/VT and may split PHASE-01 (engine additions vs
command veneer) if it sizes large.

### Adversarial pass (internal, integrated)

- **F1 (correctness) — integrated.** §9 wrongly claimed publication VT-1..6 all
  stay green; **VT-3** asserts every shipped entry is `licence=MIT` and *changes*
  when reference/GPL/fixed entries land. §9 corrected.
- **F2 (scope) — integrated as D6.** FR-009 was in slice scope but unrealizable
  (no customization verb; only the opt-in install prompt). Left prompt as-is,
  FR-009 deferred `pending`. Slice scope reconciled to match.
- **F3 (dissolved).** Reference-doc backing keys resolve over the `InstallAssets`
  (`install/`) root (`asset_source::read_bytes`); reference docs stay physically
  in `install/`, no move — physical `reference/` root reorg is out of scope.
- **F4 (precision) — integrated.** The `[base]` section is a `Manifest`-struct
  schema change (`install.rs:60-135`) + deserialization test; named in §5.3.
- **F5 (precision).** `OutputFormat` is indicative — PHASE-01 binds the exact
  house flatten type (`main.rs:150-199`); no bespoke flag.
- **F6 (verification).** The NF-002 byte-unchanged test covers all three
  protected roots: client project + runtime state (disk-checkable over a temp
  repo); source/backing is immutable-by-embedding (no write handle exists).
- **F7 (doctrinal).** RFC-021's body still names `boot-project.md` — a
  point-in-time discussion artifact, deliberately not retro-edited; the live
  contract is SPEC-009 (REV-031). Noted so the divergence is not read as a miss.
