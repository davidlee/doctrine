# IMP-133: CLI UX review — findings

Date: 2026-06-21
Method: Systematic audit of all 34 CLI verbs and their subcommands against
six dimensions: argument order consistency, help text clarity,
confirmation/echo on writes, discoverability, error message quality, and
symmetry with sibling verbs.

## Dimension 1: Argument ordering

### F-1 (major) — Top-level `needs`/`after` positional args are reversible

ISS-040 documents the core problem: `needs SL-060 SL-047` encodes "SOURCE
needs TARGET" (I depend on TARGET) but natural English reads "needs X" as X
is the thing needed (prerequisite). The verb is implied between two bare refs
whose order is not self-evident.

`after SL-060 SL-047` has the identical encoding: "SOURCE after TARGET"
(I run after TARGET) vs. natural "after X, do Y." Users can — and have —
authored the reverse edge by accident (SL-129 plan review, ISS-040
evidence).

Contrast with `link SL-048 governed_by ADR-010` — the label between operands
makes direction unambiguous.

The `backlog needs`/`backlog after` counterparts have the same encoding but
slightly better help text: `<ID>` (the dependent) `<PREREQS>` or `<TO>` (the
predecessor) — the role labels make direction clearer than bare `<SOURCE>
<TARGET>`.

**Existing mitigation:** The echo outputs resolve this locally — every
successful write prints the human-readable sentence (`"SL-060 needs SL-047"`,
`"SL-060 after SL-047"`). But this is post-hoc; the user already typed the
command before seeing confirmation.

**Recommendation:** Adopt the `backlog` pattern of role-labelled args in help
text (`<DEPENDENT> <PREDECESSOR>` instead of `<SOURCE> <TARGET>`) for the
top-level verbs. Add a confirmation echo on write (already exists). ISS-040
already proposes these.

### F-2 (minor) — `supersede` help lacks explicit directional phrasing

`supersede ADR-012 ADR-004` — the help says "NEW supersedes OLD" in the
prose intro but the usage line just shows `<NEW> <OLD>`. Adding a brief
directional note in the arg descriptions ("The superseding entity (newer)")
would reduce the 1-character-glance confusion risk.

### F-3 (nit) — `estimate set` dual-optional bounds hide the requirement

Usage: `doctrine estimate set <ID> [LOWER] [UPPER]`. Both `[LOWER]` and
`[UPPER]` are marked optional but are *only* optional when `-x/--exact` is
given. If neither `-x` nor both bounds are supplied, the command fails at
runtime with `"must supply both lower and upper, or -x/--exact"`.

The arg descriptions do say "omit with -x" but the framing is easily missed.
The `-x` flag should be mentioned in the usage line, or the dual-required
nature should be called out explicitly.

## Dimension 2: Help text clarity

### F-4 (major) — `concept-map add` and `concept-map remove` have no arg descriptions

```
Usage: doctrine concept-map add [OPTIONS] <ID> <SOURCE> <REL> <TARGET>

Arguments:
  <ID>
  <SOURCE>
  <REL>
  <TARGET>

Options:
  --force
```

Five positional arguments plus `--force` with zero description text.
A user calling `--help` learns nothing about what these arguments mean or
what `--force` does (it allows duplicate edges — a meaningful behaviour
with no discoverable explanation). `concept-map remove` has the same gap.
Every other command provides at least minimal descriptions. This is the
worst offender in the CLI surface.

### F-5 (minor) — Help text verbosity is inconsistent across verbs

- `link`/`unlink`: rich prose with §§ cross-references, idempotency docs,
  memory branch notation
- `needs`/`after`: moderate — cross-kind gate described, rank documented
- `backlog needs`/`backlog after`: short — just the function, no §§ refs
- `concept-map add`/`remove`: zero help (see F-4)
- `estimate set`: moderate
- `backlog tag`: moderate

The variance is not driven by complexity — `backlog needs` performs cycle
detection (a significant feature) but its help says "Validates every ref
exists, then refuses a closing dependency cycle" without explaining what a
"closing cycle" means or how the user should fix one. Compare to `needs`
(no cycle check) which has more prose.

### F-6 (nit) — `after` target marked `[TARGET]` (optional brackets) but required

