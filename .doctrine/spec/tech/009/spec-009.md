# SPEC-009: Install & distribution

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

> **Status — partly forward-intent (RFC-021 C1 / ADR-019).** The
> embed-as-storage, root-detection, idempotent-plan, and `asset_text` mechanisms
> are **shipped and verified**. The projection model described below — minimal
> three-file base, embed/publish/project separation, semantic source roots,
> materialize-on-demand surfaces (D6–D9) — is **target architecture**: authored
> intent the shipped code does not yet implement. Its requirements stand
> `pending` until an implementation slice lands them and reconciliation verifies
> coverage. Do not read D6–D9 or the `pending` requirements as current behaviour.

Install & distribution is the container that gets Doctrine *into* a project: how
the binary carries the files a fresh repo needs and provisions them beside the
code. `doctrine install` embeds the semantic source roots into the binary at
compile time as the **storage** mechanism, then projects a **minimal, explicitly
justified** base fileset into a target directory (default `.doctrine`) under the
detected project root and materializes the rest on first use. There is no second
asset bundle and no network fetch — distribution is the single self-contained
binary plus a compile-time embed, but embedding is *storage*, not the projection
policy: what lands is drawn from an explicit manifest policy, not from what the
binary happens to embed (ADR-019).

It sits beneath the whole-system root (SPEC-003) and rides the shared entity
engine (SPEC-004) for nothing it materialises — install writes raw bytes into a
target tree, it does not scaffold entities. What it owns is the
project-then-materialize mechanism: the manifest that declares the projection
policy, project-root detection, the idempotent plan/execute split, and the shared
`asset_text` read seam that the entity-scaffolding verbs draw their templates
through. Physical source roots follow semantic ownership (directional per
ADR-019: templates / composable guidance / sealed definitions / curated
reference / integrations / base-projection policy / memory corpus) rather than a
single `install/` root — exact root names are left to the implementing slice.

Two boundaries are deliberate. Install owns the skill canon tree and its links
for harnesses with a `skills` route (true since SL-250); SPEC-010 specifies that
mechanism. For harnesses routed `skills_via`, install delegates to `npx skills
add`, a peer it uses but does not own. That
peer `uses` edge (not containment) is authored as a durable `spec interactions`
edge by the implementing slice, not asserted here. **Global-memory
materialisation is not orchestrated here**: install prints a next-step hint
pointing at the standalone `doctrine memory sync` verb rather than running it as a
hidden side effect.

## Responsibilities

Mirrors the structured `responsibilities` list: embed the semantic source roots
as the *storage* mechanism; project a minimal, justified base fileset from the
manifest's projection policy and materialize the rest on first use; own the
manifest as the projection policy; detect the project root; build and idempotently
run an inspectable plan; ship the entity templates and the shared `asset_text`
read seam; and carry the authored-entity wiring contract.

### Compile-time embed, runtime lay-down

A `rust-embed` `#[derive(RustEmbed)] #[folder = "install/"]` asset set bakes every
file under `install/` into the binary at compile time. At runtime the installer
reads those embedded bytes — never the on-disk `install/` — and writes them into
the target tree. An embedded text asset is fetched as UTF-8 through one helper; a
missing or non-UTF-8 asset is a hard error, since it means a broken build, not a
user mistake. Because the embed happens at compile time, an edit to a shipped
source file is invisible until the embedding crate recompiles.

### The manifest as install configuration

`install/manifest.toml` is the single configuration surface. It is embedded like
every other asset but **excluded from the installed fileset** — it configures the
install, it is not part of it. All sections are optional and fall back to defaults:

- **`target`** — the directory the tree is laid into, relative to the project
  root. Default `.doctrine`.
- **`[dirs].create`** — directories created even when no file maps into them, so
  the empty authored-kind trees (`slice/`, `adr/`, `spec/product`, `spec/tech`,
  `memory/items`, the governance-kind dirs) exist from first install.
