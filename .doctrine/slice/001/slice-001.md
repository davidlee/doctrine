# Implement slices: doctrine slice new/list

## Summary & Context

Add the **slice** — doctrine's unit of intentional change — as a real,
scaffoldable entity. This slice implements the `doctrine slice` subcommand group
(v1: `new` + `list`) and the on-disk artefact shape, dogfooding the workflow on
doctrine itself (this very directory is its output).

Design is settled across three notes:

- `doc/slices-spec.md` — the entity: directory + sister TOML + prose, numeric id
  with slug symlink, `doctrine slice new/list`, pure/imperative split.
- `doc/reservation-spec.md` — collision-free id allocation. v1 uses the **local
  backend** only (the `mkdir`-claim); the `git-ref` backend is later.
- `doc/relation-index.md` — confirms no cache/index work is needed now.

## Motivation

Slices are the spine of the town-planner workflow: the declarative change bundle
that says *what changes and why* before code moves. Nothing else (lifecycle
gates, spec linkage, audit/patch) can land until the slice artefact and its
scaffolding exist. This is the first brick.

## Scope & Objectives

- `doctrine slice` subcommand group, parallel to `install` / `skills`.
- **`doctrine slice new [<title>] [--slug <slug>]`** — allocate the next id,
  create `.doctrine/slice/<id>/`, write `slice-<id>.toml` (created/updated = today)
  and a scaffolded `slice-<id>.md` from `install/templates/slice.{toml,md}`, and
  create the `<id>-<slug>` symlink. Print the new path.
- **`doctrine slice list [--status <s>]`** — enumerate slices by id: id, status,
  slug, title, read from each `slice-<id>.toml`.
- **Id allocation = local reservation** (reservation-spec § local backend): scan
  numeric dirs for `max + 1`, atomic `mkdir` claim, retry on `EEXIST`. Collision-
  free for concurrent local agents. 3-digit zero-pad (`001`).
- **Architecture**: pure planner (candidate id, slug derivation, toml/md render,
  list formatting) + thin IO shell (dir scan, the `mkdir` claim, file writes,
  symlink), reusing project-root detection and the file-copy/IO seam from
  `doctrine install` / `doctrine skills`.
- Templates ship via the installer (`.doctrine/templates/slice.{toml,md}`;
  `.doctrine/slice/` added to `manifest.toml` `[dirs]`).

## Out of Scope

- **`git-ref` reservation backend** and the `LeaseBackend` trait beyond the local
  case — reservation-spec, later.
- **Transient leasing / write-gating / lifecycle close + coverage gates** —
  deferred with the spec registry they depend on.
- **Spec / requirement linkage** — `[relationships]` stays inert (slices-spec).
- **Relation index / cache** — not needed at current scale (relation-index note).
- **Edit / remove / re-slug** commands — manual in v1.

## Approach

Mirror the existing subcommands. A pure library module decides everything from
data — `(title, slug-override, dir-listing, today) → scaffold plan` — and a thin
clap/IO shell executes it. The one impure point is the atomic `mkdir` claim
(only the syscall arbitrates the id race); it sits behind the same IO seam used
by `install`/`skills`, so the planner is unit-tested without disk or clock.
Scaffolding is template-copy + token substitution (`{{id}}`, `{{slug}}`,
`{{title}}`, `{{date}}`) — the shape lives once, in the templates, not duplicated
in Rust render code.

## Done

- `doctrine slice new` creates a well-formed slice (dir, sister toml, scaffolded
  md, symlink) with a reservation-allocated id; re-runs never collide.
- `doctrine slice list` enumerates with `--status` filter.
- Unit tests per slices-spec § Testing (candidate-id incl. EEXIST→retry, slug
  derivation, scaffold plan, toml round-trip, list formatting) — passing.
- Installer ships the templates and creates `.doctrine/slice/`.
- Lint clean (zero warnings), formatted.

## Risks & Follow-ups

- **Distributed id collision** across worktrees/clones — closed later by the
  `git-ref` backend (reservation-spec § Known risks). Local `mkdir` covers the
  single-tree case now.
- **Symlink portability** on filesystems without symlink support — numeric dir is
  canonical, alias degrades only (slices-spec § Known risks).
- **Reference token** (`SL-001` vs bare `001`) and lifecycle commands
  (`complete`) — open, tracked in slices-spec.