`doctrine after [OPTIONS] <SOURCE> [TARGET]` — `[TARGET]` is shown as
optional (bracketed), but it's required unless `--prune` is set. The bracket
notation is technically correct per clap (conditional requirement) but adds
friction at a glance. The inner description says "Required unless --prune is
set" — this is helpful but easily overlooked.

## Dimension 3: Confirmation / echo on writes

### F-7 (nit) — Write echoes are consistent; no gaps found

All write-side verbs echo on success. Findings above are about the
*clarity* of those echoes (F-1), not their presence. The echo convention
is uniform and well-adhered-to:

| Verb | Echo form | Idempotency distinguished? |
|------|-----------|---------------------------|
| `link` | `linked: SRC LABEL TGT` / `already linked: ...` | yes |
| `unlink` | `unlinked: SRC LABEL TGT` / `not linked: ...` | yes |
| `needs` | `SRC needs TGT` | no (idempotent write is silent) |
| `after` | `SRC after TGT` / `(rank N)` / `removed (N edges)` | yes (remove path) |
| `after --remove` | `SRC after TGT removed (N edge{s})` | n/a |
| `supersede` | `NEW supersedes OLD` / `already superseded` | yes |
| `estimate set` | `estimate set: ID lower=X upper=Y` / `estimate unchanged: ...` | yes |
| `estimate clear` | `estimate cleared: ID` / `no estimate to clear: ID` | yes |
| `value set` | `value set: ID value=X` / `value unchanged: ...` | yes |
| `value clear` | `value cleared: ID` / `no value to clear: ID` | yes |
| `backlog tag` | `Tagged ID: tags` | no |
| `backlog needs` | `ID needs PREREQS` | no |
| `backlog after` | `ID after TO` / `(rank N)` / `removed (N edges)` | yes (remove) |
| `backlog edit` | `Edited ID: status · resolution` | no |
| `backlog new` | `Created ID: path` | n/a |

**Missing:** `needs`, `backlog needs`, and `backlog tag` don't distinguish
initial write from idempotent re-run (a user re-running `needs` gets the
same output as the first time). `link` and `unlink` set the gold standard
here ("already linked" / "not linked"). Low severity because these are
intentionally idempotent and re-running is harmless.

## Dimension 4: Discoverability

### F-8 (nit) — No top-level `help` for sub-subcommands from the top-level `--help`

`doctrine --help` shows top-level verbs with one-line descriptions. To
discover subcommands of, say, `memory`, you need `doctrine memory --help`.
This is standard clap behaviour and not broken, but the richness of
subcommands (11 for `memory`, 8 for `slice`, 10 for `review`, 9 for
`worktree`, etc.) means many verbs are invisible from the top-level glance.
No action needed — just noting the pattern for context.

### F-9 (nit) — `memory record` alias `memory new` is documented; no hidden verbs

The alias is explicitly called out in the help text: `"memory new" is the
uniform canonical alias (SL-025 §5.4 / D8); both names dispatch the
identical handler`. Good practice.

## Dimension 5: Error message quality

### F-10 (confirmed-good) — Error messages are specific and actionable

Sampled errors from testing:

- `"nonexistent" is not a canonical ref (expected e.g. SL-031)` — clear, includes expected format
- `"SL-999" does not resolve to an entity (no SL-999 at <path>)` — includes exact filesystem path
- `"entity not found: SL-999"` — terse but clear
- `"`{target}` is a {} entity — needs/after may only target work (a slice or a backlog item); cross-tier dep/seq is not yet allowed"` — specific, names the constraint
- Cycle detection: `"`backlog needs` would close a dependency cycle: {path} (nothing written)"` — describes consequence, affirms nothing was written
- F-1 pre-flight guards in supersede: names the exact malformed field and path; "the file is left untouched"

No low-hanging UX improvements found in error messages. The existing
messages are precise, include corrective guidance, and degrade gracefully.

## Dimension 6: Symmetry with sibling verbs

### F-11 (minor) — `needs` has no `--remove`/`--prune` counterpart

`after` supports `--remove` (remove a specific edge), `--prune` (drop
dangling edges), and `--rank`. `needs` has none of these; removing a hard
prerequisite requires hand-editing the TOML. This may be intentional (hard
deps should be deliberate), but the asymmetry isn't documented. If
intentional, a note in the `needs` help saying "Use `doctrine inspect` to
view existing edges; remove via TOML edit" would be helpful.

