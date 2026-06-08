# Uniform DRY CLI surface: shared list/show/filter/render contract

## Context

An agent-UX review of the `doctrine` CLI (preflight under this slice) found the
read/inspect surface — `list`, `show`, filtering, and output rendering — is
hand-rolled per entity kind rather than lifted onto the shared entity engine.
The result is a ragged surface that contradicts its own governance and forces
agents into the failure modes doctrine tells them to avoid.

The kinds (`slice`, `adr`, `spec`, `memory`, `backlog`) diverge along every read
axis. `backlog` (the newest, SL-020) is the closest to the intended shape and
serves as the de-facto reference implementation; the older kinds lag it.

The findings this slice addresses (F-numbers from the review):

- **F-1 — `show` gap breaks a documented invariant.** The boot guardrail says
  "read entities via `doctrine <kind> show <ID>`, not raw files," but `slice
  show` and `adr show` do not exist — the two most-cited kinds. An agent obeying
  doctrine hits `unrecognized subcommand`, then falls back to reading raw
  TOML+MD, which the same rule forbids. Self-contradicting governance.
- **F-2 — id form in `list` output is non-uniform and non-canonical.**
  `slice`/`adr`/`spec` emit bare `001`; `backlog` emits prefixed `ISS-001`;
  `memory` emits its full `mem_…` uid. The citation convention mandates the
  prefixed canonical id everywhere, so the bare forms are not copy-paste-correct.
- **F-3 — default `list` filtering diverges.** `backlog` hides terminal states
  by default (`--all` reveals); `slice`/`adr`/`spec`/`memory` dump everything,
  including `done`/`superseded`. The hide-terminal default is the saner one.
- **F-4 — filter surface is ragged.** `slice`/`adr`/`spec` accept only
  `--status`; `memory` adds `--type`/`--tag`; `backlog` adds a `[SUBSTR]`
  positional plus `--kind`/`--tag`/`--all`. No substring/title filter on the
  older kinds.
- **F-5 — output shape is non-uniform.** Header rows present on `slice`/`spec`,
  absent on `adr`/`backlog`/`memory`; column sets differ beyond what the data
  requires.
- **F-6 — no machine-readable output.** No `--json`/`--format` anywhere; agents
  parse whitespace columns, which is brittle. A single shared serializer is the
  highest-leverage agent-UX win.
- **F-7 — create-verb naming split.** `memory record` versus everyone else's
  `new`, breaking "same verb, every kind."

Root cause is singular: list/show/filter/render are not a shared contract on the
entity engine. The DRY remedy is to lift that contract once and make every kind
conform, rather than continue patching five bespoke surfaces.

## Scope & Objectives

Deliver a uniform, DRY read/inspect surface across all entity kinds by lifting a
shared contract onto the entity engine. Concretely:

1. **`show` everywhere (F-1).** Add `slice show <SL-NNN>` and `adr show
   <ADR-NNN>` so the boot guardrail becomes true for every kind. `show`
   reassembles the entity's TOML + MD tiers into one readable whole, matching the
   existing `spec`/`memory`/`backlog` `show` behaviour.
2. **Canonical prefixed ids in all output (F-2).** `list` and `show` emit the
   prefixed canonical id (`SL-025`, `ADR-001`, `PRD-001`) for every kind.
   `memory` is conformant-by-exception: its uid *is* its canonical id, so it
   keeps emitting `mem_…` (there is no short prefix-number form to switch to).
   The id prefix becomes a single engine-known property, not a hand-coded string
   per kind.
3. **Hide-terminal default + `--all` (F-3).** `slice`/`adr`/`spec`/`memory`
   `list` hide terminal states by default and reveal them under `--all` or an
   explicit `--status`, matching `backlog`. Each kind supplies a **list hide-set**
   predicate, kept *distinct* from any lifecycle/divergence-terminal predicate
   (`slice::is_terminal_status` `{done}` stays divergence-only and unchanged); the
   hide-set is the presentation axis consumed by the shared `retain`.