- **`[gitignore].entries`** — the runtime-state part of the managed gitignore
  block. The block is rendered from these entries, a `<root>/*` line per
  framework root and one anchored line per doctrine link present after the run,
  and spliced between fixed markers, replacing the previous block. Lines outside
  the markers are never read or changed; a malformed marker pair is refused and
  the file left untouched.
- **`[root_markers].markers`** — the files/dirs that identify the project root.

### Project-root detection

Install lands relative to the repo root, not the working directory. With `--path`
the root is taken directly; otherwise the installer walks up from CWD, treating the
first directory that contains any manifest marker as the root, and errors if it
reaches the filesystem root without a match. The marker set is the same
manifest-driven walk the rest of the tool resolves the root through.

### Plan then execute, idempotently

A run is split into an inspectable plan and its execution. The base plan is
typed create-dir / install / skip steps; framework-owned paths get an ownership
plan of write / link / relink / unlink / prune / report actions computed purely
from the declaration, the embedded set and a disk snapshot. `--dry-run` prints
both, including would-prune actions, and changes nothing. Re-install leaves
user-owned files untouched and converges framework-owned ones; the executor
re-proves each entry before mutating it. Removals never pass through a symlinked
ancestor.

### Templates and the shared read seam

The entity templates ship under `install/templates/` as ordinary embedded assets,
laid into the target like any other file. The same embed is also the source the
entity-scaffolding verbs read their templates from: `asset_text` is the one place
embedded assets are fetched as text, and the scaffolding verbs provision a new
entity's files by token substitution over a fetched template body. So a template is
both *installed* into a project and *read live* from the embed when scaffolding —
one asset set, two consumers.

### The authored-entity wiring contract

A new authored kind reaches a fresh project only by wiring it into the manifest:
its directory is added to `[dirs].create` so the empty tree exists, and its
derived or runtime subtrees are negated narrowly in `[gitignore].entries` — never
a blanket ignore that would swallow a committed authored tree. This is the contract
a new kind honours to be distributable; the manifest is where distribution is
declared.

## Concerns

- **Embed staleness across recompiles.** The embed is fixed at compile time, so an
  edit to a shipped source file (a template, a rule) is invisible until the
  embedding crate is recompiled — a refresh discipline the distribution-side verbs
  must respect, not a runtime fault.
- **Never-overwrite is the safety contract for user-owned files.** Idempotency is
  what makes re-install safe, but the same property means install will never
  *update* an installed user-owned file whose source has since changed; bringing a
  customised install forward is out of this container's scope. Framework canon
  roots are declared framework-owned and gitignored, so a user file placed inside
  one is removed and reported (accepted).
- **Gitignore scoping.** A too-broad ignore entry would silently exclude a
  committed authored tree from version control; the managed block is narrow and
  rewritten — user `!` rules placed after it survive re-install — and the
  authored-entity wiring contract is where that discipline is enforced.
- **Root misdetection.** A walk that matches the wrong ancestor marker lays the
  tree in the wrong place; `--path` is the explicit escape from an ambiguous walk.

## Hypotheses

- **A single embedded binary beats a separate asset bundle.** Baking the source
  tree into the binary is preferred over shipping a sidecar directory or fetching
  assets, so distribution is one artefact with no version-skew between binary and
  assets and no network dependency at install time.
- **Idempotent never-overwrite is the right default for user-owned files;
  framework-owned files converge so a re-install pushes updates and retires
  removed instructions.** Treating an existing user-owned file as a skip rather
  than a merge or overwrite is preferred so re-install is always safe and a user's
  local edits are never lost — at the accepted cost that install does not push
  updates to already-installed user-owned files.
- **One manifest drives the whole lay-down, including the ownership declaration
  and harness registry.** Centralising target, dirs, gitignore,
  and markers in one embedded TOML is preferred over scattering install policy
  across code, so making a new kind distributable is a manifest edit, not a code
  change.

