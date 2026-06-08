# Slices specification

## Overview

A **slice** is the primary unit of intentional change in doctrine — a
*declarative change bundle*. It answers **"what changes, why, what it touches,
what risks, and what 'done' looks like"** *before* code moves.

It is the doctrine form of what `spec-driver` calls a *delta*. The concept is
deliberately pared back: doctrine has no spec or requirement registry yet, so
a slice in v1 anchors to nothing and enforces no coverage gates. The artefact is
shaped so that linkage and an audit/patch lifecycle can attach **later, without
restructuring**, once product/tech specs exist (§ Forward compatibility).

Key property: **declarative, not imperative**. A slice declares the desired end
state, scope, and constraints. The *how* — detailed design, task breakdown — is
execution that lives in the prose body or in sibling artefacts added later. The
slice is the contract.

## On-disk structure

Slices live under `.doctrine/slice/` in the target project. Each slice is a
**directory** named by a zero-padded sequential integer, with a sibling symlink
carrying a human-readable slug:

```
.doctrine/slice/
  001/                      ← canonical slice directory
    slice-001.toml          ← structured metadata (§ Metadata)
    slice-001.md            ← prose body (§ Prose body)
  001-add-skill-removal -> 001   ← convenience symlink (slug alias)
  002/
    slice-002.toml
    slice-002.md
  002-vendor-skills -> 002
```

- The **numeric directory** (`001/`) is canonical. Tools resolve slices by id.
- The **symlink** (`001-<slug>`) is a convenience alias for humans browsing the
  tree. It is created and maintained by `doctrine slice`; it is not authoritative.
- A slice is a directory (not a single file) so design/plan/phase siblings can
  be added later without moving the artefact. v1 ships only `slice-<id>.toml`
  and `slice-<id>.md`.

The installer creates `.doctrine/slice/` (added to `manifest.toml` `[dirs]`).

### Id allocation

Ids are monotonic integers, zero-padded to **three digits** by default (`001`,
`002`, …, `999`). Past `999` the width grows naturally (`1000`, …); supporting
that auto-grow is acceptable as a later refinement — v1 may assume the 3-digit
range. Padding is cosmetic: the id's identity is its integer value, so a
width change never renames an existing slice. Slugs are not unique and carry no
ordering; the id is the identity.

Allocation is **collision-free under concurrent local agents** without a lock or
a daemon, because the numeric directory *is* the reservation primitive:

1. Scan `.doctrine/slice/` for numeric directories; candidate = `max + 1` (or
   `001` when empty).
2. `mkdir` the candidate directory. The create is atomic and exclusive — it
   fails with `EEXIST` if another agent claimed that id between the scan and the
   create.
3. On `EEXIST`, recompute and retry (bounded retries).

Naïve "scan then write" is a TOCTOU race: two agents read the same `max` and
write the same id. The `mkdir` claim closes it. This is VCS-agnostic — it works
the same under git and jj (both are valid project roots), unlike a git-custom-ref
reservation scheme, which is git-only.

What this does **not** cover: two agents in **different worktrees or clones**
(separate filesystems). `mkdir` cannot see across them, so they can still claim
the same id and collide at merge. That distributed case needs a shared
reservation authority (a remote ref, à la lazyspec RFC-030) and is out of scope
for v1 (§ Known risks).

## Metadata (`slice-<id>.toml`)

Most doctrine entities carry their structured data as **TOML frontmatter** in
the markdown file itself. Slices (and, later, specs) are the exception: they are
folder-backed and carry **more matter than sits comfortably in frontmatter**, so
their structured data lives in a **sister TOML file** instead.

It carries what a `spec-driver` delta split across frontmatter and its
relationships block, in one place, designed to accrete further structured
sections over time.

```toml
id = 1
slug = "add-skill-removal"
title = "Add skill removal to doctrine skills"
status = "proposed"          # proposed | ready | started | audit | done | abandoned  (read-filter enforced; writes manual)
created = "2026-06-03"
updated = "2026-06-03"

[relationships]
# Reserved. Empty in v1 — no spec/requirement registry to point at yet.
# When specs land, this section gains (illustrative, not v1):
#   specs        = ["..."]   # primary ∪ collaborators — scope for coverage gates
#   requirements = ["..."]   # implements / updates / verifies
#   supersedes   = [2]       # slice-to-slice links
```

- `status` is recorded but **not gated** in v1 (§ Lifecycle). Any value in the
  set is accepted; transitions are by hand.
- `[relationships]` is present-but-empty in v1. It is the seam the future
  spec/audit/patch lifecycle attaches to.
- Additional `[…]` sections (risks, context inputs) may be added later as
  structured blocks; v1 keeps risks and context in prose (§ Prose body) to avoid
  premature schema.

The reader extracts only what it needs (`id`, `slug`, `title`, `status`) via
`toml` + `serde`; unknown keys are opaque and preserved.

