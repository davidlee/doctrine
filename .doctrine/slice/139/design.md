# Design SL-139: Uniform entity show and file path surfaces

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Doctrine's authored entity commands should present a predictable read surface.
`show` is already the canonical full-reading surface: it renders an entity's
readable body/reconstruction, and in JSON mode returns the body plus the
kind-owned structured/embedded data needed for full inspection. File paths are a
different user intent: editor and shell plumbing normally wants only paths, not
body text or metadata.

SL-139 therefore has two targets:

1. preserve and normalize `show` parity for authored entity commands, including
   fixing concept-map's missing `--json` shorthand; and
2. add a dedicated `paths` verb to every in-scope authored entity command so a
   caller can reliably obtain the authored files backing one or more entities.

The richer summary/metadata surface was explored during design and found to be a design dead-end (IMP-145, closed wont-do). This design does not impose a follow-up for it.
This slice should not overload `show` with operational file-location output that
would later be moved to an `info` surface.

## 2. Current State

The in-scope authored entity commands are:

- `adr`
- `policy`
- `standard`
- `rfc`
- `spec`
- `backlog`
- `knowledge`
- `slice`
- `memory`
- `review`
- `rec`
- `revision`
- `concept-map`

All have a `show` verb. Most accept `--json`; `concept-map show` currently
accepts `--format json` but lacks the `--json` shorthand used by other entity
commands.

There is no uniform file-path read surface. Operators either know the storage
layout and construct paths manually, or infer paths from `show`/list output and
kind-specific knowledge.

Existing `show --json` is not merely a compact metadata view. It is tested and
used as a full-inspection surface:

- `spec show --json` carries the spec body, members, resolved requirement data,
  requirement bodies, and interactions.
- `slice show --json` carries the scope body and relationship/dependency axes.
- `memory show --json` carries the memory body, relations, wikilinks, scope,
  trust, and anchor data.
- `review show --json` and MCP `review_show` expose the review body/findings and
  derived status.

Adding file paths to `show --json` would mix operational file-location metadata
into a body/reconstruction surface — a path previously explored and found to be a dead-end (IMP-145, closed wont-do).

## 3. Forces & Constraints

- **ADR-001 layering**: filesystem reads do not belong in pure presentation
  leaves such as `listing.rs`. Clap-facing argument parsing stays command-side;
  shared path selection must not import clap.
- **SPEC-013 CLI contract**: the CLI shape is a uniform `<kind> <verb>` grammar.
  `paths` expands the entity verb set and will need reconciliation against
  SPEC-013 after implementation evidence exists.
- **SPEC-004 storage rule**: authored entities are directory-backed, with an
  identity TOML and prose Markdown body plus kind-specific sibling files.
- **ADR-005 read discipline**: `show` remains the full read/reconstruction
  surface for entity content; file paths are a separate operational concern.
- **No parallel implementation**: use a shared path projection helper and narrow
  per-kind adapters rather than bespoke path scanning in each command.
- **No broad parser refactor**: IMP-125 may eventually consolidate reference
  parsing, but SL-139 should not take that dependency unless implementation
  proves unavoidable.

## 4. Guiding Principles

1. **One command, one intent**: `show` reads content; `paths` returns paths.
2. **Shell-friendly output**: `paths` emits plain lines, no table, no JSON,
   no headers, and no grouping labels.
3. **Stable path order**: output is deterministic and usable in scripts.
4. **Root-relative paths**: output is portable across worktrees and test roots.
5. **Atomic stdout for splats**: if any requested ref fails, do not emit partial
   path output where practical.
6. **Minimal surface area**: add the new verb and a shared helper; do not rewrite
   kind-specific `show` renderers for path concerns.

## 5. Proposed Design

### 5.1 System Model

A `paths` command resolves one or more entity references through the same
kind-specific reference rules as `show`, computes the entity's authored file set,
applies selector flags, and prints the selected root-relative paths.

Each resolved entity has three conceptual path classes:

1. **identity TOML** — the primary structured entity file;
2. **identity Markdown** — the primary prose/body file when the kind has one;
3. **other direct regular files** — additional direct files in the entity folder,
   sorted by root-relative path.

The helper excludes symlink aliases, directories, and runtime state. It sees only
regular files in the resolved entity's authored folder.

### 5.2 Interfaces & Contracts

Every in-scope command gains:

```text
doctrine <kind> paths <REF>... [-t|--toml] [-m|--md] [-e|--entity] [-s|--single]
```

Examples:

```text
doctrine slice paths 012
doctrine slice paths SL-012
doctrine slice paths 012 056 129 --single
doctrine slice paths 012 -m -s | glow --pager
```

Reference rules:

- one or more refs are required;
- output preserves CLI input order;
- each ref is parsed by the owning kind's existing parser;
- splats may mix bare numeric and canonical references where that kind already
  supports both.

Selector rules:

- if no selector is supplied: all direct regular files in the entity folder,
  ordered as identity TOML, identity Markdown, then other files sorted
  lexicographically;
- once any selector is supplied, the default all-files set is disabled and only
  the selected classes are emitted;
- `-t`, `--toml`: identity TOML;
- `-m`, `--md`: identity Markdown;
- `-e`, `--entity`: identity TOML + identity Markdown;
- selectors compose additively; `-t -m` is equivalent to `-e`;
- selector flags do not reorder output; canonical path class order always wins;
- `-s`, `--single`: per input ref, truncate the selected ordered result to its
  first path.

`--single` examples:

- `paths SL-012 --single` returns the identity TOML path;
- `paths SL-012 --md --single` returns the identity Markdown path;
- `paths SL-012 --entity --single` returns the identity TOML path.

Output contract:

- one root-relative path per line;
- no table, JSON, headers, or grouping labels;
- no partial stdout if any ref fails before output is written.

### 5.3 Data, State & Ownership

Add a small shared path projection module, `src/paths.rs`.

**ADR-001 tier: engine.** The module depends only on leaf-tier modules (stdlib,
`src/entity.rs` for `rel_path`/`id_path` helpers) and is depended on by
command-tier modules (per-kind CLI dispatch). The pure types (`EntityPathSet`,
`PathSelection`) and the pure selection logic live directly in the module; the
filesystem-scanning function is also engine-tier because the existing engine
already carries filesystem access (`entity.rs` `id_path`, `scan_ids`,
`materialise`). The module imports no clap types and no command-tier modules.

It must not live in `src/listing.rs` because `listing.rs` is the pure list
spine under SPEC-013. It should not grow `src/entity.rs` unless implementation
finds a small path helper belongs there; `entity.rs` owns materialisation,
claiming, and id path construction, not command read projection.

Suggested clap-free types:

```rust
pub(crate) struct EntityPathSet {
  pub(crate) toml: PathBuf,
  pub(crate) md: Option<PathBuf>,
  pub(crate) others: Vec<PathBuf>,
}

pub(crate) struct PathSelection {
  pub(crate) toml: bool,
  pub(crate) md: bool,
  pub(crate) entity: bool,
  pub(crate) single: bool,
}
```

The shared module owns:

- converting absolute paths under `root` to root-relative display strings;
- scanning an entity directory for direct regular files;
- classifying identity TOML / identity Markdown / other regular files;
- applying selector flags in canonical class order;
- returning selected path strings without writing stdout.

Command modules own:

- clap parsing for `paths` flags;
- finding the project root;
- parsing kind-specific references;
- constructing the identity TOML and Markdown paths for that kind;
- calling the shared helper for each resolved ref;
- writing stdout once after all refs succeed.

### 5.4 Lifecycle, Operations & Dynamics

For a `paths` invocation:

1. Find the project root.
2. Parse and resolve every input ref in order.
3. For each ref, compute identity TOML and identity Markdown paths.
4. Ask the shared helper to build and select the path set for that entity.
5. Accumulate all output lines in memory.
6. If every ref succeeds, write the joined lines to stdout.
7. If any ref fails, return an error before stdout is written.

Per-kind adapters:

- numeric stem kinds (`adr`, `policy`, `standard`, `rfc`, `slice`, `review`,
  `rec`, `revision`, `concept-map`) parse refs and derive paths from their kind
  descriptor/stem.
- `backlog` resolves the prefix (`ISS`, `IMP`, `CHR`, `RSK`, `IDE`) to the
  sub-kind directory `.doctrine/backlog/{issue|improvement|chore|risk|idea}/{id}/`
  with identity files following the sub-kind stem (`issue-NNN.toml`, etc.).
- `spec` resolves `PRD-NNN` or `SPEC-NNN` to the sub-kind directory
  `.doctrine/spec/{product|tech}/{id}/` with identity files following the
  sub-kind stem (`product-NNN.toml`, `tech-NNN.toml`).
- `knowledge` resolves the prefix (`ASM`, `DEC`, `QUE`, `CON`) to the sub-kind
  directory `.doctrine/knowledge/{assumption|decision|question|constraint}/{id}/`
  with identity files following the sub-kind stem (`assumption-NNN.toml`, etc.).
- `memory` resolves uid/key to its concrete memory directory; identity files are
  `memory.toml` and `memory.md`.

### 5.5 Invariants, Assumptions & Edge Cases

- Output paths are always root-relative.
- Symlink aliases such as `139-slug` or memory key links are not emitted.
- Directories and runtime-state links/files are not emitted.
- **Exclusion filter.** The paths helper excludes non-authored regular files:
  (a) entries whose name starts with `.` (hidden files, e.g. `.DS_Store`,
  `.gitkeep`); (b) entries whose name starts with `#` or ends with `~` or `.swp`
  (editor temporaries); (c) entries matching known tool-artifact patterns
  (`.orig`, `.bak`). The exclusion is applied at the file-listing stage before
  classification into TOML/MD/other.
- Missing identity TOML is an error.
- Missing identity Markdown is an error when explicitly selected by `--md` or
  `--entity`; for default all-files mode it should be treated as absent only if
  the kind legitimately has no Markdown body. The current in-scope kinds all have
  Markdown companions, so implementation should start strict and let any contrary
  discovery route through `/consult` rather than silently weakening the contract.
  **Note:** `concept_map::read_concept_map` currently tolerates a missing
  Markdown body via `unwrap_or_default()`. This pre-existing tolerance is
  preserved for `concept-map show` (show is a content-reader, not a
  file-existence assertion) but `concept-map paths --md` will enforce the
  design's strict contract and error on a missing body file.
- Other regular files are direct children only; recursive traversal is out of
  scope.
- For multi-ref invocations, selectors apply independently per ref.
- `show` output and `show --json` output stay unchanged except concept-map gains
  the `--json` shorthand.

## 6. Open Questions & Unknowns

No blocking open questions remain.

Known reconciliation item: SPEC-013's uniform verb-set text must be reconciled to
include `paths` once implementation evidence exists.

## 7. Decisions, Rationale & Alternatives

### D1 — `show` remains the reading surface

`show` renders body/reconstruction plus kind-owned embedded data. `show --json`
remains full inspection JSON. File paths do not enter `show`.

Rejected alternative: add `show --filepaths` and top-level JSON `filepaths`.
That would mix file-location metadata into a body/reconstruction surface and
create planned rework toward a future info surface (IMP-145, closed wont-do — the dead-end confirmed).

### D2 — `paths` is the file-location surface

A dedicated verb best matches the user intent. When a caller asks for file paths,
they usually want only paths for shell/editor composition.

Rejected alternative: add a broad `info` verb now. That is likely the right home
for richer non-body summaries, but the immediate need is narrower and should not
force a larger metadata design.

### D3 — path output is shell-oriented

`paths` prints plain lines only. JSON/table output would make the command less
useful for direct shell composition and would overlap future `info` work.

### D4 — path classes are explicit

`--toml`, `--md`, `--entity`, and `--single` make the "which path?" question
explicit while keeping the common case short.

### D5 — partial output is avoided