## Decisions

- **D1 — embedding is a storage mechanism, not the projection policy (ADR-019).**
  Compile-time rust-embed remains the production *storage* default — one
  self-contained binary, no sidecar bundle, no network fetch. It no longer defines
  the *installed* fileset: what lands in a project is drawn from an explicit
  projection policy (D7), not from what the binary happens to embed. The manifest
  is embedded but excluded from every projected set.
- **D2 — install acts by ownership.** User-owned files (the base backings, hymn
  starters, authored entities) are written only when absent and never overwritten
  or pruned. Framework-owned artefacts — the `[framework]` canon roots and
  doctrine links into them — converge: every shipped member is written and
  refreshed, and on a project install anything else inside a canon root is
  removed and every removal reported; a global install writes and relinks but
  only reports orphans. In a shared harness link dir a symlink whose target lies
  inside a canon root is doctrine's (relinked or removed); anything else is
  foreign and kept. Directories are create-if-missing. Framework gitignore lines
  live in one managed block rewritten each project install.
- **D3 — `asset_text` is the single embedded-asset read seam.** Both the installer
  and the entity-scaffolding verbs fetch templates through one helper, so the
  shipped template set has one source of truth and scaffolding reads the same bytes
  install lays down.
- **D4 — the manifest declares projection *policy*, not embed contents.** The
  manifest declares the minimal base fileset (D7), the materialize-on-demand
  surfaces (D8), the dirs, gitignore negations, and root markers. A new authored
  kind is made *materializable* — its root appears on first use — via a manifest
  edit, not by base-projecting its tree or changing install code. It also
  declares ownership: `[framework] roots` (the canon roots install converges) and
  the `[harness.*]` registry — per harness its detect dir, boot adapter, agent
  link dir and one skills route (`skills` link dir or `skills_via` npx target).
  The harness vocabulary is the registry; an unregistered name gets no defs or
  links and is disclosed.
- **D5 — memory sync is not absorbed; skills are delegated only for `skills_via`
  harnesses.** Install owns the skill canon tree and its links for harnesses with a `skills` route (true since
  SL-250); SPEC-010 specifies that mechanism. For harnesses routed `skills_via`,
  install delegates to `npx skills add`, a peer it uses but does not own.
  Global-memory materialisation is a standalone `memory sync` verb install only
  hints at.
- **D6 — the seven independent asset properties (ADR-019).** Cite ADR-019's
  canonical vocabulary *unchanged* — owned, versioned, distributed, embedded,
  runtime-loaded, published, projected — no property implies another; in
  particular embedding ≠ publication ≠ projection. (Publication *policy* only;
  publication *delivery* is contract B.)
- **D7 — minimal, justified base projection.** The base install is exactly three
  files — `.gitignore`, `doctrine.toml`, `project-orientation.md`. A file is projected
  only when its presence at a stable project path is operationally necessary or
  materially improves discoverability, integration, control, or supported
  customization. No template, reference doc, agent definition, hymn, or
  integration asset is projected by default. An explicit `--agent` harness leg,
  or a detected harness the user confirms (at the prompt or under `-y`), is
  integration, not default projection: it installs that harness's full agent-def
  set and links, which the base projection never does (DEC-372).
- **D8 — materialize on first use.** Surfaces that are distinct but not yet
  populated are not shipped empty: entity roots, customized hymns, and standing
  governance appear on disk only when first used/customized — the pattern already
  used for entity roots and hymns.
- **D9 — governance is a *physically distinct* surface (trust deferred).**
  Standing project governance materializes on **explicit user definition** into
  its own surface (candidate `governance.md`), never folded into the
  agent-improvable `project-orientation.md` orientation surface. This draws the
  volatility / mutation-authority boundary up front (ADR-019 position 3).
  Enforcing trust/approval on edits to that surface is a **Stage-5 concern,
  explicitly out of scope here** — D9 establishes physical separability, not an
  approval mechanism.