### F-12 (nit) — `estimate set` / `value set` vs their `clear` counterparts

`estimate set` takes `<ID> [LOWER] [UPPER]` + `-x/--exact`. `estimate clear`
takes `<ID>`. `value set` takes `<ID> <MAGNITUDE>`. `value clear` takes
`<ID>`. The set/clear symmetry is correct and expected. No issue.

### F-13 (minor) — `backlog needs` uses `canonical_id` echo; top-level `needs` uses raw ref

`backlog needs ISS-007 SL-132` echoes `ISS-007 needs SL-132` (canonical form).
Top-level `needs SL-060 SL-047` echoes `SL-060 needs SL-047` (raw input).
Both work, but the inconsistency between the backlog and top-level paths
is a papercut. The backlog path pads+prefixes; the top-level path echoes
exactly what was typed.

## Summary

| ID | Severity | Area | Title |
|----|----------|------|-------|
| F-1 | **major** | Arg ordering | `needs`/`after` positional args are reversible (ISS-040) |
| F-2 | minor | Help text | `supersede` lacks explicit directional help |
| F-3 | nit | Arg ordering | `estimate set` dual-optional bounds hide the requirement |
| F-4 | **major** | Help text | `concept-map add`/`remove` have zero arg descriptions |
| F-5 | minor | Help text | Help text verbosity inconsistent across verbs |
| F-6 | nit | Help text | `after` target bracketed as optional but required |
| F-7 | nit | Echo | `needs`/`backlog tag` don't distinguish idempotent re-run |
| F-8 | nit | Discoverability | Rich subcommands invisible from top-level `--help` |
| F-9 | nit | Discoverability | `memory record` alias documented — good practice noted |
| F-10 | — | Error messages | Error messages are specific and actionable — confirmed |
| F-11 | minor | Symmetry | `needs` has no `--remove`/`--prune` counterpart |
| F-12 | nit | Symmetry | `set`/`clear` asymmetry is correct and expected |
| F-13 | minor | Echo | `backlog needs` echo form differs from top-level `needs` |

---

## Second pass: Relation/edge surface audit

Date: 2026-06-21
Method: Full enumeration of all 23 relation labels in `RELATION_RULES`
table, cross-referenced against read verbs (`inspect`, `catalog graph`),
write verbs (`link`/`unlink` + typed verbs), and the `outbound_for`
dispatch in `src/catalog/scan.rs`. Edge type census from live catalog graph
(1004 edges across 1125 entities).

### Edge type inventory

| Label | Tier | Link policy | Write verb | Read path | Live count |
|-------|------|-------------|------------|-----------|------------|
| specs | One | Writable | `link` | inspect, catalog | 56 |
| requirements | One | Writable | `link` | inspect, catalog | 52 |
| supersedes (SL→SL) | One | Writable | `link` | inspect, catalog | 5 |
| supersedes (GOV→GOV) | One | LifecycleOnly | `supersede` | inspect, catalog | 0 |
| supersedes (REC→REC) | One | LifecycleOnly | `supersede` | inspect, catalog | 0 |
| descends_from | Typed | TypedVerbOnly | spec scaffold | inspect, catalog | 18 |
| parent | Typed | TypedVerbOnly | spec scaffold | inspect, catalog | 21 |
| members | Typed | TypedVerbOnly | `spec req add` | inspect, catalog | 321 |
| interactions | Typed | TypedVerbOnly | spec scaffold | inspect, catalog | 4 |
| contextualizes | One | Writable | `link` | **BLIND** | **0** |
| shapes | One | Writable | `link` | inspect, catalog | 0 |
| spawns | One | Writable | `link` | inspect, catalog | 0 |
| governed_by | One | Writable | `link` | inspect, catalog | 82 |
| consumes | One | Writable | `link` | inspect, catalog | 1 |
| slices | One | Writable | `link` | inspect, catalog | 67 |
| related (GOV) | One | Writable | `link` | inspect, catalog | 36 |
| related (SL+RFC) | One | Writable | `link` | inspect, catalog | 18 |
| reviews | Typed | TypedVerbOnly | `review new` | inspect, catalog | 120 |
| owning_slice | Typed | TypedVerbOnly | `rec new` | inspect, catalog | 31 |
| drift | One | Writable | `link` | inspect, catalog | 0 |
| decision_ref | Typed | TypedVerbOnly | `rec new` | inspect, catalog | 0 |
| revises | Typed | TypedVerbOnly | `revision change add` | inspect, catalog | 17 |
| originates_from | Typed | TypedVerbOnly | `revision new --originates-from` | inspect, catalog | 0 |