Splat support is useful, but partial stdout is dangerous in pipelines. Resolve and
collect before writing.

### D6 — implementation stays narrow

The shared helper owns path projection and selection. Per-kind modules supply only
reference resolution and identity file paths. No broad parser consolidation unless
forced.

### D7 — spec update is reconciliation work

The design deliberately notes the SPEC-013 mismatch. Reconciliation should update
SPEC-013 after implementation proves the final verb shape.

### D8 — show-parity scope is CLI-grammar parity, not JSON-output-shape uniformity

The design's show-parity objective is: every in-scope entity command MUST accept
`--json` as a boolean shorthand alongside `--format json`. The concrete deviation
is concept-map's missing `--json` flag — fixing it achieves the objective. JSON
output-shape normalization across kinds (e.g., a common top-level envelope or
kind-key) is explicitly out of scope for SL-139. The info/summary surface was
explored and closed as a design dead-end (IMP-145, resolved wont-do); this
decision crystallises that boundary and closes that avenue.

## 8. Risks & Mitigations

- **Risk: per-kind adapters duplicate reference parsing.** Mitigation: use existing
  parse functions; do not invent new parsing rules. If duplication becomes
  material, record it against IMP-125 rather than refactoring mid-slice.
- **Risk: helper placement violates layering.** Mitigation: keep filesystem path
  projection out of `listing.rs`; keep clap out of the shared helper.
- **Risk: entity kinds differ in identity file names.** Mitigation: adapters pass
  explicit identity TOML/MD paths rather than forcing every kind through a single
  descriptor shape.
- **Risk: multi-ref partial output leaks before an error.** Mitigation: collect
  output lines first and write once.
- **Risk: SPEC-013 drift.** Mitigation: carry the reconciliation item explicitly
  and update the spec during reconcile.

## 9. Quality Engineering & Validation

Add or extend tests for:

1. **Paths conformance matrix**
   - every in-scope kind accepts `paths <ref>`;
   - every in-scope kind accepts `paths <ref> --single` and short `-s`;
   - every in-scope kind accepts `--toml`/`-t`, `--md`/`-m`, and
     `--entity`/`-e`.

2. **Path ordering and selection goldens**
   - default output is TOML, Markdown, then sorted other regular files;
   - `--entity` outputs TOML then Markdown;
   - `--toml` outputs only TOML;
   - `--md` outputs only Markdown;
   - `--single` truncates per ref;
   - root-relative paths only.

3. **Exclusion behavior**
   - symlink aliases excluded;
   - subdirectories excluded;
   - runtime links/state excluded.

4. **Splat behavior**
   - multiple refs preserve input order;
   - `--single` applies per ref;
   - invalid second ref yields non-zero exit and empty stdout.

5. **Show preservation**
   - existing show JSON goldens stay unchanged;
   - concept-map `show --json` shorthand is covered.

6. **Validation/gates**
   - `doctrine validate` remains clean;
   - `just check` during implementation;
   - `just gate` before close.

## 10. Review Notes

Internal adversarial pass completed after the first draft.

Findings and dispositions:

- **F1 — selector default ambiguity (major, fixed).** The draft said selectors
  compose but did not explicitly say whether `--toml` adds to the default all-file
  set or replaces it. Fixed in §5.2: no selector means all files; once any selector
  is present, only selected classes emit.
- **F2 — missing Markdown weakening risk (minor, fixed).** The draft allowed
  absent Markdown in default mode for theoretical no-body kinds, but every
  in-scope kind has a Markdown companion today. Fixed in §5.5: implementation
  starts strict and must consult if a contrary kind is discovered.
- **F3 — SPEC-013 drift (major, accepted/reconciled later).** A new uniform
  `paths` verb conflicts with SPEC-013's current `new/list/show/status` wording.
  The design names this explicitly in §6 and D7; per user direction, the spec
  update is reconciliation work after implementation evidence exists.
- **F4 — show JSON consumer risk (major, avoided by design).** The design no
  longer adds paths to `show --json`, preserving existing full-inspection
  consumers and goldens.