4. **Uniform filter surface (F-4).** A shared filter vocabulary across `list`,
   carried in a flattened `CommonListArgs` clap bundle: `--filter/-f` (substring
   on slug+title), `--regexp/-r` + `--case-insensitive/-i` (regex over
   canonical-id+slug+title), `--status/-s` (multi-value; any value reveals
   terminal), `--tag/-t` (repeatable, OR), `--all/-a`. Kind-specific axes
   (`memory --type`, `backlog --kind`) layer on the same base. Filter axes AND
   across each other. backlog's existing `[SUBSTR]` positional is retained as a
   deprecated alias of `--filter` (no break). Adds the `regex-lite` crate
   (decided explicitly — the repo was deliberately regex-free; `regex-lite`
   chosen over full `regex` for compile time + binary size).
5. **Uniform output shape + shared renderer (F-5, F-6).** One render path
   produces the human table (consistent header/column policy) and a `--json`
   (or `--format json`) machine-readable form, for `list` and `show`, across all
   kinds. The serializer lives once on the engine/render layer.
6. **Create-verb alignment (F-7).** `memory new` becomes the canonical create
   verb (uniform with every other kind) via `#[command(alias = "record")]`;
   `memory record` keeps working unchanged. One create verb across kinds, zero
   break to existing call sites in skills/scripts.
7. **Slice status vocabulary + boot consumer (F-N5/F-N2, from external review).**
   Amend `slices-spec.md` to the enforced set
   `{proposed, ready, started, audit, done, abandoned}` (`abandoned` replaces the
   out-of-spec `superseded`); enforce it as the `slice list --status` filter
   known-set via the shared `validate_statuses` (write-time/transition enforcement
   stays deferred — the lifecycle verb's job). Migrate the 2 live `superseded`
   slices to `abandoned`. `boot.rs` is a declared consumer of the refactored
   `list_rows`: its snapshot ADR/Memory sections adopt the new surface (prefixed
   `ADR-` ids + header; memory hide-default).

**Closure intent.** "Done" is judged by: `show` resolves for all five kinds;
every `list`/`show` emits canonical prefixed ids; default `list` hides terminal
across all kinds with `--all` revealing; a shared filter base and a shared
renderer (human + JSON) back every kind; create-verb naming reconciled. The
behaviour-preservation gate holds — existing engine suites stay green unchanged;
list-output snapshot tests change legitimately (bare→prefixed, fewer default
rows) and are updated as part of the work, not worked around.

## Non-Goals

- **Destructive / lifecycle-mutation verbs.** No `delete`, `archive`, or new
  status-transition machinery. The status-transition verbs stay ragged
  (`slice phase`, `adr status`, `backlog edit`, `memory verify`) — reconciling
  them, and the known slice-lifecycle-transition gap, is out of scope. (Follow-up.)
  (`--status` *filter*-validation IS in scope — it validates read input against the
  vocabulary, not stored-status writes/transitions.)
- **Boot snapshot memory *trim* (F-8).** SL-025 *does* update boot — its ADR/Memory
  sections render through the refactored `list_rows`, so they adopt the new surface.
  What stays out of scope is the heavier F-8 *trim* of the memory section
  (signpost-only / capped), a separate `boot`-render concern. (Follow-up.)
- **`find`/`retrieve` ranking semantics.** Memory's scope-ranked retrieval is a
  distinct surface; this slice does not touch its ranking or holdback.
- **New `--json` schema as a versioned external contract.** The machine-readable
  form is for agent consumption; whether it becomes a committed versioned schema
  is deferred unless design argues otherwise.

## Summary

Lift list/show/filter/render onto the shared entity engine so every kind exposes
the same read surface: `show` for all, canonical prefixed ids, hide-terminal
defaults with `--all`, a uniform filter vocabulary, and one renderer emitting
both a consistent human table and machine-readable JSON. `backlog` is the
reference shape; the work brings the other kinds up to it and factors the common
contract out of the five bespoke implementations. Status-transition uniformity
and destructive verbs are explicitly deferred.

## Follow-Ups

- **Destructive / lifecycle verbs.** A later slice for `delete`/`archive` and a
  uniform status-transition surface (covering the slice-lifecycle-transition
  gap). Capture as a backlog item.
- **F-8 boot memory render.** Trim the boot snapshot's memory section (signpost-
  only, or capped) on the `boot` render seam. Capture as a backlog item.
- **Standalone plan validation** (pre-existing CLI gap) is adjacent to a uniform
  `validate` verb but not pulled in here.