170 raw (pre-migration) edges also exist: `Raw(related)` (156) and
`Raw(descends_from)` (17).

### RF-1 (blocker) — Concept-map `contextualizes` edges are writable but invisible

`RELATION_RULES` declares `contextualizes` as `LinkPolicy::Writable` for
source kind `CM` (concept map). `doctrine link CM-001 contextualizes SL-047`
succeeds and writes to `[[relation]]` in `concept-map-001.toml`. But
`outbound_for` in `src/catalog/scan.rs` explicitly returns `Ok(Vec::new())`
for CM: `"REQ" | "CM" => Ok(Vec::new())`.

Consequence:
- `inspect CM-001` shows "(no relations)"
- `catalog graph` shows zero CM-sourced edges
- `doctrine validate` (which calls `validate_relations`) never sees these edges
- The edges exist on disk but are invisible to every read path

This is a spec/implementation mismatch: either the RELATION_RULES entry
should be removed (CM doesn't author Tier-1 relations through the link seam),
or `outbound_for` should dispatch to a concept-map relation reader.

The concept-map module stores its edges in a DSL string (`dsl = "..."`),
not in `[[relation]]` blocks. `concept-map add` writes to the DSL;
`link` writes to `[[relation]]`. These are parallel, disconnected storage
paths with no bridge or migration.

**Recommendation:** Either (a) remove the `contextualizes` entry from
RELATION_RULES if concept maps are intended to use only their DSL, or
(b) implement the bridge — read DSL edges into the relation graph and
migrate existing DSL edges to `[[relation]]`.

### RF-2 (major) — No corpus-level relation query verb

The only relation read surface is `inspect <ID>`, which shows outbound
and inbound edges for ONE entity. There is no verb for:

- **"Show all edges of type X"** — e.g., "list every governed_by edge
  in the corpus"
- **"Show all inbound edges to target Y"** — e.g., "which entities
  reference ADR-001?" (currently only discoverable by running
  `inspect ADR-001` and hoping you picked the right target)
- **"Show orphan entities"** — entities with zero outbound and zero
  inbound relations
- **"Show edge type census"** — the distribution I computed manually
  above via `catalog graph | python`

`catalog graph` emits the raw JSON but is marked "developer-facing; not
gating for acceptance" — it's not a user surface. `export lazyspec`
exists but only emits lazyspec format, not a human-readable relation
dashboard.

**Recommendation:** A `doctrine relation list [--label governed_by]
[--target ADR-001] [--source-kind SL]` verb, or extend `inspect` with
a `--corpus` flag that reports corpus-wide relation statistics.

### RF-3 (minor) — No relation-transitive walk

`blockers --transitive` walks dep edges transitively but there is no
equivalent for relations. You can't ask "show me everything shaped by
ASM-001, transitively" or "show all specs that descend from PRD-001,
transitively." The cordage graph already builds with relation overlays, so
the walk machinery exists. This could be a `--transitive` flag on `inspect`
(analogous to `blockers --transitive`).

### RF-4 (nit) — `doctrine validate` bundles relation validation silently

`validate_relations` in `relation_graph.rs` checks for dangling edges and
illegal rows. It IS called by `doctrine validate`. But there's no way to
run relation validation independently — it's bundled with id-integrity
checks. The verb doesn't advertise that relation health is included.
Adding a line to `doctrine validate --help` mentioning relation checks
would improve discoverability.

### RF-5 (minor) — `related` edges use Raw() label for pre-migration data

156 edges use `Raw(related)` instead of `Validated(Related)`. This is the
pre-migration form — these were authored before the PHASE-04 migration
that moved `related` to the validated `RELATION_RULES` table. The
`Raw(descends_from)` edges (17) are the same pattern. These carry no
functional difference (they resolve the same way), but their presence is a
signal that the migration is incomplete. A `doctrine validate` or
`doctrine doctor` finding for stale raw labels would surface this.

### RF-6 (confirmed-good) — Typed-edge write verbs are discoverable and complete