## Prose body (`slice-<id>.md`)

Pure prose — no frontmatter (it lives in the sister TOML), no embedded YAML. The
slice body is the **contract** (the WHAT and whether), not the design (the HOW):

1. **Context** — the situation; why this slice exists.
2. **Scope & Objectives** — what changes; the desired end state.
3. **Non-Goals** — explicit exclusions; the scope boundary.
4. **Summary** — a brief gesture at the how *and* how "done" is recognised; one
   paragraph of colour, deliberately not a task breakdown or a test list.
5. **Follow-Ups** — deferred work and tracking (supersede links, later slices).

`doctrine slice new` scaffolds the file with these headings and empty bodies.

### Division of labour with the design doc

A guiding rule (entity-model.md: avoid duplication between artifacts — duplication
breeds drift, and drift is the disease doctrine exists to kill): each fact lives
in exactly **one** artifact. The slice body and its design-doc sibling have a hard,
non-overlapping edge:

- **Slice body owns the WHAT** — context, scope, non-goals, summary, follow-ups.
- **Design doc owns the HOW** — approach, architecture, decisions, **risks**, and
  **validation design**. These were once duplicated in the slice body (`Approach`,
  `Risks`, `Verification`); they are removed from it so a design-doc review finding
  never has to be reconciled in two places.

A slice has a design doc **by default** — mandatory except for a trivial change,
and only with explicit user approval. A design-doc-less (trivial) slice writes
whatever the `Summary` and `Follow-Ups` need; it does not grow the HOW sections
back. The `Summary` is the one intentional overlap-by-altitude: a stable précis
that gestures at the how without tracking the design doc's findings.

This rule is, for now, a **convention no command enforces** — a slice can be
`ready` with no `design.md` and nothing surfaces it. Enforcement is deferred to a
future `doctrine slice validate`: a non-trivial slice must have a `design.md` or an
explicit trivial/no-design marker (a `slice-<id>.toml` field — anything queryable
lives in TOML, not prose, entity-model.md); validation never parses the prose for
headings (templates are defaults, not contracts). No gate ships with slice-003.

## CLI

`doctrine slice` is a new subcommand group, parallel to `doctrine install` and
`doctrine skills`. v1 scope is **new + list** only.

```
doctrine slice new [<title>] [--slug <slug>]
doctrine slice list [--status <status>]
```

### `doctrine slice new`

```
doctrine slice new "Add skill removal"            # title given, slug derived
doctrine slice new "Add skill removal" --slug rm  # explicit slug
doctrine slice new                                # prompts for title
```

Allocates the next id, creates `.doctrine/slice/<id>/`, writes
`slice-<id>.toml` (with `created`/`updated` set to today) and a scaffolded
`slice-<id>.md`, and creates
the `<id>-<slug>` symlink. Slug is derived from the title (lowercase, spaces →
hyphens, non-alphanumerics stripped) unless `--slug` is given. Prints the path
of the new slice.

The current date is supplied by the caller (no clock in the pure layer,
§ Architecture).

### `doctrine slice list`

Enumerates slices under `.doctrine/slice/`, ordered by id:

```
001  started      add-skill-removal   Add skill removal to doctrine skills
002  proposed     vendor-skills       Vendor skills instead of npx delegation
```

`--status` filters to one status. Output is id, status, slug, title — read from
each `slice-<id>.toml`.

## Architecture

Same split as `doctrine install` and `doctrine skills`: the CLI layer is thin and
dumb; all decisions live in pure functions over data.

| Pure (library, unit-tested)                                  | Imperative (thin shell)                |
|--------------------------------------------------------------|----------------------------------------|
| candidate id from a directory listing → `u32`                | the atomic `mkdir` claim + retry loop  |
| slug derivation from a title → `String`                      | read `.doctrine/slice/` entries        |
| scaffold plan (dir, toml bytes, md bytes, symlink) from inputs + date | resolve project root (shared walk-up) |
|                                                              | write files, create symlink            |
| `slice-<id>.toml` render / parse → struct                         | print plan / list / prompt for title   |
| list formatting (rows from parsed metadata) → `String`       | stat / read each `slice-<id>.toml`          |

The date and the directory listing are **inputs** to the pure layer, never read
inside it, so candidate-id computation and scaffolding are asserted without a
clock or disk. The atomic claim is the one piece that cannot be pure — only the
`mkdir` syscall arbitrates the race — so it lives in the shell, behind the same
IO seam, with the pure candidate function feeding it. Project-root detection
reuses the existing walk-up and `root_markers` (install-spec § Project-root
detection). Shared code.

## Lifecycle

`status ∈ {proposed, ready, started, audit, done, abandoned}` is recorded in
`slice-<id>.toml` and advanced by hand in v1. The stages track a slice from
intent to reconciled change:

- **proposed** — drafted; scope and motivation captured, not yet agreed.
- **ready** — accepted and scoped; cleared to start, work not begun.
- **started** — implementation under way.
- **audit** — code shipped; reconciling what shipped against the slice's
  declared scope. This is the *status* a slice carries while its audit (the
  `AUD-` artefact, glossary) is in progress — the stage produces the artefact;
  the two are not the same thing and do not share an id.
- **done** — reconciled and closed.
- **abandoned** — dropped before completion: the work was superseded by a later
  slice, descoped, or otherwise will not ship. A terminal state like `done` (it
  drops from the default `list`), but it carries no claim that the scope was
  delivered — only that the slice is no longer live. Distinct from `done`: an
  `abandoned` slice with incomplete phases is *consistent*, not divergent.

There is **no `complete` command and no closure gate** in v1 — there is nothing
to gate against until specs and verification artefacts exist, so transitions are
by hand and any value in the set is accepted. Note the seam is *entirely*
manual: v1 ships `new` (which always writes `proposed`) and `list`, and **no
transition verb** — the other five states are reached only by hand-editing
`slice-<id>.toml`. The vocabulary *is* enforced on the `list --status` filter
(an out-of-vocab `--status` is rejected; an out-of-vocab *stored* status is
never hidden and renders with a trailing `?` drift marker), but write-time
transitions remain ungated. The richer vocabulary is recorded now so the lifecycle stages
are deliberate, not retrofitted; gating attaches to them later. This is the
deliberate "one thing at a time" boundary.

## Forward compatibility

The shape anticipates the spec lifecycle without building it:

- **Spec linkage** attaches in `[relationships]` (specs / requirements). When
  present it becomes the source of truth for a slice's scope — the seam exists
  now, empty.
- **Audit / patch** (post-implementation reconciliation of specs against what
  shipped) attaches as a later lifecycle stage keyed off the same slice id; the
  directory holds its artefacts as new siblings.
- **DR / IP / phase siblings** (design revision, implementation plan, phase
  runsheets) can land as additional files in the slice directory if the workflow
  grows to need them. v1 keeps everything in `slice-<id>.md`.

None of these require restructuring the v1 artefact.

## Out of scope (v1)

- **Close / complete + coverage gates.** No spec registry to enforce against yet.
- **Spec / requirement linkage enforcement.** `[relationships]` is reserved but
  inert.
- **Audit / patch lifecycle.** Deferred to the spec work.
- **Edit / remove / re-slug.** Slices are created and listed; mutation is manual
  (edit `slice-<id>.toml` / `slice-<id>.md`, fix the symlink by hand) in v1.
- **Embedded structured YAML blocks.** doctrine uses the TOML sister instead;
  risks and context stay in prose until a structured need is proven.

## Known risks

- **Distributed id collision.** The `mkdir` claim (§ Id allocation) serialises
  only agents sharing one filesystem. Two agents in separate worktrees or clones
  (i.e. separate teams) can claim the same id and collide at merge. The fix — a
  shared reservation authority via lease — is specified separately and is not
  slice-specific (see reservation-spec). Until that lands, `mkdir` covers the
  single-working-tree case; the lease layer composes over it for the inter-team
  case without changing the on-disk shape here.
- **Symlink portability.** Slug symlinks are git-tracked but degrade on
  filesystems without symlink support (e.g. some Windows checkouts). The numeric
  directory is canonical, so tooling is unaffected; only the human alias is lost.
  Accepted for v1.
- **Stale symlink after manual re-slug.** Editing `slug` in `slice-<id>.toml` by hand
  does not move the symlink. v1 accepts drift; a future `re-slug` command
  reconciles.

## Open questions

1. **Reference token in prose.** On-disk id is bare numeric (`001`). Whether prose
   and commit messages should use a prefixed shorthand (e.g. `SL-001`) for
   greppability is unresolved.
2. **Where specs live.** Specs will not live in `doc/`; their home and their
   relationship-block schema are out of scope here and decided with the spec work.

## Testing

Unit tests (pure layer) cover:

- Candidate-id computation — empty dir ⇒ `001`; gaps and max selection; 3-digit
  padding, and width-grow past `999`.
- Slug derivation — title normalisation; `--slug` override.
- Scaffold plan — directory, `slice-<id>.toml` bytes (with injected date), `slice-<id>.md`
  headings, and symlink target, from given inputs.
- `slice-<id>.toml` round-trip — render then parse yields the same metadata; unknown
  keys preserved.
- List formatting — rows from parsed metadata; `--status` filter.

The atomic claim is exercised against the IO seam: an `EEXIST` on the first
`mkdir` drives a recompute-and-retry, and the loop is asserted to land the next
free id (a mock filesystem that fails the first claim, succeeds the second).

Imperative IO (mkdir, file writes, symlink creation, directory scan) sits behind
the same seam as `doctrine install` / `doctrine skills`, asserted without disk.
