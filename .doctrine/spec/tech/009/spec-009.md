# SPEC-009: Install & distribution

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Install & distribution is the container that gets Doctrine *into* a project: how
the binary carries the files a fresh repo needs and lays them down beside the
code. `doctrine install` embeds the whole `install/` source tree into the binary
at compile time and reproduces it into a target directory (default `.doctrine`)
under the detected project root. There is no second asset bundle and no network
fetch — distribution is the single self-contained binary plus a compile-time
embed.

It sits beneath the whole-system root (SPEC-003) and rides the shared entity
engine (SPEC-004) for nothing it materialises — install writes raw bytes into a
target tree, it does not scaffold entities. What it owns is the embed-and-lay-down
mechanism: the manifest that configures the lay-down, project-root detection, the
idempotent plan/execute split, and the shared `asset_text` read seam that the
entity-scaffolding verbs draw their templates through.

Two boundaries are deliberate. **Skills distribution is a sibling container**
(PRD-003): install does not own the skill symlink tree — it only ensures the
skills gitignore negation and shares its confirm prompt; the skill provisioning
mechanism lives next door, and install *uses* it as a peer (the interaction edge is
PHASE-05). **Global-memory materialisation is not orchestrated here**: install
prints a next-step hint pointing at the standalone `doctrine memory sync` verb
rather than running it as a hidden side effect.

## Responsibilities

Mirrors the structured `responsibilities` list: embed the source tree and
reproduce it into a target; own the manifest as install configuration; detect the
project root; build and idempotently run an inspectable plan; ship the entity
templates and the shared `asset_text` read seam; and carry the authored-entity
wiring contract.

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
- **`[gitignore].entries`** — lines appended to the project `.gitignore`,
  narrowly scoped so the disposable/derived tiers are ignored while committed
  authored trees (e.g. `memory/items/`) stay tracked.
- **`[root_markers].markers`** — the files/dirs that identify the project root.

### Project-root detection

Install lands relative to the repo root, not the working directory. With `--path`
the root is taken directly; otherwise the installer walks up from CWD, treating the
first directory that contains any manifest marker as the root, and errors if it
reaches the filesystem root without a match. The marker set is the same
manifest-driven walk the rest of the tool resolves the root through.

### Plan then execute, idempotently

A run is split into an inspectable plan and its execution. The plan is a list of
typed steps — create-dir, install, skip, gitignore — built by diffing the target
tree against the embedded fileset: a destination that already exists becomes a
*skip*, an absent one an *install*. `--dry-run` prints the plan and stops; a normal
run prints it, confirms (unless `--yes`), then executes. Execution is idempotent by
construction: directories are `create_dir_all` (no-op if present), files are
written only when the destination is absent and **never overwritten**, and
`.gitignore` lines are appended only when not already present (line-based
deduplication, creating the file if missing). Re-running install is a safe no-op —
local edits to an installed file are preserved because install never clobbers.

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
- **Never-overwrite is the safety contract.** Idempotency is what makes re-install
  safe, but the same property means install will never *update* an installed file
  whose source has since changed; bringing a customised install forward is out of
  this container's scope.
- **Gitignore scoping.** A too-broad ignore entry would silently exclude a
  committed authored tree from version control; entries are deliberately narrow and
  additive, and the authored-entity wiring contract is where that discipline is
  enforced.
- **Root misdetection.** A walk that matches the wrong ancestor marker lays the
  tree in the wrong place; `--path` is the explicit escape from an ambiguous walk.

## Hypotheses

- **A single embedded binary beats a separate asset bundle.** Baking the source
  tree into the binary is preferred over shipping a sidecar directory or fetching
  assets, so distribution is one artefact with no version-skew between binary and
  assets and no network dependency at install time.
- **Idempotent never-overwrite is the right default.** Treating an existing file as
  a skip rather than a merge or overwrite is preferred so re-install is always safe
  and a user's local edits are never lost — at the accepted cost that install does
  not push updates to already-installed files.
- **One manifest drives the whole lay-down.** Centralising target, dirs, gitignore,
  and markers in one embedded TOML is preferred over scattering install policy
  across code, so making a new kind distributable is a manifest edit, not a code
  change.

## Decisions

- **D1 — distribution is a compile-time embed, not a bundle.** The `install/` tree
  is embedded via rust-embed and reproduced at runtime; there is no second asset
  artefact and no network fetch. The manifest is embedded but excluded from the
  installed fileset — it configures the install rather than being installed.
- **D2 — install is idempotent and never overwrites.** Files are written only when
  absent, directories are create-if-missing, and gitignore lines are
  deduplicated; re-install is a no-op and local edits survive. The cost — install
  never updates an existing installed file — is accepted.
- **D3 — `asset_text` is the single embedded-asset read seam.** Both the installer
  and the entity-scaffolding verbs fetch templates through one helper, so the
  shipped template set has one source of truth and scaffolding reads the same bytes
  install lays down.
- **D4 — distributability is declared in the manifest.** A new authored kind is made
  distributable by adding its directory to `[dirs].create` and negating its
  derived/runtime subtrees narrowly in `[gitignore].entries` — never by changing
  install code.
- **D5 — skills distribution and memory sync are not absorbed.** The skill symlink
  tree is a sibling container's mechanism (install only ensures its gitignore
  negation and shares the confirm prompt), and global-memory materialisation is a
  standalone `memory sync` verb install only hints at — install orchestrates
  neither.