Every `TypedVerbOnly` edge has a dedicated, discoverable write verb:
- `descends_from` / `parent` / `interactions` → spec scaffold
- `members` → `spec req add`
- `reviews` → `review new --target`
- `owning_slice` / `decision_ref` → `rec new`
- `revises` → `revision change add`
- `originates_from` → `revision new --originates-from`

No gap. The typed verbs validate targets and produce consistent output.

### RF-7 (confirmed-good) — `inspect` is a strong single-entity read surface

`inspect <ID>` shows outbound relations (grouped by label) and derived
inbound relations (with reciprocal naming — `governed_by` → "governs",
`supersedes` → "superseded by", `originates_from` → "precursor of").
The rendering is clear, grouped, and uses the correct derived names from
RELATION_RULES. No findings on the inspect render.

---

## Summary (second pass)

| ID | Severity | Area | Title |
|----|----------|------|-------|
| RF-1 | **blocker** | Read gap | CM contextualizes writable but invisible (outbound_for returns empty) |
| RF-2 | major | Read gap | No corpus-level relation query verb |
| RF-3 | minor | Read gap | No relation-transitive walk (unlike blockers --transitive) |
| RF-4 | nit | Discoverability | validate bundles relation checks silently |
| RF-5 | minor | Data quality | 173 pre-migration Raw label edges (related, descends_from) |
| RF-6 | — | Write verbs | Typed-edge write verbs are discoverable and complete |
| RF-7 | — | Read verbs | inspect is a strong single-entity read surface |

---

## Merged severity summary (both passes)

| ID | Severity | Area | Title |
|----|----------|------|-------|
| F-1 | **major** | Arg ordering | needs/after positional args are reversible (ISS-040) |
| F-4 | **major** | Help text | concept-map add/remove have zero arg descriptions |
| RF-1 | **blocker** | Read gap | CM contextualizes writable but invisible |
| RF-2 | **major** | Read gap | No corpus-level relation query verb |
| F-2 | minor | Help text | supersede lacks explicit directional help |
| F-5 | minor | Help text | Help text verbosity inconsistent across verbs |
| F-11 | minor | Symmetry | needs has no --remove/--prune counterpart |
| F-13 | minor | Echo | backlog needs echo form differs from top-level needs |
| RF-3 | minor | Read gap | No relation-transitive walk |
| RF-5 | minor | Data quality | 173 pre-migration Raw label edges |
| F-3 | nit | Arg ordering | estimate set dual-optional bounds hide requirement |
| F-6 | nit | Help text | after target bracketed as optional but required |
| F-7 | nit | Echo | needs/backlog tag don't distinguish idempotent re-run |
| F-8 | nit | Discoverability | Rich subcommands invisible from top-level --help |
| F-9 | nit | Discoverability | memory record alias documented — good |
| F-12 | nit | Symmetry | set/clear asymmetry is correct and expected |
| RF-4 | nit | Discoverability | validate bundles relation checks silently |
| F-10 | — | Error messages | Error messages are specific and actionable — confirmed |
| RF-6 | — | Write verbs | Typed-edge write verbs are discoverable and complete |
| RF-7 | — | Read verbs | inspect is a strong single-entity read surface |

## Recommended next actions (updated)

1. **RF-1 (blocker)** — Fix the CM outbound_for gap or decide the design
   intent (CM-only DSL vs bridge to relation graph). The RELATION_RULES
   entry and the scan dispatch must agree.
2. **F-1 + F-4** (major) — ISS-040 (needs/after arg order) and concept-map
   help text.
3. **RF-2** (major) — Corpus-level relation query verb. Start with a
   `--label` and `--target` filter on a new `doctrine relation list`
   subcommand, or extend `catalog` with a human-readable mode.
4. **RF-5 + RF-4** (minor) — Surface raw-label edges as `validate` warnings
   and document relation validation in `validate --help`.
5. **F-5, F-11, F-13, RF-3** (minor) — Backlog for cleanup pass.

1. **Address F-1 and F-4 first** — these are the two major findings.
   F-1 (arg order) is already captured in ISS-040; F-4 (concept-map help)
   is a straightforward fix to add arg descriptions.
2. **F-5, F-11, F-13** (minor) — backlog for a cleanup pass.
3. **F-2, F-3, F-6, F-7** (nit) — low-cost improvements, pick up as
   drive-by fixes.
4. **F-10** — error messages are solid. No action needed here.
